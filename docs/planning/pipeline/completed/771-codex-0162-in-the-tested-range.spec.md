---
pipeline_id: 8480609c-c65b-4915-89de-385c6128f8f1
ticket: docs/planning/tickets/closed/TICKET-771-codex-0162-in-the-tested-range.md
status: Phase 4 — Complete PASS
title: Codex 0.162 in the tested range
type: chore
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), Codex's App Server (#650)
references:
  - docs/planning/pipeline/completed/648-agent-version-check.spec.md
  - docs/planning/pipeline/completed/650-codex-app-server-state.spec.md
  - docs/planning/pipeline/completed/651-codex-approvals-and-prompts.spec.md
---

## Title
Codex 0.162 in the tested range. Marley joins Codex's App Server (#650) only on the versions
whose generated protocol was checked (#648's table, 0.155.1 to 0.158.0); the box runs Codex
0.162.0, so #650, #651 and the tickets after them (#750, #752, #753) reach nothing there. The
protocol is checked at 0.162.0 and the range widened to it.

## Scope
### In
- The check: Codex 0.158.0 (installed under a scratch `MISE_DATA_DIR`) and 0.162.0 (the box's)
  generate the App Server's JSON Schema, stable and experimental, with the network cut off and a
  scratch `CODEX_HOME`; every definition reachable from the messages Marley reads and sends is
  compared, along with `initialize` and the command line Marley runs.
- `CODEX_APP_SERVER.tested` up to 0.162.0 (`before: 0.162.1`); the doc comments and the guide
  that name the range.

### Out (explicitly deferred)
- Turning `marley.codex_app_server` on: Chad's call, asked once this lands.
- Codex 0.159 to 0.161, between the two ends checked: inside the range, as 0.156 and 0.157 were.
- Fields Marley does not read yet (the new `rootTurnId`, `partial_answer` phase, `tooManyDenials`).

## Reference (§20)
N/A — Marley-specific: the range is #648's own record of what Marley checked. Codex's published
material is the generated schema (`codex app-server generate-json-schema`), which this check reads.

### Prior art
- **The code we ship:** `marley_agent::versions::CODEX_APP_SERVER` (`versions.rs` 98-115), whose
  doc says the range closes at the two ends checked, "whose generated schemas agree on every field
  Marley reads"; `codex_events.rs` (the shapes Marley parses, with `#[serde(other)]` on the open
  enums); `codex_server.rs` (the `initialize` it sends at 921-934 and the `app-server --listen`
  command at 379-386).
- **Earlier checks:** #650 and #651 generated the schema from both binaries and compared the
  messages Marley reads (`651-codex-approvals-and-prompts.notes.md` 84-140).
- **Published material:** Codex's `app-server generate-json-schema --out DIR [--experimental]`.

## UI proof
Scenario `script/e2e/771-codex-0162-app-server.sh` (`compositor sway`): `MARLEY_CODEX` names the
box's Codex 0.162.0 and `CODEX_HOME` a scratch folder; `marley.codex_app_server` on; Codex
started from a project terminal with `marley agent-cli --provider=codex` (#749).

| Shot | Shows | Criteria |
|---|---|---|
| `771-01-codex` | Codex's terminal with its agent bar and no "App Servers off" chip; the run's log holds no "so Codex starts without one" line and an `app-server --listen unix://` process runs for it | REQ-001 |

## Locked-In Decisions
- **D1 — The range's top end is 0.162.0** (`before: Some(Version::new(0, 162, 1))`), the version
  checked, as 0.158.0 was: a patch release of it counts, a minor one does not.
- **D2 — What the check found** (recorded in the notes): of 154 definitions reachable from
  `thread/status/changed`, `thread/started`, `thread/closed`, `thread/settings/updated`,
  `thread/tokenUsage/updated`, `turn/started`, `turn/completed`, `item/started`,
  `serverRequest/resolved`, the four server requests (#651) and the four client requests Marley
  sends, six changed between 0.158.0 and 0.162.0, all additively: `Turn` gains `rootTurnId`,
  `MisalignmentErrorDetails` gains `reviewTarget`, `SubAgentActivityThreadItem` gains `model` and
  `reasoningEffort`, `MessagePhase` gains `partial_answer`, `ResponseItem` gains
  `AdditionalToolsResponseItem`, `CodexErrorInfo` gains `tooManyDenials` (and becomes `anyOf`).
  None is a field Marley reads, and none removes or retypes one. `initialize`'s definitions and
  the `--listen` and `--remote` flags are the same.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Codex 0.162.0 starts from Marley with `marley.codex_app_server` on, the system shall start and join its App Server, with no chip saying the integration is off. | shot `codex`, the log and the process check |
| REQ-002 | WHERE Codex is outside 0.155.1 to 0.162.0, the system shall keep the App Server off and say why, as before. | review of the diff (the range's other end is untouched; #648's scenario covers the chip) |

## Phase Plan
- **P1 Plan** — this spec, the check in the notes.
- **P2 Code** — `versions.rs`, `codex_events.rs`, `codex_server.rs` doc lines; the guide's range;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check, every shot read.
- **P4 Complete** — CHANGELOG, ledger, close, archive, commit.
