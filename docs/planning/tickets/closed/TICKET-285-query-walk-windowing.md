# TICKET-285 — Window the highlight query walk to the damage (the measured O(file) incremental floor)

- **Forge ticket:** #285 `e6210a61-79e6-4e65-90fa-2748332fa89e` (feature, M17)
- **Owner:** autonomous /goal run (sprint #30)
- **AAR:** `435358c3-3dd6-4467-b0b1-16739213870b`
- **Pipeline doc:** ../../pipeline/active/285-query-walk-windowing.spec.md
- **Source ticket:** #274 inspect TS-F2 (the M17 follow-up shelf)
- **Status:** closed

## Summary
Follow-up from #274. The incremental highlight path re-parses the tree in ~1.0ms
(9× faster than a 9.2ms fresh parse) but then runs the SAME full-tree query walk
(`spans_from_tree` — a `QueryCursor` over the whole root) at ~5.7ms on every
keystroke, flooring end-to-end incremental at 0.46 of a full highlight. This
ticket windows that walk: after the incremental re-parse, compute the damage =
`old_tree.changed_ranges(&new_tree)` ∪ the edit span, run the query over only the
merged damage window (`QueryCursor::set_byte_range`), and splice the fresh
in-window spans into the CACHED out-of-window spans (rebased by the edit delta).
The result must stay byte-identical to a fresh full highlight on the #274
equivalence corpus (15 acid cases incl. block-comment cascades, where
`changed_ranges` spans most of the file and the window degrades gracefully to
~full). Pure cov/MSI 100 on the window-merge + splice arithmetic.

## Acceptance
Incremental highlight ≡ full highlight (per-line, byte-identical) on the #274
corpus after windowing; end-to-end incremental < 1/3 of a full highlight on the
~8k-line fixture (the originally-intended #274 number, now achievable). Full EARS
criteria in the pipeline spec.
