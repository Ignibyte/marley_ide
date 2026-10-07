# The Marley agent on Codex and on Zed's agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-687-the-marley-agent-on-codex-and-zeds-agent.md
- **Pipeline spec:** 687-the-marley-agent-on-codex-and-zeds-agent.spec.md

## Phase 1 — Plan
- **Request:**
  - The plan's phase 1 item 5, second half, split from #683: the Marley agent for people without
    Claude Code.
  - Chad, 2026-10-07: "lets let you direct the harness and yourself and just have it go", so this
    phase runs without a review stop.
- **Classification / tier:** feature, prong 2 C. Marley crate work, plus Marley's own files in Zed
  crates (`settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`) and one renderer
  line.
- **Recall (§18.3):**
  - AD-683: the entry lives in the in-memory defaults, and its prompt rides
    `marley.agent_session_meta`. Its rejected list names a Zed-agent profile alone, because
    Zed's agent takes no system prompt; this ticket adds the profile for people without Claude
    Code.
  - L-683: no new fields on Zed's settings enums. A value only Marley sets lives in Marley's block.
  - AD-661 (one switch; off leaves no trace) and PR-686 (an offer that comes on its own is decided
    in the e2e copy). The copy already sets the switch off, and these scenarios undo that in
    `setup`.
  - Brain: consultation `03b63114f0fb40d5ad839f372f0648cd`. It returned only the follow-ups due;
    nothing on this seam.
- **Discovery** (an Explore pass over Zed's profile code, and codex-acp's source on GitHub):
  - Zed's profiles are `agent.profiles` (`AgentProfileContent`). The picker lists the merged
    settings', so an in-memory default shows under "Custom Profiles".
  - An explicit `context_servers.marley.tools` entry turns tools on with
    `enable_all_context_servers: false`.
  - A missing profile gives a thread no tools (`thread.rs:4176`). Picking a profile writes
    `agent.default_profile` to the user's file.
  - Zed's agent has no prompt per profile. `InitializeResponse` drops MCP `instructions`.
  - codex-acp 2.1.1 reads `INITIAL_AGENT_MODE` and `CODEX_CONFIG` from its environment.
  - Codex's config schema has `developer_instructions` and `sandbox_mode`. `codex login status`
    exits 0 or 1.
- **Decisions:** D1 to D6 in the spec.

### Design
**Approach.**
1. **The setting** (`settings_content/src/marley.rs`, Marley's file in a Zed crate):
   - `MarleyAssistantSettingsContent` gains `agent: Option<MarleyAssistantAgent>`.
   - `MarleyAssistantAgent { ClaudeCode, Codex, Zed }`, snake_case, with the strum derives the
     dropdowns take.
   - `default.json` names `"agent": "claude_code"`, so the dropdown always has a value. The
     switch's doc names the three agents.
2. **The dropdown:**
   - `assistant_section()` gains an Agent item (`marley.assistant.agent`), and the switch's
     description stops naming Claude Code alone.
   - `settings_ui.rs` gains the renderer line.
3. **`assistant.rs`:**
   - `Assistant.enabled: bool` becomes `applied: Option<MarleyAssistantAgent>`, what the defaults
     hold now.
   - `sync` compares the wanted `enabled.then(agent)` with `applied`. When they differ, it takes
     out what `applied` added, then adds the wanted agent's defaults:
     - **Claude Code:** #683's: the session meta, a `claude-acp` default when the user has none,
       then the resolve.
     - **Codex:** a `codex-acp` registry default when the user has none, then the resolve.
     - **Zed:** the `marley` profile.
   - Taking the profile out also resets a user `agent.default_profile` of `marley` to `write`,
     through `update_settings_file` (D6).
   - `resolve` takes the adapter id from the agent.
   - `set_entry`'s environment depends on the agent:
     - Claude Code: `CLAUDE_CODE_EXECUTABLE` and an empty `ANTHROPIC_API_KEY`.
     - Codex: `INITIAL_AGENT_MODE=read-only` and `CODEX_CONFIG` =
       `{"developer_instructions": INSTRUCTIONS, "sandbox_mode": "read-only"}`.
   - The profile: `AgentProfileContent { name: "Marley", tools: {}, enable_all_context_servers:
     Some(false), context_servers: {marley: {tools: the seven → true}}, default_model: None }`.
     The server key is `mcp::CONTEXT_SERVER`.
   - The offer is `found_agent`:
     - `claude auth status`'s `loggedIn`;
     - else `codex login status`'s exit status, run on the `codex` Marley launches (the one
       `agent_versions` found, else `MARLEY_CODEX`, else the search path);
     - else `LanguageModelRegistry::default_model()` with an authenticated provider.
   - `show_offer(agent)` names the agent. `choose(true, agent)` writes both keys, and Not Now
     writes `enabled: false`.
   - `filter_palette` lists the terminal command while on and the agent is not `zed`.
   - `open_in_terminal` builds the Claude Code line as #684 does. For Codex the line is
     `codex --sandbox read-only -c developer_instructions=<json string>`: a JSON string is a TOML
     basic string, quoted for the shell.
   - `INSTRUCTIONS` gains the keymap line, and its last paragraph no longer says the tools are
     "turned off".
