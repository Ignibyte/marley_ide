# Thread states as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-927-thread-states-as-the-board.md
- **Pipeline spec:** 927-thread-states-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (ThreadStates).
- **Classification / tier:** feature, M. Marley crates, plus additive hunks in
  `agent_ui/src/conversation_view/thread_view.rs` under `MarleyThreadLook`.
- **Recall (§18.3):**
  - F-claude-798-a-pane-bounds-read-in-render-deadlocked-the-app-001 and PR-claude-798: the width
    label reads the row's flexes (`marley_row_share`), never the pane's bounds; the fraction is
    formatted from that same share.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: `tool_call_view` is called
    from the thread view's render; the page card takes what it needs from the call's input and
    reads no thread view. Show's click runs later, outside the render.
  - PR-claude-785-a-menu-that-dispatches-a-pane-action-names-the-pane-as-its-context-001: the
    tab's extra entries are dispatched with the item's focus handle as context
    (`pane.rs:3131`); the new actions are handled on the thread tab's element, so they reach it.
  - L-claude-825-a-scripted-acp-agent-writes-a-file-through-the-client-001: the scripted agent
    reports tool calls and asks permission over ACP; its `cwd` is Marley's agent folder.
  - F-claude-555-a-menu-walked-by-its-keys-opened-zeds-own-agent-001: the scenario clicks menu
    entries by position from a shot, never walks them by keys.
  - L-claude-832-a-right-click-menu-opens-at-the-pointer-so-click-its-entries-inside-001.
- **Discovery:**
  - `crates/marley_workbench/src/agent_panel_width.rs`: `width_label` (81).
  - `crates/agent_ui/src/conversation_view/thread_view.rs`: `marley_card_label` (583), the RUN
    slot (8837), the EDIT label (11007), `render_permission_buttons_flat` (10727).
  - `crates/marley_workbench/src/component_tools.rs`: `tool_call_view` (31),
    `is_component_show` (61); `browser_pages.rs`: `show_page` (70), `page_file_name`.
  - `crates/marley_agent/src/handoff.rs`: `block`, `select`, `preamble` (104);
    `crates/marley_workbench/src/handoff.rs` (250).
  - `crates/marley_workbench/src/thread_tab.rs`: `tab_extra_context_menu_actions` (319),
    `no_model_callout` (611), `render` (642), `open_beside`.
  - `crates/marley_workbench/src/rail.rs`: thread menu (6531), `delete_thread` (3423),
    `archive_thread`, `rename_terminal` (the rename field).
- **Decisions:** D1–D5 in the spec.

### Design
- Width: `width_label` maps the share to `WidthStep { Third, Half, TwoThirds, Whole }` by the
  nearest value and returns its label; a zoomed thread is `Whole`.
- ASKS: in the header's label slot, `if waiting { ASKS + chip(tool_name) } else { RUN | EDIT }`.
  The chip is a mono `Label` on `element_background` with `rounded_sm`. The tool's name is
  `tool_call.tool_name`, else the label's first word.
- Button words: in `render_permission_buttons_flat`, the label is
  `marley_option_label(option, cx)`: "Allow" for `AllowOnce`, "Deny" for `RejectOnce`, else
  `option.name`, only while `MarleyThreadLook` is set. The row is `h_flex().flex_wrap()` under
  the look; Zed's layout keeps `v_flex`.
- Page card: `tool_call_view` matches `browser_show_page` by the same bare-name rule as
  `component_show` (one helper, `bare_tool_name`). The file name is `page_file_name(name)` for
  `html`, the path's file name for `path`. Show calls `browser_pages::show_tab_of(file_name,
  window, cx)`, which finds the Browser tab whose address is that page in the window's
  workspaces and activates it.
- Handoff: `handoff::block` counts the chosen items by kind into `HandoffCounts { requests,
  answers, tool_calls }` (named fields) and returns them with the text; `preamble` writes the
  counts. `open_beside` takes an optional notice; `ThreadTab` keeps `handoff_notice:
  Option<SharedString>` and draws it framed at the top of the conversation column until the
  thread has a user message.
- No-model card: `no_model_card` returns a `v_flex` card (rounded, bordered, elevated surface,
  a 32 px "Z" tile, title, line, filled Configure Providers), drawn `absolute` and centred over
  the conversation column while the thread has no entries; above the conversation otherwise.
- Menu: actions `RenameThread`, `ArchiveThread`, `DeleteThread` in `thread_tab.rs`, handled by
  `ThreadTab`'s element; Review Last Reply uses `plan_review::ReviewLastReply`. Archive and Delete
  call shared functions moved out of the rail (`rail::archive_thread` becomes
  `pub(crate)`; `delete_thread`'s prompt and delete become a free function both call). Rename
  opens the rail's rename field and writes the title through `AcpThread::set_title`, as Zed's
  title editor's handler does.
- File manifest:
  - Zed crate: `crates/agent_ui/src/conversation_view/thread_view.rs`.
  - Marley crate: `agent_panel_width.rs`, `component_tools.rs`, `browser_pages.rs`,
    `marley_workbench/src/handoff.rs`, `marley_agent/src/handoff.rs`, `thread_tab.rs`, `rail.rs`.
- Ledger: the `crates/agent_ui/src/conversation_view/thread_view.rs` row in
  `docs/marley/zed-touchpoints.md` gains the ASKS label and tool chip on a waiting call, and the
  Allow and Deny words of the flat permission buttons, under `MarleyThreadLook` (#927).

### Visual check plan
- REQ-001: a file and the thread tab side by side (split right); `927-01-asks` shows "½".
- REQ-002: the scripted agent asks permission for `terminal_run`; `927-01-asks` shows ASKS, the
  chip and Allow · Deny.
- REQ-003: the scripted agent reports a completed `browser_show_page` with `html` and
  `name: report`; the call also reaches Marley's MCP server so the Browser tab exists (or the
  scenario opens it with `browser_show_page` from the stand-in MCP client first). `927-02-page`,
  then Show clicked: `927-03-shown`.
- REQ-004: the tab's menu, New Thread with Context…, Marley's agent picked; `927-04-handoff`.
- REQ-005: `agent.default_model` cleared in the run's profile and a Zed Agent thread opened;
  `927-05-no-model`. No provider is configured, so no model call can start.
- REQ-006: right-click on the thread's tab: `927-06-menu`; Rename, a new title typed, Enter:
  `927-07-renamed`.
- Marley Day: `927-08-day`.

### Risks
- The scripted agent's `browser_show_page` row may not carry `tool_name` the way Claude Code's
  does; the card matches the bare name in the title too, as `component_show` does.
- Moving Archive and Delete out of the rail must keep the rail's entries working; the Code phase
  checks the rail row's menu in the same run.
- A centred card over the conversation must not cover the message box; it is placed in the
  column above the composer.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
