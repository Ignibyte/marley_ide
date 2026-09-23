# Desktop notifications from a terminal, and from Claude Code — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-478-terminal-notifications.md
- **Pipeline spec:** 478-terminal-notifications.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "There is a 'enable claude code notifications' not sure how it
  actually works but lets explore it".
- **What the exploration found.** Warp's chip installs its plugin
  (`claude plugin install warp@claude-code-warp`). Claude Code 2.1.281 writes OSC 9 (iTerm2),
  OSC 99 (kitty), OSC 777 (Ghostty) or BEL itself, but its `auto` channel recognizes only
  iTerm2, kitty, Ghostty and Apple Terminal, so it sends nothing in Zed's terminal. Its hooks
  can return a `terminalSequence` for it to emit. `claude plugin marketplace add` takes a local
  path; `~/.claude/plugins/installed_plugins.json` lists what is installed.
- **Recall.** PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: a frame
  from output may be forged, so a notification only shows text. gpui's system notifications
  have no caller in Zed yet.
