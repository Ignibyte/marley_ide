# The Marley look: implementation plan

Status: filed as tickets 2026-10-20 (TICKET-825 to 858); work starts with TICKET-828.

The design canvas "Marley, Warp-like" (`claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn`, 56 boards;
its source is kept in the private design record, `docs/design/marley-canvas-2026-10-20`) draws
every screen Marley has and a set it does not have yet. This plan turns the canvas into work: the
order to build it in, the code each piece touches, and the ticket each piece becomes.

Each item below becomes one ticket and runs through the usual pipeline: plan, code (the gate),
test (a visual check that drives the real app and reads its shots), complete. The boards are the
target; where code or upstream Zed pushes back, the ticket records how the result differs from
its board.

## What the code already gives us

- **The rail takes its colors from the theme.** `rail.rs` reads `cx.theme().colors()` and holds no
  color of its own, so a new theme reaches the rail, the tabs and most of Marley with no change
  to Marley's views.
- **Marley already adds to the bottom row.** `status_buttons.rs` implements `StatusItemView` for
  Home and Debug, so new bottom-row items live in a Marley crate. Only removing or replacing
  Zed's own items touches `crates/zed/src/zed.rs`.
- **The fleet takes outside providers.** `marley.fleet.providers` already accepts `pseudo`, `mcp`
  and `http`, and the Fleet panel draws whatever a provider answers in the shape of
  [the fleet contract](fleet-contract.md). Forge can arrive the same way.
- **Zed ships a component gallery.** The `component_preview` crate draws GPUI's components; with
  Marley's themes loaded it becomes the visual check for every theme change.
- **All of Marley's UI is in one crate.** Every tab, panel, dialog and picker Marley draws (54 of
  them) lives in `marley_workbench`. The other Marley crates draw nothing.

## The order

Seven waves. Each wave's tickets can run one after another in any order inside the wave; a later
wave assumes the earlier ones landed.

| Wave | What | Why first |
|---|---|---|
| 0 | Theme and the cat mark | Every other board is drawn in these colors |
| 1 | Chad's asks: the shrinking rail, `logo.png`, tmux Sessions, the bottom row | Asked for by name, and seen every minute |
| 2 | The core screens: rail rows, the thread, terminal blocks, the window chrome, Home, Threads | The three filed tickets and the screens around them |
| 3 | Panels, drawers, pickers and notices | Larger changes to how panels sit |
| 4 | Forge | A new data source; needs the fleet provider work |
| 5 | Every other screen restyled | Mostly layout once the theme is in |
| 6 | Polish: editor pop-ups, empty and error states, motion, focus | Last, across everything |

Sizes: **S** a day or less, **M** a few days, **L** a week or more.

## Wave 0: theme and the cat mark

