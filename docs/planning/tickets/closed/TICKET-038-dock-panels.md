# TICKET-038 — dock panels + titlebar styling

- **Forge ticket:** #38 `d6bf9b17-cc33-4cd1-8792-599806b52e22` (feature, M1.E — The Warp Look, seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `47b891b7-03fa-4399-b892-82f46c5217ab`
- **Pipeline doc:** ../../pipeline/active/dock-panels.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Status:** closed

## Summary
The docks are bare surface divs with a hardcoded "Files"/"Details" label. Give them panel treatment
(header + inner-edge divider + padding + elevation) and extract the label to a pure
`dock_title(DockSide)` (Left→Files, Right→Details). Confirm the root bg is `background` for a cohesive
window with the native "Marley" titlebar. The most shim-heavy ticket — pure surface is `dock_title`;
the panels are masked visual. Icons/real content deferred. Deps #34/#35 done.

## Acceptance
`dock_title` at cov 100/MSI 100 (both arms); the dock-panel render (masked 3-region visual — headers
+ dividers, chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
