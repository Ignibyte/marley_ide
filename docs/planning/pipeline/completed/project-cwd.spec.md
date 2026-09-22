---
pipeline_id: 63498cd2-613c-4cd3-8bc9-e04f2964cb79
ticket: forge#160 (0f7a5507-c378-4b11-a675-e5c68aea8c25) · local docs/planning/tickets/open/TICKET-160-project-cwd.md
aar_id: 801acd3b-6101-4751-8d99-327348f4b7d6
status: Phase 5 — Complete PASS
title: M9 — subsequent terminals/splits/agents spawn in the ACTIVE project's root
type: chore
milestone: M10 sprint #21 closeout
references:
  - crates/marley_app/src/app.rs (SHIM: the 3 spawn sites → spawn_session_in(active root))
---

## Title
"cwd follows the project" fully: ⌘T/+ tabs, ⌘D/context-menu splits, and the new-agent split all spawn in the
ACTIVE project's root — not the app launch cwd. (#156 shipped it for a new project's first terminal only.)

## Scope
### In — SHIM only (app.rs, coverage-excluded)
- `new_terminal_pane`, `split_focused_pane`, the "new-agent" split: `spawn_session` →
  `spawn_session_in(&active_root, …)` with the root cloned before the `&mut self`/closure use.
### Out
- The boot sites (:408/:688 — they ARE the launch-cwd project); per-PANE cwd inheritance (a pane follows its
  project, not its sibling pane's `cd`).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN project B is active, a new tab/split/agent PTY shall start in B's root (not the launch cwd). | driven: 2-project shell → rail-click B → ⌘T → `lsof -d cwd` on the newest zsh |
| REQ-002 | No user-facing spawn site shall still call the cwd-less `spawn_session` (grep-clean besides boot). | grep + self-review |
| REQ-003 | gate GREEN. | gate |

## Phase Plan
P2 folded (a 3-site mirror chore). P3 implement. P3.5 documented SELF-REVIEW (tiny-diff pattern; lenses:
borrow-before-&mut, closure capture, missed sites). P4 the lsof driven proof + gate. P5 docs.
