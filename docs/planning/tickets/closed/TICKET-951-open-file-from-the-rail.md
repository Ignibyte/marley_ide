# TICKET-951 — Open File… on the rail's +

- **Ticket:** LOCAL #951 (feature, the rail)
- **Owner:** 9f998252-2a54-4459-b037-d2e59b730208
- **Pipeline doc:** [951-open-file-from-the-rail.spec.md](../../pipeline/completed/951-open-file-from-the-rail.spec.md)
- **Source:** Chad, 2026-10-30: "Can we add a basic file viewer on the left + so i can just open a
  file inside the pane over there? files as in md etc." and, approving the plan, "zed is project
  oriented but marley goes up another level where its not. so i want to be able to open files
  outside a project"
- **Size:** S
- **Status:** closed

## Summary

A project's `+` (and the rail's right-click on empty space, for the Default group) gets Open
File…. It asks for any file on disk, not only the project's, and opens it in that project's pane: a Markdown
file as its formatted preview, anything else in the editor. The file's row appears under the
project's Files fold, and a file from outside the project names the folder it came from.

## Acceptance

Open File… on a project's `+` opens a Markdown file from outside the project as a formatted page
and a text file in the editor, each with a row under Files naming its folder; the empty space's
menu does the same in the Default group.
