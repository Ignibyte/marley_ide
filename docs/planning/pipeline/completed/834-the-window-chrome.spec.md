---
pipeline_id: 3df7d11a-5c58-4d61-9bce-e8ff1bd0730e
ticket: docs/planning/tickets/open/TICKET-834-the-window-chrome.md
status: Phase 4 — Complete PASS
title: The title bar's palette field and agent pills
type: feature
slice: The Marley look, W2.1 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The title bar becomes the window's place to start and to look. Its middle holds a field, "Search,
run a command, or ask an agent", with the palette's key, that opens the command palette. Its
right holds pills that count the agents at work ("2 agents working", amber) and those waiting on
you ("1 needs you", pink); a click on the pink one goes to the oldest request. The project's mark
(#831) and branch, and Rusty's and Marley's buttons, stay where they are. Today the middle is
empty and the window's agent state shows only in the rail.

## Scope
### In
- `MarleyTitleBarCenter`, a third `Global` hook in `title_bar.rs` (the shape of
  `MarleyTitleBarButtons`), whose view the title bar puts between its left and right groups,
  centered. Marley's view is the palette field: a 32 px tall rounded field, up to 520 px wide,
  with a magnifier, the muted words and the `command_palette::Toggle` binding; a click toggles
  the palette.
- In Marley's title buttons (`status_buttons.rs`, `TitleButtons`), before Rusty and Marley: a
  working pill (agents whose state is working) and a needs-you pill (waiting or failed), each shown
  only with a count, from the window's rail (`agents_at_work`); the needs-you pill opens the rail's
  oldest inbox entry, the working pill's tooltip names the agents.

### Out (explicitly deferred)
- Tabs as pills, the + menu's shortcuts, the split menu's pictures and the themed tmux close
  prompt: TICKET-866.
- Zed's layout: the field and pills show in the Marley layout only (in Zed's, the hook draws
  nothing).

## Reference (§20)
Upstream Zed (`title_bar`): the title bar's left and right groups and its hooks for Marley
(`MarleyTitleBarButtons`, #746; `MarleyProjectMark`, #831) stay; Marley adds a third hook of the
same shape for the middle. The command palette is Zed's (`command_palette::Toggle`). The look is
the design canvas's Main and Chrome boards (the field with Ctrl K, the "2 agents working" and "1
needs you" pills).

### Prior art
- The code we ship: `MarleyTitleBarButtons` and `MarleyProjectMark` (the hook shape, a view made
  per title bar), `TitleButtons` (`status_buttons.rs`), `Rail::agents_at_work` (Home's card) and
  `Rail::inbox`/`open_inbox_entry` (#833), the rail found from the window's `MultiWorkspace`
  sidebar (Home, #833's focus row), `KeyBinding::for_action`, `zed_actions::command_palette::Toggle`.
- Ledger: PR-claude-831 (a view a title bar hook makes reads no workspace in `new`), the layout
  flag (`MarleySettings::layout`). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/834-the-window-chrome.sh` (sway): project `b` with #542's stand-in Claude
Code waiting and project `a` with one working; `marley.tmux_hosts` `[]`.
- `834-01-night`: the title bar: the palette field in the middle with `Ctrl+Shift+P`, the pills "1
  agent working" and "1 needs you" at the right (REQ-001, REQ-002).
- `834-02-palette`: the field clicked: the command palette open (REQ-001).
- `834-03-opened`: Escape, then the needs-you pill clicked: b's waiting terminal in front
  (REQ-003).
- `834-04-day`: Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — A third hook for the middle, drawn only in the Marley layout; the left and right groups
  are unchanged.
- D2 — Counts come from the rail's own `agents_at_work`, so the pills, Home's card and the rail
  agree; waiting and failed both count as needs you.
- D3 — The field is a button that opens Zed's palette, not an editor of its own.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout shows, the title bar shall show a palette field in its middle that opens the command palette when clicked, in Marley Night and Marley Day. | `834-01-night`, `834-02-palette`, `834-04-day` |
| REQ-002 | WHILE agents work or wait on the user, the title bar shall show pills with their counts. | `834-01-night`, `834-04-day` |
| REQ-003 | WHEN the user clicks the needs-you pill, Marley shall open the oldest request waiting on the user. | `834-03-opened` |
| REQ-004 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the hook and its row; the field; the pills; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
