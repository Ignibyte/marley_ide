# Marley on Zed: the three-prong plan

*Written 2026-09-18 in the fork at `/srv/stacks/marley_ide`. Sources: the Zed terminal
crates as they are today, the design record in `/srv/stacks/marley/docs` (the Warp
deconstruction, the fleet control plane, the orchestration shell, the embedded-browser model
and its 2026-08-11 amendment), `rustal-harness/docs`, `rustal-brain/docs/architecture`,
and Rusty's `ROADMAP.md` and `docs/architecture.md`. A plan, not a spec; each slice gets its
own ticket when it starts.*

Marley is the Zed fork. Zed supplies the editor, language intelligence, project model,
settings, themes, the agent panel with MCP and ACP, and a terminal. The three prongs are what
Zed does not supply and what Marley was always about:

| Prong | One line | Ported crates it builds on |
|---|---|---|
| 1. The terminal | A Warp-class block terminal inside Zed's terminal, not beside it | `marley_terminal` |
| 2. The control plane | Marley as the mechanism shell for rustal-brain, the harness runtime and Rusty | `marley_fleet`, `marley_mcp`, `marley_agent`, `marley_remote` |
| 3. The browser | One Chromium, run as a service, rendered in a pane and driven over CDP by both the human and the agent | none yet |

The prongs share one spine: everything an agent can see or do goes through a typed seam
(a Block, a Session event, a CDP node), never through screen scraping. That was the lesson of
the fleet evidence night and it holds for all three.

## Where the fork stands

- Upstream Zed `main` at `78648aaf7d`, no divergence except the five ported crates under
  `crates/marley_*` (compiled, 246 tests, clippy and fmt clean, not yet used by the app).
- Zed's terminal (`crates/terminal`, `crates/terminal_view`) is a flat alacritty grid. It has
  no command boundaries, no shell integration, and learns the running command and cwd by
  polling the process table. Its only per-command status is `TaskStatus` for spawned tasks,
  shown as a tab icon and a summary line appended to scrollback.
- The parser runs on alacritty's own "PTY reader" thread inside the fork
  `zed-industries/alacritty` (rev `4c12966`). vte 0.15 swallows every DCS sequence and every
  unknown OSC before the `Term` handler sees it, so shell hooks cannot be caught without
  touching that layer.
- Zed already gives agents MCP servers by URL with headers (`context_servers` in settings)
  and runs Claude, Codex, Gemini and OpenCode as external agents over ACP. Prong 2 does not
  need to rebuild either.

## Prong 1: the block terminal

### The bar

What "Warp-like" means, taken from the Warp deconstruction and from what Marley had already
shipped at parity:

1. Every command is a Block: the command line, its framed output, an exit-status pill,
   the cwd and git branch it ran in, its duration, and a Running state while it runs.
2. Block boundaries come from the shell, not from heuristics. Marley's shell hooks
   (`init`, `preexec`, `precmd`, `bootstrapped`) carry the command text, exit code, pwd,
   branch and subshell identity as a DCS payload with a tested codec.
3. Block actions: copy command, copy output, rerun, jump between blocks, collapse, select a
   block, and open the first `file:line:col` in the failed block's output at the block's own
   cwd (Zed today resolves paths against whatever cwd the process has when you click).
4. The prompt is an editor: multi-line editing, history ghost text, completions, syntax
   colouring of the command line, and raw passthrough the moment a program owns the
   terminal (alternate screen, a running command, bracketed paste, application cursor keys).
5. Tasks and runnables land as Blocks with a status pill, not as a tab with an icon.
6. Agents read Blocks as data (prong 2), never scrollback text.

Out of scope on purpose: Warp Drive, notebooks, block sharing, Warp's cloud AI, any login.

### Design decisions

**D1. Catch shell hooks in the alacritty fork's event loop, not with a second PTY engine.**
`marley_terminal` as ported owns its own PTY and its own headless `Term`; wiring it whole
would run two terminal engines per pane. Instead, Marley carries its own copy of the crate
at Zed's rev, `vendor/alacritty_terminal` in this repository (#461; Chad chose one repository
over a fork of its own on 2026-09-23), re-synced when Zed bumps the rev, with one change:
`event_loop.rs` runs the DCS scanner from `marley_dcs` (a leaf crate since #462, since
`marley_terminal` depends on alacritty) over each read buffer,
feeds the passthrough bytes to the parser as before, and at each complete hook snapshots the
grid position (history size, cursor line, and a new monotonic evicted-lines counter on the
grid) and emits `Event::ShellHook { final_byte, payload, position }`. Zed's
`TerminalBackendEvent` gains the mirror variant. Decoding and the block state machine stay in
`marley_terminal` (`dcs.rs`, `apply.rs`, `block.rs`); `session.rs` and `pty_os.rs` shrink to
the mock-tested hook logic or go. Fallback if the fork branch proves painful: a filtering
reader wrapped around the PTY that strips hooks and reports them on the next wakeup, at the
cost of exact positions.

**D2. Blocks are ranges over the one scrollback, anchored by absolute line.**
A `BlockList` on `Terminal` maps absolute lines (evicted count plus scrollback position) to
block metadata. When the history is full, evicted lines shift positions; the counter keeps
older anchors valid until their output is gone, at which point the block keeps its metadata
and is marked output-evicted. Reflow on resize was the known weak spot; the mitigation to
evaluate was tagging the prompt's first cell with a synthetic OSC 8 hyperlink id that
survives reflow and lets the block re-find its start. #544 settled it without the tag: every
anchor is carried across a width change as logical lines from the cursor's logical line, which
alacritty's rewrap keeps, and a character offset inside its own.

