---
pipeline_id: 35fce306-2ac0-4148-8626-e36b0b5f3a14
ticket: docs/planning/tickets/closed/TICKET-807-marley-rusty-and-home-icons.md
status: Phase 4 — Complete PASS
title: Marley's, Rusty's and Home's own icons
type: feature
slice: the Marley layout (Home #701/#782, the Marley agent #683, Rusty #696)
references:
  - agent-icons/README.md
---

## Title
Chad's three icons take the places of Zed's: a house for Home, a cat for the Marley agent and a
butler for Rusty.

## Scope
### In
- Three SVGs into Zed's icon set: `assets/icons/marley_home.svg`, `marley_cat.svg`,
  `rusty_butler.svg` (from `agent-icons/home.svg`, `marley-cat.svg`, `rusty-butler.svg`,
  unchanged: 16 × 16, black strokes and fills that the renderer tints), and their names in
  `IconName` (`MarleyHome`, `MarleyCat`, `RustyButler`).
- Home: its tab, its rail row (the tab's icon) and the status bar's Home button.
- The Marley agent: the title bar's Marley button; a Marley thread's tab, rail row and New Agent
  entry.
- Rusty: the title bar's Rusty button (`RUSTY_ICON`); a Rusty thread's tab, rail row and New Agent
  entry.

### Out (explicitly deferred)
- The Default group's header icon (a projectless group's, #600), Home's cards, the generic agent
  sparkle (Agents at Work, a Browser tab's agent mark) and the PNG copies.

## Reference (§20)
Upstream Zed: `crates/icons` (`IconName`, one SVG per name under `assets/icons/`, tinted by the
theme), used as it is.

### Prior art
- The code we ship: `IconName::path`; `agents::thread_icon` (the picker and the rail's thread
  rows); `ThreadTab::tab_icon`; `status_buttons`; `HomePage::tab_icon`.
- Behavior maps, published material: none on this seam.

## UI proof
Scenario `script/e2e/807-marley-rusty-and-home-icons.sh`: `807-01-title-bar` (the Marley and
Rusty buttons, Home in the status bar, REQ-002, REQ-003); `807-02-home` (Home's tab and rail row,
REQ-001); `807-03-picker` (the New Agent picker's Marley and Rusty entries, REQ-002, REQ-003);
`807-04-thread` (a Marley thread's tab and row, REQ-002); `807-05-light` (a light theme: the icons
tinted, REQ-004).

## Locked-In Decisions
- **D1 — Zed's icon set, not files read at run time**, so the icons tint with the theme like every
  other icon.
- **D2 — Only the places that mean Home, Marley or Rusty** change; the generic sparkle stays.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The system shall draw the house icon for Home's tab, its rail row and the status bar's Home button | shots `807-01`, `807-02` |
| REQ-002 | The system shall draw the cat icon for the title bar's Marley button and for a Marley thread's tab, rail row and New Agent entry | shots `807-01`, `807-03`, `807-04` |
| REQ-003 | The system shall draw the butler icon for the title bar's Rusty button and a Rusty thread's tab, rail row and New Agent entry | shots `807-01`, `807-03` |
| REQ-004 | The icons shall take the theme's colors, as Zed's do | shot `807-05-light` |

## Phase Plan
- **P1 Plan:** this spec. **P2 Code:** the SVGs, the names, the call sites; gate.
- **P3 Test:** the scenario above. **P4 Complete:** CHANGELOG, guide, Feature Map card, close, commit.
