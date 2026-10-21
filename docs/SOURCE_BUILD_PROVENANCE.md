# CADStrike source and build provenance — 2026-10-21

## Finding

Public `main` was reverified at `845267650898615ff892eee477e8cfacf514ceba`, the PR #2 merge, on the authorized Windows host. It contains both the original root bracket/disk editor and the actual imported CADCraft workspace under `foundation/`. The README's primary desktop build command still selected the root legacy `cadstrike-ui` binary. That was a reproducible source-to-launch guidance mismatch and can expose the old interface even when the imported source is present.

The correction makes `scripts/Start-CADStrike.ps1` the primary documented Windows entry point. It selects the foundation manifest, package and binary explicitly, keeps output under `foundation/target`, and uses Cargo to build as needed before running. It embeds Git revision identity through the desktop's existing `CADCRAFT_BUILD_SHA` hook. `-PlanOnly` runs no Cargo command; `-BuildOnly` opens no app. The original root editor and CLI remain available, including their editable legacy files and STL reference export.

## Donor source mapping

The pinned CADCraft base is [`storytold/cadcraft@c57879a5c8f5e3a6be574a820add5e28eb25e9b1`](https://github.com/storytold/cadcraft/commit/c57879a5c8f5e3a6be574a820add5e28eb25e9b1). `craft-ui` remains pinned to `storytold/craft-libs@5c874d51eeb3de70e735c8ce87a59a7dedd47675` in the foundation workspace.

A Git blob-ID comparison of `main`'s `foundation/` tree against the complete pinned donor tree found:

- 516 imported files byte-identical to the donor.
- 21 donor files modified for CADStrike integration and identity.
- 7 added files: redistribution notes, preserved upstream README, original icon source, the legacy-project converter crate and tests, and the engine import command.
- 76 donor files excluded, confined to `.cargo`, `.github`, `CLAUDE.md`, assets, documentation, examples and packaging.

This is an imported foundation with localized integration changes. It is not merely a thin custom substitute. Git ancestry alone does not prove a copied-source import; the blob mapping establishes the retained donor content. This comparison does not establish that any installed binary was built from it.

## Upstream update status

Official CADCraft `main` was last reverified at [`0cf3bdea83ebbe02fb2e674b23f07de4acfa73da`](https://github.com/storytold/cadcraft/commit/0cf3bdea83ebbe02fb2e674b23f07de4acfa73da), dated 2026-10-21 07:40:18 UTC. GitHub reports it 232 commits ahead of the imported base. It advanced by two commits during this audit; those two commits only change `.github/workflows/nix.yml`, switching to shared Nix CI, with no application-source delta from the previously inspected `7fef8cb` head.

The existing donor pin has not been advanced. The broader delta requires compatibility review and focused tests around the retained CADStrike import, settings, UI and identity changes before integration. The coordinating task is auditing that detailed upstream delta separately; this branch does not claim the 232-commit update is integrated or tested.

## Room PC evidence and limits

No matching CADStrike/CadCraft process or named desktop/Start Menu shortcut was found in the inspected Room user and shared locations. No existing CADStrike/CadCraft checkout was found in the Room user's Codex project directories or restored PWDS-Strike directory. A fresh isolated checkout was therefore created in this task's workspace on `fix/cadcraft-launch-provenance`; existing projects and branding assets were not modified.

**The exact executable the user saw could not be identified.** The old README launch command is a demonstrated selection problem, not proof of a specific stale installed executable or shortcut. The first CIM process query was unavailable under the Windows permissions; the independent `Get-Process` and file checks succeeded.

Lightweight PowerShell checks cover the foundation selection, separate target directory, spaces/apostrophes in argument forwarding, no-launch modes, Cargo failure propagation and caller-environment restoration. Cargo is intercepted in those tests. The initial launcher-only checkpoint had no native build or app launch; the completed build and native runtime evidence are recorded below. This candidate was reviewed in isolation. No existing installed app or shortcut was redirected, and the earlier executable seen by the user remains unidentified.

## Import-safeguard continuation

The historical local `fdaa6ab3b57dbe1067d171a343404c900b42a9e9` commit is not available from the public CADStrike repository (GitHub returned 422, no commit found). The observed requirements were reimplemented on the isolated branch: imported geometry remains unsaved; the native host supplies a dedicated `.cadstrike` picker; cancellation preserves the original drawing; programmatic imports do not open a picker. Hosts without the picker report a visible status/history message. The legacy root app, donor pin, existing branding assets and full imported feature set remain unchanged.

An independent read-only review cleared the launcher and six-file safeguard change for build after one visible-feedback issue was fixed. Rust formatting, whitespace checks and the intercepted-Cargo launcher tests pass. Using installed Rust 1.96.1/MSVC, locked dependencies, two build jobs and a separate foundation target directory:

- The engine regression first failed at the new unsaved-import assertion with the fix temporarily omitted (2 passed, 1 failed, exit 101). Restoring the reviewed fix made all 3 engine import tests pass.
- All 6 legacy converter tests passed, including stable IDs, unchanged source bytes and native DXF round-trip preservation.
- The UI regression gate initially deferred to other Cargo jobs. Once the single build slot was available, deliberately routing imports through the generic drawing picker produced the expected failures (2 passed, 3 failed, exit 101). Restoring the dedicated picker made all 5 UI import tests pass. Every temporary regression mutation was restored.

## Completed native verification

The build exposed an additional Windows identity defect: `winresource` escaped an apostrophe in the user-directory path, causing the Windows resource compiler to report RC2135 for the icon path. The build script previously tolerated that error and omitted the icon/version resources. The fix emits the same icon, resource ID 1, through the documented raw resource-content API with correct filename quoting. The launcher and CI now require successful resource compilation. No artwork was changed. An independent review cleared this follow-up after a small documentation correction.

The source-matched desktop was built from clean commit `6a104ffb7b915deb335dcefe2464f57b999b0d0f`. Earlier converter, engine and UI receipts retain their exact source snapshots; the later follow-up changes the Windows build resources, launcher, CI and documentation, not the tested import implementation.

| Gate | Result |
| --- | --- |
| Legacy sketch converter | 6 passed |
| Engine import safeguards | 3 passed; unsaved regression reproduced before restoring the fix |
| UI import safeguards | 5 passed; wrong-picker regression reproduced before restoring the fix |
| Donor dialog dispatch integration | 5 passed |
| Resource-required native desktop check | Passed |
| Native desktop tests | 36 passed |
| Foundation desktop build | Passed |
| Legacy root workflow tests | 3 passed |
| Intercepted-Cargo launcher tests, selected Rust formatting and whitespace checks | Passed |

These are 58 focused Rust test cases, plus the check/build and launcher gates. They do not claim exhaustive workspace or cross-platform acceptance. Logs, commands, source patches and exit codes are retained beside this checkout under `build-evidence/`.

The resulting `foundation/target/debug/cadstrike.exe` is 52,771,328 bytes, SHA-256 `212E992CAB2C631077404375FD1A82E5CEC4840A7C4EEB7621D436E53E2E7FD5`. Windows metadata identifies CADStrike 0.5.0 with the description `CADStrike local 2D drafting candidate, based on CADCraft`. Executing `--version` returned:

```text
CADStrike 0.5.0 (6a104ffb7b915deb335dcefe2464f57b999b0d0f; CADCraft foundation)
```

That exact local executable launched a native window titled CADStrike. Its documented loopback control API reported GPU rendering and captured a 1600 x 1000 PNG of the complete native editor displaying the donor sample drawing. The image was visually reviewed and delivered privately through Library. This is a capture of the running native foundation window. The retained README image still depicts the earlier root editor.

The same native process imported a task-owned legacy sketch into a second drawing with six entities. Inspection confirmed both drawings remained present, the import was dirty, and Close displayed an unsaved-drawing prompt. The legacy source hash remained unchanged. The test application then exited cleanly through its documented API. Native receipts, image identity and source/binary hashes are retained beside this checkout under `native-review/`.

The operating-system file picker was not manually driven; its dedicated callback and `.cadstrike` filter are covered by source review and focused tests. Broader pointer, keyboard, scaling and accessibility acceptance remains open. This is a 2D drafting candidate, without a claim of general 3D solids. C: remained above 2 TB free, exceeding the required 20 GiB floor. No installer or downloaded prebuilt app was run, no existing installation was replaced, and no push, merge or release occurred. No OS execution denial or pending execution approval remains.

## Subsequent targeted upstream source port

The separate `fix/cadcraft-control-server-bounds` branch ports CADCraft `972ae788` and its `61c01884` line/connection prerequisite into the control transport, with no dependency or bind-setting changes. Fifteen transport-only tests pass against the real cached libraries, including HTTP-body nondispatch and queue, connection, line and timeout bounds. The [control update record](CADCRAFT_CONTROL_UPDATE.md) identifies the exact donor commits, one parser compatibility correction, tests and remaining desktop gates. The desktop executable and native capture above remain at `6a104ff`; they have not been rebuilt for this source-only update. Stored-reference commit `8fe7c421` remains planning only because it requires separate craft-docpath dependency and behavior review.
