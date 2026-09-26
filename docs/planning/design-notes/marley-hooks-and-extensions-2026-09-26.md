# Hooks and extensions for Marley, 2026-09-26

Long-range research, not for the current queue: no ticket, and nothing here changes the backlog.

Chad asked on 2026-09-26: "We should have event subscriber / hook systems throughout the system
where things make sense so lets research. Allowing the user to build and extend the system if they
want to in a way that maybe doesnt require recompiling or offer a way to extend crates maybe. this
is way down the road so not priority but i want to see where it makes sense." One agent read the
Marley crates for the events they already produce and the Zed events they sit on, Zed's extension
crates in this tree, Claude Code's hooks, MCP and channels references (code.claude.com), the MCP
specification's 2025-06-18 and 2026-07-28 revisions, the Orca survey and Orca's MIT source at
`/srv/stacks/orca-refs/orca`, Warp's published docs (docs.warp.dev only, no Warp source,
CONSTITUTION §20), and the documentation of mlua, Rhai and wasmtime. Nothing was built or run.

Marley already produces most of the events a hook system needs. Claude Code's hook events reach
the rail in-band through Marley's plugin, the shell integration opens and closes blocks, the
Browser tab's hub reports pages, dialogs and picks, and programs raise OSC 9 and 777
notifications. Each consumer reads its own source, though, and none of it reaches a user who wants
to add behavior. The proposal has three parts: one typed, versioned event stream inside Marley
(S); events going out through user hooks declared as Zed tasks, the mechanism Zed already uses for
its `create_worktree` hook (M), and through a feed on Marley's MCP server for agents and outside
tools (S to M); and actions coming back in through Marley's MCP tools, whose grant table already
decides what a caller may do, with a thin `marley call` CLI for scripts. Hooks observe. A short
list of events that come before an action may refuse it, and nothing a hook returns approves
anything. A hook from a project's own files runs only in a worktree Zed trusts, after Chad
approves its exact text. Lua or Rhai scripting and a WebAssembly extension API of Marley's own wait
until shell hooks fall short or Marley has users besides Chad, and Zed's extension API stays as
upstream ships it.

## What Marley produces today

