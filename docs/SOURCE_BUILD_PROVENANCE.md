# CADStrike source and build provenance — 2026-10-21

## Finding

Public `main` was reverified at `845267650898615ff892eee477e8cfacf514ceba`, the PR #2 merge, on the authorized Room PC (`DESKTOP-A4P8DV3`). It contains both the original root bracket/disk editor and the actual imported CADCraft workspace under `foundation/`. The README's primary desktop build command still selected the root legacy `cadstrike-ui` binary. That was a reproducible source-to-launch guidance mismatch and can expose the old interface even when the imported source is present.

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

Official CADCraft `main` was reverified at [`7fef8cbe2fb8dbafdcf6ebda8aa3ba0623bcbec1`](https://github.com/storytold/cadcraft/commit/7fef8cbe2fb8dbafdcf6ebda8aa3ba0623bcbec1), dated 2026-10-20. GitHub reports it 230 commits ahead of the imported base. The existing donor pin has not been silently advanced. That broader delta requires compatibility review and focused tests around the retained CADStrike import, settings, UI and identity changes before integration. The coordinating task is auditing the detailed upstream delta separately.

## Room PC evidence and limits

No matching CADStrike/CadCraft process or named desktop/Start Menu shortcut was found in the inspected Room user and shared locations. No existing CADStrike/CadCraft checkout was found in the Room user's Codex project directories or restored PWDS-Strike directory. A fresh isolated checkout was therefore created in this task's workspace on `fix/cadcraft-launch-provenance`; existing projects and branding assets were not modified.

**The exact executable the user saw could not be identified.** The old README launch command is a demonstrated selection problem, not proof of a specific stale installed executable or shortcut. The first CIM process query was unavailable under the Windows permissions; the independent `Get-Process` and file checks succeeded.

Lightweight PowerShell checks cover the foundation selection, separate target directory, spaces/apostrophes in argument forwarding, no-launch modes, Cargo failure propagation and caller-environment restoration. Cargo is intercepted in those tests. No substantial build, dependency installation, application launch, release, merge or deployment was performed for this correction. No new runtime screenshot or Library screenshot ID is claimed. A fresh source-matched foundation build and safe native runtime review remain required before asserting that the user's displayed app is corrected.
