---
pipeline_id: 5ad97523-e09d-4ed8-b025-6ef8495417c1
ticket: docs/planning/tickets/open/TICKET-776-the-shared-plugin-with-plan-approval.md
status: Phase 4 — Complete PASS
title: The shared plugin with plan approval and the plan-questions skill
type: chore
slice: agents together, #709's carry (as #724 was), for #726 and #728
references:
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/planning/pipeline/completed/724-the-shared-plugin-with-rustal-ste.spec.md
  - docs/planning/pipeline/completed/726-plan-review-in-a-tab.spec.md
  - docs/planning/pipeline/completed/728-question-cards-in-plans.spec.md
---

## Title
Marley carries rustal-harness's shared Claude Code plugin at its TICKET-119 (`5333d15`, 0.4.0,
digest `b3b5ee99…`). Under Marley's host its mod hands a call whose tool `MARLEY_APPROVE_TOOLS` names
(`ExitPlanMode` while `marley.plan_review` is on) to `"$MARLEY_BIN" approve`, so a real Claude Code's
plan opens in #726's Plan tab; and its skill `plan-questions` teaches the agent #728's `:::question`
blocks. Chad, 2026-10-12: "send anything to harness also to work on" (MREQ-012, MREQ-013).

## Scope
### In
- The plugin's files from rustal-harness `5333d15`, unchanged: `plugin.json` (0.4.0), `register.js`,
  and `skills/plan-questions/SKILL.md`; `shared_plugin::FILES` at 17 in the harness's order.
- The guides and the module docs that said the plan review waits on a plugin revision.

### Out (explicitly deferred)
- Marley's `approve` and the Plan tab: #726, #727, #728 shipped them.
- A Marley-own copy of `plan-questions` for terminals without the shared plugin.

## Reference (§20)
N/A — Marley-specific: Marley carries files another Ignibyte project ships, as #709 and #724 did.

### Prior art
- **The code we ship:** `shared_plugin.rs` (#709, #724): `FILES`, `digest`, `install_in`; the
  installer makes each file's folder, so a new skill folder needs no list. `identity.rs` already
  sets `MARLEY_APPROVE_TOOLS` (#726).
- **The harness's own test** (`scripts/test_plan_approval.py`, TICKET-119): the pinned Claude Code
  2.1.287 against its scripted Messages API (`scripts/fixtures/claude_api.py`) in a user and network
  namespace, with rules that write the plan file the plan-mode reminder names and call
  `ExitPlanMode`. The scenario takes the fixture and the rules' shape; the harness's `MARLEY_BIN` is
  a stand-in, Marley's is the real one.

## UI proof
`script/e2e/776-the-shared-plugin-with-plan-approval.sh`, under `compositor sway`. In the project's
terminal, the pinned Claude Code 2.1.287 runs in plan mode inside the scripted API's namespace, with
Marley's variables and the shared plugin from `CLAUDE_CODE_PLUGIN_DIRS`.

Shots:
- `776-01-plan`: the model's plan, from its plan file, in the Plan tab with its question as a card.
- `776-02-answers`: after Accept recommended and Send Answers, the terminal: the call refused with
  "Answers to your questions", Claude Code still in plan mode.
- `776-03a-revised`: the revised plan, version 2, its question settled.
- `776-03-approved`: the revised plan approved: Claude Code has left plan mode.

## Locked-In Decisions
- **D1:** the plugin's files are the harness's, unchanged, at `5333d15` (#709's rule).
- **D2:** the visual check runs the pinned Claude Code against the harness's scripted Messages API,
  run from the harness's checkout, in a user and network namespace: no model, no key. The fixture's
  placeholder key is built at run time.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shared plugin is on, a new terminal's Claude Code shall load rustal-harness 0.4.0 from the folder named by digest `b3b5ee99…`, with the skill `plan-questions` listed to the model. | The folder in the run's data directory; the scripted API's record of the listing |
| REQ-002 | WHEN Claude Code in plan mode calls `ExitPlanMode`, the system shall open its plan in the Plan tab, its open question as a card. | Shot 776-01-plan |
| REQ-003 | WHEN the user answers the card and sends the answers, Claude Code shall receive them as the call's refusal and stay in plan mode. | Shot 776-02-answers; the terminal's text |
| REQ-004 | WHEN the user approves the revised plan, Claude Code shall leave plan mode. | Shot 776-03-approved; the scripted API's record of "User has approved your plan" |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the files, `FILES`, the docs; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG and architecture docs, ledger capture, close the ticket, archive,
  commit.
