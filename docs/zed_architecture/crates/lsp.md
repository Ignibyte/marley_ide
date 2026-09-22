# lsp

> Per-crate reference (Marley Zed-architecture, round 2) — crate dir `crates/lsp`. The EDITOR
> reference is **Zed** (github.com/zed-industries/zed, GPL-3.0); the terminal/cockpit half is Warp.
> Compare with the Warp counterpart [`../../warp_architecture/crates/lsp.md`](../../warp_architecture/crates/lsp.md).

| Field | Value |
|-------|-------|
| Subsystem | [05 — LSP & Language Intelligence](../subsystems/05-lsp-language-intelligence.md) |
| License | **GPL-3.0-or-later** — explicit per-crate `license = "GPL-3.0-or-later"` in `Cargo.toml` + a `LICENSE-GPL` symlink. `[Zed-derived]` |
| Internal deps | **4** (`collections`, `gpui`, `util`, `release_channel`) |
| Used by | **25** internal crates — the busiest hub in the language stack |
| Size | `lsp.rs` 2329 LOC + `input_handler.rs` 213 LOC (self-contained) |

## Purpose

`lsp` is Zed's **Language Server Protocol client** — a thin, self-contained protocol transport. It is
**one `LanguageServer` struct per server child process**: it spawns the process, speaks JSON-RPC 2.0 over
`Content-Length`-framed stdio, correlates responses to requests, dispatches server-initiated
notifications/requests, negotiates capabilities via the `initialize` handshake, and exposes typed
`request` / `notify` methods. **It knows nothing about buffers, languages, worktrees, or coordinates** —
all of that intelligence lives one layer up in `project::lsp_store` (see the subsystem doc).

Two things distinguish it from the Warp counterpart:

1. **Self-contained transport.** Warp split a separate [`jsonrpc`](../../warp_architecture/crates/jsonrpc.md)
   crate under `lsp`; Zed **inlines** the JSON-RPC machinery here (`Request`/`AnyResponse`/`Notification`
   wire structs + `input_handler.rs` framing). No separate protocol crate.
2. **It is a pure client, not a manager.** Warp's `lsp` was a manager + installer + supported-servers
   registry (`LspManagerModel`, `supported_servers`, auto-install via node) with a **single** dependent
   (`warp`). Zed's `lsp` is *only* the transport and has **25** dependents; spawn policy, install, and
   per-worktree routing all live in `project`. Keeping the transport dumb is the headline design lesson.

The public type surface is `pub use lsp_types::request::*;` + `pub use lsp_types::*;` — the entire protocol
vocabulary is re-exported from a **Zed fork** of `lsp-types` (`git = zed-industries/lsp-types`, rev
`f4dfa89`), not the crates.io release.

## Key types, modules & public API

Re-exported from `crates/lsp/src/lsp.rs`:

- **`pub struct LanguageServer`** — a running server process. Holds `server_id`, a monotonic
  `next_id: AtomicI32`, an `outbound_tx` channel, a separate `notification_tx` channel, `capabilities:
  RwLock<ServerCapabilities>`, the `notification_handlers` / `response_handlers` (keyed by `RequestId`) /
  `pending_respond_tasks` / `io_handlers` maps, the `io_tasks` handles, the `Child`, `workspace_folders`,
  and `root_uri`.
- **`LanguageServerId(pub usize)`**, **`LanguageServerName(SharedString)`**,
  **`LanguageServerSelector::{ Id, Name }`** — identity.
- **`LanguageServerBinary { path, arguments, env }`** + **`LanguageServerBinaryOptions {
  allow_path_lookup, allow_binary_download, pre_release }`** — how to launch / discover.
- **`RequestId::{ Int(i32), Str(String) }`** — `#[serde(untagged)]`; the client always **emits `Int`** but
  accepts either inbound (the spec permits string ids).
- Wire structs: **`Request<'a, T>`**, **`AnyResponse<'a>`** (uses borrowed `serde_json::value::RawValue`
  for zero-copy inbound dispatch), `Response<T>`, `Notification`; constant `JSON_RPC_VERSION = "2.0"`,
  `CONTENT_LEN_HEADER = "Content-Length: "`.
- **`trait LspRequestFuture<O>: Future<Output = ConnectionResult<O>>`** with `id() -> i32` — the request
  future carries its own id so it can be cancelled; `ConnectionResult::{ Result, ConnectionReset, Timeout }`.
