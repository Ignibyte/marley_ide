---
pipeline_id: 4df6a71f-c635-4671-b7da-5ade6997879e
ticket: docs/planning/tickets/open/TICKET-929-terminal-more-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Table tabs, restored blocks, dropped links and the mic as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/planning/pipeline/completed/764-restored-blocks.spec.md, docs/planning/pipeline/completed/641-ssh-links-that-know-they-are-dead.spec.md]
---

## Title
The table tab's header, a "Marley started again" divider after restored blocks, the dropped
link's line with Try Now and its reassurance, the microphone's "Listening · Voxtype" label, and a
screen-read state drawn grey, as the TerminalMore board draws them.

## Scope
### In
- **The table tab's header** (`block_tables.rs`, `BlockTable` and its `render`; Marley crate): a
  row over the table: a kind chip ("csv", "tsv", "json", "jsonl", "md", from
  `marley_terminal::tables::TableKind`), the tab's name, and "N rows · from <command>" muted.
  `BlockTable` keeps the kind, the row count once parsed, and the command (a block's command, or
  `show <file>` for #805's tables).
- **The restart divider** (`scrollback.rs`; `block_headers.rs`; `marley_terminal/src/anchored.rs`;
  Marley crates): a restored terminal remembers when it was restored; a centred muted line
  between two rules, "Marley started again · HH:MM", is drawn between the last restored block and
  the first live prompt. The restore leaves one empty row for it after the replayed output.
- **The dropped link** (`remote.rs`, `link_overlay`; Marley crate): the line reads "<host>
  stopped answering · checking again in N s · N keys held back" (or "· checking now"), with a
  Try Now button that ends the wait and checks the host at once; under the strip, "The tmux
  session on the host keeps running; the tab attaches again when the host answers." The stopped
  phase (ssh refused, "Rerun to log in") keeps its words.
- **The microphone's label** (`agent_bar.rs`, `microphone_look` and the button; Marley crate):
  while Voxtype records, the button is a pill with a red dot and "Listening · Voxtype"; while it
  transcribes, "Transcribing · Voxtype"; idle stays the icon.
- **A screen-read state drawn grey** (`rail.rs`, `harness_row`; Marley crate): when a seat's
  state was not declared and its source is `detected`, the row's line is muted as its dot already
  is (AD-claude-640), and its mark reads "read off the screen"; other undeclared sources keep
  their word, also muted.

### Out (explicitly deferred)
- The microphone's live waveform and the dictated words streaming into the rich input with "…":
  Voxtype reports its state, not its levels or partial text (AD-claude-480); a follow-up ticket
  if Voxtype's streaming engines expose them.
- The permission modes list (Default, Accept edits, Plan, Bypass with dots, lines and "Shift Tab
  steps to the next"): not in this ticket's list; a follow-up ticket.
- A local agent terminal whose state comes from its quiet time and bell drawn grey: the review's
  evidence is the harness row (`rail.rs:8180`); local rows are the rail ticket's (build order
  item 3).

## Reference (§20)
- Warp (blocks): Session Restoration brings back recent blocks when Warp relaunches
  (docs.warp.dev/terminal/sessions/session-restoration, as #764's spec cites it, with
  `docs/warp_architecture/subsystems/06-platform-settings-infra.md:89-90`); Marley keeps #764's
  restored blocks and adds the board's divider. No Warp code.
- Upstream Zed: the table tab is Zed's `TableView` (kept, a header added over it); the remote
  terminal is a Zed task terminal whose Rerun Marley drives (AD-claude-543), kept.
- Voxtype and the harness rows are Marley-specific (#480, #640).

### Prior art
- `block_tables.rs:784` `BlockTable { title, tooltip, table }` and `:799` its `render`;
  `:642` `open_output` and `:712` `show_table` know the command; `marley_terminal/src/tables.rs:53`
  `TableKind { Delimited(char), JsonArray, JsonLines, Markdown }`.
- `marley_terminal/src/anchored.rs:468` `restore`, `:490` `restored_count`;
  `block_headers.rs:146` writes "restored" for a restored block; `scrollback.rs:162` `restored`
  hands the replay to the builder (AD-claude-764).
- `remote.rs:524` `link_overlay`; `:383` the backoff check and `:447` the rerun once the host
  answers; `Phase { Waiting, Checking, Stopped }`.
- `agent_bar.rs:407` `microphone_look`; `voice.rs:182` maps Voxtype's `recording` and
  `streaming` to `Recording`.
- `rail.rs:8140` `harness_row`: `weaker = !signals.declared()` puts the source's word in the
  mark; `harness.rs:1359` the source words (`protocol`, `reported`, `runtime`, `detected`).
- Scenarios to reuse: `764-restored-blocks.sh`, `804-a-blocks-table-in-the-terminal.sh`,
  `641-ssh-links-that-know-they-are-dead.sh` (#445's fake SSH host), `480-voice-input.sh` (a
  stand-in Voxtype), `640-agent-state-source-progress-quota.sh` (a harness seat with a source).
- The brain: not consulted at queueing; the promoting session asks.

## UI proof
Scenario `script/e2e/929-terminal-more-as-the-board.sh` (sway). Setup: a scratch repository;
`marley.restore_scrollback` on; #445's fake SSH host for a remote terminal; a stand-in
`voxtype` first on PATH with `marley.voice.enabled`; a stand-in harness seat with a `detected`
state as #640's scenario makes one; stand-ins on the terminals' PATH through the `.bashrc`.
- `929-01-table`: `printf 'crate,tests\nrail,212\n'` then Open as Table: the tab's header
  (REQ-001).
- `929-02-restored`: two commands run, Marley quit and started again on the same profile: the
  restored blocks, the divider, the live prompt (REQ-002).
- `929-03-link`: the fake host stops answering: the strip with Try Now and the tmux line
  (REQ-003); `929-04-try-now`: the host back, Try Now clicked: the terminal attached again
  (REQ-003).
- `929-05-mic`: dictation started on the stand-in Claude Code's agent bar: "Listening · Voxtype"
  (REQ-004).
- `929-06-screen-read`: the harness seat's row drawn grey with "read off the screen" (REQ-005).
- `929-07-day`: the table header, the divider and the link strip in Marley Day (REQ-001 to
  REQ-003).

## Locked-In Decisions
- D1 — The table's header names what the tab already knows; no new parsing.
- D2 — The divider's time is when the terminal was restored, in the local clock's hours and
  minutes; it shows once, after the last restored block.
- D3 — Try Now runs the same check the backoff runs, at once; it never reruns a task while the
  host does not answer.
- D4 — The tmux line shows for a remote terminal only, all of which run in Marley's tmux
  (AD-claude-543).
- D5 — The microphone's words name Voxtype's state; nothing claims to show audio.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a table tab shows a block's or a file's table, a header over it shall show the table's kind, its name and "N rows · from <command>". | `929-01-table`, `929-07-day` |
| REQ-002 | WHEN a terminal is restored after a restart, a divider reading "Marley started again · HH:MM" shall sit between its restored blocks and its live prompt. | `929-02-restored`, `929-07-day` |
| REQ-003 | WHILE a remote terminal's host does not answer, its strip shall say the host stopped answering, when it checks again and how many keys it holds back, offer Try Now, and say that the tmux session keeps running. | `929-03-link`, `929-04-try-now`, `929-07-day` |
| REQ-004 | WHILE Voxtype records for an agent bar, the microphone shall read "Listening · Voxtype". | `929-05-mic` |
| REQ-005 | WHILE a harness seat's state is only read off the screen, its row shall be drawn grey and say "read off the screen". | `929-06-screen-read` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `block_tables.rs`, `scrollback.rs`, `block_headers.rs`, `anchored.rs`,
  `remote.rs`, `agent_bar.rs`, `rail.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
