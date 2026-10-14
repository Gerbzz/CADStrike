# CADStrike screenshot provenance

`assets/screenshots/cadstrike-bracket-editor.png` is a 1280 × 820, uncomposited image made by the app's retained `cadstrike-capture.exe` from its actual native renderer. SHA-256: `209CC6A9614EB73D42B7C888A2DE874B8000C0711DA17D523B4E71EEB1826777`.

The capture executable has SHA-256 `43F20D6E0DCF9A384A6A6FF218C6DFD885BD4477BC7D7911C89EF8843F968A8B`. Its four depfile inputs (`src/bin/cadstrike-capture.rs`, `src/lib.rs`, `src/part.rs`, `src/ui.rs`) exactly match the public repository's blobs at this update. The capture path opened a self-authored 120 × 80 mm six-edge bracket with a 16 mm leg and 10 mm extrusion depth. The retained CLI independently reported a 20-triangle manifold solid and 29,440 mm³ volume for that project. The image shows the native **XY sketch outline** and driving dimensions, not a 3D shaded viewport. The fixture `.cadstrike` file is not distributed with the repository.

This run verifies opening and rendering the sample project. It does not verify native pointer editing, file save, STL export, accessibility, or a newly built standalone checkout. No new build, test suite, dependency download, or third-party model was used for this screenshot.
