# TICKET-198 — Pane scrollbar thumb + jump-to-bottom

- **Forge ticket:** #198 (a71a6924-b6d2-47f6-a45b-3734defc3ffb) (feature, M12.2, terminal/scroll)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** f61dd6f9-6679-4a61-b2e0-9f3ca05fd361
- **Pipeline doc:** ../../pipeline/active/warp-scrollbar.spec.md
- **Source ticket:** M12.2 sprint #25 (the polish batch, after the warp-parity thread)
- **Status:** closed

## Summary
A scrolled-up pane has no scroll indicator + no quick way back to the bottom. Add a scrollbar thumb
(position + proportional height, via a pure `scrollbar_thumb` over the viewport row-accounting) on the
pane's right edge, and a jump-to-bottom affordance (via a pure `at_bottom` predicate) shown while scrolled
up, whose click re-anchors to the latest output (reusing the tested `scroll_down` follow mechanism).
Clean-room — tokens only.

## Acceptance
A thumb appears + tracks the scroll position when content overflows (none when it fits); a jump-to-bottom
button appears when scrolled up and returns to the bottom on click. Full EARS (REQ-001..005) in the spec.
