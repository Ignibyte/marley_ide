---
pipeline_id: 48490af7-ab17-4347-88f6-3203359aed20
ticket: forge#186 (9ecc172c-c47d-4e41-aec9-35935ea656c5)
aar_id: 17d0ba55-e30d-4553-a785-7314d23e393f
---

# Notes — M12 #186 scrollback find next/prev + "n of m"

## Phase 1 — Plan / Phase 2 — Design (folded)
**Reuse.** The #51 find bar ALREADY has: find_open/find_query/find_index state, handle_find_key (Enter/⇧Enter →
match_navigation → scroll_focused_to_row), find_match_rows (the global match list), and a find_bg row highlight.
So next/prev + wrap + scroll are DONE. #186 only adds the "n of m" counter + the active-match distinct tint,
plus extracts the pure row-flatten core.

**Pure additions (find.rs).**
- `scrollback_matches(rows, query) -> Vec<(usize, Range<usize>)>`: for (i, row) in rows.enumerate(), for range
  in find_matches(row, query) → push (i, range). The pure core find_match_rows now delegates to.
- `match_label(current, total) -> String`: total==0 → ""; else format!("{} of {}", current+1, total).

**Shim (app.rs).**
- find_match_rows → `scrollback_matches(&content_row_texts(state), &self.find_query)`.
- The find-bar label: replace the bare `(count)` with match_label(self.find_index, count) → "n of m".
- The pane render: before the row loop, `let active_find_row = self.find_match_rows().get(self.find_index).map(|(r,_)| *r);`
  when find_open; in the row highlight, i == active_find_row → an active tint (accent at higher opacity), else
  the existing find_bg. (Per-row active tint; a char-range highlight is out of scope.)

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `scrollback_matches_flattens_rows` | REQ-001 — matches across rows with row indices, in order; empty query/rows → empty; case-insensitive (via find_matches) |
| `match_label_counter` | REQ-002 — total 0 → ""; (0, 3) → "1 of 3"; (2, 3) → "3 of 3" (kills the +1 off-by-one + the ==0 guard) |
| driven | REQ-003 — ⌘F a repeated word → "n of m"; Enter steps + scrolls + the counter increments |
| gate --diff (staged) | REQ-004 |

**Risk.** find_index can momentarily exceed a shrunk match list after an edit (find_index reset to 0 on
edit — safe). The active_find_row lookup uses .get (None-safe). match_label is 1-based display over a 0-based
index — the +1 is the load-bearing mutation target. scrollback_matches ordering: row-major (outer rows, inner
find_matches already in column order) — deterministic.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean. Most of #186 (next/prev/wrap/scroll) was ALREADY done
in #51 — this adds the counter + the active tint + extracts the pure core.
- `find.rs` (PURE) — `scrollback_matches(rows, query)` (flatten find_matches per row with the row index) +
  `match_label(current, total)` ("" at 0, else "{current+1} of {total}").
- `app.rs` (SHIM) — imported both; `find_match_rows` now delegates to `scrollback_matches(&content_row_texts
  (state), &query)` (was inlined); the find-bar label shows `match_label(find_index, count)` → "n of m" (and a
  "(no matches)" arm when count==0); the render captures `active_find_row` (find_match_rows.get(find_index).row)
  once + a brighter `active_find_bg` (success @ 0.6 vs find_bg 0.35), and the row highlight tints the active
  match's row with it (else the existing find_bg for other match rows).

**Deviation:** the active-match highlight stays PER-ROW (matching #51's per-row find_bg) — a per-character
active-range highlight is out of scope (a follow-up). The counter's count==0 case shows "(no matches)" (clearer
than an empty "()").
## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/counter-semantics/mutation + render/row-alignment). Both traced
the render `row` counter against content_row_texts (fold-aware, in lockstep — the highlight lands on the right
row) and confirmed the counter has NO off-by-one (label/highlight/scroll all key off the same 0-based find_index;
+1 is display-only). Findings + fixes:

- **[MED] The active tint bled to non-focused panes.** `active_find_row == Some(row)` is a bare index compare;
  each pane restarts `row=0`, so a non-focused pane's same-index row got the bright active tint (no match there).
  REAL, new in #186 (render critic). FIXED: gated BOTH find highlights (header + output) on `is_focused` —
  find operates on the focused pane, so scoping the highlight to it is the correct fix (also tidies #51's
  all-panes find_bg). → prevention rule (an index-only highlight over a repeated-index render needs the owner guard).
