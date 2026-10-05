# Rusty's Decisions tab, with System One's log renamed System One calls: Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-659-rusty-decisions-tab.md
- **Pipeline spec:** 659-rusty-decisions-tab.spec.md

## Phase 1: Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty in Marley: "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley", then "Plan it now". The plan's open decision
  4 asked whether to rename Marley's System One "Decisions" tab (for example "System One calls")
  so Rusty's decisions keep the name; the plan records Chad's answer the same day: "Marley's System
  One log becomes "System One calls"; Rusty's tab keeps "Decisions". The rename rides with R7."
  On 2026-10-03 the first batch (#643 to #647) came from "lets make a plan to begin the work and
  spec out the tickets" and "lets make sure we use the gpui components we found here"; this ticket
  is in the follow-up batch, #654 to #659, which carries on from the same request. Plan:
  `docs/marley/rusty-in-marley.md`, R-D3's DecisionsPage row, R-D9 (the fixed row: Today, Graph,
  Tasks, Decisions, Memory, Skills, Secrets), open decision 4 and the slices table's R7, whose
  Decisions half this is.
- **Classification / tier:** feature, medium, in two parts. The rename is wide and shallow: one
  file moved and six edited in three Marley crates and two of Marley's files in Zed crates, the
  guide page, four scenarios and the golden list, then six docs at Complete. The tab is one new
  view in `marley_workbench::rusty`, one pure module in `marley_rusty`, an entry in #644's row, and
  `brain_due` in #643's stand-in. Zed touches: two existing `zed-touchpoints.md` rows widened
  (`settings_ui/src/marley_page.rs`, `settings_content/src/marley.rs`), no new row, no new
  dependency, no new asset.
- **Recall (§18.3):**
  - AD-claude-565-the-system-one-layer-is-a-pure-core-behind-an-adapter-off-by-default-001: the
    Decisions view is the layer's log of calls; the rename changes names, not behavior.
  - F-claude-565-the-check-read-its-workspace-inside-that-workspaces-update-001: an action
    registered on the workspace runs inside its update. Both openers use the `&mut Workspace` they
    are handed, as `decisions::open` does, and read no workspace handle (D4).
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: a center tab
    found again by its opener, reading only while it is its pane's active item, not a
    `SerializableItem`. D9 and D13 follow it, as #647's D9 does.
  - AD-claude-599-the-guide-is-one-page-in-the-crate-kept-by-each-ticket-001: each ticket keeps its
    article; this one renames one and adds one, in the Code phase (the receipt binds the page).
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001: the fixed row's entry
    dispatches through the window; the settings link already dispatches through the original
    window (`marley_page.rs:1103-1108`).
  - PR-claude-rename-sweeps-intradoc-links-001: a renamed item's intra-doc links break only at
    rustdoc. Grepped: no `[`DecisionsView`]`, `[`decisions::...`]` or `[`OpenDecisions`]` link in
    the tree today; the Code phase greps again after the rename.
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001: element ids must be unique;
    a due decision is drawn in both sections, so its ids carry the section (D6).
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001: the scenario
    sets `marley.rusty` itself and names its stand-in in `MARLEY_RUSTY_MCP`.
  - L-claude-633-the-mcp-servers-page-opens-by-a-keymap-in-the-runs-profile-001: a scenario binds a
    key in `$E2E_PROFILE/config/keymap.json`; here, to the old id.
  - L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001: under sway the
    Settings window tiles beside Marley's and halves it; shots 659-04 and 659-05 are read with that.
  - Completed pipelines: 565 (the view, its scenario's palette steps and its coordinates), 599 (the
    guide page), 609 (`AgentView`), 633 (a stand-in `rusty-mcp`). Queued: #643 to #647, read for
    their names (below).
  - Brain (`rusty-cli brain search`, read only): the #565 decision
    (`decisions/marleys-system-one-layer-a-pure-core-behind-a-workbench-adapter-off-by-default-marley-565`)
    names "a Decisions view"; nothing on the rename, on aliases, or on a decisions screen in Marley.
    Promotion asks the brain (`brain ask`) the alias question; Complete records the decision.
