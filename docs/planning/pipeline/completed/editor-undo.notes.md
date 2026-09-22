---
pipeline_id: b22246aa-1ad9-4cab-93b4-91c322039305
ticket: forge#253 (83b62a5a-f30e-48cf-b135-0c2430474eca)
aar_id: df83094b-ffd1-42e1-88ef-1c85c04ac691
---

# Notes — Editor undo/redo (forge#253)

## Plan (Phase 1)

**Classification:** work pipeline, feature, MEDIUM-LARGE (a real undo subsystem in marley_editor + the wiring).
AUTONOMOUS (chad's `/goal /work 249 to 258`). chad put undo in v1 ("might as well do it now").

**Discovery (anchors):**
- `Buffer` (buffer.rs:17) = `{ rope, selection, version }`; `edit(range, replacement, origin) -> EditResult`
  (buffer.rs:104) does remove+insert+version.next(); NO undo. Add an `undo: UndoHistory` field.
- The undo model: `EditRecord { at: CharOffset, removed: String, inserted: String }`; undo = put `removed`
  back over `at..at+inserted`; redo = put `inserted` back over `at..at+removed`. `edit()` records the `removed`
  text (captured BEFORE `rope.remove`) + clears redo; coalesces contiguous single-char Human inserts.
- **THE re-record trap (D2):** `undo()/redo()` apply an inverse edit that must NOT push a record → a private
  `apply_raw(range, text)` (rope mutate + version bump, no history) that undo/redo use; `edit()` = `apply_raw`
  + record.
- **Caret (D3):** `undo()/redo() -> Option<CharOffset>` (the edit site); the app (app.rs) syncs `active_caret`
  (#250 render source) + notifies. (Buffer's selection is secondary.)
- **Wiring (app.rs shim):** ⌘Z/⌘⇧Z bindings in keymap `default_bindings` (roster count 39→41 + tests) +
  `dispatch_action` "undo"/"redo" arms → route to the active editor buffer + caret sync; guarded to an editor
  tab. ⚠️ KEEP `dispatch_action`'s `mutants::skip` — the #252 DETACH TRAP (adding a fn near it stranded the skip)
  → RUN `cargo mutants --list -f app.rs | grep -c dispatch_action` MUST be 0 after.

**Decisions:** D1 pure UndoHistory in marley_editor · D2 non-recording apply_raw for the inverse · D3
undo/redo return the caret site, app syncs active_caret · D4 coalesce = contiguity + origin (no timer) · D5
undo doesn't un-dirty (version only increments; noted).

**Risks / load-bearing:**
- **The inverse must round-trip EXACTLY** — undo then redo returns the buffer to the post-edit state
  byte-for-byte; the char-offset math (`at..at+inserted.chars`) must be right (multibyte: use CHAR counts, not
  bytes — inserted.chars().count()).
- **No re-record on inverse** (D2) — else the stack never drains / grows unbounded.
- **Coalescing correctness** — a run of typed chars is ONE step; a backspace mid-run breaks it; the predicate
  is the subtle part (contiguity + insert-run + same origin).
- **redo-clear** — a new edit after an undo clears redone (standard); else redo re-applies a stale branch.
- **The #252 detach-trap** — re-run mutants --list to confirm dispatch_action stays skipped.

**Test plan (finalized at design):** pure units on UndoHistory/Buffer: record-then-undo round-trips (text +
caret); coalesce a 3-char run → ONE undo removes all 3; backspace breaks the run (2 steps); redo re-applies;
a new edit clears redo; undo-empty → None; the inverse doesn't re-record (undo N times drains, not loops);
multibyte (é/😀) offsets. cov/MSI 100. + the ⌘Z/⌘⇧Z binding (action_for). DRIVEN: type → ⌘Z → gone → ⌘⇧Z → back.

**Reference (§20):** Warp's input undo/redo (chords + typing-run coalescing) — behavior reference; the history
model is Marley's own pure impl. Filled in the spec.

**AAR:** df83094b-ffd1-42e1-88ef-1c85c04ac691 (opened).

**Phase 1 status: Plan PASS — autonomous (M15 /goal). Ready for Phase 2 — Design.**

## Design (Phase 2)

**## Reference (§20) confirmed:** Warp's input undo/redo (chords + typing-run coalescing). This design matches
the BEHAVIOR (⌘Z undoes the last coalesced edit, ⌘⇧Z redoes, a new edit clears redo) with Marley's own pure
`UndoHistory`. Clean-room: observed behavior, ropey + in-repo, no Warp/Zed source.

**Architecture.** A pure `undo.rs` in marley_editor + `Buffer` gains a history field + `apply_raw`/`undo`/`redo`
+ `edit()` records; the app.rs shim adds ⌘Z/⌘⇧Z dispatch arms (KEEP dispatch_action's skip — #252 lesson).

### THE UndoHistory + EditRecord — RESOLVED (crates/editor/src/undo.rs, PURE)
```
pub(crate) struct EditRecord { at: CharOffset, removed: String, inserted: String, origin: EditOrigin }
#[derive(Default)] pub(crate) struct UndoHistory { undone: Vec<EditRecord>, redone: Vec<EditRecord> }
impl UndoHistory {
    pub(crate) fn record(&mut self, rec: EditRecord) {      // the "new edit" path — clears redo + coalesces
        self.redone.clear();
        if let Some(last) = self.undone.last_mut() {
            if last.removed.is_empty() && rec.removed.is_empty()             // both insert-runs
                && rec.inserted.chars().count() == 1                         // the new one a single char
                && last.origin == rec.origin                                 // same origin
                && last.at.as_usize() + last.inserted.chars().count() == rec.at.as_usize() {  // contiguous
                last.inserted.push_str(&rec.inserted);
                return;
            }
        }
        self.undone.push(rec);
    }
    fn pop_undo(&mut self) -> Option<EditRecord>;  fn push_redo(&mut self, EditRecord);
    fn pop_redo(&mut self) -> Option<EditRecord>;  fn push_undo(&mut self, EditRecord);
}
```
UndoHistory is a DUMB stack (no rope); `Buffer` drives the apply. `#[cfg(test)]` unit-tests the coalesce/
record/pop directly (pub(crate) — same-crate tests). cov/MSI 100.

### Buffer::edit records + undo/redo — RESOLVED (buffer.rs, D2 no-re-record)
- ADD field `undo: UndoHistory` (default in `new`/`from_text`). Re-export nothing new (UndoHistory is
  pub(crate); `Buffer::undo/redo` are the public API).
- **`apply_raw(&mut self, range, replacement) -> EditResult`** = the CURRENT edit() body (buffer.rs:138-156: byte
  span, `rope.remove`, `rope.insert`, `version.next`, delta, EditResult) — NO recording.
- **`edit()`** = `let at = range.start; let removed = self.text_in_range(range.start..range.end); let result =
  self.apply_raw(range, replacement); self.undo.record(EditRecord{at, removed, inserted: replacement.to_string(),
  origin}); result`. So edit() RECORDS (+ clears redo via record); apply_raw does NOT.
- **`undo() -> Option<CharOffset>`**: `let rec = self.undo.pop_undo()?; let end = rec.at + rec.inserted.chars()
  .count(); self.apply_raw(rec.at..end, &rec.removed); let caret = rec.at + rec.removed.chars().count();
  self.undo.push_redo(rec); Some(caret)` — apply_raw (NO re-record). **redo()** symmetric (pop_redo → apply_raw
  the inserted over the removed span → push_undo → caret = rec.at + rec.inserted.chars). Borrow: pop (self.undo)
  → apply_raw (self) → push (self.undo), sequential + rec owned (borrows of rec end before push_redo moves it).
- Multibyte: ALL offset math uses `.chars().count()` (CHAR, not bytes) — CharOffset is char-based.
- REDO-CLEAR is in `record()` (the edit path); undo/redo use push_undo/push_redo (no clear, no coalesce) so a
  redo restores the exact record.

### CARET SYNC + ⌘Z/⌘⇧Z WIRING — RESOLVED (keymap.rs + app.rs shim, D3/D4)
- keymap `default_bindings`: ADD `(chord(true,false,false,false,"z"),"undo")` + `(chord(true,false,false,true,
  "z"),"redo")`. Roster guard `all_chords_lists_every_binding` 39→41 + `contains(⌘Z)`/`contains(⌘⇧Z)`;
  tests-phase adds `cmd_z_maps_to_undo`/`cmd_shift_z_maps_to_redo`. (⌘Z/⌘⇧Z free — grep the vec confirms.)
- app.rs `dispatch_action`: ADD arms `"undo" => { if let Some(s) = ...active_tab_mut().editor_mut() { let (b,c)
  = s.active_buffer_and_caret_mut(); if let Some(pos) = b.undo() { *c = pos; } } }` + `"redo"` (b.redo()). The
  `editor_mut()` None guard = a terminal tab is a no-op (REQ-005). The caller (dispatch site ~4267) `cx.notify()`s
  → the #250 render redraws. ⚠️ These are ARMS inside `dispatch_action` (NOT a new fn above it) → dispatch_action's
  `mutants::skip` is UNTOUCHED — but RUN `cargo mutants --list -f app.rs | grep -c dispatch_action == 0` at
  implement to CONFIRM (the #252 detach-trap).

### File Manifest
| File | Change |
|---|---|
| crates/editor/src/undo.rs | NEW — `EditRecord` + `UndoHistory` (record+coalesce / pop_undo / push_redo / pop_redo / push_undo). Pure. Tests-phase. |
| crates/editor/src/lib.rs | `mod undo;` (no re-export — pub(crate)). |
| crates/editor/src/buffer.rs | `undo: UndoHistory` field; `apply_raw` (extracted edit body); `edit()` records; `undo()`/`redo()` → `Option<CharOffset>`. Tests-phase. |
| crates/marley_app/src/keymap.rs | ADD `(⌘Z,"undo")` + `(⌘⇧Z,"redo")`; roster 39→41. Tests-phase. |
| crates/marley_app/src/app.rs | ADD `"undo"`/`"redo"` dispatch arms + caret sync. SHIM (KEEP dispatch_action's skip). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | undo.rs `record` coalesce: 3 contiguous single-char Human inserts → ONE record with inserted "abc"; a 4th at a non-contiguous `at` → a 2nd record; different origin → no coalesce; a record with removed non-empty → no coalesce | REQ-003 |
| T2 | undo.rs `record` clears redone; pop_undo/pop_redo LIFO | REQ-004 |
| T3 | buffer.rs: `edit`×3 ("a","b","c" contiguous) → `undo()` → text "" + caret 0 (the coalesced run) + returns Some(0); `redo()` → "abc" + caret 3 | REQ-001/002/003 |
| T4 | buffer.rs: type "ab", backspace, type "c" → 3 undo steps (the backspace breaks the run) — undo/undo/undo walks back correctly | REQ-003 |
| T5 | buffer.rs: undo then a NEW edit → `redo()` returns None (redo cleared) | REQ-004 |
| T6 | buffer.rs: undo on an empty history → None; undo drains (undo the only edit → then undo → None, NO loop/growth — the inverse doesn't re-record) | REQ-001/002 |
| T7 | buffer.rs multibyte: `edit(0..0,"é😀",Human)` → undo → "" (removed the 2 chars, offset math by chars) | REQ-002 |
| T8 | keymap: `action_for(⌘Z)==Some("undo")`, `action_for(⌘⇧Z)==Some("redo")`; roster 41 | REQ-005 (binding) |
| T9 (driven) | type a word into the restored editor tab → ⌘Z removes it → ⌘⇧Z re-adds it (the caret tracks) | REQ-005 |

**Uncoverable / shim:** the app.rs dispatch arms + caret sync (gpui) are shim (driven T9 + review). The
UndoHistory + Buffer undo/redo/apply_raw are the pure seam (cov/MSI 100).

**Risks:** (1) the inverse round-trip exactness — T3/T7 (multibyte char math). (2) no-re-record (apply_raw) — T6
(drains, no loop). (3) coalesce predicate — T1/T4. (4) redo-clear — T5. (5) the #252 detach-trap — mutants
--list on app.rs (the arms don't touch the skip, but verify).

**Phase 2 status: Design PASS — the pure UndoHistory (record+coalesce) + Buffer apply_raw/edit-records/undo/redo
(no-re-record, char-offset math) + the ⌘Z/⌘⇧Z bindings + the dispatch arms + T1-T9 locked. Ready for Phase 3 —
Implement.**

---

## Phase 3 — Implement (PASS)

Built to the manifest, no deviations from the Phase 2 design:

- **crates/editor/src/undo.rs** (NEW, pure): `EditRecord { at, removed, inserted, origin }` +
  `UndoHistory { undone, redone }` with `record` (redo-clear + coalesce), `pop_undo`, `push_redo`, `pop_redo`,
  `push_undo`. All `pub(crate)`, all doc'd (crate is `#![deny(missing_docs)]`). The coalesce predicate is the
  5-way `&&`: `last.removed.is_empty() && rec.removed.is_empty() && rec.inserted.chars().count()==1 &&
  last.origin==rec.origin && last.at + last.inserted.chars == rec.at`.
- **crates/editor/src/lib.rs**: `mod undo;` (after `mod types;`; no `pub use` — pub(crate)).
- **crates/editor/src/buffer.rs**: `use crate::undo::{EditRecord, UndoHistory}`; `undo: UndoHistory` field
  (added to the struct + `new()` + `from_text()` Self blocks). EXTRACTED `fn apply_raw(&mut self, range,
  replacement, origin) -> EditResult` = the exact prior `edit()` body (rope remove/insert, version bump, delta) —
  it does NOT record. `edit()` now: capture `at=range.start` + `removed=text_in_range(..)` BEFORE mutating →
  `apply_raw(range, replacement, origin)` → `undo.record(EditRecord{..})` → return the result. `undo()` /
  `redo()` → `Option<CharOffset>`: pop the record, `apply_raw` the inverse (NO record → drains, no loop), move
  the record to the other stack, return the caret site (undo → `at + removed.chars`, redo → `at +
  inserted.chars`).
  - DEVIATION (minor, design-improving): `apply_raw` takes `origin` (the design sketch omitted it) so the
    `EditResult.origin` stays faithful on an undo/redo re-apply — `undo()`/`redo()` pass `rec.origin`.
- **crates/marley_app/src/keymap.rs**: ADDED `(⌘Z,"undo")` + `(⌘⇧Z,"redo")` after the ⌘S "save" entry. Roster
  guard `all_chords_lists_every_binding`: `39→41` (32 vec-literal + 9 switch-tab) + `contains(⌘Z)`/`contains(⌘⇧Z)`.
- **crates/marley_app/src/app.rs**: ADDED `"undo"`/`"redo"` ARMS to `dispatch_action` (after the "save" arm) —
  each guards `active_project_mut().active_tab_mut().editor_mut()` (a terminal tab → None → no-op), then
  `let (b, c) = s.active_buffer_and_caret_mut(); if let Some(pos) = b.undo() { *c = pos; }`. The arms are INSIDE
  the fn body — dispatch_action's `#[cfg_attr(test, mutants::skip)]` was NOT touched.

**Checks:** `cargo fmt` clean; `cargo check -p marley_editor` clean; `cargo check -p marley --all-targets`
clean (only the pre-existing `block v0.1.6` dep warning). **`cargo mutants --list -f app.rs | grep -c
dispatch_action` == 0** — the #252 detach trap did NOT recur (the arms live inside the body, the skip still
binds `fn dispatch_action`). Pure-seam mutants surfaced for Phase 4 to kill: undo.rs (16 — the coalesce
`&&`/`==`/`+` set; the `Some(Default::default())` pair is unviable, no Default on EditRecord), buffer.rs
undo/redo (the `+`→`-`/`*` char-math set + None; `apply_raw`→Default::default() unviable, no Default on
EditResult). Everything T1-T9 targets.

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

---

## Inspect (Phase 3.5) — PASS (0 code fixes)

3 independent critics over the 5-file diff (round-trip/no-re-record · coalesce/redo-clear/edges ·
wiring/regression/clean-room). **All load-bearing invariants CONFIRMED by all three; zero HIGH/MED code
defects.** Every finding was a Phase-4 test-coverage item (expected at inspect, the validator writes them).

### Critic 1 — round-trip exactness + no-re-record (all CONFIRMED)
- (a) NO-RE-RECORD/DRAIN: `undo`/`redo` apply the inverse via `apply_raw` (rope edit + version bump, NO
  `undo.record`) — the only `record` caller is `edit()`. 1 edit → undo → undone empty (rec moved to redone) →
  2nd undo → `None`. Stack DRAINS, no loop, no growth. **The D2 trap is closed.**
- (b) CHAR-MATH by CHARS: every offset is `.chars().count()`, never `.len()`; the only `.len()` is the correct
  `new_byte_len` field. `edit(0..0,"é😀")` → undo → `apply_raw(0..2,"")` (2 chars, not 6 bytes) → "". Verified.
- (c) `removed` captured BEFORE `apply_raw` mutates; `range` (non-Copy) handled via Copy'd CharOffset endpoints,
  no use-after-move. (d) push_undo is a plain push (no clear/coalesce) → undo;redo;undo;redo walks.
  (e) caret lands at the end of the affected span both directions (no off-by-one).

### Critic 2 — coalesce + redo-clear + edges (all CONFIRMED)
- (e) The 5-guard coalesce is all `&&`, zero `||`; each guard's counter-case verified load-bearing (backspace,
  type-over-selection, 2-char paste, Human→Agent, caret-jump each START A NEW record).
- (f) `redone.clear()` is undo.rs:31 — FIRST, before the coalesce branch → redo is cleared on EVERY edit path
  incl. a coalesced keystroke (proved with a reachable coalesce-while-redone-non-empty sequence).
- (g) empty/boundary: `pop_*` = `Vec::pop`→None; `undo()/redo()` short-circuit via `?`; `last_mut()` on empty →
  push. No panic, no unwrap. (h) staleness: the contiguity `at`-check + the LIFO/inverse invariant make a
  cross-gap stale coalesce impossible. (i) guard-5 arithmetic uses the full run length (`last.at +
  last.inserted.chars`), not `+1` — no off-by-one, char-space not byte.

### Critic 3 — wiring + regression + clean-room (all CONFIRMED)
- (i) ⌘Z/⌘⇧Z are the only "z" bindings (free, no collision; the `chords_unique` debug_assert + test guard dups).
  (i2) roster = 32 vec-literal + 9 switch-tab = **41** (test `all_chords_lists_every_binding` GREEN — re-run at
  inspect). (j) the arms are INSIDE `dispatch_action`'s body; the doc + `#[cfg_attr(test,mutants::skip)]` sit
  directly above `fn dispatch_action`; **`cargo mutants --list -f app.rs | grep -c dispatch_action` == 0** — the
  #252 detach trap did NOT recur (re-verified at inspect). (k) Buffer derives nothing → the `undo` field is
  additive; `ac1_default_equals_an_empty_buffer` ignores it; `edit()` signature + the EditResult delta are
  byte-identical (apply_raw builds the same). (l) EditOrigin derives PartialEq+Eq+Copy (both load-bearing:
  `==` in coalesce, by-value `rec.origin` into apply_raw before the `rec` move); no unwrap; clean-room OK (a
  plain two-Vec push/pop/coalesce stack — Marley's own, nothing transliterated).

### Findings (all deferred to Phase 4 — Validate; no code change at inspect)
1. **[MED, critic 3] The keymap test pins the chord via `contains`, not the ACTION string.** The #252 sibling
   `cmd_s_maps_to_save` asserts both `action_for(⌘S)==Some("save")` AND `bare-s==None`; #253 must mirror it.
   → **VALIDATE T8 REFINEMENT:** add `cmd_z_maps_to_undo` asserting `action_for(⌘Z)==Some("undo")`,
   `action_for(⌘⇧Z)==Some("redo")`, and `action_for(bare "z")==None`. (Not just the roster `contains`.)
2. **[MED/LOW, all 3 critics] The pure undo/redo/coalesce seam has ZERO tests yet.** Expected at inspect (tests
   are Phase 4). The editor crate is NOT coverage-excluded → cov/MSI 100 REQUIRES T1-T9. The exact mutants to
   kill (from `cargo mutants --list`): undo.rs coalesce `&&`/`==`/`+` set; buffer.rs undo/redo `+`→`-`/`*` +
   None; the `Some(Default::default())`/`apply_raw→Default` variants are unviable (no Default derive). → the
   T1-T9 plan already targets these; Validate must land cov/MSI 100.
3. **[LOW, informational — OUT OF #253 SCOPE, no action] Version-based dirty never re-cleans after an
   undo-to-original.** `apply_raw` bumps `version` on undo/redo too, so undoing a typed run back to the saved
   text leaves `version != saved_version` → the tab stays dirty though content matches disk. This is the
   pre-existing #252 version-based dirty model (acknowledged in #251/#252/#253 planning — "an undo is a NEW edit
   bumping version"), NOT a #253 regression. A saved-checkpoint model is a separate future ticket if desired.

**No `failure-record`** — no real bug was found (the implementation is correct). The test-gap is expected-at-
inspect, not a defect. **Phase 3.5 status: Inspect PASS — ready for Phase 4 — Validate.**

---

## Phase 4 — Validate (PASS — GATE GREEN [diff])

### Tests added (15 pure-seam units, cov/MSI 100 on the seam)
**crates/editor/src/undo.rs** — NEW `#[cfg(test)] mod tests` (the file had none):
- `t1_contiguous_single_char_inserts_coalesce_into_one_record` — 3 contiguous 1-char Human inserts at a
  NON-ZERO base (5,6,7) → ONE record "xyz" (base-5 so a `+`→`*` mutant `5*1=5≠6` can't masquerade as `+`).
- `t2_guard1..guard5` (5 tests) — each isolates ONE failing guard (prior-delete run / replace / 2-char insert /
  different origin / non-contiguous) → a SEPARATE record (2 pops). Kills every `&&`→`||` + the `==`→`!=` on
  guards 3/4/5.
- `t3_record_clears_the_redo_stack` + `t3_redo_is_cleared_even_on_the_coalesce_path` — the unconditional
  line-31 `redone.clear()` (incl. the coalesce path). `t4_pop_is_lifo_and_push_undo_preserves_redo` — LIFO +
  push_undo neither clears nor coalesces the redo stack.

**crates/editor/src/buffer.rs** — 6 tests near ac3/ac5:
- `undo_reverts_a_coalesced_typed_run_in_one_step` — type "abc" char-by-char → one undo → "" (caret 0) → redo →
  "abc" (caret 3). `undo_redo_round_trips_a_replace_with_caret_at_span_end` — "abc"→"XY": undo caret 3 (removed
  chars), redo caret 2 (inserted chars) — pins the removed↔inserted swap. `undo_of_a_multibyte_insert_spans_
  chars_not_bytes` — "é😀" (2 chars/6 bytes) → undo spans 0..2 CHARS (a byte 0..6 would panic). `undo_drains_
  and_does_not_re_record` — undo→Some, undo→None (drains, no loop); fresh buffer undo→None (no panic).
  `a_new_edit_after_undo_clears_redo`. `a_delete_breaks_the_coalesced_insert_run` — type "abc" (coalesces),
  delete "c" → the delete is its own undo step (2 undos to drain).

**crates/marley_app/src/keymap.rs** — `cmd_z_maps_to_undo` (mirrors `cmd_s_maps_to_save`, critic-3's
refinement): `action_for(⌘Z)==Some("undo")`, `action_for(⌘⇧Z)==Some("redo")`, `action_for(bare "z")==None`.

### Test runs (actual)
- `cargo nextest run -p marley_editor` → **43 passed** (15 new undo/redo tests + the prior 28).
- `cargo nextest run -p marley keymap::` → `cmd_z_maps_to_undo` PASS; `all_chords_lists_every_binding` (41) PASS.
- `cargo nextest run -p marley` → **357 passed, 2 skipped**.
- `cargo mutants --list -f app.rs | grep -c dispatch_action` → **0** (skip held).

### Driven live proof (REQ-005 — mac unlocked, DATA-SAFE)
Bundled + `open target/Marley.app` (WIN 19434). The #205 session restored four real-file editor tabs
(selection.rs active). Sent `focus type:UNDOTEST` → **"UNDOTEST" inserted at line 1 offset 0 AND the selection.rs
tab showed the cyan dirty ●** (the #252 integration). `focus cmd:z` → **the ENTIRE "UNDOTEST" run vanished in ONE
undo** (the 8-char coalesced run — proves coalescing live; the ● persisted, exactly the ledger's finding-3
version-model note). `focus cmdshift:z` → **"UNDOTEST" restored** by one redo. A final `cmd:z` reverted the
buffer; quit without saving. **DATA-SAFETY honored: NO ⌘S was ever sent — `git status` confirms selection.rs (the
file typed into) is PRISTINE on disk;** the only modified files are the #253 source I authored via Edit.
Captures: m253_02_typed / 03_after_undo / 04_after_redo.

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0 failed** (incl. gate:4 coverage ≥100%, gate:5
mutation MSI ≥100% [13 caught, 0 missed, 0 timeout, 2 unviable on the pure seam], gate:1 rustfmt, gate:6 miri,
gate:15 visual). Receipt `1b0ec851…` matches the current `.rs` state → the commit gate will pass.

**FLAKE NOTE (pre-existing, NOT #253):** the first two gate attempts wedged ~35 min each on the real-PTY
integration test `workspace_two_real_sessions_are_independent` sitting at 0.0% CPU under llvm-cov instrumentation
(a spawn/handshake deadlock). Memory was healthy (76% free — the `--jobs 2` fix held). Killed + re-ran on a quiet
system → it passed in ~2s and the whole gate finished in ~105s. This is a flaky real-session test under coverage,
worth a follow-up ticket (a nextest retry/timeout or a more deterministic session-handshake in the test), but it
does not touch #253 (undo/redo is pure — no PTY/session code).

**Phase 4 status: Validate PASS — GATE GREEN [diff]. Ready for Phase 5 — Complete.**
