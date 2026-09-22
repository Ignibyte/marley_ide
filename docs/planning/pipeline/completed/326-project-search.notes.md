# Project-wide content search (⌘⇧F) — Notes

- **Forge ticket:** #326 992a1da5-7236-4e66-8bc2-e1557a672e78
- **AAR:** 206fb9a2-6683-47d2-bcc1-c7d687c1b0ac
- **Local ticket doc:** docs/planning/tickets/open/TICKET-326-project-search.md
- **Pipeline spec:** 326-project-search.spec.md

## Phase 1 — Plan
- **Request:** ⌘⇧F content search across the workspace — roadmap B7 phase 1 (read-only results surface).
  Pure literal-substring engine (case/word toggles) over an off-thread walker; dirty buffers win over disk;
  honest caps + "+N more"; generation-cancel on re-query; finder-recipe picker with grouped `path:line` rows +
  Enter → #312 open_and_place_caret + NavStack.
- **Classification / tier:** FEATURE, single shippable slice. Larger than a typical work-pipeline (new pure
  engine + off-thread worker + a new overlay + possibly a new dependency), but the read-only surface is ONE
  coherent slice — phase-2 editability is a separate future pipeline, so no split needed.
- **Systems involved:** server/ui (the app shim + a new ⌘⇧F overlay), a new PURE search engine, the walker
  seam in `marley_project`, an off-thread worker (channel + pump-drain). No settings, no fleet/agents/bridge.
- **Forge recall (§18.3):** no bulletins. AAR 206fb9a2 opened (0 pre-flagged codes, 0 inferred kinds).
  knowledge-search surfaced the prevention-rule families that apply here (resolved by code, not id): the
  stale-generation guard family (a new query cancels the old walk), the finder recipe + `open_and_place_caret`
  guard inheritance (`PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`), the #325
  cap==nav rule (`PR-claude-display-cap-must-equal-navigation-cap-001`), and dirty-buffer precedence. Fed to
  Design.
- **Discovery (seams verified in the tree, not guessed):**
  - `marley_project::should_skip` (lib.rs:61 — `.git|node_modules|target|.DS_Store`) + `list_files_in`
    (lib.rs:98, a hand-rolled DFS; "a full .gitignore parser is deferred"). The walker seam.
  - `is_probably_binary` (code_view.rs:390) + `VIEWER_MAX_BYTES = 2_097_152` (app.rs:659) via `viewer_size_ok`
    — the editor's own text/loadable rules. D3's "one truth about text".
  - `open_docs` (editor_surface.rs:418) → `impl Iterator<Item = (&Path, &Buffer)>` — every open file's live
    buffer. The dirty-buffer source (D4).
  - The syntax worker: `syntax_worker: Option<(…)>` (app.rs:367), `ensure_syntax_worker_and_send` (app.rs:8957),
    `std::sync::mpsc::channel` + the pump-drain arm (app.rs:1092). The off-thread + streaming precedent (D5).
  - `FinderState` (finder.rs:10) — the picker recipe (D6).
  - `deny.toml [licenses] allow` includes `MIT` → `ignore` ("Unlicense OR MIT") passes the license gate via
    MIT; the OPEN cost is its transitive tree needing per-crate license-allow + `[bans]` review (D-WALKER,
    design confirms with a real `scripts/gates.sh` deny run if Route A).
- **Decisions:** D1–D7 locked (see spec). **D-WALKER is the one design-owned decision** — Route A (`ignore`
  crate) vs Route B (extend `list_files_in`); A leans right for gitignore correctness but must clear the
  dependency-provenance gate; design picks + justifies.
- **§20:** behavior-only reference to Zed/VS Code ⌘⇧F; implementation is Marley's own composition + the
  permissive `ignore` crate (if A). The deconstruction doc (06-project-fs-search.md §4) is our own analysis.
  Phase-1 = generic grep capability; the `[Zed-derived]` phase-2 multibuffer is deferred. No GPL source read.

## Phase 2 — Design

