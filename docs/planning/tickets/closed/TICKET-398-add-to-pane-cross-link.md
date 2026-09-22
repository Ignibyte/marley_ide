# TICKET-398 — Add-anything-to-a-pane + the ContentId cross-link: one instance, many views goes live (the #388 slice-5)

- **Forge:** #398 `7aa54235-760c-4191-a5e6-12a5b65dfa7e` (sprint M28 "The Registry Payoff" `6558258f-a0d2-45fb-b601-cd5329a1cd77`)
- **Type:** feature
- **Milestone:** M28
- **Status:** closed (2026-08-05 — shipped; pipeline archived at docs/planning/pipeline/completed/398-add-to-pane-cross-link.{spec,notes}.md)
- **Depends:** #396 (terminals onto the registry, slice-2) — **HARD**; #397 (editors onto the registry, slice-3) — **HARD**. The ticket's own gate line: "needs #396 + slice-3 — terminals AND editors registered." Pre-migration there is nothing to `acquire_view`; /work must not promote #398 while either is unshipped.
- **React-first:** **APPLICABLE — Zone A** (rail + context menus + palette). Build the add-to-pane gesture, the Pane-row labels, and the home-section cross-listing in marley-web FIRST (`artifacts/marley-ide/src/components/LeftRail.tsx`, `components/ContextMenu.tsx`, `components/views/SplitTerminalView.tsx`, the `PaneItem { type, name }` stand-in), confirm the flow at localhost:5173, then port 1:1 via the `marley-web/docs/MARLEY-PARITY.md` port map (§ Shared vocabulary keeps `PaneItem` ContentId-aligned for exactly this slice).
- **Pipeline:** queued — `docs/planning/pipeline/queued/398-add-to-pane-cross-link.spec.md` (Phase-1 drafted; promotes to active at Phase 2)

## Summary
The **first user-visible payoff** of the #388 registry spine (#394 lifecycle → #396 terminals → #397 editors — until now every entry sits at view_count 1; #396's own line: "every view is 1-view until add-to-pane / slice-5"). Two halves:

1. **The add gesture** — a verb/command set that drops an **OPEN** `ContentId` into a split cell: a palette flow + context-menu rows, folded into the ONE #166/#175/#393 menu machinery (no parallel popover, no drag — gpui 0.2.2's `on_drag`/`on_drop` is the noted-not-adopted v2). Any section's content becomes a Pane view of it: a terminal into a split, an editor into a split — **one instance, many views, live for the first time** (shared session/scrollback for terminals; one Buffer, two carets for editors). Every add `acquire_view`s; every cell close routes through the #396 `release_view` contract; the last-view drop returns ownership to the caller (terminals keep the ~600ms off-thread reap).
2. **The cross-link half deferred from #390** — Panes cell rows carry a ContentId-derived label (what the cell actually shows, not the positional "pane n"), and the Editor/Terminal home sections cross-list content that also lives in a Pane (two navigator entries, one instance — chad's model A: "the original stays open"). Presentation-only, derived fresh each render like the #390 Panes section itself.

## Out (later train slices)
Drag-and-drop add (v2); nameable arrangements + `[[panes]]` persistence (slice-6 #399); cockpit/browser kinds onto the registry (slice-8 #400); global cross-workspace Panes (gated slice-7); restore-time de-dup of same-key cells (sharing is session-local this slice — the persistence codec stays byte-identical, ids never serialize).

## Headline acceptance
An open terminal and an open editor can each be added into a split cell with both views presenting the one instance (typing/edits visible in both; view_count 2, driven-captured); Panes cell rows read content labels; the home sections cross-list paned content; closing one view releases exactly one refcount and the instance survives to the last close (then the existing off-thread reap — unit-proven over every close path, incl. closing the home tab of paned content); the dynamic palette rows REBUILD, never append (the #204/R1 id-reuse constraint); full EARS in the pipeline spec; the React↔Marley parity pair captured at Validate.
