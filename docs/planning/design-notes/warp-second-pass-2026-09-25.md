# Warp second pass, 2026-09-25

Chad asked, after the once-over: "look at both orca and warp again just to make sure we didnt
miss anything else cool. dont stretch though." One agent read Warp's published docs section by
section (docs.warp.dev, through the section files its `llms.txt` lists) and the changelog from
2026.06.24 to 2026.09.16, and checked each candidate against the once-over note, `CHANGELOG.md`,
the three-prong plan, the backlog and the Orca survey. No Warp source was opened (CONSTITUTION
§20). Block actions and natural-language execution have their own note,
`warp-blocks-and-natural-language-2026-09-25.md`. Three things came back that Marley has not
planned and would use every day with several agents running: one key that sends the editor's
selection to the agent in a terminal; a question before a close or a quit ends a working agent;
and a command's end seen from outside its terminal, as a notification and on the rail row. Each
is about a day of work and none is ticketed. The rest of Warp's docs describes Warp's cloud,
Warp's own agent, or features Marley has shipped or queued.

## 1. Send the editor's selection to the agent in a terminal

**What it is:** With Claude Code, Codex, OpenCode or another CLI agent running in a tab, Warp
puts a selection from its code editor or its Code Review panel into that agent's prompt with
Ctrl+Shift+L (Cmd+L on macOS): the relative path, the line numbers and the selected text, not
submitted, so the user types the question after it. The CLI-agent table marks "Attach code as
context" for all ten agents it lists, and a hovered diff hunk in Code Review goes the same way.

**Where it came from:** docs.warp.dev/agents/local-agents/agent-context/selection-as-context/
and docs.warp.dev/agents/cli-agents/overview/.

**Marley today:** Zed's Add to Agent Thread (`agent::AddSelectionToThread`, `ctrl->`, and the
item in the editor's and the terminal's right-click menus) quotes a selection into an Agent Panel
thread and nowhere else. Attach File types whole paths from a file chooser. Picks from the Browser
tab already type a reference into the agent terminal used last (`LastTerminal` in
`crates/marley_workbench/src/browser.rs`), which is the routing this needs.

**What Marley would do:** An action in `marley_workbench`, `marley: send selection to agent`,
and in the Marley layout Zed's `ctrl->` sent to the agent terminal when a CLI agent was the agent
used last. For Claude Code it types the reference form Claude Code's JetBrains plugin inserts,
`@src/auth.ts#L1-99` (code.claude.com/docs/en/jetbrains, "File reference shortcuts"), as one
bracketed paste with no Enter and with the path relative to the agent's working directory; with
no selection it sends `@path`. While rich input is open, the reference goes there instead. The
other CLIs get `@path` with the range written out until each one's syntax is checked.

The pull side can follow. Plan D9 reserves `editor.open`, `editor.goto` and `editor.diff` on
Marley's MCP server; `editor_selection` and `editor_diagnostics` beside them would let any agent
ask what Chad has selected and what the language servers report. Claude Code's own IDE protocol
is the fuller route (code.claude.com/docs/en/ide-integrations). The CLI finds a loopback
WebSocket through a lock file the IDE writes, sends the selection and the active file with every
prompt, opens its diffs in the IDE and calls `mcp__ide__getDiagnostics`. Its internal calls are
documented by purpose only, so that route is L and uncertain.

**Hard parts:** Picking the target when three agents run in one project. Paths relative to the
agent's directory, which for a worktree agent (#510) is not the editor's project root. A paste
into an agent that shows a permission prompt answers that prompt, so the send refuses while the
agent waits, the rule #508 sets for picks. Warp's Ctrl+Shift+L is Zed's `editor::SelectAllMatches`
on Linux, so Marley keeps `ctrl->`.

**Size and ticket:** S for the key and the send to the agent used last; M with hunks from the
diff view and a picker for projects running several agents. No ticket.

## 2. Ask before a close or a quit ends a working agent

