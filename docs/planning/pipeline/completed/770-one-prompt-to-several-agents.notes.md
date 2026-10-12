# One prompt to several agents, each in its own worktree — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-770-one-prompt-to-several-agents.md
- **Pipeline spec:** 770-one-prompt-to-several-agents.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** the goal "continue until all tickets are finished"; #770 is next on the after-the-
  finish shelf (#717, #769 done).
- **Classification / tier:** feature, Marley-only, small to medium; no Zed path.
- **Recall (§18.3):**
  - Failures on the seam: F-claude-587-the-trust-watch-read-an-answered-question-from-the-scrollback-001
    and F-claude-587-the-answers-toast-shared-the-questions-notification-id-001 (each notice its own
    id); F-claude-585-the-setup-offer-read-the-wrong-worktrees-tasks-001. Each create keeps its own
    trust watch, so a set of two Claude Codes is out of scope anyway.
  - #510's archive: the one-at-a-time guard, `free_folder`'s three names (Zed's rollback removes
    the target folder with force), the seed's mark per create.
  - The brain (consultation d982cf6fd8ce4b308174de96c279d5ac): nothing on this seam.
- **Discovery** (read directly; one file and its one caller):
  - `worktree_agents.rs`: `open_prompt(workspace, kind, window, cx)` (230-258), called only by the
    rail's `worktree_agent_entries` (`rail.rs:5741-5768`), which has `agents::installed_clis`;
    `WorktreePrompt` (339-464) with the setup `Checkbox`; `start` (468-505) holds `creating` around
    one `create` (508-607); `free_folder` (676-702) and `free_name` (201-226).
  - The rail lists every linked worktree as a row on its own, so a set needs nothing there.
- **Decisions:** D1-D4 in the spec.

### Design
- **`worktree_agents.rs` (Marley crate):**
  - `free_folder` becomes `free_folders(plan, suffixes, …)`: the first base name (of three) whose
    folder is free for every suffix, `-<program>` per agent for a set and none for one agent, with
    each name and folder; `create` takes its name and folder from it.
  - `free_name` also counts a taken name with an agent program's suffix stripped, so a later set
    never reuses a base an earlier set took.
  - `open_prompt(workspace, kind, others: Vec<AgentKind>, …)`; `WorktreePrompt.others:
    Vec<(AgentKind, bool)>`, drawn as "Also start:" and a `Checkbox` per agent; the branch line
    names each branch when one is ticked.
  - `confirm` collects the agents (chosen first, then ticked) and `start` takes the list: one
    `creating` insert, `free_folders` for the set, then `create` for each agent in turn. `Launch`
    derives `Clone`.
- **`rail.rs` (Marley crate):** passes the installed agents other than `kind`.
- **Docs:** guide (Worktree agents), `marley_workbench.md` (Worktree agents), CHANGELOG.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001 | `+` → New Agent in Worktree → Claude Code, with fakes `claude` and `codex` installed | 770-01: "Also start:", Codex clear |
| 002 | A click on the Codex box | 770-02: the line names both branches |
| 003, 004 | Types a prompt, Enter | 770-03: two worktree rows with terminals; the launch log has both agents in `-claude` and `-codex` folders with the same prompt; two `agent/*-claude`/`-codex` branches from main |
| 005 | A second prompt, nothing ticked | 770-04: a third row; one more branch, `agent/<name>` with no suffix |

### Risks
- Two Claude Code trust questions at once cannot happen: one Claude per set.
- A long set holds the guard until the last agent starts; a second prompt meanwhile gets #510's
  "being made" toast.

## Phase 2 — Code

- **Built:** in `worktree_agents.rs`, `Plan::branches` and `suffixes`; `free_name` counts a set's
  base name as taken; `open_prompt` and `WorktreePrompt` take the other installed agents and draw
  "Also start:" with a `Checkbox` each; `kinds()`; `start` takes the agents, finds the set's
  folders with `free_folders` (which replaces `free_folder`) and awaits `create` for each, which
  now takes its name and folder; `Launch` is `Clone`. In `rail.rs`, the submenu passes the others.
- **Deviations:** the boxes toggle by click only: the keys stay with the prompt's text, as the
  setup line's box does (the spec says so now). The set's names are found once for the whole set
  (`free_folders`) instead of per create, so a taken folder changes the set's name, not one
  agent's.
- **The review found** nothing to fix: the guard is inserted once and removed after the last
  create; a failed create returns and the loop goes on; each Claude create keeps its own trust
  watch.
- **Clippy:** clean on the first run.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the tree the scenario ran.

## Phase 3 — Test

- **Scenario:** `script/e2e/770-one-prompt-to-several-agents.sh` under `compositor sway`: a scratch
  repository on `main`, fakes `claude` and `codex` first on the terminal's PATH logging their
  arguments and folder. The first runs stopped at 770-00 and 770-01 to place the submenu's Claude
  Code (325,260) and the Codex box (618,178). Every check passed on the third run.
- **770-00-menu:** the `+` menu's New Agent in Worktree submenu lists Claude Code, Codex, Gemini CLI
  and OpenCode (the last two are the box's own,
  L-claude-770-the-worktree-submenu-lists-the-boxs-real-agent-clis-001).
- **770-01-boxes (REQ-001):** "Claude Code in a New Worktree", the prompt, "Also start:" with clear
  boxes for Codex, Gemini CLI and OpenCode, and `agent/tidal-prism from main`.
- **770-02-ticked (REQ-002):** Codex ticked; the line reads `agent/cyan-poppy-claude,
  agent/cyan-poppy-codex from main`.
- **770-03-two-rows (REQ-003, REQ-004):** the rail shows `cyan-poppy-claude` with its Claude Code
  terminal and `cyan-poppy-codex` with its Codex terminal. The log: `claude cyan-poppy-claude/repo`
  then `codex cyan-poppy-codex/repo`, both with `["compare the \"login\" fix"]`; branches
  `agent/cyan-poppy-claude` and `agent/cyan-poppy-codex`, each with base `main`.
- **770-04-one (REQ-005):** a third row, `warm-bloom` on `agent/warm-bloom`, with Claude Code alone
  launched there with `["on its own"]`.
- **Not reached:** a taken folder for one agent of a set (the set moves to the next name), checked
  in the review against `free_folders`.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide's Worktree agents step 2; `marley_workbench.md`
  (Worktree agents, "Several agents from one prompt"). No Zed path changed.
- **Knowledge:** L-claude-770-the-worktree-submenu-lists-the-boxs-real-agent-clis-001;
  AD-claude-770-a-set-of-worktree-agents-shares-one-name-001. No failure found.
- **Brain:** `brain decide` on d982cf6fd8ce4b308174de96c279d5ac.
