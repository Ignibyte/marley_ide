# warp_js

> Per-crate reference (Marley round 2) for `crates/warp_js`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL]` — Rust↔JS conversion glue; not ported (Marley has no embedded JS runtime). Gap / likely N/A. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [06 — Platform, Settings, Persistence & Infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `license`; no per-crate LICENSE marker) |
| **Internal deps** | 0 |
| **Used by** | 2 |

## Purpose

`warp_js` is the **Rust ↔ JavaScript bridge** for Warp's plugin system. It lets the app
register JS functions (provided by plugins, executed by an embedded
[`rquickjs`](https://crates.io/crates/rquickjs) QuickJS engine), call them from Rust in a
**type-safe** way, and pass inputs/outputs **across process boundaries** (the app process
calls JS that runs in a separate plugin-host process). It is the conversion + registry
layer that makes plugin functions look like ordinary typed Rust calls.

## Key types, modules & public API

`src/lib.rs` re-exports everything from two modules:

- **`js_function/` (`mod.rs` + native.rs)**
  - `SerializedJsValue(Vec<u8>)` — a bincode-serialized function input/output, the
    cross-process transport unit. `from_value<T: Serialize>` / `to_value<T: DeserializeOwned>`.
  - `JsFunctionId` — a UUID-backed unique id for a registered JS function.
  - `TypedJsFunctionRef<I, O>` — a typed handle to a registered function (phantom-typed on
    input `I` / output `O`); `Copy`/serializable so it can be passed around and across
    processes. `#[cfg(feature = "test-util")]` adds `new_for_test()`.
  - `JsFunctionRegistry` (native only) — `new()`, builder `on_registered_js_function(cb)`,
    and `register_js_function::<I, O>(function: rquickjs::Function, ctx) -> TypedJsFunctionRef<I,O>`
    which `Persistent::save`s the function and stores it as an `Arc<dyn CallableJsFunction>`.
  - `CallableJsFunction` trait — object-safe wrapper over the generic `TypedJsFunction<I,O>`
    so heterogeneous functions live in one map.
- **`convert/` (native only)** — the conversion traits, mirroring `rquickjs`'s `FromJs`/`IntoJs`:
  - `FromWarpJs<'js>` — `from_warp_js(ctx, value, &mut JsFunctionRegistry)`; differs from
    `FromJs` by threading the registry so nested JS *functions* get registered during deserialization.
  - `IntoWarpJs<'js>` — `into_warp_js(self, ctx)`; a wrapper over `IntoJs` needed to dodge a
    compiler limitation with recursive blanket impls (e.g. `Vec`).
  - blanket impls for `Vec<T>` etc., plus `convert::native::util`.

## Depends on (internal)

None. (External: `rquickjs` (native only), `bincode`, `serde`, `uuid`, `thiserror`, `cfg-if`.)

## Used by (internal dependents)

- [./warp.md](./warp.md) — the main binary; hosts the plugin runtime and registers/calls JS functions.
- [./warp_completer.md](./warp_completer.md) — completion logic backed by JS plugin functions.

Total: **2** dependents.

## Related crates

- [./warp_completer.md](./warp_completer.md) — a concrete consumer of typed JS function refs.
- Any plugin-runtime / plugin-host crate in the workspace — `warp_js` is the marshalling layer those build on.

## Marley relevance

**KEEP / RENAME (deferred).** This is the plugin/extensibility seam — strategically aligned
with Marley goal (1) "expand the UI surface": a custom panel that runs user/plugin JS logic
would call through `JsFunctionRegistry` + `TypedJsFunctionRef`. No auth and no Warp cloud
branding in the code, so it's not a de-auth (goal 3) blocker. The `warp_js` / `IntoWarpJs` /
`FromWarpJs` names are Warp-branded and would be renamed in a full rebrand (goal 4) — but
they thread through the conversion traits and 2 dependents, so **defer the rename** until
the broader `warp_*` → `marley_*` sweep. If Marley ships without the plugin system at first,
this whole crate can be left compiled-but-unused rather than removed (it has zero internal
deps, so it's cheap to keep).

## Notes / gotchas

- **Native vs WASM split**: the entire `convert` and the `js_function::native` halves (and the `rquickjs` dependency) are `cfg(not(target_family = "wasm"))`. On WASM only the type-marker layer (`SerializedJsValue`, `JsFunctionId`, `TypedJsFunctionRef`) exists; `lib.rs` `#[cfg_attr(wasm, allow(unused_imports))]`s the convert re-export.
- **Out-of-repo heavyweight dep**: embeds **QuickJS** via `rquickjs` — a C engine; affects build time and platform support.
- `IntoWarpJs` exists purely to work around a Rust compiler limitation around recursive blanket `IntoJs` impls (documented in `convert/native/mod.rs`) — don't try to collapse it back into `IntoJs`.
- `JsFunctionRegistry` is **thread-local** by design (persisted `rquickjs` values aren't `Send`); don't assume cross-thread sharing.
