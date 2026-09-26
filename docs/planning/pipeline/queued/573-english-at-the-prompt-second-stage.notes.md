# English at the prompt, second stage: a System One reading for the lines the rules leave open — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-573-english-at-the-prompt-second-stage.md
- **Pipeline spec:** 573-english-at-the-prompt-second-stage.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the System One uses; for the typed line the Warp note's
  sentence ("It could judge whether a typed line is a command, a request, a comment or a command
  followed by English (only for lines the local rules leave open, asked after about 250 ms without
  typing so Enter never waits on the network)") and its recommendation 6 ("A System One model as
  the second stage of detection, opt-in, once the local rules exist and behind #516's
  redaction"), with the brief's words "opt-in, redacted, or a local model". #557 is the first
  stage (the local rules, the hint in the suggestion slot, Ctrl+Shift+Enter, the exit-127
  button) and names this ticket as the second. Drafted by the second spec drafter beside #565
  and #566 to #568, and aligned to #557's and #565's pairs once they were on disk.
- **Classification / tier:** feature, prong 1 (T3). Marley crates, plus the page's dropdown for
  the use's mode and one hunk in `terminal_element.rs` (the warning paint). Size S to M.
- **Recall (§18.3):**
  - AD-claude-484 (the typed text is read from the grid's cells between the input's start and the
    cursor; no change to the user's prompt): `typed_text` is the read, unchanged.
  - L-claude-484 (the input's start is the cursor at the first key after `precmd`; wrong only
    when a key lands before the prompt has drawn, and then nothing matches): the same limit here,
    and a line with no start is never asked.
  - BF-claude-sync-network-fetch-on-gpui-main-thread-freezes-ui-001 (a network call on the main
    thread froze the UI for seconds): the call runs on the layer's executor and its answer comes
    back as an update; Enter's path touches no network.
  - AD-claude-516 and PR-claude-redact-the-whole-text-before-cutting-it-001: the line is redacted
    whole; a count above zero cancels the call.
  - PR-claude-474 (a hook frame is output until its nonce says otherwise): the `verified command`
    fact reads `command_verified`.
  - #557's D1 (`Reading` is the interface this ticket fills; "there are none yet"), D2 (Enter is
    the shell's), D3 (the marker rule), D4 (the hint rides the suggestion slot, never the
    footer); #565's D1 and D5.
  - Brain: not consulted in this drafting session; promotion asks.
- **Discovery** (at `51bfe04034`; promotion re-verifies, and reads #557 as shipped):
  - `crates/marley_workbench/src/autosuggest.rs`: `init` (29-52, the suggestion hook and
    `AcceptSuggestion`), `suggestion` (56-72), `typed_text` (76-105), `read_history` (120-149).
  - `crates/terminal_view/src/terminal_view.rs`: `MarleyFooterContext` (130-139),
    `MarleyTerminalFooter` (141-145), `MarleyTerminalSuggestion` (150); `terminal_element.rs`:
    the `marley_suggestion` paint at the cursor in the predictive color (#484's hunk).
  - `crates/marley_terminal/src/anchored.rs`: `at_prompt` (176), `note_input` (182),
    `input_start` (190), `history_file` (196); `AnchoredBlock::command_verified` (44-45).
  - `crates/terminal/src/terminal.rs`: `input` (2316-2326), `last_content` (2134).
  - `crates/marley_workbench/src/rail.rs`: the re-armed one-shot timers (364, 431);
    `crates/marley_workbench/src/agents.rs:205` (a timer as a timeout).
  - `crates/marley_workbench/src/voice.rs:66` (`which::which` off the main thread).
  - `crates/marley_workbench/src/rich_input.rs`: `send` (97-116).
  - `crates/marley_workbench/src/mcp.rs`: `agent_redactor` (297-303).
  - #557 (queued): `marley_terminal::english::read_line(line, is_command) -> Reading {Command,
    English, Blank}` and its marker rule; `is_command` (the search path's names, the builtins,
    the terminal's verified commands); the hint ` · ctrl-shift-enter asks the agent` in the
    suggestion slot when the line reads as English and no history suggestion applies;
    `marley::AskAgent`; `marley.english_hint`.
  - #565 (queued): `UseSpec`, `system_one::ask`, `StateBuilder`, `Detail`, `Reading`, the day's
    file, the replay file and `system_one_setting`; `marley.system_one.uses`; the Decisions view.
  - `crates/settings_ui/src/marley_page.rs` (6-15, 42-91).
  - `script/e2e/484-autosuggestions.sh` (bash with a plain prompt, `type_text`, the ghost-text
    shots) and the golden set (484 in it).
- **Decisions:** D1 to D8 in the spec.

### Design
- **The open case.** `english::open_case(line, is_command) -> bool`: the first word is a
  command, the line has two or more words, and no word is one of #557's shell-syntax signals
  (`|`, `<`, `>`, `&&`, `;`, `$`, a `-` flag, a glob, a path with `/`, `./` or `~`, a first
  word holding `=`); a line starting with `#`, `!` or `\` is never open. #557's `read_line`
  stays the leaning: `English` by the marker rule, else `Command`.
- **The watch.** `prompt_line::init` wraps the suggestion hook's read: at each draw where
  `typed_text` yields a line, the module compares it with the last text seen for that view; a
  change (or a new prompt) cancels the view's pending `Task` and, when the line is open, starts a
  new one: `cx.background_executor().timer(250 ms)` then, on the foreground, a check that the
  text is unchanged and the redactor finds nothing, then `system_one::ask` with the text and the
  facts; the answer, when the text still matches, sets `Readings[view] = Reading { text, class,
  confidence }` and refreshes the window. Any later change clears the entry. An empty text clears
  everything and, if a call was in flight, logs it dropped.
- **The facts** (`typed_line::facts(line, terminal, shell)`, pure where it can be): the first
  word; `on_path` (#557's `is_command` set, read off the main thread as `voice.rs` reads
  `which`); `builtin` from #557's list; `verified_command` from the terminal's blocks with
  `command_verified`; `history_match` from the same history `autosuggest` reads; counts and
  presences by a tokenizer over the raw text (flags `-x`/`--x`, `|`, `>`, `<`, `&&`, `;`, `=`,
  `$`, a `/` path, a quoted string); the shell from the integration's `init` frame; the last
  block's command's first word and exit code. The state: `StateBuilder::new(project, detail)`
  with those as facts and `line` as the one text, through the layer's `Mask`; the use's verdict
  (#557's reading) with it; `UseSpec { name: "typed_line", deadline: 600 ms }`.
- **The slot.** The suggestion hook's answer gains an optional reading: the module appends to
  #557's hint (or supplies it where the rules gave none): ` · ctrl-shift-enter asks the agent`
  for a request, ` · a comment` for a comment, ` · English after <word>, ctrl-shift-enter asks
  the agent` for the middle, each with `?` in `suggest`; a command adds nothing. The hook's
  return type carries a `warning: bool`, and `terminal_element.rs`'s paint uses the theme's
  warning color for it (a small change inside #484's hunk; its row exists). A history suggestion
  still wins the slot.
- **The outcome.** The module keeps the call id with the text; the terminal's next `preexec`
  (a new block whose command equals the text: `enter`, with the exit code added at `precmd`), a
  Ctrl+Shift+Enter send (`agent`, from #557's send), a change to a different non-empty text
  (`edited`) or an empty prompt (`cleared`) writes the outcome line.
- **File manifest.** Marley: `crates/marley_workbench/src/prompt_line.rs` (new),
  `marley_workbench.rs` (the module, `init`), `autosuggest.rs` (the hook's answer gains the
  reading and the warning flag); `crates/marley_terminal/src/english.rs` (#557's, gains
  `open_case`) and `typed_line.rs` (new, pure: the facts' tokenizer);
  `crates/marley_system_one/src/question.rs` (the `typed_line/1` set) and the workbench's
  `system_one.rs` (the `UseSpec`); `script/e2e/573-english-at-the-prompt-second-stage.sh`. Zed:
  `crates/settings_ui/src/marley_page.rs` (the use's dropdown), `crates/terminal_view/src/terminal_view.rs`
  (`MarleyTerminalSuggestion`'s answer type, if the flag needs it) and
  `crates/terminal_view/src/terminal_element.rs` (the warning color in the paint).
- **Ledger rows.** The `marley_page.rs` row gains the `typed_line` dropdown (#573); the
  `terminal_view.rs` and `terminal_element.rs` rows gain the warning variant of the suggestion
  paint (#573), each before its edit.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` sets `PS1='$ '` and puts `$E2E_WORK/bin`
first on the PATH, with `bin/rm` (a script that appends its arguments to `$E2E_WORK/rm.log` and
exits 0); the profile's settings: `marley.english_hint` on (#557's default), #565's block enabled
on `replay` with the repository listed and `uses.typed_line` rewritten per step by
`system_one_setting`; `$E2E_PROFILE/system_one/replay.jsonl` written by setup (matched on the
line's words); the slot's text read from the shots.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | mode `suggest`; `type_text "find all the large files in this repo"`; settle 1 | `573-01-open-line`; the day's file: one row |
| REQ-003 | Ctrl-U; `type_text "kill the dev"`; `type_text " server"` at once; settle 1 | `573-02-typing-cancels`; the day's file: one row, for the whole line |
| REQ-004 | Ctrl-U; `type_text "ls -la"`; settle 1 | `573-03-command`; no new row |
| REQ-005 | Ctrl-U; `type_text "export TOKEN=<value assembled in setup>"`; settle 1 | `573-04-secret`; no new row |
| REQ-006 | Ctrl-U; `type_text "rm the old build folder"` and `press "" Return` in the same breath; settle 1 | `573-05-enter-never-waits`; `holds rm.log "the old build folder"`; the day's file: no row or a dropped row |
| REQ-007 | mode `act`; `type_text "rm the old build folder"`; settle 1 | `573-06-act-warning` |
| REQ-008 | `marley: decisions` from the palette; click the row | `573-07-decisions` |
| REQ-009 | the repository taken off the list; Ctrl-U; the same line; settle 1 | `573-08-unlisted`; no new row |
| REQ-010 | mode `off`; Ctrl-U; the same line; settle 1 | `573-09-off`; no new row |
| REQ-011 | after 01: Ctrl-U (`cleared`); after 06: Return (`enter`, exit 0 from the fake) | the outcome lines |
| REQ-012 | review, and 05 | the diff; the shot |
| REQ-013 | the golden set's 515 run | its page shot |

Not reachable by a scenario: a real provider's latency under Enter (the replay answers at once;
REQ-006 proves Enter's path by construction and by the block starting before any reading could
show); a local model.

### Risks
- #557 may ship `read_line` and the hint under names this spec cannot know; promotion binds
  them, and nothing here depends on their shape beyond a reading and a slot for words.
- A `which` lookup per first word: #557's command set is read once and cached; a word not in it
  costs nothing more.
- With TypeSafe as the only provider, every open line of a listed project leaves the box; that is
  the reason for `off` by default and the allow list, and for keeping this ticket Deliberate
  until Chad picks the projects or #565 gains a local provider.
- A reading that arrives as the user presses Enter: the reading is dropped by the text check, and
  the slot never draws over a running block.
- The warning color in the grid's paint is one condition inside #484's Marley hunk; if
  `MarleyTerminalSuggestion`'s answer type cannot grow cheaply, the middle's words carry the
  warning in text alone (`English after rm!`) and the Zed touch goes.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
