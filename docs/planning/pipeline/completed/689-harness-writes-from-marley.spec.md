---
pipeline_id: 0627182c-164b-4099-867b-e108468a8cba
ticket: docs/planning/tickets/closed/TICKET-689-harness-writes-from-marley.md
status: Phase 4 — Complete PASS
title: Harness writes from Marley
type: feature
slice: phase 2 item 4 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/534-harness-sessions-in-the-rail.spec.md
---

## Title
A harness session's tab answers its question, sends it text and lists its views, and a palette
command opens a session from a profile. All of it goes through the harness's write verbs, and
only while `marley.harness_writes` is on.

## Scope
### In
- **The setting:** `marley.harness_writes` (default false). While it is on, the embedded harness
  is followed with `rh --state ROOT mcp --grant write`, and a change restarts the follow.
- **The tab, while the setting is on:**
  - **The question.** When the seat waits on a question, its prompt shows with a button per
    option. A click calls `session_answer {id, choice, prompt}` with the full prompt.
  - **Send.** A one-line editor with Send (or Enter) calls
    `session_send {id, text, delivery}`, with a new UUID each time. Under the editor goes the
    receipt's state and detail, or the refusal's reason.
  - **Views.** The button calls `session_surface_to_human {id}` and lists each view's kind and
    command line, each with Copy.
- **The palette:** `marley: open harness session` is listed only while the setting is on and a
  harness is followed. A picker lists `ROOT/profiles/*.json` names when Marley runs the harness
  itself, and always offers the typed name. Choosing one calls `session_open {profile, request}`
  and opens the new session's tab.
- **Receipts:** a write's answer is read as `{result: accepted, value}` or
  `{result: refused, reason}`, whether or not `isError` is set.

### Out (explicitly deferred)
- Running a view in a terminal pane: item 5, after harness TICKET-109.
- `session_stop`.
- The New Agent form (`rh seat add` over SSH): item 6.

## Reference (§20)
N/A — Marley-specific. The verbs are rustal-harness's (`docs/MCP.md` in its repo). The tab's
controls are Marley's own, built from Zed's `Editor::single_line`, `Button` and `picker`.

### Prior art
- **The code we ship:**
  - `marley_fleet::verbs` already types `SendRequest`, `OpenRequest`, `SurfaceRequest`,
    `AnswerRequest` and their receipts.
  - `harness.rs`'s `call` and `HarnessView` (#534).
  - The `picker` crate, used by `agents.rs`'s `NewAgentDelegate`.
  - `rail.rs`'s inbox already opens a harness question's tab (`InboxTarget::Harness`).
- **Published material:** rustal-harness `docs/MCP.md`.
  - The write verbs' arguments and receipts.
  - `--grant write`, and the read grant's refusal of a write.
  - The prompt that names a question's instance.
  - Profiles in `ROOT/profiles/NAME.json`, 0600.
  - Views as `{kind, argv, input}`.
- **Behavior maps:** none.

## UI proof
`script/e2e/689-harness-writes-from-marley.sh` (`compositor sway`, the harness's built `rh`,
as #534 uses it):
- `689-01-question`: the asker's tab, with the question and its option buttons.
- `689-02-answered`: the tab after "main" is clicked.
- `689-03-sent`: a listener actor's tab after text is sent.
- `689-04-views`: the views listed.
- `689-05-picker`: the open-session picker with the typed profile.
- `689-06-opened`: the new session's tab.

## Locked-In Decisions
- **D1:** One switch, `marley.harness_writes`, off by default (AD-661's way).
  - Marley adds `--grant write` only to the command it builds itself, the embedded harness's.
  - A `marley.harness` command is the user's. Its grant is in its arguments, or in an SSH
    `ForceCommand`.
- **D2:** The controls live in the session's tab, which the rail's row and the inbox entry
  already open. No new panel.
- **D3:** Views are listed with Copy. Running one in a pane is item 5.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.harness_writes` is on and a session waits on a question, its tab shall show the prompt and a button per option | shot `689-01-question` |
| REQ-002 | WHEN an option is clicked, the system shall answer with `session_answer`, and the session shall move on | shot `689-02-answered` and a check of the actor's output |
| REQ-003 | WHEN text is sent from the tab, the system shall deliver it with `session_send` and show the receipt's state | shot `689-03-sent` and a check of the actor's output |
| REQ-004 | WHEN Views is clicked, the tab shall list the session's views with their commands | shot `689-04-views` |
| REQ-005 | WHEN a profile name is chosen in `marley: open harness session`, the system shall open a session with `session_open` and show its tab | shots `689-05-picker` and `689-06-opened` |
| REQ-006 | WHILE `marley.harness_writes` is off, the tab shall show no write controls, and the palette shall not list the command | review of the diff |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** the setting, `harness.rs` (writes, the tab and the picker), a review of the diff,
  and the gate.
- **P3 Test:** the scenario, with every shot read.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
