---
pipeline_id: bcf7ae56-3806-4fb7-be9f-fb215010f679
ticket: docs/planning/tickets/open/TICKET-610-host-collector-over-ssh.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The host collector: a script Marley runs over SSH"
type: feature
slice: prong 2, D20, wave 1; after #607 (and #608 for the snapshot's bars)
references: [docs/marley/fleet-contract.md, docs/planning/pipeline/completed/584-outside-clients-from-other-machines.spec.md, docs/planning/pipeline/completed/543-remote-terminals-survive-a-drop.spec.md]
---

## Title
Marley reads each host's resources and running agents itself, by running a small POSIX `sh`
script there over SSH, and joins what it finds to the agents the workflow store reports.

## Scope
### In
- **The script**, `marley-collect.sh`, shipped inside Marley (`include_str!`), POSIX `sh` with
  no dependencies beyond `/proc`, `df` and `base64`.
  - It prints one `marley.host/v1` document: host id and name, OS, cores, uptime, CPU percent,
    load, memory, disks, network rates.
  - It lists the agent processes it recognizes (`claude`, `codex`, and names a setting adds),
    each with pid, runtime, working folder, start, CPU percent, RSS, and the session id when the
    runtime exposes one.
  - CPU and network rates come from the counters it keeps under `$XDG_RUNTIME_DIR`, else `/tmp`;
    on a first run it samples twice, a second apart.
  - It passes shellcheck.
- **Running it:** over SSH as #526's bootstrap runs its script, base64 in the command, so no
  stdin and no new spawn site: `ssh -T -o BatchMode=yes -o ConnectTimeout=5 -- <dest> 'printf %s
  <b64> | base64 -d | sh'`, through `process::output`. Each connection is reused with SSH's own
  `ControlMaster`/`ControlPersist` under the runtime directory, so a poll does not log in again.
  For this machine, the same script runs under `sh` directly.
- **The hosts setting:** `"marley": { "fleet": { "hosts": [ { "ssh": "user@build-1" },
  { "local": true } ] } }`, each with an optional `name`. Destinations go through
  `marley_remote::parse_ssh_target`, which refuses a leading dash (BF-claude-ssh-command-leading-
  dash-host-is-option-smuggling-injection-001).
- **Polling:** each listed host every `poll_s` while a fleet surface shows it, and none while
  hidden, as the port scan does (F-claude-521).
- **The join:** a host's agent processes are joined to the store's agents by `session`, else by
  host with `cwd` and `runtime`.
  - A process no store agent claims shows under its host as an agent with no work records
    (runtime, folder, pid, CPU, memory).
  - A store agent whose host is listed takes that host's snapshot for its bars (#608) and its
    graphs (#609).
- **States:** a host that does not answer reads *unreachable*, with the SSH error's last line in
  its tooltip, and its agents read *offline*.

### Out (explicitly deferred)
- Password prompts: `BatchMode` means a host needs a key or an agent, as #543's hosts do.
- A collector that pushes to the store (the contract allows it; the store serves it as `host`).
- Hosts found automatically from Zed's `ssh_connections`: the list is explicit, so Marley never
  logs in anywhere unasked.

## Reference (§20)
N/A — Marley-specific: the collector and its output are Marley's own contract. Orca shows CPU and
memory per repository, worktree and session in its Resource Manager
(`docs/orca_architecture/05-terminal-and-workspace.md:502`); Marley reads the host over SSH
instead of from its own process tree, so remote agents count too.

### Prior art
- **Behavior maps:** the Orca note above.
- **Published material:** `proc(5)` for `/proc/stat`, `meminfo`, `loadavg`, `uptime`, `net/dev`,
  and a pid's `stat`, `status`, `cmdline`, `cwd`; OpenSSH's `ControlMaster`, `ControlPersist`,
  `BatchMode`.
- **Code we already ship:**
  - #526's base64 bootstrap under `sh -c` (`crates/marley_terminal/src/shell_integration.rs:69-110`).
  - `marley_remote::parse_ssh_target` and `ssh_command` (`crates/marley_remote/src/marley_remote.rs:43-117`).
  - `MARLEY_SSH`, which a scenario uses to point Marley at its own `ssh` wrapper
    (`crates/marley_workbench/src/remote.rs:113-114`).
  - `process::output`, the workbench's one spawn module (gate:22).
  - #584's `ssh -T -o BatchMode=yes` (`crates/marley_workbench/src/clients.rs:556-562`).
  - `marley_browser::ports` for reading `/proc` safely (`crates/marley_browser/src/ports.rs`).
  - L-claude-584: a scenario runs an `sshd` of its own on localhost.

## UI proof
The scenario `script/e2e/610-host-collector-over-ssh.sh` (sway):
- It starts an `sshd` of its own on localhost with a scratch key (L-claude-584), and points
  `MARLEY_SSH` at a wrapper that adds the scenario's options.
- It starts a process named `claude` (a stand-in that sleeps) in a scratch folder.
- It lists three hosts: that `sshd`, this machine, and an unreachable one.
- Shots:
  - `hosts.png`: the SSH host with its resources and the stand-in `claude` listed under it; this
    machine; the unreachable host with its state;
  - `snapshot.png`: the stand-in agent selected, with CPU and memory bars;
  - `refused.png`: a host named `-oProxyCommand=…` refused in the log and not run (`refused.txt`).

## Locked-In Decisions
- D1 — The script travels in the SSH command, base64-encoded, as #526's does: no stdin, no
  install, no new spawn site.
- D2 — Hosts are listed explicitly; Marley logs in nowhere it was not told to.
- D3 — Polls reuse one SSH connection per host, and stop while nothing shows the host.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a listed SSH host answers, the Fleet panel shall show its CPU, memory, disk and network from the collector script. | Shot `hosts.png` |
| REQ-002 | WHILE an agent process runs on a listed host, the panel shall list it under that host, joined to the store's agent when one claims it, else as an agent with no work records. | Shots `hosts.png`, `snapshot.png` |
| REQ-003 | WHEN a listed host does not answer, the panel shall mark it unreachable, with the reason in its tooltip, and its agents offline. | Shot `hosts.png` |
| REQ-004 | WHERE the hosts setting has `{ "local": true }`, the panel shall show this machine from the same script run locally. | Shot `hosts.png` |
| REQ-005 | IF a host's destination starts with a dash, THEN Marley shall refuse it and run nothing. | `refused.txt`; review |
| REQ-006 | WHILE no fleet surface shows a host, Marley shall not poll it. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the script's JSON writer, the ring buffer shared with
  #609, the control path).
- **P2 Code** — the script, the runner, the setting, the join; shellcheck clean; a review; the gate
  green.
- **P3 Test** — the scenario with its own `sshd`, every shot read.
- **P4 Complete** — docs (§21), the contract doc's collector line, the ledger; close, archive,
  commit.
