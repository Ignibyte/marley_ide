---
pipeline_id: d9685a67-a717-4e90-be10-301c1b7f3535
ticket: forge#42 (d1b32f42-f609-4192-8214-12a09e3e8e03) · local docs/planning/tickets/open/TICKET-042-bracketed-paste.md
aar_id: cba47182-58ae-4665-bdf7-de42bcd9ddf9
status: Phase 5 — Complete PASS
title: clipboard paste + bracketed-paste mode
type: feature
milestone: M1.F
references:
  - crates/terminal_blocks/src/keys.rs (paste_bytes — the pure wrap + injection guard)
  - crates/terminal_blocks/src/session.rs (is_bracketed_paste — DECSET 2004 via term.mode())
  - crates/marley_app/src/app.rs (the cmd-V handler — shim)
  - docs/specs/SPEC-terminal-blocks.spec.md + SPEC-app-shell.spec.md
---

## Title
The app has NO paste handling today — you can't cmd-V into the terminal. Add clipboard paste, with
BRACKETED-PASTE safety: when a program enables bracketed paste (DECSET 2004 — editors, REPLs, Claude
Code), the pasted text is wrapped in `ESC[200~`…`ESC[201~` so a multi-line paste arrives as literal
DATA, not a flood of keystrokes with each newline EXECUTING (the "pasted a script and it ran half of
it" hazard). Closes M1.F.

## Scope
### In
- `crates/terminal_blocks/src/session.rs` (PURE, gpui-free — cov/MSI 100):
  `TerminalSession::is_bracketed_paste(&self) -> bool` = `self.term.mode().contains(TermMode::
  BRACKETED_PASTE)` (alacritty 0.26 has the flag; mirrors `is_alt_screen`).
- `crates/terminal_blocks/src/keys.rs` (PURE, gpui-free — cov/MSI 100):
  `paste_bytes(text: &str, bracketed: bool) -> Vec<u8>` — WHEN `bracketed`, `ESC[200~` + the text with
  every embedded `ESC[201~` stripped + `ESC[201~` (the strip is the paste-INJECTION guard: a pasted
  end-marker cannot close the bracket early and inject the tail as commands); WHEN not bracketed, the
  raw text bytes.
- `crates/marley_app/src/app.rs` (SHIM): a cmd-V paste handler — read the gpui clipboard
  (`cx.read_from_clipboard()` → `ClipboardItem::text()`), then `paste_bytes(text,
  session.is_bracketed_paste())`, routed by #40: while a command runs → `write_bytes` to the PTY; at
  the bare prompt → insert into the local prompt buffer. (Design picks keymap-binding vs inline.)
- SPEC-terminal-blocks (is_bracketed_paste + paste_bytes) + SPEC-app-shell (the paste handler).
  CHANGELOG + arch docs.

### Out (explicitly deferred)
- Rich multi-line editing of a paste at the BARE prompt (Marley's local line editor is single-line; a
  multi-line paste at the prompt inserts as-is for the first cut — the shell/program gets it on Enter
  or once a command runs). Primary-selection / middle-click paste, image/file paste, paste history
  (M2+). COPY (cmd-C) from the terminal selection is a separate ticket (no selection model yet).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `paste_bytes` strips embedded `ESC[201~` ONLY in the bracketed branch (that's where the marker
  is load-bearing); the non-bracketed branch passes raw bytes (no bracket to protect). This is a real
  security guard (bracketed-paste escape → command injection), not cosmetic.
- D2 — `is_bracketed_paste` mirrors `is_alt_screen` exactly (a `term.mode().contains(..)` read),
  tested with the same mock+DCS pattern (feed `ESC[?2004h`/`l`).
- D3 — Paste routing reuses #40: `command_running` → the PTY (the program, bracketed if it asked);
  else the local buffer. PURE: `paste_bytes` + `is_bracketed_paste`; the clipboard read + routing +
  buffer insert are SHIM (app.rs, masked visual).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `paste_bytes(text, true)` is called, it shall return `ESC[200~` + `text` (with every embedded `ESC[201~` removed) + `ESC[201~`; WHEN `paste_bytes(text, false)`, it shall return the raw UTF-8 bytes of `text`. | unit (goldens: plain; wrapped; multi-line preserved) |
| REQ-002 | WHEN the pasted text contains an `ESC[201~` end-marker, `paste_bytes(_, true)` shall emit exactly ONE trailing `ESC[201~` (the wrapper's) — the embedded marker(s) stripped, so the bracket cannot be closed early. | unit (an embedded-marker fixture) |
| REQ-003 | WHILE a program holds DECSET 2004, `TerminalSession::is_bracketed_paste()` shall return `true`; otherwise `false`. | unit (mock+DCS: `ESC[?2004h` → true; default/`l` → false) |
| REQ-004 | WHEN cmd-V is pressed, the app shall read the clipboard and send `paste_bytes(text, is_bracketed_paste())` to the running program, or insert it at the prompt. | shim + masked visual (chad pastes multi-line into an editor → one literal block) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `paste_bytes` + `is_bracketed_paste`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — `paste_bytes`'s exact shape + the strip, `is_bracketed_paste`'s delegate, the app.rs
  cmd-V handler (keymap vs inline; the clipboard read + route + buffer insert), the SPEC edits +
  mutation targets + the is_bracketed_paste mock+DCS test.
- **P3 Implement** — session.rs + keys.rs + app.rs handler + specs + CHANGELOG.
- **P3.5 Inspect** — critics: the marker goldens + the injection strip (can any embedded/split marker
  survive?), is_bracketed_paste true/false, the shim routes via #40, no keymap regression.
- **P4 Validate** — the paste_bytes + is_bracketed_paste tests + gate GREEN.
- **P5 Complete** — docs, AAR, archive, close #42 — **and close the M1.F sprint**.
