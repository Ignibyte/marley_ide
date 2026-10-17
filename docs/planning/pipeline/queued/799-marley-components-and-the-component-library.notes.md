# Marley components and the Component Library — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-799-marley-components-and-the-component-library.md
- **Pipeline spec:** 799-marley-components-and-the-component-library.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see the ticket). He confirmed the batch the same day.
- **Research:** `docs/planning/design-notes/agent-components-2026-10-17.md`.
- **Discovery (drafted by /spec; re-verified at promotion):**
  - Zed's registry: `crates/component/src/component.rs` (`Component`, `ComponentScope`,
    `register_component`), `crates/ui_macros/src/derive_register_component.rs`,
    `crates/component_preview/src/component_preview.rs` (the page layout, `PreviewPage`).
  - `ui` pieces: `crates/ui/src/components/{data_table,progress,callout,chip,divider,label}.rs`.
  - Home: `crates/marley_workbench/src/home_page.rs`, `render_configure` (a fixed array of six
    buttons dispatched by id).
  - Settings: `crates/settings_ui/src/marley_page.rs`, the "System One Calls" `ActionLink`
    (built by name with `cx.build_action`, sent to the original window, Settings closed).
  - Tab pattern: `crates/marley_workbench/src/guide_tab.rs` (`open`, reused by title).
  - Workspace deps: `schemars` 1.0, `serde_json` with `preserve_order`.
