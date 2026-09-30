---
pipeline_id: a54e966d-6914-4f80-a550-d7579200a9a3
ticket: docs/planning/tickets/open/TICKET-599-marley-guide-page.md
status: Phase 4 — Complete PASS
title: "The Marley guide as a page, opened from a ? in the title bar"
type: feature
slice: workbench shell (help); a title_bar touchpoint
references: [docs/marley/guide.md, docs/marley/walkthrough.md, CHANGELOG.md, docs/planning/pipeline/completed/586-file-pages-in-the-browser-tab.spec.md]
---

## Title
Ship a self-contained HTML guide to Marley with the app, and open it from a new `?` button in the
title bar beside Sign In (and from `marley: open guide`), in a Browser tab of the project on
screen, or in the system browser when the window shows no project.

## Scope
### In
- **The page.** `crates/marley_workbench/guide/index.html`: one HTML file with its styles and a
  small script inline, loading nothing from the network. A contents column down the side with a
  filter field, and one section per feature area, in the walkthrough's order: install and first
  launch, the layouts, the rail, the editor (Zed's), the block terminal, agent CLIs in
  terminals, Zed's Agent Panel, Marley's MCP tools, the Browser tab, launch configs, worktree
  agents, System One, settings, keys, commands, where data lives, troubleshooting. Each feature
  opens with **What it is** (a short summary) and then **How to use it** (numbered steps, the
  keys, the settings it reads, and the ticket that shipped it). Light and dark through
  `prefers-color-scheme`.
- **Its content** is current through #598 and this ticket, taken from `CHANGELOG.md`, the code and
  `docs/marley/walkthrough.md`; `docs/marley/guide.md` is a source only where it is still true.
- **Shipping it.** `include_str!` in `marley_workbench`, as `claude_plugin::BRIDGE` and the opener
  are; on open, the page is written to `<data dir>/guide/index.html` when the file there differs,
  so an updated Marley shows its own guide.
- **The action.** `marley::OpenGuide` ("marley: open guide"), registered on the workspace.
- **Where it opens.** When the active workspace holds a local worktree: a Browser tab of that
  project through `browser::open_url_tab` on the page's `file://` URL, which brings forward a tab
  already on the page. Otherwise: `cx.open_url`, the system browser.
- **The button.** In `crates/title_bar/src/title_bar.rs`, an `IconButton` with
  `IconName::CircleHelp` and the tooltip "Marley Guide", in the right-hand group directly before
  Sign In (before the user menu once signed in), shown whether or not the user is signed in. It
  dispatches `marley::OpenGuide` by name (`cx.build_action`), since `title_bar` depends on no
  Marley crate. One row in `docs/marley/zed-touchpoints.md`.

### Out (explicitly deferred)
- Generating the page from Markdown, search beyond the contents filter, screenshots in the page,
  and a hosted copy.
- Bringing `docs/marley/guide.md` itself up to date (its own docs ticket if wanted).
- A key binding for `marley: open guide`, and an entry in Zed's Help menu.
- Changing Zed's `zed: open docs` (it still opens zed.dev's documentation).

## Reference (§20)
Upstream Zed: the title bar's right-hand group (`crates/title_bar/src/title_bar.rs`, the Sign In
button in `render_sign_in_button` and the user menu), whose buttons Marley's `?` sits among and
matches in size and style; and Zed's own help entry, `zed_actions::OpenDocs` and the Help menu's
Documentation item (`crates/zed/src/zed/app_menus.rs`), which open zed.dev's docs in the system
browser. Marley keeps the idea of a help entry one click away and points it at its own guide,
shipped with the app, in its own Browser tab.

### Prior art
- **Behavior maps.** `docs/zed_architecture/` has no help-surface notes; the walkthrough
  (`docs/marley/walkthrough.md`) and `docs/marley/guide.md` are the content's sources.
