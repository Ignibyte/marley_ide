---
pipeline_id: ba1b260c-126e-4659-bc75-0bbc0a54e6db
ticket: docs/planning/tickets/open/TICKET-832-a-sessions-view-for-tmux.md
status: Phase 4 — Complete PASS
title: A Sessions view for every tmux session
type: feature
slice: The Marley look, W1.3 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The rail gets a third view beside Projects and Threads: Sessions, every tmux session on every host
Marley looks on. Each host gets a heading with its state and count; each session gets a row with
the project it is placed under, whether it is attached, and its windows; a click shows or attaches
it, and the row's menu closes it. Under the sessions, every host Marley could look on (this machine
and each saved ssh host) has a checkbox that adds it to `marley.tmux_hosts` or takes it out. The
shrunk strip (#830) gets a Sessions tile that opens the view. Today tmux sessions only show
scattered under their projects (#821), a session attached in a tab disappears from the rail, and
the host list is a setting typed by hand.

## Scope
### In
- The rail's view switch: PROJECTS · THREADS · SESSIONS. Sessions shows in place of the
  projects' filter, inbox and rows; the header's add-project button shows with Projects only.
- The Sessions list, from the `TmuxHosts` global (#808):
  - A heading per host: a dot (green when listed, red when the listing failed, grey while
    waiting), the host's label (`this machine` or the saved host's) and its session count. A
    failed host shows its error under the heading.
  - A row per session: the agent icon or `>_`, its name, and a line `<placed under> · <state> ·
    N windows`. The state is `attached here` (a Marley tab holds it), `attached elsewhere` (a
    client outside Marley) or `detached`. "Placed under" is the project's name the Projects view
    puts it under, or Home.
  - A click shows the tab for a session attached here, or attaches it in the project it is
    placed under, as #821's row click does.
  - The row's right-click menu: Show or Attach, then "Close Session…", which asks and then ends
    the session (`tmux_sessions::kill`).
- HOSTS TO LOOK ON, under the sessions: this machine (while tmux is installed) and each
  `ssh_connections` host, each with a checkbox. A tick writes `marley.tmux_hosts` with that host
  added, starting from the hosts listed now when the setting is unset; an untick writes it without
  that host.
- The strip: a Sessions tile under the project tiles (`tx`, with a count of sessions); a click
  opens the rail on Sessions.

### Out (explicitly deferred)
- The board's center Sessions page (a table, New Session…, the placement note): TICKET-861.
- The bottom row's tmux count: TICKET-833 (W1.4).
- A filter field for sessions (the list is short; the projects' filter stays on Projects).

## Reference (§20)
N/A — Marley-specific. tmux sessions in the rail are Marley's own (#808, #821); neither Zed nor
Warp lists tmux sessions (Warp's `docs/warp_architecture/` maps no session browser). The behavior
is the design canvas's TmuxSessions board.

### Prior art
- The code we ship: `tmux_sessions.rs` (#808, #819, #821) owns the hosts (`TmuxHosts`,
  `TmuxHost { name, place, listing }`, `label`, `is_here`), the listing every 10 s, `attach`,
  `kill`, `session_words`, `error_words` and `report`. `rail.rs` has `placed_tmux` (the attached-
  here set and the placement through `tmux_group` or Home), `tmux_row`, `attach_tmux`, and the
  Projects | Threads switch (`render_view_switch`, `show_view`). `remote::saved_hosts` lists the
  `ssh_connections` entries. `ui::Checkbox` and `ToggleState` are Zed's checkbox.
- Ledger: L-claude-808-a-scenarios-tmux-socket-must-be-short-001 and L-claude-543 (a scenario's
  own tmux server under `XDG_RUNTIME_DIR`, killed at teardown), #821's scenario's setup (its own
  `TMUX_TMPDIR` exported to Marley), L-claude-831 (set `marley.tmux_hosts` for a stable layout;
  here the run's own server is the box's "local"), AD-claude-808 (each named host's own tmux).
  The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/832-a-sessions-view-for-tmux.sh` (sway): the run's own tmux server
(`TMUX_TMPDIR` under `XDG_RUNTIME_DIR`) with `build`, started in project `repo`, and `loose`,
started in the run's home; `repo` opened; one saved ssh host `forge` (an unreachable
`127.0.0.1:1` entry in `ssh_connections`) not in `tmux_hosts`.
- `832-01-sessions`: the SESSIONS button clicked. `this machine` with 2, `build` (under repo ·
  detached) and `loose` (under Home · detached), and HOSTS TO LOOK ON with this machine ticked and
  `forge` not (REQ-001, REQ-002, REQ-005).
- `832-02-attached`: `build` clicked: a tab attaches it, and its row reads `attached here`
  (REQ-003, REQ-002).
- `832-03-close-asks`, `832-04-closed`: `loose`'s menu, Close Session…, the question, then
  confirmed: `loose` is gone from the list and from `tmux ls` (REQ-004).
- `832-05-host-added`: `forge` ticked: `marley.tmux_hosts` printed as `["local", "forge"]`, and
  `forge`'s heading shows its failure (REQ-005).
- `832-06-strip`: the rail shrunk: the Sessions tile with its count; a click opens the rail on
  Sessions (`832-07-from-strip`) (REQ-006).
- `832-08-day`: Marley Day, the Sessions view (REQ-001).

## Locked-In Decisions
- D1 — The rail's views become `RailView { Projects, Threads, Sessions }` in the rail. The Threads
  side keeps its own state, and `show_view` takes the view.
- D2 — A session attached here keeps a row in Sessions (it leaves the Projects view's tmux rows
  as before, since its terminal row stands for it there).
- D3 — "attached elsewhere" is `attached && !attached_here`; tmux reports clients, not who.
- D4 — Ticking a host while `tmux_hosts` is unset writes the hosts listed now plus that one, so
  this machine stays listed; unticking the last host writes `[]`.
- D5 — Close asks first ("End the tmux session NAME on HOST? Its programs stop."), as #821's
  close of an attach tab does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user picks SESSIONS, the rail shall list each listed host with its state and count, and its sessions under it, in Marley Night and Marley Day. | `832-01-sessions`, `832-08-day` |
| REQ-002 | Each session row shall say where the session is placed, whether it is attached here, elsewhere or not, and its windows. | `832-01-sessions`, `832-02-attached` |
| REQ-003 | WHEN the user clicks a detached session, Marley shall attach it in a tab of the project it is placed under; a session attached here shall show its tab. | `832-02-attached` |
| REQ-004 | WHEN the user confirms Close Session… on a row, Marley shall end that tmux session. | `832-03-close-asks`, `832-04-closed`; `tmux ls` in the log |
| REQ-005 | The view shall list this machine and each saved ssh host with a checkbox, and a tick or untick shall write `marley.tmux_hosts`. | `832-01-sessions`, `832-05-host-added`; the printed setting |
| REQ-006 | WHILE the rail is shrunk, the strip shall show a Sessions tile with the session count, and a click shall open the rail on Sessions. | `832-06-strip`, `832-07-from-strip` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `RailView`, the Sessions list, the hosts checkboxes, the close question, the strip
  tile; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
