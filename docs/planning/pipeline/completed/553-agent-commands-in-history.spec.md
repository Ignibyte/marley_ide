---
pipeline_id: bc4baa6a-3b84-4d7e-bc67-ab04b772753a
ticket: docs/planning/tickets/open/TICKET-553-agent-commands-in-history.md
status: Phase 4 — Complete PASS
title: "Whether an agent's commands enter the shell history, as a setting"
type: feature
slice: prong 1 T7 (CLI agents in the terminal) with prong 2 (`terminal_run`, #556); the Warp second pass, "Smaller"
references: [docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/pipeline/queued/556-terminal-run.spec.md, docs/planning/pipeline/completed/484-autosuggestions.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md]
---

## Title
Warp keeps agent-run commands out of the user's history unless a setting lets them in. Marley's
`terminal_run` types into Chad's shell, so its commands land in bash's and zsh's history and in
Marley's autosuggestions, which read the terminal's own commands first. One setting decides both,
and the shell side is done with the convention every shell already has: a leading space.

## Scope
### In
- **The setting** `marley.agent_commands_in_history`, a bool, `true` by default, in the Marley
  page's Agents section as "Agent Commands in History": "Whether commands an agent runs in your
  terminal enter your shell history and Marley's suggestions. Applies to terminals opened after
  a change."
- **Typing.** With the setting off, `terminal_run`'s typing path (#556) sends the command with a
  leading space: Ctrl-U, a space, the command, a return. With it on, as before.
- **The shells** (changed at promotion). Each local terminal's environment carries
  `MARLEY_AGENT_HISTORY=0` when the setting is off at spawn, inserted beside the nonce in
  `TerminalBuilder::new` from a global the workbench keeps from the settings, and the terminal's
  blocks remember it (`AnchoredBlocks::agents_out_of_history`), so `terminal_run` types the
  leading space only where the rule is installed. The scripts take the variable out of the
  environment, as the nonce, and install the rule:
  - bash: `PS0`'s `__marley_preexec` runs in a command substitution, a subshell, so it cannot set
    a flag the prompt sees; `__marley_precmd`, first in `PROMPT_COMMAND`, runs `builtin history
    -d -1` when the last entry (`fc -ln -1`, printed after a tab and a space) starts with a
    space. The hook has read the line by then, and the entry is gone before any `history -a` the
    user's `PROMPT_COMMAND` runs. `ignorespace` leaves `HISTCONTROL` (`ignoreboth` becomes
    `ignoredups`), since a line bash never keeps makes `fc -ln -0` report the one before it; the
    drop does what `ignorespace` did.
  - zsh: `__marley_addhistory` in `zshaddhistory_functions` returns 1 for a line starting with a
    space, so the line is never saved; `preexec` still gets it in `$1`, which the frame now
    trims of leading whitespace as bash's does.
  Without the variable the scripts install nothing new, so the default changes no shell's
  behavior.
- **The blocks.** The block's command and pill are unchanged: the frame carries the line the
  shell read, and the agent's mark (#556 D8) pairs it with the leading space trimmed.
- **The suggestions.** `autosuggest::suggestion` skips the blocks an agent ran in a terminal
  whose blocks keep agents out of history; elsewhere they count as the user's.
- **`terminal_blocks`** reports each block's `command` without the leading space.

### Out (explicitly deferred)
- Applying a change to terminals already open (the shell rule is installed at spawn; Marley
  cannot change a running shell without typing into it).