**What it is:** Warp warns before a quit or a window close while a session runs a process, with
Yes quit, Show running processes (the session palette filtered to running sessions), Cancel and
Don't ask again. Since 2025.01.08 it also asks before closing one session with a long-running
process (`should_confirm_close_session`, on by default). A closed tab can be reopened for 60
seconds (`[general.undo_close]`, `grace_period = 60`, Ctrl+Alt+T on Linux, since 2023.08.03). On
2026.07.03 Warp stopped blocking a system logout, shutdown or OS update while a process runs.

**Where it came from:** docs.warp.dev/terminal/more-features/quit-warning/,
docs.warp.dev/terminal/windows/tabs/ (Tab Restoration),
docs.warp.dev/terminal/settings/all-settings/ and docs.warp.dev/changelog/.

**Marley today:** Nothing asks. Zed counts a terminal as dirty only while a task runs in it or its
bell is unread (`TerminalView::is_dirty` in `crates/terminal_view/src/terminal_view.rs`). Closing
items asks about unsaved files only; quit (`Workspace::prepare_to_close`) serializes and exits;
`confirm_quit` is off by default and, when on, asks "Are you sure you want to quit?" with nothing
about what runs. Zed's Reopen Closed Item reopens by path, so a closed terminal cannot come back.
Ctrl+Shift+W in Claude Code's terminal, or a click on its tab's close button, ends that agent
mid-turn, and the restart that picks up a new `just install` build ends every agent and every dev
server. Session resume (#540) brings a conversation back; the turn in flight and the servers are
lost.

**What Marley would do:** One check in `marley_workbench` lists, per window, the terminals whose
agent is working or whose last block is still running. The rail's Close (`close_terminal` in
`crates/marley_workbench/src/rail.rs`) and the tab close ask when the terminal is on that list
("Claude Code is working in marley_ide. Close it?"). Quit and window close show one dialog that
names them all, with a Show button that filters the rail to them. The undo is a second step:
Marley keeps a closed terminal's `Terminal` entity, and with it the PTY, alive for 60 seconds,
and Reopen Closed Item puts it back in a new view.

**Hard parts:** The tab close and quit paths are Zed's, so each needs a small hook and a row in
`docs/marley/zed-touchpoints.md`. The dialog must never hold up a logout or a shutdown, the bug
Warp fixed. "Working" judged from two seconds of quiet misreads an agent that thinks silently;
#519's hook events fix that. A held PTY keeps producing output with no view attached and must
still die at its deadline.

**Size and ticket:** S for the questions, M with the undo. No ticket. The Orca second pass found
the same gap from Orca's side and ranks it first (`orca-second-pass-2026-09-25.md`, finding 1),
so one ticket can take both.

## 3. A command's end, seen from outside its terminal

**What it is:** Warp posts a desktop notification when a command finishes after a threshold
(`long_running_threshold`, 30 seconds by default, on by default) or when a running command waits
for a password, and only while Warp is not the app in front; the notifications date from
2022.05.26. Its tab bar marks a tab whose command exited with an error, its vertical tabs can
title a row with the last command, and its session palette finds sessions by running command,
last command and status ("Running…", "Completed 10 minutes ago").

**Where it came from:** docs.warp.dev/terminal/more-features/notifications/,
docs.warp.dev/terminal/appearance/tabs-behavior/, docs.warp.dev/terminal/windows/vertical-tabs/,
docs.warp.dev/terminal/sessions/session-navigation/, and `[notifications.preferences]` in
docs.warp.dev/terminal/settings/all-settings/.

**Marley today:** Blocks carry the command, exit code, start and end (`Terminal::blocks()`), and
only the terminal's drawing and the MCP tools read them. Desktop notifications come from a
program's OSC 9 or OSC 777 (`crates/marley_workbench/src/notifications.rs`) and from the Claude
Code plugin's hooks. A rail terminal row holds a title, a subtitle, the bell and the agent
(`TerminalSnapshot` in `crates/marley_rail/src/marley_rail.rs`), and nothing about the command. A
`cargo build`, a test run or a dev server asks for no notification, so none comes.

