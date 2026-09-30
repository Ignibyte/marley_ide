---
pipeline_id: 0ac0b16e-db0a-4331-8923-76d41415fdf9
ticket: docs/planning/tickets/open/TICKET-607-fleet-panel-with-pseudo-agents.md
status: Phase 4 — Complete PASS
title: "The fleet contract's types, a pseudo provider, and the Fleet panel's list"
type: feature
slice: prong 2, D20 (docs/marley/fleet-contract.md), wave 1 of the remaining work
references: [docs/marley/fleet-contract.md, docs/marley/three-prong-plan.md, docs/planning/design-notes/remaining-work-2026-09-30.md]
---

## Title
The contract's `v1` types in a new pure crate, a pseudo provider that serves the contract's
example data and moves it, and a Fleet panel in the right dock that lists the agents by host.

## Scope
### In
- **`marley_sdk`**, a new pure crate (serde only, no gpui), the SDK's seed:
  - the `marley.work/v1` types: `Handshake`, `AgentList`, `AgentSummary`, `AgentDetail`,
    `WorkItem`, `Run`, `PhaseRun`, `Gate`, `Event`, `Usage`, `Question`, `Changes`, their state
    enums;
  - the `marley.host/v1` types: `HostSnapshot`, `AgentProcess`;
  - the doc's examples as fixture JSON files, `include_str!`'d and parsed in the crate;
  - `is_stale(last_seen_ms, now_ms, poll_s, stale_after_s)`: three missed polls, or the
    provider's `stale_after_s`.
- **The pseudo provider:** a pure state machine in `marley_sdk` that, given the time, returns the
  three agents on two hosts from the fixtures, moved on. CPU and network wander, an event arrives
  every few seconds, and a run advances a phase about once a minute. One agent waits on a
  question, one has a failed gate, and one stops reporting after a while so it reads stale.
- **The setting:** `"marley": { "fleet": { "providers": [ { "kind": "pseudo" } ] } }`, empty by
  default. With no provider, the panel says the fleet is not set up and how to set it.
- **The Fleet panel** (`marley_workbench`, `fleet.rs`): a `workspace::Panel` in the right dock,
  with its own icon button and a toggle action (`marley: toggle fleet`). It lists the agents
  grouped by host (a host header with the host's name), each row with:
  - the runtime's icon and the agent's name;
  - a state chip: working, idle, waiting, error, done, stale;
  - the work item's key;
  - the phase as `name n/m`;
  - an attention mark for a question or a failure.

  It reads the provider every `poll_s` while it shows, and redraws only on change.

### Out (explicitly deferred)
- The selected agent's snapshot (#608), the Agent tab (#609), the SSH collector (#610), the MCP
  and HTTP clients (#611).
- Actions (answer, stop), and restoring the panel's selection after a restart.

## Reference (§20)
Upstream Zed: a dock panel is `workspace::Panel` (`crates/workspace/src/dock.rs:36-104`), as
`OutlinePanel` implements it (`crates/outline_panel/src/outline_panel.rs:5448-5552`): its icon,
toggle action, persistent name and size. Warp keeps an agent management view in its right panel
(`docs/warp_architecture/subsystems/07-app-entry-build-tooling.md:277-286`); Orca's agent dashboard
groups agents by state with a host badge for SSH
(`docs/orca_architecture/05-terminal-and-workspace.md:466-494`,
`docs/orca_architecture/01-agents-and-sessions.md:266-272`). Marley keeps a list by host in the
right dock, drawn in its rail's row style.

### Prior art
- **Behavior maps:** the Warp and Orca notes above; Orca (MIT) may be read and adopted with its
  notice kept.
- **Published material:** none needed; the contract is Marley's own
  (`docs/marley/fleet-contract.md`).
- **Code we already ship:**
  - `marley_fleet`'s six states match the contract's `state`, and its serde conventions are the
    ones to follow (`crates/marley_fleet/src/session.rs:12-86`). Its `Question` differs, and it
    has no run, phase, gate, event, usage or host types, hence the new crate.
  - `marley_fleet::attention::is_stale` (`attention.rs:41`) is the staleness precedent.
  - The Marley layout already docks the Agent Panel on the right (`marley_workbench.rs:907-920`),
    and #456's displacement logic (`LayoutState::displaced`, `marley_workbench.rs:514`) acts on
    whatever shares that dock.
  - Dock activation priorities must be unique per dock (`dock.rs:784-796`); 0 to 7 are taken.
  - Settings: `MarleySettingsContent` with nested `push` and `system_one` precedents
    (`crates/settings_content/src/marley.rs:10-184`), resolved in `MarleySettings`
    (`marley_workbench.rs:319-440`).

## UI proof
The scenario `script/e2e/607-fleet-panel-with-pseudo-agents.sh` (sway) sets the pseudo provider
in the run's settings and opens the Fleet panel with its toggle. It shoots the list (`list.png`:
three agents under two host headers, chips, keys, phases, the attention marks); again a minute on
(`moved.png`: the working agent's phase has moved, and the quiet agent reads stale); with the
provider removed (`not-set-up.png`); and with the provider back, after a round trip through Zed's
layout and a wait past two minutes (`layout.png`: the panel in the right dock, still reading, the
working agent in its last phase). The settings edits come before the round trip: a hand edit to
settings.json after Marley has written the file does not reload, which predates this ticket
(#612).

## Locked-In Decisions
- D1 — The contract's types live in a new pure crate, `marley_sdk`, which later carries the
  schemas and the conformance kit; `marley_fleet` stays the session envelope.
- D2 — The pseudo provider is data, not a mock: it is the fixtures moved by time, so the same
  JSON documents the contract, feeds the demo and later seeds the conformance kit.
- D3 — The Fleet panel is a right-dock panel of its own, beside the Agent Panel, with a
  priority no other panel uses.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the pseudo provider is set, the Fleet panel shall list its agents grouped under their hosts, each with its name, state chip, work item key, phase `n/m` and attention mark. | Shot `list.png` |
| REQ-002 | WHILE the panel shows, it shall follow the provider's changes, and an agent with no sign for three polls shall read stale. | Shot `moved.png` |
| REQ-003 | WHEN no provider is set, the panel shall say the fleet is not set up and name the setting. | Shot `not-set-up.png` |
| REQ-004 | WHEN the provider's JSON is parsed, fields the contract does not name shall be ignored and optional fields may be missing. | Review of the types (serde defaults, no `deny_unknown_fields`) |
| REQ-005 | WHERE the Marley layout moves the Agent Panel between docks, the Fleet panel shall keep its place in the right dock. | Shot `layout.png`, and review against #456's displacement logic |

## Phase Plan
- **P1 Plan** — promote, recall, the design in the notes (the panel's priority, the crate's
  lint table, the pseudo provider's clock).
- **P2 Code** — `marley_sdk`, the setting, `fleet.rs`; a review of the diff; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — CHANGELOG, a `marley_sdk` architecture note and `marley_workbench.md`, the
  guide page, guide.md and the walkthrough; the ledger; close, archive, commit.
