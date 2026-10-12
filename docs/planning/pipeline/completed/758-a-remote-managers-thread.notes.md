# A remote manager's thread — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-758-a-remote-managers-thread.md
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

### Promotion (2026-10-11)
- **Promoted** from `queued/` to `active/` under the goal "continue until all tickets are
  finished"; the BACKLOG row removed; the ticket `in-progress`, its link on `active/`.
- **Brain:** consultation b65622e5845c4230a2c82aa0f4d428ab; nothing on this seam came back, only
  the due follow-ups.
- **Seams checked again** (an Explore pass at `a8dc7d48ac`). Every seam exists; line numbers moved,
  most in `rail.rs` and `thread_tab.rs`. What changes the design:
  - `manager_in_front` (`harness.rs:441-464`) already counts a thread tab through
    `thread_tab::shows_agent` (652-656), on the active workspace's active item; it is the first
    harness's only. It becomes `manager_in_front(slot)`.
  - `follow_thread` (518-543) and `manager_posted` (547-565) read the first harness's globals
    throughout; `read_thread` (481-510) needs only a `ContextServer`. The follower's call sits at
    972-975, guarded `Slot::Primary`. `sync_manager_entry` (572-609) runs from `change_seats`'
    primary arm (292) and `follow_setting` (651, 674), never for a host, and resolves no bare
    program (the `which` fix is `ensure_seat_entry`'s, `harness_seat.rs:443-447`).
  - `Followed` (`harness_hosts.rs:182-194`) has no cursor or unread fields; a host whose entry and
    grant are unchanged keeps its `Followed` (243-248), so its cursor survives a settings change.
  - `seat_of_entry` (`harness_seat.rs:418-434`) would read `Manager · box-2` as profile `Manager`:
    `thread_agents` would hide the entry and restore would write `acp --seat Manager`. So
    `manager_of_entry` is checked first and `seat_of_entry` refuses the profile `Manager`.
  - `rail.rs`: `InboxTarget::Manager` 958-959 (also matched at 2004 and 2666);
    `open_inbox_entry` 2293-2317; `open_manager_thread` 2321-2352, which starts through
    `thread_tab::start` and joins a local folder; `harness_row` 7422-7526 reads only the `profile`
    label; `with_seat_menu` 7530-7565; `manager_entries` 7749-7769.
  - `thread_tab.rs`: `deserialize` 282-351 and `open_saved` 845-935 make a seat entry again;
    `activate_agent` 676-690; `start_on_seat` 696-720; `ThreadSpec` 938-948 has no session;
    `build_view` 953-1009 takes the panel's connection store and the workspace's project, and
    reads the session from `ThreadMetadataStore` by thread id. `ConversationView::new`'s fourth
    argument, `resume_session_id`, loads a session when the connection `supports_load_session`.
  - **`session_list`** (D5): Zed's ACP connection offers it when the agent advertises
    `sessionCapabilities.list` (`agent_servers/src/acp.rs:999-1013`); `rh acp` advertises it and
    `loadSession: true` (rustal-harness `acp.rs:171-173`). The built `rh` is `4029e3c` (TICKET-116).
  - `agents.rs:167-188`: `thread_agents` filters seat entries only, so `Manager` shows in New
    Agent (`NewAgentPicker::new`, 974) and `Manager · box-2` will too once `seat_of_entry` refuses it.
  - The manager's banner is raised in `harness.rs:557-562`; a click on it does nothing (Out).
  - The scenarios: #688 (the manager fixture, `post.py`, the private notification bus), #741 and
    #742 (the fake `ssh` that runs the rest here; #742's stand-in `rh` speaks `session/list` and
    `session/load`). `rh` is `/srv/stacks/rustal-harness/bin/rh`, a wrapper on `target/debug/rh`.
- **Design changes at promotion:**
  - Manager state per slot: the entries in a map on `Harness` keyed by entry name; the cursor and
    unread records in `Harness` for the first harness and in `Followed` for a host, so a host's go
    with it.
  - **REQ-008 gets a shot.** #445 landed, so a remote project's window is in a scenario's reach:
    #445's fake host (an sshd whose ForceCommand runs in a fake home) and a server staged for the
    debug build's commit. The fake `ssh` for `box-2` passes every other destination to the
    wrapped real `ssh`. Shot `758-08-remote-window`: a remote project's window, Open Manager
    Thread from the rail, the tab opens there and answers. If the remote window cannot be had,
    REQ-008 falls back to the review and the by-hand list, as written.

