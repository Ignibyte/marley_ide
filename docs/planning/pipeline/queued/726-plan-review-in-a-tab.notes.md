# Plan review in a center tab, slice 1 — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-726-plan-review-in-a-tab.md
- **Pipeline spec:** 726-plan-review-in-a-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 1 (`docs/planning/intake/plannotator-findings.md`): "Catch
  Claude Code's `ExitPlanMode` (a `PermissionRequest` hook in the shared plugin, which needs a
  request to rustal-harness) and Codex's plan from the App Server (#650), and show the plan as
  rendered Markdown in a center tab: select text to comment, delete or label it, then Approve or
  Send Feedback. The plan also waits in Needs you." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "10.) yes". Everything opens in a
  center tab (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`). It waits under
  Deliberate with the rest of the review batch until after the finish.
- **Classification:** feature, large in all; this slice is medium to large. One Zed touch
  (`crates/markdown`), the settings file's row; the rest in Marley crates. It waits on one
  rustal-harness change (below).

### The slices
| Slice | What | Needs |
|---|---|---|
| 1 (this spec) | Claude Code's plan from a terminal: the switch, `marley-agent approve`, the tab, comments on passages and an overall note, Approve and Send Feedback, Needs you | the harness request |
| 2 | Delete and label marks, "Looks good", editing a note | slice 1 |
| 3 | A plan in a Zed agent thread (Claude Code over ACP): the permission request's plan in the tab; Approve picks its allow option; Send Feedback is "No, keep planning" and a post through the thread's own path (AD-claude-706-agents-post-and-answer-through-the-panels-own-paths-001) | slice 1 |
| 4 | Codex's plan: read from the App Server (`item/completed` plan items or `turn/plan/updated`); the TUI's "Implement this plan?" is not a server request, so the slice starts by deciding how an approval reaches Codex (AD-claude-650 rejects Marley starting turns) | slice 1, a decision |

Slices 2 to 4 need ticket numbers from the backlog's owner; #727 (versions) and #728 (question
cards) build on slice 1 and can come before slices 2 to 4.

### The harness request
For rustal-harness's shared Claude Code plugin (`crates/harness-runtime/src/claude/plugin/`,
`hooks/register.js`, `tool.check`):

> Under Marley's host (`MARLEY_BIN` and `MARLEY_TERMINAL_ID` set, no `RH_BIN`), when the verdict is
> `ask` and the call's tool is named in `MARLEY_APPROVE_TOOLS` (a comma-separated list of tool
> names), run `$MARLEY_BIN approve --source mod:claude-code --call <tool_use_id> --tool <tool>`
> with the tool's input as JSON on stdin and `DECISION` as the timeout, exactly as the harness
> path runs `rh approve`. Report `waiting` with the call as the activity before the run, as the
> Marley path does now. Map the answer as the harness path does: `approve` allows, `deny` denies
> with the reason, `cancel` denies and ends the turn. If the run fails, times out or exits non-zero,
> or stdout holds no decision, return the verdict unchanged, so Claude Code's own dialog decides.
> Calls whose tool the variable does not name keep the current Marley path.

Marley then takes the plugin at the revision that ships it, as #724 did (`shared_plugin::FILES`,
the digest). Until then the scenario's stand-in plays the plugin's part.

- **Recall (§18.3):**
  - AD-claude-709-the-shared-plugin-is-carried-by-digest-and-off-by-default-001: Marley carries the
    harness's plugin unchanged; a change to it is the harness's.
  - AD-claude-652-an-agents-report-outranks-its-hook-frames-001: `MARLEY_BIN` and the agent socket;
    a request is known by its process chain reaching a local terminal's shell and the terminal's
    id; the reporter holds the terminal.
  - AD-claude-508-one-inbox-lists-every-agent-that-waits-on-the-user-001: answer in place only what
    needs no further choice; "answering in place waits for a hook that waits on Marley (the second
    slice in the notes)": this slice is that hook, for plans.
  - AD-claude-651-codex-approvals-answer-in-the-inbox-only-when-clicked-001: an answer goes once,
    only to the request shown; the model for D7.
  - AD-claude-645-a-brain-page-is-drawn-by-zeds-markdown-after-marleys-wikilink-pass-001: Zed's
    `markdown` crate draws Marley's documents; a link click is deferred out of the entity's update.
  - PR-claude-async-answer-carries-question-identity-every-hop-001: D7.
  - F-claude-592-the-launch-approval-was-rendered-as-markdown-001: D8.
  - PR-claude-703-an-unactivated-insert-keeps-the-active-tab-001: the tab added beside the terminal
    without activation keeps the pane's active item.
  - PR-claude-check-a-unix-socket-path-against-sun-path-001: the socket exists already; nothing new.
  - The finish-line note: "permission-mode presets" were skipped ("each CLI keeps its own
    prompts"), which D2 respects.
- **Discovery:**
  - `crates/marley_workbench/claude_shared_plugin/hooks/register.js`: host choice (133-146),
    `tool.check` (256-276; Marley path 259-265, harness path 266-275), `DECISION` (33),
    `tool.call` (279-285).
  - `crates/marley_workbench/src/shared_plugin.rs`: `FILES` (23), `SharedPlugin::of` (102-114),
    `wanted` (124), `reconcile` (130-153).
  - `crates/marley_workbench/bin/marley-agent`: `DEADLINES` (22), the parser (52-72), `send`
    (113-130), the exits (144-150).
  - `crates/marley_mcp/src/agent_socket.rs`: `MAX_LINE` (20), `READ_TIMEOUT` (23),
    `ANSWER_TIMEOUT` (26), `AgentRequest` (30-45), `serve` (145-147).
  - `crates/marley_workbench/src/agent_reports.rs`: `start` (70-124), `take` (180-194),
    `terminal_of` (198-214), `accept` (240-321).
  - `crates/marley_agent/src/report.rs`: `Refusal` (124-170), `Request` (373-390), `answer_ok`
    (394), `answer_refused` (400).
  - `crates/marley_terminal/src/identity.rs`: `MARLEY_BIN` (39), `agent_environment` (67-84),
    called from `crates/terminal/src/terminal.rs:1356`.
  - `crates/markdown/src/markdown.rs`: `Selection` (1438-1445), the field (489), `has_selection`
    (1103), `selected_source` (1107), `set_search_highlights` (1114), the clear on a click outside
    (2434-2438), `context_menu_selected_text` (1208); `parser.rs:13-23` (options, no directives).
  - `crates/marley_workbench/src/rusty/page.rs`: the entity (338-351), `markdown_element`
    (1545-1569), `render_read` (1571-1603).
  - `crates/marley_rail/src/marley_rail.rs`: `InboxKind` (457), `InboxAnswer` (472-517),
    `InboxEntry` (521-546).
  - `crates/marley_workbench/src/rail.rs`: `InboxTarget` (796-823), `observe_marks` (829-840),
    `open_inbox_entry` (1948), `render_inbox_entry` (2065, icons 2067-2081), `answer_inbox` (2255),
    `inbox_entries` (6688-6759), `codex_entry` (7085-7128), `build_snapshot` (8125).
  - `crates/marley_workbench/src/threads_page.rs`: `init` (37-45), `open` (47-60), `Item` (437-457).
  - `crates/settings_content/src/marley.rs`: `claude_code_shared_plugin` (215) as the model for the
    switch; `crates/settings_ui/src/marley_page.rs`: its row (500).
  - Claude adapter (outside the tree, read for comparison): `claude-agent-acp` 0.89.1,
    `tool-calls/reporters/interaction.js:14-25`, `permissions/options/tools.js:42-98`,
    `permissions/effects.js:115`.
- **Decisions:** D1 to D8 in the spec.

### Design
- **Zed touch, `crates/markdown/src/markdown.rs`:** `pub fn selected_range(&self) ->
  Option<Range<usize>>` beside `selected_source`, `// Marley:`.
- **Settings** (`crates/settings_content/src/marley.rs`, its row): `plan_review: Option<bool>`;
  `MarleySettings.plan_review: bool`; the Settings window row (`crates/settings_ui/src/marley_page.rs`,
  its row), whose text says it needs the shared plugin.
- **`crates/marley_terminal/src/identity.rs`** (Marley crate): `MARLEY_APPROVE_TOOLS` set to
  `ExitPlanMode` by `agent_environment` when Marley says plan review is on (a flag beside the
  program, set from the workbench as `set_agent_program` is).
- **`crates/marley_agent/src/report.rs`** (Marley crate): `Request::Approve { terminal, source,
  call, tool, input }`, `answer_decision(decision, reason)`, refusals for an unknown tool and a
  terminal not held.
- **`crates/marley_mcp/src/agent_socket.rs`** (Marley crate): the answer's deadline per request
  (the handler says it; reports keep 3.5 s, an approve takes up to the program's deadline), and the
  connection's end seen by the handler so a withdrawn request is known.
- **`crates/marley_workbench/bin/marley-agent`:** `approve` with `rh approve`'s arguments;
  `DEADLINES["approve"]` under the plugin's 600 s (570 s); stdout the answer's JSON; exit 1 with
  nothing on stdout on a refusal or the deadline.
- **`crates/marley_workbench/src/agent_reports.rs`** (Marley crate): `take` hands an `Approve` for
  `ExitPlanMode`, from the terminal's holder, to `plan_review::arrived`, keeping the request's
  answer sender; any other tool is refused.
- **New `crates/marley_workbench/src/plan_review.rs`** (Marley crate):
  - `PendingPlans` global: per terminal, the request (call id, a nonce, the answer sender), the
    plan text, the project; observed by the rail.
  - `PlanReview` item: a `Markdown` entity (`parse_html: false`), the notes (source range, quote,
    words), the overall note, the state (waiting, approved, feedback sent, withdrawn); the tab
    "Plan · <project>"; `Render` with the plan at left and the notes column at right; the key
    context `PlanReview` with `c` bound to `plan_review::Comment`; the right-click menu's
    Comment…; highlights through `set_search_highlights`.
  - `arrived`: find or make the tab for the terminal's session (same tab while the session is the
    same), set the plan, clear the notes, add it beside the terminal (activated only if the
    terminal has the focus).
  - `answer`: checks the nonce and that the request is still there, sends the JSON, marks the tab;
    `withdrawn`: marks the tab and drops the entry.
- **`crates/marley_agent/src/marley_agent.rs`** (Marley crate): `PlanNote { quote, comment }` and
  `plan_feedback(notes, overall) -> String` (D6).
- **The inbox:** `InboxKind::Plan` (`marley_rail`); `plan_entries` from `PendingPlans` in
  `inbox_entries`; `InboxTarget::Plan`; `open_inbox_entry` brings the tab forward; the entry's
  icon; `PendingPlans` in `observe_marks`.
- **`marley_workbench.rs`:** the module, its `init`, the action.
- **The guide:** "Plan review".
- **File manifest:** `crates/markdown/src/markdown.rs`, `crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs` (Zed crate paths); `marley_terminal/src/identity.rs`,
  `marley_agent/src/report.rs`, `marley_agent/src/marley_agent.rs`, `marley_mcp/src/agent_socket.rs`,
  `marley_rail/src/marley_rail.rs`, `marley_workbench/bin/marley-agent`,
  `marley_workbench/src/agent_reports.rs`, `marley_workbench/src/plan_review.rs` (new),
  `marley_workbench/src/rail.rs`, `marley_workbench/src/marley_workbench.rs` (Marley crates); the
  guide; the scenario.
- **Ledger rows (`docs/marley/zed-touchpoints.md`):** the row for `crates/markdown/src/markdown.rs`
  (#530's `code_block_action`, line 91) gains `selected_range` (#726; why: a note anchors on the
  selected source range; on merge: keep it, or move to an upstream accessor if one appears); the
  rows for `settings_content/src/marley.rs` and `settings_ui/src/marley_page.rs` gain
  `plan_review` (#726).

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Turns `marley.plan_review` on in the run's settings; in the terminal the stand-in prints the variable | 726-01-env |
| 003, 004 | The stand-in reports idle, then runs `approve` with the plan | 726-02-tab: the tab beside the terminal; Needs you's entry |
| 005 | Drags over a passage, presses `c`, types, Enter; types an overall note | 726-03-note: the note and its highlight |
| 006 | Clicks Send Feedback | 726-04-feedback: the terminal's printed deny and reason; no entry |
| 007 | The stand-in sends the revised plan | 726-05-revised: the same tab, the new plan, no notes |
| 008 | Clicks Approve | 726-06-approved: approve and exit 0; the tab says Approved |
| 002 | Turns the switch off; opens a new terminal; the stand-in prints the variable | 726-07-off |
| 009, 010 | (review of the diff: the withdrawn path; the program's deadline) | — |

What no scenario reaches: the real Claude Code's call, until the harness ships its change; then
one run in plan mode, recorded in Phase 3.

### Risks
- Whether the plugin's `tool.check` sees `ExitPlanMode` with the verdict `ask` is the harness's to
  confirm; plannotator's `PermissionRequest` matcher on `ExitPlanMode` suggests the dialog is a
  permission prompt.
- `ExitPlanMode`'s input names the plan `plan`; a Claude Code that sends only a plan file's path
  needs the file read (the adapter has such a case for one client). The Plan phase checks the
  version in use.
- A plan over the socket's 64 KiB line is refused; plans are far smaller, and the refusal leaves
  Claude Code's dialog in charge.
- Selection by drag in the headless sway: #645's scenarios drove the same element; the shot shows
  the highlight, which proves the range was read.
- An answer that crosses a withdrawn request: the nonce and the request's presence are checked on
  the main thread before the send (D7).
