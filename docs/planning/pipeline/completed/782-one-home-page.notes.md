# One Home page, with Rusty's cards while Rusty is on, and Home first in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-782-one-home-page.md
- **Pipeline spec:** 782-one-home-page.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-13: fold Rusty's home page into Home, Rusty's items only when Rusty is
  configured; "in default the Home that always stays should always be at the top."
- **Classification / tier:** feature, Marley crates only (`marley_workbench`, `marley_rail`); no Zed
  crate, so no touchpoint row.
- **Checklist (no TaskCreate in this harness):** pick the item ✓, pre-flight ✓ (gate, e2e, shear,
  hooks wired, no active pipeline, README marker present, cargo idle), recall ✓, mint the pair ✓
  (no queued pair, no BACKLOG row to remove), prior-art sweep ✓, spec ✓, design ✓, present ✓.
- **Reading "at the top":** the rail lists the groups' tabs under each group, and `marley_rail::walk`
  puts other tabs after every terminal, worktree and Browser tab, so Home sat under Default's
  terminals. The tab strip already has Home first (`ensure` inserts at index 0). "Top" is the rail.
- **Recall (§18.3):**
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001: Home's handlers
    stay plain closures over the weak workspace. `RustyCards`' listeners lease `RustyCards`, never
    the item; check no workspace operation reads into it.
  - F-claude-703-homes-page-hid-a-tab-just-opened-into-home-001: `ensure` re-activates the tab that
    was active before the insert. Untouched here; keep it when moving the fill observers.
  - AD-claude-679 (one Rusty button and a home page), AD-claude-743 (no Rusty group; the + menu's
    quick links moved to Rusty's home page), #778 (the status bar's Rusty button back beside Home,
    2026-10-12). This ticket supersedes #778's status-bar half; Complete writes the AD.
  - The brain: `rusty-cli brain ask` returned only follow-ups due, nothing on this seam.
- **Discovery:**
  - `home_page.rs`: `MarleyHome`, its cards, `ensure`, `open_here`, `fill_emptied`; imports `card`,
    `row` and `muted` from `rusty::home_tab`.
  - `rusty/home_tab.rs`: `RustyHome` (an `Item` titled Rusty), `OpenHome`, `open_later`,
    `open_here`, `load`, the card helpers, and `init`, which also holds Home's fill observers.
  - `rusty.rs:86`: `open_home_here` re-export; `is_on`, `is_connected`, `unavailable`;
    `hide_namespace("rusty")` while off.
  - `status_buttons.rs`: `HomeButton` draws Home and, while Rusty is on and connected, Rusty.
  - `commands.rs:799`: `Verb::Rusty(None)` opens Rusty's home after `rusty_unavailable`.
  - `rail.rs:9893` `member_tabs` sets `TabKind`; `rail.rs:6656` `render_tab_row` matches it.
  - `marley_rail.rs`: `TabKind` (`File`, `Other`), `walk` at 1321, `selection` at 1617, the focus at
    927.

### Design
- **Rusty's cards (`rusty/home_tab.rs`).** `RustyHome` becomes `RustyCards`: no `Item`, `Focusable`
  only for its own node, the same `read`, `reading`, `again` and subscriptions, and the same `load`.
  Its `render` returns the RUSTY card, RECENT PAGES | FOLLOW-UPS DUE and TASKS while connected, and
  one RUSTY card with the reason while not (the `unavailable` text, or Home's own wording for
  `Starting`). The forms keep dispatching from `RustyCards`' own focus handle, which sits under
  Home's node, so the workspace's handlers still hear them. `OpenHome` stays, registered as now;
  `open_later` runs `home_page::open_here` in the shown group with no `capture::ready` toast. A
  `pub(crate) fn cards(window, cx) -> Entity<RustyCards>` makes the view for Home.
- **Home (`home_page.rs`).** `MarleyHome` gains `rusty: Option<Entity<RustyCards>>`, set when
  `rusty::is_on` turns true and cleared when it turns false, checked on the `SettingsStore` and
  `Rusty` globals (Home already observes `SettingsStore`). `render` puts the child after RECENT
  PROJECTS | AGENTS AT WORK, then AGENT ACTIVITY, then CONFIGURE. `card`, `row` and `muted` move
  here and `rusty/home_tab.rs` imports them. A new `home_page::init` holds the two fill observers
  moved from `rusty::home_tab::init`; `marley_workbench::init` calls it.
