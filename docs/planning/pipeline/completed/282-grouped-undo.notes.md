# Editor grouped-undo transaction seam — Notes

- **Forge ticket:** #282 (99bd47ba-5660-419a-920d-84be6d7baefe)
- **AAR:** 04479f69-abdc-4fb9-a933-524e7230f829
- **Local ticket doc:** docs/planning/tickets/open/TICKET-282-grouped-undo.md
- **Pipeline spec:** 282-grouped-undo.spec.md

## Phase 1 — Plan
- **Request:** #282 (M17 follow-up, from #276 inspect F3 + the #272 replace-all precedent) —
  a grouped-undo transaction seam so one ⌘Z reverts a whole block indent/dedent or replace-all,
  and undo restores the `(anchor, caret)` selection pair.
- **Classification / tier:** work pipeline (feature). PURE-crate seam (`marley_editor::undo` +
  `Buffer` bracket API) cov/MSI 100 + app-arm adoption (the #276 indent arm, the ⌘Z/⌘⇧Z wiring)
  + an in-crate migration (#272 `replace_all`).
- **Forge recall (§18.3):** `bulletin-list` → none. `knowledge-search` (undo/grouping) → mostly
  opaque cross-tenant UUIDs + a monorpgmaker Studio undo-redo doc (different project — the
  command/delta-stack model is a useful precedent: inverse-per-op, not snapshots; a new op clears
  redo; selection is NOT part of the doc there, but #282 DOES restore it). AD 902ea928 (#250
  offset↔column) governs the caret geometry.
- **Discovery (grounded in current code):**
  - `crates/editor/src/undo.rs` — `EditRecord { at, removed, inserted, origin }`; `UndoHistory {
    undone: Vec<EditRecord>, redone: Vec<EditRecord> }`; `record` (redo-clear + single-char
    coalesce, 5 guards), `pop_undo`/`push_redo`/`pop_redo`/`push_undo`. PURE, `pub(crate)`.
  - `crates/editor/src/buffer.rs:233 edit()` records one `EditRecord` per call; `:253 undo()` /
    `:263 redo()` pop ONE record, invert via `apply_raw` (no re-record), return a caret `CharOffset`.
    `Buffer` also owns a `SelectionSet` (`selection()`/`set_selection`) but the code editor uses
    the APP's caret, not this.
  - `crates/editor/src/find.rs:103 replace_all(buffer, matches, repl)` — loops `buffer.edit` over
    matches `.rev()` (back-to-front) → N records → N ⌘Z. Test `replace_all_undo_unwinds_one_match_
    at_a_time` (find.rs:212) encodes the CURRENT per-match unwind — will be REPLACED by a one-step
    assertion.
  - `crates/editor/src/indent.rs` — `indent_edits`/`dedent_edits(&Buffer, first, last, tab_width)
    -> Vec<LineEdit>` where `LineEdit = (CharOffset, usize, String)`. PURE (read-only compute);
    the APP applies them one `buffer.edit` per line.
  - `crates/marley_app/src/app.rs:6108/6110` — the Tab/⇧Tab arm applies `indent_edits`/`dedent_
    edits`; `:4387 "undo"` / `:4403 "redo"` call `b.undo()`/`b.redo()` and set only `*c = pos`
    (caret), never the anchor. `active_buffer_and_caret_mut()` returns `(buffer, &mut caret)`.
    `code_view.rs:281 caret: CharOffset` + a separate selection anchor.
- **Decisions:** D1–D4 in the spec. Leaning a **stack-of-groups** restructure: `undone:
  Vec<UndoGroup>` where `UndoGroup { records: Vec<EditRecord>, sel_before, sel_after }`; a single
  edit is a 1-record group; `begin_group`/`end_group` open/close the current group; coalesce
  applies to the top group's last record when the group is a live single-record insert-run.
  Phase 2 finalizes the exact shape (stack-of-groups vs a group-id tag + boundary markers) and the
  selection-snapshot type (a `SelSnapshot { anchor, caret }`). The `pub(crate)` visibility means
  the kill-list is in-crate (buffer.rs / undo.rs tests).
- **§20:** N/A — universal editor-undo convention; clean-room over the existing stack.

## Phase 2 — Design

### Architecture / approach
**Model = stack-of-groups** (D2). Restructure `marley_editor::undo`:
- `EditRecord { at, removed, inserted, origin }` — UNCHANGED.
- NEW `pub SelSnapshot { anchor: CharOffset, caret: CharOffset }` — a gpui-free selection capture
  (collapsed selection ⟺ `anchor == caret`). Re-exported from `lib.rs`.
- NEW private `UndoGroup { records: Vec<EditRecord>, sel_before: Option<SelSnapshot>, sel_after:
  Option<SelSnapshot> }` — one undo unit (records in APPLICATION order).
- `UndoHistory { undone: Vec<UndoGroup>, redone: Vec<UndoGroup>, open: Option<UndoGroup> }`.

`UndoHistory` methods:
- `record(rec)` — if a transaction is `open`, push to it (no coalesce, no redo-clear — `begin_group`
  already cleared). Else: `redone.clear()`, then coalesce a contiguous single-char insert into the
  TOP group's sole record **only when that group is `records.len()==1 && sel_before.is_none()`** (an
  ungrouped typing run — D4: never coalesce into a bracketed group), else push a new 1-record
  `sel_before:None` group. (The 5 coalesce guards are preserved verbatim.)
- `begin_group(sel_before)` — `redone.clear()`; `open = Some(UndoGroup{ records:[], sel_before:Some, sel_after:None })`.
- `end_group(sel_after)` — take `open`; **drop it if `records.is_empty()`** (a 0-match replace / all-blank
  indent pushes nothing); else set `sel_after` and push onto `undone`.
- `pop_undo()->Option<UndoGroup>` / `push_redo(g)` / `pop_redo()->Option<UndoGroup>` / `push_undo(g)`
  (push_undo/push_redo neither clear nor coalesce — the redo-walk survives).

`Buffer` (buffer.rs):
- `edit()` UNCHANGED (still `undo.record(EditRecord{…})`).
- NEW `pub begin_undo_group(&mut self, sel_before: SelSnapshot)` / `pub end_undo_group(&mut self, sel_after: SelSnapshot)` — thin passthroughs.
- `undo()` / `redo()` return `Option<HistoryMove>` (was `Option<CharOffset>`), NEW
  `pub HistoryMove { caret: CharOffset, selection: Option<SelSnapshot> }`.
  - `undo`: pop the group; apply each record's inverse **newest-first** (`records.iter().rev()`,
    `apply_raw(at..at+inserted, removed)`); `caret = sel_before.caret` else (ungrouped 1-record)
    the classic `at + removed.chars`; `selection = sel_before`; `push_redo(group)`.
  - `redo`: pop redo group; re-apply each **oldest-first** (`apply_raw(at..at+removed, inserted)`);
    `caret = sel_after.caret` else `at + inserted.chars`; `selection = sel_after`; `push_undo(group)`.
  - Replay order = reverse-of-application (undo) / application (redo) — the standard invariant; each
    recorded `at` is valid in the state its record produced, reached by undoing/redoing in order.

