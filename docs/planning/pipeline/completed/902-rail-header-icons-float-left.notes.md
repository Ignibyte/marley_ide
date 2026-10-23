# The rail header's icons all float to the left — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-902-rail-header-icons-float-left.md
- **Pipeline spec:** 902-rail-header-icons-float-left.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23 (the ticket's source).
- **Classification / tier:** bug, XS. Marley crate only.
- **Recall (§18.3):** #873 turned the view names into icons; the header's `flex_1` spacer sat
  between the switch and the other buttons. The brain had nothing.

## Phase 2 — Code
- `render_header`: the `flex_1` spacer moved after Shrink, its right-click menu kept.
- Gate: `just gate-diff` 17 passed, 0 failed.

## Phase 3 — Test
- `902-01-night`, `902-02-day` (REQ-001): Folder, Thread, Terminal, Files, Add Project and Shrink
  together from the left, the cat mark after the empty space.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (Threads in the rail).
- Knowledge: nothing new.
- Brain: decided on consultation 21f4f0521b1c41f2877d34c92071e6a8.
