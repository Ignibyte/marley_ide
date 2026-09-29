# TICKET-559 — Bookmarks on blocks, and find within a block

- **Ticket:** LOCAL #559 (feature, prong 1 T1: stage-one block actions; the Warp blocks note, recommendation 7)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/559-block-bookmarks-and-find-in-block.spec.md
- **Source ticket:** docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md ("Bookmarks and find within block (S each)")
- **Status:** closed

## Summary
Two of Warp's block actions that need no block selection. A bookmark is a mark on a block for the
session: Ctrl+Shift+B toggles it on the newest block in view, a Bookmark button on a hovered
block toggles any block, a bookmark icon sits before the pill of a marked block, ticks at the
terminal's right edge show where the marked blocks are in the scrollback, and Alt+Up and Alt+Down
jump between them. Find within a block limits Zed's terminal search to one block: a Find button
on a hovered block, or the palette action on the newest block in view, opens the search bar
scoped to that block's lines, the scoped block gets an outline, and closing the bar clears the
scope.

## Acceptance
Ctrl+Shift+B marks the newest block in view and Alt+Up and Alt+Down jump between marks; the
marked block shows the icon and the right edge shows a tick per mark; Find on a block scopes the
search bar to that block's lines and the count says so; Escape clears the scope; marks end with
the session. The full EARS criteria live in the pipeline spec.
