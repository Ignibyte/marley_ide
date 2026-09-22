---
pipeline_id: a0c363fe-88af-4055-b1f1-d59a0f4e4de6
ticket: forge#270 (8334665d-8f60-4a52-b9b2-97237087d43f) · local docs/planning/tickets/open/TICKET-270-caret-follow-scroll.md
aar_id: 8e65777f-8e89-4949-9ddd-6bf71963fab5
status: Phase 5 — Complete PASS
title: Editor caret-follow scroll — keep the caret visible on motion
type: bug
milestone: M18
references: []
---

## Title
After a caret/selection-moving editor action, scroll the editor's uniform_list so
the caret's row is visible — reusing #273's `scroll_editor_to_row`. Fixes the
"⌘↓/⌘D/Down-past-viewport looks like a no-op" limitation and unblocks #290.

## Scope
### In
- `app.rs`: a `follow_editor_caret(&mut self)` shim — `active_editor_mut().map(|s| line_col(caret).0)`
  then `scroll_editor_to_row(row)` (borrow-safe: the row is computed in the `map`, then the scroll runs).
- `app.rs`: call `view.follow_editor_caret()` after each editor caret-moving dispatch — the #257
  plain-editor key branch (Up/Down/Word/Home/End + arrows), the platform ⌘-motion branch (doc/line),
  the #272 ⌘D select-next. (A click lands where you clicked → already visible → not wired.)

### Out (explicitly deferred)
- A new pure `ensure_visible` helper — `scroll_editor_to_row` (#273) + `Buffer::line_col` already
  provide the row + the non-strict scroll; no new pure logic.
- Horizontal (column) follow — the uniform_list is vertical; long-line horizontal follow is a follow-up.
- Terminal / read-only pane scroll (`cv.scroll`) — this is the EDITABLE editor's `editor_scroll` handle.

## Reference (§20)
Zed (the editor) — the universal "the caret stays on screen" / ensure-visible
convention: any caret motion that would leave the viewport scrolls the minimum to
keep the caret visible. Marley matches via gpui's non-strict `scroll_to_item`
(centered when off-screen, no-op when visible). Clean-room §20: gpui's public
`UniformListScrollHandle`; no Zed source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — reuse `scroll_editor_to_row` (#273).** It is THE shared non-strict scroll
  (its doc pre-names #270). No hand-rolled offsets; no new pure fn.
- **D2 — follow UNCONDITIONALLY after a caret-moving action.** `scroll_to_item` is
  non-strict (no-op when the row is already visible), so calling it on every motion
  is correct and cheap — no "did it move / is it off-screen" pre-check needed.
- **D3 — the caret ROW is `Buffer::line_col(caret).0`** (0-based) — the uniform_list
  item index. Computed after the motion updates the caret, inside a borrow-releasing `map`.
- **D4 — wire the MOTION sites, not the click.** A click-to-place lands where the
  pointer was → already visible. The keyboard motions (which can leave the viewport)
  are the sites: the #257 key branch, the ⌘-motion branch, the #272 ⌘D.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a keyboard caret motion moves the caret to a row outside the viewport, the editor shall scroll so that row is visible. | Headless: seed a tall file, press Down past the viewport (or ⌘↓), assert `editor_scroll_y_for_test()` increased from 0. |
| REQ-002 | WHEN the ⌘D select-next-match wraps/moves the selection off-screen, the editor shall scroll to the new caret. | Headless: a tall file with a match near the end, ⌘D, assert the scroll followed. |
| REQ-003 | WHEN a motion keeps the caret within the viewport, the scroll shall not jump (non-strict). | Headless / mechanism: `scroll_to_item` is non-strict (a visible row → no offset change). |

## Phase Plan
- **P2 Design** — `follow_editor_caret` + the exact call sites; the headless test plan.
- **P3 Implement** — the shim + the call-site wiring.
- **P3.5 Inspect** — the borrow-safety, the site coverage (which motions), the non-strict no-op.
- **P4 Validate** — the #264/#273 headless lane (tall file → Down/⌘D → editor_scroll_y > 0); gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #270.
