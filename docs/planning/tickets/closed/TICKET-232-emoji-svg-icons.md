# TICKET-232 — Replace remaining emoji/glyphs with clean-room SVG icons + emoji→icon audit

- **Forge ticket:** #232 (9e62e2a0-96e7-485c-b07e-0e2fcc9c36c9) (chore, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 42d78ef3-f433-44fe-b773-da667a1b53f5
- **Pipeline doc:** ../../pipeline/active/emoji-svg-icons.spec.md
- **Source ticket:** M13 "The Workspace Cockpit" sprint #26 (chad live feedback #1)
- **Status:** closed

## Summary
chad feedback #1 ("remove all emojis for icons" + "a list of icons an artist should make"). The top
bar + cockpit are ALREADY real SVG icons (#137); this ticket converts the most-visible remaining
rendered glyphs — the per-block **status indicator** (○ ✓ ✗) and the **↻ run** affordance — to
clean-room-authored SVG icons, and delivers the emoji→icon **audit** + the **brand-icon list**. The
internal-overlay glyphs (Forge overlay, agent-status, file-type) are catalogued + deferred to a
follow-up so this slice stays focused. Fold/disclosure chevrons (▸ ▾) are UI primitives — kept.

## Acceptance
The block-status indicator + the run affordance render themed SVG icons (not text glyphs); the pure
`status_indicator`→`(Icon, Hsla)` + the extended `icon_path` are cov/MSI 100; the 4 new SVGs are
clean-room 24×24 monochrome silhouettes; and the audit doc catalogues every glyph + splits the
standard vs brand-artist icon sets. Full EARS criteria in the pipeline spec.
