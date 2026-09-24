# Desktop notifications from a terminal, and from Claude Code — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-478-terminal-notifications.md
- **Pipeline spec:** 478-terminal-notifications.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "There is a 'enable claude code notifications' not sure how it
  actually works but lets explore it".
- **What the exploration found.** Warp's chip installs its plugin
  (`claude plugin install warp@claude-code-warp`). Claude Code 2.1.281 writes OSC 9 (iTerm2),
  OSC 99 (kitty), OSC 777 (Ghostty) or BEL itself, but its `auto` channel recognizes only
  iTerm2, kitty, Ghostty and Apple Terminal, so it sends nothing in Zed's terminal. Its hooks
  can return a `terminalSequence` for it to emit. `claude plugin marketplace add` takes a local
  path; `~/.claude/plugins/installed_plugins.json` lists what is installed.
- **Recall.** PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: a frame
  from output may be forged, so a notification only shows text. gpui's system notifications
  have no caller in Zed yet.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Split.** The plugin chip became #482, queued next: the terminal side (scanner, event loop,
  terminal event, notification, focus, mark) is a ticket's work on its own.
- **Recall:** the brain (consultation bd81d2b3a9ae4fada39b03c31fb38efa) returned only unrelated
  follow-ups; PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001 holds, as
  a notification only shows text and focuses a terminal.
- **Seams re-verified.** The vendored event loop's `State` keeps `hooks: DcsScanner` and parses
  each read through `advance_with_hooks`; vte drops OSC 9 and 777 unread; `vendor/README.md`
  lists every Marley hunk. `TerminalView` turns `Event::Bell` into `has_bell` and the system
  bell; the rail reads `has_bell`. gpui's `show_system_notification`, `set_app_identity` and
  `on_system_notification_response` have no caller; the test context records what is shown,
  answers with `simulate_system_notification_response`, and has `deactivate_window`. The rail's
  `activate_terminal` shows a terminal: the workspace in the multi-workspace, then the item,
  then `clear_bell`.

