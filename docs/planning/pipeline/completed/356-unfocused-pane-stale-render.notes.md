# Unfocused editable pane renders from its Buffer (#356) — Notes

- **Forge ticket:** #356 (1c248d03-43d4-4424-89f1-aec2074f5f6c)
- **AAR:** 2bc1e904-1aa2-4449-bc65-c20afbea8531
- **Local ticket doc:** docs/planning/tickets/open/TICKET-356-unfocused-pane-stale-render.md
- **Pipeline spec:** 356-unfocused-pane-stale-render.spec.md

## Phase 1 — Plan
- **Request:** #356 (M15, ships M22) — an unfocused editable split pane renders stale `cv.lines` instead of its
  live Buffer. Fix: Buffer render for unfocused too; only focused owns carets + the geom/IME slot.
- **Discovery (the seam):**
  - `code_view_body(cv, colors, editor: Option<EditorDraw>, cx)` (app.rs:5042): `Some(ed)` → live Buffer render;
    `None` → the lossy `cv.lines` path (app.rs:5751; comment app.rs:652).
  - `EditorDraw { buffer, carets, cell }` (app.rs:653); `editor_draw_for(surface, window)` (app.rs:5001,
    `mutants::skip`) builds it from a surface. The geom/IME `canvas` is registered unconditionally in the
    `Some(ed)` arm at `if row == first` (app.rs:5424–5450) — writes the single `self.editor_geom` (app.rs:436)
    + `window.handle_input`.
  - The BUG site — split-pane caller (app.rs:16606–16619): `pane_draw = if focused { Some(editor_draw_for(...)) }
    else { None }` → unfocused = None = cv.lines.
  - The editor-TAB caller (app.rs:15513–15515): `active_editor().map(|s| editor_draw_for(s, window))` — the
    active editor IS the focused surface.
  - `editable_surface()` (workspace.rs:399) → `Some` for an editable pane, `None` for a #246 read-only file pane.
- **Decisions:** D1 Buffer render (not a cv.lines re-sync — cv.lines is lossy) · D2 D-FOCUS-OWNS-THE-SLOTS
  (focused-only carets + geom/IME) · D3 a pure `pane_body` decision the render consumes.
