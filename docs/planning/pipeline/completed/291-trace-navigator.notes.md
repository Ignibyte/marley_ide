# Multi-frame stack-trace navigator — Notes

- **Forge ticket:** #291 `66bf7033-82aa-427c-986a-4652e7cec6df`
- **AAR:** `1c28410d-d18b-4bfc-ad88-367f651bf9e6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-291-trace-navigator.md
- **Pipeline spec:** 291-trace-navigator.spec.md

## Phase 1 — Plan
- **Request:** parse trace frames (esp. the Python `File "path", line N` shape #212
  misses) + fold the open-file frames into the #289/#290 diagnostics.
- **Classification / tier:** work pipeline slice, `feature`. Crates: `marley_app`
  (`links.rs` pure parser, `app.rs` shim fold). Validate: pure fixtures + headless.
- **Forge recall:** #212 `scan_links`/`LinkTarget::File{line}`; #289
  `open_file_diagnostic_rows`/`diagnostics_for_file` (the gutter row set) +
  `resolve_under_root` (#190); #290 F8 nav walks the same rows. So folding the
  trace frames into that row set gives gutter + nav for free.
- **Key discovery (the scope):** rust panic (`panicked at a.rs:10:5`), backtrace
  (`at ./x.rs:12`), node (`at f (b.js:3:1)`) are all `path:line`-shaped → `scan_links`
  ALREADY linkifies them (verified: node's `(b.js:3:1)` strips the parens; the panic's
  trailing `:` is trimmed). The Python `File "p.py", line 42, in fn` is the ONLY gap —
  `scan_links` mangles the quoted path (`"p.py`) and misses the separated line.
