---
pipeline_id: 439a7d8f-f8a2-433e-badd-316511052ebf
ticket: forge#47 (78f75b06-83b1-4bc1-8e98-d89742cf1434) · local docs/planning/tickets/open/TICKET-047-find-scrollback.md
aar_id: b78678e8-f4c0-49a2-b943-a9f174581c9a
status: Phase 5 — Complete PASS
title: find in scrollback (cmd-F)
type: feature
milestone: M1.G
references:
  - crates/marley_app/src/find.rs (NEW — find_matches + match_navigation)
  - crates/marley_app/src/app.rs (the find-bar overlay + highlight + scroll — shim)
  - docs/specs/SPEC-app-shell.spec.md (R51)
---

## Title
cmd-F opens a find bar that searches the block output, highlights the matches, and scrolls to them
(Enter/Shift-Enter cycle next/prev). The everyday "where did that error scroll to" search.

## Scope
### In
- `crates/marley_app/src/find.rs` (NEW, gpui-free PURE — cov/MSI 100):
  - `find_matches(haystack: &str, query: &str) -> Vec<Range<usize>>` — all NON-overlapping matches of
    `query` in `haystack`, case-insensitive (ASCII lowercase fold); an EMPTY query → an empty vec
    (NOT a match at every position). The advance is `start = match_end`, so `"aa"` in `"aaa"` yields
    ONE match `0..2`, not two overlapping. Byte ranges into the (folded) haystack — valid for ASCII;
    a Unicode width-changing fold is a documented limitation.
  - `match_navigation(len: usize, current: usize, forward: bool) -> usize` — wrap-around next/prev:
    `len == 0` → `0`; `forward` → `(current + 1) % len`; else `(current + len - 1) % len`.
- `crates/marley_app/src/lib.rs` — `mod find;`.
- `crates/marley_app/src/app.rs` (SHIM): a find-bar overlay mirroring the palette (#25) pattern — a
  `find_open` bool + the query string + the current match index on `RootView`. cmd-F opens it; on a
  query change → `find_matches` over the focused pane's block output text; the render paints a
  highlight bg on matched runs; Enter → `match_navigation(len, cur, true)`, Shift-Enter →
  `(…, false)` → scroll the viewport (#32 `scroll_up`/`scroll_down`) toward the match's row; Esc closes.
- SPEC-app-shell (R51, the find clause + Mutation-Targets). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Regex / whole-word / case-sensitive toggle — later (the first cut is a plain case-insensitive
  substring). Find-and-REPLACE. Searching the alt-screen grid (the first cut searches the Block
  output). A match COUNT/position indicator ("3 of 12") beyond the highlight — a later polish.
  Incremental scroll-follow while typing (scroll happens on Enter).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Case-insensitive by `to_ascii_lowercase` (byte-length-preserving), so the returned byte ranges
  are ALWAYS valid offsets into the ORIGINAL haystack (no Unicode-width-changing-fold landmine — the
  inspect LOW-2 fix). Non-ASCII letters are matched case-sensitively; acceptable for terminal output,
  which is overwhelmingly ASCII.
- D2 — NON-overlapping: `start = match_end` after each hit, so `"aa"` in `"aaa"` is one match — the
  standard find behavior (no double-counting).
- D3 — An empty query returns NO matches (a no-op find bar), never a match at every position.
- D4 — PURE: `find_matches` + `match_navigation` (cov/MSI 100). The overlay + input + highlight +
  scroll-to are SHIM (app.rs, masked), mirroring the palette overlay (#25) + viewport (#32).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `find_matches(haystack, query)` is called with a non-empty query, it shall return every NON-overlapping, case-insensitive match as a byte range; `"aa"` in `"aaa"` shall yield exactly one match `0..2`. | unit (multi/overlap/case) |
| REQ-002 | WHEN the query is empty, `find_matches` shall return an empty vec (not a match at every position). | unit (empty query → []) |
| REQ-003 | WHEN `match_navigation(len, current, forward)` is called, it shall return the next (`forward`) or previous index with wrap-around; `len == 0` → `0`. | unit (fwd wrap; back wrap; len 0) |
| REQ-004 | WHEN cmd-F is pressed, the app shall open a find bar; typing a query highlights the matches; Enter/Shift-Enter cycle through them and scroll to the current one; Esc closes. | shim + masked visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `find_matches` + `match_navigation`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the `find.rs` shapes (find_matches loop + match_navigation), the find-bar overlay
  shim (mirror the palette), the SPEC clause + mutation targets.
- **P3 Implement** — find.rs + mod + the app.rs find-bar overlay/highlight/scroll + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the non-overlap advance (no double-count / no infinite loop on empty
  match), the case-fold correctness, the wrap arithmetic (fwd/back at the ends), the empty-query
  guard, the byte-range validity, no palette/viewport regression.
- **P4 Validate** — the find_matches/match_navigation unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #47.
