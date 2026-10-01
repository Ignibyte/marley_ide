---
pipeline_id: 376fe55c-41ee-428b-9552-3e454faf4218
ticket: docs/planning/tickets/open/TICKET-573-english-at-the-prompt-second-stage.md
status: Phase 4 — Complete PASS
title: "English at the prompt, second stage: a System One reading for the lines the rules leave open"
type: feature
slice: prong 1 T3 (after #484 and #557, the local rules); the System One layer's typed-line use, on #565
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/queued/557-inline-assist-and-english-at-the-prompt.spec.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/484-autosuggestions.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/warp_architecture/subsystems/04-agent-ai-mcp.md, docs/warp_architecture/crates/input_classifier.md, docs/warp_architecture/crates/natural_language_detection.md]
---

## Title
For a typed line #557's local rules decide only by their weakest rule (a command's name followed
by plain words: `find all the large files in this repo`, `kill the dev server`, `rm the old build
folder`), Marley asks #565's layer, through the `typed_line/1` set, whether the line is a command,
a request, a comment, or a command followed by English, and shows the reading after the line:
only after 250 ms without typing, never in Enter's path, only for a listed project, masked, and
never for a line holding a candidate secret. In `act` the dangerous middle (`rm the old build
folder`) is marked in the warning colour before Enter. Since #627 the line is typed in the shell's
prompt editor by default, so #557's hint and Ctrl+Shift+Enter first move into that editor.

## Scope
### In
- **#557 in the prompt editor** (the ground this stage stands on): the editor's line is read by
  #557's rules as the grid's is; a line that reads as English shows #557's hint after the text, in
  the predictive colour, as an inlay; Ctrl+Shift+Enter in the editor asks the agent with the
  editor's line and empties it. With the editor closed (`marley.prompt_editor` off, or Escape),
  the grid's slot works as #557 shipped it.
- **The open case** (`marley_terminal::english::open_case(line, is_command) -> bool`, pure,
  beside `read_line`): the first word is a command, plain words follow, at least one of them one
  of #557's English markers (so `git status` and `ls src` cost no call), and no word is shell
  syntax (#557's `looks_like_shell`; a line starting with `#`, `!` or `\`, or a first
  word holding `=`, is never open). #557's reading stays the leaning shown until a reading
  arrives; every other line is settled by #557 alone.
- **The watch** (`marley_workbench::typed_line`, new): per terminal, the line in front (the
  editor's text while the shell's editor is open, else `autosuggest::typed_text`); each change
  re-arms a 250 ms timer; when it fires on an open line still in front, the layer is asked once
  with that exact text; a reading is shown only while the line still equals the text asked about,
  and any change clears it.
- **The set** `typed_line/1` and the use `TYPED_LINE` (`marley_system_one`): one choice, `kind`:
  `command`, `request`, `comment`, `command_then_english`, `cannot_tell`; a 600 ms deadline; #557's
  reading handed as the use's verdict.
- **The state** (D5): the line as masked text; facts computed in code: the first word when it is
  a known command and where it is known from (the search path, a builtin, this terminal's
  commands), the word count, the count of #557's marker words, whether the shell's history holds a
  command starting with the line, the last block's exit code, and the project's name.
- **The words shown** (the slot in the grid, the inlay in the editor): `request` gives ` · a
  request, ctrl-shift-enter asks the agent`; `comment` gives ` · a comment`;
  `command_then_english` gives ` · English after <word>, ctrl-shift-enter asks the agent`; in
  `suggest` the class carries a `?`, in `act` it does not; `command` in `act` hides #557's hint,
  and in `suggest` leaves it. A history suggestion still wins the grid's slot (#557's D4).
- **The warning** (`act`, `command_then_english`, the editor): the words after the first drawn in
  the theme's warning colour, so what the command would take as arguments shows before Enter.
- **Ctrl+Shift+Enter** asks the agent for a line #557 reads as English, as before, and also for a
  line whose shown reading offers it (`request`, `command_then_english`); Enter never changes.
- **Outcomes** for the day's file, one row each: `entered` (a block opens with the line), then
  `exit N` when that block ends; `asked the agent`; `edited` (the line in front became another
  non-empty line); `cleared` (emptied at the same prompt); `dropped` (a reading that came after
  the line changed).
- **Settings:** `marley.system_one.uses.typed_line` (`off` by default) in `default.json`, and its
  dropdown, Typed Line, in the Marley page's System One section.
- `script/e2e/573-english-at-the-prompt-second-stage.sh`.

