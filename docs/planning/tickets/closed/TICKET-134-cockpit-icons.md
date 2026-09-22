# TICKET-134 — the cockpit as top-bar icons (Details/Agents/Forge) [M7 seq-3]

- **Forge ticket:** #134 `696ea5d0-ccd2-4fa8-8a2b-89140e2f68d0` (feature, M7; sprint #18)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `aef07d46-eac7-4321-bcbf-fd806c876cc6`
- **Pipeline doc:** ../../pipeline/active/cockpit-icons.spec.md
- **Status:** closed

## Summary
`section_icon(RightSection)` (right_dock.rs, cov/MSI 100) + the #126 cockpit block renders the glyph instead
of the text label — the Details/Agents/Forge cockpit becomes a compact icon cluster in the top bar (no
overlapping text). Reuses top_tabs + the click handler. Deps #132 + M6 #126.

## Acceptance
section_icon at cov/MSI 100; the cockpit as icons in the top bar (live capture); FULL gate GREEN. Full EARS
in the spec.
