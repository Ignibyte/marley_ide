# TICKET-799 — Marley components and the Component Library

- **Ticket:** LOCAL #799 (feature, agent components)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/799-marley-components-and-the-component-library.spec.md
- **Source ticket:** Chad, 2026-10-17: "Lets build a gpui component system where if you are talking to an agent in the zed agent panel it can be provided tools that would build tables and stuff like that. In the settings or from the home we could see the component library."
- **Status:** open

## Summary
A new crate, `marley_components`, holds a catalog of components an agent can ask for: text,
heading, row, column, card, table, key-value list, stat, badge, progress, bar chart, sparkline,
code and callout. A component is described as a nested JSON tree, checked against the catalog,
and drawn natively with Zed's `ui` pieces in the user's theme. A Component Library tab shows
every component drawn from its example, with the JSON an agent sends for it. It opens from
Home's Configure card, from Marley's Settings page and from the palette. The research and the
choice of format are in `docs/planning/design-notes/agent-components-2026-10-17.md`.

## Acceptance
The Component Library opens from all three places and shows each catalog component drawn in the
theme beside its JSON; a malformed example shows its error in place of the component.
