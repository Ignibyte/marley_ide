---
pipeline_id: 0e8a1428-9d8d-4fcf-88fa-25f33d1725a6
ticket: docs/planning/tickets/open/TICKET-522-review-notes-to-the-agent.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Review notes to the agent: Zed's review comments pasted into an idle agent terminal, and kept"
type: feature
slice: prong 2, the review loop (Orca survey item 11); pairs with #509 and #511
references: [docs/orca_architecture/02-worktrees-and-review.md, docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/481-rich-input.spec.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md]
---

## Title
Marley handles Zed's Send Review to Agent: the review comments of a project diff or a branch diff
go into an idle agent terminal of the diffed tree that Chad picks, as one prompt in Orca's
`File:`/`Line:`/`User comment:` format, and stay in the diff marked Sent.

## Scope
### In
- **Review comments on in every build:** Zed's `diff-review` feature flag gates the gutter's Add
  Review button, and a release build honors no settings override for it; Marley turns it on for
  everyone (D2, a Zed touch in `crates/feature_flags`).
- **The handler** for `editor::actions::SendReviewToAgent`, registered on every workspace by
  `marley_workbench`: from the toolbar's Send Review to Agent (N) in a project diff (`ProjectDiff`)
  or a branch diff (`BranchDiff`), or from the palette's `editor: send review to agent` with such a
  diff active. The notes are the diff's unsent review comments.
- **The target picker:** the terminals running a known agent CLI whose working directory lies
  inside the diffed tree's root, each with its title, its agent and its state: ready, working,
  asking for permission, or no idle signal. Enter on a ready entry sends; the others cannot be
  chosen. Copy notes, at the picker's foot, puts them on the clipboard. With no agent in the tree
  the picker says so and offers Copy.
- **Ready** (D6): Claude Code whose Marley plugin has sent this terminal a notification, the last
  of them not "needs your permission", and whose terminal has printed nothing for two seconds.
- **The prompt** (D4): each note as `File: <path>`, then `Line: N` or `Lines: A-B`, then
  `User comment: "<text>"`, backslashes, quotes, carriage returns and newlines escaped, the notes
  separated by a blank line, in file order then line order; lines count from 1; a path is
  relative to the agent's working directory when the file lies under it, else absolute.
- **Delivery** (D7): `Terminal::paste` then Enter, as rich input sends, then the agent's terminal
  is shown.
- **Sent notes stay** (D3, a Zed touch in `crates/editor`): each sent note keeps its place, its
  anchors following the agent's edits, and shows Sent in its row; the diff's count, and so the
  button, count only unsent notes.

### Out (explicitly deferred)
- An Agent Panel thread as a target (what upstream's removed handler did, into the message
  editor as creases).
- Holding notes until an agent goes idle and sending them then.
- Readiness from structured Claude Code events (turn start, tool, permission), the Orca survey's
  item 1: when that ticket lands, readiness reads its events instead of the three notifications.
- Agents that report no idle signal (Codex, Gemini CLI, OpenCode, and Claude Code without the
  plugin): Copy only, for now.
