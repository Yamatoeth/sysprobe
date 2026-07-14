use crate::collectors::Collector;
use crate::snapshot::ProcessInfo;

#[derive(Debug, Clone, Copy)]
pub enum TopProcessSort {
    Cpu,
    Memory,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessCollector {
    sort_by: TopProcessSort,
    limit: usize,
}

impl ProcessCollector {
    pub fn new(sort_by: TopProcessSort, limit: usize) -> Self {
        Self { sort_by, limit }
    }
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new(TopProcessSort::Cpu, 10)
    }
}

impl Collector for ProcessCollector {
    type Output = Vec<ProcessInfo>;

    fn collect(&mut self, sys: &mut sysinfo::System) -> anyhow::Result<Self::Output> {
        Ok(sort_and_limit_processes(
            collect_processes(sys),
            self.sort_by,
            self.limit,
        ))
    }
}

pub fn collect_processes(sys: &sysinfo::System) -> Vec<ProcessInfo> {
    sys.processes()
        .values()
        .map(|process| ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cpu_percent: process.cpu_usage(),
            memory_bytes: process.memory(),
        })
        .collect()
}

pub fn sort_and_limit_processes(
    mut processes: Vec<ProcessInfo>,
    sort_by: TopProcessSort,
    limit: usize,
) -> Vec<ProcessInfo> {
    match sort_by {
        TopProcessSort::Cpu => processes.sort_by(|left, right| {
            right
                .cpu_percent
                .total_cmp(&left.cpu_percent)
                .then_with(|| left.pid.cmp(&right.pid))
        }),
        TopProcessSort::Memory => processes.sort_by(|left, right| {
            right
                .memory_bytes
                .cmp(&left.memory_bytes)
                .then_with(|| left.pid.cmp(&right.pid))
        }),
    }

    processes.truncate(limit);
    processes
}

#[cfg(test)]
mod tests {
    use super::{sort_and_limit_processes, TopProcessSort};
    use crate::snapshot::ProcessInfo;

    #[test]
    fn sorts_by_cpu_descending_and_limits() {
        let sorted = sort_and_limit_processes(mock_processes(), TopProcessSort::Cpu, 2);

        let pids: Vec<_> = sorted.into_iter().map(|process| process.pid).collect();
        assert_eq!(pids, vec![2, 3]);
    }

    #[test]
    fn sorts_by_memory_descending_and_limits() {
        let sorted = sort_and_limit_processes(mock_processes(), TopProcessSort::Memory, 2);

        let pids: Vec<_> = sorted.into_iter().map(|process| process.pid).collect();
        assert_eq!(pids, vec![1, 3]);
    }

    fn mock_processes() -> Vec<ProcessInfo> {
        vec![
            ProcessInfo {
                pid: 1,
                name: "memory-heavy".to_owned(),
                cpu_percent: 1.0,
                memory_bytes: 900,
            },
            ProcessInfo {
                pid: 2,
                name: "cpu-heavy".to_owned(),
                cpu_percent: 90.0,
                memory_bytes: 100,
            },
            ProcessInfo {
                pid: 3,
                name: "balanced".to_owned(),
                cpu_percent: 45.0,
                memory_bytes: 500,
            },
        ]
    }
}
