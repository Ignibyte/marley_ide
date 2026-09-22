# marley_settings — the typed declarative settings framework (M1.B)

**Status:** M1 · TICKET-021 · forge #21 · sprint M1.B "The Cockpit" seq 5/5 (THE FINALE). Contract:
[`SPEC-settings.spec.md`](../specs/SPEC-settings.spec.md) (R1–R18). `visual_acceptance: N/A`. **Current
@ M15:** the framework crate is **unchanged since M1.B** (its API landed complete) but is now wired and
widely consumed (see [Consumers](#consumers)). Clean-room `[Marley-original]`, gpui-free (see
[Provenance](#provenance-clean-room)).

## Purpose

`marley_settings` lets each Marley feature declare its user preferences as **strongly-typed Rust values**
and get TOML-file persistence, per-setting defaults, lazy resolution, caller-driven reload, and change
notifications — with no per-setting boilerplate. Per the Marley charter this build is **local TOML only**:
no cloud sync, no secure-storage backend; every setting is non-syncable.

## Shape

- **`SettingsValue`** (`value.rs`) — the file-format trait, defaulted over serde (`to_file_value` via
  `toml::Value::try_from`, `from_file_value` via `try_into().ok()`); a blanket impl makes every
  `Serialize + DeserializeOwned` type a `SettingsValue`.
- **`Setting`** (`lib.rs`) — a feature's typed preference: an associated `Value: SettingsValue + PartialEq
  + Debug + Clone`, plus `default_value` / `toml_path` / `is_private` (+ the provided `storage_key` /
  `hierarchy`). `toml_path_storage_key` / `toml_path_hierarchy` split a dotted `toml_path` into its leaf +
  ordered parent segments.
- **`SettingsManager`** (`manager.rs`) — the runtime registry over a **retained `toml::Table` working
  tree**. Resolution is **lazy**: `get::<S>()` decodes `S`'s value from the tree node at its `toml_path` on
  demand (registration/load order independent), falling back to `default_value`. `set`/`clear` mutate the
  tree + persist; `reload` re-reads without write-back, emitting a `ChangeEvent` per setting whose resolved
  value changed — diffed via a **type-erased resolve closure** captured at `register` (each closure is
  monomorphised over `S`, so the manager compares canonical `toml::Value`s without the type). `subscribe`
  returns a `Subscription` whose drop stops its callback.
- **Macros** (`macros.rs`) — `define_setting!` / `define_settings_group!`; a `const { assert!(...) }`
  requires a non-private setting to declare a non-empty `toml_path` (R2 — a `trybuild` compile-fail case).
- **`SecondsDuration`** — a `Duration` persisted as whole seconds (R16), distinct from serde's
  `{ secs, nanos }` form.
- **`SettingsError { Parse, Io, Serialize }`** (+ `Display`/`Error`/`From`).

## The seam — whole-crate PURE

Unlike the app / harness crates, `marley_settings` has **no gpui/OS shim**: `load`/`set`/`clear`/`reload`
take + persist a `PathBuf`, tested against a `tempfile::tempdir()` path. So the **entire crate is the
testable surface — cov 100 / MSI 100, with no `rust_cov` exclude and no `mutants::skip`.** The only runtime
panic is `register`'s duplicate-key `assert!` (R4, a `#[should_panic]` test); the only compile-time panic
is the `define_setting!` `const` assert (R2). (Persist's `toml::to_string` never errors on a well-formed
tree, so its `Serialize` error is a coverage *region*, not a *line* — the line-coverage gate is satisfied;
the `Serialize` variant is exercised directly in a `SettingsError` unit test.)

## Consumers

The M1.B ticket shipped the framework with **zero consumers**; it has since been wired end to end and is
now the app's one persistence backbone. `marley_app` owns a pure `settings` module (its own cov 100 / MSI
100 seam) that declares the schema via `define_setting!` and loads a `SettingsManager` from
`<config-dir>/settings.toml` at boot — a missing/invalid file boots defaults and is **never clobbered**;
config-dir isolation keeps tests off the live `~/.marley`.

- **TICKET-026 (M1.C "The Wired Cockpit", THE CLOSER)** — the first consumer: `appearance.theme` /
  `docks.left` / `docks.right` / `terminal.cols` / `terminal.rows`. `RootView` applies the saved theme +
  dock states + terminal size at boot (an unknown theme name validated back to the Dark default) and
  persists theme + dock changes (a persist failure never crashes the terminal).
- **Layered on the same manager since** — the app schema (`crates/marley_app/src/settings.rs`) has grown
  from those five to **~18 `define_setting!`s**, each a typed value at a dotted `toml_path` exactly as the
  framework intends: the live theme picker (#199, persists the pick so you no longer hand-edit the TOML),
  command-palette **Workflows** (#204 — a `Vec<Workflow>` round-trip), **remote hosts** (#87 —
  `Vec<RemoteHost>`), the right-dock section (#95), the read-only code-panel tab width (#106), the Files
  panel width/open + window geometry (#168 / #176), and the launcher's recent roots (#234) +
  collapsed-project rail state (#245). Even session/window **layout** rides this framework — the serialized
  pane grid (`workspace.grid`, M6 #122) and workspace shell (`workspace.shell`, M10 #163) persist as plain
  string-blob settings (the #205 cwd / #258 split-file grid codecs live *inside* that blob).

## Provenance (clean-room)

`[Marley-original]`, and unusually **gpui-free** — the crate depends only on `serde` + `toml` (no `gpui`,
no OS shim), which is why the *whole* crate is the pure testable surface (cov 100 / MSI 100). It is a
from-scratch rebuild of Warp's `settings` / `settings_value` / `settings_value_derive` trio; the
`SettingsValue` blanket impl over `Serialize + DeserializeOwned` obviates the entire `settings_value_derive`
proc-macro, and per the Marley charter it is **local TOML only** — none of Warp's cloud-sync /
`SupportedPlatforms` / secure-storage machinery. The Warp platform-settings re-review tags it
`[Marley-original]` — "Rebuilt as `marley_settings` (M1.B). Done." (see
[`warp_architecture/subsystems/06-platform-settings-infra.md`](../warp_architecture/subsystems/06-platform-settings-infra.md)
§ "Built — Marley-original reimplementations"). Carries **no** Warp copyleft.

## See also

- [SPEC-settings.spec.md](../specs/SPEC-settings.spec.md) — the R1–R18 contract.
- [crate-map.md](crate-map.md) — the Marley crate lineage.
- [`warp_architecture/subsystems/06-platform-settings-infra.md`](../warp_architecture/subsystems/06-platform-settings-infra.md)
  — the Warp settings reference; confirms `marley_settings` is a clean-room rebuild.
