#![allow(dead_code)]

mod alert;
mod cli;
mod collectors;
mod config;
mod daemon;
mod error;
mod export;
mod history;
mod snapshot;
mod ui;

use clap::Parser;
use sysinfo::System;

use crate::cli::{Cli, Command, SnapshotArgs, TopSortKey};
use crate::collectors::processes::{ProcessCollector, TopProcessSort};
use crate::collectors::Collector;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli
        .command
        .unwrap_or(Command::Snapshot(SnapshotArgs { json: cli.json }))
    {
        Command::Snapshot(args) => run_snapshot(args.json || cli.json)?,
        Command::Watch(args) => run_watch(args.interval)?,
        Command::Daemon(args) => run_daemon(args.config)?,
        Command::History(args) => run_history(&args.last),
        Command::Top(args) => run_top(args.by, args.limit)?,
    }

    Ok(())
}

fn run_snapshot(json: bool) -> anyhow::Result<()> {
    let mut sys = System::new_all();
    let snapshot = collectors::collect_all(&mut sys)?;

    if json {
        println!("{}", export::to_json(&snapshot)?);
    } else {
        println!("{}", ui::table::render_snapshot_table(&snapshot));
    }

    Ok(())
}

fn run_watch(interval: u64) -> anyhow::Result<()> {
    ui::dashboard::run_dashboard(interval)
}

fn run_daemon(config: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(daemon::run_daemon(config.as_deref()))
}

fn run_history(last: &str) {
    println!("placeholder: dispatch history command (last={last})");
    todo!("dump recent history");
}

fn run_top(by: TopSortKey, limit: usize) -> anyhow::Result<()> {
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut collector = ProcessCollector::new(by.into(), limit);
    let processes = collector.collect(&mut sys)?;
    println!(
        "{}",
        ui::table::render_process_table(&processes, "top processes")
    );

    Ok(())
}

impl From<TopSortKey> for TopProcessSort {
    fn from(value: TopSortKey) -> Self {
        match value {
            TopSortKey::Cpu => Self::Cpu,
            TopSortKey::Memory => Self::Memory,
        }
    }
}
