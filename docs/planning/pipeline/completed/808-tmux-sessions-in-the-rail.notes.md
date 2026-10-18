# tmux sessions in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-808-tmux-sessions-in-the-rail.md
- **Pipeline spec:** 808-tmux-sessions-in-the-rail.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "attaching to existing tmux agents or an ssh server that we can
  connect to has tmux it starts a new session", "maybe ssh servers we know about that could
  actually just open them up. This would be settings im guessing?"; he approved the sentence in
  the ticket ("ok lets build 3 and 4").
- **Classification / tier:** feature; the rail and remote terminals.
- **Recall (§18.3):**
  - PR-claude-740-quote-each-word-after-an-ssh-destination-001: each word after the destination
    goes through `harness_hosts::shell_word`.
  - AD-claude-543 and AD-claude-641: a remote terminal is a task terminal behind `ssh -t`; a task
    id under `marley-remote` gets the keepalive's dead-link checks and the reattach, and counts as
    remote for Claude Code's frames.
  - F-claude-742: a bare `ssh` resolves on two search paths; the attach uses `MARLEY_SSH` when
    set, as `remote::open` does, so a scenario's fake wins in both.
  - Rusty's brain (consultations `1f1dd7f7cdba4a5aa6a90aa7bb14daea`,
    `fb95e999ae7c4d3dbccb4ab7ecaf17fb`): nothing on this seam.
- **Discovery:** `rail.rs` (`render_harness` at the end of the project blocks, `harness_section`,
  `row_card`, `RowLine`, `groups::in_group`), `remote.rs` (`Host`, `saved_hosts`, `open`,
  `Links`), `harness_hosts.rs` (`shell_word`, `ssh_arguments`), `process.rs` (`output`),
  `settings_content/src/marley.rs`.

### Design
- **Setting** (`crates/settings_content/src/marley.rs`, Zed crate, its row extended):
  `tmux_hosts: Option<Vec<String>>`, documented with `local` and the `ssh_connections` names. No
  `default.json` entry: unset is off, as `harnesses` is.
- **`crates/marley_workbench/src/tmux_sessions.rs`** (new, Marley crate):
  - `TmuxHosts` global: per named host its `name`, its label (`this machine` or the name), its
    target (`None` here, else the saved host's `SshTarget`), its `Listing` (`Waiting`,
    `Sessions(Vec<TmuxSession>)`, `Failed(String)`) and `folded`; the poll task and the one-shot
    "list soon" task.
  - `init`: observes `SettingsStore`; `follow` reads the merged `marley.tmux_hosts`, keeps the
    listing of an unchanged host, resolves each name against `ssh_connections` (by nickname, then
    host; none: `Failed("no saved host named …")`), and starts the poll only while a host is named.
  - The poll: every 10 s, each host listed at once (`futures::future::join_all`), each listing
    raced against 20 s. Here: `tmux list-panes -a -F FORMAT`; a host: `ssh` (or `MARLEY_SSH`),
    `harness_hosts::ssh_arguments` plus `ConnectTimeout`, then the remote words through
    `shell_word`. Exit 1 with "no server running" or "error connecting to" is no sessions; any
    other failure is the first line ssh or tmux printed.
  - `parse`: one line per pane, `session \t windows \t attached \t command`; sessions in the
    order tmux gives them; the first pane whose command `agent_kind_of` names marks the session.
  - `attach(host, session, workspace, window, cx)` and `new_session(host, …)`: refused in a
    project not on this machine. A running tab of the same task id is activated; else the task
    is scheduled. Here: `env -u TMUX tmux attach-session -t =NAME` (id base `marley-tmux`);
    a host: through `remote::open_words` with the remote words quoted (id base
    `marley-remote-tmux`). The + uses `new-session -A -s marley-<8 hex>` and lists the host again
    after 1 s.
- **`remote.rs`** (Marley crate): `open`'s scheduling half becomes
  `pub(crate) fn open_words(host, label, id_base, words, workspace, window, cx)`, `Host` and
  `saved_hosts` become `pub(crate)`; `open` calls it.
- **`harness_hosts.rs`**: `ssh_arguments` becomes `pub(crate)`.
- **`rail.rs`** (Marley crate): `render_tmux` after `render_harness`, hidden while filtering:
  per host a heading (`TMUX · label`, the state words, a + `IconButton`), folded on a click, and
  a `row_card` per session (agent icon or `IconName::Terminal`, the name, one line: the agent's
  name, `N windows`, `attached`); a click runs `attach` in the Home group (`groups::in_group`).
  The rail observes `TmuxHosts`.
- **`process.rs`** and `.config/spawn-sites.txt`: their lists of programs name `tmux` and `ssh`.
  No new spawn call, so `SPAWN_SITES_PIN` stays.
