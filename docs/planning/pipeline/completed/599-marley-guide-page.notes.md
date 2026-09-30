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

### Promotion (2026-09-30, `/pipeline:plan 599`)
- **Run mode:** back to back, as Chad chose; no wait at the plan.
- **Pre-flight:** no active pipeline; README marker present; cargo idle; `/mnt/fast` at 91%.
- **Brain (`rusty-cli brain ask`, consultation 8b459c93f5ee436b891e341675d18d02):** nothing on this
  seam.
- **Seams re-verified:** the right-hand `h_flex` in `title_bar.rs` (378-410): `render_call_controls`,
  `render_connection_status`, `update_version`, then Sign In (when signed out and
  `show_sign_in`), the "Signing in…" label, then the user menu (when `show_user_menu`).
  `marley_page.rs` builds `marley::OpenDecisions` by name with `cx.build_action(..).log_err()`.
  `browser::open_url_tab(workspace, url, window, cx) -> Entity<BrowserView>` reuses a tab of the
  workspace already on the same parsed URL.

### Design (settled)
- **`crates/marley_workbench/guide/index.html`** (new): the page. Drafted in the session
  scratchpad (`guide/`, one file per area) while #598 ran; assembled into one file at Code, with
  the contents `<nav>` generated from the page's `<section id>`/`<h2>` and `<article id>`/`<h3>`
  by the assembly step, so the nav never drifts from the content, and a small inline script that
  filters the nav by text.
- **`crates/marley_workbench/src/guide.rs`** (new):
  - `PAGE: &str = include_str!("../guide/index.html")`.
  - `write_page_in(data_dir: &Path) -> anyhow::Result<PathBuf>`: `guide/index.html` under the data
    folder, created with its folder, written only when its bytes differ from `PAGE`.
  - `open(workspace, window, cx)`: runs the write on the background executor, then back on the
    foreground opens `file://` + the path: `browser::open_url_tab` when the workspace's project is
    local and has a visible worktree, else `cx.open_url`. A write that fails shows the error in the
    workspace (`show_error`), never a silent log.
  - `init(cx)`: `observe_new::<Workspace>` registering `OpenGuide`.
