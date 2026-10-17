# An agent's HTML page in a Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-802-an-agents-html-page-in-a-browser-tab.md
- **Pipeline spec:** 802-an-agents-html-page-in-a-browser-tab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "So the options can be inline editing for agents to show
  components or static html in a scratch file or existing project for that sort of style."
- **Discovery (drafted by /spec; re-verified at promotion):** `browser_open_url`
  (`crates/marley_workbench/src/browser_tools.rs`, `open_url`; registry description names #586's
  local HTML pages); the Browser tab's CDP session in `crates/marley_browser`;
  `paths::data_dir()`; `theme::ActiveTheme`.