- **Status bar (`status_buttons.rs`).** `HomeButton` draws Home alone; its observers of
  `SettingsStore` and `Rusty` go, and the module and struct doc comments drop Rusty's button.
- **The verb (`commands.rs`).** `Verb::Rusty(None)` keeps `rusty_unavailable`, then calls
  `crate::home_page::open_here`. `rusty::open_home_here` goes.
- **The rail.** `marley_rail.rs`: `TabKind::Home` ("Home's page, listed first under its group,
  #782"); `walk` builds `homes` from `tabs` of that kind and chains it before `terminals`;
  `selection` maps `TabKind::Home` like `Other`; the doc comment on `walk` names Home first.
  `rail.rs`: `member_tabs` checks `item.downcast::<crate::home_page::MarleyHome>()` before the
  file check; `render_tab_row`'s two matches take `Home` with `Other`. Any test with an exhaustive
  match on `TabKind` takes the arm.
- **Errors and panics:** no new fallible path. `RustyCards` keeps `log_err` on its updates and its
  `Read::Failed` text for a refused read.
- **File manifest** (all Marley crates):
  - `crates/marley_workbench/src/home_page.rs`: holds `RustyCards`, the card order, the helpers,
    `init`.
  - `crates/marley_workbench/src/rusty/home_tab.rs`: `RustyCards`, the not-connected card,
    `OpenHome` to Home, `init` slimmed.
  - `crates/marley_workbench/src/rusty.rs`: drop the `open_home_here` re-export.
  - `crates/marley_workbench/src/status_buttons.rs`: Home alone.
  - `crates/marley_workbench/src/commands.rs`: the `rusty` verb opens Home.
  - `crates/marley_workbench/src/marley_workbench.rs`: call `home_page::init`.
  - `crates/marley_workbench/src/rail.rs`: mark Home's tab; draw it as `Other`.
  - `crates/marley_rail/src/marley_rail.rs`: `TabKind::Home` and its order.
  - `script/e2e/782-one-home-page.sh`: the scenario (Test).
- **Ledger rows:** none for Zed. Complete: an AD superseding #778's status-bar half and the two-page
  split of AD-679 and AD-743.

### Visual check plan
Under `compositor sway`, no path (Default shown), Rusty on through the stand-in, seeded as
`script/e2e/679-the-rusty-home-page.sh` seeds it: a page under `notes`, a decision due yesterday,
`tasks.json` with two made-up lists.

| REQ | Set-up and action | Shot |
|---|---|---|
| REQ-001, REQ-005 | Start; Home shows in Default | `782-01-home-rusty`: Marley's cards, then RUSTY with the screens and captures, RECENT PAGES, FOLLOW-UPS DUE listing the decision; the status bar's right end with Home only |
| REQ-001, REQ-004 | Scroll Home to its end | `782-02-home-bottom`: TASKS with the seeded tasks, AGENT ACTIVITY, CONFIGURE last |
| REQ-008 | Click Home's New Terminal; the terminal opens in Default | `782-03-rail-home-first`: the rail under Default lists Home, then the terminal |
| REQ-006 | Terminal tab in front; palette `rusty: open home` | `782-04-open-home`: Home in front |
| REQ-007 | Terminal tab in front; a second launch of the run's binary with `--user-data-dir $E2E_PROFILE zed://marley/rusty` (#762's hand-off) | `782-05-verb-home`: Home in front |
| REQ-002 | Swap `$bin/rusty-mcp` for a script that exits 1, then stop the stand-in by its state path | `782-06-not-connected`: one RUSTY card with the reason, none of Rusty's lists |
| REQ-003 | `marley.rusty.enabled` false in the run's settings | `782-07-rusty-off`: Home with Marley's cards only |

Out of reach: the cards re-reading on a change Rusty announces is #679's behavior, unchanged here.
If the second launch cannot hand over in sway, REQ-007 falls back to the diff's review.

### Risks
- **Reconnect race (REQ-002).** Marley may restart `rusty-mcp` after the stand-in stops; the swapped
  failing binary keeps it down. If Marley holds `Starting`, the card shows the connecting text,
  which also meets REQ-002's "not connected and why".
- **Reads in the background.** Each window's Home now re-reads Rusty on every announced change,
  where Rusty's page did only while open. Local and cheap; noted under Out.
- **Lease panics.** A `RustyCards` listener that ends in a workspace update which reads every item
  is safe while `MarleyHome` does not read `RustyCards` in `is_dirty`, `tab_content_text` or
  similar. Keep it that way.
