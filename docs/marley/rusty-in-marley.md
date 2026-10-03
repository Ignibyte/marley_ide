# Rusty in Marley: Rusty's app rebuilt as Marley's panels and tabs

*Written 2026-10-02 in the fork. A plan, not a spec; each slice gets its own ticket when it
starts. It amends plan D11 ("Rusty stays Rusty") in [three-prong-plan.md](three-prong-plan.md)
and grows out of the knowledge layer in
[herdr-and-hermes-2026-10-02.md](../planning/design-notes/herdr-and-hermes-2026-10-02.md),
Part 3.*

## What Chad asked for

The idea, 2026-10-02:

> one thing im considering is an all in one system for Marley which brings in obsidian like
> knowledge graphs and also voice talking so that you can manage these projects + have a sort
> of developer centric knowledge system for yourself.
>
> However, i would like these to be enabled rather than by default because some may not want.

Then, answering the research:

> maybe rusty becomes Marley. We would take our rusty custom QML app and build it inside of
> marley. The mcp then lives inside of the marley ide but enabled/disabled

And asked whether to plan it now or leave it a maybe: "Plan it now".

## Decisions so far

Chad, 2026-10-02:

1. **Off until switched on.** Every part of this ships off by default with its own switch.
2. **The mic and Rusty's tools for Zed's agents** go off by default too ("Both off by
   default"), ticketed as #642.
3. **Embeddings are configurable.** Rusty already has the switch: `embedding_provider` is
   `auto` (local Ollama only), `ollama`, `openai` or `off`, and OpenAI runs only when named
   (`rusty-core/src/brain/semantic.rs:1-9, 204-215`). This box names `openai`, so vectors go
   off the machine by choice. Marley shows the setting; it adds no fallback.
4. **Voice is shelved but open**; phone approvals wait until the end and come as an app.
5. **Hosting agents is the harness's work** (rustal-harness D163, D164), embedded in Marley or
   standalone (plan D19).

## How Rusty works today

Four crates in `/srv/stacks/rusty-v3`, MIT:

- `rusty-core`: the store. SQLite at `~/.rusty/rusty.db`, the brain vault at `~/.rusty/brain`
  (803 pages, Obsidian-format markdown, git-backed, full-text plus vectors, a typed link
  graph), the skill store at `~/.rusty/skills`, tasks, memories, secrets, settings.
- `rusty-mcp`: 85 MCP tools over the store. It runs as `rusty-mcp.service` (Streamable HTTP on
  `127.0.0.1:4174/mcp`) and as a stdio child of any agent whose `.mcp.json` names it. It sends
  `notifications/resources/list_changed` on every change it sees (`main.rs:1896-1913,
  2061-2085`).
- `rusty-cli`: the terminal side, linking `rusty-core` directly.
- `rusty-app`: the Qt/QML desktop app, 30 QML files (about 7.9k lines) over a Rust backend.
  It is an MCP client of `rusty-mcp` for everything except agent sessions, which its own
  binary hosts (`rusty agent host --id …` in transient user units, a versioned NDJSON socket
  per session, `agent/protocol.rs`).

Rusty serves the whole box, not only its app. Claude Code's hooks (the brain loop's
ask-before-write and decide-before-stop, the transcript archive at session end), every store
skill, `rusty-cli` and agent sessions in other repositories use it with no app open. That is
the main constraint on this plan: Rusty's engine must keep running with Marley closed.

## The seam

Marley never links `rusty-core`. §20's brain boundary keeps any brain a separate program
reached over MCP, and the reason above makes it practical as well as legal. The shape is
plan D19's for the harness: Marley either starts `rusty-mcp` as a process of its own or finds
the one already running, and speaks MCP to it. What moves into Marley is the app: the screens
are rebuilt natively in GPUI, drawn from Rusty's tools, and the Qt app retires once Marley
covers it.

D11 now reads (Chad approved the wording 2026-10-02): *Marley draws Rusty's knowledge workspace (pages, graph, search,
tasks, memory, skills, secrets) natively from `rusty-mcp`; Rusty stays the store, the index
and the only writer of its data.*

## Design

### R-D0. The switch

