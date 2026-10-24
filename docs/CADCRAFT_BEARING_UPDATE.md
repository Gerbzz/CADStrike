# Narrow CADCraft surveyor-bearing correction

This backports the three-line range check from official CADCraft commit
[`0fc8e5241c6bad35b7ca66cece328dab137afeab`](https://github.com/storytold/cadcraft/commit/0fc8e5241c6bad35b7ca66cece328dab137afeab),
"Reject surveyor bearing angles outside 0-90 degrees" (October 24, 2026).
The change is in `foundation/crates/engine/src/units.rs`; the regression test is
adapted and expanded from that commit's `crates/engine/tests/units_bearing.rs`.
The donor code remains under the retained MIT OR Apache-2.0 licenses.

## Source selection and scope

The starting point is CADStrike `2c640d68fa9884f030678a00aadffef49cefa319`.
Its real CADCraft drawing application remains the `foundation/` workspace,
with CADCraft 0.5.0 donor `c57879a5c8f5e3a6be574a820add5e28eb25e9b1` and
craft-libs `5c874d51eeb3de70e735c8ce87a59a7dedd47675`. The launcher still selects
`foundation/Cargo.toml`, package and binary `cadstrike`, not the retained root
legacy editor. This source selection does not identify or replace any installed
desktop executable.

Official [CADCraft v0.12.0](https://github.com/storytold/cadcraft/releases/tag/v0.12.0)
was published October 23, 2026. This correction was merged **after** that release;
it is a targeted source backport, not an upgrade to v0.12.0 or current upstream.
No manifests, lockfiles, version numbers, drawing formats, UI, control transport,
native identity or license files change. The bracket/disk converter and its
source-preserving import and unsaved-drawing guards remain unchanged.

## Behavior

A surveyor's bearing specifies an angle from north or south toward east or west.
The parsed total angle must be between 0 and 90 degrees, inclusive. Previously,
`N100E` produced a direction of -10 degrees and `N-45E` produced 135 degrees,
silently placing them outside their named quadrant. They now return a parse
failure instead. A DMS total just above 90 or below 0 is rejected as well.

Valid boundary and quadrant directions, case/space handling, and the absolute
bearing convention are preserved, independent of AUNITS, ANGBASE and ANGDIR.
Ordinary negative or greater-than-90-degree angles remain valid. Nonfinite
angles were already rejected by the existing parser; this does not introduce
that behavior. This is total bearing-angle validation, not a rewrite of DMS
syntax: existing minute/second normalization remains unchanged.

## Verification

The new integration test was run against the unchanged engine first: one test
passed and two failed, with `N100E` and `N91E` incorrectly returning directions.
It uses the real `cadcraft-engine` crate, not a substituted parser.

Repeat the focused and adjacent angle checks with the unchanged lockfile:

```sh
cargo test --locked --manifest-path foundation/Cargo.toml -p cadcraft-engine --test units_bearing --test angle_settings --test angle_prompts
cargo test --locked --manifest-path foundation/Cargo.toml -p cadcraft-engine --lib
cargo test --locked --manifest-path Cargo.toml
```

The existing `foundation-review.yml` remains the complete review gate: foundation
formatting, clippy, workspace tests, asset and layering checks, WASM, legacy
conversion/import coverage, Windows desktop build and exact executable identity.
A focused parser pass is not a full gate pass or native interactive acceptance;
the PR's checks establish the status for its exact commit. No release, deployment
or installed-app acceptance is part of this patch.
