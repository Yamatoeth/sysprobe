use std::process::Command;

#[test]
fn json_snapshot_contains_expected_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_sysprobe"))
        .arg("--json")
        .output()
        .expect("failed to run sysprobe --json");

    assert!(
        output.status.success(),
        "sysprobe --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");

    for field in [
        "timestamp",
        "cpu",
        "memory",
        "disks",
        "network",
        "top_processes",
    ] {
        assert!(json.get(field).is_some(), "missing field: {field}");
    }

    assert!(json["cpu"].get("global_usage_percent").is_some());
    assert!(json["cpu"].get("per_core_usage_percent").is_some());
    assert!(json["cpu"].get("load_avg_1").is_some());
    assert!(json["memory"].get("total_bytes").is_some());
    assert!(json["memory"].get("used_bytes").is_some());
    assert!(json["memory"].get("used_percent").is_some());
    assert!(json["memory"].get("swap_used_bytes").is_some());
    assert!(json["network"].get("rx_bytes_per_sec").is_some());
    assert!(json["network"].get("tx_bytes_per_sec").is_some());
    assert!(json["network"].get("active_connections").is_some());
}
