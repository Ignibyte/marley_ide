---
pipeline_id: ddb46c16-8dab-4b59-9727-0b451b0e54d3
ticket: docs/planning/tickets/open/TICKET-758-a-remote-managers-thread.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: A remote manager's thread
type: feature
slice: the control plane (docs/marley/three-prong-plan.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/completed/694-the-manager-in-the-agent-panel.spec.md
  - docs/planning/pipeline/completed/688-a-notice-for-the-managers-reports.spec.md
  - docs/planning/pipeline/completed/740-one-harness-per-host.spec.md
  - docs/planning/pipeline/completed/742-a-thread-on-a-remote-seat.spec.md
  - docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md
---

## Title
A manager on any followed harness, this machine's or another's, gets its own Agent Panel entry,
its own notices and Needs you entries, and a thread tab that talks to it through the harness. A
seat's or a manager's thread opens from any window, a remote project's included. Chad, 2026-10-11:
"is there a way to do remote agent threads. meaning if manager lives on a remote server and we dont
want to steer them through the cli. Using or not using our harness (preferrable our harness)", then
"ok we can build it. once things are spec and then add the things for the missing manager."

## Scope
### In
- **A Manager entry per harness.** For the first harness and each `marley.harnesses` entry, while
  that harness takes writes and its fleet has a session labelled `role: manager`, an agent server in
  the settings' in-memory defaults:
  - named `Manager` for the first harness (as today) and `Manager · <name>` for the others;
  - running `seat_command_of(slot)` + `acp`, so a host's manager runs as
    `ssh … -- HOST rh --state ROOT acp`;
  - with a bare program resolved on Marley's own search path (PR-claude-742), the first harness's
    included.
  An entry leaves when its harness loses its manager or stops being followed.
- **The follower per harness.** #688's `thread_read {after}` runs on each harness's own connection
  after that harness's fleet poll, with its own cursor and unread records. The first read stays
  quiet, as does a resync.
- **Notices and Needs you name the harness.**
  - The desktop notice reads `Manager · box-2: Report`; the first harness's stays `Manager: Report`.
  - The Needs you entry sits under `Harness · box-2` (or `Harness`) and keeps its record's kind and
    first line.
- **In front.** A manager counts as in front when the active window shows either of:
  - the Agent Panel with that entry selected (as now);
  - a thread tab on that entry, active in its pane.
  Today a Manager thread in a tab does not count (`harness.rs:441-458`).
- **Opening the thread.** A click on a manager's Needs you entry opens a thread tab on that
  harness's entry in the shown group. So does **Open Manager Thread**, a new item in the menu of the
  manager's row in the Harness section. Which tab:
  - an open tab on the entry comes forward;
  - else the newest saved thread on the entry opens;
  - else a new tab opens and loads the harness's whole retained thread (`session/list` gives
    `thread-<instance>-0`, `session/load` replays it), so the record that raised the entry shows.
  #688 opens the Agent Panel instead (`rail.rs:1972-2003`).
- **Saved threads.** Restore (#736) and the Threads page (#737) make a `Manager · <name>` entry again
  when that harness is followed, as they do for seat entries. `seat_of_entry` never reads a manager
  entry as a seat.
- **Threads from any window.** A thread on a harness entry, a seat's (#742) or a manager's, is built
  on a local project with no folders that Marley keeps for harness threads, with its own agent
  connection store. The window's project is not used. A restored tab is built the same way.
- `docs/marley/guide.md`: a manager on each harness; harness threads from a remote project's
  window; the by-hand check below as a short "try it on a real host" list.

### Out (explicitly deferred)
- A manager across roots or hosts: the harness keeps one per root and plans none across them
  (rustal-harness `docs/MANAGER.md:184`, D130).
- Answering a manager's confirmation from Needs you. That stays in the thread, as in #688.
- A click on the desktop notice, which does nothing today (`notifications.rs:256-270`).
- New Agent defaulting to a remote window's harness, and one host picker: the research note's M2
  and M3.
- Marley's own remote server (the research note's M1). Until it lands, the installed Marley opens
  an SSH project only on a host that already holds a matching `zed-remote-server-dev-build`.
- Codex seats in a thread: `rh acp --seat` serves Claude Code seats only (rustal-harness
  `docs/ACP.md:158`).

## Reference (§20)
N/A — Marley-specific, over rustal-harness's `rh acp` (its TICKET-111 manager window and TICKET-116
seat window) and ACP. Upstream Zed's part is kept as #694 and #742 use it:
- the custom agent server (`agent_servers`);
- `ConversationView`;
- `session/list` and `session/load` through the agent connection, as Zed's thread import uses them
  (`crates/agent_ui/src/thread_import.rs:689-807`).

