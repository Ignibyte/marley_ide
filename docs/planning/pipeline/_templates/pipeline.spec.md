---
pipeline_id: <uuid — python3 -c 'import uuid;print(uuid.uuid4())'>
ticket: <docs/planning/tickets/open/TICKET-<n>-<slug>.md>
status: IN PROGRESS
title: <one-line title>
type: <feature|bug|chore|spike|docs>
slice: <prong + slice from docs/marley/three-prong-plan.md, e.g. T1>
references: []
---

<!-- This is the pipeline SPEC template. The placeholder pipeline_id + the
     `IN PROGRESS` status are intentionally rejected by the enforcement hooks
     (enforce-phase-gate wants a real 36-char UUID; enforce-pipeline-completion
     rejects `IN PROGRESS`/`<…>`) — fill them in /pipeline:plan. Keep the spec
     lean; the design + per-phase detail live in the .notes.md. -->

## Title
<what this change is + why>

## Scope
### In
- …

### Out (explicitly deferred)
- …

## Reference (§20)
<!-- REQUIRED (enforce-warp-reference.sh binds it). Name the reference behavior + how Marley matches it.
     The Warp wall (§20): observe behavior, reimplement, NEVER read or translate Warp's AGPL source — a
     reworded translation is still a derivative work. Zed is this codebase: its crates are read and
     extended directly. Fill with ONE of:
       - Warp (terminal / blocks / cockpit / UX) — cite docs/warp_architecture/… OR an observed capture
         saved to docs/warp_architecture/observed/ (a screenshot/notes; NOT ephemeral scratchpad).
       - Upstream Zed (editor / workspace / panel behavior) — name the crate and the behavior kept.
       - N/A — Marley-specific, no reference analog + WHY. -->

### Prior art
<!-- REQUIRED (enforce-warp-reference.sh binds it). The sweep Plan owes BEFORE locking any decision. The
     wall says what you may not READ (Warp's source); it does not excuse reinventing what is already ours
     to take. Three legs:
       1. The behavior maps — docs/warp_architecture/, docs/zed_architecture/, observed captures. Research, not source.
       2. Published material — docs, blogs, talks, the protocol specs (LSP, MCP, ACP, CDP).
       3. **THE CODE WE ALREADY SHIP — the highest-yield leg.** Zed's own crates (editor, terminal,
          terminal_view, task, context_server, agent_servers, gpui …) and every dependency in Cargo.lock
          (alacritty, vte, tree-sitter, regex …). Reading them is ADOPTION.
          Ask plainly: DOES A CRATE WE ALREADY BUILD OWN THIS SEAM?
     It has paid repeatedly: gpui settled #336's forks; the `regex` crate dissolved #339's locked decision;
     Zed's `terminal` crate turned the block terminal into a one-file fork hook instead of a second engine.
     A locked decision dying because the substrate already does it is a WIN to record, not a deviation to hide.
     Found nothing? That is a PASS — write "none: checked gpui/terminal/regex, no owner". Silence is not. -->

## UI proof
<!-- Does this ticket change anything the user SEES or types into (a pane, a panel, the terminal, a
     keystroke or mouse handler, an affordance)? Fill with ONE of:
       - UI-AFFECTING: the gpui driven tests (TestAppContext / VisualTestContext) that will prove it, and
         the live drive the Test phase runs on the real app (cargo run + a screenshot through dev-box-desktop).
       - N/A — no UI delta: <why> (first line must start "N/A"). -->

## Locked-In Decisions
- D1 — …

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN … the system shall … | … |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — …; fmt and clippy clean; a review of the diff.
- **P3 Test** — write + RUN the tests; the live drive for a UI change; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
