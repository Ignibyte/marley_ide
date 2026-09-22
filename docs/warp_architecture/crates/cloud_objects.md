# cloud_objects

> Per-crate reference (Marley round 2). Crate dir: `crates/cloud_objects`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** the Warp Drive substrate (server ids, object metadata/permissions, sharing). Marley keeps workflows/settings as **local files** (`marley_settings`) → **N/A**. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true`; no own `LICENSE` marker) |
| Internal deps | 5 |
| Used by | 5 |

## Purpose

`cloud_objects` is the **low-level, model-agnostic substrate** for everything Warp stores in "Warp Drive" (the cloud object store: notebooks, workflows, folders, env-var collections, MCP servers, etc.). Its own `lib.rs` doc says it best:

> "This crate defines the low-level, model-agnostic cloud object substrate shared by Warp crates. It owns server-facing identifiers, user identifiers, object metadata, object type and format definitions, and sharing or drive primitives that do not depend on concrete Warp object models."

It deliberately holds **no concrete model payloads** (those live in [`cloud_object_models`](./cloud_object_models.md)), **no SQLite code**, and **no app runtime state**. It is the shared vocabulary — IDs, `ObjectType`, metadata, permissions, sharing levels — that the model, persistence, and client crates all build on.

## Key types, modules & public API

Four top-level modules (`src/lib.rs`):

- **`ids`** (`src/ids.rs`) — the identifier zoo. `ClientId(Uuid)`, `ServerId`, `SyncId { ClientId | ServerId }` (an object that may or may not have been synced yet), `FolderId`, `GenericStringObjectId`, `ObjectUid`, `HashedSqliteId`. The `HashableId` trait (`to_hash` / `from_hash`) gives every id a stable string form prefixed by type (e.g. `Client-<uuid>`). Many derive `schemars::JsonSchema`.
- **`cloud_object`** (`src/cloud_object/mod.rs` + submodules) — the core abstractions:
  - `ObjectType { Notebook, Workflow, Folder, GenericStringObject(GenericStringObjectFormat) }` and `ObjectIdType`, with `sqlite_object_type_as_str()` / `sqlite_prefix()` for persistence.
  - `GenericCloudObject` / `GenericServerObject` generic wrappers and the `ServerObject` trait (`server_object.rs`) — the type-erased `Box<dyn ServerObject>` used across the client API.
  - `creation.rs` → `CreateObjectRequest`, `Owner`; `update.rs` → `Revision`, update result types; `generic_string_model.rs` → `GenericStringModel`, `GenericStringObjectFormat`; metadata/permissions types (`ServerMetadata`, `ServerPermissions`, `CloudLinkSharing`, `CloudObjectGuest`).
- **`drive`** (`src/drive/mod.rs`) — `CloudObjectTypeAndId` (id+type pair passed between actions) and **`drive::sharing`**: `SharingAccessLevel { View, Edit, Full }` with capability predicates (`can_trash`, `can_delete`, `can_move_drive`), plus `Subject`, `TeamKind`, `UserKind` describing share targets.
- **`auth`** (`src/auth/mod.rs`) — a thin re-export shim: `pub use warp_server_auth::user_uid;` and `UserUid`, `TEST_USER_UID`, `TEST_USER_EMAIL`.

## Depends on (internal)

- [`warp_core`](./warp_core.md) — feature flags (`FeatureFlag`), UI primitives (`Icon`, `Appearance`, theme `Fill`) used by object metadata/rendering helpers.
- [`warp_graphql`](./warp_graphql.md) — `AccessLevel`, `ServerTimestamp` and other GraphQL scalars that ids/permissions map onto the wire format.
- [`warp_server_auth`](./warp_server_auth.md) — re-exported `user_uid` / `UserUid` so cloud objects can name owners and guests.
- [`settings_value`](./settings_value.md) — settings value types embedded in object models.
- [`warpui_core`](./warpui_core.md) — `Element` and UI element types (Stack, Align, Hoverable…) for drive-row rendering helpers baked into the object layer.

(Also external: `cynic`, `session-sharing-protocol`, `pathfinder_geometry`, `lasso`, `schemars`.)

## Used by (internal dependents)

- [`cloud_object_models`](./cloud_object_models.md) — builds concrete models on these aliases.
- [`cloud_object_persistence`](./cloud_object_persistence.md) — persists this metadata/permissions to SQLite.
- [`cloud_object_client`](./cloud_object_client.md) — the GraphQL client trades in these ids/types.
- [`warp_server_client`](./warp_server_client.md) — server I/O for the same.
- [`warp`](./warp.md) — top-level app.

(Total internal dependents: 5.)

## Related crates

- [`cloud_object_models`](./cloud_object_models.md) — the concrete payloads layered on top of this substrate.
- [`warp_server_auth`](./warp_server_auth.md) — owns `UserUid`/`user_uid` re-exported here.
- [`warp_graphql`](./warp_graphql.md) — the wire vocabulary these ids serialize to.
- `session-sharing-protocol` *(external / not a workspace crate)* — supplies `Role`/`ProfileData` used in `drive::sharing`.

## Marley relevance

**Classification: KEEP (rename deferred).**

This is foundational plumbing, not a UI or login surface — it does the right thing in isolation and is depended on by 5 crates.

- **(2) session spawn/write/read:** Although named "cloud," the type vocabulary here (`ObjectType`, `SyncId`, metadata) is what Warp Drive uses to store notebooks/workflows. If Marley keeps a local "drive" for its custom panel content, these types are reusable as a **local** object model — keep them and swap the *client* (see `cloud_object_client`) rather than this layer.
- **(3) de-auth + login stub:** The `auth` module only re-exports `UserUid` / test-user constants — harmless. `TEST_USER_UID` / `TEST_USER_EMAIL` are exactly the seam an offline stub leans on (a canned owner for every object).
- **(4) de-Warp rebrand:** Only naming. No user-visible "Warp" strings here beyond `Warp Drive`-flavored comments; safe to leave until a coordinated rename. A `warp_*` → `marley_*` rename is **deferred** because of the 5 dependents.

Recommendation: **KEEP as-is** for offline boot; revisit only if Marley drops cloud objects entirely.

## Notes / gotchas

- **`edition = "2024"`.** Uses `cynic` (GraphQL) and `lasso` (string interning) transitively even though it has "no networking" — these come in via `warp_graphql`/model glue.
- `ToString`/`Display` are used as the **serialization** mechanism for `ObjectType`/`ObjectActionType` (Warp notes this is temporary, with a `clippy::to_string_trait_impl` allow). Don't "modernize" to a serde impl without checking SQLite round-trips.
- `auth/mod.rs` re-exporting `warp_server_auth::user_uid` means a change to that crate's id type ripples here silently.
- Hidden UI dependency: pulling `warpui_core` `Element`s into the "model-agnostic" layer means this crate is **not** truly headless — a real concern if Marley ever wants cloud objects in a non-GUI tool.
