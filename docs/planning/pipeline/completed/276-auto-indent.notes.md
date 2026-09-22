# 276-auto-indent — Notes

- **Forge ticket:** #276 2af791c8-03bb-45b6-93e2-3fa91e3443f8
- **AAR:** 9cfbf20a-0cdd-49a2-85b5-428da6918264
- **Local ticket doc:** docs/planning/tickets/open/TICKET-276-auto-indent.md
- **Pipeline spec:** 276-auto-indent.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 3 of 10.
- **Recon:** Enter today = apply_editor_key → Char('\n') (router-handled
  since #267, stops propagation); Tab today = key_char "\t" → the
  claim-propagate arm → the IME handler inserts a literal tab; ⇧Tab =
  "tab"+shift, NO key_char (mac "backtab" semantics; the terminal raw
  route encodes BackTab as CSI Z — keys.rs:22/100, untouched). The
  editor arm's routing table (the #267 structure) is the insertion
  point: Tab/⇧Tab rows go BEFORE the Char|Other claim. The #89 cooked-
  prompt Tab arm sits LOWER in the ladder and only fires for terminal
  tabs (editor() gate above) — untouched by construction.
- **Selection line-op rebase:** the #255 selection is (anchor, caret)
  char offsets; indent/dedent shifts every line start by ±delta_i —
  each endpoint moves by the sum of deltas on lines strictly before its
  line PLUS the clamp within its own line (an endpoint inside stripped
  whitespace clamps to the line start). Pure fn returns the edits +
  the rebased pair; the shim applies edits back-to-front (the #272
  replace_all idiom).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Pure seams (marley_editor::indent — a new module; buffer-adjacent
ops belong in the pure crate like find.rs)
- `indent_for_newline(line: &str, caret_col_chars: usize) -> String` —
  the leading-whitespace clone clipped at the caret (D1). Returns the
  string to insert AFTER '\n' (the caller inserts "\n" + clone).
- `line_span(buffer, anchor, caret) -> (first_row, last_row)` — the
  touched rows (selection or bare caret).
- `indent_lines(buffer, first, last, tab_width) -> Vec<(CharOffset,
  String)>` — per-line inserts (at each line start, tab_width spaces);
  ascending.
- `dedent_lines(buffer, first, last, tab_width) ->
  Vec<(CharOffset, CharOffset)>` — per-line delete ranges (up to one
  stop of leading spaces OR one \t); ascending; empty per line when
  nothing to strip.
- `rebase_through_line_edits(pos, edits±) -> CharOffset` — reuse the
  #269 `rebase_offset` per edit (the deltas ARE BufferDeltas in
  spirit); implement as a fold over the op list using the same
  boundary convention.
- `spaces_to_next_tab_stop(col, tab_width) -> usize` (max(1) guarded).

### Shim (the editor key arm)
- Enter row: compute the current line + caret col via line_col; insert
  "\n" + indent_for_newline (ONE Buffer::edit — one undo step, caret
  after); clear_marked; stop.
- Tab row: selection → apply indent_lines back-to-front + rebase both
  endpoints; bare caret → insert stop-padding at the caret. ⇧Tab row:
  dedent_lines span (selection or caret line) back-to-front + rebase.
  All stop_propagation.
- Arm placement: new match arms in the editor key routing BEFORE the
  Char|Other claim (Tab has key_char "\t" — must not reach the IME).

