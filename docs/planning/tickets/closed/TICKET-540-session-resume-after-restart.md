# TICKET-540 — Claude Code sessions resumed after a Marley restart

- **Ticket:** LOCAL #540 (feature, prong 2 sessions)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/540-session-resume-after-restart.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey's open question 9 ("Resume Claude Code sessions now (item 7), or wait for the harness to keep the processes alive?", docs/orca_architecture/README.md): specced, and it waits, since the embedded rustal-harness may keep terminals alive instead. Report 05 §2.2 and §3 item 3, report 01 §2.6 and §3 item 7.
- **Status:** closed (2026-10-01)
- **Backlog:** Deliberate. Once rustal-harness is embedded, its tmux server keeps terminal processes alive across a Marley restart, and a resume would start a second Claude Code on a session that is still running. Chad picks this ticket if he wants resume before the harness lands. It also needs #519, whose `SessionStart` gives the session id.

## Summary
A Marley quit ends every Claude Code session running in its terminals, and Zed brings each terminal
back as a new shell at its saved folder. With #519, a terminal knows its Claude Code session id and
the folder the session started in, from `SessionStart`. Marley saves both with the terminal, and
after a relaunch starts the restored terminal's shell in that folder and types
`claude --resume <id>` once the shell is ready, so the conversation comes back where it stopped. A
session that ended before the quit comes back as a plain shell, and one session resumes in one
terminal only.

## Acceptance
Quit Marley with Claude Code running in a terminal and relaunch it: that terminal runs
`claude --resume <id>` in the session's first folder, and it does so again after a second relaunch;
a terminal whose Claude Code had exited comes back as a plain shell.
