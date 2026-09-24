# TICKET-482 — Enable Claude Code notifications: Marley's plugin for Claude Code

- **Ticket:** LOCAL #482 (feature, prong 1: T7b, second half)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/482-claude-code-notifications-chip.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7), split from #478
- **Status:** closed

## Summary
#478 turns OSC 9 and OSC 777 into desktop notifications, but Claude Code sends neither in
Marley: its `auto` channel does not recognize the terminal. Warp's answer is a chip that
installs its plugin for Claude Code. Marley's agent bar gets "Enable Claude Code notifications",
which installs a small Marley plugin whose hooks ask Claude Code to send OSC 777 when it wants
permission, waits for you, or finishes.

## Acceptance
While Claude Code runs without Marley's plugin, the chip shows; clicking it installs the plugin
through Claude Code's own `claude plugin` commands; its hooks answer with OSC 777 in Marley's
terminals and with nothing elsewhere. The EARS criteria are in the spec.
