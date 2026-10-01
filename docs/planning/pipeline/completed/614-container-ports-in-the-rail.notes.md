# Container ports in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-614-container-ports-in-the-rail.md
- **Pipeline spec:** 614-container-ports-in-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-603 defers containers: their ports belong to root's `docker-proxy`, which the rail does not list.
  - The trap: `service_of` would read `docker-proxy`'s cgroup as `docker.service`, and Stop would stop Docker itself.
  - L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001: fakes first on the PATH reach Marley.
  - F-claude-521: the scan runs only while a rail watches.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:** as queued. The brain (consultation 8daed629f3ad45f29fb78e6f708ba898):
  nothing on this seam.
- **Seams re-verified:**
  - `marley_browser::ports::Listener { address, pid, name, command, cwd, service }` and
    `Service { unit, user }`; `listeners_in` ties sockets to pids through readable `fd` links only.
  - `marley_workbench::ports`: `ProjectListener { folder, listener }`, `Ports { by_group, … }`,
    `scan_while_watched` (a background `attribute(listeners_in(..), &folders)`, then the global
    written on change), `stop`, `stop_unit`, `refusal_reason`, `hand_command`.
  - `marley_rail::PortSnapshot` and `PortRow` (`service: Option<PortService>`); the rail's
    `port_snapshots`, `render_port_row` and `stop_port`.
  - On the dev box, root's real `docker-proxy` processes' command lines are readable and
    NUL-separated (`/usr/bin/docker-proxy -proto tcp -host-ip … -host-port … -container-ip …
    -container-port …`); `perl -e '$0 = "docker-proxy …"'` gives a stand-in whose command line
    joins its words with spaces.

### Design
- **Approach.**
  - *`marley_browser`* (pure, a new module `containers.rs`): `ProxiedPort { pid, address,
    target }` and `proxied_ports_in(proc_root)` (each `/proc/<pid>/cmdline` whose first word is
    `docker-proxy`, split on NULs and spaces, `-proto tcp` only); `Engine { Docker, Podman }`;
    `Container { id, name, engine, host_ports, working_dir }`; `parse_docker_ps` (JSON lines:
    `ID`, `Names`, `Ports` like `0.0.0.0:8081->80/tcp, [::]:8081->80/tcp`, `Labels` as
    `k=v,…`) and `parse_podman_ps` (a JSON array: `Id`, `Names`, `Ports [{host_port, …}]`,
    `Labels {…}`); `HELPERS`, the process names that hold a container's port (`rootlessport`,
    `rootlessport-child`, `pasta`, `pasta.avx2`, `slirp4netns`, `docker-proxy`); `ENGINE_UNITS`
    (`docker.service`, `podman.service`, `containerd.service`).
  - *`marley_workbench::ports`*: `ProjectListener` gains `container: Option<ContainerRef { engine,
    name, id, target, refusal }>`; `Ports` gains `containers` (container ports in no project) and
    an engine cache. The scan's background part also reads the proxied ports and keeps helper
    listeners out of `attribute` (they become Podman container ports); a listener whose service is
    an engine's own unit loses it. Between scans, when an engine's set of ports changed (or 30 s
    passed), `docker ps --format '{{json .}}'` or `podman ps --format json` runs through
    `process::output`; a refusal keeps its stderr's last line. Container ports then go to the
    project whose folder holds their Compose working folder, else to `containers`.
    `stop_container(engine, name)` runs `<engine> stop <name>`; a port with no name is refused at
    once, with `docker stop $(docker ps -q --filter publish=<port>)` to copy.
  - *`marley_rail`* (pure): `PortSnapshot` and `PortRow` gain `container: Option<PortContainer {
    engine, name, target }>`.
  - *The rail*: `port_snapshots` fills it (the row's second line "container web-614", or the
    target "→ 172.17.0.2:80" when unnamed); Stop on a container row stops the container, its
    refusal a toast with the reason and Copy Command as #603's; a Containers section after the
    projects, its rows drawn with `render_port_row`, opening in the shown workspace.
- **File manifest** (Marley crates only): `crates/marley_browser/src/containers.rs` (new) and its
  `mod` line; `crates/marley_workbench/src/ports.rs`; `crates/marley_rail/src/marley_rail.rs`;
  `crates/marley_workbench/src/rail.rs`.
- **Visual check plan** (`script/e2e/614-container-ports-in-the-rail.sh`, sway): a fake `docker`
  first on the PATH (its answer and its refusal switched by a file), and a stand-in
  `docker-proxy` for a free port.

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001 | The fake answers with `web-614`, Compose folder the scratch project | `named.png`: the port under the project, "container web-614" |
  | REQ-002 | Stop on that row | `stopped.txt`: the fake's record of `stop web-614` |
  | REQ-003 | The fake refuses | `refused.png`: the port under Containers with its target, Stop's toast with Copy Command |
  | REQ-004 | Review | `ENGINE_UNITS` cleared in `attribute` and `stop` |

