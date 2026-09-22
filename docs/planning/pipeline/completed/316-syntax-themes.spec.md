---
pipeline_id: 6d0401a3-0ec4-4ce7-bc47-4d63db0e052f
ticket: forge#316 (7f9d185e-ca50-4026-b548-5546864113a1) · local docs/planning/tickets/open/TICKET-316-syntax-themes.md
aar_id: 71a05e24-d2f5-4372-a8c7-4c87cbcbc1bb
status: Phase 5 — Complete PASS
title: The syntax theme system — a per-theme SyntaxPalette, contrast-proven by test
type: feature
milestone: M20
references: [token_color TODAY: Str/Number are HARDCODED hsla literals identical across themes (app.rs:801-806; its own doc says Hint "is what #316's theme palette will pull apart"), the WCAG helpers ALREADY SHIP: relative_luminance + contrast_ratio (ui_components/src/lib.rs:128/:142, with ≥7.0 fg/bg tests at :248), ThemeRegistry::builtin = exactly TWO themes (Marley Light/Dark, themes.rs:35), NO SyntaxPalette type exists (grep-confirmed), THE DISSOLVE: the syntax cache stores KINDS not colors (SyntaxLines = (Range, TokenKind) — app.rs:627), so a theme switch needs NO cache invalidation, #315's 10-slot taxonomy (HARD DEP), the #199 set_theme/live-picker path (app.rs:2024)]
---

## Title
Switch Marley's theme and the chrome restyles — but code keeps one hardcoded look: `Str` green and
`Number` orange are fixed hsla literals identical in Light and Dark. This gives every theme its own
`SyntaxPalette` over the #315 taxonomy, tuned per built-in theme and **contrast-proven by test** — a
palette typo cannot ship illegible code.

**A worry the recon DISSOLVED (record it, don't design for it):** the ticket description feared a
stale-color cache on theme switch ("the highlight cache keys on theme identity or is invalidated").
False — the syntax cache stores `(Range, TokenKind)` KINDS; colors resolve per-frame in `token_color`.
A theme switch repaints and is correct by construction. Nothing to invalidate, no design needed.

## Scope
### In
- **`SyntaxPalette` (in `marley_ui_components`, next to `ThemeColors`):** one `Hsla` per taxonomy slot
  (Keyword, Function, Type, Str, Number, Comment, Attribute, Punctuation, Property) + a TOTAL
  `color_for(kind) -> Hsla` (Plain → the theme's `foreground`; total = no panic, no Option). `Theme`
  (themes.rs) carries one.
- **Tuned palettes for BOTH built-ins** (Marley Light, Marley Dark — the registry's whole population
  today): each a deliberate pass, not a derived default. The two hardcoded literals in `token_color`
  DIE; `token_color(kind, theme)` resolves through the ACTIVE theme's palette — it is already the ONE
  color seam (the editor rows, the #331 hover fence, and the #246 split pane all route through it;
  recon-verified), so consumers unify by construction, and a grep-gate pins that no orphaned literal
  token color survives.
- **The derived-default generator (small, pure):** a theme WITHOUT an explicit palette derives one from
  its `ThemeColors` (an accent-anchored hue spread — deterministic, distinct slots) so a future
  third-party/added theme is never uncolored. It must pass the SAME contrast gate as the tuned ones.
  (With only two built-ins today this is insurance, sized accordingly — a fn and its table, not a system.)
- **Contrast PROVEN, not eyeballed:** the SHIPPED `contrast_ratio`/`relative_luminance`
  (ui_components:128/142 — the recon confirmed they exist with ≥7.0 fg/bg tests already) gate EVERY
  (theme × slot) pair in tests: **each slot ≥ 4.5:1 (WCAG AA) against that theme's `background`**, both
  built-ins, plus the derived-default generator's output for both appearances. `Comment` may sit AT 4.5
  deliberately (muted is its job) but never below. (`Hint` is app-side, not a taxonomy slot — its muted
  treatment is #331's, untouched here.)
- **Live switch:** the #199 picker path (`set_theme` → notify → repaint) needs NO new wiring — colors
  resolve per frame (the dissolve above). A headless drive pins the observable: flip theme, the computed
  token color for a known span CHANGES in the same frame.
### Out (explicitly)
- User-custom palettes / a palette editor / per-language overrides; APCA (AA ratio v1); themed
  TERMINAL/ANSI colors (a different subsystem); new themes beyond the two built-ins (the generator
  covers arrivals); re-theming `Hint`/selection/find-band alphas (#331/#272 own those channels).

## Reference (§20)
The role taxonomy parallels TextMate/Zed SCOPE-theming as a CONCEPT (published, ubiquitous); every value
is Marley's own. WCAG 2.x contrast (published formula — already implemented in-repo). No copyleft source.

### Prior art
1. **Behavior maps / observed** — every serious editor themes syntax with the chrome; illegible
   pale-on-white token colors in light themes are the observed failure class the contrast gate exists for.
2. **Published material** — WCAG relative-luminance/contrast (the shipped fns implement it; tests cite AA).
3. **OUR OWN CODE — the sweep shrank the ticket twice:** (a) the WCAG helpers already ship WITH
   precedent tests (the description guessed right; the recon confirmed file:line); (b) the cache-
   invalidation half of the description DISSOLVED — kinds-not-colors in the cache means the live switch
   is free. What remains: the type, two tuned palettes, the generator, the gate tests, and the
   `token_color` reroute. `gpui::Hsla` is the value type throughout (no new color math beyond the
   shipped luminance).

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-PALETTE-ON-THEME** — `SyntaxPalette` lives with `ThemeColors` in ui_components; `Theme` carries it.
- **D-TOTAL-COLOR-FOR** — total accessor, Plain→foreground; unthemed impossible by type.
- **D-CONTRAST-GATED-BY-TEST** — the (theme × slot) AA matrix is a TEST; a palette edit that breaks it
  fails the gate, not a reviewer's eye.
- **D-KINDS-NOT-COLORS-IN-CACHE** — the dissolve, recorded so nobody re-invents invalidation.
- **D-DERIVED-DEFAULT-SAME-GATE** — insurance-sized, held to the same floor.
- **D-AFTER-315** — hard dep: the taxonomy must exist before it is themed.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | resolve every TokenKind to a color through the active theme's palette (total; Plain→foreground) | pure |
| REQ-002 | pass the AA contrast matrix: every slot ≥4.5:1 vs background, BOTH built-in themes | pure — the gate test |
| REQ-003 | derive a palette for a theme without one (deterministic, distinct slots) that passes the SAME matrix, both appearances | pure |
| REQ-004 | change a known span's computed color in the SAME frame on theme switch (no stale cache — the dissolve's observable) | headless |
| REQ-005 | leave NO hardcoded token-color literal outside the palette (the grep-gate; the two shipped literals die) | unit/grep-gate |
| REQ-006 | route the editor rows, the hover fence, and the split pane through the one seam (unchanged call sites, new resolution) | headless state |
| REQ-007 | keep Light and Dark palettes DISTINCT where it matters (Str/Number differ per theme — the anti-goal was one look) | pure table |

## Phase Plan
P2 tune the two palettes (against real fixtures in both appearances) + the generator's spread math + the
grep-gate's exact pattern; P3 the type + palettes + reroute (compiler-forced arms), the generator, the
tests; P3.5 critics on the AA matrix edges (Comment at-the-floor, accent-vs-Keyword collision — two slots
too close to distinguish is a legibility bug the ratio test can't see; add a min-hue-distance check or an
eyeball note), the grep-gate false-positives; P4 the matrix + drives + gate; P5 docs (editor.md's theme
section + ui_components crate-map row). Standing traps:
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
