# Copy without the agent's gutter, bracketed pastes and raw image paths while an agent runs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-536-agent-aware-copy-and-paste.md
- **Pipeline spec:** 536-agent-aware-copy-and-paste.spec.md

## Phase 1 — Plan
- **Request:** Chad asked on 2026-09-25 for every item decided that day to be specced; this is
  item 7 of report 05's list (`docs/orca_architecture/05-terminal-and-workspace.md` §3), listed in
  the survey README among the smaller things worth a day each. The brief for this draft: copy
  without the agent's gutter (Orca's `terminal-selection-gutter.ts`), bracketed pastes whenever
  an agent runs in the terminal, and raw image paths in a bracketed paste for dropped images,
  where Zed's `add_paths_to_terminal` shell-quotes them.
- **Classification / tier:** feature, S to M: three small rules in one pure module and three small
  hunks in Zed's terminal crates, all three of whose paths are already in the touchpoints ledger.
- **Recall (§18.3):**
  - BF-claude-single-str-replace-sanitizer-reconstitutes-split-tokens and
    PR-claude-sanitizer-must-loop-until-stable-single-replace-reconstitutes-001: stripping the end
    marker piece by piece can rebuild it. Zed's bracketed paste removes every ESC byte, which
    cannot; the forced bracket reuses exactly that.
  - AD-claude-479-attach-file-types-paths-through-zeds-path-prompt-001: Attach File types paths
    through `add_paths_to_terminal`, so the image rule reaches it with no change of its own.
  - #481's notes: rich input sends one `Terminal::paste` and a carriage return, and its scenario's
    stand-in reads lines (`read -r line`) and printed the two lines of a two-line prompt as two
    reads. After this ticket the same send arrives bracketed, so that stand-in must print raw
    bytes for its shot to say anything; it is updated and run again at Test.
  - L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001: drags under sway
    work through the held virtual pointer; targets come from the first shot.
  - Brain: not consulted in this drafting pass, which was read-only; `/pipeline:plan` runs
    `brain_ask` at promotion.
- **Discovery (opened and checked):**
  - `crates/terminal/src/terminal.rs`: `Modes::BRACKETED_PASTE` (413); the primary-selection
    writes (1928, 1942, 1961); the `InternalEvent::Copy` arm (1969: `selection_text(term)` to
    `cx.write_to_clipboard`, then the selection cleared unless `keep_selection_on_copy`);
    `Terminal::copy` (2198); `Terminal::paste` (2582: bracketed means `ESC [200~`, the text with
    every ESC removed, `ESC [201~`; otherwise `\r\n` and `\n` become `\r`);
    `foreground_process_command_name` (3071); `foreground_process_command_from_argv` (3615).
  - `crates/terminal/src/alacritty.rs:248`: `selection_text`.
  - `crates/terminal_view/src/terminal_view.rs`: the `copy` action (927); the `paste` action (945:
    an image forwards Ctrl+V through `forward_ctrl_v` at 992, `ExternalPaths` go to
    `add_paths_to_terminal` at 955, text goes to `Terminal::paste`); `paste_text` (979);
    `add_paths_to_terminal` (998 to 1008: `" " + shlex::try_quote(path)` for each, one trailing
    space, the view focused, one `Terminal::paste`); the drop handler (external paths at 1660 to
    1663, local projects only; a dragged tab's file at 1762; a project panel selection at 1768 to
    1778; a project entry at 1788).
  - `assets/keymaps/default-linux.json`: in `Terminal`, `ctrl-shift-c` is `terminal::Copy`
    (1300) and `ctrl-shift-v` is `terminal::Paste` (1303).
  - `crates/marley_workbench/src/agent_bar.rs`: `attach` (47) hands the chosen paths to
    `add_paths_to_terminal` (73); `agent_in` (130) is the agent test the bar makes.
  - `crates/marley_workbench/src/rich_input.rs`: `send` (97; `terminal.paste` at 110, then
    `\r`).
  - `crates/marley_agent/src/marley_agent.rs:73`: `agent_kind_of`.
  - `crates/terminal/Cargo.toml:34` and `crates/terminal_view/Cargo.toml:32`: both already depend
    on `marley_terminal`; the ledger has rows for both `Cargo.toml`s' Marley lines, for
    `terminal.rs` and for `terminal_view.rs`.
  - `crates/gpui_linux/src/linux/wayland/clipboard.rs:199`: `read` offers text, then an image, and
    never paths (the Out item).
  - `script/e2e/489-browser-input.sh` (151, 164): `wl-copy` and `wl-paste` against the run's sway
    through `SWAY_DISPLAY`; `script/e2e/481-rich-input.sh` (the line-reading stand-in).
  - Orca: `src/shared/terminal-selection-gutter.ts` (`measureGutter`: blank and space-only lines
    are skipped; `\r` kept with its line); `src/renderer/src/components/terminal-pane/terminal-agent-paste-bracketing.ts`
    (why silence about DECSET 2004 is not evidence: some shells never send it);
    `terminal-drop-image-path.ts` (`isImageDropPath`, `canPasteImageDropPathRaw` with the POSIX
    list); `terminal-drop-path-writer.ts` (one bracketed paste per image, a separating space only
    before a non-image path); `terminal-bracketed-paste.ts` (`wrapTerminalBracketedPasteText`).
