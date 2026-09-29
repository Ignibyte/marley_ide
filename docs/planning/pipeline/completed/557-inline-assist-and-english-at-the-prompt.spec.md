---
pipeline_id: 0a630590-9f9c-41e3-a44a-6541d94c6b7c
ticket: docs/planning/tickets/open/TICKET-557-inline-assist-and-english-at-the-prompt.md
status: Phase 4 — Complete PASS
title: "Inline Assist proven, and English at the prompt by local rules"
type: feature
slice: prong 1 T3 (the prompt) with prong 2 (a request to an agent); the Warp blocks note, recommendation 4
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/484-autosuggestions.spec.md, docs/planning/pipeline/completed/481-rich-input.spec.md, docs/warp_architecture/crates/natural_language_detection.md, docs/warp_architecture/crates/input_classifier.md]
---

## Title
Zed's terminal Inline Assist is Warp's Generate: Ctrl+Enter, a request, one command on the prompt
line. It is in the tree and no scenario has ever run it in the Marley layout, so this ticket
proves it against a stand-in model. Then the cheaper half of Warp's natural-language detection,
by rules on the machine: a hint when the typed line reads as English, Ctrl+Shift+Enter to hand
that line to the project's agent, and an "Ask the agent" button under a block that ended in
exit 127. Enter never leaves the shell. Chad: "local first and then jev second"; the second
stage is #573.

## Scope
### In
- **The Inline Assist scenario.** `script/e2e/557-inline-assist-and-english-at-the-prompt.sh`
  starts a stand-in Ollama server on localhost (Python, `/api/tags`, `/api/show`, `/api/chat`
  answering one fixed command) and points the profile copy's settings at it
  (`language_models.ollama.api_url`, one `available_models` entry, `agent.default_model` and
  `agent.inline_assistant_model` on it). Ctrl+Enter in the center terminal opens the prompt
  block; a request typed and Enter streams the command onto the prompt line; Ctrl+Enter runs
  it. No Marley code changes for this half unless the run finds one to make; what it finds is
  recorded.
