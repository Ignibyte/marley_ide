# rail-remove-session-header — pipeline notes (forge #239, M14 sprint #27)

Pipeline: 6df7a9da-f321-4038-8bc4-28a77f0b2728 · AAR: 9d29dd82-6701-474a-88a1-61e1f8b59c52
Ticket: forge#239 (e5fde138-6d0f-4b55-91a5-60ac3d9f9d50). 2nd of the M14 round (#238 was a config-reset, no code).

## Phase 1 — Plan (discovery inline)

**chad issue #1 (live use, 2026-07-10):** the rail's top "MARLEY" caps header is redundant — the status bar +
tabs already identify the workspace. Remove it.

**Discovery (this session):**
- The rail render loops `for row in rail_rows(&self.shell, &self.collapsed_projects)` (app.rs:~4256), matching on
  `row.level`: `RailLevel::Workspace` (app.rs:4258-4266) renders `row.label.to_uppercase()` as a muted
  `type_scale(Role::Caption)` caps header — THAT is "MARLEY". Then `Project` (4268), `Tab` (4352), `Pane` (4490).
- `tabs::rail_rows` (tabs.rs:508) initializes `rows` with ONE `RailLevel::Workspace` RailRow (`label = ws.name`,
  lines 512-520), then pushes Project/Tab/Pane rows. So the Workspace row is always `rows[0]`, emitted once.
- `RailLevel` (tabs.rs:454) = { Workspace, Project, Tab, Pane }. `RailRow` (tabs.rs:469) carries level/label/
  active/project/tab/pane/collapsed.
- **Consumers of `RailLevel::Workspace`:** ONLY the render arm (app.rs:4258) + two tests (`rail_rows_one_project`
  asserts `rows[0].level == Workspace` at tabs.rs:1005; the others count/filter by level). No nav / click /
  collapse / scroll logic reads it — the click path uses `project`/`tab`/`pane` fields, and rail_scroll skips by
  row index (fewer rows is fine). So removing the Workspace row is behaviorally safe.

**Design surface (for Phase 2 — D2):** (a) PURE removal — `rail_rows` starts `rows` empty (drop lines 512-520),
remove the `RailLevel::Workspace` variant + the render arm, re-index the tests; cleanest (no dead output). (b)
SHIM skip — keep the pure row, the render arm `continue`s; smaller but leaves an unrendered pure row + a variant
constructed-only-to-be-skipped. **Lean (a).** Either way `RailLevel::Workspace` likely disappears; confirm at
design that the variant is unused after (a) so clippy stays clean.

