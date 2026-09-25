# B3b: Open a picked element's listener source in the editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-497-pick-source.md
- **Pipeline spec:** 497-pick-source.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar A's "signature move" (`browser-handoff.md`: "picked
  element → listener source → source map → open the file in the editor at that line").
- **Order:** after #496, whose bundle carries the listeners' script locations.
- **Classification:** feature; `marley_browser` (the map fetch and decode), `marley_workbench`
  (the lookup in the worktrees, the tray's link, opening the editor). No Zed path expected.
- **Recall (§18.3):** no source-map code in the tree or the lockfile; Zed's debugger opens a
  stack frame's file at a line, the nearest model; the probe's `sourceMapURL` is relative.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