**W0.1 Marley Night and Marley Day themes.** *Done (#828, 2026-10-20).* Boards: Theme, GpuiFoundations, MainDay.
- A theme family at `assets/themes/marley/marley.json` with both variants, loaded as `ayu`,
  `gruvbox` and `one` are. Each of GPUI's 25 `Color` variants and the elevation levels map to the
  tokens on the Theme board: coat `#ff8c42` as the accent, eye green `#9be15d` for success and
  added lines, warm darks from `#0f0c0a` to `#2a221d`, honey `#f6c453` for warnings, pink
  `#ff6f91` for conflicts and needs-you, red `#ff6b5a` for errors.
- `assets/settings/default.json`: `theme.light` and `theme.dark` become Marley Day and Marley
  Night. A Zed-crate change: one ledger row.
- The Omarchy theme (#759) keeps working: following Omarchy still overrides.
- Visual check: `component_preview` in both themes, the Main board's screen in both themes.

**W0.2 The cat mark.** *Done (#829, 2026-10-20).* Board: Theme.
- The app icon at every size (`crates/zed/resources/app-icon*.png`, the desktop entry's icon),
  drawn from the Theme board's SVG; the title bar's mark.
- Chad sets the wording of the line about Marley wherever it shows (About, first run).

## Wave 1: Chad's asks

**W1.1 The rail shrinks to a strip.** *Done (#830, 2026-10-20).* *M–L.* Boards: RailStates, States (motion).
- `marley.rail_shrunk` (proposed setting), and a button beside the rail's search to toggle it.
- Shrunk: a 60 px strip of project marks with needs-you badges, a flyout of a project's rows on
  hover, and the search kept at the top as a button that opens the filter as a popover (Ctrl F
  still works).
- Code: `rail.rs`, `rail_switcher.rs`. The 180 ms width animation and reduced-motion jump come
  from the States board.

**W1.2 A project's mark from `logo.png`.** *Done (#831, 2026-10-21; notifications and the switcher done, #859, 2026-10-22; remote projects done, #860, 2026-10-22).* *M.* Board: ProjectLogo.
- Look for `logo.png`, then `logo.svg`, in the project's top folder; a file picked in Settings
  wins; otherwise the name's first letter on a tint of its own. `marley.project_logo` (proposed).
- Read once and cached; a remote project's file is read once over SSH and kept; a change to the
  file updates the mark.
- Shown in the rail's group header, the shrunk strip, the title bar, Home's recent projects, the
  switcher, and the project's notifications.
- Depends on W1.1 for the strip.

**W1.3 A Sessions view for tmux.** *Done (#832, 2026-10-21; the center Sessions page #861; faded rows, hosts not looked on, the page's columns, chips, Show, summary and host ticks, and the bottom row's chip opening the page, #930, 2026-10-25).* *M.* Board: TmuxSessions.
- A third choice at the top of the rail: Projects, Threads, Sessions. Every session by host, its
  windows, where it is placed and whether it is attached, with Attach or Show and Close.
- The hosts list from `ssh_connections`, each toggled into `marley.tmux_hosts` from the view.
- A Sessions tile in the shrunk strip, and the count in the bottom row (W1.4).
- Code: `tmux_sessions.rs` (#808, #821) already reads every session; this is a new view of it.

**W1.4 The bottom row says what has the focus.** *Done (#833, 2026-10-21; the panel group done, #862, and the problems and panels cards, #863, 2026-10-22; the debugging row and Ask an Agent about a server done, #901, 2026-10-22; the row for a thread, #907, for a terminal, #908, and for the other screens, #909, 2026-10-25).* *L.* Board: BottomRow.
- New `StatusItemView` items in `marley_workbench` for five focuses: a file, a terminal, an agent
  thread, debugging, and an agent that needs you (with Allow and Open in the row).
- The panel buttons become one labeled group, a dot on any panel already open as a tab.
- The cards that open above the row: language servers, problems, panels, edit predictions.
- Hiding or replacing Zed's own items is a change to `crates/zed/src/zed.rs`: a ledger row, kept
  to swapping items, not rewriting them.

## Wave 2: the core screens

**TICKET-827 Rail rows say their state.** *Done (#827, 2026-10-21; what is going on, pills and the Default tile, #910; the menus and the strip's +, #911, 2026-10-25).* *M.* Boards: Rail, RailMenus.
A tile per kind, the state dot, a colored second line, folded headers that still say what needs
you. Mostly `rail.rs` layout once W0.1 lands.

**TICKET-825 The agent thread with panels.** *Done (#825, 2026-10-21; plan, card labels, chips, terminal peek and checks done, #864, 2026-10-22; answer headers, filled labels, pills and the send square, #914, 2026-10-25; the composer's chips and Plan / Act, the details panel as cards with Open in a tab, #915, 2026-10-25; the width in fractions, ASKS with Allow and Deny, the page card, the handoff notice, the no-model card and the tab's thread menu, #927, 2026-10-25).* *L.* Boards: Main, ThreadStates, MainDay.
User messages as bubbles, the agent's plan, EDIT and RUN cards, reply chips, the rounded composer
with context chips, and a details panel (context use, files changed, its terminal, checks). The
thread view lives in Zed's `agent_ui`, so this is the largest upstream touch in the plan. Keep
each hunk additive, behind `MarleyThreadHost`, as #798 and #822 did. The details panel is a
Marley view beside the thread, not a change to Zed's.

**TICKET-826 Terminal blocks with the agent beside them.** *Done (#826, 2026-10-21; the agent beside the terminal, Apply and the progress line done, #865, 2026-10-22; rounded frames, state dots, the failed band, the action row, the running time and the bottom progress line, #916, 2026-10-25; the prompt card, the sticky header's words, session chips and Tell me when done, #917, 2026-10-25; the block map, a build's progress and the package hint, #918, 2026-10-25; the waiting-command card with Always run … here, the typing bar's words and keys, and the agent editor's key row, #928, 2026-10-25; the table tab's header, the restart divider, the dropped link's words and Try Now, the microphone's pill and the screen-read row, #929, 2026-10-25).* *L.* Boards: Terminal,
TerminalSession, TerminalMore, AgentTerminal.
Framed blocks with state colors, the failed block's actions, the running block's progress, the
prompt bar's chips, the block map, the agent bar, the waiting-command card, restored and remote
states. Block drawing is Marley's code.

**W2.1 The window chrome.** *Done (#834, 2026-10-21: the palette field and agent pills; tabs, the + menu and the split menu done, #866, 2026-10-22; tabs tinted by kind, #905, 2026-10-25).* *M–L.* Board: Chrome.
- Title bar: project mark and branch, the palette field, agent pills, Rusty and Marley buttons.
- Tabs as pills that show their kind and state; the + menu with Marley's real shortcuts; the
  split menu drawn as pictures; the tmux close prompt.
- Touches Zed's `title_bar` and the workspace's tab drawing: ledger rows, additive.

**W2.2 Home.** *Done (#835, 2026-10-21; the prompt's project chip #867; the board's layout #903, 2026-10-23; Marley and Rusty as chips #904).* *M.* Board: Home. `home_page.rs`: one prompt box with agent chips that starts the
picked agent in the picked project, then the cards (agents at work, tasks, follow-ups, Rusty,
recent projects, activity).

**W2.3 Threads with a preview.** *Done (#836, 2026-10-21; cards, states, header count, WHO tiles and the facts grid, #919, 2026-10-25).* *M.* Board: Threads. `threads_page.rs`: filters at the left,
cards by day, a preview with Open and Open in a Split.

## Wave 3: panels, drawers, pickers, notices

**W3.1 Side panels as one inspector.** *Done (#837, 2026-10-21; a choice beside #747's tabs; the segmented switcher, its count, the tab bar's button, Files' filter, letters and width, #923, 2026-10-25).* *L.* Board: SidePanels.
Files, Changes and Outline in one column beside the pane, with a switcher at its top, instead of
a full-width tab each. This changes #747's panel tabs (`panel_tabs.rs`), so it needs Chad's call
first (see the open decisions).

**W3.2 The bottom drawer.** *Done for Problems and Debug (#838, 2026-10-21; Output and Tasks later; Problems as a list, tab pills, Move, Maximize and Close, the card, #922, 2026-10-25).* *L.* Board: BottomPanel.
Problems, Debug, Output and Tasks as tabs of one drawer. Debug is Zed's debugger panel (#774
keeps it in the bottom dock); Problems is Zed's diagnostics. Drawing them as one drawer touches
Zed's dock code: start with Problems and Debug, and leave Output and Tasks for a later ticket.

**W3.3 Pickers.** *Done (#839, 2026-10-21; the agent cards a list, not a grid; the palette's plain names, tiles, chip, footer and "?" help, #920, 2026-10-25; the file finder's rows, key hint and preview header, #921, 2026-10-25).* *M.* Board: Palette. The command palette's groups, the agent picker as a grid of
cards with a line about each agent, the file finder's preview.

**W3.4 Needs you, toasts and prompts.** *Needs you done (#840, 2026-10-21); toasts done (#869, 2026-10-21); prompts done (#870, 2026-10-21); the agents' questions done (#871, 2026-10-21); the trust modal done (#872, 2026-10-21); Welcome to Marley done (#912); Wait, the tmux prompt and the second-launch notice done (#913, 2026-10-25); toast kinds, the plan entry's words and Open the plan, and the banner and push wording done (#925, 2026-10-25).* *M.* Boards: Inbox, Dialogs, MoreForms.
The inbox entries with their buttons, toasts, the trust, close, quit and apply prompts, the
pairing and agent-starts-agent prompts.

## Wave 4: forge

Forge (Ignibyte Forge) already keeps what the boards need: agent sessions with their host,
folder, ticket and last output; tickets with ten statuses and a pipeline phase; sprints with a
release branch; bulletins; and a knowledge base. It serves them over MCP (`agent-list`,
`ticket-list`, `sprint-current`, `bulletin-list`, `knowledge-search`, and more).

**W4.1 Forge as a fleet provider.** *Done (#841, 2026-10-21).* *L.* Board: ForgeAgents.
- A `forge` provider in `marley_fleet` that calls forge's MCP tools and answers in the fleet
  contract's shape, so the Fleet panel shows forge's agents with no new panel code.
- Settings (proposed): `marley.forge.enabled`, `endpoint`, `projects`, `poll_seconds`. The bearer
  token comes from the system keyring, never `settings.json`.
- Agent state follows forge's own rule: running and silent past 3 minutes reads needs attention.

**W4.2 The sprint board.** *Done (#842, 2026-10-21).* *L.* Board: ForgeBoard.
- A tab: columns for Open, In progress, In review, QA (qa-review, qa-failure, qa-pass) and Done,
  each card a ticket with its kind, phase and agent; a ticket's detail on the right.
- Actions through forge's tools: claim, move, comment; "Start here in a worktree" hands the
  ticket to New Agent in Worktree (#510).
- Depends on W4.1.

**W4.3 Forge around Marley.** *Done (#843, 2026-10-21).* *S–M.* Board: ForgeAgents.
A Forge section in the rail, a card on Home, a chip in the bottom row, bulletins.

## Wave 5: every other screen

Each is a ticket of its own, *S–M*, mostly layout once the theme is in:

| Screen | Boards |
|---|---|
| Settings window and every setting's control (done, #844, 2026-10-21; lists and maps stay in settings.json) | Settings, SettingsAll, Keymap |
| Rusty: Tasks (done, #845, 2026-10-21; Follow Up from Decisions due, task cards, LISTS swatches, page checkboxes, Linked here, property pills, ★ favourites and the Outline button, #939, 2026-10-28; the Knowledge panel to come, #948), Decisions and Follow Up (done, #874), Brain and pages (done, #875), Memory, Skills, Secrets, Capture, Rusty's Server (done, #876) | Tasks, MoreTabs, BrainPage, RustyTabs, RustyServer, Knowledge |
| MCP Servers (done, #846, 2026-10-21; the consent and pairing questions #871; the subtitle, agent tiles, pills, chips and Marley's own card, #933, 2026-10-25), Agent Activity (the row cards with each call's result or reason, #933, 2026-10-25; Rusty's calls, #934, 2026-10-25), the server form, what agents may do (the edit question, Kept from agents and the level words, #938, 2026-10-28; the pairing card to come) | Tools, MoreForms, AgentAccess |
| System One (done, #847, 2026-10-21; the System One page with each use's mode, the calls heading and failed rows tinted, #937, 2026-10-28) | SystemOne |
| Fleet panel (done, #848, 2026-10-21; the snapshot beside the list at 760 px and the headline, #935, 2026-10-25), the session tab (done, #878), harness sessions in the rail, scheduled runs and the seat form (done, #886), 2026-10-22 | Fleet, MoreTabs, OpenConnect |
| Component Library (done, #849, 2026-10-21; the template chips, Try it's header and the chosen name, #936, 2026-10-28) and Component tabs | Components, MoreTabs |
| Browser tab (toolbar done, #850, 2026-10-21; the recording's minute, Draft a test and the scripts' last runs, #932, 2026-10-25; the pick marks, chips, drawer, card head, Note, To, take-over and dialogs' words, #931, 2026-10-25; picks panel #879, console drawer #887, recordings and Send #892, downloads and pick options #896 done, 2026-10-22) and its tools | Browser, BrowserTools |
| Worktrees (New Agent in Worktree done, #851, 2026-10-21), plan review (cards done, #881), the guide (done, #888), the diff review's header, the guide's Send To, plan review's tabs and turns (done, #893), 2026-10-22; plan review's foot, notes and turn pills, the gutter mark and Send to's states (done, #924), the Merge prompt's checks and Remove's words (done, #926), 2026-10-25 | Worktrees, Review, DiffReview |
| Workflows (Save and Run done, #852, 2026-10-21), launch configs (approval done, #882), the block filter (done, #889), 2026-10-22 | Workflows, OpenConnect |
| Splits (focused pane done, #853; switcher done, #883; 2026-10-22; tinted by kind with state dots, #905, 2026-10-25), ports, containers, collab (done, #890, 2026-10-22) | Splits |
| Project search, branches and extensions (done, #891), recent (done, #854), the guide's look (done, #884), 2026-10-22; the guide's Try it (done, #897; history is Zed's git panel's History tab) | SearchMore |
| New pages: this build, data folders (done, #855), troubleshooting and the rest (done, #885), 2026-10-22 | SystemPages |

## Wave 6: polish

- **Editor pop-ups** (EditorPopups; completions, hover and code actions done, #856, 2026-10-22;
  Ask an agent to fix this and the rename note done, #899; inline assist's line done, #900): completions, hover, code actions,
  rename, inline assist.
- **Empty and error states** (States; done, #857 and #894, 2026-10-22): no project, no threads, no filter match, Rusty dropped,
  Chromium missing, an agent CLI missing, forge offline, the harness stopped.
- **Motion and focus** (States; the needs-you pulse done, #858, 2026-10-22; the labels' fade and one focus ring done, #895): the rail's shrink, the needs-you pulse (two pulses, then still),
  one focus ring everywhere, reduced motion honored.

## Across every wave

- **Upstream.** Every Zed-crate change is additive, small and in `zed-touchpoints.md` before it
  is written. The ones this plan expects: `default.json` (themes), `zed.rs` (status items),
  `title_bar`, the workspace's tab drawing, `agent_ui`'s thread view, and the dock (W3.2).
- **Visual checks.** Each ticket's scenario drives the real app and reads its shots; a theme or
  color change is shot in both Marley Night and Marley Day.
- **The canvas stays the reference.** A ticket that lands differently from its board says why in
  its notes; the board is updated when Chad prefers the result.
- **Settings.** The proposed keys (`marley.project_logo`, `marley.rail_shrunk`,
  `marley.forge.*`) are added with the ticket that uses them, each with its doc comment and its
  row on the settings page.

## Open decisions for Chad

1. **Side panels (W3.1):** does the inspector replace #747's full-width panel tabs, or sit beside
   them as a choice?
2. **The line about Marley (W0.2):** its wording, and where it shows.
3. **Forge (W4.1):** Marley adapts forge's tools to the fleet contract (this plan), or forge
   learns to serve the contract itself.
4. **The bottom drawer (W3.2):** start with Problems and Debug only, or all four tabs?
5. **Light mode:** does Marley Day become the default whenever the system is light?

## Tickets

Filed 2026-10-20, in the Queue of `docs/planning/tickets/BACKLOG.md` in this order:

| Wave | Tickets |
|---|---|
| 0 | TICKET-828 (themes), TICKET-829 (the cat mark) |
| 1 | TICKET-830 (shrinking rail), 831 (logo.png), 832 (tmux Sessions), 833 (bottom row) |
| 2 | TICKET-827 (rail rows), 825 (thread with panels), 826 (terminal blocks), 834 (chrome), 835 (Home), 836 (Threads) |
| 3 | TICKET-837 (inspector), 838 (bottom drawer), 839 (pickers), 840 (Needs you and prompts) |
| 4 | TICKET-841 (forge provider), 842 (sprint board), 843 (forge around Marley) |
| 5 | TICKET-844 to 855, one per area of the Wave 5 table |
| 6 | TICKET-856 (editor pop-ups), 857 (empty and error states), 858 (motion and focus) |

The five open decisions were filed with the plan's defaults, each named in its ticket's Decision
section: the inspector as a choice beside #747's tabs, the Theme board's line about Marley, Marley
adapting forge's tools, Problems and Debug first in the drawer, and Marley Day as the light
default.
