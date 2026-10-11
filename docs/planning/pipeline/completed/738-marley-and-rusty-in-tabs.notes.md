# Marley and Rusty in tabs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-738-marley-and-rusty-in-tabs.md
- **Pipeline spec:** 738-marley-and-rusty-in-tabs.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #683, #687, #696 (the entries and their agents), #698 (the Marley agent kept to its own tools), #684 (its terminal command, the palette filter pattern).
  - PR-claude-687-a-scenario-types-into-a-thread-only-after-checking-it-runs-the-stand-in-001.
  - #696's scenario set-up gives Rusty through `marley_rusty`'s stand-in `rusty-mcp`.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `assistant.rs`: `Entry` (150) with `name()`, `profile()`, `ALL`; `Entries::of` (197) on
    `Assistant.applied`; `filter_palette` (856) for `OpenMarleyAgentInTerminal`, called where the
    applied entries change (264, 344).
  - `ConversationView::as_native_thread` is `pub` (3090); `agent::Thread::set_profile` (2339).
  - `thread_tab::start` (#734) and `threads_page::open_thread` (#737).
  - `paths::data_dir()` is the profile's data folder (`--user-data-dir` in a run).
  - #696's scenario: the Rusty entry through `marley_rusty`'s stand-in `rusty-mcp`
    (`MARLEY_RUSTY_MCP`), both entries on `MARLEY_ASSISTANT_ADAPTER`.

### Design
- **`assistant.rs`** (Marley crate):
  - `actions!(marley, [TalkToMarley, TalkToRusty, NewMarleyConversation,
    NewRustyConversation])`, registered on every workspace in `init`.
  - `talk_to(entry, fresh, workspace, window, cx)`: nothing unless `applied.of(entry)`; the
    agent (the entry's server, or Zed's agent with the profile); unless `fresh` or on Zed's agent,
    the entry's newest unarchived conversation comes forward through `window.defer` +
    `threads_page::open_thread`; else `thread_tab::start_thread` on the entry's folder.
  - `folder_in(data, entry)`: `<data>/assistant/marley|rusty`, made when missing; called with
    `paths::data_dir()`.
  - The palette lists the Marley pair only while the Marley entry is there and the Rusty pair only
    while Rusty's is: `filter_palette` grows to the four types.
- **`thread_tab.rs`** (Marley crate): `start` resolves the agent and folder and calls
  `pub(crate) fn start_thread(workspace, agent: Agent, folder, profile: Option<AgentProfileId>,
  …)`; `ThreadSpec` gains `profile`; `build_view` sets it on Zed's agent's thread once it exists
  (an observer that runs once).
- **`threads_page.rs`:** `open_thread` becomes `pub(crate)`.
- **The guide:** "The Marley agent" and "Rusty in the Agent Panel" name the commands.
- **File manifest:** `assistant.rs`, `thread_tab.rs`, `threads_page.rs` (Marley crate); the guide;
  the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `marley: talk to marley` in the repo; "hello" | 738-01-marley; the log's `cwd` ends in `assistant/marley` |
| 002 | Home shown (its header clicked); `marley: talk to marley` | 738-02-again: the repo shown, the Marley tab in front |
| 003 | `marley: talk to rusty` | 738-03-rusty; the log's `cwd` ends in `assistant/rusty` |
| 004 | `marley: new marley conversation` | 738-04-new: a second Marley tab |
| 005 | — | The review of the diff |

### Risks
- **The user's own conversations.** The profile is a copy; a non-archived Marley conversation of
  the user's would be the "latest", and the run would open it. The scenario checks the new
  session first (PR-claude-687) and types nothing into another.

## Phase 2 — Code
- **Checklist** (no TaskCreate here): `assistant.rs` ✓, `thread_tab.rs` ✓, `threads_page.rs` ✓,
  guide ✓, scenario (before the gate) ✓, gate ✓.
- **Built:**
  - `assistant.rs`: `TalkToMarley`, `TalkToRusty`, `NewMarleyConversation`,
    `NewRustyConversation`, registered on every workspace; `talk_to` (the entry's agent, its
    latest unarchived conversation brought forward through `window.defer` +
    `threads_page::open_thread`, else a new thread on its folder; on Zed's agent always new, with
    the profile); `latest_conversation`; `folder_in(data, entry)` with `paths::data_dir()`;
    `filter_talk` lists each pair only while its entry is there (`talk_shown` keeps what the
    palette was told).
  - `thread_tab.rs`: `start_thread(workspace, agent, folder, profile, …)` split out of `start`;
    `ThreadSpec.profile`, which `build_view` puts on Zed's agent's thread once it exists (an
    observer that acts once); `open_thread` takes a `ThreadSpec`.
  - `threads_page.rs`: `open_thread` is `pub(crate)`.
  - The guide: "In a tab, anywhere" under the Marley agent and Rusty.
- **Deviations:** none from the design.
- **Found before the gate:** clippy's `too_many_arguments` (`open_thread` takes a `ThreadSpec`)
  and three `needless_pass_by_ref_mut` (`start_thread` takes shared references, `talk_to` a
  shared `Window`).
- **Review of the diff:** REQ-001/003 `talk_to` on a new folder; REQ-002 the deferred
  `open_thread`; REQ-004 `fresh`; REQ-005 `filter_talk`. Re-entrancy: the deferral keeps
  `open_thread` out of the workspace's update (PR-claude-735). IO: the folder is made under the
  data folder handed in.
- **Gate:** `738-gate-1.log`: GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/738-marley-and-rusty-in-tabs.sh`, under `compositor sway`. Both entries
  on #736's scripted agent, Rusty on through its stand-in `rusty-mcp`. The copy's thread list is
  emptied in `setup`.
- **Runs before the green** (`shots-738a`, `shots-738b`): the first talk started no session. The
  palette listed the command (738-00-palette), and the log held no error: the copied profile held a
  Marley conversation of the user's own, so talk-to opened that one, as designed, and the
  scenario's check stopped the run before anything was typed (PR-claude-687). `setup` now empties
  the copy's `sidebar_threads_v2` and `sidebar_threads`; the user's own database is untouched
  (`shots-738c` passed).
- **The Test phase's run (`shots-738-test`), after `just build` and `738-gate-1.log` green: every
  check passes.**
  - **738-00-palette:** `marley: talk to marley` listed in the palette while the entry is there.
  - **738-01-marley (REQ-001):** a "hello" tab in the repo answering "Noted: hello", its row
    "hello · Marley · idle"; the checks: one session, its `cwd` ending in `assistant/marley`.
  - **738-02a-home:** Home shown, its page with New Agent… first in NEW AGENT (#735).
  - **738-02-again (REQ-002):** the command again: the repo shown with the hello tab in front;
    the check: still one session.
  - **738-03-rusty (REQ-003):** a tab "Message Rusty" in the repo, its row "Rusty · idle"; the
    checks: a second session, its `cwd` ending in `assistant/rusty`.
  - **738-04-new (REQ-004):** a second Marley tab, its row "Marley · idle"; the check: a third
    session.
  - **REQ-005:** the review of the diff (`filter_talk`).
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: Marley and Rusty in tabs); the architecture note
  (`marley_workbench.md`, "Marley and Rusty in tabs (#738)" beside the Marley agent's entry); the
  slice line in `workbench-shell.md`; the guide came with Phase 2. No Zed path is touched.
- **Knowledge appended:** L-claude-738-a-run-that-counts-on-no-conversations-empties-the-copy-001.
- **Brain:** no `rusty` MCP server in this repository's sessions; no brain loop ran.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `738-gate-1.log`, GATE GREEN [diff], 17 passed, on the tree committed.
