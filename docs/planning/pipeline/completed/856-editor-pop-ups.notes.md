# The editor's pop-ups in the board's look — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-856-editor-pop-ups.md
- **Pipeline spec:** 856-editor-pop-ups.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row (#890 and #895 wait on Chad).
- **Classification / tier:** feature, S. A Zed settings default and one small Zed hunk.
- **Recall (§18.3):** a probe with rust-analyzer in a scratch crate shot the pop-ups as they are:
  already in the theme. Zed has settings for the kind letter and the detail's side. The agent
  entries and the rename note are new behaviour: split to #899. The brain had nothing.
- **Decisions:** D1 in the spec.

## Phase 2 — Code
- `default.json`: `completion_menu_item_kind: "symbol"`, `completion_detail_alignment: "right"`,
  their comments saying why.
- `code_context_menus.rs`: `MarleyCodeActionsLook`; with it set the popover leads with "Code
  actions" and `KeyBinding::for_action(&ToggleCodeActions::default())`.
- `search_looks.rs` sets the global with #891's three.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/856-editor-pop-ups.sh` (sway, rust-analyzer in a scratch crate).
- `856-01-completions`: after `rail.wid`, "f width : u32" and "m width_for (&self, bool) → u32",
  the letters in the field and method colours, the detail at the right, the chosen row in the
  coat's tint (REQ-001).
- `856-02-hover`: on `wid`, the error card in the error's tint: "no field wid on type Rail /
  available fields are: width, shrunk (rustc E0609)" (REQ-003).
- `856-03-actions`: on `Rail`, "Code actions" and Ctrl-. over the four actions, the first in the
  accent (REQ-002).
- `856-04-day`: completions in Marley Day (REQ-001).

## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, the ledger rows, look-plan W6.
- Brain decision recorded on consultation fcdc4bab.
