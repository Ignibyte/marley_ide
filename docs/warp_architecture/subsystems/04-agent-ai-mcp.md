# Subsystem 04 — Agent, AI & MCP

> Part of the Marley architecture docs (round 3 re-review, 2026-07-12). Marley is forked from Warp (warpdotdev/warp — **AGPL-3.0**).
> **This is the most commercially-sensitive subsystem in the map: it deconstructs the layer Marley intends to SELL — the agent "brain".** The sold brain must be **clean-room / Marley-original**, built from agent *concepts* (tool-calling, MCP, context assembly), NOT from Warp's AGPL agent source — or AGPL copyleft attaches and it cannot be sold. See the **Provenance & licensing** section below for the (existential) AGPL network-use trigger and the `EditOrigin::Agent` clean seam.

This document maps the crates and `app/` modules that implement **Agent Mode**: how a user
prompt becomes a request to Warp's AI server, how the streamed response is turned into
client-side *actions* (run a command, edit a file, call an MCP tool, spawn a child agent),
and how Model Context Protocol (MCP) servers, computer-use, and input classification plug
into that loop. It is the most important subsystem for Marley's "visualize the agentic
workflow" and "bring-your-own-agent" goals.

---

## 0. The central fact: the AI brain lives on a proprietary server

There is **no LLM inference and no agent orchestration logic in this repository.** The client
builds a protobuf `Request`, POSTs it to a Warp-hosted HTTPS endpoint, and consumes a stream
of protobuf `ResponseEvent`s that *tell the client what to do*. The orchestration loop —
deciding which tool to call next, the system prompt, the model routing — all runs server-side.

Two hard boundaries make this concrete:

1. **The wire schema is an out-of-repo git dependency.** `Cargo.toml:341`:
   ```toml
   warp_multi_agent_api = { git = "https://github.com/warpdotdev/warp-proto-apis.git", rev = "ac1af730..." }
   ```
   Every `api::Request`, `api::ResponseEvent`, `api::message::ToolCall`, `api::ToolType`, etc.
   is generated from `.proto` files in that separate repo. This repo only contains the *client
   adapter* that converts those proto types to/from local Rust enums.

2. **The endpoint is hardcoded to Warp infrastructure.**
   `crates/warp_core/src/channel/config.rs:59` → `server_root_url: "https://app.warp.dev"`.
   The multi-agent client (below) appends `/ai/multi-agent` to it.

**Marley implication:** "bring-your-own-agent" / "own the agentic workflow" cannot be achieved
by editing these crates alone. The agent intelligence is behind `app.warp.dev`. Marley must
either (a) stand up a compatible server speaking the `warp-proto-apis` `ResponseEvent`
protocol, (b) override `server_root_url` to a self-hosted proxy, or (c) introduce a *new*
agent backend at the `ResponseStream` seam in `app/src/ai/agent/api/`. The good news: the
client→server boundary is small, well-isolated, and trait-shaped (see §2).

---

## 1. Crate inventory

| Crate / area | LOC | Role in the agent loop |
|---|---|---|
| `crates/ai` | ~27.7k | Client-side agent *domain model*: `AIAgentActionType` (tool calls), action results, proto⇄local conversion, skills, project context, diff validation, API-key/credential management, LLM ids. |
| `crates/warp_multi_agent_client` | 247 | Thin transport: opens the SSE event stream to `…/ai/multi-agent`, base64+protobuf-decodes `ResponseEvent`s. **The literal client→server boundary.** |
| `crates/mcp` | ~2.2k | MCP client runtime (built on `rmcp`): spawn/connect MCP servers over stdio / streamable-HTTP / SSE, OAuth, list tools+resources. |
| `crates/computer_use` | ~4.3k | OS-level mouse/keyboard/screenshot actor (mac/win/linux-x11/wayland). Backs the `UseComputer` agent tool. |
| `crates/jsonrpc` | 461 | Minimal JSON-RPC 2.0 client (`JsonRpcService` + `Transport`). Used by `lsp`, **not** by Agent Mode's main path. |
| `crates/input_classifier` | ~2.0k | Decides whether typed input is a **shell command vs. natural-language prompt** (ONNX model + heuristic fallback). Gates whether input enters Agent Mode. |
| `crates/natural_language_detection` | 126 | Word-list/stemmer scorer used by the input classifier heuristics. |
| `crates/warp_completer` | ~18.2k | Shell command parser/autocomplete engine. Feeds `ParsedTokensSnapshot` into the input classifier; not an LLM component. |
| `app/src/ai/**` | ~238k (479 files) | The actual agent **orchestration client**: conversation model, request building, response-stream controller, MCP managers, skills, todos, ambient/child agents, computer-use wiring. This is where the loop lives. |

