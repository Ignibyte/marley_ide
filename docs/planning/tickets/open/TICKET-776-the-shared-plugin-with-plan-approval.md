# TICKET-776 — The shared plugin with plan approval and the question skill

- **Ticket:** LOCAL #776 (chore; #709's carry, as #724 was)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** none yet; minted when the harness ships.
- **Source ticket:** #726 (its harness request, D1) and #728 (D5); Chad, 2026-10-12: "send anything
  to harness also to work on", after which both went to rustal-harness as MREQ-012 and MREQ-013.
- **Status:** open (waits on rustal-harness MREQ-012 and MREQ-013)

## Summary
rustal-harness's shared Claude Code plugin learns two things Marley asked for: under Marley's host,
`tool.check` hands a call whose tool `MARLEY_APPROVE_TOOLS` names (Claude Code's `ExitPlanMode`) to
`"$MARLEY_BIN" approve`, so #726's Plan tab sees a real plan (MREQ-012); and a skill tells an agent
writing a plan to put each open question in a `:::question` block, which #728 draws as a card
(MREQ-013). Marley carries the plugin's files at a digest (#709): when the harness ships, Marley
takes its files and the new digest, and the Plan tab's guide text no longer says it waits on the
harness.

## Acceptance
With `marley.plan_review` and the shared plugin on, a real Claude Code in plan mode opens its plan
in the Plan tab when it asks to leave plan mode, and a plan with an open question shows its card.
