# Marley — Crate Triage & Health

> **Status:** updated at **M15** ("The Editable Editor"). Two layers: **§A — the M15 health report** for
> the 15 crates that shipped in `crates/` at M15 (the current, load-bearing part — though one of them,
> `marley_forge_client`, has since been **deleted**: the 2026-08-09 scrap-forge product rip, #411), and **§B — the original
> 77-Warp-crate triage** (2026-06-27, the clean-room REUSE / REIMPLEMENT / SKIP verdict — retained as the
> planning + licensing/provenance reference, superseded by §A for live status). Companion to
> [crate-map.md](crate-map.md) (what each crate *is*) and [clean-build-plan.md](clean-build-plan.md).

## §A — Marley crate health (M15)

All **15 workspace crates were BUILT and green** at this M15 snapshot (`marley_forge_client` deleted since — #411). Each **🟢 PURE** seam is gated at **cov/MSI 100** (no
display/IO/FFI). Each **🟡 SHIM** crate holds that bar on its pure core and masks exactly one
ACCEPTED-UNTESTABLE boundary (`#[mutants::skip]` + coverage-excluded, asserted by the gate-15 headed
harness); `· N` is its skip-site count. "~Tests" = in-crate test fns (a cheap real proxy; the gate
enforces the actual coverage). "Since" = the milestone the crate was born (it may have grown since).

| Crate (pkg if different) | Since | ~Tests | Seam | Health as of M15 |
|---|---|---:|---|---|
| `marley_text_offsets` | M0 | 27 | 🟢 PURE | The one `CharOffset`/`ByteOffset` vocabulary; cov/MSI 100. Foundational, stable. |
| `marley_util` | M0 | 16 | 🟢 PURE | Value types (`FileId`/`HostId`/`StandardizedPath`…) built **ahead of demand — currently orphan**: no other crate depends on it yet. Healthy but unwired. |
| `marley_core` | M0 | 22 | 🟢 PURE | `SessionId` + `~/.marley` paths + `Config` + flag registry; offline, no cloud/auth. Stable. |
| `marley_command` | M0 | 17 | 🟢 PURE | The non-PTY spawn seam; real-subprocess integration tests (no shim). **Unix-only** — Windows `CREATE_NO_WINDOW`/`JobObject` deferred to TICKET-004b (needs a Windows CI runner). |
| `marley_visual_harness` | M0 | 158 | 🟡 SHIM · 14 | Gate-15 apparatus; pure `geometry`/`image_diff`/`ax_parse` + masked `launch`/`capture`/`ax_exec`. Load-bearing for every `visual_acceptance`. |
| `marley_editor` (`editor`) | M1.A→M15 | 46 +proptest | 🟢 PURE | Grew from the M1.A prompt buffer into the **full M15 editable editor** (undo/redo + coalescing, selection, char/word/line/vertical movement). ⚠️ `editor.md`'s status line still reads "M1.A subset" — **stale**, predates the M15 editable-editor train (#249–258). |
| `marley_terminal` (`terminal_blocks`) | M1 | 108 | 🟡 SHIM · 9 | Block model + styled runs + OSC 8 links over the alacritty grid; raw PTY (`pty_os`) masked. Mature. |
| `marley_ui_components` (`ui_components`) | M1.A→M1.B | 32 | 🟡 SHIM · 9 | `ThemeColors` value vocab + the 5 widgets; gpui `render` masked, gallery-asserted. |
| `marley_settings` | M1.B | 4 +trybuild | 🟢 PURE | Typed declarative TOML settings; the theme-pick / RemoteHosts / Workflows round-trips ride it. |
| `marley_app` (`marley`) | M1.A→M15 | 362 | 🟡 SHIM · 105 | The **22k-LoC apex cockpit**; a huge pure wiring surface + the single `app.rs` gpui shim. Where the workspace/pane algebra, palette, keymap, editor surface, docks, and agent panes live. |
| `marley_project` | M2.A | 18 | 🟢 PURE | Git-root discovery (`discover_in`); std-only. |
| `marley_search_core` | M2.A | 4 | 🟢 PURE | `fuzzy_score`/`fuzzy_rank` over nucleo; small + fully covered. |
| `marley_agent` | M2.B | 5 | 🟢 PURE | Agent kind/status model; launch + observation live in the app shim. |
| `marley_remote` | M3.A | 10 | 🟢 PURE | Ssh-target parse → safe `ssh` argv; holds no secrets. Wired (a remote pane spawns the user's `ssh`); the broader ops/remote story grows later. |

### §A.1 — REIMPLEMENT verdicts that have shipped (10 of 29)

Ten of §B's 29 REIMPLEMENT verdicts have materialized as **named Marley crates**:

| Warp crate (REIMPLEMENT, §B) | Shipped as | Since |
|---|---|---|
| `string-offset` | `marley_text_offsets` | M0 |
| `warp_util` (value-type subset) | `marley_util` | M0 |
| `warp_core` (+ `warp_channel_config` + `warp_features`) | `marley_core` | M0 |
| `command` | `marley_command` | M0 |
| `warp_editor` | `marley_editor` | M1.A→M15 |
| `warp_terminal` | `marley_terminal` | M1 |
| `ui_components` | `marley_ui_components` | M1.A/B |
| `warp` (apex app) | `marley_app` | M1.A→M15 |
| `settings` (+ `settings_value` + `_derive`) | `marley_settings` | M1.B |
| `warp_search_core` (fuzzy slice) | `marley_search_core` | M2.A |

Plus the three **INVENT** crates with no Warp lineage: `marley_project`, `marley_agent`, `marley_remote`
(a fourth, `marley_forge_client`, was deleted at #411 — the scrap-forge rip).

### §A.2 — still deferred / folded-in / not-yet-a-crate

- **Folded into existing crates, not extracted:** session + layout persistence (`persistence`) rides the
  `marley_app` grid codec (`t=<cwd>` / `c=<path>`); the file/project tree (`repo_metadata`, `warp_files`)
  lives in `marley_app` + `marley_project`; syntax highlight and completions are the in-app `code_syntax`
  and `complete` modules (not yet `syntax_tree`/`languages`/`warp_completer` crates).
- **Roadmap (no crate yet):** the `ai` action model, `warp_cli` control verbs, `local_control`,
  `input_classifier` (+ `natural_language_detection`), `markdown_parser`, `node_runtime`, and the
  asset crates (`asset_cache`/`asset_macro`/`warp_assets` — no remote-asset surface yet).
- **Deferred past v1:** `vim` (opt-in modal FSA).
- **Built but unwired:** `marley_util` (orphan — see §A).

---

## §B — the original Warp triage (planning reference)

> The pre-build clean-room verdict (2026-06-27) for all 77 Warp crates. **Superseded by §A for live
> status**; kept as the licensing/provenance record and the REUSE shopping list, and cross-linked from
> [crate-map.md](crate-map.md). The **Forge**-facing INVENT items below (Forge detail panes, the
> `marley:`/`forge:` search filters, Forge-owned ops/remote, the M2 "first Forge pane" milestone) are
> **retired** — the 2026-08-09 scrap-forge pivot (#409/#410/#411) scrapped Forge for Marley, process and
> product; they read as the 2026-06 plan, not current design.

We triage all **77 Warp crates** into exactly three actionable buckets: **REUSE** a permissive (MIT/Apache/BSD)
third-party crate that already does the job; **REIMPLEMENT** the Warp-specific behavior/UX fresh in clean Rust
from our spec docs (never from the AGPL source); or **SKIP** Warp's cloud/account/telemetry/web-shell — or anything
the Ignibyte **INVENT** layer replaces. The shape of the result: the hardest, most generic foundations (GPU UI,
terminal grid, rope, fuzzy match, tree-sitter, http, mcp, json-rpc, logging, fs-watch) are all **REUSE** — we never
hand-roll a renderer, a B-tree, or an LSP transport. The Warp *feel* (Blocks, the editor/buffer core, the unified
prompt + classifier, completions, the command palette / search mixer, settings framework, themes, the apex app shell)
is **REIMPLEMENT** — that is the real build. Roughly a third of Warp — every cloud/auth/Drive/telemetry/web-shell
surface — is **SKIP**, deleted outright because Marley is local-first and the Ignibyte layer (project model, agents,
brain-MCP, Forge, ops) supplants it.

### Bucket tally

| Bucket | Count | Meaning |
|---|---:|---|
| **REUSE** | 23 | Integrate a permissive crate; do not reimplement. Unrestricted. |
| **REIMPLEMENT** | 29 | Build fresh in clean Rust from our spec docs (not the AGPL source). The real work. |
| **SKIP** | 25 | Warp cloud/account/telemetry/web-shell, or replaced by the INVENT layer. Deleted. |
| **Total** | **77** | |

---

## REUSE (23) — permissive foundations, integrate don't rewrite

| Crate | Replacement crate(s) | Rationale |
|---|---|---|
| **warpui_core** | **gpui** (Zed, Apache-2.0) | In-house GPUI-style retained-mode toolkit (App → entities → handles → Element → Presenter → Scene). Feature-for-feature parallel of gpui, the plan's canonical GPU-UI reuse target. 35 dependents — exactly what we must not clean-room. Build Marley's View/Element trees on gpui. |
| **warpui** | **gpui** platform/render backends (Apache-2.0) | Pure platform layer (winit windowing, wgpu/Metal render, font-kit/cosmic-text glyphs). Meaningless without warpui_core; reusing gpui inherits its window + GPU surface + glyph atlas + font discovery. No Warp logic ("no de-auth/de-Warp"). |
| **warpui_extras** | **keyring** + **directories** + serde/toml (or `config`) | Two thin wrappers over OS facilities. `secure_storage` = Keychain/Secret-Service/DPAPI = the `keyring` crate (Marley needs a credential store for the Claude API key). `user_preferences` = persisted settings = directories + serde. Warp's auth-token use is SKIP; the storage capability is REUSE. |
| **sum_tree** | **sum_tree** (Zed, Apache-2.0) or **ropey** (MIT) | Direct descendant of Zed's sum_tree, which ships standalone on crates.io, dependency-free, zero Warp branding. Pure COW B-tree infra. Plan lists "rope → ropey or sum_tree if standalone" as REUSE. |
| **fuzzy_match** | **nucleo** (MIT, Helix) | Thin adapter over SkimMatcherV2 + a glob helper. nucleo returns score + matched indices (= `FuzzyMatchResult`), faster and permissive. Palette/file-finder/completion ranking integrate it directly; use `globset` for the wildcard path. |
| **warp_terminal** | **alacritty_terminal** (Apache-2.0) + **vte** | Doc: entire `model/` subtree is "adapted from Alacritty" under LICENSE-ALACRITTY. The substance (cell grid, ANSI params, C0/escape constants, kitty keyboard, TermMode flags) is upstream Alacritty = the permissive crate. Only `BlockId/BlockIndex` newtypes are Warp-specific → folded into our block model. |
| **ipc** | **tarpc** (MIT) over **interprocess** (MIT/Apache) | Generic typed RPC: `Server`/`Service`/`Client` over length-prefixed bincode on UDS/named pipes. Zero Warp business logic. tarpc gives the same typed contract; interprocess (already its dep) is the transport. For the plugin/agent host channel, standardize on **rmcp**. |
| **mcp** | **rmcp** (official MCP SDK, MIT/Apache) + **reqwest** | Doc: "a thin Warp-flavoured façade over the upstream rmcp SDK." Integrate rmcp directly (stdio/HTTP/SSE transports, capability negotiation, tool/resource enumeration, OAuth/DCR). Add only a small (S) provider-neutral gating/offline-creds wrapper. No Warp-cloud coupling. |
| **computer_use** | **enigo** (MIT/Apache) + **xcap**/screenshots-rs + **image** | Provider-neutral OS automation, zero cloud/auth coupling. Synthetic input + screen capture across macOS/Win/X11/Wayland is exactly enigo + xcap, replacing Warp's hand-rolled objc2/windows/x11rb/ashpd backends. Thin (S) `Actor` trait + noop fallback for headless. |
| **jsonrpc** | **async-lsp** / **lsp-server** (MIT/Apache); **jsonrpsee** for generic | Brand-neutral JSON-RPC 2.0 client whose only consumer is the LSP client. Mature permissive crates cover it exactly. Lowest-value crate to port. Only coupling is the warpui_core executor → swap for Tokio. |
| **cloud_object_persistence** | **diesel** (sqlite+chrono) or **rusqlite**/**sqlx** | Actual engine is diesel/SQLite; the crate's own content is thin Warp-Drive glue (callback upsert/delete, bincode guests, cloud force-refresh) that exists only for cloud sync — SKIP that. Reuse a SQLite/ORM crate for the local object cache + a trivial fresh schema. |
| **websocket** | **tokio-tungstenite** (+ tokio-rustls / rustls-platform-verifier); **ws_stream_wasm** wasm | Vendor-neutral, dependency-free cross-target WS shim over already-permissive libs. No Warp feel. Pull tokio-tungstenite + rustls directly. Drop the graphql_ws_client coupling. Can go unused if Marley ships no realtime subscriptions. |
| **http_client** | **reqwest** + **reqwest-eventsource**/eventsource-stream (SSE) | Opinionated wrapper around reqwest. The substance (reqwest + SSE) is permissive — reuse it. Everything Warp-specific layered on (X-Warp-* headers, IAP challenge, WARP_EXTRA_HTTP_HEADERS) is SKIP. A shared client is a trivial in-house S wrapper. |
| **http_server** | **axum** (already in Marley's stack via rusty-server) | Tiny gpui singleton binding axum on 127.0.0.1 to catch OAuth callbacks — and the OAuth purpose is SKIP. The remainder (host a loopback Router) is reused straight from axum. Drop the PORT_BASE=9277 easter-egg and per-channel offsets. |
| **virtual-fs** | **assert_fs** (MIT/Apache) (+ tempfile/dunce) | Despite the name, a dev-only test-fixture filesystem builder (tempfile + typed-path). assert_fs covers it. Add a thin local helper for the niche bits (0o755 mock exes, a bundled git-repo fixture, a `target/{debug,release}` locator). The embedded `Warp` binary-name helper → Marley name. |
| **prevent_sleep** | **keepawake** (MIT/Apache) or nosleep | Generic OS power-management (NSProcessInfo / SetThreadExecutionState). The RAII-guard-while-work-in-flight pattern is keepawake off the shelf. Add a tiny `Stream` adapter to hold the guard for a stream lifetime. Useful for long agent/terminal sessions. |
| **watcher** | **notify** + **notify-debouncer-full** (MIT/Apache) | The hard part (cross-platform recursive watching + debouncing) is the notify crates it already wraps — integrate directly. The thin bespoke part (warpui_core binding + `deduplicate_and_merge_raw_notifier_events`) is a small (S) adapter on Marley's model loop. |
| **warp_assets** | **rust-embed** (MIT) | Mechanical rust-embed glue: a zero-sized derive + a one-method `AssetProvider` delegating to `RustEmbed::get`. Reuse rust-embed + a ~10-line provider shim; the embedded payload (where branding lives) is swapped, not reimplemented. |
| **warp_logging** | **tracing** + **tracing-subscriber** + **tracing-appender** + **log-panics** | File-vs-stderr, size/startup rotation, panic capture = fully covered by the tracing ecosystem (tracing-appender rolling files). Only a tiny `init(LogConfig)` + `marley.log` naming is app-specific. SKIP: Sentry crash_reporting (off) + support-bundle zip. |
| **simple_logger** | **tracing-appender** (RollingFileAppender + non_blocking) | Generic subprocess log-stream plumbing (child-session stderr to disk). tracing-appender covers rotation + non-blocking. Only the singleton path-reservation registry is a small (S) wrapper. `secure_state_dir`/`WARP_LOGS_DIR` → marley_core path rebrand. |
| **handlebars** | **handlebars** (real crates.io, MIT/Apache) or minijinja/tinytemplate | NOT the real crate — Warp's homegrown 14-line `{{name}}`-only substitution shadowing the name. Zero value porting a hand-rolled subset; the mature engine does more for free. Preserve "unknown vars left literal" if bug-parity wanted. Delete the in-tree copy. |
| **warp_ripgrep** | **grep**/**grep-regex**/**grep-searcher** + **ignore** (BurntSushi, MIT/Unlicense) | Thin wrapper over the ripgrep library crates (RegexMatcherBuilder, WalkBuilder, SearcherBuilder). Integrate them directly. The Warp out-of-process plumbing (re-exec via current_exe, JSON-over-stdout, parent-death watchdog) is not needed — call the `grep` crates in a Tokio task. |
| **lsp** | **async-lsp** (MIT) + **lsp-types** 0.97 (MIT) | Doc: generic feature-plumbing, no Warp branding, "essentially a JSON-RPC client with LSP semantics" over stdio Content-Length framing = exactly async-lsp/lsp-types. Only the small `LspManagerModel` routing + optional auto-install is bespoke (S) glue. Feature-gate off for slim builds. |

---

## REIMPLEMENT (29) — Warp behavior, built fresh from spec (sorted L → S)

### Complexity L (10) — the real work

| Crate | Cx | Behavior to mimic / reuse underneath |
|---|---|---|
| **warp_editor** | L | Rope-backed buffer with snapshot/version model, stable anchors across edits, multi-cursor selection, grouped undo/redo, in-buffer find/replace, and a shared line-layout + decoration pipeline (input editor + rich-text blocks). The load-bearing text-editing core. Reuse underneath: **ropey**/**sum_tree** (rope), **imara-diff**, **regex-automata** (find), **pulldown-cmark** (md). Defer ipynb/mermaid to v1+. |
| **warp** | L | The apex app crate: `run()`/`run_internal(LaunchMode)` bootstrap, `RootView` + workspace/pane-group layout, command palette, settings + settings_view, themes/appearance, multi-headed single-lib/many-bins channel pattern. Built on **gpui**. SKIP-class deps (auth, firebase, cloud_objects, voice, onboarding, telemetry) omitted; the Ignibyte project/agents/brain/Forge/ops panels are INVENTed here. |
| **ai** | L | The agent action / action-result type system (`AIAgentActionType` / `AIAgentActionResultType`) the LLM writes and the UI reads — the local domain model of an agent turn (read/edit/glob/grep/search-codebase/call-mcp-tool/use-computer/run-agents/ask-user + result variants). Marley's panel↔agent contract. Reuse subcomponents (tree-sitter, an embedding/vector lib, pulldown-cmark, secrecy/keyring). Gut Warp-cloud credential paths (GEAP, Grok OAuth). |
| **warp_cli** | L | One multi-personality binary: parse argv once, route to GUI / hidden workers (terminal-server, plugin/agent host, ripgrep) / scriptable CLI, plus a `warpctrl`-style control CLI (window/tab/pane/session/input/surface/theme/setting verbs) against Marley's session/window model. Built on **clap** + **jaq** (jq filter). SKIP the cloud-auth subcommands (Login/Whoami/--api-key/secret/federate). |
| **warp_completer** | L | Parse command line + cursor into commands/tokens, look up signatures, emit ranked fuzzy suggestions (flags/args/paths/vars/subcommands/aliases); expose `ParsedTokensSnapshot` for the classifier. Ranking on **nucleo** + clap-derived signatures. STUB the v2 JS engine (no rquickjs). Vendor/replace the out-of-repo command-signatures data. 8 internal deps. |
| **warp_search_core** | L | The `SearchMixer` framework: fan a typed `Query` to many registered sync/async data sources, collect scored `QueryResult<T>`, dedupe/order by priority tier + score, track per-source loading/error, render rows; plus the `FilterAtom`/`QueryFilter` taxonomy (history:/workflows:/sessions:/blocks:/# + new marley:/forge:). Backbone of every palette/finder. Reuse **tantivy** (FTS) + **nucleo** (fuzzy) as engines; build the mixer fresh. INVENT integration point (marley:/forge: launcher = one AsyncDataSource). |
| **repo_metadata** | L | A reactive project/file-tree model: detect git roots, build + incrementally maintain a .gitignore-aware tree, watch the FS, expose as a Marley reactive model the file/project panel + completions + AI-context query. Reuse **ignore** (gitignore walk) + **notify**/**notify-debouncer-full**; build the incremental diffs/standing-queries/CanonicalizedPath glue fresh. Drop the remote/SSH snapshot half for first ship. |
| **warp_files** | L | One async `FileModel` that opens/reads/saves/watches files and broadcasts FileLoaded/Saved/Updated with monotonic version tracking, plus a byte-budgeted line-range text reader. Built on Marley's model framework. Reuse leaves (**notify-debouncer-full**, **async-fs**, **async-channel**, **futures**). Trim `FileBackend::Remote` until remote editing is needed. |
| **vim** | L | Keystroke-driven modal FSA (Normal/Insert/Visual/Replace): parse counts/operators/registers/text-objects + dot-repeat, emit semantic `VimEvents` decoupled from buffer mutation via a `VimHandler` trait; generic word/paragraph/find-char/bracket traversal over a `TextBuffer` abstraction. ~2000+ lines. No permissive decoupled vim engine exists (Zed=GPL). **Deferrable past v1** (opt-in). |
| **persistence** | M-L | A local relational durable-state layer restoring windows/tabs/panes/blocks/workflows/notebooks/commands/projects/MCP installs/agent conversations+tasks across restarts, via embedded migrations + typed row models. Reuse the ORM (**diesel** + diesel_migrations, or sqlx/rusqlite) but author OUR OWN schema for Marley's model. DROP/SKIP the account/cloud tables (teams, user_profiles, server_experiments, object_metadata). |

### Complexity M (10)

| Crate | Cx | Behavior to mimic / reuse underneath |
|---|---|---|
| **ui_components** | M | Stateful `Component/Params/Options` widget contract (button, switch, dialog, tooltip, lightbox, shortcut chip) with appearance-driven defaults; persistent component structs reused across frames so hover/tooltip state stays correct. Build on **gpui**, optionally bootstrap from the permissive **gpui-component** (longbridge, MIT/Apache). Constraint: components are long-lived View fields, never recreated per render. |
| **syntax_tree** | M | Hold a live tree-sitter `Tree` per buffer, translate buffer deltas → `InputEdit` for incremental re-parse, run highlight+indent `.scm` queries → `RangeMap<offset,color>` + indent deltas, with a parse-size cap + highlight cache. Reuse **tree-sitter** (MIT) + **tree-sitter-highlight**; build the incremental glue (cache invalidation by version, MAX_PARSE_BYTES guards) fresh. Target upstream tree-sitter, not Warp's vendored `arborium`. |
| **markdown_parser** | M | Parse markdown into a flat, line-oriented, diffable `FormattedText` model (Heading/Line/List/CodeBlock/TaskList/Table/Image/Embedded-YAML + inline fragments + action-or-url Hyperlinks), plus `compute_formatted_text_delta(old,new)` for incremental re-render of streaming agent output. Reuse **pulldown-cmark** (CommonMark/GFM) + **html5ever**; build only the FormattedText struct + line-diff + hyperlink-action dispatch fresh. |
| **local_control** | M | Out-of-process control protocol: request/response envelopes, a macro-generated action catalog (window/tab/pane/session list+create+activate+close, input.insert/replace), `TargetSelectors`, discovery records. The agentic seam (agents driving the cockpit). Reuse the transport (**jsonrpc** crate or **rmcp**); build the semantics fresh. Two deltas: add output read-back (Warp has none), run over a permissive transport so it doubles as an agent tool surface. Stub the auth gating. |
| **cloud_object_models** | M | Fresh plain-Rust structs for the families Marley ships (Workflow + Argument saved-commands, MCPServer records, EnvVarCollection, Notebook), each with a small persistence adapter. Mimic the data shapes, NOT the GenericCloudObject/TryFromGql cloud-sync machinery. MCP records lean on **rmcp** transport types. SKIP the cloud-only families (scheduled_ambient_agent, cloud_environment, ai_execution_profile). |
| **settings** | M | Each feature crate declares typed pref structs/enums via `define_settings_group!`/`define_setting!` + a `SettingsManager` registry, getting free TOML persistence, JSON-Schema gen, per-platform gating, hot-reload, change events. Build on **serde** + **toml/toml_edit** + **schemars** + **notify** + **inventory**. DROP cloud-sync routing (SyncToCloud/Warp Drive) — treat as Never. |
| **input_classifier** | M | The smart-prompt decision engine: `InputClassifier` trait, `InputType` (Shell/AI), `Context`, `HeuristicClassifier` weighing `natural_language_words_score` against token thresholds (0.6, low-token 0.8) + allowlists. Decide Shell vs AI and flip the unified prompt; fall back heuristic→keep-current-mode. STUB the ONNX/BERT-tiny path for first boot (later REUSE candle/ort + tokenizers). Revisit the claude/codex/gemini force-shell allowlist for Marley's UX. |
| **node_runtime** | M | Download pinned Node, extract (tar.gz/zip) to the app data dir, validate, else fall back to recent-enough system node; plus an npm-registry version query. Reuse **reqwest** + **flate2/tar** + **zip** + **semver**; build the validate/wipe/download/extract/fallback orchestration fresh. **Conditional** — only if Marley ships npm-distributed LSPs; feature-gate (`local_fs`), else drops to SKIP with lsp. |
| **warp_util** → **marley_util** | M | Foundational utility crate: shared value types (`FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`) + `standardize_path`. 16 dependents — the workspace's value-type vocabulary, re-declared in clean code. **M0 ships ONLY the value-type subset** (specced in `docs/specs/SPEC-marley-util.spec.md`, seam-contracts §8); reuse **typed-path** + **dirs** + **serde**. The rest — path-massaging extras, **async git invocation, sync primitives, worktree-name generation** (and the deferred reuses **content_inspector** + **mime_guess**, **event-listener**, **rand**) — is **DEFERRED to M1+**, re-scoped out until a concrete consumer needs it (worktree-name → M3 agent-orchestration; async-git → first git-shelling crate). Do the value types first (downstream blocks on them). `make_absolute_url` / the Warp-CDN helper is **dropped**, not stubbed — seam-contracts §10.2 removes the remote-CDN macro path, so `marley_assets` no longer dangles it here. |
| **warp_features** | S/M | Clean-room feature-flag registry: a `FeatureFlag` enum + compile-time-sized lock-free atomic state arrays; resolution order thread-local override → user preference (tri-state) → global state → false; per-channel `*_FLAGS` const sets; a test RAII override guard. Only dep is **enum-iterator**. Keep the name `marley_features`. Add MarleyPanel/session flags; force cloud/team flags fixed; retire FetchChannelVersionsFromWarpServer. |

### Complexity S (9)

| Crate | Cx | Behavior to mimic / reuse underneath |
|---|---|---|
| **string-offset** | S | Two newtypes `CharOffset`/`ByteOffset` that can't be silently mixed, a macro generating `zero()`/`as_usize()`/`add_signed()`/`range()` + Add/Sub, and an incremental `CharCounter` byte→char converter over `&str`. ~150 lines, 12-dependent hub. Keep serde derives; drop get-size unless adding memory accounting. |
| **languages** | S | A registry mapping language name + filename/ext → a loaded `Language` bundle (grammar + highlight/indent/symbol queries + indent unit + comment prefix + bracket pairs), lazily memoized in an Arc cache, grammars + `.scm` embedded at build time. The loader is thin glue; the CONTENT (grammars + `.scm` queries) is REUSE from upstream **tree-sitter-\<lang\>** crates + Helix/Zed query sets via **rust-embed**. Pick our own SUPPORTED_LANGUAGES for binary size. |
| **command** | S | A thin process-spawn wrapper. On macOS/Linux a transparent passthrough to `std::process::Command` + tokio/async-process (effectively a re-export). Real code is Windows polish: `CREATE_NO_WINDOW` + kill-on-parent JobObject, via permissive **win32job** + **windows** crates. Keep the seam so Windows drops in later. |
| **natural_language_detection** | S | Port the lexical AI-vs-shell scorer: `natural_language_words_score` (NL tokens minus shell-syntax tokens, clamped ≥0), `is_word` against three baked dictionaries, contraction expansion, `check_if_token_has_shell_syntax`. Reuse **rust-stemmers** + **regex**; re-generate the embedded word lists. The baseline signal kept even if ONNX is stubbed. |
| **cloud_objects** | S | A minimal fresh identity/type vocabulary: a Uuid object id, an `ObjectType` enum (Workflow/Notebook/Folder/EnvVars/McpServer), a light metadata struct; mimic stable hashed-id strings (`HashableId`) for SQLite keys. Drop the cloud-sharing/GraphQL/UI-coupled pieces (the warpui_core Element types baked into the "headless" layer are a smell). |
| **settings_value** | S | The single `SettingsValue` trait (`to_file_value`/`from_file_value`/`file_schema`, defaulted to serde passthrough) + primitive/blanket/recursive impls (Vec/Option/HashMap/HashSet) + int-seconds Duration + snake_case-enum conventions, so the human-edited TOML is friendlier than raw serde. Built on **serde** + **serde_json** + **schemars**. Own tiny crate to avoid a dep cycle with settings. |
| **settings_value_derive** | S | `#[proc_macro_derive(SettingsValue, attributes(serde))]`: enums → snake_case string / single-key object / array; structs → JSON object with recursive `to_file_value`; honor serde rename_all/rename/skip/default + cfg. Reuse **syn v2** + **quote** + **proc-macro2** + **convert_case**. Optional: collapses into serde+schemars derives if we don't diverge from serde's snake_case. |
| **asset_cache** | S | Extend the UI runtime's `AssetCache` with async sources: fetch URL bytes (**reqwest**), optionally persist to a hashed file, decode `data:[mediatype];base64,…` URIs with a 16 MiB pre-decode guard. Reuse reqwest + **base64**/**data-url**. Keep the oversize-rejection posture + hex-not-base64 cache filename. Enables remote thumbnails/avatars/inline images in a panel. |
| **asset_macro** | S | A thin proc-macro resolving a bundled-asset path to our `AssetSource`, compile-erroring if the file is missing under `app/assets`. SKIP the WASM/Warp-CDN branches (`remote_asset!`/`bundled_or_fetched_asset!`, SHA-256-into-CDN-URL). What remains is an optional compile-time path-check over **rust-embed** — could collapse to nothing if we call `rust-embed.get()` directly. |

---

## SKIP (25) — Warp cloud / web-shell / dev-only, deleted

| Crate | Reason |
|---|---|
| **warp_core** | Despite 33 dependents, its load-bearing surface is Warp's cloud/branding nucleus (ChannelState GraphQL URLs, Oz/Iap auth, RudderStack/Sentry/Firebase telemetry, Channel branding, warp_home paths). The rest (SessionId u64, paths, flags) is trivial infra the Ignibyte `marley_core` recreates clean. Gravity is a fork-migration cost, not a reuse argument. |
| **warp_tui** | Zero-dependent ratatui launcher building a production ChannelState and calling `run_tui()`. Marley is a UI-first gpui cockpit (terminal = one pane); no ratatui product. If a headless mode is ever wanted, **ratatui** (MIT) is off the shelf — not on the clean path. |
| **command-signatures-v2** | A six-line rust-embed shim baking a TypeScript completions plugin into the binary for Warp's JS plugin host. Marley adopts no JS plugin host, so there's nothing to host. Its build.rs needs Node18+corepack+yarn4-PnP at compile time — exactly what a clean Rust build avoids. Already gated off. |
| **warp_multi_agent_client** | Points straight at Warp's hosted multi-agent backend; requires a Warp token; decodes base64-of-protobuf-over-SSE from an out-of-repo proprietary proto. Pure Warp cloud. Replace with a direct provider call in INVENT (Anthropic Messages streaming via reqwest, + OpenAI/local adapters) behind a Rust streaming interface yielding the same event shape `ai` consumes. |
| **warp_server_auth** | Client-side Warp-account auth state (logged-in User, Firebase/API-key/Bearer creds, WARP_USER_SECRET backdoor, AuthStateProvider singleton). Marley is local-first with no Warp login. A future "current user" is a small INVENT-layer local profile, not Firebase/IAP machinery. Doc's KEEP+STUB only reflects fork-cascade, which doesn't exist clean-room. |
| **warp_server_client** | The action layer performing network login, OAuth2 device-code, Firebase token exchange, Google IAP proxy, API-key CRUD, settings sync, Warp Drive access (13 cloud deps). Every behavior is Warp-cloud-specific. Any authed networking Marley needs (Forge/Ignibyte) is INVENT-layer reqwest, not this. |
| **firebase** | Tiny leaf of serde models for Google identitytoolkit REST responses + a WorkOS-SSO heuristic. Exists solely to decode Firebase login JSON. Marley does no Firebase auth; never constructed. Doc flags STUB/candidate-REMOVE. Drop entirely. |
| **warp_graphql** | Typed cynic client bound to Warp's `warp-server` schema (37 queries / 71 mutations, IAP-challenge + Cloud-Armor staging baked in, /graphql/v2). Every op targets Warp's backend. If Marley ever needs GraphQL, REUSE **cynic** + **graphql-ws-client** off the shelf — not these ops. |
| **warp_graphql_schema** | Nothing but the checked-in ~4,442-line `api/schema.graphql` for Warp's backend, registered with cynic as "warp-server." The contract with a server Marley never talks to. No behavior to reimplement. Marley writes its own SDL + **cynic-codegen** if needed. |
| **cloud_object_client** | The `ObjectClient` trait — the seam to Warp's servers (GraphQL/Firebase/RTC ws), ~35 async cloud-object methods + live sync + sharing/guests. Exactly "Warp's cloud." Marley's saved-object surface is a fresh local store (cloud_object_models/persistence) + INVENT. Drop rather than port a no-op stub. |
| **onboarding** | The "Welcome to Warp" multi-slide first-run + skip-login/account-gating dialog (OnboardingAuthState LoggedOut/FreeUser/PayingUser). Pure Warp-cloud-auth UX + the most-branded asset surface in the batch. Marley invents its own first-run in the INVENT layer. (Note: its `bin/main.rs` standalone gpui app is a useful reference template only.) |
| **warp_managed_secrets** | Pure client of Warp's GraphQL secrets backend: HPKE envelope-seal against a server keyset, CRUD over GraphQL, short-lived task identity tokens, GCP Workload Identity Federation. Meaningless without a Warp backend. A future Marley vault is an INVENT decision (OS keychain / age), not a port. |
| **managed_secrets_wasm** | A cdylib wasm-bindgen shim exposing the above envelope encryption to Warp's hosted dashboard. Marley runs no Warp web dashboard and has no managed-secrets cloud. Zero dependents, leaf. Nothing to build. |
| **warp_web_event_bus** | AGPL WASM-only shim (`#![cfg(target_family="wasm")]`) pushing 5 lifecycle events Rust→JS to Warp's out-of-repo TS shell. Marley is native; this compiles to nothing and is needed by nothing. If a web build is ever revived it's a ~30-line REIMPLEMENT, never a port. |
| **remote_server** | Client of Warp's SSH remote-dev: ~3k-line gpui-coupled `RemoteServerManager` + prost protocol talking to a prebuilt daemon tarball that lives OUT OF REPO. Orthogonal to every MVP goal and to the INVENT ops story (Forge owns remote/infra). Future clean REIMPLEMENT (L) on **russh** + our own daemon if remote-dev becomes a requirement. |
| **serve-wasm** | Standalone dev-tooling binary statically serving the Warp-on-Web wasm bundle on 127.0.0.1:8000. 0 deps, 0 dependents; serves a browser build the native clean build doesn't produce. If web dev is ever needed, rebuild on **axum** + **tower-http** (~60 lines, S). |
| **warp_isolation_platform** | Detects Warp's hosted sandbox/Docker/K8s/Namespace env and issues a workload-identity JWT to auth a headless agent to Warp's *backend* (audience = a Warp URL). All 3 consumers are cloud-auth paths the clean build replaces. No login, no backend, no nsc CLI. |
| **channel_versions** | Warp's dev/preview/stable update-channel metadata fetched from a warp.dev/GCP endpoint; drives autoupdate + soft-cutoff banners. Marley is offline/self-built with no update ladder and no Warp round-trip. Short-circuit callers to a single static current-build VersionInfo. |
| **warp_channel_config** | Embed-vs-generate loader that either `include_str!`s per-channel config or shells out to Warp's out-of-repo `warp-channel-config` generator on PATH (panics on a clean checkout). No internal generator, no multi-channel pipeline. Keep only the `ChannelConfig` type (in marley_core) with one hardcoded single-channel config. |
| **app-installation-detection** | A loopback axum router exposing `GET /install_detection` with CORS hardcoded to warp.dev/*.warp.dev so the marketing site can detect an install. Literal de-Warp grep target. Marley has no such website. Delete; stub `make_router()` to empty if the call site is disruptive. |
| **field_mask** | Pure protobuf `FieldMask` merge logic whose only consumer is `warp` applying partial-update deltas to cloud/account/team-sync objects. The clean build SKIPs that seam, so the single callsite disappears. (A future merge is a few lines over `prost-reflect` if ever needed.) |
| **warp_js** | The typed Rust↔JS marshalling/registry for Warp's QuickJS plugin runtime (**rquickjs**, a C engine; thread-local persisted values; cross-process bincode). Marley's extensibility is the INVENT layer (MCP tools, skills, Claude CLI, Forge), which replaces a JS plugin host. SKIP the costly rquickjs dep. If a sandboxed user-script runtime is ever wanted, REUSE rquickjs directly then. |
| **voice_input** | Capture-only mic→base64-WAV whose sole consumer is Wispr, a hosted authenticated transcription service. Touches no core goal; with auth/cloud stubbed there's no endpoint, so a live button would be dead. If voice is ever wanted: a small REUSE-based reimplement (**cpal** + **rubato** + **hound**), not a port. |
| **ipynb_parser** | Render-only Jupyter (.ipynb) → editor `FormattedText` viewer. Self-contained, offline — but notebook viewing is not a clean-build goal and it's tightly bound to Warp's editor/markdown internals (also AGPL). Defer; if a preview pane is wanted later it's a ~400-line S reimplement against our own rich-text model. |
| **integration** | The `warp-integration-test` runner: a Builder + a catalogue of Warp-specific E2E scenarios booting the real `warp` app, with heavy test-only deps. Not linked into the product (Used-by = 0). Marley authors its own headless tests. (The IANA test-net black-hole pattern is a useful one-line idea, not a reason to carry the crate.) |

---

## REUSE shortlist — the foundation crates to pull in

Deduped across all REUSE verdicts and the REUSE-underneath notes inside REIMPLEMENT crates. These are the
permissive dependencies the clean build integrates; none is reimplemented.

| Capability | Crate(s) | License |
|---|---|---|
| **GPU UI framework + platform/render backends** | **gpui** (Zed) — wraps winit, wgpu, blade/Metal, cosmic-text, font-kit/fontdb | Apache-2.0 |
| Widget set seed (optional) | gpui-component (longbridge) | MIT/Apache |
| **Terminal grid + ANSI** | **alacritty_terminal** + **vte** | Apache / MIT |
| **Rope / B-tree** | **ropey** or **sum_tree** (standalone, Zed) | MIT / Apache |
| Text diff | imara-diff | Apache-2.0 |
| Find/regex | regex-automata, regex | MIT/Apache |
| **Markdown / GFM** | **pulldown-cmark** + html5ever (HTML path) | MIT / MIT-Apache |
| **Fuzzy match** | **nucleo** (+ globset for glob) | MIT |
| **Tree-sitter** | **tree-sitter** + tree-sitter-highlight + tree-sitter-\<lang\> grammars + Helix/Zed query sets | MIT |
| **Full-text search index** | **tantivy** | MIT/Apache |
| **Code search via ripgrep lib** | grep / grep-regex / grep-searcher + **ignore** | MIT / Unlicense |
| Gitignore-aware walk | ignore (BurntSushi) | MIT/Unlicense |
| **HTTP + SSE** | **reqwest** + reqwest-eventsource / eventsource-stream | MIT/Apache |
| WebSocket | tokio-tungstenite (+ tokio-rustls, rustls-platform-verifier); ws_stream_wasm (wasm) | MIT/Apache |
| Loopback server | **axum** (already in stack) + tower-http | MIT |
| **MCP SDK** | **rmcp** (official) | MIT/Apache |
| **JSON-RPC / LSP** | **async-lsp** + **lsp-types**; jsonrpsee (generic); lsp-server | MIT/Apache |
| Typed local IPC | tarpc over interprocess | MIT/Apache |
| **SQLite + ORM** | **diesel** (+ diesel_migrations) or rusqlite / sqlx (+ chrono) | MIT/Apache |
| Secret store | **keyring** | MIT/Apache |
| Settings persistence/schema | serde, toml/toml_edit, schemars, inventory; directories / `config` | MIT/Apache |
| **FS watching** | **notify** + **notify-debouncer-full** | MIT/Apache |
| Asset embedding | **rust-embed** | MIT |
| Async asset fetch/decode | reqwest + base64 / data-url | MIT/Apache |
| Logging | **tracing** + tracing-subscriber + tracing-appender + log-panics | MIT/Apache |
| Process spawn (Windows polish) | win32job + windows; async-process / tokio | MIT/Apache |
| Sleep prevention | **keepawake** | MIT/Apache |
| OS automation (computer-use) | **enigo** + **xcap**/screenshots-rs + image | MIT/Apache |
| Node bootstrap (conditional) | reqwest + flate2/tar + zip + semver | MIT/Apache |
| CLI + jq | **clap** + **jaq** | Apache/MIT |
| Proc-macro toolchain | syn v2 + quote + proc-macro2 + convert_case | MIT/Apache |
| Path/text utils | typed-path, content_inspector, mime_guess, dirs, event-listener, rand | MIT/Apache |
| Test fixtures | assert_fs (+ tempfile, dunce) | MIT/Apache |
| Template engine | handlebars (real) / minijinja | MIT/Apache |
| Stemming / NL heuristic | rust-stemmers + regex | MIT/Apache |
| Direct LLM provider (INVENT transport) | reqwest streaming / anthropic-sdk; later candle / ort + tokenizers for on-device classify | MIT/Apache |
| Future GraphQL (if Marley backend) | cynic + cynic-codegen + graphql-ws-client | MIT/Apache |

---

## INVENT — the net-new Ignibyte layer (the moat, no Warp crate)

These components have **no Warp crate** to triage — they are the reason Marley exists and carry **zero clean-room
concern**. They are authored fresh and are where product value concentrates.

- **Project model** — open a project = a git repo or local folder; the project owns its agents, sessions, and Forge
  state. Warp has no project concept. (M2)
- **Panel system** — a pane can be a terminal, an embedded Chromium browser (CEF OSR), an agentic-workflow
  visualizer, or a Forge detail page. Net-new pane abstraction over gpui. (M2/M4)
- **Agent orchestration** — the brain delegates tasks down to project-scoped agents, with two first-class isolation
  modes: **git worktree** and **1-agent = 1-folder + full setup** (preferred). Agents eventually run on the web
  (remote runners). (M3/M5)
- **Brain-MCP (local/remote)** — the brain as an in-app MCP server **or** remote, as a deployment flag not a rewrite.
  Lineage: the Forge RLM / Rusty brain. Served over rmcp. (M4)
- **Forge detail panes** — tickets / sprints / RLM / knowledge rendered as pane content from the Forge system;
  the marley:/forge: search filter feeds a launcher AsyncDataSource into `warp_search_core`. (M2 first pane)
- **Ops surfaces** — manage Kubernetes clusters and Acquia (via Acquia CLI) from panes/agents. Ties to
  Lagoon-in-Rust + the Forge Delivery Platform. (M5)
- **Chromium/CEF pane** — embedded browser via CEF off-screen rendering (DRM out of scope) for live preview / agent
  browsing. (M4)
- **Direct model-provider streaming** — the replacement for `warp_multi_agent_client`: an async stream of response
  events from Anthropic/OpenAI/local providers behind one Rust interface (small S/M, lives in INVENT). (M3)
- **marley_core** — owns SessionId (u64 newtype), `~/.marley` paths, single-channel ChannelConfig, and the
  feature-flag seam — the clean replacement for `warp_core`'s non-cloud substrate.

---

## Build order — REIMPLEMENT + INVENT mapped onto M0–M5

| Milestone | REIMPLEMENT work | INVENT work |
|---|---|---|
| **M0 — Foundation spike** (gpui window + alacritty PTY + one Block) | marley_core (SessionId/paths/flags) · `marley_util` value types (`FileId`/`ContentVersion`/`HostId`/`StandardizedPath`/`LocalOrRemotePath`/`standardize_path` — async-git/sync/worktree-name DEFERRED) · `string-offset` · `command` spawn seam · `warp_features` | — (validate the REUSE stack: gpui + alacritty_terminal) |
| **M1 — Terminal MVP** (Blocks, input editor, panes, palette, themes) | `warp_editor` (L) · `syntax_tree` · `languages` · `markdown_parser` · `settings` + `settings_value` + `settings_value_derive` · `ui_components` · `warp_search_core` (palette) · `warp_completer` · `input_classifier` + `natural_language_detection` · `warp` app shell (root_view/pane_group/palette/themes) · `asset_cache` + `asset_macro` · `vim` (deferred, opt-in) | — |
| **M2 — Project + first Forge pane** | `repo_metadata` · `warp_files` · `persistence` · `cloud_object_models` + `cloud_objects` (Workflow/MCP/EnvVar local) · `local_control` | **Project model** · **Panel system** · **first Forge detail pane** + marley:/forge: search source |
| **M3 — Agent orchestration** | `ai` action model (L) · `warp_cli` control verbs | **Agent orchestration** (worktree + folder-per-agent) · **direct provider streaming** · brain→agent delegation |
| **M4 — Brain-MCP + Chromium pane** | (MCP via REUSE rmcp; node_runtime only if npm LSPs shipped) | **Brain-MCP (local/remote flag)** · **Chromium/CEF pane** |
| **M5 — Ops + remote** | (remote-dev = future clean REIMPLEMENT on russh, not a port) | **k8s + Acquia ops panes/agents** · **agents-on-web remote runners** · brain optionally remote |

---

## Discipline note (the one rule that fixes the license)

- **REUSE is unrestricted** — pull the permissive foundation crates in freely (with attribution); never rewrite a
  renderer, B-tree, terminal grid, LSP/MCP transport, fuzzy matcher, or parser we can adopt.
- **REIMPLEMENT must be written from our spec docs** — the [architecture](../architecture/) + [crates](../crates/)
  references and observed behavior — **not** by reading or translating the AGPL source line-by-line. A reworded
  translation is still a derivative work. The fork + Zap are guides to understand behavior, never the code we ship.
- **SKIP is deleted** — Warp's cloud/account/telemetry/web-shell crates are not ported, not stubbed-for-cascade, not
  carried. They simply do not exist in the clean build; the Ignibyte INVENT layer supplies anything real that's needed.
- Get IP-counsel sign-off before commercializing (see [licensing-ownership-strategy.md](licensing-ownership-strategy.md)).