4. **`Cargo.toml`:** `language_model` moves into `[dependencies]`; the dev entry keeps its
   `test-support`.
5. **Ledger:** rows for `marley.rs`, `default.json`, `marley_page.rs` and `settings_ui.rs` in
   `docs/marley/zed-touchpoints.md`.

**File manifest.**
| File | Owner | Change |
|---|---|---|
| `crates/settings_content/src/marley.rs` | Zed crate (Marley's file) | `agent`, `MarleyAssistantAgent` |
| `assets/settings/default.json` | Zed crate | `marley.assistant.agent: "claude_code"` |
| `crates/settings_ui/src/marley_page.rs` | Zed crate (Marley's file) | the Agent dropdown |
| `crates/settings_ui/src/settings_ui.rs` | Zed crate | the renderer line |
| `crates/marley_workbench/src/assistant.rs` | Marley crate | per-agent defaults, the offer, the terminal |
| `crates/marley_workbench/Cargo.toml` | Marley crate | `language_model` |
| `docs/marley/zed-touchpoints.md` | docs | the rows |
| `script/e2e/687-the-marley-agent-on-codex.sh` | Marley e2e | new |
| `script/e2e/687-the-marley-agent-on-zeds-agent.sh` | Marley e2e | new |

### Visual check plan
Each scenario runs under `compositor sway`, starts undecided (its `setup` removes
`marley.assistant` from the copy) and puts fakes on `MARLEY_CLAUDE` and `MARLEY_CODEX`.

**`687-the-marley-agent-on-codex.sh`**
- Its fakes:
  - `claude`: signed out (`{"loggedIn": false}`).
  - `codex`: signed in (exit 0). Its other arguments are written to a file, then it sleeps.
  - `MARLEY_ASSISTANT_ADAPTER`: #683's scripted agent, which also writes its `INITIAL_AGENT_MODE`
    and `CODEX_CONFIG` to a file at start.
- Its steps:

| REQ | What the scenario does | Shot / check |
|---|---|---|
| REQ-001 | waits for the offer | `687-01-codex-offer`: names Codex |
| REQ-002 | clicks Turn On | the copy holds `enabled: true`, `agent: "codex"` |
| REQ-003 | opens a Marley thread from the project's submenu and types `hello` | `687-02-codex-thread`; the stand-in's file holds `read-only` and a config whose `developer_instructions` starts "You are Marley's own agent" with `sandbox_mode` `read-only` |
| REQ-004 | opens the Settings window and searches "Marley Agent" | `687-03-setting`: Agent: Codex |
| REQ-005 | runs `open marley agent in terminal` from the palette | `687-04-codex-terminal`; the fake's arguments hold `--sandbox read-only` and `developer_instructions=` with the role |

**`687-the-marley-agent-on-zeds-agent.sh`**
- Its fakes and setup:
  - `claude`: signed out. `codex`: signed out (exit 1).
  - `OPENAI_API_KEY` is set to a dummy, and the copy's `agent.default_model` names OpenAI. Zed's
    OpenAI provider is authenticated by the variable, and no request is ever sent.
- Its steps:

| REQ | What the scenario does | Shot / check |
|---|---|---|
| REQ-006 | waits for the offer | `687-05-zed-offer`: names Zed's agent |
| REQ-007 | Turn On, then a Zed Agent thread, then the profile selector | `687-06-profile`: "Marley" under Custom Profiles; `settings_read agent.profiles.marley` through the stand-in MCP client shows `tools` empty, `enable_all_context_servers` false and the seven Marley tools |
| REQ-008 | picks Marley (the user's `default_profile` becomes `marley`), then writes the switch off | `settings_read` finds no `agent.profiles.marley`; the copy's `default_profile` is `write` |

**What no scenario reaches:** a real Codex thread on a real login. The stand-in reads what Zed
starts the adapter with; the adapter's reading of it rests on its source (Prior art). Chad's own
Codex is the check after install.

### Risks
- codex-acp's read-only mode asks before an edit, so the user can still approve one or switch the
  mode in the panel. That stays the user's call; the instructions say not to edit.
- Zed's providers authenticate after start. A provider that is not ready 5 seconds after start is
  missed, and the offer comes at a later start (the switch is still undecided).
- Turning a tool on in Zed's tool picker for the Marley profile copies the profile into the user's
  `settings.json`, where it outlives the switch. This is Zed's behavior for a profile that exists
  only in the defaults.
- `settings_read` of an in-memory default: checked in Code. If `effective` does not see the
  defaults, the check reads them through `raw_default_settings` instead.

## Phase 2 — Code
- **Built:**
  - `MarleyAssistantAgent { ClaudeCode, Codex, Zed }` and `MarleyAssistantSettingsContent.agent`.
    `default.json` names `"agent": "claude_code"`.
  - The Settings window's Agent dropdown, with its renderer.
  - `assistant.rs`, reworked:
    - `wanted` (the switch with the agent) and `Assistant.applied`.
    - `sync` takes the old agent's defaults out and puts the new one's in (`take_out`,
      `put_in`).
    - `profile()`, the Marley profile. `codex_config()`, with `set_entry`'s environment set per
      agent.
    - `resolve` tracks the agent it resolves for (`resolving: Option<_>`), and sets the entry only
      while that agent is still applied.
    - `forget_default_profile`, called when the profile leaves and at the offer's moment.
    - `offer_later` asks `claude_signed_in`, then `codex_signed_in`, then `zed_model_ready`.
      `show_offer(agent)` names the agent, and `choose(Option<agent>)` records the answer.
    - `open_in_terminal` builds the Claude Code or Codex line.
    - `INSTRUCTIONS` names `keymap_change`.
  - `agent_versions::variable` is now `pub(crate)` for `program`. `mcp::CONTEXT_SERVER` is now
    `pub(crate)`. `language_model` joins the workbench's dependencies.
- **Deviations:** none from the design.
- **Review of the diff:**
  - Correctness: an agent switch made during a resolve could have set the old agent's entry. The
    resolve now checks the applied agent when it finishes (`still_applied`).
  - REQ-008's reset runs only when the merged settings lack the profile, so a `marley` profile of
    the user's own keeps their default.
  - Re-entrancy: each `update_default_settings` finishes before the next starts. The observer it
    fires sees `applied` already moved.
  - No token is read: `codex login status` is read for its exit status only.
  - Provenance: nothing taken from Warp. The reset of `default_profile` follows the behavior of
    Zed's profile delete, written here as a check of Marley's own key.
  - `settings_read` reads the Default file, the in-memory defaults (`get_all_files` ends with
    `SettingsFile::Default`), so REQ-007's check can read the profile there.
- **Gate:**
  - `scratchpad/687-gate-1.log` was red, on two clippy findings in `profile()`
    (`default_trait_access`, and `into_iter` on one item). The maps are now built plainly.
  - `scratchpad/687-gate-2.log`: **GATE GREEN [diff]**, 17 passed.

## Phase 3 — Test
- **Scenarios** (`compositor sway`):
  - `script/e2e/687-the-marley-agent-on-codex.sh`, run 4: 4 of 4 (`scratchpad/687-codex-e2e-4.log`).
  - `script/e2e/687-the-marley-agent-on-zeds-agent.sh`, run 2: 5 of 5 (`scratchpad/687-zed-e2e-2.log`).
- **Shots** (Marley only):
  - `687-01-codex-offer` (REQ-001): "Marley can help set itself up, through Codex in its read-only
    mode…", with Turn On and Not Now.
  - `687-02a-codex-menu`: New Agent Thread's submenu lists Zed Agent, Claude Agent, Codex and
    Marley. Codex is the registry entry Marley added; Marley is three steps down.
  - `687-02-codex-thread` (REQ-003): a thread titled "hello" on the Marley entry. The rail reads
    "Marley · idle", and the stand-in answered "Noted: hello". Its environment had
    `INITIAL_AGENT_MODE=read-only`, and its `CODEX_CONFIG` held `developer_instructions` (which
    starts "You are Marley's own agent" and names `keymap_change`) and `sandbox_mode` read-only.
    It had no `CLAUDE_CODE_EXECUTABLE`.
  - `687-03-setting` (REQ-004): the Settings window searched for "Marley Agent". The section shows
    the switch on and Agent: Codex, with the dropdown's description.
  - `687-04-codex-terminal` (REQ-005): a center terminal "repo — codex …". The fake printed
    `--sandbox read-only -c developer_instructions="You are Marley's own agent…"`, the whole
    instructions in one JSON string.
  - `687-05-zed-offer` (REQ-006): "Marley can help set itself up, through Zed's agent: a Marley
    profile, with Marley's tools and no file tools…".
  - `687-06-profile` (REQ-007): a New Zed Agent Thread on GPT-4o Mini, with the profile picker
    open: Write ✓, Ask, Minimal, then "Custom Profiles", then Marley.
    `settings_read agent.profiles.marley` answered name Marley, `tools: {}`,
    `enable_all_context_servers: false`, and `context_servers` holding only `marley` with the
    seven tools.
  - REQ-008: picking Marley wrote `agent.default_profile: "marley"`. With the switch off,
    `settings_read` answered `no_setting` and the default was `write` again.
- **Focus report:** both runs said "1 Marley windows before the run, 1 after; the run added no
  rule and did not reload it".
- **Fixes to the scenarios** (no source change):
  - Codex run 1 counted two steps down the submenu, as #683 did. With Codex's registry entry there,
    that opened the real Codex through `codex-acp` on Chad's login, and the typed "hello" got an
    answer. The step count is now three.
  - Codex run 2: the Settings search took no text until its field was clicked.
  - Zed run 1: `agent: new thread` opened the panel's last agent, Claude Agent through
    `claude-acp` on Chad's login. With no profile picker there, the typed "Marley" and Return
    likely went to it as a prompt. The scenario now opens Zed Agent from the submenu, where it is
    always first.
  - The Codex scenario types only after a check that the thread runs the stand-in.
  - These are F-687 and PR-687 at Complete.

## Phase 4 — Complete
- **Docs (§21):**
  - `CHANGELOG.md` (Added, #687).
  - The guide's "The Marley agent" section: the three agents, turning it on, using it, the
    terminal, and how it is made.
  - The in-app guide's paragraph.
  - `docs/marley_architecture/marley_workbench.md` (since #687).
  - The plan doc's item 5, marked done.
  - `docs/marley/zed-touchpoints.md` rows for `marley.rs`, `default.json`, `marley_page.rs` and
    `settings_ui.rs` (written in Code).
- **Knowledge (§19):**
  - `AD-claude-687-the-marley-agent-runs-on-codex-through-codex-acps-environment-and-on-zeds-agent-as-a-profile-001`
  - `L-claude-687-zeds-agent-drops-mcp-instructions-and-has-no-prompt-per-profile-001`
  - `F-claude-687-a-scenario-a-step-off-opened-a-real-agent-on-the-users-login-001`
  - `PR-claude-687-a-scenario-types-into-a-thread-only-after-checking-it-runs-the-stand-in-001`
  - Brain: consultation `03b63114f0fb40d5ad839f372f0648cd` closed with `brain decide`
    (`decisions/the-marley-agent-runs-on-codex-through-codex-acps-environment-and-on-zeds-agent-as-a-profile`).
- **Ticket:** TICKET-687 closed.
