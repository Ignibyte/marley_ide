# TICKET-223 — M12.2 — Warp parity: route chrome text sizes through type_scale (consolidate hardcoded px() literals)

- **Ticket:** LOCAL #223 (chore, M12.2)
- **Tags:** M12.2, visual, warp-parity, typography, tech-debt
- **Created:** 2026-07-09
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id b51a8774-f68b-4c73-a9cc-70c24625bd56)
- **Status:** closed

## Description

Follow-up surfaced by #195's inspect. typography.rs's stated purpose is "so the render stops scattering ad-hoc FontWeight/text_sm literals," but the CHROME still scatters ~20 hardcoded `text_size(px(9|11|12|13))` literals across app.rs (sidebar nav ~2036+, dock, status bar ~5561+, tabs, search ~3671+, the completion popup ~5517). Only the terminal block command/output + the dock caption_header + block-action affordances actually route through `type_scale(Role)` (5 sites). CONSEQUENCE: any future cockpit-wide density tune (another 1pt shift, or a user font-size setting) must hand-edit each literal and risks drift. SCOPE: add type_scale Roles (e.g. Body/Label/Meta/Header) covering the chrome bands, route the hardcoded app.rs literals through them, so text size has a SINGLE source. Also refresh the last stale `14.0` in the type domain: workspace.rs:108 `fallback_cell` non-positive-input guard default (unreachable dead branch since TERMINAL_FONT_SIZE=13, but reads stale) + its doc comment. PURE seam cov/MSI 100: the extended type_scale table (exact-value pins) + the fallback_cell guard. SHIM: the render reads the roles. Do this BEFORE #216 (density) / #219 (sidebar) so those calibrate through type_scale, not literals. Clean-room; no behavior change (values kept, just sourced from type_scale). Deps: #195 (type_scale foundation).
