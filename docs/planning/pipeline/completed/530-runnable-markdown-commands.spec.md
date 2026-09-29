---
pipeline_id: ca6b3d30-4034-4150-acce-70a15b638743
ticket: docs/planning/tickets/open/TICKET-530-runnable-markdown-commands.md
status: Phase 4 — Complete PASS
title: "Shell commands in the Markdown preview go to the terminal"
type: feature
slice: prong 1 with the editor (Warp once-over item 6)
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/496-element-picker.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md]
---

## Title
A shell code block in Zed's Markdown preview gets a button that puts its command at the prompt of
the terminal used last, without running it, and takes the user there: Warp's runnable commands in
its Markdown viewer, for runbooks and READMEs in Marley.

## Scope
### In
- **The button**: on the hover row of a fenced code block in the Markdown preview whose fence names
  `sh`, `shell`, `bash`, `zsh` or `fish`, or names no language, beside Copy: Insert in Terminal
  (`IconName::Terminal`). Other languages, indented code blocks and every other Markdown view
  (the Agent Panel, hovers) are unchanged.
- **A click**: the terminal the user focused last (#496's `LastTerminal`); while its shell waits
  at its prompt (Marley's blocks, `at_prompt`), Ctrl-U clears the line and the block's text goes
  in with `Terminal::paste`, no carriage return; the terminal comes to the front and takes the
  focus, as a sent pick does (`reveal_terminal`).
- **Several lines** go in only when the shell turned bracketed paste on, as one paste the shell
  holds until Enter; without bracketed paste nothing is typed and a toast says why, since the
  lines would run one by one.
- **Nothing typed, and a toast** in the preview's workspace, when no terminal was focused yet or it
  is gone, or when the terminal's shell is not at its prompt (a program runs, or the shell has no
  Marley integration), naming what runs when Marley knows it.
- **Two small seams in Zed's crates**: `markdown`'s `MarkdownElement` takes an optional code-block
  action, an element added to a code block's hover row for the block's fence language and text;
  `markdown_preview` passes one when Marley's global hook is set. Marley's workbench decides which
  blocks get the button and what it does.

### Out (explicitly deferred)
- Warp's keys: Ctrl-Up and Ctrl-Down to select code blocks, Ctrl-Enter to insert, Ctrl-Shift-L back
  to the terminal.
- `{{param}}` placeholders filled in as arguments (Warp's workflow arguments).
- Opening a terminal when none was focused (AD-claude-496 declined guessing a terminal).
- The button on code blocks in the Agent Panel's threads and other Markdown views.
- Terminals without Marley's integration (fish until #466, Zed's remote-project terminals), which
  report no prompt.

## Reference (§20)
- **Warp, the Markdown viewer's runnable commands**
  (https://docs.warp.dev/terminal/more-features/markdown-viewer/; the Warp once-over,
  `docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 6): shell code blocks show a run
  icon (`>_`); a click inserts the command "into the terminal input" of "your active terminal
  session" without executing it; code blocks count as shell commands when they have no language
  tag or use `sh`, `shell`, `bash`, `fish`, `zsh` or `warp-runnable-command`; only fenced blocks,
  never inline code; all code blocks keep their copy buttons. Marley keeps the button, its
  language list (without Warp's own tag), fenced blocks only, insertion without running, and the
  copy button; the keys and the parameters come later. No Warp code.
- **Upstream Zed:** the Markdown preview (`markdown_preview`) and the `markdown` crate's default
  code-block renderer, whose hover row holds Copy (and a wrap toggle where asked); the preview
  keeps both, and the new button joins that row.

### Prior art
- **Behavior maps and reports.** The once-over, item 6 ("Zed's Markdown preview has nothing like
  it, and runbooks are mostly commands to copy"). Orca: its rich Markdown editor (report 05 §2.14)
  and its document previews in the browser pane (report 03 §2.16) run no commands; nothing to
  take.
- **Published material.** Warp's page above; bash and zsh turn on bracketed paste (mode 2004) at
  their prompts, and hold a bracketed paste as text until Enter.
- **The code we already ship.**
  - Zed's `markdown`: `CodeBlockRenderer::Default` and the hover row built at a code block's end
    (`render_copy_code_block_button`); `CodeBlockKind::FencedLang` carries the fence's language
    name. No Zed view acts on a code block beyond copying it (the Agent Panel, the ACP log,
    hovers, signature help and diagnostics all use the default renderer).
  - Zed's `markdown_preview`: `render_markdown_element`, which builds the preview's element.
  - Marley's #496 pick: `LastTerminal`, kept by `track_terminals` as the focus enters a terminal;
    `reveal_terminal`; `send_pick`'s paste and its message when no terminal was used yet.
  - `Terminal::paste` (bracketed while the program asked), `AnchoredBlocks::at_prompt`, #474's
    Ctrl-U before Rerun, and `workspace::Toast`.

## UI proof
UI-AFFECTING. `script/e2e/530-runnable-markdown-commands.sh` (`compositor sway`, for the hover and
the clicks). The scenario's own bash; `RUNBOOK.md` in the scratch repository with a `bash` block
(`echo hello from the runbook`), a bare fenced block (`ls`), a `bash` block of two lines, a `rust`
block and an indented block. Steps and shots: the first terminal clicked, `RUNBOOK.md` opened through the
file finder, `markdown: open preview to the side`; the pointer on the `bash` block (`530-01-button`); on the `rust` block
(`530-02-no-button`); `abc` typed at the terminal's prompt, then the `bash` block's button
(`530-03-inserted`: the terminal in front with the focus, `echo hello from the runbook` alone at
the prompt, no output); Enter (`530-04-ran`); the two-line block's button (`530-05-two-lines`: both
lines at the prompt, not run); `sleep 30` running and the button (`530-06-busy`: the toast naming
`sleep 30`, nothing typed); a second terminal where `bind 'set enable-bracketed-paste off'` was typed,
and the two-line block's button (`530-07-no-bracketed-paste`:
the toast, nothing typed); the terminals closed and the button (`530-08-no-terminal`: the toast).

## Locked-In Decisions
- D1 — The terminal focused last, as #496's picks go there (AD-claude-496), brought to the front
  with the focus so Enter runs the command.
- D2 — Only at a shell prompt, which Marley's blocks report: a program in the terminal (vim, a
  REPL, a running command) would take the text as its own input.
- D3 — Nothing runs: no carriage return, and several lines only as a bracketed paste.
- D4 — Ctrl-U first, as Rerun sends it (#474 D2), so the line holds the command and nothing typed
  before it.
- D5 — Warp's shell languages, less its own tag; a `console` block (a session with its output) gets
  no button, as in Warp.
- D6 — The seam in Zed's `markdown` crate is generic (an element for a code block's hover row,
  from a callback), only the preview passes one, and the rules are Marley's, in
  `marley_workbench`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the pointer is over a fenced code block in the Markdown preview whose fence names `sh`, `shell`, `bash`, `zsh` or `fish` or no language, the block shall show Insert in Terminal beside Copy. | Shot `530-01-button` |
| REQ-002 | WHILE the pointer is over a code block of another language, or an indented one, the block shall show no Insert in Terminal button. | Shot `530-02-no-button` |
| REQ-003 | WHEN the user clicks Insert in Terminal while the terminal focused last waits at its shell prompt, the system shall clear the prompt's line, put the block's command there without running it, and bring that terminal to the front with the focus. | Shot `530-03-inserted` |
| REQ-004 | WHEN the user then presses Enter, the shell shall run the inserted command. | Shot `530-04-ran` |
| REQ-005 | WHEN the block holds several lines and the shell turned bracketed paste on, the system shall put them in as one paste without running them. | Shot `530-05-two-lines` |
| REQ-006 | WHEN the terminal focused last is not at its shell prompt, the system shall type nothing and say in a toast what runs there. | Shot `530-06-busy` |
| REQ-007 | WHEN the block holds several lines and the shell has not turned bracketed paste on, the system shall type nothing and say why in a toast. | Shot `530-07-no-bracketed-paste` |
| REQ-008 | WHEN no terminal was focused yet, or it has closed, the system shall type nothing and say so in a toast. | Shot `530-08-no-terminal` |
| REQ-009 | The diff gate shall be green. | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design and the test plan in the notes.
- **P2 Code** — the touchpoints rows first; the code-block action in `markdown`; the hook in
  `markdown_preview`; the button and the insertion in `marley_workbench`; fmt and clippy clean; a
  review of the diff (no carriage return on any path; nothing typed outside a prompt).
- **P3 Test** — write and run the scenario, read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the workbench's notes under `docs/marley_architecture/` (§21), a check
  that the new rows in `docs/marley/zed-touchpoints.md` describe what shipped, ledger capture
  (§19), close, archive, commit.
