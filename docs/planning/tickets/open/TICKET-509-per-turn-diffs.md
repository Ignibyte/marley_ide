# TICKET-509 — Per-turn diffs for Claude Code in a terminal

- **Ticket:** LOCAL #509 (feature, prong 2 (review); built on #519)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/509-per-turn-diffs.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 7 of the list after the browser waves). Revised the same night from the Orca survey's notes on #509 and its default for open question 6, "pinned under `refs/marley/turns/`, pruned with the worktree" (`docs/orca_architecture/README.md`; report 02 item 4 and §2.13, report 01 item 3).
- **Status:** open

## Summary
Claude Code in a terminal edits files with nothing in Marley that says what each turn changed. On #519's events, Marley takes a git checkpoint of the terminal's repository when a turn opens (the user's prompt) and when it closes (Stop, StopFailure, a manual `/compact`, or the next prompt, since an interrupted turn sends nothing). Zed's `Repository::checkpoint` does it with a temporary index and touches neither the user's index nor a branch. A turn that changed files becomes a commit whose parent is the turn's start, pinned under `refs/marley/turns/<session>/<n>`, and is listed under the terminal's rail row; a click opens it in Zed's commit view, a diff multibuffer of that turn's changes alone, shell-made edits included.

## Acceptance
Each Claude Code turn that changes files in a Marley terminal is listed under the terminal's row with its prompt and file count; opening it shows that turn's diff and nothing else; the user's index and branch are untouched, and the turn's commit is pinned under `refs/marley/turns/`.
