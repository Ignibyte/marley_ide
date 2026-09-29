---
pipeline_id: b0236cf0-5d16-45ea-a78c-276097d62822
ticket: docs/planning/tickets/closed/TICKET-586-file-pages-in-the-browser-tab.md
status: Phase 4 — Complete PASS
title: "A project's local HTML pages open in its Browser tab"
type: feature
slice: prong 3 with prong 1, a follow-up slice of #561
references: [docs/planning/pipeline/completed/561-browser-env-opener.spec.md, docs/planning/pipeline/completed/503-terminal-urls-open-in-the-browser.spec.md, docs/marley/three-prong-plan.md]
---

## Title
A program in a project's terminal that opens a local HTML page, `cargo doc --open`, a coverage
report, a `file://` URL through Python's `webbrowser`, gets it in a Browser tab of that project,
as #561's opener already does for a local dev server, instead of the system browser. Agents'
own navigation keeps plan D15's rule of http and https only.

## Scope
### In
- **`browser_open_url`** (the tool #561's opener calls) opens a local HTML page: a `file://` URL,
  or a path, absolute or relative to the program's folder, that names an existing file whose
  name ends in `.html` or `.htm`. `cargo doc --open` hands the opener a plain path (checked
  2026-09-28: `…/target/doc/<crate>/index.html`), Python's `webbrowser` a `file://` URL. The tab
  opens in the project that holds the program's folder, wherever the file sits (on this box
  `cargo doc` writes under the shared target folder, outside the project), with the focus, as a
  local URL's does.
