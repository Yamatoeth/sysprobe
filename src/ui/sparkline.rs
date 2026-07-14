use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Borders, Sparkline};

use crate::history::HistoryBuffer;

pub fn cpu_sparkline_data(history: &HistoryBuffer) -> Vec<u64> {
    percent_data(history.cpu_history())
}

pub fn memory_sparkline_data(history: &HistoryBuffer) -> Vec<u64> {
    percent_data(history.memory_history())
}

pub fn network_sparkline_data(history: &HistoryBuffer) -> Vec<u64> {
    history
        .network_rx_history()
        .into_iter()
        .zip(history.network_tx_history())
        .map(|(rx, tx)| rx.saturating_add(tx))
        .collect()
}

pub fn percent_sparkline<'a>(title: &'a str, data: &'a [u64], color: Color) -> Sparkline<'a> {
    Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(data)
        .max(100)
        .bar_set(symbols::bar::NINE_LEVELS)
        .style(Style::default().fg(color))
}

pub fn network_sparkline<'a>(data: &'a [u64]) -> Sparkline<'a> {
    let max = data.iter().copied().max().unwrap_or(1).max(1);

    Sparkline::default()
        .block(
            Block::default()
                .title("Network history")
                .borders(Borders::ALL),
        )
        .data(data)
        .max(max)
        .bar_set(symbols::bar::NINE_LEVELS)
        .style(Style::default().fg(Color::Magenta))
}

fn percent_data(values: Vec<f32>) -> Vec<u64> {
    values
        .into_iter()
        .map(|value| value.clamp(0.0, 100.0).round() as u64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{memory_sparkline_data, network_sparkline_data};
    use crate::history::HistoryBuffer;
    use crate::snapshot::{
        CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, SystemSnapshot,
    };

    #[test]
    fn sparkline_data_includes_memory_and_network_history() {
        let mut history = HistoryBuffer::new(2);
        history.push(snapshot(40.4, 10, 20));
        history.push(snapshot(51.6, 30, 40));

        assert_eq!(memory_sparkline_data(&history), vec![40, 52]);
        assert_eq!(network_sparkline_data(&history), vec![30, 70]);
    }

    fn snapshot(memory_percent: f32, rx: u64, tx: u64) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: chrono::Utc::now(),
            cpu: CpuInfo {
                global_usage_percent: 10.0,
                per_core_usage_percent: vec![10.0],
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
                used_bytes: 10,
                used_percent: 10.0,
            }],
            network: NetworkInfo {
                rx_bytes_per_sec: rx,
                tx_bytes_per_sec: tx,
                active_connections: 0,
            },
            top_processes: vec![ProcessInfo {
                pid: 1,
                name: "test".to_owned(),
                cpu_percent: 10.0,
                memory_bytes: 1,
            }],
        }
    }
}
