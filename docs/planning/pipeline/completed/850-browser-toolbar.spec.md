---
pipeline_id: 9a83a94b-a8ec-4d55-b3d7-15efa55bd1af
ticket: docs/planning/tickets/open/TICKET-850-browser-tab-and-tools.md
status: Phase 4 — Complete PASS
title: The Browser toolbar as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Browser tab's toolbar reads as the Browser board draws it: the address box led by the
project's name and followed by where the page runs ("port 4180 · this machine"), and Pick,
Annotate and Record as labelled buttons in one framed group.

## Scope
### In
- `browser.rs` `render_toolbar`: the project chip and the place in the address box; the three
  buttons with their labels in a group.

### Out (explicitly deferred)
- The picks panel, the console drawer, held clicks, dialogs, Playwright scripts, recordings, take
  over and clearing a project's data (TICKET-879).

## Reference (§20)
N/A — Marley-specific: Marley's Browser tab (#488 onward); the Browser board.

### Prior art
- `render_toolbar`, `render_pick_button`, `render_annotate_button`, `render_record_button`.
- The brain (consultation fd9bc1dc): nothing on this seam.

## UI proof
Scenario `script/e2e/850-browser.sh` (sway): an offline Chromium, a page served in the run.
- `850-01-toolbar`: the project's name, the address, "port N · this machine", Pick, Annotate and
  Record labelled in their group (REQ-001).
- `850-02-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The place reads "port N · this machine" for 127.0.0.1 and localhost, the host otherwise,
  nothing for a page with no host.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Browser toolbar shall lead its address with the project's name, follow it with where the page runs, and offer Pick, Annotate and Record as labelled buttons in one group, in Marley Night and Marley Day. | `850-01-toolbar`, `850-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
