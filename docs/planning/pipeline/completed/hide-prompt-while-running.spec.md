---
pipeline_id: 520c9232-851b-406e-8c47-5b4f8b6bafb7
ticket: forge#193 (968fc19a-2615-4d41-b8c8-b506fc01416d) · local docs/planning/tickets/open/TICKET-193-hide-prompt-while-running.md
aar_id: 4410fd5e-e0bd-46ef-ba7f-bb1f83097916
status: Phase 5 — Complete PASS
title: Hide the shell prompt input row while a foreground command runs
type: feature
milestone: M12.1
references: []
---

## Title
Hide the Marley cooked-shell prompt input row while the pane's session has a
foreground command running (e.g. `claude`), and keep the viewport/content-row
count consistent with what is rendered so scroll math cannot skew. chad live-app
feedback #5 — the "double prompt": when an agent launches, both its own input
area and Marley's ❯ row were visible.

## Scope
### In
- The cooked-block pane render: gate the trailing prompt input row on
  `!state.session.is_command_running()`.
- The content-row accounting (`content_rows`, app.rs): drop the prompt row from
  the count while a foreground command runs, so it matches the render.
- A new PURE helper `content_row_count(visible_rows, prompt_visible) -> usize`
  (nav.rs) that both the count and (conceptually) the render gate derive from —
  one source of truth for "is the prompt row present".

### Out (explicitly deferred)
- Alt-screen / raw-mode rendering (there is no cooked prompt row there anyway —
  #33 handles that surface).
- Hiding the agent's *own* UI (that belongs to the agent, not Marley).
- Any change to `is_command_running()` semantics or the session lifecycle.
- The command-aware tab title (#201) and background-done notify (#203), which
  also read `is_command_running` but are separate tickets.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — Extract a pure `content_row_count(visible_rows: usize, prompt_visible: bool) -> usize`
  in `nav.rs` beside `fold_visible_rows`; bar cov 100 / MSI 100. `content_rows`
  becomes `content_row_count(fold_visible_rows(...).len(), !is_command_running())`.
- **D2** — `prompt_visible = !state.session.is_command_running()` is the SINGLE
  condition; the prompt-row render gate uses the same expression. Render and
  count cannot disagree (the skew failure mode the ticket warns about).
- **D3** — Insert `content_row_count` ABOVE any doc/`#[cfg_attr(test, mutants::skip)]`
  boundary, never between an attribute and its target fn — the documented
  `mutants::skip` detach trap. Verify post-edit with `cargo mutants --list -f nav.rs`.
- **D4** — Validate on the LIVE app (display is awake): driven ⌘⇧A launch `claude`,
  capture that only the agent UI shows, then that the ❯ prompt returns on exit.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pane's session has a foreground command running, the pane render shall not display the trailing prompt input row. | Driven capture (claude running → no ❯ row) + unit test on the gate predicate |
| REQ-002 | WHILE a foreground command is running, the content-row count used for viewport/scroll math shall exclude the prompt row. | Unit + mutation: `content_row_count(v, false) == v` |
| REQ-003 | WHEN the foreground command finishes, the pane render shall restore the prompt input row and the content count shall include it again. | Driven capture (❯ returns on exit) + unit: `content_row_count(v, true) == v + 1` |
| REQ-004 | WHILE the prompt row is hidden, the scrollback blocks shall remain visible. | Driven capture (blocks still shown while claude runs) |
| REQ-005 | The prompt-row render gate and the prompt-row count shall derive from the same `is_command_running` condition (no independent duplication). | Review (single predicate) + the REQ-002/003 mutation tests kill a skewed count |

## Phase Plan
- **P2 Design** — confirm the exact render site of the ❯ input row (app.rs pane
  body ~4110–4260); the `content_row_count` signature + placement; the manifest;
  the regression test table (one row per REQ). No code.
- **P3 Implement** — add `content_row_count` (nav.rs, above any skip); rewrite
  `content_rows` to call it; gate the input-row render on `!is_command_running`.
- **P3.5 Inspect** — critics vs the diff (skip-detach check, off-by-one in the
  count, does the gate compose with scroll-to-bottom / find / fold).
- **P4 Validate** — unit+mutation on `content_row_count`; run the gate; drive the
  live app (claude launch) and capture the before/after.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR capture; close #193 (closes
  the M12.1 sprint's last open ticket).
