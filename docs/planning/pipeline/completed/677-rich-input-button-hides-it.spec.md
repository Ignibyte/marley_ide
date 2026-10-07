---
pipeline_id: 4b88e10f-21d8-4dcc-899d-f01d064aef7c
ticket: docs/planning/tickets/open/TICKET-677-rich-input-button-hides-it.md
status: Phase 4 — Complete PASS
title: "The Rich Input button hides it too"
type: feature
slice: the agent bar (#481)
references: [docs/planning/pipeline/completed/481-rich-input.spec.md]
---

## Title
The agent bar's Rich Input button toggles the agent's editor. Chad, 2026-10-07: "when i click rich
input here for the claude session only way to close it is esc. on warp is said Rich Input and Hide
Rich input."

## Scope
### In
- **The button**: pressed while the agent's editor is open, its tooltip Hide Rich Input (with
  Ctrl-G's binding as before); a click then closes the editor, keeps the draft and gives the
  terminal the focus. Closed, it reads Rich Input and opens it, as now.

### Out (explicitly deferred)
- **Ctrl-G inside the editor**: unchanged; Escape still closes it.
- **The shell's prompt editor** (#624, #627): it has no button.

## Reference (§20)
Warp — Chad's observation of Warp's agent input, whose control reads Rich Input and, while the
input shows, Hide Rich Input (2026-10-07). Marley's own Rich Input is #481's, built from
`docs/warp_architecture/`'s behaviour notes; no source read.

### Prior art
- **Behaviour maps:** #481's notes on Warp's rich input; Chad's observation above.
- **Published material:** none needed.
- **The code we ship:** `ui::IconButton::toggle_state`, `Tooltip::for_action_title`;
  `rich_input::{open, close}` and its `Prompts` global, which says whether a terminal's editor is
  open and for what.

## UI proof
`script/e2e/677-rich-input-button-hides-it.sh` (`compositor sway`): #481's stand-in Claude
Code (`exec -a claude`). Shots: `677-01-open` (the pencil clicked: the editor above the bar, the
button pressed, its tooltip Hide Rich Input); `677-02-hidden` (clicked again after typing a draft:
the editor gone, the button unpressed); `677-03-draft` (clicked again: the editor back with the
draft).

## Locked-In Decisions
- D1 — **The click reads the state when it happens**, not the state the bar last drew.
- D2 — **Hiding is Escape's close**: the draft stays and the terminal takes the focus.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the agent's editor is open, the Rich Input button shall show pressed, with the tooltip Hide Rich Input. | `677-01-open` |
| REQ-002 | WHEN the button is clicked while the editor is open, the system shall close the editor and keep its draft. | `677-02-hidden`, `677-03-draft` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rich_input.rs`, `agent_bar.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture note, close, archive, commit.
