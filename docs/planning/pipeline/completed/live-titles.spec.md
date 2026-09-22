---
pipeline_id: 6da6bc64-1c2c-47da-b17b-54da9e3b9652
ticket: forge#157 (13cbc967-15c4-4712-9d62-3f9a27ca8171) · local docs/planning/tickets/open/TICKET-157-live-titles.md
aar_id: d2091c32-0201-4af7-8ebc-913a24f9242c
status: Phase 5 — Complete PASS
title: M9 seq-8 — live rail tab titles + real branch subtitle
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/titlebar.rs (PURE: rail_tab_title; reuse branch_from_git_head)
  - crates/marley_app/src/app.rs (SHIM: live_tab_title; the rail Tab + Project render)
---

## Title
The rail comes alive — a terminal tab row shows the running command (not "terminal N"), and a project row
shows its real git branch.

## Scope
### In
- PURE `titlebar.rs`: `rail_tab_title(command: Option<&str>, fallback: &str) -> String`.
- SHIM `app.rs`: `live_tab_title(project, tab, fallback)` (a terminal tab → its focused terminal's latest
  command → rail_tab_title; code/cockpit keep their title); the rail Tab arm uses it; the rail Project arm
  shows `name · branch` (reuse `branch_from_git_head` on `projects()[i].root/.git/HEAD`).

### Out
- A live-updating title as the command streams (recomputed each render is enough). Per-tab icons. Truncating
  very long commands beyond the program token.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `rail_tab_title` shows the PROGRAM (first whitespace token of the command), trimmed; a blank/None
  command falls back to the tab's static title. (Concise, Warp-like.)
- D2 — code tabs (title = file) + cockpit tabs (title = section) keep their title — only terminal tabs go live.
- D3 — the project branch is read per-render from `root/.git/HEAD` via the existing pure `branch_from_git_head`;
  no branch → `name` alone (no ` · `).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rail_tab_title` gets a non-blank command, it shall return the first whitespace token; WHEN blank/None, it shall return the fallback. | unit |
| REQ-002 (visual) | WHEN a command runs in a terminal tab, that tab's rail row shall show the command (its program), not "terminal N". | driven capture |
| REQ-003 (visual) | WHEN a project has a git branch, its rail row shall show `name · branch` (the real branch). | driven capture |
| REQ-004 | gate GREEN; `rail_tab_title` at cov/MSI 100; the shim render masked. | gate |

## Phase Plan
- **P2** — rail_tab_title; live_tab_title + the rail Tab/Project render; test plan.
- **P3** — implement (titlebar.rs + app.rs).
- **P3.5** — 1–2 critics (rail_tab_title edge cases; the live-title nav + branch IO + no-panic).
- **P4** — rail_tab_title tests (cov/MSI 100) + a driven capture (run `sleep 30` → the tab shows "sleep"; the project shows its branch) + gate.
- **P5** — docs, AAR, archive, close #157 → **M9 sprint #20 CLOSES.**
