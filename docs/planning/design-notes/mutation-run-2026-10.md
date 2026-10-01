# Mutation run 2026-10: the Marley crates' pure cores

#636, 2026-10-01. The end-of-sprint mutation run the workflow keeps outside the gate
(AD-claude-the-workflow-is-four-phases-and-mutation-waits-for-the-end-001), over the nine Marley
crates with no gpui. Every missed mutant, with its place and its reading, is in
`mutation-run-2026-10-survivors.md` beside this note.

## How it ran

Since #638 the same run is `just mutants` on this machine, or `just mutants-cloud` on GitHub's
runners, split into shards (`docs/marley/mutation-runs.md`).

- `cargo-mutants` 27.1.0, `--no-config` (no exclusions, no masks), nextest, copy mode with two
  workers, each copy building in its own target (AD-claude-443-mutation-topology-and-no-masks-001,
  F-claude-443-a-full-mutation-workers-shared-one-target-001); the copies under
  `~/.cache/marley-mutants`, the output outside the repository.
- The unmutated baseline passed: #634 had made the suites green.
- 07:00 to 08:28, 1 h 27 min, for 2,658 mutants. Exit 3: some mutants timed out, which counts as
  detection (a hang is caught).
- Out of scope: `marley_browser` (1,197 mutants) and `marley_workbench` (5,442), which depend on
  gpui and whose suites reach little of them.

## Results

| Crate | Mutants | Caught | Timeouts | Missed | Unviable | Killed of viable |
|---|---|---|---|---|---|---|
| `marley_agent` | 685 | 21 | 0 | 626 | 38 | 3.2% |
| `marley_dcs` | 43 | 34 | 0 | 5 | 4 | 87.2% |
| `marley_fleet` | 38 | 33 | 0 | 0 | 5 | 100.0% |
| `marley_mcp` | 412 | 218 | 0 | 153 | 41 | 58.8% |
| `marley_rail` | 212 | 85 | 0 | 99 | 28 | 46.2% |
| `marley_remote` | 43 | 20 | 0 | 17 | 6 | 54.1% |
| `marley_sdk` | 100 | 93 | 0 | 0 | 7 | 100.0% |
| `marley_system_one` | 113 | 99 | 0 | 0 | 14 | 100.0% |
| `marley_terminal` | 1,012 | 358 | 12 | 548 | 94 | 40.3% |
| **All nine** | 2,658 | 961 | 12 | 1,448 | 237 | 40.2% |

Killed of viable is (caught + timeouts) / (caught + timeouts + missed).

## What the survivors are

The score is what the workflow predicts. Since #483 (2026-09-23) no ticket writes a unit test,
and the Marley crates' newer modules were proven by e2e scenarios instead: `marley_fleet`,
`marley_sdk` and `marley_system_one`, built with tests, kill everything; `marley_agent`'s
`risk`, `claude_events`, `stop_kind`, `stall`, `route` and `trust`, and `marley_terminal`'s
`links`, `workflow`, `failures`, `english`, `suggest`, `agent_commands` and the rest, have no
unit test at all, and nearly every mutant in them survives.

- **1,358 survivors sit in functions no test names.** 35 of the 68 source files have no unit
  test. These are untested by construction: no test calls the function, so no test can see it
  change. They were not read one by one; a bug among them would show only on reading or under a
  test.
