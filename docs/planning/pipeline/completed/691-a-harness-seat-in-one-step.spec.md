---
pipeline_id: 6f470be4-2825-4ba6-a250-36e06c0688f7
ticket: docs/planning/tickets/closed/TICKET-691-a-harness-seat-in-one-step.md
status: Phase 4 — Complete PASS
title: A harness seat in one step
type: feature
slice: phase 2 item 6 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/689-harness-writes-from-marley.spec.md
---

## Title
`marley: new harness seat`: a form that runs the harness's `seat add` and `seat start` and opens
the seat's tab.

## Scope
### In
- **The form.** A modal with Name, an Agent choice (Claude Code or Codex), Folder (the active
  project's folder at first) and Role. Create runs the seat commands, and Escape closes it.
- **The command.** It is derived from what Marley follows:
  - for a `marley.harness` command, its program and its arguments up to its `mcp`, then
    `seat …`. For `ssh HOST rh --state ROOT mcp --grant write` that is
    `ssh HOST rh --state ROOT seat …`;
  - for the embedded harness, the `rh` it found with `--state <data dir>/harness`.
- **The run.** `seat add NAME --agent claude|codex --cwd DIR [--role ROLE]`, then
  `seat start NAME`. Each answers one JSON object on stdout, or `rh: CODE: reason` on stderr,
  which the form shows. On success the form closes and the new session's tab opens.
- **The gate.** Listed in the palette only while `writes_on`.

### Out (explicitly deferred)
- Choosing among fleet hosts. The form targets the harness Marley follows.
- `--model`, `--binary` and the supervision flags.
- The Marley agent's tool for the same: TICKET-692.

## Reference (§20)
N/A — Marley-specific. The commands are the harness's (its `AGENT_SEATS.md`, "A seat in one
step", TICKET-109).

### Prior art
- The harness's TICKET-109 shape, sent 2026-10-07 and filed under the plan's item 6.
- `process::output`, the adapter for one-shot programs.
- `clients.rs`'s `BrowserClientsModal`, a modal with an editor.
- `harness::open`, for the session's tab.

## UI proof
`script/e2e/691-a-harness-seat-in-one-step.sh`. Its `rh` stand-in answers `seat` from a script, and
its `seat start` opens a real actor. Everything else goes to the harness's own `rh`.
- `691-01-form`: the form filled in.
- `691-02-opened`: the new seat's tab.
- `691-03-refused`: the form showing `seat_role_reserved`.

## Locked-In Decisions
- **D1:** the seat command comes from the followed harness's command, cut before `mcp`. One
  harness, wherever it runs.
- **D2:** behind `marley.harness_writes`, as #689.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the write verbs are on, `marley: new harness seat` shall open a form with Name, Agent, Folder and Role | shot `691-01-form` |
| REQ-002 | WHEN Create is pressed, the system shall run `seat add` with the form's values and then `seat start` through the followed harness's command, and open the session's tab | shot `691-02-opened` and a check of the stand-in's log |
| REQ-003 | WHEN the harness refuses, the form shall stay open and show the refusal's code and reason | shot `691-03-refused` |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** `harness_seat.rs`, `harness.rs`'s `seat_command`, a review, and the gate.
- **P3 Test:** the scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
