# settings wired — Notes

- **Forge ticket:** #26 `be140e90-3ddd-41e5-8110-4e76ca89112a`
- **AAR:** `c91e044c-40ea-4633-9f0c-06f963d41697`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-026-settings-wired.md
- **Pipeline spec:** settings-wired.spec.md

## Phase 1 — Plan
- **Request:** forge #26 (M1.C seq-5, THE CLOSER, auto-approved run) — wire marley_app to
  marley_settings; boot-load + persist theme/docks/terminal size.
- **Classification / tier:** work pipeline, `feature` — one slice (a new pure settings.rs + a
  paths.rs fn + the app.rs boot/persist shim + Cargo deps).
- **Forge recall (§18.3) + Explore map (the load-bearing facts):**
  - `SettingsManager::load(path: PathBuf)` — takes a full FILE path; NO dir/override/ambient
    constructor. `register::<S>()`, `get::<S>() -> S::Value`, `set::<S>(v)` (writes via private
    `persist` = `toml::to_string` + `fs::write`, does NOT create parent dirs), `reload`. Missing
    file → empty table, file NOT created; invalid TOML → `Err(Parse)`; a dir path → `Err(Io)`.
  - Value types: ANY serde type (blanket `impl<T: Serialize+DeserializeOwned> SettingsValue`).
    std types (String/u32/bool) need NO serde dep in marley_app → **D2 uses bool for docks**.
    `define_setting!(Name: Ty = default, "dotted.path")` (a const-assert bars an empty path);
    `define_settings_group!` EXISTS but is UNUSED repo-wide (untested) — prefer per-setting
    `define_setting!` (the exercised path).
  - `marley_core::paths`: NO runtime override — only a PRIVATE `home_dir_from(Option<PathBuf>)`
    seam its own tests use; the doc explicitly rejects `$HOME` env (races the threaded runner).
    `marley_config_dir() -> <home>/.marley/config`. NO settings-file helper yet → this ticket
    ADDS `marley_settings_file_path` (+ a `_from(base)` pure seam, the paths precedent).
  - marley_app depends on NEITHER marley_settings NOR marley_core today — both get added; it is
    marley_settings' FIRST consumer.
- **Decisions:** D1–D5 in the spec (dir-isolation via `*_in(dir)`; bool docks; theme-validate +
  parse-error → defaults; `applied_from` pure over a loaded manager; pane layout DEFERRED).
- **Open questions for Design:** load-or-default control flow on a Parse error (start-fresh manager
  vs skip-persist Option); persist granularity (per-setting set vs a batched save); whether
  `RootView` holds `SettingsManager` directly or an `Option`; the settings.rs `_in(dir)` fn set;
  the exact `AppliedSettings` → `SessionOptions`/`docks`/`set_theme` application order in boot.
- **AAR id:** `c91e044c-40ea-4633-9f0c-06f963d41697`.

## Phase 2 — Design

### Architecture / approach
NEW pure `crates/marley_app/src/settings.rs`:
```rust
use marley_settings::{define_setting, SettingsError, SettingsManager};
define_setting!(ThemeName: String = String::from("Marley Dark"), "appearance.theme");
define_setting!(DockLeft:  bool  = true, "docks.left");    // true = Open
define_setting!(DockRight: bool  = true, "docks.right");
define_setting!(TermCols:  u16   = 80,  "terminal.cols");  // matches SessionOptions.cols (u16)
define_setting!(TermRows:  u16   = 24,  "terminal.rows");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedSettings { pub theme_name: String, pub left: DockState, pub right: DockState,
                             pub cols: u16, pub rows: u16 }

fn dock_state(open: bool) -> DockState { if open { DockState::Open } else { DockState::Closed } }
pub fn settings_file_in(dir: &Path) -> PathBuf { dir.join("settings.toml") }
fn register_all(m: &mut SettingsManager) { m.register::<ThemeName>(); m.register::<DockLeft>(); … }
pub fn load_manager_in(dir: &Path) -> Result<SettingsManager, SettingsError>
// = create_dir_all(dir)? ; let mut m = SettingsManager::load(settings_file_in(dir))? ;
//   register_all(&mut m) ; Ok(m)   — io::Error → SettingsError::Io via From
pub fn applied_from(m: &SettingsManager, registry: &ThemeRegistry) -> AppliedSettings
// theme = registry.by_name(&m.get::<ThemeName>()).map(|t| t.name.clone())
//         .unwrap_or_else(|| registry.dark_default().name.clone())   // unknown/absent → dark
// left = dock_state(m.get::<DockLeft>()), right = dock_state(m.get::<DockRight>()),
// cols = m.get::<TermCols>(), rows = m.get::<TermRows>()
```
Purity: `applied_from` is PURE given a loaded manager (`get` reads the in-memory table — the file
was read at `load`). The `_in(dir)` IO (`load_manager_in`) is thin + tempdir-testable (§14).
Persistence is the manager's own `set::<S>()` (writes the whole file) — driven from the shim; the
round-trip test (set→drop→reload→applied) proves it without gpui.

