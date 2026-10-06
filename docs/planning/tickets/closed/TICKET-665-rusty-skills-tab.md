# TICKET-665 — The Skills tab

- **Ticket:** LOCAL #665 (feature, Rusty in Marley R8)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** ../../pipeline/completed/665-rusty-skills-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** closed

## Summary
Rusty's skills store (`~/.rusty/skills`, a git repository) is served by `skill_list`, `skill_view`, `skill_create`, `skill_update`, `skill_delete`, `skill_scan`, `skill_approve`, `skill_reject`, and scripts by `script_list`, `script_view`, `script_update`, `script_run`. This ticket adds a Skills tab: pending skills first with Approve, Approve Anyway (after the scan's findings) and Reject; active skills and scripts; a skill or script opens in an editor with Save; New Skill; Delete (asked first); a script's Run in a Marley terminal, as Rusty's app runs it in a terminal. Over MCP only `skill_update` and `script_update` commit; the others' commits go to Rusty.

## Acceptance
A pending skill approved in the tab moves to the active list, an edit saved reaches the store, and a script's Run opens a terminal running it.
