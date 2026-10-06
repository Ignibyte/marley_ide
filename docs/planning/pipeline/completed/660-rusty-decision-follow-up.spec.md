---
pipeline_id: 0092d987-cbb6-4e11-b227-3c53fea6e964
ticket: docs/planning/tickets/closed/TICKET-660-rusty-decision-follow-up.md
status: Phase 4 — Complete PASS
title: "Record a decision's follow-up from the Decisions tab"
type: feature
slice: Rusty in Marley R7b (docs/marley/rusty-in-marley.md), prong C2 of docs/marley/three-prong-plan.md
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/659-rusty-decisions-tab.spec.md]
---

## Title
Record a decision's follow-up from the Decisions tab. #659's tab only reads, and Chad's brain holds
about 15 overdue follow-ups that only an agent's `brain_follow_up` or `rusty-cli brain follow-up`
can record. A Follow Up form on a decision row sends Rusty's `brain_follow_up` (status, outcome,
a new day when revised, the successor when superseded), and each row shows what Rusty already
serves and #659 left out: the last follow-up's day and, for a superseded decision, the decision
that replaced it.

## Scope
### In
- **The form.** Follow Up on a decision row opens a form in the workspace's modal layer, named for
  the decision: the status (Kept, Revised, Superseded, none chosen at first), the outcome (a
  field of several lines), the next follow-up day when Revised, the decision that replaced it when
  Superseded (a picker over the decisions the tab holds), and Record. Enter records a ready form,
  Shift+Enter breaks a line in the outcome, Escape closes it without a call.
- **The write.** One `brain_follow_up { slug, outcome, status, successor, follow_up_by }` call.
  A refusal shows Rusty's words in the form with every field kept; a success closes the form and
  the tab reads `brain_due` again.
- **Where it is offered.** A Follow Up button on each Due row, and Follow Up… in a right-click
  menu on every row that is not superseded, beside Open Page; neither while Rusty is off or not
  connected.
- **The rows.** "followed up DAY" from Rusty's `followed_up`; on a superseded row, "replaced by"
  and the successor's title from `superseded_by`, a link that opens its page.
- **`marley_rusty::decisions`:** the summary's `followed_up` and `superseded_by`; the follow-up's
  status, its draft and what a draft still lacks, and the call's arguments.
- **The stand-in:** `brain_follow_up` with Rusty's checks and writes, and `brain_due` serving
  `followed_up` and `superseded_by`.
- The in-app guide page's Decisions article.

### Out (explicitly deferred)
- **A follow-up on a superseded decision.** Rusty takes one, but a replaced decision's follow-up
  belongs to its successor (D10).
- **Recording a new decision (`brain_decide`) from Marley.** The loop starts with `brain_ask` in an
  agent's session; a decision written by hand skips the consultation.
- **Editing or removing a past follow-up.** Rusty has no tool for it; the page's Edit (#645) shows
  the file.
- **A keyboard path into the form from the tab.** The tab's rows have no selection yet (#659); the
  form itself is keyboard-ready once open.
- **Dates computed in Marley.** `due`, `overdue` and `followed_up` stay Rusty's (#659's D5,
  Rusty's TICKET-044).

## Reference (§20)
N/A — Marley-specific. Rusty's own app (`crates/rusty-app/qml/DecisionsPage.qml` in Rusty's
repository at `13249a8`, read 2026-10-05) lists decisions and records nothing; its only follow-up
paths are `brain_follow_up` from an agent and `rusty-cli brain follow-up`. The form's parts are
Zed's: `ui::ToggleButtonGroup` (the debugger's new-process modal picks among three the same way),
an auto-height `Editor` as the git commit modal's message field, and `picker::Picker` with
`fuzzy::match_strings` as #655's Link a Page.

### Prior art
- **Behaviour maps:** `docs/zed_architecture/` (modals through `Workspace::toggle_modal` and
  `ModalView`); nothing in `docs/warp_architecture/` or `docs/orca_architecture/` records a
  decision log. Warp has no counterpart.
