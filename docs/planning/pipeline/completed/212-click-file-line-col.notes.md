# Click a file:line:col → open the editor at that line — Notes

- **Forge ticket:** #212 `680aa96d-cb42-4a41-81a7-c5766e2fd73b`
- **AAR:** `e9aadbf3-f9e4-47f4-b0ab-1f4de35cdd4c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-212-click-file-line-col.md
- **Pipeline spec:** 212-click-file-line-col.spec.md

## Phase 1 — Plan
- **Request:** capture the `:line[:col]` #196 already strips off a clicked file ref,
  and place the editor caret there. The M18 fusion foundation (four of the six new
  #289-294 reuse this parser + open-at-line).
- **Classification / tier:** work pipeline, foundation slice, `feature`. Crate:
  `marley_app` (`links.rs` pure parser + `app.rs` shim). UI/input path → Validate
  drives a live capture (grep/cargo click → editor at line).
- **Forge recall (§18.3):** the intake `editor-as-peer-and-terminal-fusion.md`
  names this as the wedge; #196 built the scanner ANTICIPATING #212 (`strip_line_col`
  doc: "the line/col is #212's concern"); #190 `resolve_under_root` + the M15 editor
  (`OpenFile.caret`, `Buffer::line_start`/`line_col`) are the reuse.
- **Discovery (grounded this phase):**
  - `links.rs:51` `scan_links` → `path_link` (182) → `strip_line_col` (238) ALREADY
    does the 2-iteration trailing-`:digits` scan but returns only the path; `LinkTarget`
    (15) `File(PathBuf)` — the doc comment (19) pre-declares "#212 extends this to
    carry an optional `:line:col`."
  - `app.rs:2683` `open_link_target` — the `File(path)` arm calls
    `open_file_in_viewer(resolve_under_root(root, path))`; `7848` the render click
    passes the `LinkTarget`. `2743` `open_file_in_viewer` opens at the TOP (no caret).
  - `editor_surface.rs` `OpenFile.caret: CharOffset` (21); `active_caret()` reader
    (205) — needs a `set_active_caret` mutator. `buffer.rs` `line_start(row)` (164),
    `line_col(offset)` (156).
- **Decisions:** D1–D4 in the spec. Crux: the scaffolding exists — `parse_line_col`
  just captures what `strip_line_col` discards (same 2-iter logic, innermost group =
  col when two, else line); `caret_for_line_col` is 1-based→0-based clamped math.
- **Risk:** minimal. The parser change is a superset of `strip_line_col` (path
  detection unchanged → #196 links stay identical); the enum-variant change touches
  2 consumers; the caret placement is additive (no location → caret 0 = today).
  Windows drive-letter `C:` collides with the `:` separator — macOS-first, noted Out.

## Phase 2 — Design

### Approach
Capture the location `strip_line_col` already parses-then-discards, carry it on the
link, and place the caret on open. §20 confirmed (Zed/Warp file-ref click — our own
parser + arithmetic). Three small edits + one pure helper.

- **`parse_line_col(s) -> (&str, Option<usize>, Option<usize>)`** replaces
  `strip_line_col`. Same 2-iteration trailing-`:digits` scan; captures the groups.
  Collected innermost-first: 1 group → `(path, Some(line), None)`; 2 groups →
  `(path, Some(line), Some(col))` (the innermost/last-in-string is col, the outer is
  line — D1). Non-digit tail → stop (grep `file:line:` already ends at line since the
  match content is a separate whitespace token). `path_link` calls it and fills the link.
- **`LinkTarget::File { path, line, col }`** (struct variant; was `File(PathBuf)`).
  Consumers: `path_link` (build), `open_link_target` (read). `line`/`col` `Option<usize>`.
- **`caret_for_line_col(buffer: &Buffer, line: usize, col: Option<usize>) -> CharOffset`**
  (pure, marley_app over marley_editor `Buffer`): `row = line-1` (saturating; `line_start`
  clamps a past-EOF row); `char = col-1` (saturating, default 0); clamp `char` to the
  row's char length (`line_text(row).chars().count()` — col past EOL → line end); offset
  = `line_start(row) + clamped_char`. Lives in `code_view.rs` (caret geometry home).
- **`open_link_target` File arm** → resolve under root (#190, unchanged) → a new
  `open_file_at(path, line, col)` shim: `open_file_in_viewer(path)`; then if `line` is
  Some, `*active_editor.active_caret_mut() = caret_for_line_col(active_editor.active_buffer(), line, col)`
  (both accessors already exist — no new setter). No location → caret stays 0 (today's #196 behavior).

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/links.rs` | `strip_line_col` → `parse_line_col(&str) -> (&str, Option<usize>, Option<usize>)`; `LinkTarget::File(PathBuf)` → `File { path, line, col }`; `path_link` fills line/col. |
| `crates/marley_app/src/code_view.rs` | ADD pure `caret_for_line_col(buffer, line, col) -> CharOffset` (1-based line/col → clamped offset). |
| `crates/marley_app/src/app.rs` | `open_link_target` File arm → `open_file_at(resolve_under_root(root, path), line, col)`; ADD `open_file_at` shim (open_file_in_viewer + place caret via the active editor surface). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | links.rs `parse_line_col` truth-table: `a/b.rs:12:5`→(`a/b.rs`,12,5); `a/b.rs:42`→(_,42,None); `a/b.rs`→(_,None,None); a trailing non-digit tail (`a/b.rs:5:x` via a whitespace-split token) leaves both None. | REQ-001/002 |
| T2 | links.rs `scan_links` end-to-end: `--> src/main.rs:42:10` → a `File{line:42,col:10}` link over the path range; `(src/lib.rs:7)` parenthesized → `File{line:7}`; a bare `src/lib.rs` → `File{None,None}`; the existing #196 file/url tests stay green (updated to the struct variant). | REQ-001/002/004 |
| T3 | code_view.rs `caret_for_line_col`: `(line 2, col 3)` on a known 3-line buffer → the exact offset; col past EOL → line end; line past EOF → last line clamp; `col None`→ column 0; `line 1,col None`→ offset 0. | REQ-003 |
| T4 | Driven capture: `grep -n <token> <file>` (or `cargo build` with an error), CLICK the `path:line` in the block output → the editor opens with the caret at that line. | REQ-003 (live) |
| — | cov/MSI 100 on `parse_line_col` + `caret_for_line_col` via `--diff`. | REQ-005 |

