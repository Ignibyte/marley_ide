---
pipeline_id: de85d227-a41b-4390-ae4b-25bc5790d7e9
ticket: forge#356 (1c248d03-43d4-4424-89f1-aec2074f5f6c) · local docs/planning/tickets/open/TICKET-356-unfocused-pane-stale-render.md
aar_id: 2bc1e904-1aa2-4449-bc65-c20afbea8531
status: Phase 5 — Complete PASS
title: An unfocused editable split pane re-syncs its read-only lines from the live Buffer
type: bug
milestone: M22
references:
  - crates/marley_app/src/code_view.rs
  - crates/marley_app/src/editor_surface.rs
  - crates/marley_app/src/app.rs
---

## Title
#259 (inspect F2): a FOCUSED editable split pane renders live from its Buffer, but an UNFOCUSED one renders the
read-only `CodeViewState.lines` — which is set once at file-open (`CodeViewState::new`) and NEVER re-synced from
the Buffer. So after editing (focused = live) and clicking away, the pane reverts to the pre-edit text until
re-focus ("my edits vanished"; no data loss — the Buffer is truth). Fix: keep each open file's read-only
`view.lines` in sync with its `buffer`, version-memoized, so the unfocused pane's existing (overlay-free)
read-only render shows the current text.

## Scope
### In
- `crates/marley_app/src/code_view.rs` (pure, cov/MSI 100): `CodeViewState` gains
  `lines_version: Option<BufferVersion>` + `sync_lines_from(&mut self, buffer: &Buffer, tab_width, max_cols)` —
  recomputes `lines` (via the existing tested `code_lines`) + records the version ONLY when
  `buffer.version()` differs (a no-op otherwise). The version guard keeps it O(1) except on an actual edit.
- `crates/marley_app/src/editor_surface.rs`: `EditorSurface::resync_active_view(&mut self, tab_width, max_cols)`
  → `self.files[active].view.sync_lines_from(&self.files[active].buffer, …)` (disjoint borrow of the `OpenFile`'s
  `view` + `buffer` fields).
- `crates/marley_app/src/app.rs` (shim, `mutants::skip`): `resync_editable_pane_views(&mut self)` iterates every
  editable pane (`shell.grids_mut()` → `states_mut()` → `editable_surface_mut()`) and calls `resync_active_view`;
  invoked ONCE at the top of `render(&mut self)`, before the element tree. The existing split-pane render
  (16606, the `None`/`cv.lines` arm) is UNCHANGED — it now reads the freshly-synced lines.

### Out (explicitly deferred)
- Rendering the unfocused pane from the Buffer via the editor-tab `Some(ed)` arm — REJECTED: that arm binds
  SEVEN active-editor overlays (fold projection · inlay hints · foldable headers · find matches · diagnostic
  rows · git marks · diagnostic spans) + the caret's bracket-match, all `self.<active-editor>`-derived, which
  would misrender on an unfocused pane showing a DIFFERENT file. Gating all seven in an unvalidated,
  coverage-excluded render arm is higher-risk than the staleness bug warrants. The read-only arm is overlay-free
  by construction — syncing its source is the surgical fix (D1).
- The read-only view's `CODE_MAX_COLS` (200) truncation — unchanged (consistent with every Marley read-only
  code view; the unfocused pane is a preview, re-focus to edit full width).

## Reference (§20)
N/A — Marley-specific. The editable split pane (#259, a focused code pane inside a terminal tab) is Marley's own
surface; no Warp/Zed analog. No copyleft source consulted.

### Prior art
1. **OUR own code (highest-yield):** `code_lines` (code_view.rs:499) is the ALREADY-TESTED read-only line
   builder — the sync reuses it verbatim; no new rendering. `OpenFile` (editor_surface.rs:32) already holds
   `view: CodeViewState` + `buffer: Buffer` as separate fields, so a disjoint-borrow sync is trivial.
   `shell.grids_mut()` (tabs.rs:445) + `editable_surface_mut()` (workspace.rs:408) already exist for the render
   pre-pass. `Buffer::version()` (editor/buffer.rs:91) gives the memo key.
2. **Behavior maps / published:** n/a.

## Locked-In Decisions
- **D1 — sync the read-only lines, render via the overlay-free arm** (not a Buffer render through the
  overlay-laden editor-tab arm). Lowest-risk correct fix: it cannot bleed the active editor's overlays onto an
  unfocused pane. The `CODE_MAX_COLS` truncation is Marley's standard read-only fidelity.
- **D2 — version-memoized.** `sync_lines_from` recomputes ONLY when `buffer.version()` changed, so a repaint of
  a static unfocused pane is O(1) — honoring the codebase's O(visible)-not-O(file) discipline (#336).
- **D3 — synced in a `&mut` render pre-pass**, before the element tree, over the stored `CodeViewState` (so the
  fix is observable in a headless test: after an edit + repaint, the pane's stored `view.lines` is current).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `sync_lines_from` is called and the buffer version differs from `lines_version`, it shall rebuild `lines` from `buffer.text()` (via `code_lines`) and record the new version. | code_view.rs unit |
| REQ-002 | WHEN `sync_lines_from` is called and the buffer version equals `lines_version`, it shall be a NO-OP (lines + version unchanged). | code_view.rs unit (set a sentinel `lines`, re-sync same version → sentinel intact) |
| REQ-003 | WHEN an editable pane's buffer is edited (focused) and the pane is then rendered unfocused, its read-only `view.lines` shall reflect the edited text. | headless drive: split → type → repaint → the pane's stored `view.lines` == the edited buffer text (was stale before the fix) |

## Testing boundary (honest)
`CodeViewState::sync_lines_from` is pure → cov/MSI 100 in code_view.rs (REQ-001/002). `resync_active_view` +
`resync_editable_pane_views` are thin `mutants::skip` shims (app.rs coverage-excluded). REQ-003 is a headless
drive over the STORED view.lines (observable because the pre-pass syncs the stored state, per D3) — the #307
"the drive is the proof" pattern, plus the pure `code_lines` reuse.

## Phase Plan
- **P2 Design** — the field + `sync_lines_from` + `resync_active_view` + `resync_editable_pane_views` + the
  render-top call; the test plan (REQ-001/002 units; REQ-003 headless).
- **P3 Implement** — code_view.rs, editor_surface.rs, app.rs; update `CodeViewState` literals for the new field.
- **P3.5 Inspect** — adversarial: the version guard correctness, the disjoint borrow, the pre-pass covers all
  editable panes, the focused pane unaffected, no double-render.
- **P4 Validate** — code_view.rs units cov/MSI 100; the headless drive; existing #259 test green; gate.
- **P5 Complete** — archive, AAR, close #356.
