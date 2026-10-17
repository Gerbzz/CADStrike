# CADStrike source migration redistribution notes

The donor `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, and `ATTRIBUTION.md` remain intact in this isolated worktree. Do not distribute the donor's excluded `docs/brand/` marks as the Strike identity.

The pinned `Cargo.lock` includes `acadrust` 0.6.3 under MPL-2.0. `cadcraft-io` reaches the native `cadcraft-dwg` bridge unconditionally, so a native CADStrike build must be treated as including that separate library until a reviewed feature boundary proves otherwise. Before distribution, include its MPL-2.0 notice and a way to obtain its corresponding unmodified source at the exact version; identify any local modifications to MPL files and make those files available under MPL-2.0 if modifications are ever made. The current migration does not modify `acadrust`.

The lock also includes `colored` 2.2.0, declared MPL-2.0. Determine whether it is in the actual Windows release graph and include the required notice and corresponding-source path if it is. Preserve donor attribution and assemble a complete resolved dependency and asset notice bundle after the native build graph is known. These are technical release tasks, not evidence that a distributable binary has been produced.
