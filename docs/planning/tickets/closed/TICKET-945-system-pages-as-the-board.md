# TICKET-945 — The system pages as the board draws them

- **Ticket:** LOCAL #945 (feature, the Marley look, the System tab)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [945-system-pages-as-the-board.spec.md](../../pipeline/completed/945-system-pages-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/G-system.md, board SystemPages)
- **Boards:** SystemPages
- **Size:** M
- **Status:** closed

## Summary

The System tab (#855, #885) has all nine pages, but four of them say less than the SystemPages
board. Install and update shows fixed sentences, not the last install's steps, and never notices
that a newer build was installed while this one runs. Marley's MCP endpoint has no bridge, tool
count, last refusal or limits, and no buttons. Marley's Chromium has no version, no count of the
sites a profile keeps cookies for, and no limits row. Shell integration, Fleet providers and For
developers have no buttons. This ticket fills those rows and buttons. `script/install-marley`
writes a small record of each install so the page can show its steps.

## Acceptance

Install and update lists the last install's steps and warns when the running build is older than
the installed one; the MCP endpoint page shows its bridge, tools, last refusal and limits with Open
Agent Activity and Copy the endpoint; Chromium shows its version, cookies per profile and limits;
the other pages have their buttons. Shot in Marley Night and Marley Day.
