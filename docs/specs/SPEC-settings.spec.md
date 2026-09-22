---
spec_id: settings
component: marley_settings
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Typed declarative settings framework (TOML file + typed groups)
goal: Let each Marley feature declare its user preferences as strongly-typed Rust values and get TOML-file persistence, defaults, hot-reload, and change events for free — with no per-setting boilerplate and no cloud sync.
reuses: [toml, serde]
spec_source: "behavior-only reference (observable I/O): a typed declarative settings framework that persists feature-declared preference groups to a single human-editable TOML file under the app config dir, with per-setting defaults, caller-driven hot-reload, and change notifications — no fork file paths and no private type/module/static names"
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
---

## Purpose
`marley_settings` is Marley's typed, declarative settings framework. A feature crate declares a settings group as strongly-typed Rust values and, without writing persistence code, gets: a TOML file representation under `~/.marley/settings.toml`, a default for every setting, caller-driven hot-reload from disk, and change notifications. It keeps the human-editable TOML file and each feature's typed value in sync through a `Setting` trait, a `SettingsValue` file-format trait (a serialization path parallel to and independent of serde), a `define_settings_group!` macro family, and a runtime `SettingsManager` registry keyed by a setting's storage key. Resolution is **lazy**: the manager retains the parsed file as an in-memory working tree and resolves each setting's typed value from that tree only when the caller asks for it, so registration order and load order are independent. Per the Marley charter this build is **local TOML only**: there is no cloud-sync or secure-storage backend, and every setting behaves as non-syncable.

## Public surface (the contract)
All under `crates/marley_settings/src/`.

