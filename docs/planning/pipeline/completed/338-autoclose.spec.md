---
pipeline_id: c3ce81e3-dbcb-4792-876d-e4d935608c22
ticket: forge#338 (19428bae-8f16-4bdb-b3c8-c9850e54dd27) · local docs/planning/tickets/closed/TICKET-338-autoclose.md
aar_id: acd1a989-2571-43db-82d1-78a7a467c007
status: Phase 5 — Complete PASS
title: Auto-close brackets + quotes — pair-insert, type-over, backspace-pair, wrap-selection (at N cursors)
type: feature
milestone: M22
references: [the #296/#297 multi-cursor edit engine (edit_at_selections_with — TEXT half only; see the Phase 1 ledger), crates/editor/src/ime.rs:100 replace_text (THE typed-char door), the #299 per-cursor line-op template, the #330 settings four-wiring]
---

<!-- PRE-AUTHORED (the Fable method), promoted by /pipeline:plan, which re-verified every cited seam.
     THE PHASE 1 LEDGER IS IN THE NOTES AND IT MATTERS: the seams are real, but the bullet under
     "N cursors, one undo unit" below is WRONG for 3 of its 4 actions — the shipped engine returns bare
     CARETS at the end of each insert, so InsertPair lands after the ')', Wrap destroys the selection it is
     meant to preserve, and TypeOver is not an edit at all. Read the ledger before designing. -->

## Method note (run 3 of promote-don't-author)
Runs 1 (#336) and 2 (#337) each caught an authorship error before Design — a REQ resting on a deleted seam,
then a **false central premise**. Run 3 continues the streak: the seams here all exist, but the spec's claim
about what they DO is wrong (F1/F2 in the ledger). **A first-pass verification of this spec already existed in
the design-note — written by me while blocked on #337's gate — and Phase 1 treated it as a claim rather than a
fact, correctly: it was imprecise about the IME door and understated the wrap problem.** The method's rule
applies to its own author.

## Title
Typing `(` gives you `(`. Every editor a developer has used in the last decade gives `()` with the caret
inside — plus type-over on the closer, pair-delete on backspace, and wrap-on-type around a selection. Its
absence is felt within seconds, and no ticket for it exists. The entire behavior is **one pure decision
table** applied through the SHIPPED multi-cursor edit engine — this is a small ticket that reads like a big
feature.

## Scope
### In
- **The pure decision seam `auto_close.rs`** (cov/MSI 100 — the truth table IS the mutation surface):
  `pair_action(typed: char, prev: Option<char>, next: Option<char>, has_selection: bool) -> Action` where
  `Action ∈ { InsertPair, TypeOver, Wrap, Insert }`. Total over all inputs (a fuzz row pins no-panic).
  Pairs: `()` `[]` `{}` `""` `''` `` ` `` (backtick pairs with itself, as do quotes).
- **The rules (the table's rows):**
  - *InsertPair* — typing an OPENER with an empty selection when `next` is none/whitespace/a closer (the
    VS Code "autoCloseBefore" posture) → insert both, caret between. Typing an opener directly before a
    word char inserts ONLY the opener (`(` before `foo` must not yield `()foo`).
  - *TypeOver* — typing a CLOSER when `next` is exactly that closer → step the caret past it, insert
    nothing. The adjacency rule (no auto-inserted bookkeeping): positions shift under multi-cursor edits and
    caret anchors are not on the caret path yet, so a position-stack would be stale-prone; plain adjacency
    is what several editors ship and mis-fires only on "type a closer immediately before an identical
    closer" — accepted + documented.
  - *Backspace-pair* — backspace with the caret exactly between an adjacent known pair deletes BOTH (rides
    the existing `backspace_at_selections` path with a widened span).
  - *Wrap* — typing an opener with a NON-EMPTY selection wraps it (`open + sel + close`), the selection
    preserved on the inner text. Per cursor: each of N selections wraps its own.
  - *The `'` lifetime guard* — `'` does NOT pair when `prev` is `&`, `<`, or an identifier char (`&'a`,
    `Foo<'a>`, `impl<'a>`, and the apostrophe in `don't`); it pairs in char-literal positions (prev is
    none/whitespace/opener/`,`/`=`). **AMENDED at Phase 3.5:** this bullet cited `T: 'b` as covered — it is
    not, and cannot be. A bound's `'` follows a SPACE, exactly like `let c = 'x'`, so no one-char rule
    separates them; `<` was added (it is unambiguous) and the bound positions await the grammar (#315).
    A documented Rust-shaped heuristic in the pure table, trivially adjustable when #315 lands languages.
- **N cursors, one undo unit** — ~~every action applies through `edit_at_selections_with` (the #297 engine);
  undo restores all carets exactly (the machinery already guarantees it)~~ **— AMENDED at Phase 1 (F1/F2).**
  The engine does the **TEXT** half only: its post-state is always bare `Selection::caret`s at the END of each
  insert (`selections_after_multi_edit`, selection.rs:253-255). So InsertPair lands after the `)`, **Wrap
  destroys the very selection REQ-005 preserves** (the engine cannot return a range at all), and TypeOver is
  not an edit (it hits the no-op sweep, buffer.rs:396-400). **The SELECTION half is this ticket's own work**,
  and the private `edit_ranges_restoring` (buffer.rs:359) is the only seam that can express it → the work
  likely lands in `crates/editor/src/buffer.rs`, NOT only the app shim. Undo restoring all carets IS
  guaranteed by the machinery (`restore` is recorded separately) — but the caret placement above is also *"the
  precondition, and the ONLY one, under which `coalesces_into` may append a following keystroke"*
  (buffer.rs:428-431), so a careless fix-up breaks typed-run ⌘Z granularity invisibly. See the ledger's
  Fork A/B.
- **`editor.auto_close`** (bool, default **true**) — the #330 four-wiring + the NON-DEFAULT round-trip leg +
  a "Toggle Auto-Close Brackets" palette row. **OFF ⇒ byte-identical to today** (every keystroke inserts
  literally — the OFF-identity discipline).
### Out (explicitly)
- Syntax-aware suppression (no pairing inside strings/comments via tree-sitter) — a #315-era refinement; the
  adjacency + word-char rules cover the common annoyances. Auto-SURROUND on paste. Auto-close of HTML/JSX
  tags. Per-pair enable/disable settings.

## Reference (§20)
**VS Code = OBSERVED behavior** (the pair set, autoCloseBefore, type-over, backspace-pair, wrap-on-type) —
universal editor affordances, no copyleft source read. The decision table is Marley-original.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-PURE-TABLE** — one total `pair_action` fn (**first clause STANDS**); ~~the app shim only maps `Action`
  onto the shipped edit fns~~ **— REOPENED at Phase 1 (F2), RESOLVED at Phase 2 → see D-FIXUP-IN-ENGINE.**
- **D-FIXUP-IN-ENGINE** *(NEW — Phase 2 resolves Forks A+B, which turned out to be ONE question)* — the
  post-selection is computed **inside** `edit_ranges_restoring`, before `end_group`. The shim shape
  (`edit_at_selections_with` then `set_selection`) is **WRONG, not merely untestable**: the undo group STORES
  the post-selection, so a post-hoc correction (a) leaves `sel_after` uncorrected and **redo restores the
  caret after the `)`** (buffer.rs:504-512 restores `sel_after`), and (b) breaks
  `coalesces_into`'s `prev.sel_after == group.sel_before` chain (undo.rs:181) so **the N-cursor typed run
  stops coalescing** — ⌘Z one char at a time, "the very harm the single-cursor rule exists to prevent"
  (undo.rs:150-152). `set_selection` being `pub` is a trap here, not an affordance.
  **Mechanism:** generalize the engine's post-state from a LENGTH to a per-cursor SPAN — `Selection::new(base +
  anchor_rel, base + head_rel)` — where the identity `(repl_len, repl_len)` reproduces today's math exactly
  (property-pinned, the #331 idiom). That is what makes Wrap's preserved selection (REQ-005) expressible at
  all.
  ~~**#338 adds NO undo group** — the engine self-brackets, and a stray `begin_undo_group` would set
  `sel_before = Some(..)`, which `record`'s guard uses to refuse coalescing the NEXT keystroke.~~
  **AMENDED at Phase 3.5 (F2) — half right, and the wrong half was load-bearing.** The reasoning stands for
  ORDINARY typing and #338 adds no group there. But the engine opens no group for a SINGLE cursor, and an
  ungrouped record has `sel_after: None`, so `redo` recomputes the caret as "the end of the insert" — *the
  very placement this decision exists to prevent*, alive at N=1, the common case. So a NON-IDENTITY span at
  one cursor now **does** bracket a group: it is the only place the corrected selection can be kept. It costs
  no coalescing, because `record` already refused to coalesce a 2-char `()` or a Wrap's non-empty `removed`.
- **D-EARN-THE-TAG** *(NEW — Phase 3.5 F1, the ticket's CRITICAL)* — `cursor_anchored` is COMPUTED
  (`spans.iter().all(Option::is_none)`), never asserted. It names the invariant "every cursor ends at the end
  of its own insert", which `coalesces_into` **trusts instead of checking** — and `CaretSpan` is the first
  thing in the editor's history able to falsify it. Hardcoding `true` let the next typed char be appended onto
  the InsertPair record (`inserted = "()x"` where the text reads `(x)`), and **redo replayed it verbatim:
  silent text corruption at N≥2**. Every pre-#338 caller yields `true` and is byte-identical.
- **D-TYPEOVER-IS-NOT-AN-EDIT** *(NEW — Phase 2)* — a plan whose every member is a pure caret move never
  enters the engine (it would hit the no-op sweep at buffer.rs:396-400 and not move at all); it is a
  `set_selection`. This is the honest model: stepping over a closer writes no text, so it must not burn an
  undo record, bump the version, or mark the file dirty. A MIXED set (some TypeOver, some InsertPair) rides
  the engine unmodified — `acts()` skips the non-editing member while `after` still gives it `base + 1`.
- **D-ADJACENCY-NOT-BOOKKEEPING** — type-over and backspace-pair key on adjacency, not an auto-inserted
  position stack (stale-position class avoided; the mis-fire case documented + accepted).
- **D-WRAP-ON-TYPE** — a selection + an opener wraps; this shadows "replace selection with the char" for
  openers ONLY (closers + ordinary chars still replace — the table says so explicitly).
- **D-LIFETIME-GUARD** — `'` pairs positionally (the prev-char rule); named as Rust-shaped.
- **D-OFF-IDENTICAL** — the setting gates the ENTIRE table at the shim; off = the pre-ticket insert path.
- **D-ORDINARY-TYPING-ONLY** *(NEW — Phase 1 F3; the spec never mentioned IME)* — the table fires ONLY on
  `ime::replace_text`'s **branch 1** (`range_utf16.is_none() && marked.is_none()` — ordinary typing at N
  cursors). Branch 2 (a platform-addressed range, or committing a live composition) never pairs. This is not
  a new mechanism: the door already splits there, so **excluding mid-composition pairing is free and
  structural**. Note branch 2 is not exotic — ime.rs:97-99 records that macOS uses `replacementRange` for
  press-and-hold accents and Text Replacement / autocorrect on a stock US keyboard — so this decision has
  everyday reach, not just CJK.
