# Subsystem 05 — LSP & Language Intelligence

Part of the Marley Zed-architecture docs (round 1). Marley's editor surface is modeled on **Zed**
(github.com/zed-industries/zed, GPL-3.0); the terminal/cockpit half is modeled on Warp.

> Scope: the language-intelligence stack — the LSP client (`lsp`), the project-level LSP
> orchestration (`project`'s `lsp_store` / `lsp_command`), the diagnostics UI (`diagnostics`), the AI
> edit-prediction context + UI (`edit_prediction_context`, `edit_prediction_ui`), and the LSP
> observability tools (`language_tools`). This is the subsystem that gives an editor completions,
> diagnostics (squiggles), go-to-definition, hover, rename, code actions, and format-on-save.

## Provenance banner (READ FIRST)

The **Language Server Protocol is a published Microsoft specification** (JSON-RPC 2.0 over
`Content-Length`-framed stdio, UTF-16 positions). That matters enormously for the license boundary:

- `[permissive/public]` — **the protocol itself, and the `lsp-types` data structures**, are public. The
  wire format is a spec; the crates.io `lsp-types` crate is **MIT**; JSON-RPC framing is standard. Marley
  can implement a conformant client **from the spec**, using MIT `lsp-types` (or its own generated types),
  owing nothing to Zed.
- `[Zed-derived]` — Zed's **specific client wiring** (`crates/lsp`), and especially its **project
  orchestration** (`LocalLspStore`, the `LspCommand` trait, the coordinate/anchor round-trip) are GPL. We
  study these for design, but Marley writes its own.
- `[Marley-original]` — Marley's LSP client + orchestration will be new code built on the public spec and
  MIT types. Because the protocol is public, this is one of the *cleanest* subsystems to reimplement
  without copyleft exposure — the design ideas below (per-worktree server registry, the request-as-command
  pattern, UTF-16→anchor clipping) are architecture, and the concrete code is Marley's.
- **Brain boundary:** LSP *features* (completion/diagnostics/goto) live in the GPL editor layer. **Edit
  prediction** (§9) is the AI/agent surface and belongs to the **proprietary brain** — Zed already draws
  this exact line (context assembly + prediction is a provider-agnostic store; the GPL editor only renders
  ghost text and applies the edits).

## Marley baseline (post-M15): none of this exists yet

- **No LSP.** No language servers, no completions/diagnostics/goto/hover/rename/format. The #200
  "ghost-text" is **shell-history** suggestion (`history::suggest`), not code intelligence.
- **No anchors.** `crates/editor/src/buffer.rs` edits by `Range<CharOffset>` with an `EditOrigin`
  (`crates/editor/src/types.rs:10` → `Human | Agent`) but has **no stable-position layer**. This is the
  **hard prerequisite**: every async LSP result (a completion that lands 200 ms later, a diagnostic that
  arrives after you've typed three more chars) is a position into a buffer version that no longer exists.
  Without anchors, results land in the wrong place. **Anchors come before LSP.** (See §11.1.)
- **What *is* ready to reuse:** the #196 link mechanism (`open_link_target` at
  `crates/marley_app/src/app.rs:2317`, with `LinkTarget` carrying `:line:col` via #212, `strip_line_col`
  at `links.rs:238`) is the **file:line:col → open-in-editor** path that goto-definition plugs straight
  into (§6). The `EditOrigin::Agent` variant is the **apply-seam** for programmatic buffer writes
  (workspace-edits, §8). The #198 scrollbar (`scrollbar_thumb`/`at_bottom` at
  `crates/marley_app/src/viewport.rs:98`) is the diagnostic-marker gutter track (§5).

---

## 1. Purpose & big picture

The stack is three layers, cleanly separated:

```
  language servers (rust-analyzer, gopls, pyright, tsserver, clangd …) — external processes
        ▲  JSON-RPC 2.0 over stdio, Content-Length framed, UTF-16 positions
        │
 ┌──────┴───────────────┐  crates/lsp  [Zed-derived wiring / public spec]
 │  LanguageServer       │  one struct per server process: transport, request/response/notify,
 │  (protocol client)    │  cancellation, capabilities, initialize handshake
 └──────┬───────────────┘
        │  typed requests / notifications
 ┌──────┴───────────────┐  crates/project  ::lsp_store / ::lsp_command  [Zed-derived]
 │  LocalLspStore        │  server registry per (worktree × language), buffer document-sync,
 │  (orchestration)      │  publishDiagnostics fan-in, the LspCommand request catalog,
 │                       │  UTF-16 → Unclipped<PointUtf16> → clip → Anchor round-trip
 └──────┬───────────────┘
        │  buffer-anchored results
 ┌──────┴───────────────────────────────────────────────────────────────┐
 │  editor (completion menu, hover popover, inline diagnostics, blocks)    │  [GPL editor]
 │  diagnostics (project panel + status item)   language_tools (RPC log)   │
 │  edit_prediction_* (AI inline edits — BRAIN layer)                      │
 └────────────────────────────────────────────────────────────────────────┘
```

