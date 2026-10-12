# New Thread with Context (second opinion and handoff, first slice) — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-717-second-opinion-and-handoff.md
- **Pipeline spec:** 717-second-opinion-and-handoff.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** MonoCode finding 4 (`docs/planning/intake/monocode-findings.md`): "Ask another agent"
  on a turn, a turn's diff or a worktree branch, opened in a split with a recap, and handoff to
  another provider. Chad, 2026-10-11, answering item 12 of the findings review: "12.) yes, with a
  star new thread with context would be cool". The starred part is this slice. It sits in the
  agents-together batch after the finish (Deliberate).
- **Classification:** feature, M.
- **Recall (§18.3):**
  - #734 and AD-claude-734: a thread Marley starts is filed under its own folders; thread tabs build
    on the hosting workspace's project.
  - #706 and AD-claude-706: agents post and answer through the panel's own paths. Its `thread_read`
    and `thread_post` enumerate only `AgentPanel::conversation_views`
    (`thread_tools.rs:61-86`), not tab threads (`thread_tab::conversations_of`, 955-967). That is a
    gap to note to #716, and the reason D4 does not point at `thread_read`.
  - #516: secret redaction for agents, the patterns this block passes through.
  - #555 (`send_block.rs:29-63`): a long block goes as a reference, not pasted. This slice does the
    same for what does not fit, with a file.
  - T3 report 01 item 3 and open question 4: the old session stays open beside the new one.
  - The ledgers hold no entry on Zed's from-summary or on `AgentInitialContent`.
- **Discovery:** see the spec's prior art for lines.
  - The menu is `ThreadTab::tab_extra_context_menu_actions` (`thread_tab.rs:230-239`), a Marley
    file. Zed's pane already calls an item's extra actions (`crates/workspace/src/pane.rs:2942`,
    `3125`), so no Zed touch is needed.
  - Initial content: carry it from `ThreadSpec` to `ConversationView::new` in `build_view`
    (`thread_tab.rs:906`).
  - Zed's own from-summary is reachable only from the token-limit callout, for its own agent, under
    an 80,000-token window (`crates/agent/src/thread.rs:125`). Nothing else uses
    `IconName::ThreadFromSummary`, which can be the menu item's icon.
  - A possible upstream bug, noted and left alone: `Thread::summary` keeps only the first line of
    each streamed chunk (`crates/agent/src/thread.rs:3989-3990`).
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again. A sketch follows.

### Design
- **Approach:**
  - **`crates/marley_agent/src/handoff.rs`** (new, pure; Marley crate):
    - `Item { role, text, label }`, where the role is a request, an answer or a tool call;
    - `select(items, budget) -> Selection` in T3's order, whole items only;
    - `render(selection, title, agent, file) -> String`, with the preamble, the labels in
      conversation order and the coverage line.
    No IO, no gpui.
  - **`crates/marley_workbench/src/handoff.rs`** (new; Marley crate):
    - the action `NewThreadWithContext` and its palette entry;
    - the agent picker, `thread_agents` without seat or Manager entries, the thread's own agent
      first;
    - gathering `Item`s from the live `AcpThread`;
    - redaction;
    - writing the markdown file with mode 0600 under the data folder through an `*_in(dir)` helper
      (§14);
    - the split, then `thread_tab::start_with_content`.
  - **`thread_tab.rs`:** `ThreadSpec` gains `initial: Option<AgentInitialContent>`, passed in
    `build_view`, and the tab menu gains New Thread with Context….
  - **`marley_workbench.rs`:** the module and its `init`.
- **File manifest:** `marley_agent/src/handoff.rs` and `marley_agent.rs` (the module line),
  `marley_workbench/src/handoff.rs`, `thread_tab.rs` and `marley_workbench.rs` (all Marley crates);
  the guide; the scenario. No Zed path is touched, so no ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | A tab on `Scripted A`, its conversation played; right-click the tab; choose the item | 717-01-menu, 717-02-picker |
| 002, 003, 005, 007 | Pick `Scripted B` | 717-03-prefilled |
| 006 | Add "carry on from here", Enter | 717-04-sent |
| 004, 005 | A second run with 6 KB answers | 717-05-budget; a check of the file |

- **Not reached:** the real Claude Agent and Codex. Each receives a plain text block, the same as the
  stand-in, and the review covers them.

### Risks
- Zed's own agent may not read a file outside the project's folders. The selection still carries
  the context; the file line is for agents that can read it. The guide says so.
- A thread whose single latest answer is over the budget gets that answer left out. The block then
  holds the latest request and the first request, and the coverage line names the file.
- Prefilled text sits in the editor's undo history. Sending it unedited is the expected case.

### Promotion (2026-10-11)
- **Promoted** from `queued/` to `active/` under the goal "continue until all tickets are
  finished": the finish-line queue is done, so the after-the-finish shelf comes next (#757, #719
  and #716 stay where Chad put them: shelved, moved to the harness, and waiting on the harness).
  The BACKLOG row removed; the ticket `in-progress`, its link on `active/`.
