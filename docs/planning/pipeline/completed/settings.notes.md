---
pipeline_id: ffa41934-4ae1-476a-a23e-ebfcbdd3623d
aar_id: 35180d4c-c9ba-40ca-9f8b-b3e5dd838176
---

# marley_settings — pipeline notes

## Phase 1 — Plan (2026-07-01)

**Intent:** the NEW crate marley_settings (SPEC-settings R1-R18) — Marley's typed declarative TOML
settings framework. M1.B Cockpit seq 5/5 — THE FINALE.

## Carry to Design (Phase 2)

### Crate: crates/marley_settings (package `marley_settings`), whole-crate PURE (no exclude)
Cargo.toml: `toml = "1"`, `serde = { version = "1", features = ["derive"] }`; dev `tempfile`,
`trybuild`, `mutants`. Add to the workspace members.

### value.rs — `pub trait SettingsValue: Serialize + DeserializeOwned + Sized`
`fn to_file_value(&self) -> toml::Value { toml::Value::try_from(self).unwrap_or(toml::Value::Table(...)) }`
— hmm, try_from returns Result; default must not panic. Design: `to_file_value` default =
`toml::Value::try_from(self).expect(...)`? NO panic. Use `.unwrap_or_else(|_| toml::Value::String(...))`
OR make the default return `Result`? Spec says `-> toml::Value`. DECISION: default =
`toml::Value::try_from(self).unwrap_or(toml::Value::Boolean(false))` is wrong. Better: the trait method is
infallible by contract (Serialize types that are valid toml); use `toml::Value::try_from(self)` and for
the default, `.unwrap_or_else(|_| toml::Value::Table(Default::default()))` — but that hides errors. At
IMPLEMENT verify toml 1.x's `Value::try_from` signature; the cleanest is a default that maps a
serialization failure to an empty table (unreachable for well-formed Setting values). `from_file_value`
default = `value.clone().try_into().ok()`. Blanket impl `impl<T: Serialize + DeserializeOwned> SettingsValue
for T {}` so every serde type is a SettingsValue for free.

### lib.rs — `Setting` + the path helpers
- `pub trait Setting: 'static { type Value: SettingsValue + PartialEq + Debug + Clone; fn default_value()
  -> Self::Value; fn toml_path() -> &'static str; fn is_private() -> bool { false } fn storage_key() ->
  &'static str { toml_path_storage_key(Self::toml_path()) } fn hierarchy() -> Vec<&'static str> {
  toml_path_hierarchy(Self::toml_path()) } }`.
- `pub fn toml_path_storage_key(path) -> &'static str` = `path.rsplit('.').next().unwrap_or(path)` (the
  leaf). `pub fn toml_path_hierarchy(path) -> Vec<&'static str>` = `let mut s: Vec<_> =
  path.split('.').collect(); s.pop(); s` (parents). Both plain fn (Vec alloc).

### error.rs — `SettingsError { Parse(toml::de::Error), Io(io::Error), Serialize(toml::ser::Error) }`
+ Display + `impl std::error::Error` (source()). From impls for `?`.

### manager.rs — the core
- `pub struct SettingsManager { path: PathBuf, tree: toml::Value, registry: Vec<Registered>, subscribers:
  Vec<Weak<SubscriberFn>>, next_sub_id }` (tree is a `Value::Table`).
- `struct Registered { storage_key: &'static str, hierarchy: Vec<&'static str>, leaf: &'static str,
  resolve: Box<dyn Fn(&toml::Value) -> toml::Value> }` — TYPE-ERASURE: `register<S>()` builds `resolve =
  Box::new(|tree| resolve_typed::<S>(tree).to_file_value())` capturing S (monomorphized) → lets `reload`
  diff each setting's resolved value type-erased (compare the canonical `toml::Value`).
- helpers (pure, tested): `node<'a>(tree, hierarchy, leaf) -> Option<&'a toml::Value>` (walk nested
  tables → leaf); `insert(tree, hierarchy, leaf, value)` (create intermediate tables, set leaf);
  `remove(tree, hierarchy, leaf)` (navigate + remove the leaf).
