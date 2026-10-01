---
pipeline_id: dcd7d4d6-0180-4478-82f4-91b7eb9608f7
ticket: docs/planning/tickets/open/TICKET-630-block-density-and-two-line-headers.md
status: Phase 4 — Complete PASS
title: "Block headers with the folder and branch, on by default"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, third slice: a block's header shows the folder it ran in and the git branch beside its
command: on two lines where its prompt took two rows or more, after the command where it took one;
and the headers are on by default (plan T5). Gaps and a density setting go to #631.

## Scope
### In
- The header's folder (the block's `pwd`, home as `~`) and branch, the branch recorded when the
  block starts, from the project's repository holding the folder, so a later checkout does not
  rewrite an older header.
- Two rows or more: the folder and branch on the first row, muted, the command on the second; one
  row: the command, then the folder and branch, muted.
- `MarleyBlockHeader` passes the rows the header covers (a Zed-crate hook #628 added).
- `marley.block_headers` on by default (`default.json`, the docstring, `BlockHeaders`).

### Out (explicitly deferred)
- Gaps between blocks, a header taller than its prompt, and `marley.block_density` (#631,
  Deliberate: rows that are no grid line).
- A header's duration (blocks record no time yet, #470's Out); collapsing blocks.

## Reference (§20)
- **Warp (behavior):** a block's header carries its folder and branch above the command, and blocks
  stand apart (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6). The tree has no
  note that defines Warp's density; the sizes here are Marley's own, set in this spec.
- **Upstream Zed:** `terminal.line_height` (`comfortable` 1.618, `standard` 1.3); `scroll_top`.

### Prior art
- **Behavior maps:** the note above; the gpui-era #216 density notes (chrome and fonts, not
  blocks).
- **Published material:** none needed.
- **Code we already ship:** `block_below_cursor` and `scroll_top` (`terminal_view.rs`); #476's
  `bottom_shift`; `PromptInfo` (pwd, branch).
- **Re-verified at promotion (2026-10-01):** #628 shipped without a display-row map (a header
  takes its prompt's rows), so gaps and a header taller than its prompt need rows that are no
  grid line: a map through thirteen line-to-y and four pixel-to-row sites of Zed's terminal
  element, which the fork keeps small. That part is split into #631 (Deliberate). `PromptInfo`'s
  `pwd` is the folder a block ran in (the `precmd` frame's), and its `git_branch` is read from a
  `git=` field neither script sends, so the branch comes from Zed's git store: the agent bar's
  lookup (`agent_bar::contents`, `branch_for`), the innermost repository of the project holding
  the folder.

## UI proof
`script/e2e/630-block-density-and-two-line-headers.sh`: a git repository on branch `main`, the
setting left at its default.
- `two-rows.png`: bash with a two-line PS1: each header's first row `~/…/repo · main`, muted, the
  second the command, with its pill; the live prompt as bash draws it;
- `one-row.png`: a one-line PS1 in a second terminal: each header one row, the command then the
  folder and branch, muted;
- `checkout.png`: after `git checkout -b other`, the older headers still read `main`, the new one
  `other`.

## Locked-In Decisions
- D1 — The header uses its prompt's rows (#628's D1); with one row the folder and branch follow
  the command.
- D2 — The headers are on by default once this slice ships.
- D3 — The branch is the one the block's folder was on when the block started, from Zed's git
  store, recorded once per block.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the headers are on, a block's header shall show the folder it ran in and its branch, with its command and pill. | Shots `two-rows.png`, `one-row.png` |
| REQ-002 | WHEN the branch changes, the headers of earlier blocks shall keep the branch they ran on. | Shot `checkout.png` |
| REQ-003 | WHILE the headers are on, the live prompt shall be drawn as the shell drew it. | Shots `two-rows.png`, `one-row.png` |
| REQ-004 | WHEN Marley starts with no `block_headers` setting, the headers shall be on. | Shots (the scenario sets nothing) |

## Phase Plan
- **P1 Plan** — promote, re-verify against #628, split #631, the design.
- **P2 Code** — the branch record, the header's lines, the hook's rows, the default; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
