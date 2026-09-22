# TICKET-427 — Multibuffer core: excerpts + the read-only stitched view (B-c step 3)

- **Ticket:** LOCAL #427 (feature, M32)
- **Pipeline doc:** ../../pipeline/active/427-multibuffer-core.spec.md
- **Tags:** editor, multibuffer, excerpts, b-c
- **Created:** 2026-08-14
- **Provenance:** roadmap B-c chain (m22-editing-bar.md item 16, first slice) under Chad's
  "/work 425 to 430" directive; shelf: `../../design-notes/display-map-shelf.md`
- **Status:** in-progress (pipeline `130b4894-b401-4516-8fcd-429750df7404`, promoted 2026-08-14)

## Summary

The first multi-file surface. A `MultiBuffer` model — an ordered sequence of
`Excerpt { file, buffer range + context }` over N underlying Buffers (half-open runs from
the start, AD-claude-305) — rendered as one stitched view: per-file header rows, syntax
highlighting via the shipped per-language pipeline, and jump-to-source (Enter/click on an
excerpt row opens the real file at that line, through the reveal-hook discipline of
PR-#305). Read-only in this slice: typing routes nothing (428 makes it editable). The
consumer that makes it real UI: ⌘⇧F grows an "open results as multibuffer" action that
materializes the current search hits (with context lines) into the new surface as a pane
content kind (the #394 content-registry shape). Zed's §6 is the behavior reference — the
excerpt tree, headers-as-blocks, the singleton unification — adopted at Marley's
smaller per-line shape (contract over container; no SumTree obligation).

## Acceptance

Headline: from a ⌘⇧F result set, one action opens a stitched view showing every hit's
excerpt under its file header; excerpts are syntax-highlighted and scroll as one surface;
Enter/click jumps to the true file:line; the surface persists/closes like any pane content.
Full EARS at promotion.

## React-first (parity)

UI-affecting — the multibuffer surface is a NEW Zone A row designed React-first: a
`MultibufferView` in the POC (excerpt headers, context rows, jump affordance), screenshot +
read the PNG, then port 1:1; MARLEY-PARITY.md gains the row.
