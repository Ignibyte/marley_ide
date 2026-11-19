---
pipeline_id: b454d82e-8990-44e7-87f4-75ae42ce4f6c
ticket: docs/planning/tickets/closed/TICKET-952-merge-upstream-zed.md
status: Phase 4 — Complete PASS
title: Merge upstream Zed through 2026-11-19
type: chore
slice: upstream sync
references:
  - docs/marley/zed-touchpoints.md
  - docs/planning/pipeline/completed/950-merge-upstream-zed.spec.md
  - docs/planning/pipeline/completed/797-merge-upstream-zed.spec.md
---

## Title
Merge upstream Zed's main of 2026-11-19 (`3d3bd04fad`) into Marley, so the fork carries what
v1.27 to v1.29 and v1.30.1-pre ship and the work since, and keep Marley's rules where agent
sessions read them now that upstream has moved its rules into `AGENTS.md`.

## Scope
### In
- `git merge --no-ff --no-commit upstream/main` at `3d3bd04fad` on
  `marley/merge-upstream-2026-11-19` (from `marley/workbench-shell` at `cacf1f61d8`), each of the
  eleven conflicts resolved under D5.
- The rules file: upstream's `AGENTS.md` (#66716) with Marley's two sections where `.rules` had
  them (D3), `.rules` deleted as upstream deleted it, `CLAUDE.md` back as a link to `AGENTS.md`
  (D4). Marley's own docs and pipeline commands that name `.rules` name `AGENTS.md`.
- Build breaks in the Marley crates, fixed at the source: `ProjectDiff::deploy_at_project_path`
  removed (`0c5af8c563`) in `review_tools.rs` and `guide_tab.rs` (D6);
  `Case::smart_if_uppercase_in` removed (`36acea54c5`) in `rusty/page_picker.rs`;
  `root_thread_has_pending_tool_call` renamed `root_thread_is_waiting_for_input` (`270df71b8d`) in
  `tab_tiles.rs` and `thread_activity.rs`; and whatever else `cargo check -p zed` names.
- `Cargo.lock`: Marley's kept, cargo left to add upstream's (PR-claude-695).
- The merge-time steps in `docs/marley/zed-touchpoints.md` ("At an upstream merge"), the upstream
  base moved there and in `UPSTREAM_BASE_FALLBACK`.

### Out (explicitly deferred)
- Adopting upstream's new features in Marley (project bookmarks, the Linux sandbox, the language
  model selector crate, response controls in Marley's own composer).
- `GEMINI.md`: upstream deleted it and Marley never changed it; it stays deleted.
- The golden set's drift; its repair belongs to the testing phase at the end (§7).

## Reference (§20)
Upstream Zed itself: this is the merge. Behavior that upstream changes is kept as upstream wrote
it (D5). The hunks Marley carries keep the behavior their own tickets' specs describe.

### Prior art
- `docs/marley/zed-touchpoints.md`: every change outside Marley's paths, with what to keep at a
  merge, and its "At an upstream merge" checklist.
- #695, #797 and #950's specs and notes (`docs/planning/pipeline/completed/`): the lockfile trap,
  the merge-aware `upstream_base`, gates 12, 13 and 21 during a merge, marker counts per file, the
  removed-symbol sweep before the build (L-claude-950-merging-upstream-zed-001), the commit-gate
  hook's text match on prose.
- The code we ship, for the removed diff entry point: upstream kept `ProjectDiff::deploy_at`
  (`crates/git_ui/src/project_diff.rs`, public), which opens or reuses the project diff, follows
  the active repository and moves to a status entry, and `ProjectDiff::move_to_project_path`
  (public). Together they do what `deploy_at_project_path` did, so Marley calls them and carries
  no Zed hunk (D6).
- For the rules file: upstream's `AGENTS.md` is the old `.rules` with four mentions of `.rules`
  renamed; `docs/AGENTS.md` replaces `docs/.rules`. Claude Code reads `CLAUDE.md`, not
  `AGENTS.md`, so the link upstream dropped is what carries the rules into every session here.

