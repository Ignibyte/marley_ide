# A Zed Agent thread with no model — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-781-zed-agent-blank-in-the-agent-panel.md
- **Pipeline spec:** 781-zed-agent-blank-in-the-agent-panel.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-12: "if you click agent panel, choose zed agent it shows blank, then
  choose select model it brings up the old right agent panel. add this somewhere as a bug". Run
  autonomously with the batch.
- **Classification / tier:** bug, Marley crate only (`marley_workbench`).
- **Checklist:** pick ✓ (BACKLOG row removed), pre-flight ✓, recall ✓, mint ✓, prior art ✓,
  spec ✓, design ✓.
- **Reproduction** (a scratch scenario on a copy of Chad's profile, under sway): New Agent, "zed",
  Enter, Enter: "New Agent Thread", "Message the Zed Agent…", nothing else, "Select a Model" at the
  bottom right; its picker: "Select a model…", "No matches", Configure; Configure: the right dock
  opens on Zed's Agent Panel, "New Claude Agent Thread", "Loading…".
- **Recall (§18.3):** #744's `picker_in_place_of` diverts Zed's ways into the Agent Panel in the
  Marley layout; the ledgers hold nothing on `agent::OpenSettings`; the brain (consultation
  34e3b59f…): nothing on this seam.
- **Discovery** (Explore): Configure dispatches `zed_actions::agent::OpenSettings`
  (`model_selector.rs` footer, `model_selector_components.rs`); `agent_panel::init`'s handler runs
  `focus_panel::<AgentPanel>` and then `open_configuration`, which only dispatches
  `OpenSettingsPage { page: "AI" }`; the panel, made active, starts a thread of its selected agent.
  The "configure a provider" card is the panel's (`should_render_new_user_onboarding`); a tab shows
  only the thread view, whose own "No model selected" callout appears only after a send fails.

### Design
- `agents.rs` (Marley): in `init`'s action renderer, `.capture_action(cx.listener(
  providers_in_place_of_panel))`; the handler, in the Marley layout, stops propagation and
  dispatches `zed_actions::OpenSettingsAt { path: "llm_providers", target: None }` on the window.
- `thread_tab.rs` (Marley): `ThreadTab::new` observes `LanguageModelRegistry::global(cx)`;
  `render` puts a `Callout` above the conversation while the thread is the Zed Agent's and
  `default_model()` is none: "The Zed Agent has no model yet", a line on what to do, and a
  Configure Providers button dispatching the same `OpenSettingsAt`.

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| 002 | `agent.default_model` null in the run's settings; New Agent → Zed Agent | `781-01-callout` |
| 001 | Select a Model → Configure | `781-02-configure` |
| 002 | The callout's Configure Providers | `781-03-callout-button` |
REQ-003 (the callout goes once a model is set) needs a signed-in provider; read in the review.

### Risks
- The capture catches the action from anywhere in the window in the Marley layout, Zed's panel
  included if a user shows it; the page it opens is the one the panel's own handler opens next.

## Phase 2 — Code
- **Built:** `agents.rs`: `providers_in_place_of_panel`, a capture of
  `zed_actions::agent::OpenSettings` on the workspace root beside #744's, which in the Marley
  layout stops it and dispatches `llm_providers_page()` (`OpenSettingsAt { path: "llm_providers"
  }`). `thread_tab.rs`: `ThreadTab::new` observes `LanguageModelRegistry::global`;
  `no_model_callout` (the Zed Agent's thread, `default_model()` none) draws a `Callout` (Info,
  "The Zed Agent has no model yet", Configure Providers → the same page) above the conversation.
- **Deviations:** none.
- **Review:** the capture runs before Zed's handler on the same root, as #744's do, and only in
  the Marley layout; the dispatched action is queued, not run inside the capture. The callout reads
  the conversation and the registry in render only. No Zed hunk.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 of 17 (`gate-781.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/781-zed-agent-blank-in-the-agent-panel.sh` under `compositor sway`, on
  a copy of the user's profile (Zed's agent there has no model it can reach), the copy's thread
  list emptied; the reproduction's clicks.
- **The runs found:** the first run's guess for the callout's button missed (it sits at the
  callout's right end); the model picker stays open behind the Settings window, so the scenario
  closes it before the last click, which makes the last shot's cause unambiguous.
- **The final run** (`e2e exit 0`):
  - **781-01-callout** (REQ-002): New Agent → Zed Agent: above "Message the Zed Agent…", the
    callout "The Zed Agent has no model yet", "Configure a language model provider, then pick a
    model under Select a Model.", Configure Providers at its right; Select a Model at the bottom.
  - **781-01a-picker / 781-02-configure** (REQ-001): the picker ("No matches", Configure); after
    Configure, the Settings window on "User / AI / General / LLM Providers" (Zed, Amazon Bedrock,
    Anthropic, …), searched `#llm_providers`; the main window has no Agent Panel at its right,
    where the reproduction showed "New Claude Agent Thread · Loading…".
  - **781-02a-closed / 781-03-callout-button** (REQ-002): with the Settings window and the picker
    closed, the callout's Configure Providers opens the same LLM Providers page.
- **Not reached:** REQ-003 (the callout leaving once a default model is set) needs a provider
  signed in; the registry observation that redraws the tab is read in the review.
- **Gate after the scenario:** `script/gates.sh --diff` again on the final tree.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #781); `docs/marley/guide.md` (Zed's Agent Panel with Marley's
  tools); `docs/marley_architecture/marley_workbench.md` (the captures). No Zed path.
- Ledger appends: `F-claude-781-configure-under-select-a-model-opened-the-hidden-agent-panel-001`,
  `PR-claude-781-divert-every-zed-action-that-focuses-the-agent-panel-001`.
- Brain: consultation 34e3b59f… closed with `brain no-decision` (a fix on #744's pattern).
- Ticket closed; pipeline archived; committed on `marley/workbench-shell`.