- **Decisions:** D1 to D6 in the spec.

- **Promotion, 2026-09-29:** every seam re-read: `Terminal::paste` (`terminal.rs:2692`), the
  `InternalEvent::Copy` arm (2050), `foreground_process_command_name` (3191) and
  `foreground_process_command_from_argv` (3750); `add_paths_to_terminal`
  (`terminal_view.rs:1240`) and its callers (1197 the clipboard's paths, 1965 and 2064 to 2090 the
  drops, `agent_bar.rs:74` Attach File); rich input's `send` (`rich_input.rs:119`) now pastes
  through `terminal_drive::paste_then` (#594), so it reaches `Terminal::paste`;
  `marley_agent::agent_kind_of` (`marley_agent.rs:99`); `marley_agent` has no gpui and no
  `terminal` dependency, so `terminal` can take it. PR-claude-594 (keys after a paste go in a later
  write) weighed for the drop's pieces: its failure was an Enter absorbed into a paste, and a drop
  presses none, so the pieces are written in order as Orca writes them. Brain (consultation
  1bff15f3): nothing on this seam.

### Design
- **`crates/marley_terminal/src/paste.rs`, a new pure module** (declared in `marley_terminal.rs`):
  - `strip_shared_indent(text) -> Cow<str>`: split on `\n`, keep a trailing `\r` with its line,
    measure the smallest run of leading spaces over the lines that hold anything but spaces, and
    remove that many from every line (fewer from a shorter blank line); a run of zero returns the
    text untouched.
  - `is_raw_image_path(path) -> bool`: the extension of the file name (not of a directory) is one
    of the five, case-insensitive; no byte below 0x20 and no 0x7f; none of the special characters.
- **`crates/terminal/src/terminal.rs`, three hunks, each with `// Marley: #536`:**
  - `pub fn marley_agent_in_foreground(&self) -> bool`: `foreground_process_command_name()` passed
    to `marley_agent::agent_kind_of`.
  - `paste`: brackets when the mode is set, or when the text holds `\n` or `\r` and
    `marley_agent_in_foreground()` holds. `pub fn marley_paste_bracketed(&mut self, text)`
    always brackets, with the same ESC removal, for the image paths.
  - The `InternalEvent::Copy` arm: the selection's text goes through `strip_shared_indent` when an
    agent is in the foreground. `process_terminal_event` receives the locked grid as its own
    argument, so the `&self` read of the foreground process does not collide with it.
- **`crates/terminal_view/src/terminal_view.rs`, one hunk in `add_paths_to_terminal`:** with an
  agent in the foreground, each path in order: a raw image through `marley_paste_bracketed`,
  followed by one space only when a quoted path comes next; any other path as
  `shlex::try_quote(path)` and a space, through `Terminal::paste`. The view takes the focus as
  before. Without an agent, Zed's code runs unchanged.
- **`crates/terminal/Cargo.toml`:** `marley_agent.workspace = true`.
- **Rich input:** no change; its send reaches the new condition in `Terminal::paste`.
- **File manifest.** Marley: `crates/marley_terminal/src/paste.rs` (new),
  `crates/marley_terminal/src/marley_terminal.rs`; `script/e2e/536-agent-aware-copy-and-paste.sh`. Zed: `crates/terminal/src/terminal.rs`,
  `crates/terminal_view/src/terminal_view.rs`, `crates/terminal/Cargo.toml`.
- **Ledger rows.** The existing rows for those three paths grow by one clause each, written before
  the code (§14): what each hunk does, and at a merge, keep the agent condition in `paste`, the
  trim in the `Copy` arm and the agent branch at the top of `add_paths_to_terminal`. At Complete:
  an AD for "agent terminals only" (D2) and the one agent test (D1).

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | setup: `repo` with `shot.png` (a small PNG that Python writes) and `notes file.txt`; the scenario's HOME whose `.bashrc` sets `PS1='$ '` and defines `stand_in`, which runs `exec -a claude bash -c '<print "  first line of the reply", "    a nested line", "  last line">; stty -icanon -echo -icrnl; exec -a claude cat -v'`. Steps: trust the repository; type `stand_in`, Enter; drag from the first column of the reply's first line to the end of its last; Ctrl+Shift+C; print `wl-paste -n \| cat -A` to the run log; Ctrl+Shift+V | `536-01-copy-and-paste`: `^[[200~first line of the reply`, `  a nested line`, `last line^[[201~`; the log shows the clipboard flush left |
| REQ-003 | drag `shot.png` from the project panel onto the terminal, then `notes file.txt` | `536-02-dropped-paths`: `^[[200~<repo>/shot.png^[[201~`, one space, `'<repo>/notes file.txt' ` |
| REQ-004 | `wl-copy --foreground` of `one`, a newline, ESC `[201~two`, against the run's sway; Ctrl+Shift+V | `536-03-escape-stripped`: `^[[200~one`, then `[201~two^[[201~`, with no `^[` inside |
| REQ-005 | Ctrl+G on the stand-in; two lines typed in rich input (Shift+Enter between); Enter | `536-05-rich-input`: one bracketed paste, then `^M` |
| REQ-006 | Ctrl+~ for a new center terminal (plain bash); type `printf` of the same three lines, `; stty -icanon -echo -icrnl; cat -v`; drag over the three printed lines; Ctrl+Shift+C; Ctrl+Shift+V; then drag `shot.png` from the project panel onto it | `536-04-plain-shell`: the indentation kept, no markers, each line ended by `^M`, then ` <repo>/shot.png ` as Zed writes it (a space before and after, quotes only where the shell needs them), no markers |

Not reachable by a scenario: Claude Code's own attachment of a dropped PNG, which needs a logged-in
Claude Code. At Test, a PNG is dropped on Chad's real Claude Code in the installed Marley (by Chad
or with him), and the notes record whether it shows as an attached image.

