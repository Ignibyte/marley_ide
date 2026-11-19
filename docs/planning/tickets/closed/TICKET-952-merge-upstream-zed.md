# TICKET-952 — Merge upstream Zed through 2026-11-19

- **Ticket:** LOCAL #952 (chore, upstream sync)
- **Owner:** d8824430-9091-4135-8d83-d2ccacb0fd86
- **Pipeline doc:** [952-merge-upstream-zed.spec.md](../../pipeline/completed/952-merge-upstream-zed.spec.md)
- **Source ticket:** Chad, 2026-11-19: "lets bring down latest zed updates into our fork. its been a while. fix any conflicts"
- **Status:** closed

## Summary
The fork carries upstream Zed's main of 2026-10-29 (`0373366f43`, #950). Upstream main has 266
more commits through `3d3bd04fad` (2026-11-19), across 715 files, which hold the v1.27 to v1.29
releases and v1.30.1-pre. A trial merge conflicts in eleven files. Upstream moved its agent rules
from `.rules` into `AGENTS.md` and deleted the `CLAUDE.md` and `GEMINI.md` links (#66716), which
is how Claude Code sessions here read Marley's rules. Upstream also removed two items Marley calls:
`ProjectDiff::deploy_at_project_path` (#67149) and `Case::smart_if_uppercase_in` (#67151), and
renamed `root_thread_has_pending_tool_call` to `root_thread_is_waiting_for_input` (#66708).

## Acceptance
The fork builds on upstream's main of 2026-11-19 with every Marley hunk kept, agent sessions in the
repository still read Marley's rules, the review tools still open the project diff at a file, the
gate is green, and the merged release build starts in the Marley layout.
