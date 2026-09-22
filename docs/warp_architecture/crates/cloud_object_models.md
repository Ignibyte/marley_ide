# cloud_object_models

> Per-crate reference (Marley round 2). Crate dir: `crates/cloud_object_models`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** concrete Warp Drive typed models (workflow/notebook/folder/mcp/env_vars). Marley is local-first → **N/A** (local files, not a synced cloud store). See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true`; no own `LICENSE` marker) |
| Internal deps | 11 |
| Used by | 4 |

## Purpose

`cloud_object_models` defines the **concrete, typed Warp Drive object models** — the actual payloads stored as cloud objects — on top of the model-agnostic [`cloud_objects`](./cloud_objects.md) substrate. From `lib.rs`:

> "This crate defines the concrete Warp cloud object models and typed cloud object aliases built on top of `cloud_objects`. Each model module should own the model payload for one cloud object family, plus any model-specific adapters."

One module per object family; each provides a model struct plus `Cloud*` / `Server*` type aliases that specialize the generic `GenericCloudObject` / `GenericServerObject` wrappers, and (on native, non-wasm targets) a `persistence` submodule with the SQLite read/write adapters specific to that model.

## Key types, modules & public API

`src/lib.rs` re-exports every module with a glob (`pub use notebook::*;` etc.; `#![allow(ambiguous_glob_reexports)]` because several modules each carry a `persistence` submodule). Per-family highlights:

- **`notebook`** — `CloudNotebookModel`, `SerializedNotebook`, `NotebookId(ServerId)`; aliases `CloudNotebook`, `ServerNotebook`.
- **`workflow`** — the big one: `Workflow` (the runnable command template), `Argument` / `ArgumentType`, `CloudWorkflowModel`, `WorkflowId`; aliases `CloudWorkflow`, `ServerWorkflow`.
- **`folder`** — `CloudFolderModel`, `CloudFolder`, `ServerFolder`.
- **`mcp`** — MCP server records as cloud objects: `MCPServer`, `JSONMCPServer`, `TransportType`/`JSONTransportType`, `CLIServer`, `ServerSentEvents`, `TemplatableMCPServer` + `GalleryData`/`JsonTemplate`/`TemplateVariable` (the MCP gallery), with `CloudMCPServer = GenericCloudObject<GenericStringObjectId, GenericStringModel<MCPServer, JsonSerializer>>`.
- **`cloud_environment`** — ambient/cloud-agent environments: `AmbientAgentEnvironment`, `SourceRepo`, `GithubRepo`, `CodeForge`, `BaseImage`, `ProvidersConfig` (`GcpProviderConfig`/`AwsProviderConfig`), `EnvironmentSecretRef`.
- **`env_vars`** — `EnvVarCollection`, `EnvVar`, `EnvVarValue`, `ExternalSecret` (`OnePasswordSecret`, `LastPassSecret`), plus helpers `serialize_variables_internal()` and `get_init_command_for_env_var_value()`.
- **`json_model`** — the `JsonModel` trait and `JsonSerializer` that back all the `GenericStringModel`-based families.
- **`server_cloud_object`** — `ServerCloudObject` (a sum type over all server object families) and the `TryFromGql` trait for decoding GraphQL into models.
- **`user_profile`** — `UserProfileWithUID`, `UserProfileIdAndName`, `TeamProfileIdAndName`.
- Plus `ai_execution_profile`, `ai_fact`, `cloud_agent_config`, `preference`, `scheduled_ambient_agent`, `workflow_enum`.

## Depends on (internal)

- [`cloud_objects`](./cloud_objects.md) — the substrate (ids, `GenericCloudObject`, `ObjectType`) these models specialize.
- [`cloud_object_persistence`](./cloud_object_persistence.md) — shared SQLite helpers used by the model-local `persistence` submodules (native only).
- [`persistence`](./persistence.md) — the diesel schema/models for the SQLite tables (native only).
- [`ai`](./ai.md) — LLM/agent types referenced by `ai_execution_profile`, `ai_fact`, `cloud_agent_config`.
- [`warp_cli`](./warp_cli.md) — CLI command shapes embedded in workflow/agent models.
- [`warp_core`](./warp_core.md) — feature flags and core utilities.
- [`warp_graphql`](./warp_graphql.md) — GraphQL scalars/decoders for `TryFromGql`.
- [`warp_util`](./warp_util.md) — misc shared utilities.
- [`settings`](./settings.md) / [`settings_value`](./settings_value.md) — settings payloads embedded in models (e.g. preferences).
- [`handlebars`](./handlebars.md) — templating for MCP gallery / workflow argument substitution.

(Plus external `warp-workflows`, `serde_regex`, `schemars`, `session-sharing-protocol`.)

## Used by (internal dependents)

- [`cloud_object_client`](./cloud_object_client.md) — re-exports all of these (`pub use cloud_object_models::*;`) and trades in them across the GraphQL API.
- [`mcp`](./mcp.md) — consumes the MCP-server cloud models.
- [`warp_server_client`](./warp_server_client.md) — server I/O.
- [`warp`](./warp.md) — top-level app.

(Total internal dependents: 4.)

## Related crates

- [`cloud_objects`](./cloud_objects.md) — substrate beneath.
- [`cloud_object_persistence`](./cloud_object_persistence.md) — the shared persistence half; this crate's model-local `persistence` modules call into it.
- [`cloud_object_client`](./cloud_object_client.md) — the network client that surfaces these models.
- [`mcp`](./mcp.md) / [`ai`](./ai.md) — consumers of the MCP and agent model families.

## Marley relevance

**Classification: KEEP / EXTEND.**

This is where the actual data shapes live, and several of them are directly useful to Marley.

- **(1) custom panel + (2) session read/write:** `Workflow`, `EnvVarCollection`, `MCPServer`, and `notebook` models are reusable for a Marley panel that lists/edits saved commands, env-var sets, and MCP servers — backed locally instead of by Warp's cloud. **EXTEND**: keep the model structs, point their `persistence` adapters at a local DB, and skip the server round-trip.
- **(3) de-auth:** Models carry `Owner`/`UserProfileWithUID` from `cloud_objects`; with the auth stub these resolve to the canned test user. No login logic lives here.
- **(4) de-Warp rebrand:** Mostly internal type names; the user-facing strings to watch are in `mcp` gallery templates and `cloud_agent_config` (defaults that reference Warp services). Rename deferred (4 dependents, glob re-exports make renames noisy).

Recommendation: **KEEP** the model definitions; **EXTEND** the persistence adapters for local-first storage. Prune families Marley won't ship (`scheduled_ambient_agent`, `cloud_environment`, `cloud_agent_config`) only after confirming nothing in `warp`/`mcp` references them.

## Notes / gotchas

- **Native-vs-wasm split:** `cloud_object_persistence`, `diesel`, and `persistence` are gated behind `cfg(not(target_family = "wasm"))`. On wasm the models compile **without** SQLite adapters — don't assume `persistence` submodules always exist.
- **Glob re-export collisions:** every family globs out a `persistence` submodule, hence `#![allow(ambiguous_glob_reexports)]`; the doc comment instructs you to use fully-qualified paths (`notebook::persistence::…`) rather than the glob.
- **`agent_mode_evals` feature** is a marker feature (`[]`) gating eval-only code paths.
- Heaviest crate in this batch (11 internal deps incl. `ai`, `settings`, `warp_cli`); a slow rebuild target — avoid churn here.
