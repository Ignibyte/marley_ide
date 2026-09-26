# TICKET-574 — Browser tools act in the caller's project

- **Ticket:** LOCAL #574 (feature, prong 2, after #520's slice 1)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/574-browser-tools-in-the-callers-project.spec.md
- **Source ticket:** cut from TICKET-520 at its promotion, 2026-09-26, to keep the slice one piece
- **Status:** closed

## Summary
#520's slice 1 gives each Marley terminal an id and a project and hands the caller to the app (`AppCall::caller`). Here the browser tools use it: a tool that names no tab acts on the tab the user focused last in the caller's project (its terminal's project group, else `Marley-Project`, else `Marley-Cwd`, matched against each group's workspaces' own folders, longest wins), `browser_navigate` with none opens a new tab there when the project has none, and `browser_tabs` gives each tab's project and marks the one a call with no tab would act on. An Agent Panel agent is placed by its bridge's folder, the project root Zed starts it in. The design is #520's notes, items 9 and 10 (the hub's focus history, `focused_in`, `caller_project`, `place_tab`), and its queued REQ-007 and REQ-008.

## Acceptance
With two projects open, an agent in one project's terminal that calls `browser_navigate` with no tab opens its page in its own project's Browser tab, and the other project's tab keeps its page; an Agent Panel agent does the same for its thread's project; a caller with no project keeps today's behavior.
