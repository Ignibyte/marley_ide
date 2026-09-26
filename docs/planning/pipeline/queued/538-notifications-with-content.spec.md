---
pipeline_id: af192d45-78ad-4610-98cc-46cf029709aa
ticket: docs/planning/tickets/open/TICKET-538-notifications-with-content.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Notifications that say what happened"
type: feature
slice: prong 1 T7b's follow-on with prong 2's attention (report 01 §3 item 4)
references: [docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md]
---

## Title
Marley's banner for a Claude Code event in a terminal says what happened: the title
`<project>: Claude finished` (or `needs input`, `failed`), and the body the turn's last assistant
message cut short, else `Using <tool>: <input>`. An event marks its terminal unread even when no
banner shows; focusing the terminal clears the mark; a repeated ping of the same state does not
mark it again; and a burst of events from one project makes one banner.

## Scope
### In
- `crates/marley_workbench/src/notifications.rs`: a banner for each change of a seat that #519's
  `marley_workbench::agent_events` keeps per terminal, when the seat starts waiting (a permission
  request, or a question through `AskUserQuestion`), when a `Stop` ends a working turn, and when a
  `StopFailure` arrives:
  - the title from `marley_agent::event_line(project, kind, event)` (new, pure, beside #519's
    `claude_events`; #535's push line is the same string), `<project>` the terminal's workspace
    root's last component;
  - the body, from the fields of #519's frame: a finish's `message` (the turn's
    `last_assistant_message`); a permission request's `Using <tool>: <preview>` (the command for
    Bash, the path for Edit and Write, the URL for WebFetch); a question's text; a failure's
    `error` in words (`rate limit`, `overloaded`, `authentication failed`); else nothing;
  - whitespace collapsed to single spaces, and a body over 180 characters cut at 179 on a
    character boundary with `…` added (Orca's `normalizeNotificationText`).
- `hooks/event.py` (#519's hook): the preview of `AskUserQuestion` becomes its first question's
  text, cut as other previews are, since its input has no top-level string for #519's preview
  rule to pick.
- The unread mark, Marley's own per terminal (keyed by the view's entity id, dropped with the
  view): an event marks its terminal unless the terminal holds focus in the active window, whether
  or not a banner shows. The rail's row shows it with the dot it shows for a bell (`TerminalSnapshot`
  reads the bell or the mark).
- Acknowledged by viewing: when the terminal gains focus in the active window (`on_focus_in`),
  its mark clears and the seat's state is recorded as seen. Only a change of the seat's state
  (into waiting, into idle by a `Stop`, into failed) can mark again; an event that leaves the seat
  in the state already seen (a manual `PostCompact` or a subagent's `SubagentStop` after a seen
  finish) cannot.
