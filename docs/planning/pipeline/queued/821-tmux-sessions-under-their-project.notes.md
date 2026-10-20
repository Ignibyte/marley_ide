# tmux sessions under their project, and a close prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-821-tmux-sessions-under-their-project.md
- **Pipeline spec:** 821-tmux-sessions-under-their-project.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-20, in order: move the tmux sessions up near Threads; close a tmux
  session; how remote tmux sessions work. Asked where, he picked a TMUX tab of its own, then
  overrode it: "tmux sessions should be inside the panes as a normal terminal session not
  isolated to its own thing. When exit we should have a popup "do you want to close the tmux
  session" or "hide"". Asked where a session in no project goes: Home group. On remote: told
  that `marley.tmux_hosts` already takes `ssh_connections` names (his box has a saved ssh host,
  `tmux_hosts` unset). He also had me close the one Marley session on the box
  (`marley-2418910d`, a bare bash in `~`), since nothing in Marley could.
- **Classification / tier:** feature; the rail, tmux, the close guard.
- **Recall (§18.3):**
  - PR on #819: an agent CLI goes into a tmux pane by name through `send-keys`, never by a path
    (prevention-rules.md:3217); the remote project's New tmux Terminal keeps that.
  - #808 notes: each word after an ssh destination goes through `harness_hosts::shell_word`; the
    listing uses `MARLEY_SSH` so a scenario's fake wins; a host is raced on its own (20 s).
  - #550 close guard: a second `window.prompt` cannot show over the first (`Guard::asking`).
  - Rusty's brain (consultations `45b014cc668d428ca7c65212a7903f6b`,
    `c75e17158b6e463188ead7370113d900`): nothing on this seam.
- **Discovery (Explore):** terminal rows come only from open `TerminalView` items
  (`rail.rs` `build_snapshot` → `terminal_snapshot`, `marley_rail::TerminalSnapshot`); groups
  carry `ProjectGroupKey { paths, host }`; the Home group is `groups::GroupKind::Home`, first in
  `rail_groups`; `remote::open_words` refuses a non-local project; the close guard is
  `close_guard.rs::guard` behind Zed's `MarleyCloseGuard` (`workspace.rs:733`,
  `pane.rs:1985`); thread rows are the model for rows with no open item.

