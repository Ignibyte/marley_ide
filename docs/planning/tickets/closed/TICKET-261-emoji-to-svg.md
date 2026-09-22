# TICKET-261 — Kill the remaining 11 colorful emoji → clean-room SVG icons

- **Forge ticket:** #261 c8ab7eee-86ba-4213-90db-05d8f9e38f45 (chore, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** f54937d4-737f-443b-837c-9c10d01562bf
- **Pipeline doc:** ../../pipeline/active/261-emoji-to-svg.spec.md
- **Source ticket:** sprint #29 "M16 — Cleanup + Editor Frontier" (forge)
- **Status:** closed

## Summary
Convert the 11 colorful emoji the #232 audit found still rendering (overlay/
search headers ×7 emoji over 12 sites, `file_icon` ×4, `pane_icon` ×2 + its
two bucket-B arms) to the #137 vendored tintable SVG system: 10 new Icon
variants + 10 self-authored clean-room SVG assets (generic glyphs, no
language logos — §20), `file_icon`/`pane_icon` become Icon-returning pure
seams (#232 idiom), and the header sites swap the emoji prefix for a tinted
svg child (reusing sparkle/forge/files where the audit says so). Buckets B
(affordance/status sets) and the artist brand list stay out of scope.

## Acceptance
The mappings unit-pinned; every icon_path distinct + asset present; driven
captures show tinted SVG headers/rows/titles with zero colorful emoji; the
11 codepoints grep to zero in crates source. Full EARS in the spec.
