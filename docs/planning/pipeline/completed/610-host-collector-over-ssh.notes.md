# The host collector: a script Marley runs over SSH — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-610-host-collector-over-ssh.md
- **Pipeline spec:** 610-host-collector-over-ssh.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - BF-claude-ssh-command-leading-dash-host-is-option-smuggling-injection-001: parse_ssh_target refuses a leading dash, and ssh_command puts -- before the destination.
  - L-claude-584: a scenario runs an sshd of its own on localhost; MARLEY_SSH points Marley at a wrapper.
  - F-claude-521: poll only while a surface shows the data.
  - gate:22: process.rs is the workbench's one spawn module; process::output has no stdin, hence the script in the command.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:**
  - PR-claude-607 and #609: the reads run while a Fleet surface shows (`fleet_shows`); host
    collection joins that loop and stops with it (REQ-006).
  - PR-claude-600: nothing the collection calls reads an entity; it runs in the loop's task
    between reads.
  - The brain (consultation fc6f82f0c931407eaf7cc650af837f61): nothing on this seam, only
    follow-ups due on other projects.
- **Seams re-verified:**
  - `process::output(program, args, dir, env)` is async (`process.rs:18`), so the loop awaits
    the collector without blocking the window; gate:22's count stays.
  - `marley_remote::parse_ssh_target` refuses a leading dash (`marley_remote.rs:43`);
    `ssh_command` gives `ssh [-p port] -- dest` (`marley_remote.rs:104-117`).
  - `MARLEY_SSH` (`remote.rs:113-114`); #584's `ssh -T -o BatchMode=yes` (`clients.rs:556-562`)
    and its `env` for any login shell.
  - `start_sshd` in `584-outside-clients-from-other-machines.sh:53-93`.
  - No Marley crate uses `ControlPath` yet; `$XDG_RUNTIME_DIR` is the user's own 0700 folder, so
    the socket goes there, named `marley-ssh-%C`, and nothing is created.
  - `settings_content::MarleyFleetSettingsContent { providers }` (`marley.rs:138`) and
    `MarleySettings::fleet_providers` (`marley_workbench.rs:374, 508`).
  - The shellcheck gate's list (`script/gates.sh:181`).

