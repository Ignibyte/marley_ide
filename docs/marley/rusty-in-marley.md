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
  "service_url": "http://127.0.0.1:4174/mcp",
  "agent_tools": false
}
```

Off means nothing starts: no process, no MCP connection, no panel, no dock button
(`workspace::Panel::enabled`, with `Panel::icon` returning `None`, which is what hides the
button). A `rusty:` action run while Rusty is off or not connected shows a toast saying so and
where to turn it on, as #642's dictation action does; it opens nothing (settled 2026-10-03 across
#643 to #659). Since #661 off also hides every `rusty:` command and `marley: toggle brain view` from
the command palette (Zed's `CommandPaletteFilter`, as `disable_ai` uses it) and leaves the settings
page's Rusty section to its switch: Chad chose one build with one switch (2026-10-06), and off
shows nothing of Rusty but the switch; a bound key keeps the toast. `connection` is `embedded` (Marley starts `rusty-mcp` on stdio,
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
| Explorer (vault tree) | The rail's **Brain** view (R-D9) over `brain_tree`; new, rename, move and delete through `brain_new_page`, `brain_rename`, `brain_delete_*`, never the disk |
| SearchPane | Brain search in the Knowledge panel over `brain_search`, with its `tag:`, `path:`, `type:` operators |
| QuickSwitcher | `rusty: open page`, a picker over `brain_list_pages` by title, favourites first, create on a miss (#654; the favourite pages after the page in front since #662) |
| BookmarksPane | The Brain view's **Favourites** over `bookmark_list` and its three siblings, the Page tab's star (#662) |
| DecisionsPage | A **Decisions** tab over `brain_due` and decision pages; Marley's System One tab becomes "System One calls" (open decision 4). Shipped in #659 |
| TasksPage | A **Tasks** tab over the twelve task tools |
| MemoryPage, SkillsPage, SecretsPage | One tab each over their tools; Memory shipped in #664, Skills in #665, Secrets in #667 |
| SettingsPage | The Rusty section of the Marley settings page (R-D0) |
| Main.qml dialogs | Palette actions (#663): `rusty: capture to today`, `rusty: capture to inbox`, `rusty: capture url`, `rusty: import vault` (the plan, then the import and its report), `rusty: open today` |
| Agent screens | Not rebuilt (R-D6) |
| TopBar, Splitter, Scanlines, Icon, CommandPalette | Zed's own chrome; the CRT overlay is dropped |

### R-D4. Rendering and editing

`brain_render` returns Qt rich-text HTML with inline colours (`rusty-core/src/brain/render.rs:
192-209`), which GPUI cannot draw, though its `outline`, `links`, `unresolved` and `tasks`
fields are usable as they are. Zed's `markdown` crate turns wikilinks off on purpose
(`markdown/src/parser.rs:985-987`). The Page tab (#645, shipped 2026-10-04) runs a pass in
`marley_rusty::page` that finds the wikilinks with Rusty's own parse options and rewrites each
into an ordinary link with Rusty's address (`rusty:page/SLUG#HEADING`, or `rusty:new/TARGET` for a
page `brain_render`'s `links` did not resolve); Zed's `markdown` crate draws the result unchanged,
so no touchpoint in Zed's parser. Rusty's structured render (RQ3, its TICKET-036, on its main
since 2026-10-04 as `brain_render { blocks: true }`) would replace the pass's lookup, not Zed's
renderer.

Source edits open the vault file in a Zed editor buffer, inside the Page tab, and save to disk
unformatted, as Obsidian does. Rusty's watcher announces a disk edit about 0.6 s later and
reindexes it about 5 s after; since Rusty's TICKET-043 (on its main 2026-10-04) the indexer
commits it on its own. With the older binaries still on the box until Chad reinstalls, nothing
commits it until the next tool write's `git add -A` sweeps it into that tool's commit, as with an
Obsidian edit. Renames, moves, deletes and property edits go
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
Since #667 Zed's MCP client logs the messages of Rusty's two servers (`marley-rusty`, `rusty`)
by size, never by content, so a PIN, a token or a value cannot reach Zed's log through trace
logging either (the `context_server` rows in `docs/marley/zed-touchpoints.md`).

### R-D8. Scenarios never touch the real brain

Marley's origin is public and the brain is private. Every scenario runs a stand-in
`rusty-mcp` from `marley_rusty`'s fixtures over a scratch vault, never the user's, the rule
#633's scenario already keeps (`script/e2e.sh` sets `rusty_tools` false in each copy).

### R-D9. Brain navigation is a switch in the rail's header

