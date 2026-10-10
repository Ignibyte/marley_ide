# TICKET-724 — The shared plugin with Rustal STE

- **Ticket:** LOCAL #724 (chore; #709's carry)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [724-the-shared-plugin-with-rustal-ste.spec.md](../../pipeline/completed/724-the-shared-plugin-with-rustal-ste.spec.md)
- **Source ticket:** Chad, 2026-10-09: "also for harness communication we should use this:
  https://github.com/danyuchn/asd-ste100-skill".
- **Status:** closed

## Summary
The skill is in Rusty's store as `rustal-ste` (adapted from ASD-STE100 at 32511c6, with Rustal's
glossary, verbs and message shapes; store commit cb90144), so sessions on this box have it. rustal-harness was asked, after its TICKET-114, to
ship it in the shared Claude Code plugin and name the rule in its agent-mail tools and in the
manager's and foreman's instructions. Marley carries the plugin's files at a digest (#709): when
the harness ships the new plugin, Marley takes its files and the new digest. Consider at the same
time a line in Marley's own `thread_post` description and the Marley agent's instructions, for
messages Marley's tools carry between agents.

## Acceptance
A new Marley terminal loads the shared plugin at the harness's new digest, and its skill list
holds `rustal-ste`.
