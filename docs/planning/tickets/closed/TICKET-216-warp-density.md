# TICKET-216 — Warp visual parity: spacing & density calibration

- **Forge ticket:** #216 (54be0f67-0332-41e5-a9d6-31268f1ce23c) (feature, M12.2, warp-parity)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 2a540c15-0767-4908-af34-f015a9c9f458
- **Pipeline doc:** ../../pipeline/active/warp-density.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25
- **Status:** closed

## Summary
Calibrate Marley's load-bearing spacing/density drivers (pane-title height, dock/panel header
padding, rail/sidebar row density, bar heights, dock width) toward Warp's compact density.
Bounded — tune only genuine deltas vs the captured Warp reference; leave already-matching surfaces.
Clean-room. Pure seam: exact-value pins on any changed layout const.

## Acceptance
Density drivers calibrated toward Warp (or confirmed matching); no layout break; values
Marley-original. Full EARS (REQ-001..003) in the pipeline spec.