Chad, 2026-10-02, picking how Rusty's screens are reached: "Switch in the rail header". The
rail's header, which reads `PROJECTS` today (`marley_workbench/src/rail.rs`, `render_header`),
becomes two icon buttons, **Projects** and **Brain**, in the manner of Obsidian's ribbon and VS
Code's activity bar. Brain swaps the rail's content for:

- a fixed row: Today, Graph, Tasks, Decisions, Memory, Skills, Secrets, each one click to open
  or focus its center tab (moved up into the header itself, in both views, by #672: Chad wanted
  them "not two levels deep");
- (since #678 the whole view is the left column of a Brain tab in the Rusty group, the page on
  the tab's right, and the rail no longer swaps: on 2026-10-07 Chad asked for the Brain to carry
  its own navigation, as the Tasks tab does, rather than take over the rail)
- brain search and the favourites;
- the vault tree, where one click opens a page as a preview tab and a second keeps it, as Zed's
  file tabs do.

One key flips between the two views. The Knowledge panel in the right dock follows the open
page whichever view the rail shows. It replaces the left-dock Vault panel the first draft had,
which would have put two columns side by side. A pop-up menu was rejected: two clicks for every
open, and no tree that stays open. The catch: Zed hides the whole rail when its AI features are
off (`multi_workspace_enabled`, workbench-shell.md, "Deferred"), and the Brain view goes with
it; the Knowledge panel and the palette actions still work. No Zed touchpoint: the rail is
Marley's own.

### R-D10. Ely GPUI Components, ported, not depended on

Chad, 2026-10-02: "lets make sure we use the gpui components we found here". Ely GPUI
Components (github.com/ZacharyZhang-NY/Ely-GPUI-Components, `MIT OR Apache-2.0`, research page
`research/ui-and-design/ely-gpui-components`) has a part for nearly every Rusty screen. It
can't be a dependency: it builds `gpui` from Zed's repository at rev `1a28cff`, not the fork's
own in-tree `gpui`, and two `gpui` copies don't mix; it brings its own theme, fonts (Inter,
JetBrains Mono, IBM Plex Sans) and Lucide icons; and it is tested on macOS only. Each slice
copies the components it needs into `marley_rusty` (pure parts) or `marley_workbench::rusty`
(views), with Ely's MIT notice on each file, rewritten onto the fork's `gpui`, Zed's theme
colours and the `ui` crate's primitives, so the screens look like the rest of Marley and run on
Linux. Every spec's Prior art names the Ely story it ports.