The plugin's `hooks/hooks.json` runs `hooks/event.py` on twelve Claude Code events (SessionStart,
UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Stop,
StopFailure, PostCompact, SubagentStart, SubagentStop, SessionEnd). The script answers each with a
`terminalSequence`: an OSC 777 notify titled `marley-event` whose body is the base64 of a JSON
summary under 2,900 bytes. Claude Code writes it into its own terminal, so the event travels
in-band, with no socket. `notifications.rs` hands that title to `agent_events::on_frame`, which
checks that Claude Code is the terminal's foreground program, decodes the frame
(`claude_events::decode` in `crates/marley_agent/src/claude_events.rs`) and folds it
(`claude_events::fold`) into `marley_fleet::SessionEvent`s on a seat per terminal view. The
`AgentEvents` global holds the snapshot. The rail observes it, and `mcp.rs` publishes it as
`fleet_snapshot` and the `fleet://snapshot` resource (#547). An observer learns that something
changed and reads the whole snapshot, and nothing hands it the transition. The phone push, active
in the pipeline as this is written, has `on_frame` report each seat's state before and after a
frame for its new `push.rs` (#535's spec), and #538's banners want the same changes: that report
is the seed of the stream proposed below.

The shell integration (`crates/marley_terminal/shell_integration/marley.bash` and `marley.zsh`)
prints DCS frames for `init`, `precmd` (exit code, cwd), `preexec` (the command and the terminal's
nonce), `history` and `bootstrapped`. The vendored alacritty event loop reports them as
`Event::ShellHook`, and `Terminal::apply_shell_hook` (`crates/terminal/src/terminal.rs`) applies
each to the anchored blocks, stamps the times and calls `cx.notify()`. No event says that a block
started or finished; the terminal element and the MCP tools read `blocks()` when they need to.
Other programs' OSC 9 and 777 arrive as `terminal::Event::MarleyNotification` (#478). The Browser
tab's `BrowserHub` emits `BrowserEvent` (`PageOpened`, `PageClosed`, `PageInfoChanged`,
`DialogOpened`, `DialogClosed`, `SelectOpened`, `PickStaged`), and `marley_browser::observe` keeps
each page's console messages and requests in rings of 200.

Marley's MCP server (`crates/marley_mcp`) speaks the 2025-06-18 revision, advertises
`tools.listChanged` and `resources.subscribe`, and serves one resource, `fleet://snapshot`, whose
changes it pushes as `notifications/resources/updated` on a held GET stream; opening the stream is
the subscription (`serve_sse_stream` in `transport.rs`). The plugin's stdio bridge
(`claude_plugin/marley/bin/marley-mcp-bridge`) forwards requests and answers, and sends a
`tools/list_changed` of its own when Marley starts or quits. It never opens the stream, so neither
Claude Code nor Zed's agents see a resource update.

The rail is the largest subscriber to Zed's own events: `MultiWorkspaceEvent`,
`workspace::Event`, `project::Event` for folder changes, `terminal::Event` (`Wakeup` for output
quiet, the rest for a refresh), `ItemEvent`, `AgentPanelEvent`, `AcpThreadEvent`,
`AgentServersUpdated` and the thread metadata store (`sync_subscriptions` in `rail.rs`). Zed
itself has one user-facing event hook: a task in `tasks.json` with `"hooks": ["create_worktree"]`
runs after Zed creates a linked worktree (`TaskHook` in `crates/task/src/task_template.rs`,
`run_create_worktree_tasks` in `crates/workspace/src/tasks.rs`, `docs/src/tasks.md`).

## The event map

"Observe" means a hook learns of the event and acts on its own or through Marley's tools; "refuse"
means its answer can stop the action. Some events are observe-only by nature: the shell reports
`preexec` as it runs the command, and Marley sees an agent's PreToolUse after Claude Code's own
hooks have decided it. Events name a terminal by #520's `MARLEY_TERMINAL_ID` once that lands, and
until then by the view's entity id, which changes at each launch.

| Event | Where it comes from today | Who would subscribe | A hook may |
|---|---|---|---|
| `block.started` | the `preexec` frame in `Terminal::apply_shell_hook`, which only calls `cx.notify()` | a time log; the System One note's watch on a running command (use 7) | observe |
| `block.finished` | the `precmd` frame: exit code, cwd, branch; `BlockTimes` for the duration | the long-command notification (Warp second pass, finding 3), #535, jump to the first failure (plan T2), a log in Rusty | observe |
| `terminal.ready` | the `bootstrapped` frame, after `init` and `history` | #527's launch configs typing a first command, #510 | observe |
| `terminal.opened`, `terminal.closed` | `observe_new::<TerminalView>` and `on_release` in `notifications.rs` | the rail; the hold on a closed working terminal (Warp second pass, Chad's answer 3) | observe |
| `terminal.notification` | `terminal::Event::MarleyNotification`, from OSC 9 and 777 | the desktop banner, #535 | observe |
| `terminal.program_changed` | the rail's poll of `foreground_process_command_name`; #547 ends a seat when Claude Code leaves | the agent bar, the rail | observe |
| `agent.working`, `agent.waiting`, `agent.stopped`, `agent.failed` | seat transitions through `claude_events::fold` in `agent_events::on_frame`; nothing emits the transition | #538, #535, #542, #508, the System One stop kind | observe |
| `agent.prompt_submitted`, `agent.tool_started`, `agent.tool_finished` | the same frames: UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure | #509's per-turn diffs, an audit log | observe |
| `agent.permission_requested` | the PermissionRequest frame | #508's inbox | observe; the answer stays Chad's (recommendation 3) |
| `agent.session_started`, `agent.session_ended` | SessionStart, SessionEnd, and #547's end on exit | #540, a transcript archive | observe |
| `agent.quiet` | computed when read (`claude_events::seat_line`, `AgentEvents::next_quiet_change`, #547) | a notification | observe |
| `thread.*` for Zed's Agent Panel | `AcpThreadEvent` (`ToolAuthorizationRequested`, `Stopped`, `Error`), which the rail subscribes to | #508, #538 | observe |
| `browser.page_opened`, `browser.page_closed`, `browser.navigated` | `BrowserEvent::PageOpened`, `PageClosed`, `PageInfoChanged` on `BrowserHub` | #504's rail rows, a history log | observe |
| `browser.dialog_opened` | `BrowserEvent::DialogOpened`; the page's script waits for the answer | a notification | observe |
| `browser.pick_staged` | `BrowserEvent::PickStaged` (#496) | the agent, #505, a pick archive | observe |
| `browser.console_error`, `browser.request_failed` | `ConsoleLog::apply` and `NetworkLog::apply` in `marley_browser::observe` | the System One note's use 7, the agent feed | observe |
| `browser.recording_saved` | `recorder::save_in` (#499) | #506's Playwright draft | observe |
| `mcp.tool_call` | `answer` in `crates/marley_workbench/src/mcp.rs`, after `marley_mcp`'s grant check | an audit log, #524's client names | refuse, write tools only |
| `project.opened` | `observe_new::<Workspace>` with `opened_from_saved_state` (#455) | #527, per-project setup | observe |
| `worktree.created` | Zed's `create_worktree` task hook | setup scripts | Zed's rule: a failing task stops that worktree's later setup tasks |
| `worktree.removing` | nothing yet (#511) | an archive hook | refuse: a failure blocks the removal |
| `file.saved`, `git.head_changed`, `diagnostics.updated` | `workspace::Event::UserSavedItem`; `GitStoreEvent::RepositoryUpdated` with `RepositoryEvent::HeadChanged`; `project::Event::DiskBasedDiagnosticsFinished` | #531, an `editor_diagnostics` tool (Warp second pass, finding 1), formatters | observe; Zed's `format_on_save` covers the usual save case |
| `app.started`, `app.quitting` | `mcp::start` from `zed`'s `main`; `cx.on_app_quit` | cleanup, Rusty | observe; asking before a quit ends a working agent is Marley's own dialog (Warp second pass, finding 2) |

Frequency settles the rest. Tool events come many times a minute while an agent works, block
events a few times a minute per terminal, and the others rarely. PTY output
(`terminal::Event::Wakeup`), screencast frames, keystrokes and rendering fire nothing: a hook
there would cost more than it could do. Agents other than Claude Code have quiet-based status
only, until they report events of their own (#538's scope says the same).

## The mechanisms

### Zed's extensions

Without a rebuild, a Zed user can add settings and keymaps, themes, snippets, `tasks.json` entries
with their one event hook, MCP servers for the agents (`context_servers`), agents from the ACP
Registry, and extensions. An extension is a directory with an `extension.toml`. Languages,
grammars, themes, icon themes and snippets are data; code is Rust compiled to a WebAssembly
component for the `wasm32-wasip2` target (`docs/src/extensions/developing-extensions.md`). The
world in `crates/extension_api/wit/since_v0.8.0/extension.wit` exports language-server commands
and options, completion and symbol labels, slash commands, context-server commands, docs indexing
and debug adapters. It imports downloads, processes, HTTP, GitHub, Node, settings and read access
to a worktree. Nothing in it sees an editor event, a terminal, an agent or the UI. The one event
bus in the extension crates, `ExtensionEvents` (`crates/extension/src/extension_events.rs`), tells
Zed itself that an extension was installed or removed.

The sandbox is wasmtime 48 with the component model (`crates/extension_host/src/wasm_host.rs`).
`build_wasi_ctx` gives WASI preview 2 a single preopened directory, the extension's own work
directory, read-write, and inherits stdio. Epoch interruption every 100 ms makes a busy extension
yield to the executor, but no deadline ends a long call, and the host sets no memory limit and no
fuel. The capabilities are `process:exec`, `download_file` and `npm:install`
(`crates/extension/src/capabilities.rs`). `CapabilityGranter` checks each call against the
manifest and the `granted_extension_capabilities` setting, whose default grants all three for any
command, host and package (`assets/settings/default.json`), so an extension that declares it can
run any program the user can. Extensions install from the zed.dev registry
(`/extensions/{id}/download` in `crates/extension_host/src/extension_host.rs`), or as dev
extensions built from a directory with rustup's `wasm32-wasip2` target and wasi-sdk 34 for grammars
(`crates/extension/src/extension_builder.rs`).

The host keeps every API version it has shipped: ten version modules under `wasm_host/wit/`
(3,977 lines) beside `wit.rs` (1,308). Stable and preview builds load API versions up to 0.7.0;
0.8.0 is marked unreleased and loads on dev and nightly only (`wasm_api_version_range` and
`authorize_access_to_unreleased_wasm_api_version` in `wit.rs`), and Marley runs on the dev
channel. Marley imports or exports added to the `zed:extension` package would need a hunk in each
new version module upstream adds, so a Marley world, if one is ever built, belongs in its own WIT
package and its own crate. It could reuse the wasmtime and wasmtime-wasi that Zed already builds,
and it would follow Zed's pin (`wasmtime = "48"` in `Cargo.toml`, while docs.rs shows 49.0.1), so a
wasmtime bump in an upstream merge can break Marley's host code.

### Claude Code's hooks

Per code.claude.com/docs/en/hooks, hooks live in JSON settings (user, project, local, managed
policy, a plugin's `hooks/hooks.json`, skill and subagent frontmatter). Each is an event, a matcher
group that filters it (a tool name, a regular expression, `*`), and handlers of type `command`,
`http`, `mcp_tool`, `prompt` or `agent`. A command hook reads the event's JSON on stdin. Exit 0 lets
the action proceed and reads JSON from stdout. Exit 2 blocks on the events that can block
(PreToolUse, UserPromptSubmit, Stop and the others the page lists), with stderr as the reason. Any
other exit is a non-blocking error, and the page warns: "If your hook is meant to enforce a policy,
use `exit 2`." The JSON output has universal fields (`continue`, `stopReason`, `systemMessage`
for the user, and `terminalSequence`, limited to OSC 0, 1, 2, 9, 99, 777 and BEL), a top-level
`decision: "block"` with a `reason`, and `hookSpecificOutput` for PreToolUse's
`permissionDecision` (allow, deny, ask, defer), `updatedInput` and `additionalContext`.

Matching hooks run in parallel, and one handler defined in two settings files runs once. Command
hooks default to a 600 s timeout, 30 s on UserPromptSubmit, and a PreToolUse hook that times out
does not block. `async` runs a hook in the background; `asyncRewake` does too, and on exit 2 it
"wakes Claude immediately even when the session is idle", with stderr shown to Claude as a system
reminder. On safety the page says "Command hooks execute shell commands with your full user
permissions." An interactive session holds back hooks from every settings file until the folder's
trust dialog is accepted, while `-p` treats every folder as trusted. Edits take effect through a
file watcher, `/hooks` lists every hook read-only, and `disableAllHooks` turns them all off.

Marley's plugin already rides this contract, and Chad already runs blocking hooks through it:
Rusty's `brain-ask-before-write.sh` on PreToolUse and `brain-decide-before-stop.sh` on Stop.
Marley's user hooks should copy the shape (a command, the event's JSON on stdin, exit 2 or
`{"decision": "block", "reason": "…"}` to refuse where refusing is allowed) and avoid its trap:
exit 1 does not block, so a gate written with `set -e` fails open.

### MCP notifications, subscriptions and channels

In the 2025-06-18 revision that Marley speaks, a client subscribes with `resources/subscribe` and
the server sends `notifications/resources/updated` carrying only the resource's URI, after which
the client reads the resource again (modelcontextprotocol.io/specification/2025-06-18/server/resources).
A subscription says that state changed and never what happened, so a feed of events needs a
resource read with a cursor. The 2026-07-28 revision removes protocol sessions and the
`initialize` handshake, replaces the HTTP GET stream and `resources/subscribe` with one
`subscriptions/listen` request whose response stream carries the notifications a client opted
into, and drops stream resumability (specification/2026-07-28/changelog and
basic/patterns/subscriptions). Claude Code's v2 client runtime speaks the new revision with HTTP
servers that support it, and keeps stdio servers such as Marley's bridge on the earlier handshake
unless `MCP_PROTOCOL_NEGOTIATION=auto` (code.claude.com/docs/en/mcp, "MCP client runtimes"). The
same page documents Claude Code acting on `list_changed` notifications and says nothing about
resource updates. So `fleet://snapshot` is for outside clients (rustal-harness's manager, Rusty),
and an agent inside Claude Code learns of an event only by calling a tool.

Channels are Claude Code's way for an MCP server to push into a running session, in research
preview (code.claude.com/docs/en/channels and channels-reference). A stdio server declares
`capabilities.experimental['claude/channel']` and sends `notifications/claude/channel` with a
`content` string and string `meta`, which reaches Claude's context as a `<channel source="…">`
tag. Events queue and arrive together on the next turn, unacknowledged. A server must be named in
`--channels`, and during the preview only allowlisted plugins register, so Marley's would need
`--dangerously-load-development-channels`. The reference warns "An ungated channel is a prompt
injection vector." A channel that also declares `claude/channel/permission` receives each tool
approval prompt as `notifications/claude/channel/permission_request` (`request_id`, `tool_name`,
`description`, `input_preview`) and may answer `allow` or `deny`; the terminal's own dialog stays
open and the first answer wins. MCP gives a client no way to veto anything inside Marley, since a
client only calls tools. MCP as the control plane in both directions is an active decision already
(`AD-claude-mcp-first-class-control-plane-001` in `docs/planning/knowledge/architecture-decisions.md`).

### Orca

Orca's plugin system is experimental and off by default (`pluginSystemEnabled: false` in
`src/shared/default-global-settings.ts`), with a closed event list. A plugin's `orca-plugin.json`
subscribes under `contributes.events` to `worktree.created`, `worktree.removed` or
`agent.status.changed` (`src/shared/plugins/plugin-manifest.ts`). Each payload is a zod schema
with length bounds that `plugin-events.ts` calls "bounded projections" and "never raw runtime
objects". A subscribed plugin's worker is started to take the event, and an event it cannot take
is logged and dropped (`src/main/plugins/plugin-event-delivery.ts`). Seven capabilities
(`workspace:read`, `terminal:send`, `notifications:show`, `storage`, `secrets`,
`events:subscribe`, `settings:own`) gate calls into Orca, with consent bound to a fingerprint of
the set (`plugin-capabilities.ts`, `plugin-consent-fingerprint.ts`), and the worker is a forked
Node process that can still reach the file system, the network and child processes (Orca survey
05, §2.20).

Repo hooks are `orca.yaml`'s `scripts.setup` and `scripts.archive`. They run after the user
approves the script, with the approval tied to a hash of its text (`ensure-hooks-confirmed.ts`).
The archive hook runs as a process-group leader with a 2-minute deadline and a 10 MiB output cap,
and a failure or a timeout blocks the worktree's removal (`HOOK_TIMEOUT` and
`HOOK_OUTPUT_LIMIT_BYTES` in `src/main/hooks.ts`; survey 02, §2.2). Automations start an agent on a
cron or RRULE schedule after a precheck command whose non-zero exit skips the run (survey 06,
§2.10). Orca's agent status comes from entries it writes into `~/.claude/settings.json` that post
to a loopback server (survey 06, §2.14), the job Marley's plugin does in-band.

### Warp

Warp's published docs describe no plugin, extension or hook API for the terminal. It extends
through files and escape sequences: YAML workflows, parameterized commands kept on the machine or
in a repo's `.warp/workflows/` (docs.warp.dev/terminal/entry/yaml-workflows/, which now points to
Warp Drive workflows); Tab Configs, TOML files that set a tab's directory, startup commands,
panes, shell and theme with parameterized inputs (docs.warp.dev/terminal/windows/tab-configs/); a
URI scheme (`warp://action/new_tab?path=`, `warp://launch/<path>`, `warp://tab_config/<name>`;
docs.warp.dev/terminal/more-features/uri-scheme/); and "Custom notification hooks (OSC 9 / OSC
777)": "Warp supports pluggable notifications triggered by terminal escape sequences, so scripts
and tools can raise desktop notifications without additional dependencies"
(docs.warp.dev/terminal/more-features/notifications/). The docs call MCP servers plugins for
Warp's agent (docs.warp.dev/knowledge-and-collaboration/warp-drive/ai-objects/), and the cloud
Automation Platform starts agents from a manual, cron, webhook or integration trigger
(docs.warp.dev/platform/faqs/). Marley has Warp's in-band notifications already (#478), and by
Chad's answer 2 in `warp-blocks-and-natural-language-2026-09-25.md`, Marley's workflows will be
Zed tasks.

### Shell hooks and scripting in the process

Shell hooks cost Marley no runtime: any language, the user's own tools (`jq`, `notify-send`,
`curl`), one process per event. A process keeps no state between events except in files, runs with
the user's full permissions, and has no sandbox.

Lua through mlua (0.12.1, MIT) binds Lua 5.1 to 5.5, LuaJIT and Luau. Its `vendored` feature builds
and links the interpreter from source, `Lua::sandbox` exists for Luau only, the crate is `!Send`
unless the `send` feature is on, and its README says "`mlua` does not provide absolute safety even
without using `unsafe`" (docs.rs/mlua, github.com/mlua-rs/mlua). Lua is already how Chad's desktop
is configured: Hyprland 0.56 on the dev box takes its config and dispatchers as Lua
(`docs/planning/knowledge/lessons.md`).

Rhai (1.26.1, MIT OR Apache-2.0) is pure Rust. An engine declared immutable "cannot mutate the
containing environment unless explicitly permitted", it is protected against "stack-overflow,
over-sized data, and runaway scripts", a host can track a script's progress and stop it, and "Any
panic is a bug" (rhai.rs/book/about/features.html). Its few dependencies are small crates, and
nobody on the box writes it today.

WebAssembly components through wasmtime are already in `Cargo.lock` (48.0.1, with the component
model and async). They give the strongest isolation Marley could offer: WASI preopens decide what a
component sees, an epoch deadline can trap a runaway call, and `StoreLimitsBuilder` caps memory
(docs.rs/wasmtime). They also ask the most of an author: a toolchain, a WIT file and a build step.

All of these carry licenses on `deny.toml`'s permissive list. None may run on gpui's foreground
thread: a script or a component gets a thread of its own and a deadline, so a slow hook never
holds up a frame.

### An in-process subscriber API

gpui's `EventEmitter` and `cx.subscribe` are how Zed's crates talk to each other, and
`ExtensionEvents` is one global entity that emits. Marley has the same shapes: `AgentEvents` is a
global the rail and the MCP server observe, and `BrowserHub` emits `BrowserEvent`. A `MarleyEvents`
global that emits one closed enum would give every Marley crate one place to subscribe. It also
answers "extend crates": a crate of Chad's in the workspace subscribes in its `init` and is built
with Marley, typed and with no sandbox to design. That route needs a rebuild. Rust without a
rebuild means WebAssembly, since Rust has no stable ABI for native plugins.

## Where each fits Marley

The mechanisms Marley should take fit one design, which sends events out and takes actions in.
One stream of typed events leaves Marley's crates, and user hooks, the MCP feed and any later
script or component read from it. Whatever a hook or an outside tool wants done comes back through
Marley's MCP tools, the surface agents already use, where the grant table (deny by default,
`browser.write` granted today) decides. No mechanism gets a second way to act on Marley, so there
is one permission model and one place to audit.

| Mechanism | Fits | Safety | Cost at an upstream merge | User effort |
|---|---|---|---|---|
| In-process event stream | every Marley crate; the source for everything below | reviewed Rust in the process | none: Marley crates observing Zed's events | none for users; ordinary work for Chad's own crates |
| User hooks as Zed tasks | the first user-facing extension: scripts on lifecycle events | full user permissions, so trust, approval of the text and deadlines | one row: new `TaskHook` variants in `crates/task/src/task_template.rs` | lowest: a script and a `tasks.json` entry |
| MCP event feed and tools | agents, rustal-harness's manager, Rusty, the phone relay | the grant table, the bearer on loopback, #516's redaction | none | none for agents; an MCP client or `marley call` otherwise |
| An `asyncRewake` hook in the plugin | waking an idle Claude Code with one of Marley's events | Marley's own words only, never page or program text | none | none |
| Claude Code channels | pushing events into a session; answering prompts from the rail | research preview, a dev flag, a prompt injection route | none | none once allowlisted |
| Zed extensions given Marley capabilities | nothing Marley needs | Zed's sandbox, with any program allowed by default | high: a hunk in every new upstream API version | high: Rust, WIT, `wasm32-wasip2` |
| A Marley WebAssembly world | sandboxed extensions shared with other users | the strongest: preopens, deadlines, memory limits | low in Zed's code; wasmtime bumps can break the host | high |
| Luau through mlua | handlers that keep state or run often; small transforms | a sandbox mode, not absolute | none in Zed's code; a C++ library in the build | medium: Lua is familiar, the API is new |
| Rhai | the same, in pure Rust | sandboxed by default, with DoS limits | none | medium: a new language |
| Native Rust plugins | nothing | none: a fault takes the app down | high: tied to the compiler and to gpui | high |

## Safety rules

1. Trust follows the source. Global hooks (`~/.config/marley/tasks.json`) are Chad's. Project
   hooks from a repo's `.zed/tasks.json` run only in a worktree Zed trusts (`TrustedWorktrees`,
   `crates/project/src/trusted_worktrees.rs`) and only after Chad approves the hook's exact text;
   Marley keeps a hash, and a changed text asks again, as #527 does for launch configs and Orca
   does for `orca.yaml`. Zed has no such gate on tasks: `.zed/tasks.json` loads without the trust
   check that `.zed/settings.json` gets (`update_settings` in
   `crates/project/src/project_settings.rs`), and I found no trust check before
   `run_create_worktree_tasks` runs a new worktree's `create_worktree` tasks, its own included.
2. Agents run as Chad and can edit his task files, so a new or changed hook asks once before it
   runs, wherever it lives. That stops a prompt-injected agent from quietly installing a hook that
   fires on every event.
3. Hooks observe unless an event is on the short refuse list (MCP write tool calls, worktree
   removal). A refusal is exit 2 or `{"decision": "block", "reason": "…"}`, as in Claude Code, and
   nothing a hook returns approves: the grant table and the agents' own prompts stay the only ways
   in. A hook that times out or crashes allows, except the archive hook, whose failure blocks the
   removal as Orca's does.
4. Events carry bounded projections, capped as the plugin caps its frames (a prompt or a message at
   300 characters, a preview at 200), and #516's redactor runs before an event leaves the process.
   No transcript, block output or page text rides in an event. A hook that wants a block's output
   asks for it with `marley call terminal_read`, under the grants.
5. Each hook run gets a deadline (30 s unless the spec finds a better default), its process group
   is killed after a 2 s grace, and its output is capped and logged. One run of a hook at a time;
   events that arrive meanwhile queue up to a bound, and the rest are counted as dropped.
6. An event caused by a hook's own action carries the hook as its origin and fires no hook.
7. Nothing fires on output, frames, keystrokes or rendering, and no hook, script or component runs
   on gpui's foreground thread.
8. Text pushed into an agent (the `asyncRewake` pointer, a channel) is Marley's own sentence with
   ids, never a page's console text or a program's output.

## Recommendations, ranked

S is a day, M a few days, L a week or more, as in the Orca survey.

1. **The event stream (S), the first slice.** A small pure crate, `marley_events`, holds one
   closed, serde-tagged, versioned enum (`v`, `seq`, `ts_ms`, the terminal and project, the
   payload); `marley_fleet` cannot hold it, since its crate doc forbids product vocabulary. A
   `MarleyEvents` global entity in `marley_workbench` emits it and keeps a ring of the newest 1,000
   events. Its sources are seat transitions, compared before and after the fold in
   `agent_events::on_frame`; block starts and finishes, found by observing each `Terminal` and
   comparing its block count and last block's state, which needs no Zed change (a typed
   `terminal::Event` variant is the fallback, at the cost of rows for `terminal.rs` and for
   `agent_panel.rs`'s exhaustive match); `BrowserEvent`s mapped from the hub; and the terminal
   notifications. The first subscribers are #535 and #538, which both need seat transitions; the
   before-and-after report #535 adds to `on_frame` is where the stream would start, and #538 can
   read that same report rather than keep a copy of its own. The stream waits on #520 for terminal
   ids that survive a relaunch, and it has no user surface. Beyond that shared report, it waits
   with the rest of this note.
2. **User hooks as tasks (M).** Zed's `TaskHook` gains variants such as `marley.agent.waiting`,
   `marley.agent.stopped`, `marley.agent.failed`, `marley.block.finished`,
   `marley.browser.pick_staged` and `marley.terminal.notification`, an additive enum change with one
   touchpoint row. A stock Zed that reads such a task drops it with a logged error and keeps the
   rest of the file (`update_file_based_tasks` in `crates/project/src/task_inventory.rs`). Marley
   finds the matching templates (`templates_with_hooks`), resolves each (`resolve_task`) and runs it
   without a terminal: the event's JSON on stdin; `MARLEY_EVENT`, `MARLEY_TERMINAL_ID` and
   `MARLEY_PROJECT` in the environment; output to a hooks log. This slice reads only the global
   `tasks.json`, observes only, and asks before a new hook's first run (rule 2). It also adds the
   Orca survey's `marley call <tool> --json '<args>'` (report 06, item 8), so a hook acts through
   the tools agents use. Its e2e scenario (§7) is a hook that appends each event to a file the
   scenario shows.

   ```json
   [
     {
       "label": "Log each finished command",
       "command": "jq -c '{command, exit_code, duration_ms}' >> ~/.local/state/marley/blocks.log",
       "hooks": ["marley.block.finished"]
     }
   ]
   ```

3. **The feed for agents (S to M).** Marley's MCP server gains a `marley://events` resource, read
   with `after` and `kinds`, and an `events_wait {after, kinds, timeout_ms}` tool capped at 60 s,
   since Claude Code moves an MCP call still running after two minutes to a background task
   (code.claude.com/docs/en/mcp).
   Both keep working under MCP 2026-07-28, where the GET stream and `resources/subscribe` do not.
   To wake an idle Claude Code, the plugin gains an `asyncRewake` command hook on Stop that runs
   `marley call events_wait` in a loop for events addressed to its own terminal (a pick Chad sent,
   a review note from #522, its dev server failing) and exits 2 with one line on stderr, such as
   "Chad sent pick 3 from the Browser tab; call browser_pick with id 3." It exits quietly when its
   agent starts a new turn, since Claude Code does not deduplicate async hooks. That is Orca's
   pointer delivery without typing into the TUI. Channels wait for the end of the research
   preview. For #508, two routes would let the rail answer a Claude Code prompt in place: a
   PermissionRequest hook of type `mcp_tool` that waits on Marley (the Orca survey README, item 3)
   and a channel's permission relay. Either keeps the answer Chad's, and #508's spec can weigh them.
4. **Refusals and project hooks (M).** `mcp.tool_call` for write tools runs after the grant check
   and before the app acts; a refusal reaches the agent as the tool error with its reason and a next
   step (the Orca survey's report 06, item 7), and a timeout allows. It could keep agent navigation
   to listed hosts, or keep #525's `terminal_type` out of a production shell. `worktree.removing`
   is #511's archive hook, whose failure blocks the removal until Chad overrides. Project hooks in
   `.zed/tasks.json` follow rule 1. Putting the same gate on Zed's `create_worktree` path is one
   more row, for `crates/workspace/src/tasks.rs`, and worth it once #510 makes worktrees from
   branches agents wrote.
5. **A scripting layer (L), only when shell hooks fall short.** The signs: a handler needs state
   across events, runs on tool events often enough that a process per event shows, or must shape an
   outcome faster than a process starts (a banner's wording, a rail row's text). The layer would
   then be Luau through mlua in sandbox mode, on its own thread with a deadline, with registered
   functions that reach the MCP registry. The lean toward Luau over Rhai is a judgment: Chad's
   desktop is configured in Lua, and an agent asked to write a hook has far more Lua than Rhai to
   learn from. Rhai's case is its pure-Rust build and the limits it ships with.
6. **A Marley WebAssembly world (L), only when Marley has users besides Chad.** A `marley:extension`
   WIT package in a Marley crate: it exports `on-event`, imports `call-tool` (through the grant
   table) and `log`, gets no preopened directories, runs under an epoch deadline that traps and a
   `StoreLimitsBuilder` memory cap, and installs from a directory as Zed's dev extensions do. It
   never changes `zed:extension`.

## What to leave out

- Native Rust plugins (`libloading`, `abi_stable`, cdylibs): Rust has no stable ABI, gpui types
  cannot cross the boundary, and a fault takes the app down.
- Any change to Zed's `zed:extension` world or to `extension_host`.
- Hooks on PTY output, frames, keystrokes or rendering.
- Hooks that approve. None may allow a tool call, answer an agent's prompt or widen a grant; the
  answer to an approval stays Chad's (#508), as the System One note's safety rules keep it.
- Policy over an agent's own tool calls, which belongs in that agent's hooks. The one-cargo rule,
  for instance, is a `pgrep cargo` in a Claude Code PreToolUse hook and needs nothing from Marley.
- A veto on shell commands: `preexec` reports a command the shell is already running, and the
  shell is the user's.
- Panels, rail rows or other UI from extensions: gpui has no plugin UI surface and Zed has no panel
  extension point (Orca survey 05, §2.20).
- A marketplace, a registry, auto-update or sharing. Orca's marketplace lists eight plugins, all
  its own.
- Managed hook entries in `~/.claude/settings.json`; the plugin stays the channel (Orca survey 06,
  §4).
- A Node or Python plugin host.

## Open questions for Chad

1. Where do user hooks live: `tasks.json` entries with `marley.*` names on Zed's `hooks` field,
   which matches your workflows answer; a `marley.hooks` block in `settings.json`, whose project
   copies Zed already gates by worktree trust; or `.zed/marley.json` beside #527's launch configs?
   Default: `tasks.json`.
2. Should hooks from a repo's `.zed/tasks.json` run at all? Default: your global hooks only at
   first; project hooks later, behind worktree trust and approval of their exact text.
3. Should a new or changed hook in your own global file also ask once before it runs, since agents
   in Marley's terminals can edit that file? Default: yes.
4. May any hook refuse an action? Default: observe only in the first user slice; later, refusals
   for MCP write tools and worktree removal, and never approvals.
5. For agents, the resource and `events_wait` first, then the plugin's `asyncRewake` waiter, then
   channels once they leave research preview? Default: yes, in that order.
6. Park scripting and WebAssembly until a hook needs state or speed that a process cannot give, or
   until Marley has users besides you? Default: park both.
7. For "extend crates", is a crate of yours in the workspace, subscribing to the event stream and
   rebuilt with Marley, enough? Default: yes.

## Sources

Claude Code: https://code.claude.com/docs/en/hooks (events, matchers, exit codes, JSON output,
`terminalSequence`, async hooks, security considerations),
https://code.claude.com/docs/en/channels, https://code.claude.com/docs/en/channels-reference and
https://code.claude.com/docs/en/mcp ("MCP client runtimes", "Dynamic tool updates", "Automatic
backgrounding of long tool calls").

MCP: https://modelcontextprotocol.io/specification/2025-06-18/server/resources,
https://modelcontextprotocol.io/specification/2026-07-28/changelog and
https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/subscriptions.

Warp, published docs only: https://docs.warp.dev/terminal/entry/yaml-workflows/,
https://docs.warp.dev/terminal/windows/tab-configs/,
https://docs.warp.dev/terminal/sessions/launch-configurations/,
https://docs.warp.dev/terminal/more-features/uri-scheme/,
https://docs.warp.dev/terminal/more-features/notifications/,
https://docs.warp.dev/terminal/integrations-and-plugins/,
https://docs.warp.dev/knowledge-and-collaboration/warp-drive/ai-objects/,
https://docs.warp.dev/platform/faqs/, and the section files listed in https://docs.warp.dev/llms.txt.

Scripting: https://docs.rs/mlua/latest/mlua/, https://github.com/mlua-rs/mlua,
https://rhai.rs/book/about/features.html, https://rhai.rs/book/safety/index.html,
https://docs.rs/wasmtime/latest/wasmtime/struct.StoreLimitsBuilder.html, and crates.io for the
mlua and Rhai versions and licenses.

Orca (MIT, `/srv/stacks/orca-refs/orca`): `src/shared/plugins/plugin-manifest.ts`,
`src/shared/plugins/plugin-events.ts`, `src/shared/plugins/plugin-capabilities.ts`,
`src/main/plugins/plugin-event-delivery.ts`, `src/shared/default-global-settings.ts`,
`src/main/hooks.ts`, and the survey in `docs/orca_architecture/` (README; reports 02, 05 and 06).

Marley and Zed code read: `crates/marley_agent/src/{marley_agent.rs,claude_events.rs}`,
`crates/marley_workbench/src/{agent_events.rs,notifications.rs,mcp.rs,rail.rs,blocks.rs,browser.rs,claude_plugin.rs,marley_workbench.rs}`,
`crates/marley_workbench/claude_plugin/marley/` (`hooks/hooks.json`, `hooks/event.py`,
`hooks/notify.sh`, `bin/marley-mcp-bridge`), `crates/marley_mcp/src/{marley_mcp.rs,dispatch.rs,resource.rs,registry.rs,transport.rs}`,
`crates/marley_fleet/src/{marley_fleet.rs,reducer.rs}`,
`crates/marley_terminal/src/{marley_terminal.rs,dcs.rs,block.rs,anchored.rs}` and
`shell_integration/marley.bash`, `crates/marley_browser/src/{observe.rs,recorder.rs}`,
`crates/settings_content/src/marley.rs`, `crates/terminal/src/terminal.rs`,
`crates/task/src/task_template.rs`, `crates/workspace/src/{tasks.rs,workspace.rs}`,
`crates/project/src/{project.rs,git_store.rs,task_inventory.rs,task_store.rs,project_settings.rs,trusted_worktrees.rs}`,
`crates/acp_thread/src/acp_thread.rs`, `crates/extension/src/{extension_events.rs,capabilities.rs,extension_manifest.rs,extension_builder.rs}`,
`crates/extension_api/wit/since_v0.8.0/extension.wit`, `crates/extension_api/README.md`,
`crates/extension_host/src/{wasm_host.rs,wasm_host/wit.rs,capability_granter.rs,extension_settings.rs,extension_host.rs}`,
`assets/settings/default.json`, `Cargo.toml`, `docs/src/tasks.md`,
`docs/src/extensions/{developing-extensions.md,capabilities.md,agent-servers.md}`, and the plan,
touchpoints, backlog, tickets and design notes named above.
