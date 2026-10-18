# The Take Over bar can be closed — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-810-the-take-over-bar-can-be-closed.md
- **Pipeline spec:** 810-the-take-over-bar-can-be-closed.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: an outside client typed into his manager terminal, and the bar
  with Take Over and Hand Back "never leaves. It should be able to exit it."
- **Recall:** the terminal's bar shows while `last_write` names a program still running, or a run
  is going, or the user has control; Hand Back only turns it back to Take Over. The page's #767 bar
  shows while `last_agent` is set. Clippy's `struct_excessive_bools` (pedantic) refuses a fourth
  bool in either struct, so the state is an enum.

### Design
- `terminal_drive.rs`: `Control { Agents, Quiet, Taken }` in place of `taken_over`; a write or a
  run turns `Quiet` back to `Agents`; `toggle_control` goes `Taken` ↔ `Quiet`; `close_bar` turns
  `Agents` into `Quiet`; the footer draws nothing while `Quiet`, and the close button only while
  not `Taken`.
- `browser.rs`: `PageControl` the same way, in place of `taken_over`; `close_control_bar`.

## Phase 2 — Code
- **Built:** as the design. Clippy: `struct_excessive_bools` refused a fourth bool in `Drive` and in
  `PageState`; each pair became the three-state enum the design names (`Control`, `PageControl`).

## Phase 3 — Test
- **Scenario:** `script/e2e/810-the-take-over-bar-can-be-closed.sh` (`compositor sway`, the
  browser fixture's stand-in named "Outside client"). Run 1: the page's Ctrl-I clicked the terminal
  pane, since the Browser tab opened in a split on the right; PAGE_X moved to 1250. Run 2, every
  check passing and every shot read:
  - `810-01-bar`: under the REPL, "Outside client typed into python3: "print(6 * 7)" <enter>", Take
    Over and the close button.
  - `810-02-closed` (REQ-001): after the close button, no bar.
  - `810-03-back` (REQ-003): the next write's bar, "… "print('again')" <enter>".
  - `810-04-taken`: Ctrl-I: "You have control" and Hand Back (Ctrl-I's one-time toast covers the
    bar's right end; the close button is drawn only while agents hold control).
  - `810-05-handed-back` (REQ-002): Ctrl-I again: no bar.
  - `810-06-page-bar`: "An agent acted in this page", Take Over and the close button.
  - `810-07-page-closed` (REQ-001): no bar, the page's Start and Stop at its top.
  - `810-08-page-back` (REQ-003): the agent's click on Stop: the bar again.
  - `810-08b-page-taken`: Ctrl-I in the page: "You have control: agents' actions in this page are
    refused", Hand Back, no close button.
  - `810-09-page-handed-back` (REQ-002): no bar.

## Phase 4 — Complete
- **Gate:** `script/gates.sh --diff` green on the tested tree (every gate PASS, the receipt
  written).
- **Documented:** `CHANGELOG.md` (Fixed: "The Take Over bar can be closed"); `docs/marley/guide.md`
  (the bar's close button and Hand Back closing it); `docs/marley_architecture/marley_workbench.md`
  (`Control` and `PageControl`); `docs/marley/walkthrough.md` stops 5.11 and the Browser tab's
  chooser stop. No Zed path changed.
- **Knowledge:** F-claude-810-the-take-over-bar-had-no-way-out-001,
  PR-claude-810-a-bar-for-an-agents-action-has-a-way-out-001.
- **Brain:** no decision recorded: the fix follows the ticket.
