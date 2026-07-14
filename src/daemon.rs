use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use sysinfo::System;
use tokio::time;
use tracing::{error, info, warn};

use crate::alert::{AlertEngine, AlertEvent, AlertRule, MetricKind};
use crate::config::Config;
use crate::history::{HistoryBuffer, HistoryReport};

pub async fn run_daemon(config_path: Option<&Path>) -> anyhow::Result<()> {
    init_tracing();

    let config = match config_path {
        Some(path) => Config::load_from_file(path)
            .with_context(|| format!("failed to load config from {}", path.display()))?,
        None => Config::default(),
    };

    let mut sys = System::new_all();
    let mut history = HistoryBuffer::new(config.general.history_capacity);
    let mut alerts = AlertEngine::new(alert_rules_from_config(&config));
    let mut interval = time::interval(Duration::from_secs(
        config.general.refresh_interval_secs.max(1),
    ));

    info!(
        interval_secs = config.general.refresh_interval_secs.max(1),
        history_capacity = config.general.history_capacity,
        json_output_path = %config.export.json_output_path,
        history_output_path = %config.export.history_output_path,
        "sysprobe daemon started"
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match crate::collectors::collect_all(&mut sys) {
                    Ok(snapshot) => {
                        history.push(snapshot.clone());
                        for event in alerts.evaluate(&snapshot) {
                            log_alert_event(&event);
                        }

                        if let Err(error) = crate::export::write_json_file(
                            &config.export.json_output_path,
                            &snapshot,
                        ) {
                            error!(error = %error, "failed to write snapshot JSON");
                        }

                        let report = HistoryReport::from_history(
                            "daemon",
                            config.general.refresh_interval_secs.max(1),
                            &history,
                        );
                        if let Err(error) = crate::export::write_history_file(
                            &config.export.history_output_path,
                            &report,
                        ) {
                            error!(error = %error, "failed to write history JSON");
                        }
                    }
                    Err(error) => {
                        error!(error = %error, "failed to collect system snapshot");
                    }
                }
            }
            result = tokio::signal::ctrl_c() => {
                match result {
                    Ok(()) => info!("shutdown signal received"),
                    Err(error) => warn!(error = %error, "failed to listen for shutdown signal"),
                }
                break;
            }
        }
    }

    info!(history_len = history.len(), "sysprobe daemon stopped");
    Ok(())
}

fn alert_rules_from_config(config: &Config) -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "cpu-high".to_owned(),
            metric: MetricKind::Cpu,
            threshold_percent: config.alerts.cpu_percent_max,
            recovery_threshold_percent: recovery_threshold(
                config.alerts.cpu_percent_max,
                config.alerts.recovery_margin_percent,
            ),
            sustained_ticks: config.alerts.sustained_ticks,
        },
        AlertRule {
            name: "memory-high".to_owned(),
            metric: MetricKind::Memory,
            threshold_percent: config.alerts.mem_percent_max,
            recovery_threshold_percent: recovery_threshold(
                config.alerts.mem_percent_max,
                config.alerts.recovery_margin_percent,
            ),
            sustained_ticks: config.alerts.sustained_ticks,
        },
        AlertRule {
            name: "root-disk-high".to_owned(),
            metric: MetricKind::Disk("/".to_owned()),
            threshold_percent: config.alerts.disk_percent_max,
            recovery_threshold_percent: recovery_threshold(
                config.alerts.disk_percent_max,
                config.alerts.recovery_margin_percent,
            ),
            sustained_ticks: config.alerts.sustained_ticks,
        },
    ]
}

fn recovery_threshold(threshold_percent: f32, recovery_margin_percent: f32) -> f32 {
    (threshold_percent - recovery_margin_percent.max(0.0)).clamp(0.0, threshold_percent)
}

fn log_alert_event(event: &AlertEvent) {
    match event {
        AlertEvent::Triggered {
            rule_name,
            value,
            at,
        } => warn!(rule = %rule_name, value, at = %at, "alert triggered"),
        AlertEvent::Resolved { rule_name, at } => {
            info!(rule = %rule_name, at = %at, "alert resolved");
        }
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sysprobe=info".into()),
        )
        .try_init();
}
