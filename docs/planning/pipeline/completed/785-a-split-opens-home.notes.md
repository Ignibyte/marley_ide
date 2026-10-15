# A split opens Home, and a tab's menu splits it out — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-785-a-split-opens-home.md
- **Pipeline spec:** 785-a-split-opens-home.spec.md

## Phase 1 — Plan (2026-10-15)
- **Request:** Chad, 2026-10-15, approved as one sentence, then the tab menu added; "plan and
  continue" for the queue.
- **Classification / tier:** feature, the Marley layout; one Marley crate and one small Zed hunk.
- **Recall (§18.3):**
  - #747's `panel_tabs::move_split` catches `SplitLeft/Right/Up/Down` at the workspace root
    (`register_action_renderer`, `capture_action`) and turns a panel tab's clone into a move; it
    returns without stopping the action for any other tab, so a second listener beside it runs.
  - #699's `MarleyNewItemMenu` is the hook pattern for a pane menu: a `Global` of an `Arc<dyn Fn>`
    that `pane.rs` calls while building the menu.
  - #701/#782: `home_page::ensure` and `open_here` look for the first Home in a workspace;
    nothing assumes there is only one.
  - The brain (`rusty-cli brain ask`, consultation cce7c412e0924db886af5871533ac3e6) returned
    nothing on this seam.
