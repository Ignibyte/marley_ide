# An agent CLI's terminal as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-928-agent-terminal-as-the-board.md
- **Pipeline spec:** 928-agent-terminal-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (AgentTerminal).
- **Classification / tier:** feature, M. Marley crate, plus one field in the Marley settings file
  that lives in a Zed crate (`settings_content/src/marley.rs`).
- **Recall (§18.3):**
  - AD-claude-556-an-agent-runs-at-the-prompt-behind-two-lists-and-a-signed-prompt-001: the two
    lists and the signed prompt; Always run adds to the allow side only and never overrides the
    denylist.
  - L-claude-556-nothing-typed-since-the-prompt-is-an-unset-input-start-001: the run's checks at
    the prompt stay as they are; Always run goes through the same `type_run`.
  - F-claude-810-the-take-over-bar-had-no-way-out-001 and PR-claude-810: the bar keeps its close
    button and its third state.
  - F-claude-595-the-drive-bar-outlived-the-program-it-named-001: the bar still checks that the
    program it names runs; only its words change.
  - F-claude-593-the-write-toast-covered-the-cards-allow-and-deny-001: the write's toast stacks at
    the bottom right; the card's buttons stay at its left.
  - PR-claude-783-an-editor-in-a-terminal-keeps-the-keys-its-actions-pass-on-001: the hint row
    names keys the editor keeps.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001, PR-claude-687, F-claude-819: the
    stand-in Claude Code first on the terminals' PATH through the `.bashrc`.
  - L-claude-871-a-scenarios-mcp-client-runs-in-marleys-terminal-001: the stand-in MCP client
    calls `terminal_run` from a terminal of the run.
- **Discovery:**
  - `crates/marley_workbench/src/terminal_drive.rs`: `Pending` (127), `footer` (666), the run's
    verdict (836), `ask_to_run` (1031), `answer`.
  - `crates/marley_terminal/src/agent_commands.rs`: `verdict`, `Verdict`.
  - `crates/marley_workbench/src/rich_input.rs`: `element`, `placeholder` (367).
  - `crates/marley_workbench/src/marley_workbench.rs`: `MarleySettings` (529, 960).
  - `crates/settings_content/src/marley.rs`: `agent_command_allowlist` (150).
  - `crates/marley_workbench/keymap.json` (34–38) and `keymap-macos.json` (19).
- **Decisions:** D1–D5 in the spec.

### Design
- `Pending` becomes a struct with named fields for the card: `who`, `command`, `folder:
  Option<PathBuf>`, `listed: Listed` (`enum Listed { AskList, Neither }`), `always:
  Option<String>` (the prefix, none when D3 rules it out), `kind: PendingKind { Run, Write }`,
  `answer`. A write keeps its words and gets the bar it has.
- `footer` for a run: `v_flex` card, `rounded_lg`, `border_1` in `border_variant`, a header
  `h_flex` (8 px dot in `status().warning`, who in default weight, "in <folder>" muted, the
  status muted at the right), the command as a mono `Label` in a `div` on
  `editor_background`, and a button row: Run (filled) with `KeyBinding::for_action_in(
  &RunAgentCommand, focus, window)`, Refuse with its key, Always run <prefix> here (subtle).
  The footer has no `Window`; the key labels come from `KeyBinding::for_action(&action, cx)`
  where available, else the text of the keymap's binding (the Code phase picks what Zed's `ui`
  offers without a window).
- Always run: `answer_always(id, cx)` writes `agent_command_allowlist_by_folder[folder]` with the
  escaped pattern through `settings::update_settings_file`, then answers `true`. The verdict in
  `run` builds the allowlist from the global list plus every map entry whose key is a parent of
  the terminal's folder.
- `marley_workbench.rs`: `MarleySettings.agent_command_allowlist_by_folder: BTreeMap<PathBuf,
  Vec<String>>`; `settings_content`: `agent_command_allowlist_by_folder: Option<BTreeMap<String,
  Vec<String>>>`, the user's settings only.
- Typing bar: `format!("{} is typing into this terminal's program", written.who)`; Take Over and
  Hand Back labels gain " · <key>".
