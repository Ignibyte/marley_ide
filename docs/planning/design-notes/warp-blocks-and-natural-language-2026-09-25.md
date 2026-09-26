# Warp's block actions and natural-language execution, 2026-09-25

Chad asked the same night: "there is also a 'copy as context' or 'save as workflow' and various
other block commands. Lets research those again. Also there is something really cool where agent
can take natural language and execute code." One agent read Warp's published docs through the
section bundles `docs.warp.dev/llms.txt` lists, its changelog from 2021 to 2026 and two blog
posts. No Warp source was opened (CONSTITUTION §20), and the `warpdotdev/workflows` repository
was left unread, so the YAML workflow fields are not here. Warp offers about fifteen actions on a
selected block; Marley has Copy Output and Rerun on a hovered block, Ctrl-Up and Ctrl-Down to jump
between blocks, and no block selection, so selection comes first. Warp has no action called "copy
as context"; blocks reach its agent through "Attach as context", and Marley's form of that is
Send to Agent (a reference typed into the agent's terminal, as a pick is) beside Copy as
Markdown. Warp's natural-language execution has two parts: a local classifier labels a typed line
English or command before Enter, and an agent runs commands under regex allow and deny lists and
reads their blocks. Zed's Inline Assist already turns English into one command in a Marley
terminal. The piece Marley lacks is `terminal_run` (plan D9), which lets agents run commands in
Chad's terminal as blocks. Only the block filter (#528) and the sticky header (#529) have tickets.

## Warp's block actions, and Marley's

Warp opens a block's menu by right-click or by the kebab that shows on hover, and its keyboard
actions act on the selected block or blocks. Keys are Linux, with macOS in brackets. Sources:
docs.warp.dev/terminal/blocks/ (`block-basics`, `block-actions`, `find`, `block-filtering`,
`background-blocks`, `sticky-command-header`, `block-sharing`) and
docs.warp.dev/getting-started/keyboard-shortcuts/.

| Action | In Warp | Marley today |
|---|---|---|
| Select a block | `Ctrl+↑` (`⌘↑`) selects the newest; the arrows move it; Esc clears | no selection; the same keys scroll to a block's first line |
| Select several | Shift+click, `Shift+↑/↓`, Select All Blocks `Ctrl+Shift+A` | none |
| Copy command | `Ctrl+Shift+C` (`⇧⌘C`) | none; the text is on `AnchoredBlock::command` |
| Copy output | `Ctrl+Shift+Alt+C` (`⌥⇧⌘C`) | Copy on hover, mouse only |
| Copy both | menu | none |
| Reinput | `Ctrl+Shift+I` (`⌘I`): the commands back in the input, not run | none; Rerun types Ctrl-U, the command and a return, for nonce-verified commands at a prompt |
| Reinput as root | the same with `sudo`; unbound on Linux | none |
| Toggle bookmark | `Ctrl+Shift+B` (`⌘B`); `Alt+↑/↓` to jump; markers on the scrollbar | none |
| Find within block | `Ctrl+Shift+F` (`⌘F`), with regex and case toggles | Zed's terminal search over all scrollback, regex only, no case toggle |
| Filter block | `Alt+Shift+F`: only matching lines, nothing deleted | #528 |
| Scroll within block | `Ctrl+Shift+↑/↓` to its top or bottom | block starts only |
| Sticky command header | a setting | #529 |
| Share | `Ctrl+Shift+S`: a public link on Warp's servers | none |
| Save as Workflow | menu | none |
| Attach as context | the sparkles icon; `Ctrl+↑` in the agent's input | agents read blocks over MCP (`terminal_blocks`, `terminal_read`); Chad cannot hand one over |
| Clear blocks | `Ctrl+Shift+K` (`⌘K`) | Zed's Clear, `Ctrl+Shift+L` |
| Background blocks | output from `&` jobs gets a block with no command | that output belongs to no block |

Right-click in a Marley terminal opens Zed's terminal menu (`TerminalView::deploy_context_menu`
in `crates/terminal_view/src/terminal_view.rs`): New Terminal, Copy, Paste, Select All, Clear,
Inline Assist, Add to Agent Thread (with a text selection) and Close Terminal Tab. None of it
knows blocks. The once-over counted blocks as context done because agents read blocks over MCP.
That gave agents the read and gave Chad no way to point an agent at a block, which is what "copy
as context" asks for.

## What Marley would take from the block actions

