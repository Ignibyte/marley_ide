---
pipeline_id: aaca10a4-46a1-4d84-a160-d6c63a13f635
ticket: docs/planning/tickets/open/TICKET-450-new-agent-from-the-keyboard.md
status: Phase 4 — Complete PASS
title: New Agent from the keyboard
type: feature
slice: workbench shell W5c
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/449-terminal-keys.spec.md, docs/planning/pipeline/completed/440-rail-agent-clis.spec.md]
---

## Title
One key starts an agent. `ctrl-alt-n` (`cmd-alt-n` on macOS) opens a New Agent picker over
Zed's agents and the installed agent CLIs, and the choice starts in the active project, as the
rail's `+` menu does. Chad's first complaint was that he could not find how to start an agent;
W4 answered it for the mouse. The key needs a binding of Marley's own, so this slice brings the
Marley keymap of workbench-shell D7.

## Scope
### In
- An action `marley::NewAgent`, registered on every workspace, that opens the New Agent picker
  (a `Picker` in the workspace's modal layer). It lists:
  - the Zed Agent, then the project's configured external agents, as the rail's New Agent
    Thread submenu does;
  - then each agent CLI found on the search path, as the rail's Agent CLIs entries do.

  Typing filters the list fuzzily. A Zed agent starts a new thread, focused, in the
  workspace's Agent Panel. A CLI starts in a new center terminal where New Terminal would
  open, with its program name written once the shell is ready. With AI disabled
  (`disable_ai`), the action opens nothing.
- The code the picker and the rail share moves out of `rail.rs` into a new `agents` module: the
  choices and their icons, the two launches, and one seam for the search path and the terminal
  factory, which replaces the rail's two fields.
- The Marley keymap: `crates/marley_workbench/keymap.json` with one binding,
  `secondary-alt-n` to `marley::NewAgent` in the Workspace context. `load_keymap` parses it with
  `KeymapFile::load` and binds it tagged `KeybindSource::Default`; a malformed keymap is
  logged and binds nothing.
- One line at the end of `load_default_keymap` in `crates/zed/src/zed.rs` calls
  `marley_workbench::load_keymap`, with a `// Marley:` comment, its row in
  `docs/marley/zed-touchpoints.md`, and a test beside Zed's own keymap tests.

### Out (explicitly deferred)
- More Marley bindings; each new one gets the same shadow sweep.
- The chord in the rail's `+` tooltip or the picker's footer.
- Agent CLIs beyond `marley_agent`'s four.
- Starting an agent in a project other than the active one (the rail's `+` does that).

## Reference (§20)
- **Warp:** an agent starts from the keyboard, in the session in front of you: typed text that
  reads as a prompt enters Agent Mode, with no trip to a menu
  (`docs/warp_architecture/subsystems/04-agent-ai-mcp.md`, the input classifier; behavior only,
  and Warp's agent source stays unread). Marley's key does that job for its agent CLIs and
  Zed's agents.
- **Upstream Zed:** the picker is Zed's `picker::Picker` in the workspace's modal layer, as the
  task and recent-projects pickers are; its Zed-agent entries and launch are the Agent Panel's
  own (`AgentPanel::new_external_agent_thread`), as the rail uses them (#439). The keymap follows
  Zed's model: a JSON keymap loaded with `KeymapFile::load` and tagged as a default source, as
  `load_default_keymap` loads Zed's (`crates/zed/src/zed.rs:2344-2377`).

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/09-vim-keymap-contexts.md` (contexts and
  precedence), `07-workspace-panes-palette.md` (the modal layer and pickers); the Warp agent map
  above.
- **Published material:** Zed's key-binding docs (`docs/src/key-bindings.md:169-172`: a lower
  context wins, then the later binding, and user bindings load after the defaults).
- **Code we already ship.**
  - `picker::Picker`, `PickerDelegate` (`crates/picker/src/picker.rs:165-300`),
    `Workspace::toggle_modal` (`workspace.rs:8527`), `fuzzy::match_strings`.
  - The rail's agent code (#439, #440): `agent_choices`, `agents_on_path`, `agent_icon`,
    `agent_icon_name`, `Rail::new_agent`, `Rail::new_agent_thread`
    (`crates/marley_workbench/src/rail.rs:439-550`, `:952-968`, `:1109-1152`).
  - `KeymapFile::load` (`crates/settings/src/keymap_file.rs:258`), `KeybindSource`;
    `load_default_keymap` and `reload_keymaps`, which clears every binding and calls
    `load_default_keymap` before binding the user's (`zed.rs:2325-2342`);
    `init_keymap_test` and `test_disable_ai_filters_keybindings` (`zed.rs:6475`) as the test's
    model. `secondary-` means `cmd` on macOS and `ctrl` elsewhere
    (`crates/gpui/src/platform/keystroke.rs:116-143`).
  - `project::DisableAiSettings` (`crates/project/src/project.rs:1151`).
- **The chord sweep** (`PR-claude-new-chord-shadowed-by-hardcoded-key-001`): `ctrl-alt-n` is
  unbound in every context of Zed's Linux and Windows defaults, `cmd-alt-n` in macOS's, and
  neither is in Vim's keymap or the Atom, Cursor, Emacs, Sublime Text, TextMate or VS Code base
  keymaps. JetBrains's Linux base keymap binds `ctrl-alt-n` in Editor and in Workspace: for a
  JetBrains user its Editor binding wins inside an editor, since the deeper context wins, and
  Marley's wins elsewhere, since it loads after the base keymap. Hyprland's configuration on
  the dev box binds nothing on it. Chad's `base_keymap` is `Zed`.

## UI proof
UI-AFFECTING.
- **Driven tests:** the action opens the picker with the right entries in order; typing
  filters them; choosing the Zed Agent opens a thread in the Agent Panel; choosing a CLI opens a
  center terminal that receives the launch bytes (a display-only terminal and its write log, as
  #440's tests do, so no real CLI starts); the chord opens the picker once the Marley keymap is
  bound; a user binding on the chord wins; with AI disabled nothing opens. A test in `zed.rs`
  shows `load_default_keymap` binds the Marley keymap, and a reload binds it again.
- **Live drive:** press `ctrl-alt-n`, filter, choose Claude Code; screenshot the picker and the
  started agent. It needs keys, so it runs only while Chad is away from the desk; otherwise the
  Test phase records why.

## Locked-In Decisions
- D1 — A picker, not the rail's `+` menu: the picker works in both layouts and without the rail
  open, and fuzzy filtering suits the keyboard.
- D2 — The chord is `secondary-alt-n` in the Workspace context, after the sweep above.
- D3 — The Marley keymap is a JSON file in the crate, bound from one line at the end of
  `load_default_keymap`, so every reload binds it again, it beats Zed's defaults at equal
  depth and it loses to the user's keymap (workbench-shell D7).
- D4 — The picker and the rail share one implementation of the choices and the launches, and
  one seam for the search path and the terminal factory; no real agent CLI starts in a test.
- D5 — With AI disabled the action opens nothing, as Zed hides its own agent entry points.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley::NewAgent` runs in a workspace, a picker shall open listing the Zed Agent, then the project's configured external agents, then each agent CLI found on the search path | driven test |
| REQ-002 | WHEN a Zed agent is chosen, the picker shall close and a new thread for it shall open, focused, in the workspace's Agent Panel | driven test |
| REQ-003 | WHEN an agent CLI is chosen, the picker shall close, a new center terminal shall open where New Terminal would, and the CLI's program name and Enter, and nothing else, shall be written to it once its shell is ready | driven test with a display-only terminal and its write log |
| REQ-004 | WHILE a query is typed, the picker shall list only the choices whose names match it | driven test |
| REQ-005 | WHILE AI is disabled, `marley::NewAgent` shall open nothing | driven test |
| REQ-006 | The Marley keymap shall bind `secondary-alt-n` to `marley::NewAgent` in the Workspace context as a default source, and a user binding on the same keys shall win over it | driven tests with simulated keystrokes |
| REQ-007 | A Marley keymap that does not parse, or that names an unknown action, shall bind nothing and report the error | unit test |
| REQ-008 | WHEN Zed's `load_default_keymap` runs, at startup or in a reload, it shall bind the Marley keymap | test in `crates/zed/src/zed.rs` |
| REQ-009 | The rail's `+` menu shall start agents as before | the #439 and #440 rail tests, unchanged but for the seam's setup |
| REQ-010 | The `zed.rs` hook shall be one line with a `Marley:` comment and a ledger row, and the diff gate shall be green | review, gate:16, `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the `agents` module and the picker, the rail's move onto it, the keymap and its
  loader, the `zed.rs` line and its test, the ledger row; fmt and clippy clean; a review of the
  diff.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
