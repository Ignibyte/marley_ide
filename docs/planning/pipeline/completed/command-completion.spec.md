---
pipeline_id: 5547a43c-2693-479d-b0ef-a02cc941fe19
ticket: forge#183 (22a3f883-77a7-46ab-b2b7-0c75c4b1ed10) · local docs/planning/tickets/open/TICKET-183-command-completion.md
aar_id: 15a5b83a-f1a3-476c-8c47-01485e8ca40f
status: Phase 5 — Complete PASS
title: M12 — Tab-completion beyond dirs: PATH commands + shell history (first word)
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/complete.rs (PURE: is_command_position + merge_candidates)
  - crates/marley_app/src/app.rs (SHIM: complete_at_prompt branches; $PATH basenames cache + history words)
---

## Title
Extend the #89/#96/#178 completion engine: when the caret is on the FIRST word (the command position), Tab
completes against $PATH executables + the session's command history (most-recent-first, deduped) instead of the
cwd listing; a later word keeps the #89 path completion. The #96 popup + #178 live-filter come along for free.

## Scope
### In
- PURE `complete.rs`:
  - `is_command_position(line, word_start) -> bool` — true iff everything before `word_start` (a char offset) is
    whitespace (the word being completed is the first word).
  - `merge_candidates(path_names, history_words, prefix) -> Vec<String>` — prefix-filter both; order = history
    hits (in given order, most-recent-first, DEDUPED) then PATH hits (sorted, excluding any already shown).
- SHIM `app.rs`: `complete_at_prompt` branches on `is_command_position(&line, start)`. Command position →
  candidates = `merge_candidates(&path_cmds, &history_first_words, &word)`; the popup entries snapshot = the full
  merged pool (empty prefix); the same replacement / common-prefix / popup logic. Else → the existing cwd path.
  `path_cmds` = $PATH dir basenames, read once + cached on the view (a session-stable set). `history_first_words`
  = the focused pane's history, first word of each, most-recent-first.

### Out
- Executable-bit filtering (a $PATH basename is treated as a command — matches most shells' PATH hashing).
- Completing flags/subcommands. Refreshing the $PATH cache mid-session (a new install needs a relaunch).
- Fuzzy command match (prefix only, like the dir completion).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `is_command_position` shall be true when only whitespace precedes the word start, false when a non-space word precedes it. | unit + mutation |
| REQ-002 | `merge_candidates` shall prefix-filter both lists, put deduped history hits first (given order) then sorted PATH hits (excluding dups), overall deduped. | unit + mutation |
| REQ-003 (visual/driven) | WHEN the prompt is empty and `ca`+Tab is pressed, the completion shall offer PATH commands (cargo/cat/…); a mid-line arg shall still complete paths. | driven capture |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the two pure fns + the shim branch (+ the $PATH cache + history-words gather). P3.5 1-2 critics
(is_command_position boundary incl. leading whitespace + word_start==0; merge_candidates dedup/order/prefix +
the seen-set; the $PATH read + cache; the popup entries snapshot; mutation). P4 unit + driven + gate.
P5 docs.
