---
pipeline_id: 376fe55c-41ee-428b-9552-3e454faf4218
ticket: docs/planning/tickets/open/TICKET-573-english-at-the-prompt-second-stage.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "English at the prompt, second stage: a System One reading for the lines the rules leave open"
type: feature
slice: prong 1 T3 (after #484 and #557, the local rules); the System One layer's typed-line use, on #565
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/queued/557-inline-assist-and-english-at-the-prompt.spec.md, docs/planning/pipeline/queued/565-system-one-layer.spec.md, docs/planning/pipeline/completed/484-autosuggestions.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/warp_architecture/subsystems/04-agent-ai-mcp.md, docs/warp_architecture/crates/input_classifier.md, docs/warp_architecture/crates/natural_language_detection.md]
---

## Title
For a typed line #557's local rules decide only by their weakest rule (a command's name followed
by plain words: `find all the large files in this repo`, `kill the dev server`, `rm the old build
folder`, `echo what is this`), Marley asks #565's layer, through the `typed_line` set, whether the
line is a command, a request, a comment, or a command followed by English, and shows the reading
in #557's hint slot after the cursor: only after 250 ms without typing, never in Enter's path,
only for a listed project, through #516's redactor, and never for a line holding a candidate
secret. The dangerous middle (`rm the old build folder`) gets a warning before Enter. Enter and
Ctrl+Shift+Enter stay what #557 makes them.

