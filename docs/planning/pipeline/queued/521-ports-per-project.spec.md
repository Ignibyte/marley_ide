---
pipeline_id: 01b43aa7-a84b-4799-9e5a-18cfe59bc18d
ticket: docs/planning/tickets/open/TICKET-521-ports-per-project.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Ports per project in the rail"
type: feature
slice: prong 3 with the rail, and a tool for prong 2; the Orca survey's item 6
references: [docs/orca_architecture/03-browser-and-design-mode.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/orca_architecture/02-worktrees-and-review.md]
---

## Title
A pure `marley_ports` crate finds every TCP listener of the user's processes (`/proc/net/tcp`
and `tcp6`, socket inodes mapped to pids through `/proc/<pid>/fd`) and gives each to the project
whose folder holds the process's working directory. The rail lists a project's live ports under
it with Open (a Browser tab of the project), Copy and Stop, and the `ports_list` MCP tool gives
agents the same list.

## Scope
### In
- `crates/marley_ports` (new, Marley-owned, `MIT OR Apache-2.0`, rustal's lint table, library
  root `src/marley_ports.rs`): `parse_proc_net_tcp` (the rows in state `0A`, LISTEN, of either
  table, with the local address, port and inode); `scan_in(proc_root)` (both tables; each inode
  mapped to a pid through the `socket:[<inode>]` links under `<proc_root>/<pid>/fd`, skipping
  what it cannot read; each pid's `comm`, `cmdline` and `cwd`); `attribute` (a listener goes to
  the project whose folder is the deepest one holding its working directory); `connect_host`
  (`0.0.0.0` opens as `127.0.0.1`, `::` as `::1`, any other address as bound); one listener per
  port and pid, so a server bound on two addresses of one port is one row.
- `crates/marley_workbench/src/ports.rs` (new): the app's `Ports`, which scans off the main thread
  every three seconds while a window in the Marley layout shows its rail, against the folders of
  every project in every window; leaves out Marley's own pid and the Chromium units whose command
  line names Marley's `browser/` folder; and stops a listener after a fresh scan proves it.
- `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot::ports`, `PortSnapshot`,
  `Row::Port(PortRow)` after a project's terminals and threads, `Selection::Port`; the filter
  matches a port row by its port and process name; the keyboard steps through port rows.
- `crates/marley_workbench/src/rail.rs`: `build_snapshot` fills each project's ports; a port row
  shows the port, the process name and the URL, with Open, Copy and Stop, and a tooltip with the
  command line, the working directory and the pid.
- `crates/marley_workbench/src/browser.rs`: a crate-visible "new Browser tab at a URL" for a
  workspace, the new-tab path with the URL in place of `about:blank` (#503 may add the same; take
  whichever lands first).
- `crates/marley_mcp/src/registry.rs` and `dispatch.rs`: `Family::Ports`, served, and
  `ports_list` (read tier), deferred to the app like the terminal family;
  `crates/marley_workbench/src/mcp.rs` answers it from a fresh scan.
- `Cargo.toml`: the member and its `[workspace.dependencies]` entry, and its row in
  `docs/marley/zed-touchpoints.md` updated (ten Marley members).
- `script/e2e/browser-fixture.sh`: `mcp_agent ports`. `script/e2e/521-ports-per-project.sh`.

### Out (explicitly deferred)
- The URL a dev server printed (Orca's advertised-URL watcher, report 03 §2.3): #503's.
- Port offsets for worktree agents (#510) and Orca's `*.localhost` label proxy (report 02 §3
  item 7).
- Remote projects' ports (a scan over SSH, report 04 §3.2 item 6) and containers' ports (a
  `docker-proxy` of root's, whose fds Marley cannot read).
- Orca's fallback to the command line when the working directory is in no project: it can hand an
  editor's or a tool's listener to any project whose path the command merely names.
- Listeners outside every project in the rail or the tool, and a Ports panel for the whole
  machine.
- SIGKILL, or stopping a process group.

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
  walk, `comm`, `cmdline` and `cwd`, the deepest worktree path, a timeout backoff from one to
  five minutes, 0.0.0.0 and `::` opened as loopback; about 2,400 lines in Orca). Report 05 §3
  item 5 (a pure `marley_ports`, rows beside #504's, `ports_list`; scan while the rail is
  visible; Orca scans every 30 s). Report 02 §3 item 7 (discovery matters more than allocation,
  since Vite and Next move to a free port on their own; two apps on localhost ports share
  cookies, which #507 answers per project). In Orca's checkout: `parseProcNetTcp` (state `0A`,
  the address in field 1, the inode in field 9, inode 0 skipped), `mapLinuxInodesToPids` and
  `loadLinuxProcessMetadata` in the scanner; `pickDeepestMatching`, `isSameOrDescendant`, the
  command-line fallback and the port-to-protocol guess in the attribution file;
  `parseProcAddress` (IPv4 as one little-endian word, IPv6 as four), `connectHostForBindHost`
  and `dedupeRawPorts` in `local-workspace-port-address.ts`; `killWorkspacePort` in the
  ownership file (the rescan is the authorization, a port outside a workspace and Orca's own pid
  are refused, SIGTERM, `ESRCH` counts as stopped); `WORKSPACE_PORT_SCAN_INTERVAL_MS = 30_000`,
  paused while the window is hidden (`src/renderer/src/components/ports/WorkspacePortScanner.tsx:20`).
- **Published material.** The kernel's `/proc/net/tcp` document
  (kernel.org, networking/proc_net_tcp, read 2026-09-25): hex `address:port` fields, listening
  sockets listed first, and the interface marked deprecated in favor of `tcp_diag` (the netlink
  socket diagnostics `ss` reads). The text tables are still there on the dev box's kernel 7.2,
  where they list 68 listening sockets, and Orca reads them; a netlink reader is the swap if they
  ever go.
- **The code we already ship.** Zed's `terminal::pty_info` reads a process's working directory
  and arguments through `sysinfo` 0.39 and signals with `libc::killpg`
  (`crates/terminal/src/pty_info.rs:154`, `169`); `sysinfo` has no socket table, so it cannot map
  a listener to its process, and no crate in `Cargo.lock` reads `/proc/net/tcp` (`procfs` is not
  there). `libc` and `nix` 0.30 are workspace dependencies. The rail's pure model (`marley_rail`:
  `Row`, `Selection`, `rail_rows`, `marley_rail.rs:253`, `182`, `500`) and its snapshot builder
  (`rail.rs:1734`) already walk every member workspace of a group, a linked worktree's included,
  each against its own root. `marley_mcp`'s registry adds a family by a `Family` variant, a
  registry row and its schemas (`registry.rs:12`, `38`, `281`). The Browser tab's new-tab path
  (`browser.rs:5744`) and `open_page_in` (`browser.rs:2960`) open a page for a tab. Does a crate
  we build own the seam? No crate owns listener discovery; the rail, the MCP registry and the
  Browser tab own everything around it.

