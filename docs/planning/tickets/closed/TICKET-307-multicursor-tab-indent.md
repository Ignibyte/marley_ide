# TICKET-307 — Multi-cursor Tab / ⇧Tab: both branches of the indent arm learn about the whole cursor set

- **Forge ticket:** #307 `c8a3a909-6ad3-4360-bfb9-ee225ccf9a2a` (bug, M19)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `7cb51412-b1db-4a6a-b593-a6cfddd3588c`
- **Pipeline doc:** ../../pipeline/active/307-multicursor-tab-indent.spec.md
- **Source ticket:** the M22 integrity-five shelf — ../../design-notes/integrity-five-shelf.md
- **Status:** closed

## Summary

With three cursors on three blocks, Tab indents only the block under the PRIMARY cursor. #297's
inspect surfaced the gap, #299's design scoped it out while building the exact seam it needs
(`indent::touched_rows` — the deduped ascending union of every member's `line_span`), and the Tab
arm's own comment names this ticket as the owner. The fix widens `indent_edits`/`dedent_edits` from
a contiguous `(first, last)` range to a discontiguous `rows: &[usize]` list (matching their shipped
sibling `comment_edits`, which already has that signature and already anticipates this caller), and
points the arm's block branch at `touched_rows`.

Phase-1 recon added what the ticket missed: **its own verify step exercises the branch its work
section never touches.** ⌘⌥↓ ×2 produces bare *carets*, which take the pad branch — so a rows-only
fix would fail the ticket's own drive. Both branches go whole-set: ranged/⇧Tab → the touched-rows
union; all-bare-carets Tab → a pad at every caret, each sized from its own display column. A second
latent bug falls out on the way: the branch predicate `has_selection` reads only the PRIMARY's
range, so a mixed set (caret primary + ranged secondary) misroutes today.

## Acceptance

Tab/⇧Tab act on every cursor — N cursors on N blocks indent all N, two cursors sharing a row indent
it once, each bare caret pads to its own next tab stop — and one ⌘Z reverts the lot with every
cursor carried. Single-caret and single-range Tab stay byte-identical (including ⌘Z granularity: a
lone pad stays ungrouped so it still coalesces with following typing). Full EARS criteria (REQ-001
… REQ-009) live in the pipeline spec.
