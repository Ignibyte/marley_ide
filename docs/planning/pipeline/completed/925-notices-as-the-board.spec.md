---
pipeline_id: 9534c51f-6b95-4354-a964-1a5592338aae
ticket: docs/planning/tickets/open/TICKET-925-notices-as-the-board.md
status: Phase 4 — Complete PASS
title: Toasts, the plan entry and notifications as the board words them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/869-toasts.spec.md]
---

## Title
Toasts get a kind and a dot to match; the plan's Needs you entry names the plan and offers Open
the plan; a long command's desktop notification and an agent's phone push are worded as the
Inbox board words them.

## Scope
### In
- **Toast kinds** (`workspace/src/workspace.rs` and `workspace/src/notifications.rs`, Zed crate
  hunks; `toasts.rs`, Marley crate): `MarleyToastKind { Warning, Success, Error, Question,
  Neutral }` beside `MarleyToast`; `Toast` gains an optional kind with a builder
  `Toast::marley_kind`, and `show_toast` hands it to `MarleyToastLook`. `toasts::look` colours
  the dot by kind: Warning yellow (`Color::Warning`), Success green (`Color::Success`), Error red
  (`Color::Error`), Question the accent, Neutral grey (`Color::Muted`). A toast with no kind is
  read from its words: a failure word is an Error, a title ending in "?" a Question, the rest
  Neutral.
- **Kinds at Marley's call sites** (Marley crate): the toasts that warn with an undo (closing a
  working agent's terminal, `close_guard.rs:361`) are Warning; the ones that report a thing done
  (captured, saved, copied, written: `rusty/capture.rs`, `forge.rs:215` and the like) are
  Success. A toast that asks is a Question by its "?". The Code phase walks the 67 `Toast::new`
  sites in `crates/marley_*` and sets a kind where the words do not already give it.
- **The plan entry** (`plan_review.rs`, `entry_of`; `rail.rs`, `render_inbox_buttons`, Marley
  crate): `Pending` keeps the plan's title (its first heading, else its first line, cut to 60
  characters) when the plan arrives; the ask reads "Plan to review: “<title>”". A plan entry's
  button is "Open the plan", filled in the accent, in place of the quiet Open; it opens the plan's
  tab as Open does.
- **The long command's desktop notification** (`command_watch.rs`, Marley crate): the title is
  "<command> finished · exit <code>" (exit 0 for a clean end); the body is "<terminal's tab
  title> ran <time> while you were in <the active item's title>." when the window is active and
  shows another item, and "<terminal's tab title> ran <time>." otherwise. The password tell is
  unchanged.
- **The phone push** (`push.rs`, Marley crate; `marley_agent.rs`, Marley crate): ntfy's `Title`
  header is the event: "<agent> needs you" for an approval or a question, "Turn finished",
  "Turn failed". The body is "<project> · <detail>": "approval", "a question", or the agent's
  name for a finished or failed turn. A new `marley_agent::push_text(project, agent, event)`
  returns both as a struct with named fields; `event_line` stays for the agents' desktop banners
  (`notifications.rs:155`).

### Out (explicitly deferred)
- Notices Zed builds itself (`MessageNotification`s such as plugin too old, trust this folder,
  agents can act again) drawn as dot, title, body and actions: #869 Out.
- The ask itself in the push ("run cargo test?"): #535 sends nothing the agent wrote.
- The password tell's wording: the board does not draw it.