- **The rules**, pure, in `marley_terminal::english`: `read_line(line, is_command) ->
  Reading { Command, English, Blank }`. Blank for an empty or whitespace line. Command when the
  line starts with `#`, `!` or `\`, when its first word holds `=`, or when any word is a shell
  operator or looks like one (`|`, `<`, `>`, `&&`, `;`, `$`, a `-` flag, a glob, a path with
  `/`, `./` or `~`). Otherwise the first word decides: not a command, English (Warp's rule, a
  single word included); a command alone, Command; a command followed by three or more words
  of which one is an English marker (the, a, an, my, me, all, this, that, these, please, what,
  how, why, which, where, is, are, does, do, of, for, with, from, into, in, on, to), English,
  the case of `find all the large files in this repo` and `rm the old build folder`; otherwise
  Command.
- **What counts as a command** (`is_command`): a program on Marley's search path (the one
  `agents::installed_clis` uses), a fixed list of bash and zsh builtins and keywords, or the
  first word of a verified command among the terminal's own blocks (which covers aliases the user
  runs). The search path's names are read once off the main thread and again when the path
  changes.
- **The hint.** While the shell waits at its prompt (the conditions `autosuggest::typed_text`
  checks) and the line reads as English, and no history suggestion applies, the suggestion slot
  after the cursor shows ` · ctrl-shift-enter asks the agent`, dimmed as suggestions are. → never
  takes it: `AcceptSuggestion` recomputes the history suggestion itself. A command shows nothing.
- **`marley::AskAgent`** on Ctrl+Shift+Enter in `Terminal`: with a typed line, Ctrl-U clears the
  shell's line, and the text goes to the agent through #549's targets
  (`send_selection::agent_targets`, the window's agent terminals, the one focused last first):
  one, it is revealed, focused, and gets the text as a paste and a return (`send_text` with a
  return, the rich input's route when it is open); several, #549's `TargetPicker` names them;
  none, Claude Code starts in the project with the text as its first argument
  (`agents::start_cli_with_prompt`, the worktree agents' route). The send refuses with a toast
  while the target's seat waits on a permission (`send_text`'s rule). With nothing typed, the key
  reaches the program. With a block selected, the deeper `MarleyBlockSelected` binding sends the
  block as before.
- **The exit-127 button** (changed at promotion): #555's "Ask the agent" chip before the newest
  failed block's pill, widened. A block that ended with 127, whose command is verified
  (`command_verified`, D6) and reads as English or starts with no command, shows the chip even
  with no agent running, and its click asks with the block's command, as Ctrl+Shift+Enter would;
  any other failed block keeps #555's chip and send. It reaches Zed's element through the chip
  hook (`MarleyBlockChip`), composed in `bookmarks.rs`; no Zed hunk.
- **The setting** `marley.english_hint` (default true) turns the hint and the button off; the key
  stays. It joins the Marley page's Layout section as "English at the Prompt".

### Out (explicitly deferred)
- A network model for the lines the rules leave open: #573 (System One, opt-in, behind #516's
  redaction), and #548's provider. Nothing here waits for it.
- Taking over a plain Enter (the note's open question 3): Enter goes to the shell, always.
- Rich input at a shell prompt (item 3 of the note, T3's first slice) and Warp's `!` and `*`
  prefixes, which need Marley to own the line.
- Send to Agent from a block, and the block menu (recommendations 1 and 2, their own tickets);
  the exit-127 button shares their send route once it exists.
- A picker when several agents run is shared with the selection ticket of the second pass; if it
  lands first, its picker is taken.
- Warp's command denylist for detection (`ai_command_denylist`): a word on the PATH already reads
  as a command.

## Reference (§20)
- **Warp, natural-language detection and Generate**
  (https://docs.warp.dev/agents/cli/input-and-shell-commands/, read 2026-09-26: "Short or
  ambiguous input stays in Agent Mode, and a single word switches to shell mode only when it
  matches a command available in your shell"; https://docs.warp.dev/agents/local-agents/generate/;
  the settings reference: `ai_auto_detection_enabled` true by default, `nld_in_terminal_enabled`
  false by default). The blog post of 2024-06-17 (warp.dev/blog/agent-mode): "Nothing you type in
  the input ever leaves your machine during the natural language detection classification."
  Marley keeps the single-word rule, the on-machine detection, the hint before Enter and the
  separate key; it leaves Enter with the shell where Warp's terminal-input detection is off by
  default. Behavior map: `docs/warp_architecture/crates/natural_language_detection.md` (a lexical
  scorer with word lists, a first-token-is-command signal and shell metacharacters; "gap —
  reimplement the scorer from the public concept") and `input_classifier.md` (the three signals,
  and the one-off list that forces `claude`, `codex` and `gemini` to shell). Research, not
  source; the rules here are Marley's own.
- **Upstream Zed:** Inline Assist in a terminal (`crates/agent_ui`), kept as is: the prompt block
  under the cursor, one command streamed onto the line, Enter to keep it, Ctrl+Enter to run it.

### Prior art
- **Behavior maps and reports.** The Warp blocks note, items 1 and 2 of "What Marley would do",
  recommendation 4, open questions 3 and 4; the jev note's use 1 (detection as a second stage,
  "only for lines the local rules leave open"), which fixes the interface `Reading` leaves for
  #573; the Warp second pass's Chad answer 1 (several agents: a picker). The Warp maps named
  above.
- **Published material.** Warp's docs and blog above. Claude Code takes its first prompt as an
  argument (`claude "<prompt>"`), which #510 also relies on. bash's `HISTCONTROL` and readline
  leave Ctrl-U as the line kill, the byte Rerun and the inline assistant already send.
- **The code we already ship.**
  - Inline Assist: the workspace action `agent_ui::InlineAssistant::inline_assist`
    (`crates/agent_ui/src/inline_assistant.rs:206`; registered in `crates/zed/src/zed.rs:906`
    outside tests), which needs `agent.enabled` and no `disable_ai`, the Agent Panel registered
    (`:226`), and a model whose provider `is_authenticated`; `resolve_inline_assist_target`
    (`:1460`) finds the focused center terminal, so the Marley layout needs nothing of its own.
    `TerminalInlineAssistant::assist` (`terminal_inline_assistant.rs:61`) shows the prompt as a
    block under the cursor (`set_block_below_cursor`, `terminal_view.rs:909`), sends Ctrl-U and
    streams the answer through `TerminalTransaction` (`terminal_codegen.rs:177`, `CLEAR_INPUT`);
    `secondary_confirm` (`inline_prompt_editor.rs:534`) runs it. The binding: `ctrl-enter` →
    `assistant::InlineAssist` in `Terminal` (`default-linux.json:1304`). The template:
    `assets/prompts/terminal_assistant_prompt.hbs`.
  - The stand-in model: the Ollama provider needs no key; `is_authenticated` is "at least one
    model listed" (`crates/language_models/src/provider/ollama.rs:68`), `api_url` from settings
    (`:245`), `default_model` is never a fallback (`:280`), so the settings must name it;
    `stream_chat_completion` posts `/api/chat` and reads NDJSON (`crates/ollama/src/ollama.rs:296`,
    `:303`), `get_models` reads `/api/tags` (`:342`), `show_model` posts `/api/show` (`:377`).
    `agent.inline_assistant_model` (`settings_content/src/agent.rs:268`) and
    `OllamaSettingsContent` (`settings_content/src/language_model.rs:189`). The fake provider
    (`language_model/src/fake_provider.rs`) is test-only and in-process, no route for an e2e.
    Providers authenticate on the first Ctrl+Enter (`inline_assistant.rs`, the
    `ProviderNotAuthenticated` arm) or at once when the native agent builds
    (`crates/agent/src/agent.rs:361`).
  - The line: `autosuggest::typed_text` (`autosuggest.rs:76`) and the suggestion hook
    `MarleyTerminalSuggestion` (`terminal_view.rs:150`), painted at the cursor in the predictive
    color (`terminal_element.rs`, the `marley_suggestion` paint); `AcceptSuggestion` recomputes
    (`autosuggest.rs:36-49`) and propagates with nothing to accept. `marley_terminal::suggestion`
    (`suggest.rs:7`).
  - The send: `rich_input::send` (`rich_input.rs:97`, `paste` then `\r` at `:109-112`);
    `LastTerminal` (`browser.rs:5400`) and `send_pick` (`:3645`, activate, reveal, focus, paste);
    `agent_bar::agent_in` (`agent_bar.rs:129`); `agents::start_cli` (`agents.rs:188`) and
    `installed_clis` (`:85`, `which::which_in` on `launcher(cx).search_path`);
    `marley_agent::send_payload` (`marley_agent.rs:87`); `agent_events::seat` (`agent_events.rs:30`)
    for a waiting Claude Code.
  - The button: `marley_block`'s actions row (`terminal_element.rs:2366`), `ExitCode`
    (`block.rs:41`), `command_verified` (`anchored.rs:45`).
  - PATH lookup: `which` 8 (a workspace dependency); no builtin list and no command-line parser
    exists in the Marley crates (`shlex` splits words). Does a crate we build own this seam?
    `agent_ui` owns Inline Assist whole; the workbench owns the typed line and the send; the rules
    are new and pure.

## UI proof
UI-AFFECTING. `script/e2e/557-inline-assist-and-english-at-the-prompt.sh` (`compositor sway`,
for the block's button). Fixtures: a scratch repository; the scenario's bash; the stand-in Ollama
server started in `setup`, its port written into the profile's settings; a stand-in `claude` on
the PATH that prints what it receives (its first argument, then each pasted line). Shots:
`557-01-inline-prompt` (Ctrl+Enter, the prompt block under the cursor); `557-02-generated`
("print a marker" and Enter, `echo marley-inline-assist` on the prompt line, not run);
`557-03-ran` (Ctrl+Enter, the block with `marley-inline-assist`); `557-04-hint` (`what is using
port 3000` typed, the hint after the cursor); `557-05-no-hint` (`ls -la` typed, nothing after the
cursor; then Ctrl+Shift+Enter with the line still there, the shell's own reaction in the log);
`557-06-asked-new` (`what is using port 3000` and Ctrl+Shift+Enter with no agent running: the
stand-in `claude` starts in a new terminal with the line as its argument, the shell's line
cleared); `557-07-asked-running` (back in the shell, `find all the large files in this repo` and
Ctrl+Shift+Enter: the running stand-in's terminal focused, the line pasted); `557-08-exit-127`
(`show me the biggest folders` and Enter, exit 127, the block's "Ask the agent"); `557-09-button`
(a click, the text in the stand-in); `557-10-off` (the setting false, the same English line, no
hint, the exit-127 block without the button).

## Locked-In Decisions
- D1: Local rules only. No line leaves the machine; a second stage is #573, and `Reading` is the
  interface it fills for the lines the rules leave open (there are none yet: every non-blank
  line reads as one or the other, since a hint that is sometimes wrong costs nothing).
- D2: Enter is the shell's. The agent gets a line only through Ctrl+Shift+Enter or the button.
- D3: A first word that is no command reads as English, a single one included (Warp's rule); a
  command followed by three or more words with an English marker reads as English, because
  `rm the old build folder` runs `rm` on files named `the`, `old`, `build` and `folder`, and the
  hint has to come before Enter. `#` and `!` never read as English (Warp's `#` lesson).
