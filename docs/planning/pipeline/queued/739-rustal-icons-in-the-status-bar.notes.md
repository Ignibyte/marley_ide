# Rustal icons in the status bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-739-rustal-icons-in-the-status-bar.md
- **Pipeline spec:** 739-rustal-icons-in-the-status-bar.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - No Marley status item exists; the Fleet icon is a dock button. Right items render reversed (`status_bar.rs:221`).
  - #679 replaced #672's row of screen buttons with the single Rusty button this removes.
  - #701: Home's page is Home's first tab and comes back when it closes; opened elsewhere it is an ordinary tab.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
