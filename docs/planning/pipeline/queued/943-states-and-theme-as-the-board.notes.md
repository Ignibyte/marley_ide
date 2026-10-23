# Empty states and theme details as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-943-states-and-theme-as-the-board.md
- **Pipeline spec:** 943-states-and-theme-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11: States, and the Theme board's About line).
- **Classification / tier:** feature, S. Marley crates, one hunk each in `project_panel` and `zed`.
- **Recall (§18.3):**
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the project panel calls the
    Files card's function from its render; the function reads no panel and takes only the window
    and the app.
  - F-claude-867-opening-a-project-from-a-page-read-the-page-001: the card's Open a Folder
    dispatches `workspace::Open`, as Home's does today; no page opens a project inside its update.
  - L-claude-897-look-for-zeds-own-view-before-building-one-001: Zed's `ProjectEmptyState` is
    shared by the agent panel, the git panel and the sidebar; the hook sits in the project panel's
    call, not in the shared component.
  - #857's scenario sets the missing harness; #870's makes Zed's save prompt Marley's card; #829's
    shoots About.
  - The brain: to be consulted when the spec is promoted.
- **Discovery:** `home_page.rs` (`render_recent`, `project_choices`, `render`), `states.rs`,
  `project_panel.rs` (the empty branch of `render`), `rail.rs` (the harness card), `prompts.rs`
  (`render_button`), `zed.rs` (`open_about_window`), `marley_workbench/Cargo.toml`.
- **Decisions:** D1–D4 in the spec.

### Design
- **The card:** `states::no_project_card(cx: &App) -> AnyElement` builds
  `EmptyState::new("..", "No project open", "Open a folder, clone one, or start an agent in your
  home folder.")` with Open a Folder (`Role::Primary`, dispatching `workspace::Open::DEFAULT`).
- **Home:** `MarleyHome::window_has_project(&self, cx: &App) -> bool` from the open list
  `project_choices` builds (factored so both read it); `render` puts the card in a centered row
  above the grid while it is false. `render_recent` with no recents returns
  `muted("No recent projects yet.")`.
- **Files:** in `project_panel.rs`, `pub struct MarleyProjectEmptyState(pub fn(&mut Window, &mut
  App) -> AnyElement);` with `impl Global`; the empty branch reads `cx.try_global` and, when set,
  puts its element in the wrapper in place of `ProjectEmptyState` (`// Marley:` comment). Marley
  sets it in `states::init` with a function that centres `no_project_card`.
- **Harness:** the title string in `rail.rs`.
- **Ring:** in `render_button`, `let ring = chosen && role == Role::Primary;` A ring is a
  `BoxShadow` with spread 2 px and no blur, its colour `colors.border_focused.blend(colors.text.opacity(0.6))`;
  the border stays the fill's own while ringed. The signature does not change (32 callers).
- **About:** the label's text gains the second sentence; it stays `Color::Muted`, Small.
- **File manifest:** `crates/marley_workbench/src/home_page.rs` (Marley crate),
  `crates/marley_workbench/src/states.rs` (Marley crate), `crates/marley_workbench/src/rail.rs`
  (Marley crate), `crates/marley_workbench/src/prompts.rs` (Marley crate),
  `crates/marley_workbench/Cargo.toml` (Marley crate), `crates/project_panel/src/project_panel.rs`
  (Zed crate), `crates/zed/src/zed.rs` (Zed crate),
  `script/e2e/943-states-and-theme-as-the-board.sh` (Marley-owned).
- **Ledger rows (docs/marley/zed-touchpoints.md):** extend `crates/project_panel/src/project_panel.rs`
  (`MarleyProjectEmptyState` and its one ask in the empty branch) and `crates/zed/src/zed.rs` (the
  About line's second sentence).

### Visual check plan
- REQ-001: `workspace: new window`; the new window opens on Home with no project; `943-01-home`.
- REQ-002: in that window, show Files (the rail's Files button or `project panel: toggle focus`);
  `943-02-files`.
- REQ-003: back in the repository's window, the rail's harness section; `943-03-harness`.
- REQ-004: edit the README, `pane: close active item`, press Right until Save is chosen;
  `943-04-ring`. Escape after the shot keeps the edit.
- REQ-005: `zed: about`; `943-05-about` (the About window, as #829 shot it).
- Day: `profile_setting theme '"Marley Day"'`, the new window's Home, then the save prompt again;
  `943-06-day`.

### Risks
- A new window may restore the last session's groups instead of Home alone; the scenario starts
  from a fresh profile, and `marley home` in a window with no folder is the fallback.
- Two sentences may push About's buttons down in a 300 px window: D4 grows the height on Marley's
  channel only, if the shot shows it.
- The ring is drawn as a shadow; a parent that clips its children would cut it. The prompt card
  has padding round its buttons.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
