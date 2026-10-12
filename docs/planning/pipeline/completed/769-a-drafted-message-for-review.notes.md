# A drafted message for the person to review — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-769-a-drafted-message-for-review.md
- **Pipeline spec:** 769-a-drafted-message-for-review.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** split from #716 on 2026-10-11 (Chad, "spec all"); on the after-the-finish shelf,
  taken under the goal "continue until all tickets are finished" once the finish-line queue was
  done. Its spec was minted here from #716's Out line.
- **Classification:** feature, S to M; Marley crates only (`marley_mcp`, `marley_workbench`).
- **Recall (§18.3):** #706 (`thread_post` reaches only the Agent Panel's threads); #555 and #549
  (`send_selection::send_text` delivers to an agent CLI as the person would); #522 (Review notes
  refuse a target that is not ready); #692 and #682 (proposals wait 25 seconds for Apply); #703
  (the activity gate and kill switch); #766 (annotations from the registry). Brain: consultation
  eb10b170204e476b83697f8907b0b67e, nothing on this seam.
- **Discovery** (an Explore pass at `3f3f96430b`):
  - The registry (`registry.rs`): `Family` (15-45), `as_str`, `is_served`, `ToolSpec`, `REGISTRY`,
    the exhaustive `tool_schemas` (875-903) and `dispatch::tools_call` (243-261); the registry test
    lists every name and the count, 66.
  - The grants (`mcp.rs:106-116`), the router (`mcp.rs:525-608`), the activity gate for every
    Write tool (`agent_activity.rs:147-161`). An app call has 30 seconds.
  - Terminal targets: `terminal_with_id` (`mcp.rs:878-899`); `send_selection::target_of` (331-347,
    private) and `send_text` (387-434), which refuses a waiting agent with a toast and pastes with
    Enter (`terminal_drive::paste_then`).
  - Thread targets: `thread_tab::conversations_of` covers panel and tab threads; posting as
    `thread_tools::post` (265-319): refuse while the person has unsent text, else insert and
    `ThreadView::send`, in the thread's own window (`browser::window_of`).
  - The caller's name: `click_pause::Who::of(call, cx).words`. The tab template:
    `rusty/skills_tab.rs` (a multi-line `Editor`, Filled buttons, `impl Item`).
  - INSTRUCTIONS is 1,504 bytes of its 2 KB.

### Design
- **`marley_mcp/src/registry.rs`:** `Family::Draft` ("draft", served); `draft_message`, Write,
  `draft.write`; `draft_schemas`; the registry test's list and count (67).
- **`marley_mcp/src/dispatch.rs`:** `Family::Draft` among the deferred families; INSTRUCTIONS names
  the tool beside "never type into another agent".
- **`marley_workbench/src/mcp.rs`:** the `draft.write` grant; the route to `drafts::answer`.
- **`marley_workbench/src/drafts.rs` (new):** `answer` (arguments, the target, the tab, the answer
  `{draft, to, opened}`); `DraftTarget::{Terminal, Thread}`; `DraftView` (an `Item` with the
  sender, the target, an editor, Send and Drop); `send` per target; a toast where it went.
- **`marley_workbench/src/send_selection.rs`:** `target_of` crate-visible.
- **Docs:** the guide's tool table and a "Drafts" passage; CHANGELOG; the architecture note.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001, 004 | `draft_message` to the stand-in Claude Code's terminal; refusals for a bad id and a shell | 769-01; the answer `opened`; the stand-in's log empty; the codes |
| 002 | Types more into the draft, Send | 769-02; the stand-in logged the edited text |
| 003 | A draft to a scripted ACP thread, Drop | 769-03; the agent's log has no prompt |
| 002 | A second draft to the thread, Send | 769-04; the thread shows it, the agent answers |
| 005 | — | The review (the activity gate covers every Write tool) |

### Risks
- A draft whose target closes before Send: Send says the target is gone and keeps the tab.
- Several drafts at once: each its own tab.

## Phase 2 — Code

