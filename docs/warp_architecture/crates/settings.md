# settings

> Per-crate reference (Marley round 2) — crate dir `crates/settings`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Marley-original]` — Warp's AGPL settings framework; **Marley shipped its own** `marley_settings` (typed declarative TOML, M1.B) — local-only, non-syncable, no cloud-sync seam. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no per-crate marker) |
| **Internal deps** | 4 |
| **Used by** | 4 |

## Purpose

`settings` is the typed, declarative settings framework for the app. It lets each
feature crate declare its user/app preferences as strongly-typed Rust structs/enums
and get, for free: persistence to the right backend (TOML settings file vs.
platform-native store vs. secure storage), JSON-Schema generation for the settings
file, cloud-sync routing (Warp Drive), per-platform gating, hot-reload from disk,
and change events delivered through the `warpui_core` entity/model system.

The core problem it solves is keeping **three** representations of every setting in
sync without per-setting boilerplate: (1) the in-memory typed value, (2) the
serialized form in a durable store, and (3) the human-editable TOML file
representation. It does this through a trait (`Setting`) plus a family of macros
(`define_settings_group!`, `define_setting!`, `implement_setting_for_enum!`) and a
runtime registry (`SettingsManager`).

## Key types, modules & public API

- **`trait Setting`** (`src/lib.rs`) — the central abstraction. Associated types
  `Group: Entity` and `Value: Serialize + DeserializeOwned + PartialEq + Debug + SettingsValue`.
  Key methods: `value()`, `set_value()`, `set_value_from_cloud_sync()`,
  `load_value()` (hot-reload, no write-back), `clear_value()`, `default_value()`,
  `storage_key()`, `toml_path()` / `toml_key()` / `hierarchy()`, `is_private()`,
  `sync_to_cloud()`, `supported_platforms()`. Provided methods handle the actual
  I/O: `read_from_preferences()`, `write_to_preferences()` (semantic diff before
  write), `clear_from_preferences()`, `new_from_storage()`, and
  `preferences_for_setting()` which routes private→`PrivatePreferences`,
  public→`PublicPreferences` (gated on the `SettingsFile` feature flag).
- **`trait SecureSetting: Setting`** — overlay for values that live in OS secure
  storage (`read_from_secure_storage`, `write_to_secure_storage`,
  `clear_from_secure_storage`), via `warpui_extras::secure_storage`.
- **`trait ToggleableSetting: Setting`** — blanket impl for `Value: Not + Copy`,
  exposes `toggle_and_save_value()`.
- **`SettingsManager`** (`src/manager.rs`) — a `SingletonEntity` registry keyed by
  storage key. `register_setting(...)` stores closures (`UpdateFn`, `ClearFn`,
  `LoadFn`, `EqualsFn`, `IsSyncableFn`) plus `SettingsInfo` metadata. Public API:
  `update_setting_with_storage_key`, `load_setting`, `reload_all_public_settings`,
  `validate_all_public_settings`, `clear_cloud_settings_local_state`,
  `read_local_setting_value`, `public_storage_keys`, `default_values_for_settings_file`.
- **Preferences newtypes** (`src/lib.rs`): `PublicPreferences` and
  `PrivatePreferences`, each wrapping a `Box<dyn warpui_extras::user_preferences::UserPreferences>`
  and both `SingletonEntity`s. `PublicPreferences` deliberately hides its inner
  backend behind `pub(crate) as_preferences()` so nothing bypasses the macros.
- **Enums** (`src/lib.rs`): `SupportedPlatforms` (`ALL/DESKTOP/MAC/LINUX/WINDOWS/WEB/OR`
  with `matches_current_platform()`), `SyncToCloud` (`Globally/PerPlatform/Never`),
  `RespectUserSyncSetting`, `ChangeEventReason`.
- **Schema generation** (`src/schema.rs`): `SettingSchemaEntry` collected via the
  `inventory` crate (`inventory::collect!`); the `submit_schema_entry!` macro emits
  one entry per setting so a generator binary can build the settings JSON Schema.
- **Macros** (`src/macros.rs`): `define_settings_group!`, `define_setting!`,
  `maybe_define_setting!`, `implement_setting_for_enum!`, `register_settings_events!`.
  A `const _` compile-time assert enforces "non-private settings must specify a
  toml_path".
- **`const fn` helpers**: `toml_path_storage_key()` and `toml_path_hierarchy()`
  split a dotted `toml_path` (e.g. `"appearance.text.font_name"`).

## Depends on (internal)

- [settings_value](./settings_value.md) — provides the `SettingsValue` trait (with
  `derive` feature) that governs the TOML-file representation of each `Value` type;
  `Setting` calls `to_file_value`/`from_file_value`/`file_schema` on it.
- [warpui_core](./warpui_core.md) — the entity/model runtime: `AppContext`,
  `Entity`, `ModelContext`, `SingletonEntity`. Setting groups *are* entities and
  emit change events through it. (built with `settings_value` feature)
- [warpui_extras](./warpui_extras.md) — supplies `user_preferences::UserPreferences`
  (the backend trait) and `secure_storage`, the actual durable stores behind the
  framework.
- [warp_features](./warp_features.md) — `FeatureFlag` (notably `FeatureFlag::SettingsFile`)
  to gate the TOML-file backend and per-flag schema inclusion.

## Used by (internal dependents)

- [warp_core](./warp_core.md) — registers and reads the bulk of the app's settings.
- [warp](./warp.md) — top-level app wiring and the settings UI.
- [cloud_object_models](./cloud_object_models.md) — settings that sync via Warp Drive.
- [integration](./integration.md) — integration-test harness exercising settings.

Total: 4 dependents.

## Related crates

- [settings_value](./settings_value.md) / [settings_value_derive](./settings_value_derive.md)
  — the file-format layer this crate sits on top of.
- [warp_features](./warp_features.md) — feature flags that gate settings.
- [persistence](./persistence.md) — the *other* durable store (SQLite/Diesel) for
  non-preference app state; conceptually adjacent but a separate concern.

## Marley relevance

**Classification: KEEP (with surgical STUBs).**

This crate is core plumbing and touches three of the four Marley goals:

1. **Expand the UI with a custom panel** — a Marley panel that has any user-facing
   preferences should declare them with `define_settings_group!` here rather than
   inventing a parallel store; that gets it free TOML persistence + schema + the
   settings UI surface.
3. **De-auth + login stub** — `SyncToCloud` / Warp Drive sync is the auth-coupled
   part. For offline boot, **stub** the cloud-sync path: have
   `SettingsManager::clear_cloud_settings_local_state` and
   `is_current_value_syncable` short-circuit (treat everything as
   `SyncToCloud::Never`) so no setting attempts a Drive round-trip. Keep the local
   TOML/private-store paths fully intact.
4. **De-Warp rebrand** — mostly cosmetic here: the package name stays, but the
   default settings-file path, schema `$id`, and any `warp.dev`-referencing
   descriptions in downstream `define_setting!` calls get rebranded. The crate
   itself has no hard-coded "Warp" string in its logic.

Do **not** rename the package: it is depended on by `warp_core`, `warp`,
`cloud_object_models`, and `integration` — a rename ripples widely for no product
value. Keep the API stable and change behavior via the feature flags it already
exposes (`FeatureFlag::SettingsFile`).

## Notes / gotchas

- **`inventory`-based registration**: schema entries are collected at link time via
  `inventory::collect!`/`submit!`. This relies on link-section magic and can be
  fragile under aggressive dead-code stripping / unusual link configs and on wasm.
- **Compile-time invariant**: `define_setting!` embeds a `const { panic!(...) }`
  that fails the build if a non-private setting omits `toml_path` — expect build
  errors, not runtime errors, when adding settings.
- **Two serialization paths**: serde is used for the native/secure stores; the
  `SettingsValue` trait is used *only* for the TOML file. They can disagree (e.g.
  `Duration` is `{secs,nanos}` in serde but integer-seconds in the file). Keep that
  in mind when adding Marley settings.
- **`edition = "2024"`** and heavy macro use (`concat-idents`, `#[macro_use]`) — the
  macros are large (`src/macros.rs` ≈ 950 lines) and re-export `inventory`,
  `schemars`, `settings_value`, `warpui_core` as `#[doc(hidden)]` for downstream
  macro expansion. Downstream crates need those in scope transitively.
- Hot-reload uses `load_value`/`load_setting` (in-memory only) plus
  `inhibit_writes_for_key` to avoid file-watcher write-back loops — preserve that if
  you touch reload logic.
