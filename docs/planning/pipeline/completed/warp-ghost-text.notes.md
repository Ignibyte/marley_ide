# Inline history ghost-text — Notes

- **Forge ticket:** #200 (841e188f-5de7-47a0-93a1-c36647b9b243)
- **AAR:** b01def64-0cb9-46f4-a4ee-f1183baf5fec
- **Local ticket doc:** docs/planning/tickets/open/TICKET-200-warp-ghost-text.md
- **Pipeline spec:** warp-ghost-text.spec.md

## Phase 1 — Plan
- **Request:** fish/Warp-style inline history ghost-text at the prompt. Auto-approved (/work 195-222, M12.2
  polish).
- **Classification / tier:** work pipeline, small feature. Systems: history.rs (or prompt.rs — the pure
  `suggest`) + app.rs prompt render (the ghost span) + input dispatch (the → accept). Reuses #29 history,
  #218 block cursor.
- **Forge recall (§18.3):** #29 built `CommandHistory`; #183 the Tab popup; #218 the block-cursor prompt
  render (`split_caret_char`, before|cursor|after). The exact-value string rule (prefix/suffix boundaries)
  + the mutation-list rule ([[PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators]] — RUN
  `cargo mutants --list` for the real set) apply. AAR opened.
- **Discovery (code read):**
  - history.rs: `CommandHistory { entries: Vec<String>, capacity, cursor, draft }`; **`recent(&self) ->
    Vec<&str>` is MOST-RECENT-FIRST + de-duplicated** (iter().rev().filter(seen.insert)) — so `suggest`
    takes the FIRST prefix match (newest wins, no reversal). PaneState.history.
  - The prompt render (app.rs ~4650-4695): the #218/#220 block-cursor inner flex `before | cursor(block) |
    after` (from `split_caret_char(&buffer.text(), caret)`). At EOL `after` is empty → the ghost span
    appends after it, flush past the cursor.
  - input.rs `apply_key`: `Key::Right => *caret = move_char_right(buffer, *caret)` (86) — at EOL this is a
    no-op. The dispatch (app.rs ~3649) `match apply_key(&mut state.buffer, &mut state.caret, key)` — the
    ghost/history context is on PaneState, so the accept check lives in the DISPATCH (before/around apply_key
    for Right), not in the pure apply_key.
  - `self.completion: Option<(PaneId, CompletionState)>` (app.rs:177) — the #96 tab popup; the ghost is
    suppressed when this is `Some` for the pane.
- **The delta:** pure `suggest(prefix, &[&str]) -> Option<String>` (first strict-prefix match's suffix;
  None on empty/no-match/exact); render the muted suffix at EOL (not while completion open); Right at EOL
  with a ghost inserts the suffix (else the normal move). Maybe a pure `accept_suffix` (= `suggest`, reused)
  so the accept path is tested; the shim does `buffer.edit`.
- **Deferred:** ⌘→ accept; mid-line ghost; fuzzy/substring; non-history sources.
- **Decisions:** D1 EOL-only; D2 suppress under the tab popup; D3 display-only until accepted; D4 recent()
  newest-first (first match wins); D5 auto-approved, document w/ a ghost+accept capture.
- **Open questions for Design:** (1) `suggest` home — history.rs (beside `recent`, it's history's concern)
  vs prompt.rs (the prompt's concern); recommend history.rs (pure, gpui-free, has the entries). (2) the
  accept: is it `suggest` reused at Right-dispatch (compute the suffix again, insert it) — yes, simplest,
  no new pure fn; OR a distinct `accept_suffix`. (3) the caret-at-EOL check: `caret.as_usize() ==
  buffer.text().chars().count()` — confirm the char-count EOL test. (4) the ghost span color: `muted`
  (the deferred/hint color) — confirm it reads dim-but-visible after the bright block cursor. (5) `suggest`
  input type — `&[&str]` (matches `recent()`) vs `&[String]`; use `&[&str]`.

## Phase 2 — Design

### Discovery verified
- **The Right dispatch** (app.rs ~3646-3660): `let key = key_from_keystroke(&event.keystroke); if let Some(state)
  = view.workspace_mut().focused_terminal_mut() { match apply_key(&mut state.buffer, &mut state.caret, key) {
  Edited/Submit/Ignored } }`. The ghost-accept must intercept `key == Key::Right` at EOL+ghost BEFORE
  `apply_key` (the ghost/history/completion context is on PaneState/self, not in the pure `apply_key`).
- **The insert idiom** (input.rs:62 Char): `buffer.edit(at..at, &s, EditOrigin::Human); *caret =
  CharOffset::from(caret.as_usize() + s.chars().count())`. The recall path (app.rs:3637) uses `state.buffer
  = Buffer::from_text(&line); state.caret = CharOffset::from(line.chars().count())`. Either works for the
  accept insert.
- **The EOL test:** `state.caret.as_usize() == state.buffer.text().chars().count()`.
- **Completion suppression:** `self.completion: Option<(PaneId, CompletionState)>` — suppress the ghost when
  `self.completion.is_some()` (per-pane; the ghost is only on the focused pane's prompt anyway).
- **The render** already sits inside the `prompt_visible` (#193 `!is_command_running`) block — so the ghost
  is naturally gated to when the prompt is shown (not while a foreground command runs).
- **`suggest` mutant set (RAN `cargo mutants --list -f history.rs` on a stub — the definitive set):** body →
  `{None, Some(String::new()), Some("xyzzy")}`; `&&` → `||`; `>` → `{==, <, >=}`. (NOT `<=`/`!=` — confirms
  [[PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators]].) Traced my 6-case matrix kills ALL:
  body-3 by T1 (a Some case); `&&`→`||` by T2 (`"echo"` starts_with true but len> false → `and` skips → " hi",
  `or` matches → "" — differ); `>`→`==`/`<` by T1, `>`→`>=` by T2. No gap.

### Architecture / approach
- **PURE seam** (history.rs, beside `recent`, cov/MSI 100):
  ```
  pub fn suggest(prefix: &str, history: &[&str]) -> Option<String> {
      if prefix.is_empty() { return None; }
      history.iter()
          .find(|entry| entry.starts_with(prefix) && entry.len() > prefix.len())
          .map(|entry| entry[prefix.len()..].to_string())
  }
  ```
  Multi-byte SAFE: `starts_with(prefix)` guarantees prefix's BYTES are a prefix of entry → `prefix.len()`
  (bytes) is a valid char boundary in entry → `entry[prefix.len()..]` never panics mid-codepoint. `recent()`
  is newest-first + de-duped → `.find` takes the most-recent strict-prefix match. The accept REUSES `suggest`
  (no separate pure fn).
- **SHIM** (app.rs, mutants::skip):
  - **RENDER** (the #218 prompt input-row, ~4684, after the `before|cursor|after` inner flex): compute
    `let ghost = if state.caret.as_usize() == state.buffer.text().chars().count() && self.completion.is_none()
    { crate::history::suggest(&state.buffer.text(), &state.history.recent()) } else { None };` then if
    `Some(g)`, append `.child(div().text_color(colors.muted).child(g))` INSIDE the inner gapless flex after
    the `after` child (so it's flush after the block cursor). **Decision (i):** the ghost sits AFTER the block
    cursor (the cursor is on the EOL space, the muted ghost follows) — bounded; the Warp-style cursor-on-first-
    ghost-char (reverse-video the first ghost char in the cursor) is a polish follow-up (would touch the #218
    cursor). Document the choice.
  - **ACCEPT** (the Right dispatch, ~3647, before `apply_key`): when `key == Key::Right`, check the focused
    state: `let text = state.buffer.text(); let at_eol = state.caret.as_usize() == text.chars().count();` and
    `if at_eol && self.completion.is_none() { if let Some(suffix) = crate::history::suggest(&text,
    &state.history.recent()) { state.buffer.edit(state.caret..state.caret, &suffix, EditOrigin::Human);
    state.caret = CharOffset::from(state.caret.as_usize() + suffix.chars().count()); cx.notify(); return; } }`
    — else fall through to the normal `apply_key(…, Key::Right)`. (A mid-line Right, or Right with no ghost, is
    the normal move — unchanged.)
- Display-only until accepted (the ghost is NEVER in the buffer until Right inserts it).

### File manifest
- `crates/marley_app/src/history.rs` — ADD `pub fn suggest(prefix: &str, history: &[&str]) -> Option<String>`
  (module-level or an assoc fn? — a FREE fn beside `recent`'s impl, or a `CommandHistory` method taking
  &prefix; RECOMMEND a free `pub fn suggest(prefix, history)` at module scope, so it's pure over a slice + easy
  to test without a CommandHistory). Tests at validate.
- `crates/marley_app/src/app.rs` — (a) the RENDER ghost span in the prompt input-row (~4684); (b) the ACCEPT
  intercept in the Right dispatch (~3647). Import `suggest` (or call `crate::history::suggest`).

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-005 (suggest) | history.rs `suggest_returns_the_recent_prefix_suffix` — the 6-case matrix (RAN vs the real mutant list): ("echo",["echo hello","ls"])→Some(" hello"); ("echo",["echo","echo hi"])→Some(" hi") [strict-skip + first-match, kills `&&`→`||` & `>`→`>=`]; ("",["echo"])→None; ("xyz",["echo"])→None; ("echo",["echo"])→None [exact]; ("caf\u{e9}",["caf\u{e9} bar"])→Some(" bar") [multibyte]. Kills body-3 + `&&`→`||` + `>`→{==,<,>=}. |
| REQ-001 | driven capture — record `echo hello`, type `echo` → a muted ` hello` ghost after the caret at EOL. |
| REQ-002 | driven capture — with the ghost live, press → → the line becomes `echo hello` (suffix inserted, caret at end). |
| REQ-003 | unit (suggest None for empty/no-match/exact) + capture — no ghost when the prefix doesn't match. |
| REQ-004 | review + capture — the ghost is suppressed while the #96 tab popup is open (`completion.is_some()`). |
- **Uncoverable by unit test:** the render span + the Right-accept intercept (`mutants::skip` render/dispatch)
  — validated by the driven captures; `suggest` carries the mutation load.

### Risks / decisions
- **R1 — the accept must intercept Right ONLY at EOL+ghost+no-popup**, else it's the normal move. A mid-line
  Right (caret < EOL) or a Right with no suggestion falls through to `apply_key(Key::Right)`. Verified the
  fall-through path.
- **R2 — ghost display-only** — never written to the buffer until Right; `suggest` is a read (over
  `buffer.text()` + `recent()`), no mutation. The render is display-only.
- **R3 — the cursor-on-ghost visual** (Warp reverse-videos the first ghost char in the cursor) — DEFERRED
  (decision (i): ghost after the cursor). A follow-up; not a blocker.
- **R4 — history detach** — typing after a recall detaches history (#R36, app.rs:3653); `suggest` reads
  `recent()` (the full de-duped history), independent of the recall cursor — so the ghost works whether or
  not you've been ↑/↓-recalling. No interaction.
- **R5 — the `&&` short-circuit** — `starts_with(prefix) && entry.len() > prefix.len()`: starts_with first
  (cheap-ish), then the len (O(1)). The `&&`→`||` mutant is killed by T2 (the test guards the AND).

## Phase 3 — Implement
- **history.rs** — added module-level `pub fn suggest(prefix: &str, history: &[&str]) -> Option<String>`
  (empty→None; `.find(starts_with && len>)`.map(slice-after-prefix)) after the `CommandHistory` impl. Pure.
- **app.rs** — (a) the ACCEPT intercept in the key-edit dispatch (after `let key = key_from_keystroke`):
  `if key == Key::Right && view.completion.is_none()` → focused state → at-EOL → `suggest` Some → `buffer.edit(
  caret..caret, &suffix, Human)` + advance caret + notify + return (else falls through to the normal
  `apply_key`). (b) the RENDER ghost: computed after `split_caret_char` (`if caret==char-count && !completion_open
  { suggest(...) }`), appended as `.children(ghost.map(|g| div().text_color(colors.muted).child(g)))` after the
  `after` child in the #218 inner flex.
- **Deviation (borrow fix):** the render's ghost check first used `self.completion.is_none()`, but the pane
  loop holds a mutable `workspace_mut().terminal(pane_id)` borrow → E0502 (can't borrow `self` immutably).
  Fixed by SNAPSHOTTING `let completion_open = self.completion.is_some();` beside the existing `find_open`/
  `find_query` render snapshots (~3259) + using `!completion_open` in the render. (The ACCEPT path is in the
  event closure with `view: &mut RootView`, not the render's borrow, so `view.completion.is_none()` is fine
  there.) This matches the established "snapshot self-flags before the pane loop" pattern.
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note). The
  `.children(Option<Div>)` form compiled (Option is an IntoIterator of elements).

## Inspect (Phase 3.5)
1 focused critic (suggest correctness + accept-intercept + render/divergence + scope) + my own structural
verification. **No confirmed findings.** (The critic ran + was mid-verification of the borrow sequencing +
return placement when the ledger was written; I verified the same points directly from the code below — the
compiler already confirmed the borrows, and the fall-through is structural.)

**Self-review (evidence-backed):**
- **suggest correctness + panic-safety — VERIFIED.** Multi-byte safe: `starts_with(prefix)` guarantees
  prefix's BYTES are a prefix of the matched entry ⇒ `prefix.len()` (bytes) is a valid char boundary ⇒
  `entry[prefix.len()..]` never panics mid-codepoint. `.find` returns the FIRST (most-recent, `recent()` is
  newest-first) strict match; empty-prefix → None before the find; `entry.len() > prefix.len()` skips an
  exact-equal entry. MSI 100: I RAN `cargo mutants --list` at design — the real set is body→{None,Some(""),
  Some("xyzzy")}, `&&`→`||`, `>`→{==,<,>=} — the 6-case validate matrix kills every one (traced).
- **Accept-intercept correctness (the KEY check) — VERIFIED structurally.** The `return` (app.rs:3670) is
  INSIDE `if let Some(suffix)` (3658) — hit ONLY on a successful accept. Every miss falls through to the
  existing `apply_key` dispatch (3675): (a) non-Right → the `key == Key::Right` guard (3653) is false → the
  whole block skipped; (b) popup open → `view.completion.is_none()` (3653) false → skipped; (c) mid-line
  (caret < char-count) → `at_eol` (3656) false → no insert; (d) no matching history → `suggest` None →
  no insert. All four reach `apply_key(Key::Right)` (a no-op at EOL, the normal move mid-line). **No
  double-borrow:** the intercept's `if let Some(state)` (3654) scope ENDS at 3673, BEFORE the dispatch's own
  `if let Some(state)` (3675) — sequential borrows (compiler-confirmed: `cargo check` clean). The insert:
  `buffer.edit(caret..caret, &suffix, Human)` + `caret = from(caret.as_usize() + suffix.chars().count())`
  (chars, not bytes — multi-byte-correct) → caret at the new EOL, buffer = prefix+suffix = the full history
  entry. Display-only until accepted (`suggest` is a read; the buffer isn't mutated until →).
- **Render + no divergence — VERIFIED.** The render uses the `completion_open` SNAPSHOT (3259, beside
  `find_open`) — avoids the E0502 (the pane loop mutably borrows `workspace_mut().terminal(pane_id)`, so the
  render can't borrow `self.completion`). The render's ghost predicate (3684: `caret==chars().count() &&
  !completion_open` + `suggest` Some) and the accept's (3653/3656: `Key::Right && completion.is_none() &&
  at_eol` + `suggest` Some) are the SAME logical condition over the SAME `suggest(&<buffer text>,
  &state.history.recent())` — so the ghost you SEE is exactly what → accepts. They run at different times
  (render per-frame, accept on the → event), but the → re-reads fresh state → accepts whatever ghost is
  current at the press. No divergence.
- **Scope / clean-room — VERIFIED.** The diff's `+` lines are ONLY: `suggest` (history.rs), the snapshot,
  the ghost render, the accept intercept (app.rs). NO change to `apply_key`, recall (↑/↓), the tab popup, or
  submit (the `Char` edit still detaches history at 3680, unchanged). Muted token only — no new hsla/hex.

**Critic returned (after this ledger draft) with 2 REAL MED findings my self-review MISSED — both FIXED at
validate (source fixes for genuine defects, gate re-green):**
- **[MED — FIXED] the ghost rendered on UNFOCUSED panes.** The render condition (`caret==char-count &&
  !completion_open`) was NOT gated on `is_focused` (which IS in scope in the pane loop), yet the ACCEPT is
  focused-only (`focused_terminal_mut()`) — so a split pane at EOL showed a ghost that → could never accept
  (render/accept divergence across PANES), and it diverges from Warp (autosuggest is active-input only).
  There's an in-file precedent: the #186 find-highlight is focused-only for exactly this reason. **Fix:**
  added `is_focused &&` to the ghost condition (mirrors #186). I MISSED this — my divergence check reasoned
  about time (same-frame render vs the → event) but not about the MULTI-PANE case. Good catch.
- **[MED — FIXED] the ghost-accept omitted `history.detach()`.** Every other buffer-editing path calls
  `state.history.detach()` (the R36 contract, app.rs ~3680 for Char/Backspace/DeleteForward), but the accept
  branch mutated the buffer WITHOUT detaching. Repro: mid-recall (↑ to "git", cursor Some(0)), accept the
  ghost → "git push" but cursor still Some(0); a subsequent ↑ recalls "git" and DISCARDS the accepted "git
  push" (never stashed). A genuine R36 violation. **Fix:** added `state.history.detach();` in the accept
  branch before `cx.notify()` (mirrors ~3680). I MISSED this — I checked the accept's borrow/return but not
  its parity with the R36 detach contract the other edit paths honor.
- **[LOW — FIXED] `text()` recomputed 3×/frame** (rope→String) + `recent()` per render. **Fix:** hoisted
  `let line = state.buffer.text();` once, reused by the caret split + EOL check + suggest.
- **[LOW — accepted] block cursor sits BETWEEN text and ghost at EOL** (`git[█] push` vs the cursor overlaying
  the first ghost char, fish-style) — the design pre-accepted this (decision (i)); a polish follow-up.

**Verdict:** 2 MED + 1 LOW fixed at validate (all consistency-with-established-pattern defects the critic
surfaced + I missed); the critic VERIFIED the rest (suggest byte-safety/mutation, the accept fall-through +
borrow sequencing, the completion snapshot, scope/clean-room). No forge failure-record (caught pre-ship at
inspect→validate). LESSON: my self-review under-weighted the MULTI-PANE case (a render in the pane loop
needs an `is_focused` gate like #186) + the R36 detach-on-edit contract — an adversarial critic caught both.

## Phase 3.5 — Inspect
- (superseded by "## Inspect (Phase 3.5)" above)

## Phase 4 — Validate
- **Unit test (REQ-005):** added `suggest_returns_the_recent_prefix_suffix` in history.rs — the 6-case matrix
  verified vs the REAL `cargo mutants --list` set (design): basic→Some(" hello"); strict-skip+first-match
  ("echo",["echo","echo hi"])→Some(" hi") [kills `&&`→`||` + `>`→`>=`]; empty→None; no-match→None;
  exact→None; multi-byte ("caf\u{e9}",["caf\u{e9} bar"])→Some(" bar"). `cargo nextest run -p marley` = **305
  passed, 2 skipped** (304 + this); isolated run confirms `history::tests::suggest_returns_the_recent_prefix_suffix`
  PASS.
- **Driven captures (live app; RE-BUNDLED at inspect; #198 focus-first lesson applied):** (the app is on the
  Light theme, persisted from #199 — which usefully makes the muted ghost's dimness obvious vs the dark text.)
  - **REQ-001** `scratchpad/200-ghost.png` (+ `-crop`): ran `echo hello` (into history), typed `echo` → the
    prompt shows `❯ Marley echo` (bright) + the block cursor + a DIMMED muted ghost ` hello` after it. The
    inline history suggestion.
  - **REQ-002** `scratchpad/200-accept.png` (+ `-crop`): pressed → → the ghost was ACCEPTED — the line reads
    `❯ Marley echo hello` in full bright text, block cursor at the new EOL, and the ghost is GONE (the buffer
    now == the full history entry → `suggest` returns None on the exact match).
  - **REQ-003** `scratchpad/200-noghost.png` (+ `-crop`): appended ` zzz` → `echo hello zzz`, which no history
    entry starts with → NO ghost. (The exact-match no-ghost is also shown by the accept capture.)
  - **REQ-004 (popup suppression):** code-verified — the render + accept both gate on `!completion_open` /
    `completion.is_none()`; a full tab-popup-open driven scenario is a separate interaction (the gate + the
    code path cover it; the shim is capture-validated for the primary flow).
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN; cov/MSI 100 on `suggest` (the 6-case matrix on
  the real mutant set). **Re-run GREEN after the 2 MED + 1 LOW inspect-critic fixes** (the focused-only ghost
  gate + the R36 `history.detach()` on accept + the hoisted `text()` — all focused-pane-only correctness that
  don't alter the single-pane ghost/accept flow already captured; 305 tests pass post-fix).
- **Pre-existing exclusions:** none (the `block v0.1.6` note is upstream).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #200 under [Unreleased]/Added (below #199). app_shell.md — the history.rs
  (#29) entry extended with the #200 ghost-text note (suggest, the focused/EOL/!popup gates, the Right-accept
  + detach).
- **Knowledge (forge):** `aar-submit` b01def64 (completed, effectiveness 5). **Two failure-records** (the
  inspect critic's real catches): `BF-claude-200-ghost-renders-on-unfocused-panes-001` (ed34f8e5) +
  `BF-claude-200-ghost-accept-omits-history-detach-001` (00eb9357). **Prevention rule** →
  `PR-claude-per-pane-render-affordance-must-gate-on-is-focused-001` (21f265a8): a focused-only affordance in
  the per-pane render loop must gate on `is_focused` (mirrors #186); check BOTH the time-axis AND the
  multi-pane axis of divergence.
- **Lessons:** (1) the adversarial critic earned its keep — it caught 2 real MED defects my self-review
  missed (the unfocused-ghost + the missing R36 detach), both "consistency with an established pattern"
  (#186 focus-gate, the detach-on-edit contract). My self-review checked the local mechanics (borrow, return,
  same-frame divergence) but not the cross-pane + cross-contract consistency. (2) the mutation-list rule paid
  off again — I RAN `cargo mutants --list` on a suggest stub at DESIGN, so the validate matrix hit MSI 100
  first try (no phantom-mutant chasing). (3) the #198 focus-first self-test lesson held — every driven ghost/
  accept capture landed on Marley.
- **Close/archive:** TICKET-200 open→closed; forge ticket-close #200 done; pipeline pair → completed/.
