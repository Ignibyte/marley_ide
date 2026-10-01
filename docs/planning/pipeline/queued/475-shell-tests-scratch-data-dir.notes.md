# The shell tests install Marley's scripts in a scratch directory — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-475-shell-tests-scratch-data-dir.md
- **Pipeline spec:** 475-shell-tests-scratch-data-dir.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 of `design-notes/remaining-work-2026-09-30.md` (Chad, 2026-09-30); the
  ticket was Deliberate until the tests run again, which #634 does.
- **Recall (§18.3):** #474's Phase 3 found the tests writing the user's scripts; AD-claude-483
  kept the tests in the tree, building, run by no gate; L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001.
