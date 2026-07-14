use std::io::{self, IsTerminal, Stdout};
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Row, Table};
use ratatui::Terminal;
use sysinfo::System;

use crate::collectors;
use crate::config::Config;
use crate::history::HistoryBuffer;
use crate::snapshot::{ProcessInfo, SystemSnapshot};
use crate::ui::sparkline::{cpu_sparkline, cpu_sparkline_data};
use crate::ui::table::render_snapshot_table;

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn run_dashboard(interval_secs: u64) -> anyhow::Result<()> {
    if !io::stdout().is_terminal() {
        return run_text_fallback("stdout is not an interactive terminal");
    }

    let mut terminal = match TerminalGuard::enter() {
        Ok(guard) => guard,
        Err(error) => {
            restore_terminal();
            eprintln!("sysprobe watch: TUI unavailable ({error}); falling back to text output");
            return run_text_fallback("TUI initialization failed");
        }
    };

    let mut sys = System::new_all();
    let mut history = HistoryBuffer::new(Config::default().general.history_capacity);
    let interval = Duration::from_secs(interval_secs.max(1));

    loop {
        let snapshot = collectors::collect_all(&mut sys).context("failed to collect snapshot")?;
        history.push(snapshot.clone());
        terminal
            .draw(|frame| draw_dashboard(frame, &snapshot, &history))
            .context("failed to render dashboard")?;

        if should_quit_before_next_tick(interval)? {
            break;
        }
    }

    Ok(())
}

fn draw_dashboard(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &SystemSnapshot,
    history: &HistoryBuffer,
) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(8),
        ])
        .split(frame.area());

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[0]);

    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("CPU").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .label(format!("{:.1}%", snapshot.cpu.global_usage_percent))
            .ratio(percent_ratio(snapshot.cpu.global_usage_percent)),
        gauges[0],
    );

    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("RAM").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Blue))
            .label(format!("{:.1}%", snapshot.memory.used_percent))
            .ratio(percent_ratio(snapshot.memory.used_percent)),
        gauges[1],
    );

    let sparkline_data = cpu_sparkline_data(history);
    frame.render_widget(cpu_sparkline(&sparkline_data), root[1]);
    frame.render_widget(process_table(&snapshot.top_processes), root[2]);
}

fn process_table(processes: &[ProcessInfo]) -> Table<'_> {
    let header = Row::new(["PID", "Name", "CPU %", "Memory"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows = processes.iter().map(|process| {
        Row::new([
            Cell::from(process.pid.to_string()),
            Cell::from(process.name.clone()),
            Cell::from(format!("{:.1}", process.cpu_percent)),
            Cell::from(format_bytes(process.memory_bytes)),
        ])
    });

    Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title("Top processes")
            .borders(Borders::ALL),
    )
}

fn should_quit_before_next_tick(interval: Duration) -> anyhow::Result<bool> {
    let deadline = Instant::now() + interval;

    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(false);
        }

        if event::poll(deadline.saturating_duration_since(now))? {
            match event::read()? {
                Event::Key(key) if key.code == KeyCode::Char('q') => return Ok(true),
                Event::Key(key)
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return Ok(true);
                }
                _ => {}
            }
        }
    }
}

fn run_text_fallback(reason: &str) -> anyhow::Result<()> {
    let mut sys = System::new_all();
    let snapshot = collectors::collect_all(&mut sys)?;
    eprintln!("sysprobe watch fallback: {reason}");
    println!("{}", render_snapshot_table(&snapshot));
    Ok(())
}

fn percent_ratio(value: f32) -> f64 {
    f64::from(value.clamp(0.0, 100.0)) / 100.0
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

struct TerminalGuard {
    terminal: TuiTerminal,
    previous_hook: SharedPanicHook,
}

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide).context("failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("failed to initialize terminal backend")?;
        let previous_hook = install_panic_restore_hook();

        Ok(Self {
            terminal,
            previous_hook,
        })
    }
}

impl std::ops::Deref for TerminalGuard {
    type Target = TuiTerminal;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl std::ops::DerefMut for TerminalGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
        if let Some(previous_hook) = self
            .previous_hook
            .lock()
            .ok()
            .and_then(|mut hook| hook.take())
        {
            panic::set_hook(previous_hook);
        }
    }
}

type PanicHook = Box<dyn Fn(&panic::PanicHookInfo<'_>) + Sync + Send + 'static>;
type SharedPanicHook = Arc<Mutex<Option<PanicHook>>>;

fn install_panic_restore_hook() -> SharedPanicHook {
    let previous_hook = panic::take_hook();
    let previous_hook = Arc::new(Mutex::new(Some(previous_hook)));
    let hook_for_panic = Arc::clone(&previous_hook);

    panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        eprintln!("sysprobe watch restored the terminal after a panic");
        if let Ok(hook) = hook_for_panic.lock() {
            if let Some(previous_hook) = hook.as_ref() {
                previous_hook(panic_info);
            }
        }
    }));

    previous_hook
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, Show, LeaveAlternateScreen);
}
