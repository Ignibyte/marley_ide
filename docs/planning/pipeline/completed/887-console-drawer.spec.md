---
pipeline_id: cb2e3ed5-d4ee-460a-ae42-d078968b8acb
ticket: docs/planning/tickets/open/TICKET-887-browser-console-and-tools.md
status: Phase 4 — Complete PASS
title: The Browser's console drawer, and its tools' buttons
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Browser tab gains a drawer under the page, as the Browser board draws it: Console with what the
page logged (time, level, message, where) and Network with what it fetched (method, status, URL,
time), opened from the toolbar's Console button, which counts the page's errors. A held click asks
with "Refuse · Esc" and a primary "Allow · Enter", and the page's own dialog answers with the
prompts' buttons.

## Scope
### In
- `browser.rs`: `Drawer { Console, Network }`, the toolbar's Console button, `render_drawer`;
  the pause bar's and the page dialog's buttons.

### Out (explicitly deferred)
- The drawer's Recordings tab, its Send to the agent, a pick's "Goes with it" and "To", and the
  download bar: new behaviour rather than a look, TICKET-892.

## Reference (§20)
N/A — Marley-specific: Marley's Browser tab and what it observes (#492's console and network);
the Browser and BrowserTools boards.

### Prior art
- `BrowserHub::console_entries`, `network_entries` (`marley_browser::observe`); the pause bar
  (#571), `render_dialog`; #870's `prompts::render_button`.
- The brain (consultation fad6d30f).

## UI proof
Scenario `script/e2e/887-console-drawer.sh` (sway): an offline Chromium; a page that logs a line,
warns, throws an error and fetches a missing file.
- `887-01-console`: the Console button with its count; the drawer's Console (REQ-001).
- `887-02-network`: the drawer's Network (REQ-001).
- `887-03-day`: Marley Day (REQ-001).
- `887-04-dialog`: a page's `confirm()`: Cancel and OK as the prompts' buttons (REQ-002).
- Not shot: the held click's buttons, which need an agent's click held by System One (#571's
  setup); the same `render_button` draws them.

## Locked-In Decisions
- D1 — The count is the page's errors and exceptions; none shows no number.
- D2 — The drawer is the tab's own: it stays open across the page's reloads, at 220 px.
- D3 — The newest entries are at the bottom, as the page logged them.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-002 | WHEN a page shows its own dialog, the tab shall answer it with the prompts' Cancel and a primary OK. | `887-04-dialog` |
| REQ-001 | WHEN the user opens the Browser tab's drawer, it shall show the page's console (time, level, message, where) and its requests (method, status, URL, time) on two tabs, the toolbar's button counting the page's errors, in Marley Night and Marley Day. | `887-01-console`, `887-02-network`, `887-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
