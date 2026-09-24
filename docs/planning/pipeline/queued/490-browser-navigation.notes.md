# B1a: The address bar and navigation — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-490-browser-navigation.md
- **Pipeline spec:** 490-browser-navigation.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the chrome a person needs to browse in the tab.
- **Classification:** feature; `marley_workbench` (the toolbar, the dialog card, the key
  context) and `marley_browser` (history, loading and dialog events). No Zed path.
- **Recall (§18.3):**
  - #406's lesson: a page that fails can deliver nothing; the loading state ends on the
    frame's stop event, and a connection that closes shows in the tab as #488 draws it.
  - #481: an `Editor` inside a container that filters keys (the rich input) is the model for
    the address bar inside a tab that forwards keys to the page.
- **Discovery:** the probe's CDP protocol dump for the Page domain.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