| Ely story | For |
|---|---|
| `pagetree-favorites-pinned`, `filetree` | The rail's Brain view (R-D9) |
| `markdownrenderer-tableofcontents-documentoutline-readingprogress`, `pagecover-pageicon-pageproperties` | The Page tab (R2); its outline read for R2b's column (#656) |
| `markdowneditor`, `inlineedit-editabletext` | Inline title and property edits (R2b; `InlineEdit` ported as `rusty/inline_edit.rs`, #656) |
| `backlinks-graphview`, `searchpanel-searchresultitem-searchfilters` | The Knowledge panel (R3, R6) |
| `networkgraph-forcegraph-chorddiagram-parallelcoordinates` | The Graph tab (R5) |
| `slider-rangeslider-verticalslider` | The Graph tab's Display and Forces (R5b; `Slider` ported as `rusty/slider.rs`, #657) |
| `sortablelist-reorderablelist`, `kanbanboard-kanbancolumn-kanbancard-issuecard-sprint-board` | The Tasks tab (R7) |
| `propertygrid`, `settingslayout-settingssection-settingsrow-settingssearch` | Memory, Secrets, Rusty's settings (R8) |
| `searchpalette-spotlightsearch-quicklauncher` | `rusty: open page` (R3) |

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
| R1 | #643, shipped 2026-10-04. The switch and the connection: `marley.rusty`, embedded or service, status on the settings page; `rusty_tools` moves in | M |
| R4 | #644, shipped 2026-10-04. The rail's Brain view (R-D9): the header switch, Today, search on Enter, the vault tree with new, rename, move and delete through tools. Since #672 (2026-10-07) the seven screens sit in the header itself, beside Projects and Brain, and since #675 every screen and page opens in the window's Rusty group; since #678 the view is the Brain tab's left column; since #679 the rail holds one Rusty button, and the screens' buttons are on Rusty's home page; since #699 (2026-10-09) the Rusty group's + lists the screens, Open Page… and the captures, and the group never shows no tab: its home page opens when it would | M |
| R2 | #645, shipped 2026-10-04. The Page tab: Zed's `markdown` with wikilinks rewritten to `rusty:` links, properties, back and forward, preview tabs, Edit in a buffer; the `rusty::OpenPage` action | M |
| R3 | #646, shipped 2026-10-04. The Knowledge panel in the right dock: the Page tab's tags with counts, backlinks with their lines, outgoing links with Create, and brain search on Enter with match case and regex; the project view went to R6 | M |
| R5 | #647, shipped 2026-10-04. The Graph tab: whole vault or local with depth, filters, page-type colours, decision edges dashed, Ely's force layout off the main thread, a 2,000-node cap | M |
| R3a | #654, shipped 2026-10-04. `rusty: open page` (Ctrl+Alt+U): a picker over `brain_list_pages`, titles and slugs matched in Marley, the recently opened first, create on a miss through `brain_new_page { path }`; an unresolved link makes its page | S |
| R6 | #655, shipped 2026-10-04. The project join (R-D5) and the panel's project view: the project's page by `path:` then by name, its follow-ups due, its task group's open tasks, Link a Page and Link a Task Group through `brain_set_property`; the Graph tab's project centre | M |
| R2b | #656, shipped 2026-10-04. The Page tab's outline column in Read (Zed's outline panel in Edit), a heading brought to the top; the title (the `title` property), the name (`brain_rename`, every tab following) and each property edited in place by its kind, removed, or added | S |
| R5b | #657, shipped 2026-10-04. The Graph tab's colour groups, Display (arrows, text fade, node size, link thickness) and Forces on Ely's slider, one record for every window in Zed's key-value store, and the tab restored after a restart while Rusty is on | M |
| R4b | #662, shipped 2026-10-06. Favourites above the Brain view's tree (pages, folders, searches and headings in Rusty's order, renamed and removed from their menu), the Page tab's star and Ctrl+D, and the favourite pages as the page picker's first group, over `bookmark_list` and its three siblings | S |
| R7b | #660, shipped 2026-10-05. Recording a follow-up from the Decisions tab (`brain_follow_up`: status, outcome, a new day when revised, the successor when superseded, picked from the tab's decisions); the rows show TICKET-048's `followed_up` and `superseded_by` | S |
| R7 | The Tasks tab (#658, shipped 2026-10-04: Rusty's lists and tasks over its twelve task tools, every change one call in order and read back, the drag and the keys, Open in Tasks from the project view) and the Decisions tab (#659, shipped 2026-10-04: `brain_due`'s follow-ups due, then every decision with its status and dates, each opening its page; Marley's System One log renamed System One calls, the old action id kept as an alias) | M |
| R8 | Memory, Skills and Secrets tabs; Rusty's server settings on the settings page. The Memory tab shipped in #664 (2026-10-06): the memories in Rusty's order, added, filtered by category, edited and deleted over Rusty's four memory tools. The Skills tab shipped in #665 (2026-10-06): staged skills approved, approved anyway or rejected, skills and scripts edited, made and deleted over Rusty's eleven skill and script tools, and a script run in a terminal. Rusty's settings on the Rusty's Server page shipped in #666 (2026-10-06): the ten keys Rusty's app lists, the other stored keys with credentials masked, a key added, and the embedding status. The Secrets tab shipped in #667 (2026-10-06): the names, the PIN set and unlocked, values revealed, copied, replaced and deleted with the token, the tab locking on expiry, on Lock and on losing focus, and Rusty's MCP messages logged by size only. R8 is complete | M |
| R9 | Parity check against the Qt app; the app retires in Rusty. Moot: Chad retired the Qt app on 2026-10-09 without it (Rusty TICKET-053, code at the tag `rusty-app-final`); `rusty-mcp` and its tools are unchanged, and new UI asks come to Marley | Rusty-side |

The second batch, #654 to #659, was queued 2026-10-03: R3a (#654, the open-page picker on
Ctrl+Alt+U), R6 (#655, the project view), R2b (#656, the outline and in-place edits; the title
edit sets the `title` property and a separate name field renames, confirmed by Chad 2026-10-05), R5b (#657),
R7 split as #658 (the Tasks tab) and #659 (the Decisions tab, with Marley's System One tab
renamed "System One calls"). R7b followed as #660, R4b as #662, and R-D3's Main.qml dialogs as
#663 (capture and import from the palette). R8 followed as #664 (Memory), #665 (Skills), #666
(Rusty's settings) and #667 (Secrets). R9 waits for Chad's word.

The first batch, #643 to #647, was queued 2026-10-03 in this order: R1, R4, R2, R3, R5, after
#640 to #642 (#643 builds on #642's settings changes). Drafting split out R3a, R6, R2b, R5b and
R4b, which need tickets; R7 and R8 follow them. R9 waits for Chad's word after using the rest.

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
   calls"; Rusty's tab keeps "Decisions". The rename rides with R7. Done in #659 (2026-10-04).
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

Marley's drafting findings followed the same day as Rusty's TICKET-040 to TICKET-043 (Rusty
commit `295565c`): a deleted folder's pages coming back from `archive/` into the index (040; until
it lands, `brain_list_pages` is not to be trusted after a folder delete, and whether `brain_tree`
shows `archive/` is open for #644); `brain_new_page` turning `a/b` into a root page `a-b` (041;
until it lands, Marley never sends a slashed name); the page's file path and the vault root in
`brain_read_page` and `brain_stats` (042); and disk edits committed on their own, with tool
commits naming only their paths (043, which shares a design question with 035). `brain_graph`
stays the Knowledge panel's source for tags: about 9 ms on the real store.

The second batch's findings became Rusty's TICKET-044 to TICKET-048 (Rusty commit `5e27add`):
every date on the UTC day instead of the local one (044), foreign keys off and writes on a
missing id reporting success (045), three writers that reorder a page's frontmatter (046),
aliases and chosen properties in `brain_list_pages` (047), and `superseded_by` and a follow-up
date in the decision summaries (048).

### Landed on Rusty's main, 2026-10-04

The rusty-v3 session reported TICKET-035 and TICKET-040 to TICKET-048 on Rusty's `main` (035 is
`ea46a83`). The box runs the old binaries until Chad reinstalls Rusty (`omarchy/install.sh` in
its repository), so each goes live then.

