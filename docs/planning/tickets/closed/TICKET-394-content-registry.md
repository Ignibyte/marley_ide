# TICKET-394 — The ContentId registry: refcounted content, terminals migrate first (M27 spine)

- **Forge:** #394 `adf0b9d6-08e1-43c6-8c7c-26c48670c2a5` (sprint #38 `fc972431`)
- **Type:** feature (structural refactor)
- **Milestone:** M27
- **Status:** closed (done 2026-07-22)

> **D5 SPLIT (design, 2026-07-22):** shipped as slice-1 = the pure generic `ContentRegistry<C>` only.
> The L-sized terminal-**ownership** migration (Content enum + ~46 accessor sites + 7 reaper-preserving
> close paths + persistence rebuild) moved to **#396** (`cd706b38`), its own reviewed ticket.
- **Pipeline:** docs/planning/pipeline/queued/394-content-registry.spec.md

## Summary
Train slice 1 (+ first resident) of the #388 pane-composition model: a Marley-owned ContentId
registry (NOT gpui Entity) with an explicit view refcount + drop-on-last-close; the close→reaper
contract moves into it; terminals migrate first (the Content enum declares all six kinds, only
Terminal wired). Codec untouched (ids never persist); zero visible change. Unblocks add-to-pane /
one-instance-many-views / nameable arrangements.

## Headline acceptance
Registry insert/view/release with exact drop-on-last-release (pure matrix, cov/MSI 100); terminal
close reaps byte-identically via the registry; all persistence round-trips and the full suite pass
unchanged.
