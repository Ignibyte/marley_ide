# 00 — Marley System Architecture Overview

> Part of the Marley architecture docs (round 1). Marley is forked from Warp (warpdotdev/warp).

This is the tie-it-together document. The per-subsystem docs (`01`–`07`) survey
the parts; this one shows how they compose into one running application, traces a
keystroke from the keyboard to the GPU, locates the agent/AI and cloud/auth seams,
draws the MIT-vs-AGPL license line, and — most importantly for the fork — names
**the three Marley hook points** with the exact crates/files each will touch.

Read this first, then drill into the subsystem doc cited in each section.

**Cross-links:**
[01 UI/Rendering](01-ui-framework-rendering.md) ·
[02 Editor/Text](02-editor-and-text.md) ·
[03 Terminal/Session](03-terminal-session-core.md) ·
[04 Agent/AI/MCP](04-agent-ai-mcp.md) ·
[05 Cloud/Auth/Net](05-cloud-auth-networking.md) ·
[06 Platform/Settings/Infra](06-platform-settings-infra.md) ·
[07 App Entry/Build](07-app-entry-build-tooling.md) ·
[De-Auth dossier](../plans/de-auth-and-login-stub.md) ·
[Rebrand plan](../plans/rebrand-warp-to-marley.md) ·
[Seed map](_seed-initial-map.md)

---

> **Round 3 re-review · 2026-07-12 · Marley @ M15.** Sections 1–9 below are the **original Jun 27 map**, written
> when Marley was a *research target* with three planned "hook points" bolted onto Warp's own source
> (`app/src/…`, `warp_core`, …). They remain an accurate description of **Warp**, but Marley has since
> reimplemented the terminal + cockpit in its own 15 crates and built an editor Warp never had — so as a
> description of **Marley** they are stale. Read this block first; treat 1–9 as the Warp reference they are.
> Provenance tags below mirror the [Zed deconstruction](../../zed_architecture/README.md):
> `[Warp-derived/AGPL]` (behaviorally reimplemented from an AGPL work — the exposure the fork carries),
> `[permissive: <dep>]` (an adopted permissive crate), `[Marley-original]` (net-new, clean of Warp code).

## Marley status @ M15

Marley is no longer three hooks on Warp's tree; it is its own app on `gpui` `[permissive: gpui Apache-2.0]`
with 15 built crates (see [crate-map](../../marley_architecture/crate-map.md)). Scorecard against Warp's seven
subsystems:

