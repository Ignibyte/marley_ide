# Design review, 2026-10-23: what the boards draw that Marley does not

Chad, 2026-10-23: "there is quite a bit of things in the design thats not built. home page being
one". Home was rebuilt first (#903). Then seven reviewers compared the other 49 screen boards of
the design canvas "Marley, Warp-like" with Marley at `3ffbab6a9c`. They used shots of the debug
build from the two design tours, the closed tickets and their specs, and the code. The six
reference boards (Coverage, Actions, SettingsAll and the three GPUI catalogs) were left out.

Each file here is one reviewer's report: a section per board, a bullet per gap. Each gap names
what the board draws, what Marley does now with its evidence (a shot, a `file:line` or what was
grepped), its kind and a size.

| File | Boards |
|---|---|
| [A-core.md](A-core.md) | Main, MainDay, Terminal, TerminalSession, AgentTerminal, TerminalMore, Workflows |
| [B-rail-chrome.md](B-rail-chrome.md) | Rail, RailStates, RailMenus, ProjectLogo, TmuxSessions, Chrome, BottomRow, Splits |
| [C-threads.md](C-threads.md) | Threads, ThreadStates, Inbox, Review, DiffReview, Worktrees, Palette |
| [D-panels-editor.md](D-panels-editor.md) | SidePanels, BottomPanel, EditorPopups, SearchMore, Keymap, Settings, States, Theme |
| [E-rusty.md](E-rusty.md) | Tasks, Knowledge, BrainPage, RustyTabs, RustyServer, MoreTabs, MoreForms |
| [F-tools.md](F-tools.md) | Browser, BrowserTools, Tools, Fleet, Components, SystemOne, AgentAccess |
| [G-system.md](G-system.md) | Dialogs, OpenConnect, SystemPages, ForgeBoard, ForgeAgents |

## What it found

About 375 gaps. No board is built in full, but most gaps are small. Two thirds are `look`
(the thing exists and is drawn differently, size S). About 110 are `missing` (no ticket left them
out). The rest are `deferred`: a ticket left them out on purpose and says why.

Several tour shots missed their screen: the Rusty palette entries of the first tour ran with Rusty
off and landed on the Component Library (`tour-15` to `tour-18`), `t2-04` repeats Tasks, `t2-10`
shows no open menu and `t2-15` shows the split menu, not a split. Those screens were judged from
the code; each ticket's own visual check sees them.

## Build order

Daily use first, then each area whole. Each line becomes one ticket, taken whole.

1. **What is wrong every day (S each).** The Threads page squeezes its list to about 100 px in a
   half-width pane (C). Terminal tabs show two icons (B). Tab and Ctrl Tab tiles are one grey for
   every kind, and a thread tab shows its title's first letter, not its agent (B). The Git and
   Outline tabs say "Git Panel" and "Outline Panel" (D). Primary buttons use the light orange, not
   the coat (D). The open group header has no summary (B).
2. **The bottom row says what is in front, for every screen (M).** The thread (model, mode, what
   it does and for how long, context, files changed), the terminal (folder, branch and changes,
   Ask an agent on a failed block, size), the Threads page, the Browser, MCP Servers, the Fleet,
   the Component Library and Rusty's tabs (A, B, C, E, F).
3. **Rail rows say what is going on (M).** A thread row's line says what the agent does or asks; a
   shell row's line shows progress, or the exit and Ask an agent; the folded header's pills; the
   menus' missing entries (B).
4. **First run and dialogs (L).** "Welcome to Marley" in place of Zed's onboarding; Wait in the
   close dialog; a notice when a second launch hands a folder over (G).
5. **The agent thread (M).** A header on each answer (agent, model, effort, how long), the
   composer's context chips, the details panel's state line and tiles, filled EDIT and RUN labels,
   the Plan / Act switch, the send button (A).
6. **Terminal blocks (M).** Rounded frames with a state dot, a ticking time, the action row under a
   block's output, the prompt as a card, the sticky header's wording, the block map (A).
7. **The Threads page (M).** Cards with a state dot and a snippet, the header count and New Agent,
   the preview's state line and facts (C).
8. **The palette and the file finder (M).** Plain command names, glyph tiles, the scope chip and
   footer; file rows with an extension tile, the path under the name and a status badge (C).
9. **The bottom drawer (M–L).** Problems as a filterable list of rows, the drawer's own Move /
   Maximize / Close, count pills on its tabs (D).
10. **The side panels (M).** The Inspector's segmented switcher and button, a filter in Files, the
    Changes panel as the board lays it out, the breadcrumb (D).
11. **Each remaining board, one ticket each, in this order:** Review and DiffReview, Inbox,
    Worktrees, ThreadStates, AgentTerminal, TerminalSession, TerminalMore, TmuxSessions, Splits,
    Browser and BrowserTools, Tools, Fleet, Components, SystemOne, AgentAccess, Rusty's boards,
    Settings and Keymap, SearchMore, EditorPopups, States, OpenConnect, SystemPages, Forge.

## For Chad

- **Deferred items.** Each report marks what a ticket left out on purpose. Some of that was Chad's
  own call (Files as a pop-out, #868; one focus ring skipping rows and pickers, #895). The rest
  were defaults taken while building. They stay out unless Chad says to build them.
- **Fonts.** The boards set words in Geist and code in Geist Mono; Marley ships Zed's fonts. A
  change of fonts reaches every screen, so it waits for Chad's word.