### Out (explicitly deferred)
- Taking over Enter, or any change to where Enter sends a line: the Warp note's open question 3
  is Chad's, and "taking over a plain Enter comes last, if ever" (#557's D2).
- The warning colour in the grid's slot: it needs the Zed hook's answer type to grow
  (`MarleyTerminalSuggestion`, `terminal_element.rs`); the grid shows the words alone.
- History ghost text in the prompt editor (#627's follow-up).
- The note's other typed-line questions (which running agent takes a request; one command for
  Inline Assist or a job for an agent; which output lines matter; workflow parameters).
- A local model provider: #565's; this use takes whatever provider the layer has.
- fish (#466), the alternate screen, vi mode, an agent's own prompt in the editor.

## Reference (§20)
- **Warp:** its natural-language detection runs a local classifier before Enter, marks the input
  "(autodetected)", switches a single word to shell mode only when it is a command available in the
  shell, keeps a denylist and two switches (`ai_auto_detection_enabled`, `nld_in_terminal_enabled`
  off by default), and promises that "Nothing you type in the input ever leaves your machine
  during the natural language detection classification" (warp.dev/blog/agent-mode,
  docs.warp.dev/agents/cli/input-and-shell-commands/, as `warp-blocks-and-natural-language-2026-09-25.md`
  and #557 record them). The behavior maps `docs/warp_architecture/crates/input_classifier.md`
  (an ONNX model with a heuristic fallback, `InputClassifierDecisionSource` recording which path
  decided) and `natural_language_detection.md` (a word-list scorer), AGPL crates "to reimplement
  from the public concept". Marley's first stage (#557) is the heuristic; this stage is the
  model, which Warp runs on the machine and Marley runs through #565, opt-in and redacted until a
  local provider exists (Chad's rule). Nothing of Warp's was read.
- **Upstream Zed:** Inline Assist in a terminal (`crates/agent_ui/src/terminal_inline_assistant.rs`,
  the prompt `assets/prompts/terminal_assistant_prompt.hbs`), which #557 proves and this ticket
  leaves untouched.

### Prior art
- **Behavior maps and reports.** The Warp note: "English at the prompt, by local rules" (the
  rules, the hint, Ctrl+Shift+Enter, the exit-127 button, "the hard middle is a real command
  followed by English"; `rm the old build folder` "would run `rm` on files named `the`, `old`,
  `build` and `folder`, so the check has to come before Enter"), "Where a System One model fits"
  (this ticket's sentence), the changelog's lessons (`#` comments read as AI search until a
  setting stopped it; a follow-up that turns out to be a command switches back) and its open
  questions 3 and 4. #557's D1 ("`Reading` is the interface it fills for the lines the rules
  leave open (there are none yet: every non-blank line reads as one or the other, since a hint
  that is sometimes wrong costs nothing)"), D3 (the marker rule that reads a command followed by
  three or more words with an English marker as English) and D4 (the hint rides the suggestion
  slot, never the footer). The Jev note: "English at the prompt ... read what nothing sends today,
  so they run on a local model or stay off; a candidate secret is never sent out to ask about
  it"; the interactive budget (200 to 400 ms, p99 near 550 ms at 8k tokens); Laya as a CPU model
  with a 512-token state. #565's layer (`UseSpec`, `ask`, `StateBuilder`, `Reading`, the
  providers, the day's file).
- **Published material.** TypeSafe's docs as the note cites them (a choice with a "none" option;
  "Describe situations, not degrees"; the state's text-only reading). fish's and zsh's
  autosuggestion timing, which shows ghost text at each keystroke with no network, the bar this
  stage must not lower.
- **The code we already ship.** `autosuggest::typed_text`
  (`crates/marley_workbench/src/autosuggest.rs:76-105`: the cells from `input_start` to the
  cursor, the main screen, no vi mode) and the suggestion hook installed at `:31-34`
  (`MarleyTerminalSuggestion`, `crates/terminal_view/src/terminal_view.rs:150`, run at each draw,
  painted at the cursor in the predictive color by the `marley_suggestion` hunk of
  `terminal_element.rs`); `read_history` (`autosuggest.rs:120-149`: `cx.spawn`,
  `background_spawn(lazy)`, then `refresh_windows`: the pattern for an off-thread result that
  redraws); `AnchoredBlocks::at_prompt`, `note_input` and `input_start`
  (`crates/marley_terminal/src/anchored.rs:176-192`), `Terminal::input` noting the input's start
  (`crates/terminal/src/terminal.rs:2316-2326`), `AnchoredBlock::command_verified`
  (`anchored.rs:44-45`) for the verified-command fact; #557's queued `english::read_line`,
  `is_command` (the search path's names, the builtins, the verified commands) and its hint in the
  slot; `cx.background_executor().timer` as a re-armed one-shot (`rail.rs:364`, `:431`); `which`
  8.0.5 (`voice.rs:66`) for the PATH fact; `mcp::agent_redactor` (`mcp.rs:297`) and
  `Redactor::redact`'s count (`crates/marley_mcp/src/redact.rs:117-166`); `rich_input::send`
  (`rich_input.rs:97-116`, the paste and `\r` #557's Ctrl+Shift+Enter path uses). Does a crate we
  build own the seam? `marley_terminal` owns the prompt state, #557 the rules, the command set and
  the hint, #565 the ask; nothing owns a second opinion.

- **Re-verified at promotion (2026-09-30).** #557 shipped `marley_terminal::english::read_line`,
  `looks_like_shell`, `MARKERS` and `BUILTINS` (`crates/marley_terminal/src/english.rs`), and the
  workbench's `english::hint`, `is_command`, `ask_typed` and `ask`
  (`crates/marley_workbench/src/english.rs`); the hint rides `MarleyTerminalSuggestion`
  (`terminal_view.rs:273`) after the history suggestion (`autosuggest.rs:31-38`). #565's sets and
  uses live in `crates/marley_system_one/src/marley_system_one.rs` (no `question.rs`), the ask is
  `system_one::ask(spec, &Asking, cx) -> Task<Asked>`, outcomes are `system_one::outcome(call, ..)`,
  and the mask is applied by the layer (`state_for`, `model_redactor`). #572's `running_errors.rs`
  is the newest use and the template. #627 docks the shell's editor at every prompt, where
  `typed_text` sees nothing and Zed's `Editor && mode == auto_height` binds Ctrl+Shift+Enter to
  `editor::NewlineBelow`, so #557's hint and key do not reach the editor today.
- **Zed's editor, for the editor's surface.** `Editor::splice_inlays` with
  `Inlay::edit_prediction` (`crates/editor/src/inlays.rs`) draws text after the line in the
  suggestion style every editor mode gets (`make_suggestion_styles`); a reserved id keeps it apart
  from Zed's own. `Editor::highlight_text` colours a range; underlines are filtered out in an
  editor without diagnostics (`show_underlines`), so the warning is a colour, not a squiggle.
  `highlight_inlays` is crate-private, so the inlay keeps one style.

## UI proof
UI-AFFECTING: the hint after the line in the prompt editor and in the grid's slot, the warning
colour, the Decisions view and the Marley page's dropdown.
`script/e2e/573-english-at-the-prompt-second-stage.sh` (`compositor sway`). Fixtures: bash with a
plain prompt and `$E2E_WORK/bin` first on the PATH, holding an `rm` that logs its arguments and
removes nothing; #565's layer on `replay` with the scratch repository listed; the replay file
(`find all the large files in this repo`: request 0.90; `kill the dev server`: request 0.85; `rm
the old build folder`: command_then_english 0.93, repeated). Shots:
- `english`: the use `off`; `kill the dev server` typed in the docked editor: #557's hint after it.
- `open-line`: `suggest`; `find all the large files in this repo`, 1 s: ` · a request? ctrl-shift-enter asks the agent`.
- `typing-cancels`: `kill the dev`, then ` server` at once, 1 s: the reading for the whole line;
  the day's file holds no call for `kill the dev`.
- `command`: `ls -la`: nothing after the line, no call.
- `secret`: `curl <a token assembled at run time>`: no call, nothing from the model.
- `entered`: `rm the old build folder` and Enter in the same breath: the block, the fake `rm`'s log
  holding the four words, no call for it.
- `act-warning`: `act`; the same line left 1 s: ` · English after rm, ctrl-shift-enter asks the
  agent`, with `the old build folder` in the warning colour.
- `grid`: Escape, the same line typed at the shell's own prompt: the words in the grid's slot.
- `decisions`: `marley: open decisions`: the `typed_line` rows with their readings.
- `unlisted`: the repository off the list: a new open line, no call, no reading.
- `settings`: the Marley page's System One section showing Typed Line.
Checks on the day's file: the rows name `typed_line/1`, carry the line masked and the facts, and
the outcome rows `cleared`, `edited`, `entered` and `exit 0` follow their calls.

## Locked-In Decisions
- D1 — Enter never waits: the ask runs "only after about 250 ms without typing so Enter never
  waits on the network". The timer, the call and the reading live beside Enter's path, never in
  it; Enter is the shell's, as #557's D2 leaves it.
- D2 — "local first and then jev second" (Chad, 2026-09-26): #557's rules label every line, and
  only the open case is asked. With the `rules` provider, the project unlisted, the provider
  unreachable or no signal, what shows is #557's alone: only a model's reading is drawn.
- D3 — Off by default, with its own mode in `marley.system_one.uses` and no provider named
  (Chad: "we need probably every aspect of this configurable and turned off / on ... Otherwise
  this becomes a jev required system"). The line leaves only for a listed project, masked; a line
  in which the redactor finds anything never leaves.
- D4 — The reading is words and a colour: it marks and never acts. It never presses a key and
  never changes Enter; Ctrl+Shift+Enter, a key the user presses, follows what the hint offers.
- D5 — The state is the masked line and facts computed in code (PR-claude-a-state-fact-holds-only-what-code-computed-001):
  the first word only when code knows it as a command; never output lines or files. A
  metadata-only project sends the facts alone.
- D6 — `cannot_tell`, a confidence under 0.5 (#565's floor), no signal, a refusal or an
  unavailable provider shows nothing.
- D7 — 250 ms of quiet, a 600 ms deadline, no retry, one call per settled line, and a reading bound
  to the exact text it was asked about; a late one is dropped and logged as such.
- D8 — Modes as #565 defines them: `off` asks nothing; `shadow` logs and shows nothing beyond
  #557's hint; `suggest` shows the words with `?`; `act` shows them plain, colours the middle's
  English words and lets `command` hide #557's hint.
- D9 — The prompt editor is the default input (#627), so the reading's main surface is the
  editor: an inlay after its text, in the editor's suggestion style, which needs no Zed change;
  the grid keeps #557's slot with words only.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shell's prompt editor holds a line #557 reads as English, the editor shall show #557's hint after the line. | Shot `english` |
| REQ-002 | WHERE the use is not `off` and the project listed, WHEN a line in the open case has not changed for 250 ms, the system shall ask the `typed_line/1` set once with the masked line and the facts. | The day's file after `open-line` |
| REQ-003 | WHERE the mode is `suggest`, WHEN the reading names a class at or above the floor, the line's hint shall show it with a `?`. | Shot `open-line` |
| REQ-004 | WHEN the line changes within 250 ms, the system shall make no call for the earlier line. | Shot `typing-cancels`; the day's file |
| REQ-005 | WHEN #557's rules settle the line outside the open case, the system shall make no call. | Shot `command`; the day's file |
| REQ-006 | WHEN the redactor finds a candidate secret in the line, the system shall make no call and show nothing from the model. | Shot `secret`; the day's file |
| REQ-007 | WHEN Enter is pressed before a reading arrives, the shell shall receive the line at once, and no reading shall show. | Shot `entered`; the fake `rm`'s log; the day's file |
| REQ-008 | WHERE the mode is `act`, WHEN the reading is a command followed by English, the editor shall name the first word in the hint and draw the words after it in the warning colour, before Enter. | Shot `act-warning` |
| REQ-009 | WHILE the shell's editor is closed, the grid's slot shall show the same words. | Shot `grid` |
| REQ-010 | WHEN the Decisions view opens, it shall list each `typed_line` call with its reading. | Shot `decisions` |
| REQ-011 | WHERE the project is not on the allow list, or the mode is `off`, the system shall make no call. | Shots `unlisted`, `english`; the day's file |
| REQ-012 | WHEN a line asked about is entered, sent to an agent, edited or cleared, the system shall log the call's outcome. | The day's file's outcome rows |
| REQ-013 | The system shall never change where Enter sends a line. | Review of the diff; shot `entered` |
| REQ-014 | WHEN the Marley settings page opens, its System One section shall show the use's mode. | Shot `settings` |

## Phase Plan
- **P1 Plan:** this spec, re-bound at promotion to what #557, #565 and #627 shipped; the design and
  the visual check plan in the notes.
- **P2 Code:** the ledger rows first (`assets/settings/default.json`,
  `crates/settings_content/src/marley.rs`'s `uses` doc, `crates/settings_ui/src/marley_page.rs`);
  `english::open_case`; the set and the use; `typed_line`; #557 in the editor; the words; the
  outcomes. fmt and clippy clean; the gate green.
- **P3 Test:** the scenario, every shot read, the day's file checked.
- **P4 Complete:** CHANGELOG; the guide; `marley_workbench.md`, `terminal_blocks.md` and
  `marley_system_one.md`; the plan's T3 row; knowledge; close the ticket, archive, commit, push.
