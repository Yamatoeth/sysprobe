use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, help = "Emit the snapshot as JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    #[command(about = "Collect a single system snapshot")]
    Snapshot(SnapshotArgs),
    #[command(about = "Run the live terminal dashboard")]
    Watch(WatchArgs),
    #[command(about = "Run the background polling daemon")]
    Daemon(DaemonArgs),
    #[command(about = "Dump recent history")]
    History(HistoryArgs),
    #[command(about = "Show top processes")]
    Top(TopArgs),
}

#[derive(Debug, Clone, Default, clap::Args)]
pub struct SnapshotArgs {
    #[arg(long, help = "Emit the snapshot as JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct WatchArgs {
    #[arg(long, default_value_t = 2, help = "Refresh interval in seconds")]
    pub interval: u64,
}

#[derive(Debug, Clone, clap::Args)]
pub struct DaemonArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Path to a sysprobe TOML config file"
    )]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct HistoryArgs {
    #[arg(long, default_value = "1h", help = "History window to dump")]
    pub last: String,

    #[arg(
        long,
        value_name = "PATH",
        default_value = "./sysprobe_history.json",
        help = "History JSON file written by `sysprobe daemon`"
    )]
    pub file: PathBuf,

    #[arg(long, default_value_t = 1, help = "Seconds between live samples")]
    pub interval: u64,

    #[arg(
        long,
        default_value_t = 5,
        help = "Maximum live samples to collect for this command"
    )]
    pub samples: usize,

    #[arg(long, help = "Emit the history report as JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct TopArgs {
    #[arg(long, value_enum, default_value_t = TopSortKey::Cpu, help = "Process sort key")]
    pub by: TopSortKey,

    #[arg(
        short = 'n',
        default_value_t = 10,
        help = "Number of processes to show"
    )]
    pub limit: usize,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TopSortKey {
    Cpu,
    Memory,
}
