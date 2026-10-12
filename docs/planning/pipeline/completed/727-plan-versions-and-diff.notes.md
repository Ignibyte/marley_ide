# Plan versions and a diff between them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-727-plan-versions-and-diff.md
- **Pipeline spec:** 727-plan-versions-and-diff.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 2 (`docs/planning/intake/plannotator-findings.md`): "Keep each
  resubmitted plan, badge it +N/-M against the last, and show the rendered diff, so a revised plan
  needs no re-reading." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "10.) yes". It waits under Deliberate
  with the rest of the review batch until after the finish.
- **Classification:** feature, small. Marley crate only; no Zed touch.
- **Deviation from the ticket:** "the rendered diff" becomes Zed's raw diff of the Markdown (D2).
  plannotator offers both; the raw one is Zed's own drawing and agrees with the count.
- **Recall (§18.3):**
  - #726's spec and notes: the tab, one per Claude Code session; a new plan replaces the tab's plan
    and clears its notes, which this ticket turns into a new version.
  - No ledger entry on in-memory diffs; agent threads draw `acp_thread::Diff` in a read-only editor
    (`entry_view_state.rs:755-790`).
- **Discovery:**
  - `crates/acp_thread/src/diff.rs`: `Diff::finalized` (282), `multibuffer` (421).
  - `crates/agent_ui/src/entry_view_state.rs`: `create_editor_diff` (747),
    `create_multibuffer_diff_editor` (755-790).
  - `crates/buffer_diff/src/buffer_diff.rs`: `changed_row_counts` (2203), `set_base_text` (2234).
  - `crates/language/src/text_diff.rs`: `line_diff` (158).
  - `crates/ui/src/components/diff_stat.rs:15`; the rail's use (`rail.rs:4929`).
  - `marley_workbench` already depends on `acp_thread`, `editor` and `git_ui`.
- **Decisions:** D1 to D4 in the spec.

### Design
- **`crates/marley_workbench/src/plan_review.rs`** (Marley crate, made by #726):
  - `PlanReview.versions: Vec<PlanVersion { text, arrived }>`; `arrived` pushes a version instead of
    replacing the text; the notes belong to the latest.
  - The header: "Version N" and `DiffStat` from `line_diff(previous, latest)` (counted once per
    version).
  - `view: PlanView { Plan, Changes }`; Changes builds once per version an `acp_thread::Diff`
    (`finalized`, the path `plan.md` so the Markdown language loads) and a read-only editor set up
    as `create_multibuffer_diff_editor` sets its own (the settings repeated; that function is
    private to `agent_ui`).
- **The guide:** the version count and Changes under "Plan review".
- **File manifest:** `plan_review.rs` (Marley crate); the guide; the scenario. No ledger rows.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Version 1 arrives; a note; Send Feedback; version 2 arrives | 727-01-version: "Version 2", `+2 −1` |
| 002 | Clicks Changes | 727-02-changes: the old step removed, the new lines added |
| 003 | Clicks Plan | 727-03-plan: version 2 as Markdown, the buttons on |
| 004 | (review of the diff; #726's 726-02 shows a first version without them) | — |

### Risks
- Claude Code may resend the same plan unchanged; the count reads `+0 −0` and Changes shows no
  hunks, which is true. A same-text resend could be folded into the last version instead; the
  Plan phase decides if a run shows it happening.
- `Diff::finalized` loads the language asynchronously; the Changes view draws plain text until it
  arrives.

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation bad21916cfd9405a8dc400ac1c3b35ba) had nothing on the seam.
- **Seams re-verified** directly at `a1f6bfb8f1`: the plan tab is #726's `PlanReview`
  (`plan_review.rs`, shipped today), one per terminal, whose `show` takes each plan;
  `acp_thread::Diff::finalized(path, old, new, language_registry, cx)` (`diff.rs:282`) and
  `multibuffer()` (421); the agent thread's diff editor settings
  (`agent_ui/src/entry_view_state.rs:755-790`), of which the tab takes the read-only, no-gutter,
  all-hunks-expanded ones and keeps its own scrolling (the thread's sizes to content and forbids
  scrolling); `language::text_diff::line_diff` (`text_diff.rs:158`) for the count; `ui::DiffStat`.
- **Design:** `PlanReview` keeps `versions` (each plan's text, from 1); from the second, `show`
  counts lines added and removed from the one before (`line_diff`) and makes a `Diff::finalized`
  with the project's language registry and a read-only editor on its multibuffer; the column's
  header shows "Version N", the count and a Changes / Plan toggle; the left area draws the diff
  editor while Changes is on, else the Markdown. Notes are taken on the plan view only.

## Phase 2 — Code

- **Built:** in `plan_review.rs`, `versions`, `Changes`, `showing_changes`; `changes_from` (the
  count by `language::line_diff`, the diff by `acp_thread::Diff::finalized`, a read-only full-mode
  editor); `show` takes the language registry; the header's "Version N", `DiffStat` and the
  Changes / Plan toggle; the left area swaps to the diff editor in its own container.
- **Deviations:** the registry is passed in from `open_tab` (the fix below), and the tab no longer
  keeps a workspace handle.
- **Clippy:** `line_diff` is re-exported as `language::line_diff` (its module is private);
  `clone_on_ref_ptr` and `unused_qualifications` on the registry's clone; the unused field.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the tree the third run passed on.

## Phase 3 — Test

- **Scenario:** `script/e2e/727-plan-versions-and-diff.sh` under `compositor sway`, on #726's
  stand-in; version 1 of three steps, an overall note and Send Feedback, version 2 rewording step 2
  and adding step 4.
- **The first run crashed** when version 2 arrived
  (F-claude-727-the-plans-diff-read-the-workspace-inside-its-update-001); fixed, the second run
  stopped at 727-01 to place Changes (1359,128), and the third passed with no panic.
- **727-00-first (REQ-004):** version 1, no version line, count or Changes.
- **727-01-version (REQ-001):** "Version 2", `+2 −1`, Changes; the plan shows version 2's four
  steps.
- **727-02-changes (REQ-002):** the diff: "2. Change the label." removed, "2. Change the label, and
  its twin in `signup.rs`." and "4. Report what failed." added, in Zed's diff colors; the button
  reads Plan.
- **727-03-plan (REQ-003):** version 2 as Markdown again, Approve and Send Feedback in place.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide's Plan review list and the in-app guide's Plan review
  steps; `marley_workbench.md` (Plan review, versions). No Zed path changed.
- **Knowledge:** F-claude-727-the-plans-diff-read-the-workspace-inside-its-update-001,
  AD-claude-727-plan-versions-live-with-the-tab-and-diff-as-zed-draws-one-001.
- **Brain:** `brain decide` on the consultation recorded at promotion.
