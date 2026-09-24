# Autosuggestions from history, accepted with → — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-484-autosuggestions.md
- **Pipeline spec:** 484-autosuggestions.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request:** Chad, 2026-09-23: "warp has an awesome auto complete tell where you type
  something and from history and use the right arrow to auto complete. we need that".
- **Recall.** The brain (consultation cf6e190f1cf846ddaf6102aaf3cc8691) returned only unrelated
  follow-ups. #485 (a shell's first prompt laid out for the wrong width) and #481 (a key bound in
  `Terminal` that propagates when it has nothing to do) are the nearest seams.
- **Seams.** `AnchoredBlocks::apply` stages the prompt at `Precmd` and takes it at `Preexec`, so
  a staged prompt means the shell waits at it. `Terminal::input` is where every typed key goes.
  `Content` holds the viewport's cells with their points, the cursor, `marley_screen_top` and
  `display_offset`, and the mode (the alternate screen). The element paints IME marked text at
  `layout.ime_cursor_bounds` with a shaped line; ghost text is the same without the background.
  The shells' `init` frames carry fields the decoder looks up by name, so `histfile=` is
  additive.

### Design
- **`marley_terminal`.** `AnchoredBlocks` gains the point where the typed command starts (set
  by `note_input(line, column)` while a prompt is staged and none is set; cleared by `Precmd`
  and `Preexec`), `at_prompt()`, and the history file `InitShell` named. A new `suggest` module:
  `suggestion(typed, history)` (the rest of the newest entry that starts with the typed text and
  is longer; none for blank text) and `parse_history(text)` (bash's lines without its `#<time>`
  lines, zsh's `: <time>:<n>;command` lines, continuations joined). `DcsHook::InitShell` gains
  `history_file`.
- **The shells.** `marley.bash` adds `;histfile=` with `$HISTFILE` (bash sets it to
  `~/.bash_history` when unset); `marley.zsh` adds it when `$HISTFILE` is set.
- **`crates/terminal`.** `Terminal::input` notes the cursor's absolute point in its blocks
  before it writes.
- **`crates/terminal_view`.** A `MarleyTerminalSuggestion` global hook, and in the element's
  prepaint its text for the terminal, painted dimmed at the cursor's bounds after the blocks and
  before the cursor.
- **`marley_workbench`** (a new `autosuggest.rs`): the hook (the typed text from the cursor's
  line of cells, the lookup over the terminal's own verified commands and then its history
  file's), the history files read once per path in the background (deferred out of the frame),
  and `marley::AcceptSuggestion` (types the rest, or propagates), bound to `right` in `Terminal`.
- **Files:** `marley_terminal/src/{anchored,dcs,suggest,marley_terminal}.rs`, both shell
  integrations, `crates/terminal/src/terminal.rs`, `crates/terminal_view/src/terminal_view.rs`,
  `crates/terminal_view/src/terminal_element.rs`, `marley_workbench/src/{autosuggest,marley_workbench}.rs`,
  `marley_workbench/keymap.json`, `script/e2e/484-autosuggestions.sh`. The three Zed files have
  ledger rows already; each extends.

### E2E plan
| REQ | Shot |
|---|---|
| 001 | `484-01-ghost`: `$ ech` and `o hello world` dimmed after it |
| 002 | `484-02-taken`: after →, `$ echo hello world` with no ghost; `484-03-ran`: `hello world` printed |
| 003 | `484-04-nothing`: `zzz` with no ghost |
| 004 | `484-05-session-first`: `ec` suggests `ho from this session`, run in the session, over the file's `echo hello world` |
| 005 | `just gate-diff` |

### Risks
- Where the command starts is a guess from the first key: typed ahead of a slow prompt, it lands
  inside the prompt, and then nothing matches, so nothing shows.
- → in `Terminal` passes through the action on every press; it propagates at once unless a
  suggestion shows.

## Phase 2 — Code (2026-09-23)
- **Built.**
  - `marley_terminal`: `DcsHook::History { file }` (decoded from `history;file=`; the old
    `SessionModel` ignores it); `AnchoredBlocks`' `input_start`, `note_input`, `at_prompt` and
    `history_file`; `suggest.rs` with `suggestion` and `parse_history`.
  - The scripts: `marley.bash` and `marley.zsh` send `history;file=` after `init` when
    `$HISTFILE` is set.
  - `crates/terminal`: `input` notes the cursor's absolute point off the alternate screen
    (`HookPosition::of`, the hooks' own coordinates); `marley_anchored`.
  - `crates/terminal_view`: the `MarleyTerminalSuggestion` global; in the element,
    `LayoutState::marley_suggestion` from it in `prepaint`, painted in `status().predictive` (the
    color of Zed's edit predictions) at `ime_cursor_bounds`, after the blocks and before the
    cursor, while no IME text is marked.
  - `marley_workbench`: `autosuggest.rs` (the hook, `typed_text`, the lookup, `HistoryFiles`,
    `marley::AcceptSuggestion`), `log` as a dependency; the keymap binds `right`.
  - The three Zed rows in the ledger extended first.
- **Deviation.** `histfile=` is a frame of its own, `history`, instead of a field on `init`:
  `InitShell` is built in a dozen tests that still have to compile.
- **Review.** The hook runs in every terminal's prepaint and returns at once off a prompt or
  before a key is typed there. A history file is read once; bash writes its history when a shell
  exits, so another terminal's commands reach suggestions after that, or at the next launch.

## Phase 3 — Test (2026-09-23)
- **E2E** (`SHOT_DIR=<scratchpad>/e2e484 just e2e script/e2e/484-autosuggestions.sh`; focus
  report: "the user's window and workspace are as they were"):
  - `484-01-ghost`: `$ ech`, and `o hello world` dimmed from the cursor on (REQ-001).
  - `484-02-taken`: after →, `$ echo hello world`, the cursor after it, no ghost (REQ-002).
  - `484-03-ran`: `hello world` printed; a new prompt.
  - `484-04-nothing`: `$ zzz`, no ghost (REQ-003).
  - `484-04-left`, `484-04-right`: Left puts the cursor on the last `z`; → with no suggestion
    moves it back after the text, so → reached readline (REQ-002).
  - `484-05-session-first`: after `echo from this session` ran, `ec` suggests
    `ho from this session` over the file's `echo hello world` (REQ-004).
- **Gate.** `just gate-diff`: `GATE GREEN [diff]`, 15 gates and the receipt, which matches the
  tree.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Added: autosuggestions); `docs/marley_architecture/terminal_blocks.md`
  (the typed-text start, the `History` hook, `suggest.rs`); `marley_workbench.md` (the
  Autosuggestions section and the scenario); the plan's T3 row marks T3a shipped; the three Zed
  rows in the ledger.
- **Knowledge.** AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001,
  L-claude-484-where-a-typed-command-starts-without-touching-the-prompt-001.
- **Brain.** Consultation cf6e190f1cf846ddaf6102aaf3cc8691 closed with
  `marleys-autosuggestions-read-the-typed-command-from-the-grid`, follow-up by 2026-10-07.
