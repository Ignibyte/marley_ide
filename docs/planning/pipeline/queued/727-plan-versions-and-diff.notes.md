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