- **`marley_workbench.rs`**: `mod tmux_sessions;` and its `init`.
- Ledger: the `crates/settings_content/src/marley.rs` row gains `tmux_hosts` (#808).

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-006 | Marley opens with `tmux_hosts` unset. | `808-01-off` |
| REQ-001, REQ-003, REQ-005 | The run writes `tmux_hosts: ["local", "box-2", "gone", "nowhere"]` into its settings; `local` has `build` (a shell that printed a line) and `agent` (`exec -a claude sleep`), `box-2` has `deploy` (`exec -a codex sleep`), the fake ssh fails for `gone`, and `nowhere` is no saved host. | `808-02-rail` |
| REQ-002 | A click on `build`; a second click on it; a click on `deploy`. | `808-03-attached`, `808-04-again`, `808-05-remote` |
| REQ-004 | A click on `box-2`'s +. The run checks the remote server lists a `marley-` session. | `808-06-new` |
- Not reached: a real ssh link, and a dropped one (#641's own scenario covers the supervision).

### Risks
- A host with a slow login delays only its own heading: each host is raced on its own.
- The 10 s listing over ssh adds a login per host every 10 s; a host whose login needs a touch
  or a password fails at once under `BatchMode` and says so.

## Phase 2 — Code
- **Built:** as the design. `tmux_sessions.rs` (the `TmuxHosts` global, `follow`, the poll,
  `list`, `read_listing`, `parse`, `attach`, `new_session`, `open`, `list_soon`, the words the rail
  shows, `report`); `remote::open_words` from `open`'s second half, with `Host`, `saved_hosts` and
  `TMUX_ID_BASE`; `harness_hosts::ssh_arguments` shared; the rail's `render_tmux`,
  `tmux_section`, `tmux_row`, `attach_tmux`, `new_tmux_session` and its fourteenth observer; the
  setting; `process.rs`'s and `.config/spawn-sites.txt`'s program lists.
- **Deviations:** `open_words` also shows a running tab of the same task, so the remote terminal's
  picker gets that too (its sessions are random, so it never matches). The error for a project not
  on this machine now reads "This terminal opens from a project on this machine", since a local
  tmux attach takes the same path.
- **Review of the diff:**
  - A listing that came back after the settings moved its host was written to the host of the
    same name: `list_all` now writes a listing only to a host still at the place it was listed
    for.
  - `RemoteSettings` is read at `init`: settings register through `inventory` when the store is
    made, and `initialize_workspace` runs after `recent_projects::init`, so the read cannot panic.
  - No entity is read while it updates: the rail's listeners defer through `groups::in_group`.
  - Clippy: `needless_pass_by_ref_mut` on `list_all`, which takes `&AsyncApp`.
- **Gate:** `script/gates.sh --diff` green on the tested tree (every gate PASS, the receipt
  written); it ran after the visual check, so one run covers the final tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/808-tmux-sessions-in-the-rail.sh` (`compositor sway`; the run's own
  tmux servers under `$XDG_RUNTIME_DIR/marley-e2e-808`, since a socket path under the run's work
  folder passes tmux's 108-byte limit). Runs 1 and 2 found the click points (the Default group
  grows a row per attach, so the rows under it move down); run 3 passed every check (a client on
  `build`, still one client after the second click, a `marley-` session on the remote server) and
  every shot was read:
  - `808-01-off` (REQ-006): Default and the repository, no TMUX heading.
  - `808-02-rail` (REQ-001, REQ-003, REQ-005): `TMUX · this machine 2 sessions` with `agent`
    (Claude Code's icon, `Claude Code · 1 window`) and `build` (terminal icon, `1 window`);
    `TMUX · box-2 1 session` with `deploy` (Codex's icon, `Codex · 1 window`); `TMUX · gone` with
    `ssh: connect to host gone p…` in red; `TMUX · nowhere` with `no saved host is named…` in red.
    Each header has its +.
  - `808-03-attached` (REQ-002): a tab `this machine · build` with the session's `hello from
    build` and tmux's status line; its row in Default. `build`'s row still says `1 window`: the
    next listing, up to 10 s later, adds `attached`.
  - `808-04-again` (REQ-002): the same single tab after the second click; `build` now reads
    `1 window · attached`.
  - `808-05-remote` (REQ-002): a tab `box-2 · deploy` with `Codex is working on box-2` and the
    remote server's status line `[deploy] 0:codex*`.
  - `808-06-new` (REQ-004): a tab `box-2 · marley-ed66f32b` with a shell; under `TMUX · box-2 2
    sessions`, `marley-ed66f32b` (`1 window · attached`), and `deploy` now `attached`; the +'s
    tooltip "New tmux session on box-2".
- Not reached: a real ssh link and a dropped one; #641's scenario covers the supervision that the
  `marley-remote-tmux` id base joins.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: "tmux sessions in the rail"); `docs/marley/guide.md`
  ("tmux sessions in the rail (#808)" after the remote terminals); `docs/marley/walkthrough.md`
  stop 4.29 and the header; `docs/marley_architecture/marley_workbench.md` (`open_words` under
  the remote terminals, and a section for `tmux_sessions.rs`); `docs/marley/zed-touchpoints.md`
  (the `crates/settings_content/src/marley.rs` row names `tmux_hosts`).
- **Knowledge:** AD-claude-808-the-rail-lists-each-named-hosts-own-tmux-sessions-001,
  L-claude-808-a-scenarios-tmux-socket-must-be-short-001,
  L-claude-808-zed-holds-a-second-run-of-a-running-task-001. No bug found in Code or Test beyond
  the review's stale-listing fix, recorded in Phase 2.
- **Brain:** `brain decide` on consultation `1f1dd7f7cdba4a5aa6a90aa7bb14daea`:
  `decisions/marley-lists-each-named-hosts-own-tmux-sessions-in-the-rail-by-polling`, follow-up
  by 2026-10-31.
