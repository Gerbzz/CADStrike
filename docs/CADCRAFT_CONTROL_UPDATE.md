# Narrow CADCraft control transport update

This local source port follows [CADCraft `972ae7888cdeccfa3054b76c41518b5f9c22dbbd`](https://github.com/storytold/cadcraft/commit/972ae7888cdeccfa3054b76c41518b5f9c22dbbd), together with its missing transport prerequisite [bounded lines/connections, `61c018845a1a1f7019d496df4f5fcc1067c365e4`](https://github.com/storytold/cadcraft/commit/61c018845a1a1f7019d496df4f5fcc1067c365e4). The existing `ControlRequest::new`, `begin` and pending-reply cancellation API already matches the port, so the UI/engine request API and dependency graph need no changes.

Only `foundation/apps/cadcraft/src/control_server.rs` changes product code. The earlier explicit foundation launcher, import safeguards and Windows resource fixes are preserved. The imported base and craft-libs pins remain unchanged; this is a targeted source port, without a bulk upstream merge.

## Protocol behavior

The existing optional JSON-lines control server still binds only `127.0.0.1`, using the existing port selection and enablement. Neither network configuration nor external exposure is changed.

| Resource or input | Behavior |
| --- | --- |
| Simultaneous clients | At most 16; extra connections receive an error and close |
| Request line | At most 256 MiB, without buffering the unbounded remainder |
| Waiting requests | Queue of 64; overflow replies `busy` with `state: not-run` and the request ID |
| Idle socket | Closes after 600 seconds without input |
| Blocked reply write | Times out after 30 seconds |
| App reply | Existing 60-second wait and cancellation semantics retained |
| HTTP, malformed JSON, non-object, missing/non-string method or invalid UTF-8 | One rejection, then close; later input on that connection is not dispatched |
| Blank line | Skipped |
| Valid request | Preserves request ID; missing ID remains `null` |

Two compatibility corrections differ from the donor. HTTP detection runs only after JSON parsing fails: the donor's earlier detection could misclassify valid JSON ending in text such as `literal HTTP/1.1`. The bounded reader also allows both LF and CRLF terminators at the exact payload limit, validating payload length after stripping them. Each regression failed before its fix; HTTP body nondispatch and rejection of an extra payload byte still pass.

## Verification scope

The bounded transport was compiled and tested directly with installed Rust 1.96.1 against the already-built real `cadcraft-ui-egui`, `egui`, `serde_json` and `log` libraries. No dependencies, engine/UI crate or desktop executable were rebuilt, and no GUI was launched. The harness uses synthetic loopback sockets and an echo dispatcher, without invoking drawing commands or accessing user files.

The initial public-transport regression failed on the old source because an HTTP request's synthetic body reached the dispatcher (1 dispatch, expected 0). After the upstream port, all eight donor tests and that regression passed. Five additional tests cover partial-line timeout, a blocked write with an 8 MiB synthetic reply, overlong socket input, queued-request cancellation and string/null/missing IDs. The valid-JSON HTTP-text compatibility test then failed on the donor parser and passed after the correction. The completed harness has 15 passing tests: 14 in the real transport module and one public-API wrapper regression.

Exact rustc commands, cached-library hashes, source snapshots, test logs and exit codes are retained outside the repository under `control-evidence/`. The first harness link attempt lacked the cached Windows import-library search path; supplying that existing path resolved LNK1181 without installing or compiling anything new. Selected-file rustfmt and patch whitespace checks also pass.

Independent read-only review cleared the port with no unresolved findings. Its minor test-timing concern was corrected by keeping the blocked-write test's reply deadline beyond its completion assertion; the final 15-test receipt matches the resulting source hash. Tests use small line/queue limits and short deadlines where appropriate; the production 256 MiB line capacity was not allocated or benchmarked.

Publication review additionally identified the donor CRLF boundary above. A small-limit regression reproduced it, alongside the pre-existing renamed-desktop layer-registration defect. The expanded transport/layer harness covers both corrections. Hosted CI now also runs the full foundation `xtask ci` gate (formatting, clippy, workspace tests, asset attribution, layering and WASM checks). Its result is reported on the PR; the earlier focused receipts do not claim that full gate passed.

The full desktop Cargo test/check/build gates have not been rerun for this port while another project holds the heavy-build slot. Before treating this newer source as a native desktop build, run the existing Windows review gates, including:

```powershell
cargo test --locked --manifest-path foundation/Cargo.toml -p cadstrike --bin cadstrike
cargo check --locked --manifest-path foundation/Cargo.toml -p cadstrike
cargo build --locked --manifest-path foundation/Cargo.toml -p cadstrike --bin cadstrike
```

The earlier binary and Library capture remain matched to `6a104ffb7b915deb335dcefe2464f57b999b0d0f`; they are not runtime proof of this newer transport source. Its executable hash remains unchanged, as recorded in the source/build audit.

## Stored-reference update remains planning only

[CADCraft `8fe7c421ea95b986cedaa49314ce74f607a8a388`](https://github.com/storytold/cadcraft/commit/8fe7c421ea95b986cedaa49314ce74f607a8a388) changes xref and table data-link resolution, adds an engine `doc_path` module and requires `craft-docpath`. Its manifest points to `../secfix-craft-libs/crates/docpath`, which is not a usable dependency declaration in this isolated CADStrike workspace. The currently pinned craft-libs revision has no `crates/docpath/Cargo.toml` (public fetch returned 404).

That update needs a reviewed, resolvable dependency pin, source/license and platform behavior checks, then separate synthetic xref/data-link tests. No part of it or its dependency/lockfile changes is applied here.