- **90 survivors sit in functions a test names**, and each was read:
  - **2 equivalent:** `TerminalSession::shutdown` replaced by `()` (its `self` drops at the
    function's end either way), and `pty_os.rs:121`'s `!=` to `==` in the reap's error check,
    which changes only whether a log line is written.
  - **88 untested branches** of tested functions. A test reaches the function, but not the
    branch the mutant changes: the remote-connection paths of `anchored::apply` and the signed
    and `history`/`remote` frames of `dcs::decode_hook` (#526, #484); the ssh path of
    `shown_arguments` and the fish arm of `for_program` (#526, #466); the browser, port and
    worktree branches of the rail's `parent` and `rail_rows`; `marley_dcs`'s notification
    length cap at its boundary (`<` to `<=`); `marley_mcp`'s session cap (`>=` to `<`). Many
    of the 90 match a test only by a common word (`run`, `summary`, `program`, `matches`,
    `windows`, `holds`, `scan`, `tick`), and their functions have no test of their own.
- **No survivor read as a bug**, so no code changed.

### Worth a test first

If unit tests come back for survivors, these are where a wrong change would cost the most:

- `marley_mcp::clients`: `ClientTable::allow` replaced by `Ok(())` (the client's token never
  stored) and `touch` survive; outside clients' access (#524) is proven only by its scenario.
- `marley_mcp::registry::Family::is_served` replaced by `true` survives: `tools/list` would list
  the fleet and session families. Since #634 the dispatch test compares `tools/list` with the
  registry's own list, so it no longer pins the served set; the registry's tests list every tool,
  served or not.
- `marley_mcp::session::SessionRegistry::assign`'s per-client cap.
- `marley_terminal::dcs::decode_hook` and `anchored::apply` for signed and remote frames: they
  decide which frames a shell may be trusted for (#474, #526).

## Per file

| File | Missed | In functions a test names | Tests in the file or its `_tests.rs` |
|---|---|---|---|
| `crates/marley_agent/src/risk.rs` | 209 | 3 | 0 |
| `crates/marley_terminal/src/links.rs` | 168 | 3 | 0 |
| `crates/marley_terminal/src/anchored.rs` | 120 | 3 | 11 |
| `crates/marley_agent/src/claude_events.rs` | 119 | 0 | 0 |
| `crates/marley_rail/src/marley_rail.rs` | 99 | 29 | 34 |
| `crates/marley_agent/src/stop_kind.rs` | 90 | 1 | 0 |
| `crates/marley_agent/src/stall.rs` | 76 | 0 | 0 |
| `crates/marley_agent/src/marley_agent.rs` | 67 | 0 | 5 |
| `crates/marley_terminal/src/workflow.rs` | 54 | 0 | 0 |
| `crates/marley_terminal/src/failures.rs` | 47 | 0 | 0 |
| `crates/marley_mcp/src/registry.rs` | 42 | 1 | 3 |
| `crates/marley_agent/src/route.rs` | 39 | 0 | 0 |
| `crates/marley_mcp/src/transport.rs` | 34 | 0 | 19 |
| `crates/marley_mcp/src/clients.rs` | 31 | 5 | 0 |
| `crates/marley_terminal/src/english.rs` | 30 | 0 | 0 |
| `crates/marley_agent/src/trust.rs` | 26 | 0 | 0 |
| `crates/marley_terminal/src/suggest.rs` | 26 | 0 | 0 |
| `crates/marley_terminal/src/agent_commands.rs` | 22 | 14 | 0 |
| `crates/marley_terminal/src/running_errors.rs` | 20 | 9 | 0 |
| `crates/marley_terminal/src/filter.rs` | 19 | 0 | 0 |
| `crates/marley_mcp/src/find.rs` | 17 | 8 | 0 |
| `crates/marley_mcp/src/session.rs` | 17 | 1 | 8 |
| `crates/marley_remote/src/marley_remote.rs` | 17 | 0 | 10 |
| `crates/marley_terminal/src/paste.rs` | 12 | 0 | 0 |
| `crates/marley_terminal/src/shell_integration.rs` | 12 | 5 | 6 |
| `crates/marley_mcp/src/redact.rs` | 8 | 0 | 0 |
| `crates/marley_terminal/src/ports.rs` | 8 | 0 | 0 |
| `crates/marley_terminal/src/dcs.rs` | 4 | 4 | 21 |
| `crates/marley_terminal/src/identity.rs` | 4 | 0 | 0 |
| `crates/marley_dcs/src/notification.rs` | 3 | 1 | 6 |
| `crates/marley_dcs/src/marley_dcs.rs` | 2 | 0 | 12 |
| `crates/marley_mcp/src/dispatch.rs` | 2 | 0 | 12 |
| `crates/marley_mcp/src/marley_mcp.rs` | 1 | 1 | 0 |
| `crates/marley_mcp/src/tools.rs` | 1 | 0 | 4 |
| `crates/marley_terminal/src/session.rs` | 1 | 1 | 52 |
| `crates/marley_terminal/src/pty_os.rs` | 1 | 1 | 0 |
