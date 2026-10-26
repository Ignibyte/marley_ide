# tmux sessions as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-930-tmux-sessions-as-the-board.md
- **Pipeline spec:** 930-tmux-sessions-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (TmuxSessions).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`); no Zed hunk, so
  no `docs/marley/zed-touchpoints.md` row.
- **Recall (§18.3):**
  - L-claude-808-a-scenarios-tmux-socket-must-be-short-001: the run's tmux server under
    `$XDG_RUNTIME_DIR`, started with `-f /dev/null`, killed and removed in `teardown`.
  - PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001 and
    F-claude-880-a-scenario-opened-a-live-tmux-session-001: `script/e2e.sh` already points tmux at
    the run's own server; the scenario sets `TMUX_TMPDIR` for its own server too, as #832 does.
  - L-claude-831-a-clicking-scenario-sets-tmux-hosts-to-nothing-001: a clicking scenario fixes
    `marley.tmux_hosts`, here to `["local"]` on the run's own server, so the points hold.
  - L-claude-832-a-right-click-menu-opens-at-the-pointer-so-click-its-entries-inside-001: not
    needed (this scenario clicks rows and buttons, no menus); points come from a probe run.
  - L-claude-808-zed-holds-a-second-run-of-a-running-task-001: Show must activate the running
    attach, never schedule a second one; `attached_tab` finds it as `open_session` does.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the page reads its window's
    workspaces and their terminal views in its render, never itself.
  - F-claude-867-opening-a-project-from-a-page-read-the-page-001: Show activates a workspace from
    a page's click; it runs in the click handler, outside the page's update, and touches no
    project list.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: no agent starts here; the stand-in ssh
    reaches Marley through `MARLEY_SSH`.
- **Discovery:**
  - `rail_sessions.rs`: `render_sessions` (the not-looked-on rows, the empty line, the checkbox
    block removed), `session_row` (the fade), `host_heading` (a grey-dot variant with no count),
    `host_choices` (removed from the rail; its checkbox code moves to the page).
  - `rail.rs`: `RowIcon` and `row_card` (the fade on the tile's slot), `tmux_attached_here` and
    the lookup in `open_session` (both call the new helpers).
  - `tmux_sessions.rs`: `SessionKey`, `AttachedTab`, `attached_here`, `attached_tab`.
  - `sessions_page.rs`: `Row` (`SessionState`), `render_header` (summary), a new header row,
    `render_row` (chip, Show or Attach), `render_hosts` (checkboxes).
  - `focus_row.rs`: `render_terminal` (the chip and its click).
- **Decisions:** D1–D6 in the spec. D2 drops the rail's checkboxes because the board draws the
  ticks on the page only; D5 picks the page over the rail's view for the bottom row's click.

### Design
- `tmux_sessions.rs` (Marley crate): `SessionKey { host: SharedString, session: String }`
  (`Hash`, `Eq`), `AttachedTab { workspace: WeakEntity<Workspace>, view: Entity<TerminalView> }`,
  `attached_here(workspaces: &[Entity<Workspace>], cx) -> HashSet<SessionKey>` and
  `attached_tab(workspaces, host, session, cx) -> Option<AttachedTab>`, both over
  `items_of_type::<TerminalView>` with the running-task test the rail makes now.
- `rail.rs` (Marley crate): `enum TileFade { Full, Faded }` on `RowIcon` (default `Full`);
  `row_card` sets `.opacity(0.55)` on the slot when faded. `tmux_attached_here` and
  `open_session` call the helpers with the snapshot's workspaces.
- `rail_sessions.rs` (Marley crate): the session row passes `TileFade::Faded` unless the key is
  in `attached_here`; after the looked-on hosts, one block per not-listed `HostChoice`: grey dot
  (`text_muted`), its label, and the line with the link as a `Button` in link style.
- `sessions_page.rs` (Marley crate): `enum SessionState { AttachedHere, AttachedElsewhere,
  Detached }` with its words and chip colours in one method; `rows` takes the set from
  `attached_here(window's workspaces)` (`window.root::<MultiWorkspace>()`); a header row with
  the same widths as `render_row`; Show runs `MultiWorkspace::activate` and
  `Workspace::activate_item` on the `attached_tab`; `render_hosts` draws `Checkbox` rows calling
  `set_listed`.
- `focus_row.rs` (Marley crate): the count also counts detached sessions; the chip on
  `RowKind::Tmux`'s tint (`status().hint_background`) with an `on_click` that dispatches
  `OpenSessionsPage`.
- Ledger rows: none (no Zed path).

### Visual check plan
- REQ-001: the run's server has `build`, `loose` and `deploy` (an outside client on `deploy`);
  the rail's Sessions view is opened and `build` clicked so it attaches. `930-01-rail` shows
  `build` at full strength and the other two tiles faded; `930-06-day` again in Marley Day.
- REQ-002: `forge` is saved and not looked on. `930-01-rail` shows its grey-dot row with the line
  and link; the link clicked, `930-04-host-added` shows the setting printed and `forge` heading
  with its ssh failure.
- REQ-003, REQ-004, REQ-006: the bottom row's chip clicked opens the page; `930-02-page` shows the
  headers, the three chips and "3 on 1 host · 1 attached here · 1 attached elsewhere · 1
  detached".
- REQ-005: `930-02-page` shows Show on `build`, Attach on `loose` and `deploy`; Show clicked,
  `930-03-show` shows the `build` tab in front.
- REQ-007: `930-02-page` shows the checkboxes (`This machine` looking, `forge` not listed); after
  REQ-002's add, `forge` unticked on the page, `930-05-unticked` shows the printed setting and
  `forge` back as a not-looked-on row in the rail.
- REQ-008: with the `build` terminal focused, `930-01-rail` shows "tmux: 3 · 1 detached" in the
  bottom row.
- REQ-009: the chip clicked; `930-02-page` is the page it opened.
- What no scenario reaches: a real remote host's sessions. The not-looked-on row and the add need
  only a saved host, which the refused stand-in gives.

### Risks
- An outside tmux client in the background: a control-mode `tmux -C attach` exits when its input
  closes, so it reads from a FIFO held open by a background `sleep`; `teardown` kills both before
  the server. Check `#{session_attached}` is 1 for `deploy` in the setup's log.
- Dropping the rail's checkboxes takes away unticking from the rail. The board wants it; if Chad
  misses it, the host heading's right-click can offer it in a follow-up.
- Reading the window's `MultiWorkspace` from the page's render: the page is drawn as its own view,
  outside the workspace's update, as the rail reads it today; any panic shows in the first run.
- The bottom-row ticket (build order item 2) also changes `render_terminal`; whichever lands
  second keeps this chip.

## Phase 2 — Code
- **Built:** `tmux_sessions::{SessionKey, AttachedTab, attached_here, attached_tab}`;
  `rail::TileFade`, `RowIcon::with_fade`, `Rail::window_workspaces`; `rail_sessions::not_looked_on`
  in place of the checkbox block; `sessions_page` `SessionState`, `render_columns`, `state_chip`,
  `show_or_attach`, the summary and the host checkboxes; `focus_row::tmux_chip` opening the page
  (and `show_tmux_sessions` gone with the rail parameter it needed).
- **Deviations:** the page's Show reveals the terminal through `browser::reveal_terminal`, which
  shows its workspace first. "Attached here" now also requires the attach's task to be running in
  the rail's open (it was required only for the placed rows).
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- Scenario `script/e2e/930-tmux-sessions-as-the-board.sh` (sway), green.
- `930-01-rail`: build "attached here" at full strength, deploy "attached elsewhere" and loose
  "detached" faded; forge "Not looked on yet. Add to tmux hosts"; no checkbox block; the bottom row
  "tmux: 3 · 2 detached" before the attach, "· 1 detached" after. REQ-001, REQ-002, REQ-008.
- `930-02-page`: the chip clicked: the page, its column headers, the three chips, Show on build and
  Attach on the others, "3 on 1 host · 1 attached here · 1 attached elsewhere · 1 detached", the
  host checkboxes with "looking" and "not listed" and `marley.tmux_hosts`. REQ-003 to REQ-007,
  REQ-009.
- `930-03-show`: Show: build's tab in front. REQ-005.
- `930-04-host-added`: Add to tmux hosts: `['local', 'forge']`, forge's heading with ssh's refusal.
  REQ-002.
- `930-05-unticked`, `930-06-day`: forge unticked on the page: `['local']`, "not listed", and
  forge back as a not-looked-on row in the rail; Marley Day. REQ-007.

## Phase 4 — Complete
- Docs: CHANGELOG; `marley_workbench.md` (the Sessions page as the board draws it, the bottom
  row's chip); the look plan's W1.3. No Zed path.
