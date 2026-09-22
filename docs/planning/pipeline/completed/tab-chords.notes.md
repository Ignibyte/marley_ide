# M10 — tab chords — Notes

- **Forge ticket:** #170 `0cb3582e-7424-47a2-8c84-fd6246efa639` · **AAR:** `04f3bfca-cde2-4b88-bf97-be78e601eeab`

## Phase 1 — Plan
- Collision-checked (⌘T/⌘[/⌘digits free). Pure prev_index mirrors next_index; the arms mirror next-tab.
- **AAR id:** `04f3bfca-cde2-4b88-bf97-be78e601eeab`.

## Phase 2 — Design (folded)
- prev_index: `if len==0 {0} else if current==0 {len-1} else {current-1}`.
- keymap: 2 singles + a 1..=9 loop pushing (chord(⌘,digit), format!("switch-tab-{n}")).
- dispatch: "new-tab" → new_terminal_pane(); "prev-tab" → the next-tab arm with prev_index; else-if
  strip_prefix("switch-tab-") → parse::<usize>() → switch_tab(n-1).

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- tabs.rs prev_index (mirror of next_index; 0→len-1, len 0→0).
- keymap.rs: default_bindings now builds `let mut keymap` + appends ⌘T→new-tab, ⌘[→prev-tab, and a 1..=9 loop of ⌘N→switch-tab-N.
- app.rs: "new-tab"→new_terminal_pane; "prev-tab" mirrors next-tab with prev_index; the catch-all now parses switch-tab-N → switch_tab(n-1) (guarded).
- fmt; check 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: adversarial SELF-REVIEW (mirror-pattern additions; gate:5 adversaries prev_index).

- **Chord resolution:** digits + `[` are UNSHIFTED base keys — gpui yields key="1"/"[" with shift=false (the
  #151 trap was SHIFTED symbols; ⌘[ matches like the proven ⌘]). The keymap dispatch intercepts bound chords
  BEFORE the terminal write, so ⌘1 never reaches the PTY.
- **The catch-all rewrite** (`_ => {}` → `other => {parse switch-tab-N}`): unknown actions still no-op; the
  `n >= 1` guard prevents `n-1` underflow on a malformed "switch-tab-0"; parse::<usize> rejects junk.
- **prev_index totality:** 3 branches (len 0 / wrap at 0 / interior) — the tests hit each to kill the
  ±1 / len-1 mutants.
- **⌘T → new_terminal_pane** is the same fn as "+" and ⌘D ("split-pane", the #135 DRY) — consistent.

Lenses: chord resolution, dispatch precedence, catch-all totality, underflow, mutation coverage.

## Phase 4 — Validate
- **Tests:** prev_index_wraps (wrap/interior/empty/single); tab_chords_bound (⌘T, ⌘[, ALL of ⌘1..⌘9). 7/7 with the keymap suite.
- **Self-test:** tc170_t/prev/jump.png — ⌘T added terminal 2 (active); ⌘[ moved the highlight back to terminal 1 (its split panes intact); ⌘2 jumped to terminal 2 (full-screen). All three chords pixel-proven.
- **Gate:** GREEN [diff] 15/15, MSI 100 (prev_index mutation-killed).

## Phase 5 — Complete
- CHANGELOG + app_shell #170 note; forge #170 → done. **M10 7/10.** LESSON: collision-check chords up-front + assert ALL of a binding family in the test; parameterized actions ride the dispatch tail via strip_prefix.
