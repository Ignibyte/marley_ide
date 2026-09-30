# TICKET-606 — List a window's closed projects in the rail

- **Ticket:** LOCAL #606 (feature, workbench shell: the rail)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/606-closed-projects-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-30, approving the follow-up to #601's Test finding: "lets go ahead and make that ticket and build it as well".
- **Status:** closed

## Summary
After a restart, Zed reopens only the workspace a window showed and brings its other projects back
as project groups with no workspace. The rail lists only groups with an open workspace, so those
projects vanish from the rail until they are added again. They should stay listed, dimmed, and
open on a click, as Zed's own Threads Sidebar lists and opens them.

## Acceptance
Two projects in a window, one shown at the quit: after a restart both are listed, the other
dimmed; a click on the dimmed one opens it with its terminals, and it lists as before.
