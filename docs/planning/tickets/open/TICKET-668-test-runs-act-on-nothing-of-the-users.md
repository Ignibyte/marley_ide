# TICKET-668 — Test runs act on nothing of the user's

- **Ticket:** LOCAL #668 (chore, the e2e harness)
- **Owner:** claude-opus-5-5, 2026-10-06 (from the T3 Code survey, `docs/t3code_architecture/README.md`, "For Marley's own workflow")
- **Pipeline doc:** (none yet)
- **Source ticket:** `docs/t3code_architecture/07-engineering-and-releases.md`; T3 Code's `migrate-dev-db` keeps only what a run needs
- **Status:** open

## Summary
`script/e2e.sh` copies the user's `settings.json` into each run's profile and turns off Rusty,
Voice, agent prompts in a tab, Codex's App Server and Claude Code's IDE link (#633 to #653), but
keeps `marley.push` (the phone push topic and token file), the harness settings (`harness`,
`embedded_harness`), the Fleet panel's providers and hosts, and System One's settings, and the
run inherits `MARLEY_SYSTEM_ONE_KEY` and `MARLEY_CLOUDFLARE_API_TOKEN` from the environment. A
scenario could push to the user's phone, reach real hosts or spend System One's budget. The
user's settings hold none of these keys on 2026-10-06, so nothing has leaked; the risk is the next
one he sets. The copy shall leave out every setting that reaches outside the run, and the run
shall unset the two key variables, each scenario setting back only what it fakes.

## Acceptance
A run's profile holds no `marley.push`, harness, fleet or System One settings from the user's file
and its environment no System One or Cloudflare key unless the scenario sets them; the scenarios
that use them (535, 565, 548, the fleet and harness ones) still pass with their own fakes.
