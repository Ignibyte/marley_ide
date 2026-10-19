---
pipeline_id: 093f5c34-ff25-4f08-89e6-727d02637cac
ticket: docs/planning/tickets/closed/TICKET-819-new-tmux-terminal-and-agent.md
status: Phase 4 — Complete PASS
title: New tmux Terminal and New Agent in tmux
type: feature
slice: the Marley layout's rail (#744 the + menu, #808 tmux sessions, #820 one builder)
references: []
---

## Title
When tmux is installed, a project's `+` and its right-click offer New tmux Terminal and New Agent
in tmux, so a shell or an agent started from the rail keeps running when Marley closes, and the
rail lists this machine's tmux sessions with no setting (Chad, 2026-10-19: "new tmux agent and new
tmux terminal session basically").

## Scope
### In
- `tmux_sessions::installed(cx)`: tmux on Marley's agent search path.
- An unset `marley.tmux_hosts` means `["local"]` while tmux is installed; an explicit list,
  `[]` included, is kept as it is.
- `Rail::new_entries` (#820), for a local project or group: New tmux Terminal after New Terminal,
  and New Agent in tmux (a submenu of the installed agent CLIs) after New Agent in Worktree.
- `tmux_sessions::new_local_session(folder, command, ..)`: `tmux new-session -A -s <name> -c
  <folder>`, and for an agent `; send-keys -t <name> <cli> Enter`, on this machine's default
  server, attached in a terminal tab as #808's sessions are, then the listing again soon.

### Out (explicitly deferred)
- tmux on a remote project's host: the entries show for local projects and groups only.
- Marley's per-terminal setup for the agent (its identity variables, plugin folders): a pane takes
  the tmux server's environment, which Marley does not choose.

## Reference (§20)
N/A — Marley-specific: Zed has no tmux support; #808 is Marley's own, and Warp's maps hold no
tmux launcher. The entries follow #808's attach (`tmux_sessions::open`) and #744's menu.

### Prior art
- `crates/marley_workbench/src/tmux_sessions.rs`: `follow` (reads `tmux_hosts`), `open` (the
  Place::Here words `env -u TMUX tmux …` through `remote::open_words` with `LOCAL_ID_BASE`),
  `new_session` (`new-session -A -s` with a `SessionName`), `list_soon`, `TmuxHost::label`
  ("this machine").
- `crates/marley_workbench/src/agents.rs`: `installed_clis` (`which::which_in` over the
  launcher's `search_path`), `cli_icon`, `AgentKind::program`.
- `crates/marley_workbench/src/rail.rs`: `new_entries` (#820), `worktree_agent_entries`,
  `launch_entries` (the project's first visible worktree as its root).
- L-claude-808: a scenario's tmux under a scratch `TMUX_TMPDIR` beneath `XDG_RUNTIME_DIR`,
  `-f /dev/null`, the server killed in `teardown`.

## UI proof
`script/e2e/819-new-tmux-terminal-and-agent.sh`, under sway, tmux on a scratch `TMUX_TMPDIR`,
no `tmux_hosts` set, a stand-in `claude` first on Marley's PATH:
- `819-01-plus`: `repo`'s `+`: New tmux Terminal under New Terminal, New Agent in tmux › after
  New Agent in Worktree (REQ-001).
- `819-02-tmux-terminal`: New tmux Terminal: a tab `this machine · <name>` attached to a tmux
  session, `pwd` printing `repo`, and `TMUX · this machine` in the rail listing it (REQ-002,
  REQ-004).
- `819-03-tmux-agent`: New Agent in tmux › Claude Code: a tab attached to a session running the
  stand-in in `repo`, its rail row marked with Claude Code (REQ-003).

## Locked-In Decisions
- D1 — Auto-detection means an unset `tmux_hosts` lists this machine while tmux is installed; a
  list the user wrote wins, `[]` turning it off.
- D2 — The entries start on this machine's default tmux server, in the project's first folder
  (a group's: the home folder), each in a session of Marley's naming, attached at once.
- D3 — New Agent in tmux types the CLI's name into the new session's shell (`new-session … ;
  send-keys -t <name> <cli> Enter`), so the agent runs as in any terminal of the user's, by the
  shell's own PATH, and the session stays when the agent exits. (Changed in Code: running the path
  Marley found on its login PATH started the real Claude Code in a check run, where the pane's
  shell had a stand-in first.)

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE tmux is installed, a local project's `+` and right-click shall offer New tmux Terminal and New Agent in tmux. | Shot `819-01-plus` |
| REQ-002 | WHEN the user picks New tmux Terminal, Marley shall start a tmux session in the project's folder and attach it in a terminal tab. | Shot `819-02-tmux-terminal`; `expect` on `tmux list-sessions` |
| REQ-003 | WHEN the user picks an agent under New Agent in tmux, Marley shall start that agent CLI in a new tmux session in the project's folder, attached in a tab. | Shot `819-03-tmux-agent`; `expect` on `tmux list-panes` |
| REQ-004 | WHILE tmux is installed and `marley.tmux_hosts` is unset, the rail shall list this machine's tmux sessions. | Shot `819-02-tmux-terminal` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — detection, the default, the session start, the entries; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the scenario on the debug build, every shot read.
- **P4 Complete** — CHANGELOG, the crate note, the settings doc, close, archive, commit.
