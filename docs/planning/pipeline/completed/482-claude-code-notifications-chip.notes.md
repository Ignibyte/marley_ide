# Enable Claude Code notifications: Marley's plugin for Claude Code — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-482-claude-code-notifications-chip.md
- **Pipeline spec:** 482-claude-code-notifications-chip.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23, the first of the five Warp pieces: "There is a 'enable claude
  code notifications' not sure how it actually works but lets explore it". Split from #478 at
  its promotion, when the terminal side and the plugin proved two tickets' work.
- **What the exploration found.** `~/.claude/plugins/installed_plugins.json` is
  `{"version": …, "plugins": {"<name>@<marketplace>": …}}`; `claude plugin marketplace add`
  takes a path. Claude Code 2.1.281's hook answer may carry `terminalSequence`.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall:** the brain (consultation a1e9659fabca4e8cad0dd02eb2fcfd5d) returned only unrelated
  follow-ups. #478 shipped the other half; its lesson on Quickshell serves the live check.
- **Schemas confirmed** against Claude Code's docs: a local marketplace is
  `.claude-plugin/marketplace.json` (`name`, `owner`, `plugins` with a relative `source`); a
  plugin is `.claude-plugin/plugin.json` (`name`, `description`) with `hooks/hooks.json` at its
  root (`{"hooks": {Event: [{matcher, hooks: [{type: "command", command}]}]}}`) and
  `${CLAUDE_PLUGIN_ROOT}` in commands; `Stop` takes no matcher; `claude plugin install` is
  non-interactive with user scope by default, and a running session needs `/reload-plugins`;
  hooks inherit Claude Code's environment and get `CLAUDE_PROJECT_DIR`. `terminalSequence` is not
  in the public docs; it is in Claude Code 2.1.281's hook-answer schema for every event, which
  this ticket relies on (a risk, below). `claude --plugin-dir <path>` loads a plugin for one
  session without installing it.
