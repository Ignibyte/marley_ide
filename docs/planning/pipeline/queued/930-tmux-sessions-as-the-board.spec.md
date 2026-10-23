---
pipeline_id: 658ccf32-c7ff-493c-9153-febaf17582a5
ticket: docs/planning/tickets/open/TICKET-930-tmux-sessions-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: tmux sessions as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/B-rail-chrome.md]
---

## Title
The rail's Sessions view, the Sessions page and the bottom row's tmux count as the TmuxSessions
board draws them: faded tiles for sessions not attached here, hosts not looked on as rows with an
Add link, a table with headers and state chips, Show for a session attached here, the page's host
checkboxes, and a tmux count that says what is detached and opens the page.

## Scope
### In
- **Detached rows fade** (Marley crate). `rail_sessions.rs` `Rail::session_row`: a session not
  attached here (detached, or attached elsewhere) draws its tile at 0.55 opacity. `rail.rs`
  `RowIcon` gains a `fade` field of a new enum (`TileFade::Full` / `TileFade::Faded`), which
  `row_card` applies to the tile's slot; every other caller keeps `Full`.
- **A host not looked on is a row in the list** (Marley crate). `rail_sessions.rs`
  `Rail::render_sessions`: after the looked-on hosts, each host `tmux_sessions::host_choices`
  offers that is not looked on gets a heading with a grey dot and no count, and under it the line
  "Not looked on yet." with the link "Add to tmux hosts", which calls
  `tmux_sessions::set_listed(name, true)`. The checkbox block `host_choices` (HOSTS TO LOOK ON)
  leaves the rail; the ticks live on the page. The empty line "No host is looked on. Tick one
  below." changes to say no host can be looked on, and shows only when no host is offered either.
- **One answer to "attached here"** (Marley crate). `tmux_sessions.rs` gains
  `attached_here(workspaces, cx) -> HashSet<SessionKey>` and `attached_tab(workspaces, host,
  session, cx) -> Option<AttachedTab>`, where `SessionKey { host, session }` and
  `AttachedTab { workspace, view }` are named structs. They hold the test `Rail::tmux_attached_here`
  and `Rail::open_session` make now: a running terminal whose task is Marley's attach of the
  session. The rail calls them with its snapshot's workspaces, the page with its window's
  `MultiWorkspace::workspaces`.
- **Sessions page column headers** (Marley crate). `sessions_page.rs`: a header row over the
  table, "Session", "Host", "Windows", "Folder", "State", at the widths `render_row` uses. The
  State column widens to fit "attached elsewhere".
- **State chips** (Marley crate). `sessions_page.rs` `render_row`: the state is a rounded chip,
  "attached here" (the success colours), "attached elsewhere" (the warning colours) or
  "detached" (the element background, muted ink). `Row.attached: bool` becomes
  `Row.state: SessionState` (`AttachedHere`, `AttachedElsewhere`, `Detached`).
- **Show or Attach** (Marley crate). `render_row`: a session attached here offers "Show", which
  activates its workspace and its tab (`attached_tab`); any other session offers "Attach", as now.
- **Header summary** (Marley crate). `SessionsPage::render_header`: "N on H hosts · A attached
  here · E attached elsewhere · D detached", with the elsewhere part only when E is not zero, so
  the parts add up to N.
- **Host toggles on the page** (Marley crate). `SessionsPage::render_hosts`: the heading "HOSTS
  TO LOOK ON", each host a `ui::Checkbox` with its name over where it comes from and its state at
  the right, "looking" (success) or "not listed" (muted); a tick calls
  `tmux_sessions::set_listed`. The setting's name, `marley.tmux_hosts`, sits at the foot in the
  buffer font. Host errors stay under the list.
