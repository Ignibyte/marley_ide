# TICKET-035 — Warp-matched theme palette (dark + light)

- **Forge ticket:** #35 `5181c3d7-0a86-49de-b906-f0866e360338` (feature, M1.E — The Warp Look, seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `219f0c18-e640-4709-a6b8-716d797e2c07`
- **Pipeline doc:** ../../pipeline/active/warp-palette.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Status:** closed

## Summary
The placeholder grayscale palette makes the cockpit look unfinished. Replace the `ThemeColors::
default_for` dark + light values with an ORIGINAL Warp-matched palette (deep near-black dark, warm
off-white light, a distinctive Marley accent, tuned elevation) + add a semantic `success` green for
#36's exit-status. Keep the metric tokens. Pure data + the tested accessor; the render re-skins
automatically. Clean-room (derived values, not lifted). Chad verifies the look.

## Acceptance
`default_for` at cov 100/MSI 100 (the new per-appearance values; light≠dark; `success` present +
distinct from danger/accent/background; determinism + unchanged metric tokens) via the updated
field-equality tests; the `success` field added to all 4 construction sites; FULL gate GREEN + the
masked shell_dark/light baselines. Full EARS in the pipeline spec.