SHIM (app.rs, existing exclude): `RootView` gains `settings: Option<SettingsManager>` + `term_cols`/
`term_rows` (u16, from applied, reused for splits). `new()` reorders: build the registry →
`marley_config_dir()` → `load_manager_in`: `Ok(m)` → `applied_from` → `(Some(m), applied)`;
`Err(_)` → `(None, defaults)` (a parse/io error boots defaults; NO persist this session — a corrupt
file is never clobbered). Then spawn the first session at applied cols/rows, set `self.theme =
registry.by_name(&applied.theme_name)` (guaranteed Some — validated), `self.docks = [applied.left,
applied.right]`. `spawn_session` gains `(cols, rows)` params. `toggle_theme_state` + `toggle_dock_state`
each `persist()` after flipping: `self.settings.as_mut().map(|m| m.set::<S>(value))` (a `let _ =`
on the Result — a persist failure must not crash the UI). `set_theme` (public R23) persists too.

### Decisions
- D-2.1 `settings: Option<SettingsManager>` — `None` after a load error → the persist helpers are
  no-ops (a corrupt file degrades to "defaults, no persistence this session", never clobbered/
  crashed). Not a dead branch: the invalid-TOML test exercises the Err path (returns the error;
  the shim's None mapping is the review'd shim line).
- D-2.2 Setting default = the REAL registry name `"Marley Dark"` (NOT "dark" — `by_name("dark")`
  is None; a real name makes the default self-validate). `applied_from` validates any saved name.
- D-2.3 Docks persist as `bool` (D2) — `dock_state(bool)->DockState` is the pure map (a mutation
  target); marley_app needs NO direct serde dep (std-typed settings).
- D-2.4 `TermCols`/`TermRows` are `u16` — matches `SessionOptions.{cols,rows}` with no cast.
- D-2.5 NO `marley_core` change — `settings_file_in` owns the filename (REVISED REQ-005). marley_app
  gains `marley_settings` + `marley_core` (for `marley_config_dir`) path deps.
- D-2.6 Pane-layout persistence DEFERRED (spec Out) — M2 session restore.

### File manifest
- A `crates/marley_app/src/settings.rs` — the schema + AppliedSettings + applied_from +
  dock_state + settings_file_in + load_manager_in + register_all + tests.
- M `crates/marley_app/src/app.rs` — the `settings`/`term_cols`/`term_rows` fields, `new()`
  boot-load + apply, `spawn_session(zdotdir, cols, rows)`, persist in toggle_theme/toggle_dock/
  set_theme.
- M `crates/marley_app/src/lib.rs` — `mod settings` + export `AppliedSettings`/`applied_from`/
  `load_manager_in`/`settings_file_in`.
- M `crates/marley_app/Cargo.toml` — add `marley_settings` + `marley_core` path deps.
- M `docs/specs/SPEC-app-shell.spec.md` — R33 (boot-apply) + R34 (persist-on-change) + AC/
  Test-Plan/Mutation-Targets rows. CHANGELOG; arch doc at complete.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `applied_from_reads_saved_values` (a tempdir manager with a settings.toml holding theme "Marley Light", left=false/right=true [ASYMMETRIC — kills the dock swap], cols=100/rows=30 [DISTINCT — kills rows↔cols swap] → the exact AppliedSettings); `applied_from_defaults_on_empty` (no file → "Marley Dark", both Open, 80×24 — each default); `applied_from_unknown_theme_falls_back_to_dark` (saved "Nonsense" → the dark default name) | unit (settings.rs) |
