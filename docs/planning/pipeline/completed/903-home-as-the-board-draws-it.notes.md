# Home as the Home board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-903-home-as-the-board-draws-it.md
- **Pipeline spec:** 903-home-as-the-board-draws-it.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23 (the ticket's source).
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** F-claude-867 (open a project from a spawned task); #835's prompt is a
  single-line editor bound to `menu::Confirm`; Rusty's cards are their own entity, drawn as one
  child of Home. The brain had nothing.
- **Design:** `card_with` gives a card a title row with an optional link; Home's body is a centered
  column (max 860) and a wrapping row of cards (max 1080, each `flex_basis` 300 px, growing);
  `RustyCards::home_cards` returns its cards as elements, built in its own context, and Home
  observes it; the prompt is an auto-height editor two lines tall whose `editor::Newline` starts
  the agent.

## Phase 2 — Code
- `home_page.rs`: `render` and `cards`, `card_with`, `card_link`, `pill`, `cli_glyph`,
  `glyph_tile`, `state_tile`; the prompt, start pills, project chip, agents at work and configure
  as the board draws them. `rusty/home_tab.rs`: `home_cards`, the screens' tiles, the follow-up
  and task rows.
- A first edit script truncated `home_page.rs` (it wrote the file back while reading it); it was
  restored from the commit and the edits applied again from one script.
- Gate: `just gate-diff` 17 passed, 0 failed (after shellcheck's SC2016 on the stand-in, now a
  quoted heredoc).

## Phase 3 — Test
- Scenario `script/e2e/903-home-as-the-board-draws-it.sh` under sway, Rusty's stand-in on.
- `903-02-rusty` (REQ-001, REQ-002): the heading, the two-line box with Home's chip, CC Claude Code
  lit, Codex, Gemini CLI, OpenCode and the round Start; the five pills; Agents at work · Threads,
  Tasks · Open Tasks (lists as pills), Follow-ups due · 1 overdue with Follow Up and Recent
  pages, Rusty's eight tiles and captures, Recent projects · Open Folder…, Agent activity · Open;
  the settings links.
- `903-03-day`: the same in Marley Day.
- `903-01-night` (REQ-001): Rusty off: no Today's Note, three cards.
- `903-04-started`: a task typed in the box and Enter: the stand-in claude was asked "write the
  README". The first run had no stand-in on the terminal's PATH and started the box's real Claude
  Code (F-claude-903-…); fixed and run again.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (the board's layout), look-plan W2.2.
- Knowledge: F-claude-903-a-stand-in-not-on-the-terminals-path-001.
- Brain: decided on consultation e398c080a52449138157913be01321b4.