- **Discovery:**
  - `crates/workspace/src/workspace.rs` `handle_pane_event`: `Event::Split` → `split_and_clone`
    (ClonePane), `split_pane` (EmptyPane), `split_and_move` (MovePane); `split_and_clone` returns
    nothing when the item cannot split.
  - `crates/terminal_view/src/terminal_view.rs` `clone_on_split`: `project.clone_terminal`, a new
    shell. `Item::can_split` defaults to false, so Home cannot split.
  - `crates/workspace/src/pane.rs`: `Pane::split` (MovePane on one tab emits EmptyPane the other
    way); the split button's menu uses ClonePane when the active item can split, MovePane when the
    pane has more tabs, and is disabled otherwise; the tab menu (`right_click_menu(ix)`) has no
    split entries and ends with the item's own actions. `index_for_item_id` is private, so the
    Marley side finds the index through `items()`.
  - `crates/marley_workbench/src/home_page.rs`: `MarleyHome::new(workspace, fs, window, cx)`, no
    `fs` field; `open_here`.
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.**
  - `pane.rs` (Zed hunk): `MarleyTabMenu(Arc<dyn Fn(ContextMenu, &Entity<Pane>,
    &WeakEntity<Workspace>, EntityId, &mut Window, &mut App) -> ContextMenu>)`, a `Global`, and
    `marley_tab_menu`, which returns the menu unchanged with no hook. The tab menu's builder calls
    it last, before `menu.context(menu_context)`, with the pane, `pane.workspace` (captured beside
    `pane` before the closure) and `item_id`. A `// Marley:` comment on each hunk.
  - `home_page.rs`: `MarleyHome::page(workspace, window, cx)`, building a Home from a workspace
    (its `app_state().fs`), used by `open_here` and the new code; `Item for MarleyHome` gains
    `can_split` (true) and `clone_on_split` (a new Home of the same workspace, or none when the
    workspace is gone).
  - A new module `pane_splits.rs` (`init` from `crate::init`): a `register_action_renderer` with
    capture listeners for the six split actions, each calling `split_to_home(workspace, direction,
    mode, window, cx)`, which acts only under D1's conditions; and `cx.set_global(MarleyTabMenu(..))`
    whose function adds the entries under D3's conditions. `split_out(workspace, pane, item_id,
    direction, window, cx)` does D4.
  - `SplitHorizontal`/`SplitVertical` take their direction from `SplitDirection::horizontal(cx)` /
    `vertical(cx)`, as `pane.rs` does.
  - §14: no IO, no panics (index through `position`), no spawn.
- **File manifest:**
  - `crates/workspace/src/pane.rs` — Zed crate (the hook).
  - `docs/marley/zed-touchpoints.md` — its row extended.
  - `crates/marley_workbench/src/home_page.rs` — Marley crate.
  - `crates/marley_workbench/src/pane_splits.rs` — Marley crate, new.
  - `crates/marley_workbench/src/marley_workbench.rs` — Marley crate (`mod`, `init`).
  - `script/e2e/785-a-split-opens-home.sh` — the scenario (Test phase).

### Visual check plan
`script/e2e/785-a-split-opens-home.sh`, `compositor sway`. Setup: a scratch repository with a
README, opened; its first terminal in front with `PS1='$ '`.

| REQ | What the scenario does | Shot or check |
|---|---|---|
| 001 | click the pane's split button, then Split Right | `785-01-split-right`: the terminal on the left, Home on the right; MCP `terminals` lists one |
| 002 | click the terminal, palette `pane: split down` | `785-02-split-down`: Home below the terminal |
| 003 | click a Home pane, its split button, Split Right | `785-03-split-home`: two Homes side by side |
| 004 | open README.md beside the terminal in one pane, right-click its tab, End, Enter (Split Down) | `785-04-tab-menu`: README in a new pane below, the terminal where it was |
| 005 | right-click the README's tab, now alone in its pane, End, Up, Up, Up, Enter (Split Right) | `785-05-split-only-tab`: README on the right, Home on the left |
| 006 | — | review: every Marley path checks `marley_layout` |
| 007 | — | review: `split_to_home` acts only on a `TerminalView` |

### Risks
- The split button's menu dispatches its action to the element that held the focus before the
  menu opened; if that is not the pane the button sits in, the split goes to the focused pane, as
  in Zed. The capture reads `active_pane`, which follows the focus.
- A pane's focus check: the prompt editor inside a terminal holds the focus at a prompt; it sits
  inside the pane's element, so `contains_focused` on the pane is true.
- Home in a project group lists recent projects and agents like Default's; its Terminal card opens
  in the workspace's active pane, the split one, since a click there moves the focus to it.
- `split_pane` with a pane not in the center would fail; the menu entries are offered only for
  center panes, and the capture acts only on the center's active pane.

## Phase 2 — Code (2026-10-15)
- **Built:** in `pane.rs`, `MarleyTabMenuFn`, `MarleyTabMenu` and `marley_tab_menu`, called last
  in the tab menu's builder with the pane, `pane.workspace` and the item id; and the split
  button's menu given the pane's focus as its context (D6). `home_page.rs`: `new_page`, used by
  `ensure`, `open_here` and the new module; a `fs` field; `can_split` and `clone_on_split`.
  `pane_splits.rs`, new: the six capture listeners (`split_to_home`), the tab menu's entries
  (`tab_menu`) and the move (`split_out`), with `add_home`. `marley_workbench.rs`: `pub mod
  pane_splits`, `pane_splits::init`, and `init`'s workspace observer moved to
  `observe_workspaces` to keep `init` under clippy's line limit
  (L-claude-782-marley-workbench-init-sits-at-clippys-100-lines-001).
- **Deviations from the plan,** each found by running the scenario's steps against the debug
  build before the gate:
  - `clone_on_split` first read the workspace for its `fs`, and Zed calls it while the workspace
    is being updated (`split_and_clone` runs in `handle_pane_event`): Marley panicked, "cannot
    read workspace::Workspace while it is already being updated". Home now keeps the `fs` it was
    made with.
  - The split button's Split Right did nothing on Home or on a file, in the installed build too
    (probe with `E2E_BINARY` set to it, README open): with debug logs, the action reached the
    workspace root's capture listeners, with the item's pane holding the focus, but the pane never
    emitted `Event::Split`. The menu had no context, unlike the tab menu, so its choice was
    dispatched from the menu's own focus; giving it the pane's focus handle (`.context`) puts the
    dispatch on the pane, and the split happens. A terminal had worked only because D1's listener
    sits at the root. The exact node gpui fell back to was not traced.
  - D1 finds the pane by the focus (`Workspace::panes`, the one that `contains_focused`) instead
    of `active_pane`: the menu focuses its own pane before the workspace's active pane follows.
  - `MarleyTabMenuFn` names the hook's type: rustfmt left the inline `Arc<dyn Fn(..)>` unformatted.
- **Review of the diff:** against REQ-001 to REQ-007. No entity is read while it is updated:
  `split_to_home` runs in the workspace's own listener and reads panes; `tab_menu` reads the
  workspace in the menu's builder, outside any update; `split_out` updates the workspace, then
  each pane. Indexing through `position`; no `unwrap`. Every Marley path checks
  `marley_layout`, and the tab menu's entries go only on center panes. Nothing from Warp or VS
  Code's code; Zed's hunks are additive, with `// Marley:` comments and the ledger row.