**D3. Render in two stages.** Stage one draws block decorations without changing the row
model: a gutter bar per block, a status pill at the block's top right, a background wash on
the running or failed block, hover actions. Stage two inserts native header rows above each
block and hides the shell's PS1 (the Warp look), which needs a display-row map in
`TerminalElement` like the editor's block map. Stage one ships first because it touches no
scroll math.

**D4. The prompt editor is a Zed `Editor`.** Between `precmd` and `preexec` the terminal is
at a prompt; keys route to an auto-height `Editor` docked at the bottom of the terminal
view, Enter writes the line plus CR to the PTY, and the same key ladder Marley proved
(alt screen, running command, bracketed paste, app cursor) sends everything else raw.

**D5. Shell integration ships with Marley.** `assets/shell_integration/` carries the zsh,
bash and fish hooks and Zed's spawn path injects them the way Warp and Kitty do: `ZDOTDIR`
for zsh, `--rcfile` for bash, `XDG_DATA_DIRS` for fish, with the user's own rc still
sourced. Marley's DCS format stays; emitting OSC 133 alongside it is cheap and worth doing
for other terminals. Since #474 each local terminal gives its program a nonce, which the hooks
take out of the environment and add to each `preexec` frame: a block's command is trusted only
when its frame carried it, since any output can print a frame.

**D6. Tasks and runnables spawn Blocks.** `TaskState` stays for Zed's own task machinery,
but a task's spawn spec rides as Block metadata, the summary line is replaced by the
status pill, `HideStrategy` becomes collapse-on-success, and rerun re-spawns the block's
stored spec. Failed Blocks feed the diagnostics panel through the `file:line:col` scan
Marley shipped as `#433`.

### Slices