### Design
- **Approach.**
  - *The script*, `crates/marley_workbench/bin/marley-collect.sh`, POSIX `sh`, shipped by
    `include_str!`. It reads `/proc/stat` (CPU totals and `btime`), `/proc/meminfo`,
    `/proc/loadavg`, `/proc/uptime`, `/proc/net/dev` (all but `lo`), `df -P -k /`, and each
    `/proc/<pid>` whose `comm` is `claude`, `codex` or a name in `MARLEY_COLLECT_NAMES`: its
    `cwd`, `stat` (start and CPU ticks), `status` (`VmRSS`) and `environ`
    (`MARLEY_FLEET_SESSION`). Rates (the host's CPU and network, each process's CPU) come from
    the counters of its previous run in a state file under `$XDG_RUNTIME_DIR`, else a 0700
    folder of its own in `/tmp` it checks it owns; a first run samples twice, a second apart.
    It prints one `marley.host/v1` document; strings are escaped (backslash, quote, control
    characters dropped). Integer arithmetic only, tenths printed as `N.D`.
  - *Settings* (Zed crate `settings_content`, additive): `MarleyFleetSettingsContent` gains
    `hosts: Option<Vec<FleetHostContent>>` (`ssh`, `local`, `name`, `id`) and
    `agent_processes: Option<Vec<String>>`; `default.json` gains `"hosts": []` and
    `"agent_processes": []`; their touchpoint rows first. `MarleySettings` resolves them into
    `fleet_hosts: Vec<FleetHost>` and `fleet_agent_processes`.
  - *A new file, `crates/marley_workbench/src/fleet_hosts.rs`*: `FleetHost { target:
    HostTarget::{Local, Ssh(String)}, name, id }`, `Collected { name, id, snapshot, problem }`,
    and `collect(hosts, names) -> Vec<Collected>` (async): `sh -c <script>` for this machine,
    else `parse_ssh_target` (a failure is `problem: "refused: … is not an SSH destination"`,
    logged once, nothing run) and the SSH command through `process::output`, with
    `MARLEY_COLLECT_NAMES` limited to `[A-Za-z0-9._-]` names. A failed run's problem is the last
    line of its stderr. And `join(sources, collected) -> Source`: each reachable host's snapshot
    replaces or joins a store source's host of the same id; each process is claimed by
    `session` (a store agent's id), else by host, runtime and the agent's `cwd` where its detail
    is known, else by host and runtime when exactly one agent and one process pair up; a claimed
    process is kept in the source's `processes` under the agent's id; the rest become a Hosts
    source (`capabilities: ["hosts"]`, every listed host a group, each process an agent
    `"<host id>:<pid>"` named for its folder, with a synthesized detail carrying the host's
    snapshot). A store source's host that did not answer goes in its `unreachable`, so its agents
    read `offline`.
  - *The loop* (`fleet.rs`): before each read, if 5 s have passed since the last collection,
    it awaits `collect` outside any update and stores the result in `Fleet.collected`;
    `read_providers` joins it into the sources and appends the Hosts source. `read_now` joins
    the last collection.
  - *The panel*: each host header gets a resource line ("CPU 42 % · 3.1 / 8.0 GB · disk 41 / 80
    GB · in 55 kB/s · out 20 kB/s") and, for an unreachable or refused host, a chip with the
    reason in its tooltip; a store agent on an unreachable host reads `offline`; a process
    row's second line is "pid 4121 · 18 % · 420 MB" and its chip `running`; a joined agent's
    snapshot adds "process 4121 · 18 % CPU · 420 MB" under RESOURCES. "Not set up" gives way
    to hosts too. `rate` moves from `agent_tab.rs` to `fleet.rs`.
- **File manifest:**
  - Marley: `crates/marley_workbench/bin/marley-collect.sh` (new),
    `crates/marley_workbench/src/fleet_hosts.rs` (new), `fleet.rs`, `agent_tab.rs`,
    `marley_workbench.rs`.
  - Zed: `crates/settings_content/src/marley.rs`, `assets/settings/default.json` (rows first).
  - Gate: `script/gates.sh` (shellcheck lists the collector).
- **Visual check plan** (`script/e2e/610-host-collector-over-ssh.sh`, sway; the stand-ins and
  the hosts as in the spec's UI proof):

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001, REQ-003, REQ-004 | Toggle the panel, wait for two collections | `hosts.png` |
  | REQ-002 (joined) | Click build-1 | `joined.png` |
  | REQ-002 (unclaimed) | Click the `codex` stand-in under this machine | `snapshot.png` |
  | REQ-003 (reason) | Point at `gone`'s chip | `tooltip.png` |
  | REQ-005 | Grep the log for the refusal; test that the ProxyCommand's file is absent | `refused.txt` |
  | REQ-006 | Review: collection lives in the loop, which stops when nothing shows | review |

  The shots show the dev box's own processes too, if any Claude Code runs on it; the notes
  describe the stand-ins only.
- **Risks and decisions:**
  - A slow host holds the loop for up to `ConnectTimeout` (5 s) per collection; hosts run one
    after another. Parallel collection can come if it matters.
  - `MARLEY_FLEET_SESSION` is Marley's name for the session a store's agent carries; the
    contract doc says so at Complete.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `bin/marley-collect.sh`: POSIX `sh`; the state folder is `$XDG_RUNTIME_DIR` when it is the
    user's own, else a 0700 `/tmp/marley-collect-<uid>` checked with `find -prune -user`; host
    CPU and network rates and each process's CPU over the previous run's counters (a first run
    samples twice, a second apart; a process with no earlier count reads its lifetime average);
    `comm` matched against `claude`, `codex` and `MARLEY_COLLECT_NAMES`; the session from
    `MARLEY_FLEET_SESSION`; the state written to a temporary file and moved into place. Run by
    hand under `sh` and `bash --posix`: valid JSON, nothing on stderr (the dev box has neither
    `dash` nor `busybox`).
  - `fleet_hosts.rs`: `FleetHost`/`HostTarget` from the settings, `collect` (names limited to
    `[A-Za-z0-9._-]`, at most 32 characters), `collect_one` (`sh -c` locally; `parse_ssh_target`,
    then `ssh -T -o BatchMode=yes -o ConnectTimeout=5`, `ControlMaster=auto`,
    `ControlPath=$XDG_RUNTIME_DIR/marley-ssh-%C`, `ControlPersist=60`, `ssh_command`'s `-p` and
    `--`, and `printf %s <b64> | base64 -d | env MARLEY_COLLECT_NAMES='…' sh`), `join`, `claim`,
    `process_agent`, `process_line`.
  - `fleet.rs`: `Fleet.collected`; `Source.unreachable`, `processes` and `hosts_only`, with
    `Source::for_hosts`, `process`, `unreachable`; `host_groups` takes unreachable hosts and, for
    the Hosts source, hosts with no agents; the loop collects every 5 s between reads while a
    surface shows, and `keep_collected` logs a host refused the first time; `read_providers`
    joins and appends the Hosts source; host headers carry a resource line (`host_line`) and an
    `unreachable` or `refused` chip whose tooltip is the reason; `agent_chip` (`running`,
    `offline`, else the state or `stale`); a process row's line and a joined agent's "process …"
    line; `rate` moved here from `agent_tab.rs`.
  - Settings: `FleetHostContent { ssh, local, name, id }`, `hosts` and `agent_processes`
    (`settings_content`), `"hosts": []` and `"agent_processes": []` (`default.json`), their
    touchpoint rows first; `MarleySettings.fleet_hosts` and `fleet_agent_processes` through
    `fleet_settings`.
  - `script/gates.sh`: shellcheck lists the collector.
- **Deviations:** `fleet_settings` is a helper of its own (clippy's `too_many_lines` on
  `MarleySettings::from_settings`).
- **Review of the diff:**
  - REQ-001: `host_line` gives CPU, memory, disk and network for each collected host.
  - REQ-002: `claim` by session, folder, then the one-to-one pair; unclaimed processes become
    the Hosts source's agents with details, so they select and open like any agent.
  - REQ-003: a host with no snapshot goes in `unreachable` of the Hosts source and of each store
    source with agents on it; its chip carries the reason; `agent_chip` reads `offline`.
  - REQ-004: `HostTarget::Local` runs the same script under `sh -c`.
  - REQ-005: `parse_ssh_target` refuses before `process::output` is reached; the refusal is
    logged once and shown as `refused`.
  - REQ-006: collection lives in the loop, which ends when `fleet_shows` is false.
  - Re-entrancy: collection awaits between `cx.update` calls; nothing reads an entity during it.
  - Security: the script runs as the user, reads `/proc` and `df`, writes only its state file;
    the names that reach the remote shell are checked; the destination sits after `--`.
- **Gate:** run 1 red on gate:14 (the module docs linked to the private `collect` and `join`),
  fixed at the source; run 2 `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/610-host-collector-over-ssh.sh` under sway. Its `sshd` on
  127.0.0.1 (#584's `start_sshd`), `MARLEY_SSH` pointing at a wrapper with the scenario's key;
  two stand-ins (`sleep` as `claude`, with `MARLEY_FLEET_SESSION=agent-7f3a`, and as `codex`);
  the hosts `lab` (the `sshd`, id `host-build-1`), `this machine`, `gone` (`127.0.0.1:1`, id
  `host-vps-2`) and `evil` (`-oProxyCommand=touch <scratch>/pwned`). The dev box's own Claude
  Code and Codex processes show in the shots too, since the `sshd` is this machine; the row the
  `codex` stand-in takes is worked out from `/proc` in the collector's order (`standin_row`).
- **Shots and files, each read** (run 3):
  - `hosts.png` (REQ-001, REQ-003): under the pseudo provider, the host header `lab` (build-1's
    host, joined by id) with its line "CPU 3 % · 48.3 / 134.8 GB · disk 703 / 2047 GB · in 201
    B/s · …", build-1 and review-1 under it; `vps-2` with an `unreachable` chip and docs-1
    `offline`. Under Hosts: `gone` `unreachable`, `evil` `refused`, then `lab` with its line and
    its process rows, each `running` with "pid … · … % · … MB".
  - `hosts-more.png` (REQ-004, REQ-002): scrolled down, lab's rows with `standin-codex` among
    them, then `this machine` with its own line and the same processes, `standin-codex` among
    them.
  - `snapshot.png` (REQ-002, unclaimed): `standin-codex` selected: `running`, "codex · for 43
    s", "lab · <scratch>/standin-codex"; RESOURCES with CPU and memory bars from lab and "process
    pid … · 0.0 % · 2 MB"; no work item, run or tokens, since the Hosts source offers only
    `hosts`.
  - `joined.png` (REQ-002, claimed): build-1 selected: "lab · /srv/work/pipeline", its run and
    work item from the store, RESOURCES from lab (CPU 10 %, 48.4 / 134.8 GB) and "process pid
    … · 0.0 % · 2 MB", the `claude` stand-in claimed by its session.
  - `tooltip.png` (REQ-003): the pointer on `gone`'s chip: "ssh: connect to host 127.0.0.1 port
    1: Connection refused".
  - `refused.txt` (REQ-005): `WARN [marley_workbench::fleet] fleet: host "evil" refused:
    "-oProxyCommand=touch <scratch>" is not an SSH destination`, and "the ProxyCommand's file
    does not exist: nothing ran".
  - REQ-006 is the review's (Phase 2).
  - The SSH host answered on every collection: `ControlPersist`'s master did not hold the
    command's output open.
- **Red found and fixed** (run 1): vps-2 was listed twice under the pseudo provider, and lab had
  moved below it. The join replaced a store's host by removing it and pushing the snapshot at
  the end, and `host_groups` added an unreachable host's id although the store already named
  it. The join now replaces the host in place and `host_groups` adds each id once; the Hosts
  section also lists hosts with a problem first, so they show above the process rows. Clippy
  and the build clean; `just gate-diff` runs again at Complete, after the scenario's last edit.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after; the scenario's `sshd` and stand-ins were killed in `teardown`.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: hosts read over SSH, and their agents);
  `three-prong-plan.md` (#610 shipped); `fleet-contract.md` (the collector's command, its file,
  the settings, `MARLEY_FLEET_SESSION` and the one-to-one pair); `marley_workbench.md` (the Fleet
  section's hosts bullet and a section for `fleet_hosts.rs`); the guide (a Hosts over SSH
  section and the settings block), the guide page (article `fleet-hosts`, its nav entry, the
  settings row) and the walkthrough (Part 12's hosts check). The touchpoint rows for
  `settings_content/src/marley.rs` and `default.json` were written before the code and describe
  what shipped.
- **Knowledge:** F-claude-610-a-joined-host-was-listed-twice-and-moved-001,
  L-claude-610-a-host-scenario-on-this-machine-sees-its-real-agents-001,
  AD-claude-610-marley-reads-hosts-with-its-own-script-in-the-ssh-command-001.
- **Brain:** consultation fc6f82f0c931407eaf7cc650af837f61 closed with `brain decide`
  (`decisions/marley-reads-its-fleets-hosts-with-its-own-script-sent-in-the-ssh-command`,
  follow-up by 2026-10-14).

