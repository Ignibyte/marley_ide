---
pipeline_id: 93860ed2-8626-4dd4-afc4-547c40cacc97
ticket: docs/planning/tickets/open/TICKET-563-terminal-shortcut-note.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A note the first time Marley takes a key a terminal program would have received"
type: feature
slice: prong 1 T7 (CLI agents in the terminal, after #481's rich input and #473's block keys); the Orca second pass, smaller item 4
references: [docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/planning/pipeline/completed/481-rich-input.spec.md]
---

## Title
Four of Marley's keys never reach the program in a focused terminal: Ctrl-G while an agent CLI
runs (the rich input), Ctrl-Up and Ctrl-Down (block navigation) and Ctrl-Alt-N (New Agent).
Each is on purpose, and each is a surprise the first time a program's own binding does nothing.
The first time an action takes a key from a terminal, Marley shows a toast in that workspace
naming the key as it is bound and what it did, with a button that opens the keymap, once per
action per data directory. A key that falls through to the program never shows it.

## Scope
### In
- **The actions**: `marley::RichInput` (Ctrl-G, taken while an agent CLI runs; it falls through
  otherwise), `marley::PreviousBlock` and `marley::NextBlock` (Ctrl-Up, Ctrl-Down, always taken
  in a terminal), and `marley::NewAgent` (Ctrl-Alt-N, a `Workspace` binding, taken while a
  terminal has the focus). #525's `marley::TakeOverTerminal` (Ctrl-I) joins them when it lands.
- **The note** (`crates/marley_workbench/src/shortcut_note.rs`, new): `taken(action, did,
  workspace, window, cx)`, called by each handler at the point it acts with the focus in a
  terminal. It reads the key as bound now (`window.bindings_for_action`, `ui::text_for_action`),
  so a user's rebinding shows, and shows a `Toast` in that workspace: "Ctrl+G opened Marley's
  Rich Input; the program in this terminal did not get the key." with the button Open Keymap
  (`zed_actions::OpenKeymapFile`), no autohide. Then it records the action's name in Zed's
  key-value store under the scope `marley-shortcut-note`; a recorded action shows nothing again
  on that data directory.
- **The four call sites**: `rich_input.rs` (in the arm that opens the editor, not the one that
  propagates), `blocks.rs` (the two handlers), `agents.rs` (`NewAgent`, when a terminal has the
  focus).
- `script/e2e/563-terminal-shortcut-note.sh`.

### Out (explicitly deferred)
- A setting for the note: once per action per data directory is the cap, and a shift-click on
  the toast's close suppresses it in that workspace as any Zed toast.
- Orca's "Terminal first" policy, a mode where a program's binding beats Marley's: Zed's user
  keymap already wins over Marley's, and the note says so.
- `marley::AcceptSuggestion` (Right): it acts only while a suggestion is on screen, and the
  accepted text is the point.
- Zed's own workspace keys that never reach a PTY in stock Zed (ctrl-`, ctrl-j, and the ones
  Marley reroutes in `routing.rs`): Zed's, not Marley's.
- Notes in the Browser tab or the editor: a browser page has no keys of its own to lose.

## Reference (§20)
Orca's "Terminal shortcut handled" toast (the second pass, smaller item 4; report 05 §2.13 notes
its dictation key "takes Ctrl-E from readline and Claude Code on Linux";
`src/renderer/src/lib/terminal-shortcut-capture-notification.tsx`): the action's title and its
current keys ("New terminal tab (Ctrl+T)"), a keyboard icon, an Open Shortcuts button, 20 s,
shown once per action and remembered in `localStorage`, for any action not scoped to the
terminal that Orca took while a terminal had the focus. Marley keeps the once-per-action rule,
the current keys and the button, records it in Zed's key-value store instead, and says what the
key did. Upstream Zed takes keys from terminals through its keymap and shows nothing when it
does (its Linux keymap gives some back with `SendKeystroke` overrides,
`assets/keymaps/default-linux.json:1305-1328`). Warp: N/A, its docs describe no such note.

