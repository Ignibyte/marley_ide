# TICKET-426 — Soft wrap (B-c step 2)

- **Ticket:** LOCAL #426 (feature, M32)
- **Pipeline doc:** ../../pipeline/completed/426-soft-wrap.spec.md
- **Tags:** editor, display-map, soft-wrap, b-c
- **Created:** 2026-08-14
- **Provenance:** roadmap B-c chain (m22-editing-bar.md item 14) under Chad's
  "/work 425 to 430" directive; shelf: `../../design-notes/display-map-shelf.md`
- **Status:** closed (2026-08-14 — shipped: wrap as the display map's second layer; segment rim; display-row ↑/↓; h-scroll inert ON / byte-identical OFF; 2193-suite + live drive + parity pair; GATE GREEN [diff] 15/15)

## Summary

Long lines currently clip and reach their tails only via #336's h-scroll. #426 adds soft
wrap as the second display-map layer (#425's stack): one buffer line renders as N display
rows at the pane's cell width, wrapping on word boundaries with a continuation indent
(reference behavior: Zed's wrap; the exact policy is a design-phase decision from the
observed captures). `editor.soft_wrap` setting (default a design decision; OFF must be
byte-identical) + a palette toggle. Caret/click/selection/motion/scroll/gutter all speak
the new row space: line numbers render on a line's FIRST display row only; vertical motion
moves by display row (the Zed movement model). The #336 AD is BINDING: wrap breaks the
one-row-one-line clipper assumption, so the shift site is revisited, not layered on —
wrap-ON disables h-scroll (nothing clips), and the rejected `Unconstrained` road gets
re-weighed now that the layout premise changed. Watch the F-#305/F-#352 classes: every
caret-placement path must reveal/scroll in the RIGHT row domain (`cargo check --tests`
after signature moves).

## Acceptance

Headline: with wrap ON, no horizontal clipping exists at any pane width (the #336
unreachable-tail scenario renders fully); caret/click/selection land on the same char
before and after wrap of the same content; OFF is byte-identical to pre-426 (probed at
zero-width boundaries per the PR); toggle is live without restart. Full EARS at promotion.

## React-first (parity)

UI-affecting — the editor surface's wrap behavior is designed in the POC
(`components/EditorView.tsx`) first: wrap the fixture doc at the pane width with the same
first-row-only gutter policy, screenshot + read the PNG, then port 1:1.
