//! Windows only: embed the original local CADStrike icon and version info.

#[path = "../../assets/strike_icon.rs"]
mod strike_icon;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/strike_icon.rs");
    println!("cargo:rerun-if-env-changed=CADCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CADCRAFT_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR")).join("cadstrike.ico");
    std::fs::write(&icon, strike_icon::ico(strike_icon::Kind::Cad)).expect("write generated CADStrike icon");
    let mut res = winresource::WindowsResource::new();
    // winresource 0.1.31 adds an escape before apostrophes, which rc.exe reads as a
    // different path (for example in Neil's Room). Quote the generated icon as
    // an RC filename directly, retaining literal apostrophes and escaping only
    // backslashes and double quotes. The artwork and icon resource ID stay the same.
    let icon_path = icon.to_string_lossy().replace('\\', "\\\\").replace('"', "\"\"");
    res.append_rc_content(&format!("1 ICON \"{icon_path}\"\n"));
    res.set("ProductName", "CADStrike")
        .set("FileDescription", "CADStrike local 2D drafting candidate, based on CADCraft")
        .set(
            "LegalCopyright",
            "CADCraft code copyright (c) 2026 ArtCraft Team and contributors; MIT OR Apache-2.0. CADStrike changes separately attributed.",
        )
        .set("OriginalFilename", "cadstrike.exe")
        .set("InternalName", "cadstrike");
    if let Err(e) = res.compile() {
        if std::env::var_os("CADCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=cadstrike.exe built without icon/version resources: {e}");
    }
}