## Reference (§20)
Upstream Zed (`workspace`): toasts are Zed's `Toast` and `MessageNotification`, kept; Marley's
look hook (#869) gains a kind. The desktop notification is gpui's `show_system_notification`,
kept. The plan entry and the push are Marley-specific (#508, #535, #726), no reference analog.

### Prior art
- `toasts.rs:21` `look`: red for "Could not" or "failed", the accent otherwise;
  `workspace/src/notifications.rs:75` `MarleyToastLook` and `:80` `MarleyToast { title, line,
  dot }`; `:209` `show_toast` asks the hook with the message only; `workspace.rs:778` `Toast`
  (id, message, autohide, on_click).
- 67 `Toast::new` calls in `crates/marley_*`; two helpers, `plan_review.rs:411` and
  `send_selection.rs:671`, take a message only.
- `plan_review.rs:365` `entry_of` builds the ask "Plan to review" with no answers; `rail.rs:2574`
  draws the quiet Open for an entry with no answers; `plan_review.rs:99` `Pending` holds the
  request, not its title.
- `command_watch.rs:122–155`: body "exit N after T" or "done in T"; `notifications.rs:206`
  `notify` takes the title (the command) and the body; `looking_at` decides whether to post.
- `push.rs:173` posts `event_line` ("marley_ide: Claude needs approval"), `:255` the `Title`
  header "Marley"; `marley_agent.rs:865` `event_line` filters control characters.
- `script/e2e/535-phone-push-notifications.sh` (a fake ntfy that logs each request) and
  `551-command-end-from-outside.sh` (a private session bus whose notification server logs each
  banner): reused.
- The brain: not consulted at queueing; the promoting session asks.

## UI proof
Scenario `script/e2e/925-notices-as-the-board.sh` (sway): a scratch repository; a private
session bus with a logging notification server and a fake ntfy on a loopback port, as #535's and
#551's scenarios set them up; `marley.long_command_seconds` at 2; the stand-in Claude Code first
on the terminals' PATH through the `.bashrc`, with Marley's plugin hook and `marley.plan_review`
on.
- `925-01-toasts`: four toasts raised in turn from the palette and the stand-in (a warning, a
  success, an error, a neutral one): their dots (REQ-001).
- `925-02-plan`: the stand-in asks a plan titled "Tidy the notes": the Needs you entry reads
  "Plan to review: “Tidy the notes”" with Open the plan (REQ-002).
- `925-03-opened`: Open the plan clicked: the plan's tab in front (REQ-002).
- Banner log: `sleep 3; false` in a terminal while a file is in front: the logged title and body
  (REQ-003).
- Push log: the stand-in's turn ends while the window is away: the logged `Title` header and body
  (REQ-004).
- `925-04-day`: the toasts and the plan entry in Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The kind travels with the toast (`Toast::marley_kind`); words decide only for a toast with
  no kind, so Zed's own toasts still get a dot.
- D2 — The default dot is grey: a toast that neither warns, reports success, fails nor asks is
  neutral, as the board draws "Agents can act again".
- D3 — The plan's title is the plan's first heading, the words the user reads at the top of plan
  review; it stays in Marley, never in a push.
- D4 — The push's title is fixed words and the agent's short name, so the HTTP header stays
  ASCII; the project's name goes in the body.
- D5 — "While you were in" names the active item only when the window is active; nothing is said
  of other programs.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a toast shows in the Marley layout, its dot shall be yellow for a warning, green for a success, red for an error, the accent for a question and grey for any other. | `925-01-toasts`, `925-04-day` |
| REQ-002 | WHILE Claude Code waits on a plan, its Needs you entry shall read "Plan to review: “<title>”" and offer Open the plan in the accent, which brings the plan's tab to the front. | `925-02-plan`, `925-03-opened`, `925-04-day` |
| REQ-003 | WHEN a long command ends in a terminal the user is not looking at, the desktop notification shall be titled "<command> finished · exit <code>" and say "<terminal> ran <time> while you were in <item>." | the banner log |
| REQ-004 | WHEN an agent's event is pushed to the phone, the push shall be titled with the event and its body shall be "<project> · <detail>", with nothing the agent wrote. | the push log |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the `workspace` hunks and their touchpoint rows, `toasts.rs` and the kinds at the
  call sites, `plan_review.rs`, `rail.rs`, `command_watch.rs`, `push.rs`, `marley_agent.rs`; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot and
  both logs.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
