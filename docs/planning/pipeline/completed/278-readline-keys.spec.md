---
pipeline_id: 9d995524-0a91-4969-884e-565f13fdd07f
ticket: forge#278 (672bb1f9-025c-43e7-8132-b9789b721dce) · local docs/planning/tickets/open/TICKET-278-readline-keys.md
aar_id: 9aca3e2c-4dc0-467f-96d8-5aeb5d4283eb
status: Phase 5 — Complete PASS
title: Readline keys at the cooked prompt (⌃A/⌃E/⌃K/⌃U/⌃W/⌃Y)
type: feature
milestone: M17
references:
  - docs/marley_architecture/terminal.md
---

## Title
The muscle-memory gap: at the COOKED prompt the emacs/readline control
keys do nothing useful. ⌃A line-home, ⌃E line-end, ⌃K kill-to-end,
⌃U kill-to-start, ⌃W kill-word-back, ⌃Y yank (ONE kill slot, not the
full ring) — over Marley's own prompt buffer.

## Scope
### In
- **Premise correction (plan recon):** `input_route` sends EVERY
  ctrl-chord Raw today (`alt || running || ctrl → Raw` —
  keys.rs:186), not only ⌃C/⌃D/⌃Z as the ticket text says. The six
  readline chords are therefore INTERCEPTED BEFORE the route call,
  gated `!alt_screen && !command_running`; the route itself is
  UNTOUCHED (⌃C/⌃D/⌃Z and every other ⌃ still stream Raw exactly as
  before; a RUNNING command gets raw 0x01 for ⌃A as today).
- Pure (input.rs): `ReadlineOp` (LineHome/LineEnd/KillToEnd/
  KillToStart/KillWordBack/Yank), `op_for_ctrl_key(&str) ->
  Option<ReadlineOp>` (the a/e/k/u/w/y map), and
  `apply_readline(buffer, caret, kill: &mut Option<String>, op) ->
  bool` (edited?) — kills capture the removed text into the slot
  (empty kills DON'T clobber it: ⌃K at EOL then ⌃Y must still yank
  the previous kill — the readline convention); yank inserts at the
  caret; word-back uses the #257 word class via `move_word_left`.
- `TerminalPane.kill: Option<String>` — per-pane, session-only.
- The app arm: in the terminal key region BEFORE the `input_route`
  call — `ctrl && !alt && !running && op_for_ctrl_key(key)` →
  apply_readline + notify.
- EDITOR untouched (the #267 arm's `!control` gate already excludes
  ⌃-chords; not widened).

### Out
- The full emacs kill RING / ⌘Y rotation; ⌃T transpose; ⌥F/⌥B (⌥
  arrows already word-move); ⌃R history search (a later polish).
- Any input_route change.

## Reference (§20)
The readline/emacs binding convention (public, universal — bash/zsh/
fish/every terminal's cooked line editor). Marley-original
implementation over Marley's own prompt Buffer/caret. No copyleft
source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — ONE kill slot (the last kill), not a ring: covers the
  overwhelmingly common ⌃K/⌃U→⌃Y flow; the ring is recorded future
  polish. EXPLICITLY (inspect F2): consecutive kills do NOT coalesce
  (readline appends ⌃W⌃W into one yank unit; Marley keeps only the
  last — the append needs last-op state, recorded with the ring). And
  ⌃W kills to the #257 word boundary — bash's ⌥⌫, NOT bash's
  whitespace-delimited unix-word-rubout ("/usr/local/bin" ⌃W kills
  "bin"); one word class everywhere (D4) wins over shell parity.
- D2 — An EMPTY kill (⌃K at EOL, ⌃U at col 0, ⌃W at 0) leaves the
  slot untouched (readline behavior) and reports not-edited.
- D3 — Route precedence preserved: the readline arm sits BEFORE
  `input_route` and claims ONLY its six chords in cooked conditions;
  everything else behaves byte-identically.
- D4 — ⌃W kills to the #257 word boundary (`move_word_left`), the
  same class ⌥← uses — one word definition everywhere.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `op_for_ctrl_key` shall map exactly a→LineHome, e→LineEnd, k→KillToEnd, u→KillToStart, w→KillWordBack, y→Yank, and nothing else. | pure units |
| REQ-002 | `apply_readline` shall: ⌃A/⌃E move the caret to 0/len; ⌃K remove caret..end into the slot; ⌃U remove 0..caret into the slot; ⌃W remove the word-back span into the slot; ⌃Y insert the slot at the caret advancing it — with the D2 empty-kill rule and multibyte-exact spans. | pure units (kill list) |
| REQ-003 | At the cooked prompt, the six chords shall edit Marley's prompt (headless: type, ⌃A→caret 0, ⌃K→line empties + slot holds it, ⌃Y→restored, ⌃W kills a word, ⌃U kills to start); the EDITOR's ⌃-chords shall remain excluded. | headless |
| REQ-004 | With a command RUNNING, ⌃A shall stream raw 0x01 to the PTY exactly as before (the route untouched; existing goldens green); ⌃C/⌃D/⌃Z shall stream in all states as before. | existing tests + headless negative |

## Phase Plan
- P1+P2 combined (recon complete); P3 implement; P3.5 critic (slot
  semantics, route-precedence regression, word fenceposts); P4 units
  + headless + gate; P5 docs.