- A burst cooldown per project: after a banner from a project, no other banner from that project
  for 5 seconds (Orca's `NOTIFICATION_COOLDOWN_MS`). It covers every Marley banner from the project,
  agent events and other programs' OSC 9 and 777 alike. Held-back events still mark.
- A session boundary (`SessionStart`, any source) shows no banner and marks nothing.
- One event, one banner: #519 keeps `notify.sh`'s fixed sentences beside its frames, so its three
  entries leave `hooks.json` (`permission_prompt` and `Stop`, whose events the frames carry, and
  `idle_prompt`, which only repeats a finish), and `notify.sh` goes with them. The plugin's version
  rises, and #519's chip offers the update.

### Out (explicitly deferred)
- What reaches the phone (#535): the push keeps to its one line, never this body.
- The bell dot that OSC 9 and 777 set through Zed's view (`terminal_view.rs:1187-1190`, a Marley
  hunk): it keeps today's behavior, with #519's exception for its own frames.
- Taking back a banner already shown: gpui's Linux `dismiss_system_notification` does nothing
  (`crates/gpui_linux/src/linux/system_notifications.rs:89-93`).
- Agent Panel threads: Zed's own `agent.notify_when_agent_waiting` covers them, and the rail's
  `thread_attention` dot stays as it is.
- A sound choice, per-source switches, Mark Unread, a notification feed.
- Agents other than Claude Code, until they report events.

## Reference (§20)
Warp: N/A beyond what #478 and #482 took; the Warp once-over rules notifications out because
Marley has them. Upstream Zed notifies for its own Agent Panel threads
(`agent.notify_when_agent_waiting`, `crates/agent_settings/src/agent_settings.rs:236`) and says
nothing for a terminal; Marley's banners are the terminal's. The behavior Marley matches is Orca's,
read from its MIT source (report 01 §2.5): the title `<repo> / <worktree> - Claude finished` (or
`needs input`) and a body of the last assistant message cut to 180 characters, else
`Using <tool>: <input>` (`src/main/ipc/notification-options.ts`); an unread marker written before
delivery, so a held-back banner still leaves it, and acknowledged by comparing the state's start
with the last acknowledged one (`src/renderer/src/attention/agent-attention-acknowledgement.ts`);
a 5-second cooldown per worktree (`src/shared/notification-burst-cooldown.ts`,
`src/main/notifications/notification-delivery-service.ts`). Marley keys the title on the project,
not on a worktree, and keeps the focused-terminal gate it has.

### Prior art
- **Behavior maps.** Report 01 §2.5 (triggers, policy, delivery, text) and §3 item 4; report 05
  §2.10 (unread in Orca's sidebar) and §3 item 4 (acknowledge per turn).
- **Published material.** Claude Code's hooks reference (code.claude.com/docs/en/hooks, read
  2026-09-25): every hook receives `session_id`, `prompt_id`, `transcript_path` and `cwd`; `Stop`
  carries `last_assistant_message`; `StopFailure`'s matcher is the error kind (`rate_limit`,
  `overloaded`, `authentication_failed`, `server_error` and more); `SessionStart`'s sources are
  `startup`, `resume`, `clear`, `compact` and `fork`.
- **Code we already ship.** `crates/marley_workbench/src/notifications.rs` (the gate at line 56,
  the per-terminal tag, the click that shows the terminal); `crates/terminal_view/src/terminal_view.rs`
  (`has_bell` 440, `clear_bell` 544, the notification hunk 1187, the key that clears the bell 1329,
  `marley_workspace` 905); `crates/marley_rail/src/marley_rail.rs:53` (`TerminalSnapshot::bell`) and
  its dot in `crates/marley_workbench/src/rail.rs:1295-1311`; gpui's `on_focus_in`
  (`crates/gpui/src/app/context.rs:568`); gpui's Linux banners through `notify-rust`
  (`crates/gpui_linux/src/linux/system_notifications.rs:38-87`). No crate we build owns a banner
  cooldown or an unread model for terminals; the rail's `thread_attention`
  (`crates/marley_rail/src/marley_rail.rs:140`) is the same idea for threads, lit when a run ends
  unseen and cleared when the thread is shown.

