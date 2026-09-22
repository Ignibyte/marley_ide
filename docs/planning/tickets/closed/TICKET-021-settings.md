---
ticket: TICKET-021
forge: forge#21 (a1bf519d-3b54-43a2-bfe2-de1d3cea1716)
status: closed
type: feature
milestone: M1
sprint: M1.B — The Cockpit (seq 5/5 — THE FINALE)
branch: ticket-021-settings
pipeline: docs/planning/pipeline/active/settings.spec.md
spec: docs/specs/SPEC-settings.spec.md
aar: 35180d4c-c9ba-40ca-9f8b-b3e5dd838176
---

# TICKET-021 — marley_settings (typed TOML settings framework) — THE FINALE

Sprint M1.B "The Cockpit" seq 5/5. The NEW crate `marley_settings` (SPEC-settings R1-R18): a typed
declarative settings framework — `SettingsValue`/`Setting` traits + a `SettingsManager` (load / register /
get[lazy] / set / clear / reload / default_values / subscribe / is_syncable) over a retained `toml::Value`
working tree + `define_settings_group!`/`define_setting!` macros (with a `const assert!` toml_path guard)
+ `ChangeEvent`/`SettingsError`; `Duration` persists as whole seconds.

## Acceptance
- The WHOLE crate is PURE — **cov 100 / MSI 100** (R1-R18; the file IO tested via a tempdir; NO exclude).
- R2 via a `trybuild` compile-fail snapshot; an integration seam test (write→drop→reload→survive).
- reuses `toml` + `serde`. FULL `scripts/gates.sh` → `GATE GREEN` (gate-15 N/A). §21: CHANGELOG +
  docs/marley_architecture/settings.md. **Completing this closes M1.B "The Cockpit".**
