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

- **Promotion, 2026-09-29:** every seam re-read. `markdown.rs`: `CodeBlockRenderer` (536),
  `MarkdownElement` (1712) and `new` (1733); the code block's start (2729; the plain path after
  `is_indented`, 2770) and its end (3075), whose hover row (`button_row`) holds the wrap and Copy
  buttons; `CodeBlockKind` in `parser.rs:886` (`Indented`, `Fenced`, `FencedLang`, `FencedSrc`).
  `markdown_preview_view.rs`: the view's `workspace` (67), `render_markdown_element` (1053), the
  element built at 1089 with Copy on hover. `browser.rs`: `LastTerminal` (7650, private),
  `last_terminal` (the view only), `reveal_terminal` (7687), `send_pick` (5128, its deferral and
  window activation). `AnchoredBlocks::at_prompt` (`anchored.rs:359`), `Modes::BRACKETED_PASTE`
  (`terminal.rs:413`), `send_selection::show_toast`. PR-claude-594 (keys after a paste go in a
  later write) does not bite: Ctrl-U goes before the paste. Brain (consultation d49744c5):
  nothing on this seam.

### Design
- **The seam in `markdown`** (Zed crate): `pub type CodeBlockActionFn = Arc<dyn Fn(&CodeBlockKind,
  &str, &App) -> Option<AnyElement>>`, a `code_block_action: Option<CodeBlockActionFn>` field on
  `MarkdownElement` (`None` in `new`) and a builder `code_block_action(..)`. The element keeps the
  kind at the code block's start; at its end, when the field is set, it calls the action with the
  kind and the block's text (Copy's text) and puts the element it returns first in the hover row.
  Which kinds get a button is Marley's rule, so the crate passes every kind; no window is needed to
  build a button, only at its click.
- **The hook in `markdown_preview`** (Zed crate): `pub struct MarleyCodeBlockAction(pub Arc<dyn
  Fn(&WeakEntity<Workspace>, &CodeBlockKind, &str, &App) -> Option<AnyElement>>)`, a gpui global;
  `render_markdown_element` adds `.code_block_action(..)` binding the view's workspace when the
  global is set.
