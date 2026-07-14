use std::collections::VecDeque;
use std::time::Duration;

use crate::snapshot::SystemSnapshot;

#[derive(Debug, Clone)]
pub struct HistoryBuffer {
    capacity: usize,
    snapshots: VecDeque<SystemSnapshot>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryReport {
    pub requested_window: String,
    pub sample_interval_secs: u64,
    pub samples: Vec<SystemSnapshot>,
    pub summary: HistorySummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistorySummary {
    pub sample_count: usize,
    pub cpu_percent: MetricSummaryF32,
    pub memory_percent: MetricSummaryF32,
    pub network_rx_bytes_per_sec: MetricSummaryU64,
    pub network_tx_bytes_per_sec: MetricSummaryU64,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct MetricSummaryF32 {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct MetricSummaryU64 {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
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

impl HistoryReport {
    pub fn from_history(
        requested_window: impl Into<String>,
        sample_interval_secs: u64,
        history: &HistoryBuffer,
    ) -> Self {
        let samples: Vec<_> = history.snapshots().iter().cloned().collect();
        let summary = HistorySummary::from_samples(&samples);

        Self {
            requested_window: requested_window.into(),
            sample_interval_secs,
            samples,
            summary,
        }
    }

    pub fn from_samples(
        requested_window: impl Into<String>,
        sample_interval_secs: u64,
        samples: Vec<SystemSnapshot>,
    ) -> Self {
        let summary = HistorySummary::from_samples(&samples);

        Self {
            requested_window: requested_window.into(),
            sample_interval_secs,
            samples,
            summary,
        }
    }

    pub fn for_window(&self, requested_window: impl Into<String>, window: Duration) -> Self {
        let cutoff = chrono::Duration::from_std(window)
            .map(|duration| chrono::Utc::now() - duration)
            .unwrap_or_else(|_| chrono::Utc::now());
        let samples = self
            .samples
            .iter()
            .filter(|snapshot| snapshot.timestamp >= cutoff)
            .cloned()
            .collect();

        Self::from_samples(requested_window, self.sample_interval_secs, samples)
    }
}

impl HistorySummary {
    fn from_samples(samples: &[SystemSnapshot]) -> Self {
        Self {
            sample_count: samples.len(),
            cpu_percent: summarize_f32(
                samples
                    .iter()
                    .map(|snapshot| snapshot.cpu.global_usage_percent),
            ),
            memory_percent: summarize_f32(
                samples.iter().map(|snapshot| snapshot.memory.used_percent),
            ),
            network_rx_bytes_per_sec: summarize_u64(
                samples
                    .iter()
                    .map(|snapshot| snapshot.network.rx_bytes_per_sec),
            ),
            network_tx_bytes_per_sec: summarize_u64(
                samples
                    .iter()
                    .map(|snapshot| snapshot.network.tx_bytes_per_sec),
            ),
        }
    }
}

pub fn parse_window_duration(input: &str) -> anyhow::Result<Duration> {
    let trimmed = input.trim();
    let Some(unit) = trimmed.chars().last() else {
        anyhow::bail!("history window cannot be empty");
    };

    let number = &trimmed[..trimmed.len() - unit.len_utf8()];
    let value: u64 = number
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid history window: {input}"))?;

    let seconds = match unit {
        's' | 'S' => value,
        'm' | 'M' => value.saturating_mul(60),
        'h' | 'H' => value.saturating_mul(60 * 60),
        _ => anyhow::bail!("history window must end with s, m, or h: {input}"),
    };

    if seconds == 0 {
        anyhow::bail!("history window must be greater than zero");
    }

    Ok(Duration::from_secs(seconds))
}

pub fn sample_count_for_window(window: Duration, interval: Duration, max_samples: usize) -> usize {
    if max_samples == 0 {
        return 0;
    }

    let interval_secs = interval.as_secs().max(1);
    let window_secs = window.as_secs().max(1);
    let samples = window_secs.div_ceil(interval_secs).max(1) as usize;

    samples.min(max_samples)
}

pub fn render_history_report(report: &HistoryReport) -> String {
    let mut output = String::new();
    output.push_str("sysprobe history\n");
    output.push_str(&format!(
        "window: {}  interval: {}s  samples: {}\n\n",
        report.requested_window, report.sample_interval_secs, report.summary.sample_count
    ));
    output.push_str(&format!(
        "  {:<24} {:>10} {:>10} {:>10}\n",
        "metric", "min", "avg", "max"
    ));
    append_f32_summary(&mut output, "cpu %", report.summary.cpu_percent);
    append_f32_summary(&mut output, "memory %", report.summary.memory_percent);
    append_u64_summary(
        &mut output,
        "network rx bytes/s",
        report.summary.network_rx_bytes_per_sec,
    );
    append_u64_summary(
        &mut output,
        "network tx bytes/s",
        report.summary.network_tx_bytes_per_sec,
    );

    output
}

fn summarize_f32(values: impl Iterator<Item = f32>) -> MetricSummaryF32 {
    let mut count = 0usize;
    let mut sum = 0.0f32;
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;

    for value in values {
        count += 1;
        sum += value;
        min = min.min(value);
        max = max.max(value);
    }

    if count == 0 {
        return MetricSummaryF32 {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        };
    }

    MetricSummaryF32 {
        min,
        max,
        avg: sum / count as f32,
    }
}

fn summarize_u64(values: impl Iterator<Item = u64>) -> MetricSummaryU64 {
    let mut count = 0usize;
    let mut sum = 0u128;
    let mut min = u64::MAX;
    let mut max = 0u64;

    for value in values {
        count += 1;
        sum += u128::from(value);
        min = min.min(value);
        max = max.max(value);
    }

    if count == 0 {
        return MetricSummaryU64 {
            min: 0,
            max: 0,
            avg: 0,
        };
    }

    MetricSummaryU64 {
        min,
        max,
        avg: (sum / count as u128) as u64,
    }
}

fn append_f32_summary(output: &mut String, label: &str, summary: MetricSummaryF32) {
    output.push_str(&format!(
        "  {:<24} {:>9.1} {:>9.1} {:>9.1}\n",
        label, summary.min, summary.avg, summary.max
    ));
}

fn append_u64_summary(output: &mut String, label: &str, summary: MetricSummaryU64) {
    output.push_str(&format!(
        "  {:<24} {:>10} {:>10} {:>10}\n",
        label, summary.min, summary.avg, summary.max
    ));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        parse_window_duration, render_history_report, sample_count_for_window, HistoryBuffer,
        HistoryReport,
    };
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

    #[test]
    fn parses_history_window_duration() {
        assert_eq!(
            parse_window_duration("30s").unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            parse_window_duration("5m").unwrap(),
            Duration::from_secs(300)
        );
        assert_eq!(
            parse_window_duration("2h").unwrap(),
            Duration::from_secs(7200)
        );
        assert!(parse_window_duration("10d").is_err());
        assert!(parse_window_duration("0s").is_err());
    }

    #[test]
    fn caps_sample_count_for_large_windows() {
        let samples =
            sample_count_for_window(Duration::from_secs(3600), Duration::from_secs(2), 30);

        assert_eq!(samples, 30);
    }

    #[test]
    fn renders_history_summary() {
        let mut history = HistoryBuffer::new(2);
        history.push(snapshot(10.0));
        history.push(snapshot(30.0));

        let report = HistoryReport::from_history("1h", 2, &history);
        let output = render_history_report(&report);

        assert!(output.contains("samples: 2"));
        assert!(output.contains("cpu %"));
        assert_eq!(report.summary.cpu_percent.min, 10.0);
        assert_eq!(report.summary.cpu_percent.max, 30.0);
        assert_eq!(report.summary.cpu_percent.avg, 20.0);
    }

    #[test]
    fn filters_report_to_requested_window() {
        let old = snapshot_at(10.0, chrono::Utc::now() - chrono::Duration::hours(2));
        let recent = snapshot_at(30.0, chrono::Utc::now());
        let report = HistoryReport::from_samples("daemon", 2, vec![old, recent]);

        let filtered = report.for_window("1h", Duration::from_secs(60 * 60));

        assert_eq!(filtered.requested_window, "1h");
        assert_eq!(filtered.summary.sample_count, 1);
        assert_eq!(filtered.summary.cpu_percent.avg, 30.0);
    }

    fn snapshot(cpu_percent: f32) -> SystemSnapshot {
        snapshot_at(cpu_percent, chrono::Utc::now())
    }

    fn snapshot_at(cpu_percent: f32, timestamp: chrono::DateTime<chrono::Utc>) -> SystemSnapshot {
        SystemSnapshot {
            timestamp,
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
