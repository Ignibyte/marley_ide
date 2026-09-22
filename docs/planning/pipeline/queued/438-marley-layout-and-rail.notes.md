# The Marley layout switch and the first rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-438-marley-layout-and-rail.md
- **Pipeline spec:** 438-marley-layout-and-rail.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad, 2026-09-22: Warp's layout, "the terminal is the main thing and its on the
  left under the project", as "a Marley layout which basically makes it where zed can be used
  as default but Marley basically is its own layout/addition". Rows: terminals and Zed threads
  (threads land in #439).
- **Classification / tier:** feature, large; one new Marley crate plus small Zed touchpoints in
  `settings_content`, `settings`, `zed` and the root manifest.
- **Recall (§18.3):** `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`;
  `PR-claude-selection-bg-distinct-from-container-001` (the selected fill must read against
  the rail background); `PR-claude-new-setting-needs-nondefault-roundtrip-leg-001`;
  `PR-claude-integration-only-coverage-fails-gate4-001`;
  `PR-claude-deferred-gpui-handle-op-needs-notify-in-headless-001`;
  `PR-claude-live-refresh-selection-identity-key-must-be-unique-001` (row identity keys by
  item or entity id, never by title). The gpui-era rail's failures (`F-claude-418-*`,
  `F-claude-419-*`) are about cross-surface selection writes, which the one-selector rule
  prevents.
- **Discovery (the three 2026-09-22 Explore sweeps, summarized in the plan):**
  `multi_workspace.rs:121-160`, `:387-399`, `:1996-2200`; `zed.rs:536-546`, `:2344-2376`,
  `:5856-5990`; `settings_content.rs:174`; `vscode_import.rs:183-245`;
  `settings_store.rs:919-941`; `terminal_panel.rs:835-879`; `terminal_view.rs:233`;
  `workspace.rs:4296`, `:4939`, `:5559`, `:10336-10352`; `platform_title_bar.rs:250-313`;
  `sidebar.rs:832`, `:7328-7418` (behavior reference only).
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
