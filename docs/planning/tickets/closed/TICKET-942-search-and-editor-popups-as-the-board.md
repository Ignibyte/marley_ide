# TICKET-942 — Search and the editor's pop-ups as the board words them

- **Ticket:** LOCAL #942 (feature, the Marley look, search and the editor's pop-ups)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [942-search-and-editor-popups-as-the-board.spec.md](../../pipeline/completed/942-search-and-editor-popups-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/D-panels-editor.md, boards SearchMore and EditorPopups)
- **Boards:** SearchMore, EditorPopups
- **Size:** S–M
- **Status:** closed

## Summary

The SearchMore and EditorPopups boards use different words and a different first state from what
Marley shows. Project search hides its Replace field until it is toggled; the board shows it at
all times. Open Recent, the branch picker and the extension cards keep Zed's words ("Search
projects…", "Switch or type to create a branch…", "Upgrade"). Completions label each kind with a
single letter, where the board uses a short word ("fn", "fld"). An inline prediction has no note
saying which key takes it. An error shows only on hover, because Zed's inline diagnostics are off
by default. This ticket brings each one to the board, behind the look globals Marley already sets,
so the views stay Zed's when the globals are unset.

## Acceptance

Project search shows its Replace field; Open Recent, the branch picker and the extension cards use
the board's words; completions show kind words; a grey prediction has its "Tab takes the
prediction" note; an error shows at the line's end. Shot in Marley Night and Marley Day.
