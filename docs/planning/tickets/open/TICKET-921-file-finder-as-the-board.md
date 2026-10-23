# TICKET-921 — The file finder as the board draws it

- **Ticket:** LOCAL #921 (feature, the Marley look, file finder)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [921-file-finder-as-the-board.spec.md](../../pipeline/queued/921-file-finder-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, boards Palette), build order line 8
- **Boards:** Palette (its "Go to a file" panel)
- **Size:** M
- **Status:** open

## Summary

The file finder still draws Zed's rows: a file icon, the name and the path on one line, and no
git status. The Palette board draws each row with the file's extension in a small mono slot, the
name over its path on two lines, and a git badge ("M") at the right. Its field ends with the key
that opens it ("Ctrl P"), and its preview is headed by the file's path and how many lines it has.
This ticket draws the finder that way in the Marley layout. Zed's layout keeps Zed's finder.

## Acceptance

In the Marley layout the file finder's rows show the extension, the name over the path and a git
badge on a changed file; the field shows the finder's key; the preview is headed "path · N lines".
Shot in Marley Night and Marley Day.
