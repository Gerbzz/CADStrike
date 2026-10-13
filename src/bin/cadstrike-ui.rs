fn main() -> eframe::Result<()> {
    eframe::run_native(
        "CADStrike",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(cadstrike::ui::CadStrikeApp::default()))),
    )
}
