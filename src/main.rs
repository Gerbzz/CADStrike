use cadstrike::part::Document;
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let parse = |s: &std::ffi::OsString| {
        s.to_string_lossy()
            .parse::<f64>()
            .map_err(|_| "dimensions must be numbers in millimetres".to_string())
    };
    let result: Result<String, String> = match args.as_slice() {
        [command, width, height, leg, depth, path] if command == "new-bracket" => (|| {
            Document::bracket(parse(width)?, parse(height)?, parse(leg)?, parse(depth)?)?
                .save_new(Path::new(path))?;
            Ok(format!("saved parametric bracket: {}", Path::new(path).display()))
        })(),
        [command, radius, depth, path] if command == "new-disk" => (|| {
            Document::disk(parse(radius)?, parse(depth)?)?.save_new(Path::new(path))?;
            Ok(format!("saved parametric disk: {}", Path::new(path).display()))
        })(),
        [command, path] if command == "info" => (|| {
            let document = Document::load(Path::new(path))?;
            let solid = document.build()?;
            let (bounds_min, bounds_max) = solid.bounds();
            serde_json::to_string(&serde_json::json!({
                "units": document.units, "profile": document.profile,
                "depth_mm": document.depth_mm, "volume_mm3": solid.volume(),
                "bounds_min": bounds_min, "bounds_max": bounds_max,
                "triangles": solid.triangles.len(), "manifold": solid.manifold(),
                "sketch_id": document.sketch_id, "extrusion_id": document.extrusion_id
            })).map_err(|e| e.to_string())
        })(),
        [command, input, output, width] if command == "set-width" => (|| {
            let mut document = Document::load(Path::new(input))?;
            document.set_width(parse(width)?)?;
            document.save_new(Path::new(output))?;
            Ok(format!("saved resized part: {}", Path::new(output).display()))
        })(),
        [command, input, output] if command == "export-stl" => (|| {
            let document = Document::load(Path::new(input))?;
            let report = document.export_stl_new(Path::new(output))?;
            serde_json::to_string(&report).map_err(|e| e.to_string())
        })(),
        _ => Err("usage: cadstrike new-bracket <width_mm> <height_mm> <leg_mm> <depth_mm> <new.cadstrike> | new-disk <radius_mm> <depth_mm> <new.cadstrike> | info <in.cadstrike> | set-width <in.cadstrike> <out.cadstrike> <width_mm> | export-stl <in.cadstrike> <new.stl>".into()),
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
