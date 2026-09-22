# TICKET-266 — B2 render: uniform_list + StyledText.with_highlights

- **Forge ticket:** #266 4f9c9b01-73c8-442d-9eb6-d64c3b7563fd (feature, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 7f891f5a-8208-4474-889e-0c41264d288d
- **Pipeline doc:** ../../pipeline/completed/266-b2-render.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
The editable editor tab moves onto gpui's native render substrate:
uniform_list virtualizes over the real buffer (retiring the fixed 40-row
window), each row is one StyledText with syntax + selection as
with_highlights ranges (a pure, tested chunk→byte-range adapter feeds it),
the caret bar positions via TextLayout::position_for_index, clicks/drags
map via index_for_position (byte→CharOffset at the seam), and scrolling
runs through a UniformListScrollHandle. The #246 read-only pane keeps the
old render (branch fork); tree-sitter (#268) will feed the same highlight
substrate. All APIs verified in the vendored gpui 0.2.2 first.

## Acceptance
Adapter proven pure/mutation-clean; >40-line scrolling; typing/undo
unchanged; selection/caret/click correct on multibyte + tabs; the
read-only pane pixel-unchanged. Full EARS in the spec.
