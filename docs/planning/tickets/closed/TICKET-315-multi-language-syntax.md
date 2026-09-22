# TICKET-315 — Multi-language syntax highlighting (the language axis + a 10-slot taxonomy)

- **Forge ticket:** #315 2936c168-b807-48de-8a57-d811d8288e9d (feature, M20)
- **Owner:** session 99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c
- **AAR:** c0e7cea0-9541-46b1-94bb-18fa850d7bd3
- **Pipeline doc:** ../../pipeline/active/315-multi-language-syntax.spec.md
- **Source ticket:** M20 language-intelligence batch (the goal /work 300,302,303,304,305,314,315,316,317,259)
- **Status:** closed

## Summary
`marley_syntax` highlights RUST ONLY today; a Python/TypeScript/JSON/TOML/Bash file renders plain. This threads a
`Lang` axis through the crate (per-language grammar + adopted upstream highlight query, `HighlightSession::new(lang)`,
`kind_of_capture(lang, name)` with a per-grammar pin test), grows the token taxonomy from 5 slots to 10
(`{Keyword, Function, Type, Str, Number, Comment, Attribute, Punctuation, Property, Plain}` — the set #316 themes;
Rust gains richer color as a NAMED side effect), adds 6 grammar crates (python/js/ts/json/toml/bash against core
0.26.11), makes the app's one syntax-worker session lang-aware (rebuilt on a language switch), and keeps every
no-grammar language (Markdown, .txt) on the shipped hand-lexer / plain floor by construction. Markdown grammar
deferred (its block+inline injection model fights the per-line span pipeline).

## Acceptance
Python/TS/JS/JSON/TOML/Bash fixtures highlight with the right taxonomy kinds; each grammar's CLOSED capture set is
pinned by a truth-table test that fails loudly on a grammar bump; Rust output is identical except the named
enrichments; a no-grammar language renders byte-identical to today (the floor); the worker serves non-Rust trees
(async >1000-line) + rebuilds across a language switch; cargo-deny passes with every new MIT grammar crate. Full
EARS criteria (REQ-001..010) in the pipeline spec.
