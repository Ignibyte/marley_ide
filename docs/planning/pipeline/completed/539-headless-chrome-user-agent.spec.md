---
pipeline_id: f3c6f1c0-5dea-48b1-89d8-0c806a1406ce
ticket: docs/planning/tickets/open/TICKET-539-headless-chrome-user-agent.md
status: Phase 4 — Complete PASS
title: "Marley's Chromium introduces itself as Chrome"
type: chore
slice: prong 3, the browser's identity (report 03 §3 item 10)
references: [docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/488-browser-pane.spec.md, docs/planning/pipeline/completed/494-browser-tabs-restored.spec.md, docs/planning/pipeline/completed/583-chromium-devtools-off-tcp.spec.md]
---

## Title
Every page Marley's Chromium shows has its user agent say `Chrome/<major>.0.0.0`, not
`HeadlessChrome/…`, set on the page's session over CDP before its first request wherever Marley
controls when that request goes out. The probe at promotion showed that Chromium 152's client
hints already carry no headless brand, and that an override which leaves the brand lists out keeps
Chromium's own hints exactly. So the fix is the string alone, plus the fields CDP requires.

## Scope
### In
- `crates/marley_browser/src/page.rs`: an `Identity` (the user agent string and the metadata CDP
  requires) built once per connection from `Browser.getVersion`'s user agent, `HeadlessChrome/`
  made `Chrome/`, and from Marley's own build target (platform, architecture, bitness).
  `Page::attach` sends `Emulation.setUserAgentOverride` with it on the page's session, after the
  attach and before `Page.enable`. The same call runs on a cross-site iframe's session.
