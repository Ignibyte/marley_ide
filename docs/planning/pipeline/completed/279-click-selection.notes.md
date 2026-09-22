# 279-click-selection — Notes

- **Forge ticket:** #279 520339d5-a2e1-4d9f-b36b-6051cb10b500
- **AAR:** 50f7723b-92bb-4873-b34e-e591d0e815d7
- **Local ticket doc:** docs/planning/tickets/open/TICKET-279-click-selection.md
- **Pipeline spec:** 279-click-selection.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan (combined with Phase 2)
- /goal batch ticket 9 of 10 (terminal polish #2).
- **Recon:** the grid left-mouse-down (app.rs ~3238 region) seeds
  `Selection { anchor: pos, head: pos }` via `pane_grid_pos`; the
  copy path (⌘C, :5981) reads `content_row_texts(state)` +
  `copy_payload(&rows, state.selection)` — the SAME rows feed the
  bounds fns (the paired-copy rule holds by construction). The ⌘-click
  link-open lives on the ROW SPANS (open_link_target at :7583), a
  separate listener — the grid-level handler must keep ⌘ behavior
  identical. `click_count`'s only consumer today is the #177
  tab-rename (:6689).

## Phase 2 — Design
- **text_selection.rs (pure):** `CharClass{Word,Space,Punct}` +
  `classify(c)` (Space = whitespace; Punct = `()[]{}<>"',;=` + `` ` ``;
  Word = everything else — `./-_~:@?&#%` land Word by construction);
  `word_bounds_at(row, col) -> Option<(usize,usize)>` (same-class run
  around col; col ≥ chars → None); `row_bounds(row) -> (usize,usize)`
  = (0, chars); `extend_selection(existing, pos) -> Selection`
  (anchor kept; None → seed).
- **Shim (the grid mouse-down, before the #43 seed):** shift →
  `state.selection = Some(extend_selection(state.selection, pos))`;
  click_count == 2 → word bounds over `content_row_texts(state)`'s
  row at pos.row → Selection{anchor:(row,s), head:(row,e)}... head is
  EXCLUSIVE? Verify Selection/row_selection semantics at implement
  (the #43 normalized/hilite convention) and match copy exactly;
  click_count >= 3 → row_bounds. ⌘-click: the branches AND the plain
  seed all skip when `platform` (keep today's behavior for ⌘ clicks
  verbatim — the link precedence test).
- **Manifest:** text_selection.rs (+3 fns + tests) · app.rs (the
  mouse-down branches) · headless_drive.rs (flows).
- **Test plan:** REQ-001 class kill list (paths `/a/b/file.rs`, URL
  `https://x.y/z?a=1`… note `=` in a URL query SPLITS by the class —
  document: the class trades query-string wholeness for `a=b`
  splitting; the D2 set keeps `?&#%` Word so only `=` splits — pin
  the documented trade), punct runs, space runs, multibyte 日本語,
  col-past-EOL, col==len-1; REQ-002/003 headless: build rows via a
  real echo, double-click via... headless mouse events? The #264 lane
  is keys-only historically — CHECK whether VisualTestContext has
  simulate_mouse (gpui test has simulate_event(MouseDownEvent…)?); if
  not, drive the HANDLER LOGIC via the pub(crate) decision path
  (extract the click_count/shift decision into a testable fn taking
  (click_count, shift, platform, existing, pos, rows) → the new
  Selection — the SHIM calls it; units cover the matrix; headless
  optional). Design choice: extract `click_selection(click_count,
  shift, existing, pos, row_text) -> Option<Selection>` as a PURE fn
  in text_selection.rs — the whole decision becomes kill-listable and
  the shim is a one-line call. REQ-004 = the existing single-click
  tests + the pure fn's click_count==1 arm returning the plain seed.
- **Risks:** R1 Selection head semantics (inclusive vs exclusive —
  match row_selection's convention exactly; read it first at
  implement); R2 drag-extend after double-click (the existing
  on_mouse_move overwrites head per cell — plain cell-extend after a
  word-select is ACCEPTABLE v1 (iTerm does word-snap; ours cell-snaps)
  — RECORD as the follow-up rather than build drag-mode state).
- **Autonomy note:** critics on SONNET for the remaining tickets (a
  Fable subagent hit the usage limit during #281).

## Phase 3 — Implement
- text_selection.rs: CharClass/classify (the D2 set), word_bounds_at
  (same-class run; None past EOL), row_bounds, extend_selection, and
  the WHOLE decision as pure `click_selection(click_count, shift,
  existing, pos, row_text) -> Selection` (⇧ first any-count; ≥3 row;
  ==2 word with past-EOL→plain-seed fallback; else the pre-#279 seed
  verbatim). Selection head is EXCLUSIVE-end per row_selection's
  [from,to) — a word (s,e) highlights exactly chars [s,e).
- app.rs: the mouse-down seed became a one-line call over the SAME
  content_row_texts rows copy reads. ⌘-clicks route through the
  single-seed arm (today's behavior — the #196 link listener
  untouched); no platform branch needed.
- Deviation: none. Drag-after-double stays plain cell-extend
  (word-snap drag = the recorded follow-up per the ticket's
  pre-authorization). check + clippy (one unused import) + fmt clean;
  suite 1003/1003.

## Phase 3.5 — Inspect
### Ledger (1 SONNET critic — probes byte-identical to the shipped
fns + a REAL cargo-mutants run; self-review parallel)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| C1 | [MED] `content_row_texts` materialized the ENTIRE unbounded scrollback on EVERY left click — including single/⇧ clicks that never read it (the pre-#279 seed was zero-alloc; the #274 "hot path pays for the whole history" class reintroduced for clicks). | REAL | The fetch is gated on `!shift && count >= 2`; single/⇧ clicks are zero-alloc again. (The true single-row accessor stays a possible micro-follow-up; users click near the bottom anyway.) |
| C3 | [MED] The shipped code dropped the design's `!platform` gate — ⌘-double/triple would paint a transient word/row selection under the #196 link chord (the critic TRACED the link precedence itself holds: on_click fires regardless; the divergence was cosmetic + spec-contradicting; and my Phase-3 "Deviation: none" was wrong against Phase 2's own text). | REAL | ⌘ folds `count` to 1 in the shim — the single-seed arm byte-identical to pre-#279 under any ⌘ click; the notes now reconcile honestly. |
| C2 | [MED, test-intel] classify / extend_selection / the shift branch have ZERO viable mutants — Selection/GridPos/CharClass derive no Default so every body-replacement mutant is UNVIABLE (the known #203/#204 lesson, mutant-list-verified). MSI 100 CANNOT prove the D2 table or D4 anchor rule. | REAL (plan) | Phase 4 hand-authors the per-char D2 vectors (all 14 separators + all 11 word-preserved), both extend arms, and the shift-beats-count pin — coverage-by-intent, not by mutation pressure. |
| — | Cleared (probe-verified): the FULL D2 class table (14 separators + 11 word chars char-by-char; the URL splits exactly at `=` — the documented trade); shift-wins-at-any-count is the spec's own text (a documented iTerm divergence); ⇧-extend keeps the raw anchor (xterm-style, D4 exact); pane_grid_pos clamps and the prompt-row click degrades to the plain seed / a zero-width triple (both harmless, pre-existing shape); the single-click arm is textually the pre-#279 seed; existing text_selection tests 6/6. Drag-after-double cell-extends — recorded follow-up per the ticket (files at Phase 5 if warranted). | — | — |

## Phase 4 — Validate
- **Units (3, text_selection.rs):** the D2 class table asserted CHAR
  BY CHAR (all 14 separators form their own run; all 11 path/URL
  chars stay inside; the `=` query-split documented trade pinned on a
  real URL), word_bounds edges (punct runs, space runs, multibyte,
  col==len-1/len/past, empty row); the click matrix (single=verbatim
  seed, double exclusive-end word, past-EOL fallback, triple+ whole
  row incl. trailing spaces, ⇧-beats-any-count with the anchor
  sacred, ⇧-no-selection seeds, both extend arms directly, multibyte
  row_bounds); the PAIRED-COPY rule (the same Selection through
  copy_payload yields exactly the highlighted text).
- **Gate:** first run RED — gate:5 with ONE missed mutant: my
  while-loop shape made `end = col + 1` EQUIVALENT (the clicked char
  always re-passes its own class check). Fixed at source: iterator
  take_while arithmetic (`col - back`, `col + fwd`) keeps the offsets
  load-bearing — spot-run 20 mutants: 16 caught + 3 unviable + 1
  timeout-detected, 0 missed. Re-run: **GATE GREEN [diff] — 15/15**;
  suite 1006/1006.
- **Headless:** the decision matrix is fully pure (the C2 zero-mutant
  intel made coverage-by-intent the verification spine); the shim is
  a one-line call over the same rows copy reads — no separate
  headless mouse flow (the #264 lane is keys-only; gpui mouse synth
  stays the driven lane's job).
- **Driven capture: ENV-BLOCKED (machine locked)** — double-click a
  path in ls output / triple-click / ⇧-click joins the unlock
  re-verify batch.

## Phase 5 — Complete
- CHANGELOG under "### Added"; app_shell.md's selection bullet
  updated. AAR 50f7723b submitted (materialized:
  BF-claude-hot-click-path-materialized-whole-scrollback). Forge #279
  closed; local ticket → closed/.
- Lessons: (1) a re-checking loop seed is equivalent-mutant food —
  make offsets load-bearing (take_while + arithmetic); (2) the
  zero-viable-mutant classes (no Default derives) need
  coverage-by-intent declared IN the test comments; (3) gate a
  document-derived input's computation on the arms that read it.
