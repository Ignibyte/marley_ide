---
pipeline_id: 67d81e4f-ffdc-428a-8f98-403c10aa6b6b
ticket: docs/planning/tickets/open/TICKET-659-rusty-decisions-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Rusty's Decisions tab, with System One's log renamed System One calls"
type: feature
slice: Rusty in Marley R7 (its Decisions half), R-D3's DecisionsPage row and open decision 4
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md]
---

## Title
Two parts that ship together, because the second takes the first one's name. First, Marley's
System One log, the tab #565 called Decisions, becomes **System One calls** wherever it is named:
the tab, its action (`marley::OpenSystemOneCalls`, with the old `marley::OpenDecisions` kept as
Zed's deprecated alias so a keymap that binds it still works), the settings page's link, the
strings that point at it, the docs, the in-app guide and the four System One scenarios that open
it. Second, Rusty's brain loop takes the name: a **Decisions** tab over `brain_due`, the follow-ups
due first with the overdue ones marked, then every decision page with its status and dates, each
row opening its page through #645's opener, reached from the rail Brain view's fixed row (#644).
It is the screen Rusty's Qt app draws as `DecisionsPage.qml`. The plan records Chad's choice on its
open decision 4, 2026-10-02: Marley's System One log becomes "System One calls"; Rusty's tab keeps
"Decisions".

## Scope
### In
- **Order: after #645.** Builds on #643 (`marley.rusty`; the `Rusty` global in
  `marley_workbench::rusty` with its state, its `call` and its signal on
  `notifications/resources/list_changed`; `crates/marley_rusty` with `fixtures/` and the Python
  stand-in `stand_in/rusty-mcp`, named in `MARLEY_RUSTY_MCP`), #644 (the rail's Brain view in
  `rusty/brain.rs` and its fixed row, where an entry appears with its tab) and #645 (the `rusty`
  action namespace with `rusty::OpenPage`, `rusty::page::open_later`, and `"rusty"` in Zed's
  namespace test). It takes their names as they ship. It needs neither #646 nor #647. Two tickets
  queued beside it share pieces: #655 (R6's project view) reads this ticket's
  `marley_rusty::decisions` and gives the stand-in a `brain_due` if it lands first, and #658's
  Tasks entry, like #647's Graph entry, sits before Decisions in the fixed row.

**Part 1, the rename (D1 to D3).**
- `crates/marley_workbench/src/decisions.rs` moves to `system_one_calls.rs` (`git mv`):
  `DecisionsView` becomes `SystemOneCallsView` and `decisions::open` becomes
  `system_one_calls::open`; the tab text and the heading read "System One calls"; the key context
  becomes `SystemOneCalls`; the element ids, the log line and the module doc follow. Nothing it
  does changes.
- The action: `marley::OpenSystemOneCalls` among `marley_workbench.rs`'s `marley` actions, with
  `#[action(deprecated_aliases = ["marley::OpenDecisions"])]` and its doc comment renamed;
  `system_one::init` registers it.
- What a user reads: in `system_one.rs`, "or Set Key in System One calls" (twice) and the check's
  shadow toast, "Open System One calls to see it."; on the settings page
  (`settings_ui/src/marley_page.rs`), the link's title System One Calls, its button Open System One
  Calls, its dispatch by the new name, and the six mode descriptions that say "logs ... in
  Decisions"; the settings schema's text for `SystemOneMode::Shadow`
  (`settings_content/src/marley.rs`). The comments that name the view follow (`system_one.rs`,
  `find.rs`, `marley_system_one/src/reading.rs`, `marley_page.rs`).
- The in-app guide page (`crates/marley_workbench/guide/index.html`): the System One article
  `decisions` becomes `system-one-calls` with its contents line, the two mentions in Turn it on
  and the palette table's row. The file sits under `crates/marley_*`, which the commit receipt
  binds, so it changes in the Code phase.
