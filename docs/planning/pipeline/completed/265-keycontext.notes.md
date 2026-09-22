# 265-keycontext — Notes

- **Forge ticket:** #265 73614d2c-09e9-46d9-9fb2-f3da97a08c94
- **AAR:** 6f75fb5a-6d6c-4584-bd60-c54282656727
- **Local ticket doc:** docs/planning/tickets/open/TICKET-265-keycontext.md
- **Pipeline spec:** 265-keycontext.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal run, ticket 2 of 10. B1 KeyContext — context-scoped
  keybindings, fix ⌘D (+ ⌘F folded in as the second live collision).
- **Classification / tier:** feature, single pipeline. Mechanism (pure keymap
  context model) + two collision fixes + net-new editor select-next-match v1.
- **Forge recall (§18.3):** knowledge-context (Plan) logged 13 surfacings.
  Priors applied: pure-seam discipline (shim excluded, pure layer cov/MSI
  100); run `cargo mutants --list` on the ACTUAL code before claiming the
  mutant set; per-pane focused-only affordances must gate on is_focused.
- **Discovery (Explore agent + reads):**
  - ONE key entry point: root `.track_focus(&self.focus_handle)` +
    `.on_key_down` at app.rs:4201/4206; a 27-step first-refusal ladder —
    overlays (13 flags) → hardcoded ⌘F 4451 / ⌘⇧D 4462 / ⌘⇧C 4472 →
    editor-gated ⌘-arrows 4483 + ⌘C/X/V 4518 → terminal ⌘C/⌘V 4567/4578 →
    **flat table 4603** → editor plain keys 4617 → terminal raw/cooked paths.
  - Focus facts: editor-active = `shell.active_project().active_tab()
    .editor().is_some()` (TabContent::CodeView(EditorSurface) — tab-level);
    terminal = `workspace_mut().focused_terminal().is_some()` (PaneContent in
    the grid). Overlay flags are 13 RootView bools.
  - gpui 0.2.2 (crates.io) SHIPS the whole engine (KeyBinding::new with
    context string, KeyBindingContextPredicate::parse + pub eval_inner,
    KeyContext, actions!, bind_keys, on_action) — but its keystroke→action
    resolution walks the FOCUS TREE (DispatchTree/dispatch_path), and Marley
    populates no per-surface focus. Native adoption = rewire the whole
    single-listener architecture → OUT of this slice.
  - Marley uses ZERO gpui keymap API today; keymap.rs is a pure gpui-free
    seam (41 bindings, action_for = first match, chords_unique global guard +
    debug_assert at :220; roster test pins len()==41).
  - Editor has NO select-next-match machinery (movement.rs has word MOTION,
    selection.rs single Selection{anchor,head}) → ⌘D-in-editor is net-new
    pure work.
  - ⌘/ is unbound everywhere (free); Enter/Esc are already context-routed by
    the overlay ladder; palette shows a static ⌘D chip on "New Terminal"
    (app.rs:3966) — stays, D6.