`marley.rusty` in the `marley` settings block, shaped like `marley.system_one`:

```json
"rusty": {
  "enabled": false,
  "connection": "embedded",
  "agent_tools": false
}
```

Off means nothing starts: no process, no MCP connection, no panel, no dock button
(`workspace::Panel::enabled`). `connection` is `embedded` (Marley starts `rusty-mcp` on stdio,
found on the search path) or `service` (the running HTTP service). `agent_tools` is today's
`marley.rusty_tools` (#633), off by default after #642, and moves into this block when R1
lands. The Settings window's Marley page gets a Rusty section, with Rusty's own server
settings (the embedding provider among them) read and written through `settings_list` and
`setting_set`.

### R-D1. One crate, the house pattern

Plan D8 named two adapter crates, `marley_rusty` (Rusty's agent socket) and `marley_brain`
(the brain's MCP tools). Both talk to one program, so they become one: `crates/marley_rusty`,
a pure core with no gpui (typed views of Rusty's tool results, the graph layout, the markdown
pass, the project join rule, fixtures and a stand-in server for scenarios), and a thin adapter
in `marley_workbench::rusty`, which already holds #633's context-server offer.

The MCP client is the one `harness.rs` and `fleet_providers.rs` already use
(`context_server::ContextServer::stdio`). Writes go only through Rusty's tools, so its index,
embeddings and git history stay consistent; a source edit in an editor buffer is the
exception (R-D4).

### R-D2. Live refresh

`list_changed` reaches a stdio client, but it names no resource, and a database-only write
made by another `rusty-mcp` process (an agent's stdio instance adding a task) is not announced
to anyone else. Zed's HTTP transport never opens the GET stream server notifications arrive on
(`context_server/src/transport/http.rs:107-108, 375-376`). So Marley uses the embedded stdio
connection, re-reads what it shows on `list_changed`, and asks Rusty for a change cursor
(`changes_since`, request RQ2) so writes from every process are seen, polled the way the rail
polls the harness's `fleet_events`. No Zed touchpoint.

### R-D3. Where the screens go

| Rusty screen | In Marley |
|---|---|
| NoteTab (a page) | A **Page** tab in the center (`impl Item`): rendered view, a source toggle that opens the vault file in a Zed editor buffer, the properties list, back and forward |
| RightPane: backlinks, outgoing, tags | The **Knowledge** panel in the right dock, following the focused page; outline stays Zed's `outline_panel` |
| GraphView | A **Graph** tab: `brain_graph` around the current page or project with depth, filters and colour by link type |
| Explorer (vault tree) | A **Vault** panel in the left dock over `brain_tree`; new, rename, move and delete through `brain_new_page`, `brain_rename`, `brain_delete_*`, never the disk |
| SearchPane | Brain search in the Knowledge panel over `brain_search`, with its `tag:`, `path:`, `type:` operators |
| QuickSwitcher | `rusty: open page`, a picker over `brain_list_pages` by title, favourites first, create on a miss |
| BookmarksPane | Waits on RQ4; the Qt app keeps bookmarks in its own `workspace.json` |
| DecisionsPage | A **Decisions** tab over `brain_due` and decision pages; Marley's System One tab becomes "System One calls" (open decision 4) |
| TasksPage | A **Tasks** tab over the twelve task tools |
| MemoryPage, SkillsPage, SecretsPage | One tab each over their tools |
| SettingsPage | The Rusty section of the Marley settings page (R-D0) |
| Main.qml dialogs | Palette actions: import a vault, capture a URL, today's note, capture a line |
| Agent screens | Not rebuilt (R-D6) |
| TopBar, Splitter, Scanlines, Icon, CommandPalette | Zed's own chrome; the CRT overlay is dropped |

### R-D4. Rendering and editing

`brain_render` returns Qt rich-text HTML with inline colours (`rusty-core/src/brain/render.rs:
192-209`), which GPUI cannot draw, though its `outline`, `links`, `unresolved` and `tasks`
fields are usable as they are. The Page tab renders the markdown itself in `marley_rusty`,
with wikilinks resolved from `brain_render`'s `links`; Zed's `markdown` crate turns wikilinks
off on purpose (`markdown/src/parser.rs:985-987`), so either Marley's pass produces the
elements, or Zed's parser gains a small additive switch (a touchpoint, decided in R2's Plan
phase). A structured render from Rusty (RQ3) would remove the second parser entirely.

Source edits open the vault file in a Zed editor buffer and save to disk, as Obsidian does;
Rusty's watcher reindexes and commits them. Renames, moves, deletes and property edits go
through the tools so links are rewritten.

### R-D5. The project join

Each rail project resolves to a brain project page (by the page's `path:` frontmatter, then
by name) and a Rusty task group. The Knowledge panel opens on the project's page when no page
is focused: the page, follow-ups due for it, its tasks. This is what "manage these projects"
needs first.

### R-D6. Agent sessions are not rebuilt

Rusty's agent screens (a Claude Code session drawn as a conversation, the sessions list, the
page-tied agent in the right pane) duplicate four things that already run Claude Code on the
box: Marley's terminals with their hook frames, Zed's Agent Panel over ACP, Rusty's own host,
and the harness's seats. Rusty's host is the one with no MCP surface and the one living in the
Qt binary. The plan's open question 3 asks whether Rusty's host and the harness converge;
with the harness taking agent hosting (D163, D164), the recommendation is that Rusty's host
retires and Rusty sessions become harness seats or Zed threads. The page-tied agent becomes
Zed's Agent Panel in the right dock with the page @-mentioned and Rusty's tools offered. Open
decision 2.

### R-D7. Secrets

Every secrets action has a tool. The unlock token lives only in the tab's memory; it and any
revealed value never reach Marley's MCP server, its logs, a notification or System One's
masked state, and the tab locks when Marley's window loses focus, as Rusty's page does.

### R-D8. Scenarios never touch the real brain

Marley's origin is public and the brain is private. Every scenario runs a stand-in
`rusty-mcp` from `marley_rusty`'s fixtures over a scratch vault, never the user's, the rule
#633's scenario already keeps (`script/e2e.sh` sets `rusty_tools` false in each copy).

## Rusty-side requests

Work in `/srv/stacks/rusty-v3` through Rusty's own workflow, filed there when Chad confirms:

- **RQ1.** If Rusty keeps hosting sessions: move the host out of the Qt binary (to
  `rusty-cli` or its own binary) and add session tools. If it retires (R-D6): move `rusty
  session …`, `rusty agent …` and the store-script commands the Qt binary dispatches
  (`rusty-app/src/main.rs:27-77`) to `rusty-cli` before the app goes.
- **RQ2.** A change cursor tool (`changes_since`) covering every write, from every process.
- **RQ3.** A structured render: blocks with resolved links, embeds and callouts, beside the
  HTML.
- **RQ4.** Bookmarks and favourites in the store with tools, out of the app's
  `workspace.json` (Rusty puts them in a git-tracked vault file; see Rusty's triage below).
- **RQ5.** Rusty's roadmap: M8 ("Knowledge workspace, Obsidian inside Rusty") and TICKET-033
  (composer extras) move to Marley; the Qt app takes fixes only until R9.

## Zed touchpoints this plan adds

None planned beyond the rows that already exist for the `marley` settings block
(`settings_content/src/marley.rs`, `default.json`, `settings_ui/src/marley_page.rs`). One is
possible: a wikilink switch in `markdown/src/parser.rs`, only if R2 chooses Zed's parser.

## Slices

| Slice | What | Size |
|---|---|---|
| R0 | D11 amended, decisions recorded, RQ1 to RQ5 filed in Rusty | docs |
| R1 | The switch and the connection: `marley.rusty`, embedded or service, status on the settings page; `rusty_tools` moves in | S |
| R2 | The Page tab: render, wikilinks, source edit in a buffer, back and forward; the Knowledge panel's backlinks and outgoing | M |
| R3 | `rusty: open page` and brain search | S |
| R4 | The Vault panel with rename, move and delete through tools | M |
| R5 | The Graph tab | M |
| R6 | The project join and the panel's project view (page, follow-ups, tasks) | S |
| R7 | The Tasks tab and the Decisions tab | M |
| R8 | Memory, Skills and Secrets tabs; Rusty's server settings on the settings page | M |
| R9 | Parity check against the Qt app; the app retires in Rusty | Rusty-side |

R1 comes first because nothing else can start without it; R2 and R6 give the most for the
least, R5 is the picture Chad described, R9 waits for Chad's word after using the rest.

## Risks

- **Two apps for a while.** Rusty's roadmap is still building the Qt app (M8 in progress);
  without RQ5 the same screens get built twice.
- **Rendering fidelity.** Callouts, embeds and per-section editing from the Qt app need a
  structured render or a Marley pass; the first R2 Plan phase settles it.
- **Writes Marley can't see** until RQ2: a task added by an agent appears only on the next
  full read.
- **Agent sessions in flight.** Retiring Rusty's host strands any session it runs; the
  harness's M13 must take Claude Code in its own interface first.
- **Theme.** Rusty follows the Omarchy theme (its §10); these screens use Marley's Zed theme.

## Open decisions for Chad

1. ~~The D11 amendment text above.~~ Approved 2026-10-02 ("Use this wording"); D11 updated in the three-prong plan.
2. ~~Rusty's agent sessions: retire Rusty's host in favour of the harness and Zed's Agent Panel
   (recommended), or keep it and add session tools to Rusty.~~ Decided 2026-10-02: "Retire it",
   after the harness runs Claude Code in its own interface (its M13). RQ1 takes its second branch.
3. ~~Freeze the Qt app's new work in Rusty's roadmap now (RQ5)?~~ Decided 2026-10-02: "Freeze
   now". Sent to a Rusty session with RQ1 to RQ4 the same day.
4. ~~Rename Marley's System One "Decisions" tab (for example "System One calls") so Rusty's
   decisions keep the name.~~ Decided 2026-10-02: Marley's System One log becomes "System One
   calls"; Rusty's tab keeps "Decisions". The rename rides with R7.
5. ~~Skills: manage Rusty's store only, or also link it into `~/.agents/skills` so Zed's own
   agent sees the same skills Claude Code does.~~ Decided 2026-10-02: "Link for Zed and Codex
   too". The ops install script (`/srv/stacks/omarchy-ops/bin/install.sh`, which already links
   the store into `~/.claude/skills`) also links it into `~/.agents/skills`, which Zed's agent
   and Codex read, leaving out skills that only work in Claude Code. An ops change, done beside
   R8.

## Rusty's triage, 2026-10-02

The rusty-v3 session filed the requests as Rusty's TICKET-035 to TICKET-039 under a new
milestone M10, "Rusty as Marley's back end" (Rusty commit `daf15f5`; brain decision
`decisions/rusty-files-marleys-rq1-rq5-as-ticket-035-to-039-and-m10-the-qt-app-frozen`). Its
ROADMAP carries the freeze (fixes only until R9), marks M8 handed to Marley, and closes
TICKET-033.

- TICKET-035, the change cursor (RQ2), first in Rusty's queue: a change log in `rusty.db` every
  `rusty-core` writer appends to, read by a `changes_since(cursor)` tool over stdio and HTTP
  alike, with a "re-read everything" answer past retention. It also covers R-D4's gap: a source
  edit the vault watcher picks up is announced to no other process until it lands.
- TICKET-036, the structured render (RQ3): typed blocks beside the unchanged HTML, with resolved
  wikilinks, embeds one level deep, callouts, and source byte ranges.
- TICKET-037, bookmarks and favourites (RQ4), with one change: they live in a git-tracked file in
  the vault, not in SQLite, because Rusty's §10 keeps SQLite derived-only; they follow renames.
- TICKET-038, the `rusty` command without Qt (RQ1's first half), bigger than RQ1 said: store
  skills call `rusty vm-test`, `rusty pr-checkout`, `rusty research-home` and more through the
  Qt binary, so the `rusty` name moves to a binary with no Qt and the window becomes `rusty-app`.
  `rusty session start` also starts `rusty-mcp.service`; it shrinks to that and keeps working.
- TICKET-039, retiring the agent host (RQ1's second half), blocked on the harness's M13; it lists
  every session with its transcript and resume id first, so nothing is stranded.
