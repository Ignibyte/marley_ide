---
pipeline_id: db397249-2026-4ec3-9b2c-3c0dd533043b
ticket: docs/planning/tickets/open/TICKET-680-tool-results-fit-the-agents-limit.md
status: Phase 4 — Complete PASS
title: Tool results that fit the agent's limit
type: feature
slice: prong 2 C (Marley's MCP server); phase 1 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/t3code_architecture/06-orchestration-mcp-and-automations.md
  - docs/planning/intake/marley-agent-manager-foreman.md
---

## Title
Block output an agent reads in pages that fit Claude Code's MCP limit, newest first; a short
`instructions` text in the `initialize` result; refusals with a code and next steps.

## Scope
### In
- `terminal_read` pages a block's output: without `before`, the newest whole lines that fit in
  12,000 bytes; with `before`, the newest that fit before that line. The answer names
  `first_line`, `last_line`, `total_lines`, `previous` (the `before` of the page ahead, null at the
  block's first line) and `line_cut` (a single line longer than a page, its end kept).
- `terminal_run`'s answer pages its output the same way.
- The output is redacted whole before any page is cut, and `redacted` counts the whole output.
- `instructions` in the `initialize` result: under 2,048 bytes, naming only Marley's own tools.
- Refusals carry `{result, code, reason, next_steps}`. The terminal family's own codes:
  `no_terminal` (with the ids there are), `no_block` (with the range there is), `output_gone`,
  `bad_argument`. The server's: `not_granted`, `tool_off`, `not_permitted`, `timed_out`,
  `unavailable`. Every other refusal carries `refused` until its family gets codes.
- The tool descriptions and output schemas of `terminal_read` and `terminal_run` say how paging
  works.

### Out (explicitly deferred)
- Paging `browser_snapshot` (30,000 characters today) and `browser_console`: their own ticket
  once the terminal family's paging has been used.
- Codes for the browser and editor families' refusals beyond `refused`.
- A `request_id` that replays a write tool's first outcome (T3 survey 06, item 2's last part).
- The Marley agent's tools and entry (TICKET-681 to TICKET-684).

## Reference (§20)
N/A — Marley-specific, no reference analog. The behavior is Marley's MCP server's contract with
agents. The sizes follow Claude Code's documented MCP output limits (a warning above 10,000
tokens, results cut or saved to a file above `MAX_MCP_OUTPUT_TOKENS`, 25,000 by default), and the
paging follows T3 Code's (`t3_thread_read`'s `textOffset`/`nextTextOffset`, read from source by
`docs/t3code_architecture/06-orchestration-mcp-and-automations.md` §2.4).

### Prior art
- **Behavior maps:** `docs/t3code_architecture/06-orchestration-mcp-and-automations.md` §2.3
  (instructions with the tools, `T3OrchestrationInstructions.ts`), §2.4 (results near 20 KB,
  `MAX_SNAPSHOT_TEXT_BYTES = 20_000`, paged reads, typed refusal codes such as
  `capability_denied`), §3 items 1 and 2 (the seams named here). Orca's report 06 item 7 proposed
  the codes and next steps; never built.
