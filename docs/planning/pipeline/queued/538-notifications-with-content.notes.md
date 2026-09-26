# Notifications that say what happened — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-538-notifications-with-content.md
- **Pipeline spec:** 538-notifications-with-content.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey Chad asked for on 2026-09-25 ("we will be taking what it does
  well and bring it in here"): report 01 §3 item 4, notifications that say something. The banner
  says what happened: `<project>: Claude finished` (or needs input, failed), the body the last
  assistant message cut short or `Using <tool>: <input>`; unread written even when the banner is
  held back; acknowledged by viewing; a burst cooldown per project (report 01 §2.5).
- **Classification / tier:** feature, prong 1 T7b's follow-on with prong 2's attention. Rust in
  `marley_workbench` and `marley_agent` (Marley crates), the plugin's `notify.sh`; no Zed crate.
  Queue, after #519.
- **Recall (§18.3):**
  - AD-claude-478-the-terminal-reads-the-notification-escapes-other-terminals-read-001: OSC 9 and
    777 become `Event::MarleyNotification`, the view marks itself as a bell does, and the banner is
    held back for the focused terminal of the active window.
  - AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001: the plugin's fixed
    messages per event, "so the script parses nothing and needs no `jq`". #519 is what lets the
    banner say more without parsing in the hook.
  - L-claude-478-omarchys-notifications-go-to-quickshell-001: banners go to Quickshell, and
    `busctl --user monitor` shows them. This scenario goes further and runs Marley on a private bus
    with a fake server, so no banner reaches Chad's screen at all.
  - PR-claude-pump-state-change-must-set-dirty-to-repaint-001 (gpui era, #203: a badge's
    clear-on-view path forgot to redraw while its set path rode other events): a clear must redraw
    as surely as a set. Here, clearing the mark must refresh the rail.
  - Brain: not consulted at drafting (read-only overnight drafting); the promotion runs
    `brain_ask`.
- **Discovery:**
  - #519's spec (drafted the same night, `519-claude-code-events-in-the-rail.spec.md`): frames are
    an OSC 777 under the reserved title `marley-event`, a base64 JSON summary (`event`,
    `session_id`, `prompt_id`, `tool`, `preview`, `message` cut to 300, `error`, `source`,
    `reason`), from a new `hooks/event.py`; `notifications.rs` hands such a body to a new
    `agent_events` module, which keeps a `marley_fleet` seat per terminal (Working, Waiting with a
    `Question`, Idle, Error, Done); a frame raises no banner and no bell (its D8); `notify.sh` keeps
    its fixed sentences (its Out); the plugin moves to 1.2.0 and the chip offers an update (its D9).
    The preview rule takes `file_path`, `command`, `pattern`, `url`, `query` or `description`, else
    the first string field, which `AskUserQuestion`'s input (a `questions` array) does not have.
  - `crates/marley_workbench/src/notifications.rs`: `Senders` (lines 20-24); `init` (28-45)
    subscribes each `TerminalView` to its terminal; `notify` (49-72): the gate at 56, the tag
    `marley-terminal-<entity id>` at 59, the title from the OSC or the tab (60-63);
    `show_sender` (76-90) shows the terminal and clears its bell on a click.
  - `crates/marley_workbench/claude_plugin/marley/hooks/notify.sh`: the gate (line 5), the fixed
    messages (6-11), the project from `CLAUDE_PROJECT_DIR` cleaned of control characters, quotes,
    backslashes and `;` (13), the answer (14). `hooks.json`: `permission_prompt`, `idle_prompt`,
    `Stop`.
  - `crates/marley_workbench/src/claude_plugin.rs`: the plugin's files are compiled in (`FILES`,
    lines 30-59) and written only at install (`run_install`); `plugin.json` is version `1.1.0`.
    Claude Code keeps installed plugins under `~/.claude/plugins/cache/`, so a changed hook reaches
    an installed copy only through an update.
  - `crates/terminal_view/src/terminal_view.rs`: `has_bell` (163, 440), `clear_bell` (544-547,
    emits `Wakeup`), the bell cleared by typing (1011, 1020, 1329), the notification hunk that sets
    the bell (1187-1190), `marley_workspace` (905).
  - `crates/marley_rail/src/marley_rail.rs`: `TerminalSnapshot::bell` (53), `has_attention` (636);
    `crates/marley_workbench/src/rail.rs`: `terminal_snapshot` (1666-1722) reads `has_bell`, the
    dot drawn at 1295-1311, `activate_terminal` clears the bell (526-539).
  - `crates/gpui/src/app/context.rs:568` (`on_focus_in`), `crates/gpui_linux/src/linux/system_notifications.rs`
    (`notify-rust`, one thread per banner; `dismiss` a no-op).
  - Orca (MIT): `src/main/ipc/notification-options.ts` (title 80, body 180, `Using <tool>:
    <input>`, "a still-working agent must never be announced as finished"),
    `src/shared/notification-burst-cooldown.ts` (5,000 ms, at most 50 keys),
    `src/main/notifications/notification-delivery-service.ts` (dedupe "by worktree, not source"),
    `src/renderer/src/attention/agent-attention-acknowledgement.ts` ("compare stateStartedAt (not
    updatedAt): same-state pings must not re-trigger an ack").
- **Decisions:** D1 to D7 in the spec.

