---
pipeline_id: ea76d9fa-7236-40bc-867b-7aad7e34df80
ticket: docs/planning/tickets/open/TICKET-536-agent-aware-copy-and-paste.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Copy without the agent's gutter, bracketed pastes and raw image paths while an agent runs"
type: feature
slice: prong 1 T7 (CLI agents in the terminal), after #479 and #481
references: [docs/planning/pipeline/completed/479-attach-a-file.spec.md, docs/planning/pipeline/completed/481-rich-input.spec.md, docs/orca_architecture/05-terminal-and-workspace.md]
---

## Title
While an agent CLI is the terminal's foreground program, three of Zed's terminal behaviors change:
a copy loses the indentation every copied line shares, a paste with a line break is always
bracketed, and an image path that is dropped or attached goes in raw inside a bracketed paste of
its own, the form in which Claude Code and Codex recognize an image to attach (Orca's finding). In
a plain shell nothing changes.

## Scope
### In
- **Who counts as an agent.** The program in the terminal's foreground is an agent CLI when
  `marley_agent::agent_kind_of` recognizes the name Zed reads for it
  (`Terminal::foreground_process_command_name`), the same test the rail and the agent bar make.
  It is checked at the moment of the copy or the paste.
- **Copy.** With an agent in the foreground, `terminal::Copy` (Ctrl+Shift+C, and a copy on
  select) puts the selection on the clipboard without the run of leading spaces that every
  non-blank line shares. Blank lines and lines of spaces do not count; relative indentation
  stays; a selection that starts mid-line, or covers a line at column 0, shares a run of zero and
  is copied as it is (Orca's rule).
- **Paste.** With an agent in the foreground, a paste that holds a line break is sent inside
  bracketed-paste markers (`ESC [200~` and `ESC [201~`) even when the program never turned
  bracketed paste on, with every ESC byte removed from the text as Zed already does in bracketed
  mode. This covers the terminal's Paste and rich input's send (#481), which pastes through the
  same `Terminal::paste`. A single-line paste is unchanged.
- **Image paths.** With an agent in the foreground, the paths that reach the terminal through
  `TerminalView::add_paths_to_terminal` (a drop from a file manager or from the project panel,
  and Attach File, #479) go in this way: a path whose extension is `png`, `jpg`, `jpeg`, `gif` or
  `webp`, in any case, that holds no control character and none of `"'$;&|<>(){}[]*?!#\` and no
  backtick (Orca's list), goes in raw, alone inside its own bracketed paste; every other path is
  shell-quoted as Zed quotes it and followed by a space; a space separates an image's paste from
  a quoted path that follows it. Nothing presses Enter.
- **Where the code lives.** The pure rules in `marley_terminal`; small hunks in Zed's
  `crates/terminal` and `crates/terminal_view`, each with its `// Marley:` comment and row.

### Out (explicitly deferred)
- The primary selection (middle-click paste on Linux), which Zed writes while a selection changes
  (`crates/terminal/src/terminal.rs` 1928, 1942, 1961): the trim reaches only the clipboard here.
- The trim in plain shells (Orca trims in every terminal by default) and settings to turn any of
  the three off.
- Uploading dropped files for remote terminals (Zed refuses drops on a remote project) and
  saving a clipboard screenshot to a file (Zed forwards Ctrl+V so the agent reads the clipboard
  itself).
- SVG, BMP and ICO as images: the Claude API takes PNG, JPEG, GIF and WebP (report 05 §2.15).
- A file copied in a file manager and pasted as paths: gpui's Wayland clipboard reads text and
  images only (`crates/gpui_linux/src/linux/wayland/clipboard.rs:199`), so such a paste arrives
  as text, as it does today.

## Reference (§20)
Upstream Zed's terminal, kept as it is for plain shells: the copy (`Terminal::copy` queues
`InternalEvent::Copy`, whose arm writes `selection_text` to the clipboard), the paste
(`Terminal::paste` brackets only when the program set DECSET 2004, removing ESC bytes; otherwise
it turns line feeds into carriage returns), and `TerminalView::add_paths_to_terminal` (each path
shell-quoted with a space before it and one at the end). Orca (MIT, read at `1c2cf120e3`) for the
three agent rules: `src/shared/terminal-selection-gutter.ts` (`stripTerminalSelectionGutter`: only
the run every non-blank line shares); `src/renderer/src/components/terminal-pane/terminal-agent-paste-bracketing.ts`
(a pane's known agent forces a multi-line paste into brackets, because some shells never announce
bracketed paste, so silence is no evidence); `terminal-drop-image-path.ts` and
`terminal-drop-path-writer.ts` in the same folder (a safe image path raw inside a bracketed paste,
others escaped with a trailing space, a separating space only before a non-image path). Warp: N/A,
the once-over has no such item.

### Prior art
- **Behavior maps.** `docs/orca_architecture/05-terminal-and-workspace.md` §2.6 (copy without the
  agent's gutter; protected multi-line paste), §2.15 (drops: raw image paths in a bracketed paste,
  Zed's quoting, which image types the Claude API takes) and §3 item 7; the README's list of
  smaller items.
- **Published material.** xterm's bracketed paste mode (DECSET 2004 and its `200~` and `201~`
  markers). Claude Code's and Codex's image attachment from a pasted path is documented only
  through Orca's source comments and its issue #2842, so the scenario proves the bytes that reach
  the program, and the attachment itself is checked by hand (notes, E2E plan).
- **Code we already ship.** Zed owns each seam, and the ticket extends it rather than adding a
  second path: `Modes::BRACKETED_PASTE` and `Terminal::paste` (`crates/terminal/src/terminal.rs`
  413, 2582); the `InternalEvent::Copy` arm (1969) and `selection_text`
  (`crates/terminal/src/alacritty.rs:248`); `foreground_process_command_name` (3071) with
  `foreground_process_command_from_argv` (3615: the first argument's name, or a node or Python
  script's stem); `TerminalView::add_paths_to_terminal`
  (`crates/terminal_view/src/terminal_view.rs:998`) and its callers (the clipboard's paths at 955,
  the drops at 1663, 1762, 1778 and 1788, Attach File at `crates/marley_workbench/src/agent_bar.rs:73`);
  `agent_bar::agent_in` (130); rich input's `send` (`crates/marley_workbench/src/rich_input.rs:97`,
  `terminal.paste` at 110). Zed's stripping removes every ESC byte, so no end marker can be
  rebuilt from pieces, the failure PR-claude-sanitizer-must-loop-until-stable-single-replace-reconstitutes-001
  records for marker-by-marker stripping.

## UI proof
UI-AFFECTING. `script/e2e/536-agent-aware-copy-and-paste.sh` (`compositor sway`, for the selection
drag and the project panel drag). The scratch repository holds `shot.png` and `notes file.txt`;
its terminal's `.bashrc` defines a stand-in Claude Code that runs as `claude` (`exec -a claude`),
prints an indented reply, then turns off canonical mode, echo and CR translation and runs
`cat -v` under the same name, so every byte that reaches it shows (ESC as `^[`, a carriage return
as `^M`). The stand-in never enables bracketed paste. Steps: the stand-in started; a drag over
its reply from the first column, Ctrl+Shift+C, Ctrl+Shift+V (`536-01-copy-and-paste`; the run log
prints the clipboard through `wl-paste`); `shot.png`, then `notes file.txt`, dragged from the
project panel onto the terminal (`536-02-dropped-paths`); `wl-copy` of a two-line text holding an
embedded `ESC [201~`, then Ctrl+Shift+V (`536-03-escape-stripped`); a New Terminal (plain bash)
where the same reply is printed and `stty -icanon -echo -icrnl; cat -v` runs, then the same copy
and paste, and `shot.png` dragged onto it (`536-04-plain-shell`). At Test, #481's scenario runs
again with its stand-in changed to print raw bytes.

## Locked-In Decisions
- D1: "An agent runs in the terminal" has one definition, checked at the moment of the copy or
  the paste: `marley_agent::agent_kind_of` on Zed's foreground command name, through one new
  `Terminal` method that the copy, the paste and the terminal view share.
- D2: Agent terminals only. Orca trims every terminal by default; Marley keeps Zed's exact copy in
  a plain shell, where a selection's indentation is usually wanted.
- D3: The trim is Orca's rule and nothing more: only the run of spaces every non-blank line
  shares.
- D4: Forced brackets only for a paste that holds a line break; a single line has nothing to
  submit early.
- D5: A raw image path only when it is safe to send raw (the extension list, no control
  character, none of the shell's special characters); anything else keeps Zed's quoting.
- D6: The rules are pure functions in `marley_terminal` (new module `paste`); the Zed hunks call
  them: `crates/terminal/src/terminal.rs` (the agent check, the copy's trim, the forced brackets),
  `crates/terminal_view/src/terminal_view.rs` (the agent branch of `add_paths_to_terminal`) and
  `crates/terminal/Cargo.toml` (`marley_agent`), all three paths already in the touchpoints
  ledger, whose rows grow.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an agent CLI is the terminal's foreground program, WHEN the user copies a selection, the clipboard shall hold the selection without the leading spaces every non-blank line shares, with the lines' relative indentation kept. | Shot `536-01-copy-and-paste`; the run log's `wl-paste` |
| REQ-002 | WHILE an agent CLI is the foreground program, WHEN the user pastes text that holds a line break, the terminal shall send it inside bracketed-paste markers, whether or not the program enabled bracketed paste. | Shot `536-01-copy-and-paste` |
| REQ-003 | WHILE an agent CLI is the foreground program, WHEN a PNG, JPEG, GIF or WebP file's path reaches the terminal by a drop or Attach File, the terminal shall send the raw path alone inside a bracketed paste, and any other path shell-quoted as before. | Shot `536-02-dropped-paths` |
| REQ-004 | WHEN a forced bracketed paste's text holds ESC bytes, the terminal shall remove them, so no marker inside the text ends the paste early. | Shot `536-03-escape-stripped` |
| REQ-005 | WHEN rich input sends a multi-line prompt to an agent that has not enabled bracketed paste, the prompt shall arrive as one bracketed paste followed by Enter. | #481's scenario, run again with its stand-in printing raw bytes |
| REQ-006 | WHERE no agent CLI is the foreground program, copy, paste and dropped paths shall behave as in upstream Zed. | Shot `536-04-plain-shell` |

## Phase Plan
- **P1 Plan:** promote the pair, recall, consult the brain, confirm the design in the notes.
- **P2 Code:** the touchpoint rows first; `marley_terminal::paste`; the hunks in `terminal.rs` and
  `terminal_view.rs`; `marley_agent` in `crates/terminal/Cargo.toml`; fmt and clippy clean (Zed's
  `./script/clippy` for the Zed crates); a review of the diff.
- **P3 Test:** write and run the scenario, read every shot; #481's scenario with its stand-in
  updated, and #480's (the agent bar); `script/gates.sh --diff` green; Claude Code's own
  attachment of a dropped PNG, tried by hand in Chad's Marley.
- **P4 Complete:** CHANGELOG; prong 1's T7 row in `docs/marley/three-prong-plan.md`; the three
  touchpoint rows checked against what shipped; the ledger; close, archive, commit.
