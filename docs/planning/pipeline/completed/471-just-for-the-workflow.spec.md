---
pipeline_id: 24e0a3ea-25af-4993-b888-6d8a2f7cda69
ticket: docs/planning/tickets/closed/TICKET-471-just-for-the-workflow.md
status: Phase 4 — Complete PASS
title: just for Marley's workflow
type: chore
slice: the workflow
references: [CONSTITUTION.md, script/gates.sh, .claude/hooks/enforce-tests-ran.sh]
---

## Title
Chad asked to install `just` and use it to replace or augment the workflow where it helps. It
augments: a `justfile` at the root names the commands every pipeline repeats, waits for the box's
other cargo runs before each cargo command, and wraps the live capture that each UI ticket
rebuilt by hand. `script/gates.sh` stays the gate and writes the receipt.

## Scope
### In
- `just` 1.58.0 from Arch's `extra` on the dev box.
- `justfile`: `idle` (wait for no cargo); `gate-diff`, `gate-fast`, `gate-full`; `build` (the
  debug `marley`); `test`, `clippy` and `fmt` over named crates; `shot` (the live capture).
- `script/live-shot.sh`: the capture of #467 to #470, generalized from the session scratchpad:
  a copy of the user's profile, an optional seed script, hidden workspace 9, `grim -T`, the copy
  removed afterwards.
- `enforce-tests-ran.sh` counts `just gate-*` and `just test` as runs; gate:11 lints the new
  script; CONSTITUTION's tools and the Test phase's skill name the recipes.

### Out (explicitly deferred)
- Replacing `script/gates.sh` or its receipt: the hooks and the commit gate read them.
- Recipes for the pipeline's paperwork.

## Reference (§20)
- N/A — Marley-specific: the fork's own workflow tooling; neither Warp nor Zed has one to follow.

### Prior art
- **Behavior maps:** none.
- **Published material:** the `just` manual: recipes, variadic parameters (`+crates`), the
  `prepend` function, `set shell`.
- **Code we already ship:** `script/gates.sh`; the scratch `rail-shot.sh` of #467 to #470
  (L-claude-467-capture-one-window-by-its-toplevel-001); Zed's own `script/` helpers.

## UI proof
N/A — no UI delta: workflow tooling. The capture recipe is itself verified by a capture.

## Locked-In Decisions
- D1 — `just` wraps and does not replace: every recipe runs the command a hook or the receipt
  already knows, so the pipeline's checks hold whichever way it is run.
- D2 — Every cargo recipe depends on `idle`, so the one-cargo rule is kept by the tool.
- D3 — The capture removes its profile copy when it ends.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `just --list` shall name each recipe with its one-line description | run it |
| REQ-002 | WHILE another cargo runs, a cargo recipe shall wait before it starts | a recipe started beside a running cargo |
| REQ-003 | WHEN the Test phase runs the gate or tests through `just`, `enforce-tests-ran.sh` shall accept it as it accepts a direct run | the hook fed a transcript with each form |
| REQ-004 | `just shot <name>` shall produce the capture and leave no profile copy behind | run it |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec. **P2 Code** — the install, the files, the hook. **P3 Test** — the
  checks above and the gate. **P4 Complete** — docs, close, archive, commit.