- `crates/marley_workbench/src/browser.rs`: the hub asks for the identity when it connects and
  hands it to each attach; `child_attached` sets it on each held cross-site iframe's session
  before it observes it, and lets every held target run on; `create_page_task` creates every page at `about:blank` and sends it to its URL once the
  attach has set the identity (a restored tab reopening its saved URL, a URL opened from a
  terminal's link or `browser_open_url`).
- `script/e2e/browser-fixture.sh`: the loopback server answers `/headers` with a page that shows
  the request's `User-Agent` and `Sec-CH-UA*` headers, asks for the high-entropy hints with
  `Accept-CH`, and shows `navigator.userAgent`, the brands and the high-entropy values its script
  reads; `?frame=<url>` embeds another site's copy.
- `script/e2e/539-headless-chrome-user-agent.sh`, run first on the build before the change.

### Out (explicitly deferred)
- Service workers: targets Marley does not attach. The probe showed their requests keep the
  headless string, with Chromium's own hints, which name no headless brand.
- A popup's first document: it loads before Marley attaches to the popup, so that one request
  keeps the headless string (probe). Once Marley attaches (discovery, `openerId`), the popup's
  later requests carry the identity.
- Pages a start finds alive (a relaunch while Chromium lived on, #494) keep their document's
  identity until they next navigate.
- Google's sign-in, which may refuse a CDP-driven browser whatever it says it is. Orca's Firefox
  user agent on Google's sign-in hosts (report 03 §2.8) is the fallback if Chad asks for one.
- A setting to choose the identity (Orca's "Cleaned" and "Native", report 03 §2.8).

## Reference (§20)
Orca's browser identity (report 03 §2.8): "Cleaned" strips Electron's tokens so the user agent
reads as Chrome, before anything reads it, so that documents and workers agree
(`src/main/browser/browser-process-user-agent.ts`, `initializeBrowserProcessUserAgent`), and the
hint headers come from the same metadata where Chromium emits them (`browser-session-ua.ts`).
Marley matches the behavior, a Chrome user agent whose hints agree, through CDP's per-target
override instead of Electron's session API. Upstream Zed: N/A, Zed has no browser, and its
`http_client` user agent is for Zed's own requests (`crates/http_client/src/http_client.rs`).
Warp: N/A.

### Prior art
- **Behavior maps and reports.** Report 03 §2.1 ("It sets no user agent, so pages see
  `HeadlessChrome/…`"), §2.8 (Orca's identity modes, and on `accounts.google.com` and
  `accounts.youtube.com` a Firefox 140 user agent with Chromium's hints stripped, because Google
  refuses embedded browsers and flags an inconsistent identity), §3 item 10 (the user agent and
  the `sec-ch-ua` hints must agree; `--user-agent` in `service.rs`, or
  `Emulation.setUserAgentOverride` with `userAgentMetadata` per page), §4 (the identity modes
  skipped). Read in Orca: `browser-process-user-agent.ts`, `browser-session-ua.ts`,
  `browser-google-auth-ua.ts`.
- **Published material.** CDP's Emulation domain (`Emulation.pdl`): `setUserAgentOverride`
  takes `userAgent`, optional `acceptLanguage` and `platform`, and experimental
  `userAgentMetadata`, whose `brands`, `fullVersionList` and `fullVersion` are optional and whose
  `platform`, `platformVersion`, `architecture`, `model` and `mobile` are required (`bitness`,
  `wow64` and `formFactors` optional). browserless's PR #929: an override of the string alone
  "wipes UA Client Hints entirely". A 2024 capture of headless Chrome 126 listed
  `"HeadlessChrome";v="126"` among the brands; Chromium 152 no longer does (the probe).
- **The promotion probe** (2026-09-27, `/usr/lib/chromium/chromium` 152.0.7977.82, a scratch
  profile, a loopback `/headers` server, offline resolver rules; never Chad's Chrome or Marley's
  units):
  - Chromium as it is: only the string says `HeadlessChrome/152.0.0.0`. `Sec-CH-UA` is
    `"Not?A_Brand";v="24", "Chromium";v="152"`, and the full version list is
    `"Not?A_Brand";v="24.0.0.0", "Chromium";v="152.0.7977.82"`, in the headers and in
    `navigator.userAgentData`.
  - `about:blank` made by `Target.createTarget` is not a secure context: it has no
    `navigator.userAgentData`, so it cannot give the hints. `chrome://version` is one, and it
    shows the native values the checks below compare with.
  - `--user-agent=<the Chrome string>` puts `Chrome/152` on the page, on a popup's first request
    and on a service worker's requests, and keeps the brands. It empties the full version list,
    though (an empty `Sec-CH-UA-Full-Version-List`, `[]` in script).
  - An override that passes the brand lists empty empties the hints, browserless's failure.
  - An override with only the fields CDP requires, plus `bitness` and `wow64` (`platform`
    "Linux", `platformVersion` "", `architecture` "x86", `model` "", `mobile` false, `bitness`
    "64", `wow64` false), gives the page `Chrome/152` in its header and script, and every
    high-entropy value identical to Chromium's own: the brands, the full version list,
    `uaFullVersion`, the architecture, the bitness and the rest. Passing `fullVersion` changes
    nothing.
  - With the override alone, a popup's first request and a service worker's requests keep the
    headless string, with Chromium's own hints.
- **The code we already ship** (re-verified after #583). `chromium_args`
  (`crates/marley_browser/src/service.rs:249-261`) passes no identity, and nothing in either crate
  sets one. `Page::attach` (`page.rs:270-291`) attaches, then `Page.enable`, focus emulation and
  `observe` (565-578: Runtime, Network, Log, and auto-attach with `waitForDebuggerOnStart:
  false`); the override fits between the attach and `Page.enable`. `page.call` and `call_in`
  send on a page's or an iframe's session. `create_page_task` (`browser.rs:1366-1395`) is the only
  caller of `Page::create`, which opens a target at its URL. Ctrl+T, New Browser Tab and `marley:
  open browser` create at `about:blank`, and an agent's new tab already starts blank and
  navigates once attached (`browser_tools.rs:1087`). A restored tab (`BrowserView::restore`,
  4300-4327) and `open_url_tab` (7804-7845, terminal links and `browser_open_url`) create at their
  URL. `iframe_attached` (2529-2576) observes an iframe's session. #583's relay forwards
  `Emulation.*` unchanged and gives Marley a browser session of its own. Does a crate we build own
  the seam? No: nothing in the tree sets a page's identity. CDP's Emulation domain does, through
  the session Marley already holds.

## UI proof
UI-AFFECTING (what every page sees; shown by a page that prints it).
`script/e2e/539-headless-chrome-user-agent.sh` (`compositor sway`, keys only, so nothing reaches
Chad's session). Setup: the offline Chromium; the fixture's `/headers` page served on
`127.0.0.1`, embedding its copy from `localhost` (another site, so a cross-site iframe). Shots:
`539-01-page` (the page loaded twice so the `Accept-CH` hints arrive: its header lines, its
script's lines and the iframe's block); `539-02-agent-tab` (an agent's new tab: what its first
request carried); `539-03-opened-url` (a tab `browser_open_url` opened: its first request). The
same steps on the build before the change keep `539-00-before-page`, `539-00-before-agent-tab`
and `539-00-before-opened-url`, where the user agent lines must read `HeadlessChrome`.

## Locked-In Decisions
- D1: The identity is set per target over CDP, `Emulation.setUserAgentOverride` with
  `userAgentMetadata`, on each page's session in `Page::attach` before `Page.enable`, and on each
  cross-site iframe's session before it is observed. A page's auto-attach holds each new target
  at its start (`waitForDebuggerOnStart: true`), so the identity reaches an iframe before its
  document commits, and Marley lets every held target run on (`Runtime.runIfWaitingForDebugger`),
  whatever it is and whether or not its page is known yet. Found in Test: without the hold, the
  iframe's request said `Chrome/` but its script still read `HeadlessChrome`.
- D2: Every page Marley creates starts at `about:blank` and goes to its URL once attached, so its
  first request carries the identity. Of the creators, a restored tab and `open_url_tab` change;
  the rest start blank already.
- D3: The string is `Browser.getVersion`'s user agent with `HeadlessChrome/` made `Chrome/`. The
  metadata holds only the fields CDP requires, plus `bitness` and `wow64`, from Marley's build
  target: `platform` "Linux" (macOS "macOS", Windows "Windows"), `platformVersion` "",
  `architecture` "x86" for x86 and x86_64 and "arm" for arm and aarch64, `bitness` from the
  pointer width, `model` "", `mobile` false, `wow64` false. It leaves out `brands`,
  `fullVersionList` and `fullVersion`, so Chromium keeps its own, and nothing is read from a
  page. Marley keeps no table of brands or versions, so a Chromium update needs no Marley change.
- D4: No `--user-agent` switch: the probe showed it empties Chromium's full version list, a
  mismatch of its own, so it cannot cover what D1 does not reach (a popup's first document,
  service workers).
- D5: One identity for every site: no setting and no per-host identity in this chore.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page loads in a Browser tab, the system shall send `User-Agent` and give `navigator.userAgent` as `Chrome/<major>.0.0.0`, with no `HeadlessChrome`. | Shot `539-01-page` |
| REQ-002 | WHEN a page loads in a Browser tab, the system shall give its `Sec-CH-UA` header and `navigator.userAgentData.brands` the brands and versions Chromium gives on its own, none of them `HeadlessChrome`, and send `Sec-CH-UA-Platform` as `"Linux"`. | Shot `539-01-page`; the before-run's shot for Chromium's own |
| REQ-003 | WHEN a page asks for the high-entropy hints, the system shall give a full version list with the same brands as `Sec-CH-UA` and Chromium's full version, with no `HeadlessChrome` entry and none empty. | Shot `539-01-page` (the second load's headers and the script's values) |
| REQ-004 | WHEN a cross-site iframe loads in a Browser tab's page, the system shall give it the same identity as its page. | Shot `539-01-page` (the iframe's block) |
| REQ-005 | WHEN an agent opens a new tab at a URL, the system shall send that tab's first request with the new identity. | Shot `539-02-agent-tab` |
| REQ-006 | WHEN Marley opens a tab at a URL itself (a terminal's link, `browser_open_url`, a restored tab), the system shall send that tab's first request with the new identity. | Shot `539-03-opened-url`; 494's scenario rerun (a restored tab) |

## Phase Plan
- **P1 Plan:** this spec, and the design and test plan in the notes; the probe of D4, done at
  promotion; the scenario run on the unfixed build before any code (L-claude-512).
- **P2 Code:** the identity, the attach and iframe calls, the blank-first create; fmt and clippy
  clean; a review of the diff against each REQ and against #494's restore path.
- **P3 Test:** the scenario on the change, every shot read; rerun `494-browser-restore.sh`, whose
  reopened tabs now go through `about:blank` first, and `503` and `561`, whose tabs `open_url_tab`
  opens; the golden set; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_browser.md` and
  `marley_workbench.md`; the ledger capture; close the ticket, archive, commit.