- **Decisions:** D1–D4 in the spec. Crux: `parse_trace_frames` explicitly handles
  the Python shape + falls back to `scan_links` for the `path:line` forms; the frames
  fold into `open_file_diagnostic_rows` (no new UI — reuse #289 gutter + #290 F8).
- **Risk:** low-moderate. The Python matcher is simple string parsing (`File "` …
  `", line ` … digits) — a clean pure seam (cov/MSI 100). The fold is a one-line
  union in the app.rs shim (excluded). The standalone frame-list panel is deferred.

## Phase 2 — Design

### Architecture / approach
Two pure fns in `links.rs` + a one-line union in the app.rs render shim — no new
UI, no new state. Fits the existing pipeline: gpui app → PTY Blocks → the #289
diagnostics gutter reads a pure row set; #290 F8 walks it. We add a second pure
row source (the trace frames) and union it in.

- `parse_trace_frames(output) -> Vec<(PathBuf, usize, Option<usize>)>` — per line,
  `parse_trace_frame`: try `python_frame` first, else the first `scan_links`
  File-with-line via the **reused #213 `first_failure_ref`** applied to that one
  line (a single-line `&str` → `.lines()` yields exactly `[line]`, so it returns
  the first File-with-line ON that line). Frames collected in order (D2: ≤1/line).
- `python_frame(line)` — **zero index arithmetic** (a clean mutation profile),
  a `split_once` chain:
  `line.split_once("File \"")?.1` → `.split_once('"')?` gives `(path, rest)` →
  reject empty path → `rest.split_once(", line ")?.1` → `take_while(is_ascii_digit)`
  → `parse::<usize>().ok()?`. Empty/non-digit tail → parse Err → `None`. Returns
  `(path, N, None)` — Python gives no column.
- `trace_diagnostic_rows(output, open_path, root) -> Vec<usize>` — mirrors
  `diagnostics_for_file`'s tail: `parse_trace_frames` → keep frames whose
  `resolve_under_root(root, path) == *open_path` (#190) → `l.saturating_sub(1)`
  (1-based → 0-based) → `sort_unstable` + `dedup`. Pure path arithmetic, no IO.
- **Shim** `open_file_diagnostic_rows` (app.rs:5160, already `mutants::skip` +
  file-excluded): bind `let out = last.output_text();` once, then
  `rows.extend(diagnostics_for_file(&out,…)); rows.extend(trace_diagnostic_rows(&out,…));`
  The trailing `sort_unstable`+`dedup` already collapses the overlap (a rust panic
  line is found by BOTH scanners → one row after dedup).

**§14:** all pure, `Option` returns (no panics on the output path); the string
parse can't overflow-panic (`parse().ok()?` swallows a giant N → `None`). No IO,
no process-spawn — the parser is a pure seam; the shim is the only app-state read.

**§20 (clean-room) — CONFIRMED.** Reference = Zed / a debugger **call-stack**: a
trace's frames are navigable source locations. Marley matches by parsing every
frame's location (incl. Python's, which #212's `path:line` scanner misses) and
surfacing them through the shipped #289 gutter + #290 F8. The parser is Marley's
own (reusing #212 `scan_links` + #213 `first_failure_ref`); **no Zed/Warp source
read or translated** — observed behavior only.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/links.rs` | + `pub fn parse_trace_frames` , `fn parse_trace_frame` , `fn python_frame` , `pub fn trace_diagnostic_rows` (all pure; tests added in P4) |
| `crates/marley_app/src/app.rs` | `open_file_diagnostic_rows`: bind `out` once, add a second `rows.extend(trace_diagnostic_rows(&out,&open_path,&root))` union (shim; `mutants::skip` + file-excluded) |
| `crates/marley_app/src/headless_drive.rs` | + a headless fold smoke (mirrors #289's empty-case) — see test plan T5 |

### Regression Test Plan
| # | Test (`links.rs` `#[cfg(test)]` unless noted) | AC | Asserts |
|---|---|---|---|
| T1 | `parse_trace_frames_python_shape` | REQ-001 | `File "p.py", line 42, in fn` → `[(p.py,42,None)]`; an indented `  File "a/b.py", line 7, in <module>` → `[(a/b.py,7,None)]` |
| T2 | `parse_trace_frames_reuses_scan_for_path_line` | REQ-001 | rust `thread 'main' panicked at src/a.rs:10:5:` → `[(src/a.rs,10,Some(5))]`; node `    at f (b.js:3:1)` → `[(b.js,3,Some(1))]`; backtrace `   at x/y.rs:12` → `[(x/y.rs,12,None)]` |
| T3 | `parse_trace_frames_no_frame_and_multi_and_malformed` | REQ-002 | a plain line → none; a 3-line Python traceback → 3 frames IN ORDER; malformed `File ""`, `File "a.py"` (no `, line`), `File "a.py", line abc`, `File "unterminated` → none |
| T4 | `trace_diagnostic_rows_filters_dedups_zero_bases` | REQ-003 (pure) | frames into the open file → sorted, deduped, 0-based rows; an other-file frame excluded; a bare non-frame line → `[]`; empty → `[]` |
| T5 | `diagnostics_fold_includes_trace_rows_headless` (headless) | REQ-003 (fold) | If a failed block can be injected in the headless lane: a Python-traceback block + the open file → `open_file_diagnostic_rows()` includes the frame rows. **If block-injection is env-blocked (as #289/#290): mirror #289's empty-case smoke (no failure → `[]`, no regression), and carry REQ-003 via T4 (pure rows correct) + the mechanism (the shim `.extend` is byte-identical to the shipped #289 one that renders).** Resolved concretely in P4. |
| T6 | gate `scripts/gates.sh --diff` | REQ-004 | 100% line cov + MSI 100 on the four new `links.rs` fns; brand-scrub; clippy -D |

**Uncoverable note:** the live UI (the gutter danger-color pixel for a real
Python traceback) needs a spawned process emitting a traceback into a failed
block — heavy/flaky in headless and env-blocked for a driven capture (locked
machine, per this session). Carried by T4 (pure) + the byte-identical shim
mechanism + the #289 render already proven. Stated, not silently skipped.

### Risks / decisions
- **D5 — reuse `first_failure_ref` per-line, no refactor.** `first_scan_frame`
  would duplicate #213's match arm; instead `parse_trace_frame` calls
  `first_failure_ref(line)` directly (a one-line `&str` → the first File-with-line
  on it). DRY, satisfies the inspect reuse-lens, zero churn to the tested #213 fn.
- **D6 — union BOTH pure sources, don't replace.** `diagnostics_for_file`
  (`scan_links`) collects ALL File refs per line; `parse_trace_frames` collects ≤1
  frame/line (D2). Neither subsumes the other → the union is the correct superset;
  `sort`+`dedup` collapses the path:line overlap.
- **D7 — Python path taken verbatim from the quotes**, then `resolve_under_root`
  like every other ref (spec D4). A path with an embedded `"` is not a real
  traceback shape → out of scope.
- Reversible-but-load-bearing: the `split_once` chain (vs index math) is a
  deliberate choice for a clean mutation profile — run `cargo mutants --list -f
  links.rs` in P4 for the REAL operator set (per the trace-the-real-list rule)
  before assuming which mutants exist.

## Phase 3 — Implement
- **`links.rs` (+50 lines)** — `parse_trace_frames` (`.lines().filter_map(parse_trace_frame)`),
  `parse_trace_frame` (`python_frame(line).or_else(|| first_failure_ref(line))` — D5 reuse),
  `python_frame` (the `split_once("File \"")?.1.split_once('"')?` chain → empty-path
  reject → `split_once(", line ")?.1` → `take_while(char::is_ascii_digit)` → `parse().ok()?`),
  `trace_diagnostic_rows` (mirror of `diagnostics_for_file`'s tail over the frame source).
- **`app.rs` `open_file_diagnostic_rows`** — bound `let out = last.output_text();` once
  (was inline), then two `rows.extend(...)`: the existing `diagnostics_for_file` +
  the new `trace_diagnostic_rows` (D6 union). Trailing `sort_unstable`+`dedup` collapses
  the path:line overlap. Shim stays `mutants::skip` + file-excluded.
- **Deviations from design:** none. `take_while(char::is_ascii_digit)` (fn-pointer
  form) instead of a `|c| c.is_ascii_digit()` closure — same behavior, `char::is_ascii_digit`
  is `fn(&char)->bool` which satisfies `take_while`'s `FnMut(&char)->bool`. `cargo check
  -p marley` clean (the transitive `block v0.1.6` warning is pre-existing, unrelated).
- **Tests deferred to P4** (validate) per the phase split.

## Inspect (Phase 3.5)
Two parallel critics (correctness; simplification-reuse+provenance) + my own review
+ a standalone scratch-compile of `python_frame` across a 10-case edge table.

**My empirical confirmation (scratch `rustc`, not the crate):** all 10 edge cases
matched the trace-through EXACTLY, zero panics — canonical, indented, mid-line
marker → `Some`; no-marker / unterminated-quote / empty-path / no-`,line` /
non-digit-tail / empty-digit / 27-digit-overflow → `None`. `resolve_under_root`
re-read + confirmed pure (`is_absolute`+`join`, no stat/IO).

**Correctness critic — verdict: correct + panic-safe; all 6 concerns NON-ISSUES**
(independently scratch-compiled the same table). Notable confirmations: (a) NO
double-count via three mechanisms — `or_else` short-circuits, `scan_links` yields
`File{line:None}` on a Python line (quoted-path token, the `N` is a separate
token) so `diagnostics_for_file` skips it, and the shim `sort`+`dedup` collapses
the plain-`path:line` overlap; (b) `line 0`→`saturating_sub`→row 0, no underflow,
consistent with the shipped #289; (c) Windows `C:\…` paths parse (keys on `, line `
not `:`) — more robust than `scan_links`. One actionable: **[MED] zero tests on
the new pure fns** — a VALIDATE-phase item (tests written there; T1–T4 already in
the design plan; the critic handed over its scratch table as an oracle). Two LOWs:
redundant non-Python compute (idempotent, dedup collapses — no change); a two-space
`, line  10` → None (Python always emits one space — out of the real shape).

**Reuse/provenance critic — verdict: diff sound; §20 confirmed** (whole-word brand
grep `warp|zed|iterm|tmux|kitty|alacritty|wezterm` = ZERO matches; the parser is
Marley's own elementary string logic reimplementing the *publicly observable*
CPython traceback format — observed-behavior, no GPL/AGPL source). Rejected the
union/alloc/`into_iter` concerns with evidence (neither diagnostic fn subsumes the
other → union required; `into_iter` is the correct owned consumption). Two LOWs:

| Finding | Verdict | Action |
|---|---|---|
| **LOW #1 — tail dup** (`trace_diagnostic_rows` vs `diagnostics_for_file`: the `filter→sub(1)→sort→dedup` tail) | **REJECTED** — the critic itself judged extraction "line-neutral" + "defensible as-is"; a shared helper would churn the shipped/tested #289 fn for no net LOC. D6 documented-twin stands. | none |
| **LOW #2 — `parse_trace_frame` → `first_failure_ref`** coupling (a cross-output-named fn used per-line → the "lines() degenerates to one" reasoning + name-mismatch smell; my own D5 flagged it) | **ACCEPTED** — behavior-preserving cleanup both my review + the critic converged on. | **FIXED:** extracted `fn first_file_ref_on_line(line)`; `first_failure_ref` now delegates (`output.lines().find_map(first_file_ref_on_line)`), `parse_trace_frame` calls it directly. `cargo check` clean (no dead-code), all 9 existing `links` tests green incl. the #213 `first_failure_ref` test → behavior-preserving. |

**Ledger:** 1 finding fixed (LOW #2 extraction); 1 deferred to validate (MED tests,
planned); 3 rejected with evidence (LOW #1 dup, 2× correctness LOWs). No CRITICAL/
HIGH/MED code defects. Lenses covered: correctness/edges, panic-safety, double-count/
union, simplification/reuse, provenance/§20, MSI reachability.

## Phase 4 — Validate
### Tests added (`links.rs` `#[cfg(test)]`)
- **T1 `parse_trace_frames_python_shape`** (REQ-001) — canonical `File "p.py", line 42, in fn`
  → `(p.py,42,None)`; indented `  File "a/b.py", line 7, in <module>` → `(a/b.py,7,None)`.
- **T2 `parse_trace_frames_reuses_scan_for_path_line`** (REQ-001) — rust panic
  `…panicked at src/a.rs:10:5:` → `(src/a.rs,10,Some(5))`; node `at f (b.js:3:1)` →
  `(b.js,3,Some(1))`; backtrace `at x/y.rs:12` → `(x/y.rs,12,None)`.
- **T3 `parse_trace_frames_no_frame_multi_and_malformed`** (REQ-002) — plain line → `[]`
  (kills the "always Some" body mutants); a 3-line traceback → 3 frames IN ORDER; 4 malformed
  Python shapes (empty path / no `, line ` / non-digit tail / unterminated) → `[]`.
- **T4 `trace_diagnostic_rows_filters_dedups_zero_bases`** (REQ-003 pure) — a.rs frames at
  12/9/12 + a b.rs frame → sorted-deduped-0-based `[8,11]`; the other-file frame kills the
  resolve `==`→`!=` mutant; other-file-only / empty → `[]`. The extraction kept #213 green.

### Real results
- **cargo mutants --list -f links.rs** (the trace-the-real-list rule) → the REAL set on the 4
  new fns is **32 mutants**: whole-body `None`/`Some(Default…)`/`vec![…]` returns + the match-arm
  delete on `first_file_ref_on_line` + the ONE `==`→`!=` in `trace_diagnostic_rows`. NOTABLY **no**
  `saturating_sub`/`sort_unstable`/`dedup` mutants (cargo-mutants leaves those method calls) — so
  no over-engineered tests. T1–T4 kill all 32.
- **`cargo nextest run -p marley`** → **451 passed, 2 skipped** (incl. the 4 new + the #289
  `diagnostics_gutter_empty_without_a_failure_headless` still green = no shim regression).
- **`scripts/gates.sh --diff`** → **GATE GREEN [diff]** — 15/15, incl. gate:4 coverage ≥100% lines
  + gate:5 mutation MSI ≥100% on the new `links.rs` fns, brand-scrub, miri, visual/AX.

### LIVE DRIVE — end-to-end pixel proof (the machine unlocked mid-session; captures work)
Staged the exact #291 delta on the running app (self-test harness, AX_TRUSTED):
`New empty workspace` → ONE terminal → `echo /tmp/marley_291/boom.py:3` (a clean link) +
`python3 /tmp/marley_291/boom.py` (FAILS → the last block, a Python traceback) → clicked the
link → boom.py opened in the editor. Absolute paths make `resolve_under_root` an identity, so
the frame path == the open path regardless of the workspace root.
- **Instrumented `open_file_diagnostic_rows` (temporary `eprintln`, since REMOVED) proved the fold:**
  `open_path="/tmp/marley_291/boom.py"`, `term last-block kind=Failure`,
  **`trace_rows=[2]  diag_rows=[]`** — the Python frame's row (2) came ENTIRELY from #291's
  `trace_diagnostic_rows`; the #289 `scan_links` source (`diagnostics_for_file`) found NOTHING
  (it can't parse `File "…", line N`). Exactly the delta the ticket exists for.
- **Pixel read of the editor gutter** (`291-08-editor.png`): line 3's number = **(218,98,105)
  danger-red** (redness +113); lines 1/2/4/5/6/7 = (145,149,161) muted gray. The gutter marks
  the traceback line. **REQ-003 proven live**, not merely by mechanism.
- **Discovery (out of #291 scope, follow-up):** the first driven attempt (a multi-tab persisted
  workspace) did NOT light — root-caused to `workspace()` via `terminal_grid_index()` scanning
  only the ACTIVE-or-FIRST terminal grid; my python block was in a different terminal tab than the
  one it resolved to after the editor became active. A single-terminal workspace works. This is a
  #289 `open_file_diagnostic_rows` scoping limitation (scan ALL terminals across tabs), NOT a #291
  parse bug — filed as a follow-up (see Phase 5).
- Cleanup: debug `eprintln` reverted (app.rs byte-identical to the gated state), `~/.marley`
  settings restored, `/tmp/marley_291` removed, tree clean. Gate re-run GREEN [diff] after cleanup.

No pre-existing failures in scope.

## Phase 5 — Complete
- **CHANGELOG.md** — `### Added` (M18): the #291 trace-navigator entry (Python delta,
  the pure fns, the live-drive `trace_rows=[2] diag_rows=[]` + danger-red proof).
- **docs/marley_architecture/app_shell.md** — added the `M18 (#291)` note after #290,
  incl. the known scope limit + the #295 follow-up pointer.
- **Knowledge (forge wired):**
  - AAR `1c28410d` submitted (completed, effectiveness 5; 2 novel findings).
  - `failure-record` **BF-claude-diagnostics-single-terminal-scope-001** — the live
    drive surfaced a #289-era scoping limit invisible to pure tests.
  - `prevention-rule` **PR-claude-shim-aggregate-all-terminal-grids-001** — a
    workspace-wide shim scan must iterate ALL terminal grids, not `workspace()`'s
    active-or-first; and a cross-tab aggregation needs a MULTI-terminal-tab driven test.
  - Follow-up **ticket #295** (bug) — fix `open_file_diagnostic_rows` to aggregate
    across all terminal grids.
- **Lessons:** (1) The trace-the-real-mutants-list rule paid off again — `--list -f
  links.rs` showed 32 real mutants and NO `saturating_sub`/`sort`/`dedup` mutants, so
  T1–T4 stayed lean. (2) `split_once` chains beat index arithmetic for a clean mutation
  profile (python_frame had zero off-by-one mutants). (3) Reusing #213 via the extracted
  `first_file_ref_on_line` (the inspect LOW #2 fix) removed a coupling smell AND kept
  #213 green. (4) The LIVE DRIVE was decisive: it PROVED the Python delta end-to-end
  (a pixel-measured danger-red gutter) AND caught the cross-tab scoping gap that unit
  tests + #289's env-blocked validation both missed — reinforcing the drive mandate.
