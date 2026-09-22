# 272-editor-find — Notes

- **Forge ticket:** #272 ef9424f3-45d6-4d67-8d12-c3488cb136d6
- **AAR:** d9e23b8d-57ea-4f12-8114-49362f47835c
- **Local ticket doc:** docs/planning/tickets/open/TICKET-272-editor-find.md
- **Pipeline spec:** 272-editor-find.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 2 of 10 (after #273 —
  this consumes `scroll_editor_to_row`).
- **Recon (grepped this session):** `find.rs` exports `word_range_at` /
  `next_occurrence(buffer, needle, from)` / `select_next_match` (#265,
  tested) — `find_all` is NEW; the #47 terminal find-bar state family is
  `find_open/find_query/find_index` + `handle_find_key` (app.rs:1318,
  Esc/nav via `match_navigation`) — the editor bar mirrors the shape
  with an `efind_*` family + a replace field + field focus; the #273
  scroll helper + two-frame constraint are fresh (the bar operates on
  the ALREADY-OWNED file, so same-frame jumps are safe);
  `styled_slices(syntax, selection)` (code_view.rs:180) is the
  generalization point — the #266 inspect proved its disjoint-output
  contract against gpui compute_runs, so the N-range extension must
  preserve exactly that contract.
- **The #267 rules bind:** the new bar arm must `cx.stop_propagation()`
  (PR-claude-input-handler-overlay-arms-must-stop-propagation-001) and
  join `text_input_blocked()` (the editor's OWN typing must not reach
  the buffer through the IME fallback while the bar owns keys).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Approach (§20 re-confirmed: N/A — Marley's own idioms composed)
Four pieces, each on an existing pattern:

1. **Pure find/replace (marley_editor::find — cov/MSI 100):**
   - `find_all(buffer, needle) -> Vec<(CharOffset, CharOffset)>` —
     ASCII-case-insensitive (D4 RESOLVED: matches the #47 terminal
     find's ASCII-fold precedent; the scan walks CHAR indices so
     multibyte is exact), non-overlapping (advance past a match),
     ordered; empty needle → empty.
   - `replace_all(buffer, matches, repl)` — applies BACK-TO-FRONT via
     `Buffer::edit` (earlier offsets never shift); returns the count.
     D3 RESOLVED: one undo step PER edit is the v1 semantics (distinct
     ranges do not coalesce) — recorded as a limitation; ⌘Z unwinds one
     match at a time.
2. **The highlight-channel generalization (code_view.rs):**
   - `pub enum MarkTier { Match, Current }`
   - `styled_slices_with_marks(syntax, selection, marks: &[(Range<usize>,
     MarkTier)]) -> Vec<(Range, TokenKind, bool /*selected*/,
     Option<MarkTier>)>` — the SAME cut-at-every-boundary pass extended
     with mark boundaries; per slice the covering mark's tier (Current
     beats Match when overlapping); Plain+unselected+unmarked dropped.
     Preserves the #266 ascending/disjoint/char-boundary contract.
   - `styled_slices` becomes a THIN DELEGATE (empty marks, tuple
     mapped) — zero duplicate logic, every existing test still pins it.
