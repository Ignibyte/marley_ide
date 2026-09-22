# TICKET-341 — The editor's horizontal scrollbar thumb (deferred from #336)

- **Forge ticket:** #341 `ccdf9d4a-69cb-4168-b697-9e682eee8f18` (feature, M22)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `e6764acf-6091-425a-b395-988670972188`
- **Pipeline doc:** ../../pipeline/active/341-editor-hscroll-thumb.spec.md
- **Source:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #336 follow-up)
- **Status:** closed

## Summary

#336 shipped editor horizontal scroll (wheel + caret-follow + click) but DEFERRED the visible thumb (REQ-008).
This ships it as a display-only affordance: a bottom-edge bar over the code column, positioned + sized from the
pure scroll fractions, hidden when the code fits. Reuses `viewport::scrollbar_thumb` but over the **virtual
extent** `viewport_px + max_scroll_x` so #336's overscroll slack doesn't overflow the track. Drag-to-scroll is
a separate follow-up (zero drag precedent — #198 deferred it too).

## Acceptance

A horizontal thumb appears over the code column when the code overflows, with its length proportional to
visible/total and its position tracking `scroll_x` (right edge at the track edge at max scroll); no thumb when
the code fits; the #336 wheel/caret/click paths unchanged. Full EARS (REQ-001..005) in the pipeline spec.
Verified headlessly (live pixel deferred — chad at the machine).
