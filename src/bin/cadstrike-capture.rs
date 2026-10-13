//! Evidence-only capture of the actual native editor renderer.
use cadstrike::ui::CadStrikeApp;
use eframe::egui;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

struct Capture {
    app: CadStrikeApp,
    output: PathBuf,
    frames: usize,
}

impl eframe::App for Capture {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Err(error) = save_png(&self.output, &image) {
                    eprintln!("capture failed: {error}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.frames += 1;
        if self.frames == 2 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(
        BufWriter::new(file),
        image.size[0] as u32,
        image.size[1] as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let rgba: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_array())
        .collect();
    writer.write_image_data(&rgba).map_err(|e| e.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let document = PathBuf::from(args.next().ok_or("missing .cadstrike input")?);
    let output = PathBuf::from(args.next().ok_or("missing .png output")?);
    if args.next().is_some() {
        return Err("usage: cadstrike-capture <input.cadstrike> <new.png>".into());
    }
    let mut app = CadStrikeApp::default();
    app.open(&document)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CADStrike capture",
        options,
        Box::new(move |_| {
            Ok(Box::new(Capture {
                app,
                output,
                frames: 0,
            }))
        }),
    )?;
    Ok(())
}