- **D-ONE-CHAR-ONLY** *(NEW — Phase 1 F4)* — `replace_text` takes `text: &str`, not a `char`, while
  `pair_action(typed: char, …)` assumes one. The table fires only when the inserted text is **exactly one
  char**; a multi-char insert (a Text Replacement expansion, a committed composition) is an ordinary insert.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | insert the pair with the caret between, for every opener, when next is none/ws/closer | pure table |
| REQ-002 | insert ONLY the opener when next is a word char | pure table |
| REQ-003 | type over an adjacent identical closer instead of inserting | pure table + headless |
| REQ-004 | delete BOTH chars on backspace between an adjacent known pair | pure + undo round-trip |
| REQ-005 | wrap a non-empty selection on typing an opener, selection preserved inside — at EACH of N cursors | headless (N=3) |
| REQ-006 | not pair `'` after `&` or an identifier char; pair it in char-literal positions | pure table |
| REQ-007 | apply every action at N cursors as ONE undo unit; ⌘Z restores text + all carets exactly | headless |
| REQ-008 | be byte-identical to today's insert path when `editor.auto_close` is false (+ the NON-DEFAULT round-trip leg) | property + settings leg |
| REQ-009 | never panic for any (typed, prev, next, sel) combination — the table is total | fuzz/property row |
| REQ-010 | **(NEW — Phase 1 F3/D-ORDINARY-TYPING-ONLY)** not pair during an IME composition or on a platform-addressed range — only `ime::replace_text` branch 1 pairs | unit on the branch gate |
| REQ-011 | **(NEW — Phase 1 F4/D-ONE-CHAR-ONLY)** not pair when the inserted text is not exactly one char (a Text-Replacement expansion containing `(` inserts literally) | unit |
| REQ-012 | **(NEW — Phase 1 F1b)** preserve typed-run undo COALESCING — typing `foo` at N cursors is still ONE ⌘Z, not three, with auto-close on | headless regression |

## Phase Plan
P2 confirm the exact insert-path seam (where a typed char becomes `edit_at_selections`) + the table's final
rows; P3 the pure table → the shim mapping → the setting + palette row; P3.5 critics on the table's edge
rows (BOL/EOL prev/next = None, the `'` guard, wrap-vs-replace at mixed cursors — some with selections and
some without in ONE keystroke: per-cursor action resolution is the subtle bit); P4 tables + headless N-cursor
drives + gate; P5 docs. Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
