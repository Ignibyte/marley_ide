# TICKET-948 — The Knowledge panel as the board draws it

- **Ticket:** LOCAL #948 (feature, the Marley look, Knowledge)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** none yet
- **Source ticket:** split from #939 (its Out and D5); the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/F-tools.md, board Knowledge)
- **Boards:** Knowledge
- **Size:** M
- **Status:** open

## Summary

The Knowledge board draws the panel's search as "Search the brain, or ask it". Asking uses
`brain_ask` beside `brain_search`, and because an ask records a consultation it needs a decision
of its own. The Following card shows each page's type tile and its `#tag` without counts. The graph
keeps its settings in a fixed right column, not a floating card, with the legend in a row under the
graph and Link distance and Repel shown open. Marley's Knowledge panel draws none of these.

## Acceptance

The Knowledge panel's search offers asking the brain, its Following card and graph settings are
drawn as the Knowledge board draws them; shot in Marley Night and Marley Day.
