# Settings changes the user accepts — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-682-settings-changes-accepted-as-a-diff.md
- **Pipeline spec:** 682-settings-changes-accepted.spec.md

## Phase 1 — Plan
- **Request:** the queue's top, phase 1 item 4 of the Marley-agent plan, on Chad's "just have it
  go" (2026-10-07). Scoped at planning to the user's settings: `keymap_change` became TICKET-686
  (Zed's keymap updater is a different path), a project's settings file a later ticket.
- **Classification / tier:** feature, prong 2 C; Marley crates only.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (no active pipeline; cargo busy with
  #681's install, which planning does not need) · recall ✓ · mint ✓ · prior art ✓ · spec ✓ ·
  design ✓.
- **Recall (§18.3):**
  - #681: `settings_tools` resolves a key path in the schema (`describe`, `resolve`) and hides
    secret-named values (`hidden`); the first is split out for this ticket.
  - `PR-claude-681-a-settings-value-in-effect-counts-the-project-files-001`: the answer names the
    user's file as the one written, not "the value in effect".
  - #556's and #525's write tools wait on the user with a deadline (25 s there) and answer what
    happened; #591's folder trust asks through a two-button app notification.
  - `L-claude-681-a-new-scenario-runs-in-its-own-sway-while-chad-uses-marley-001`: the scenario runs
    under sway.
- **Decisions:** D1 to D5 in the spec.

### Design

**Approach.**

1. **`marley_mcp`.** A `Family::Settings` row `change`, `Tier::Write`, grant class
   `settings.write`; `settings_change_schemas`: `key` and `value` in; `result`, `key`, `file`,
   `before`, `after` out. `INSTRUCTIONS`: "settings_change proposes a value for one of the user's
   settings; Marley asks the user, and writes it only when they accept."
2. **`mcp.rs`.** `settings.write` joins the granted classes in `start`; `settings_change` goes to
   `settings_change::answer` (the `settings_` branch sends it on).
3. **`settings_tools.rs`.** `find_setting(schema, key) -> Result<&Value, Refusal>` split out of
   `describe` (the walk and its `no_setting` refusal); `describe` calls it. `schema_params` builds
   the owned inputs to `json_schema` once for both callers.
4. **`settings_change.rs`** (Marley crate, new).
   - `answer(call, cx)`: reads `key` and `value` (`bad_argument` when either is missing or the
     value is `null`), the schema inputs, the `Fs` and the agent's name, then a foreground task:
   - On the background executor: the schema, `find_setting`.
   - `fs.load(paths::settings_file())` (a missing file reads as `{}`), parsed leniently
     (`serde_json_lenient`) into a `Value`; the new root is the old with the key path set
     (intermediate objects made); `update_value_in_json_text(&mut text, &mut vec![], 2, &old_root,
     &new_root, &mut edits)`.
   - `UserSettingsContent::parse_json` on the old text and the new: a new `Failed` the old did
     not have refuses `invalid_value` with its error. An unchanged text answers `unchanged`.
   - The question: `show_app_notification` with a `NotificationId` per request; a
     `MessageNotification::new_from_builder` with the headline ("<agent> wants to change your
     settings"), the key, before → after (compact JSON, cut at 200 characters), the file;
     `primary_message("Apply")` and `secondary_message("Decline")` send on a `oneshot`.
     A 60-second timer races it; on the timer the notification is dismissed.
   - Apply: the file read again; changed since → `changed`; else `fs.atomic_write`.
   - The answer through `call.answer`.
5. **The stand-in** needs nothing new: `tool settings_change {...}`, run in the background while
   the scenario clicks.

**File manifest.**

| File | Crate | Change |
|---|---|---|
| `crates/marley_mcp/src/registry.rs` | Marley | the row, its schemas, the test's list |
| `crates/marley_mcp/src/dispatch.rs` | Marley | the instructions' sentence |
| `crates/marley_workbench/src/settings_change.rs` | Marley | new: the check, the question, the write |
| `crates/marley_workbench/src/settings_tools.rs` | Marley | `find_setting`, `schema_params` |
| `crates/marley_workbench/src/mcp.rs` | Marley | the grant class; the branch |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley | the module |
| `script/e2e/682-settings-changes-accepted.sh` | script | the scenario |

### Visual check plan

| REQ | Setup and action | Shot and evidence |
|---|---|---|
| 001 | the copy's settings open with `// kept by #682`; `tool settings_change {"key": "terminal.font_size", "value": 19}` in the background | `682-01-card`: the notification, the agent "Stand-in agent", `terminal.font_size`, before → 19, the file |
| 002, 003 | click Apply | the answer `applied`; the file holds `"font_size": 19` and the comment; `682-02-applied`: the terminal's prompt larger |
| 004 | propose 30, click Decline | the answer `declined`; the file still 19; `682-03-declined` |
| 005, 006 | propose `"big"`; propose `terminal.no_such_key` | `invalid_value` with Zed's error; `no_setting`; `682-04-refused`, no notification |
| 007 | propose 21, no click, 62 seconds | the answer `no_answer`; `682-05-no-answer`, no notification; the file still 19 |
| 008 | `instructions` | names `settings_change`, ≤ 2,048 bytes |

The buttons' positions come from the first run's `682-01-card`.

### Risks
- **The notification's buttons in a headless sway**: the first run's shot places them.
- **A parse that does not catch a type:** `parse_json`'s leniency records a bad field as an error,
  which is what D4 reads; if a field's type is lenient (a string where a number goes), the check
  passes and Zed's own error shows later. The scenario's `"big"` for `font_size` shows which.
- **The settings watcher** may take a moment; the shot after Apply waits two seconds.

## Phase 2 — Code
- **Built:**
  - `marley_mcp`: the `settings_change` row (write, `settings.write`), `settings_change_schemas`,
    the instructions' sentence, the registry test (39).
  - `marley_workbench/src/settings_change.rs`: `answer` (the arguments, the schema inputs, the
    `Fs`, the client's name), `propose` (the schema check on the background executor, `load`,
    `edit` through `settings::update_value_in_json_text` over the old and new roots, `parse_error`
    on old and new, the question, a 25-second race, the reread before `atomic_write`), `set_at`,
    `shown`, `ask` (`show_app_notification`, a `MessageNotification` with Apply and Decline),
    `give`.
  - `settings_tools.rs`: `SchemaInputs` (`of`, `schema`) and `find_setting` split out;
    `key_argument`, `hidden` and `value_at` shared; `answer` routes `settings_change`.
  - `mcp.rs`: `settings.write` granted at start.
  - The scenario.
- **Deviations:**
  - The answer window is 25 seconds, not 60: the server stops waiting for the app's answer after
    30 (`APP_CALL_TIMEOUT_SECONDS`), so a 60-second question could only end as `timed_out`.
    `terminal_type` waits 25 for the same reason. Caught before the first run; spec, registry
    and scenario follow.
  - The answer slot is an `Arc<Mutex<…>>`: `show_app_notification`'s builder must be `Send` and
    `Sync`.
- **Review of the diff:** an absent settings file reads as `{}` and an unreadable one refuses
  rather than being overwritten; `set_at` refuses a key under a non-object; the old file's own
  parse error does not count against the change (D4); the write rereads the file first (D3).
  `propose` takes `&AsyncApp` (clippy's `needless_pass_by_ref_mut`). Nothing is read during an
  entity's update.
- **Process note:** the 682 sources were written while #681's release install compiled; its
  `marley_workbench` compile had begun before the first edit, and the installed binary holds no
  682 string ("wants to change your settings" absent), so the install is #681's code.
- **Checks before the gate:** clippy clean after one lint; the scenario's first run placed Apply
  and Decline (958, 901) and (1030, 901); its second run green, 11 of 11.
- **Gate:** `just gate-diff` green, 17 of 17 (`scratchpad/682-gate-1.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/682-settings-changes-accepted.sh` (`compositor sway`). Run 3, the Test
  phase's: 10 of 10 checks (`scratchpad/682-e2e-3.log`); run 1 placed the buttons, run 2 was the
  first green.
- **What the stand-in got:** Apply answered `{"result": "applied", "key": "terminal.font_size",
  "file": "<the run's settings>", "before": null, "after": 19}`, and the file held 19 and its
  `// kept by #682`. Decline answered `declined`, the file still 19. `"big"` answered
  `invalid_value` with Zed's error "invalid type: string \"big\", expected f32"; `terminal.no_such_key`
  answered `no_setting` with terminal's keys. A question left alone answered `no_answer` after 25
  seconds, the file still 19. The instructions name `settings_change`.
- **Shots** (`scratchpad/shots-682/`, Marley only):
  - `682-01-card` (REQ-001): bottom right, "Stand-in agent wants to change your settings",
    "terminal.font_size: not set → 19", the run's settings path, Apply and Decline.
  - `682-02-applied` (REQ-002, REQ-003): the question gone; the terminal's `$` and cursor drawn
    larger than in `01`, the new size applied without a restart.
  - `682-03-declined` (REQ-004): no question, the terminal at the size 19 gave.
  - `682-04-refused` (REQ-005, REQ-006): no question after the two refusals.
  - `682-05-waiting`: the third question, "terminal.font_size: 19 → 21", the value now read
    from the file Apply wrote.
  - `682-06-no-answer` (REQ-007): the question gone after the 25 seconds, the size still 19's.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #682); `docs/marley/guide.md` (the write tool's table) and
  the in-app guide's row; `docs/marley_architecture/marley_mcp.md` (the row and its grant class)
  and `marley_workbench.md` (`settings_change.rs`); the plan doc marks item 4's first half done and
  points at TICKET-686.
- **Knowledge (§19):** `L-claude-682-a-tool-that-waits-for-the-user-answers-inside-the-servers-30-seconds-001`,
  `L-claude-682-no-source-edit-while-an-install-compiles-001` (lessons);
  `AD-claude-682-an-agents-settings-change-waits-for-the-users-apply-001` (decisions). No `F-`
  block: the 25-second wait was caught in design, before the first run, and the `Send` bound by
  the compiler. Brain: consultation 20a02345d1f1437c9c7e4d50710a56a6 closed with `brain decide`
  (`decisions/an-agents-settings-change-waits-for-the-users-apply`, follow-up by 2026-10-28).
- **Ticket:** TICKET-682 closed; TICKET-686 (`keymap_change`) queued after TICKET-684.
- **Gate:** the in-app guide changed, so the gate ran again before the commit.
