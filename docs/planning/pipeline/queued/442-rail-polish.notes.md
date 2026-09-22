# Rail persistence and polish — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-442-rail-polish.md
- **Pipeline spec:** 442-rail-polish.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** the remainder of the plan's rail (workbench-shell W6).
- **Classification / tier:** feature, medium; `marley_workbench` only.
- **Recall (§18.3):** `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`
  (filtered rows keep their item-id keys); `PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001`
  (the switcher under a hovering pointer); `PR-claude-gpui-keyboard-focus-needs-a-mouse-down-not-a-raise-001`
  (focus in live drives); the gpui-era rename and search tickets (#112, #177) in
  `docs/planning/pipeline/completed/`.
- **Discovery:** the sidebar sweep of 2026-09-22 (the switcher needs at least two entries, the
  filter editor, restore ordering at `workspace.rs:10336-10352`).
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