3. **Bar state + routing (app.rs, the #47 family):** `efind_open`,
   `efind_query`, `efind_replace`, `efind_index`, `efind_focus_replace`
   + a render-path memo `refresh_efind_matches()` keyed
   `(nonce, version, query)` (the #268 memo pattern — covers buffer
   edits, file switches, query edits in one choke) storing
   `efind_matches: Vec<(usize, usize)>`. The key ladder gains ONE arm
   adjacent to the #47 `find_open` arm, and it OBEYS #267:
   `cx.stop_propagation()` on every consumed key + `efind_open` joins
   `text_input_blocked()`. Key table: Esc close · Enter next/⇧Enter
   prev (sets selection to the match + `scroll_editor_to_row(row)` —
   the file already owns the handle, same-frame safe) · Enter with
   replace-field focus = Replace One + advance · ⌘Enter = Replace All ·
   Tab toggles field focus · Backspace pops · printable pushes (via
   key_char) · ⌘V pastes into the focused field. "open-editor-find"
   seeds the query from a single-line selection (D5); reopening
   refocuses the find field.
4. **Keymap:** two Editor-context rows — ⌘F → "open-editor-find",
   ⌘A → "select-all" (chords_unique_scoped holds: ⌘F Terminal row is a
   disjoint context; ⌘A has no global row). "select-all" verb: anchor=0,
   caret=len_chars on the active surface (clear_marked per #267).
5. **Render feed:** the row closure already computes `sel_bytes`; add
   per-row match slices — `matches_in_row` via `partition_point` over
   the match starts (the #268 idiom, O(log n) per row) → row-local
   display-byte ranges through the SAME row_selection_cols/cols_to_bytes
   path → `styled_slices_with_marks`; the style map adds two
   backgrounds: Match = a dim accent wash (≈0.16), Current = brighter
   (≈0.35) — both under the selection tint's family so they read as one
   system. The bar renders as a compact overlay row pinned above the
   editor body (the #47 bar's render idiom).

### File manifest
| file | change |
|---|---|
| crates/editor/src/find.rs | + find_all + replace_all + tests |
| crates/editor/src/lib.rs | export find_all, replace_all |
| crates/marley_app/src/code_view.rs | + MarkTier + styled_slices_with_marks; styled_slices delegates; tests |
| crates/marley_app/src/keymap.rs | 2 Editor rows (+ the tie-pin test rows) |
| crates/marley_app/src/app.rs | efind_* state/init; refresh_efind_matches; the bar arm; the bar render; row-closure mark feed; the 2 verbs; text_input_blocked join |
| crates/marley_app/src/headless_drive.rs | the REQ flow tests |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | find_all units: fold both directions ("FOO" finds "foo"), non-overlap ("aaa"/"aa" → one), multibyte hay+needle ("é日" in "xé日y"), empty needle, none, at 0/at end; headless live count |
| REQ-002 | headless: Enter cycles 1→2→wrap→1 with selection set to each match; off-screen match centers (scroll_y moved); ⇧Enter reverses |
| REQ-003 | styled_slices_with_marks units: mark∩syntax cuts, Current beats Match, selection+mark coexist, Plain+unmarked dropped, delegation (old == new with empty marks); driven capture of tints |
| REQ-004 | replace_all units: back-to-front with longer/shorter repl (exact final text), N undo steps restore one-by-one; headless replace-one advances + replace-all rewrites + ⌘Z |
| REQ-005 | headless: bar open → type "x" → buffer UNCHANGED + query grew (the #267 overlay-leak pattern); Esc → typing reaches the buffer again |
| REQ-006 | headless: ⌘A editor selects (0, len); terminal-tab ⌘A unchanged (no selection state change) |
| REQ-007 | existing terminal find tests + the driven capture set |
- Uncoverable: the bar render + arm live in the documented app.rs
  exclude — pinned by the headless flows + driven captures.

### Risks / decisions
- D3/D4/D5 resolved above (per-edit undo v1; ASCII-fold; reseed-on-⌘F).
- R1 match memo keyed (nonce, version, query) — same staleness class the
  #268 nonce fixed; reuses that lesson by construction.
- R2 the bar is STATE-routed like every #47-family bar (no second
  FocusHandle; the ladder arm owns keys while open).
- R3 O(n·m) scan per memo refresh — bounded by keystroke cadence on the
  bar; fine at dogfood sizes (measured class: ms).
- R4 current-match index clamps after every recompute (matches may
  shrink under replace).

## Phase 3 — Implement
- **Built to manifest:** find.rs `find_all` (ASCII-fold char scan,
  non-overlap) + `replace_all` (back-to-front, count returned) + lib
  exports; code_view `MarkTier` + `styled_slices_with_marks` (the
  boundary-cut pass extended with mark cuts; Current beats Match via
  max_by_key; Plain+unselected+unmarked dropped); keymap the 2 Editor
  rows; app.rs — the efind_* state family + init, the
  `refresh_efind_matches` render memo (`(nonce, version, query)` keyed +
  index clamp), `select_efind_current` (selection = match +
  scroll_editor_to_row — the file already owns the handle),
  `handle_efind_key` (Esc/Tab/Enter-next/⇧Enter-prev/replace-focus
  Enter = Replace One/⌘Enter = Replace All/Backspace/⌘V/printables via
  key_char), the ladder arm beside the #47 bar (stop_propagation +
  text_input_blocked joined), the row-closure mark feed
  (partition_point row slice → row_selection_cols → cols_to_bytes →
  the marks variant; band priority selection 0.28 > current 0.35 >
  match 0.16), the two dispatch verbs, and the two-row bar card
  (find "i of N" + replace, focused-field caret marker ▏).
- **Deviations:** (1) `styled_slices` DELETED rather than delegated —
  the row closure now calls the marks variant directly, leaving the
  delegate dead code under -D warnings (§0: no dead code, no
  suppression); its 4 tests were ported to the marks variant with
  empty marks + 4-tuple expectations (same kill power, less code).
  (2) The #265 roster pins (`all_chords_lists_every_binding`,
  `cmd_f_is_terminal_scoped`) updated to the new 45-chord/4-scoped
  truth — the designed change, not a regression.
- **Verification:** check clean; fmt; clippy -D warnings clean; full
  `cargo nextest run --workspace` **956/956**.

## Phase 3.5 — Inspect
- **Critic run:** 1 deep critic — a 54-test probe crate over the REAL
  sources (ropey panics EXECUTED, not theorized), the full
  `cargo mutants --list` census, gpui dispatch traces. A parallel
  self-review found+fixed the tab-switch swallow class BEFORE the report
  (the critic confirmed all three mid-run fixes in-tree).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| S1 (self) | HIGH-class, pre-empted | the efind arm fired on ANY tab — a bar left open + a terminal tab = every key swallowed into the bar, the card drawn over the terminal; and the `_` swallow ate ⌘Z/⌘1/⌘F even on the editor | REAL (critic confirmed the fixes) | FIXED mid-inspect: `handle_efind_key -> bool` with an OWNED-chord set (⌘↵, ⌘V + all plain keys) — unowned ⌘/⌃ chords FALL THROUGH the ladder (⌘Z undo, ⌘1 tabs, ⌘F reseed all work with the bar open); a render-TOP close-choke shuts the bar when no editor tab is active (relocated refresh_efind_matches to the render head so terminal frames run it) |
| F1 | HIGH | **mutants::skip DETACHED from handle_find_key — STRIKE 5 of the memory trap**: inserting handle_efind_key between the R51 doc+attr and its fn re-bound them; `--list` showed 12 resurrected handle_find_key mutants | REAL (list-verified) | FIXED: the R51 doc+skip re-seated directly on handle_find_key; `--list` re-run = 0 handle_find_key mutants, app.rs total back to 17 |
| F2 | HIGH | stale `efind_matches` feed EDIT OFFSETS: two keys between frames (Enter autorepeat; the headless lane dispatches all keys then parks ONCE) → the second replace edits with pre-edit offsets → ropey OOB PANIC (probe-executed) or silent text corruption ("aaa bbb" → "xbb") | REAL (executed) | FIXED: `refresh_efind_matches()` at the TOP of handle_efind_key (no-ops on an unchanged key — zero fresh-path cost); also heals typed-query+⌘Enter-in-one-frame |
| F3 | MED | the covering filter's `r.end > r.start` clause is REDUNDANT (`start <= a && a < end` is unsatisfiable for degenerate ranges) → its `>`→`>=` mutant is EQUIVALENT → MSI<100 | REAL (probe + list) | FIXED: clause deleted from the FILTER (behavior-identical, probe-proven); the PUSH guard kept (load-bearing — degenerate marks must not add cut points; its 3 mutants killable) |
| F4 | MED | SPACE can't be typed into either field (gpui names the spacebar "space" — fails the 1-char guard; the repo fixed this exact gap for renaming_tab) | REAL | FIXED: a "space" arm (push ' ' + index reset) |
| F5 | MED | Replace One never advances when the replacement CONTAINS the needle ("a"→"aa": the re-scan re-matches at `s`, the clamp keeps index there → Enter loops forever). Replace All immune (applies the OLD set once — probe-verified) | REAL (traced) | FIXED: `efind_resume: Option<usize>` parked at `s + repl.chars()`; the refresh consumes it via `partition_point(ms < resume)` before the clamp |
| F6-F8, notes | LOW/INFO | terminal find_open lacks the symmetric close-choke (pre-existing #265 asymmetry — follow-up noted); ⌘F reseed indexes 0 not the seeded occurrence; first Enter skips visually-current match 0 (#47 parity — by design); ⌘X/⌘A act on the BUFFER while the bar is open (coherent with the fall-through model — documented here); a ⌘V multiline needle works end-to-end (probed) but renders a raw \n in the card | assessed | recorded; no code change |

- **Verified clean (probe-executed, 54/54):** find_all fold both
  directions + 'İ'/'ı' exact-only + non-overlap + needle==len + flush-at-
  end + multibyte precision + \n-needles across lines (no equivalent
  mutants — both clauses of the fold OR killable); replace_all exact
  final texts (longer/shorter/empty/adjacent/at-0/at-end) + undo exactly
  N steps front-first + the descending/overlapping precondition panic
  documented (unreachable via find_all); the memo key correct with
  len-0 → .get→None no-op everywhere; row-feed fenceposts (end-at-
  row_start excluded, start-at-\n excluded, spanning matches clip per
  row via row_selection_cols); closed-bar rendering byte-identical to
  pre-#272 (the empty-marks output ≡ old styled_slices — REQ-007);
  text_input_blocked gains efind_open; keymap roster tests pass;
  PERF measured: find_all @8k lines/3-char needle = 17ms release
  (ms-class as designed; 565ms DEBUG — test-lane awareness only).
- **Phase-4 kill list (real census):** find.rs find_all 19 viable
  (the probe's l1_* set covers all — incl. the two timeout-kills
  `+=`→`*=`), replace_all 2 (text + count==2 in one test);
  code_view styled_slices_with_marks — ported tests + NEW rows for the
  push-guard trio (empty-mark-ignored kills `>=`), the covering `&&`→
  `||` pair, the push-condition trio (a Plain+unselected+MARKED slice
  must appear); app.rs delta must stay ZERO (re-verified post-F1).
- **Post-fix verify:** fmt; check; `--list` app.rs = 17 (0 in
  handle_find_key); marley suite 393/393.

## Phase 4 — Validate
- **Kill-list tests:** find.rs — `find_all_folds_ascii_case_both_directions`
  (+ ß/STRASSE exact-only), `find_all_non_overlapping_and_boundaries`
  (aaa/aa, aaaa/aa, nonzero start, needle==len, flush-at-end, empty/
  overlong/empty-hay — the 19-mutant set incl. both timeout-kills),
  `find_all_multibyte_and_multiline` (é日 pairs + a \n-spanning needle),
  `replace_all_back_to_front_exact_texts_and_count` (longer + empty repl,
  count returned), `replace_all_undo_unwinds_one_match_at_a_time`
  (exactly N steps then drained). code_view —
  `marks_cut_tier_and_push_rules` (mark cuts inside syntax; plain+marked
  pushed; Current-beats-Match BOTH orders; sel+mark on one slice; empty
  mark adds NO cuts [kills the push guard's `>=`]; inverted ignored).
- **Headless flows (the #264 lane, 3 new):**
  `editor_find_bar_flows_headless` — ⌘F opens the EDITOR bar (terminal
  bar untouched), typing edits the bar not the buffer, Enter selects
  match 2 at (11,16) then centers the far match (scroll moved >100px),
  Esc returns typing to the buffer, ⌘A selects (0, len).
  `editor_replace_flows_headless` — the F5 case LIVE: needle "a",
  replacement "aa", THREE replace-Enters in one dispatch burst (the F2
  window) advance across occurrences hand-traced to
  "aabaa aaba aba\n" (no loop, no panic, no corruption); ⌘Enter
  replace-all; ⌘Z falls THROUGH the open bar and undoes.
  `efind_bar_closes_on_tab_switch_headless` — the S1 choke: the bar
  dies when the terminal tab activates; terminal ⌘F still opens the
  scrollback bar.
  My first drafts of two asserts hung the lane (the panic-masks-as-hang
  rule, again — both were WRONG EXPECTATIONS: typing after Esc replaces
  the still-live selection wherever it is, and the three-replace trace
  needed the F5 arithmetic done by hand). Fixed the ASSERTS, not the
  code.
- **Driven captures: ENV-BLOCKED (machine locked/asleep — the
  full-screen probe is black; the documented locked-screen protocol:
  no password attempt, fall back to mechanism).** Mechanism carry:
  the match bands ride the IDENTICAL #266 with_highlights channel whose
  pixels were live-proven in 266-1/266-2 (only the alpha table
  extended — unit-pinned); the bar card is the #47 card idiom
  (pixel-proven M2); the headless flows drove the REAL key ladder +
  render end-to-end. Re-verify live (⌘F on movement.rs, 30s) when the
  machine unlocks — no ticket needed.
- **Full suite:** 961/961 through the gate run.
- **Gate:** run 1 RED ×2 — rustfmt (fresh blocks) + clippy
  collapsible_if on the new efind arm (collapsed to `&&` at source).
  Run 2: **GATE GREEN [diff] 15/15** (receipt written).

## Phase 5 — Complete
- (pending)
