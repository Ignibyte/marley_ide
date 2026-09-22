---
pipeline_id: 306f8712-27e5-4894-8085-db20c8551faf
ticket: forge#280 (9d9d9550-a1da-4f22-a223-fa35efa66cc7) · local docs/planning/tickets/open/TICKET-280-tui-mouse.md
aar_id: a42de6e5-1fcf-493b-b7e0-aeef8ff36c27
status: Phase 5 — Complete PASS
title: Mouse reporting to TUI programs (SGR 1006 + legacy X10 + alt-scroll)
type: feature
milestone: M17
references:
  - docs/marley_architecture/terminal_blocks.md
  - docs/marley_architecture/app_shell.md
---

## Title
The TUI mouse gap: vim/htop/less/lazygit request mouse tracking
(DECSET 1000/1002/1003/1006) — alacritty's Term already parses the
flags — but Marley never consults them: the wheel scrolls MARLEY's
dead scrollback and clicks seed Marley selection while the program
wants the events. Encode + forward when tracking is on; byte-identical
local behavior when off; ⇧ is the universal local-selection bypass.

## Scope
### In
- Pure (terminal_blocks, next to keys.rs — `mouse.rs`):
  `MouseModes { click, drag, motion, sgr, alt_scroll, alt_screen }`
  (a gpui-free snapshot of the TermMode flags);
  `MouseButton3 { Left=0, Middle=1, Right=2 }`-style plain codes;
  `MouseEvent { Press(u8), Release(u8), Drag(u8), WheelUp, WheelDown }`;
  `MouseMods { shift, alt, ctrl }` (xterm bits 4/8/16);
  `mouse_report(modes, ev, col, row /*1-based*/, mods) ->
  Option<Vec<u8>>`:
  · no tracking mode → None — EXCEPT the ALTERNATE_SCROLL fallback
    (alt_screen ∧ alt_scroll ∧ wheel → the arrow-key bytes CSI A/B);
  · click-mode (1000) reports press/release only; drag (1002) adds
    button-held drags; motion (1003) adds them too (hover motion is
    never emitted by the shim — drags only);
  · wheel = buttons 64/65, press-only;
  · SGR (1006): `ESC [ < b ; x ; y M|m`; legacy X10 otherwise
    (`ESC [ M` + 32-offset bytes, 223-clamped) — cheap, included;
  · button bits: base 0/1/2, +32 drag, +64 wheel, +4 shift +8 alt
    +16 ctrl.
- Session accessor `mouse_modes() -> MouseModes` (the is_alt_screen
  idiom over `term.mode()`).
- The grid handler branches (down/up/move/wheel): when the FOCUSED
  pane reports tracking AND ⇧ is not held → encode + `write_bytes` +
  return (BEFORE the #279 selection call / the R39 scroll); drags
  throttle to one report per CELL change (a `last_mouse_cell` on
  TerminalPane, session-only). Coordinates are the VISIBLE alt-grid
  cell (top-left 1-based over the pty grid geometry) — NOT
  pane_grid_pos's bottom-anchored content rows.
- ⇧ held → the entire branch is skipped: local selection/scroll work
  exactly as today INSIDE htop (the universal copy-out hatch).

### Out
- Right/middle-button reporting — v1 wires LEFT only; right keeps
  Marley's own context menu unconditionally (even inside a tracking
  TUI) and middle is unbound. (The encoder accepts any button code —
  plain u8, a naming drift from the spec's sketched enum — so wiring
  more buttons later is shim-only.)
- Hover motion reporting (1003's any-motion half — no TUI Marley
  targets needs it; drags cover the real consumers).
- Focus events (1004), extended pixel reporting (1016).
- UTF8 mouse (1005 — superseded by SGR everywhere).

## Reference (§20)
The xterm mouse-tracking specification (ctlseqs — public) — SGR 1006,
legacy X10 encoding, the alternate-scroll fallback, and the
shift-bypass convention every terminal implements. Marley-original
encoder + wiring. No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — SGR preferred, legacy X10 kept (6 lines; some older TUIs never
  negotiate 1006); UTF8-mouse skipped.
- D2 — ⇧ bypasses reporting entirely in the SHIM (never reaches the
  encoder): the local-selection hatch beats protocol completeness.
- D3 — Drag reports at CELL granularity (per-cell throttle) — the
  spec's cadence; per-pixel spam would flood the PTY.
- D4 — The wheel fallback (alt-scroll → arrows) fires only in
  alt-screen with tracking OFF — the less/vim-without-mouse case.
- D5 — Coordinates: the visible grid cell over pty_size geometry,
  clamped into [1, cols]×[1, rows].

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `mouse_report` shall produce the exact SGR bytes for press/release/drag/wheel with correct button+modifier bits and 1-based coords, the exact X10 bytes when SGR is off (223-clamped), arrows for the alt-scroll fallback, and None for every untracked combination — the full fixture matrix. | pure units (kill list) |
| REQ-002 | With tracking on (headless: a PTY program enabling 1000/1006 via printf), a grid click shall write the SGR press+release to the PTY and NOT seed Marley selection; the wheel shall write 64/65 and NOT scroll Marley scrollback. | headless (PTY echo asserts) |
| REQ-003 | With ⇧ held, clicks/wheel shall behave byte-identically to tracking-off (local selection + scrollback). | headless |
| REQ-004 | With tracking off, all mouse behavior shall be byte-identical to pre-#280 (the #279 selection suite + R39 scroll tests green). | existing suites |

## Phase Plan
- P1+P2 combined; P3 implement; P3.5 sonnet critic (the bit matrix vs
  ctlseqs, coordinate geometry, precedence vs #279/#196/R39); P4
  units + headless + gate; P5 docs.
