---
pipeline_id: ffc6c7ad-7db3-4088-9886-a7e08b88d478
ticket: docs/planning/tickets/open/TICKET-821-tmux-sessions-under-their-project.md
status: Phase 1 — Plan PASS; queued (Chad 2026-10-20: "not yet")
title: tmux sessions under their project, and a close prompt
type: feature
slice: the Marley layout's rail (#808 tmux sessions, #819 New tmux Terminal, #550 the close guard)
references: []
---

## Title
A tmux session shows as an ordinary terminal row under the project it started in, not in a TMUX
section of its own, and closing its tab asks whether to end the session or only hide it (Chad,
2026-10-20: "tmux sessions should be inside the panes as a normal terminal session not isolated
to its own thing. When exit we should have a popup "do you want to close the tmux session" or
"hide"").

## Scope
### In
- The listing reads each session's start folder (`#{session_path}`).
- Placement: a local session goes under the open local project whose folder holds its start
  folder, the deepest such folder winning; a session of a host `marley.tmux_hosts` names goes
  under an open remote project on that host whose folder holds it; any other session goes under
  the Home group.
- A session with no attached tab is a row of its own under its project: the agent's icon or the
  terminal icon, its name, and `tmux` with the agent, the windows and `attached` when another
  client has it. A click attaches it in a tab of that project's workspace.
- A session with an attached tab in Marley is that tab's terminal row only, with no second row.
- A host that cannot be listed shows one error row under Home: `tmux on <host>: <reason>`.
- Closing one attached tmux tab (a tab close, not a window close or a quit) asks Close Session,
  Hide or Cancel. Close Session runs `tmux kill-session -t =<name>` on its host and closes the
  tab; Hide closes the tab and leaves the session; Cancel keeps the tab.
- A remote project on a host `marley.tmux_hosts` names offers New tmux Terminal on its + and
  right-click, as a local project does since #819, so the + that the removed TMUX · host heading
  carried has a home.
- The TMUX · host sections, their fold and their + are removed from the rail.

### Out (explicitly deferred)
- New Agent in tmux on a remote project: the installed-agent list is this machine's.
- A prompt when the window closes or Marley quits: the attach clients end and every session
  keeps running, as today.
- Turning `tmux_hosts` on for a saved host on Chad's box: his setting, his call.
- Placing a session under a closed project (#606): a session whose project is closed goes under
  Home.

## Reference (§20)
N/A — Marley-specific. Neither Warp nor Zed lists tmux sessions; Orca's report
(`docs/orca_architecture/04-remote-control-and-mobile.md` §3.2 item 5) proposes tmux only as the
persistence layer under remote terminals and shows no list of sessions. Upstream Zed keeps the
tab close path that #550's guard hooks (`workspace` crate, `Pane::close_items` asking
`MarleyCloseGuard`), and this ticket adds a branch to that guard.

### Prior art
- tmux 3.7c (on the box) gives the start folder as `#{session_path}`, ends a session with
  `kill-session -t =NAME`, and detaches a client when its process ends, which is all Hide needs.
- Marley's own close guard, `marley_workbench/src/close_guard.rs` (#550): a three-button
  `window.prompt` before a tab close, an `asking` flag against a second prompt, and the tab
  close told apart from a window close by `MarleyClose::intent`. Reused, not rebuilt.
- The rail's thread rows (`rail.rs` `group_threads`, `tab_homes`): rows that exist without an
  open item, placed by `ProjectGroupKey::path_list()` and `host()`, with a tab's own row winning
  over the listing's. The tmux rows follow the same rule.
- `remote::open_words` and the task id it resolves (`<id_base>_<hashes>`), so a tmux attach tab
  can be known again by its spawned task's id (`remote::is_remote` reads the same id).
- Zed's remote-project terminals and tasks run on the project's host over Zed's own ssh, so a
  remote project's attach is the plain `tmux attach-session` run in that workspace.
- Checked and not the owner: `terminal_view`'s `is_dirty`/save prompt (only for saveable items)
  and `marley_remote` (one pane per ssh, no tmux).

## UI proof
Scenario `script/e2e/821-tmux-sessions-under-their-project.sh`, `compositor sway`, a tmux server
of the run's own (`TMUX_TMPDIR`), `marley.tmux_hosts` unset, a project `repo`.
- `821-01-placed` — `build` (started in `repo`) under `repo`, `loose` (started in the run's home)
  under Home, no TMUX section (REQ-001, REQ-002, REQ-007).
- `821-02-attached` — after a click on `build`: a tab attached to it in `repo`'s workspace, and
  `build` as that tab's terminal row only (REQ-003, REQ-004).
- `821-03-asked` — the tab's close: the prompt naming `build` with Close Session, Hide and Cancel
  (REQ-005).
- `821-04-hidden` — after Hide: the tab gone, `build`'s own row back under `repo`; the run checks
  tmux still lists `build` (REQ-006).
- `821-05-closed` — `build` attached again, its tab closed, Close Session: no `build` row; the run
  checks tmux no longer lists it (REQ-005).
- `821-06-cancel` — `loose` attached, its close cancelled: the tab still open (REQ-005).
- `821-07-host-down` — a `tmux_hosts` entry the fake ssh fails: its error row under Home
  (REQ-008).
- Not reached: a real remote project on a real host (REQ-009, REQ-010 are read in the diff; the
  remote words go through the same `open_words` and listing a scenario's fake ssh covers).

## Locked-In Decisions
- D1 — Placement is by the session's start folder (`#{session_path}`), not a pane's current
  folder: it is fixed for the session's life, so a row does not jump while someone `cd`s.
- D2 — The deepest project folder holding the start folder wins, as a worktree's terminals go to
  the worktree over its parent checkout.
- D3 — The prompt shows for a tab close only. A window close or a quit detaches every attach and
  leaves the sessions, since nothing is lost and a prompt per session would bury a quit.
- D4 — The tmux question comes before #550's working-agent question: a tmux tab's foreground
  program is the `tmux` client, so the agent check never matches it, and Hide keeps the agent
  running anyway.
- D5 — A remote session attaches inside its remote project's workspace through Zed's own remote
  task path; under Home it attaches as #808 does, `ssh -t` from a local workspace.
- D6 — Close Session ends the session after the tab closes, through `kill-session` on its host,
  and lists the host again soon; a failed kill shows in a prompt and the row stays.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a session on this machine started in a folder an open local project holds, the rail shall show the session as a terminal row under that project. | `821-01-placed` |
| REQ-002 | WHEN a session's start folder is in no open project's folder, the rail shall show it under the Home group. | `821-01-placed` |
| REQ-003 | WHEN the user clicks a session's row, Marley shall attach the session in a terminal tab of that row's project. | `821-02-attached` |
| REQ-004 | WHILE a session has an attached tab in Marley, the rail shall show it only as that tab's terminal row. | `821-02-attached` |
| REQ-005 | WHEN the user closes the tab of one attached tmux session, Marley shall ask Close Session, Hide or Cancel; Close Session shall end the session, and Cancel shall keep the tab. | `821-03-asked`, `821-05-closed`, `821-06-cancel` |
| REQ-006 | WHEN the user picks Hide, Marley shall close the tab, leave the session running, and show the session's row again. | `821-04-hidden` |
| REQ-007 | The rail shall show no TMUX · host section. | `821-01-placed` |
| REQ-008 | WHEN a host `marley.tmux_hosts` names cannot be listed, the rail shall show one error row for it under Home naming the host and the reason. | `821-07-host-down` |
| REQ-009 | WHEN a session of a named host started in a folder an open remote project on that host holds, the rail shall show it under that project and attach it in that project's workspace. | review of the diff |
| REQ-010 | WHEN a remote project's host is one `marley.tmux_hosts` names, its + and right-click shall offer New tmux Terminal. | review of the diff |
| REQ-011 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the listing's start folder, placement in the snapshot, the tmux row kind, the
  close guard's tmux branch, the remote project's +, the TMUX sections removed; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, guide and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