- **Brain:** consultation 49281a5ea1c741aea7b552b30674951c, nothing on this seam.
- **Seams checked again** (an Explore pass at `892a795b3d`). What changes the design:
  - `ThreadTab::tab_extra_context_menu_actions` (`thread_tab.rs:249-261`) returns nothing in the
    Marley layout (#772), so New Thread with Context… goes in both layouts' lists.
  - `ThreadSpec` (954-967) has `session` since #758; it gains `initial`, passed as
    `ConversationView::new`'s eighth argument in `build_view` (1158-1173), `None` today.
    `AgentInitialContent::ContentBlock { blocks, auto_submit }` (`agent_ui.rs:624`) with
    `acp_v2::ContentBlock::Text(acp_v2::TextContent::new(text))` (`agent_panel.rs:4876`); applied
    when the `ThreadView` is made (`thread_view.rs:833-868`).
  - The live thread: `ConversationView::root_thread_view()?.read(cx).thread` (`root_thread` is
    crate-private); `agent_key()`, `title(cx)`, `parent_id()`; `conversations_of` (1215) lists a
    workspace's panel and tab views.
  - The history: `AcpThread::entries`, `title`, `work_dirs`, `to_markdown`. A user message's text
    is `content.to_markdown(cx)`; an assistant message's `Message` chunks only (no `Thought`); a tool
    call's `label.read(cx).source()`, `status()` (`Display`), and paths from `diffs()`
    (`file_path(cx)`) and `locations`.
  - The split: `find_pane_in_direction(Right)` (relative to the active pane), else `split_pane`;
    `add_item(pane, item, None, true, true, window, cx)`.
  - Redaction: `crate::mcp::agent_redactor(cx)` (`None` while the setting is off).
  - The file: `paths::data_dir()/handoffs`, written off the main thread (the dylint
    `blocking_io_on_foreground`), mode 0600 as `agent_activity.rs:271-282` does.
  - The picker: a small `PickerDelegate` over `agents::thread_agents` without seat and Manager
    entries (`harness::manager_of_entry`).
  - The scenario: #749's `profile_setting agent_servers.<name>` with a scripted ACP agent, and
    #772's tab right-click (`click X Y right`).

## Phase 2 — Code (2026-10-11)
- **Built:** `marley_agent/src/handoff.rs` (pure: `Item`, `Kind`, `Source`, `BUDGET`, `block`,
  `select`, the preamble, the sections, the coverage line) and its module line;
  `marley_workbench/src/handoff.rs` (`NewThreadWithContext`, `init`, `source_view`, `thread_of`,
  `open_picker`, `picker_agents`, `items_of`, `redacted`, `write_handoff_in`, `start`,
  `AgentPickerDelegate`); `thread_tab.rs` (`ThreadSpec.initial` into `ConversationView::new`,
  `active_conversation`, `open_beside`, the menu item in both layouts, `refuse` crate-visible,
  `handoff::init` from `thread_tab::init`). The guide, `marley_workbench.md`, `marley_agent.md`,
  CHANGELOG.
- **Deviations from the design:** the preamble's first line is `Continuing "<title>" from
  <agent>.`, because Zed titles the thread from it (L-claude-717-…); a title passed for the new
  thread does not show, so none is passed.
- **The review:** the file is written off the main thread; a failed write leaves the block naming
  no file and logs; the picker acts after it closes, so the old thread's pane is active; the old
  thread is only read.
- **Clippy:** `derive Eq` on the action, `needless_borrow`, `map_or_else`, two
  `needless_pass_by_ref_mut`, and `too_many_lines` on the crate's `init` (the module's `init` moved
  into `thread_tab::init`). Clean.
- **The gate:** the first run red on dylint's `async_block_without_await` (the handoff file's
  write in an `async` block); `futures::future::lazy` as the crate does elsewhere, and the second run
  green, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/717-second-opinion-and-handoff.sh`, under `compositor sway`: two
  scripted ACP agents as custom agent servers, a key that opens a thread tab on `Scripted A`. Exit
  0, six checks passing. Shots read (the last run):
  - `717-01-menu`: the tab's menu, New Thread with Context… last, under Zed's Close and Pin items
    (REQ-001).
  - `717-02-picker`: Scripted A, Zed Agent, Claude Agent, Scripted B (REQ-001).
  - `717-03-prefilled`: a split on the right, a tab on Scripted B whose message box holds
    `Continuing "the first request" from Scripted A.`, the history line, each request and answer
    labelled, `[Earlier tool call: Edit notes.md, Completed]` with its path, the key shown as
    `[redacted: github token]`, and the file line; Scripted A's tab unchanged on the left (REQ-002,
    REQ-003, REQ-005, REQ-007).
  - `717-04-sent`: " carry on from here" sent; B answers "Got 900 bytes. First line: Continuing
    "the first request" from Scripted A. Last line: carry on from here"; the tab is titled from the
    first line (REQ-006).
  - `717-05-budget`: after three 6 KB answers, a second handoff into the right pane; its last line
    "1 of 15 earlier items are left out. The whole conversation is in …/handoffs/….md" (REQ-004).
  - Checks: a handoff file holds the conversation and the third big answer, and no file holds the
    token (REQ-005).
- **Fixes on the way:** Home in the tab menu picked Close and shut A's tab (L-claude-717-…, the
  item clicked by point); the log check's pattern; the preamble's first line as the title.
- **What no scenario shows:** the Agent Panel as the source and the real Claude Agent and Codex,
  which take the same text block; read in the diff.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #717); the guide's "New Thread with Context (#717)";
  `marley_workbench.md` and `marley_agent.md`. No Zed path touched.
- **Knowledge appended:**
  AD-claude-717-a-new-thread-carries-a-selection-of-the-old-ones-history-001;
  L-claude-717-an-acp-threads-title-is-its-first-messages-first-line-001;
  L-claude-717-zeds-tab-menu-puts-marleys-items-last-001.
- **Brain:** consultation 49281a5ea1c741aea7b552b30674951c closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. The second opinion
  proper ("Ask another agent" on a turn) is this block's next use, not yet ticketed.
