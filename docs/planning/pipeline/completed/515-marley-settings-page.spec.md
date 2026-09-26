---
pipeline_id: a88a5a1b-396e-4b50-9ea1-09a1cb6bde24
ticket: docs/planning/tickets/closed/TICKET-515-marley-settings-page.md
status: Phase 4 — Complete PASS
title: "A Marley page in the Settings window"
type: feature
slice: settings, cross-cutting (Chad's configuration page)
references: [docs/planning/pipeline/completed/514-telemetry-off-by-default.spec.md, docs/planning/pipeline/completed/460-marley-layout-by-default.spec.md]
---

## Title
Zed's Settings window gains a Marley page, first in its list, with Marley's layout as a dropdown
and the telemetry toggles; `marley: open settings` opens the window on it. It is where every later
Marley setting goes.

## Scope
### In
- `crates/settings_ui/src/marley_page.rs` (new, a Zed crate): `marley_page()` builds a
  `SettingsPage` titled "Marley" with a Layout section (`marley.layout`) and a Privacy section
  (`telemetry.diagnostics`, `telemetry.metrics`, the same fields as Zed's General page).
- `crates/settings_ui/src/page_data.rs`: the page first in `settings_data`.
- `crates/settings_ui/src/settings_ui.rs`: `mod marley_page;` and the dropdown renderer for
  `settings::MarleyLayout`.
- `crates/settings_content/src/marley.rs`: `MarleyLayout` derives `strum::VariantArray` and
  `strum::VariantNames`, which the dropdown renderer needs.
- `marley: open settings` in `crates/marley_workbench` (Marley-owned), dispatching
  `zed_actions::OpenSettingsPage { page: "Marley" }`.
- Rows for the Zed paths in `docs/marley/zed-touchpoints.md`.

### Out (explicitly deferred)
- Settings that do not exist yet (the browser's per-project Chromium, agents' permission modes,
  secret redaction, Jev): each ticket adds its section to this page.
- Marley's MCP grants, which live in `marley_mcp`'s own configuration rather than Zed's settings.

## Reference (§20)
Upstream Zed's settings UI (`settings_ui`): a page is a `SettingsPage` of section headers and
`SettingItem`s whose `SettingField` picks and writes one key of `SettingsContent`, rendered by
type (toggles for `bool`, a dropdown for an enum with `strum::VariantArray`). Marley adds a page in
the same shape, so search, the file picker (user or project) and "Edit in settings.json" work on
it as on Zed's own. Warp: N/A. Orca's settings search and deep links (report 05 §2.18) are the same
idea; Zed already has both.

### Prior art
- **Code we already ship.** `crates/settings_ui/src/page_data.rs` (`settings_data`,
  `privacy_section`, `general_page`), `crates/settings_ui/src/settings_ui.rs` (the renderer
  registry, `render_dropdown`, the `OpenSettingsPage` handler), `crates/zed_actions/src/lib.rs`
  (`OpenSettingsPage { page, target }`), `crates/settings_content/src/marley.rs`
  (`MarleySettingsContent`, `MarleyLayout`), `crates/marley_workbench/src/marley_workbench.rs`
  (the `marley` actions and the layout switch that follows the setting).
- **Reports.** docs/orca_architecture/05-terminal-and-workspace.md §2.18.

## UI proof
UI-AFFECTING. `script/e2e/515-marley-settings-page.sh` (`compositor sway`): `marley: open
settings` from the palette opens the Settings window on the Marley page, first in the list,
showing the Layout dropdown at Marley and both telemetry toggles off (`515-01-marley-page`); the
dropdown set to Zed switches the main window to Zed's layout (`515-02-zed-layout`).

## Locked-In Decisions
- D1 — The page lives in `settings_ui` as a new file, not in a Marley crate: a page is data the
  settings UI owns (its fields are closures over `SettingsContent`), and `settings_ui` cannot
  depend on a Marley crate without a cycle through the workspace.
- D2 — Marley first in the list: it is the page Chad opens this window for.
- D3 — The telemetry toggles appear on both pages, reading and writing the same keys.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley: open settings` runs, the Settings window shall open on a page titled Marley, first in its list. | Shot `515-01-marley-page` |
| REQ-002 | WHILE the Marley page shows, it shall show the Layout dropdown with the current layout and the Telemetry Diagnostics and Telemetry Metrics toggles. | Shot `515-01-marley-page` |
| REQ-003 | WHEN the Layout dropdown is set to Zed, the windows shall switch to Zed's layout. | Shot `515-02-zed-layout` |

## Phase Plan
- **P1 Plan** — this spec.
- **P2 Code** — the rows, the derives, the page, its registration, the action; fmt and clippy on
  `settings_ui`, `settings_content` and `marley_workbench`.
- **P3 Test** — the scenario, the shots; the gate.
- **P4 Complete** — docs, knowledge, close, archive, commit.
