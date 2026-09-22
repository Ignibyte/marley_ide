---
pipeline_id: ffa41934-4ae1-476a-a23e-ebfcbdd3623d
ticket: forge#21 (a1bf519d-3b54-43a2-bfe2-de1d3cea1716) · local docs/planning/tickets/open/TICKET-021-settings.md
aar_id: 35180d4c-c9ba-40ca-9f8b-b3e5dd838176
sprint: M1.B — The Cockpit (cbc92bf0) seq 5/5 — THE FINALE
status: Phase 5 — Complete PASS
title: marley_settings — typed declarative TOML settings framework
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-settings.spec.md
---

## Title

TICKET-021 — the NEW crate `marley_settings` ([`SPEC-settings`](../../../specs/SPEC-settings.spec.md),
R1–R18): Marley's typed declarative settings framework — feature crates declare typed preference groups
and get TOML persistence, per-setting defaults, lazy resolution, caller-driven reload, and change events.
M1.B "The Cockpit" **FINALE** (seq 5/5).

## Scope

### In (a new crate `crates/marley_settings`, package `marley_settings`)
- **`value.rs`** — `pub trait SettingsValue: Serialize + DeserializeOwned + Sized` with defaulted
  `to_file_value(&self) -> toml::Value` (`toml::Value::try_from`) + `from_file_value(&toml::Value) ->
  Option<Self>` (`value.clone().try_into().ok()`).
- **`lib.rs`** — `pub trait Setting: 'static` (assoc `Value: SettingsValue + PartialEq + Debug + Clone`;
  `default_value()`, `toml_path()`, `is_private()` [default false]; provided `storage_key()`,
  `hierarchy()`); free `toml_path_storage_key(&'static str) -> &'static str` (leaf) +
  `toml_path_hierarchy(&'static str) -> Vec<&'static str>` (parent segments) — plain `fn` (NOT const).
- **`error.rs`** — `pub enum SettingsError { Parse(toml::de::Error), Io(std::io::Error),
  Serialize(toml::ser::Error) }` + `impl Display/Error`.
- **`manager.rs`** — `pub struct SettingsManager` holding a retained `toml::Value` working tree + a
  registry + subscribers + the `PathBuf`: `load(PathBuf) -> Result` (parse or empty table, R5/R7),
  `register<S>()` (index by storage_key; panic on dup, R4), `get<S>() -> S::Value` (lazy resolve, R6/R8/
  R9), `set<S>(v) -> Result` (semantic-diff write + persist + emit, R10/R11/R12/R13), `clear<S>()`
  (remove + persist + emit, R14), `reload()` (re-read + per-changed emit, no write-back, R15),
  `default_values() -> toml::Value` (R18), `subscribe(f) -> Subscription` (R17), `is_syncable<S>() ->
  bool` (always false, R18). `ChangeEvent { storage_key, reason }` + `ChangeReason { Set, Cleared,
  Reloaded }` + `Subscription` (drop stops callbacks, R17).
- **`macros.rs`** — `define_settings_group!` / `define_setting!` — `define_setting!` embeds a
  `const { assert!(!is_private → !toml_path.is_empty()) }` (R2, a trybuild compile-fail case).
- **`Duration` file form** — whole seconds (R16), distinct from serde's `{secs,nanos}`.
- §21 — CHANGELOG + a new `docs/marley_architecture/settings.md`.

### Out / deferred
- Wiring `marley_app` to load/persist settings at boot — a follow-up (this ticket delivers the CRATE).
  cloud-sync / secure-storage (charter: local TOML only).

## Acceptance Criteria (EARS — adopt SPEC-settings R1–R18)
Each Rn is verified by the named unit test (r1..r18) + R2 by a `trybuild` compile-fail + an integration
seam test (two groups, a whole-second `Duration` + an enum, write→drop→reload→survive). Highlights:
- **R3** — `toml_path_storage_key("a.b.c")=="c"`; `toml_path_hierarchy("a.b.c")==["a","b"]`.
- **R4** — `register` indexes by `storage_key`; a duplicate key **panics** (`#[should_panic]`).
- **R5** — `load(absent)` → empty tree, defaults, no file created.
- **R6/R8/R9** — `get` resolves from the file node via `from_file_value`, else the default; a type
  mismatch → default with **no** working-tree/cross-setting mutation.
- **R7** — invalid TOML → `Err(Parse)`, no panic.
- **R10/R11** — a changed `set` writes + persists + emits `Set`; a no-op `set` (PartialEq-equal) does
  neither.
- **R12/R13/R16** — writes nest under the dotted `toml_path`; `set`→`load` round-trips; `Duration`
  serializes as whole seconds.
- **R14/R15** — `clear` removes + persists + emits `Cleared`; `reload` re-reads + emits `Reloaded` per
  changed setting, no write-back.
- **R17** — a dropped `Subscription` stops callbacks.
- **R18** — `is_syncable` is always `false`; `default_values()` maps each `toml_path` → default.
- **AC-gate** — the WHOLE crate is PURE (no gpui, no OS beyond the `PathBuf` file IO tested via a
  tempdir) → cov 100 / MSI 100 whole-crate (NO rust_cov exclude, NO `mutants::skip`); `trybuild` snapshot
  green; FULL `scripts/gates.sh` → `GATE GREEN [diff]` (gate-15 N/A — no UI).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **Whole-crate PURE** — unlike the app/harness crates, marley_settings has NO gpui/OS shim: `load`/
   `set`/`clear`/`reload` take/persist a `PathBuf` (tested against a `tempfile::tempdir()` path). So the
   ENTIRE crate is the testable surface (cov 100 / MSI 100), no exclude.
2. **Lazy resolution** — the manager retains the parsed `toml::Value` working tree; `get` resolves a
   typed value on demand (registration/load order independent). `set`/`clear` mutate the tree + persist.
3. **`Duration` as whole seconds** (R16) — a `SettingsValue` impl (or a newtype) serializing to an integer
   second count, NOT serde's `{secs,nanos}`.
4. **deps** — `toml` (v1.1.2) + `serde` (derive) + dev `tempfile` + `trybuild` + `mutants`.

## Phase Plan
- **P2 Design** — the `SettingsManager` internals (the working-tree get/insert/remove at a dotted path,
  the registry, subscribers, the diff-before-write) + the macro expansions + the `Duration` newtype + the
  mutation map (the spec's Mutation Targets) + the r1–r18 + trybuild + integration test plan.
- **P3 Implement** — the crate (Cargo.toml + workspace member) + the 5 modules + the macros.
- **P3.5 Inspect** — `cargo mutants --list` (whole crate) + the working-tree path insert/remove edges.
- **P4 Validate** — the 18 unit tests + trybuild + the integration seam test + gate.
- **P5 Complete** — §21; close #21; **M1.B "The Cockpit" COMPLETE** → close the sprint.
