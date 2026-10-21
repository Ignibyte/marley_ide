# The agent thread with a details panel, bubbles and a composer card — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-825-agent-thread-with-panels.md
- **Pipeline spec:** 825-agent-thread-with-panels.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #827; Chad's overnight goal.
- **Classification / tier:** feature, L scoped to M: the plan checklist, card labels, reply chips,
  terminal peek and checks are TICKET-864 (queued before #794).
- **Recall (§18.3):**
  - PR-claude-798: no pane bounds read in render (a deadlock); the panel reads entities only.
  - PR-claude-687 / #822's scenario: a scripted ACP agent as Marley's own agent
    (`MARLEY_ASSISTANT_ADAPTER`), and check it runs before typing.
  - The brain (consultation 596cf904): nothing on this seam.
- **Discovery** (an Explore map, this session): `thread_view.rs` user message 6888-7084 (the box at
  6967-6998, the outer `v_flex` at ~6930 `.w_full().px_2()`), the composer
  `render_message_editor` 4975-5121, `MarleyThreadHost` (`agent_ui.rs:517-555`, set in
  `thread_tab.rs:146`), #822's `render_marley_text_width` (6393), `ThreadTab::render`
  (`thread_tab.rs:607`), `root_thread_view` (`conversation_view.rs:887`), `TokenUsage`
  (`acp_thread.rs:3371`, `token_usage` :4128), `action_log` (:3896), `changed_buffers`
  (`action_log.rs:1026`), `diff_stats` (:1037), `AcpThreadEvent` (:3601).
- **Decisions:** D1 to D4 in the spec.

### Design
- **`thread_tab.rs`** (Marley):
  - `ThreadDetails` settings struct (`RegisterSetting`) over `marley.thread_details`, default
    true; `toggle_thread_details(cx)` writes its negation.
  - `ThreadTab` gains `details: Entity<ThreadDetailsPanel>`, made in its constructor from the
    `ConversationView` (no workspace read).
  - `render`: `h_flex().size_full()` of the existing `v_flex` (flex_1, min_w_0) and, while on,
    the panel (`w(px(280.))`, `border_l_1`, `panel_background`); while off, a small
    `IconButton` (`IconName::ThreadsSidebarRightClosed` or `PanelRight`) absolutely at the
    tab's top right to show it.
  - `ThreadDetailsPanel { conversation: Entity<ConversationView>, _subscriptions }`: observes the
    conversation; on each render it reads `root_thread_view()`, its `thread`, and subscribes
    (once per thread entity id) to the thread's events and observes its action log, so token use
    and edits redraw.
  - Panel content: header (`DETAILS` and the hide button); the agent (`agent_icon` +
    `agent_display_name`), the model (`current_model_id`, else "default model"); CONTEXT
    (`humanize_token_count(used) of humanize_token_count(max)` and a 4 px bar, or "not
    reported"); FILES CHANGED (`action_log.changed_buffers(cx)`: each buffer's file path
    relative to its worktree, `+a −r` from the diff's hunks via `diff_stats` or the buffer diff,
    a click opening the buffer in the workspace through the tab's workspace weak handle), or
    "no files changed yet".
- **`thread_view.rs`** (Zed, `agent_ui`): two `// Marley:` hunks behind
  `cx.has_global::<MarleyThreadHost>()`:
  - the user message: on the outer `v_flex`, `.when(marley && !is_indented, |this|
    this.items_end())`; on the box, `.when(marley && !is_indented, |this|
    this.max_w(relative(0.85)).rounded_xl().bg(accent.opacity(0.08)))`.
  - `render_message_editor`: `.when(marley, |this| this.mx_3().mb_3().rounded_lg().border_1()
    .border_color(border))` and the top border only when not marley.
  Row 122 of `zed-touchpoints.md` gains the clause.
- **Settings:** `settings_content/src/marley.rs` `thread_details: Option<bool>`; `default.json`
  `"thread_details": true`; touchpoint clauses on both rows. The action in
  `marley_workbench.rs`'s `actions!` with its global handler.
- **File manifest:** `thread_tab.rs` (Marley), `marley_workbench.rs` (Marley),
  `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed), `settings_content/src/marley.rs`
  (Zed crate, Marley file), `assets/settings/default.json` (Zed asset),
  `docs/marley/zed-touchpoints.md`, the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001..004 | #822's scripted agent, Marley's thread from repo's +; prompt 1 (`write notes`): the agent writes `notes.md` via `fs/write_text_file`, answers; prompt 2 answers | `825-01-night` |
| REQ-005 | click the panel header's hide button; print the setting | `825-02-hidden` |
| REQ-001..004 | the tab's show button clicked; theme Marley Day | `825-03-day` |

Context use: the scripted agent reports none, so the shot shows "not reported" (D4); a reported
use is read in the diff.

### Risks
- `fs/write_text_file` from the scripted agent needs a request id answered before the turn ends;
  the agent reads the client's response line before going on.
- The bubble's `items_end` must not move the edit overlay or the checkpoint row; the shot shows.

## Phase 2 — Code
- **Built:** `thread_view.rs` (Zed): `marley_bubble` (a top-level user message while
  `MarleyThreadHost` is set): the outer column `items_end`, the wrapper `max_w(relative(0.75))`,
  the box `rounded_xl` on the accent at 8 % with `px_3`; `marley_card` in
  `render_message_editor`: `mx_3 mb_3 rounded_lg border_1`, the top border only without it.
  `thread_tab.rs` (Marley): `ThreadDetails` (`RegisterSetting` over `marley.thread_details`,
  default true), `toggle_thread_details`, the action `ToggleThreadDetails` with its global
  handler, `ThreadTab.details_thread` (observes the shown `AcpThread` and its `ActionLog`, once
  per thread), `render` as an `h_flex` of the conversation and the panel (or the show button),
  `render_details` (agent icon and name, model, CONTEXT with a bar from `token_usage`, FILES
  CHANGED from `changed_buffers` with `DiffStats::single_file`, each opening its file),
  `context_section`, `files_section`, `humanize`. The settings field, `default.json`, the
  touchpoint clauses; `action_log` added to `marley_workbench`'s dependencies.
- **Deviations:** the bubble is capped at 75 %, not the planned 85 %: it holds Zed's message
  editor, which takes the full width it is given, so the cap is the bubble's width; 75 % read as a
  bubble in the probe, 85 % as a box. The show button uses `IconName::ListTree`.
- **Review of the diff:** the panel subscribes in render only when the thread entity changes (a
  keyed `Option`), so no subscription piles up per frame; the share is computed in thousandths
  through `u16` (no lossy casts).
- **Gate:** clippy at source (five findings: `map_or_else`, `Arc::clone`, three lossy casts);
  `script/gates.sh --diff` 17/17 after the 75 % change.

## Phase 3 — Test
- **Scenario:** `script/e2e/825-agent-thread-with-panels.sh` (sway, the gated tree): the scripted
  agent ran (`check … pass`), exit 0, three shots. One probe run before the gate.
- `825-01-night`: the thread tab: both user messages right-aligned in rounded bubbles on a warm
  tint at 75 % width, the agent's "Noted: …" answers left; the message box a rounded, bordered card
  with margins under Zed's Edits bar; the DETAILS panel at the right with Marley's icon and name,
  "its default model", CONTEXT "not reported by the agent" (the scripted agent sends no usage),
  FILES CHANGED `notes.md +4 −0` (REQ-001 to REQ-004 met).
- `825-02-hidden`: the panel's × clicked: the conversation takes the tab's width, a show button at
  the tab's top right, and the log prints `thread_details: False` (REQ-005 met).
- `825-03-day`: the show button clicked, Marley Day: the panel back, the bubbles and card in Day's
  colors (REQ-001 to REQ-004 met).
- Not reachable here: a reported context use (the scripted agent reports none); its bar is read in
  the diff.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #825); the guide's thread tab section; the architecture note's
  thread tab section; look-plan TICKET-825 done with #864; touchpoint row for `thread_view.rs`
  checked.
- **Knowledge:** L-claude-825-a-scripted-acp-agent-writes-a-file-through-the-client-001.
- **Brain:** decision on consultation 596cf904.
- **Ticket** closed; #864 filed; pipeline archived; committed on `marley/workbench-shell`.
