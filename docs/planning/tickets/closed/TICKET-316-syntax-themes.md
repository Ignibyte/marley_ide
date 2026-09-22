# TICKET-316 — The syntax theme system: a per-theme SyntaxPalette, contrast-proven by test

- **Forge ticket:** #316 7f9d185e-ca50-4026-b548-5546864113a1 (feature, M20)
- **Owner:** claude (session 99b5bc91)
- **AAR:** 71a05e24-d2f5-4372-a8c7-4c87cbcbc1bb
- **Pipeline doc:** ../../pipeline/active/316-syntax-themes.spec.md
- **Source ticket:** M20 language intelligence (#308-317); the hard-ordered follow-up to #315 (the taxonomy this themes)
- **Status:** closed

## Summary
Switch Marley's theme and the chrome restyles, but code keeps one hardcoded look — after #315 that is ~6 fixed `gpui::hsla` literals in `token_color` (Str/Number + #315's Function/Type/Attribute/Property), identical in Light and Dark. This gives every theme its own `SyntaxPalette` over the #315 10-slot taxonomy, tuned per built-in theme and **contrast-proven by test** (each slot ≥AA vs the theme background) — a palette typo cannot ship illegible code. The live switch is free: the syntax cache stores kinds, not colors, so a theme flip repaints correct by construction.

## Acceptance
Every `TokenKind` resolves through the active theme's palette (total; Plain→foreground); every (theme × slot) pair ≥4.5:1 (WCAG AA) vs background, both built-ins + the derived default; no hardcoded token-color literal survives outside the palette (the ~6 die); a theme switch recolors a known span in the same frame. Full EARS criteria in the pipeline spec.
