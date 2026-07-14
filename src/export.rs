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
