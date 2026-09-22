# 280-tui-mouse — Notes

- **Forge ticket:** #280 9d9d9550-a1da-4f22-a223-fa35efa66cc7
- **AAR:** a42de6e5-1fcf-493b-b7e0-aeef8ff36c27
- **Local ticket doc:** docs/planning/tickets/open/TICKET-280-tui-mouse.md
- **Pipeline spec:** 280-tui-mouse.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan (combined with Phase 2)
- /goal batch ticket 10 of 10 (terminal polish #3 — the LAST of the
  goal range).
- **Recon (vendored 0.26 verified):** TermMode bits —
  MOUSE_REPORT_CLICK (1<<3), SGR_MOUSE (1<<5), MOUSE_MOTION (1<<6),
  MOUSE_DRAG (1<<13), ALTERNATE_SCROLL (1<<15), MOUSE_MODE = the
  click|motion|drag union; `Term::mode()` is pub; the session already
  reads it (`is_alt_screen`, session.rs:170 — the accessor idiom).

## Phase 2 — Design
- **terminal_blocks/src/mouse.rs (pure):** the types + `mouse_report`
  per the spec's matrix. SGR: `\x1b[<{b};{x};{y}{M|m}` (m only for
  Release). X10: `\x1b[M` + (32+b, 32+x.min(223), 32+y.min(223)) as
  bytes. Alt-scroll fallback: `\x1b[A`/`\x1b[B` (normal-mode CSI —
  the cursor-keys-app-mode variant is a recorded nuance; less/vim
  accept CSI A/B). Buttons: press/release carry base 0/1/2; Release
  in SGR keeps the base code (the `m` marks release); X10 release =
  code 3 (the spec's release byte). Drag = base+32. Wheel 64/65
  press-only. Mods add 4/8/16.
- **Session:** `mouse_modes() -> MouseModes` snapshot incl.
  `alt_screen` (one lock read).
- **App shim:** in the grid handlers, FIRST (before #279 selection /
  #196 link rows / R39 scroll): compute the visible cell from the
  PANE RECT + pty_size (col = (x-rect.x)/cell_w +1 clamped to cols;
  row = rows - ((rect.bottom - y)/cell_h) clamped — the grid is
  bottom-packed via justify_end, so measure from the BOTTOM; the
  alt grid always has exactly pty rows so bottom-anchoring is exact);
  if !shift && modes.tracking → write + return. Mouse-up: the grid
  has no explicit up-listener today — add one (report-only). Drag:
  the existing on_mouse_move drag path gets the branch + the
  per-cell throttle (`last_mouse_cell: Option<(u16,u16)>` on
  TerminalPane). Wheel: the scroll handler branch (report, else
  alt-scroll arrows, else Marley scroll).
- **Manifest:** terminal_blocks/src/mouse.rs (+lib export) ·
  terminal_blocks/src/session.rs (accessor) · marley_app workspace.rs
  (last_mouse_cell) · app.rs (the four handler branches) ·
  headless_drive.rs (flows).
- **Test plan:** REQ-001 the encoder matrix (SGR press/release/drag/
  wheel × mods; X10 exact bytes + clamp; alt-scroll arrows; every
  untracked combo → None; click-mode drops drags; drag-mode accepts);
  REQ-002 headless: spawn a PTY, printf DECSET 1000;1006 (`printf
  '\\e[?1000h\\e[?1006h'`), poll until mouse_modes reports, then
  drive the app-side branch by... the #264 lane is keys-only — mouse
  synth unavailable: drive the DECISION via the pub(crate) branch
  method or assert at the ENCODER+accessor seam (the session reports
  modes; the encoder is fully unit-pinned; the handler branch is a
  4-line compose). Honest scope: headless = the accessor round-trip
  through a REAL PTY DECSET + the encoder units; the handler
  composition is inspected + carried by the driven capture on
  unlock (htop wheel/click — the ticket's own validate plan).
- **Risks:** R1 the coordinate row math under bottom-packing (the
  critic's geometry hunt); R2 the up-listener addition must not
  disturb #279's flows (report-only, gated); R3 SGR x/y are 1-based
  u16 — no 223 clamp in SGR (only X10).
- **Autonomy:** sonnet critic.

## Phase 3 — Implement
- terminal_blocks: mouse.rs (MouseModes+tracking(), MouseEvent,
  MouseMods+mod_bits, mouse_report — SGR/X10/alt-scroll per the spec
  matrix; X10 release = code 3, 223 clamps) + lib exports;
  session.mouse_modes() snapshot (one mode read, the is_alt_screen
  idiom).
- marley_app: TerminalPane.last_mouse_cell (+init);
  `pane_mouse_cell` (the VISIBLE pty cell, bottom-anchored like #179,
  clamped 1-based — distinct from pane_grid_pos's content rows); the
  four handler branches — down (Press(0) before the #279 selection,
  focus/dismiss still run), NEW up listener (Release(0), report-only,
  clears the throttle), move (Drag(0) per-cell throttle before the
  selection extend), wheel (per-step 64/65 reports after
  scroll_steps; the encoder's None → the local scroll untouched; the
  alt-scroll fallback rides the same call). ⇧ gates every branch
  (D2). Left button only v1 (right = the menu, middle unbound) —
  documented.
- No deviations. check + clippy + fmt clean; suite 1006/1006 (the
  #279 selection + R39 scroll suites green — the tracking-off
  identity).

## Phase 3.5 — Inspect
### Ledger (1 SONNET critic — path-dep probe over the REAL crate +
a real mutants --list + git archaeology)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| F1 | [HIGH] THE DETACH TRAP, STRIKE SIX: pane_mouse_cell inserted between pane_grid_pos's doc+skip and its fn — the skip re-bound to the NEW fn (pane_grid_pos: 14 live mutants, untested-by-design; pane_mouse_cell: ZERO — a blind spot on exactly the new geometry; the doc turned Frankenstein). | REAL | Re-seated (skip back on pane_grid_pos; pane_mouse_cell gets its own doc, NO skip — pure and unit-pinnable). --list verified: 0 / 28. BF-claude-mutants-skip-detach-strike-six. |
| F2 | [MED] The alt-scroll fallback ignores DECCKM (always CSI A/B; spec says SS3 under app-cursor mode). NOT a regression — encode_key's REAL arrow keys also ignore DECCKM project-wide (verified: zero APP_CURSOR consumers). | REAL, follow-up | One ticket covering BOTH (MouseModes.app_cursor + encode_key's cursor()) at Phase 5 — a partial fix scoped to the fallback alone would leave the arrows inconsistent. |
| F3 | [MED] pane_mouse_cell inherits a PRE-EXISTING pty_size skew: the resize loop feeds plan_resize the FULL pane rect while the render carves PANE_TITLE_H (24px) — pty rows ≈1 too many since M5 #108 (git-blamed), the TUI's true top row is already clipped; clicks near the top skew the same ≈1 row. | REAL, pre-existing, follow-up | One follow-up fixing the resize rect (`r.h - PANE_TITLE_H`) heals the clipping AND the mouse rows together. Noted as a known limitation. |
| F4 | [LOW] Left-only is a real scope decision the spec never stated. | REAL (doc) | The spec's Out section now states it (+ the u8-vs-enum naming drift note). |
| — | Cleared (probe-verified): the FULL SGR/X10 bit matrix exact vs ctlseqs (incl. X10 release=3 identity loss, wheel 0x60, the 223 boundary, combined mod bits 56); the alt-scroll AND-gate; click-mode drops drags; geometry hand-traces (bottom band → last row; above-content → row 1; pty (0,0) guarded); the title bar is UNREACHABLE by these handlers (the content div starts below it — hit-test bounds); fractional truncation matches bottom_anchored_row bit-for-bit; all four precedence hunts (selection return, no link spans exist in alt-screen so ⌘ can't regress, no shadowed up-listener, right-click untouched); the wheel remainder survives the early return; mouse_modes() is a bare field read; suites 113/113 + 15/15. | — | — |

## Phase 4 — Validate
- **Encoder units (3, mouse.rs):** the SGR matrix byte-exact (press/
  release/drag/wheel, unclamped coords, mod bits incl. the combined
  56 and button-2+shift=6); the X10 matrix (32-offsets, release=3
  identity loss, wheel 0x60, the 223→255 clamp boundary); the
  decision edges (untracked None, click-only drops drags, the
  alt-scroll AND-gate + wheel-only, tracking() per-flag).
- **Geometry kill list (app.rs):** pane_mouse_cell's 28 mutants —
  interior mapping, bottom band, the 1px-border BOUNDARY vector
  (y=431 — the first gate run missed the `-1.0`→`+`/`/` pair; my
  429 vector straddled the wrong side), top row, above/below clamps,
  right/left edges, the (0,0) pty guard, the epsilon guards.
- **In-crate DECSET test (session.rs):** mouse_modes flips on
  1000+1006+1007 via the MockPtyChannel (the is_alt_screen idiom) —
  the first gate run ALSO missed the Default-body mutant because
  cargo-mutants scopes tests to the MUTATED PACKAGE: the app-side
  headless flow can never kill a terminal_blocks mutant. Lesson
  recorded.
- **Headless:** `mouse_modes_track_decset_headless` — a REAL PTY
  printf's the exact htop negotiation; the snapshot flips through
  the live pump.
- **Gate:** first `--diff` run RED (the 3 misses above) → fixed at
  source → **GATE GREEN [diff] — 15/15**; suite 1011/1011; doctests
  green.
- **Driven capture: ENV-BLOCKED (probe max RGB 41 — dim/asleep,
  not interactive)** — the htop wheel/click captures join the unlock
  batch (now 7 items, ~2 min total when live).

## Phase 5 — Complete
- CHANGELOG under "### Added"; app_shell.md + terminal_blocks.md
  updated. Follow-up tickets minted: DECCKM (the fallback + encode_key
  arrows together) and the pty_size title-bar skew (F3 — heals render
  clipping + mouse rows in one change). AAR a42de6e5 submitted
  (materialized: BF-claude-mutants-skip-detach-strike-six).
- Forge #280 closed (done); local ticket → closed/.
- Lessons: (1) the detach trap struck a SIXTH time — the --list check
  must be reflexive near any skip attribute; (2) cargo-mutants scopes
  to the mutated package: a cross-crate seam needs its kill test
  IN-CRATE (the MockPtyChannel idiom); (3) boundary mutants need
  vectors on BOTH sides of the offset.