### Prior art
- **The code we ship:**
  - `harness.rs`:
    - `MANAGER_ENTRY` (427);
    - `has_manager` (431-437) and `sync_manager_entry` (566-603), both on the first harness only;
    - `manager_in_front` (441-458), which reads the Agent Panel only;
    - `read_thread` (475-504), `follow_thread` (512-537) and `manager_posted` (541-559);
    - the follower's call, guarded `if *slot == Slot::Primary` (966-969);
    - `seat_command_of` (227-239) and `writes_on_slot` (258-263).
  - `harness_hosts.rs`: `Followed` (181-194), where a host's cursor and unread records go;
    `HarnessHost::base` (75-93).
  - `harness_seat.rs`:
    - `seat_entry_name` (411-414) and `seat_of_entry` (418-434), which splits on the last ` · `;
    - `ensure_seat_entry` (439-481), with its search-path fix at 443-447;
    - `open_seat_thread` (486-522).
  - `rail.rs`:
    - `manager_entries` (7037-7057), which hard-codes the project `Harness`;
    - `InboxTarget::Manager` (813-814) and `open_manager_thread` (1972-2003);
    - `harness_row` (6775-6884) and `with_seat_menu` (6888-6923).
  - `thread_tab.rs`:
    - `build_view` (876-932), on the hosting workspace's project and the Agent Panel's connection
      store;
    - `activate_agent` (601-615), `start_on_seat` (621-645), `open_saved` (768-858) and the restore
      (`deserialize`, 267-336);
    - `ThreadSpec` (861-871), which has no session field.
  - Zed:
    - `AgentConnectionStore::new(project)` (`crates/agent_ui/src/agent_connection_store.rs:76`)
      and `Project::local` (`crates/project/src/project.rs:1232`) are public;
    - a remote project lists only the agents its host names
      (`crates/project/src/agent_server_store.rs:710-786`), and the host receives only the user's
      settings file (`crates/project/src/project_settings.rs:979-999`).
- **The harness:**
  - `rh acp` is the manager's window, and over SSH it is `ssh HOST /path/to/rh --state ROOT acp`
    (`docs/ACP.md:15-23`, D178). Its `initialize` answers `loadSession: true` and the title "Manager
    thread" (`crates/harness-runtime/src/acp.rs:168-178`).
  - `session/list` gives one session, `thread-<instance>-0`; loading it replays the retained thread
    (`acp.rs:190-200`, `217-258`).
  - Confirmations come as `session/request_permission` (`acp.rs:329-356`).
  - `thread_read` is served to every grant (`docs/MCP.md:85`), so each host's follow connection can
    read its thread.
- **The research note** `docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md`, §3c: a
  seat thread opened from a remote window cannot start, found by reading and not yet run.
- **Behavior maps:** none.
- **Published material:** ACP (`session/new`, `session/list`, `session/load`,
  `session/request_permission`).

## UI proof
`script/e2e/758-a-remote-managers-thread.sh`, under `compositor sway`, with #688's notification
server on a private bus. Two scratch roots of the built `rh`:
- the first harness: `marley.harness` is `rh --state ROOT1 mcp --grant write`;
- `box-2`: an entry in `marley.harnesses`, reached through #741's fake `ssh`, which logs its
  arguments and runs the rest here.

Each root has a manager made from #688's fixture. It reports idle, is designated with
`rh manager`, and posts a report through `rh mcp --grant agent`'s `thread_post` whenever the
scenario drops a cue file. `marley.harness_writes` is on.

Shots:
- `758-01-entries`: New Agent's agent list shows `Manager` and `Manager · box-2`.
- `758-02-needs-you`: box-2's manager posts a report. Needs you lists `Report: …` under
  `Harness · box-2`, and the notice log holds `Manager · box-2|…`.
- `758-03-tab`: the entry is clicked. A `Manager · box-2` thread tab opens in the shown group and
  shows the report, and the entry is gone. The ssh log holds `-- box-2 … acp`.
- `758-04-sent`: "hello manager" is typed. box-2's `rh thread show` holds the person's message;
  the first root's does not.
- `758-05-in-front`: a second report arrives while the tab is in front. The tab shows it, and the
  notice log gains no line.
- `758-06-first`: the first harness's manager posts. The notice reads `Manager: Report`, and the
  click opens a `Manager` tab, not the Agent Panel.
- `758-07-row-menu`: right-click on box-2's manager row, then Open Manager Thread. The same tab
  comes forward.

Every harness thread in the run is built on Marley's harness project; the log line at its creation
says so. A remote project's window is out of a scenario's reach: Zed's SSH transport needs its own
master connection and a server built from the same commit, which the fake `ssh` cannot stand in
for. The remote window, and a real second machine, are checked by hand.

**By-hand check on a real host** (Chad; `HOST`, `ROOT` and the `rh` path are his):
1. On `HOST`: `rh` installed, `ROOT` owned by the SSH user with mode 0700, `rh --state ROOT serve`
   running, and a manager: `rh --state ROOT seat add boss --agent claude --role manager --cwd DIR`,
   then `rh --state ROOT seat start boss`.
