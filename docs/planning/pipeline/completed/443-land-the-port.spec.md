---
pipeline_id: 0414e732-52ac-4b3e-a669-c3bd79682eab
ticket: docs/planning/tickets/closed/TICKET-443-land-the-port.md
status: Phase 5 — Complete PASS
title: Land the ported Marley tree behind a green gate
type: chore
slice: workbench shell baseline (before W0)
references: [docs/planning/design-notes/workbench-shell-shelf.md, docs/marley/workbench-shell.md]
---

## Title
Land the uncommitted 2026-09-18 port as the first commit on `marley/workbench-shell`, behind a
real green gate, by fixing the three things the first FULL run found between the tree and green.

## Scope
### In
- **The archive:** each gpui-era spec in `docs/planning/pipeline/completed/` that lacks the §20
  `## Reference (§20)` section or its `### Prior art` subsection gets a short note saying it
  predates the rule (done on 2026-09-22 before the first run: 218 sections and 71 subsections).
- **The transport:** `crates/marley_mcp/src/transport.rs` gets in-file tests that drive a real
  server over a loopback socket, one behavior per surviving mutant, and loses the equivalent
  `content_length > 0` guard around a `read_exact` that reads zero bytes anyway.
- **The gate:** FULL mode in `script/gates.sh` runs its mutation copies with isolated target
  directories, so two workers can no longer overwrite each other's test binary.
- **The ledger:** the race and the stripped masks go into `failures.md` and
  `prevention-rules.md`.
- **The commit:** the port, the workbench-shell plan and sprint docs, and this ticket's changes,
  as one commit on `marley/workbench-shell`, gated by `script/gates.sh --diff`.

### Out (explicitly deferred)
- The touchpoint ledger's gate:16 and hook (#436).
- Removing `transport.rs` from the coverage exclude list: its IO-error arms still cannot be
  provoked from a loopback peer.
- Pre-existing `let _ =` on the transport's thread spawns (Zed's `.rules` would flag them); a
  later cleanup, not part of landing the tree.

## Reference (§20)
N/A — Marley-specific: this lands Marley's own ported tree and fixes Marley's own gate. No Warp
or Zed behavior is reimplemented; the transport keeps the MCP Streamable-HTTP behavior it
already had and only gains tests.

### Prior art
- **Behavior maps:** none apply.
- **Published material:** the MCP Streamable-HTTP transport and session-management sections of
  the 2025-06-18 spec (what the transport tests assert: the `Mcp-Session-Id` echo on
  `initialize`, 400 without a session, 404 for an unknown one, 202 for a notification-only POST,
  DELETE to end a session); cargo-mutants' docs on copy mode versus `--in-place` and on
  `--jobs`; cargo's artifact naming, whose metadata hash for a workspace path package does not
  include the checkout's location (observed: one `marley_mcp-<hash>` artifact serves both copies).
- **Code we already ship:** the gate's DIFF mode already mutates in place with one job for
  exactly the shared-target reason (`script/gates.sh` mutation_g); the gpui era masked
  `transport.rs` with `mutants::skip` (history in `docs/marley_architecture/orchestration-shell.md`
  §10 and the #370/#375 pipeline notes); `std::net` is enough for a raw HTTP/1.1 test client, so
  no new dependency. No Zed crate owns any of this.

## UI proof
N/A — no UI delta: gate tooling, tests of a loopback server, and archived docs.

## Locked-In Decisions
- D1 — The transport's mutants die to tests, never to a restored mask; the fork's mutation gate
  keeps no exclusions (CONSTITUTION §0).
- D2 — FULL keeps two workers but gives each its own target directory; DIFF stays in place with
  one job.
- D3 — The baseline's receipt comes from `script/gates.sh --diff` (the untracked port enters as
  whole-file diffs, one job, in place), the same mode `/commit` uses.
- D4 — The archive notes say the sections predate the rule; nothing is backfilled as if it had
  been written at the time.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Every spec in `docs/planning/pipeline/completed/` shall carry a non-empty `## Reference (§20)` with a non-empty `### Prior art` | the hook's own checks run over the directory |
| REQ-002 | WHEN a client sends a request without the bearer, with a wrong bearer, or from a non-loopback Origin, the transport shall answer `403 Forbidden` | loopback test |
| REQ-003 | WHEN a client sends `initialize` with the bearer, the transport shall answer 200 over SSE and echo a fresh 32-hex `Mcp-Session-Id` | loopback test |
| REQ-004 | WHEN a request lacks a session or names an unknown one, the transport shall answer `400 Bad Request` or `404 Not Found` | loopback test |
| REQ-005 | WHEN a notification-only POST arrives on a live session, the transport shall answer `202 Accepted` | loopback test |
| REQ-006 | WHEN a DELETE names a live session, the transport shall end it and answer 200 | loopback test |
| REQ-007 | WHEN the session cap is full, a further `initialize` shall get `503 Service Unavailable` | loopback test |
| REQ-008 | WHEN a request's Content-Length exceeds 1 MiB, the transport shall answer 400 without reading the body, and a body of exactly 1 MiB shall be read | loopback tests |
| REQ-009 | WHILE a GET stream is open, the transport shall push exactly one `notifications/resources/updated` per `signal_change` and none without one, and shall free the stream's session when the client hangs up | loopback tests |
| REQ-010 | The handle's Debug output shall redact the bearer, and its URL shall be a loopback `/mcp` URL | unit test |
| REQ-011 | WHILE FULL mode mutates, each worker shall build in its own target directory | review + a FULL mutation log whose test build shows only its own mutant |
| REQ-012 | `script/gates.sh --diff` shall end `GATE GREEN [diff]` over the tree, and the baseline commit shall pass the commit, changelog and reference hooks | the gate output; the commit |

## Phase Plan
- **P2 Design** — the test list against the surviving mutants, the gate change, the commit's
  staging.
- **P3 Implement** — the tests (drafted during the red run's investigation; recorded as a
  deviation), the guard removal, the gate change.
- **P3.5 Inspect** — critics on the tests' flakiness and completeness and on the gate change;
  fix the real findings.
- **P4 Validate** — `cargo nextest run -p marley_mcp`, a transport-scoped mutation check, then
  `script/gates.sh --diff`.
- **P5 Complete** — CHANGELOG, ledger, close the ticket, archive; then `/commit`.
