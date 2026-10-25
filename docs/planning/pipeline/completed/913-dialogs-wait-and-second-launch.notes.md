# Wait before closing, a second launch says so, the tmux close prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-913-dialogs-wait-and-second-launch.md
- **Pipeline spec:** 913-dialogs-wait-and-second-launch.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 4 (first run and dialogs).
- **Classification / tier:** feature, M. Marley crate, two `zed` crate hunks.
- **Recall (§18.3):**
  - AD-claude-550-the-close-guard-asks-in-zeds-own-close-paths-and-holds-the-view-001: the guard
    is asked from Zed's close paths and reads only terminal views; the wait must close through
    the same path (`Pane::close_item_by_id`) so the guard is asked again.
  - F-claude-880-a-scenario-opened-a-live-tmux-session-001 and
    PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001: the run's tmux server is its
    own; L-claude-808-a-scenarios-tmux-socket-must-be-short-001: `TMUX_TMPDIR` under
    `$XDG_RUNTIME_DIR`, `-f /dev/null`, killed in `teardown`.
  - L-claude-831-a-clicking-scenario-sets-tmux-hosts-to-nothing-001: the rail's points move with
    the box's own sessions; the scenario names its hosts.
  - L-claude-768-a-prompts-buttons-move-with-its-detail-001: calibrate the dialog's buttons from a
    shot of the final words.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001 and
    PR-claude-819-marley-starts-an-agent-cli-by-name-in-a-shell-001: the stand-in `claude` goes on
    the terminals' PATH through the scenario's `.bashrc`.
  - PR-claude-810-a-bar-for-an-agents-action-has-a-way-out-001: the waiting toast has Keep It
    Open, a way out of the wait.
  - #513's scenario: `second_launch` starts the run's binary on the same profile and waits for it
    to exit.
- **Discovery:**
  - `crates/marley_workbench/src/close_guard.rs` (`guard`, `ask_tmux`, `question`, a new `Waits`
    state in `Guard`).
  - `crates/marley_workbench/src/prompts.rs` (`card`, a `PromptHead` global).
  - `crates/marley_workbench/src/tmux_sessions.rs` (`TmuxSession`, the listing's parse at :368,
    a lookup of a session by host and name).
  - `crates/marley_workbench/src/single_instance.rs` (`note_handed`, `announce_handed`).
  - `crates/zed/src/zed/open_listener.rs:446-455` (the listener's loop).
  - `crates/zed/src/main.rs:1327-1355` (the open-paths task).
- **Decisions:** D1–D5 in the spec.

- **Promoted 2026-10-25:** seams re-read: `close_guard.rs` (`guard`, `question`, `ask_tmux`,
  `working_status`, `hold`), the seat's `title` (`marley_fleet::Session`), `prompts.rs`'s card,
  `tmux_sessions.rs` (`TmuxSession`, `PANE_FORMAT` already carries `pane_current_command`,
  `parse`), `open_listener.rs` `listen_for_cli_connections`, `main.rs` `handle_open_request`'s
  open-paths task (it has the window from `open_paths_with_positions`), `single_instance.rs`.

### Design
- Wait:
  - `guard` picks the buttons from the one working agent: `Close, Wait, Cancel` when
    `agent_events::seat_of` gives a seat, else today's three. Answer 1 is Wait or Show by that
    choice; an enum `Middle { Wait, Show }` names it, not a bool.
  - `Guard` gains `waits: Vec<Waiting { view: WeakEntity<TerminalView>, pane: WeakEntity<Pane>,
    toast: NotificationId }>`; one `observe_global::<AgentEvents>` set in `init` walks it: a view
    whose seat is gone or not `Starting | Working | Waiting` closes from its pane, a released view
    drops out. The guard's `asking` flag is untouched by a wait.
  - The toast: `Workspace::show_toast` with an `on_click("Keep It Open", …)` that removes the
    wait.
  - Words: `question` takes the seat's title for the one-agent case.
- tmux prompt:
  - `prompts.rs`: `pub(crate) struct PromptHead { tile: SharedString, tint: Hsla, line:
    SharedString }` in a `Global` map from message to head; `card` draws a 36 px tile and the line
    when the map has the message. `close_guard` inserts before `window.prompt` and removes in the
    answer's task (and on a refused second prompt).
  - `tmux_sessions.rs`: `command: Option<String>` from `pane_current_command`, `None` for
    `bash`, `zsh`, `fish`, `sh` and the agent CLIs (those already name the agent).
- Hand-off notice:
  - `single_instance.rs`: `static HANDED: Mutex<Vec<PathBuf>>`; `pub fn note_handed(url: &str)`
    keeps a `file://` URL's path; `pub fn announce_handed(window: WindowHandle<MultiWorkspace>,
    paths: &[PathBuf], cx: &mut AsyncApp)` takes the noted ones among `paths` and shows the toast
    in the window's active workspace. A poisoned lock is logged and skipped (no `unwrap`).
  - Zed hunks, each with `// Marley:`: the listener calls `note_handed(&url)` before
    `opener.open`; the open-paths task keeps a clone of the paths, and after the open calls
    `announce_handed` with `_window` (renamed `window`). Ledger rows for `open_listener.rs` and
    `main.rs` in `docs/marley/zed-touchpoints.md`: what changed (#913), why (the board's notice;
    the listener is the only place that knows a URL came from another launch), and at a merge
    "keep the call before `opener.open` / after the open".
