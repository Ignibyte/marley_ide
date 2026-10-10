# The Threads page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-737-the-threads-page.md
- **Pipeline spec:** 737-the-threads-page.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - Zed's thread history is `ThreadsArchiveView` in its sidebar (`agents_sidebar::ToggleThreadHistory`); the Marley layout's rail replaced the sidebar, so the history went with it.
  - PR-claude-701: a page's handlers that update the workspace are plain closures over a weak handle.
  - The rail's archived threads (#616) match a project by `main_worktree_paths`; the page lists every record regardless.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
