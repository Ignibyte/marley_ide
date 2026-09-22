---
pipeline_id: 5c109ca0-c094-4c2f-8e11-6f88a7223707
ticket: forge#142 (1e361d9f-4f16-4f03-b824-1d509091528d) · local docs/planning/tickets/open/TICKET-142-titlebar-context.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: cwd + git branch in the title bar [M8]
type: feature
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/src/titlebar.rs (NEW, PURE: abbreviate_path, branch_from_git_head, titlebar_label)
  - crates/marley_app/src/app.rs (SHIM: read HOME + .git/HEAD, render the label in the top bar)
  - crates/marley_app/src/lib.rs (mod titlebar)
---

## Title
The unified titlebar tells you where you are — an abbreviated working directory and the current git branch
(`~/…/Marley · main`), read live, instead of nothing.

## Scope
### In
- PURE `titlebar.rs`: `abbreviate_path`, `branch_from_git_head`, `titlebar_label`.
- SHIM: read `HOME` + `{project_root}/.git/HEAD`; render the label in the top bar, left of the search.

### Out
- Per-pane cwd tracking (uses `project_root`). Fixing the sidebar's hardcoded "main" (separate). Detached-HEAD sha display.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — branch comes from parsing `{project_root}/.git/HEAD` (`ref: refs/heads/NAME`), NOT the per-block
  git_branch or the sidebar's hardcoded "main". Detached HEAD (bare sha) → no branch shown.
- D2 — `abbreviate_path`: `home`→`~`; a `~`-path deeper than 3 segments collapses to `{first}/…/{last}`.
- D3 — the label renders left-aligned in the top bar (after the icons), muted, left of the centered search.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `abbreviate_path("/Users/x/Projects/ignibyte/Marley","/Users/x")` runs, it shall return `~/…/Marley`; a `~/Marley` path stays. | unit |
| REQ-002 | WHEN `branch_from_git_head("ref: refs/heads/main\n")` runs, it shall return `Some("main")`; a bare sha or junk → `None`. | unit |
| REQ-003 | WHEN `titlebar_label(cwd, home, Some("main"))` runs, it shall return `~/…/Marley · main`; branch `None` → just the path. | unit |
| REQ-004 (visual) | WHEN the app runs, the titlebar shall show the abbreviated cwd + branch. | live capture |
| REQ-005 | gate GREEN, cov/MSI 100 on titlebar.rs; the shim masked. | gate |

## Phase Plan
- **P2** — the 3 pure fns; the shim read + render position; test plan.
- **P3** — implement (titlebar.rs + app.rs + lib.rs).
- **P3.5** — 1 self-review: the parse edge cases; the abbreviation; the best-effort read.
- **P4** — the 3 fns' tests (cov/MSI 100) + a LIVE capture + gate GREEN.
- **P5** — docs, AAR, archive, close #142.
