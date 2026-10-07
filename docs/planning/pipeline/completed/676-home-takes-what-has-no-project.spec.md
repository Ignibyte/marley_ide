---
pipeline_id: e97a7b0c-6cab-415a-9696-7c46cbce58ec
ticket: docs/planning/tickets/open/TICKET-676-home-takes-what-has-no-project.md
status: Phase 4 — Complete PASS
title: "Home takes the screens that belong to no project"
type: feature
slice: the rail (decision 4 of docs/planning/intake/rail-and-center-tabs.md)
references: [docs/planning/pipeline/completed/675-the-rusty-group.spec.md, docs/planning/pipeline/completed/600-rail-menu-and-projectless-groups.spec.md]
---

## Title
The Agent tab, System One calls and a harness session's tab open in the window's Home group,
whatever project the window shows. Chad, 2026-10-07: "I think maybe anything that doesnt fit a
category lands there?", then "ok" to the rule: projects take what is theirs, Rusty takes Rusty's,
Home takes the rest.

## Scope
### In
- **Routing**: the Agent tab (from the Fleet panel), System One calls (its action, from the palette
  and the Settings page's link) and a harness session's tab (from the rail's inbox and Harness rows)
  open in Home, which the window then shows; the first one makes Home.
- **One mechanism**: #675's Rusty-group routing becomes `groups::in_group(kind, …)` for Home and
  Rusty alike, with its guard against two groups made by two quick opens; the rail's empty-space
  menu makes Home through it too.
- **The Zed layout**: these open where they did.

### Out (explicitly deferred)
- Any other screen: none other belongs to no project today.

## Reference (§20)
N/A — Marley-specific: Home is Marley's folderless group (#600), and the rule is Chad's. Orca's
floating workspace is the nearest outside shape (`docs/orca_architecture/05-terminal-and-workspace.md`).

### Prior art
- **Behaviour maps:** Orca's floating workspace, recorded in the intake note.
- **Published material:** none applies.
- **The code we ship:** #675's `in_rusty_group` and `with_rusty_group` (generalised here), the
  rail's `in_home` (#600), `MultiWorkspace::activate`.

## UI proof
`script/e2e/676-home-takes-what-has-no-project.sh` (`compositor sway`), a scratch project.
Shots: `676-01-calls` (`marley: open system one calls` from the project: a Home group, System One
calls its row and in front); `676-02-project` (the project's terminal clicked: in front, the calls
still under Home); `676-03-again` (the action again from the project: Home shown with the same tab,
one Home group).

## Locked-In Decisions
- D1 — **Home is made on first use**, as the empty-space menu makes it.
- D2 — **The Agent tab and the harness tab ride the same path**: shown by the review of the diff
  and the tab's opener, since a scenario would need a fleet and a harness of its own; System One
  calls stands for the three in the check.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN System One calls, the Agent tab or a harness session's tab opens in the Marley layout, the system shall open it in the window's Home group, making the group when there is none, and show the group. | `676-01-calls`; the review for the other two |
| REQ-002 | WHEN the window shows a project again, the Home group shall keep the tab. | `676-02-project` |
| REQ-003 | WHEN one opens again, the system shall bring the tab in Home forward and make no second Home group. | `676-03-again` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `groups.rs`, `rusty.rs`, `agent_tab.rs`, `system_one.rs`, `rail.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
