use crate::part::{Document, Profile};
use eframe::egui;
use std::path::{Path, PathBuf};

enum PendingAction {
    Bracket([f64; 4]),
    Disk([f64; 2]),
    Open(PathBuf),
}

pub struct CadStrikeApp {
    document: Option<Document>,
    saved: Option<Document>,
    path: Option<PathBuf>,
    file_hash: Option<String>,
    undo: Vec<Document>,
    redo: Vec<Document>,
    form: [f64; 4],
    disk_mode: bool,
    status: String,
    error: Option<String>,
    pending: Option<PendingAction>,
}

impl Default for CadStrikeApp {
    fn default() -> Self {
        Self {
            document: None,
            saved: None,
            path: None,
            file_hash: None,
            undo: Vec::new(),
            redo: Vec::new(),
            form: [80.0, 50.0, 10.0, 6.0],
            disk_mode: false,
            status: "Create a dimensioned bracket or disk to begin.".into(),
            error: None,
            pending: None,
        }
    }
}

impl CadStrikeApp {
    fn dirty(&self) -> bool {
        self.document
            .as_ref()
            .is_some_and(|d| Some(d) != self.saved.as_ref())
    }

    fn install(&mut self, document: Document, path: Option<PathBuf>) -> Result<(), String> {
        let hash = path.as_deref().map(Document::file_sha256).transpose()?;
        self.saved = path.as_ref().map(|_| document.clone());
        self.document = Some(document);
        self.path = path;
        self.file_hash = hash;
        self.undo.clear();
        self.redo.clear();
        self.sync_form();
        self.error = None;
        Ok(())
    }

    fn sync_form(&mut self) {
        if let Some(document) = &self.document {
            match document.profile {
                Profile::Bracket {
                    width_mm,
                    height_mm,
                    leg_mm,
                    ..
                } => {
                    self.disk_mode = false;
                    self.form = [width_mm, height_mm, leg_mm, document.depth_mm];
                }
                Profile::Disk { radius_mm, .. } => {
                    self.disk_mode = true;
                    self.form = [radius_mm, 0.0, 0.0, document.depth_mm];
                }
            }
        }
    }

    fn new_bracket(&mut self, width: f64, height: f64, leg: f64, depth: f64) -> Result<(), String> {
        let document = Document::bracket(width, height, leg, depth)?;
        self.install(document, None)?;
        self.status = "New parametric six-edge bracket in millimetres".into();
        Ok(())
    }

