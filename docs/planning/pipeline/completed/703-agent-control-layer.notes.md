# The agent-control layer: an activity log and a kill switch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-703-agent-control-layer.md
- **Pipeline spec:** 703-agent-control-layer.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: full control of Zed over Marley's MCP server, with security that
  doesn't ask for every action. The intake `docs/planning/intake/zed-control-over-mcp.md` orders it
  #703 to #707; this is the first.
- **Classification:** feature, Marley's MCP server. Marley crates, plus the `marley` settings
  block's Zed paths (rows exist; each is extended).
- **Recall (§18.3):**
  - No log of agents' tool calls exists. The server logs nothing per call; System One's day log
    covers only model-checked uses. #571's click pause and #525's terminal consent ask per call.
  - The app never sees an MCP session id, and Zed's agent threads share one session. A "this
    session" approval must key on the caller's terminal or client and the project, so it moves
    to #704 with the tool that first asks.
  - There is no global stop. The nearest are #525's Take Over (one terminal) and
    `transport::set_enabled` (`tool_off`).
- **Discovery (an Explore pass, 2026-10-09):** the settings pattern of `marley.voice` (#642);
  `mcp::answer` (mcp.rs ~514); `marley_mcp::{lookup, ToolSpec.tier, AppCall}`;
  `marley_system_one::files::{append_in, read_day_in}`; `click_pause::Who::of`; `home_page.rs`'s
  cards and the `rusty::home_tab::{card, row, muted}` helpers; `settings_change.rs` for writing a
  user setting.

### Design
- **`marley_mcp` (Marley crate):** `AppCall` gains `on_answer: Option<AnswerHook>`.
  - `AnswerHook` wraps `Box<dyn FnOnce(&Result<ToolAnswer, Refusal>) + Send>`, with a manual
    `Debug`.
  - `AppCall::on_answer(&mut self, hook)` sets it.
  - `answer` converts the result, calls the hook with it, then sends it.
- **Settings (Zed paths, rows extended first):**
  - `settings_content/src/marley.rs`: `agent_control: Option<MarleyAgentControlContent>` with
    `stopped: Option<bool>`, docs and its default;
  - `default.json`: `"agent_control": { "stopped": false }` with a comment;
  - `settings_ui/src/marley_page.rs`: an Agent Control section with the Stopped toggle.
- **`crates/marley_workbench/src/agent_activity.rs` (new, Marley crate):**
  - `ActivityRow { time, who, tool, summary, outcome }` and `Outcome { Done, Refused { code } }`.
  - `append_in(dir, day, row)` and `read_day_in(dir, day)`, System One's pattern.
  - The `AgentActivity` global holds the rows, newest last, capped at 500. It is filled at `init`
    from today's file and written off the main thread through a channel.
  - `stopped(cx)` reads the setting.
  - `gate(call: &mut AppCall, cx) -> Result<(), Refusal>` runs from `mcp::answer` for a
    write-tier tool. It sets the hook that records the outcome, and refuses with
    `agent_control_stopped` when stopped.
  - `summary(tool, args)` takes `command`, `text`, `url`, `key`, `keystrokes`, `name` or `path`,
    else the compact JSON, cut to 160 chars and redacted with `mcp::model_redactor`.
  - Actions: `StopAgentControl`, `ResumeAgentControl`, `OpenAgentActivity`. The first two write
    the setting through `settings::update_settings_file`.
  - `AgentActivityView`, an `Item` opened in the Home group (`groups::in_group(Home)`): a header
    with the state and Stop/Resume, then the rows.
- **`mcp.rs`:** in `answer`, after `permits`, when `marley_mcp::lookup(&call.tool)` is
  `Tier::Write`, it calls `agent_activity::gate(&mut call, cx)`. A refusal answers, and the hook
  logs it.
- **`home_page.rs`:** an AGENT ACTIVITY card.
  - It observes the `AgentActivity` global.
  - It shows the state, the button (dispatching the actions from the page's focus handle), the five
    newest rows, and Open Activity.
- **`marley_workbench.rs`:** `pub mod agent_activity;` and its `init`.
- **File manifest:**
  - Marley crates: `marley_mcp/src/marley_mcp.rs`; `marley_workbench/src/agent_activity.rs`
    (new), `mcp.rs`, `home_page.rs` and `marley_workbench.rs`.
  - Zed crates: `settings_content/src/marley.rs`, `assets/settings/default.json` and
    `settings_ui/src/marley_page.rs`.
  - Docs: `docs/marley/zed-touchpoints.md` (three rows) and `docs/marley/guide.md`.
  - The scenario.

### Visual check plan
Under `compositor sway`, #491's scripted MCP client (Python, reads `MARLEY_MCP_ENDPOINT` =
`$E2E_PROFILE/mcp-endpoint.json`), run from the scenario's shell, writing each reply to a file.

| REQ | The scenario does | The shot |
|---|---|---|
| 001 | The client calls `terminal_list`, then `terminal_run` with `echo hello` on the project's terminal; then `marley: open agent activity` | 703-01-activity: the row (the client, `terminal_run`, `echo hello`, done) |
| 003 | `marley: stop agent control` | 703-02-stopped: the tab's header stopped, with Resume |
| 002 | The client calls `terminal_run` again and `terminal_list`; the replies are checked: `agent_control_stopped` and a list | 703-03-refused: the refused row |
| 003 | Home shown | 703-04-home-card: the card's state (stopped), Resume, the rows |
| 004 | — | The gate |

### Risks
- **A tool that answers twice or never.** The hook is `FnOnce` and runs inside `answer`, so a
  call that is never answered logs nothing. The connection's 30-second wait still ends it.
- **Arguments that carry secrets.** The summary goes through `model_redactor`, and only one line
  of it is kept.
- **The kill switch read from settings on every write call:** cheap, and a `SettingsStore`
  change applies at once.

## Phase 2 — Code
- **Built:**
  - `marley_mcp`: `AnswerHook` (over the `AnswerFn` alias) and `AppCall::on_answer`. `answer`
    converts the result, runs the hook, then sends.
  - Settings (Zed paths; the three rows were extended first): `MarleyAgentControlContent { stopped }`
    on `MarleySettingsContent.agent_control`, `default.json`'s `agent_control.stopped: false`, and
    the Settings page's Agent Control section with its Stopped toggle.
  - `marley_workbench.rs`: `AgentControl { Running, Stopped }` (from `marley.agent_control.stopped`)
    on `MarleySettings`, and the module with its `init`.
  - `agent_activity.rs` (new):
    - `ActivityRow` and `Outcome`, and `append_in`/`read_day_in` under
      `<data>/agent_control/activity-<day>.jsonl` (0600).
    - `AgentActivity`, a global of the rows. A background task reads today's file and then writes
      each new row in order; a foreground task shows them.
    - `gate` (the hook and the `agent_control_stopped` refusal), `summary`, and `set_stopped`
      through `settings::update_settings_file`.
    - The actions `StopAgentControl`, `ResumeAgentControl` and `OpenAgentActivity`.
    - `AgentActivityView` in the Home group, and `render_state`/`render_rows` shared with Home's
      card.
  - `mcp.rs`: `answer` gates a write-tier tool of a listed family before routing.
  - `home_page.rs`: the AGENT ACTIVITY card (state, Stop/Resume, five rows, Open Agent Activity),
    re-rendered on the log and the settings.
  - `guide.md`: "Agent activity and the kill switch".
- **Deviation:** the gate covers write tools of the families `tools/list` serves. The unlisted editor
  family (`editor_open`, `editor_wait`) serves `marley-edit`, the user's own `$EDITOR` (#649), which
  the switch must not stop. #704 adds agent-facing editor tools in a listed family, so they are
  gated. Their names must not collide with these two.
- **Review of the diff:**
  - The hook runs on whatever thread answers. It only sends on an unbounded channel, so it is
    `Send` and touches no entity.
  - `gate` reads globals during `mcp::answer`'s top-level update.
  - The Home page's handlers are plain closures over weak handles, as #701's rule asks.
  - The summary is redacted with `model_redactor` and cut to one line of 160 characters.
- **Gate:** `703-gate-1.log` RED (clippy `type_complexity` on the hook's box; dylint
  `async` blocks with no `.await`, from reading and writing the day file inside
  `background_spawn`); `703-gate-2.log` RED (clippy: `Eq` on the actions, `map_or_else`, a fourth
  bool on `MarleySettings`, `from_settings` over 100 lines). Fixes: the `AnswerFn` alias; one
  background task looping over the channel, as System One's writer does, and a foreground one
  showing; `#[derive(Eq)]`; `map_or_else`; `AgentControl` in place of the bool, one line in
  `from_settings`. `703-gate-3.log`: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/703-agent-control-layer.sh`, under `compositor sway`. A scripted MCP
  client (`clientInfo.name` `e2e-agent`) runs from the harness through the plugin's bridge, as
  Claude Code reaches Marley; it lists the terminals and calls `terminal_run` with `echo hello` on
  the first, printing `done` or `refused <code>` to a file the checks read.
- **Runs before the final one:**
  - `shots-703a`: the client named the terminal by `terminal_id` (the caller's own id, null for a
    terminal no agent runs in), so `terminal_run` refused `bad_argument`. It now names `id`, which
    `terminal`'s schema takes. A scenario fix.
  - `shots-703b` (**red, REQ-001's shot**): every check passed, but 703-01 showed Home's page in
    front, not the Agent Activity tab that `marley: open agent activity` had just made.
    - #701's `home_page::ensure`, run when the Home group is shown, inserts the page at index 0
      without activating it.
    - `Pane::add_item_inner` doesn't move the active index for an item it doesn't activate, so the
      page took index 0, the active one.
    - Any tab opened into a Home group that had no page yet hid behind Home's page.
    - Fixed in `home_page::ensure`: the tab active before the insert is activated again.
  - `shots-703c`: 703-01 to 703-03 right; 703-04's click on the Home header left the tab in front,
    so the scenario now clicks the rail's Home row (y 129).
- **Gate after the fix:** `703-gate-4.log`, GATE GREEN [diff].
- **Final run (`shots-703d`): every check passes, every shot shows its criterion.**
  - **Checks:**
    - the first `terminal_run` ran (`done`);
    - the second was refused with `agent_control_stopped`, its next step naming `marley: resume
      agent control`;
    - `terminal_list` still answered (`listed 1`);
    - after the resume `terminal_run` ran again;
    - the day file `agent_control/activity-<day>.jsonl` holds the refusal.
  - **703-01-activity (REQ-001):** the Agent Activity tab in front in the Home group (Home's page
    beside it, rail rows Home and Agent Activity): "Agents can use Marley's tools that act; each
    call is listed here." with Stop, and the row 13:28:19, e2e-agent, terminal_run, echo hello,
    done.
  - **703-02-stopped (REQ-003):** after `marley: stop agent control`: "Stopped: Marley's tools that
    act refuse every agent. The tools that only read still answer." in red, with Resume.
  - **703-03-refused (REQ-002):** two rows, newest first: 13:28:35 … refused:
    agent_control_stopped (red), then the earlier done row.
  - **703-04-home-card (REQ-003):** Home's page with the AGENT ACTIVITY card under CONFIGURE: the
    stopped line, Resume, the two rows, Open Agent Activity.
  - Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md`, under Added (agent activity and the kill switch) and Fixed (a tab opened into
    Home comes to the front).
  - `docs/marley_architecture/marley_workbench.md`, "Agent activity and the kill switch".
  - The guide's "Agent activity and the kill switch" (Phase 2), and the intake
    `docs/planning/intake/zed-control-over-mcp.md`, the plan for #703 to #707.
  - The rows for `settings_content/src/marley.rs`, `default.json` and `settings_ui/src/marley_page.rs`
    in `docs/marley/zed-touchpoints.md`, checked against what shipped.
- **Knowledge appended:** F-claude-703-homes-page-hid-a-tab-just-opened-into-home-001,
  PR-claude-703-an-unactivated-insert-keeps-the-active-tab-001,
  AD-claude-703-agent-control-starts-with-a-log-and-a-kill-switch-001.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decision is in the ledger.
- **Ticket:** closed; the BACKLOG row left at promotion. TICKET-704's text now carries the
  per-area modes and the once-per-session question.
- **Gate:** `703-gate-4.log`, GATE GREEN [diff], on the tree committed.
