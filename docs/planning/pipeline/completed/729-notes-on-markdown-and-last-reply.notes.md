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

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation 2df63ab8cc154a2e9e1ffb03e1ff8d98) had nothing on the seam; a
  duplicate ask was closed with no decision.
- **Seams re-verified** by an Explore agent at `97318db8c4`. All hold. Three line numbers moved
  (`review_prompt` is at `marley_agent.rs:412`, `conversations_of` at `thread_tab.rs:1287`, the rail's
  menus at `rail.rs:6195` and inside `render_thread_row` at about 5986), and `event.py` lives under
  `crates/marley_workbench/claude_plugin/`. New findings:
  - `MarkdownPreviewView.active_editor` is private, but its `Item::act_as_type` hands out the
    source `Editor`, so `active_item(cx)?.act_as::<Editor>(cx)` serves a Markdown editor and the
    preview alike; `is_markdown_file` is public.
  - `drafts.rs` already has `send_to_terminal` and `send_to_thread` (#769), the draft check and
    the thread's own send outside MCP; they become `pub(crate)` instead of moving `post`.
  - `AssistantMessage::to_markdown` adds a heading and the thoughts; the reply is gathered from the
    `Message` chunks, as the private `get_agent_message_content` does.
  - The seat's label is the literal `"transcript_path"`, set for the lead agent only.
- **Design, as built on these seams:**
  - `plan_review.rs`: a `Subject` (`Plan { view }`, `File { path }`, `Reply(ReplyOrigin)` with
    `ReplyOrigin::Terminal { view, project }` or `Thread { conversation, workspace }`) in place of
    the tab's terminal id; `State::Open` for a document, which takes notes with no request waiting;
    a note's `sent` mark; the column's title, status and Send Notes by subject; question cards for a
    plan only. `open_document` opens a tab per subject and text in the active pane, or brings back
    the one already showing the same text.
  - The commands `marley: review markdown` and `marley: review last reply`, registered by
    `plan_review::init`: the active item's `Editor` when `is_markdown_file`; the active thread tab,
    the active terminal, else the Agent Panel's thread.
  - `last_reply.rs` (new): `of_thread` (the `Message` chunks after the last user message);
    `transcript_of` (the seat's label); `read_reply` on the background executor, the file's last
    16 MiB; `reply_in` (assistant `text` blocks after the last user prompt, side chains and meta
    entries skipped).
  - `review_notes.rs`: `send_file_notes` (#522's picker for one file, lines taken as they are,
    `on_sent`), its prompt built through `prompt_of_lines`.
  - `marley_agent.rs`: `ReviewLine.quote` (a `Quote:` line), `first == 0` for a note with no lines;
    `reply_notes` ("Notes on your last reply:", quotes and comments, `Overall:`).
  - `rail.rs`: Review Last Reply in the terminal menu and the thread row's menu (the thread opened,
    then the command).
  - Sends: a file's through the picker; a terminal reply's through `drafts::send_to_terminal` after
    the seat reads ready; a thread reply's through `drafts::send_to_thread`. Each failure says why
    in a toast; a sent note is marked Sent, the overall note kept as one.
- **Manifest:** `plan_review.rs`, `last_reply.rs` (new), `review_notes.rs`, `drafts.rs`, `rail.rs`,
  `marley_workbench.rs` (module) in `marley_workbench`; `marley_agent.rs`; the guides; the
  scenario. No Zed path.
- **Visual check:** as planned above, with the thread opened by a key bound to
  `marley::NewAgentThread` on a scripted agent (#717's way), and the terminal's stand-in sending a
  `Stop` frame through Marley's plugin hook with a scratch `transcript_path`.

## Phase 2 — Code
- **Built:** `plan_review.rs` (`Subject`, `ReplyOrigin`, `State::Open`, `Note.sent`,
  `show_document`, `plan_view`, `taking_notes`, `unsent`, `send_notes`, `mark_sent`, `status`,
  `send_notes_button`, the column and the tab by subject, the overall editor redrawing the column;
  the actions `marley::ReviewMarkdown` and `marley::ReviewLastReply`; `review_markdown`,
  `review_last_reply`, `review_thread_reply`, `review_terminal_reply`, `open_document`,
  `send_to_reply_terminal`, `lines_of`, `show_toast`); `last_reply.rs` (new); `review_notes.rs`
  (`send_file_notes`, `row_of`, `refused`, `paste_and_submit`, `message_of`, `reply_message`);
  `drafts.rs` (the two sends crate-visible); `rail.rs` (the two menu entries); `marley_agent.rs`
  (`ReviewLine.quote`, lineless notes, `reply_notes`); the guides.
- **Deviations from the design:** none in substance. The commands are registered by
  `plan_review::init`, the tab's own module. A file's tab opens in the active pane, which is the
  file's.
- **The review found:** nothing before the first run; the run found the `c` regression below.
- **The gate:** see Phase 3's fix; the gate ran once on the final tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/729-notes-on-markdown-and-last-reply.sh` under `compositor sway`.
- **A bug the first runs found:** `c` made no note (`729-01a-dragged-c`: the passage selected, no
  note editor). #728's focus check asked whether the plan area's own handle held the focus, and the
  Markdown element takes it on a mouse-down; fixed to propagate only while a card's field contains
  the focus (F-claude-729-the-plans-c-key-made-no-note-after-728-001; #728's lesson corrected).
  The next run clicked where Send Notes stood with no note listed and hit the note
  (L-claude-729-a-columns-button-moves-as-its-notes-list-grows-001); re-placed.
- **729-01-file** (REQ-001): `Review · README.md` beside `README.md`, the file drawn (heading,
  paragraph, list), the column "Notes go to an agent working on this file.", the overall field
  "A note on the whole file" and Send Notes off.
- **729-01a-dragged / -c:** the passage selected; after `c` the note's editor.
- **729-02a-picker** (REQ-002): the note listed, Send Notes (1), the picker "Send the notes on this
  file to…" with "Claude Code · repo · ready" and Copy notes.
- **729-02-file-sent** (REQ-002): the terminal shown; the stand-in echoes the template, `File:
  README.md`, `Line: 3`, `Quote: "The search reads the index on every keystro"`, `User comment:
  "Say which index"`; the check passed.
- **729-03a-menu, 729-03-terminal-reply** (REQ-003): the terminal row's menu with Review Last
  Reply; the tab `Reply · Claude Code · repo` with the transcript's two texts ("I read the search
  code." … "**twice**" … "Shall I add one?"), the thinking and the tool's result left out.
- **729-04-terminal-sent, 729-04b-marked** (REQ-004): the echo of "Notes on your last reply:",
  `Quote: "A cache would halve the reads"`, `User comment: "Only for file names"` (check passed);
  back on the tab the note marked Sent and Send Notes off.
- **729-05-thread-reply** (REQ-005): after "check the cache" in the scripted thread, `marley:
  review last reply` opens `Reply · check the cache` with "Noted: check the cache" and its second
  paragraph.
- **729-06-thread-sent** (REQ-006): the thread shows the notes as the user's message (template,
  "Notes on your last reply:", the quote, the comment) and the agent's answer; the agent's log holds
  them (check passed).
- **Not reached by the scenario:** REQ-007 (an agent at work, a thread with a draft) and REQ-008
  (no transcript), by review of the diff: `send_to_reply_terminal` refuses a seat that is not
  ready with the readiness in a toast; `drafts::send_to_thread` refuses a draft; a terminal with no
  `transcript_path` label is a toast before anything opens.
- **The gate:** `script/gates.sh --diff` on the final tree: 17 of 17 green, receipt written; the
  scenario ran again after it with every check passing and the shots as above.

## Phase 4 — Complete
- CHANGELOG (Added: notes on a Markdown file or a reply; Fixed: `c` in the Plan tab);
  `docs/marley/guide.md` and the in-app guide (a section and an article of their own);
  `docs/marley_architecture/marley_workbench.md` (the review tab's subjects, `last_reply.rs`, the
  rail's entries, `review_notes`' shared send) and `marley_agent.md` (`ReviewLine.quote`,
  `reply_notes`). #728's completed notes carry a correction of their focus report.
- Knowledge: F-claude-729-the-plans-c-key-made-no-note-after-728-001,
  L-claude-729-a-markdown-element-takes-the-focus-on-a-mouse-down-001,
  L-claude-729-a-columns-button-moves-as-its-notes-list-grows-001,
  AD-claude-729-one-review-tab-for-plans-files-and-replies-001; L-claude-728-a-letter-bound-on-an-area-yields-to-a-field-inside-it-001
  corrected.
