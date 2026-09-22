# TICKET-231 — Darker, Warp-like gray for panel/dock surfaces

- **Forge ticket:** #231 (e8e48112-9bf6-420a-a832-920018691593) (chore, M13)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018 (autonomous /work 228-237 run)
- **AAR:** ae52900d-1266-4d1e-96a3-ae2b73faf7f6
- **Pipeline doc:** ../../pipeline/active/warp-darker-surface.spec.md
- **Source ticket:** sprint #26 M13 "The Workspace Cockpit" (fd182395) — chad live feedback #3
- **Status:** closed

## Summary
The dark theme's `surface` (docks/sidebar/Files/panels/cards/inputs/overlays) is `hsla(0.62,0.09,0.155)` —
lighter than Warp's panels. Darken it to a Warp-like darker gray (chad #3). A single pure theme-value
change in `ui_components/lib.rs` re-tones every surface consistently; the light foreground/muted text's
contrast only improves. Bounded pure seam at cov/MSI 100.

## Acceptance
The dark panels/docks/sidebar render a darker gray (lower `surface` lightness), still distinct from the
near-black background + the border, and the light text stays legible (WCAG). Full EARS (REQ-001..004) in
the pipeline spec.