### File manifest
| file | change |
|---|---|
| crates/editor/src/indent.rs | NEW — the six fns + tests |
| crates/editor/src/lib.rs | pub mod indent + exports |
| crates/marley_app/src/app.rs | the three editor-arm rows |
| crates/marley_app/src/headless_drive.rs | REQ flows |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | indent_for_newline: full clone, caret-inside-indent clip, no-indent line, tabs-in-indent, empty line; headless Enter-under-indent |
| REQ-002 | indent/dedent_lines + rebase: 3-line selection both ops (exact text + selection covers the same text), mixed tabs/spaces dedent, under-one-stop lines, single line, selection endpoints inside stripped whitespace clamp; headless Tab/⇧Tab on a selection |
| REQ-003 | spaces_to_next_tab_stop cols 0/1/3/4/7 tw4 + tw1 + tw0-guard; headless bare-caret Tab at col 2 → 2 spaces |
| REQ-004 | headless: editor Tab inserts SPACES not \t (buffer text assert); terminal-tab Tab still completes paths (existing tests); raw-mode BackTab golden untouched (existing) |
| REQ-005 | headless ⌘Z after each op restores |
- Uncoverable: the arm rows (app.rs exclude) — the headless flows pin.

### Risks / decisions
- R1 rebase via rebase_offset keeps ONE boundary convention (the #269
  o==e ON-span rule) — the endpoints-inside-stripped-whitespace case is
  exactly its covering-collapse arm.
- R2 dedent of a \t: strips the ONE tab char (not tab_width spaces) —
  per D2; tested.
- R3 Enter's single-edit insert keeps undo one-step (vs newline+indent
  as two) — deliberate.

## Phase 3 — Implement
- **crates/editor/src/indent.rs (NEW):** six pure fns per design —
  `indent_for_newline` (chars().take(caret_col).take_while(ws) — the D1
  clip falls out of iterator order), `spaces_to_next_tab_stop`
  (tab_width.max(1) twin guard), `line_span`, `indent_edits`,
  `dedent_edits` (both clamp rows past the buffer;
  `LineEdit = (CharOffset, usize, String)` — at/remove_chars/text,
  ascending), `rebase_through` (delta fold; inside-a-removed-span
  clamps to the edit's shifted start — the #269 covering-collapse
  convention, restated locally).
- **Deviations from the design sketch (both simplifications):**
  (1) `indent_lines`/`dedent_lines` were designed to return edits +
  the rebased pair; shipped as `indent_edits`/`dedent_edits` returning
  ONLY the edit list, with `rebase_through` a separate fn the shim
  applies to each endpoint — smaller seams, each independently
  kill-listable. (2) rebase reuses the #269 *convention* but not
  `rebase_offset` itself — a `LineEdit` is a char-space triple (not a
  `BufferDelta`), and one 12-line fold with a single boundary rule
  beats converting types to borrow a fn.
- **crates/editor/src/lib.rs:** `pub mod indent;` (namespaced —
  callers read `indent::indent_edits`; no flat re-export).
- **crates/marley_app/src/app.rs:** two rows in the editor key arm
  BEFORE the Char|Other claim (Tab's key_char "\t" must never reach
  the IME): the Tab row (`key == "tab"`; ⇧ or selection → line ops
  applied back-to-front + both endpoints rebased; bare-caret Tab →
  display-col stop padding via the #250 `line_layout`/`col_of_offset`
  so tabs in the prefix count) and the Enter row (`key == "enter"`;
  type-over replace per #255, then ONE `Buffer::edit` inserting
  "\n"+clone — one undo step, R3). Both `clear_marked` +
  `stop_propagation` + `notify`. The claim comment's "space/tab" list
  updated to "space".
- `cargo check` + clippy `-D warnings` + fmt clean; full suite 965/965
  green (no existing pin broke — nothing asserted bare-\n-under-indent
  or a literal editor \t).

