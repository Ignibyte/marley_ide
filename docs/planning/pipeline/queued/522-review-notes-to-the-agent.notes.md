# Review notes to the agent: Zed's review comments pasted into an idle agent terminal, and kept — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-522-review-notes-to-the-agent.md
- **Pipeline spec:** 522-review-notes-to-the-agent.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey Chad asked for on 2026-09-25 ("we will be taking what it does
  well and bring it in here"): its item 11, review notes back to the agent, one of the new tickets
  the survey implies (`docs/orca_architecture/README.md`; report 02 §2.9 and §3 item 5). The lead's
  brief: Zed's diff views draw "Send Review to Agent (N)" (`crates/git_ui/src/project_diff.rs`) and
  nothing handles `editor::SendReviewToAgent`; the comments are `StoredReviewComment` in
  `crates/editor/src/git.rs`, with `take_all_review_comments` `pub(super)`; a handler in Marley
  pastes the notes into a chosen agent terminal only while it is idle, in Orca's
  `File:`/`Line:`/`User comment:` format, keeping the notes marked sent.
- **Classification / tier:** feature, prong 2, one slice (S to M). Zed touches in
  `crates/feature_flags` (one method) and `crates/editor` (a field, a count, two public methods, a
  label, an export); the rest in `marley_agent` and `marley_workbench`.
- **Recall (§18.3):**
  - AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001: the plugin's
    `Notification` (`permission_prompt`, `idle_prompt`) and `Stop` hooks answer with an OSC 777
    notify with fixed messages, only where `TERM_PROGRAM` is `zed`; `terminalSequence` is in
    Claude Code's schema but not its docs, so a Claude Code upgrade can silence it. D6 rests on
    those three messages, and a silent plugin reads as "no idle signal", which fails safe.
  - The bracketed-paste injection lesson (`docs/planning/knowledge/lessons.md:811`): a single
    `replace` of the end marker can splice a new one. Zed's `Terminal::paste` removes every ESC
    before it brackets, so no end marker survives; D7 uses it and adds no paste code.
  - BF-claude-specified-keybinding-cmd-r-unimplemented-only-click-shipped: a behavior with two
    triggers ships both. The toolbar button and the palette's `editor: send review to agent` both
    dispatch the one action, and the scenario uses the button; P2's review checks the palette path.
  - #481 (rich input): the send path this ticket reuses; #496: a pick's Send shows the terminal it
    typed into, the model for D7's last step.
  - Brain: no consultation run by this drafting agent; the Planner's `brain_ask` at promotion is
    owed.
- **Discovery (checked in the tree):**
  - `crates/editor/src/git.rs`: `DiffHunkKey` (162: `file_path: Arc<RelPath>`, `hunk_start_anchor`);
    `StoredReviewComment` (171: `id`, `comment`, `range: Range<Anchor>`, `is_editing`) and its `new`
    (218); `total_review_comment_count` (749); `add_review_comment` (757, emits
    `EditorEvent::ReviewCommentsChanged { total_count }`); `remove_review_comment` (1167);
    `render_comments_section` (2681) and `render_comment_row` (2748), whose display mode draws
    nothing on the right (`gpui::Empty`); `diff_review_line_range` (2900, anchors to buffer rows
    with `point_to_buffer_point`); `take_all_review_comments` (2918).
  - `crates/editor/src/editor.rs:118`: `pub(crate) use git::{DiffHunkKey, StoredReviewComment};`
    and the storage at 1168.
  - `crates/editor/src/split.rs:558`: `SplittableEditor::rhs_editor`, where the comments live (the
    removed upstream handler read them there).
  - `crates/editor/src/actions.rs:929`: `SendReviewToAgent`, "Sends all stored review comments to
    the Agent panel"; no key binds it (`assets/keymaps` grepped).
  - `crates/git_ui/src/project_diff.rs`: `total_review_comment_count` (329), `editor` (334), the
    toolbar reads the count (818) and draws the button while it is above zero (962 to 970,
    `render_send_review_to_agent_button` 974 to 990), dispatching through `dispatch_action` (747:
    focus the diff, then `cx.dispatch_action` deferred), so a workspace action receives it.
  - `crates/git_ui/src/branch_diff.rs`: `editor` (435), the button (898 to 905).
  - `crates/git_ui/src/diff_multibuffer.rs`: `set_show_diff_review_button(true)` (89), the count
    cached from `ReviewCommentsChanged` (98 to 99), `total_review_comment_count` (293).
  - `crates/feature_flags/src/flags.rs:30-40`: `DiffReviewFeatureFlag`, `"diff-review"`,
    `enabled_for_staff` false; `crates/feature_flags/src/store.rs`: `overrides_enabled` (105 to
    107, `cfg!(debug_assertions) || staff`), `try_flag_value` (164, `enabled_for_all` first);
    `crates/feature_flags/src/feature_flags.rs:130`, `enabled_for_all`'s default. The flag's checks:
    `crates/editor/src/element.rs:2742` (the button), `crates/editor/src/element/mouse.rs:116` (the
    drag).
  - The removed handler: `git show a5e6964186^:crates/agent_ui/src/text_thread_editor.rs`, 1524 to
    1650, `handle_send_review_to_agent`: find the `ProjectDiff`, take every comment from its
    `rhs_editor`, open or reuse an Agent Panel thread, insert one crease per code range with its
    comments.
  - `crates/terminal/src/terminal.rs:2582`, `Terminal::paste`.
  - `crates/marley_workbench/src/rich_input.rs`: `send` (97 to 116).
  - `crates/marley_workbench/src/notifications.rs:28-45`: `init` subscribes every terminal view to
    its terminal's `Event::MarleyNotification { title, body }`.
  - `crates/marley_workbench/claude_plugin/marley/hooks/hooks.json` (`Notification` with
    `permission_prompt` and `idle_prompt`, `Stop`) and `hooks/notify.sh` (`needs your permission`,
    `is waiting for you`, `finished`, after the project's name, under the title `Claude Code`).
  - `crates/marley_workbench/src/agent_bar.rs:129`, `agent_in`; `crates/marley_agent/src/marley_agent.rs:117`,
    `WAITING_AFTER`, and 121, `agent_status`.
  - `crates/marley_workbench/src/rail.rs:429`: the rail marks output on `terminal::Event::Wakeup`,
    the event the readiness tracker counts too.
  - Orca (MIT, read): `src/shared/diff-comments-format.ts` (the whole format, 34 lines);
    `src/renderer/src/lib/active-agent-note-send-delivery.ts` (`sendPromptWithGuardedPasteAndEnter`).
- **Decisions:** D1 to D9 in the spec.

### Design
- **Approach.**
  - *Zed touch 1, `crates/feature_flags/src/flags.rs`:* `fn enabled_for_all() -> bool { true }` on
    `DiffReviewFeatureFlag`, with `// Marley:`.
  - *Zed touch 2, `crates/editor/src/git.rs`:* `StoredReviewComment.sent: bool` (false in `new`);
    `total_review_comment_count` counts `!sent`; `pub struct ReviewNote { pub id: usize, pub path:
    Arc<RelPath>, pub abs_path: Option<PathBuf>, pub first_line: u32, pub last_line: u32, pub
    comment: String }` (lines from 1, from the anchors as `diff_review_line_range` resolves them,
    the absolute path from the buffer's file when it is local); `pub fn unsent_review_notes(&self,
    cx: &App) -> Vec<ReviewNote>` in file then line order; `pub fn mark_review_notes_sent(&mut self,
    ids: &[usize], cx: &mut Context<Self>)`, which sets `sent`, emits `ReviewCommentsChanged` with
    the new count and notifies; `render_comment_row` draws a muted `Sent` label where display mode
    draws nothing. Every hunk carries `// Marley:`.
  - *Zed touch 3, `crates/editor/src/editor.rs:118`:* `pub use git::ReviewNote;` beside the
    `pub(crate)` export.
  - *`marley_agent`:* `ReviewNote`-shaped plain input (`path: String, first_line, last_line,
    comment`) and `review_prompt(notes) -> String`, Orca's format.
  - *`marley_workbench/src/agent_state.rs` (new):* a global map from terminal view to its last
    output (`terminal::Event::Wakeup`) and its last plugin notification (`Event::MarleyNotification`
    titled `Claude Code`, classified by the three suffixes), filled from `observe_new` as
    `notifications::init` does, and `readiness(view, now) -> Readiness { Ready, Working,
    AskingPermission, NoSignal(AgentKind) }`.
  - *`marley_workbench/src/review_notes.rs` (new):* the `SendReviewToAgent` workspace action: the
    active `ProjectDiff` or `BranchDiff`, its `rhs_editor`, `unsent_review_notes`; a toast when none
    are unsent; the tree's root (the diff's repository work directory); the candidates (every
    `TerminalView` of the workspace's project group whose `agent_in` is `Some` and whose working
    directory lies under the root); the picker (a `Picker` like `agents::NewAgentPicker`, each
    entry's state from `readiness`, a Copy notes entry at the foot); on confirm: readiness checked
    again, `terminal.paste(prompt)`, `terminal.input(b"\r")`, `mark_review_notes_sent`, the
    terminal shown with the rail's activation path; Copy writes `ClipboardItem::new_string`.
    Errors reach a toast or `detach_and_prompt_err`.
  - *`marley_workbench/src/claude_plugin.rs`:* the three messages and the title as constants, and a
    comment in `notify.sh` naming them.
  - *`marley_workbench/Cargo.toml`:* `git_ui` (for `ProjectDiff` and `BranchDiff`).
- **File manifest.** Zed crates: `crates/feature_flags/src/flags.rs`, `crates/editor/src/git.rs`,
  `crates/editor/src/editor.rs`. Marley crates: `crates/marley_agent/src/marley_agent.rs`,
  `crates/marley_workbench/src/agent_state.rs` (new), `crates/marley_workbench/src/review_notes.rs`
  (new), `crates/marley_workbench/src/claude_plugin.rs`,
  `crates/marley_workbench/claude_plugin/marley/hooks/notify.sh` (a comment only; the plugin's
  version stays), `crates/marley_workbench/src/marley_workbench.rs` (modules, `init`),
  `crates/marley_workbench/Cargo.toml`. Test phase: `script/e2e/522-review-notes-to-the-agent.sh`.
- **Ledger rows (`docs/marley/zed-touchpoints.md`, before the edits):**
  - `crates/feature_flags/src/flags.rs`: `DiffReviewFeatureFlag::enabled_for_all` returns true
    (#522). Why: review comments are the review loop's input and the flag keeps them from every
    non-staff build. On merge: keep the method; drop the row when upstream ships the flag on.
  - `crates/editor/src/git.rs`: `StoredReviewComment::sent`, the count of unsent notes,
    `ReviewNote`, `unsent_review_notes`, `mark_review_notes_sent`, and the Sent label in
    `render_comment_row` (#522). Why: Marley sends notes to a terminal agent and keeps them, so the
    fix can be checked against them. On merge: keep the field, the methods and the label; if
    upstream brings back a handler or a public read of the comments, move Marley onto it.
  - `crates/editor/src/editor.rs`: `pub use git::ReviewNote` (#522). Why: the public read's type.
    On merge: keep the line.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: `notes.txt` committed, its third line changed and not staged; steps: `git: diff` from the palette, the pointer on the changed line's gutter, click Add Review, type `Use the new label here`, Enter | `522-01-comment` |
| REQ-002 | setup: the `.bashrc` stand-ins; steps: four terminals from the rail's New Terminal running `ready`, `busy`, `asking` and `codexish`; back to the diff, click Send Review to Agent (1) | `522-02-picker` |
| REQ-003 | steps: select the ready agent, Enter | `522-03-sent` |
| REQ-004 | steps: click the diff's tab | `522-04-marked` |
| REQ-005 | the picker of REQ-002: `busy` reads working, `asking` asking for permission, `codexish` no idle signal, none of them choosable (Enter on one sends nothing, which the stand-ins' logs confirm) | `522-02-picker`; the run log |
| REQ-006 | steps: a second note on another changed line, Send Review, Copy notes; a plain terminal from the rail, Ctrl+Shift+V | `522-05-copied` |
| REQ-007 | none reachable in the same run without closing the agents; P2's review of the empty state, and P3 may add a run with no agent terminal | review |

The run log keeps the ready stand-in's received lines (it echoes them to a file under
`$E2E_WORK` too), so the format is checked byte for byte against D4, escapes included.

### Risks
- D6 assumes Claude Code redraws while a turn runs (its spinner and elapsed time), so two quiet
  seconds mean the turn is over. P3 checks it once with the real Claude Code during a long tool
  call (a `sleep 20` it is asked to run) and records what the tracker read; if the terminal goes
  quiet mid-turn, D6 also requires the last notification to be newer than the last delivery
  Marley made.
- A draft Chad left in Claude Code's input box is submitted with the notes: the paste appends to
  it. The picker's ready line says "sends and submits"; Chad can Copy instead.
- The plugin's messages carry the project's name first; a project named so that its name ends
  with one of the suffixes is still read by the suffix alone, so the classification holds.
- Zed's `take_all_review_comments` resets the ids. Nothing in the tree calls it outside the
  editor's tests, and Marley never does, so ids stay unique for the editor's life.
- The `diff-review` flag also turns on the drag to select lines (`element/mouse.rs:116`); that
  is the same feature, wanted.
