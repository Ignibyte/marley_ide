# cloud_object_persistence

> Per-crate reference (Marley round 2). Crate dir: `crates/cloud_object_persistence`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** the local SQLite mirror that reconciles Warp Drive objects against the server. Marley is local-first (no cloud store to mirror) → **N/A**. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true`; no own `LICENSE` marker) |
| Internal deps | 4 |
| Used by | 2 |

## Purpose

`cloud_object_persistence` holds the **shared SQLite persistence infrastructure** for Warp cloud objects — the model-*agnostic* half of "save a Warp Drive object to disk." From `lib.rs`:

> "This crate defines shared SQLite persistence infrastructure for Warp cloud objects. It owns model-agnostic persistence helpers for object metadata, permissions, refresh scheduling, guest and link-sharing encoding, callback-based object upsert and delete operations, and generic string object table access."

The design deliberately splits responsibilities: the **shared** metadata/permissions/refresh plumbing lives here; the **per-model** read/write adapters live in `cloud_object_models`'s model-local `persistence` modules (which call into this crate). Note the inversion — this crate does **not** depend on `cloud_object_models`; the models depend on it.

## Key types, modules & public API

Three private modules re-exported through `src/lib.rs`:

- **`objects`** (`src/objects.rs`) — the core. Callback-based upsert/delete so each model supplies its own table writer:
  - Type aliases `CloudObjectId = i32`, and the boxed closures `CreateCloudObjectFn`, `UpdateCloudObjectFn`, `DeleteCloudObjectFn` (each handed a `&mut SqliteConnection` already inside a transaction).
  - Functions: `upsert_cloud_object`, `delete_cloud_object`, `update_object_metadata`, `update_object_after_server_creation`, `mark_object_as_synced`, `increment_retry_count`, `id_from_metadata`, `metadata_object_type_key`, `to_cloud_object_metadata`, `to_cloud_object_permissions`, `load_cloud_object_read_context`.
  - Generic-string-object table access: `read_generic_string_object_rows`, `upsert_generic_string_objects`, `delete_generic_string_object`, structs `GenericStringObjectPersistenceData`, `GenericStringObjectRow`, and `CloudObjectReadContext`.
- **`encoded_permissions`** (`src/encoded_permissions.rs`) — bincode/serde encoding of share state for SQLite: `encode_guests` / `decode_guests`, `encode_link_sharing` / `decode_link_sharing` (operating on `cloud_objects` `CloudObjectGuest`, `CloudLinkSharing`, `SharingAccessLevel`).
- **`refresh`** (`src/refresh.rs`) — the Warp Drive force-refresh schedule: `record_time_of_next_refresh(conn, timestamp)` and `read_time_of_next_force_object_refresh(conn)` against the `cloud_objects_refreshes` table.

## Depends on (internal)

- [`cloud_objects`](./cloud_objects.md) — the substrate types (`CloudObjectMetadata`, `CloudObjectPermissions`, `SyncId`, `ObjectType`, sharing) it serializes.
- [`persistence`](./persistence.md) — the diesel schema (`persistence::schema`) and ORM row models (`ObjectMetadata`, `ObjectPermissions`, `GenericStringObject`, `NewCloudObjectsRefresh`, …).
- [`warp_core`](./warp_core.md) — `FeatureFlag` gating used in the persistence paths.
- [`warp_graphql`](./warp_graphql.md) — `ServerTimestamp` and wire scalars stored in rows.

(Plus external `diesel` (sqlite+chrono), `bincode`, `log`.)

## Used by (internal dependents)

- [`cloud_object_models`](./cloud_object_models.md) — its model-local `persistence` submodules build on these shared helpers (native targets only).
- [`warp`](./warp.md) — top-level app.

(Total internal dependents: 2.)

## Related crates

- [`persistence`](./persistence.md) — the actual diesel schema + migrations this crate writes through.
- [`cloud_object_models`](./cloud_object_models.md) — the per-model persistence adapters that call these functions.
- [`cloud_object_client`](./cloud_object_client.md) — produces the server objects that get upserted here after a sync.

## Marley relevance

**Classification: KEEP / EXTEND.**

This is the crate that makes Warp Drive objects **work offline** — it's the local cache. That is squarely aligned with Marley's local-first goals.

- **(2) session/object read/write:** The callback-upsert pattern (`upsert_cloud_object` + a model `CreateCloudObjectFn`) is exactly the mechanism a Marley panel would use to persist workflows / env-vars / MCP servers to a local SQLite file *without any server*. **KEEP and EXTEND** as the storage backend.
- **(3) de-auth:** `refresh.rs` schedules the cloud force-refresh; with the de-auth stub there is no server to refresh from, so `read_time_of_next_force_object_refresh` can be left to return `None` (or the refresh scheduler simply never fires). Permissions encoding still works for a single local owner.
- **(4) de-Warp rebrand:** No user-facing strings — pure SQLite plumbing. Table name `cloud_objects_refreshes` lives in the `persistence` crate's schema, not here.

Recommendation: **KEEP**. This is the part of the cloud stack worth preserving as Marley's local object store; the de-auth work happens upstream in the *client*, leaving these writers intact.

## Notes / gotchas

- **Inverted dependency on purpose:** this crate must *not* depend on `cloud_object_models` (enforced by the lib doc). Model-specific adapters live with the models; only generic helpers live here. Don't "consolidate" them back together.
- **Everything runs inside a diesel transaction** the caller opens; the `*Fn` callbacks assume the connection is already transactional — calling them standalone will misbehave.
- `bincode` is used for the `source` field of link-sharing and guest encoding — a binary format; schema-coupled, so version bumps to those structs can break stored rows.
- diesel features `["sqlite", "chrono"]`; native-only by nature (no wasm sqlite here), which is why `cloud_object_models` gates it behind `cfg(not(wasm))`.
