# Shell commands in the Markdown preview go to the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-530-runnable-markdown-commands.md
- **Pipeline spec:** 530-runnable-markdown-commands.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** Chad approved all seven items of the Warp once-over on 2026-09-25. Item 6, as
  asked: shell code blocks in Zed's Markdown preview (`crates/markdown_preview/`) get a button that
  puts the command into the active terminal's input without running it.
- **Classification:** feature, size S. Zed crates `markdown` (a generic code-block action) and
  `markdown_preview` (the hook, passed to its element); Marley crate `marley_workbench` (which
  blocks, the button, the insertion).
- **Recall (§18.3):**
  - AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001: a pick pastes into the terminal
    the user focused last, brings it to the front with the focus, presses no Enter, and says so when
    no terminal was used yet instead of guessing one. The same rules serve a runbook's command.
  - AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001 and #474's D2: Rerun
    sends Ctrl-U first so a half-typed line does not prefix the command.
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001: `at_prompt` is the
    blocks' word on whether the shell waits at its prompt.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: bringing a terminal forward in a
    pane moves the focus; here that is wanted, as in `send_pick`.
- **Discovery (the seams, checked 2026-09-25):**
  - `crates/markdown/src/markdown.rs`: `CodeBlockRenderer` (536) with `Default { copy_button_visibility,
    wrap_button_visibility, border }` and `Custom`; `MarkdownElement` (1712), its callbacks as
    `Option<Rc<..>>` fields and builder methods (`code_block_renderer`, 1777); the code block's start
    (`MarkdownTag::CodeBlock { kind, .. }`, 2729) and end (`MarkdownTagEnd::CodeBlock`, 3075), where
    the hover row is built (`button_row`, 3111) with wrap and copy (`render_copy_code_block_button`,
    3482); `code_block_language` (1600) resolves a language, not the fence's name.
  - `crates/markdown/src/parser.rs:886` `CodeBlockKind`: `Indented`, `Fenced` (no language),
    `FencedLang(SharedString)` (893), `FencedSrc`.
  - `crates/markdown_preview/src/markdown_preview_view.rs:1053` `render_markdown_element`, which sets
    `CodeBlockRenderer::Default` with copy on hover (1090); the view holds its `workspace`.
  - `crates/markdown_preview/src/markdown_preview.rs`: `OpenPreview` and `OpenPreviewToTheSide`
    from `zed_actions`.
  - `crates/marley_workbench/src/browser.rs`: `send_pick` (3645), its message when no terminal was
    used (3659) and its paste (3685); `LastTerminal` (5400), kept by `track_terminals` (5409);
    `reveal_terminal` (5430). All three are private to `browser.rs` today.
  - `crates/terminal/src/terminal.rs:2582` `paste`: bracketed while `Modes::BRACKETED_PASTE` is set,
    else each newline becomes a carriage return, which would run the lines.
  - `crates/marley_terminal/src/anchored.rs:172` `at_prompt`.
  - `crates/workspace/src/workspace.rs:730` `Toast`.
  - `crates/marley_workbench/Cargo.toml`: no `markdown_preview` dependency today.
- **Decisions:** D1 to D6 in the spec.

### Design
- **The seam in `markdown`** (Zed crate): `pub type CodeBlockActionFn = Arc<dyn Fn(Option<&str>, &str,
  &mut Window, &mut App) -> Option<AnyElement>>`, a `code_block_action: Option<CodeBlockActionFn>`
  field on `MarkdownElement` (`None` in `new`) and a builder `code_block_action(..)`. The element
  keeps each code block's `CodeBlockKind` from its start event; at the block's end, when the field is
  set and the block is fenced, it calls the action with the fence's language (`None` for a bare
  fence) and the block's text, and puts the element it returns first in the hover row, before
  Copy. Indented blocks are never passed.
- **The hook in `markdown_preview`** (Zed crate): `pub struct MarleyCodeBlockAction(pub Arc<dyn
  Fn(&WeakEntity<Workspace>, Option<&str>, &str, &mut Window, &mut App) -> Option<AnyElement>>)`, a
  gpui global; `render_markdown_element` adds `.code_block_action(..)` binding the view's workspace
  when the global is set.
- **The workbench** (`crates/marley_workbench/src/markdown_commands.rs`, new): sets the global at
  `init`. For a shell language or none, it returns an `IconButton` (`IconName::Terminal`, tooltip
  "Insert in Terminal", in `CopyButton`'s size). Its click: upgrade `LastTerminal` (else a toast
  "No terminal to insert into: click in one, then try again."); read the terminal's blocks: at the
  prompt (else a toast naming the running block's command or the foreground program); a text of
  several lines needs `Modes::BRACKETED_PASTE` (else a toast); then, deferred as `send_pick` defers,
  activate the terminal's window when it is another, `reveal_terminal`, focus it, and
  `terminal.input(b"\x15")` then `terminal.paste(text)`, the text's trailing newline trimmed.
- **Shared pieces:** `LastTerminal` and `reveal_terminal` become `pub(crate)` in `browser.rs` (or
  move to a small `terminals.rs` beside it), so the pick and the runbook share them.
- **File manifest.**
  - Zed crates: `crates/markdown/src/markdown.rs`, `crates/markdown_preview/src/markdown_preview_view.rs`.
  - Marley crates: `crates/marley_workbench/src/markdown_commands.rs` (new), `browser.rs`
    (visibility), `marley_workbench.rs` (the module, the init), `crates/marley_workbench/Cargo.toml`
    (`markdown_preview`); `script/e2e/530-runnable-markdown-commands.sh` (Test).
- **Ledger rows:** two new rows in `docs/marley/zed-touchpoints.md`, for
  `crates/markdown/src/markdown.rs` and `crates/markdown_preview/src/markdown_preview_view.rs`,
  written before the hunks.

### E2E plan
`script/e2e/530-runnable-markdown-commands.sh`, `compositor sway`. Setup: a scratch HOME with
`PS1='$ '`; `RUNBOOK.md` in the scratch repository with the five blocks.

| REQ | Step | Shot |
|---|---|---|
| REQ-001 | a click in the first terminal; the file finder, `RUNBOOK.md`; `markdown: open preview to the side`; the pointer on the `bash` block | `530-01-button` |
| REQ-002 | the pointer on the `rust` block, then the indented one | `530-02-no-button` |
| REQ-003 | `abc` typed in the terminal; the `bash` block's button | `530-03-inserted` |
| REQ-004 | Enter | `530-04-ran` |
| REQ-005 | the two-line block's button | `530-05-two-lines` |
| REQ-006 | Ctrl-C; `sleep 30`; the `bash` block's button | `530-06-busy` |
| REQ-007 | Ctrl-C; a new center terminal (Ctrl-~); `bind 'set enable-bracketed-paste off'` typed there; the two-line block's button | `530-07-no-bracketed-paste` |
| REQ-008 | both terminals closed; the `bash` block's button | `530-08-no-terminal` |
| REQ-009 | `just gate-diff` | the gate's exit |

### Risks
- **The preview re-renders** on each edit of the source; the button is rebuilt with the element, and
  its click reads the block's text as rendered then. The text comes from the code block's content
  range, as Copy's does, so the two agree.
- **The language name** is the fence's info string as the parser keeps it (`FencedLang`); an info
  string with attributes (`bash title="x"`) may keep more than the name. Take the first word.
- **Zed's `markdown` crate** is shared by every Markdown view; the field is `None` by default, so no
  other view changes (checked: every other caller builds its element without it).
