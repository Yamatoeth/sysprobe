use crate::history::HistoryReport;
use crate::snapshot::SystemSnapshot;

pub fn to_json(snapshot: &SystemSnapshot) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(snapshot)?)
}

pub fn write_json_file(
    path: impl AsRef<std::path::Path>,
    snapshot: &SystemSnapshot,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, to_json(snapshot)?)?;
    Ok(())
}

pub fn history_to_json(report: &HistoryReport) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn history_from_json(contents: &str) -> anyhow::Result<HistoryReport> {
    Ok(serde_json::from_str(contents)?)
}

pub fn write_history_file(
    path: impl AsRef<std::path::Path>,
    report: &HistoryReport,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, history_to_json(report)?)?;
    Ok(())
}
