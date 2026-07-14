use crate::collectors::Collector;
use crate::snapshot::CpuInfo;

pub struct CpuCollector;

impl Collector for CpuCollector {
    type Output = CpuInfo;

    fn collect(&mut self, sys: &mut sysinfo::System) -> anyhow::Result<Self::Output> {
        Ok(CpuInfo {
            global_usage_percent: sys.global_cpu_usage(),
            per_core_usage_percent: sys.cpus().iter().map(sysinfo::Cpu::cpu_usage).collect(),
            load_avg_1: sysinfo::System::load_average().one,
        })
    }
}
