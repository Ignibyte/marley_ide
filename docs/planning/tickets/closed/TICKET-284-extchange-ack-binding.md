# TICKET-284 — Bind external-change acknowledgments to the observed disk state

- **Forge ticket:** #284 (c439b877-1e0d-4478-b359-f8b11a563a91) (bug, M17)
- **Owner:** autonomous /goal run (session a2a59fa8)
- **AAR:** 18068cb8-5856-43a4-8231-ca2ee680c1f4
- **Pipeline doc:** ../../pipeline/active/284-extchange-ack-binding.spec.md
- **Source ticket:** M17 follow-up shelf (#282–287, sprint #30); from #275 inspect F3/F4a
- **Status:** closed

## Summary
The ⌘S-under-conflict arm and the banner's Keep-mine acknowledge the disk state at press/click time, not
the state the `Changed` warning described — so a newer agent write landing between the warning and the
acknowledgment gets overwritten (armed 2nd ⌘S) or re-snapshotted (Keep-mine) sight-unseen. Bind each
acknowledgment to the observed `(mtime, len)` (a pure `acknowledgment_is_stale` check + a
`conflict_observed` field) and re-warn/re-flag when the disk moved again.

## Acceptance
An arm/Keep-mine made against W1 re-warns (does not overwrite / re-snapshot) when a newer W2 landed; an
unchanged disk proceeds exactly as today. Pure decision at cov/MSI 100 + headless
W1-arm/W2-lands/press-2-re-warns + the Keep-mine equivalent. Full EARS in the pipeline spec.
