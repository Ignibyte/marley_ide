---
pipeline_id: 79d0f7d3-f9bf-4442-b14d-0379a7160738
ticket: docs/planning/tickets/open/TICKET-682-settings-changes-accepted-as-a-diff.md
status: Phase 4 — Complete PASS
title: Settings changes the user accepts
type: feature
slice: prong 2 C (Marley's MCP server); phase 1 item 4 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/intake/marley-agent-manager-foreman.md
  - docs/planning/pipeline/completed/681-docs-and-settings-tools.spec.md
---

## Title
`settings_change`: an agent proposes a value for a key of the user's settings; Marley shows the
change and writes it only when the user accepts, keeping the file's comments and layout. With
#681's read tools, it is what lets the Marley agent configure Marley without a file-editing tool.

## Scope
### In
- `settings_change` (write tier, grant class `settings.write`, which Marley grants its own
  clients): `key` (a key path) and `value` (JSON).
- Before anyone is asked: the key must be in Zed's settings schema (`no_setting`, as
  `settings_schema` refuses), and the file with the change must parse as Zed parses it
  (`invalid_value`, with Zed's parse error); a change that leaves the file as it was answers
  `unchanged` and asks nothing.
- The question: a notification in Marley's windows naming the agent (the client's name), the key,
  the value now and the value proposed, and the file, with Apply and Decline. Apply writes;
  Decline answers `declined`; no answer within 25 seconds (under the 30 the server waits for
  the app's answer, as `terminal_type`'s 25) answers `no_answer` and takes the notification away.
- The write: the user's settings file as it was read, edited in place by Zed's JSON editor
  (`settings_json::update_value_in_json_text`, the path Zed's own settings writer takes), so
  comments and the other keys stay; refused with `changed` when the file changed while the user
  was asked. Zed's settings watcher applies it.
- The answer: `applied` with the key, the file, the value before and after (values under secret
  names hidden, as `settings_read` hides them).
- `INSTRUCTIONS` names `settings_change` and says the user is asked.

### Out (explicitly deferred)
- `keymap_change` (TICKET-686).
- A project's `.zed/settings.json`; removing a key (`null`); several keys in one question.
- An outside client (Browser Clients) calling it: it stays on no client list.

## Reference (§20)
Upstream Zed (the `settings`, `settings_json` and `settings_content` crates, and the workspace's
notifications): the edit is the one Zed's settings writer makes (`update_value_in_json_text` over
the file's old and new JSON), the check is Zed's own lenient parse
(`settings_content::RootUserSettings::parse_json`, whose `ParseStatus::Failed` carries the
error), and the question is a `MessageNotification` with two buttons, as #591's folder-trust
question asks.

### Prior art
- **Behavior maps:** `docs/t3code_architecture/05-terminal-browser-and-capture.md` item 6 (T3's
  config writes show the change, check the file did not move, keep a backup and rename into
  place): the check that the file did not change while the user was asked is taken from there.
- **Published material:** MCP 2025-06-18: a write tool whose effect a person approves; nothing in
  the spec carries the approval, so the tool's own answer says what happened.
- **The code we ship:** `crates/settings_json/src/settings_json.rs:15`
  (`update_value_in_json_text`, edits applied to the text in place, objects diffed key by key);
  `crates/settings/src/settings_store.rs:559` (`update_settings_file_inner`: load, new text,
  atomic write, which this follows for a key-path change); `crates/settings_content/src/
  fallible_options.rs:11` (`parse_json`, the leniency that records a bad field as an error rather
  than failing the file); `crates/marley_workbench/src/agent_trust.rs:280-300` (a two-button
  `MessageNotification` through `show_app_notification`); #681's `settings_tools` (the schema
  and its key resolution, `hidden`).

## UI proof
`script/e2e/682-settings-changes-accepted.sh` (`compositor sway`, for the buttons). The run's
copy of the user settings opens with a comment. The stand-in agent proposes
`terminal.font_size` 19: the notification (`682-01-card`); Apply, the answer `applied`, the file
holding 19 and the comment, the terminal drawn larger (`682-02-applied`). It proposes 30 and
Decline answers `declined`, the file still 19 (`682-03-declined`). A wrong type and an unknown
key are refused with no notification (`682-04-refused`). A proposal left unanswered answers
`no_answer` after 25 seconds and the notification goes (`682-06-no-answer`).

## Locked-In Decisions
- D1 — The user's settings only, one key per question.
- D2 — A notification in every window (`show_app_notification`), not a card in one view: the agent
  may run where the user is not looking.
- D3 — The file is edited as text from the file read at the start, never re-serialized, and the
  write is refused if the file changed meanwhile.
- D4 — Validation is Zed's parse of the whole new file, compared with the old file's: a change
  that adds a parse error is refused; an error the file already had is not the agent's.
- D5 — A new grant class `settings.write`, granted at start like `terminal.write`, so a future
  setting can take it away without touching the tool.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `settings_change` with a key of the schema and a value Zed parses, the system shall show a notification naming the agent, the key, the value now, the value proposed and the file, with Apply and Decline. | `682-01-card` |
| REQ-002 | WHEN the user picks Apply, the system shall write the value into the user's settings file, keep its comments and other keys, and answer `applied` with the value before and after. | The stand-in's answer; the file holds 19 and the comment; `682-02-applied` |
| REQ-003 | WHEN the change is written, the system shall apply it without a restart. | `682-02-applied`: the terminal's text larger than in `682-01-card` |
| REQ-004 | WHEN the user picks Decline, the system shall leave the file as it was and refuse with code `declined`. | The answer; the file still 19; `682-03-declined` |
| REQ-005 | WHEN the value would not parse as the setting's type, the system shall refuse with code `invalid_value` and Zed's error, and show nothing. | The answer for `"big"`; `682-04-refused` with no notification |
| REQ-006 | WHEN the key is not in the schema, the system shall refuse with code `no_setting` and show nothing. | The answer for `terminal.no_such_key` |
| REQ-007 | WHEN the user answers nothing within 25 seconds, the system shall refuse with code `no_answer` and take the notification away. | The answer; `682-05-no-answer` with no notification |
| REQ-008 | WHEN a client initializes, the instructions shall name `settings_change` and say the user is asked, staying under 2,048 bytes. | `instructions` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_mcp` (the row, its schemas, the grant class in the instructions),
  `marley_workbench` (`settings_change.rs`, the grant, the answer branch, a shared key resolver
  in `settings_tools.rs`); the scenario; a review; `just gate-diff`.
- **P3 Test** — the scenario; every shot read.
- **P4 Complete** — docs, ledger, close, archive, commit, push, install.
