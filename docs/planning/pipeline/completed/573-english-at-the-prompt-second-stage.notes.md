# English at the prompt, second stage: a System One reading for the lines the rules leave open — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-573-english-at-the-prompt-second-stage.md
- **Pipeline spec:** 573-english-at-the-prompt-second-stage.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-26; its design is superseded below)
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
- **Discovery** (at `ca70b6488d`; promotion re-verifies, and reads #557 as shipped):
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

## Phase 1 — Plan (promoted 2026-09-30)
- **Recall (§18.3):**
  - AD-claude-557: the hint rides the slot, never the footer, since a footer that changes height
    resizes the PTY; the editor's inlay changes no height, so it keeps that rule.
  - AD-claude-572 and `running_errors.rs`: the use pattern (a const `UseSpec`, `use_mode`,
    `Asking`, `ask` awaited in a spawn, `record` for a rules verdict, `outcome` rows later).
  - PR-claude-a-state-fact-holds-only-what-code-computed-001: the line goes in as text; the
    first word is a fact only when code knows it as a command.
  - F-claude-627-a-quick-commands-prompt-went-unseen-001: a terminal observer sees states, not
    edges; the outcome `entered` is read from a new block, not from a flag.
  - Brain (`rusty-cli brain ask`, consultation e9af265999534a5f80012f8c1dc4165e): nothing on this
    seam.
- **Re-binding:** the queued draft predates #627. With the prompt editor the default, #557's hint
  and Ctrl+Shift+Enter do not reach the line (the Explore map: `typed_text` reads the grid, and
  Zed binds Ctrl+Shift+Enter in an auto-height editor to `editor::NewlineBelow`), so this ticket
  moves them into the editor first (REQ-001) and draws the reading there. The grid's warning
  colour, a Zed touch, is dropped: the grid shows the words alone (D9).

### Design
- **`marley_terminal::english`** gains `open_case(line, is_command) -> bool` and `marker_count`,
  pure, beside `read_line`.
- **`marley_system_one`** gains `TYPED_LINE_SET` (`typed_line/1`, one `Choice` `kind` with
  `command`, `request`, `comment`, `command_then_english`, `cannot_tell`) and `TYPED_LINE`
  (`typed_line`, 600 ms).
- **`marley_workbench::typed_line`** (new): a global `Lines` keyed by the terminal's entity id,
  each a `Line { text, settle: Option<Task<()>>, asked: Option<Asked>, entering, awaiting }`
  where `Asked` holds the text, the call id and the shown reading. `changed(terminal, text, cx)`
  is the one entry: a different text writes `edited` or `cleared` for a call still open (unless
  `entering`), clears the reading, drops the timer, and when the use is on and the line is open
  and the redactor finds nothing, arms a 250 ms `background_executor().timer`; when it fires and
  the text is unchanged it builds the `Asking` (facts and text as in D5) and awaits
  `system_one::ask(TYPED_LINE, ..)`; an answer for a changed text writes `dropped`. A reading is
  kept only when it is `Reading::Model` with a `kind` choice at or above the floor that is not
  abstaining. `init` observes each `TerminalView`'s terminal: while the shell's editor is closed
  it feeds `changed` with `autosuggest::typed_text`, and in every case it checks a newest block
  whose command is the asked text (`entered`, then `exit N` when it finishes). Settings changes
  clear every reading. `shown(terminal, line, cx) -> Option<Shown>` gives the words and whether
  the English words are coloured.
- **`english.rs` (workbench):** `hint` takes the line (from the grid or the editor), asks
  `typed_line::shown`, and returns the words; `ask_typed` reads the editor's line when the shell's
  editor is open (and empties it), accepts a line whose shown reading offers the agent, and
  writes `asked the agent`.
- **`rich_input.rs`:** the shell's editor subscribes to its buffer edits: each edit calls
  `typed_line::changed` and refreshes the inlay (`splice_inlays` at the buffer's end, id
  `InlayId::EditPrediction(usize::MAX - 573)`) and the warning colour (`highlight_text` over the
  words after the first, or cleared); `send` for the shell calls `typed_line::entering` first;
  `typed_line` calls back `rich_input::refresh_hint(terminal, cx)` when a reading arrives.
  `keymap.json`: `ctrl-shift-enter` is `marley::AskAgent` in `MarleyShellInput > Editor`.
- **Settings:** `uses.typed_line: "off"` in `default.json` with its comment line;
  `settings_content`'s `uses` doc names it; the Typed Line dropdown in the System One section
  (the array grows to 17).

### File manifest
- Marley: `crates/marley_terminal/src/english.rs`; `crates/marley_system_one/src/marley_system_one.rs`;
  `crates/marley_workbench/src/typed_line.rs` (new), `marley_workbench.rs` (module, init),
  `english.rs`, `rich_input.rs`, `keymap.json`;
  `script/e2e/573-english-at-the-prompt-second-stage.sh`.
- Zed: `assets/settings/default.json`, `crates/settings_content/src/marley.rs` (doc only),
  `crates/settings_ui/src/marley_page.rs`; each row widened before the edit.

### Visual check plan
| REQ | Scenario | Shot or log |
|---|---|---|
| REQ-001, REQ-011 | use `off`; click the terminal (the editor docks); `kill the dev server` | `english`; no call |
| REQ-002, REQ-003 | `suggest`; Ctrl+C; `find all the large files in this repo`; 1 s | `open-line`; one call |
| REQ-004 | Ctrl+C; `kill the dev`, then ` server` at once; 1 s | `typing-cancels`; no call for the prefix |
| REQ-005 | Ctrl+C; `ls -la` | `command`; no call |
| REQ-006 | Ctrl+C; `curl <token>` | `secret`; no call |
| REQ-007, REQ-013 | Ctrl+C; `rm the old build folder` and Return at once | `entered`; `rm.log`; no call |
| REQ-008 | `act`; the same line; 1 s | `act-warning` |
| REQ-009 | Escape; the same line typed at the shell's prompt; 1 s | `grid` |
| REQ-010 | `marley: open decisions` | `decisions` |
| REQ-011 | the repository off the list; Ctrl-U; a new open line | `unlisted`; no call |
| REQ-012 | the day's file after the run | `cleared`, `edited`, `entered`, `exit 0` rows |
| REQ-014 | the Settings window searched for Typed Line | `settings` |
Not reachable: a real provider's latency (the replay answers at once; REQ-007 proves Enter's path
by the block starting with no reading shown); a local model.

