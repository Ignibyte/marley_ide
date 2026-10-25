---
pipeline_id: 3a4d28c2-3103-4a74-bacb-52c84b047dd7
ticket: docs/planning/tickets/open/TICKET-904-home-prompt-asks-marley-or-rusty.md
status: Phase 4 — Complete PASS
title: Home's prompt asks Marley or Rusty
type: feature
slice: The Marley look, W2.2 (split from #903)
references: [docs/marley/design-review-2026-10-23/A-core.md, crates/marley_workbench/src/home_page.rs, crates/marley_workbench/src/assistant.rs, crates/marley_workbench/src/thread_tab.rs]
---

## Title
Home's prompt offers Marley's agent as its first chip and Rusty's as its last, while each is on.
Start with one of them picked opens a new conversation with that agent in a center tab, the
prompt's text sent as its first message.

## Scope
### In
- **The chips** (Marley crate, `marley_workbench/src/home_page.rs`, `render_prompt`): a Marley
  chip (the cat mark, "Marley") before the agent CLIs while `assistant::marley_present`, and a
  Rusty chip (the butler mark, "Rusty") after them while `assistant::rusty_present`. The picked
  chip lights as a CLI's does. `picked` becomes a `Pick` enum: `Marley`, `Cli(AgentKind)`,
  `Rusty`.
- **The project chip** greys out and stops opening its menu while Marley or Rusty is picked,
  its tooltip saying the agent works in its own folder (`marley.assistant.folder`,
  `marley.rusty.agent_folder`, #780). It keeps its place, so the chips beside it do not move.
- **Start** (`home_page.rs`, `start_prompt`): with Marley or Rusty picked, a new conversation of
  that entry starts in a center tab of Home's workspace, in the entry's folder, and the prompt's
  text is sent as its first message; an empty box opens the conversation with nothing sent. The
  box empties as it does for a CLI.
- **Where Home's terminals start** (Marley crate, `agents.rs`, `start_directory`): a terminal or
  agent CLI never starts in a hidden worktree's folder. A Marley or Rusty conversation in Home
  joins its folder to Home's project hidden (#780), and Zed's `default_working_directory` then
  chose it, so Home's chip read "config" and its Claude Code started in Marley's settings
  folder. The home folder takes its place, as a group with no folder starts its CLIs (#600).
  Every Marley caller of `default_working_directory` goes through it.
- **Starting with a message** (Marley crate, `assistant.rs`, `thread_tab.rs`): a new
  `assistant::start_with_task(entry name, task, …)` runs `talk_to`'s fresh path with an
  initial message; `thread_tab::start_thread` takes an `initial: Option<AgentInitialContent>`
  that it hands to the `ThreadSpec` it builds (its callers pass `None`).

### Out (explicitly deferred)
- Marley picked by default when the box is typed into without a pick: the first installed CLI
  stays the default (D1).
- Rusty's or Marley's conversation in a picked project's folder: they keep their own folders (D2).
- The prompt's agent chips as a menu when they overflow the row: the row wraps today (#903).

## Reference (§20)
N/A — Marley-specific: Home (#701, #903) and Marley's and Rusty's Agent Panel entries (#683,
#696, #738) are Marley's own. The first message rides Zed's `AgentInitialContent::ContentBlock`
with `auto_submit`, as Zed's Agent Panel starts a thread from a prompt (`agent_ui`,
`agent_panel.rs`).

### Prior art
- `assistant.rs`, `talk_to` with `fresh: true`: a new conversation of an entry in its folder,
  through `thread_tab::start_thread` (`NewMarleyConversation`, `NewRustyConversation`).
- `assistant.rs`, `ask_beside` (#865): Marley's entry opened beside with text in its box,
  `ContentBlock { auto_submit: false }`, through `thread_tab::open_beside`.
- `thread_tab.rs`, `ThreadSpec.initial`: already carried to `ConversationView::new`;
  `thread_view.rs` sends it when `auto_submit` is set, once the thread view is made.
- `assistant.rs`, `assistant_icon`: `IconName::MarleyCat` and `IconName::RustyButler` mark the
  entries' threads (#807); the chips use the same marks.
- The Home board: the chips begin with Marley and end with Rusty; the box reads "Ask Marley, or
  describe a task and pick who does it" (already the placeholder since #903).
- The brain: nothing on this seam.

## UI proof
Scenario `script/e2e/904-home-prompt-asks-marley-or-rusty.sh` (sway). Setup: Rusty on against
Rusty's stand-in server (`MARLEY_RUSTY_MCP`), `marley.assistant.agent` set to `claude_code`
with `MARLEY_ASSISTANT_ADAPTER` naming a scripted ACP stand-in that answers each prompt with the
text it got, and #903's stand-in `claude` first on the terminals' PATH through the scenario's
`.bashrc`.
- `904-01-chips`: Home with Marley first and Rusty last among the chips (REQ-001).
- `904-02-marley-picked`: Marley picked: its chip lit, the project chip greyed in its place
  (REQ-002).
- `904-03-marley-thread`: after Start, a Marley conversation tab with the task sent and the
  stand-in's answer (REQ-003).
- `904-04-rusty-thread`: Rusty picked and started: a Rusty conversation tab with the task sent
  (REQ-003).
- `904-05-day`: Marley Day: Home's chips with Rusty picked, the project chip reading Home with
  both conversations open (REQ-001, REQ-002, REQ-004).
- `904-06-cli-home`: Claude Code picked and started from Home: the stand-in runs in the home
  folder (REQ-004).

## Locked-In Decisions
- D1 — The first installed CLI stays the default pick. Marley sits first in the row as the board
  draws it, but a task typed without a pick still goes to the agent CLI, as it has since #835.
- D2 — Marley and Rusty start in their own folders. The project chip greys out while one is
  picked, so it never offers a folder the conversation does not use. It stays in place: the first
  run hid it, and every chip after it jumped left under the pointer.
- D3 — Start sends the text (`auto_submit: true`); an empty box opens the conversation unsent.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley's entry is on, Home's prompt shall show a Marley chip before the agent CLIs; WHILE Rusty's entry is on, a Rusty chip after them. | `904-01-chips`, `904-05-day` |
| REQ-002 | WHEN Marley or Rusty is picked, its chip shall light and the project chip shall grey out in its place. | `904-02-marley-picked`, `904-05-day` |
| REQ-003 | WHEN Start is pressed with Marley or Rusty picked, a new conversation with that agent shall open in a center tab with the prompt's text sent as its first message. | `904-03-marley-thread`, `904-04-rusty-thread` |
| REQ-004 | WHILE a Marley or Rusty conversation is open in a group with no folder of its own, Home's project chip shall read Home and a terminal or agent CLI started there shall start in the home folder, never in the conversation's folder. | `904-05-day`, `904-06-cli-home` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the chips, the start path and the initial message; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
