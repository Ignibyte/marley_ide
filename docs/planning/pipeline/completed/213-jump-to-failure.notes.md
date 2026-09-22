# A failing command → Jump to Failure — Notes

- **Forge ticket:** #213 `6c23355e-5688-404b-88e5-05fefa772aa9`
- **AAR:** `ea72f711-876f-44db-803f-98b07dd18c30`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-213-jump-to-failure.md
- **Pipeline spec:** 213-jump-to-failure.spec.md

## Phase 1 — Plan
- **Request:** a failed command block gains a "Jump to Failure" action opening the
  editor at the primary failure line. The run→fix half of the wedge; reuses #212.
- **Classification / tier:** work pipeline slice, `feature`. Crate: `marley_app`
  (`links.rs` pure ref-pick, `context_menu.rs` pure menu row, `app.rs` shim). UI
  path → Validate drives / mechanism-carries.
- **Forge recall:** #212 (SHIPPED this session) gives `scan_links` (File{line,col})
  + `open_file_at`; #175 gives the block context menu (`context_menu.rs`
  `MenuAction`/`BLOCK_MENU_ITEMS`/`items_for`/`MenuKind::Block`); `block_status.rs`
  `exit_status_kind` for failed-detection; `block.rs` `output_text()` for the output.
- **Discovery (grounded):**
  - `context_menu.rs:10` `MenuAction` (6 variants); `:49` `BLOCK_MENU_ITEMS[6]`; `:59`
    `items_for(kind)`; `:33` `MenuKind::Block{pane,block}`.
  - `block.rs:113` `Block::output_text() -> String`; `:41` `ExitCode(Option<i32>)`.
  - `block_status.rs:26` `exit_status_kind(state, exit) -> StatusKind` (the failed/danger kind).
  - `links.rs` `scan_links(line) -> Vec<Link>` with `LinkTarget::File{path,line,col}` (#212).
- **Decisions:** D1–D4 in the spec. Crux: reuse #212 end-to-end; the only new logic
  is `first_failure_ref` (first line-carrying ref) + the conditional 7th menu row
  gated on `has_failure` (a new `MenuKind::Block` field).
- **Risk:** low. `MenuKind::Block` gains a field → touches its construction sites
  (grep at design; likely 1-2) + `items_for` + the render dispatch. The pure ref-pick
  reuses the #212 scanner. No new parse/open path.

## Phase 2 — Design

### Approach
Two pure seams + a conditional menu row + a shim dispatch, all reusing #212.
§20 confirmed (IDE "jump to first error" — our own ref-pick + menu). No new
parsing/open path.

- **`first_failure_ref(output: &str) -> Option<(PathBuf, usize, Option<usize>)>`** (links.rs):
  `output.lines()`, `scan_links(line)`, return the FIRST `LinkTarget::File { path, line: Some(l), col }`
  → `(path, l, col)`. A bare-path ref (no line) is skipped (not a jump target).
- **`MenuAction::JumpToFailure`** + **`BLOCK_MENU_ITEMS_FAILED: [_; 7]`** (Jump-to-Failure first, then
  the existing 6) + **`MenuKind::Block { pane, block, has_failure: bool }`** + `items_for` →
  `Block { has_failure: true, .. } => &BLOCK_MENU_ITEMS_FAILED` else the 6-row table.
- **app.rs shim:** at the block-menu open (≈7435), `has_failure = exit_status_kind(state, exit) ==
  StatusKind::Failure && first_failure_ref(block.output_text()).is_some()`; the dispatch extraction
  (≈3543) becomes `MenuKind::Block { pane, block, .. }`; a new `MenuAction::JumpToFailure` arm reaches
  the block (`blocks().nth(block)`), `first_failure_ref(output_text())` → `open_file_at(resolve_under_root(root, path), line, col)`.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/links.rs` | ADD pure `first_failure_ref(&str) -> Option<(PathBuf, usize, Option<usize>)>`. |
| `crates/marley_app/src/context_menu.rs` | ADD `MenuAction::JumpToFailure`; `BLOCK_MENU_ITEMS_FAILED[7]`; `MenuKind::Block` gains `has_failure: bool`; `items_for` branches. |
| `crates/marley_app/src/app.rs` | block-menu open computes `has_failure`; the `MenuKind::Block` extraction adds `..`; a `MenuAction::JumpToFailure` dispatch arm → `first_failure_ref` → `open_file_at`. |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | links.rs `first_failure_ref`: multi-line (`note: ...\nsrc/a.rs:12:5 error\nsrc/b.rs:9`) → the FIRST line-carrying ref `(a.rs,12,Some(5))`; a bare-path-only output (`src/x.rs` no line) → None; empty → None; a URL-only line → None. | REQ-001 |
| T2 | context_menu.rs `items_for`: `Block{has_failure:true}` → 7 rows, `[0]==JumpToFailure`; `Block{has_failure:false}` → the 6-row `BLOCK_MENU_ITEMS`; `Split` → `MENU_ITEMS`. (Non-degenerate: distinct row counts + the first-row identity.) | REQ-002 |
| T3 | Driven capture: a failing `cargo`/`grep` block → right-click → "Jump to Failure" → the editor opens at the ref line. Env-blocked → mechanism (reuses #212 `open_file_at`, unit-proven; the row-gating is T2). | REQ-003 |
| — | cov/MSI 100 on `first_failure_ref` + `items_for` via `--diff`. | REQ-004 |

Uncoverable: the app.rs dispatch arm + has_failure computation are `#[cfg_attr(test, mutants::skip)]`-class shim (live menu + editor open); covered behaviorally by T3 / mechanism; the decision logic is T1 (ref pick) + T2 (row gating).

### Risks / decisions
- **R1:** `MenuKind::Block` gains a field → 2 sites (construction ≈7435, extraction ≈3543). A missed one fails to compile. The context_menu.rs tests constructing `MenuKind::Block` (if any) also update.
- **D-first-ref:** "primary = first line-carrying ref" (D1) — simple + matches compiler/test output order; a scored error-vs-warning rank is a noted follow-up.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** `links.rs`: pure `first_failure_ref(output)` (`lines().find_map` → `scan_links` → first `File{line:Some}`). `context_menu.rs`: `MenuAction::JumpToFailure` + `BLOCK_MENU_ITEMS_FAILED[7]` (Jump-to-Failure first) + `MenuKind::Block{...,has_failure}` + `items_for` branches on `has_failure:true`. `app.rs`: menu-open computes `has_failure = exit_status_kind(b.state,b.exit_code)==Failure && first_failure_ref(output_text).is_some()`; the dispatch extraction adds `..`; a `JumpToFailure` arm + a `#[cfg_attr(test, mutants::skip)]` `jump_to_block_failure` (block → `first_failure_ref` → `open_file_at`, reusing #212).
- **Compile-fix (test helpers):** the 4 existing `MenuKind::Block{pane,block}` constructions in the context_menu.rs test got `has_failure: false` (they test the 6-row menu — Phase 4 T2 adds the `true`/7-row case).
- `cargo check --all-targets` clean; `cargo fmt` clean; `cargo clippy` clean; `block_menu_kind_items_and_wrap` stays green.

## Inspect (Phase 3.5)
Inline adversarial trace (small mechanical diff — pure ref-pick + a conditional menu row + a shim dispatch). **No defects.**

| Angle | Verdict | Evidence |
|---|---|---|
| `first_failure_ref` picks the FIRST line-carrying ref | SAFE | `lines().find_map` (line order) → `scan_links().into_iter().find_map` (link order) → `File{line:Some(l)}`; a bare path (`line:None`) or URL hits `_ => None` and is skipped. A leading bare-path line returns None → moves to the next line. |
| `items_for` branch | SAFE | `Block{has_failure:true,..}`→7-row FAILED (JumpToFailure `[0]`); `Block{..}`→6-row; `Split`→3-row. Exhaustive. |
| `has_failure` gate | SAFE | `exit_status_kind(state,exit)==Failure && first_failure_ref(output).is_some()`. A Running block is neither Success nor Failure (`block_status` reads the code only when Finished) → false; a succeeding or no-ref block → false → the 6-row menu. |
| `MenuKind::Block` field consumers | SAFE | Construction (7455) + extraction (3543 `{..}`) + 4 test sites; `cargo check --all-targets` confirms none missed (a miss fails to compile). |
| JumpToFailure dispatch reuse | SAFE | `jump_to_block_failure` → `output_text()` → `first_failure_ref` → `open_file_at(resolve_under_root(root,path), Some(line), col)` — `open_file_at`'s `Option<usize>` line matches; reuses #212's proven open+caret. |
| Double-parse (menu-open + dispatch) | SAFE (not a defect) | `first_failure_ref` runs at menu-open (for `has_failure`) and again at dispatch — a finished block's output is immutable, so consistent; a click, not a hot loop. The dispatch `if let Some` handles the (impossible) None. |
| MSI | Phase-4-gated | `first_failure_ref` fn-replacement + match killed by T1 (multi-line→ref, bare-path→None); `items_for` failed-arm killed by T2 (7-row vs 6-row, first-row identity). |

No `failure-record` (no bug). No new prevention rule (reuses #212 + the #175 menu pattern).

## Phase 4 — Validate
- **Tests added:** links.rs `first_failure_ref_picks_the_first_line_carrying_ref` (T1 — a bare-path note line then the primary `src/main.rs:12:5` ref then a second ref → the first line-carrying wins; bare-path-only → None; empty → None; URL-only → None); context_menu.rs `failed_block_menu_leads_with_jump_to_failure` (T2 — `has_failure:true` → 7 rows, `[0]==JumpToFailure`; `false` → 6 rows starting CopyCommand, no JumpToFailure). The existing `block_menu_kind_items_and_wrap` updated to `has_failure:false` (unchanged 6-row behavior).
- **`cargo nextest run -p marley` (the 3 tests): PASS.**
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** first-try — 15/15 incl. coverage 100% + MSI 100% on `first_failure_ref` + `items_for`.
- **Live-drive: documented env-block.** A right-click menu + editor open — no Marley window running + synthetic input env-blocked all session. Carried by the mechanism: `first_failure_ref` + the `items_for` gating are MSI-100 unit-proven, and `jump_to_block_failure` reuses #212's `open_file_at` (this session's proven caret path). Re-verify with a failing `cargo`/`grep` → right-click → Jump to Failure when unlocked.
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Added (#213); app_shell.md extended with the #213 "Jump to Failure" block action (the run→fix loop).
- **Knowledge (forge):** AAR `ea72f711` submitted — completed, effectiveness 5. No failure-record (inline inspect clean). No new prevention rule (reused #212's parser/open + the #175 conditional-menu pattern). Lesson: a conditional menu row is cleanest as a bool on the menu-kind (`has_failure`) computed at open + a second const table + an `items_for` branch — no dynamic Vec, no dead/disabled rows.
- **Ticket** TICKET-213 → closed/ + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** the run→fix loop shipped — a failed block jumps to its primary error. cov/MSI 100, GATE GREEN [diff] first-try. LOCAL commit only.
