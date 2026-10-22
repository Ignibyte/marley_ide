---
pipeline_id: a5457100-9b05-4a40-9267-f1f33c9f82a7
ticket: docs/planning/tickets/closed/TICKET-856-editor-pop-ups.md
status: Phase 4 — Complete PASS
title: The editor's pop-ups in the board's look
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
Completions show each item's kind as a coloured letter and its detail at the right; code
actions open under a "Code actions" heading with their key; hover and rename keep Zed's look in
the Marley theme, which already matches the board.

## Scope
### In
- `assets/settings/default.json` (Zed file): `completion_menu_item_kind: "symbol"` and
  `completion_detail_alignment: "right"`, Zed's own settings for the board's completion rows.
- `crates/editor/src/code_context_menus.rs` (Zed crate): `MarleyCodeActionsLook`, a global; with
  it set, the menu leads with the heading and the toggle's key.
- `search_looks.rs`: sets the global with the other looks (#891).

### Out (explicitly deferred)
- "Ask an agent to fix this…", the inline rename note, and inline assist with a selection sent
  to an agent: TICKET-899, new behaviour.

## Reference (§20)
Upstream Zed for the editor's menus: `editor::code_context_menus`, and its settings
`completion_menu_item_kind` and `completion_detail_alignment`.

### Prior art
- A probe run with rust-analyzer: completions, hover, code actions and rename all draw in the
  Marley theme already, the chosen row in the coat's tint; the board adds the kind badge, the
  right-hand detail and the code actions' heading.
- `render_completion_kind_letter` colours the letter from the syntax theme.
- The brain (consultation fcdc4bab): nothing on this seam.

## UI proof
Scenario `script/e2e/856-editor-pop-ups.sh` (sway): a scratch crate rust-analyzer reads.
- `856-01-completions`: `rail.wid` and Ctrl+Space: kind letters, the detail at the right
  (REQ-001).
- `856-02-hover`: hover on an error: the type and the diagnostic (REQ-003).
- `856-03-actions`: Ctrl+. on the struct: the heading and the actions (REQ-002).
- `856-04-day`: Marley Day, completions (REQ-001).

## Locked-In Decisions
- D1 — The completion rows use Zed's own settings, set in Marley's defaults; a user can turn
  them back.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The completions menu shall show each item's kind as a letter in its syntax colour and its detail at the right, in Marley Night and Marley Day. | `856-01-completions`, `856-04-day` |
| REQ-002 | The code actions menu shall lead with "Code actions" and the key that opens it. | `856-03-actions` |
| REQ-003 | Hover on a diagnostic shall show the type and the diagnostic in the theme. | `856-02-hover` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
