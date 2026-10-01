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

## Wave 2: the rail's follow-ups (specced 2026-09-30)

Wave 1 shipped on 2026-09-30 (#607 to #611).

| Ticket | What |
|---|---|
| #613 | Move a terminal to another project in the rail, its shell kept (#602's deferral) |
| #614 | Container ports in the rail, named by their container, Stop stopping it (#603's) |
| #615 | Restart a service from its port row, with its state and logs (#603's) |
| #616 | Delete a thread, and unarchive threads, from the rail (#605's) |
| #617 | A closed project's threads and ports under its header (#606's) |
| #618 | A port row's clipped lines, and a header tooltip over its open menu (Test notes of #603, #606) |
| #612 | A hand edit to settings.json after Marley writes it does not reload (found in #607's Test) |
| #578 | The restored Browser tab that sometimes draws nothing (queued since 2026-09-26) |

## Wave 3: the terminal

T2 (a block's paths resolved against its folder, and jump to the first failure), the rest of T4
(tasks and runnables as blocks, failed blocks in the diagnostics), T6 (completions and colouring
at the prompt), the rest of T3 (the prompt editor), T5 (stage two), #466 (fish) and #573
(English at the prompt, second stage).

First batch (specced 2026-09-30), T2 and the rest of T4:

| Ticket | What |
|---|---|
| #619 | A block's path links resolve against the block's own folder |
| #620 | Jump to a failed block's first failure |
| #621 | A task's run as a block |
| #622 | A task block's Rerun and its pill |
| #623 | A failed block's errors as project diagnostics |

Second batch (specced 2026-09-30), T3 and T6, the prompt, then #573 on it:

| Ticket | What |
|---|---|
| #624 | A prompt editor at the shell's prompt, on a key |
| #625 | Completions in the prompt editor |
| #626 | A command's colours at the prompt |
| #627 | The prompt editor by default, with the raw-passthrough ladder |
| #573 | English at the prompt, second stage (queued since 2026-09-26) |

Third batch (specced 2026-09-30), fish and T5 stage two:

| Ticket | What |
|---|---|
| #466 | Shell integration for fish |
| #628 | Blocks with native headers, PS1 hidden |
| #629 | Block navigation in display rows |
| #630 | Block density: two-line headers and gaps |

## Wave 4: the harness and Rusty

#534 (the harness's sessions in the rail, read side; its three blockers are done), shipping and
starting `rh` with Marley (D19), the harness's passthrough terminals (D10, C3), Rusty's sessions
in the rail (C2), and #540 (Claude Code sessions resumed after a restart).

## Wave 5: the test pass the workflow owes

Since 2026-09-29 each ticket is proven by a visual check alone; the unit tests, the golden
regression run and mutation testing were left for when the queue empties. This wave does them,
and takes in #475 (the shell tests' scratch data directory), which matters again once the tests
run.
