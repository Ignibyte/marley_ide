# persistence

> Per-crate reference (Marley round 2) — crate dir `crates/persistence`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — deliberately NOT ported]` — the Diesel/SQLite account+teams DB is AGPL and de-auth-sensitive; Marley persists **only layout**, via its own grid/shell text codecs (`marley_app/grid_layout.rs`, #163/#205/#243/#258) — no account/identity/teams DB. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no per-crate marker) |
| **Internal deps** | 0 (per depgraph; see Notes — Cargo.toml also lists `warp_multi_agent_api`) |
| **Used by** | 5 |

## Purpose

`persistence` is the local **SQLite/Diesel** persistence layer for the terminal
app's durable structured state — the database that restores your windows, tabs,
panes, blocks, workflows, notebooks, MCP servers, agent conversations/tasks, user
profile, teams, and cloud-object metadata across restarts.

It is the structured-state counterpart to the [`settings`](./settings.md)
preferences layer: where `settings` stores small key/value preferences, this crate
stores the relational app state via Diesel ORM, an embedded-migrations bundle, and
typed row models.

## Key types, modules & public API

- **`src/lib.rs`** — tiny surface:
  - `pub mod model;` and `pub mod schema;`
  - `pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations` — gated on the
    `local_fs` feature, produced by `embed_migrations!("migrations")`. Consumers run
    this against a SQLite connection to bring a DB up to schema.
- **`src/schema.rs`** — `@generated` by Diesel CLI; **50** `diesel::table!`
  definitions including `windows`, `tabs`, `panes` (`pane_branches`/`pane_leaves`/
  `pane_nodes`), `blocks`, `workflows`, `notebooks`, `commands`, `folders`,
  `projects`, `project_rules`, `mcp_server_installations`/`active_mcp_servers`/
  `mcp_environment_variables`, `agent_conversations`/`agent_tasks`,
  `ai_document_panes`/`ai_memory_panes`/`ai_queries`, `cloud_objects_refreshes`,
  `object_metadata`/`object_actions`/`object_permissions`, `teams`/`team_members`/
  `team_settings`, `user_profiles`, `workspaces`/`workspace_metadata`, `app`, etc.
- **`src/model.rs`** (≈1.5k lines) — Diesel row structs implementing
  `Queryable`/`Insertable`/`Identifiable`, named after tables. Examples: `Window`,
  `NewApp`, `GenericStringObject` / `NewGenericStringObject<'a>`, `Workflow` /
  `NewWorkflow`. Several models bridge to the agent API types
  (`warp_multi_agent_api::{self as api, response_event::stream_finished}`) for
  serializing agent conversation/task payloads.
- **`build.rs`** — sets `cargo:rustc-cfg=feature="local_fs"` whenever the target
  family is not `wasm`, so native builds always get migrations/embedded-DB support
  and wasm builds compile it out.

## Depends on (internal)

- *(none recorded in the depgraph)* — see Notes. The Cargo manifest does pull the
  internal **`warp_multi_agent_api`** crate (used by `model.rs` to (de)serialize
  agent conversation/task rows), but the depgraph keys this crate with `deps: []`.

Third-party: `diesel` (features `chrono`, `32-column-tables`), `diesel_migrations`,
`chrono`, `serde`.

## Used by (internal dependents)

- [warp](./warp.md) — the app boots the DB, runs `MIGRATIONS`, and restores state.
- [warp_graphql](./warp_graphql.md) — maps persisted rows to/from GraphQL.
- [ai](./ai.md) — persists AI/agent conversation, task, and document-pane state.
- [cloud_object_models](./cloud_object_models.md) / [cloud_object_persistence](./cloud_object_persistence.md)
  — persist cloud-object metadata and refresh bookkeeping.

Total: 5 dependents.

## Related crates

- [settings](./settings.md) — sibling durable layer (preferences vs. relational state).
- [cloud_object_persistence](./cloud_object_persistence.md) — builds on these tables
  for cloud-object sync.
- [virtual-fs](./virtual-fs.md) — unrelated mechanically, but the other "filesystem
  state" crate in this subsystem doc.

## Marley relevance

**Classification: KEEP (rebrand the on-disk DB path/name; do not rename the crate).**

Directly serves goal (2) **session spawn/write/read** — terminal panes, blocks,
tabs, and agent conversations are exactly the session state a Marley fork must read
and write. Keep the schema and models as-is to preserve restore compatibility.

- **De-Warp rebrand (goal 4)**: the *table* `warp_ai_width` / `warp_drive_index_width`
  columns and any "warp"-named on-disk database file should be rebranded at the
  edges. Be careful: column renames require new Diesel **migrations**, and the
  `schema.rs` is generated — change the migration + regenerate, never hand-edit
  generated output. Cosmetic DB-file path renames (`warp.sqlite` → `marley.sqlite`)
  are low-risk and worth doing.
- **De-auth / login stub (goal 3)**: tables like `teams`, `team_members`,
  `current_user_information`, `user_profiles`, `server_experiments` carry
  account-bound state. For offline boot, leave the tables present but have callers
  **stub** them to empty/local defaults rather than dropping the schema (dropping
  risks migration/restore breakage).

A package rename is unjustified: 5 dependents and a deeply load-bearing schema. The
brand lives in *data*, not the crate name — rebrand the data, keep the crate.

## Notes / gotchas

- **Feature-via-build.rs quirk**: `local_fs` is enabled by `build.rs`, *not* by the
  feature resolver. The Cargo.toml comment explicitly notes this is why
  `diesel_migrations` cannot be made `optional` — the resolver never sees the
  feature. Don't "clean this up" by marking it optional; it'll break wasm/native
  builds.
- **Generated `schema.rs`**: `// @generated automatically by Diesel CLI.` Edit via
  migrations + `diesel print-schema`, not by hand.
- **Long migration history**: `migrations/` goes back to 2021 (`2021-10-14_…`),
  embedded into the binary. Adding columns means adding a new dated migration dir.
- **Depgraph vs. manifest mismatch**: the depgraph lists no internal deps, but
  `model.rs` imports `warp_multi_agent_api`. Treat that as a real edge when
  refactoring (e.g. de-authing the agent API ripples into row (de)serialization).
- `32-column-tables` Diesel feature is enabled because some tables (e.g. `windows`)
  are very wide.