- **`marley_workbench.rs`**: `OpenGuide` in `actions!(marley, …)` with its doc comment ("Opens the
  Marley guide: a Browser tab of this project, or the system browser with no project open"), and
  `guide::init(cx)` in `init`.
- **`crates/title_bar/src/title_bar.rs`** (Zed crate): after `.child(self.update_version.clone())`,
  a `// Marley:` hunk adding `IconButton::new("marley-guide", IconName::CircleHelp)` with
  `IconSize::Small`, `Tooltip::text("Marley Guide")` and an `on_click` that builds
  `marley::OpenGuide` by name and dispatches it on the window (`.log_err()` on the build). Placed
  there, it sits directly before Sign In, the "Signing in…" label or the user menu.
- **`docs/marley/zed-touchpoints.md`**: the row for `crates/title_bar/src/title_bar.rs`, written
  before the Zed edit.

### File manifest
- `crates/marley_workbench/guide/index.html` (Marley crate, new)
- `crates/marley_workbench/src/guide.rs` (Marley crate, new)
- `crates/marley_workbench/src/marley_workbench.rs` (Marley crate)
- `crates/title_bar/src/title_bar.rs` (Zed crate, one additive hunk)
- `docs/marley/zed-touchpoints.md` (the row)
- `script/e2e/599-marley-guide-page.sh` (scenario)
- At Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
  `docs/marley/walkthrough.md` (the `?` in part 1 and appendix B), `docs/marley/guide.md` (commands).

### Checklist (no TaskCreate in this harness)
- [x] Pick · [x] pre-flight · [x] recall · [x] promote · [x] prior art · [x] spec · [x] design ·
  [x] presented (autonomous run)

## Phase 2 — Code
- **Built:**
  - `crates/marley_workbench/guide/index.html` (116 KB): one self-contained page, 11 areas and 92
    articles, each a "What it is" summary then "How to use it" steps, keys, settings and the
    tickets that shipped it; a contents column with a filter that matches each article's whole
    text; light and dark through `prefers-color-scheme`. Written in the session scratchpad as one
    file per area and assembled by `assemble.py` there, which generates the contents from the
    page's `<section id>`/`<h2>` and `<article id>`/`<h3>` so they cannot drift apart.
  - `crates/marley_workbench/src/guide.rs`: `PAGE` (`include_str!`), `write_page_in(data_dir)`
    (the folder made, the file written only when its bytes differ), `open` (the write on the
    background executor, then a Browser tab through `browser::open_url_tab` when the project is
    local with a visible worktree, else `cx.open_url`; a failed write is `show_error`, the task
    `detach_and_log_err`), and `init` registering `OpenGuide`.
  - `marley_workbench.rs`: `pub mod guide`, the `OpenGuide` action with its doc comment, and
    `guide::init` after `browser::init`.
  - `crates/title_bar/src/title_bar.rs`: one `// Marley:` child after `update_version`, an
    `IconButton` (`CircleHelp`, `IconSize::Small`, tooltip "Marley Guide") that builds
    `marley::OpenGuide` by name and dispatches it on the window. Its row in
    `docs/marley/zed-touchpoints.md` was written first.
  - `script/e2e/599-marley-guide-page.sh` (sway): the scenario for Test.
- **Page checks (script, before the gate):** tags balanced; no broken `#` link and no duplicate
  id; no `src`/`href` to a network resource and no `@import`; zero em dashes; the banned-word
  grep finds only "harness" as a noun (an agent harness). REQ-007 coverage: every ticket from
  #438 on that the CHANGELOG names and the page did not were listed; the user-facing ones among
  them (#442, #485, #517, #539, #576) were already described and are now credited; the rest are
  internal (gates #447 #448 #469 #541, e2e tooling #483 #487 #588, the fleet contract #533 #597,
  licenses #446, the ported crates #443 #444 #461 #462 #464, fixes with no user step #458 #467
  #512 #593).
- **Deviations:** none from the design.
- **Review of the diff:** REQ-001 by the hunk's place (after `update_version`, before Sign In,
  "Signing in…" and the user menu, in every sign-in state). REQ-002/003 through `open_url_tab`,
  which reuses a same-URL tab of the workspace. REQ-004: `cx.open_url` for a project that is not
  local or has no visible worktree. REQ-009: `write_page_in` compares bytes before writing. No
  entity is read while updated (`project` is read, a bool taken, then `open_url_tab` runs). The
  title bar hunk is additive; the by-name build `.log_err()`s rather than dropping an error.
- **What the gate found, and the fixes:** the first run failed to compile `guide.rs` (the
  `AppContext` and `TaskExt` traits not in scope for `background_spawn` and
  `detach_and_log_err`); the second failed clippy's `needless_pass_by_ref_mut` on `init`'s `cx` and
  `open`'s `window` and `cx` (now `&App`, `&Window`, `&Context<Workspace>`, and the handler no
  longer takes the unused workspace) and dylint's async-without-await on the background write (now
  `futures::future::lazy`, as `ports.rs` runs its reads). The third run: every gate PASS, receipt
  written (`gate-599c.log` in the session scratchpad).

## Phase 3 — Test
- **Scenario:** `script/e2e/599-marley-guide-page.sh` (sway) on the debug build: a scratch folder
  as the project, a fake `xdg-open` first on Marley's PATH writing its argument to
  `fallback.txt`. The first run measured the points (the filter field at 397, 212; the first
  match at 360, 291) and showed two things to fix, below; the third run is the one recorded.