- `pub trait SettingsValue: Serialize + DeserializeOwned + Sized` (`value.rs`) — the TOML-file representation, all methods defaulted:
  - `fn to_file_value(&self) -> toml::Value` — default delegates to `toml::Value::try_from(self)`.
  - `fn from_file_value(value: &toml::Value) -> Option<Self>` — default delegates to `value.clone().try_into().ok()`.
  - blanket/primitive impls for `bool`, integer/float primitives, `String`, `PathBuf`, `Vec<T>`, `Option<T>`; a custom impl for `Duration` (integer **whole seconds** in the file — see R16's precision contract).
- `pub trait Setting: 'static` (`lib.rs`) — associated `type Value: SettingsValue + PartialEq + Debug + Clone`. Methods: `fn default_value() -> Self::Value`; `fn toml_path() -> &'static str`; `fn is_private() -> bool` (default `false`). Provided: `fn storage_key() -> &'static str`, `fn hierarchy() -> Vec<&'static str>`.
- `pub fn toml_path_storage_key(path: &'static str) -> &'static str` and `pub fn toml_path_hierarchy(path: &'static str) -> Vec<&'static str>` — split a dotted `toml_path` (e.g. `"appearance.text.font_name"`) into its leaf key (`"font_name"`) and its ordered parent segments (`["appearance", "text"]`). **Not `const`**: `toml_path_hierarchy` allocates a `Vec`, which a `const fn` cannot do, and splitting a `&'static str` into borrowed sub-slices is not available in a `const fn` without `unsafe` (which this crate forbids); both helpers are therefore plain `fn`s evaluated at call time. The compile-time `toml_path`-presence guard (R2) is a `const { assert!(...) }` on the literal and does **not** depend on these helpers.
- `pub struct SettingsManager` (`manager.rs`):
  - `pub fn load(path: PathBuf) -> Result<Self, SettingsError>` — parses the file into a retained `toml::Value` working tree (or an empty table if the file is absent); **does not** eagerly populate per-setting values.
  - `pub fn register<S: Setting>(&mut self)` — indexes `S` by `S::storage_key()`; no file or working-tree access.
  - `pub fn get<S: Setting>(&self) -> S::Value` — resolves lazily from the working tree's `toml_path` node via `from_file_value`, falling back to `S::default_value()`.
  - `pub fn set<S: Setting>(&mut self, value: S::Value) -> Result<(), SettingsError>`
  - `pub fn clear<S: Setting>(&mut self) -> Result<(), SettingsError>`
  - `pub fn reload(&mut self) -> Result<(), SettingsError>` (caller-driven hot-reload, no write-back)
  - `pub fn default_values(&self) -> toml::Value`
  - `pub fn subscribe(&mut self, f: impl Fn(&ChangeEvent) + 'static) -> Subscription`
  - `pub fn is_syncable<S: Setting>(&self) -> bool`
- `pub struct ChangeEvent { pub storage_key: &'static str, pub reason: ChangeReason }`; `pub enum ChangeReason { Set, Cleared, Reloaded }`.
- `pub enum SettingsError { Parse(toml::de::Error), Io(std::io::Error), Serialize(toml::ser::Error) }` (`impl std::error::Error`).
- Macros (`macros.rs`): `define_settings_group!`, `define_setting!` — `define_setting!` embeds a `const { assert!(...) }` requiring a non-private setting to specify a non-empty `toml_path`.

## EARS Requirements
R1. The system shall provide a `Setting` trait whose `Value` associated type is bound by `SettingsValue + PartialEq + Debug + Clone`, and shall expose each setting's default through `default_value()`.

R2. WHEN `define_setting!` is invoked for a setting that is not private and omits (or supplies an empty) `toml_path`, the system shall fail to compile via a `const { assert!(...) }` build error.

R3. WHEN `toml_path_storage_key("a.b.c")` is evaluated, the system shall return `"c"`; and WHEN `toml_path_hierarchy("a.b.c")` is evaluated, the system shall return the ordered segments `["a", "b"]`.

R4. WHEN `register::<S>()` is called, the system shall index the setting by `S::storage_key()`; and IF a second setting with an already-registered storage key is registered, THEN the system shall panic with a message naming the duplicate key.

R5. WHEN `SettingsManager::load(path)` is called and `path` does not exist, the system shall return a manager whose working tree is an empty table — so every registered setting resolves to its `default_value()` — without creating the file.

R6. WHEN `get::<S>()` is called after a valid file was loaded and `S` was registered, the system shall resolve the value from the loaded file's working-tree node at `S::toml_path()` via `SettingsValue::from_file_value`, returning that file value whenever the node is present and decodable.

R7. IF the settings file contains syntactically invalid TOML, THEN `load(path)` shall return `Err(SettingsError::Parse(..))` and shall not panic.

R8. WHEN a registered setting's `toml_path` is absent from the loaded working tree, the system shall resolve `get::<S>()` to `S::default_value()`.

R9. IF the working-tree node at a setting's `toml_path` is present but `from_file_value` returns `None` (type mismatch), THEN `get::<S>()` shall resolve to `S::default_value()`, and resolving it shall not mutate the working tree or any other setting's resolved value.

R10. WHEN `set::<S>(v)` is called and `v` differs from the current resolved value, the system shall write `v.to_file_value()` into the working tree at `S`'s `toml_path`, persist the full working tree to the file as TOML, and emit a `ChangeEvent { storage_key, reason: ChangeReason::Set }`.

R11. WHEN `set::<S>(v)` is called and `v` equals the current resolved value (semantic diff via `PartialEq`), the system shall not write the file and shall not emit a `ChangeEvent`.

R12. WHEN the working tree is written to the file, the system shall place each setting's `to_file_value()` output under its dotted `toml_path` hierarchy.

R13. WHEN `set::<S>(v)` is followed by a fresh `load` of the same path with `S` registered, the system shall resolve `get::<S>()` to a value equal to `v` (write/read round-trip), subject to the per-type precision contract (R16 for `Duration`).

R14. WHEN `clear::<S>()` is called, the system shall remove the setting's node from the working tree (so `get::<S>()` resolves to `S::default_value()`), persist the working tree to the file, and emit a `ChangeEvent { reason: ChangeReason::Cleared }`.

R15. WHEN `reload()` is called, the system shall re-read the file into a fresh working tree, emit a `ChangeEvent { reason: ChangeReason::Reloaded }` for each registered setting whose resolved value changed, and shall not write back to the file during the reload.

R16. WHEN a `Duration` setting is written to the file, the system shall serialize it as an integer number of **whole seconds** (distinct from serde's `{ secs, nanos }` form), truncating any sub-second component; consequently the file representation carries whole-second precision only, and the R13 round-trip equality holds for `Duration` values that are an exact whole number of seconds (the R13 `Duration` fixture is whole-second).

R17. WHEN `subscribe(f)` returns a `Subscription` and that `Subscription` is dropped, the system shall stop invoking `f` for subsequent change events.

R18. The system shall report `is_syncable::<S>()` as `false` for every setting, and `default_values()` shall return a `toml::Value` mapping every registered setting's `toml_path` to its `to_file_value()` default.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `Setting` trait bounds `Value: SettingsValue + PartialEq + Debug + Clone`; `default_value()` exposed (R1) | planned |
| 2 | Non-private setting without (or with empty) `toml_path` fails to compile (R2) | planned |
| 3 | `toml_path_storage_key`/`toml_path_hierarchy` split a dotted path; both are non-`const` (R3) | planned |
| 4 | Registration indexes by storage key; duplicate key panics (R4) | planned |
| 5 | Missing file → empty working tree → all defaults, no file created (R5) | planned |
| 6 | After a valid load, `get::<S>()` resolves lazily from the `toml_path` node (R6) | planned |
| 7 | Invalid TOML → `Err(Parse)`, no panic (R7) | planned |
| 8 | Absent `toml_path` node → default value (R8) | planned |
| 9 | Type-mismatched node → default; no working-tree or cross-setting mutation (R9) | planned |
| 10 | Changed `set` writes node + persists tree + emits `Set` event (R10) | planned |
| 11 | No-op `set` skips write and event (semantic diff) (R11) | planned |
| 12 | File write nests each value under its `toml_path` hierarchy (R12) | planned |
| 13 | `set` then re-`load` round-trips to an equal value (R13) | planned |
| 14 | `clear` removes node (→ default), persists, emits `Cleared` (R14) | planned |
| 15 | `reload` re-reads, emits `Reloaded` per changed setting, no write-back (R15) | planned |
| 16 | `Duration` writes as integer whole seconds (≠ serde shape); whole-second precision contract holds (R16) | planned |
| 17 | Dropping a `Subscription` stops callbacks (R17) | planned |
| 18 | `is_syncable` always `false`; `default_values()` maps all defaults (R18) | planned |

## Visual / Behavioral Acceptance
N/A — non-UI persistence/plumbing crate; no window, pane, or AXUIElement surface. The settings UI that renders these groups is a separate downstream spec, and the visual/AX harness ([../pipeline/visual-testing.spec.md](../pipeline/visual-testing.spec.md)) is exercised there, not here.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement, each over a `tempfile`-backed settings path:
  - `r1_setting_exposes_default`, `r3_toml_path_split_storage_key_and_hierarchy`, `r4_register_indexes_by_key` (+ `#[should_panic]` `r4_duplicate_storage_key_panics`), `r5_missing_file_yields_defaults_no_create`, `r6_get_resolves_from_toml_path_after_load`, `r7_invalid_toml_returns_parse_err`, `r8_absent_node_yields_default`, `r9_type_mismatch_yields_default_no_mutation`, `r10_changed_set_writes_and_emits`, `r11_noop_set_skips_write_and_event`, `r12_write_nests_under_hierarchy`, `r13_set_then_load_roundtrips`, `r14_clear_removes_node_and_emits`, `r15_reload_rereads_emits_per_changed_no_writeback`, `r16_duration_writes_whole_seconds`, `r17_dropped_subscription_stops_callbacks`, `r18_is_syncable_false_and_default_values_map`.
  - R2 is a `trybuild` compile-fail case (`tests/ui/missing_toml_path_fail.rs` + `.stderr`) asserting the `const`-assert build error.
  - R6/R8/R9 prove lazy resolution explicitly: a setting is registered **after** `load`, and `get::<S>()` still resolves the loaded node (R6) / default for an absent node (R8) / default for a type-mismatched node (R9) — demonstrating that load order and register order are independent. R9 additionally re-reads a sibling setting before and after the mismatched `get` and asserts it is unchanged.
  - R11/R15 use a counter-backed subscriber to assert the exact event count (0 for the no-op set; once per changed setting on reload).
  - R15 "no write-back" is asserted by capturing the file's mtime + bytes before/after `reload` and proving them unchanged.
  - R16 asserts the on-disk node is a bare integer of whole seconds (not a `{ secs, nanos }` table) and that a sub-second `Duration` is truncated to its whole-second floor on write.
  - 100% line coverage on every touched line of `lib.rs`, `value.rs`, `manager.rs`, `macros.rs`.
- **Integration:** a cross-module seam test registers two groups (one with a whole-second `Duration` and an enum setting), writes via `set`, drops the manager, reloads from the same path with the same registrations, and asserts both values survive — exercising the macro → registry → file → lazy-resolve path end to end.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite plus the `trybuild` snapshot must stay green; the TOML on-disk shape (nesting + `Duration`-as-whole-seconds) is pinned by R12/R16 golden assertions so a format drift fails.

## Mutation Targets
`cargo mutants` must kill every viable mutant across `value.rs`, `manager.rs`, and the path helpers:
- semantic-diff guard in `set` (deleting the equality check, swapping `==`↔`!=`) — killed by R10/R11.
- the lazy default-fallback branch in `get` (returning the default when the node IS present and decodable, or returning a decoded node as default when it is absent) — killed by R6/R8/R9.
- `toml_path` split off-by-one (dropping the last segment, including `storage_key` in `hierarchy`) — killed by R3.
- write-back suppression during `reload` (removing the inhibit, writing anyway) — killed by R15.
- node-removal in `clear` (skipping the removal, removing the wrong node) — killed by R14.
- the duplicate-key guard in `register` (removing the panic) — killed by R4's `#[should_panic]`.
- `Duration` file encoding (whole-seconds↔nanos swap, truncation-vs-round, `*1000` ms-factor mutants) — killed by R16.
- `ChangeReason` variant selection (`Set`↔`Cleared`↔`Reloaded`) — killed by R10/R14/R15.
- MSI target: **100%** on the testable surface. No ACCEPTED-UNTESTABLE lines anticipated; there is no `unsafe`, so the miri gate is N/A for this crate.

## Dependencies
- REUSE (permissive, MIT/Apache): `toml` (file parse/serialize + `toml::Value` tree), `serde` (`Serialize`/`DeserializeOwned` bounds and derive on setting value types).
- Marley components: none required at M1. The future settings **UI** panel and the optional `SettingsValue` derive macro (a sibling proc-macro crate) are downstream consumers, not dependencies of this spec.

## Out of scope / deferred
- **Cloud sync / per-platform sync routing** (a syncable-vs-local routing backend with a remote store and a "clear remote state, fall back to local" round-trip) — permanently out of scope per the Marley charter; this build is local TOML only and reports every setting non-syncable (R18).
- **Secure storage** (a secret-setting kind backed by the OS keychain) — deferred; secrets are not a settings concern at M1.
- **JSON-Schema generation** via a link-time collector — deferred to a later milestone; M1 exposes only `default_values()` for file scaffolding, not a published `$schema`.
- **`#[derive(SettingsValue)]`** proc-macro (the snake_case/format-overriding derive) — deferred to a sibling `marley_settings_value_derive` spec; M1 uses the defaulted serde-passthrough impls plus the hand-written `Duration` impl.
- **File-watcher / live disk monitoring** — `reload()` is caller-driven at M1; an OS file-watch trigger is a later spec.
- **gpui entity/model change-event routing** — M1 delivers changes through the in-crate `subscribe`/`Subscription` callback; wiring into the gpui entity system is deferred to app integration.

## Clean-room provenance
Behavior-derived from a fork-reference doc (observable I/O only — no AGPL/fork source read, no private module/type/static names, no fork file paths): a typed declarative settings framework that persists feature-declared preference groups to one human-editable TOML file with per-setting defaults, caller-driven hot-reload, and change notifications. Public-surface identifiers are Marley-original. REUSE crates (`toml`, `serde`) are MIT/Apache. The package is named `marley_settings`. The cloud-sync, secure-storage, and link-time-schema behaviors are intentionally not reproduced. IP-counsel sign-off on the behavioral wall is pending (open item in `clean-build-plan.md`).
