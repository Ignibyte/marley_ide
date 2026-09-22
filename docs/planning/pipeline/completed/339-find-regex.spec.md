---
pipeline_id: 7928abd0-f636-48fc-8406-eb65ee527cc4
ticket: forge#339 (1e6c0a52-f41b-46d5-90d7-1f98e036bd36) · local docs/planning/tickets/closed/TICKET-339-find-regex.md
aar_id: 34cfa1c3-97cb-48a8-a60b-1e999f67b518
status: Phase 5 — Complete PASS
title: Regex FIND — a .* mode on the ⌘F bar + the case chip (capture-REPLACE split to #347 at Phase 2)
type: feature
milestone: M22
references: [the #272 find bar (find_all/replace_all, n-of-m — NOT F3, see F4), find.rs:73/:106, AD-claude-edit-post-state-span-and-earned-undo-tags-001 (#338 — verified compatible, F9)]
---

<!-- PRE-AUTHORED (the Fable method), promoted by /pipeline:plan, which re-verified every cited seam.
     THE PHASE 1 LEDGER IS IN THE NOTES AND THIS TICKET HAS THE MOST DRIFT OF THE BATCH:
       * D-SAME-SHAPE is FALSE as stated — and this spec CONTAINS ITS OWN REFUTATION (F1).
       * The case chip and F3 that REQ-002/003/007 reference DO NOT EXIST (F4).
       * `fold` (ASCII) and `(?i)` (Unicode) genuinely disagree — a product decision this spec never names (F5).
       * Empty matches resurrect the F5 infinite loop the resume mechanism was built to prevent (F3).
     Read the ledger before designing. -->

## Method note (run 4 of promote-don't-author)
Runs 1-3 each caught an authorship error before Design (a REQ on a deleted seam; a false central premise; a
central bullet wrong for 3 of 4 actions). Run 4 caught the most: a central decision that is **self-refuting**,
two REQs naming controls that were never built, and two silent-corruption hazards sitting in a
coverage-excluded `mutants::skip` shim — the exact place #336's bug hid.

## Title
The ⌘F bar is literal + ASCII-fold only (`find_all(buffer, needle, fold)`, find.rs:73). This adds a **regex
mode** — the `.*` chip — with capture-group replace (`$1`/`${name}`), riding everything #272 already built:
the match list, n-of-m, F3/⇧F3, and `replace_all` (find.rs:106, back-to-front — it already exists; this
ticket is regex *mode*, not replace-from-scratch — **VERIFIED: `replace_all` TAKES its matches, so it is
already mode-agnostic**). ~~**Zero new crates**: `regex` + `regex-automata` are already in Cargo.lock via
#326's `ignore` adoption~~ — **AMENDED at Phase 1 (F7): the conclusion holds, the attribution is FALSE.**
`cargo tree -i regex` shows `regex` arrives via **`gpui_util`→gpui** and **`tree-sitter`→marley_syntax**;
`ignore` pulls `globset` → `regex-automata` + `regex-syntax` but **NOT `regex` itself**. So there is no new
audit/deny surface — but `crates/editor` depends on none of gpui/tree-sitter/ignore, and its Cargo.toml is
deliberately `ropey` + `marley_text_offsets` and nothing else. It would gain its **third** dependency, in the
workspace's purest crate. A decision, not a footnote (Fork D).

## SCOPE SPLIT at Phase 2 (Fork C) — this ticket is the FIND half
**#339 = regex FIND + the `.*` chip + the case chip.** Capture-group REPLACE (`expand_captures`,
`replace_all_with`, and the F2/F3-replace silent-corruption fixes) is **→ #347**, with the findings already
written up for it. Not dropped, deferred with evidence: (a) F1 — capture-replace *breaks* D-SAME-SHAPE (the
match list discards the captures replace needs) and needs its own design + critics; (b) F2/F3 are fixes in a
`mutants::skip` shim inside a **coverage-excluded** file — the exact place #336's bug hid; (c) regex FIND is
useful alone and provable via the OFF-identity property. **REQ-005/010/011 move to #347.** Until it lands the
bar has a `.*` chip whose Replace is literal-only — it never claims capture support, and it keeps working.

## Scope
### In
- **The mode chip** — a `.*` toggle on the ⌘F bar (in-memory per-run bar state; not a persisted setting v1).
  ~~The existing case-fold chip maps to `(?i)` in regex mode.~~ **AMENDED at Phase 1 (F4): THERE IS NO CASE
  CHIP.** `fold: true` is HARDCODED at app.rs:10932 — "like the fold chip" describes a control that was never
  built. This is *build a chip* (new state + render + binding), not *map* one. **And (F5) `fold` and `(?i)`
  genuinely disagree**: `fold` compares per-char with `eq_ignore_ascii_case` (find.rs:81-86 — ASCII-only,
  length-preserving BY CONSTRUCTION, pinned at find.rs:165), while `(?i)` is Unicode simple case folding
  (`(?i)k` matches U+212A KELVIN SIGN). Same query + same chip → **different match counts**. Matching today
  needs `RegexBuilder::unicode(false)`, which also changes `.`/`\w` — a product call (Fork B).
- **The pure seam extends `find.rs`** (cov/MSI 100):
  - `find_all_regex(text, pattern, case_insensitive) -> Result<Vec<(CharOffset, CharOffset)>, FindError>` —
    the SAME range shape `find_all` returns, so the match list / n-of-m / highlight bands consume it with zero
    change. **VERIFIED for the READ path** (F1): nothing recomputes a match end from the query's length —
    zero hits for `+ needle/query.len()` — so varying-length matches carry, and the bar already stores
    `Rc<Vec<(usize,usize)>>` with the newtype stripped at exactly ONE `map` (app.rs:10934).
    **But this return type is ALSO why D-SAME-SHAPE cannot hold for the WRITE path: it discards the captures
    `expand_captures` needs.** See F1 / Fork A. (`FindError` is NEW — zero hits repo-wide.) Byte↔char offset mapping at the seam (the regex crate speaks bytes; the buffer speaks
    chars — the #309-class bridge, one place).
  - **The empty-match rule** — a pattern that matches empty (`a*`, `^`, `\b`) advances one char past an
    empty match instead of looping; matches stay finite and ordered. Pinned by its own table row.
  - `expand_captures(match, captures, template) -> String` — `$1`/`${name}`/`$$` (the regex crate's own
    expansion), feeding a generalized `replace_all_with(buffer, &[(range, String)])` — per-match
    replacement strings, because captures differ per match; the existing single-string `replace_all` stays
    for literal mode (byte-identical path).
  - **Compile safety** — `RegexBuilder` with a bounded `size_limit`; the regex crate is linear-time by
    construction (no catastrophic backtracking class at all).
- **Invalid-pattern UX** — mid-typing patterns are transiently invalid (`fo(` on the way to `fo(o)`): the
  bar shows an inline error state (danger-tinted border + the message), the match list is empty, F3 no-ops,
  Replace disables. NEVER a panic; the error is a value (`FindError`), not an exception path.
- **Replace with captures** — Replace-one steps per match; Replace-all is ONE undo unit (the existing
  back-to-front discipline extended to per-match strings).
- **Literal mode untouched** — with the chip off, every path is byte-identical to today (the OFF-identity
  discipline; the property test pins `find_all` unchanged).
### Out (explicitly)
- Regex in the ⌘⇧F PROJECT search (#326 phase 2 territory — its own ticket; this is the editor bar only).
  Unicode case-fold for literal mode (a named #272 follow-up, separate). Multiline-mode flags UI (`(?m)`
  typed inline works; no chip v1). Search history.

## Reference (§20)
**The `regex` crate = published-API reuse (MIT/Apache, already a transitive dep)**; VS Code = OBSERVED bar
behavior (the `.*` chip, inline invalid-pattern error, capture `$n` replace). No copyleft source read.

### Prior art
*(Recorded retroactively at Phase 3 — the sweep itself happened at Phase 2 and is why D-EMPTY-ADVANCE is gone.
This ticket is the evidence the step now exists.)*

1. **Behavior maps / observed** — VS Code's find bar is the observed shape (the `.*` chip, the inline
   invalid-pattern error, `$n` replace). Behavior only; no copyleft source read.
2. **Published material** — the `regex` crate's own `Replacer` syntax is the de-facto standard for `$1` /
   `${name}` / `$$`, so #347 adopts the convention rather than inventing one.
3. **OUR PERMISSIVE DEPS — and this leg paid, decisively.**
   - **`regex` 1.12.4** (in-lock via gpui + tree-sitter). **It DISSOLVED a locked decision.** D-EMPTY-ADVANCE
     proposed hand-rolling "empty matches advance one char instead of looping". `Regex::find_iter` already
     does it — regex-automata 0.4.13 `util/iter.rs:30-36` states the problem and the fix verbatim, and its
     rule is **better than the one we were about to write**: it advances only on an empty match that
     *overlaps a previous match*, where a blind advance-by-one would make `^` skip every other line start.
     Reading it is ADOPTION, outside the wall (the #336 gpui precedent).
   - **`regex::Error`** (error.rs:8-31) has exactly two shapes — `Syntax(String)` and `CompiledTooBig(usize)`
     — so `FindError`'s two variants MIRROR the crate instead of guessing. Read before writing.
   - **`regex::escape`** already owns "text → literal pattern", so `escape_literal` wraps it (one line) rather
     than hand-rolling metacharacter escaping. REQ-012 came free.
   - **`RegexBuilder::size_limit`** already owns the complexity bound — REQ-008 is a call, not an algorithm.
   - **Checked and found NO owner:** the byte→char conversion. `regex` reports bytes, ropey counts chars, and
     nothing bridges them — so that pass is genuinely ours to write, and it is the ticket's only real offset
     math. Saying so is the point of the sweep: it separates what we must build from what we merely had not
     looked up.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-SAME-SHAPE** — ~~regex results reuse the exact `(CharOffset, CharOffset)` list; downstream (#272's
  n-of-m/F3/bands/replace plumbing) does not know the mode exists.~~ **AMENDED at Phase 1 (F1) — NARROWED to
  the READ path, where it is a real and verified result; FALSE for the write path, where this spec refutes
  itself.**
  - **READ (bands, n-of-m, select-current, ⌘D): VERIFIED mode-blind.** Nothing recomputes an end from the
    query's length (zero hits), so varying-length matches carry. ⌘D is a PEER consumer (`fold: false`,
    literal `text_in_range`), so the mode cannot reach it.
  - **WRITE: impossible.** `find_all_regex -> Vec<(CharOffset, CharOffset)>` **discards the captures**
    `expand_captures` requires — the shape carries only BECAUSE it throws away what capture-replace needs.
    And D-PER-MATCH-REPLACE below introduces `replace_all_with`, which IS "the replace plumbing" this
    decision claims is untouched. **Fork A: grow the return type to carry captures, or re-run the regex at
    replace (violating D-BYTE-CHAR-AT-ONE-SEAM with a second match site).**
- **D-READ-PATH-MODE-BLIND** *(the honest half of D-SAME-SHAPE, kept as its own decision)* — the match
  list/bands/counter consume `(CharOffset, CharOffset)` unchanged; they also rely on ORDERING + NON-OVERLAP,
  which regex output satisfies by construction but which the TYPE does not carry — say it, do not inherit it.
- **D-ERROR-IS-A-VALUE** — `FindError` renders as bar state; no `unwrap` on the compile path (§14).
- ~~**D-EMPTY-ADVANCE** — empty matches advance-by-one; the table row is the guard.~~ **DISSOLVED at Phase 2 —
  the crate already does it, and better.** `regex::find_iter` owns this: regex-automata 0.4.13
  `util/iter.rs:30-36` says *"if an empty match is found… iteration would never end. Instead, a `Searcher`
  knows how to detect these cases and forcefully advance iteration in the case of an empty match that
  **overlaps with a previous match**."* That last clause is why it is better than the proposed rule: blindly
  advancing on every empty match would make `^` skip line starts. **A spec decision dying because the
  substrate already does it is a WIN to record** (the #336 precedent — reading a permissive dep settles the
  fork). The test row survives as a PIN on the crate's behavior at our seam, not as a guard on our own loop.
- ~~**D-PER-MATCH-REPLACE**~~ **→ #347** (the Phase 2 split). `replace_all` stays exactly as-is here — it
  already TAKES its matches, so it is mode-agnostic, and it already satisfies #338's earn-the-tag rule
  (`begin_undo_group` passes `cursor_anchored: false`, buffer.rs:683 — verified, F9).
- **D-BYTE-CHAR-AT-ONE-SEAM** — the byte→char conversion happens once in `find_all_regex`, nowhere else.
  (`Buffer::char_to_byte`/`byte_to_char` exist — buffer.rs:108/113. Fork A's "re-run the regex at replace"
  option would break this by adding a second match site; that is the cost to weigh.)
- **D-REPLACE-ONE-MUST-NOT-TRUST-THE-TEMPLATE** *(NEW — Phase 1 F2)* — Replace-One currently computes both
  the caret AND `efind_resume` from `repl.chars().count()` (app.rs:2079/2084) — the **template's** length.
  Under capture expansion the template and the inserted text differ (`${2}_${1}` is 9 chars; `cd_ab` is 5),
  so the caret lands past the replacement and **`efind_resume` silently skips every match in the gap**. Both
  lines are `#[cfg_attr(test, mutants::skip)]` in a **coverage-excluded** file — neither coverage nor mutation
  can catch this. It must use the EXPANDED length.
- **D-EMPTY-MATCH-IS-MORE-THAN-A-FIND-RULE** *(NEW — Phase 1 F3)* — D-EMPTY-ADVANCE covers the FIND loop.
  It does not cover: (a) **Replace-One on an empty match with an empty replacement sets `efind_resume = s`,
  and the next `partition_point` lands on the SAME match — the exact infinite loop the resume mechanism was
  built to prevent**; (b) an empty match is COUNTED by n-of-m but paints NOTHING (`row_selection_cols` → None
  when `s >= e`, code_view.rs:259), so the bar reads "3 of 7" over 4 visible bands. Both need a decision.
- **D-RESEED-IS-NOT-A-PATTERN** *(NEW — Phase 1 F6)* — ⌘F stuffs raw selected text into `efind_query`
  (app.rs:6361). Select `foo(bar)` → a VALID regex meaning something else; select `a[` → the bar opens
  already in the error state. Reseed must `regex::escape` or force literal mode.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | find regex matches in mode-on (`fo+` over "fo foo fooo" → 3 correct char ranges, multibyte-safe: a match after an emoji lands on the right chars) | pure table |
| REQ-002 | ~~map the case chip to `(?i)` in regex mode~~ **AMENDED (F4): BUILD a case chip** — `fold: true` is HARDCODED (app.rs:10932); no chip exists. **And (F5) decide + pin whether regex mode matches `fold`'s ASCII folding (`unicode(false)`) or diverges** — same query + same chip currently yields different counts | pure + headless |
| REQ-003 | surface an invalid pattern as an inline bar error — empty matches, **find-next** no-op, Replace disabled, no panic. **AMENDED (F4): there is no F3** — find-next is Enter/⇧Enter (`handle_efind_key`, app.rs:2087-2092) | pure (FindError) + headless |
| REQ-004 | terminate with finite ordered matches on empty-matching patterns (`a*`, `^`, `\b`) via advance-by-one | pure table |
| ~~REQ-005~~ | ~~expand `$1`/`${name}`/`$$` per match in Replace~~ **→ MOVED to #347 (the Phase 2 split)** | — |
| REQ-006 | keep literal mode byte-identical to today with the chip off (`find_all` + `replace_all` untouched paths) | property |
| REQ-007 | feed regex matches through the UNCHANGED n-of-m / **find-next (Enter/⇧Enter, not F3 — F4)** / highlight-band plumbing | headless |
| ~~REQ-010~~ | ~~compute Replace-One's caret + `efind_resume` from the EXPANDED text's length~~ **→ MOVED to #347** (unreachable until captures land: with a literal `repl`, template length == inserted length) | — |
| REQ-011 | **(NEW — F3, SPLIT at Phase 2)** the FIND half only: an empty match must not break the match list, and the n-of-m/paint disagreement is DOCUMENTED (an empty match is counted but paints no band — suppressing them would make `^`/`\b` report 0, a worse lie; the zero-width band is a filed follow-up). **The replace-side loop → #347.** | pure + headless |
| REQ-012 | **(NEW — F6)** treat ⌘F's reseeded selection as TEXT, not a pattern (`foo(bar)` finds itself; `a[` does not open the bar in an error state) | headless |
| REQ-008 | bound pattern compilation (`size_limit`); an over-limit pattern is a FindError, not an OOM | pure |
| REQ-009 | add zero new workspace dependencies — **AMENDED (F7): true for Cargo.lock** (regex is in-lock via gpui + tree-sitter, NOT via `ignore` as claimed), but `crates/editor/Cargo.toml` gains its THIRD dep | Cargo.lock diff = empty |

## Phase Plan
P2 confirm the bar-state seam (`efind_query`, the chips) + where `find_all` is invoked + the replace-all
call site; P3 pure fns first (find_all_regex / expand_captures / replace_all_with + FindError), then the
chip + bar states; P3.5 critics on the byte↔char seam (the ONE conversion), the empty-match rule, and the
undo unit; P4 the tables + a headless regex-replace drive + gate (note: NEW file? — none expected; find.rs
extends, so no add -N needed unless design splits a module); P5 docs.
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