- **The scenarios** that open the tab from the palette: `script/e2e/565-system-one-layer.sh` (two
  places), `566-stop-kind.sh`, `568-inbox-order-and-risk-chips.sh` and
  `573-english-at-the-prompt-second-stage.sh` type `marley: open system one calls` in place of
  `marley: open decisions`, and their comments and `echo` lines name the tab; the shot names stay.
  `script/e2e/golden`'s lines for 565 and 568 name the tab. D3 says why each must change.

**Part 2, Rusty's Decisions tab (D4 to D13).**
- **The typed view** (`marley_rusty::decisions`, pure): `brain_due`'s answer as `Due { due, all }`
  of `DecisionSummary { slug, title, question, status, decided, follow_up_by, overdue }`, with
  `DecisionStatus { Decided, Kept, Revised, Superseded, Other(String) }`; the entries the tab
  draws (the Due header and its rows when any are due, the count header, every decision), each
  with an element id unique across both sections; each row's words. Unknown fields are ignored.
  The module has one owner (§14), as #655 has it: if #655 lands first and writes it with the
  fields it reads, this ticket takes it and adds what the tab needs.
- **The action and the opener:** `rusty::OpenDecisions` ("rusty: open decisions") beside #645's
  `rusty` actions, registered on each workspace; it brings the workspace's Decisions tab forward
  or adds one to the active pane, as `system_one_calls::open` does. While `marley.rusty.enabled`
  is off it opens nothing and shows #644's toast (D10).
- **The tab** (`BrainDecisionsView` in `marley_workbench::rusty::decisions_tab`, an
  `impl workspace::Item`): "Decisions" with `IconName::CheckDouble`; under the title, Rusty's own
  line on the loop; a Due section when `due` is not empty, then "N decisions"; a row per decision
  with its title (cut with an ellipsis, whole in its tooltip beside the slug), its status as a
  `ui::Chip`, "decided DATE", and "follow up by DATE", in the warning colour with "overdue" when
  Rusty flags it (D5, D6).
- **Opening a page:** a click on a row calls `rusty::page::open_later(workspace, slug, false, ..)`
  (D7).
- **The fixed row:** #644's Brain view gains the Decisions entry at R-D9's place, after Today,
  #647's Graph and #658's Tasks, whichever have landed: `IconName::CheckDouble`, the tooltip
  "Decisions", dispatching `rusty::OpenDecisions` through the window (D8).
- **Reads** (D9): `brain_due { days: 0 }` on opening; on `list_changed` while the tab is its pane's
  active item, else when it next shows; on each showing with the service connection; one read in
  flight and one queued. A failed read keeps the list and shows the error with Read again.
- **States** (D10): reading; "No decisions yet. brain_ask, then brain_decide, writes the first
  one."; Rusty off (the list dropped, nothing called); not connected (the list kept, and a line
  saying so with #643's reason).
- **The stand-in** gains `brain_due` over its scratch vault by Rusty's rule
  (`rusty-core/src/brain/decisions.rs:384-436`), a `today` file in its state folder that fixes its
  date, and a `fail` file there that makes the tool it names answer a JSON-RPC error as `rusty-mcp`
  does (`main.rs:36-41`); where #655 or #643 to #647 gave it any of these first, it takes them.
- The in-app guide page's Decisions article (`rusty-decisions`, after the Brain view's), in the
  Code phase, and `script/e2e/659-rusty-decisions-tab.sh`.

### Out (explicitly deferred)
- **Following up from the tab, its own ticket** (D11). One-line scope: a Follow up button on each
  decided or revised row opens an inline form (Kept; Revised with a new date; Superseded with the
  successor picked from the decisions listed) that calls `brain_follow_up { slug, outcome, status,
  successor, follow_up_by }`, shows Rusty's refusals and reads again; the stand-in gains
  `brain_follow_up` with Rusty's checks (`decisions.rs:302-372`). Fifteen follow-ups on the live
  store are overdue today.
