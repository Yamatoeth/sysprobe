use crate::snapshot::{DiskInfo, ProcessInfo, SystemSnapshot};

pub fn render_snapshot_table(snapshot: &SystemSnapshot) -> String {
    let mut output = String::new();

    output.push_str("sysprobe snapshot\n");
    output.push_str(&format!("timestamp: {}\n\n", snapshot.timestamp));
    output.push_str("system\n");
    output.push_str(&format!(
        "  {:<18} {:>10.1}%\n",
        "cpu", snapshot.cpu.global_usage_percent
    ));
    output.push_str(&format!(
        "  {:<18} {:>10.1}% ({}/{})\n",
        "memory",
        snapshot.memory.used_percent,
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes)
    ));
    output.push_str(&format!(
        "  {:<18} rx {:>10}/s  tx {:>10}/s  connections {:>4}\n\n",
        "network",
        format_bytes(snapshot.network.rx_bytes_per_sec),
        format_bytes(snapshot.network.tx_bytes_per_sec),
        snapshot.network.active_connections
    ));

    output.push_str("disks\n");
    output.push_str(&format!(
        "  {:<32} {:>12} {:>12} {:>8}\n",
        "mount", "used", "total", "used %"
    ));
    for disk in &snapshot.disks {
        append_disk_row(&mut output, disk);
    }

    output.push('\n');
    output.push_str(&render_process_table(
        &snapshot.top_processes,
        "top processes",
    ));

    output
}

pub fn render_process_table(processes: &[ProcessInfo], title: &str) -> String {
    let mut output = String::new();

    output.push_str(title);
    output.push('\n');
    output.push_str(&format!(
        "  {:>8}  {:<28} {:>8} {:>12}\n",
        "pid", "name", "cpu %", "memory"
    ));
    for process in processes {
        append_process_row(&mut output, process);
    }

    output
}

fn append_disk_row(output: &mut String, disk: &DiskInfo) {
    output.push_str(&format!(
        "  {:<32} {:>12} {:>12} {:>7.1}%\n",
        truncate(&disk.mount_point, 32),
        format_bytes(disk.used_bytes),
        format_bytes(disk.total_bytes),
        disk.used_percent
    ));
}

fn append_process_row(output: &mut String, process: &ProcessInfo) {
    output.push_str(&format!(
        "  {:>8}  {:<28} {:>7.1}% {:>12}\n",
        process.pid,
        truncate(&process.name, 28),
        process.cpu_percent,
        format_bytes(process.memory_bytes)
    ));
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = UNITS[0];

    for next_unit in UNITS.iter().skip(1) {
        if size < 1024.0 {
            break;
        }
        size /= 1024.0;
        unit = next_unit;
    }

    if unit == UNITS[0] {
        format!("{bytes} {unit}")
    } else {
        format!("{size:.1} {unit}")
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() {
        truncated
    } else {
        value.to_owned()
    }
}
