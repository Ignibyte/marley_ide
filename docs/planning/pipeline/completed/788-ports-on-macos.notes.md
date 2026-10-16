# The Ports list on macOS — Notes

- **Local ticket doc:** [TICKET-788](../../tickets/open/TICKET-788-ports-on-macos.md)
- **Pipeline spec:** 788-ports-on-macos.spec.md

## Phase 1 — Plan
- **Request:** seen in #787's check on the Mac. The Ports list reads `/proc` and warns every 3 s.
  The move's plan lists the Ports list among the Mac's gaps.
- **Classification / tier:** bug, macOS. Marley crates only (`marley_browser`,
  `marley_workbench`), no Zed hunk.
- **Recall (§18.3):**
  - AD-521: listeners by working directory, Marley's own left out, Stop only after a fresh look.
    A round over 500 ms backs off to 30 s. It names `sock_diag` behind `listeners_in` as the way
    out if the kernel drops the tables. Here macOS gets its own readers behind the same
    `Listener`.
  - L-521: two loops read the tables, #521's scan every 3 s and #503's offer every 2 s, so both
    move to the Mac's reads.
  - PR-786: a Linux-only call sits behind `cfg(target_os = "linux")`, with a macOS form beside
    it.
  - The brain (consultation `4ef50d85d8264052b025396ef181c955`): nothing on ports off Linux.
- **Discovery:**
  - `marley_browser/src/ports.rs`:
    - `listening_ports_in(dir)` (:25) and `listeners_in(proc_root)` (:149, through `fd` links,
      `cwd`, `comm`, `cmdline` and `cgroup`);
    - `stop_in(proc_root, port, pid)` (:254, a fresh `listeners_in`, then rustix SIGTERM);
    - `reachable` (:272);
    - `Listener { address, pid, name, command, cwd, service }` (:73).
  - `marley_workbench/src/ports.rs`:
    - `scan_while_watched` (:168-260), where `listeners_in` and `proxied_ports_in` run in a
      background `lazy`;
    - `stop` (:676, `own_service_in`, `stop_unit` or `stop_in`);
    - `unit_states`, which runs no `systemctl` when no listener has a service.
  - `marley_workbench/src/links.rs:373` `watch_ports`: `listening_ports_in("/proc/net")` every
    `PORTS_POLL`.
  - `marley_browser::containers::proxied_ports_in` returns empty when `/proc` cannot be read.
  - `crate::process::output(program, args, dir, env)` is the adapter (`.config/spawn-sites.txt`).
  - The Mac's `lsof -F pctn` gives `p<pid>`, `c<name>`, then per descriptor `f<fd>`,
    `t<IPv4|IPv6>`, `n<addr>:<port>`, with `*` for any address.
- **Decisions:** D1 to D5 in the spec.

### Design
- **`marley_browser::ports`** (cross-platform, pure):
  - `pub fn lsof_listening(text: &str) -> Vec<(u32, String, SocketAddr)>`: walk the lines and
    keep pid and name; per `t` the family, per `n` the address. `*:<port>` is the unspecified
    address of the family, `[v6]:<port>` and `v4:<port>` parse through `SocketAddr`. Malformed
    lines are skipped.
  - `pub fn lsof_folders(text: &str) -> HashMap<u32, PathBuf>`: `p` then `n`.
  - `pub fn ps_commands(text: &str) -> HashMap<u32, String>`: each line's first word is the pid,
    the rest trimmed is the command.
  - `pub fn listeners_from(listening, folders, commands) -> Vec<Listener>`: drop pids with no
    folder, one listener per port per pid (the first address after sorting), `service: None`,
    sorted by port and pid, as `listeners_in` ends.
  - `pub fn reachable_ports(listening) -> BTreeSet<u16>`: through `reachable`.
- **`marley_workbench::ports`:**
  - `#[cfg(target_os = "macos")] pub(crate) async fn mac_listening() -> Result<Vec<(u32, String,
    SocketAddr)>>`: `/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -F pctn`. lsof exits 1 when nothing
    matches, so stdout is read whatever the status, and only a failure to start is an error.
  - `#[cfg(target_os = "macos")] pub(crate) async fn mac_listeners() -> Result<Vec<Listener>>`:
    the first, then for the pids, joined with commas, `lsof -nP -a -d cwd -p … -F pn` and
    `/bin/ps -ww -o pid=,command= -p …`, then `listeners_from`.
  - `scan_while_watched` on macOS: `listened` from `mac_listeners().await`, `proxied` empty. The
    Linux branch keeps its lines.
  - `stop` on macOS: `mac_listeners` finds the pid on the port, else `NotListening`, then rustix
    TERM, mapping `SRCH` to `Gone`.
