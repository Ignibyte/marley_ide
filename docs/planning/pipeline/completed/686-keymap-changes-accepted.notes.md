# Keymap changes the user accepts — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-686-keymap-changes-accepted-as-a-diff.md
- **Pipeline spec:** 686-keymap-changes-accepted.spec.md

## Phase 1 — Plan
- **Request:** the queue's top, #682's second half, on Chad's "just have it go" (2026-10-07).
- **Classification / tier:** feature, prong 2 C; Marley crates only.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (#684's install compiling; planning
  only) · recall ✓ · mint ✓ · prior art ✓ · spec ✓ · design ✓.
- **Recall (§18.3):** #682's question, 25-second wait (`L-claude-682-a-tool-that-waits-for-the-user-answers-inside-the-servers-30-seconds-001`)
  and reread; `L-claude-682-no-source-edit-while-an-install-compiles-001`.

### Design
- `marley_mcp`: `Family::Keymap` with the verb `change` (the wire name `keymap_change`), write,
  grant class `settings.write`; its schemas, the dispatch arm, the registry test.
- `settings_change.rs`: the question generalized (`Question { headline, change, file }`, `ask_user`
  returning the answer after the 25-second race and the dismissal); `settings_change` uses it.
  `keymap_change` in the same module: parse the keystrokes (`Keystroke::parse` per word, then
  `KeybindingKeystroke` through the platform's keyboard mapper), check the action against
  `cx.all_action_names()`, parse the context (`KeyBindingContextPredicate::parse`), load
  `paths::keymap_file()` (an absent file reads as `[]`), `KeymapFile::update_keybinding(Add {
  source, from: None })`, ask, reread, `atomic_write`.
- `mcp.rs`: `keymap_` calls go to the module.
- Scenario: the stand-in's `tool keymap_change`; Apply at #682's position; Ctrl+Alt+M; the right
  dock's project panel gone.

**File manifest:** `crates/marley_mcp/src/{registry.rs, dispatch.rs}`,
`crates/marley_workbench/src/{settings_change.rs, mcp.rs}`, the scenario.

### Risks
- The keystrokes' form: Zed's keymap writes `ctrl-alt-m`; the tool takes that form and refuses
  others with Zed's parse error.

## Phase 2 — Code
- **Built:** `marley_mcp`: `Family::Keymap` (`keymap_change`, write, `settings.write`),
  `keymap_change_schemas`, the dispatch arm, the instructions' clause, the registry test (40).
  `settings_change.rs`: the question generalized (`Question { headline, change, file }`) and the
  ask, wait, reread and write moved into `ask_then_write`, which `settings_change` now calls;
  `load` takes what an absent file reads as (`{}` for settings, Zed's initial keymap for the
  keymap); `answer_keymap`, `check_binding` (the action among the app's with close names,
  `Keystroke::parse` per step, `KeyBindingContextPredicate::parse`), `propose_binding`
  (`KeymapFile::update_keybinding` with `Add`, the keystrokes mapped through the platform's
  keyboard mapper on the main thread). `mcp.rs` routes `keymap_change`. The scenario.
- **Fix outside the plan:** `script/e2e.sh`'s copy of the settings now sets
  `marley.assistant.enabled: false`. #683's offer came over this scenario's question in its first
  run (the copy is the user's settings, which do not name the switch, and the user's `claude` is
  signed in); it would cover any scenario's clicks five seconds in. #683's scenario takes the key
  out again in its `setup`.
- **Review:** an unknown action, unparseable keystrokes or context are refused before the file is
  read; the binding is added (`Add`), never replacing one; the reread compares with the same
  absent-file text it read with. Clippy: `Rc::clone` for the mapper.
- **Checks before the gate:** run 1 green but with #683's offer stacked over the card; run 2 green,
  3 of 3, the card alone (`scratchpad/686-e2e-2.log`).
- **Gate:** `just gate-diff` green, 17 of 17 (`scratchpad/686-gate-1.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/686-keymap-changes-accepted.sh` (`compositor sway`), run 3, the Test
  phase's: 3 of 3 (`scratchpad/686-e2e-3.log`); #683's scenario rerun after the runner change,
  2 of 2, its offer still shown (`scratchpad/683-e2e-3.log`).
- **Shots** (Marley only):
  - `686-01-card` (REQ-001): "Stand-in agent wants to bind a key", "ctrl-alt-m → workspace: toggle
    right dock (workspace::ToggleRightDock) in Workspace", the run's `keymap.json`, Apply and
    Decline, alone.
  - `686-02-key-works` (REQ-002, REQ-003): after Apply and Ctrl+Alt+M, the right dock is gone and
    the center reaches the window's edge; the answer `applied` and the file holds the binding.
  - `686-03-refused` (REQ-004): no question after `workspace::NoSuchAction`, answered `no_action`.
  - `683-01-offer` (rerun): the offer still comes in #683's own scenario.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #686, and the e2e copy's change); the guide's write-tool
  table and the in-app guide's row; `docs/marley_architecture/marley_mcp.md` and
  `marley_workbench.md`; the plan doc's item 4.
- **Knowledge (§19):** `PR-claude-686-a-prompt-that-comes-on-its-own-is-decided-in-the-e2e-copy-001`
  with `F-claude-686-the-agents-offer-came-over-a-scenarios-question-001`. Brain: no consultation;
  the ticket's decisions are #682's and Zed's keymap updater's.
- **Ticket:** TICKET-686 closed.
