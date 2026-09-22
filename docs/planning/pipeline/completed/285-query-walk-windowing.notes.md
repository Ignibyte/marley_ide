# Window the highlight query walk to the damage span — Notes

- **Forge ticket:** #285 `e6210a61-79e6-4e65-90fa-2748332fa89e`
- **AAR:** `435358c3-3dd6-4467-b0b1-16739213870b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-285-query-walk-windowing.md
- **Pipeline spec:** 285-query-walk-windowing.spec.md

## Phase 1 — Plan
- **Request:** Window the tree-sitter highlight query walk to the edit's damage
  span (forge #285, follow-up from #274 inspect TS-F2). After the cheap
  incremental re-parse, run `QueryCursor::set_byte_range` over the merged damage
  window instead of the whole tree, and splice the fresh in-window spans into the
  cached out-of-window spans (rebased by the edit delta). Must stay byte-identical
  to a fresh full highlight on the #274 corpus; tighten the e2e pin to <1/3.
- **Classification / tier:** work pipeline, one shippable slice. Single crate
  (`marley_syntax`): `parse.rs` (windowed query pass) + `lib.rs`
  (`HighlightSession` cache + `highlight_incremental` rewire) + two new pure fns.
  No app/UI surface — library crate, `marley_visual_harness` N/A; the equivalence
  corpus + perf pin are the acceptance.
- **Forge recall (§18.3):**
  - Ticket #285 carries the MEASURED decomposition (release, 8k lines / 27k
    captures): re-parse 1.0ms vs fresh 9.2ms; the full query WALK is 5.7ms every
    call; the `span→lines` tail allocates O(lines) (~0.3ms by back-solving the
    0.46 floor). Removing the 5.7ms walk is the whole win.
  - knowledge-search (tree-sitter/incremental/windowing) surfaced the syntax-crate
    prevention rules from #274/#269 — span rebasing conventions (#269), the sweep's
    disjoint-clip, the corpus-as-gate. Fed into the design watch-list.
  - #274 lesson carried: "measure a perf pin's DECOMPOSITION before writing it" —
    already have the decomposition from the ticket, so the pin target (<1/3) is
    grounded, not guessed.
- **Discovery (Design surface):**
  - `crates/syntax/src/parse.rs` — `spans_from_tree(tree, src)` is the full
    `QueryCursor::new().matches(query, root, bytes)` walk = the O(file) target.
    Add a windowed sibling that sets `set_byte_range(window)` before `.matches`.
  - `crates/syntax/src/lib.rs` — `highlight_incremental` (235-283) does the
    validity check → `tree.edit(InputEdit)` → `parse(new, Some(old))` → then the
    full `spans_from_tree` (5.7ms) → `lines_from_spans`. Rewire the tail: clone the
    edited old tree BEFORE parse, `old.changed_ranges(&new)`, `damage_window`,
    windowed pass, `splice_spans`, then the UNCHANGED `lines_from_spans` on the
    spliced full set (D1). `HighlightSession` gains `last_spans: Vec<Span>`.
  - The equivalence corpus `t274_incremental_equals_full_corpus` (lib.rs ~526) and
    the perf pin `t274_perf_ratio_pins` (~623) are the gate; expand the corpus with
    window-boundary/straddle cases, tighten the pin `/4→? ` … actually the pin is
    `inc_e2e < full_e2e * 3/4` today → tighten to `< full_e2e / 3` (REQ-004).
- **Decisions:** D1–D5 in the spec. Crux: splice at the SPAN level and keep
  `lines_from_spans` on the full spliced set — identical span multiset ⇒ identical
  lines, so equivalence risk lives entirely in the splice, which the corpus gates
  hard. `set_byte_range` = INTERSECTING (D4) is the load-bearing tree-sitter
  contract to verify in Design/Implement.
- **Risk:** the load-bearing assumption is `set_byte_range` returns matches whose
  captured node INTERSECTS the window (so straddlers come back with full extent).
  If it instead returned only CONTAINED matches, a straddler would be truncated →
  the corpus catches it (GATE RED), not a silent ship. So the risk is EFFORT
  (iteration on the corpus), not correctness.

## Phase 2 — Design

### Approach (span-splice over the raw capture set)
After the cheap incremental re-parse, replace the full `spans_from_tree` walk with:
a **windowed** query over the edit's damage, spliced into the **cached raw spans**
from the previous highlight (rebased by the edit delta). `lines_from_spans` runs
UNCHANGED on the spliced full raw-span set (D1) — identical span multiset ⇒
identical per-line output, so all equivalence risk lives in the span splice, which
the #274 corpus gates hard.

**Two verified tree-sitter contracts** (checked against the pinned tree-sitter
0.26.11 source): (1) `Tree::changed_ranges(&old_edited, &new)` returns the ranges
(new coords) whose structure changed — call after `edit`+`parse`; (2)
`QueryCursor::set_byte_range(r)` restricts the walk to matches INTERSECTING `r`
(the viewport-highlight primitive editors rely on). Neither is load-bearing for
SAFETY — a wrong assumption fails the corpus (GATE RED), never ships wrong colors.

**The grow-to-fixpoint loop** makes correctness independent of exactly how
tree-sitter clips straddling matches. After a windowed query, extend the window to
the fresh spans' own extent (`window_extent`) and RE-QUERY; repeat until the window
is stable. At the fixpoint every fresh span lies fully inside the window (no
straddle) AND fresh = every capture intersecting that window (complete), so the
partition "fresh inside ∪ cached-kept outside" tiles the file with NO gap and NO
double at the boundary. Converges monotonically (bounded by `[0, len]`; a
block-comment cascade grows the window to ~full in ≤2 iters → the walk degrades to
the #274 full walk, still exact). Typical single-char edit: 1 iteration, tiny
window.

**The splice** drops any cached span TOUCHING the replaced region `[start,old_end)`
(stale) and any cached span OVERLAPPING the window (resupplied by fresh); the
survivors — entirely before `start` (identity) or entirely at/after `old_end`
(shifted by `new_end-old_end`) AND entirely outside the window — are genuinely
unchanged content, so their rebase is unambiguous (no clamp/empty-span fiddliness).

**Perf.** Removes the 5.7ms C query walk (the dominant term), leaving reparse
(~1.0ms C) + a tiny windowed query + O(cached) integer partition (~0.1ms) + the
unchanged `lines_from_spans` (~0.3ms). Est. ratio ≈ 0.22 in the debug test build
(the C walk is not debug-inflated, so removing it drops the ratio hard) — under the
<1/3 pin. The O(cached) splice is still O(file) in span COUNT; true O(damage) via
per-line-vector reuse is the deferred follow-up (spec Out) IF the pin needs it.

§20 confirmed: Zed editor incremental highlight — behavior matched (windowed
result byte-identical to a full re-highlight), tree-sitter public API only,
clean-room. Splice/window arithmetic is Marley's own.

### File manifest
| File | Change |
|------|--------|
| `crates/syntax/src/parse.rs` | ADD `spans_in_window(tree, src, window: Range<usize>)` — `spans_from_tree` with `cursor.set_byte_range(window)` before `.matches` (FFI-adjacent sibling; same coverage-excluded boundary). |
| `crates/syntax/src/lib.rs` | ADD pure `damage_window(changed, edit) -> Range` (min-start/max-end over changed ∪ edit); pure `window_extent(window, spans) -> Range` (grow to cover spans); pure `splice_spans(cached_old, edit:(start,old_end,new_end), window, fresh) -> Vec` (drop touching-edit + overlapping-window, rebase+keep the rest, union fresh). ADD `HighlightSession.last_spans: Vec<(usize,usize,TokenKind)>` (seed in `new()`; set in `highlight_full`). REWIRE `highlight_incremental`: clone the edited old tree, `changed_ranges`, the grow-loop, `splice_spans`, cache `last_spans`, then the UNCHANGED `lines_from_spans`. |

### Regression Test Plan
| # | Test (lib.rs `#[cfg(test)]` unless noted) | Proves |
|---|---|---|
| T1 | `damage_window`: empty `changed` → edit span; `changed` straddling both sides → union min/max; `changed` inside edit → edit bounds. | REQ-002 window merge |
| T2 | `window_extent`: no spans → identity; a span ending past `window.end` → grows end; a span starting before `window.start` → grows start; fully-inside spans → unchanged. | REQ-002 grow |
| T3 | `splice_spans` truth-table: before-span kept as identity; after-span shifted by delta (insert AND delete); straddler of `[start,old_end)` dropped; after-span overlapping the window dropped; boundary `e==window.start` kept; boundary `s==window.end` kept. | REQ-002 splice |
| T4 | EXPAND `t274_incremental_equals_full_corpus`: + edit at a token boundary; + edit INSIDE a multi-line block comment (straddle → grow-loop); + edit growing an identifier across a boundary; + edit near a multi-capture construct. Assert `inc == highlight_lines(new)` per line. | REQ-001 equivalence |
| T5 | Corpus cascade rows retained/added: block-comment open at BOF (`/*` at byte 0) → `changed_ranges` ~ whole file → window grows to full → equal; the 10-step chain. | REQ-003 cascade |
| T6 | `spans_in_window` sanity: a window over one token returns that token (intersecting) incl. a straddler with FULL extent; an empty-overlap window returns none. | REQ-002 windowed query behavior (pins the excluded parse.rs seam) |
| T7 | Tighten `t274_perf_ratio_pins`: `inc_e2e < full_e2e / 3` (was `* 3/4`). | REQ-004 perf |
| — | cov/MSI 100 on `damage_window`/`window_extent`/`splice_spans` via `scripts/gates.sh --diff`. | REQ-005 |