## Scope
### In
- **The open case** (`marley_terminal::english::open_case(line, is_command) -> bool`, pure,
  beside #557's `read_line`): true when the line's first word is a command, the line has two or
  more words, and no word is shell syntax (#557's operators, flags, globs and paths); #557's
  `Reading` stays the leaning shown until a reading arrives. Every other line is settled by #557.
- **The trigger** (`marley_workbench::prompt_line`, new, beside `autosuggest`): whenever the
  typed text at a prompt changes (`autosuggest::typed_text`, read where the suggestion hook already
  runs), a 250 ms timer per terminal view is re-armed; when it fires with an open line, the layer
  is asked once for that exact text; a reading is shown only while the typed text still equals the
  text asked about, and any change clears it and re-arms.
- **The set** `typed_line/1` (in `marley_system_one::question`): a `choice` (command, request,
  comment, a command followed by English, cannot tell); the state of D5; the use registered with
  #565 (`UseSpec { name: "typed_line", deadline: 600 ms }`), the use's own verdict (#557's
  reading) handed with each ask.
- **The hint** (#557's suggestion slot after the cursor, dimmed): a request adds #557's own hint
  (` · ctrl-shift-enter asks the agent`) where the rules showed none; a command shows nothing; a
  comment shows ` · a comment`; a command followed by English shows ` · English after <word>,
  ctrl-shift-enter asks the agent`, painted in the warning color; in `suggest` the added words
  carry a `?`, in `act` they do not, and the warning color needs `act`. The confidence shows in
  the Decisions view, never in the grid. Nothing changes Enter or Ctrl+Shift+Enter, and a history
  suggestion still wins the slot (#557's D4).
- **Deadlines**: 600 ms for the call, no retry; a call whose text has changed by the time it
  answers is dropped and logged as such; no call for a text the layer has asked about already
  (its dedupe by hash).
- **Secrets**: a line in which #516's `Redactor` finds anything is never sent, and the slot shows
  nothing from the model for it.
- **Outcomes** for the day's file: what happened to the line: Enter (with the block's exit code
  once it ends: 127 says the shell rejected it), Ctrl+Shift+Enter (sent to an agent), edited away,
  or cleared.
- **Settings**: the use's mode in `marley.system_one.uses` (`off` by default), its dropdown on the
  Marley page's System One section; #557's `marley.english_hint` must be on, since the reading
  rides its slot.
- One Zed touch: the suggestion paint in `crates/terminal_view/src/terminal_element.rs` (the
  `marley_suggestion` hunk of #484) takes a warning variant, so the middle's hint can be painted
  in the warning color; its row exists and widens.
- `script/e2e/573-english-at-the-prompt-second-stage.sh`.

### Out (explicitly deferred)
- Taking over Enter, or any change to where a line goes: the Warp note's open question 3 is
  Chad's to answer, and "taking over a plain Enter comes last, if ever" (#557's D2).
- The note's other typed-line questions (which running agent takes a request; one command for
  Inline Assist or a job for an agent; which output lines matter; workflow parameters).
- A local model provider: #565's; this use takes whatever provider the layer has.
- Lines typed in rich input (a Zed editor already) and in fish; the alternate screen; vi mode.
- Warp's denylist of commands never read as English: #557's rules own it.

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

## UI proof
UI-AFFECTING: #557's hint slot after the cursor, and the Decisions view.
`script/e2e/573-english-at-the-prompt-second-stage.sh` (`compositor sway`: the Decisions view's
row is clicked). Fixtures: bash with a plain prompt and `$E2E_WORK/bin` first on the PATH,
holding a `rm` that logs its arguments and removes nothing; #557's hint on; #565's layer enabled
on `replay` with the scratch repository listed and `uses.typed_line` rewritten per step by
`system_one_setting`; the replay file at `$E2E_PROFILE/system_one/replay.jsonl` (`find all the
large files in this repo`: a command followed by English, 0.90; `kill the dev server`: request,
0.85; `rm the old build folder`: a command followed by English, 0.93); #557's `read_line` as it
ships. Shots:
- `573-01-open-line`: the mode `suggest`; `find all the large files in this repo` typed, 1 s: the
  slot reads ` · English after find? ctrl-shift-enter asks the agent`.
- `573-02-typing-cancels`: `kill the dev` typed, then ` server` within 200 ms, 1 s: one row in the
  day's file for the whole line and none for the prefix.
- `573-03-command`: `ls -la`: nothing after the cursor, no call.
- `573-04-secret`: `export TOKEN=<a value assembled at run time>`: no call; the slot as #557
  leaves it.
- `573-05-enter-never-waits`: `rm the old build folder` typed and Enter pressed inside 100 ms:
  the block starts, the fake `rm`'s log holds the four words, and the day's file has no row for it
  (the timer never fired) or a row marked dropped.
- `573-06-act-warning`: the mode `act`; the same line typed and left: the slot's warning
  ` · English after rm, ctrl-shift-enter asks the agent` in the warning color.
- `573-07-decisions`: `marley: decisions`: the row `typed_line · repo · replay · a command
  followed by English 0.93` with its masked state.
- `573-08-unlisted`: the repository taken off the allow list: the same line, no call, no reading.
- `573-09-off`: the mode `off`: the same line, no call, #557's slot alone.
Checks: the day's file's rows name `typed_line/1`, carry the line redacted and the facts, and no
output lines; each has its outcome line (`enter`, `agent`, `edited`, `cleared`).

## Locked-In Decisions
- D1 — Enter never waits: the ask runs "only after about 250 ms without typing so Enter never
  waits on the network" (Chad's brief for this ticket, from the Warp note). The timer, the call
  and the reading live beside Enter's path, never in it; Enter is the shell's, as #557's D2 leaves
  it.
- D2 — "local first and then jev second" (Chad, 2026-09-26): #557's rules label every line, and
  only the open case, the rules' weakest branch, is asked. With the `rules` provider, the project
  unlisted (`Refused`), the provider unreachable (`Unavailable`) or the reading `NoSignal`, the
  slot is #557's alone.
- D3 — Off by default, with its own switch and its own mode, and no provider named: "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system" (Chad). The switch is the use's mode
  in `marley.system_one.uses` (#565's shape). "Opt-in, redacted, or a local model": the line
  leaves only for a listed project, through the redactor as the layer's `Mask`; a line with a
  candidate secret never leaves; a local provider is #565's and this use takes it.
- D4 — The reading is words in the hint slot and a color: it marks and never acts. No keystroke,
  no route, no Enter (the note's rule: a System One answer may mark, never approve; the Warp
  note's open question 3 stays Chad's).
- D5 — The state is the typed line and facts computed in code: the first word (on the PATH, a
  builtin, a verified command of this terminal's blocks, a history match), the word count, the
  presence of flags, pipes, redirects, paths, `=` and `$`, the shell, the last block's command
  and exit code, and the project's name; never output lines or files; `Detail::Facts` for a
  metadata-only project, which makes the reading nearly useless, so the Decisions view says so.
- D6 — The choice carries `cannot_tell`; `cannot_tell`, a confidence under the floor (0.5,
  #565's starting point), `NoSignal`, `Refused` or `Unavailable` shows nothing.
- D7 — 250 ms of quiet, a 600 ms deadline, no retry, one call per settled text, and a reading
  bound to the exact text it was asked about (the note's rule 5 applied to a line: a reading for
  an older text is dropped).
- D8 — Modes as #565 defines them: `off` asks nothing; `shadow` logs and shows nothing beyond
  #557's hint; `suggest` shows the words with `?`; `act` shows them plain and the warning color
  for the middle. The outcome line (Enter and the exit code, the agent, edited, cleared) is the
  label the golden report is fitted on.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is not `off` and the project listed, WHEN a typed line in the open case has not changed for 250 ms, the system shall ask the `typed_line` set once with the redacted line and the facts. | The day's file: one row for `573-01-open-line` |
| REQ-002 | WHERE the mode is `suggest`, WHEN the reading names a class at or above the floor, the hint slot shall show it with a `?`. | Shot `573-01-open-line` |
| REQ-003 | WHEN the typed text changes within 250 ms, the system shall make no call for the earlier text. | Shot `573-02-typing-cancels`; the day's file: one row |
| REQ-004 | WHEN #557's rules settle the line outside the open case, the system shall make no call. | Shot `573-03-command`; the day's file |
| REQ-005 | WHEN the redactor finds a candidate secret in the line, the system shall make no call and show nothing from the model. | Shot `573-04-secret`; the day's file |
| REQ-006 | WHEN Enter is pressed before the reading arrives, the shell shall receive the line at once, and the reading, if any, shall be dropped. | Shot `573-05-enter-never-waits`; the fake `rm`'s log; the day's file |
| REQ-007 | WHERE the mode is `act`, WHEN the reading is a command followed by English, the slot shall show a warning naming the first word, in the warning color, before Enter. | Shot `573-06-act-warning` |
| REQ-008 | WHEN the Decisions view opens, it shall list the call with the reading and its confidence. | Shot `573-07-decisions` |
| REQ-009 | WHERE the project is not on the allow list, the system shall make no call. | Shot `573-08-unlisted`; the day's file |
| REQ-010 | WHERE the mode is `off`, the system shall make no call and leave #557's slot as it is. | Shot `573-09-off`; the day's file |
| REQ-011 | WHEN a line asked about is entered, sent, edited or cleared, the system shall log the call's outcome. | The day's file's outcome lines |
| REQ-012 | The system shall never change where Enter or Ctrl+Shift+Enter send a line. | Review of the diff; shot `573-05-enter-never-waits` |
| REQ-013 | WHEN the Marley settings page opens, its System One section shall show the use's mode. | The 515 scenario's page shot |
| REQ-014 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` (484 sits in the set and types at the prompt) |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #557 and #565
  have shipped (`read_line`, `is_command`, the slot's hint; the ask API, the replay file);
  `brain_ask`.
- **P2 Code:** the ledger rows first (`crates/settings_ui/src/marley_page.rs`, the use's
  dropdown; `crates/terminal_view/src/terminal_element.rs`, the warning variant of the
  suggestion paint); `english::open_case`; `prompt_line`, the timer, the facts, the ask and the
  outcomes; the `typed_line/1` set in `marley_system_one::question`; the slot's words. fmt and
  clippy clean; a review of the diff against REQ-012 first.
- **P3 Test:** write and run the scenario and read every shot; rerun #484's and #557's scenarios;
  `just regress`; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md`,
  `terminal_blocks.md` and `marley_system_one.md`; the plan's T3 row; ledger capture; close the
  ticket, archive, commit.
