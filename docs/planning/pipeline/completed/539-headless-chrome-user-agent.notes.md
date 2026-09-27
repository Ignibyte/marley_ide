# Marley's Chromium introduces itself as Chrome — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-539-headless-chrome-user-agent.md
- **Pipeline spec:** 539-headless-chrome-user-agent.spec.md

## Phase 1 — Plan
- **Request:** report 03 §3 item 10 of the Orca survey (2026-09-25), in the README's smaller
  items; Chad asked that day for every decided item to be specced. The prompt for this spec:
  Marley's Chromium announces `HeadlessChrome/…`, which bot checks reject; set a normal user agent
  with matching client hints; check what `service.rs` passes today.
- **Classification / tier:** chore, prong 3. Two Marley crates and the fixture; no Zed path. Size
  S (report 03 says S).
- **Recall (§18.3):**
  - L-claude-512-show-the-bug-on-the-unfixed-build-first-001: the scenario runs on the build
    before the change and must show `HeadlessChrome` there, then runs on the change.
  - AD-claude-494-browser-tabs-reattach-or-reopen-001: a restored tab whose Chromium did not live
    on opens its saved URL in a new page; D2 routes that page through `about:blank` first, so
    #494's scenario is rerun.
  - L-claude-492-runtime-and-log-replay-network-does-not-001: domains turned on late miss what
    came before; the override goes before the page's domains are turned on, for the same reason.
  - AD-claude-490 (the browser's e2e scenarios run an offline Chromium): the `/headers` page is
    served on loopback, and nothing leaves the machine.
  - Brain: no page on the browser's user agent (searched 2026-09-25).
- **Discovery:**
  - What Marley passes today (`crates/marley_browser/src/service.rs`, `chromium_args`, 81 to 93):
    `--headless`, `--remote-debugging-port=0`, `--user-data-dir=<profile>`, `--no-first-run`,
    `--no-default-browser-check`, `--password-store=basic`, `--no-startup-window`. No
    `--user-agent`, and no override anywhere in `marley_browser` or `marley_workbench` (no match
    for `userAgent`, `UserAgent` or `HeadlessChrome`).
  - What Chromium says (the running unit, read only, 2026-09-25): `/json/version` answers
    `Browser: Chrome/152.0.7977.82` and `User-Agent: Mozilla/5.0 (X11; Linux x86_64)
    AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/152.0.0.0 Safari/537.36`. The page-side
    brands (`navigator.userAgentData.brands`, `Sec-CH-UA`) were not read, since that means running
    script in one of Chad's pages. A published capture shows headless Chrome 126 listing the brand
    there too: `sec-ch-ua: "Not/A)Brand";v="8", "Chromium";v="126", "HeadlessChrome";v="126"`
    (deviceandbrowserinfo.com, "How to detect (modified, headless) Chrome instrumented with
    Selenium (2024 edition)", read 2026-09-25). The unfixed-build run records what 152 sends.
  - `crates/marley_browser/src/page.rs`: `Page::create` (256, `Target.createTarget` with the URL);
    `Page::attach` (270: `Target.attachToTarget` flat, `Page.enable`,
    `Emulation.setFocusEmulationEnabled`, then `observe`); `set_viewport` (another Emulation call
    on the same session, which the override must not disturb); `observe` (542: `Runtime.enable`,
    `Network.enable`, `Log.enable`, `Target.setAutoAttach` with `waitForDebuggerOnStart: false`).
  - `crates/marley_workbench/src/browser.rs`: `create_page_task` (809, `Page::create` with the URL
    given); `attach` (695, which runs `Page::attach` in a task); `iframe_attached` (1691: an
    auto-attached target of type `iframe` gets `observe` and `watch_selects` on its session);
    `new_page` (2174) and `open_page_in` (2960), which every new page goes through.
  - Orca: `src/main/browser/browser-process-user-agent.ts`, `browser-session-ua.ts`,
    `browser-google-auth-ua.ts`, read at `1c2cf120e3`.
- **Decisions:** D1 to D5 in the spec, as drafted; D3 and D4 were rewritten at promotion.

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo
  held while #584's release install ran); recall ✓; promote ✓ (the pair to `active/`, the backlog
  row removed, the ticket in-progress); the seams re-verified ✓; the probe ✓; spec and design
  updated ✓. The run on the unfixed build waits for the install's golden set to end (Test's
  first step, on the build before any code of this ticket).