- **`Subscription::{ Notification, Io }`** — `#[must_use]` RAII handle whose `Drop` removes the handler
  from the map (via a `Weak` upgrade). `detach()` leaks it deliberately.
- **`IoKind::{ StdOut, StdIn, StdErr }`**, **`AdapterServerCapabilities`**.
- **`FakeLanguageServer`** (behind `test-support`) — the in-process test double.

### Methods on `LanguageServer` (the API surface)

- **`new(stderr_capture, id, name, binary, root_path, code_action_kinds, workspace_folders, cx)`** — spawns
  the child: `stdin/stdout/stderr` all piped, **`kill_on_drop(true)`**, `current_dir` = the resolved root
  (or its parent if the root is a file). Delegates to `new_internal`, which is **generic over the three IO
  streams** — the same core builds both the real server and the `FakeLanguageServer` (over `async_pipe`).
- **Three background IO tasks** on the gpui executor: `handle_incoming_messages` (stdout → dispatch),
  `handle_stderr` (line-buffered, mirrored to `io_handlers` + an optional captured log `String`), and
  `handle_outgoing_messages` (frames + writes stdin).
- **`request::<T: request::Request>(params, timeout) -> impl LspRequestFuture<T::Result>`** — the core call
  (§ below). `request_with_timer` takes a custom timeout future; `request_timer(timeout)` builds one
  (`Duration::MAX` / `ZERO` ⇒ a `future::pending` that never fires).
- **`notify::<T: notification::Notification>(params)`** — fire-and-forget. Note the two-channel design:
  notifications go through a dedicated `notification_tx` carrying a **serializer closure** so that ordering
  against in-flight requests is well-defined and large params aren't serialized on the caller's thread.
- **`initialize(self, params, configuration, timeout, cx) -> Task<Result<Arc<Self>>>`** — sends the
  `Initialize` request, stores the returned `ServerCapabilities` (every later request is gated on these) +
  `server_info`, then fires the `Initialized` notification. **Consumes `self`, returns `Arc<Self>`.**
- **`default_initialize_params(pull_diagnostics, augments_syntax_tokens, cx)`** — builds the
  `ClientCapabilities`: advertises **UTF-16 positions** (`position_encodings`), completion
  (`snippet_support`, resolve for `additionalTextEdits`/documentation, item defaults), hover Markdown,
  rename `prepare_support`, code actions (literal + resolve), `publishDiagnostics` (related info +
  `UNNECESSARY`/`DEPRECATED` tags + version support + code descriptions), semantic tokens (full + delta),
  inlay hints (resolve), formatting/range, and `workspace_edit` with document changes + snippet edits.
- **`shutdown()`** — sends `Shutdown` (5 s `SERVER_SHUTDOWN_TIMEOUT`), then the `Exit` notification, closes
  channels, drains the output barrier, and kills the child.
