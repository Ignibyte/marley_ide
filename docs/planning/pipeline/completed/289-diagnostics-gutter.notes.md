# Editor diagnostics gutter — Notes

- **Forge ticket:** #289 `084d5945-7a7f-45d2-a04f-4453e25d9235`
- **AAR:** `9bc05f30-652b-4532-bbcd-1d7b8daacb73`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-289-diagnostics-gutter.md
- **Pipeline spec:** 289-diagnostics-gutter.spec.md

## Phase 1 — Plan
- **Request:** a failed block's file:line refs into the OPEN editor file → inline
  gutter markers. The "errors in the editor" wedge half; unblocks #290 + #292.
- **Classification / tier:** work pipeline slice, `feature`. Crates: `marley_app`
  (`links.rs` pure filter, `app.rs` shim + render marker). UI render → Validate uses
  the headless lane over `open_file_diagnostic_rows` + pure fixtures.
- **Forge recall:** #212 (SHIPPED) gives `scan_links` → `LinkTarget::File{path,line,col}`;
  #190 `resolve_under_root`; the #272 find-matches (`efind`) are computed as a sorted
  set before the uniform_list + captured into the 'static row closure — THE pattern to
  mirror (app.rs:3062); the gutter is `gutter_label`/`gutter_width` (code_view.rs).