### Risks and decisions
- The inlay uses Zed's edit-prediction kind with a reserved id; the prompt editor has no
  edit-prediction provider (`edit_prediction_registry` skips non-full editors), so nothing else
  removes it.
- `HighlightKey::Editor` is the key the warning colour uses; Zed's navigation uses it only in
  full editors.
- Ctrl+Shift+Enter now also asks for a line the reading offers; Enter is untouched (REQ-013).

## Phase 2 — Code (2026-09-30)
- **Built:** `english::open_case` and `marker_count` (marley_terminal); `TYPED_LINE_SET` and
  `TYPED_LINE` (marley_system_one); `typed_line.rs` (the watch, the quiet timer, the ask, the kept
  reading, the outcomes `entered`, `exit N`, `asked the agent`, `edited`, `cleared`, `dropped`);
  `english.rs`'s `hint_for` (#557's hint or the reading, and the warning's range), `offers_agent`,
  `command_source`, and `ask_typed` reading the editor's line; `rich_input.rs`'s edit
  subscription, the hint inlay (`InlayId::EditPrediction(usize::MAX - 573)`) and the warning
  colour (`highlight_text`), `entering` at send, `shell_editor_open`, `shell_text`,
  `clear_shell`, `refresh_hint`; `keymap.json`'s Ctrl-Shift-Enter in the shell's editor;
  `default.json`, the `uses` docstring and the Typed Line dropdown.
