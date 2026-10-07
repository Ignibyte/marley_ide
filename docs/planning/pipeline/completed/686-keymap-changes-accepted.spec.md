---
pipeline_id: beb50ce1-411d-4d76-97b5-f5113fe9f29c
ticket: docs/planning/tickets/open/TICKET-686-keymap-changes-accepted-as-a-diff.md
status: Phase 4 — Complete PASS
title: Keymap changes the user accepts
type: feature
slice: prong 2 C; phase 1 item 4's second half of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/682-settings-changes-accepted.spec.md
---

## Title
`keymap_change`: an agent proposes a key binding for the user's `keymap.json`; Marley asks as
#682 asks for a setting and writes it only on Apply, through Zed's own keymap updater.

## Scope
### In
- `keymap_change` (write, grant class `settings.write`): `keystrokes` (`ctrl-alt-m`, a sequence
  space-separated), `action` (an action's name), optional `context` (a key context predicate,
  such as `Workspace`) and optional `arguments` (the action's JSON arguments).
- Checked before the user is asked: the action must be one of the app's (`no_action`, with close
  names), the keystrokes must parse (`bad_argument`), and the context must parse (`bad_argument`).
- The question: #682's notification, now shared: "<agent> wants to bind a key", the keystrokes, the
  action and its palette name, the context, the file; Apply, Decline, 25 seconds.
- The write: `KeymapFile::update_keybinding` with `KeybindUpdateOperation::Add` on the user's
  `keymap.json` as read, which keeps its comments and other bindings; refused with `changed` when
  the file moved meanwhile. Zed's keymap watcher applies it at once.
- The answer: `applied` with the keystrokes, the action, the context and the file.

### Out (explicitly deferred)
- Removing or replacing a binding (`KeybindUpdateOperation::Remove` and `Replace`).
- Unbinding a default key.

## Reference (§20)
Upstream Zed (`settings::keymap_file`, `keymap_editor`): the keymap editor adds a binding with
`KeymapFile::update_keybinding` and writes `paths::keymap_file()`; Marley takes the same path.

### Prior art
- **The code we ship:** `crates/settings/src/keymap_file.rs:891` (`update_keybinding`), `:1291`
  (`KeybindUpdateOperation`), `:1357` (`KeybindUpdateTarget`);
  `crates/keymap_editor/src/keymap_editor.rs:3640-3690` (the editor's save: the operation, the
  update, the write); #682's `settings_change.rs` (the question, the 25-second wait, the reread
  before the write).
- **Behavior maps:** T3 survey report 05 item 6 (config writes that show the change and check the
  file did not move), as #682.

## UI proof
`script/e2e/686-keymap-changes-accepted.sh` (`compositor sway`). The stand-in agent proposes
`ctrl-alt-m` for `workspace::ToggleRightDock` in `Workspace`: the question (`686-01-card`);
Apply, the answer `applied`, the copy's `keymap.json` holding the binding; Ctrl+Alt+M then hides
the right dock (`686-02-key-works`). An unknown action is refused with `no_action` and no question
(`686-03-refused`).

## Locked-In Decisions
- D1 — Zed's own keymap updater, not a JSON edit of Marley's: it knows the file's array shape,
  keeps comments, and is what the keymap editor uses.
- D2 — #682's question and wait, shared rather than copied.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `keymap_change` with keystrokes that parse, an action the app has and a context that parses, the system shall show a notification naming the agent, the keystrokes, the action, the context and the file, with Apply and Decline. | `686-01-card` |
| REQ-002 | WHEN the user picks Apply, the system shall add the binding to the user's `keymap.json` and answer `applied`. | The answer; the file |
| REQ-003 | WHEN the binding is written, the keystrokes shall run the action without a restart. | `686-02-key-works`: the right dock gone after Ctrl+Alt+M |
| REQ-004 | WHEN the action is not one of the app's, the system shall refuse with code `no_action` and show nothing. | The answer; `686-03-refused` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the registry row and schemas; `settings_change.rs`'s question made shared;
  `keymap_change` beside it; the scenario; a review; `just gate-diff`.
- **P3 Test** — the scenario; every shot read.
- **P4 Complete** — docs, ledger, close, archive, commit, push, install.
