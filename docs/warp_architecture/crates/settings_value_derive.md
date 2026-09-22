# settings_value_derive

> Per-crate reference (Marley round 2) — crate dir `crates/settings_value_derive`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Marley-original / obviated]` — `marley_settings::SettingsValue` is a blanket impl over `Serialize + DeserializeOwned`, so **no derive proc-macro is needed**. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no per-crate marker) |
| **Internal deps** | 0 |
| **Used by** | 1 |

## Purpose

A `proc-macro` crate (`[lib] proc-macro = true`) providing **`#[derive(SettingsValue)]`**.
It auto-generates the `to_file_value` / `from_file_value` implementations of the
[`settings_value`](./settings_value.md) `SettingsValue` trait so that setting value
types get the correct TOML-file representation without hand-writing recursive
serialization.

It exists purely to remove boilerplate from the file-format layer: hand-writing
`to_file_value`/`from_file_value` for every enum and struct setting type would be
tedious and error-prone, especially the snake_case variant conversion and recursive
descent into inner fields.

## Key types, modules & public API

Single entry point in `src/lib.rs`:

- **`#[proc_macro_derive(SettingsValue, attributes(serde))]`** →
  `pub fn derive_settings_value(input: TokenStream) -> TokenStream`.

Generation rules (the macro reads `#[serde(...)]` attributes to stay consistent
with serde where it matters):

- **Enums**: unit variants → snake_case JSON string; single-field tuple/struct
  variants → single-key object `{ "snake_case_variant": <recursive value> }`;
  multi-field tuple variants → array under that key. Variant `#[cfg(...)]` attrs are
  preserved.
- **Structs**: named-field structs serialize to a JSON object, each field value
  produced by a recursive `to_file_value()` call; field names default to the Rust
  identifier but honor `#[serde(rename = "...")]`. Newtype structs (`struct Foo(T)`)
  delegate to the inner type.
- **serde attribute handling**: container `#[serde(rename_all = "...")]`,
  field/variant `#[serde(rename)]`, `#[serde(skip)]` (excluded; filled via
  `Default` on read), `#[serde(default)]` (fall back to `Default` when absent).

Internal helpers (private fns): `get_serde_rename_all`, `file_variant_name`,
`get_cfg_attrs`.

External deps: `syn` v2, `quote`, `proc-macro2`, `convert_case` (for the
snake_case conversion).

## Depends on (internal)

- *(none)* — leaf proc-macro crate; pulls only third-party `syn`/`quote`/etc.

## Used by (internal dependents)

- [settings_value](./settings_value.md) — re-exports this derive under its `derive`
  feature (`pub use settings_value_derive::SettingsValue`). All transitive users of
  the derive go through `settings_value`, never this crate directly.

Total: 1 dependent.

## Related crates

- [settings_value](./settings_value.md) — defines the trait this derive implements;
  the two are a matched pair.
- [settings](./settings.md) — the ultimate consumer (enables the `derive` feature).

## Marley relevance

**Classification: KEEP (verbatim).**

Pure compile-time codegen. No runtime behavior, no auth, no network, no "Warp"
strings — entirely brand-neutral. It supports goal (1) indirectly: custom Marley
panels that define enum/struct settings can `#[derive(SettingsValue)]` instead of
hand-rolling file serialization.

No reason to rename (single dependent, no public surface area beyond the derive
name; renaming the derive would force edits across every `#[derive(SettingsValue)]`
site for zero benefit). Leave it alone.

## Notes / gotchas

- The generated code hard-references `::settings_value::SettingsValue` and
  `serde_json` by absolute path, so consumers must have both in scope (they get
  `settings_value` transitively via the re-export, and `serde_json` is a common
  workspace dep).
- `edition = "2024"`. `syn` is built with `default-features = false` plus an
  explicit feature set (`derive`, `parsing`, `proc-macro`, `printing`,
  `extra-traits`) — keep that list if you extend the macro, or compilation of the
  proc-macro will fail.
- Because the derive **bypasses serde** for value production, a type that derives
  both `Serialize` and `SettingsValue` can have two *different* JSON shapes; only
  `#[serde(rename...)]`/`skip`/`default` are mirrored, not full serde semantics.
