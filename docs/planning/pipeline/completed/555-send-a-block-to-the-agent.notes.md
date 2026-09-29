# Send a block to the agent, and Ask the agent under a failed block — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-555-send-a-block-to-the-agent.md
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

- **Promotion (2026-09-29), against what #549 and #554 shipped:**
  - #549: `send_selection.rs` keeps `Target`, `agent_targets` (`:268`), `target_of`, the picker
    `TargetPicker::new(rows, placeholder, on_pick, ..)` with `Row`/`Pick`/`OnPick` (#522's
    generalization), and `send` (`:352`: rich input takes the text; a waiting seat gets a toast
    and nothing; else the terminal revealed, focused, one paste, no Enter), which takes a
    `Selection`; it splits into `send_text(target, text, ..)` for the block's text.
  - #554: the Block section is `blocks.rs`'s `block_menu`, asked by `MarleyTerminalBlockMenu`
    with the block's index; the Markdown form is `AnchoredBlock::markdown(output, took)`; the
    selected block is `MarleyBlockSelection::selected`; the keymap's `Terminal &&
    MarleyBlockSelected` block is where `ctrl-shift-enter` goes.
  - `mcp.rs`: `agent_redactor(cx) -> Option<Arc<Redactor>>` (`:440`), `terminal_list`'s `id` is
    the view's entity id (`:805`). `terminal_element.rs`: `marley_block` (`:2376`) builds each
    block's first row (the hover actions, then the pill); the block elements come from the
    `marley_starting` map in `prepaint`.
  - Changes to the design: `block_selection.rs` is `blocks.rs`; the chip hook is
    `MarleyBlockChip(Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>, usize, &App) ->
    Option<AnyElement>>)`, called in `prepaint`'s map and handed to `marley_block` as an
    element; the chip's click calls `send_block::send` directly; `SendBlockToAgent` is a plain
    action for the key (the menu calls the function). No real Claude Code runs in the check (the
    rule for scenarios); the reference's use by an agent rests on `terminal_read`'s own check.
  - REQ-009 is the gate alone (no golden set, 2026-09-29).
  - Brain: consultation cffd3e517f5b4d9390680db51000046c, nothing on this seam.

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

### Visual check plan

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
whole. Not reachable: a real agent's reply to the reference (no scenario starts the real `claude`).

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
- **Built:**
  - `send_selection.rs`: `send` split into the reference and `send_text(target, text, ..)`, the
    paste, the rich input and the waiting seat's toast as before; `show_toast` is `pub(crate)`.
  - `send_block.rs` (new): `text_for` (the output redacted first, then inline when at most 32
    lines and 4 KiB or gone: the whole Markdown redacted; else `[terminal <id> block <n>:
    <command, redacted>, exit <code>; terminal_read terminal=<id> block=<n>] `), `targets` (the
    window's agent terminals but the block's own), `send` (deferred by `window.defer`, then a
    toast, `send_text` or the picker), and `chip` (the newest block, Finished with a non-zero exit,
    `at_prompt`, no agent in front, a target: a tinted `Ask the agent` button).
  - `blocks.rs`: Send to Agent first in the Block section; `send_focused` for
    `SendBlockToAgent`. `marley_workbench.rs`: the module, `send_block::init`, the action.
    `keymap.json`: `ctrl-shift-enter`.
  - `terminal_view.rs`: `MarleyBlockChip`. `terminal_element.rs`: the hook asked in `prepaint`'s
    map, its element handed to `marley_block` before the pill; and for the newest block when its
    first row is above the screen, an element of its own on the block's last row.
- **Deviations:**
  - The last-row chip: writing the scenario (`seq 1 200; false`) showed that the chip on the
    first row, the only row the block element has, is off screen for exactly the long failures it
    is for. The newest block gets the chip on its last row, just above the prompt, when its first
    row has scrolled away.
  - `send` defers its work: `ctrl-shift-enter`'s action runs while the workspace is being
    updated, and the send reads and updates it.
  - Send to Agent leads the Block section, so #554's scenario, which counts items from the end,
    is unchanged.
  - No action argument: the menu and the chip call `send_block::send` with the block;
    `SendBlockToAgent` is the key's.
- **Review of the diff:** the redaction runs on the output before the size test and on the whole
  Markdown after, and the reference's command is redacted; the block's own terminal is filtered
  from every target list; the chip's four conditions are the spec's D5; the chip hook runs in the
  element's `prepaint`, where the element already reads its view (`:1446`), not in a render.
- **Gate:** `just gate-diff`, GATE GREEN [diff], with the scenario in the tree (log in the
  scratchpad).

## Phase 3 — Test
- **The scenario:** `script/e2e/555-send-a-block-to-the-agent.sh`, `compositor sway`, #549's
  stand-ins `claude` and `codex` on the terminal's PATH; Zed's own agent off in the run's settings
  copy (`zed_agent_off`, since run 2; below); terminals brought forward by their tabs (`alt-1` to
  `alt-3`); Send to Agent chosen from the Block section's end.
- **Run 1:** every check to the redaction passed; the redaction check failed because it named the
  kind: the token came back `token=[redacted: secret]`, the `key=value` rule matching before the
  GitHub one. The check now takes any marker. The Markdown of a block with no output had an empty
  line inside the fence; `AnchoredBlock::markdown` (#554) now leaves the output line out when it
  is empty. Gate again: GATE GREEN [diff].
- **Run 2, a scenario fault that opened Zed's own agent:** once the claude stand-in's seat waited,
  the rail's Needs you section moved its rows down, the scenario's click on the second terminal's
  rail row missed, and the right-click landed in the claude terminal on its running block. There
  both Reinput items are disabled, which the menu's keys skip, so End and six Ups went past the
  Block section into Zed's items and chose Add to Agent Thread: a New Claude Agent Thread opened
  in a split (its shots 555-07, 555-08). No prompt was typed into it, and no agent process
  outlived the run's Marley (checked after the run). The check of the waiting agent had passed
  without testing it. Fixed in the scenario: the tabs, not the rail, bring a terminal forward;
  Zed's agent is off in the run's settings, so the menu offers neither Inline Assist nor Add to
  Agent Thread and no step can start it.
- **Run 3:** gate green first; exit 0, every check passed, no `panicked` in any log; the focus
  report: nothing on Hyprland, the run's sway stopped.
- **The shots (runs 2 and 3, the same code):**
  - 555-01-chip: after `seq 1 200; false`, `Ask the agent` on the block's last row (`200`),
    right-aligned above the prompt (REQ-004).
  - 555-02-reference-sent: the claude terminal in front, focused, with `[terminal 21474836748
    block 0: seq 1 200; false, exit 1; terminal_read terminal=21474836748 block=0] ` at its
    prompt, unsent; after Return the stand-in's `got:` holds it, and `mcp_agent terminals` lists
    terminal `21474836748` (REQ-001).
  - 555-03-inline-sent: `false`, `ctrl-up`, `ctrl-shift-enter`: the stand-in's `got:` lines are
    the fence, `$ false`, the fence and `exit 1 · 0 ms · <folder>` (REQ-002, REQ-008).
  - 555-04-before-send / 555-05-redacted: the token exact in its own terminal; at the agent
    `$ echo token=[redacted: secret]` and `token=[redacted: secret]` (REQ-003).
  - 555-06-refused: after `wait` the seat waits (Needs you: Permission for Bash); Send to Agent
    from the second terminal types nothing and the toast reads "Claude Code in repo waits on a
    permission or a question; nothing was sent." (REQ-006).
  - 555-07-picker: with codex in a third terminal, "Send the block to…" lists `Codex · repo` and
    `Claude Code · repo · waiting`, not the block's own terminal (REQ-007).
  - 555-08-no-agent-no-chip: both stand-ins ended, every tab `repo — bash`; a new `false` shows
    `exit 1` and no chip (REQ-005).
- **Seen, not in scope:** after the tab keys the rail is hidden in 555-05 to 555-08; run 1, which
  clicked the rail, kept it. Not followed up here.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #555); `marley_workbench.md` (a section for `send_block.rs`,
  `send_text` in #549's); `terminal_blocks.md` (the Markdown's empty output); the touchpoints rows
  for `terminal_view.rs` and `terminal_element.rs` (written before the code); the plan's T7 row.
- **Knowledge:** F-claude-555-a-menu-walked-by-its-keys-opened-zeds-own-agent-001,
  PR-claude-a-scenario-that-opens-zeds-terminal-menu-turns-zeds-agent-off-001,
  AD-claude-555-a-block-goes-to-an-agent-by-reference-or-inline-redacted-001,
  L-claude-555-the-rails-rows-move-when-needs-you-shows-001.
- **Brain:** consultation cffd3e517f5b4d9390680db51000046c closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-555 moved to `tickets/closed/`; its BACKLOG row went at promotion.
