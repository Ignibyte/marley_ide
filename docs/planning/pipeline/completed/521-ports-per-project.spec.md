---
pipeline_id: 01b43aa7-a84b-4799-9e5a-18cfe59bc18d
ticket: docs/planning/tickets/closed/TICKET-521-ports-per-project.md
status: Phase 4 — Complete PASS
title: "Ports per project in the rail"
type: feature
slice: prong 3 with the rail, and a tool for prong 2; the Orca survey's item 6
references: [docs/orca_architecture/03-browser-and-design-mode.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/orca_architecture/02-worktrees-and-review.md, docs/planning/pipeline/completed/503-terminal-urls-open-in-the-browser.spec.md, docs/planning/pipeline/completed/504-browser-tabs-in-the-rail.spec.md]
---

## Title
Marley finds every TCP listener of the user's processes, the sockets `/proc/net/tcp` and `tcp6`
list and the processes whose `fd` links name them, and gives each to the project whose folder
holds the process's working directory. The rail lists a project's live ports under it with Open
(a Browser tab of the project), Copy and Stop, and a `ports_list` MCP tool gives Marley's agents
the same list. It builds on #503's `marley_browser::ports`, which already reads the tables through
`procfs-core`.

## Scope
### In
- `crates/marley_browser/src/ports.rs`: `listeners_in(proc_root)`: the listening sockets of both
  tables (`procfs-core`'s `TcpNetEntries`, with `inode`), each mapped to a pid through the
  `socket:[<inode>]` links under `<proc_root>/<pid>/fd` (`procfs-core`'s `FDTarget`), skipping
  what cannot be read, with each pid's `comm`, command line and `cwd`, one listener per port and
  pid; `url(listener)` (D5); and `stop_in(proc_root, port, pid)`, which rescans, and sends SIGTERM
  through rustix only when that pid still listens on that port. `listening_ports_in` (#503) stays.
- `crates/marley_workbench/src/ports.rs` (new): the app's `Ports` global, which scans off the main
  thread every three seconds while a rail shows, attributes each listener to the project group
  whose member folder is the deepest one holding its working directory, across every window, and
  leaves out Marley's own listeners (D2); `ports_list`'s fresh scan; Stop.
- `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot::ports`, `PortSnapshot`,
  `Row::Port(PortRow)` after a project's terminals, Browser tabs and threads, `Selection::Port`;
  the walk, `row_shows`, `parent` and `open_row`'s kinds; the tests' exhaustive matches.
- `crates/marley_workbench/src/rail.rs`: `build_snapshot` fills each project's ports from `Ports`,
  and the rail observes `Ports`; a port row shows the port, the process name and the URL, with
  Open, Copy and Stop on hover, and a tooltip with the command line, the working directory and the
  pid; Open goes through #503's `open_url_tab`.
- `crates/marley_mcp/src/registry.rs` and `dispatch.rs`: `Family::Ports`, served, and `ports_list`
  (read tier), deferred to the app; `crates/marley_workbench/src/mcp.rs` answers it off the main
  thread, as the browser tools are answered. Outside clients (#524) do not get it.
- `crates/marley_browser/Cargo.toml`: `rustix` with `process`.
- `script/e2e/browser-fixture.sh`: `mcp_agent ports`. `script/e2e/521-ports-per-project.sh`.

### Out (explicitly deferred)
- The URL a dev server printed: #503 offers it in the terminal's footer.
- Port offsets for worktree agents (#510) and Orca's `*.localhost` label proxy (report 02 §3
  item 7).
- Remote projects' ports (a scan over SSH) and containers' ports (a `docker-proxy` of root's,
  whose fds Marley cannot read).
- Orca's fallback to the command line when the working directory is in no project: it can hand an
  editor's or a tool's listener to any project whose path the command merely names.
- Listeners outside every project, and a Ports panel for the whole machine.
- SIGKILL, or stopping a process group.
- `ports_list` for outside clients: their tools are an explicit list of browser tools (AD-524).

## Reference (§20)
Orca's port discovery and the worktree card's port rows (report 03 §2.3; report 02 §2.4, "Orca
allocates no ports. It discovers them"; `src/renderer/src/components/sidebar/WorktreeCardPorts.tsx`):
a row per live listener under the worktree whose path
holds the listener's working directory, with Open, Copy and Stop, where Stop acts only after a
fresh scan proves the pid still owns the port (`src/main/ports/local-workspace-platform-port-scanner.ts`,
`local-workspace-port-attribution.ts`, `workspace-port-ownership.ts`). Marley reimplements that
behavior in Rust from the design; if Code ports a function body instead, the Marley file carries
Orca's MIT notice and names the Orca file (`docs/orca_architecture/README.md`, "The source, and
the licence"). Upstream Zed: SSH connections take static `port_forwards`
(`crates/remote/src/transport/ssh.rs:143`) and nothing in Zed discovers listeners. Warp: N/A,
Warp lists no ports.

### Prior art
- **Behavior maps and reports.** Report 03 §2.3 (the Linux scan: both tables, the inode to pid
  walk, `comm`, `cmdline` and `cwd`, the deepest worktree path, a timeout backoff, 0.0.0.0 and
  `::` opened as loopback). Report 05 §3 item 5 (rows beside #504's, `ports_list`; scan while the
  rail is visible; Orca scans every 30 s). Report 02 §3 item 7 (discovery matters more than
  allocation, since Vite and Next move to a free port on their own). In Orca's checkout:
  `parseProcNetTcp`, `mapLinuxInodesToPids`, `loadLinuxProcessMetadata`, `pickDeepestMatching`,
  `connectHostForBindHost`, `dedupeRawPorts` and `killWorkspacePort` (the rescan is the
  authorization, Orca's own pid refused, SIGTERM, `ESRCH` counts as stopped).
- **Published material.** The kernel's `/proc/net/tcp` document (hex `address:port` fields, the
  interface marked deprecated in favor of `tcp_diag`); the tables are there on the dev box's
  kernel 7.2, with 72 listening sockets on 2026-09-27.
- **The code we already ship** (re-verified at promotion, after #503, #504, #507, #539 and
  #583). #503's `marley_browser::ports::listening_ports_in` reads both tables through
  `procfs-core` 0.18 (a workspace dependency since #503, at the version `minidump-writer` builds);
  its `TcpNetEntry` carries `uid` and `inode`, and its `FDTarget` parses a `socket:[N]` link, so
  nothing is parsed by hand. #503's `browser::open_url_tab` opens a Browser tab of a workspace at a
  URL, or brings forward the tab already on it. `browser_tools::holding` gives a directory to the
  workspace whose folder is the longest one holding it. `marley_terminal`'s `pty_os.rs` signals
  through `rustix::process::kill_process`; the Marley crates deny `unsafe_code`, so `libc::kill`
  is out. `sysinfo` 0.39 reads a process's `cwd` and arguments (Zed's `terminal::pty_info`) but has
  no socket table. The rail's pure model (`marley_rail`: `ProjectSnapshot` with `browsers` since
  #504, `Row`, `Selection`, the walk) and `rail.rs`'s `build_snapshot`, which walks every member
  workspace of a group. `marley_mcp`'s registry adds a family by a `Family` variant, rows and
  schemas. A walk of the box's `/proc` on 2026-09-27, in Python while a release install ran, took
  108 ms over 1,291 processes and 3,543 socket links. Does a crate we build own the seam?
  `marley_browser::ports` owns the tables; the process walk and the attribution are new, beside it.

## UI proof
UI-AFFECTING (rows in the rail, their buttons, a Browser tab). `script/e2e/521-ports-per-project.sh`
(`compositor sway`: it clicks the rows' buttons). Setup: the offline Chromium; a scratch HOME
whose `.bashrc` is the scenario's; a scratch repository with `web/` (an `index.html`) and
`tools/`; five servers started before Marley: `python3 -m http.server --bind 0.0.0.0` in
`repo/web`, a stand-in in `repo/tools` that listens on `127.0.0.1` and `[::1]` on one port, one in
`$E2E_WORK/elsewhere` (outside the project), one in `repo/web` that names itself `marley`, and
one in `repo/web` whose command line names the profile's `browser/` folder. Steps and shots: the
rail with the project's two rows, the web server's URL on `127.0.0.1`, the stand-in once, nothing
for the other three (`521-01-rows`); the pointer on the web server's row, its tooltip
(`521-02-tooltip`); the rail closed with Ctrl+Alt+J, Marley's reads over nine seconds against
the open rail's (the log: D3's scan stops), a server started meanwhile, and its row once the
rail opens again (`521-07-reopened`); a server typed into the project's terminal, its row
within five seconds (`521-03-appeared`); Copy on the stand-in's row, the URL on the clipboard
(`521-05-copied`);
Stop on the typed server's row, its block ended in the terminal and its row gone
(`521-06-stopped`); Open on the web server's row, a Browser tab on its page (`521-04-opened`),
last, since the tab's row moves the port rows down. The run log carries `ss -ltnp` for the run's
servers, the stand-in agent's `ports_list`, the clipboard, and the stopped pid's absence.

## Locked-In Decisions
- D1: Discovery, not allocation, and the working directory decides: a listener belongs to the
  project group whose member folder is the deepest one holding the process's working directory,
  across every window. A linked worktree's folders belong to its project's group.
- D2: The user's own processes only. A listening socket whose inode no readable `fd` link names
  (another user's, or root's) is dropped. So are Marley's own listeners: its pid, any process
  named `marley` (another Marley's MCP server, a project Chromium's relay, which runs Marley's
  executable), and any process whose command line names Marley's `<data>/browser/` folder (a
  Chromium an earlier build started on a port).
- D3: Scan every three seconds, off the main thread, while a rail shows; at once for `ports_list`
  and before a Stop. A scan over 500 ms backs off to 30 s.
- D4: Stop sends SIGTERM (`rustix::process::kill_process`) to the listener's pid only after a
  fresh scan finds that pid still listening on that port; `ESRCH` means it is gone, which counts as
  stopped. No SIGKILL, no process group, no confirmation: the button is on the row it acts on, and
  the row's tooltip names the process.
- D5: A row's URL is `http://<host>:<port>/`, the host being `127.0.0.1` for `0.0.0.0`, `[::1]`
  for `::`, and the bound address otherwise; `https` for ports 443 and 8443, as Orca guesses.
- D6: `ports_list` returns what the rails show, listener by listener: the project, the folder the
  working directory is in, the address, port, URL, pid, process name (`comm`) and working
  directory. Never the command line, which can carry a token; the rail's tooltip shows it on the
  user's own screen only.
- D7: Port rows sit after the project's terminals, Browser tabs and threads, in port order. The
  rail's cycle actions leave them out, since a port is not a place to switch to; Enter on a
  selected port row opens it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a process of the user listens on a TCP port and its working directory lies inside a project's folder, the system shall show a row under that project in the rail with the port, the process name and the URL. | Shot `521-01-rows` |
| REQ-002 | WHEN the pointer rests on a port row, the system shall show the process's command line, working directory and pid in a tooltip. | Shot `521-02-tooltip` |
| REQ-003 | WHERE a listener's working directory lies in no project's folder, or its process is another user's or Marley's own, the system shall show no row for it. | Shot `521-01-rows` against the log's `ss -ltnp` |
| REQ-004 | WHEN a process in a project starts listening, the system shall show its row within five seconds. | Shot `521-03-appeared` |
| REQ-005 | WHEN the user clicks a port row's Open, the system shall open a Browser tab in that project on the row's URL. | Shot `521-04-opened` |
| REQ-006 | WHEN the user clicks a port row's Copy, the system shall put the row's URL on the clipboard. | Shot `521-05-copied` |
| REQ-007 | WHEN the user clicks a port row's Stop, the system shall send SIGTERM to the process only if a fresh scan shows it still listening on that port in that project, and remove the row within five seconds. | Shot `521-06-stopped`; the log: the pid is gone |
| REQ-008 | WHEN a server binds `0.0.0.0` or `::`, the system shall give its row a URL on `127.0.0.1` or `[::1]`, and show one row for a server listening on two addresses of one port. | Shot `521-01-rows` |
| REQ-009 | WHEN an agent calls `ports_list`, the system shall return each listener the rail shows, with its project, folder, address, port, URL, pid, process name and working directory, and no command line. | The run log: `mcp_agent ports` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** the scan in `marley_browser::ports`, `Ports`, the rail's model and rows, the tool;
  fmt and clippy clean on every touched crate; a review of the diff against each REQ.
- **P3 Test:** write and run the scenario and read every shot; rerun `500-browser-from-the-rail.sh`
  and `504-browser-tabs-in-the-rail.sh`, whose clicks sit in the rail; the golden set;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_browser.md`, `marley_rail.md` and
  `marley_workbench.md`; the ledger capture; close the ticket, archive, commit.
