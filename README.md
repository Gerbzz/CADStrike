<p align="center">
  <img src="assets/branding/CADStrike.png" alt="Original CADStrike stag beetle logo" width="160" height="160">
</p>

<h1 align="center">CADStrike</h1>

<p align="center">
  <a href="https://prowebdevsolutions.com"><picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/PWDS-logo-orange.svg">
    <img src="assets/branding/PWDS-logo.png" alt="PWDS company logo" width="120">
  </picture></a>
</p>

**Give an idea its dimensions.**

## CADCraft foundation review branch

The [foundation](foundation/) directory is an isolated CADStrike desktop candidate built from pinned CADCraft source. It includes CADCraft's editable 2D drawing engine, constraints, DXF persistence and native editor, plus a selected-file import of this repository's bracket and disk sketches into a separate drawing. The existing bracket editor at the repository root remains available during review. [Migration details and limits](docs/CADCRAFT_FOUNDATION.md) explain how to run both workflows and what has been verified.

CADStrike starts a simple part with dimensions instead of a fixed drawing. Create bracket or disk sketches, edit features and undo changes, save the `.cadstrike` project, and export a binary STL reference.

![CADStrike native editor showing a dimensioned six-edge bracket sketch](assets/screenshots/cadstrike-bracket-editor.png)

*A retained Windows editor opened a self-authored 120 × 80 × 16 mm bracket project at 10 mm depth. The preview is a 2D sketch outline.*

## Highlights

- Create dimension-driven bracket and disk parts.
- Keep stable feature IDs in an editable project.
- Export a binary STL for inspection.

## Current status

Windows engineering checkpoint. The original local workflow was exercised, and a retained capture build whose four depfile source files exactly match this public repository opened a self-authored bracket in the native editor. This standalone package has not been freshly built or interactively reviewed. The canvas shows a 2D sketch outline, not a rotatable solid; broader CAD formats and assemblies remain future work. [Screenshot provenance](docs/SCREENSHOT_PROVENANCE.md) records the bounded capture.

## Keep the editable part

The `.cadstrike` project preserves dimensions and stable feature IDs so a part can be revised. STL export is a triangle reference: it does not carry dimensions, IDs, or intrinsic units. Output coordinates use millimetres by convention, and export does not overwrite an existing file. Keep the project for later edits; the native canvas currently shows a 2D outline.

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

The root source package contains the original bounded part workflow. Its optional DXF inspection remains outside this root crate. The separate `foundation/` workspace brings a pinned CADCraft 2D drawing candidate into this branch, with a visible legacy sketch import; it is still short of a unified editable CADStrike part workflow. General 3D parametric features, assemblies, STEP, B-rep solids, and cross-platform acceptance remain open. Complete native pointer, keyboard, scaling, and accessibility review before a release.

The app source is original first-party work. No donor code, logo, model, or font is bundled. Third-party Rust dependencies are listed in `Cargo.lock` and retain their own upstream licenses. No first-party source license has been selected for this checkpoint.

## PWDS brand artwork

The company mark in this masthead is the existing [official PWDS logo](https://github.com/Gerbzz/PWDS-Website/blob/main/public/prowebdevsolutions-logo.png), with the [official orange display asset](https://prowebdevsolutions.com/images/prowebdevsolutions-logo-orange.svg) for dark backgrounds. Any app logo above it remains the original Strike suite artwork.
