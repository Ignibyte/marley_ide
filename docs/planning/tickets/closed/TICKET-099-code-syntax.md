# TICKET-099 — syntax highlighting (clean-room, by language)

- **Forge ticket:** #99 `33efa431-5ed9-470c-86dc-480c85ab034d` (feature, M4 seq-3; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `488f8f67-809d-4166-aecb-ce0266820dd3`
- **Pipeline doc:** ../../pipeline/active/code-syntax.spec.md
- **Status:** closed

## Summary
A clean-room per-language lexer for the #97 viewer: `language_of(path)` + `highlight_line(line, lang)` →
`Vec<Span>` (Keyword/Str/Comment/Number/Plain) for Rust/TOML/JSON/Shell (Markdown/Plain pass through).
cov/MSI 100; the viewer maps kinds → theme colors. Deps #97 + #35 + #31.

## Acceptance
language_of + highlight_line at cov/MSI 100 (classifiers/coalesce/per-lang/odd-input); a code file colors
(engine/self-test); FULL gate GREEN. Full EARS in the spec.