    fn new_disk(&mut self, radius: f64, depth: f64) -> Result<(), String> {
        let document = Document::disk(radius, depth)?;
        self.install(document, None)?;
        self.status = "New parametric disk in millimetres".into();
        Ok(())
    }

    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        let document = Document::load(path)?;
        self.install(document, Some(path.to_path_buf()))?;
        self.status = format!("Opened {}", path.display());
        Ok(())
    }

    fn commit(&mut self, next: Document) -> Result<(), String> {
        next.build()?;
        let previous = self
            .document
            .as_ref()
            .ok_or("create or open a part first")?
            .clone();
        if next != previous {
            self.document = Some(next);
            self.undo.push(previous);
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        self.sync_form();
        self.status = "Dimensions applied; solid rebuilt".into();
        Ok(())
    }

    fn set_bracket(&mut self, width: f64, height: f64, leg: f64, depth: f64) -> Result<(), String> {
        let mut next = self
            .document
            .as_ref()
            .ok_or("create or open a part first")?
            .clone();
        let ids = match next.profile {
            Profile::Bracket { entity_ids, .. } => entity_ids,
            Profile::Disk { .. } => return Err("open a bracket to edit bracket dimensions".into()),
        };
        next.set_dimensions(
            Profile::Bracket {
                width_mm: width,
                height_mm: height,
                leg_mm: leg,
                entity_ids: ids,
            },
            depth,
        )?;
        self.commit(next)
    }

    fn set_disk(&mut self, radius: f64, depth: f64) -> Result<(), String> {
        let mut next = self
            .document
            .as_ref()
            .ok_or("create or open a part first")?
            .clone();
        let id = match next.profile {
            Profile::Disk { entity_id, .. } => entity_id,
            Profile::Bracket { .. } => return Err("open a disk to edit disk dimensions".into()),
        };
        next.set_dimensions(
            Profile::Disk {
                radius_mm: radius,
                entity_id: id,
            },
            depth,
        )?;
        self.commit(next)
    }

    fn undo(&mut self) -> Result<(), String> {
        let previous = self.undo.pop().ok_or("nothing to undo")?;
        self.redo
            .push(self.document.replace(previous).ok_or("no part open")?);
        self.sync_form();
        self.status = "Undid dimension edit".into();
        Ok(())
    }

    fn redo(&mut self) -> Result<(), String> {
        let next = self.redo.pop().ok_or("nothing to redo")?;
        self.undo
            .push(self.document.replace(next).ok_or("no part open")?);
        self.sync_form();
        self.status = "Redid dimension edit".into();
        Ok(())
    }

    fn save(&mut self, path: &Path) -> Result<(), String> {
        let document = self
            .document
            .as_ref()
            .ok_or("create or open a part first")?;
        if self.path.as_deref() == Some(path) {
            document.save_replace(
                path,
                self.file_hash.as_deref().ok_or("saved file hash missing")?,
            )?;
        } else {
            document.save_new(path)?;
        }
        self.file_hash = Some(Document::file_sha256(path)?);
        self.saved = Some(document.clone());
        self.path = Some(path.to_path_buf());
        self.status = format!("Saved editable part: {}", path.display());
        self.error = None;
        Ok(())
    }

    fn export_stl(&mut self, path: &Path) -> Result<(), String> {
        let report = self
            .document
            .as_ref()
            .ok_or("create or open a part first")?
            .export_stl_new(path)?;
        self.status = format!(
            "Exported {} triangles to {}. STL has no units or constraints; coordinates are millimetres.",
            report.triangles,
            path.display()
        );
        self.error = None;
        Ok(())
    }

    fn request(&mut self, action: PendingAction) -> Result<(), String> {
        if self.dirty() {
            self.pending = Some(action);
            self.status =
                "Unsaved dimensions: save, discard or cancel before opening another part".into();
            Ok(())
        } else {
            self.perform(action)
        }
    }

    fn perform(&mut self, action: PendingAction) -> Result<(), String> {
        match action {
            PendingAction::Bracket([w, h, l, d]) => self.new_bracket(w, h, l, d),
            PendingAction::Disk([r, d]) => self.new_disk(r, d),
            PendingAction::Open(path) => self.open(&path),
        }
    }

    fn choose_save_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("CADStrike part", &["cadstrike"])
            .set_file_name("part.cadstrike")
            .save_file()
    }

    fn save_current(&mut self) -> Result<(), String> {
        let path = match &self.path {
            Some(path) => path.clone(),
            None => Self::choose_save_path().ok_or("save canceled")?,
        };
        self.save(&path)
    }
}