Note the asymmetry: the assigned `crates/*` are mostly *libraries and value types*; the
behavioral heart of Agent Mode is in `app/src/ai/`, which depends on them.

---

## 2. The request/response boundary (proprietary-server seam)

### 2a. Transport — `crates/warp_multi_agent_client/src/lib.rs`

One public function:

```rust
pub async fn generate_multi_agent_output(
    client: &BaseClient,
    request: &warp_multi_agent_api::Request,
) -> Result<OutputStream, Error>
```

- `OutputStream = BoxStream<Result<warp_multi_agent_api::ResponseEvent, Error>>` (`LocalBoxStream` on wasm).
- Builds a POST to `endpoint_url(...)`:
  `{ChannelState::server_root_url()}/ai/multi-agent` (or `/ai/passive-suggestions`, or
  `/agent-mode-evals/...` under the `agent_mode_evals` feature).
- Sends the request body as protobuf (`.proto(request)`), attaches a **bearer token** from
  `client.get_or_refresh_access_token()` plus "ambient headers" (`AmbientHeaderPolicy`).
- Opens an **SSE eventsource**; each `Message` event's `data` is a base64-url string wrapping a
  protobuf `ResponseEvent`, decoded by `decode_response_event`.
- Recognizes two control events for tracing: `response_event::Type::Init` (carries
  `conversation_id`, `request_id`, `run_id`) and `…::Finished`.
- Calls `.prevent_sleep("Agent Mode request in-progress")` to keep the machine awake during a run.

`Error` variants name the boundary's failure modes: `Authentication`, `AmbientHeaders`,
`Base64Decode`, `ProtobufDecode`, `EventSource`.

