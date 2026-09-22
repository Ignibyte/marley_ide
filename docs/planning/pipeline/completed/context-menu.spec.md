---
pipeline_id: 49751003-6858-48c1-ae79-199a96afea24
ticket: forge#166 (d783dfa6-f858-420b-975e-104819cc3a44) · local docs/planning/tickets/open/TICKET-166-context-menu.md
aar_id: a1f30333-325c-43e9-a52d-14b912674ecd
status: Phase 5 — Complete PASS
title: M10 — the split context menu (right-click → Split Right / Split Down / Close Pane)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/context_menu.rs (PURE, NEW: MenuAction, ContextMenuState, menu_origin)
  - crates/marley_app/src/app.rs (SHIM: the overlay render + routing; split_focused_pane(axis))
---

## Title
Right-click a terminal opens a Warp-style menu at the pointer — Split Right, Split Down, Close Pane — instead
of splitting directly. Esc / click-away dismisses; ↑↓ + Enter navigate.

## Scope
### In
- PURE `context_menu.rs` (new): `MenuAction`, `MENU_ITEMS`, `ContextMenuState` (new/move_up/move_down wrap 3/
  selected/action), `menu_origin(x,y,mw,mh,ww,wh)` clamped so the menu never overflows.
- SHIM `app.rs`: `context_menu: Option<ContextMenuState>`; the right-click opens the menu (focus first); a
  click-away backdrop + the 3-row menu render (selected highlighted); Esc/↑↓/Enter routed while open;
  `split_focused_pane(axis)`; ClosePane → `dispatch_action("close-pane")` (#161 semantics).

### Out
- Submenus, per-item icons, a code/cockpit-tab context menu, rename-tab items.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the menu replaces #155's direct split (Split Right = the old behavior, one click deeper — Warp parity).
- D2 — ClosePane reuses the ⌘W dispatch (pane-then-tab + the LastTerminal guard come free).
- D3 — the pointer origin clamps inside the window (menu_origin), never negative.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `ContextMenuState` shall wrap selection in both directions over 3 items and expose the selected `MenuAction`. | unit |
| REQ-002 | `menu_origin` shall return the pointer for an interior click and clamp at the right/bottom edges, never negative. | unit |
| REQ-003 (visual) | WHEN a terminal is right-clicked, the menu shall render at the pointer; clicking Split Down shall produce two STACKED panes (nested rail rows). | driven capture |
| REQ-004 (visual) | WHEN Close Pane is chosen on a 2-pane tab, one pane shall close; Esc shall dismiss the menu without acting. | driven capture |
| REQ-005 | gate GREEN; the pure state/clamp at cov/MSI 100; the overlay masked. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (overlay routing: backdrop vs menu ordering, key swallowing, the
split-axis plumbing). P4 tests + driven + gate. P5 docs/AAR/archive.
