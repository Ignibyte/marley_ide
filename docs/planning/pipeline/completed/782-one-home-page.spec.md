---
pipeline_id: 9489c328-efb5-4db9-be8f-3038ca938075
ticket: docs/planning/tickets/closed/TICKET-782-one-home-page.md
status: Phase 4 — Complete PASS
title: One Home page, with Rusty's cards while Rusty is on, and Home first in the rail
type: feature
slice: the Marley layout (Home's page #701, Rusty's home page #679, the status bar #739 and #778)
references: [679, 701, 703, 739, 743, 778]
---

## Title
Rusty's home page folds into Home. Its cards show on Home while `marley.rusty.enabled` is on, and
Rusty's separate tab and status-bar button go. Home's row comes first under its group in the rail.

Chad, 2026-10-13: "combining the rusty home page with the marley home page and then just show rusty
items if it is configured", then "lets do that and also in default the Home that always stays
should always be at the top."

## Scope
### In
- Rusty's cards on Home, below RECENT PROJECTS and AGENTS AT WORK: RUSTY (the screens, Open Page…
  and the three captures), RECENT PAGES beside FOLLOW-UPS DUE, and TASKS. They are drawn by a view
  of the `rusty` module that Home holds while the switch is on.
- Rusty on and not connected: one RUSTY card saying Rusty is not connected and why.
- Rusty off: no Rusty card and no placeholder.
- Home's cards in this order: START, NEW AGENT, RECENT PROJECTS | AGENTS AT WORK, RUSTY, RECENT
  PAGES | FOLLOW-UPS DUE, TASKS, AGENT ACTIVITY, CONFIGURE.
- Rusty's separate tab (`RustyHome`, titled Rusty) goes, and so does the status bar's Rusty button.
- `rusty: open home` opens Home in the shown group. The `rusty` verb with no screen keeps its
  refusal while Rusty is off or not connected, and otherwise opens Home.
- The rail lists Home's row first under the group that holds it, above its terminals, worktrees
  and Browser tabs. Default always holds it.

### Out (explicitly deferred)
- The tab strip. `home_page::ensure` already puts Home at index 0 in Default. Pinning it with Zed's
  pinned tabs (`Pane::pinned_tab_count`) would keep it there against a drag, but Chad asked about
  the rail.
