# 267-b2-input — Notes

- **Forge ticket:** #267 7f1f13d4-b1fb-4334-bb1a-bba77a0a8eca
- **AAR:** 669ad392-f54b-41d7-86f8-7d670061de95
- **Local ticket doc:** docs/planning/tickets/open/TICKET-267-b2-input.md
- **Pipeline spec:** 267-b2-input.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 9 of 10. B2 input half (IME).
- **API ground truth (vendored gpui 0.2.2 — verified BEFORE the spec, the
  #264 lesson):** `EntityInputHandler` (input.rs:10 — 8 methods, UTF-16
  ranges) + `ElementInputHandler::new(bounds, entity)` (input.rs:77) +
  `Window::handle_input(focus, handler, cx)` (window.rs:3400 —
  paint-phase-only, focus-gated, per-frame) + `UTF16Selection`
  (platform.rs:982). ALL present.
- **Dispatch order PROVEN (the load-bearing fact):**
  `Window::dispatch_keystroke` (window.rs:3540 — the test-platform path
  `simulate_keystrokes` drives): (1) the KeyDown event dispatches through
  the listener ladder; (2) ONLY IF `result.propagate` still true AND
  `keystroke.key_char` is Some AND an input handler is registered →
  `input_handler.dispatch_input(text)`. `with_simulated_ime`
  (keystroke.rs:241) fills `key_char` for printables (shift → uppercase)
  + space/tab/**enter → "\n"**, skipping cmd/ctrl/fn/alt chords. The REAL
  mac path (platform/mac/window.rs) wires `insertText:replacementRange:` /
  `setMarkedText:selectedRange:replacementRange:` (NSTextInputClient) with
  the same router-first contract.
- **Consequences carried into the spec:**
  1. Marley's editor arm (app.rs:4737-4801) today handles chars via
     `apply_editor_key` and NEVER calls `stop_propagation` → registering a
     handler without cutting the char arm = DOUBLE INSERT. → D1: the char
     arm is DELETED; remaining editor arms stop propagation (Enter's
     key_char "\n" makes this mandatory, not stylistic).
  2. The arm's `!platform && !control` gate lets ⌥-chars through today
     (⌥E inserts a plain 'e' — no dead-key). Post-#267 ⌥E falls through
     to the OS composition path (the driven REQ-006 proof).
  3. Registration inside the #266 editor-branch canvas → paint-scoped =
     terminal tabs NEVER register (their prompt/PTY path untouched by
     construction).
- **UTF-16 seam ground truth:** ropey 1.6.1 ships
  `try_char_to_utf16_cu`/`try_utf16_cu_to_char` (rope.rs:706/727) — the
  Buffer seam wraps the `try_` forms (no unwraps, §14) with clamping.
- **Recon:** the editor key arm + selection semantics live at
  app.rs:4737-4801 (#255 idioms); `EditorSurface`/`OpenFile` (editor_
  surface.rs) needs the per-file `marked` span; the #266 x0 canvas is the
  natural `handle_input` site (bounds + window + app in paint).
- **Forge recall:** knowledge-context surfaced the API-verify-first PR
  (applied — this plan), the selftest focus/capture rules, and the fresh
  #266 PRs; no IME-specific priors exist yet.
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### The two routing hazards (found by reading the ladder, not guessed)
- **H1 — overlay double-insert.** Every text-capturing overlay arm above
  the editor arm (renaming_tab, naming_workflow, completion, context_menu,
  session_search, commit_focused, top_search, diff, agent_launcher,
  history_open, finder_open, palette_open, find_open) `return`s WITHOUT
  `stop_propagation` — so with a handler registered, finder typing (and
  finder-Enter's key_char "\n") would ALSO reach `replace_text_in_range`.
  FIX (single choke point, closes the class for future overlays too): a
  `text_input_blocked()` predicate on RootView (the OR of those overlay
  states); the handler's MUTATING methods no-op and the query methods
  return None while blocked. Auditing/stopping propagation in 13 arms is
  the rejected alternative (easy to miss one; reopens per new overlay).
- **H2 — hidden-terminal fallthrough.** `focused_terminal_mut()` resolves
  even when an editor tab is active (the #71 finder-insert behavior), so
  if the char arm were simply DELETED, a printable would fall through the
  ladder into the cooked-prompt arm — corrupting an invisible terminal
  prompt AND then inserting via the handler. FIX: the editor arm KEEPS
  claiming plain chars but with the split below.

### The editor key arm's new routing table (the heart of the change)
With an editor tab active (same `!platform && !control` guard):
| key class | action |
|---|---|
| motions (arrows/Home/End/word, ±shift) | handled as today + `cx.stop_propagation()` + return |
| Backspace/Delete (incl. selection-replace) | handled as today + stop + return |
| Enter | handled as today (`apply_editor_key` → `\n`) + stop + return — Enter MUST stay on the router: real macOS routes Return via `doCommandBySelector(insertNewline:)`, not `insertText`, so the handler path would drop it; and its simulated key_char "\n" would double-insert without the stop |
| Tab / other deferred keys | swallowed no-op + stop + return (behavior unchanged — tab-insert policy is a later ticket) |
| **plain `Key::Char(_)` (incl. space)** | **`return` WITHOUT handling and WITHOUT stop** — the arm claims the key (the lower terminal ladder never runs) but leaves propagation true, so the platform text path (mac `insertText:` / test `dispatch_keystroke` fallback) delivers it to the input handler. Type-over-selection works because `ime::replace_text(None, …)` targets marked→selection→caret |

### Pure seams (marley_editor)
- `Buffer::char_to_utf16(CharOffset) -> usize` / `utf16_to_char(usize) ->
  CharOffset` — ropey `try_char_to_utf16_cu`/`try_utf16_cu_to_char`,
  clamped at both ends (never panic; out-of-range → end).
- NEW `ime.rs` (free fns over `(&mut Buffer, &mut CharOffset caret, &mut
  Option<CharOffset> anchor, &mut Option<(CharOffset,CharOffset)> marked)`):
  - `replace_text(…, range_utf16: Option<Range<usize>>, text)` — target =
    explicit range → *marked* → selection → caret; `Buffer::edit(…,
    EditOrigin::Human)` (undo-recorded); caret = target.start + chars;
    anchor = None; marked = None. (NSTextInputClient `insertText:`.)
  - `replace_and_mark(…, range_utf16, new_text, new_sel_utf16:
    Option<Range<usize>>)` — same target rule; after the edit the marked
    span = inserted text (empty `new_text` ⇒ marked = None per the
    protocol); composition selection: `new_sel_utf16` is RELATIVE to
    `new_text` → converted via a pure str-level `utf16_ix_to_char_ix` →
    absolute anchor/caret (equal ends ⇒ bare caret). None ⇒ caret at end.
    (`setMarkedText:`.)
  - `unmark(marked)` — clear (composition finalizes in place).
  - `text_for_range(buffer, range_utf16, adjusted: &mut Option<…>) ->
    Option<String>` — clamps, writes the adjusted (clamped) UTF-16 range
    back, returns the slice.
  - `selected_utf16(buffer, caret, anchor) -> (Range<usize>, bool)` +
    `marked_utf16(buffer, marked) -> Option<Range<usize>>` — pure; the
    shim wraps `UTF16Selection`.

### Shim (app.rs, all inside the documented skip/exclude)
- `EditorFrameGeom { x0: f32, y0: f32, first: usize, last: usize, cell_w:
  f32, cell_h: f32 }` (Copy) replacing the bare x0 Cell: the SAME
  first-row canvas records origin.x AND origin.y + the rendered range +
  cell — every IME geometry answer becomes pure arithmetic (no
  scroll-handle reads): caret rect = `(x0 + ccol·cell_w, y0 +
  (crow−first)·cell_h)`; `character_index_for_point` inverts it then
  reuses `offset_for_click`.
- `impl EntityInputHandler for RootView`: thin routing → the ime ops on
  the active surface via a new combined accessor
  `active_ime_mut() -> (&mut Buffer, &mut caret, &mut anchor, &mut
  marked)`; every method None/no-op when no editor tab is active OR
  `text_input_blocked()`.
- Registration: the first-row canvas's (currently empty) PAINT closure
  calls `window.handle_input(&focus_handle, ElementInputHandler::new(
  bounds, entity), cx)` — paint-scoped ⇒ only a painted editor frame
  registers; terminal frames never do (their path untouched by
  construction).
- `EditorSurface`: `OpenFile.marked` + `active_marked()` +
  `active_ime_mut()` + `clear_marked()`; marked cleared at the non-IME
  edit/interaction sites (click-down, the arm's edits, ⌘X/⌘V, undo/redo)
  — the inspect critics sweep for missed sites.

### File manifest
| file | change |
|---|---|
| crates/editor/src/buffer.rs | + char_to_utf16 / utf16_to_char + tests |
| crates/editor/src/ime.rs | NEW — the ops above + utf16_ix_to_char_ix + tests |
| crates/editor/src/lib.rs | export ime |
| crates/marley_app/src/editor_surface.rs | OpenFile.marked + accessors + clear_marked |
| crates/marley_app/src/app.rs | EntityInputHandler impl; EditorFrameGeom Cell; canvas registration; the key-arm routing table; marked-clear sites; text_input_blocked() |
| crates/marley_app/src/headless_drive.rs | the REQ-004/005 + overlay-gate + hidden-terminal tests |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | buffer utf16 seam on "aé日😀b" (cu map 0,1,2,3,5,6; char map inverse; clamps at len+7) — mutants killed |
| REQ-002 | replace_text targets: explicit range / selection / marked / bare caret; caret-after; anchor+marked cleared; ONE undo step restores |
| REQ-003 | replace_and_mark scripted dead-key (⌥E: mark "´" sel 0..1 → commit "é") + 2-stage CJK (mark "に" → extend "にほ" → commit "日本") + empty-text unmark + relative multibyte selection |
| REQ-004 | headless: editor tab `simulate_keystrokes("x")` → buffer "x" (handler-only path); AND the hidden terminal's prompt cooked buffer UNCHANGED (H2 proof); terminal-tab typing tests unchanged |
| REQ-005 | headless: Enter → exactly one "\n"; Backspace/motions as before (existing asserts re-run through the new arm) |
| REQ-006 | driven: ⌥E then E → é rendered in the editor (capture) + persisted via ⌘S file readback |
| REQ-007 | direct trait-method tests through the window (window.update → view.selected_text_range/text_for_range/marked_text_range) over multibyte content |
| H1 | headless: finder open over an editor tab, type "x" → buffer UNCHANGED, finder query advanced |
- Uncoverable: the ElementInputHandler registration + mac insertText wiring
  are platform/shim surface — asserted by the headless dispatch fallback
  (same contract, proven at plan) + the driven ⌥E capture.

### Risks / decisions
- R1 overlay gate list drifts from the ladder → the predicate is defined
  ADJACENT to the ladder with a comment tying them; critic-checked.
- R2 `key_from_keystroke` char classification must exactly match the
  "claims but doesn't handle" split (space!); verified by the headless
  matrix.
- R3 ime.rs mutants: Option/Vec returns have Defaults (viable mutants);
  Range arithmetic traced with `cargo mutants --list` per the PR rule.
- R4 First-frame geom Cell zeros (same class as x0 today): candidate
  window may anchor at origin for the very first composition frame —
  self-heals next frame; documented, not fixed (matches #254 precedent).
- R5 undo granularity of composition: each replace_and_mark edit is one
  Buffer edit; the commit is another — ⌘Z after é un-does the commit to
  the marked intermediate. Accepted v1 (matches simple editors);
  composition-collapsing undo is future work with anchors (#269 gives the
  tools).

## Phase 3 — Implement
- **Built to manifest:** `Buffer::{char_to_utf16, utf16_to_char}` (ropey
  `try_` forms, clamped both ends); NEW `marley_editor::ime` (pub module) —
  `edit_target` (explicit-range→marked→selection→caret), `replace_text`,
  `replace_and_mark` (incl. relative-UTF-16 composition selection via
  `utf16_ix_to_char_ix`; empty new_text = protocol cancel), `unmark`,
  `text_for_range` (clamp + adjusted write-back), `selected_utf16`
  (reversed-aware), `marked_utf16`; `EditorSurface` — `OpenFile.marked`,
  `active_marked()`, `active_anchor()` (the RAW directional anchor —
  `selectedRange` reports reversed selections), `active_ime_mut()`
  (4-way disjoint borrow), `clear_marked()`; app.rs — `EditorFrameGeom`
  (x0→full frame geometry: x0/y0/first/last/cell, one Cell, recorded by
  the SAME first-row canvas), the canvas PAINT closure now registers
  `ElementInputHandler` via `window.handle_input` (paint-scoped,
  focus-gated), `impl gpui::EntityInputHandler for RootView` (8 methods →
  ime ops; geometry = pure cell arithmetic off the recorded geom;
  `text_input_blocked()` gates every text answer/edit), the editor key
  arm's new routing table (Char|Other → claim-and-propagate `return`;
  handled classes → `clear_marked` + `cx.stop_propagation()` + notify),
  and `clear_marked()` at the non-IME edit sites (click-down, ⌘V paste,
  ⌘X cut, undo, redo).
- **Deviations from design (both recorded):**
  1. `Key::Other` joined `Key::Char(_)` in the claim-and-propagate class —
     dead keys (⌥E) arrive with NO key_char → `Other`; stopping them would
     kill the IME path entirely (the design table only named Char).
  2. The design table's "Tab = swallowed no-op" was WRONG about today's
     behavior: Tab maps to `Char('\t')` via key_char and INSERTS — it
     stays in the Char class (now via the handler), byte-identical net.
- **Verification:** `cargo check --workspace` clean; fmt; full
  `cargo nextest run --workspace` **915/915**. Honest coverage note: the
  existing lane exercises the restructured arm's CHORD + editor-open paths
  (cmd-d/cmd-t/open_file_in_viewer) — no existing test types a PLAIN char
  into the editor, so the handler insert path is first covered by Phase
  4's REQ-004 headless test; 915 green proves no regression in covered
  behavior, not the new path.

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel — A (IME ops correctness: a probe crate mounting
  the REAL sources via path-dep; every lens probe-executed; the REAL
  `cargo mutants --list` census) + B (routing/regression: every dispatch claim
  verified against the vendored gpui source with file:line citations; mac +
  test platform both traced). Plus a self-review pass run in parallel while
  the critics worked (3 findings found+fixed before their reports landed —
  both critics independently CONFIRMED all three).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| S1 (self) | MED | `active_project()` panics on the zero-project launcher state; the 8 handler methods called it unguarded (reachable in the close-last-workspace → next-paint gap — mac queries `selectedRange` eagerly) | REAL (B confirmed: latent; defense-in-depth) | FIXED: `active_editor()`/`active_editor_mut()` helpers gate on `project_count()==0`; all 8 methods routed through them |
| S2 (self) | LOW-MED | Esc-deselect regression — the old `_` arm collapsed the anchor for EVERY unhandled key; the claim-and-propagate split dropped it | REAL (B confirmed; also proved pre-diff Esc was otherwise a no-op, and mac's `cancelOperation:` path makes the stop safe) | FIXED: explicit `escape` arm — deselect + clear_marked + stop |
| S3 (self) | MED | `completion` in `text_input_blocked` swallowed editor typing (its printables FALL THROUGH by design — click-switch to an editor tab with a live popup → chars eaten) | REAL (B confirmed "CORRECT for printables") | FIXED: removed from the predicate + comment |
| A-F1 | MED | TWO equivalent mutants (`<`→`<=` under `a != caret` guards, ime.rs:39/:150) = guaranteed MSI<100 | REAL (list-verified) | FIXED: `min`/`max` refactor in `edit_target` + `selected_utf16` (`reversed = a == e`); re-`--list` proves both GONE (64-mutant census; the one remaining `<`→`<=` in `utf16_ix_to_char_ix` kills on any exact-boundary case) |
| A-F2 | MED | `utf16_ix_to_char_ix` mid-surrogate rounded UP — contradicting its own doc AND the buffer seam (ropey rounds down) | REAL (probe: cu4 on "aé日😀b" → 4 vs seam 3) | FIXED: round-down (`cu < units + len_utf16`) — seam parity; doc updated |
| A-F3 | LOW | a no-op IME call (`insertText:@""` at a bare caret — input-source switches send these) pushed an empty undo record and DESTROYED the redo stack | REAL (probe: redo → None) | FIXED: both ops skip `buffer.edit` when target-empty ∧ text-empty (bookkeeping still runs) |
| A-F7 | INFO | `MarkedSpan` ordering is convention-only; an inverted span would panic in ropey | hardening | FIXED free: the marked arm normalizes via `s.min(e)..s.max(e)` (method calls — zero new mutants) |
| A-F8c | INFO | `unmark_text` skipped the `text_input_blocked` gate the other 7 methods check | consistency | FIXED: gate added |
| B-F1 | HIGH | overlay-consuming keys LEAK their key_char into the editor buffer via the HEADLESS input-handler fallback (every overlay arm returns with propagation live; `text_input_blocked` evaluates AFTER the arm flipped its state off — finder plain-Enter lands "\n"; top-search Enter on a File hit opens the editor then dirties it with "\n"). Real macOS unaffected (branch-B `doCommandBySelector` discards) — but the enforced test lane IS headless | REAL (gpui-cited: window.rs:3540-3561, keystroke.rs:241-263, mac/window.rs:1716/1736-1765) | FIXED: `cx.stop_propagation()` in all 13 full-capture overlay arms + the completion HANDLED branch (fall-through printables stay live) |
| B-F2 | MED | stale marked span survives ⌘D/⌘-arrow/file-switch — the next IME insert targets the DEAD span (edit_target prefers marked) | REAL | FIXED: clear_marked at the keymap-dispatch choke (ANY chord verb ends composition — over-clearing merely finalizes in place), the ⌘-arrow arm, and `EditorSurface::{open, close, activate}` |
| A-F4 | INFO | R5's wording vs reality: the FIRST preedit char can COALESCE into a preceding typed-run undo record (probe: "type a, compose é" → undo#1 "a´") — fewer steps, same accepted intermediates | verified | R5 stands with this coda; the Phase-4 undo-ladder test encodes the coalesced reality |
| A-F5/F6, B-F3 | INFO | replacementRange treated document-absolute (de-facto standard; doc sentence added to edit_target); mid-pair ranges snap DOWN (never split a pair); bounds_for_range anchors at the caret cell (documented v1); `geom.last` break-overstatement unreachable in practice | verified | doc-only |

- **Residual (recorded, accepted):** a mouse-only rail tab-switch away+back with a
  live composition can still leave a stale marked (no chord fired) — self-heals on
  the first click into the editor (click-down clears); LOW, folded into the F2 sites
  if it ever bites.
- **Verified clean (critics, evidence-backed):** dispatch-order + single-delivery
  proofs (no double-fire path exists for any handled key; mac Enter carries "\n" so
  the stop is load-bearing); Key::Other enumeration — pre-diff the arm consumed all
  of these too, so NO lower ladder arm was reachable on an editor tab before or
  after (zero lower-arm regressions); ⌥-chords: mac fills key_char (⌥A="å") →
  Char → propagate → insert — correct; per-frame handler registration is gpui's
  intended model (draw() re-takes each frame); terminal frames register nothing;
  the headless lane's typing reaches `replace_text_in_range` (test platform
  round-trips the handler); `text_input_blocked` 1:1 with the ladder (nothing
  missing; nothing after the editor arm captures typing); the full edit_target
  precedence/state-machine/truth tables probe-exact; §20 clean (Apache-2.0 public
  API; impl bodies original; no correspondence to the GPL editor's version).
- **Phase-4 kill list (critic A, real census post-refactor 64 mutants):** T1 seam
  tables; T2 rel-seam incl. cu=0/boundary/past-end/mid-pair-3; T3 explicit-range;
  T4 BOTH selection orientations; T5 `→()`; T6 asymmetric caret math (1,3); T7
  dead-key stage-1 at start=1; T8 empty-cancel; T9 multibyte rel-sel; T10
  collapsed rel-sel; T11 mark-over-selection; T12 unmark; T13/T14 text_for_range
  exact/past-end (adjusted None/Some); T15 selected_utf16 truth table ×3; T16
  marked_utf16 Some. Buffer seam: 3 viable killed by T1.
- **Post-fix verify:** fmt; check clean; **915/915**; `--list` re-run confirms the
  equivalent-mutant class is gone.

## Phase 4 — Validate
- **Kill-list tests (critic A's census, all landed):** buffer.rs T1 (seam
  tables on "aé日😀b" incl. mid-pair round-down + past-end + empty — kills
  the 3 viable seam mutants); ime.rs T2–T16 as 13 tests (rel-seam
  boundaries; explicit-beats-marked; BOTH selection orientations +
  degenerate; asymmetric caret math + one-step undo round-trip; dead-key
  mark/commit at non-zero start; empty-cancel; 2-stage CJK growth;
  multibyte rel-sel + collapsed + the mid-pair round-down end-to-end;
  mark-over-selection; unmark; text_for_range adjusted protocol ×5;
  selected_utf16 truth table ×4; marked_utf16; the A-F3 no-op undo/redo
  guard; the A-F7 inverted-span defense; the A-F4 coalesced undo ladder).
  marley_editor 93/93.
- **Headless (the #264 lane, 8/8):** NEW
  `editor_typing_inserts_via_the_input_handler_headless` (REQ-004: "x"
  arrives through the handler; REQ-005: Enter = exactly one "\n",
  router-stopped; H2: the hidden terminal prompt byte-identical),
  `finder_over_editor_never_reaches_the_buffer_headless` (the B-F1
  regression: overlay typing blocked, finder-Enter's "\n" does NOT leak,
  typing resumes after close),
  `input_handler_methods_are_utf16_correct_headless` (REQ-007: the trait
  methods through the window over multibyte content + a trait-level
  insert + bounds_for_range resolves from the recorded geometry).
- **Driven (REQ-006, bundled fresh binary, PNGs read):** drive.swift
  gained the `alt:` verb (⌥-chord — the #218 harness-extension precedent).
  `267-1-deadkey-marked.png`: after synthetic ⌥E the editor shows the
  ´ preedit at the caret with the dirty ● — the REAL macOS
  NSTextInputClient path delivered `setMarkedText:` into
  `replace_and_mark`. `267-2-deadkey-committed.png`: after `e` the ´ was
  REPLACED IN PLACE by **é** (one glyph — the marked-span targeting
  proven on pixels). First working IME composition in Marley. The scratch
  edit was ⌘Z'd back out.
- **Full suite:** 918+/918 across the final gate run (marley 383, editor
  93, all others unchanged).
- **Gate:** run 1 RED ×5 — rustfmt (fresh blocks), clippy
  `reversed_empty_ranges` (an inverted `3..1` DATUM — my own #266 PR
  violated; struct-literal spelling applied), rustdoc (module-doc links
  unresolved under the merged outer doc — de-linked, redundant outer doc
  dropped), coverage (editor_surface's new accessors shim-only-reached →
  2 direct unit tests added; buffer.rs' 2 DEAD unwrap_or_else closures —
  restructured to clamp-then-direct-ropey-call, no dead lines), MSI
  (the same editor_surface accessors → killed by the new units). Run 2:
  coverage-only RED (buffer closures). Run 3: **GATE GREEN [diff] 15/15**
  (receipt written). `--list` re-verified after every body change (the
  4-strike rule): ime.rs 64, buffer utf16 3 viable, no skip-detach.

## Phase 5 — Complete
- (pending)