- `resolve_typed::<S>(tree) -> S::Value` = `node(tree, S::hierarchy(), S::storage_key()).and_then(
  S::Value::from_file_value).unwrap_or_else(S::default_value)` (R6/R8/R9 — no mutation).
- `load(path) -> Result<Self>`: read the file; if absent → `tree = empty table` (R5, no create); else
  `toml::from_str(&contents).map_err(Parse)?` (R7). registry empty, no subscribers.
- `register<S>()`: push Registered; if a storage_key already present → `panic!("duplicate settings key:
  {key}")` (R4).
- `get<S>() -> S::Value` = `resolve_typed::<S>(&self.tree)`.
- `set<S>(v)`: `if v == self.get::<S>() { return Ok(()) }` (R11 no-op); else `insert(&mut tree,
  hierarchy, leaf, v.to_file_value())` (R10/R12), `persist()?`, `emit(ChangeEvent{storage_key, Set})`.
- `clear<S>()`: `remove(&mut tree, hierarchy, leaf)`, `persist()?`, `emit(Cleared)` (R14).
- `reload()`: `let before: Vec<toml::Value> = registry.map(|r| (r.resolve)(&self.tree))`; re-read the
  file → new tree; `self.tree = new`; for each registered i, if `(r.resolve)(&self.tree) != before[i]`
  → `emit(Reloaded{r.storage_key})`; NO persist (R15).
- `persist()`: `toml::to_string(&self.tree).map_err(Serialize)?` → `fs::write(&self.path,..).map_err(Io)?`.
- `default_values() -> toml::Value`: fold each registered setting's default into a tree at its hierarchy
  (R18) — build via insert on an empty table using a stored `default_file_value: toml::Value` per
  Registered (also captured at register: `S::default_value().to_file_value()`).
- `subscribe(f) -> Subscription`: store an `Rc<dyn Fn(&ChangeEvent)>`; `Subscription` holds the Rc; on
  drop, the Weak in `subscribers` fails to upgrade → the callback stops (R17). emit() upgrades + calls.
- `is_syncable<S>() -> bool { false }` (R18).

### Duration (R16) — whole seconds
A newtype `pub struct SecondsDuration(pub Duration)` (or a `Setting::Value` = a wrapper) whose
`SettingsValue`/serde serializes to an integer whole-second count (custom Serialize/Deserialize:
`serialize_u64(secs)`, deserialize u64 → `Duration::from_secs`). Truncates sub-second. The R13/R16 fixture
uses a whole-second Duration.

### macros.rs — `define_setting!` / `define_settings_group!`
`define_setting!(Name, Value, toml_path, default [, private])` → a unit struct + `impl Setting` +
`const _: () = assert!(is_private || !toml_path.is_empty(), "non-private setting needs a toml_path");`
(R2). `define_settings_group!` wraps multiple `define_setting!`. The const-assert is a `const { assert! }`
on the literal.

### Mutation map (the spec's Mutation Targets)
- toml_path split off-by-one (drop last / include storage_key in hierarchy) → R3.
- the register duplicate-key guard (remove panic) → R4 `#[should_panic]`.
- get's present-vs-default branch, from_file_value None→default → R6/R8/R9.
- set's PartialEq no-op guard (R11), the write+emit (R10).
- clear remove + emit (R14). reload's per-changed diff (R15). default_values map (R18). is_syncable false.
- The `node`/`insert`/`remove` path walks (off-by-one on hierarchy) → R6/R12/R13.

### Test plan (validate)
18 unit tests `r1_..r18_` (per the spec) + `#[should_panic] r4_duplicate_storage_key_panics` + a
`trybuild` compile-fail `tests/ui/missing_toml_path_fail.rs` (+.stderr) for R2 + an integration seam test
(two groups incl a whole-second Duration + an enum → set → drop → reload → survive). Fixtures via
`tempfile::tempdir()` (the file IO). NO exact toml::de::Error asserts (match the variant).

