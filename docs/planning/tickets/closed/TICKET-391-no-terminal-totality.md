# TICKET-391 — No-terminal totality: decouple workspace() from ≥1-terminal (enabler)

- **Forge:** #391 `41161e92-e28d-46b9-a938-750e2d83e1bc` (sprint #38 `fc972431`)
- **Type:** chore (enabler / risk-carrier)
- **Milestone:** M27
- **Status:** closed
- **Pipeline:** docs/planning/pipeline/queued/391-no-terminal-totality.spec.md

## Summary
Behavior-neutral audit making every app-layer site total over a no-terminal (and zero-tab) project —
the ~90-site `workspace()`-assumes-a-terminal reality (#202) that makes `LastTerminal` "a
requirement". Guards stay on; the suite proves byte-identical behavior; the audit ledger is a
deliverable. #392 (the actual guard dissolve) hard-depends on this.

## Headline acceptance
Full suite green with zero behavior change; every audited site handles a terminal-less project by
type (no panic paths); a zero-`T=` project line parses; the ledger enumerates every touched site.