## Phase 3.5 — Inspect
### Self-review (run in parallel with the critics)
- **Modifier leak — CLEAR:** the whole editor arm is gated
  `!platform && !control` (app.rs:5513-5516), so ⌘/⌃-modified
  Tab/Enter never reach the new rows (they fall to the raw route as
  before — #251/#267 comment). ⌥Enter/⌥Tab DO reach them, matching the
  pre-#276 treatment of ⌥-variants as plain inserts on the editor.
- **Bar ordering — CLEAR:** the #272 efind arm runs at app.rs:5339,
  before the keymap dispatch (~5492) and the editor arm (5513) — an
  open bar owns Tab (focus toggle) and Enter (nav) first; no shadow.
- **Empty selection — CLEAR:** `active_selection()` normalizes
  anchor==caret to `None` (editor_surface.rs:187-199) — a stale
  zero-width anchor still takes the bare-caret Tab path (D4 exact).
- **tab_width parity — CLEAR:** `code_tab_width` is the same width the
  render (`CodeViewState::new` 2602), caret layout (4967) and click
  mapping (4995) use — ops and pixels agree.
- **Key mapping — CLEAR:** "enter" → `Key::Enter` unconditionally
  (app.rs:1724) — the old path WAS the `_` arm (router-handled, stop);
  the new row intercepts the same class earlier. Pre-#276 Tab was
  `Char('\t')` via key_char → claim → IME literal tab (the ticket's
  bug); ⇧Tab was `Other` → claimed no-op. `apply_editor_key`'s Enter
  stays live via the terminal cooked prompt — no dead code.
- **No render-memo input — CLEAR:** the rows read buffer/caret/anchor/
  `code_tab_width` only; the #272 stale-frame class (memo → edit)
  can't apply. efind memo self-heals by (owner, BufferVersion, query)
  key at render head after any indent edit.
- **rebase_through `.max(0)`:** provably unreachable for
  builder-produced lists (per-line disjoint spans ⇒ at+delta ≥ 0);
  kept as an API-misuse belt. cargo-mutants does NOT mutate method
  calls (#201 lesson) so it is not equivalent-mutant food.

### Critic findings ledger (2 critics: indent arithmetic ×13 probes;
routing/state ×10 hunts + filtered suites 101 green)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| F1 | [MED, BOTH critics] Enter-over-selection = TWO undo steps (delete then insert); the arm comment claimed "ONE edit"; diverges from the #255 one-step type-over. Probe h9: one ⌘Z left the selection deleted — a state the user never saw. | REAL | ONE `buffer.edit(s..e, &insert)`: the (row,cin)+clone computed BEFORE the replace — `indent_for_newline` reads only chars strictly before `s` on its line, untouched by the delete (probe h9b proved byte-identical). Bare caret degenerates to the same call (s==e==caret). |
| F2 | [MED] `line_span` drags in a row the selection touches only at col 0 (shift+Down = the most common gesture) and `indent_edits` pads EMPTY lines → stray whitespace-only lines; VS Code/Zed exclude both. | REAL (reference parity) | col-0 carve in `line_span` (endpoints picked via `.min()`/`.max()` METHOD calls — an `a_row <= c_row` orientation branch is an EQUIVALENT MUTANT at row-equality since the col is dead when `last==first`; methods are unmutated) + empty-line `.filter` in `indent_edits`. Both independently killable (carve: non-empty col-0-end selection; filter: empty line mid-selection). D4 intact (carve gated `last > first`). |
| F3 | [MED] REQ-005 "⌘Z restores the pre-op text and selection shape" unsatisfiable: multi-line Tab = one undo record PER LINE (undo.rs coalesces only 1-char appends) and the undo verb restores caret only, never the anchor. | REAL (spec bug) | REQ-005 reworded to the shipped #272 per-edit v1 convention (Enter = one step; Tab/⇧Tab unwind per-line; full sequence restores exact text). Grouped-undo/transaction seam = recorded follow-up (Phase 5 ticket). |
| F4 | [MED] ⌘Enter on an editor tab SUBMITS the hidden terminal prompt (cooked fallthrough at ~5796; `key_from_keystroke` maps enter regardless of modifiers); ⌃Enter/⌃Tab raw-route bytes to the hidden PTY; ⌘Backspace edits the invisible prompt. | REAL but PRE-EXISTING (#251-era; byte-identical before this diff) | No #276 change — follow-up ticket at Phase 5: gate the terminal fallthrough for modified Enter/Tab/Backspace when the active tab is an editor. |
| F5 | [LOW] `rebase_through` doc said "strictly before" while inserts shift at p==at (the same-text convention); mutants 84/87 `<`→`<=` are killed ONLY by the p==at indent case (dedent boundary is algebraically equivalent). | REAL (doc + test-plan) | Docstring rewritten (AT-or-before + the p==at push-right rule); validate MUST pin `rebase(0)` through a 2-line indent (kills both). |
| F6 | [LOW] `strip > 0` `>`→`>=` mutant is TEXT-INVISIBLE (a `(at,0,"")` no-op edit changes no text but bumps version + records an empty undo). | REAL (test-plan) | Validate asserts LIST SHAPE: a no-indent line contributes NO edit (`edits.is_empty()` for "a\n\nb" dedent). |
| F7 | [LOW] Enter's undo coalescing now depends on the landing line's indent (bare "\n" still coalesces into a typed run; "\n    " is its own step). | NOTED, no change | Cosmetic; the indented case is arguably better. |
| F8 | [LOW] clear_marked/selection read order drifted from the sibling rows (#256/`_` arm read sel first). | REAL (cosmetic) | Both rows reordered: selection read, then `clear_marked`. |
| — | Mutant-set intel (real `--list`: 40 viable pre-fix; re-list after the F2 edits): `CharOffset` derives Default so `rebase_through→Default::default()` IS viable; `line_span` tuple constants need 2 distinct spans; `spaces_to_next_tab_stop` `-`→`+` needs col%w≠0; `indent_for_newline` `=='\t'`→`!=` needs a non-ws char before the caret. | test-plan | Folded into the Phase 4 kill-list. |
| — | Cleared hunts (both critics): modifier gate (⌘/⌃ excluded at 5513); efind arm order (5339 < 5513, flows green); terminal Tab/BackTab byte-identical (goldens green); IME clear_marked convention; tab_width single-source; borrows; efind memo self-heals by (nonce, BufferVersion, query); empty selection→None; ⌘Z double-protected; no render-memo input. | — | — |

## Phase 4 — Validate
- **indent.rs kill-list units (7):** t1 clone/clip/charset (incl. the
  critic's non-ws-before-caret case "a  b"→""), t2 stop-padding cols +
  the max(1) guard (incl. (1,4) for `-`→`/` and (7,4) for `%`→`/`), t3
  line_span orientations + the col-0 carve (incl. bare-caret-col0 at
  ROW 1 — kills `&&`→`||`/`>`→`>=`; a (1,2) span distinct from every
  tuple constant), t4 indent_edits exact lists + empty-line skip
  ("a\n\nb" — kills the filter's delete-`!`) + clamps, t5 dedent strip
  rules + the F6 list-SHAPE assert (no-strip row absent; all-empty →
  `vec![]`), t6 rebase fenceposts (the p==at pin 0→4 — the ONLY killer
  for 98/101 `<`→`<=`; mid-list clamp 8→4; the p==at+remove
  fall-through 10→4; 12→6 for `+`→`*` which would wrongly clamp under
  at*remove; identity), t7 the applied round-trip on a real Buffer
  (indent → exact text + same-text selection (4,17) → dedent → exact
  original + (0,9)).
- **Mutation (real run):** `cargo mutants -f crates/editor/src/indent.rs`
  → **48 mutants tested in 27s: 48 caught** — MSI 100, zero unviable,
  zero equivalent (the F2 `.min()/.max()` endpoint-pick avoided the
  orientation-branch equivalent; re-listed post-fix per the detach-trap
  rule).
- **Headless flows (2, the #264 lane, REAL key ladder):**
  `editor_enter_auto_indent_flows_headless` — EOL clone, D1 clip at
  col 2, type-over Enter → ONE ⌘Z restores the exact pre-Enter text
  (the F1 pin); `editor_tab_indent_dedent_flows_headless` — shift+Down
  shape (anchor 0, caret row3 col0): carve + empty-row skip →
  "    aa\n      bb\n\ncc\n", NO `\t` anywhere (REQ-004), same-text
  selection (4,17); ⇧Tab restores exact original + (0,9); the hidden
  terminal prompt byte-identical before/after; 2×⌘Z re-unwinds the
  ⇧Tab per-line (the reworded REQ-005); bare-caret Tab at col 6 → 2
  spaces + caret 8; bare ⇧Tab dedents the caret line, caret rebases
  to 4. Plus 3 helpers (set_editor_caret / editor_caret /
  editor_selection).
- **REQ map:** REQ-001 = t1 + enter-flow; REQ-002 = t3/t4/t5/t7 +
  tab-flow; REQ-003 = t2 + tab-flow phases 4-5; REQ-004 = tab-flow
  (no-\t + hidden-prompt-identical) + the pre-existing goldens the
  routing critic re-ran green (CSI Z `encode_backtab_and_insert`,
  `input_route_cases`, completion suite); REQ-005 = the two flows'
  undo asserts per the F3-reworded convention.
- **Suite:** `cargo nextest run --workspace` → **974/974 passed, 5
  skipped** (965 pre-existing + 7 units + 2 flows); clippy
  `-D warnings` clean; fmt clean.
- **Driven capture: ENV-BLOCKED (machine asleep/locked)** — the
  full-screen probe returned a pure-black 4K frame (the documented
  locked signature; same as #272/#205/#214). Per the locked-screen
  protocol: no unlock attempt; the interaction is carried by the
  headless lane (simulate_keystrokes drives the SAME on_key_down →
  arm → Buffer::edit path the live app runs — only pixels are
  unverifiable headless) + MSI-100 units. Re-verify live on unlock
  (batched with #272's deferred captures; ~30s, no ticket).
- **Gate:** first `--diff` run RED on gate:14 — the brand-scrub half
  caught "VS Code/Zed observed behavior" in the new `line_span` doc
  (crates/ .rs comments are a brand-free zone since #262; the
  architecture docs are the only allowed home). Rephrased to "the
  universal reference-editor convention" → re-run:
  **GATE GREEN [diff] — 15/15 PASS** (fmt · clippy · tests · audit ·
  deny · machete · gitleaks · shellcheck · no-suppressions ·
  source-bans · docs · coverage 100 · MSI 100 · miri · visual/AX).
  No pre-existing failures; nothing excluded.

## Phase 5 — Complete
- CHANGELOG: #276 entry under "### Added" (above #272's).
- Architecture: `docs/marley_architecture/editor.md` pure-core section
  gains the full `indent.rs` bullet (ops, the col-0 carve + the
  equivalent-mutant avoidance note, rebase conventions, the arm rows,
  the one-edit Enter, the undo convention).
- Knowledge: AAR 9cfbf20a submitted (completed, 3 materialized codes:
  BF-claude-enter-type-over-was-two-undo-steps,
  BF-claude-line-span-col0-endpoint-drags-extra-row,
  PR-claude-composite-type-over-one-edit-001).
- Follow-up tickets minted (out of the /work 272-281 range, on the
  shelf): **#282** grouped-undo transaction seam (F3 — restore the
  stronger REQ wordings in #272/#276 when it lands) and **#283**
  editor-tab modified-key fallthrough gating (F4 — ⌘Enter submits the
  hidden prompt; pre-existing #251-era, bug-typed).
- Forge ticket #276 closed (done). Local ticket → tickets/closed/.
- Lessons (also in the AAR): (1) the PR one-edit rule — a composite
  type-over must be ONE Buffer::edit computed pre-delete; (2) an
  orientation BRANCH picking (row,col) endpoint pairs is an equivalent
  mutant at row-equality — pick by `.min()/.max()` method calls
  (unmutated) instead; (3) the brand-scrub gate reaches NEW doc
  comments — name conventions ("the universal reference-editor
  convention"), not brands, in crates/ .rs; (4) the fifo `read -t`
  timed-wait pattern keeps a turn alive for background critics without
  a blocked foreground sleep.
