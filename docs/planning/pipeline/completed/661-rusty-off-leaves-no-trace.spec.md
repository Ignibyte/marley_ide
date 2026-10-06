---
pipeline_id: 5f0f673a-99bc-4105-8d06-75082855497c
ticket: docs/planning/tickets/closed/TICKET-661-rusty-off-leaves-no-trace.md
status: Phase 4 — Complete PASS
title: "Rusty off leaves no trace"
type: chore
slice: Rusty in Marley R-D0 (docs/marley/rusty-in-marley.md); Chad 2026-10-06, one build and one switch
references: [docs/marley/rusty-in-marley.md, docs/planning/tickets/open/TICKET-661-rusty-off-leaves-no-trace.md]
---

## Title
Rusty off leaves no trace. Chad chose one build with one switch for the personal assistant
(2026-10-06). With `marley.rusty.enabled` off, Marley shall show nothing of Rusty but the switch:
no command in the palette and no item on the settings page past the switch. Turning it on brings
everything back without a restart.

## Scope
### In
- **The palette.** While off, Zed's `CommandPaletteFilter` hides the `rusty` namespace and the
  `marley::ToggleBrainView` action, set at start and whenever the switch changes.
- **The settings page.** While off, the Marley page's Rusty section holds its header and the Rusty
  switch only: Connection, Service URL, Rusty Tools for Agents and the Rusty's Server link go,
  and come back when it turns on. The settings window rebuilds its pages when the switch changes.

### Out (explicitly deferred)
- **Keys bound to a `rusty` action.** Marley's keymap binds Ctrl+Alt+V and Ctrl+Alt+U; while off a
  press still shows the toast that names the switch (R-D0), which tells a user who pressed it why
  nothing opened.
- **The in-app guide and the docs**, which describe Rusty whether it is on or off.
- **A compile-time switch** (Chad, 2026-10-06: one build).

## Reference (§20)
Upstream Zed: `agent_ui` hides its AI commands while `disable_ai` is set
(`crates/agent_ui/src/agent_ui.rs:684-697`, `:790-860`): `CommandPaletteFilter::update_global`
with `hide_namespace` and `hide_action_types`, run at init and on every `SettingsStore` change.
Marley does the same for Rusty. The settings window's `developer_page(cx)` builds its items from
app state, and `rebuild_pages` runs when the staff flag changes (`crates/settings_ui/src/
page_data.rs:65-100`, `settings_ui.rs`'s `FeatureFlagStore` observer); the Marley page does the same
for Rusty's switch.

### Prior art
- **Behaviour maps:** `docs/zed_architecture/` (the command palette and the settings window); nothing
  in `docs/warp_architecture/` or `docs/orca_architecture/` hides a feature by a setting.
- **Published material:** none needed; the behaviour is Zed's own.
- **The code we ship:** `command_palette_hooks::CommandPaletteFilter` (`hide_namespace`,
  `hide_action_types`, `show_namespace`, `show_action_types`; the palette checks it in
  `command_palette.rs:120-128`, and a shown action type overrides a hidden namespace,
  `command_palette_hooks.rs:55-66`); `settings_data(cx)` and `rebuild_pages` in `settings_ui`. The
  Knowledge panel already hides its dock button while off (`knowledge_panel.rs:1382-1402`, as
  `AgentPanel` does) and the rail drops the Brain switch (`rail.rs:4124-4134`).

## UI proof
`script/e2e/661-rusty-off-leaves-no-trace.sh` (`compositor sway`: the settings search is clicked).
Setup: a scratch repository; `marley.rusty.enabled` false in the run's settings, the embedded
connection, `MARLEY_RUSTY_MCP` naming `marley_rusty`'s stand-in over a scratch vault (never the
user's Rusty, R-D8). Shots:
- `661-01-palette-off`: the palette with "rusty" typed: no `rusty:` command; then "brain view"
  typed: no `marley: toggle brain view` (`661-02-brain-off`).
- `661-03-settings-off`: the Settings window's Marley page searched for "Rusty": the Rusty section's
  header and switch, and nothing else of Rusty's.
- `661-04-settings-on`: the switch turned on from the settings file, the window still open:
  Connection, Service URL, Rusty Tools for Agents and Rusty's Server back.
- `661-05-palette-on`: the palette with "rusty" typed: the `rusty:` commands listed.
- `661-06-off-again`: the switch off again: the palette's "rusty" lists nothing.

## Locked-In Decisions
- D1 — **Zed's own filter.** `CommandPaletteFilter`, as `disable_ai` uses it: the `rusty` namespace
  hidden, and `marley::ToggleBrainView` by its type, since the `marley` namespace holds everything
  else of Marley's. Rejected: unregistering the actions (keys bound to them would do nothing and
  say nothing).
- D2 — **Set from the switch as resolved**, at Marley's start and whenever `MarleySettings`
  changes, comparing the last value so the filter is touched only on a change.
- D3 — **The page leaves the items out** while off, built from the user's settings
  (`marley.rusty.enabled`, default false), and the window rebuilds its pages when that changes, in
  a Marley hunk of its `SettingsStore` observer. Rejected: a per-item hide predicate on every
  `SubPageLink` (fifteen literals in Zed's pages would change); a `DynamicItem` (its fields can
  only be setting items, and Rusty's Server is a sub-page link).
- D4 — **Keys keep the toast** (Out): a bound key that does nothing silently would look broken.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rusty.enabled` is off, the command palette shall list no command of the `rusty` namespace. | Shot `661-01-palette-off` |
| REQ-002 | WHILE it is off, the command palette shall not list `marley: toggle brain view`. | Shot `661-02-brain-off` |
| REQ-003 | WHILE it is off, the Marley settings page's Rusty section shall show its header and the Rusty switch and nothing else. | Shot `661-03-settings-off` |
| REQ-004 | WHEN it turns on, the settings window shall show Connection, Service URL, Rusty Tools for Agents and Rusty's Server without being reopened. | Shot `661-04-settings-on` |
| REQ-005 | WHEN it turns on, the command palette shall list the `rusty:` commands without a restart. | Shot `661-05-palette-on` |
| REQ-006 | WHEN it turns off again, the command palette shall hide them again. | Shot `661-06-off-again` |
| REQ-007 | WHILE it is off, a key bound to a `rusty` action shall still show the toast naming the switch. | Review (unchanged code path) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the `settings_ui.rs` ledger row widened first; the palette filter in
  `marley_workbench::rusty`; `marley_page(cx)` and the Rusty section's items by the switch; the
  window's rebuild on the switch's change; a review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R-D0, the architecture notes (§21), the guide's Rusty
  section, ledger capture (§19), the brain decision, close, archive, commit.