- **Built:** `Family::Draft`, `draft_message` and `draft_schemas` (`registry.rs`); the deferred
  arm and the INSTRUCTIONS line (`dispatch.rs`); the `draft.write` grant and the route
  (`mcp.rs`, with `terminal_with_id` crate-visible); `send_selection::target_of` crate-visible;
  `drafts.rs` (the answer, `DraftTarget`, `DraftView`, Send per target, `window_holding`).
- **Deviations:**
  - The thread target carries its workspace, and Send takes the draft's own window handle (the
    review's fix below), instead of finding the window through `all_workspaces`.
  - The editor hides its gutter and soft-wraps: the first run's shot showed a line number "1"
    beside the message (L-claude-769-a-multi-line-editor-shows-a-code-gutter-001).
  - A terminal whose agent waits keeps the draft with a note, rather than `send_text`'s toast, so
    the text is not lost.
- **The review found:** Send would have looked for the target's window with `cx.windows()` inside
  the draft window's own update, and found it gone (PR-claude-596). Fixed before the first run:
  `window_holding` falls back to the window in hand (F-claude-769-send-looked-for-the-draft-tabs-own-window-and-found-it-gone-001).
- **Clippy:** `use_self` and `semicolon_if_nothing_returned`, both fixed at the source.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the tree the scenario ran.

## Phase 3 — Test

- **Scenario:** `script/e2e/769-a-drafted-message-for-review.sh` under `compositor sway`: a
  stand-in `claude` first on the terminal's PATH, a scripted ACP agent `Scripted` in an Agent Panel
  thread opened by `agent::NewExternalAgentThread`, and `mcp_agent` as `planner-agent`. The first
  run stopped after 769-01 to place the buttons (Send 296,942; Drop 343,942). Every check passed on
  the second run.
- **769-01-draft (REQ-001):** a "Draft to Claude Code in repo" tab with an envelope icon, the header
  "Draft from planner-agent to Claude Code in repo", the line "Nothing is sent until you press
  Send. Edit it first if you like.", the text in an editor with no gutter, Send and Drop. The answer
  was `{"draft": …, "to": "Claude Code in repo", "opened": true}`; the stand-in's log was empty.
- **769-02-sent (REQ-002):** the tab closed, the terminal came forward, the stand-in printed
  `got: Please run the tests and tell me what fails.`, and a toast said "Sent to Claude Code in
  repo."
- **769-03-dropped (REQ-003):** after a second draft and Drop, only the terminal tab is left and
  the log has no new line.
- **769-04a-thread-draft, 769-04-thread (REQ-002):** a draft to the thread "hello", opened beside
  the Agent Panel, edited to end "Keep it short."; after Send the tab closed, the thread shows the
  edited message and "Noted: Review the plan in notes.md. Keep it short.", the agent's log holds
  it, and a toast says "Sent to the thread "hello"."
- **Refusals (REQ-004):** `no_terminal` for 99999 (with the terminals listed), `not_an_agent` for a
  bash terminal, `no_thread` for "nope", `bad_argument` for a blank text. 769-05 shows no draft tab
  opened for any of them.
- **REQ-005:** not reachable by a scenario that also sends; `mcp::answer` runs
  `agent_activity::gate` for every served Write tool, `draft_message` included (the review).
- **Not reached:** an agent CLI that waits on a question at Send, and a thread with the person's
  own text typed (both keep the draft with a note); covered by the review against
  `send_selection::target_of`'s `waiting` and `thread_tools::post`'s check.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide's Agent Control list (`draft_message`) and the
  in-app guide's Drafts table; `marley_mcp.md` (the `draft` family); `marley_workbench.md`
  ("Drafted messages"). No Zed path changed, so no ledger row.
- **Knowledge:** F-claude-769-send-looked-for-the-draft-tabs-own-window-and-found-it-gone-001
  (under PR-claude-596-look-up-across-windows-before-updating-one-001, no new rule);
  L-claude-769-a-multi-line-editor-shows-a-code-gutter-001;
  AD-claude-769-an-agent-drafts-to-another-agent-and-the-person-sends-001.
- **Brain:** `brain decide` on eb10b170204e476b83697f8907b0b67e.
