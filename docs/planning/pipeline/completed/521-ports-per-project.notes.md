# Ports per project in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-521-ports-per-project.md
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

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #539's release install, so no cargo ran); recall ✓; promote ✓ (the pair to `active/`, the
  backlog row removed, the ticket in-progress); the seams re-verified ✓ (Explore); the scan's cost
  measured ✓; spec and design updated ✓.
- **Recall, added:** AD-claude-503-marley-routes-a-terminals-urls-and-offers-a-listening-dev-server-001
  (#503 reads `/proc/net/tcp` through `procfs-core` and rejected parsing it by hand);
  AD-claude-504-browser-tabs-are-rows-of-their-project-in-the-rail-001 (Browser tab rows after a
  project's terminals, before its threads; the hub tells the rail through events, never its
  notify); F-claude-442-a-close-deferred-on-the-rail-ran-inside-the-rails-update-001 (the rail's
  deferred work runs through `window.defer`); F-claude-488-a-signal-skipped-the-e2e-cleanup-001
  (a scenario's background servers are stopped through the harness's traps). The brain
  (consultation d9ef4cc2314d447189c8d8eb257c61aa): nothing on listening ports.
- **What moved since the queue** (Explore, file:line in its report): #503 added
  `marley_browser::ports::listening_ports_in` on `procfs-core` 0.18, whose `TcpNetEntry` has
  `uid` and `inode` and whose `FDTarget` parses `socket:[N]`; `browser::open_url_tab` (7931) opens
  or brings forward a tab at a URL; `browser_tools::holding` (171-188) is the deepest-folder rule.
  The Marley crates deny `unsafe_code`, so the plan's `libc::kill` would not compile;
  `marley_terminal/src/pty_os.rs:115-121` signals through `rustix::process::kill_process`. #583 put
  Chromium on its pipe: the listener to leave out is the relay's (comm `marley`, argv
  `--browser-relay`, working directory `$HOME`), and another Marley's MCP server is another pid
  named `marley`. The rail: `ProjectSnapshot` gained `browsers` (#504); `Row` and `Selection` sit at
  312 and 212; the walk chains terminals, Browser tabs and threads (434); `build_snapshot` (2064)
  reads each member's first root only, so ports need every root; the rail refreshes on events
  alone, and `observe_global_in` is the pattern a `Ports` global follows. The MCP registry: `Family`
  (13-22), `is_served`, `tool_schemas` (exhaustive), the deferred arm (`dispatch.rs:188`),
  `mcp.rs::answer` (397); `CLIENT_READ_TOOLS` is an explicit list, so `ports_list` stays with
  Marley's own agents. Two scenarios click rail rows by position: 500 and 504. Unit tests are
  built, never run: new variants must compile in the tests' exhaustive matches.
- **The cost, measured again** (2026-09-27, Python, during the release install's golden set): 72
  listening sockets, 1,291 processes, 3,543 socket links, 32 of the listening sockets owned by
  processes whose `fd` could be read; the walk took 108 ms. Rust reads the same links; D3's
  backoff stays.
- **Decisions changed:** no new crate (the scan joins `marley_browser::ports`, beside #503's
  reader), Stop through rustix, D2 widened to Marley's other processes, D7 names the Browser tab
  rows and leaves ports out of the cycle actions.

### Design
- **The scan** (`marley_browser::ports`, IO under a directory it is given):
  - `pub struct Listener { pub address: SocketAddr, pub pid: u32, pub name: String, pub command:
    String, pub cwd: PathBuf }`.
  - `listeners_in(proc_root: &Path) -> Result<Vec<Listener>>`: `net/tcp` and `net/tcp6` through
    `TcpNetEntries::from_file` (as `listening_ports_in` reads them), the `Listen` entries with a
    non-zero inode, by inode; each numeric entry of `proc_root` whose `fd` folder reads, its links
    parsed as `FDTarget`, a `Socket(inode)` in the map giving (pid, address); for each such pid
    `comm` (trimmed), `cmdline` (NUL-separated, joined with spaces) and the `cwd` link; one
    listener per (port, pid), the first address kept. An unreadable process, a vanished one or a
    malformed line is skipped; only both tables failing is an error.
  - `url(address) -> String`: D5.
  - `stop_in(proc_root, port, pid) -> Result<Stopped>` with `Stopped::{Sent, Gone, NotListening}`:
    a fresh `listeners_in`, then `rustix::process::kill_process(pid, Signal::TERM)` only when
    (port, pid) is among them; `Errno::SRCH` is `Gone`.
- **`Ports`** (`crates/marley_workbench/src/ports.rs`): a global with the last attributed scan
  (`HashMap<ProjectGroupKey, Vec<PortListener>>`), a watcher count, and the running scan task.
  `Ports::watch(cx)` returns a guard the rail keeps while it shows; the first starts a loop that
  scans, sleeps three seconds (30 after a scan over 500 ms) and runs again while any guard lives.
  Each scan gathers, on the main thread, every window's project groups and each member
  workspace's root paths, then runs `listeners_in("/proc")` and the attribution off it. The
  attribution is the deepest folder holding the working directory, compared by components. It
  drops D2's listeners (`std::process::id()`, `name == "marley"`, a command naming
  `paths::data_dir()/browser/`) and notifies only when the result changed. `Ports::list(cx)` scans
  at once for `ports_list`, and `Ports::stop(group, port, pid, cx)` runs `stop_in` off the main
  thread and answers with a toast on failure.
- **The rail.** `PortSnapshot { key: (u16, u32), port, name, url, tooltip, matched }`,
  `ProjectSnapshot::ports`, `Selection::Port(u16, u32)`, `Row::Port(PortRow)`, `Shown::Port`,
  chained after threads in the walk; `parent` and `open_row` gain their arms; `cycle_row` leaves
  ports out. `build_snapshot` reads `Ports` for the group's key (every root of every member counts,
  through the attribution). The row draws `IconName::Server`, `:<port> <name>`, the URL muted as its
  second line, and three icon buttons shown on hover: Open (`ToolWeb`, through
  `browser::open_url_tab` in the group's workspace), Copy (`Copy`, the URL on the clipboard) and
  Stop (`Stop`); a tooltip on the row with the command line, the working directory and the pid.
  The rail keeps a `Ports::watch` guard while it is open and observes the global
  (`observe_global_in`), rebuilding when the ports changed.
- **The tool.** `Family::Ports` (served) and `ToolSpec { family: Ports, verb: "list", tier: Read }`
  with a no-argument input schema and an output of one `ports` array (D6); `dispatch` defers it;
  `mcp.rs::answer` routes `ports_list` to a task that awaits `Ports::list` and answers, as
  `browser_tools::answer` does.
- **File manifest.** Marley crates: `crates/marley_browser/src/ports.rs`,
  `crates/marley_browser/Cargo.toml` (`rustix` with `process`); `crates/marley_workbench/src/ports.rs`
  (new), `rail.rs`, `mcp.rs`, `marley_workbench.rs` (the module), `browser_tools.rs` (none, unless
  `holding`'s rule is shared); `crates/marley_rail/src/marley_rail.rs` and its tests;
  `crates/marley_mcp/src/registry.rs` and `dispatch.rs`. No Zed path (the workspace already lists
  `rustix` and `procfs-core`), so no ledger row. Scripts: `script/e2e/browser-fixture.sh`,
  `script/e2e/521-ports-per-project.sh`, `script/e2e/golden`.

### E2E plan
`compositor sway` (it clicks the rows' buttons). Setup: `offline_chromium`; `write_mcp_agent`; a
scratch repository with `web/` and `tools/`, and `$E2E_WORK/elsewhere` outside it. Before Marley
starts, the scenario starts its servers in the background, each with its pid kept for teardown:
`python3 -m http.server --bind 0.0.0.0 0` in `repo/web` and the same in `elsewhere`; a stand-in in
`repo/tools` that listens on `127.0.0.1` and `[::1]` on one port; a server in `repo/web` run through
a copy of `python3` named `marley` (D2's name rule); and one in `repo/web` whose command line names
the profile's `browser/` folder (D2's folder rule). Each prints its port to a file the scenario
reads. The typed server's port and pid come from `mcp_agent ports`, run before its Stop.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the rail after Marley opens the repository; the project expanded | `521-01-rows`: rows for `web` and `tools` |
| REQ-003 | the same view, with the log's `ss -ltnp` listing `elsewhere`'s port, the two D2 servers' and the relay's | `521-01-rows`: no row for any of them |
| REQ-008 | the same view: the web server bound to `0.0.0.0` shows `http://127.0.0.1:<port>/`; the stand-in on two addresses shows once | `521-01-rows` |
| REQ-002 | `pointer_to` the web server's row; settle for the tooltip | `521-02-tooltip` |
| REQ-004 | click the project's terminal; type `python3 -m http.server 0`; settle four seconds | `521-03-appeared` |
| REQ-005 | hover the web server's row; click Open | `521-04-opened`: a Browser tab on the directory listing of `web/` |
| REQ-006 | hover the stand-in's row; click Copy; read the clipboard under the sway | `521-05-copied`; the log: the clipboard holds the stand-in's URL |
| REQ-007 | hover the typed server's row; click Stop; settle four seconds | `521-06-stopped`: the block ended, the row gone; the log: `kill -0 <pid>` fails |
| REQ-009 | `mcp_agent ports` before the stop | the log: the listeners with project and folder, no `command` key |

Then 500's and 504's scenarios run again (their clicks sit in the rail, above the new rows), and
the golden set with 521 added. Not reachable by a scenario: another user's listener inside a
project folder (the run has no second user); REQ-003's "another user's" half rests on the review
of the unreadable-`fd` path and on the dev box's own listeners of other users in the log.

### Risks
- Cost on a busy machine: the walk reads every readable `fd` link, 108 ms in Python on
  2026-09-27 with 1,291 processes. Test logs a scan's duration; D3 backs off to 30 s past 500 ms.
- A pid reused between the scan and Stop: the fresh scan before the signal narrows the window;
  Orca accepts the same.
- Servers that daemonize change their working directory (to `/`), which puts them in no
  project. The row is then missing, not wrong.
- The rail's rows move down by the port rows, so coordinates in 500's and 504's scenarios can
  shift; both run again at Test.
- `/proc/net/tcp` is deprecated in the kernel's document; `sock_diag` is the replacement if it
  goes, behind the same `listeners_in`.
- D2's name rule leaves out a user's own server named `marley`, which only someone working on
  Marley would run.

## Phase 2 — Code
- **Checklist** (no task tool): `marley_browser::ports` ✓; `marley_rail` ✓; `marley_mcp` ✓;
  `marley_workbench` (`ports.rs`, `rail.rs`, `mcp.rs`, the module) ✓; check, fmt and clippy ✓;
  the review ✓. No Zed path.
- **Built.**
  - `marley_browser::ports`: #503's table reading is shared (`listening_entries_in`), and
    `listening_ports_in` keeps its behavior. `Listener { address, pid, name, command, cwd }`.
    `listeners_in(proc_root)` maps each listening inode to the pids whose `fd` links name it
    (`FDTarget::Socket`), reads `comm`, `cmdline` and the `cwd` link, and keeps one listener per
    (port, pid), the addresses sorted so the same one wins each scan. A process whose `fd` or
    `cwd` cannot be read is left out. `url(address)` gives D5's URL, an IPv4-mapped address taken
    as IPv4. `stop_in(proc_root, port, pid)` rescans and sends `Signal::TERM` through
    `rustix::process::kill_process` only when (port, pid) still listens, with `Errno::SRCH` as
    `Gone`. `rustix` with `process` joins the manifest.
  - `marley_rail`: `PortSnapshot` (port, pid, title `:<port> <name>`, url, tooltip, matched),
    `ProjectSnapshot::ports`, `Selection::Port(port, pid)`, `PortRow`, `Row::Port`, `Shown::Port`,
    the walk's chain after threads, `parent`'s arm, `rail_rows`'s arm; `cycle_row` leaves ports out
    (its doc says so); the tests' snapshot helper and exhaustive matches gain their arms, and
    `rail_tests.rs`'s three.
  - `marley_mcp`: `Family::Ports` (served, `ports`), the `ports_list` row (read tier), its schemas
    (no arguments; `project`, `folder`, `address`, `port`, `url`, `pid`, `name`, `cwd`), and the
    deferred arm in `dispatch`. `CLIENT_READ_TOOLS` is untouched, so outside clients do not get
    it.
  - `marley_workbench::ports` (a public module, like the crate's others): the `Ports` global
    (listeners by `ProjectGroupKey`, a watcher count, a scanning flag); `watch` and `unwatch`; the
    loop, which reads every window's groups and their member folders on the main thread, scans
    and attributes off it, and stops once no rail is open. It reads the global through
    `try_global` and mutates it only when it stops or the listeners changed, since gpui's
    `global_mut` and `default_global` tell every observer. `attribute` applies D1 and D2;
    `project_names`, `list` and `stop` are also here.
  - `rail.rs`: `Rail::new` watches `Ports`, unwatches on release, and observes it
    (`observe_global_in`, as for `AgentEvents`). `build_snapshot` fills each project's ports
    (`port_snapshots`, the title matched by the filter). `render_port_row` draws `Server`, the
    title, the URL, and on hover Open (`ToolWeb`), Copy and Stop, with a tooltip on the row. A
    click on the row or Enter opens it (`open_port`: the group's workspace shown, then
    `browser::open_url_tab`); `stop_port` shows why not as a toast.
  - `mcp.rs`: `ports_list` answers from `ports::list` in a task, by project and port.
- **Deviations from the plan.**
  - `Ports` holds the listeners with their folder (`ProjectListener`), so `ports_list` has the
    folder without a second attribution.
  - The module is `pub mod ports` with `pub` items: `pub(crate)` inside a private module trips
    clippy's `redundant_pub_crate`, and `pub` there trips rustc's `unreachable_pub`.
- **Checks.** `cargo check` clean on the four crates; `cargo fmt` applied; `just clippy
  marley_browser marley_rail marley_mcp marley_workbench` (all targets, `-D warnings`) clean after
  five rounds. It flagged three long first doc paragraphs, the module's visibility, three `&mut`
  parameters used as `&` (`scan_while_watched`, `ports_list`, `stop_port`'s window and context),
  and `Ports` without `Debug`.
- **Review of the diff.** REQ-001 to REQ-009 each have their path: the rows and their tooltip,
  D2's three filters and the unreadable-`fd` skip, the three-second loop, Open through
  `open_url_tab`, Copy, Stop's fresh scan, D5's URL, one row per (port, pid), and `ports_list`
  without the command line. Races: a rail closing and another opening between two ticks keeps the
  one loop, since the loop reads the watcher count each tick. Re-entrancy: the loop's updates run
  in their own `cx.update`, and the rail's handlers update the workspace, never the rail, inside
  another update. Provenance: Orca's behavior from the design, no Orca function body; no Warp or
  Zed body. Found nothing else to change.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; `mcp_agent ports` ✓; run and read every shot ✓
  (four runs); 500 and 504 again ✓ (twice); the golden set with 521 ✓; the gate ✓.
- **The scenario.** `script/e2e/521-ports-per-project.sh`, `compositor sway`, so nothing reaches
  Chad's session and no focus report applies. Before Marley starts it starts five servers: in
  `repo/web` one bound to `0.0.0.0` (`http.server`), in `repo/tools` a stand-in on `127.0.0.1`
  and `::1` on one port, in `$E2E_WORK/elsewhere` one outside the project, in `repo/web` one that
  writes `marley` to its own `/proc/self/comm`, and in `repo/web` one whose command line names the
  profile's `browser/` folder. The log lists what each listens on (`ss -ltnpH`, by pid).
  `mcp_agent ports` in the fixture prints each `ports_list` entry with its keys.
- **Deviations from the E2E plan.**
  - The project's ports are picked free and rising (web, tools, the typed server, then run 3's
    late server), so the rows keep one order; the typed server gets its port, not `0`.
  - D2's name rule is proven by a server that renames itself through `/proc/self/comm`, not by a
    copy of `python3` named `marley`; the rule reads `comm` either way.
  - `repo/web` holds an `index.html`, so Open's tab has a title to check (`Web`), not a directory
    listing.
  - Copy is proven by reading the headless sway's clipboard (`wl-paste`), not by pasting into
    the terminal.
  - Open runs last. Its Browser tab adds a row above the port rows (#504's order), which would
    move the rows Copy and Stop click.
- **Run 1** (coordinates guessed from the layout): fourteen of fifteen checks passed, Open failed.
  At x 200 the click landed on Copy's button: `521-04-opened` shows "Copy URL" under the pointer,
  and no tab. The shots gave the real places: rows at y 182, 228 and 274, and the buttons at x
  186, 210 and 234. Rebuilt after `cargo fmt` touched `ports.rs` (whitespace only) so the proof
  runs on the tree as committed.
- **Run 2: every check passes.** The shots, in `scratchpad/521/run2`:
  - `521-01-rows` (REQ-001, REQ-003, REQ-008): under `repo`, after `repo — bash`, two rows: `:33807
    python3` with `http://127.0.0.1:33…` (the web server, bound to `0.0.0.0`) and `:41355
    python3` (the stand-in, one row for its two addresses: the log's `127.0.0.1:41355
    [::1]:41355`). The log shows the other three servers listening (`elsewhere` on 37611, `named`
    on 34159 and `profile` on 45467, the last two in `repo/web`), and none has a row.
  - `521-02-tooltip` (REQ-002): the pointer on the web row; Open, Copy and Stop show at its right,
    and the tooltip reads `python3 -u -m http.server --bind 0.0.0.0 33807`, `in <repo>/web` and
    `pid 1498577`. The tooltip wraps the long path at its slashes; it reads.
  - `521-03-appeared` (REQ-004): five seconds after `python3 -m http.server 60731` in the
    terminal, whose block says "running", a third row `:60731 python3`. #503's footer offers
    `127.0.0.1:60731` too.
  - `521-05-copied` (REQ-006): the pointer on the stand-in's row, "Copy URL" under it; the log:
    the clipboard holds `http://127.0.0.1:41355/`.
  - `521-06-stopped` (REQ-007): the typed server's row is gone, and its block ended
    `Terminated`, `exit 143` (SIGTERM). The log: `kill -0` on its pid fails, and `ports_list` no
    longer lists the port.
  - `521-04-opened` (REQ-005): a Browser tab `Web` on `http://127.0.0.1:33807/` showing "The web
    folder", its row in the rail above the port rows; `browser_tabs` lists it in project `repo`,
    focused.
  - The log (REQ-009): `ports_list` gives the web server and the stand-in with `project repo`, the
    folder `<work>/repo`, their working directories and `http://127.0.0.1:<port>/`, the typed
    server once it runs; each entry's keys are `address, cwd, folder, name, pid, port, project,
    url`, no command line; none of the other three servers' pids appears.
  - The Marley log has no `ports:` line: no scan failed and none took over 500 ms (it logs at
    INFO), with 1,313 processes on the box. Its ERROR lines are the headless run's usual ones (the
    Vulkan loader, llama.cpp's provider, the headless keymap): pre-existing, not in scope.
- **Found while the golden set ran: the scan outlived a closed rail.** `Rail::new` called
  `ports::watch` for the rail's whole life, so the scan went on while the user had closed the
  rail (Ctrl+Alt+J), against D3's "while a rail shows": one read of both tables (66 KB on this
  box) and every `fd` link of the user's processes every three seconds, for rows no one saw. Fixed:
  the rail watches while it is the window's sidebar, open, with AI on (`watch_ports_while_shown`,
  from the observer it already puts on the `MultiWorkspace`, and from one on the settings, since
  turning AI back on shows the rail with no word from the `MultiWorkspace`), and unwatches when it
  closes or goes. The
  scenario gained a step: Marley's `rchar` (`/proc/<pid>/io`) over nine seconds with the rail
  open, then closed, the closed count under a quarter of the open one; a server started while
  the rail is closed; its row once the rail opens again (`521-07-reopened`).
  - Where the step goes matters. The golden set read the edited scenario while the unfixed build
    ran, and 521 failed the step there, as it should: 634,556 bytes open, 552,079 closed. The
    numbers were larger than the scan explains. A throwaway probe of Marley's threads (scratchpad
    `521/diag-reads.sh`, not in the repository) on the fixed build: open, three `Worker` threads
    read 68.9, 68.8 and 68.6 KB (one round each) and 207,891 bytes in all; closed, 1,360 bytes,
    from `Timer` and the main thread. The rest came from #503, whose offer of a printed URL reads
    the same tables every two seconds while a terminal holds one (`links.rs`, `PORTS_POLL`), and
    the typed server had printed one. So the step runs before the typed server, after
    `ports_list`; Open stays last, since its Browser tab adds a row above the port rows.
- **Run 3, on the fixed build: every check passes, ten of ten.** The ports: web 33021, tools
  35265, typed 37137, late 41253. The step: 213,844 bytes read in nine seconds with the rail
  open, 1,344 closed. The shots, in `scratchpad/521/run3`, each read:
  - `521-01-rows`, `521-02-tooltip`, `521-05-copied`: as in run 2, with run 3's ports and pid.
  - `521-07-reopened` (D3): after the rail closed, the late server started and the rail opened
    again, three rows, `:33021`, `:35265` and the late `:41253`; the terminal's cursor is hollow,
    since the reopened rail holds the focus.
  - `521-03-appeared` (REQ-004): four rows, the typed `:37137` between the stand-in and the late
    server; the block running, #503's footer offering `127.0.0.1:37137`.
  - `521-06-stopped` (REQ-007): `:37137` gone, the late server's row in its place under the
    pointer with its buttons; the block `Terminated`, `exit 143`.
  - `521-04-opened` (REQ-005): the Browser tab `Web` on `http://127.0.0.1:33021/`, "The web
    folder", its row above the three port rows; `browser_tabs` lists it in project `repo`.
- **500 and 504 again:** both pass (1 and 7 checks) on the build before the fix and on the fixed
  one. Their shots (`500-02-opened`, `500-03-chip`, `504-01-rows`, `504-06-keyboard`,
  `504-08-closed`) show their rails as before, with no port row: their site servers run from the
  checkout, outside the project.
- **The golden set** (37 with 521), on the build before the fix: 36 pass; 521 failed only the
  closed-rail step, which it read from the edited scenario (above). The whole set runs again at
  the release install, on the tree as committed.
- **The gate** (`script/gates.sh --diff`, the logs in `scratchpad/521/gate.log` and `gate2.log`).
  The first run was red at gate:21, dylint's `async_block_without_await`: `ports.rs`'s three
  `background_spawn(async move { … })` around calls that never await (the scan, the scan for
  `ports_list`, and Stop). They now hand the executor `futures::future::lazy`, as #503's
  `links.rs` does. The second run: every gate PASS (rustfmt, clippy on every target, audit, deny,
  shear, gitleaks, shellcheck, no-suppressions, source bans, rustdoc, the Zed ledger, manifests,
  typos, semgrep, dylint), `GATE GREEN [diff]`, and the receipt written.
- **Run 4, on the tree as committed** (rebuilt after the dylint fix): ten of ten checks pass;
  213,844 and 230,148 bytes open against 1,344 and 1,336 closed across runs 3 and 4. Its seven
  shots, in `scratchpad/521/run4`, each read, show what run 3's do.
- **Not reached by a scenario:** another user's listener inside a project folder (the run has no
  second user); `listeners_in` drops a socket no readable `fd` link names, and the review read
  that path. The 30-second backoff after a slow scan: no scan on this box was slow.
- **Verdict: PASS.** Every criterion shows in the shots or the log, D3's closed rail included,
  and the gate is green.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md` (Added: a project's ports in the rail). The crate notes:
  `marley_browser.md` (Listening ports: `listeners_in`, `url`, `stop_in`), `marley_rail.md`
  (`PortSnapshot`, the walk, `parent`, `cycle_row` and the switcher leaving ports out),
  `marley_workbench.md` (the rail's port rows, the watch while the rail shows, the `Ports`
  section, `ports_list` in the MCP server's list) and `marley_mcp.md` (the `ports` family). The
  guide (`docs/marley/guide.md`): the rail's rows in order, a Port row in its table, what a port
  row shows and does, and `ports_list` in the tools. No plan row: the slice is the Orca survey's
  item 6, which `docs/marley/three-prong-plan.md` does not list. No Zed path changed beyond
  `Cargo.lock`, and gate:16 (the ledger) passed.
- **Knowledge appended:** F-claude-521-the-port-scan-ran-behind-a-closed-rail-001,
  PR-claude-run-a-views-poll-while-it-shows-001,
  L-claude-521-gpuis-mutable-global-access-tells-every-observer-001,
  L-claude-521-two-loops-read-the-tcp-tables-001,
  AD-claude-521-a-projects-ports-are-found-by-their-working-directory-001. The brain:
  consultation d9ef4cc2314d447189c8d8eb257c61aa closed with
  `decisions/marley-finds-a-projects-ports-by-their-processs-working-directory-marley-521`,
  follow-up by 2026-10-27.
- **Closed** TICKET-521 (with an As built line: the scan joined `marley_browser::ports` and
  `marley_workbench::ports`, no `marley_ports` crate); its backlog row left at promotion.
