# TICKET-676 — Home takes the screens that belong to no project

- **Ticket:** LOCAL #676 (feature, the rail)
- **Owner:** claude-opus-5-5, 2026-10-07 (Chad's request)
- **Pipeline doc:** ../../pipeline/completed/676-home-takes-what-has-no-project.spec.md
- **Source ticket:** `docs/planning/intake/rail-and-center-tabs.md` (decision 4), #600 (the Home
  group), #675 (the Rusty group)
- **Status:** closed

## Summary
Chad, 2026-10-07: "right now if i right click and do a new terminal without being in a project it
opens in a home folder. I think maybe anything that doesnt fit a category lands there?" Asked to
confirm the rule (projects take what is theirs, Rusty takes Rusty's, Home takes the rest): "ok".
Three screens belong to no project and are not Rusty's: the fleet's Agent tab (#609), System One
calls (#659) and a harness session's tab (#534). Each opens in whatever project the window shows;
each now opens in the window's Home group, made on first use, and the window shows Home.

## Acceptance
Opening the Agent tab, System One calls or a harness session's tab in the Marley layout opens it
in the Home group, making Home when the window has none, and shows Home; a project click goes back
with the tab kept under Home.
