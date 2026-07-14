use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Borders, Sparkline};

use crate::history::HistoryBuffer;

pub fn cpu_sparkline_data(history: &HistoryBuffer) -> Vec<u64> {
    history
        .cpu_history()
        .into_iter()
        .map(|value| value.clamp(0.0, 100.0).round() as u64)
        .collect()
}

pub fn cpu_sparkline<'a>(data: &'a [u64]) -> Sparkline<'a> {
    Sparkline::default()
        .block(Block::default().title("CPU history").borders(Borders::ALL))
        .data(data)
        .max(100)
        .bar_set(symbols::bar::NINE_LEVELS)
        .style(Style::default().fg(Color::Cyan))
}