### D-WALKER DECIDED → Route A (adopt `ignore`)
The dep-cost objection COLLAPSED on a real check: every transitive dep of `ignore` is ALREADY in
`Cargo.lock` (globset 0.4.18, aho-corasick 1.1.4, regex-automata 0.4.14, regex-syntax 0.8.11, walkdir 2.5.0,
same-file 1.0.6, crossbeam-deque 0.8.6, crossbeam-utils 0.8.21, memchr 2.8.2, bstr 1.12.3, log 0.4.33,
winapi-util 0.1.11). Adopting `ignore` adds **exactly one crate** (`ignore` itself, MIT OR Unlicense → passes
deny's MIT allow). `[bans] multiple-versions = "allow"`, `wildcards = "allow"` → no dup/wildcard concern. So
Route A buys gitignore-aware walking (respects `.gitignore`/`.ignore`/hidden — the correctness a content
search lives on: surface real source, not `dist/`/generated junk) for ~zero marginal cost, vs Route B's
hand-rolled gitignore parser (more code, more risk, all to reimplement `ignore`). **Route A.** `marley_project`
has NO deps today, so `ignore` is its first — it stays gpui-free (a fs/model crate). Implement MUST run
`scripts/gates.sh` (cargo-deny) after adding it to confirm green; if `ignore` itself needs a one-line license
note, add it to `deny.toml` with a reason (the TICKET-007 pattern) — not expected (MIT is already allowed).

### Architecture / approach
Three layers, the pure/masked split the house uses (pure seams cov/MSI 100; IO/thread/gpui = masked shim,
behavior-verified):
1. **PURE engine — `marley_project::search`** (gpui-free, editor-free, tool-shaped for a future
   `workspace.search` MCP read-tier): `search_lines(text, needle, &SearchOpts) -> FileMatches`. Splits text on
   `\n` (row index); per line collects every literal-substring match up to `opts.max_matches_per_file`
   (→ `truncated`); `col`/`len` are CHAR offsets (REQ-002 non-ASCII correctness, the #309 unit); `preview` is
   the line clamped to `opts.max_preview_chars` from the start. Case-insensitive = **ASCII fold** v1
   (length-preserving → col stays exact; Unicode case-fold + regex are the NAMED follow-up). Whole-word via an
   `is_word_char` (`alphanumeric || '_'`) boundary test. Empty needle → empty.
2. **IO walker — `marley_project::walk_text_files(root) -> Vec<PathBuf>`** (masked shim over `ignore`,
   `#[cfg_attr(test, mutants::skip)]`): `WalkBuilder` with `.hidden(true)` + gitignore on, files only, still
   filtered by the pure `should_skip` (belt-and-suspenders for `.git`/`target`/`node_modules`/`.DS_Store` even
   when not gitignored). Behavior-verified by a tempdir integration test.
3. **App orchestration — `marley_app`** (the masked shim + a NEW pure helper module):
   - **PURE `marley_app::editor_search`** (cov/MSI 100): `SearchQuery{query,gen}` (the stale key);
     `MatchRow{file_idx,row,col,len,preview}` flat nav model; `accept_gen(result_gen, live_gen)->bool` (mirror
     the syntax worker's applied-gen drop); `visible_rows(rows, MAX_ROWS)` + reuse `editor_symbols::
     cap_with_tail` so the render cap EQUALS the ↑/↓/Enter clamp (the #325 rule); `footer_summary(total,
     files, dropped_files, dropped_ranges)`; the `MAX_FILES`/`MAX_MATCHES_PER_FILE`/`MAX_ROWS`/
     `MAX_TOTAL_RANGES` consts.
   - **Masked shim in `app.rs`**: mirror the syntax worker EXACTLY — `search_worker: Option<(Sender<SearchReq>,
     Receiver<SearchMsg>)>`, `search_gen: u64`, `search_applied_gen: u64`, plus a shared `Arc<AtomicU64>`
     latest-gen the worker checks BETWEEN files to CANCEL a superseded walk (REQ-007, real CPU cancel not just
     result-drop). `SearchReq{gen, root, query, opts, overrides: HashMap<PathBuf, Arc<str>>}` — `overrides`
     carries open-buffer live text snapshotted from `open_docs` BEFORE send, so the worker never touches gpui
     (REQ-004: worker uses `overrides.get(path)` if present, else reads disk). Worker per file: override-or-disk
     → `is_probably_binary(&bytes)` skip / `bytes.len() > VIEWER_MAX_BYTES` skip / `from_utf8_lossy` →
     `search_lines` → stream `SearchMsg::File{gen,path,matches,truncated}`, then `SearchMsg::Done`. The pump
     drains each frame (the app.rs:1092 precedent), keeps only `accept_gen` results, `dirty`-repaints.
   - **Enter = jump_to_match** reusing the #312 `open_and_place_caret` + NavStack + encoding-aware landing
     WHOLE (guard inheritance — a failed open flashes and moves nothing; ⌃- returns).
   - **⌘⇧F Editor-scoped** (mirrors #325's ⌘T scoping) → dispatch "project-search" → `open_search` (clears any
     live completion/other overlay — the #325 F-COMPLETION lesson).

### §20 confirm
Behavior-only reference to Zed/VS Code ⌘⇧F (a query, grouped `path:line` rows, Enter jumps) — reimplemented via
Marley's finder + worker + PURE engine + the permissive `ignore` crate. The two-phase plan is our OWN
deconstruction (06-project-fs-search.md §4). Phase-1 read-only surface = generic grep; the `[Zed-derived]`
phase-2 multibuffer stays deferred. No GPL source read/translated. Holds.

### File manifest
- `crates/marley_project/Cargo.toml` — ADD `ignore` (dep) + `tempfile` (dev-dep, the walker integration test).
- `crates/marley_project/src/search.rs` — NEW, PURE: `SearchOpts`, `LineMatch`, `FileMatches`, `search_lines`
  + `is_word_char`/the match-scan helper. cov/MSI 100.
- `crates/marley_project/src/lib.rs` — `pub mod search;` + re-exports; ADD `walk_text_files` (masked shim) +
  a tempdir integration test (gitignore respected, skip-set enforced, files-only).
- `crates/marley_app/src/editor_search.rs` — NEW, PURE: `SearchQuery`, `MatchRow`, `accept_gen`,
  `visible_rows`, `footer_summary`, the consts. cov/MSI 100.
- `crates/marley_app/src/lib.rs` — `mod editor_search;`.
- `crates/marley_app/src/app.rs` — the `search_worker`/`search_gen`/`search_applied_gen` fields + the shared
  latest-gen; `open_search`/`park_search`/`consume_search_query` (bump gen, snapshot `overrides` from
  `open_docs`, send); the worker body (masked); the pump-drain arm; `apply_search_msg`; `handle_search_key`
  (↑/↓/Enter/Esc, nav over MatchRows clamped to `visible_rows`); `jump_to_match` (reuse #312); `search_overlay`
  render (grouped-by-file headers + highlighted preview + footer); drain arm; dispatch/router/text_input_blocked/
  choke-clear arms; `open_search_for_test`/`push_search_msg_for_test`/`search_rows_for_test` hooks.
- `crates/marley_app/src/keymap.rs` — ⌘⇧F Editor-scoped → "project-search"; roster/scoped counts +1; assert.
- `crates/marley_app/src/headless_drive.rs` — the drives (below).
- Phase 5: `CHANGELOG.md`, `docs/marley_architecture/editor.md`, `crate-map.md`.

### Regression Test Plan (≥1 per REQ)
| REQ | Test(s) | Kind |
|---|---|---|
| 001 | `search_open_owns_keyboard_esc_closes` (drive) | headless + LIVE (fallback) |
| 002 | `search_lines_multi_match_and_nonascii_col`, `search_lines_empty_needle_empty` | pure unit cov/MSI 100 |
| 003 | `search_lines_case_toggle_ascii_fold`, `search_lines_whole_word_boundaries` | pure unit |
| 004 | `worker_prefers_override_text_over_disk` (decision) + `search_dirty_buffer_found_presave` (drive) | pure unit + headless |
| 005 | `should_skip_*` (exists), `walk_respects_gitignore_and_skipset` (tempdir), `search_skips_binary_and_oversize` (drive) | pure + integration + headless |
| 006 | `cap_with_tail_boundaries` (reuse), `footer_summary_states_truncation` | pure unit |
| 007 | `accept_gen_drops_stale`, `search_stale_generation_dropped` (drive) | pure unit + headless |
| 008 | `search_enter_jumps_and_navstack`, `search_failed_open_flashes_moves_nothing` (drives) | headless + review |
| 009 | `search_streams_results_via_drain` (drive pushes SearchMsg::File through the pump) | headless drain + review |
| 010 | `visible_rows_cap_equals_nav`, `search_cap_clamps_navigation` (drive: >MAX_ROWS, ↓ past cap stops at last visible) | pure unit + headless |
| walk | `walk_text_files` tempdir: a `.gitignore`'d file absent, `.git`/`target`/`node_modules` absent, a plain `.rs` present, dirs excluded | integration (marley_project) |

LIVE drive (env-fallback to units+mechanism if screen locked): seed two probe files with a planted needle →
⌘⇧F → both rows grouped, Enter lands centered (⌃- returns); edit one match away WITHOUT saving → re-search
finds N-1 (the dirty-buffer proof).

### Risks / decisions (load-bearing)
- **Route A (`ignore`)** — justified by zero-marginal-deps; implement re-confirms `cargo-deny` green post-add.
- **Case-insensitive = ASCII fold v1** — keeps `col` exact; Unicode case-fold + regex = named follow-up.
- **Search domain = the walked (gitignore-aware) set**; open buffers OVERRIDE a walked path's source, they are
  not an extra domain (a gitignored-but-open edited file is not searched v1) — noted follow-up.
- **Preview clamp** — `preview` = line clamped to `max_preview_chars` from start; a far-right match still gives
  a correct row + caret jump, only the inline highlight preview is imperfect (rare long-line case).
- **Cancellation** — a shared `Arc<AtomicU64>` latest-gen the worker checks between files (real CPU cancel);
  results also gen-guarded at the drain (`accept_gen`), so a slow stale walk can never paint.
- **Memory** — `overrides` clones open-buffer text per query; bounded by open-file count. Acceptable.

## Phase 3 — Implement
- **Built to the manifest.** `marley_project`: `search.rs` (pure `search_lines`/`LineMatch`/`FileMatches`/
  `SearchOpts` + `line_matches`/`window_eq`/`is_whole_word`/`is_word_char`/`clamp_preview`), `lib.rs`
  `pub mod search` + re-exports + the masked `walk_text_files` (`ignore::WalkBuilder.hidden.git_ignore`,
  files-only, `should_skip` belt); `Cargo.toml` +`ignore` +`mutants`. `marley_app`: `editor_search.rs` (pure
  `SearchQuery`/`accept_gen`/`locate_match`/`footer_summary`/`visible_matches`/`admit_matches` + the consts);
  `app.rs` the 6 fields + `OpenSearch`/`SearchFile` + `SearchReq`/`SearchMsg`/`run_search` + `open_search_finder`/
  `park_search_query`/`consume_search_query`/`ensure_search_worker_and_send`/`apply_search_msg`/
  `drain_search_worker`/`handle_search_key`/`jump_to_match`/`search_overlay` + the 3 test hooks + the wiring
  (dispatch/choke/text_input_blocked/key-event/render-mount/pump-drain); `lib.rs` `mod editor_search`; `keymap.rs`
  ⌘⇧F Editor-scoped + the roster guards.
- **Compile / gates:** `cargo check --workspace` + `--all-targets` clean (only the pre-existing `block v0.1.6`
  future-incompat). **`cargo deny check` → advisories ok, bans ok, licenses ok** — `ignore` resolved to
  **v0.4.29, exactly ONE new crate** (whole closure already in-lock), confirming the D-WALKER dep finding.
  `cargo fmt --all` clean.
- **Deviations from design (with reason):**
  1. **Nav = a flat match index + `locate_match`**, not a `MatchRow` struct — `finder.selected()` is one usize;
     `locate_match(file_counts, sel)` resolves it to `(file, match)` for the jump. Simpler, fewer types.
  2. **`admit_matches` pure helper added** for the `MAX_TOTAL_RANGES` merge, so the cap decision is mutation-
     tested (the app-side merge that calls it is in the coverage/mutation-EXCLUDED app.rs — gates.sh:217).
  3. **`jump_to_match` uses `PositionEncoding::Utf32`** — our `col` is a CHAR offset (not an LSP-encoded
     column), so Utf32 maps it directly through the #309 bridge; no owning-host encoding lookup (unlike #325).
  4. **Any OPEN file is searched via its live buffer** (the `overrides` snapshot covers every open doc), which
     subsumes the "dirty buffer" case and is identical for a clean file — no Buffer dirty-flag exists to check.
  5. **`SearchMsg::Done` carries only `gen`** (dropped the unused `files_scanned`) — it just triggers the
     final footer repaint via `accept_gen`.
  6. **`mutants` added as a normal dep to `marley_project`** (mirroring `marley_lsp`), so `#[mutants::skip]`
     resolves on the masked `walk_text_files`.
- **Coverage/mutation reminder for Validate:** the pure `search.rs` + `editor_search.rs` (and `walk_text_files`
  in the non-excluded `lib.rs`) need cov/MSI 100 via units + the tempdir walker integration test; the app.rs
  shim is gate-excluded (behavior-verified by the headless drives + the live drive).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Inspect (Phase 3.5)
5 parallel critics (general-purpose) over the diff — lenses: pure-engine correctness, concurrency/state,
reuse/guards/provenance, security/resource, simplification. Verdicts + fixes below.

### CONFIRMED → FIXED at source
- **[MED · C4 security] `run_search` read the whole file BEFORE the size gate.** `std::fs::read` pre-sizes
  its buffer from metadata → a multi-GB non-gitignored file (a CSV/sqlite/blob in a non-git dir) → a
  multi-GB transient alloc that ABORTS the process on OOM; plus a TOCTOU fifo/device hang (`is_file` at
  enumeration, read later). The repo already fixed this exact antipattern in `load_code_view_state`
  (stat-first, the #106 lesson). **Fix:** `std::fs::metadata` gate (`is_file` + `len <= VIEWER_MAX_BYTES`)
  BEFORE `fs::read`, keeping the post-read size check as a TOCTOU backstop. (app.rs `run_search`.)
- **[MED · C2 concurrency] Non-respawning worker = permanent silent breakage.** `search_worker` was set to
  `Some` once and never reset, so if the thread ever died, every future ⌘⇧F sent into a dead channel and
  the picker showed nothing forever. **Fix:** drop the handle (`search_worker = None`) when `tx.send` errs,
  so the next launch respawns. (Latent today — no panic surface — but removes the whole class.)
- **[MED · C2 + C5] `search_request` / `SearchQuery` were write-only dead state.** Mirrored #325's
  `symbol_request`, but #326 gates on the scalar `search_gen` via `accept_gen` — the field was never read
  (a needless `query.clone()`, and a `= None` "reopen protection" the `search_gen += 1` bump actually
  provides). **Fix:** deleted the field + its 5 sites + the `SearchQuery` struct; `search_gen` is the single
  source of truth.
- **[MED · C5 simplification] `apply_search_msg` reallocated the whole match Vec** (`into_iter().take().
  collect()`) even in the common keep==len case. **Fix:** `matches.truncate(keep)` in place.
- **[LOW-MED · C2 + C4] `walk_text_files` materialized an uncapped `Vec<PathBuf>` with no mid-walk cancel.**
  On a giant monorepo, the whole tree enumerated (100+MB) per debounced keystroke before the first gen
  check. **Fix:** return `impl Iterator` — the worker consumes it lazily, so a superseded walk abandons
  mid-enumeration and no whole-tree Vec is built. (Capping would be wrong — search must visit all files.)
- **[LOW · C1] CRLF files left a trailing `\r` in every `preview`** (cosmetic). **Fix:** `strip_suffix('\r')`
  before matching + preview (the `\r` is always last, so `col`/`len` are unaffected).

### CONFIRMED → deferred to VALIDATE (test coverage, not code bugs — mid-pipeline state)
- **[MED · C3] The pure seams + the 3 test hooks are untested (dead-code warning).** Expected — implement
  built the hooks, validate wires the tests. Validate MUST: unit-test `search.rs` + `editor_search.rs` to
  cov/MSI 100, and drive the app glue via `open_search_for_test`/`push_search_msg_for_test`/`search_for_test`
  (match / stale-gen / cap cases) — which also clears the `never used` warning before the `-D warnings` gate.
- **[VALIDATE note] `line_matches`' `nlen == 0` guard is unreachable via the public API** (search_lines
  early-returns on empty needle), so a delete-`nlen==0`-operand mutant survives UNLESS a direct module test
  calls `line_matches` with an empty needle (the mutant then infinite-loops → killed by timeout). Add it.
- **[LOW · C5] `case_sensitive`/`whole_word` are hardcoded false in the app** (v1 — no UI toggle), so the
  `window_eq(case_sensitive=true)` + `is_whole_word` branches are reachable ONLY from engine unit tests —
  validate must cover both directly in `search_lines` units.

### CONFIRMED → named FOLLOW-UPS (out of v1 scope)
- **[LOW · C1 + C4] preview clamped from line start** — a match beyond `max_preview_chars` (500) renders with
  an empty highlight (still a correct row + caret jump). Follow-up: window the preview around `col`.
- **[LOW · C4] override branch has no size re-gate** — a buffer grown past 2 MiB via paste AFTER open isn't
  re-bounded. User-driven, acceptable v1.
- **[LOW · C2] overrides staleness on a non-keyboard (LSP-applied) edit during streaming** — the modal blocks
  keyboard buffer edits while open, so only an LSP-applied edit mid-stream could stale. Acceptable v1.

### REJECTED / non-issues (with reason)
- **C1: ASCII-fold `é≠É`, multi-line needles can't match, preview-from-start** — documented, deliberate v1
  limitations (Unicode case-fold + regex are named follow-ups; the engine is line-oriented).
- **C5: hoist the per-line needle `Vec<char>`** — skipped: cheap (short needles), and it entangles the
  `nlen==0` guard's mutation story; left as-is with the validate note above.
- **C5: `Cow` the override text; `Done => false`** — skipped: the clone is bounded by open-file count (1-5);
  the `Done` repaint is a harmless once-per-search settle.
- **C3-INFO: the new off-thread worker** — a justified extension (mirrors the #274 syntax worker; pure
  decisions extracted, IO/thread body `mutants::skip`'d, gen protocol sound). No defect.

**Post-fix:** `cargo check --workspace --all-targets` clean (only the expected 3-test-hooks-unused warning,
which validate clears, + the pre-existing `block v0.1.6`); `cargo fmt` clean.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate
### Tests added (28)
- **marley_project::search — 15 pure units** (cov/MSI 100): multi-match char-offset cols, non-ASCII col,
  cross-line rows, empty/too-long needle, non-overlapping advance, miss-then-match (advance-by-one), case
  toggle (ASCII fold both directions), whole-word boundaries (incl `_`), per-file cap→truncated, CRLF `\r`
  strip, preview clamp (col stays true), the DIRECT `line_matches` empty-needle test (kills the delete-
  `nlen==0` mutant via timeout), `window_eq`, `is_whole_word`, `is_word_char`.
- **marley_app::editor_search — 5 pure units** (cov/MSI 100): `accept_gen`, `locate_match` (incl zero-count
  file + past-the-end), `admit_matches` (under/straddle/at/over the total cap), `footer_summary`
  (singular/plural + "+K more"), `visible_matches`.
- **marley_project walker — 1 integration test** (tempdir + tempfile): a `.gitignore`'d file excluded (a
  `.git` marker makes `ignore` honor it), `.git`/`target`/`node_modules` dropped, a plain + nested `.rs`
  present, files-only — over the NOW-LAZY `walk_text_files` iterator.
- **marley_app — 7 headless drives** (real RootView, no GUI): open + Esc close (REQ-001); per-file merge +
  total (REQ-004); stale-gen result dropped (REQ-007); MAX_FILES cap → overflow dropped (REQ-006); ↓ past
  the visible cap clamps to the last visible match, cap == nav (REQ-010); Enter → jump + NavStack push
  (REQ-005/008); opening ⌘⇧F dismisses a live completion popup (the #325 lesson). Clears the 3-hook warning.

### Gate — GREEN [diff]
`scripts/gates.sh --diff` → **`GATE GREEN [diff]`, 15 passed / 0 failed**. gate:4 coverage **100% lines**,
gate:5 mutation **MSI 100%** (the `search`/`editor_search` pure seams), miri + visual green. Receipt written.
Two reds fixed at source before green (§0, no suppressions):
- **gate:5** — `search_overlay` was missing `#[cfg_attr(test, mutants::skip)]` (its 12 render-math mutants
  survived; the app.rs render shims are mutation-excluded by that ATTRIBUTE, not a file rule — `symbols_
  overlay`/`def_picker_overlay` both carry it). Added it.
- **gate:14** — the brand-scrub flagged the word "Zed" in a keymap comment (the repo keeps 0 Zed mentions in
  source; §20 keeps it a docs-only reference). Reworded to "the standard find-in-files chord".
- (Also: the freshly-written test code was rustfmt-dirty — `cargo fmt --all` fixed it; the first gate run had
  to be killed + re-run on clean source since fmt-while-mutation-compiles corrupts the run.)

### LIVE DRIVE — ENV-BLOCKED (locked screen), fell back to units + mechanism
The screen was **LOCKED** (`CGSSessionScreenIsLocked=1`, `onconsole=1` — chad AFK, same as #324/#325). A
locked mac fully blocks synthetic CGEvents + `screencapture` (`PR-claude-selftest-locked-screen-blocks-
capture-fall-back-to-mechanism`); I did NOT attempt a password. **Coverage without the live drive** (the
honest #204/#205/#324/#325 fallback): the 7 headless drives exercise the REAL RootView through the actual
open→park→merge→nav→Enter→jump path (including the stale-gen drop, the cap==nav clamp, and the completion-
dismiss inspect fix); the render reuses the #325 `symbols_overlay` primitives (`menu_origin`/`popup_window`/
`truncate_cols`) + the cap==nav rule (critic-3-verified); the jump reuses #312's **live-proven**
`open_and_place_caret` + NavStack; and the gitignore-aware walk is proven by the tempdir integration test.
**Re-run the ⌘⇧F live drive when the screen is unlocked** (~2 min via the reusable harness: seed two probe
files with a planted needle → ⌘⇧F → both rows grouped → Enter lands centered → edit one match away unsaved →
re-search finds N-1, the dirty-buffer proof) — a confidence top-up, no ticket.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **CHANGELOG** — #326 entry above #325 (Added): the ⌘⇧F content-search picker, the pure `search_lines`
  engine, the `ignore`-crate lazy walker (one new crate), the off-thread worker + generation-cancel +
  dirty-buffer-wins, the `editor_search` glue + caps, the Editor-scoped ⌘⇧F shadow.
- **Architecture docs** — `editor.md` gained a "Project-wide content search (#326) SHIPS" paragraph after the
  #325 workspace-symbols block. `crate-map.md`: the `marley_project` row now names the `search` engine +
  `walk_text_files` + the `ignore` dep (its first real dependency), status "🟢 PURE (+ masked walk shim)".
- **Knowledge captured (forge wired §19)** — AAR `206fb9a2` CLOSED (`completed`, effectiveness 5, 5 novel
  findings; jobs distillation + confidence_drift + pattern_emergence). The 3 failures + 2 prevention rules
  were recorded at inspect: `BF-search-read-whole-file-before-size-gate-001`,
  `BF-spawn-once-worker-never-respawns-001`, `BF-copied-stale-key-field-never-read-001`;
  `PR-claude-spawn-once-worker-drop-handle-on-send-error-001`,
  `PR-claude-copied-stale-key-must-be-read-or-deleted-001` (the stat-first fix ALSO reinforced the existing
  `PR-stat-before-read`). **AD recorded** `AD-claude-project-search-adopt-ignore-crate-001` (`8660d893`) —
  the D-WALKER decision: adopt `ignore` because its whole transitive closure was already in-lock (one crate
  for gitignore-correctness); LESSON = quantify a dep's MARGINAL cost against the actual lock before
  rejecting it on "weight".
- **Ticket closed** — forge #326 (`992a1da5`) → `done`; local `TICKET-326` status → closed, moved to
  `tickets/closed/`.
- **Pipeline archived** — this doc pair moved to `pipeline/completed/`.

status: Phase 5 — Complete PASS
