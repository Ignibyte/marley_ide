---
pipeline_id: 6741052b-b1bc-413c-b8e6-0da16dc1bf61
ticket: docs/planning/tickets/closed/TICKET-555-send-a-block-to-the-agent.md
status: Phase 4 — Complete PASS
title: "Send a block to the agent, and Ask the agent under a failed block"
type: feature
slice: prong 2 with prong 1 (a block into a CLI agent's prompt); the Warp blocks note's recommendation 2; after #549 and #554
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/queued/549-selection-to-the-agent.spec.md, docs/planning/pipeline/queued/554-block-selection-and-menu.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/planning/pipeline/completed/508-approvals-inbox.spec.md]
---

## Title
Send to Agent, in the block menu and on a selected block (`ctrl-shift-enter`), types a
reference into a CLI agent's terminal the way a pick's Send does,
`[terminal 42 block 7: cargo build, exit 101; terminal_read terminal=42 block=7] `, unsent, and
focuses that terminal; a block whose output is short goes inline as its Markdown form instead,
so it works where the plugin's bridge is not installed. The target is #549's (one agent, the
picker for several, never the block's own terminal, nothing while the agent waits), and what
leaves Marley passes #516's redactor. Under the newest failed block, while the shell waits at
its prompt and an agent terminal exists, an `Ask the agent` chip does the same in one click,
Marley's form of Warp's hint after a failed command.

## Scope
### In
- `crates/marley_workbench/src/send_block.rs` (new): the reference and inline forms, the
  choice between them, the redaction, the send through #549's target rule and `send`, the
  action `marley::SendBlockToAgent` (a block argument from the menu, else the selected block),
  and the chip's renderer.
- **The menu and the key.** `Send to Agent` in #554's Block section; `ctrl-shift-enter` in
  `Terminal && MarleyBlockSelected`, free in Zed's `Terminal` context.
- **The text.** The reference line when the output is long:
  `[terminal <id> block <n>: <command>, exit <code>; terminal_read terminal=<id> block=<n>] `
  (`running` in place of `exit <code>` for a running block), naming the tool and its parameters
  as `terminal_read`'s schema names them, the id being the one `terminal_list` gives. The
  Markdown form (#554's `AnchoredBlock::markdown`) when the output is at most 32 lines and 4 KiB, or has
  left the scrollback. Both pass the agent redactor (`mcp::agent_redactor`) when
  `marley.redact_secrets_for_agents` is on: the command inside the reference and the whole
  Markdown, before any cut.
- **The chip.** A `MarleyBlockChip` hook in `terminal_element.rs`'s block element (the footer's
  pattern): the workbench returns `Ask the agent` for the newest block when it is Finished with
  a non-zero exit, the shell waits at its prompt, the terminal's foreground is not an agent, and
  at least one agent terminal exists in the window; drawn beside the pill, visible without
  hover; a click sends the block as Send to Agent does.
- `script/e2e/555-send-a-block-to-the-agent.sh`.

### Out (explicitly deferred)
- Zed's Agent Panel thread as a target (`AddSelectionToThread` needs a text selection): a
  "Send to Zed's agent" item later.
- Stable terminal ids across relaunches: #520's UUID in the reference once the tools take it.
- Warp's Prompt Suggestions and attaching the newest block on its own.
- An "Ask the agent" for an English line at the prompt (exit 127; the note's natural-language
  part).
- A chip on older failed blocks: the newest only, at the prompt.

## Reference (§20)
- **Warp:** Attach as context: the sparkles icon on a block, and `Ctrl+↑` in the agent's input
  adds the selected blocks (docs.warp.dev/agents/local-agents/agent-context/blocks-as-context/);
  after a failed command the hint line offers "attach '<command>' output as agent context"
  (the same page). Marley types a reference or the block's Markdown into the CLI agent's terminal
  instead of an in-app agent, and draws the hint as a chip on the failed block.
- **Upstream Zed:** `AddSelectionToThread` for a terminal's text selection
  (`crates/agent_ui/src/agent_panel.rs:644`; the lines joined at `:798`) stays for the panel,
  and the terminal menu's item (`crates/terminal_view/src/terminal_view.rs:590`) stays beside
  the new one.
- **Orca:** none; agents read Orca's terminals through its CLI, and nothing sends a block.

### Prior art
- **Behavior maps and research.** The blocks note's Send to Agent paragraph (`:92` to `:104`:
  the reference form, short output inline, never the block's own terminal, #516's redaction,
  the refusal while waiting) and its "Ask the agent" chip (`:99`); #508's D6
  (`508-approvals-inbox.spec.md:122`); #516's D2 (`516-secret-redaction-for-agents.spec.md:81`:
  redact at the tool boundary, never in the buffer) and
  F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001 (redact the whole text before a
  cut); #520 (`520-terminal-identity.spec.md:64`, `:133`: ids stay the entity id until the tools
  take the UUID); #530 (`530-runnable-markdown-commands.spec.md:22`) plans the same
  `LastTerminal` and paste route for Markdown commands, a shared helper when both land.
  `docs/warp_architecture/` has no page on blocks as context; its subsystem 03 §4 describes the
  agent's write path into a session.
