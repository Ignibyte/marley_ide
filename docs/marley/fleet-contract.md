# The fleet contract: what Marley reads to show agents, their work and their hosts

Status: draft for review, 2026-09-30. Nothing here is built yet. This document fixes the outputs
Marley expects; the SDK that packages them (types, schemas, a conformance kit) comes later, and
until the Rustal services can serve them, Marley shows pseudo data (see [Pseudo data](#pseudo-data)).

## What this is

Marley treats outside systems the way an editor treats language servers: it defines what it
expects, any system that answers in that shape is shown, and Marley owns how it looks. A
provider sends data, never UI. Marley renders it natively, in GPUI, in its own panes.

Two contracts cover the fleet:

| Contract | Who serves it | What it holds |
|---|---|---|
| `marley.work/v1` | The central workflow store (rustal-brain first; any system that keeps the same records) | Agents, the work items they work on, runs of a workflow through its phases, gates, events, questions and token use |
| `marley.host/v1` | A collector script Marley runs on each host over SSH | The host's CPU, memory, disk, network and uptime, and the agent processes running on it |

Marley joins the two per agent. With only the store, a detail shows the work and no resources.
With only SSH access to a host, it shows the host's resources and the agents running there, with
no work records.

## Assumptions

- Work runs through workflows, and their records live in one central place. Agents write their
  progress there; Marley reads it and, later, sends actions through it.
- A workflow is an ordinary software lifecycle: an ordered list of phases. The phases can have
  any names (`plan, code, test, complete`, or `design, implement, inspect, validate`); Marley
  shows them in the order given and never interprets a name.
- An agent is a runtime (Claude Code, Codex, a script) running on a host, and for the most part
  an agent and its host are one thing: clicking the agent shows what it works on and what it
  uses. A host can be this machine, an agent box, or a VPS.
- An agent's host is reachable over SSH with the user's own keys when resources are wanted.

## The picture

```
 central workflow store ──(MCP or HTTP)──┐
   marley.work/v1                         │
                                          ├──► Marley ──► Fleet panel (right dock): the list, and the
 host collector, per host ──(SSH)────────┘              selected agent's snapshot
   marley.host/v1                                       ──► Agent tab (center): the full detail
```

## Conventions

- JSON, UTF-8. Field names are `snake_case`.
- Ids are opaque strings, stable for the thing's life. Marley never parses them.
- Times are integer milliseconds since the Unix epoch, UTC, in fields ending `_ms`. Durations
  are integer seconds in fields ending `_s`.
- Sizes are integer bytes (`_bytes`); rates are bytes per second (`_bps`); percentages are
  numbers from 0 to 100.
- A field marked optional may be left out or be `null`; Marley hides what it cannot show.
  Unknown fields are ignored, so a provider may send more.
- Versioning: the contract name carries its major version. Within `v1`, fields are only added
  and never change meaning; a change that breaks that is `v2`, and a provider may serve both.

## `marley.work/v1`: the central workflow store

### The calls

| Call | MCP tool | HTTP | Returns |
|---|---|---|---|
| Handshake | `work_handshake` | `GET /marley/v1/handshake` | [Handshake](#handshake) |
| List agents | `work_agents` | `GET /marley/v1/agents` | [AgentList](#agentlist) |
| One agent in full | `work_agent` (`{ "id": … }`) | `GET /marley/v1/agents/{id}` | [AgentDetail](#agentdetail) |
| One run in full | `work_run` (`{ "id": … }`) | `GET /marley/v1/runs/{id}` | [Run](#run) |
| What changed | `work_changes` (`{ "after": … }`) | `GET /marley/v1/changes?after=…` | [Changes](#changes) |

Over MCP, each tool answers in `structuredContent` with the object as its value (the text
content may carry the same JSON). Since #611 Marley reads a store named in its settings'
`marley.fleet.providers`: `{ "kind": "mcp", "command": …, "args": […] }`, `{ "kind": "mcp",
"url": … }` or `{ "kind": "http", "url": … }`, each with an optional `name` and `bearer_env`.
It uses `work_changes` when the handshake's capabilities name `changes`, and does not call
`work_run` yet: the detail's run is what the panel and the Agent tab draw. Over HTTP, each answers `200` with the object as the body.
Auth is the provider's own: an MCP server's configured bearer, or an HTTP `Authorization`
header from Marley's settings. Marley reads only, in `v1`; actions come in a later version
(see [Later](#later)).

### Handshake

```json
{
  "contract": "marley.work/v1",
  "provider": { "name": "rustal-brain", "version": "0.4.0" },
  "capabilities": ["agents", "work_items", "runs", "gates", "events", "usage", "questions", "hosts"],
  "poll_s": 5,
  "stale_after_s": null
}
```

- `capabilities` says which parts the provider fills. A part it leaves out is hidden in
  Marley, not shown empty. The names are fixed by this document; an unknown name is ignored.
- `poll_s` (optional) is how often Marley should ask for changes; Marley's default is 5.
- `stale_after_s` (optional): Marley decides when an agent is stale, from `last_seen_ms`, after
  three polls with no sign of it. A provider whose agents report less often sets this to say how
  long a quiet spell is still normal.

### AgentList

What the Fleet panel's list needs, for every agent, cheaply.

```json
{
  "agents": [
    {
      "id": "agent-7f3a",
      "name": "build-1",
      "runtime": "claude-code",
      "state": "working",
      "state_since_ms": 1759240810000,
      "last_seen_ms": 1759243100000,
      "host_id": "host-build-1",
      "work_item": { "id": "wi-142", "key": "RB-142", "title": "Split the pipeline module" },
      "phase": { "name": "code", "index": 1, "count": 4 },
      "attention": "none",
      "question": null
    }
  ],
  "cursor": "c-1843"
}
```

| Field | Meaning |
|---|---|
| `id`, `name` | The agent's id, and the name Marley shows |
| `runtime` | What runs: `claude-code`, `codex`, `script`, or any other name, shown as given |
| `state` | `starting`, `working`, `idle`, `waiting`, `error` or `done`, the states Marley's fleet already uses |
| `state_since_ms` | When it entered that state |
| `last_seen_ms` | The last time the agent reported anything; Marley marks it stale after a quiet spell |
| `host_id` | Optional: the host it runs on, which joins the host collector's data |
| `work_item` | Optional: a short form of what it works on |
| `phase` | Optional: its run's current phase, by name, its place and the phase count |
| `attention` | `none`, `question` (waits on a person), `failed` (a gate or run failed) |
| `question` | Optional: a [Question](#question) when it waits |
| `cursor` | Where [Changes](#changes) continues from |

### AgentDetail

The snapshot in the Fleet panel and the Agent tab read this.

```json
{
  "agent": { "id": "agent-7f3a", "name": "build-1", "runtime": "claude-code", "state": "working",
             "state_since_ms": 1759240810000, "last_seen_ms": 1759243100000,
             "host_id": "host-build-1", "cwd": "/srv/work/pipeline", "model": "claude-opus" },
  "work_item": {
    "id": "wi-142", "key": "RB-142", "title": "Split the pipeline module",
    "status": "in progress", "url": "https://brain.example/tickets/RB-142",
    "summary": "Move dispatch into its own crate and keep the public API."
  },
  "run": { "id": "run-88", "…": "a Run, as below" },
  "recent_events": [
    { "at_ms": 1759243000000, "kind": "gate", "text": "cargo check green" },
    { "at_ms": 1759242400000, "kind": "phase", "text": "code started" }
  ],
  "usage": {
    "run":   { "input_tokens": 812000, "output_tokens": 64000, "cache_read_tokens": 2300000 },
    "today": { "input_tokens": 3900000, "output_tokens": 310000 }
  },
  "question": null,
  "host": null
}
```

- `agent` is the list entry plus `cwd` and `model` (both optional).
- `work_item`: `key` is the short name people use; `status` is the store's own word, shown as
  given; `url` opens in a Browser tab.
- `run` is the agent's current [Run](#run), or `null`.
- `recent_events`: the newest first, at most 50; the Agent tab asks `work_run` for the rest.
  `kind` is free text (`phase`, `gate`, `commit`, `message`, `tool`), shown as a small label.
- `usage`: the tokens the agent's model used, for the current run and for the day. Every field
  is optional. Usage is reported by the agent to the store, which knows it from its runtime.
  Money is left out of `v1` on purpose (Chad, 2026-09-30); a later minor version can add a
  `cost` beside the tokens without breaking a reader.
- `host`: optional, a [HostSnapshot](#hostsnapshot) when the store keeps one (a collector may
  push to the store instead of being read over SSH). When `null`, Marley reads the host itself.

### Run

One pass of a work item through its workflow.

```json
{
  "id": "run-88",
  "work_item_id": "wi-142",
  "workflow": { "id": "wf-std", "name": "Standard" },
  "agent_id": "agent-7f3a",
  "started_ms": 1759236000000,
  "ended_ms": null,
  "outcome": null,
  "phases": [
    { "name": "plan", "state": "passed", "started_ms": 1759236000000, "ended_ms": 1759238000000,
      "summary": "Spec and design approved", "gates": [] },
    { "name": "code", "state": "active", "started_ms": 1759238000000, "ended_ms": null,
      "summary": null,
      "gates": [ { "name": "check", "state": "pass", "detail": null },
                 { "name": "clippy", "state": "pending", "detail": null } ] },
    { "name": "test", "state": "pending", "started_ms": null, "ended_ms": null, "summary": null, "gates": [] },
    { "name": "complete", "state": "pending", "started_ms": null, "ended_ms": null, "summary": null, "gates": [] }
  ],
  "events": [ { "at_ms": 1759243000000, "kind": "gate", "text": "cargo check green" } ]
}
```

- `phases` are in the workflow's order. A phase's `state` is `pending`, `active`, `passed`,
  `failed` or `skipped`; exactly one is `active` while the run is under way.
- A gate's `state` is `pending`, `pass`, `fail` or `skipped`; `detail` is a short line for a
  failure.
- `outcome` is `null` while it runs, then `passed`, `failed` or `abandoned`.
- `events` is the run's whole log, the newest first.

### Question

```json
{ "id": "q-19", "prompt": "Delete the old dispatch module?", "options": ["Yes", "No"],
  "asked_ms": 1759243050000 }
```

Marley shows it; answering is a later version's action.

### Changes

```json
{ "cursor": "c-1851", "reset": false,
  "changed": [ { "kind": "agent", "id": "agent-7f3a" }, { "kind": "run", "id": "run-88" } ] }
```

Marley asks with the last cursor it holds, then fetches what changed. `reset: true` means the
provider lost the cursor, and Marley reads the list again from the start. A provider that has no
change feed leaves out `work_changes`, and Marley reads `work_agents` every `poll_s` instead.

## `marley.host/v1`: the host collector

A POSIX `sh` script Marley ships, which prints one JSON document and exits. Marley runs it on the
host over SSH with the user's own configuration, the script carried in the command (`printf %s
<base64> | base64 -d | sh`, as #526's shell bootstrap travels), every 5 seconds while a Fleet
surface shows; nothing is installed there. Since #610 the script is
`crates/marley_workbench/bin/marley-collect.sh`, and the hosts are the settings'
`marley.fleet.hosts`. It reads `/proc` and
`df`, needs no root and opens no port, and the same script runs locally for this machine. Any
other collector that prints the same document works, and one may instead push it to the
workflow store, which then returns it as an agent detail's `host`.

### HostSnapshot

```json
{
  "contract": "marley.host/v1",
  "host": { "id": "host-build-1", "name": "build-1", "os": "Linux 6.12", "cores": 8,
            "uptime_s": 1209600 },
  "sampled_ms": 1759243100000,
  "cpu": { "percent": 42.0 },
  "load": [1.2, 0.9, 0.7],
  "memory": { "used_bytes": 3100000000, "total_bytes": 8000000000 },
  "disks": [ { "mount": "/", "used_bytes": 41000000000, "total_bytes": 80000000000 } ],
  "network": { "rx_bps": 120000, "tx_bps": 45000 },
  "agents": [
    { "pid": 4121, "runtime": "claude-code", "cwd": "/srv/work/pipeline",
      "started_ms": 1759236000000, "cpu_percent": 18.5, "rss_bytes": 420000000,
      "session": "agent-7f3a" }
  ]
}
```

- `cpu.percent` and `network` are measured over the interval since the collector's previous
  run (it keeps its last counters in a file under `$XDG_RUNTIME_DIR`, else `/tmp`); on a first
  run it samples twice, a second apart.
- `agents` lists the processes the collector recognizes as agent runtimes, by name and command
  line: `claude`, `codex`, and whatever its configuration adds. `session` is optional: an id the
  runtime exposes (an environment variable or an argument), which matches the store's agent id.
- Marley joins a store agent to a process by `session`, else by `host_id` with `cwd` and
  `runtime`. A process no store agent claims still shows, as an agent with no work records.
- Marley's collector reads a process's session from its `MARLEY_FLEET_SESSION` environment
  variable, when the process's environment can be read (#610): a harness or store that starts an
  agent sets it to the agent's id. Without a session and with no folder to match, a process
  joins an agent only when its host has exactly one agent and one process of that runtime.

## What Marley shows

| Where | What | From |
|---|---|---|
| Fleet panel, right dock | Every agent, grouped by host: name, runtime, state chip, work item key, phase `2/4`, and an attention mark | `work_agents`, host snapshots |
| Fleet panel, below the list | The selected agent's snapshot: state and for how long, the work item, the phase strip, CPU and memory bars, tokens today, the question when there is one | `work_agent`, the host snapshot |
| Agent tab, center | The full detail: the phase timeline with each gate, the event log, resource history graphs (Marley keeps the samples it read), usage for the run and the day, the host's other agents | `work_agent`, `work_run`, host snapshots over time |

One click on an agent selects it and fills the snapshot; a double-click, Enter or Open opens its
Agent tab, as a port row in the rail does (#604). The rail on the left stays per-project; a
project row may later carry a mark when an agent works in that folder.

Each source shows its own state rather than empty panes: *not set up* (no provider configured),
*connecting*, *unreachable* (with the reason), *stale* (no answer for three polls),
*incompatible* (a contract version Marley does not read). An agent whose `last_seen_ms` is older
than three polls (or the provider's `stale_after_s`) reads *stale*, and one whose host is
unreachable reads *offline*.

## Pseudo data

Until the Rustal services serve these contracts, Marley ships a pseudo provider for both, so the
panes can be built, shown and tested:

- A setting selects it: `"marley": { "fleet": { "providers": [{ "kind": "pseudo" }] } }`.
- It serves three agents on two hosts, with work items, runs in different phases, one waiting
  on a question, and one failed gate. Every few seconds it moves: CPU and network wander, events
  arrive, and a run advances a phase.
- Its data is the JSON in this document, kept as fixture files beside the future SDK, so the
  same examples are the contract's documentation, the pseudo provider's data and, later, the
  conformance kit's cases.
- Since #607 the types and the pseudo provider are `crates/marley_sdk` (`work`, `host`, `stale`
  and `pseudo`), with the fixtures in its `fixtures/`. Its handshake says `poll_s: 2`, so the
  demo moves while you watch; the quiet agent reads stale about 18 s after the first reading.

## Security

- Marley reads only, in `v1`. It never passes a provider's bearer into a log, a prompt or a
  file other than the user's settings.
- SSH uses the user's own keys and configuration; Marley stores no host credentials.
- The collector runs as the user, needs no root, and prints only what is listed here: no
  command lines beyond the runtime's name and working folder, no environment values.

## Later

- Actions: answer a question, stop or pause an agent, start one on a host (`work_action`), under
  a write grant as Marley's own MCP server grants writes.
- The live terminal of an agent that runs in the harness, shown in its Agent tab (plan D10).
- The SDK: the contract's types in a `marley_sdk` crate, JSON Schema for other languages, the
  fixtures, and a conformance kit a provider runs against itself.
- A fleet board in the center: every agent as a tile.
- More contracts in the same pattern, such as the work items board of a whole project.

## Settled (Chad, 2026-09-30)

1. Marley decides when an agent is stale, from `last_seen_ms` after three missed polls; a
   provider may widen that with `stale_after_s`.
2. The collector is a script Marley pipes over SSH, not a binary installed on the host.
3. Tokens only: money is out of `v1`.
