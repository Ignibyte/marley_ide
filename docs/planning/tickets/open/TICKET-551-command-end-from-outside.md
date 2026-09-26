# TICKET-551 — A command's end, seen from outside its terminal

- **Ticket:** LOCAL #551 (feature, prong 1 T7b with the rail: plain commands report their end the way agents do)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/551-command-end-from-outside.spec.md
- **Source ticket:** The Warp second pass of 2026-09-25, finding 3 and its third recommendation (`docs/planning/design-notes/warp-second-pass-2026-09-25.md`), with Chad's answer of 2026-09-26 to open question 4 left at its default: Warp's 30 seconds, plain commands only, never agent terminals. Specced because Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** open

## Summary
A `cargo build`, a test run or an `rsync` asks for no notification, so none comes, and the
rail row of the terminal it runs in says nothing about it. Warp posts a desktop notification
when a command ends after 30 seconds while Warp is in the background, and when a command
waits for a password; its tab marks an error and its session list shows the running or last
command. Marley watches each terminal's blocks: when one ends after
`marley.long_command_seconds` (30 by default, 0 for off) while its terminal is not the focused
one of the active window, a notification titled with the command says `done in 45 s` or
`exit 1 after 4 m 12 s`, through the notify-and-click path #478 built. The terminal's rail row
gains a line with the running command, or the last command's result with a red mark on a
failure, and the rail's filter matches the command. A password prompt shows in the PTY's own
flags: canonical mode with echo off while a block runs means `sudo`, `ssh` or `read -s` is
reading a password, which the row says and one notification reports. Agent terminals are left
out: their session is one long block, and #538 words their banners.

## Acceptance
A command over the threshold ending in an unfocused terminal posts one notification with the
command and its verdict, and its row shows the result with the red mark; a short command
posts none; a running command shows on the row; a password read shows `waiting for a
password` on the row and posts once; the filter finds the row by the command; a block in an
agent's terminal posts nothing.