- Hint row: in `rich_input::element`, for `Target::Agent`, a muted `h_flex` of four
  `key · words` pairs under the editor, keys from the keymap (`editor::Newline` for Shift-Enter
  in the rich input's context, Marley's send and hide actions, the paste action).
- File manifest:
  - Marley crate: `terminal_drive.rs`, `rich_input.rs`, `marley_workbench.rs`.
  - Zed crate (Marley's file): `crates/settings_content/src/marley.rs`.
- Ledger: the `crates/settings_content/src/marley.rs` row in `docs/marley/zed-touchpoints.md`
  gains `agent_command_allowlist_by_folder: Option<BTreeMap<String, Vec<String>>>`, the patterns
  an agent's `terminal_run` runs without asking in a folder and under it (#928).

### Visual check plan
- REQ-001: `mcp_agent tool terminal_run '{"terminal": …, "command": "cargo test -p demo"}'`
  from terminal 2; `928-01-card` shows the card under terminal 1.
- REQ-002: click Always run cargo test here; the scenario prints the run's settings file to show
  the folder's pattern; a second `terminal_run` of `cargo test --doc` answers at once (its answer
  printed) and `928-02-always` shows no card. `cargo` is the scenario's stand-in.
- REQ-003: `python3` in terminal 1, then `terminal_type` with `print(1)`: `928-03-typing`.
- REQ-004: in terminal 1, `claude` (the stand-in), then Ctrl-G: `928-04-hints`.
- Marley Day: `928-05-day`.

### Risks
- A real `cargo test` in the run would build in the box's shared target folder. The scenario
  puts a stand-in `cargo` (prints a line, exits 0) first on the terminals' PATH, so no real cargo
  runs.
- `update_settings_file` writes the user's settings; in a run that is the run's profile copy,
  never the user's (the runner's profile isolation).

## Phase 2 — Code
- **Built:** `agent_command_allowlist_by_folder` in `settings_content` and `MarleySettings`
  (`allowlist_by_folder`); `agent_commands::always_prefix` and `prefix_pattern`; in
  `terminal_drive.rs` `PendingKind`, `RunCard`, `Listed`, `run_card`, `run_card_buttons`,
  `write_card`, `answer_always`, the folder patterns in `check_run_in`, the typing bar's words
  and Take Over's key; `rich_input::agent_key_hints`.
- **Deviations:** the keys come from `KeyBinding::for_action_in` on the footer context's focus
  handle, which needs no window. `run_card` and `write_card` came out of `footer` for the line
  limit. `Written` keeps only who wrote (D5).
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- Scenario `script/e2e/928-agent-terminal-as-the-board.sh` (sway), full run green; the stand-in
  client is `mcp_agent` on Marley's own endpoint.
- `928-01-card`: "Stand-in agent wants to run in <repo>", "not on either list", `cargo test -p
  demo` in mono, Run Enter, Refuse Escape, Always run cargo test here. REQ-001.
- `928-02-always`: the click ran it (exit 0); the settings hold `"<repo>": ["cargo test(\\s.*)?"]`;
  `cargo test --doc` then ran with no card. REQ-002.
- `928-05-day-card`: `cargo build --release` waiting on the card in Marley Day; Escape refused it.
  REQ-001.
- `928-03-typing`: python3 with `print(6 * 7)` → 42; the bar "Stand-in agent is typing into this
  terminal's program" with Take Over Ctrl-I and the close button. REQ-003.
- `928-04-hints`, `928-05-day-hints`: the stand-in Claude Code, Ctrl-G, "Run the tests" typed: the
  row Enter sends · Shift-Enter new line · Escape keeps the draft · Ctrl-Shift-V pastes an image.
  REQ-004.

## Phase 4 — Complete
- Docs: CHANGELOG; `docs/marley_architecture/marley_workbench.md` (the drive's card and bar, the
  rich input's key row); the look plan's TICKET-826 line; the `settings_content/src/marley.rs`
  ledger row.
- Knowledge: L-claude-927 corrected to point at L-claude-871 and `mcp_agent`.
