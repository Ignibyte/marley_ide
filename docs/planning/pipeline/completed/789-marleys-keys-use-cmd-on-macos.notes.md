# Marley's own keys use Cmd on macOS — Notes

- **Local ticket doc:** [TICKET-789](../../tickets/open/TICKET-789-marleys-keys-use-cmd-on-macos.md)
- **Pipeline spec:** 789-marleys-keys-use-cmd-on-macos.spec.md

## Phase 1 — Plan
- **Request:** the move's plan, Phase 5 item 1: a macOS keymap.
- **Classification / tier:** feature, macOS. Marley crate only.
- **Recall (§18.3):**
  - From the summary of #784: bindings rank by context depth, `null` suppresses outranked
    bindings, and Marley's keymap loads after Zed's as a default source.
  - #649: Ctrl+G is the agents' own key, so it stays.
  - The brain (consultation `8513e137bdf048ef80a013418105ac16`): nothing.
- **Discovery:**
  - `keymap.json`: 29 `ctrl-` bindings and 6 `secondary-`, listed by context in the spec.
  - `load_keymap` (`marley_workbench.rs:1113`) calls `load_keymap_from(KEYMAP)`. The tests load
    `KEYMAP` too (`marley_workbench_tests.rs:734`, `rail_tests.rs:1107`).
  - Zed's Mac defaults on the twins' keys are listed in the spec's Prior art.
- **Decisions:** D1 to D3 in the spec.

### Design
- `crates/marley_workbench/keymap-macos.json`: the contexts and keys in the spec's scope, each
  with the action of its Ctrl twin, and a top comment saying what it is for.
- `marley_workbench.rs`: `#[cfg(target_os = "macos")] const MAC_KEYMAP: &str =
  include_str!("../keymap-macos.json");`. `load_keymap` adds, under the same `cfg`,
  `load_keymap_from(MAC_KEYMAP, cx).log_err()`.
- **File manifest:** `crates/marley_workbench/keymap-macos.json` (new) and
  `crates/marley_workbench/src/marley_workbench.rs` (Marley crate).

### Visual check plan
| REQ | What runs on the Mac | What shows it |
|---|---|---|
| 001, 002 | install, restart, then `actions_list` through Marley's MCP | the Cmd keys on each overlay action, and `ctrl-w` still on the word delete |
| 003 | the log since the restart | no "Marley keymap" error |
| 004 | Linux: `just shot` | Marley starts. The overlay is `cfg`-bound |

No keystroke can be sent to the Mac's Marley over SSH, so the keys are seen as Marley binds them,
not as typed.

## Phase 2 — Code
- **What was built:**
  - `keymap-macos.json`, 8 blocks with 19 actions as the spec lists, each a twin of an action
    `keymap.json` already binds (checked by a script: none missing). Its top comment says what
    stays Ctrl and why.
  - `marley_workbench.rs`: `MAC_KEYMAP` (`include_str!`) and its `load_keymap_from` after
    `KEYMAP`, both `cfg(target_os = "macos")`.
- **Review:**
  - REQ-001: every twin sits in the same context as its Ctrl key. Where Zed's Mac keymap binds
    the same Cmd key, it is at Workspace or Editor depth, under Marley's contexts.
  - REQ-002: no shell key has a twin.
  - REQ-003: the overlay loads all or nothing through `load_keymap_from`, and an error is logged
    with "the Marley keymap did not load".
  - REQ-004: on Linux nothing is compiled in.
- **Checks:** box clippy on `marley_workbench` green; **GATE GREEN [diff]**, 17 of 17. Mac
  clippy `-D warnings` on `marley_workbench` green, then `just install` there.

## Phase 3 — Test
- **The run, on the Mac.** The install had this diff at `4a9a183c66`. The test Marley was quit
  and reopened, and the log's line count kept as the start. The bindings were read through
  `actions_list` on Marley's MCP: the running app's own keymap, since no key can be typed there
  over SSH.
- **REQ-003:** 0 log lines naming the keymap since the restart.
- **REQ-001:** each twin is bound in its context, listed before its Ctrl key:
  - `marley::UndoCloseTerminal`: Command-Shift-T and Control-Shift-T, Workspace;
  - `marley::TakeOverTerminal`: Command-I and Control-I, Terminal;
  - `marley::RichInputPaste`: Command-V and Control-Shift-V; `RichInputCopy`: Command-C and
    Control-Shift-C; both `MarleyRichInput > Editor`;
  - `marley::NewBrowserTab`: Command-T and Control-T, MarleyBrowser;
  - `rusty::PageBack`: Command-[ and Option-Left, RustyPage.
- **REQ-002:** `editor::DeleteToPreviousWordStart` is still Control-W in
  `MarleyRichInput > Editor`.
- **REQ-004:** `just build`, then `COMPOSITOR=sway just shot 789-linux-unchanged`.
  `789-linux-unchanged.png` is Marley as before, and the run's log has no keymap line.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Added: "Cmd keys on a Mac".
  - `docs/marley/guide.md`, On a Mac: a Keys item, and Cmd off the lacks list.
  - `docs/marley/three-prong-plan.md`: the macOS bullet.
  - `docs/marley_architecture/marley_workbench.md`: the overlay.
- **Ledger:** nothing new. No bug was found, and the decision is in the brain.
- **Brain:** consultation `8513e137bdf048ef80a013418105ac16` closed by
  `decisions/marleys-mac-keys-are-a-cmd-overlay-on-its-ctrl-keymap`.

