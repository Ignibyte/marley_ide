# clipboard paste + bracketed-paste — Notes

- **Forge ticket:** #42 `d1b32f42-f609-4192-8214-12a09e3e8e03`
- **AAR:** `cba47182-58ae-4665-bdf7-de42bcd9ddf9`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-042-bracketed-paste.md
- **Pipeline spec:** bracketed-paste.spec.md

## Phase 1 — Plan
- **Request:** forge #42 (M1.F "Real Interactivity" seq-3 FINALE, auto-approved) — cmd-V paste +
  bracketed-paste safety.
- **Classification / tier:** work pipeline, `feature`, two PURE surfaces (`paste_bytes` in keys.rs +
  `is_bracketed_paste` in session.rs — cov/MSI 100) + a SHIM cmd-V handler. terminal_blocks + marley_app.
- **Discovery (§18):**
  - `TermMode::BRACKETED_PASTE` (1<<4) exists in alacritty 0.26 (term/mod.rs:61) → `is_bracketed_paste`
    = `term.mode().contains(..)`, mirroring `is_alt_screen` (#33) exactly.
  - The app has NO paste today (no clipboard/cmd-v handler) — #42 adds it.
  - gpui: `cx.read_from_clipboard() -> Option<ClipboardItem>` (app.rs:1053) + `ClipboardItem::text() ->
    Option<String>` (platform.rs:1553) — the shim's clipboard read.
  - The on_key_down handler (app.rs:569) dispatches cmd chords via `keymap.action_for(binding)` (577) →
    an action name → `dispatch_action`. So cmd-V can be a keymap binding→"paste" action, OR handled
    inline (design decides).
  - Paste routing reuses #40's `is_command_running` (→ PTY vs local buffer).
  - Deps #40 (routing) + #33 (TermMode read + the `is_alt_screen_tracks_decset` mock+DCS test pattern).
- **Decisions:** D1–D3 in the spec (strip only in bracketed; is_bracketed_paste mirrors is_alt_screen;
  route via #40; pure paste_bytes/is_bracketed_paste + shim handler).
- **Open questions for Design:** keymap-binding (cmd-v→"paste", ripples the keymap test) vs inline
  cmd-v in on_key_down (no keymap ripple — leaner); the Buffer insert API for the at-prompt paste
  (find in marley_editor); whether to strip `\r` from the paste (CR→LF normalization — likely leave
  raw for the first cut).
- **AAR id:** `cba47182-58ae-4665-bdf7-de42bcd9ddf9`.
- **NOTE:** this is the LAST M1.F ticket — Phase 5 also CLOSES forge sprint #6.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/keys.rs`
```rust
/// The PTY bytes for a paste (R25): WHEN a program has bracketed paste on, wrap the text in
/// `ESC[200~`…`ESC[201~` so a multi-line paste is literal DATA (each newline is not a submitted
/// command). Every embedded `ESC[201~` is stripped first — a pasted end-marker must not close the
/// bracket early and let the tail run as commands (a paste-injection guard). Otherwise the raw bytes.
pub fn paste_bytes(text: &str, bracketed: bool) -> Vec<u8> {
    if bracketed {
        let safe = text.replace("\x1b[201~", "");
        let mut v = Vec::with_capacity(safe.len() + 12);
        v.extend_from_slice(b"\x1b[200~");
        v.extend_from_slice(safe.as_bytes());
        v.extend_from_slice(b"\x1b[201~");
        v
    } else {
        text.as_bytes().to_vec()
    }
}
```
Only the END marker (`ESC[201~`) is stripped — an embedded START marker inside the payload can't
escape the bracket, so it's harmless.

### PURE — `crates/terminal_blocks/src/session.rs`
```rust
/// Whether a program has enabled bracketed paste (DECSET 2004) (R26) — the app wraps a paste in the
/// `ESC[200~`…`ESC[201~` markers while this holds. Mirrors `is_alt_screen`.
pub fn is_bracketed_paste(&self) -> bool {
    self.term.mode().contains(TermMode::BRACKETED_PASTE)
}
```

### SHIM — `crates/marley_app/src/app.rs` (`on_key_down`, inside the mutants::skip `render`)
After the palette check, before the keymap lookup, a cmd-V case:
```rust
if event.keystroke.modifiers.platform && event.keystroke.key == "v" {
    if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
        let running = view.workspace.state(view.workspace.focused())
            .is_some_and(|s| s.session.is_command_running());
        if running {
            if let Some(state) = view.workspace.focused_state_mut() {
                let bracketed = state.session.is_bracketed_paste();
                let _ = state.session.write_bytes(&paste_bytes(&text, bracketed));
            }
        } else if let Some(state) = view.workspace.focused_state_mut() {
            // At the bare prompt: insert into the local buffer at the caret (the #28 edit path).
            state.buffer.edit(state.caret..state.caret, &text, EditOrigin::Human);
            state.caret = CharOffset::from(usize::from(state.caret) + text.chars().count());
        }
        cx.notify();
    }
    return;
}
```

### File manifest
- M `crates/terminal_blocks/src/keys.rs` — `paste_bytes` + tests.
- M `crates/terminal_blocks/src/session.rs` — `is_bracketed_paste` + a mock+DCS test.
- M `crates/terminal_blocks/src/lib.rs` — export `paste_bytes`.
- M `crates/marley_app/src/app.rs` — the cmd-V handler (imports `paste_bytes`, `EditOrigin`).
- M `docs/specs/SPEC-terminal-blocks.spec.md` (R25/R26) + SPEC-app-shell (the paste handler).
  CHANGELOG; arch docs.

### Mutation Targets
- `paste_bytes` — the `bracketed` branch; the `ESC[200~`/`ESC[201~` marker bytes (goldens); the
  `.replace("\x1b[201~","")` strip (a mutant dropping it → the embedded-marker fixture's extra `201~`
  survives → caught). No arithmetic; the `with_capacity` hint is not behavior.
- `is_bracketed_paste` — `.contains(BRACKETED_PASTE)` → other flag / true / false. Killed by the
  DECSET-2004 mock (on → true) + a fresh session (false).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `paste_bytes_plain_and_wrapped` — `("hi", false)` → `b"hi"`; `("hi", true)` → `ESC[200~hi ESC[201~`; `("a\nb", true)` → wrapped with the `\n` preserved (multi-line as data) | unit (keys.rs) |
| REQ-002 | `paste_bytes_strips_embedded_end_marker` — `("x\x1b[201~y", true)` → `ESC[200~xy ESC[201~` (the embedded marker removed → exactly one closing marker) | unit |
| REQ-003 | `is_bracketed_paste_tracks_decset` — mock feeds `\x1b[?2004h` → pump → true; a fresh session → false | unit (session.rs) |
| REQ-004 | the cmd-V handler (clipboard → route) | shim + masked visual (chad pastes multi-line into an editor → one block) |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs cmd-V handler (mutants::skip render; needs a live clipboard + window).

### Risks / decisions
- D-2.1 Strip ONLY `ESC[201~` (the security-critical end marker) — an embedded start marker is
  harmless. D-2.2 cmd-V handled INLINE in on_key_down (no keymap ripple — leaner than a bound action).
  D-2.3 At the bare prompt, paste inserts into the local buffer via the #28 `edit(caret..caret, …)`
  path (multi-line-at-the-prompt is a rough edge — the richer local editing is deferred; the primary
  value is pasting into a RUNNING program). D-2.4 Route via #40's `is_command_running`.
- `is_bracketed_paste` reflects the FOREGROUND program's mode; at the prompt it's typically false, but
  the prompt path doesn't wrap anyway (it inserts into the buffer). No CR/LF normalization in the
  first cut (raw text) — a later refinement if a program is picky.

## Phase 3 — Implement
- **Built (per manifest):** keys.rs — `paste_bytes(text, bracketed)` (bracketed → `ESC[200~` +
  `text.replace(ESC[201~, "")` + `ESC[201~`; else raw); lib.rs export; session.rs —
  `is_bracketed_paste()` = `term.mode().contains(TermMode::BRACKETED_PASTE)`; app.rs — the inline
  cmd-V handler in on_key_down (clipboard read → `running ? write_bytes(paste_bytes(text,
  is_bracketed_paste())) : buffer.edit(caret..caret, text) + advance caret`). SPEC-terminal-blocks
  R28 + mutation targets; SPEC-app-shell R46; CHANGELOG (### Added).
- **Deviations from design:** the caret advance uses `state.caret.as_usize()` (the CharOffset API),
  not `usize::from(caret)` — `CharOffset` impls `From<usize>` but not the reverse (caught at compile).
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0
  (terminal + marley); docs gate 0; 92 terminal tests pass. The `paste_bytes` + `is_bracketed_paste`
  unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a working exploit probe + a real scoped cargo-mutants + an alacritty/vte
  parser-path trace). Verdict: **BLOCK → the HIGH security bug is real; FIXED.**
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH (security) | Marker RECONSTITUTION: the single `str::replace("\x1b[201~","")` strip is bypassable — `ESC[20`+`ESC[201~`+`1~` reconstitutes a live `ESC[201~` after one pass (str::replace never re-scans). PoC: clipboard `ESC[20 ESC[201~ 1~ rm -rf ~\n` → the interior marker closes the bracket → `rm -rf ~` runs. The exact injection the guard should prevent. | REAL (proven exploit) | Loop until stable: `while safe.contains("\x1b[201~") { safe = safe.replace("\x1b[201~", ""); }`. Verified 0 residual on split/nested/adjacent/1000×-stress (inline rustc probe). Terminates (each pass shrinks). |
  | F2 | MED (Phase-4) | MSI is STRUCTURALLY BLIND to the guard — cargo-mutants generates only 3 whole-fn-return mutants on `paste_bytes` (`vec![]`/`[0]`/`[1]`), NONE on the `.replace` strip. So MSI 100 stays green even if the guard is deleted; a naive single-embedded-marker golden passes on buggy AND fixed code. | REAL (the gate can't force it) | P4 adds a SPLIT-MARKER structural regression golden: `paste_bytes("\x1b[20\x1b[201~1~", true)`'s wrapped body has NO interior `ESC[201~` — a hand-written test, not mutation-forced. |
- **Verified CLEAN by the critic:** the normal goldens exact (`("hi",false)`→`b"hi"`, `("hi",true)`→
  wrapped, `("a\nb",true)` preserves `\n`); `is_bracketed_paste` correct + killable (traced
  vte→alacritty: DECSET 2004 → `mode.insert(BRACKETED_PASTE)`; feed `ESC[?2004h`); no panic/UTF-8
  issue (ASCII markers); the shim routes via #40, caret math correct, no panic.
- **Knowledge captured:** `BF-…-single-str-replace-sanitizer-reconstitutes-split-tokens` (id
  4a200e45, category security) + `PR-…-sanitizer-must-loop-until-stable-single-replace-reconstitutes`
  (a single replace isn't a safe sanitizer; loop until stable + a structural test since MSI can't
  force it — the sanitizer cousin of the invariant-invisible-logic rule).
- **Post-fix:** the loop compiles + clippy clean; the reconstitution is closed (probe: 0 interior
  markers). F2 is the Phase-4 split-marker golden.

## Phase 4 — Validate
- **Tests added:** `keys.rs` — `paste_bytes_plain_and_wrapped` (raw/wrapped/multi-line goldens),
  `paste_bytes_strips_embedded_end_marker`, and `paste_bytes_resists_split_marker_reconstitution`
  (the SECURITY regression — the F1 exploit input `ESC[20 ESC[201~ 1~ rm -rf ~`; asserts exactly ONE
  end-marker, at the very end, none interior — the hand-written guard MSI can't force, per F2).
  `session.rs` — `is_bracketed_paste_tracks_decset` (mock feeds `ESC[?2004h` → true; off by default).
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 96 passed (all #42 tests PASS);
  `cargo nextest run --workspace` → 516 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **5 caught /
  0 missed → MSI 100.0%** (paste_bytes's 3 return-mutants + is_bracketed_paste's 2). Receipt written.
  No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `terminal_blocks.md` — the paste bullet.
  SPEC-terminal-blocks R28 + SPEC-app-shell R46 at implement.
- **Knowledge captured:** `BF-…-single-str-replace-sanitizer-reconstitutes-split-tokens` (id 4a200e45,
  security) + `PR-…-sanitizer-must-loop-until-stable-single-replace-reconstitutes` (a single replace
  isn't a safe sanitizer; loop until stable + a structural test since MSI can't force it). aar-submit
  `completed` (score 5). **The headline:** the adversarial inspect caught a real, exploitable
  paste-injection (`rm -rf ~` via a split end-marker) that all my goldens + MSI 100 would have MISSED
  — the single biggest save of the sprint, and exactly why the critic phase runs a real exploit probe.
- **Ticket:** forge #42 → done; local doc → closed/; pipeline pair archived. **3 of 3 in M1.F.**
- **SPRINT:** forge sprint #6 "M1.F — Real Interactivity" CLOSED. Interactive programs now work:
  input routes to running programs (#40), the full key vocabulary reaches them (#41), and paste is
  safe (#42).
