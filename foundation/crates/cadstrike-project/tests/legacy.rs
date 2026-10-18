use cadstrike_project::{LegacyProfile, MAX_LEGACY_BYTES, import_legacy_sketch, read_legacy_v1};
use serde_json::json;

fn bracket() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema_version": 1,
        "units": "mm",
        "sketch_id": 1,
        "extrusion_id": 2,
        "next_id": 9,
        "profile": {"kind": "Bracket", "width_mm": 80.0, "height_mm": 50.0,
            "leg_mm": 10.0, "entity_ids": [3, 4, 5, 6, 7, 8]},
        "depth_mm": 6.0
    }))
    .expect("fixture serializes")
}

#[test]
fn bracket_is_read_without_changing_source_bytes() {
    let source = bracket();
    let before = source.clone();
    let part = read_legacy_v1(&source).expect("valid bracket");
    assert_eq!(part.profile_name(), "bracket");
    assert_eq!(part.depth_mm, 6.0);
    assert_eq!(source, before);
}

#[test]
fn disk_has_stable_legacy_ids() {
    let source = json!({"schema_version":1,"units":"mm","sketch_id":1,"extrusion_id":2,
        "next_id":4,"profile":{"kind":"Disk","radius_mm":25.0,"entity_id":3},"depth_mm":8.0});
    let bytes = serde_json::to_vec(&source).expect("fixture serializes");
    let part = read_legacy_v1(&bytes).expect("valid disk");
    assert!(matches!(&part.profile, LegacyProfile::Disk { radius_mm: 25.0, entity_id: 3 }));
    let sketch = import_legacy_sketch(&part).expect("disk sketch maps to CADCraft");
    assert_eq!(sketch.drawing.model.len(), 1);
    assert_eq!(sketch.legacy_to_handle.len(), 1);
}

#[test]
fn bracket_maps_six_original_ids_to_six_editable_lines() {
    let part = read_legacy_v1(&bracket()).expect("valid bracket");
    let sketch = import_legacy_sketch(&part).expect("bracket sketch maps to CADCraft");
    assert_eq!(sketch.drawing.model.len(), 6);
    assert_eq!(sketch.legacy_to_handle.len(), 6);
    assert_eq!(sketch.legacy_to_handle.keys().copied().collect::<Vec<_>>(), [3, 4, 5, 6, 7, 8]);
    for handle in sketch.legacy_to_handle.values() {
        assert!(sketch.drawing.model.contains(*handle));
    }
}

#[test]
fn converted_bracket_handles_survive_native_dxf_round_trip() {
    let part = read_legacy_v1(&bracket()).expect("valid bracket");
    let sketch = import_legacy_sketch(&part).expect("CADCraft sketch");
    let bytes = cadcraft_io::write_dxf(&sketch.drawing).into_bytes();
    let reopened = cadcraft_io::read_dxf(&bytes).expect("native DXF reopens");
    assert_eq!(reopened.model.len(), 6);
    for handle in sketch.legacy_to_handle.values() {
        assert!(reopened.model.contains(*handle), "source handle lost in native DXF");
    }
}

#[test]
fn duplicate_ids_and_unknown_fields_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_slice(&bracket()).expect("fixture parses");
    value["profile"]["entity_ids"] = json!([3, 4, 5, 6, 7, 7]);
    assert!(read_legacy_v1(&serde_json::to_vec(&value).expect("fixture serializes")).is_err());
    value["profile"]["entity_ids"] = json!([3, 4, 5, 6, 7, 8]);
    value["invented"] = json!(true);
    assert!(read_legacy_v1(&serde_json::to_vec(&value).expect("fixture serializes")).is_err());
}

#[test]
fn malformed_and_oversize_are_rejected() {
    assert!(read_legacy_v1(b"{").is_err());
    assert!(read_legacy_v1(&vec![b' '; MAX_LEGACY_BYTES + 1]).is_err());
}
