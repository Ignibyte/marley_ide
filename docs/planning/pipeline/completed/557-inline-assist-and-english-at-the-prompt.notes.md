# Inline Assist proven, and English at the prompt by local rules — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-557-inline-assist-and-english-at-the-prompt.md
- **Pipeline spec:** 557-inline-assist-and-english-at-the-prompt.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note (2026-09-25), recommendation 4: "An e2e scenario that proves
  `Ctrl+Enter` Inline Assist (S); then English detection by local rules, hint only, with
  `Ctrl+Shift+Enter` and the exit-127 chip (S to M)." Chad, 2026-09-26: "local first and then
  jev second"; no network model here; #573 is the second stage.
- **Classification / tier:** feature, S to M. A scenario with a stand-in model and no code for
  the first half; a pure rules module, a command set, the hint through an existing hook, one
  action, one button and one setting for the second. Zed paths: the settings trio (rows exist)
  and the element's hook if #556 or #528 has not added it.
- **Recall (§18.3):**
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001 and
    L-claude-484-where-a-typed-command-starts-without-touching-the-prompt-001: the typed line is
    read from the grid between the input's start and the cursor; the rules read the same text.
  - PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: the exit-127 button
    turns a block's command into input, so it needs `command_verified`.
  - F-claude-481-the-rich-input-dropped-every-typed-character-on-linux-001: keys and the
    terminal's `key_down`; the picker is a modal and needs none of that.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: revealing the agent's tab
    moves the focus on purpose here (the user asked the agent), so nothing to work around.
  - L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001: the stand-in
    `claude` must print before the bar and `agent_in` see it.
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001: not in play; the
    stand-in stands for a real agent.
  - Brain: no page on natural-language detection (searched 2026-09-26); the jev note is the
    design record for the second stage.
- **Discovery:**
  - Inline Assist: `crates/agent_ui/src/inline_assistant.rs`: `inline_assist` (206), the Agent
    Panel requirement (226), `resolve_inline_assist_target` (1460);
    `terminal_inline_assistant.rs`: `assist` (61), the block (98 to 103), `request_for_inline_assist`
    (212), `finish_assist` (284); `terminal_codegen.rs`: `CLEAR_INPUT` (177), the transaction
    (182 to 217); `inline_prompt_editor.rs`: `secondary_confirm` (534). `crates/zed/src/zed.rs:906`
    registers the action; `crates/agent_ui/src/agent_ui.rs:900` chooses the models from the
    settings; `crates/agent/src/agent.rs:361` authenticates every provider when the native agent
    builds.
  - The stand-in: `crates/language_models/src/provider/ollama.rs`: `is_authenticated` (68),
    `authenticate` (92), `fetch_models` (112), `api_url` (245), `default_model` (280),
    `provided_models` (287); `crates/ollama/src/ollama.rs`: `stream_chat_completion` (296,
    `/api/chat` at 303, NDJSON lines into `ChatResponseDelta`, 155), `get_models` (342),
    `show_model` (377); `LocalModelListing` (167). Settings: `language_model.rs:22` (`ollama`),
    `189` (`OllamaSettingsContent`), `199` (`OllamaAvailableModel`); `agent.rs:261`
    (`default_model`), `268` (`inline_assistant_model`); `default.json:1177` (the default model,
    `zed.dev`) and `2621` (`ollama.api_url`). The profile copy carries Chad's settings; the
    scenario's patch names the stand-in as the default and inline model, so no real provider is
    called. `514-telemetry-off-by-default.sh:14-23` (`settings_prepend`) and
    `516-secret-redaction-for-agents.sh:104-118` (`set_marley`) patch the profile's settings.
  - The line: `autosuggest.rs`: `init` (29), `suggestion` (56), `typed_text` (76);
    `terminal_view.rs:150` (`MarleyTerminalSuggestion`); `terminal_element.rs`: the hook call in
    layout (1663) and the paint at the cursor (1833 to 1857). `suggest.rs:7`.
  - The send: `rich_input.rs`: `open` (57), `send` (97), `element` (140); `browser.rs`:
    `send_pick` (3645), `LastTerminal` (5400), `track_terminals` (5409); `agents.rs`:
    `installed_clis` (85), `start_cli` (188); `agent_bar.rs`: `agent_in` (129);
    `agent_events.rs`: `seat` (30); `marley_agent.rs`: `agent_kind_of` (76), `send_payload` (87),
    `launch_input` (93).
  - The button: `terminal_element.rs`: `marley_block` (2320), the actions row (2366);
    `anchored.rs:38-58`; `block.rs:41` (`ExitCode`).
  - Keys: Zed's `Terminal` context binds no `ctrl-shift-enter` (`default-linux.json:1295-1341`);
    Marley's `keymap.json:17-23`.
