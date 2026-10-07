# Tool results that fit the agent's limit — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-680-tool-results-fit-the-agents-limit.md
- **Pipeline spec:** 680-tool-results-fit.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-07, "ok lets start on 1": phase 1 of
  `docs/planning/intake/marley-agent-manager-foreman.md`, item 2 first (T3 survey 06, §3 items 1
  and 2), then items 3 to 6 queued as TICKET-681 to TICKET-684.
- **Classification / tier:** feature, prong 2 C (Marley's MCP server). Marley crates only; no Zed
  hunk, so no ledger row.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (cargo idle, gate and e2e OK, hooks
  wired, no active pipeline, README marker present; `/mnt/fast` at 94%) · recall ✓ · promote/mint ✓
  · prior-art sweep ✓ · spec ✓ · design ✓ · queue 681 to 684 ✓ · present ✓.
- **Recall (§18.3):**
  - `F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001` and its rule
    PR-claude-redact-the-whole-text-before-cutting-it-001: redact the whole output, then cut. A page
    is a cut, so the rule binds every page (D3).
  - `F-claude-546-…`: block reads come from the main screen (`Term::main_grid`); paging reads the
    same `block_output`, so nothing changes there.
  - #556 (`terminal_run`) shares `tail` with `terminal_read`; #567 (`terminal_find`) uses `tail`
    then `MAX_FIND_BYTES` for its own window (D4 keeps it).
  - The Rusty brain (`rusty-cli brain ask`, consultation 97ad3bd8c4534c6c9dd48a2761f99a08): nothing
    on this seam; it listed unrelated follow-ups.
- **Discovery:** `crates/marley_workbench/src/mcp.rs:51-54` (`MAX_READ_LINES`, `MAX_READ_BYTES`),
  `:939-973` (`terminal_read`), `:977-992` (`tail`), `:801` (`terminal_of`, called by six terminal
  tools in `mcp.rs` and `terminal_drive.rs`); `terminal_drive.rs:996` (`run_answer`);
  `crates/marley_mcp/src/tools.rs` (`tool_answer_result` puts the output in the text block and in
  `structuredContent`; `tool_error` answers `{"result":"refused","reason"}`); `dispatch.rs:85-92`
  (an app answer's refusal, timeout and unavailability), `:171-185` (grant, off, permission
  refusals), `:107-113` (`initialize_result`, no `instructions`); `marley_mcp.rs:171-250`
  (`AppCall`, whose answer channel carries `Result<ToolAnswer, String>`); `registry.rs:126-133`
  (`terminal_read`'s description), `:1425-1455` (its schemas). The bridge returns Marley's own
  `initialize` result (`marley-mcp-bridge:317-322`), so `instructions` reach the client.
  The default scrollback is 10,000 lines (`assets/settings/default.json:2393`); `block_output`
  answers `None` once a block's start has left it.
- **Decisions:** D1 to D6 in the spec.

### Design

**Approach.**

1. **The pager** (`crates/marley_workbench/src/mcp.rs`). A pure `page(text, before) ->
   Result<Page, Refusal>` beside `tail`: the output's lines numbered from 1; the page ends at
   `before - 1` (the last line without `before`); it takes whole lines from the end backwards while
   their bytes, newlines counted, fit in `PAGE_BYTES` (12,000) and `MAX_READ_LINES`; when the
   first line taken is already over a page, it keeps that line's last `PAGE_BYTES` bytes at a char
   boundary and sets `line_cut`. `Page { text, first_line, last_line, total_lines, previous,
   line_cut }`, `previous = (first_line > 1).then_some(first_line)`. `before` outside `2..=total
   + 1` is `bad_argument`. An empty output is one page with `total_lines` 0 and no lines.
2. **`terminal_read`** takes `before` (optional integer). It redacts the whole output, pages it,
   and answers `terminal, block, command, running, output, first_line, last_line, total_lines,
   previous, line_cut, truncated, redacted`. `truncated` stays, meaning that the answer leaves part
   of the output out (earlier lines or a cut line), so readers of the old field keep working.
3. **`terminal_run`** (`run_answer`, `terminal_drive.rs`): the same pager with no `before`, and
   the same fields.
4. **Refusals** (`crates/marley_mcp`). `pub struct Refusal { code: &'static str, reason: String,
   next_steps: Vec<String> }` with `Refusal::new(code, reason)`, `.next(step)`, and
   `From<String>` giving `refused`. `AppCall::answer` takes `Result<ToolAnswer, impl
   Into<Refusal>>`, so the call sites answering `Result<_, String>` stay as they are and answer
   `refused`. `AppOutcome::Answered` carries `Result<ToolAnswer, Refusal>`. `tools::tool_error`
   becomes `tool_refusal(&Refusal)`, answering `{result: "refused", code, reason, next_steps}`
   with `isError` true; the tool's name stays a prefix of `reason`, as today. `dispatch.rs`'s
   own sites get `not_granted`, `tool_off`, `not_permitted`, `timed_out`, `unavailable`; the
   surface tool's argument error gets `bad_argument`.
5. **The terminal family's codes** (`mcp.rs`, `terminal_drive.rs`). `terminal_of` and
   `terminal_with_id` return `Refusal`: an unknown id is `no_terminal`, with next steps
   "terminal_list lists them" and the ids there are (at most 20); no id from a call outside a
   terminal is `bad_argument`. A missing block is `no_block` with "terminal_blocks lists terminal
   N's blocks: 0 to M". A block whose output left the scrollback is `output_gone`, with "run the
   command again with terminal_run, or ask the user for the output". The terminal handlers return
   `Result<_, Refusal>`; their other `String` errors convert to `refused` through `?`.
6. **The instructions** (`crates/marley_mcp/src/marley_mcp.rs`, a `pub const INSTRUCTIONS`, read
   by `initialize_result`). Draft, to be held under 2,048 bytes:
   > Marley is the editor and terminal the user is working in; these tools read and drive what
   > the user sees. Before asking the user to paste a command's output, read it:
   > `terminal_blocks` lists a terminal's commands, and `terminal_read` reads one block's output,
   > newest lines first; pass its `previous` as `before` to read the page before. `terminal_find`,
   > when listed, finds a line by meaning. Use `terminal_run` for a command the user should see run
   > in their terminal, not for your own scratch work. Use `terminal_screen`, then `terminal_type`,
   > to answer a program waiting for input (a REPL, a debugger); never type into another agent.
   > For a web page, use Marley's Browser tab: `browser_tabs`, then `browser_snapshot` or
   > `browser_look`, before you drive it; do not start Playwright or another browser for a page the
   > user has open in Marley. A refusal carries a `code` and `next_steps`: follow them rather than
   > repeating the call.
7. **Descriptions and schemas** (`registry.rs`): `terminal_read`'s description says pages of at
   most 12,000 bytes, newest first, `previous` → `before`; its input schema adds `before`
   (integer, minimum 2) and its output schema the new fields; `terminal_run`'s description says
   its output is the newest page and `terminal_read` reads the rest.
8. **The stand-in agent** (`script/e2e/browser-fixture.sh`): `terminal-page <text> [--before
   <n>]` prints a page's fields, its output's byte count and the result's byte count as received;
   `terminal-pages <text> <file>` walks from the newest page to the first and saves the joined
   output; `refusal <tool> <json>` prints a refusal's code, reason and next steps;
   `mcp_http … initialize` and the bridge client print `instructions` and its length.

**File manifest.**

| File | Crate | Change |
|---|---|---|
| `crates/marley_mcp/src/marley_mcp.rs` | Marley | `Refusal`, `INSTRUCTIONS`, `AppCall::answer`, `AppOutcome` |
| `crates/marley_mcp/src/tools.rs` | Marley | `tool_refusal`; tests in the file stay building |
| `crates/marley_mcp/src/dispatch.rs` | Marley | `initialize_result`'s `instructions`; the refusal sites' codes |
| `crates/marley_mcp/src/transport.rs` | Marley | the answer channel's type, if it names it |
| `crates/marley_mcp/src/registry.rs` | Marley | `terminal_read` and `terminal_run`'s descriptions and schemas |
| `crates/marley_workbench/src/mcp.rs` | Marley | `page`, `Page`, `terminal_read`, `terminal_of`, the terminal handlers' refusals |
| `crates/marley_workbench/src/terminal_drive.rs` | Marley | `run_answer`, the handlers' error type |
| `crates/marley_workbench/src/browser_tools.rs` | Marley | only if a call site needs the `Into<Refusal>` |
| `script/e2e/browser-fixture.sh` | script | the stand-in's new commands |
| `script/e2e/680-tool-results-fit.sh` | script | the scenario |

No Zed crate changes; no `docs/marley/zed-touchpoints.md` row.

### Visual check plan

`script/e2e/680-tool-results-fit.sh`, hidden workspace 9, Marley's own terminal in the scratch
repository; the stand-in agent runs from the scenario's shell through the plugin's bridge.

| REQ | Setup and action | Evidence |
|---|---|---|
| 001 | `./pages.sh` (a fake AWS key put together at run time, then `seq 1 8000`) run with `terminal_run`; `terminal-page pages.sh` | `680-01-blocks` shows the block; the saved page: `last_line` 8001, output ≤ 12,000 bytes, ends `8000` |
| 002 | `terminal-pages seq pages.txt` | the scenario compares `pages.txt` with the fixture's own `printf …; seq 1 8000` after redaction: equal |
| 003 | the walk's last page | `first_line` 1, `previous` null |
| 004 | `python3 -c 'print("x" * 40000)'` typed; `terminal-page x\*` | one line, ≤ 12,000 bytes, `line_cut` true |
| 005 | every page's result size as received | the largest under 40,000 bytes |
| 006 | the first page of the `seq` block | `[redacted: …]` where the key was, `redacted` 1 on every page |
| 007 | `terminal-run <title> "seq 1 8000"` | output ≤ 12,000 bytes, `previous` set |
| 008 | `mcp_http <endpoint> initialize` and the bridge's `initialize` | `instructions` printed, length ≤ 2,048; each tool name in it is in `tools/list` or is `terminal_find` |
| 009 | `refusal terminal_read {"terminal": 999999, "block": 0}` | code `no_terminal`, next steps naming `terminal_list` and the terminal's id |
| 010 | `refusal terminal_read {"terminal": <id>, "block": 999}` | code `no_block`, next steps with the range |
| 011 | `refusal terminal_read {"terminal": <id>, "block": <seq>, "before": 0}` | code `bad_argument` |
| 012 | `refusal browser_pick {"id": "nope"}` | code `refused`; the server's own codes by the review of `dispatch.rs` |
| 013 | the newest `seq` page, both channels | the text block's first line `[lines … of 8001; read earlier lines with terminal_read before=…]`; the structured output starts with a number |

`680-02-after` is Marley after the reads, still drawing. No scenario can show whether Claude Code
puts `instructions` into its model's context; Claude Code 2.1.293 lists other servers'
instructions under "MCP Server Instructions" in its system prompt, and the first Claude Code
session in a Marley terminal after the install is where to look.

### Risks
- **The answer's size on the wire.** JSON escapes in the output (quotes, backslashes, control
  characters as `\u00XX`) can grow a page past the estimate; REQ-005 measures it on the scenario's
  blocks, and a page of escapes is bounded by six bytes per byte, still under Claude Code's cut.
- **A running block that scrolls its start out.** Line numbers count from the block's first line
  while `block_output` answers; once its start leaves the scrollback the read is `output_gone`, as
  today. Reported, not hidden.
- **`AppCall::answer` taking `impl Into<Refusal>`** touches every answer site's types; the
  `From<String>` keeps them source-compatible, and the gate's clippy run says where it does not.
- **Clients that read only `content`** (Zed's agent through `context_server`, perhaps Codex) would
  see the page's text and not `previous`. D7: when the page leaves part of the output out, the
  text block starts with one line naming the lines shown and the `before` for the rest; the
  structured `output` stays the plain page, as #491 wants a block's output to read as itself.

## Phase 2 — Code
- **Built:** `marley_mcp::Refusal { code, reason, next_steps }` with `From<String>` and `From<&str>`
  (code `refused`); `AppCall::answer` takes `Result<ToolAnswer, E: Into<Refusal>>`, and
  `AppOutcome::Answered` carries a `Refusal`. `tools::tool_refusal` replaces `tool_error` and
  answers `{result, code, reason, next_steps}`. `dispatch.rs`: `instructions` in `initialize`
  (1,060 bytes); the codes `not_granted`, `tool_off`, `not_permitted`, `timed_out`, `unavailable`
  and `bad_argument`, each with a next step; an app's refusal keeps its code and gets the tool's
  name in front of its reason, as before. `registry.rs`: `terminal_read` takes `before` and
  describes paging and its codes; both tools' output schemas carry the page's fields.
  `marley_workbench/src/mcp.rs`: `Page`, `newest_page`, `page`, `page_ending` (`PAGE_BYTES`
  12,000), `Page::fill` and `Page::text_block` (the first line naming the lines and the `before`
  when the page leaves output out); `terminal_read` pages the redacted output; `terminal_of`,
  `terminal_with_id` (listing up to 20 ids), `block_argument`, `no_block`, `output_gone` refuse
  with codes; `terminal_blocks`, `terminal_read` and `FindLines::of` return `Refusal`.
  `terminal_drive.rs`: `screen` returns `Refusal`; `run_answer` uses `newest_page`. The stand-in
  agent (`script/e2e/browser-fixture.sh`) gained `terminal-page`, `terminal-pages`,
  `read-refusal`, `refusal` and `instructions`, and records each answer's size as the bridge wrote
  it.
- **Deviations:**
  - `INSTRUCTIONS` is a private const in `dispatch.rs` beside `initialize_result`, its one reader,
    not a `pub const` in `marley_mcp.rs`.
  - `check` and `check_run` in `terminal_drive.rs` split into the terminal lookup, which returns
    the typed refusal, and `check_in`/`check_run_in`, whose word refusals stay `String` and answer
    `refused`. That keeps the diff to the lookup instead of every refusal in the two functions.
  - Four answer sites with a bare `Ok(...)` (`agent_editor.rs` three, `mcp.rs` one) name the error
    type, `answer::<Refusal>`, since a generic `answer` cannot infer it; `agent_editor.rs` joins
    the manifest.
  - `mcp_http … initialize` prints no `instructions`: the bridge's `initialize`, which Claude Code
    gets, is what the stand-in's `instructions` reads.
- **Review of the diff:** against REQ-001 to REQ-013. The redaction runs on the whole output
  before `page` (D3) in both `terminal_read` and `run_answer`; `redacted` counts the whole output
  on every page. `page_ending`'s bounds: a line exactly `PAGE_BYTES` long fits whole
  (`line_cut` false); an empty output answers one page with `total_lines` 0 and `previous` null;
  `before` 0, 1 and over `total_lines + 1` are `bad_argument`; a negative or non-integer `before`
  is `bad_argument` before any lookup. No entity is read during its own update (the reads are
  `view.read`). Found and changed: the `tool_off` next step read badly ("reads without it"); now
  "without it, terminal_read or browser_snapshot gives the whole text to search". No bug.
- **Gate:** `just gate-diff`, three runs. Run 1 red, gate:2: `tools.rs`'s doc comment's first
  paragraph too long; split. Run 2 red, gate:2: `Page::partial` could be a `const fn`; made one
  (`just clippy marley_mcp marley_workbench` then clean). Run 3 **green**, 17 of 17, receipt
  written (`scratchpad/680-gate-3.log`). No pre-existing failures.

## Phase 3 — Test
- **Scenario:** `script/e2e/680-tool-results-fit.sh` (`compositor sway`, for the click into the
  terminal). Setup pins `marley.agent_commands_outside_lists` to `"run"` in the run's profile copy
  and writes two executables into the scratch repository: `pages.sh` (a fake AWS key put together
  at run time, then `seq 1 8000`) and `long.sh` (a 40,000-character line). The stand-in agent
  reaches the server through the Claude Code plugin's bridge.
- **Runs:** three, on the debug build (`just build`, 34 s).
  - Run 1 red at "it ran": `terminal_run "bash pages.sh"` waited on the card, since `bash …` is on
    the default denylist, and the call was refused after 25 seconds.
  - Run 2 red the same way after pinning the outside-lists setting; the denylist, not that
    setting, was asking.
  - Run 3 green, 27 of 27 checks, after the scripts became executables run as `./pages.sh` and
    `./long.sh` (on neither list). These were the scenario's own mistakes, not the change's.
- **What the stand-in read** (run 3, `scratchpad/680-e2e-3.log`):
  - REQ-008: `instructions` 979 bytes, naming browser_look, browser_snapshot, browser_tabs,
    terminal_blocks, terminal_find, terminal_read, terminal_run, terminal_screen, terminal_type;
    "named but not listed: none".
  - REQ-007: `terminal_run ./pages.sh` answered exit 0, redacted 1, page 6002-8001 of 8001,
    previous 6002, output 9,999 bytes.
  - REQ-001, REQ-013: the newest page is lines 6002 to 8001 of 8001 (`previous` 6002), output 9,999
    bytes ending `8000`, result 24,389 bytes as the bridge wrote it. The text block starts
    `[lines 6002 to 8001 of 8001; read earlier lines with terminal_read before=6002]`; the
    structured output starts `6001`, the plain page.
  - REQ-002, REQ-003, REQ-005, REQ-006: five pages, 6002-8001, 4002-6001, 2002-4001, 2-2001 and
    1-1. The last has `previous None`. Every page says `redacted 1`; the largest answer is 24,389
    bytes and the largest output 9,999. Joined oldest first they are 8,001 lines: `aws: [redacted:
    aws key id]`, then 1 to 8000 in order. The 2,000-line cap ends each page before the byte cap
    on lines this short, so the first line gets a page of its own.
  - REQ-004: `long.sh` reads as page 1-1 of 1, `line_cut True`, `truncated True`, output 12,000
    bytes, result 24,348; the text block starts `[line 1 of 1, the start of the line left out]`.
  - REQ-009: `no_terminal`, next steps "terminal_list lists the terminals, with their titles and
    projects" and "the terminals now: 12884902152".
  - REQ-010: `no_block`, next step "terminal_blocks lists terminal 12884902152's blocks: 0 to 1".
  - REQ-011: `before` 0 is `bad_argument`, "`before` 0 is outside the output: it has 8001 lines, so
    `before` is 2 to 8002", next step "pass a page's `previous` as `before`, or leave it out for
    the newest page".
  - REQ-012: `terminal_run` with a blank command is `refused`, "give `command`, the line to run".
    The server's own codes (`not_granted`, `tool_off`, `not_permitted`, `timed_out`,
    `unavailable`) rest on the review of `dispatch.rs`; no scenario here makes Marley time out or
    turns a granted tool off.
- **Shots** (`scratchpad/shots-680/`, both Marley only):
  - `680-01-blocks`: the project's `repo — bash` terminal after the two runs, the 40,000-character
    line wrapped across the rows under its block bar and the prompt after it; the rail's terminal
    row reads "done · 0 s". The `seq` block is above, scrolled out by the long line.
  - `680-02-after`: the same view after every read and refusal; Marley still draws, and the
    refused blank `terminal_run` typed nothing at the empty `$` prompt.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it"; the run's sway, Marley, pointer and keyboard stopped with it.
- **Not reached by a scenario:** whether Claude Code puts `instructions` into its model's context.
  Claude Code 2.1.293 lists servers' instructions under "MCP Server Instructions" in its system
  prompt; the first Claude Code session in a Marley terminal after the install is where to look.
- **Gate after the scenario:** the scenario and the stand-in changed after run 3's green, so the
  gate ran again for the commit's receipt: green, 17 of 17 (`scratchpad/680-gate-4.log`).

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #680). `docs/marley/guide.md`: `terminal_read`'s row in
  the tool table, and a new "What agents are told, and how a call is refused" section before
  Limits (the instructions, the codes, the page size and its first line); Limits' timeout now
  says `timed_out`. `crates/marley_workbench/guide/index.html`: `terminal_read`'s row.
  `docs/marley_architecture/marley_mcp.md`: `instructions`, the `Refusal` type and the server's
  codes, `terminal_run`'s page. `docs/marley_architecture/marley_workbench.md`: the pager, the
  terminal tools' codes, redaction before a page is cut. The plan doc
  (`docs/planning/intake/marley-agent-manager-foreman.md`) marks phase 1 item 2 done and names
  TICKET-685 for the ACP proof. No Zed crate touched, so no `zed-touchpoints.md` row.
- **Knowledge (§19):** `L-claude-680-a-scenario-s-terminal-run-runs-a-program-not-a-shell-001`
  (lessons), `AD-claude-680-mcp-results-page-newest-first-and-refusals-carry-a-code-001`
  (decisions). No `F-` block: the review found no bug in the change; the scenario's two red runs
  were its own commands on the denylist, which the lesson records. Brain: consultation
  97ad3bd8c4534c6c9dd48a2761f99a08 closed with `rusty-cli brain decide`
  (`decisions/marleys-mcp-results-page-newest-first-and-refusals-carry-a-code`, follow-up by
  2026-10-21).
- **Ticket:** TICKET-680 closed (`tickets/closed/`), its pipeline link at `completed/`; it had no
  `BACKLOG.md` row since promotion. TICKET-681 to TICKET-685 stay queued.
- **Gate:** the in-app guide is in the receipt's fingerprint, so the gate ran again before the
  commit.
