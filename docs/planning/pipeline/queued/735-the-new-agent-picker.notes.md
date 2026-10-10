# The New Agent picker — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-735-the-new-agent-picker.md
- **Pipeline spec:** 735-the-new-agent-picker.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #450 made the picker (`agents.rs` `NewAgentPicker`), #532 the permission modes it starts CLIs with, #701 Home's New Agent card.
  - PR-claude-695-a-scenario-chooses-a-menu-entry-from-home-001: menus open with nothing chosen.
  - `Workspace::prompt_for_open_path` is already used by `agent_bar.rs` and `rusty/import.rs`; Zed's picker returns one path, file or folder.
  - The project panel has no hook for another crate's menu entry: the touch is three hunks beside Open in Terminal.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
