# The Marley agent in the Agent Panel — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-683-the-marley-agent-in-the-agent-panel.md
- **Pipeline spec:** 683-the-marley-agent-in-the-agent-panel.spec.md

## Phase 1 — Plan
- **Request:** the queue's top, phase 1 item 5 of the Marley-agent plan, on Chad's "just have it
  go" (2026-10-07). Scoped at planning to Claude Code; Codex and Zed's agent became TICKET-687.
- **Classification / tier:** feature, prong 2 C, with three small additive Zed hunks.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (#682's install compiling; planning
  only until it ends) · recall ✓ · mint ✓ · prior art ✓ (an Explore read; the adapter's source in
  the user's Zed data directory, 0.81.0) · spec ✓ · design ✓.
- **Recall (§18.3):**
  - Chad's rules: a new Marley layer ships off with its own switch, and off leaves no trace
    ([[assistant-is-one-switch-not-two-builds]] for Rusty); `marley.assistant.enabled` follows it.
  - #685: Zed shows an ACP agent's messages and requests in the panel; #605/#501's stand-in agents
    are custom entries in the copied settings.
  - `F-claude-588-…`: a scenario opens its own folder, never the user's session, and here never a
    real Claude thread on the user's login.
  - `L-claude-682-no-source-edit-while-an-install-compiles-001`: source edits wait for #682's
    install.
- **Discovery:** the spec's prior art; the touchpoint ledger has rows for
  `settings_content/src/marley.rs` only, so `settings_content/src/agent.rs`,
  `project/src/agent_server_store.rs`, `agent_servers/src/acp.rs` and
  `agent_servers/src/custom.rs` each need a row first.
- **Decisions:** D1 to D4 in the spec.

### Design

**Approach.**

1. **Zed: settings** (`settings_content/src/agent.rs`). `CustomAgentServerSettings::Registry`
   gains `registry_id: Option<String>`, `display_name: Option<String>` and `session_meta:
   Option<serde_json::Map<String, Value>>`; `Custom` gains `session_meta`. Each with a doc comment
   and `// Marley:`. The project-side mirror in `agent_server_store.rs` carries them.
2. **Zed: the store** (`agent_server_store.rs`). `reregister_agents` looks a registry entry up by
   `registry_id` when set, else by its key, and passes `display_name` over the registry's name.
3. **Zed: the session** (`agent_servers/src/acp.rs`). `AcpConnectionDefaults` reads the entry's
   `session_meta` with its other settings; `into_new_session_request`, `into_load_session_request`
   and `into_resume_session_request` take it and set `.meta(...)` when present.
4. **Zed: the key rule** (`agent_servers/src/custom.rs`). `ANTHROPIC_API_KEY=""` goes to an entry
   whose registry id is `claude-acp`, the alias included.
5. **Marley: the switch** (`settings_content/src/marley.rs`, `default.json`, `MarleySettings`,
   the Settings window's Marley page). `assistant: Option<MarleyAssistantSettingsContent {
   enabled }>`, default off.
6. **Marley: `assistant.rs`** (new, `marley_workbench`).
   - `init`: observes the settings; while on, `update_default_settings` inserts the `Marley`
     registry entry (D2), and takes it out when off.
   - `INSTRUCTIONS`, the system-prompt append.
   - `offer`: after start, when the user's raw settings lack the key, runs `claude auth status`
     off the main thread through the module that already runs `claude --version`, and on
     `loggedIn` shows the app notification; Turn On and Not Now write the choice with
     `settings::update_settings_file`.
   - The `claude` path: the one Marley launches (`MARLEY_CLAUDE`, else the search path).
7. **The scenario** registers #685's scripted agent with a `session_meta` and reads its log; a fake
   `claude` on the PATH answers `auth status` and `--version`.

**File manifest.**

| File | Crate | Change |
|---|---|---|
| `crates/settings_content/src/agent.rs` | Zed | three fields on `Registry`, one on `Custom` |
| `crates/project/src/agent_server_store.rs` | Zed | lookup by `registry_id`; `display_name`; the mirror |
| `crates/agent_servers/src/acp.rs` | Zed | `session_meta` on the three requests |
| `crates/agent_servers/src/custom.rs` | Zed | the key rule by registry id |
| `crates/settings_content/src/marley.rs` | Zed (ledgered) | `assistant` |
| `assets/settings/default.json` | Zed (ledgered) | `marley.assistant.enabled: false` |
| `crates/marley_workbench/src/assistant.rs` | Marley | new |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley | settings field, `init` |
| `crates/marley_workbench/src/agent_versions.rs` (or the module that runs `claude`) | Marley | `claude auth status` |
| `docs/marley/zed-touchpoints.md` | docs | four new rows, two extended |
| `script/e2e/683-the-marley-agent-in-the-agent-panel.sh` | script | the scenario |

### Visual check plan

| REQ | Setup and action | Shot and evidence |
|---|---|---|
| 001 | the fake `claude` signed in; the copy's settings without `marley.assistant` | `683-01-offer` |
| 002 | click Turn On | the copy's settings hold `"assistant": {"enabled": true}`; `683-02-turned-on` |
| 003 | the project's + → New Agent Thread | `683-03-menu`: Marley among the agents |
| 005 | a thread of the scripted agent whose entry carries `session_meta: {"marley": "proof"}` | its log's `session/new` holds `_meta.marley == "proof"` |
| 006 | `settings_read agent_servers.Marley` through the stand-in | the entry: `registry_id` `claude-acp`, `display_name` Marley, the meta's append and the five tools |
| 004 | `marley.assistant.enabled` set false in the copy, the submenu again | `683-04-off`: no Marley |

No scenario opens a real Marley thread (it would run Claude Code on the user's login); Chad's
first one is the live check.

### Risks
- **The registry list must be fetched** for the alias to resolve; the copied profile carries the
  cached `registry.json`.
- **A user who changes the Marley entry's mode** gets a bare `{"type":"registry"}` written for it
  (`custom.rs`'s `default_settings_for_agent`), which replaces the default entry and drops the
  alias; recorded, to fix if it bites.
- **Four Zed files touched**: each hunk additive with `// Marley:`, each row written first.

## Phase 2 — Code
- **Built:**
  - Zed: `crates/agent_servers/src/acp.rs` `marley_session_meta(id, cx)` and `.meta(...)` on the
    three session requests (one hunk, its touchpoint row first).
  - `crates/settings_content/src/marley.rs`: `assistant` (`MarleyAssistantSettingsContent {
    enabled }`) and `agent_session_meta`.
  - `assets/settings/default.json`: `marley.assistant.enabled: false`.
  - `crates/settings_ui/src/marley_page.rs`: `assistant_section()`, the Marley Agent switch. The
    rows for all four files are written or extended.
  - Marley: `crates/marley_workbench/src/assistant.rs` (`init`, `sync`, `session_meta`, `resolve`,
    `set_entry`, `claude_program`, `offer_later`, `decided`, `show_offer`, `choose`), `init` from
    `marley_workbench::init`. `claude auth status` runs through `process::output`, so the spawn
    count does not move.
  - The scenario.
- **Deviations from the plan, all toward fewer Zed hunks:**
  - **No registry alias or display name.** Adding fields to `CustomAgentServerSettings` would
    break struct literals in `agent_ui`, `onboarding`, `project` and `agent_servers`. Instead:
    - the session meta lives in Marley's own settings block (`marley.agent_session_meta`, keyed
      by agent id), so `acp.rs` is the one Zed hunk with behavior;
    - the `Marley` entry is a custom agent whose command Marley resolves from the `claude-acp`
      registry entry through a project's `AgentServerStore` (`get_command`, which installs the
      adapter when it must);
    - its key, "Marley", is the name the menu shows;
    - when the user's settings lack `claude-acp`, Marley adds it to the in-memory defaults too,
      and takes it out with the switch.
  - `custom.rs` is untouched: the entry sets `ANTHROPIC_API_KEY=""` in its own environment.
  - `MARLEY_ASSISTANT_ADAPTER` names an ACP program to run in the adapter's place, as
    `MARLEY_CODEX` names a stand-in: the scenario needs it to run without downloading Node and
    the adapter into its fresh profile, and it lets the check read the `_meta` a real Marley
    thread's `session/new` carries.
