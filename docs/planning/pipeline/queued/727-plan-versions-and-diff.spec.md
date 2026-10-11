---
pipeline_id: 0c6a656b-861b-465f-b852-d1cab92210e3
ticket: docs/planning/tickets/open/TICKET-727-plan-versions-and-diff.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Plan versions and a diff between them
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 2; docs/planning/design-notes/finish-line-2026-10-11.md row 10); after #726 slice 1
references:
  - docs/planning/pipeline/queued/726-plan-review-in-a-tab.spec.md
---

## Title
The plan tab (#726) keeps every plan a session sends. From the second one on, its header says which
version it shows and how many lines were added and removed since the last, and Changes shows the
diff, so a revised plan needs no second full reading. Chad, 2026-10-11, on the finish-line list:
"10.) yes".

## Scope
### In
- The tab keeps each plan its session sends, numbered from 1, for as long as the tab is open.
- From version 2 on, the header shows "Version N" and a `+A −R` count of the lines added and
  removed since version N−1.
- **Changes**, beside the count, turns the plan into a read-only diff of version N−1 against
  version N, as Zed draws a diff (removed and added lines in their colors); **Plan** turns it back.
- Notes are made on the plan view only; the diff takes none.

### Out (explicitly deferred)
- A rendered diff that marks changed blocks in the drawn Markdown, and a note on a changed block
  that says so.
- Comparing with a version other than the one before, and reading an earlier version on its own.
- Keeping versions after the tab closes or Marley quits.

## Reference (§20)
N/A — Marley-specific: the plan tab is Marley's (#726). The behavior follows plannotator's version
history (published docs, `docs.plannotator.ai/open-source/workflows/version-history`): a revised
plan from the same source joins its history, the review shows a count of added and removed lines,
and opening it compares the current plan with the previous one, rendered or raw. Marley keeps the
raw view, drawn by Zed's own diff editor. No plannotator code is taken.

### Prior art
- **The code we ship:**
  - `acp_thread::Diff::finalized(path, old_text, new_text, language_registry, cx)`
    (`crates/acp_thread/src/diff.rs:282`): a read-only multibuffer with a `BufferDiff` whose base is
    the old text, built without git; `multibuffer()` (421). Agent threads draw it in an editor made
    by `create_multibuffer_diff_editor` (`crates/agent_ui/src/entry_view_state.rs:755-790`:
    gutter off, read-only, `set_expand_all_diff_hunks`), the settings to repeat in Marley.
  - `buffer_diff::BufferDiff::set_base_text` (`crates/buffer_diff/src/buffer_diff.rs:2234`, "Used
    in cases where the change set isn't derived from git") and `changed_row_counts` (2203).
  - `language::text_diff::line_diff(old, new)` (`crates/language/src/text_diff.rs:158`) for the
    count without an entity.
  - `ui::DiffStat::new(id, added, removed)` (`crates/ui/src/components/diff_stat.rs:15`), which the
    rail already draws (`crates/marley_workbench/src/rail.rs:4929`).
  - `git_ui::TextDiffView` (`crates/git_ui/src/text_diff_view.rs:45, 165`) needs a source editor and
    titles itself "Clipboard ↔ …": not a fit.
- **Behavior maps:** none cover plan versions; `docs/zed_architecture/subsystems/03-editor-multibuffer.md`
  for the diff editor.
- **Published material:** plannotator's version history (above).

## UI proof
`script/e2e/727-plan-versions-and-diff.sh`, under `compositor sway`, on #726's set-up: the stand-in
`claude` sends a three-step plan; the scenario writes a note and sends feedback; the stand-in sends
version 2, with one step reworded and one added.

Shots:
- `727-01-version`: the tab's header with "Version 2" and `+2 −1`.
- `727-02-changes`: Changes clicked: the diff with the reworded step's old line removed and the new
  lines added.
- `727-03-plan`: Plan clicked: version 2 drawn as Markdown again, Approve and Send Feedback on.

## Locked-In Decisions
- **D1:** versions belong to the tab, which belongs to one Claude Code session (#726), so a plan's
  history is its session's. Rejected: plannotator's key of project, first heading and date (two
  sessions on one plan heading would share a history).
- **D2:** the diff is Zed's own: `acp_thread::Diff::finalized` and a read-only editor set up as
  agent threads set theirs. Rejected: a rendered diff of Markdown blocks (a second drawing of the
  plan to keep in step with the first); `TextDiffView` (bound to a source editor).
- **D3:** in memory only. A plan's review lasts minutes, and a restart ends the waiting call.
- **D4:** the count compares lines, as the diff does, so the count and the diff agree.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a session sends a plan after an earlier one, the plan tab shall show "Version N" and the count of lines added and removed since version N−1. | Shot 727-01 |
| REQ-002 | WHEN the user chooses Changes, the system shall show a read-only diff of version N−1 against version N. | Shot 727-02 |
| REQ-003 | WHEN the user chooses Plan, the system shall show version N as Markdown, with its notes and answers. | Shot 727-03 |
| REQ-004 | WHILE the tab shows version 1, the system shall show neither the count nor Changes. | Review of the diff (shot 726-02 shows version 1) |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `plan_review.rs`: the versions, the count, the Changes view and its editor; the
  guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