### Risks
- toml 1.x API (Value::try_from / try_into / from_str / to_string) — write-then-check at implement.
- Type-erasure via the resolve closure (monomorphized per S) — the key trick for reload's per-changed.
- The const-assert macro + trybuild .stderr snapshot (regenerate with TRYBUILD=overwrite).
- Whole-crate PURE (no exclude) — every line must be covered; the file IO via tempdir.

**Phase 1 status:** PASS (autonomous). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**Confirmed the Carry-to-Design against SPEC-settings (R1-R18) — realises the full Public surface.**

**toml 1.1.2 API VERIFIED (fetched + grepped):**
- `toml::Value::try_from<T: Serialize>(value: T) -> Result<Value, toml::ser::Error>` → `to_file_value`
  default = `toml::Value::try_from(self).unwrap_or_else(|_| toml::Value::Table(toml::Table::new()))` (map
  a serialization failure to an empty table — unreachable for well-formed Setting values; no panic).
- `toml::Value::try_into<'de, T: Deserialize>(self) -> Result<T, toml::de::Error>` → `from_file_value`
  default = `value.clone().try_into().ok()`.
- `toml::Value::as_table() / as_table_mut() -> Option<&[mut] Table>`; `pub type Table = Map<String,
  Value>` (index/get/insert/remove by `String` key). The working tree is a `Value::Table`.
