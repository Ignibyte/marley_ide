# fix sticky/ambiguous search focus — Notes

- **Forge ticket:** #119 `f4bb512a-f31d-4d91-ad59-db32bd2f818b` · **AAR:** `e7a700d7-c54f-4001-b0d8-94a7c1a14f3d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-119-focus-fix.md

## Phase 1 — Plan
- **Request:** forge #119 (chad-reported "funky search") — the 3 input focus flags aren't mutually exclusive
  or cleared on outside-click; keystrokes hit the wrong box + focus sticks.
- **Pre-flight:** flags set at app.rs 1951/3042/3330; checked in order at 1692/1697/1702; terminal click at 2370.
- **Decisions:** D1 clear_input_focus helper → mutual exclusion + clear-on-outside-click; masked shim, no pure surface.
- **AAR id:** `e7a700d7-c54f-4001-b0d8-94a7c1a14f3d`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **app.rs (masked shim):** `#[cfg_attr(test,mutants::skip)] fn clear_input_focus(&mut self)` sets session_search_focused/commit_focused/top_search_focused = false. Call sites: (1) the session-search box click → clear then session_search_focused=true; (2) the commit-message box click → clear then commit_focused=true; (3) the top-search box click → clear then top_search_focused=true; (4) the terminal-pane click-to-focus (~2370) → clear (outside-click). No pure surface.
- **Test plan:** none (shim-only, masked/coverage-excluded); the gate runs the existing suite. Interactive drive env-blocked → code-reviewed.
- **Risks:** borrow — inside cx.listener the receiver is `view`; call view.clear_input_focus() (a &mut method) then set the field, both on view (sequential, fine).

## Phase 3 — Implement
- **Built:** clear_input_focus() helper (masked) + wired all 4 sites: the session-search / commit-message / top-search box clicks each `clear_input_focus()` before setting their own focus true (mutual exclusion), and the terminal-pane click-to-focus calls clear_input_focus() (clear on outside-click).
- **Verification:** fmt; check 0 err; clippy OK (helper used — no dead code).

## Phase 3.5 — Inspect
- **Method:** self-review of a masked focus fix (no pure surface; the interactive drive is env-blocked).
- **Lenses — no findings:** clear_input_focus zeroes all 3 input-focus flags; each input click clears-then-sets (so exactly one is ever true); the terminal click clears (so keys return to the terminal). on_key_down still checks session→commit→top, but now at most one is true, so no ambiguity + no stickiness. Borrow-safe (view.clear_input_focus() then view.<x>_focused=true, sequential on view). No unwrap/panic. Fixes the reported "funky search". No findings. NOTE: verified by inspection; the click/type end-to-end drive needs synthetic input (env-blocked) — pending the verify-approach decision.
- **Fix applied:** the bug fix itself (this ticket).

## Phase 4 — Validate
- **Tests:** none added — shim-only (app.rs is coverage-excluded + the render/handlers are mutants::skip). The gate runs the existing suite.
- **Self-test:** the fix is interactive (click A/type/click B/type/click terminal) → needs synthetic input, ENV-BLOCKED; code-reviewed for correctness (mutual exclusion + clear-on-outside-click). A live drive is pending the verify-approach decision.
- **Gate:** GREEN [diff] 15/15 (shim-only; existing suite passes).

## Phase 5 — Complete
- CHANGELOG ### Fixed; forge #119 → done. The search/input focus is now mutually exclusive + cleared on a terminal click (fixes chad-reported "funky search"). Shim fix; interactive drive pending the synthetic-input decision.
