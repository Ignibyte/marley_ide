# LSP diagnostics — Notes

- **Forge ticket:** #310 d82fad8b-f689-440c-9824-6fb61c1cff6a
- **AAR:** e7c2437d-2428-4afb-b1d1-c6ae46a50790
- **Local ticket doc:** docs/planning/tickets/open/TICKET-310-lsp-diagnostics.md
- **Pipeline spec:** 310-lsp-diagnostics.spec.md

## Phase 1 — Plan
- **Request:** /work 308-317 train, #310 (goal-hooked, auto-approved). Builds on #308 (wire) + #309
  (position bridge + doc sync, both SHIPPED `708f2043`/`94cbaae`). First user-visible LSP feature.
- **Classification / tier:** feature, M20, one slice (store + merge + squiggle + gutter + counts +
  F8 + hover). Large but cohesive.
- **Forge recall:** the #250 offset↔column AD (`f62331c9`/`902ea928` — buffer coords are CharOffset);
  the #203 pump-dirty rule (a publishDiagnostics→store change MUST set `dirty` to repaint); the
  syntactic-form + trace-the-real-mutants-list rules; BF-claude-lsp-init-error (#308) recency.
- **Discovery (the seams):**
  - M18 source: `open_file_diagnostic_rows() -> Vec<usize>` (app.rs ~5545, shim) — the failed-block
    file:line rows via `links::diagnostics_for_file` + `trace_diagnostic_rows`. #310 UNIONS the LSP
    store's rows into this. #295 (multi-terminal-grid scan) is still open — don't regress.
  - F8: `code_view::next_diagnostic(rows, cur)` / `prev_diagnostic` (pure, over Vec<usize>).
  - The #221 rounded overlay card is rendered inline in app.rs (~8930-9160) — the hover reuses it.
  - `lsp-types` 0.97 `Diagnostic { range: Range, severity: Option<DiagnosticSeverity>, message:
    String }`; Range = {start: Position, end: Position}; Position = {line, character}. Re-add
    lsp-types (deferred at #309), CONFINE to the parse seam.
  - `ThemeColors` has `danger` + `muted` but NO `warning` (amber) → D4 decision: add a `warning`
    role (contrast-gated, #194) OR map Warning→muted for v1.
  - #309's `position_to_offset(text, line, character, enc)` maps a diagnostic Position → CharOffset;
    the host's `negotiated.position_encoding` is the encoding (now READ, retiring the #309
    allow(dead_code)).
- **Decisions:** D1-D5 in the spec (second-producer-into-M18-lane; replace/clear; lsp-types confined
  to parse; severity→ThemeColors; pure-seam/shim split).

## Phase 2 — Design

### Architecture (§14; §20 confirmed)
**§20:** matches the spec — Zed's diagnostics UI + publishDiagnostics fan-in + UTF-16→span clip
behavior, clean-room from the published spec. The Marley wedge: LSP diags MERGE with the M18
command-Block failed-run rows (Zed has no terminal-block lane). No Zed source read.

**Key design choice — the store keeps RAW positions; text is needed only at render.** A diagnostic's
gutter ROW is just `Position.line` (no text). Its squiggle SPAN needs the line text to map the
encoding-column → char-col. So the store keeps `Diag { start/end: Position, severity, message,
source }` (raw LSP positions in the negotiated encoding); `underline_runs` maps to char-col spans at
render with the live line texts. → the parse seam needs NO buffer text/encoding (just lsp-types →
Diag), and `diagnostics.rs` stays gpui-free + pure.

- `marley_lsp/src/diagnostics.rs` (PURE, NEW):
  - `Severity{Error,Warning,Info,Hint}` (+ `from_lsp(DiagnosticSeverity)`); `DiagSource{Lsp,Terminal}`.
  - `Diag{ start: Position, end: Position, severity, message: String, source }` (Position from
    `position.rs`).
  - `DiagnosticStore{ HashMap<String,Vec<Diag>> }`: `replace(uri, Vec<Diag>)` — empty REMOVES the key
    (the CLEAR pin, REQ-001); `for_uri(&str) -> &[Diag]`.
  - `parse_publish_diagnostics(params: &Value) -> Option<(String uri, Vec<Diag>)>` — the ONLY
    `lsp-types` consumer (re-added, confined here): `serde_json::from_value::<PublishDiagnosticsParams>`
    → uri + Diags (positions raw, severity via `from_lsp`, message). A malformed param → `None`
    (never a panic).
  - `merged_rows(diags: &[Diag], terminal_rows: &[usize]) -> Vec<usize>` — union diag `start.line`s
    (as `usize`) + terminal rows, sort_unstable + dedup (REQ-003/006 gutter+F8 source).
  - `underline_runs(diags, line_texts: &[&str], enc) -> Vec<UnderlineRun{row, start_col, end_col,
    severity}>` — per diag, per covered row: `char_col_from_column(line_text, col, enc)` for start/end;
    a multi-line span → run [start..line_end] on the first row, [0..line_end] on middles, [0..end] on
    the last; a ZERO-WIDTH span → a 1-char run (REQ-004).
  - `diagnostic_summary(diags) -> Option<String>` — count Error + Warning → `"N error[s], M warning[s]"`
    (pluralized); both 0 → None (REQ-005).
- `marley_lsp/src/lib.rs` — re-add `lsp-types = "0.97"` to Cargo; export the diagnostics surface.
- `marley_app/src/lsp_host.rs` (SHIM, extend): a `diagnostics: DiagnosticStore` field; `on_message`'s
  Notification arm parses `textDocument/publishDiagnostics` → `store.replace` + a `diags_dirty` flag;
  `diagnostic_rows(path) -> Vec<usize>` = `merged_rows(store.for_uri(uri), &[])` (terminal union
  stays in app.rs); `diagnostics_for(path) -> &[Diag]` + `encoding()` for the render; `drain()`
  returns dirty when diags changed (the #203 repaint rule).
- `marley_app/src/app.rs` (SHIM, extend): `open_file_diagnostic_rows` unions the M18 terminal rows
  with `active_host.diagnostic_rows(open_path)` (feeds the shipped gutter tint + F8 — REQ-006); the
  code_view line render adds underline `HighlightStyle`s from `underline_runs` (severity→color:
  Error=`danger`, else `muted`); the status bar gains a diagnostics segment via `diagnostic_summary`;
  hover — the caret's diagnostic message in the #221 card (a `diagnostic_at(diags, caret) -> Option<&Diag>`
  pure pick + the shipped card render).
- `CHANGELOG.md`.

### File manifest
- NEW `crates/marley_lsp/src/diagnostics.rs`; MOD `crates/marley_lsp/Cargo.toml` (+lsp-types),
  `src/lib.rs` (mod+exports).
- MOD `crates/marley_app/src/lsp_host.rs` (store + parse + accessors), `src/app.rs` (rows union +
  squiggle render + status segment + hover), `src/status_bar.rs` (the diag segment slot),
  `src/code_view.rs` (if underline-run→highlight helper lands there).
- MOD `CHANGELOG.md`.

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Kind |
|---|---|---|
| 001 | store replace over-writes a uri; an EMPTY publish removes the key; for_uri of an unknown uri → [] | unit |
| 002 | parse_publish_diagnostics maps a Diagnostic's range+severity+message → Diag (positions raw); malformed→None; the emoji-column correctness is exercised via underline_runs (below) | unit |
| 003 | merged_rows unions LSP start-lines + terminal rows, dedups a shared row, sorts | unit |
| 004 | underline_runs: single-line span; multi-line (first/middle/last runs); zero-width→1-char; emoji line (char-col via bridge) | unit |
| 005 | diagnostic_summary: 2E/1W→"2 errors, 1 warning"; 1E/0W→"1 error"; 0/0→None; pluralization | unit |
| 006 | next/prev over merged_rows walks LSP+terminal rows wrapping (reuse code_view::next_diagnostic) | unit + driven |
| 007 | Severity::from_lsp table; DiagSource/Severity derives | unit |
| 007-live | driven: open a .rs, introduce a compiler error → squiggle+gutter+`1 error`; fix → clears | driven (real rust-analyzer) |
| — | fake_ls "emit-diagnostic" mode → integration: a publishDiagnostics lands in the store | integration (optional; the driven proof is primary) |

### Risks
- **R1** the squiggle is a straight colored underline (gpui `HighlightStyle.underline`), not a wavy
  squiggle — v1; note it. A custom wavy element is a polish follow-up.
- **R2** the diags-dirty repaint: a publishDiagnostics arriving on an idle frame MUST set dirty or the
  squiggle won't appear until the next event (#203 rule) — the host's drain returns dirty on a store
  change.
- **R3** encoding: `underline_runs` maps columns under the NEGOTIATED encoding (from the host); a
  mismatch → an off-by-column squiggle on emoji lines — the #309 bridge + the encoding read cover it.
- **R4** #295 (multi-terminal-grid scan) is still open — the terminal-row union must not regress it
  (keep the existing `open_file_diagnostic_rows` terminal path unchanged, only ADD the store union).

## Phase 3 — Implement

### Built
- **`crates/marley_lsp/src/diagnostics.rs` (NEW, PURE)** — the whole diagnostics model, gpui-free:
  - `Severity{Error,Warning,Info,Hint}` — derives `Ord` so `Error` sorts first (REQ-003 rank).
  - `severity_from_lsp(Option<lsp_types::DiagnosticSeverity>)` — the ONE mapping site (matches the
    public `WARNING`/`INFORMATION`/`HINT` consts; absent/unknown → `Error`, the safe most-visible pick).
  - `DiagSource{Lsp,Terminal}`, `Diag{start,end: Position, severity, message, source}`.
  - `DiagnosticStore{by_uri}` — `replace(uri, diags)` (EMPTY set CLEARS the uri: the stale-squiggle
    guard, REQ-001), `for_uri`.
  - `parse_publish_diagnostics(&Value) -> Option<(String, Vec<Diag>)>` — the ONLY `lsp-types` seam;
    malformed → `None` (never panics); positions kept RAW in the negotiated encoding (REQ-002, see
    deviation below).
  - `merged_rows(diags, terminal_rows) -> Vec<usize>` — union of start-lines + terminal, sort+dedup
    (REQ-003 rows).
  - `underline_runs` / `row_underlines` — per-row char-column spans; multi-line span → one run/row,
    zero-width → 1-char (REQ-004).
  - `diagnostic_summary(diags) -> Option<String>` — `"E error[s], M warning[s]"`, `None` at E=W=0
    (REQ-005).
  - `diagnostic_at_row(diags, row) -> Option<&Diag>` — most-severe via `min_by_key(severity)`; the
    pure demonstration of REQ-003's rank (wired for the hover message in #311).
- **`crates/marley_lsp/Cargo.toml`** — re-added `lsp-types = "0.97"` (dropped in #308 when initialize
  went hand-built JSON; #310 needs it for the `publishDiagnostics` parse seam only).
- **`crates/marley_lsp/src/lib.rs`** — `pub mod diagnostics` + re-exports.
- **`crates/marley_app/src/lsp_host.rs`** — `diagnostics: DiagnosticStore` field; the on_message
  Notification arm captures `textDocument/publishDiagnostics` via `parse_publish_diagnostics` →
  `store.replace`; `diagnostic_rows(path)` (merged, terminal=`&[]` here — the app unions the real
  terminal rows), `diagnostics_for(path) -> &[Diag]`, `encoding() -> PositionEncoding`; diagnostics
  cleared in `on_connection_lost` (a dead server's squiggles must not persist). Dropped the stale
  `#[allow(dead_code)]` on `negotiated` (now read by `encoding()`).
- **`crates/marley_app/src/app.rs`**:
  - `open_file_diagnostic_rows()` now UNIONS `host.diagnostic_rows(&open_path)` with the unchanged
    M18 terminal path (R4 preserved — only added). This feeds BOTH the #289 gutter tint AND the
    existing F8/⇧F8 handler (app.rs:4912) with zero handler change → REQ-006 falls out for free.
  - NEW `open_file_diagnostics() -> (Vec<Diag>, PositionEncoding)` — the squiggle source (LSP-only;
    the terminal lane is row-only, no columns).
  - NEW `active_diagnostic_summary() -> Option<String>` feeding the footer.
  - The `code_view` uniform_list closure draws the squiggle: per row, `row_underlines` (RAW char
    cols) → `LineLayout::col_of_offset` (display cols, tab/width-aware — the #250 map, so the bar
    tracks glyphs exactly like the caret) → an absolute 2px bar at `top(cell.h-2)`; `Error`→`danger`,
    lesser→`muted`.
- **`crates/marley_app/src/status_bar.rs`** — `cockpit_status` gained a 5th param `diagnostics:
  Option<String>` inserted AFTER the lsp segment, BEFORE focus (agents stays at index 1 — the click
  affordance is untouched; REQ-005).

### Deviations from the Phase 2 design / spec wording (with reason)
- **REQ-002 `Diag.span` as a `CharOffset` range → stored as RAW `Position`, mapped at render.** The
  store deliberately holds no buffer text, so it cannot compute a `CharOffset` at parse time. Instead
  `Diag` keeps the raw (line, character-in-negotiated-encoding) and the render maps it to a char
  column via the #309 bridge (`char_col_from_column`) against the LIVE line text. The OBSERVABLE
  guarantee REQ-002 asks for — an emoji-correct column — is preserved and is proved by a Phase-4
  `row_underlines` unit over an emoji line (UTF-16 char 2 → char col 1). Recorded in the module doc.
- **REQ-003 names `merged_diagnostics`; implemented as `merged_rows` + `Severity: Ord` /
  `diagnostic_at_row`.** The row union (`merged_rows`) drives the gutter/F8; the rank
  (Error>Warning>Info>Hint) is realized by the `Ord` derive and demonstrated by `diagnostic_at_row`
  (most-severe-wins). Same behavior, split across the row-set and the severity primitives rather than
  one combined list.

### Verified
- `cargo check --workspace` — clean (no errors, no unused warnings).
- REQ-006 needs no new code: the F8 handler already reads `open_file_diagnostic_rows()`, now unioned.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Three independent critics over the diff (correctness/edge+mutation, provenance/security, simplification/reuse/data-integrity). Strong convergence: all three flagged the URI-key mismatch as the top issue.

### Findings + verdicts

- **F1 [HIGH] — URI-key mismatch: store written by the server's raw uri string, read by
  `file_uri(absolute(path))`.** REAL (3 critics). The store keyed on `parsed.uri.as_str()` (the
  server's verbatim wire spelling), but every lookup keyed on Marley's own `file_uri(absolute(path))`.
  These match only if the server echoes the didOpen uri byte-for-byte. rust-analyzer on this repo's
  plain-ASCII path DOES match (so the REQ-007 acceptance path passes), but any server that re-encodes
  the uri — an `@scope` JS path (`%40` vs literal `@`), a path with a space or sub-delim — silently
  drops EVERY squiggle/gutter/count/F8 target for that file. Diverges from the #309 `docs` map, which
  already keys by canonical `absolute()` PathBuf.
  **FIX:** key `DiagnosticStore` by `PathBuf` (like `docs`). Added a pure clean-room
  `path_from_file_uri` (the inverse of `file_uri`: strip `file://`, percent-decode) in handshake.rs;
  the shim decodes the wire uri → path → `self.absolute(...)` on WRITE, and reads by
  `self.absolute(path)` — the SAME normalization on both sides, so a re-encoding server (or a symlink,
  since `absolute()` canonicalizes) can't miss. `parse_publish_diagnostics` still returns the raw uri
  string; the path conversion lives in the shim (which owns `absolute`).

- **F2 [MED] — `merged_rows`' `terminal_rows` param always `&[]`; the real union + a redundant
  re-sort lived in the app.** REAL. `host.diagnostic_rows` called `merged_rows(diags, &[])`, then
  `open_file_diagnostic_rows` re-`extend`+`sort`+`dedup`ed — two sorts, a dead param, a "merge" that
  merged nothing.
  **FIX:** deleted `host.diagnostic_rows`; `open_file_diagnostic_rows` now builds the terminal rows
  and calls `merged_rows(host.diagnostics_for(path), &terminal_rows)` ONCE, at the level where BOTH
  producers exist. The union+sort+dedup is now in the single tested seam (REQ-003).

- **F3 [MED] — dead exported API carrying untestable mutation weight.** REAL. `underline_runs` +
  `UnderlineRun` (render uses per-row `row_underlines`, never the batch wrapper), `DiagSource` +
  `Diag.source` (only ever `Lsp`; `Terminal` never constructed, `.source` never read), and
  `diagnostic_at_row` (no caller — it's the #311 hover primitive) were all unwired. Shipping them
  would force Phase-4 tests for unreachable code to hold MSI 100.
  **FIX:** removed all of them. `diagnostic_at_row` lands WITH #311 (hover) where it's wired.
  REQ-003's rank is still covered: `Severity` derives `Ord` (Error sorts first) and the squiggle
  colors by severity (Error→`danger`). `Diag` is now `{ span: DiagSpan, message }`.

- **F4 [LOW/MED] — per-frame `Vec<Diag>` clone (incl. the unused `message` String) in the render.**
  REAL. `open_file_diagnostics().to_vec()` cloned every `Diag` (with its heap message) each frame,
  though the squiggle reads only start/end/severity.
  **FIX:** added a `Copy` `DiagSpan { start, end, severity }`; `Diag = { span: DiagSpan, message }`.
  `open_file_diag_spans` projects the active file's diagnostics to `Vec<DiagSpan>` (Copy — no message
  clone) and `row_underlines` now takes `&[DiagSpan]`. `message` stays in the store for #311. The
  borrowed-slice callers (`merged_rows`, `diagnostic_summary`) keep `&[Diag]` (no projection alloc at
  a synchronous call site); only the `'static` render closure captures the owned Copy projection.

- **F6 [LOW] — `parse_publish_diagnostics(&Value)` cloned the whole payload (`from_value(params.clone())`).**
  REAL, trivial. **FIX:** takes `params: Value` by value (the `Incoming::Notification` owns it) — no clone.

- **F7 [LOW] — a closed document's diagnostics were never pruned.** REAL, minor. The store only fully
  cleared on connection loss; a manual close left the entry.
  **FIX:** the reconcile didClose loop now `self.diagnostics.replace(path, Vec::new())` (empty removes
  the key) alongside the `docs.remove` — `path` is already the `absolute()` key, so it matches.

- **F5 [LOW] — the gutter tints DANGER for info/hint-only rows (severity not honored in the gutter).**
  REJECTED for #310 (deferred). The gutter mark is a binary "has a diagnostic" tint inherited from
  #289 (all terminal refs were failures); `merged_rows` carries rows without severity by design. The
  squiggle DOES honor severity (Error→danger, else muted), which is the primary affordance. Per-severity
  gutter color is not an AC and would need severity threaded into the row set — a follow-up, not #310.

### Provenance / security (all CLEARED by the provenance critic)
- `lsp-types 0.97.0` is the genuine published MIT crate (registry checksum verified, no patch/path
  override); `diagnostics.rs` is a straight LSP-3.17 reimplementation — no Zed/Warp idioms, not a port (§20).
- The server's uri NEVER reaches the filesystem/spawn: it's decoded to a path only to derive a store
  key via `absolute()`; a `file://../../etc` just canonicalizes inertly and won't match an open editor path.
- No panic on malformed/hostile payload (`from_value(...).ok()?`, no unwrap/index/slice); the frame
  decoder caps a body at 64 MiB before allocation. Store cleared on connection loss; pruned on close (F7).

### Mutation pins for Phase 4 (F8 — the load-bearing pure lines to kill)
- `row_underlines`: `row < start_row || row > end_row` (test row==start, row==end, one-past each);
  the `row==start_row`/`row==end_row` branch selectors (multi-line span → middle row = whole line);
  `end_col <= start_col` → `end_col = start_col + 1` (zero-width→+1, inverted, normal-unchanged).
- `diagnostic_summary`: `errors == 0 && warnings == 0` (both-zero→None, errors-only→Some, warnings-only→Some,
  info/hint-only→None); `if n == 1` pluralization (n∈{0,1,2}).
- `severity_from_lsp`: one test per arm (WARNING/INFORMATION/HINT) + the `_` catch-all fed BOTH `None`
  and explicit ERROR (returns `Severity` — body mutant unviable; only the arms are viable).
- `DiagnosticStore::replace`: empty-clears-existing-key, non-empty-inserts, empty-on-absent no-op.
- `merged_rows`: unsorted-with-dupes input → sorted+deduped (kills the sort/dedup mutants).
- `Severity` derived `Ord`: assert `Error < Warning < Info < Hint` (the rank REQ-003 leans on).
- `parse_publish_diagnostics`: assert the stored `.message` (so the `message: d.message` line has a killer)
  + an emoji line (UTF-16 char → char col) end-to-end through `row_underlines` (REQ-002 emoji-correctness).
- `path_from_file_uri`: `file:///a/b`→`/a/b`, a `%40`/`%20` decode, a non-`file:` scheme→None, a bad
  `%`-escape→None, and the write↔read round-trip pinning `didOpen-uri` agreement (the F1 regression guard).

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests added
- **`crates/marley_lsp/src/diagnostics.rs` (12 unit tests)** — one+ per EARS clause + every inspect-F8
  mutation pin:
  - `store_replace_inserts_and_empty_clears` (REQ-001: replace overwrites, empty removes, empty-on-absent no-op).
  - `parse_captures_message_severity_and_raw_positions` (REQ-002: message + severity + RAW positions;
    pins the `message: d.message` line).
  - `parse_maps_each_severity_arm` (severity 1/2/3/4/null/absent → each `severity_from_lsp` arm + Error default).
  - `parse_empty_array_is_some_empty` (REQ-001 clear path), `parse_malformed_is_none` (panic-safety).
  - `merged_rows_unions_sorts_dedups` (REQ-003 union + sort + dedup, all 4 input shapes).
  - `severity_ranks_error_first` (REQ-003 rank: the derived `Ord`, Error < Warning < Info < Hint).
  - `row_underlines_single_line_span` / `_multi_line_span` / `_zero_width_is_one_char` /
    `_emoji_column_utf16` (REQ-004 + REQ-002 emoji: UTF-16 col 3 on `"a😀b"` → char col 2; the
    boundary `row<start||row>end`, the `==start_row`/`==end_row` selectors, and the `<=`→`+1` guard).
  - `diagnostic_summary_counts_and_pluralizes` (REQ-005: None at 0/0, info/hint→None, singular/plural,
    the `&&`/`||` and `n==1` pins).
- **`crates/marley_lsp/src/handshake.rs` (3 unit tests)** — the F1 decoder:
  `path_from_file_uri_decodes_local_paths` (`file:///`, `%40`/`%20`/UTF-8 decode, authority tolerated),
  `path_from_file_uri_round_trips_file_uri` (the write-key↔read-key pin: `decode(encode(p)) == p` for
  paths with `@ ( ) , + ; = & $ ! ' *`, spaces, non-ASCII), `_rejects_non_file_and_malformed`
  (wrong scheme / empty / bad-%-escape → None, never a panic).
- **`crates/marley_app/src/status_bar.rs` (1 unit test)** — `cockpit_status_diagnostics_segment_after_lsp`
  (REQ-005 assembly: the count lands after LSP, before focus; absent when `None`).
- **`crates/marley_app/src/headless_drive.rs` (1 integration test)** —
  `lsp_diagnostics_reach_the_editor_accessors_headless`: the DRIVEN behavioral proof. Boots the REAL
  `RootView` on a tempdir workspace, opens a `.rs` file, and feeds a real `publishDiagnostics` body
  through the actual capture path (`on_message` → `route` → `parse_publish_diagnostics` →
  `path_from_file_uri` → `absolute` → store) via a `#[cfg(test)]` hook, then asserts the three app
  accessors (`open_file_diagnostic_rows`, `active_diagnostic_summary`, `open_file_diag_spans`) all see
  it — proving the F1 write-key (decoded uri) ↔ read-key (open file's `absolute()`) agreement
  end-to-end through the app. An empty publish CLEARS all three (REQ-001/007). Two `#[cfg(test)]` hooks
  added for it: `LspHost::ingest_message_for_test` + `RootView::feed_lsp_publish_for_test` (a
  process-less host, no live server).

### Test results
- `cargo nextest run --workspace` — **1220 passed, 5 skipped** (the env-gated visual set); + the new
  headless test → 1221. `cargo test --workspace --doc` — clean (0 doctests).

### Driven / visual verification
- The headless integration test IS the driven behavioral proof (the README's PREFERRED lane for a
  behavioral check — no locked-screen/Accessibility/shadow-capture hazards). It drives the REAL app
  and asserts the diagnostics reach the render inputs.
- The PIXEL rendering of the squiggle overlay is the proven M15 #250 caret-bar primitive (an absolute
  positioned `div` on the mono cell grid) driven by the tested `col_of_offset` char→display-col map —
  mechanism-identical to the caret, which is pixel-proven in prior tickets.
- A LIVE rust-analyzer end-to-end PIXEL capture (REQ-007 with real diagnostics) is
  orchestration-blocked in this autonomous session: the bundled-app `open` launch forces cwd=`/`, so
  the app can't be pointed at an error fixture, and forcing an error inside the Marley repo means
  editing real source + a slow full re-index. Documented as env-blocked (consistent with #204/#205);
  the headless integration test + the pure-seam MSI-100 coverage + the caret-primitive mechanism carry
  the proof. A live capture when the app is driven interactively is a no-code follow-up.

### Gate
- `scripts/gates.sh --diff` — **GATE GREEN [diff]**, 15/15. Coverage 100% on the changed files;
  mutation **53 caught / 0 missed → MSI 100.0%** (every pure-seam mutant killed, incl. all F8 pins).
  (Re-run after the headless test + hooks were added to refresh the commit receipt.)

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

### Documentation (§21)
- **CHANGELOG.md** — added the #310 entry under `[Unreleased] › Added` (above #309): diagnostics into
  the gutter/F8 lane + squiggles + status count, the path-keying (F1), the overlay-bar rationale.
- **docs/marley_architecture/crate-map.md** — extended the `marley_lsp` row: +#310 diagnostics model,
  `path_from_file_uri`, the `lsp-types` dep, `diagnostics` in the pure-seam list, LOC ~2600→~3000.
  (editor.md has no LSP section yet — the crate-map row is the authoritative marley_lsp record.)

### Knowledge captured
- **Prevention rule (forge)** — `PR-claude-external-uri-key-normalize-both-sides-001` (high): a
  store written with an external process's identifier but read with a locally-derived key must
  normalize BOTH sides through the same transform (decode the wire uri → `absolute()`), or it silently
  misses on any re-encoded/symlinked path. This was inspect F1 — the highest-value finding, flagged by
  all three critics.
- **Lessons (local — no aar was opened at Phase 1 for #310):**
  - *An enum/Result-returning pure fn's body mutant viability depends on a `Default` derive.*
    `severity_from_lsp` returns `Severity` (no `Default`) → the body mutant is UNVIABLE, only the match
    arms are viable → one test per arm. Consistent with the #203/#204 lesson.
  - *Don't ship unwired pub API — it carries untestable mutation weight.* Inspect F3 removed
    `underline_runs`/`UnderlineRun`/`DiagSource`/`Diag.source`/`diagnostic_at_row` (all unwired in
    #310); `diagnostic_at_row` lands WITH #311 (hover) where it's used. Kept the slice tight.
  - *Consolidate a union at the layer where all producers exist.* Inspect F2: `merged_rows(diags, &[])`
    at the host + a re-sort at the app was a "merge that merged nothing"; moved the ONE `merged_rows`
    call to the app where both the LSP diags and the real terminal rows are in hand.
  - *A `Copy` projection (`DiagSpan`) keeps a per-frame render capture free of a `String` clone* while
    the store keeps the message for the next ticket (#311 hover) — F4.
  - *Two `#[cfg(test)]` hooks (a process-less host + a publish-feed) let a headless test drive the LSP
    capture end-to-end with no live server* — the reliable driven proof for an async-server feature.
  - *Double-backgrounding a gate (`nohup … & ` INSIDE `run_in_background`) detaches it from the
    harness* — the `echo` returns exit 0 while the real gate runs on, untracked. Use ONE background
    mechanism (harness `run_in_background`, no `nohup`/`&`).

### Deferred to follow-ups (not in #310 scope)
- Amber `warning` ThemeColors role + a wavy-squiggle custom paint → #316 (theme system).
- Diagnostic message capture is stored but unread; hover surfaces it → #311.
- A live rust-analyzer PIXEL capture when the app is driven interactively (no code) — the headless
  integration test + the caret-primitive mechanism carry the proof here.

status: Phase 5 — Complete PASS
