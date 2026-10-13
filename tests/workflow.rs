use std::process::Command;

#[test]
fn bracket_dimensions_rebuild_and_stl_export() {
    let dir = tempfile::tempdir().unwrap();
    let document = dir.path().join("bracket.cadstrike");
    let resized = dir.path().join("wider.cadstrike");
    let stl = dir.path().join("bracket.stl");
    let create = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .args(["new-bracket", "80", "50", "10", "6"])
        .arg(&document)
        .output()
        .unwrap();
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let info = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("info")
        .arg(&document)
        .output()
        .unwrap();
    assert!(info.status.success());
    let value: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(value["volume_mm3"], 7200.0);
    assert_eq!(value["bounds_max"], serde_json::json!([80.0, 50.0, 6.0]));
    let change = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("set-width")
        .arg(&document)
        .arg(&resized)
        .arg("90")
        .output()
        .unwrap();
    assert!(
        change.status.success(),
        "{}",
        String::from_utf8_lossy(&change.stderr)
    );
    let info = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("info")
        .arg(&resized)
        .output()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(value["volume_mm3"], 7800.0);
    let export = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("export-stl")
        .arg(&resized)
        .arg(&stl)
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let bytes = std::fs::read(stl).unwrap();
    assert_eq!(
        bytes.len(),
        84 + u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize * 50
    );
}

#[test]
fn invalid_bracket_is_rejected_without_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.cadstrike");
    let create = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .args(["new-bracket", "80", "50", "80", "6"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!create.status.success());
    assert!(!path.exists());
}

#[test]
fn disk_round_trip_is_manifold_and_export_preserves_mm_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let document = dir.path().join("disk.cadstrike");
    let stl = dir.path().join("disk.stl");
    let create = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .args(["new-disk", "25", "8"])
        .arg(&document)
        .output()
        .unwrap();
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let info = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("info")
        .arg(&document)
        .output()
        .unwrap();
    assert!(info.status.success());
    let value: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(value["manifold"], true);
    assert_eq!(value["bounds_min"], serde_json::json!([-25.0, -25.0, 0.0]));
    assert_eq!(value["bounds_max"], serde_json::json!([25.0, 25.0, 8.0]));
    let volume = value["volume_mm3"].as_f64().unwrap();
    let exact = std::f64::consts::PI * 25.0 * 25.0 * 8.0;
    let triangles = value["triangles"].as_u64().unwrap() as usize;
    let facets = (triangles + 4) / 4;
    let sagitta_mm = 25.0 * (1.0 - (std::f64::consts::PI / facets as f64).cos());
    assert!(sagitta_mm <= 0.05, "{sagitta_mm} mm chord error");
    assert!((volume / exact - 1.0).abs() < 0.003);
    let export = Command::new(env!("CARGO_BIN_EXE_cadstrike"))
        .arg("export-stl")
        .arg(&document)
        .arg(&stl)
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(std::fs::metadata(stl).unwrap().len() > 84);
}