- Home across a restart (`SerializableItem`). It is added again when Default is shown (#701).
- Re-reading Rusty's cards only while Home is visible. They re-read on each change Rusty announces,
  as Rusty's page did while it was open.

## Reference (§20)
Upstream Zed, workspace: `welcome::WelcomePage`. Home already keeps its useful entries (#701). This
change adds a section that comes and goes with a setting, as Zed's own pages hide AI entries behind
`DisableAiSettings`. N/A for Warp: Marley's Home and Rusty's cards have no Warp analog, and Warp's
welcome flow (`docs/warp_architecture/crates/onboarding.md`) is a first-run wizard, not a page that
stays open.

### Prior art
- **The code we ship:** `rusty::home_tab::RustyHome` already draws and reads every Rusty card, and
  its `load` (the follow-ups due and every list's open tasks, one read at a time) moves over as it
  is. `rusty::is_on`, `rusty::is_connected` and `rusty::unavailable` already say whether Rusty is on
  and why it is not connected. `CommandPaletteFilter::hide_namespace("rusty")` already hides
  `rusty: open home` while Rusty is off. gpui renders a child `Entity<V: Render>` in place, so Home
  can hold Rusty's view without a second item.
- **The rail:** `marley_rail::walk` orders a group's rows by kind (terminals, worktrees, Browser
  tabs, other tabs, threads, files, ports). Home is an `Other` tab today, so it comes after every
  terminal. `TabKind` is the seam: one more kind, ordered first.
- **Zed's pane:** `Pane::pinned_tab_count` pins tabs to the strip's start. It covers the tab strip,
  not the rail's order, so it is noted under Out.
- **Behavior maps:** nothing on a home page in `docs/zed_architecture/` or `docs/orca_architecture/`;
  Warp's is its onboarding crate, above.
- **The brain:** `rusty-cli brain ask` on the merge and the rail's order returned nothing on this
  seam.

## UI proof
`script/e2e/782-one-home-page.sh`, under `compositor sway`, with Rusty on through its stand-in
`rusty-mcp` (seeded tasks and a decision due, as #679's scenario seeds them) and no path, so Default
shows:
- `782-01-home-rusty`: Home's top with the RUSTY card and Rusty's lists under Marley's cards; the
  status bar's right end with Home alone (REQ-001, REQ-005).
- `782-02-home-bottom`: Home scrolled down: TASKS, AGENT ACTIVITY, CONFIGURE (REQ-001, REQ-004).
- `782-03-rail-home-first`: after Home's New Terminal, the rail under Default: Home, then the
  terminal (REQ-008).
- `782-04-open-home`: the terminal's tab in front, then `rusty: open home` from the palette: Home in
  front (REQ-006).
- `782-05-verb-home`: the terminal in front, then a second launch handing over
  `zed://marley/rusty`: Home in front (REQ-007).
- `782-06-not-connected`: the stand-in swapped for one that fails and stopped: one RUSTY card with
  the reason (REQ-002).
- `782-07-rusty-off`: Rusty turned off in the run's settings: Home with no Rusty card (REQ-003).

## Locked-In Decisions
- D1 — Rusty's cards are a view of the `rusty` module (`RustyHome` becomes `RustyCards`, no longer
  an `Item`) that Home holds as a child entity. Home makes it when the switch turns on and drops it
  when it turns off; Home never calls Rusty's tools itself.
- D2 — The cards follow the switch (`rusty::is_on`), not the connection. While on and not connected,
  `RustyCards` draws one RUSTY card with the reason; Rusty's own refusal text is kept. Rusty
  appearing only once connected would make the cards vanish on every `rusty-mcp` restart, with no
  sign that Rusty is down.
- D3 — CONFIGURE moves to the bottom, under AGENT ACTIVITY: it is the card used least.
- D4 — `rusty: open home` opens Home with no readiness toast, since Home itself says whether Rusty is
  connected. The `rusty` verb keeps `rusty_unavailable`'s refusal, as every Rusty verb does.
- D5 — `card`, `row` and `muted` move from `rusty::home_tab` to `home_page`, and Home's fill
  observers (`fill_emptied` on a removed item, `fill_shown` on a group change) move from
  `rusty::home_tab::init` to `home_page::init`. Behavior is unchanged; the Rusty module keeps only
  what the switch turns on.
- D6 — `marley_rail::TabKind` gains `Home`. `rail.rs`'s `member_tabs` marks a `MarleyHome` item with
  it, `walk` lists a group's Home rows before its terminals, and the row is drawn as an `Other` tab
  (its icon, indent and close). It applies in whichever group holds Home.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is on and connected, the system shall show on Home, under RECENT PROJECTS and AGENTS AT WORK, the RUSTY card with Rusty's screens, Open Page… and the three captures, then RECENT PAGES, FOLLOW-UPS DUE and TASKS read from Rusty. | shots `782-01-home-rusty`, `782-02-home-bottom` |
| REQ-002 | WHILE Rusty is on and not connected, the system shall show one RUSTY card on Home that says Rusty is not connected and why, and no other Rusty card. | shot `782-06-not-connected` |
| REQ-003 | WHILE Rusty is off, the system shall show no Rusty card on Home. | shot `782-07-rusty-off` |
| REQ-004 | The system shall show Home's cards in the order START, NEW AGENT, RECENT PROJECTS beside AGENTS AT WORK, Rusty's cards, AGENT ACTIVITY, CONFIGURE. | shots `782-01-home-rusty`, `782-02-home-bottom` |
| REQ-005 | The system shall show no Rusty button in the status bar, whose right end holds Home. | shot `782-01-home-rusty` |
| REQ-006 | WHEN the user runs `rusty: open home`, the system shall bring Home forward in the shown group. | shot `782-04-open-home` |
| REQ-007 | WHEN `zed://marley/rusty` is handed to Marley while Rusty is on and connected, the system shall bring Home forward in the shown group. | shot `782-05-verb-home`; else the diff's review |
| REQ-008 | WHILE a group holds Home's page, the system shall list Home's row first under that group in the rail, above its terminals and Browser tabs. | shot `782-03-rail-home-first` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `home_page.rs`, `rusty/home_tab.rs`, `rusty.rs`, `status_buttons.rs`, `commands.rs`,
  `rail.rs`, `marley_workbench.rs`, `marley_rail.rs`; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG, the guide, the walkthrough, `rusty-in-marley.md` and the tutorial
  outline (§21), ledger capture (§19), close the ticket, archive, commit.