| REQ-002 | `load_manager_in_missing_file_yields_defaults_and_creates_no_file` (tempdir, assert applied defaults + settings.toml does NOT exist); `load_manager_in_invalid_toml_errors` (write garbage → `Err`); (dir auto-create covered by the tempdir subpath) | unit |
| REQ-002b | `settings_round_trip_survives_reload` (load → `set::<ThemeName>("Marley Light")` + `set::<DockLeft>(false)` → drop → `load_manager_in` again → `applied_from` = light + left Closed) | unit |
| REQ-003 | `applied_from`/`load_manager_in` coverage above; the boot apply order is the app.rs shim (review + deferred headed lane) | unit + review |
| REQ-004 | REQ-002b round-trip + shim review (toggle→set→persist) | unit + review |
| REQ-005 | `settings_file_in_joins_settings_toml` (`dir.join("settings.toml")`) | unit |
| REQ-006 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs boot/persist call sites (existing exclude); headed boot-theme baseline
rides the #23 deferral.

### Risks / decisions
- The dock-side swap + rows↔cols swap mutants need ASYMMETRIC + DISTINCT fixtures (the #24
  symmetric-fixture lesson) — pinned in REQ-001.
- `define_settings_group!` is untested repo-wide → use per-setting `define_setting!` (the
  exercised macro). Confirm the macro invocation compiles first-try via cargo check (write-first).
- Theme name default MUST equal a real registry name — verified against themes.rs at implement
  (a wrong name would make the default fall back to dark, still valid but confusing; the
  `applied_from_defaults_on_empty` test pins the exact expected name).
- A persist failure is swallowed (`let _`) — a settings write must never crash the terminal; the
  in-memory state already changed, only the file lags. Documented.

## Phase 3 — Implement
- **Built (per manifest):** settings.rs — the 5 `define_setting!`s (theme "Marley Dark",
  docks bool true, cols/rows u16 80/24), `AppliedSettings`, `dock_state(bool)`, `settings_file_in`,
  `register_all`, `load_manager_in` (create_dir_all + load + register), `applied_defaults` (via
  `Setting::default_value()` — one source per default), `applied_from` (theme-validate +
  fields), `persist_theme`/`persist_dock_left`/`persist_dock_right`; app.rs — the
  `settings: Option<SettingsManager>` + `term_cols`/`term_rows` fields, `new()` boot-load
  (`marley_config_dir` → `load_manager_in`: Ok → applied_from; Err → applied_defaults + None),
  theme/docks/size applied into `Self`, `spawn_session(zdotdir, cols, rows)` (split reuses
  term size), persist in `toggle_theme_state`/`toggle_dock_state`/`set_theme` (all `let _`
  swallowed); Cargo.toml gains `marley_settings` + `marley_core`; lib.rs exports. SPEC R33/R34 +
  AC 33/34 + Test-Plan + Mutation-Targets; CHANGELOG.
- **Deviations from design:** (D-3.1) added `applied_defaults` as a NAMED pure fn (the design
  sketched it inline) drawing from `Setting::default_value()` — one source per default, directly
  testable (the Err-branch boot value); (D-3.2) three `persist_*` fns instead of one (per-setting
  set — clearer call sites, each a thin manager pass-through). Both additive, no scope change.
