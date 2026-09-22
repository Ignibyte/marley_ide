# TICKET-221 — Warp visual parity: corner radii, borders & dividers

- **Forge ticket:** #221 (58a6eb14-fef1-46b2-b678-446c7d17cbdd) (feature, M12.2, warp-parity, borders)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** d6efcd44-6272-46fb-b167-6d02dcad6ae8
- **Pipeline doc:** ../../pipeline/active/warp-radii-borders.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25 · closes #224
- **Status:** closed

## Summary
Match Warp's rounding on the floating overlay cards. Explore mapped all 10 floating overlays (palette,
launcher, finder, history, forge, fleet, diff, find, completion, context-menu) as SQUARE — Warp's are
subtly-framed rounded cards. Round them (reusing the `corner_radius` token) + add a 1px muted border to the
6 borderless ones so the rounding reads. This CLOSES the open #224 (corner_radius orphan). Borders,
separators, and the #130 divider already read Warp-like → no churn. Clean-room — tokens only.

## Acceptance
The floating overlays render as rounded, framed Warp-style cards; contents/position/separators/divider
unchanged; #224 resolved. Full EARS (REQ-001..004) in the pipeline spec.
