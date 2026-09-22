---
pipeline_id: e5d49865-0b84-44df-a796-bcae35b8d70e
ticket: forge#286 (bf9c4440-69b5-4249-9cbb-09bbae161d4a) · local docs/planning/tickets/open/TICKET-286-decckm-app-cursor-keys.md
aar_id: 7daf1868-02b1-4827-8b7e-6ff41f3f83c6
status: Phase 5 — Complete PASS
title: Honor DECCKM (application cursor keys) — SS3 arrows + the alt-scroll fallback
type: bug
milestone: M17
references: []
---

## Title
Honor DECCKM (application cursor-key mode, DECSET 1). Under app-cursor mode a
terminal emits the SS3 form (`ESC O A`) for unmodified arrow / Home / End keys and
for the #280 alternate-scroll wheel fallback; Marley always emits the legacy CSI
form (`ESC [ A`). Thread `TermMode::APP_CURSOR` from the session snapshot into both
`encode_key` and the mouse fallback so the wire bytes are xterm-spec-correct.

## Scope
### In
- `MouseModes.app_cursor` field, fed from `TermMode::APP_CURSOR` in `session.mouse_modes()`.
- `session.is_app_cursor()` accessor (mirrors `is_alt_screen`/`is_bracketed_paste`).
- A shared pure `cursor_key_bytes(final, app_cursor)` (SS3 vs CSI for ONE unmodified cursor key).
- `encode_key(input, app_cursor)` — the unmodified cursor-key branch uses SS3 under app-cursor.
- The `mouse_report` alt-scroll fallback emits SS3 A/B under app-cursor.
- `app.rs` feeds `session.is_app_cursor()` into the `encode_key` call site.

### Out (explicitly deferred)
- Application KEYPAD mode (DECPAM/DECKPAM — the numeric keypad SS3), a separate flag.
- The modified cursor-key encoding (`ESC[1;<param>X`) is UNCHANGED — xterm keeps
  modified cursor keys in CSI form even under DECCKM.

## Reference (§20)
N/A — Marley-specific wire-protocol compliance with the PUBLIC xterm ctlseqs
specification (DECCKM / application cursor keys), not a Warp/Zed UX feature. The
`mouse.rs` header already cites the xterm ctlseqs spec as the reference for the
#280 encoding; this extends the same spec (SS3 vs CSI cursor keys) and is
reimplemented from the public spec — no copyleft source read (§20 clean-room). The
observable analog is that arrows drive vim/less correctly in app-cursor mode, as
any xterm-compatible terminal (incl. Warp) does.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — SS3 only for the UNMODIFIED cursor keys under app-cursor.** xterm keeps a
  MODIFIED cursor key (`param > 1`) in CSI form (`ESC[1;<param>X`) even under
  DECCKM; only `param == 1` switches CSI→SS3. So `csi_cursor`'s `param > 1` branch
  is untouched; only the `param == 1` branch consults `app_cursor`.
- **D2 — Terminal mode as a PARAM, not a `KeyInput` field.** Follows the existing
  `paste_bytes(text, bracketed)` precedent (terminal state is a separate arg, not
  baked into the keystroke value); keeps `KeyInput` a pure key event.
- **D3 — One shared `cursor_key_bytes(final, app_cursor)`** used by BOTH the
  unmodified cursor branch and the mouse fallback, so the SS3-vs-CSI spec rule
  lives in exactly one tested place (the ticket's "shared" requirement).
- **D4 — All six cursor keys** (Up/Down/Right/Left/Home/End, all routed through
  `csi_cursor`) get the SS3 form unmodified under app-cursor — xterm switches
  Home/End too (`ESC O H`/`ESC O F`).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN application cursor-key mode is active AND an UNMODIFIED arrow / Home / End is pressed, `encode_key` shall emit the SS3 form (`ESC O X`). | keys.rs golden: `encode_key(key(Up), true) == [0x1b,0x4f,0x41]` for all six. |
| REQ-002 | WHEN application cursor-key mode is OFF, `encode_key` shall emit the legacy CSI form (`ESC [ X`) — unchanged. | keys.rs golden: `encode_key(key(Up), false) == [0x1b,0x5b,0x41]`. |
| REQ-003 | WHEN a cursor key is pressed WITH a modifier, `encode_key` shall emit the CSI modified form (`ESC[1;<param>X`) regardless of app-cursor mode. | keys.rs golden: `encode_key(key_mod(Up, ctrl), true)` == the CSI `1;5A` bytes. |
| REQ-004 | WHEN application cursor-key mode is active, the alt-scroll wheel fallback shall emit SS3 A/B; when off, CSI A/B. | mouse.rs: fallback under `app_cursor` true/false. |
| REQ-005 | The session snapshot shall set `MouseModes.app_cursor` (and `is_app_cursor()`) iff `TermMode::APP_CURSOR` is set. | session.rs: a `MockPtyChannel` DECSET 1 / DECRST 1 round-trip. |
| REQ-006 | The shared `cursor_key_bytes` + the app-cursor cursor path shall be pure fns at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `cursor_key_bytes` + `csi_cursor(param, final, app_cursor)` + `encode_key(input, app_cursor)`; `MouseModes.app_cursor` + `session.is_app_cursor()`/`mouse_modes()`; the app.rs call-site feed; the regression test plan.
- **P3 Implement** — the keys.rs/mouse.rs/session.rs seams + the one app.rs call.
- **P3.5 Inspect** — critics/inline: the SS3-only-unmodified boundary, all-six-keys, the fallback, the round-trip.
- **P4 Validate** — goldens both modes + modified + fallback + DECSET round-trip; gate green [diff]; DRIVE the live app (arrow in vim → SS3).
- **P5 Complete** — CHANGELOG + editor.md/terminal doc, AAR, close #286.