impl eframe::App for CadStrikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 7.0);
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
            style
                .text_styles
                .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(30, 37, 43);
            visuals.window_fill = egui::Color32::from_rgb(36, 43, 49);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(46, 57, 64);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(57, 74, 80);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(49, 112, 119);
            visuals.selection.bg_fill = egui::Color32::from_rgb(38, 117, 126);
            style.visuals = visuals;
        });
        egui::TopBottomPanel::top("cad_toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("CADSTRIKE");
                ui.separator();
                if ui.button("New bracket").clicked() {
                    let result = self.request(PendingAction::Bracket([80.0, 50.0, 10.0, 6.0]));
                    self.error = result.err();
                }
                if ui.button("New disk").clicked() {
                    let result = self.request(PendingAction::Disk([25.0, 8.0]));
                    self.error = result.err();
                }
                if ui.button("Open part").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("CADStrike part", &["cadstrike"])
                        .pick_file()
                    {
                        let result = self.request(PendingAction::Open(path));
                        self.error = result.err();
                    }
                }
                let has_doc = self.document.is_some();
                if ui
                    .add_enabled(has_doc, egui::Button::new("Save  Ctrl+S"))
                    .clicked()
                {
                    let result = self.save_current();
                    self.error = result.err();
                }
                if ui
                    .add_enabled(has_doc, egui::Button::new("Save as…"))
                    .clicked()
                {
                    if let Some(path) = Self::choose_save_path() {
                        let result = self.save(&path);
                        self.error = result.err();
                    }
                }
                if ui
                    .add_enabled(has_doc, egui::Button::new("Export STL"))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Triangle STL", &["stl"])
                        .set_file_name("part.stl")
                        .save_file()
                    {
                        let result = self.export_stl(&path);
                        self.error = result.err();
                    }
                }
            });
        });
        egui::SidePanel::left("cad_features")
            .default_width(215.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Features");
                if let Some(doc) = &self.document {
                    ui.label(format!("Sketch {} · XY plane", doc.sketch_id));
                    ui.label(format!(
                        "Extrusion {} · {:.3} mm",
                        doc.extrusion_id, doc.depth_mm
                    ));
                    ui.separator();
                    match doc.profile {
                        Profile::Bracket { entity_ids, .. } => {
                            ui.strong("Six-edge bracket profile");
                            for id in entity_ids {
                                ui.small(format!("Line {id}"));
                            }
                        }
                        Profile::Disk { entity_id, .. } => {
                            ui.strong(format!("Circle {entity_id}"));
                        }
                    }
                    ui.separator();
                    ui.label(if self.dirty() {
                        "Unsaved dimensions"
                    } else {
                        "Saved"
                    });
                    if let Some(path) = &self.path {
                        ui.small(path.file_name().unwrap_or_default().to_string_lossy())
                            .on_hover_text(path.display().to_string());
                    }
                } else {
                    ui.label("No part open");
                }
            });
        egui::SidePanel::right("cad_dimensions").default_width(260.0).resizable(true).show(ctx, |ui| {
            ui.heading("Driving dimensions");
            if self.document.is_some() {
                let labels=if self.disk_mode { ["Radius mm", "", "", "Depth mm"] } else { ["Width mm", "Height mm", "Leg mm", "Depth mm"] };
                for i in 0..4 {
                    if labels[i].is_empty() { continue; }
                    ui.horizontal(|ui| { ui.label(labels[i]); ui.add(egui::DragValue::new(&mut self.form[i]).speed(0.1)); });
                }
                if ui.button("Apply dimensions").clicked() {
                    let result=if self.disk_mode { self.set_disk(self.form[0],self.form[3]) }
                        else { self.set_bracket(self.form[0],self.form[1],self.form[2],self.form[3]) };
                    self.error=result.err();
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.undo.is_empty(),egui::Button::new("Undo  Ctrl+Z")).clicked() { let result=self.undo(); self.error=result.err(); }
                    if ui.add_enabled(!self.redo.is_empty(),egui::Button::new("Redo  Ctrl+Y")).clicked() { let result=self.redo(); self.error=result.err(); }
                });
                if let Some(Ok(solid))=self.document.as_ref().map(Document::build) {
                    let (min,max)=solid.bounds();
                    ui.separator();
                    ui.strong("Rebuilt solid");
                    ui.label(format!("Volume  {:.3} mm³",solid.volume()));
                    ui.label(format!("Envelope  {:.3} × {:.3} × {:.3} mm",max[0]-min[0],max[1]-min[1],max[2]-min[2]));
                    ui.label(format!("{} triangles · manifold",solid.triangles.len()));
                }
            } else { ui.label("Create a bracket or disk to edit its dimensions."); }
            ui.separator();
            ui.small("STL is a unitless triangle reference. Constraints and feature history stay in .cadstrike.");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sketch & solid preview");
            if let Some(doc)=&self.document { draw_part(ui,doc); }
            else { ui.label("Start with New bracket or New disk. The sketch is dimension driven and rebuilds its extrusion."); }
        });
        egui::TopBottomPanel::bottom("cad_status").show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 150, 140),
                    format!("Error: {error}"),
                );
            }
            ui.label(&self.status);
        });
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            let result = self.undo();
            self.error = result.err();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
            let result = self.redo();
            self.error = result.err();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            let result = self.save_current();
            self.error = result.err();
        }
        if self.pending.is_some() {
            let mut save = false;
            let mut discard = false;
            let mut cancel = false;
            egui::Window::new("Unsaved CAD dimensions")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save this editable part before opening another?");
                    ui.horizontal(|ui| {
                        save = ui.button("Save and continue").clicked();
                        discard = ui.button("Discard and continue").clicked();
                        cancel = ui.button("Cancel").clicked();
                    });
                });
            if cancel {
                self.pending = None;
            }
            if save {
                match self.save_current() {
                    Ok(()) => {
                        if let Some(action) = self.pending.take() {
                            let result = self.perform(action);
                            self.error = result.err();
                        }
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            if discard {
                if let Some(action) = self.pending.take() {
                    let result = self.perform(action);
                    self.error = result.err();
                }
            }
        }
    }
}

