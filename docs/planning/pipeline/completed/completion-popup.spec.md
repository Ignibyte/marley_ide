---
pipeline_id: de065ed8-7084-4dcd-9dce-67288aa05670
ticket: forge#96 (e75ddab6-8c2b-45ef-852b-5594ebb4b31f) · local docs/planning/tickets/open/TICKET-96-completion-popup.md
aar_id: 465f7aa8-0f17-4034-b06b-49c6e1dfbd53
status: Phase 5 — Complete PASS
title: Tab-completion popup for multiple matches (follow-up to #89)
type: feature
milestone: M10 sprint #21 closeout (finale)
references:
  - crates/marley_app/src/complete.rs (PURE: CompletionState + popup_window)
  - crates/marley_app/src/app.rs (SHIM: the open branch, the modal keys, accept, render)
---

## Title
The ambiguous-Tab case gets a menu: when Tab finds many matches and the common prefix is already typed, a
popup lists the candidates under the prompt — Tab/↓/↑ cycle, Enter (or click) accepts, Esc dismisses, any
other key dismisses AND types through (zsh-menu feel).

## Scope
### In
- PURE `complete.rs`: `CompletionState { candidates, selected, start }` (wrap move_up/move_down,
  selected_candidate) + `popup_window(len, selected, max) -> (start, end)` (the visible slice, selected
  always in view).
- SHIM: `completion: Option<(PaneId, CompletionState)>` — PANE-BOUND (#167's globally-unique ids make the
  guard exact: a popup opened in one pane can never edit another's buffer); the `many + prefix-not-longer`
  branch opens it; an EARLY modal key branch (before #166's menu branch: tab/down/up/enter/esc; other keys
  dismiss + fall through); accept = replace `start..caret` with the candidate (+ a space unless a dir/);
  render-LAST an occluding box above the input row (≤8 rows + a "… n more" tail; rows clickable); the
  terminal focus-click dismisses.

### Out
- Live filtering as you keep typing (any key dismisses; the next Tab reopens on the narrowed word);
  descriptions/icons per candidate; command/history completion (#89 scope is dir entries).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | CompletionState shall wrap-cycle and map `selected_candidate`; `popup_window` shall clamp and keep the selection visible (boundary mutants killed). | unit |
| REQ-002 (visual) | WHEN Tab yields many matches at the common prefix, a popup shall list them; ↓ then Enter shall insert the selected candidate into the prompt and close it. | driven (empty-word Tab → the full dir listing; down ×2; enter; capture) |
| REQ-003 | The popup shall be pane-bound: acceptance with a different focused pane shall be a no-op dismiss. | code/critic |
| REQ-004 | gate GREEN; the pure surface at cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (modal fall-through vs the #166 branch ordering; accept clamps;
pane-guard; render gating on the grid-visible tab). P4 unit + driven + gate. P5 docs + CLOSE SPRINT #21.
