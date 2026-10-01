# TICKET-616 — Delete and unarchive threads from the rail

- **Ticket:** LOCAL #616 (feature, workbench shell, the rail's threads (after #605))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/616-delete-and-unarchive-threads-from-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
#605 archives a thread from its rail row. Deleting one, and bringing an archived thread back, still need Zed's archive view. The rail's thread menu gains Delete Thread (confirmed), and a project's menu opens its archived threads with Unarchive.

## Acceptance
WHEN the user deletes a thread from its rail row and confirms, Marley shall delete it; WHEN the user unarchives an archived thread from the rail, it shall be listed again.
