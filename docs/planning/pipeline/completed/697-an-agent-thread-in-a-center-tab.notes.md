# An agent thread in a center tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-697-an-agent-thread-in-a-center-tab.md
- **Pipeline spec:** 697-an-agent-thread-in-a-center-tab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-08 (#3 of his three): move an agent conversation into the main
  content. The work runs autonomously, on his word.
- **Classification:** feature, Marley layout, Marley crate only.
- **Recall (§18.3):**
  - The knowledge ledger has nothing on hosting an Agent Panel thread elsewhere.
  - The rail already opens threads through the panel (`show_thread`, #616 and #617) and answers
    their permissions from the rail (#508). Both work on the panel's views, so a view moved to the
    center stays reachable to them.
  - PR-687 shapes the scenario: check the stand-in before typing into a thread.
  - The brain: "the manager talks to the person in Zed's Agent Panel over ACP" (#694). Nothing on
    a thread in the center.
- **Discovery:**
  - `agent_ui`: `AgentPanel::{active_conversation_view, active_thread_is_draft, clear_base_view,
    is_retained_thread, activate_retained_thread, load_agent_thread}`, and `AgentPanelEvent::
    ActiveViewChanged`.
  - `ConversationView::{title, parent_id, agent_key}` and `Focusable`; `ThreadViewItem`, Zed's
    test-only pattern.
  - `workspace::item::{Item, ItemEvent::CloseItem, tab_extra_context_menu_actions}`.
  - `rail.rs`: `show_thread` and the thread row's `right_click_menu`.

### Design
- **New `crates/marley_workbench/src/thread_tab.rs`** (Marley crate):
  - **Actions:** `actions!(marley, [OpenThreadInCenter, MoveThreadToPanel])`, registered on each
    new workspace from `init`.
  - **`ThreadTab`:** `conversation_view: Entity<ConversationView>`, `panel:
    WeakEntity<AgentPanel>`, `_subscriptions`.
    - `cx.observe(&conversation_view)` emits `ItemEvent::UpdateTab` and notifies.
    - `cx.subscribe(&panel)`: on `ActiveViewChanged`, if the panel's active view is this entity,
      it emits `ItemEvent::CloseItem`.
  - **`impl Item`:**
    - `type Event = ItemEvent`, and `to_item_events` passes the event through;
    - `tab_content_text` is the thread's title, and `tab_icon` `IconName::ZedAssistant`;
    - `tab_tooltip_text` is "Agent thread: <title>";
    - `tab_extra_context_menu_actions` is `[("Move Thread to Panel", MoveThreadToPanel)]`.
  - **`Focusable`** delegates to the view, and **`Render`** is `v_flex().size_full()` with the
    view.
  - **`open_in_center(workspace, window, cx)`:**
    - It refuses, through `show_toast`, when there is no panel, no active thread, or an empty
      draft (REQ-005).
    - A tab already showing that entity is activated.
    - Otherwise the panel `clear_base_view`s, the tab is added with `add_item_to_center`, and the
      view is focused.
  - **`move_to_panel(workspace, window, cx)`** works on the active center item as a `ThreadTab`:
    - when the panel retains its `parent_id`, it calls `activate_retained_thread(id, true)`, and
      the tab closes itself through its subscription;
    - otherwise it calls `load_agent_thread(agent_key, id, None, Some(title), true, …)` and closes
      the tab.

    Either way it then calls `focus_panel::<AgentPanel>`.
  - **`activate_for(workspace, thread_id, window, cx) -> bool`:** the rail's hook, which brings
    forward a tab showing that thread.
- **`rail.rs`:**
  - `show_thread` first tries `thread_tab::activate_for` and returns when a tab came forward;
  - the thread row menu gains "Open in Center", which opens the thread as a click does and then
    runs `open_in_center` in that workspace.
- **`marley_workbench.rs`:** `pub mod thread_tab;` and its `init`.
- **The ledger:** no Zed path is touched.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Marley entry on the scripted agent (`agent: claude_code`, fake signed-in `claude`); opens a Marley thread from the rail's + → New Agent Thread; checks the agent log's `session/new` (PR-687); types "hello", Return; runs `marley: open thread in center` from the palette | 697-01-in-center: a center tab with the thread ("Noted: hello"), the panel on a new draft |
| 002 | Types "again", Return, in the tab | 697-02-typed: "Noted: again" in the tab, and the agent log holds the prompt |
| 003 | Runs `marley: move thread to panel` | 697-03-back: the thread in the panel, no thread tab in the center |
| 004 | The rail's thread row, right-click → Open in Center | 697-04-from-rail: the thread in a center tab again |
| 005 | — | Review: the draft check comes before `clear_base_view` |

### Risks
- **An evicted thread.** An idle thread that can be loaded may be evicted from the panel's
  retained threads while it sits in a tab. Moving it back then loads it again from history, as a
  new view, so a turn running in the old view is not carried over. Running threads are never
  evicted.
- **Opening it twice.** If the panel's history opens the same thread while it is in a tab, the tab
  closes when the panel shows that same entity. A history load of an evicted one makes a second
  view, which closing the tab ends.
- **Tab right-click actions.** These dispatch to the focused element. The workspace-level handler
  acts on the active center item, so the tab must be the active item; it is, once right-clicked.

## Phase 2 — Code
- **Built:**
  - `crates/marley_workbench/src/thread_tab.rs`:
    - the actions `OpenThreadInCenter` and `MoveThreadToPanel`;
    - `ThreadTab`, an `Item` hosting the panel's `ConversationView`. Its tab shows the title and
      `ZedAssistant`, its right-click menu has "Move Thread to Panel", and it closes itself on
      `ActiveViewChanged` when the panel shows its view;
    - `open_in_center`, `open_thread_in_center`, `move_to_panel` and `activate_for`.
  - `rail.rs`:
    - `show_thread` brings an open thread tab forward first;
    - `open_listed_thread(key, center)` and `open_thread_with(…, center, …)`;
    - the thread row's menu gains "Open in Center".
  - `marley_workbench.rs`: the module and its `init`.
  - `docs/marley/guide.md`: "An agent thread in a center tab".
- **Deviations:** the tab holds no panel handle. Its subscription to the panel is enough, and
  `move_to_panel` looks the panel up on the workspace.
- **Review of the diff:**
  - The draft check comes before `clear_base_view` (REQ-005).
  - `activate_for` runs before the move, so a thread is never in two tabs.
  - The panel is updated from the workspace's context, never from inside its own update.
  - Clippy's three findings were fixed: `Eq` on the actions, `init(cx: &App)`, and a manual
    `Debug` for the tab.
- **Gate (`697-gate-1.log`):** GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/697-an-agent-thread-in-a-center-tab.sh`, under `compositor sway`, on
  the debug build. The Marley entry runs the scripted ACP agent named by
  `MARLEY_ASSISTANT_ADAPTER`, which answers each prompt "Noted: <prompt>" and logs what it reads.
  The rail row's guess (130, 175) held: once the tab closes, the thread row sits there.
- **First run (red, REQ-002):** 697-01 was right, but "again" typed after the move reached
  nothing: neither the tab's message editor nor the agent log held it. The tab focused the
  `ConversationView`'s handle, which is the `ThreadView`'s own and takes no typing; the Agent
  Panel focuses the thread's message editor (`activation_focus_handle`, `pub(crate)` in
  `agent_ui`).
- **Fix:** `ThreadTab`'s `focus_handle` is the active thread's `message_editor` (a `pub` field),
  the view's own handle when there is no thread; `open_in_center` focuses the tab's handle. Pane
  activation and `activate_for` get the editor through the same `Focusable`.
- **Gate after the fix:** `697-gate-2.log` was RED on clippy (`option_if_let_else` on the new
  `match`); rewritten as `map_or_else`, `697-gate-3.log` is GATE GREEN [diff], 17 passed.
- **Final run (shots-697b): every check passes.**
  - **697-01-in-center (REQ-001):** a center tab "hello" beside "repo — bash" holds the thread
    ("hello", "Noted: hello"); the panel shows "New Marley Thread" with an empty message box.
  - **697-02-typed (REQ-002):** the tab shows "again" and "Noted: again" under the first turn, and
    the check "the tab's message reached the agent" passes on the agent log.
  - **697-03-back (REQ-003):** the panel's title is "hello" with both turns; the center holds only
    "repo — bash"; the rail lists the terminal and "hello · Marley · idle".
  - **697-04-from-rail (REQ-004):** after the thread row's right-click → Open in Center, the
    thread is a center tab again with both turns, and the panel is on a new draft.
  - **REQ-005:** the review of the diff (the draft check comes before `clear_base_view`).
  - The run reports Chad's Hyprland untouched: one Marley window before and after, no rule added.
- **Seen, not in scope:** while the thread is in a tab the rail lists it twice, as the tab's row
  ("hello", the tab's icon) and as the thread row ("hello · Marley · idle"), since the rail lists
  center tabs and the panel's threads alike. Worth asking Chad whether the thread row should hide
  while its thread is in a tab.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: An agent thread in a center tab); the architecture note
  `docs/marley_architecture/marley_workbench.md` ("An agent thread in a center tab"); the slice
  status in `docs/marley/workbench-shell.md`; the user's guide section came with Phase 2. No Zed
  path is touched, so no touchpoint row.
- **Knowledge appended:** F-claude-697-a-thread-in-a-center-tab-took-no-typing-001 and
  PR-claude-697-a-hosted-view-focuses-where-its-own-host-does-001.
- **Brain:** this repository's sessions have no `rusty` MCP server, so no brain loop ran; nothing
  was recorded there, as for #696.
- **Ticket:** closed (`tickets/closed/`); BACKLOG had no row left for it.
- **Gate:** `697-gate-3.log`, GATE GREEN [diff], 17 passed, on the tree committed.
