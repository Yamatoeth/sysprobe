use std::collections::HashMap;

use crate::snapshot::SystemSnapshot;

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRule {
    pub name: String,
    pub metric: MetricKind,
    pub threshold_percent: f32,
    pub recovery_threshold_percent: f32,
    pub sustained_ticks: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricKind {
    Cpu,
    Memory,
    Disk(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertEvent {
    Triggered {
        rule_name: String,
        value: f32,
        at: chrono::DateTime<chrono::Utc>,
    },
    Resolved {
        rule_name: String,
        at: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AlertEngine {
    rules: Vec<AlertRule>,
    states: HashMap<String, AlertState>,
}

#[derive(Debug, Clone, Default)]
struct AlertState {
    consecutive_over_threshold: u32,
    active: bool,
}

impl AlertEngine {
    pub fn new(rules: Vec<AlertRule>) -> Self {
        let states = rules
            .iter()
            .map(|rule| (rule.name.clone(), AlertState::default()))
            .collect();

        Self { rules, states }
    }

    pub fn evaluate(&mut self, snapshot: &SystemSnapshot) -> Vec<AlertEvent> {
        let mut events = Vec::new();

        for rule in &self.rules {
            let Some(value) = metric_value(rule, snapshot) else {
                continue;
            };

            let state = self.states.entry(rule.name.clone()).or_default();
            if value >= rule.threshold_percent {
                state.consecutive_over_threshold += 1;
                if !state.active && state.consecutive_over_threshold >= rule.sustained_ticks.max(1)
                {
                    state.active = true;
                    events.push(AlertEvent::Triggered {
                        rule_name: rule.name.clone(),
                        value,
                        at: snapshot.timestamp,
                    });
                }
            } else if value <= rule.recovery_threshold_percent {
                state.consecutive_over_threshold = 0;
                if state.active {
                    state.active = false;
                    events.push(AlertEvent::Resolved {
                        rule_name: rule.name.clone(),
                        at: snapshot.timestamp,
                    });
                }
            } else if !state.active {
                state.consecutive_over_threshold = 0;
            }
        }

        events
    }
}

fn metric_value(rule: &AlertRule, snapshot: &SystemSnapshot) -> Option<f32> {
    match &rule.metric {
        MetricKind::Cpu => Some(snapshot.cpu.global_usage_percent),
        MetricKind::Memory => Some(snapshot.memory.used_percent),
        MetricKind::Disk(mount_point) => snapshot
            .disks
            .iter()
            .find(|disk| disk.mount_point == *mount_point)
            .map(|disk| disk.used_percent),
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertEngine, AlertEvent, AlertRule, MetricKind};
    use crate::snapshot::{
        CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, SystemSnapshot,
    };

    #[test]
    fn triggers_after_sustained_ticks() {
        let mut engine = AlertEngine::new(vec![cpu_rule(80.0, 3)]);

        assert!(engine.evaluate(&snapshot(81.0, 10.0, 10.0)).is_empty());
        assert!(engine.evaluate(&snapshot(82.0, 10.0, 10.0)).is_empty());
        let events = engine.evaluate(&snapshot(83.0, 10.0, 10.0));

        assert!(matches!(
            events.as_slice(),
            [AlertEvent::Triggered { rule_name, value, .. }]
            if rule_name == "cpu-high" && (*value - 83.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn does_not_trigger_on_isolated_spike() {
        let mut engine = AlertEngine::new(vec![cpu_rule(80.0, 2)]);

        assert!(engine.evaluate(&snapshot(90.0, 10.0, 10.0)).is_empty());
        assert!(engine.evaluate(&snapshot(20.0, 10.0, 10.0)).is_empty());
        assert!(engine.evaluate(&snapshot(90.0, 10.0, 10.0)).is_empty());
    }

    #[test]
    fn emits_resolution_when_metric_drops_below_threshold() {
        let mut engine = AlertEngine::new(vec![cpu_rule(80.0, 1)]);

        assert_eq!(engine.evaluate(&snapshot(90.0, 10.0, 10.0)).len(), 1);
        let events = engine.evaluate(&snapshot(20.0, 10.0, 10.0));

        assert!(matches!(
            events.as_slice(),
            [AlertEvent::Resolved { rule_name, .. }] if rule_name == "cpu-high"
        ));
    }

    #[test]
    fn keeps_alert_active_until_recovery_threshold() {
        let mut engine = AlertEngine::new(vec![AlertRule {
            name: "cpu-high".to_owned(),
            metric: MetricKind::Cpu,
            threshold_percent: 80.0,
            recovery_threshold_percent: 70.0,
            sustained_ticks: 1,
        }]);

        assert_eq!(engine.evaluate(&snapshot(90.0, 10.0, 10.0)).len(), 1);
        assert!(engine.evaluate(&snapshot(75.0, 10.0, 10.0)).is_empty());

        let events = engine.evaluate(&snapshot(70.0, 10.0, 10.0));
        assert!(matches!(
            events.as_slice(),
            [AlertEvent::Resolved { rule_name, .. }] if rule_name == "cpu-high"
        ));
    }

    #[test]
    fn evaluates_memory_and_disk_rules() {
        let mut engine = AlertEngine::new(vec![
            AlertRule {
                name: "memory-high".to_owned(),
                metric: MetricKind::Memory,
                threshold_percent: 70.0,
                recovery_threshold_percent: 70.0,
                sustained_ticks: 1,
            },
            AlertRule {
                name: "disk-high".to_owned(),
                metric: MetricKind::Disk("/".to_owned()),
                threshold_percent: 90.0,
                recovery_threshold_percent: 90.0,
                sustained_ticks: 1,
            },
        ]);

        let events = engine.evaluate(&snapshot(10.0, 75.0, 95.0));

        assert_eq!(events.len(), 2);
    }

    fn cpu_rule(threshold_percent: f32, sustained_ticks: u32) -> AlertRule {
        AlertRule {
            name: "cpu-high".to_owned(),
            metric: MetricKind::Cpu,
            threshold_percent,
            recovery_threshold_percent: threshold_percent,
            sustained_ticks,
        }
    }

    fn snapshot(cpu_percent: f32, memory_percent: f32, disk_percent: f32) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: chrono::Utc::now(),
            cpu: CpuInfo {
                global_usage_percent: cpu_percent,
                per_core_usage_percent: vec![cpu_percent],
                load_avg_1: 0.0,
            },
            memory: MemoryInfo {
                total_bytes: 100,
                used_bytes: memory_percent as u64,
                used_percent: memory_percent,
                swap_used_bytes: 0,
            },
            disks: vec![DiskInfo {
                mount_point: "/".to_owned(),
                total_bytes: 100,
                used_bytes: disk_percent as u64,
                used_percent: disk_percent,
            }],
            network: NetworkInfo {
                rx_bytes_per_sec: 0,
                tx_bytes_per_sec: 0,
                active_connections: 0,
            },
            top_processes: vec![ProcessInfo {
                pid: 1,
                name: "test".to_owned(),
                cpu_percent,
                memory_bytes: 1,
            }],
        }
    }
}
