# Component templates from files — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-801-component-templates-from-files.md
- **Pipeline spec:** 801-component-templates-from-files.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Maybe even research how we could make this pluggable to add
  outside components without recompile." On the plan: "I'm not sure if the WASM plugins would be
  useful but maybe we create a ticket for that later." Templates are the first step; WASM and MCP
  Apps wait in the intake doc.
- **Discovery (drafted by /spec; re-verified at promotion):** `jsonschema` 0.51 and `schemars` 1.0
  in `[workspace.dependencies]`; `paths::config_dir()`; `Fs::watch`; the settings file watcher in
  `crates/settings` as the reload pattern.
