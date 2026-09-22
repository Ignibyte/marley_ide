# Zed agent threads in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-439-rail-zed-threads.md
- **Pipeline spec:** 439-rail-zed-threads.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad chose "Terminals and Zed threads" for the rail rows (2026-09-22), and the
  original complaint was "I cant figure out even how to start a new agent".
- **Classification / tier:** feature, medium; `marley_workbench` only, no new Zed touchpoint.
- **Recall (§18.3):** `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`
  (the Agent Panel arm goes into the one selector); `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`
  (key thread rows by thread id); `PR-claude-async-answer-carries-question-identity-every-hop-001`
  (a confirmation row must name the thread it belongs to).
- **Discovery:** the sidebar sweep of 2026-09-22 (plan D3, D8): thread sources, the public
  AgentPanel API, the suppression check in `conversation_view.rs:2863-2915`.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