- **The workbench** (`crates/marley_workbench/src/markdown_commands.rs`, new): sets the global at
  `init`. For a shell language or none, it returns an `IconButton` (`IconName::Terminal`, tooltip
  "Insert in Terminal", in `CopyButton`'s size). Its click: upgrade `LastTerminal` (else a toast
  "No terminal to insert into: click in one, then try again."); read the terminal's blocks: at the
  prompt (else a toast naming the running block's command or the foreground program); a text of
  several lines needs `Modes::BRACKETED_PASTE` (else a toast); then, deferred as `send_pick` defers,
  activate the terminal's window when it is another, `reveal_terminal`, focus it, and
  `terminal.input(b"\x15")` then `terminal.paste(text)`, the text's trailing newline trimmed.
- **Shared pieces:** `browser.rs` gains `pub(crate) fn last_terminal_with_window`, the view and its
  window from `LastTerminal`; `reveal_terminal` is `pub(crate)` already.
- **File manifest.**
  - Zed crates: `crates/markdown/src/markdown.rs`, `crates/markdown_preview/src/markdown_preview_view.rs`.
  - Marley crates: `crates/marley_workbench/src/markdown_commands.rs` (new), `browser.rs`
    (`last_terminal_with_window`), `marley_workbench.rs` (the module, the init),
    `crates/marley_workbench/Cargo.toml` (`markdown`, `markdown_preview`); `script/e2e/530-runnable-markdown-commands.sh` (Test).
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

## Phase 2 — Code
- **Built:**
  - `markdown.rs` (Zed): `CodeBlockActionFn` (an `Rc`, as the element's other callbacks are), the
    `code_block_action` field (`None` in `new`) and builder; the render loop keeps a code
    block's kind at its start (`marley_code_block_kind`) and, at its end, puts the action's
    element first in the hover row, wrapped in a div whose id is the block's start, so each
    block's button has an id of its own, as Copy's does.
  - `markdown_preview_view.rs` (Zed): the `MarleyCodeBlockAction` global; `render_markdown_element`
    passes an action that binds the preview's workspace while the global is set.
  - `marley_workbench/src/markdown_commands.rs` (new): `is_shell` (a bare fence, or a fence whose
    info string's first word is `sh`, `shell`, `bash`, `zsh` or `fish`); `button`, Insert in
    Terminal (`IconName::Terminal`) on a shell block with text; `insert`: the terminal the focus
    entered last, else a toast; not at its prompt, a toast naming the running block's command or
    the foreground program; several lines without `Modes::BRACKETED_PASTE`, a toast; otherwise,
    after this update, the terminal's window activated when it is another, `reveal_terminal`, the
    focus, Ctrl-U and `Terminal::paste` of the text less its trailing newline.
  - `browser.rs`: `last_terminal_with_window`. `marley_workbench.rs`: the module and its init.
    `Cargo.toml`: `markdown` and `markdown_preview`.
- **Deviations:** the action takes the block's `CodeBlockKind`, not its language, so the rule of
  which blocks get a button stays Marley's; it takes `&App`, since only the click needs a window.
  The per-block id wrapper was added in review: every button would otherwise share one id.
- **Review of the diff:** no path sends a carriage return; nothing is typed away from a prompt;
  the insertion runs after the click's update, as a sent pick's does, since the terminal's pane
  may be the preview's; the markdown crate's field is `None` for every other caller.
- **Gate:** run 1 red: `arc_with_non_send_sync` on the preview's closure, which captures the
  hook's `Arc<dyn Fn>`; the action became an `Rc`. Its warnings (`mermaid.rs:220`, `:272`,
  `markdown_preview_view.rs:1567`, `:1584`, `:1665`) are Zed's own code: pre-existing, not in
  scope. Run 2 on the final tree, with the scenario: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/530-runnable-markdown-commands.sh` under `compositor sway`.
- **Runs 1 to 3, the scenario's faults:** run 1 measured the preview (the guessed button spot
  clicked the block's body, so nothing was typed). Run 2's last check failed on the screen, not
  the product: step 05's cancelled two lines stay in the scrollback, so grepping the whole screen
  for them always matched; the check now reads the prompt line only, as #528's does. Run 3 green;
  a hover on the bare fence was added for REQ-001's "no language", then the gate and run 4.
- **Run 4, on the gated tree:** exit 0; both checks pass (the prompt holds the command alone; the
  prompt line holds nothing after the refused paste). Every shot read:
  - `530-00-preview`: the terminal on the left, the runbook's preview on the right.
  - `530-01-button`, `530-01b-bare-fence` (REQ-001): the `bash` block and the bare `ls` fence
    show the terminal button before Copy.
  - `530-02-no-button`, `530-02b-indented` (REQ-002): the `rust` and the indented blocks show
    Copy alone.
  - `530-03-inserted` (REQ-003): `abc` gone, `$ echo hello from the runbook` at the prompt, not
    run, the terminal's tab in front.
  - `530-04-ran` (REQ-004): Enter ran it, `hello from the runbook`.
  - `530-05-two-lines` (REQ-005): both lines at the prompt as one paste, not run.
  - `530-06-busy` (REQ-006): the toast "That terminal is running sleep 30: nothing was typed."
  - `530-07-no-bracketed-paste` (REQ-007): after `bind 'set enable-bracketed-paste off'`, the
    toast "That shell has bracketed paste off, so these lines would run one by one: nothing was
    typed." and an empty prompt.
  - `530-08-no-terminal` (REQ-008): after `exit` closed the terminal, the toast "No terminal to
    insert into: click in one, then try again."

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: shell commands in the Markdown preview go to the
  terminal); `docs/marley/three-prong-plan.md` T4; `docs/marley_architecture/marley_workbench.md`
  (runnable commands in the Markdown preview); the two new touchpoint rows describe what shipped.
- **Knowledge:** `AD-claude-530-runbook-commands-go-to-the-last-terminal-at-its-prompt-001`,
  `L-claude-530-a-nothing-typed-check-reads-the-prompt-line-001`. No `F-…` block: the review's
  shared-id gap was fixed before any run, and the gate's red was a lint.
  Brain: the decision on consultation d49744c5, follow-up by 2026-10-29.
- **Closed** TICKET-530, archived the pair.
