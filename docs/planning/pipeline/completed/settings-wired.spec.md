---
pipeline_id: c0a1f890-c9ae-4ce9-afa7-b225ce97d5d7
ticket: forge#26 (be140e90-3ddd-41e5-8110-4e76ca89112a) · local docs/planning/tickets/open/TICKET-026-settings-wired.md
aar_id: c91e044c-40ea-4633-9f0c-06f963d41697
status: Phase 5 — Complete PASS
title: settings wired — the marley settings schema + boot-load + persist-on-change
type: feature
milestone: M1.C
references:
  - docs/specs/SPEC-settings.spec.md (the shipped framework — SettingsManager/define_setting!/Setting)
  - docs/specs/SPEC-app-shell.spec.md (gains the boot-apply + persist-on-change clauses R33/R34)
  - crates/marley_settings/ (the framework #21 shipped — marley_app is its FIRST consumer)
  - crates/marley_core/src/paths.rs (marley_config_dir — the boot path source)
---

## Title
The M1.C closer: `marley_app` finally DEPENDS on `marley_settings`. Define Marley's first real
settings schema (theme / dock states / terminal size), LOAD it at boot to apply the saved theme +
dock states, and PERSIST on change — so theme + dock state survive a relaunch. That round-trip is
the sprint's proof the cockpit is actually wired, not just rendered.

## Scope
### In
- NEW pure `crates/marley_app/src/settings.rs`: the settings schema via `define_setting!`
  (`appearance.theme: String = "dark"`; `docks.left`/`docks.right: bool = true` [true = Open];
  `terminal.rows: u32 = 24` / `terminal.cols: u32 = 80`) + `AppliedSettings { theme_name,
  left_open, right_open, rows, cols }` + the PURE `applied_from(&SettingsManager, &ThemeRegistry)
  -> AppliedSettings` (reads each setting; VALIDATES the theme name via `by_name`, unknown →
  the Dark default's name) + the thin `*_in(dir)` IO seam: `settings_file_in(dir)` (join
  `settings.toml`), `load_manager_in(dir)` (create_dir_all + `SettingsManager::load` + register
  all), `persist_*` setters (set through the manager → it writes). A load Parse error →
  applied-DEFAULTS, never a boot failure.
- `crates/marley_app/src/app.rs` (shim): a `settings: SettingsManager` (or defaults) field;
  `RootView::new` loads from `marley_config_dir()` and applies theme + docks + terminal rows/cols;
  `set_theme`/`toggle_theme_state` + `toggle_dock`/`toggle_dock_state` persist through the manager.
- `crates/marley_app/Cargo.toml`: add `marley_settings` + `marley_core` path deps.
- **[REVISED at Design]** the `settings.toml` filename lives in `settings.rs` (`settings_file_in(dir)
  = dir.join("settings.toml")`, pure + tested) — NOT a new `marley_core` path fn: one source of
  truth for the filename, co-located with the loader that uses it; the shim composes
  `marley_config_dir()` (marley_core, unchanged) + `load_manager_in(config_dir)`.
- SPEC-app-shell: R33 (boot-apply) + R34 (persist-on-change) + AC/Test-Plan/Mutation-Targets;
  SPEC-settings unchanged (framework). CHANGELOG + arch doc.

### Out (explicitly deferred)
- PANE-LAYOUT persistence (serializing the `PaneGroup` tree + respawning N sessions at boot) —
  M2 session restore; theme + docks + terminal size is the demonstrable round-trip.
- A settings UI / editor (M2); live file-watch reload; keybinding persistence; multi-profile.
- Headed baselines ride the #23 desktop-session deferral.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Config-dir isolation via `*_in(dir)` fns (the marley_settings-tests precedent — `load(path)`
  takes a full file path; NO env override exists and none is added). Tests pass a tempdir; the
  shim passes `marley_config_dir()`. No live `~/.marley` in tests (§14).
- D2 — Docks persist as `bool` (true = Open) — serde-native, so marley_app needs NO direct serde
  dep; `bool → DockState` is a tiny pure map (a mutation target). (An `open|closed` enum would
  need serde on `DockState`; bool is simpler for M1.)
- D3 — Theme validated on load: an unknown/missing `appearance.theme` → the Dark default's name
  (never a boot failure); a Parse error on the whole file → applied-DEFAULTS.
- D4 — `applied_from` is PURE given a loaded manager (get reads the in-memory table, no IO) — the
  cov-100/MSI-100 surface; the IO (`load_manager_in`/persist) is thin + tempdir-testable.
- D5 — Pane layout persistence DEFERRED (D-scope: out) — M2.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `applied_from` reads a loaded manager, the system shall map each setting to `AppliedSettings` — the theme name validated via `by_name` (unknown/absent → the Dark default's name), `docks.left`/`right` bools, `terminal.rows`/`cols` — with each default applied when its key is absent. | unit tests over tempdir-backed managers (each default; a valid + an unknown theme name; dock/rows/cols round-trip) |
| REQ-002 | WHEN `load_manager_in(dir)` is called for a directory with no settings file, the system shall create the dir if needed, return a manager yielding every default, and NOT create the file; WHEN the file exists with saved values, it shall return them; WHEN the file is invalid TOML, the loader shall surface the error so the caller falls back to defaults. | unit tests (missing/valid/invalid, all in a tempdir) |
| REQ-002b | WHEN a setting is persisted via the `persist_*` seam in `dir`, the system shall write `settings.toml` under `dir` such that a fresh `load_manager_in(dir)` + `applied_from` yields the persisted value. | unit round-trip test (set → reload → applied) |
| REQ-003 | WHEN `RootView` boots, it shall apply the loaded theme (so `active_theme()` matches the saved/validated name), the loaded dock states, and the loaded terminal rows/cols — and a missing or invalid file shall boot with the defaults, never a failure. | pure `applied_from` + `load_manager_in` coverage; the boot call site is the app.rs shim (review + the deferred headed lane) |
| REQ-004 | WHEN the theme or a dock toggles at runtime, the system shall persist the new value so a subsequent boot restores it. | the persist round-trip unit test (REQ-002b) + shim review |
| REQ-005 | WHEN `settings_file_in(dir)` is called, the system shall return `<dir>/settings.toml`. | unit test (settings.rs, a plain dir — no live home) |
| REQ-006 | WHEN `scripts/gates.sh --diff` runs over the staged change, every gate shall be GREEN with coverage 100%/MSI 100% on the touched pure surface. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact schema fields + `AppliedSettings` shape; the load-or-default flow on a
  Parse error; the `marley_settings_file_path` `_from` seam; whether persist is per-setting or a
  batched save; the R33/R34 text; the mutation targets.
- **P3 Implement** — settings.rs + paths.rs fn + app.rs wiring + Cargo deps + spec + CHANGELOG.
- **P3.5 Inspect** — critics (2): the load/validate/default matrix + the boot/persist wiring.
- **P4 Validate** — tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #26 → the M1.C sprint's last feature ticket.
