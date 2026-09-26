---
pipeline_id: f3c6f1c0-5dea-48b1-89d8-0c806a1406ce
ticket: docs/planning/tickets/open/TICKET-539-headless-chrome-user-agent.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Marley's Chromium introduces itself as Chrome"
type: chore
slice: prong 3, the browser's identity (report 03 §3 item 10)
references: [docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/488-browser-pane.spec.md]
---

## Title
Every page Marley's Chromium shows gets Chromium's own identity without the headless marker: a
user agent that says `Chrome/<major>.0.0.0`, and user-agent client hints that list the same
brands and versions, set on each page's session over CDP before its first request where Marley
controls when that request goes out.

## Scope
### In
- `crates/marley_browser/src/page.rs`: an identity (the user agent string and CDP's
  `userAgentMetadata`) built once per browser start from Chromium's own values; `Page::attach`
  sends `Emulation.setUserAgentOverride` with both on the page's session before it turns the
  page's domains on; the same call for a cross-site iframe's session.
- `crates/marley_workbench/src/browser.rs`: the hub builds the identity when a browser starts and
  hands it to each attach; `create_page_task` opens every page at `about:blank` and sends it to
  its URL once the attach has set the identity (an agent's new tab, a restored tab reopening its
  saved URL); `iframe_attached` sets it on each cross-site iframe's session before observing it.
- `script/e2e/browser-fixture.sh`: the loopback server answers `/headers` with a page that shows
  the request's `User-Agent` and `Sec-CH-UA*` headers, asks for the high-entropy hints with
  `Accept-CH`, and shows `navigator.userAgent`, `navigator.userAgentData.brands` and the
  high-entropy values the page's script reads; `?frame=<url>` embeds another origin's copy.
- `script/e2e/539-headless-chrome-user-agent.sh`, run first on the build before the change.

### Out (explicitly deferred)
- Service and shared workers: targets Marley does not attach, so their own requests may keep
  Chromium's headless identity. The promotion probe (D4) says whether they do.
- A page a page opens (a popup) loads its first document before Marley attaches to it, so that
  first request may keep the headless identity; everything after it carries the new one.
