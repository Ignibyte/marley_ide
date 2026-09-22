# TICKET-070 — forge ticket row → copy '#N — title'

- **Forge ticket:** #70 `657f8e58-987c-49ae-8ec0-83f7d538c983` (feature, M2.C seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `9bade5c7-445f-41b6-a2d0-f2a8e716f0ee`
- **Pipeline doc:** ../../pipeline/active/ticket-copy.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
Click a forge ticket row (⌘⇧F overlay) to copy a pasteable `#N — title` ref to the clipboard (to feed an
agent). PURE: `ticket_ref`. SHIM: each row gets on_mouse_down(Left) → cx.write_to_clipboard (reuse #44).
Read-only. cov/MSI 100 on ticket_ref; the click is masked + self-test-verified. Deps #64 + #44.

## Acceptance
ticket_ref at cov/MSI 100 ("#N — title", U+2014); clicking a row copies the ref (self-test + pbpaste);
FULL gate GREEN. Full EARS in the spec.
