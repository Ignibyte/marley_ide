# Agent CLIs in rail terminals — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-440-rail-agent-clis.md
- **Pipeline spec:** 440-rail-agent-clis.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** the Warp-style agent Chad works with daily: `claude` or `codex` in a terminal,
  listed under its project.
- **Classification / tier:** feature, medium; `marley_workbench` plus the pure `marley_agent`.
- **Recall (§18.3):** the gpui-era agent cockpit (#72-#80, #187) shipped `agent_kind_of`,
  `agent_status_from` and the Waiting threshold as pump ticks; the fork has no 16ms pump, so
  the threshold becomes a duration. `PR-claude-raw-input-passthrough-must-filter-platform-chords-001`
  does not apply (no key routing here).
- **Discovery:** on this box `claude` is `~/.local/bin/claude` → `~/.local/share/claude/versions/2.1.280`
  (native), `codex` and `opencode` come from mise shims, `gemini` is a shell script in
  `~/.local/bin`. The terminal sweep of 2026-09-22 found the handshake pair and argv-based
  naming.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