- **Decisions:** D1 to D7 in the spec.

- **Recall at promotion (2026-09-29):**
  - #555's chip (`send_block::chip`, `send_block.rs:135`) already shows "Ask the agent" before the
    newest failed block's pill at a prompt, for any non-zero exit, when another terminal runs an
    agent; it sends the block's Markdown and does not check `command_verified`. The 127 button
    widens it instead of adding a second one.
  - #549's `send_selection`: `agent_targets(workspace, cx)` (window-wide, the one focused last
    first), `Target`, `send_text` (rich input first; a toast while the seat waits; reveal, focus,
    paste, no return), `TargetPicker`, `show_toast`. `terminal_drive::paste_then` sends a return
    after a paste. `agents::start_cli_with_prompt` starts an agent CLI in a new center terminal
    with a first prompt, quoted as one argument.
  - The suggestion hook (`MarleyTerminalSuggestion`, `terminal_view.rs:273`) draws one dimmed run
    after the cursor; `AcceptSuggestion` recomputes `autosuggest::suggestion` and types it, so a
    hint returned only by the hook's closure is never typed (REQ-011).
  - Zed's Inline Assist: `ctrl-enter` in `Terminal` (`assistant::InlineAssist`), handled by
    `InlineAssistant::inline_assist`, which needs `agent.enabled`, the Agent Panel (the Marley
    layout keeps it) and an `inline_assistant_model`; Marley disables none of it and binds no
    `ctrl-enter`. No scenario configures a model provider yet.
  - `ctrl-shift-enter` is Marley's only in `Terminal && MarleyBlockSelected` (SendBlockToAgent);
    the plain `Terminal` context has none. Brain (consultation ecaf28bb): nothing on this seam.

### Design (at promotion)
- `marley_terminal::english` (pure) as drafted: `Reading`, `read_line`, `MARKERS`, `BUILTINS`.
- `marley_workbench::english` (new): the command set (`Commands` global, the search path's
  executables read off the main thread through `agents::launcher(cx).search_path`, refreshed
  when the path string changes; until read, an unknown word counts as a command);
  `is_command(word, terminal, cx)`; `reading(terminal, cx)` for the typed line; `hint(terminal,
  cx)`; `AskAgent`'s handler and `ask(workspace, text, window, cx)`: the targets, the picker, or
  `start_cli_with_prompt` with Claude Code; `asks_on_127(view, terminal, index, cx)` for the chip.
- `send_selection::send_text` gains `submit: bool` (a return after the paste, through
  `paste_then`); its two callers pass false.
- `autosuggest::init`'s hook closure: the history suggestion, else `english::hint`.
- `send_block::chip`: for a block `english::asks_on_127` accepts, the chip shows without targets
  and its click calls `english::ask` with the block's command.
- The setting `english_hint` (`settings_content`, `default.json` true, `MarleySettings`, a toggle
  in the Layout section); `AskAgent` in `actions!` and `ctrl-shift-enter` in the keymap's
  `Terminal` block.
- **File manifest.** Marley: `marley_terminal/src/{english.rs, marley_terminal.rs}`;
  `marley_workbench/src/{english.rs, autosuggest.rs, send_block.rs, send_selection.rs,
  marley_workbench.rs}`, `marley_workbench/keymap.json`. Zed: `settings_content/src/marley.rs`,
  `settings_ui/src/marley_page.rs`, `assets/settings/default.json`. Script:
  `script/e2e/557-inline-assist-and-english-at-the-prompt.sh`.