**Driven plan (control granted):** relaunch (dark, post-#238) → the rail's first row is the "Marley · main"
project row; NO "MARLEY" caps header above it; tabs (Marley/Details/Agents/Forge/…) + collapse chevron unchanged.

**ENV:** control granted this session (driven captures on). Autonomous auto-approved (M14) → run through commit;
do not stop; do not push.

## Phase 2 — Design

**D2 LOCKED — (a) PURE removal.** Drop the Workspace row from `tabs::rail_rows` AND remove the now-unused
`RailLevel::Workspace` variant + its render arm. Confirmed clean: the ONLY references to the variant are the
render arm (app.rs:4258) and one test assertion (`rail_rows_one_project` at tabs.rs:1005) — the other three
rail_rows tests look up rows by LABEL / filter by `RailLevel::Pane`, so they're removal-safe. After removing the
variant the app.rs `match row.level` stays exhaustive over the remaining `{Project, Tab, Pane}` (no wildcard).

**Architecture / approach.** Purely the pure→shim rail seam (no PTY / IO / forge). `rail_rows` is the pure
projection of the Workspace tree into `RailRow`s; the shim render loop matches `row.level`. Removing the
top-of-tree session header = start `rows` empty + drop the variant + drop the render arm. Behaviorally safe (the
click path keys off `project`/`tab`/`pane` fields; `rail_scroll` skips by row-index — one fewer row is fine; the
#236 collapse + #169 scroll are untouched). §14: pure stays pure; no panics; nothing else moves.

**File manifest:**
- `crates/marley_app/src/tabs.rs` — (1) `rail_rows`: `let mut rows = vec![RailRow{Workspace…}]` →
  `let mut rows: Vec<RailRow> = Vec::new();` (drop lines 512-520); update the fn doc ("a Workspace header, then
  …" → "for each project a Project header …"). (2) `enum RailLevel`: remove the `Workspace` variant (+ its doc).
  (3) `RailRow.project` doc "(`0` for the workspace header)" → trim the workspace-header mention. (4) test
  `rail_rows_one_project`: len 4→3, drop the 3 Workspace asserts (1005-1007), re-index rows[1..3]→rows[0..2].
- `crates/marley_app/src/app.rs` — remove the `RailLevel::Workspace => { … }` render arm (4258-4266). The match
  remains exhaustive over Project/Tab/Pane.

**Note (no orphan):** `Workspace::name` is a `pub` field (also used by the titlebar / launcher) → removing its
rail_rows use raises no dead-code warning. Confirm at inspect nothing else matches `RailLevel::Workspace`.

**Regression Test Plan:**
| AC | Test (tabs.rs) | Proves |
|----|----------------|--------|
| REQ-001 | `rail_rows_one_project` (rewritten): `rows.len()==3`; `rows[0]==Project("proj", active)`; `rows[1]==Tab("a", tab0, !active)`; `rows[2]==Tab("b", tab1, active)` | output begins with the Project row — no Workspace/session row (structurally impossible once the variant is gone) |
| REQ-002 | `rail_rows_two_projects` + `rail_rows_collapsed_hides_children` + `rail_rows_nested_panes` (UNCHANGED — label / `RailLevel::Pane`-filter based) | Project / Tab / Pane rows + active flags + collapse + pane nesting all intact |
| REQ-003 | DRIVEN capture (post-#238 dark relaunch) | the rail renders no "MARLEY" caps header; the "Marley · main" project row is the top rail row; tabs + chevron unchanged |

**Mutation / coverage:** the change REMOVES code (the Workspace-row construction) → removes mutants, adds none;
the retained `rail_rows` logic (project/tab/pane emission, the `&&` active-tab logic, the collapse guard, the
`panes>1` guard) stays killed by two_projects / collapsed / nested_panes. Confirm at validate via
`cargo mutants --list -f crates/marley_app/src/tabs.rs` (expect the retained set only) → cov/MSI 100. The removed
app.rs render arm is shim (coverage-excluded).

**Risks:** low. Only load-bearing check = no OTHER `match`/construction of `RailLevel::Workspace` (discovery:
none beyond the render arm + the one test) — re-confirm at implement with a grep before deleting the variant.

**Phase 2 status: Design PASS — D2=(a) pure removal; manifest + test plan set.**

## Phase 3 — Implement

Applied the manifest exactly (6 edits, 2 files). Pre-edit grep RE-CONFIRMED only 3 `RailLevel::Workspace`
references (tabs.rs:513 construction, tabs.rs:1005 test, app.rs:4258 render arm) — nothing else constructs or
matches it.
- **tabs.rs:** `rail_rows` now starts `let mut rows: Vec<RailRow> = Vec::new();` (dropped the Workspace RailRow);
  fn doc updated (no "Workspace header"; cites #239). `enum RailLevel` → `{Project, Tab, Pane}` (Workspace variant
  + doc removed). `RailRow.project` doc trimmed (no "workspace header"). `rail_rows_one_project` re-indexed: len
  4→3, dropped the 3 Workspace asserts, rows shifted up one.
- **app.rs:** removed the `RailLevel::Workspace => {…}` render arm; the `match row.level` is now exhaustive over
  Project/Tab/Pane (no wildcard needed — confirmed by compile).
- **Deviations:** none. `ws.name` is no longer read by rail_rows but `Workspace::name` is a `pub` field of a
  re-exported type → no dead-code warning (confirmed: clippy -D warnings clean).
- **Build:** `cargo fmt` clean; `cargo check --all-targets -p marley` clean; `cargo clippy --all-targets -p
  marley -- -D warnings` clean (only the pre-existing `block v0.1.6` future-incompat cargo note). Tests run at
  Phase 4.

**Phase 3 status: Implement PASS — compiles + clippy clean.**

## Phase 3.5 — Inspect

1 general-purpose critic (completeness / regression / mutation / clean-room) + an exhaustive self-review that
independently ran each lens with commands. **No findings — the change is trivially subtractive.**

- **[CLEAN] Completeness** — `grep -rn "RailLevel::Workspace" crates/` returns ZERO; `enum RailLevel` is now
  `{Project, Tab, Pane}`; the app.rs `match row.level` is exhaustive over the three with no wildcard (compiles).
  All three original references (construction tabs.rs:513, test 1005, render arm app.rs:4258) removed.
- **[CLEAN] Behavior / off-by-one** — the shim render loop ITERATES `rail_row_list` (no `rows[0]`/`.first()` on
  the rail list; the `.first()` hits in app.rs are argv/cwds, unrelated). `rail_scroll`/`rail_skip` use `.len()`
  → one fewer row is fine. No click/collapse(#236)/nested-pane(#155) logic indexed the leading row. The 3
  UNCHANGED tests (`rail_rows_two_projects`, `_collapsed_hides_children`, `_nested_panes` — label & Pane-filter
  based) PASS with row 0 gone; the re-indexed `rail_rows_one_project` PASSES. (`cargo nextest run rail_rows` = 4/4.)
- **[CLEAN] Mutation / coverage** — `cargo mutants --list -f tabs.rs` → 11 rail_rows mutants, ALL on the retained
  project/tab/pane/active/collapse logic (511 body, 518/530 `==`/`&&`, 543 panes-`>1`, 549 focused-pane), each
  killed by the 4 rail_rows tests. The removal added NO new mutant (`vec![Default::default()]` is unviable — no
  `Default` on RailRow). cov/MSI 100 maintained (gate:5 confirms at validate).
- **[CLEAN] Separate-render sanity** — the "Workspace" DOCK PANEL TITLE (`dock_title(side)` →
  `caption_header`, app.rs:427) is a DIFFERENT render from the rail ROW; untouched. #239 removed only the
  "MARLEY" rail row (the uppercased `ws.name`), exactly chad's target.
- **[CLEAN] Clean-room §20** — pure removal; no copied Warp source.

The background critic RETURNED and FULLY CONCURS — no CRITICAL/HIGH/MEDIUM/LOW findings. It independently
verified: exhaustive 3-arm match (compiler-proven, no wildcard — a missing arm would be E0004); `rail_skip =
rail_scroll.min(rail_total.saturating_sub(1))` self-clamps so one fewer row can't blank/over-skip; every
click/collapse/pane handler captures `row.project`/`.tab`/`.pane` via closure (never by row position);
`dock_title` lives in layout.rs (untouched) so the "Workspace" panel title is a separate render; 9 rail_rows
mutants all killed, `Default::default()` unviable. Zero defects.

**Phase 3.5 status: Inspect PASS — no findings; all four lenses independently verified clean.**

## Phase 4 — Validate

**Tests (no new tests — implement re-indexed one; validate confirms):** `cargo nextest run -p marley rail_rows`
→ **4/4 PASS** (`rail_rows_one_project` re-indexed len 4→3; `_two_projects` / `_collapsed_hides_children` /
`_nested_panes` unchanged, all hold with row 0 gone). Covers REQ-001 (no Workspace row) + REQ-002 (project/tab/
pane rows intact).

**Driven capture (REQ-003, control granted):** rebuilt the bundle (with #239), quit the stale #237 build,
relaunched → `scratchpad/239-no-header.png`, READ it. CONFIRMED: the left rail shows NO "MARLEY" caps header —
the rail now goes search box → the "Marley · main" PROJECT row (with its ▾ collapse chevron + × close) directly;
the "Workspace" DOCK PANEL TITLE (top), the tabs (Marley/Details/Agents/Forge), and the collapse chevron are all
still present; still DARK (from #238). Exactly chad's ask — the redundant session label is gone.

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov 100% + MSI 100% (the
retained 9 rail_rows mutants killed by the 4 tests; the removed app.rs render arm is shim/coverage-excluded);
clippy/fmt/docs/visual all green. Staged: tabs.rs (−), app.rs (−), + the 3 pipeline docs.

**Phase 4 status: Validate PASS — 4/4 tests, driven capture confirms the header removed, gate green [diff].**
