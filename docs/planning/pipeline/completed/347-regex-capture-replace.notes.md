# Regex REPLACE — capture groups + the two #339 fixes — Notes

- **Forge ticket:** #347 `1df08cb6-9ac1-4cd1-ad0c-820386f268a2`
- **AAR:** `3b62706e-970a-47e3-82d4-e12f82d7ea2c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-347-regex-capture-replace.md
- **Pipeline spec:** 347-regex-capture-replace.spec.md
- **pipeline_id:** c7b460d4-5479-4989-b11a-d03ceeeb504b · on `d08688d`

## Phase 1 — Plan

**Request:** capture-group REPLACE (`$1`/`${name}`/`$$`) for the find bar — the second half of M22 #339 — plus fix
the two pre-existing silent-corruption bugs #339 surfaced (F2 template-vs-expanded length; F3 empty-match resume
loop). Both bugs sit in a `mutants::skip` shim inside the coverage-excluded `app.rs`.

**Classification / tier:** work pipeline, one shippable slice (capture-replace + the two fixes it makes reachable).
The #339 FIND path shipped; this is its write-path completion.

**Forge recall (§18.3):** no bulletins. `knowledge-context` (Plan) surfaced 13 nodes (3 architecture-decisions,
3 distilled-lessons, 5 prevention-rules, 2 failures) — the #338 edit-post-state ADR, the #339 regex-find
decisions (D-SAME-SHAPE / D-BYTE-CHAR-AT-ONE-SEAM / D-EMPTY-ADVANCE), the #336 cov-blind-shim lesson, and prior
find/replace failures. Logged to the AAR surfacing-log (verdicts at aar-submit).

**★ Prior-art WIN (the highest-yield leg — recorded in the spec):** `regex 1.12.4` ships
`Captures::expand(&self, replacement: &str, dst: &mut String)` at `src/regex/string.rs:1884`. Its documented
grammar is EXACTLY the ask — unbraced `$ref` (numeric → index else name, longest `[0-9A-Za-z_]`), braced
`${ref}`, invalid ref → empty, `$$` → literal `$`. So `expand_captures` is a THIN adapter, not a reimplementation
(mirrors #339's D-EMPTY-ADVANCE dissolving into `find_iter`). **The fork-feeding caveat:** `Captures<'h>` borrows
the haystack → it cannot be stored in the find-result `Vec`; "carry captures" means carry OWNED group byte-ranges,
OR re-run the regex at replace time for a live `Captures`. → D2.

**Decisions recorded (D1-ADOPT-EXPAND, D2-THE-FORK [OPEN → P2], D3-REPLACE-ALL-WITH, D4-EXPANDED-LENGTH [F2],
D5-ADVANCE-EMPTY [F3], D-LITERAL-UNCHANGED)** — see the spec. D2 is deliberately left for Design to settle with
code evidence (the planner does not make design decisions).

**Discovery (recon in progress — an Explore agent over find.rs/app.rs/buffer.rs/headless_drive.rs):** verifying
the `find_all_regex`/`FindError` shape + the `efind` flow (the D2 fork), the F2 site (Replace-One caret +
`efind_resume` vs `repl.chars().count()`), the F3 site (`partition_point` resume), the existing literal
`replace_all` + `begin_undo_group(cursor_anchored:false)`, and the headless find/replace drive pattern. Findings
appended below; the exact line numbers are re-verified at Design (they drift), so the spec cites them as
approximate.

### Recon findings (Explore agent, live on `d08688d`)

**The fork (D2) — evidence LEANS B (re-run at replace):**
- `find_all_regex(text, pattern, ci) -> Result<Vec<(CharOffset, CharOffset)>, FindError>` (find.rs:203) uses
  `re.find_iter(text).map(|m| (char_at(m.start()), char_at(m.end())))` (find.rs:253) — position-only `Match`, NOT
  `captures_iter`; captures discarded.
- The `Regex` is built PER-CALL via `RegexBuilder…build()` (find.rs:211) and dropped at fn end — NEVER stored.
- The app memoizes only `efind_matches: Rc<Vec<(usize,usize)>>` (app.rs:486) + `efind_key` (a version/query memo
  key, app.rs:487). No captures, no compiled regex retained.
