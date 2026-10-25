# Extreme-view grid overflow backport — 2026-10-25

## Origin and retained source

- Selected CADCraft foundation: `c57879a5c8f5e3a6be574a820add5e28eb25e9b1`.
- Narrow upstream fix: [storytold/cadcraft 8343045863dc1ace20ee3e3e378cdac639dbcf94](https://github.com/storytold/cadcraft/commit/8343045863dc1ace20ee3e3e378cdac639dbcf94), contributor MarekSpodBiedry, upstream PR #979 / issue #920.
- Source path: `crates/ui-egui/src/canvas.rs`; retained path: `foundation/crates/ui-egui/src/canvas.rs`.
- Strike base: `f64b0ecf95e98b150365802ed32be0429f116038`. The retained canvas source was byte-identical to the selected foundation before this patch; its LF line endings are preserved.
- Production change: upstream `grid_index_ranges` checked subtraction plus its `draw_grid` call site. The comment clarifies the existing index-span threshold; arithmetic and loop bounds match the donor. Both axes still require a span below 2000 (at most 2000 inclusive indices). Extreme saturated ranges are skipped rather than overflowing.

## License and preservation

Upstream is `MIT OR Apache-2.0`. The imported helper and donor regression retain [MIT](../foundation/LICENSE-MIT) and [Apache-2.0](../foundation/LICENSE-APACHE) notices, copyright 2026 ArtCraft Team and CADCraft contributors. Both notices are byte-identical at the donor commit and retained foundation.

Foundation/craft-libs pins, Cargo.lock, CADStrike identity/branding, legacy conversion, previous bearing and DXF-unit fixes, isometric grid behavior and existing workflows are unchanged. This is a narrow backport, not a full upstream update.

## Source-matched regression

The headless test calls actual `draw_grid` with ordinary and extreme view transforms. It explicitly clears unused egui texture deltas rather than pretending to render them. On exact test-only source `4e636a3c91ce78ca68839609606849fc2e571a60`, the ordinary view passed and the extreme view failed at the real subtraction with overflow ([run 40228383670](https://github.com/Gerbzz/CADStrike/actions/runs/40228383670), job 121274637818, 2026-10-25 20:09 UTC).

Earlier run 40227221983 found both the expected overflow and an unrelated test-harness texture-delta drop panic; the harness was repaired before the clean red receipt above. Initial workflow YAML quoting and test formatting issues were also corrected, not counted as product failures.

The fixed source additionally tests donor normal/extreme ranges and the existing 1999/2000 index-span boundary on both axes. The dedicated immutable-head workflow runs:

```
cargo test --locked -p cadcraft-ui-egui --lib grid_overflow_tests::
cargo test --locked -p cadcraft-ui-egui --lib
cargo test --locked -p cadstrike-project --test legacy
```

The unchanged full foundation review runs formatting, Clippy, tests, assets/layers/WASM and native/legacy gates. Final hosted green checks and independent review remain required; this red receipt alone is not delivery or visual/released-binary acceptance.
