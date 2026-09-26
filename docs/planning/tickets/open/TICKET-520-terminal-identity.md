# TICKET-520 — Each terminal knows its id, and Marley's tools know their caller

- **Ticket:** LOCAL #520 (feature, prong 2 C0 follow-on: the caller behind a tool call; prong 3: browser tools scoped to the caller's project)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/520-terminal-identity.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here". The survey ranks terminal identity second and names it with #519 as the tickets before #508 and #509 (`docs/orca_architecture/README.md`, "What Orca does well that Marley lacks, first", item 2; report 06 §2.5 and item 1; report 03 item 4).
- **Status:** open

## Summary
Marley's terminals carry nothing that names them, and the plugin's bridge forwards no caller, so an agent cannot ask for its own terminal's blocks and a browser tool that names no tab acts on the tab the user focused last, in any project. Every local Marley terminal gets `MARLEY_TERMINAL_ID` (a UUID of its own, kept through session restore) and `MARLEY_PROJECT` (its project's folder), replacing whatever it inherited. The bridge forwards them, with its working directory, as request headers; `marley_mcp` passes the caller to the app; `terminal_list` marks the caller's own terminal, the terminal tools default to it, and browser tools with no tab act in the caller's project.

## Acceptance
`printenv` in a Marley terminal shows its own id and project, a split gets a new id, and a restored terminal keeps its id; an agent in that terminal sees its row marked `self` in `terminal_list`; `browser_navigate` with no tab, called from project A while the user last looked at project B's tab, opens the page in A.