- **Risks and decisions:**
  - The Podman path (helpers, `podman ps`) cannot run on the dev box (no Podman); its parser and
    flow share the Docker path's, and the notes will say it is untried.
  - The Containers section's rows are outside the rail's keys and filter for now.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `marley_browser::containers` (new): `HELPERS`, `ENGINE_UNITS`, `Engine` (`command`,
    `of_helper`), `ProxiedPort` and `proxied_ports_in` (a command line split on NULs and spaces,
    TCP only), `Container`, `parse_docker_ps` (with `docker_host_ports`) and `parse_podman_ps`.
  - `marley_rail`: `PortContainer { engine, name, target }` on `PortSnapshot` and `PortRow`.
  - `marley_workbench::ports`: `ProjectListener.container: Option<ContainerRef { engine, name,
    target, refusal }>`; `Ports.containers` and `Ports.engines` (`EngineAnswer { ports, asked,
    containers }`); the scan also reads `proxied_ports_in` in its background part, then
    `container_ports`, `ask_engines` (an engine is asked when its ports changed or 30 s passed)
    and `ask_engine` through `process::output`, then `attribute_containers` with `deepest`;
    `attribute` passes helper processes by and clears an engine's own unit, as `stop` does;
    `container_at` and `stop_container` (`ContainerStop::{Stopped, Refused { command, reason }}`).
  - The rail: `port_snapshot` (from `port_snapshots`, with the container and its tooltip),
    `container_line`, `container_refused_toast`, `stop_words` with a container's words,
    `render_port_row`'s container line and Stop, `stop_container_port`, and `render_containers`
    after the projects' blocks.
- **Deviations:** none from the design.
- **Review of the diff:**
  - REQ-001: a port whose container's Compose folder is in a project goes to that project's rows,
    named "container …".
  - REQ-002: Stop on a container row runs `<engine> stop <name>`.
  - REQ-003: without the engine, the proxy's port is listed apart with its target, and Stop is
    refused at once with the engine's reason and a command that finds the container by port.
  - REQ-004: `ENGINE_UNITS` are cleared in `attribute` and never pass `stop`'s filter; helper
    processes never reach `attribute`'s rows, so no `docker-proxy` is signalled.
  - The engine runs at most once per scan per engine, and only when it has ports; a scan with no
    container ports asks nothing.
  - The Podman path cannot run here (no Podman on the dev box): untried, sharing the Docker path.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/614-container-ports-in-the-rail.sh` under sway: a fake `docker` first
  on the PATH (answering `ps` with `web-614` on a free port, its Compose folder the scratch
  project, recording `stop`, refusing when its mode file says so) and a stand-in `docker-proxy`
  (Perl with `$0` set to `docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port <port>
  -container-ip 172.18.0.5 -container-port 80`). Positions measured from the first runs.
- **Shots and files, each read** (the last run):
  - `named.png` (REQ-001): under `repo`, ":<port> web-614", its URL and "container web-614";
    after the projects a CONTAINERS label with the dev box's own Docker ports, ":8081 docker →
    172.19.0.2:8081" and ":8083 docker → 172.18.0.2:8083", unnamed since the fake does not list
    them.
  - `hover.png`: the row's Open, Copy and Stop on hover; its tooltip "docker container web-614,
    to 172.18.0.5:80, proxy pid …".
  - `stopped.txt` (REQ-002): "stop web-614".
  - `apart.png` (REQ-003): the fake refusing, after the engine's answer aged out, the stand-in's
    port under CONTAINERS as ":<port> docker → 172.18.0.5:80", none under `repo`.
  - `refused.png` (REQ-003): Stop on it: its tooltip "Stop the Container, docker did not name it",
    and a toast "Could not stop the container: permission denied while trying to connect to the
    docker API at unix:///var/run/docker.sock. To stop it yourself, run: docker stop $(docker ps
    -q --filter publish=<port>)" with Copy Command.
  - REQ-004 is the review's (Phase 2).
- **Seen, not in scope:** in `apart.png` the Stop button's tooltip from the earlier hover still
  shows where the row was before it moved to CONTAINERS: gpui hides a tooltip on the pointer's
  next move, the class #618 handles for headers.
- **Reds:** none; no source changed in this phase.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after; the stand-in proxy was killed in `teardown`.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: container ports in the rail); `workbench-shell.md` (#614 in
  the shell's list); `marley_workbench.md` (a Container ports bullet in the Ports section); the
  guide, the guide page (the port rows article) and the walkthrough (a container ports check in
  2.8). No path outside the Marley-owned set changed.
- **Knowledge:** AD-claude-614-container-ports-come-from-the-proxys-command-line-and-the-engines-cli-001,
  PR-claude-614-a-port-of-a-container-engines-own-unit-is-never-stopped-through-it-001.
- **Brain:** consultation 8daed629f3ad45f29fb78e6f708ba898 closed with `brain decide`.

