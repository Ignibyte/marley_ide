---
pipeline_id: 81293771-51b4-422d-8602-c9eae4ddc4b7
ticket: forge#116 (bc1a1312-eb69-4e08-9b9c-8a6c505e6459) · local docs/planning/tickets/open/TICKET-116-commit-flow.md
aar_id: 9c401f8b-623e-4f36-92e5-73ff0700051b
status: Phase 5 — Complete PASS
title: the commit flow (git WRITE)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/git_diff.rs (PURE: commit_enabled)
  - crates/marley_app/src/app.rs (SHIM: the confined git-write adapter + the commit UI)
---

## Title
Stage files, type a message, and commit — from the git panel. Marley's FIRST git write, tightly confined to
`add` / `restore --staged` / `commit`.

## Scope
### In
- PURE `commit_enabled(staged_count, message)` (≥1 staged AND a non-blank message).
- SHIM: a git-WRITE adapter (stage/unstage/commit); a commit-message input + a Commit button; clicking a
  file toggles its staged state.

### Out
- Push / pull / fetch. Amend, rebase, reset, `--force`, any history rewrite. Partial (hunk) staging. Diffs
  per file (the row still opens the whole working diff).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `commit_enabled` = `staged_count >= 1 && !message.trim().is_empty()`.
- D2 — **SECURITY:** the write adapter runs ONLY `git add -- <path>`, `git restore --staged -- <path>`,
  `git commit -m <msg>` via `marley_command::blocking`, each path/message a SINGLE argv arg (never a shell
  string) → no injection. NO push, NO `-f`/`--force`, NO rebase/reset/amend/history-rewrite. A flash surfaces
  the result.
- D3 — the commit-message input mirrors the #112 session-search focus + key routing.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN there is ≥1 staged file AND a non-blank message, `commit_enabled` shall be true (else false). | unit |
| REQ-002 (security) | The git-write adapter shall run only add/restore/commit — never push/force/rewrite; the message is an argv arg. | code review |
| REQ-003 (visual) | WHEN files are staged + a message typed, the Commit button shall enable; a commit clears the panel. | self-test (env-blocked → engine + review) |
| REQ-004 | gate GREEN, cov/MSI 100 on commit_enabled; the shim masked. | gate |

## Phase Plan
- **P2** — commit_enabled; the write adapter + the commit UI (stage toggle, message input, Commit button); test plan.
- **P3** — implement (git_diff.rs + app.rs).
- **P3.5** — 1 correctness/SECURITY critic: commit_enabled MSI; the adapter confinement (add/restore/commit,
  argv message, no push/force/rewrite); the Commit gate.
- **P4** — commit_enabled tests (cov/MSI 100) + gate GREEN (NOT auto-committing during validation).
- **P5** — docs, AAR, an AD for the git-write confinement, archive, close #116.