- **`on_notification::<T>` / `on_request::<T>` / `on_io`** → `#[must_use] Subscription`. `register_buffer` /
  `unregister_buffer`, `add_workspace_folder` / `remove_workspace_folder` / `set_workspace_folders` (gated
  on the server's `workspace_folders` change-notification capability), `capabilities()`,
  `update_capabilities`, `code_action_kinds()`.

### The request / response / cancellation model (load-bearing)

`request_internal_with_timer` (the heart):

1. `id = next_id.fetch_add(1)`; serialize `{ jsonrpc, id, method, params }`.
2. Register a **response handler keyed by `RequestId::Int(id)`** that deserializes and pushes to a
   **`oneshot`** channel; `try_send` the JSON onto `outbound_tx`.
3. The returned future `select!`s the `oneshot` **against a timeout timer**
   (`DEFAULT_LSP_REQUEST_TIMEOUT = 120 s`). Timer wins ⇒ remove the handler, return `Timeout`; server
   resets the connection ⇒ the `oneshot` `Canceled` maps to `ConnectionReset`.
4. **`cancel_on_drop`** — the future holds a `util::defer(...)` guard. If the future is **dropped before
   completing** (e.g. the user moved the cursor and the stale completion is abandoned), the guard sends
   **`$/cancelRequest`** to the server. On normal completion it `.abort()`s the guard. **As-you-type
   features depend on this**: fire a completion/hover on every keystroke and let drop cancel the losers.
- **Inbound cancellation:** `handle_incoming_messages` special-cases the `$/cancelRequest` method from the
  *server* by removing the matching `pending_respond_tasks` entry — dropping the task cancels an in-flight
  server→client request handler.

### Transport & framing — `input_handler.rs`

- **`LspStdoutHandler`**: `read_headers` scans stdout to the `\r\n\r\n` delimiter, parses the
  `Content-Length`, then `read_exact`s the body. Each body is tried first as `NotificationOrRequest`, else
  as `AnyResponse` (routed by `id` to the pending `response_handlers`), else logged as undeserializable.
- **Backpressure:** the reader feeds a **bounded channel (`INCOMING_MESSAGE_QUEUE_CAPACITY = 128`)**. When
  full it *stops reading stdout*, so the OS pipe applies backpressure to the **server** rather than
  buffering unbounded in RAM while the foreground dispatcher is busy. A cheap, worth-copying robustness
  property (there's a dedicated `test_backpressure_when_messages_are_not_consumed`).
- **Outgoing frame:** `Content-Length: {n}\r\n\r\n{json}`, flushed per message.

### `FakeLanguageServer` (test-support)

`FakeLanguageServer::new(id, binary, name, capabilities, cx)` builds **two cross-wired `LanguageServer`s**
over `async_pipe` pipes — the client under test and the fake — so no real binary is needed. It
auto-answers `Initialize` (returning the supplied `ServerCapabilities`) and `Shutdown`, and exposes
`set_request_handler` / `handle_notification` / `notify` / `receive_notification` /
`try_receive_notification` / `start_progress` / `end_progress`. `LanguageServer::full_capabilities()` is a
convenience for tests. **This is what lets every downstream feature be tested without a server binary** —
Marley wants the equivalent from day one.

## Depends on (internal)

Only **4** workspace crates — deliberately minimal:

- `gpui` — `App` / `AsyncApp` / `BackgroundExecutor` / `Task` / `SharedString` (the async runtime the IO
  tasks run on). This is the one structural coupling.
- `util` — `ConnectionResult`, `command::{ Child, Stdio }` (process spawn), `defer` (the cancel-on-drop
  guard), `ResultExt`.
- `collections` — `HashMap` / `BTreeMap` handler maps.
- `release_channel` — channel-aware behaviour.

Non-workspace: `lsp-types` (**Zed git fork**), `smol` / `futures` (async IO), `postage` (barrier),
`parking_lot`, `serde` / `serde_json` (incl. `RawValue`), `schemars`. Test-support adds `async-pipe`,
`gpui_util`.

## Used by (internal dependents)

**25** crates — the widest fan-out in the language stack (vs. **1** for Warp's `lsp`):

`project` (the `lsp_store` orchestrator — the primary consumer), `editor`, **`diagnostics`**, `language`,
`languages`, `copilot`, `copilot_ui`, `edit_prediction`, `edit_prediction_context`, `extension`,
`extension_host`, `language_extension`, `language_tools` (RPC-trace/health), `outline`, `outline_panel`,
`project_symbols`, `prettier`, `search`, `vim`, `remote_server`, `collab`, `agent`, `agent_ui`,
`language_core`, plus `benchmarks`. The breadth is why the client API is kept small and stable.

## Related crates

- `project` (`lsp_store` / `lsp_command`) — the orchestration brain: server registry per
  (worktree × language), document sync, the `LspCommand` request catalog, and the
  UTF-16 → `Unclipped<PointUtf16>` → clip → `Anchor` round-trip. **All the hard stuff is there, not here.**
- `language` — owns `point_to/from_lsp`, `range_to/from_lsp`, and the `Unclipped<PointUtf16>` newtype (the
  coordinate crux); also the `Diagnostic` type this client's `PublishDiagnostics` feeds.
- [`diagnostics`](./diagnostics.md) — a downstream UI consumer of the notifications this client dispatches.
- Warp's [`jsonrpc`](../../warp_architecture/crates/jsonrpc.md) — the equivalent transport Warp factored
  out; here it is inlined.

## Marley — reimplementation on our stack

**Marley baseline (post-M15): none of this exists.** No LSP client, no language servers, and — the hard
prerequisite — **no anchor layer** and no UTF-16 coordinate module. Every LSP result is asynchronous and
positional (a completion lands 200 ms later; a diagnostic arrives after three more keystrokes), so results
must land on **anchors**, not raw `CharOffset`s. **The sequence is not negotiable: anchors → UTF-16
coords → LSP client.**

**The plan — a clean `marley_lsp` client written from the spec:**

1. **New crate `marley_lsp`**, built on the **published Microsoft LSP spec** + **crates.io `lsp-types`
   (MIT)** (not Zed's fork). This is the *cleanest* subsystem to reimplement without copyleft exposure,
   because the wire protocol and the type vocabulary are public.
2. **Copy the shape, not the source** (design lessons from this crate):
   - one `LanguageServer` per child process; **keep it dumb** (no buffers/coords) — concentrate
     orchestration in a separate `LspStore`;
   - the request/notify model: monotonic id → `oneshot` response handler → `select!` against a timeout;
   - **`$/cancelRequest`-on-drop** — Marley's as-you-type completion/hover reuse the same
     drop-cancels-the-loser trick (this pairs naturally with Marley's #200 ghost-text keystroke path);
   - the **bounded-channel backpressure** (128) on the stdout reader;
   - the `initialize` capability handshake advertising **UTF-16** + the feature flags Marley actually
     renders (start with diagnostics + hover + goto; add completion/rename later);
   - a **`FakeLanguageServer` equivalent from day one** so features are testable with no real binary
     (matches Marley's shim/capture discipline — a headless fake keeps the gates green).
3. **Server discovery: assume a user-provided binary on `PATH`** first (offline, simple) rather than
   porting Zed's auto-install/node-runtime path — that keeps the crate minimal and avoids the network.
4. **Reuse seams already in Marley:** goto-definition responses (`LocationLink`) plug straight into the
   #196 `open_link_target` + `:line:col` path; workspace-edit apply uses `EditOrigin::Agent`; diagnostics
   gutter marks reuse the #198 scrollbar track. (Details in the subsystem doc §§4–8.)

## Provenance

- **`[permissive/public: LSP spec, lsp-types MIT]`** — the wire protocol (JSON-RPC 2.0 over
  `Content-Length`-framed stdio, UTF-16 positions) is a **published Microsoft specification**; the
  `lsp-types` data structures are **MIT** on crates.io. Marley implements a conformant client **from the
  spec** using MIT `lsp-types`, owing nothing to Zed. Note Zed uses a *git fork* of `lsp-types` — Marley
  should use the **crates.io MIT release** for the cleanest provenance.
- **`[Zed-derived]`** — this crate is **GPL-3.0-or-later**. The *specific wiring* (the three-task IO model,
  the `cancel_on_drop` guard via `util::defer`, the `RawValue`-borrowed inbound dispatch, the two-channel
  request/notification split, the `FakeLanguageServer` design) is studied as **design reference only**;
  no source is copied into `marley_lsp`.
- **`[Marley-original]`** — the `marley_lsp` client, the `LspStore` orchestration, the anchor layer, and
  the UTF-16 coordinate module are all new Marley code on the public spec + MIT types.
- **Brain boundary:** the LSP *client* and its *features* (completion, diagnostics, goto, hover, rename)
  live in Marley's GPL editor/terminal layer — fine. **Edit prediction**, which repurposes this client's
  goto-definition as RAG context, is the **proprietary brain** seam and must stay clean of Zed-derived
  source (see subsystem doc §9).

## Notes / gotchas

- **Self-contained** — unlike Warp, there is no separate `jsonrpc` crate; the protocol machinery is inlined
  here + in `input_handler.rs`.
- **UTF-16 positions.** The client speaks the spec's `(line, UTF-16 code-unit column)` coordinates. Marley's
  buffer is char-offset indexed, so a `PointUtf16` + `Unclipped` + `clip_point_utf16` conversion is
  mandatory (in Zed it lives in `language`, *not* here) — and it differs from char offsets for any non-BMP
  character (emoji, which Marley's buffer tests already exercise).
- **`RequestId` asymmetry** — the client only emits `Int` ids but must deserialize `Int | Str` (untagged),
  because the spec permits string ids from a server.
- **Two shutdown paths** — graceful `shutdown()` (Shutdown + Exit + drain) vs. `kill_on_drop` + the `Drop`
  impl (abrupt). Both must clear the handler maps to avoid leaking `oneshot` senders (there's a dedicated
  `test_subscription_leaks_handlers_after_server_drop`).
- **Subscriptions are `#[must_use]`** — dropping the returned `Subscription` unregisters the handler; keep
  it alive for the handler's lifetime (or `detach()` deliberately).
- **`lsp-types` is a git fork** in Zed — API drift from crates.io is possible; another reason Marley should
  build against the MIT crates.io release directly.
- **25 dependents** = every change here ripples; the transport API is intentionally tiny and stable.
