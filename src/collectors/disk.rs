use crate::collectors::Collector;
use crate::snapshot::DiskInfo;

pub struct DiskCollector;

impl Collector for DiskCollector {
    type Output = Vec<DiskInfo>;

    fn collect(&mut self, _sys: &mut sysinfo::System) -> anyhow::Result<Self::Output> {
        let mut disks = sysinfo::Disks::new_with_refreshed_list();
        disks.refresh();

        Ok(disks
            .iter()
            .map(|disk| {
                let total_bytes = disk.total_space();
                let used_bytes = total_bytes.saturating_sub(disk.available_space());
                let used_percent = if total_bytes == 0 {
                    0.0
                } else {
                    (used_bytes as f32 / total_bytes as f32) * 100.0
                };

                DiskInfo {
                    mount_point: disk.mount_point().to_string_lossy().into_owned(),
                    total_bytes,
                    used_bytes,
                    used_percent,
                }
            })
            .collect())
    }
}