- **Published material.** None needed: the page is plain HTML and CSS.
- **Code we already ship.**
  - Zed's `zed_actions::OpenDocs` and `OpenBrowser { url }` (`crates/zed_actions/src/lib.rs`,
    handled in `crates/zed/src/zed.rs`) open a URL with `cx.open_url`; the fallback reuses
    `cx.open_url` the same way.
  - Shipping a file: `include_str!` plus a write under `paths::data_dir()`, as
    `mcp::write_program_in` writes the bridge and the opener and `claude_plugin::write_plugin_in`
    the plugin. Zed's `assets` crate (`fs_embed!` over `assets/`) was the other candidate and is
    rejected: `assets/` is Zed's (a glob touchpoint and Zed's license), and a debug build reads
    it from the checkout.
  - Opening: `browser::open_url_tab` (`crates/marley_workbench/src/browser.rs`) reuses a tab of the
    workspace already on the URL; #586 showed file pages load in the project's Chromium.
  - Dispatch by name from a Zed crate with no Marley dependency:
    `cx.build_action("marley::OpenDecisions", None)` in `crates/settings_ui/src/marley_page.rs`.
  - The icon: `IconName::CircleHelp` (`crates/icons/src/icons.rs`).

## UI proof
The scenario `script/e2e/599-marley-guide-page.sh` (sway, since it clicks) opens a scratch
project, shoots the title bar (`titlebar.png`), clicks the `?` and shoots the Browser tab on the
guide (`guide.png`), scrolls to two sections and shoots them (`guide-rail.png`,
`guide-browser.png`), clicks the `?` again and shoots that no second tab opened (`again.png`), and
runs `marley: open guide` in a window with no folder while a fake `xdg-open` first on the PATH
records its argument (`fallback.txt`).

## Locked-In Decisions
- D1 — One static, self-contained HTML file, written by hand, kept in the Marley crate, not
  under Zed's `assets/`. No network resource: fonts are the system's, styles and script inline.
- D2 — The page reaches the disk at open time, rewritten only when its bytes differ, at
  `<data dir>/guide/index.html`; nothing is written at startup.
- D3 — The button sits in the title bar's right-hand group directly before Sign In, shown in
  every state, as the only change to `title_bar`. It dispatches by name.
- D4 — A local worktree in the active workspace means a Browser tab in that project; anything else
  (no folder, a remote project) means the system browser.
- D5 — The page's prose follows the house writing rules (`no-ai-slop`): at most two em dashes.
- D6 — Later tickets that change what a user sees update the page in their Phase 4, beside the
  CHANGELOG.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a Marley window is open, its title bar shall show a `?` button with the tooltip "Marley Guide" directly before Sign In, or before the user menu when signed in. | Shot `titlebar.png` |
| REQ-002 | WHEN the user clicks the `?` and the active workspace holds a local worktree, Marley shall open the guide in a Browser tab of that project, with the focus. | Shot `guide.png` |
| REQ-003 | WHEN a Browser tab of that workspace already shows the guide, a click on the `?` shall bring that tab forward and open no second one. | Shot `again.png` |
| REQ-004 | WHEN the user clicks the `?` or runs `marley: open guide` and the active workspace holds no local worktree, Marley shall open the guide's `file://` URL in the system browser. | `fallback.txt` from the fake `xdg-open` |
| REQ-005 | WHEN the user runs `marley: open guide` from the command palette, Marley shall do what the `?` does. | Shot `guide.png` (the scenario opens it once each way) |
| REQ-006 | The guide shall present every feature area as a summary of what it is followed by a detailed how-to with steps, keys and settings. | Shots `guide-rail.png`, `guide-browser.png`; review of the page |
| REQ-007 | The guide shall describe every user-facing feature shipped through #598. | Review against `CHANGELOG.md`: a checklist of ticket numbers in the notes |
| REQ-008 | The guide shall load no network resource. | Review: no external `src`, `href` to a stylesheet, or `@import` |
| REQ-009 | WHEN the page Marley carries differs from `<data dir>/guide/index.html`, opening the guide shall rewrite the file before it opens. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes (the page's outline; the
  touchpoint row).
- **P2 Code** — `guide/index.html` (the content is the bulk of the work); `guide.rs` in
  `marley_workbench` (embed, write, open, the action); the title bar button and its touchpoint
  row; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot, then read the page
  whole in Marley for errors.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the walkthrough's
  mention of the `?` (§21); the ledger; close, archive, commit.
