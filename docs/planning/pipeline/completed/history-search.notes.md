# command history fuzzy search (cmd-R) — Notes

- **Forge ticket:** #60 `9620ea3a-0ade-44c7-acb0-8945b8f64adb`
- **AAR:** `a697bf07-4d48-4ba3-b114-34ed89c9844c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-060-history-search.md

## Phase 1 — Plan
- **Request:** forge #60 (M2.B seq-2, auto-approved) — cmd-R history search. 2nd search_core consumer.
- **Classification:** work pipeline, `feature`, small PURE (recent() + keymap) + REUSE FinderState + an
  app.rs SHIM. UI — validate self-test-captures.
- **Fork resolved (D1):** cmd-r → open-history-search; rerun-last → cmd-shift-r. "rerun-last" dispatch
  (app.rs:689) stays reachable via cmd-shift-r; the ↻ block click (`rerun_command`, app.rs:1366) is
  independent (per-block rerun) — unaffected.
- **Reuse:** `CommandHistory` has `entries: Vec<String>` (private), `record`/`recall_prev/next` — NO
  public accessor, so add `recent()`. `FinderState` (finder.rs) is `&str`-based → reuse for the history
  state directly.
- **#59 lesson applied (D3):** the Enter (chosen command) inserts into the cooked buffer via
  `buffer.edit`, NOT `write_bytes` — else invisible in cooked mode (PR-claude-insert-at-prompt-goes-to-
  cooked-buffer-not-raw-pty-001). Same as #59's file-click fix.
- **AAR id:** `a697bf07-4d48-4ba3-b114-34ed89c9844c`.

## Phase 2 — Design

### PURE-1 — `history.rs` (CommandHistory impl)
```rust
    /// The submitted commands MOST-RECENT-FIRST, de-duplicated (the most-recent occurrence of a repeat
    /// wins) — the source list for the cmd-R fuzzy history search.
    pub fn recent(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        self.entries
            .iter()
            .rev()
            .filter(|cmd| seen.insert(cmd.as_str()))
            .map(String::as_str)
            .collect()
    }
