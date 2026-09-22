# #96 — the tab-completion popup — Notes

- **Forge ticket:** #96 `e75ddab6-8c2b-45ef-852b-5594ebb4b31f` · **AAR:** `465f7aa8-0f17-4034-b06b-49c6e1dfbd53`

## Phase 1 — Plan / Phase 2 — Design (folded)
- Pure CompletionState (wrap cycle + windowing) beside #89's engine; the shim binds the popup to the
  opening PaneId (#167 uniqueness = an exact staleness guard), routes modal keys early (others dismiss +
  type through — cmd-chords therefore auto-dismiss), accepts by start..caret replace (+space unless dir),
  renders an occluding ≤8-row box above the input. Driven via the empty-word Tab (all entries → prefix ""
  → popup) — sidesteps the synthetic-typing gap.
- **AAR id:** `465f7aa8-0f17-4034-b06b-49c6e1dfbd53`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- complete.rs: CompletionState (wrap cycle via the REUSED tabs::next_index/prev_index; selected_candidate)
  + popup_window(len, selected, max) — short lists whole; else the window ends at the selection.
- app.rs: `completion: Option<(PaneId, CompletionState)>`; the many/prefix-not-longer branch opens it
  pane-bound; the EARLY modal key branch (before #166's — tab/down=cycle, up, enter=accept, esc=dismiss,
  OTHERS dismiss + fall through so typing/⌘-chords continue); accept_completion (pane guard → start..caret
  replace + space-unless-dir, caret/start clamped); the terminal focus-click dismisses; render-LAST an
  occluding box anchored above the opening pane's input row (≤8 rows + "… n more"; rows click-accept;
  self-dismisses if the pane left the screen).
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (routing/fall-through/borrow/accept/window lenses) + my own routing check.

- **Critic verified the core:** ONE on_key_down in the crate (root) — the modal branch is its first
  statement, the #89 Tab branch below it in the SAME closure (no double-fire possible); typing falls
  through to apply_key in the same closure (dismiss-and-type, nothing lost); render is &mut self, the
  clone avoids borrow conflicts, and rect_list is same-frame fresh (the None-arm self-dismiss covers the
  hidden-terminal Tab case, which is pre-existing routing); the input buffer is app-side — the pump never
  touches it, so `start` can't shift while open; complete_word re-prefixes dirs so the full-candidate
  replace is right; enumerate-BEFORE-skip keeps original click indices (#169's lesson held).
- **[MEDIUM → FIXED, self-caught + critic-confirmed] right-click modal priority inversion** — the #166
  menu opened OVER a live popup: mouse to the menu, keys to the popup (Enter would silently edit the
  buffer). The right-click handler now dismisses the popup first (parity with the left-click).
- **[LOW → FIXED] modifier shadowing** — ⌘↑/⌘↓ (jump-block), ⌘⌥-arrows (focus), ⌘/⌃-Enter hit the bare
  nav arms. The arms now require plain (no ⌘/⌃/⌥) keys; shift-Tab cycles BACKWARD (menu convention);
  modified keys fall to `_` (dismiss + normal dispatch) as the comment always claimed.
- **[LOW → FIXED] overflow guards** — rows clip long names (overflow_hidden); the tail row counts ALL
  hidden candidates (above + below the slid window); a short pane shrinks the window (≥2 rows) so the box
  never paints past the pane bottom.
- nextest complete-module 4/4; clippy clean.

Lenses: event routing/ordering, fall-through integrity, render-time mutation, accept-time invariants,
window math + indices, modal exclusivity.

## Phase 4 — Validate
- **Tests:** completion_state_wraps_and_maps (new→0; ↑ wraps 0→last; ↓ wraps last→0; the plain step);
  popup_window_cases (whole ≤max incl. ==; first page; the ==max-1 boundary; slide-by-one; bottom-ends-at-len;
  the max-0 empty window). 2/2 + the #89 four.
- **Self-test:** cp_open2.png — Tab at the empty prompt → the popup above the input row: 8 sorted rows
  (.cargo/ selected in accent), the "… 9 more" tail. cp_accept.png — ↓↓ + Enter → the prompt reads
  `Marley .git/` (a dir: no trailing space), the popup closed. REQ-002 pixel-proven end to end.
  (Harness note: the drive verb is the BARE key name — `tab`, not `key:tab`; the first attempt no-oped.)
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG (Added) + app_shell #96 note; forge #96 → done. **SPRINT #21 FULLY DONE (12 tickets).** LESSONS: modal nav arms need plain-modifier guards; every modal opener dismisses the others; pane-binding via unique ids; empty-word Tab as the driveable ambiguous case.
