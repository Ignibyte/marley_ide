# Enable Claude Code notifications: Marley's plugin for Claude Code — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-482-claude-code-notifications-chip.md
- **Pipeline spec:** 482-claude-code-notifications-chip.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23, the first of the five Warp pieces: "There is a 'enable claude
  code notifications' not sure how it actually works but lets explore it". Split from #478 at
  its promotion, when the terminal side and the plugin proved two tickets' work.
- **What the exploration found.** `~/.claude/plugins/installed_plugins.json` is
  `{"version": …, "plugins": {"<name>@<marketplace>": …}}`; `claude plugin marketplace add`
  takes a path. Claude Code 2.1.281's hook answer may carry `terminalSequence`.
