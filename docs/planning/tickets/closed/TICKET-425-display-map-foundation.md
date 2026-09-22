# TICKET-425 — The display-map foundation (B-c step 1)

- **Ticket:** LOCAL #425 (feature, M32)
- **Pipeline doc:** ../../pipeline/completed/425-display-map-foundation.spec.md
- **Tags:** editor, display-map, b-c, refactor, foundation
- **Created:** 2026-08-14
- **Provenance:** minted from the roadmap's B-c chain (m22-editing-bar.md item 13) under
  Chad's "/work 425 to 430" directive; shelf: `../../design-notes/display-map-shelf.md`
- **Status:** closed (2026-08-14 — shipped: the DisplayMap facade + BufferRow/DisplayRow typed spaces; 16 sites re-routed; byte-identical proven (equivalence tests + 2174-suite unchanged + live drive); GATE GREEN [diff] 15/15)

## Summary

Today the buffer→display transform is three shipped-but-separate pieces: `FoldProjection`
(#305 — buffer row ↔ visible slot, half-open hidden runs, converted once at the
`uniform_list` rim), the #331 phantom/inlay column layer inside `code_view::LineLayout`
(`col_starts` + the two-boundary accessors), and the #336 h-scroll clipper/shift split.
Soft wrap (#426) breaks row↔line identity and the multibuffer (#427+) breaks row↔buffer
identity — neither can be bolted onto the ad-hoc trio. #425 introduces the ONE composable
coordinate stack (`display map`): typed row/point spaces, a uniform layer contract
(Marley-shaped: transform runs + bidirectional conversion + an invalidation-aware sync),
`FoldProjection` re-homed as its first layer, and the rim conversion sites routed through
the stack. **Byte-identical UI** — zero visible change; the existing suite must pass
unmodified (any test edit is a red flag at inspect), plus a new unit surface on the stack.
Keep AD-claude-two-boundary-maps (code spans hug the code, caret ranges track the caret)
EXPOSED in the stack's API; prefer half-open runs (AD-claude-305); probe the byte-identical
claim at zero-width boundaries (PR #333); `cargo check --tests` after every signature move
(F-#386).

## Acceptance

Headline: the fold pipeline (render, caret, click, jump, auto-reveal, sticky header,
h-scroll) runs through the new display-map stack with the full pre-425 test suite green
UNCHANGED; the stack's own unit tests pin bidirectional conversion at run boundaries; MSI
100 on the new surface. Full EARS in the pipeline spec at promotion.

## React-first (parity)

N/A — no UI delta: a byte-identical internal refactor (the shelf note records the posture).