- **Seams:** the agent bar (#477) and its footer context, which carries the workspace;
  `util::command::new_command` for the spawns, as Zed's clippy bans `std::process::Command`;
  `Workspace::show_toast` and `show_error` for the outcome; `which` for `claude`.

### Design
- `marley_workbench::claude_plugin` (Marley):
  - `write_plugin_in(dir)`: the marketplace, the plugin, `hooks/hooks.json` and an executable
    `hooks/notify.sh`. The script is silent unless `TERM_PROGRAM` is `zed`, and prints
    `{"terminalSequence":"\u001b]777;notify;Claude Code;<project> <message>\u0007"}` for
    `permission` ("needs your permission"), `waiting` ("is waiting for you") and `finished`
    ("finished"), the project being `CLAUDE_PROJECT_DIR`'s last part, stripped of control
    characters, quotes, backslashes and `;`. `hooks.json` wires `permission_prompt`,
    `idle_prompt` and `Stop` to them.
  - `installed_in(config_dir)` and `marketplace_known_in(config_dir)`: `installed_plugins.json`
    lists `marley@marley`; `known_marketplaces.json` lists `marley`.
  - A global `ClaudePlugin` (the marketplace directory under Marley's data directory, Claude
    Code's configuration directory, from `CLAUDE_CONFIG_DIR` or `~/.claude`, `claude` from the
    PATH, and whether the plugin is installed, read in the background at `init`), which tests
    point at scratch directories and a fake `claude`.
  - `install(workspace, cx)`: writes the plugin, runs `claude plugin marketplace add <dir>`
    unless the marketplace is known, then `claude plugin install marley@marley`; on success the
    plugin counts as installed and a toast says to run `/reload-plugins` in a running session;
    a failure goes to `show_error`.
- The agent bar shows the chip for Claude Code while the plugin is known not to be installed,
  and "Installing…" while it installs.

### Test plan
| REQ | Test |
|---|---|
| 001, 003 | driven: a stand-in `claude` in the foreground with no plugin listed shows the chip; with it listed, none |
| 002 | driven: clicking runs a fake `claude` that logs its arguments; the log holds the two commands, the plugin is written, the chip goes; a failing fake leaves the chip and shows the error |
| 004 | unit: the files parse, the script is executable, and `sh notify.sh` answers inside `TERM_PROGRAM=zed` and stays silent outside |
| 005 | `just gate-diff` |

### Risks
- `terminalSequence` could change in a later Claude Code; the live check with `--plugin-dir`
  proves it on 2.1.281, and the notes say what to look for if it stops.

## Phase 2 — Code (2026-09-23)
- **Built.**
  - The plugin as files in the crate, `crates/marley_workbench/claude_plugin/`: the
    marketplace, `marley/.claude-plugin/plugin.json`, `marley/hooks/hooks.json`
    (`permission_prompt`, `idle_prompt`, `Stop`) and `marley/hooks/notify.sh`.
  - `marley_workbench::claude_plugin`: the files by `include_str!`; `write_plugin_in(dir)`;
    `installed_in` and `marketplace_known_in`, over Claude Code's two lists; the global
    `ClaudePlugin` with `init` (Marley's data directory, `CLAUDE_CONFIG_DIR` or `~/.claude`) and
    `set_up` (the list read off the main thread); `install`, which writes the plugin, adds the
    marketplace unless Claude Code knows it, installs the plugin through
    `util::command::new_command`, and shows a toast or the error.
  - The agent bar's chip for Claude Code while the plugin is known not to be installed, and a
    label while it installs.
  - `paths` became a dependency of `marley_workbench`; `script/gates.sh`'s shellcheck gained
    `notify.sh`.
- **Review.** Nothing blocks the main thread: the list is read, and the plugin written, in
  background tasks. The spawns go through Zed's `util::command`. The script is silent outside
  `TERM_PROGRAM=zed` and strips control characters, quotes, backslashes and `;` from the
  project's name. Clippy: two tests took `&mut TestAppContext` unused.

## Phase 3 — Test (2026-09-23)
- **Tests.**
  - `claude_plugin` (unit): `the_plugin_is_a_marketplace_a_plugin_and_an_executable_hook`,
    `claude_codes_lists_say_what_is_installed_and_known`,
    `the_hook_answers_in_marleys_terminals_and_is_silent_elsewhere` (the script run for each
    event, inside `TERM_PROGRAM=zed` and outside), `init_uses_marleys_data_directory_and_claude_codes_own`.
  - `agent_bar` (driven, a stand-in `claude` in the foreground and a fake installer `claude`
    that logs its arguments): `the_chip_installs_marleys_plugin_for_claude_code`,
    `a_marketplace_claude_code_knows_is_not_added_again`,
    `with_marleys_plugin_installed_there_is_no_chip`,
    `a_failed_install_keeps_the_chip_and_says_why`.
  All 16 of the plugin's, the bar's and the notifications' tests pass; clippy clean.
- **Negative checks**, each file restored by sha256:
  1. the chip shown whenever the list has been read: FAIL in the installed test;
  2. the marketplace always added: FAIL, the log held `plugin marketplace add …` too;
  3. the script without its `TERM_PROGRAM` check: FAIL, it answered outside Marley;
  4. the project's name not cleaned: FAIL, the quote broke the JSON;
  5. a failing `claude` taken as success: FAIL, `left: Some(true)`, `right: Some(false)`.
- **Live drive.**
  - `claude -p … --plugin-dir <the plugin>` in a Marley terminal answered "ok", but no
    notification came. A logging copy of the hook showed `Stop` did run, with
    `TERM_PROGRAM=zed`. Claude Code's bundle writes a hook's `terminalSequence` through the
    writer its interactive UI registers, and print mode registers none, so print mode drops it
    (L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001).
  - An interactive `claude --plugin-dir <the plugin> "Reply with the single word ok."`, run in
    a Python pty with `TERM_PROGRAM=zed` in `/srv/stacks`, a folder Claude Code already trusts
    with no project settings of its own, wrote `ESC ] 777 ; notify ; Claude Code ; stacks
    finished` to its terminal once the response ended. With #478's live check, which showed
    Marley posting OSC 777 to the desktop, that is the chain end to end.
  - `OPEN=<a fresh repository> just shot chip-482` with the stand-in `claude`: the bar shows
    "Claude Code" and "Enable Claude Code notifications", since Chad's Claude Code lists no
    `marley@marley`. The click itself installs into Chad's Claude Code and is left to him.
- **The gate's first run was red** on gate:21: Zed's dylint lint `async_block_without_await`,
  an error in the Marley crates, caught the two `background_spawn(async move { … })` blocks
  around the list read and the plugin's writing. A first fix, `smol::unblock`, turned the
  second run red on gate:3: every test that calls the crate's `init` then woke gpui's test
  scheduler from smol's threads, which it reports as non-determinism. The work now goes to
  gpui's background executor as `futures::future::lazy(move |_| …)`, a future made from a
  closure, so there is no `async` block and the tests stay deterministic.
- **Then gate:4 was red twice.** One line, the `else` of `if let Some(parent)` around
  `create_dir_all`, could not happen and became `path.parent().unwrap_or(dir)`. The other was
  the `which` fallback for `claude` inside the install: every test passes its own fake, so the
  closure never ran (`cargo llvm-cov report --json` named it:
  `run_install::{closure#0}::{closure#0}::{closure#0}`), and a test could not run it without
  running the real `claude`. `claude` is now found on the PATH at start, with Claude Code's list,
  unless given, and the `init` test covers that.
- **Gate:** `just gate-diff` over `marley_workbench`, on the fifth run: 20 passed, 0 failed,
  `GATE GREEN [diff]`; 161 tests pass, coverage 100% of lines.

## Phase 4 — Complete (2026-09-23)
- **Docs:** `CHANGELOG.md` (#482, and #478's line that pointed here); the three-prong plan (T7b
  shipped); `docs/marley_architecture/marley_workbench.md` (the plugin). No Zed path changed.
- **Knowledge:** AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001,
  L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001,
  L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001. No F-block.
- **Brain:** consultation a1e9659fabca4e8cad0dd02eb2fcfd5d, decided at Complete.