- **Review of the diff:** `sync` acts only when the switch moved (the defaults it writes fire the
  settings observer again); `resolve` runs once at a time and again from a new workspace when no
  project was open; the offer runs only while the user's raw settings lack the key, and both
  answers write it. Nothing reads an entity during its own update.
- **Checks before the gate:** clippy clean after two `needless_pass_by_ref_mut`; the scenario's
  first run green, 2 of 2, every guessed position right (`scratchpad/683-e2e-1.log`).
- **Gate:** `just gate-diff` green, 17 of 17 (`scratchpad/683-gate-1.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/683-the-marley-agent-in-the-agent-panel.sh` (`compositor sway`), run
  2, the Test phase's: 2 of 2 checks (`scratchpad/683-e2e-2.log`). The scripted agent's
  `session/new` carried `_meta.systemPrompt.append` starting "You are Marley's own agent, inside
  Marley…", `claudeCode.options.disallowedTools` `[Bash, Edit, Write, NotebookEdit, MultiEdit]`
  and `allowDangerouslySkipPermissions: false`.
- **Shots** (`scratchpad/shots-683/`, Marley only):
  - `683-01-offer` (REQ-001): bottom right, "Marley can help set itself up, through Claude Code:
    an agent in the Agent Panel that explains Marley and proposes settings changes for you to
    accept.", with Turn On and Not Now.
  - `683-02-turned-on` (REQ-002): the offer gone; the copy's settings hold
    `marley.assistant.enabled: true`.
  - `683-03-menu` (REQ-003): the project's + menu, New Agent Thread's submenu listing Zed Agent,
    Claude Agent and Marley.
  - `683-04-thread` (REQ-005, REQ-006): a Marley thread in the Agent Panel, "hello" answered
    "Noted: hello", the editor's placeholder "Message Marley", the rail's row "hello · Marley ·
    idle".
  - `683-05-off` (REQ-004): the switch off, the same submenu lists Zed Agent and Claude Agent only.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".
- **Not reached by a scenario:** Claude Code itself under the real adapter, on the user's login;
  Chad's first Marley thread after the install is the live check, and it spends his plan.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #683); `docs/marley/guide.md` (a "The Marley agent"
  section before the MCP server's) and the in-app guide (a section of its own);
  `docs/marley_architecture/marley_workbench.md` (`assistant.rs`); `docs/marley/zed-touchpoints.md`
  (the `acp.rs` row; the `marley.rs`, `default.json` and `marley_page.rs` rows extended, written
  in Code before the edits); the plan doc marks item 5's Claude Code half done.
- **Knowledge (§19):** `AD-claude-683-marleys-own-agent-is-claude-code-with-session-meta-from-marleys-settings-001`
  (decisions), `L-claude-683-a-zed-settings-enum-variant-is-costly-to-grow-001` (lessons). No
  `F-` block: no bug. Brain: consultation 597d7ec222cb4e43a0e0798dab19ef02 closed with
  `brain decide`, follow-up by 2026-10-28.
- **Ticket:** TICKET-683 closed; TICKET-687 (Codex, Zed's agent) queued.
- **Gate:** the in-app guide changed, so the gate ran again before the commit.
