# warp_graphql_schema

> Per-crate reference (Marley round 2). Crate dir: `crates/warp_graphql_schema`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** the cynic SDL registration for the `warp-server` GraphQL schema. Marley is local-first → **N/A** (Warp backend schema). See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (no own `LICENSE` marker; inherits workspace `license.workspace = true`) |
| Internal deps | 0 |
| Used by | 1 |

## Purpose

`warp_graphql_schema` is the single source of truth for the Warp server's GraphQL **schema registration**. It owns the SDL file (`api/schema.graphql`, ~4,442 lines) and registers it with [`cynic`](https://cynic-rs.dev) under the schema name `"warp-server"` so that every other crate can generate strongly-typed GraphQL operations against it at compile time.

It exists as its own crate (deliberately separated from `warp_graphql`) so the schema is registered exactly once, in one place, and the large generated/derived type surface has a stable home. Note the version is pinned at `0.0.0` — it is an internal plumbing crate, never published independently.

## Key types, modules & public API

- `src/lib.rs` — the entire public surface is a single macro invocation:
  ```rust
  #[cynic::schema("warp-server")]
  pub mod schema {}
  ```
  This expands (via `cynic`) into the `schema` module containing Rust marker types for every GraphQL type, field, enum, and input in the SDL (e.g. `schema::Time`, `schema::Uint`, query/mutation root markers).
- `build.rs` — registers the schema for codegen:
  ```rust
  cynic_codegen::register_schema("warp-server")
      .from_sdl_file("api/schema.graphql")
      .expect(...)
      .as_default()?;
  ```
- `api/schema.graphql` — the canonical SDL. `api/client-schema.ts` sits alongside it (TS-side mirror used by the web client tooling).

## Depends on (internal)

None. This is a leaf crate (`deps: []`). Its only real dependency is the external `cynic` / `cynic-codegen` crates plus the checked-in SDL file.

## Used by (internal dependents)

- [`warp_graphql`](./warp_graphql.md) — re-exports `schema` and builds all typed query/mutation/subscription fragments on top of it. (Total internal dependents: 1.)

## Related crates

- [`warp_graphql`](./warp_graphql.md) — the operations/client layer that consumes this schema.
- Everything transitively GraphQL: [`warp_server_client`](./warp_server_client.md), [`warp_server_auth`](./warp_server_auth.md), and the `cloud_object_*` crates all reach the schema via `warp_graphql`.

## Marley relevance

**Classification: KEEP (rename deferred).**

This crate is the contract with Warp's hosted GraphQL backend. For Marley's four goals it is mostly inert plumbing:

- **(3) de-auth + login stub:** We do not gut the schema. The schema can stay intact; the *operations* that hit the network are short-circuited upstream in `warp_graphql` / `warp_server_client`. Keeping the SDL means typed code still compiles even while the network calls are stubbed.
- **(4) de-Warp rebrand:** The schema name string `"warp-server"` is load-bearing — it must match between `build.rs` here and `build.rs` in `warp_graphql`. Renaming it to e.g. `"marley-server"` is a coordinated two-file change and offers no user-visible benefit, so **defer** any rename. The package name `warp_graphql_schema` could be renamed to `marley_graphql_schema`, but it has 1 dependent (`warp_graphql`) so it is a low-cost rename if/when we do a sweep.

If Marley eventually points at its own backend, this is where a replacement SDL would land. Until then: KEEP, do not touch.

## Notes / gotchas

- **Generated code at build time.** Nothing here is hand-written types; `cynic::schema(...)` + `cynic_codegen` generate the type surface from the SDL during the build. A stale or malformed `api/schema.graphql` breaks the whole workspace compile.
- **Two `register_schema` call sites.** Both this crate and `warp_graphql` register `"warp-server"` from the *same relative SDL path* (`warp_graphql/build.rs` reads `../warp_graphql_schema/api/schema.graphql`). Moving the SDL file requires updating both build scripts.
- **`version = "0.0.0"`** signals this is non-semver internal plumbing — don't treat it as a stable API crate.
- Tiny code footprint, but it gates compilation of a large fraction of the workspace; treat changes here as high-blast-radius.
