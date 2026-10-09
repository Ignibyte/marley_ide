# TICKET-722 — The guide page caught up

- **Ticket:** LOCAL #722 (docs, the in-app guide)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [722-the-guide-page-caught-up.spec.md](../../pipeline/completed/722-the-guide-page-caught-up.spec.md)
- **Source ticket:** Chad, 2026-10-09: "lets spin up some documentation bots and update the marley
  html for knoweldge marley can use".
- **Status:** closed

## Summary
`crates/marley_workbench/guide/index.html`, the page `marley: open guide` and the title bar's `?`
show, stopped at #687 on 2026-10-07. Three documentation bots draft the areas that have changed
since, from `docs/marley/guide.md`, the tickets and the code:
- the rail, Home and Rusty (#697, #699 to #702);
- the agents and the harness (#684, #688 to #694, #696, #698, #709);
- Marley's tools for agents (#680, #689, #692, #703 to #707, #711).

Their fragments are merged into the page, and statements made wrong since are fixed: the Marley
agent is no longer "offered once", the tool families are more than four, and agent control is
more than "your client asks".

## Acceptance
The guide's contents filter finds the new articles, and each reads as what shipped.
