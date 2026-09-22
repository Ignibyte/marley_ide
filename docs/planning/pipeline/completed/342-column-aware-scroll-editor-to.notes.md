# 342 — column-aware scroll_editor_to — notes

- pipeline_id 1db6ab7f-a75e-4c6e-9746-69ca8d614090 · forge #342 f6df65f0-8c45-4c4f-b973-e5a764c4bddd
- aar_id 3af09837-6f88-4637-ae52-b0b56f3fe61f · on `afd4438`

## Phase 1 — Plan

**Intent:** a #336 recorded imperfection — ⌘D scrolls the vertical to the ADDED cursor's row but the horizontal
follows the PRIMARY. Add a column-aware `scroll_editor_to` so both axes track the added cursor.

**Recon (live on `afd4438`) — the confident-sentence corrections are the yield:**
1. **Gap CONFIRMED.** `active_caret()` (editor_surface.rs:233) = `active_buffer().selection().primary().head()`
   — the topmost/primary. `follow_editor_caret_x()` (app.rs:12843, `mutants::skip` shim) always derives its
   target col from `active_caret()`. `scroll_editor_to_row(row)` (app.rs:12778) scrolls the vertical slot to
   `row`, then calls `follow_editor_caret_x()`. So ⌘D's vertical → added row, horizontal → primary col. Hidden
   by `follow_caret_x`'s identity no-op (the primary is usually on screen).
2. **★ ⌘D-ONLY (ticket said "three callers").** Direct callers of `scroll_editor_to_row`: (a) ⌘D
   add-next-occurrence (app.rs:7293, inside `dispatch_action` = `mutants::skip`); (b) `goto_preview` (:3686,
   caret at origin); (c) `reveal_and_scroll_to_row` (:12800, the shared JUMP primitive); (d)
   `follow_editor_caret` (:12819, keyboard motion). go-to-def (`consume_pending_center` :11574) + find
   (`select_efind_current` :13032) go THROUGH `reveal_and_scroll_to_row` and set a SINGLE caret at the target
   first (`select_efind_current` → `set_active_selections(SelectionSet::single(...))`), so their primary IS the
   target → no mismatch. **Only ⌘D adds a non-primary cursor and scrolls to it.** Scope = ⌘D + the seam; leave
   the others.
3. **⌘D already has the added cursor.** app.rs:7282-7293: `after = add_next_occurrence(before, buffer)`, then
   `added_member(&before, &after)` (multi_cursor.rs:296, tested :1076
   `added_member_names_the_new_cursor_not_the_primary`) → the added `Selection`; takes `line_col(sel.head()).0`
   = the added ROW, DISCARDS the col. The fix threads that `head()` (a `CharOffset`) through the col-aware scroll.
4. **★ Pass the CharOffset, not a raw `col`** (D1). The follow needs the DISPLAY col (tab/wide-char via
   `col_of_offset`); a raw `col: usize` invites the `PR 71b36786` char-vs-cell domain trap. The offset lets
   `follow_editor_caret_x_to` derive the display col via the SAME shipped path (`AD-...two-boundary-maps...`).
