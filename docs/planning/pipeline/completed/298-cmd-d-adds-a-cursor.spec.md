---
pipeline_id: 86ea7fdf-675e-4abb-a779-87e60ec4e10a
ticket: forge#298 (fb7ba9ea-3757-4b7e-a928-57bc5b21a07e) · local docs/planning/tickets/open/TICKET-298-cmd-d-adds-a-cursor.md
aar_id: 3adb9d1d-4e70-4ca3-a0ca-c7d281f19e1e
status: Phase 5 — Complete PASS
title: ⌘D ADDS the next occurrence as a cursor (additive multi-select) + ⌘⇧L selects every occurrence
type: feature
milestone: M19
references: [docs/planning/pipeline/completed/297-multi-cursor-gestures-and-shim.spec.md]
---

## Title
Turn ⌘D from a single-selection *advance* into the universal **additive** multi-select: each press KEEPS the
existing selections and ADDS the next occurrence as a NEW cursor, so repeated ⌘D builds a multi-selection you
type over in one stroke. Plus **⌘⇧L** — every occurrence at once. This is the most-used multi-cursor gesture
in any editor, and #272 already built the search half.

## Scope

### In
- **`add_next_occurrence(set, buffer) -> SelectionSet`** (pure, `crates/editor`) — a bare caret SELECTS the
  word under it (the #272 feel, preserved); a subsequent press finds the next occurrence of the primary's
  text **after the LAST (bottom-most) cursor**, WRAPPING past EOF, and **ADDS** it; a NO-OP once every
  occurrence is already selected.
- **`select_all_occurrences(set, buffer) -> SelectionSet`** (pure) — every match of the primary's text.
- Both **REUSE** #272's `find.rs` (`word_range_at`, `next_occurrence`, `find_all`) — the search is already
  built and tested; do not re-derive it.
- **Keymap**: rebind the Editor-scoped ⌘D from `select-next-match` → `add-next-occurrence`; add an
  Editor-scoped ⌘⇧L row. Dispatch arms for both.
- `find::select_next_match` becomes dead once ⌘D rebinds → delete it (§0). **Verified safe:** its only
  production caller is the ⌘D arm (`app.rs:4700`); the find bar uses `find_all`/`replace_all`.

### Out (explicitly deferred)
- Case-sensitivity toggles and whole-word-only matching (⌘D matches the selected TEXT literally, as it does
  today via `next_occurrence`).
- ⌘K⌘D (skip-this-occurrence-and-add-the-next).
- The N-selection **highlight already renders** — #297 shipped N selection bands. Verify it, don't rebuild it.
- Typing over N selections in ONE undo unit **already works** — #296/#297's `edit_at_selections`. Verify.

## Reference (§20)
**Zed / VS Code ⌘D** = *add-selection-to-next-find-match*: the first press selects the word under the caret,
each further press adds the next occurrence as an additional cursor (wrapping), and once every occurrence is
selected it does nothing. **⌘⇧L** = *select-all-occurrences*. Marley matches the observed BEHAVIOR on its own
`SelectionSet` + the shipped `find.rs` search. Clean-room §20 — observed behavior only; no Zed or VS Code
source read or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## THE FOUNDATION — this ticket was impossible one commit ago
`find_all("aa")` over `"aaaa"` yields the **touching** matches `(0,2)` and `(2,4)`. Under #296's merge rule
(`<=` for everything) they **collapsed into ONE cursor** — ⌘D would have given one cursor where every editor
gives two. #296 flagged this for #298, but the refinement it documented was **catastrophic**:
`sel.start() < cur.end() || sel.is_caret() || cur.is_caret()` — `is_caret()` short-circuits the whole guard to
`true`, so ANY two carets anywhere in the document merge into one giant range.

#297's inspect replaced it with `SelectionSet::overlaps`, which is caret/range-**asymmetric**: `<=` only where
a CARET is involved (two carets at one offset ARE one cursor), `<` where both are RANGES (touching ranges are
two distinct cursors over DISJOINT spans). **Verified against the shipped code before writing this spec:**

```
find_all("aa") over "aaaa" → [(0, 2), (2, 4)]
through from_selections    → 2 cursor(s)   ✓
```

**The merge rule needs no change.** REQ-001 pins it, because it was a live bug one commit ago and nothing else
in this ticket works without it.

## Locked-In Decisions
- **D1 — ⌘⇧L is Editor-SCOPED, shadowing the global `split-right`.** The chord is genuinely taken
  (`keymap.rs:121` → `split-right`, M12.2 #197, with a test asserting it on a terminal). The shipped
  `action_for` precedent (⌘D, ⌘F, ⌘A, and #297's ⌘⌥↑/↓) ranks a context-scoped row above a global one, so an
  Editor-scoped ⌘⇧L wins on the editor while ⌘⇧L still splits panes everywhere else. **Cost, stated plainly:
  while an editor pane is focused, ⌘⇧L no longer splits the pane.** That matches the reference editors and is
  the same trade #297 made for ⌘⌥↑/↓. Rejected: inventing a non-standard chord — muscle memory is the whole
  point of this gesture.
- **D2 — search from the LAST cursor, not the primary.** `select_next_match` searched from the single
  selection's end. With N cursors the next occurrence must be found after the **bottom-most** one, or ⌘D would
  re-add a match already selected and stall.
- **D3 — ⌘D is a NO-OP when every occurrence is selected.** `next_occurrence` WRAPS, so without this guard a
  further ⌘D would re-find an already-selected match; `from_selections` would merge it away, leaving the set
  unchanged but the keypress silently doing nothing *by accident* rather than by decision. Make it explicit —
  and make it a test, because "it happens to work" is exactly how #296's merge bug survived.
- **D4 — a bare caret's first ⌘D REPLACES the set with the word** (one selection), it does not add. That is
  the #272 behavior and the universal one: the first press establishes what you are matching.
- **D5 — reuse `find.rs` wholesale.** `word_range_at` (the caret's word, sharing `movement::is_word_char` so
  ⌘D and ⌥-arrow never disagree), `next_occurrence` (a wrapping scan), `find_all` (every match,
  ASCII-case-folding, non-overlapping). All shipped and tested by #265/#272.

## OPEN DECISION for Phase 2 — ⌘D and ⌘⇧L currently DISAGREE about what an "occurrence" is
Found at PLAN time by reading both matchers, not at the drive:

- **`next_occurrence`** (⌘D's engine) — `rope.char(i + k) == ndl[k]`. **Case-SENSITIVE.**
- **`find_all`** (⌘⇧L's engine) — `c == ndl[k] || c.eq_ignore_ascii_case(&ndl[k])`. **ASCII case-INSENSITIVE.**

Build the two seams naively on those and the **two gestures this ticket ships would behave differently on the
same text**: over `"foo FOO"`, ⌘D selects one occurrence and ⌘⇧L selects two.

**This is a correctness problem, not just a fidelity one.** These gestures exist to be *typed over*. If ⌘D
silently selects `FOO` when the user asked for `foo`, the next keystroke rewrites text they never targeted —
the same class of harm as #297's undo-restores-the-wrong-ranges bug. The reference editors are case-SENSITIVE
for ⌘D/⌘⇧L (the case toggle lives on the find bar, where it belongs).

`find_all`'s folding is **right for its existing caller** — the find bar (#272), which inherited the terminal
find's case-folding precedent — and wrong for these two. So this is not "fix `find_all`"; it is "these two
callers want a different matcher."

Phase 2 decides HOW (a `fold: bool` on `find_all` so the find bar keeps its behavior and ⌘⇧L opts out, versus
a separate exact scan), but **THAT they must agree, and agree on CASE-SENSITIVE, is the decision.** A test
pins it.

## Open question for Phase 2
`select_next_match` composes exactly the two halves the new seam needs, but with the wrong shape (it searches
from THE selection and REPLACES; we need to search from the LAST cursor and ADD). **Delete it, or recompose
it?** Phase 2 decides with the real diff in hand. Deleting is the §0 default once it has no caller; keeping a
`pub` fn that nothing calls is an `#[allow(dead_code)]` in spirit.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | **THE FOUNDATION.** WHEN two occurrences of the search text TOUCH (`find_all("aa")` over `"aaaa"` → `(0,2)`+`(2,4)`), the system shall carry them through `from_selections` as **TWO** cursors, not one. | Pure unit. **Write this FIRST** — it was a live bug one commit ago and everything else rests on it. |
| REQ-002 | WHEN ⌘D is pressed with a bare caret, the system shall SELECT THE WORD under the caret (the #272 feel), yielding exactly one selection. | Pure unit on `add_next_occurrence` + **live drive**. |
| REQ-003 | WHEN ⌘D is pressed with a live selection, the system shall ADD the next occurrence of the primary's text as a NEW cursor — KEEPING the existing ones — searching after the LAST cursor and WRAPPING past EOF. | Pure unit (2nd press → 2 selections; a press that must wrap) + **live drive** (⌘D ⌘D → two highlighted occurrences). |
| REQ-004 | WHEN every occurrence is already selected, a further ⌘D shall be a NO-OP — no duplicate cursor, no unbounded growth. | Pure unit: press ⌘D N+2 times over N occurrences → still N selections. |
| REQ-005 | WHEN ⌘⇧L is pressed on the editor, the system shall select EVERY occurrence of the primary's text at once; and ⌘⇧L shall still split the pane everywhere else. | Pure unit + a keymap RESOLUTION test (Editor → `select-all-occurrences`; Terminal/global → `split-right`) + **live drive**. |
| REQ-006 | WHEN a character is typed over an N-occurrence multi-selection, the system shall replace ALL N in ONE undo unit. | Headless + **live drive**. Rides #296/#297 — verify, don't rebuild. |
| REQ-007 | ⌘D and ⌘⇧L shall agree on what an occurrence IS, and both shall be **case-SENSITIVE**: over `"foo FOO foo"`, both shall match exactly the two lowercase `foo`s. (The find bar keeps its case-folding — that is its own, correct, behavior.) | Pure unit on BOTH seams over the same fixture. **They diverge today** — see the open decision above. |
| REQ-008 | The new pure seams shall be at 100% line coverage and MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — the two seams' signatures + where they live; DELETE-or-recompose `select_next_match`; the
  keymap rows + the roster-guard updates (`all_chords()` counts ROWS not unique chords: +1 chord, +1 scoped);
  the test plan.
- **P3 Implement** — the pure seams, then the keymap + dispatch.
- **P3.5 Inspect** — adversarial critics on the diff. **Spawn them and WAIT.** On #297 five critics found TEN
  real bugs (six HIGH) with the gate FULLY green — clippy clean, 1066 tests, 100% coverage, MSI 100, a
  50k-step fuzzer. A green gate is not evidence. Lenses: the wrap/no-op boundary (an infinite-add or a stall),
  the LAST-cursor search (off-by-one at a match boundary), overlapping/adjacent matches, the case-folding
  asymmetry (`find_all` folds ASCII case; `next_occurrence` does NOT — **is that a real divergence between ⌘D
  and ⌘⇧L?**), and the keymap shadow.
- **P4 Validate** — units + the traced mutant kill set + the gate; then the **LIVE DRIVE**: ⌘D ⌘D → two
  highlighted occurrences → type → both replaced in one undo unit → ⌘⇧L → every occurrence. Capture and READ
  the pixels.
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #298.
