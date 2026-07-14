# sysprobe

Terminal system intelligence for local machines: snapshots, process ranking, a live TUI dashboard, alerting, daemon exports, and lightweight history.

`sysprobe` is a Rust CLI built to make system state inspectable from a terminal without pulling in a full monitoring stack. It focuses on fast local feedback: CPU, memory, disk, network, top processes, threshold alerts, JSON output, and a dashboard suitable for demos or day-to-day checks.

## Status

Implemented:

- One-shot system snapshots in table or JSON format
- CPU, memory, disk, network, and top-process collectors
- `top` command with CPU or memory sorting
- Live TUI dashboard with CPU/RAM gauges, history sparklines, network panel, and process table
- In-memory history and `history` reports from live samples or daemon-written JSON
- Configurable alert engine with sustained thresholds and recovery hysteresis
- Daemon mode that writes latest snapshot JSON and rolling history JSON
- Linux `/proc/net/tcp` parsing for active TCP connection counts

Planned polish:

- Add a dashboard screenshot or GIF to this README
- Package/release binaries

## Demo Preview

Add a dashboard GIF or screenshot here after recording locally.

Suggested capture:

```bash
cargo run -- watch
```

## Install

Requirements:

- Rust stable toolchain
- macOS or Linux

Build from source:

```bash
git clone https://github.com/Yamatoeth/sysprobe.git
cd sysprobe
cargo build --release
```

Run from source:

```bash
cargo run -- --help
```

## Usage

Collect a readable snapshot:

```bash
cargo run -- snapshot
```

Emit JSON:

```bash
cargo run -- --json
```

Show top processes:

```bash
cargo run -- top --by cpu -n 10
cargo run -- top --by memory -n 10
```

Run the TUI dashboard:

```bash
cargo run -- watch --interval 2
```

Run daemon mode with the example config:

```bash
cargo run -- daemon --config config/sysprobe.example.toml
```

Read history from the daemon output file:

```bash
cargo run -- history --last 1h --file ./sysprobe_history.json
```

If no history file exists, `history` falls back to a short live sampling window:

```bash
cargo run -- history --last 30s --samples 5 --interval 1
```

Emit history JSON:

```bash
cargo run -- history --last 1h --json
```

## Configuration

See [config/sysprobe.example.toml](config/sysprobe.example.toml).

Key settings:

- `general.refresh_interval_secs`: daemon sampling interval
- `general.history_capacity`: rolling history size
- `alerts.*_percent_max`: alert thresholds
- `alerts.sustained_ticks`: consecutive samples required before triggering
- `alerts.recovery_margin_percent`: hysteresis margin before resolving
- `export.json_output_path`: latest snapshot JSON path
- `export.history_output_path`: rolling history JSON path

## Why This Project

`sysprobe` is intentionally small but covers several production-relevant concerns:

- Systems programming: local metrics, process inspection, and Linux `/proc` parsing
- CLI product design: table output for humans and JSON output for automation
- Terminal UI: live dashboard with bounded state and graceful fallback outside a TTY
- Reliability patterns: config parsing, alert hysteresis, daemon output, and tests
- Rust practice: typed snapshots, focused modules, no unsafe code, and CI checks

It is designed as a portfolio project that demonstrates practical Rust beyond toy algorithms.

## Development

Run the full local quality gate:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Useful commands:

```bash
cargo check
cargo run -- --json
cargo run -- history --last 5s --samples 1
```