The **`lsp` crate is a thin protocol client** — it knows nothing about buffers, languages, or worktrees;
it spawns a process and moves typed JSON-RPC messages. **All the intelligence is in `project`** (the
`lsp_store`), which owns *which* servers exist, *which* buffers they see, and the *coordinate translation*
between LSP's UTF-16 world and the editor's anchor world. That separation is the single most important
design lesson for Marley: **keep the transport dumb; concentrate the orchestration + coordinate math in
one place.**

---

## 2. The LSP client — `crates/lsp`  `[Zed-derived wiring on a permissive/public spec]`

`crates/lsp/src/lsp.rs` (~2.3k LOC) + `input_handler.rs` (~213 LOC). **Self-contained**: unlike Warp
(which split a separate `jsonrpc` crate), Zed inlines the JSON-RPC transport here; deps are just
`smol`/`futures` for async IO and a Zed **fork** of `lsp-types` (`git = zed-industries/lsp-types`). The
public type surface is `pub use lsp_types::*` — the whole protocol vocabulary is re-exported.

### 2.1 Transport & framing

- **`LanguageServer::new(...)`** spawns the server as a child process (`stdin/stdout/stderr` all piped,
  `kill_on_drop(true)`, `current_dir` = the resolved root). Identity is a `LanguageServerId(usize)` +
  `LanguageServerName(SharedString)`.
- Three background tasks on the gpui executor: **incoming** (`handle_incoming_messages`), **stderr**
  (`handle_stderr`, captured to a log buffer), **outgoing** (`handle_outgoing_messages`).
- **Framing** (`handle_outgoing_messages`): writes `Content-Length: {n}\r\n\r\n{json}` — the exact LSP
  wire format. Reading is `input_handler::LspStdoutHandler`: `read_headers` scans to the `\r\n\r\n`
  delimiter, parses `Content-Length`, then `read_exact`s the body.
- **Backpressure:** the reader feeds a **bounded channel (`INCOMING_MESSAGE_QUEUE_CAPACITY = 128`)**; when
  full, it stops reading stdout so the OS pipe backpressures the *server* rather than buffering unbounded
  in RAM. A nice, cheap robustness property worth copying.

### 2.2 Request / response / notification model

- **Requests** (`request::<T: request::Request>(params, timeout)`): a monotonic `AtomicI32` id, serialize
  `{jsonrpc, id, method, params}`, register a **response handler keyed by `RequestId`** in a map, and await
  a **`oneshot`** channel. The returned future `select!`s the response against a **timeout timer** (default
  `DEFAULT_LSP_REQUEST_TIMEOUT = 120 s`; `Duration::MAX/ZERO` = never). Result is a
  `ConnectionResult::{Result, ConnectionReset, Timeout}`.
- **Cancellation is automatic and load-bearing:** the request future holds a `cancel_on_drop` guard — if
  it's dropped (e.g. the user moves the cursor and the stale completion request is abandoned), the client
  sends **`$/cancelRequest`**. As-you-type features *depend* on this: you fire a completion on every
  keystroke and let drop cancel the losers.
- **Notifications** (`notify::<T: notification::Notification>(params)`): fire-and-forget, serialized and
  queued to the outbound channel.
- **Inbound dispatch:** `input_handler` deserializes each message; if it has a `method` it's a
  notification or server→client request (routed by method string to `notification_handlers`); otherwise
  it's a response (routed by `id` to the pending `response_handlers`). `$/cancelRequest` from the server
  aborts a pending server-request task.

### 2.3 UTF-16 coordinates & the `initialize` handshake

- **`default_initialize_params`** builds the `ClientCapabilities` — this is the negotiation of *what the
  client can render*. Zed advertises `position_encodings: [UTF16]` (**the spec default; positions are
  `(line, UTF-16 code-unit column)`**), and capability flags for every feature: completion
  (`snippet_support`, `resolve_support` for `additionalTextEdits`/`documentation`), hover (Markdown),
  rename (`prepare_support`), code actions (literal + resolve), `publishDiagnostics` (related info +
  tags), formatting/range/on-type, semantic tokens, inlay hints, document symbols, folding, links, colors,
  signature help, and `workspace_edit` with `document_changes` + create/rename/delete + `snippet_edit`.
