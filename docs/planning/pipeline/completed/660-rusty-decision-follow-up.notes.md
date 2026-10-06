# Record a decision's follow-up from the Decisions tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-660-rusty-decision-follow-up.md
- **Pipeline spec:** 660-rusty-decision-follow-up.spec.md

## Phase 1 — Plan (2026-10-05)
- **Request:** Chad, 2026-10-05, at the end of the #643 to #659 batch: write the R7b ticket and put
  it at the top of the Queue. The goal "continue on tickets until finished with everything" runs
  it.
- **Classification / tier:** feature, Rusty in Marley R7b; Marley crates only, no Zed touchpoint.
  Minted (no queued pair); the BACKLOG row removed at promotion; the ticket in progress.
- **Pre-flight:** green; no active pipeline; the README marker present; cargo idle.
- **Recall (§18.3):**
  - #659 (completed): the tab's `Link` and `ReadDue` rules (L-658), the `header()` box (F-659), the
    stand-in's `today` and `fail` files, and its D5: Marley works out no date while Rusty's two
    calendars differ (TICKET-044).
  - L-656: a one-line editor in a `menu` key context takes Enter and Escape as `menu::Confirm` and
    `menu::Cancel`; keep the cancel handler's `cx.propagate()`.
  - #655's `LinkPicker` and `LinkDelegate` (a choice list matched with `fuzzy::match_strings`, a
    pick called back after a defer), and #658's hand-deployed `ContextMenu` on a right-click.
  - The brain (`brain ask`, consultation `2880862b3bc44885ac5b3cd5d5317ff1`): nothing on this seam;
    it listed 15 follow-ups due on other work, the backlog this form serves.