- **`marley_workbench::links::watch_ports`** on macOS: `reachable_ports(mac_listening().await)`.
- **File manifest** (all Marley crates):
  - `crates/marley_browser/src/ports.rs`: the parsers and `listeners_from`;
  - `crates/marley_workbench/src/ports.rs`: the Mac reads, the scan and stop branches;
  - `crates/marley_workbench/src/links.rs`: the footer's ports on macOS.

  The parsers have no `cfg`, so the Linux build compiles and clippy checks them; only their
  callers sit behind `cfg(target_os = "macos")`. With no caller on Linux they are `pub`, so no
  dead-code warning.

### Visual check plan
| REQ | What runs on the Mac | What shows it |
|---|---|---|
| 001 | `just install` (Mac, as #787); a folder `/tmp/ports-788` opened as a project (`open marley://file/tmp/ports-788`); `python3 -m http.server 8788` started in that folder over ssh | `788-mac-ports.png`, the rail with the project's port row 8788 `python3` |
| 002 | the same run, minutes on | `grep -c "ports:"` over the log lines since the start: 0 warnings |
| 003 | Stop on the row needs a click. The Mac has no Accessibility for `System Events`, so this is a review of the diff, plus a check that the server stops when signalled outside Marley and the row goes | the row gone in a second shot |
| 004 | a terminal in that project printing `http://localhost:8788/` | `788-mac-offer.png`, if a terminal can be typed into without Accessibility; else the review of the diff |
| 005 | Linux: `just shot` | Marley starts. The diff shows every Linux line inside the Linux branch |

What no scenario reaches: clicks and typing in the Mac's Marley. The Mac grants no Accessibility
to ssh or `osascript`, so Stop's click and typing in a terminal there are proven by the review
and by what the app shows after.

### Risks
- **A slow `lsof` on the Mac.** It can take hundreds of ms with many processes. The 500 ms
  back-off covers it, and the Test phase reads the scan times in the log.
- **lsof's exit status.** It exits 1 when nothing matches, so the code must not treat that as a
  failure.
- **A GUI launch's PATH.** Absolute paths (`/usr/sbin/lsof`, `/bin/ps`) are used, since a Dock
  launch's PATH may be bare until the login shell's environment loads.

## Phase 2 — Code
- **What was built:**
  - `marley_browser::ports`:
    - `LsofListening`;
    - `lsof_listening` (the `p`/`c`/`f`/`t`/`n` walk, `*` as the family's unspecified
      address);
    - `lsof_folders`, `ps_commands`, `listeners_from` (one per port of each pid, sorted as
      `listeners_in` sorts, `service: None`) and `reachable_ports`.

    `stop_in`'s signal half is now `stop_among(listeners, port, pid)`, which `stop_in` calls with
    its own fresh `listeners_in`, so Linux is unchanged.
  - `marley_workbench::ports`:
    - `mac_listening` (`/usr/sbin/lsof … -F pctn`, read whatever its exit status) and
      `mac_listeners` (plus `lsof -d cwd` and `/bin/ps` for the pids), both
      `cfg(target_os = "macos")`;
    - the scan's `listened` and `proxied` per platform. On macOS a test's `proc_root` stands for
      "nothing listens", so the field is read there too and tests stay off the real machine;
    - `stop`'s listeners per platform; Linux's no-service arm still calls `stop_in` (its second
      fresh scan), and macOS's `stop_among` over the scan it just made;
    - `list`, the agents' `ports_list`: one background task picking its listeners per platform.
  - `marley_workbench::links::watch_ports`: on macOS `reachable_ports(mac_listening())`. The
    `PROC_NET` constant and the `AppContext` import are Linux-only, and `Path` is spelled out at
    its one use.
- **Deviations:**
  - `ports::list` was not in the manifest. It calls `listeners_in` for the agents' `ports_list`
    tool, which the Mac's clippy found (E0425).
  - `list` now runs an async block, not a `lazy`, on Linux too, so neither platform has a
    `let_and_return`. Same thread, same work.
- **Review:**
  - REQ-001: the scan feeds `attribute` the same `Listener`s, so rows, project folders and
    Marley's own exclusion (by name `marley`) are #521's.
  - REQ-002: a macOS round errs only when `lsof` cannot start; lsof's exit 1 on "nothing
    listens" is not an error.
  - REQ-003: Stop finds the pid on the port in a fresh scan before the signal.
  - REQ-004: the footer's ports use `reachable` as on Linux.
  - REQ-005: every Linux statement is unchanged or behind `cfg(not(target_os = "macos"))`. The
    one shape change is `list`'s async block.
  - No `unwrap`, no `let _ =`, no panic on parse: slices are taken at a char boundary and
    numbers through `parse().ok()`.
- **Checks:**
  - box clippy on `marley_browser` and `marley_workbench`: red on two long first doc paragraphs
    and two `HashMap` parameters not generic over the hasher (`implicit_hasher`), fixed, then
    green;
  - Mac clippy `-D warnings` on the same crates: red on E0425 (`list`) and an unused
    `AppContext` import in `links.rs`, fixed, then green.
- **Process slip:** a `cargo fmt` ran while the first gate's cargo was running, against the
  one-cargo-at-a-time rule. `fmt` takes no build lock and writes no target, and nothing broke.
  The first gate also ran on a tree that changed under it, so it is void and the gate ran
  again.

- **Gate:**
  - run 1 was void: the tree changed under it. It still showed gate:18 `typos` reading lsof's
    `-F pn` as a misspelling of "on" (written `-F np`, the same fields), and gate:21 (Zed's
    dylint lints) refusing `list`'s async block, which had no `.await` on Linux.
  - `list` became `cx.background_spawn(listing(folders))`, with `listing` per platform: Linux's
    the old `lazy`, macOS's async.
  - Run 2 was stopped during its shellcheck step, with no cargo running, since the tree had
    changed again.
  - Run 3, on the final tree: **GATE GREEN [diff]**, 17 of 17. Mac clippy `-D warnings` on
    `marley_browser` and `marley_workbench` is green on the same tree.

## Phase 3 — Test
- **The run, on the Mac.**
  - The installed `Marley.app` was built from `169de085be` with this diff (`just install`, clippy
    first).
  - The test Marley was quit and reopened, and the log's line count kept as the start.
  - `~/ports-788` was opened as a project with `open "marley://file$HOME/ports-788"`, and
    `python3 -m http.server 8788 --bind 127.0.0.1` started in it.
  - The app was driven through its own MCP server (`mcp-endpoint.json`'s URL and headers, from a
    script on the Mac that prints none of them).
- **REQ-001:**
  - `ports_list`, the same listeners the rail draws: `{"project":"ports-788","folder":"/Users/…/
    ports-788","address":"127.0.0.1","port":8788,"pid":83537,"name":"Python","cwd":"/Users/…/
    ports-788"}`. `lsof` itself: `p83537 cPython n127.0.0.1:8788`.
  - `788-mac-ports.png`, Marley's window shot through iTerm: the rail's `ports-788` group holds
    its terminal and a port row ":8788 Python", "127.0.0.1:8788/", behind Zed's Unrecognized
    Project prompt, which every new folder gets.
- **REQ-002:** from 14:25:45 to 14:28:11 the log has 0 lines from `marley_workbench::ports`. The
  old code warned every 3 s, about 50 in that span. With the server stopped, `ports_list` answers
  `{"ports": []}` within one round.
- **REQ-003 (Stop): the review of the diff.** Stop needs a click, and the Mac grants
  Accessibility neither to ssh nor `osascript`. The fresh `mac_listeners` scan, then
  `stop_among`'s pid-and-port check and SIGTERM, are what `stop_in` does on Linux.
- **REQ-004 (the footer's offer): the review of the diff.** `terminal_run` refused to type at
  the Mac terminal's prompt ("a program runs in the terminal's foreground"), and `terminal_type`
  waits on the user's take-over. `reachable_ports` applies Linux's `reachable` to the same
  sockets.
- **REQ-005:** `just build`, then `COMPOSITOR=sway just shot 788-linux-unchanged`.
  `788-linux-unchanged.png` is Marley as before (the rail, `repo — bash`, the trust prompt), and
  the run's log has no `marley_workbench::ports` line. Chad's Hyprland Marley was untouched.
- **Seen, not in scope.** On the Mac, Marley's terminal shows zsh's plain prompt under "Last
  login: …": no prompt editor, `blocks: 0` in `terminal_list`, and `terminal_run` taking the shell
  for a foreground program. macOS starts the shell through `/usr/bin/login -flp`, so the shell is
  not the terminal's child, and Marley's zsh integration does not show. This is the next ticket.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Fixed: "The Ports list on a Mac".
  - `docs/marley/guide.md`, On a Mac: a Ports item, and the Ports list off the lacks list.
  - `docs/marley/three-prong-plan.md`: the macOS bullet.
  - `docs/marley_architecture/marley_browser.md` (the parsers and `stop_among`) and
    `marley_workbench.md` (the Mac round, `stop`, `list`, `links::watch_ports`).
  - No Zed crate, so no touchpoint row.
- **Ledger:**
  - F-claude-788-the-ports-scan-warned-every-3-s-on-macos-001
  - L-claude-788-drive-a-mac-marley-through-its-own-mcp-server-001
  - L-claude-788-a-lint-on-one-platform-splits-the-function-not-the-block-001
- **Brain:** consultation `4ef50d85d8264052b025396ef181c955` closed by
  `decisions/marleys-ports-list-on-macos-reads-lsof-and-ps`.
- **Next:** the zsh integration on macOS (seen in Phase 3) is minted as #792.