- **`initialize(params, config, timeout)`** sends the `Initialize` request, stores the returned
  **`ServerCapabilities`** (every later request is gated on these), then sends the `Initialized`
  notification. `add_workspace_folder`/`set_workspace_folders` manage multi-root.

### 2.4 Handler registration, lifecycle, test double

- `on_notification::<T>` / `on_request::<T>` / `on_io` return a **`#[must_use]` `Subscription`** whose
  `Drop` unregisters — an RAII handler-lifetime pattern. `register_buffer`/`unregister_buffer` track open
  documents; `shutdown()` sends `Shutdown`+`Exit` and drains the IO tasks.
- **`FakeLanguageServer`** (behind `test-support`) is a full in-process fake — set request handlers,
  push notifications, drive progress. Essential: it lets every downstream feature be tested with **no real
  server binary**. Marley wants the equivalent from day one.

---

## 3. Orchestration — `project` :: `lsp_store` / `lsp_command`  `[Zed-derived]`

This is the brain of the subsystem: `lsp_store.rs` (~15k LOC) + `lsp_command.rs` (~5k) + the
`lsp_store/` submodule set. **Everything hard lives here.**

### 3.1 Server registry & spawn (per worktree × language)

- **`LocalLspStore`** owns two maps: `language_server_ids: HashMap<LanguageServerSeed,
  UnifiedLanguageServer>` and `language_servers: HashMap<LanguageServerId, LanguageServerState>`.
- **`LanguageServerSeed { worktree_id, name, toolchain, settings }`** is the identity key: the same seed
  **reuses one server across multiple roots** (`UnifiedLanguageServer.project_roots`), so you don't spawn
  five rust-analyzers for five crates in one workspace.
- `LanguageServerState::{ Starting { pending_workspace_folders }, Running { adapter, server, … } }`. A
  **`LanguageServerTree`** maps `ProjectPath → language → server node` to decide which servers apply to a
  file.
- **Spawn flow:** opening a buffer runs `register_buffer_with_language_servers` → reuse an existing server
  or `lsp_tree.walk` to find/lazily-`start_language_server` → `insert_newly_running_language_server` →
  `setup_lsp_messages` wires the notification handlers (diagnostics, progress, log, applyEdit, etc.).

### 3.2 Document sync

