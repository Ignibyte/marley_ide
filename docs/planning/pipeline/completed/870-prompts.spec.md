---
pipeline_id: 8f423384-da6d-41e3-addb-3d403bace29c
ticket: docs/planning/tickets/open/TICKET-870-prompts-as-the-dialogs-board.md
status: Phase 4 — Complete PASS
title: Prompts as the Dialogs board draws them
type: feature
slice: The Marley look, W3.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
In the Marley layout a prompt (gpui's `window.prompt`, which Linux draws with Zed's `ui_prompt`)
reads as the Dialogs board draws one: a rounded card with the message as its title, the detail
as its body, and its buttons in a row at the right, Cancel quiet at the left, the others plain,
and the main one at the right, primary or, for a destructive verb, danger.

## Scope
### In
- A Marley hook in `ui_prompt`'s renderer: Marley draws the card; Zed's renderer keeps the scrim,
  the focus and the keys (Enter, Escape, the arrows).
- Every `window.prompt`: Marley's close and quit guard, its 16 delete and remove questions,
  Zed's save, conflict and quit prompts.
- Zed's layout keeps Zed's prompts.

### Out (explicitly deferred)
- Zed's trust modal (`SecurityModal`, a `ModalView`) and the questions agents ask through
  notifications (settings and keymap changes, seats, `agent_start`, pairing): TICKET-871.
- The board's "when" line: it captions each dialog on the board, and a prompt shows only itself.
- The first-run screen on the same board (W1's onboarding, its own ticket when planned).

## Reference (§20)
Upstream Zed: `ui_prompt` (`zed_prompt_renderer`, `ZedPromptRenderer`: a scrim, a `w_80` card of
the message and detail as Markdown, and full-width buttons, the active one tinted) set with
`App::set_prompt_builder`; gpui's `PromptButton` (`Ok`, `Cancel`, `Other`) and `PromptLevel`.

### Prior art
- The code we ship: `ui_prompt.rs` (the renderer and its key handlers), gpui's prompt builder;
  #840's `answer_style` (three button weights) in `rail.rs`.
- Ledger: the Zed hook pattern of #834, #838, #839, #869.
- The brain (consultation 0fbecdb5): nothing on this seam.

## UI proof
Scenario `script/e2e/870-prompts.sh` (sway): project `b` runs #833's stand-in Claude Code, which
asks a permission and waits; a README edited and left unsaved.
- `870-01-save`: closing the edited README: "README.md contains unsaved edits…" as a card, its
  buttons Cancel (quiet), Don't Save (plain), Save (primary, at the right) (REQ-001, REQ-002).
- `870-02-close`: closing the waiting agent's terminal: the close guard's card, its detail the
  agent's line, Cancel (quiet), Show (plain), Close (danger) (REQ-001, REQ-002).
- `870-03-day`: Marley Day, the close guard's card (REQ-001).

## Locked-In Decisions
- D1 — Marley draws the card's content through a hook; Zed's renderer keeps the scrim, focus,
  keys and the answer, so no Zed function body moves into a Marley crate (§20).
- D2 — The buttons draw in reverse order (Zed lists the main one first; the board puts it at the
  right). A `Cancel` button, or one labelled Cancel, Dismiss or Not Now, is quiet; the first is
  danger when its label starts with Close, Quit, Delete, Remove, Discard, End, Clear or Overwrite,
  and primary otherwise; the rest are plain.
- D3 — The message and the detail draw as plain text (Zed's Markdown is dropped in the Marley
  look; no Marley prompt uses Markdown).
- D4 — The arrow keys' choice shows as an accent border on the chosen button when it is not the
  main one.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley shows a prompt in the Marley layout, it shall draw a rounded card with the message as its title and the detail as its body, in Marley Night and Marley Day. | `870-01-save`, `870-02-close`, `870-03-day` |
| REQ-002 | WHEN a prompt has buttons, Marley shall draw them in a row at the right, the cancel one quiet at the left and the main one at the right, danger for a destructive verb and primary otherwise. | `870-01-save`, `870-02-close` |

## Phase Plan
- **P1 Plan**, **P2 Code** (the hook in `ui_prompt`, `prompts.rs`), **P3 Test**, **P4 Complete**.