- **Published material:** the MCP specification 2025-06-18 (Marley's `MCP_PROTOCOL_VERSION`):
  `InitializeResult.instructions` is an optional string the client may add to the model's context;
  a tool result's `structuredContent` comes with its serialized form in a text block "for
  backwards compatibility", which is why Marley's answers carry the output twice. Claude Code
  2.1.293 (installed): above the limit it saves the result to a file and tells the model to read
  it in chunks (strings in its binary, "exceeds maximum allowed tokens. Output has been saved
  to"); it adds servers' `instructions` to the system prompt under "MCP Server Instructions".
- **The code we ship:** `crates/marley_mcp/src/tools.rs` (`tool_result`, `tool_answer_result`,
  `tool_error`), `dispatch.rs` (`initialize_result`, the refusal sites), `marley_mcp.rs`
  (`AppCall::answer`, `AppOutcome`), `crates/marley_workbench/src/mcp.rs` (`terminal_read`,
  `tail`, `terminal_of`), `terminal_drive.rs` (`run_answer`), the bridge
  (`claude_plugin/marley/bin/marley-mcp-bridge`, which returns Marley's own `initialize` result to
  the client, so `instructions` pass through). Zed's `context_server` crate is the client side and
  owns nothing here. No crate we build pages text for a reader; `tail` is ours. None owns the seam.

## UI proof
`script/e2e/680-tool-results-fit.sh`, on the default hidden workspace (no clicks). Fixtures: a
scratch repository; the scenario's own bash; the stand-in agent from `browser-fixture.sh`
(`mcp_agent`, over the plugin's bridge, as Claude Code in a Marley terminal reaches the server),
taught to page a block, read every page, print `initialize`'s instructions and print a refusal's
code and next steps. Steps and shots: in the terminal, `printf 'AKIA…\n'; seq 1 8000` (about
39 KB, four pages, a fake key on the first line, inside the 10,000 lines of scrollback a terminal
keeps) and a 40,000-character single line; the terminal with
both blocks (`680-01-blocks`); the stand-in's answers saved to the run's work folder and checked by
the scenario's `expect` lines (the newest page, the walk to the first page, the long line, the
refusals, the instructions); a final shot after the reads, Marley still drawing (`680-02-after`).

## Locked-In Decisions
- D1 — Pages run newest first, addressed by line numbers counted from the block's first output
  line (`before`), not by an offset from the end: a running block grows at the end, and numbers
  from the start keep every earlier page where it was.
- D2 — A page holds at most 12,000 bytes of output. The answer carries the output twice (the text
  block and `structuredContent.output`), and the JSON escapes each newline, so a whole answer
  stays near 30 KB on the wire and about 6,000 tokens of text, under Claude Code's 10,000-token
  warning; the duplication stays because clients differ in which channel they read.
- D3 — Redaction runs on the whole output before the page is cut (PR-claude-redact-the-whole-text-
  before-cutting-it-001), and `redacted` counts the whole output.
- D4 — `terminal_find` keeps its own window (`tail`, then `MAX_FIND_BYTES`); only `terminal_read`
  and `terminal_run` page.
- D5 — A refusal is a `marley_mcp::Refusal { code, reason, next_steps }`. `From<String>` gives
  `refused`, so the families not converted here keep compiling and answer `refused`; the terminal
  family returns its own codes.
- D6 — The instructions are one static text in `marley_mcp`: rules beside the tool they name, what
  a tool is not for, "when listed" for tools that come and go (`terminal_find`).
- D7 — A client that reads only the text block still learns how to page: when a page leaves part
  of the output out, the text starts with one line, `[lines A–B of N; read earlier lines with
  terminal_read before=A]`, and the structured `output` stays the plain page.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `terminal_read` without `before` on a block whose output is over 12,000 bytes, the system shall answer the newest whole lines that fit in 12,000 bytes, with `first_line`, `last_line`, `total_lines` and `previous`. | The stand-in's newest page: `last_line` = `total_lines`, output ≤ 12,000 bytes, ends with `8000` |
| REQ-002 | WHEN an agent passes `before` equal to the last answer's `previous`, the system shall answer the page that ends at the line before it, so that consecutive pages repeat and skip no line. | The stand-in reads every page; the scenario checks the joined pages equal the block's lines in order |
| REQ-003 | WHEN a page starts at the block's first line, the system shall answer `previous` as null. | The walk ends on a page with `first_line` 1 and `previous` null |
| REQ-004 | WHEN a single line is longer than a page, the system shall answer that line's end within 12,000 bytes and `line_cut` true. | The stand-in reads the 40,000-character block: one line, ≤ 12,000 bytes, `line_cut` true |
| REQ-005 | The system shall keep a whole `terminal_read` result, as the client receives it, under 40,000 bytes for any block. | The stand-in prints each result's byte size; the largest is under 40,000 |
| REQ-006 | WHEN the output holds a secret on a page other than the newest, the system shall answer that page with the secret redacted and `redacted` counting it on every page. | The first page shows `[redacted: …]` in place of the fake key; every page's `redacted` is 1 |
| REQ-007 | WHEN `terminal_run` answers, the system shall page its output as `terminal_read` does, newest page first, with `previous`. | The stand-in's `terminal-run` of `seq 1 8000`: output ≤ 12,000 bytes, `previous` set |
| REQ-008 | WHEN a client sends `initialize`, the system shall answer `instructions` of at most 2,048 bytes in which every tool named is one of Marley's tools. | The stand-in prints the instructions and their length; the scenario checks each `terminal_*`/`browser_*` name against `tools/list` plus `terminal_find` |
| REQ-009 | WHEN `terminal_read` names a terminal that does not exist, the system shall refuse with code `no_terminal` and next steps naming `terminal_list` and the terminals there are. | The stand-in's refusal print |
| REQ-010 | WHEN `terminal_read` names a block that does not exist, the system shall refuse with code `no_block` and next steps naming `terminal_blocks` and the range there is. | The stand-in's refusal print |
| REQ-011 | WHEN `before` is below 2 or above `total_lines` + 1, the system shall refuse with code `bad_argument`. | The stand-in's refusal print for `before` 0 |
| REQ-012 | WHEN the server refuses a call for its grant, a tool turned off, a permission, a timeout or Marley not answering, the system shall carry codes `not_granted`, `tool_off`, `not_permitted`, `timed_out` or `unavailable`; any other refusal shall carry `refused`. | The review of the diff (`dispatch.rs`'s refusal sites); the scenario's `browser_pick` of an unknown id answers `refused` |
| REQ-013 | WHEN a page leaves part of the output out, the system shall start the result's text block with one line naming the lines shown, the total and the `before` for the rest, and keep it out of `structuredContent.output`. | The stand-in prints the text block's first line and the structured output's first line for the newest `seq` page |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_mcp` (the refusal type, the instructions, the result envelope),
  `marley_workbench` (the pager, `terminal_read`, `run_answer`, the terminal family's refusals),
  the stand-in's new commands; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/680-tool-results-fit.sh`, read every
  shot and the saved answers.
- **P4 Complete** — CHANGELOG, `docs/marley/guide.md`'s tool table, the architecture doc (§21),
  ledger capture (§19), close the ticket, archive, commit.
