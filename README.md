# CADStrike

**by PWDS**

**Give an idea its dimensions.**

CADStrike starts a simple part with dimensions instead of a fixed drawing. Create bracket or disk sketches, edit features and undo changes, save the `.cadstrike` project, and export a binary STL reference.

### Highlights

- Create dimension-driven bracket and disk parts.
- Keep stable feature IDs in an editable project.
- Export a binary STL for inspection.

## Current status

Source-only Windows engineering checkpoint. The original local workflow was exercised, but this standalone package has not been built or interactively reviewed. The canvas shows a 2D sketch outline, not a rotatable solid; broader CAD formats and assemblies remain future work.

## Shape a part

Start with a dimension-driven bracket or disk, edit its features, and undo a change if needed. Save a `.cadstrike` project to preserve the dimensions, then export a binary STL reference for inspection. The current canvas is a 2D sketch view.

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
