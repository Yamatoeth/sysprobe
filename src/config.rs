use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub alerts: AlertConfig,
    pub export: ExportConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub refresh_interval_secs: u64,
    pub history_capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertConfig {
    pub cpu_percent_max: f32,
    pub mem_percent_max: f32,
    pub disk_percent_max: f32,
    pub sustained_ticks: u32,
    #[serde(default = "default_recovery_margin_percent")]
    pub recovery_margin_percent: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportConfig {
    pub json_output_path: String,
    #[serde(default = "default_history_output_path")]
    pub history_output_path: String,
}

impl Config {
    pub fn load_from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path.as_ref()) {
            Ok(contents) => Self::from_toml_str(&contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn from_toml_str(contents: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(contents)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                refresh_interval_secs: 2,
                history_capacity: 120,
            },
            alerts: AlertConfig {
                cpu_percent_max: 90.0,
                mem_percent_max: 90.0,
                disk_percent_max: 95.0,
                sustained_ticks: 3,
                recovery_margin_percent: 5.0,
            },
            export: ExportConfig {
                json_output_path: "./sysprobe_snapshot.json".to_owned(),
                history_output_path: "./sysprobe_history.json".to_owned(),
            },
        }
    }
}

fn default_recovery_margin_percent() -> f32 {
    5.0
}

fn default_history_output_path() -> String {
    "./sysprobe_history.json".to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::Config;

    #[test]
    fn load_missing_file_returns_defaults() {
        let path = unique_temp_path("missing");

        let config = Config::load_from_file(&path).expect("missing config should use defaults");

        assert_eq!(config, Config::default());
    }

    #[test]
    fn parses_valid_toml() {
        let config = Config::from_toml_str(
            r#"
[general]
refresh_interval_secs = 5
history_capacity = 42

[alerts]
cpu_percent_max = 80.5
mem_percent_max = 81.5
disk_percent_max = 82.5
sustained_ticks = 4
recovery_margin_percent = 6.5

[export]
json_output_path = "/tmp/sysprobe.json"
history_output_path = "/tmp/sysprobe_history.json"
"#,
        )
        .expect("valid TOML should parse");

        assert_eq!(config.general.refresh_interval_secs, 5);
        assert_eq!(config.general.history_capacity, 42);
        assert_eq!(config.alerts.cpu_percent_max, 80.5);
        assert_eq!(config.alerts.mem_percent_max, 81.5);
        assert_eq!(config.alerts.disk_percent_max, 82.5);
        assert_eq!(config.alerts.sustained_ticks, 4);
        assert_eq!(config.alerts.recovery_margin_percent, 6.5);
        assert_eq!(config.export.json_output_path, "/tmp/sysprobe.json");
        assert_eq!(
            config.export.history_output_path,
            "/tmp/sysprobe_history.json"
        );
    }

    #[test]
    fn invalid_toml_returns_error() {
        let error = Config::from_toml_str("not valid toml =").expect_err("invalid TOML must fail");

        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn missing_recovery_margin_uses_default() {
        let config = Config::from_toml_str(
            r#"
[general]
refresh_interval_secs = 5
history_capacity = 42

[alerts]
cpu_percent_max = 80.5
mem_percent_max = 81.5
disk_percent_max = 82.5
sustained_ticks = 4

[export]
json_output_path = "/tmp/sysprobe.json"
"#,
        )
        .expect("valid TOML should parse");

        assert_eq!(config.alerts.recovery_margin_percent, 5.0);
        assert_eq!(config.export.history_output_path, "./sysprobe_history.json");
    }

    #[test]
    fn load_existing_file_parses_toml() {
        let path = unique_temp_path("valid");
        fs::write(
            &path,
            r#"
[general]
refresh_interval_secs = 3
history_capacity = 9

[alerts]
cpu_percent_max = 70.0
mem_percent_max = 75.0
disk_percent_max = 90.0
sustained_ticks = 2
recovery_margin_percent = 4.0

[export]
json_output_path = "./out.json"
history_output_path = "./history.json"
"#,
        )
        .expect("failed to write temp config");

        let config = Config::load_from_file(&path).expect("existing config should parse");
        fs::remove_file(&path).expect("failed to remove temp config");

        assert_eq!(config.general.refresh_interval_secs, 3);
        assert_eq!(config.general.history_capacity, 9);
    }

    fn unique_temp_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("sysprobe-{label}-{nanos}.toml"))
    }
}
