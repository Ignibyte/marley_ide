# A thread on a remote seat — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-742-a-thread-on-a-remote-seat.md
- **Pipeline spec:** 742-a-thread-on-a-remote-seat.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #694 (the Manager entry through `rh acp`) and rustal-harness TICKET-111.
  - The harness: a non-manager seat has no thread; it has a fleet record, its retained output (`session_read`) and a message path (`session_send` → `deliver`, `delivery_state`). TICKET-116 decides where its replies come from.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-11)
- **Pre-flight:** no active pipeline; README marker present; no cargo running (the harness's
  TICKET-116 gate finished).
- **Recall, at promotion:**
  - rustal-harness TICKET-116 (`4029e3c`, pushed; gate `.artifacts/gates/ticket-116-dev-1`
    passed): `rh --state ROOT acp --seat NAME`; session id `seat-<instance>-<NAME>`; `initialize`
    names the seat as `agentInfo.title`; `session/new` shows the seat's conversation from then on,
    `session/load` replays it; a prompt waits while the seat is busy, goes once as a delivery, and
    answers `end_turn` with `_meta`; updates arrive between prompts too; no permission requests;
    only Claude Code interface seats (an actor or Codex seat ends the prompt with the reason).
  - A profile-opened session carries the label `profile` = NAME (`profiles.rs:327`); its title is
    `NAME-<8 of the request>`.
  - #694's Manager entry and `assistant::set_entry`: a custom agent server in the settings'
    in-memory defaults; `agents::thread_agents` lists every external agent, so seat entries must be
    filtered out of the picker.
  - `thread_tab`: `start_thread` checks `is_dir` and makes a hidden worktree; `open_saved` (#737)
    and the restore (#736) make one for the first folder when `needs_worktree`.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.

### Design
- **`harness_seat.rs`:**
  - `seat_entry(slot, profile)`: the name `<profile> · <host>` (or `· harness`), and the
    command `seat_command_of(slot)` + `acp --seat <profile>`, the words quoted when they cross the
    host's shell; `ensure_seat_entry(slot, profile, cx) -> AgentId` puts it in the settings'
    in-memory `agent_servers` when it is missing or differs.
  - `seat_of_entry(agent_id, cx) -> Option<(Slot, String)>`: the slot and profile a name means,
    when its harness takes seats.
  - `start_agent_on` gains the view to open: `SeatView::Terminal` (#741) or `SeatView::Thread`,
    which takes `seat start`'s `profile`, makes the entry and calls `thread_tab::start_on_seat`.
  - `open_seat_thread(workspace, slot, profile, window, cx)`: an open tab on that entry comes
    forward, else a new one starts (Open as Thread).
- **`thread_tab.rs`:** `start_on_seat(workspace, agent, folder, window, cx)`: a `ThreadSpec` on the
  remote folder with `own_folders`, no `is_dir` check and no worktree, opened one tick later so the
  agent server store has the new entry. `open_saved` and the restore skip the worktree for a seat
  entry and make the entry again first.
- **`agents.rs`:** `Start::seat()` gives the agent kind and the view: a seat-capable CLI →
  Terminal; the Claude Agent thread (`claude-acp`) → Thread. `thread_agents` leaves seat entries out.
- **`rail.rs`:** a right-click on a harness row whose session has a `profile` label opens a menu
  with **Open as Thread**, while its harness takes writes.
- **File manifest (all Marley crate):** `harness_seat.rs`, `thread_tab.rs`, `agents.rs`, `rail.rs`;
  the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `box-2` through #740's fake `ssh`; a stand-in `rh` whose `seat add` writes an `actor` profile and runs the real `seat start`, and whose `acp --seat NAME` is a scripted ACP agent answering as TICKET-116 documents. Ctrl+Alt+N → Claude Agent → On box-2… → `/srv/web` | 742-01-thread, the ssh log's `acp --seat claude-web` |
| 002 | "hello" in the tab | 742-02-reply: "Seat claude-web: hello" |
| 003 | A seat `claude-other` started in setup; right-click its row → Open as Thread | 742-03-open-as-thread, the ssh log's `acp --seat claude-other` |
| 004 | — | The review of the diff |

- **Not reached:** the real window on a real Claude Code seat: TICKET-116's own suite runs it on
  the pinned Claude Code and the scripted Messages API, and `rh claude signin` needs that binary.

### Risks
- Zed's custom agent server may want an absolute command; `ssh` is found on the PATH for the MCP
  connection (#740), and the first run shows whether the agent server store does the same.

## Phase 2 — Code (2026-10-11)
- **Built:**
  - `harness_seat.rs`: `SeatView { Terminal, Thread }`; the `SeatEntries` global (each entry's
    command, and the folder New Agent gave a seat); `seat_entry_name` (`<profile> · <harness>`,
    `· the harness` for the first); `seat_of_entry`; `ensure_seat_entry` (the settings' in-memory
    `agent_servers`, set only when the command changed); `open_seat_thread` (an open tab on the
    entry comes forward, else a thread starts under the seat's folder, else the harness's root);
    `start_agent_on` takes the view and, for a thread, opens it from `seat start`'s `profile`.
  - `harness.rs`: `root_of(slot)`; `harness_hosts.rs`: `HarnessHost::state()`.
  - `thread_tab.rs`: `activate_agent`; `start_on_seat` (no `is_dir`, no worktree, one tick later);
    `open_saved` and the restore make a seat entry again and skip the worktree for it.
  - `agents.rs`: `Start::seat()` (CLI → Terminal, Claude Agent's thread `claude-acp` → Thread);
    `thread_agents` leaves seat entries out.
  - `rail.rs`: `with_seat_menu`: a seat row with a `profile` label, while its harness takes writes,
    gets a right-click menu with **Open as Thread**.
  - The scenario `script/e2e/742-a-thread-on-a-remote-seat.sh`.
- **Deviations:**
  - The first harness's entries end in `· the harness`, not `· harness`: the space keeps them apart
    from a host named `harness` when a name is read back.
  - Open as Thread files a seat Marley did not start under the harness's root on its host: the
    fleet envelope carries no folder, and a thread with no folder is archived by Zed's store.
- **Review:**
  - Re-entrancy: `ensure_seat_entry` changes the settings global inside a workspace update; its
    observers run after the update. The rail's menu entry reads the rail, then updates the workspace
    outside it (PR-claude-701).
  - Errors reach the UI: no reachable harness, no `profile` in `seat start`'s answer, and no root
    each end in a notification.
  - Restore (#736) and the Threads page (#737) make the entry again from its name, so a seat's
    thread opens there without a worktree; out of scope before, it came with the naming.
- **Checklist:** harness_seat.rs ✓, thread_tab.rs ✓, agents.rs ✓, rail.rs ✓, harness.rs ✓,
  harness_hosts.rs ✓, scenario ✓; the guide at Complete.
- **Gate:** `script/gates.sh --diff` GREEN, 17 passed.

## Phase 3 — Test (2026-10-11)
- **Scenario:** `script/e2e/742-a-thread-on-a-remote-seat.sh` (sway). A runtime of the built
  `rh` on a scratch root is `box-2` through #740's fake `ssh`; the stand-in `rh` writes an actor
  profile on `seat add` and runs the real `seat start`, so each seat row carries its `profile`
  label; its `acp --seat NAME` is a scripted window answering "Seat NAME: <prompt>" as rustal-harness
  TICKET-116 documents. `claude-other` is started in setup.
- **Runs:**
  1. The thread tab opened, but its agent failed: "ssh: Could not resolve hostname box-2". Zed
     starts an agent server in the project's shell environment, so it ran the system `ssh`, not the
     one Marley's own search path finds and follows the harness with. Fixed: `ensure_seat_entry`
     resolves a bare program on Marley's search path. Gate re-run GREEN (17 passed).
  2. Every check passes; the shots below.
- **Shots (run 2):**
  - `742-00-where`: Where for Claude Agent, `box-2` typed: **On box-2…**, "a seat, folder next".
  - `742-01-thread` (REQ-001): a thread tab in the repo group whose message box reads "Message
    claude-web · box-2"; the rail lists it under repo as `claude-web · box-2 · idle`, and
    `claude-other` and `claude-web` under HARNESS · box-2. The ssh log holds `-- box-2 …/rh --state
    … acp --seat claude-web`; the seat log holds `seat add claude-web --agent claude --cwd /srv/web`.
  - `742-02-reply` (REQ-002): "hello" sent; the thread shows "Seat claude-web: hello", the tab
    titled "hello". The window's log holds the prompt.
  - `742-03a-menu`: a right-click on `claude-other`'s row shows **Open as Thread**.
  - `742-03-open-as-thread` (REQ-003): a second thread tab, "Message claude-other · box-2", listed
    under repo; the ssh log holds `… acp --seat claude-other`.
- **REQ-004** (by review): the thread's record names `/srv/web` with `own_folders`, and its agent
  `claude-web · box-2`; `start_on_seat`, `open_saved` and the restore make no worktree for a seat.
- **Not reached:** the real window on a real Claude Code seat (it needs the pinned Claude Code
  and a declared login; rustal-harness's `scripts/test_acp_seat.py` runs it, gate
  `ticket-116-dev-1` passed); bringing an open seat tab forward a second time (`activate_agent`,
  by review); a seat's tab after a restart and from the Threads page (`seat_of_entry` in the restore
  and `open_saved`, by review).
- **Focus:** 1 Marley window before and after on Hyprland; no rule added.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added: A thread on a harness seat); `docs/marley/guide.md`
  (Where's thread on a seat, Open as Thread under "Several harnesses");
  `docs/marley_architecture/marley_workbench.md` (A thread on a harness seat);
  `docs/marley/workbench-shell.md` (#742's line). No Zed path touched.
- **Knowledge:** F-claude-742-the-seats-agent-ran-the-systems-ssh-001,
  PR-claude-742-an-agent-servers-program-is-resolved-by-marley-001,
  AD-claude-742-a-seats-thread-is-an-agent-server-named-for-the-seat-001.
- **Brain:** no `rusty` MCP server in this repository's sessions, so no consultation to close.