- Review comments in the commit view, and sending the notes of one file only.
- Editing, deleting or clearing notes (Zed's comment rows have no such buttons yet), and unmarking
  a sent note.
- Hosted pull request comments.

## Reference (§20)
Upstream Zed's diff review: comments on hunks anchored with buffer anchors, the "Send Review to
Agent (N)" button in `ProjectDiff` and `BranchDiff`, and the action it dispatches. Its handler
lived in `agent_ui` until upstream's thread-view refactor (#48339, commit `a5e6964186`, Feb 2026)
deleted it; it took every comment and put them in the Agent Panel's message editor. Marley keeps
the comments, the button and the action, and sends to a terminal agent instead. The delivery
follows Orca (report 02 §2.9): notes pasted into the agent's terminal as one prompt, only while the
agent can take them, in `File:`/`Line:`/`User comment:` form; Marley keeps the notes where Orca
deletes them. Warp: its Interactive Code Review batches line comments and sends them to the CLI
agent in one pass (docs.warp.dev, via the once-over note); Marley does the same, one paste per
send. No Warp source was read.

### Prior art
- **Behavior maps and reports.** Orca report 02 §2.9 and §3 item 5: the format in
  `src/shared/diff-comments-format.ts` (`formatDiffComment`, `formatDiffComments`); the delivery in
  `src/renderer/src/lib/active-agent-note-send-delivery.ts` (check the agent is sendable, bracketed
  paste, 50 ms, check again, then Enter); delete-on-send, which Orca added in May 2026 and which
  the report recommends leaving out. Report 01 §3 item 2: never paste into an agent that waits on
  a prompt (Orca's `running-agent-targets.ts`). The Warp once-over's Interactive Code Review line.
- **Published material.** Claude Code's hooks as Marley's plugin already uses them: the
  `Notification` matchers `permission_prompt` and `idle_prompt`, and `Stop` (AD-claude-482).
- **Code we already ship.** `crates/editor/src/git.rs`: `StoredReviewComment` (171, `pub(super)`),
  `total_review_comment_count` (749), `add_review_comment` (757, which emits
  `ReviewCommentsChanged`), `render_comment_row` (2748, nothing at the right in display mode),
  `diff_review_line_range` (2900, anchors to buffer rows), `take_all_review_comments` (2918,
  `pub(super)`, clears the notes and resets the ids); `crates/editor/src/editor.rs:118`
  (`pub(crate) use git::{DiffHunkKey, StoredReviewComment}`); `SplittableEditor::rhs_editor`
  (`crates/editor/src/split.rs:558`); `ProjectDiff::editor` and `total_review_comment_count`
  (`crates/git_ui/src/project_diff.rs:329,334`), the button (974 to 990) and its dispatch through
  the toolbar (747); `BranchDiff::editor` (`branch_diff.rs:435`) and its button (898 to 905);
  `DiffMultibuffer` turns the review button on (`diff_multibuffer.rs:89`) and caches the count from
  `ReviewCommentsChanged` (98). `crates/feature_flags/src/flags.rs:30-40` (`DiffReviewFeatureFlag`,
  off for staff too), `store.rs:105-107` (overrides only in a debug build or for staff) and
  `:164-168` (`enabled_for_all` wins). `crates/editor/src/element.rs:2742` and
  `element/mouse.rs:116` (the flag's two checks). Marley's `rich_input::send` (97 to 116: paste,
  then `\r`), `Terminal::paste` (`crates/terminal/src/terminal.rs:2582`: every ESC stripped,
  bracketed when the program asked), `notifications.rs` (every terminal's
  `Event::MarleyNotification`), the plugin's `hooks/hooks.json` and `hooks/notify.sh` (three fixed
  messages), `agent_bar::agent_in` (129), `marley_agent::WAITING_AFTER` (117). The sweep's win:
  Zed already stores, anchors, draws and counts the notes; Marley adds a flag, a mark and a sender.

## UI proof
UI-AFFECTING. `script/e2e/522-review-notes-to-the-agent.sh` (`compositor sway`: it clicks the
gutter and the toolbar). Setup: a scratch repository with `notes.txt` committed and one line of it
changed, not staged; a HOME whose `.bashrc` defines stand-in agents that run as `claude` or `codex`
by name (`exec -a`): `ready` prints Marley's plugin's "finished" notification (the OSC 777 its
`notify.sh` sends) and then echoes each line it reads, `busy` prints the same notification and then
a dot every half second, `asking` prints the "needs your permission" notification and sleeps, and
`codexish` runs as `codex` and sleeps; four terminals run one each. Shots: `522-01-comment` (the
project diff with a note on the changed line and Send Review to Agent (1)), `522-02-picker` (the
four agents: ready, working, asking for permission, no idle signal), `522-03-sent` (the ready
agent's terminal, shown, echoing the `File:`, `Line:` and `User comment:` lines), `522-04-marked`
(back in the diff: the note marked Sent, the button gone), `522-05-copied` (a second note, Copy
notes, then Ctrl+Shift+V into a plain terminal shows it; the button still reads (1)).

## Locked-In Decisions
- D1 — The handler is Marley's and lives in `marley_workbench`: a workspace action for
  `editor::actions::SendReviewToAgent`. Upstream's button dispatches into nothing since #48339, so
  no Zed code has to move.
- D2 — Review comments are on for everyone: `DiffReviewFeatureFlag::enabled_for_all` returns true
  (a Zed touch of one method). Without it the gutter's Add Review never shows outside Zed's staff,
  and a release Marley honors no `feature_flags` override.
- D3 — Sending keeps the notes: `StoredReviewComment` gains `sent`, the count
  (`total_review_comment_count`, and with it `ReviewCommentsChanged` and the button) counts
  unsent notes, and a sent note's row shows Sent. The editor gains a public read of the unsent notes
  as plain data and a public `mark_review_notes_sent`, both in `crates/editor/src/git.rs`;
  `take_all_review_comments` stays as it is. Upstream never sets `sent`, so its behavior is
  unchanged.
- D4 — The prompt is Orca's `formatDiffComment`, reimplemented in `marley_agent` (pure, beside the
  launch lines) with the Orca path in a comment: `File:`, `Line: N` or `Lines: A-B`,
  `User comment: "..."` with `\`, `"`, CR and LF escaped, the notes separated by a blank line.
- D5 — The targets are the agent terminals working in the diffed tree (their working directory
  under the tree's root, the worktree's own for a worktree's diff), so the notes' relative paths
  mean the same files to the agent. Chad picks one; nothing is sent without that choice.
- D6 — Ready means Claude Code with Marley's plugin reporting: at least one of the plugin's
  notifications seen from that terminal, the last one not "needs your permission", and two seconds
  with no output (`marley_agent::WAITING_AFTER`). Claude Code animates while a turn runs, so a
  quiet terminal is not mid-turn. A terminal that has sent no notification (a trust dialog, a
  fresh start, a CLI without the plugin) is never ready: Marley cannot tell a prompt from an idle
  agent there, and a paste into a dialog would answer it.
- D7 — Delivery is rich input's: `Terminal::paste` (bracketed when the agent asked for it, every
  ESC removed) and then `\r`. Then Marley shows the agent's terminal, as a pick's Send does.
- D8 — Copy notes puts the same text on the clipboard and marks nothing sent: only a delivery
  Marley made itself counts as sent.
- D9 — The plugin's three messages are a contract between two Marley files: `notify.sh`'s
  `needs your permission`, `is waiting for you` and `finished`, under the title `Claude Code`, named
  once in Rust beside the parser, with a comment in each file naming the other.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley runs, a project diff and a branch diff shall offer Add Review in the gutter of a changed line. | Shot `522-01-comment` |
| REQ-002 | WHEN the user sends a review from a diff with unsent notes, the system shall list the agent terminals working in the diffed tree, each with its state. | Shot `522-02-picker` |
| REQ-003 | WHEN the user picks a ready agent, the system shall paste the unsent notes into its terminal as one prompt in the `File:`/`Line:`/`User comment:` format, press Enter and show that terminal. | Shot `522-03-sent` |
| REQ-004 | WHEN notes are sent, the system shall keep each in the diff marked Sent, and the diff's Send Review to Agent count shall count only unsent notes. | Shot `522-04-marked` |
| REQ-005 | WHILE an agent's terminal printed output in the last two seconds, or its last plugin notification asked for permission, or it has sent none, the system shall not send to it and shall show which. | Shot `522-02-picker` |
| REQ-006 | WHEN the user chooses Copy notes, the system shall put the notes on the clipboard in the same format and leave them unsent. | Shot `522-05-copied` |
| REQ-007 | WHERE no agent terminal works in the diffed tree, the picker shall say so and offer only Copy notes. | Review of the picker's empty state |

## Phase Plan
- **P1 Plan** — this spec; the design and the E2E plan in the notes.
- **P2 Code** — the touchpoint rows first; the flag; the editor's `sent` mark, count, row label and
  public read and mark; `marley_agent`'s formatter; the workbench's readiness tracker, handler,
  picker and delivery; fmt and clippy clean; a review of the diff (§18.1).
- **P3 Test** — write and run `script/e2e/522-review-notes-to-the-agent.sh`, read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and
  `marley_agent.md`, the touchpoint rows checked, ledger capture, close, archive, commit.
