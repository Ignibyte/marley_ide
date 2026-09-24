---
pipeline_id: 999f8226-1633-4c31-9d3f-3deae99085df
ticket: docs/planning/tickets/closed/TICKET-479-attach-a-file.md
status: Phase 4 — Complete PASS
title: Attach a file to an agent's prompt
type: feature
slice: prong 1 T7c
references: [docs/planning/pipeline/queued/477-agent-bar.spec.md]
---

## Title
An Attach File button in the agent bar that types the chosen files' paths into the terminal.

## Scope
### In
- A `+` button, "Attach File", in the agent bar (#477), and the action `marley::AttachFile` for
  the command palette.
- It opens `Workspace::prompt_for_open_path` for files, several allowed (the desktop portal's
  chooser, or Zed's own path prompt when the setting or the portal says so), and sends the
  chosen paths with `TerminalView::add_paths_to_terminal`: absolute, shell-quoted, one space
  around each, as one paste (bracketed while the program asked for it).

### Out (explicitly deferred)
- A fuzzy search over the project's files, and `@` mentions (rich input, later).
- Images as images: a path to one serves Claude Code, which reads the file.

## Reference (§20)
- **Warp:** the agent toolbelt's + and attaching files and images for context
  (https://docs.warp.dev/guides/agent-workflows/how-to-use-voice-and-images-to-prompt-coding-agents/),
  seen in Chad's session on 2026-09-23. No Warp code.
- **Upstream Zed:** the terminal's drop handler, which types a dropped file's path
  (`TerminalView::handle_drop`, `add_paths_to_terminal`).

### Prior art
- **Published material:** Claude Code reads a file from a path in the prompt, and from `@path`.
- **Code we already ship:** `Workspace::prompt_for_open_path` returns the chosen paths to its
  caller and falls back to Zed's in-app prompt; tests inject one with `set_prompt_for_open_path`.
  `add_paths_to_terminal` is public, quotes with `shlex`, and pastes.

## UI proof
UI-AFFECTING: a button, a chooser, text sent to the terminal.
- **Driven tests** (`marley_workbench`): with a fake `claude` in the foreground and an injected
  path prompt, clicking Attach File sends the chosen paths, quoted, to the PTY (its write log);
  a cancelled prompt sends nothing.
- **Live drive:** the button in the bar, in a capture; choosing a file needs a pointer, so the
  chooser is not driven live.

## Locked-In Decisions
- D1 — The path goes in as text, as a drop does, so it works for any CLI agent and any shell.
- D2 — Zed's path prompt, not a new picker.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an agent runs, the agent bar shall show Attach File | driven |
| REQ-002 | WHEN files are chosen, the terminal shall be sent their absolute paths, quoted and spaced as a drop sends them | driven |
| REQ-003 | WHEN the chooser is cancelled, nothing shall be sent | driven |
| REQ-004 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion re-verifies the seams and asks the brain.
- **P2 Code** — the button and the action in `marley_workbench`.
- **P3 Test** — driven tests, negative checks, a capture, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
