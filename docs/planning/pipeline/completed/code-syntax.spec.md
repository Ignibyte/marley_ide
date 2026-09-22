---
pipeline_id: 53e51355-5001-4cb7-89ae-a8a3912430c4
ticket: forge#99 (33efa431-5ed9-470c-86dc-480c85ab034d) · local docs/planning/tickets/open/TICKET-099-code-syntax.md
aar_id: 488f8f67-809d-4166-aecb-ce0266820dd3
status: Phase 5 — Complete PASS
title: syntax highlighting (clean-room, by language)
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_syntax.rs (NEW PURE: Language, language_of, TokenKind, Span, highlight_line)
  - crates/marley_app/src/lib.rs (mod code_syntax)
  - crates/marley_app/src/app.rs (SHIM: the viewer maps Span.kind → theme colors)
---

## Title
A lightweight, clean-room per-language lexer so the #97 viewer colors keywords, strings, comments, and
numbers — for Rust / TOML / JSON / Shell (Markdown + Plain pass through in v1). No tree-sitter/syntect.

## Scope
### In
- NEW pure `code_syntax.rs`: `Language`, `language_of(path)`, `TokenKind`, `Span`, `highlight_line`.
- SHIM: the viewer render maps each `Span.kind` → a theme color.

### Out
- Multi-line constructs (block comments/strings spanning lines — the viewer is line-by-line; approximated).
- Rich Markdown highlighting (v1 = Plain). More languages. Semantic highlighting.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a hand-rolled single-line lexer per `LangSpec { keywords, line_comment, quotes }`; clean-room (§20).
- D2 — precedence per position: line-comment → string → number(at a word boundary) → keyword-word → plain.
- D3 — adjacent Plain runs coalesce into one `Span`; Markdown/Plain → a single Plain span.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `language_of(path)` runs, it shall map the extension to its Language (unknown → Plain). | unit |
| REQ-002 | WHEN `highlight_line(line, lang)` runs, it shall classify comments/strings/numbers/keywords, else Plain, coalescing plain runs. | unit |
| REQ-003 | WHEN a string is unterminated / the line is empty / non-ASCII, `highlight_line` shall not panic. | unit |
| REQ-004 (visual) | WHEN a code file is open, its tokens shall be colored. | self-test (engine; synthetic open env-blocked) |
| REQ-005 | gate GREEN, cov/MSI 100 on code_syntax.rs; the shim masked. | gate |

## Phase Plan
- **P2** — code_syntax.rs API + the lexer algorithm; the viewer color mapping; test plan.
- **P3** — implement (code_syntax.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: highlight_line MSI (each classifier + coalesce + per-lang specs + Plain); language_of;
  no panic on odd input.
- **P4** — language_of + highlight_line tests (cov/MSI 100) + gate GREEN (colored render engine-verified).
- **P5** — docs, AAR, archive, close #99.
