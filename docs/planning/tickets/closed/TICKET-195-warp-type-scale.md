# TICKET-195 — Warp visual parity: font-size calibration (type scale)

- **Forge ticket:** #195 (9377f953-94ed-41de-80af-307095b55bb8) (feature, M12.2, warp-parity)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 0ef796a3-e28e-4b95-bb78-283b96c83c5e
- **Pipeline doc:** ../../pipeline/active/warp-type-scale.spec.md
- **Source ticket:** M12.2 "Terminal fidelity & cockpit UX" (sprint #25); Warp-parity thread (#215)
- **Status:** closed

## Summary
Calibrate Marley's type scale (`typography.rs type_scale` Command/Output/Caption +
`TERMINAL_FONT_SIZE`) to match Warp's rendered font proportions/density. Measure against a
captured Warp reference (the host terminal). The mono cell metric derives from
`TERMINAL_FONT_SIZE` so it auto-tracks (columns stay aligned). Clean-room — observe Warp's
sizes, copy no assets/font. Pure seam: the `type_scale` exact-value pins.

## Acceptance
Terminal + role sizes calibrated toward Warp's proportions; the cell metric tracks (columns
aligned); text legible (≥12pt); values Marley-original. Full EARS (REQ-001..004) in the spec.
