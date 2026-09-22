# TICKET-387 — Per-section ＋ actions + never-empties guards re-expressed per section

- **Forge ticket:** #387 `e2ffcfbb-06dd-4c66-90e4-e25758769e64` (feature, M26)
- **Owner:** unclaimed (queued)
- **AAR:** — (opened at promotion)
- **Pipeline doc:** ../../pipeline/queued/387-section-actions.spec.md
- **Source:** chad's 2026-07-22 sectioned-shell direction; depends on #385
- **Status:** closed

## Summary
Sections become functional: each section header carries a hover-revealed ＋ whose action is
section-scoped — Terminal＋ → new terminal tab (existing path), Editor＋ → the open-file flow,
Browser＋ → open-or-switch the Forge cockpit (its transitional resident). The never-empties guard
is re-expressed per section and tested: Terminal keeps ≥1 (exact current LastTerminal refusal);
Editor/Browser may empty out to a bare section header. The LastTab arm is expected subsumed by
LastTerminal — verified, not assumed.

## Acceptance
Each ＋ drives its section's action and the new tab files correctly; last-terminal close still
refuses; last editor/browser close leaves the empty header; all live-driven. Full EARS in the
pipeline spec.
