# Ports per project in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-521-ports-per-project.md
- **Pipeline spec:** 521-ports-per-project.spec.md

## Phase 1 — Plan
- **Request:** the Orca survey's item 6 (2026-09-25), "Ports that belong to a project", which the
  survey's list of new tickets places after the structured agent events; Chad asked that day for
  every decided item to be specced. The prompt for this spec: a pure `marley_ports` crate that
  reads `/proc/net/tcp` and `tcp6`, maps socket inodes to pids through `/proc/<pid>/fd`, and gives
  each listener to the project whose root holds the process's cwd; rows under the project in the
  rail with Open in a Browser tab, Copy and Stop; a `ports_list` MCP tool.
- **Classification / tier:** feature. A new Marley crate, the rail's model and view, one MCP
  family, one helper in the Browser tab. One Zed path: the root `Cargo.toml`, whose touchpoints row
  already exists and changes its count. Size M (report 05 §3 item 5).
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: wire names are
    `family_verb`, and tools whose answers are the app's come back deferred and are answered on
    the main thread; `ports_list` follows the terminal family's route.
  - The memory note on `script/clippy` (it runs cargo-shear and typos over the whole repository;
    an unused workspace dependency fails): the new crate's `[workspace.dependencies]` entry must
    have a user in the same change.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a scenario that clicks by
    coordinates breaks silently when a later ticket adds beside what it clicks. Port rows push the
    rail's later rows down; the only earlier scenario that clicks the rail is #500's, on the first
    project's + above any port row, and Test reruns it and reads its log.
  - Brain: no page on ports or dev servers (searched 2026-09-25).
- **Discovery:**
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (30: name, expanded, terminals,
    threads, matched), `Selection` (182), `ProjectRow` (195), `Row` (253: Project, Terminal,
    Thread), `Shown` (314) and `walk` (347), `rail_rows` (500), the step and cycle functions (403
    to 478), `has_attention` (636). The crate is pure and gpui-free, and the workbench renders
    its rows.
  - `crates/marley_workbench/src/rail.rs`: `refresh` (299); `new_browser_tab` (607);
    `render_project_row` (1016); `build_snapshot` (1734), which walks each group's member
    workspaces and reads each member's own root (the comment at 1762: a linked worktree's, not
    the main repository's); `group_names` (1989).
  - `crates/marley_workbench/src/mcp.rs`: `answer` (219) routes `browser_*` to the browser tools
    and the terminal tools by name; `terminal_list` (282) shows the answer shape.
  - `crates/marley_mcp/src/registry.rs`: `Family` (12) with `is_served` (38), `REGISTRY` (78),
    `tools_list` (260), `tool_schemas` (281, exhaustive over `Family`, so a new variant fails to
    compile until its schemas exist). `dispatch.rs`: `tools_call` (133), the permission check
    (153), the deferred arm for the app's families (165).
  - `crates/marley_workbench/src/browser.rs`: `new_tab` (5744: a view, then `open_page_in` with
    `about:blank`); `open_page_in` (2960).
  - `crates/terminal/src/pty_info.rs`: `sysinfo` for the foreground process, `libc::killpg` with
    SIGKILL (154) and SIGTERM (169).
  - Root `Cargo.toml`: members at 140 to 148, `[workspace.dependencies]` Marley entries at 421 to
    428; `libc = "0.2"` (697), `nix = "0.30"` (713), `sysinfo = "0.39"` (873). `Cargo.lock` has no
    `procfs`.
  - Orca: the files named in the spec's prior art, read at `1c2cf120e3`.
  - The live box, read only on 2026-09-25: 68 listening sockets in the two tables, 933 processes,
    2,338 socket links readable (the user's own processes; other users' `fd` folders refuse),
    found in about 25 ms by a `find` over `/proc/*/fd`. Processes of more than a dozen other
    system users run on the box, whose listeners Marley cannot attribute and must not list. The
    Browser's Chromium has the home directory as its working directory.
- **Decisions:** D1 to D7 in the spec.

### Design
- **`marley_ports`** (pure core, IO taking its directory):
  - `pub struct Socket { address: IpAddr, port: u16, inode: u64 }` and
    `parse_proc_net_tcp(text: &str) -> Vec<Socket>`: skip the header line; split on whitespace;
    field 3 must be `0A`; field 1 is `ADDR:PORT` in hex, the address one little-endian `u32` for
    IPv4 and four for IPv6; field 9 is the inode, 0 skipped. Malformed lines are skipped, never a
    panic.
  - `pub struct Listener { address, port, pid: u32, name: String, command: String, cwd: PathBuf }`
    and `scan_in(proc_root: &Path) -> io::Result<Vec<Listener>>`: read `net/tcp` and `net/tcp6`
    (a missing table is empty); for each numeric entry of `proc_root`, read the links of its `fd`
    folder and keep those of the form `socket:[<inode>]` that name a listening inode (an
    unreadable folder is skipped); read `comm`, `cmdline` (NUL-separated, joined with spaces) and
    the `cwd` link for each pid found; one listener per (port, pid).
  - `pub struct ProjectFolders { project: String, folders: Vec<PathBuf> }` and
    `attribute(listeners, projects) -> Vec<(usize, Listener)>`: the deepest folder that is the
    working directory or one of its ancestors, compared by path components; ties go to the first
    project.
  - `connect_host(address) -> String` and `url(listener) -> String` (D5).
