# TICKET-552 — Notification setup for Codex and OpenCode from the agent bar

- **Ticket:** LOCAL #552 (feature, prong 1 T7b: agent notifications; the Warp second pass, "Smaller")
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/552-codex-and-opencode-notifications.spec.md
- **Source ticket:** docs/planning/design-notes/warp-second-pass-2026-09-25.md ("Notification setup for Codex and OpenCode"; Chad, 2026-09-26: every remaining finding gets built)
- **Status:** open

## Summary
The agent bar offers "Connect Claude Code to Marley" and nothing for Codex or OpenCode, though
Marley shows any OSC 9 or OSC 777 a program prints. Under a Codex terminal the bar gains "Turn on
Codex notifications", which sets three keys under `[tui]` in Codex's `config.toml` (its own
desktop notifications, always, as OSC 9) with comments kept; under an OpenCode terminal it gains
"Connect OpenCode to Marley", which writes a small plugin file into OpenCode's plugin folder that
prints an OSC 777 on a turn's end, a permission request and an error, and only in a Marley
terminal. Each chip shows while its setup is missing, says "restart" when done, and offers an
update when Marley ships a newer plugin file. Warp does the same from its agent bar.

## Acceptance
A Codex terminal's bar shows the chip until `config.toml` carries the keys; the click writes them
and keeps the file's other content; an OpenCode terminal's bar shows its chip until the plugin
file is current; the click writes it; both stay silent outside Marley's terminals; a Marley
terminal shows the notification each mechanism sends. The full EARS criteria live in the
pipeline spec.