```

### PURE-2 — `keymap.rs` default_bindings
- CHANGE the existing `(chord(true,false,false,false,"r"), "rerun-last")` → chord shift=TRUE
  (`chord(true,false,false,true,"r")`, cmd-shift-r) keeping "rerun-last".
- ADD `(chord(true,false,false,false,"r"), "open-history-search")` (bare cmd-r).
- keymap test: `action_for(cmd-r)==Some("open-history-search")` + `action_for(cmd-shift-r)==Some("rerun-last")`.

### PURE-3 — REUSE `FinderState` (finder.rs, unchanged)
The history state is a `FinderState` (query/selected/push/backspace/move_up/move_down/results). Its
`results(&[&str])` ranks the history commands. `chosen` (PathBuf-specific) is NOT used — the shim
resolves the picked command inline: `results.get(selected).and_then(|&i| recent.get(i))`.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `RootView { history_open: bool, history_finder: FinderState }`.
- Dispatch `"open-history-search"` → `self.history_finder = FinderState::new(); self.history_open = true`.
- Key routing: gate `history_open` FIRST (before finder/palette) → `handle_history_key`.
- `handle_history_key` (mirror `handle_finder_key`): build `recent: Vec<String>` (owned, from the focused
  session's `history.recent()`); `results = history_finder.results(&recent_refs)`. `escape`→close;
  `enter`→ resolve `recent[results[selected]]`, INSERT it into the cooked buffer
  (`state.buffer.edit(caret..caret, &cmd, Human)` + advance caret — per #59, NOT write_bytes), close;
  `up`/`down`→move (down uses `results.len()`); `backspace`/printable→edit query.
- Overlay (mirror the cmd-P finder overlay): the query line + the ranked `recent[results[i]]` rows, the
  `history_finder.selected()` row highlighted; rendered when `history_open`.

### File manifest
- MODIFY `crates/marley_app/src/history.rs` — `recent()` + a test.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-r/cmd-shift-r swap + test.
- MODIFY `crates/marley_app/src/app.rs` — RootView fields, dispatch, handle_history_key, key routing, overlay.

### Mutation Targets (pure)
- `recent()`: `.rev()` (order — a `[a,b,a]`→`[a,b]` test with distinct order catches drop-rev), the
  `seen.insert` dedup filter (drop → duplicates leak), `.map`. keymap: the two `action_for` arms.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `recent_most_recent_first_deduped` — `record(a);record(b);record(a)` → `recent()==["a","b"]`; empty → `[]` | unit |
| REQ-002 | keymap `cmd_r_history_cmd_shift_r_rerun` — `action_for(cmd-r)=="open-history-search"`, `action_for(cmd-shift-r)=="rerun-last"` | unit |
| REQ-003 | cmd-R overlay lists ranked history; Enter inserts at prompt | self-test (drive commands, cmd-R, capture; Enter) |
| REQ-004 | gate GREEN, cov/MSI 100 recent()+keymap; app shim excluded | gate |

Uncoverable: the app.rs cmd-R overlay/key-routing/insert — masked + cov-excluded, proven by REQ-003.

### Risks / decisions
- D-2.1 `recent()` returns `Vec<&str>` borrowing `self.entries` — the shim clones to owned Strings before
  `buffer.edit` (avoids a borrow of self.workspace while mutating it), as in #59. D-2.2 rerun-last stays
  reachable (cmd-shift-r) so app.rs:689's dispatch arm isn't orphaned (no dead code). D-2.3 Enter inserts
  (cooked buffer) — run-on-enter deferred; consistent with #59's insert-not-run.

## Phase 3 — Implement
- **Built (PURE):** `history.rs` `CommandHistory::recent()` (rev + HashSet-dedup + map). `keymap.rs` —
  cmd-r → `open-history-search`, added cmd-shift-r → `rerun-last`; the keymap test updated to assert both.
- **Built (SHIM, app.rs — mutants::skip):** `RootView { history_open, history_finder: FinderState }` (+
  new()); dispatch `open-history-search`; key routing gates `history_open` BEFORE finder/palette;
  `handle_history_key` (Enter inserts the chosen command into the cooked buffer via `buffer.edit`, per
  #59 — NOT write_bytes); a 🕐 cmd-R overlay (mirrors the finder). Source = focused session's `history.recent()`.
- **Deviations:** none — `FinderState` reused as-is (chosen resolved inline over `&str`, since its
  `chosen` is PathBuf-specific).
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 123 pass (keymap test updated, no regression). recent()+keymap tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (verbatim recent() probe + a keymap action_for probe + real cargo-mutants + a dispatch
  reachability trace). Verdict: **PASS — pure logic correct, no defect.**
- **Findings:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | MED (test mandate) | recent()'s 3 mutants are all whole-body constant returns (`vec![]`/`vec![""]`/`vec!["xyzzy"]`) — cargo-mutants makes NO `.rev()`/`.filter` mutant. So a SINGLE non-empty case hits MSI 100, but MSI 100 would NOT verify order OR dedup. | P4 ships BOTH `[a,b,c]→[c,b,a]` (pins `.rev()`) AND `[a,b,a]→[a,b]` (pins the dedup) + `[]→[]`, regardless of MSI — else a future drop of `.rev()`/the filter passes green. (My REQ-001 already has both.) |
  | F2 | LOW | Overlay caps at `.take(20)` while move_down/Enter use full len → selection can go off-screen (highlight vanishes; Enter still inserts the right command). | Accept — IDENTICAL to the existing finder overlay (#57), the sanctioned pattern. |
- **Verified (probe/trace):** recent() `[a,b,c]→[c,b,a]`, `[a,b,a]→[a,b]` (newest wins), `[x,a,b,a]→[a,b,x]`
  (keep-most-recent discriminator vs keep-oldest), `[]→[]`, capacity-evict, `record`-collapses-adjacent;
  `action_for(cmd-r)=open-history-search` + `(cmd-shift-r)=rerun-last` (no collision, Eq incl shift);
  **rerun-last NOT orphaned** (dispatch app.rs:745 reachable via cmd-shift-r); **Enter inserts into the
  COOKED buffer** (`buffer.edit`, per #59, NOT write_bytes); borrow clean (owned recent+chosen before
  focused_state_mut); routing mutually exclusive (history_open gated first); handle_history_key
  mutants::skip + app.rs cov-excluded. keymap test updated + 3 pass; no other test asserted the old binding.
- **No code change** — F1 = the P4 order+dedup tests; F2 accepted.

## Phase 4 — Validate
- **Tests added:** `history.rs` `recent_most_recent_first_deduped` (F1 — BOTH `[a,b,c]→[c,b,a]` order +
  the re-run-a dedup `→[a,c,b]` keep-most-recent + empty). `keymap.rs` — the test was updated in P3
  (cmd-r→open-history-search, cmd-shift-r→rerun-last).
- **Runs (actual):** `cargo nextest -p marley -E 'test(recent) or test(keymap)'` → 4 passed; workspace 158 passed.
- **SELF-TEST (UI — REQ-003, drove the LIVE app):** ran `echo one` + `echo two`, then cmd-R → the 🕐
  history overlay opened listing the ranked history **most-recent-first** (`echo two` highlighted above
  `echo one`) (`scratchpad/history_overlay.png`). Pressed Enter → `echo two` inserted at the prompt
  (`❯ Marley echo two`) via the cooked buffer (`scratchpad/history_enter.png`). Both the overlay + the
  Enter-insert verified — the #59 cooked-buffer fix made the insert visible.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app.rs shim
  excluded.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added` (+ the keymap change); marley_search_core.md consumer line.
- **Knowledge:** aar-submit completed (5). Reused #59 PR `PR-claude-insert-at-prompt-goes-to-cooked-buffer-not-raw-pty-001` (applied the cooked-buffer insert from the start → the Enter-insert worked first try). Reused FinderState (#57) verbatim.
- **Ticket:** forge #60 → done; archived. **2/6 of M2.B.**
