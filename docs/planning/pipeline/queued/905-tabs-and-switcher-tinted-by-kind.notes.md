# Tabs and the Ctrl Tab switcher tinted by kind — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-905-tabs-and-switcher-tinted-by-kind.md
- **Pipeline spec:** 905-tabs-and-switcher-tinted-by-kind.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 1, "What is wrong every day": terminal tabs show two icons;
  tab and Ctrl Tab tiles are one grey for every kind; a thread tab shows its title's first
  letter).
- **Classification / tier:** feature, S–M. Marley crate, plus one small hunk in `terminal_view`.
  Build order of the review's tickets: 906, 905, 907, 908, 909, 910, 911. This one lands the
  shared palette the later ones reuse.
- **Recall (§18.3):**
  - F-claude-865: a Marley function a Zed view calls from its render never reads that view.
    `tab_mark` runs inside the pane's render and `switcher_row` inside the picker's; both read
    items (the thread tab, the terminal, the Browser view), never the pane or the picker. The
    terminal view's new global is asked from `tab_content` and is handed only `&App`; it must not
    read the terminal view.
  - F-claude-893 and PR-claude-831: a view a workspace hook makes never reads its workspace in an
    update. The switcher's project lookup reads the window's workspace from the picker's render,
    which runs outside a workspace update; the tab dot needs no project, so `tab_mark` reads no
    workspace at all.
  - F-claude-903: stand-ins first on the terminals' PATH through the scenario's `.bashrc`.
  - F-claude-880: tmux on the run's private socket only (`TMUX_TMPDIR`).
  - #866 D1 (a terminal tab's dot), #883 D1 and D2 (the switcher's tile and line), #827 (the rail's
    kind tints, "the icons stay").
- **Discovery:**
  - `crates/marley_workbench/src/tab_tiles.rs`: `tab_mark`, `state_dot`, `glyph_of`, `tile`,
    `switcher_row` (whole file, 132 lines).
  - `crates/marley_workbench/src/rail.rs`: `RowState` (:11000-11031), `RowKind` (:11033-11058),
    `row_card`'s tile (:11174), `thread_row_icon` (:11261), `thread_row_state` (:11276),
    `tmux_row` (:8264).
  - `crates/marley_workbench/src/home_page.rs`: `cli_glyph` (:925).
  - `crates/terminal_view/src/terminal_view.rs`: `TerminalView::tab_content` (:2067-2130).
  - Read, not changed: `workspace/src/pane.rs` (:2883-2891, :4347-4357),
    `tab_switcher/src/tab_switcher.rs` (:734-746).
- **Decisions:** D1 to D5 in the spec.

### Design
- Approach.
  - `item_kind.rs` (new, Marley crate) owns `ItemKind`, `AgentMark` and `ItemState`.
    `ItemKind::tint` is `ink.opacity(0.15)`, as the rail's Browser and Port tints already are;
    `ItemKind::ink` reads the theme's colours named in D2. `AgentMark::from_cli(AgentKind)` and
    `AgentMark::from_thread_agent(&AgentId, &Entity<Project>, cx)`; the latter tells Claude Code
    and Codex by `CLAUDE_AGENT` and `CODEX_AGENT` (made `pub(crate)` in `agents.rs`), Marley and
    Rusty by `assistant.rs`'s entry names, and any other agent by the first letter of
    `thread_agent_name`. A thread tab has no project at hand in `tab_mark`, so the other-agent
    case takes the agent id's first letter there; the ACP agents Marley knows by name are covered.
  - `rail.rs` imports `ItemKind` and `ItemState` in place of its own `RowKind` and `RowState`;
    `row_card` keeps its slot and dot code. The rail's thread rows take their agent's tint.
  - `tab_tiles.rs`: `kind_of` replaces `glyph_of`; `tile(kind, width, cx)` draws the tint and ink;
    `state_of(item, cx) -> Option<ItemState>` replaces `state_dot`, which becomes
    `state.dot(cx)`; `switcher_row` puts the dot in the tile's corner (absolute, ringed in the
    picker's background, as the rail's tile does) and builds the line from `state_words` and the
    project.
  - `terminal_view.rs` (Zed crate): `pub struct MarleyTerminalTabIcon(pub fn(&App) -> bool)` and
    `impl Global`; `tab_content` skips its icon block (the icon and the rerun button) when the
    global says no. `tab_tiles::init` sets it to "draw the icon unless the Marley layout is on".
- File manifest.
  - `crates/marley_workbench/src/item_kind.rs` — Marley crate, new.
  - `crates/marley_workbench/src/marley_workbench.rs` — Marley crate, `mod item_kind;`.
  - `crates/marley_workbench/src/tab_tiles.rs` — Marley crate.
  - `crates/marley_workbench/src/rail.rs` — Marley crate.
  - `crates/marley_workbench/src/home_page.rs` — Marley crate.
  - `crates/marley_workbench/src/agents.rs` — Marley crate (two constants made `pub(crate)`).
  - `crates/terminal_view/src/terminal_view.rs` — Zed crate, one hunk with a `// Marley:` comment.
  - `script/e2e/905-tabs-and-switcher-tinted-by-kind.sh` — the scenario.
- Ledger rows: `docs/marley/zed-touchpoints.md`, the `crates/terminal_view/src/terminal_view.rs`
  row gains "the global `MarleyTerminalTabIcon`, which `tab_content` asks whether to draw its icon
  and rerun button; the workbench says no in the Marley layout, where the tab's tile is its mark
  (#905)", with the merge note "keep the ask around the icon block of `tab_content`; with no
  global set, the icon draws".

### Visual check plan
- REQ-001, REQ-002, REQ-003, REQ-004: the scenario opens a Marley thread (scripted ACP agent), an
  agent terminal (stand-in `claude`), a tmux tab (private socket), a Browser tab (fixture page),
  `rail.rs` and a shell that ran `false`, then shoots the tab bar: `905-01-tabs`. Read: each
  tile's fill and glyph colour by kind, `M` on the thread tab, one mark on each terminal tab,
  dots on the thread, Browser, tmux and failed-shell tabs.
- REQ-005, REQ-006: `tab switcher: toggle` from the palette (it stays open with no key held, as in
  #883's scenario): `905-02-switcher`. Read: tinted tiles with corner dots; each line
  "<project> · <state>".
- REQ-007: the rail with the same rows: `905-03-rail`. Read: the rows keep their layout; the tmux
  tile is teal; the thread row's tile takes Marley's tint. The rest is the diff review.
- REQ-001, REQ-005 in Day: `profile_setting theme '"Marley Day"'`, the tab bar and the switcher:
  `905-04-day`.
- REQ-008: the diff review (the hook returns "draw" outside the Marley layout; `tab_mark` already
  returns none there).
- Not reached: a Codex or Rusty thread tab (`CX`, `R`) needs a second ACP stand-in; the glyph
  table is checked in the diff review, and the scenario covers `M` and `CC`.

### Risks
- `switcher_row` finds the project through `window.root::<MultiWorkspace>()`. If the switcher
  ever renders inside a workspace update, that read panics (F-claude-893). The first run's shot
  shows it; the fallback is the item's own worktree name through its project path, no workspace.
- Hiding the rerun button removes a hover action from task tabs in the Marley layout (Out names
  it). A tmux tab is a task terminal, so this covers it too.
- Turning a running shell's dot from the accent to amber changes #866's D1; Chad sees it in the
  install.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
