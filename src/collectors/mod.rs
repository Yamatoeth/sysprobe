pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod network_proc;
pub mod processes;

use crate::collectors::cpu::CpuCollector;
use crate::collectors::disk::DiskCollector;
use crate::collectors::memory::MemoryCollector;
use crate::collectors::network::NetworkCollector;
use crate::collectors::processes::ProcessCollector;
use crate::snapshot::SystemSnapshot;

pub trait Collector {
    type Output;

    fn collect(&mut self, sys: &mut sysinfo::System) -> anyhow::Result<Self::Output>;
}

pub fn collect_all(sys: &mut sysinfo::System) -> anyhow::Result<SystemSnapshot> {
    sys.refresh_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let cpu = CpuCollector.collect(sys)?;
    let memory = MemoryCollector.collect(sys)?;
    let disks = DiskCollector.collect(sys)?;
    let network = NetworkCollector.collect(sys)?;
    let top_processes = ProcessCollector::default().collect(sys)?;

    Ok(SystemSnapshot {
        timestamp: chrono::Utc::now(),
        cpu,
        memory,
        disks,
        network,
        top_processes,
    })
}
