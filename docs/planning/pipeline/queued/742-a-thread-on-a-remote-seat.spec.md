---
pipeline_id: 84d0f12b-e679-42fc-8f9c-4d54af5aee57
ticket: docs/planning/tickets/open/TICKET-742-a-thread-on-a-remote-seat.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: A thread on a remote seat
type: feature
slice: the control plane (docs/marley/three-prong-plan.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/741-a-new-agent-on-a-remote-host.spec.md
  - docs/planning/pipeline/completed/694-the-manager-in-the-agent-panel.spec.md
---

## Title
A harness seat on any host as a thread tab: what you type goes to the seat as a message, and its
replies come back as the thread, over the harness's ACP window. Today `rh acp` opens onto the
root's manager only (#694, rustal-harness TICKET-111); rustal-harness TICKET-116 opens it onto any
seat, and this ticket uses it.

## Scope
### In
- The New Agent picker, with **On <harness>…** and a thread chosen (#741's Where), makes the seat
  as #741 does, then opens a thread tab (#734) whose agent is
  `[ssh HOST] rh --state ROOT acp --seat NAME`: a custom agent server Marley adds to the settings'
  in-memory defaults for that seat, as the Marley entry is added (#683).
- The thread's folder is the seat's folder on its host. No hidden worktree is made: the seat works
  on its own machine, and the harness's ACP window reads no file through Zed.
- The Threads page (#737) lists the thread with its host's name beside the folder.
- An existing seat in the rail's Harness section gets **Open as Thread** in its menu.
- `docs/marley/guide.md`: a thread on a seat.

### Out (explicitly deferred)
- The harness side, `rh acp --seat`: rustal-harness TICKET-116, built in that repository first.
- Restoring such a tab after a restart beyond what #736 gives (the seat's entry is made again when
  its harness is followed).

## Reference (§20)
N/A — Marley-specific over the harness's `rh acp` (rustal-harness TICKET-111 and TICKET-116) and
ACP. Upstream Zed's part is the custom agent server and the thread view, used as #694's Manager
entry uses them.

### Prior art
- **The code we ship:**
  - #694's Manager entry (`harness.rs`): a custom agent server running the harness's `rh acp` with
    the command Marley follows the harness with, ending in `acp`.
  - `assistant.rs` `set_entry`: a custom agent server added to the settings' in-memory defaults.
  - #734's `thread_tab::start` with explicit `work_dirs` and no worktree.
- **The harness:** `crates/harness-runtime/src/acp.rs` (`session/new`, `prompt`, `progress`), the
  seat's message path `session_send` → `deliver` and `delivery_state` (`mcp.rs:589, 811, 1141`).
- **Behavior maps:** none.
- **Published material:** ACP (`session/new`, `session/prompt`, `session/update`).

## UI proof
`script/e2e/742-a-thread-on-a-remote-seat.sh`, under `compositor sway`. `box-2` through the fake
`ssh` of #741, on a root of the built `rh` with TICKET-116, and a stand-in seat agent that answers
each delivered message.

Shots:
- `742-01-thread`: New Agent → Claude Code (thread) → On box-2… → a folder: a thread tab in the
  shown group; the fake `ssh` log holds `acp --seat claude-…`.
- `742-02-reply`: "hello" typed: the seat's reply in the thread.
- `742-03-open-as-thread`: the seat's row in the Harness section → Open as Thread: the same thread
  comes forward.

## Locked-In Decisions
- **D1:** the thread is a window onto the seat through the harness, never a second agent: the seat
  keeps running when the tab closes.
- **D2:** no hidden worktree for a remote folder.
- **D3:** this ticket waits for rustal-harness TICKET-116; if that ticket changes the verb's shape,
  this spec follows it.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a thread agent is started on a harness host, the system shall open a thread tab whose agent is that harness's `acp --seat NAME`. | Shot 742-01 and the fake ssh log |
| REQ-002 | WHEN a message is sent in that tab, the system shall deliver it to the seat and show the seat's reply in the thread. | Shot 742-02 |
| REQ-003 | WHEN Open as Thread is chosen on a seat's row, the system shall open or bring forward that seat's thread tab. | Shot 742-03 |
| REQ-004 | The thread's record shall name the seat's folder and host, and Marley shall add no worktree for it. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes; rustal-harness TICKET-116 done.
- **P2 Code** — `harness.rs` (the seat's agent server, Open as Thread), `agents.rs` (the thread on a
  host), `thread_tab.rs` (no worktree for a remote folder), the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