- **What `brain_due` does not serve:** the date a follow-up was recorded (it is in the page's
  `### Follow-up DATE: status` heading and its timeline, not in `DecisionSummary`), and a
  superseded decision's successor (`superseded_by` is a property, not a summary field). A
  Rusty-side request when Chad confirms: both on `DecisionSummary`. Until then a row shows the two
  dates Rusty serves.
- The question on each row (Rusty's page shows none; the typed view keeps it for when it is
  wanted), filters by status, search, a choice of horizon (`days`), keys to walk the rows, and a
  count of due follow-ups on the fixed row's entry or the tab.
- **R6's project view, #655:** a project's follow-ups due beside its page, read from the same
  `brain_due` through `marley_rusty::decisions`.
- **The Tasks tab, #658**, R7's other half.
- **Rusty's TICKET-035, the change cursor.** Nothing here depends on it. Until it lands, a decision
  another `rusty-mcp` process records shows at the next `list_changed` the embedded connection
  hears or at the next showing; Rusty's watcher announces a page file's change itself.
- **Rusty's TICKET-040.** Until it lands, a decisions folder deleted through Rusty can come back in
  `brain_due` from `archive/`, since `due` lists pages by type through the index. Marley shows what
  Rusty serves.
- **Rusty's two calendars**, a finding for Rusty (Risks): the loop's `today_iso` is the UTC date and
  `due`'s horizon the local one, so on this box from 19:00 to midnight a follow-up due today reads
  overdue and a decision recorded then is dated tomorrow. Marley shows Rusty's flags (D5); a
  Rusty-side ticket when Chad confirms.
- Recording decisions or consultations from Marley (`brain_ask`, `brain_decide`,
  `brain_no_decision`): agents do that; the tab reads.
- Restoring the tab after a restart (D13), and removing the old action id.

## Reference (§20)
Upstream Zed, kept as Zed has it: an action renamed with its old name kept through
`#[action(deprecated_aliases = [...])]`, so gpui's registry resolves the old name for keymaps and
`build_action`, the command palette lists only the new one, and the keymap schema marks the old one
deprecated with the new name, as Zed renamed `editor::OpenFile` to `editor::OpenSelectedFilename`
(`editor/src/actions.rs:733`) and `pane::CloseInactiveItems` to `pane::CloseOtherItems`
(`workspace/src/pane.rs:125`); a center tab as a `workspace::Item`, opened or focused through
`Workspace::items_of_type`, `activate_item` and `add_item_to_active_pane`; rows drawn with the `ui`
crate's `ListItem`, `ListSubHeader` and `Chip` in a `uniform_list`. The tab's content follows
Rusty's own `DecisionsPage.qml` (`/srv/stacks/rusty-v3/crates/rusty-app/qml/DecisionsPage.qml`,
MIT, the same owner): one `brain_due` call, the due first, then every decision with its status and
dates, a click opening the page, a read again on Rusty's change notice. No Warp behavior applies:
Warp keeps no decision log (its maps use "decision" only for the input classifier's verdict and
the agent's spawn choice).

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (`:52`,
  `:344-345`, `:519`): the command palette lists `available_actions`, each named by
  `humanize_action_name(action.name())`, which is why the palette shows only the new name and why
  the four scenarios must type it. `docs/orca_architecture/06-cli-automations-skills.md` §2.17
  (`:636-643`): Rusty holds decisions through `brain_ask`, `brain_decide` and `brain_follow_up`,
  and Marley grows no store of its own; this tab only reads. `docs/warp_architecture/`: grepped for
  "decision"; the input classifier's decision source and the agent's spawn choice, nothing to take.
- **Published material:** Michael Nygard, "Documenting Architecture Decisions" (2011): records
  whose status later reads superseded, the shape Rusty's decision pages share (`supersedes`,
  `superseded_by`); the MCP specification's `notifications/resources/list_changed`. Rusty's own:
  the store skill `ask-decide-follow-up` ("When the date comes (`brain_due`, the Decisions view,
  `/brief`), call `brain_follow_up`") and Rusty's TICKET-018, the brain loop.