- **Reverses #778's status-bar half from yesterday.** Chad asked for this one today; the AD at
  Complete says so.

## Phase 2 — Code
- **Checklist:** `rusty/home_tab.rs` ✓, `home_page.rs` ✓, `rusty.rs` ✓, `status_buttons.rs` ✓,
  `commands.rs` ✓, `marley_workbench.rs` ✓, `rail.rs` ✓, `marley_rail.rs` ✓, the review ✓, the
  gate (below).
- **Built:**
  - `RustyHome` is now `RustyCards`, a plain view with no `Item`, `Focusable` or `EventEmitter`. It
    takes Home's focus handle, so the forms dispatch from Home's node. Its render is the RUSTY card,
    RECENT PAGES | FOLLOW-UPS DUE and TASKS, or one RUSTY card (`marley-rusty-home-not-connected`)
    with `not_connected`'s reason: "Rusty is still connecting." while starting, else
    `rusty::unavailable`.
  - `OpenHome` runs `home_page::open_here` in the shown group; `open_here`, `home_of` and the
    `capture::ready` check went from the module.
  - `MarleyHome` holds `rusty: Option<Entity<RustyCards>>`, which `follow_rusty` makes or drops on
    the `Rusty` global, and renders it after RECENT PROJECTS | AGENTS AT WORK. CONFIGURE moved under
    AGENT ACTIVITY. `card`, `row` and `muted` moved to `home_page.rs`, and the fill observers to
    `home_page::init`, called just before `rusty::init`. `fill_emptied` is private now.
  - `HomeButton` draws Home alone. It lost its constructor and its two observers; the struct is
    made in place.
  - `Verb::Rusty(None)` opens Home after `rusty_unavailable`; the help line reads "the Home page,
    or one of Rusty's screens".
  - `TabKind::Home`; `walk` chains `homes` before `terminals`; `selection` maps it as `Other`;
    `member_tabs` marks `MarleyHome`; `render_tab_row` draws it as `Other`.
- **Deviations:**
  - `agent_activity.rs`, `mcp_servers.rs` and `threads_page.rs` imported `muted` from
    `rusty::home_tab`; each now imports it from `home_page` (one line each, not in the manifest).
  - The first gate run was red on gate:2 only. `home_page::init(cx)` took `marley_workbench::init`
    to 101 lines (`too_many_lines`), so `LayoutState`'s construction left `init` for
    `LayoutState::read`, the same reads in the same order. Calling Home's init from `groups::init`
    or `status_buttons::init` was rejected: neither owns Home's page, and either would move the
    fill observers' registration order. The cards module's first doc paragraph was too long
    (`too_long_first_doc_paragraph`) and is split.
- **Review of the diff:**
  - Found: Home now keeps Rusty's cards for the whole session where Rusty's page lived only while
    open, and the cards re-read on reconnection only after a failed read. After a `rusty-mcp`
    restart they would have shown the lists from before it. Fixed: `RustyCards` keeps `connected`
    and re-reads whenever Rusty connects again. Not a bug in shipped code, so no `F-` block.
  - Re-entrancy: the screens' `cx.listener` leases `RustyCards`, which is not an item, so no
    workspace read of every item reaches it (PR-claude-701 holds). Home's handlers stay plain
    closures. `follow_rusty` runs from an observer of Home itself, never inside another update of
    Home.
  - REQ-001 to REQ-008 are each met by code in the diff; REQ-005: no Rusty button is drawn anywhere
    in the status bar now.
  - Provenance: nothing from Warp; no Zed crate touched.
- **Gate:** `script/gates.sh --diff` green on the second run, 17 passed, 0 failed (receipt written).
  The first run failed gate:2 only, fixed as above.
## Phase 3 — Test
- **Checklist:** the scenario ✓, the build ✓, a first run to read the positions ✓, two full runs ✓,
  every shot read ✓, the gate again for the scenario (below).
- **Scenario:** `script/e2e/782-one-home-page.sh`, `compositor sway`, a fresh profile (`db`
  removed) on `repo`, Rusty on through the stand-in, seeded as #679's scenario seeds it, the debug
  binary first on the terminals' PATH as `marley`. `STOP_AT=1` ends it after the first shot.
