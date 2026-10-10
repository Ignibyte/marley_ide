---
pipeline_id: 798db173-afae-496b-988c-9a30c0410b2c
ticket: docs/planning/tickets/open/TICKET-740-one-harness-per-host.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: One harness per host
type: feature
slice: the control plane (docs/marley/three-prong-plan.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/completed/534-harness-sessions-in-the-rail.spec.md
---

## Title
Marley follows a harness on each host the settings list, not just one. The harness settled that a
fleet holds agents on several machines and that Marley reaches each box over its own SSH connection
(rustal-harness D164); Marley's `Harness` global holds one connection today, so a remote box
replaces the local one.

## Scope
### In
- A setting `marley.harnesses`: a list of `{ "name": …, "ssh": "HOST", "state": "ROOT", "rh": "rh" }`.
  `ssh` is absent for this machine and `rh` defaults to `rh`. Marley builds every command from it:
  `[ssh HOST] rh --state ROOT mcp` to follow it, `… seat …` for seats, and (#741) `ssh -t HOST …`
  for the commands a session's Views name. `marley.harness` (a raw command) and
  `marley.embedded_harness` stay as the unnamed local harness.
- The `Harness` global holds one connection per name, each with its own source, server, snapshot,
  restart backoff and stale state.
- The rail's Harness section lists each harness under a sub-header with its name and its own
  connection line; one that fails says why under its name while the others go on.
- Session tabs, the inbox's harness entries and the Manager entry (#694) carry their harness's
  name, and every write (`session_answer`, `session_send`, seats) goes to the session's own harness.
- The seat form (#691) gains a Harness choice when more than one is followed.
- `docs/marley/guide.md`: several harnesses.

### Out (explicitly deferred)
- Starting an agent on a host from New Agent: #741.
- A Manager per harness in the Agent Panel: the entry follows the first harness that has a manager,
  as today; per-harness managers wait for Chad's word on one manager per project (harness ROADMAP
  open choice 2).

## Reference (§20)
N/A — Marley-specific: the harness client (#534, #632) and the harness's own rule that Marley
reaches each box over its own SSH connection (rustal-harness D164). No Zed or Warp behavior covers
it.

### Prior art
- **The code we ship:**
  - `harness.rs`: `Harness` (a single `Global`, 137-162), `follow_setting` (499-534) mapping the
    settings to `Source::{Command, Embedded, Off}`, `Harness::seat_command` (225-241, the base
    before `mcp`), the backoff and stale rows.
  - `harness_seat.rs` `Seat::commands` (80-98) and `NewSeatModal`.
  - `settings_content/src/marley.rs:76-94`: the two settings today.
  - `context_server::ContextServer`: one per harness, as now.
- **Behavior maps:** none.
- **Published material:** MCP (each harness is an MCP server).

## UI proof
`script/e2e/740-one-harness-per-host.sh`, under `compositor sway`. Two harness roots from the built
`rh` (#710's set-up): `local` (no `ssh`), and `box-2` (`ssh: box-2`) reached through a fake `ssh`
on the PATH that drops its host argument, logs the command and runs the rest here. Each root has one session.

Shots:
- `740-01-two`: the rail's Harness section with `local` and `box-2`, each with its session.
- `740-02-seat`: the seat form with a Harness choice; a seat made on `box-2` shows under `box-2`,
  and the fake `ssh` log holds `seat add`.
- `740-03-one-down`: `box-2`'s root stopped: `box-2` marked stale with its reason, `local` live.

## Locked-In Decisions
- **D1:** a list of named harnesses beside the existing settings, so nothing set today changes.
  Each names its host, root and program rather than a raw command, so Marley knows which part of a
  command reaches the host: the seat form and #741's attach need that, and today's
  `seat_command` only guesses it from the position of `mcp`.
- **D2:** every write goes to the harness the session came from; a session is named by harness and
  id together.
- **D3:** one Manager entry, from the first harness with a manager, until Chad decides how many
  managers there are.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.harnesses` names two harnesses, the rail shall list each one's sessions under its name. | Shot 740-01 |
| REQ-002 | WHEN a seat is made with the second harness chosen, the system shall run the seat commands through that harness's command and list the seat under its name. | Shot 740-02 and the fake ssh log |
| REQ-003 | IF one harness stops answering, THEN the rail shall mark it stale with its reason and keep the other's rows live. | Shot 740-03 |
| REQ-004 | WHEN a session's question is answered or a message sent, the system shall send it to that session's own harness. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `settings_content` (the setting; a Zed-crate touch with its ledger row, beside the
  existing Marley settings), `harness.rs`, `harness_seat.rs`, `rail.rs`, the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
