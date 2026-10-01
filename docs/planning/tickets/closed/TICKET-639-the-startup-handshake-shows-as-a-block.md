# TICKET-639 — The startup handshake shows as a block

- **Ticket:** LOCAL #639 (bug, prong 1 blocks)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/completed/639-the-startup-handshake-shows-as-a-block.spec.md
- **Source ticket:** found in #540's dry run (`540-02-resumed`)
- **Status:** closed (2026-10-01)

## Summary
Marley types a command into a new or restored terminal after Zed's startup handshake: the shell
runs `printf '%s%s%s\n' __zed_init_command_ready_ 1 __`, Zed waits for the marker, then writes
the command (`start_in_terminal` for every agent CLI, `resume::resume_restored` since #540). The
shell's hooks report the handshake's `printf` as a command like any other, so the terminal shows
it as a block, header and all, and the command Marley typed after it reads as that block's
output. The handshake is Zed's plumbing, not the user's command.

## Acceptance
An agent CLI started from Marley, and a resumed session, show no block for the handshake's
command; the command Marley typed is the first block.
