---
pipeline_id: 9e731810-d45e-4bb0-be1d-d64405098f8d
ticket: docs/planning/tickets/open/TICKET-842-forge-sprint-board.md
status: Phase 4 — Complete PASS
title: Forge's sprint board
type: feature
slice: The Marley look, W4.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
`marley: open forge board` opens a Forge tab: the current sprint's tickets in five columns (Open,
In progress, In review, QA, Done), each card its number, kind, phase, title and the agent on it;
a ticket's detail at the right with its description, status, agent and comments; and Claim,
Move to…, Comment and Start here in a worktree, written through Forge's tools.

## Scope
### In
- A Marley item (`forge_board.rs`): a header (Forge, the sprint's name, release branch and days
  left, "N of M done", a filter field), the columns, the cards, the detail pane.
- Reads: `sprint-current`, `ticket-list` for the sprint (`status: all`), `agent-list` (a card's
  agent is the running session whose `current_ticket` names its number), `ticket-get` with
  comments for the selected ticket. Read when the tab opens, after each action, and every 30 s
  while it shows.
- Writes: Claim (`ticket-claim`), Move to… (a menu of the statuses, `ticket-update {status}`),
  Comment (a field, Enter sends `ticket-comment`).
- Start here in a worktree: New Agent in Worktree (#510) with the ticket's number, title and
  description in its prompt.
- The connection: the first `forge` fleet provider's URL and token (#841's keyring, else
  `bearer_env`); without one the tab says how to add it.

### Out (explicitly deferred)
- The board's By agent and List views, New Ticket, the project picker (the token's project, or the
  provider's first `projects` entry), and the bottom row's Forge segment (#843).

## Reference (§20)
N/A — Marley-specific: Ignibyte Forge's ticket tools (`/srv/stacks/ignibyte-forge`,
`crates/forge-mcp/src/tools/tickets/`: `ticket-list`, `ticket-get`, `ticket-claim`,
`ticket-update`, `ticket-comment`; `tools/sprints/sprint_current.rs`), the ForgeBoard board.

### Prior art
- The code we ship: #841's Forge client and token (`forge.rs`, `fleet_providers::connect`); a
  Marley item with columns (`threads_page.rs`, #836); #510's `worktree_agents::open_prompt`;
  `ContextMenu` for Move to….
- The brain (consultation 3885085e): nothing on this seam.

## UI proof
Scenario `script/e2e/842-forge-board.sh` (sway): #841's stand-in Forge grown a sprint, nine
tickets across the statuses, two comments, and `ticket-claim`, `ticket-update` and
`ticket-comment` that change its state and log.
- `842-01-board`: the board: the header with the sprint and "N of M done", five columns with
  their counts, the cards (REQ-001).
- `842-02-detail`: #828 clicked: its detail with description, status, agent and comments
  (REQ-002).
- `842-03-moved`: Move to… → QA pass: the card in QA under its new status, the stand-in's log
  showing the `ticket-update` (REQ-003).
- `842-04-comment`: a comment typed and sent: it appears under Comments (REQ-003).
- `842-05-day`: Marley Day, the board (REQ-001).

## Locked-In Decisions
- D1 — Columns: Open = `open`; In progress = `in-progress`; In review = `in-review`; QA =
  `qa-review`, `qa-failure`, `qa-pass` (each card says which); Done = `done`, `closed`.
  `won-t-fix` and `abandoned` are left out.
- D2 — One Forge connection per tab, made from the first `forge` fleet provider, so the board and
  the fleet share their settings and token.
- D3 — An action's failure is a toast with Forge's reason; the board re-reads after every action.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens the Forge board, Marley shall show the current sprint's tickets in Open, In progress, In review, QA and Done, each card with its number, kind, phase, title and agent, under a header with the sprint and its progress, in Marley Night and Marley Day. | `842-01-board`, `842-05-day` |
| REQ-002 | WHEN the user clicks a card, the board shall show the ticket's detail: title, description, status, agent and comments. | `842-02-detail` |
| REQ-003 | WHEN the user claims, moves or comments on a ticket, Marley shall write it through Forge's tool and show the result. | `842-03-moved`, `842-04-comment`, the stand-in's log |

## Phase Plan
- **P1 Plan**, **P2 Code** (`forge_board.rs`, `forge.rs`'s shared client, #510's seed), **P3
  Test**, **P4 Complete**.
