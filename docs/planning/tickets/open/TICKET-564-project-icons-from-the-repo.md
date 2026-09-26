# TICKET-564 — Project icons from the repository's own files

- **Ticket:** LOCAL #564 (feature, workbench: the rail's project headers)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/564-project-icons-from-the-repo.spec.md
- **Source ticket:** The Orca second pass of 2026-09-25 (`docs/planning/design-notes/orca-second-pass-2026-09-25.md`), the five smaller details, item 5; Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** open

## Summary
Every project header in the rail is a name. With several projects and their worktrees open, an
icon tells them apart faster than reading. Orca finds a repository's own icon: a favicon or logo
at the usual paths, or the icon its `index.html` declares. Marley reads the same files, once per
project off the main thread and again when one of them changes, decodes them with the `image`
crate gpui already builds, and draws the icon at 16 px before the project's name; a worktree row
(#510) shares its project's icon. Nothing is fetched: Orca's trips to Google's favicon service
and GitHub's avatars are left out. A project with no icon file looks as it does today.

## Acceptance
A project with `favicon.png` at its root shows it before its name; one whose `index.html`
declares `<link rel="icon" href="assets/mark.svg">` shows that SVG; one with no icon file shows
the header as today; a favicon added while Marley runs appears within a few seconds; a file
over the size cap or one that is not an image is ignored with one log line; Marley makes no
network request for icons.
