---
pipeline_id: 9c3c794a-1c33-436c-b2a6-c2a80cf715bc
ticket: docs/planning/tickets/open/TICKET-808-tmux-sessions-in-the-rail.md
status: Phase 4 — Complete PASS
title: tmux sessions in the rail
type: feature
slice: the rail (#504) and remote terminals (#543, #641)
references:
  - docs/planning/pipeline/completed/543-remote-terminals-survive-a-drop.spec.md
  - docs/planning/pipeline/completed/641-ssh-links-that-know-they-are-dead.spec.md
  - docs/planning/pipeline/completed/740-one-harness-per-host.spec.md
---

## Title
For each host `marley.tmux_hosts` names (an `ssh_connections` entry, or `local` for this
machine), the rail lists the tmux sessions on that host's own tmux server under a heading of its
own. A click attaches a session in a terminal tab, a session running an agent CLI shows that
agent's icon and name, and the heading's + starts a new session there. Chad, 2026-10-17: "attaching
to existing tmux agents or an ssh server that we can connect to has tmux it starts a new session".

## Scope
### In
- `marley.tmux_hosts: Option<Vec<String>>`: `local`, or an `ssh_connections` entry's nickname or
  host. Unset or empty: no heading, no listing, nothing run.
- A listing every 10 seconds per host: `tmux list-panes -a` with a format, run here or behind
  `ssh -T -o BatchMode=yes` with Marley's keepalive and connect timeout; 20 seconds at most.
- A heading per host, after the harness sections: `TMUX · this machine` or `TMUX · <name>`, how
  many sessions or why none could be read, a + button; a click on the heading folds its rows.
- A row per session: the agent CLI's icon and name when a pane runs one (`agent_kind_of` on the
  pane's command), else a terminal icon; the windows' count; `attached` when a client has it.
- A click on a row: a terminal tab attaches it (`tmux attach-session -t =NAME`, behind `ssh -t`
  for a host). A tab of that attach that still runs is shown instead of a second one.
- The +: a terminal tab runs `tmux new-session -A -s marley-<8 hex>` on the host's own server, and
  the host is listed again a second later.
- A host's attach and new session ride #641's link supervision: the task id base starts with
  `marley-remote`, and the host goes into `Links`, so a dropped link dims the tab and reattaches.

### Out (explicitly deferred)
- Killing, renaming or detaching a session from the rail.
- tmux servers other than the default one (`-L`, `-S`), and #543's own `-L marley` sessions.
- Settings UI entry: the setting is a list, written in `settings.json`, as `marley.harnesses` is.
- Hosts from `~/.ssh/config` that `ssh_connections` does not name.

## Reference (§20)
N/A — Marley-specific: Zed has no tmux integration and no rail; the rail section follows
Marley's own harness sections (#534, #740), and the attach follows #543's remote terminal.

### Prior art
- The code we ship: `remote.rs` (#543, #641) already opens a task terminal behind `ssh -t` with
  link supervision; `harness_hosts.rs` (#740) reads hosts from settings, quotes remote words
  (`shell_word`) and builds `BatchMode` ssh arguments; `process::output` runs a program without a
  new spawn site; `marley_agent::agent_kind_of` and `agents::cli_icon` name an agent CLI by its
  program; `RemoteSettings::ssh_connections` lists the saved hosts.
- tmux's own formats (`tmux(1)`, FORMATS): `list-panes -a -F` gives every pane's session, its
  windows' count, its clients' count and its foreground command. Checked on tmux 3.7c: Claude
  Code's pane reports `claude`, Codex's `codex`; no server gives "no server running" or "error
  connecting to", exit 1.
- Zed's `terminal_panel::spawn_task`: a task that may not run twice waits for the running one,
  so a second click would show nothing; the module finds the running tab first.

## UI proof
Scenario `script/e2e/808-tmux-sessions-in-the-rail.sh` (`compositor sway`): two tmux servers of
the run's own (`TMUX_TMPDIR`), one as this machine and one as the host `box-2` behind a fake ssh
(`MARLEY_SSH`); a second saved host whose ssh fails, and a name no saved host has.
- `808-01-off`: the setting unset, no TMUX heading (REQ-006).
- `808-02-rail`: four headings; this machine's `build` (terminal icon) and `agent` (Claude
  Code's icon and name); `box-2`'s `deploy` (Codex); the failed host's ssh words; the unknown
  name's words (REQ-001, REQ-003, REQ-005).
- `808-03-attached`: `build` attached in a tab named `this machine · build`, its text on screen
  (REQ-002).
- `808-04-again`: a second click: the same tab, no second one (REQ-002).
- `808-05-remote`: `deploy` attached in a tab `box-2 · deploy` (REQ-002).
- `808-06-new`: the +: a tab on `box-2`, and the new session's row under `box-2` (REQ-004).

## Locked-In Decisions
- D1 — The setting names hosts by their `ssh_connections` entry, so the rail, Open Remote and the
  remote terminal share one list; `local` names this machine.
- D2 — The host's own default tmux server, with the user's config; nothing of Marley's is set on
  it.
- D3 — Listing through `process::output`, so the spawn sites and their pin stay where they are.
- D4 — Every word after the ssh destination goes through `harness_hosts::shell_word`
  (PR-claude-740-quote-each-word-after-an-ssh-destination-001).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.tmux_hosts` names a host, the rail shall show a heading for it after the harness sections, with a row for each tmux session on that host's default server. | `808-02-rail` |
| REQ-002 | WHEN the user clicks a session's row, Marley shall open a terminal tab attached to that session, or show the tab of that attach that still runs. | `808-03-attached`, `808-04-again`, `808-05-remote` |
| REQ-003 | WHERE a pane of a session runs an agent CLI Marley knows, the session's row shall show that agent's icon and name. | `808-02-rail` |
| REQ-004 | WHEN the user clicks a heading's +, Marley shall open a terminal tab in a new session on that host, and its row shall appear under the heading. | `808-06-new` |
| REQ-005 | IF a host cannot be listed, THEN its heading shall say why, in the error color. | `808-02-rail` |
| REQ-006 | WHILE `marley.tmux_hosts` is unset or empty, the rail shall show no TMUX heading and Marley shall run no listing. | `808-01-off`, the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the setting, `tmux_sessions.rs`, `remote.rs`'s shared launch, the rail's
  section; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, guide, architecture note, walkthrough stop (§21), ledger capture
  (§19), close the ticket, archive, commit.
