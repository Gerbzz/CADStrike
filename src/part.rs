//! Narrow parametric parts: dimensions are authoritative; triangles are derived.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

const MAX_DOCUMENT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Profile {
    Bracket {
        width_mm: f64,
        height_mm: f64,
        leg_mm: f64,
        entity_ids: [u64; 6],
    },
    Disk {
        radius_mm: f64,
        entity_id: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub schema_version: u32,
    pub units: String,
    pub sketch_id: u64,
    pub extrusion_id: u64,
    pub next_id: u64,
    pub profile: Profile,
    pub depth_mm: f64,
}

#[derive(Debug, Clone)]
pub struct Solid {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportReport {
    pub triangles: usize,
    pub bounds_min_mm: [f64; 3],
    pub bounds_max_mm: [f64; 3],
    pub unit_convention: String,
    pub warning: String,
}

fn valid_dimension(value: f64) -> bool {
    value.is_finite() && (0.001..=100_000.0).contains(&value)
}

impl Document {
    pub fn bracket(
        width_mm: f64,
        height_mm: f64,
        leg_mm: f64,
        depth_mm: f64,
    ) -> Result<Self, String> {
        let document = Self {
            schema_version: 1,
            units: "mm".into(),
            sketch_id: 1,
            extrusion_id: 2,
            next_id: 9,
            profile: Profile::Bracket {
                width_mm,
                height_mm,
                leg_mm,
                entity_ids: [3, 4, 5, 6, 7, 8],
            },
            depth_mm,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn disk(radius_mm: f64, depth_mm: f64) -> Result<Self, String> {
        let document = Self {
            schema_version: 1,
            units: "mm".into(),
            sketch_id: 1,
            extrusion_id: 2,
            next_id: 4,
            profile: Profile::Disk {
                radius_mm,
                entity_id: 3,
            },
            depth_mm,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.units != "mm" {
            return Err("unsupported CADStrike schema or unit".into());
        }
        if !valid_dimension(self.depth_mm) {
            return Err("extrusion depth must be finite and 0.001-100000 mm".into());
        }
        let mut ids = std::collections::HashSet::new();
        if !ids.insert(self.sketch_id) || !ids.insert(self.extrusion_id) {
            return Err("duplicate feature ID".into());
        }
        match self.profile {
            Profile::Bracket {
                width_mm,
                height_mm,
                leg_mm,
                entity_ids,
            } => {
                if !valid_dimension(width_mm)
                    || !valid_dimension(height_mm)
                    || !valid_dimension(leg_mm)
                    || width_mm <= leg_mm
                    || height_mm <= leg_mm
                {
                    return Err(
                        "bracket width and height must exceed a positive finite leg width".into(),
                    );
                }
                for id in entity_ids {
                    if !ids.insert(id) {
                        return Err("duplicate sketch entity ID".into());
                    }
                }
            }
            Profile::Disk {
                radius_mm,
                entity_id,
            } => {
                if !valid_dimension(radius_mm) {
                    return Err("disk radius must be finite and 0.001-100000 mm".into());
                }
                if !ids.insert(entity_id) {
                    return Err("duplicate sketch entity ID".into());
                }
            }
        }
        if self.next_id <= *ids.iter().max().unwrap() {
            return Err("next CAD ID must exceed used IDs".into());
        }
        Ok(())
    }

    pub fn set_width(&mut self, width_mm: f64) -> Result<(), String> {
        let mut candidate = self.clone();
        match &mut candidate.profile {
            Profile::Bracket {
                width_mm: width, ..
            } => *width = width_mm,
            Profile::Disk { .. } => return Err("width applies only to bracket profiles".into()),
        }
        candidate.validate()?;
        candidate.build()?;
        *self = candidate;
        Ok(())
    }

    pub fn set_dimensions(&mut self, profile: Profile, depth_mm: f64) -> Result<(), String> {
        let mut candidate = self.clone();
        candidate.profile = profile;
        candidate.depth_mm = depth_mm;
        candidate.validate()?;
        candidate.build()?;
        *self = candidate;
        Ok(())
    }

    pub fn build(&self) -> Result<Solid, String> {
        self.validate()?;
        let outline = match self.profile {
            Profile::Bracket {
                width_mm: w,
                height_mm: h,
                leg_mm: l,
                ..
            } => vec![(0.0, 0.0), (w, 0.0), (w, l), (l, l), (l, h), (0.0, h)],
            Profile::Disk { radius_mm, .. } => disk_outline(radius_mm, 0.05)?,
        };
        let solid = extrude(&outline, self.depth_mm)?;
        if !solid.manifold() {
            return Err("derived solid is not edge-manifold".into());
        }
        Ok(solid)
    }

    pub fn save_new(&self, path: &Path) -> Result<(), String> {
        self.build()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err("CAD document is too large".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        pending
            .write_all(&bytes)
            .and_then(|_| pending.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        Self::load(pending.path())?;
        pending
            .persist_noclobber(path)
            .map_err(|e| e.error.to_string())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if path.metadata().map_err(|e| e.to_string())?.len() > MAX_DOCUMENT_BYTES {
            return Err("CAD document is too large".into());
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let document: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        document.build()?;
        Ok(document)
    }

    pub fn file_sha256(path: &Path) -> Result<String, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err("CAD document is too large".into());
        }
        Ok(format!("{:x}", Sha256::digest(&bytes)))
    }

    pub fn previous_path(path: &Path) -> std::path::PathBuf {
        path.with_extension("previous.cadstrike")
    }

    pub fn save_replace(&self, path: &Path, expected_hash: &str) -> Result<(), String> {
        Self::load(path)?;
        if Self::file_sha256(path)? != expected_hash {
            return Err("CAD document changed on disk".into());
        }
        self.build()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        pending
            .write_all(&bytes)
            .and_then(|_| pending.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        Self::load(pending.path())?;
        let mut backup = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        std::io::copy(
            &mut std::fs::File::open(path).map_err(|e| e.to_string())?,
            backup.as_file_mut(),
        )
        .map_err(|e| e.to_string())?;
        backup.as_file().sync_all().map_err(|e| e.to_string())?;
        backup
            .persist(Self::previous_path(path))
            .map_err(|e| e.error.to_string())?;
        pending.persist(path).map_err(|e| e.error.to_string())?;
        Ok(())
    }

    pub fn export_stl_new(&self, path: &Path) -> Result<ExportReport, String> {
        let solid = self.build()?;
        let (min, max) = solid.bounds();
        let report = ExportReport { triangles: solid.triangles.len(), bounds_min_mm: min, bounds_max_mm: max,
            unit_convention: "millimetres (STL itself has no units)".into(),
            warning: "STL contains a triangle reference mesh only; no dimensions, constraints or feature history".into() };
        let triangles =
            u32::try_from(solid.triangles.len()).map_err(|_| "STL triangle count exceeds u32")?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        let mut header = [0u8; 80];
        header[..28].copy_from_slice(b"CADStrike STL; units: mm    ");
        pending
            .write_all(&header)
            .and_then(|_| pending.write_all(&triangles.to_le_bytes()))
            .map_err(|e| e.to_string())?;
        for triangle in &solid.triangles {
            let [a, b, c] = triangle.map(|i| solid.vertices[i]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let length = n.iter().map(|x| x * x).sum::<f64>().sqrt();
            for value in n.map(|x| x / length).into_iter().chain(a).chain(b).chain(c) {
                let value = value as f32;
                if !value.is_finite() {
                    return Err("STL coordinate exceeds f32 range".into());
                }
                pending
                    .write_all(&value.to_le_bytes())
                    .map_err(|e| e.to_string())?;
            }
            pending
                .write_all(&0u16.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        pending.as_file().sync_all().map_err(|e| e.to_string())?;
        pending
            .persist_noclobber(path)
            .map_err(|e| e.error.to_string())?;
        Ok(report)
    }
}

impl Solid {
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| self.vertices[i]);
                let cross = [
                    b[1] * c[2] - b[2] * c[1],
                    b[2] * c[0] - b[0] * c[2],
                    b[0] * c[1] - b[1] * c[0],
                ];
                (a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2]) / 6.0
            })
            .sum::<f64>()
            .abs()
    }

    pub fn bounds(&self) -> ([f64; 3], [f64; 3]) {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for p in &self.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        (min, max)
    }

    pub fn manifold(&self) -> bool {
        let mut edges = HashMap::new();
        for t in &self.triangles {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                *edges.entry((a.min(b), a.max(b))).or_insert(0u8) += 1;
            }
        }
        edges.values().all(|&count| count == 2)
    }
}

fn extrude(outline: &[(f64, f64)], depth: f64) -> Result<Solid, String> {
    if outline.len() < 3 || outline.len() > 4096 {
        return Err("profile facet count outside supported range".into());
    }
    let n = outline.len();
    let mut vertices = Vec::with_capacity(2 * n);
    for &(x, y) in outline {
        vertices.push([x, y, 0.0]);
    }
    for &(x, y) in outline {
        vertices.push([x, y, depth]);
    }
    let mut triangles = Vec::with_capacity(4 * n - 4);
    for i in 1..n - 1 {
        triangles.push([0, i + 1, i]);
        triangles.push([n, n + i, n + i + 1]);
    }
    for i in 0..n {
        let next = (i + 1) % n;
        triangles.push([i, next, n + next]);
        triangles.push([i, n + next, n + i]);
    }
    Ok(Solid {
        vertices,
        triangles,
    })
}

fn disk_outline(radius: f64, chord_error: f64) -> Result<Vec<(f64, f64)>, String> {
    let minimum =
        (std::f64::consts::PI / (1.0 - (chord_error / radius).min(1.0)).acos()).ceil() as usize;
    let facets = minimum.max(8).next_multiple_of(4);
    if facets > 4096 {
        return Err("disk needs more than 4096 facets at 0.05 mm chord error".into());
    }
    Ok((0..facets)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / facets as f64;
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect())
}
