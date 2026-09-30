# The remaining work, in waves (2026-09-30)

Chad, 2026-09-30, after #598 to #606 shipped: "lets spec out the remaining tickets and we can
begin everything except the cloud flare ones". Everything left on the backlog, the plan and the
recent tickets' deferred lists, in the order it is built. Each wave is specced with `/spec` when
the one before it is under way, and built back to back.

## Decisions taken with the list

- **#548 (System One through Cloudflare):** left out, as asked.
- **#445 (Marley's own release identity):** later, when Marley ships a package; the `dev`
  channel keeps a second Marley safe until then.
- **T5 (the terminal's stage two, native headers with PS1 hidden):** built. This settles the
  plan's open decision 2.
- **#466 (fish):** fish may be installed on the dev box to build and test it.
- **The fleet (D20, `docs/marley/fleet-contract.md`):** Marley decides staleness from
  `last_seen_ms`; the host collector is a script piped over SSH; tokens only, no money in `v1`.
  The Rustal services are not ready, so the fleet's panes work from a pseudo provider first.

## Wave 1: the fleet (specced 2026-09-30)

| Ticket | What |
|---|---|
| #607 | The contract's types, a pseudo provider, and a Fleet panel in the right dock listing its agents by host |
| #608 | The selected agent's snapshot under the Fleet panel's list |
| #609 | The Agent tab in the center: phases, gates, events, resource history, tokens |
| #610 | The host collector: a script piped over SSH, a hosts setting, the join to store agents |
| #611 | The `marley.work/v1` clients over MCP and HTTP, with the source states |

## Wave 2: the rail's follow-ups

Moving a row to another group (from #602's deferred list); container ports in the rail and
restarting a service from its row (#603's); deleting and unarchiving a thread from the rail
(#605's); threads and ports under a closed project (#606's); the port row's clipped lines and a
header tooltip over its open menu; and #578, the restored Browser tab that sometimes draws
nothing.

## Wave 3: the terminal

T2 (a block's paths resolved against its folder, and jump to the first failure), the rest of T4
(tasks and runnables as blocks, failed blocks in the diagnostics), T6 (completions and colouring
at the prompt), the rest of T3 (the prompt editor), T5 (stage two), #466 (fish) and #573
(English at the prompt, second stage).

## Wave 4: the harness and Rusty

#534 (the harness's sessions in the rail, read side; its three blockers are done), shipping and
starting `rh` with Marley (D19), the harness's passthrough terminals (D10, C3), Rusty's sessions
in the rail (C2), and #540 (Claude Code sessions resumed after a restart).

## Wave 5: the test pass the workflow owes

Since 2026-09-29 each ticket is proven by a visual check alone; the unit tests, the golden
regression run and mutation testing were left for when the queue empties. This wave does them,
and takes in #475 (the shell tests' scratch data directory), which matters again once the tests
run.
