---
pipeline_id: f53808a1-8d4e-499f-98a3-e2361cc1988c
ticket: docs/planning/tickets/closed/TICKET-785-a-split-opens-home.md
status: Phase 4 — Complete PASS
title: "A split opens Home, and a tab's menu splits it out"
type: feature
slice: the Marley layout
references: [docs/marley/workbench-shell.md]
---

## Title
Splitting a center pane clones its active tab (`SplitMode::ClonePane`). A terminal or an agent CLI
tab clones through Zed's `TerminalView::clone_on_split`, which starts a new shell in the new pane,
so every split of a terminal forces one open. Home (#701, #782) cannot be cloned, so splitting it
does nothing and the split button is greyed out. Chad, 2026-10-15: "the split left and right we
shouldnt force a terminal to opne. instead we need open a home instead", approved as "When you
split a terminal or agent tab left, right, up or down, the new pane opens on Home instead of a new
terminal", then "also you should be able to right click on a tab split it out in another
direction."

## Scope
### In
- `crates/marley_workbench`, in the Marley layout: a split of a center terminal (a shell or an
  agent CLI) by the split button, a key or the palette opens Home in the new pane. Home splits
  into a second Home. A tab's right-click menu lists Split Right, Split Left, Split Up and Split
  Down, which move that tab into a new pane on that side; a pane the move would leave empty shows
  Home.
- `crates/workspace/src/pane.rs`: a hook for a Marley crate's entries at the end of a tab's
  right-click menu, as #699's hook does for the + menu; and the split button's menu given its
  pane's focus as context, without which its choice reached no pane (found in the Test probes,
  in the installed build too).

### Out (explicitly deferred)
- The Zed layout: splits and the tab menu stay Zed's, as the layout is there to compare with stock
  Zed.
- A file, a diff or any other tab Zed clones: it still splits into a second view of itself.
- The terminal dock's own split, which gives a new terminal in the dock.
- Tabs that cannot be cloned and are not terminals (a thread tab, the Browser): a split by a key
  still does what Zed does.
- Dragging a tab to a pane's edge, which already moves it.

## Reference (§20)
- **Upstream Zed (workspace crate):** `Pane::split` emits `Event::Split`, and `Workspace` clones
  the active item (`split_and_clone`), opens an empty pane (`split_pane`) or moves the item
  (`split_and_move`); `MovePane` on a pane's only tab opens an empty pane on the other side
  instead. Marley keeps every mode and catches only the clone of a center terminal, before Zed's
  pane handles the action, as #747's panel tabs do (D4 there).
- **Marley-specific** for the new pane's content: Warp's split opens a new session, which is what
  Chad asked to stop; Home is Marley's own page.
- **Published material:** VS Code's editor tab menu offers Split Up, Down, Left and Right on a tab;
  Marley's entries do the same for any tab and move it, as dragging it to that edge does in Zed.

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/03-terminal-session-core.md` lists
  `pane.split` among Warp's actions with a new session in it; no `docs/zed_architecture/` map
  covers `workspace`. #747 (`panel_tabs::move_split`) catches the split keys at the workspace
  root for a panel tab and turns them into a move; this ticket adds the terminal case beside it.
- **Published material:** VS Code's tab context menu (Split Up/Down/Left/Right).
- **Code we already ship:** `Workspace::split_pane`, `split_and_move`, `Pane::split` and
  `Item::can_split`/`clone_on_split` own every split; `home_page::MarleyHome` and `open_here`
  own Home; `MarleyNewItemMenu` (#699) is the precedent for a hook in a pane menu. No crate owns a
  split that opens a page of its own.

## UI proof
`script/e2e/785-a-split-opens-home.sh`, under `compositor sway` (clicks on the split button and a
tab's menu). A scratch repository opened, its first terminal in front. Shots `785-01-split-right`
to `785-06-file-split`; checks through Marley's MCP server (`terminals`) that no second
terminal started.

## Locked-In Decisions
- D1 — A capture listener at the workspace root, beside #747's, takes the split keys
  (`pane::SplitLeft/Right/Up/Down` and `SplitHorizontal/Vertical`) in `ClonePane` mode when the
  Marley layout is on, the center's active pane holds the focus, and its active item is a
  `TerminalView`. It stops the action, opens an empty pane on that side (`split_pane`) and puts a
  new Home there with the focus. Every other case goes on to Zed.
- D2 — `MarleyHome` answers `can_split` with true and `clone_on_split` with a new Home, so Zed's
  own clone splits Home and its split button is live.
- D3 — `MarleyTabMenu` in `pane.rs`, a `Global` holding a function that takes a tab's menu, its
  pane, its workspace and the tab's item id, called last in the tab menu's builder. Marley's
  function adds a separator and the four Split entries, in the Marley layout only and for a center
  pane only (`Workspace::panes`). They come last so the menu's End key reaches them.
- D4 — A Split entry on a pane with other tabs activates the tab and moves it
  (`split_and_move`). On a pane's only tab it opens the new pane on the other side
  (`split_pane` with the opposite direction), as Zed's `MovePane` does, and puts Home there, so
  the tab ends up on the side picked and no pane is blank; the tab keeps the focus.
- D5 — Zed hunks in `pane.rs` only, with its row in `docs/marley/zed-touchpoints.md`.
- D6 — The split button's menu gets `.context(pane focus)`, as the tab menu has `.context(menu_context)`:
  without it a choice was dispatched where no pane hears it, so the button did nothing on a file
  or Home (a terminal worked only through D1's listener at the workspace root). D1 then finds the
  pane the split is for as the center pane holding the focus, since the menu focuses its own pane
  before the workspace's active pane follows.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a center terminal is split with the split button's Split Right, the system shall open Home in a new pane on the right and start no terminal | shot `785-01-split-right`; MCP `terminals` lists one terminal |
| REQ-002 | WHEN a center terminal is split by its action (`pane: split down` from the palette), the system shall open Home in a new pane below | shot `785-02-split-down` |
| REQ-003 | WHEN Home is split with the split button, the system shall open a second Home in the new pane | shot `785-03-split-home` |
| REQ-004 | WHEN Split Down is chosen in the right-click menu of a tab whose pane holds other tabs, the system shall move that tab into a new pane below | shot `785-04-tab-menu` |
| REQ-005 | WHEN a Split entry is chosen in the right-click menu of a pane's only tab, the system shall put the tab on that side and Home in the new pane on the other | shot `785-05-split-only-tab` |
| REQ-006 | WHILE the Zed layout is on, splits and the tab menu shall be Zed's | review of the diff (`marley_layout` guards) |
| REQ-007 | WHEN a file tab is split with the split button, the system shall show the file in the new pane, as Zed does | shot `785-06-file-split` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the hook in `pane.rs` and its ledger row, Home's clone, the capture and the menu
  entries; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
