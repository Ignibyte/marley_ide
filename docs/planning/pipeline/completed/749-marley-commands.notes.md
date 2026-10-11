# marley <command> — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-749-marley-commands.md
- **Pipeline spec:** 749-marley-commands.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11: "we should think about commands we can run like marley <command>
  where it can open things inside marley. for example i could cd /whatever/folder and do marley
  agent-thread --provider=claude and it would pop open a new agent thread"; offered the list of
  verbs, "yes to all of these". The quick prompt (#721, "A `marley` verb Hyprland binds to a key")
  folds in. Shelf: `docs/planning/design-notes/tabs-everywhere-2026-10-11.md`.
- **Classification / tier:** feature; Marley crates (`marley_workbench`, `marley_terminal`) and
  small hunks in `crates/zed` (`main.rs`, `zed/open_listener.rs`), both already in the ledger.
  Size L: ten verbs, the cold start, the group routing. `panel` needs #747 and #748 shipped.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - #513 (AD and F-claude-513): one Marley per data directory; a second launch hands its paths as
    URLs over the data directory's datagram socket; a socket path over 108 bytes fails the bind, so
    the runner's profiles sit under `$XDG_RUNTIME_DIR/marley-e2e/`
    (PR-claude-check-a-unix-socket-path-against-sun-path-001).
  - #545: the launch's activation token travels first, so the window the request brings forward
    comes as the launcher's click; a verb gets this for free.
  - #583: the browser relay's pre-parse intercept in `main`, the shape D2 copies.
  - PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001 (from F-claude-561): a
    program run inside Marley must reach the Marley that runs it, never the default data
    directory, or an e2e run drives the user's own window. D9 applies it to `marley` typed in a
    Marley terminal.
  - #734 and #735: `thread_tab::start` and the New Agent picker's `Start::run_in` already start a
    thread or a CLI in any folder; #738 `talk_to`; #739 the openers' `open_here` shape.
  - #736: a thread tab's restore waits for the Agent Panel; the cold start here waits the same way
    (`take_panels_task`, then the panel).
  - Orca's CLI map (`docs/orca_architecture/06-cli-automations-skills.md` §2.1-2.2): 238 commands,
    a two-way runtime socket; read for the verb shape only.
- **Discovery** (the spec's Prior art has every line):
  - The ticket's premise that "a cold start handles URLs after the session restore" does not hold:
    `main.rs` 950-975 takes the first queued request and, unless it only focuses the app, runs
    `handle_open_request` in place of `restore_or_create_workspace`. `zed://agent` with no Marley
    running opens one empty window and no session. D8 restores first for a Marley verb.
  - `Args::parse` refuses `--provider=claude` (no such flag) and reads `agent-thread` as a path
    (`file://<cwd>/agent-thread`), so the intercept must come before it, not after.
  - `--user-data-dir` moves the config directory under the data directory (`paths.rs` 104-128), so
    a terminal must not pass the default directory as a custom one (D9: the variable is set only
    for a custom one).
  - The Agent Panel's selected agent changes only through the panel (`agent_panel.rs` 1420, 1933,
    1937); threads Marley starts in tabs do not touch it, so "the agent last started" needs its own
    record (D6).
  - `routing::open_center_terminal` is private; `assistant::talk_to` is private beside
    `talk_to_marley`; several Rusty `open_later`s are `pub(super)`. Each widens to `pub(crate)`.

### Design
- **`crates/marley_workbench/src/commands.rs`** (new, Marley crate):
  - `Command` (one variant per verb) and `Provider` (a short name or an id); `parse(argv) ->
    Result<Command, Usage>`; `to_path_and_query(&Command) -> String` and
    `from_path_and_query(&str) -> Result<Command, String>`, no scheme in either; `HELP`, the verbs'
    text; `URL_PREFIX = "zed://marley/"`.
  - `args_if_invoked() -> Invoked { NotACommand, Exit(i32), Launch(Vec<String>) }`, called in
    `main` before `Args::parse`: reads `std::env::args_os`, an optional leading
    `--user-data-dir DIR` (else `MARLEY_USER_DATA_DIR` when set and non-empty), the verb, its
    arguments; resolves the folder (`std::fs::canonicalize` against the working directory) and a
    `browser` text (`marley_browser::address::url_for`, `local_page` against the folder); checks
    the 1024 bytes; prints `HELP` or the error.
  - `command_in(url) -> Option<String>`: the rest after `URL_PREFIX`, for `OpenRequest::parse`.
  - `run(command: &str, app_state, cx)`: parses; finds the target window and group (D5), makes
    Default when needed, activates them; for `agent-thread` awaits the workspace's
    `take_panels_task()`, then waits for the Agent Panel as the thread tab's restore does; runs the
    verb's opener; a refusal is a toast in the target workspace.
  - `remember(kind, agent, cx)` and `last(kind, cx)`: the last thread agent and the last CLI in
    `db::kvp::KeyValueStore` under a `marley-commands` scope.
- **Marley crate edits:** `single_instance.rs` (the hand-off line names the verb); `thread_tab.rs`
  (`start_thread` calls `commands::remember`); `agents.rs` (`start_in_terminal` with an agent calls
  `remember`; a `pub(crate)` start of a CLI in a folder from `Start::run_in`'s arm);
  `routing.rs` (`open_center_terminal` `pub(crate)`); `assistant.rs` (`talk_to` or a
  `talk_to_rusty` `pub(crate)`); `rusty.rs` (one `pub(crate) open_screen(name, …)` over the
  screens' openers); `marley_workbench.rs` (`mod commands`, `pub use`);
  `marley_terminal/src/identity.rs` (`MARLEY_USER_DATA_DIR`, set from the launch's noted
  `--user-data-dir`, emptied otherwise).
- **Zed hunks** (`// Marley:` each; the existing rows extended first):
  - `crates/zed/src/main.rs`: before `Args::parse` (223), `marley_workbench::commands::args_if_invoked()`
    → exit with its code, or `Args::parse_from` its argv, or `Args::parse` as now; in the first
    request's match (950-975), a Marley request restores, then runs; in `handle_open_request`
    (1029), the `MarleyCommand` arm calls `marley_workbench::commands::run`. About 20 lines.
  - `crates/zed/src/zed/open_listener.rs`: `OpenRequestKind::MarleyCommand { command: String }`
    with its `Debug` arm, and in `parse` a `commands::command_in` check before the `zed://agent`
    prefix (177). About 10 lines.
  - Ledger: `docs/marley/zed-touchpoints.md`'s `crates/zed/src/main.rs` row gains the intercept,
    the restore-first arm and the `MarleyCommand` arm (#749), why (Marley ships no CLI; clap
    refuses the verbs' flags; a cold verb must not lose the session) and the merge note (keep the
    intercept ahead of `Args::parse`, the arm ahead of the generic `Some(request)`); the
    `open_listener.rs` row gains the variant and the prefix check (keep it ahead of `zed://agent`).
- **Docs:** `docs/marley/guide.md` (a `marley <command>` section: verbs, folder and group rules,
  the provider names and default, the Hyprland binding), `docs/marley/walkthrough.md` (a stop),
  `docs/marley_architecture/marley_workbench.md` (Complete).
- **Scenario:** `script/e2e/749-marley-commands.sh` (new).

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Setup per the spec; in the repo's center terminal types `marley help` | `help` |
| 002 | Types `marley agent-cli --provider=nope; echo "exit $?"` | `error` |
| 003, 004, 005 | Types `cd sub && marley agent-thread --provider=Stand-in`; the check greps the agent log for `session/new` with `sub` | `thread` |
| 005 | Types `marley agent-thread --provider=marley` | `short-name` |
| 006 | Types `marley agent-thread` | `default-provider` |
| 007 | Types `marley agent-thread --folder=<other>` | `default-group` |
| 008 | Back in the repo, types `marley agent-thread --provider=nobody` | `unknown-agent` |
| 009 | Types `marley agent-cli --provider=claude` | `cli` |
| 010 | Types `marley terminal ..` | `terminal` |
| 011 | Types `marley talk marley`, `marley threads`, `marley home`, `marley rusty brain`, `marley panel git`, `marley browser page.html` one at a time | `talk`, `threads`, `home`, `rusty`, `panel`, `browser` |
| 012 | Quits Marley; launches the debug binary with `--user-data-dir <profile> agent-thread --provider=Stand-in` from `sub` (513's `second_launch` shape) | `cold` |
| 013 | All the typed commands above reach the run's Marley; the runner's focus report shows the user's Marley untouched | the run's log and focus report |
- Not reachable: `--provider=claude` for a thread starts the real Claude adapter, which reaches
  outside; its mapping is read in the diff, and `agent-cli --provider=claude` runs the fake.

### Risks
- A `zed://marley/…` URL handed to an older Marley (one without this ticket) is logged as an
  unhandled URL and nothing opens; the launch has already printed that it handed it. Accepted:
  one data directory runs one build.
- `take_panels_task` is taken once: a second verb for the same new group finds `None` and must
  still wait for the Agent Panel; `run` waits on the panel itself after the task.
- The cold start's restore may bring back no group holding the folder (the session had none); the
  verb then lands in Default, as D5 says.
- A folder inside two projects' roots (nested repositories) picks the deepest root; two windows
  holding the same root pick the active one.
- The scenario types into a terminal with `marley` first on PATH. If the PATH order slips, or
  `MARLEY_USER_DATA_DIR` is missing, a verb would reach the user's own Marley on the default data
  directory. Before the first verb the scenario has the terminal print `command -v marley` and
  `$MARLEY_USER_DATA_DIR`, and stops unless they name the run's binary and the run's profile.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress.
- **Seams checked again** at `cbbf8efdd4` (an Explore pass over all 14, plus `main.rs` and
  `open_listener.rs` read directly). `main.rs`: the relay intercept at 219, `Args::parse` at 223,
  the hand-off at 393-402, the cold start's first-request match at 950-975, `handle_open_request`
  at 1029. `open_listener.rs`: `OpenRequestKind` at 49, `parse` at 135, `zed://agent` at 177.
  `single_instance`: `hand_off` returns its line and `main` prints it; 1024 bytes; token first.
- **What changed since the draft, and where the design takes it:**
  - `threads_page` has `open` (112), no `open_here`.
  - No panel opener by name exists (`panel_tabs`' `show_tab` and `workspace_panel` are private).
    The verb opens a panel as `focus_panel` does, by name through its dock (activate, open, focus),
    and #747's observer makes the tab in the Marley layout; Debug stays in the bottom dock (D7).
  - `thread_tab::start_thread` is also `assistant::talk_to`'s path, so the last agent is kept in
    `thread_tab::start` (the picker, the Manager), in `Start::run_in`'s CLI arm and by the verbs
    (D6).
  - `talk_to` and `Entry` are private: one `pub(crate)` wrapper.
  - Rusty's screens already have a dispatcher, `brain::open_screen(Screen, multi_workspace, …)`,
    which opens in the shown group: the verb shows the target group first, then calls it;
    `rusty::open_home_here` for no screen.
  - `routing::open_center_terminal` is private; `local` means a local terminal in a remote
    project, which ignores `directory`. The verb widens it to `pub(crate)` and passes
    `local: false` with the folder, as the project's own New Terminal does.
  - Marley keeps no note of its own `--user-data-dir` (Zed's `restart_arguments` only): D9 now
    says where it is noted.
  - `MultiWorkspace::activate(workspace, None, window, cx)` shows a group.
- **Recall** stands as drafted; the brain consulted again at Complete.

## Phase 2 — Code
- **Built:**
  - `crates/marley_workbench/src/commands.rs` (new): the grammar (`Command`, `Verb`, `Typed`), the
    launch's read (`args_if_invoked` → `Invoked::{NotACommand, Exit { code, message }, Launch}`;
    `read_launch` takes a leading `--user-data-dir`, else `MARLEY_USER_DATA_DIR`; `help` and
    `--help`; folders canonicalized against the working directory; `browser` resolved with
    `marley_browser::address::local_page`, else `url_for`; the 1024-byte check), the URL both ways
    (`to_url`, `from_path_and_query`, `form_urlencoded`), `command_in`, `verb_in`, `run` (the
    target per D5 through `group_holding`, else `get_any_active_multi_workspace` and Default by
    `groups::with_group`, or Zed's shown workspace; `MultiWorkspace::activate` and
    `activate_window`; a wait of up to 10 s for the Agent Panel or the named panel; then the verb),
    `thread_agent` (short names, an id or a display name from `agents::thread_agents`, else the
    registry; Marley's entries by name), `open_panel` (as `focus_panel`, by persistent name), the
    toasts, `remember_thread_agent` / `remember_cli` / `last` over `KeyValueStore`
    (`marley-commands`), `note_data_dir`.
  - `assistant.rs`: `thread_agent_of` out of `talk_to`; `talk_by_name(name, folder, …) -> bool`.
  - `agents.rs`: `start_cli_in(kind, folder, …)` out of `Start::run_in`'s CLI arm; it and
    `start_cli_with_prompt` (Home's and the rail's CLI starts) call `remember_cli`.
  - `thread_tab::start` calls `remember_thread_agent`; `routing::open_center_terminal` is
    `pub(crate)`; `single_instance::hand_off` names a handed verb ("it was asked to run `marley
    agent-thread`"); `marley_workbench.rs`: `pub mod commands`, `pub use commands::{Invoked,
    args_if_invoked}`, `commands::note_data_dir()` in `init`.
  - `marley_terminal::identity`: `DATA_DIR_VARIABLE` (`MARLEY_USER_DATA_DIR`), `set_data_dir`,
    and `agent_environment` writes it (empty unless the terminal is named and Marley has a data
    directory of its own).
  - Zed hunks, rows extended first: `main.rs` (the intercept after the relay's, with `Exit`
    printed there; the restore-first arm; the `MarleyCommand` arm), `open_listener.rs` (the
    variant, its `Debug` arm, `is_marley_command`, the prefix check ahead of `zed://agent`).
- **Deviations:**
  - The launch's messages print in `main`, not in the Marley module: clippy's `print_stdout` and
    `print_stderr` are errors in the Marley crates, and `main` already prints #513's hand-off
    line. `Invoked::Exit` carries the code and the message.
  - The existing `agents::start_cli` (Home's and the rail's) kept its name; the folder-taking one
    is `start_cli_in`.
  - `start_cli_with_prompt` also keeps its CLI, so "the CLI started last" counts Home's and the
    rail's starts too.
- **Review of the diff:**
  - Re-entrancy: `run` updates through `cx.update_window`, which leases no entity, so
    `with_group`, `MultiWorkspace::activate` and the openers each update their own entity; the
    Rusty screen dispatcher runs outside any workspace update; `open_panel` updates the dock
    inside the workspace's update, as `focus_or_unfocus_panel` does, and #747's observer runs at
    the flush after it.
  - A command with no folder its project holds and no window at all: `get_any_active_multi_workspace`
    makes one.
  - A wait that never ends: the verb runs after 10 s anyway and its opener answers (the thread's
    "This window has no Agent Panel." toast, the panel's "This window has no … panel").
  - A path argument that names a verb (`marley home`) is the verb (D2); `./home` opens the folder.
  - Provenance: Zed's code read for its seams only; nothing copied.
- **Gate:** the first run red at gate:14 (rustdoc, fixed in Test); green over the final tree, see Phase 3.

## Phase 3 — Test
- **Scenario:** `script/e2e/749-marley-commands.sh` (`compositor sway`); `script/e2e.sh` gains
  `launch_marley_with <arguments>` (which `launch_marley` now calls with its path) and
  `marley_binary`. Five runs; the last ran through with every check passing.
- **Reds and their fixes:**
  - The gate's rustdoc (gate:14): Chad's quote in the module doc had a bare `<command>`, read as an
    HTML tag. The phrase went into a code span; the words are his.
  - The first `just build`: `WindowHandle::root` is `test-support` only, which clippy's
    `--all-targets` turns on through the dev-dependencies, so clippy passed and the build failed.
    `window.update(cx, |_, _, cx| cx.entity())` instead.
  - Scenario only: Default's new rows moved the repo terminal's row, so a click typed the next
    command into the Default group's thread (the scenario folds Default after its shot); `talk
    marley` reopened a conversation from the profile copy, which the scripted agent could not load
    (the copy's thread list is emptied, L-claude-738, and the agent answers `session/load`); the
    cold start from `sub/` panicked, "dev asset loading requires running from within the
    checkout": a debug build reads its assets from the checkout found from the working directory,
    so the cold launch starts there and names `--folder` (the installed build embeds them).
- **Shots** (the last run):
  - `749-00-env` (REQ-013): `command -v marley` and `$MARLEY_USER_DATA_DIR` in the repo's terminal
    name `<work>/bin/marley` and the run's profile; `check …: pass`. Every verb after reached the
    run's Marley; the runner's report: the user's Hyprland Marley untouched (1 window before, 1
    after).
  - `749-01-help` (REQ-001): the verbs and the paragraph below them, printed in the terminal.
  - `749-02-error` (REQ-002): `marley: "nope" is not an agent CLI: claude, codex, gemini,
    opencode`, `Run \`marley help\``, `exit 2`.
  - `749-03-thread` (REQ-003, 004, 005): a Stand-in thread tab ("Message Stand-in") in the repo
    group; `sessions in …/repo/sub: 1`.
  - `749-04-short-name` (REQ-005): `--provider=marley`: a Marley thread ("Message Marley"); 2 in sub.
  - `749-05-default-provider` (REQ-006): no provider: Marley again, the agent started last; 3 in sub.
  - `749-06-default-group` (REQ-007): `--folder=<other>`: the Default group shown with Home and a
    Stand-in thread; `sessions in …/other: 1`.
  - `749-07-unknown-agent` (REQ-008): the toast "marley: No agent is named "nobody": try claude,
    codex, zed, marley, rusty or an agent server's id"; the terminal shows each hand-off's line
    "Marley is already running on …; it was asked to run \`marley agent-thread\`."
  - `749-08-cli` (REQ-009): a new terminal "sub — sleep 600" ran `claude`, which printed `fake
    claude in …/repo/sub`.
  - `749-09-terminal` (REQ-010): `terminal ..`: a new "repo — bash" terminal at the repo's root.
  - `749-10-talk` … `749-15-browser` (REQ-011): a Marley thread; the Threads page; the Home page;
    Rusty's Brain tab listing `start`; the Git Panel tab; a Browser tab "Sub page" on
    `file://…/repo/sub/page.html`. Each in the repo group.
  - `749-16-cold` (REQ-012): after the quit, the cold launch: the repo group restored (its
    terminals, Sub page, Git Panel, threads) and a new Stand-in thread in front; `sessions in
    …/repo/sub: 4`.
- **Seen, not in scope:** a second launch prints Zed's "starting zed version" log line and "zed is
  already running" before exiting, beside #513's line (pre-existing, #513); ticketed as #775, a
  quiet handed verb.
- **Gate with the scenario in the tree:** `just gate-diff` GREEN, 17 passed (after the rustdoc fix and the build fix; one run covers the code and the scenario).

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: `marley <command>`); `docs/marley/guide.md` (a `marley
  <command>` section with the commands, where they open, `--provider`, errors, a cold start, a
  Marley on its own data directory, the Hyprland binding for #721); `docs/marley/walkthrough.md`
  (stop 1.7); `docs/marley_architecture/marley_workbench.md` (a #749 section);
  `docs/marley/workbench-shell.md` (the slice line); `docs/marley/zed-touchpoints.md` (the
  `main.rs` and `open_listener.rs` rows, written before the hunks and brought up to what shipped).
- **Ledgers:** F-claude-749-a-test-support-only-api-passed-clippy-and-failed-the-build-001,
  PR-claude-749-build-the-binary-before-trusting-clippy-on-a-new-gpui-call-001,
  L-claude-749-a-debug-build-starts-only-from-the-checkout-001,
  L-claude-749-a-new-group-row-moves-the-rail-rows-a-scenario-clicks-001,
  AD-claude-749-marley-verbs-ride-the-single-instance-hand-off-as-urls-001.
- **Brain:** consultation fb8b2ba8…, `brain decide`:
  decisions/marley-command-verbs-ride-the-single-instance-hand-off-as-zedmarley-urls.
- **Follow-up:** TICKET-775 (a quiet handed verb), queued under #774's place. #721 was already
  closed as folded into this ticket.
- Ticket closed, pipeline archived, committed.
