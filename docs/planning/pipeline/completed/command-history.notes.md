# command history — Notes

- **Forge ticket:** #29 `f6447fdf-d9cf-4543-b67e-79b6aa7230d9`
- **AAR:** `c9c222c0-06fa-499b-8bd4-f545b0da2753`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-029-command-history.md
- **Pipeline spec:** command-history.spec.md

## Phase 1 — Plan
- **Request:** forge #29 (M1.D "The Daily Driver" seq-2, auto-approved) — per-pane command history
  with ↑/↓ recall.
- **Classification / tier:** work pipeline, `feature` — one slice (a pure CommandHistory + the
  PaneState field + the app.rs routing). marley_app only.
- **Forge recall (§18.3) + discovery:**
  - `PaneState<S> { session, buffer, caret }` (workspace.rs:89) — add `history: CommandHistory`
    (per-pane, default-empty in `PaneState::new`).
  - The palette OWNS ↑/↓ while open (`handle_palette_key:272-273` — "up"→move_up, "down"→move_down);
    so the prompt-history ↑/↓ must fire only in the palette-CLOSED path (D5). on_key_down already
    branches on `palette_open` first.
  - The non-palette path dispatches `apply_key(focused pane, key_from_keystroke(...))`; ↑/↓ →
    `Key::Other` → Ignored today. History intercepts them before/instead of apply_key.
  - **Detach-on-edit (the #28 carry-over):** motions AND edits both return `KeyOutcome::Edited`
    (#28 D2 — no Moved variant), so the outcome can't distinguish them. The shim knows the
    dispatched `Key`, so detach on the EDIT keys (Char/Backspace/DeleteForward), not motions (D6).
  - `submit_line` already trims → `record` receives a trimmed non-empty command.
- **Decisions:** D1–D6 in the spec (pure state machine per-pane; dedup-last; bounded ring;
  draft-stash + clamp; palette-closed only; detach on edit keys).
- **Open questions for Design:** `CommandHistory` cursor representation (index into the ring with a
  "live line" sentinel, vs an enum `{Live, At(usize)}`); prev/next return `Option<&str>` (borrow)
  vs `Option<String>` (owned — simpler for the shim's buffer-replace); the capacity const value
  (1000?); whether `record` also detaches (yes — a submit ends navigation); whether the shim's
  buffer-replace (set text + caret to end) needs a tiny pure helper or is inline shim.
- **AAR id:** `c9c222c0-06fa-499b-8bd4-f545b0da2753`.

## Phase 2 — Design

### Architecture / approach
NEW pure `crates/marley_app/src/history.rs`:
```rust
pub struct CommandHistory { entries: Vec<String>, capacity: usize, cursor: Option<usize>, draft: String }
// cursor: None = on the live line; Some(i) = viewing entries[i] (0=oldest, len-1=newest).

impl CommandHistory {
  pub fn new() -> Self                 // = with_capacity(DEFAULT_CAPACITY=1000)
  fn with_capacity(cap: usize) -> Self // TESTABLE seam — tests use cap=2 to hit eviction cheaply
  pub fn record(&mut self, cmd: String) {
    // dedup-last: skip when cmd == entries.last(); else push; evict oldest while len > capacity;
    // reset navigation (cursor=None, draft cleared) — a submit ends any recall.
  }
  pub fn prev(&mut self, current_line: &str) -> Option<String> {
    // empty → None. cursor None → stash draft=current_line, cursor=Some(len-1), return newest.
    // cursor Some(0) → clamp (stay, return oldest — no wrap). Some(i) → cursor=i-1, return that.
  }
  pub fn next(&mut self) -> Option<String> {
    // cursor None → None (nothing newer). Some(newest) → cursor=None, return the stashed draft
    // (detach). Some(i) → cursor=i+1, return that.
  }
  pub fn detach(&mut self) { self.cursor = None; }  // an edit began; next prev re-stashes
}
```
Return `Option<String>` (owned) — the shim replaces the pane buffer with it (D-2.1; clone of a
command line is trivial). `with_capacity` is the testability seam (the M0 `*_in(dir)` analog) so
the eviction bound is mutation-killable with 3 records instead of 1001 (D-2.2).

SHIM (app.rs, existing exclude) in the palette-CLOSED key path:
- `keystroke.key == "up"` → `if let Some(s)=focused { if let Some(r)=s.history.prev(&s.buffer.text())
  { s.buffer = Buffer::from_text(&r); s.caret = CharOffset::from(r.chars().count()); notify } }` return.
- `"down"` → same with `s.history.next()` (None → no-op; the live line is already shown).
- else: `apply_key(...)`; if the dispatched `Key` was an EDIT key (Char/Backspace/DeleteForward)
  and it `Edited`, call `s.history.detach()` (motions don't detach — D6).
- `on_submit(line)` → `s.history.record(line.clone())` before `write_command`.

PaneState (workspace.rs) gains `history: CommandHistory` (default `new()` in `PaneState::new`;
per-pane, travels with the pane).

### Decisions
- D-2.1 `Option<String>` (owned) returns — simplest shim buffer-replace.
- D-2.2 `with_capacity(cap)` private seam for cheap eviction testing; `new()` = 1000.
- D-2.3 cursor = `Option<usize>` index (None = live line) + a `draft: String` stash. Simpler +
  fewer mutants than an enum with an embedded draft.
- D-2.4 `record` resets navigation + clears the draft (a submit ends recall).
- D-2.5 Clamp at oldest returns the oldest again (no wrap, no None) — matches the ticket; `next`
  at the live line returns None (nothing to do).

### File manifest
- A `crates/marley_app/src/history.rs` — `CommandHistory` + `with_capacity` + tests.
- M `crates/marley_app/src/lib.rs` — `mod history;` + `pub use history::CommandHistory;`.
- M `crates/marley_app/src/workspace.rs` — `PaneState` gains `history: CommandHistory`; init in
  `PaneState::new`.
- M `crates/marley_app/src/app.rs` — up/down routing (palette closed) + buffer-replace, detach on
  edit keys, `on_submit` record.
- M `docs/specs/SPEC-app-shell.spec.md` — R36 (history) + AC + Test-Plan + Mutation-Targets.
  CHANGELOG; arch doc at complete.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `record_appends_dedups_last_and_evicts` — record "ls" then "ls" → one entry (dedup-last kills the guard-drop); `with_capacity(2)` + record a,b,c → entries [b,c], oldest evicted (kills the `> capacity` bound + the remove(0)); record after navigating resets cursor to live | unit |
| REQ-002 | `prev_stashes_draft_walks_older_and_clamps` — `prev("draft")` from live → stashes + returns newest; repeated prev walks toward oldest; prev at oldest returns the oldest again (clamp, no wrap/underflow); prev on empty history → None | unit |
| REQ-003 | `next_walks_newer_restores_draft_and_none_at_live` — after prev×2, next walks newer; next past the newest returns the stashed "draft" + detaches (cursor None); next at the live line → None | unit |
| REQ-004 | `detach_lets_next_prev_restash_current_line` — prev (stash "d1") → detach → prev("d2") re-stashes "d2" so a later next-past-top returns "d2", not "d1" | unit |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs up/down routing + buffer-replace + the on_submit/detach call sites
(existing shim exclude).

### Risks
- The detach-on-edit relies on the shim knowing the dispatched `Key` (Char/Backspace/DeleteForward)
  — motions must NOT detach (else moving the caret in a recalled line drops it). The pure
  `detach()` is tested; the shim's call-site decision is reviewed (app.rs exclude).
- Draft-stash only on the FIRST prev (cursor transition None→Some) — a mutant re-stashing every
  prev would clobber the draft mid-walk; the walk-then-restore test catches it.
- `record`'s dedup uses `entries.last()` — correct for consecutive dups; non-adjacent dups are
  intentionally kept (D2, shell-like).

## Phase 3 — Implement
- **Built (per manifest):** history.rs — `CommandHistory { entries, capacity, cursor:
  Option<usize>, draft }` + `new`/`with_capacity(seam)`/`record` (dedup-last + `while len >
  capacity remove(0)` + reset)/`prev` (None→stash+newest, Some(0)→clamp, Some(i)→i-1)/`next`
  (None→None, Some(i) i+1<len→i+1, else→cursor None + `mem::take(draft)`)/`detach`; lib.rs `mod
  history` + export; workspace.rs `PaneState` gains `history: CommandHistory` (init in `new`);
  app.rs — up/down routing in the palette-closed path (prev/next → `Buffer::from_text(recalled)` +
  caret to char-count end), detach on the Char/Backspace/DeleteForward edit keys after an Edited,
  `on_submit` records the line. SPEC-app-shell R36 + AC row 36 + Test-Plan + Mutation-Targets;
  CHANGELOG.
- **Deviations from design:** (D-3.0) `prev`/`next` renamed to `recall_prev`/`recall_next` — clippy
  `should_implement_trait` flags a bare `next(&mut self) -> Option<_>` as confusable with
  `Iterator::next`; the rename is collision-free + reads as an action (fixed at source, no
  suppression). (D-3.1) `recall_next`'s past-newest arm uses `std::mem::take(&mut draft)`
  (returns the draft + empties it) instead of a clone — the draft is consumed on restore, and a
  later `prev` re-stashes anyway; avoids a clone. Behaviorally identical.
- **Verification at this phase:** `cargo check -p marley` 0 errors; `cargo fmt`; 71 lib tests
  pass. The R36 unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critic run:** 1 correctness critic — 9 adversarial state-machine probes (full walk, dedup-last,
  eviction FIFO, record-resets-nav, stash-only-first-prev, detach re-stash, empty history,
  mem::take draft, shim guard) + 2 doc-gate reproductions.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH | The `detach` doc kept `[`prev`](Self::prev)` after the `prev`→`recall_prev` rename → a broken intra-doc link that HARD-FAILS gate:14 (`RUSTDOCFLAGS=-D warnings cargo doc`; `CommandHistory` is pub-exported so rustdoc link-checks it). Deterministic gate blocker. | REAL (caught pre-gate) | `Self::prev` → `Self::recall_prev`; re-ran the exact gate:14 command → **Finished, exit 0**. |
  | F2 | LOW | CHANGELOG/spec prose still said `prev`/`next` (stale after the rename) — cosmetic (.md not rustdoc'd, no gate impact). | REAL (consistency) | CHANGELOG reworded to `recall_prev`/`recall_next`. |
  | F3 | LOW-info | `entries.remove(0)` O(n) eviction (~24 KB memmove at cap 1000, human-paced) | ACCEPTED (spec acknowledges) | none |
  | F4 | LOW-info | up/down match ignores modifiers (shift/alt-Up also recall) — harmless today (no selection) | ACCEPTED (future selection guard) | none |
  | F5 | LOW-info | `record(line.clone())` clones even on a dedup-skip — micro, human-paced | ACCEPTED | none |
- **State-machine logic: SOUND** — all 9 adversarial probes PASS (zero correctness defects across
  the clamp/stash/dedup/eviction/detach/empty matrix). Shim wiring audited clean: up/down only when
  palette closed (after the palette + keymap-chord returns); caret = `chars().count()` (multibyte);
  detach on edit keys not motions; record before write_command; per-pane (both PaneState sites →
  `new()`).
- **Post-fix verification:** the docs gate passes (exit 0); 71 lib tests still green; clippy clean.
  The lesson: a symbol RENAME must sweep intra-doc `[`x`](Self::x)` links — rustdoc `-D warnings`
  turns a stale one into a hard gate failure (`PR-claude-rename-sweeps-intradoc-links`).

## Phase 4 — Validate
- **Tests added (history.rs, 4):** `record_appends_dedups_last_and_evicts` (non-adjacent dup kept,
  `with_capacity(2)` FIFO eviction, record-resets-nav), `prev_stashes_draft_walks_older_and_clamps`
  (empty→None, first-prev stash, walk, clamp-at-oldest, draft not re-stashed via a SENTINEL),
  `next_walks_newer_restores_draft_and_none_at_live`, `detach_lets_next_prev_restash_current_line`
  (the detach re-stash returns the NEW draft d2).
- **Runs (actual):** `cargo nextest run -p marley` → the 4 new PASS (78 in marley); full workspace
  green; doctests green.
- **Visual (gate:15):** PASS (no new render surface — ↑/↓ just replace the prompt buffer).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**, 0 SLOW hangs; coverage
  100%; mutation **28 caught / 0 missed → MSI 100.0%**; receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (done at implement); `app_shell.md` gains a `history.rs`
  bullet (CommandHistory + the shim wiring). SPEC-app-shell R36 landed at implement.
- **AAR capture:** `PR-claude-rename-sweeps-intradoc-links-001` (the F1 lesson — a rename must sweep
  `[`x`](Self::x)` intra-doc links or rustdoc `-D warnings` hard-fails the docs gate, invisible to
  compile/clippy/tests); aar-submit `completed`. Win: the pure state-machine + `with_capacity`
  testability seam gave a clean 28/28 MSI first try; the inspect critic caught the doc-link gate
  blocker BEFORE the gate ran (saved a full gate cycle).
- **Ticket:** forge #29 → done; local doc → closed/; pipeline pair archived. 2 of 6 in M1.D.
