use std::collections::VecDeque;

use crate::snapshot::SystemSnapshot;

#[derive(Debug, Clone)]
pub struct HistoryBuffer {
    capacity: usize,
    snapshots: VecDeque<SystemSnapshot>,
}

impl HistoryBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            snapshots: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, snapshot: SystemSnapshot) {
        if self.capacity == 0 {
            return;
        }

        if self.snapshots.len() == self.capacity {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    pub fn snapshots(&self) -> &VecDeque<SystemSnapshot> {
        &self.snapshots
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn cpu_history(&self) -> Vec<f32> {
        self.cpu_history_last(self.capacity)
    }

    pub fn cpu_history_last(&self, points: usize) -> Vec<f32> {
        self.last_points(points)
            .map(|snapshot| snapshot.cpu.global_usage_percent)
            .collect()
    }

    pub fn memory_history(&self) -> Vec<f32> {
        self.memory_history_last(self.capacity)
    }

    pub fn memory_history_last(&self, points: usize) -> Vec<f32> {
        self.last_points(points)
            .map(|snapshot| snapshot.memory.used_percent)
            .collect()
    }

    pub fn network_rx_history(&self) -> Vec<u64> {
        self.network_rx_history_last(self.capacity)
    }

    pub fn network_rx_history_last(&self, points: usize) -> Vec<u64> {
        self.last_points(points)
            .map(|snapshot| snapshot.network.rx_bytes_per_sec)
            .collect()
    }

    pub fn network_tx_history(&self) -> Vec<u64> {
        self.network_tx_history_last(self.capacity)
    }

    pub fn network_tx_history_last(&self, points: usize) -> Vec<u64> {
        self.last_points(points)
            .map(|snapshot| snapshot.network.tx_bytes_per_sec)
            .collect()
    }

    fn last_points(&self, points: usize) -> impl Iterator<Item = &SystemSnapshot> {
        let skip = self.snapshots.len().saturating_sub(points);
        self.snapshots.iter().skip(skip)
    }
}

#[cfg(test)]
mod tests {
    use super::HistoryBuffer;
    use crate::snapshot::{
        CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, SystemSnapshot,
    };

    #[test]
    fn evicts_oldest_snapshot_when_capacity_is_exceeded() {
        let mut history = HistoryBuffer::new(2);

        history.push(snapshot(10.0));
        history.push(snapshot(20.0));
        history.push(snapshot(30.0));

        assert_eq!(history.len(), 2);
        assert_eq!(history.cpu_history(), vec![20.0, 30.0]);
    }

    #[test]
    fn returns_last_n_points_for_metrics() {
        let mut history = HistoryBuffer::new(4);

        history.push(snapshot(10.0));
        history.push(snapshot(20.0));
        history.push(snapshot(30.0));

        assert_eq!(history.cpu_history_last(2), vec![20.0, 30.0]);
        assert_eq!(history.memory_history_last(2), vec![40.0, 60.0]);
        assert_eq!(history.network_rx_history_last(2), vec![200, 300]);
        assert_eq!(history.network_tx_history_last(2), vec![400, 600]);
    }

    #[test]
    fn zero_capacity_buffer_keeps_no_snapshots() {
        let mut history = HistoryBuffer::new(0);

        history.push(snapshot(10.0));

        assert!(history.is_empty());
        assert_eq!(history.cpu_history(), Vec::<f32>::new());
    }

    fn snapshot(cpu_percent: f32) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: chrono::Utc::now(),
            cpu: CpuInfo {
                global_usage_percent: cpu_percent,
                per_core_usage_percent: vec![cpu_percent],
                load_avg_1: 0.0,
            },
            memory: MemoryInfo {
                total_bytes: 100,
                used_bytes: cpu_percent as u64,
                used_percent: cpu_percent * 2.0,
                swap_used_bytes: 0,
            },
            disks: vec![DiskInfo {
                mount_point: "/".to_owned(),
                total_bytes: 100,
                used_bytes: cpu_percent as u64,
                used_percent: cpu_percent,
            }],
            network: NetworkInfo {
                rx_bytes_per_sec: (cpu_percent * 10.0) as u64,
                tx_bytes_per_sec: (cpu_percent * 20.0) as u64,
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
