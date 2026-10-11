# New Thread with Context (second opinion and handoff, first slice) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-717-second-opinion-and-handoff.md
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
