use crate::collectors::Collector;
use crate::snapshot::MemoryInfo;

pub struct MemoryCollector;

impl Collector for MemoryCollector {
    type Output = MemoryInfo;

    fn collect(&mut self, sys: &mut sysinfo::System) -> anyhow::Result<Self::Output> {
        let total_bytes = sys.total_memory();
        let used_bytes = sys.used_memory();
        let used_percent = if total_bytes == 0 {
            0.0
        } else {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        };

        Ok(MemoryInfo {
            total_bytes,
            used_bytes,
            used_percent,
            swap_used_bytes: sys.used_swap(),
        })
    }
}
