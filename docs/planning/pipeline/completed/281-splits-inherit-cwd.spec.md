---
pipeline_id: ae4f0c91-8c57-46d8-8380-48dbacf1afc0
ticket: forge#281 (ff81ab5c-41a3-4298-8dd4-b2e7fc800f1a) · local docs/planning/tickets/open/TICKET-281-splits-inherit-cwd.md
aar_id: 0d5b24ce-335a-4209-9965-d5c92d4e97e4
status: Phase 5 — Complete PASS
title: New splits/tabs inherit the focused pane's LIVE cwd
type: feature
milestone: M17
references:
  - docs/marley_architecture/app_shell.md
---

## Title
The daily cwd friction: 4 dirs deep, split — and land back at the
project root (#160's decision). Splits (⌘⇧L/⌘⇧J + the context menu)
and new terminals (⌘T new-tab, ⌘D/"+" new-terminal) now spawn in the
FOCUSED pane's live prompt pwd when it's a real directory, else the
project root exactly as before.

## Scope
### In
- Pure: ONE validation core `valid_dir_or(cand: Option<&str>, root)
  -> PathBuf` — non-empty + ABSOLUTE + `is_dir` → cand, else root
  (the absolute check is a STRENGTHENING: a relative candidate would
  resolve against the app process cwd — garbage for both consumers);
  `cwd_or_root` (#205 restore) delegates to it; the new
  `spawn_cwd_for(live, root)` = the same core (alias or direct use —
  design picks the thinnest shape).
- The two spawn fns thread the focused terminal's
  `current_prompt().pwd` through it: `new_terminal_pane` (⌘T, ⌘D,
  "+") and `split_focused_pane` (both axes + the context menu rows).
- UNTOUCHED: #156 open-project + #247 launcher (a new project starts
  at ITS root), the #205 relaunch-restore (its own persisted cwds),
  agent/remote spawns.
- NO settings escape hatch (the ticket's lean: YAGNI — the fallback
  covers every edge; revisit on real demand).

### Out
- OSC 7 / non-shell-integration cwd sources (prompt pwd is the #201
  source of truth).
- Inheriting into new PROJECTS/workspaces.

## Reference (§20)
The universal cwd-inheritance convention (Warp/iTerm/tmux behavior
class — public product behavior; docs/warp_architecture notes Warp's
split-inherits-cwd). Marley-original decision seam over Marley's own
prompt-pwd tracking. No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Inherit from the FOCUSED pane (the split anchor / the pane
  you're looking at), not the active tab's first pane.
- D2 — Validation = non-empty ∧ absolute ∧ is_dir; anything else
  falls back to the project root silently (no flash — the fallback
  IS the pre-#281 behavior).
- D3 — One validation authority: `cwd_or_root` refactors onto the
  same core (the absolute-check strengthening applies to #205
  restores too — a relative persisted cwd resolving against the app
  cwd was a latent wrong).
- D4 — YAGNI on `terminal.split_inherits_cwd`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `valid_dir_or` shall return the candidate iff non-empty ∧ absolute ∧ is_dir, else root — with `cwd_or_root` delegating (the #205 vectors green + a new relative-rejection row). | pure units (tempdir fs) |
| REQ-002 | A split of a pane whose live pwd is a real subdirectory shall spawn its PTY in that subdirectory; likewise ⌘T/⌘D new terminals. | headless (PTY pwd assert) |
| REQ-003 | When the live pwd is unknown, deleted, or relative, the spawn shall fall back to the project root (the pre-#281 behavior byte-identically). | pure units + headless fallback row |

## Phase Plan
- P1+P2 combined; P3 implement; P3.5 critic (site completeness — any
  spawn path missed? the #205 delegation regression matrix); P4
  units + headless + gate; P5 docs.
