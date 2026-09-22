---
pipeline_id: f021ef66-910c-4540-8fda-578cccca43f9
ticket: forge#268 (68014b92-f5b9-4cf4-9267-18980b286418) · local docs/planning/tickets/open/TICKET-268-b3-treesitter.md
aar_id: 9b7dc2f7-06f2-4fa9-b42d-dbff893bd770
status: Phase 5 — Complete PASS
title: B3 tree-sitter — semantic highlight for the editor (Rust first)
type: feature
milestone: M16
references:
  - docs/zed_architecture/subsystems/04-language-syntax-treesitter.md
  - docs/zed_architecture/crates/language.md
---

## Title
Give the EDITOR tab real semantic highlighting: a new `marley_syntax`
crate wraps **tree-sitter** (MIT, 0.26) + **tree-sitter-rust** (MIT,
0.24) — one whole-document parse produces per-line, ascending-disjoint
`(byte-range, TokenKind)` spans via the grammar's own `highlights.scm`
captures, mapped onto Marley's existing 5-kind token palette by a pure,
tested capture→kind table. The editor render swaps its Rust syntax
SOURCE from the hand lexer to the parse (memoized per buffer version);
`styled_slices`/`with_highlights` (#266) consume it unchanged. The hand
lexer remains for non-Rust languages and the #246 read-only pane.

## Scope
### In (slice 1 of B3 — the ticket's "start with Rust")
- NEW crate `crates/syntax` (`marley_syntax`): safe wrapper — no own
  `unsafe` (miri gate skips it by rule), deps `tree-sitter` +
  `tree-sitter-rust` only. API: `highlight_lines(text: &str) ->
  Vec<Vec<(Range<usize>, TokenKind)>>` (per-line byte spans over the
  TAB-UNEXPANDED source line) + the pure `kind_of_capture(name) ->
  TokenKind` table + a line-splitting/span-clipping layer (multi-line
  tokens — strings/comments — clip per line). Its own `TokenKind` mirror
  re-exported or mapped at the seam (decide at design).
- app.rs editor branch: a `SyntaxCache { path, version, lines }` memo on
  RootView, refreshed in the render path when `(path, buffer.version())`
  changes (SYNCHRONOUS whole-file parse — single-digit ms at our file
  sizes); rows read the cached per-line spans; the display-column
  conversion feeds the existing `cols_to_bytes`-adjacent path (spans are
  over the RAW line; the render works on the tab-EXPANDED display —
  a pure raw→display byte-range remap goes with the cache or in
  code_view; decide at design).
- Rust only through tree-sitter; `Language::{Toml, Json, Shell,
  Markdown}` + the #246 read-only pane keep `highlight_line` (the hand
  lexer's Rust arm stays compiled for the pane — divergence documented).
- deny/machete/rustdoc gates green with the two new MIT deps.

### Out (recorded follow-ups — the rest of B3)
- OFF-THREAD + INCREMENTAL re-parse (tree-sitter `InputEdit` fed by the
  #269 `edits_since` deltas; a background executor against a buffer
  snapshot) — the memo keyed by version is the v1 correctness anchor.
- The other grammars (toml/json/bash/md) + full hand-lexer retirement.
- New token kinds / theme colors (function/type/property render Plain
  in v1 — palette parity, no theme growth).
- Indent / brackets / outline / runnables (later B3+ slices; runnables
  are Phase C fusion).
- The marked-text underline channel (deferred from #267; belongs to the
  highlight generalization once kinds grow).

## Reference (§20)
**Zed (the editor reference)** — behavior matched: grammar-driven
semantic highlighting (a parse tree + the grammar's `highlights.scm`
captures rendered as ranged styles) as described behaviorally in
`docs/zed_architecture/subsystems/04-language-syntax-treesitter.md`.
Clean-room: tree-sitter, the Rust grammar, and its query file are
PERMISSIVE/public artifacts consumed as shipped dependencies; Marley's
wrapper, capture→kind mapping, line clipping, and cache orchestration
are original. No GPL editor source is read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Slice 1 is Rust-only + synchronous memoized whole-file parse;
  incremental/off-thread is explicitly the next slice (the ticket's own
  sequencing; correctness first, latency after).
- D2 — The capture→kind map targets the EXISTING 5-kind palette; unknown
  captures → Plain. No theme changes.
- D3 — `marley_syntax` contains no `unsafe` of its own; the FFI stays
  inside the vendored tree-sitter crates (gate:6 rule honored, not
  dodged — the wrapper's logic is fully testable without miri).
- D4 — Spans are computed over RAW line bytes; the raw→display remap for
  tab-expanded rendering is a pure seam beside `cols_to_bytes`.
- D5 — The #246 pane and non-Rust langs keep the hand lexer this slice;
  `highlight_ranges` (the #266 fold) stays as the fallback feed.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `kind_of_capture` shall map keyword/string/comment/number-class captures to the matching `TokenKind` and every other capture to Plain. | pure unit tests, mutants killed |
| REQ-002 | `highlight_lines` shall produce per-line ascending-disjoint spans within line bounds for representative Rust (keywords, strings incl. multi-line, comments incl. `//!`/`/* */` spanning lines, numbers, multibyte idents) — matching tree-sitter's parse, clipped per line. | unit tests on fixture source |
| REQ-003 | The editor tab shall render Rust through the parse: a construct the hand lexer got WRONG and tree-sitter gets right (e.g. `"a // not a comment"` inside a string; a keyword-lookalike ident like `formatter`) shall paint correctly. | headless open + driven capture |
| REQ-004 | The syntax memo shall refresh exactly when `(path, version)` changes — typing updates highlighting on the next frame; switching files re-parses; an unchanged frame re-uses the cache. | unit (cache key fn) + headless typing assert |
| REQ-005 | Non-Rust files and the #246 read-only pane shall render byte-identical to pre-#268 (the hand-lexer path untouched). | existing tests + driven capture of the #246 pane |
| REQ-006 | The new deps shall pass deny (MIT), machete (used), and the docs gate. | gates.sh --diff |

## Phase Plan
- **P2 Design** — the exact marley_syntax API + TokenKind seam, the
  capture set of tree-sitter-rust's highlights.scm, line clipping
  arithmetic, the cache placement + refresh point, the raw→display
  remap, test plan.
- **P3 Implement** — crate + wrapper + map + cache + render swap.
- **P3.5 Inspect** — critics (span clipping/multibyte fenceposts, cache
  staleness, provenance/licensing, render regression).
- **P4 Validate** — units + mutants; headless; driven captures; gate.
- **P5 Complete** — CHANGELOG, editor.md/crate-map, AAR, archive, close.
