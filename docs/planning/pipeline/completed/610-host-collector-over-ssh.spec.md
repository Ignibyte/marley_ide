---
pipeline_id: bcf7ae56-3806-4fb7-be9f-fb215010f679
ticket: docs/planning/tickets/open/TICKET-610-host-collector-over-ssh.md
status: Phase 4 — Complete PASS
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
    each with pid, runtime, working folder, start, CPU percent, RSS, and the session id from the
    process's `MARLEY_FLEET_SESSION` environment variable, which a harness or store sets when it
    starts an agent.
  - CPU and network rates come from the counters it keeps under `$XDG_RUNTIME_DIR`, else `/tmp`;
    on a first run it samples twice, a second apart.
  - It passes shellcheck.
- **Running it:** over SSH as #526's bootstrap runs its script, base64 in the command, so no
  stdin and no new spawn site: `ssh -T -o BatchMode=yes -o ConnectTimeout=5 -- <dest> 'printf %s
  <b64> | base64 -d | env MARLEY_COLLECT_NAMES=… sh'`, through `process::output`. Each connection
  is reused with SSH's own `ControlMaster`/`ControlPersist`, its socket in `$XDG_RUNTIME_DIR`,
  so a poll does not log in again. For this machine, the same script runs under `sh -c`.
  `MARLEY_SSH` names the `ssh` to run, as for remote terminals.
- **The hosts setting:** `"marley": { "fleet": { "hosts": [ { "ssh": "user@build-1" },
  { "local": true } ] } }`, each with an optional `name` and an optional `id`, the id the store
  gives the host (else the host's own name as the script reads it), and `"agent_processes"`,
  process names beyond `claude` and `codex` the script lists. Destinations go through
  `marley_remote::parse_ssh_target`, which refuses a leading dash (BF-claude-ssh-command-leading-
  dash-host-is-option-smuggling-injection-001).
- **Polling:** each listed host every 5 s while a fleet surface shows (a Fleet panel, or an Agent
  tab in front), and none while hidden, as the port scan does (F-claude-521).
- **The join:** a host's agent processes are joined to the store's agents by `session`, else by
  host with `cwd` and `runtime`.
  - A process no store agent claims shows under its host, in a Hosts section after the stores,
    as an agent with no work records (runtime, folder, pid, CPU, memory), chip `running`.
  - Each host's header carries a line of its resources: CPU, memory, disk and network.
  - A store agent whose host is listed takes that host's snapshot for its bars (#608) and its
    graphs (#609), and its snapshot names the process that joined it (pid, CPU, memory).
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
  - L-claude-584: a scenario runs an `sshd` of its own on localhost (`584-outside-clients-from-
    other-machines.sh`'s `start_sshd`, re-used here).
  - The shellcheck gate lists its scripts (`script/gates.sh:181`); the collector joins it.

## UI proof
The scenario `script/e2e/610-host-collector-over-ssh.sh` (sway):
- It starts an `sshd` of its own on localhost with a scratch key (L-claude-584), and points
  `MARLEY_SSH` at a wrapper that adds the scenario's options.
- It starts a process named `claude` (a stand-in that sleeps) in a scratch folder.
- It lists three hosts: that `sshd`, this machine, and an unreachable one.
- It starts two stand-ins (`sleep` under the names `claude` and `codex`): `claude` with
  `MARLEY_FLEET_SESSION=agent-7f3a`, the pseudo provider's build-1.
- It lists four hosts: that `sshd` as `lab` with the store's id `host-build-1`; this machine; an
  unreachable one as `gone` with the id `host-vps-2`; and `-oProxyCommand=…` as `evil`.
- Shots:
  - `hosts.png` and `hosts-more.png` (scrolled): under the pseudo provider, build-1's host
    header with lab's resource line and docs-1 `offline` under an unreachable vps-2; under Hosts,
    `gone` unreachable and `evil` refused first, then lab and this machine with their resource
    lines and the `codex` stand-in `running`;
  - `joined.png`: build-1 selected, its bars from lab and the stand-in `claude` named as its
    process;
  - `snapshot.png`: the `codex` stand-in selected, with CPU and memory bars;
  - `tooltip.png`: the pointer on `gone`'s chip, the SSH error in its tooltip;
  - `refused.txt`: Marley's log line refusing `evil`, and no file the ProxyCommand would have
    made.

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