- **The code we already ship:**
  - gpui (Zed) owns the rename. `deprecated_aliases` (`gpui/src/action.rs:84-87`); `insert_action`
    (`:293-333`) puts each alias in `by_name` and `all_names` and maps it to the new name, so
    `build_action` and a keymap resolve it; the palette names each available action by its own name
    (`command_palette/src/command_palette.rs:131`); the keymap schema says "Deprecated, use NEW"
    (`settings/src/keymap_file.rs:640-641`, `:690-691`); the keymap editor resolves it
    (`keymap_editor/src/keymap_editor.rs:1443`). Twenty-nine of Zed's actions carry one, fourteen
    of them in `zed_actions`. Taken unchanged: the alias needs no Zed file.
  - `workspace` and `ui` (Zed): `Item`; `ListItem`; `ListSubHeader`
    (`ui/src/components/list/list_sub_header.rs:14-38`); `Chip` (`chip.rs:29-92`); `Label`;
    `Tooltip`; gpui's `uniform_list`; `IconName::CheckDouble`, so no new asset. `ui::sticky_items`
    (`sticky_items.rs:20`, the project panel's at `project_panel.rs:7630`) would pin the section
    headers, which Rusty's page does not do: not used.
  - Cargo.lock: `serde` and `serde_json` read the answer. No date crate is needed (D5).
  - Marley: `decisions.rs` (`DecisionsView`, its opener `:52-60`, its `Item` `:414-420`), which
    this ticket renames and whose opener the new tab copies; `system_one.rs:288-300` (the
    registration); `settings_ui/src/marley_page.rs:1090-1113` (the link that dispatches by name,
    since the UI crate depends on no Marley crate); #643's client and stand-in, #644's fixed row,
    #645's `open_later`, #647's D9 and D13 (when a center tab reads, and what it does while off).
  - Rusty: `brain_due` (`rusty-mcp/src/main.rs:1185-1190`, `DueParams` `:403-409`) over
    `BrainManager::due` (`rusty-core/src/brain/decisions.rs:416-436`) and `decision_summary`
    (`:384-413`): `due` holds the decided or revised decisions whose `follow_up_by` falls on or
    before today plus `days`, earliest first; `all` holds every decision page, newest first, up to
    1000; `overdue` is a follow-up date before today. `DecisionsPage.qml:1-90` and its host
    (`Main.qml:955`; `openPage(slug, false)`, `:384-392`; the ribbon's `check-square`, `:1051`).
  - Ely GPUI Components (`2f8b2f6`): R-D10's table names no Ely story for Decisions, and nothing
    is ported (D12). Read: `src/lists/grouped.rs` (`GroupedList`, rows under group headers that
    pin while their group scrolls, over Ely's `StickyHeader`, `src/layout/scroll_aids.rs:121`):
    not taken, since Rusty's page pins nothing and Zed's `sticky_items` would own it;
    `src/data_display/badge.rs:74-125` (`Badge`, a label tinted by a `Tone`): Zed's `Chip` with a
    label colour draws the status; `src/data_display/timeline.rs` (`Timeline`): a decision's
    follow-up history, which this tab does not show.

## UI proof
`script/e2e/659-rusty-decisions-tab.sh` (`compositor sway`: it clicks the rail's header, the fixed
row, rows, a button and the settings link). Setup: the scratch repository opened with `open_path`;
a scratch vault under `$E2E_WORK/vault` (git initialised) with `projects/atlas`, which is not a
decision, and seven decision pages: `try-the-stand-in` (decided 2026-09-20, follow up by
2026-09-27), `ship-the-rail-switch` (decided 2026-09-30, follow up by 2026-10-03), a long-titled
`keep-zeds-theme-...` (decided 2026-09-25, follow up by 2026-10-20), `batch-tickets-by-five`
(revised, decided 2026-09-12, follow up by 2026-10-10), `use-a-rail` (kept, decided 2026-09-10),
`split-the-crate` (decided 2026-09-02, superseding the next) and `one-big-crate` (superseded,
decided 2026-09-01), all under `decisions/`. #643's stand-in is linked as `$E2E_WORK/bin/rusty-mcp`,
named in `MARLEY_RUSTY_MCP` and pointed at that vault, with `2026-10-03` in its state folder's
`today`; the run's copy of the settings turns `marley.rusty` on with the embedded connection; the
run's keymap (`$E2E_PROFILE/config/keymap.json`) binds Ctrl+Alt+Shift+Y to `marley::OpenDecisions`,
the old id. Never the user's brain (R-D8). Shots:
- `659-01-system-one-calls`: `marley: open system one calls` from the palette: a tab titled System
  One calls, the heading System One calls, its header lines, Run Check, Set Key and Forget Key.
- `659-02-palette`: the tab closed, the palette open with "open decisions" typed: `rusty: open
  decisions` listed, no `marley: open decisions`.
- `659-03-old-id`: Escape, then Ctrl+Alt+Shift+Y: the System One calls tab.
- `659-04-settings-link`: the tab closed; `marley: open settings`, scrolled to the System One
  section's end: System One Calls with its Open System One Calls button.
- `659-05-settings-opened`: Open System One Calls clicked: the Settings window gone and the System
  One calls tab in Marley's window.
- `659-06-fixed-row`: the tab closed, Brain clicked in the rail's header: the fixed row with Today,
  Graph and Tasks where #647 and #658 have shipped, and Decisions last.
- `659-07-decisions`: Decisions clicked: the Decisions tab; Due holding Try the stand-in (follow up
  by 2026-09-27, overdue, in the warning colour) above Ship the rail switch (follow up by
  2026-10-03); "7 decisions", newest first, with decided, kept, revised and superseded chips, the
  decided dates, the follow-up dates where set, the long title cut.
- `659-08-opened`: a click on Use a rail: its page in a kept Page tab, in front.
- `659-09-found`: while the Page tab shows, `ship-the-rail-switch.md` set to `status: kept` without
  `follow_up_by` from outside and the stand-in sent `SIGUSR1`: the stand-in logs no `brain_due` (an
  `expect`); then `rusty: open decisions` from the palette: the same Decisions tab in front, one in
  the tab bar, Due holding Try the stand-in alone, Ship the rail switch kept; the log holds one more
  `brain_due`.
- `659-10-live`: `try-the-stand-in.md` set to `status: revised` with `follow_up_by: 2026-10-17` and
  `SIGUSR1`, with no input: no Due section; the row revised, follow up by 2026-10-17.
- `659-11-failed`: a `fail` file naming `brain_due` in the stand-in's state folder and `SIGUSR1`:
  the error and Read again above the list kept.
- `659-12-read-again`: the `fail` file removed, Read again clicked: the list, no error.
- `659-13-empty`: `decisions/` moved out of the vault and `SIGUSR1`: "No decisions yet. brain_ask,
  then brain_decide, writes the first one."
- `659-14-off`: `marley.rusty.enabled` set false from outside: the tab says Rusty is off; the log
  holds no `brain_due` after it.
- `659-15-off-action`: `rusty: open decisions` from the palette: the toast naming the switch, one
  Decisions tab still.

## Locked-In Decisions
- D1: **The names.** Chad's "System One calls" (the plan's open decision 4). The tab text and its
  heading are "System One calls", in the sentence case of Marley's tabs (#647's "Local graph"); the
  settings item is "System One Calls" with the button "Open System One Calls", in that page's title
  case (Daily Budget, Typed Line); the palette reads "marley: open system one calls" from the
  action's name; prose says "the System One calls tab". Rusty's tab is "Decisions", its action
  `rusty::OpenDecisions` and its palette line "rusty: open decisions", as R-D3 has it.
- D2: **The action changes its id; the old id stays as Zed's deprecated alias.**
  `marley::OpenSystemOneCalls` with `#[action(deprecated_aliases = ["marley::OpenDecisions"])]`: a
  keymap that binds the old name opens the same tab, the keymap schema says "Deprecated, use
  marley::OpenSystemOneCalls", the palette lists only the new name, and nothing is stored under
  either name (the tab is not serialized; the settings link dispatches by name when clicked). The
  alias stays; its removal is not planned. Rejected: (a) keeping `marley::OpenDecisions` as the id,
  which leaves "marley: open decisions" for System One's log beside "rusty: open decisions" for
  Rusty's, the confusion the rename removes; (b) a new id with no alias, after which a binding to
  the old name fails to load and Zed reports it; (c) a keymap migration in Zed's `migrator`, a
  touchpoint and Zed's migration prompt for one action Marley's own keymap does not bind.
- D3: **Every name follows, not only the words a user reads.** File, type, opener, key context,
  element ids, log line and comments move with the tab's name, so after the rename the System One
  files (`system_one.rs`, `system_one_calls.rs`, `find.rs`, `marley_workbench.rs`'s action,
  `marley_system_one/src/reading.rs`, `settings_ui/src/marley_page.rs`,
  `settings_content/src/marley.rs`) say "Decisions" only in the alias string, and `decisions` in
  `marley_workbench` means Rusty's tab. The file moves with `git mv` so its history follows. The
  four scenarios have to change: the palette lists actions by name, and "marley: open decisions"
  matches no command once the name is gone (the fuzzy match needs a `d` after "open", and "system
  one calls" has none), so Enter would run nothing and every later step would shoot the wrong
  screen. Their shot names stay, since the archived #565, #566, #568 and #573 specs cite them.
  Rejected: renaming only the strings (a `decisions` module and a `DecisionsView` for System One
  beside Rusty's Decisions tab); renaming the shots.
- D4: **Rusty's tab is a type of its own, one per workspace.** `BrainDecisionsView` in
  `marley_workbench::rusty::decisions_tab` (beside #647's `graph_tab`), key context
  `RustyDecisions`, opened by `rusty::OpenDecisions` as `system_one_calls::open` opens its tab: the
  workspace's one tab comes forward, else a new one joins the active pane with the focus. The
  handler uses the `&mut Workspace` it runs in and reads no workspace handle (F-565). Not
  `DecisionsView`: the ledgers and the archived pipelines use that name for System One's tab, and a
  new type of the old name would make each of them ambiguous. Rejected: one tab with both logs (two
  programs, two meanings of "decision").
- D5: **One tool, its answer as served.** The tab calls `brain_due { days: 0 }`, as Rusty's page
  does, and draws `due` then `all` in Rusty's order with Rusty's `overdue`; Marley parses no date
  and compares none. Rusty owns the rule, so the tab, `rusty-cli brain due` and `/brief` agree, and
  the calendar finding (Out) is fixed once, in Rusty. A due decision is listed in both sections, as
  Rusty lists it, and the Due section shows only when `due` is not empty, as Rusty's does.
  Rejected: decision pages by type from `brain_list_pages`, each read for its dates (the summary is
  one call already); a horizon in the tab (Rusty's page uses 0; Out).
- D6: **The row.** The title (the slug when the title is empty), cut with an ellipsis where it does
  not fit, since the live store's decision slugs average 74 characters and the longest has 138,
  with the whole title and the slug in the row's tooltip; the status as a `ui::Chip` whose label
  takes the accent colour for decided (waiting on its follow-up), the muted colour for kept,
  revised and a status Marley does not know, and the disabled colour for superseded, as Rusty
  colours them; "decided DATE" when set; "follow up by DATE" when set, in `Color::Warning` with
  "· overdue" when Rusty flags it (Rusty's gold). Element ids carry the section and the slug, since
  a due decision is drawn twice (F-606). The entries are a `uniform_list`, each one row high,
  headers included: Rusty serves up to 1000 decisions and the live store has 239. The question is
  not shown (Out).
- D7: **A click opens the page kept.** `rusty::page::open_later(workspace, slug, false, window,
  cx)`, #645's deferred opener: a tab that shows the page comes forward, else a kept Page tab opens
  with the focus, as Rusty's page opens with `openPage(slug, false)` and #647's graph opens a node
  (its D12). Rejected: a preview tab (Zed's preview rule belongs to the project panel and the file
  finder; a click here chooses a decision to read).
- D8: **The fixed row's entry.** #644's row gains Decisions at R-D9's place (Today, Graph, Tasks,
  Decisions, Memory, Skills, Secrets, the absent ones skipped; Graph is #647's, Tasks #658's): an
  `IconButton` with `IconName::CheckDouble` (Rusty's ribbon draws a checked box, which Zed's icons
  lack; the double tick reads as decided, then followed up), the tooltip "Decisions", dispatching
  `rusty::OpenDecisions` with `window.dispatch_action` (L-515), which runs the action outside any
  view's update. The tab carries the same icon.
- D9: **When it reads.** On opening; on #643's `list_changed` while the tab is its pane's active
  item; a hidden tab marks itself stale and reads when it shows (the workspace's
  `ActiveItemChanged` naming it), as #647's D9 and AD-609 have it; with the service connection,
  which hears no notification (#643's D5), every showing reads. One read in flight, later changes
  folded into one more. A failed read keeps the list it drew and shows the error with Read again.
- D10: **Off and down.** `rusty::OpenDecisions` is registered on every workspace whatever the
  switch; while `marley.rusty.enabled` is off it opens nothing and shows "Rusty is off. Turn it on
  in the Rusty section of the Marley settings." (#644's D4). An open tab drops its list, says Rusty
  is off and calls nothing until Rusty is on again, when it reads. While Rusty is on and not
  connected, the tab keeps the list it drew and says Marley is not connected to Rusty, with #643's
  reason; it reads when the connection is back.
- D11: **Following up from the tab is its own ticket.** `brain_follow_up` takes a status and an
  outcome, a date for revised and, for superseded, a successor that Rusty checks is a decision.
  That successor needs a picker, since decision slugs run to 138 characters and nobody types one.
  With three shapes of form, Rusty's six refusals to show, and a stand-in that rewrites the page as
  Rusty does, it is not small, and Rusty's Qt page, the screen this rebuilds, has no such form to
  follow. It is the next slice (Out).
- D12: **No Ely port.** Zed's `ui` draws every piece (Prior art), and R-D10's table names no Ely
  story for this screen.
- D13: **Not restored after a restart** (AD-609, #645's D10): it holds nothing but a list one call
  brings back.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley: open system one calls` runs, the system shall open System One's log in a tab whose title and heading read "System One calls". | Shot `659-01-system-one-calls` |
| REQ-002 | WHEN "open decisions" is typed in the command palette, the system shall list `rusty: open decisions` and no `marley: open decisions`. | Shot `659-02-palette` |
| REQ-003 | WHEN a key that a keymap binds to `marley::OpenDecisions` is pressed, the system shall open the System One calls tab. | Shot `659-03-old-id` |
| REQ-004 | The Settings window's System One section shall show the System One Calls item with its Open System One Calls button. | Shot `659-04-settings-link` |
| REQ-005 | WHEN Open System One Calls is clicked, the system shall show the System One calls tab in the window the Settings window was opened from. | Shot `659-05-settings-opened` |
| REQ-006 | The system shall name System One's log "System One calls" in every string that named it Decisions: the mode descriptions, the check's shadow toast, the missing-key lines, the Shadow mode's schema text and the in-app guide page. | Review (the notes' grep) |
| REQ-007 | The scenarios 565, 566, 568 and 573 shall open System One's log by `marley: open system one calls`. | Review of the diff |
| REQ-008 | WHILE Rusty is on and connected, the Brain view's fixed row shall show a Decisions entry at its place in R-D9's order. | Shot `659-06-fixed-row` |
| REQ-009 | WHEN the fixed row's Decisions entry is clicked, the system shall open the Decisions tab. | Shot `659-07-decisions` |
| REQ-010 | WHERE `brain_due` serves follow-ups due, the Decisions tab shall list them first under "Due", in the order served, an overdue one before one due today. | Shot `659-07-decisions` |
| REQ-011 | The Decisions tab shall draw a follow-up date that Rusty flags overdue in the warning colour, with the word overdue. | Shot `659-07-decisions` |
| REQ-012 | The Decisions tab shall list every decision `brain_due` serves, newest first, under a header with their count. | Shot `659-07-decisions` |
| REQ-013 | Each decision row shall show the decision's title, its status (decided, kept, revised or superseded), the date it was decided and its follow-up date when one is set. | Shot `659-07-decisions` |
| REQ-014 | WHEN a decision row is clicked, the system shall open that decision's page in a kept Page tab. | Shot `659-08-opened` |
| REQ-015 | WHEN `rusty: open decisions` runs while the workspace has a Decisions tab, the system shall bring that tab forward and open no second one. | Shot `659-09-found` |
| REQ-016 | WHILE the Decisions tab is not its pane's active item, the system shall not read `brain_due` on Rusty's change notice, and shall read it when the tab shows again. | The stand-in's log; shot `659-09-found` |
| REQ-017 | WHEN Rusty announces a change while the Decisions tab is its pane's active item, the tab shall show the decisions as they now are, with no Due section when none is due. | Shot `659-10-live` |
| REQ-018 | IF the `brain_due` call fails, THEN the tab shall show the error with a Read again button and keep the list it drew. | Shot `659-11-failed` |
| REQ-019 | WHEN Read again is clicked, the tab shall read `brain_due` again and show its answer. | Shot `659-12-read-again` |
| REQ-020 | WHEN `brain_due` serves no decision, the tab shall say there are no decisions yet. | Shot `659-13-empty` |
| REQ-021 | WHILE `marley.rusty.enabled` is off, an open Decisions tab shall say Rusty is off and call no tool. | Shot `659-14-off`; the stand-in's log |
| REQ-022 | WHILE `marley.rusty.enabled` is off, `rusty: open decisions` shall open no tab and shall show a toast naming the switch. | Shot `659-15-off-action` |
| REQ-023 | WHILE Rusty is on and Marley is not connected to it, the Decisions tab shall keep the list it drew and say Marley is not connected. | Review |
| REQ-024 | The Decisions tab shall draw Rusty's due list, order and overdue flags as served, and compute no date itself. | Review |

## Phase Plan
- **P1 Plan**: promote after #645 completes (with #643 and #644 before it); re-verify every seam the
  notes cite against the code as shipped (the `Rusty` global's `call`, state and `list_changed`
  signal; the stand-in's state folder and whether it has a failure switch; `rusty/brain.rs`'s fixed
  row; #645's `rusty` actions and `open_later`; #647's Graph entry, if in); re-read the System One
  lines, which may have moved; ask the brain the alias question; Chad confirms the two spellings
  (D1) and the split of the follow-up form (D11).
- **P2 Code**: the `README.md` marker first; the two `zed-touchpoints.md` rows widened before their
  files; the rename (`git mv`, then the edits, the guide page, the four scenarios and the golden
  list's two lines); `marley_rusty::decisions`; the stand-in's additions; `rusty::OpenDecisions`,
  the tab and the fixed row's entry; the guide page's Decisions article; a review of the diff with
  the notes' greps; `script/gates.sh --diff` green.
- **P3 Test**: write and run `script/e2e/659-rusty-decisions-tab.sh`, read every shot.
- **P4 Complete**: CHANGELOG (Changed: the rename and the alias; Added: the Decisions tab) and
  architecture docs (§21), the user docs the notes list, the plan's R-D3 row, open decision 4 and
  R7 marked, ledger capture (§19), the brain decision, close the ticket, archive, commit.
