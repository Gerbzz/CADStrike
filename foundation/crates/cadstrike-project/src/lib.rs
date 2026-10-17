//! Read-only compatibility boundary for CADStrike v1 parts.
//! A parsed part is not yet a CADCraft drawing or a validated STL.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use cadcraft_doc::{Circle, Drawing, Entity, EntityKind, Handle, Line, geom::Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const MAX_LEGACY_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum LegacyProfile {
    Bracket { width_mm: f64, height_mm: f64, leg_mm: f64, entity_ids: [u64; 6] },
    Disk { radius_mm: f64, entity_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPart {
    pub schema_version: u32,
    pub units: String,
    pub sketch_id: u64,
    pub extrusion_id: u64,
    pub next_id: u64,
    pub profile: LegacyProfile,
    pub depth_mm: f64,
}

/// A donor-native, editable 2D sketch preview. Feature history and STL export are not admitted by
/// this type until the native project round-trip and precision fixtures pass.
#[derive(Clone, Debug)]
pub struct ImportedSketch {
    pub drawing: Drawing,
    pub legacy_to_handle: BTreeMap<u64, Handle>,
    pub source: LegacyPart,
}

impl LegacyPart {
    /// Validate the original schema and identifier relationships without generating geometry.
    /// Precision-invalid v1 parts can therefore enter a separate read-only repair path later.
    pub fn validate_schema(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.units != "mm" {
            return Err("unsupported CADStrike schema or unit".into());
        }
        if !valid_dimension(self.depth_mm) {
            return Err("extrusion depth must be finite and 0.001-100000 mm".into());
        }
        let mut ids = HashSet::new();
        if !ids.insert(self.sketch_id) || !ids.insert(self.extrusion_id) {
            return Err("duplicate feature ID".into());
        }
        match &self.profile {
            LegacyProfile::Bracket { width_mm, height_mm, leg_mm, entity_ids } => {
                if !valid_dimension(*width_mm)
                    || !valid_dimension(*height_mm)
                    || !valid_dimension(*leg_mm)
                    || width_mm <= leg_mm
                    || height_mm <= leg_mm
                {
                    return Err("bracket width and height must exceed a positive finite leg width".into());
                }
                for id in entity_ids {
                    if !ids.insert(*id) {
                        return Err("duplicate sketch entity ID".into());
                    }
                }
            }
            LegacyProfile::Disk { radius_mm, entity_id } => {
                if !valid_dimension(*radius_mm) {
                    return Err("disk radius must be finite and 0.001-100000 mm".into());
                }
                if !ids.insert(*entity_id) {
                    return Err("duplicate sketch entity ID".into());
                }
            }
        }
        if ids.iter().any(|id| *id >= self.next_id) {
            return Err("next CAD ID must exceed used IDs".into());
        }
        Ok(())
    }

    pub fn profile_name(&self) -> &'static str {
        match &self.profile {
            LegacyProfile::Bracket { .. } => "bracket",
            LegacyProfile::Disk { .. } => "disk",
        }
    }
}

fn valid_dimension(value: f64) -> bool {
    value.is_finite() && (0.001..=100_000.0).contains(&value)
}

/// Parse an existing v1 file from bounded bytes without writing or opening a donor session.
pub fn read_legacy_v1(bytes: &[u8]) -> Result<LegacyPart, String> {
    if bytes.len() > MAX_LEGACY_BYTES {
        return Err("CAD document is too large".into());
    }
    let part: LegacyPart = serde_json::from_slice(bytes).map_err(|e| format!("invalid CADStrike v1 JSON: {e}"))?;
    part.validate_schema()?;
    Ok(part)
}

/// Recreate the legacy profile as CADCraft line/circle entities with a stable migration map.
/// This is a 2D conversion only; callers must not present it as a validated parametric solid.
pub fn import_legacy_sketch(part: &LegacyPart) -> Result<ImportedSketch, String> {
    part.validate_schema()?;
    let mut drawing = Drawing::new_metric();
    let mut legacy_to_handle = BTreeMap::new();
    match &part.profile {
        LegacyProfile::Bracket { width_mm: w, height_mm: h, leg_mm: l, entity_ids } => {
            let points = [(0.0, 0.0), (*w, 0.0), (*w, *l), (*l, *l), (*l, *h), (0.0, *h)];
            for (index, old_id) in entity_ids.iter().enumerate() {
                let a = points.get(index).ok_or("bracket endpoint is missing")?;
                let b = points.get((index + 1) % points.len()).ok_or("bracket endpoint is missing")?;
                let handle = drawing.new_handle();
                drawing.model.push(Entity::new(handle, EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) })));
                legacy_to_handle.insert(*old_id, handle);
            }
        }
        LegacyProfile::Disk { radius_mm, entity_id } => {
            let handle = drawing.new_handle();
            drawing.model.push(Entity::new(handle, EntityKind::Circle(Circle { center: Vec3::new(0.0, 0.0, 0.0), radius: *radius_mm })));
            legacy_to_handle.insert(*entity_id, handle);
        }
    }
    Ok(ImportedSketch { drawing, legacy_to_handle, source: part.clone() })
}
