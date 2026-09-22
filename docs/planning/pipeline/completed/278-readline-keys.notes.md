# 278-readline-keys — Notes

- **Forge ticket:** #278 672bb1f9-025c-43e7-8132-b9789b721dce
- **AAR:** 9aca3e2c-4dc0-467f-96d8-5aeb5d4283eb
- **Local ticket doc:** docs/planning/tickets/open/TICKET-278-readline-keys.md
- **Pipeline spec:** 278-readline-keys.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan (combined with Phase 2 — recon complete)
- /goal batch ticket 7 of 10 (terminal polish #1).
- **Recon:** `input_route` (terminal_blocks/keys.rs:186) Raw-routes on
  `alt || running || ctrl` — EVERY ctrl chord, not just C/D/Z (the
  ticket text corrected in-spec). The route call sits at app.rs:6203;
  the cooked `apply_key` dispatch at :6300 over
  `TerminalPane{buffer, caret, …}` (workspace.rs:266). Key::Other is
  where ctrl chords die in `key_from_keystroke` (ctrl→Other before
  key_char). The #267 editor arm excludes ⌃ (`!control`) — untouched.

## Phase 2 — Design
- **input.rs (pure):**
  - `ReadlineOp { LineHome, LineEnd, KillToEnd, KillToStart,
    KillWordBack, Yank }`.
  - `op_for_ctrl_key(key: &str) -> Option<ReadlineOp>` — the closed
    a/e/k/u/w/y map (kill-listable).
  - `apply_readline(buffer, caret, kill: &mut Option<String>, op) ->
    bool` — LineHome/End via `move_line_home/end`; KillToEnd removes
    `caret..len` (text captured via `text_in_range` before the edit);
    KillToStart removes `0..caret` (caret → 0); KillWordBack removes
    `move_word_left(caret)..caret`; ALL kills: an EMPTY span leaves
    the slot untouched + returns false (D2); Yank inserts the slot at
    the caret + advances by its char count (None slot → false).
- **workspace.rs:** `TerminalPane.kill: Option<String>` (+ init).
- **app.rs:** in the terminal key region, BEFORE the `input_route`
  call: `if ctrl && !alt_screen && !command_running { if let
  Some(op) = op_for_ctrl_key(&keystroke.key) { apply_readline(...);
  cx.notify(); stop; return; } }` — unclaimed ⌃ falls to the route
  (Raw) exactly as today.
- **Manifest:** input.rs (+ops/tests) · workspace.rs (field) ·
  app.rs (the arm) · headless_drive.rs (flows).
- **Test plan:** REQ-001 the closed map (7 rows incl. a non-member);
  REQ-002 kill-list — ⌃A/⌃E from mid/0/end; ⌃K at 0/mid/EOL(empty,
  D2)/multibyte (é日 spans); ⌃U at 0(empty)/mid/end; ⌃W over
  "ab cd"/leading-space runs/at 0(empty)/multibyte word; ⌃Y None(f)/
  Some-insert-mid/advance-count; kill-then-yank round-trips; slot
  overwrite by a NON-empty kill only. REQ-003 headless: type
  "echo hi", ⌃A caret 0, ⌃E end, ⌃K empty+slot, ⌃Y restore, ⌃W word,
  ⌃U rest; editor-tab ⌃A negative (caret unmoved — the #267 gate).
  REQ-004: existing `input_route_cases` + raw goldens green
  (untouched file); headless negative optional (the route path is
  pre-existing).
- **Risks:** R1 the arm placement must not shadow ⌃C/⌃D/⌃Z (map
  returns None for c/d/z → falls to the route ✓ by construction);
  R2 gpui ctrl chords — keystroke.key is the LETTER with
  `modifiers.control` (the keymap dispatch precedent); R3 the #89
  Tab-completion arm sits elsewhere in the ladder (untouched).

## Phase 3 — Implement
- input.rs: `ReadlineOp` + `op_for_ctrl_key` (the closed 6-map) +
  `apply_readline` (a shared `kill_span` closure returning the
  captured text; empty span → None → false, slot untouched — D2;
  yank guards `Some(non-empty)` and advances by char count).
- workspace.rs: `TerminalPane.kill: Option<String>` + init.
- app.rs: the arm right after the (alt_screen, running) read, BEFORE
  the route block — gated `ctrl && !alt && !running && !platform`; a
  MATCHED chord is owned even when the op no-ops (an empty ⌃K must
  not stream 0x0B); unmatched ctrl (incl. C/D/Z) falls to the route
  byte-identically. Disjoint-field borrows through one `state`.
- No deviations. check + clippy (one unused-mut fixed) + fmt clean;
  suite 998/998.

## Phase 3.5 — Inspect
### Critic ledger (1 critic, the real input.rs compiled verbatim into
its probe — 26/26; ladder enumerated end-to-end)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| F1 | [HIGH] On an EDITOR/COCKPIT tab the six ⌃-chords edited the HIDDEN terminal's prompt (focused_terminal falls back to the first terminal pane — #71; the #251 arm's !control gate lets ⌃ fall through; efind returns false for unowned ⌃ BY DESIGN) — the #283 class upgraded from raw bytes to visible prompt corruption, against the spec's own "EDITOR untouched". | REAL | The arm now also gates on `active_tab().grid().is_some()` (the established terminal-tab idiom); non-terminal tabs revert byte-identically to the raw route. Phase 4 adds the headless editor-tab negative. |
| F2 | [MED, decision] Consecutive kills don't coalesce (readline appends ⌃W⌃W into one yank unit; probe: "aé 日x" ⌃W⌃W⌃Y → "aé " — the first kill lost); and ⌃W is bash's ⌥⌫ (word class), not unix-word-rubout ("/usr/local/bin" kills "bin"). | ACCEPTED-DOCUMENTED | D1 amended to state BOTH divergences explicitly; the append (needs last-op-was-kill state) recorded with the ring as future polish. |
| F3 | [MED] The yank `!text.is_empty()` guard is unkillable through the APP (empty can't enter the slot via kill_span) — but killable via the pub field. | REAL (test-plan) | Phase 4 adds the Some("")-injected unit (false + no edit); the guard STAYS (kill is a pub field — future writers). |
| F4 | [LOW] ⌥⌃A was claimed (alt unchecked). | REAL | `!modifiers.alt` added — ⌥⌃ combos keep streaming ESC-prefixed bytes as before. |
| F5 | [LOW] LineHome/End returned true unconditionally vs the "whether anything changed" doc — a needless notify per no-op press. | REAL | Both return MOVED-ness now. |
| F6 | [LOW] ⌃A/⌃E line-scoped vs ⌃K/⌃U buffer-scoped on a multi-line pasted prompt. | NOTED | Invisible on the single-line prompt; informational, docs line. |
| — | Cleared: the full ladder enumeration (every overlay stops before the arm; the completion popup's dismiss-then-apply is desirable); zero ctrl keymap rows (a future user row would correctly win); route negatives read-verified (running/alt → raw 0x01; ⌘⌃ skipped both sides); ONE TerminalPane constructor (kill: None on every path, restore-safe); D2 empty edges + multibyte spans + ⌃W whitespace-run semantics executed; op map closed (c/d/z → None); the launcher unreachable; no mutants::skip rebind. Unviable: `op_for_ctrl_key → Some(Default)` (no Default on ReadlineOp — the #203/#204 lesson). | — | — |
- Post-fix: check + fmt clean; suite 998/998.

## Phase 4 — Validate
- **Units (2, input.rs):** the closed op map (7 rows incl. c/d/z →
  None); the REQ-002 family — ⌃A/⌃E moved-ness (F5: a no-op press
  returns false), ⌃K mid/EOL-empty (slot preserved, D2), ⌃Y multibyte
  char-count advance, ⌃U mid/at-0, ⌃W whitespace-run + multibyte + at-0,
  the F3 Some("")-injected yank pin (false + no edit — the pub-field
  path), None-slot yank, the kill→yank round-trip.
- **Headless:** `readline_keys_at_cooked_prompt_headless` — REAL
  ctrl keystrokes: type → ⌃A/⌃E caret ends, ⌃W kills "hi" into the
  slot, ⌃Y restores, ⌃U kills the head + ⌃Y restores; then the F1
  NEGATIVE: open a file (editor tab) → ⌃K leaves the hidden prompt
  byte-identical (the grid().is_some() gate reverts non-terminal tabs
  to the raw route). REQ-004's route preservation = the untouched
  input_route goldens (keys.rs) staying green.
- **Suite:** 1006/1006 in-gate; doctests green; fmt/clippy clean.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15**
  (coverage 100 · MSI 100 · no exclusions).
- **Driven capture: ENV-BLOCKED (machine locked — black probe)** —
  the live ⌃U line-clear capture joins the unlock re-verify batch;
  the headless flow drives the same listener end-to-end.

## Phase 5 — Complete
- CHANGELOG under "### Added"; terminal.md gains the readline section
  (the op table, D1/D2 divergences, the route precedence + the F1
  terminal-tab gate).
- AAR 9aca3e2c submitted (completed; materialized:
  BF-claude-terminal-arm-reachable-from-editor-tab).
- Forge #278 closed (done); local ticket → closed/.
- Lessons: (1) any terminal-region arm that MUTATES prompt state
  gates on grid().is_some() — the (alt, running) read describes the
  HIDDEN fallback pane, not the visible surface (the #283 class);
  (2) compile-the-real-file probes (#[path] include) catch behavior
  the copy-paste probe can't drift from; (3) claim-even-when-noop is
  the right ownership rule for chord families (an empty ⌃K must not
  stream 0x0B).