- **Published material.** Warp's blocks-as-context page; MCP's tool schemas (the reference
  names the tool's parameters so an agent maps it to a call).
- **The code we already ship.** The pick's send (`browser.rs:3645`, `pick_line` `:5658`, the
  toast `:3779`); #549's target rule, picker, `send` and `rich_input::insert`; #554's
  `block_markdown`, selection global and menu hook. `mcp.rs`: `agent_redactor` (`:314`),
  `for_agents` (`:323`), `terminal_read` (`:508`: the command and the whole output redacted at
  `:525` and `:528` before `tail` at `:547`), `terminal_with_id` (`:381`), `terminal_list`
  (`:397`: the id is the view's entity id), `MAX_READ_LINES` (`:49`) and `MAX_READ_BYTES`
  (`:52`); `marley_mcp::redact::Redactor` (`crates/marley_mcp/src/redact.rs:126`, `redact`
  `:152`); the registry's `terminal_read` row (`crates/marley_mcp/src/registry.rs:113`) and
  schemas (`:916`: `terminal` and `block`, both required; `terminal_argument_schema` `:820`).
  `terminal_element.rs`: `marley_block` (`:2320`), `marley_pill` (`:2262`: `exit <code>` in the
  error color), `marley_wash` (`:2237`), the prompt gate at `:1632`; the agent bar's chip
  (`agent_bar.rs:305`) as the chip's look; `AgentEvents::seat` (`agent_events.rs:30`). Does a
  crate we build own this seam? The MCP tools own the content and the redactor, #549 owns the
  send; the chip's hook is one hunk in the block element.

## UI proof
UI-AFFECTING: the reference at the agent's prompt, the chip, the picker, a toast.
`script/e2e/555-send-a-block-to-the-agent.sh` (`compositor sway`: it clicks the chip and the
menu). Setup: a scratch repository; a HOME whose `.bashrc` is the scenario's; #549's stand-ins
`claude` and `codex` (each line read printed as `got: <line>`; `wait` makes the claude one emit
a PermissionRequest through `event.py`). Steps: `claude` in terminal 1; `ctrl-~`, terminal 2;
`seq 1 200; false`, Return: the chip beside the block's pill (`555-01-chip`); click it:
terminal 1 focused with `[terminal <id> block <n>: seq 1 200; false, exit 1; terminal_read
terminal=<id> block=<n>] ` at the input (`555-02-reference-sent`); Return there; `mcp_agent
terminals` and `mcp_agent blocks` hold the id and the index named. `ctrl-tab`; `false`,
Return; `ctrl-up`, `ctrl-shift-enter`: the Markdown form at the stand-in's input, line by line
(`555-03-inline-sent`). `ctrl-tab`; `echo token=ghp_` followed by 36 letters and digits,
Return (`555-04-before-send` shows it exact); right-click the block, Send to Agent: the
agent's terminal shows `[redacted: github token]` (`555-05-redacted`). `ctrl-tab`; `wait`,
Return to the claude stand-in; `ctrl-tab`; right-click the last block, Send to Agent: nothing
typed, the toast (`555-06-refused`). `ctrl-~`, `codex`; `ctrl-tab` to terminal 2; right-click,
Send to Agent: the picker with two rows and not terminal 2 (`555-07-picker`); Escape. Both
stand-ins ended; in terminal 2 `false`, Return: no chip (`555-08-no-agent-no-chip`).

## Locked-In Decisions
- D1: The reference names the tool call the agent should make, `terminal_read terminal=<id>
  block=<n>`, with the parameter names of the tool's schema, so any agent with Marley's server
  finds the block; the id is the view's entity id `terminal_list` gives today, the UUID once
  #520's tools take it.
- D2: Short output goes inline as the Markdown form (#554's): at most 32 lines and 4 KiB after
  redaction, or output that has left the scrollback (the reference would answer "not kept"); the
  rest goes by reference, so a long build log never floods the prompt and the agent reads it
  under the caps `terminal_read` keeps.
- D3: What leaves Marley for an agent is redacted, #516's rule at the tool boundary applied to
  a paste: the reference's command text and the whole Markdown, before any cut. #554's clipboard
  copies are not.
- D4: The target is #549's: one agent terminal, the picker for several, a toast for none; never
  the block's own terminal, since a block in an agent's terminal is that agent's session. The
  seat refusal holds.
- D5: The chip is the hint in Marley's place: on the newest block only, while it is Finished
  with a non-zero exit and the shell waits at its prompt, beside the pill, and only while an
  agent target exists (a chip that leads to a toast is noise). Warp's hint sits under the block;
  stage one inserts no rows, so the pill's row carries it until T5 moves it under.
- D6: The chip is drawn through a hook (`MarleyBlockChip`), the footer's pattern
  (AD-claude-477: what Marley draws in the terminal goes through one hook), so the visibility
  rule and the click live in `marley_workbench`; one hunk in the block element.
- D7: `ctrl-shift-enter` on a selected block, Warp's key for a conversation with the selected
  blocks attached; Zed leaves it free in `Terminal`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user picks Send to Agent on a block whose output is long, the system shall type the reference line at the target agent's prompt without Enter and focus that terminal. | Shot `555-02-reference-sent`; `mcp_agent terminals` and `blocks` hold the id and the index named |
| REQ-002 | WHEN the block's output is short or no longer in the scrollback, the system shall type the block's Markdown form instead. | Shot `555-03-inline-sent` |
| REQ-003 | WHEN the sent text holds a secret the redactor knows, the agent's terminal shall receive it redacted while the block's own terminal shows it exact. | Shots `555-04-before-send`, `555-05-redacted` |
| REQ-004 | WHILE the newest block is finished with a non-zero exit, the shell waits at its prompt and an agent terminal exists in the window, the system shall show an `Ask the agent` chip beside its pill; WHEN clicked, the chip shall send the block as Send to Agent does. | Shots `555-01-chip`, `555-02-reference-sent` |
| REQ-005 | WHEN no agent terminal exists in the window, the system shall show no chip. | Shot `555-08-no-agent-no-chip` |
| REQ-006 | WHILE the target's seat waits on a permission or a question, the system shall type nothing and show a toast. | Shot `555-06-refused` |
| REQ-007 | WHEN several agent terminals exist, the system shall open the picker, and it shall never list the block's own terminal. | Shot `555-07-picker` |
| REQ-008 | WHEN the user presses `ctrl-shift-enter` with a block selected, the system shall send it as the menu item does. | Shot `555-03-inline-sent` |
| REQ-009 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. Promote after #549 and
  #554 land, and re-verify their names (`agent_targets`, `send`, `block_markdown`, the menu
  hook) against what shipped.
- **P2 Code:** the ledger row first (`crates/terminal_view/src/terminal_element.rs`, the chip
  hook); `send_block.rs`; the menu item and the keymap line; fmt and clippy clean; a review of
  the diff (redaction before the size check; the block's own terminal never a target; the chip's
  four conditions).
- **P3 Test:** write and run the scenario and read every shot (the visual check of the change
  only, 2026-09-29; no golden run).
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md` and
  `terminal_blocks.md`; the touchpoints row checked; the plan's slice status; the ledger
  capture; close the ticket, archive, commit.
