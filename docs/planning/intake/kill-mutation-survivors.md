---
status: intake
created: 2026-10-01
ticket: <unassigned>
pipeline_spec: <unassigned>
---

# Tests to kill the mutation run's survivors (a decision for Chad)

## What
Whether, and where, Marley writes unit tests again to kill the survivors of #636's mutation run:
1,448 missed mutants in the nine pure-core crates, 40.2% of viable mutants killed
(`docs/planning/design-notes/mutation-run-2026-10.md`, every survivor in
`mutation-run-2026-10-survivors.md`).

## Why
The survivors are not bugs: none read as one. They are the cost of the rule since #483 that no
ticket writes a unit test (CONSTITUTION §0, Chad, 2026-09-23): the modules built since are proven
by e2e scenarios, which hold the behavior a person sees, not each branch. The crates built with
tests (`marley_fleet`, `marley_sdk`, `marley_system_one`) kill every mutant; the ones built after
have almost no unit test (`marley_agent` 3.2%).

The rule forbids the fix, so this is Chad's call. The choices:

1. **Keep the rule.** Accept the score; the golden e2e set (#635) stays the regression net. Rerun
   the mutation pass at each sprint's end and read only new survivors in tested files for bugs.
2. **A survivors pass at the end of each sprint.** Unit tests are written only to kill survivors,
   in a ticket of their own, the way the workflow keeps mutation for the end. The per-ticket rule
   stays as it is.
3. **Tests for the risky paths only.** Kill the survivors listed below, which guard access and
   trust, and leave the rest to e2e.

## Notes
The survivors worth a test first (the findings doc's "Worth a test first"):

- `marley_mcp::clients::ClientTable::allow` replaced by `Ok(())`, and `touch`: outside clients'
  access (#524) is proven only by its scenario.
- `marley_mcp::registry::Family::is_served` replaced by `true`: `tools/list` would list the fleet
  and session families. #634 made the dispatch test compare with the registry's own list, so no
  test pins the served set now.
- `marley_mcp::session::SessionRegistry::assign`'s per-client cap.
- `marley_terminal::dcs::decode_hook` and `anchored::apply` for signed and remote frames (#474,
  #526): which frames a shell is trusted for.
- `marley_dcs::notification::NotificationScanner::push` at its length cap.

Then, by size: `marley_agent::risk` (209 survivors), `marley_terminal::links` (168),
`marley_terminal::anchored` (120), `marley_agent::claude_events` (119), the rail (99),
`marley_agent::stop_kind` (90), `marley_agent::stall` (76).

The run's topology (copy mode, two workers, a target per copy, `--no-config`, nextest, the copies
under `~/.cache`) took 1 h 27 min for the nine crates; `marley_workbench` (5,442 mutants) and
`marley_browser` (1,197) would need a test reach they do not have before a run means much.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
