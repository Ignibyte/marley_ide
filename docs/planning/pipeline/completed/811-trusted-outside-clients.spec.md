---
pipeline_id: 026e301b-b2e6-4a1a-ad78-44cd22a9c09c
ticket: docs/planning/tickets/open/TICKET-811-trusted-outside-clients.md
status: Phase 4 — Complete PASS
title: Trusted outside clients, paired with a code
type: feature
slice: Marley's MCP server (#491, #524, #584, #755)
references:
  - docs/planning/pipeline/completed/524-trusted-outside-browser-access.spec.md
  - docs/planning/pipeline/completed/584-outside-clients-from-other-machines.spec.md
  - docs/planning/pipeline/completed/755-three-small-hardening-fixes.spec.md
---

## Title
A program outside Marley gets write or run powers only after it pairs: it asks with
`marley-mcp-bridge pair NAME`, Marley shows its name and a six-digit code, and the user picks what
it may do (Read; Read and Write; Read, Write and Run) or refuses. A program that has not paired
reads and nothing more. Chad, 2026-10-17: "registering a mcp server to marley needs to be trusted
not just open and then given permissions (read, write, execute etc)"; 2026-10-18 he chose the
pairing prompt ("lets make it happen").

## Scope
### In
- **Who is Marley's own.** The bearer in `mcp-endpoint.json` alone no longer gives every tool. A
  call is Marley's own when it carries that bearer and proves a Marley terminal (#755's token), or
  carries a second per-run bearer that only Marley's own launches get: Zed's agents' context
  server reads it from `<data>/mcp/own-endpoint.json` (0600, gone at quit).
- **Unpaired programs.** The shared bearer without a proven terminal: Read-tier tools only, listed
  and callable; every other tool refused with `not_paired`, whose next steps say how to pair. No
  standing stream (GET 405) and no resources, as a client.
- **Powers.** A paired client has one power: Read, Write (Read and every write tool) or Run (Write
  and the tools that run a command or a program, start or stop an agent, or answer an agent's
  tool call: `terminal.write`, `harness.write`, `agents.write`, `thread_answer`). The approvals,
  Take Over and the kill switch apply as before.
- **Pairing.** `marley-mcp-bridge pair NAME` (on this machine, or over SSH from another) calls the
  unlisted `client_pair` and `client_pair_status`. Marley shows a notification in every window: the
  name, the folder it asked from, the code, and Read, Read and Write, Read Write and Run, Refuse,
  for two minutes. Trust lets the client in as Browser Clients does (a token per start in its
  0600 endpoint file, the list in `clients.json`), with its power; the command prints the line to
  run the bridge with, here and over SSH.
- **The list.** Browser Clients becomes Outside Clients (`marley: outside clients`; `marley: browser
  clients` still opens it): a paired client's row shows its power with Read, Write and Run to
  change it, and Cut Off. A client allowed by name there keeps the browser tools alone.
- `marley-open-url` proves its terminal, as the bridge and `marley-edit` do.

### Out (explicitly deferred)
- A process of the same user reading `own-endpoint.json`, a terminal's environment or a client's
  endpoint file: same-user isolation, as AD-claude-755 leaves it.
- Pairing from inside an agent's conversation; harness seats in tmux pair as any program does.
- A setting for what unpaired programs get.

## Reference (§20)
N/A — Marley-specific: Marley's own MCP server and its clients (#491, #524, #584). The pairing
follows the device-code pattern (a code shown on both sides, confirmed on the trusted screen) as
Bluetooth pairing and OAuth 2.0's device authorization grant (RFC 8628) use it.

### Prior art
- The code we ship: `marley_mcp::clients` (`ClientTable`, `Principal`, `permits`, tokens per
  start), `transport::allow_client`/`cut_off_client`, the workbench's `clients.rs` (registry,
  endpoint files, Browser Clients, the SSH line of #584), #755's terminal token
  (`identity::token_matches`), `agent_control`'s app-wide question notification, the registry's
  tiers and grant classes, which already split read tools from write tools.
- RFC 8628 (device authorization grant): a short user code shown by the device, confirmed by the
  user on a trusted screen, polled for by the device.
- Zed's crates own no client trust for a server Zed itself serves; none to adopt.

## UI proof
Scenario `script/e2e/811-trusted-outside-clients.sh` (`compositor sway`), driving the bridge as a
program outside Marley's terminals:
- `811-01-prompt`: `marley-mcp-bridge pair laptop` running; the notification names `laptop` and
  the code the command printed, with its four buttons (REQ-002).
- `811-02-clients`: Outside Clients after Read and Write: `laptop` with Write pressed (REQ-003,
  REQ-005).
- `811-03-run`: after Run in the list, the client's `terminal_type` into a Python REPL: the
  terminal's bar names `laptop` (REQ-004).
- Run checks: unpaired `terminal_list` answers and `terminal_type` is refused `not_paired`
  (REQ-001); the pair command prints the code and the endpoint line (REQ-002); the paired client's
  `terminal_type` is refused `not_granted` at Write (REQ-004); after Cut Off it is refused
  (REQ-006); the own endpoint's caller still types (REQ-007).

## Locked-In Decisions
- D1 — Pairing, not a list kept in settings (Chad, 2026-10-18).
- D2 — Powers are levels derived from the registry's tier and grant class, so each new tool gets
  one; Run is the grant classes that run things plus `thread_answer`.
- D3 — Unpaired programs keep the read tools, so docs, terminal reads and the like still answer
  any agent on the machine; they never write.
- D4 — A paired client's token stays per start in its endpoint file (AD-524); the trust lasts in
  `clients.json` with its power.
- D5 — Marley's own launches are the proven terminals and the own endpoint; no new variable in
  terminals, so no Zed hunk.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a program calls with the shared endpoint's bearer and no proven terminal, Marley shall answer its read tools and refuse its other tools with `not_paired`. | run checks |
| REQ-002 | WHEN `marley-mcp-bridge pair NAME` runs, Marley shall show NAME, its folder and the code the command prints, with Read, Read and Write, Read Write and Run, and Refuse, for two minutes. | `811-01-prompt`, run check |
| REQ-003 | WHEN the user picks a power, the command shall print that the client is trusted and the line to run the bridge with, and Outside Clients shall list it with that power. | run check, `811-02-clients` |
| REQ-004 | WHILE a client holds a power, Marley shall answer the tools that power covers and refuse the others with `not_granted`. | run checks, `811-03-run` |
| REQ-005 | WHEN the user changes a client's power in Outside Clients, its next call shall be judged by the new power. | `811-02-clients`, `811-03-run` |
| REQ-006 | WHEN the user cuts a client off, its next call shall be refused. | run check |
| REQ-007 | WHILE an agent runs in a Marley terminal or as Zed's agent, it shall keep every tool, under the approvals and kill switch as before. | run check, the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_mcp` (powers, principals, the own bearer, the pairing tools), the
  workbench (own endpoint, pairing, Outside Clients), the bridge's `pair`, `marley-open-url`;
  review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, guide, architecture notes, walkthrough (§21), ledger (§19), close,
  archive, commit.