- File manifest: Marley crate: `close_guard.rs`, `prompts.rs`, `tmux_sessions.rs`,
  `single_instance.rs`. Zed crate: `crates/zed/src/zed/open_listener.rs`,
  `crates/zed/src/main.rs`.

### Visual check plan
- REQ-001, REQ-002: Return in the stand-in (seat working), `ctrl-shift-w` on its tab;
  `913-01-close`; read the title, the turn's words, the hold's seconds and the button order.
- REQ-003: click Wait; `913-02-waiting` (tab present, toast); Return in the stand-in (seat idle),
  settle; `913-03-closed`; `expect` the stand-in's pid is gone.
- REQ-004: click `build`'s rail row to attach, `ctrl-shift-w`; `913-04-tmux`; read the tile, the
  line ("On this machine · 1 window · sleep is running") and the order; Cancel.
- REQ-005: `second_launch "$E2E_PROFILE" "$E2E_WORK/repo-b"`; settle; `913-05-handed`; `expect`
  the second launch exited 0.
- REQ-001, REQ-002 in Day: switch the theme live (L-claude-828's way), repeat the close;
  `913-06-day`; Cancel.

### Risks
- The seat's title may be the agent's session name rather than the turn's request. If it reads
  oddly in the shot, fall back to "Its turn" and record what the seat carries.
- The listener thread runs before gpui's app is ready; `note_handed` must not touch gpui, only the
  `Mutex`.
- A folder already open in a window: Zed activates that window and opens nothing new; the notice
  still shows there, which is right (the launch handed it over).

## Phase 2 — Code
- **Built:** as scoped in `close_guard.rs`, `prompts.rs` (`PromptHead`, `set_head`,
  `clear_head`, `title`), `tmux_sessions.rs` (`command`, `SHELLS`, `is_local`, `session_of`),
  `single_instance.rs` (`note_handed`, `announce_handed`), and two Zed hunks with their ledger
  rows (`open_listener.rs`, `main.rs`).
- **Deviations:** the turn is the seat's `prompt` label, not its title: the first run's dialog
  quoted "Claude Code", the seat's title being the agent's name. With no prompt the sentence reads
  "It is in a turn."
- **Review:** the wait closes through Zed's own close (D2), from the window the dialog was in;
  the toast's Keep It Open and the observer both end the wait by id, deferred. A hand-off is known
  by the listener noting it (D5).
- **Gate:** `just gate-diff` 17/17 green after missing imports, `option_if_let_else`,
  `needless_pass_by_value`, a rustdoc HTML tag and a duplicate scenario function.

## Phase 3 — Test
- **Scenario:** `script/e2e/913-dialogs-wait-and-second-launch.sh` (sway): #827's stand-in in `a`
  stepped to working (prompt "Run the build"), then to its stop; tmux `build` running `sleep` on a
  private socket; a second launch for `b`.
- `913-01-close`: "Claude Code is still working", "“Run the build” is in a turn. Close keeps it
  running for 60 s so Undo can bring it back.", Cancel · Wait · Close (REQ-001, REQ-002).
- `913-06-day`: the same in Marley Day (REQ-001, REQ-002).
- `913-02-waiting`: the tab still open, "Claude Code in a closes when its turn ends", Keep It Open
  (REQ-003).
- `913-03-closed`: the stand-in stopped: the tab closed (REQ-003).
- `913-04-tmux`: the `tx` tile, "Close the tmux session “build”?", "On this machine · 1 window ·
  sleep is running", the body, Cancel · Hide · Close Session (REQ-004).
- `913-05-handed`: `b` opened by the second launch, "Opened b from another launch" with Dismiss,
  behind `b`'s trust prompt (REQ-005).

## Phase 4 — Complete
- CHANGELOG, `docs/marley_architecture/marley_workbench.md`, two ledger rows.
- Knowledge: nothing new.
