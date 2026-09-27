# B1c: Browser tabs return after a relaunch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-494-browser-tabs-restored.md
- **Pipeline spec:** 494-browser-tabs-restored.spec.md

## Phase 1 — Plan (drafted 2026-09-25, split from #493)
- **Request:** the restore half of the queued #493, split so each ticket is one slice.
- **Recall (§18.3):** #403's rule, no URL in a layout codec; #493's pages keyed by target id;
  the harness has no relaunch step yet, and #491's scenario quits Marley through the palette
  (`zed: quit`: Ctrl+Q in a terminal goes to the shell).
- **Checklist (no TaskCreate in this harness):** mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #493 committed (33af3fafad); no other active pipeline; cargo idle.
- **Recall (§18.3):**
  - `PR-claude-persist-verify-trigger-not-just-codec-001` and its failure: a restore that only
    round-trips its codec can pass while nothing ever saves. Zed saves an item when it joins a
    workspace and on each event `should_serialize` accepts; the tab emits `UpdateTab` when its
    page's URL or title changes, and the e2e test is a real quit and relaunch.
  - #403's rule (`embedded-browser-model.md` Q4): no URL in a layout codec.
  - #493: pages are keyed by target id; the unit outlives Marley, so a relaunch finds its pages
    alive.
  - The brain (consultation `8f645b9609844a15b491df556c6eab17`): nothing on this seam.
- **Seams re-verified:** `workspace::SerializableItem` (`serialized_item_kind`, `cleanup`,
  `deserialize(project, workspace, workspace_id, item_id, window, cx)`, `serialize(&mut self,
  workspace, item_id, closing, cx)`, `should_serialize`); `register_serializable_item`;
  `delete_unloaded_items`; `db::static_connection!(…, [WorkspaceDb])` with `query!`, as
  `crates/onboarding` and `crates/image_viewer` use them. Zed serializes an item on
  `added_to_workspace` and on events, throttled, and runs `cleanup` with the ids of the items it
  loaded. `script/e2e.sh` launches Marley once; it has no quit or relaunch step, and the run's
  profile is a copy of Chad's database, so a relaunch with the same path restores the run's own
  workspace.
- **The probe:** Marley's Chromium starts with `about:blank` as its URL, so a fresh start has one
  page; without a URL headless Chromium opens `chrome://newtab/`; with `--no-startup-window` it
  opens none and still makes pages on request.

### Design
- **`marley_browser::service`:** `--no-startup-window` in place of `about:blank`.
- **The start (`browser.rs`):** `pages_at_start` only lists; the hub attaches each listed page
  with `listed: true`, and `PageOpened` carries it. `open_tab` gives a listed page to a tab that
  claims it, else to an adopting tab, else leaves it without a tab. An adopting tab (opened
  while the browser starts) waits for the start's attaches and, still without a page, opens a
  blank one. `marley: open browser` gives the pages without a tab their tabs first.
- **Saving:** a `db` domain, `BrowserTabsDb`, with the table `marley_browser_tabs`
  (`workspace_id`, `item_id`, `target_id`, `url`, `title`, keyed by workspace and item,
  deleted with its workspace). `BrowserView` is a `SerializableItem` of kind
  `MarleyBrowserTab`: `serialize` saves its page's id, committed URL and title (the saved ones
  while its page is not back yet); `should_serialize` takes `UpdateTab`; `cleanup` is
  `delete_unloaded_items`. Registered in `browser::init`.
- **Restoring:** `deserialize` reads the row and builds a tab that claims the saved page id at
  once, so the start's `PageOpened` for that page finds it, and shows the saved title and URL.
  Its restore task waits for the hub to show its pages and for the start's attaches; the page
  back means it is reattached, and otherwise it opens a new page at the saved URL and takes it.
