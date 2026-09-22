# settings_value

> Per-crate reference (Marley round 2) — crate dir `crates/settings_value`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Marley-original]` — folded into `marley_settings::SettingsValue`, a blanket `serde`-over-`toml::Value` trait; no separate crate. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no per-crate marker) |
| **Internal deps** | 1 |
| **Used by** | 7 |

## Purpose

`settings_value` defines exactly one thing: the **`SettingsValue` trait**, which
controls how a value type is represented in the user-visible TOML settings file —
a *parallel* serialization path that is independent of serde.

It exists because the on-disk human-edited file wants a friendlier shape than serde
produces by default (e.g. enums as snake_case strings, `Duration` as integer
seconds instead of `{ secs, nanos }`), while cloud sync and the platform-native
stores still want plain serde. Splitting this into its own tiny crate lets many
crates implement `SettingsValue` for their own types **without** depending on the
much heavier [`settings`](./settings.md) crate (avoiding a dependency cycle, since
`settings` itself depends on this one).

## Key types, modules & public API

All in `src/lib.rs`:

- **`trait SettingsValue: Serialize + DeserializeOwned`** with three methods, all
  defaulted:
  - `fn to_file_value(&self) -> serde_json::Value` — default delegates to
    `serde_json::to_value`.
  - `fn from_file_value(value: &Value) -> Option<Self>` — default delegates to
    `serde_json::from_value(...).ok()`.
  - `fn file_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema` —
    default delegates to `schemars::JsonSchema`.
  Because every method is defaulted, a serde-passthrough type only needs
  `impl SettingsValue for T {}`.
- **`#[derive(SettingsValue)]`** — re-exported from
  [`settings_value_derive`](./settings_value_derive.md) when the `derive` feature is
  on (`pub use settings_value_derive::SettingsValue;`).
- **`macro_rules! impl_snake_case`** — bulk serde-passthrough impls for external
  (orphan-rule) types: `impl_snake_case!(TypeA, TypeB, ...)`.
- **Blanket/primitive impls**: `impl_default_file_format!` covers `bool`, the
  integer/float primitives, `String`, `PathBuf`, `DateTime<Utc>`. Generic recursive
  impls for `Vec<T>`, `Option<T>`, `HashSet<T>`, `HashMap<K, V>`. A custom impl for
  `instant::Duration` (integer seconds, schema = `u64`).

The crate doc comment itself explains the three implementation strategies (derive /
empty passthrough / manual override) — useful guidance for Marley type authors.

## Depends on (internal)

- [settings_value_derive](./settings_value_derive.md) — the proc-macro that
  generates `SettingsValue` impls; pulled in only under the optional `derive`
  feature (`derive = ["dep:settings_value_derive"]`).

## Used by (internal dependents)

- [settings](./settings.md) — consumes the trait for every setting's file representation.
- [warp_core](./warp_core.md), [warp](./warp.md) — implement `SettingsValue` for app value types.
- [cloud_object_models](./cloud_object_models.md), [cloud_objects](./cloud_objects.md) — cloud value types.
- [warp_server_client](./warp_server_client.md) — server-facing value types.
- [warpui_core](./warpui_core.md) — built with its `settings_value` feature.

Total: 7 dependents (one of the more widely used leaf crates).

## Related crates

- [settings_value_derive](./settings_value_derive.md) — its companion proc-macro.
- [settings](./settings.md) — the framework that *uses* this trait at the file boundary.

## Marley relevance

**Classification: KEEP (verbatim).**

This is a pure, brand-neutral serialization-format trait with no auth, no network,
no "Warp" strings in its logic. It is foundational to goal (1) — any custom Marley
panel's settings types must implement `SettingsValue` to be file-persistable — and
otherwise irrelevant to the de-auth / login-stub / rebrand goals.

Leave it untouched. The only conceivable change is a package rename for a thorough
rebrand, but with **7 dependents** and zero user-visible surface, a rename is pure
churn — **defer / skip**.

## Notes / gotchas

- The `derive` feature is **off by default**; `settings` opts in via
  `settings_value = { workspace = true, features = ["derive"] }`. A new crate that
  wants `#[derive(SettingsValue)]` must enable the feature.
- The derive path **bypasses serde entirely** (it does not call
  `serde_json::to_value`), whereas the empty-impl passthrough respects custom
  `Serialize`/`Deserialize`. Choosing the wrong strategy silently changes the file
  format — see the crate-level docs.
- Uses `instant::Duration` (not `std::time::Duration`) for wasm-compatible timing.
- `edition = "2021"` and a pinned `version = "0.1.0"` (unlike most workspace crates
  that inherit version).
