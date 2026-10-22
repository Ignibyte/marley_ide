---
pipeline_id: 5855c9ce-2605-40ce-8aa8-021354aa015f
ticket: docs/planning/tickets/open/TICKET-851-worktrees-and-review.md
status: Phase 4 — Complete PASS
title: New Agent in Worktree as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The worktree agent's prompt reads as the Worktrees board's card: the agent's badge and "Claude
Code in a new worktree", a "First prompt" box, each branch as a chip "from" its base, Also start,
the facts a new worktree gets (what `.worktreeinclude` copies, the permissions, the setup), and a
footer with the keys and a primary Start button.

## Scope
### In
- `worktree_agents.rs` `WorktreePrompt`: the header, the label, the chips, the facts, the
  footer; `.worktreeinclude`'s lines read when the prompt opens.

### Out (explicitly deferred)
- The port slot fact: a worktree's slot is given when it is made (`assign_slot`), so the prompt
  has none to show yet.
- Plan review, turns and the diff review (Review, DiffReview boards): TICKET-881.
- The rail's worktree rows and drift chips (drawn since #511), and merge and remove, which ask
  through `window.prompt` in #870's look.

## Reference (§20)
N/A — Marley-specific: Marley's worktree agents (#510, #585, #770); the Worktrees board.

### Prior art
- `WorktreePrompt` (#510), `setup_offer` (#585), `agents::launch_mode` (#532),
  `worktree_include::INCLUDE_FILE`; the keymap's Enter, Shift-Enter and Escape.
- The brain (consultation 6b1372ec).

## UI proof
Scenario `script/e2e/851-worktree-prompt.sh` (sway): a scratch repository with a commit, a
`.worktreeinclude`, bypass for Claude Code in the project, a fake `claude` and `codex`.
- `851-01-prompt`: the card with its badge, title, First prompt, the branch chip, Also start,
  the facts and the footer (REQ-001).
- `851-02-two`: Codex ticked: two branch chips, "Start 2 agents" (REQ-001).
- `851-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The facts show what Marley knows before the worktree exists: the include file's lines
  (or "Nothing: no .worktreeinclude"), the chosen agent's mode in this project, and the setup
  (the `create_worktree` task, the offered command with its box, or none).
- D2 — The primary button runs the same confirm as Enter.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The worktree agent's prompt shall show the agent's badge and title, a First prompt box, a chip for each branch from its base, Also start, the facts the worktree gets, and a footer with the keys and a Start button naming how many agents start, in Marley Night and Marley Day. | `851-01-prompt`, `851-02-two`, `851-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