Uncoverable: none new. `spans_in_window` is FFI-adjacent (parse.rs coverage-excluded like `spans_from_tree`) but MUTATION-covered — pinned by T4/T5/T6 end-to-end.

### Risks / decisions
- **R1 (load-bearing, mitigated):** `set_byte_range` = intersecting. If it were
  contained-only, a straddler would be missed → the grow-loop wouldn't see it →
  corpus RED (not a silent wrong ship). Verified intersecting in the 0.26.11 doc +
  it's the standard viewport-highlight primitive. The grow-loop + corpus are the
  net.
- **R2:** `changed_ranges` must be a superset of every span-kind change. It is
  (tree-sitter's contract); a KIND change without a byte change still changes the
  node → reported. Corpus T4/T5 probe it.
- **D-perf:** ship span-splice (meets <1/3); defer per-line-vector reuse (true
  O(damage)) unless the pin comes RED in Validate — already the spec's Out/D1.

## Phase 3 — Implement
- **Built to the manifest, no deviations.**
  - `parse.rs`: `spans_in_window(tree, src, window)` — `spans_from_tree` + `cursor.set_byte_range(window)`.
  - `lib.rs`: pure `damage_window`, `window_extent`, `splice_spans` (exactly the Phase-2 signatures); `HighlightSession.last_spans` field (seeded in `new()`, set in `highlight_full`); `highlight_incremental` rewired — clone the edited old tree, `changed_ranges`, `damage_window`, the grow-loop (`window_extent` + re-query to fixpoint), `splice_spans`, cache `last_spans`, then the UNCHANGED `lines_from_spans`.
  - Kept the `map_or_else(Vec::new, …)` shape (over a LOCAL `new_tree`, splicing against a `let last_spans = &self.last_spans` reborrow) so the dead None-parse arm stays region-attributed like #274 — no bare uncovered match arm; disjoint borrow is clean.
- **Early de-risk (ran EXISTING tests, none modified):** all 17 `marley_syntax` tests PASS — crucially `t274_incremental_equals_full_corpus` (byte-identical to the full walk across the 7 cases + the 10-step chain, INCLUDING the block-comment cascade) and `t274_splice_lies…fall_back_to_full` (the validity fallback). The equivalence approach is proven before inspect. `t274_perf_ratio_pins` still green at the `*3/4` bound (0.66s) — Validate tightens it to `/3`.
- `cargo check --workspace` clean; `cargo fmt --check` clean; `cargo clippy -p marley_syntax --all-targets` clean (gate uses `-D warnings`).

## Phase 3.5 — Inspect
One background sonnet critic (correctness/equivalence lens) — stalled at 156B (the
known signature; 4th stall this sprint). Primary = a thorough INLINE adversarial
trace over five attack angles. **No real defects found.** Lenses/angles covered:

| Angle | Verdict | Evidence |
|---|---|---|
| **Multi-capture match straddle** (a match intersecting the window yields ALL its captures incl. distant out-of-window ones → double vs cached-kept) | **SAFE** (structural + net) | Every KEPT-kind pattern in tree-sitter-rust `highlights.scm` (`@string`/`@constant.builtin`/`@comment`/`@keyword`) is SINGLE-capture. The only distant-multi-capture patterns (`type_arguments "<"…">"`, macro `"!"`) map to `@punctuation.bracket`/`@function.macro` → NOT in `kind_of_capture` → `Plain` → dropped in `spans_in_window`. So no distant capture ever reaches an output span. Second net: even if it did, the grow-loop pulls any out-of-window fresh span into the window (`window_extent` grows to its extent) → cached twin dropped → single. |
| **Deletion rebase** (`new_end < old_end`) | **SAFE** | `splice_spans` shift `s - old_end + new_end` runs only for `s >= old_end`, so `s - old_end >= 0` in usize (no underflow); left-shift by `old_end-new_end` is correct. |
| **Boundary off-by-one** (`e <= start`, `s >= old_end`, `ne <= window.start`, `ns >= window.end`) | **SAFE** | Traced each: a cached token entirely before `changed.start` is byte-identical in the new tree (kept); a token crossing `changed.start` has changed content (dropped + resupplied by fresh/grow-loop). The load-bearing invariant is `changed_ranges` marking the FIRST byte of difference — so kept survivors are genuinely unchanged, no gap/double at either edge. |
| **`changed_ranges` coord space** | **SAFE** | `tree.edit()` is applied BEFORE the clone, so `old_edited`'s ranges "match up to the new tree" (tree-sitter 0.26.11 doc) → `changed_ranges` returns POST-edit coords, consistent with the edit-span (`start..new_end`) and `fresh` (queried on `new_src`). |
| **Grow-loop convergence/cost** | **SAFE** | `window_extent` is monotonic (only grows), bounded by `[0,len]`; empirically `t274_perf_ratio_pins` ran 0.66s on the 8k-line fixture (no blow-up). Cascade → grows to ~full in ≤2 steps → the #274 walk. |

Also checked (all clean): the `&self.last_spans` reborrow vs the later `self.last_spans = …` (NLL ends the borrow after `map_or_else`; `cargo check` green); first-call seeding (`new()` → `fits` fails on `tree=None` → `highlight_full` seeds both `tree`+`last_spans` as a pair; they are ALWAYS set together, so never desynced); empty-file / delete-all edit (window `[0,0)`, all cached dropped, `fresh` empty → `[empty line]`, correct); inverted-range impossibility (`edit.start <= edit.new_end` always, so `damage_window` yields `lo<=hi`).

**One accepted observation (not a defect):** `spans.clone()` in both `highlight_full` and `highlight_incremental` — necessary because `lines_from_spans` CONSUMES the Vec and the cache needs a copy; a raw span-vec clone is cheap vs the 5.7ms walk it saves. Left as-is.

No `failure-record` (no bug found). The durable equivalence invariant ("windowed splice ≡ full walk iff window ⊇ changed_ranges ∧ grow-loop absorbs straddlers ∧ distant-multi-captures are output-dropped") → captured as a prevention rule in Phase 5.

## Phase 4 — Validate
- **Tests added (all in `crates/syntax/src/lib.rs`):** `t285_damage_window_merges_changed_and_edit` (T1), `t285_window_extent_grows_to_cover_overhang` (T2), `t285_splice_spans_rebase_drop_keep` (T3 — two cases: window==edit-span boundary keeps; window-past-edit drops), `t285_spans_in_window_intersects_with_full_extent` (T6 — the straddling-comment full-extent + empty-window contracts). **Corpus (T4/T5):** expanded `t274_incremental_equals_full_corpus` with 4 windowing cases — edit inside a multi-line block comment (straddle → grow-loop), a multi-byte deletion shifting spans left (rebase `s-old_end+new_end`), a string-boundary insert, a macro-adjacent edit (distant-multi-capture, Plain-dropped). **Perf (T7):** tightened `t274_perf_ratio_pins` end-to-end bound `full/3` (was `full*3/4`).
- **`cargo nextest run -p marley_syntax`: 21 passed, 0 skipped** — incl. the tightened perf pin (0.667s, comfortably under `/3` — the removed 5.7ms C walk is the win) and the expanded corpus (all windowing cases byte-identical to a fresh full highlight).
- **Mutation note (recurring lesson confirmed via `cargo mutants --list`):** `.min()`/`.max()` are METHOD calls → UNMUTATED, so `damage_window`/`window_extent` carry only fn-replacement mutants, and every `Range::new()`/`Range::from_iter`/`Default::default()` variant is UNVIABLE (no such API / `TokenKind` has no `Default` derive). The viable set is `splice_spans`'s operator mutants (179 `<=`, 181 `>=`, 182 rebase `±`, 186 `||`/`<=`/`>=`) — all killed by T3's two boundary cases — plus `spans_in_window`'s `!=`→`==` (85), killed by T6. Gate confirms MSI 100.
- **Live-app drive/capture: N/A.** `marley_syntax` is a LIBRARY crate with no app shim / render / input surface; REQ-001 makes the windowed output BYTE-IDENTICAL to the pre-existing full walk (zero visual delta by construction), so there is nothing new to render. gate:15 visual/AX passed via the library skip-clean.
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15: rustfmt · clippy -D · tests · audit · deny · machete · gitleaks · shellcheck · no-suppressions · source-bans · docs+brand-scrub · **coverage ≥100% lines** · **mutation MSI ≥100%** · miri · visual/AX. One clippy false-positive (`single_range_in_vec_init` on a single-element `&[Range]` test arg) fixed at source by using 2-element arrays (unambiguous intent, same coverage) — no suppression (§0).
- No pre-existing failures encountered.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Changed entry (#285); docs/marley_architecture/editor.md `marley_syntax` section updated — the #274 "0.46 floor / windowing is the follow-up" note replaced with the shipped windowing (damage_window → grow-loop → splice, byte-identical, O(damage), 3/4→1/3 pin) + the true-O(damage) line-cache as the recorded next step.
- **Knowledge (forge):**
  - AD `f62331c9` `AD-claude-highlight-query-windowing-001` — span-splice over line-cache (concentrate equivalence risk in the splice; removing the 5.7ms C walk alone clears <1/3).
  - PR `c0e30a1e` `PR-claude-windowed-query-splice-equivalence-001` (high) — the windowed-splice equivalence invariant (window ⊇ changed_ranges∪edit + grow-loop-to-fixpoint absorbs straddlers/multi-capture + drop-touching/overlapping + rebase; corpus with a cascade is the gate).
  - AAR `435358c3` submitted: completed, effectiveness 5. No failure-record (inspect found no defects). Reinforced (not re-recorded) the method-calls-unmutated + fn-replacement-unviable lesson: `.min()`/`.max()` unmutated and `Range::new()`/`Default::default()` variants unviable → `damage_window`/`window_extent` have ZERO viable mutants (coverage-gated only); the viable set is `splice_spans` operators + `spans_in_window`'s `!=`, all killed by T3/T6.
- **Ticket** TICKET-285 → closed/ (status closed) + forge ticket-close. **Pipeline** spec+notes → pipeline/completed/.
- **Result:** windowing shipped byte-identical to the full walk (the #274 corpus + 4 new windowing cases green), end-to-end pin tightened 3/4→1/3, pure seams cov/MSI 100, GATE GREEN [diff]. LOCAL commit only (push un-OK'd).