**What Marley would do:** Beside `notifications.rs`, Marley watches each terminal's blocks. When
one ends after the threshold (a `marley` setting, 30 seconds by default, 0 for off) while its
view is not the focused one in the active window, Marley posts a notification titled with the
command that says "exit 1 after 4m 12s" or "done in 45s", through the notify and click path it
already has. A terminal row's second line shows the running command, or the last one's result
with a red mark on a failure, and the rail filter matches the command text.

A password prompt shows in the PTY's own flags. While the foreground process is not the shell, a
PTY in canonical mode with echo off means a program is reading a password (sudo, ssh, gpg). Zed's
`ProcessIdGetter` already holds the PTY master and calls `tcgetpgrp` on it
(`crates/terminal/src/pty_info.rs`); a `tcgetattr` beside it reads the flags, since Linux answers
TCGETS on a master with the slave's settings. The row then says "waiting for a password" and one
notification goes out.

**Hard parts:** A dev server never finishes, so its useful signal is a failure printed while it
runs, which takes judgment (use 7 in `jev-system-one-2026-09-25.md`). A Claude Code session is
one long block that ends when `claude` exits, so agent terminals keep their own banners (#538)
and skip this one. Reading termios costs a syscall, so it belongs with the existing
foreground-process check, not on every read of output. Blocks stop at the first `ssh` until
#526 lands, though the password check still sees ssh's own local prompt.

**Size and ticket:** S for the notification, the row and the filter; a few hours more, and one
Zed touchpoint in `crates/terminal`, for the password check. No ticket. #538 words Claude Code's
banners and #542 orders the rail; this finding serves plain commands.

## Smaller

- **Notification setup for Codex and OpenCode.** Warp's agent bar installs each one's
  notifications in one click: a Warp plugin for Codex (`codex plugin marketplace add
  warpdotdev/codex-warp`; the notifications page still gives the older
  `notification_condition = "always"` under `[tui]` in `~/.codex/config.toml`) and
  `"@warp-dot-dev/opencode-warp"` in OpenCode's `plugin` array
  (docs.warp.dev/agents/cli-agents/codex/, docs.warp.dev/agents/cli-agents/opencode/,
  docs.warp.dev/agents/capabilities/agent-notifications/). Marley's agent bar offers only
  "Connect Claude Code to Marley". If Codex's own notifications arrive as OSC 9, Marley shows them
  already and a chip only has to set the config. S, after checking what Codex prints in a Marley
  terminal. No ticket.
- **Agent commands in the user's history.** Warp has a setting for whether commands an agent runs
  enter the user's history (`include_agent_commands_in_history`, on the all-settings page).
  Marley's autosuggestions read the terminal's own commands first, so an agent typing into Chad's
  shell would feed them. #525, the agent that types into a running program, is where to decide
  it; its draft does not say yet.

## Ruled out

Section by section, the rest is covered or not worth taking. The input editor, completions and
the fzf and atuin handoff are plan T3 and T6. The prompt's chips are the agent bar's folder and
branch. Themes, system mode and inactive-pane dimming are Zed's; the error mark on a tab is part
of finding 3. Tab configs are #527 and vertical-tab metadata is #531; tab colours, groups, pins
and the global hotkey are low value on Hyprland, which has special workspaces. Scrollback and
blocks after a relaunch are in the Orca survey (report 05, item 3), with the conversation itself
in #540. Warp Drive, notebooks, environment variables and their secret-manager imports stay out
of scope, and workflows are in the blocks note. Warp's SSH extension is Zed's remote development,
and blocks over SSH are #526. Warp's own agent (profiles, planning, task lists, rules, skills,
memory, forking, cloud handoff) has no place here, since the agents Marley hosts bring their own.
Code review is #511 and #522. The MCP server manager, which imports Claude Code's and Codex's
servers, would help only Zed's agent. Launchers are `marley <path>` and the URL scheme #445
settles; the Slack, Linear and GitHub Actions integrations are Warp's cloud. Secret redaction is
#516, telemetry is off by default, and a network log is low value. The changelog from 2026.06.24
to 2026.09.16 is mostly cloud (Oz, Factories); its local items are the fzf and atuin handoff and
native completions (T3, T6), tab groups and pins, detection of more CLIs (Grok, omp, agy: one
entry each in `marley_agent` if Chad runs them), a Codex plugin (under Smaller), Kitty keyboard
fixes, `warpctrl` (Marley's MCP server does that job) and the logout fix in finding 2.

## Recommendations, ranked

1. Finding 1, the selection to the agent (S). Chad would use this gesture most, and it reuses the
   picks' routing with nothing to wait for.
2. Finding 2's questions (S), without the undo. A new `just install` build takes effect only
   after a restart, and today that restart ends every agent's turn. Take the undo only if stray
   closes keep happening.
3. Finding 3 with the password check (S).
4. The Codex check under Smaller, if Chad runs Codex in Marley's terminals.

## Open questions for Chad

1. With several agents in one project, where does a selection go: the agent terminal used last
   (the picks' rule), or a small picker? Default: the one used last.
2. In the Marley layout, should `ctrl->` go to the terminal agent when it was used last, instead
   of Zed's Agent Panel? Default: yes.
3. Closing a terminal whose agent is working: ask, or close it and hold it 60 seconds for Reopen?
   Default: ask.
4. The long-command threshold: Warp's 30 seconds, for plain commands only and never for agent
   terminals? Default: yes to both.

## Sources

Warp, published docs only:

- https://docs.warp.dev/agents/local-agents/agent-context/selection-as-context/
- https://docs.warp.dev/agents/cli-agents/overview/,
  https://docs.warp.dev/agents/cli-agents/codex/, https://docs.warp.dev/agents/cli-agents/opencode/
- https://docs.warp.dev/agents/capabilities/agent-notifications/
- https://docs.warp.dev/terminal/more-features/quit-warning/,
  https://docs.warp.dev/terminal/windows/tabs/
- https://docs.warp.dev/terminal/more-features/notifications/,
  https://docs.warp.dev/terminal/appearance/tabs-behavior/,
  https://docs.warp.dev/terminal/windows/vertical-tabs/,
  https://docs.warp.dev/terminal/sessions/session-navigation/
- https://docs.warp.dev/terminal/settings/all-settings/ (`[general]`, `[general.undo_close]`,
  `[notifications.preferences]`)
- https://docs.warp.dev/changelog/ (2022.05.26, 2023.08.03, 2025.01.08, 2026.06.24 to 2026.09.16)
- Section files: https://docs.warp.dev/llms.txt

Claude Code, for the reference format and the IDE protocol:
https://code.claude.com/docs/en/jetbrains and https://code.claude.com/docs/en/ide-integrations.

Marley and Zed code read: `crates/marley_workbench/src/{browser.rs,rail.rs,notifications.rs}`,
`crates/marley_rail/src/marley_rail.rs`, `crates/terminal_view/src/terminal_view.rs`,
`crates/workspace/src/workspace.rs`, `crates/zed/src/zed.rs`, `crates/terminal/src/pty_info.rs`,
`assets/keymaps/default-linux.json`.

## Chad's answers, 2026-09-26
1. With several agents, a selection goes through a picker, not to the one used last.
3. Closing a terminal, or quitting, while an agent works asks first. The prompt names the working
   agents; an idle agent closes without asking; the undo that holds a closed agent for 60 seconds
   stays out.
   Revised the same morning: both. It asks first, and a closed working terminal is also held for a
   configurable number of seconds so an accidental close can be undone ("allows stopping of
   accidental. so both").