## Phase 2 — Code (2026-10-11)
- **Built:** `harness.rs`: `ManagerFollow` (cursor and unread, in `Harness.manager` and in
  `Followed.manager`), `manager_entries` replacing `manager_entry`, `slots`, `connection_of`,
  `manager_follow(_mut)`, `unread_of`, `manager_entry_name`, `manager_of_entry`,
  `has_manager(slot)`, `manager_in_front(slot)`, `follow_thread(slot)` for every slot (the
  `Primary` guard gone), `manager_posted(slot)`, `manager_command` (with the `which` fix),
  `sync_manager_entries`, `ensure_manager_entry`, `manager_server`; `change_seats` syncs for every
  slot; a new source resets the first harness's `ManagerFollow`. `harness_hosts.rs`:
  `Followed.manager`, a sync at the end of `follow_hosts`. `harness_seat.rs`: `seat_of_entry` and
  `ensure_seat_entry` refuse the profile `Manager`, and Open as Thread on such a seat says why.
  `thread_tab.rs`: `ThreadSpec.session`, `ensure_harness_entry`, the `HarnessProject` global and
  `harness_project_for`, `open_manager_thread` and `retained_session`, `build_view` on the harness
  project for harness entries, `start` sending a manager entry to `open_manager_thread`, the
  restore and `open_saved` through `ensure_harness_entry`. `rail.rs`: `InboxTarget::Manager(Slot)`,
  `open_manager_thread(slot)`, `manager_entries` per slot, Open Manager Thread in
  `with_seat_menu`, `shown_workspace`. The guide (the manager per harness, from any window, the
  by-hand list) and the in-app guide (#758's manager, and #445's Install line carried here).
- **Deviations from the design:** the open tab is titled with the entry's name, seen untitled
  ("New Agent Thread") in the first Test run; REQ-008's shot shows a third report streaming in
  rather than a message answered (L-claude-758-a-fixture-managers-turn-never-ends-001).
- **The review:** found the entry a restored tab makes removed by the sync its own write runs
  (F-claude-758-…, PR-claude-758-…), fixed before the first run; the `which` fix now covers the
  first harness's Manager entry too; `retained_session` runs a tick later so the store has an
  entry just made, and falls back to a new session with a log line; the workspace is updated from
  a spawned task in `start`, never inside its own update.
- **Clippy:** `clone_on_ref_ptr` (two), `option_if_let_else`; the rail tests needed `AgentId`
  imported once `rail.rs` stopped using it. `cargo fmt`, then clippy again, clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **Set-up:** a server staged for `a8dc7d48ac` (`script/install-marley --remote-server-only`
  into the scratchpad, 2 min 48 s), the debug build at the same commit.
- **The scenario:** `script/e2e/758-a-remote-managers-thread.sh`, under `compositor sway`: two
  roots of the built `rh` with #688's manager fixture each, `box-2` through #741's fake `ssh`
  (any other destination to the real `ssh`), #688's private notification bus, and #445's fake
  host. Exit 0, eight checks passing. Shots read (the last run):
  - `758-01-entries`: New Agent filtered on "Manager" lists `Manager` and `Manager · box-2`, both
    Thread (REQ-001).
  - `758-02-needs-you`: Needs you 1, "Manager · box-2 · Harness · box-2", "Report: box2's report
    post1"; the banner log holds `Marley|Manager · box-2: Report|box2's report post1` (REQ-002).
  - `758-03-tab`: a `Manager · box-2` tab with the report loaded from the retained thread, the
    entry gone; the ssh log holds `-- box-2 … rh --state … acp` (REQ-003).
  - `758-04-sent`: "hello manager" in the tab; box-2's thread holds it, the first root's does not
    (REQ-004).
  - `758-05-in-front`: "box2's report post2" streamed into the tab; still one box-2 banner
    (REQ-005).
  - `758-06-first`: the first harness's report opened a `Manager` tab with "first's report
    post1"; its banner reads `Manager: Report` (REQ-003).
  - `758-07a-menu` and `758-07-row-menu`: Open Manager Thread on box-2's manager row; the
    `Manager · box-2` tab comes forward (REQ-006).
  - `758-08-remote-window`: a window on the fake host (`127.0.0.1`, `project`, Restricted Mode)
    with the `Manager · box-2` tab, its thread loaded and "box2's report post3" streamed in; one
    box-2 banner still (REQ-008). The log shows three harness threads built on Marley's harness
    project.
- **Fixes on the way:** the entry's place (y 116); manager tabs titled by entry; the remote
  step's message replaced by a third report after "a prompt is already running"; the menu shot
  before the row's tooltip.
- **What no scenario shows:** a restore (REQ-007) and an entry leaving with its manager or its
  harness (REQ-009), read in the diff; a real second machine: the guide's "Try it on a real host"
  list, for Chad.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #758); the guide (notices opening a tab, Open Manager
  Thread, a manager on each harness, threads from any window, "Try it on a real host"); the in-app
  guide's manager article, and #445's Install line; `marley_workbench.md` (a new section, and the
  #688, #694 and #740 notes pointing to it). No Zed path touched.
- **Knowledge appended:**
  AD-claude-758-every-followed-harness-gets-its-manager-and-harness-threads-build-on-marleys-own-project-001;
  F-claude-758-an-entry-made-for-a-saved-thread-was-removed-by-the-sync-its-own-write-ran-001;
  PR-claude-758-a-defaults-write-reenters-its-own-reconcile-001;
  L-claude-758-a-fixture-managers-turn-never-ends-001;
  L-claude-758-a-rail-rows-tooltip-covers-its-right-click-menu-in-a-shot-001.
- **Brain:** consultation b65622e5845c4230a2c82aa0f4d428ab closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. The by-hand list on
  a real host is Chad's.