**Block selection (M), first because every keyboard action needs it.** In Warp, `Ctrl+↑` selects
the newest block and takes the focus from the input (`preserve_input_focus_on_block_selection`
keeps it there). Marley would keep a selection of block indices per terminal view in
`marley_workbench::blocks`, in a global keyed by the view's entity id as rich input keeps its
editors, and draw an outline around the selected block beside the existing block decorations.
Ctrl-Up and Ctrl-Down would select and scroll instead of only scrolling (open question 6). While
a block is selected the view carries a key context (say `MarleyBlockSelected`), so Marley's
keymap sends ↑, ↓, Esc, Enter and the action chords to the selection; any typed character drops
the selection and goes to the shell. `Terminal::select_matches` can also make the block's lines
the text selection, so Zed's own Copy, Add to Agent Thread and Inline Assist work on a block with
no new Zed code. The hard part is that ↑ and ↓ belong to readline, fzf and every TUI: the mode is
entered on purpose, ends the moment Chad types, and never exists on the alternate screen. Zed
binds `Ctrl+Shift+B` to the outline panel in the `Workspace` context, so Warp's bookmark key
needs a `Terminal`-context binding to win. No ticket.

**The block menu (S once selection exists).** A hook in `deploy_context_menu`, filled by the
workbench the way its terminal footer and suggestion hooks are, adds a Block section when the
right-click lands on a block's rows; `marley_block_spans` already maps rows to blocks. Items: Copy
Command, Copy Output, Copy Both, Copy as Markdown, Send to Agent, Save as Workflow, Reinput,
Reinput with sudo, Rerun, Find in Block, Filter Block (#528) and Toggle Bookmark. Reinput is
Rerun's path without the return and, like Rerun, is offered only for commands the shell hook
reported with the terminal's nonce (#474). It needs one touchpoint row for `terminal_view.rs`. No
ticket.

**"Copy as context": Send to Agent and Copy as Markdown (S each).** Warp's docs have no action by
that name. Selected blocks become pending context when a conversation starts, commands run inside
a conversation are its context, `Ctrl+Shift+L` sends an editor or review selection to a running
Claude Code or Codex (2026-04-01), and after a failed command the hint line offers "attach
'<command>' output as agent context"
(docs.warp.dev/agents/local-agents/agent-context/blocks-as-context/). In Marley, Zed's Add to
Agent Thread reaches an Agent Panel thread or the panel's own terminal, never Claude Code in a
center terminal. Marley would add:

- Copy as Markdown: a fenced block with `$ <command>` and the output, then one line with the cwd,
  the branch, the exit code and the duration, all on `AnchoredBlock` and `BlockTimes` already.
  When the output has left the scrollback, the copy says so.
- Send to Agent: types a reference into the agent terminal used last, the way a pick's Send does
  (`send_pick` and `LastTerminal` in `crates/marley_workbench/src/browser.rs`), for example
  `[terminal 42 block 7: cargo build, exit 101; terminal_read terminal=42 block=7] `, and opens
  rich input for the question. Short output can go inline as the Markdown instead, which also
  works where the plugin's bridge is missing. The target is a terminal whose foreground process is
  an agent CLI, never the block's own terminal.
- For an Agent Panel thread: select the block and dispatch `AddSelectionToThread`.
- An "Ask the agent" chip under a failed block, Marley's form of Warp's hint.

`terminal_read` takes the view's entity id, which changes each launch; #520's
`MARLEY_TERMINAL_ID` would make references stable across launches. #516's redaction runs before
a block leaves Marley, and both sends refuse while the target agent waits on a permission prompt,
as #508 refuses picks. No ticket.

**Save as Workflow (M).** In Warp it is reached from the block menu, agent results, Generate
results, Warp Drive and the Command Palette. The editor takes a name, the command and arguments
written `{{name}}`, each text or an enum (fixed, or taken from a shell command's output) with a
description and a default, and AutoFill asks an agent for the title, descriptions and parameters.
Running a workflow puts the command in the input without running it, and Shift+Tab walks its
arguments. Workflows sync through Warp Drive and export to YAML; file workflows still load from
`<repo>/.warp/workflows/` (docs.warp.dev/knowledge-and-collaboration/warp-drive/workflows/,
docs.warp.dev/terminal/entry/yaml-workflows/). The fork has nothing yet, but the gpui-era app had
workflows, and its pure module (`crates/marley_app/src/workflows.rs` in the gpui-era tree, 297
lines: `Workflow`, `substitute`, `params_of`, `ParamPrompt`) is Marley's own code and can move
over unchanged. Marley would put it in `marley_terminal`, add a Zed `Picker` and the parameter
modal in `marley_workbench`, and type the filled command at the shell prompt with no return, so
Chad reads it before Enter. Rules guess parameters first (paths that exist, the current branch,
numbers, URLs, quoted strings). Storage is Chad's call (open question 2): Marley YAML files under
`~/.config/marley/workflows/` and `<project>/.marley/workflows/` keep parameters, while Zed's
`tasks.json` cannot prompt for a value (task variables are context such as `$ZED_WORKTREE_ROOT`).
The hard parts are typing into readline only at a prompt, and multi-line commands. No ticket.

**Bookmarks and find within block (S each).** Bookmarks are a set of block indices per terminal,
`Alt+↑/↓` to jump (Zed's `Terminal` context leaves those keys unbound, though a program may read
them), a mark on the block's pill and markers on the scrollbar; they end with the session, as
Warp's do. Find within block limits Zed's terminal search to the selected block by filtering
matches on its absolute line range, one touchpoint in `terminal_view.rs`. No ticket.

## Natural language to execution in Warp

**Detection.** Warp ships a local classifier: "Nothing you type in the input ever leaves your
machine during the natural language detection classification" (warp.dev/blog/agent-mode,
2024-06-17). It labels the line before Enter, "(autodetected)" beside the input in terminal mode,
and Enter on a line labelled as a request starts a conversation. The Warp Agent CLI docs state
the rule: "Short or ambiguous input stays in Agent Mode, and a single word switches to shell mode
only when it matches a command available in your shell"
(docs.warp.dev/agents/cli/input-and-shell-commands/). `Ctrl+I` flips the mode, `!` at the start
forces shell and `*` forces agent, and `Ctrl+Shift+Enter` opens a conversation with the selected
blocks attached. `#` at the start opened AI command search, now called Generate (Legacy), which
turns a description into candidate commands to run or save as a workflow
(docs.warp.dev/agents/local-agents/generate/). Against false positives Warp keeps a denylist of
commands never read as English (`ai_command_denylist`), a first-use banner that offers to turn
detection off, and two switches (`ai_auto_detection_enabled`, and `nld_in_terminal_enabled`, off
by default). The changelog shows where it hurt: `#` shell comments opened AI search until a
setting could stop it (2026-08-27), a follow-up that turns out to be a command switches back
(2025-03-05), and an attached image locks agent mode (2026-05-06).

**After Enter.** The agent answers in an AI block. A suggested command lands in the input for the
user to run, and running it sends nothing back to the model. A requested command waits for an
explicit Enter; then the command and its output go to the model, and on a failure the agent
requests another until the task is done (docs.warp.dev/terminal/input/classic-input/). In today's
agent view those commands are blocks that belong to the conversation, `/plan` opens a versioned
plan, a task list tracks the steps, and Full Terminal Use lets the agent drive psql or a dev
server behind a takeover control.

**Permissions.** Each agent profile sets each kind of action (apply diffs, read files, create
plans, execute commands, interact with running commands, ask questions) to Agent Decides, Always
ask, Always allow or Never (docs.warp.dev/agents/capabilities/agent-profiles-permissions/).
Commands also pass two regex lists. The allowlist runs without asking; the settings reference
gives `cat`, `echo`, `find .*`, `grep`, `ls` and `which .*` as its default, though the profiles
page calls it empty. The denylist always asks and beats both the allowlist and Agent Decides; its
default is the shells (`bash`, `fish`, `pwsh`, `sh`, `zsh`) and `curl`, `eval`, `exec`, `source`,
`wget`, `dig`, `nslookup`, `host`, `ssh`, `scp`, `rsync`, `telnet` and `rm`. Read-only commands
can run unasked (`agent_mode_execute_readonly_commands`, 2025-01-29). Auto-approve runs everything
until the task ends, denylisted commands included unless `auto_approve_bypasses_command_denylist`
is off, and a team admin's denylist is never bypassed. In the CLI an approval card also takes
guidance instead of yes or no, and `E` edits the command first
(docs.warp.dev/agents/cli/permissions-and-profiles/).

**Around it.** Next Command suggests the next command from the session and history (2024-12-19),
Prompt Suggestions offer an agent prompt with the newest block attached, Command Corrections apply
the rules of the open-source thefuck project, and Command Inspector documents each part of the
typed command. Today's docs describe no Explain action.

## What Marley would do

Warp owns its input editor, so it classifies a line before any shell sees it. Marley's terminals
type straight into readline. The prompt editor (plan T3 and D4) is not built, so by the time Chad
presses Enter the line belongs to the shell. A line reaches an agent only if Marley catches Enter
at a prompt, or gives the line to a Marley editor first.

1. **Inline Assist, already in the tree (S to prove).** `Ctrl+Enter` in a center terminal opens
   Zed's inline prompt. Its model gets the OS, the shell, the cwd, the last 50 non-empty lines and
   the request (`assets/prompts/terminal_assistant_prompt.hbs`) and streams one command into the
   prompt line; confirming inserts it, and the execute variant also sends the return
   (`crates/agent_ui/src/terminal_codegen.rs`, `terminal_inline_assistant.rs`). This is Warp's
   Generate, and it should work in the Marley layout wherever a Zed model is set up. No e2e
   scenario covers it yet, and no ticket.
2. **English at the prompt, by local rules (S as a hint, M with Enter routing).** `typed_text()`
   in `crates/marley_workbench/src/autosuggest.rs` already reads what was typed. Rules label it
   with no network: a first word that is not on `PATH`, not a builtin and not the first word of a
   verified command reads as English; pipes, redirects, `--flags`, `=`, `$` and paths read as a
   command; a single word goes to the shell only when it is a command, the rule Warp's CLI
   publishes. A chip at the end of the prompt row says where the line would go. Enter stays with
   the shell, and `Ctrl+Shift+Enter` sends the line to an agent after clearing the shell's line
   with Ctrl-U, as Rerun does. When bash or zsh report exit 127 for an English-looking line, a chip
   under the failed block offers "Ask the agent". Taking over a plain Enter comes last, if ever
   (open question 3), because a bug there eats commands, and a `#` prefix would collide with shell
   comments as it did in Warp. The hard middle is a real command followed by English: "find all
   the large files in this repo", "kill the dev server", "rm the old build folder". The last one
   would run `rm` on files named `the`, `old`, `build` and `folder`, so the check has to come
   before Enter. No ticket.
3. **Rich input at a shell prompt (M).** Ctrl-G opens rich input only while an agent runs. At a
   shell prompt the same editor would let Marley own the text before the shell does: Enter
   classifies, then types the line and a return into the shell or sends it to an agent. That is
   Warp's input in small, and a first slice of T3. No ticket.
4. **`terminal_run`, an agent running commands in Chad's terminal (M).** Plan D9 reserves
   `terminal.run` behind a grant. Under a new grant class, `terminal.write`, in
   `crates/marley_mcp/src/registry.rs` and `permission.rs`, it would take a terminal id and a
   command, and when that terminal waits at a nonce-verified prompt it would type Ctrl-U, the
   command and a return, wait for the block to end (the next Precmd) or for a timeout, and return
   the exit code, the duration and the output under `terminal_read`'s caps (2,000 lines, 256 KiB).
   The block's pill carries an agent mark. Claude Code's own prompt for each tool call stays the
   first approval. Marley adds Warp's two regex lists with Warp's defaults, shows a denied command
   as a card in that terminal (Enter runs it, Esc refuses), and has a takeover key that makes
   `terminal_run` refuse until Chad hands back, as #525 does for typing into a running program.
   Then Claude Code, Codex and the Zed Agent each get Warp's loop, and every command they run is a
   block Chad can copy, rerun or send back. The prompt state has to be exact, text Chad has half
   typed must survive (refuse while the input shows typing), a long command needs a timeout that
   returns "still running" with the output so far, and output an agent reads can carry
   instructions aimed at it, so the lists and the agent's own prompts remain the gate. No ticket;
   #525 leaves `terminal.run` to a later one.
5. **Where a request goes, and who is driving.** Inline Assist takes "a command that does X". A
   longer request goes to the agent terminal used last as a pasted prompt, to a new Claude Code in
   the project with the request as its first prompt on argv (a paste can race the TUI's start,
   which is why #510 puts the first prompt on the command line), or to a Zed Agent thread. Claude
   Code runs its own Bash tool in its own subprocess, so those commands never become Marley blocks;
   `terminal_run` is what makes them blocks. Marley shows who is driving with an Agent chip in the
   terminal like the Browser tab's, the agent mark on each block it ran, and the rail row's second
   line ("Claude Code · running cargo test in terminal 3").

## Where a System One model fits

A typed-decision model such as TypeSafe's Jev can make the yes-or-no and pick-one calls around
these features; it cannot write a command. It could judge whether a typed line is a command, a
request, a comment or a command followed by English (only for lines the local rules leave open,
asked after about 250 ms without typing so Enter never waits on the network); which running agent
should take a request; whether it is one command for Inline Assist or a job for an agent; which
lines of a block's output matter; whether to offer "Ask the agent" under a failed block; which
tokens of a command are workflow parameters, with names from a fixed list; and how risky an agent
command is, as a second signal on the card while the lists stay the gate. Warp's classifier never
sends a typed line off the machine and Jev would, so detection by Jev stays off until Chad turns
it on, runs behind #516's redaction, or moves to a local model. The costs, the rules and the other
uses are in `jev-system-one-2026-09-25.md`.

## Left out

Share (public permalinks on Warp's servers), Warp Drive sync, and Next Command and Prompt
Suggestions, which send session content to a model all the time.

## Recommendations, ranked

1. Block selection, the block menu, and its copy and reinput items: Copy Command, Copy Both, Copy
   as Markdown, Reinput, Reinput with sudo. M, then S. Every other block action needs the
   selection.
2. Send to Agent and the "Ask the agent" chip under a failed block. S each.
3. `terminal_run` with Warp's list defaults, the agent mark and a takeover key. M. It is the
   "agent takes natural language and executes" piece, for every agent Marley hosts.
4. An e2e scenario that proves `Ctrl+Enter` Inline Assist (S); then English detection by local
   rules, hint only, with `Ctrl+Shift+Enter` and the exit-127 chip (S to M).
5. Save as Workflow from the gpui-era module. M.
6. A System One model as the second stage of detection, opt-in, once the local rules exist and
   behind #516's redaction; then for trimming output and for workflow parameters.
7. Bookmarks and find within block. S each.

## Open questions for Chad

1. By "copy as context", did you mean Warp's Attach as context (the block goes into the agent's
   prompt) or a clipboard copy shaped for pasting into any agent? The note proposes both, as Send
   to Agent and Copy as Markdown.
2. Workflows as Marley YAML files (`~/.config/marley/workflows/`, `<project>/.marley/workflows/`)
   or as entries in Zed's `tasks.json`? Should Marley import Warp's exported YAML?
3. May Marley ever send a plain Enter to an agent when it is confident, or does Enter always go to
   the shell, with a separate key for the agent?
4. May the typed lines the local rules cannot decide go to TypeSafe, with redaction and opt-in, or
   should detection stay on the machine as Warp's does?
5. For `terminal_run`, is Claude Code's own prompt enough, or should Marley also ask for every
   command outside its allowlist, as Warp does by default?
6. Should Ctrl-Up and Ctrl-Down stay "scroll to block", or become "select block and scroll" as
   Warp binds them?

## Sources

Warp's published docs, read 2026-09-25; URLs given inline are not repeated. Also read under
https://docs.warp.dev/: `terminal/appearance/blocks-behavior/`, `terminal/settings/all-settings/`,
`knowledge-and-collaboration/warp-drive/` (import and export), `terminal/entry/command-search/`,
`agents/local-agents/interacting-with-agents/terminal-and-agent-modes/`,
`agents/local-agents/agent-context/selection-as-context/`, `agents/local-agents/active-ai/`,
`terminal/input/universal-input/`, `terminal/entry/command-corrections/`,
`terminal/editor/command-inspector/`, `agents/capabilities/planning/`,
`agents/capabilities/task-lists/`, `agents/capabilities/full-terminal-use/`,
`agents/cli-agents/rich-input/` and `changelog/` (2021 to 2026); and the blog post
https://www.warp.dev/blog/introducing-the-warp-agent-cli-coding-agent (2026-08-04). Marley and Zed
code read: `crates/marley_workbench/src/{blocks.rs,rich_input.rs,autosuggest.rs,browser.rs,mcp.rs}`,
`crates/marley_terminal/src/{anchored.rs,block.rs}`, `crates/terminal/src/terminal.rs`,
`crates/terminal_view/src/{terminal_view.rs,terminal_element.rs}`, `crates/agent_ui/src/`,
`crates/marley_mcp/src/`, `crates/task/src/task.rs`, and the gpui-era `workflows.rs`.
