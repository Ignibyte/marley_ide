# Subsystem 03 — Terminal & Session Core (PTY, Blocks)

Part of the Marley architecture docs (round 1). Marley is forked from Warp (warpdotdev/warp).

> **Re-review (round 3, 2026-07-12 · verified @ M15).** This doc was written **2026-06-27, pre-M12.2**,
> as a read of **Warp's** code (the reference Marley reimplements) and carried **no provenance tags**.
> The Warp mechanics below (PTY / Blocks / DCS hooks) still describe Warp accurately and are unchanged;
> what is now stale is only the forward-looking *"Marley note / relevance"* asides, written before
> Marley had its own terminal. **Marley has since shipped its block-terminal** — `crates/terminal_blocks`,
> a clean-room reimplementation on the upstream `alacritty_terminal` crate — and the **terminal→editor
> link half**: clickable `file:line:col` (**#196**, text-scan) and **OSC 8** explicit hyperlinks
> (**#214**, `StyledRun.hyperlink`). Terminal-session-core is **Marley's strongest subsystem**. Two new
> sections at the bottom carry the re-review: [**Marley status @ M15**](#marley-status--m15) and
> [**Provenance & licensing**](#provenance--licensing). Cross-read the Zed terminal doc
> [`../../zed_architecture/subsystems/08-terminal-tasks-fusion.md`](../../zed_architecture/subsystems/08-terminal-tasks-fusion.md)
> — Zed has an editor + a plain terminal but **no command-block model at all**, so Marley's Blocks are
> the terminal↔editor **fusion wedge**.

> **TL;DR for Marley goal (b) — "spawn a session, write to it, read output back":**
> The byte-level engine lives in **`app/src/terminal/`**, not in the `crates/*` named in the
> assignment. The three entry points you want are:
> - **Spawn:** `Pty::new(options, …)` → `PtySpawner::spawn_pty` → `local_tty::spawn` (`nix::pty::openpty` + fork)
>   — `app/src/terminal/local_tty/unix.rs`, `app/src/terminal/local_tty/spawner.rs`.
> - **Write:** `PtyController::write_bytes` / `write_command` / `write_agent_bytes`
>   → `Message::Input(bytes)` over a `mio_channel` → `EventLoop::pty_write` → `pty.writer().write()`
>   — `app/src/terminal/writeable_pty/pty_controller.rs`, `app/src/terminal/local_tty/event_loop.rs`.
> - **Read:** a dedicated **"PTY reader"** thread runs `EventLoop::pty_read` → `pty.reader().read()`
>   → `ansi::Processor::parse_bytes` mutates the shared `TerminalModel` and fires `send_wakeup_event()`
>   — `app/src/terminal/local_tty/event_loop.rs`.
>
> The `crates/*` in the assignment (`warp_terminal`, `warp_core`, `warp_tui`, `local_control`,
> `command`, `command-signatures-v2`, `ipc`) are the **model / protocol / plumbing layer** around
> that engine. Each is documented below with its real role and its relevance to the engine.

---

## 1. Where the session core actually lives

The assignment lists seven workspace crates, but a repo-wide search for the PTY primitive
(`nix::pty::openpty`, `TIOCSWINSZ`, fork/`pre_exec`) returns exactly one hit:
`app/src/terminal/local_tty/unix.rs`. The terminal **session engine is in the top-level `app`
crate**, under `app/src/terminal/`. The named `crates/*` provide:

| Crate | Real role | Relation to the session engine |
|-------|-----------|--------------------------------|
| `warp_terminal` | Shared, lower-level terminal **model** primitives (grid storage, ANSI/escape parsing, `BlockId`/`BlockIndex`, key/mouse modes). Adapted from Alacritty (`crates/warp_terminal/src/model/LICENSE-ALACRITTY`). | Foundation types consumed by the app's richer `app/src/terminal/model/*`. |
| `warp_core` | App/runtime core: `SessionId`, channel config, paths, feature flags, telemetry. | Defines `SessionId` (the per-subshell identity used everywhere). |
| `warp_tui` | Headless TUI front-end binaries; mostly an editor-backed input view. | A *consumer* front-end; defers to `app`'s `warp::run_tui()`. Not the PTY engine. |
| `local_control` | External automation RPC ("warpctrl"): action catalog, wire envelopes, discovery, auth. | The existing **out-of-process control seam** (create tab/session, insert input). Higher-level than raw bytes. |
| `command` | Drop-in `std::process::Command` / `async_process::Command` wrapper (Windows `CREATE_NO_WINDOW`). | The `Command` builder used by `spawn_command_in_pty`. |
| `command-signatures-v2` | `rust-embed` bundle of a JS command-signature parser (`COMMAND_SIGNATURES_JS`). | Static asset; not on the spawn/read/write path. |
| `ipc` | Generic request/response IPC over Unix domain sockets / named pipes. | Transport used by the plugin host and (conceptually) the terminal server. |

A Marley implementer expanding the UI to spawn/drive sessions must integrate with
`app/src/terminal/{writeable_pty,local_tty}` — the `crates/*` alone are insufficient.

---

## 2. PTY abstraction (the core traits & types)

Defined in `app/src/terminal/local_tty/mod.rs`:

- **`PtyOptions`** (`mod.rs:91`) — the full spawn spec: `size: SizeInfo`, `shell_starter: ShellStarter`,
  `start_dir`, `env_vars: HashMap<OsString,OsString>`, SSH-wrapper flags, `honor_ps1`,
  `node_version_chip_enabled`, `close_fds`. `Serialize`/`Deserialize` so it can be shipped to the
  out-of-process terminal server.
- **`EventedReadWrite`** trait (`mod.rs:47`) — abstraction over mio readiness: `register`/`reregister`/
  `deregister`, `reader() -> &mut Reader`, `writer() -> &mut Writer`, `read_token`/`write_token`.
- **`EventedPty: EventedReadWrite`** trait (`mod.rs:73`) — adds child lifecycle:
  `child_event_token()`, `next_child_event() -> Option<ChildEvent>`, `on_resize(&SizeInfo)`,
  `kill(self)`. `ChildEvent::Exited` is the race-free SIGCHLD-driven exit signal on Unix.

The concrete Unix implementation is **`Pty`** (`app/src/terminal/local_tty/unix.rs:156`):

```rust
pub struct Pty {
    pty_handle: Box<dyn PtyHandle>,  // owns/kills the child process
    fd: File,                        // the PTY *leader* (master) fd — used for BOTH read and write
    token: mio::Token,               // PTY_TOKEN
    signals: Signals,                // SIGCHLD via signal_hook_mio
    signals_token: mio::Token,       // SIGNALS_TOKEN
}
```

Key detail: `reader()` and `writer()` both return `&mut self.fd` — a single leader fd serves
both directions (lines 612–630). `on_resize` issues `ioctl(fd, TIOCSWINSZ, &winsize)` (line 657).
`kill` drops the fd then calls `pty_handle.kill()` (line 667).

Windows has a parallel ConPTY implementation under `app/src/terminal/local_tty/windows/`
(`PseudoConsoleChild`, `conpty_api.rs`); `mod.rs` re-exports per-OS via `cfg`.

---

## 3. Session SPAWN — exact path

`SessionId` itself is defined in **`warp_core` (`crates/warp_core/src/session_id.rs`)**:
`pub struct SessionId(u64)` — "each bootstrapped subshell (including SSH sessions) gets its own
`SessionId`." It is assigned during shell bootstrap (carried in DCS hook payloads, §6), not at
fork time.

Spawn call chain (Unix), top to bottom:

1. **`TerminalManager::spawn_pty`** (`app/src/terminal/local_tty/terminal_manager.rs:619`) — reads
   settings (shell debug, honor PS1, SSH warpification, Node chip), grabs the grid `size` from the
   model, builds `PtyOptions`, and calls `Pty::new`.
2. **`Pty::new(options, is_crash_reporting_enabled, ctx)`** (`unix.rs:532`) — installs the SIGCHLD
   `Signals` handler, then delegates to the spawner singleton:
   ```rust
   let (PtySpawnResult { pid, leader_fd }, pty_handle) =
       PtySpawner::handle(ctx).update(ctx, |s, ctx| s.spawn_pty(options, …, ctx))?;
   ```
   Then sets the leader fd non-blocking, wraps it in `File`, and calls `on_resize(&size)`.
3. **`PtySpawner::spawn_pty`** (`app/src/terminal/local_tty/spawner.rs:170`) — a `SingletonEntity`.
   Tries the **terminal server** first; on failure (except deterministic `E2BIG`) falls back to a
   direct spawn. Emits a `PtySpawned { mode }` telemetry event (`TerminalServer` /
   `FallbackToDirect` / `Direct`).
   - **Server path** (`spawn_pty_via_server`, `spawner.rs:256`): `TerminalServerClient::spawn_pty`
     (`local_tty/server/client.rs:43`) sends `PtyOptions` over a Unix domain socket; the server
     process (spawned very early via `PtySpawner::new` → `TerminalServer::new`) does the fork and
     returns the **leader fd over `SCM_RIGHTS`** (see `server/protocol.rs` `SpawnShellResponse`
     handling, and `server/mod.rs` doc: the server exists to fork shells from a *clean* fd state).
     Returns a `ServerOwnedPtyHandle`.
   - **Direct path** (`spawn_pty_directly`, `spawner.rs:234`): calls `local_tty::spawn(options)` in
     the current process, returns a `DirectPtyHandle { child: std::process::Child }`.
4. **`local_tty::spawn`** (`unix.rs:170`) — dispatches Docker-sandbox sessions early, otherwise
   `build_host_shell_command` (`unix.rs:224`, sets `TERM=xterm-256color`,
   `TERM_PROGRAM=WarpTerminal`, `WARP_*` env, history sentinels, etc.) then
   **`spawn_command_in_pty`** (`unix.rs:403`):
   - `make_pty(size.to_winsize())` (`unix.rs:46`) → `nix::pty::openpty(...)` returns
     `(leader, follower)` fds, both set `FD_CLOEXEC`.
   - applies UTF-8 termios (`IUTF8`).
   - installs a `command.pre_exec` hook (runs **after fork**, before exec): sets the session/
     controlling terminal (`setsid` + `ioctl(TIOCSCTTY)`), dup2's the follower onto
     stdin/stdout/stderr, optional `close_fds`, Linux OOM rebias.
   - `command.spawn()` → returns `PtySpawnInfo { result: PtySpawnResult { pid, leader_fd }, child }`.

Spawn-result/handle types: **`PtySpawnResult { pid: u32, leader_fd: i32 }`** (`unix.rs:164`),
**`PtySpawnInfo`** (`spawner.rs:103`), **`PtyHandle`** trait (`spawner.rs:16`: `pid`,
`has_process_terminated`, `kill`) with `DirectPtyHandle` and `ServerOwnedPtyHandle` impls.

After spawn, `TerminalManager::start_pty_event_loop` (`terminal_manager.rs:691`) constructs
`EventLoop::new(model, channel_event_proxy, pty, rx)` and calls `event_loop.spawn()` to launch the
reader/writer thread (§5).

---

## 4. Session WRITE — exact path

The write coordinator is **`PtyController<T: EventLoopSender>`**
(`app/src/terminal/writeable_pty/pty_controller.rs:83`), a `warpui` model entity. It serializes
writes from three sources (user commands, raw input, in-band/agent commands) against the line
editor's readiness state.

Public write entry points (all take a `&mut ModelContext<Self>`):
- **`write_bytes<B: Into<Cow<'static,[u8]>>>(bytes, ctx)`** (`pty_controller.rs:610`) — non-command
  passthrough input (long-running command / alt-screen).
- **`write_command(...)`** (`pty_controller.rs:511`) — a user command; appends ENTER
  (`COMMAND_ENTER = CR,LF`), tracks line-editor state, supports bracketed paste.
- **`write_agent_bytes(bytes, mode, ctx)`** (`pty_controller.rs:591`) — AI-agent input with an
  `AIAgentPtyWriteMode`.
- **`write_end_of_transmission_char`**, **`resize_pty(size_update, ctx)`** (sends
  `Message::Resize`), **`shutdown_pty`** (sends `Message::Shutdown`).

Internal vocabulary: **`PtyWrite`** enum (`pty_controller.rs:40`: `Command` / `Bytes` /
`AgentInput` / `RunNativeShellCompletions`) is queued in `pending_writes: VecDeque<PtyWrite>` and
drained by `execute_next_queued_write`. Each drained write becomes a
**`Message::Input(Cow<'static,[u8]>)`** delivered via `send_message_to_event_loop`
(`pty_controller.rs:686`) over `event_loop_tx: T` where `T: EventLoopSender` (default
`mio_channel::Sender<Message>`). A failed send emits `PtyControllerEvent::PtyDisconnected`.

The UI-facing abstraction that feeds the controller is **`PtyIntent`**
(`app/src/terminal/writeable_pty/terminal_surface.rs:23`): `CtrlD`, `ShutdownPty`, `WriteBytes`,
`WriteAgentInput`, `Resize`, `ExecuteCommand`, `RunNativeShellCompletions`. A `TerminalSurface`
view projects its own event type into an `Option<PtyIntent>` via the `PtyIntentEvent` trait — this
is the **narrow seam `TerminalManager` uses to drive the PTY without knowing the concrete UI**.
*(Marley note: this is the cleanest place for a custom Ignibyte panel to inject writes — implement
`PtyIntentEvent` / route through `PtyController::write_bytes`.)*

**`Message`** (`app/src/terminal/writeable_pty/message.rs`): `Input(Cow<[u8]>)`, `Shutdown`,
`ChildExited` (Windows), `Resize(SizeInfo)`.

On the loop side, `EventLoop::drain_recv_channel` (`event_loop.rs:162`) pushes `Message::Input`
bytes onto `State.write_list`; `EventLoop::pty_write` (`event_loop.rs:283`) drains that list and
calls `self.pty.writer().write(remaining_bytes)` against the leader fd, handling partial writes
(`Writing { source, written }`) and `WouldBlock`.

---

## 5. Session READ / output streaming — exact path

Reading is entirely owned by the **`EventLoop<T: EventedPty>`**
(`app/src/terminal/local_tty/event_loop.rs:40`):

```rust
pub struct EventLoop<T: local_tty::EventedPty> {
    poll: mio::Poll,
    pty: T,
    rx: Receiver<Message>,                       // writes/resize/shutdown in
    terminal: Arc<FairMutex<TerminalModel>>,     // shared model, the output sink
    event_listener: ChannelEventListener,        // wakeups / events out
}
```

`EventLoop::spawn` (`event_loop.rs:323`) launches a dedicated OS thread named **`"PTY reader"`**.
The thread registers the PTY fd and the channel receiver with `mio::Poll` for
`READABLE | WRITABLE`, then loops on `poll.poll(...)`:

- **Read:** `EventLoop::pty_read` (`event_loop.rs:194`) reads up to `READ_BUFFER_SIZE` from
  `self.pty.reader().read(...)`, locks the `FairMutex<TerminalModel>`, and feeds bytes to
  **`state.parser.parse_bytes(terminal, &buf, &mut response_sequences)`**. The ANSI parser is
  `ansi::Processor` (`app/src/terminal/model/ansi/mod.rs`) — a wrapper around Alacritty's VTE
  parser that delegates to a `Performer`/`Handler` which mutate the terminal model (printing cells,
  CSI/OSC actions, Warp DCS hooks, §6). Terminal *response* sequences (e.g. device queries) are
  pushed back onto `write_list` to be written to the PTY. After processing,
  `event_listener.send_wakeup_event()` requests a redraw. The lock is periodically yielded with
  `FairMutexGuard::bump` so the render thread can read.
- **Write:** same loop calls `pty_write` (§4) when the fd is writable and `write_list` is non-empty.
- **Child exit:** `next_child_event()` (SIGCHLD on Unix) → `ChildEvent::Exited` winds the loop down;
  `Message::Shutdown` does likewise.
- **Synchronized output:** the parser supports DEC synchronized-output batching via
  `sync_output_remaining_timeout` / `finish_sync_output`.

So **output is not "returned"** from a function — it is streamed into the shared `TerminalModel`
(blocks/grid) and observers are notified via `ChannelEventListener`. *(Marley note: to surface raw
output to a custom panel you either (a) read structured state off `TerminalModel`/`BlockList` after
wakeups, or (b) tap the byte stream. A passthrough tap point already exists — `pty_read` has an
optional "copy of all bytes read" writer hook, used by `local_tty/recorder.rs`.)*

---

## 6. Blocks — per-command grids + shell-hook DCS metadata

Warp's signature feature: terminal output is segmented into **Blocks**, one per command. The
app-level model lives in `app/src/terminal/model/`:

- **`Block`** (`app/src/terminal/model/block.rs:274`) — a single command's record:
  `id: BlockId`, `header_grid: HeaderGrid`, `output_grid: BlockGrid`, `rprompt_grid`,
  `state: BlockState` (`block.rs:610`), `exit_code: ExitCode`, `session_id: Option<SessionId>`,
  and prompt context (`pwd`, `git_branch`, `virtual_env`, `conda_env`, `node_version`, `rprompt`),
  plus `bootstrap_stage` and an `event_proxy: ChannelEventListener`.
- **`BlockCommand`** (`block.rs:467`) — `{ command, output, stylized_command, stylized_output,
  stylized_prompt }`. **`PromptInfo`** (`block.rs:494`) groups the prompt fields fed by precmd.
- **`BlockGrid`** (`app/src/terminal/model/blockgrid.rs:1`) — the per-block cell grid (the actual
  character/style storage for that command's output).
- **`BlockList`** (`app/src/terminal/model/blocks.rs:225`) — the ordered collection of blocks plus
  filtering, scroll position, gaps; the `TerminalModel` exposes it via `block_list()` /
  `block_list_mut()` (`terminal_model.rs:1619`). The grid `size` lives here and seeds `PtyOptions`.
- Lower-level shared identity/index types come from **`warp_terminal`**: `BlockId`, `BlockIndex`
  (`crates/warp_terminal/src/model/mod.rs` re-exports `block_id::BlockId`, `block_index::BlockIndex`).

**Block boundaries and metadata come from the shell, not from heuristics.** The shell's bootstrap
script emits **DCS (or OSC) hook escape sequences** that the ANSI `Handler` interprets. The schema
is **`app/src/terminal/model/ansi/dcs_hooks.rs`**:

- **`DProtoHook`** enum (`dcs_hooks.rs:~40`) — the JSON payloads the shell sends:
  - **`InitShell`** — registers a subshell + its `SessionId`.
  - **`Precmd`** (`PrecmdValue`) — fired before the prompt: carries `pwd`, git branch, venv,
    node version, exit code, etc. → populates the *next* block's `PromptInfo`.
  - **`Preexec`** (`PreexecValue`) — fired when a command starts executing → opens/marks a block
    and records the command text.
  - **`Bootstrapped`** (`BootstrappedValue`, `is_subshell`) — shell finished bootstrap.
- Encodings (`dcs_hooks.rs:1–30`): a hex-encoded variant, an unencoded variant, and an ANSI-C
  quoted (`$'...'`) key-value variant — the final DCS char / first OSC parameter selects the codec.
- Each hook optionally carries a `SessionId` (`warp_core::SessionId`); the processor rejects hooks
  that require a registered session but lack one.

These hooks are what let Warp attach exit codes, working directory, and git/venv context to each
block, and what tie a block to its originating subshell `SessionId`. *(Marley note: any
"visualize an agentic workflow" panel can consume the same block stream + DCS metadata to know,
per command, the cwd / exit status / session — no extra instrumentation needed.)*

---

## 7. The named `crates/*` in detail

### `warp_terminal` (`crates/warp_terminal/`)
Shared, UI-agnostic terminal **model primitives**, adapted from Alacritty
(`src/model/LICENSE-ALACRITTY`). Modules: `model::grid` (flat cell storage, rows, dimensions,
graphemes, attribute maps), `model::ansi` (control-sequence parameters), `model::escape_sequences`
(incl. kitty keyboard protocol, `C0` constants used by `PtyController` for `CR/LF/EOT/ESC`),
`model::mode` (`TermMode`, `KeyboardModes`), `model::mouse`, and the `BlockId`/`BlockIndex` identity
types. `lib.rs` exposes `model`, `shell`, and an internal `shared_session` shim that converts
`Point` to/from `session_sharing_protocol::common::Point`. **No PTY here** — pure data model.
*Marley relevance: rebrand surface (the `C0`/escape constants and grid are Warp-neutral); the shared
grid types a custom renderer would read.*

### `warp_core` (`crates/warp_core/`)
Runtime core. Defines **`SessionId`** (`session_id.rs`) — the per-subshell identity threaded through
blocks and DCS hooks — plus `AppId`, `Channel`/`ChannelState`/`ChannelConfig`
(`channel/`, used by `warp_tui`'s bins and by the shell env in `build_host_shell_command`),
`paths`, `features::FeatureFlag` (gates SSH wrapper, HOA notifications during spawn), telemetry,
and OS info. *Marley relevance: `ChannelState`/`WarpServerConfig`/`OzConfig` are where server URLs &
channel branding are set — touchpoints for **de-auth** and **rebrand**.*

### `warp_tui` (`crates/warp_tui/`)
A headless terminal-UI front-end. `lib.rs` exposes only `input` (`TuiEditorModel` + `TuiInputView`).
The real entry is the per-channel bins (`src/bin/{oss,dev,stable,preview,local}.rs`) which build a
`ChannelState` and call **`warp::run_tui()`** (defined in the `app` crate, `app/src/lib.rs`).
*Marley relevance: a minimal, login-light front-end already exists here — a useful reference for a
stripped Marley shell, and proof the engine can run without the full GUI.*

### `local_control` (`crates/local_control/`)
The **external automation protocol** ("warpctrl"), UI-agnostic so the app and a CLI share wire
types. `lib.rs` re-exports: `protocol` (`RequestEnvelope`/`ResponseEnvelope`/`Action`/`ControlError`/
`ErrorCode`, `PROTOCOL_VERSION = 1`), `catalog` (`ActionKind` + metadata), `selectors`
(`TargetSelector` → window/tab/pane/session), `discovery` (instance records under a discovery dir),
and `auth` (`AuthToken`, `CredentialGrant`, scoped credentials). The action catalog
(`catalog.rs:167`, a `define_action_catalog!` macro) enumerates everything an external client may
drive, including:
- `window.create`, **`tab.create`** (`TabCreateParams { tab_type, shell }`), `tab.activate/close`,
  `pane.split/focus/resize/close`;
- **`session.list/inspect/activate/previous/next/reopen_closed`**;
- **`input.insert` / `input.replace`** (`TextParams { text }`) — inject text into the active input.

Crucially, the catalog has **no raw-byte write and no output-read-back action** — it is a
higher-level "drive the GUI" surface (create/focus tabs, insert input text). `ErrorCode` includes
`LocalControlDisabled`, `UnauthorizedLocalClient`, `InsufficientPermissions`, `NotAllowlisted`.
*Marley relevance (goal b): this is the **ready-made out-of-process control seam** for an agentic
panel to open sessions and type into them — but to read structured output back you must either add a
new `ActionKind` (e.g. `session.read`) backed by `TerminalModel`/`BlockList`, or tap the byte stream
in `event_loop.rs`. Also a de-auth touchpoint: `auth.rs` + the `Unauthorized*`/`NotAllowlisted`
error codes gate who may control the app.*

### `command` (`crates/command/`)
A thin drop-in wrapper over `std::process::Command` (`blocking`) and `async_process::Command`
(`async`), plus `unix`/`windows`/`wsl` helpers. Sole purpose: force the Windows `CREATE_NO_WINDOW`
flag so spawning never flashes a console. `spawn_command_in_pty` uses `command::blocking::Command`.
*Marley relevance: low; the substrate that builds the shell process.*

### `command-signatures-v2` (`crates/command-signatures-v2/`)
A `rust-embed` wrapper exposing `COMMAND_SIGNATURES_JS` — a precompiled JS bundle
(`js/build`) used elsewhere to parse command signatures. Not on the spawn/read/write path.
*Marley relevance: negligible (rebrand/license only).*

### `ipc` (`crates/ipc/`)
A generic request/response IPC framework: a server hosting typed `Service`s and typed
`ServiceCaller` clients across process boundaries, over the `interprocess` crate (UDS on Unix,
named pipes on Windows; WASM unsupported). Documented as built first for the **plugin host**, "but
designed generically to be extended to other use cases (such as the terminal server)." Note the
terminal server in `app/src/terminal/local_tty/server/` currently uses its **own** hand-rolled
socket protocol (`server/protocol.rs`, with `SCM_RIGHTS` fd passing) rather than this crate.
*Marley relevance: the transport pattern a Marley control channel could reuse.*

---

## 8. End-to-end data-flow summary

```
            ┌──────────────────────── UI surface (View) ───────────────────────┐
            │  emits Event ──(PtyIntentEvent::pty_intent)──▶ PtyIntent          │
            └───────────────────────────────┬──────────────────────────────────┘
                                            ▼
   TerminalManager ──▶ PtyController.write_bytes / write_command / resize_pty
                                            │  PtyWrite queue (line-editor aware)
                                            ▼
                       Message::Input(bytes) | Resize | Shutdown
                                            │  event_loop_tx (mio_channel::Sender)
                                            ▼
   ┌──────────────────── "PTY reader" thread: EventLoop (mio::Poll) ───────────────────┐
   │  pty_write ──▶ Pty.writer().write() ─────────────▶ leader fd ─▶ shell process      │
   │  shell stdout ─▶ leader fd ─▶ Pty.reader().read() ─▶ ansi::Processor.parse_bytes   │
   │                                       │                                            │
   │                                       ▼                                            │
   │                         Arc<FairMutex<TerminalModel>>  (BlockList / BlockGrid)     │
   │      shell DCS/OSC hooks (DProtoHook: Precmd/Preexec/Bootstrapped/InitShell)       │
   │                                       │  + SessionId                               │
   │                                       ▼                                            │
   │                       ChannelEventListener.send_wakeup_event() ─▶ render/observers │
   └────────────────────────────────────────────────────────────────────────────────────┘

   Spawn:  TerminalManager.spawn_pty ─▶ Pty::new ─▶ PtySpawner.spawn_pty
           ├─ server: TerminalServerClient.spawn_pty ─(UDS + SCM_RIGHTS leader fd)─▶ child shell
           └─ direct: local_tty::spawn ─▶ openpty + Command.pre_exec(fork) ─▶ child shell

   Out-of-process control (existing seam): warpctrl ─▶ local_control RequestEnvelope
           ─▶ {tab.create, session.activate, input.insert, …}   (no raw byte read/write)
```

---

## 9. Marley relevance — consolidated

- **Goal (b), spawn/write/read (primary):** fully realized in `app/src/terminal/writeable_pty`
  (`PtyController`, `PtyIntent`, `Message`) and `app/src/terminal/local_tty`
  (`Pty`, `PtySpawner`, `EventLoop`, terminal server). The cleanest integration points for an
  Ignibyte panel are: **write** via `PtyController::write_bytes` / a custom `PtyIntentEvent` surface;
  **read** via observing `TerminalModel`/`BlockList` on `ChannelEventListener` wakeups, or tapping
  the `pty_read` byte-copy hook (`recorder.rs`); **spawn** via `TerminalManager::spawn_pty` with a
  custom `PtyOptions`.
- **Goal (b), out-of-process driving:** `local_control` already lets an external process create
  tabs/sessions and inject input text — but lacks byte-level read-back, which Marley would need to
  add (new `ActionKind` over `BlockList`, or an IPC tap).
- **UI-surface expansion:** the `TerminalSurface` / `PtyIntentEvent` traits are the explicit,
  UI-agnostic seam — a new panel implements them without touching the PTY engine. Blocks + DCS
  metadata give a custom "agentic workflow" view per-command cwd/exit/session for free.
- **De-auth:** gating lives in `local_control::auth` and the `Unauthorized*` / `NotAllowlisted`
  error codes; channel/server config (and thus any login coupling) is in `warp_core::channel`.
- **Rebrand:** Warp-specific strings appear in the shell env (`TERM_PROGRAM=WarpTerminal`,
  `WARP_*`, `WarpServerConfig`) and the Alacritty/Warp model crates — mechanical, but the env-var
  names are a protocol contract with the bundled shell bootstrap scripts (changing `WARP_*` requires
  updating the shell hooks too).

---

## Marley status @ M15

Terminal-session-core is **Marley's strongest subsystem**. The block-terminal is at or near Warp
parity, and Marley has the **editor Warp lacks**, so the terminal↔editor fusion (the product wedge) is
Marley's to win. Where the Warp mechanics above are the *reference*, Marley's own implementation lives
in **`crates/terminal_blocks`** (UI-agnostic PTY session + Block model) rendered by `crates/marley_app`
— see [`../../marley_architecture/terminal_blocks.md`](../../marley_architecture/terminal_blocks.md).

| Capability | Warp (this doc) | Marley @ M15 | Verdict |
|---|---|---|---|
| **Byte engine / grid / VTE** | Alacritty-derived grid embedded in `warp_terminal`; app-level `ansi::Processor` | **`alacritty_terminal` + `vte` used directly** (`terminal_blocks::{session,styled}`) | **Matched** — same permissive substrate, adopted not forked |
| **PTY spawn / read / write** | `Pty`/`PtySpawner`/`EventLoop` ("PTY reader" thread) + terminal server (`SCM_RIGHTS`) | `terminal_blocks::pty_os` (the 4 raw OS calls, **`unsafe`-free** via `rustix`) + a `PtyChannel`-trait `TerminalSession` (`pump`/`write_bytes`/`resize`) | **Matched** (single-process); out-of-process terminal server **deferred** |
| **Command Blocks** (per-command grid, exit code, cwd, duration, rerun) | `Block`/`BlockList`/`BlockGrid`, `BlockState`, `PromptInfo` | `terminal_blocks::block` (`Block`/`BlockList`/`BlockState`/`ExitCode`/`PromptInfo`) + `rerun_command`/`copy_text` | **Matched** |
| **Shell-integration metadata** | `DProtoHook` (Precmd/Preexec/Bootstrapped/InitShell) DCS/OSC schema | `terminal_blocks::dcs` — a **clean-room-invented** wire format (`ESC P <h\|p\|q> name;key=value ESC\`), behavior-derived | **Matched, provenance-clean** — own schema, not Warp's |
| **Clickable `file:line:col` → editor** | *not covered by this doc* | **Shipped #196** (text-scan) — paths open in the code view, URLs in the browser | **Shipped** (front half of the wedge) |
| **OSC 8 explicit hyperlinks** | *not covered* | **Shipped #214** — `StyledRun.hyperlink` carries the grid cell URI; render prefers it over the heuristic | **Shipped** |
| **Terminal ↔ editor fusion** | Warp has no editor | **Front half shipped** (link-open into Marley's editor); back half = editor maturity (M13/M14) | **Marley's wedge** (Warp can't; Zed has no blocks) |
| **Out-of-process control / brain-tap** | `local_control` "warpctrl" (drive-only, **no** byte read-back) | seam reimplemented for the brain; **`session.read` read-back delta is a Marley INVENT** (Warp has none) | **Exceeds (planned)** |

**The fusion wedge, precisely.** Warp frames Blocks but has no editor to jump *into*; Zed has an editor
but [**no command-block model at all**](../../zed_architecture/subsystems/08-terminal-tasks-fusion.md)
(it infers the "current command" by polling the OS process table, not shell hooks). Marley has **both**
— a Warp-parity block-terminal *and* an editor — so a failed build Block can resolve its `file:line:col`
links against **that Block's captured cwd** and open the code view in place. #196/#214 shipped the
link-open half; the remaining upgrade is scoping resolution to the Block's cwd and routing *runnables
into Blocks* (the Zed doc §8 details the runnable-as-Block plan). The Block is also the structured unit
the proprietary **brain** layer observes (`session.read`) — pass/fail is machine-readable with no
scrollback scraping.

---

## Provenance & licensing

**Warp is AGPL-3.0-only.** Every crate in this subsystem inherits the workspace
`license = "AGPL-3.0-only"` (the MIT `warpui*` GUI crates live in other subsystems). AGPL's network-use
copyleft is *stronger* than Zed's GPL, so the boundary matters even more: **the Warp Block model + the
shell-integration protocol are the AGPL surface Marley reimplements clean-room, never copies.** Tags
mirror [`../../zed_architecture/README.md`](../../zed_architecture/README.md).

- **`[permissive: alacritty_terminal Apache-2.0/MIT — Marley already uses it]`** — the **byte engine**:
  the PTY, the VTE grid + ANSI/escape parser, cell/color/flags storage. Warp *embeds* Alacritty-derived
  code (`warp_terminal/src/model/LICENSE-ALACRITTY`); **Marley depends on the upstream
  `alacritty_terminal` + `vte` crates directly** (`crate-map.md`: REUSE). This layer carries **no
  copyleft risk** — it is the one thing Warp and Marley share cleanly.
- **`[Warp-derived: AGPL-3.0]`** — the *design* Marley reimplements, **not** carried as source: the
  **command-Block model** (`Block`/`BlockList`/`BlockGrid`, per-command exit/cwd/duration), the
  **shell-integration handshake** (`DProtoHook` Precmd/Preexec/Bootstrapped/InitShell DCS/OSC schema),
  `SessionId`/`ChannelState` (`warp_core`), the "warpctrl" control protocol (`local_control`), and the
  `warp_terminal` / `warp_tui` / `command` / `ipc` wrappers. These are **architecture, reimplemented
  clean-room** — every description in this doc is Marley's own; **no Warp source is committed to the
  repo.**
- **`[Marley-original]`** — Marley's actual code: **`crates/terminal_blocks`** (the PTY session + Block
  model, a **clean-room-invented DCS wire format**, `pty_os` raw calls, the `PtyChannel` seam),
  **`crates/marley_core`** (own `SessionId`, **no cloud/auth**), **`crates/marley_command`** (the non-PTY
  spawn seam — the PTY path goes through `terminal_blocks`/`alacritty_terminal::tty` directly), the
  **clickable-link render** (`marley_app::links`, #196/#214), and — critically for the business model —
  the **brain-side `session.read` read-back + fusion glue**, which must stay clean of any AGPL-derived
  code to remain sellable (per `../../zed_architecture/README.md`).

**De-auth / rebrand touchpoints** (from §9, now provenance-tagged): server URLs + channel branding live
in `warp_core::ChannelState` **[Warp-derived]** → dropped in `marley_core` **[Marley-original]**; the
shell-env protocol contract (`TERM_PROGRAM=WarpTerminal`, `WARP_*`) is **[Warp-derived]** and must be
renamed in lockstep with the (clean-room) shell hooks it contracts with.

---

## Key files (paths relative to repo root)

- `app/src/terminal/local_tty/mod.rs` — `PtyOptions`, `EventedReadWrite`, `EventedPty`, `ChildEvent`.
- `app/src/terminal/local_tty/unix.rs` — `Pty`, `make_pty`/`openpty`, `spawn`, `spawn_command_in_pty`,
  `PtySpawnResult`.
- `app/src/terminal/local_tty/spawner.rs` — `PtySpawner`, `PtyHandle`, `Direct`/`ServerOwnedPtyHandle`,
  `PtySpawnInfo`.
- `app/src/terminal/local_tty/event_loop.rs` — `EventLoop`, `pty_read`, `pty_write`, `spawn` ("PTY reader").
- `app/src/terminal/local_tty/terminal_manager.rs` — `TerminalManager::spawn_pty`, `start_pty_event_loop`.
- `app/src/terminal/local_tty/server/{mod,client,protocol,api}.rs` — out-of-process PTY server (SCM_RIGHTS fd passing).
- `app/src/terminal/writeable_pty/pty_controller.rs` — `PtyController` write coordinator (`write_bytes`/`write_command`/`resize_pty`/`shutdown_pty`).
- `app/src/terminal/writeable_pty/message.rs` — `Message` (Input/Resize/Shutdown/ChildExited).
- `app/src/terminal/writeable_pty/terminal_surface.rs` — `PtyIntent`, `PtyIntentEvent`, `TerminalSurface` seam.
- `app/src/terminal/model/block.rs` — `Block`, `BlockCommand`, `BlockState`, `PromptInfo`.
- `app/src/terminal/model/blocks.rs` — `BlockList`.
- `app/src/terminal/model/blockgrid.rs` — `BlockGrid`.
- `app/src/terminal/model/ansi/dcs_hooks.rs` — `DProtoHook` (Precmd/Preexec/Bootstrapped/InitShell) shell metadata schema.
- `app/src/terminal/model/ansi/mod.rs` — `Processor`/`Performer`/`Handler` ANSI pipeline.
- `crates/warp_core/src/session_id.rs` — `SessionId`.
- `crates/warp_terminal/src/model/mod.rs` — shared `BlockId`/`BlockIndex`/grid/escape primitives.
- `crates/local_control/src/catalog.rs` — `ActionKind` catalog (session/tab/input actions).
- `crates/local_control/src/protocol.rs` — control wire envelopes + `ErrorCode`.
- `crates/command/src/{blocking,async}.rs` — `Command` wrapper used to build the shell process.
