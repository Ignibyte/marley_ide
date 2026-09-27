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
| T0 | Split at its promotion: the vendored `alacritty_terminal` (#461, T0a, shipped); `Event::ShellHook` from the event loop, with unit tests on recorded byte streams (#462, T0b, shipped); the anchored `BlockList` on `Terminal` and a `blocks()` accessor (#464, shipped); the hook scripts and their injection, bash first (#463, T0c, shipped), then zsh (#465, shipped) and fish (#466) | M |
| T1 | Stage-one rendering: gutter, pill, wash (#470, T1a, shipped); hover copy/rerun (#474, T1b, shipped); block navigation keys (#473, T1c, shipped); the content drawn against the bottom edge, so the prompt sits on the last row (#476, T1d, shipped) | M |
| T2 | Block-scoped path links (resolve against the block's cwd) and jump-to-first-failure | S |
| T3 | The prompt editor with history ghost text and the raw-passthrough ladder; history ghost text on the shell's own prompt, → to take it (#484, T3a, shipped) | L |
| T4 | Tasks and runnables as Blocks; failed Blocks into diagnostics | M |
| T5 | Stage-two rendering: native header rows, PS1 hidden, Warp density | L |
| T6 | Completions in the prompt (paths, history, tasks) and command-line colouring | M |
| T7 | CLI agents in the terminal, after Warp's agent toolbelt: the agent bar with the folder and branch (#477, T7a, shipped); desktop notifications from OSC 9 and 777 (#478, T7b, shipped) and a Claude Code plugin that sends them (#482, T7b, shipped); Attach File (#479, T7c, shipped); voice through Voxtype (#480, T7d, shipped); rich input, a Zed editor for the agent's prompt and the agent-first half of T3 (#481, T7e, shipped) | M |

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
| Session substrate | rustal-harness `rh` runtime (owned tmux server, Unix socket, protocol v1) | workspaces, windows, panes, actor identities and incarnations, managed input with observer and controller claims, snapshot plus event subscriptions, durable messages, Codex sessions, evidence imports | a Rust client of its socket protocol |
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
  terminals; a TUI `rh view` exists). Marley is the natural home for that client. The harness
  is paused at the owner's request since 2026-09-14, so Marley consumes the protocol as it
  is and files anything missing as a harness ticket rather than patching around it.

### Design decisions

**D7. `marley_fleet` is the envelope for all three sources.** The reducer already folds a
generic event stream into a snapshot with attention and staleness. Each service gets a
projection: harness actor events and messages, Rusty agent-session events, brain run and
phase events as opaque labels. Marley never learns what a ticket is.

**D8. One adapter crate per service, all pure-core plus a masked transport, the house
pattern.** `marley_harness` (the `rh` protocol client: subscribe with resume, managed-input
controller claims with generations, message send and receive with delivery ids),
`marley_rusty` (the agent-socket client: attach, replay, send, status), and
`marley_brain` (MCP client for the brain's tools and resources, reusing Zed's
`context_server` transport rather than a new one).

**D9. Marley's own MCP server grows the tool families the docs already reserve.** On top of
`fleet.snapshot` and `session.surface_to_human`: `terminal.blocks`, `terminal.read`,
`terminal.run` (grant-gated), `session.send`, `session.read`, `session.answer`,
`editor.open`, `editor.goto`, `editor.diff`, and later the browser family. The discovery
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

### Slices

| Slice | Delivers | Size |
|---|---|---|
| C0 | `marley_mcp` started by the app with the terminal read tools and the discovery file; an agent can list a pane's blocks (#491, pulled forward for the browser's agent tools; shipped) | M |
| C1 | `marley_harness` read side: subscribe, snapshot, events into `marley_fleet`; a fleet rail panel with state chips, question cards and staleness. First piece shipped ahead of the harness (#519): a terminal's own Claude Code hook events, carried in-band by Marley's plugin, fold into `marley_fleet` and drive the terminal's rail row; #547 (shipped) publishes that snapshot to `fleet_snapshot`, ends a seat when its Claude Code leaves, and adds the update chip and the `no update in N m` form | L |
| C2 | Rusty sessions in the same rail through `marley_rusty`; brain-loop and Rusty tools in the default `context_servers` | M |
| C3 | Harness passthrough terminals (D10), observer first, controller claim second | L |
| C4 | Dispatch: `session.send` and `session.answer` over Marley's MCP, harness messages with delivery states rendered as chips | M |
| C5 | Work objects: brain tickets, runs and gate evidence as labels on seats and as a native pane; `rw` phase state in the status bar | M |

### Risks

- The harness protocol is version 1 and its owner paused work; Marley may hit a missing
  verb (peer messaging is listed as future). File it, do not fork.
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
from web pages (their `Origin`) and requests whose `Host` is not local, so the exposure is to
processes of the same user, who could already read the profile directory. Agents are meant
to come through Marley's MCP server (D17), which has its per-boot bearer. No tool evaluates
script in the page, agent navigation takes only `http` and `https`, and the network and trace
readers redact headers and secret-looking query values before an agent or a file sees them.
The 2026-07 idea of a navigation allowlist per project gives way to two checks an agent
cannot skip: the client's approval of each write call (Claude Code asks by default) and the
Browser tab, where every agent action happens in front of Chad.

**D16. Marley starts the Chromium (Chad, 2026-09-24).** The first `marley: open browser`
starts it as a transient user unit (`systemd-run --user --collect`, Rusty's pattern), one
unit per Marley data directory, so an e2e run gets its own. It runs the Chromium binary
itself (`/usr/lib/chromium/chromium` where the distribution wraps it in a launcher) with
only Marley's flags, headless, with its profile under Marley's data directory and
`--remote-debugging-port=0`; Chromium writes the port it chose into the profile's
`DevToolsActivePort`, where Marley and any other CDP client find it. The unit outlives
Marley's windows and Marley itself; it ends at logout.

**D17. Agents reach the browser through Marley's MCP server.** C0 (#491) starts `marley_mcp`
in the app, and the Marley Claude Code plugin (#482) carries a small stdio bridge to it, so
every Claude Code session on the machine finds Marley's tools while Marley runs, and an empty
server when it does not. The `browser_*` tools (#492) read what Chad sees (the page, its
accessibility snapshot including cross-site iframes, the frame on his screen, the console and
network) and act in the same tab.

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
