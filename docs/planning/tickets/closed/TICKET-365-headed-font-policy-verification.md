# TICKET-365 — test: headed-lane end-to-end font-policy verification (resolvable-apply #344 + monospace #361)

- **Ticket:** LOCAL #365 (chore, M-unset)
- **Tags:** editor, settings, font, test-lane, headed, 344-followup, 361-followup
- **Created:** 2026-07-19
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id afb7aa3c-c8e3-4d95-a2c7-79a26eb0198d)
- **Status:** closed (2026-08-14 — shipped; pipeline `365-headed-font-policy`, gate GREEN [diff])

## Closed — 2026-08-14

Shipped as designed, with one seam correction discovered at design: the
`*_for_test` oracles don't exist in a spawned binary, so the drives assert on an
env-gated stdout self-report (`MARLEY_FONT_POLICY_SELFTEST=1` → one tab-delimited
post-boot line) instead — see
`AD-claude-365-env-gated-stdout-self-report-is-the-headed-state-assert-seam-001`.
Both drives green on the live session (Monaco applies silently; Helvetica falls
back + "not monospace" flash); the always-false sabotage smoke flips Monaco RED
end-to-end. Needs only a live WindowServer session — the harness's AX/Screen
Recording grants are NOT required, which un-blocks this lane class from most of
the #417 activation constraints.

## Description

Two font-policy boot paths are currently NOT headless-testable and rely only on pure-unit proof of the decision + trust in the app wiring:
- #344 resolvable-APPLY: a resolvable mono family (e.g. Fira Code) actually applies (mono_family == the family, no flash).
- #361 resolvable-but-PROPORTIONAL: a resolvable proportional family (e.g. Helvetica) falls back to the built-in mono + a "not monospace" flash; and a resolvable MONO family (e.g. Monaco) applies silently (the negative-of-the-negative — a font_is_monospace-always-false regression would false-warn every real mono font).

## Why not headless
`#[gpui::test]` uses gpui's NoopTextSystem: `all_font_names()` resolves no real font (so any family reads UNRESOLVABLE → the boot wire's `!resolves` short-circuit fires and font_is_monospace is never called), and Noop's synthetic per-glyph advance makes every font read as equal-width (so it couldn't distinguish mono from proportional even if reached). The #344 garbage drive documents this ("The RESOLVABLE case is not headless-testable here (Noop resolves nothing)"). So the app-side metric reads (`font_resolves`, `font_is_monospace`) + the boot wiring are mutants::skip shims proven only by the pure `resolve_font_family`/`is_monospace_advance` units + manual/headed inspection.

## The work
Add a HEADED / real-CoreText drive (the marley_visual_harness / render_to_image lane — the #264/#271 deferred, real text system) that boots RootView with (a) a real mono family and asserts applied+silent, (b) a real proportional family and asserts fallback+flash, (c) a real mono family and asserts applied+silent (the #361 no-false-warn direction). Depends on the headed/real-CoreText harness (gpui render_to_image on a gpui upgrade, or the headed self-test lane with a real screen). Low priority — the pure decision is fully unit-tested; this closes the app-wiring gap.

References: #344 (resolve fallback), #361 (monospace warn), #271 (pixel capture / render_to_image deferral), #264 (the headless lane's NoopTextSystem limitation). M22, area editor/settings/test-lane.
