# B1c: Browser tabs return after a relaunch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-494-browser-tabs-restored.md
- **Pipeline spec:** 494-browser-tabs-restored.spec.md

## Phase 1 — Plan (drafted 2026-09-25, split from #493)
- **Request:** the restore half of the queued #493, split so each ticket is one slice.
- **Recall (§18.3):** #403's rule, no URL in a layout codec; #493's pages keyed by target id;
  the harness has no relaunch step yet, and #491's scenario quits Marley through the palette
  (`zed: quit`: Ctrl+Q in a terminal goes to the shell).
- **Checklist (no TaskCreate in this harness):** mint ✓, prior art ✓, spec ✓.
