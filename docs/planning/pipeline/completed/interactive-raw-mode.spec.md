---
pipeline_id: a3d28d94-fe16-4df5-b182-628c0e80497c
ticket: forge#33 (09a81942-989f-4d0f-a191-1a4290131e3f) · local docs/planning/tickets/open/TICKET-033-interactive-raw-mode.md
aar_id: c58ef971-3d07-432f-b251-1b2e56504f1b
status: Phase 5 — Complete PASS
title: interactive/raw mode — vim, top, less, Ctrl-C (the flagship)
type: feature
milestone: M1.D
references:
  - crates/terminal_blocks/src/keys.rs (NEW — the pure keystroke→bytes encoder + routing)
  - crates/terminal_blocks/src/session.rs (is_alt_screen + grid_styled_rows accessors)
  - crates/marley_app/src/app.rs (the shim: gpui Keystroke→KeyInput, route, mode-switched render)
  - docs/specs/SPEC-terminal-blocks.spec.md + SPEC-app-shell.spec.md
  - docs/planning/intake/prompt-shell-line-editing-model.md (the DEFERRED tab-completion decision)
---

## Title
Today input is LINE-BUFFERED (local `apply_key` Buffer, whole line on Enter), so the shell/program
never sees keystrokes — no Ctrl-C, no interactive/full-screen programs. Make Marley a real terminal
for full-screen apps: a PURE keystroke→bytes encoder + ALT-SCREEN raw mode (detect `ALT_SCREEN`,
stream keystrokes to the PTY, render the live grid), so vim/top/less/htop work; plus Ctrl-C/D/Z
routed to the PTY in cooked mode. Tab-completion at the bare prompt is DEFERRED (the local-editing-
vs-shell-ZLE model decision — intake `prompt-shell-line-editing-model.md`).

## Scope
### In
**PURE — NEW `crates/terminal_blocks/src/keys.rs` (gpui-free):**
- `KeyCode { Char(char), Enter, Backspace, Tab, Escape, Up, Down, Left, Right, Home, End, PageUp,
  PageDown, Delete }` + `KeyInput { code: KeyCode, ctrl: bool, alt: bool }`.
- `encode_key(input) -> Vec<u8>` — the VT input encoding: printable → UTF-8 (ESC-prefixed when
  `alt`); `Ctrl+letter` → the C0 control byte via `ctrl_byte`; Enter → `\r` (CR); Backspace → DEL
  `0x7f`; Tab → `0x09`; Escape → `0x1b`; arrows → `ESC [ A/B/C/D`; Home/End → `ESC [ H/F`; PageUp/
  PageDown → `ESC [ 5~/6~`; Delete → `ESC [ 3~`.
- `ctrl_byte(c: char) -> u8` = `(c.to_ascii_uppercase() as u8) & 0x1f` (Ctrl-C→3, Ctrl-D→4, Ctrl-Z→26).
- `input_route(alt_screen: bool, ctrl: bool) -> Route{Cooked, Raw}` — alt-screen → Raw (all
  keystrokes stream); else a control key → Raw (the signal reaches the shell); else Cooked (local
  edit).

**SHIM (tested where possible) — `crates/terminal_blocks/src/session.rs`:**
- `is_alt_screen(&self) -> bool` = `self.term.mode().contains(TermMode::ALT_SCREEN)` — TESTED via a
  mock feeding the DECSET `\x1b[?1049h` / reset `\x1b[?1049l` sequences.