- **Verification at this phase:** workspace check 0 errors; clippy `-D warnings` clean; 58 lib
  tests pass. The R33/R34 unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critics run:** 2 — (A) correctness/state-integrity (16/16 real-disk probes: the round-trip,
  the validation matrix, IO edges incl. wrong-type/overflow/corrupt-file graceful degradation,
  the never-clobber-a-foreign-key property); (B) provenance/simplification/dependency-hygiene.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED (MSI risk) | `register_all` is OBSERVATIONALLY INERT for get/set-only usage — verified against manager.rs: `get`/`set` resolve off `self.tree` via the compile-time `toml_path`, never `self.registry` (which only `reload`/`default_values`/`subscribe` read, none called). So a mutant deleting the `register_all(...)` call in `load_manager_in` SURVIVES → MSI < 100. | REAL | DROPPED `register_all` + its call at the SOURCE (no dead code, no `mutants::skip`) — `load_manager_in` = `create_dir_all(dir)?; SettingsManager::load(file)`. Compiles clean, `Setting` still used by `applied_defaults`. |
  | F2 | MED (MSI risk) | The R34 round-trip sets "a dock" (singular) → the OTHER `persist_dock_*` fn's `Ok(())` body-mutant survives. | REAL (Phase 4 test-design) | SPEC R34 test note tightened to persist the theme + BOTH docks; the Phase 4 test will. |
  | F3 | LOW (MSI risk) | `create_dir_all(dir)?` delete-mutant survives unless a test drives a MISSING nested dir where a later `set` would fail without it (a `load` alone treats a missing file as empty either way). | REAL (Phase 4 test-design) | SPEC R34 test note: the round-trip runs over a NESTED missing config dir (set→persist would fail without create_dir_all). |
  | F4 | LOW | lib.rs exports `applied_from`/`load_manager_in`/`settings_file_in`/`AppliedSettings` but not `applied_defaults`. | ACCEPTED (correct) | The Phase 4 tests are in-crate unit tests (`super::*`); the exports are the frozen M2-facing surface; `applied_defaults` is app.rs-internal. No change. |
  | F5 | LOW | "one source per default"/"terminal size survive relaunch" CHANGELOG wording overstated (theme default has the literal + registry; terminal size is boot-read, no runtime resist). | REAL (wording) | CHANGELOG reworded: applied_defaults' NON-theme fields from `default_value()`; terminal size boot-applied (no runtime resize UI). |
  - Critic A forward-looking LOWs (not this diff): a future setting with a colliding leaf would
    panic `register` at boot (framework #21 — but we no longer call register, so N/A now);
    multi-window lost-update (run() opens ONE window). Noted for M2.
- **Rejected / accepted:** no correctness/state/provenance/cycle/hygiene defect (Critic A PASS,
  16/16; Critic B CLEAN on those lenses). marley_core adds no cycle (its deps: home/enum-iterator/
  once_cell/serde — no Marley crate). Both new deps used (machete-clean).
- **Post-fix verification:** fmt/check/clippy clean; 58 lib tests pass; the register_all drop
  removed the F1 mutant at source.

## Phase 4 — Validate
- **Tests added (settings.rs, 9):** `applied_from_reads_saved_values` (ASYMMETRIC docks + DISTINCT
  cols/rows — kills the swaps; valid-non-dark kept), `applied_from_defaults_on_empty`,
  `applied_from_unknown_theme_falls_back_to_dark`, `applied_defaults_are_the_schema_defaults`
  (+ == the empty-file value, no drift), `load_manager_in_missing_file_yields_defaults_and_creates_no_file`,
  `load_manager_in_invalid_toml_errors`, `settings_round_trip_survives_reload` (theme + BOTH docks
  over a NESTED missing dir — the create_dir_all + both-persist mutant kills, the F2/F3 inspect
  arms), `settings_file_in_joins_settings_toml`, `dock_state_maps_bool_to_state`.
- **Runs (actual):** `cargo nextest run --workspace` → all pass (70 in marley; full workspace
  green); doctests green.
- **Visual (gate:15):** PASS headless; no new UI (theme/dock apply rides existing baselines); the
  boot-theme headed baseline rides the #23 desktop-session deferral.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**; coverage 100% lines;
  mutation **4 caught / 0 missed → MSI 100.0%** (the register_all drop removed the F1 risk mutant;
  both-docks + nested-dir killed F2/F3); receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG entry (wording corrected at inspect); `app_shell.md` gains the
  `settings.rs` bullet (schema + applied_from + the `*_in(dir)` seam + boot/persist; marley_app =
  marley_settings' first consumer). SPEC-app-shell R33/R34 landed at implement/inspect.
- **AAR capture:** `PR-claude-inert-framework-call-is-an-unkillable-mutant-drop-or-justify-001`
  (the register_all inert-call → surviving-mutant lesson); aar-submit `completed`. Lesson: a
  framework setup call with NO observable effect on your access path is dead code, not diligence —
  read the accessor source (get/set read the tree, not the registry) and drop it, else MSI can't
  reach 100. A measuring critic that read manager.rs caught it.
- **Ticket:** forge #26 → done; local doc → closed/; pipeline pair archived. **This closes the
  M1.C feature set (#22–#26).** Only #27 (delete marley_spike, a chore) remains in the sprint.