### Design (as drafted; the promotion's above wins where they differ)
- **`marley_terminal::english`** (pure): `pub enum Reading { Blank, Command, English }`;
  `pub fn read_line(line: &str, is_command: impl Fn(&str) -> bool) -> Reading` by the spec's
  rules, in that order; `pub const MARKERS: &[&str]`; `pub const BUILTINS: &[&str]` (bash and
  zsh: `cd`, `export`, `alias`, `unalias`, `source`, `.`, `echo`, `printf`, `read`, `set`,
  `unset`, `exit`, `type`, `command`, `builtin`, `eval`, `exec`, `jobs`, `fg`, `bg`, `kill`,
  `wait`, `pushd`, `popd`, `dirs`, `history`, `fc`, `local`, `declare`, `typeset`, `let`, `test`,
  `[`, `[[`, `if`, `then`, `else`, `fi`, `for`, `while`, `until`, `do`, `done`, `case`, `esac`,
  `function`, `select`, `time`, `sudo`, `true`, `false`, `return`, `shift`, `trap`, `umask`,
  `ulimit`, `hash`, `help`, `logout`, `setopt`, `unsetopt`, `autoload`, `bindkey`, `zle`,
  `emulate`, `rehash`, `which`, `whence`). Words are split on whitespace; quotes are left as they
  are (a quoted string decides nothing).
- **The command set** (`marley_workbench::english`, new): a global `Commands { path: OsString,
  names: Option<Arc<HashSet<String>>> }`, read off the main thread from every directory of
  `launcher(cx).search_path` (executable files' names; L-claude-482's lazy future), refreshed
  when the path string changes; `is_command(word, terminal, cx)` = builtins, the set, or the first
  word of a verified block of that terminal. Until the set is read, every unknown word reads as
  a command (no hint), never the other way.
- **The hint.** `autosuggest::init`'s hook closure: the history suggestion when there is one;
  else, with `MarleySettings::english_hint`, `typed_text` read as English → the hint string.
  `AcceptSuggestion` is unchanged (it recomputes the history suggestion).
