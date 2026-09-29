# Whether an agent's commands enter the shell history, as a setting — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-553-agent-commands-in-history.md
- **Pipeline spec:** 553-agent-commands-in-history.spec.md

## Phase 1 — Plan
- **Request:** the Warp second pass (2026-09-25), "Smaller": "Warp has a setting for whether
  commands an agent runs enter the user's history (`include_agent_commands_in_history`, on the
  all-settings page). Marley's autosuggestions read the terminal's own commands first, so an
  agent typing into Chad's shell would feed them." Chad, 2026-09-26: every remaining finding
  gets built. The producer of agent-typed commands is #556's `terminal_run`, so this ticket
  follows it.
- **Classification / tier:** feature, S. A setting, a global and one environment pair in the
  terminal's spawn, a rule in each shell script, a space in the typing path, a filter in the
  suggestions. Zed paths: `crates/terminal/src/terminal.rs` (row exists) and the settings trio.
- **Recall (§18.3):**
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001: the suggestions'
    sources are the terminal's own verified commands, newest first, then the shell's history
    file; the filter sits on the first source.
  - PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: the agent's block
    is paired by a verified frame (#556 D8); nothing here trusts an unverified one.
  - The scripts' own comment (`marley.bash:43-44`): the line is read "as history holds it",
    which is the trap this design works around (bash's hook depends on the history list).
  - Brain: no page on shell history (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_terminal/shell_integration/marley.bash`: the nonce (10 to 13), the user's
    `.bashrc` (15 to 17), `__marley_quote` (24 to 32), `__marley_precmd` (36 to 41),
    `__marley_preexec` (45 to 52: `fc -ln -0` at 47, the trim at 48), `PROMPT_COMMAND` (55 to
    59), `PS0` (60), the `init`, `history;file` and `bootstrapped` frames (62 to 68).
    `marley.zsh`: the nonce (20 to 23), `__marley_precmd` (49 to 54), `__marley_preexec` (57 to
    62, `$1`), `__marley_install` (66 to 78, `precmd_functions` first, `preexec_functions+=`),
    the install hook (79). Neither touches `HISTCONTROL`, `HISTIGNORE`, `HIST_IGNORE_SPACE`,
    `history -d` or `zshaddhistory` today (grep, 2026-09-26).
  - `crates/marley_terminal/src/shell_integration.rs`: `BASH_INTEGRATION` (20),
    `ZSH_INTEGRATION` (26), `MARKER_VARIABLE` (41), `NONCE_VARIABLE` (44), `new_nonce` (48),
    `install_in` (69), `for_program` (89), `shown_arguments` (129).
  - `crates/terminal/src/terminal.rs`: `marley_shell_integration` (90), `insert_zed_terminal_env`
    (716), the nonce hunk (1209 to 1219), the integration gate (1223 to 1227), `input` (2316).
    `TerminalBuilder::new` runs inside a future with `cx`, so a `cx.try_global` read is possible
    where the nonce is made.
  - `crates/marley_workbench/src/autosuggest.rs`: `suggestion` (56 to 72, the filter on
    `command_verified` at 63), `typed_text` (76), `read_history` (120). `suggest.rs`: `suggestion`
    (7), `parse_history` (24, bash's `#<seconds>` lines and zsh's extended history).
  - `crates/marley_terminal/src/anchored.rs`: `apply`'s `preexec` arm (108 to 128), `command`
    (42), `history_file` (196). `crates/marley_workbench/src/mcp.rs`: `block_entry` (477).
  - Settings: `settings_content/src/marley.rs:10-29`, `settings_ui/src/marley_page.rs:42-91`,
    `default.json:1665-1674`, `marley_workbench.rs:167-198`; `mcp.rs:81-82` observes the
    settings store for the redaction, the same for the environment global.
  - The manuals: bash `HISTCONTROL` (`ignorespace`, `ignoreboth`), `history -d offset` with a
    negative offset; zsh `HIST_IGNORE_SPACE` and `zshaddhistory` (the line "lingers in the
    history until the next line is executed", which is why zsh's `preexec` still sees it).
  - The 516 scenario's `set_marley` (`516-secret-redaction-for-agents.sh:104-118`) sets a
    `marley.<key>` in the profile's settings; the 484 scenario shows a suggestion's shot.
- **Decisions:** D1 to D5 in the spec.

- **Recall at promotion (2026-09-29):** #556 shipped `type_run` (`\u{15}{command}\r`) and the
  pairing by trimmed command; `Drive.runs` keyed by view. PR-claude-474: frames trust only the
  nonce. #537's `marley_extra_env` serves the agent terminals Marley opens, not every terminal,
  so the variable goes beside the nonce in `TerminalBuilder::new` (`terminal.rs:1231`), which has
  `cx`. Brain: nothing on this seam. Checked on the box: `fc -ln -1` prints a tab and a space
  before the line, so a spaced line shows two spaces; `history -d -1` works in bash 5.3; zsh is
  installed.
- **Seams re-verified:** `marley.bash:61-88` (`__marley_precmd` first in `PROMPT_COMMAND`, `PS0`
  running `$(__marley_preexec)` in a subshell, `fc -ln -0`, leading whitespace trimmed);
  `marley.zsh:62-85` (`precmd_functions`, `preexec_functions`, `$1` untrimmed); the nonce's
  insertion and `AnchoredBlocks::with_nonce` (`terminal.rs:1231`, `:1520`);
  `shell_integration::NONCE_VARIABLE` (`shell_integration.rs:53`); `autosuggest::suggestion`
  (`autosuggest.rs:56`, called with the terminal from the `MarleyTerminalSuggestion` hook and
  `AcceptSuggestion`); `terminal_drive::type_run` and `run_by_agent`.

### Design (at promotion)
- `marley_terminal::shell_integration::AGENT_HISTORY_VARIABLE` (`MARLEY_AGENT_HISTORY`);
  `AnchoredBlocks` gains `agents_out_of_history: bool`, `keep_agents_out_of_history()` and
  `agents_out_of_history()`.
- `terminal.rs`: `pub struct MarleyAgentHistory(pub bool)` (a Global: whether agents' commands
  enter the history; absent is true); in `TerminalBuilder::new`, a local terminal with it false
  gets the variable `0` and its blocks are marked.
- The scripts: bash reads the variable into `__MARLEY_AGENTS_OUT` and unsets it; with it, the
  hooks' block drops a spaced last entry in `__marley_precmd` and takes `ignorespace` out of
  `HISTCONTROL` after the user's file ran. zsh the same variable, `__marley_addhistory` added to
  `zshaddhistory_functions` in `__marley_install`, and `__marley_preexec` trims `$1`.
- `terminal_drive.rs`: `type_run` types `\u{15} {command}\r` when the terminal's blocks keep
  agents out; `Drive` records its terminal's entity id; `agent_blocks(terminal, cx)` gives a
  terminal's agent-run indices.
- `autosuggest.rs`: `suggestion` takes the terminal entity and skips the agent's blocks when the
  terminal keeps agents out.
- Settings: `agent_commands_in_history: Option<bool>` (`settings_content`), `default.json` true,
  `MarleySettings`, the Agents section's toggle; the workbench sets `MarleyAgentHistory` at init
  and on each settings change.
- **File manifest.** Marley: `marley_terminal/src/{shell_integration.rs, anchored.rs}`,
  `marley_terminal/shell_integration/{marley.bash, marley.zsh}`, `marley_workbench/src/
  {terminal_drive.rs, autosuggest.rs, marley_workbench.rs}`. Zed: `crates/terminal/src/terminal.rs`,
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Script: `script/e2e/553-agent-commands-in-history.sh`.

### E2E plan (at promotion)
bash terminal with `HISTFILE` in the scenario's HOME and `PROMPT_COMMAND='history -a'` so the
file is written at each prompt; a zsh terminal (`terminal.shell` set in the run's settings) with
`INC_APPEND_HISTORY`. Steps: the default: `terminal-run repo "echo agent-on"`, `history | tail`
and the file hold it (`553-01`); `ech` typed shows the suggestion `o agent-on` (`553-02`); the
setting off, a new bash terminal, `terminal-run repo "echo agent-off"`: the block and its mark
(`553-03`), `history` and the file lack it (`553-04`), `echo agent-o` typed suggests nothing
from it (`553-05`); the same run in a zsh terminal, its file lacks it (the log); the Agents
section's toggle (`553-06`). REQ-006: a bash terminal opened with the setting on and
`HISTCONTROL` unset keeps a line typed with a leading space (the log).

### Design (as drafted; the promotion's above wins where they differ)
- **The setting.** `agent_commands_in_history: Option<bool>` in `MarleySettingsContent`,
  `true` in `default.json`, `MarleySettings::agent_commands_in_history`, and the page item.
- **The global.** `terminal::MarleyTerminalEnv(pub Vec<(String, String)>)`, a `Global` defined
  in `terminal.rs` beside `Event::MarleyNotification`'s hunk; the workbench sets it at `init` and
  in an `observe_global::<SettingsStore>` (as `refresh_redaction` does) to
  `[("MARLEY_AGENT_HISTORY", "0")]` while the setting is off, else empty. In
  `TerminalBuilder::new`, beside the nonce and for local terminals only, each pair is inserted
  into `env`, and the variable is removed when the global gives none (a value inherited from
  Marley's own environment must not leak in).
- **bash** (`marley.bash`): after the `__MARLEY_HOOKS` guard, `__MARLEY_AGENT_HISTORY=${MARLEY_AGENT_HISTORY-1}`
  and `unset MARLEY_AGENT_HISTORY` (before the user's file, beside the nonce, so no program
  inherits it). In `__marley_preexec`, before the trim: `[ "$__MARLEY_AGENT_HISTORY" = 0 ] &&
  case $line in ' '*) __MARLEY_DROP=1 ;; esac`. In `__marley_precmd`, after the frame: `if
  [ "${__MARLEY_DROP-}" = 1 ]; then case $(HISTTIMEFORMAT='' builtin fc -ln -0 2>/dev/null) in
  ' '*) builtin history -d -1 ;; esac; unset __MARLEY_DROP; fi`, before `return "$status"`.
- **zsh** (`marley.zsh`): the variable taken and unset with the nonce; in `__marley_install`,
  when it is `0`, `zshaddhistory_functions+=(__marley_addhistory)` where `__marley_addhistory()
  { emulate -L zsh; [[ $1 == ' '* ]] && return 1; return 0 }`. `__marley_preexec` trims leading
  whitespace from `$1` as bash's hook trims its line (`${1##[[:space:]]#}`).
- **The typing** (#556's `terminal_run`): `format!("\u{15}{space}{command}\r")` with `space`
  a single space while `MarleySettings::agent_commands_in_history` is false; the pairing trims.
- **The suggestions.** `suggestion` takes the terminal's agent marks (from #556's `Runs` global)
  and, while the setting is false, filters `own` to blocks without a mark.
- **`block_entry`** trims a leading space from `command` (the frame is already trimmed by the
  hooks; this covers a frame from an older script).
- **File manifest.** Marley crates: `crates/marley_terminal/shell_integration/{marley.bash,
  marley.zsh}`; `crates/marley_workbench/src/{autosuggest.rs, terminal_run.rs, mcp.rs,
  marley_workbench.rs}`. Zed paths: `crates/terminal/src/terminal.rs` (the global and the pairs),
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Scripts: `script/e2e/553-agent-commands-in-history.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal.rs` row gains the global and
  the environment pairs; the three settings rows gain the key.

### E2E plan (as drafted)
The scenario runs its steps twice, once with bash and once with zsh (the second `launch_marley`
on the same profile with `terminal.shell` set to zsh in the profile's settings and a `ZDOTDIR`
of the scenario's), and records both shells' history files. `set_marley agent_commands_in_history
false` between the default steps and the "off" steps; a new terminal from the rail's + after it.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `run <id> "echo agent-one"`; type `history 3`, Return | `553-01-default-in-history`; the log: the history file after `history -a` |
| REQ-002 | type `echo ag` | `553-02-default-suggested`: `ent-one` after the cursor |
| REQ-003 | the setting off; a new terminal; `run <id2> "echo agent-two"` | `553-03-off-block`: the block, `echo agent-two`, the pill; the log: `blocks` without a leading space |
| REQ-004 | type `history 3`, Return | `553-04-off-not-in-history`; the log: the file lacks `agent-two`, both shells |
| REQ-005 | type `echo ag` | `553-05-off-not-suggested`: `ent-one` (the file's) shown, `ent-two` never |
| REQ-006 | the setting on; type ` echo mine` (a leading space), Return; `history 2`; then the same with `HISTCONTROL=ignorespace` exported first | the log: listed, then not listed |
| REQ-007 | `marley::OpenSettings` | `553-06-setting` |

Not reachable by a scenario: a shell whose `PROMPT_COMMAND` writes the file before Marley's
entry (Marley's runs first by construction; a user who prepends their own after Marley's is the
edge, recorded).

### Risks
- bash reads the hook's line from history: with the user's own `HISTCONTROL=ignorespace`, a
  space-prefixed line reports the previous command in its `preexec` frame, so the block's
  command is wrong and #556's pairing by text fails (no mark). This is true today for a user's
  own space-prefixed lines; with the setting off it hits every agent command for such users.
  The Plan phase measures it and, if it matters, the pairing falls back to "the first verified
  block after the typing" for the agent's run, recorded in #556's notes.
- `history -d -1` needs bash 5.0 or later for negative offsets; Omarchy ships bash 5.3. An older
  bash logs an error to the terminal once; the script guards with `${BASH_VERSINFO[0]} -ge 5`.
- A user's `PROMPT_COMMAND` entry placed before Marley's by a later `.bashrc` edit would append
  the entry to the file before Marley drops it; the file then keeps what the list dropped.
- The setting's change reaches only new terminals; the description says so, and the
  suggestion filter applies at once everywhere.

## Phase 2 — Code
- **Built to the promoted design.** `shell_integration::AGENT_HISTORY_VARIABLE`;
  `AnchoredBlocks::keep_agents_out_of_history` and `agents_out_of_history`; `terminal.rs`'s
  `MarleyAgentHistory` global, read in `TerminalBuilder::new` before its future, the variable
  inserted beside the nonce and the blocks marked; the two scripts' rules; `type_run` types a
  leading space where the blocks say so and records its terminal's id; `agent_blocks`;
  `autosuggest::suggestion` takes the terminal entity and skips the agent's blocks there; the
  setting in `settings_content`, `default.json`, the Agents section and `MarleySettings`
  (`AgentCommandHistory`, an enum, since clippy's `struct_excessive_bools` counts a fourth bool);
  `terminal_drive::init` keeps the global with the setting.
- **Found in Test, fixed here.** The drop never ran: inside `PROMPT_COMMAND`, `fc -ln -1` skips
  the last entry, taking it for the `fc` command itself, so the check read the line before the
  agent's. Reproduced by hand in a pty (`script -qfc "bash --rcfile marley.bash -i"`) with the
  agent's exact bytes. `history 1` has no such skip: its line after the number and two
  characters is the entry, and a spaced one starts with a space. The first fix stripped the
  spaces after the number with the digits; the second strips them apart. By hand then: both
  spaced lines gone from the list and the file, an empty Enter between them changes nothing,
  `ignoreboth` leaves the frame reporting the agent's line, and without the variable a spaced line
  stays; zsh's rule kept `echo agent-zsh` out of its file with the frame trimmed.
- **Review.** The builder's first draft read the global inside the builder's future, which is
  `Send` and cannot hold `&App`; it moved beside the version read. The rule changes nothing
  without the variable, so a default install behaves as before. With it, bash's own `ignorespace`
  goes, since a line bash never keeps makes `PS0`'s `fc -ln -0` report the one before, and the drop
  does its work; a spaced line loaded from the history file as the last entry is dropped at the
  first prompt, as the setting's description says of spaced lines.
- **Gate.** Run 1: `GATE GREEN [diff]`, 16 passed, 0 failed, the receipt written.

## Phase 3 — Test
- **Scenario** `script/e2e/553-agent-commands-in-history.sh` under `compositor sway`: a bash
  terminal with `HISTFILE` and `PROMPT_COMMAND='history -a'`, a zsh one with
  `INC_APPEND_HISTORY`, the stand-in agent through the plugin's bridge.
- **Run 1** failed at "bash's file lacks it" (the `fc` skip above). **Run 2** passed every check,
  but shots 02 and 05 typed prefixes one character short of the command, whose ghost text the
  block cursor covers; run 3 types shorter prefixes. **Run 3**: all 9 checks passed.
- **Every shot read** (run 2 for 01, 03, 04, 04b and 06, which run 3 did not change; run 3 for 02
  and 05):
  - `553-01-default-in-history`: the marked block `echo agent-on`, then `history | tail -3`
    listing `1 echo agent-on`; the file holds it. REQ-001.
  - `553-02-default-suggested`: `$ echo agen` with the ghost `t-on` past the cursor. REQ-002.
    Before it, ` echo user-spaced` (typed with a leading space, `HISTCONTROL` unset) is in the
    file. REQ-006.
  - `553-03-off-block`: in the terminal opened after the setting went off, `$  echo agent-off`
    typed with its leading space, a block with the mark and the pill; `terminal_blocks` gives
    `'echo agent-off-quiet'`, unspaced, `agent True`. REQ-003.
  - `553-04-off-not-in-history`: `history | tail -3` lists no agent line; the file neither.
    REQ-004 (bash).
  - `553-05-off-not-suggested`: `$ echo agent-of` with no ghost. REQ-005.
  - `553-04b-zsh`: in zsh, the agent's `echo agent-zsh` block and the user's `echo after-zsh`; the
    file holds the second only. REQ-004 (zsh).
  - `553-06-setting`: Agent Commands in History under Marley › Agents, off, with its reset arrow
    and description. REQ-007.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added; `marley_workbench.md` (the run's leading space, `agent_blocks`, the
  global), `terminal_blocks.md` (the flag and both scripts' rules); the touchpoint rows for
  `terminal.rs`, the settings files and `default.json` describe what shipped.
- **Knowledge.** F-claude-553-fc-in-prompt-command-reads-the-entry-before-the-last-001,
  L-claude-553-a-shell-rule-is-checked-by-hand-in-a-pty-first-001. Brain: consulted at Complete
  (the Plan's ask was missed), and the decision recorded with `brain_decide`.
- **Closed** the ticket, archived the pair, committed.
