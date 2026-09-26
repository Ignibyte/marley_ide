# Send a block to the agent, and Ask the agent under a failed block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-555-send-a-block-to-the-agent.md
- **Pipeline spec:** 555-send-a-block-to-the-agent.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note's recommendation 2 (2026-09-25): Send to Agent and the
  "Ask the agent" chip under a failed block; Chad's answer of 2026-09-26 to its first question:
  "copy as context" is both, Send to Agent here and Copy as Markdown in #554. Drafted in the
  spec batch of 2026-09-26.
- **Classification / tier:** feature, size S each once #549 (the target rule and the send) and
  #554 (the selection, the menu hook, the Markdown form) have landed. One Zed hunk: the chip's
  hook in the block element.
- **Recall (§18.3):**
  - AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001: the line is the reference
    and the tool the content; no Enter; the terminal focused. The reference here names
    `terminal_read` as a pick's names `browser_pick`.
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001 and
    F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001: redact what an agent gets,
    the whole text before any cut; never the terminal's buffer or the clipboard.
  - AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001: what
    Marley draws in the terminal goes through a hook; the chip does.
  - AD-claude-474-hover-actions-live-on-one-element-per-block-001 and
    F-claude-474-the-occluding-buttons-hid-under-the-pointer-001: a button on the block's element
    sits in a wrapper that stops the press; the chip is such a button.
  - F-claude-546-block-reads-answered-from-the-alternate-screen-001: `block_output` reads the
    main screen; the inline form reads it too.
  - L-claude-491-a-session-per-agent-needs-room-and-a-close-001: the stand-in agent's MCP
    session; the scenario's `mcp_agent` calls close theirs.
  - Brain: no page on sending blocks to agents (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_workbench/src/browser.rs:3645` `send_pick` (`:3676` the deferred activate,
    reveal, focus, `:3685` paste); `:5658` `pick_line`
    (`[browser pick {id}: {summary} on {host/path}; browser_pick id {id}] {caption}`); `:3779`
    a toast.
  - `crates/marley_workbench/src/mcp.rs:49` `MAX_READ_LINES`, `:52` `MAX_READ_BYTES`, `:314`
    `agent_redactor` (None when the user turned redaction off), `:323` `for_agents`, `:350`
    `terminals`, `:381` `terminal_with_id` (the view's entity id), `:397` `terminal_list`,
    `:434` `terminal_blocks` (`:453` the command redacted), `:477` `block_entry`, `:508`
    `terminal_read` (`:525` the command, `:528` the whole output, then `:547` `tail`).
  - `crates/marley_mcp/src/registry.rs:78` `REGISTRY`; `:113` the `terminal_read` row; `:820`
    `terminal_argument_schema`; `:856` `terminal_blocks_schemas`; `:916` `terminal_read_schemas`
    (`terminal`, `block`; the answer's `output`, `truncated`, `redacted`).
  - `crates/marley_mcp/src/redact.rs:117` `Redacted { text, count }`; `:126` `Redactor`; `:152`
    `redact`.
  - `crates/terminal_view/src/terminal_element.rs:1632` the prompt gate; `:2237` `marley_wash`;
    `:2262` `marley_pill`; `:2311` `marley_keep_from_terminal`; `:2320` `marley_block`
    (`:2330` Copy Output's button, `:2343` Rerun's, `:2368` `visible_on_hover`).
  - `crates/terminal_view/src/terminal_view.rs:141` `MarleyTerminalFooter` (the hook shape);
    `:590` Add to Agent Thread.
  - `crates/marley_workbench/src/agent_bar.rs:305` `claude_plugin_chip` (a chip's look);
    `:129` `agent_in`.
  - `crates/marley_workbench/src/agent_events.rs:30` `seat`;
    `crates/marley_agent/src/claude_events.rs:303`, `:353` (Waiting).
  - `crates/marley_terminal/src/anchored.rs:38` `AnchoredBlock`; `:63` `BlockTimes`; `:170`
    `times`; `:176` `at_prompt`.
  - `crates/marley_workbench/keymap.json:17` the `Terminal` block; Zed leaves
    `ctrl-shift-enter` unbound in `Terminal` (`assets/keymaps/default-linux.json:1295` to
    `:1342`).
  - The e2e stand-in's `terminals`, `blocks` and `terminal-read` commands
    (`script/e2e/browser-fixture.sh:25` to `:30`).
- **Decisions:** D1 to D7 in the spec.

### Design
- **`send_block.rs`.** `init`: `SendBlockToAgent { block: Option<usize> }` on the workspace
  (the menu passes the block; the key sends the selected one); the `MarleyBlockChip` global set
  to `chip`.
- **`text_for(view, block, cx) -> String`**: the block, its output (`block_output`), the
  redactor (`mcp::agent_redactor`); the Markdown form when `!block_output_kept` or the redacted
  output is at most 32 lines and 4 KiB, else the reference:
  `[terminal {id} block {index}: {command}, exit {code}; terminal_read terminal={id} block={index}] `
  with the command redacted, `running` for a running block.
- **`send_block(view, block, window, cx)`**: `agent_targets` minus the block's own terminal;
  none: a toast ("No agent is running in a terminal"); one: `send_selection::send`; several: the
  picker with `send` as its confirm.
- **`chip(context, window, cx) -> Option<AnyElement>`**: the block is the last, Finished,
  `exit_code` is `Some(code)` with `code != 0`, `at_prompt`, `agent_in(terminal)` is None, and
  `agent_targets` minus this terminal is not empty; a small `Button` labelled `Ask the agent`
  in `marley_keep_from_terminal`'s wrapper, whose click dispatches `SendBlockToAgent { block }`
  on the view's focus handle.
- **The hook** (`terminal_element.rs`): `pub struct MarleyBlockChip(pub Arc<dyn Fn(&MarleyBlockChipContext,
  &mut Window, &mut App) -> Option<AnyElement>>)` with `impl Global`, declared in
  `terminal_view.rs` beside the footer hook; `marley_block` calls it for the block and places
  the element on the first row before the pill.
- **The menu and the key.** #554's section gains `Send to Agent`
  (`SendBlockToAgent { block: Some(index) }`); `keymap.json`'s
  `Terminal && MarleyBlockSelected` block gains `ctrl-shift-enter`.
- **File manifest.** Zed: `crates/terminal_view/src/terminal_element.rs` (the hook's call),
  `crates/terminal_view/src/terminal_view.rs` (the hook's type, beside #554's). Marley:
  `crates/marley_workbench/src/send_block.rs` (new), `block_selection.rs` (the item),
  `marley_workbench.rs` (the module, the action), `keymap.json`. Scripts:
  `script/e2e/555-send-a-block-to-the-agent.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal_element.rs` row gains the
  chip's call, the `terminal_view.rs` row its type.

### E2E plan

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-004 | `claude` in terminal 1; terminal 2: `seq 1 200; false` | `555-01-chip` |
| REQ-001, 004 | click the chip | `555-02-reference-sent`; Return there; `mcp_agent terminals` and `blocks` hold the id and index |
| REQ-002, 008 | terminal 2: `false`; `ctrl-up`, `ctrl-shift-enter` | `555-03-inline-sent`: the fence, `$ false`, `exit 1 · …`, as `got:` lines |
| REQ-003 | `echo token=ghp_<36 chars>` | `555-04-before-send`: exact in terminal 2 |
| REQ-003 | right-click, Send to Agent | `555-05-redacted`: `[redacted: github token]` in terminal 1 |
| REQ-006 | `wait` to the claude stand-in; a block in terminal 2, Send to Agent | `555-06-refused`: the toast, nothing typed |
| REQ-007 | `codex` in a third terminal; Send to Agent from terminal 2 | `555-07-picker`: claude and codex, not terminal 2 |
| REQ-005 | both stand-ins ended; `false` in terminal 2 | `555-08-no-agent-no-chip` |

The stand-ins enable no bracketed paste, so `Terminal::paste` turns the Markdown's newlines
into returns and each line prints as its own `got:`; a real Claude Code takes the same paste
whole. Not reachable: a real agent's reply to the reference; Test runs one real Claude Code
once in a pty (L-claude-482's method) and reads its `terminal_read` call in the bridge's log.

### Risks
- The reference's id changes at every launch; an agent that keeps a reference across a relaunch
  reads the wrong terminal or none. #520's UUID fixes it; until then the reference is for the
  turn it is sent in.
- The size rule counts after redaction, so a block full of secrets can shrink into the inline
  form; the redacted text is what the agent gets either way.
- The chip sits on the pill's row and shares it with Copy and Rerun on hover; the review checks
  the widths at the rail's narrowest terminal.
- A block in an agent's terminal after the agent exited (a shell again) can carry the chip; the
  target rule still never picks that terminal for itself.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
