# Notes on any Markdown file or the agent's last reply — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-729-notes-on-markdown-and-last-reply.md
- **Pipeline spec:** 729-notes-on-markdown-and-last-reply.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 4 (`docs/planning/intake/plannotator-findings.md`): "Extend
  #522's Add Review from diffs to Zed's Markdown preview and to a terminal's or thread's last
  message, with the same agent picker and readiness rules." Chad, 2026-10-11, on the finish-line
  list (`docs/planning/design-notes/finish-line-2026-10-11.md`): "11.) yes". Everything opens in a
  center tab (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`). It waits under
  Deliberate with the rest of the review batch until after the finish.
- **Classification:** feature, medium to large: one review tab with three sources and two
  deliveries, all over seams that exist. If the Plan phase finds it too big, the split is the
  file (REQ-001, REQ-002) first and the replies (REQ-003 to REQ-008) second.
- **Deviation from the ticket:** the notes are made in #726's review tab, not inside Zed's Markdown
  preview (D1). The picker and readiness rules are #522's, as the ticket asks.
- **Recall (§18.3):**
  - AD-claude-522-review-notes-go-to-an-idle-terminal-agent-and-stay-marked-sent-001: readiness from
    the seat (`Idle`), the paste then Enter, notes kept marked Sent, Copy.
  - AD-claude-706-agents-post-and-answer-through-the-panels-own-paths-001: a post goes through the
    thread's message editor and `ThreadView::send`; refused while the user has a draft.
  - AD-claude-652-an-agents-report-outranks-its-hook-frames-001: the hook frames still carry the
    prompt and the tool; `transcript_path` rides on them.
  - PR-claude-keys-after-a-paste-go-in-a-later-write-001: the Enter after a paste goes through
    `terminal_drive::paste_then`, which `deliver` already uses.
  - F-claude-592-the-launch-approval-was-rendered-as-markdown-001: the tab draws with HTML not
    parsed (#726 D8); a file's or reply's comments stay visible.
  - #726's notes: the tab, `selected_range`, the notes as source ranges; #732's template wraps every
    send through `review_notes.rs`.
- **Discovery:**
  - `crates/marley_workbench/src/review_notes.rs`: `open_picker` (38), `holds_every_file` (85),
    `readiness` (93), `deliver` (108), `prompt` (162).
  - `crates/marley_workbench/src/send_selection.rs`: `Target`, `agent_targets`, `TargetPicker`,
    `Row`, `Pick`, `OnPick` (#549, generalized in #522).
  - `crates/markdown_preview/src/markdown_preview_view.rs`: `MarkdownPreviewView` (84),
    `active_editor` (87), `resolve_active_item_as_markdown_editor` (344), `is_markdown_file` (536).
  - `crates/acp_thread/src/acp_thread.rs`: `AssistantMessage` (361), `AssistantMessageChunk` (380),
    `AgentThreadEntry` (486), `MessageContent::to_markdown` (2000), `entries` (3842).
  - `crates/agent_ui/src/conversation_view.rs`: `root_thread_view` (861);
    `crates/agent_ui/src/conversation_view/thread_view.rs`: `thread` (573), the turn's gathering in
    `get_agent_message_content` (8325, private), the last assistant index (7191).
  - `crates/marley_workbench/src/thread_tab.rs`: `conversations_of` (955), `tab_threads` (971);
    `thread_tools.rs`: `thread_named` (89), `post` (265).
  - `crates/marley_agent/src/claude_events.rs`: `transcript_path` (91), set for the lead (396);
    `crates/marley_workbench/src/agent_events.rs`: `seat` (70).
  - `crates/marley_workbench/src/rail.rs`: the thread row's menu (5442), `terminal_context_menu`
    (5595).
- **Decisions:** D1 to D6 in the spec.

### Design
- **`crates/marley_workbench/src/plan_review.rs`** (Marley crate, #726's): `Subject { Plan(..),
  File { path, text }, Reply { origin: Terminal(view) | Thread(conversation), text } }`; the tab's
  title and buttons by subject; notes gain `sent`; Send Notes counts the unsent.
- **New `crates/marley_workbench/src/last_reply.rs`** (Marley crate):
  - `of_thread(conversation, cx) -> Option<String>`: the last turn's assistant `Message` chunks
    joined with blank lines, as the private gatherer does.
  - `of_terminal(view, cx) -> Result<String, &'static str>`: the seat's `transcript_path` label,
    read on the background executor (bounded size), the `text` blocks of the `assistant` entries
    after the last `user` entry that holds a prompt; errors for no seat, no path, an unreadable
    file, no reply.
- **The commands** (`marley_workbench.rs`): `marley: review markdown` (the active Markdown editor,
  or the preview's source editor) and `marley: review last reply` (the active thread tab, the
  Agent Panel's active thread, or the active agent terminal).
- **`rail.rs`:** Review Last Reply in the thread row's menu and, for a terminal whose agent is
  Claude Code, in `terminal_context_menu`.
- **`review_notes.rs`:** a send that takes plain notes (path, lines, quote, words) and either the
  picker (a file) or one fixed target (a terminal reply), using #732's template; the toasts for a
  target not ready.
- **`thread_tools.rs`:** `post`'s draft check and send moved into a function both the tool and the
  tab call.
- **`crates/marley_agent/src/marley_agent.rs`:** `ReviewLine.quote: Option<String>` (the `Quote:`
  line, escaped as the comment is; lines optional for a reply); the reply's lead line.
- **The guide:** "Review a Markdown file or a reply".
- **File manifest:** `plan_review.rs`, `last_reply.rs` (new), `review_notes.rs`, `thread_tools.rs`,
  `rail.rs`, `marley_workbench.rs` (Marley crate `marley_workbench`); `marley_agent.rs` (Marley crate);
  the guide; the scenario. No Zed path: every seam named is public. No ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Opens `README.md`; runs `marley: review markdown` | 729-01-file |
| 002 | Drags over a passage, `c`, a note, Enter; Send Notes; Enter on the ready row | 729-02-file-sent: the echo |
| 003 | The stand-in sends a `Stop` frame naming the scratch transcript; Review Last Reply from the terminal's rail menu | 729-03-terminal-reply |
| 004 | A note; Send Notes | 729-04-terminal-sent: the echo; the note Sent |
| 005 | In the thread tab, one prompt, the scripted answer; `marley: review last reply` | 729-05-thread-reply |
| 006 | A note; Send Notes | 729-06-thread-sent: the notes as the user's message; "Noted: …" |
| 007, 008 | (review of the diff: a working agent, a draft, no transcript) | — |

### Risks
- Claude Code's transcript format is not a published contract; the reader takes only `type`,
  `message.role` and `text` blocks, and says "no reply found" when they are missing.
- A transcript can be large; the reader reads it from the end, bounded, off the main thread.
- A reply review of a terminal agent that has started a new turn sends nothing (it is working),
  which is right; the tab keeps the notes for a later send.
