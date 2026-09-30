---
pipeline_id: 45c258b1-e166-4af5-a824-dde2817085ed
ticket: docs/planning/tickets/open/TICKET-598-worktree-agent-under-agent-thread.md
status: Phase 4 — Complete PASS
title: "New Agent in Worktree sits under New Agent Thread in the rail's +"
type: feature
slice: workbench shell (the rail's +), after #510 and #527
references: [docs/planning/pipeline/completed/510-worktree-agents.spec.md, docs/planning/pipeline/completed/527-project-launch-configs.spec.md]
---

## Title
Move New Agent in Worktree up in a project's + menu, from after the Agent CLIs to directly below
New Agent Thread, so the two submenus that start an agent sit together and the Agent CLIs header
holds only the CLIs.

## Scope
### In
- The rail's project + menu (`render_project_menu` in `crates/marley_workbench/src/rail.rs`):
  the New Agent in Worktree submenu is added right after the New Agent Thread submenu, before the
  Agent CLIs separator and header.
- The doc comment on `worktree_agent_entries`, which says where the entry sits.
- The guide's and the walkthrough's wording where they place the entry.
- The e2e scenarios that reach the entry by position keep working: `510-worktree-agents.sh` and
  `585-worktree-environment.sh` (both in the golden set), `587-claude-code-trust-in-a-new-worktree.sh`
  and `589-remove-a-worktree.sh` press Down `WORKTREE_STEPS=7` times and click the submenu at fixed
  points; the count becomes 3 and the points are measured from the new menu.

### Out (explicitly deferred)
- Any change to what the submenu offers, when it shows (#510's rule: a local project whose folder
  is a git repository, with a CLI installed), or to the Launch entries (#527).
- A separator between the two agent submenus and the plain entries.

## Reference (§20)
N/A — Marley-specific: the + menu and both of its agent submenus are Marley's own (#440, #510),
with no Warp or Zed menu that orders the same entries. Zed's `ContextMenu` draws the entries in
the order they are added, and that is kept.

### Prior art
- **Behavior maps.** `docs/marley/workbench-shell.md` D4 (one visible way to start things) and
  the Orca worktree notes behind #510 (`docs/orca_architecture/02-worktrees-and-review.md`) say
  nothing about the entry's place in the menu.
- **Published material.** None applies.
- **Code we already ship.** `ui::ContextMenu` (`crates/ui/src/components/context_menu.rs`) adds
  entries in call order, so the move is the order of two calls in `render_project_menu`:
  `worktree_agent_entries` before `agent_cli_entries`. No other owner.

## UI proof
The scenario `script/e2e/598-worktree-agent-under-agent-thread.sh` (sway, since it clicks) opens
a scratch git repository with a fake agent CLI first on the PATH, clicks the project's + in the
rail, and shoots the open menu (`menu.png`).

## Locked-In Decisions
- D1 — The entry moves as it is: same label, same submenu, same condition. Only its position
  changes.
- D2 — When the condition fails (not a git repository, or no CLI installed), the menu is as it is
  today: New Agent Thread is followed by the Agent CLIs header, or by Launch.
- D3 — No Zed crate changes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens the + menu of a local project whose folder is a git repository and an agent CLI is installed, the menu shall list New Agent in Worktree directly below New Agent Thread. | Shot `menu.png` |
| REQ-002 | WHEN that menu is open, the Agent CLIs header and its CLI entries shall follow New Agent in Worktree, and the Launch header (when the project has launch configs) shall follow the CLIs. | Shot `menu.png` |
| REQ-003 | WHERE the project is not a git repository or no agent CLI is installed, the + menu shall show no New Agent in Worktree entry and its other entries in their current order. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes.
- **P2 Code** — swap the two calls in `render_project_menu`, fix `worktree_agent_entries`'s doc
  comment; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read `menu.png`.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the guide and the
  walkthrough (§21); the ledger; close, archive, commit.
