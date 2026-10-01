---
status: intake
created: 2026-10-01
ticket: <unassigned>
pipeline_spec: <unassigned>
---

# A harness session's live terminal (C3's observer half, plan D10)

## What
A harness session's tab in Marley shows the session's live terminal, a display-only Zed terminal
fed by the harness's own terminal stream, keys reaching nothing; a controller claim follows later.

## Why
#534's tab shows `session_read`'s rendered lines, without colours or full-screen programs. D10
promises the session's real terminal, the way `rh view` shows it.

## Notes
An Explore read of the harness (2026-10-01) found D10 not ready as the plan writes it:
- No real agent seat has a terminal: Codex runs as its App Server and Claude Code as `claude -p`
  with stream-json (the harness's `MCP.md`: "A Codex or Claude session has no terminal"). Only
  actor seats (scripted fixtures in an owned tmux pane) and plain `rh new` workspaces do.
- The terminal stream exists and is versioned (protocol v1, the harness's `CAPTURE.md`,
  `SCREENS.md`, `SUBSCRIPTIONS.md`): the byte archive (`rh output --raw`, read from the journal),
  `subscribe` events (`output_chunk` ranges, not bytes), and `screen` frames (whole styled screens
  with their size, sampled, lossy). All of it is local and same-user, over `runtime.sock` or the
  CLI; none of it is on `rh mcp`, which D19 names as the one protocol, so a remote harness is left
  out.
- `SCREENS.md` offers no way to seed a terminal from a screen and then append bytes; the stream
  carries no resizes (`ignore-size`), so a Marley observer must draw on the pane's grid, not its
  own; Zed's `write_output` turns every bare LF into CRLF, which changes raw bytes from full-screen
  programs.
A slice that could be specced now, if Chad accepts a local-only path: an observer for actor seats
and plain workspaces only, fed by `screen` frames (or the archive from offset 0) through the local
`rh` CLI, in a display-only terminal pinned to the pane's size, in a Marley item that is never
restored as a shell.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`. It waits for Chad on the local-only
question, or for the harness to carry a terminal stream over `rh mcp`.
