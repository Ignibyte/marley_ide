# TICKET-648 — Marley checks the agent's version before an untested integration turns on

- **Ticket:** LOCAL #648 (feature, agents on their own tools B7; T-series agent terminals)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/648-agent-version-check.spec.md
- **Source ticket:** `docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`
  (B7); Chad, 2026-10-02: "Your install, with a check"; the batch #648 to #653, in build order
  after #640 to #647
- **Status:** closed (2026-10-04)

## Summary
Marley runs whatever `claude` and `codex` the user has installed, and Claude Code updates itself
several times a week. B1 to B3 will lean on surfaces that can change with any release, and one of
today's already does: the tags Marley reads to tell the user's prompts from the ones Claude Code
injects, which Claude Code never documents. This ticket gives Marley a check instead of a pin. It
reads each agent's version with `--version`, at start and when the file on its search path
changes, and compares it with a table in `marley_agent::versions` of the range each such
integration was tested on. Outside its range the integration stays off, and a chip in the agent
bar says so, with the version and path found, the range tested, and the setting
`marley.allow_untested_versions`, off by default, that turns it on for trying a new release. The
one row today is the prompt tags; Claude Code now documents `terminalSequence`, so the hook channel
itself is not gated. B1 to B3 register their rows when they land.

## Acceptance
Marley reads the version of the `claude` and `codex` it would run, once at start and again when
the file changes, and logs each read. While Claude Code is in the tags' tested range, the rail
keeps the user's prompt through an injected one; on an untested version, an unreadable one or a
missing program the tags are off, every prompt reads as the user's, and the agent bar shows a chip
whose tooltip says why and whose click opens the setting. With the setting on, the tags work on any
version and the chip goes. A remote terminal keeps today's reading. The e2e harness allows the row
so no other scenario depends on the user's installed version.
