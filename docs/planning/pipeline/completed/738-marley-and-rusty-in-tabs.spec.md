---
pipeline_id: e1b0abab-6804-4092-89de-ce0b18a997c9
ticket: docs/planning/tickets/closed/TICKET-738-marley-and-rusty-in-tabs.md
status: Phase 4 — Complete PASS
title: Marley and Rusty in tabs
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/734-an-agent-thread-starts-in-a-tab.spec.md
  - docs/planning/pipeline/queued/737-the-threads-page.spec.md
---

## Title
The Marley agent and Rusty open where you are, in a tab, with no project needed. Chad,
2026-10-10: Marley "is an all around agent", and the agents "only show up in the project". Each
runs on a private folder of its own, since neither edits a file or runs a command (#683, #696).

## Scope
### In
- `marley: talk to marley` and `marley: talk to rusty`: the entry's latest conversation comes
  forward where it is when its tab is open in the window; otherwise its latest conversation opens
  in a tab of the shown group (#734, loaded as #736 does), or a new one starts when it has none.
- `marley: new marley conversation` and `marley: new rusty conversation` always start a new one in
  the shown group.
- The folder: `<Marley's data folder>/assistant/marley` and `…/assistant/rusty`, made when missing,
  through `*_in(dir)` functions (§14).
- On Zed's own agent (`marley.assistant.agent = zed`) the tab's thread is a Zed Agent thread with
  the entry's profile (`marley` or `rusty`).
- The commands are listed only while their entry is there (the Marley entry while
  `marley.assistant.enabled`; Rusty while Rusty is on and reached), as
  `marley: open marley agent in terminal` is.
- The Agent Panel keeps both entries in its New Thread menu.
- `docs/marley/guide.md`: "The Marley agent" and "Rusty in the Agent Panel" say how to open them.

### Out (explicitly deferred)
- The status bar buttons that run these commands: #739.

## Reference (§20)
N/A — Marley-specific: the Marley and Rusty agent entries are Marley's (#683, #687, #696), and the
tab is #734's. Upstream Zed's part is the thread view and the agent servers, used as #734 uses them.

### Prior art
- **The code we ship:**
  - `assistant.rs`: `enum Entry { Marley, Rusty }` with `name()` (the custom agent server's id)
    and `profile()` (the Zed-agent profile id); `Entries::of` on the `Assistant` global's
    `applied` says which entry is there; `OpenMarleyAgentInTerminal` and its palette filter (the
    house pattern for a command shown only while its entry exists).
  - `ThreadMetadataStore::entries()` with `agent_id` to find an entry's latest conversation.
  - Zed's agent: a thread's profile is set on its `Thread` (`agent` crate), as the profile picker
    does.
  - Marley's data folder through `paths::data_dir` and the `*_in(dir)` rule.
- **Behavior maps:** none: no map covers an assistant entry.
- **Published material:** none.

## UI proof
`script/e2e/738-marley-and-rusty-in-tabs.sh`, under `compositor sway`, launched with no folder so
Home shows. The Marley entry runs the scripted agent; Rusty is on with `marley_rusty`'s stand-in
`rusty-mcp` (#696's set-up), and its entry runs the same scripted program.

Shots:
- `738-01-marley`: `marley: talk to marley` in Home: a Marley tab; "hello" answered; the log's
  `cwd` is `…/assistant/marley`.
- `738-02-again`: a project shown, the command again: the Marley tab comes forward in Home.
- `738-03-rusty`: `marley: talk to rusty` in the project: a Rusty tab there, its `cwd`
  `…/assistant/rusty`.
- `738-04-new`: `marley: new marley conversation`: a second Marley tab in the shown group.

## Locked-In Decisions
- **D1:** a conversation is in one place: talk-to brings an open tab forward where it is rather
  than moving it, since a thread view belongs to the project of the group it opened in.
- **D2:** each entry has its own folder, so their records never mix with a project's threads.
- **D3:** "latest" is the entry's conversation with the newest `updated_at`, archived ones left
  out. On Zed's own agent every thread carries the Zed Agent's id, so an entry's conversations
  cannot be told apart there: talk-to starts a new conversation each time, with the profile.
- **D4:** an open conversation comes forward through the Threads page's `open_thread` (#737), run
  from `window.defer`: the command's handler runs inside the workspace's update, and
  `open_thread` reads every workspace of the window (PR-claude-735).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley: talk to marley` runs and no Marley conversation exists, the system shall start one in a tab of the shown group, in Marley's own folder. | Shot 738-01 and the agent log |
| REQ-002 | WHEN it runs while the latest Marley conversation's tab is open, the system shall bring that tab forward. | Shot 738-02 |
| REQ-003 | WHEN `marley: talk to rusty` runs, the system shall open Rusty's conversation in a tab of the shown group, in Rusty's own folder. | Shot 738-03 and the agent log |
| REQ-004 | WHEN a new-conversation command runs, the system shall start a new conversation of that entry in a tab of the shown group. | Shot 738-04 |
| REQ-005 | WHILE an entry is not there, the palette shall not list its commands. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `assistant.rs` (the commands, the folders, the profile), the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