- **Shots** (second full run; the first run's 01 to 05 and 07 read the same):
  - `782-01-home-rusty`: Default's Home: START, NEW AGENT, RECENT PROJECTS | AGENTS AT WORK, then
    RUSTY with the eight screens and Open Page… and the three captures, RECENT PAGES ("No page
    opened yet.") | FOLLOW-UPS DUE ("Ship the home page", 2026-10-12, in red), TASKS with the four
    open seeded tasks (the completed one left out), AGENT ACTIVITY, CONFIGURE at the bottom edge.
    The status bar's right end, cropped 4×: Home's icon, then the dock's panel buttons; no Rusty
    icon. REQ-001, REQ-004, REQ-005 met.
  - `782-02-home-bottom`: scrolled: TASKS, AGENT ACTIVITY, then CONFIGURE last with its six
    buttons. REQ-001, REQ-004 met.
  - `782-03-rail-home-first`: after Home's New Terminal the rail lists, under Default, Home, then
    `cpeppers — bash`, selected; `repo` and its terminal below. REQ-008 met. Before #782 the
    terminal would have come first.
  - `782-04-open-home`: from the terminal's tab in front, `rusty: open home`: Home's tab in front
    and its row selected in the rail. REQ-006 met.
  - `782-05-verb-home`: `marley rusty` typed in Default's terminal (its row reads "marley rusty ·
    done · 0 s"): Home in front. REQ-007 met through the CLI, the verb's user-facing path.
  - `782-06-not-connected`: one RUSTY card, "Rusty is not connected: rusty-mcp did not answer
    within 5 s", and no RECENT PAGES, FOLLOW-UPS DUE or TASKS. REQ-002 met.
  - `782-07-rusty-off`: `marley.rusty.enabled` false: Home with Marley's cards only; the title
    bar's Rusty button gone too. REQ-003 met.
- **Red, then fixed in the scenario:** the first run's `782-06` still showed every Rusty card. The
  stand-in had stopped (Zed's stdio transport logged a broken pipe), but Marley pings Rusty every
  5 s and gives a ping 5 s to answer, so a stopped server shows as down up to 10 s later; the
  scenario waited 6 s. It waits 15 s now, with a comment saying why. No source change.
- **Focus report:** the scenario ran in its own headless sway; Hyprland had 1 Marley window
  before and after, no rule added, no reload.
- **In-app guide:** `crates/marley_workbench/guide/index.html` named Rusty's home page and its
  status-bar button, so it changed here, inside the gate's hashed set: Home's article (#782, the
  RUSTY row, CONFIGURE last, Home first under Default), the Rusty group's article rewritten as
  "Rusty on Home" (its anchor kept), and every "Rusty home page" now Home's RUSTY card.
- **Gate:** the run with the scenario passed every gate but the receipt, which failed because the
  in-app guide changed during the run. Run again on the settled tree: green, 17 passed, 0 failed, receipt written.
- **Not reached:** the cards re-reading after a reconnection (the Code phase's fix) needs a
  stand-in that comes back; the scenario's failing binary keeps Rusty down. The diff's review
  covers it.

## Phase 4 — Complete
- **Checklist:** document ✓, capture knowledge ✓, close the ticket ✓, archive ✓, commit (below).
- **Documented:** `CHANGELOG.md` (Changed, #782); `docs/marley_architecture/marley_workbench.md`
  (Home's page: the helpers, `home_page::init`, the cards; the Rusty home page section now
  `RustyCards`); `docs/marley_architecture/marley_rail.md` (Home's row first);
  `docs/marley/workbench-shell.md` (#782 in the shell's shipped list); the in-app guide, in Test.
  No Zed path touched, so no touchpoint row. The user guide, walkthrough and tutorial outline
  follow in their own commit, as #778's did.
- **Knowledge appended:** AD-claude-782-one-home-page-rustys-cards-follow-the-switch-and-home-heads-default-001;
  L-claude-782-a-stopped-rusty-shows-as-down-up-to-10-s-later-001;
  L-claude-782-marley-workbench-init-sits-at-clippys-100-lines-001;
  L-claude-782-the-in-app-guide-is-inside-the-gates-receipt-001. No F- block: the one defect found
  (the cards not re-reading after a reconnection) was in this ticket's own unshipped code, and the
  Test red was the scenario's timing.
- **Brain:** consultation 87a8dff2bbb1475e8a3a728db0cfbf16 closed with `brain decide`:
  `decisions/marley-has-one-home-page-rustys-cards-follow-the-switch-home-heads-default-in-the-rail`,
  follow-up by 2026-10-27.
