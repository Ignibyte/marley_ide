# find in scrollback (cmd-F) — Notes

- **Forge ticket:** #47 `78f75b06-83b1-4bc1-8e98-d89742cf1434`
- **AAR:** `b78678e8-f4c0-49a2-b943-a9f174581c9a`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-047-find-scrollback.md
- **Pipeline spec:** find-scrollback.spec.md

## Phase 1 — Plan
- **Request:** forge #47 (M1.G "Block Workflows & Selection" seq-5, auto-approved) — cmd-F
  find-in-scrollback.
- **Classification / tier:** work pipeline, `feature`, PURE (find.rs — cov/MSI 100) + a SHIM (find-bar
  overlay + highlight + scroll). marley_app only.
- **Discovery (§18):**
  - The palette (#25) gives the overlay pattern — `palette_open: bool` + a pure `PaletteState` model +
    escape/enter handling (app.rs). The find-bar mirrors it: a `find_open` bool + query + current-index.
  - The viewport (#32) — `visible(content, capacity)` + `scroll_up`/`scroll_down` (no explicit
    scroll_to; the shim scrolls toward the match row via up/down).
  - `find_matches` + `match_navigation` are self-contained (gpui-free) → a new find.rs.
  - Deps #25 (overlay) + #32 (viewport) — done.
- **Decisions:** D1–D4 in the spec (ASCII fold; non-overlap; empty→[]; pure fns + shim overlay).
- **Open questions for Design:** the case-fold (ASCII `to_lowercase` on both — byte-length-preserving
  for ASCII, so ranges valid; note the Unicode caveat); whether a FindState model (like PaletteState)
  vs the two free fns + RootView state — lean the two free fns (simpler, matches the ticket); the
  scroll-to (compute the match's content row → scroll_up/down delta) — shim.
- **AAR id:** `b78678e8-f4c0-49a2-b943-a9f174581c9a`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/find.rs` (NEW, gpui-free)
```rust
use std::ops::Range;

/// Every NON-overlapping, case-insensitive (ASCII fold) match of `query` in `haystack`, as byte
/// ranges (R51). An empty query → no matches. `"aa"` in `"aaa"` → one match `0..2` (advance past
/// each hit). Ranges index the folded haystack — equal to the original for ASCII (byte-length-
/// preserving); a Unicode width-changing fold is a documented limitation.
pub fn find_matches(haystack: &str, query: &str) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    let hay = haystack.to_lowercase();
    let needle = query.to_lowercase();
    let mut matches = Vec::new();
    let mut start = 0;
    while let Some(pos) = hay[start..].find(&needle) {
        let begin = start + pos;
        let end = begin + needle.len();
        matches.push(begin..end);
        start = end; // non-overlapping: resume AFTER this match
    }
    matches
}

/// The next (`forward`) or previous match index, wrapping (R51): `len == 0` → `0`; forward →
/// `(current + 1) % len`; backward → `(current + len - 1) % len` (no underflow at `current == 0`).
pub fn match_navigation(len: usize, current: usize, forward: bool) -> usize {
    if len == 0 {
        return 0;
    }
    if forward {
        (current + 1) % len
    } else {
        (current + len - 1) % len
    }
}
```
(`query.is_empty()` guards the loop: a non-empty query lowercases to a non-empty needle, so `find`
always advances — no infinite loop. `start = end` lands on a char boundary in `hay`, so `hay[start..]`
never panics.)

### SHIM — `crates/marley_app/src/app.rs`
- `RootView` gains `find_open: bool`, `find_query: String`, `find_index: usize`. cmd-F opens the bar;
  while open, keystrokes edit `find_query` (append/backspace), Esc closes.
- The find-bar overlay mirrors the palette (#25): a small input strip showing `find_query`.
- HIGHLIGHT + navigation use BOTH pure fns: per output row, `find_matches(row_text, &find_query)` →
  paint a highlight bg on each matched span (live use of `find_matches`); accumulate `(row_index,
  range)` into a flat match list; Enter → `find_index = match_navigation(list.len(), find_index,
  true)`, Shift-Enter → `(…, false)` → scroll the viewport (#32 `scroll_up`/`scroll_down`) to
  `list[find_index].row`.

### File manifest
- A `crates/marley_app/src/find.rs` — `find_matches` + `match_navigation` + tests.
- M `crates/marley_app/src/lib.rs` — `mod find;`.
- M `crates/marley_app/src/app.rs` — the RootView find state + cmd-F + the overlay + per-row highlight
  + Enter/Shift-Enter cycle-scroll + Esc.
- M `docs/specs/SPEC-app-shell.spec.md` — R51 + Mutation-Targets. CHANGELOG; arch.

### Mutation Targets
- `find_matches` — the `query.is_empty()` guard (empty → []); the `start = end` non-overlap advance
  (a `start = begin + 1` would double-count `"aa"` in `"aaa"`); the fold (`to_lowercase` on both).
- `match_navigation` — the `len == 0` guard (→ 0); the `(current + 1) % len` forward wrap; the
  `(current + len - 1) % len` backward wrap.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `find_matches_non_overlapping` — `"abcabc"`/`"bc"` → `[1..3, 4..6]`; `"aaa"`/`"aa"` → `[0..2]` (ONE); `"aaaa"`/`"aa"` → `[0..2, 2..4]` | unit |
| REQ-001 | `find_matches_case_insensitive` — `"Hello WORLD"`/`"world"` → `[6..11]`; `"ABC"`/`"abc"` → `[0..3]` | unit |
| REQ-002 | `find_matches_empty_and_no_match` — `"abc"`/`""` → `[]`; `"abc"`/`"xyz"` → `[]` | unit |
| REQ-003 | `match_navigation_wraps` — len 3: `(3,0,fwd)=1`, `(3,2,fwd)=0`, `(3,0,back)=2`, `(3,1,back)=0`; `(0,0,fwd)=0` | unit |
| REQ-004 | cmd-F overlay: type → highlight, Enter cycles + scrolls, Esc closes | shim + masked visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs find-bar overlay + input + highlight + scroll (shim exclude; needs a live
window + keyboard).

### Risks / decisions
- D-2.1 ASCII lowercase fold — byte-length-preserving for ASCII, so the returned ranges are valid for
  the original haystack (the common terminal case); a Unicode width-changing fold is the documented
  D1 limit. D-2.2 `start = end` (non-overlap) + the `query.is_empty()` guard together prevent both
  double-counting AND an infinite loop. D-2.3 `match_navigation`'s backward uses `+ len - 1` (not
  `- 1`) to avoid usize underflow at `current == 0`. D-2.4 The shim uses find_matches PER ROW (clean
  per-row highlight spans) + match_navigation over the flat match list — both live, both tested.

## Phase 3 — Implement
- **Built (per manifest):** `find.rs` (NEW, gpui-free) — `find_matches` (empty guard, `to_lowercase`
  fold, `start = end` non-overlap loop) + `match_navigation` (len-0 guard, fwd/back wrap); `lib.rs`
  `mod find;`. `app.rs` — `RootView` gained `find_open`/`find_query`/`find_index` (+ init); a cmd-F
  opener + a `find_open` key interceptor (both before the terminal routing); `handle_find_key`
  (Esc/Enter-cycle-via-match_navigation+scroll/Backspace/printable); `find_match_rows` (per-row
  `find_matches` over `content_row_texts`); `scroll_focused_to_row` (viewport scroll_up/down); the
  render snapshots find_open/find_query, tints matched output rows (`success` @ 0.35), and draws a
  top-right find-bar strip (query + match count). SPEC-app-shell R51 + row 51 + Mutation-Targets;
  CHANGELOG.
- **Deviations:** the highlight is per-ROW (tints a row that CONTAINS a match) rather than per-match-
  SPAN for the first cut — simpler over the color-run spans (find_matches' ranges are computed but the
  paint is whole-row); the exact per-span highlight is a later refinement. The find bar searches the
  Block content rows (not the prompt/alt-screen), as scoped.
- **Verification at this phase:** `cargo check -p marley` 0 err; fmt; clippy `-D warnings` 0; docs 0;
  104 marley lib tests pass. `find_matches` + `match_navigation` are USED (find_match_rows +
  handle_find_key + the render highlight) — live. Unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a verbatim probe of every case + a real scoped cargo-mutants + the gate-script read).
  Verdict: **PASS — ship-ready; the pure core is correct, MSI 100 reachable, no panic/underflow/real-
  input infinite-loop.**
- **Mutants:** 25 on find.rs → 17 caught + 1 TIMEOUT (counted as caught) + 7 unviable / **0 missed**
  with the 4 planned tests → MSI 100.
- **Findings (all LOW):**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | LOW | One mutant (`end = begin + len` → `begin - len`) is a genuine INFINITE LOOP, killed via cargo-mutants TIMEOUT (the gate counts Timeout as caught → MSI stays 100; the shipped code always terminates: non-empty query ⇒ needle len ≥ 1 ⇒ end > begin ⇒ start strictly increases). | ACCEPTED | No code fix (no in-loop guard possible without an uncovered branch breaking cov 100); the gate's timeout-as-caught policy is correct. Optional future gate-infra `--timeout` to bound the slow mutant's wall-clock. |
  | F2 | LOW | The doc said "ASCII fold" but the code used `to_lowercase()` (full Unicode) — self-contradictory; a Unicode width-changing fold would make the returned range misalign with the ORIGINAL haystack (latent — the shim discards the range via `(row, _)` + only `.is_empty()`, so harmless today, but a landmine for a future slicer). | REAL | **FIXED** — switched to `to_ascii_lowercase()` (byte-length-preserving → ranges always valid on the original; non-ASCII case-sensitive, documented acceptable) + corrected the doc. |
  | F3 | LOW | The find-bar COUNT (`find_match_rows` over `content_row_texts`) and the render HIGHLIGHT (over the visible line runs) derive row text from different sources → could disagree cosmetically (e.g. a command-row match counts but only output rows are tinted). | ACCEPTED | Shim-only, cosmetic; the first-cut highlight is per output row. Noted. |
- **Verified CORRECT:** non-overlap (`"aaa"/"aa"` → 1); terminates for all real inputs; NO panic on
  multi-byte (café/naïve/Greek/İ/ẞ) — `start = end` is always a char boundary; `match_navigation` no
  underflow (`+ len - 1` after the `len==0` guard) + stale-cursor-safe (`% len` wraps, `.get()`
  None-safe); both fns live in the shim (find_match_rows + handle_find_key + the render highlight);
  cmd-F returns before the PTY path; clean-room (pure std str::find).

## Phase 4 — Validate
- **Tests added (find.rs):** `find_matches_non_overlapping` ("abcabc"/"bc"→[1..3,4..6]; "aaa"/"aa"→
  [0..2] ONE; "aaaa"/"aa"→[0..2,2..4]; "aaaaa"/"aa"→2); `find_matches_case_insensitive` (both
  directions, ASCII fold); `find_matches_empty_and_no_match` (empty query→[], no match→[], empty
  haystack→[]); `match_navigation_wraps` (fwd/back wrap, single, len-0).
- **Runs (actual):** `cargo nextest run -p marley` → 111 passed (all 4 new PASS); `--workspace` → 537
  passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **18 caught /
  0 missed → MSI 100.0%** (find_matches + match_navigation; the F1 infinite-loop mutant counted as
  caught via timeout). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` — the find bullet.
  SPEC-app-shell R51 at implement.
- **Knowledge captured:** no new failure (inspect's LOW-2 was a same-phase doc/code fix, not a shipped
  bug). aar-submit `completed` (score 5). Win: the critic RAN cargo-mutants + read the gate script to
  prove the one infinite-loop mutant is caught-via-timeout (MSI stays 100) AND that the shipped loop
  always terminates — then the `to_lowercase`→`to_ascii_lowercase` fix removed a latent Unicode-range-
  misalignment landmine (byte-length-preserving fold → ranges valid on the original) that no test
  would have caught (the shim never slices with the range today).
- **Ticket:** forge #47 → done; local doc → closed/; pipeline pair archived. 5 of 6 in M1.G.