- **The bottom row's tmux count** (Marley crate). `focus_row.rs` `FocusRow::render_terminal`: the
  muted "tmux N" becomes a chip on the tmux tint, "tmux: N · D detached" (D counts sessions no
  client has attached; the part goes when D is zero), and a click dispatches
  `sessions_page::OpenSessionsPage`. This is the only bottom-row change here; the bottom-row
  ticket (the review's build order, item 2) builds the rest of the terminal's row around it.

### Out (explicitly deferred)
- The Sessions view's "Filter sessions" field: #832 Out.
- "under <project>" under the page's folder, and "· placed under" in its column header: #861 Out.
- "New Session…" as a dialog that asks the host and folder: new behaviour. The button keeps
  starting a session in the window's first folder and keeps its name without the ellipsis.
  A follow-up ticket.
- The `tx` glyph tile on session rows: #827 Out ("the icons stay").
- The page's three-column layout (hosts in a column at the right), the "How a session is placed"
  card and the red Close…: not named in the review's TmuxSessions gaps; the page keeps #861's
  layout.
- The bottom row's other parts for a terminal: the bottom-row ticket.

## Reference (§20)
N/A — Marley-specific: Marley's own tmux views (#808, #821, #832, #861) over tmux's
`list-sessions`. Upstream Zed has no tmux view, and the Warp maps describe no session list.

### Prior art
- The board: `TmuxSessions.dc.html` in the design record. Its `renderVals` fade a session's tile
  unless it is attached here (`fade: attached ? '1' : '0.55'`), colour the three states, and pick
  `primary: state === 'attached' ? 'Show' : 'Attach'`.
- `rail_sessions.rs:30-97` `render_sessions` (the hosts, their rows, the empty line at `:62`, the
  checkbox block at `:70`, Open as a Page); `:99-186` `session_row` (the three state words at
  `:110-116`, `row_card` at `:142`, the menu already says Show or Attach); `:188-227`
  `open_session`; `:231-258` `host_heading` (dot and count); `:261-305` `host_choices`
  (the rail's checkboxes, the pattern the page takes).
- `rail.rs:5156-5169` `tmux_attached_here`; `:3062-3090` `activate_workspace`,
  `activate_terminal`; `:11046-11057` `RowKind::tint` (`Tmux` is `hint_background`);
  `:11061-11082` `RowIcon`; `:11155` `row_card`, whose slot takes the kind's tint.
- `sessions_page.rs:50-59` `Row` (`attached: bool`); `:106-159` `render_header` (the summary at
  `:113-118`, New Session at `:130`); `:161-255` `render_row` (the bare columns, the state word at
  `:167`, Attach at `:230`); `:257-303` `render_hosts` (read-only "looked on" labels).
- `tmux_sessions.rs:41` the listing's `#{session_attached}`; `:48-57` `TmuxSession`; `:125-157`
  `Attach`, `TmuxHosts::attach_of`; `:686-720` `HostChoice`, `host_choices`; `:735-744`
  `set_listed`.
- `focus_row.rs:135-143` the session count; `:191-193` the "tmux N" label; `:369` `chip`.
- `workspace::MultiWorkspace::workspaces` (`crates/workspace/src/multi_workspace.rs:1320`) and
  `activate` (`:1343`): public, read only; no Zed hunk.
- `prompts.rs:129` `render_button` and `Role`; `ui::Checkbox` as the rail uses it.
- Found no crate that owns a session table or state chips; the look is Marley's.

## UI proof
Scenario `script/e2e/930-tmux-sessions-as-the-board.sh` (sway). Setup: the run's own tmux server
under `$XDG_RUNTIME_DIR` (L-808), with `build` in the scratch repository, `loose` in the run's
home, and `deploy` with a client attached from outside Marley (a control-mode `tmux -C attach`
held open in the background); `marley.tmux_hosts` is `["local"]`; `forge` is a saved ssh host
behind #832's refused stand-in ssh (`MARLEY_SSH`), not looked on.
- `930-01-rail`: the Sessions view after `build` was clicked and attached: `build` "attached
  here" at full strength, `loose` and `deploy` with faded tiles, `forge` with a grey dot and "Not
  looked on yet. Add to tmux hosts", no checkbox block; the bottom row reads "tmux: 3 · 1
  detached" (REQ-001, REQ-002, REQ-008).
- `930-02-page`: the tmux chip clicked: the Sessions page with its column headers, the three
  chips, Show on `build` and Attach on the others, "3 on 1 host · 1 attached here · 1 attached
  elsewhere · 1 detached", and the host checkboxes (REQ-003 to REQ-007, REQ-009).
- `930-03-show`: Show on `build`: its terminal tab in front (REQ-005).
- `930-04-host-added`: Add to tmux hosts on `forge`: `marley.tmux_hosts` printed, `forge`'s
  heading with its failure in the rail (REQ-002).
- `930-05-unticked`: `forge` unticked on the page: the setting printed, `forge` "not listed" on
  the page and back as a not-looked-on row in the rail (REQ-007).
- `930-06-day`: the page and the rail in Marley Day (REQ-001 to REQ-007).

## Locked-In Decisions
- D1 — A session's tile fades unless it is attached here, as the board's data does: "attached
  elsewhere" fades too. Only the tile fades, not the words.
- D2 — Every host `host_choices` offers and Marley does not look on gets a row in the rail's
  list. The rail's checkbox block goes: the board draws the ticks on the page only. A host is
  taken off on the page.
- D3 — "Attached here" means a running terminal of this window whose task is Marley's attach of
  the session, the test the rail makes now. The rail and the page share one function for it.
- D4 — The header summary counts attached elsewhere apart, and only when there is one, so the
  three parts always add up to the total.
- D5 — The bottom row's tmux chip opens the Sessions page, not the rail's view: the page opens
  whether the rail is open, shrunk or hidden. Its "detached" counts sessions no client has
  attached.
- D6 — New Session stays as it is (no dialog); a dialog is new behaviour, a follow-up ticket.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a session is not attached in this window, its row in the rail's Sessions view shall draw its tile faded. | `930-01-rail`, `930-06-day` |
| REQ-002 | WHERE a host can be looked on and is not, the Sessions view shall list it with a grey dot and "Not looked on yet. Add to tmux hosts", and the link shall add it to `marley.tmux_hosts`. | `930-01-rail`, `930-04-host-added` |
| REQ-003 | The Sessions page shall head its table with Session, Host, Windows, Folder and State. | `930-02-page`, `930-06-day` |
| REQ-004 | The Sessions page shall show each session's state as a chip: attached here, attached elsewhere or detached, each in its own colour. | `930-02-page`, `930-06-day` |
| REQ-005 | WHEN a session is attached here, its page row shall offer Show, which brings its tab forward; other rows shall offer Attach. | `930-02-page`, `930-03-show` |
| REQ-006 | The Sessions page's header shall count the sessions and hosts, then those attached here, those attached elsewhere when there are any, and those detached. | `930-02-page` |
| REQ-007 | The Sessions page shall list the hosts to look on as checkboxes with their state, and a tick or untick shall change `marley.tmux_hosts`. | `930-02-page`, `930-05-unticked` |
| REQ-008 | WHILE a terminal has the focus and a host has sessions, the bottom row shall show "tmux: N · D detached" as a chip. | `930-01-rail` |
| REQ-009 | WHEN the bottom row's tmux chip is clicked, the system shall open the Sessions page. | `930-02-page` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rail_sessions.rs`, `rail.rs` (`RowIcon`, `row_card`, the two tmux helpers moved
  out), `tmux_sessions.rs`, `sessions_page.rs`, `focus_row.rs`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