**Adoption (masked shim):**
- `find.rs replace_all` brackets its own back-to-front loop: `sel = collapse(matches[0].0)` (the first/
  lowest match start — stable under back-to-front application), `begin_undo_group(sel)` … loop …
  `end_undo_group(sel)`. Empty `matches` → early `return 0` (no group). Stays a pure fn (no new param).
- `app.rs` Tab/⇧Tab arm (6104-6117): wrap the `edits.iter().rev()` apply loop in
  `begin_undo_group(SelSnapshot{anchor:(*anchor).unwrap_or(*caret), caret:*caret})` …
  `end_undo_group(SelSnapshot{anchor after rebase, caret after rebase})`.
- `app.rs` `"undo"`/`"redo"` arms (4387/4403): switch `active_buffer_and_caret_mut()` →
  `active_buffer_caret_anchor_mut()`; on a move set `*caret = mv.caret` and
  `*anchor = mv.selection.and_then(|s| (s.anchor != s.caret).then_some(s.anchor))` (a real selection
  restores the anchor; ungrouped/collapsed → `None`, matching today's caret-only feel).

### File manifest
| File | Change |
|---|---|
| `crates/editor/src/undo.rs` | Restructure to stack-of-groups; add `SelSnapshot`, `UndoGroup`, `open`, `begin_group`/`end_group`; group-return `pop_undo`/`pop_redo`; coalesce guarded to the ungrouped top group. Rewrite the in-module tests. |
| `crates/editor/src/buffer.rs` | Add `HistoryMove`; `begin_undo_group`/`end_undo_group`; `undo`/`redo` → `Option<HistoryMove>` with group replay. Update the undo/redo unit tests. |
| `crates/editor/src/lib.rs` | `pub use` `SelSnapshot` + `HistoryMove` (the app imports them). |
| `crates/editor/src/find.rs` | `replace_all` brackets its loop in one undo group; rewrite `replace_all_undo_unwinds_one_match_at_a_time` → one-step assertion. |
| `crates/marley_app/src/app.rs` | Tab/⇧Tab arm brackets the indent/dedent loop; `"undo"`/`"redo"` arms use the 3-tuple accessor + apply the restored selection. Import `SelSnapshot`. |
| `crates/marley_app/src/headless_drive.rs` | Integration test: indent 3 lines → one ⌘Z reverts all + restores selection. |

### Regression Test Plan
| Test (crate::module) | Proves | REQ |
|---|---|---|
| `undo::group_of_n_pops_as_one` | begin/edit×3/end → `pop_undo` returns one group of 3 records | REQ-001 |
| `buffer::undo_reverts_a_grouped_indent_in_one_step` | 3 grouped edits, one `undo()` reverts all; text back to original | REQ-001 |
| `find::replace_all_undo_reverts_all_matches_in_one_step` (rewrite) | replace_all(3) then one undo restores original | REQ-002 |
| `buffer::undo_restores_the_selection_pair` | group `sel_before{2,7}` → `undo().selection == Some{2,7}` (anchor, not just caret) | REQ-003 |
| `buffer::redo_reapplies_group_and_restores_sel_after` | one `redo()` re-applies all + `selection == sel_after` | REQ-004 |
| `undo::t1..t4` (rewritten) + `buffer::undo_reverts_a_coalesced_typed_run` | single-char run still coalesces to one group | REQ-005 |
| `undo::empty_group_is_dropped` | `begin_group`/no edits/`end_group` pushes nothing | kill-list |
| `undo::record_in_open_group_does_not_coalesce_or_clear_redo` | transaction append path | kill-list |
| `undo::coalesce_only_into_ungrouped_single_record_top` | a keystroke after a bracketed group starts a NEW group | REQ-005/kill |
| `buffer::multi_record_undo_applies_inverses_newest_first` | two edits at distinct offsets undo in order (offset validity) | kill-list |
| `buffer::ungrouped_undo_caret_falls_back_to_edit_site` | `sel_before:None` → classic caret | kill-list |
| `headless_drive::indent_three_lines_then_one_undo_reverts_all` | app-arm: Tab (3-line sel) then ⌘Z → original text + selection restored | REQ-001/003 |

- **trybuild:** none — `SelSnapshot`/`HistoryMove` are plain pub structs, no compile-fail contract.
- **Uncoverable/optional:** a driven PIXEL capture (⌘Z visually collapses the indent) is optional — the
  headless keys-only lane asserts the state; per the sprint harness limits a live capture is a bonus.

### Risks / decisions
- **Test churn (accepted):** `undo.rs` t1–t4 + `buffer.rs` undo/redo tests are rewritten for the group
  shape / `HistoryMove` — re-asserting the SAME coalesce+caret behavior (a mechanical restatement, not a
  floor change); full kill-list re-derived via `cargo mutants --list -f` on the new code.
- **replace_all selection:** undo lands the caret at the first (lowest) match start — offset-stable, and
  the find bar has no caret of its own. Documented; refinable later.
- **Ungrouped undo collapses the selection (`*anchor=None`):** prior code left the anchor untouched;
  after a single-edit undo the selection is already collapsed in practice, so `None` is cleaner (no stale
  anchor) and not a user-visible regression.
- **Consumers of `undo()`/`redo()`:** only the 2 app editor arms (grep-verified); the return-type change
  is contained. Implement re-greps `.undo()`/`.redo()` across crates to confirm no terminal-prompt caller.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` green; `cargo test -p marley_editor` 107/107 green.
- **undo.rs** — restructured to stack-of-groups: `SelSnapshot{anchor,caret}` (pub), `UndoGroup{records,
  sel_before,sel_after}` (pub(crate)), `UndoHistory{undone,redone,open}`. `record` routes to the open
  transaction (append, no coalesce/clear) else coalesces into the top group ONLY when
  `sel_before.is_none() && records.len()==1` (D4). `begin_group`(clears redo)/`end_group`(drops empty).
  Rewrote the test module for groups + added 5 grouping tests (group-of-N, empty-drop, in-group
  no-coalesce, keystroke-after-group-new-group, begin_group-clears-redo).
- **buffer.rs** — `HistoryMove{caret,selection}` (pub); `begin_undo_group`/`end_undo_group` passthroughs;
  `undo`/`redo` → `Option<HistoryMove>` replaying the whole group (undo newest-first, redo oldest-first),
  caret from `sel_before/after` else the classic edit-site. Updated the existing undo/redo asserts to
  `.map(|m| m.caret)`.
- **find.rs** — `replace_all` brackets its back-to-front loop in one group (`sel = collapse(matches[0].0)`;
  empty → early return). Rewrote the undo test to the one-step contract (+ redo). Import `SelSnapshot`.
- **lib.rs** — `pub use buffer::HistoryMove` + `pub use undo::SelSnapshot`.
- **ime.rs** — one `assert_eq!(undo(), Some(co(0)))` → `.map(|m| m.caret)` (return-type compile fix).
- **app.rs** — Tab/⇧Tab arm brackets the indent/dedent apply loop (sel_before pre-rebase, sel_after
  post-rebase); `"undo"`/`"redo"` arms switched to `active_buffer_caret_anchor_mut()` and apply
  `mv.caret` + `mv.selection.and_then(|s| (s.anchor!=s.caret).then_some(s.anchor))`. Import `SelSnapshot`.
- **Deviations from design:** none. Confirmed only the 2 app arms + test modules (buffer/ime/find) consume
  `undo()`/`redo()`; the terminal prompt does NOT (no Buffer undo wiring there). `headless_drive` integration
  test deferred to Phase 4 (validate) per the manifest.

## Inspect (Phase 3.5)
Two independent critics (general-purpose): **correctness** and **state-integrity/simplification/clean-room**.
Both ran `cargo test -p marley_editor` (107→green) + `cargo check -p marley`; the correctness critic built a
throwaway path-dep crate against the live `marley_editor` to prove the finding. Lenses: replay-order/offset-
validity, coalesce∩group, redo-clear, selection restore, panic-safety, empty-group, app-arm borrow,
replace_all caret choice, anchor semantics, ime fallout, clone/alloc cost, type reuse, clean-room.

| # | Finding | Sev | Verdict | Fix |
|---|---------|-----|---------|-----|
| F1 | A no-op grouped indent/dedent (⇧Tab on a flush-left line → empty `edits`) **clears the redo stack** — `begin_group` cleared redo eagerly, but the app opened a group before knowing `edits` was empty. Regression (pre-#282 the no-op made no `record`, so redo survived); also violates the project's own `ime.rs:458 "redo survives the no-op"` invariant. BOTH critics found it independently. | **MED** | **REAL** (reproduced) | **Moved the redo-clear from `begin_group` into `end_group`'s non-empty COMMIT branch** (undo.rs) — the robust module-level fix the correctness critic recommended, protecting ALL callers (present + future), not just the two guarded ones. Reverted the app-arm `if !edits.is_empty()` guard (now redundant). `replace_all`'s `matches.first() else return` guard stays (still needed for the `.first()` access). Added the regression test `an_empty_group_preserves_the_redo_stack` + renamed `a_committed_group_clears_the_redo_stack`. |
| F2 | The `mv.selection.and_then(|s| (s.anchor!=s.caret).then_some(s.anchor))` rule is byte-identical in the ⌘Z and ⌘⇧Z arms — a drift risk. | LOW | REAL (minor) | Added `HistoryMove::ranged_anchor()` (buffer.rs) next to the type; both arms now call `mv.ranged_anchor()`. The indent-arm `SelSnapshot{..}` pair reads *different* pre/post values → left as-is (a shared ctor saves nothing). |
| F3 | `begin_group`/`end_group` has no RAII guard — a panic/early-return between them would leak `open`. | LOW | REJECTED (latent, unreachable) | Both callers are straight-line begin→edits→end with no `?`/panic between; nesting is unreachable for a synchronous single-keystroke dispatch. Noted as an API fragility, not actionable this ticket. |
| — | `headless_drive.rs:793 editor_replace_flows_headless` has a now-stale COMMENT ("undoes the last replace step") though its assertion (`assert_ne!`) still passes under grouped undo. | nit | note | Will refresh the comment in Phase 4 while adding the new integration test to that file. |

**Confirmed CLEAN (both critics, concretely traced — matches my own inline trace):** replay order + byte-exact
round-trip (3-line indent, mixed-length dedent, `foo→Z` replace); coalesce never crosses a group boundary;
redo-clear/preserve across the undo→redo walk; selection restore (anchor+caret) for grouped + caret-only for
ungrouped; `group.records[0]` panic-safe (no `sel_before==None && records.is_empty()` group is constructible);
empty-group text/caret/undo correct; the 3-way `active_buffer_caret_anchor_mut` borrow is sound; `replace_all`'s
`matches[0]` is the lowest offset (ascending `find_all`) and valid in both states; the ordinary-undo `*a=None`
is a **bugfix** (kills a stale-anchor phantom selection), not a regression; ime.rs change is the only needed
compile fix; no needless clones (groups are moved, `SelSnapshot` is `Copy`); `SelSnapshot` vs `Selection` reuse
NOT cleaner (private fields / `SelectionSet` unused by the code editor); clean-room grep (Warp/Zed/iTerm/tmux)
→ none. Post-fix: `cargo check --workspace` green, `cargo test -p marley_editor` **108/108** green.

## Phase 4 — Validate
**Tests added** (to the design's plan + the inspect edges):
- `crates/editor/src/undo.rs` — the 5 grouping tests (group-of-N pops-as-one + sel snapshots, empty-group
  dropped, in-group no-coalesce, keystroke-after-group starts new, a-committed-group-clears-redo) PLUS the
  inspect-F1 regression `an_empty_group_preserves_the_redo_stack`; t1–t4 coalesce suite kept (group-aware).
- `crates/editor/src/buffer.rs` — `undo_reverts_a_grouped_edit_in_one_step`,
  `grouped_undo_restores_the_selection_pair_not_just_the_caret` (REQ-003+004),
  `multi_record_group_undo_replays_newest_first_across_shifting_offsets` (order-sensitive — kills the
  `records.iter().rev()`→forward mutant), `ungrouped_undo_has_no_selection_and_the_edit_site_caret`.
- `crates/editor/src/find.rs` — `replace_all_undo_reverts_all_matches_in_one_step` (REQ-002, rewritten).
- `crates/marley_app/src/headless_drive.rs` — `block_indent_then_one_undo_reverts_all_lines_and_restores_
  selection_headless` (REQ-001/003 end-to-end via real Tab + cmd-z keystrokes) + **fixed** the existing
  `editor_tab_indent_dedent_flows_headless`: its `cmd-z cmd-z` (v1 per-line unwind) is now ONE `cmd-z` (the
  ⇧Tab dedent is one #282 group). That stale 2-step assertion was the ROOT of the earlier full-suite HANG:
  assertion-fail → panic BEFORE `reap_sessions` → a live TerminalSession dropped on the test thread blocks
  forever (the documented gpui-headless hazard). Refreshed the `editor_replace_flows_headless` comment too.
- **`ime.rs`** one assert → `.map(|m| m.caret)` (return-type compile fix).

**Test run (ACTUAL):** `cargo nextest run --workspace` → **1023 tests run: 1023 passed, 0 failed, 5 skipped**
(2.540s). `cargo test --workspace --doc` → 0 doctests (no `///` examples), ok. All nine new #282 tests +
the fixed indent/dedent flow are green (verified by name in the run output).

**Drive-capture:** #282 is a BEHAVIORAL keystroke→undo-state change with NO new render surface (⌘Z just
changes buffer text, which the earlier #276 capture already showed renders). Per the #264 headless-first
policy ("behavioral driven checks → prefer the headless lane"), the `block_indent_then_one_undo…headless`
integration IS the proof (real Tab+cmd-z through the keymap→dispatch→arm path, asserting text + selection).
A live pixel capture is an optional bonus (attempted post-gate if the screen is interactive).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — all 15 gates PASS: coverage **100% lines**,
mutation **MSI 100.0% (33 caught / 0 missed)**, clippy -D, brand-scrub, visual/AX, miri (skip-clean). First
run was RED (coverage 99.6% + MSI 90.9%): 3 mutants survived on `HistoryMove::ranged_anchor` (the app-arm
consumer is a `mutants::skip` shim and the headless test lives in `marley_app`, a cross-crate seam whose
`marley_editor` mutants only die to an IN-CRATE test) + find.rs:109 (replace_all empty-return) and undo.rs:110
(end_group no-open arm) uncovered. Fixed at source with 3 unit tests: `ranged_anchor_maps_collapsed_to_none_
and_ranged_to_the_anchor` (buffer.rs), `replace_all_of_zero_matches_is_a_noop_returning_zero` (find.rs),
`end_group_with_no_open_transaction_is_a_noop` (undo.rs). Editor crate now **115/115**.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` entry; `editor.md` updated — the `undo()`/`redo() -> Option<HistoryMove>`
  signature, the `undo.rs` stack-of-groups description (begin/end_group, coalesce-into-ungrouped-top,
  redo-clear-on-commit), and the "grouped-undo seam is the follow-up" note retired to "landed".
- **Knowledge:** `failure-record` BF-claude-grouped-undo-noop-transaction-wipes-redo-001 + `prevention-rule-
  record` PR-claude-defer-redo-clear-to-transaction-commit-001 (both at inspect). AAR submitted.
- **Lessons:** (1) a transaction seam that clears redo on OPEN wipes redo on a no-op transaction — clear on
  COMMIT (both critics found it independently). (2) The cross-crate seam struck AGAIN — a helper used only by a
  `mutants::skip` app arm + a `marley_app` headless test leaves its `marley_editor` mutants alive; needs an
  in-crate unit test (the ranged_anchor 3-mutant survivor). (3) A stale multi-⌘Z assertion in an EXISTING
  headless test, failing under the new one-step group, HUNG the whole suite (panic before `reap_sessions` →
  live-session drop blocks forever) — when a behavior changes N-step→1-step, grep existing tests for the old
  step count. (4) Running `cargo fmt` while a background `nextest` compiles disrupts it — sequence them.