- **`marley::AskAgent`** (on `ctrl-shift-enter` in `Terminal` in `keymap.json`), registered on
  every workspace like `AcceptSuggestion`: `focused_terminal`, `typed_text`; blank → `cx.propagate()`;
  else `terminal.input("\u{15}")`, then `send_to_agent(workspace, text, window, cx)`:
  the agent terminals = the workspace's terminal views (center and panel) whose `agent_in` is
  `Some`; one → reveal (`send_pick`'s route), focus, refuse if `agent_events::seat` says
  `Waiting`, else `paste` + `\r`; several → a `Picker` listing "<agent> · <tab title>", Enter
  sends; none → `agents::start_cli` with a first argument (a small extension: `start_cli_with(kind,
  argument)`, quoting with `ShellKind::try_quote`).
- **The button.** `MarleyBlockExtras::buttons` returns, for a `Finished` block with
  `ExitCode(Some(127))`, `command_verified` and an English reading (or an unknown first word), an
  `IconButton` "Ask the agent" (`IconName::ZedAssistant` or the sparkle Zed ships) that the
  element places in the actions row, shown without hover (the row's `visible_on_hover` applies
  to the hover buttons only; the element gets a second, always-visible slot from the hook).
- **The setting.** `english_hint: Option<bool>` in `MarleySettingsContent`, `true` in
  `default.json`, a toggle "English at the Prompt" in the Marley page's Layout section
  (`marley_page.rs:17`), read as `MarleySettings::english_hint`.
- **File manifest.** Marley crates: `crates/marley_terminal/src/{english.rs (new),
  marley_terminal.rs}`; `crates/marley_workbench/src/{english.rs (new), autosuggest.rs,
  agents.rs, marley_workbench.rs}`, `crates/marley_workbench/keymap.json`. Zed paths:
  `crates/terminal_view/src/terminal_element.rs` (the hook, if new; the always-visible slot),
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Scripts: `script/e2e/557-inline-assist-and-english-at-the-prompt.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal_element.rs` row (the hook and
  the slot), the three settings rows.

### E2E plan
`setup` writes the stand-in Ollama server (`ThreadingHTTPServer` on `127.0.0.1:0`; `GET
/api/tags` → one model `fake`; `POST /api/show` → `{"capabilities":["completion"],
"model_info":{"general.architecture":"llama","llama.context_length":4096}}`; `POST /api/chat` →
one line `{"model":"fake","created_at":"…","message":{"role":"assistant","content":"echo
marley-inline-assist"},"done":true}`), starts it with `setsid -f`, records its port, patches the
profile's settings (`language_models.ollama.{api_url, available_models}`,
`agent.{default_model, inline_assistant_model}` on `ollama`/`fake`, `agent.enabled` true), and
puts a stand-in `claude` first on the PATH that prints `argv: …` then echoes each line it reads.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | Ctrl+Enter at the prompt (twice with a settle between, if the first press only authenticates the provider) | `557-01-inline-prompt` |
| REQ-002 | type `print a marker`, Return; settle | `557-02-generated`: `echo marley-inline-assist` on the prompt line |
| REQ-003 | Ctrl+Enter | `557-03-ran`: the block with `marley-inline-assist`, exit 0 |
| REQ-004 | type `what is using port 3000` | `557-04-hint` |
| REQ-005 | Ctrl-U; type `ls -la`; then Ctrl+Shift+Enter | `557-05-no-hint`; the log: the shell's hook frames show no command typed by Marley |
| REQ-006 | Ctrl-U; type `what is using port 3000`; Ctrl+Shift+Enter | `557-06-asked-new`: a new terminal running the stand-in; its log holds `argv: what is using port 3000` |
| REQ-007 | click the first terminal's tab; type `find all the large files in this repo`; Ctrl+Shift+Enter | `557-07-asked-running`: the stand-in's terminal focused; its log holds the line |
| REQ-008 | back in the shell: `show me the biggest folders`, Return | `557-08-exit-127`: the block, exit 127, the button |
| REQ-009 | click the button | `557-09-button`; the stand-in's log |
| REQ-010 | `set_marley english_hint false`; the English line typed again; the 127 block | `557-10-off` |
| REQ-011 | with the hint shown, press →; read the line | the log: `typed_text` unchanged (the hook log's next `preexec` after a Return shows the plain line) |

Not reachable by a scenario: a real model behind Inline Assist (a stand-in stands in; the
provider's protocol is Zed's and unchanged) and a real Claude Code taking the line (the
stand-in shows the bytes; L-claude-482's pty method can show one real paste if Test wants it).

### Risks
- Inline Assist may fail in the Marley layout for a reason of its own (the Agent Panel not
  registered in a window without Zed's dock, the provider's first authentication eating the
  first Ctrl+Enter): that is the finding this half exists for; a fix is in scope when small,
  else it becomes its own ticket, recorded in the notes.
- A false "English" on a real command (an alias, a script in the project's own `bin/`, a word
  the search path lacks): the hint is passive and → never takes it, so the cost is one dimmed
  line. The set includes the terminal's verified commands, which learns the user's aliases as
  they run.
- Ctrl+Shift+Enter is bound in Zed's editor and agent contexts to other things; in `Terminal` it
  is free, and a program that wants it (few do) loses it while a line is typed only.
- The suggestion slot is one string: a very long typed line pushes the hint off the row's end;
  the element clips as it clips suggestions today.

## Phase 2 — Code
- **Built to the promoted design.** `marley_terminal::english` (`Reading`, `read_line`, `MARKERS`,
  `BUILTINS`); `marley_workbench::english` (the `Commands` global read off the main thread from
  the launcher's search path, `is_command`, `hint`, `ask_typed` and `ask`, `asks_on_127`,
  `ask_later`); `send_selection::send_text` gains `submit`, a return after the paste through
  `terminal_drive::paste_then`; the suggestion hook's closure falls back to the hint;
  `send_block::chip` shows for a 127 block that `asks_on_127` accepts and asks with its command;
  `AskAgent` on `ctrl-shift-enter` in `Terminal`; `english_hint` in the settings, `default.json`
  and the Layout section, read as `EnglishHint` (clippy's `struct_excessive_bools` again).
- **Found in Test, fixed here.**
  - Inline Assist's prompt never showed: #476's bottom shift draws short content down onto the
    terminal's bottom edge, and Zed places the prompt block under the cursor line, off the
    view. The prompt still took the keys, so a request still generated. The shift now waits
    while a block sits below the cursor (`terminal_element.rs`, one condition, its row grown).
  - `ls -la` and Ctrl+Shift+Enter went to an agent: `ask_typed` asked with any typed line. It
    now asks only for a line that reads as English, so a command's key reaches the shell
    (REQ-005); with the hint off, English still asks (REQ-010).
- **Deviations.** REQ-003 runs the command with the prompt's run button (▶): on Linux Zed's prompt
  editor binds Ctrl+Enter itself (`editor::NewlineBelow` in `Editor`, deeper than `menu`'s
  `SecondaryConfirm`), so the key never runs it. Upstream behaviour, recorded and left.
- **Review.** Nothing leaves the machine: the reading is local, and the stand-in Ollama the
  scenario uses is on `127.0.0.1`. The hint rides only the hook, so `AcceptSuggestion` never
  types it. The 127 chip needs `command_verified` (PR-claude-474); a block whose frames came from
  output gets none. The chip's click defers to after the element's update (`window.defer`).
- **Gate.** Run 1 red on gate:21 only: dylint's `async_block_without_await` on the search
  path's walk, a synchronous closure inside `background_spawn`; it runs through `smol::unblock`
  now (`smol` added to the workbench). Run 2: `GATE GREEN [diff]`, 16 passed, 0 failed.

## Phase 3 — Test
- **Scenario** `script/e2e/557-inline-assist-and-english-at-the-prompt.sh` under `compositor
  sway`: a stand-in Ollama (`/api/tags`, `/api/show`, `/api/chat` answering `echo
  marley-inline-assist`) on a free loopback port in the run's settings, and a stand-in `claude`
  (`exec -a claude python3`) that logs its arguments and each line it reads.
- **Runs.** 1: the prompt off the view and the command not run (the shift, and Ctrl+Enter).
  2: the prompt shows; the command not run. 3: ▶ clicked; `ls -la` asked an agent. 4: the chip's
  guessed position missed. 5: every check passed. 6: the REQ-010 key check added; all 6 passed.
- **Every shot read** (run 5; run 6's `557-10-off` the same):
  - `557-01-inline-prompt`: the prompt block under the `$` line, `Generate… (Ctrl-? to chat …)`
    and the model `Fake`. REQ-001.
  - `557-02-generated`: `$ echo marley-inline-assist` on the prompt line, not run; the prompt
    with 👍 👎 ✓ ▶. REQ-002.
  - `557-03-ran`: the block `echo marley-inline-assist` with its output and ✓. REQ-003.
  - `557-04-hint`: `$ what is using port 3000` and, dimmed after the cursor, `· ctrl-shift-enter
    asks the agent`; after →, the line unchanged (the log). REQ-004, REQ-011.
  - `557-05-no-hint`: `$ ls -la` with nothing after the cursor; Ctrl+Shift+Enter asked no agent.
    REQ-005.
  - `557-06-asked-new`: a new terminal, `$ claude 'what is using port 3000'`, the stand-in's
    `argv: what is using port 3000`, its rail row an agent; the first terminal's line cleared.
    REQ-006.
  - `557-07-asked-running`: the stand-in's terminal in front with `got: find all the large files
    in this repo`. REQ-007.
  - `557-08-exit-127`: `$ show me the biggest folders`, `bash: show: command not found`, and
    `Ask the agent` beside `exit 127`. REQ-008.
  - `557-09-button`: the stand-in got `show me the biggest folders`. REQ-009.
  - `557-10-off`: with the setting off and the stand-in ended, two 127 blocks with no chip and the
    English line with no hint; Ctrl+Shift+Enter then started a second stand-in (the log). REQ-010.
- **Not reachable:** a real model (the stand-in speaks Zed's Ollama protocol) and a real Claude
  Code taking the line.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added and Fixed; `marley_workbench.md` (a section for `english.rs`),
  `terminal_blocks.md` (`english.rs`); the touchpoint rows for `terminal_element.rs`, the settings
  files and `default.json` describe what shipped.
- **Knowledge.** F-claude-557-the-bottom-shift-hid-inline-assists-prompt-001,
  AD-claude-557-english-at-the-prompt-by-local-rules-001. Brain: `brain_decide` on consultation
  ecaf28bb.
- **Closed** the ticket, archived the pair, committed.
