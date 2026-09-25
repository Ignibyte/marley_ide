# B1b: Browser tabs, restore on relaunch, and the select picker — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-493-browser-tabs-and-restore.md
- **Pipeline spec:** 493-browser-tabs-and-restore.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the rest of B1.
- **Classification:** feature; `marley_browser` (sessions per page, the select's DOM work) and
  `marley_workbench` (the tabs, the serializable item and its table, the picker). No Zed path
  expected: `register_serializable_item` and the `db` crate are public.
- **Recall (§18.3):**
  - The #403 decision (docs/marley_architecture/embedded-browser-model.md Q4): never put a URL
    in a layout codec; a side table keeps it. The item's own table is that side table.
  - AD-claude-registry-lifecycle-fork-pinned-vs-dropped-001: the old Browser tab dropped its
    resident on last close; here the page closes with its tab (D2).
  - The Explore report: `SerializableItem` needs its own `db` domain, and `Onboarding` is the
    lightest real implementation.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Split.** The queued #493 held three slices (tabs as pages, restore on relaunch, the select
  picker). #493 keeps tabs as pages, the base the other two build on; #494 (restore) and #495
  (the select picker) are minted and queued, each with its ticket, its queued pair and its
  BACKLOG row.
- **Pre-flight:** #492 committed (2b5423f0b1); no other active pipeline; cargo idle.
- **Recall:** the old gpui-era Browser tab dropped its resident on last close
  (`AD-claude-registry-lifecycle-fork-pinned-vs-dropped-001`); here the page closes with its tab.
  `PR-claude-watch-before-you-announce-ready-001` (#492): a page is observed before it is
  announced, so each new page's observers come on before its tab opens. Brain consultation
  `392bca5cfd0c4115b96a668af502b429`: nothing on this seam.
- **The probe (2026-09-25):** `_blank` and `window.open` pages carry `openerId`; a
  `createTarget` page carries none and its `targetCreated` precedes the `createTarget` answer;
  `closeTarget` gives `targetDestroyed`.
- **Seams re-verified:** `Item::on_removed(&self, cx)` runs from `Pane::_remove_item`, which a
  close and a move between panes (`remove_item_and_focus_on_pane`) both reach; at app quit
  Zed removes no items one by one. `Pane::add_item(item, activate_pane, focus_item,
  destination_index, window, cx)`; `Pane::remove_item(item_id, activate_pane,
  close_pane_if_empty, …)`; `Workspace::items_of_type`, `panes()`, `activate_item`.

### Design
- **`marley_browser`:** `Page::attach(connection, target_id)` (attach with `flatten`, the Page
  domain, focus emulation, the observers), which `attach_first` becomes a caller of;
  `Page::close()` (`Target.closeTarget`); `create_page(connection, url)`
  (`Target.createTarget`, answering with the target id).
- **The hub (`browser.rs`):** a `Vec<PageState>` in place of the one page's fields (the page,
  its opener, frame and metadata, title, URL, pending URL, loading, history, dialog, iframes,
  rings, refs, the agent's action, load waiters, viewport, viewers, screencast, held buttons),
  the connection, and `focused` (the page whose tab the user focused last). Events route by
  session: a page's own, or the page that owns the iframe session they came from.
  `Target.targetCreated` for a `page` attaches it in a task of its own and, once attached,
  emits `PageOpened { target, opener, focus }`; `targetDestroyed` and `targetCrashed` drop the
  page and emit `PageClosed`; `targetInfoChanged` updates that page. At start the hub turns on
  discovery, which reports the pages that exist as created, and creates `about:blank` when
  there are none. Every page-scoped method takes the target id; `BrowserEvent`'s variants carry
  it, and each view listens for its own.
- **Tabs:** a subscription made with the hub opens a tab for each `PageOpened` (beside the
  opener's tab and focused when there is one; else in the pane of the focused Browser tab, or
  the active workspace's active pane, activated without the focus) and closes the tabs of each
  `PageClosed`. `BrowserView` holds its target id; its `on_removed` defers a check and closes
  the page when no Browser tab shows it any more. Its focus sets the hub's `focused`.
- **`marley::NewBrowserTab`:** the workspace action creates `about:blank`, waits for the page to
  be attached, focuses its tab and its address bar. Ctrl+T in `MarleyBrowser`. `open` activates
  the newest Browser tab, else gives a tab to the newest page that has none, else calls
  `NewBrowserTab`'s path.
- **The agent tools:** every browser tool's input gains `tab`; `browser_tabs` lists the pages;
  `browser_navigate` gains `new_tab`; answers carry `tab`. `show_for_agent` takes the target.
- **Manifest:** `crates/marley_browser/src/page.rs`; `crates/marley_mcp/src/registry.rs` (the
  schemas, one row); `crates/marley_workbench/{src/browser.rs, src/browser_tools.rs,
  src/marley_workbench.rs, keymap.json}`; `script/e2e/493-browser-tabs.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | open the browser, go to the page, click its `_blank` link | `493-01-new-tab` |
| REQ-002, 005 | the agent: `browser_navigate` with `new_tab`, then `browser_tabs`, then `browser_look` with that `tab` | `493-02-agent-tab`, run log |
| REQ-003 | close the link's tab with a middle click; the agent lists the tabs; close the agent's page through CDP | `493-03-closed`, run log |
| REQ-004 | Ctrl+T | `493-04-ctrl-t` |
| (regression) | #488's, #489's, #490's and #492's scenarios on the new hub | their shots |

### Risks
- The refactor touches every page path of the Browser tab (#488 to #492); the four earlier
  scenarios run again in the Test phase.
- A page with no window to open a tab in lives on without one, and gets one the next time the
  browser is opened.

### Checklist (no TaskCreate in this harness)
pick ✓, split ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (and the probe), spec ✓,
design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built.**
  - `marley_browser::page`: `Page::discover`, `page_ids`, `create`, `attach` and
    `close(connection, target_id)`; `attach_first` is gone.
  - `marley_workbench::browser`: the hub keeps a `PageState` per page, routes each event by the
    session it came from (a page's, or one of its iframes'), attaches each `page` target once
    (`attaching`), and drops a page that closed or went away while it was being attached
    (`closing`). `BrowserEvent` names the page. A global subscription opens a tab for each
    `PageOpened` and closes the tabs of each `PageClosed`; a registry of weak `BrowserView`s
    finds a page's tab. `BrowserView` holds its page's id, its workspace and its window; its
    focus marks the page the tools default to; `on_removed` closes the page after the effect
    cycle unless a pane still holds a tab of it. `marley::NewBrowserTab` (Ctrl+T in a Browser
    tab, and the palette).
  - `browser_tools`: every tool takes `tab`; `browser_tabs`; `browser_navigate` takes `new_tab`
    and opens a page when the browser has none; every answer names its tab.
  - `marley_mcp::registry`: the `browser_tabs` row, the `tab` argument on each browser tool,
    `new_tab`, and `tab` in the outputs.
- **Deviations from the design, and why.**
  - `create_page_task` answers when `Target.createTarget` does, not after the attach: a tab that
    asked for the page (Ctrl+T, `open browser`) takes its id at once, so the page's
    `PageOpened` finds a tab showing it and opens no second one. Only a tab opened while the
    browser starts adopts a page (the first one the start attaches).
  - The start attaches the pages `Target.getTargets` lists itself instead of waiting for
    discovery's events, so the hub knows it has pages from the moment it shows them.
  - `Page::close` takes the connection and a target id: a tab closed before its page attached
    still closes the page.
  - D5, the placement of a tab that must not take the focus (see the spec): Zed's workspace
    restores a lost focus to the nearest pane (`on_focus_lost`), so a tab brought to the front
    of the focused pane would take the focus. The split beside a busy pane is what an agent's
    first page gets while the user works in the agent's terminal.
  - A tab counts as its page's viewer from its first paint in front of its pane until
    `Item::deactivated` or its release, so a page behind another tab stops streaming.
  - An agent's call starts a stopped browser again, as `marley: open browser` does.
- **Review of the diff.**
  - Found and fixed: a page closed elsewhere closed its tab, and the tab's removal sent
    `Target.closeTarget` for the gone page again, logging an error and leaving a stale
    `closing` entry. `close_tabs` now has the tab forget its page first.
  - Re-entrancy: `open` and `new_tab` run inside the workspace's own update, so they read its
    panes through `&mut Workspace` and never through the workspace's entity; the pane lookups
    through `Entity<Workspace>` (`pane_of`) run only from effects: the hub's subscription,
    `cx.defer`, and tasks.
  - A move between panes: Zed removes the tab and adds it again in one update; the deferred
    check reads the panes' items, not `panes_by_item`, which the pane events after the defer
    update.
  - Provenance: nothing from Warp; no Zed crate touched, so no ledger row.
- **Checks:** `cargo check`, `cargo fmt`, and `cargo clippy -p marley_browser -p marley_mcp
  -p marley_workbench --all-targets -- -D warnings`, clean.
- **Checklist (no TaskCreate in this harness):** page.rs ✓, browser.rs ✓, browser_tools.rs ✓,
  registry ✓, the action and its key ✓, review ✓.

Phase 2 PASS. Next: `/pipeline:test`.

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/493-browser-tabs.sh`, in the headless sway with an offline Chromium.
  The loopback site: page A ("Tabs home") with a `_blank` link and a `window.open` button;
  pages B ("Linked page") and C ("Popup page"), which print each key they get, so a shot shows
  which page Marley's keys reach; page D ("Agent page"); page E ("Typed page"). The stand-in
  MCP agent moved from #492's scenario into `browser-fixture.sh` as `mcp_agent`, with `--tab`,
  `--new-tab` and `tabs`; the CDP `agent` gained `close <text>`.
- **Shots** (the second run's, after the viewport fix below; the first run's showed the same):
  - `493-01-new-tab` (REQ-001): tabs `repo — bash | Tabs home | Linked page`, B in front at
    `linked.html`; "bbb", typed after the click, shows in B: the focus went with the new tab.
  - `493-01b-window-open` (REQ-001): after Ctrl+PageUp to A and its `window.open` button:
    `… | Tabs home | Popup page | Linked page`. C opened beside its opener, before B, and "ccc"
    shows in C.
  - `493-02-agent-tab` (REQ-002): the agent's `browser_navigate` with `new_tab` put
    `Agent page` in the tab bar behind C (`… | Popup page | Agent page | Linked page`); C stays in
    front, and "cc", typed after the call, shows in C ("ccccc"): the focus stayed.
  - `493-03a-tab-closed` (REQ-003): a middle click on B's tab closed it; `browser_tabs` right
    after lists A, C and D.
  - `493-03-closed` (REQ-003): the CDP client closed page D and its tab went; `browser_tabs`
    lists A and C.
  - `493-04-ctrl-t` (REQ-004): Ctrl+T in C opened `about:blank` after C, and the URL typed next
    stands in the new tab's address bar.
  - `493-04b-typed`: Enter took the new tab to "Typed page".
- **The run log** (REQ-005): `browser_tabs` gives each page's id, title and URL and marks C, the
  tab the user focused last; at the end it marks the new tab. `browser_look` with D's id
  answered D's URL and title while C had the focus.
- **A fix found here:** the first run's look at D reported 780×493, Chromium's default: a page
  behind another tab had never been laid out, so an agent saw it at a size no tab has. A new
  page now takes the viewport of the page it opens beside (its opener, else the focused page);
  the rerun's look reports 1100×860, the tab's own size.
- **Regressions on the new hub:**
  - #488: shots 01 to 04 as at #488 (the unit, the agent's page and highlight, the page laid
    out again at 1340 wide). 05 differs by design: Ctrl+W now closes the page (D2), so the tab
    opened again shows a new `about:blank` from the same Chromium ("units: 1 running; this run's
    is active"); the scenario's header says so now. `488-no-chromium`: the reason and the hint,
    in the tab that waited for a page.
  - #489: every shot as at #489 (typing, editing, compose, clicks, the wheel to scrollY 500,
    the cross-site frame's field, the clipboard both ways, Super+x kept from the page); latency
    median 22.2 ms, 95th percentile 32.5 ms.
  - #490: the fifteen shots as at #490.
  - #491: fourteen tools now, `browser_tabs` among them; blocks, read and the bridge's life as
    at #491.
  - #492: every criterion holds. The agent's first page now opens in a pane split beside the
    terminal, which keeps the focus (D5), so the page is 535 pixels wide. In the first rerun,
    `492-01` showed the cross-site frame unpainted while the agent's look a moment later had it;
    a second rerun's `492-01` has it painted: the frame's first paint raced the shot.
- **Focus report:** every run was in its own headless sway; each ended "hyprland: 0 Marley
  windows before the run, 0 after; the run added no rule and did not reload it".
- **Out of reach of a scenario:** none of the criteria.
- **Gate:** `just gate-diff` on the scope `marley_browser`, `marley_mcp`, `marley_workbench`: all
  fifteen gates PASS (rustfmt, clippy on every target, audit, deny, shear, gitleaks, shellcheck,
  no-suppressions, source bans, rustdoc, the Zed ledger, manifests, typos, semgrep, dylint),
  `GATE GREEN [diff]`, and the receipt matches the tree. No pre-existing failure.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  REQ-005 ✓, the regressions ✓, the gate ✓.

Phase 3 PASS. Next: `/pipeline:complete`.

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` ("A Browser tab for each page"); `marley_browser.md` (the
  page's life: `discover`, `page_ids`, `create`, `attach`, `close`); `marley_mcp.md`
  (`browser_tabs`, the `tab` argument, `new_tab`); `marley_workbench.md` (the hub's pages, tabs
  as pages and their placement, the view's viewer and close rules, `open` and `NewBrowserTab`,
  the agent tools' `tab`, a known limit); the plan's B1b row shipped, with #494 (B1c) and #495
  (B1d) as their own rows. No Zed path touched, so no ledger row to check.
- **Knowledge appended:** F-claude-493-a-closed-page-was-closed-again-by-its-tabs-removal-001,
  F-claude-493-a-page-behind-a-tab-kept-chromiums-default-size-001,
  PR-claude-lay-out-what-no-view-draws-yet-001,
  L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001,
  L-claude-493-a-move-between-panes-is-a-remove-then-an-add-001,
  L-claude-493-headless-chromium-lives-on-with-no-pages-001,
  AD-claude-493-one-browser-tab-per-page-001.
- **Brain:** consultation `392bca5cfd0c4115b96a668af502b429` closed with `brain decide`:
  `decisions/marleys-browser-one-tab-per-page-and-an-agents-tab-never-takes-the-focus`, a
  follow-up by 2026-10-09.
- **Closed:** the ticket in `tickets/closed/`; BACKLOG holds no #493 row (#494 and #495 stay
  queued).