- **Prior-art (§20):** `editor_draw_for` is the SHARED Buffer-render builder (editor tab + focused pane, proven
  by #259) — the fix reuses it for the unfocused pane. No new render path. N/A reference (Marley-specific #259).

## Phase 2 — Design
> **SUPERSEDED — approach changed A → B during design.** The original `pane_body`/`EditorDraw.focused`
> (render the unfocused pane from the Buffer via the editor-tab arm) was abandoned once the audit found that
> arm binds SEVEN active-editor overlays (fold/inlay/foldables/find/diag-rows/git/diag-spans) + bracket-match,
> which would misrender on an unfocused pane showing a different file. The shipped design (in the REVISED spec)
> is the version-memoized `cv.lines` sync rendered via the overlay-free read-only arm. The A design below is
> kept for the record.

### (superseded A design, kept for the record)
- **`crates/marley_app/src/code_view.rs` (pure):**
  ```rust
  /// #356: how a split-pane code body draws. `Lines` = the #246 read-only file pane (no buffer). `Buffer` =
  /// an editable pane drawn from its LIVE buffer; `interactive` (only the focused pane) gates the carets + the
  /// single geom/IME slot. The fix: an UNFOCUSED editable pane is `Buffer { interactive: false }` (was `Lines`).
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum PaneBody { Lines, Buffer { interactive: bool } }
  pub fn pane_body(has_buffer: bool, focused: bool) -> PaneBody {
      if has_buffer { PaneBody::Buffer { interactive: focused } } else { PaneBody::Lines }
  }
  ```
- **`crates/marley_app/src/app.rs`:**
  - `EditorDraw` → add `focused: bool`.
  - `editor_draw_for(surface, window, focused: bool)` → carets computed ONLY `if focused` (empty otherwise);
    `EditorDraw { …, focused }`.
  - `code_view_body` editable arm → the `if row == first` geom/IME canvas becomes `if row == first && ed.focused`
    (carets already empty when unfocused via the empty `ed.carets`, so no caret gate needed there).
  - Split-pane caller (16606–16619):
    ```rust
    let focused = self.workspace().focused() == pane_id;
    let surface = self.workspace().state(pane_id).and_then(|s| s.editable_surface());
    let pane_draw = match crate::code_view::pane_body(surface.is_some(), focused) {
        crate::code_view::PaneBody::Buffer { interactive } =>
            surface.map(|s| self.editor_draw_for(s, window, interactive)),
        crate::code_view::PaneBody::Lines => None,
    };
    ```
  - Editor-tab caller (15515) → `editor_draw_for(s, window, true)`.
- **File manifest:** code_view.rs (+`PaneBody`/`pane_body` + tests); app.rs (EditorDraw field, editor_draw_for
  param + 2 callers, code_view_body geom gate, the split-pane caller).
- **Regression Test Plan:**
  | Test | Proves | AC |
  |---|---|---|
  | `pane_body_editable_focused_is_interactive_buffer` (code_view.rs) → `Buffer{interactive:true}` | REQ-001 |
  | `pane_body_editable_unfocused_is_noninteractive_buffer` (code_view.rs) → `Buffer{interactive:false}` (the fix) | REQ-002 |
  | `pane_body_no_buffer_is_lines_regardless_of_focus` (code_view.rs) → `Lines` for focused AND unfocused | REQ-003 |
  | existing `split_file_pane_is_editable_when_focused_headless` (#259) stays green | the focused arm + editor_draw_for(…, true) unbroken | REQ-004 (focused half) |
  - REQ-004 (unfocused render source): composition — `pane_body` (tested decision) ∘ the render matching it
    (inspect) ∘ `editor_draw_for` (proven). Not pixel-observable headlessly (documented boundary, §7).
- **Risks:** the single geom/IME writer (only focused) — the gate `&& ed.focused` is the invariant; the
  editor-tab focus value (`true` — active_editor is focus-aware); borrow soundness of the `surface` reuse.

## Phase 3 — Implement (approach B — version-memoized sync)
- **`code_view.rs`:** `CodeViewState` gains `lines_version: Option<BufferVersion>` (init `None` in `new`) +
  `sync_lines_from(&mut self, buffer, tab_width, max_cols)` — rebuilds `lines` via the tested `code_lines` +
  records the version ONLY when `buffer.version()` differs; a no-op otherwise. Imported `BufferVersion`.
- **`editor_surface.rs`:** `EditorSurface::resync_active_view(&mut self, tab_width, max_cols)` →
  `let file = &mut self.files[self.active]; file.view.sync_lines_from(&file.buffer, …)` (disjoint borrow).
- **`app.rs`:** `resync_editable_pane_views(&mut self)` (`mutants::skip`) iterates
  `self.shell.grids_mut()` → `states_mut()` → `editable_surface_mut()` and calls `resync_active_view(…, CODE_MAX_COLS)`;
  invoked ONCE at the top of `render(&mut self)` (after `let colors`). The split-pane `None`/`cv.lines` arm is
  UNCHANGED — it reads the now-synced lines.
- **Deviations:** approach changed A→B (see the Phase 2 superseded note) — lower-risk (overlay-free render),
  perf-correct (version guard). **No** `pane_body`/`EditorDraw.focused`.
- **Compile:** `cargo check -p marley` clean; `sync_lines_from` units (REQ-001/002) pass.
## Phase 3.5 — Inspect
One general-purpose critic (correctness + integrity). **Two real findings, BOTH confirmed + FIXED**; the 5
named lenses (max_cols consistency, version guard, coverage, borrow, focused/read-only regression) clean.

| # | Finding | Sev | Verdict → Fix |
|---|---|---|---|
| F1 | **Reload version-collision (correctness).** `reload_active` swaps in a fresh `Buffer` at version 0 (editor_surface.rs:437); an UNEDITED pane's memo was `Some(0)`, so `Some(0)==Some(0)` → `sync_lines_from` no-ops → the unfocused pane renders PRE-reload text (the "vanish" bug, reintroduced in the agent/disk-reload path — exactly the `(nonce,version)` trap #268/#273 documents). | MEDIUM | **FIXED** — `f.view.lines_version = None;` in `reload_active`, beside the #268 nonce re-mint, so the next sync rebuilds. |
| F2 | **Focused-pane O(file)/keystroke (perf).** The sync rebuilt EVERY editable pane incl. the FOCUSED one, which renders from its Buffer (never `cv.lines`) — so each keystroke bumped its version and this rebuilt+discarded O(file) `cv.lines`, defeating the guard + the O(visible) discipline. | MEDIUM | **FIXED** — skip the focused pane (`workspace().focused()`); it syncs once on the first unfocused frame, BEFORE the read-only render reads it. Added a `project_count()==0` guard (resync runs before the launcher early-return, and `workspace()` needs an active project). |

Clean lenses (critic-verified): max_cols is `CODE_MAX_COLS`(200) at every production `CodeViewState::new` (loader/restore paths) AND at the sync — no truncation change; `Buffer::version()` advances on every edit/undo/redo via `apply_raw` (buffer.rs:194); `grids_mut()` reaches exactly the split panes that use `cv.lines` (the editor TAB always renders faithfully); the disjoint view-mut/buffer-immut borrow is sound; the focused pane + non-CodeView panes are untouched. `cargo check` clean after both fixes. **failure-record filed for F1** (a version-only memo across a buffer-identity change).
## Phase 4 — Validate
- **Tests (4):** `sync_lines_from_rebuilds_when_version_differs` (REQ-001), `sync_lines_from_is_a_noop_at_the_same_version`
  (REQ-002), `t356_resync_active_view_rebuilds_the_active_view` (the surface delegator rebuilds from the live
  buffer), and `t275_reload_active_epoch_and_clamps` EXTENDED to assert the inspect-F1 `lines_version` reset.
- **Run:** 4 passed.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (7:37) — cov 100, MSI 100, visual/AX. Receipt
  written. The touched-line mutants (sync's `!=`→`==`, `resync_active_view`'s body, `reload_active`'s reset line)
  are all killed; the app.rs `resync_editable_pane_views` render pre-pass is `mutants::skip`.
- **REQ-003 (unfocused pane shows current text, end-to-end):** carried by composition — `sync_lines_from`
  (tested) ∘ `resync_active_view` (tested) ∘ `resync_editable_pane_views` (`mutants::skip`, EXERCISED by the
  #259 split-pane headless render) ∘ the UNCHANGED read-only render arm. The pre-pass syncs the STORED
  `view.lines` before the element tree, so the unfocused pane's existing render reads current lines. app.rs
  render is coverage-excluded (#307 pattern: the composition + the incidental drive are the proof).
- **Pre-existing:** none in scope.
## Phase 5 — Complete
- **Docs (§21):** `CHANGELOG.md` → a `### Fixed` entry (#356); `app_shell.md` → a #356 note on the #259
  editable-split-pane section.
- **AAR (forge):** `aar-submit` completed, effectiveness **4** (a real introduced-then-caught F1). `failure-record`
  `BF-claude-lines-memo-version-only-across-reload-001` + `prevention-rule`
  `PR-claude-version-only-memo-unsound-across-buffer-identity-change-001` (a version-only memo is unsound across
  a buffer-IDENTITY change — reset it wherever the Buffer is replaced, or key on `(nonce, version)`; the #268/#273 trap).
- **Lessons:** approach A (buffer-render through the editor-tab arm + gate SEVEN active-editor overlays) was
  analyzed + REJECTED for approach B (version-memoized `cv.lines` sync rendered via the overlay-free read-only
  arm) — lower-risk (no overlay bleed) + testable + perf-correct. Inspect earned its keep: F1 (reload version
  collision, correctness) + F2 (focused-pane O(file)/keystroke, perf) both caught + fixed.
- **Close:** forge #356 → done; ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.