Uncoverable: `open_file_at` is a `#[cfg_attr(test, mutants::skip)]`-class shim (open + set caret over the live editor surface) — covered behaviorally by T4; its correctness = T1+T3.

### Risks / decisions
- **R1:** the `LinkTarget::File` variant change touches every match on it — grep-confirmed only 2 (`path_link`, `open_link_target`) + the links.rs tests. A missed one fails to compile (not silent).
- **R2 (pre-existing, noted):** the #196 path heuristic (`contains('/')`) will still over-linkify a `path:line:no-space-content` token as a bogus path — unchanged by #212 (not made worse); a #196-heuristic limit, out of scope.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** `links.rs`: `LinkTarget::File{path,line,col}` struct variant + `parse_line_col` (innermost-first 2-group scan → `[Some(line),None]` / `[Some(col),Some(line)]` match) + `path_link` fills it. `code_view.rs`: pure `caret_for_line_col(buffer,line,col)` (1-based→0-based, `col` clamped to the row's char length, `line_start` clamps a past-EOF row) + the `marley_editor::Buffer` import. `app.rs`: `open_link_target` File arm → `open_file_at(resolve_under_root(root,path), line, col)`; the `#[cfg_attr(test, mutants::skip)]` `open_file_at` opens via `open_file_in_viewer` then `*editor.active_caret_mut() = caret_for_line_col(editor.active_buffer(), line, col)` (both accessors pre-existed).
- **Compile-fix (test helpers):** the two links.rs test helpers (`file`, `l`) built the old tuple `File(PathBuf)` → updated to the struct variant `{path, line:None, col:None}` (the #196 tests they back are location-less).
- `cargo check -p marley --all-targets` clean; `cargo fmt` clean; `cargo clippy -p marley` clean.

## Inspect (Phase 3.5)
Inline adversarial trace (small mechanical diff — parser + caret math + shim). **One real finding, fixed.**

| Finding | Verdict | Resolution |
|---|---|---|
| **F1 — clickable range only covered the filename** | **REAL (medium), FIXED** | The `Link.range` was `at..at + path.len()` (the #196 span), so clicking the `:12:5` of `foo.rs:12:5` did nothing — but the whole point of #212 is "click the ref → open at that line." Changed to `at..at + trimmed.len()` (the full ref minus the leading bracket + trailing punct). UNCHANGED for a location-less ref (`trimmed == path`), so #196 link-range tests stay green; only refs-with-locations get the wider clickable span. `BF-claude-fusion-click-range-covers-only-primary-subtoken`. |
| Parse 2-group ordering (col=innermost, line=outermost; 1-group=line) | SAFE | Traced `a:12:5`→(12,5), `a:42`→(42,None), `a`→(None,None); the `[Some(line),None]`/`[Some(col),Some(line)]`/`_` arms are exhaustive (`[None,Some(_)]` unreachable — slot 0 fills first). |
| Non-digit tail stop / usize overflow | SAFE | A non-digit tail (`a:12:xyz`) breaks the scan → no line (the grep-content case). A 20-digit number → `parse::<usize>()` Err → `let Ok(n) = … else break` (no panic, no line). |
| `caret_for_line_col` clamps | SAFE | col past EOL → `.min(line_len)` → line end; line past EOF → `line_text` empties → char_col 0 + `line_start` clamps the row → last-line start; no col → 0; line 0 → `saturating_sub` → row 0 (no underflow). |
| `LinkTarget::File` consumers | SAFE | grep + `cargo check --all-targets` confirm exactly 2 (`path_link`, `open_link_target`) + the 2 updated test helpers; a missed one fails to compile. |
| grep vs compiler | SAFE | grep `path:line:` (match content a separate whitespace token) → line only; compiler `path:line:col` → both. The pre-existing `contains('/')` over-linkify (R2) is unchanged, not made worse. |

MSI note for Validate: `parse_line_col` needs a `parse_line_col("a:")` case (empty-digits tail → the `is_empty()` branch); `caret_for_line_col` needs non-degenerate values (row≥1, a clamping col vs a non-clamping col) to kill the `saturating_sub`/`min`/`+` mutants.

## Phase 4 — Validate
- **Tests added:** links.rs `parse_line_col_captures_line_and_col` (T1 — 2-group/1-group/none/non-digit-tail/empty-`a:`), `scan_links_file_line_col` (T2 — rustc `-->` with col, parenthesized `(path:line)`, bare path; the FULL ref is the clickable range) + a `file_at` helper + F10 updated (the `:10` is now captured + the whole ref clickable); code_view.rs `caret_for_line_col_places_and_clamps` (T3 — non-degenerate row≥1, col-past-EOL, line-past-EOF, no-col, line-0).
- **`cargo nextest run -p marley` (link + code_view tests): PASS**; the #196 regressions stay green (only F10 changed — the intended #212 behavior).
- **Two gate reds fixed at source (§0):** (a) coverage — line 267 (the parse-`else break`) was uncovered AND (b) `is_empty() || !all_digit` had an EQUIVALENT `||`→`&&` mutant (the `is_empty()` check is redundant with the `parse::<usize>()` that follows — empty/non-digit both fail parse). Simplified the guard to `!all_digit`: the empty case now flows through the parse-break (covering 267) and the `||` mutant is gone. Both a coverage AND a mutation win from one simplification.
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15 incl. coverage 100% + MSI 100% on `parse_line_col` + `caret_for_line_col`.
- **Live-drive: documented env-block.** This is a click-handler + editor-open path, but no Marley window is running and synthetic input (to click a rendered output ref) has been env-blocked all session (every driven capture #204/#205/#284/#287). Carried by the mechanism: the parser + caret math are MSI-100 unit-proven, and `open_file_at` reuses the already-proven `open_file_in_viewer` + `active_caret_mut` (the M15 editor's own caret path). Re-verify with a driven `grep -n`/`cargo` click→open-at-line when the machine is unlocked (30s, no ticket).
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Added (#212); docs/marley_architecture/app_shell.md clickable-links section extended with the M18 #212 fusion foundation (`LinkTarget::File{line,col}`, `parse_line_col`, the full-ref range, `caret_for_line_col`, `open_file_at`).
- **Knowledge (forge):** failure `BF-fusion-click-range-covers-only-primary-subtoken-001` (`0cc1f04a`, the F1 click-range finding); prevention rule `PR-claude-compound-token-affordance-covers-whole-token-001` (medium — a compound-token affordance's hit-region must cover the whole token). AAR `e9aadbf3` submitted: completed, effectiveness 5. Validate lesson (not re-recorded — a known class): `is_empty() || !all_digit` was an equivalent `||`→`&&` mutant because the following `parse::<usize>()` already rejects empty/non-digit → simplify redundant guards (fixes MSI AND coverage at once).
- **Ticket** TICKET-212 → closed/ (status closed) + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** the fusion FOUNDATION shipped — a `file:line:col` ref opens the editor at that line/col; four downstream M18 tickets reuse it. Parser + caret math cov/MSI 100, GATE GREEN [diff]. LOCAL commit only.
