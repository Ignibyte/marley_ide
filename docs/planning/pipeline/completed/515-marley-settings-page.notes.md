# A Marley page in the Settings window — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-515-marley-settings-page.md
- **Pipeline spec:** 515-marley-settings-page.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "WE need a marley settings pane"; "Configuration page for marley".
- **Classification / tier:** feature; small additive hunks in two Zed crates, a new file in one.
- **Recall (§18.3):** #460 made `marley.layout` a setting (the content type lives in
  `settings_content` because its derive macros resolve only there); #501 found that
  `agent: open settings` opens Zed's Settings window at a page through `OpenSettingsPage`. #514
  set the telemetry defaults this page shows.
- **Discovery:** `settings_data` (page_data.rs:65) lists the pages; `privacy_section` (page_data.rs:431)
  holds the two telemetry items; the renderer registry (settings_ui.rs:551) maps a type to a
  renderer, and `render_dropdown` needs `strum::VariantArray + VariantNames`; `OpenSettingsPage`
  (zed_actions lib.rs:161) takes a page title; settings_content depends on strum already.
- **Checklist (no task tool):** done.

### Design
- **Approach.** `marley_page.rs` mirrors `general_page`: `SettingsPage { title: "Marley", items }`
  with a Layout section (one `SettingItem`, `json_path: Some("marley.layout")`, pick and write on
  `settings_content.marley`) and a Privacy section (the two telemetry items, copied from
  `privacy_section`, which is private to page_data.rs). Registration: `mod marley_page;` and
  `.add_basic_renderer::<settings::MarleyLayout>(render_dropdown)`. The action: `OpenSettings` in
  Marley's `actions!` list, registered on the workspace, dispatching `OpenSettingsPage`.
- **File manifest:** `crates/settings_ui/src/marley_page.rs` (Zed crate, new);
  `crates/settings_ui/src/page_data.rs` and `settings_ui.rs` (Zed crate, one line each plus the
  module line); `crates/settings_content/src/marley.rs` (Zed crate, two derives);
  `crates/marley_workbench/src/marley_workbench.rs` (Marley crate: the action).
- **E2E plan:** as the spec's UI proof. Coordinates of the page's dropdown come from the first run.
- **Risks:** the page title must match `OpenSettingsPage`'s lookup exactly ("Marley").

## Phase 2 — Code
- **Built.** The ledger rows first (`settings_ui`'s new `marley_page.rs`, `page_data.rs`,
  `settings_ui.rs`, and the widened `settings_content/src/marley.rs` row). `marley_page()` builds
  the page from a Layout section (`marley.layout`) and a Privacy section (the two telemetry keys),
  chained into the page's items; `settings_data` lists it first; `settings_ui.rs` declares the
  module and registers `render_dropdown` for `settings::MarleyLayout`, which now derives
  `strum::VariantArray` and `strum::VariantNames`. `marley::OpenSettings` dispatches
  `zed_actions::OpenSettingsPage { page: "Marley", target: None }`.
- **Checks.** `cargo check`, `cargo fmt` and `just clippy settings_content settings_ui
  marley_workbench` (all targets, `-D warnings`) clean.
- **Deviation.** None from the design.
- **Review.** Each Zed hunk is additive with a `// Marley:` comment; the page's fields read and
  write the same keys as Zed's own items, so the file picker, search and "Edit in settings.json"
  work on it. Nothing reads or updates an entity during its own update.

## Phase 3 — Test
- **Scenario:** `script/e2e/515-marley-settings-page.sh` (`compositor sway`), four runs.
  - **Run 1, red:** `marley: open settings` ran and no Settings window opened; the log said
    `window not found` (gpui `app.rs:2572`) at that moment. The handler was a global
    `cx.on_action` that called `cx.dispatch_action(&OpenSettingsPage)`, and an app-level dispatch
    made from inside the palette's own dispatch finds no window, since the window is out of the
    app's map while it is being updated. Fixed at the source: the action is registered on the
    workspace and calls `window.dispatch_action`, as the Agent Panel opens its page
    (`agent_panel.rs:3753`). Recorded as F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001.
  - **Runs 2 and 3:** the page, then the open dropdown, to measure the click points.
  - **Run 4, green:** the full scenario.
- **Shots (run 4), read:**
  - `515-01-marley-page` — the Settings window beside the main window: the list starts with
    Marley (selected), then General, Appearance and the rest; the page shows "Marley", a Layout
    section with the Layout dropdown at Marley, and a Privacy section with Telemetry Diagnostics
    and Telemetry Metrics, both off. REQ-001, REQ-002.
  - `515-02a-dropdown-open` — the dropdown's menu: Zed, and Marley with a tick.
  - `515-02-zed-layout` — the dropdown now at Zed, and the main window in Zed's layout: the
    Threads Sidebar ("Search threads...", "No threads yet") where the rail was. REQ-003.
- **Focus:** sway; nothing on Hyprland.
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`, every gate PASS with `settings_ui` and
  `settings_content` in scope (`gate-515.log` in the scratchpad).

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: a Marley page in the Settings window);
  `docs/marley_architecture/marley_workbench.md` (the settings page, under the switch); the four
  touchpoint rows describe what shipped.
- **Knowledge:** F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001;
  L-claude-515-dispatch-through-the-window-from-inside-an-action-001.
- **Brain:** a decision recorded directly (no consultation was opened for this ticket).
- **Ticket:** closed; the pair archived.
