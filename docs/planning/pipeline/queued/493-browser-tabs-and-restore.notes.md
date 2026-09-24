# B1b: Browser tabs, restore on relaunch, and the select picker — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-493-browser-tabs-and-restore.md
- **Pipeline spec:** 493-browser-tabs-and-restore.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the rest of B1.
- **Classification:** feature; `marley_browser` (sessions per page, the select's DOM work) and
  `marley_workbench` (the tabs, the serializable item and its table, the picker). No Zed path
  expected: `register_serializable_item` and the `db` crate are public.
- **Recall (§18.3):**
  - The #403 decision (docs/marley_architecture/embedded-browser-model.md Q4): never put a URL
    in a layout codec; a side table keeps it. The item's own table is that side table.
  - AD-claude-registry-lifecycle-fork-pinned-vs-dropped-001: the old Browser tab dropped its
    resident on last close; here the page closes with its tab (D2).
  - The Explore report: `SerializableItem` needs its own `db` domain, and `Onboarding` is the
    lightest real implementation.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