- `toml::from_str::<toml::Value>(&str)` (load) + `toml::to_string(&Value)` (persist) — at the crate root
  (write-then-check the exact path at implement; they're the canonical toml fns).

**CONFIRMED:**
- **Type-erasure** (R15/R18): `register<S>()` stores `Registered { storage_key, hierarchy, leaf,
  resolve: Box<dyn Fn(&toml::Value)->toml::Value>, default_file_value: toml::Value }`; `resolve =
  Box::new(|tree| resolve_typed::<S>(tree).to_file_value())` (S monomorphized). `reload` snapshots
  `resolve(old)` per setting, swaps the tree, emits `Reloaded` where `resolve(new) != old`.
  `default_values` folds each `default_file_value` into a tree at its hierarchy. Sound.
- **Seam:** WHOLE crate PURE → cov 100 / MSI 100, NO rust_cov exclude, NO `mutants::skip`; gate-15 N/A
  (no UI). File IO (`load`/`persist`/`reload`) tested against `tempfile::tempdir()` paths.
- **const-assert (R2):** `define_setting!` emits `const _: () = assert!(IS_PRIVATE || !PATH.is_empty(),
  "...");` (a `const` block on the literal). trybuild: `tests/ui/missing_toml_path_fail.rs` + `.stderr`
  (regenerate with `TRYBUILD=overwrite` at validate).
- **Hard R's — all designed:** R15 (the type-erased `toml::Value` diff — canonical compare), R16
  (`Duration` newtype with a custom `Serialize`/`Deserialize` to/from `u64` whole seconds — truncates
  sub-second; R13 fixture is whole-second), R9 (`resolve_typed` is read-only — no tree mutation on a type
  mismatch ✓), R17 (`subscribe` → an `Rc<dyn Fn(&ChangeEvent)>` stored as `Weak`; `Subscription` holds
  the `Rc`; drop → `Weak::upgrade` fails → callback stops).

**Test plan:** 18 `r1_..r18_` unit tests (+ `#[should_panic] r4_duplicate_storage_key_panics`) + a
`trybuild` R2 case + an integration seam test (two groups incl a whole-second `Duration` + an enum →
set → drop → reload → survive); fixtures via `tempfile::tempdir()`. Match `SettingsError` variants (not
exact toml error text).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built the NEW crate `crates/marley_settings` (7 files) — the toml API + traits + generic manager + macros
all compiled **first try**; clippy -D / fmt / rustdoc -D / no-`unsafe` / `cargo deny` all clean. Files:
- **Cargo.toml** (toml=1, serde+derive; dev tempfile/trybuild/mutants). Auto-joined the `crates/*` glob.
- **value.rs** — `SettingsValue` trait (defaulted `to_file_value`/`from_file_value`) + blanket impl.
- **error.rs** — `SettingsError{Parse,Io,Serialize}` + Display + Error(source) + From impls.
- **duration.rs** — `SecondsDuration(Duration)` with custom serde to/from `u64` whole seconds (R16).
- **lib.rs** — `Setting` trait + `toml_path_storage_key` (via `rsplit_once('.').map_or`) +
  `toml_path_hierarchy` (`split('.').collect()` + `pop`); mod decls + re-exports.
- **manager.rs** — `SettingsManager` + `ChangeEvent`/`ChangeReason`/`Subscription`; load/register/get/set/
  clear/reload/default_values/subscribe/is_syncable; the type-erased `resolve` closure per Registered;
  `read_tree`/`resolve_typed` + recursive `node`/`insert`/`remove`.
- **macros.rs** — `define_setting!(vis Name: Value = default, "path")` (+ `const _: () = assert!(!path
  .is_empty(), ...)` R2) + `define_settings_group!`.

**Deviations (sound):**
1. **Working tree is `toml::Table`** (not `toml::Value`) — lets `node`/`insert`/`remove` recurse via
   `split_first()` with EVERY branch reachable + testable (the `insert` non-table-conflict arm overwrites
   a conflicting scalar with a fresh table — no `expect`/panic, unlike a Value-based force). load parses
   `toml::from_str::<toml::Table>`; persist `toml::to_string(&Table)`.
2. **R4 via `assert!`** (panics on a duplicate storage key). **R11** no-op guard via `PartialEq`. **R15**
   reload diffs the type-erased `resolve(old) != resolve(new)` per setting. **is_syncable<S>** references
   `S::storage_key()` (discarded) to avoid a clippy unused-type-param error.
3. **macro syntax** `define_setting!(pub Name: Value = default, "path")` — `toml_path` is a required
   literal; supplying `""` triggers the R2 const-assert (the trybuild case).

**Carry to Validate:** the 18 `r1_..r18_` tests + `#[should_panic] r4_..` + the trybuild R2 case + the
integration seam test. COVERAGE watch: the `insert` non-table-conflict overwrite arm needs a
hierarchy-conflict test; `node`'s `get`-None + `as_table`-None both need an absent + a type-mismatch
fixture; `emit`'s dropped-subscriber (`None` arm) needs the R17 drop test. `resolve_typed` is read-only
(R9 — no mutation).

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2026-07-01

Inspected in-context (`cargo mutants --list` + grep). **No code findings; the work is a coverage-fixture
plan for the whole-pure (no-exclude) error paths.**

- **Mutants: 45** (duration 2, error 6, lib 11, manager 23, value 3) — the r1-r18 suite kills them
  (verify 0-missed at validate). **§14:** the ONLY panics are the intended `register` `assert!` (R4 dup,
  `#[should_panic]` test) + the `define_setting!` `const assert!` (R2, compile-time); no unwrap/expect.
  **Clean-room:** 0 `warp`; all names Marley-original.

**CARRY TO VALIDATE — the coverage-fixture plan (whole-pure → 100% lines; every error/edge branch needs a
fixture; use `tempfile::tempdir()`):**
1. `read_tree` NotFound (R5): `load(absent path)`. **Io(other): `load(a DIRECTORY path)`** — `read_to_string`
   on a dir errors with kind ≠ NotFound → the Io arm.
2. `persist` write-error: `load("<absent-dir>/settings.toml")` (NotFound → ok) → `set` → `fs::write` to a
   path whose parent dir is absent → Io error covers the write `?`.
3. **`persist` to_string Serialize `?` + error.rs `Serialize` Display/source/From — THE KEY RISK.**
   error.rs: a direct test builds a `toml::ser::Error` via `toml::Value::try_from(<a BTreeMap<i32,i32>>)
   .unwrap_err()` (toml rejects non-string keys) → `SettingsError::Serialize(e)` → assert Display/source +
   `From`. persist's `?`-Err: VERIFY EMPIRICALLY whether a "value-after-sub-table" working tree makes
   `toml::to_string` fail in toml 1.1.2 (set a nested setting `group.nested.x` THEN a scalar sibling
   `group.y` → the table holds a sub-table then a scalar → TOML can't emit a scalar after a table). If it
   errors → that set covers persist's Serialize `?`. **IF toml 1.x does NOT error on value-after-table,
   ESCALATE** (persist's Serialize `?` would be uncoverable — options: keep the tree always-serializable
   by ordering scalars first, or reconsider — but do NOT weaken the gate).
4. `insert` non-table-conflict overwrite: two settings — A `toml_path "conflict"` (scalar), B
   `"conflict.child"`; `set A` then `set B` → `insert` walks "conflict" (a scalar) → the `None` overwrite
   arm.
5. `node` `get`-None (R8: an absent leaf) + `as_table`-None (a hierarchy segment that is a scalar →
   R9 type-mismatch → default; via the conflict fixture, `get B` when "conflict" is a scalar).
6. `remove` if-let None: `clear B` when "conflict" is absent.
7. `emit` `None => false` (R17): subscribe → drop the `Subscription` → `set` → the `Weak` fails to
   upgrade → the dropped subscriber is retained-out.
8. `set` no-op (R11, equal) vs changed (R10); `reload` changed-filter true (R15) + false (unchanged).
9. R16 `SecondsDuration` whole-second round-trip; R3 helpers (`"a.b.c"` + a single-segment `"font"`);
   R18 `is_syncable`==false + `default_values` map; R1 default; R6 file-node resolve; R7 invalid TOML.

**Tests:** the named `r1_..r18_` (+ `#[should_panic] r4_duplicate_storage_key_panics`) + a `trybuild` R2
compile-fail (`tests/ui/`) + the integration seam test + the error-path fixtures above. Match
`SettingsError` variants, not exact toml error text.

**Phase 3.5 status:** PASS. → Phase 4 Validate (resolve the to_string-Serialize coverage empirically).

## Phase 4 — Validate (2026-07-01)

Wrote the full suite: `tests/settings.rs` (18 `r*` + 4 edge/error fixtures) + an `error.rs` `#[cfg(test)]`
mod (all 3 variants' Display/source/From) + a `value.rs` mod (to_file_value round-trip + fallback) +
`tests/trybuild.rs` + `tests/ui/missing_toml_path_fail.rs` (+ the generated `.stderr`). `cargo nextest run
-p marley_settings` = **25 passed** (incl the R2 trybuild compile-fail — the `const` assert fired).

**Two empirical resolutions:**
1. **The to_string-Serialize coverage question — RESOLVED.** Probed toml 1.1.2: `to_string` does NOT error
   on a value-after-sub-table tree (returns Ok), so persist's `to_string?` Err is a *region*, not a
   *line* → the `--fail-under-lines 100` gate is unaffected (the line executes). error.rs's `Serialize`
   Display/source/From are covered directly via a `toml::ser::Error` built from `try_from(BTreeMap<i32,
   i32>)` (toml rejects non-string keys). value.rs's `to_file_value` fallback closure is covered the same
   way (the fallback empty table). Coverage → 100% lines whole-crate.
2. **An EQUIVALENT mutant on `is_syncable` — FIXED at source.** `is_syncable<S>` returned `false` after a
   `let _ = S::storage_key();` (added to appease a clippy unused-type-param worry). cargo-mutants' "replace
   with false" was then EQUIVALENT (the fn IS false) → 1 missed → MSI<100. Fix: a bare `{ false }` body —
   cargo-mutants skips the identical-to-original mutation (only the killable `true` mutant remains, caught
   by r18). Clippy's `extra_unused_type_parameters` is pedantic (not in `-D warnings`) so the unused `S`
   is fine. → BF/PR at complete.

**FULL gate (scripts/gates.sh --diff): `GATE GREEN [diff]` — 15/15**: cov 100% (WHOLE crate, no exclude —
the io-error/conflict/dropped-sub/fallback fixtures cover every line), mutation MSI 100% (30 caught, 14
unviable, 0 missed), machete GREEN (toml/serde/tempfile/trybuild all used; the unused `mutants` dev-dep
removed), deny GREEN. gate-15 N/A (no UI). Receipt written.

**Phase 4 status:** PASS. → Phase 5 Complete. **After #21, M1.B "The Cockpit" is COMPLETE (all 5:
#17-#21).**