- **Decisions:** D1–D6 in the spec. Key one: Marley-original pure model in
  keymap.rs per the subsystem doc §A.7 (not gpui types — keeps the seam
  gpui-free + fully mutation-testable; gpui's native path needs the focus
  tree Marley doesn't have).
- **Autonomy note:** /goal Stop-hook run — proceeding to Phase 2 without a
  human pause (chad pre-authorized the batch).

## Phase 2 — Design

### Architecture / approach
Three pure layers + one shim wiring, all Marley-original (§20 confirmed: the
MECHANISM's semantics mirror the gpui Apache-2.0 model as documented in
subsystem-09 §A; no copyleft source read; select-next-match is the public
editor idiom implemented on Marley's own Buffer/Selection):

1. **keymap.rs context model** (pure, gpui-free — unchanged discipline):
   - `pub enum KeyContext { Terminal, Editor }` — v1 surface identity.
   - Bindings become `(KeyBinding, String, Option<KeyContext>)` (None =
     global). Constructor helpers keep the vec literal readable.
   - `action_for(&self, chord, stack: &[KeyContext]) -> Option<&str>`:
     rank = context-bound → `stack.iter().rposition(|c| c == ctx)` mapped to
     `Some(depth)`; ineligible (context not in stack) → skipped; global →
     rank "below any stack index". Max rank wins. **Ties are impossible by
     construction**: the scoped uniqueness guard forbids same (chord,
     context) — including two globals — so no order rule is needed (cleaner
     than the reference's insertion-order tiebreak; documented).
   - `chords_unique_scoped(&[(KeyBinding, Option<KeyContext>)]) -> bool` —
     HashSet over the pair; the `debug_assert` in `default_bindings()`
     upgrades to it. The old global `chords_unique` is deleted (its only
     caller was the assert + its test).
   - Table delta: + `(Editor, cmd-d → "select-next-match")`,
     + `(Terminal, cmd-f → "open-find")`; `cmd-d → "new-terminal"` STAYS
     GLOBAL (preserves today's cockpit-tab behavior; Editor's deeper match
     shadows it on the editor tab per REQ-003).
   - `all_chords()` keeps its shape (roster count 41 → 43).
2. **Editor select-next-match** (`crates/editor/src/find.rs`, NEW, pure):
   - `word_range_at(buffer, off) -> Option<(CharOffset, CharOffset)>` —
     char at `off` is a word char → expand both ways; else char before
     `off` is (caret at word end) → that word; else None. Reuses
     `movement::is_word_char` (visibility → `pub(crate)`).
   - `next_occurrence(buffer, needle, from) -> Option<(CharOffset,
     CharOffset)>` — char-wise scan `[from..]` then wrap `[0..from]`; the
     sole occurrence re-finds itself (visible no-op; deterministic).
     Naive O(n·m) over `buffer.text().chars()` — fine at v1 file sizes
     (B2/B3 own perf later). Empty needle → None (guard).
   - `select_next_match(buffer, caret, selection) -> Option<(s, e)>` —
     non-empty selection → needle = `text_in_range`, search from sel end;
     else `word_range_at(caret)`. Returned range is the new selection.
3. **Context-stack builder** (`tabs.rs`): `TabContent::key_context(&self)
   -> &'static [KeyContext]` — `CodeView → [Editor]`, `Terminal(_) →
   [Terminal]` (tab-level: a FileTree-focused pane still publishes Terminal,
   matching today's ⌘D/⌘F reach), `Cockpit → []`. Pure method, unit-tested.
4. **app.rs shim wiring** (inside the coverage-excluded shim):
   - DELETE the hardcoded ⌘F arm (app.rs:4450-4460); its 3 lines become the
     `"open-find"` dispatch arm (find_open=true, clear query, index 0).
   - At the table step (4603): build
     `let stack = …active_tab().key_context()` (after `.content` access per
     tabs.rs shape) and call `action_for(&binding, stack)`.
   - New dispatch arm `"select-next-match" => self.editor_select_next_match()`
     — gets `editor_mut()`, calls the pure fn with (buffer, caret,
     active_selection), on Some sets `*anchor = Some(start); *caret = end;
     cx.notify()`.
   - The overlay ladder and every other hardcoded arm are untouched (D5).

§14: no new IO, no unwrap/expect on reachable paths (all Option-flows), no
new shared types across crates (KeyContext stays in marley_app; the editor
fns speak Buffer/CharOffset — types the editor crate already owns).

### Reference (§20) — confirmed
As planned: Zed is the behavior reference via subsystem-09 (§A model, §D ⌘D
worked example); mechanism semantics = gpui (Apache); implementation 100%
Marley-original. No change.

### File manifest
| file | change |
|---|---|
| crates/marley_app/src/keymap.rs | KeyContext enum; context-carrying bindings; `action_for(chord, stack)` rank resolution; `chords_unique_scoped`; 2 new table rows; tests reworked |
| crates/editor/src/find.rs (NEW) | `word_range_at` / `next_occurrence` / `select_next_match` + unit tests |
| crates/editor/src/movement.rs | `is_word_char` → `pub(crate)` (one line) |
| crates/editor/src/lib.rs | `pub mod find;` |
| crates/marley_app/src/tabs.rs | `TabContent::key_context()` + tests |
| crates/marley_app/src/app.rs | ⌘F arm deleted; stack passed at the table step; `"open-find"` + `"select-next-match"` dispatch arms; `editor_select_next_match` shim method |

### Regression test plan
| REQ | Test(s) | Kind |
|---|---|---|
| REQ-001 | keymap: `[Terminal]` + ⌘D → "new-terminal"; `[]` (cockpit) + ⌘D → "new-terminal" (global fallback) | unit |
| REQ-002 | find.rs: word-under-caret (mid-word / at-end / on-space→None / multibyte); repeat advances; wrap past EOF; sole-occurrence self-find; empty buffer; arbitrary (non-word) selection as needle | unit (≥8 cases) |
| REQ-003 | keymap: `[Editor]` + ⌘D → "select-next-match" (deeper beats global); context binding ineligible off-stack (`[Terminal]` never yields it) | unit |
| REQ-004 | keymap: `[Terminal]` + ⌘F → "open-find"; `[Editor]` + ⌘F → None; `[]` + ⌘F → None | unit |
| REQ-005 | `chords_unique_scoped`: same chord+same context rejected (incl. two Nones); same chord+disjoint contexts accepted; default table passes | unit |
| REQ-006 | the full existing roster test updated to `action_for(chord, &[Terminal])` (and spot-checks under `[Editor]`/`[]` for the global cockpit chords); len == 43 | unit |
| REQ-007 | driven self-test: terminal tab ⌘D → pane count +1 (capture); editor tab ⌘D ×2 → visible selection highlight advances, pane count unchanged (captures read + asserted) | visual (validate step 3) |
| — | `TabContent::key_context` all three arms | unit |
| — | mutation: `cargo mutants --list` on find.rs + keymap.rs resolution AFTER implementing (PR-trace-the-real-list); kill-all target | gate:5 |
- Uncoverable: none new — the pure layers are fully testable; the app.rs
  wiring lives in the existing documented shim exclude (asserted by the
  driven capture instead).

### Risks / decisions
- R1 `action_for` signature change ripples ~14 keymap tests — mechanical,
  bounded to one file.
- R2 Cockpit-tab ⌘F becomes a no-op (was: opened the find bar over a grid
  that isn't there). Deliberate scoping win; noted for the CHANGELOG.
- R3 Naive scan in next_occurrence — v1-acceptable; perf substrate is B2.
- R4 Roster len test 41→43 — the intended regression guard doing its job.
- R5 Palette ⌘D chip stays static on "New Terminal" (D6) — truthful in its
  context; per-context chips deferred.
- R6 The construction debug_assert MUST move to the scoped guard in the same
  edit as the new ⌘D row, or default_bindings() panics in tests.

## Phase 3 — Implement
- **Built exactly to manifest:**
  - keymap.rs: `KeyContext{Terminal,Editor}` (Copy+Eq+Hash); bindings →
    3-tuples with `Option<KeyContext>`; the 32-row global literal preserved
    (built as 2-tuples, mapped to `(c, a, None)` — keeps the literal readable,
    scoped rows in their own commented block); `action_for(chord, stack)` with
    rank = global 0 / scoped rposition+1, strict-max wins, off-stack scoped →
    ineligible; `scoped_entries()`; `chords_unique_scoped` (old global
    `chords_unique` + its test deleted); debug_assert upgraded; 2 new rows
    (Editor ⌘D → select-next-match, Terminal ⌘F → open-find).
  - editor find.rs (NEW): `word_range_at` (rope-char scan, caret-on or
    caret-after-word pivot), `next_occurrence` (char-wise, [from..] then wrap
    [0..from), sole occurrence self-finds), `select_next_match` (composed ⌘D
    decision); `movement::is_word_char` → pub(crate) (shared word class);
    lib.rs flat re-exports per idiom.
  - tabs.rs: `Tab::key_context()` → Terminal/[Terminal], CodeView/[Editor],
    Cockpit/[] with the design's doc.
  - app.rs: hardcoded ⌘F arm DELETED (comment breadcrumb left); table step
    passes `active_tab().key_context()`; new dispatch arms
    `"select-next-match"` (pure fn → set anchor/caret, None = no-op, matches
    the undo/redo arm idiom) + `"open-find"` (the 3 lines the deleted arm
    did).
- **Test rework in the same file (compile-necessary):** all action_for call
  sites gained a stack arg (TERM const; mechanical python rewrite + 1 manual
  multi-line fix); chords_unique test → scoped-guard cases; roster test →
  len 43 + scoped-roster asserts (exactly 2 scoped rows); NEW tests:
  keycontext_resolves_cmd_d_by_surface, cmd_f_is_terminal_scoped,
  deeper_context_wins_between_scoped_bindings (synthetic two-scoped table —
  kills rposition/rank mutants), ⌘Z-on-ED global spot-check. (Phase 4 runs +
  extends; writing these here was required for `cargo check --workspace` to
  compile the crate's test target.)
- **Deviations from design:** none functional. The literal-preserving
  `global: Vec<(KeyBinding,String)>` + map construction is the design's
  "constructor helpers keep the vec literal readable" realized concretely.
- **Verification:** `cargo fmt --all` applied; `cargo check --workspace`
  green (only the pre-existing upstream `block v0.1.6` note).

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel — A = algorithm correctness (50+ concrete probes
  against the real crate via a scratchpad probe bin: every rope.char bound
  hand-traced, EOB/multibyte/CJK/emoji/wrap/overlap/reversed-selection all
  exercised; full 27-step ladder re-read; 417/417 tests green);
  B = behavior-regression + reuse + provenance (palette path traced —
  invokes dispatch_action directly, never action_for → no editor-tab
  regression; overlay ordering quoted line-by-line; alt-screen ⌘F unchanged;
  gpui-0.2.2 keymap source spot-compared → different types/traversal/
  semantics, incl. the OPPOSITE global-binding precedence rule; clippy
  clean).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| A1 | LOW | ⌘D wrap can land the selection off-screen (no caret-follow scroll) | REAL but PRE-EXISTING editor-wide (#257 ⌘↓ identical) | follow-up ticket at Phase 5 (post-validate polish rule) |
| A2 | LOW | occurrences overlapping the current selection unreachable ("aaa"/"aa") | REJECTED as defect — documented search-from-selection-end semantics; matches the reference idiom | none |
| A3 | LOW | KeyContext not re-exported though public API takes it | REAL (latent) | FIXED: lib.rs re-export |
| A4 | INFO | find.rs ships zero tests in-diff | AS PLANNED — Phase 4 owns the suite; critic's probe menu adopted | Phase 4 |
| B1 | MED | find.rs = 50 real mutants w/ no tests yet; keymap `>`→`>=` tie mutant unkillable via default_bindings (guard forbids ties) — needs a synthetic same-context-dup literal test; test comment claimed a nonexistent rposition mutant | REAL (validate-blocking if ignored) | comment FIXED now; tie-pin test + find.rs suite = Phase 4 (explicit) |
| B2 | MED | SPEC-app-shell.spec.md (R19 chords_unique / R51 unconditional ⌘F / action_for arity) + app_shell.md flat-model prose now stale | REAL doc drift | Phase 5 doc pass (explicit checklist below) |
| B3 | LOW | chords_unique_scoped cloned entries (old guard inserted refs) | REAL idiom regression | FIXED: insert refs |
| B4 | LOW | stale PRIOR-session Zed clone (GPL) on disk outside the repo | REAL hygiene | FIXED: rm -rf'd |
| B5 | INFO | next_occurrence is O(n·m·log n) via rope.char | acknowledged in-file; B2/B3 territory | none |

- **Phase 4 obligations (from inspect):** find.rs unit suite (probe menu:
  word-at-0 / EOB-after-trailing-word / after-trailing-newline→None /
  "a.b"-on-dot→left-word / on-word-beats-after / single-char / whole-buffer /
  _-digit / é+CJK / emoji-nonword / zero-width-selection→word (kills
  find.rs:73 `>`→`>=`) / wrap / self-find / from>last_start / multi-line
  needle); keymap tie-pin test via a `Keymap{bindings:…}` same-chord-
  same-context literal (kills keymap `>`→`>=`); run `cargo mutants --list`
  on both files for the REAL set.
- **Phase 5 obligations:** SPEC-app-shell R19/R51/action_for-arity deltas +
  app_shell.md keymap section rewrite + caret-follow-scroll follow-up ticket.
- **Post-fix verify:** cargo check green on both crates.

## Phase 4 — Validate
- **Tests added:** find.rs suite (17 tests over the inspect probe menu:
  word-at-0/mid/single-char; EOB-after-trailing-word + past-len clamp;
  after-trailing-newline→None; punctuation→left-word; ON-beats-after;
  underscore/digit/é/CJK/emoji word class; whole-buffer word; advance→wrap;
  sole-occurrence self-find; from>last_start empty-tail; empty/overlong
  needles + empty buffer; char-offset multibyte scan; case-sensitivity;
  composed first-press/repeat/wrap; zero-width + reversed selection fallback
  [kills find.rs:73 `>`→`>=`]; no-word None; multi-line + non-word needles).
  keymap: `equal_rank_keeps_the_first_binding` (synthetic same-context-dup
  literals — kills the action_for `>`→`>=`/`==`/`<` family the guard makes
  unreachable through default_bindings) + the Phase-3 context tests; tabs:
  `key_context_maps_tab_kind_to_surface` (all 3 arms — kills the
  `Vec::leak(Vec::new())` mutant; the `vec![Default::default()]` variant is
  unviable, no Default on KeyContext).
- **Mutant trace (real `--list`, per the forge rule):** find.rs = 50;
  keymap.rs = 21 total — action_for 10 (3 return-value + `!=`→`==` chord
  filter + `==`→`!=` rposition predicate + `+`→`-`/`*` rank + `>`→
  `==`/`<`/`>=`), scoped_entries 3 (2 unviable Default-tuple), guard 2,
  display/chord others; tabs key_context = 2 (1 unviable). Each viable
  mutant maps to a specific killing test (traced in-transcript).
- **RUN:** `cargo nextest run -p marley_editor -p marley` → **437/437
  passed, 2 skipped** (pre-existing #[ignore] headed lane).
  `cargo test --workspace --doc` → 0 failed.
- **Driven live captures (REQ-007 + REQ-001/002/004, PNGs read + asserted;
  scratchpad/265-*.png):**
  1. Session restored on the EDITOR tab (.mcp.json): ⌘D → layout identical,
     NO tab created (old behavior would have made a terminal); caret at 0 on
     `{` = non-word → correct None no-op.
  2. ⌘T then ⌘D on the terminal tab → the expanded rail shows the new rows
     "Marley 4" (⌘T) AND "Marley 5" (⌘D, active) — new-terminal fired
     (REQ-001).
  3. ⌘F on the terminal tab → the find bar box renders top-right (REQ-004a);
     esc closes.
  4. Editor tab, click into `http` (line 4 `"type": "http"`), ⌘D → `http`
     highlighted with the #255 accent tint (REQ-002 first press).
  5. ⌘D again → the highlight ADVANCED to line 5's `http` (inside the url),
     line 4 unhighlighted (REQ-002 repeat; a visibly different location).
  6. ⌘F on the editor tab → NO find bar (layout byte-similar to 5's pre-⌘F
     state; REQ-004b).
- **Gate:** first `--diff` run → RED on gate:14 ONLY (rustdoc: `action_for`'s
  public doc linked [`chords_unique_scoped`], a private item from the crate's
  public-docs view — the keymap module is private; coverage 100 + MSI 100 +
  everything else PASSED on that same run). Fix at source: de-linked to plain
  backticks with a "(crate-internal, not re-exported)" note. Re-run →
  **GATE GREEN [diff] — 15/15** (rustdoc + doc-todos + brand-scrub, coverage
  ≥100%, mutation MSI ≥100%, miri, visual/AX). Receipt written after the
  final .rs edit.
- **Pre-existing:** upstream `block v0.1.6` future-incompat note only.
- **Doc deltas landed during the gate wait** (inspect B2, early): SPEC-app-shell
  keymap API sketch + R19 (scoped guard) + NEW R19a (context model) + R20
  (off-stack None) + R51 (Terminal-scoped ⌘F); app_shell.md keymap.rs bullet
  gained the #265 paragraph.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG [Unreleased]/Added entry (top); editor.md gained
  the find.rs bullet; SPEC-app-shell (API sketch, R19, NEW R19a, R20, R51) +
  app_shell.md keymap bullet landed at validate (inspect B2 cleared early).
- **Knowledge (§19):** AAR 6f75fb5a submitted (completed, 5/5 — every phase's
  recall shaped the build). Captured: BF-claude-rustdoc-private-intra-doc-link
  (a public item's doc link must target the CRATE-PUBLIC surface, not
  module-local visibility) + AD-claude-keycontext-pure-model-001 (pure model
  over gpui-native dispatch; guard-forbids-ties ⇒ no order rule; native
  migration stays a mechanism swap).
- **Follow-up filed:** caret-follow scroll ticket (inspect A1 — wrap can land
  the selection off-screen; pre-existing, editor-wide) → forge (see create
  result; out of the /work range, shelf).
- **Lessons:** (1) the guard-makes-ties-impossible design choice moved a
  whole mutant family into "killable only via a synthetic invalid literal" —
  design decisions about invariants ARE test-plan decisions. (2) The driven
  session restored onto an EDITOR tab and the very first ⌘D press was the
  old-behavior counterexample (no tab created) — a restored-state capture can
  hand you the regression proof for free. (3) rustdoc private-intra-doc-links
  reds only surface at the gate's public-docs build — check links against the
  re-export surface when documenting pub-in-private-module items.
- **Ticket:** TICKET-265 → tickets/closed/, forge #265 → done.
- **Archive:** spec+notes → docs/planning/pipeline/completed/.
