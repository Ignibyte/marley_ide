# TICKET-478 — Desktop notifications from a terminal, and from Claude Code

- **Ticket:** LOCAL #478 (feature, prong 1: T7b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/478-terminal-notifications.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7)
- **Status:** open

## Summary
Warp tells you when a CLI agent needs you: a chip installs Warp's plugin for Claude Code, whose
hooks report a finished turn, a permission prompt or an idle prompt, and Warp shows a
notification. Claude Code can already ask a terminal to notify, with the desktop-notification
escapes OSC 9 and OSC 777, but its default channel recognizes only iTerm2, kitty, Ghostty and
Apple Terminal, and Zed's terminal ignores both escapes. Marley reads them and shows a desktop
notification when the terminal is not in front of you, and the agent bar offers "Enable Claude
Code notifications", which installs a small Marley plugin for Claude Code.

## Acceptance
A notification escape from a terminal you are not looking at becomes a desktop notification
that takes you to it when clicked; the chip installs Marley's plugin, whose hooks send the
escape only inside Marley. The EARS criteria are in the spec.
