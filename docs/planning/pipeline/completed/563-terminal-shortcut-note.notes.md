# A note the first time Marley takes a key a terminal program would have received — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-563-terminal-shortcut-note.md
- **Pipeline spec:** 563-terminal-shortcut-note.spec.md

## Phase 1 — Plan
- **Request:** the Orca second pass of 2026-09-25, the five smaller details, item 4: Orca's
  "Terminal shortcut handled" toast names the action and the keys, once per action; "Marley takes
  Ctrl-G for rich input whenever an agent runs, on purpose; the note would say so once." Chad
  decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Classification / tier:** feature, S. Marley crate only (`marley_workbench`); no Zed touch;
  no new dependency (`db` is already one).
- **Recall (§18.3):**
  - F-claude-481 and L-claude-481: keys inside the terminal view must not be swallowed on Linux;
    the rich input's container stops what the terminal would send its program. This ticket adds
    no key handling, only a call after a handler has already taken its key.
  - The lesson at `lessons.md:2594`: "#481's rich input solves the same problem the other way
    round, stopping the keys at the …": the rich input is the case the note explains. #481's
    spec (line 43) records that Claude Code's own Ctrl-G opens the prompt in `$VISUAL` or
    `$EDITOR`, the binding Marley's takes; its notes (25) that Zed's Linux keymap binds `ctrl-g`
    in editor and sidebar contexts, not in the terminal.
  - PR (prevention-rules.md:312): a one-shot flag consumed by an action is cleared on every exit
    path. This flag is set, never consumed; the only path that matters is "written after the
    toast", and a lost write costs one more note.
  - AD-claude-477: everything under a terminal goes through the footer; the note is a workspace
    toast, not a footer row, so no PTY resize.
  - The ledgers hold nothing on shortcut notes (grepped `shortcut`, `ctrl-g`, `rich input`,
    2026-09-26). Brain: not consulted in this drafting pass; promotion runs `brain_ask`.
