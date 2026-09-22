---
pipeline_id: 9b30978e-9d26-4d55-87a6-3f2657b12edc
ticket: forge#41 (acf70690-53e8-45ab-a9ad-3e86598d51a4) · local docs/planning/tickets/open/TICKET-041-interactive-keys.md
aar_id: a5c302fd-1892-41ed-9006-4efced4cb324
status: Phase 5 — Complete PASS
title: full interactive key coverage (Shift-Tab, F-keys, modified arrows, Insert)
type: feature
milestone: M1.F
references:
  - crates/terminal_blocks/src/keys.rs (encode_key + KeyCode + KeyInput — the pure encoder)
  - crates/marley_app/src/app.rs (key_input_from_keystroke:278 — the shim mapping, mutants::skip)
  - docs/specs/SPEC-terminal-blocks.spec.md (R25)
---

## Title
Now that #40 streams input to running programs, they need the keys interactive menus + editors use
beyond #33's basic set (arrows/Home/End/PageUp-Dn/Delete/Enter/Tab/Esc/ctrl): **Shift-Tab** (menus +
Claude Code navigate backward with it), **F1–F12**, **modified cursor keys** (Shift/Alt/Ctrl+arrow —
word-jump + selection), and **Insert**. Extend the pure gpui-free `encode_key`.

## Scope
### In
- `crates/terminal_blocks/src/keys.rs` (PURE, gpui-free — cov/MSI 100):
  - `KeyCode` gains `BackTab`, `F(u8)`, `Insert`.
  - `KeyInput` gains `shift: bool` (for the modified-key CSI parameter).
  - `modifier_param(shift: bool, alt: bool, ctrl: bool) -> u8` = `1 + shift as u8 + 2*(alt as u8) +
    4*(ctrl as u8)` — the xterm modifier code (1 = none, 2 = Shift, 5 = Ctrl, 8 = all).
  - `csi_cursor(param: u8, final_byte: u8) -> Vec<u8>` — `ESC[1;<param><final>` when `param > 1`,
    else the plain `ESC[<final>`.
  - `encode_key` adds: `BackTab → ESC[Z`; `Insert → ESC[2~`; `F(n)` → SS3 `ESC O P/Q/R/S` for F1–4 and
    CSI `ESC[<code>~` for F5–12 (codes 15/17/18/19/20/21/23/24), unknown `n` → empty; and the cursor
    keys `Up/Down/Left/Right/Home/End` route through `csi_cursor(modifier_param(...), final)` so a
    modified arrow emits the parameterized form and a plain one is unchanged.
- `crates/marley_app/src/app.rs` (SHIM, `key_input_from_keystroke` — already `mutants::skip`): set
  `shift: keystroke.modifiers.shift`; map `"tab"`+shift → `BackTab`, `"f1"…"f12"` → `F(n)`,
  `"insert"` → `Insert`. The 4 test `KeyInput` literals gain `shift: false`.
- SPEC-terminal-blocks R25 (the extended key set + mutation targets). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Mouse reporting (DECSET 1000/1006 — a running program wanting mouse events). Kitty/modifyOtherKeys
  extended keyboard protocols. Bracketed paste (#42). The application-cursor-keys mode (DECCKM —
  `ESC O` vs `ESC [` for plain arrows) — Marley emits the normal-mode CSI form (what most programs
  accept); the app-mode toggle is a later refinement.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `modifier_param` follows the xterm convention `1 + Shift(1) + Alt(2) + Ctrl(4)`; a value of 1
  (no modifier) means the cursor key emits its PLAIN legacy form (no `1;` parameter), so existing
  programs + the #33 goldens are unaffected.
- D2 — `F(u8)` carries the function number; F1–4 use SS3 (`ESC O …`), F5–12 use CSI (`ESC[<code>~`)
  with the standard non-contiguous codes; an out-of-range `n` encodes to nothing (a safe no-op).
- D3 — `KeyInput.shift` is a new field; for a printable `Char` the shift is already resolved into the
  character (encode_key's Char arms ignore `shift`), so `shift` only affects the named-key CSI param.
- D4 — PURE: `encode_key`/`modifier_param`/`csi_cursor` (cov/MSI 100 via golden byte vectors + the
  arithmetic); the gpui `Keystroke` → `KeyCode` mapping is SHIM (app.rs, mutants::skip).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `encode_key` is given `BackTab` / `Insert` / `F(n)`, it shall emit the exact bytes: `BackTab → ESC[Z`; `Insert → ESC[2~`; `F1–4 → ESC O P/Q/R/S`; `F5–12 → ESC[{15,17,18,19,20,21,23,24}~`; an out-of-range `F(n)` → empty. | unit (golden vectors, incl. the `_` arm) |
| REQ-002 | WHEN `modifier_param(shift, alt, ctrl)` is called, it shall return `1 + Shift + 2·Alt + 4·Ctrl`. | unit (none→1, Shift→2, Alt→3, Ctrl→5, all→8) |
| REQ-003 | WHEN a cursor key (`Up/Down/Left/Right/Home/End`) is encoded, it shall emit `ESC[1;<param><final>` while any modifier is held and the plain `ESC[<final>` otherwise. | unit (Shift-Up → `ESC[1;2A`; Ctrl-Right → `ESC[1;5C`; plain Up → `ESC[A`) |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on the new `encode_key` arms + `modifier_param` + `csi_cursor`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `KeyCode`/`KeyInput` changes, `modifier_param` + `csi_cursor` + `encode_key`
  arms + the F-key sequences, the shim mapping, the SPEC edits + mutation targets.
- **P3 Implement** — keys.rs (+ the shift ripple in the 4 test literals + session.rs) + app.rs shim +
  spec + CHANGELOG.
- **P3.5 Inspect** — critics: the golden byte sequences correct (against xterm), modifier_param
  arithmetic killable (no equivalent), csi_cursor param>1 branch, the F(n) _-arm, no #33 regression.
- **P4 Validate** — the encode goldens + modifier_param + csi_cursor tests + gate GREEN.
- **P5 Complete** — docs, AAR, archive, close #41.
