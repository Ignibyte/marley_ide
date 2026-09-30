# The Marley guide as a page, opened from a ? in the title bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-599-marley-guide-page.md
- **Pipeline spec:** 599-marley-guide-page.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30 (quoted in the ticket). Choices made in the conversation: the `?`
  in the title bar by Sign In; the guide opens in Marley's Browser tab, the system browser when no
  project is open.
- **Classification / tier:** feature; a Marley module, a static page, one Zed touchpoint
  (`crates/title_bar`).
- **Recall (§18.3):** AD-claude-451 kept the title bar untouched for the layout presets; this
  ticket is its first touchpoint, one additive hunk. The walkthrough written 2026-09-29 found
  `guide.md` about 45 features behind (#503 to #597 largely missing) and its grants line stale
  (`terminal.write` is granted beside `browser.write`), so the page's content must come from the
  CHANGELOG and the code, not from `guide.md` alone.
- **Discovery (Explore, 2026-09-30):**
  - Sign In: `render_sign_in_button`, `crates/title_bar/src/title_bar.rs:1198-1216`, added in the
    right-hand `h_flex` at 378-410 (after call controls, connection status and the update
    version; before the user menu at 406-408). Nothing of Marley's in `title_bar` yet; no
    touchpoint row for it.
  - `title_bar` has no Marley dependency and `marley_workbench` depends on it, so the button
    dispatches by name: `cx.build_action("marley::OpenGuide", None)`, as
    `crates/settings_ui/src/marley_page.rs:954-961` does for `OpenDecisions`.
  - Shipping: `include_str!` plus `write_program_in`-style writes under `paths::data_dir()`
    (`crates/marley_workbench/src/mcp.rs:310-363`); `write_if_changed` in
    `crates/marley_terminal/src/shell_integration.rs:170-176` is the compare-then-write shape.
  - Opening: `browser::open_url_tab` (`browser.rs:8219-8261`, `pub(crate)`) reuses a same-URL tab
    of the workspace, else opens one through `BrowserProject::of(project)`. A workspace with no
    worktree would key a Chromium on the hash of nothing (`e3b0c44298fc1c14`), which D4 avoids by
    falling back to `cx.open_url`.
  - Actions: `actions!(marley, [...])` in `marley_workbench.rs:103-304`; workspace actions are
    registered in each module's `init` through `cx.observe_new::<Workspace>` and
    `register_action` (`browser::init`, `browser.rs:7489-7501`).
  - Zed's own help: `zed_actions::OpenDocs` (`crates/zed_actions/src/lib.rs:70-71`, handled in
    `crates/zed/src/zed.rs:920`, `DOCS_URL` zed.dev/docs) and the Help menu's Documentation
    (`app_menus.rs:319-324`). Left as they are.
  - Icon: `IconName::CircleHelp` (`crates/icons/src/icons.rs:72`).

### Design (to confirm at promotion)
- Files: `crates/marley_workbench/guide/index.html` (new), `crates/marley_workbench/src/guide.rs`
  (new: `PAGE` via `include_str!`, `write_page_in(data_dir)`, `open(workspace, window, cx)`, and
  `init` registering `OpenGuide`), `marley_workbench.rs` (the action and `guide::init`),
  `crates/title_bar/src/title_bar.rs` (the button, `// Marley:` comment),
  `docs/marley/zed-touchpoints.md` (the row).
- The page's outline follows the walkthrough's parts; each feature is an `<article>` with an id,
  an `<h3>`, a "What it is" paragraph, and a "How to use it" list, plus a small table of keys or
  settings where it has them and its ticket number. A contents `<nav>` with a filter `<input>`
  whose inline script hides entries that do not match.

### Visual check plan
- REQ-001: `titlebar.png`, a crop-free shot of the window's top; the `?` left of Sign In.
- REQ-002, REQ-005: click the `?` (sway pointer on the button's position, found from the shot or
  a debug selector); `guide.png` shows a Browser tab titled "Marley Guide" on the page. Then close
  it and run `marley: open guide` from the palette; same result.
- REQ-003: click the `?` again; `again.png` shows one guide tab.
- REQ-004: an empty window (`workspace: new window` or a scratch with no folder), a fake `xdg-open`
  on the PATH writing its arguments to `fallback.txt`; the file holds `file://…/guide/index.html`.
- REQ-006: scroll the Browser tab (wheel) to the rail and the Browser tab sections; shots read for
  a summary paragraph followed by steps.
- REQ-007, REQ-008, REQ-009: review.

### Risks
- The page is long prose: its proofreading is the Code phase's longest step. Read it whole in the
  Browser tab at Test as well.
- The `fallback.txt` check depends on gpui's Linux `open_url` calling an `xdg-open` found on the
  PATH; if it resolves another opener first, fake that one too.
