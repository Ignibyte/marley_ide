---
pipeline_id: 476fe8e4-27a8-45f5-9ea5-f8c83fc24914
ticket: docs/planning/tickets/open/TICKET-749-marley-commands.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: marley <command>
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/513-one-marley-per-data-dir.spec.md
  - docs/planning/pipeline/completed/545-activation-token-hand-off.spec.md
  - docs/planning/pipeline/completed/734-an-agent-thread-starts-in-a-tab.spec.md
  - docs/planning/pipeline/queued/747-side-panels-as-tabs.spec.md
  - docs/planning/pipeline/queued/748-more-side-panels-as-tabs.spec.md
---

## Title
Verbs on the `marley` command line that open things inside the running Marley, in the folder the
shell is in. Chad, 2026-10-11: "we should think about commands we can run like marley <command>
where it can open things inside marley. for example i could cd /whatever/folder and do marley
agent-thread --provider=claude and it would pop open a new agent thread", and of the list of verbs:
"yes to all of these". `marley` is the app itself, and a second launch already hands its paths to
the running Marley (#513); the verbs ride the same hand-off.

## Scope
### In
- The verbs, each run in a folder (the one named, else the shell's working directory):
  - `marley agent-thread [--provider=NAME] [--folder=PATH]`: an agent thread in a center tab,
    working in the folder;
  - `marley agent-cli [--provider=NAME]`: an agent CLI in a new center terminal in the folder;
  - `marley terminal [PATH]`: a center terminal in PATH, else the folder;
  - `marley talk marley|rusty`: #738's conversation with the Marley or the Rusty agent;
  - `marley threads`: #737's Threads page;
  - `marley home`: Home's page (#701);
  - `marley rusty [SCREEN]`: Rusty's home page, or one of its screens (`brain`, `tasks`,
    `memory`, `skills`, `decisions`, `graph`, `secrets`, `today`);
  - `marley panel NAME`: a side panel (`project`, `git`, `outline`, `collab`, `fleet`,
    `knowledge`, `containers`) as #747's tab, or `debug`, the bottom dock (#748);
  - `marley browser URL`: a Browser tab on URL, read as the address bar reads typed text;
  - `marley help`: the verbs, printed.
- Each verb lands in the group whose project holds the folder, else in Default; the window and
  the group come forward.
- `--provider` defaults to the agent last started that way: the last thread's agent for
  `agent-thread`, the last agent CLI for `agent-cli`.
- With no Marley running, the launch starts Marley, which restores its session and then runs the
  verb.
- `marley` typed in a Marley terminal reaches that Marley, also when it runs on a data directory
  of its own (`--user-data-dir`).
- The quick prompt (#721) folds in: a Hyprland key bound to `marley agent-thread` covers it. The
  guide shows the binding; no Hyprland config is written.
- `docs/marley/guide.md`: a `marley <command>` section, the Hyprland binding;
  `docs/marley/walkthrough.md`: a stop for it.

### Out (explicitly deferred)
- A `marley://` scheme: #445. The verbs' grammar names no scheme, so the URL prefix can change
  there.
- A reply from the running Marley (an exit code or JSON from the app's side): the hand-off is
  one-way; what only the app can judge shows there as a toast.
- Detaching a Marley that a verb starts from a shell: it runs as `marley PATH` does today (#513).
- A prompt line on the command (`--prompt`), remote hosts (`--host`), and verbs beyond this list.
- Starting an agent without bringing Marley forward (#721's first summary asked for it).

## Reference (§20)
N/A — Marley-specific: Zed's own command line opens paths and `zed://` links, and Marley ships no
Zed CLI binary (#513). Upstream Zed's behavior kept: `OpenRequest::parse` and
`handle_open_request` stay the one path every URL takes, launch or hand-off, and `zed://agent`
and every other link keep their meaning. Orca's CLI (`docs/orca_architecture/06-cli-automations-skills.md`
§2.1) is the nearest behavior map, read for its verb shape (`orca open`, `orca help <command>`), not
its transport.

### Prior art
- **The code we ship:**
  - `crates/zed/src/main.rs`: `run_browser_relay_if_invoked` runs before `Args::parse` (219-221),
    the precedent for a pre-parse intercept; `Args::parse` (223) would refuse `--provider` as an
    unknown flag and read `agent-thread` as a path; `paths_or_urls` (1722) and `parse_url_arg`
    pass a `zed://` URL through (1831-1847); a failed single-instance check hands the launch's
    paths to the running Marley (393-400); a cold start queues the launch's URLs (925-940), and
    the first request then runs **instead of** the session restore unless it only focuses the app
    (950-975); `handle_open_request` (1029), whose `zed://agent` arm finds or makes a window,
    awaits `take_panels_task()` and calls the Agent Panel directly (1061-1097);
    `restore_or_create_workspace` (1445).
  - `crates/zed/src/zed/open_listener.rs`: `OpenRequestKind` (48-82), `OpenRequest::parse`
    (135-215) with the `zed://agent` prefix at 177; the Linux datagram listener (409-438), one URL
    per datagram, the activation token caught first (425-430).
  - `crates/marley_workbench/src/single_instance.rs`: `hand_off` and `send`: one datagram per URL,
    1024 bytes at most, one-way, the token first; `url_for` passes a `zed://` URL as it is
    (87-153).
  - `crates/marley_workbench/src/thread_tab.rs`: `start(workspace, agent, folder)` (650-667,
    `pub(crate)`), the panel's selected agent when `agent` is `None`, `start_thread` checks the
    folder and joins it as a hidden worktree (672-742).
  - `crates/marley_workbench/src/agents.rs`: `Start::run_in` starts a thread or an agent CLI in a
    folder (1041-1066); `launch_input` and `start_in_terminal` (368-430), whose `launch_mode`
    keeps each CLI's own permission prompts unless the settings say otherwise (#532);
    `installed_clis` (110).
  - `crates/marley_workbench/src/assistant.rs`: the registry adapter of Claude Code and Codex,
    `claude-acp` and `codex-acp` (362-368); `talk_to` with `Entry::Marley` and `Entry::Rusty`
    (281-292), `talk_to_marley` (1003); the entries' ids `Marley` and `Rusty` (184-188).
  - Openers: `routing.rs` `open_center_terminal` (334); `threads_page::open` (47);
    `home_page::open_here` (64); `rusty/home_tab.rs` `open_here` (118) and the screens'
    `open_later` (`brain_tab.rs` 28, `tasks_tab.rs` 74, `memory_tab.rs` 53, `skills_tab.rs` 59,
    `decisions_tab.rs` 52, `graph_tab.rs` 119, `secrets_tab.rs` 56), `capture.rs` `OpenToday`;
    `browser::open_url_tab` (8338); `marley_browser::address::url_for` and `local_page` (35-90);
    #747's and #748's panel opener.
  - `crates/marley_workbench/src/groups.rs`: `with_group(GroupKind::Home, …)` makes Default when a
    window has none (476-522); `in_group` and `in_shown` (524-581).
  - `marley_terminal::identity` (#520): the variables Marley's local terminals carry, and
    `BIN_VARIABLE` emptied so a Marley started from a Marley terminal does not inherit its parent's.
- **Behavior maps:** `docs/orca_architecture/06-cli-automations-skills.md` §2.1: Orca's CLI finds the
  running app through a file in its data directory and talks over a socket with replies; its verbs
  are `noun verb` with `--help` per command.
- **Published material:** the Hyprland wiki's `bind` and `exec` dispatchers, for the guide's binding.

## UI proof
Scenario `script/e2e/749-marley-commands.sh`, under `compositor sway`. The repo has a `sub/`
folder; another folder no project holds sits beside it. A scripted ACP agent (#734's) joins the
profile's `agent_servers` as `Stand-in` and is also the Marley entry's adapter
(`MARLEY_ASSISTANT_ADAPTER`); a fake `claude` first on the terminals' PATH prints its working
directory; Rusty is on through its stand-in; `$E2E_WORK/bin/marley` is the debug binary, first on
the terminals' PATH. Commands are typed into a terminal of the run's Marley.

Shots:
- `help`: `marley help` in the repo's terminal: the verbs (REQ-001).
- `error`: `marley agent-cli --provider=nope; echo "exit $?"`: the error and `exit 2` (REQ-002).
- `thread`: `cd sub && marley agent-thread --provider=Stand-in`: a Stand-in thread tab in the repo
  group; the agent's log shows `session/new` in `sub` (REQ-003, REQ-004).
- `short-name`: `marley agent-thread --provider=marley`: a thread of the Marley entry (REQ-005).
- `default-provider`: `marley agent-thread`: a thread of the Marley entry, the agent last started
  (REQ-006).
- `default-group`: `marley agent-thread --folder=<other>`: Default shown, the thread in `<other>`
  (REQ-007).
- `unknown-agent`: `marley agent-thread --provider=nobody`: a toast in the repo naming the
  provider (REQ-008).
- `cli`: `marley agent-cli --provider=claude`: a center terminal in the repo running the fake
  `claude`, which prints `sub` (REQ-009).
- `terminal`: `marley terminal ..`: a center terminal at the repo's root (REQ-010).
- `talk`, `threads`, `home`, `rusty`, `panel`, `browser`: `marley talk marley`, `marley threads`,
  `marley home`, `marley rusty brain`, `marley panel git`, `marley browser page.html`: each one's
  tab in the repo group (REQ-011).
- `cold`: Marley quit; `marley agent-thread --provider=Stand-in` launched from `sub`: Marley starts
  with the repo group restored and a Stand-in thread tab in it (REQ-012).

## Locked-In Decisions
- **D1 — The verbs ride #513's hand-off as `zed://marley/<verb>?<query>` URLs.** A Marley module
  (`commands.rs`) owns the grammar: argv to a `Command`, a `Command` to the URL's path and query and
  back. It never names the scheme; the prefix is one constant beside it, and #445 may add
  `marley://`. Rejected: Zed's `cli` crate and its IPC (Marley ships no CLI binary, #513); a
  socket with replies, as Orca's (a second channel for little gain: the app shows its own
  errors).
- **D2 — The intercept runs in `main` before `Args::parse`**, as the browser relay's does. A first
  argument (after an optional `--user-data-dir DIR`) that names a verb is a verb, so a path named
  like one opens as `./agent-thread`. The intercept answers one of three: not a verb (Zed's parse
  as now), exit with a code (`help`, an argument error), or the argv to launch with: the same
  binary, `--user-data-dir` kept, and the one URL.
- **D3 — What the launch can judge, it judges.** `help` prints the verbs and exits 0. An unknown
  flag, a missing or extra argument, a name outside a fixed list (`talk`, `rusty`, `panel`,
  `agent-cli --provider`), a folder that does not exist, or a URL over the listener's 1024 bytes
  prints one line and exits 2. What only the app knows (an agent name that matches no agent, AI
  disabled, Rusty off, the Marley entry absent, a panel disabled) shows as a toast in the target
  group. A handed verb prints what was asked, beside #513's line, and exits 0.
- **D4 — The folder is absolute.** `--folder`, `terminal PATH`, or the working directory,
  canonicalized by the launch, since the running Marley's working directory is another.
- **D5 — The target group.** Across Marley's windows, the group whose project has a visible root
  holding the folder, the deepest root first; else Default of the active Marley window, made when
  missing (`groups::with_group(GroupKind::Home, …)`); in the Zed layout, the workspace holding the
  folder, else the window's shown one. The window comes forward (with #545's token) and the group
  shows. Rejected: always the shown group (Chad's example opens the thread in the folder's
  project).
- **D6 — Providers.** For `agent-thread`, short names first: `claude` (Claude Code, `claude-acp`),
  `codex` (`codex-acp`), `zed` (Zed's own agent), `marley` and `rusty` (the assistant's entries),
  mapped as `assistant.rs` 362-368 maps Claude Code and Codex; any other name is an agent server's
  id as `agent_servers` and the registry name it. For `agent-cli`: `claude`, `codex`, `gemini`,
  `opencode` (`marley_agent::AgentKind`). The default is the agent last started in a tab or a
  CLI, kept per kind in Zed's key-value store by `thread_tab::start_thread` and
  `agents::start_in_terminal`; with none kept, the Agent Panel's selected agent, and the first
  installed CLI. `--provider=NAME` and `--provider NAME` are both read.
- **D7 — Direct calls in the app, never a dispatched action**, which reaches whatever the window
  shows: `thread_tab::start` once the workspace's `take_panels_task()` has finished (as the
  `zed://agent` arm waits), `agents` start in the folder (made `pub(crate)` as needed),
  `routing::open_center_terminal`, `assistant::talk_to`, `threads_page::open`,
  `home_page::open_here`, Rusty's openers, the panel opener, `browser::open_url_tab`.
- **D8 — A cold start restores, then runs the verb.** Zed runs a first request in place of the
  session restore (`main.rs` 950-975); a Marley verb restores first, as a plain `marley` does,
  then runs. Rejected: Zed's way, where `marley agent-thread` with no Marley running would open a
  lone window and leave the session's windows closed.
- **D9 — `marley` in a Marley terminal reaches that Marley.** Marley's local terminals carry
  `MARLEY_USER_DATA_DIR` while Marley runs with `--user-data-dir` (empty otherwise, as
  `MARLEY_BIN` is emptied for a nested Marley), and a verb with no `--user-data-dir` takes it
  (PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001). The launch notes its
  own `--user-data-dir` at the top of `main`, as it keeps the activation token. Never the default
  directory for a custom one: `--user-data-dir` also moves the config directory (`paths.rs`
  104-128).
- **D10 — Agent CLIs keep their own permission prompts**: `agent-cli` starts through the same
  `launch_mode` as the + menu (#532).
- **D11 — #721 folds in.** The guide shows a binding such as
  `bind = SUPER SHIFT, A, exec, marley agent-thread`; Hyprland runs it from the home folder, so the
  thread lands in Default unless `--folder` names a project. The new thread brings Marley forward.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley help` runs, the launch shall print every verb with its arguments and exit 0 without reaching Marley. | shot `help` |
| REQ-002 | IF a verb's arguments are wrong (an unknown flag, a missing or extra argument, a name outside its list, a missing folder, a URL too long), THEN the launch shall print the error and exit 2 without reaching Marley. | shot `error` |
| REQ-003 | WHEN `marley agent-thread` runs in a folder a group's project holds, the system shall start a thread in a center tab of that group, with the group shown. | shot `thread` |
| REQ-004 | WHEN a verb runs without `--folder` or PATH, the system shall work in the launch's working directory. | shot `thread` (the agent's `session/new`) |
| REQ-005 | WHEN `--provider` names a short name (`claude`, `codex`, `zed`, `marley`, `rusty`) or an agent server's id, the system shall start that agent. | shots `thread`, `short-name` |
| REQ-006 | WHEN `--provider` is absent, the system shall start the agent last started that way. | shot `default-provider` |
| REQ-007 | WHEN no group's project holds the folder, the system shall run the verb in the window's Default group and show it. | shot `default-group` |
| REQ-008 | IF the app cannot run a verb (an unknown agent, AI disabled, Rusty off, a disabled panel), THEN the system shall show a toast in the target group that names why. | shot `unknown-agent` |
| REQ-009 | WHEN `marley agent-cli` runs, the system shall start the agent CLI in a new center terminal in the folder, with the permission mode its settings give. | shot `cli` |
| REQ-010 | WHEN `marley terminal [PATH]` runs, the system shall open a center terminal in PATH, else the folder. | shot `terminal` |
| REQ-011 | WHEN `marley talk`, `threads`, `home`, `rusty`, `panel` or `browser` runs, the system shall open its tab, or bring it forward, in the target group. | shots `talk`, `threads`, `home`, `rusty`, `panel`, `browser` |
| REQ-012 | WHEN a verb runs with no Marley running, the system shall start Marley, restore its session, then run the verb. | shot `cold` |
| REQ-013 | WHEN `marley` runs in a terminal of a Marley started with `--user-data-dir`, the launch shall reach that Marley. | every shot from `thread` on (the run's Marley, not the user's) and the runner's focus report |
| REQ-014 | The change shall pass `script/gates.sh --diff`. | the gate's exit code |

## Phase Plan
- **P1 Plan** — this spec; at promotion the design in the notes, every cited seam checked again,
  #747's and #748's openers as they shipped.
- **P2 Code** — README marker present (§3); the ledger rows for `crates/zed/src/main.rs` and
  `crates/zed/src/zed/open_listener.rs` extended first (`enforce-zed-ledger.sh`); `commands.rs`
  (new) in `marley_workbench`, `single_instance.rs`, `thread_tab.rs`, `agents.rs`, `routing.rs`,
  `assistant.rs`, `rusty.rs`, `marley_terminal::identity`; the Zed hunks; the guide and the
  walkthrough; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/749-marley-commands.sh`, read every
  shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket;
  #721 noted as folded in; archive, commit.
