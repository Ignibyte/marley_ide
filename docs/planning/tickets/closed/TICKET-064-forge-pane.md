# TICKET-064 — Forge pane: the current sprint's tickets

- **Forge ticket:** #64 `63c1376d-52aa-4c20-9a05-e431215a6728` (feature, M2.B seq-6)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `602bf407-9035-4eab-9084-8df25478928e`
- **Pipeline doc:** ../../pipeline/active/forge-pane.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
The visible cockpit surface: cmd-shift-f toggles a Forge overlay showing the current sprint + its
tickets, read live from marley_forge_client (#63). PURE: `status_glyph` + `sprint_rows`/`TicketRow`
(forge_view.rs) + the keymap. SHIM: a best-effort startup fetch (.mcp.json → ForgeClient) into
`RootView.forge_sprint` + the overlay render. cov/MSI 100 on the pure parts; the overlay is masked +
self-test-verified. Deps #63. **Closes M2.B.**

## Acceptance
status_glyph + sprint_rows + keymap at cov/MSI 100 (each status→glyph; multi-ticket→rows; empty→empty;
cmd-shift-f→toggle-forge); cmd-shift-f shows the live sprint (self-test capture); FULL gate GREEN. Full
EARS in the pipeline spec.