## UI proof
The visual check covers the Marley code this ticket rewrites and the conflicts where upstream
reworked the code under a Marley hunk. Each scenario also runs on the installed pre-merge build
(`cacf1f61d8`). Only a check that passes before and fails after counts against the merge.
- `script/e2e/733-guided-review.sh`: the guide's chip opens the project diff at its file
  (`733-02-chip`), through D6.
- `script/e2e/730-agent-findings-as-diff-notes.sh`: `review_notes_add` opens the diff with the
  proposals at their lines (`730-02-proposed`), through D6.
- `script/e2e/921-file-finder-as-the-board.sh`: the preview header's line count, beside
  upstream's reworked `picker_preview`.
- `script/e2e/854-open-recent.sh`: Marley's marks and places on the recent rows.
- `script/e2e/907-bottom-row-for-a-thread.sh`: a thread's bottom row, beside upstream's response
  controls in `thread_view.rs`.
- `just shot 952-merged`: the merged build starts in the Marley layout.

## Locked-In Decisions
- **D1:** merge `main` at `3d3bd04fad`, not a release tag: the fork's base is main.
- **D2:** a merge commit, not a rebase, so the pushed commits keep their hashes.
- **D3:** `AGENTS.md` is upstream's file with Marley's "Marley" section on top and its Google Rust
  style section after Zed's Rust coding guidelines, where `.rules` had them; upstream's lines stay
  as written. Marley's sections name `AGENTS.md` where they named `.rules`.
- **D4:** `CLAUDE.md` is a Marley link to `AGENTS.md`, with its own touchpoints row, so Claude
  Code sessions in the repository read the rules as they did. `GEMINI.md` stays deleted.
- **D5:** upstream is right in every conflict (Chad, 2026-10-29, AD-claude-950-upstream-wins-a-
  merge-conflict-001). Each conflicted file starts from upstream's side. A Marley hunk goes back
  only as an addition that leaves upstream's lines as they are; where it would change them, the
  behavior moves around upstream's code instead.
- **D6:** the review tools and the guide open the diff through upstream's `ProjectDiff::deploy_at`
  with no entry, then `move_to_project_path` on the diff it opened, in one Marley helper both call.
  No Zed hunk brings `deploy_at_project_path` back.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN merged, the tree shall carry upstream through `3d3bd04fad` with every touchpoints row's hunk present | review of each conflict against its row; `Marley:` markers counted per file before and after; gate:16 green |
| REQ-002 | WHEN built, the gate shall be green | `script/gates.sh --diff` exit code |
| REQ-003 | WHEN an agent session opens in the repository, it shall read Marley's section and Zed's rules from `AGENTS.md` through `CLAUDE.md` | review: `CLAUDE.md` is a link to `AGENTS.md`, which holds both Marley sections and upstream's lines verbatim (`git diff upstream/main -- AGENTS.md` shows only additions) |
| REQ-004 | WHEN the guide's chip is clicked, Marley shall open the project diff at the chip's file, as before the merge | `733-02-chip` on the merged release build, against the pre-merge build |
| REQ-005 | WHEN an agent adds review notes, Marley shall open the project diff with the proposals at their lines, as before the merge | `730-02-proposed`, against the pre-merge build |
| REQ-006 | WHEN the conflicted Marley looks draw (the preview's line count, the recent rows, a thread's bottom row), they shall show as before the merge | 921, 854 and 907 on the merged release build, against the pre-merge build |
| REQ-007 | WHEN started, the merged Marley shall open in the Marley layout with the rail and a center terminal | `just shot 952-merged` |

## Phase Plan
- **P1 Plan:** this spec, from the trial merge (11 conflicted files) and the removed-symbol sweep.
- **P2 Code:** merge, resolve, the rules file, build, fix, the review, and the gate.
- **P3 Test:** the five scenarios on a release build against the pre-merge build, and the shot.
- **P4 Complete:** the CHANGELOG, the touchpoints rows and base that moved, the ledger, commit,
  push and install.
