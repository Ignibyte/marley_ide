---
pipeline_id: 76921186-e24a-426e-a5a9-6cabf7fc6068
ticket: docs/planning/tickets/open/TICKET-733-guided-review.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Guided review
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 8; docs/planning/design-notes/finish-line-2026-10-11.md row 11); after #730 (the `review` tool family)
references:
  - docs/planning/pipeline/queued/730-agent-findings-as-diff-notes.spec.md
  - docs/planning/pipeline/queued/731-viewed-marks-in-the-diff.spec.md
---

## Title
An agent splits a change (a turn, a branch, a pull request) into ordered sections and sends them
over MCP; Marley draws them in a center tab, each with its summary, its files and a Reviewed tick,
and a click on a file shows it in the project diff. Chad, 2026-10-11, on the finish-line list:
"11.) yes".

## Scope
### In
- `review_guide_open` in the `review` family (#730): `title`, an optional `summary`, and 1 to 30
  `sections`, each with a `title`, a `summary` (Markdown) and `files`, each an absolute `path` with
  an optional `first_line` and `last_line`.
- **The tab** ("Guide · <title>"), in the project whose folder holds the first file, else the
  caller's project: the guide's title and summary, "K of N reviewed", then the sections in the
  order sent, each numbered, with its title, its summary drawn as Markdown, a chip per file
  (`path` or `path:A-B`) and a **Reviewed** box.
- Ticking a section folds it to its title line; unticking opens it.
- A file chip shows that file in the project diff when the diff holds it, else opens the file in an
  editor at its first line.
- A guide sent with the same title to the same project replaces the open tab's sections, its ticks
  cleared.
- The tool asks the user as the editor tools do (`marley.agent_control.editors`, once per session
  under Ask First), and lists in Agent Activity.

### Out (explicitly deferred)
- Keeping guides after the tab closes or Marley quits ("Previous guides"), and saving ticks.
- Marking a guide stale when the repository moves to another commit ("Diff changed").
- Sections over a branch diff (#511's Review) and a pull request's hosted diff: a chip opens the
  project diff or the file.
- A skill that teaches an agent to write a guide, and starting a guide agent from Marley (#717's
  second opinion is the place).
- Linking ticks to #731's per-file Viewed marks.

## Reference (§20)
N/A — Marley-specific: Zed has no guided review. The behavior follows plannotator's Guided Review
(published docs, `docs.plannotator.ai/open-source/workflows/agent-reviews`): the change organized
into ordered sections, each with a summary and its diffs; completion per section, a completed
section collapsed; a file chip that scrolls to the section and file. Marley draws the sections and
leaves the diffs to Zed's project diff. No plannotator code is taken.

### Prior art
- **The code we ship:**
  - The `review` family and the admission through the editors' area (#730's spec, D2); the
    registry seam (`crates/marley_mcp/src/registry.rs:15, 125, 715`), the deferral
    (`crates/marley_mcp/src/dispatch.rs:243-260`), the route (`crates/marley_workbench/src/mcp.rs:516`).
  - A tool that opens a center tab in the right project: `editor_tools::open` and
    `project_holding` (`crates/marley_workbench/src/editor_tools.rs:470, 565`), `browser_tools::open_in_tab`
    (366-395).
  - A list tab with ticks: `rusty/tasks_tab.rs` (`render_task` 956-1042: a `Checkbox` in a
    `ListItem`; `Item` 1188-1229); `threads_page.rs` (`open` 47-60, rows 239, `Item` 437-457).
  - Markdown in a tab: `rusty/page.rs` (338-351, 1545-1603).
  - The project diff: `ProjectDiff::deploy_at_project_path` (`crates/git_ui/src/project_diff.rs:188`)
    opens or brings forward the diff and moves to a file; files not in the diff need the editor
    (`Workspace::open_path`).
  - Every center tab gets a rail row (`rail.rs:8790-8846`).
- **Behavior maps:** none cover a guide; Orca report 02 §2.8 (the diff viewer's file tree) is the
  nearest.
- **Published material:** plannotator's agent reviews page (above). Its data format for a guide is
  not published; Marley's arguments are its own.

## UI proof
`script/e2e/733-guided-review.sh`, under `compositor sway`: a scratch repository with `alpha.txt`
and `beta.txt` committed and each changed; #704's scripted MCP client through the plugin's bridge
calls `review_guide_open` with two sections, one per file.

Shots:
- `733-01-guide`: after Allow for This Session, the Guide tab: the title, "0 of 2 reviewed", both
  sections with their summaries, chips and boxes.
- `733-02-chip`: the chip `alpha.txt:3` clicked: the project diff in front at `alpha.txt`.
- `733-03-ticked`: back on the Guide tab, section 1's box ticked: "1 of 2 reviewed", section 1
  folded to its title.

## Locked-In Decisions
- **D1:** the guide is a tool call with its sections inline; Marley keeps no guide format of its
  own beyond the tool's arguments. Rejected: a guide file in the repository (it would be committed
  by mistake).
- **D2:** the tab draws sections and leaves diffs to the project diff, one click away. Rejected:
  embedding each section's diff in the tab (a second diff view to keep in step with Zed's).
- **D3:** ticks live with the tab. Rejected for now: persistence (a guide is read once, in one
  sitting, and a later guide replaces it).
- **D4:** the same admission as #730's tool: the editors' area, the `Act` level.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an allowed `review_guide_open` call arrives, the system shall open a Guide tab in the file's project with every section in the order sent, each with its summary, its file chips and a Reviewed box. | Shot 733-01 |
| REQ-002 | WHEN the user clicks a file chip whose file the project diff holds, the system shall show that file in the project diff. | Shot 733-02 |
| REQ-003 | WHEN the user ticks a section, the system shall fold it to its title and count it in "K of N reviewed". | Shot 733-03 |
| REQ-004 | WHEN the user clicks a chip whose file is not in the project diff, the system shall open the file in an editor at its first line. | Review of the diff |
| REQ-005 | WHEN a guide with the same title arrives for the same project, the system shall replace the open tab's sections and clear its ticks. | Review of the diff |
| REQ-006 | IF a call has no sections, more than 30, or a relative path, THEN the system shall refuse it and say which. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — the registry row and schema; `review_tools.rs` (#730's) gains the tool; a new
  `guide_tab.rs` in `marley_workbench`; the guide; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
