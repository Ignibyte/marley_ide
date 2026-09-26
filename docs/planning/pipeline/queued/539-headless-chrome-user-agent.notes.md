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
- **Decisions:** D1 to D5 in the spec.

### Design
- **The identity** (`marley_browser`, pure where it can be): `pub struct Identity { user_agent:
  String, metadata: Value }`. `Identity::from_parts(browser_user_agent, hints)` makes the string
  (`HeadlessChrome/` becomes `Chrome/`; a string without it is kept) and the metadata (the hints'
  `brands` and `fullVersionList` without any entry whose brand is `HeadlessChrome`; `platform`,
  `platformVersion`, `architecture`, `model`, `mobile`, `bitness`, `wow64` as Chromium gave them).
  The adapter asks `Browser.getVersion` for the string and, in an isolated world of the first
  attached page that is a secure context, `navigator.userAgentData.getHighEntropyValues([
  'architecture', 'bitness', 'fullVersionList', 'model', 'platformVersion', 'wow64'])` together
  with `brands`, `mobile` and `platform`.
- **Where it is applied.** The hub holds the browser's `Option<Identity>`. `Page::attach` takes it
  and, when present, sends `Emulation.setUserAgentOverride { userAgent, userAgentMetadata }` right
  after the attach, before `Page.enable`; `iframe_attached` sends the same on the iframe's session
  before `observe`. Pages attached before the identity exists (the very first one of a start) get
  it the moment it is built.
- **Blank first.** `create_page_task(url)` creates at `about:blank`, waits for the hub to attach
  the page (which sets the identity), then sends `Page.navigate` to the URL, and answers with the
  target id as before. A URL of `about:blank` goes straight through.
- **The fixture.** `serve.py` gains `/headers`: it sends `Accept-CH: Sec-CH-UA-Full-Version-List,
  Sec-CH-UA-Platform-Version, Sec-CH-UA-Arch, Sec-CH-UA-Bitness, Sec-CH-UA-Model`, and a page that
  lists the request's `User-Agent` and every `Sec-CH-UA*` header it received, then a script block
  that writes `navigator.userAgent`, the brands, and the high-entropy values; `?frame=<url>`
  embeds that URL in an iframe below.
- **File manifest.** Marley crates: `crates/marley_browser/src/page.rs`,
  `crates/marley_workbench/src/browser.rs`. Scripts: `script/e2e/browser-fixture.sh`,
  `script/e2e/539-headless-chrome-user-agent.sh`. No Zed path.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`.

### E2E plan
Setup serves the site twice (`127.0.0.1` and the `localhost` name of the same server both work
under the offline Chromium's resolver rules) and builds `/headers?frame=http://localhost:<port>/headers`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `marley: open browser`; Ctrl+L; the `/headers` URL; Enter; then Ctrl+R so the `Accept-CH` hints arrive | `539-01-page`: the `User-Agent` line and `navigator.userAgent` read `Chrome/152.0.0.0` |
| REQ-002 | the same page | `539-01-page`: `Sec-CH-UA` and the script's brands list the same entries; `Sec-CH-UA-Platform: "Linux"` |
| REQ-003 | the same page, second load | `539-01-page`: `Sec-CH-UA-Full-Version-List` and the script's full version list agree, `152.0.7977.82`, no `HeadlessChrome` |
| REQ-004 | the iframe's block of the same page | `539-01-page` |
| REQ-005 | `mcp_agent navigate --new-tab "$SITE/headers?first"`; the new tab brought forward | `539-02-agent-tab`: the headers of that tab's first request |

The unfixed build's run takes the same steps and keeps its shots as `539-00-before-page` and
`539-00-before-agent-tab`; the notes record what its hints said.

### Risks
- Where the metadata comes from: the first attached page may be an insecure context (a relaunch
  onto an `http://` page of another host), where `navigator.userAgentData` is absent. The
  identity then waits for a page that is one; Marley's blank pages are. P1's probe checks that
  `about:blank` made by `Target.createTarget` answers `getHighEntropyValues`.
- The override is per target: popups' first documents and service workers are outside it (Out),
  and the probe measures both before D4 is final.
- A reattached page (#494) keeps the identity its document loaded with until it navigates; a
  site that checks only at sign-in sees the new one on the next load.
- Google may still refuse a CDP-driven browser at sign-in whatever it says it is; report 03 names
  Orca's Firefox identity for Google's hosts as the fallback, left for Chad to ask for.
- `Emulation.setUserAgentOverride` and `setDeviceMetricsOverride` are both Emulation calls on the
  same session; the scenario's page at the tab's size shows the viewport untouched.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
