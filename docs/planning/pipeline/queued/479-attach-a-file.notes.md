# Attach a file to an agent's prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-479-attach-a-file.md
- **Pipeline spec:** 479-attach-a-file.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "Attach a file which basically just provides the location of a
  file after choosing it in the claude session".
- **Recall.** The gpui-era Marley inserted paths from its file finder and its tree
  (`docs/marley/history/CHANGELOG-gpui-era.md`); not ported. Zed's file finder cannot return a
  path to a caller; its path prompt can.
