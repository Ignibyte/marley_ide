# TICKET-165 — M10: code tab wheel-scroll (regression fix)

- **Forge ticket:** #165 `fd1caabd-bb28-4b40-9eeb-be5ba4468e12` (bug, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `f2f0f603-6b77-4a5f-a354-28b885e3a7fb`
- **Pipeline doc:** ../../pipeline/active/code-scroll.spec.md
- **Status:** closed

## Summary
#154 left `cv.scroll` read-only — a >40-line file can't scroll in a code tab. Fix: pure `scroll_code` clamp
(cov/MSI 100) + an `.on_scroll_wheel` on the code body (remainder-accumulated steps, terminal sign convention)
+ a `scrollat` harness verb. Deps #154.

## Acceptance
scroll_code at cov/MSI 100; driven — the gutter advances on wheel-down and pins to 1 on far wheel-up; FULL
gate GREEN.
