---
pipeline_id: b6f8f86e-f859-4dbc-bbdd-f2b61ef28a75
ticket: docs/planning/tickets/open/TICKET-614-container-ports-in-the-rail.md
status: Phase 4 — Complete PASS
title: "Container ports in the rail"
type: feature
slice: workbench shell, the rail's ports; after #521 and #603
references: [docs/planning/pipeline/completed/603-service-aware-port-rows.spec.md, docs/planning/pipeline/completed/521-ports-per-project.spec.md]
---

## Title
A port a Docker or Podman container publishes is listed in the rail, named by its container when
Marley can ask the engine, and Stop stops the container, never the engine.

## Scope
### In
- **Finding them without the engine:** Docker's published ports belong to root's `docker-proxy`,
  whose sockets the scan cannot tie to a process. Its command line is readable by everyone and
  names the host port and the container's address and port, so the scan reads `docker-proxy`
  processes' command lines and lists each as a container port. Rootless Podman's helpers
  (`rootlessport`, `pasta`, `slirp4netns`) run as the user; a port they hold is a container
  port, not a process to signal.
- **Naming them with the engine:** where `docker ps` or `podman ps` answers (the user can reach
  the socket), the port is named by its container, and a container whose Compose label
  `com.docker.compose.project.working_dir` is inside a project's folder is listed under that
  project; the rest go under a Containers section at the rail's end (its rows open, copy and stop
  as a port row does; the keys and the filter pass them by in this ticket). Where the engine refuses,
  the port is listed under Containers as "container port 8081 → 172.17.0.2:80".
- **Stop:** `docker stop <container>` or `podman stop <container>` through the workbench's spawn
  module. Without engine access, Stop is refused with the reason and Copy Command, as #603's
  refusals are. A listener whose service would be `docker.service` or `podman.service` is never
  stopped through systemd.
- The engine is asked once per scan at most, and only while the rail watches ports (#521).

### Out (explicitly deferred)
- Container logs, restart, and starting containers.
- Remote engines, Kubernetes, and ports a container exposes without publishing.
- Granting access to the Docker socket.

## Reference (§20)
N/A — Marley-specific: the rail's port rows are Marley's (#521, #603). Docker's and Podman's CLIs
are the published interface read (`docker ps --format '{{json .}}'`, `podman ps --format json`).

### Prior art
- **Behavior maps:** #603's service rows and refusals (AD-claude-603, L-claude-603).
- **Published material:** the Docker and Podman CLI references for `ps --format` and `stop`; the
  Compose labels (`com.docker.compose.project.working_dir`).
- **Code we already ship:**
  - `marley_browser::ports::listeners_in` (`ports.rs:149-212`) and `service_of` (104).
  - `marley_workbench::ports` (`attribute`, `stop`, `Stop::Refused`, `hand_command`).
  - `process::output`, the workbench's one spawn module (gate:22).
  - No crate in `Cargo.lock` talks to Docker; the CLI is the route.
- **Found on the dev box (read-only):** ports 8081 and 8083 belong to root's `docker-proxy`
  (`-use-listen-fd`), whose cgroup is `system.slice/docker.service`; the user is not in the
  `docker` group, so `docker ps` is refused. Podman is not installed.

## UI proof
The scenario `script/e2e/614-container-ports-in-the-rail.sh` (sway) puts fakes first on the PATH
(L-claude-603): a `docker` that answers `ps --format` with a container whose Compose working
folder is the scratch project, and records `stop`; and a process named `docker-proxy` with the
real command line's shape (Perl with its `$0` set to it). The dev box's own Docker ports (root's
real `docker-proxy`) show too, under Containers, unnamed, since the fake engine does not list
them. Shots:
- `named.png`: the port under the project, named by its container;
- `stopped.txt`: Stop ran `docker stop <container>`;
- `refused.png`: with the fake `docker` refusing, the port under Containers with its address,
  and Stop refused with Copy Command.

## Locked-In Decisions
- D1 — The engine's CLI through `process::output`; no Docker API crate.
- D2 — Stop never goes through systemd for a container engine's own unit.
- D3 — Without engine access the port is still listed, from `docker-proxy`'s command line.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a container publishes a port and the engine answers, the rail shall list the port named by its container, under the project its Compose folder is in. | Shot `named.png` |
| REQ-002 | WHEN the user stops a container port, Marley shall stop the container through the engine. | `stopped.txt` |
| REQ-003 | WHILE the engine refuses, the rail shall list the port from `docker-proxy`'s command line, and Stop shall be refused with its reason and the command. | Shot `refused.png` |
| REQ-004 | Marley shall never stop `docker.service` or `podman.service` from a port row. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the scan's container pass, the engine call's cadence).
- **P2 Code** — the scan, the naming, Stop; a review; the gate green.
- **P3 Test** — the scenario with its fakes, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
