# M12 #180 — agent output tail — Notes

- **Forge ticket:** #180 `81ab5a37-70df-4cea-8993-ee0bcb804f78` · **AAR:** `8d34ce20-8dd8-4e60-ac2a-952bc5da42f9`

## Phase 1 — Plan / Phase 2 — Design (folded)
- pure agent_tail(output, n): strip trailing blanks, last n, trim_end each; the Agents rows stack a muted
  tail under agent_row_text via agent_tail_lines(pane, 6) reading grids().find_map(terminal). The #173 pump
  keeps background agents fresh.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- agent_view.rs: agent_tail(output, n) — pop trailing blank lines, take last n from start=len.saturating_sub(n),
  trim_end each (indentation kept). Empty→[]; n>=len→all.
- app.rs: AGENT_TAIL_N=6; agent_tail_lines(pane, n) (masked shim: grids().find_map(terminal) →
  content_row_texts.join → agent_tail); the Agents row became a flex_col entry = the clickable
  agent_row_text + the muted 11px tail lines (pl_3) stacked below; pushed to the list.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a pure projection + a cockpit render addition):
- THE STRIP/WINDOW EDGES: trailing-blank pop is a while-loop (all-blank output → empty → start 0 → []);
  start=len.saturating_sub(n) (n>=len → 0 → all lines; n=0 → start=len → []); trim_end preserves leading
  indentation (code output stays readable) while dropping the terminal's trailing pad.
- CROSS-GRID READ: agent_tail_lines uses grids().find_map(terminal(pane)) — the #173/#182 pattern; a
  background agent's tail is fresh (the pump drains it); a gone pane → [] (no row tail, the row itself
  already handles the gone case on click).
- RENDER COST: the tail reads content_row_texts (a bounded walk of the pane's blocks) once per agent per
  cockpit render — only when the Agents tab is ACTIVE (cockpit_body runs for the active section), so it's
  not a per-frame all-agents cost; N=6 caps the rows.
- NO REGRESSION: agent_row_text (the row's one-line summary incl. the #78 last_line) is unchanged; the tail
  is ADDITIVE below it.
Lenses: projection edges, cross-grid freshness, render cost scope, additive-no-regression.

## Phase 4 — Validate
- **Tests:** agent_tail_cases — more-than-n→last-n; trailing-blanks stripped before the window; fewer-than-n
  →all; trim_end keeps leading indentation; a MID blank line preserved; empty/all-blank→[]; n==0→[]. 1 new.
- **Self-test (REQ-002):** clean boot, ⌘⇧A launched claude, opened the Agents cockpit tab — the row
  `○ claude (waiting…)` renders a live muted TAIL below it: claude's welcome-banner tail
  (`…able-5-promotional-access)`), its `❯ Try "create a util logging.py…"` prompt suggestion, and the
  status bar (at_tail1_crop.png). A second frame 5s later (at_tail2_crop.png) shows the row transitioned
  `○ waiting` → `● working` while the tail persists — the cockpit LIVE re-renders via the #173 pump. The
  observe surface works: watch an agent's recent output without focusing its pane.
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG (Added) + app_shell #180 note; forge #180 → done. **M12 3/10.** LESSONS: grids().find_map(terminal) is now a settled idiom for "read an agent live wherever it lives" (3rd use); trim_END keeps code indentation in a multi-line tail; a status transition across frames IS the auto-refresh liveness proof.