- **Recall, added:** PR-claude-restore-is-a-second-constructor-001: the blank-first path must
  cover the restored tab as well as the interactive creators, so every creator is listed below.
  F-claude-494-a-created-page-with-a-cross-site-iframe-stopped-its-screencast-001: `set_viewport`
  resizes the page's window before its metrics override; the identity is another Emulation call
  on the same session and must leave that alone. L-claude-583-one-pipe-many-clients-through-browser-sessions-001:
  Marley's commands reach Chromium through its own browser session in the relay. The brain
  (consultation 9f0c091ef0504d44966eefed4a1adef5): nothing on the browser's user agent.
- **The seams, re-verified** (Explore, after #583's relay):
  - `chromium_args` (`service.rs:249-261`): `--headless`, `--remote-debugging-pipe`,
    `--user-data-dir`, `--no-first-run`, `--no-default-browser-check`, `--password-store=basic`,
    `--no-startup-window`, given to Chromium by the relay's command (`relay_command`, 306-319). No
    user agent anywhere in `marley_browser` or `marley_workbench`.
  - `Page::attach` (`page.rs:270-291`): `Target.attachToTarget` flat, `Page.enable` (283),
    `Emulation.setFocusEmulationEnabled` (284-288), `observe` (289; 565-578). `page.call`
    (346-350) sends on the page's session, `page.call_in` (550-557) on another. The only
    `Browser.getVersion` is the relay's readiness probe (`relay.rs:216-219`), whose answer the
    relay drops; Marley asks for none.
  - `BrowserHub::attach` (`browser.rs:1232-1272`) is the only caller of `Page::attach`, from the
    pages a start finds (1037-1039) and from discovery (3253-3272). `attached` (1274-1333) pushes
    the page's state, lays it out (`set_viewport`) and emits `PageOpened`.
  - `create_page_task` (1366-1395), the only caller of `Page::create`, creates at the URL given;
    the attach follows through `Target.targetCreated`. The creators: Ctrl+T and New Browser Tab
    (`new_tab` 7774-7800) and `marley: open browser` (`open` 7651-7722; a waiting tab's task,
    4227-4241) at `about:blank`; an agent's `browser_navigate` with `new_tab` at `about:blank`,
    then `navigate_task` once `page_of` sees it attached (`browser_tools.rs:1060-1110`); #494's
    restore (`BrowserView::restore` 4300-4327, `open_page_in` at 4324) and `open_url_tab`
    (7804-7845: terminal links, `links.rs:245`, and `browser_open_url`, `browser_tools.rs:310`,
    350) at their URL. An address typed before the attach is queued (`go_to_address` 4509-4517)
    and sent once the page is ready (`page_ready` 4383-4386).
  - `iframe_attached` (2529-2576) finds the parent page by session, keeps `type == "iframe"`, and
    spawns `observe` and `watch_selects` on the iframe's session (2571-2574). An `attachedToTarget`
    that arrives before the parent's state is pushed is dropped (2539-2541), as today: with
    `waitForDebuggerOnStart: false` such an iframe goes unobserved, never paused.
  - Popups: discovery attaches a `page` target with its `openerId` (3253-3272) and opens a tab
    beside the opener; nothing attaches at the browser level, so a popup's first document loads
    before Marley attaches.
  - The relay (`relay.rs:692-743`) forwards `Emulation.*` on a session the client owns unchanged,
    and Marley's root commands go on its own browser session.
  - The fixture's server (`write_server`, `browser-fixture.sh:358-393`): `do_GET` has one route
    of its own (`/slow`) before the file fallthrough; `/headers` goes beside it. Its access log
    carries no header values.
- **The probe** (`scratchpad/539/probe`: `server.py`, `probe.mjs` to `probe5-*.mjs`; the results in
  the spec's Prior art). What decided the design: Chromium 152's own hints name no headless
  brand; `about:blank` has no `navigator.userAgentData`; the switch empties the full version list;
  an override with CDP's required fields and the brand lists left out keeps every hint as
  Chromium gives it. D3 no longer reads the hints from a page, D4 is settled (no switch), and
  REQ-005 needed no new path, since an agent's new tab already starts blank. REQ-006 was added for
  the two creators that open at a URL.
- **The run on the unfixed build** (L-claude-512; the debug build of 422bea4c1c, before any code
  of this ticket was built): the scenario reached every step and failed its first check, "the
  user agent says Chrome/, not HeadlessChrome, in every request and script", as the bug says.
  Every request and every script (the page, its cross-site iframe, the agent's new tab, the tab
  `browser_open_url` opened) said `HeadlessChrome/152.0.0.0`; the brands (`"Not?A_Brand";v="24",
  "Chromium";v="152"`) and the full version list (`"Not?A_Brand";v="24.0.0.0",
  "Chromium";v="152.0.7977.82"`) were Chromium's own, with no headless brand, as the probe said.
  The shots, kept as `539-00-before-page`, `539-00-before-agent-tab` and
  `539-00-before-opened-url`, show the same lines on the page and in the iframe's block.

### Design
- **The identity** (`marley_browser::page`): `pub struct Identity { user_agent: String,
  metadata: serde_json::Value }`. `Identity::from_browser(user_agent: &str)` makes the string
  (`HeadlessChrome/` becomes `Chrome/`; a string without it is kept) and the metadata from the
  build target: `platform` from `std::env::consts::OS` ("linux" → "Linux", "macos" → "macOS",
  "windows" → "Windows", anything else as it is); `architecture` from `std::env::consts::ARCH`
  ("x86" and "x86_64" → "x86", "arm" and "aarch64" → "arm", else ""); `bitness` from
  `usize::BITS`; `platformVersion` "", `model` "", `mobile` false, `wow64` false. No brands, no
  full version list, no full version: Chromium keeps its own. `Identity::read(connection)` asks
  `Browser.getVersion` on the root and builds it. Pure apart from `read`.
- **Where it is applied.** `Page::attach(connection, target, identity: Option<&Identity>)` sends
  `Emulation.setUserAgentOverride { userAgent, userAgentMetadata }` on the new session right
  after the attach, before `Page.enable`. `Page::set_identity(session, identity)` does the same on
  another session, for an iframe. The hub reads the identity once per connection, before it
  attaches anything (the pages a start finds included), keeps it beside the connection, and hands
  it to `Page::attach`; `iframe_attached` sends it on the iframe's session before `observe`. A
  failed `getVersion` or override is logged and the page attaches as before, with Chromium's
  identity: a page with the headless string beats no page.
- **Blank first.** `create_page_task(url)` creates at `about:blank` and records the URL as the
  page's pending address; when the hub has attached the page (the identity set), it navigates
  the page there. An `about:blank` URL needs nothing. Where the tab's view already queues an
  address until `page_ready` (`go_to_address`), the created page's URL rides the same queue, so
  a restored tab and `open_url_tab` reach their URL the way a typed address does.
- **File manifest.** Marley crates: `crates/marley_browser/src/page.rs` (Identity, the attach's
  override, `set_identity`), `crates/marley_workbench/src/browser.rs` (the hub's identity, the
  iframe call, blank first). Scripts: `script/e2e/browser-fixture.sh` (`/headers`),
  `script/e2e/539-headless-chrome-user-agent.sh`. No Zed path, so no ledger row.

### E2E plan
`compositor sway`, keys only. Setup: `offline_chromium`; the fixture's site served on 127.0.0.1;
`/headers?frame=http://localhost:<port>/headers` embeds the same server's page under another
site's name (the offline resolver rules pass `localhost`); `write_mcp_agent`; `open_path` a
scratch repository.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `marley: open browser`; Ctrl+L; the `/headers?frame=…` URL; Enter; then Ctrl+R so the `Accept-CH` hints arrive | `539-01-page`: the `User-Agent` line and `navigator.userAgent` read `Chrome/152.0.0.0`; the log greps the page's text through `browser_snapshot` |
| REQ-002 | the same page | `539-01-page`: `Sec-CH-UA` and the script's brands list the same entries as the before-run's (`"Not?A_Brand";v="24", "Chromium";v="152"`); `Sec-CH-UA-Platform: "Linux"` |
| REQ-003 | the same page, second load | `539-01-page`: `Sec-CH-UA-Full-Version-List` and the script's full version list agree, `152.0.7977.82`, not empty |
| REQ-004 | the iframe's block of the same page | `539-01-page`: the iframe's `User-Agent` and `navigator.userAgent` read `Chrome/` |
| REQ-005 | `mcp_agent navigate --new-tab "$SITE/headers?first"` | `539-02-agent-tab`: that tab's first request's headers |
| REQ-006 | `mcp_agent open-url "$SITE/headers?opened" <the repository>` (`browser_open_url`, which goes through `open_url_tab`) | `539-03-opened-url`: that tab's first request's headers; 494's scenario rerun for a restored tab |

The unfixed build's run takes the same steps and keeps `539-00-before-page`,
`539-00-before-agent-tab` and `539-00-before-opened-url`; the notes record what its hints said.
Beyond reach: a site's own bot check (no network, and none is under test); Google's sign-in.

### Risks
- The build-target metadata could differ from what Chromium reports on another machine (an arm
  board reporting "arm" and "64" as expected, a platform Marley does not ship on). The scenario
  compares with the before-run's values on this machine; a mismatch elsewhere shows as a
  hint that differs, not as a failure to load.
- Blank first adds one `about:blank` load before a restored or opened tab's URL; the tab's title
  and address show the URL once it navigates, as a typed address does. 494's, 503's and 561's
  scenarios are rerun.
- The override is per target: a popup's first document and service workers keep the headless
  string (Out, measured by the probe).
- `Emulation.setUserAgentOverride` and `setDeviceMetricsOverride` are both Emulation calls on the
  same session; the scenario's page at the tab's size shows the viewport untouched.
- Google may still refuse a CDP-driven browser at sign-in whatever it says it is (Out).

## Phase 2 — Code
- **Checklist** (no task tool): `page.rs` ✓; `browser.rs` ✓; check, fmt and clippy ✓; the
  review ✓. No Zed path.
- **Built.**
  - `marley_browser::page`: `Identity { user_agent, metadata }`; `Identity::from_browser` (the
    string with `HeadlessChrome/` made `Chrome/`; the metadata from `std::env::consts` through
    `hint_platform` and `hint_architecture`, `bitness` from `usize::BITS`, no brands or full
    versions) and `Identity::read` (`Browser.getVersion` on the root). `Page::attach(connection,
    target, identity)` sends `Emulation.setUserAgentOverride` on the new session before
    `Page.enable`; a refused override is logged and the page attaches as Chromium introduces it.
    `Page::set_identity(session, identity)` for an iframe's session; `Page::reset_history`
    (`Page.resetNavigationHistory`).
  - `marley_workbench::browser`: `ProjectBrowser` gains `identity`, `pending_urls` and
    `blank_entries`, and `ProjectBrowser::starting` builds it. `pages_and_identity` reads the
    identity with the pages a start finds, and the Showing update sets it before those pages
    attach. `attach` hands it to `Page::attach`. `create_page_task` creates at `about:blank`, and
    a URL of its own waits in `pending_urls`, or goes at once if the page attached first.
    `attached` takes the page's pending URL, notes it in `blank_entries` and navigates.
    `refresh_history` resets the history of a page in `blank_entries` when its first non-blank
    URL commits, just before it reads the history. `iframe_attached` sets the identity on the
    iframe's session before `observe`. `page_gone`, a failed attach and a page closed while
    attaching drop the page from both lists.
- **Deviations from the plan.**
  - The blank entry. A probe after the design (`scratchpad/539/probe/history*.mjs`): a page
    created at `about:blank` keeps that entry once it navigates (`["about:blank", url]`), where a
    page created at its URL has `[url]`, so a restored or opened tab would have gained a Back to a
    blank page. `Page.resetNavigationHistory` after the commit leaves `[url]` (the same probe), so
    `refresh_history` resets it for a page sent on from blank, once.
  - Where the state lives. The flag first sat in `PageState`, which clippy's
    `struct_excessive_bools` refuses past three bools; it is the hub's `blank_entries` list, like
    `pending_urls`. The identity is read with the start's pages rather than right after the
    connection, and `ProjectBrowser::starting` took the struct literal out of `start`, which
    clippy's `too_many_lines` held to 100 lines. Neither changes when anything happens: the
    identity is set in the same update that attaches the first pages, and discovery follows it.
- **Checks.** `cargo check -p marley_browser -p marley_workbench` clean; `cargo fmt` applied;
  `just clippy marley_browser marley_workbench` (all targets, `-D warnings`) clean after four
  rounds: a first doc paragraph too long and a `PartialEq` nothing used (`page.rs`), then the
  bools and the long `start` (`browser.rs`), then `const fn` for `starting`.
- **Review of the diff.**
  - REQ-001 to REQ-003: one override per page session, before its domains, from one value; the
    probe showed the hints then match Chromium's own. REQ-004: the iframe's session gets it before
    `observe`. REQ-005: the agent's new tab starts blank already and attaches before its
    navigation. REQ-006: every `create_page_task` page starts blank; a restored tab and
    `open_url_tab` go to their URL once attached, and their history keeps only the URL.
  - Races: the pending URL is recorded in the same update that records the placement, as #574's
    placements are, and a page that attached first is navigated at once. A page destroyed before
    its URL commits leaves both lists through `page_gone`.
  - Re-entrancy: `refresh_history` takes the hub in its own update, outside any other; the
    navigation in `attached` runs inside the hub's own update through `navigate`, which the code
    already does from there.
  - Provenance: CDP calls only, nothing from Warp or a Zed function body.
  - Found and fixed: a page closed while attaching kept its pending URL; it is now dropped
    there too.
- **Addendum, from Test's first run.** Every request and script said `Chrome/` except the
  cross-site iframe's script, which still read `HeadlessChrome` although its request said
  `Chrome/`. A probe (`scratchpad/539/probe/iframe.mjs`) that answered the iframe's
  `attachedToTarget` at once got `Chrome/` either way, and one that held the iframe at its start
  (`waitForDebuggerOnStart: true`) and let it run after the override got it too. So Marley's own
  path, from the relay through the hub's event loop to the override, arrived after the local
  iframe's document committed. The change:
  - `Page::observe`'s auto-attach holds each new target at its start. `Identity::apply` (the
    override on any session, in place of `Page::set_identity`) and `page::resume`
    (`Runtime.runIfWaitingForDebugger`) are new.
  - `iframe_attached` became `child_attached`. For every auto-attached target it takes the
    connection and the identity, applies the identity to an iframe, keeps the iframe and
    observes it when its page is known (`record_iframe`), and resumes it when it was held. A
    target whose page is not attached yet (the race the seams check named), or that is not an
    iframe (a worker), still runs on, so nothing stays held.
  - Clippy clean again (all targets).
- **Addendum, from Test's rerun of 494.** The tab a restart reopened drew its page 87 pixels
  short (`494-03-reopened`: the page ended at y=881 over a dark band, where #507's golden run of
  the same scenario filled the pane), although `browser_look` read the page's viewport as
  1100 by 860. Probes (`scratchpad/539/probe/viewport*.mjs`) found the cause. A headless window of
  1100 by 860 has a content area of 1100 by 773. Sent in Marley's order (the URL goes out, then
  the tab's size and the stream it draws), `Page.startScreencast` failed with "Not attached to an
  active page" while the first commit replaced the blank document. The size and the stream then
  kept the window's size, and the tab sends a size only when it changes. So when the first
  non-blank URL commits for a page in `blank_entries`, `refresh_history` sends the page's
  viewport again and stops and starts its stream while the tab draws it. A size that already
  held changes nothing. The rerun's `494-03-reopened` fills the pane, with Back dim.

## Phase 3 — Test
- **Checklist** (no task tool): REQ-001 to REQ-006 ✓; 494, 488 again ✓; the golden set with 539
  ✓; the gate ✓.
- **The scenario:** `script/e2e/539-headless-chrome-user-agent.sh` (`compositor sway`, keys only)
  and the fixture's `/headers` route (`browser-fixture.sh`: the request's `User-Agent` and
  `Sec-CH-UA*` headers, `Accept-CH` for the high-entropy hints, `?frame=<url>`, and the page's
  script reporting what it read as `?report=<json>`, each logged as a line). Every shot and the
  server's log come first and the checks after, so the unfixed build's run shows each step
  (L-claude-512, in Phase 1's promotion entry).
- **Runs on the change.** The first run: five of six checks passed, the iframe's did not (its
  request said `Chrome/`, its script `HeadlessChrome`). The held auto-attach fixed it (Phase 2's
  first addendum). Then 494 run again showed the reopened tab's page drawn 87 pixels short; the
  size and stream after the first commit fixed it (Phase 2's second addendum). The final build:
  all six checks pass. The server heard `Chrome/152.0.0.0` in every request (the page's two
  loads, the iframe's two, `/headers?first`, `/headers?opened`). Every script reported the same,
  with brands `"Not?A_Brand";v="24", "Chromium";v="152"`, a full version list
  `"Not?A_Brand";v="24.0.0.0", "Chromium";v="152.0.7977.82"` equal to the header's, and platform
  Linux. The focus report: "hyprland: 0 Marley windows before the run, 0 after; the run added no
  rule and did not reload it".
- **The shots** (read; the final build's run):
  - `539-01-page`: the page's block and the iframe's block both read `Chrome/152.0.0.0` in the
    request and in `navigator.userAgent`, with Chromium's brands and full version list; the
    iframe's request has no full version list (a cross-site frame is not sent the high-entropy
    hints). REQ-001 to REQ-004. The before shot, `539-00-before-page`, reads `HeadlessChrome` in
    all four places.
  - `539-02-agent-tab`: the agent's new tab, brought forward with Ctrl+PageDown, at
    `/headers?first`, `Chrome/152` in its first request and script. REQ-005.
  - `539-03-opened-url`: the tab `browser_open_url` opened at `/headers?opened`, `Chrome/152` in
    its first request and script, the page filling the pane. Its Back arrow is dim (a crop's
    brightest pixel 138, as the dim Forward arrows; the page's and the agent's tab's, whose
    history keeps the blank start they always had, 223). REQ-006.
  - Each shows only Marley in the scenario's sway.
- **494 and 488 again.** 494 (restored tabs): both tabs reopened their saved URLs after a
  restart with a new Chromium; `494-03-reopened` fills the pane, with Back dim, as #507's golden
  run did before this ticket. 488 (the pane): its checks pass; `488-04-wider` shows the page at
  1340 by 860 with its cross-site frame drawn, the #494 screencast case, under the held
  auto-attach.
- **The golden set** with 539 added (36 entries), on the debug build: "regress: all 36 passed".
- **The gate:** `script/gates.sh --diff`: 16 passed, 0 failed, "GATE GREEN [diff]", on the first
  run (`scratchpad/539/gate.log`).
- **Pre-existing, not in scope:** an agent's new tab and a typed address keep the blank start in
  their history (Back leads to `about:blank`), as before this ticket. Only the pages
  `create_page_task` opens at a URL forget it.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md`: Fixed (websites see Marley's browser as Chrome).
  `docs/marley_architecture/marley_browser.md`: the identity, `apply`, `resume`, the held
  auto-attach and `reset_history`, in "The page". `marley_workbench.md`: pages see Chrome, the
  hub's identity, `child_attached`, blank first with `pending_urls` and `blank_entries`, and the
  size and stream after the first commit. `docs/marley/guide.md`: a bullet on the browser's
  identity and what still sees the headless name. No row in `docs/marley/three-prong-plan.md`
  covers this chore; the Orca survey's README lists it among its smaller items (report 03 §3 item
  10). No Zed path was touched.
- **Knowledge appended:** F-claude-539-a-cross-site-iframes-script-kept-the-headless-name-001,
  F-claude-539-a-reopened-tab-drew-its-page-87-pixels-short-001,
  PR-claude-hold-a-target-at-its-start-until-it-has-what-it-needs-001,
  PR-claude-a-page-sent-on-from-blank-gets-its-size-and-stream-again-001,
  L-claude-539-chromium-152s-hints-name-no-headless-brand-001,
  L-claude-539-a-page-created-at-about-blank-keeps-its-blank-entry-001,
  AD-claude-539-pages-see-chrome-through-a-per-target-override-001. The brain: consultation
  9f0c091ef0504d44966eefed4a1adef5 closed with
  `decisions/marleys-browser-introduces-itself-as-chrome-through-a-per-target-override-marley-539`,
  follow-up by 2026-10-27.
- **Closed:** `tickets/closed/TICKET-539-headless-chrome-user-agent.md`; its backlog row went at
  the promotion.
