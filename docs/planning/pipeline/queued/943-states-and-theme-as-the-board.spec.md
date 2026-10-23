---
pipeline_id: c060374e-4538-44b2-8dd7-a52f9b838a02
ticket: docs/planning/tickets/open/TICKET-943-states-and-theme-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Empty states and theme details as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/marley/look-plan.md]
---

## Title
The States and Theme boards' details: Home's "No project open" card in a window with no project;
the same card in the Files panel there; the harness card's "The harness is stopped"; a lighter ring
on a chosen orange button, so the ring still shows; and the About line's second sentence.

## Scope
### In
- `crates/marley_workbench/src/home_page.rs` (Marley crate): while no group in the window has a
  folder open (Home's own project aside), Home draws the "No project open" card from the States
  board above its grid: glyph `..`, the line "Open a folder, clone one, or start an agent in your
  home folder.", and Open a Folder. The Recent Projects card with no projects says "No recent
  projects yet." as a muted line.
- `crates/project_panel/src/project_panel.rs` (Zed crate): `MarleyProjectEmptyState`, a global
  holding a function that returns an element. While it is set, the panel draws that element in
  place of `ProjectEmptyState`, inside the same wrapper, so dropping a folder on it still works.
- `crates/marley_workbench/src/states.rs` (Marley crate): the card both places draw
  (`no_project_card`), and the function the Files panel's global holds; the global set at init.
  `marley_workbench` gains `project_panel` as a dependency (it is in the build already).
- `crates/marley_workbench/src/rail.rs` (Marley crate): the harness card's title "The harness is
  stopped".
- `crates/marley_workbench/src/prompts.rs` (Marley crate): in `render_button`, a chosen Primary
  button draws #895's 2 px ring outside its shape in a lighter colour (D3), in place of the focus
  border; other roles keep the focus border.
- `crates/zed/src/zed.rs` (Zed crate): `open_about_window`'s Marley line gains "Her coat is the
  color you act with, her eyes the color of things done.", wrapping under the first sentence.

### Out (explicitly deferred)
- A card for a missing agent CLI ("Codex is not installed"): #894 Out.
- Rusty's retry countdown ("Trying again in 4 s"): its own change to the Rusty client's retry; a
  follow-up ticket.
- The focus ring on rail rows and pickers: #895 Out (Chad's answer).
- The coat as the primary buttons' fill: #906 (the build order's first item). D3's ring shows on
  either fill.
- Geist fonts: waits for Chad's word (the review's "For Chad").
- Day's accent #d9661a: deferred on purpose (#828, contrast).
- The cat mark on Home or a first-run screen: #829 Out.
- About's build line, update line and buttons: #944.