On buffer open/edit/save the store sends `textDocument/didOpen` / `didChange` / `didSave` / `didClose`
(the client's `register_buffer` half). Edits are shipped as incremental content changes keyed to a
document version — the server and client must agree on versions so that a diagnostic tagged for version N
can be discarded if the buffer is already at N+3.

### 3.3 The `LspCommand` trait — requests as commands

`lsp_command.rs:92`. **Every request is a struct implementing one trait**, which unifies the local path
and the collab/remote-proto path:

```
trait LspCommand {
    type Response;                              // buffer-anchored result
    type LspRequest: lsp::request::Request;     // the wire request
    type ProtoRequest;                          // collab mirror
    fn check_capabilities(&self, caps) -> bool; // gate on ServerCapabilities
    fn to_lsp(&self, path, buffer, server, cx) -> LspRequest::Params;   // → UTF-16 params
    async fn response_from_lsp(self, msg, store, buffer, server_id, cx) -> Response; // → anchors
    // + to_proto / from_proto / response_to_proto / response_from_proto  (host-delegated)
}
```

The **23 concrete commands** are the request catalog: `PrepareRename`, `PerformRename`, `GetDefinitions`,
`GetDeclarations`, `GetImplementations`, `GetTypeDefinitions`, `GetReferences`, `GetDocumentHighlights`,
`GetDocumentSymbols`, `GetSignatureHelp`, `GetHover`, `GetCompletions`, `GetCodeActions`,
`OnTypeFormatting`, `InlayHints`, `SemanticTokensFull`/`Delta`, `GetCodeLens`, `LinkedEditingRange`,
`GetDocumentDiagnostics`, `GetDocumentColor`, `GetFoldingRanges`, `GetDocumentLinks`. Each stores a
`position: PointUtf16` (or range), converts it in `to_lsp`, and re-anchors the response in
`response_from_lsp`. **This "request = a struct with to_lsp/response_from_lsp" pattern is the single
cleanest thing to copy** — it makes each feature one small, testable unit and centralizes the coordinate
conversion at the two ends.

### 3.4 The coordinate / anchor round-trip (the crux)

`crates/language/src/language.rs:1478-1512` — four tiny functions do the translation:

- `point_to_lsp(PointUtf16) -> lsp::Position`, `range_to_lsp` (guards inverted ranges),
- `point_from_lsp(lsp::Position) -> Unclipped<PointUtf16>`, `range_from_lsp(lsp::Range) ->
  Range<Unclipped<PointUtf16>>` (swaps inverted).

The **`Unclipped<PointUtf16>` newtype is the key insight**: an LSP position from an external server is
*untrusted* — it may point past the line's end, into the middle of a surrogate pair, or into a stale
version. So it arrives wrapped as `Unclipped`, and the store must **clip it against a real buffer
snapshot** (`snapshot.clip_point_utf16(pt, Bias::Left)`) before turning it into an **`Anchor`**
(`anchor_before` / `anchor_after`). Diagnostics are even stored *un-clipped*
(`DiagnosticEntry<Unclipped<PointUtf16>>`) and resolved lazily against whatever snapshot is current. **The
whole subsystem is a pipeline: server UTF-16 → `Unclipped<PointUtf16>` → clip → `Anchor` → survives
edits.** Marley cannot skip a single stage of this — and it has none of it today.

---

## 4. Completions  `[Zed-derived orchestration; public spec]`

- **Request:** `GetCompletions` (`response_from_lsp` handles both `CompletionList` and a bare array, applies
  list-level `defaults`). For each item it computes the **replace range**: an explicit `CompletionTextEdit`
  if present, else inferred from `defaults.edit_range` or the surrounding-word syntax token — then clipped
  to `anchor_before..anchor_after`. As-you-type: fire on keystroke; drop-cancel stale requests (§2.2).
- **The two-step resolve:** the list request returns cheap items (label + sort text). The **`resolve`
  step** (`resolve_completions` / `resolve_completion_local`, gated by `can_resolve_completions`) fetches
  the expensive bits — **documentation, `additionalTextEdits`** — only for the item under the cursor.
  (Zed's capabilities deliberately omit `textEdit` from resolve for speed.)
- **Apply:** `apply_additional_edits_for_completion` resolves the item, pulls `additional_text_edits`
  (e.g. auto-imports), runs them through `edits_from_lsp`, and applies as one buffer **`Transaction`**.
  **Snippets** (`InsertTextFormat::Snippet`) flow via a distinct `LspStoreEvent::SnippetEdit { edits:
  Vec<(lsp::Range, Snippet)> }` so tab-stops are expanded, not inserted literally.
- **Marley today:** none (the #200 ghost text is shell history). **Rebuild:** needs anchors (the completion
  lands async), a completion menu overlay (gpui `anchored()` popover positioned at the cursor anchor), the
  resolve-on-highlight step, and snippet expansion. **Provenance:** client + resolve logic
  `[Marley-original]` from the spec; the menu is `[Marley-original]` gpui.

---

## 5. Diagnostics  `[Zed-derived]` — `crates/diagnostics`

The crate is thin — the actual squiggle drawing lives in the `editor` crate; `diagnostics` owns
aggregation, the message block/hover content, and the status item.

- **Fan-in (in `lsp_store`):** `on_notification::<PublishDiagnostics>` → `adapter.process_diagnostics` →
  **`merge_lsp_diagnostics`**, which sorts by severity, assigns a **`group_id`** (so a primary error and
  its `related_information` render as one group), and produces `DocumentDiagnostics { diagnostics:
  Vec<DiagnosticEntry<Unclipped<PointUtf16>>> }`. Stored per `WorktreeId → RelPath`; a `DiagnosticSummary`
  tracks counts. Pull diagnostics (`textDocument/diagnostic`) coexist with pushed ones.
- **Squiggle:** the editor draws a gpui **`HighlightStyle { underline: UnderlineStyle { wavy: true,
  thickness: 1.0, color }, fade_out }`** over the diagnostic's anchor range (`display_map.rs`).
  Unnecessary-code (unused) fades out instead of underlining.
- **Gutter / scrollbar markers:** per-severity **`ColoredRange`** marks in the gutter and scrollbar track
  (`element.rs`).
- **Severity → color:** `DiagnosticSeverity::{ERROR, WARNING, INFORMATION, HINT}` maps to
  `cx.theme().status()` (`StatusColors`) — one theme lookup, no hard-coded reds.
- **Inline message block:** `DiagnosticRenderer` (registered via `editor::set_diagnostic_renderer`)
  produces `DiagnosticBlock`s rendered as an `h_flex()` with `border_l_2()`, severity `bg`/`border`, a
  `MarkdownElement`, and a copy button — inserted as an editor block at `BlockPlacement::Near(anchor)`.
- **Hover:** `render_hover` returns an `Entity<Markdown>` for the group under the cursor; the editor's
  `hover_popover` displays it. Related-info entries become in-document markdown links that navigate between
  primary and related sites.
- **Project panel:** `ProjectDiagnosticsEditor` = a `MultiBuffer` + `Editor::for_multibuffer` that
  aggregates every file's diagnostics into one scrollable buffer, updated debounced (~50 ms), sorted by
  location, grouped by `group_id`. **Status item:** `DiagnosticIndicator` (`StatusItemView`) shows the
  error/warning counts and the diagnostic under the cursor.
- **Marley today:** none. **Rebuild:** `publishDiagnostics` handler → anchored `DiagnosticEntry`s → a wavy
  underline decoration + a **gutter/scrollbar marker reusing #198's `scrollbar_thumb` track** + a hover
  popover (`anchored()`) + optionally a project panel. **Provenance:** all `[Marley-original]` on gpui;
  the `Diagnostic` shape is `[permissive/public]` `lsp-types`.

---

## 6. Navigation: go-to-definition / references / implementation  `[Zed-derived; reuses Marley #196]`

- `GetDefinitions` / `GetDeclarations` / `GetImplementations` / `GetTypeDefinitions` each return
  **`Vec<LocationLink>`** (a `LocationLink` = `target_uri` + `target_range` + an optional origin range for
  the underline). `GetReferences` returns `Vec<Location>`.
- The editor resolves each `LocationLink` to a `(file, line, col)` and opens it — single result jumps
  directly, multiple results open a picker/multibuffer.
- **This is the one feature where Marley already has the back half.** The #196 mechanism —
  `open_link_target(&LinkTarget)` at `crates/marley_app/src/app.rs:2317`, with `LinkTarget` carrying
  `:line:col` (#212) — is exactly "open file at line:col in the editor." A goto-definition response is just
  a `LocationLink` whose `target_uri`+`target_range.start` becomes a `LinkTarget{path, line, col}`. **The
  reuse is: LSP `LocationLink` → existing #196 open path.** The only new work is the *request* (fire
  `GetDefinitions` at the cursor anchor) and multi-result disambiguation.
- **Provenance:** the request is `[Marley-original]`; the open path is Marley's existing `[Marley-original]`
  #196 code.

---

## 7. Hover  `[Zed-derived; public spec]`

- `GetHover` sends `textDocument/hover` at the cursor and returns Markdown content (`MarkupContent`) plus an
  optional range to highlight. Zed renders it in a **`hover_popover`** — a Markdown block anchored to the
  hovered token, debounced on cursor dwell, dismissed on move. Diagnostics-at-point and type/doc hover
  share the same popover.
- **Marley today:** none. **Rebuild:** a dwell timer → `GetHover` at the cursor anchor → a gpui
  `anchored()` Markdown popover positioned at the token. Marley has no Markdown renderer for popovers yet,
  so plain-text first, Markdown later. **Provenance:** `[Marley-original]`.

---

## 8. Rename · code actions · formatting (workspace-edit apply)  `[Zed-derived]`

All three converge on **one apply path**, which is why they're grouped.

- **Rename:** `PrepareRename` (validates the token + returns its range) → user types the new name →
  `PerformRename`, whose **`Response = ProjectTransaction`**. Code actions: `GetCodeActions` →
  `apply_code_action` (which may lazily `resolve` the action's edit) → the same `ProjectTransaction`.
- **The workspace-edit deserializer is the heart:** `deserialize_workspace_edit` normalizes a
  `WorkspaceEdit` (new `document_changes` — text edits + create/rename/delete file operations — or legacy
  `changes`), performs file ops via `fs`, and for every text edit calls **`edits_from_lsp`**:
  `range_from_lsp` → clip → **diff multi-line replacements with `language::text_diff`** (so anchors in
  unchanged regions survive) → coalesce adjacent edits → `Vec<(Range<Anchor>, Arc<str>)>`, packaged as a
  `ProjectTransaction` (one undo entry across possibly many files).
- **Formatting:** `format_locally` picks a formatter list from `(FormatTrigger, FormatOnSave)` settings.
  `Formatter::Auto` resolves to **Prettier if allowed, else the LSP formatter**; `apply_formatter`
  dispatches to `prettier_store::format_with_prettier`, `format_via_lsp` (`textDocument/formatting` →
  again a set of edits), or an external command. `code_actions_on_format` runs source-action code-actions
  (e.g. organize-imports) as part of formatting. **Format-on-save** is a hook on the save path.
- **Marley today:** none — but the **apply-seam is ready**: `EditOrigin::Agent` is exactly the origin for a
  programmatic (non-keystroke) buffer write. A workspace-edit becomes a batch of
  `buffer.edit(range, text, EditOrigin::Agent)` calls. **Rebuild sequence:** anchors → the workspace-edit
  deserializer (`Vec<(Range<Anchor>, String)>`) → apply via `EditOrigin::Agent` → group as one undo. This
  is the **`#252` format-on-save** ticket's foundation. **Provenance:** deserializer + apply
  `[Marley-original]`; `WorkspaceEdit`/`TextEdit` shapes `[permissive/public]`.

---

## 9. Edit prediction — `edit_prediction_context` + `edit_prediction_ui`  `[the BRAIN seam]`

Zed's **inline AI edits** ("predict my next multi-line edit"). Architecturally this is *not* an LSP
feature — LSP is only a **context source** here — and Zed draws exactly the boundary Marley needs: a
**provider-agnostic prediction store** (the brain: context assembly + prediction + interpolation) vs. the
**GPL editor** that only renders ghost text and applies edits. **This is the model for Marley's brain
owning AI edits while the GPL editor stays clean.**

- **Two context assemblers feed the model:**
  - **LSP-driven `RelatedExcerptStore`** (`edit_prediction_context.rs`): debounced on cursor move, it
    collects identifiers within ±3 lines of the cursor (tree-sitter highlight captures), ranks the nearest
    ~32, and **calls the LSP per identifier** — `project.workspace_definitions()` /
    `workspace_type_definitions()` — caching the `LocationLink` results, then expands each definition to
    its outline item (eliding large bodies) into `RelatedExcerpt { context_source: ContextSource::Lsp }`.
    So **goto-definition is repurposed as RAG**: "pull the definitions of the symbols near my cursor into
    the model's context." (`fake_definition_lsp.rs` is the test double that confirms this keys off
    `LocationLink`.)
  - **Local retrieval `collect_editable_context`** (`editable_context.rs`) over `ContextSource::{CursorExcerpt,
    CurrentFile, EditHistory, GitLog, Bm25, …}`: a **BM25** index (`bm25_context.rs`: `git ls-files`, 40-line
    chunks, camelCase/snake tokenization, classic BM25 k1=1.2/b=0.75, top-12) and a **git-log co-change**
    index (`git_log_context.rs`: `git log -5000 --name-only` → symmetric co-edit frequency → top-10 files
    historically edited together). Disjoint-merged and byte-budgeted.
- **Apply / interpolation:** a prediction is `EditPrediction { edits: Arc<[(Range<Anchor>, Arc<str>)]>,
  edit_preview }`. **`interpolate_edits`** continuously rebases the model's edits against the keystrokes
  typed since the prediction snapshot — if your typing is a prefix of a predicted insertion it survives;
  otherwise the prediction is rejected. **The brain never calls `buffer.edit`** — the editor applies the
  interpolated edits on accept; the delegate's `accept()` only records telemetry.
- **UX:** `SuggestionDisplayType::{ GhostText, DiffPopover, Jump }`; **Tab accepts**; an
  `EditPredictionButton` status item toggles per-provider (Zed/Copilot/Codestral/…); a
  `RatePredictionsModal` previews the predicted diff in a read-only multibuffer and steps through
  **multi-location** edits (`NextEdit`/`PreviousEdit`).
- **Vs LSP completion:** completion = a single-symbol textual insertion at one point from the language
  server; edit prediction = **multi-line, multi-location structural edits + cross-file jumps** from an AI
  model, interpolated against live typing. Different providers, different UX, different layer.
- **Marley relevance:** this is the **brain's** territory. Marley's brain would own the context assembly
  (cursor excerpt + BM25 + git-log + *optionally* LSP definitions once §6 exists) and the prediction; the
  GPL editor renders ghost text (Marley's #200 ghost-text render path is a starting point) and applies via
  `EditOrigin::Agent`. **Provenance:** `[Marley-original]` in the brain; the `EditPrediction`/interpolation
  design is `[Zed-derived]` architecture but must be re-implemented as clean brain code (copyleft must not
  reach the sold layer).

---

## 10. Observability — `language_tools`  `[Zed-derived]`

The LSP debugging/observability surface — small but strategically interesting for Marley's brain (which
wants to *observe* servers, not just use them).

- **`LspLogView`** — a per-server **RPC trace + log viewer**. `enable_rpc_trace_for_language_server` taps
  the client's `on_io` hook; a **`LogStore`** (`lsp_store::log_store`) captures server log messages
  (`window/logMessage`) and every RPC in/out (`LogKind`, `Message`). This is "watch the language server's
  conversation live."
- **`LspButton`** — a `StatusItemView` showing per-server **health**: `LanguageServerHealthStatus` over
  `ServerHealth::{Ok, Warning, Error}` and `BinaryStatus::{None, Starting, Stopping, Stopped, Failed}` →
  a color + label ("Running" / "Starting…" / "Stopped" / "Error"). A popover menu lists servers and
  actions (restart, view log).
- Also in-crate: `SyntaxTreeView`, `HighlightsTreeView`, `KeyContextView` (tree-sitter / keymap debug —
  adjacent, not LSP).
- **Marley relevance:** the **brain observing agent/tool processes** is a stated Marley intent; the
  `on_io`-tap + `LogStore` + health-status pattern is directly reusable for surfacing what a language
  server (or any managed subprocess) is doing. **Provenance:** `[Marley-original]` on gpui.

---

## 11. Marley: today (nothing) → the rebuild

### 11.1 The prerequisite — an **anchor layer** must land first

Every result in §§4-9 is **asynchronous** and **positional**. A completion, a diagnostic, a hover, a
rename edit, a predicted edit — all arrive tens-to-hundreds of ms after the request, referencing a buffer
version the user has already typed past. Marley's `Buffer` edits by `Range<CharOffset>` with **no stable
positions** (verified: no `Anchor` type exists). Landing an LSP result by raw offset would put it in the
wrong place the moment the user keeps typing.

**So the sequence is not negotiable: anchors → LSP.** The anchor layer Marley needs (mirroring Zed's
`text::Anchor`): a position that (a) is bias-aware (`Bias::Left/Right` — does it stick to the char before
or after an insertion at its exact spot), (b) survives edits by rebasing through the edit delta, and (c)
converts to/from `CharOffset` against a given buffer version. On top of anchors, Marley also needs the
**UTF-16 `PointUtf16` + `Unclipped` + `clip_point_utf16`** conversion (§3.4), because the server speaks
UTF-16 and Marley's buffer speaks char offsets — those are *different* for any non-BMP character (emoji,
which Marley's buffer tests already exercise).

### 11.2 Sequencing (dependency order)

1. **Anchor layer** in `crates/editor` — `Anchor`, `anchor_at(offset, bias)`, `to_offset(version)`,
   rebase-through-edit. *(Blocks everything below. Standalone, testable, no LSP.)*
2. **UTF-16 coordinate module** — `PointUtf16`, `Unclipped<PointUtf16>`, `point_to/from_lsp`,
   `range_to/from_lsp`, `clip_point_utf16`. *(Pure functions; unit-testable against the emoji buffer.)*
3. **LSP client** (`crate: marley_lsp`) — spawn/manage a server, Content-Length framing, request/notify
   with **drop-cancellation**, `initialize` capability handshake, a `FakeLanguageServer` for tests. Built
   **from the spec + MIT `lsp-types`.**
4. **Orchestration** (`LspStore`) — server registry per (workspace × language), document sync
   (didOpen/didChange), the **request-as-command** pattern (`to_lsp`/`response_from_lsp`), the coordinate
   round-trip.
5. **Features, cheapest-value-first:** **diagnostics** (§5, pure inbound notification, no request
   round-trip, reuses #198 scrollbar) → **goto-definition** (§6, reuses #196 open path) → **hover** (§7) →
   **completions** (§4, needs the menu overlay + resolve) → **rename/code-action/format** (§8, needs the
   workspace-edit apply via `EditOrigin::Agent`; format-on-save = #252).
6. **Edit prediction** (§9) — brain layer, independent of the LSP *features*; can reuse the LSP *client*
   for definition-context once §6 lands.

### 11.3 Per-feature rebuild table

| Feature | Marley today | Rebuild (needs anchors first) | Reuses | Provenance |
|---|---|---|---|---|
| LSP client | none | spawn + framed stdio + request/notify + drop-cancel + `initialize` | — | client `[M-orig]`; spec + `lsp-types` `[public]` |
| Coordinates | none | `PointUtf16`/`Unclipped`/clip + `*_from_lsp` | emoji buffer tests | `[M-orig]` on `[public]` spec |
| Completions | #200 = shell history only | menu popover (`anchored()`) + resolve-on-highlight + snippets | #200 ghost render | `[M-orig]` |
| Diagnostics | none | `publishDiagnostics` → anchored entries → wavy underline + gutter marks + hover | **#198 scrollbar** | `[M-orig]`; `Diagnostic` `[public]` |
| Goto def/refs/impl | none | fire `GetDefinitions` at cursor anchor → `LocationLink` → open | **#196 `open_link_target` + `:line:col`** | request `[M-orig]`; open path existing `[M-orig]` |
| Hover | none | dwell → `GetHover` → `anchored()` popover | — | `[M-orig]` |
| Rename/code-action/format | none | workspace-edit deserializer → `edits_from_lsp` → apply | **`EditOrigin::Agent`**; #252 on-save | `[M-orig]`; `WorkspaceEdit` `[public]` |
| Edit prediction | none | context (BM25 + git-log + LSP defs) + interpolate + ghost/diff UX | #200 ghost; LSP client | **brain `[M-orig]`**; design `[Zed-derived]` |
| LSP log / health | none | `on_io` tap + `LogStore` + status button | — | `[M-orig]` |

---

## 12. Provenance summary

- **`[permissive/public]` — reuse freely, owe nothing to Zed:** the LSP protocol (published Microsoft
  spec), JSON-RPC 2.0 framing, and the `lsp-types` **data structures** (crates.io = MIT; Zed uses a fork,
  but the crates.io MIT crate is available to Marley). **Marley implements its client from the spec.**
- **`[Zed-derived]` — GPL; study the design, write our own code:** `crates/lsp`'s concrete client wiring;
  the `LocalLspStore` orchestration, the `LspCommand` trait, the `Unclipped<PointUtf16>`→`Anchor` pipeline;
  the diagnostics renderer; the edit-prediction context/interpolation architecture. These inform Marley's
  design; none of the source is copied.
- **`[Marley-original]`:** Marley's anchor layer, `marley_lsp` client, `LspStore`, every feature renderer
  (completion menu, diagnostic decorations, hover popover), and the **brain's** edit-prediction engine.
- **Copyleft boundary:** LSP *features* live in Marley's GPL editor layer (fine). **Edit prediction / AI
  edits must be clean-room brain code** — the `[Zed-derived]` architecture of §9 is a design reference
  only; the sold brain cannot contain GPL-derived source.

---

## 13. Key files (in the Zed clone)

- `crates/lsp/src/lsp.rs` — `LanguageServer` (spawn, `request`/`notify`, `initialize`,
  `default_initialize_params` capabilities, drop-cancellation), `FakeLanguageServer`.
- `crates/lsp/src/input_handler.rs` — `LspStdoutHandler`, `read_headers`, Content-Length framing,
  bounded-channel backpressure.
- `crates/language/src/language.rs:1478-1512` — `point_to/from_lsp`, `range_to/from_lsp`,
  `Unclipped<PointUtf16>` (the coordinate crux).
- `crates/project/src/lsp_store.rs` — `LocalLspStore`, server registry (`LanguageServerSeed`,
  `LanguageServerTree`), `setup_lsp_messages`, `merge_lsp_diagnostics`, `deserialize_workspace_edit` /
  `edits_from_lsp`, completion resolve/apply, `format_locally`.
- `crates/project/src/lsp_command.rs` — the `LspCommand` trait + the 23 request commands.
- `crates/project/src/lsp_store/` — per-feature submodules (`document_symbols`, `inlay_hints`,
  `semantic_tokens`, `code_lens`, `document_colors`, `document_links`, `folding_ranges`, `log_store`,
  server `*_ext` extension requests).
- `crates/project/src/prettier_store.rs` — Prettier integration for formatting.
- `crates/diagnostics/src/diagnostic_renderer.rs` — `DiagnosticRenderer`, `DiagnosticBlock`, severity→
  `StatusColors`, `render_hover`.
- `crates/diagnostics/src/diagnostics.rs` / `buffer_diagnostics.rs` / `items.rs` —
  `ProjectDiagnosticsEditor`, per-buffer view, `DiagnosticIndicator` status item.
- `crates/edit_prediction_context/src/{edit_prediction_context,editable_context,bm25_context,git_log_context}.rs`
  — the context RAG (LSP-definition + BM25 + git-log).
- `crates/edit_prediction_ui/src/{edit_prediction_button,rate_prediction_modal}.rs` — inline AI-edit UX.
- `crates/language_tools/src/{lsp_log_view,lsp_button}.rs` — RPC trace + server-health observability.

---

## 14. Open questions (for round 2)

1. **Anchor design.** Marley's buffer is a rope of `CharOffset`-indexed items; what's the cheapest anchor
   representation (fractional index vs. version-vector like Zed's `text::Anchor`) that rebases through the
   existing `EditDelta`? This is the gating decision.
2. **Which server(s) first.** rust-analyzer (Marley is Rust) is the obvious first target; does it need the
   `experimental`/`rust-analyzer.*` extension requests (`rust_analyzer_ext.rs`) or does vanilla LSP
   suffice for MVP (diagnostics + goto + hover)?
3. **Completion menu vs. the #200 ghost path.** Is the completion UI a dropdown menu (Zed-style) or an
   inline ghost (extending #200)? Different overlay mechanics.
4. **Server install/discovery.** Zed auto-installs servers (node runtime, downloads). Does Marley assume
   a user-provided binary on `PATH` (simpler, offline) or take on installation?
5. **Brain vs. editor split for edit prediction.** Confirm the exact trait seam (Zed's
   `EditPredictionDelegate`) so the brain owns context+prediction and the GPL editor owns render+apply —
   the boundary that keeps the sold layer clean.
