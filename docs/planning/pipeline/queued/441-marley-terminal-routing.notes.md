# Terminal routing and keys in the Marley layout — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-441-marley-terminal-routing.md
- **Pipeline spec:** 441-marley-terminal-routing.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad chose "Hide, route to center" for the bottom panel (2026-09-22).
- **Classification / tier:** feature, medium; `marley_workbench` plus one `zed.rs` line.
- **Recall (§18.3):** `PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`;
  `PR-claude-new-chord-shadowed-by-hardcoded-key-001` (check each new chord against Zed's
  defaults in every context it can reach); `PR-claude-key-arm-above-the-keymap-must-gate-on-modifiers-001`.
- **Discovery:** the terminal sweep of 2026-09-22 listed every path that forces the panel
  (NewTerminal, OpenTerminal, tasks, vim, agent login, the toggles) and the two zero-touch
  overrides (the provider and capture-phase actions); the defaults sweep showed why
  `cx.bind_keys` at init cannot work.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
