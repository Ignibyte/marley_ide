---
pipeline_id: 97420e97-5adf-48e9-900a-7a0866b40b5f
ticket: docs/planning/tickets/open/TICKET-730-agent-findings-as-diff-notes.md
status: Phase 4 — Complete PASS
title: Agent findings as diff notes
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 5; docs/planning/design-notes/finish-line-2026-10-11.md row 11)
references:
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/planning/tickets/open/TICKET-717-second-opinion-and-handoff.md
---

## Title
A reviewing agent, or any MCP client such as a linter's wrapper, puts its findings into the project
diff as review notes tagged with where they came from. The user keeps or drops each one; kept notes
go to the author agent with the user's own through Send Review to Agent (#522). Chad, 2026-10-11, on
the finish-line list: "11.) yes".

## Scope
### In
- A new MCP tool family, `review`, and its first tool, `review_notes_add`: `source` (a short name,
  such as `clippy` or `codex review`; the caller's client name when left out) and `notes`, up to 50,
  each an absolute `path`, `first_line`, `last_line` (optional) and `comment`.
- Each note lands in the project diff of the project whose folder holds its file; Marley opens that
  diff when it is not open and shows it once per call. A note whose lines are not in the diff is not
  placed, and the answer names it and why.
- A placed note shows in the hunk's review block like the user's own, with its source beside the
  text and **Keep** and **Drop** buttons. Until kept it is a proposal: it is not counted by Send
  Review to Agent (N) and is not sent.
- Keep makes it an ordinary unsent note that still shows its source; Drop removes it.
- A kept note reaches the agent with a `Source: <name>` line after its `User comment:`.
- The tool asks the user first, as the editor tools do (`marley.agent_control.editors`, once per
  session under Ask First), and lists in Agent Activity.

### Out (explicitly deferred)
- Starting a reviewing agent on a diff, with a skill as its prompt: #717's second opinion.
- Notes on the branch diff (#511's Review), on unchanged lines, and on deleted lines (the old side).
- A severity or kind field (plannotator's `concern` and `suggestion`), and suggested code.
- Editing a proposal before keeping it, and Keep All or Drop All.

## Reference (§20)
Upstream Zed's diff review comments, as #522 left them: comments stored per hunk on the diff's
right-hand editor, drawn in a block under the hunk, counted by the toolbar's Send Review to Agent.
Kept as they are; Marley adds a source and a proposal state to a note, and a way in for tools. The
behavior follows plannotator's external annotations (published docs,
`docs.plannotator.ai/open-source/reference/external-annotations`): a finding carries a required
`source`, a file and a line range, and appears among the reviewer's annotations; its agent reviews'
findings "appear as review annotations, where you decide what to keep". No plannotator code is
taken.

### Prior art
- **The code we ship:**
  - `crates/editor/src/git.rs`: `StoredReviewComment` (171, `sent` from #522 at 180),
    `ReviewNote` (188), `total_review_comment_count` (764), `unsent_review_notes` (776),
    `mark_review_notes_sent` (810), `add_review_comment` (825, `pub(super)`, stores a note but
    draws no block), `show_diff_review_overlay` (572, public: the hunk key and the block for a
    range of display rows), `remove_review_comment` (1235), the unused `DeleteReviewComment`
    action's handler (1548), `render_comment_row` (2816: display mode draws nothing at the right,
    "edit/delete not yet implemented"), `calculate_overlay_height` (2134).
  - `crates/marley_workbench/src/review_notes.rs`: the picker and the send (#522), and
    `marley_agent::review_prompt` (`crates/marley_agent/src/marley_agent.rs:371`).
  - The MCP seam: `Family` and `REGISTRY` (`crates/marley_mcp/src/registry.rs:15, 125`),
    `tool_schemas` (715), the deferral of app families (`crates/marley_mcp/src/dispatch.rs:243-260`),
    the grants at start (`crates/marley_workbench/src/mcp.rs:108-116`), the route by prefix
    (`mcp.rs:516-545`), `agent_activity::gate` (525).
  - Admission: `agent_control::admit(call, Area, Level, what, cx)`
    (`crates/marley_workbench/src/agent_control.rs:161`), `Area::Editors` read from
    `marley.agent_control.editors` (62).
  - A tool that finds a file's project and shows it: `editor_tools::project_holding` and `open`
    (`crates/marley_workbench/src/editor_tools.rs:565, 470`).
  - The project diff: `ProjectDiff::deploy_at_project_path` (`crates/git_ui/src/project_diff.rs:188`),
    `editor` (335), `multibuffer` (340); files load into the diff asynchronously, 16 at a time
    (`crates/git_ui/src/diff_multibuffer.rs:42`, `refresh` 626).
  - The caller: `Caller { terminal, project, cwd, client }` (`crates/marley_mcp/src/marley_mcp.rs:159`),
    the client's name from `initialize` (`crates/marley_mcp/src/session.rs:139-150`).
  - No tool in `REGISTRY` touches diffs or review notes.
- **Behavior maps:** Orca report 02 §2.9 (`docs/orca_architecture/02-worktrees-and-review.md`): notes
  carry a `source` field (diff or markdown) but no tool adds them.
- **Published material:** plannotator's external annotations (above): `source` required, `scope`,
  `filePath`, `lineStart`, `lineEnd`, `type`, `text`; added over a local HTTP API. The MCP spec's
  tool results (structured content) for the answer.

## UI proof
`script/e2e/730-agent-findings-as-diff-notes.sh`, under `compositor sway`: #522's set-up (a scratch
repository with `notes.txt` committed and lines 3 and 5 changed, the stand-in `claude` that sends
Marley's plugin's `SessionStart` and echoes each line it reads), and #704's scripted MCP client
through the plugin's bridge.

Shots:
- `730-01-asked`: the client calls `review_notes_add` with source `scenario-lint` and two notes
  (lines 3 and 5) and one on an unchanged line 1: Marley's question for the call.
- `730-02-proposed`: after Allow for This Session, the project diff in front with both notes under
  their hunks, each with `scenario-lint`, Keep and Drop; the toolbar shows no Send Review count. The
  client's answer (saved by the scenario) names line 1 as not placed.
- `730-03-kept`: Keep on line 3's note, Drop on line 5's: line 3's note stays with its source and no
  buttons, line 5's is gone, and the toolbar shows Send Review to Agent (1).
- `730-04-sent`: Send Review to Agent, Enter on the ready row: the stand-in's echo shows `File:`,
  `Line: 3`, `User comment:` and `Source: scenario-lint`.

## Locked-In Decisions
- **D1:** one tool, `review_notes_add`, taking a batch, so a review of twenty findings is one
  question and one view of the diff. Rejected: a tool per note (twenty questions and twenty jumps).
- **D2:** a new family, `review`, with the grant class `review.write` granted at start, admitted
  through the `Editors` area at the `Act` level. Rejected: a verb in the `editor` family (#733's
  guide, the family's next tool, is not an editor); a new area with its own setting (another switch
  for a tool that writes nothing to disk).
- **D3:** a finding is a proposal until the user keeps it, and only kept notes count or go out.
  Rejected: findings as ordinary notes the user may delete (a review would forward findings nobody
  read).
- **D4:** the Zed touch stays in `crates/editor/src/git.rs` and is additive: `source` and
  `proposed` on `StoredReviewComment`; a public `add_review_note` that places a note on a range of
  the diff and draws its hunk's block without taking focus; `ReviewNote.source`; Keep and Drop in
  `render_comment_row` for a proposal and the source label for any note that has one; the count and
  `unsent_review_notes` skip proposals. Upstream never sets either field, so its behavior is
  unchanged.
- **D5:** a note goes only on lines the diff shows (the new side of a hunk with its context); others
  are refused by name. Rejected: widening the diff's excerpts to fit a note (a Zed change to
  `DiffMultibuffer` for a rare case).
- **D6:** `source` falls back to the caller's client name, so every finding says where it came from.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an MCP client calls `review_notes_add`, the system shall ask the user as the editor tools do before it places anything. | Shot 730-01 |
| REQ-002 | WHEN the call is allowed, the system shall place each note whose lines the project diff shows under its hunk with its source, Keep and Drop, and show the diff. | Shot 730-02 |
| REQ-003 | WHEN a note's file or lines are not in the project diff, the system shall leave it out and name it, with the reason, in the tool's answer. | The client's answer, saved by the scenario and read in P3; shot 730-02 shows no note on line 1 |
| REQ-004 | WHILE a note is a proposal, the system shall leave it out of Send Review to Agent's count and out of what is sent. | Shot 730-02 |
| REQ-005 | WHEN the user chooses Keep, the system shall make the note an unsent note that keeps its source label, and count it. | Shot 730-03 |
| REQ-006 | WHEN the user chooses Drop, the system shall remove the note. | Shot 730-03 |
| REQ-007 | WHEN a kept note is sent to an agent, the system shall add a `Source:` line with its source after its `User comment:`. | Shot 730-04 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — the touchpoint rows first; the editor's fields, `add_review_note`, the row's
  buttons and label; the `review` family in `marley_mcp` (registry row, schema, dispatch); a new
  `review_tools.rs` in `marley_workbench` and its route; `review_prompt`'s `Source:` line; the
  guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
