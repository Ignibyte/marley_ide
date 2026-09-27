# TICKET-507 — A Chromium and a profile per project

- **Ticket:** LOCAL #507 (feature, prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/507-browser-context-per-project.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 4, second half, of the list after the browser waves). Chad's decision the same day on the Orca survey's open question 2: one Chromium per project, worktrees sharing their project's profile (`docs/orca_architecture/README.md`, "Open questions for Chad" 2; report 03 §2.8 and §3 item 3).
- **Status:** closed

## Summary
Every Browser tab shares one Chromium profile, so a login made for one project is a login for all of them, and two projects on localhost trample each other's cookies. The ticket first planned a CDP browser context per project with a cookie jar Marley would keep. Report 03 found that such contexts are off the record: cookies could be saved and restored, but `localStorage` and IndexedDB, where Supabase and Firebase keep their logins, die at a restart. So Chad chose one Chromium per project. Each project gets its own Chromium unit and profile directory, keyed on the project's main worktree paths, which is how Zed already groups a linked worktree under its repository, so worktrees share their project's profile. A project's Chromium starts with its first Browser tab and stops when the project is removed from Marley; the single profile of earlier builds becomes the first project's. Clear Browser Data, which resets one project, moved to #581 at promotion.

## Acceptance
Tabs of two projects keep separate cookies and localStorage, and both survive a restart of Marley and of the projects' Chromiums; a worktree's tabs share its project's login; removing a project stops its Chromium; the old profile's logins arrive in the first project whose browser starts. (Clear Browser Data signing one project out is #581's.)
