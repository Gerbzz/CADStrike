# CADStrike

CADStrike is a local Rust parametric part editor prototype by PWDS. Its current Windows source workflow creates dimension-driven bracket and disk sketches, keeps stable feature IDs, supports edit and undo/redo in the native UI, saves editable `.cadstrike` projects, and exports a binary STL reference. The canvas shows a 2D sketch outline; it does not show a rotatable 3D solid.

**Status:** source-only engineering checkpoint. The original local part workflow and tests were exercised on Windows, but this standalone source package has not been built or interactively reviewed. No binary, installer, account, or hosted service is included.

## Build and use

Install a Rust toolchain supporting edition 2024 and the Windows desktop prerequisites for eframe. From the repository root:

```powershell
cargo test --locked --offline
cargo run --locked --offline --features native-ui --bin cadstrike-ui
```

The CLI works without the native UI feature:

```powershell
cargo run --locked --offline -- new-bracket 80 50 10 6 bracket.cadstrike
cargo run --locked --offline -- info bracket.cadstrike
cargo run --locked --offline -- export-stl bracket.cadstrike bracket.stl
```

Remove `--offline` if required crates are not cached locally. Existing output files are not overwritten. Keep the editable project: STL carries triangles, not dimensions, stable IDs, or units; output coordinates use millimetres by convention.

## Scope and next steps

The source package contains the original bounded part workflow. Optional DXF inspection from the local prototype is omitted here because it depends on a much larger pinned donor workspace. The newer full CAD foundation migration remains separate local work and has no verified finished CADStrike workflow. Arbitrary sketches, geometric constraints, assemblies, STEP, general B-rep, 3D viewport, autosave recovery, and cross-platform acceptance remain open. Validate this standalone package, then migrate the verified features and complete native pointer, keyboard, scaling, and accessibility review before a release.

The app source is original first-party work. No donor code, logo, model, or font is bundled. Third-party Rust dependencies are listed in `Cargo.lock` and retain their own upstream licenses. No first-party source license has been selected for this checkpoint.