- **Published material:** Rusty's tool contract (`crates/rusty-mcp/src/main.rs:426-441` and
  `:1216-1231` at `13249a8`): `brain_follow_up` takes `slug`, `outcome`, `status` (`kept`,
  `revised` or `superseded`), `successor` (a `decisions/` slug) and `follow_up_by` ("a new
  follow-up date when revised (ISO date); cleared otherwise"). `follow_up`
  (`crates/rusty-core/src/brain/decisions.rs:336-407`) refuses a status outside the three, an empty
  outcome, a day not `YYYY-MM-DD`, a missing or non-decision page, and superseded without a
  successor; then it adds `### Follow-up DAY: status` and the outcome before `## Timeline`, sets
  `status` and `followed_up`, sets or removes `follow_up_by`, sets `superseded_by`, and adds a
  timeline line. Its success emits `DataChanged` (`mutate`, `:793-801`).
- **The code we ship:** `ui::ToggleButtonGroup::single_row` (`crates/ui/src/components/button/
  toggle_button.rs:171-190`), used by `debugger_ui/src/new_process_modal.rs:632`; the auto-height
  editor's keys (`assets/keymaps/default-linux.json:165-170`: Shift+Enter and Ctrl+Enter are
  `editor::Newline`, and Enter is not bound in that mode, so it reaches `menu::Confirm`, as L-656
  found for single-line editors); `picker::Picker` and #655's `LinkDelegate`
  (`crates/marley_workbench/src/rusty/project.rs:477-660`), which matches choices with
  `fuzzy::match_strings` and calls a pick back, reused for the successor rather than written
  again; the Tasks tab's hand-deployed `ContextMenu` (`rusty/tasks_tab.rs:775-800`). No crate we
  build owns a decision follow-up.

## UI proof
`script/e2e/660-rusty-decision-follow-up.sh` (`compositor sway`: it clicks rows, buttons, a menu
and the form). Setup as #659's: the scratch repository opened with `open_path`; a scratch vault
under `$E2E_WORK/rusty/vault` with the stand-in named in `MARLEY_RUSTY_MCP`, its day fixed at
2026-10-03 by the state folder's `today`; `marley.rusty` on with the embedded connection. Decision
pages: `try-the-stand-in` (decided 2026-09-20, follow up by 2026-09-27), `ship-the-rail-switch`
(decided 2026-09-30, follow up by 2026-10-03), `batch-tickets-by-five` (revised, decided
2026-09-12, follow up by 2026-10-10, `followed_up: 2026-09-26`), `split-the-crate` (decided
2026-09-02) and `one-big-crate` (superseded, decided 2026-09-01, `superseded_by:
decisions/split-the-crate`). Never the user's brain (R-D8). Shots:
- `660-01-rows`: the Decisions tab: Follow Up on the two Due rows; Batch tickets by five reads
  "followed up 2026-09-26"; One big crate reads "replaced by Split the crate".
- `660-02-successor-opened`: "replaced by Split the crate" clicked: Split the crate's page in front.
- `660-03-form`: back on Decisions, Follow Up on Try the stand-in: the form "Follow up: Try the
  stand-in" with its follow-up line, Kept, Revised and Superseded, none chosen, the outcome field
  with the caret, Record disabled with its hint.
- `660-04-revised`: Revised chosen and an outcome typed over two lines (Shift+Enter): the Next
  follow-up field below, Record enabled.
- `660-05-refused`: "soon" typed as the day and Enter: Rusty's "follow_up_by is not a date
  (YYYY-MM-DD): soon" in the error colour, every field as typed; the page file unchanged (an
  `expect`).
- `660-06-recorded`: the day changed to 2026-10-24 and Enter: the form gone; Try the stand-in
  revised, follow up by 2026-10-24, followed up 2026-10-03, no longer under Due; the stand-in's log
  holds one `brain_follow_up` with that slug, status, day and the two-line outcome (an `expect`).
