---
pipeline_id: b7b131c0-e6e9-4929-8d54-803ffac4bbbd
ticket: docs/planning/tickets/closed/TICKET-788-ports-on-macos.md
status: Phase 4 — Complete PASS
title: "The Ports list on macOS"
type: bug
slice: macOS
references: [docs/planning/pipeline/completed/521-ports-per-project.spec.md, docs/planning/pipeline/completed/787-marleys-mac-app.spec.md]
---

## Title
The Ports list (#521), the served-URL offer (#503) and Stop read the user's listening sockets
from `/proc`, which macOS does not have. On the Mac (#787) the list stays empty, the footer never
offers a printed URL, and every 3 s the log says "ports: reading the listening ports: … reading
/proc/net/tcp6: File not found": 115 lines in a run's first six minutes. On macOS, Marley reads
the same facts from macOS's own `lsof` and `ps` instead.

## Scope
### In
- `crates/marley_browser/src/ports.rs`, pure parsers for:
  - `lsof -nP -iTCP -sTCP:LISTEN -F pctn` (pid, name, family, address);
  - `lsof -nP -a -d cwd -p <pids> -F pn` (each pid's folder);
  - `ps -ww -o pid=,command= -p <pids>` (each pid's command line);
  - an assembly into the `Listener`s Linux builds, one per port, `service: None`;
  - the reachable ports of the first output, for the footer.
- `crates/marley_workbench/src/ports.rs`: on macOS, the scan, and Stop's fresh look before its
  SIGTERM, read listeners through `crate::process::output`. The listing guard stays, and Stop
  never goes to `systemctl`.
- `crates/marley_workbench/src/links.rs`: on macOS, the footer's listening ports come from the
  same `lsof` output.

### Out (explicitly deferred)
- Containers on macOS: Docker Desktop's ports come from its VM, not `docker-proxy`. With no
  engine installed on the Mac, there is nothing to show.
- launchd jobs as services (Linux's systemd Stop and Restart). On macOS a listener is a process,
  and Stop signals it.
- Any change to the Linux path.

## Reference (§20)
N/A — Marley-specific: the Ports list (#521) is Marley's. The behavior kept is #521's on Linux:
- the user's listeners, given to the project whose folder deepest holds the process's working
  directory;
- Marley's own left out;
- Stop only after a fresh look finds that pid still on that port.

### Prior art
- **The code we ship:**
  - `marley_browser::ports` (`listeners_in`, `listening_ports_in`, `stop_in`) reads `/proc`, with
    `procfs-core` for the tables.
  - `sysinfo` (0.31, 0.39 in `Cargo.lock`) lists processes and working directories on macOS, but
    not sockets.
  - No `libproc` or `netstat2` crate is in the lock.
  - `crate::process::output` in `marley_workbench` is the adapter allowed to start programs
    (`.config/spawn-sites.txt`).
- **macOS's own tools:**
  - `lsof`'s field output (`-F`, `p`/`c`/`t`/`f`/`n` lines) is stable and documented in
    `lsof(8)`. `*` stands for any address.
  - `ps -ww -o pid=,command=` gives whole command lines.
- **Sample on the Mac:**
  - `p6926` / `cpostgres` / `tIPv4` / `n127.0.0.1:5434`;
  - `lsof -d cwd` → `p6926` / `fcwd` / `n/Volumes/…/rustal`;
  - `ps` → `6926 /opt/homebrew/opt/postgresql@18/bin/postgres -D …`.

## UI proof
On the Mac, where no `script/e2e.sh` scenario runs:
- `788-mac-ports.png`: Marley's rail with a local folder open as a project and a `python3 -m
  http.server 8788` started in that folder. The project's port row reads 8788 with `python3`.
- Marley's log over a stretch of the run, with no `ports:` warning in it.

On Linux, nothing changes and no Linux line runs differently; the run is `just shot`.

## Locked-In Decisions
- D1 — The parsers are pure functions of the programs' text in `marley_browser::ports`, where
  Linux's readers live. The programs run in `marley_workbench`'s `process` adapter, the one place
  allowed to start them.
- D2 — Per scan, the Mac runs at most three programs:
  - `lsof` once for the listeners;
  - once for their folders, named by pid;
  - `ps` once for their command lines.

  With no listener, only the first runs. A scan over 500 ms still backs off to 30 s, as on Linux.
- D3 — A listener whose folder `lsof` cannot read is skipped, as Linux skips a pid whose `cwd`
  link it cannot read. A missing command line leaves `command` empty, as Linux's does.
- D4 — `*` binds any address: `0.0.0.0` for `IPv4`, `::` for `IPv6`. An address `lsof` writes
  that does not parse is skipped.
- D5 — Stop on macOS looks again with the same `lsof` reads and then sends SIGTERM through
  rustix, as `stop_in` does on Linux. A gone process is `Gone`, a port no longer held is
  `NotListening`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a rail shows on macOS, a process of the user listening on a TCP port in a project's folder shall show as that project's port row, with its port and name | `788-mac-ports.png` |
| REQ-002 | WHILE Marley runs on macOS, the port scan shall log no warning on each round | the Mac's log over a stretch of minutes |
| REQ-003 | WHEN Stop is chosen on a macOS port row whose process still listens, Marley shall send that process SIGTERM | the server's process gone after Stop, and the row with it; shot or `pgrep` |
| REQ-004 | WHEN a terminal on macOS prints a local URL whose port listens, its footer shall offer the URL | `788-mac-offer.png`, or the review of the diff where no shot reaches it |
| REQ-005 | WHILE Marley runs on Linux, the scan, Stop and the footer shall read `/proc` as before | the review of the diff: each change is behind `cfg(target_os = "macos")`; `just shot`; the gate |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the parsers, the macOS reads in `ports.rs` and `links.rs`; a review of the diff;
  `script/gates.sh --diff` green on the box; `cargo clippy` over the touched crates green on the
  Mac.
- **P3 Test** — on the Mac: install, a local project, a server in it, shots, Stop, the log. On
  Linux: `just shot`.
- **P4 Complete** — CHANGELOG, the guide's Mac section, the crates' notes (§21), the ledger
  (§19), close, archive, commit.
