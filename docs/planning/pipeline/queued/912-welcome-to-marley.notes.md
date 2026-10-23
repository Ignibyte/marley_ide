# Welcome to Marley on first run — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-912-welcome-to-marley.md
- **Pipeline spec:** 912-welcome-to-marley.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 4 (first run and dialogs).
- **Classification / tier:** feature, L. Marley crate, one `zed` hunk, one runner change.
- **Recall (§18.3):**
  - F-claude-700-home-refused-zeds-start-workspace-for-its-onboarding-tab-001: on a first start
    Home takes Zed's start workspace with whatever tab Zed opened there. The welcome tab lands in
    that workspace, so it must not stop Home from claiming it, and Home's `ensure` must not take
    the active tab from it.
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001 and
    F-claude-867-opening-a-project-from-a-page-read-the-page-001: the screen's buttons close its
    own tab and open Home or a folder, so they are plain closures over a weak workspace handle,
    and the close runs from a spawned or deferred task, never inside the page's update.
  - PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001: Connect the plugin runs
    `claude plugin install`; the scenario's `CLAUDE_CONFIG_DIR` and stand-in `claude` keep the
    user's Claude Code out.
  - PR-claude-819-marley-starts-an-agent-cli-by-name-in-a-shell-001: the install runs the `claude`
    Marley found, not one in a terminal; `MARLEY_CLAUDE` names the stand-in.
  - #870's notes: prompts and cards share `prompts::render_button`'s roles.
- **Discovery:**
  - `crates/zed/src/main.rs:1606` (the branch), `:40` (the import).
  - `crates/onboarding/src/onboarding.rs:183` (what `show_first_run` mirrors).
  - `crates/marley_workbench/src/marley_workbench.rs` (module list, `init`, `actions!`, and
    `write_layout` at :1445, private: make it `pub(crate)`).
  - `crates/marley_workbench/src/home_page.rs` (`open_here`, `pill`), `agent_versions.rs`
    (`found`), `claude_plugin.rs` (`install`, `ClaudePlugin`), `omarchy_theme.rs` (a
    `pub(crate) fn installed()` from `OmarchyPaths`), `rusty.rs` (the enabled setting).
  - `script/e2e.sh:640-830`: the profile copy and the folder rule.
- **Decisions:** D1–D6 in the spec.

### Design
- `welcome_page.rs` (Marley crate, new):
  - `pub(crate) fn show_first_run(app_state: Arc<AppState>, cx: &mut App) -> Task<Result<()>>`:
    `workspace::open_new`, then in the new workspace add a `MarleyWelcome` to the center and
    focus it, and write `first_open` through `db::write_and_log`, as Zed's does.
  - `MarleyWelcome { workspace: WeakEntity<Workspace>, focus_handle: FocusHandle }`, an `Item`
    whose tab reads "Welcome", `Render` draws the column (max 560 px, the board's card).
  - A row is built from a slice of `WelcomeOption { name, about, on: bool, choose: fn(&mut App) }`
    (a struct, not a tuple). The Theme choice is an enum `ThemeChoice { Night, Day, Omarchy }`,
    the Layout choice `MarleyLayout`, Rusty `RustyChoice { On, Off }`.
  - Settings writes go through `settings::update_settings_file(<dyn Fs>::global(cx), …)`.
  - `pub(crate) fn open_here(workspace, window, cx)` for `marley: show welcome`, activating an
    open one first.
- `marley_workbench.rs` (Marley crate): `pub mod welcome_page`, `ShowWelcome` in `actions!` with a
  doc comment, the handler registered on every workspace.
- `crates/zed/src/main.rs` (Zed crate): the branch's call and the import line; a `// Marley:`
  comment. Ledger row in `docs/marley/zed-touchpoints.md`: "`main.rs`: a first start opens
  Marley's Welcome (`marley_workbench::welcome_page::show_first_run`) in place of
  `onboarding::show_onboarding_view` (#912) | the Dialogs board's first-run screen | keep the
  `FIRST_OPEN` check; if upstream moves it, call Marley's function where Zed's is called".
- `script/e2e.sh` (tooling): `first_run() { FIRST_RUN=1; }` as a scenario keyword; with it the
  copy skips `cp -r "$data/db"` and the threads, and the folder check at :809 allows no folder.
  The header comment names it.

### Visual check plan
- REQ-001, REQ-002: `912-01-night`, the first frame after the window maps; read the mark, the
  title, the four rows and which card in each is lit.
- REQ-002, REQ-003: click Marley Day, then Rusty On; `912-02-day`; `expect holds` on the
  settings file for `"light"` and the Rusty key.
- REQ-004: the Agents row in `912-01-night` ("Found: 2.1.304", "Not on the PATH"); click Connect
  the plugin; `912-03-plugin`; `expect holds` on the stand-in's argv log for `plugin`.
- REQ-005: click Go to Home; `912-04-home`.
- REQ-006: the palette, "marley: show welcome"; `912-05-again`.
- REQ-007: `quit_marley`, `launch_marley`; `912-06-next-start`.
- Points for clicks come from a probe run's shot of `912-01-night` (L-claude-803's rule: measure,
  do not guess).

### Risks
- Zed's onboarding also sets the base keymap, AI and telemetry choices. Marley's defaults already
  cover telemetry (#514); the keymap stays Zed's default unless the user changes it. Recorded, not
  built.
- The welcome tab and Home's page race for the active tab on a first start (#703's index note in
  `home_page::ensure`). Add the welcome tab after `ensure` and activate it.
- A first start with Rusty turned on from the screen starts `rusty-mcp`; the scenario points
  `MARLEY_RUSTY_MCP` at nothing (the runner's default), so the card lights and the connection
  fails quietly.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