- `660-07-escape`: Follow Up on Ship the rail switch, Kept chosen, words typed, Escape: the form
  gone, no new `brain_follow_up` (an `expect`), the row as it was.
- `660-08-kept`: Follow Up on Ship the rail switch again, Kept, an outcome, Record clicked: kept,
  no follow-up day, followed up 2026-10-03; Due is gone.
- `660-09-menu`: a right-click on Split the crate: Follow Up… and Open Page.
- `660-10-superseded-menu`: a right-click on One big crate: Open Page only.
- `660-11-successor-picker`: Follow Up… on Batch tickets by five, Superseded chosen: the Replaced
  by picker listing the other decisions, Batch tickets by five not among them; "split" typed.
- `660-12-successor-chosen`: Enter in the picker: "Replaced by Split the crate" in the form, the
  form still open, Record disabled until an outcome is typed.
- `660-13-superseded`: an outcome and Enter: Batch tickets by five superseded, replaced by Split
  the crate, followed up 2026-10-03.

## Locked-In Decisions
- D1 — **A form in the modal layer**, `FollowUpModal` in a new `rusty/follow_up.rs`, opened from
  the tab with the decision's summary and the tab's decisions. Rejected: a form unfolding under the
  row (the column scrolls and a long list reflows around it); a hover slot (Zed's `ListItem`
  `end_hover_slot` hides the dates while hovered); a palette action (it would need a decision
  picker first).
- D2 — **Offered on Due rows as a button, on every row not superseded by right-click.** The Due
  rows are the ones waiting; the menu reaches a decision before its day. The menu also holds Open
  Page. Rejected: a button on all ~240 rows of Chad's brain.
- D3 — **The status starts unchosen**, three toggles of `ToggleButtonGroup`; Record waits for one.
  A wrong default would write to the brain. The toggle's click gives the focus back to the outcome
  field, so Enter still records.
- D4 — **The outcome is an auto-height editor** (3 to 8 lines). Enter records a ready form
  (`menu::Confirm` through a `menu` key context, L-656); Shift+Enter and Ctrl+Enter are Zed's own
  newlines for that mode. Escape closes the form without a call (`menu::Cancel`; the modal layer's
  dismissal). Sent trimmed; Rusty refuses an empty one, and Record waits for words.
- D5 — **A day only for Revised**, as Rusty's tool documents ("a new follow-up date when revised;
  cleared otherwise"): a single-line field, empty for no further follow-up. Kept and Superseded send
  none, so Rusty clears the day and the decision leaves Due. Marley does not check the day's form:
  Rusty's refusal names it (no date work in Marley, #659's D5).
- D6 — **The successor is picked**, from the decisions the tab holds (`all`, without this one),
  matched by title and slug, in a picker that takes the form's place until a pick or Escape, built
  on #655's `LinkDelegate` made shareable (its dismissal and its pick given as callbacks). Escape
  in the picker returns to the form. Rejected: a typed slug; a second modal (the workspace holds
  one, and opening it closes the form).
- D7 — **One call, read back.** `brain_follow_up` through `rusty::call_tool`; while it runs Record
  reads Recording… and does nothing more. A refusal's first line shows in the form in the error
  colour, fields kept. A success closes the form and asks the tab to read (with the service
  connection Rusty announces nothing, #647's D9). Escape during a call closes the form; the call is
  not withdrawn, and the tab shows Rusty's answer at its next read.
- D8 — **The rows draw Rusty's two fields as served:** "followed up DAY" (muted, after the decided
  line) when `followed_up` is set; on a superseded row with `superseded_by`, "replaced by" and the
  successor's title from the tab's list (its slug when the list lacks it), a link that opens the
  page and does not open the row's own.
- D9 — **Not offered while Rusty is off or not connected**: no button, no menu entry. A form left
  open when the link drops answers Record with Rusty's "not connected" in the form.
- D10 — **No follow-up on a superseded row.** Rusty takes one, but a replaced decision's next step
  is its successor's; its menu holds Open Page alone.