| Slice | Delivers | Size |
|---|---|---|
| T0 | Split at its promotion: the vendored `alacritty_terminal` (#461, T0a, shipped); `Event::ShellHook` from the event loop, with unit tests on recorded byte streams (#462, T0b, shipped); the anchored `BlockList` on `Terminal` and a `blocks()` accessor (#464, shipped); the hook scripts and their injection, bash first (#463, T0c, shipped), then zsh (#465, shipped) and fish (#466, shipped); a new terminal opens at the size the last one laid out (#485, shipped), and a launch's first terminals at the last session's (#486, shipped) | M |
| T1 | Stage-one rendering: gutter, pill, wash (#470, T1a, shipped); hover copy/rerun (#474, T1b, shipped); block navigation keys (#473, T1c, shipped); the content drawn against the bottom edge, so the prompt sits on the last row (#476, T1d, shipped); a selected block and the block menu: copies, Copy as Markdown, Reinput (#554, shipped); a block's output filtered in a panel over the terminal (#528, shipped; in place waits for T5); bookmarks on blocks with Alt+Up and Alt+Down and ticks at the edge, and Zed's search bar held to one block (#559, shipped); a scrolled-back block's command pinned over the top row, a click to its start (#529, shipped) | M |
| T2 | Block-scoped path links (resolve against the block's cwd) (#619, shipped) and jump-to-first-failure (#620, shipped) | S |
| T3 | The prompt editor with history ghost text and the raw-passthrough ladder; history ghost text on the shell's own prompt, → to take it (#484, T3a, shipped); a prompt editor at the shell's prompt on Ctrl+G (#624, T3b, shipped), with completions (#625, shipped) and the command's colours (#626, shipped); the prompt editor docked at every prompt by default, closing for a command or a full-screen program (#627, shipped); English at the prompt's second stage, the System One layer's reading of the lines the rules leave open, after the line in the editor and the grid, with #557's hint and Ctrl+Shift+Enter in the editor (#573, shipped) | L |
| T4 | Tasks and runnables as Blocks (#621, a task's run a block, shipped; #622, its Rerun Task and pill, shipped); failed Blocks into diagnostics (#623, shipped); failed Blocks into diagnostics; a block's command saved as a workflow, a Zed task with `{{name}}` parameters asked for when it runs (#558, shipped); a Markdown runbook's shell blocks put at the prompt of the terminal used last (#530, shipped) | M |
| T5 | Stage-two rendering: native header rows, PS1 hidden, Warp density; a block's prompt rows drawn as Marley's header with its command and pill, one for one, behind `marley.block_headers` (#628, shipped); the pinned header read as the native one and no search match drawn in a hidden prompt (#629, shipped) | L |
| T6 | Completions in the prompt (paths, history, tasks) (#625, in the prompt editor, shipped) and command-line colouring (#626, shipped) | M |
| T7 | CLI agents in the terminal, after Warp's agent toolbelt: the agent bar with the folder and branch (#477, T7a, shipped); desktop notifications from OSC 9 and 777 (#478, T7b, shipped) and a Claude Code plugin that sends them (#482, T7b, shipped); Attach File (#479, T7c, shipped); voice through Voxtype (#480, T7d, shipped); rich input, a Zed editor for the agent's prompt and the agent-first half of T3 (#481, T7e, shipped); a block sent to a CLI agent, by reference or inline, and Ask the agent under a failed block (#555, shipped); copy and paste that know an agent runs: its reply copied without the gutter, multi-line pastes bracketed, dropped images' raw paths (#536, shipped); git's credential prompts off in the terminals Marley opens for agents (#537, shipped); ssh's passphrase asked in a Marley dialog for an agent's terminal, Escape failing the command at once (#596, shipped); banners that say what Claude did, and an unread dot until the terminal is seen (#538, shipped); a plain command's end and a password prompt told from a terminal not in front, and the command's state on its rail row (#551, shipped); a running command that prints a failure and keeps running told once, marked on its row until it recovers, the shapes first and System One for the open lines (#572, shipped) | M |

Acceptance for T0 is the Marley integration test moved onto Zed's terminal: a real shell
emits the hook stream and a Finished block with exit 0 and the output "hi" appears.

### Risks

- The vendored copy is a standing re-sync cost each time Zed bumps alacritty
  (`vendor/README.md`). One file changes, and the diff is small.
- Reflow versus anchors (D2). Settled by #544 (logical places around each resize).
- Two input models in one view (D4) is where Warp itself is hardest to get right; T3 needs
  the driven-keystroke tests Marley used, ported to Zed's `TestAppContext`.

## Prong 2: the control plane (rustal, the harness, Rusty)

### The cast, updated

The orchestration-shell design named Forge as the brain. Forge is gone; the roles now land on
real services on this box:

| Role | Service | What it holds | How Marley reaches it |
|---|---|---|---|
| Work-state authority | rustal-brain (Postgres, MCP route with a project bearer, `rw` CLI) | tickets, sprints, runs, phases, gates, evidence, knowledge | MCP as a context server for agents; a native adapter for the work objects |
| Session substrate | rustal-harness `rh` runtime (owned tmux server, Unix socket, protocol v1) | workspaces, windows, panes, actor identities and incarnations, managed input with observer and controller claims, snapshot plus event subscriptions, durable messages, Codex sessions, evidence imports | `rh mcp`, Marley's fleet contract over MCP on stdio (the envelope, its changes by cursor, the session verbs under a write grant); its socket protocol later, for managed input (C3) |
| Personal brain and agent host | Rusty (`rusty-mcp` on `127.0.0.1:4174/mcp`, 85 tools; `rusty agent` sessions as transient user units with an NDJSON socket) | brain pages, tasks, memories, skills, secrets, the brain loop; Claude sessions that outlive any window | MCP for agents; the agent socket for transcripts and turn state |
| Judgment | the manager agent (Claude or Codex) | policy: dispatch, review, halt decisions | drives Marley over Marley's own MCP server |
| Mechanism shell | Marley | rendering, receipted verbs, hosting | this prong |

Two corrections to the old design fall out of reading the harness:

- Seat liveness comes from the harness, not from the brain. The harness already declares
  actor states (starting, idle, running, waiting, blocked, exited, failed, unknown) with a
  source, timestamp and sequence, and streams them with a consistent snapshot join. That is
  the `seat_events` feed the fleet design was waiting on, delivered by a different service
  than expected. The brain contributes labels (ticket, phase, gate) to the same seats.
- The harness has its own native-client milestone (M6, a mixed-pane client over its tmux
  terminals; a TUI `rh view` exists). Marley is the natural home for that client. As the
  harness's `docs/STATUS.md` gives it on 2026-09-29, its owner lifted the pause of 2026-09-14
  on 2026-09-22; M9 is complete (every agent session has one envelope in Marley's v1 shape,
  published as a durable feed, and `rh mcp` serves it with `session_answer`, `session_send`,
  `session_read`, `session_open` and `session_surface_to_human`, checked with Marley's own
  `marley_fleet`), and M10 (supervision, the manager's inbox) and M11 (the Brain in each
  session, declared tools, phase reports) are complete on fixture agents. Marley consumes what
  the harness serves and answers the harness's requests in its own tickets (D19's list).

### Design decisions

**D7. `marley_fleet` is the envelope for all three sources.** The reducer already folds a
generic event stream into a snapshot with attention and staleness. Each service gets a
projection: harness actor events and messages, Rusty agent-session events, brain run and
phase events as opaque labels. Marley never learns what a ticket is.

**D8. One adapter crate per service, all pure-core plus a masked transport, the house
pattern.** `marley_harness` (the harness client: first over `rh mcp`, the fleet contract on
stdio, then the `rh` socket protocol for managed-input controller claims with generations),
`marley_rusty` (the agent-socket client: attach, replay, send, status), and
`marley_brain` (MCP client for the brain's tools and resources, reusing Zed's
`context_server` transport rather than a new one).

**D9. Marley's own MCP server grows the tool families the docs already reserve.** On top of
`fleet_snapshot` and `session_surface_to_human`: `terminal_blocks`, `terminal_read`,
`terminal_run` (grant-gated, #556), `session_send`, `session_read`, `session_answer`,
`editor_open`, `editor_goto`, `editor_diff`, and later the browser family. Tool names are
`family_verb`, the form a Claude client can call (#491); the grant classes keep their dots
(`session.write`), since they are settings, never a tool name. The discovery
file lands in the workspace's `.mcp.json` shape so Claude Code and Rusty's agent host pick
it up. Deny-by-default grants stay; the settings live in Zed's settings tree.

**D10. Terminal passthrough for a harness seat is a Zed terminal in display-only mode fed by
the harness capture stream, promoted to a controller by claiming input.** Zed's
`TerminalType::DisplayOnly` plus `write_output` is exactly the seam: bytes from the harness
archive and live stream go in, and a claimed controller generation turns key input into
harness managed-input writes. Blocks work on these terminals too once the seat's shell runs
the same hooks.

**D11. Rusty stays Rusty.** Marley does not rebuild the knowledge workspace. It embeds
Rusty's sessions in the fleet rail, points agents at `rusty-mcp`, and uses the brain loop
tools for its own decisions. Whether Rusty's agent host and the harness runtime converge is
an open decision for the owner, not something Marley forces.

**D19. The harness is embedded in Marley and also runs standalone (Chad, 2026-09-25).**
Embedded, Marley runs `rh` as a process of its own, never linked in (§20's program boundary),
and speaks to it over `rh mcp`, the protocol a standalone harness on another host speaks over
SSH; one client serves both. Standalone, the harness keeps its own lifecycle outside Marley.
Packaging `rh` with Marley and starting its runtime is its own ticket after #534.

**D20. Marley is the SDK: it defines the outputs, providers fill them, Marley draws them
(Chad, 2026-09-30).** Agents, their work and their hosts reach Marley through typed contracts,
not through UI a provider sends: `marley.work/v1` from the central workflow store (rustal-brain
first) and `marley.host/v1` from a collector on each host, read over SSH. An agent and its host
are for the most part one thing; a click shows its snapshot in a Fleet panel on the right, and
opening it gives an Agent tab in the center. The draft is [fleet-contract.md](fleet-contract.md);
until the Rustal services serve it, Marley shows a pseudo provider's data. The SDK (types,
schemas, a conformance kit) follows the contract. Wave 1 (#607 to #611) builds it:
- #607 (shipped): the contract's types and the pseudo provider in `crates/marley_sdk`, and the
  Fleet panel in the right dock listing its agents by host;
- #608 (shipped): the selected agent's snapshot under the list;
- #609 (shipped): the Agent tab in the center;
- #610 (shipped): the host collector over SSH;
- #611 (shipped): the `marley.work/v1` clients over MCP and HTTP.

Wave 1 shipped on 2026-09-30. What Marley reads now waits on a store that serves the contract:
rustal-brain's own implementation is a brain ticket, when the brain resumes.

The harness's requests to Marley (`docs/planning/MARLEY_REQUESTS.md` in rustal-harness) and
their answers. The contract's home is the fork's `crates/marley_fleet` and `crates/marley_mcp`;
the harness still pins the gpui-era repository, whose `tool_name` joined with a dot, and moving
its pin is its own call.

- MREQ-001, tool names a Claude client can call: answered. Marley's server has served
  `family_verb` names since #491; #533 renamed the verbs in `marley_fleet`'s docs and in D9.
- MREQ-002, retry ids on `SendRequest` and `OpenRequest`: answered in #533. `SendRequest.delivery`
  and `OpenRequest.request`, optional and left out of the JSON when absent, and the receipts'
  values `SendReceipt { id, delivery, state, detail }` and `OpenReceipt { id, title, profile,
  request }`, as `rh mcp` returns them.
- MREQ-003, each verb's accepted payload in `marley_fleet`, `SurfaceAck` first: answered in #597.
  `SurfaceAck { surfaced }` moved from `marley_mcp` into `marley_fleet`, beside `ReadReceipt { id,
  start, end, total, lines, gaps }` and `AnswerReceipt { id, choice }`, as `rh mcp` returns them.
  The harness's `views` on a surface stay its own; a substrate's extra fields are ignored.
- MREQ-004, a typed `capabilities` map on `Session` and an optional `requires` on
  `SendRequest`: answered in #597. `Capabilities` is a name-to-value map, the harness's shape,
  left out of the JSON when empty; checking `requires` against it is C4's.
- The surface verb: the harness serves it as `session_surface_to_human` (TICKET-056), Marley's
  name since #370, so the two agree.

### Slices

| Slice | Delivers | Size |
|---|---|---|
| C0 | `marley_mcp` started by the app with the terminal read tools and the discovery file; an agent can list a pane's blocks (#491, pulled forward for the browser's agent tools; shipped) | M |
| C1 | `marley_harness` read side: subscribe, snapshot, events into `marley_fleet` (the harness's sessions in the rail over `rh mcp`, #534); a fleet rail panel with state chips, question cards and staleness. First piece shipped ahead of the harness (#519): a terminal's own Claude Code hook events, carried in-band by Marley's plugin, fold into `marley_fleet` and drive the terminal's rail row; #547 (shipped) publishes that snapshot to `fleet_snapshot`, ends a seat when its Claude Code leaves, and adds the update chip and the `no update in N m` form; the approvals inbox (#508, shipped) lists at the top of the rail every agent that waits on the user, the longest waiting first: Agent Panel prompts answered in place, Claude Code's permissions and questions, and the Browser tab's held clicks; per-turn diffs (#509, shipped) keep each turn of a terminal's Claude Code that changed the tree as a commit of its end on its start's checkpoint, pinned under `refs/marley/turns/`, listed under the terminal's row and opened in Zed's commit view; the rail ordered by attention, needs you first, with a collapsed project's counts and the order held under the pointer (#542, shipped) | L |
| C2 | Rusty sessions in the same rail through `marley_rusty`; brain-loop and Rusty tools in the default `context_servers` | M |
| C3 | Harness passthrough terminals (D10), observer first, controller claim second. A stopgap ahead of it: remote terminals on saved SSH hosts in a tmux session of Marley's own, which survive a dropped link and reattach with Rerun, with Claude Code's events from the host (#543, shipped) | L |
| C4 | Dispatch: `session_send` and `session_answer` over Marley's MCP, harness messages with delivery states rendered as chips; the retry ids and receipts are in `marley_fleet` (#533, shipped) | M |
| C5 | Work objects: brain tickets, runs and gate evidence as labels on seats and as a native pane; `rw` phase state in the status bar | M |
| S1 | The System One layer (#565, shipped): typed questions to a model (TypeSafe's Jev first) about states Marley builds, off by default and sent only for listed projects, masked; providers `typesafe`, `compatible`, `rules` and `replay`; the check and Decisions. The stop kind (#566, shipped) is its first use: an idle Claude Code's row says what the stop needs, the rules first and the model for the rest, off by default. The find tools (#567, shipped) are the second: `browser_find` and `terminal_find` answer an element or a line from words, the words first and the model for the rest, listed only while on. The stall kind (#569, shipped) is the third: a working Claude Code's row reads `looping?` from Marley's own rule and `stalled?` when a quiet turn with no tool using the CPU reads stuck to the model, off by default, never stopping the agent. The click consequence (#571, shipped) is the fourth: an agent's click that pays, deletes, sends or changes an account waits in the Browser tab for Allow or Refuse, the rules first and the model only adding pauses, by default only for agents with no prompt of their own. The inbox's risk chips (#568, shipped) are the fifth: each entry of the approvals inbox marked from the tool and what it acts on, ordered by level, then age, the rules first and the model only adding chips and raising levels, approving nothing. The typed line (#573, shipped) reads a line at a shell's prompt the rules leave open, after 250 ms without typing, and shows the reading after it. The question route (#570, shipped) is the sixth: each entry marked for you, for the manager, could proceed or unclear, the rules first from #568's chips, the model for the rest, ordering within a level in act and answering nothing; the manager's half waits on rustal-harness's M10 and #534 | M |

### Risks

- The harness moves fast (M9 to M11 in a week). Marley reads what it consumes from the
  harness's documents at a named state, answers its requests in its own tickets, and files what
  it needs as a harness ticket rather than patching around it. File it, do not fork.
- The brain's transitional `brain_pk_` bearer must never enter a log, a prompt or a file
  other than the operator's config; the never-logged-bearer rule from `marley_mcp` applies.
- Three services means three failure modes to render honestly: unconfigured, misconfigured,
  reconnecting. Marley already has the pure state machines for this; keep them.

## Prong 3: the browser service

### What "browser service" means here

Chad's 2026-08-11 amendment settled the target: one Chromium serves both the human's view
and the agent's eyes, because "the agent describes what the user is looking at" only works
when both are attached to the same session, cookies and DOM. The 2026-06-27 CEF study
showed real Chromium fidelity is achievable and priced the maintenance tax. The shipped
WKWebView pane was macOS-only and is not part of the Linux fork.

Reading "service" literally fixes the architecture: Chromium runs as its own user unit,
the way Rusty runs agent hosts and the harness runs its runtime, with a dedicated profile
and `--remote-debugging-port` on loopback. Marley attaches over CDP to render it; the
Playwright MCP, Claude Code, or Marley's own tool family attach to the same endpoint to
drive it. The browser outlives the IDE window, Chad's real Chrome is never touched (the
standing ops rule), and "the agent sees what you see" is true by construction.

Chad restated the goal on 2026-09-24: "the browser lives in marley and the marley ecosystem
has first class access to what the user sees and the project / forge brain. i want it to be
as close as 'seeing' what the user does as possible and cursor like experience." The page is
a tab in Marley's main area; every frame Chad sees and every click and key he makes passes
through Marley; agents reach the same page through Marley's MCP server, next to the brain
and Forge tools they already have, and act in the tab while he watches.

### What the 2026-09-24 probe answered

A throwaway headless Chromium 152 and a small CDP client answered the amendment's five
questions at the protocol level, before any Marley code (the answers Marley's side has to
confirm are in #488 and #489):

1. **IME and composition.** `Input.insertText` types composed text; `Input.imeSetComposition`
   shows an underlined composition in the field and `insertText` commits over it (152 has no
   `imeCommitComposition`). gpui delivers a finished compose sequence as a key with its
   character. Live Japanese or Chinese through an input method stays untested: Chad chose
   "compose only, CJK later".
2. **Latency.** A key dispatched over CDP reaches a frame in 6 to 7 ms on the dev box; twenty
   wheel steps gave twenty frames. Through Marley (#489), from the key to the decoded frame, the
   median is 20 ms and the 95th percentile 36 ms in a debug build, once its JPEG decoder is
   built optimized; unoptimized, a frame took 130 ms to decode.
3. **The inspect highlight** (`Overlay.setInspectMode`, with its accessibility tooltip) is in
   the screencast frames.
4. **Cross-site iframes** render in the frame and take input routed through the main page's
   session. The main frame's `Accessibility.getFullAXTree` leaves them out; their trees come
   from the sessions `Target.setAutoAttach` opens.
5. **Coordinates.** Frames come at exactly the viewport `Emulation.setDeviceMetricsOverride`
   sets; each frame's metadata carries the viewport in DIP, the scroll offsets in CSS pixels
   and the pinch scale; input coordinates are viewport CSS pixels. Frames stay at 1× when a
   larger device scale factor is emulated, so a HiDPI screen needs
   `--force-device-scale-factor` when Chromium starts (the dev box runs at scale 1).

Also found: `<select>` popups do not render in headless frames, and `/usr/bin/chromium` on
Arch is a launcher that adds the user's `~/.config/chromium-flags.conf` (on the dev box,
Omarchy's three extensions and the keyring password store); `/usr/lib/chromium/chromium` is
the browser itself.

### Design decisions

**D12. CDP screencast into a gpui image, not a native child window.** `Page.startScreencast`
frames decode to `RenderImage`s drawn with `img()`; gpui overlays layer above the page for
free, which deletes the hide-shim coupling the WKWebView pane needed. Input goes back
through `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent` and `Input.insertText`.
The workspace already depends on `async-tungstenite` and `image`. CEF off-screen rendering
stays the named revisit if screencast latency or fidelity fails the spike.

**D13. `marley_browser` is a pure protocol core plus thin adapters.** The CDP client
(JSON-RPC over WebSocket, target and session management), frame coordinate math
(device pixel ratio, page scale, scroll offset from frame metadata), locator ranking, the
annotation model and the ring-buffer trace are kept free of gpui views; the socket, the
decoder and the service start are adapters. The Browser tab itself is a `workspace::Item` in
`marley_workbench`, beside the terminal features, which is how `marley_terminal` and the
workbench already split. Since #483 each slice is proven by an e2e run, not unit tests.

**D14. The three pillars from the amendment are the product, in this order.**
A: the element picker (`Overlay.setInspectMode`, a durable locator bundle with role, name,
listeners and their source locations, and the Marley-only move of opening the listener's
source file in the editor). B: the annotation layer rendered natively and anchored in page
coordinates. C: the flight recorder, a rolling structured trace of input, frames, console
and network that "record this" saves retroactively.

**D15. Security is designed in the first slice.** Chromium's debugging endpoint listens on
127.0.0.1 only and takes no token (Chromium offers none); it refuses WebSocket connections
from web pages (their `Origin`) and requests whose `Host` is not local. A TCP port on loopback
checks no user, though, so every local process can reach it, other users' included (this
decision first said "processes of the same user", which holds for the profile's files and not
for the port, #524). Since #583 Chromium speaks CDP on its pipe to Marley's relay and listens on
no port; other clients reach the relay's loopback WebSocket with a token only the user can read. Agents are meant to come
through Marley's MCP server (D17), which has its per-boot bearer. No tool evaluates
script in the page, agent navigation takes only `http` and `https`, and the network and trace
readers redact headers and secret-looking query values before an agent or a file sees them.
The 2026-07 idea of a navigation allowlist per project gives way to two checks an agent
cannot skip: the client's approval of each write call (Claude Code asks by default) and the
Browser tab, where every agent action happens in front of Chad. Since #571 a third, for agents
with no prompt of their own, holds a consequential click in the tab until Chad allows it.

**D16. Marley starts the Chromium (Chad, 2026-09-24).** The first `marley: open browser`
starts it as a transient user unit (`systemd-run --user --collect`, Rusty's pattern), one
unit per Marley data directory, so an e2e run gets its own. It runs the Chromium binary
itself (`/usr/lib/chromium/chromium` where the distribution wraps it in a launcher) with
only Marley's flags, headless, with its profile under Marley's data directory and
`--remote-debugging-port=0`; Chromium writes the port it chose into the profile's
`DevToolsActivePort`, where Marley and any other CDP client find it. Since #583 the unit's main
process is Marley's relay, Chromium runs on its pipe, and Marley and other CDP clients find the
relay: its Unix socket, and `relay.json` with the loopback WebSocket's address and token. The unit outlives
Marley's windows and Marley itself; it ends at logout. Since #507 (Chad, 2026-09-25) it is one
unit per project: each project, as the rail groups it, has a profile of its own under
`browser/projects/<key>/`, keyed on its main folders, so its logins stay its own and a linked
worktree shares its repository's. A project's unit starts with its first Browser tab, outlives
Marley's quit, and stops when the project is removed; Marley asks Chromium to close over CDP
before it stops a unit. The one profile of earlier builds moved to the first project whose
Chromium started.

**D17. Agents reach the browser through Marley's MCP server.** C0 (#491) starts `marley_mcp`
in the app, and the Marley Claude Code plugin (#482) carries a small stdio bridge to it, so
every Claude Code session on the machine finds Marley's tools while Marley runs, and an empty
server when it does not. The `browser_*` tools (#492) read what Chad sees (the page, its
accessibility snapshot including cross-site iframes, the frame on his screen, the console and
network) and act in the same tab. Since #524 (Chad, 2026-09-25) programs outside Marley that
the user allows by name reach the browser tools too, each with a token of its own, new at each
start, and a grant to read pages or also to act in them over an explicit list of tools; a tab
one drives names it, and Cut Off refuses its token at once. Since #584 a client on another
machine runs Marley's bridge on this one over SSH, from the line Browser Clients shows; nothing
new listens.

**D18. Scenarios that click run Marley in a headless sway (#487).** Hyprland cannot send a
pointer event to one window, and a real click would move Chad's pointer. A headless sway with
a virtual pointer and keyboard of its own takes clicks, drags and the wheel with nothing
reaching his desktop.

### Slices

| Slice | Ticket | Delivers | Size |
|---|---|---|---|
| (tooling) | #487 | e2e scenarios that click, drag and scroll, in a headless sway (shipped) | S |
| B0a | #488 | `marley_browser`: Marley's Chromium as a transient unit, the CDP client, the screencast; the Browser tab shows the page at its size, with its title (shipped) | M |
| B0b | #489 | Typing and clicking in the page: keys, compose and input-method text, the mouse and the wheel, the clipboard; the input-to-frame time logged (shipped) | M |
| B1a | #490 | The address bar, back, forward, reload and stop, the loading state, JavaScript dialogs (shipped) | M |
| C0 | #491 | Prong 2's C0, pulled forward: Marley's MCP server in the app, the plugin's bridge, and the terminal block tools (shipped) | M |
| B2 | #492 | The `browser_*` tools: look, snapshot, console, network; navigate, click, type, press, scroll (shipped) | M |
| B1b | #493 | Tabs as page targets: pages the page, an agent or Ctrl+T opens appear as tabs, a closed tab closes its page, the tools take `tab` (shipped) | M |
| B1c | #494 | The Browser tabs saved with the workspace and back after a relaunch, on their pages or at their URLs (shipped) | S |
| B1d | #495 | Marley's own `<select>` lists, drawn under the select in any frame, for the user's opening (shipped) | S |
| B3a | #496 | Pillar A: pick mode, the durable bundle (ranked locators, AX node, listeners, blocking styles), picks staged for Chad to caption and send (shipped) | L |
| B3b | #497 | A picked element's listener source, through its source map, opened in the editor at the line (shipped) | M |
| B4 | #498 | Pillar B, annotations drawn by gpui and anchored in page coordinates (shipped) | M |
| B5 | #499 | Pillar C, the flight recorder and "record this" (shipped) | L |
| B2b | #574 | The browser tools act in the caller's project (#520's caller): a call with no tab takes the tab of the agent's project the user focused last, `browser_navigate` opens one beside the agent's terminal when the project has none, and `browser_tabs` names each tab's project and the default (shipped) | M |
| B6a | #503 | A terminal's local URLs open in a Browser tab of its project, Shift+Ctrl+click and `marley.terminal_links` choosing the other place; an OSC 8 link on a plain click; a terminal over ssh keeps the system browser; the footer offers a dev server's printed URL while its port listens (shipped) | M |
| B6b | #579 | A plain click on a terminal URL opens a menu (a Browser tab, the system browser, Copy Link), the right-click menu starts with it, the default is offered until chosen, and a URL a program wrapped at the edge or drew in a box opens whole (shipped) | M |
| B6c | #561 | Programs that open a browser through `BROWSER` open a local URL in a Browser tab of their project: Marley's opener in every local terminal, `browser_open_url`, the system browser for the rest (shipped) | S |
| B7a | #504 | Browser tabs as rows of their project in the rail: the page's icon, a spinner while it loads, host and port, the tray's picks and the page's annotations, and the agent's mark until the user looks; a click, the keys, the filter and a close button (shipped) | M |
| B3c | #518 | A fuller pick for agents: the element's HTML (no scripts, field values or URL queries, secret-looking attributes replaced, 4,096 characters), sixteen computed styles, the siblings' texts and the selection, and on a React dev build the component chain and the file and line it was written at (React 19 through the source maps); every field redacted whole before it is cut (shipped) | M |
| B3d | #505 | Pick, fix, check: a pick found again after a change (test id, id, role and name, text, CSS path; the nearest of several), scrolled into view, the crop at the pick beside the crop now and what changed, the tray's verdict, and `browser_check_pick`; no generated id in a locator (shipped) | M |
| B5b | #506 | A recording drafted as a Playwright test the project keeps: clicks, fills and presses recorded with the target's locators at the event, ordinary fields' text kept and secret fields' not, and `browser_draft_test` (the first locator found alone, `baseURL`, `toHaveURL` after each navigation, secrets from the environment, a path in the project) (shipped) | M |
| B5c | #523 | Playwright scripts kept in Marley, per project and for every project, and run on a Browser tab: the Scripts tray, a new script from the template, Marley's runner attached over the tab's Chromium in a terminal beside the tab, `playwright-core` installed by the first run, a failed run's minute saved as a recording (shipped) | M |
| B7c | #581 | Clear Browser Data for one project: the rail's project menu and `marley: clear project browser data` ask, then close the project's tabs, close and stop its Chromium and delete its profile, keeping `project.json`; a toast reports it (shipped) | S |
| B7b | #507 | A Chromium and a profile per project: logins (cookies, `localStorage`, IndexedDB) kept apart and across restarts, a linked worktree on its repository's, the unit started by the project's first tab, kept at a quit and stopped when the project is removed (the rail's Remove Project), the old profile moved to the first project, the tools across every project's browser (shipped; clearing one project's data is #581) | L |
| B8a | #524 | Trusted outside clients, slice 1: named clients with per-start tokens in endpoint files of their own, a read or act grant over an explicit list of browser tools, the tab's "Driven by" mark and Cut Off, owned sessions, and the MCP server's read bounded before authentication (shipped) | L |
| B8b | #583 | Slice 2: Chromium's DevTools off TCP, on its pipe behind Marley's relay, which gives each client a browser session, serves Marley over a 0600 Unix socket and other clients a loopback CDP WebSocket that takes a token it mints at each start (shipped) | L |
| B8c | #584 | Slice 3: a client on another machine runs Marley's bridge here over SSH, from the line Browser Clients shows and copies; a cut-off client is told it is not allowed (shipped) | S |

Wave 2 was specced on 2026-09-25, once wave 1 had landed (#496 to #499; the shelf note is
`docs/planning/design-notes/browser-wave-2-shelf.md`).

### Risks

- Japanese and Chinese through a live input method are untested (Chad's call). The input
  path is built (#489); the first user who needs it tests it.
- HiDPI frames need `--force-device-scale-factor` at the service's start; a Marley window
  moved to a screen of another scale gets soft frames until Chromium restarts.
- Headless Chromium draws no browser UI, so everything that is browser UI in a normal
  browser is Marley's to draw. `<select>` lists (#495) and JavaScript dialogs (#490) are drawn;
  file choosers, downloads, context menus and the page's cursor shape are not yet.
- The unit outlives Marley, so a Chromium that crashed or was stopped must read as such in
  the tab, with a way back (the #406 lesson: transport failures arrive as silence).
- Each open project's Chromium holds about 290 MB, and no cap stops an idle one (#507).
- A unit stopped by a signal, at logout say, loses the cookies Chromium set in its last 30
  seconds: Chromium writes them every 30 seconds from its network process, which systemd kills
  once the browser exits. Marley's own stops close Chromium over CDP first (#507).
- Screencast is frame-streamed; fine for browsing and agent work, wrong for video. Accepted.

## Cross-cutting

- **Licensing.** The fork is GPL-3.0-or-later where Zed is; the `marley_*` crates keep
  `MIT OR Apache-2.0` and stay distinct from upstream code; the brain and the manager agent
  are separate programs speaking MCP, so the AGPL section 13 concern from the Warp study never
  reaches them; CDP, Chromium and the harness protocol are permissive or Ignibyte's own.
- **Gates.** Zed's `./script/clippy` and fmt on every change, with the meta-gates. The
  Marley pure-core discipline (logic in gpui-free modules, adapters thin) continues in the new
  crates. Since 2026-09-23 no gate runs tests (#483): the 100% line-coverage floor that came
  over with the workflow port (2026-09-18) and the end-of-sprint mutation run retired, and each
  change is proven by an e2e visualization test on the real Marley (CONSTITUTION §7).
- **Upstream merges.** Keep every change additive: new crates, new events, new panels, one
  fork branch of alacritty. Touching `terminal_element.rs` for stage-two rendering is the
  one place merge conflicts are likely; isolate it behind a `blocks` module. Every change
  outside Marley-owned paths gets a row in [zed-touchpoints.md](zed-touchpoints.md).
- **Order across prongs.** The workbench shell comes before all three
  ([workbench-shell.md](workbench-shell.md), Chad's call on 2026-09-22), because Blocks,
  fleet rows and the browser pane all render into it. Then T0 and T1 (nothing else has value
  until Blocks exist),
  then C0 (agents get Blocks as data), then B0 and C1 in parallel (both are read-only and
  independent), then T2 to T4, C2 to C4, B1 to B2, and the long tails T5, T6, C5, B3 to B5.
  On 2026-09-24 Chad moved the browser first: prong 3's wave 1 (#487 to #495) runs now, with
  C0 pulled forward inside it for the agent tools; the terminal's T2 to T6 and the rest of
  prong 2 follow.
- **Tickets.** Each slice becomes a ticket in this repo when it starts; decisions get
  recorded in the brain with `brain_decide`, as this plan is.

## Open decisions for Chad

1. ~~The alacritty fork branch: an `Ignibyte/alacritty` fork of Zed's fork, or a patch
   carried in this repo through `[patch]`.~~ Decided 2026-09-23: the copy lives in this
   repository, `vendor/alacritty_terminal` (#461).
2. Stage two of the terminal (native headers, PS1 hidden) as the Warp look, or stop at stage
   one and keep the shell's own prompt visible.
3. Whether Rusty's agent host and the harness runtime should converge, and which one Marley
   treats as the seat substrate of record.
4. ~~Chromium packaging: a Marley-owned user unit, or one shared with the Playwright MCP.~~
   Decided 2026-09-24: Marley starts it (D16); the Playwright MCP, agent-browser and
   Claude Code attach to the endpoint in its `DevToolsActivePort`.
5. ~~The written three-prong plan Chad referred to for the browser.~~ Resolved 2026-09-24:
   his 2026-09-18 message ("Look at rusty we havd a plan for a 3 prong approach for this")
   named the three prongs of this plan, the terminal, the control plane and the browser; no
   separate browser plan exists in the old repository, its history, Rusty or the brain. The
   browser's own three parts are the 2026-08-11 amendment's pillars (D14).
6. Whether picks go to the agent at once or wait for Chad to caption them (B3a), and whether
   annotations persist across sessions (B4). Wave 2's specs take staged picks and
   session-only annotations as the defaults unless Chad says otherwise; #496 shipped staged
   picks and #498 session-only annotations.