- `grid_styled_rows(&self) -> Vec<StyledLine>` = `term_to_styled_rows(&self.term)` (the live grid
  for the alt-screen render; reuses #31).

**SHIM — `crates/marley_app/src/app.rs`:** map the gpui `Keystroke` → `KeyInput`; `route =
input_route(session.is_alt_screen(), ctrl)`; Raw → `session.write_bytes(&encode_key(input))`;
Cooked → the existing `apply_key`/Blocks path. The render: WHEN `is_alt_screen()`, paint the live
grid (`grid_styled_rows()` → the #31 colored-span loop) instead of the Block list + prompt.

**Integration test:** a real-zsh session runs `sleep 100`; sending `encode_key(Ctrl-C)` (0x03)
interrupts it (the block finishes / a new prompt returns).

### Out (explicitly deferred)
- **Tab-completion / Ctrl-R / shell keybindings at the bare prompt** — the local-editing-vs-shell-
  ZLE model decision (intake `prompt-shell-line-editing-model.md`); #33 does alt-screen + signals only.
- Mouse reporting, bracketed paste, the kitty/extended keyboard protocol, focus events, scrollback
  WITHIN alt-screen, the cursor position/shape in the grid render. The Shift+special encodings
  (Shift-Tab `ESC [ Z`). These are later terminal-fidelity cuts.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — The encoder lives in `marley_terminal` (gpui-free VT-input encoding — the terminal protocol
  layer, symmetric with alacritty owning output parsing). PURE cov 100/MSI 100.
- D2 — `ctrl_byte` = `(uppercase & 0x1f)` — one formula for every Ctrl+letter (mutation-tight).
- D3 — Routing: alt-screen → all Raw; cooked + control → Raw (signals interrupt); cooked + no-ctrl
  → Cooked (local edit preserved — #28/#29 still work at the prompt).
- D4 — Alt-screen render REUSES #31 (`grid_styled_rows` → the colored-span loop) — no new render.
- D5 — NOT split 6a/6b: "alt-screen complete" (encoder + raw streaming + grid render) is the clean
  deliverable; tab-completion-at-prompt is a genuinely separate model decision (deferred, D-out).
- D6 — Honest partial delivery: #33 makes vim/top/less/htop + Ctrl-C work; it does NOT make
  tab-completion at the prompt work (that's the intake). The ticket's title lists tab-completion —
  the completion is captured + deferred, not silently dropped.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `encode_key` is given a printable `Char`, it shall emit its UTF-8 bytes (ESC-prefixed when `alt`); WHEN given a `Ctrl+letter`, it shall emit the C0 control byte `(uppercase & 0x1f)`. | unit ('a'→[0x61]; alt-'a'→[0x1b,0x61]; Ctrl-C→[0x03]; Ctrl-D→[0x04]; Ctrl-Z→[0x1a]; multibyte char) |
| REQ-002 | WHEN `encode_key` is given a named key, it shall emit its VT sequence — Enter→`\r`, Backspace→`0x7f`, Tab→`0x09`, Escape→`0x1b`, arrows→`ESC [ A/B/C/D`, Home/End→`ESC [ H/F`, PageUp/Down→`ESC [ 5~/6~`, Delete→`ESC [ 3~`. | unit (one assertion per named key — distinct finals) |
| REQ-003 | WHEN `input_route` is given `alt_screen=true`, it shall return `Raw` for any key; WHEN `alt_screen=false`, it shall return `Raw` for a control key and `Cooked` otherwise. | unit (the four cases) |
| REQ-004 | WHEN a session has entered the alternate screen (DECSET 1049), `is_alt_screen` shall return true, and false after the reset. | unit (mock feeds `\x1b[?1049h` then `\x1b[?1049l`) |
| REQ-005 | WHEN `encode_key(Ctrl-C)` is written to a session running `sleep 100`, the sleep shall be interrupted (the running block finishes / control returns). | integration (real zsh) |
| REQ-006 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `encode_key`/`ctrl_byte`/`input_route`; the headed vim/top alt-screen baseline rides the desktop-session deferral (masked). | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `KeyCode`/`KeyInput`/`Route` shapes, `encode_key` arms + `ctrl_byte` +
  `input_route`, the `is_alt_screen`/`grid_styled_rows` accessors + the mock-DECSET test approach,
  the app.rs Keystroke→KeyInput map + the render switch, the two SPEC amendments + mutation targets.
- **P3 Implement** — keys.rs + session accessors + app.rs routing/render + specs + CHANGELOG.
- **P3.5 Inspect** — critics: every encode_key byte, the ctrl_byte formula, the routing, the
  alt-screen detection, the render switch, the gpui-free boundary.
- **P4 Validate** — the encoder/route/is_alt_screen unit suites + the Ctrl-C integration test + gate.
- **P5 Complete** — docs, AAR, archive, close #33; the sprint closes.
