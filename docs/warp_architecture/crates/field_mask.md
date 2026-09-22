# field_mask

> Per-crate reference (Marley round 2) for `crates/field_mask`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — `prost_reflect` FieldMask ops; gRPC/proto plumbing, N/A for Marley's offline build. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [06 — Platform, Settings, Persistence & Infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `license`; no per-crate LICENSE marker) |
| **Internal deps** | 0 |
| **Used by** | 1 |

## Purpose

`field_mask` implements protobuf **`FieldMask` semantics** (`google.protobuf.FieldMask`):
given a source and a destination message of the same type plus a list of dotted field
paths, it produces a new message where only the masked fields are copied from source to
destination. This is the standard wire pattern for *partial updates* — a server (or
client) sends only the fields that changed, plus a mask naming them, and this crate merges
them into the existing object. It is a tiny, self-contained, dependency-free (internally)
leaf utility.

## Key types, modules & public API

Everything lives in **`src/lib.rs`** (single file):

- **`FieldMaskOperation<'a, T: prost::Message + Default>`** — the entry point. Built with
  one of two constructors and consumed with `apply()`:
  - `FieldMaskOperation::update(descriptor, destination, source, mask)` — for each masked
    path, **replace** the destination field with the source value.
  - `FieldMaskOperation::append(descriptor, destination, source, mask)` — for each masked
    **string** field, **concatenate** source onto destination (`format!("{value}{patch_value}")`); errors with `UnsupportedAppend` for non-string fields.
  - `apply(self) -> Result<T>` — transcodes both messages to `prost_reflect::DynamicMessage`, walks each path, and transcodes back to `T`.
- **`FieldMaskError`** — `Decode`, `InvalidPath`, `UnsupportedAppend`, `SetField`; plus the
  crate's `Result<T>` alias.
- Internal: `OperationType { Update, Append }` and the recursive `apply_path(...)` helper
  that handles nested message paths and repeated (`List`) fields element-wise.

The descriptor is taken as a `&'static MessageDescriptor` (from `prost-reflect`), so
callers pass the reflection descriptor for `T`.

## Depends on (internal)

None. (External crates only: `prost`, `prost-reflect`, `prost-types`, `itertools`, `thiserror`.)

## Used by (internal dependents)

- [./warp.md](./warp.md) — the main binary, the sole consumer; applies partial updates to protobuf-backed objects (e.g. cloud/account state deltas).

Total: **1** dependent.

## Related crates

- [./cloud_object_models.md](./cloud_object_models.md) — the protobuf object models that field masks would typically be applied to (it sits next to `handlebars` as another small proto-adjacent leaf).
- Any crate generating `prost` types (the protobuf codegen pipeline) is conceptually upstream.

## Marley relevance

**KEEP (likely dormant).** Pure data-transformation logic with no auth, no branding, no
network. It is only reachable from `warp` and only matters when Marley exchanges
partial-update protobufs with a backend — which is exactly the cloud/account path Marley's
de-auth goal (3) aims to short-circuit. So in practice this code becomes **dead weight once
the cloud sync seam is stubbed**: keep it compiled (removing a zero-internal-dep leaf buys
nothing and risks the single `warp` callsite), but expect its callsites to disappear when
account/team sync is disabled. No rename needed — `field_mask` is already a generic,
un-Warp-branded name. Revisit for REMOVE only after the cloud object-sync path is fully
gone.

## Notes / gotchas

- Relies on **`prost-reflect` dynamic messages**: it transcodes `T` → `DynamicMessage` → mutate → `T` on every `apply()`, which is allocation-heavy; not for hot loops.
- **Unknown fields no-op silently**: `apply_path` returns `Ok(())` when a masked field name isn't in the descriptor (documented as forward-compat with newer server schemas) — a typo'd path will be silently ignored, not error.
- **Repeated fields require equal lengths**: nested paths through a `List` error with `InvalidPath` unless target and patch lists are the same length and all elements are messages.
- Uses `edition = "2024"`.
