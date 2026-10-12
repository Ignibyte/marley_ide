---
pipeline_id: 4c7809a6-940f-4852-94d3-52e4b15b053c
ticket: docs/planning/tickets/open/TICKET-770-one-prompt-to-several-agents.md
status: Phase 4 — Complete PASS
title: One prompt to several agents, each in its own worktree
type: feature
slice: agents together, after the finish (#716's Out: one prompt to several agents), T3 Code item 9
references:
  - docs/planning/pipeline/queued/716-delegation-tools.spec.md
  - docs/planning/pipeline/completed/510-worktree-agents.spec.md
  - docs/t3code_architecture/02-threads-and-composer.md
---

## Title
New Agent in Worktree (#510) starts one agent CLI per prompt. Comparing Claude Code and Codex on
the same task means doing it twice and typing the prompt twice. This ticket puts a checkbox for
each other installed agent CLI in the prompt: one Enter makes a worktree and a branch for each
agent ticked and starts each with the same first prompt, each with its own row in the rail. Split
from #716 (Chad, 2026-10-11, "spec all"), after T3 Code's one prompt to several models.

## Scope
### In
- **The checkboxes.** Under the prompt, "Also start:" and one checkbox for each agent CLI the
  submenu listed other than the one chosen, all clear when the prompt opens. A click toggles
  one; the keys stay with the prompt's text, as with the setup line's box (#585).
- **The names.** With one agent, the branch stays `agent/<name>`. With more, each agent's
  worktree and branch take the same name and the agent's program: `agent/<name>-claude`,
  `agent/<name>-codex`, so the set reads as one. The line under the prompt names every branch:
  `agent/<name>-claude, agent/<name>-codex from main`.
- **The starts.** Enter makes the worktrees one after another, the chosen agent first, then the
  ticked ones in the submenu's order, each as #510 makes one: its folder, its branch from the same
  base, the base in the config, `.worktreeinclude`'s copy and the setup line as checked (#585),
  its port slot (#590), the project's permission mode for that agent (#532), and Claude Code's
  trust watch (#587). The one-at-a-time guard holds for the whole set; a create that fails says
  why in its toast and the next one still runs.
- **A name taken by one of the set** is replaced for the whole set's next try: a folder already at
  `<name>-codex` makes the set try another name, keeping the suffix.
- Docs: the guide's Worktree agents passage; the architecture note.

### Out (explicitly deferred)
- Several copies of one agent (two Claude Codes on one prompt).
- Threads (ACP agents) in worktrees: New Agent in Worktree starts agent CLIs only.
- Comparing the worktrees side by side: the rail's drift chips and Review (#560, #511) serve.

## Reference (§20)
N/A — Marley-specific: T3 Code sends one prompt to several models, one thread and one worktree
each (`docs/t3code_architecture/02-threads-and-composer.md` §2.6, item 4 of its findings), and
Orca documents fan-out as a manual recipe (`docs/orca_architecture/01-agents-and-sessions.md`
§2.11). Marley's version is #510's prompt with checkboxes. Upstream Zed's part is the worktree
service (`git_ui_core::worktree_service`), used as #510 uses it.

### Prior art
- **The code we ship:** `worktree_agents.rs` (`open_prompt`, `WorktreePrompt`, `start`, `create`,
  `free_folder`, `free_name`, the `creating` guard), the rail's `worktree_agent_entries` and
  `agents::installed_clis`, `ui::Checkbox` as the setup line uses it, Zed's
  `worktree_names::generate_worktree_name`.
- **Behavior maps:** T3 Code §2.6 and its finding 4 (the branch pair `agent/<name>-claude`,
  `agent/<name>-codex`; the trust question once per agent); Orca §2.11 (fan-out as a recipe).
- **Published material:** none needed; git worktrees as #510 uses them.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN New Agent in Worktree's prompt opens with more than one agent CLI installed, the system shall show a clear checkbox for each installed agent other than the one chosen. | Shot 770-01 |
| REQ-002 | WHILE one or more boxes are ticked, the line under the prompt shall name one branch per agent, `agent/<name>-<program>`, and the base. | Shot 770-02 |
| REQ-003 | WHEN the person presses Enter with boxes ticked, the system shall make one worktree and branch per agent from the same base and start each agent there with the same first prompt. | Shot 770-03, the launch log, `git worktree list` and the branches |
| REQ-004 | WHEN the set's worktrees exist, the rail shall show each as a row of its own with its agent's terminal. | Shot 770-03 |
| REQ-005 | WHEN no box is ticked, the system shall make one worktree on `agent/<name>`, as before. | Shot 770-04 and the branch list |

## UI proof
Scenario: `script/e2e/770-one-prompt-to-several-agents.sh` (`compositor sway`), after #510's: a
scratch repository, fakes for `claude` and `codex` first on the terminal's PATH that log their
arguments and folder.

| Shot | Shows |
|---|---|
| 770-01-boxes | The Claude Code prompt with "Also start:" and a clear Codex box (REQ-001) |
| 770-02-ticked | Codex ticked by a click; the line names `agent/<name>-claude, agent/<name>-codex from main` (REQ-002) |
| 770-03-two-rows | Two worktree rows, each with its agent's terminal (REQ-003, REQ-004) |
| 770-04-one | A later prompt with nothing ticked made one `agent/<name>` (REQ-005) |

## Decisions
- **D1 — One after another,** under the existing one-at-a-time guard, since Zed's worktree service
  and the seed's mark are per create, and a failed create rolls back with force.
- **D2 — Suffixed names for a set,** after T3 Code's finding, so `git branch` and the rail show
  the set together; a single agent keeps #510's name.
- **D3 — The boxes start clear,** so Enter does what it did before unless the person ticks one.
- **D4 — The agents offered are the submenu's,** the installed CLIs, each with its own permission
  mode for the project (T3 Code's open question 5, its default).