### Design
- **`tmux_sessions.rs`** (Marley crate):
  - `PANE_FORMAT` gains `#{session_path}` as a fifth field; `TmuxSession` gains
    `folder: PathBuf`. `parse` reads five fields, `splitn(5, '\t')`.
  - `TmuxHosts` drops `folded` and `toggle_folded`. It gains `attaches: Vec<Attach>`, one per
    attach Marley opened: the task id `open_words` resolves, the host's name and the session's
    name. `open`/`open_here` record it; the close guard and the rail read it.
  - `attach(name, session, workspace, ..)` takes the workspace of the row's project. For
    `Place::Host` with a remote workspace on that host it runs the plain tmux words in that
    workspace (D5): `open_words` gains a flag that lets a remote project through when the words
    are meant for its host. Else as today.
  - `kill(host_name, session, cx) -> Task<anyhow::Result<()>>`: `tmux kill-session -t =NAME`
    here, or behind the listing's `ssh` arguments on a host; then `list_soon`.
  - `new_session` (the heading's +) goes; `new_remote_session(workspace, folder, ..)` replaces
    it for a remote project's + (REQ-010), `new-session -A -s <name> -c <folder>` in that
    remote workspace.
  - `heading_words` goes; a failed host's words become `error_words(name, reason)`.
- **`marley_rail`** (Marley crate, pure): `ProjectSnapshot` gains `tmux: Vec<TmuxSnapshot>`
  (host name, session name, agent, words); `Row::Tmux(TmuxRow)` drawn after the project's
  terminals and before its Browser tabs; `walk` and `rail_rows` place it; it takes part in keys
  and the filter, as other rows do. A host's failure is a `TmuxSnapshot` with an error.
- **`rail.rs`** (Marley crate):
  - `build_snapshot` places each listed session: skip one whose name and host match an entry in
    `attaches` whose task still has a running `TerminalView` (REQ-004); else the deepest local
    group folder holding `folder` for `Place::Here`, the deepest remote group on that host for a
    host (matching `key.host()` against the saved host's `SshTarget`: host, user, port), else
    the Home group (REQ-001, REQ-002, REQ-009). Failed hosts go to Home (REQ-008).
  - `render_row` draws `Row::Tmux` with `row_card` as `tmux_row` does; a click runs
    `attach_tmux` in the row's group, not always Home.
  - `render_tmux`, `tmux_section`, `new_tmux_session` go (REQ-007).
  - `tmux_folder` returns the remote project's first folder when its host is named in
    `tmux_hosts`; the menu then offers New tmux Terminal (not New Agent in tmux) (REQ-010).
- **`close_guard.rs`** (Marley crate): before `working_agents`, when `close.intent.is_none()`
  and the items hold exactly one terminal whose task id is in `attaches`: `window.prompt(
  Warning, "Close the tmux session <name>?", detail, ["Close Session", "Hide", "Cancel"])`
  under the same `asking` flag. `Ok(0)`: true, then `kill`; a failed kill goes to a prompt
  (D6). `Ok(1)`: true. Else false. Several tmux tabs at once (Close All) or a window close:
  no question (D3).
- **Docs at Complete:** `docs/marley/guide.md` tmux section, `docs/marley_architecture/
  marley_workbench.md`, `marley_rail.md`, CHANGELOG; the `tmux_hosts` doc string in
  `settings_content/src/marley.rs` (Zed crate, its ledger row already names #808).

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_workbench/src/tmux_sessions.rs` | Marley | start folder, attaches, kill, remote new session; headings and fold go |
| `crates/marley_workbench/src/rail.rs` | Marley | placement in the snapshot, `Row::Tmux` drawn, TMUX sections go, remote + |
| `crates/marley_workbench/src/close_guard.rs` | Marley | the tmux question |
| `crates/marley_workbench/src/remote.rs` | Marley | `open_words` lets a remote project's own tmux words through |
| `crates/marley_rail/src/marley_rail.rs` | Marley | `TmuxSnapshot`, `Row::Tmux`, walk order |
| `crates/settings_content/src/marley.rs` | Zed | the `tmux_hosts` doc string only |
| `script/e2e/821-tmux-sessions-under-their-project.sh` | — | the visual check |

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-001, REQ-002, REQ-007 | The run's tmux has `build` (`-c repo`) and `loose` (`-c $HOME`); Marley opens `repo`. | `821-01-placed` |
| REQ-003, REQ-004 | A click on `build`. | `821-02-attached` |
| REQ-005 | The tab's close key. | `821-03-asked` |
| REQ-006 | A click on Hide; the run checks `tmux has-session -t =build`. | `821-04-hidden` |
| REQ-005 | A click on `build`, the close key, Close Session; the run checks `build` is gone. | `821-05-closed` |
| REQ-005 | A click on `loose`, the close key, Cancel. | `821-06-cancel` |
| REQ-008 | The run writes `tmux_hosts: ["local", "gone"]` with `gone` a saved host the fake ssh fails. | `821-07-host-down` |
- Not reached: a remote project on a real host (REQ-009, REQ-010), read in the diff.

### Risks and decisions
- The rail draws from a snapshot rebuilt on workspace changes; a tmux listing change must
  trigger a rebuild, not only a `notify` (today the rail only observes `TmuxHosts` to notify).
- Matching a remote group to a saved host: `RemoteConnectionOptions::Ssh` against `SshTarget`;
  a mismatch drops the session to Home, which is safe.
- The sessions become part of the keys and the filter: a filter now matches a session's name.
- Decision for the brain at Complete: placement by start folder, the tab-close-only prompt.

## Phase 2 — Code
