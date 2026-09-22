# cloud_object_client

> Per-crate reference (Marley round 2). Crate dir: `crates/cloud_object_client`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** the GraphQL CRUD/sync/sharing client for Warp Drive objects. Marley is local-first → **N/A**. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true`; no own `LICENSE` marker) |
| Internal deps | 4 |
| Used by | 2 |

## Purpose

`cloud_object_client` defines the **`ObjectClient` trait — the abstract API the rest of Warp calls to talk to Warp Drive**: create/update/delete notebooks, workflows, folders, and generic string objects; move/trash/transfer them; manage link-sharing and guests; subscribe to live updates; and do the initial bulk load. It is the seam between "the app wants to mutate a cloud object" and "an HTTP/GraphQL request actually goes to Warp's servers."

Critically, this crate is **interface + value types only** — the concrete networking implementation lives in [`warp_server_client`](./warp_server_client.md). That makes `ObjectClient` the natural mock/stub point (it carries an `#[automock]` impl under `test-util`).

It also acts as the **umbrella re-export**: `pub use cloud_object_models::*;` and `pub use cloud_objects::cloud_object::*;`, so consumers get the trait and all model/substrate types from one crate.

## Key types, modules & public API

Everything is in `src/lib.rs`:

- **`trait ObjectClient: 'static + Send + Sync`** — the headline. `#[async_trait]` (with a `?Send` variant for wasm), `#[cfg_attr(..., automock)]`. ~35 async methods, including:
  - Creation/update: `create_workflow`, `update_workflow`, `create_notebook`, `update_notebook`, `create_folder`, `update_folder`, `create_generic_string_object`, `bulk_create_generic_string_objects`, `update_generic_string_object`.
  - Lifecycle: `trash_object`, `untrash_object`, `delete_object`, `empty_trash`, `move_object`, `leave_object`, `transfer_notebook_owner` / `transfer_workflow_owner` / `transfer_generic_string_object_owner`.
  - Sync/load: `fetch_changed_objects` (returns `InitialLoadResponse`), `fetch_single_cloud_object` (`GetCloudObjectResponse`), `get_warp_drive_updates(message_sender, stream_ready_sender)` — the live websocket subscription that streams `ObjectUpdateMessage`s.
  - Editing/concurrency: `grab_notebook_edit_access` / `give_up_notebook_edit_access`, `record_object_action`.
  - Sharing: `set_object_link_permissions`, `remove_object_link_permissions`, `add_object_guests`, `update_object_guests`, `remove_object_guest`, `fetch_environment_last_task_run_timestamps`.
- **Value/result types:** `ObjectUpdateMessage` (the streamed change enum: `ObjectMetadataChanged`, `ObjectContentChanged`, `ObjectDeleted`, `ObjectActionOccurred`, `AmbientTaskUpdated`, …, with `.as_str()`), `InitialLoadResponse`, `GetCloudObjectResponse`, `ObjectAction`/`ObjectActionHistory`/`ObjectActionType`/`ObjectActionSubtype`, `GuestIdentifier { Email, TeamUid }`, and the per-op result enums (`ObjectDeleteResult`, `ObjectMetadataUpdateResult`, `ObjectPermissionUpdateResult`, `ObjectPermissionsUpdateData`).
- **`test-util` feature** → `mockall::automock` produces `MockObjectClient`.

## Depends on (internal)

- [`cloud_object_models`](./cloud_object_models.md) — the concrete models the methods accept/return (re-exported wholesale).
- [`cloud_objects`](./cloud_objects.md) — ids, `ObjectType`, `Owner`, sharing levels (`cloud_object::*` re-exported).
- [`warp_core`](./warp_core.md) — core utilities/feature flags.
- [`warp_graphql`](./warp_graphql.md) — `AccessLevel`, `MCPGalleryTemplate`, and the GraphQL types method signatures expose.

(Plus external `async-trait`, `async-channel`, `chrono`, `mockall` (optional), `uuid`.)

## Used by (internal dependents)

- [`warp_server_client`](./warp_server_client.md) — provides the real, networked `impl ObjectClient`.
- [`warp`](./warp.md) — the app holds an `Arc<dyn ObjectClient>` and drives all Drive operations through it.

(Total internal dependents: 2.)

## Related crates

- [`warp_server_client`](./warp_server_client.md) — the concrete implementation (reqwest + cynic GraphQL + RTC websocket).
- [`cloud_object_models`](./cloud_object_models.md) / [`cloud_objects`](./cloud_objects.md) — the data it moves.
- [`cloud_object_persistence`](./cloud_object_persistence.md) — where `InitialLoadResponse` results land locally.
- [`warp_graphql`](./warp_graphql.md) — the operations underlying each method.

## Marley relevance

**Classification: STUB (the highest-value de-auth lever in this batch).**

This trait is the **clean injection point for offline Marley.** The whole cloud surface narrows to one `dyn ObjectClient`.

- **(3) de-auth + login stub:** Ship a **`StubObjectClient`** (or reuse the `MockObjectClient` from `test-util`) that satisfies `ObjectClient` entirely from the local store: creates/updates return synthesized ids, `fetch_changed_objects` returns a default `InitialLoadResponse` from SQLite, `get_warp_drive_updates` simply never streams (and immediately fires `stream_ready_sender`), and sharing/guest ops are no-ops. Inject it in `warp` instead of `warp_server_client`'s real impl. This severs the GraphQL/Firebase dependency without touching every call site.
- **(2) session/object read/write:** A stub backed by [`cloud_object_persistence`](./cloud_object_persistence.md) gives Marley fully local workflows/notebooks/MCP servers.
- **(1) custom panel:** A Marley Drive panel can be developed against `ObjectClient` directly; using the mock keeps the UI buildable before any backend exists.
- **(4) rebrand:** `ObjectUpdateMessage` strings (`"ObjectMetadataChanged"`, etc.) are internal/log-only; method names like `get_warp_drive_updates` are rename candidates but low-priority.

Recommendation: **STUB** — implement a local `ObjectClient` and stop wiring in the networked one. Keep the trait shape as-is so a future optional sync backend can be re-added.

## Notes / gotchas

- **Interface, not implementation:** there is *zero* networking in this crate — easy to mistake for the client. The HTTP/GraphQL lives in `warp_server_client`.
- **`automock` is gated on `any(test, feature = "test-util")`** — Marley should turn on `test-util` (and its `cloud_object_models/test-util`, `cloud_objects/test-util` passthroughs) to get `MockObjectClient` for the stub path.
- **wasm dual-trait:** `async_trait(?Send)` on `target_family = "wasm"` vs `async_trait` elsewhere — any Marley stub must compile under both if web targets stay.
- `get_warp_drive_updates` takes **two** `async_channel::Sender`s (messages + a `stream_ready` signal); a stub must still fire `stream_ready_sender` or callers may block waiting for the subscription to come up.
- `ToString for ObjectActionType` carries an inline `#[allow(clippy::to_string_trait_impl)]` with a justification comment — keep the justification if you touch it.
