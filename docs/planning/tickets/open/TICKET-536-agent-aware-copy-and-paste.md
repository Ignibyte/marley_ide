# TICKET-536 — Copy and paste that know an agent is running

- **Ticket:** LOCAL #536 (feature, prong 1 T7: CLI agents in the terminal)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/536-agent-aware-copy-and-paste.spec.md
- **Source ticket:** Chad, 2026-09-25: specced at his request with every item decided that day (the brief quotes no words for this one). From the Orca survey: `docs/orca_architecture/05-terminal-and-workspace.md` §3 item 7, with §2.6 and §2.15, and the README's "Smaller things worth a day each"
- **Status:** open

## Summary
Three things go wrong when an agent CLI runs in Marley's terminal. Text copied out of Claude
Code's replies keeps the indentation every line of the reply carries, so it pastes indented. A
multi-line paste reaches the agent without bracketed-paste markers whenever the agent has not
turned bracketed paste on, and each line break can then submit part of the prompt. A dropped
image's path arrives shell-quoted with a space in front, which defeats the check Claude Code and
Codex make before they attach an image. While an agent CLI runs in the terminal, a copy drops the
leading spaces every non-blank line shares, a multi-line paste is always bracketed, and an image
file (PNG, JPEG, GIF or WebP) that is dropped or attached goes in as its raw path, alone in a
bracketed paste. A plain shell keeps Zed's behavior for all three.

## Acceptance
With an agent CLI in the terminal, an indented reply that is copied pastes flush left with its
relative indentation kept, a multi-line paste arrives bracketed, and a dropped image arrives as
its raw path in its own bracketed paste; in a plain shell, copy, paste and drops behave as in Zed.
