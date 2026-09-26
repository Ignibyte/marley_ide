---
pipeline_id: 741a3b93-68a1-4e61-91b6-8fe3460db690
ticket: docs/planning/tickets/open/TICKET-529-sticky-command-header.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A long block's command stays in view"
type: feature
slice: prong 1 T1 (stage-one block rendering; Warp once-over item 5)
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/470-blocks-drawn-in-the-terminal.spec.md, docs/planning/pipeline/completed/473-block-navigation-keys.spec.md, docs/planning/pipeline/completed/484-autosuggestions.spec.md]
---

## Title
While the terminal is scrolled back into a block whose first row is above the view, that block's
command is pinned at the top of the terminal, and a click on it scrolls to the block's start:
Warp's sticky command header, drawn over Zed's rows as stage one draws everything else.

## Scope
### In
- **When it shows:** while the view is scrolled back (`display_offset` above 0), off the alternate
  screen, and the block covering the view's top row started above it. At the live screen it never
  shows, so it never covers output that is still being written.
- **What it shows:** one row over the terminal's top row: the block's command in the terminal's
  font, cut to the width, with the block's state at the right (running, a check, or `exit N`) and
  an arrow that says a click goes to the start.
- **A click** scrolls the view so the block's first row (its prompt row) is at the top, as the
  block keys do (#473); the header then goes, since the start is in view. A press on the header
  starts no selection and sends no mouse report, as #474's buttons keep their presses.
- **The setting** `marley.sticky_command_header`, on by default, a toggle in a Terminal section of
  the Marley page (#515).
- **A hook in `terminal_view`**: the element asks Marley's workbench for the header in `prepaint`,
  lays it out over the top row and paints it after the blocks, as it does the autosuggestion
  (#484).

### Out (explicitly deferred)
- Warp's per-pane minimize arrow and its per-pane toggle key.
- A header while a full-screen program runs (the alternate screen keeps no blocks, AD-claude-470).
- Stacking the headers of blocks inside other blocks (ssh sessions in stage two, T5).
- The block's other fields in the header (directory, branch, duration).

## Reference (§20)
- **Warp, sticky command header** (https://docs.warp.dev/terminal/blocks/sticky-command-header/;
  the Warp once-over, `docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 5): while you
  scroll through a block with a lot of output, its command shows "at the top of the active Window,
  Tab, or Pane"; "click on the Sticky Command Header to quickly jump to the top of the Block"; for
  a command whose output fills the screen the header "remains hidden until you scroll upward",
  so it does not cover the top of the display; on by default, with "Show sticky command header"
  in the settings. Marley keeps the header, the click, the rule that it shows only once you scroll
  back, and the setting; it leaves out the minimize arrow and the per-pane key. No Warp code.
- **Upstream Zed:** the editor's sticky scroll (`crates/editor/src/element/header.rs`,
  `sticky_headers` and `paint_sticky_headers`): the first lines of the scopes above the view are
  pinned over its top rows, and a left press on one scrolls it to the top and stops the press;
  `sticky_scroll.enabled` turns it on (off by default in Zed's `default.json`). The terminal's
  header follows the same shape for blocks.

### Prior art
- **Behavior maps and reports.** The once-over, item 5 ("Neither is in T1 to T7"); none of the
  seven Orca reports has a sticky header (the one "sticky" in them, report 03, is a CSS position).
- **Published material.** Warp's page above.
- **The code we already ship.**
  - `marley_terminal::visible_spans` gives each block in view its rows and `starts_in_view`, so the
    block to pin is the one covering row 0 that does not start in view; `block_scroll` and
    `blocks.rs`'s `scroll_to_block` (sync, `scroll_to_bottom`, `scroll_up_by`) are the jump.
  - The terminal element's `MarleyTerminalSuggestion` hook (#484): a Marley element asked for in
    `prepaint` and painted in a fixed place in the paint order.
  - #474's `marley_keep_from_terminal` and #470's `marley_pill`, the press guard and the state's
    look.
  - Zed's editor sticky scroll, above, for the click that scrolls and stops the press.
  - The Marley page (#515) and `MarleySettings` for the toggle.

## UI proof
UI-AFFECTING. `script/e2e/529-sticky-command-header.sh` (`compositor sway`, for the click). The
scenario's own bash; `seq 1 400`, then `echo after`. Steps and shots: the live screen after both
(`529-01-live`: no header); Shift+PageUp twice, into `seq`'s output (`529-02-pinned`: `seq 1 400`
at the top with its check); Shift+PageUp until the block's first row shows (`529-03-start-in-view`:
no header); Shift+PageDown back into the middle, and a click on the header (`529-04-jumped`: the
prompt row of `seq 1 400` at the top, no header, no selection); a running
`for i in $(seq 1 300); do echo $i; sleep 0.02; done` scrolled back (`529-05-running`: the header
with `running`); `less` on a long file and Shift+PageUp (`529-06-alternate`: no header); the setting off
in the profile copy, scrolled back into `seq`'s block (`529-07-off`); `marley: open settings`
(`529-08-setting`: the toggle in the Terminal section).

## Locked-In Decisions
- D1 — Only while scrolled back: at the live screen the header would cover the newest output, which
  Warp avoids too. Scrolled back, it covers one row of history.
- D2 — Drawn over the top row, never as a row of its own: a row of its own would resize the PTY as
  the view scrolls, and stage one keeps Zed's row model (AD-claude-470).
- D3 — The click scrolls to the block's first row, the one the block keys stop at (AD-claude-473),
  and selects nothing.
- D4 — The header is Marley's element in `marley_workbench`, through a hook the terminal element
  calls, as the autosuggestion is (#484); the setting stays Marley's.
- D5 — On by default, as Warp's; the command shown is what the block's frame reported, which a
  header only shows (PR-claude-474: showing a field needs no nonce).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the terminal is scrolled back and its top row lies inside a block whose first row is above the view, the system shall show that block's command and state pinned over the terminal's top row. | Shot `529-02-pinned` |
| REQ-002 | WHILE the terminal shows its live screen, the system shall show no header. | Shot `529-01-live` |
| REQ-003 | WHILE the block's first row is in view, the system shall show no header. | Shot `529-03-start-in-view` |
| REQ-004 | WHEN the user clicks the header, the system shall scroll the view so the block's first row is at the top, and shall start no selection. | Shot `529-04-jumped` |
| REQ-005 | WHILE the pinned block runs, its header shall say it runs. | Shot `529-05-running` |
| REQ-006 | WHILE the alternate screen shows, the system shall show no header. | Shot `529-06-alternate` |
| REQ-007 | WHERE `marley.sticky_command_header` is false, the system shall show no header. | Shot `529-07-off` |
| REQ-008 | WHEN the Marley page shows, its Terminal section shall offer the sticky command header toggle. | Shot `529-08-setting` |
| REQ-009 | The diff gate shall be green. | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design and the test plan in the notes.
- **P2 Code** — the touchpoints rows first; the hook in `terminal_view`; the pure choice of the
  block in `marley_terminal`; the header and its click in `marley_workbench`; the setting and its
  toggle; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the scenario, read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley/three-prong-plan.md` (T1), the Marley crates' notes under
  `docs/marley_architecture/` (§21), ledger capture (§19), close, archive, commit.