| Warp subsystem | Marley @ M15 | Verdict | Provenance |
|---|---|---|---|
| **03 Terminal / Blocks** | `terminal_blocks` — per-command Blocks over the alacritty grid, shell DCS/OSC hooks, block status, hover-reveal actions, scrollbar / jump-to-bottom, ghost text; **plus** clickable `file:line:col` (#196) + OSC 8 hyperlinks (#214) | **MATCHED (deep)** | `[Warp-derived/AGPL]` behavior · `[permissive: alacritty_terminal MIT]` grid |
| **01 UI framework + 07 app / cockpit** | own docks + `PaneGroup` split/close, command palette, keymap chords, status bar, 5 cockpit widgets (`ui_components`); **M13** multi-workspace + PhpStorm-style launcher, collapsible rail + focused-workspace highlight, workspace-scoped top bar | **MATCHED; launcher EXCEEDS** (Warp has no multi-workspace launcher) | `[Marley-original]` on `[permissive: gpui Apache-2.0]` — Marley does **not** use Warp's MIT `warpui` |
| **02 Editor / text** | **the M15 editable editor** — `marley_editor` Buffer + caret, type / save (⌘S + dirty ●) / undo (coalesced ⌘Z) / select (shift + drag) / copy-cut-paste / full caret-motion, files as editor tabs, split-to-file, persisted split panes (#249–258) | **EXCEEDS — Warp has NO file editor** (its SumTree editor only backs the command input) | `[Marley-original]`; the editor reference is **[Zed/GPL](../../zed_architecture/subsystems/00-overview.md), not Warp** |
| **06 Platform / settings / infra** | `marley_settings` typed-TOML framework; session/layout + open-file + collapsed-rail persistence (#205 / #243 / #245 / #258) | **MATCHED** (telemetry / Sentry deliberately dropped) | `[Marley-original]` · `[permissive: serde / toml]` |
| **04 Agent / AI / MCP** | `marley_agent` cockpit-observability slice + `marley_forge_client`; the **direct-provider-streaming brain** is M3+ (the *sellable* layer — replaces `warp_multi_agent_client` + `app.warp.dev`) | **NOT YET BUILT — by design** (the moat) | `[Marley-original]` — must stay clean of Warp code (see overlay) |
| **05 Cloud / auth / networking** | none — no Firebase / GraphQL / websocket; local-first, **no login wall ever existed** | **N/A** — the whole §5 seam + §7.3 de-auth hook describe a problem Marley doesn't have | `[Marley-original]` local profile seam replaces `warp_server_auth` |

Net: **terminal / cockpit = parity+, editor = Marley's structural win, brain / cloud = deliberately absent**
(the first is the moat, the second was never wanted).

## Provenance & licensing overlay

The [Zed overview](../../zed_architecture/subsystems/00-overview.md) states this posture for the editor tier;
what follows is its Warp/**AGPL** twin — and AGPL is the *stricter* of the two, so the discipline matters more here.

- **The exposure (why the terminal/cockpit is `[Warp-derived/AGPL]`).** The repo header says *"Marley is forked
  from Warp (warpdotdev/warp)"* and Warp is **AGPL-3.0**. Even though Marley's terminal/cockpit is built
  clean-room from **observed behavior** (§20 `observed/`), reusing only permissive crates — never Warp's source —
  the conservative legal read is that a close behavioral reimplementation of an AGPL work carries AGPL exposure.
  So the **intended posture (chad)** is not to fight it: **AGPL the terminal + cockpit** (open-core, exactly as
  Warp itself is), keep the Warp attribution, and **sell the proprietary brain**. Legal review is the release gate.
- **The AGPL §13 network-use trigger — the hazard GPL does NOT have.** GPL copyleft triggers only on
  *distribution* (conveying a binary). **AGPL §13 ("Remote Network Interaction") additionally triggers on
  *network interaction*:** if a user interacts with a *modified* AGPL program **remotely over a network**, they
  can demand its Corresponding Source. **This is the live danger for the brain.** If the sold brain is a network
  service that *incorporates or is built from* AGPL-derived terminal/cockpit code, §13 attaches copyleft to the
  brain **even though you never ship a binary** — and the open-core model collapses. (Zed is GPL, so its editor
  tier has no equivalent network trigger; the Warp-derived side is where §13 bites.)
- **The boundary that keeps the brain sellable.** The brain must be a **separate program** —
  `[Marley-original]`, containing **no** Warp-derived code — that talks to the AGPL terminal/editor only across an
  **arm's-length protocol boundary** (separate process / IPC / the `EditOrigin::Agent` write seam /
  `session.read`-`write` / MCP), never by linking AGPL code into the brain process. Mere aggregation of two
  separate programs over a wire is not a derivative work; **linking is.** The `EditOrigin{Human,Agent}` seam
  already in `marley_editor` (`editor/types.rs`) is exactly that seam — the same one the Zed overlay names for its
  GPL editor. Keep the brain on its own side of it and §13 does not reach it. **This is an architecture note for
  the lawyer review, not legal advice.**

## Round-3 staleness + gaps for the other subsystem agents

- **Biggest staleness (whole doc): the "three hook points" frame (§7) is obsolete.** All three assumed editing
  *Warp's* tree: Hook 1 (add a panel to Warp's `initialize_app`) → Marley built its own cockpit; Hook 2
  (`app/src/terminal/…` spawn / write / read) → Marley reimplemented on `alacritty_terminal`; **Hook 3 (de-auth)
  is fully moot** — Marley is local-first and never had a login wall. §5 (cloud/auth seam) and §7.4 (rebrand of
  `warp*` crates) likewise describe Warp, not Marley.
- **No provenance tags anywhere.** The Round-3 ask is to tag each subsystem doc the way the Zed docs do
  (`[Warp-derived/AGPL]` / `[permissive: <dep>]` / `[Marley-original]`). Each per-subsystem agent (01–07) should
  add a provenance note and re-cast the §6 "MIT vs AGPL" analysis in **Marley** terms: permissive-adopted
  (gpui / alacritty / ropey / nucleo / tree-sitter) vs the intended-AGPL terminal / cockpit vs the clean brain.
- **Per-agent flags:**
  - **01 (UI):** the "`warpui` / `warpui_core` are MIT and reusable" thesis no longer transfers — Marley builds
    on **`gpui` (Apache-2.0)**, not `warpui`. Re-anchor the license lever on gpui.
  - **02 (editor):** the most stale of all — it says Warp has no file editor and describes only the
    command-input SumTree buffer. Marley's M15 editable editor now fills that gap, and its **live reference is
    the [Zed deconstruction](../../zed_architecture/README.md)**, not this Warp doc. Note the handoff.
  - **04 (agent):** "the brain lives on a proprietary server" is accurate for Warp *and* is the model Marley
    copies (sell the brain) — but Marley's brain is `[Marley-original]` direct-provider-streaming, not a
    `warp-proto-apis` client. **This subsystem is where the §13 network trigger is most acute** — call it out.
  - **05 (cloud/auth):** flag the whole subsystem **N/A for Marley** (no Firebase / GraphQL / login); its value
    now is purely as a Warp reference, not a Marley work surface.

---

## 1. What Marley is, in one paragraph

Marley is a GPU-rendered, native desktop terminal built almost entirely in Rust
(~1.46M LOC, **78 workspace crates** + the top-level `app` crate). The visible
application is `app/` (package name literally `warp`), which compiles to a set of
near-identical per-channel binaries (`warp-oss` is the default/OSS build). It is
built on an **in-house, MIT-licensed GPU UI framework** (`warpui` / `warpui_core`)
that is conceptually a Flutter/GPUI hybrid. On top of that framework sit the
SumTree text editor, an Alacritty-derived terminal/PTY engine that segments output
into per-command **Blocks**, and an **Agent Mode** that is a thin client to a
*proprietary* cloud backend at `app.warp.dev`. Local terminal use needs no account;
Warp Drive, AI/agents, session sharing, teams, and settings sync all do.

---

## 2. Top-level component diagram

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ app/  (crate "warp")  — boot, registration, the on-screen views   [07]   AGPL         │
│                                                                                        │
│   src/bin/{oss,local,dev,stable,preview}.rs ── thin main() ── warp::run()              │
│        └─ run_internal() ─ builds warpui::AppBuilder ─ initialize_app(ctx)             │
│                                          │  (the panel/feature registration list)      │
│   ┌──────────────────── RootView (per window)  app/src/root_view.rs ────────────────┐  │
│   │  AuthOnboardingState gate ──► Workspace (app/src/workspace/view.rs)             │  │
│   │   ┌─────────────┬───────────────────────────────┬──────────────────────────┐   │  │
│   │   │ LeftPanel   │  center PaneGroup             │ RightPanel               │   │  │
│   │   │ (explorer/  │  (terminal panes = Blocks)    │ (Agent Mode / review)    │   │  │
│   │   │  search/    │                               │                          │   │  │
│   │   │  agent      │   ◄── Marley panel goes here ──►                          │   │  │
│   │   │  convos)    │                               │                          │   │  │
│   │   └─────────────┴───────────────────────────────┴──────────────────────────┘   │  │
│   └────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                        │
│   app/src/terminal/  ── PTY engine (spawn/write/read, Blocks, DCS hooks)   [03]        │
│   app/src/ai/        ── Agent Mode orchestration client (ResponseStream)   [04]        │
│   app/src/auth/      ── AuthManager + login UI (policy layer)              [05]        │
└───────────────┬─────────────────────────────┬───────────────────────────┬─────────────┘
                │                             │                            │
   ┌────────────▼───────────┐   ┌─────────────▼────────────┐   ┌───────────▼─────────────┐
   │ UI FRAMEWORK   [01]     │   │ EDITOR / TEXT   [02]      │   │ CLOUD / AUTH SEAM  [05]  │
   │ warpui      (MIT)       │   │ editor (warp_editor)      │   │ warp_server_auth         │
   │ warpui_core (MIT)       │   │ sum_tree, string-offset   │   │ warp_server_client       │
   │ warpui_extras (AGPL*)   │   │ syntax_tree, languages    │   │ graphql, firebase        │
   │ ui_components (AGPL)     │   │ markdown_parser           │   │ cloud_object_* , websocket│
   │  App/View/Element/Scene │   │ CoreEditorModel trait     │   │ remote_server (SSH)      │
   └────────────┬───────────┘   └─────────────┬────────────┘   └───────────┬─────────────┘
                │                             │                            │
   ┌────────────▼───────────┐   ┌─────────────▼────────────┐   ┌───────────▼─────────────┐
   │ TERMINAL CORE   [03]    │   │ AGENT / AI / MCP   [04]   │   │ PLATFORM/INFRA   [06]    │
   │ warp_terminal           │   │ ai, warp_multi_agent_     │   │ settings, persistence    │
   │ warp_core (SessionId,   │   │  client (247 LOC seam)    │   │ warp_features, warp_     │
   │  channel config)        │   │ mcp (rmcp, local)         │   │  logging (Sentry)        │
   │ local_control (warpctrl) │   │ computer_use              │   │ warp_assets, channel_*   │
   │ command, ipc            │   │ input_classifier          │   │ vim, voice_input, …      │
   └─────────────────────────┘   └──────────────┬───────────┘   └──────────────────────────┘
                                                │
                                  ════════════════════════════════════
                                  PROPRIETARY (NOT in repo):
                                  app.warp.dev  — LLM + agent orchestration
                                  warpdotdev/warp-proto-apis — wire schema
                                  ════════════════════════════════════
```

`*` `warpui_extras` declares `license.workspace = true` ⇒ **AGPL**, not MIT
(corrected from the seed map; see §6 and the [rebrand plan](../plans/rebrand-warp-to-marley.md)).

---

## 3. The data-flow spine: keystroke → editor → session/PTY → Blocks → UI render

This is the single most important path to understand because Marley's hook points
all attach to it.

```
(1) KEY EVENT
    platform event loop (mac Cocoa / winit)              [01 warpui platform]
      └─► AppContext ─► Presenter::dispatch_event
            └─ root View's Element tree hit-tests, an Element handles the key
               and emits an Action / mutates a Model                       [01]

(2) INTO THE INPUT EDITOR
    key → typed action (e.g. TuiInputAction / BufferEditAction)            [02]
      └─ model.update(ctx, |m| m.user_insert(text, ctx))
           → CoreEditorModel::insert(EditOrigin::UserTyped)                [02]
           → BufferEditAction::Insert on a SumTree<BufferText>             [02]
      EditOrigin = {UserTyped | UserInitiated | SystemEdit}  ◄─ the seam that
      distinguishes human typing from programmatic / agent writes          [02]

(3) SUBMIT → WRITE TO THE SESSION
    on Enter: read buffer back as String (plain_text / text_in_range)      [02]
      └─ host emits Submitted(String)  ──►  PtyController::write_command    [03]
           → PtyWrite queue → Message::Input(bytes)                        [03]
           → mio_channel → EventLoop::pty_write → pty.writer().write()      [03]
           → leader fd → shell process                                     [03]
    Spawn path:  TerminalManager::spawn_pty → Pty::new → PtySpawner         [03]
                 → local_tty::spawn (openpty + fork)  OR  terminal server   [03]

(4) OUTPUT → MODEL (the "read" half)
    shell stdout → leader fd → "PTY reader" thread:                        [03]
      EventLoop::pty_read → pty.reader().read()
        → ansi::Processor::parse_bytes  mutates  Arc<FairMutex<TerminalModel>>
        → shell DCS/OSC hooks (Precmd/Preexec/Bootstrapped/InitShell)      [03]
            attach cwd / exit code / git / SessionId to each Block
        → ChannelEventListener::send_wakeup_event()                        [03]

(5) BLOCKS → UI RENDER
    wakeup invalidates the terminal View                                  [01]
      └─ Presenter::build_scene: View::render → Box<dyn Element>           [01]
           layout (constraint solve) → after_layout → paint               [01]
           paint records Rect/Glyph/Image/Icon into a retained Scene       [01]
      └─ Window::render_scene(Rc<Scene>) → platform Renderer (Metal/wgpu)   [01]
           atlas + glyph cache → GPU draw → present                        [01]
```

Two directions of flow, framework-wide: **state flows DOWN** via `View::render`
reading view/model state; **events flow UP** via `Element::dispatch_event` →
`Action`s → `ViewContext`/`ModelContext` mutations → `notify()` → re-render.
**Async/streaming** results (subprocess output, SSE agent stream) enter through
`ViewContext::spawn*` and fold back into view state. See [01 §"Data/control flow"].

---

## 4. Where the agent / AI layer sits

Agent Mode is a **thin client over a proprietary server** — there is no LLM
inference or orchestration logic in the repo (see [04 §0]). The loop:

```
typed input ─► input_classifier (shell vs natural language)               [04]
   NL ─► app/src/ai/agent/api/impl.rs  builds protobuf api::Request
         (advertises client tool capabilities + context)                  [04]
      ─► crates/warp_multi_agent_client (247 LOC — the literal client↔server
         boundary): POST {server_root_url}/ai/multi-agent  (bearer + SSE)  [04]
      ═══ app.warp.dev: LLM + orchestration decides next tool call ═══
      ─► streamed protobuf ResponseEvents
      ─► app/src/ai/blocklist/controller/response_stream.rs :: ResponseStream
         turns each event into typed AIAgentActionType                     [04]
      ─► action executors (run shell cmd → [03] PTY, edit files, CallMCPTool
         → crates/mcp, UseComputer → crates/computer_use, RunAgents → child)
      ─► results fed back as the next Request's input  (loop)              [04]
```

Key facts for Marley:
- The **wire schema is an out-of-repo git dependency** (`warpdotdev/warp-proto-apis`,
  rev `ac1af730`). This repo only has the client adapter. [04 §0]
- The endpoint is hardcoded (`warp_core/src/channel/config.rs:59`,
  `server_root_url = "https://app.warp.dev"`) and `/ai/multi-agent` is appended. [04]
- **MCP** (`crates/mcp`, on `rmcp`) and **computer_use** are *local and
  self-hostable* — the no-server-change path to inject custom tools. [04 §4]
- Every agent step passes through the single typed `ResponseStream` event boundary
  — **the natural insertion point for a workflow-visualization panel.** [04 §2c, §7]

---

## 5. Where the cloud / auth seam sits

There is **no single login wall**; gating is split across two layers ([05 §7],
[De-Auth dossier §1]):

- **Seam layer (transport):** cloud endpoints are compile-time constants in
  `warp_core/src/channel/config.rs`; the OSS channel is hardwired to them and
  *blocks* `--server-root-url` overrides (`Channel::allows_server_url_overrides()`
  = `false` for `Stable|Preview|Oss`). Credentials are modeled in
  `warp_server_auth` (`Credentials` enum, `AuthState`/`AuthStateProvider`
  singleton); `warp_server_client::AuthSession::get_or_refresh_access_token()` is
  the single token choke point (and already `bail!`s under the `skip_login`
  feature). [05 §1–§5]
- **Policy layer (app):** the master predicate `AuthState::is_anonymous_or_logged_out()`
  is checked at ~30 UI sites; `AuthManager` shows the auth modal; and the *actual*
  initial gate is one `if/else` choosing `AuthOnboardingState` in
  `app/src/root_view.rs:1662-1698` — only `AuthOnboardingState::Terminal` renders
  the real workspace. [05 §7b], [De-Auth dossier §1.1]

What works offline vs. needs an account: **local terminal/PTY/Blocks/themes/settings
= no account**; Warp Drive, AI/agents (Oz), session sharing, teams, managed secrets,
cloud settings sync = **account required**. [05 §8]

The recommended de-auth approach (from the dossier) is to adopt **warp-offline's
runtime `network_policy` / `ServicesMode::LocalOnly`** pattern (small, reversible)
rather than OpenWarp/Zap's delete-the-crates approach, and layer an `AuthProvider`
trait over the existing `AuthStateProvider` so an Ignibyte login drops in later
with zero call-site changes. See §7.3 and the [De-Auth dossier §3].

---

## 6. License map (MIT vs AGPL) — the line Marley must respect

| Layer | Crates | License | Why it matters |
|---|---|---|---|
| **GPU UI framework** | `warpui`, `warpui_core` | **MIT** | Permissive, reusable in a closed product. No `warp_core` dep. A new UI surface written *purely* against these can stay MIT. [01 §License boundary] |
| **UI platform extras** | `warpui_extras` | **AGPL** (`license.workspace = true`) | Seed map/README wrongly called this MIT — **corrected**. [01], [rebrand §1.1] |
| **Themeable widgets** | `ui_components` | **AGPL** (depends on `warp_core` for `Appearance`) | The moment UI links `ui_components` (or any `warp_core`), the derived work is AGPL. [01] |
| **Everything else** | `app` (crate `warp`) + the other ~75 crates | **AGPL-3.0-only** | The client. Distributing or network-hosting Marley triggers AGPL §13 source-offer obligations. [07], [De-Auth dossier §4] |
| **Not in repo** | `app.warp.dev` server, "Oz" orchestration, `warp-proto-apis` schema | Proprietary | Marley must replace or re-spec these for BYO-agent. [04 §0] |

**Practical rule:** the `warpui*` MIT crates are a strategic asset. New
framework-level capabilities can stay MIT *only if* they avoid `warp_core`. Any
code that reads terminal/session/appearance state inherits AGPL. The panel→session
bridge (Marley hook #2) necessarily crosses the MIT↔AGPL line. Keep the Warp
attribution per AGPL; see the [rebrand plan §3] for the retained credit block.

---

## 7. The three Marley hook points

These are the actual work surfaces. Each names the exact crates/files it touches.

### 7.1 Hook 1 — UI-surface expansion (the agentic-workflow panel)

**Goal:** a custom Ignibyte panel that visualizes an agentic workflow.

A panel is just a struct implementing `Entity + View` whose `render(&AppContext) ->
Box<dyn Element>` returns an element tree (stock `Flex`/`Stack`/`Table`/`List`/`Text`
+ `ui_components` widgets, or a custom `Element` painting directly via the imperative
`Scene` API for bespoke graph/timeline drawing). Backing state lives in a `Model`
the panel `observe`s. [01 §Marley relevance]

Where it physically lands ([07 §2]):
- **Registration:** add `my_panel::init(ctx)` to the `initialize_app(...)` list in
  `app/src/lib.rs` (≈1782–1840), exactly like `ai_assistant::panel::init(ctx)`.
- **Placement:** model it on `RightPanelView` (`app/src/workspace/view/right_panel.rs`
  — the existing Agent Mode / review side panel, lowest-risk analog), or add a
  `ToolPanelView` variant to `LeftPanelView`, or a new pane kind in
  `crate::pane_group`. Per-window container is `RootView` (`app/src/root_view.rs`).
- **Live data to render:** tap `ResponseStreamEvent::ReceivedEvent` in
  `app/src/ai/blocklist/controller/response_stream.rs` and the `AIAgentActionType` /
  `…ActionResult` enums in `crates/ai/src/agent` (steps, tool calls, child-agent
  dispatch, todos) — *no server cooperation needed to observe*. [04 §7] Per-command
  Block metadata (cwd/exit/SessionId from DCS hooks) is free workflow context. [03 §6]

**Key files:** `app/src/lib.rs` (`initialize_app`), `app/src/workspace/view/right_panel.rs`,
`app/src/root_view.rs`, `crates/warpui_core/src/core/view/mod.rs` (`View` trait),
`crates/warpui_core/src/scene.rs` (custom paint), `app/src/ai/blocklist/controller/response_stream.rs`.

### 7.2 Hook 2 — Session spawn / write / read

**Goal:** spawn terminal sessions from a UI component, write to them, read output back.

This is the cluster in [03] — and note the byte-level engine lives in
**`app/src/terminal/`**, not the `crates/*` the seed map named.

- **Spawn:** `TerminalManager::spawn_pty` (with a custom `PtyOptions`) → `Pty::new`
  → `PtySpawner::spawn_pty` → `local_tty::spawn` (`nix::pty::openpty` + fork) or the
  out-of-process terminal server (SCM_RIGHTS fd passing).
  `app/src/terminal/local_tty/{terminal_manager,unix,spawner}.rs`. [03 §3]
- **Write:** `PtyController::write_bytes` / `write_command` / `write_agent_bytes`, or
  implement the UI-agnostic `PtyIntentEvent` / `TerminalSurface` seam (the cleanest
  place for an Ignibyte panel to inject writes).
  `app/src/terminal/writeable_pty/{pty_controller,terminal_surface,message}.rs`. [03 §4]
- **Read:** observe `TerminalModel` / `BlockList` on `ChannelEventListener` wakeups,
  or tap the `pty_read` byte-copy hook (used by `recorder.rs`).
  `app/src/terminal/local_tty/event_loop.rs`. [03 §5]
- **Input half (from a UI editor):** drive a `CodeEditorModel` with `BufferEditAction`s;
  use `EditOrigin::SystemEdit` for programmatic/agent text vs `UserTyped` for human
  keys; read back with `plain_text` / `Buffer::text_in_range`. Reference impl:
  `crates/warp_tui/src/input/view.rs` (emits `Submitted(String)`). [02 §6, §9]
- **Out-of-process driving (existing):** `local_control` (warpctrl) already creates
  tabs/sessions and injects input text (`tab.create`, `session.*`, `input.insert`) —
  **but has no byte-level read-back**; Marley would add a new `ActionKind`
  (e.g. `session.read`) over `BlockList`, or an IPC tap. `crates/local_control/`. [03 §7]
- **Remote/SSH template:** `remote_server` models host/session-scoped RPC with a
  decoupled `RemoteServerAuthContext` (Bearer credential, no Firebase refresh) — a
  strong template for driving sessions without `app.warp.dev`. [05 §9]

**Key files:** `app/src/terminal/writeable_pty/pty_controller.rs`,
`.../terminal_surface.rs`, `app/src/terminal/local_tty/{terminal_manager,spawner,event_loop,unix}.rs`,
`app/src/terminal/model/{block,blocks,ansi/dcs_hooks}.rs`, `crates/warp_core/src/session_id.rs`,
`crates/local_control/src/{catalog,protocol}.rs`.

### 7.3 Hook 3 — De-auth + login stub

**Goal:** boot with no mandatory Warp login, with a clean seam for Ignibyte's own
future login.

The whole enforcement decision is **one `if/else` in
`app/src/root_view.rs:1662-1698`** choosing the initial `AuthOnboardingState`; only
`Terminal` renders the workspace. It pivots on `auth_state.is_logged_in()` (=
credentials present), `FeatureFlag::ForceLogin`, and
`FeatureFlag::SkipFirebaseAnonymousUser`. [De-Auth dossier §1.1]

Recommended approach (warp-offline pattern + provider seam):
- **Step A (boot straight to terminal):** port `crates/network_policy`
  (`ServicesMode::LocalOnly` + `check_url` transport guard), add
  `services_mode` to `ChannelConfig`, set it `LocalOnly` for Marley's channel in
  `warp_core/src/channel/state.rs`, and add `is_local_only()` to the skip branch of
  the gate (or default-enable `SkipFirebaseAnonymousUser`, which the gate already
  honors). Non-loopback cloud egress is then denied at the transport layer even if a
  call site forgets to check. [De-Auth dossier §3.1]
- **Step B (pluggable login):** add a first-class `Credentials::Local` variant
  (`AuthToken::NoAuth`) and an `AuthProvider` trait backing the existing
  `AuthStateProvider` singleton; ship `LocalUserAuthProvider` by default. The
  RootView gate is then untouched — `is_logged_in()` returns `true` via the local
  provider; registering `IgnibyteAuthProvider` later needs **no call-site changes**.
  [De-Auth dossier §3.2]
- **Single registration choke point:** all of this is reachable from the one
  `auth::init(ctx)` line in `initialize_app` (`app/src/auth/mod.rs:56`). [07 §4]
- **Guard, don't delete:** Warp Drive, shared sessions, settings sync, telemetry must
  be gated by `is_local_only()` (they auto-off when local), not ripped out, to stay
  close to upstream and keep `cargo build` green (the cloud crates are in
  `default-members`). Beware `log_out()` deletes the SQLite DB. [De-Auth dossier §3.3, §4]

**Key files:** `app/src/root_view.rs:1581-1700`,
`crates/warp_server_auth/src/{auth_state,credentials,lib}.rs`,
`crates/warp_server_client/src/auth/session.rs:97-141`, `app/src/auth/{mod,auth_manager}.rs`,
`crates/warp_features/src/lib.rs` (`ForceLogin`, `SkipFirebaseAnonymousUser`),
`crates/warp_core/src/channel/{config,state,mod}.rs`, and (to port) `crates/network_policy/`.

### 7.4 Cross-cutting: rebrand

Not a "hook point" so much as a sweep, but it overlaps all three. The per-binary
`src/bin/*.rs` wrappers carry the user-visible identity (`AppId`, `Info.plist`,
URL schemes, "Denver Technologies" copyright); `WARP_*` env vars are a *contract*
with the bundled shell bootstrap scripts (rename in lockstep); `warp.dev` endpoints
overlap the de-auth work; crate renames (23 `warp*`/`warpui*` packages) are
high-blast-radius and deferred. Keep the Warp credit per AGPL. See the
[rebrand plan](../plans/rebrand-warp-to-marley.md) for the tiered, ordered checklist.

---

## 8. How the hook points interlock

- Hook 1 (panel) **renders** data produced by Hook 2 (sessions) and the agent
  stream; it dispatches `Action`s that Hook 2 turns into PTY writes.
- Hook 2's panel→session bridge **crosses the MIT↔AGPL line** (panel can be MIT until
  it touches `warp_core`/terminal state). Decide this deliberately. [01 §License lever]
- Hook 3 (de-auth) is largely *orthogonal* to Hooks 1–2 (local terminal already works
  logged-out) but shares the `app.warp.dev` / Sentry endpoints with rebrand, and gates
  whether the agent backend (Hook-1 data source) is Warp's server, a Marley BYO
  backend, or disabled. [04 §7, 05 §13]
- BYO-agent (replacing the `app.warp.dev` brain) is the one goal **not** fully
  solvable inside this repo — it needs a server that speaks `warp-proto-apis`, a
  proxy at `override_server_root_url`, or a new backend at the `ResponseStream` seam;
  MCP is the no-server-change extension path. [04 §0, §7]

---

## 9. Subsystem doc index (what to read next)

| Doc | Covers | Primary Marley hook |
|---|---|---|
| [01-ui-framework-rendering](01-ui-framework-rendering.md) | `warpui`/`warpui_core` GPU framework, App/View/Element/Scene, renderers | Hook 1 (UI) |
| [02-editor-and-text](02-editor-and-text.md) | SumTree editor, `CoreEditorModel`, `EditOrigin`, capture→submit | Hook 2 (write half) |
| [03-terminal-session-core](03-terminal-session-core.md) | PTY spawn/write/read, Blocks, DCS hooks, `local_control` | Hook 2 (sessions) |
| [04-agent-ai-mcp](04-agent-ai-mcp.md) | Agent Mode client, proprietary-server seam, MCP, `ResponseStream` | Hook 1 data + BYO-agent |
| [05-cloud-auth-networking](05-cloud-auth-networking.md) | Credentials, channel config, GraphQL/RTC, where login is gated | Hook 3 (de-auth) |
| [06-platform-settings-infra](06-platform-settings-infra.md) | settings, persistence, feature flags, telemetry, assets | Hook 3 + rebrand |
| [07-app-entry-build-tooling](07-app-entry-build-tooling.md) | `app/` boot, `initialize_app` registration, `script/`, build | all hooks (where they land) |
| [../plans/de-auth-and-login-stub](../plans/de-auth-and-login-stub.md) | the executable de-auth/login-stub plan | Hook 3 |
| [../plans/rebrand-warp-to-marley](../plans/rebrand-warp-to-marley.md) | the tiered rebrand plan | rebrand |
| [_seed-initial-map](_seed-initial-map.md) | the original v1 research map | orientation |