5. **★ No new pure seam** (D4). The math is the shipped `h_scroll::follow_caret_x`/`caret_px` (cov/MSI 100) +
   `col_of_offset` (tested) + `added_member` (tested). The new fns are app.rs shims (skip'd, cov-excluded).
   Proof = a headless behavioral test + the #336 suite green. `--diff` gate:5 will see ~0 new mutants — stated,
   correct for wiring.

**Prior-art sweep:** (1) our own #336 h_scroll + `added_member` + `col_of_offset` own the seam. (2) gpui checked
— `requested_autoscroll` is element-bounds-based, doesn't fit the `uniform_list` + hand-rolled-h-margin model;
no owner. (3) ropey/regex/tree-sitter — not their seam. §20 = N/A (Marley-specific multi-cursor scroll).

**knowledge-context (Plan):** 13 nodes surfaced + logged into the AAR — incl. PR 71b36786 (width-in-chars ≠
cells, the col-domain guard), PR-hook-the-shared-primitive, AD-two-boundary-maps. All reinforce D1/D3.

**EARS:** REQ-ADDED-BOTH-AXES (⌘D off-screen-right → scroll_x follows the added col), REQ-DELEGATION-UNCHANGED
(scroll_editor_to_row byte-identical via None), REQ-IDENTITY-NOOP (target on screen → no move).

**Risks:** (a) the headless drive — can a test reach the ⌘D added-cursor path + read scroll_x? Design settles
whether a small test hook is needed (the #336 lane has `scroll_x_for_test` + `set_h_scroll_for_test`; the added
path may need seeding two occurrences + driving add-next-occurrence). (b) the delegation MUST be byte-identical
— the risk is a subtle behavior change to the 4 existing callers; the `None` split guards it.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture.** Pure wiring in `app.rs` over the shipped #336 h-scroll primitives — no crate boundary crossed,
no new type. `CharOffset` is already imported (app.rs:41). §20 = N/A confirmed (Marley multi-cursor scroll; the
mechanism is our own `follow_caret_x`). All 4 decisions confirmed against live code.

**The three edits (exact):**
1. **`follow_editor_caret_x_to(&self, target: CharOffset) -> bool`** — LIFT the current
   `follow_editor_caret_x` body (app.rs:12843-12867), substituting `target` for the two `s.active_caret()`
   reads (the `line_col(target)` row + the `in_line` = `target.as_usize().saturating_sub(line_start)`). Keeps
   the `active_editor().map(...)` guard (→ `false` when no editor / the buffer that owns `target`). Runs the
   shipped `h_scroll::follow_caret_x` over `caret_px(display_col, cell_w)` and re-clamps. `#[cfg_attr(test,
   mutants::skip)]`. PRECONDITION (doc it, plain backticks — no intra-doc `[link]`): `target` is an offset in
   the ACTIVE buffer (⌘D's added cursor always is).
2. **`follow_editor_caret_x(&self) -> bool`** becomes the delegator:
   `self.active_editor().map(|s| s.active_caret()).is_some_and(|c| self.follow_editor_caret_x_to(c))` — the
   `CharOffset` is `Copy`, the borrow drops before the re-borrow inside `_to`; no active editor → `false`
   (byte-identical to today). `mutants::skip`.
3. **`scroll_editor_to(&self, row: usize, target: Option<CharOffset>)`** — the current `scroll_editor_to_row`
   body (`slot_of(row)` + `scroll_to_item(slot, Center)`) then `match target { Some(t) =>
   self.follow_editor_caret_x_to(t), None => self.follow_editor_caret_x() };` (bool discarded, as today).
   `scroll_editor_to_row(&self, row)` becomes `self.scroll_editor_to(row, None)` — pub signature preserved, and
   `None` → the UNCHANGED `follow_editor_caret_x()`, so all four existing callers (goto_preview,
   reveal_and_scroll_to_row, follow_editor_caret, #336 lane) are byte-identical. Both `mutants::skip`.
4. **The ⌘D site** (app.rs:7288-7293, inside `dispatch_action` = already `mutants::skip`): `follow` changes from
   `Option<usize>` (row) to `Option<(usize, CharOffset)>` — `added_member(&before,&after).map(|sel| { let head
   = sel.head(); (s.active_buffer().line_col(head).0, head) })`; the call becomes `if let Some((row, head)) =
   follow { self.scroll_editor_to(row, Some(head)); }`.

**File manifest:**
| file | change |
|------|--------|
| `crates/marley_app/src/app.rs` | +`follow_editor_caret_x_to`; `follow_editor_caret_x`→delegator; +`scroll_editor_to(row, Option<CharOffset>)`; `scroll_editor_to_row`→delegate `None`; the ⌘D one-liner threads the added head. ALL inside `mutants::skip` fns → no new mutation surface. |
| `crates/marley_app/src/headless_drive.rs` | +1 `#[gpui::test]` for REQ-ADDED-BOTH-AXES (+ optionally REQ-IDENTITY-NOOP). Test code — not a cov/MSI surface. |

**Mutation/coverage:** NO new pure fn → NO new cov/MSI home (D4). The 3 production fns are `mutants::skip`; the
⌘D arm is inside `mutants::skip` `dispatch_action`. `--diff` gate:5 → ~0 mutable new lines (trivially MSI 100);
gate:4 → app.rs coverage-excluded, the new test is test code. This is a wiring ticket: the proof is behavioral.

### Regression Test Plan (headless — live drives OFF-LIMITS, the AC is a headless scroll-target assert)
| # | REQ | test (headless_drive.rs) |
|---|-----|--------------------------|
| T1 | REQ-ADDED-BOTH-AXES | `cmd_d_horizontal_follow_tracks_the_added_cursor_headless`: `boot` + `open_rs_fixture` with an identifier appearing TWICE — occ1 near top-left (small col, on screen), occ2 on a later row at a FAR-RIGHT col (~300, past a 50-col viewport). In ONE update block: `set_h_scroll_for_test(0.0, ~4000.0, 400.0, 8.0)`; `set_single_caret(occ1)`; `view.dispatch_action("add-next-occurrence")` ×2 (1st word-selects occ1, 2nd adds occ2 + scrolls — the real handler+seam, no render → geom persists); read `scroll_x_for_test()`. Assert `scroll_x > 0` AND `≈ follow_caret_x(0, caret_px(occ2_display_col, 8), 400, 8)`. ★ The OLD primary-follow would leave `scroll_x` at 0 (occ1 on screen) — the assert distinguishes added-vs-primary. |
| T2 | REQ-IDENTITY-NOOP | same fixture but occ2's col is WITHIN the viewport (on screen) → after the 2× dispatch, `scroll_x_for_test() == 0.0` (the follow's identity case no-ops). |
| T3 | REQ-DELEGATION-UNCHANGED | diff review + the existing `jump_runs_the_horizontal_follow_headless` (#336, uses `follow_caret_x_for_test` → the `None` path) stays green; goto/find/goto-line tests green. |

Uncoverable: the live GUI pixel (chad at machine) — DEFERRED + stated; the geometry is a headless `scroll_x`
assert and the seam is exercised by the real handler.

**Risks:** (a) headless reachability — SETTLED: `dispatch_action` has no `&mut Window` (app.rs:7095) so it runs
in an update block with no render, and `set_h_scroll_for_test` in the SAME block keeps `editor_geom` stable
(driving via `simulate_keystrokes` would render + reset geom to 0 — avoided). (b) delegation byte-identical —
`None` → the unchanged `follow_editor_caret_x()`; the #336 suite is the guard. (c) `CharOffset` — already
imported (app.rs:41), no new import. (d) intra-doc-link trap — the new doc uses plain backticks, no `[link]` to
a private item.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest — app.rs ONLY, 4 edits, exactly as designed:
1. **`follow_editor_caret_x_to(&self, target: CharOffset) -> bool`** (NEW) — the old `follow_editor_caret_x`
   body with `target` substituted for both `s.active_caret()` reads. `mutants::skip`; doc uses plain backticks
   (no intra-doc `[link]` — the gate:14 trap). Keeps the `active_editor().map(...) else return false` guard.
2. **`follow_editor_caret_x(&self) -> bool`** → delegator:
   `self.active_editor().map(|s| s.active_caret()).is_some_and(|c| self.follow_editor_caret_x_to(c))`. Used
   `is_some_and` (the app.rs idiom — 38 uses, 0 `map_or(false)`). Byte-identical: same caret, same math, `false`
   when no editor.
3. **`scroll_editor_to(&self, row, target: Option<CharOffset>)`** (NEW) — the old `scroll_editor_to_row` body
   (`slot_of` + `scroll_to_item`) then `match target { Some(t) => follow_editor_caret_x_to(t), None =>
   follow_editor_caret_x() };`. `scroll_editor_to_row(row)` → `self.scroll_editor_to(row, None);` (pub sig
   preserved). Both `mutants::skip` (scroll_editor_to_row ALREADY was — :12777 — so its delegating body
   inherits it → no mutation surface). **Doc migrated:** the substantive #273/#336 doc moved to
   `scroll_editor_to`, and the now-FIXED "Known imperfection … a column-aware scroll_editor_to(row,col) is the
   follow-up" note was rewritten (it shipped — avoids a false-doc); `scroll_editor_to_row` got a brief
   delegate doc.
4. **The ⌘D site** (`dispatch_action` "add-next-occurrence", already `mutants::skip`): `follow`
   `Option<usize>`→`Option<(usize, CharOffset)>` — `added_member(...).map(|sel| { let head = sel.head();
   (line_col(head).0, head) })`; call → `if let Some((row, head)) = follow { self.scroll_editor_to(row,
   Some(head)); }`. Kept the "Follow the cursor we just ADDED" comment (still accurate).

**Deviation:** none. All exactly as designed. `CharOffset` needed no new import (app.rs:41). The `match … ;`
bool is intentionally discarded (as the originals did — no `must_use`).

**Verification:** `cargo check -p marley` clean; `cargo clippy -p marley --all-targets -- -D warnings` rc=0 (the
`block v0.1.6` line is a pre-existing dep future-incompat, not this change). `cargo fmt --all` applied. Diff =
app.rs ONLY. No test expansion (Phase 4). No Zed/Warp.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Scaled to **1 critic** (general-purpose, behavior-preservation + domain-correctness lens) + own review — a small
wiring change (4 shim edits over shipped tested primitives; the risk is a subtle delegation behavior change or a
char-vs-display-col slip, not a new subsystem).

### Critic findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| C1 | LOW | **⌘⌥↓ `add-cursor-below` has the SAME class of gap** — app.rs:7261 calls `follow_editor_caret()` (the PRIMARY); `add_cursor_vertical(VDir::Down)` (multi_cursor.rs:67-77) seeds from `set.last()` so the new caret sorts to the BOTTOM while the primary stays member 0 at the top → the view follows the stationary primary, not the new bottom caret. (⌘⌥↑ sorts the new caret to the top → member 0 → correct by luck.) | **REAL, but PRE-EXISTING + OUT-OF-SCOPE — filed follow-up #360 (c243e4d4).** ★ Own-verified against app.rs:7247-7261 + multi_cursor.rs:67-77. My diff does NOT touch this path (before AND after #342: `follow_editor_caret → scroll_editor_to_row → scroll_editor_to(None) → follow_editor_caret_x` = primary), so it is not a regression. It corrects the spec's **D3-⌘D-ONLY over-claim** — the mismatch is NOT unique to ⌘D. #342 stays scoped to ⌘D (its AC + tests); the general `scroll_editor_to(row, Some(target))` seam #342 built is EXACTLY what #360 needs (its fix = `added_member` at the add-cursor arm → `scroll_editor_to(sel_row, Some(sel.head()))`). |

Critic confirmed **OK** on all six lenses (own-spot-checked the crux (a) + (c)):
- **(a) delegation byte-identical (the crux):** `scroll_editor_to(row, None)`'s body = the OLD `scroll_editor_to_row` body verbatim (`slot_of` + `scroll_to_item(Center)`) + the `None` arm calls the UNCHANGED `follow_editor_caret()`; the new `follow_editor_caret_x()` delegator uses the same PRIMARY caret, `_to` re-derives the same buffer/row/col + same `follow_caret_x`/`caret_px`/`h_scroll_clamp` tail, no-editor→`false` both. The double `active_editor()` read is idempotent (both `&self`, synchronous, no interior mutability feeds the resolution, the only `.set()` is scroll_x AFTER both reads).
- **(b) domain:** both `line_col(target)` (row) AND the `in_line` saturating_sub use the SAME `target`; `col_of_offset` is a genuine UAX#11 display col (tab/CJK/zero-width aware), not a raw char count.
- **(c) ⌘D wiring:** `head` is the ADDED member's head (`added_member` returns a member of `after` absent from `before`; the old primary was in `before`), `row` derives from the same `head` (consistent axes), destructure correct. First ⌘D (word-select) → the selected word becomes the single primary → old+new target the identical caret (harmless no-op); divergence only on the 2nd+ downward ⌘D — exactly the fix's case.
- **(d) ⌘D-only for the `Some` site:** the ONLY `Some(...)` caller is ⌘D; all `None` callers correct (goto_preview origin; the 4 `reveal_and_scroll_to_row` jumps all set a single caret/selection at the target FIRST — verified 3646→3652, 10316→10318, 11534→11580, 13054→13061; follow_editor_caret IS primary). The only miss is C1 (⌘⌥↓, a `follow_editor_caret` caller, not a `Some` site → #360).
- **(e) provenance:** the false "Known imperfection … follow-up" doc is GONE (rewritten to the shipped fix); the change is MORE conservative — it converted the old `[`follow_editor_caret`]`/`[`crate::h_scroll::follow_caret_x`]` intra-doc links to PLAIN backticks (gate:14-safe); no Zed/Warp; no unsafe.
- **(f) borrow:** `is_some_and` — `.map(|s| s.active_caret())` yields an owned `Option<CharOffset>` (Copy), dropping the `&self` borrow before `_to` re-borrows; compiled clean.

**Result: 0 findings requiring a fix to #342** (the change is correct + behavior-preserving on every point). C1
is a pre-existing analogous gap → follow-up #360 + the D3 correction above. No forge failure-record (no bug in
this diff); the D3 over-claim is a planning-lesson (a "unique to X" recon claim needs the exhaustive
add-cursor/paste sweep the critic ran — captured in the AAR at complete). Lenses: behavior-preservation,
domain, ⌘D-wiring, caller-audit, provenance, borrow. `git status --porcelain` = app.rs + the #342 docs only.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests — 2 headless drives added to `headless_drive.rs` (the behavioral proof; NO cov/MSI surface — the 3
production fns + the ⌘D arm are all `mutants::skip`, app.rs coverage-excluded, per D4):**
| test | REQ | what it drives |
|------|-----|----------------|
| `cmd_d_horizontal_follow_tracks_the_added_cursor_headless` | ADDED-BOTH-AXES | fixture with `target` twice — occ1 top-left (on screen), occ2 far right (col 296, off a 50-col viewport). One update block (no `simulate_keystrokes` → geom survives): `set_h_scroll_for_test(0,4000,400,8)` → caret on occ1 → `dispatch_for_test("add-next-occurrence")` ×2 (the REAL handler + the new `scroll_editor_to(row, Some(head))` seam). Asserts `cursors == 2` (the add ran), and `scroll_x` == the EXACT pure-fn follow of the added cursor's HEAD column, `> 0`, revealing occ2. |
| `cmd_d_added_cursor_on_screen_does_not_scroll_headless` | IDENTITY-NOOP | both occurrences at col 8 → the added cursor is on screen → `scroll_x == 0.0` (the `follow_caret_x` identity case). |

**★ The test caught a REAL subtlety (and confirmed correct behavior):** the first run asserted against occ2's START
col (296 → expected 2000) but the actual `scroll_x` was 2048 — because ⌘D selects the word LEFT-TO-RIGHT, so the
added selection's `head()` is at its END (col 302). The follow correctly tracks the head (the caret), not the
anchor. Fixed the expectation to `occ2_head_col = start + "target".len()` (302) → exact match. This is the seam
working precisely (a stronger proof than a bare `> 0`).

**Runs (all `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley cmd_d_…` → **2/2 PASS**.
- `cargo nextest run -p marley` (full crate) → **715 passed, 0 failed, 2 skipped** — REQ-DELEGATION-UNCHANGED
  confirmed: the #336 `jump_runs_the_horizontal_follow_headless` (the `None`/primary path) + the whole #297 ⌘D /
  goto / find lane stay green.

**LIVE PIXEL — DEFERRED + STATED (not masked).** chad is at the machine; a synthetic drive hits his frontmost
window. The AC is a headless scroll-target assert and is fully met — the tests drive the REAL `add-next-occurrence`
handler + the new seam and assert the exact follow. Re-eyeball the live ⌘D-onto-an-off-screen-occurrence scroll
when the machine is free (~30s, no ticket).

**FULL `--diff` GATE → `GATE GREEN [diff]` 15/15** (gate:3 tests incl. the 2 new drives; gate:4 coverage ≥100
[app.rs excluded, the drives are test code]; gate:5 mutation MSI 100 [~0 diff mutants — the app.rs change is
inside `mutants::skip` fns, exactly D4]; gate:14 docs PASS [the new docs use plain backticks — no intra-doc link
to a private item]; gate:15 visual PASS). Receipt `8a960ffe3970dd5fc47776ed98a5881430933275`, verified
worktree-bound (== `gate_state_hash`). #334 coverage flake did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