- ⟹ There is NO live `Captures`/`Regex` at replace time. Since the regex is recompiled per call anyway, B (re-run
  at replace) adds a second match PASS but no stored state; A (grow the shape) ripples the `(CharOffset,CharOffset)`
  type through bands/n-of-m/nav (which consume it unchanged, find.rs:15-18 doc) + the memo key. `Captures<'h>`
  borrows the haystack → cannot be stored regardless. **P2 confirms B via a pure `find.rs` fn that owns the re-run
  + the byte↔char conversion in ONE place (contains the D-BYTE-CHAR-AT-ONE-SEAM cost).**

**The regex crate (D1):** `regex 1.12.4` `Captures::expand(&self, replacement: &str, dst: &mut String)`
(string.rs:1884) — the whole `$1`/`${name}`/`$$` grammar (invalid ref → empty, `$$` → `$`). Adopt; don't hand-roll.

**F2 (D4) — CONFIRMED, line drift noted:** Replace-One is the `"enter" if efind_focus_replace` arm
(app.rs:2218-2251). `repl = self.efind_replace.clone()` (the raw template). Caret: `Selection::caret(CharOffset::from(s + repl.chars().count()))`
(app.rs:2233). Resume: `s + repl.chars().count()` (app.rs:2247). BOTH use the TEMPLATE length. (Ticket cited
2079/2084 — drifted.)

**F3 (D5) — the literal case is ALREADY guarded; the delta is the capture-era case:** the resume already special-
cases `s == e && repl.is_empty() → s + 1` (app.rs:2245-2248) — so an empty match + empty *literal* replacement
already advances. The read side `refresh_efind_matches` (app.rs:13004, mutants::skip) resumes via
`matches.partition_point(|&(ms,_)| ms < resume)` (app.rs:13073). The capture-era F3: a zero-width match whose
EXPANSION is empty (template `${invalidref}` → "") — `repl.is_empty()` is false (template non-empty) so today it
would use the template length. Fix unifies with F2: key length AND emptiness off the expanded text. (Ticket cited
partition_point at 10939 w/ predicate `ms < s` — actually 13073 w/ `ms < resume`.)

**The undo primitive (D3):** the literal `replace_all(buffer, &[(CharOffset,CharOffset)], repl) -> usize` lives in
**find.rs:114** (NOT buffer.rs — ticket wrong): `begin_undo_group(sel)` → `for (s,e) in matches.iter().rev() {
buffer.edit(s..e, repl, Human) }` → `end_undo_group(sel)`, early-return 0 on empty. `begin_undo_group`
(buffer.rs:731) calls `undo.begin_group(sel, false)` — the `false` = not-cursor-anchored (positional; the named
`cursor_anchored` field is undo.rs:49). The new `replace_all_with(buffer, &[(range, String)]) -> usize` mirrors it
exactly, built on `Buffer::edit` (buffer.rs:250). No `replace_all_with`/capture primitive exists today (net-new).

