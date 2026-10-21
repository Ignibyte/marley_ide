# A Sessions view for every tmux session — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-832-a-sessions-view-for-tmux.md
- **Pipeline spec:** 832-a-sessions-view-for-tmux.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #831; Chad's overnight goal (2026-10-20/21).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`).
- **Split:** the board's center Sessions page → TICKET-861 (queued before #794); the bottom row's
  count is #833's.
- **Recall (§18.3):**
  - L-808/L-543: a scenario's tmux server needs a short socket path (`TMUX_TMPDIR` under
    `XDG_RUNTIME_DIR`), `-f /dev/null`, and a kill at teardown; #821's scenario exports it to
    Marley, so the box's own sessions stay out.
  - L-808 (Zed holds a second run of a running task): an attach clicked twice must activate the
    running terminal. `attach` already goes through `open`, which does.
  - The brain (consultation 332d2acc): nothing on this seam.
- **Discovery:** `tmux_sessions.rs` (types :46-158, `follow` :176, `attach` :400, `kill` :591,
  `session_words` :652, `error_words` :666, `report` :674); `rail.rs` `placed_tmux` (:4994-5043),
  `tmux_row` (:7928), `tmux_error_row` (:7972), `tmux_group` (:7998), `attach_tmux` (:2344),
  `render_view_switch` / `show_view` (~:5213-5268), `render_header`'s `!threads` checks, the root
  render's view branch, `threads.shown` uses; `remote::saved_hosts` (`remote.rs:150`);
  `rail_strip.rs` tiles.
- **Decisions:** D1 to D5 in the spec.

### Design
- **View state:** `enum RailView { Projects, Threads, Sessions }` in `rail.rs`, kept in a new
  `view` field; `threads.shown` stays as the Threads side's flag and is set from it (to keep its
  many readers), or is replaced by `self.view == RailView::Threads` where it is read. Code picks
  the smaller diff after counting the uses. `show_view(view, window, cx)`: Sessions focuses the
  rail itself. The switch draws three buttons; SESSIONS shows a dot when a host failed.
