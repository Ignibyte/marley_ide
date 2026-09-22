# collapsible-rail — pipeline notes (forge #236, M13 sprint #26)

Pipeline: 4397528e-666d-4347-bd54-ddc5f415b5e8 · AAR: f0bf7067-1fdb-434e-93ee-59496d4c9ac3
Ticket: forge#236 (1c9c86a3). 9th of the /work 228-237 train. Deps #233 (done), #152/#155/#169 (the rail).

## Phase 1 — Plan / Phase 2 — Design (discovery inline)

**chad:** #8 collapse workspace tabs + sub-tabs; + the rail-highlight ask.

**Discovery:**
- `tabs::rail_rows<S>(ws) -> Vec<RailRow>` (tabs.rs:468) builds Workspace header → per-project: Project
  row + Tab rows + nested Pane rows (#155). `RailRow { level, label, active, project, tab, pane }`
  (tabs.rs:450); `active` already marks the active project (i == active_project_index, #233).
- Rail render: app.rs:4204 `rail_rows(&self.shell)` + a 2nd call at :4214 (the scroll `total`); the loop
  at :4221 matches `row.level`. The Project row (:4233) shows `name · branch` + a × close (:4252,
  stop_propagation) + click→switch_project (:4288). The ACTIVE state (:4280) is TODAY just bright text
  (`text_color(foreground)`) — the #219 comment reserved the "box" for leaf Tab/Pane rows. chad wants a
  clearer highlight → upgrade the active project row to a bg highlight (`rail_highlight`, app.rs:429).
- The #184 block fold is the chevron+toggle idiom to mirror (▸/▾ + stop_propagation).

**Design:**
- **tabs.rs:** `RailRow += collapsed: bool`; `rail_rows(ws, collapsed: &HashSet<usize>)` — Project row
  `collapsed: collapsed.contains(&i)`; when contained, SKIP the Tab/Pane loop. All non-Project rows
  `collapsed: false`. Update the existing rail_rows tests (pass `&HashSet::new()`) + add a collapse test.
- **app.rs:** `collapsed_projects: HashSet<usize>` on RootView (init empty); pass `&self.collapsed_projects`
  to both rail_rows calls; the Project row prepends a ▸/▾ chevron (`row.collapsed`) with a
  toggle-collapse click (a `toggle_project_collapse(p)` helper: insert/remove p); the active project row
  gets `.bg(rail_highlight(&colors, true))` (the prominent focused-workspace highlight).

**Driven plan (control):** boot (1 project, 4 tabs) → capture the active-project HIGHLIGHT (bg) → click
the project chevron → its tabs HIDE (capture) → click again → tabs return.

**ENV:** chad granted control → driven captures on; report at the end of the train.

## Phase 3 — Implement
tabs.rs: `RailRow += collapsed`; `rail_rows(ws, collapsed: &HashSet<usize>)` — Project row
`collapsed: contains(&i)` + `if is_collapsed { continue }` (skip children); all 5 rail_rows test call
sites updated (`&HashSet::new()`) + `rail_rows_collapsed_hides_children`. app.rs: `collapsed_projects:
HashSet<usize>` field + boot init; both rail_rows calls pass it; the Project row prepends a ▸/▾ chevron
(`row.collapsed`) with a stop_propagation toggle → `toggle_project_collapse(p)`; the ACTIVE project row
gets `.bg(rail_highlight(true))`. `cargo check` clean; 325 tests pass (+1).

## Phase 3.5 — Inspect
1 critic + self-review. **1 MEDIUM (real bug, FIXED) + 2 LOW (FIXED):**
- **[MED] collapse index-shift aliasing** — `collapsed_projects` (raw indices) wasn't remapped when
  `close_project` removes a project (Vec shift) → the collapse aliased the wrong project. FIXED: remap
  in `close_project_at` (drop idx, decrement >idx), mirroring the #177 renaming_tab guard. Captured
  `BF-claude-index-keyed-state-not-remapped-on-vec-remove` + `PR-claude-remap-index-keyed-state-on-collection-remove-001`.
- **[LOW]** my `toggle_project_collapse` insert split `close_project_at`'s doc block → reorganized (each
  fn its own intact doc). **[LOW]** the stale #219 "(no box)" comment now contradicted the #236 `.bg()` →
  reworded.
- **CLEAN:** rail_rows correctness (expanded emits children, collapsed suppresses only its own, active
  flag correct); MSI 100 (the collapse logic is method-call/bare-bool → no NEW mutants; the test gives
  coverage of the continue/fall-through; existing rail_rows mutants stay killed); no 0-workspace panic
  (the rail is after the launcher branch); clippy/clean-room clean.
**Phase 3.5 status: Inspect PASS.**

## Phase 4 — Validate
**Tests:** 326 pass — T1 `rail_rows_collapsed_hides_children`, + `remap_indices_after_remove_shifts` (the
inspect fix, extracted to a pure fn + tested → regression-protected). **Gate:** `scripts/gates.sh --diff`
→ **GATE GREEN [diff] 15/15**, cov + MSI 100. **DRIVEN capture (control granted):**
- `236-a-rail.png` — the active project row **"▾ Marley · main"** with a highlight BACKGROUND (the
  focused-workspace highlight) + the ▾ chevron → **REQ-002 + REQ-003 ✓**.
- `236-b-collapsed.png` — after clicking the chevron: **"▸ Marley · main"** (chevron flipped ▾→▸, still
  highlighted) + the "Marley" tab HIDDEN → **REQ-004 ✓** (+ REQ-002 toggle).
REQ-001 unit-tested. **Phase 4 status: Validate PASS.**
