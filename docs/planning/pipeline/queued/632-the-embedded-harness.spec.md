---
pipeline_id: 2a499c5f-1dcb-4a56-a079-747874a5942e
ticket: docs/planning/tickets/open/TICKET-632-the-embedded-harness.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The embedded harness: Marley starts the harness's runtime itself"
type: feature
slice: prong 2 C1, plan D19 (after #534)
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/534-harness-sessions-in-the-rail.spec.md]
---

## Title
With `marley.embedded_harness` on, Marley finds `rh`, starts `rh --state <data dir>/harness serve`
as a process of its own, and follows it through `rh --state <root> mcp` with #534's section; the
section's header also says whether the runtime runs, since `rh mcp` reads the journal without it.

## Scope
### In
- **The setting:** `marley.embedded_harness: Option<bool>`, off by default: `rh` is not shipped
  with Marley (the harness has no install path and no license yet), so the user turns it on where
  `rh` is installed. `marley.harness`, when set, wins: Marley then starts nothing (#534's path).
- **Finding `rh`:** `MARLEY_RH`, else `rh` on the search path Marley's agents use, as Chromium is
  found (`MARLEY_CHROMIUM`, then the system's); `rh compatibility` (it reads no state) confirms the
  binary.
- **The root:** `<data dir>/harness`, which `serve` creates at mode 0700; its socket's path must
  stay under 108 bytes, so a longer root is refused with that reason.
- **The runtime:** `rh --state <root> serve` started through Marley's spawn site (`process.rs`,
  gate:22's list) as a child, its JSON ready line (`{"ready": true, …}`) awaited for 10 s; then the
  section follows `rh --state <root> mcp` as #534 does.
- **Its state in the header:** connected; starting; `runtime stopped:` and the child's exit or its
  last stderr line, when the child exits, with the rows kept and marked stale; `another runtime owns
  this root` when the lock is taken. A stopped runtime is started again after 1, 2, 4 … 60 s.
- **Stopping:** turning the setting off, or quitting Marley, ends the child; the harness's tmux
  server and its sessions live on, and the next `serve` on the root reconnects to them. Marley never
  runs `shutdown --stop-backend`.
- `script/e2e/632-the-embedded-harness.sh`.

### Out (explicitly deferred)
- Shipping `rh` inside Marley's install (`just install`): the harness has no install target or
  license; when it has a release path, a later ticket packages it.
- A `systemd-run --user` unit for the runtime (`serve --service-id` exists): a child first, as the
  smallest step; the browser's unit is the precedent if sessions should outlive a crash of Marley.
- Write verbs (C4), the live terminal (intake `harness-session-live-terminal.md`).

## Reference (§20)
N/A — Marley-specific: the harness is Ignibyte's own runtime, and nothing in Warp or Zed runs one.
Upstream Zed's `context_server` client stays as #534 uses it.

### Prior art
- **Behavior maps:** plan D19 ("Embedded, Marley runs `rh` as a process of its own, never linked
  in … speaks to it over `rh mcp` … one client serves both"); AD-claude-534.
- **Published material:** the harness's own docs: `TERMINALS.md` (the root's rules, `serve` in
  the foreground, `shutdown` refusals, a later `serve` reconnecting to tmux), `MCP.md` (`rh mcp`
  reading the journal while `serve` is down), `RELEASE.md` (`rh compatibility`), `SERVICES.md`
  (`serve --service-id` for a unit).
- **Code we already ship:** `marley_workbench::harness` (#534: the follow loop, the section, the
  backoff); `marley_browser::service` (Chromium found by `MARLEY_CHROMIUM` then the host, started as a
  unit); `marley_workbench::process` (the listed spawn site: a child whose stdout is piped and which
  dies with its handle); `mcp.rs` (programs written under the data dir at start).

## UI proof
`script/e2e/632-the-embedded-harness.sh` (`compositor sway`): `MARLEY_RH` pointing at the
harness's built `bin/rh`, `marley.embedded_harness` on, the run's own data dir. Shots:
- `632-01-connected`: the Harness section connected, no sessions;
- `632-02-session`: `rh --state <the run's root> actor new` from the scenario: its row;
- `632-03-stopped`: the `serve` child killed: `runtime stopped`, the row kept, stale;
- `632-04-back`: Marley's restart of `serve`: connected, the actor's row back (tmux kept it);
- `632-05-missing`: a second run with `MARLEY_RH` unset and no `rh` on the path: the header says
  `rh` was not found and where Marley looked.
Teardown stops the root's workspaces and runs `shutdown --stop-backend`.

## Locked-In Decisions
- D1 — Opt-in, and `marley.harness` wins: one setting names a harness Marley follows, the other
  lets Marley run one.
- D2 — A child process, never linked in, through the listed spawn site.
- D3 — The header follows the child as well as `rh mcp`, since `rh mcp` answers while `serve` is
  down.
- D4 — Marley never stops the harness's sessions: quitting ends `serve` only.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.embedded_harness` is on and `marley.harness` unset, WHEN Marley starts with `rh` found, the system shall start `rh serve` on its own root and show the Harness section connected. | Shot `632-01-connected` |
| REQ-002 | WHEN a session starts on that root, the section shall list it. | Shot `632-02-session` |
| REQ-003 | IF the runtime exits, THEN the header shall say so with the reason, keep the rows marked stale, and Marley shall start the runtime again with a growing delay. | Shots `632-03-stopped`, `632-04-back` |
| REQ-004 | WHERE `rh` cannot be found, the header shall say where Marley looked. | Shot `632-05-missing` |
| REQ-005 | WHERE `marley.harness` is set, the system shall start no runtime of its own. | Review; #534's scenario |
| REQ-006 | WHEN the setting is turned off or Marley quits, the system shall end the runtime and leave the harness's sessions running. | Review; the teardown finds the actor's workspace running |

## Phase Plan
- **P1 Plan** — promote, re-read the harness's `TERMINALS.md` and `MCP.md`, check the spawn site.
- **P2 Code** — the setting, finding `rh`, the child, the header's states; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
