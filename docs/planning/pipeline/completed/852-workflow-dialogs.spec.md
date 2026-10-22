---
pipeline_id: 73a5d289-b951-401e-9566-8413f1024f57
ticket: docs/planning/tickets/open/TICKET-852-workflows-and-launch.md
status: Phase 4 — Complete PASS
title: Save as Workflow and Run as the Workflows board draws them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Save as Workflow shows the command with its parameters as chips, says how many parts became
parameters, and ends with where it saves, Cancel and a primary "Save · Enter". Running a workflow
shows the command as it will run, filled in as the values are typed, and ends with the keys and a
primary Run.

## Scope
### In
- `workflows.rs`: `WorkflowEditor` (the chips line, the count, the footer) and
  `ParameterPrompt` (the preview, the footer); the parameter rows and the global box stay.

### Out (explicitly deferred)
- The Launch menu and its approval, remote terminals and folders, the block filter and the block
  keys: TICKET-882.
- Making a word a parameter by clicking it: the command field takes `{{name}}` typed.

## Reference (§20)
N/A — Marley-specific: Marley's workflows (#558); the Workflows board.

### Prior art
- `WorkflowEditor`, `ParameterPrompt`, `marley_terminal::workflow::{params_of, substitute}`;
  #870's `prompts::render_button` for the buttons.
- The brain (consultation d9c1af10).

## UI proof
Scenario `script/e2e/852-workflow-dialogs.sh` (sway): a scratch repository with `web/`, a fake
`serve` on the terminal's PATH.
- `852-01-save`: `serve --port 4180 web` run, its block saved: the name, the command, the
  chips line with `port` and `path`, "2 parts became parameters", the rows, the footer (REQ-001).
- `852-02-run`: the saved workflow run from Zed's task picker: its fields and the command filled
  in with a value typed (REQ-002).
- `852-03-day`: Marley Day, the run prompt (REQ-002).
- `852-04-save-day`: Marley Day, the save card (REQ-001).

## Locked-In Decisions
- D1 — The chips line reads the command field as typed: each `{{name}}` an accent chip.
- D2 — The preview substitutes the fields' values as they are; a field left empty shows its
  `{{name}}`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Save as Workflow shall show the command with each parameter as a chip, how many parts became parameters, and a footer with where it saves, Cancel and a primary Save, in Marley Night and Marley Day. | `852-01-save`, `852-04-save-day` |
| REQ-002 | WHEN a workflow with parameters runs, its prompt shall show the command filled in with the values as typed and a footer with the keys and a primary Run. | `852-02-run`, `852-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