- **Deviations:**
  - The open case needs an English marker among the words after the command, so `git status`
    and `ls src` cost no call (the spec's In section says so now).
  - An unlisted project's line is not asked at all (`system_one::detail` first), so the day's
    file holds no refusal row per typed line.
  - The last line asked about is kept with its reading, so the same line again (after Escape,
    or a real provider's repeat check) shows its reading without a call; a settings change drops
    it.
- **Review:** the observer runs inside the view's update and reads only the terminal and
  globals; the editor's hint is painted from its edit subscription and from tasks, never inside
  the editor's own update. An ask in flight is detached, not dropped with the line, so its row is
  always written and a late answer logs `dropped`. `arguments` reads its range with `get`, since a
  draft's first line can end before its words. Enter's path gained only `typed_line::entering`,
  which moves state and sends nothing.
- **Clippy found:** the first doc paragraph's length, an unneeded qualification, an unused
  import, `usize` where the editor wants `MultiBufferOffset`, a collapsible `if`.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/573-english-at-the-prompt-second-stage.sh` (sway). Two runs; every
  log check passed in both.
- **First run, red:** `grid.png` showed #557's hint (` · ctrl-shift-enter asks the agent`) in the
  grid's slot, not the reading. The grid's line was read from an observer of the terminal, and
  typing reaches the terminal as `Wakeup` events, which notify no observer, so the typed line was
  never asked about.
- **Fix:** the grid's line is read in the suggestion hook, at each frame, from the same snapshot
  the slot is drawn from (`typed_line::follow_grid`, skipped while a shell editor holds the
  terminal's line); the observer keeps only the blocks' outcomes, and each line keeps its view's
  workspace. Gate GREEN, 17 PASS.
- **Second run:**
  - `english` (REQ-001, REQ-011): the use off; `kill the dev server` in the docked editor with
    #557's ` · ctrl-shift-enter asks the agent` dimmed after it; no call.
  - `open-line` (REQ-002, REQ-003): suggest; ` · a request? ctrl-shift-enter asks the agent`
    after `find all the large files in this repo`; one call, its state the masked line and the
    facts (`first word: find`, `first word is: a program on the search path`, `words: 8`,
    `English markers after the first word: 4`).
  - `typing-cancels` (REQ-004): `kill the dev` then ` server`: the reading for the whole line,
    and one call, for it alone.
  - `command` (REQ-005): `ls -la`, nothing after it, no call.
  - `secret` (REQ-006): `curl the ghp_…`, nothing after it, no call.
  - `entered` (REQ-007, REQ-013): the block `$ rm the old build folder` with its check, the
    editor back empty; `rm.log` holds `the old build folder`; no call for the line.
  - `act-warning` (REQ-008): act; ` · English after rm, ctrl-shift-enter asks the agent`, with
    `the old build folder` in the warning amber and `rm` in its syntax blue (cropped).
  - `grid` (REQ-009): Escape, the line typed at the shell's own prompt: ` · English after rm,
    ctrl-shift-enter asks the agent` in the grid's slot, from the kept reading, with no new call.
  - `unlisted` (REQ-011): the allow list emptied; `echo what is this` with #557's hint; no call.
  - `decisions` (REQ-010): four `typed_line · repo · replay` rows: `command_then_english 0.93`,
    `no signal (no replay row)` for `kill the dev server now`, `request 0.85`, `request 0.90`.
  - `settings` (REQ-014): the Marley page's System One section, Typed Line set to Act.
  - The day's file (REQ-012): `cleared`, `edited`, `cleared`, then `entered` and `exit 0` for the
    act call.
- **Not shot:** a real provider's latency (the replay answers at once; `entered` shows Enter's
  path holds no call); Ctrl+Shift+Enter in the editor, which would start Claude Code (#557's
  scenario covers the send; the binding is reviewed).

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Rich input's English paragraph, System One's "The
  typed line", the keys table); `marley_workbench.md` (English at the prompt, the typed line, the
  prompt editor); `terminal_blocks.md`; `marley_system_one.md`; the plan's T3 and S1 rows; the
  ledger rows for `default.json`, `settings_content` and `marley_page.rs`.
- **Pre-existing, not in scope:** the guide had no section for #557's English at the prompt (now
  covered beside the prompt editor) nor for #572's running error under System One.
- **Knowledge:** F-claude-573-the-grids-typed-line-was-never-read-001,
  PR-claude-read-a-terminals-typed-line-where-its-frame-is-drawn-001,
  L-claude-573-detach-an-ask-in-flight-instead-of-dropping-it-with-its-line-001,
  AD-claude-573-the-typed-line-reads-the-open-case-after-the-line-in-the-editor-first-001.
- **Brain:** the consultation closed with `brain decide`.