- **Declined as today** (the opener then hands them to the system's handler): a directory, a
  missing file, a file of any other kind, and every page while `marley.terminal_links` is
  `system_browser`.
- The tool's description and schema say it takes local HTML pages too.
- **Scenario 561's** `file://` step keeps its meaning with a page that is not HTML; its HTML
  page now opens a tab, so the step's check moves.
- `script/e2e/586-file-pages-in-the-browser-tab.sh`.

### Out (explicitly deferred)
- `browser_navigate` and the other agent tools: they keep plan D15's http and https (D2).
- A `file://` link clicked in a terminal's text: Zed opens it in the editor, which #579 kept.
- Local files of other kinds (images, PDFs, text) and directory listings: the system's handler
  opens them.
- A page served from a folder by a local server (a coverage tool's own `--serve`): a local URL,
  which #561 opens already.

## Reference (§20)
Orca's browser (MIT, `/srv/stacks/orca-refs/orca` at `1c2cf120`): a `file://` page opens only as
the first load of a preview the user opened (`src/main/browser/browser-manager-guest-navigation-policy.ts:15-19`,
"initial file:// attach is allowed for user-opened previews"), and its agents' navigate command
refuses `file:` (`browser-client-page-command-execution.ts:33`, `browser-route-guest-lifecycle.ts:88,110`).
Marley keeps the split: a page the user's program opens loads in the project's Browser tab,
and agent navigation stays http and https (plan D15). Upstream Zed: its terminal turns a
`file://` link into a path it opens in the editor (`crates/terminal/src/alacritty/hyperlinks.rs`),
which this change leaves alone. Warp: N/A, no Browser tab.

### Prior art
- **The code we ship.** #561's opener, `crates/marley_workbench/bin/marley-open-url`: it
  forwards every argument and its folder to `browser_open_url` and falls back to `xdg-open`
  unless Marley answers `opened: true`. `browser_tools::open_url` declines at
  `address::agent_url` (`crates/marley_browser/src/address.rs:28-43`, http and https only), then
  applies #503's rule (`links::destination`, http and https only), finds the project
  (`holding`: the deepest local root holding the folder) and opens the tab (`open_url_tab`, a tab
  already on the URL reused). Past `agent_url` nothing refuses `file:`: `Target.createTarget`
  and `Page.navigate` pass it to Chromium, which runs with no file-access flags, so a page
  cannot fetch other files; the address bar already takes typed `file:` URLs
  (`address::url_for`, `SCHEMES`). `browser_open_url` is a `browser.write` tool listed to every
  agent with Marley's bearer, and nothing tells the opener from an agent (the Explore report,
  2026-09-28); `browser_snapshot` returns page text unredacted.
- **Published material.** Chromium loads a `file://` URL as a browser-initiated top-level
  navigation, and without `--allow-file-access-from-files` a `file://` page cannot `fetch` or XHR
  other files (Chromium's file-scheme origins); a web page cannot navigate to `file:`. Cargo's
  `doc --open` runs `$BROWSER <path>` with the index's absolute path (observed 2026-09-28 with a
  logging `BROWSER`); Python's `webbrowser.open` passes its argument, a `file://` URL.
- **Behavior maps.** `docs/orca_architecture/03-browser-and-design-mode.md` (Orca's browser pane);
  nothing in `docs/warp_architecture/` or `docs/zed_architecture/` on local pages.
- Does a crate we build own this seam? No: the opener and its tool are #561's; this widens what
  the tool takes.

## UI proof
UI-AFFECTING (a Browser tab opens on a local page). `script/e2e/586-file-pages-in-the-browser-tab.sh`
(`compositor sway`: it clicks a link inside the page, as 561 clicks the rail), on 561's fixture
(the offline Chromium, the fakes for `xdg-open` and the browsers, the stand-in agent). Setup:
`pages/index.html` with a link to `pages/other.html`, `notes.txt`, a folder `docs/`, and a
project `repo` whose terminal gets the opener. Shots: `586-01-file-url` (Python's
`webbrowser.open("file://…/pages/index.html")`: a Browser tab of the project on "A local page",
in front); `586-02-path` (a program that hands its opener a plain path, as `cargo doc --open`
does: the tab on the page); `586-03-link` (a click on the page's link: the tab on "Another page");
`586-04-declined` (`notes.txt`, `docs/` and a missing page: no new tab, and the fake `xdg-open`
logged the three); `586-05-agent` (the stand-in agent's `browser_navigate` to the page refused,
and its `browser_open_url` of `notes.txt` declined); `586-06-system` (`terminal_links:
system_browser`: the page reaches the fake `xdg-open`, no tab).

## Locked-In Decisions
- D1: The opener's tool opens a local page only when it is an existing file named `.html` or
  `.htm`, from a `file://` URL or a path (resolved against the program's folder), a regular file
  and not a directory. Everything else goes back to the opener's fallback, as today.
- D2: Plan D15 stands for agents: `browser_navigate` and every agent tool stay http and https.
  `browser_open_url` is callable by agents too (Marley's bearer is the opener's), so what it
  opens is bounded: HTML pages, never a directory's listing or another kind of file; and Claude
  Code asks before each call of a write tool by default.
- D3: `marley.terminal_links: system_browser` sends local pages to the system browser, as it
  sends every URL; the opener is not exported then, and the tool declines if called.
- D4: The project is the one holding the program's folder, as for a local URL; the page may sit
  anywhere.
- D5: A page's own links to other local pages load in the tab (a rustdoc page's to the next),
  as Chromium allows a `file://` page; Marley adds no navigation filter.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a program in a project's terminal opens a `file://` URL of an HTML page, the system shall show the page in a Browser tab of that project, with the focus. | Shot `586-01-file-url`; the stand-in's `tabs` |
| REQ-002 | WHEN a program hands its opener a path to an HTML page, as `cargo doc --open` does, the system shall show the page in a Browser tab of that project. | Shot `586-02-path` |
| REQ-003 | WHEN the user follows a link in such a page to another local page, the tab shall show that page. | Shot `586-03-link` |
| REQ-004 | WHEN the file is not an HTML page, or is a folder, or does not exist, the system shall open no tab, and the opener shall hand it to the system's handler. | Shot `586-04-declined`; the fake `xdg-open`'s log |
| REQ-005 | WHERE an agent navigates the browser, the system shall refuse a `file://` URL, and `browser_open_url` shall decline a file that is not an HTML page. | Shot `586-05-agent`; the stand-in's answers |
| REQ-006 | WHILE `marley.terminal_links` is `system_browser`, the system shall send a local page to the system browser. | Shot `586-06-system`; the fake `xdg-open`'s log |
| REQ-007 | The diff gate shall be green, and the golden set shall pass with 561 updated and 586 added. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design in the notes.
- **P2 Code:** the page reader in `marley_browser::address`; `open_url`'s branch; the tool's text;
  fmt and clippy clean; a review of the diff against D2.
- **P3 Test:** scenario 586; 561's step; the golden set; the gate.
- **P4 Complete:** CHANGELOG (the #561 line about `file://` pages superseded); the guide's opener
  paragraph; the architecture notes; the ledger capture; close, archive, commit.
