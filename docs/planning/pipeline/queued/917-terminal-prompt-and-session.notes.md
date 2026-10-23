# The terminal prompt as a card, the sticky header, session chips — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-917-terminal-prompt-and-session.md
- **Pipeline spec:** 917-terminal-prompt-and-session.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 6 (terminal blocks: the prompt as a card, the sticky header's
  wording).
- **Classification / tier:** feature, M. Marley crate, one `workspace` crate hunk.
- **Recall (§18.3):**
  - F-claude-893-a-toolbar-item-read-the-workspace-001 and
    PR-claude-831-a-view-a-workspace-hook-makes-reads-the-workspace-only-after-new-001: the tab
    bar hook is asked while the pane renders, inside the workspace's update in places; it reads
    only the item it is given and globals, never the workspace or the pane's own entity.
  - PR-claude-798-read-no-pane-bounds-in-render-001: the chips take no pane bounds.
  - PR-claude-833-a-status-item-caps-its-parts-instead-of-shrinking-001: the chips' long parts (a
    folder, a branch) get a `max_w` and truncate; the group stays `flex_none` so the tabs keep
    their room.
  - PR-claude-812-an-element-that-stops-a-press-over-a-view-focuses-it-001: Jump to it and Tell me
    when done stop presses over the terminal and focus it themselves, as the sticky row does.
  - L-claude-808-a-scenarios-tmux-socket-must-be-short-001 and
    PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001: the run's own tmux server,
    short socket path, `-f /dev/null`, killed in `teardown`.
  - L-claude-831-a-clicking-scenario-sets-tmux-hosts-to-nothing-001: the scenario names its hosts.
  - #551's scenario: the private session bus and notification server; banners one project shows
    come at least five seconds apart (#538's cooldown).
  - #637's notes: the suggestion is an inlay in the shell's editor; whether one shows is
    `history_suggestion`.
- **Discovery:**
  - `crates/marley_workbench/src/agent_bar.rs:164-245` (`footer_without_agent`, `place_chip`).
  - `crates/marley_workbench/src/rich_input.rs:452-500` (`paint_hint`, `shell_hint`), `:900-967`
    (`element`).
  - `crates/marley_workbench/src/sticky_header.rs` (whole file).
  - `crates/workspace/src/pane.rs:3606-3633`, `:4347-4357`.
  - `crates/marley_workbench/src/tmux_sessions.rs`, `remote.rs`, `close_guard.rs:166`.
  - `crates/marley_workbench/src/command_watch.rs` (`Watch`, the block-end check),
    `notifications.rs:206`.
  - `crates/marley_workbench/src/block_headers.rs` (`header`, where the button goes).
- **Decisions:** D1–D5 in the spec.

### Design
- Prompt card (Marley crate):
  - `agent_bar::footer_without_agent`: while the prompt editor is open, the editor and the chips
    go into one `prompt_card(...)` (a `v_flex` with `rounded(px(14.))`, `border_1`, `m_2`); the
    flat strip's own top border and background in `rich_input::element` go when it sits in the
    card (a parameter `PromptFrame { Card, Strip }`, not a bool).
  - `changed_count(project, folder, cx) -> usize` beside `branch_of`, from the repository that
    holds the folder: `cached_status().count()`.
  - `last_exit(terminal) -> Option<i32>`: the newest live block's non-zero exit while at the prompt.
  - `rich_input`: `pub(crate) fn shows_suggestion(view, cx) -> bool` for "→ to accept".
  - `key_hint(action, label, focus, window, cx)` with `ui::KeyBinding::for_action_in`.
  - Start an agent here: `window.dispatch_action(Box::new(NewAgent { folder }), cx)`.
- Sticky header (Marley crate): `header`'s row becomes the command, "is above you · <state>", and
  a `Button` "Jump to it" at the right; the whole row's press still jumps.
- Session chips:
  - Zed crate, `pane.rs`: `pub struct MarleyTabBarEnd(pub fn(&dyn ItemHandle, &mut Window, &mut
    App) -> Option<AnyElement>)`, a `Global`; in `configure_tab_bar_start`, when set and the pane
    has an active item, its element goes first among the end children. Ledger row: "`pane.rs`:
    `MarleyTabBarEnd`, an element before the tab bar's end buttons for the active item (#917) |
    the TerminalSession board names the session at the tab row's right | keep the call before the
    end children".
  - Marley crate, `session_chips.rs` (new): `pub(crate) fn tab_bar_end(item, window, cx)`; for a
    `TerminalView`: `SessionKind { Shell(name), Tmux { session, windows }, Ssh { host, link } }`
    and its chip, then the folder and branch chips. Set in `marley_workbench.rs`'s `init`.
- Tell me when done (Marley crate):
  - `command_watch.rs`: `Told` gains `tell_for: Option<usize>`; `pub(crate) fn tell_when_done(view,
    index, cx)` toggles it; the block-end check posts when the block is `tell_for`, skipping the
    threshold.
  - `block_headers.rs`: the button in a running block's header, after the command, its label from
    `command_watch::will_tell(view, index, cx)`.
- File manifest: Zed crate: `crates/workspace/src/pane.rs`. Marley crate: `agent_bar.rs`,
  `rich_input.rs`, `sticky_header.rs`, `session_chips.rs` (new), `command_watch.rs`,
  `block_headers.rs`, `marley_workbench.rs`.

### Visual check plan
- REQ-001, REQ-002: `false`, then type `git log --oneline`; `917-01-prompt`; read each chip and
  its colour, `❯`, the suggestion and "→ to accept", the hints' keys and the button.
- REQ-002: click Start an agent here; `917-02-start-agent`; Escape.
- REQ-003: `seq 300; sleep 8`; settle two seconds (300 lines put the block's start above the
  view, so the top row lies in it); `917-03-sticky`; click Jump to it; `917-04-jumped`.
- REQ-004: `917-05-chips` on the plain terminal, then click `build`'s rail row and shoot again.
- REQ-005: in terminal 1 `sleep 5`; click Tell me when done; Alt+N to terminal 2; settle seven
  seconds; `917-06-tell`; `expect holds` on the notification log for "sleep 5".
- REQ-001, REQ-004 in Day: switch the theme live; `917-07-day`.

### Risks
- The card's margin takes height from the terminal's grid; the footer's rows come from the grid
  (`MarleyTerminalFooter` takes the rows it needs), so the grid loses up to one more row. Check
  the shot for a clipped prompt.
- `cached_status()` covers the whole repository; a terminal in a subfolder counts the repo's
  changes, as `git status --short` from there would.
- The tab bar hook runs for every pane's render; keep it to globals and the item, and return
  `None` fast for non-terminal items.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