## UI proof
UI-AFFECTING (the banner's words, and the rail's dot on a terminal row).
`script/e2e/538-notifications-with-content.sh` (`compositor sway`: the focus cases need Marley's
window active). Marley runs on a private D-Bus session bus whose fake notification server logs each
banner's summary and body, so nothing reaches Chad's desktop; the banner's text is proved by that
log and the marks by the shots. Fixtures: two scratch repositories as two projects; a fake `claude`
(L-claude-480) in terminals A, B and D of `repo1` and C of `repo2`, each printing the #519 frame a
trigger file names, with fixed `prompt_id`s; B holds focus unless a step moves it. Shots:
`538-01-marked` (A's finish: A's row dot on), `538-02-held-back` (D's permission request two
seconds after A's banner: no second `repo1` banner in the log, D's dot on; C's event in `repo2` in
the same seconds has its own banner), `538-03-seen` (A focused: its dot gone),
`538-04-repeat-ping` (a manual `PostCompact` from A after its seen finish, while B holds focus: no
dot, no banner), `538-05-new-state` (A's permission request in a new turn: dot and banner),
`538-06-focused` (an event in the focused terminal: no dot, no banner).

## Locked-In Decisions
- D1 — The words come from #519's event, never from the terminal's screen: no transcript read, no
  scraping.
- D2 — Title `<project>: Claude <event>`, one function shared with #535, so the desktop and the
  phone name an event the same way. `<project>` is the workspace root's folder name, not the
  rail's disambiguated label, so it reads the same outside the window.
- D3 — Body order: last message, then `Using <tool>: <input>`, then the question, then the
  failure's kind; 180 characters at most, Orca's limit, which fits a banner's two or three lines.
- D4 — "Seen" is the terminal holding focus in the active window, the test the banner gate uses
  today (`notifications.rs:56`). A terminal visible in another pane but not focused is not seen.
- D5 — The mark belongs to the state that raised it: a permission request, a question, a finish or
  a failure starts one, and seeing it acknowledges it. A later ping of the same state marks
  nothing; a new state marks again, so a second permission request in one turn still lights the
  row.
- D6 — The cooldown is per project and 5 seconds, over every banner Marley shows for the project.
  A finish and a build script's OSC 9 in the same seconds make one banner.
- D7 — The mark is Marley's own, kept in the workbench and read by the rail, so no Zed crate
  changes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Claude Code's turn ends in a terminal that is not focused in the active window, the system shall show a banner titled `<project>: Claude finished` whose body is the turn's last assistant message. | The private bus's log; shot `538-01-marked` |
| REQ-002 | WHEN Claude Code asks permission for a tool in such a terminal, the system shall show a banner titled `<project>: Claude needs input` whose body is `Using <tool>: <input>`. | The log; shot `538-05-new-state` |
| REQ-003 | WHEN Claude Code asks a question in such a terminal, the system shall show a banner titled `<project>: Claude needs input` whose body is the question's text. | The log |
| REQ-004 | WHEN Claude Code's turn fails in such a terminal, the system shall show a banner titled `<project>: Claude failed` whose body names the failure's kind. | The log |
| REQ-005 | WHEN a banner's body would pass 180 characters, the system shall cut it to 179 on a character boundary and end it with `…`. | The log, for a fixture message of 300 characters with a multibyte character near the cut |
| REQ-006 | WHEN an agent event arrives for a terminal that is not focused in the active window, the system shall mark the terminal unread, whether or not a banner shows. | Shots `538-01-marked`, `538-02-held-back` |
| REQ-007 | WHEN a terminal marked unread gains focus in the active window, the system shall clear its mark. | Shot `538-03-seen` |
| REQ-008 | WHEN an event leaves a seat in the state already seen (a manual `PostCompact` after a seen finish), the system shall show no banner and mark nothing. | Shot `538-04-repeat-ping`; the log |
| REQ-009 | WHEN an event starts a new state in a terminal whose earlier state was seen, the system shall mark it again. | Shot `538-05-new-state` |
| REQ-010 | WHILE a banner from a project is less than 5 seconds old, the system shall show no other banner from that project. | The log: one banner for two `repo1` events; `repo2`'s event still has its own |
| REQ-011 | WHEN a session starts, resumes, clears or compacts, the system shall show no banner and mark nothing. | The log; the rail in shot `538-06-focused` |
| REQ-012 | WHILE a terminal holds focus in the active window, the system shall show no banner and no mark for its events. | Shot `538-06-focused`; the log |
| REQ-013 | WHEN Claude Code finishes a turn or asks for a permission, the system shall show one banner for it, and the plugin shall send no fixed sentence beside the frame. | The log: one entry per event, with the fake running every hook the plugin's `hooks.json` lists |

## Phase Plan
- **P1 Plan** — this spec, and the design and the test plan in the notes; `brain_ask` at
  promotion.
- **P2 Code** — `event_line` in `marley_agent`; the text, the marks, the acknowledgment and the
  cooldown in `notifications.rs`; the rail reading the mark; the plugin's fixed sentence and its
  version; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the three-prong
  plan's T7 row, ledger capture, close the ticket, archive, commit.
