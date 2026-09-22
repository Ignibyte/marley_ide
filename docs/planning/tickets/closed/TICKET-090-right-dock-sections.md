# TICKET-090 — right-dock sections: Details / Agents / Forge tabs

- **Forge ticket:** #90 `d9ff256c-ea0c-4130-9ae9-626688d6c8ef` (feature, M2.F seq-1 FOUNDATION; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `0423389e-798b-4e91-a1e9-98ca9f99a8b4`
- **Pipeline doc:** ../../pipeline/active/right-dock-sections.spec.md
- **Status:** closed

## Summary
Tabbed right dock (Details/Agents/Forge). PURE `right_dock.rs` (`RightSection` + `section_tabs`/
`section_label`, cov/MSI 100); the SHIM adds `right_section` + a tab strip + body switch; Details keeps #58,
Agents/Forge placeholders (#91/#92). Deps #58 + #38 + #35.

## Acceptance
section_tabs/section_label at cov/MSI 100; the dock shows 3 tabs + switches on click (self-test/engine);
FULL gate GREEN. Full EARS in the spec.
