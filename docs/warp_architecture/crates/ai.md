# ai

> Per-crate reference (Marley round 2). Crate dir: `crates/ai`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL — DO NOT derive the sold brain from these] · Marley status: clean-room-target — study the action/result model; reimplement a Marley-native agent vocabulary from concepts, never port these enums into the sold brain (AGPL §13 network trigger — see subsystem doc's Provenance & licensing).

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`; no per-crate LICENSE marker) |
| **Internal deps** | 15 |
| **Used by** | 3 |

## Purpose

`ai` is the heart of Warp's Agent Mode: the **client-side domain model** for the AI
agent. It does *not* talk to the LLM transport itself (that is `warp_multi_agent_client`
+ `warp_multi_agent_api`); instead it owns everything the app needs to *represent,
persist, validate, and reason about* an agent conversation locally:

- the agent **action / action-result** type system (every tool the agent can call),
- **API key / credential** management (BYO-key for Anthropic, OpenAI, Google, OpenRouter, custom endpoints; plus AWS and GEAP/Google-enterprise and xAI/Grok OAuth),
- the **codebase index** (file outlines, full-source-code embeddings for semantic search),
- **project context** (global rules / `WARP.md`-style rules), **skills** (SKILL.md parsing & providers), **document** model, **diff validation**, **workspace** metadata, and GFM table rendering.

It is the largest crate in the subsystem and the integration point the top-level `warp`
binary wires everything else into.

## Key types, modules & public API

- **`agent`** (`src/agent/mod.rs`) — the agent domain model.
  - `agent::action::AIAgentActionType` (`src/agent/action/mod.rs`) — the master enum of every action the LLM can request: `RequestCommandOutput`, `EditDocuments`, `ReadFiles`, `FileGlob`/`FileGlobV2`, `Grep`, `SearchCodebase`, `CallMCPTool`, `ReadMCPResource`, `UseComputer`/`RequestComputerUse`, `RunAgents`/`StartAgent`/`SendMessageToAgent`, `AskUserQuestion`, `ReadSkill`, `WaitForEvents`, etc.
  - `agent::action_result::AIAgentActionResultType` (`src/agent/action_result/mod.rs`) — the matching result type per action (`CallMCPToolResult`, `UseComputerResult`, `SearchCodebaseResult`, …).
  - `agent::{AIAgentCitation, FileLocations, group_file_contexts_for_display}`, `agent::orchestration_config`, `agent::convert` (proto ↔ domain conversion).
  - Re-exports `warp_multi_agent_api::LifecycleEventType`.
- **`api_keys`** (`src/api_keys.rs`) — `ApiKeys` (with `anthropic`/`openai`/`google`/`open_router`/`custom_endpoints`), `ApiKeyManagerEvent`, secure-storage backed (`SECURE_STORAGE_KEY = "AiApiKeys"`). Sibling credential modules: `aws_credentials` (`AwsCredentials`), `geap_credentials` (`GeapCredentials`), `grok_subscription` (xAI OAuth, native-only).
- **`llm_id::LLMId`** — newtype string identifier for a model.
- **`index`** (`src/index/mod.rs`) — `Outline`, `Symbol`, `build_outline` (feature `local_fs`); `full_source_code_embedding` manager/chunker for semantic code search; re-exports `repo_metadata` tree types (`FileMetadata`, `DirectoryEntry`, `BuildTreeError`).
- **`skills`** (`src/skills/mod.rs`) — `parse_skill`, `ParsedSkill`, `read_skills`, `SkillReference`, `SkillProvider`, `SkillScope`, `SKILL_PROVIDER_DEFINITIONS` — the SKILL.md ingestion pipeline.
- **`project_context`** — `GlobalRules` (feature-gated native/dummy) + `project_context::model`.
- **`diff_validation::ParsedDiff`**, **`document::AIDocumentId`**, **`workspace::WorkspaceMetadata`**, **`gfm_table`**.

Cargo features: `local_fs` (native filesystem indexing), `crash_reporting` (sentry), `jemalloc`, `test-util`.

## Depends on (internal)

- [`computer_use`](./computer_use.md) — supplies `UseComputer` action result types / actor model embedded in the agent action set.
- [`http_client`](./http_client.md) — HTTP for embedding sync and credential flows (native target).
- [`languages`](./languages.md) & [`syntax_tree`](./syntax_tree.md) — tree-sitter language defs + parse trees that drive file outlines and code chunking.
- [`repo_metadata`](./repo_metadata.md) — git-aware repo tree walking (`FileMetadata`, gitignore matching) for the index.
- [`persistence`](./persistence.md) — SQLite-backed storage of workspace metadata and index state.
- [`string-offset`](./string-offset.md) — UTF-8/UTF-16 offset math for citations and diffs.
- [`virtual-fs`](./virtual-fs.md) — filesystem abstraction (native vs wasm) used by indexing/documents.
- [`watcher`](./watcher.md) — file-change watching to keep the index fresh.
- [`warp_core`](./warp_core.md) — channel/state primitives shared app-wide.
- [`warp_graphql`](./warp_graphql.md) — GraphQL types for cloud AI calls.
- [`warp_terminal`](./warp_terminal.md) — `model::BlockId` and terminal block model the agent references.
- [`warp_util`](./warp_util.md) — misc shared utilities.
- [`warpui_core`](./warpui_core.md) — `Entity`/`ModelContext`/`SingletonEntity` reactive runtime for the managers.
- [`warpui_extras`](./warpui_extras.md) — `secure_storage` for API keys/credentials.

> Also pulls `warp_multi_agent_api` (proto types) and `rmcp` from Cargo, though the depgraph records those edges elsewhere/externally.

## Used by (internal dependents)

- [`warp`](./warp.md) — the top-level app binary; primary consumer.
- [`onboarding`](./onboarding.md) — uses API-key/credential setup during first-run.
- [`cloud_object_models`](./cloud_object_models.md) — shares agent/object model types.

## Related crates

- [`warp_multi_agent_client`](./warp_multi_agent_client.md) — the wire transport that streams agent responses this crate models.
- [`mcp`](./mcp.md) — runtime that fulfils the `CallMCPTool` / `ReadMCPResource` actions defined here.
- [`computer_use`](./computer_use.md) — fulfils the `UseComputer` action.

## Marley relevance

**Classify: KEEP + EXTEND (deferred RENAME).** This is the single most important crate
for Marley's "session spawn/write/read" goal — `AIAgentActionType` /
`AIAgentActionResultType` *are* the protocol our custom panel would read from and write
into. The four goals map directly:

1. **Custom panel** — our UI surfaces agent actions/results by consuming these enums; no fork needed, just new views over existing types.
2. **Session spawn/write/read** — `agent::action` (`StartAgent`, `SendMessageToAgent`, `RunAgents`) is the seam for driving sessions programmatically. EXTEND here if we add Marley-native session ops.
3. **De-auth + login stub** — `api_keys.rs` is where BYO-key lives; for an offline/local boot we STUB the cloud-credential paths (GEAP, Grok OAuth, Warp-issued keys) and route only to a user-pasted Anthropic/OpenAI/custom-endpoint key, short-circuiting `aws_credentials`/`geap_credentials`.
4. **De-Warp rebrand** — the package name `ai` is already neutral (no rename needed), but it re-exports/depends on `warp_*` crates and uses `"AiApiKeys"` storage keys and Warp cloud endpoints that carry branding.

Renaming the *package* is unnecessary (already generic); the real work is gutting the
Warp-cloud credential coupling while keeping the action model intact.

## Notes / gotchas

- **Heavy cfg surface.** Large `local_fs` vs wasm split (the whole `index` module and `project_context::GlobalRules` swap implementations); native-only `grok_subscription`, `http_client`, `syntax_tree`, `watcher`. Building for wasm strips most of the indexing engine.
- Depends on `warp_multi_agent_api` for proto types (`LifecycleEventType`, request/response) generated upstream — those protos define the cloud contract.
- Uses `rmcp` directly (not via the `mcp` crate) for some MCP model types in action results.
- Secure storage is platform-keychain backed via `warpui_extras::secure_storage`; tests for it are `api_keys_tests.rs`.
- `priority-queue`, `rayon` thread pool (`warp-code-indexing-*` threads, capped at 2) for embedding work — CPU heavy on large repos.