### Design
- `marley_dcs::NotificationScanner` (Marley, a new file `notification.rs`): a state machine over
  `ESC ] … BEL | ESC \`, across reads, with a 4 KiB cap and CAN and SUB cancelling; OSC 9
  (not a digit and `;` after it, ConEmu's commands such as the `9;4` progress) and OSC 777
  `notify;title;body`; control characters stripped from the text.
- Vendor (Marley-owned): `Event::Notification`, a second scanner in `State`, and
  `advance_with_hooks` feeding it the passthrough bytes; the README's hunk list.
- Zed: `TerminalBackendEvent::Notification` (`alacritty.rs`'s conversion, `terminal.rs`) and
  `Event::MarleyNotification { title, body }`; `TerminalView` sets `has_bell` for it and gains
  `marley_workspace()`.
- Marley (`marley_workbench::notifications`): `init` names the app, answers responses, and
  subscribes each new terminal view to its terminal; a notification not from the focused
  terminal of the active window goes to `show_system_notification`, tagged by the view, with the
  escape's title or the tab's text; answering it activates the window, the workspace and the
  item, and clears the bell. The PTY helpers of #477's tests move to `marley_workbench_tests`
  for reuse.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: with another pane focused, a script's OSC 777 becomes one notification with its title and body; an OSC 9 takes the tab's text as its title |
| 002 | driven: from the focused terminal of the active window, none |
| 003 | driven: answering it focuses the terminal |
| 004 | unit: the scanner's cases; the vendored event loop reports `Event::Notification` |
| 005 | driven: the view's bell is set |
| 006 | `just gate-diff` |

## Phase 2 — Code (2026-09-23)
- **Built.**
  - `marley_dcs::NotificationScanner` (Marley, `src/notification.rs`, re-exported with
    `Notification` and `MAX_NOTIFICATION`; `marley_terminal` re-exports `Notification`).
  - Vendor: `Event::Notification`, the scanner in the event loop's `State`, and
    `advance_with_hooks` feeding it the bytes the parser gets; the README's hunk list.
  - Zed `terminal`: `TerminalBackendEvent::Notification` (and the conversion in
    `alacritty.rs`), `Event::MarleyNotification { title, body }`, emitted in `process_event`.
  - Zed `terminal_view`: the arm that sets `has_bell` without the bell's sound, and
    `marley_workspace()`.
  - Zed `agent_ui`: its terminals' match names every `terminal::Event`, so the new variant
    joins its ignored arm (a ledger row of its own).
  - `marley_workbench::notifications`: `init` names the app `Marley`, answers responses, and
    subscribes every terminal view to its terminal; `notify` skips the focused terminal of the
    active window and otherwise shows the notification, tagged by the view; `show_sender`
    activates the window, the workspace and the item, and clears the bell. #477's PTY helpers
    moved to `marley_workbench_tests`.
- **Deviations:** the `agent_ui` arm, which the plan missed; `init` takes `&App`, as nothing in
  it needs more.
- **Review.** The subscription lives with the view and the terminal; a notification is text
  only and its click activates and focuses (PR-claude-474). Clippy: a response taken by value, a
  needless `&mut`.

## Phase 3 — Test (2026-09-23)
- **Tests.**
  - `marley_dcs` (unit): both escapes and both ends; a split read; ConEmu's commands and
    progress, an empty OSC 9, other OSCs and a DCS ignored; CAN, SUB and a new escape
    cancelling; the cap; control characters stripped.
  - Vendor: `a_notification_escape_is_reported_and_its_text_around_it_parsed`.
  - `marley_workbench` (driven, a real PTY that prints the escapes after each line it is
    sent):
    - `a_notification_from_a_terminal_out_of_focus_goes_to_the_desktop` (REQ-001, and the
      OSC 9 named by the tab);
    - `the_focused_terminal_of_the_active_window_goes_to_no_desktop` (REQ-002, then shown once
      the window is inactive);
    - `answering_a_notification_shows_its_terminal` (REQ-003).
    Each waits on the view's bell, which is REQ-005.
- **Negative checks**, each file restored by sha256:
  1. notify whatever the focus: FAIL in the focused-terminal test;
  2. the active window not required: FAIL there too, at the inactive window;
  3. the click not activating the item: FAIL in the answering test;
  4. the view's bell not set: FAIL in all three;
  5. the event loop not feeding the scanner: FAIL in the vendored test;
  6. ConEmu's commands taken as notifications: FAIL in `other_escapes_are_not_notifications`;
  7. the cap ignored: FAIL in `an_escape_past_the_cap_is_dropped_and_the_next_is_read`.
- **Live drive** (`OPEN=<a fresh folder> SETTLE=16 just shot notify-478` with a seed whose
  `.bashrc` prints an OSC 777 six seconds after it starts, while the window sat on the hidden
  workspace, so not active): `busctl --user monitor org.freedesktop.Notifications` caught
  Marley's `Notify` call, app name `Marley`, title `Marley`, the test's body and a `default`
  action for the click, and the desktop's server (Omarchy's Quickshell here, not mako) answered
  with id 21, which was then closed with `CloseNotification`. The click itself needs a pointer
  on Chad's screen and is left to the driven test.
- **The gate's first run was red** on gate:4: five lines of `notifications.rs` never ran, the
  early returns of the click and the `else` for a view made without a window. A terminal view
  is always made in a window (`TerminalView::new` takes one), so that became an `if let`. The
  click's lookups moved into `sender`, and two tests cover what the returns guard against:
  `a_click_on_a_notification_from_a_closed_terminal_does_nothing` and
  `a_response_to_a_notification_marley_did_not_post_does_nothing`. The second run still missed
  two lines, both the closing brace of an `if let` whose `else` cannot happen (a view with no
  window, a window whose root is not a `MultiWorkspace`): the guard became a one-line
  `let … else`, and `sender` takes the window as a `WindowHandle<MultiWorkspace>`, so the
  update gets the multi-workspace itself. `cargo llvm-cov` then showed 71 of 71 lines.
- **Gate:** `just gate-diff` over `marley_dcs`, `marley_terminal`, `marley_workbench`,
  `terminal`, `terminal_view` and `agent_ui`, on the third run: 20 passed, 0 failed,
  `GATE GREEN [diff]`; 338 tests in the Marley crates pass, coverage 100% of lines.

## Phase 4 — Complete (2026-09-23)
- **Docs:** `CHANGELOG.md` (#478); the three-prong plan (T7b's first half shipped);
  `docs/marley_architecture/marley_dcs.md` (the scanner) and `marley_workbench.md` (the
  module); `vendor/README.md` (the hunks); the ledger rows for `alacritty.rs`, `terminal.rs`,
  `terminal_view.rs` and, new, `agent_ui/src/agent_panel.rs`.
- **Knowledge:** AD-claude-478-the-terminal-reads-the-notification-escapes-other-terminals-read-001;
  L-claude-478-a-new-terminal-event-reaches-every-exhaustive-match-001;
  L-claude-478-omarchys-notifications-go-to-quickshell-001. No F-block.
- **Brain:** consultation bd81d2b3a9ae4fada39b03c31fb38efa, decided at Complete.
- **Tickets:** #478 closed; #482 queued next.
