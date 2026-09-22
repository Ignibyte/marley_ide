# TICKET-274 — Incremental + off-thread tree-sitter re-parse (B3.2)

- **Forge:** #274 `522d4028-081c-495f-b2a9-08a4aa639d59` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/274-incremental-treesitter.spec.md` (b5a9b214-e574-43f3-bbf3-aa290b98814d)

## Summary
The measured 25.9ms whole-file parse per keystroke at 8k lines (the
#268 critic's numbers) blows the frame. `marley_syntax` gains a
`HighlightSession` (previous Tree + previous text snapshot — the
snapshot makes every InputEdit point conversion pure and testable);
single-delta steps re-parse only the damage, multi-delta gaps fall
back to a full parse; both run on a worker thread behind a
generation-dropped channel the pump drains. Small files keep the
zero-latency sync path. Relative perf pin (incremental < full/3, same
build). Query windowing, other grammars, multi-file sessions: out.

## Acceptance
See the spec's EARS table (REQ-001..005): point/edit math, the
incremental≡full equivalence corpus, the ratio pin, generation/nonce
drop, and the threshold behavior with existing flows green.
