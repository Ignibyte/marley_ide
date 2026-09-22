# TICKET-268 — B3 tree-sitter: semantic highlight (Rust first)

- **Forge ticket:** #268 68014b92-f5b9-4cf4-9267-18980b286418 (feature, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 9b7dc2f7-06f2-4fa9-b42d-dbff893bd770
- **Pipeline doc:** ../../pipeline/completed/268-b3-treesitter.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
New `marley_syntax` crate (safe wrapper, no own unsafe): tree-sitter +
tree-sitter-rust parse the whole document; the grammar's highlights.scm
captures map through a pure capture→kind table onto Marley's existing
5-kind palette; per-line ascending-disjoint byte spans (multi-line
tokens clipped) feed the #266 styled_slices/with_highlights substrate
unchanged. The editor memoizes the parse by (path, buffer version).
Rust only this slice; the hand lexer stays for other langs + the #246
pane. Incremental/off-thread re-parse and the other grammars are the
recorded next slices.

## Acceptance
Capture map + line clipping pure/mutation-clean; representative Rust
(incl. constructs the hand lexer gets wrong) renders correctly; the
memo refreshes exactly on version change; non-Rust + the #246 pane
byte-identical; deny/machete/docs green with the MIT deps. Full EARS
in the spec.
