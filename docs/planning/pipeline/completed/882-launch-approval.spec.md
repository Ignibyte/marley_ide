---
pipeline_id: a77a3982-e4d1-460b-b718-c6164f714865
ticket: docs/planning/tickets/open/TICKET-882-launch-remote-and-block-tools.md
status: Phase 4 — Complete PASS
title: A launch config's approval as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Running a launch config not approved yet asks with a card, as the Workflows board draws it: "Dev
runs these, once you say so", a row per item (a glyph, what it runs, where it opens), Cancel and a
primary Run. Every command is shown whole, wrapped, never cut.

## Scope
### In
- `launch.rs`: `LaunchApproval`, a modal that answers through a oneshot, in place of
  `window.prompt`; `rows` from the config's items. The approval's hash and the store are as
  they are.

### Out (explicitly deferred)
- The Launch section of the project's + menu, remote terminals and folders, the block filter's
  look and the block keys: TICKET-889.

## Reference (§20)
N/A — Marley-specific: Marley's launch configs (#527, #592); the Workflows board.

### Prior art
- `launch::run`, `text`, `verbatim` (#592: a command shown as written); `workflows.rs`'s
  `ParameterPrompt` (a modal answered through a oneshot); #870's `prompts::render_button`.
- The brain (consultation 2bf892a4).

## UI proof
Scenario `script/e2e/882-launch-approval.sh` (sway): #527's scratch repository and
`.zed/marley.json` naming `Dev` (a server terminal whose command ends in a Markdown comment, an
agent split right, a browser split down).
- `882-01-approve`: Dev chosen: the card with three rows, the server's command whole as written,
  Cancel and Run (REQ-001).
- `882-02-day`: Marley Day (REQ-001). Escape then opens nothing.

## Locked-In Decisions
- D1 — Enter runs and Escape cancels, as Zed's prompt did with Run first.
- D2 — A command is drawn in the buffer font with no Markdown and wraps; nothing truncates it.
- D3 — A changed config's card says so in its title, as before.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a launch config not approved yet runs, Marley shall ask with a card listing each item's kind, what it runs whole, and where it opens, with Cancel and a primary Run, in Marley Night and Marley Day. | `882-01-approve`, `882-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
