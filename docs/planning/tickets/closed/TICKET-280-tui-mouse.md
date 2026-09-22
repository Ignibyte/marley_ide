# TICKET-280 — Mouse reporting to TUI programs (SGR 1006)

- **Forge:** #280 `9d9d9550-a1da-4f22-a223-fa35efa66cc7` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/280-tui-mouse.spec.md` (306f8712-27e5-4894-8085-db20c8551faf)

## Summary
When a TUI requests mouse tracking (alacritty's TermMode flags —
already parsed, never consulted), the grid's clicks/drags/wheel encode
as SGR 1006 (legacy X10 fallback; alt-scroll arrows when only that
mode is on) and stream to the PTY instead of driving Marley's
selection/scrollback. ⇧ held = the universal local bypass. Tracking
off = byte-identical pre-#280 behavior. Pure encoder next to keys.rs
(the encode_key symmetry) with the full ctlseqs bit matrix; per-cell
drag throttle; visible-grid coordinates.

## Acceptance
Spec REQ-001..004: the encoder matrix, the PTY-echo headless flows,
the ⇧ bypass, and the tracking-off identity.