- **Discovery:**
  - Rusty at `13249a8`: `FollowUpParams` (`crates/rusty-mcp/src/main.rs:426-441`), the tool
    (`:1216-1231`, its success emitting `DataChanged` through `mutate`, `:793-801`), `follow_up`
    (`crates/rusty-core/src/brain/decisions.rs:336-407`) with its five refusals and its writes,
    `DecisionSummary`'s `followed_up` and `superseded_by` (`:19-33`), filled by `decision_summary`
    (`:419-456`: the property, else the page's last follow-up heading). `FOLLOW_UP_STATUSES` is
    `kept`, `revised`, `superseded`. A refusal is a JSON-RPC internal error with Rusty's words;
    `rusty::call_tool` strips "brain_follow_up failed: " from it (`rusty.rs:528-546`).
  - The stand-in's `due` (`stand_in/rusty-mcp:253-283`) sends `followed_up: False` and
    `superseded_by: None`, which Rusty never sends; its `set_property` and `remove_property`
    (`:324-349`) are what `brain_follow_up` will use. Slugs are `decisions/<name>`.
  - `decisions_tab.rs` (#659): rows built in `render_row` (`ListItem`, end slot of chip and
    lines, `on_click` to `page::open_later`); `read` is private.
  - `project.rs:477-660`: `OnPick`, `Choice`, `LinkPicker`, `LinkDelegate` (its `modal:
    WeakEntity<LinkPicker>`; `confirm` dismisses, then defers the pick).
  - `ui::ToggleButtonGroup::single_row` and `ToggleButtonSimple`
    (`crates/ui/src/components/button/toggle_button.rs:62-190`); `ButtonLike`'s click stops its
    propagation (`button_like.rs:843`), so a link in a row's end slot does not open the row.
  - The auto-height editor keys (`assets/keymaps/default-linux.json:165-170`).
  - Rusty's app (`crates/rusty-app/qml/DecisionsPage.qml`, 90 lines) records nothing.

### Design
- **`marley_rusty::decisions`** (Marley crate): `BRAIN_FOLLOW_UP`; `DecisionSummary` gains
  `followed_up: String` and `superseded_by: String` (`serde(default)`), with `followed_up_line()`
  ("followed up DAY", none when empty) and `successor()` (none when empty);
  `FollowUpStatus { Kept, Revised, Superseded }` with its word and its button label;
  `FollowUpDraft { status: Option<FollowUpStatus>, outcome, day, successor: Option<String> }` with
  `missing() -> Option<Missing>` (`Status`, `Outcome`, `Successor`, each with its hint line) and
  `arguments(slug) -> Option<serde_json::Value>` (the outcome trimmed; `follow_up_by` only for
  Revised with a day; `successor` only for Superseded). Pure: no gpui, no IO.
- **The stand-in** (`stand_in/rusty-mcp`): `brain_follow_up` in `VAULT_TOOLS`, with Rusty's
  refusals in Rusty's order and words, the section added before `\n## Timeline` or at the end,
  `status`, `followed_up` from `today_in_state`, `follow_up_by` set or removed, `superseded_by`;
  `due` serving `followed_up` (the property, else the last `### Follow-up DAY:` heading) and
  `superseded_by` as strings. Its vault watcher announces the write.
- **`rusty/project.rs`** (Marley): `LinkDelegate` loses its `WeakEntity<LinkPicker>` for two
  callbacks, `on_pick` and `on_close`, and whether a pick also closes. `LinkPicker` passes its own
  dismissal; the follow-up form passes "back to the form". `Choice`, `OnPick` and the delegate
  become `pub(super)`.
- **`rusty/follow_up.rs`** (new, Marley): `pub(super) fn open(workspace, tab, summary, candidates,
  window, cx)` calls `toggle_modal`. `FollowUpModal { tab: WeakEntity<BrainDecisionsView>, slug,
  title, follow_up_line, draft, outcome: Entity<Editor> (auto-height 3 to 8), day: Entity<Editor>
  (single line), successor_title, choosing: Option<Entity<Picker<LinkDelegate>>>, recording:
  Option<Task<()>>, refusal: Option<SharedString>, focus_handle }`. Render: the title "Follow up:
  TITLE", the decision's follow-up line, the toggle row, the outcome, the day (Revised), Replaced
  by and its Choose… button (Superseded), the refusal, then the hint or Record. Key context
  `RustyFollowUp menu`; `menu::Confirm` records when `draft.missing()` is none; `menu::Cancel`
  emits `DismissEvent`. A status click sets the draft and focuses the outcome. Record reads the
  editors into the draft, sends `call_tool(BRAIN_FOLLOW_UP, arguments)`, and on `Ok` asks the tab
  to read and emits `DismissEvent`; on `Err` keeps the first line as the refusal.
- **`rusty/decisions_tab.rs`** (Marley): `read` becomes `pub(super)`; a Follow Up `Button` at the
  end of each Due row while connected; `on_secondary_mouse_down` deploys a `ContextMenu` (Follow
  Up… unless superseded, then Open Page); the decided line followed by `followed_up_line`; on a
  superseded row with a successor, "replaced by" and a link-styled `Button` with its title that
  opens the successor's page; `open_follow_up(section, index)` hands `follow_up::open` the summary
  and the `all` list as `Choice`s (title, slug as the detail) without the decision itself.
- **`rusty.rs`:** `mod follow_up;`.
- **`guide/index.html`:** the Decisions article gains Follow Up.
- **File manifest:**
  - `crates/marley_rusty/src/decisions.rs` — Marley crate.
  - `crates/marley_rusty/stand_in/rusty-mcp` — Marley crate (fixture program).
  - `crates/marley_workbench/src/rusty/follow_up.rs` — Marley crate, new.
  - `crates/marley_workbench/src/rusty/decisions_tab.rs` — Marley crate.
  - `crates/marley_workbench/src/rusty/project.rs` — Marley crate.
  - `crates/marley_workbench/src/rusty.rs` — Marley crate.
  - `crates/marley_workbench/guide/index.html` — Marley crate.
  - `script/e2e/660-rusty-decision-follow-up.sh` — the scenario (Test).
  - No Zed crate: no `zed-touchpoints.md` row.

### Visual check plan
The scenario `script/e2e/660-rusty-decision-follow-up.sh`, `compositor sway`, set up as the spec's
UI proof says. The Brain view reached by the rail header's switch and the Decisions tab by its
fixed-row entry, as #659's scenario does; positions measured from the first shots.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 014, 015 | Open the Decisions tab | `660-01-rows` |
| REQ-016 | Click "replaced by Split the crate" | `660-02-successor-opened` |
| REQ-004 | Back to Decisions (its tab); click Follow Up on Try the stand-in | `660-03-form` |
| REQ-005, 006 | Click Revised; type a line, Shift+Enter, a second line | `660-04-revised` |
| REQ-012 | Click the day field, type "soon", Enter; `expect` the page unchanged | `660-05-refused` |
| REQ-009, 011 | Select the day, type 2026-10-24, Enter; `expect` the logged call | `660-06-recorded` |
| REQ-013, 005 | Follow Up on Ship the rail switch, Kept, type, Escape; `expect` no new call | `660-07-escape` |
| REQ-009, 011 | Follow Up again, Kept, an outcome, click Record | `660-08-kept` |
| REQ-002 | Right-click Split the crate | `660-09-menu` |
| REQ-003 | Escape; right-click One big crate | `660-10-superseded-menu` |
| REQ-007 | Escape; right-click Batch tickets by five, Follow Up…, Superseded, type "split" | `660-11-successor-picker` |
| REQ-008 | Enter | `660-12-successor-chosen` |
| REQ-009, 011 | Type an outcome, Enter; `expect` the logged successor | `660-13-superseded` |
| REQ-010, 017, 018 | Not shot | Review: Record's state during the call; the connection checks; no date work |

Not reached by a shot: Record's Recording… state (the stand-in answers within milliseconds) and
the not-connected state (the keeper reconnects within a second, as #659 found). The review reads
both.

### Risks
- **Rusty's two calendars (TICKET-044).** `followed_up` is Rusty's UTC day; from 19:00 to midnight
  on this box a follow-up reads tomorrow's day. Shown as served.
- **Escape during a call** closes the form but the call goes on; Rusty may record it. The tab reads
  on Rusty's announcement (embedded) or at its next read, so the row catches up; the refusal of
  such a call is not shown.
- **The embedded picker's keys.** Up, Down, Enter and Escape must stay the picker's while it shows:
  its key context is deeper than the form's `menu`, so its handlers run first. Escape returns to
  the form rather than closing it (D6); a second Escape closes the form.
- **The toggle buttons take the focus** on a click; the form puts it back on the outcome so Enter
  keeps working (D3).
- **Rusty's own refusal words** may change; the form shows whatever Rusty says.
- **About 240 decisions** in Chad's brain make the successor picker's list long; `fuzzy` matches
  it in the background as #655's does.

### Checklist (no TaskCreate in this harness)
- [x] Pick: TICKET-660, the Queue's top.
- [x] Pre-flight green.
- [x] Recall: the ledgers, #655, #656, #658 and #659, the brain.
- [x] Mint the pair (id `0092d987-cbb6-4e11-b227-3c53fea6e964`); BACKLOG row removed; ticket in
      progress.
- [x] Prior-art sweep, three legs.
- [x] Spec: scope, Reference, Prior art, UI proof, D1 to D11, 18 EARS rows, phase plan.
- [x] Design: approach, manifest, visual check plan, risks.
- [x] Phase 1 PASS: the work runs autonomously under Chad's goal.

## Phase 2 — Code (2026-10-05)
- **Built:**
  - `marley_rusty::decisions`: `BRAIN_FOLLOW_UP`; the summary's `followed_up` and
    `superseded_by` with `followed_up_line` and `successor`; `FollowUpStatus` (`ALL`, `word`,
    `label`); `Missing` with its hint; `FollowUpDraft` with `missing` and `arguments` (the
    outcome trimmed, the day only for Revised and given, the successor only for Superseded),
    built from a field list since `Map::insert`'s discarded results trip `unused_results`.
  - The stand-in: `brain_follow_up` (Rusty's five refusals in its order and words, the section
    before `## Timeline` or at the end, `status`, `followed_up`, `follow_up_by` set or removed,
    `superseded_by`; Rusty's timeline line left out), listed in `TOOL_NAMES`; `brain_due` serving
    `followed_up` (the property, else the last follow-up heading) and `superseded_by` as strings.
    A smoke run in the scratchpad (stdio, a scratch state) showed the refusals, the writes, the
    two fields and the announcement.
  - `rusty/project.rs`: `LinkDelegate` takes `on_pick`, `on_close` and whether a pick closes;
    `LinkPicker` passes its own dismissal. `Choice`, `OnPick`, `OnClose` and the delegate are
    `pub(super)`.
  - `rusty/follow_up.rs` (new): `open` and `FollowUpModal` (status toggles starting unchosen,
    the auto-height outcome, the day for Revised, Replaced by with an embedded successor picker
    for Superseded, the refusal line, the hint and Record; `RustyFollowUp menu` for Enter and
    Escape; a refocus flag puts the focus back on the outcome when the picker closes).
  - `rusty/decisions_tab.rs`: `read_again`; `render_end_slot` (the chip, decided, followed up,
    follow up by, "replaced by" with the successor's title as a link, Follow Up on Due rows while
    connected); `render_successor`; `open_follow_up` (deferred into the workspace's modal
    layer); the right-click `ContextMenu` (`MenuChoice::FollowUp`, `OpenPage`) as the Tasks
    tab's; `shown_title` and `section_word`.
  - `rusty.rs`: `mod follow_up`. The guide page's Decisions article gains the follow-up steps.
- **Deviations:**
  - REQ-007's picker shows when Superseded is chosen with no successor; Escape in it goes back to
    the form with "none chosen" and a Choose… button (D6), so "WHILE … the form shall show a
    picker" reads "WHEN Superseded is chosen with no successor, the form shall show a picker".
  - The successor picker sits inside the form under Replaced by, the outcome below it, rather
    than in the form's place: the outcome typed before the choice stays in view.
- **Review:**
  - The form watched the whole outcome editor (`observe`), so every caret blink drew the form
    again: now it subscribes to `BufferEdited` only.
  - Re-entrancy: the picker's pick and close reach the form through `cx.defer`, after the
    picker's update; the tab's form opens through `window.defer`.
  - A successor link and the Follow Up button are `Button`s, whose click stops at them
    (`ButtonLike`), so the row's own page does not open.
  - No date is worked out: the day goes as typed, and Rusty's refusal names a bad one.
- **Gate:** the first `just gate-diff` was red on gate:21 (dylint's
  `shared_string_from_str_literal`, a `"none chosen".into()`); fixed with
  `SharedString::new_static`. The second run is GREEN, 17 of 17.

## Phase 3 — Test (2026-10-05)
- **Scenario:** `script/e2e/660-rusty-decision-follow-up.sh` under `compositor sway`, against the
  debug `marley`. The final run exited 0 with its six checks passing: "the refused follow-up wrote
  nothing", "one brain_follow_up revised to 2026-10-24 with two lines", "the outcome kept its line
  break", "no brain_follow_up after Escape", "Ship the rail switch kept, with no day", "the
  successor was sent for Batch tickets by five". The stand-in's log holds exactly four
  `brain_follow_up` calls: the refused "soon", the revised one with `follow_up_by` 2026-10-24 and
  the outcome's `\n`, the kept one with no day, and the superseded one with
  `decisions/split-the-crate`.
- **Runs:** three. The first measured the positions; the guesses from #659's layout held except
  Record under a Kept form (y 306, not 330: the click fell outside the form, which closed it
  unsent, so the menu steps then ran against the wrong rows). The pointer now leaves the rows
  before the form and menu shots, whose tooltips covered them.
- **Fixed in Test (source):**
  - The status toggles used the group's default transparent style, so the unchosen three read as
    plain words. Every other Zed use of `ToggleButtonGroup` is `Outlined`; the form is too.
  - The successor picker opened at Zed's modal width, 34 rem, inside a form of that width with
    0.75 rem padding a side, so its selected row ran about 10 px past the form's edge. It opens at
    the form's inner width now (`FORM_WIDTH - 1.5`).
  - Both were linted (`just clippy marley_workbench` clean); the full gate runs again at Complete.
- **Shots**, each read (final run):
  - `660-01-rows` (REQ-001, 014, 015): Due holds Try the stand-in (overdue, warning colour) and Ship
    the rail switch, each with Follow Up at its end; Batch tickets by five reads "followed up
    2026-09-26"; One big crate reads "replaced by" with Split the crate in the accent colour.
  - `660-02-successor-opened` (REQ-016): Split the crate's page in a kept Page tab in front.
  - `660-03-form` (REQ-004): "Follow up: Try the stand-in", "follow up by 2026-09-27 · overdue",
    the outlined Kept, Revised and Superseded with none lit, the outcome's placeholder, Record
    muted, "Choose kept, revised or superseded." The focus is the outcome's: the next step types
    into it without a click.
  - `660-04-revised` (REQ-005, 006): Revised lit; the outcome on two lines; Next follow-up with its
    placeholder; Record enabled, no hint.
  - `660-05-refused` (REQ-012): "follow_up_by is not a date (YYYY-MM-DD): soon" in the error colour
    above Record, the outcome and "soon" as typed; the page's checksum unchanged.
  - `660-06-recorded` (REQ-009, 011): the form gone; Due holds Ship the rail switch alone; Try the
    stand-in reads revised, followed up 2026-10-03, follow up by 2026-10-24.
  - `660-07-escape` (REQ-013): the form gone after Kept and typed words; the rows as in 06; no new
    call.
  - `660-08-kept` (REQ-009, 011): no Due section; Ship the rail switch reads kept, followed up
    2026-10-03, with no follow-up day. One big crate's row is lit by a hover left from the Record
    click, a gpui hover state the closing modal did not clear; nothing else shows it.
  - `660-09-menu` (REQ-002): Split the crate's menu, Follow Up… (selected) and Open Page.
  - `660-10-superseded-menu` (REQ-003): One big crate's menu, Open Page alone.
  - `660-11-successor-picker` (REQ-007): "Follow up: Batch tickets by five", Superseded lit,
    Replaced by with "split" typed: Split the crate (selected) and Ship the rail switch, each with
    its slug; Batch tickets by five not among them; the picker within the form's edges; the
    outcome below; "Write how it went."
  - `660-12-successor-chosen` (REQ-008): "Replaced by Split the crate" and Change…; the form open;
    Record muted with "Write how it went."
  - `660-13-superseded` (REQ-009, 011): Batch tickets by five reads superseded, followed up
    2026-10-03, replaced by Split the crate.
- **Not reached by a shot:** REQ-010 (Record's Recording… lasts the stand-in's few milliseconds)
  and REQ-017's not-connected half (the keeper reconnects within a second, as #659 found), by
  review; REQ-018 by review: the day goes as typed and every day shown is Rusty's.
- **Focus:** the keys reached the form's fields, the picker and the menus; Escape closed the form
  and the menus.

## Phase 4 — Complete (2026-10-05)
- **Documented:**
  - `CHANGELOG.md`: Added (a decision's follow-up from the Decisions tab).
  - `docs/marley/guide.md`: the Decisions tab's rows (the last follow-up, the successor), opening
    the successor, the Following up bullets; the palette row.
  - `docs/marley/walkthrough.md`: stop 2.15d (it writes to the brain, so it says so first); the
    palette row.
  - `docs/marley_architecture/marley_workbench.md`: the shared `LinkDelegate` in the project-view
    section; the Decisions tab's rows since #660; a follow-up form section.
  - `docs/marley_architecture/marley_rusty.md`: the `decisions` module's #660 types; the
    stand-in's `brain_follow_up` and the two fields.
  - `docs/marley/rusty-in-marley.md`: R7b shipped; the batch paragraph. `three-prong-plan.md`:
    C2 gains #660.
  - No Zed path: no `zed-touchpoints.md` row.
- **Knowledge:** F-claude-660-the-successor-picker-ran-past-the-follow-up-forms-edge-001,
  F-claude-660-the-follow-up-form-drew-again-on-every-caret-blink-001,
  L-claude-660-a-picker-inside-a-form-opens-at-the-modal-width-001,
  L-claude-660-zeds-toggle-group-reads-as-a-choice-only-outlined-001,
  L-claude-660-a-missed-click-outside-a-zed-modal-closes-it-silently-001,
  AD-claude-660-a-follow-up-is-a-form-over-one-brain-follow-up-call-001.
- **Brain:** `brain decide` on consultation `2880862b3bc44885ac5b3cd5d5317ff1`, follow up by
  2026-10-19: `decisions/marley-records-a-decisions-follow-up-through-a-form-over-one-brain-follow-up-call`.
- **Closed:** the ticket in `tickets/closed/`; no BACKLOG row (removed at promotion); this pair in
  `completed/`.
- **Gate:** run again on the final tree, since the source changed in Test.