- **Discovery** (read 2026-10-03, each line re-read at the source):
  - **System One's tab today:** `crates/marley_workbench/src/decisions.rs` (420 lines): module doc
    `:1-7`; `DecisionsView` `:24-37`; `open` `:51-60` (`items_of_type`, `activate_item`,
    `add_item_to_active_pane`); the log line "decisions: reading today's calls" `:79`; button ids
    `decisions-run-check`, `decisions-set-key`, `decisions-forget-key` `:239-251`; row id
    `decision-{id}` `:292`; `.id("decisions")` and `.key_context("Decisions")` `:364-365`; the
    heading `Label::new("Decisions")` `:381`; `impl Item` with `tab_content_text` "Decisions"
    `:414-420`. No `SerializableItem`, no `tab_icon`. `marley_workbench.rs`: `pub mod decisions;`
    `:41`; `OpenDecisions` and its doc in the `marley` actions `:320-323`. `system_one.rs`: the
    module doc `:9`; the import `:45`; comments `:69`, `:185`, `:288`, `:1010`; the registration
    `:296-298`; the strings "set {KEY_VARIABLE}, or Set Key in Decisions" `:384`, "no key: set
    MARLEY_SYSTEM_ONE_KEY, or Set Key in Decisions" `:831`, "System One logged the check in shadow
    mode. Open Decisions to see it." `:1236`. `find.rs:168` (a comment).
    `marley_system_one/src/reading.rs:143`, `:156` (doc comments). No in-tree test asserts any of
    these strings (grepped).
  - **The settings page** (`crates/settings_ui/src/marley_page.rs`, Marley's file in Zed's crate):
    the section's comment `:639-642`; six mode descriptions that say "in Decisions" (`:805` Stop
    Kind, `:837` Browser Find, `:869` Terminal Find, `:901` Stall Kind, `:1029` Running Error,
    `:1061` Typed Line); the `ActionLink` `:1090-1113`, title "Decisions", button "Open
    Decisions", `cx.build_action("marley::OpenDecisions", None)` `:1099`, dispatched on the
    original window, then the Settings window removed.
    `crates/settings_content/src/marley.rs:343`: `SystemOneMode::Shadow`'s doc, the schema's text.
    Ledger rows `docs/marley/zed-touchpoints.md:59` (settings_content) and `:63` (marley_page, "an
    Open Decisions link that dispatches `marley::OpenDecisions` by name (#565)").
  - **gpui's aliases:** `crates/gpui/src/action.rs:84-87` (the attribute's doc: old names "should
    not correspond to any actions that are registered", still invoke the action, and the keymap
    schema warns); `insert_action` `:293-333` (an alias in `by_name` and `all_names`, panics on a
    clash); `build_action` `:351-369`. `crates/settings/src/keymap_file.rs:611-618`, `:640-641`,
    `:690-691`; `crates/keymap_editor/src/keymap_editor.rs:1443`;
    `crates/command_palette/src/command_palette.rs:131` (`humanize_action_name(action.name())`).
    Uses: `editor/src/actions.rs:733`, `workspace/src/pane.rs:125`,
    `git_ui/src/project_diff.rs:66`, 26 more. The `marley` namespace is in Zed's test list
    (`crates/zed/src/zed.rs:5954`, `test_action_namespaces` `:5868`), so the alias needs no line.
  - **The scenarios:** each defines `palette()` (Ctrl+Shift+P, the text, Return; e.g.
    `565-system-one-layer.sh:176-182`). `565-system-one-layer.sh`: comments `:10`, `:18`; `echo
    "== Decisions"` and the palette `:233-236` (shot `565-02-decisions-view`, then a click at
    `$DECISION_X`, `$DECISION_Y`); `:306-309` (shot `565-08b-decisions-all`); the settings shots
    `:336-347` do not click the link. `566-stop-kind.sh`: comment `:19`, palette `:267` (shot
    `566-01-shadow`). `568-inbox-order-and-risk-chips.sh`: comment `:10`, `:427-430` (shot
    `568-03-decisions`). `573-english-at-the-prompt-second-stage.sh`: comment `:12`, `:198-201`
    (shot `decisions`). `script/e2e/golden:41`, `:47` (comments naming Decisions). No scenario
    greps for the log line, the toast or an element id (grepped).
  - **The docs that name the tab:** `docs/marley/guide.md` (12: `:37`, `:1256-1258`, `:1312`,
    `:1332`, `:1362`, `:1393`, `:1420`, `:1444`, `:1467`, `### Decisions` `:1471-1478`, `:1497`);
    `docs/marley/walkthrough.md` (§10.1 `:1379-1386`, `:1447`, the palette table `:1598`);
    `docs/marley_architecture/marley_workbench.md` (`:1705`, `:1714`, `:1755-1760`, `:2590`);
    `docs/marley/three-prong-plan.md:266` (the S1 row); `crates/marley_workbench/guide/index.html`
    (the contents `:243`, Turn it on `:1433-1434`, `<article id="decisions">` `:1440-1449`, the
    palette table `:1657`). `CHANGELOG.md` and the archives keep the old name as history.
  - **Rusty's loop** (`/srv/stacks/rusty-v3`, MIT): tools in `crates/rusty-mcp/src/main.rs`:
    `brain_ask` `:1116-1132`, `brain_decide` `:1134-1151`, `brain_follow_up` `:1153-1168` (through
    `mutate` `:734-742`, which emits `DataChanged`, so `list_changed` follows), `brain_no_decision`
    `:1170-1183`, `brain_due` `:1185-1190` (`days` defaults to 0); params `AskParams` `:345-353`,
    `DecideParams` `:355-375`, `FollowUpParams` `:377-392`, `DueParams` `:403-409`;
    `json_result` `:36-41` (pretty JSON in one text block; a failure is a JSON-RPC
    `internal_error`, not an `isError` result). `crates/rusty-core/src/brain/decisions.rs`:
    `FOLLOW_UP_STATUSES` `:13`; `DecisionSummary { slug, title, question, status, decided,
    follow_up_by, overdue }` `:17-28`; `Due { due, all }` `:70-76`; `date_after` (local)
    `:90-94`; `decide` `:205-299` (writes `status: decided`, `decided` from `today_iso`,
    `consulted`, `supersedes`, and marks the old one superseded); `follow_up` `:302-372` (status in
    the three, a non-empty outcome, an ISO date, superseded needs a successor that is a decision;
    appends `### Follow-up DATE: status`, sets `status`, sets or removes `follow_up_by`, sets
    `superseded_by`, adds a timeline entry); `decision_summary` `:384-413` (an empty status reads
    decided; `overdue` when `follow_up_by` is before `today_iso`); `due` `:416-436` (`all` from
    `list_pages(Some("decision"), Some(1000))`, sorted by `decided` descending then slug; `due`
    the decided or revised ones with a `follow_up_by` on or before `date_after(days)`, sorted by
    `follow_up_by`). `crates/rusty-core/src/brain/frontmatter.rs:351-360`: `today_iso` from
    seconds since the epoch, the UTC date.
  - **Rusty's page:** `crates/rusty-app/qml/DecisionsPage.qml`: `brain_due { days: 0 }` `:22`; the
    status labels `:23-25`; a read again on `onDataChanged` `:39`; the row (accent ◆, the title or
    the slug, the status coloured faint for superseded and "alive" for decided, "decided DATE",
    "follow up by DATE" in gold when overdue, a click opening the page) `:43-63`; Rusty's line on
    the loop `:71`; "Due" when any `:74`; "N decisions" with the singular `:76`; the empty line or
    the connection status `:86`. `Main.qml:955` (the page's host, `openPage(s, false)`),
    `:384-392` (`openPage`: an open tab first, else a new one), `:1051` (the ribbon's
    `check-square` button, "decide"). The store skill `ask-decide-follow-up` (Rusty's store)
    names the Decisions view as where follow-ups come due.
  - **The live store** (read only): 239 decision pages (229 decided, 7 kept, 3 superseded, 0
    revised), 174 with a `follow_up_by`; `rusty-cli brain due` lists 15 due, all overdue, earliest
    2026-09-19, and answers in about 11 ms (it links `rusty-core`). Decision file names average 74
    characters, the longest 138. The box runs on CDT (UTC-5).
  - **The batch's drafts** (queued 2026-10-03, read for their names): #643's D4, D5, D8 and D9 and
    its notes' `Rusty` global (state, `call` within 5 s, the `list_changed` subscription) and the
    Python stand-in over a state folder; #644's D4 (the toast while off), D5 (the fixed row, "an
    entry appears with its tab", Today alone, R-D9's order) and its stand-in's `SIGUSR1`; #645's
    `rusty` actions, `rusty::OpenPage { slug, preview }`, `rusty::page::open_later` and its D9
    (every open deferred); #646's Out (R6's project view reads `brain_due`); #647's D1 (the opener
    in `decisions::open`'s shape), D9 (when a center tab reads), D12 (a node opens kept) and D13
    (off), and its Graph entry (`IconName::GitGraph`, dispatching `rusty::OpenGraph`).
  - **The follow-up drafts beside this one** (2026-10-03): #655, R6's project view, reads
    `brain_due { days: 0 }` and names this ticket's `marley_rusty::decisions` as the one owner of
    that view (§14), whichever of #655, #658 and #659 lands first writing it with the fields the
    others read; its stand-in gains a `brain_due` against the day it runs. #658, the Tasks
    tab, R7's other half: its entry (`IconName::ListTodo`) goes after Graph, so before Decisions;
    its D11 answers an action while Rusty is off "as #645's `rusty:` actions do", and its Out
    leaves the Decisions half and the rename to this ticket.
  - **Zed's pieces:** `ui::ListItem`; `ui::ListSubHeader` (`list_sub_header.rs:14-38`); `ui::Chip`
    (`chip.rs:29-92`, `label_color`, `bg_color`, `tooltip`); `ui::sticky_items`
    (`sticky_items.rs:20`, the project panel's `:7630`); `IconName::CheckDouble` and `Notepad`
    (`icons/src/icons.rs`). Ctrl+Alt+Shift+Y is bound in neither `default-linux.json` nor Marley's
    keymap.
  - **Ely** (HEAD `2f8b2f6`): `src/lists/grouped.rs:1-85` (`GroupedList` over `StickyHeader`,
    `src/layout/scroll_aids.rs:121`); `src/data_display/badge.rs:15-125` (`Tone`, `Badge`);
    `src/data_display/timeline.rs:1-30` (`TimelineItem`, `Timeline`).
- **Decisions:** D1 to D13 in the spec. In short: the tab is "System One calls" and every name in
  the code follows it; the action's new id keeps the old one as Zed's deprecated alias; the four
  scenarios type the new palette name because the old one matches nothing; Rusty's tab is
  `BrainDecisionsView` over one `brain_due` call, drawn as Rusty serves it with no date math; a row
  opens its page kept; its entry sits at R-D9's place in the fixed row; it reads while shown and
  when shown again; off and down as the sibling tabs are; the follow-up form is the next slice; no
  Ely port; no restore.

### Design
- **Approach, part 1 (the rename).**
  - `decisions.rs` moved by `git mv` to `system_one_calls.rs` (both in
    `crates/marley_workbench/src/`); in it: the module doc ("System One calls (#565): ..."),
    `SystemOneCallsView` (its `Debug` name too), `open`, the ids `system-one-calls`,
    `system-one-calls-run-check`, `system-one-calls-set-key`, `system-one-calls-forget-key` and
    `system-one-call-{id}`, the key context `SystemOneCalls`, the heading and `tab_content_text`
    "System One calls", the log line "system one calls: reading today's calls". No other line
    changes.
  - `marley_workbench.rs`: `pub mod system_one_calls;` in place of `pub mod decisions;` (the list
    stays sorted); in the `marley` actions, `/// Opens System One calls: the System One layer's
    calls today, the day's spend and where the key comes from.`, `#[derive(Eq)]`,
    `#[action(deprecated_aliases = ["marley::OpenDecisions"])]`, `OpenSystemOneCalls`.
  - `system_one.rs`: the import (`OpenSystemOneCalls`, `system_one_calls`), the registration, the
    module doc and four comments, and the three strings: "set {KEY_VARIABLE}, or Set Key in System
    One calls", "no key: set MARLEY_SYSTEM_ONE_KEY, or Set Key in System One calls", "System One
    logged the check in shadow mode. Open System One calls to see it."
  - `find.rs:168` and `marley_system_one/src/reading.rs:143`, `:156`: "System One calls" in the
    comments.
  - `settings_ui/src/marley_page.rs`: the comment `:639`; the six descriptions ("Shadow logs it in
    System One calls" and the like, each sentence otherwise as it is); the `ActionLink`'s title
    "System One Calls", button "Open System One Calls", its description's first words unchanged,
    and `cx.build_action("marley::OpenSystemOneCalls", None)`.
  - `settings_content/src/marley.rs:343`: "The use asks and logs; what it read shows only in the
    System One calls view."
  - `crates/marley_workbench/guide/index.html`: the contents link `#system-one-calls` "System One
    calls"; Turn it on's two mentions; the article `id="system-one-calls"`, its heading and its
    `marley: open system one calls` step; the palette table's row.
  - The scenarios: `palette "marley: open system one calls"` at each of the five places; the
    `echo` lines ("== System One calls", "== every call in System One calls", "== System One
    calls says what the reading would show") and the header comments; shot names unchanged.
    `script/e2e/golden:41`, `:47`: "in System One calls".
  - The sweep the review runs: `grep -rn 'Decisions\|decisions' <the files of D3>` returns only
    the alias line;
    `grep -rn 'marley: open decisions' script/ crates/marley_workbench/guide/` returns nothing;
    `grep -rn '\[`.*[Dd]ecisions' crates/marley_workbench crates/marley_system_one` returns no
    intra-doc link.
- **Approach, part 2 (the tab).**
  - `marley_rusty::decisions` (pure, no gpui, serde): `Due { due: Vec<DecisionSummary>, all:
    Vec<DecisionSummary> }` and `DecisionSummary` as Rusty serializes them, with `#[serde(default)]`
    on every field and unknown fields ignored; `DecisionStatus::parse(&str)` (an empty string
    reads `Decided`, as Rusty's summary does); `parse_due(text) -> Result<Due>` over the tool's text
    block; `Entry { DueHeader, AllHeader { count }, Row { section: Section, index } }` and
    `entries(&Due) -> Vec<Entry>` (the Due header and rows only when `due` is not empty); the
    row's words: `decided_line` ("decided 2026-09-20") and `follow_up_line` ("follow up by
    2026-09-27", with "· overdue" when flagged), `count_line` ("1 decision", "7 decisions");
    `element_key(section, slug)`. No date is parsed.
  - The stand-in (`crates/marley_rusty/stand_in/rusty-mcp`, #643's Python program with the vault
    tools #644 to #647 gave it): `brain_due { days }` reads every `*.md` under the vault whose
    frontmatter `type` is `decision`, builds each summary as `decision_summary` does (title from
    the frontmatter, else the file name; status empty reads decided), sorts `all` by `decided`
    descending then slug, filters and sorts `due` as `due` does with today and today plus `days`
    from one date: the state folder's `today` when present, else the local date. A state-folder
    `fail` file whose first line names a tool makes that tool answer a JSON-RPC error with the
    file's second line as its message. Each call is logged, as for every tool; `SIGUSR1` sends
    `list_changed` (#644's).
  - `marley_workbench::rusty`: `OpenDecisions` joins the `actions!(rusty, [...])` #645 declares,
    with its doc ("Opens Decisions: Rusty's brain loop, the follow-ups due first, then every
    decision with its status and dates."); `init` registers it on every workspace
    (`cx.observe_new`), its handler checking `marley.rusty.enabled` each time; `pub mod
    decisions_tab;`.
  - `marley_workbench::rusty::decisions_tab`:
    - `open(workspace, window, cx)`: off, the toast through the workspace (`NotificationId` of the
      view) and nothing else; on, `items_of_type::<BrainDecisionsView>` and `activate_item`, else a
      new view through `add_item_to_active_pane` with the focus.
    - `BrainDecisionsView { workspace: WeakEntity<Workspace>, focus_handle, shown: Shown, notice:
      Option<SharedString>, stale: bool, reading: Option<Task<()>>, queued: bool, scroll:
      UniformListScrollHandle, _subscriptions }`, where `Shown` is `Reading | Off | Listed(Due)`
      and the notice holds a failed read's error or the not-connected line.
    - `read`: while one is in flight, `queued = true`; else `brain_due { days: 0 }` through #643's
      `call`, parsed on the background executor, then `shown`, `notice` and `cx.notify()`, and the
      queued read if any. Off, nothing is called.
    - Subscriptions: #643's change signal (active item: `read`; else `stale = true`), the `Rusty`
      global (off: `Shown::Off`, the list dropped; on again: `read`; not connected: the notice;
      connected again: `read`), and the workspace's `ActiveItemChanged` (shown and stale, or the
      service connection: `read`). "Its pane's active item" is asked of the pane that holds it, as
      `fleet_shows` asks (`fleet.rs:330-347`).
    - `render`: `v_flex().key_context("RustyDecisions").track_focus(..)`; the title "Decisions"
      (`LabelSize::Large`, as the System One calls tab draws its own); Rusty's line on the loop in
      muted small text; the notice with a Read again `Button` when it is an error; then the
      states: Reading ("Reading Rusty's decisions..."), Off ("Rusty is off. Turn it on in the
      Rusty section of the Marley settings."), the empty line, or the `uniform_list` over
      `entries`: headers as `ListSubHeader`s ("Due", the count line), rows as `ListItem`s with the
      title `Label` (truncated) and an end slot of the status `Chip`, the decided line and the
      follow-up line (`Color::Warning` when overdue); the row's tooltip holds the title and the
      slug; `on_click` calls `rusty::page::open_later(self.workspace.clone(), slug, false, window,
      cx)`.
    - `Item`: `tab_content_text` "Decisions", `tab_icon` `IconName::CheckDouble`,
      `tab_tooltip_text` "Rusty's decisions and follow-ups"; `Focusable`; `EventEmitter<()>`.
  - `marley_workbench::rusty::brain` (#644's `BrainView`): the fixed row gains an `IconButton`
    (`IconName::CheckDouble`, `Tooltip::for_action("Decisions", &OpenDecisions, cx)`) whose click
    calls `window.dispatch_action(Box::new(rusty::OpenDecisions), cx)`, placed after Graph (and
    after Tasks once it exists).
  - `crates/marley_workbench/guide/index.html`: an article `rusty-decisions`, "Decisions", after
    the Brain view's: what it lists, the fixed row's entry and `rusty: open decisions`, the click,
    that Rusty's agents record the decisions and follow-ups.
- **File manifest.**
  - Marley: `crates/marley_workbench/src/system_one_calls.rs` (moved from `decisions.rs`),
    `crates/marley_workbench/src/marley_workbench.rs`, `crates/marley_workbench/src/system_one.rs`,
    `crates/marley_workbench/src/find.rs`, `crates/marley_system_one/src/reading.rs`,
    `crates/marley_workbench/guide/index.html`; `crates/marley_rusty/src/decisions.rs` (new),
    `crates/marley_rusty/src/marley_rusty.rs` (the module),
    `crates/marley_rusty/stand_in/rusty-mcp`;
    `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/src/rusty/decisions_tab.rs`
    (new), `crates/marley_workbench/src/rusty/brain.rs`; `script/e2e/565-system-one-layer.sh`,
    `script/e2e/566-stop-kind.sh`, `script/e2e/568-inbox-order-and-risk-chips.sh`,
    `script/e2e/573-english-at-the-prompt-second-stage.sh`, `script/e2e/golden`;
    `script/e2e/659-rusty-decisions-tab.sh` (new, Test phase). No manifest changes:
    `marley_workbench` already depends on `ui`, `workspace` and (after #643) `marley_rusty`, which
    already has `serde` and `serde_json`.
  - Zed: `crates/settings_ui/src/marley_page.rs` (crate `settings_ui`, Marley's file): the comment,
    the six descriptions and the link. `crates/settings_content/src/marley.rs` (crate
    `settings_content`, Marley's file): one doc comment. Each existing hunk's `// Marley:` comment
    stays. No `crates/zed/src/zed.rs` change: `"rusty"` is #645's line and the alias is in the
    `marley` namespace.
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  - `:63` (`crates/settings_ui/src/marley_page.rs`): "an Open Decisions link that dispatches
    `marley::OpenDecisions` by name" becomes "a System One Calls link (Open System One Calls) that
    dispatches `marley::OpenSystemOneCalls` by name (#565; renamed by #659, which also names the
    System One calls tab in the six mode descriptions)".
  - `:59` (`crates/settings_content/src/marley.rs`): "#659: `SystemOneMode::Shadow`'s doc names the
    System One calls view" appended. No new row.
- **The commit receipt** binds the guide page, the stand-in, the scenarios and the golden list, so
  all of them change in the Code phase, before the gate.

### Visual check plan
The scenario `script/e2e/659-rusty-decisions-tab.sh`, `compositor sway`. Setup as the spec's UI
proof says: the scratch repository opened with `open_path`; the vault written by the scenario
under `$E2E_WORK/vault` with `.git` initialised; the stand-in linked as `$E2E_WORK/bin/rusty-mcp`,
named in `MARLEY_RUSTY_MCP`, pointed at the vault, `2026-10-03` in its state folder's `today`;
`profile_setting` for `marley.rusty.enabled` true and the embedded connection; the run's keymap
binding Ctrl+Alt+Shift+Y to `marley::OpenDecisions`. The Brain view is reached by #644's header
switch, as #644's scenario reaches it; the positions of the switch, the fixed row's entry, rows,
Read again and the settings link are measured from the first shots, as other click scenarios do.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `palette "marley: open system one calls"` | `659-01-system-one-calls`: the tab's title and heading |
| REQ-002 | Ctrl-W; Ctrl+Shift+P, type "open decisions", no Return | `659-02-palette`: `rusty: open decisions`, no `marley: open decisions` |
| REQ-003 | Escape; Ctrl+Alt+Shift+Y | `659-03-old-id`: the System One calls tab |
| REQ-004 | Ctrl-W; `palette "marley: open settings"`; the wheel to the System One section's end | `659-04-settings-link`: System One Calls, Open System One Calls |
| REQ-005 | Click Open System One Calls | `659-05-settings-opened`: the tab in Marley's window, no Settings window |
| REQ-008 | Ctrl-W; click Brain in the rail's header | `659-06-fixed-row`: Today, Graph and Tasks (where shipped), Decisions last |
| REQ-009 to 013 | Click Decisions | `659-07-decisions`: Due (overdue first, warning colour), "7 decisions", chips, dates, a cut title |
| REQ-014 | Click Use a rail | `659-08-opened`: its kept Page tab in front |
| REQ-016, 015 | Edit `ship-the-rail-switch.md` from outside; `SIGUSR1`; `expect` no new `brain_due`; `palette "rusty: open decisions"`; `expect` one more | `659-09-found`: the same tab, one in the bar, the edit shown |
| REQ-017 | Edit `try-the-stand-in.md` from outside; `SIGUSR1`; settle | `659-10-live`: no Due section, the row revised |
| REQ-018 | Write `fail` (`brain_due`, a message); `SIGUSR1` | `659-11-failed`: the error, Read again, the list kept |
| REQ-019 | Remove `fail`; click Read again | `659-12-read-again`: the list, no error |
| REQ-020 | Move `decisions/` out of the vault; `SIGUSR1` | `659-13-empty`: "No decisions yet." |
| REQ-021 | Set `marley.rusty.enabled` false from outside; settle; `expect` no `brain_due` after | `659-14-off`: "Rusty is off" |
| REQ-022 | `palette "rusty: open decisions"` | `659-15-off-action`: the toast; one Decisions tab |
| REQ-006, 007, 023, 024 | Not driven | Review: the sweep above; the scenarios' diff; the connection subscription; no date math |

Not reached by this scenario: the not-connected line (the keeper reconnects the stand-in within a
second, so the state does not hold still for a shot; the review reads the subscription), and the
keymap schema's deprecation note (Zed's own, from the alias). The four edited System One scenarios
are not run (§7: no regression per ticket); they now type the palette line `659-01` shows working,
and the review checks each edited line. The golden set's 565 and 568 run by hand at the next
`just regress`.

### Risks
- **Shapes this ticket does not own.** #643's `Rusty` global, `call`, change signal and stand-in,
  #644's fixed row, #645's `rusty` actions and `open_later` were drafted beside one another and
  this ticket takes their names as drafted; #655 may write `marley_rusty::decisions` and the
  stand-in's `brain_due` first. Promotion re-reads each as shipped; the decisions do not depend on
  their spelling, and without a `today` file the stand-in keeps the day it runs, as #655's
  fixtures expect.
- **An action while Rusty is off.** The drafts differ: #645 opens no Page tab, #658 answers as
  #645 does, and #644's `ToggleBrainView` and #646's toggle show the toast that names the switch.
  This ticket opens nothing and shows the toast (D10, REQ-022); promotion takes one rule for every
  `rusty:` action from what #645 shipped and moves REQ-022 with it.
- **A missed line in a System One scenario** fails only when that scenario runs, which no ticket
  does now. The sweep (`grep -rn 'marley: open decisions' script/`) must come back empty.
- **A user's keymap** binding `marley::OpenDecisions` keeps working through the alias; the schema
  marks it deprecated. A binding in a key context named `Decisions` would stop matching, since the
  context becomes `SystemOneCalls`; the view has no bindings of its own (Marley's keymap names no
  such context), so none is expected.
- **The settings link dispatches by a string.** A typo logs an error and does nothing; shot
  `659-05` clicks it.
- **Rusty's two calendars.** `today_iso` (`frontmatter.rs:351-360`) is the UTC date while `due`'s
  horizon is `chrono::Local` (`decisions.rs:90-94`). On this box (UTC-5) from 19:00 to midnight a
  follow-up due today is flagged overdue though `due` counts it as today's, and a decision recorded
  then gets tomorrow's `decided`. #644 met the same split between `brain_daily_note` and
  `brain_capture`. Marley shows Rusty's flags (D5); a Rusty-side ticket when Chad confirms. The
  scenario's stand-in uses one date, so its shots are not affected.
- **`brain_due` reads every decision page twice per call** (`decision_summary` reads the page and
  its file): 239 today, about 11 ms through `rusty-cli`. Reads happen on changes while the tab
  shows, folded, so the cost stays per change.
- **Rusty's cap of 1000 decisions** (`due`'s `list_pages` limit): past it the oldest drop out of
  `all` and the count reads 1000. A Rusty-side matter, as #647 noted for typed edges.
- **TICKET-040**: archived decisions may come back after a folder delete until it lands (Out).
- **Two tabs once called Decisions.** Old docs, closed tickets and the CHANGELOG name System One's
  log Decisions. The CHANGELOG's Changed entry says it is now System One calls and that Decisions
  is Rusty's.
- **The follow-up form waits.** The live store holds 15 overdue follow-ups and nothing in Marley
  records one; until the next slice, agents and `rusty-cli brain follow-up` do.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the twelve places above (`### Decisions` becomes `### System One
  calls`, its first line `marley: open system one calls`); a Decisions section in the Rusty part,
  beside #644's to #647's; the palette tables.
- `docs/marley/walkthrough.md`: §10.1 "System One calls and the check (#565)" with its steps and
  box, `:1447`, the palette table `:1598`; a stop for Rusty's Decisions tab after the Brain view's,
  following this ticket's shots.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the System One section's
  file, type and action names; the Agent tab's "as `system_one_calls::open`"; Rusty's Decisions
  tab), `marley_rusty`'s page (the `decisions` module), `docs/marley/rusty-in-marley.md` (R-D3's
  DecisionsPage row done, open decision 4 carried out, R7's Decisions half shipped, the follow-up
  form and the Rusty-side findings named), `docs/marley/three-prong-plan.md:266` (the S1 row),
  the two `zed-touchpoints.md` rows checked, `CHANGELOG.md` (Changed and Added).

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the three templates and the brief's three parts.
- [x] Read the Rusty-in-Marley plan in full (R-D0 to R-D10, the slices, open decision 4, Rusty's
      triage) and the queued #643 to #647 specs and notes for their names.
- [x] Recall: the ledgers (AD-565, F-565, AD-609, AD-599, L-515, PR rename sweep, F-606, L-633
      twice, L-531, L-516), the completed pipelines 565, 599, 609 and 633, a read-only brain
      search.
- [x] Discovery with file:line: System One's tab, its action, every string, comment, doc, guide
      line and scenario that names it; gpui's aliases, the palette and the keymap schema; Rusty's
      five loop tools, `due`, `decision_summary`, `follow_up` and `DecisionsPage.qml`; the live
      store's counts and dates (read only).
- [x] Found the scenarios that open the tab (565 twice, 566, 568, 573) and why each must change
      (the palette's match), and that none checks a renamed string, id or log line.
- [x] Prior-art sweep, three legs; Ely's `grouped.rs`, `badge.rs` and `timeline.rs` read, none
      ported, and why.
- [x] The alias decision locked (D2); the follow-up form judged not small and split out with a
      one-line scope (D11).
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D13, 24 EARS rows, phase plan.
- [x] Design: both parts, the file manifest by crate, the two touchpoint rows, the review's
      sweep, the visual check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
- Rusty's TICKET-044 (confirmed): every date Rusty writes (`created`, `updated`, `decided`,
  follow-up headings, timeline) counts the UTC day while the due list counts the local day. Until
  it lands the tab shows `brain_due`'s flags and horizon as served and computes no overdue state
  of its own; the 19:00 to midnight mismatch is expected and named in the notes. TICKET-048 will
  add `superseded_by` and a `followed_up` date to the summaries.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline; #643 to #658 landed (the last 22e0b4dafc).
- **Brain:** `brain ask` before the rename (a consultation id with due follow-ups on other work
  only).
- **Recall, added:**
  - L-657: the palette ranks the command used last first; the scenario binds keys where two
    names share a prefix.
  - L-658: act on a change of the `Rusty` global's link, not on each of its notifications; hide a
    tab by its pane's other tab.
  - F-658: the hidden-tab rule broken by a broad observer.
- **As shipped, against the draft:**
  - `marley_rusty::decisions` exists (#655): `BRAIN_DUE`, `DecisionSummary { slug, title,
    follow_up_by: Option<String>, overdue }` and `due_from_answer` (the `due` list only); the
    Knowledge panel's project view reads it. This ticket extends it, adding `status`, `decided`,
    `all`, `Due`, `DecisionStatus`, the entries and the row's words. The project view's reader
    keeps working. Rusty serves `follow_up_by` as a string, empty when none: the view reads an
    empty one as none.
  - Rusty at `13249a8` also serves `followed_up` and `superseded_by` on each summary (its
    TICKET-048), which the draft's Out says it lacks. They stay out of this ticket's rows (the
    rows show what the spec names) and are noted for the follow-up ticket.
  - Rusty sorts `all` by `decided` descending, then slug, and `due` by `follow_up_by`. The
    stand-in's `brain_due` (#655) serves files in vault order and the real date. It gains Rusty's
    `all` order, a state-folder `today` that fixes its date, and a `fail` file that makes the named
    tool answer a JSON-RPC error.
  - The stand-in has no `SIGUSR1`. Its vault watcher announces each page edit by itself, so the
    scenario's edits of decision pages are announced with no signal.
  - The fixed row holds Today, Graph and Tasks. Decisions goes last and opens through the rail's
    multi-workspace, as Graph and Tasks do (`open_later`), not by `dispatch_action`, which from
    the rail does not reach the workspace's handlers.
  - The `rusty` actions are declared per module (`graph_tab.rs`, `tasks_tab.rs`), so
    `OpenDecisions` is declared in `decisions_tab.rs`.
  - `marley_workbench::decisions` (#565's System One view) is the module the rename moves; the
    touchpoint rows for `settings_ui/src/marley_page.rs` (`:63`) and
    `settings_content/src/marley.rs` (`:59`) exist and are widened before their hunks.
- **Phase 1 PASS** (2026-10-04): the work runs autonomously under Chad's goal.

## Phase 2: Code (2026-10-04)
- **Built, part 1:**
  - `decisions.rs` was moved by `git mv` to `system_one_calls.rs`: `SystemOneCallsView`, the ids
    `system-one-calls*`, the key context `SystemOneCalls`, the heading and tab "System One calls",
    and the log line.
  - `marley::OpenSystemOneCalls` with `deprecated_aliases = ["marley::OpenDecisions"]`.
  - `system_one.rs`: the import, the registration, the comments, and the three strings.
  - `find.rs`, `reading.rs`, `settings_content/src/marley.rs`: comments and the Shadow text.
  - `marley_page.rs`: the comment, the six descriptions, and the link (System One Calls, Open
    System One Calls, `marley::OpenSystemOneCalls`).
  - The guide's article `system-one-calls` and its mentions; scenarios 565, 566, 568 and 573;
    the golden list's two lines. Both ledger rows were widened first.
- **Built, part 2:**
  - `marley_rusty::decisions` (#655's, extended): `status`, `decided`; `follow_up`,
    `decided_line` and `follow_up_line` (empty `follow_up_by` read as none); `DecisionStatus`;
    `Due` with `all`; `parse_due`; `Section`, `Entry`, `entries`, `count_line`.
    `due_from_answer` stays for the project view.
  - The stand-in: the state folder's `today`, a `fail` file naming a tool and its message, and
    `all` in Rusty's order.
  - `rusty/decisions_tab.rs` (new):
    - `rusty::OpenDecisions`, `open` and `open_later`.
    - `BrainDecisionsView` with `Link` and `ReadDue` (L-658), the title and Rusty's line on the
      loop, and the state lines.
    - Due and the count as `ListSubHeader`s; rows as `ListItem`s (title, a status `Chip`, the
      decided line, the follow-up line in the warning colour when overdue, a tooltip with the
      slug) opening the page through `page::open_later`.
  - `brain.rs`: Decisions after Tasks in the fixed row. The guide gains a Decisions article.
- **Deviations:**
  - The rows are a plain scrolling column, not a `uniform_list`: headers and rows differ in
    height, and Rusty lists at most 1,000 decisions.
  - The fixed row's entry opens through the rail's multi-workspace, not `dispatch_action`
    (Promotion).
  - The tooltip is `Tooltip::text`, since the entry is reached by click.
- **Review:**
  - The read rules are the Tasks tab's: no read while hidden (`ReadDue::WhenShown`, read at the
    next draw), a read on the link coming up, and Rusty off drops the list and calls nothing.
  - A failure keeps the list drawn and shows Read again. Nothing computes a date: `overdue` and
    the order are Rusty's. The open defers.
  - The sweep finds "Decisions" only in the alias line and the module doc's history.
- **Gate:** `just gate-diff` GREEN, 17 of 17, on the first run.

## Phase 3: Test (2026-10-04)
- **Scenario:** `script/e2e/659-rusty-decisions-tab.sh` under `compositor sway`, against the debug
  `marley`. The final run exited 0 with its four checks passing: "brain_due was asked", "no
  brain_due while hidden", "one more brain_due once shown" and "no brain_due once off". The
  stand-in's log closes on four `brain_due {"days": 0}` calls from one pid.
- **Measuring runs** (scratch scripts, not committed) found the positions. The Settings window's
  System One section is long and its end moved between runs, so the scenario types "System One
  Calls" into the settings search, which narrows the page to that section, and scrolls to its end.
- **Bug found in Test:** the first full run drew the Due header, the count and the rows with wide
  gaps. `ListSubHeader` carries `flex_1`, so in the tab's column each header grew to share the
  spare height. The headers are now wrapped in `div().flex_none()` (`header()` in
  `decisions_tab.rs`). A source change after the Phase 2 green: the final gate runs again at
  Complete.
- **Shot 06 retaken:** its first take had the rail header's tooltip ("Brain Ctrl-Alt-V") over the
  third and fourth entries. The scenario now moves the pointer off the header before the shot.
- **Shots**, each read:
  - `659-01-system-one-calls` (REQ-001): the tab titled System One calls, heading System One calls,
    the off line, provider, key, Run Check, Set Key, Forget Key, "No calls today."
  - `659-02-palette` (REQ-002): "open decisions" typed; one match, `rusty: open decisions`; no
    `marley: open decisions`.
  - `659-03-old-id` (REQ-003): Ctrl+Alt+Shift+Y, bound to `marley::OpenDecisions`, opened the
    System One calls tab.
  - `659-04-settings-link` (REQ-004, REQ-006): the System One section's end: System One Calls with
    Open System One Calls; the Stall Kind, Running Error and Typed Line descriptions say "in System
    One calls".
  - `659-05-settings-opened` (REQ-005): Marley's window alone on the output, the System One calls
    tab in front.
  - `659-06-fixed-row` (REQ-008): cropped at 4x: Today, Graph, Tasks, then Decisions (the double
    check), last.
  - `659-07-decisions` (REQ-009 to REQ-013): the Decisions tab with Rusty's line on the loop; Due
    holds Try the stand-in (follow up by 2026-09-27 · overdue, in the warning colour) above Ship
    the rail switch (follow up by 2026-10-03); "7 decisions", newest decided first; decided, kept,
    revised and superseded chips (superseded muted); decided dates on every row, follow-up dates
    where set. The entry's tooltip reads Decisions.
  - `659-08-opened` (REQ-014): Use a rail in a kept Page tab in front: title, status kept, decided
    2026-09-10.
  - `659-09-found` (REQ-015, REQ-016): the same Decisions tab in front, one in the bar; Due holds
    Try the stand-in alone; Ship the rail switch reads kept with no follow-up. The log held no
    `brain_due` while the Page tab showed and one more after the palette.
  - `659-10-live` (REQ-017): no Due section; Try the stand-in revised, follow up by 2026-10-17.
  - `659-11-failed` (REQ-018): "The brain is busy; try again." in the error colour with Read again,
    the seven rows kept below.
  - `659-12-read-again` (REQ-019): the list, no error line.
  - `659-13-empty` (REQ-020): "No decisions yet. brain_ask, then brain_decide, writes the first
    one."; the rail's tree holds projects only. The open Page tab's title falls back to its slug
    once its page is gone: the Page tab's own behaviour, not this ticket's.
  - `659-14-off` (REQ-021): "Rusty is off. Turn it on in the Rusty section of the Marley settings.";
    the rail is back on Projects; no `brain_due` after the switch.
  - `659-15-off-action` (REQ-022): the toast with the same words; one Decisions tab in the bar.
- **The title cut**, which the UI proof names under `659-07`: at the window's full width the long
  title fits, so a scratch run moved the tab into a half-width pane (`pane: split and move right`).
  Its shot shows "Keep Zed's theme for every…" cut before the status chip, with the dates intact.
- **Not reached by a shot:** the not-connected line (REQ-023) and the computed-nothing rule
  (REQ-024), by review as planned; REQ-006's other strings and REQ-007 by the diff and the sweep.
- **Focus:** no stray focus seen; the palette, the keys and the clicks all reached Marley.

## Phase 4: Complete (2026-10-04)
- **Documented:**
  - `CHANGELOG.md`: Added (the Decisions tab) and Changed (System One calls).
  - `docs/marley/guide.md`: System One's section renamed throughout (`### System One calls`, the
    command, the key step, every "row in" line, the modes), with a line on the old name and
    alias; a `### The Decisions tab` section after the Tasks tab's; the palette table's row.
  - `docs/marley/walkthrough.md`: stop 2.15c (the Decisions tab, read only); 10.1 and 10.8
    renamed; the palette table's System One row renamed, and rows for `rusty: open tasks` (which
    #658 missed) and `rusty: open decisions`.
  - `docs/marley_architecture/marley_workbench.md`: the System One section's file, type, action,
    a note on the rename and alias, the Agent tab's `system_one_calls::open`; a Decisions tab
    section.
  - `docs/marley_architecture/marley_rusty.md`: the `decisions` module's #659 additions; the
    stand-in's `all` order, `today` and `fail`.
  - `docs/marley/rusty-in-marley.md`: R-D3's DecisionsPage row shipped, R7 shipped, R7b names
    TICKET-048's fields, open decision 4 done. `docs/marley/three-prong-plan.md`: C2 gains the
    Decisions tab; S1 names System One calls.
  - `docs/marley/zed-touchpoints.md`: the `settings_content/src/marley.rs` and
    `settings_ui/src/marley_page.rs` rows checked; both describe what shipped.
- **Knowledge:** F-claude-659-the-decisions-tabs-headers-grew-to-fill-its-column-001,
  L-claude-659-a-listsubheader-grows-in-a-column-001,
  L-claude-659-reach-a-settings-item-by-the-settings-search-001,
  AD-claude-659-decisions-is-rustys-and-system-ones-log-is-system-one-calls-001.
- **Brain:** `brain decide` on the Promotion's consultation, follow up by 2026-10-18:
  `decisions/marley-decisions-is-rustys-tab-system-ones-log-is-system-one-calls`.
- **Closed:** the ticket in `tickets/closed/`, the BACKLOG row gone (removed at promotion), this
  pair in `completed/`.
- **Gate:** run again after every edit, since the source changed after Phase 2's green (the
  `header()` fix).