## UI proof
UI-AFFECTING (rows in the rail, their buttons, a Browser tab). `script/e2e/521-ports-per-project.sh`
(`compositor sway`: it clicks the rows' buttons). Setup: the offline Chromium; a scratch HOME
whose `.bashrc` is the scenario's; a scratch repository with `web/` and `tools/`; three servers
started before Marley: `python3 -m http.server --bind 0.0.0.0 0` in `repo/web`, the same in
`$E2E_WORK/elsewhere` (outside the project), and a stand-in in `repo/tools` that listens on
`127.0.0.1` and `[::1]` on one port. Steps and shots: the rail with the project's rows, the web
server's URL on `127.0.0.1`, the stand-in once, nothing for `elsewhere` (`521-01-rows`); the
pointer on the web server's row, its tooltip (`521-02-tooltip`); a server typed into the
project's terminal (`python3 -m http.server 0`), its row within five seconds
(`521-03-appeared`); Open on the web server's row, a Browser tab on its directory listing
(`521-04-opened`); Copy on the stand-in's row, then Ctrl+Shift+V in the terminal, the URL at the
prompt (`521-05-copied`); Stop on the typed server's row, its block ended in the terminal and its
row gone (`521-06-stopped`). The run log carries `ss -ltnp` for the run's ports, the stand-in
agent's `ports_list`, and the stopped pid's absence.

## Locked-In Decisions
- D1: Discovery, not allocation, and the working directory decides: a listener belongs to the
  project whose folder is the deepest one holding the process's working directory (the rule the
  ticket's request names, and Orca's first rule). A project's folders are those of every member
  workspace of its group, so a server started in a linked worktree lands under its project.
- D2: The user's own processes only. A listening socket whose inode no readable `fd` link names
  (another user's, or root's) is dropped, and so are Marley's own pid and its Chromium units
  (their command line names `<data>/browser/`), whose DevTools port would otherwise land under a
  project opened at the home directory, the units' working directory.
- D3: Scan every three seconds, off the main thread, while a window in the Marley layout shows
  its rail, and at once for `ports_list` and before a Stop. Orca's 30 s interval pays for
  forking `lsof` on macOS; here the walk is file reads, about 25 ms on the dev box on 2026-09-25
  (2,338 socket links across 933 processes, 68 listening sockets).
- D4: Stop sends SIGTERM to the listener's pid only after a fresh scan finds that pid still
  listening on that port in that project; `ESRCH` means it is already gone, which counts as
  stopped. No SIGKILL, no process group, no confirmation: the button is on the row it acts on, and
  its tooltip names the process.
- D5: A row's URL is `http://<host>:<port>/`, the host being `127.0.0.1` for `0.0.0.0`, `[::1]`
  for `::`, and the bound address otherwise; `https` for ports 443 and 8443, as Orca guesses.
- D6: `ports_list` returns what the rail shows, listener by listener: the project, the folder the
  working directory is in, the address, port, URL, pid, process name (`comm`) and working
  directory. Never the command line, which can carry a token; the rail's tooltip shows it on the
  user's own screen only.
- D7: Port rows sit after the project's terminals and threads (and after #504's Browser tab rows
  once that lands), in port order.

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
- **P1 Plan:** this spec; the design and the test plan in the notes. On promotion, check what #503
  and #504 left in `browser.rs` and the rail (a URL-to-tab helper, a Browser tab row kind).
- **P2 Code:** the crate, `Ports`, the rail's model and rows, the tool, the manifest and its
  touchpoints row; fmt and clippy clean (the new crate joins `script/clippy`'s cargo-shear and
  typos run); a review of the diff against each REQ and §14's adapter rule for the signal.
- **P3 Test:** write and run the scenario and read every shot; rerun `500-browser-from-the-rail.sh`,
  whose clicks sit above the new rows, and read its log; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_ports.md` (new) and
  `marley_rail.md`; the plan's slice status; the ledger capture; close the ticket, archive,
  commit.
