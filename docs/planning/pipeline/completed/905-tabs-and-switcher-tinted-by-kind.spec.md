---
pipeline_id: b70bec50-be1f-46f0-b89d-2009df4a2c95
ticket: docs/planning/tickets/open/TICKET-905-tabs-and-switcher-tinted-by-kind.md
status: Phase 4 — Complete PASS
title: Tabs and the Ctrl Tab switcher tinted by kind
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/B-rail-chrome.md, docs/marley/design-review-2026-10-23/README.md]
---

## Title
Center tabs and Ctrl Tab switcher rows tinted by kind, as the Chrome and Splits boards draw them.
A thread tab shows its agent, not its title's first letter. A terminal tab shows one mark, not
two. Thread, Browser and tmux tabs carry a state dot, as terminal tabs already do. A switcher
row's tile carries the dot too, and its line says "project · state". The kind tints, the agent
glyphs and the state colours become one shared palette in a module of their own, so the rail, the
tabs, the switcher and the bottom row (#907, #908, #909) draw from one owner.

## Scope
### In
- `item_kind.rs` (Marley crate, new): the shared palette.
  - `ItemKind`: `Thread(AgentMark)`, `AgentTerminal(AgentMark)`, `Shell`, `Tmux`, `Browser`,
    `File`, `Port`, with `tint(cx)` (the tile's fill) and `ink(cx)` (the glyph's colour).
  - `AgentMark`: Claude Code `CC`, Codex `CX`, Gemini `G`, OpenCode `OC`, Marley `M`, Rusty `R`,
    and any other agent by its display name's first letter. Built from a CLI's `AgentKind` or a
    thread's `AgentId`.
  - `ItemState`: `Working`, `NeedsYou`, `Done`, `Failed`, `Idle`, with `dot(cx)` and `line(cx)`.
  - `rail.rs`'s `RowKind` and `RowState` move here and become these types; their colours for the
    rail's existing rows stay as they are, except where D2 sets the board's tint.
- `rail.rs` (Marley crate): draws its tiles and dots from `item_kind`; no other change to rows.
- `home_page.rs` (Marley crate): `cli_glyph` gives way to `AgentMark::glyph`.
- `tab_tiles.rs` (Marley crate):
  - `glyph_of` becomes `kind_of(item, cx) -> ItemKind`: a `ThreadTab` by its conversation's agent;
    a terminal running an agent CLI (`agent_bar::agent_in`) by that agent; a terminal attached to
    tmux (`TmuxHosts::attach_of` on its task's label) as `Tmux`; a `BrowserView` as `Browser`; a
    file by its extension; else `Shell` for a terminal and the title's first letter for the rest.
  - `tile` fills with the kind's tint and draws the glyph in its ink.
  - `state_dot` covers every kind: a thread by `marley_rail::thread_status`; an agent terminal
    waiting on the user (`agent_events::waiting_agent`) as needs you; a Browser tab loading, with
    a dialog open, or loaded (`BrowserHub::is_loading`, `BrowserHub::dialog`); a tmux tab idle; a
    shell as today (running, failed, unread).
  - `switcher_row`: the tile carries the state dot at its corner, as the Splits board draws it;
    the line reads "<project> · <state>".
- `terminal_view.rs` (Zed crate, hunk in `TerminalView::tab_content`): a global
  `MarleyTerminalTabIcon`, a function the workbench sets, that says whether the tab draws its own
  icon. In the Marley layout it draws none, so the tile is the tab's one mark. Without the global,
  or in Zed's layout, Zed's icon and its rerun button stay. Its row goes in
  `docs/marley/zed-touchpoints.md` (the `terminal_view.rs` row).

### Out (explicitly deferred)
- Rounded tab pills and the "+3 more" overflow: #866 Out (the tab shape).
- The rail's rows drawing letter glyphs in place of their icons: #827 Out ("the icons stay"). The
  rail takes the shared tints only.
- State on the rail's Browser and tmux rows: #827 Out and #827's scope. The tab gets the state
  here; the rail row does not.
- Glyph tiles in the + menu: #866 Out.
- A project's mark on its tab (ProjectLogo board, missing): a follow-up ticket.
- Zed's back and forward arrows in the tab bar (Chrome board, look): a follow-up ticket.
- A task terminal's rerun-on-hover in the Marley layout: it lived in Zed's icon slot, which goes
  (D4). `task: rerun` and its key still run it.

## Reference (§20)
Upstream Zed, workspace and tab switcher behaviour kept: `workspace::pane`'s tab bar (its
`render_tab`, close button, indicator and drag) and `tab_switcher`'s picker (order, keys,
footer) stay Zed's; Marley changes only what its existing globals (`MarleyTabLook`,
`MarleySwitcherLook`) draw, plus one new global in `terminal_view`'s `tab_content`. The look
follows the Chrome and Splits boards. No terminal or block behaviour changes, so no Warp
reference applies.