- **Discovery (grounded):**
  - `code_view_body` (app.rs:3019) — the editor render; `gutter_width(total)` (3045);
    the row closure (3067) reads per-frame via `entity.read(app)`; `efind` (3062) is
    the sorted-set-captured-into-the-closure precedent (binary-searched per row via
    `partition_point`, 3127).
  - `block.rs` `output_text()` + `exit_code`; `block_status::exit_status_kind` → `StatusKind::Failure`.
  - `links.rs` `scan_links` + `LinkTarget::File{line}` (#212); `first_failure_ref` (#213, same idiom).
  - `marley_project::resolve_under_root` (#190).
- **Decisions:** D1–D4 in the spec. Crux: a PURE sorted row set (`diagnostics_for_file`,
  binary-searchable like `efind`) + a shim that sources it from the last failed block
  and captures it into the render — no new pump state (D3), reuse the existing gutter (D4).
- **Risk:** medium (render-touching). The pure filter is straightforward (reuse #212).
  The render marker must not disturb the #266/#272 row layout (the gutter already
  reserves `gw`); the capture mirrors `efind` exactly. The "last failed block across
  terminal panes" pick is a shim (app.rs excluded); the row set it produces is the tested seam.

## Phase 2 — Design

### Approach
A pure ref→row filter + an immutable pane iterator + a render capture mirroring #272's `efind`. §20
confirmed (Zed/VS Code inline-diagnostics gutter — our own filter + gutter tint). No new pump state.
- **`diagnostics_for_file(output, open_path, root) -> Vec<usize>`** (links.rs, pure): per `output.lines()`,
  `scan_links`, each `File{line: Some(l)}` whose `resolve_under_root(root, path) == open_path` → push
  `l.saturating_sub(1)` (1-based→0-based row); `sort_unstable` + `dedup`. Reuses #212 + #190.
- **`PaneGrid::states(&self)`** (workspace.rs): `self.panes.iter()` — the immutable sibling of `states_mut`.
- **`open_file_diagnostic_rows(&self)`** (app.rs shim): `open_path = active_editor().active_file().path`;
  for each `workspace().states()` terminal whose LAST block is `exit_status_kind == Failure`, extend with
  `diagnostics_for_file(last.output_text(), open_path, root)`; `sort`+`dedup`. LAST block only →
  self-clears on a new command; the union is order-independent (deterministic despite the HashMap).
- **Render** (`code_view_body`): `let ediag = self.open_file_diagnostic_rows()` before the uniform_list,
  captured into the row closure (like `efind`); the gutter `text_color` = `colors.danger` when
  `ediag.binary_search(&row).is_ok()`, else `colors.muted` (D4 — reuse `gutter_label(row+1, gw)`, no new geometry).

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/links.rs` | ADD pure `diagnostics_for_file(&str, &Path, &Path) -> Vec<usize>`. |
| `crates/marley_app/src/workspace.rs` | ADD `pub fn states(&self)` = `self.panes.iter()`. |
| `crates/marley_app/src/app.rs` | ADD `open_file_diagnostic_rows(&self)`; capture `ediag` + tint the gutter `danger` on a diagnostic row. |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | links.rs `diagnostics_for_file`: `error src/a.rs:12` + dup `src/a.rs:12` + other-file `src/b.rs:5`, open=a.rs, root=/ → `[11]` (deduped, b.rs excluded, 1-based→0-based). | REQ-001/002/004 |
| T2 | bare-path (no line) → excluded; empty → `[]`; out-of-order (`a.rs:9` then `a.rs:3`) → sorted `[2,8]`. | REQ-001/002/004 |
| T3 | Headless (#264 lane): a file + a failed block referencing it → `open_file_diagnostic_rows()` = the rows; a succeeding block → `[]`. | REQ-003/004 |
| — | cov/MSI 100 on `diagnostics_for_file` via `--diff`. | REQ-005 |

Uncoverable: `open_file_diagnostic_rows` + the render tint are app.rs-excluded shim; the row set is T1/T2, the end-to-end is T3; `states()` is exercised by T3.

### Risks / decisions
- **R1 (render):** the gutter tint reuses `gutter_label(row+1, gw)` in the existing `div().text_color(...)` — only the COLOR is conditional; the #266/#272 row layout is untouched. `ediag` capture mirrors `efind`.
- **D-source:** each pane's LAST block only → no stale-marker accumulation; #292 adds clear-on-green + the "which failure" refinement.
- **R2 (path):** `resolve_under_root` normalizes both sides; an other-file ref is excluded.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** `links.rs`: pure `diagnostics_for_file(output, open_path, root)` (per-line `scan_links` → `File{line:Some(l)}` resolving to `open_path` → `l.saturating_sub(1)`; sort+dedup) + the `Path` import. `workspace.rs`: `pub fn states(&self)` = `self.panes.iter()`. `app.rs`: `#[cfg_attr(test, mutants::skip)] open_file_diagnostic_rows(&self)` (each terminal pane's LAST block if `exit_status_kind == Failure` → union `diagnostics_for_file`; sort+dedup); captured `ediag` before the uniform_list (next to `efind`); the gutter `text_color` tints `colors.danger` when `ediag.binary_search(&row).is_ok()`, else `colors.muted`.
- `cargo check -p marley --all-targets` clean (the `move` closure captures `ediag` fresh per frame, like `efind`); `cargo fmt` clean; `cargo clippy` clean.

## Inspect (Phase 3.5)
Inline adversarial trace (a pure filter + an immutable iterator + a render tint). **One real finding, fixed.**

| Finding | Verdict | Resolution |
|---|---|---|
| **F1 — root mismatch could drop valid diagnostics** | **REAL (medium), FIXED** | `open_path` is the RESOLVED absolute path `load_code_view_state` built via `resolve_under_root(self.project_root, …)`. My shim resolved the refs via `active_project().root` instead — kept in sync (app.rs:1255/3656) but a distinct field; any sync-timing gap → the resolved ref ≠ open_path → a valid diagnostic silently excluded. Fixed: resolve against `self.project_root` (the SAME field), so the comparison is guaranteed consistent. `BF-claude-resolve-against-the-same-root-that-built-the-stored-path`. |
| Path match (the core) | SAFE (post-F1) | Both sides go through `resolve_under_root(project_root, …)` → an absolute `open_path` and a relative compiler ref (`src/a.rs:12`) resolve to the same absolute path → equal. Other-file refs resolve elsewhere → excluded. |
| 1-based → 0-based | SAFE | `l.saturating_sub(1)` (compiler lines 1-based; rows 0-based); line 0 → row 0 (saturating, no underflow). |
| sort + dedup | SAFE | `sort_unstable` + `dedup` → sorted (binary_search-ready) + one row per line; the shim re-sorts+dedups the cross-pane union. |
| Last-block self-clear | SAFE | Only each pane's LAST block, Failure only → a new command (Running/Success last block) clears the markers; a Running block is neither Success nor Failure (`block_status` reads the code only when Finished). |
| Render capture | SAFE | `ediag` (sorted `Vec<usize>`) moved into the 'static row closure like `efind`, recomputed fresh per frame in `code_view_body`; `binary_search(&row)` reads it; only the gutter `text_color` is conditional (layout untouched). |
| MSI | Phase-4-gated | `diagnostics_for_file` fn-replacement + the `== *open_path` (`==`→`!=`) + `saturating_sub(1)` (arg `1`→`0`) killed by T1/T2 (exact row values + the other-file exclusion). |

**Perf note (not a defect):** `open_file_diagnostic_rows` scans each failed block's FULL output every frame (once per render, not per row). Fine for typical compiler output (<few hundred lines); if a huge failed output janks, cache by (block-count, path) — a follow-up.

## Phase 4 — Validate
- **Tests added:** links.rs `diagnostics_for_file_filters_dedups_and_zero_bases` (T1/T2 — dup a.rs:12 → `[11]`, b.rs excluded, 1-based→0-based, out-of-order → sorted `[2,8]`, bare-path/empty/URL/other-file → `[]`); workspace.rs `states_yields_every_pane` (covers the new `states()`); headless `diagnostics_gutter_empty_without_a_failure_headless` (opens a file → `open_file_diagnostic_rows()` empty with no failure — exercises `active_editor` + `states()` in the real boot context).
- **`cargo nextest run -p marley` (the 3 tests): PASS.**
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15 incl. coverage 100% + MSI 100% on `diagnostics_for_file` (the `== *open_path`, `saturating_sub(1)`, fn-replacement all killed by T1/T2).
- **Live-drive: the non-empty gutter marker (a red line-number on a diagnostic row) is env-blocked (driven).** No app running + synthetic input blocked; injecting a REAL failed block with shell-integration exit codes into the #264 headless lane is out of reach. Carried by: `diagnostics_for_file` at MSI 100 (the exact rows), the headless shim smoke (the wiring runs), and the render tint being a trivial `if ediag.binary_search(&row).is_ok() { danger } else { muted }` over that proven set. Re-verify with a failing `cargo build` referencing an open file → the gutter line goes red, when unlocked.
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Added (#289); app_shell.md clickable-links section extended with the #289 diagnostics gutter.
- **Knowledge (forge):** failure `BF-resolve-against-same-root-that-built-stored-path-001` (`10306020`, F1); prevention rule `PR-claude-resolve-against-same-root-that-built-stored-path-001` (`22e91bec`, medium). AAR `9bc05f30` submitted: completed, effectiveness 5. Design lesson: the #272 `efind` "compute a sorted set before the uniform_list + capture into the 'static row closure + binary_search per row" is the reusable pattern for ANY per-row render overlay (find matches, diagnostics, and #290 will navigate the same set).
- **Ticket** TICKET-289 → closed/ + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** the diagnostics gutter shipped (errors light up the open file); unblocks #290 + #292. `diagnostics_for_file` cov/MSI 100, GATE GREEN [diff]. LOCAL commit only.
