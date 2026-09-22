---
pipeline_id: 24e4972c-3f50-4f31-b89d-fd5ecee6f4a4
ticket: forge#228 (2d14452f-4beb-4bab-b37a-36e7ddeb179c) · local docs/planning/tickets/open/TICKET-228-focus-border-over-divider.md
aar_id: 02e8710c-4b71-4e08-a047-e8e76fcde61f
status: Phase 5 — Complete PASS
title: Focus-box border occluded by the drag-divider on the shared pane edge
type: bug
milestone: M13
references: []
---

## Title
The #191 focus-accent border (the blue box around the focused pane) is drawn per-pane BEFORE the #130
drag-dividers, so the later-painted 6px divider occludes the border's bar on the shared edge — the
focus box looks broken/missing on the divider side (chad live feedback #4). Fix: draw the focus border
AFTER the dividers so it is the topmost pane-chrome.

## Scope
### In
- `marley_app/src/app.rs` pane render — hoist the focused pane's content rect and draw its 4 focus-border
  bars AFTER the divider loop (so the border paints over the divider), instead of inside the pane loop.

### Out (explicitly deferred)
- Any change to `focus_border_rects` geometry (it is correct + pure + tested — the bug is paint order).
- Any change to the #130 divider (still `.occlude()`, still owns the drag).
- The M13 workspace-model work (#233+) — unrelated.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — fix by RENDER ORDER, not geometry.** The border becomes the last pane-chrome drawn (after the
  divider loop) so it is on top. NOT insetting the border (that would pull it off the pane edge) and NOT
  changing the divider.
- **D2 — hoist the focused content rect** into a local `Option<Rect>` during the pane loop; draw the border
  once, after the divider loop. Minimal move.
- **D3 — `focus_border_rects` (workspace.rs) is UNCHANGED.** No new pure fn — the fix is paint-order; a
  contrived pure helper would be dead-code / equivalent-mutant bait (§0 spirit).
- **D4 — the border divs stay non-`.occlude()`** so the mouse still passes through to the occluding divider
  underneath → the drag interaction is preserved.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pane is focused AND shares an edge with a drag-divider, the system shall render the focus-accent border visible on ALL FOUR edges including the divider-side edge (not occluded). | Driven capture (focus a split pane → 4-edge border visible); ELSE env-blocked → the z-order mechanism (border drawn after the divider → paints on top) + code review |
| REQ-002 | WHILE a divider is present under the focus border, the system shall preserve the divider's drag interaction (the non-`.occlude()` border passes the mouse to the divider). | Code review (border not `.occlude()`, divider is); driven drag if unlocked |
| REQ-003 | WHERE a pane has no adjacent divider, the focus border shall be unchanged — a 2px accent frame on the content area below the title bar. | `focus_border_rects` unit tests stay green (unchanged); driven capture of a lone focused pane |

## Phase Plan
- **P2 Design** — read the exact render block (pane loop + divider loop + `is_focused`); confirm the hoist
  point + that no pure seam is warranted; file manifest (app.rs only); the validate approach (driven-or-mechanism).
- **P3 Implement** — the render-order move in app.rs.
- **P3.5 Inspect** — critic(s): does the move preserve the title-bar layering, the drag, the non-divider edges,
  and the multi-pane/focus detection? clean-room.
- **P4 Validate** — no new unit tests (pure surface unchanged, gate stays green via the app.rs render exclude);
  driven capture of a focused split (or env-blocked → mechanism + code review); gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell.md (the focus-border render-order note); AAR; close #228; archive.