2. In Marley's settings: `marley.harnesses` names `HOST` with `ssh`, `state` and `rh`;
   `marley.harness_writes` is on. The rail shows `HARNESS · HOST` with `boss`.
3. New Agent lists `Manager · HOST`. Start it: a thread tab opens. Type "hello". On `HOST`,
   `rh --state ROOT thread show` holds it, and the manager's answer appears in the tab.
4. Show another tab and have the manager post a report. A desktop notice reads
   `Manager · HOST: Report`, and Needs you lists it. Click the entry: the tab comes forward with the
   report, and the entry clears.
5. Quit Marley and start it again: the tab comes back and still talks to the manager.
6. Open a project on `HOST` (File → Open Remote; this needs a matching remote server there until
   the research note's M1 lands). In that window, right-click a seat's row → Open as Thread, then
   Open Manager Thread: both tabs open in that window and answer.
7. Cut the network to `HOST` for a minute. The rail marks `HARNESS · HOST` stale; once it is back,
   the tab answers again after a reopen.

## Locked-In Decisions
- **D1:** one entry per harness with a manager. The first harness's entry keeps the name `Manager`,
  because saved Manager threads and #688's and #694's scenarios use it. The others take
  `Manager · <name>`, the form #742 gives seats.
  - Rejected: renaming the first to `Manager · the harness`, which would orphan saved threads.
  - Rejected: one entry with a harness choice. Zed keys a conversation by its agent, so one entry
    cannot hold two harnesses' threads.
- **D2:** a manager is per root. The harness keeps at most one per root and plans none across roots
  or hosts (D130, `docs/MANAGER.md:184`). This settles what #740 deferred: it waited on ROADMAP
  open choice 2, which D130 settled.
- **D3:** each harness's thread is followed on its own connection, after its own fleet poll, as #740
  follows fleets.
  - Rejected: one follower that walks the hosts, where a slow host would delay the rest.
- **D4:** Needs you and Open Manager Thread open a tab, not the Agent Panel, because threads live in
  tabs since #738. In front counts a tab.
- **D5:** with no tab and no saved thread, a new Manager tab loads the harness's whole retained
  thread.
  - Rejected: `session/new`, which would show nothing of the record that raised the entry.
- **D6:** every harness thread, seat or manager, is built on a local project of Marley's own that has
  no folders, from every window. The seat and the manager read no file through Zed (#742 D2), and
  the entry lives in this machine's settings, which that project reads.
  - Rejected: opening the thread in the Default group, the research note's suggestion. It moves the
    person off the project they asked from.
  - Rejected: writing entries into the host's settings file (the research note's e1). That writes
    the user's settings on every host and needs M1.
  - Rejected: a separate path for remote windows only, which no scenario reaches.
- **D7:** no harness change. `ssh HOST rh --state ROOT acp` is the harness's documented form
  (`docs/ACP.md:15-23`, D178).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a followed harness takes writes and its fleet has a session labelled `role: manager`, the agent lists shall show that harness's Manager entry, `Manager` for the first harness and `Manager · <name>` for the others. | Shot 758-01 |
| REQ-002 | WHEN a harness's manager posts a record while its thread is not in front, the system shall show a desktop notice naming that harness's manager and the record's first line, and list a Needs you entry under that harness. | Shot 758-02 and the notice log |
| REQ-003 | WHEN a manager's Needs you entry is clicked, the system shall open or bring forward a thread tab on that harness's Manager entry in the shown group, showing the thread's records, and clear the entry. | Shots 758-03, 758-06 |
| REQ-004 | WHEN a message is sent in a harness's Manager tab, the system shall deliver it to that harness's thread and to no other. | Shot 758-04 and `rh thread show` on both roots |
| REQ-005 | WHILE a harness's Manager tab is active in the active window, the system shall raise no notice or Needs you entry for that manager's records. | Shot 758-05 and the notice log |
| REQ-006 | WHEN Open Manager Thread is chosen on a manager's row, the system shall open or bring forward that harness's Manager tab. | Shot 758-07 |
| REQ-007 | WHEN Marley starts with a saved `Manager · <name>` tab and that harness is followed, the system shall restore the tab and make its entry again. | The review of the diff; by-hand step 5 |
| REQ-008 | WHEN a seat's or a manager's thread is opened from a window whose project is remote, the system shall open it in that window and connect it to its harness. | The review of the diff (local windows take the same path); by-hand step 6 |
| REQ-009 | WHEN a harness loses its manager or stops being followed, the system shall remove its Manager entry and its Needs you entries. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `harness.rs` (entries and the follower per harness, in front), `harness_hosts.rs`
  (a host's cursor and unread records), `harness_seat.rs` (`manager_of_entry`; `seat_of_entry`
  leaves manager entries alone), `thread_tab.rs` (the harness project, a session to load, restore),
  `rail.rs` (entries per harness, Open Manager Thread, a tab on click), the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot; give Chad the by-hand
  list.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