- **`titlebar.png`** (REQ-001): the title bar's right end reads `?` (the circled help icon), then
  Sign In and its chevron. In `guide.png` the pointer rests on the `?` and its tooltip reads
  "Marley Guide". PASS.
- **`guide.png`** (REQ-002): a Browser tab "Marley Guide" in the project's pane beside `project —
  bash`, focused, on `file:///run/user/1000/marley-e2e/profile.…/guide/index.html` (the run's own
  data folder, so REQ-009's write went there); the rail lists a "Marley Guide" row under the
  project; the page draws its contents column and the Getting started section. PASS.
- **`filter.png`, `article.png`** (REQ-006): "teardown" in the contents filter leaves one entry,
  Agents in terminals › "Drift, review, merge and remove a worktree"; a click on it lands on
  `#worktree-review`: the title with its ticket badges (#560 · #511 · #589 · #591), the "What it
  is." box, then "How to use it" with four numbered steps and a note. PASS.
- **`terminal.png`, `again.png`** (REQ-003): with `project — bash` in front and the guide tab
  behind, a click on the `?` brings the same guide tab forward, still at `#worktree-review`, and
  the tab bar holds one guide tab. PASS (after the fix below).
- **`closed.png`, `palette.png`** (REQ-005): Ctrl+W closes the guide tab (only the terminal left,
  in the tab bar and the rail); `marley: open guide` from the palette opens it again, one tab.
  PASS.
- **`fallback.png`, `fallback.txt`** (REQ-004): `workspace: new window` opens a window with no
  folder (its title bar reads Open Recent Project and has the `?` too); `marley: open guide` there
  hands `file:///run/user/1000/marley-e2e/profile.…/guide/index.html` to the fake `xdg-open`;
  the scenario's `expect` passed. PASS.
- **REQ-007, REQ-008, REQ-009:** by review (Phase 2's page checks and coverage list; `write_page_in`).
- **Found and fixed in Test:**
  1. *A second guide tab.* The first run's `again.png` and `palette.png` showed two guide tabs:
     the contents link had moved the first tab to `…index.html#worktree-review`, and
     `open_url_tab` reuses only a tab on the exact URL. Exact is right for terminal links (a
     single-page app's `#/route` is another page), so `browser::show_tab_where(workspace,
     matches, …)` now finds and brings forward a tab by a predicate, `open_url_tab` uses it with
     exact equality, and the guide matches its own URL with the fragment dropped. Gate green
     again (`gate-599d.log`).
  2. *Light scrollbars in the dark theme.* The page's scrollbars drew light over its dark
     colors; `color-scheme: light dark` on `:root` fixed it (visible in the later shots).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and
  did not reload it"; the sway run stopped with its Marley. The new window's welcome page lists
  the profile copy's recent projects; the shots stay in the scratchpad.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Unreleased, Added: #599); `docs/marley_architecture/marley_workbench.md`
  (a section for `src/guide.rs` and `guide/index.html`, with how the page is kept); the
  `crates/title_bar/src/title_bar.rs` row in `docs/marley/zed-touchpoints.md` checked against the
  hunk shipped (a `?` after `update_version`, by name); `docs/marley/workbench-shell.md` (the
  slice line names #599); `docs/marley/guide.md` (the commands table) and
  `docs/marley/walkthrough.md` (stop 1.1 and appendix B) name the `?` and the command.
- **Ledger:** `F-claude-599-a-page-that-moved-to-its-own-anchor-got-a-second-tab-001` (failures),
  `PR-claude-599-say-which-parts-of-a-url-make-it-the-same-page-001` (prevention rules),
  `AD-claude-599-the-guide-is-one-page-in-the-crate-kept-by-each-ticket-001` (decisions).
- **Brain:** consultation 8b459c93f5ee436b891e341675d18d02 closed with `brain decide`
  (decisions/marleys-user-guide-is-one-html-page-in-the-crate-opened-from-a-in-the-title-bar).
- **Ticket:** closed; archived with this pair; committed with the change.
