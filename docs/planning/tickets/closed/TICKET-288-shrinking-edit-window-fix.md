# TICKET-288 — Fix the #285 shrinking-edit windowing regression (highlight vanishes on backspace-at-comment-end)

- **Forge ticket:** #288 `ce37cfba-3d91-45da-8514-d6f3dd91a4f0` (bug, M17)
- **Owner:** autonomous /goal run (sprint #30)
- **AAR:** `ec411627-3f7c-4507-bfe4-e28543c03f74`
- **Pipeline doc:** ../../pipeline/active/288-shrinking-edit-window-fix.spec.md
- **Source ticket:** #285 (the windowing that introduced the regression) — found by the #285 inspect differential-fuzz critic
- **Status:** closed

## Summary
CRITICAL correctness regression in #285. A pure-deletion tail-trim of a comment
(`"// abcdef"` → `"// abc"` — routine backspacing) makes `changed_ranges` return
EMPTY (only the node's EXTENT shrank; the surviving bytes are textually
unchanged), so `damage_window` degenerates to the empty range `[p, p)`;
`spans_in_window(set_byte_range([p,p)))` returns nothing (tree-sitter's intersect
needs a node STRICTLY straddling an empty range, and the shrunk comment ends
EXACTLY at `p`); the grow-loop can't widen (no fresh spans); `splice_spans`
correctly drops the stale cached comment — and **nothing replaces it, so the
highlight vanishes.** Also reproduces for a token whose surviving prefix
re-tokenizes shorter (`"42"` → `"4"`). Root flaw: #285 assumed `changed_ranges ∪
edit-span ⊇ every stale span`, FALSE for SHRINKING edits.

Fix: a pure `cover_edited_cached(window, cached_old, start, old_end, new_end)` that
widens the window to the surviving NEW-coord footprint of every cached span
TOUCHING the replaced region `[start, old_end)` — the shrink candidates
`changed_ranges` never reports. `splice_spans` and `window_extent` are unchanged.
Plus a bounded IN-TEST differential fuzzer (deterministic, random edit chains
asserting `inc == fresh`) — the durable fix for the corpus gap that let #285 ship.

## Acceptance
The pure-deletion comment/string tail-trim and the re-tokenize repros highlight
byte-identically to a fresh full highlight; a bounded differential fuzzer over
random edit chains finds zero `inc != fresh` mismatches; the #274 corpus + perf
pin stay green; `cover_edited_cached` at cov/MSI 100. Full EARS in the spec.
