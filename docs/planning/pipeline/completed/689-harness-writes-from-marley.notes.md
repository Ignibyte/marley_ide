# Harness writes from Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-689-harness-writes-from-marley.md
- **Pipeline spec:** 689-harness-writes-from-marley.spec.md

## Phase 1 — Plan
- **Request:**
  - The cron Chad set on 2026-10-07: "check on the harness bot and when it finishes finish yours".
  - Item 4 of the plan's phase 2 needs nothing from the harness, so it runs first.
- **Classification:** feature. Marley crate work, and one field in Marley's own settings file.
- **Recall (§18.3):**
  - F-534: the prompt's routing note is for display only (`shown_prompt`). An answer must send the
    full prompt.
  - L-640: a module doc naming a `pub(crate)` item breaks rustdoc. Name it in plain code.
  - L-635: no new e2e helper may share a name with a scenario's own.
  - AD-534 (polling, its own section) and AD-632 (the embedded child).
  - Brain: the same seam as #534. Nothing new was asked.
- **Discovery:**
  - `harness.rs` has `call` (reads, `answer` unwraps `isError`), `HarnessView` (lines only), and
    `follow_embedded` (args with no grant).
  - `marley_fleet::verbs` types the write verbs.
  - The harness's write receipts are `{result, value|reason}`. Under the read grant, a write is a
    refused receipt.

### Design
**Approach.**
1. **The setting:**
   - `MarleySettingsContent.harness_writes: Option<bool>`, with `default.json` set to `false`.
   - `MarleySettings.harness_writes: bool`.
   - `Source::Embedded { writes }`, so a change restarts the follow. `follow_embedded` appends
     `--grant write` when it is on.
2. **`harness.rs`, the writes:**
   - `write(server, tool, arguments, cx) -> Result<Value, String>` reads the receipt: from the
     structured content, else from the text JSON, whether `isError` is set or not. Accepted gives
     the `value`, and refused gives the `reason`.
   - `Harness::server(cx)` hands the server to the tab.
   - `writes_on(cx)`: the setting is on and a harness is connected.
3. **`HarnessView`:**
   - `send_editor: Entity<Editor>` (single line).
   - `status: Option<(SharedString, Color)>`, the last write's outcome.
   - `views: Vec<(SharedString, SharedString)>`.
   - `answer(choice)`, `send()` (Enter by `menu::Confirm` on the editor, or the button), and
     `surface()`.
   - `render` adds a controls block above the lines while `writes_on`: the question row with an
     option button each, the send row, and the Views button with the list, each view with Copy
     (`cx.write_to_clipboard`).
4. **The palette:**
   - `OpenHarnessSession` (an action in `marley`) with an `OpenSessionDelegate` picker.
   - Its matches are profile names from `<data dir>/harness/profiles/*.json` while Marley runs
     the harness itself, filtered by the query, then the typed name.
   - Confirm calls `session_open {profile, request}`, then opens the returned id's tab. A refusal
     shows as an error notification.
   - `CommandPaletteFilter` lists the action only while `writes_on`.