- **035, the change cursor (RQ2).** `changes_since { cursor?, limit? }` answers `{ cursor,
  changes: [{ seq, kind, key, op, detail, at }], reset, more }`. Kinds: page, task, task_group,
  memory, note, setting, secret, skill, script. Ops: created, updated, deleted, moved (`detail` is
  the old slug), and approved and rejected for skills. Without a cursor it starts from now;
  `reset: true` means read everything again (the log keeps 20,000 rows); `more: true` means call
  again with the cursor it answered. It carries names, never values or bodies. A page change is
  recorded where the shared index is written, with the content hash, so a disk edit that several
  processes sync is recorded once. A manager's error still reaches a client as a JSON-RPC error
  (-32603), not an `is_error` result.
- **040.** The root's `archive/` is never indexed, written into or listed: `brain_tree` leaves it
  out, and a write, a new folder or a move into it is refused. A deleted page's links are dropped,
  so it is no longer a ghost backlink. This settles #644's open point: the Brain view lists what
  `brain_tree` serves, which leaves `archive/` out.
- **041.** `brain_new_page { path }` makes the page at exactly that path, its folders with it, and
  returns an existing page as it is: the call for create-on-a-miss (#654). `{ folder, name }`
  works as before.
- **042.** `brain_read_page` and `brain_render` carry `file`, the page's absolute path;
  `brain_stats` carries `vault_root`.
- **044.** Every date is the local day. **045.** Foreign keys are on, and a write on a missing id
  fails and names it. **046.** `update_page`, `add_timeline` and a rename keep the frontmatter as
  written. **047.** `brain_list_pages` carries `aliases`, and `properties` when asked for with
  `properties: [...]`; the index's stored frontmatter was `{}` until this (a correction of the
  2026-10-03 triage), and pages indexed before it fill on the next sync. **048.** Decision
  summaries carry `followed_up` and `superseded_by`.
- Later the same day the rest of M10 but 039 landed (last `13249a8`); 039 waits on the harness's
  M13.
  - **043** (`6176b5d`): a vault commit holds only the paths its write touched, and an edit made
    outside the tools gets a commit of its own from the indexer after a sync. R-D4's commit gap
    (#645's D6) is closed.
  - **036** (`1cd368e`): `brain_render { blocks: true }` adds `blocks`, typed blocks with byte
    ranges into the body, inline runs with resolved wikilinks, callouts and page embeds one level
    deep, and `body_start`, beside the HTML.
  - **037** (`ecc2904`): bookmarks live in the vault at `.rusty/bookmarks.json`, behind
    `bookmark_list`, `bookmark_add`, `bookmark_remove` and `bookmark_set` (entries: `kind` file,
    folder, search or heading; `title`, `path`, `query`, `heading`). Renames carry them, deletes
    drop them, and `brain_import` adds a vault's own. R4b, the Brain view's favourites, can be
    planned on these. The server has 90 tools.
  - **038** (`13249a8`): `rusty` is `crates/rusty-cmd`, a binary with no Qt: `rusty session
    start|stop|status|run`, `rusty agent …` (the host included) and `rusty <store-script>`. The
    window is `rusty-app` (app id `com.ignibyte.rusty`); `rusty` alone execs it, exits 127 when it
    is not installed, and `rusty session start` then starts only `rusty-mcp.service`.