- D11 — **The stand-in does what Rusty does**: `brain_follow_up`'s checks in Rusty's order with
  Rusty's words, the section before `## Timeline` or at the end, `status`, `followed_up` (the
  stand-in's day), `follow_up_by` set or removed, `superseded_by`; `brain_due` serving
  `followed_up` (the property, else the last `### Follow-up DAY:` heading) and `superseded_by` as
  strings, empty when none, as Rusty's summary does.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is connected, each row under Due shall show a Follow Up button. | Shot `660-01-rows` |
| REQ-002 | WHEN a row that is not superseded is right-clicked, the system shall show a menu with Follow Up… and Open Page. | Shot `660-09-menu` |
| REQ-003 | WHEN a superseded row is right-clicked, the system shall show a menu with Open Page and no Follow Up…. | Shot `660-10-superseded-menu` |
| REQ-004 | WHEN Follow Up is chosen, the system shall open a form titled with the decision's title, showing Kept, Revised and Superseded with none chosen, an outcome field with the focus, and Record disabled with a hint naming what is missing. | Shot `660-03-form` |
| REQ-005 | WHILE Revised is chosen, the form shall show a Next follow-up day field; WHILE Kept or Superseded is chosen, it shall not. | Shots `660-04-revised`, `660-07-escape` |
| REQ-006 | WHEN Shift+Enter is pressed in the outcome field, the form shall break the line and record nothing. | Shot `660-04-revised`; the stand-in's log |
| REQ-007 | WHEN Superseded is chosen with no successor picked, the form shall show a picker over the tab's decisions other than this one. | Shot `660-11-successor-picker` |
| REQ-008 | WHEN a successor is picked, the form shall name it under Replaced by and stay open. | Shot `660-12-successor-chosen` |
| REQ-009 | WHEN Enter is pressed or Record clicked with a status, an outcome, and the successor when Superseded, the system shall send one `brain_follow_up` with the slug, the status, the outcome trimmed, the day when Revised and not empty, and the successor when Superseded. | The stand-in's log (`expect`); shots `660-06-recorded`, `660-08-kept`, `660-13-superseded` |
| REQ-010 | WHILE the call runs, Record shall read Recording… and send nothing more. | Review |
| REQ-011 | WHEN Rusty records the follow-up, the system shall close the form and the tab shall show the decision's new status, follow-up day or none, and "followed up DAY", moving it out of Due when it is no longer due. | Shots `660-06-recorded`, `660-08-kept`, `660-13-superseded` |
| REQ-012 | IF Rusty refuses the follow-up, THEN the form shall show Rusty's words and keep every field, and nothing shall be written. | Shot `660-05-refused`; the page file (`expect`) |
| REQ-013 | WHEN Escape is pressed in the form, the system shall close it without a call. | Shot `660-07-escape`; the stand-in's log (`expect`) |
| REQ-014 | WHERE Rusty serves a decision's `followed_up`, its row shall read "followed up" and that day. | Shot `660-01-rows` |
| REQ-015 | WHERE Rusty serves a superseded decision's `superseded_by`, its row shall read "replaced by" and the successor's title. | Shot `660-01-rows` |
| REQ-016 | WHEN the successor's title on a row is clicked, the system shall open the successor's page. | Shot `660-02-successor-opened` |
| REQ-017 | WHILE Rusty is off or not connected, the tab shall offer no Follow Up. | Review (off drops the rows, #659; not connected cannot be held for a shot) |
| REQ-018 | The tab and the form shall draw Rusty's status, days, `overdue` and `followed_up` as served and work out no date. | Review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::decisions` (the two fields, `FollowUpStatus`, the draft and its
  arguments); the stand-in's `brain_follow_up` and the two fields; #655's `LinkDelegate` made
  shareable; `rusty/follow_up.rs`; the tab's button, menu, row lines and read; the guide page; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), the user docs, R7b in the plan, ledger
  capture (§19), the brain decision, close the ticket, archive, commit.