- **`Ports`** (`crates/marley_workbench/src/ports.rs`): a global entity with the last scan by
  project, `scan_task`, and a subscriber count. The rail subscribes while it shows; with no
  subscriber there is no timer. Each tick gathers every window's project groups and their member
  workspaces' folders on the main thread, runs `scan_in("/proc")` and `attribute` on the
  background executor, drops Marley's pid (`std::process::id()`) and the listeners whose command
  holds `<data>/browser/`, and notifies only when the rows changed. `stop(project, port, pid)`
  rescans, checks, then `libc::kill(pid, SIGTERM)` in an adapter function with its `// SAFETY:`
  note (gate:13), and answers "stopped", "already gone" or why not; the rail shows a failure as a
  toast. `pub fn list(cx) -> Task<Vec<…>>` rescans for `ports_list`.
- **The rail.** `PortSnapshot { key: (u16, u32), port, name, url, tooltip, matched }`;
  `Selection::Port(u16, u32)`; `Row::Port(PortRow)`, shown when the project is expanded, or when
  the filter matched it; the walk puts ports after threads. The view draws an `IconName::Server`
  icon, `:<port> <name>`, the URL muted, and three icon buttons (`ToolWeb` Open, `Copy`, `Stop`)
  shown on hover and on the selected row; the tooltip carries the command, the working directory
  and the pid. Enter on a selected port row opens it, as Enter on a terminal row shows the
  terminal.
- **Open.** A crate-visible `open_url_in_new_tab(workspace, url, window, cx)` in `browser.rs`: a
  new view, then `open_page_in` with the URL. After #507 the tab opens in the project's own
  Chromium with no further change.
- **The tool.** `ToolSpec { family: Family::Ports, verb: "list", tier: Read }`, served; its input
  schema has no properties, its output one `ports` array of objects (D6). `dispatch` defers it;
  `mcp.rs` answers it from `Ports::list`.
- **File manifest.** New Marley crate: `crates/marley_ports/{Cargo.toml, src/marley_ports.rs}`.
  Marley crates: `crates/marley_rail/src/marley_rail.rs`; `crates/marley_workbench/src/ports.rs`
  (new), `rail.rs`, `browser.rs`, `mcp.rs`, `marley_workbench.rs` (the module),
  `crates/marley_workbench/Cargo.toml`; `crates/marley_mcp/src/registry.rs`, `dispatch.rs`. Zed
  paths: root `Cargo.toml` and `Cargo.lock`. Scripts: `script/e2e/browser-fixture.sh`,
  `script/e2e/521-ports-per-project.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `Cargo.toml` row names ten Marley
  members and `marley_ports` among the workspace dependencies; the `Cargo.lock` row is unchanged
  (generated).

### E2E plan
Setup starts the three servers with `setsid -f` and records their pids and ports (each prints
its port); teardown stops them and the browser unit. The typed server's port and pid come from
`mcp_agent ports`, run before its Stop.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the rail after Marley opens the repository; the project expanded | `521-01-rows`: rows for `web` and `tools` |
| REQ-003 | the same view, with the log's `ss -ltnp` listing `elsewhere`'s port and the browser unit's | `521-01-rows` shows no row for either |
| REQ-008 | the same view: the web server bound to `0.0.0.0` shows `http://127.0.0.1:<port>/`; the stand-in on two addresses shows once | `521-01-rows` |
| REQ-002 | `pointer_to` the web server's row; settle for the tooltip | `521-02-tooltip` |
| REQ-004 | click the project's terminal; type `python3 -m http.server 0`; settle four seconds | `521-03-appeared` |
| REQ-005 | hover the web server's row; click Open | `521-04-opened`: a Browser tab on the directory listing of `web/` |
| REQ-006 | hover the stand-in's row; click Copy; click the terminal; Ctrl+Shift+V | `521-05-copied`: the URL at the prompt |
| REQ-007 | hover the typed server's row; click Stop; settle four seconds | `521-06-stopped`: the block ended, the row gone; the log: `kill -0 <pid>` fails |
| REQ-009 | `mcp_agent ports` before the stop | the log: three listeners with project and folder, no `command` key |

Not reachable by a scenario: another user's listener inside a project folder (the run has no
second user); REQ-003's "another user's" half is taken from the dev box's own listeners of other
users, which the log's `ss -ltnp` shows and the rail leaves out, and from the review of the
unreadable-`fd` path.

### Risks
- Cost on a busy machine: the walk reads every readable `fd` link. It measured about 25 ms here;
  Test logs each scan's duration, and a scan over 500 ms backs off to 30 s (Orca's backoff is
  from one to five minutes after a timeout).
- A pid reused between the scan and Stop: the fresh scan before the signal narrows the window to
  microseconds; Orca accepts the same.
- Servers that daemonize change their working directory (to `/`), which puts them in no
  project. The row is then missing, not wrong; the command-line fallback (Out) would catch some.
- The rail's rows move down by the port rows, so coordinates in later scenarios shift (the lessons
  above); only #500's scenario clicks the rail, above them.
- `/proc/net/tcp` is deprecated in the kernel's document; `sock_diag` is the replacement if it
  goes, behind the same `scan_in`.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