- D4: The hint rides the suggestion slot and never the footer: a footer strip takes rows from the
  grid and would resize the PTY each time the reading flips. A history suggestion wins the slot,
  since a command typed before is a command.
- D5: The target follows the picks' rule: one agent terminal, it; several, a picker; none, Claude
  Code with the line on its command line. The send refuses while the seat waits on a permission.
- D6: The exit-127 button needs a verified command (PR-claude-474): a block whose frames came
  from output gets none.
- D7: `is_command` is built from Marley's search path, the builtins and the terminal's verified
  commands; the shell's aliases and functions are unknown, so an alias typed with English after
  it may read as English. The hint is passive, so the cost is one wrong hint.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Ctrl+Enter in a center terminal in the Marley layout with a model configured, the system shall show the inline prompt under the cursor. | Shot `557-01-inline-prompt` |
| REQ-002 | WHEN the user types a request and presses Enter, the system shall put the model's command on the prompt line without running it. | Shot `557-02-generated` |
| REQ-003 | WHEN the user presses Ctrl+Enter again, the system shall run the command as a block. | Shot `557-03-ran` |
| REQ-004 | WHILE the shell waits at its prompt and the typed line reads as English, the system shall show the hint after the cursor. | Shot `557-04-hint` |
| REQ-005 | WHILE the typed line reads as a command, the system shall show no hint, and Ctrl+Shift+Enter shall reach the program. | Shot `557-05-no-hint`; the log |
| REQ-006 | WHEN the user presses Ctrl+Shift+Enter on a typed line with no agent terminal in the workspace, the system shall clear the shell's line and start Claude Code in the project with the line as its first argument. | Shot `557-06-asked-new`; the stand-in's log |
| REQ-007 | WHEN the user presses Ctrl+Shift+Enter with one agent terminal in the workspace, the system shall clear the shell's line, reveal and focus that terminal and paste the line with a return. | Shot `557-07-asked-running`; the stand-in's log |
| REQ-008 | WHEN a block ends with exit 127 and its verified command reads as English, the system shall show "Ask the agent" on it without hover. | Shot `557-08-exit-127` |
| REQ-009 | WHEN the user clicks "Ask the agent", the system shall send the block's command to the agent as Ctrl+Shift+Enter would. | Shot `557-09-button`; the stand-in's log |
| REQ-010 | WHERE `marley.english_hint` is false, the system shall show neither the hint nor the button, and Ctrl+Shift+Enter shall still send. | Shot `557-10-off` |
| REQ-011 | WHEN the user presses → with the hint shown, the system shall type nothing of the hint. | The log: the typed line after → equals the line before |
| REQ-012 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, run the Inline Assist half first, before any code, since
  it may surface a Marley-layout bug that changes the scope; ask the brain; check whether the
  selection ticket's picker and #556's `MarleyBlockExtras` have landed.
- **P2 Code:** the ledger rows first (`crates/terminal_view/src/terminal_element.rs` if the hook
  is new; the settings trio); `marley_terminal::english`; the command set; the hint in
  `autosuggest`; `marley::AskAgent`, the target rule and the send; the button; the setting; fmt
  and clippy clean; a review of the diff against each REQ.
- **P3 Test:** write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (T3's status) and
  `docs/marley_architecture/` for `marley_terminal` and `marley_workbench`; the ledger capture
  (what the Inline Assist run found); close the ticket, archive, commit.
