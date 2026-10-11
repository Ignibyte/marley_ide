# marley <command> — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-749-marley-commands.md
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

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
