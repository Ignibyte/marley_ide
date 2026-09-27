# TICKET-581 — Clear a project's Browser data

- **Ticket:** LOCAL #581 (feature, prong 3, after #507)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/581-clear-a-projects-browser-data.spec.md (the design
  began as D6 and REQ-006 of #507's pair,
  `docs/planning/pipeline/completed/507-browser-context-per-project.spec.md`)
- **Source ticket:** split from TICKET-507 at its promotion, 2026-09-27
- **Status:** closed

## Summary
Since #507 each project has a Chromium of its own on a profile of its own, which keeps its
logins across restarts. Clear Browser Data resets one project. It asks first, naming the project
and saying that its Browser tabs close and every site in it signs out. It then closes the
project's Browser tabs, stops its Chromium unit, waits until systemd reports the unit inactive,
and deletes `profile/`, keeping `project.json`, so the project's next Browser tab starts a clean
Chromium. The action, `marley::ClearProjectBrowserData`, is on every workspace, and the rail's
project menu offers it as "Clear Browser Data…", beside #507's Remove Project. #507's
`browser::stop_chromium` already closes a project's Chromium over CDP and then stops its unit,
and `BrowserHub::stop_browser` forgets its pages; the clear builds on both.

## Acceptance
WHEN the user confirms Clear Browser Data for a project, the system shall close that project's
Browser tabs, stop its Chromium and delete its profile, so that the project's next Browser tab
starts signed out.