- Pages a start finds alive (a relaunch while the Chromium lived on, #494) keep their document's
  identity until they next navigate.
- Google's sign-in, which may refuse a CDP-driven browser whatever it says it is. Orca's Firefox
  user agent on Google's sign-in hosts (report 03 §2.8) is the fallback if Chad needs one.
- A setting to choose the identity (Orca's "Cleaned" and "Native", report 03 §2.8).

## Reference (§20)
Orca's browser identity (report 03 §2.8): "Cleaned" strips Electron's tokens so the user agent
reads as Chrome, before anything reads it, so that documents and workers agree
(`src/main/browser/browser-process-user-agent.ts`, `initializeBrowserProcessUserAgent`), and the
hint headers are written from the same metadata where Chromium emits them
(`browser-session-ua.ts`). Marley matches the behavior, a Chrome user agent whose hints agree, by
CDP's per-target override instead of Electron's session API. Upstream Zed: N/A, Zed has no
browser, and its `http_client` user agent is for Zed's own requests
(`crates/http_client/src/http_client.rs`). Warp: N/A.

### Prior art
- **Behavior maps and reports.** Report 03 §2.1 ("It sets no user agent, so pages see
  `HeadlessChrome/…`"), §2.8 (Orca's identity modes, and on `accounts.google.com` and
  `accounts.youtube.com` a Firefox 140 user agent with Chromium's hints stripped, because Google
  refuses embedded browsers and flags an inconsistent identity), §3 item 10 (the user agent and
  the `sec-ch-ua` hints must agree; `--user-agent` in `service.rs`, or
  `Emulation.setUserAgentOverride` with `userAgentMetadata` per page), §4 (the identity modes
  skipped). Read in Orca: `browser-process-user-agent.ts`, `browser-session-ua.ts` ("Desktop client
  hints remain browser-owned"), `browser-google-auth-ua.ts` (Firefox sends no `sec-ch-ua*`, so a
  Firefox user agent with Chromium's hints is "a sharper mismatch than either signal alone").
- **Published material.** CDP's Emulation domain (`Emulation.pdl`, read 2026-09-25):
  `setUserAgentOverride` takes `userAgent`, optional `acceptLanguage` and `platform`, and
  experimental `userAgentMetadata`, whose `brands` and `fullVersionList` are optional and whose
  `platform`, `platformVersion`, `architecture`, `model` and `mobile` are required (`bitness`,
  `wow64` and `formFactors` optional). browserless's PR #929 (microlinkhq/browserless): an
  override of the string alone "wipes UA Client Hints entirely", so pages saw a Chrome user agent
  with no `sec-ch-ua*` headers and empty brands; the fix passes brands (Chromium's GREASE entry,
  seeded by the major version), the full version list, platform and the rest. A 2024 capture of
  headless Chrome 126 shows the brand in the hints as well: `sec-ch-ua: "Not/A)Brand";v="8",
  "Chromium";v="126", "HeadlessChrome";v="126"` (deviceandbrowserinfo.com, "How to detect
  (modified, headless) Chrome instrumented with Selenium"). The running unit's `/json/version` on
  2026-09-25: `Browser: Chrome/152.0.7977.82`, `User-Agent: … HeadlessChrome/152.0.0.0 …`.
- **The code we already ship.** `chromium_args` (`crates/marley_browser/src/service.rs:81`)
  passes no identity. `Page::attach` (`crates/marley_browser/src/page.rs:270`) enables the Page
  domain and focus emulation, then `observe` (542: Runtime, Network, Log, and auto-attach with
  `waitForDebuggerOnStart: false`), so a per-page call has one place to go. `Page::create`
  (`page.rs:256`) opens a target at its URL directly, which is why an agent's new tab loads before
  Marley attaches, and `create_page_task` (`browser.rs:809`) is its one caller. Cross-site iframes
  are observed from `iframe_attached` (`browser.rs:1691`). Does a crate we build own the seam? No:
  nothing in the tree sets a page's identity; CDP's Emulation domain does, through the session
  Marley already holds.

## UI proof
UI-AFFECTING (what every page sees; shown by a page that prints it).
`script/e2e/539-headless-chrome-user-agent.sh` (`compositor sway`, keys only, so nothing reaches
Chad's session). Setup: the offline Chromium; the fixture's `/headers` page served on
`127.0.0.1`, embedding its copy from `localhost` (another site, so a cross-site iframe). Steps and
shots: `marley: open browser`, the address bar, `/headers`, loaded twice so the `Accept-CH` hints
arrive (`539-01-page`: the header lines, the script's lines and the iframe's block); the stand-in
agent's `browser_navigate` with `new_tab` to `/headers?first`, the new tab's page showing what its
first request carried (`539-02-agent-tab`). Run first on the build before the change, where the
user agent lines of both shots must read `HeadlessChrome` (kept as `539-00-before-page` and
`539-00-before-agent-tab`, with what the hints said), then on the change.

## Locked-In Decisions
- D1: The identity is set per target over CDP, `Emulation.setUserAgentOverride` with
  `userAgentMetadata`, on each page's session in `Page::attach` and on each cross-site iframe's
  session, before those sessions turn their domains on. The string and the hints come from one
  value, so they agree.
- D2: Pages Marley makes start at `about:blank` and go to their URL once attached, so their first
  request already carries the identity. Ctrl+T and New Browser Tab already start blank; an
  agent's new tab and a restored tab reopening its URL change.
- D3: The identity is Chromium's own less the headless marker: the string from `Browser.getVersion`
  with `HeadlessChrome/` made `Chrome/`; the metadata from Chromium's own high-entropy values, read
  once per browser start in an isolated world of the first attached page that is a secure context
  (Marley's own blank pages are), with every `HeadlessChrome` entry removed from `brands` and
  `fullVersionList`. Marley keeps no table of brands or versions, so a Chromium update needs no
  Marley change.
- D4: No `--user-agent` switch unless the promotion probe shows it keeps Chromium's hints in step
  with the string. A string without matching hints is the mismatch this ticket removes. The probe,
  on a scratch Chromium 152 with the fixture's `/headers` page, with and without the switch,
  covers a page, a popup's first request and a service worker's request, and decides whether the
  switch should cover what D1 cannot reach.
- D5: One identity for every site: no setting and no per-host identity in this chore.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page loads in a Browser tab, the system shall send `User-Agent` and give `navigator.userAgent` as `Chrome/<major>.0.0.0`, with no `HeadlessChrome`. | Shot `539-01-page` |
| REQ-002 | WHEN a page loads in a Browser tab, the system shall give its `Sec-CH-UA` header and `navigator.userAgentData.brands` the same brands and versions, none of them `HeadlessChrome`, and send `Sec-CH-UA-Platform` as `"Linux"`. | Shot `539-01-page` |
| REQ-003 | WHEN a page asks for the high-entropy hints, the system shall give a full version list with the same brands as `Sec-CH-UA`, Chromium's full version, and no `HeadlessChrome` entry. | Shot `539-01-page` (the second load's headers and the script's values) |
| REQ-004 | WHEN a cross-site iframe loads in a Browser tab's page, the system shall give it the same identity as its page. | Shot `539-01-page` (the iframe's block) |
| REQ-005 | WHEN an agent opens a new tab at a URL, the system shall send that tab's first request with the new identity. | Shot `539-02-agent-tab` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion, the probe of
  D4 on a scratch Chromium, and the scenario run on the unfixed build (L-claude-512).
- **P2 Code:** the identity, the attach and iframe calls, the blank-first create; fmt and clippy
  clean; a review of the diff against each REQ and against #494's restore path.
- **P3 Test:** the scenario on the change, every shot read; rerun `494-browser-restore.sh`, whose
  reopened tabs now go through `about:blank` first; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_browser.md`; the ledger capture;
  close the ticket, archive, commit.