- **`rail_sessions.rs`** (new `#[path]` submodule of `rail`, as `rail_strip.rs`):
  - `render_sessions(&self, cx) -> impl IntoElement`: a scrolling `v_flex` with, per host in
    `TmuxHosts::hosts`, `session_host_heading` and the rows, then `render_tmux_hosts`.
  - The placement and attached-here set come from a helper split out of `placed_tmux`:
    `tmux_attached_here(cx) -> HashSet<(SharedString, String)>` and `tmux_place(host, session)
    -> Option<usize>` (`tmux_group` or Home). `placed_tmux` uses both, so the two views agree.
  - Row: `row_card` like `tmux_row`, with a line `"{under} · {state} · {n} window(s)"` (`under`
    from the group's name, else "Home"); a click calls `attach_tmux` for a detached or elsewhere
    session, or activates the terminal entry whose attach matches (the Projects view's terminal-row
    click path) for one attached here; `right_click_menu` with Show/Attach and Close Session….
  - Close: `window.prompt(PromptLevel::Warning, "End the tmux session …?", Some("Its programs
    stop."), &["End Session", "Cancel"], cx)`; on End, `tmux_sessions::kill(host, session, cx)`
    awaited, its error through `tmux_sessions::report`, then `list_soon`. (`list_soon` is made
    `pub(crate)`.)
  - Hosts: `tmux_sessions::host_choices(cx) -> Vec<(SharedString /*name*/, SharedString
    /*label*/, bool /*listed*/)>`: `local` first while tmux is installed, then each saved host by
    nickname (or host), `listed` from the current `TmuxHosts` names. Each row: `Checkbox::new(id,
    ToggleState)` with the label and `local`/`ssh_connections` muted; on click
    `tmux_sessions::set_listed(name, bool, cx)`, which writes `marley.tmux_hosts` (D4).
- **Strip tile:** `rail_strip.rs` `render_strip_tiles` appends, while `TmuxHosts` has any host, a
  36 px tile with `tx` in the buffer font and a count badge (total sessions, muted fill, not the
  needs-you pink); a click sets `view = Sessions` and calls `toggle_rail_strip`.
- **File manifest:**
  | File | Owner | Change |
  |---|---|---|
  | `crates/marley_workbench/src/rail.rs` | Marley | `RailView`, switch, render branch, placement helpers |
  | `crates/marley_workbench/src/rail_sessions.rs` | Marley | new: the Sessions view |
  | `crates/marley_workbench/src/rail_strip.rs` | Marley | the Sessions tile |
  | `crates/marley_workbench/src/tmux_sessions.rs` | Marley | `host_choices`, `set_listed`, `list_soon` visibility |
  | `script/e2e/832-a-sessions-view-for-tmux.sh` | Marley | the scenario |

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002/005 | own tmux with `build` (in `repo`) and `loose` (in home); `forge` saved in `ssh_connections`; click SESSIONS | `832-01-sessions` |
| REQ-003/002 | click `build`'s row; settle | `832-02-attached` |
| REQ-004 | right-click `loose`, Close Session…, shot, Enter on End Session; `tmux ls` printed | `832-03-close-asks`, `832-04-closed` |
| REQ-005 | click `forge`'s checkbox; print `marley.tmux_hosts` | `832-05-host-added` |
| REQ-006 | `marley.rail_shrunk` true; shot; click the Sessions tile | `832-06-strip`, `832-07-from-strip` |
| REQ-001 | theme Marley Day | `832-08-day` |

Points are read from a probe run (L-claude-831); `marley.tmux_hosts` is left unset so this machine
(the run's own server) is listed, and the box's sessions stay out because the run exports its own
`TMUX_TMPDIR` to Marley (#821's way).

### Risks
- `forge` unreachable: its listing must fail within `LIST_TIMEOUT` (20 s) without stalling the
  others; ssh to `127.0.0.1:1` is refused at once.
- `threads.shown` has many readers; replacing it could touch code far from this ticket. Keeping it
  as a derived flag is the smaller, safer diff.

## Phase 2 — Code
- **Built:** `rail_sessions.rs` (new): `SessionsView { Hidden, Shown }`, `render_sessions` (a
  heading per host with a state dot and count, its error line, a row per session, the empty
  note, HOSTS TO LOOK ON), `session_row` (`row_card` with `<placed under> · <state> · <windows>`,
  click → `open_session`, right-click → Show/Attach and Close Session…), `open_session` (activate
  the attached-here terminal, else `attach_tmux` in the placed group), `host_heading`,
  `host_choices` (checkboxes), `close_session` (the prompt, `kill`, the error prompt or
  `list_soon`). `rail.rs`: the `sessions` field, `projects_shown`, `show_sessions`, `show_view`
  clearing Sessions, the three-button switch (SESSIONS with an error dot while a host fails), the
  render branch, the row keys, fold and Enter guarded to the projects' view, and
  `tmux_attached_here` / `tmux_place` split out of `placed_tmux` so both views agree.
  `tmux_sessions.rs`: `HostChoice`, `host_choices`, `listed_names`, `set_listed`, and `list_soon`
  made `pub(crate)`. `rail_strip.rs`: `sessions_tile` after the project tiles.
- **Deviations:** the Rails flag became `SessionsView` (clippy's `struct_excessive_bools`); the
  header's "placed under" names the Home group by its own name (`Default` on this profile).
- **Review of the diff:** the row keys, fold and Enter acted on hidden project rows while Sessions
  showed; guarded with `projects_shown`. A kill's error has its own words ("Could not end the tmux
  session") rather than `report`'s open wording.
- **Probe runs** (the scenario's points; L-claude-831) found only scenario points.
- **Gate:** `script/gates.sh --diff` 17/17 on the first run.

## Phase 3 — Test
- **Scenario:** `script/e2e/832-a-sessions-view-for-tmux.sh` (sway, the gated tree's debug
  build; its own tmux server under `XDG_RUNTIME_DIR`, a stand-in ssh for `forge`). Exit 0, eight
  shots. Its points came from three probe runs, all before the gate.
- `832-01-sessions`: PROJECTS · THREADS · SESSIONS with SESSIONS lit; "this machine 2" with a green
  dot; `build` "repo · detached · 1 window", `loose` "Default · detached · 1 window"; HOSTS TO
  LOOK ON with This machine (local) ticked and forge (ssh_connections) not (REQ-001, REQ-002,
  REQ-005 met).
- `832-02-attached`: `build` clicked. A tab "this machine · build" in `repo` with tmux's shell,
  and the row now "repo · attached here · 1 window" (REQ-003, REQ-002 met).
- `832-03-close-asks`: `loose`'s menu → Close Session…: "End the tmux session loose on this
  machine?", "Its programs stop.", End Session / Cancel. `832-04-closed`: after Enter, "this
  machine 1" with only `build`, and `tmux ls` in the log prints `build` (REQ-004 met).
- `832-05-host-added`: forge ticked. The log prints `tmux_hosts: ['local', 'forge']`, and a red
  "forge" heading appears with "tmux on forge: ssh: connect to host forge port 22: Connection
  refused" (REQ-005 met).
- `832-06-strip`: shrunk. Tiles D, R (ringed) and `tx` with a badge 1. `832-07-from-strip`: the
  tile clicked opens the rail on SESSIONS (REQ-006 met).
- `832-08-day`: Marley Day. The view reads with green and red dots and the red error line
  (REQ-001 met).
- Nothing unreachable in the scope; the box's sessions stayed out (the run's own
  `TMUX_TMPDIR`).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #832); the guide's tmux section (The Sessions view); the
  architecture note's tmux section; look-plan W1.3 done with #861 named. No Zed path touched.
- **Knowledge:** L-claude-832-a-right-click-menu-opens-at-the-pointer-so-click-its-entries-inside-001.
  No bug reached Code or Test beyond the guards the review added (hidden rows taking keys), which
  were caught before any run, so no F block.
- **Brain:** decision on consultation 332d2acc (follow-up 2026-11-04).
- **Ticket** closed; #861 filed; pipeline archived; committed on `marley/workbench-shell`.
