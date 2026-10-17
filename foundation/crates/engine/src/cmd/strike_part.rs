//! Bounded, read-only entry point for existing CADStrike v1 part files.
//! The sketch/solid conversion command is deliberately separate: this command never replaces a
//! drawing or implies that a precision-invalid STL is safe.

use cadstrike_project::{MAX_LEGACY_BYTES, import_legacy_sketch, read_legacy_v1};
use serde_json::{Value, json};

use super::{CommandSpec, always};
use crate::{EngineError, Result, Session};

pub const MAX_SKETCH_SOURCE_BYTES: u64 = MAX_LEGACY_BYTES as u64;

pub(super) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("strikelegacyinfo", "Inspect CADStrike Legacy Part", inspect)
            .enabled(always)
            .noundo()
            .params("{path: string} -> {profile, units, depthMm, sketchId, extrusionId, migrationStatus}; reads a CADStrike v1 file without opening or changing a drawing"),
        CommandSpec::new("strikeimportsketch", "Import CADStrike 2D Sketch (No Solid)...", import_sketch)
            .menu(&["File", "Import CADStrike 2D Sketch (No Solid)..."])
            .enabled(always)
            .noundo()
            .params("{data: base64 legacy v1 JSON} -> {index, sketchEntities, legacyToHandle, sourceDepthMm, solidImported: false}; creates a new editable 2D drawing, never overwrites the legacy source"),
    ]
}

fn import_sketch(session: &mut Session, params: &Value) -> Result<Value> {
    let data = params
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| EngineError::Other("strikeimportsketch requires base64 `data` from a user-selected file".into()))?;
    let max_encoded = MAX_LEGACY_BYTES.div_ceil(3) * 4 + 4;
    if data.len() > max_encoded {
        return Err(EngineError::Other("CADStrike legacy file exceeds 8 MiB".into()));
    }
    let bytes = crate::cmd::file::base64_decode(data).ok_or_else(|| EngineError::Other("CADStrike legacy file has invalid base64".into()))?;
    if bytes.len() > MAX_LEGACY_BYTES {
        return Err(EngineError::Other("CADStrike legacy file exceeds 8 MiB".into()));
    }
    let part = read_legacy_v1(&bytes).map_err(EngineError::Other)?;
    let sketch = import_legacy_sketch(&part).map_err(EngineError::Other)?;
    let handles = sketch.legacy_to_handle.iter().map(|(old, handle)| (old.to_string(), handle.0)).collect::<std::collections::BTreeMap<_, _>>();
    let count = sketch.drawing.model.len();
    let profile = part.profile_name();
    let title = format!("CADStrike {profile} 2D sketch (solid not imported)");
    let index = session.open_drawing(sketch.drawing, &title, None);
    Ok(json!({
        "index": index,
        "profile": profile,
        "sketchEntities": count,
        "legacyToHandle": handles,
        "sourceDepthMm": part.depth_mm,
        "solidImported": false,
        "sourcePreserved": true,
        "migrationStatus": "editable_2d_sketch_only",
    }))
}

fn inspect(_session: &mut Session, params: &Value) -> Result<Value> {
    let path = params.get("path").and_then(Value::as_str).ok_or_else(|| EngineError::Other("strikelegacyinfo requires a string `path`".into()))?;
    let bytes = read_bounded(path)?;
    let part = read_legacy_v1(&bytes).map_err(EngineError::Other)?;
    let sketch = import_legacy_sketch(&part).map_err(EngineError::Other)?;
    Ok(json!({
        "profile": part.profile_name(),
        "units": part.units,
        "depthMm": part.depth_mm,
        "sketchId": part.sketch_id,
        "extrusionId": part.extrusion_id,
        "sketchEntities": sketch.drawing.model.len(),
        "legacyToHandle": sketch.legacy_to_handle.iter().map(|(old, handle)| (old.to_string(), handle.0)).collect::<std::collections::BTreeMap<_, _>>(),
        "migrationStatus": "schema_valid_geometry_unverified"
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_bounded(path: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
    if file.metadata().map_err(|e| EngineError::Other(format!("{path}: {e}")))?.len() > MAX_LEGACY_BYTES as u64 {
        return Err(EngineError::Other("CADStrike legacy file exceeds 8 MiB".into()));
    }
    let mut bytes = Vec::new();
    file.take(MAX_LEGACY_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
    if bytes.len() > MAX_LEGACY_BYTES {
        return Err(EngineError::Other("CADStrike legacy file exceeds 8 MiB".into()));
    }
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
fn read_bounded(_path: &str) -> Result<Vec<u8>> {
    Err(EngineError::Other("local CADStrike files are unavailable on the web".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_bracket_creates_a_separate_editable_sketch_and_reports_the_missing_solid() {
        let source = json!({"schema_version":1,"units":"mm","sketch_id":1,"extrusion_id":2,
            "next_id":9,"profile":{"kind":"Bracket","width_mm":80.0,"height_mm":50.0,
            "leg_mm":10.0,"entity_ids":[3,4,5,6,7,8]},"depth_mm":6.0});
        let bytes = serde_json::to_vec(&source).expect("fixture JSON");
        let mut session = Session::new();
        let original = session.doc().expect("starter drawing").clone();
        let count = session.docs.len();
        let result = import_sketch(&mut session, &json!({"data":crate::cmd::file::base64_encode(&bytes)})).expect("2D sketch import");
        assert_eq!(session.docs.len(), count + 1);
        assert_eq!(*session.docs[0].doc, original);
        assert_eq!(session.doc().expect("new sketch").model.len(), 6);
        assert!(session.state().expect("new state").path.is_none());
        assert_eq!(result["solidImported"], false);
        assert_eq!(result["sourceDepthMm"], 6.0);
    }

    #[test]
    fn invalid_sketch_does_not_add_a_drawing() {
        let mut session = Session::new();
        let count = session.docs.len();
        assert!(import_sketch(&mut session, &json!({"data":"@@"})).is_err());
        assert_eq!(session.docs.len(), count);
    }

    #[test]
    fn missing_path_keeps_the_active_drawing() {
        let mut session = Session::new();
        let before = session.doc().expect("starter drawing").clone();
        assert!(inspect(&mut session, &json!({"path": false})).is_err());
        assert_eq!(*session.doc().expect("drawing remains"), before);
    }
}