### Prior art
- **Behavior maps and reports.** The second pass, item 4 ("Marley takes Ctrl-G for rich input
  whenever an agent runs, on purpose; the note would say so once"). Report 05 §2.13 (Ctrl-E
  taken from readline) and §2.19 (Orca's keybindings: one chord per binding, no contexts; Zed's
  contexts are ahead). Orca's file, read at `1c2cf120e3`: `showTerminalShortcutCaptureNotification`
  (43-95): the `localStorage` key `orca.terminalShortcutCapturedNotice.<actionId>` (14, 17-31),
  written before the toast shows (56-59); the title (67-71) and the description from
  `formatKeybindingList(getEffectiveKeybindingsForAction(..))` (61-64, 73; "Unassigned" when
  the action has no key); 20,000 ms (15, 76); the Open Shortcuts button (87-91); the rule `scope
  !== 'terminal' && allowInTerminal !== true` (52-55; `src/shared/keybindings/effective.ts:80-82`);
  its callers (`terminal-workspace-keydown.ts:54-63`, `app-command-handlers.ts:122-137`,
  `use-global-keybindings.ts:127-136`, and the main process's `ui:terminalShortcutCaptured` from
  `main-window-shortcut-routing.ts:64-108`); the policy `terminalShortcutPolicy`, `orca-first` by
  default (`default-global-settings.ts:161`), the only way to stop the capture and with it the
  note. No Orca doc page mentions it.
- **Published material.** Claude Code's own Ctrl-G opens the prompt in `$VISUAL` or `$EDITOR`
  (recorded in #481's spec, `481-rich-input.spec.md:43`), the binding a user reaches for and
  gets Marley's editor instead. The readline manual's command names (`bash(1)`, Readline):
  `abort (C-g)`, which is what a shell at its prompt loses to Ctrl-G; xterm's CSI modifier
  encoding (`ESC [ 1 ; 5 A` for Ctrl-Up), which is what Zed would have sent.
- **The code we already ship.** The bindings (`crates/marley_workbench/keymap.json:9, 19-22`:
  `ctrl-alt-n` in `Workspace`, `ctrl-up`, `ctrl-down`, `ctrl-g` and `right` in `Terminal`) and
  their handlers: `rich_input.rs:41-51` (`focused_terminal` and `agent_in`; `open` at 47,
  `cx.propagate()` at 49), `blocks.rs:17-24` (`PreviousBlock` and `NextBlock` through
  `register_action_renderer`, never propagating), `agents.rs:233-247` (`NewAgent`; with AI off it
  returns at 239-241 with the key consumed), `autosuggest.rs:36-49` (`AcceptSuggestion`,
  propagating at 47). gpui's dispatch order (`crates/gpui/src/window.rs:5810-5963`: bindings on
  the focused path first, a bubble-phase handler stopping propagation by default at 6342,
  `on_key_down` only after, 6003-6021), so a handled action never reaches `TerminalView::key_down`
  (`crates/terminal_view/src/terminal_view.rs:1332-1346`) or `Terminal::try_keystroke`
  (`crates/terminal/src/terminal.rs:2564-2581`), where Ctrl-G becomes `\x07`
  (`crates/terminal/src/mappings/keys.rs:120`) and Ctrl-Up `\x1b[1;5A` (175-178).
  `Window::bindings_for_action` (`window.rs:6538`) and `ui::text_for_action`
  (`crates/ui/src/components/keybinding.rs:647`) give the key as bound; `zed_actions::OpenKeymapFile`
  (`crates/zed_actions/src/lib.rs:57-58`) opens the user's keymap. `Workspace::show_toast`
  (`crates/workspace/src/notifications.rs:196-221`), `Toast::new` and `on_click`
  (`workspace.rs:738-754`), `NotificationId::composite` (notifications.rs 56), the shift-click
  suppression (984-1031); Marley's own toast with a button, the layout presets
  (`marley_workbench.rs:421-437`). `db::kvp::KeyValueStore::global` (`crates/db/src/db.rs:266-268`),
  `read_kvp` (synchronous, `kvp.rs:66-70`), `write_kvp` (72-75), `scoped` (89-95, the
  `scoped_kv_store` table at 31) and the `Dismissable` trait (43-63) that Zed's own once-only
  flags use (`auto_update_ui.rs:203-207`, `move_to_applications.rs:18-25, 115-117`); the store is
  `<data_dir>/db/0-dev/db.sqlite` (`db.rs:60-67, 162-168`), one per data directory, so the e2e
  profile copy has its own; `db` is already `marley_workbench`'s dependency (`Cargo.toml:17`,
  for `MarleyBrowserTabsDb`, `browser.rs:4907-4962`). Marley's once-only behavior today is in
  memory per process (`mcp.rs:241`). Does a crate we build own the seam? Zed's workspace owns
  the toast, `db` the record and gpui the key text; the new thing is one call in four handlers.

