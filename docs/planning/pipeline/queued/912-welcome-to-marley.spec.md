---
pipeline_id: 501de5be-02dd-466c-adc5-3a5dfee31042
ticket: docs/planning/tickets/open/TICKET-912-welcome-to-marley.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Welcome to Marley on first run
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/G-system.md, docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/marley/look-plan.md]
---

## Title
A first start of Marley shows "Welcome to Marley" in place of Zed's onboarding page: the cat
mark, the title and "Four choices, each one changeable later in Settings.", a row of option cards
for each of Layout, Theme, Agents and Rusty, then Open a Folder and Go to Home, as the Dialogs
board draws it. Each choice writes its setting at once.

## Scope
### In
- `welcome_page.rs` (Marley, new): `MarleyWelcome`, a center tab item with the screen.
  - The head: the cat mark at 56 px (`images/marley_mark.svg`, #829's asset), "Welcome to Marley"
    and the line under it.
  - Layout: Marley ("A rail of projects and their terminals") and Zed ("Zed's docks, as Zed ships
    them"); a click writes `marley.layout` through `write_layout`.
  - Theme: Marley Night, Marley Day and, where Omarchy is installed, Follow Omarchy; Night and Day
    write `theme.mode` (dark or light) and turn `marley.omarchy_theme` off; Follow Omarchy turns
    it on, and #759's reconcile does the rest.
  - Agents: a card each for Claude Code and Codex with what Marley found (`agent_versions::found`:
    "Found: <version>", or "Not on the PATH"), and Connect the plugin, which runs #547's
    `claude_plugin::install` and reads "Connected" once Claude Code lists the plugin.
  - Rusty: On ("Your brain, tasks and notes") and Off ("No trace of it anywhere"); a click writes
    `marley.rusty.enabled`.
  - The lit card in each row is the setting in effect, read at each render.
  - Open a Folder (primary) dispatches `workspace::Open`; Go to Home brings Home forward
    (`home_page::open_here`). Both close the welcome tab.
- `marley_workbench.rs` (Marley): the module, its `init`, and the action `marley: show welcome`,
  which opens the screen again in the window's Home group.
- `crates/zed/src/main.rs` (Zed crate): the `FIRST_OPEN` branch calls Marley's
  `welcome_page::show_first_run` in place of `show_onboarding_view`; the new function opens a
  window as Zed's does and writes `first_open` to the key-value store the same way. One ledger row.
- `script/e2e.sh` (tooling): a `first_run` keyword for a scenario. Its profile gets no copy of the
  user's database, so nothing restores and `first_open` is unset, and the launch names no folder.

### Out (explicitly deferred)
- Zed's own `zed: open onboarding` stays Zed's page; `marley: show welcome` is Marley's way back.
- Choosing which agents Marley offers: no such setting exists. The Agents row says what was found
  and connects the plugin; it switches nothing else.
- The line about Marley and her cat on this screen: its wording is Chad's (look-plan W0.2).
- The rest of the Dialogs board (close dialog Wait, second-launch notice): TICKET-913. Quit
  dialog wording, Rusty-not-connected dialog, the setting-change proposal as a dialog: a
  follow-up ticket for the Dialogs board's remaining looks.
- Geist fonts: the review's "For Chad" item; the screen uses Marley's fonts.

## Reference (§20)
Upstream Zed (`onboarding` crate): Zed's first-run behaviour is kept. The first start is still
`FIRST_OPEN` unset in the key-value store, the page opens in a new window's center and the key is
written as it opens, so it shows once. Only the page's content is Marley's.

### Prior art
- `crates/zed/src/main.rs:1606-1607`: `FIRST_OPEN` → `onboarding::show_onboarding_view`.
  `crates/onboarding/src/onboarding.rs:183-206` (`show_onboarding_view`: `open_new`, add the page
  to the center, focus it, write `first_open`); `basics_page.rs:203-230` (`write_mode_change`,
  `write_theme_change`: how Zed's onboarding writes the theme).
- `marley_workbench.rs:1445` `write_layout(MarleyLayout, cx)`; `MarleySettings::layout`.
- `omarchy_theme.rs` (#759): `marley.omarchy_theme` is reconciled on every settings change;
  `OmarchyPaths::from_env` finds Omarchy's folders.
- `agent_versions.rs:440` `found(kind, cx)` and `program(kind, cx)`: the `claude` and `codex`
  Marley found and their versions (#648).
- `claude_plugin.rs:224` `install(plugin, workspace, cx)`, `ClaudePlugin::installed`; the agent
  bar's Connect chip (`agent_bar.rs:450-464`) is the existing caller.
- `rusty.rs:745`: `marley.rusty.enabled` is the one switch (assistant-is-one-switch).
- `home_page.rs:99` `ensure`, `:121` `open_here`; `rail.rs:9819` `start_workspace` takes Zed's
  start workspace and its first-run tab as the Home group (#700, F-claude-700).
- `title_bar.rs:352` draws the mark with `gpui::img("images/marley_mark.svg")`.
- `prompts.rs:126` `render_button` and `home_page.rs:900` `pill`: the board's button and card
  shapes already in Marley.
- `script/e2e.sh:640-740`: the profile copy (settings, then `cp -r "$data/db"`), and line 809:
  a run must name a folder; nothing in the tree reaches a first start.
- Grep for "Welcome to Marley", "Four choices" and "Go to Home" in `crates/marley_*`: none.

## UI proof
Scenario `script/e2e/912-welcome-to-marley.sh` (sway, `first_run`): a stand-in `claude` named by
`MARLEY_CLAUDE` that answers `--version` with "2.1.304 (Claude Code)" and logs any other argv;
`CLAUDE_CONFIG_DIR` in the run's folder; Codex not found (the runner's `MARLEY_CODEX`).
- `912-01-night`: the first start: the welcome tab in Marley Night with the mark, the four rows
  (Marley, Marley Night and Rusty Off lit) and the two buttons (REQ-001, REQ-002).
- `912-02-day`: Marley Day and Rusty On clicked: the screen redrawn in Day, those cards lit; the
  settings file holds `"mode": "light"` and `"enabled": true` under `rusty` (REQ-003).
- `912-03-plugin`: Connect the plugin clicked: the stand-in's log holds the plugin install
  (REQ-004).
- `912-04-home`: Go to Home: Home in front, no welcome tab (REQ-005).
- `912-05-again`: `marley: show welcome` from the palette: the screen again (REQ-006).
- `912-06-next-start`: quit and launch again: no welcome tab (REQ-007).

## Locked-In Decisions
- D1 — The screen is a center tab, not a modal: it sits in the window's Home group (#700), keeps
  the rail beside it, and is not serialized, so a restart never restores it (TICKET-794's
  restored onboarding tab cannot happen with it).
- D2 — Each choice writes its setting on click; there is no Done step. The lit card reads the
  merged settings, so the screen shows what is in effect even after a change in Settings.
- D3 — Agents cards are facts, not switches: no setting chooses which agents Marley offers.
  Connect the plugin is the row's one action, greyed when no `claude` was found.
- D4 — Follow Omarchy shows only where Omarchy's config folder exists; without it the setting
  does nothing (#759).
- D5 — Open a Folder and Go to Home close the welcome tab; the tab's own close does the same.
- D6 — `first_open` is written when the screen opens, as Zed writes it, so a crash mid-tour does
  not show it again.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts with no `first_open` key and nothing to restore, it shall show Welcome to Marley in a center tab of the Home group, in place of Zed's onboarding page. | `912-01-night` |
| REQ-002 | The welcome screen shall draw the cat mark, the title and its line, the Layout, Theme, Agents and Rusty rows of option cards with the setting in effect lit, and Open a Folder and Go to Home, in Marley Night and Marley Day. | `912-01-night`, `912-02-day` |
| REQ-003 | WHEN a Layout, Theme or Rusty card is clicked, Marley shall write its setting to the user's settings and light that card. | `912-02-day` |
| REQ-004 | The Agents row shall show each of Claude Code and Codex with the version found or "Not on the PATH", and WHEN Connect the plugin is clicked it shall run the plugin install. | `912-01-night`, `912-03-plugin` |
| REQ-005 | WHEN Go to Home is clicked, the welcome tab shall close and Home shall come forward. | `912-04-home` |
| REQ-006 | WHEN `marley: show welcome` runs, the welcome screen shall open in the window's Home group. | `912-05-again` |
| REQ-007 | WHEN Marley starts again after a first start, it shall not show the welcome screen. | `912-06-next-start` |
| REQ-008 | The Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `welcome_page.rs`, the action, the `main.rs` hunk and its ledger row, the
  runner's `first_run`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
