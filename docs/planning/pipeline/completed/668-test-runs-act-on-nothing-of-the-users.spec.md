---
pipeline_id: bdffdaa2-e424-4d5a-a6c2-1343636276f2
ticket: docs/planning/tickets/closed/TICKET-668-test-runs-act-on-nothing-of-the-users.md
status: Phase 4 — Complete PASS
title: "Test runs act on nothing of the user's"
type: chore
slice: the e2e harness (CONSTITUTION §7)
references: [docs/t3code_architecture/07-engineering-and-releases.md, docs/planning/pipeline/completed/663-rusty-quick-capture-and-import.spec.md]
---

## Title
Test runs act on nothing of the user's. `script/e2e.sh` copies the user's `settings.json` into each
run's profile and turns off Rusty, Voice, agent prompts in a tab, Codex's App Server and Claude
Code's IDE link, but keeps `marley.push` (the phone push topic and token file), `marley.harness`
and `marley.embedded_harness`, `marley.fleet` (providers and hosts) and `marley.system_one`, and
the run inherits `MARLEY_SYSTEM_ONE_KEY` and `MARLEY_CLOUDFLARE_API_TOKEN`. A scenario could push
to the user's phone, reach real hosts or spend System One's budget. This ticket leaves all of them
out of every run.

## Scope
### In
- **The copy** drops `marley.push`, `marley.harness`, `marley.embedded_harness`, `marley.fleet`
  and `marley.system_one` from the user's settings; the rest stays as it was copied.
- **The run** unsets `MARLEY_SYSTEM_ONE_KEY` and `MARLEY_CLOUDFLARE_API_TOKEN` before the
  scenario's `setup`, so only a scenario that sets its own fake has one.
- **The harness's words** above the copy say what it leaves out and why.
- **The proof**: a scenario whose own scratch config holds every one of those settings, run with
  both variables set, checks the run's profile and environment hold none of them, and the
  scenarios that set their own (535, 548, 565, 640, 641, 651) still pass.

### Out (explicitly deferred)
- **The user's database** (`db/`), copied whole as before; its rows hold no setting of these.
- **Other variables** a program inside a run might read (the user's own shell's); the scenarios
  give terminals a HOME of their own already.

## Reference (§20)
N/A — Marley-specific: the harness is Marley's own (`script/e2e.sh`). T3 Code's `migrate-dev-db`
keeps only what a run needs (`docs/t3code_architecture/07-engineering-and-releases.md`), the
survey's note that raised this.

### Prior art
- **Behaviour maps:** `docs/t3code_architecture/07-engineering-and-releases.md` (a dev database
  rebuilt with only what the run needs).
- **Published material:** none applies.
- **The code we ship:** `script/e2e.sh`'s copy (`:627-665`), which already turns off Rusty, Voice,
  agent prompts in a tab, Codex's App Server and Claude Code's IDE link the same way.

## UI proof
`script/e2e/668-test-runs-act-on-nothing-of-the-users.sh` (no clicks). It points the harness at a
scratch config of its own holding `marley.push`, `marley.harness`, `marley.embedded_harness`,
`marley.fleet`, `marley.system_one` and a harmless key, and exports both key variables before the
copy. Its checks: the run's settings hold none of the five and still the harmless key; neither
variable is set in the run. One shot, `668-01-started`: Marley started on that profile. Then the
six scenarios that set these themselves run and pass.

## Locked-In Decisions
- D1 — **Strip in the same Python block** that turns the other features off, so one place says what
  a run inherits.
- D2 — **Unset the two variables beside `MARLEY_RUSTY_MCP` and `MARLEY_CODEX`**, before `setup`.
- D3 — **Drop the settings whole** rather than turning parts off: none of them has a harmless part a
  run needs.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a run copies the user's settings, the copy shall hold no `marley.push`, `marley.harness`, `marley.embedded_harness`, `marley.fleet` or `marley.system_one`. | The scenario's checks |
| REQ-002 | WHEN a run starts, its environment shall hold no `MARLEY_SYSTEM_ONE_KEY` or `MARLEY_CLOUDFLARE_API_TOKEN` unless its scenario sets one. | The scenario's checks |
| REQ-003 | WHILE a scenario sets its own push, harness, fleet or System One settings or keys, it shall run as before. | Scenarios 535, 548, 565, 640, 641, 651 |
| REQ-004 | WHEN a run copies the user's settings, every other setting shall stay. | The scenario's check on the harmless key |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `script/e2e.sh`; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its checks; the six scenarios that set their own.
- **P4 Complete** — CHANGELOG, the harness's notes, ledger capture, the brain decision, close,
  archive, commit.
