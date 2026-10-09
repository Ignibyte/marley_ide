# The Marley and Rusty agents out of the box — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-696-marley-and-rusty-agents-out-of-the-box.md
- **Pipeline spec:** 696-the-marley-and-rusty-agents-out-of-the-box.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-08 (#2 of his three): the Marley agent pre-configured on out of the
  box, and Rusty the agent behind Rusty, both in the Agent Panel. He also said the Marley agent's
  sole purpose is to tinker and change settings without digging through them, and that it should
  know Zed and Marley. It already does, through `docs_search`/`docs_read` over Zed's docs and
  Marley's guide (#681) and `settings_change` (#682).
- **Classification:** feature, prong 2 C, in Marley crates and Marley's settings block. The work
  runs autonomously, on Chad's word.
- **Recall (§18.3):**
  - AD-683 and AD-687: the Marley agent is a custom entry or a profile in the in-memory
    defaults, its prompt in the session `_meta` keyed by entry. Off by default and offered once;
    that is now reversed by Chad's choice.
  - AD-643 and AD-661: Rusty is one switch, and off leaves no trace. Rusty's `rusty` context
    server reaches Zed's agents only with `agent_tools` on (AD-642: Rusty's tools wait to be
    turned on), which Chad's settings leave off. So the Rusty entry must carry Rusty's server
    itself, not turn `agent_tools` on.
  - L-683: don't grow a Zed settings enum. `MarleyAssistantAgent` is Marley's own and only
    Marley code matches on it; `settings_ui` renders it generically.
  - PR-686 (an offer that comes at start is decided in the e2e copy) and PR-687 (check the stand-in
    before typing into a thread) shape the scenario.
  - The brain: "the manager talks to the person in Zed's Agent Panel over ACP" (#694) and
    "Marley offers rusty-mcp to Zed's agents where it is installed" (#633). Nothing on these
    defaults.
- **Discovery:**
  - `crates/marley_workbench/src/assistant.rs` (685 lines) holds everything for the one entry.
  - `rusty.rs` has `Source`, `find` and `loopback`.
  - `settings_content/src/marley.rs` has the enum, `default.json` the defaults, and
    `settings_ui/src/marley_page.rs` the texts.
  - claude-acp's `acp-agent.js` merges the meta's `mcpServers` (around line 6110).

### Design
- **`settings_content/src/marley.rs`** (a Marley file in a Zed crate; its row exists):
  - `MarleyAssistantAgent::Auto`, first and `#[default]`, with a doc comment for the schema;
  - `enabled`'s doc updated: default true, no offer.
- **`assets/settings/default.json`:** `"assistant": { "enabled": true, "agent": "auto" }`, with
  its comment.
- **`assistant.rs`:**
  - **Two entries.** `ENTRY` (`Marley`) and a new `RUSTY_ENTRY` (`Rusty`). The profiles are
    `marley` and `rusty`.
  - **Auto.** `Assistant` gains `detected: Option<MarleyAssistantAgent>` and `detecting: bool`.
    `resolved(cx)` gives the setting's agent, or for `Auto` the detected one.
  - **What is wanted.** `wanted(cx)` (the Marley entry) is `enabled && resolved`. A new
    `wanted_rusty(cx)` is `rusty::is_on && resolved`.
  - **Detection.** `detect(cx)` replaces `offer_later`:
    - it runs when Auto is set and something wants an agent, and nothing was detected yet;
    - it checks `claude_signed_in`, then `codex_signed_in`, then waits `ZED_MODEL_AFTER` and
      checks `zed_model_ready`;
    - it logs `assistant: auto chose …`, stores the result and calls `sync`.
  - **Removed:** `show_offer`, `choose`, `AssistantOffer` and `decided`.
  - **`sync`** follows both entries. `applied` becomes `(marley, rusty)`, and the Rusty side also
    keeps the Rusty server it was put in with, so a change of source puts it in again.
  - **Putting in and taking out.**
    - `put_in` and `take_out` work per entry.
    - The adapter is added once while either entry wants it.
    - The command is resolved once and `set_entry` writes every wanted entry.
    - The session meta and the Codex config are built per entry: `session_meta(entry)` and
      `codex_config(entry)`. Rusty's meta adds `mcpServers.rusty` and `mcp__marley` to the
      disallowed tools; Rusty's Codex config adds `mcp_servers.rusty`.
  - **Zed's agent.** The `rusty` profile has only the `rusty` server's tools (all of them,
    `enable_all_context_servers: false` with the server's preset set to all). The context server
    `rusty` is offered while that profile is wanted, through `rusty.rs`.
  - **`RUSTY_INSTRUCTIONS`:** Rusty, the user's personal assistant inside Marley, works through
    Rusty's tools (tasks, memories, the brain), does not edit files or run commands, and points
    to the Marley agent for Marley's settings.
- **`rusty.rs`:**
  - `pub(crate) enum RustyServer { Stdio(PathBuf), Http(String) }` and
    `pub(crate) fn agent_server(cx) -> Option<RustyServer>`: the source Marley's own connection
    uses, found the same way. It is resolved off the main thread, like `offer`'s `find`.
  - `offer` (the `rusty` context server for Zed's agents) also runs while the assistant wants
    the `rusty` profile.
- **`settings_ui/src/marley_page.rs`** (Zed crate, Marley file; its row exists): the Marley
  Agent and Agent descriptions. The default is on, and Auto picks the first agent found.
- **`docs/marley/guide.md`:** the Marley agent section says it is on by default with Auto and
  that no offer comes; a Rusty entry paragraph.
- **The ledger:** the rows of `default.json`, `settings_content/src/marley.rs` and
  `settings_ui/src/marley_page.rs` get #696's words.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001 | Takes `marley.assistant` out of the run's settings; a fake `claude` that says it is signed in; starts | 696-01-no-offer: the window 10 s after start shows no notification. 696-02-agents: the + menu's New Agent Thread submenu lists Marley |
| 002 | The same run | `holds` the Marley log line `assistant: auto chose Claude Code` |
| 003 | Rusty on, with the stand-in `rusty-mcp` (`MARLEY_RUSTY_MCP`) | 696-02-agents lists Rusty below Marley |
| 004 | Opens a Rusty thread from the submenu; checks the stand-in's log first (PR-687) | `holds` the agent log's `session/new` `_meta`: Rusty's instructions, `mcpServers.rusty.command` = the stand-in, `disallowedTools` with `mcp__marley`. 696-03-rusty-thread |
| 005 | `profile_setting` turns `marley.rusty.enabled` off | 696-04-rusty-off: the submenu lists Marley, not Rusty |
| 006 | — | Review of the diff. A Codex or Zed run would repeat #687's scenarios for the second entry; their shapes are the same code path as the Marley entry's |

### Risks
- **Turning it on by default.** A user signed in to Claude Code now gets a Marley entry they
  didn't ask for. Chad asked for exactly that. `enabled: false` removes it, and the entry is only
  in the defaults, never written.
- **The adapters.** Real Claude Code honours `mcp__marley` in `disallowedTools`, and real Codex
  reads `CODEX_CONFIG.mcp_servers`. No scenario reaches either (PR-687 keeps runs off real
  accounts); the scenario proves what Marley sends.
- **Repeated detection.** Detection runs `claude auth status` at each start, as the offer did,
  but now even when the user has decided. It is cheap and prints no token.

## Phase 2 — Code
- **Built:**
  - `MarleyAssistantAgent::Auto`, first and the default. `default.json` has `enabled: true` and
    `agent: "auto"`. The settings page's Agent text and the guide's Marley agent section were
    rewritten, with a new "Rusty in the Agent Panel" section.
  - `assistant.rs`:
    - `Entry { Marley, Rusty }` with each one's name, profile and instructions, and
      `RUSTY_INSTRUCTIONS`.
    - `Entries { agent, marley, rusty }` is what the defaults hold or what is wanted.
    - `sync` takes out, then puts in, only the entries that changed. The adapter changes only with
      the agent, so an entry a thread runs on stays while the other moves.
    - `detect` and the `Detected { NotYet, Looking, Found }` state replace the offer. It waits
      `DETECT_AFTER` (1 s), checks Claude Code, then Codex, then after `ZED_MODEL_AFTER` Zed's
      model, logs `assistant: auto chose …` and syncs.
    - `session_meta(entry, rusty)` adds `mcpServers.rusty` and `mcp__marley`.
      `codex_config(entry, rusty)` adds `mcp_servers.rusty`.
    - `profile(entry)`: the Rusty profile enables the context servers.
    - `set_entries` writes every applied entry once the adapter's command is known.
    - The offer, `choose` and `decided` are gone.
  - `rusty.rs`:
    - `RustyServer { Stdio, Http }`.
    - `Rusty.agent_server`, set when a connection is up, kept while it is down, and cleared when the
      source changes.
    - `agent_server()`, `offer_again()`, and `CONTEXT_SERVER` made `pub(crate)`.
    - `follow_setting` offers the `rusty` context server while `agent_tools` is on or the Rusty
      profile is wanted.
  - `script/e2e.sh`'s comments say why the copy keeps the agent off now that it is on by default.
  - The ledger: rows 59, 63 and 67 name #696.
- **Deviations:**
  - The Zed profile for Rusty turns on the context servers' tools rather than Rusty's alone
    (D6 updated). A profile names a server's tools one by one, Rusty's tool list is Rusty's to
    change, and Zed asks before each call.
  - Detection waits 1 s first (`DETECT_AFTER`). It must not spawn `claude` inside Zed's own tests,
    whose clock stands still (F-634), and the user's settings file has loaded by then.
- **Review of the diff:**
  - An entry is compared with what it holds (agent and Rusty's server), so Rusty's connection
    coming up adds Rusty's entry without touching Marley's.
  - Rusty off clears `agent_server` through `follow_setting`, whose global change syncs.
  - A detection the setting moved away from while it looked is dropped.
  - `forget_default_profile` takes the entry, so a user default of `rusty` resets when the Rusty
    profile leaves.
  - No IO on the main thread: the status checks run through `process::output`.
  - No Zed hunk beyond the existing rows.
- **Gate:**
  - Run 1 (`696-gate-1.log`) was red on rustfmt, fixed with `rustfmt` on the two files while the
    gate's cargo ran, and on clippy `option_option` (`Option<Option<_>>`), now the `Detected`
    enum.
  - Run 2 (`696-gate-2.log`): **GATE GREEN [diff]**, 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/696-the-marley-and-rusty-agents-out-of-the-box.sh`, under
  `compositor sway`, on the debug build.
  - A first run with `STOP_AT_MENU=1` found Rusty's place in the submenu before any thread
    opened: Zed Agent, Claude Agent, Marley, Rusty, so `RUSTY_STEPS=3`.
  - Nothing is typed into a thread (PR-687). The thread's `session/new` alone carries the meta.
- **Run (`696-e2e-2.log`): every check passes.**
  - **696-01-no-offer (REQ-001):** the window 15 s after start shows the project, a terminal and
    the right panel, with no notification. "auto chose Claude Code" passes in Marley's log
    (REQ-002).
  - **696-02-agents (REQ-001, REQ-003):** the + menu's New Agent Thread submenu lists Zed Agent,
    Claude Agent, **Marley** and **Rusty**, with the Rusty button beside PROJECTS.
  - **696-03-rusty-thread (REQ-004):** the Agent Panel shows "New Rusty Thread" with "Message
    Rusty — @ to include context", and the rail a row "New Agent Thread · Rusty · idle". The
    stand-in's `session/new` `_meta` has:
    - an append starting "You are Rusty, the user's personal assistant";
    - `mcpServers.rusty = {type: stdio, command: …/bin/rusty-mcp, args: []}`, the stand-in;
    - `disallowedTools` Bash, Edit, Write, NotebookEdit, MultiEdit and `mcp__marley`;
    - `allowDangerouslySkipPermissions: false`.
  - **696-04-rusty-off (REQ-005):** with `marley.rusty.enabled` off, the submenu lists Zed Agent,
    Claude Agent and Marley, and no Rusty. The Rusty button beside PROJECTS has gone (AD-661). The
    thread opened before stays in the panel; only new threads lose the entry.
- **Not reached by a scenario (REQ-006):** the Codex and Zed forms of the Rusty entry. They take
  the code paths of #687's Codex and Zed scenarios, with `codex_config(entry, rusty)` and
  `profile(entry)` read in the review of the diff. Whether real Claude Code honours
  `mcp__marley` and real Codex reads `CODEX_CONFIG.mcp_servers` is theirs: PR-687 keeps runs off
  real accounts.
- **Disk:** the build first failed with no space left on the build disk. The cleanup recipe
  removed 749 old Marley incremental variants and freed 30 GB.
- **Gate after the scenario was added (`696-gate-3.log`):** GATE GREEN [diff], 17 passed.

## Phase 4 — Complete
- **Documented:**
  - a `CHANGELOG.md` entry;
  - `docs/marley/guide.md`'s Marley agent section and a new "Rusty in the Agent Panel" section;
  - `docs/marley_architecture/marley_workbench.md`'s Marley agent note, "Since #696";
  - the touchpoints rows of `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`
    and `default.json`.
- **Knowledge appended:**
  - AD-claude-696-the-marley-agent-is-on-with-auto-and-rusty-has-an-entry-of-its-own-001
  - L-claude-696-a-session-meta-can-carry-an-mcp-server-for-one-agent-entry-001
- **Brain:** `rusty-cli brain decide` recorded
  `decisions/marleys-own-agent-is-on-with-auto-and-rusty-gets-an-agent-panel-entry-of-its-own`,
  to follow up by 2026-11-08.
- **Follow-up:** TICKET-698, keeping the Marley agent to its eight tools on Claude Code and Codex,
  queued after 697.
