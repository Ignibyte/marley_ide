# TICKET-339 — Regex find/replace: the `.*` mode on ⌘F + capture-group replace

- **Forge ticket:** #339 1e6c0a52-f41b-46d5-90d7-1f98e036bd36 (feature, M22)
- **Owner:** claude (this session)
- **AAR:** 34cfa1c3-97cb-48a8-a60b-1e999f67b518
- **Pipeline doc:** ../../pipeline/completed/339-find-regex.spec.md
- **Source ticket:** M22 "The editing bar" batch (#336–340 + the 11 that follow) — the FOURTH of the batch
  ([m22-editing-bar.md](../../design-notes/m22-editing-bar.md) · [roadmap.md](../../../marley_architecture/roadmap.md))
- **Status:** closed

## Summary
A `.*` mode on the #272 find bar: the query becomes a regex, and Replace gains capture groups
(`$1`/`${name}`/`$$`). An invalid pattern is a VALUE (an inline bar error), never a panic.

**Phase 1 found more drift here than in any other ticket this batch.** The spec's central decision
(D-SAME-SHAPE — "the plumbing doesn't know the mode exists") is **false as stated, and the spec refutes
itself**: the READ path (bands, n-of-m, ⌘D, select-current) is genuinely mode-blind — verified, nothing
recomputes a match end from the query's length — but the WRITE path cannot be, because
`find_all_regex -> Vec<(CharOffset, CharOffset)>` **throws away exactly the captures capture-replace
requires**. The shape carries only by discarding the thing the feature needs.

It also names two controls that **do not exist**: there is no case chip (`fold: true` is hardcoded) and no F3
(find-next is Enter/⇧Enter). So two REQs are "build it", not "map it".

## Acceptance
Toggling the `.*` chip makes the query a regex whose matches feed the existing bands/counter unchanged;
Replace expands `$1`/`${name}`/`$$` per match, back-to-front, in one undo unit; an invalid pattern shows an
inline error and disables Replace without panicking; and with the chip OFF the literal path is byte-identical
to today. Full EARS REQ-001..009 in the pipeline spec — **REQ-002/003/007 are AMENDED at Phase 1** (see the
ledger).
