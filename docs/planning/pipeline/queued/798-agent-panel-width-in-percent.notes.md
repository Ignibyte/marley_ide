# The agent panel's width in percent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-798-agent-panel-width-in-percent.md
- **Pipeline spec:** 798-agent-panel-width-in-percent.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Can we make the agent window able to expand the size in
  percentage. Maybe a toggle at the bottom?" He confirmed the plan's click-to-step button.
- **Discovery (drafted by /spec; re-verified at promotion):**
  - `crates/workspace/src/dock.rs`: `PanelSizeState { size, flex }`, `resize_panel_entry`, and
    `panel_uses_flexible_width` (a horizontal dock with the panel's flexible size on).
  - `crates/workspace/src/workspace.rs`: `set_panel_size_state::<T>`, `persist_panel_size_state`
    (the key-value store, per workspace), `dock_size`, `dock_flex_for_size`,
    `default_dock_flex` (1.0).
  - `crates/agent_ui/src/agent_panel.rs`: `supports_flexible_size` true, `has_flexible_size`
    from `AgentSettings::flexible` (`default.json`'s `agent.flexible: true`), `set_zoomed`,
    `toggle_zoom`.
  - `crates/agent_ui/src/conversation_view/thread_view.rs`: `render_message_editor`'s bottom
    row (left group: add context, follow, fast mode, thinking).
  - `crates/agent_ui/src/conversation_view.rs`: `is_visible_in_agent_panel`, with #734's
    `MarleyThreadHost::shows` for center tabs.