- **[MED] A match in a COMMAND (header) row never highlighted** — the highlight was output-only, so a command
  match advanced the counter + scrolled but nothing lit up, breaking #186's "see where you are" promise. REAL
  (render critic; pre-existing for find_bg, but #186's active highlight makes it a real gap). FIXED: added the
  find highlight (active + normal) to the header render, focused-pane-gated, drawn after selection/hover.
- **[LOW] Counter could show "n of m" with n > m** — a fold shrinks `count` without re-clamping find_index.
  REAL (both critics; a fold-chevron click while find is open). FIXED: `match_label(find_index.min(count-1),
  count)` (the else-branch guarantees count>0).
- **[LOW] active_find_row scanned every frame even while find closed** (Esc keeps find_query). FIXED: gated the
  scan on `find_open`.
- **Mutation corrections (both critics, from `cargo mutants --list`):** the `current + 1` is INSIDE a `format!`
  macro → cargo-mutants does NOT mutate it (macro-shielded), so there's no +1 mutant — the earlier "kill the +1"
  premise was wrong. match_label has 3 live mutants (→String::new/→"xyzzy"/==→!=), all killed by
  match_label(0,3)=="1 of 3" + match_label(0,0)=="". scrollback_matches has only the `vec![]` body mutant viable
  (the Range-typed candidates don't compile → unviable); killed by a ≥2-row fixture asserting the exact (row,
  range) list. Recorded verbatim for Phase 4.
- Counter semantics, faithful extraction (byte-for-byte the old inlined loop), fold/row alignment, panics,
  selection-vs-find order — all VERIFIED CLEAN by both.

Lenses: counter off-by-one, mutation-killability (+ the format!-shield correction), row-index alignment (fold +
header + all-panes), focused-pane scoping, panics, faithful-extraction, perf. Post-fix: `cargo check --workspace` clean.
## Phase 4 — Validate
**Tests** (find.rs): scrollback_matches_flattens_rows (the critics' ≥2-row fixture — both matches in row 0 carry
index 0, proving the enumerate/row index not a match counter; row 2 after the skipped no-match row 1; empty
rows/query → empty; case-insensitive); match_label_counter ((0,3)→"1 of 3", (2,3)→"3 of 3", (0,0)→"", (0,1)→"1
of 1"). `cargo mutants -f find.rs` → 0 missed (match_label's 3 live mutants killed by the (0,3)+(0,0) asserts;
scrollback_matches' vec![] killed by the ≥2-row fixture; the Range-typed candidates unviable; the 2 timeouts are
the pre-existing find_matches loop, counted caught). **cargo nextest run --workspace**: 762 passed.

**Driven live-app capture (REQ-003)** — minted 4 "marley" blocks (a printf with 3 "marley" in the command + 3
output rows, and 3 `echo marley` blocks). ⌘F "marley" (/tmp/mly186_find3.png) showed DEFINITIVELY:
- the find bar counter **"marley (1 of 12)"** — the "n of m" counter renders (12 = 3 printf-cmd + 3 printf-out +
  3 echo-cmd + 3 echo-out);
- the **printf command HEADER row tinted BRIGHTER** (active_find_bg) than the other matches — proving BOTH the
  active-match distinct highlight AND the inspect header-match fix (command-row matches now light up);
- all 12 match rows tinted (find_bg).
The Enter-INCREMENT step (counter n → n+1, viewport scroll) could not be cleanly re-captured — the GUI harness
can't hold continuous key focus across the ⌘F → type → Enter modal sequence (a documented modal-text-field
flakiness; the query kept accumulating to "marleymarley"). It is NOT a code gap: the next/prev navigation +
scroll is the pre-existing #51 path (already validated there), and the counter it drives is the unit-tested
match_label(match_navigation(...)) — match_navigation_wraps + match_label_counter pin the index + label math.
Stated explicitly per the validate skill (harness limit, not a silent skip).

**Gate**: `scripts/gates.sh --diff` (staged) → **GATE GREEN [diff]**, 15/15. coverage 100%; mutation 4/4 → MSI 100.0%.

**Pre-existing exclusions**: none. (Restored ~/.marley/config/settings.toml from the pre-test backup.)
## Phase 5 — Complete
(pending)