fn draw_part(ui: &mut egui::Ui, doc: &Document) {
    let available = ui.available_size();
    let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(24, 30, 36));
    let Ok(solid) = doc.build() else { return };
    let (min, max) = solid.bounds();
    let width = (max[0] - min[0]).max(0.001);
    let height = (max[1] - min[1]).max(0.001);
    let scale = ((rect.width() - 50.0) as f64 / width)
        .min((rect.height() - 50.0) as f64 / height)
        .max(0.001);
    let to_screen = |p: [f64; 3]| {
        egui::pos2(
            rect.center().x + ((p[0] - (min[0] + max[0]) / 2.0) * scale) as f32,
            rect.center().y - ((p[1] - (min[1] + max[1]) / 2.0) * scale) as f32,
        )
    };
    for i in 1..10 {
        let x = rect.left() + rect.width() * i as f32 / 10.0;
        let y = rect.top() + rect.height() * i as f32 / 10.0;
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(37, 45, 53)),
        );
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(37, 45, 53)),
        );
    }
    let n = solid.vertices.len() / 2;
    for i in 0..n {
        let next = (i + 1) % n;
        painter.line_segment(
            [
                to_screen(solid.vertices[i]),
                to_screen(solid.vertices[next]),
            ],
            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(122, 207, 212)),
        );
    }
    painter.text(
        rect.left_top() + egui::vec2(14.0, 14.0),
        egui::Align2::LEFT_TOP,
        "XY SKETCH  ·  mm",
        egui::FontId::monospace(12.0),
        egui::Color32::from_rgb(175, 187, 198),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_dimension_edit_undo_save_reopen_and_stl() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("bracket.cadstrike");
        let stl = dir.path().join("bracket.stl");
        let mut app = CadStrikeApp::default();
        app.new_bracket(80.0, 50.0, 10.0, 6.0).unwrap();
        assert_eq!(
            app.document.as_ref().unwrap().build().unwrap().volume(),
            7200.0
        );
        app.set_bracket(90.0, 50.0, 10.0, 6.0).unwrap();
        assert_eq!(
            app.document.as_ref().unwrap().build().unwrap().volume(),
            7800.0
        );
        app.undo().unwrap();
        assert_eq!(
            app.document.as_ref().unwrap().build().unwrap().volume(),
            7200.0
        );
        app.redo().unwrap();
        app.save(&document).unwrap();
        app.open(&document).unwrap();
        assert_eq!(
            app.document.as_ref().unwrap().build().unwrap().volume(),
            7800.0
        );
        app.export_stl(&stl).unwrap();
        assert!(stl.is_file());
    }

    #[test]
    fn invalid_edit_keeps_part_and_replace_keeps_previous_revision() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bracket.cadstrike");
        let mut app = CadStrikeApp::default();
        app.new_bracket(80.0, 50.0, 10.0, 6.0).unwrap();
        app.save(&path).unwrap();
        assert!(app.set_bracket(8.0, 50.0, 10.0, 6.0).is_err());
        assert_eq!(
            app.document.as_ref().unwrap().build().unwrap().volume(),
            7200.0
        );
        assert!(!app.dirty());
        app.set_bracket(90.0, 50.0, 10.0, 6.0).unwrap();
        app.save(&path).unwrap();
        assert_eq!(
            Document::load(&path).unwrap().build().unwrap().volume(),
            7800.0
        );
        assert_eq!(
            Document::load(&Document::previous_path(&path))
                .unwrap()
                .build()
                .unwrap()
                .volume(),
            7200.0
        );
    }

    #[test]
    fn external_file_change_blocks_stale_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bracket.cadstrike");
        let mut app = CadStrikeApp::default();
        app.new_bracket(80.0, 50.0, 10.0, 6.0).unwrap();
        app.save(&path).unwrap();
        app.set_bracket(90.0, 50.0, 10.0, 6.0).unwrap();
        let external = Document::bracket(100.0, 50.0, 10.0, 6.0).unwrap();
        std::fs::write(&path, serde_json::to_vec_pretty(&external).unwrap()).unwrap();
        assert!(app.save(&path).unwrap_err().contains("changed on disk"));
        assert_eq!(Document::load(&path).unwrap(), external);
    }
}
