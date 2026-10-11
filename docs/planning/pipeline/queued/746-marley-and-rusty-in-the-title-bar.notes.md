# Marley and Rusty in the title bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-746-marley-and-rusty-in-the-title-bar.md
- **Pipeline spec:** 746-marley-and-rusty-in-the-title-bar.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, the tabs-everywhere shelf
  (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`): "the compromise is Rusty and Marley
  go to the top right next to (?) and Avatar instead. That gives them more spotlight without
  mucking with the left panel tabs that should only be for swapping." Home stays in the status bar.
- **Classification:** feature; Marley crate (`marley_workbench`) plus one small additive hunk in
  Zed's `title_bar`, with its ledger row.
- **Order:** after #745, which takes the Threads button out of the same status item; after this,
  the status item holds Home only.
- **Recall (§18.3):**
  - #739 (`739-rustal-icons-in-the-status-bar.*`): the four buttons as one `StatusItemView` added
    from Marley's `init` with no `crates/zed` edit; its rules (Rusty on and connected, Marley entry
    there, AI on) and handlers (`rusty::open_home_here`, `assistant::talk_to_marley`); the scenario
    found the buttons' places by a first run that stopped at the first shot (`STOP_AT_BAR`).
  - #599: the title bar's `?`, a Zed touch in `title_bar.rs` that dispatches `marley::OpenGuide`
    by name; its ledger row (`docs/marley/zed-touchpoints.md`, the `title_bar.rs` row).
  - AD-claude-699: `MarleyNewItemMenu`, the house shape for a Marley hook in a Zed crate, a
    `Global` holding a function.
  - #738 D1: talk-to brings an open conversation forward where it is, showing that group.
  - PR-claude-701: the buttons' clicks are plain closures over a weak workspace.
- **Discovery:**
  - `crates/title_bar/src/title_bar.rs`: `init` (104, the per-workspace `TitleBar` at 112-118),
    the `TitleBar` struct (210-223), `render`'s right-hand group (379-425: call controls 384,
    connection status 385, the update notice 386, the `?` 387-400, Sign In 401-406, Signing in
    407-420, the user menu 421-423), `TitleBar::new` (466, the struct built at 539-551).
  - `crates/workspace/src/workspace.rs`: `set_titlebar_item` (3406), `titlebar_item` (3553), drawn
    in `render` (9742).
  - `crates/marley_workbench/src/status_buttons.rs` (152 lines): `init` (15), `RustalButtons`
    (30), `new` and its observers (43-53), `render` (56; Rusty 82-96, Threads 97-113, Marley
    114-134), `StatusItemView` (138-151).
  - `crates/marley_workbench/src/rusty.rs` (lines at 19dda499a2; #743 edits the file):
    `open_home_here` (86), `RUSTY_ICON` (90), `is_connected` (692), `is_on` (697).
  - `crates/marley_workbench/src/assistant.rs`: `marley_present` (996), `talk_to_marley` (1003).
  - `docs/marley/zed-touchpoints.md`: the `title_bar.rs` row (66).
  - Guide: `docs/marley/guide.md` "Buttons at the right of the status bar" and Rusty's
    line above it (#743 is editing the guide, so no line numbers); the in-app page's matching section.

### Design
- **`title_bar.rs` (Zed crate; ledger row first).**
  - `// Marley:` `pub struct MarleyTitleBarButtons(pub Arc<dyn Fn(WeakEntity<Workspace>, &mut
    Window, &mut App) -> AnyView>)` and `impl Global`, beside `init`.
  - A field `marley_buttons: Option<AnyView>` on `TitleBar`; in `new`,
    `cx.try_global::<MarleyTitleBarButtons>().cloned().map(|hook| (hook.0)(workspace.weak_handle(),
    window, cx))`.
  - In `render`'s right-hand group, `.children(self.marley_buttons.clone())` right after the `?`
    child.
  - Three hunks, each with `// Marley: Rusty and Marley sit beside the guide's ? (#746)`.
- **`status_buttons.rs` (Marley crate).**
  - `TitleButtons`: a `Render` view with the weak workspace and the same three observers; Rusty
    then Marley, `IconButton`s sized as the `?` (`IconSize::Small`), with #739's tooltips and
    handlers.
  - `init` sets `MarleyTitleBarButtons` before any window opens (it already runs from
    `crate::init`, before windows), returning `cx.new(|cx| TitleButtons::new(workspace,
    cx)).into()`.
  - `RustalButtons::render`: Home only; its observers drop to what Home needs (none) or stay for a
    later button. The module doc names both places.
- **The in-app guide page:** the title bar's buttons; the status bar's Home.
- **Ledger row** (`docs/marley/zed-touchpoints.md`, the `title_bar.rs` row extended): what changed
  ("`MarleyTitleBarButtons`, a `Global` holding a function that makes a view from the title bar's
  workspace; `TitleBar::new` asks it once and the right-hand group draws the view after the `?`
  (#746)"), why ("Rusty's and Marley's buttons beside the `?` (Chad, 2026-10-11); whether each
  shows rests on Marley's state, which a view of Marley's observes, and `title_bar` depends on no
  Marley crate"), at a merge ("keep the struct, the field set in `new`, and the child after the
  `?`").
- **File manifest:** `crates/title_bar/src/title_bar.rs` (Zed crate, ledger row);
  `crates/marley_workbench/src/status_buttons.rs`, `crates/marley_workbench/guide/index.html`
  (Marley crate); `docs/marley/zed-touchpoints.md`;
  `script/e2e/746-marley-and-rusty-in-the-title-bar.sh`; at Complete, `docs/marley/guide.md`,
  `docs/marley/walkthrough.md`, `CHANGELOG.md`, the architecture note.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | Starts on the repo with Rusty's stand-in and the Marley entry; trusts the project | 746-01-bar: Rusty and Marley after the `?`; the status bar's right end with Home only |
| 003 | Clicks Rusty | 746-02-rusty: Rusty's home page in a repo tab |
| 004 | Clicks Marley | 746-03-marley: a Marley tab in the repo; the log's one `session/new` |
| 004 | Clicks Default's row, then Marley | 746-04-again: the repo shown with the Marley tab in front; still one session |
| 005 | Writes `marley.rusty.enabled: false` to the run's settings; settles | 746-05-rusty-off: the title bar without Rusty, with Marley |
| 007 | `marley: use zed layout` from the palette | 746-06-zed: both buttons in the title bar of Zed's layout (Rusty back on first, or Marley alone, as the run's order gives) |
| 006, 008 | none | The review: the Marley condition moved unchanged; the `// Marley:` comments and the ledger row |

The buttons' coordinates come from a first run that stops at the first shot, as #739's did.

### Risks
- **One view per title bar.** Each workspace has its own `TitleBar`, so each gets its own
  `TitleButtons`; a window with many groups makes one per group, each observing three globals.
  The same holds for #739's status item today.
- **Settings written mid-run** (746-05) reach Marley through Zed's settings watcher; if the reload
  lags, the step settles longer, or Rusty's stand-in is stopped instead (Rusty not connected hides
  the button too).
- **The title bar's width.** On a narrow window the right-hand group crowds the project name; two
  small icon buttons add about 48px.