**mutants::skip boundary (the #336 cov-blind trap):** the WHOLE Replace-One + Replace-All arms are inside
`fn handle_efind_key` (`#[cfg_attr(test, mutants::skip)]` at app.rs:2175); `refresh_efind_matches` is skip'd at
app.rs:12979. ⟹ the correctness MUST live in pure `find.rs` fns (expand/replace + `replace_all_with`), cov/MSI 100;
the app shims are proven by headless drives.

**Headless-drive models (D — the required test shape):** `editor_replace_flows_headless` (headless_drive.rs:1111 —
`cmd-f`, type query, `tab`, type replacement, `enter enter enter`, assert whole buffer text, `cmd-enter` replace-
all, `cmd-z`); `efind_empty_match_replace_advances_headless` (headless_drive.rs:6172 — `toggle-find-regex` +
`efind_refresh_for_test("(?m)^")` + `enter`, read `efind_index_for_test`) — the closest #347 model. Test hooks:
`efind_refresh_for_test` (app.rs:12985), `efind_index_for_test` (app.rs:13000), dispatch `"open-editor-find"` /
`"toggle-find-regex"`. A unit test of a skip'd handler proves nothing → assert real buffer text through the
keystroke path + NEGATIVE-SMOKE.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Approach.** D2 = **B (re-run at replace)**, CONFIRMED by the recon: the regex is compiled per-call and never
stored, and the app memoizes only offset pairs, so there is no live `Captures` to grow the read shape around —
and `Captures<'h>` borrows the haystack, so it can't be stored anyway. B adds one match PASS at replace time and
NO stored state. The D-BYTE-CHAR-AT-ONE-SEAM cost is CONTAINED by putting the re-run + the byte→char walk in ONE
pure `find.rs` fn (`replace_all_regex`), reusing the exact monotone `char_at` walk `find_all_regex` already uses
(§14: no fallible lookup, no panic). ALL correctness lives in pure `find.rs` fns (cov/MSI 100); the two cov-blind
`mutants::skip` app.rs arms only wire. This honors §14 (typed `FindError`, no panic on the input path — the
pattern arrives keystroke-by-keystroke) and §20 = N/A (Marley's own find bar; `regex::Captures::expand` is
adoption).

**The pieces (exact signatures):**
1. **`find::build_regex(pattern: &str, case_insensitive: bool) -> Result<regex::Regex, FindError>`** (private) —
   extract the `RegexBuilder + size_limit + the CompiledTooBig→TooComplex / else→InvalidPattern` mapping that
   `find_all_regex` (find.rs:211-218) has today; `find_all_regex` is refactored to call it (pure extraction,
   behavior-identical — its tests stay green). DRY: `replace_all_regex` reuses the SAME compile + error mapping.
2. **`find::replace_all_regex(text, pattern, case_insensitive, template: &str) -> Result<Vec<(CharOffset,
   CharOffset, String)>, FindError>`** (pub) — empty pattern → `Ok(vec![])` (mirrors `find_all_regex`);
   `build_regex(...)?`; the SAME monotone `char_at` walk (duplicated with the same `debug_assert` loudness —
   `captures_iter` is ascending + non-overlapping like `find_iter`, so the flat boundary sequence is
   non-decreasing); then
   `re.captures_iter(text).filter_map(|caps| caps.get(0).map(|m| { let mut s = String::new();
   caps.expand(template, &mut s); (char_at(m.start()), char_at(m.end()), s) })).collect()`. The `filter_map` +
   `caps.get(0).map` keeps the "group 0 always exists" invariant TOTAL with no panic and no uncoverable branch
   (the None flow stays inside the `Option` combinator — the #346 `node_kind_at` idiom). `caps.expand` is the
   regex crate's `$1`/`${name}`/`$$` grammar (D1 — adopted, not hand-rolled).
3. **`find::replace_all_with(buffer: &mut Buffer, edits: &[(CharOffset, CharOffset, String)]) -> usize`** (pub) —
   mirrors `replace_all` (find.rs:114) EXACTLY but each edit carries its own string:
   `let Some((first,_,_)) = edits.first() else { return 0 };` → `begin_undo_group(caret@first)` →
   `for (s,e,repl) in edits.iter().rev() { buffer.edit(*s..*e, repl, Human) }` → `end_undo_group`. The `restore`/
   not-cursor-anchored compliance (AD-claude-edit-post-state-span-and-earned-undo-tags-001) falls out of reusing
   `begin_undo_group` (which passes `false`). Back-to-front is correct for owned strings exactly as for literal.
4. **`find::resume_after(start: usize, end: usize, inserted_chars: usize) -> usize`** (pub) — `if start == end &&
   inserted_chars == 0 { start + 1 } else { start + inserted_chars }`. The F2+F3 arithmetic EXTRACTED out of the
   cov-blind arm so it is cov/MSI 100 (the #336 lesson — mutation-test the arithmetic that matters). F2: the
   length is the INSERTED (expanded) char count; F3: a zero-width match with an empty expansion still advances +1.

**The app.rs arms (both in `handle_efind_key`, `mutants::skip` — kept minimal, defensive):**
- **Replace-One (app.rs:2218-2251):** capture `regex=self.efind_regex`, `query=self.efind_query.clone()`,
  `fold=self.efind_fold`, `index=self.efind_index`, `repl=self.efind_replace.clone()`; `let mut inserted =
  repl.chars().count();` (literal default); inside the surface/buffer block compute
  `let expanded = if regex { let text = buffer.text(); marley_editor::replace_all_regex(&text, &query, fold,
  &repl).ok().and_then(|e| e.get(index).map(|(_,_,x)| x.clone())).unwrap_or_else(|| repl.clone()) } else {
  repl.clone() };` → `inserted = expanded.chars().count();` → `buffer.edit(s..e, &expanded, Human)` → caret
  `CharOffset::from(s + inserted)`; then AFTER the block `self.efind_resume = Some(marley_editor::resume_after(s,
  e, inserted));`. LITERAL mode: `expanded == repl` → byte-identical to today (D-LITERAL-UNCHANGED); the resume
  equals the old `s + repl.chars().count()` / `s+1` because `resume_after` reproduces it.
- **Replace-All (app.rs:2197-2216):** capture `regex`/`query`/`fold`/`repl`; inside the block:
  `if regex { let text = buffer.text(); if let Ok(edits) = marley_editor::replace_all_regex(&text, &query, fold,
  &repl) { marley_editor::replace_all_with(buffer, &edits); } } else { marley_editor::replace_all(buffer,
  &matches, &repl); }`. An invalid pattern never reaches here (refresh sets empty matches → the `!matches.is_empty()`
  guard skips the arm); the `if let Ok` is belt-and-suspenders. LITERAL mode unchanged.

**File manifest:**
| File | Change |
|---|---|
| `crates/editor/src/find.rs` | ADD `build_regex` (private) + refactor `find_all_regex` to use it; ADD pub `replace_all_regex`, `replace_all_with`, `resume_after`. (+ Phase-4 tests.) |
| `crates/editor/src/lib.rs` | extend the find re-export (lib.rs:43) with `replace_all_regex, replace_all_with, resume_after`. |
| `crates/marley_app/src/app.rs` | Replace-One arm (regex expansion + F2/F3 via `resume_after`) + Replace-All arm (regex branch → `replace_all_regex`→`replace_all_with`). Both in the `mutants::skip` `handle_efind_key`. |

**Regression Test Plan (headless/unit — the pure seams + negative-smoked drives):**
| # | Test | Kind / home | Proves |
|---|------|-------------|--------|
| T1 | `replace_all_regex`: `(\w+)@(\w+)`+`${2}_${1}` on `ab@cd` → `[(0,5,"cd_ab")]`; `$$`→`$`; `${9}` (invalid)→""; multi-match `a@b c@d` → two triples; empty pattern → `[]`; `a[` → `Err(InvalidPattern)`; a Unicode fixture (multi-byte) → correct char offsets (the monotone walk) | pure unit, find.rs | REQ-005-EXPAND + the byte→char walk (cov/MSI 100) |
| T2 | `replace_all_with`: back-to-front exact text + count; one undo reverts all; zero edits → 0 (mirror the `replace_all` tests find.rs:342-393) | pure unit, find.rs | REQ-005-UNDO ordering + one-group `restore` |
| T3 | `resume_after`: (0,0,0)→1, (0,0,3)→3, (2,5,0)→2, (2,5,9)→11 | pure unit, find.rs | REQ-010/011 arithmetic (kills the `s==e && ins==0` guard + `s+1`/`s+inserted` mutants) |
| T4 | ★ headless drive: regex `(\w+)@(\w+)` + `${2}_${1}` on `ab@cd ef@gh`, Replace-One twice → both replaced (2nd match NOT skipped), buffer text exact; NEGATIVE-SMOKE = revert the arm to `repl.chars().count()` → the 2nd match is skipped (test fails) | headless, model :1111 | REQ-010-F2 through the real cov-blind arm |
| T5 | ★ headless drive: regex `(?m)^` + template `${9}` (empty expansion) or `""`, Replace-One → advances (efind_index moves; no loop/double-apply); NEGATIVE-SMOKE = key the guard off the template → re-hit | headless, model :6172 | REQ-011-F3 (the capture-era empty-expansion case) |
| T6 | ★ headless drive: regex `${2}_${1}` Replace-All on a multi-match fixture → all replaced, ONE `cmd-z` restores | headless, model :1111 | REQ-005 end-to-end (Replace-All regex branch) |
| T7 | literal regression: literal Replace-One/All unchanged (existing `editor_replace_flows_headless` stays green) | existing headless | D-LITERAL-UNCHANGED |

Pure `replace_all_regex`/`replace_all_with`/`resume_after` = cov/MSI 100 in `marley_editor`. The app.rs arms are
`mutants::skip` (the drives T4/T5/T6 carry them). **NO live synthetic drive** — headless-drive = `cargo nextest`
(REQUIRED + safe even with chad at the machine); live CGEvents/screencapture stay OFF-LIMITS.

**Risks / decisions:** (a) D2=B second-match-site drift — CONTAINED: `replace_all_regex` owns the re-run + the
byte→char walk in one pure fn. (b) Replace-One re-runs `replace_all_regex` (expands all N) to get the nth — O(N)
per Enter, accepted (replace is not hot, N small); it also guarantees the expansion matches the range. (c) the
F2/F3 arithmetic is EXTRACTED to `resume_after` so it is mutation-tested, not trapped in the cov-blind arm. (d)
LITERAL mode is byte-identical — the branch diverges only when `efind_regex`. (e) `find_all_regex` is refactored
to call `build_regex` (pure extraction; its tests stay green). §14 honored; §20 = N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest, 3 files, in order. `cargo fmt --all`; `cargo check --workspace` CLEAN; `cargo clippy
-p marley_editor -p marley --all-targets -D warnings` rc 0 (only the pre-existing `block v0.1.6` dep
future-incompat warning). Diff = exactly find.rs + lib.rs + app.rs.

- **find.rs** — EXTRACTED `build_regex(pattern, ci) -> Result<regex::Regex, FindError>` (the RegexBuilder +
  size_limit + CompiledTooBig→TooComplex/else→InvalidPattern mapping) and refactored `find_all_regex` to call it
  (pure extraction — behavior-identical). ADDED `replace_all_regex` (empty→`Ok(vec![])`; `build_regex?`; the SAME
  monotone `char_at` walk copied verbatim, `debug_assert` retargeted to "captures_iter boundary regressed"; then
  `captures_iter().filter_map(|caps| caps.get(0).map(|m| { caps.expand(template, &mut s); (char_at(start),
  char_at(end), s) }))`). ADDED `replace_all_with(buffer, &[(CharOffset,CharOffset,String)]) -> usize` (mirrors
  `replace_all` — first-or-return-0, begin_undo_group, back-to-front `buffer.edit`, end_undo_group, len). ADDED
  `resume_after(start, end, inserted_chars) -> usize` (`if start==end && inserted_chars==0 { start+1 } else {
  start+inserted_chars }`). All docs plain-backtick.
- **lib.rs** — extended the find re-export (lib.rs:43) with `replace_all_regex, replace_all_with, resume_after`
  (wrapped to a multi-line `pub use` for rustfmt).
- **app.rs** — Replace-One arm (`"enter" if efind_focus_replace`): capture `regex`/`query`/`fold`/`index` +
  `let mut inserted = repl.chars().count()` before the surface block; inside, `expanded = if regex {
  replace_all_regex(&buffer.text(), &query, fold, &repl).ok().and_then(|e| e.get(index).map(|(_,_,x)| x.clone()))
  .unwrap_or_else(|| repl.clone()) } else { repl.clone() }`, `inserted = expanded.chars().count()`, edit with
  `&expanded`, caret `s + inserted`; after the block `efind_resume = Some(resume_after(s, e, inserted))` (replaced
  the old `match s==e && repl.is_empty()` block). Replace-All arm (`"enter" if platform`): capture
  `regex`/`query`/`fold`; inside, `if regex { if let Ok(edits) = replace_all_regex(&buffer.text(), &query, fold,
  &repl) { replace_all_with(buffer, &edits) } } else { replace_all(buffer, &matches, &repl) }`. Both arms stay in
  the `mutants::skip` `handle_efind_key`.

**Deviations from design:** none. `build_regex` returns `regex::Regex` (fully qualified — only `RegexBuilder` was
imported); the `char_at` walk is duplicated (not extracted — it is a stateful closure, cleaner copied, and the
#336 hazard is addressed by it being the SAME monotone walk, not a per-match conversion). Literal mode is
byte-identical (the branch diverges only when `efind_regex`).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 independent critics** (general-purpose), parallel, distinct lenses — scaled to the cross-crate change. Both
verified the WORKING TREE untouched (only the 3 `.rs` + the docs). `cargo check --workspace` exit 0.

- **Critic 1 — behavior-preservation + F2/F3 correctness:** code CLEAN on all of (a) literal==old, (b) F2, (c)
  F3+no-loop, (d) the char→byte walk, (e) index-alignment. Found **2 MEDIUM test-adequacy gaps** (below).
- **Critic 2 — adoption + totality + borrow/purity/docs:** SHIP-CLEAN on all of (a)–(e). Confirmed `caps.expand`
  is the sole expander (no hand-roll), `build_regex` char-identical to the old inline builder, `replace_all_with`
  mirrors `replace_all`, borrows sound, `regex` confined to `marley_editor` (no dep added to `marley_app`), every
  intra-doc link resolves to a pub item (no link to the private `build_regex`).

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| F1 | MEDIUM | **T4 (the F2 negative-smoke) is MASKED by the resume clamp.** `refresh_efind_matches` does `…partition_point(…).min(matches.len().saturating_sub(1))` (app.rs:13103). On the planned 2-match fixture `ab@cd ef@gh`, replacing match 0 makes `ab@cd`→`cd_ab` (no `@`), so the set drops to 1 → `index.min(0)==0` for ANY resume → the buggy resume (9) and correct resume (5) BOTH land on `ef@gh`. The smoke passes even with F2 reverted → it cannot catch the bug it exists to catch. | **REAL — confirmed** (read app.rs:13103; the clamp masks it). Code is CORRECT; the TEST is inadequate. **FIX (carried to P4):** T4 uses a ≥3-match fixture with a SURVIVING MIDDLE match — `ab@cd e@f g@h` + `${2}_${1}`: correct resume (5) → next hits `e@f`; buggy resume (9) → `partition_point(ms<9)` skips to `g@h`, leaving `e@f` unreplaced → texts DIFFER → the revert is observable. |
| F2 | MEDIUM | **T5 (the F3 negative-smoke) does NOT reproduce a loop on the planned fixtures.** The loop needs resume ≤ s. Reverting the guard to `s==e && repl.is_empty()` on `(?m)^` + `${9}` (template len 4 → `s+4`, overshoots but ADVANCES) or `""` (→ `s+1`, == the fix) never yields resume ≤ s; over 2 line-starts the overshoot clamps invisibly → "index moved" passes for BOTH the fixed and reverted guard. | **REAL — confirmed.** Code is CORRECT; the smoke can't distinguish. **FIX (carried to P4):** T5 uses a short-line fixture `a\nb\nc` + `${9}` and asserts the advance lands on the IMMEDIATELY-NEXT zero-width match (offset 2): fixed steps 0→2→4 (every line-start); reverted overshoots 0→4 (skips offset 2) → observable. |
| F3 | LOW | `active_editor_mut()==None` in Replace-One (regex) leaves `efind_resume` from the template-length default. | **CLEARED — benign** (Critic 1): no edit occurred, and refresh has already closed the bar (`active_editor().is_none()` → `efind_open=false`) so the next refresh early-returns + clears matches — the stale resume is inert. No corruption. |
| F4 | LOW | Replace-One re-expands all N matches to fetch the nth (O(N)/Enter); `char_at` walk duplicated verbatim. | **ACCEPTED — design tradeoffs** (Critic 2 + Phase-2 risks a/b): replace is not hot, N small, and the re-run guarantees the expansion matches the range; the duplicated walk is a stateful closure cleaner copied than extracted (a maintainability watch-item — fix the twin if one is ever fixed). |

**Result: 0 code changes (both critics: the implementation is correct in literal AND regex mode). 2 MEDIUM
test-adequacy fixes carried into the Phase-4 plan (F1→T4 3-match surviving-middle; F2→T5 short-line
immediately-next).** These are the batch's "a green test proves less than you think" lesson made concrete — a
negative-smoke that does not actually REPRODUCE the reverted bug proves nothing. Lesson for P5 capture: a
negative-smoke must be shown to FAIL against the reverted fix on a fixture where the bug is OBSERVABLE (not masked
by a downstream clamp / a self-destroying match / an overshoot-that-still-advances).

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests written** (6 new + 1 `#[cfg(test)]` hook; all green):

| # | Test | File | Proves |
|---|------|------|--------|
| T1 | `t347_replace_all_regex_expands_captures` | find.rs | REQ-005 — `${2}_${1}`→`cd_ab`, `$$`→`$`, `${9}` invalid→"", multi-match ranges, empty pattern→`[]`, `a[`→`Err(InvalidPattern)`, a MULTIBYTE `é@x` proving CHAR (not byte) offsets |
| T2 | `t347_replace_all_with_back_to_front_one_undo` | find.rs | REQ-005-UNDO — grow/shrink/grow back-to-front (`XX--YYY`), ONE undo reverts all + caret@first, zero edits→0 no group |
| T3 | `t347_resume_after_f2_and_f3` | find.rs | REQ-010/011 arithmetic — (0,0,0)→1, (0,0,3)→3, (2,5,0)→2, (2,5,9)→11 |
| T4 | `t347_replace_one_regex_resumes_from_expanded_length_headless` | headless_drive.rs | ★ REQ-010-F2 through the real cov-blind arm — the STRENGTHENED 3-match surviving-middle fixture `ab@cd e@f g@h` |
| T5 | `t347_replace_one_empty_expansion_advances_to_next_match_headless` | headless_drive.rs | ★ REQ-011-F3 — the STRENGTHENED short-line `a\nb\nc` + `${9}` fixture (immediately-next) |
| T6 | `t347_replace_all_regex_expands_and_undoes_headless` | headless_drive.rs | REQ-005 end-to-end — regex Replace-All `b_a d_c` + one `cmd-z` restores |
| hook | `efind_set_replace_for_test` (app.rs, `#[cfg(test)]`) | app.rs | seeds the replace field so a drive can use a `${2}_${1}` template without brittle shift-combo keystrokes |

**Runs (`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley_editor` → **241 passed, 0 skipped** (T1/T2/T3 + the existing find suite).
- `cargo nextest run -p marley` → **725 passed, 2 skipped** (T4/T5/T6 + no regression; was 722, +3).

**★ NEGATIVE-SMOKES (the inspect-mandated proof — each reverted fix makes its test FAIL observably):**
- **Smoke 1 (F2):** reverted `resume_after(s,e,inserted)` → `s + repl.chars().count()` (template length 9) → T4 FAILED: `left: "cd_ab e@f h_g\n"` vs `right: "cd_ab f_e g@h\n"` — the overshoot skipped the middle match `e@f`, hit `g@h`. Restored → green.
- **Smoke 2 (F3):** reverted to the GENUINE pre-#347 arm (`if s==e && repl.is_empty() { s+1 } else { s + repl.chars().count() }`) → T5 FAILED: `left: 2` vs `right: 1` — the template-length resume (`${9}` len 4) overshot to index 2, skipping the line at offset 2. Restored → green. (A first revert with the expanded length in the else produced a DIFFERENT break — `left: 0`, a loop — also caught; the genuine-arm revert matches the inspect-F2 prediction exactly.)
- Both smokes confirm the STRENGTHENED fixtures catch the bug the original 2-match / 2-line fixtures would have masked (inspect F1/F2 + `PR-claude-negative-smoke-must-reproduce-the-reverted-bug-observably-001`).

**Mutation (`cargo mutants --in-diff` on find.rs):** **20 mutants: 19 caught, 1 unviable, 0 MISSED → MSI 100** on the viable set. The 1 unviable = `build_regex → Ok(Default::default())` (`regex::Regex` has no `Default` — legitimately excluded). Killed: `replace_all_regex` 3 body + the char-walk `>=`/`+=` by T1 (incl. multibyte); `find_all_regex` body mutants by the existing t339 suite; `replace_all_with → 0/1` by T2; all 8 `resume_after` body/guard/arithmetic by T3. lib.rs (re-export, no logic) + app.rs arms (`mutants::skip`) not mutated.

**NO LIVE SYNTHETIC DRIVE — stated.** chad may be at the machine; a live CGEvents/screencapture keyboard drive is off-limits AND unnecessary here. The three headless drives (T4/T5/T6) are `cargo nextest` code-level drives through the REAL production keystroke arms (`open-editor-find`/`toggle-find-regex`/`tab`/`enter`/`cmd-enter`/`cmd-z`) — the exact proof, safe under the constraint. headless-drive ≠ live-drive.

**FULL `--diff` gate → `GATE GREEN [diff]` — 15/15.** Receipt `54a71f5c479db1c238e8b04eb7e583df3bd7075d`. gate:4 coverage 100% lines (find.rs pure fns tested; app.rs excluded), gate:5 MSI 100, gate:14 docs PASS. **One flake handled:** the FIRST gate run tripped gate:3 on a TIMEOUT of `search_open_dismisses_live_completion_headless` (test 1720/1720, >180s) — an UNRELATED test (search live-completion, nothing to do with #347) that passes standalone in 0.084s; it hung only as the last-scheduled test under gate load. Confirmed a flake (isolated re-run green), re-ran the gate → clean. No source change; §0 upheld (no baseline/suppression).

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):**
- CHANGELOG.md — `### Added` for #347 (capture-group regex Replace via `Captures::expand`; the re-run-at-replace
  `replace_all_regex` + `replace_all_with`; the F2 expanded-length + F3 empty-expansion-advance resume; literal
  byte-identical).
- `docs/marley_architecture/editor.md` — the `find.rs` bullet gained a `M22 #347 — regex REPLACE` sub-bullet
  (the dissolved fork, `build_regex` extraction, the single byte→char walk, `resume_after` PURE-so-it's-mutation-
  tested for F2/F3); the #339 "replace-side empty-match loop is app-side" parenthetical updated to point at
  `resume_after`.
- (find.rs/app.rs doc-comments were written in Phase 3 — no restore needed.)

**Capture (forge wired):**
- **aar-submit** 3b62706e — outcome completed, effectiveness 4, 1 novel finding, 13 verdicts. Lessons: (a) the
  D2 fork DISSOLVED on the recon — the read path stores only offset pairs + never keeps the Regex + a Captures
  borrows the haystack → "carry the captures" is impossible; B (re-run) adds no stored state, contained in one
  pure find.rs fn (#336 one-forward-walk). (b) ★ the inspect WIN — code was clean both critics, but Critic 1
  found the two PLANNED negative-smokes couldn't reproduce their bug (T4 masked by the `.min(len-1)` clamp on a
  2-match self-destroying fixture; T5 overshoot-still-advances over 2 lines) → strengthened to a 3-match
  surviving-middle + a short-line immediately-next assertion; both then FAILED observably at P4. (c) the regex
  crate owns `Captures::expand` (check the crate first). (d) extract F2/F3 arithmetic to a PURE `resume_after`
  → mutation-tested (MSI 100), not trapped in the cov-blind arm. (e) #203/#204 — `build_regex → Default` unviable
  (`regex::Regex` has no Default), 1 legitimately excluded.
- **prevention-rule-record** (at inspect): `PR-claude-negative-smoke-must-reproduce-the-reverted-bug-observably-001`
  (f3a04585) — a negative-smoke proves nothing unless the reverted fix makes the test FAIL observably on that
  fixture; watch for a downstream clamp / self-destroying input / overshoot-that-still-advances masking it.
- **failure-record:** none SHIPPED — F2/F3 were PRE-EXISTING bugs this ticket FIXED (its purpose), not
  regressions; the inspect MEDIUMs were test-adequacy caught+fixed in-phase.
- No follow-up ticket — the ticket is complete; the O(N)-re-expand-per-Enter + the duplicated `char_at` walk are
  accepted LOW watch-items (Phase-3.5 F4), not scheduled work.

**Close + archive:** forge #347 closed; TICKET-347 → closed/; pair archived active/ → completed/; spec Phase 5 PASS.

⚠️ **/commit — the receipt HOLDS.** The Phase-5 edits are docs-only (CHANGELOG + editor.md); the source
(find.rs/lib.rs/app.rs/headless_drive.rs) is UNCHANGED since the Phase-4 receipt `54a71f5c…` was minted (the two
negative-smokes were reverted THEN restored byte-identical — verified: `grep NEGATIVE-SMOKE crates/` is empty, the
`resume_after(s, e, inserted)` line is restored). /commit re-runs the gate anyway per its own rule if any `.rs`
staged — but no `.rs` changed post-receipt, so a fingerprint verify is valid.

**Status: Phase 5 — Complete PASS.**
