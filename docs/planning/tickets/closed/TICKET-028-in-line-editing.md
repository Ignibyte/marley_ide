# TICKET-028 — input: in-line editing (cursor movement + edit within the line)

- **Forge ticket:** #28 `1a5d1fa1-01e1-417c-b216-7d787df93adf` (feature, M1.D — The Daily Driver, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `7e6ede9d-b180-4655-9e1f-241ff73808b4`
- **Pipeline doc:** ../../pipeline/active/in-line-editing.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
The prompt is append + backspace only — the caret can't move mid-line. Extend `Key` + `apply_key`
with ←/→/word/home/end/delete-forward, WIRING the already-tested `marley_editor::movement` fns
(not reimplementing), and fix the render so the caret paints at its position (a pure
`split_at_caret` + `before ▏ after`). Insert/backspace already work at an interior caret — pinned
by a test.

## Acceptance
The new `apply_key` arms + `split_at_caret` at cov 100/MSI 100 (each motion → the right
`movement` result + `Edited`; DeleteForward interior + end guard; interior insert/backspace;
multibyte split at both ends); FULL gate GREEN [--diff]. The enabler for #29 history + #33 raw
mode. Full EARS in the pipeline spec.