### Risks
- Sibling tickets drafted the same night paste through `Terminal::paste` (their queued specs,
  2026-09-25): #522 sends review notes to an agent as rich input does, and #525's
  `terminal.write` types an agent's text into a running program. With this ticket a multi-line
  send of either is bracketed whenever an agent CLI is the foreground program, which is what
  #522 wants; #525's writes to other programs (psql, a REPL) are unchanged.
- The foreground check reads Zed's process info, which follows the PTY's foreground process; in
  the moment after an agent exits, a paste may still be bracketed. That costs nothing: bash reads
  a bracketed paste as a paste.
- An agent CLI that never understands bracketed paste would show the markers. Orca's note in
  `terminal-agent-paste-bracketing.ts` says a TUI agent turns bracketed paste on itself, and the
  four agents Marley knows are TUIs, so the forced bracket repeats what they ask for; Test checks
  it once by hand with Chad's real Claude Code, when the dropped PNG is tried.
- A selection copied with the keyboard in vi mode goes through the same `Copy` arm and is trimmed
  the same way while an agent runs.
- `wl-copy` of text holding an ESC byte: the scenario builds it with `printf` so the byte is real.

## Phase 2 — Code
- **Built:**
  - `marley_terminal/src/paste.rs` (new, pure): `strip_shared_indent` (Orca's rule: the run of
    leading spaces every non-blank line shares) and `is_raw_image_path` (`png`, `jpg`, `jpeg`,
    `gif`, `webp` in any case; no control character; none of the shell's special characters).
  - `terminal.rs` (Zed): `marley_agent_in_foreground` (`agent_kind_of` on the foreground command's
    name); an early return at the top of `paste` for a text holding a line break while an agent
    runs, through `marley_paste_bracketed` (always bracketed, ESC bytes removed); in the
    `InternalEvent::Copy` arm, the selection through `strip_shared_indent` while an agent runs.
  - `terminal_view.rs` (Zed): an agent branch at the top of `add_paths_to_terminal`: a raw image
    path through `marley_paste_bracketed`, any other path shell-quoted with a space after it and
    a space before it when an image came just before.
  - `crates/terminal/Cargo.toml`: `marley_agent`.
- **Deviations:** the forced bracket is an early return before Zed's code rather than a wider
  condition in its `if`: rustfmt re-indented Zed's whole `if`/`else` for the longer condition,
  and the early return leaves every upstream line as it was.
- **Review of the diff:** the agent check reads Zed's process info, so the copy, the paste and the
  drop agree; the Copy arm's `&self` read does not collide with the locked grid, a separate
  guard; a path that is not UTF-8 is skipped in the agent branch, as Zed's `filter_map` skips it;
  no path presses Enter.
- **Gate:** run 1: every gate passed, and the receipt failed, since the scenario was edited while
  the run was going. Its warnings (`pty_info.rs`, `terminal.rs:1202`, `:3199`,
  `terminal_panel.rs:327`, `terminal_view.rs:1689`, `:2129`) are Zed's own code: pre-existing,
  not in scope. Run 2 on the final tree: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/536-agent-aware-copy-and-paste.sh` under `compositor sway`: a
  stand-in Claude Code that runs as `claude` (bash, then `cat -v`), prints an indented reply and
  shows every byte it gets.
- **Runs 1 to 3, the scenario's faults:** the agent bar the stand-in brings moved the reply two
  rows up (run 1's drag caught one line); the terminals' titles name the process Zed sees
  (`cat -v`), so the screen reads match the project's name; a trailing space the screen reader
  trims was in a check; Ctrl+J, meant to break lines, is Zed's dock toggle and was dropped; the
  plain shell has no agent bar, so its rows are its own.
- **Run 4:** exit 0, every check passes. Every shot read:
  - `536-00-stand-in`: the stand-in's reply in its two-space gutter; the rail reads Claude Code.
  - `536-01-copy-and-paste` (REQ-001, REQ-002): the copy pasted back as
    `^[[200~first line of the reply`, `  a nested line`, `last line^[[201~`: flush left, the
    nested line's two spaces kept, one bracketed paste; the log's `wl-paste` agrees.
  - `536-02-dropped-paths` (REQ-003): `^[[200~…/shot.png^[[201~`, then
    `'…/notes file.txt'` and a space.
  - `536-03-escape-stripped` (REQ-004): `^[[200~one`, then `[201~two^[[201~`: the inner marker
    lost its ESC.
  - `536-05-rich-input` (REQ-005): `^[[200~first prompt line`, `second prompt line^[[201~^M`.
  - `536-04-plain-shell` (REQ-006): in a new terminal, the same three lines copied with their
    indent, pasted with no markers and each line break a `^M`, and the dropped PNG as
    ` …/shot.png `.
- **Not reached:** Claude Code's own attachment of a dropped PNG needs a logged-in Claude Code; it
  waits for Chad's hand check in the installed Marley.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` T7;
  `docs/marley_architecture/terminal_blocks.md` (`paste.rs`); the three touchpoint rows describe
  what shipped.
- **Knowledge:** `AD-claude-536-agent-terminals-get-their-own-copy-and-paste-001`,
  `L-claude-536-an-agents-terminal-is-titled-by-its-process-001`. No `F-…` block: no product bug
  was found in Code or Test. Brain: the decision on consultation 1bff15f3, follow-up by
  2026-10-29.
- **Closed** TICKET-536, archived the pair.
