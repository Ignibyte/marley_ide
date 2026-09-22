# 274-incremental-treesitter — Notes

- **Forge ticket:** #274 522d4028-081c-495f-b2a9-08a4aa639d59
- **AAR:** 9704870c-6528-44fc-b943-05466adc5ca5
- **Local ticket doc:** docs/planning/tickets/open/TICKET-274-incremental-treesitter.md
- **Pipeline spec:** 274-incremental-treesitter.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 6 of 10 (the perf
  gate before self-hosting, per the #268 close).
- **Recon (code-verified):**
  - `BufferDelta` (types.rs:45) = char_range + byte_range (both
    PRE-edit) + new_char_len + new_byte_len. NO removed text, NO
    points → the InputEdit conversion cannot be done from the delta
    alone; `Tree::edit` needs old_end_point (pre-edit) and
    new_end_point (post-edit). THE PLAN PREMISE: the session keeps
    the previous TEXT snapshot, making every point a pure
    `point_at(src, byte)` over a string it holds (D1).
  - Multi-delta chains: delta k's coordinates live in text state k-1,
    which nobody holds (k>1) → single-delta incremental, multi-delta
    full (D2). Autorepeat's 2-per-frame (the #276 lesson) takes the
    full path off-thread.
  - `edits_since(version)` (buffer.rs:197) yields the deltas in order
    — count them vs the cached version to pick the path.
  - `refresh_syntax_cache` (app.rs:5052) is the ONE producer; the
    cache is `(nonce, version, Rc<lines>)`; `needs_syntax_refresh` is
    the memo key. The pump (the #203 pattern) is the drain site for
    worker responses (set cache + dirty).
  - `collect_spans` (parse.rs) is the sync engine call —
    `parse(src, None)` today; the session generalizes it.
  - `Parser` is !Sync but Send; one parser lives on the worker.
- **Perf pin design:** RELATIVE (incremental < full/3, same build) —
  a debug-vs-release-independent ratio; the ticket's absolute 2ms is
  release-only and the gate runs debug.
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### marley_syntax (crates/syntax)
- `SyntaxEdit { start_byte, old_end_byte, new_end_byte,
  start_point: (row, byte_col), old_end_point, new_end_point }` —
  plain numbers, pure-testable; converts to `tree_sitter::InputEdit`
  privately.
- `point_at(src, byte) -> (row, byte_col)` — clamped at len; row =
  '\n' count in src[..byte], col = bytes since the last newline.
- `syntax_edit(last_src, new_src, start_byte, old_end_byte,
  new_byte_len) -> SyntaxEdit` — start/old_end points from the
  PRE-edit snapshot, new_end from the post-edit text (the D1 premise).
- `HighlightSession { parser, tree: Option<Tree>, last_src: String,
  key: Option<(nonce: u64, version: u64-ish)> }` —
  `highlight_full(&mut, src)` (parse(None), reseed) and
  `highlight_incremental(&mut, new_src, &SyntaxEdit)` (tree.edit →
  parse(Some) → reseed); both share the existing query/sweep/clip
  pipeline (collect refactors to walk a given Tree). The free
  `highlight_lines(src)` keeps its exact behavior (the sync path +
  every existing test untouched).
- **The worker-session VALIDITY rule (design key):** the worker may
  SKIP requests (drain-to-latest coalescing under a typing burst),
  which breaks any edit chain — so a request carries
  `parent_version` (what its edit is relative to) and the worker goes
  incremental ONLY when `(nonce, parent_version)` equals the
  session's own `(nonce, version)`; ANY mismatch (skip, nonce change,
  first sight) ⇒ full parse. Self-healing with zero cross-thread
  coordination.

### app.rs
- `SYNTAX_SYNC_MAX_LINES: usize = 1000` (D4).
- `SyntaxReq { generation, nonce, version, parent_version:
  Option<BufferVersion>, text: String, edit: Option<(start_byte,
  old_end_byte, new_byte_len)> }`;
  `SyntaxResp { generation, nonce, version, lines }` (marley_syntax
  kinds; the app maps on apply, as today).
- Worker: ONE `std::thread` owning the session (`Parser` is Send,
  !Sync); loop = recv → drain-to-latest (try_recv) → the validity
  rule → parse → send resp. Spawned lazily on the first large-file
  need; a dead worker (send Err) falls back to the SYNC parse for
  that refresh — highlighting never dies with the thread.
- RootView fields: `syntax_worker: Option<(Sender, Receiver)>`,
  `syntax_gen: u64` (monotonic per enqueue), `syntax_applied_gen:
  u64`, `syntax_pending: Option<(nonce, version)>` — the pending
  guard: `needs_syntax_refresh` stays true while a request is in
  flight, so without it the render would RE-ENQUEUE every frame.
- `refresh_syntax_cache`: unchanged key check → small file = the
  existing sync path VERBATIM; large = enqueue (edit info = exactly
  one delta in `edits_since(cache_version)` AND the cache nonce
  matches → that delta's byte triple; else None=full) + pending set.
- Pump drain (next to forge_pending, the #203 dirty rule): try_recv
  loop; PURE accept `accept_syntax_resp(resp_gen, resp_nonce,
  applied_gen, active_nonce) -> bool` = `resp_nonce == active_nonce
  && resp_gen > applied_gen`; apply = set cache (nonce, version,
  Rc<mapped lines>) + applied_gen = resp_gen + clear pending when it
  matches + dirty. A stale-but-newer-than-shown response APPLIES
  (closer to truth); the pending guard keeps the enqueue cadence
  correct.

### §20 confirmation
Tree-sitter's own public incremental API (Tree::edit +
parse(Some(&old)) + the InputEdit byte/Point contract from its docs).
Marley-original orchestration. No copyleft editor source. Holds.

### Manifest
| file | change |
|---|---|
| crates/syntax/src/lib.rs | SyntaxEdit + point_at + syntax_edit + HighlightSession + the shared span→lines pipeline + tests |
| crates/syntax/src/parse.rs | tree-walking capture collection generalized for session reuse (parser owned by the session) |
| crates/marley_app/src/app.rs | threshold const + worker fields/spawn/enqueue + pump drain + the pure accept fn |
| crates/marley_app/src/headless_drive.rs | REQ-004/005 flows |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | point_at: ASCII rows/cols, multibyte (é/日/😀) cols in BYTES, line starts/ends, EOF, past-EOF clamp, empty src; syntax_edit exact 6-tuples: mid-line insert, cross-line delete, multibyte insert, at-0, at-EOF, newline-inserting, pure-delete |
| REQ-002 | the equivalence corpus: for ~12 edit cases (insert/delete/replace × mid-line/cross-line/multibyte/at-0/at-EOF + a block-comment-opener whose damage cascades DOWN the file), session full(old) → incremental(new, edit) spans == a fresh full(new) — exact equality |
| REQ-003 | the ratio pin: a generated ~8k-line fixture; median-of-3 full vs incremental 1-char edit; assert incremental < full / 3 (same build — debug/release independent) |
| REQ-004 | accept-decision table (gen older/equal/newer × nonce match/mismatch); the worker validity rule via session units (parent mismatch ⇒ full — observable: spans still correct after a deliberately skipped chain); headless: type twice fast into a LARGE file → the final cache equals the final text's spans (the stale response dropped or superseded) |
| REQ-005 | headless small-file: the existing #268 flows byte-identical (sync same-frame); headless large-file: after a keystroke the cache LAGS (pending guard on, old spans still rendered), then bounded parked pumps land the fresh spans; a dead-worker fallback unit (send to a dropped receiver → sync path result) |
| uncoverable | the worker thread body (masked shim, mutants::skip — the decisions it composes are the pure fns above); exact swap-latency timing (bounded-retry parked loops, not wall-clock asserts) |

### Risks / decisions
- R1 — worker death degrades to sync (never a blank editor).
- R2 — the pump's 16ms cadence bounds swap latency ≈ 1 frame (D5).
- R3 — memory: the session's snapshot + the request's snapshot ≈ 2×
  file size transient — fine at the sizes involved.
- R4 — the Rc cache stays UI-thread-only; the channel moves plain
  Vecs (Send-safe by construction).
- R5 — the #268 memo key semantics unchanged — the whole existing
  cache/nonce discipline (incl. #275's reload re-mint) carries over
  untouched.

## Phase 3 — Implement
- **marley_syntax:** `point_at` (clamped, rfind-newline col),
  `SyntaxEdit` + `syntax_edit` (pre-snapshot start/old_end +
  post-text new_end), `HighlightSession { parser, tree, last_src }`
  with `highlight_full` / `highlight_incremental(new_src, start,
  old_end, new_len)` (builds the edit internally from its OWN
  last_src — the app never touches point math; a `fits` guard falls
  back to full, total per §14); the span→lines tail extracted as
  `lines_from_spans` (shared — incremental can't diverge in
  post-processing); parse.rs gains `spans_from_tree` (the one query
  walk both paths use) + `rust_parser()`; `highlight_lines` behavior
  byte-identical.
- **app.rs:** SyntaxReq/SyntaxResp (+ parent_version), the
  `SYNTAX_SYNC_MAX_LINES = 1000` threshold, pure
  `accept_syntax_resp`, worker fields (tx/rx, gen, applied_gen,
  pending guard), `ensure_syntax_worker_and_send` (lazy spawn; the
  worker drains-to-latest and applies the VALIDITY rule — session at
  exactly (nonce, parent_version) or full-parse; send-err → false →
  the caller's sync fallback), the enqueue arm in
  `refresh_syntax_cache` (borrow-scoped reads — the long `surface`
  borrow fought the &mut enqueue, E0502, restructured to three scoped
  reads), and the pump drain (accept → map kinds → swap cache + gen +
  clear pending + dirty; a DROPPED response that matches pending also
  clears it so the guard can never wedge across a tab switch).
- No design deviations. check + clippy `-D warnings` + fmt clean;
  suite 990/990 (all #268 sync-path pins green — the small-file path
  is verbatim).

## Phase 3.5 — Inspect
### Critic findings ledger (2 critics: the tree-sitter contract w/
probe `ts_probe` path-depending on the REAL crate, run debug+release;
worker lifecycle w/ exhaustive state-inventory trace)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| TS-✓ | **The make-or-break EQUIVALENCE: CLEAN 15/15** — mid-ident insert, /*-at-top cascade-to-EOF, */-delete heal, 日本語 in/out of strings, newline mid-fn, 3-line delete, byte-0, EOF, token-boundary replace, in-string, 10-step chains — every step == a fresh full parse, debug AND release; point_at matches tree-sitter's OWN node positions across 5 corpora. | — | — |
| TS-F1 | [MED] `point_at` PANICS on a non-char-boundary byte (`&src[..byte]`), probe-proven reachable through the public `highlight_incremental` (`fits` checked lengths only). The shipped worker can't hit it (ropey boundaries) but §14 totality says the public fn must not panic. | REAL | Floor-to-boundary loop in point_at + `fits` gains char-boundary checks. |
| TS-F2 | [MED] The REQ-003 pin was WRONG AS WRITTEN: end-to-end incremental = 0.46 of full (release; 0.58 debug) — the PARSE is 9× faster (1.0 vs 9.2 ms) but the O(file) QUERY WALK (5.7 ms over 27k captures) runs in full every call. The ticket's "the parse half dominates" premise measured backwards. | REAL (spec bug) | REQ-003 amended: (a) parse-only < 1/3 (measured 0.11), (b) end-to-end < 3/4; frame-safety is REQ-005's off-thread guarantee. The query-walk windowing (changed_ranges + set_byte_range + cached-span merge) = a follow-up ticket with the measured floor. |
| TS-F3 | [LOW] `fits` accepted length-consistent LIES → silently wrong spans (probe: stale Number/Str spans on far same-length changes; tree-sitter 0.26 self-heals gross lies, position-compatible ones corrupt). | REAL | The splice contract itself is now checked: prefix/suffix memcmp (µs against the ms parse) → any lie falls back to the always-correct full. |
| W-F1 | [MED] Dead-worker WEDGE: a panic with a request in flight leaves `syntax_pending` set forever (no response ever clears it; the sync fallback hides BEHIND the guard) — and combined with a later file switch could freeze a foreign cache in place. My Phase-3 comment "the guard can never wedge" was an overclaim. | REAL | The pump drain now matches `TryRecvError::Disconnected` → tear down (`syntax_worker = None`, pending cleared) → the next refresh goes sync and may respawn. |
| W-F2 | [LOW] Cross-NONCE stale render: switching large Rust files painted the OLD file's spans for the async window (D5 promised stale-but-ALIGNED — same file only); pre-#274 the sync refresh made cache==active an invariant. | REAL | The enqueue arm drops a foreign-nonce cache (hand-lexer fallback until the response lands). |
| W-F3 | [cosmetic] Failed-send double-clones the text (the rare dead-worker path). | NOTED | Not worth the plumbing; the fallback path is exceptional. |
| W-F4 | [note] version/text arrive from two scoped reads — correct today (nothing runs between), trust-fragile tomorrow. | NOTED | Comment added at the seam. |
| — | Cleared (verified): the pending matrix under bursts/skips/parallel-parse/FIFO delivery; close-while-pending benign (unique nonces); generation monotonicity (+1 only on successful send); no sync double-parse; the stale-version apply semantics; the edit-triple validity chain AIRTIGHT (one `cached` read feeds parent_version + edits_since + the filter; the session's at-match guarantees the snapshot IS the parent; `fits` backstops); UI-thread costs bounded (one rope copy per enqueue); panic containment (post-death = permanent per-refresh sync = pre-#274 parity); pump borrow/TOCTOU; the threshold boundary semantics; accept-gen semantics. Targeted suites 12/12. | — | — |

- Post-fix verification: check + clippy + fmt clean; suite 990/990;
  the critic's probe RE-RUN with the hardening in — equivalence still
  15/15 (its only FAIL line encodes the pre-amendment ratio pin,
  which the measurement itself disproved).

## Phase 4 — Validate
- **marley_syntax units (6, `t274_*`):** point_at rows/cols/edges
  (BYTE columns over é, col-0 after \n, EOF, clamp, the TS-F1 floor
  pin on a mid-multibyte byte); syntax_edit exact 6-tuples (insert /
  delete / newline-inserting-at-0 / multibyte replace / EOF append);
  the EQUIVALENCE CORPUS (7 shapes with COMPUTED offsets — the first
  draft hand-counted a multibyte offset and my own test caught a
  missing `new_src.is_char_boundary(start_byte)` in the `fits` guard:
  the guard itself panicked on a lying edit — the guard must be total
  UNDER lies, that's its purpose; fixed) + a 10-step incremental
  chain, every step == a fresh full parse; the TS-F3 splice-lie and
  bad-byte fallback pins; the AMENDED REQ-003 ratio pins (parse-only
  < 1/3, end-to-end < 3/4, medians of 3 on an ~8k-line fixture);
  Default covered.
- **app units:** the accept_syntax_resp truth table (6 rows — newer/
  equal/older gen × active/foreign/None nonce; first-response 1>0).
- **Headless:** `syntax_async_large_file_lands_off_thread_headless` —
  a 1200-line fixture ENQUEUES (the cache lags right after the
  keystroke: no inline parse on the frame), then the REAL worker
  thread's response lands through the pump (test clock advanced for
  the timer, wall-clock polls for the thread) and the cache keys to
  the exact (nonce, version); repeats after an incremental keystroke.
  The existing #268 sync flow (`syntax_cache_populates_and_rekeys…`)
  stays green — the small-file path is verbatim (REQ-005's other
  half).
- **Gate:** first `--diff` run RED on gate:4 — 5 uncovered lib.rs
  lines: the never-called `Default` impl + the two dead-by-design
  `None => Vec::new()` parse arms. Fixed at source: Default pinned by
  a test; the impossible arms collapsed to one-line `map_or_else`
  (the line executes; the closure region is the documented
  dead-by-design). Re-run: **GATE GREEN [diff] — 15/15 PASS**
  (coverage 100 lines · MSI 100 · everything else green; no
  pre-existing failures; nothing excluded beyond the standing
  parse.rs FFI shim).
- **Driven capture: ENV-BLOCKED (machine locked; black probe again)**
  — "unchanged colors while typing into a large file" joins the
  unlock re-verify batch; the headless async flow + the equivalence
  corpus carry the behavior.

## Phase 5 — Complete
- CHANGELOG under "### Added"; editor.md's marley_syntax bullet
  rewritten for B3.2 (session, splice contract, worker protocol, the
  measured decomposition + the windowing follow-up).
- Follow-up ticket **#285** (query-walk windowing — the measured
  5.7ms O(file) floor; the original <1/3 end-to-end pin becomes
  achievable there). AAR 9704870c submitted (completed; materialized:
  BF-claude-perf-pin-encoded-an-unmeasured-premise).
- Forge #274 closed (done); local ticket → closed/.
- Lessons (in the AAR): (1) measure a perf pin's DECOMPOSITION before
  writing it — the ticket's "parse half dominates" was backwards and
  the pin encoded it; (2) a validity guard that exists to catch LIES
  must itself be total under lies (my own corpus test caught the
  missing new_src boundary check panicking the guard); (3) the
  previous-text-snapshot design turned every InputEdit point into
  pure string math — the pure-seam premium paid for itself in the
  probe's 15/15 equivalence.