- **Discovery (opened and checked, 2026-09-26):**
  - `crates/marley_workbench/keymap.json`: `ctrl-alt-n` → `marley::NewAgent` in `Workspace` (9);
    `ctrl-up` → `PreviousBlock`, `ctrl-down` → `NextBlock`, `ctrl-g` → `RichInput`, `right` →
    `AcceptSuggestion` in `Terminal` (19-22); the rich input's own context (29-31). The file loads
    as a Default source after Zed's defaults and before the user's keymap (`marley_workbench.rs:317-343`,
    `crates/zed/src/zed.rs:2378-2380`), so the user's keymap wins.
  - `crates/marley_workbench/src/rich_input.rs:41-51`: `focused_terminal` (`blocks.rs:63-85`) and
    `agent_in` (`agent_bar.rs:128-134`, by the foreground command's name:
    `marley_agent::agent_kind_of`, `marley_agent.rs:49-56`, `claude`, `codex`, `gemini`,
    `opencode`); `open` at 47 (57-70); `cx.propagate()` at 49.
  - `crates/marley_workbench/src/blocks.rs:17-24`: the two handlers through
    `register_action_renderer`, no `propagate`, so the key ends there (`window.rs:6342`).
  - `crates/marley_workbench/src/agents.rs:233-247`: `NewAgent`; AI off returns at 239-241.
  - `crates/marley_workbench/src/autosuggest.rs:36-49`: `AcceptSuggestion`, propagate at 47.
  - `crates/marley_workbench/src/voice.rs`: `ToggleDictation` has no key (the palette and the
    microphone button), so nothing to note.
  - `crates/gpui/src/window.rs`: `dispatch_key_event` (5810-5963), `dispatch_action_on_node`
    (6254-6378), `finish_dispatch_key_event` (6003-6021), `bindings_for_action` (6538).
  - `crates/terminal_view/src/terminal_view.rs`: `key_context` (1422), `key_down` (1332-1346),
    `process_keystroke` (1317-1330), `SendText` and `SendKeystroke` (84-92, 1010-1024).
  - `crates/terminal/src/terminal.rs:2564-2581` (`try_keystroke`); `mappings/keys.rs:120`
    (`ctrl-g` → `\x07`), 175-178 (`ctrl-up`), 214-230 (`ctrl-alt-n`).
  - `crates/ui/src/components/keybinding.rs:647` (`text_for_action`), 684 (`text_for_keystroke`).
  - `crates/zed_actions/src/lib.rs:57-58` (`OpenKeymapFile`).
  - `crates/workspace/src/notifications.rs`: `NotificationId` (41-61), `show_toast` (196-221),
    `dismiss_toast` (223), the suppression (984-1031, per workspace, in memory);
    `workspace.rs:729-759` (`Toast`, `new`, `on_click`, `autohide`).
  - `crates/marley_workbench/src/marley_workbench.rs:421-437`: the layout-presets toast, the
    pattern (`NotificationId::unique::<LayoutPresets>()`, `on_click("Use Zed's Layout", ..)`).
  - `crates/db/src/db.rs:60-67, 162-168, 247-293` (the file, `static_connection!`, `global`,
    `write_and_log`); `crates/db/src/kvp.rs`: the tables (25, 31), `read_kvp` (66-70),
    `write_kvp` (72-75), `scoped` (89-95) with `read`, `write`, `delete` (103-145),
    `Dismissable` (43-63). Zed's uses: `crates/auto_update_ui/src/auto_update_ui.rs:203-207, 217,
    245`; `crates/zed/src/zed/move_to_applications.rs:18-25, 115-117`.
  - `crates/marley_workbench/Cargo.toml:17` (`db`), 50 (`workspace`), 51 (`zed_actions`).
  - Orca (MIT, read at `1c2cf120e3`): the file and lines in the spec's prior art.
- **Decisions:** D1 to D5 in the spec.

- **Recall at promotion (2026-09-29):** the handlers as they stand: `rich_input.rs:41` (opens or
  propagates), `blocks.rs:62-66` (`step`, which propagates on the alternate screen and with no
  focused terminal), `agents.rs:388` (`new_agent`), `terminal_drive.rs:145` (`TakeOverTerminal`,
  which propagates when no agent typed or ran there). Workspace handlers run while the workspace
  is leased, so the note's toast defers (L on `window.defer`). The key-value store is
  `db::kvp::KeyValueStore::global(cx).scoped(..)`, read synchronously and written by a future, as
  `launch.rs:337` does. `ui::text_for_action` names the highest-precedence binding. #494's
  scenario relaunches Marley with `quit_marley` and `launch_marley`. Brain (consultation
  d3d02ae6): nothing on this seam.

### Design (at promotion)
- `marley_workbench::shortcut_note` (new): `taken(action, did, workspace, window, cx)`: the
  action's name (`Action::name`), skipped when this session showed it (a `Shown` global) or the
  store's `marley-shortcut-note` scope holds it; otherwise the key as bound now
  (`ui::text_for_action`), a toast deferred into the workspace ("<key> <did>; the program in this
  terminal did not get the key." with Open Keymap, `zed_actions::OpenKeymapFile`), and the name
  written to the store off the main thread.
- The call sites: `rich_input.rs` (the open arm), `blocks.rs` (`step` once it acts), `agents.rs`
  (`new_agent` while a terminal has the focus), `terminal_drive.rs` (the take-over when it
  toggles). No Zed crate.

### Design (as drafted; the promotion's above wins where they differ)
- **Approach.**
  - *`shortcut_note.rs`* (new): `const SCOPE: &str = "marley-shortcut-note"`;
    `pub fn taken(action: &dyn Action, did: &str, workspace: &mut Workspace, window: &mut Window,
    cx: &mut Context<Workspace>)`: the action's name (`action.name()`); `KeyValueStore::global(cx)
    .scoped(SCOPE).read(name)` (synchronous, a lookup in SQLite; the lint allows it); if
    recorded, return. Else the key text: `ui::text_for_action(action, window, cx)` (an unbound
    action reads "unbound", and the note still fires, since the key was taken by some binding);
    the message `format!("{key} {did}; the program in this terminal did not get the key.")`,
    where `did` is the handler's phrase ("opened Marley's Rich Input", "moved to the previous
    block", "moved to the next block", "opened New Agent"); `Toast::new(NotificationId::composite::
    <ShortcutNote>(name), message).on_click("Open Keymap", |window, cx|
    window.dispatch_action(zed_actions::OpenKeymapFile.boxed_clone(), cx))`; `workspace.show_toast`;
    then `cx.background_spawn(store.scoped(SCOPE).write(name, "1"))` detached with `log_err`.
  - *Call sites:* `rich_input.rs:47`'s arm, before `open`; `blocks.rs:17-24`'s two handlers, when
    `focused_terminal` is `Some`; `agents.rs`'s `NewAgent` handler, when `focused_terminal` is
    `Some`, before the AI check (the key is consumed either way).
- **File manifest.** Marley crate: `crates/marley_workbench/src/shortcut_note.rs` (new),
  `rich_input.rs`, `blocks.rs`, `agents.rs`, `marley_workbench.rs` (the module). Test phase:
  `script/e2e/563-terminal-shortcut-note.sh`.
- **Ledger rows.** None: no path outside the Marley-owned set changes.
- **Knowledge at Complete (expected).** An AD for D1 and D2 (the store, the handler as the seat).

### E2E plan
Setup is #481's (`script/e2e/481-rich-input.sh:18-30`): the HOME with `stand_in`, the scratch
repository. Each step settles a second; the toast draws at the workspace's bottom right.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `cat -v`, Ctrl-G, Enter; Ctrl-D | `563-01-passes`: `^G`, no toast |
| REQ-002 | `stand_in`, Enter, settle 3; Ctrl-G | `563-02-note`: the editor and the toast with Open Keymap |
| REQ-003 | Escape; Ctrl-G | `563-03-once`: the editor, no toast |
| REQ-002, REQ-005 | Escape; Ctrl-Up | `563-04-blocks`: the previous block in view and the Ctrl+Up toast |
| REQ-003 | `quit_marley`; `launch_marley`; `stand_in`, Enter, settle 3; Ctrl-G | `563-05-remembered`: the editor, no toast |
| REQ-003 | `sqlite3 "$E2E_PROFILE/db/0-dev/db.sqlite" "select key from scoped_kv_store where namespace = 'marley-shortcut-note'"` | the run log: `marley::RichInput`, `marley::PreviousBlock`, checked with `expect` |
| REQ-004 | review | the diff |

Not reachable under Hyprland: a click on Open Keymap (no pointer; the button dispatches Zed's own
action, read in the review). The `scoped_kv_store` column names are read from `kvp.rs:31` at
Test; the query is adjusted if they differ.

### Risks
- A quit right after the first note can lose the record, since the write is asynchronous; the
  cost is one more note. Awaiting the write before the toast would hold the handler.
- `NewAgent` with AI off consumes the key and does nothing; the note then reads "opened New
  Agent" for nothing visible. The handler's phrase for that branch can say "was taken for New
  Agent"; Code picks the wording at the branch.
- A user who rebinds `marley::RichInput` to a key Claude Code does not use still gets the note
  once, naming the new key; harmless.
- #481's golden scenario now shows a toast at its first Ctrl-G shot; its checks read the
  stand-in's block, so its verdict is unchanged, and Test rereads its shots once.

## Phase 2 — Code
- **Built to the promoted design.** `shortcut_note.rs` (`taken`: the `Shown` global, the key as
  bound, the store's `marley-shortcut-note` scope read on the main thread and written off it, the
  toast deferred into the workspace with Open Keymap dispatching `zed_actions::OpenKeymapFile`);
  the four call sites: `rich_input.rs`'s open arm, `blocks.rs`'s `step` once it acts,
  `agents.rs`'s `new_agent` from a terminal (after its AI-off return, so that branch never notes),
  `terminal_drive.rs`'s take-over when it toggles.
- **Found in Test, fixed here.** Rich Input's and the block keys' notes never showed, New
  Agent's did: `ui::text_for_action` reads `Window::highest_precedence_binding_for_action`, which
  matches against the rendered frame's root context stack, where a `Workspace` binding is and a
  `Terminal` one is not, so the key was not found and the note gave up. `taken` now takes the
  terminal's focus handle and reads `Window::bindings_for_action_in(action, focus)`, its own
  context stack, the last binding (the user's keymap wins). A session marks an action shown only
  once its key is found.
- **Review.** A handler that propagates says nothing (D2); the store is read on the main thread
  (a small key, as `launch.rs` reads its approvals) and written off it; the toast waits past the
  workspace's update (`window.defer`). No Zed crate.
- **Gate.** Run 1: `GATE GREEN [diff]`, 16 passed, 0 failed, the receipt written.

## Phase 3 — Test
- **Scenario** `script/e2e/563-terminal-shortcut-note.sh` under `compositor sway`: a stand-in
  agent (`exec -a claude cat -v`) that shows the keys it gets, and a relaunch with `quit_marley`
  and `launch_marley` on the same profile. Run 1 showed the Terminal-context notes missing (the
  fix above); run 2 showed every one.
- **Every shot read** (run 2):
  - `563-01-passes`: `cat -v` shows `^G`; no toast. REQ-001.
  - `563-02-note`: Rich Input open and the toast "Ctrl-G opened Marley's Rich Input; the program in
    this terminal did not get the key." with Open Keymap. REQ-002.
  - `563-03-once`: the toast closed, Ctrl-G opened Rich Input again with no toast. REQ-003.
  - `563-04-blocks`: the block selected and its own toast "Ctrl-Up moved to the previous block;
    …". REQ-005, REQ-002.
  - `563-05-remembered`: after the relaunch, Ctrl-Up and then Ctrl-G under the agent: no toast.
    REQ-003.
  - `563-06-new-agent`: Ctrl-Alt-N from the terminal: "Ctrl-Alt-N opened Marley's New Agent
    picker; …". REQ-005.
- **Review, not a shot:** Open Keymap dispatches `zed_actions::OpenKeymapFile` (REQ-004).
- The #481 golden scenario's first Ctrl-G shot would now carry the toast; no regression runs in
  this workflow, and its checks read the stand-in's block, not the toast.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added; `marley_workbench.md` (a section for `shortcut_note.rs`). No Zed
  path changed.
- **Knowledge.** F-claude-563-text-for-action-misses-a-terminal-binding-001,
  PR-claude-name-a-context-bound-key-through-its-focus-handle-001. Brain: `brain_decide` on
  consultation d3d02ae6.
- **Closed** the ticket, archived the pair, committed.