## UI proof
UI-AFFECTING (a toast in the workspace). `script/e2e/563-terminal-shortcut-note.sh` (Hyprland,
keys only). Setup as #481's: the scenario's HOME whose `.bashrc` defines `stand_in` (`exec -a
claude`, a shell that prints each line it reads), a scratch repository. Shots: `563-01-passes`
(`cat -v`, Ctrl-G, Enter: `^G` in the terminal, no toast); `563-02-note` (`stand_in`, Ctrl-G:
the rich input open, and the toast "Ctrl+G opened Marley's Rich Input; the program in this
terminal did not get the key." with Open Keymap); `563-03-once` (Escape, Ctrl-G again: the
editor, no toast); `563-04-blocks` (Escape; Ctrl-Up: the view at the previous block, and the
toast for Ctrl+Up); `563-05-remembered` (`quit_marley`, `launch_marley`, `stand_in`, Ctrl-G:
the editor, no toast). The run log lists the `scoped_kv_store` rows of the profile copy's
`db/0-dev/db.sqlite` under `marley-shortcut-note`, checked with `expect`: `marley::RichInput`
and `marley::PreviousBlock`.

## Locked-In Decisions
- D1: Once per action per data directory, in Zed's key-value store (Orca's `localStorage`
  twin), never per session: a note that came back at every launch would be the nuisance it
  warns about.
- D2: The note fires in the handler, where the key is taken, and never on a fall-through: a
  handler that propagates gave the key to the program, so there is nothing to say.
- D3: The text names the key as bound now and what it did, and the button opens the keymap:
  rebinding or unbinding in the user's keymap, which wins over Marley's, is how a key is given
  back (Zed's `SendKeystroke` pattern). Rejected: Orca's "Terminal first" policy, a second
  keymap layer Zed does not need.
- D4: The four actions, not every Marley binding: `AcceptSuggestion` acts only when a suggestion
  shows, and the Browser tab's and the rail's keys are not in a terminal. A later terminal key
  calls the same helper.
- D5: No setting: the cap is the setting.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE no agent CLI runs in the focused terminal, Ctrl-G shall reach the program and show no note. | Shot `563-01-passes` |
| REQ-002 | WHEN one of Marley's actions first takes a key from a focused terminal, the system shall show a toast in that workspace naming the key as bound and what it did, with Open Keymap. | Shots `563-02-note`, `563-04-blocks` |
| REQ-003 | WHEN the same action takes the key again, in the same session or after a relaunch on the same data directory, the system shall show no note. | Shots `563-03-once`, `563-05-remembered`; the run log's store rows |
| REQ-004 | WHEN the user clicks Open Keymap, the system shall open the user's keymap file. | Review of the diff: the button dispatches Zed's `OpenKeymapFile` |
| REQ-005 | Each action shall have a note of its own. | Shot `563-04-blocks` after `563-02-note` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** `shortcut_note.rs`, the four call sites; fmt and clippy clean; a review of the
  diff (the fall-through arms untouched; the record written after the toast, off the main thread).
- **P3 Test:** write and run the scenario and read every shot; rerun `481-rich-input.sh` (golden:
  its first Ctrl-G now shows the toast, and its checks read the stand-in's block, which is
  unchanged); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md` (a Shortcut note
  section); the plan's T7; the ledger capture; close the ticket, archive, commit.