## Reference (§20)
N/A — Marley-specific: Marley's own empty-state card (#857), prompts (#870), focus ring (#895) and
About line (#829). The Files panel keeps Zed's project panel (`project_panel`) and its drop
target; only the empty state's content changes. About keeps Zed's About window.

### Prior art
- `crates/marley_workbench/src/home_page.rs:611` `render_recent`: the `EmptyState` "No recent
  projects" when `recent` is empty (`:617`); `:524` `project_choices` lists the window's other open
  projects (Home's own left out), the test this ticket needs.
- `crates/marley_workbench/src/states.rs:18` `EmptyState` (#857): glyph, title, line, one action
  drawn by #870's `render_button`.
- `crates/project_panel/src/project_panel.rs:8137` the empty branch: `ProjectEmptyState::new("Project
  Panel", …)` with Open Project and Clone Repository, in a wrapper that takes dropped paths;
  `crates/ui/src/components/project_empty_state.rs` draws "Choose one of the options below to use
  the Project Panel". The panel already carries Marley hunks (#735).
- `crates/workspace/src/dock.rs` `MarleyDockHeader` and `crates/search/src/project_search.rs`
  `MarleySearchLook`: a global the Zed view asks for, the pattern for `MarleyProjectEmptyState`.
- `crates/marley_workbench/src/rail.rs:5263` the harness card (#857), title "The harness is not
  running".
- `crates/marley_workbench/src/prompts.rs:125` `render_button`: Primary fills with `text_accent`,
  a chosen button gets `border_focused` as its border (`:154`); `prompts.rs:68` a prompt's chosen
  button is the arrow keys' (`index == prompt.active && index != 0`).
- `crates/ui/src/styles/elevation.rs:122` `focus_ring` (#895): a 2 px shadow spread in
  `border_focused`, #ff8c42 in Night (`assets/themes/marley/marley.json:12`), the same as the
  coat a primary fill is moving to.
- `crates/zed/src/zed.rs:1635` the About line (#829), shown on the `dev` channel only
  (`is_marley`); the window is 440 by 300.
- The Theme board's line: "Named for Marley: a solid orange cat with green eyes. Her coat is the
  color you act with, her eyes the color of things done."
- Scenarios to reuse: `script/e2e/857-empty-states.sh` (a harness command that is not there),
  `script/e2e/870-prompts.sh` (an edited README closed: Zed's save prompt as Marley's card),
  `script/e2e/829-the-cat-mark.sh` (`zed: about`).

## UI proof
Scenario `script/e2e/943-states-and-theme-as-the-board.sh` (sway). Setup: a scratch repository
with a README; `marley.harness` naming a command that is not there (as #857); a second window
opened with `workspace: new window`, which holds Home's group only.
- `943-01-home`: the new window's Home: "No project open" over the grid (REQ-001).
- `943-02-files`: the same window with Files shown: the card in the panel (REQ-002).
- `943-03-harness`: the repository's window: the rail's harness card, "The harness is stopped"
  (REQ-003).
- `943-04-ring`: the README edited and closed, Right pressed until Save is chosen: the lighter ring
  round Save (REQ-004).
- `943-05-about`: `zed: about`: both sentences under the headline (REQ-005).
- `943-06-day`: Marley Day: the new window's Home card and the chosen Save with its ring (REQ-001,
  REQ-004).

## Locked-In Decisions
- D1 — "A window with no project": no group in the window has a folder open, Home's own group
  aside, as `project_choices` counts the window's open projects. Home and Files show the same card
  with the same words and its one action, Open a Folder. Clone Repository stays on Home's start
  pills and in the palette.
- D2 — Files' card is Marley's only while the global is set; Zed's empty state stays for Zed's
  layout tests and for the other panels that use `ProjectEmptyState`.
- D3 — The ring on a chosen Primary button is `border_focused` mixed 60% toward the theme's text
  colour: a light peach in Night (near the board's #ffd0a8), a deep brown in Day. It differs from
  the fill whether the fill is the coat or the lighter orange.
- D4 — About's line stays one label that wraps; the window keeps its size unless the shot shows
  the buttons pushed off, then its height grows on Marley's channel only.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE no group in the window has a folder open, Home shall show the "No project open" card with Open a Folder, in Marley Night and Marley Day. | `943-01-home`, `943-06-day` |
| REQ-002 | WHILE the window's project has no folder, the Files panel shall show the "No project open" card in place of Zed's empty state. | `943-02-files` |
| REQ-003 | WHILE the harness Marley follows is not running, the rail's harness card shall read "The harness is stopped". | `943-03-harness` |
| REQ-004 | WHEN a prompt's orange Primary button is chosen with the arrow keys, Marley shall draw a 2 px ring in the lighter colour outside it, in Marley Night and Marley Day. | `943-04-ring`, `943-06-day` |
| REQ-005 | WHEN the user opens About in Marley, it shall show "Named for Marley: a solid orange cat with green eyes. Her coat is the color you act with, her eyes the color of things done." | `943-05-about` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — Home's card and test, the Files global and its card, the harness title, the ring,
  the About line, with the ledger rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