### Prior art
- `tab_tiles.rs`: `tab_mark` (:27), `state_dot` (:38, terminals only), `glyph_of` (:85, a thread
  falls through to `project_icons::initial`), `tile` (:101, one `element_background` fill and a
  muted glyph for every kind), `switcher_row` (:124, the line is the path or the tooltip).
- `workspace/src/pane.rs`: `render_tab` puts the `MarleyTabLook` tile in the icon's place and
  draws its dot (:2883-2891); `MarleyTabMark { tile, dot }` and `MarleyTabLook` (:4347-4357). The
  dot already has a slot for any kind, so dots on thread, Browser and tmux tabs need no Zed hunk.
- `tab_switcher/src/tab_switcher.rs`: `MarleySwitcherRow { tile, line }` and
  `MarleySwitcherLook` (:734-746), asked in `render_match`. The tile is any element, so the dot
  goes inside it with no Zed hunk.
- The doubled icon: `terminal_view.rs` `TerminalView::tab_content` (:2067-2130) draws its own
  `Icon` (`IconName::Terminal`, or a task's status icon with its rerun button) before the title,
  inside the label the pane draws after the tile (:2098, :2121). The pane cannot drop it; the
  view has to.
- `rail.rs`: `RowState` with `dot` and `line` (:11000-11031), `RowKind` with `tint` (:11033-
  11058: Thread `warning_background`, AgentTerminal `info_background`, Shell `raised`, Browser
  `text_accent` at 15 %, Tmux `hint_background`, Port the theme's cyan, noted as "the Splits
  board's teal"), drawn in `row_card` (:11174).
- `home_page.rs` `cli_glyph` (:925, CC / CX / G / OC) and `glyph_tile` (#903); `agents.rs`
  `CODEX_AGENT` (:283) and `CLAUDE_AGENT` (:1180); `assistant.rs` `assistant_icon` (:211), which
  tells Marley's and Rusty's threads by their agent id; `thread_tab.rs` `tab_icon` (:301).
- State sources: `marley_rail::thread_status` (`marley_rail.rs:444`); `agent_bar::agent_in`
  (`agent_bar.rs:141`); `agent_events::waiting_agent` (`agent_events.rs:274`);
  `BrowserHub::is_loading` and `BrowserHub::dialog` (`browser.rs:1148`, `:1165`) by
  `BrowserView::target` (:5234); `TmuxHosts::attach_of`, as `Rail::tmux_attached_here` reads it
  (`rail.rs:5156`).
- The theme (`assets/themes/marley/marley.json`) has every board colour in both themes: amber
  `warning`, pink `conflict`, green `success`, coat-light `text.accent`, teal
  `terminal.ansi.cyan`, violet `terminal.ansi.magenta`.
- The boards: Chrome's `tab(glyph, glyphInk, title, dot, selected)` and Splits'
  `row(glyph, tile, ink, dot, title, where)` (the design record,
  `docs/design/marley-canvas-2026-10-20/project/`).
- Published material: none applies (a look over existing state). Warp and Orca maps: no tab
  tinting seam to adopt.

## UI proof
Scenario `script/e2e/905-tabs-and-switcher-tinted-by-kind.sh` (sway). Setup: a scratch repository
with `rail.rs`; the Marley entry on #798's scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) for a
thread tab; #542's stand-in `claude` first on the terminals' PATH through the scenario's
`.bashrc` for an agent terminal; a tmux session on #827's private socket (`TMUX_TMPDIR`, #880)
attached in a tab; a Browser tab on `browser-fixture.sh`'s local page; a shell whose last command
is `false`.
- `905-01-tabs`: the tab bar: each tile in its kind's tint, `M` on the thread tab, `CC` on the
  agent terminal, `tx` on the tmux tab, one mark on each terminal tab, dots on the thread,
  Browser, tmux and failed-shell tabs (REQ-001 to REQ-004).
- `905-02-switcher`: Ctrl Tab open: tinted tiles with their dots, each line "project · state"
  (REQ-005, REQ-006).
- `905-03-rail`: the rail's rows on the shared tints (REQ-007).
- `905-04-day`: the tab bar and the switcher in Marley Day (REQ-001, REQ-005).

## Locked-In Decisions
- D1 — One palette, one owner: `item_kind.rs` holds the kind tints, the agent glyphs and the
  state colours. The tabs, the switcher and the rail draw from it now; the bottom row's chips
  (#907, #908, #909) and the rail's row lines (#910) reuse it rather than add their own.
- D2 — The boards' colours, through the theme: an agent by its own colour (Claude Code amber
  `warning`, Codex violet `terminal.ansi.magenta`, Marley `text.accent`, Rusty teal
  `terminal.ansi.cyan`, any other agent amber); a file and a Browser tab `text.accent`; tmux teal
  `terminal.ansi.cyan`; a shell muted. The tint is the ink at low opacity, so Marley Day follows.
  The rail's tmux tile turns teal with it, as the Rail board draws it.
- D3 — The state dots are the rail's: working amber, needs you pink, done green, failed red, idle
  grey. A running shell's dot turns from the accent to amber, as the Splits board draws "repo ·
  cargo run". A loaded Browser tab is done (green), a loading one working, one with a dialog open
  needs you.
- D4 — In the Marley layout a terminal tab draws no icon of its own: the tile is its one mark, and
  a task's status shows as the dot. In Zed's layout nothing changes.
- D5 — The switcher's line: the project is the worktree the file or the terminal's folder lies in,
  else "Home". The state is: a thread's state in words ("needs you", "working", "failed", "idle");
  a terminal's running command or "exit N"; "tmux" for a tmux tab; a Browser tab's address host,
  or "loading"; a file's path in its project, as #883 had it. A row whose state says nothing more
  shows the project alone.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, each center tab's tile shall be filled with its kind's tint and draw its glyph in the kind's ink: an agent's colour for a thread or agent terminal, orange for a file and a Browser tab, teal for tmux, muted for a shell. | `905-01-tabs`, `905-04-day` |
| REQ-002 | WHEN a tab holds an agent thread, its tile shall show the agent's glyph (`CC`, `CX`, `M`, `R`, …), not the first letter of the thread's title. | `905-01-tabs` |
| REQ-003 | WHILE the Marley layout is on, a terminal tab shall draw one mark, its tile, with no Zed terminal icon after it. | `905-01-tabs` |
| REQ-004 | WHEN a thread, a Browser tab or a tmux tab has a state, its tab shall carry that state's dot, as a terminal tab does. | `905-01-tabs` |
| REQ-005 | WHEN the Ctrl Tab switcher is open, each row's tile shall be tinted by kind and carry its state's dot at the corner. | `905-02-switcher`, `905-04-day` |
| REQ-006 | WHEN the Ctrl Tab switcher is open, each row's line shall read the project and the tab's state ("<project> · <state>"). | `905-02-switcher` |
| REQ-007 | The rail, the tabs and the switcher shall take their tints, glyphs and dot colours from the one `item_kind` module, and the rail's rows shall keep their layout. | the review of the diff; `905-03-rail` |
| REQ-008 | WHILE Zed's layout is on, the tabs and the terminal tab's icon shall stay as Zed draws them. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `item_kind.rs`, then the rail and Home moved onto it, then `tab_tiles.rs`, then the
  `terminal_view` hunk with its ledger row; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