### Design
- **Approach.** `notifications.rs` keeps one `Global`, `Attention`, holding per terminal view the
  mark (the seat state that raised it) and the last state seen, and per project the time of its
  last banner. `agent_events` (#519) calls it with each seat change and the frame that caused it;
  one pure decision says whether the change marks and whether it may show a banner, and the gpui
  side acts: set the mark, post the banner, notify the rail. `init`'s `observe_new` for `TerminalView` also registers `on_focus_in`
  on the view's focus handle; when the window is active, focus in acknowledges the current state
  and clears the mark. The view's release drops its entry.
- **The pure part.** A small module of `marley_agent` (gpui-free): `event_line` for the title;
  `event_body(event, max)` for the body, with the whitespace collapse and the cut on a character
  boundary; `AttentionState` with `on_event` and `on_seen` returning what to do. Kept pure so the
  decisions sit in one place the scenario exercises through the real app.
- **The rail.** `terminal_snapshot` sets `bell` to `has_bell() || marked(view)`, so the existing dot
  shows the mark and `has_attention` counts it. Clearing the mark calls the rail's refresh, as the
  bell's clear does today.
- **The plugin.** `hooks.json` loses `notify.sh`'s three entries and the script is removed from
  `claude_plugin::FILES`; `event.py`'s preview picks `questions[0].question` for
  `AskUserQuestion`; `plugin.json` and `marketplace.json` move to the next version, and #519's chip
  offers the update to an installed copy.
- **File manifest.** `crates/marley_agent/src/marley_agent.rs` (Marley crate: `event_line`,
  `event_body`, the attention state); `crates/marley_workbench/src/notifications.rs` (Marley crate);
  `crates/marley_workbench/src/rail.rs` (Marley crate: the mark in the snapshot);
  `crates/marley_workbench/src/claude_plugin.rs` (`FILES` without `notify.sh`); the plugin's
  `hooks/hooks.json`, `hooks/event.py`, `.claude-plugin/plugin.json` and the marketplace's
  `marketplace.json`, with `hooks/notify.sh` removed; `script/e2e/538-notifications-with-content.sh`
  and its fakes (Test). No Zed path, so no touchpoint row.
- **Ledger rows.** None required up front. At Complete: a lesson on running Marley on a private
  session bus for banner tests, if the helper proves itself.

### E2E plan
Shared fixtures: two scratch repositories (`repo1`, `repo2`) opened into one window through the
rail's Add Project, whose Open Local Folders dispatches `workspace::Open { create_new_window:
Some(false) }` (`crates/recent_projects/src/sidebar_recent_projects.rs:392`), with Zed's own path
prompt (`use_system_path_prompts` false in the profile copy, L-claude-479); a private session bus
(`dbus-daemon --session --fork --print-address`, exported before the launch; `dbus-daemon`,
`python-gobject` and `jq` are installed) with a fake `org.freedesktop.Notifications` that logs each
`Notify` call's summary and body with a timestamp and returns an id; a fake `claude` started as
#481's scenario starts one, which acts out Claude Code for the event its trigger file names: it
runs every hook the plugin's real `hooks.json` lists for that event, a recorded payload on stdin,
and writes each answer's `terminalSequence` to its terminal, as #519's scenario fake does.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | Terminal A in `repo1`, B focused: A's `Stop` with a 300-character message | `538-01-marked`; the log's summary `repo1: Claude finished` |
| REQ-005 | The same banner | The log's body: 179 characters and `…`, cut on the boundary of the multibyte character placed at 178 |
| REQ-006 | Two seconds later, D's permission request in `repo1`; C's `Stop` in `repo2` | `538-02-held-back`: D's dot on; the log has C's banner and no second `repo1` banner |
| REQ-010 | The same part | The log: `repo1` one banner, `repo2` one banner |
| REQ-007 | Focus A (the rail's keyboard, Enter on A's row) | `538-03-seen`: A's dot gone |
| REQ-008 | Focus B again; a manual `PostCompact` from A, which leaves its seat idle | `538-04-repeat-ping`: no dot on A; nothing new in the log |
| REQ-009, REQ-002 | After the cooldown, A's permission request (`Bash`, `npm test`) | `538-05-new-state`: A's dot on; the log's body `Using Bash: npm test` |
| REQ-003 | A's question (`AskUserQuestion` with one question) | The log's body is the question's text, from `event.py`'s new preview |
| REQ-004 | A's `StopFailure` (`rate_limit`) | The log: `repo1: Claude failed`, body `rate limit` |
| REQ-011 | C's `SessionStart` (`startup`, then `clear`) | No log entry; no dot on C |
| REQ-012 | B's `Stop` while B holds focus | `538-06-focused`: no dot on B; no log entry |
| REQ-013 | A's `Stop` and a permission request, the fake running every hook `hooks.json` lists | One log entry for each |

Not reachable: the look of the banner itself (Quickshell draws it, and the run keeps it off Chad's
screen on purpose). The log carries the exact text the banner would show.

### Risks
- **#519's fields.** The body reads `message`, `tool`, `preview` and `error` from #519's frame, and
  the question's text from this ticket's change to its preview; if #519 renames a field, one pure
  function maps it, and a missing field leaves the body to the event's words.
- **Focus in the hidden or headless case.** `on_focus_in` fires on focus changes inside the window;
  whether it fires when the window itself becomes active with the terminal already focused needs a
  check at Code (the acknowledgment may also need the window's activation).
- **The plugin update path.** An installed plugin keeps its old hooks until updated through
  #519's chip; until then `notify.sh`'s sentence still fires beside the frame, and the per-project
  cooldown folds the two into one banner within 5 seconds.
- **Long messages.** `last_assistant_message` can be long markdown. #519 cuts it to 300
  characters and keeps its frame under the scanner's 4 KiB cap (`MAX_NOTIFICATION`,
  `crates/marley_dcs/src/notification.rs:11`, which drops a longer escape whole); this ticket
  collapses and cuts the body to 180 for the banner.
