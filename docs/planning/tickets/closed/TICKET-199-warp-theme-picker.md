# TICKET-199 — Live theme picker in the command palette

- **Forge ticket:** #199 (2223ad76-e2f3-4f93-92e6-ff1ec27905c8) (feature, M12.2, theme/palette/settings)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** fd3b6cad-19c8-40d3-90fe-69be0de38cb4
- **Pipeline doc:** ../../pipeline/active/warp-theme-picker.spec.md
- **Source ticket:** M12.2 sprint #25 (the polish batch)
- **Status:** closed

## Summary
Switching theme needs hand-editing TOML today. Add a command-palette entry per theme ("Theme: <name>")
that applies it instantly + persists the choice. The registry (Dark+Light), `set_theme` (applies live +
persists), and the `persist_theme`/boot-load round-trip already exist — the missing bit is the picker:
dynamic palette commands in a `THEME_BASE` id range (mirroring the #87 saved-host `CONNECT_BASE` pattern),
dispatched to `set_theme`, gated by a small pure id↔index helper (cov/MSI 100). Clean-room — reuses
existing themes/tokens.

## Acceptance
The palette lists a "Theme: <name>" per theme; selecting one switches the cockpit colors live + persists;
the pure index resolver is exact-value tested; existing commands/set_theme unchanged. Full EARS
(REQ-001..005) in the pipeline spec.