- Commands the user reruns (Rerun, #474) or sends from a workflow (#558): they are the user's.
- A mark on the pill or in the rail saying "kept out of history".
- The history *file* Marley's suggestions read: with the setting on, an agent's command reaches
  it as any command does; with it off, the shell never writes it.

## Reference (§20)
- **Warp, settings reference** (https://docs.warp.dev/terminal/settings/all-settings/, read
  2026-09-26): `include_agent_commands_in_history`, "Whether agent-executed commands are included
  in command history", boolean, default `false`. Marley keeps the setting and inverts the default:
  Marley's blocks are the user's to copy, rerun and send back, and the agent's block reads like
  any other; Chad can flip it. No Warp code.
- **Upstream Zed:** nothing to keep; Zed's terminal has no view of the shell's history.
- **The shells' own conventions:** bash's `HISTCONTROL=ignorespace` ("lines which begin with a
  space character are not saved in the history list") and `history -d offset` (a negative offset
  counts back from the last entry); zsh's `HIST_IGNORE_SPACE` and the `zshaddhistory` hook
  ("If any of the hook functions returns status 1 … the history line will not be saved, although
  it lingers in the history until the next line is executed"), from the installed manuals.

### Prior art
- **Behavior maps and reports.** The Warp second pass, "Smaller": "Marley's autosuggestions
  read the terminal's own commands first, so an agent typing into Chad's shell would feed them.
  #525, the agent that types into a running program, is where to decide it; its draft does not
  say yet." #525 types into programs, not shells, so the decision moved here, on #556's typing.
- **Published material.** The bash and zsh manuals above. bash expands `PS0` "after reading a
  command and before the command is executed", after the line was added to the history list
  (or dropped by `HISTCONTROL`), which is why the entry can be read and then deleted.
- **The code we already ship.**
  - The scripts: `crates/marley_terminal/shell_integration/marley.bash` (`__marley_precmd`
    `:36-41`, first in `PROMPT_COMMAND` `:54-59`; `__marley_preexec` `:45-52`, whose line comes
    from `fc -ln -0` `:47` with leading whitespace trimmed `:48`; `PS0` `:60`; the `history;file`
    frame `:63-67`) and `marley.zsh` (`__marley_preexec` `:57-62` from `$1`; `__marley_install`
    `:66-79` adding to `precmd_functions` and `preexec_functions`). `shell_integration.rs`: the
    constants (`:20-44`), `new_nonce` (`:48`), `for_program` (`:89`).
  - The spawn: `TerminalBuilder::new`'s nonce hunk (`crates/terminal/src/terminal.rs:1209-1219`,
    local terminals only) and the integration rewrite (`:1223-1227`); `insert_zed_terminal_env`
    (`:716`, `TERM_PROGRAM=zed` at `:721`). The settings store: `SettingsStore::get`
    (`crates/settings/src/settings_store.rs:446`) needs the `Settings` type, which is
    `marley_workbench`'s (`MarleySettings`, `marley_workbench.rs:167`), so the terminal crate
    reads a small global instead.
  - The suggestions: `autosuggest::suggestion` (`autosuggest.rs:56`, own verified blocks newest
    first, then the file), `typed_text` (`:76`), `read_history` (`:120`);
    `marley_terminal::suggestion` and `parse_history` (`suggest.rs:7`, `:24`).
  - The blocks: `AnchoredBlocks::apply`'s `preexec` arm (`anchored.rs:108-128`), `command`
    (`:42`); `terminal_blocks`' `block_entry` (`mcp.rs:477`). #556's mark and typing path.
  - Settings: `MarleySettingsContent` (`settings_content/src/marley.rs:10`), the Agents section
    (`settings_ui/src/marley_page.rs:42`), `default.json`'s `marley` block (`:1665`).
  - Does a crate we build own this seam? The shells own their history; Marley's scripts own the
    hooks; nothing else applies.

## UI proof
UI-AFFECTING: a setting on the Marley page, and what the terminal's blocks and suggestions show.
`script/e2e/553-agent-commands-in-history.sh` (`hyprland` suffices: keys and the stand-in agent).
Fixtures: a scratch repository; the scenario's bash with `HISTFILE` in the scratch HOME and
`HISTCONTROL` unset, and a second run of the same steps under zsh (`ZDOTDIR` the scenario's);
the stand-in agent's `run` from #556. Shots: with the default, `run <id> "echo agent-one"`, the
block, then `history 3` typed, the command listed (`553-01-default-in-history`); `echo ag` typed,
the suggestion `ent-one` after the cursor (`553-02-default-suggested`); the setting set to
false in the profile's settings, a new terminal from the rail, `run <id> "echo agent-two"`, the
block with its command and pill (`553-03-off-block`); `history 3`, `agent-two` absent and the
user's own commands present (`553-04-off-not-in-history`); `echo ag` typed, the suggestion
`ent-one` (from the earlier terminal's history file) and never `ent-two`
(`553-05-off-not-suggested`); the Marley page's item (`553-06-setting`). Machine checks: the
history file after `history -a` (bash) or at exit (zsh) holds `agent-one` and not `agent-two`;
`mcp_agent blocks` gives `echo agent-two` without a leading space.

## Locked-In Decisions
- D1: One setting, two effects: the shell's history and Marley's suggestions. On by default
  (the reasoning in the Reference); Warp's default is off, recorded here so Chad can choose.
- D2: A leading space is the marker, because every shell already treats it as "not for the
  history"; bash needs Marley's own drop since its hook reads the line from the history list,
  and the drop happens in `precmd`, after the hook read it and before the user's own
  `PROMPT_COMMAND` entries run. At promotion: the check is the last entry's own leading space,
  since `PS0` runs in a subshell and no flag survives it; and the rule takes `ignorespace` out of
  `HISTCONTROL`, whose kept-nothing would give the hook the previous line.
- D3: The rule is gated by `MARLEY_AGENT_HISTORY=0` at spawn: with the setting on, Marley's
  scripts change nothing about a user's own space-prefixed lines; with it off, such lines are
  dropped too in terminals opened after the change, which is what `ignorespace` would do and
  what the setting's description says.
- D4: The block keeps its command: the frame carries the line as the shell read it, trimmed of
  leading whitespace as today (`marley.bash:48`; zsh's `$1` is trimmed in the hook the same way).
- D5: The terminal crate reads the setting from a global (`MarleyAgentHistory`, a bool the
  workbench keeps current from `MarleySettings`) beside the nonce, with a `// Marley:` hunk in
  the existing row, and the terminal's blocks keep what it was at spawn, which `terminal_run`
  reads: a leading space only where the shell installed the rule.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.agent_commands_in_history` is true, a command `terminal_run` typed shall appear in the shell's history and in the history file. | Shot `553-01-default-in-history`; the log's file |
| REQ-002 | WHERE the setting is true, the system shall suggest an agent's command as it suggests the user's. | Shot `553-02-default-suggested` |
| REQ-003 | WHERE the setting is false in a terminal opened after the change, a command `terminal_run` typed shall make a block with its command and pill as before. | Shot `553-03-off-block`; the log's `blocks` |
| REQ-004 | WHERE the setting is false, that command shall be absent from the shell's history and from the history file, in bash and in zsh. | Shot `553-04-off-not-in-history`; the log's file, both shells |
| REQ-005 | WHERE the setting is false, the system shall never suggest an agent's command from the terminal's own blocks. | Shot `553-05-off-not-suggested` |
| REQ-006 | WHERE the setting is true, a user's own line typed with a leading space shall be treated as the shell's own settings say, with nothing added by Marley. | The log: the bash run with `HISTCONTROL` unset lists it; with `ignorespace` set it does not |
| REQ-007 | WHEN the Marley page shows, its Agents section shall offer the setting. | Shot `553-06-setting` |
| REQ-008 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; it follows #556 (the typing path and the mark) and reads its notes at
  promotion; ask the brain.
- **P2 Code:** the ledger rows first (`crates/terminal/src/terminal.rs` for the global and the
  environment pair; the settings trio); the scripts' rule; the typing path's space; the
  suggestion filter; `block_entry`'s trim; fmt and clippy clean; a review of the diff, with the
  shell scripts under gate:11's shellcheck.
- **P3 Test:** write and run the scenario, bash and zsh, and read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (T7's status);
  `docs/marley_architecture/marley_terminal.md` (the scripts) and `marley_workbench.md`; the
  ledger capture; close the ticket, archive, commit.
