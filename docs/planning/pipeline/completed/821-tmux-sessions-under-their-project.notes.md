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
- **Taken up again** 2026-10-20 under Chad's /goal "finish the remaining tickets we discussed",
  promoted from `queued/`; the seams re-read against the tree at `12e3bf1482` (#823's one run of
  rows did not touch the tmux sections).
- **Built:** `tmux_sessions.rs`: `#{session_path}` read into `TmuxSession::folder`; the fold,
  `toggle_folded`, `heading_words` and `new_session` gone; `TmuxHosts::attaches` with `Attach`
  and `attach_of`, recorded by each opener; `is_here` and `reached_by`; `attach` through `open`,
  which in a remote project on the host runs the plain words through `open_in_project`;
  `new_local_session` became `new_session_in`, which starts a session in a remote project on its
  host too; `kill`; `error_words`; `session_words` leads with `tmux`. `remote.rs`: `open_words`'
  scheduling half is `schedule_words`, and `open_on_project_host` uses it for a remote project.
  `rail.rs`: `placed_tmux`, `tmux_group`, `tmux_row` (the terminal row's `>_` mark and `pl_2`
  indent, a click attaching in the row's group), `tmux_error_row`, `render_blocks` appending a
  group's sessions after its rows, `render_tmux`, `tmux_section` and `new_tmux_session` gone,
  `TmuxFolder` and `tmux_folder`'s remote branch. `close_guard.rs`: `tmux_attach` and `ask_tmux`
  before the working-agent question. The `tmux_hosts` doc string and its ledger row.
- **Deviations from the design:** no `marley_rail` change. The rows are drawn from the
  `TmuxHosts` global at render time and appended to their group's block, outside the keys and
  the filter as #808's sections were, instead of a `Row::Tmux` in the pure walk; a session is not
  a keyboard row or a filter match. A listing therefore redraws them without a snapshot rebuild,
  which settles the plan's first risk. The registry keys on the task's `full_label`, not its id.
- **Review:** `placed_tmux` reads entities only (no update), so a render cannot re-enter. A
  session whose attach ended but whose tab is still open shows its own row again (the task is not
  `Running`). Matching a remote project to a host goes by `connection_type() == "ssh"` and the host
  name; a user or port mismatch on the same host still matches, and a session goes to Home when
  nothing matches. `kill`'s failure reaches a prompt. A tab closed with Close All asks nothing
  (more than one item), as D3 says.
- **Gate:** see below, after the docs.

## Phase 3 — Test
- **Scenario:** `script/e2e/821-tmux-sessions-under-their-project.sh`, `compositor sway`, a tmux
  server of the run's own, `tmux_hosts` unset, a fake ssh refusing the saved host `gone`. Fixes to
  the scenario on the way: the rows' and the prompt's points from the first runs' shots (the
  prompt's buttons stand in a column, L-claude-821). One fix to the source: the rows got the
  terminal row's `>_` mark and `pl_2` indent after `821-01` showed them a step left of `repo —
  bash`. Last run: five checks pass (`build` attached, `build` runs after Hide, `build` ended
  after Close Session, `loose` still attached after Cancel).
- **Shots:**
  - `821-01-placed` (REQ-001, REQ-002, REQ-007): `loose` (`tmux · 1 window`) under Default, `build`
    under `repo` after `repo — bash`, both with the `>_` mark; no TMUX section.
  - `821-02-attached` (REQ-003, REQ-004): the tab `this machine · build` in `repo`'s workspace,
    tmux's status line at its foot; `build` shows only as that tab's terminal row.
  - `821-03-asked` (REQ-005): "Close the tmux session build?", the detail, and Close Session, Hide,
    Cancel.
  - `821-04-hidden` (REQ-006): the tab gone, `build`'s own row back under `repo`; tmux still has it.
  - `821-05-closed` (REQ-005): after Close Session, no `build` row; tmux no longer has it.
  - `821-06-cancel` (REQ-005): `loose` attached in Default (its Home page opened there) and its tab
    still open after Cancel.
  - `821-07-host-down` (REQ-008): a red row under Default, "tmux on gone: ssh: connect to h…".
- **Not reached:** a remote project on a real host (REQ-009, REQ-010), read in the diff.
- **Seen for Chad:** an attached tab's terminal row shows the task's command (`/usr/bin/bash -i
  -c 'env' · running`) as task terminals do, not tmux words.
- **Regression note:** #808's scenario drives the TMUX headings this ticket removes; it needs
  rewriting when the regression pass runs.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, `docs/marley/guide.md` (tmux sessions in the rail),
  `docs/marley_architecture/marley_workbench.md` (the tmux section and `open_words`),
  `docs/marley/zed-touchpoints.md` (the `settings_content/src/marley.rs` row).
- **Knowledge:** `L-claude-821-a-prompts-buttons-stand-in-a-column-001`. No F block.
- **Brain:** `decisions/marley-lists-tmux-sessions-under-the-project-they-started-in-and-asks-on-closing-an-attached-tab`
  (consultation `09f43ca91bb44974a6c85d00c44500d7`, follow-up 2026-11-03).

