# A remote manager's thread — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-758-a-remote-managers-thread.md
- **Pipeline spec:** 758-a-remote-managers-thread.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11: "is there a way to do remote agent threads. meaning if manager
  lives on a remote server and we dont want to steer them through the cli. Using or not using our
  harness (preferrable our harness)". Later the same day: "ok we can build it. once things are spec
  and then add the things for the missing manager." The second widened the ticket to every gap a
  remote manager has: an entry and a follower per harness, the remote-window trap, and a by-hand
  check on a real host. It goes in the Queue, before the finish.
- **Classification:** feature, M.
- **Recall (§18.3):**
  - AD-claude-685: the manager talks to the person in the Agent Panel over `rh acp`, "as
    `ssh HOST rh acp …` for a remote root".
  - AD-claude-694: the entry follows the `role` label.
  - AD-claude-688: records are polled each second on the fleet's connection; the first read is
    quiet.
  - AD-claude-740: "The Manager entry, `open harness session` and agents' seat tools stay with the
    first". The #740 spec says "the first harness with a manager", but the code follows the
    decision: the first harness only.
  - AD-claude-742 and PR-claude-742: a seat entry is named for the seat. A bare program is
    resolved on Marley's search path, because Zed runs an agent server in the project's shell
    environment (F-claude-742).
  - F-claude-740-a-host-sessions-tab-read-the-first-harness: a call that reads the global's field
    directly is missed when a feature becomes per slot. Read every touched function whole.
  - L-claude-685: Zed's panel shows an ACP agent's messages between turns silently. The tab shows
    the records; Marley raises the notice.
  - The harness: one manager per root (D130, `docs/MANAGER.md:184`). `rh acp` over SSH was
    qualified only through a loopback sshd on one machine (`docs/TESTING.md:416-417`, `444-445`).
- **Discovery:** the precise surface.
  - `harness.rs`, first harness only:
    - `has_manager` (431-437), `sync_manager_entry` (566-603) and `manager_in_front` (441-458);
    - `Harness.thread_cursor` (159), `Harness.unread` (161) and `manager_entry` (157);
    - `follow_thread` (512-537), called only for `Slot::Primary` (966-969);
    - `manager_posted` (541-559): notice tag `marley-manager-{id}`, title `Manager: {Kind}`.
  - `sync_manager_entry` runs from `change_seats` for the first harness (292) and from
    `follow_setting` (645, 668). It does not run when a host's seats change.
  - The Manager entry's program is not resolved on Marley's search path. A `marley.harness` command
    naming a bare `ssh` can hit F-claude-742.
  - `rail.rs`:
    - `manager_entries` (7037-7057), with the key `manager:{id}` and the project `"Harness"`;
    - `open_inbox_entry` (1965) calls `open_manager_thread` (1972-2003), which opens a listed
      Manager thread through `open_thread` (2823), or else `agents::start_thread`, the Agent Panel.
  - `agents.rs:163-184` (`thread_agents`): `Manager` is not filtered, so New Agent lists it
    (agents.rs:897), and a Manager thread started there runs in a tab that `manager_in_front` does
    not see.
  - `harness_seat.rs:418-434` (`seat_of_entry`): `Manager · box-2` would parse as profile `Manager`
    on `box-2` and be taken for a seat. A manager entry must be recognised first.
  - `thread_tab.rs:876-932` (`build_view`): it builds on the hosting workspace's project and the
    panel's connection store, and passes `None` for initial content. `ThreadSpec` (861-871) carries
    no session id; `build_view` reads the session from the metadata store by thread id (893-898).
  - The remote trap: a remote project's agent server store asks the host for agents
    (`agent_server_store.rs:710-786`), and the host never sees in-memory defaults
    (`project_settings.rs:979-999`).
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again. A sketch follows.