- **Gate:** run 1 red on gate:2 only: clippy's `use_self` (`MarleyHome::new` in
  `clone_on_split`), `too_long_first_doc_paragraph` (the new module's doc) and
  `needless_pass_by_ref_mut` (`new_page`'s window, `observe_workspaces`' cx); every other row
  green. Fixed at the source. Run 2: `GATE GREEN [diff]`, all 17 rows PASS, receipt written, with
  the scenario in the tree.

## Phase 3 — Test (2026-10-15)
- **Build:** `just build` at `nice 19`; the last build, with D6, 2 m 29 s.
- **Scenario:** `just e2e script/e2e/785-a-split-opens-home.sh`, a headless sway, `SHOT_DIR` in
  the scratchpad; exit 0, both checks pass. Focus report: Hyprland 1 Marley window before and 1
  after, no rule added; sway stopped with the run's Marley.
- **Shots, each read:**
  - `785-01-split-right` (REQ-001): the terminal on the left, Home on the right; the rail lists
    Home and `repo — bash`; `terminals` lists one terminal.
  - `785-02-split-down` (REQ-002): `pane: split down` from the palette puts Home under the
    terminal; still one terminal.
  - `785-03-split-home` (REQ-003): the split button's Split Right on the right-hand Home gives
    two Homes side by side; the rail lists three.
  - `785-04-menu`, `785-04-tab-menu` (REQ-004): README's tab menu ends with Split Right, Split
    Left, Split Up and Split Down, after its own Open Markdown Preview; Split Down moves README
    into a new pane under the terminal, and the terminal stays where it was.
  - `785-05-menu`, `785-05-split-only-tab` (REQ-005): README alone in its pane, Split Right: README
    on the right of that row and Home on the left; the rail lists a fourth Home.
  - `785-06-file-split` (REQ-007): the split button's Split Right on README shows README in two
    panes; the rail's Files row counts 2.
- REQ-006 is the review's: `split_to_home` and `tab_menu` return at once in the Zed layout.
- The probes that found the deviations above are in the Code entry; their shots were deleted.
- **Fixes:** the three in the Code entry, each run again through the scenario. Gate run 1's lint
  fixes came after the last scenario run: `Self::new`, two `&mut` made `&`, and a doc comment, none
  of which changes what runs, so the run stands.

## Phase 4 — Complete (2026-10-15)
- **Docs (§21):** `CHANGELOG.md` Added ("A split opens Home, and a tab's menu splits it out");
  `docs/marley_architecture/marley_workbench.md`, a section "A split opens Home"; the shell plan's
  status paragraph in `docs/marley/workbench-shell.md`; `docs/marley/guide.md`, a "Splits"
  section after "Where terminals open"; `docs/marley/zed-touchpoints.md`'s `pane.rs` row carries
  both hunks (gate:16 green).
- **Ledger (§19):** `F-claude-785-the-split-buttons-menu-reached-no-pane-001`,
  `F-claude-785-home-read-the-workspace-inside-its-update-001`,
  `PR-claude-785-a-menu-that-dispatches-a-pane-action-names-the-pane-as-its-context-001`,
  `PR-claude-785-an-items-split-hooks-never-read-the-workspace-001`,
  `L-claude-785-run-a-probe-against-the-installed-build-to-tell-old-from-new-001`.
- **Brain:** consultation cce7c412e0924db886af5871533ac3e6 closed with `brain decide`:
  `decisions/in-marleys-layout-a-terminals-split-opens-home-and-every-center-tabs-menu-moves-the-tab-to-a-new-pane`,
  follow-up by 2026-10-29.
- **Ticket:** closed, moved to `tickets/closed/`; its BACKLOG row left at promotion.
- **Archive and commit:** the pair moved to `pipeline/completed/`; one commit carries the change,
  the scenario, the docs and this record.
