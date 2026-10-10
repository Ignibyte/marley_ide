# An agent thread starts in a tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-734-an-agent-thread-starts-in-a-tab.md
- **Pipeline spec:** 734-an-agent-thread-starts-in-a-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - PR-claude-697-a-hosted-view-focuses-where-its-own-host-does-001: the tab's focus is the thread's `message_editor`, never the view's root handle.
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001 and PR-claude-702-making-a-workspace-in-the-background-keeps-the-focus-001.
  - #697 and #702 (completed): `ThreadTab`, `activate_for`, one rail row per thread; #697's notes record the risk of an idle thread evicted from the panel's retained threads.
  - The research of 2026-10-10: `ConversationView::new` is public; the panel's `connection_store()` is shareable; the thread store archives a thread whose project has no visible folder (`thread_metadata_store.rs:1316`); Zed's agent ignores `work_dirs` and resolves only visible worktrees.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