### Design
- **Approach:**
  - **`harness.rs`:**
    - `has_manager(slot)`, `writes_on_slot(slot)`;
    - `sync_manager_entries(cx)` over every slot: the entry's name from `manager_entry_name(slot)`
      and its command from `seat_command_of(slot)` + `acp`, with a bare program resolved through
      the same helper `ensure_seat_entry` uses. It runs after any slot's seats change and when the
      settings change, and it removes entries for slots that no longer qualify.
    - `follow_thread(slot)` after every slot's fleet pass. The cursor and unread records go in
      `Harness` for the first harness and in `harness_hosts::Followed` for a host.
    - `manager_in_front(slot)`: the panel case, or `thread_tab::shows_agent` on the active
      window's active item.
    - `manager_posted(slot, …)` titles its notice `Manager · <name>: <Kind>` for a host.
  - **`harness_seat.rs`:** `manager_of_entry(name) -> Option<Slot>`, checked before
    `seat_of_entry`, which returns `None` for the profile `Manager`.
  - **`thread_tab.rs`:**
    - `HarnessProject`, a global holding a local `Project` with no folders (`Project::local` from
      the app state) and its `AgentConnectionStore`, made on first use. `build_view` uses it when
      the agent is a seat or manager entry.
    - `ThreadSpec` gains `session: Option<SessionId>`, used when there is no record yet.
    - `open_manager_thread(workspace, slot)`: an open tab, else the newest saved thread on the entry
      (the metadata store), else a new tab on `thread-<instance>-0`, taken from the connection's
      `session_list`.
    - Restore and `open_saved` make a manager entry again, as they do a seat entry.
  - **`rail.rs`:**
    - `manager_entries` per slot, keyed `manager:{name}:{id}` for a host, under `Harness · <name>`;
    - `InboxTarget::Manager(Slot)`;
    - the manager row's menu gains Open Manager Thread (the row's labels hold `role: manager`);
      `with_seat_menu` grows to hold both items.
- **File manifest (all Marley crate):** `harness.rs`, `harness_hosts.rs`, `harness_seat.rs`,
  `thread_tab.rs`, `rail.rs`; the guide; the scenario. No Zed path is touched, so no
  `zed-touchpoints.md` row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Two roots with a manager each (#688's fixture); `box-2` through the fake `ssh`; Ctrl+Alt+N | 758-01-entries |
| 002 | Cue box-2's manager to post a report | 758-02-needs-you; the notice log line |
| 003 | Click the entry | 758-03-tab; the ssh log's `acp` |
| 004 | Type "hello manager" and Enter | 758-04-sent; `rh thread show` on both roots |
| 005 | Cue a second report with the tab in front | 758-05-in-front; the notice log unchanged |
| 003 | Cue the first harness's manager; click its entry | 758-06-first |
| 006 | Show another tab; right-click box-2's manager row → Open Manager Thread | 758-07-row-menu |
| 007, 008, 009 | — | The review of the diff; by-hand steps 5 and 6 |

- **Not reached by a scenario:** a remote project's window, since Zed's SSH transport needs a
  master connection and a matching remote server that a fake `ssh` cannot stand in for; and a real
  second machine. Both go in the by-hand list in the spec, which the Test phase hands to Chad.

### Risks
- A `ConversationView` on a project other than its workspace's. Zed code in the view that reaches
  for the workspace's project, such as opening a tool call's file or a terminal, would reach the
  window's project. The harness windows send text only (rustal-harness D189, `docs/ACP.md:119-123`),
  so nothing should call it. The review reads the view's uses of `workspace` and `project`. If one
  bites, the fallback is the research note's: open the thread in the Default group.
- `session_list` for a custom agent: if Zed's connection does not offer it for this entry, a new tab
  starts with `session/new`, and the Needs you entry's line is all that shows of the record. Checked
  at promotion.
- A seat profile named `Manager` would collide with the manager entries' names. Such a seat's
  thread is refused with a notification naming the reserved word.
- The harness's SSH form has run only on one machine. The by-hand list is its first run against a
  separate host, and any failure there is filed on the harness first.
