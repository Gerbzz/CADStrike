# CADStrike drawing foundation candidate

This is an isolated Rust desktop candidate based on CADCraft commit `c57879a5c8f5e3a6be574a820add5e28eb25e9b1`. It provides editable 2D drafting, dimensions, constraints and DXF through CADCraft's engine and egui application. The original CADStrike bracket/disk editor remains in the parent directory.

On Windows with a current Rust toolchain and desktop build prerequisites, run:

```powershell
cargo test --locked -p cadstrike-project --test legacy
cargo test --locked -p cadcraft-ui-egui --lib selected_cadstrike_part_opens_a_separate_2d_drawing
cargo run --locked -p cadstrike
```

Use **File → Import CADStrike 2D Sketch (No Solid)...** to choose an existing `.cadstrike` v1 bracket or disk. The command validates the source and opens a separate editable 2D drawing without changing the original project. Save that drawing as DXF. The previous extrusion, 3D solid history and stable feature IDs are not yet represented as an editable CADStrike part in this candidate. No `.cadstrike` v2 container or assembly workflow is present.

The [migration note](../docs/CADCRAFT_FOUNDATION.md) describes the source boundary, tested fixtures and release blockers. The original donor README is retained as [UPSTREAM_README.md](UPSTREAM_README.md) for provenance, not as CADStrike feature or platform claims. Keep [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE), [NOTICE](NOTICE), [ATTRIBUTION.md](ATTRIBUTION.md) and [STRIKE-REDISTRIBUTION-NOTES.md](STRIKE-REDISTRIBUTION-NOTES.md) with this source. ArtCraft brand images and the upstream CADCraft app icons are excluded; the native icon is generated from original Strike source.
