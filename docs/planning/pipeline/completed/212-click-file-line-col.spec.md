---
pipeline_id: 559e8210-6ab8-447b-b1ab-63fa0ffbe77f
ticket: forge#212 (680aa96d-cb42-4a41-81a7-c5766e2fd73b) · local docs/planning/tickets/open/TICKET-212-click-file-line-col.md
aar_id: e9aadbf3-f9e4-47f4-b0ab-1f4de35cdd4c
status: Phase 5 — Complete PASS
title: Click a file:line:col in terminal output → open the editor at that line
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
Click a `path:line[:col]` reference in terminal block output to open that file in
the editor AT that line, caret placed at the column. The parser scaffolding
already exists (#196 `strip_line_col` strips the suffix to find the path); this
CAPTURES the stripped location and places the caret.

## Scope
### In
- `links.rs` (pure): `strip_line_col(&str) -> &str` → `parse_line_col(&str) -> (&str, Option<usize>, Option<usize>)` returning `(path, line, col)`; `LinkTarget::File` carries `line`/`col`; `path_link` fills them.
- `app.rs` (shim): `open_link_target`'s `File` arm places the caret at the location; a pure `caret_for_line_col(buffer, line, col) -> CharOffset` (1-based line/col → clamped `CharOffset` via `Buffer::line_start` + line length).
- The render click path (app.rs:7848 → `open_link_target`) is UNCHANGED — it already passes the `LinkTarget`.

### Out (explicitly deferred)
- The diagnostics gutter (#289), jump-to-failure (#213), trace frames (#291) — separate M18 tickets that BUILD on this parser.
- New output formats beyond `path:line[:col]` (the multi-frame python/node trace shapes live in #291).
- Windows drive-letter paths (`C:\…` — the `:` collides with the line separator; macOS/Unix-first, noted).

## Reference (§20)
Zed (the editor) / Warp (the terminal). Observed behavior: clicking a
`file:line:col` in compiler/test/grep output jumps the editor to that exact line
and column (every IDE + modern terminal does this). Marley matches by parsing the
ref and placing the caret. Clean-room §20: the parser + caret arithmetic are
Marley's own (extending the #196 heuristic scanner); no Zed/Warp source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the LAST trailing digit-group is the line; a preceding one is the col.**
  `path:line` → line only; `path:line:col` → both (compiler form). A third+ group
  is not stripped (the loop bounds at 2, matching #196). A trailing non-digit group
  (grep `path:line:matchtext`) leaves `matchtext` on the path → `looks_like_path`
  rejects it → so grep `-n` (`path:line:`) already stops at line; a `path:line:col`
  from a compiler carries both. (parse_line_col mirrors #196's existing 2-iter scan.)
- **D2 — `LinkTarget::File` becomes a struct variant `{ path, line, col }`** (was
  `File(PathBuf)`). Internal enum; the two consumers (`path_link`, `open_link_target`)
  update. `line`/`col` are `Option<usize>` (independent — a col without a line can't
  occur from the parser, but the type stays honest).
- **D3 — 1-based → 0-based, clamped.** Compiler line/col are 1-based; `Buffer` rows
  + line chars are 0-based. `row = line - 1` (saturating), `char = col - 1`; the
  offset clamps to the line's end (col past EOL → line end) and to the buffer end
  (line past EOF → last line). No location → caret stays at 0 (top), the #196 behavior.
- **D4 — place the caret only on a NEW open (or a switch), after the file loads.**
  Reuse `open_file_in_viewer`; then set the active `OpenFile.caret` via a new
  `EditorSurface::set_active_caret` (mirrors the existing `active_caret` reader).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a token ends in `:line:col` (both digits), `parse_line_col` shall return the path with `Some(line)` and `Some(col)`. | links.rs unit: `parse_line_col("a/b.rs:12:5") == ("a/b.rs", Some(12), Some(5))`. |
| REQ-002 | WHEN a token ends in `:line` only, `parse_line_col` shall return `Some(line)`, `None` col; a bare path shall return `None`, `None`. | links.rs unit truth-table (+ trailing-punct, parenthesized). |
| REQ-003 | WHEN a file ref carrying a location is clicked, the editor shall open the file with the caret at `caret_for_line_col(buffer, line, col)`. | `caret_for_line_col` unit (1-based→offset, col-past-EOL clamp, line-past-EOF clamp, no-loc→0) + a driven capture. |
| REQ-004 | WHEN a bare file ref (no location) is clicked, the editor shall open at the top (caret 0) exactly as #196. | Unit (`None,None`→0) + the existing #196 file-open tests stay green. |
| REQ-005 | `parse_line_col` + `caret_for_line_col` shall be pure fns at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `parse_line_col` + the `LinkTarget::File` struct-variant + `caret_for_line_col` + `set_active_caret` + the `open_link_target` arm; the regression test plan.
- **P3 Implement** — the pure parser + caret math + the two shim edits.
- **P3.5 Inspect** — the 2-group ordering, the clamps, grep-vs-compiler, the enum-variant consumers.
- **P4 Validate** — parser truth-table + caret clamps + a driven `grep -n`/`cargo` click→open-at-line capture; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md/terminal doc, AAR, close #212, mark the fusion foundation shipped.
