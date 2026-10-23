---
pipeline_id: db6fefe0-15d7-4a11-a674-70245236bcad
ticket: docs/planning/tickets/open/TICKET-903-home-as-the-board-draws-it.md
status: Phase 4 — Complete PASS
title: Home as the Home board draws it
type: feature
slice: The Marley look, W2.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
Home laid out as the Home board: a centered column with "What are we working on?" over the
prompt, then a wrapping grid of cards, then the configure links.

## Scope
### In
- `home_page.rs`: the heading; the prompt as a rounded box two lines tall ("Ask Marley, or
  describe a task and pick who does it", Enter starts), the project chip first with the project's
  name, the agent CLIs as pills with a glyph tile, a round accent Start with an arrow; the start
  actions as pills (New Terminal, Open Folder…, Clone Repository…, Reopen a Thread, and Today's
  Note while Rusty is on); the cards in a grid that wraps at 300 px, each titled in capitals with
  its link at the right (Agents at work · Threads, Recent projects · Open Folder…, Agent activity ·
  Open); agents at work as tiles with a state dot over a status line; the configure links as a
  centered row under the grid.
- `rusty/home_tab.rs`: Rusty's cards handed to Home one by one so they flow in its grid: Tasks
  (· Open Tasks) with each task's list as a pill; Follow-ups due (with how many are overdue) with
  Recent pages under a rule; Rusty's screens as a grid of tiles, the captures under them.

### Out (explicitly deferred)
- Marley and Rusty as prompt chips: #904.
- Ticking a task done from Home: new behaviour, not on any ticket.

## Reference (§20)
N/A — Marley-specific: Home's page (#701, #835, #867) and Rusty's cards (#782).

### Prior art
- `MarleyHome::render`, `card`, `render_prompt`, `render_project_chip`, `render_at_work`,
  `render_recent`, `render_activity`, `render_configure`; `RustyCards` and its `render_*`.
- The Home board (`docs/design/marley-canvas-2026-10-20/project/Home.dc.html` in the design
  record).
- The brain (consultation e398c080): nothing on this seam.

## UI proof
Scenario `script/e2e/903-home-as-the-board-draws-it.sh` (sway): a scratch project, stand-in agent
CLIs, Rusty on through its stand-in where the run allows.
- `903-01-night`: Home, Rusty off (REQ-001).
- `903-02-rusty`: Home with Rusty's cards in the grid (REQ-001, REQ-002).
- `903-03-day`: Home in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Every action Home had stays; Command Palette and New Agent… leave Home (the title bar's
  palette field and the agent chips cover them).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Home shall draw the heading, the two-line prompt with the project chip, agent pills and Start, the start pills, the cards in a wrapping grid with their links, and the configure links, in Marley Night and Marley Day. | `903-01-night`, `903-03-day` |
| REQ-002 | WHILE Rusty is on, its tasks, follow-ups with recent pages, and screens shall be cards in Home's grid. | `903-02-rusty` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
