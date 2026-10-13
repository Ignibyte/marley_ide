---
pipeline_id: 8674f8da-6e4a-40c0-bdb1-26c4607a9b39
ticket: docs/planning/tickets/open/TICKET-781-zed-agent-blank-in-the-agent-panel.md
status: Phase 4 — Complete PASS
title: A Zed Agent thread with no model says so, and Configure opens the providers' settings
type: bug
slice: agents in the Marley layout (#734's threads in tabs, #744's one New Agent…)
references: [734, 744, 772]
---

## Title
Chad, 2026-10-12: "if you click agent panel, choose zed agent it shows blank, then choose select
model it brings up the old right agent panel." Reproduced on a copy of his profile: New Agent →
Zed Agent opens an empty thread in a tab with "Select a Model" at its bottom right; its picker says
"No matches" and offers Configure, which focuses Zed's Agent Panel in the right dock (a loading
Claude Agent thread) before it asks for the Settings window. Zed's panel shows a new user a "configure
a provider" card; a thread in a tab has no panel around it, so it shows nothing.

## Scope
### In
- In the Marley layout, `agent::OpenSettings` (the model picker's Configure, the panel
  onboarding's Configure Providers) opens the Settings window's LLM Providers page and leaves the
  right dock alone.
- A Zed Agent thread in a tab, while no default model is set, shows a callout above the
  conversation saying so, with Configure Providers; it goes once a model is set.

### Out (explicitly deferred)
- Zed's layout: the action keeps opening the Agent Panel there, as upstream.
- Zed's own onboarding card and trial upsells in a tab.

## Reference (§20)
Upstream Zed, `agent_ui`: `agent_panel::init`'s `OpenSettings` handler (focus the panel, then
`open_configuration`, which only opens the Settings window's AI page), the panel's new-user
onboarding (`render_new_user_onboarding`, `ApiKeysWithoutProviders`), and the thread view's "No
model selected" callout, whose Configure Provider button dispatches `OpenSettingsAt { path:
"llm_providers" }`. Marley keeps the destination and drops the panel.

### Prior art
- Behavior maps: `docs/zed_architecture/` (the Agent Panel and its onboarding).
- Published material: none needed.
- The code we ship: `zed_actions::agent::OpenSettings`, `zed_actions::OpenSettingsAt`, the
  Settings window's LLM Providers sub page (`settings_ui`, `json_path: "llm_providers"`),
  `ui::Callout`, `LanguageModelRegistry::default_model`; #744's `picker_in_place_of` (a capture
  on the workspace root in the Marley layout) is the pattern taken.

## UI proof
`script/e2e/781-zed-agent-blank-in-the-agent-panel.sh` (`compositor sway`), on a copy of the
user's profile with Zed's agent given no default model (`agent.default_model` null):
- `781-01-callout`: New Agent → Zed Agent: the thread's tab shows the callout and Configure
  Providers (REQ-002).
- `781-02-configure`: Select a Model → Configure: the Settings window on LLM Providers; the right
  dock stays closed (REQ-001).
- `781-03-callout-button`: the callout's Configure Providers: the same page (REQ-002).

## Locked-In Decisions
- D1 — A capture on the workspace root, as #744's, catches `agent::OpenSettings` in the Marley
  layout and dispatches `OpenSettingsAt { path: "llm_providers" }`, the page Zed's own "No model
  selected" callout opens. No Zed hunk.
- D2 — The callout belongs to the tab (`ThreadTab::render`), not to Zed's thread view: it is the
  panel's onboarding that a tab lacks, so it lives where the panel would be. It shows for the Zed
  Agent while `LanguageModelRegistry::default_model()` is none and follows the registry.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, WHEN `agent::OpenSettings` runs, the system shall open the Settings window's LLM Providers page and leave the right dock closed. | shot `781-02-configure` |
| REQ-002 | WHILE no default model is set, the system shall show a Zed Agent thread's tab a callout naming that, whose Configure Providers opens the LLM Providers page. | shots `781-01-callout`, `781-03-callout-button` |
| REQ-003 | WHEN a default model is set, the system shall take the callout away. | review of the diff (the registry observation) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `agents.rs`, `thread_tab.rs`; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
