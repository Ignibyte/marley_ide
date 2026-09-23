# Voice input through Voxtype — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-480-voice-input.md
- **Pipeline spec:** 480-voice-input.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "Voice activation which would be great to have".
- **What the exploration found.** Voxtype 1.0.1 is installed and its daemon runs
  (`systemctl --user is-active voxtype`: active), model `base.en`, mode `type`. Omarchy binds F9
  to push-to-talk and Super+Ctrl+X to toggle. `voxtype status --format json` printed
  `{"alt": "idle", "class": "idle", …}`.
