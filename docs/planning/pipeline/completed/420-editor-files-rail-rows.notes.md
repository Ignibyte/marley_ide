# Open editor files as per-file rail rows — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-420-editor-files-rail-rows.md
- **Pipeline spec:** 420-editor-files-rail-rows.spec.md

## Phase 1 — Plan
- **Request:** chad's M31 simple-rail batch (docs/planning/design-notes/simple-rail-shelf.md),
  slice 3 of 418→419→420→421→422: open editor files return to the rail as per-file rows —
  label = disambiguated basename, click = focus that file in the editor surface, × = close that
  view, the frozen "Editor" tab row retires. The rail PROJECTION only; the #237 one-surface model
  and the in-editor strip stay.
- **Classification / tier:** feature, M31, UI-AFFECTING Zone A (rail). Core = a pure `rail_rows`
  change + a thin app.rs render/verb shim; no new state, no persistence delta. Queued behind #418
  (design source) with a soft ordering on #419 (the selection-flag render grammar).
- **Recall (§18.3):**
  - **#237** (completed/editor-surface-tabs.spec.md, `abbdd3e`): chad feedback #9 anti-clutter —
    N files → ONE surface, ONE rail row (the VS Code SURFACE model). The reversal keeps the
    surface; only the row projection flips — under #418's grammar the Editor section is the
    bounded open-files list, so the clutter concern the fold answered no longer applies.
  - **#240** (completed/editor-rail-label.spec.md, `b748fa8`): chad live #2 — the ONE row's label
    frozen to "Editor" (`EDITOR_TAB_TITLE`, tabs.rs:363). Its spec pre-authorized the flip ("a
    trivial future flip if he changes his mind"). The label problem dissolves WITH the row.
  - **F-#386** (knowledge/failures.md:400): plain `cargo check` is blind to `#[cfg(test)]` call
    sites (~14 broke at #386) — `cargo check --tests` is a gate at Implement AND Validate (D6).
  - **#397/#392/#395** carry the close semantics: `CloseOutcome::WouldDrain` on the last view →
    the caller drops the whole editor tab (editor_surface.rs:365-384; app.rs:8683-8691 routes it
    to `close_tab_at`), and a zero-tab project is legal + renders empty hints since #392/#395.
- **Discovery (the verified edit surface — every line re-read 2026-08-12 at promotion draft):**
  - tabs.rs: `rail_rows` :1032-1254 (Tab-row emission :1176-1223, the `label: tab.title` at
    :1212; CrossRef rows :1230-1250; `pane_mounts` :989-1024); `EDITOR_TAB_TITLE` :363;
    `open_or_switch_code` :557-570 (calls `surface.open_view`); `ContentLabels` :927-931
    (**terminal-only by design** — editor labels derive from surface paths, :921-925);
    `file_basename` :936; `TabContent::rail_section` :145-152.
  - editor_surface.rs: `EditorSurface` :313; `open_view` :353; `close` :368 + `CloseOutcome`
    :253-261; `activate` :387 (**the view-switch API**); `files()` :515; `view_ids()` :521;
    `active_index()` :526.
  - app.rs: `open_file_in_viewer` :5704-5785; `close_editor_file` :8678-8703 (**the view-close
    API** — WouldDrain → `close_tab_at`; today active-tab-addressed by caller contract
    :8684-8688 → the rail × needs the (p, t, i) generalization); the strip click driving
    `activate(i)` :19468-19479; the rail Tab arm :18639 (× → `close_tab_at` :18687) and CrossRef
    arm :18815 (the click body :18851-18867); the `(p, t)`-keyed disambiguation call :18511-18528;
    `TabLayout::Code` restore minting "Editor" :2380.
  - titlebar.rs: `disambiguate_labels` :61-77 (**the helper's real home** — the ticket's
    "app.rs:18511-18528" is its call site).
  - Tests (the D5 re-pin list): `rail_rows_groups_tabs_by_section_in_fixed_order` tabs.rs:2061,
    `rail_rows_preserves_original_tab_index` :2125, `rail_rows_collapses_a_non_active_section`
    :2620, `rail_rows_force_expands_the_active_section` :2649; `open_or_switch_code_cases` :1464
    stays green (title pins survive — tab identity, not rail label); green-unchanged:
    `rail_rows_empty_sections_still_render_headers` :2146, the #398 CrossRef suite :2887+.
  - marley-web (AHEAD of Rust here): App.tsx :70-71 `openFiles`/`activeFile`; LeftRail.tsx
    :270-283 per-file SubItem rows (active + onClose), :109-111 close re-picks active, :293-295
    the "no files open" empty caption. #418 settles the final look; the React step is reconcile.
  - Prior-art sweep: Zed map 07-workspace-panes-palette.md — no open-editors list panel to match
    (tab bar serves the role); Warp map — no analog; VS Code "Open Editors" = the published
    behavior grammar; gpui 0.2.2 elements (div/list/uniform_list) — nothing new to adopt, rail
    rows stay plain divs.
- **Decisions:** D1-D6 locked in the spec (projection walks the surface in order, pure;
  identity = ContentId, label = `file_basename` + `disambiguate_labels`; click =
  switch_project/switch_tab + `activate(i)` + #275 choke; × = `CloseOutcome` routing,
  (p, t, i)-addressed, last view closes the tab; the D5 re-pin list; `cargo check --tests`).
  Phase 2 settles D-FILE-ROW-LEVEL (new `RailLevel::File` recommended), D-FILE-XREF-COEXIST
  (home row + ⊞ row under #419's dots), D-FILE-EMPTY-SHAPE (bind to #418).
  **Two ticket corrections recorded:** the label source is NOT `ContentLabels` (terminal-only by
  design — the surface's own paths carry the labels), and `disambiguate_labels` lives in
  titlebar.rs:61, not app.rs.

## Phase 2 — Design
- …

## Phase 3 — Implement
- …

## Phase 3.5 — Inspect
- …

## Phase 4 — Validate
- …

## Phase 5 — Complete
- …

- **Promotion (2026-08-12, the `/work 418-422` batch):** #418 SHIPPED (2d244f6 + the marley-web
  twin) and #419 SHIPPED (9967ef0) — both ordering constraints hold; the "soft on #419" wish is
  satisfied (this lands under the single-selection + dot grammar). Seam shapes re-verified at
  promotion (all present, minor line drift): EDITOR_TAB_TITLE tabs.rs:363 ✓; open_or_switch_code
  :557 ✓; CloseOutcome editor_surface.rs:253 / open_view :353 / close :368 / activate :387 /
  files :515 / view_ids :521 / active_index :526 ✓; disambiguate_labels titlebar.rs:61 ✓;
  open_file_in_viewer app.rs:5723 (spec said :5704) ✓; close_editor_file app.rs:8697 (spec
  :8678) ✓. **Post-#419 re-mapping (the spec was authored pre-#419):** `rail_rows` now at
  tabs.rs:1077; the CodeView tab-row arm emits at :1287/:1306; CrossRef literal :1332;
  pane_mounts :1033. The spec's `active` vocabulary maps to the AS-BUILT `selected` +
  `dot: RailDot` model (#419's RailSelection coordinate is the selection spine the new File rows
  must join: REQ-006's "exactly one per-file row active" = the row's coordinate equals
  `RailSelection::Tab{p, editor-tab}`-with-view refinement — P2 settles the shape). D5 re-pin
  targets now: groups_tabs :2157, preserves_original_tab_index :2225, empty_sections :2246,
  collapses_non_active :2736, force_expands :2765, row_count_label_independent :2945,
  open_or_switch_code_cases :1559. BACKLOG row removed (§19).

## Phase 2 — Design

### §20 confirm + approach
Reference confirmed as speced (#418 design + VS Code Open Editors observed grammar; Marley-specific
projection; clean-room untouched). The projection joins #419's AS-BUILT selection spine — the
design binds to `RailSelection`/`RailDot`, not the spec's pre-#419 `active` vocabulary.

### The shape (D-FILE-ROW-LEVEL settled: a new variant, on the #419 spine)
- **`RailLevel::File`** — new variant (the CrossRef coordinate idiom: `tab = Some(editor-tab j)`,
  `pane = Some(view index v)`, `content = Some(cid)`). A new variant fans into exhaustive matches
  compile-visibly (D6's `cargo check --tests` catches every site).
- **`RailSelection::File { project, tab, view }`** — the derivation gains an arm AHEAD of the
  grid probe: active tab `.editor()` Some → `File { p, t, surface.active_index() }`; else the
  existing grid-multi-cell → `Cell`; else `Tab`. (A CodeView tab never has a grid, so the
  ordering is safe both ways — editor-first states the intent.) The CodeView tab-row emission
  DIES in the Editor section, so `Tab{p,t}` never dangles against a missing row.
- **File-row emission** (inside the Editor-section tab loop, replacing the CodeView Tab row):
  collect the surface's basenames (`file_basename` over `files()` paths) → `disambiguate_labels`
  (titlebar.rs:61 — one editor tab per project, so the surface IS the project's Editor row set) →
  zip with `view_ids()` → emit one File row per view in surface order:
  `selected = (selection == File{i, j, v})`; `dot = Ring` iff background-active
  (`i != active_project && j == active_tab && v == active_index()` — the rule inherited from the
  Tab row this projection replaces), never Filled (**D-FILE-XREF-COEXIST**: the pane-mount signal
  reads ONCE, on the CrossRef row — model A keeps both rows, home quiet + xref Filled).
- **D-FILE-EMPTY-SHAPE:** header-only this slice (Marley's uniform empty-section treatment).
  The POC's per-section empty captions ("no files open"/"no terminals"/"no panes") are a
  COHERENT ALL-SECTIONS follow-up — doing Editor alone would be inconsistent, and captions
  interact with the wheel-clamp row count (render-only lines the clamp doesn't count). Recorded
  as a parity delta + follow-up candidate; REQ-005 verified as header-present + the recorded
  deviation at the pair.

### app.rs manifest
- **The `RailLevel::File` render arm** — the CrossRef arm is the body template: label filter
  participation (`session_filter` by label), the #419 dot slot (`rail_dot_slot(row.dot)`),
  fill from `row.selected` only, × trailing; click = D3's shipped composition
  (`switch_project(p)` + `sync_active_project` if changed + `switch_tab(t)` +
  `EditorSurface::activate(v)` + `check_active_file_external()` (#275) + `persist_grid` (#163));
  × = the generalized close (below).
- **D4 close:** `close_editor_file(i)` (app.rs:8697, active-tab-contract, masked) generalizes to
  `close_editor_view_at(p, t, i)` — same `CloseOutcome` routing with the row's coordinates:
  `Removed(id)` → `release_editor_views([id])` + `check_active_file_external()`;
  `WouldDrain` → `close_tab_at(p, t)` (the #397 contract; a drained project is #395-legal);
  `NoOp` → nothing. The file-strip × becomes a delegating caller passing the active coords —
  ONE outcome-routing body, the #397 caller-contract comment updated (the fn no longer assumes
  the active tab).

### Test plan (per REQ; mutation-strong on the new branches)
| REQ | Proof |
|---|---|
| REQ-001 | re-pin groups_tabs (:2157): the `(Tab, "code", Some(0))` sequence arm becomes `(File, …)` and the Tab count drops; re-pin preserves_original_tab_index (:2225) onto the File row's `tab` coordinate; NEW: N-files-in-surface-order unit + "no Editor Tab row emitted" assert |
| REQ-002 | the click body is shim (masked) — the pure coordinates are pinned by the File-row emission units; the live drive exercises the click end-to-end |
| REQ-003 | `EditorSurface::close` outcomes are already unit-pinned (its own suite); the (p,t)-addressed routing is shim — the live drive closes a row × and the last-view drain; suites stay green |
| REQ-004 | NEW disambiguation unit: two same-basename files → distinct labels (order-of-appearance numbering) |
| REQ-005 | empty_sections (:2246) green-unchanged (header skeleton); the parity note records the caption deferral |
| REQ-006 | NEW: exactly-one-selected with a CodeView active tab (the File row of `active_index`, only while the editor tab is active); the #419 ≤1 sweep gains the CodeView fixture; collapse re-pins (:2736/:2765) re-key "codefile" → the file row's basename; background-active File Ring in the two-project fixture |
| REQ-007 | the React↔Marley pair at the same open-files state (the POC is already the as-built grammar — reconcile expected ZERO) |
D6: `cargo check --workspace --tests` at P3 exit + P4. Green-unchanged: row_count_label_independent
(:2945 — paths are model data), open_or_switch_code_cases (:1559 — tab identity survives), the
#398 CrossRef suite.

### Risks
- **R1** the RailLevel::File fan-out — compile-visible (exhaustive matches); D6 gates it.
- **R2** derivation ordering (editor vs grid) — safe (disjoint TabContent variants); stated intent.
- **R3** the empty-caption deferral — recorded delta; follow-up candidate covers all sections + the
  wheel-clamp interaction.
- **R4** the wheel-clamp count changes (N rows vs 1) — label-independence holds by construction;
  the clamp recounts the same fn.
- **R5** `disambiguate_labels` runs per render over the surface's rows — N is small (open files),
  matches the titlebar's own usage.

## Phase 3 — Implement

- **React-first:** POC verified live at :5173 (screenshot READ) — the as-built #418 rail already
  renders per-file rows (home rows + ×, the cross-ref Filled dot, selection) exactly as this
  ticket specifies. **Reconcile = ZERO React changes.**
- **tabs.rs:** `RailLevel::File` (the CrossRef coordinate idiom, doc'd); `RailSelection::File
  { project, tab, view }` + the editor-first derivation arm (disjoint from the grid probe — a
  CodeView tab never has a grid); the Editor-section tab loop now projects a CodeView tab as
  per-file rows (basenames → `disambiguate_labels` → zip `view_ids()`, surface order; selected =
  coordinate equality; dot = background-active Ring only — the pane-mount signal stays on
  CrossRef, model A) and `continue`s past the old Tab-row emission (the frozen "Editor" row is
  gone from the rail; `EDITOR_TAB_TITLE` and the tab identity stay).
- **app.rs:** the `RailLevel::File` render arm (compile-forced by the exhaustive match — the
  designed fan-out): label filter, #419 dot slot, selected-routed fill, × → the NEW
  `close_editor_view_at(p, t, i)` ((p,t)-addressed `CloseOutcome` routing: Removed → release +
  #275 choke; WouldDrain → `close_tab_at(p,t)`; NoOp → nothing), click = the #174 cross-project
  body + `EditorSurface::activate(v)` + #275 + persist; right-click add-to-pane (#398 — files are
  addable). `close_editor_file` is now a thin delegator passing the active coordinates (its
  file-strip call site untouched).
- **D6:** `cargo check --workspace` and `--workspace --tests` both green (the new variant's
  fan-out was compile-visible and is fully covered); fmt clean. The D5 re-pin targets now assert
  stale rail expectations at RUNTIME (groups_tabs expects a `(Tab, "code")` row) — the declared
  P3→P4 red window; P4 re-pins.
- **Deviations from design:** none.

## Phase 3.5 — Inspect

Two critics (model correctness / shim-interaction). Ledger:

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | `EditorMut::surface_mut` orphaned by the close rewrite → dead_code → gate:2 red | **REAL (high)** | `#[cfg(test)]` (headless-drive suites are its only remaining consumers) |
| 2 | 3 tests red on stale one-Editor-row expectations (groups_tabs :2241, preserves_index :2301, force_expands :2846 — mechanisms intact, labels stale) | **expected P3→P4 state** | P4 re-pins |
| 3 | `rail_rows_collapses_a_non_active_section` went **VACUOUS not red** — its hidden-child assert greps a label ("codefile") no row ever carries now; a red-list-driven re-pin would MISS it | **REAL (med) — the catch of the phase** | P4 checklist: re-pin to "no `RailLevel::File` row under the collapsed Editor" |
| 4 | The Removed × arm never persisted the shrunk open set (a closed file resurrected on relaunch) — pre-existing verbatim, but the rewrite hosts the persisting and non-persisting arms in ONE fn and #420 adds a second user-facing × | **REAL (med)** | `persist_grid()` after the #275 choke in the `other` arm (WouldDrain already persists via close_tab_at) |
| 5 | The delegator regressed totality (`active_project()` panics on the #395-legal empty workspace; latent — the strip's render gate makes it unreachable) | **REAL (low)** | total form (`projects().get(p)` + let-else return) |
| 6 | The strip's activate path lacked the #163 persist the new rail click carries (the two activate paths disagreed) | **REAL (low, pre-existing, adjacent)** | one-line `persist_grid()` in the strip handler — the rail click is the #174 canon |
| 7 | The content-match's `TabContent::CodeView` arm became unreachable (the pre-probe `continue`d first) — a stale comment + an unkillable-mutant surface for FULL audits | **REAL (low)** | the File emission FOLDED INTO the match as the CodeView arm (no pre-probe, no dead arm) |
| 8 | RailRow `tab`/`pane`/`content` field docs didn't cover the File variant (`pane` = VIEW index there) | **REAL (doc)** | all three extended |
| 9 | Disambiguation scope = the surface only; a pane-mounted CrossRef of a same-basename file can read ambiguously beside disambiguated File rows | **ACCEPTED scope** — CrossRef was never disambiguated; union-disambiguation recorded as a follow-up candidate |
| 10 | P4 coverage the red list doesn't force: the File-row selected unit, the 3 background-Ring conjuncts, a File-fixture for label-independence, surface-order/coordinate pins | **checklist** | P4 writes them |
Provenance: clean (in-house composition; gpui primitives + shipped helpers only). Verified clean
by trace: ≤1 selection over the new variant (disjoint derivation arms — `editor()` ⇔ CodeView,
`grid()` ⇔ Terminal; per-variant coordinate equality); the frozen "Editor" Tab row gone everywhere
(the sole Tab push sits behind the CodeView arm's `continue`); projection purity (the ContentLabels
param untouched by the File arm — label-independence holds by construction); the #386 force-expand
interplay (an ACTIVE editor tab makes Editor the active section → the selected File row can never
be collapse-hidden); release symmetry on every × path (Removed exactly-one; WouldDrain via
close_tab_at's whole-tab inventory; NoOp nothing); the cross-project click = the #174 canon incl.
the check→persist tail; × stop_propagation + per-frame closure re-mint; the filter idiom matches
CrossRef. Post-fix: `cargo check --workspace --tests` 0 errors, clippy 0 warnings, fmt clean.

## Phase 4 — Validate

**Tests written + RUN:**
- Re-pins: groups_tabs (the seq's `(Tab,"code",Some(0))` → `(File,"f.rs",Some(0))`; the partition
  becomes 3 Tab + 1 File); preserves_original_tab_index (the File row keeps the ORIGINAL editor-tab
  index + the view index); collapses_a_non_active_section (**the vacuous absence pin re-keyed
  STRUCTURALLY** — no `RailLevel::File` row under the collapsed Editor, per the new PR rule);
  force_expands (the "codefile" label grep → the File row "f.rs", still selected — #305 + #386
  proven on the new projection).
- New units: `rail_rows_editor_files_project_as_rows` (REQ-001/004/006: 3 views → 3 File rows in
  surface order; disambiguated same-basename labels "lib.rs"/"lib.rs 2"; NO Tab row at all;
  exactly the active view selected; coordinates (tab=editor-idx, pane=view-idx, content=cid);
  label-input-independence), `rail_rows_file_rows_background_active_ring` (the 3 Ring conjuncts
  killed on the File dot arm; non-active project never selects),
  `rail_rows_file_and_crossref_coexist` (model A: home File row quiet + CrossRef Filled, distinct
  coordinates, the split's cell holds the one selection).
- **Run:** `cargo nextest run --workspace` → **2134/2134 passed** (5 skipped); doctests clean;
  `cargo check --workspace --tests` 0 errors (D6). One of my own drafts fixed during the run (a
  literal `\n` token from the heredoc-escaping slip — the "cascade phantom" compile errors it
  caused in OTHER modules vanished with the one-byte fix).

**Live app driven (fresh bundle):** the restored #419 split state came back correctly (fill on
the focused cell); opening commit.md + spec.md from the Files tree → **the Editor section
projected THREE per-file basename rows (keymap.rs restored + the two new) with NO "Editor" row**,
spec.md (the active view) carrying the ONE fill and the Editor section force-expanded over its
stored collapse key; clicking the commit.md ROW moved the fill alone + switched the center buffer
(REQ-002 live); the spec.md × removed exactly that view from the rail AND the center strip,
selection undisturbed on commit.md (REQ-003's Removed path live). Captures read at every step
(/tmp/marley-420-*.png).

**Parity pair (REQ-007):** the POC driven to the same state — the commit.md File row clicked →
the DOM assertion returns exactly `["commit.md"]` as the one fill (byte-matching the live-app
state). **Honest gap:** the Playwright screenshot backend timed out (3 attempts) AFTER the
assertions ran — the React-side visual receipt is the #418 capture set (34–38, READ this session,
showing the identical per-file grammar: rows, ×s, selection, empty-slot alignment) + the DOM
sweep; the Marley-side captures are fresh. Behavior parity: 1:1 (click focuses, × closes, one
fill, basename labels; the POC's hover-revealed × vs Marley's persistent × is the recorded
#418-era treatment difference — the ×-visibility port is #418's DD-5 for a later slice, NOT a
#420 regression).

**Gate:** first run RED at gate:1 (rustfmt — the final count re-pin edit landed after the last
fmt; coverage + MSI PASSED on that run); `cargo fmt` + re-run → **GATE GREEN [diff], 15 passed,
0 failed** (coverage ≥100%, MSI ≥100 on the touched lines, miri, visual). Receipt written.

## Phase 5 — Complete
- **§21:** CHANGELOG entry; app_shell.md M31 #420 entry (above #419's). Parity sync:
  MARLEY-PARITY port-map row ticked LANDED with the ×-visibility gap recorded (POC
  hover-revealed vs Marley persistent — the #418 DD-5 treatment, a later polish slice).
- **§19:** F-claude-420-projection-change-turned-a-red-pin-vacuous-not-red-001 +
  PR-claude-absence-asserts-die-silently-under-projection-changes-001 (inspect);
  L-claude-420-reversing-a-projection-is-cheap-when-the-model-never-moved-001 (complete).
- **Ticket** → closed/; shelf row ticked; BACKLOG clean (left at promotion); pair → completed/.
