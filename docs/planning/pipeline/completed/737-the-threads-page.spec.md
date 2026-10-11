---
pipeline_id: bf114e10-6b1b-4d05-b5dd-c2ca5c8b533d
ticket: docs/planning/tickets/closed/TICKET-737-the-threads-page.md
status: Phase 4 — Complete PASS
title: The Threads page
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/734-an-agent-thread-starts-in-a-tab.spec.md
---

## Title
A center tab listing every agent conversation Zed keeps, from every project, group and folder, so
an old conversation can be found and reopened. Zed keeps its thread history in its own sidebar
(`agents_sidebar::ToggleThreadHistory`, `ThreadsArchiveView`), which the Marley layout replaces with
the rail; today the rail's project rows are the only way back. Chad, 2026-10-10: "a home page for
threads".

## Scope
### In
- `marley: open threads` opens the Threads page in the shown group, or brings forward the one
  already there.
- The page lists the thread store's conversations (drafts left out), newest first, each with its
  title, its agent's icon and name, its folder (home shown as `~`) and when it was last updated.
- **Marley and Rusty** head the page: the conversations of those two entries, then **All
  conversations**, then **Archived**, folded until opened.
- A search field filters by title and folder; a chip per agent present filters by agent.
- A click opens the conversation where it lives when it is open (its tab, or a panel showing it),
  else in a tab of the shown group (#734's tab, loading the thread as #736 does).
- The page follows the store: a new or renamed conversation shows without reopening.
- `docs/marley/guide.md`: the page.

### Out (explicitly deferred)
- Deleting or archiving from the page: the rail's thread menu does both (#605, #616).
- Zed's own agent's profiles: the store does not record a thread's profile, so a Zed-agent Marley
  or Rusty thread is listed under All conversations.

## Reference (§20)
Upstream Zed, `agent_ui::threads_archive_view::ThreadsArchiveView`: the thread history Zed's sidebar
shows (every `ThreadMetadataStore` entry, newest first, a search over the title then the folders'
names). Behavior kept: the same store, the same search fields. Marley's page adds the agent filter,
the Marley and Rusty section, and opens in a tab rather than the panel.

### Prior art
- **The code we ship:**
  - `ThreadsArchiveView` (`threads_archive_view.rs:163`): `update_items` (271) lists `entries()`
    by `created_at` in time buckets; search matches the title, then each folder's basename
    (305-321); a click emits `Activate { thread }` (462). Reusable, but it has no agent filter or
    sections, and its click is the sidebar's to handle; Marley's page takes its search rule.
  - `ThreadMetadataStore` (`thread_metadata_store.rs`): `global`, `entries()`, `archived_entries()`,
    `ThreadMetadata::{display_title, folder_paths, agent_id, updated_at}`.
  - `agents::thread_icon` and `thread_agent_name` (`agents.rs:125,143`) for an agent's icon and
    name.
  - Marley's center pages: `home_page.rs` (`MarleyHome`) and `rusty/home_tab.rs` (`RustyHome`):
    an `Item` with a list, handlers as plain closures over a weak workspace
    (PR-claude-701).
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (items).
- **Published material:** none.

## UI proof
`script/e2e/737-the-threads-page.sh`, under `compositor sway`. The scripted agent of #736 (it
replays a loaded session) runs both the Marley entry and a second custom agent server, `Scripted`,
set in the run's settings. Three keys bound to `marley::NewAgentThread` give three conversations:
Marley in the project's root, Scripted on `other/`, Scripted on `second/`. The run's profile is a
copy of the user's, so the page may also list the user's own conversations, below the run's newer
ones; the checks rest on the run's.

Shots:
- `737-01-page`: the page: Marley and Rusty with the Marley conversation, All conversations with the
  two Scripted ones, each with its agent, folder and time.
- `737-02-search`: the search field holding `second`: only that conversation.
- `737-03-chip`: the Marley chip: only the Marley conversation.
- `737-04-open`: the `second/` conversation's tab closed, then its row clicked: a thread tab in the
  shown group with its turn.

## Locked-In Decisions
- **D1:** Marley's own page over the thread store, not `ThreadsArchiveView`, which has no sections
  or agent filter and hands its click to Zed's sidebar.
- **D2:** a click never opens a second copy: an open conversation comes forward where it is
  (#697's one-place rule).
- **D3:** newest by `updated_at`, the order the rail's rows use.
- **D4:** the search is a case-insensitive match on the title and the folders' paths, so a folder's
  name or any part of its path finds it.
- **D5:** a conversation not open opens as #736 restores one: the record's agent, folders and
  session, the folder joining the project hidden when it needs to.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley: open threads` runs, the system shall show the Threads page in the shown group, listing every saved conversation with its title, agent, folder and time. | Shot 737-01 |
| REQ-002 | The page shall list the Marley and Rusty entries' conversations first, under their own heading. | Shot 737-01 |
| REQ-003 | WHEN text is typed in the search field, the page shall list only conversations whose title or folder matches it. | Shot 737-02 |
| REQ-004 | WHEN an agent chip is chosen, the page shall list only that agent's conversations. | Shot 737-03 |
| REQ-005 | WHEN a conversation that is not open is clicked, the system shall open it in a tab of the shown group with its earlier turns. | Shot 737-04 |
| REQ-006 | WHEN a conversation that is open is clicked, the system shall bring forward the tab or panel showing it. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — a new `threads_page.rs` in `marley_workbench`, its action and init, the guide; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