- **The harness:** `launch_marley` (the first launch uses it too; the log appends) and
  `quit_marley` (the palette's `zed: quit`, then up to 30 seconds for the process to end).
- **Manifest:** `crates/marley_browser/src/service.rs`; `crates/marley_workbench/Cargo.toml`
  (`db`); `crates/marley_workbench/src/browser.rs`, `src/browser_tools.rs` (`new_page`'s URL);
  `script/e2e.sh`; `script/e2e/494-browser-restore.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | two tabs (page 1 with typed text in its field, page 2); the tabs' ids; `quit_marley`, `launch_marley` | `494-02-reattached` (the text still there); the same ids |
| REQ-002, 004 | `quit_marley`; stop the Chromium unit; `launch_marley` | `494-03-reopened` (both pages at their URLs, the field empty, no third tab); new ids |
| REQ-003 | sqlite3 on the run's profile database | the run log: `items` rows with kind and id, the tabs' rows with URL and title |
| (regression) | #493's scenario on the changed start | its shots |

### Risks
- A relaunch that asks to trust the folder again would take a key meant for the page; the run
  shows it.
- A navigation in the last moment before a quit may not be saved: Zed throttles item saves.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (and the probe), spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built.**
  - `marley_browser::service`: Chromium starts with `--no-startup-window` in place of
    `about:blank`, so a start Marley makes opens no page.
  - `browser.rs`, the start: `pages_at_start` lists the pages and opens none; each listed page is
    attached with `listed`, which `PageOpened` carries. `open_tab` gives a listed page to a tab
    that claims it, or to a tab opened while the browser started, and otherwise leaves it without
    a tab; pages opened since the start are placed as #493 places them.
  - A tab opened while the browser starts (`adopts`) waits, in a task it owns, for the hub to show
    its pages (through a failure and the start after it) and for the start's attaches, then opens
    a blank page when it got none.
  - The saving: `MarleyBrowserTabsDb`, a `db` domain after the workspace's, with the table
    `marley_browser_tabs`. `BrowserView` is a `SerializableItem` of kind `MarleyBrowserTab`:
    `serialize` saves the page's id, URL and title (the saved ones while the page is not back),
    on `UpdateTab`, which the tab emits when its page changes and when it takes a page; `cleanup`
    is Zed's `delete_unloaded_items`; `browser::init` registers the item.
  - The restoring: `deserialize` reads the row and builds a tab that claims the saved page id at
    once and shows the saved title and URL; its task waits for the hub to show its pages and for
    the start's attaches, keeps the page when it is back, and otherwise opens the saved URL in a
    new page (`open_page_in` takes the URL now, and `new_page` too).
  - `open` gives the pages without a tab their tabs first, and counts a restoring tab as the
    workspace's.
  - `script/e2e.sh`: `launch_marley` (the first launch too; the run's log appends) and
    `quit_marley` (the palette's `zed: quit`, then up to 30 seconds for the process to end).
- **Deviations from the design:** none in substance. The waits of an adopting or restoring tab
  outlast a failure (see the review), and belong to the tab, so a closed tab stops waiting.
- **Review of the diff.**
  - Found and fixed: with no blank page at the start, an adopting tab and a restoring tab gave up
    when the hub failed (`showing` answers a failure at once), so after `open browser` started the
    browser again the tab would have waited for ever with no page. They now wait on `shown`,
    which polls until the hub shows its pages, in a task the tab holds.
  - Re-entrancy: `deserialize` builds the tab inside the window's update and reads no workspace;
    `serialize` runs inside the workspace's update and reads only the hub; the tasks update the
    tab outside any other update, and `open_page_in` takes the tab's window instead of reading the
    tab.
  - The save is triggered, not only coded (`PR-claude-persist-verify-trigger-not-just-codec-001`):
    Zed saves the item when it joins the workspace and on `UpdateTab`, which the tab emits on each
    URL or title change and when it takes a page. The Test phase's relaunch proves it end to end.
  - No Zed path touched; no ledger row.
- **Checks:** `cargo check`, `cargo fmt`, `cargo clippy -p marley_browser -p marley_workbench
  --all-targets -- -D warnings` clean; `shellcheck` on `script/e2e.sh` clean.
- **Checklist (no TaskCreate in this harness):** service ✓, the start ✓, adopting tabs ✓, the
  table and the item ✓, the restore ✓, `open` ✓, the harness ✓, review ✓.

Phase 2 PASS. Next: `/pipeline:test`.

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/494-browser-restore.sh`, in the headless sway with an offline
  Chromium. A loopback site: `first.html`, a field and the time the page loaded;
  `second.html`. The run opens the first page, types into its field, opens the second with
  Ctrl+T, then `quit_marley` and `launch_marley`; then `quit_marley`, stops the Chromium unit,
  and `launch_marley` again. After each state it prints `browser_tabs` and the profile database's
  rows: the workspace's `items` of kind `MarleyBrowserTab` and `marley_browser_tabs`.
- **Shots** (the final run, on the build with both fixes below):
  - `494-01-before`: `repo — bash | First page | Second page`, the second in front.
  - `494-02-reattached` (REQ-001): after the relaunch the same tabs in the same order; the first
    page still holds "typed before the relaunch" and its load time, 22:14:48, is the load from
    before the quit: the page was reattached, not loaded again. `browser_tabs` gives the same two
    page ids as before.
  - `494-03-reopened` (REQ-002, REQ-004): after the Chromium unit stopped ("the unit: inactive")
    and the second relaunch, the same two tabs at their URLs, the field empty and a new load time,
    22:15:24; `browser_tabs` lists two new page ids and no third page.
- **The run log** (REQ-003): each `items` row is a kind and an id only; the URL and title live
  in `marley_browser_tabs`, one row per tab, rewritten with the new page ids after the second
  relaunch and with the items' new ids after each launch (Zed's item ids change per launch, and
  `cleanup` drops the old rows).
- **Fixes found here.**
  - The first run's relaunched Marley panicked at start in gpui's Wayland keyboard code
    (`keymap_state.as_mut().unwrap()` on a `Modifiers` event): its first keymap from the headless
    sway was `NoKeymap`, since the seat's newest virtual keyboard, a step's `wtype`, had gone. The
    harness now starts a keyboard holder before each launch (`hold_keyboard`), so the seat's
    live keyboard is the holder's when Marley binds it. gpui's unwrap is Zed's and stays: a real
    compositor always sends a keymap.
  - #488's rerun showed `488-04-wider` at 1100 wide after the dock closed, twice. A debug run
    showed Chromium had the page at 1340 (the page said so) while the tab kept the old frame, and
    only with a cross-site iframe on the page. A CDP probe on a scratch Chromium reproduced it
    without Marley: a page made by `Target.createTarget` with an out-of-process iframe sent no
    screencast frame after a viewport change in four of six runs, and a page Chromium opened at
    start in none of five. Giving the page's own window the viewport's size
    (`Browser.getWindowForTarget`, `Browser.setWindowBounds`) before the override fixed it in four
    of four runs; `Page::set_viewport` does that now. #494 exposed this by making the first page
    a created one; every page Ctrl+T or an agent made since #493 had the same stall.
- **Regressions on this build:** #488 (01 to 05 as at #488, 04 laid out again at 1340 by 860;
  the unit's command line now ends `--no-startup-window`), `488-no-chromium` (the reason and the
  hint), #489 (every shot as at #489; latency median 19.5 ms, 95th percentile 40.4 ms), #490
  (the fifteen shots as at #490), #492 (every criterion; the cross-site frame painted in 01),
  #493 (the seven shots as at #493).
- **Focus report:** every run in its own headless sway, each ending "hyprland: 0 Marley windows
  before the run, 0 after; the run added no rule and did not reload it".
- **Gate:** `just gate-diff` on `marley_browser` and `marley_workbench`: all fifteen gates PASS,
  `GATE GREEN [diff]`, the receipt matching the tree. No pre-existing failure.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  the regressions ✓, the gate ✓.

Phase 3 PASS. Next: `/pipeline:complete`.

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` ("Browser tabs come back after a relaunch", with the stall fix);
  `marley_browser.md` (`--no-startup-window`; the viewport after the window's size);
  `marley_workbench.md` (a start's listed pages, the adopting tab's own page, the saved and
  restored tabs, `open`, the known limits); CONSTITUTION §7 (a quit and a launch on the same
  profile); the plan's B1c row shipped. No Zed path touched.
- **Knowledge appended:** F-claude-494-waiting-tabs-gave-up-on-a-failure-001,
  F-claude-494-a-created-page-with-a-cross-site-iframe-stopped-its-screencast-001,
  F-claude-494-the-relaunched-marley-got-no-keymap-001,
  PR-claude-a-waiter-outlasts-the-failure-it-waits-through-001,
  L-claude-494-a-created-page-keeps-the-window-of-its-first-size-001,
  L-claude-494-zed-item-ids-change-at-each-launch-001,
  AD-claude-494-browser-tabs-reattach-or-reopen-001.
- **Brain:** consultation `8f645b9609844a15b491df556c6eab17` closed with `brain decide`:
  `decisions/marleys-browser-tabs-reattach-to-their-live-pages-after-a-relaunch-or-reopen-their-urls`,
  a follow-up by 2026-10-09.
- **Closed:** the ticket in `tickets/closed/`; BACKLOG holds no #494 row.
