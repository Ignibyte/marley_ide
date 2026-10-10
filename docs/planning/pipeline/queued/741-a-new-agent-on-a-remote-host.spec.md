---
pipeline_id: 27331002-51b1-4f70-939b-f1c56411a11a
ticket: docs/planning/tickets/open/TICKET-741-a-new-agent-on-a-remote-host.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: A new agent on a remote host
type: feature
slice: the control plane (docs/marley/three-prong-plan.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/735-the-new-agent-picker.spec.md
  - docs/planning/pipeline/queued/740-one-harness-per-host.spec.md
---

## Title
New Agent starts a CLI agent on another machine through the harness there: a supervised seat on
that host, with a terminal attached to it here. Chad, 2026-10-10: "Bonus points if we can open on a
remote via the harness."

## Scope
### In
- The New Agent picker's Where (#735) lists, after the local choices, **On <harness>…** for each
  harness #740 follows (this machine's included). Choosing one asks for a folder on that host, an
  absolute path typed in the picker; the harness's own refusal (`rh: CODE: reason`) shows if the
  folder is wrong.
- For Claude Code or Codex (the agents a seat runs), Marley runs `seat add NAME --agent A --cwd DIR`
  and `seat start NAME` through that harness's commands (#691's form, without the form). The name is
  `<agent>-<folder name>`, with `-2`, `-3` … when the root has it.
- Once the seat starts, a terminal of the shown group runs the harness's attach view for its
  workspace (`rh … attach WORKSPACE`, from `session_surface_to_human`), through `ssh -t HOST` for a
  remote harness, with the SSH keepalive Marley's remote terminals use (#641).
- The session tab's Views → Open (#690) runs through `ssh -t HOST` for a remote harness too, which
  the guide says it does not yet; that line goes.
- `docs/marley/guide.md`: an agent on another host.

### Out (explicitly deferred)
- A thread (not a terminal) on a remote seat: #742.
- Agents other than Claude Code and Codex on a harness: the harness runs only those two.
- Browsing the remote host's folders: the folder is typed.

## Reference (§20)
N/A — Marley-specific: the harness's seats (rustal-harness TICKET-109, `rh seat add`, `rh seat
start`, `rh attach`) and Marley's seat form (#691) and remote terminals (#543, #641). No Zed or Warp
behavior covers starting a supervised agent on another machine.

### Prior art
- **The code we ship:**
  - `harness_seat.rs` `Seat::commands` (80-98) and `run_seat` (482-505): the seat commands and how
    their JSON or `rh: CODE: reason` is read.
  - `harness.rs` `open_view` (1459-1479): types a view's argv into a new local terminal through
    `agents::start_in_terminal`; the argv comes from the harness as `[<rh on the host>, --state,
    ROOT, attach, WORKSPACE]` (`rustal-harness/crates/harness-runtime/src/mcp.rs:1225-1258`).
  - `marley_remote::remote_terminal_command` (`marley_remote.rs:391-418`): the SSH keepalive and
    connect timeout options, and its note that every word must be plain, since ssh joins them for
    the remote shell; a view's argv words are quoted for the remote shell here.
  - #740's harness entries, which name the host apart from the rest of the command.
- **Behavior maps:** `docs/orca_architecture/04-remote-control-and-mobile.md`: Orca's SSH worktrees
  keep a daemon on the remote that outlives a disconnect; the harness's seats and their tmux play
  that part here.
- **Published material:** OpenSSH `-t` and the remote command line.

## UI proof
`script/e2e/741-a-new-agent-on-a-remote-host.sh`, under `compositor sway`. `marley.harnesses` names
`box-2` with `ssh: box-2`; a fake `ssh` on the PATH drops the host, logs its arguments and runs the
rest here; a stand-in `rh` answers `seat add` and `seat start` with the harness's JSON (workspace
`ws-claude-other`), and `attach WORKSPACE` by printing `attached: WORKSPACE` and waiting.

Shots:
- `741-01-where`: the picker's Where with **On box-2…** after the local choices.
- `741-02-attached`: after `/srv/other` typed: a terminal in the shown group showing
  `attached: ws-claude-other`; the fake `ssh` log holds `seat add claude-other --agent claude
  --cwd /srv/other`, `seat start claude-other` and `-t box-2 … attach ws-claude-other`.
- `741-03-view`: the seat's session tab → Views → Open: a terminal running the view through
  `ssh -t box-2`.

## Locked-In Decisions
- **D1:** a remote agent is a harness seat, never a bare `ssh HOST claude`: the harness supervises
  it and it outlives the link (D164, TICKET-109).
- **D2:** the attach runs through `ssh -t` directly, not inside Marley's own tmux on the host: the
  harness's tmux holds the session, and tmux refuses to attach from inside another tmux.
- **D3:** the seat's name is made from the agent and the folder, so the rail and `rh` show
  something readable.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE harnesses are followed, the New Agent picker's Where shall list On <harness>… for each. | Shot 741-01 |
| REQ-002 | WHEN a CLI agent is started on a remote harness with a folder, the system shall add and start a seat there through that harness's commands. | The fake ssh log |
| REQ-003 | WHEN the seat has started, the system shall open a terminal in the shown group attached to it through `ssh -t HOST`. | Shot 741-02 and the fake ssh log |
| REQ-004 | WHEN a remote session's Views → Open is chosen, the system shall run the view through `ssh -t HOST`. | Shot 741-03 |
| REQ-005 | IF the harness refuses the seat, THEN the system shall show its reason and open no terminal. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `agents.rs` (Where's harness choices), `harness_seat.rs` (seats without the form),
  `harness.rs` (`open_view` through ssh), the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