**File manifest.**
| File | Owner | Change |
|---|---|---|
| `crates/settings_content/src/marley.rs` | Zed crate (Marley's file) | `harness_writes` |
| `assets/settings/default.json` | Zed crate | `harness_writes: false` |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley crate | `MarleySettings.harness_writes` |
| `crates/marley_workbench/src/harness.rs` | Marley crate | writes, the tab, the picker |
| `docs/marley/zed-touchpoints.md` | docs | two rows |
| `script/e2e/689-harness-writes-from-marley.sh` | Marley e2e | new |

### Visual check plan
The scenario follows #534's setup: the harness's built `rh`, a scratch root and `marley.harness`
set to `rh --state ROOT mcp --grant write`, with `marley.harness_writes` on. Its actors:
- `asker`, which waits on "Which base branch?" with main and release;
- `listener`, which waits for a message, then prints "got it";
- a 0600 profile `greeter` (an actor script).

| REQ | What the scenario does | Shot / check |
|---|---|---|
| REQ-001 | clicks the asker's row | `689-01-question`: the prompt and the main and release buttons |
| REQ-002 | clicks main | `689-02-answered`; the asker's output holds "chose it" |
| REQ-003 | opens the listener's tab, types "hello harness", presses Enter | `689-03-sent`: the receipt's state; the listener's output holds "got it" |
| REQ-004 | clicks Views | `689-04-views`: the native or tmux view's `rh … view/attach` line |
| REQ-005 | runs `open harness session` from the palette and types `greeter` | `689-05-picker`; `689-06-opened`: greeter's tab, and the rail lists it |
| REQ-006 | — | the diff: `writes_on` gates both the controls and the palette |

### Risks
- The harness binary is mid-work in its own repo, and its gate may rebuild `rh` during a run. If
  that happens, rerun.
- A send to an actor that is not waiting for a message is refused. The tab shows the harness's
  reason, which is the intended behavior.

## Phase 2 — Code
- **Built:**
  - `harness_writes` in the settings, with `harness::harness_writes` reading it from the merged
    settings.
  - `Source::Embedded { writes }`, and `follow_embedded` adds `--grant write`.
  - `request`, shared by `call` and `call_write`. `call_write` reads a receipt with or without
    `isError`.
  - `HarnessView` gained `send_editor`, `status`, `views`, `answer`, `send`, `surface`, the
    generic `write`, and `render_controls`. Lines now scroll under the controls, and
    `menu::Confirm` sends.
  - `view_line` quotes a view's arguments.
  - `OpenHarnessSession` with `open_session_picker`, `profile_names` and `OpenSessionDelegate`.
    The palette shows it through `filter_palette`.
- **Deviations:**
  - `MarleySettings` does not hold the switch: clippy's `struct_excessive_bools` and
    `too_many_lines` refused another bool there. `harness.rs` reads it from the merged settings, as
    `assistant.rs` reads its own switch.
  - The palette lists the command while the switch is on and a harness is configured, not only
    while one is connected. A press with no connection shows "the harness is not running".
- **Review:**
  - An answer sends the full prompt, routing note included (F-534).
  - A second write waits for the first (`writing`).
  - A refusal shows the harness's reason.
  - Nothing panics on a malformed receipt.
- **Gate:**
  - Runs 1 and 2 were red: clippy (a redundant closure, the bool limit, `&mut` not needed) and
    dylint (`SharedString::new_static`, an `async` block without `.await`). Each was fixed at the
    source.
  - Run 3 (`scratchpad/689-gate-3.log`): **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/689-harness-writes-from-marley.sh` (`compositor sway`, the harness's
  built `rh`).
  - Runs 1 and 2 measured the rows and controls.
  - Run 3 passed 3 of 3, and its views shot showed a flaw (below).
  - Run 4, after the fix and a green gate (`scratchpad/689-gate-5.log`), passed 3 of 3
    (`scratchpad/689-e2e-4.log`).
- **Shots** (run 4, Marley only):
  - `689-01-question` (REQ-001): the asker's tab, with "Which base branch?" and the main and
    release buttons, and under them the send line with Send and Views.
  - `689-03-sent` (REQ-003): the listener's tab. "Sent: deposited (queued)" is the receipt, and
    the lines are "listening" then "got it". The rail reads the listener as done.
  - `689-02-answered` (REQ-002): back on the asker's tab, "Answered main", the question gone and
    the line "chose it". The rail reads the asker as working, and Needs you is gone.
  - `689-04-views` (REQ-004): native and tmux, each with Copy before
    `…/rh --state <root> view|attach <workspace>`, the line cut with an ellipsis.
  - `689-05-picker` (REQ-005): the picker offers "greeter", the typed name. A `marley.harness`
    command gives no root to list.
  - `689-06-opened`: the tab greeter-640b8a71, showing "hello from greeter", and the rail lists
    greeter-640b8a71 working.
- **Fix, run 3 to run 4:** the views' command lines ran past the tab's edge and pushed Copy out of
  sight. Copy now comes before the line, and the line is truncated.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".
- **As designed:** a harness tab opens under the rail's Home group, shown as "Home 2" in the
  e2e profile, with the window on Home. #676 made screens that belong to no project open there.

## Phase 4 — Complete
- **Docs (§21):**
  - `CHANGELOG.md` (Added, #689).
  - The guide's "Writing to a session" under the harness's sessions. The in-app guide has no
    harness section.
  - `docs/marley_architecture/marley_workbench.md` (Writes, #689).
  - The plan's item 4, marked done, with harness 092's `wait_ms`. Item 6 gains the harness's
    `seat add`/`start` shape.
  - Touchpoints rows for `marley.rs` and `default.json` (written in Code).
- **Knowledge (§19):**
  - `L-claude-689-a-source-fix-in-test-needs-the-test-phase-active-001`
  - `AD-claude-689-harness-writes-live-in-the-sessions-tab-behind-one-switch-001`
  - Brain: no new consultation. The seam is #534's, and AD-689 records the decision.
- **Ticket:** TICKET-689 closed.