**This is the de-auth touchpoint #1:** the request is rejected without a valid bearer token
from `BaseClient` (see Subsystem on auth). Stubbing login means this token must still be
producible (or this path must be rerouted to a server that doesn't require it).

### 2b. Request construction — `app/src/ai/agent/api/impl.rs`

The app-level `generate_multi_agent_output(server_api, params: RequestParams, cancellation_rx)`
(`app/src/ai/agent/api/impl.rs:13`) assembles the `api::Request`:

- `convert_input(params.input)` → `api::request::Input` (the user turn / tool results).
- `api::request::Settings { model_config, supported_tools, autonomy_level, isolation_level,
  web_search_enabled, planning_enabled, … }` — a large capability-advertisement struct telling
  the server which client features/tools are available.
- `get_supported_tools(&params)` (`impl.rs`) builds the **client tool capability list**
  (`Vec<api::ToolType>`): `Grep, FileGlob(V2), ReadMcpResource, CallMcpTool, RunShellCommand,
  Subagent, WriteToLongRunningShellCommand, ReadShellCommandOutput, Read/Create/EditDocuments,
  SuggestPrompt`, plus flag-/session-gated `ReadFiles, ApplyFileDiffs, SearchCodebase,
  UseComputer, RequestComputerUse, RunAgents, SendMessageToAgent, WaitForEvents,
  AskUserQuestion, ReadSkill, FetchConversation`. The server only emits tool calls the client
  has advertised here. Local vs. `WarpifiedRemote` sessions advertise different file tools.
- `redaction::redact_inputs` strips secrets when `should_redact_secrets`.
- API keys flow through `ApiKeyManager` with a "Warp credits" fallback toggle.

So the contract is: **client advertises capabilities + sends context; server drives the loop
and emits tool calls; client executes them and sends results back as the next request's input.**

### 2c. Response consumption — `app/src/ai/blocklist/controller/response_stream.rs`

`ResponseStream` (a `warpui` `Entity`) wraps the `OutputStream` and:

- Tracks `StreamInit` (captures `server_output_id` from `request_id`), `ClientActions` (sets
  `has_received_client_actions`), and `Finished` (with `stream_finished::Reason::Done/...`).
- Implements **retry/resume recovery** (`recovery_action`, `MAX_RETRIES = 3`): re-send verbatim
  if no actions received yet; otherwise issue a fresh `ResumeConversation` request; otherwise
  fail. Distinguishes online/offline via `NetworkStatus`.
- Emits `ResponseStreamEvent::ReceivedEvent(...)` to downstream UI/controllers.

This is **the natural insertion point for Marley's workflow visualizer**: every agent step
(message chunk, tool call, tool result, finish reason) passes through this single stream as a
typed event.

---

## 3. Agent action model — `crates/ai/src/agent`

This is the client's typed vocabulary of "things the agent asked the client to do."

- `agent::action::AIAgentActionType` (`crates/ai/src/agent/action/mod.rs`, ~929 LOC) — the big
  enum. Variants map 1:1 to server tool calls: `RequestCommandOutput` (run a shell command,
  with `is_read_only`/`is_risky`/`wait_until_completion`/`uses_pager` flags),
  `WriteToLongRunningShellCommand`, `ReadShellCommandOutput`, `ReadFiles`, `SearchCodebase`,
  `RequestFileEdits`, `Grep`, `FileGlob(V2)`, `ReadMCPResource { server_id, name, uri }`,
  `CallMCPTool { server_id, name, input: serde_json::Value }`, `Read/Edit/CreateDocuments`,
  `UseComputer` / `RequestComputerUse`, `ReadSkill`, `FetchConversation`, `AskUserQuestion`,
  and the orchestration set: `StartAgent`, `SendMessageToAgent`, `RunAgents(RunAgentsRequest)`,
  `WaitForEvents`, `TransferShellCommandControlToUser`.
- `agent::action_result::*` (`action_result/mod.rs`, ~1490 LOC) — the matching *result* types
  the client sends back (`CallMCPToolResult`, `RunAgentsResult`, `UseComputerResult`, …).
- `agent::action::convert` (`action/convert.rs`, ~704 LOC) — `From<api::message::tool_call::*>`
  impls turning proto tool calls into `AIAgentActionType`. `agent::convert`
  (`convert.rs`) defines the error types `ConvertToAPITypeError` and `ToolToAIAgentActionError`.
- `agent::orchestration_config` — config for multi-agent runs.
- `agent::citation` (`AIAgentCitation`), `agent::file_locations` (`FileLocations`).

### Multi-agent / orchestration (BYO-agent relevance)

`RunAgentsRequest` and `StartAgentExecutionMode` model **child-agent dispatch** and are the
most "agentic" structures here:

- `StartAgentExecutionMode::Local { harness_type: Option<String>, model_id }` vs.
  `Remote { environment_id, worker_host, harness_type, model_id, computer_use_enabled,
  auth_secret_name, … }`.
- `harness_type` selects a **third-party CLI agent harness** to launch locally ("non-Oz
  harness") — Warp's seam for running an external coding agent as a child. `RunAgentsExecutionMode`
  similarly has `Local` and `Remote { environment_id, worker_host, computer_use_enabled }`.

This `harness_type` mechanism is the closest existing thing to "bring-your-own-agent," but note
the *decision* to spawn a child and *which* harness is still server-driven via the emitted
`RunAgents`/`StartAgent` tool call.

---

## 4. MCP integration — `crates/mcp` + `app/src/ai/mcp`

Built on the `rmcp` crate (`RoleClient`). Warp here is an **MCP client**, connecting to
external tool servers.

- `crates/mcp/src/lib.rs` — `TemplatableMCPServerInfo`: a connected server's `name`,
  `RunningService`, `tools: Vec<rmcp::model::Tool>`, `resources`, `installation_id: Uuid`,
  `is_authenticated_transport`. Exposes `peer()` / `peer_if_connected()`, `has_tool`,
  `tool_input_schema`, etc.
- `crates/mcp/src/runtime.rs` — `spawn_server(name, description, uuid, transport_type, logger,
  auth_context)` connects via `TransportType::{CLIServer (stdio), …}` (streamable-HTTP and SSE
  transports also wired: `ReqwestHttpTransport`, `ReqwestSseTransport`). `error_to_user_message`
  maps `rmcp::RmcpError` to UI strings; capability-gated `query_*_for` helpers list tools/resources.
- `crates/mcp/src/oauth.rs` + `sse_transport/` — OAuth (`oauth2` crate) and custom SSE transport
  with auth for remote MCP servers.
- `app/src/ai/mcp/` — the **managers** that own the live server set:
  `TemplatableMCPServerManager`, file-based config managers (`file_based_manager.rs`,
  `file_mcp_watcher.rs` watches an MCP-config file), `reconnecting_peer.rs`, `gallery.rs`,
  `logs.rs`, `manager/oauth.rs`.

When the agent emits `CallMCPTool { server_id, name, input }` or `ReadMCPResource`, the client
resolves the server via these managers and calls through the `rmcp` peer, returning a
`CallMCPToolResult`. MCP tool input schemas are advertised back to the server so it knows what
it can call.

**Marley relevance:** MCP is **local and self-hostable** — no `app.warp.dev` dependency. It is
the cleanest existing channel for injecting custom capabilities/tools into the agent without
touching the proprietary server, and a strong candidate substrate for Marley's own tooling.

---

## 5. Supporting crates

- **`crates/computer_use`** — `Actor` trait (`perform_actions(&[Action], Options)`) with
  per-OS impls (`mac/`, `windows/`, `linux/{x11,wayland}/`) plus a `noop` test actor.
  `create_actor()` / `is_supported_on_current_platform()`. Backs the `UseComputer` /
  `RequestComputerUse` agent tools (GUI automation: mouse, keyboard, screenshots). Gated by
  `FeatureFlag::AgentModeComputerUse` in `get_supported_tools`.
- **`crates/input_classifier`** — `InputClassifier` trait `detect_input_type(ParsedTokensSnapshot,
  …) -> InputType` (shell vs. natural language). `OnnxClassifier` (feature `onnx`) with
  `HeuristicClassifier` fallback; `InputClassifierDecisionSource` records which path decided.
  This is the **front door to Agent Mode**: it decides whether typed text is run as a command or
  routed to the AI.
- **`crates/natural_language_detection`** — `is_word(word, WordDb::{English,StackOverflow,Command})`
  and `natural_language_words_score(...)`; a heuristic scorer (word lists + `rust_stemmers`) used
  by the input classifier.
- **`crates/warp_completer`** — large shell-completion engine (`completer/`, `parsers/`,
  `signatures/`). Produces `ParsedTokensSnapshot` consumed by `input_classifier`. Not an LLM
  component, but part of the "is this a command?" pipeline.
- **`crates/jsonrpc`** — minimal JSON-RPC 2.0 (`JsonRpcService`, `RequestId`,
  `ServerNotificationEvent`, `Transport`) on `warpui_core` background executor. Consumed by
  `crates/lsp` (only). It is **not** the agent transport (agent uses protobuf-over-SSE) — worth
  noting so Marley doesn't confuse the two RPC layers.

---

## 6. End-to-end data flow

```
user types in terminal block
   │
   ▼
warp_completer (parse) ──► input_classifier (+ natural_language_detection)
   │  shell?  → run as command (terminal subsystem)
   │  NL?     → Agent Mode
   ▼
app/src/ai/agent/...  builds RequestParams
   │   - convert_input(), get_supported_tools(), redaction, api_keys
   ▼
app/src/ai/agent/api/impl.rs :: generate_multi_agent_output
   │   builds warp_multi_agent_api::Request (protobuf)
   ▼
crates/warp_multi_agent_client :: generate_multi_agent_output(BaseClient, &Request)
   │   POST {server_root_url}/ai/multi-agent  (bearer + ambient headers, SSE)
   ▼
══════════════ PROPRIETARY SERVER (app.warp.dev) — LLM + orchestration ══════════════
   │   streams base64(protobuf ResponseEvent): Init / Message(ToolCall) / ClientActions / Finished
   ▼
crates/warp_multi_agent_client  decodes ResponseEvent stream
   ▼
app/src/ai/blocklist/controller/response_stream.rs :: ResponseStream
   │   retry/resume recovery; emits ResponseStreamEvent::ReceivedEvent
   ▼
convert (crates/ai/src/agent/action/convert.rs)  ToolCall → AIAgentActionType
   ▼
action executors (app/src/ai): run shell cmd, edit files, CallMCPTool→crates/mcp,
                               UseComputer→crates/computer_use, RunAgents→child agents
   ▼
AIAgentActionResult ──► fed back as the next Request's input  (loop continues)
```

---

## 7. Marley relevance summary

- **Visualize the agentic workflow.** Everything needed for a live workflow panel already flows
  through one typed event stream. Tap `ResponseStreamEvent::ReceivedEvent` in
  `app/src/ai/blocklist/controller/response_stream.rs` and the `AIAgentActionType` /
  `…ActionResult` enums in `crates/ai/src/agent` to render steps, tool calls, child-agent
  dispatch (`RunAgents`/`StartAgent`), and todos. No server cooperation required to *observe*.
- **Bring-your-own-agent.** The `harness_type` seam (`StartAgentExecutionMode`,
  `RunAgentsExecutionMode`) already supports third-party CLI harnesses as *children*, but the
  primary loop is server-driven. True BYO requires either replacing the backend at the
  `app/src/ai/agent/api/` `ResponseStream` seam or running a server that speaks
  `warp-proto-apis`. MCP (§4) is the no-server-change extension path for custom tools.
- **De-auth.** `generate_multi_agent_output` hard-requires a bearer token via
  `BaseClient::get_or_refresh_access_token()`. A login stub must keep this token producible, or
  the endpoint must be rerouted (`ChannelState::override_server_root_url`) to a Marley server.
- **Rebrand.** User-facing strings live in `crates/mcp/src/runtime.rs::error_to_user_message`,
  the `prevent_sleep("Agent Mode request in-progress")` label, and `ServerConversationToken`'s
  `debug_link()` / `conversation_link()` which embed `app.warp.dev` URLs — all candidates for
  Ignibyte rebranding.
- **Single override knob.** `crates/warp_core/src/channel/config.rs:59` (`server_root_url`) plus
  `ChannelState::override_server_root_url` is the one place that redirects *all* agent traffic.

---

## Marley status @ M15

**The brain is early — and it is the layer Marley intends to sell.** The terminal and editor are the open half; the agent brain is the proprietary, sellable half. Where Marley actually stands on this subsystem:

- **What exists today.** `forge` — Marley's knowledge/pipeline **MCP sidecar** — is live and drives the whole pipeline (tickets, knowledge, goldens, ADs) over MCP. So MCP *wiring is already proven in Marley*: the harness speaks MCP to a real server now. The clean editor↔brain **seam is in place**: `crates/editor/src/types.rs` defines `EditOrigin::{Human, Agent}`, carried on every `EditResult`, so an agent-issued write is distinguishable from a human keystroke *at the buffer boundary* — the exact hook a supervised brain needs (and the seam the [brain-agent-session-supervision intake](../../planning/intake/brain-agent-session-supervision.md) is designed on).
- **What is early / absent.** There is **no agent orchestration loop, no direct-provider streaming client, no agent action/result model, and no agent-session supervision** built in Marley yet. Warp's equivalents (`crates/ai`, `app/src/ai/**`, `warp_multi_agent_client`) are the *reference*, **not** the source — none is ported. The `local_control` + **`session.read`** read-back delta (launch / control / **observe** agent terminals) is intake, not code.
- **The strategic point.** Because terminal + editor are the AGPL/open layer and **the brain is the proprietary, sellable layer**, this is the one subsystem where Marley must *not* port Warp's code. The brain has to be built **clean-room from concepts** (tool-calling, MCP, context assembly, streaming) so the product is legally sellable. See **Provenance & licensing** below — the reason is existential, not stylistic.
- **The M15 build target.** Stand the brain up as a **Marley-original** agent layer: (a) a **direct-provider streaming client** (BYO-key Anthropic / OpenAI / custom-endpoint — *not* Warp's `app.warp.dev` multi-agent transport), (b) a Marley-native action/result vocabulary reimplemented from the concept (not `AIAgentActionType`), (c) **MCP as the tool substrate** reusing `forge` + upstream `rmcp`, and (d) the `EditOrigin::Agent` write path as the editor seam. **Observe-first** is the cheapest first win: Warp funnels every agent step through one typed event stream (§2c) — a workflow visualizer taps that with *no* server cooperation, and Marley's own version taps its own stream the same way.

---

## Provenance & licensing

> **The sharpest provenance call in the entire architecture map.** Warp is **AGPL-3.0**. Marley's posture (chad): AGPL the editor + terminal (free, open), **sell the proprietary brain** — exactly Warp's open-core shape. That makes the licensing boundary in *this* subsystem **existential**: if the sold brain is a derivative of Warp's AGPL agent code, it **cannot be sold**.

### The AGPL network-use trigger — why this is existential, not cosmetic

AGPL-3.0 **§13** extends copyleft to **network interaction**: if you run a *modified* AGPL program and let users interact with it **over a network**, you must offer those users the **complete corresponding source**. The brain is, by design, a **hosted / network service** (an orchestrator the app talks to). Therefore:

- If Marley's brain is **derived from Warp's AGPL agent crates** (`ai`, `warp_multi_agent_client`, the `app/src/ai/**` loop), AGPL copyleft **attaches to the hosted service** — §13 forces Marley to publish the brain's complete source to every network user. **The sold, proprietary brain would be forced open. The resale model dies.** This is worse than ordinary GPL linking: the network trigger means you don't even have to *distribute a binary* to owe source — merely *serving* the derived brain is enough.
- The derivative-work line is **not** crossed by *studying* Warp's architecture (these docs are Marley's own descriptions) or by speaking an open *protocol*. It **is** crossed by lifting or adapting Warp's AGPL source into the brain.

**Conclusion:** the brain must be **clean-room / Marley-original**, built from **agent concepts** (tool-calling, MCP, context assembly, streaming) — **not** from Warp's AGPL agent source. This subsystem's provenance must be the sharpest in the repo precisely because it is the one place where a sloppy derivation is *unsellable*, not merely untidy.

### Provenance tags for this subsystem

| Element | Provenance | Marley action |
|---|---|---|
| `crates/ai` (the agent action/result model — `AIAgentActionType` / `AIAgentActionResultType`), `app/src/ai/**` (the orchestration loop) | **`[Warp-derived/AGPL — DO NOT derive the sold brain from these]`** | **Study only.** Reimplement a Marley-native action/result vocabulary clean-room. Do **not** port the enums or the loop into the brain's derivation chain. |
| `crates/warp_multi_agent_client` (SSE transport → `app.warp.dev`) | **`[Warp-derived/AGPL]`** | **SKIP.** Do not adapt the body. INVENT a Marley direct-provider streaming client (BYO-key). Only the *shape* — a stream of typed events — is a public idea; keep none of the code. |
| `crates/mcp` (Warp's `rmcp` façade) | **`[Warp-derived/AGPL]`** wrapper over **`[permissive/public: rmcp MIT/Apache; MCP is an open protocol]`** | Reimplement the thin wrapper clean-room directly over upstream `rmcp`. The **protocol and the SDK are permissive** — reuse those; don't copy Warp's façade. |
| **MCP itself; tool-calling; context assembly; streaming agent loops** | **`[permissive/public: MCP is an open protocol]`** (Anthropic's open spec) + published agent-design concepts | **Adopt freely.** Open standards / public ideas — the legitimate substrate the clean-room brain is built *from*. `forge` already speaks MCP. |
| `crates/computer_use`, `crates/input_classifier`, `crates/natural_language_detection`, `crates/warp_completer`, `crates/jsonrpc` | **`[Warp-derived/AGPL]`** over **`[permissive/public]`** underlying concepts (OS input/capture APIs, ONNX, word-lists + stemmer, JSON-RPC 2.0 spec) | Capability / utility crates, **not the brain**. Reimplement from the public concept where needed; none belongs *inside* the sold brain's derivation chain. |
| **The Marley brain** (orchestration, direct-provider streaming, agent-session supervision) | **`[Marley-original: the brain must be clean-room]`** | **BUILD.** From concepts, not Warp source. This is the sold product; its provenance must be auditable as Warp-source-free. |
| `EditOrigin::{Human, Agent}` (`crates/editor/src/types.rs`) | **`[Marley-original]`** | **The clean editor↔brain seam — preserve and build on it** (see below). |

### The `EditOrigin::Agent` seam — the licensing firewall made concrete

`EditOrigin::Agent` is more than an enum variant: it is the **copyleft boundary made physical.** The editor is AGPL/open; the brain is proprietary/clean-room. They must communicate **only through Marley-original typed values** — an `EditOrigin` on a buffer write, an MCP tool call, a `session.read` delta — **never by sharing Warp-derived code**. As long as every brain→editor write crosses the `EditOrigin::Agent` seam and every brain→tool invocation crosses MCP, the AGPL copyleft stays where it belongs: **in the open half, off the sold half.** That is what makes the boundary *auditable* — a legal reviewer can check that the brain imports no Warp-derived crate and touches the editor only through this value seam.
