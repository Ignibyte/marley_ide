# Auto-close ' at lifetime-bound positions — Notes

- **Forge ticket:** #362 `bc01c12e-931f-4e92-a9da-44a82d603ee6`
- **AAR:** `865d79f1-e854-4f82-bd8e-6781ec822db5`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-362-lifetime-bound-auto-close.md
- **Pipeline spec:** 362-lifetime-bound-auto-close.spec.md
- **pipeline_id:** b39d555a-49a4-4095-b626-bdf7594b3ddc
- **Live on:** `3eb516d` (THIRD of the goal /work 360,361,362,363,364)

## Phase 1 — Plan
- **Request:** the deferred half of #346 — suppress `'`-pairing at lifetime-bound positions (`fn f<T: 'a>`,
  `fn f<'a>`, `where T: 'a`) so it doesn't over-pair into `''` like a char literal. A feature.
- **Classification / tier:** work pipeline; editor/auto-close/syntax; a pure marley_syntax ancestry probe + the
  #346 Context extension + an app shim.
- **Forge recall (§18.3):** bulletins none. `aar-open` → `865d79f1`. `knowledge-context` (Plan) logged 13
  surfacings — the #346/#340 tree ADs + the #299 no-second-language-source + tree-node-kind PRs. The governing
  prior art is #346's Context seam + `blocked_quote`'s own doc naming this cut.
- **★★ THE SPIKE (run in Plan, to author against reality — the #346 lesson) — DECISIVE:** a throwaway test parsed
  the REAL pre-insert fixtures and printed the `.parent()` ancestry at the caret byte (then REMOVED cleanly; lib.rs
  restored, tree clean). Results:
  - `fn f<T: '|>()` (caret after `T: `) → `> < type_parameters < function_item` — ★ **type_parameters ancestry
    resolves** even with the incomplete/empty bound (the `<>` is a structural anchor).
  - `fn f<T: Send + '|>()` → `type_parameters`. `fn f<'|>()` (fresh) → `type_identifier!MISS < type_parameter <
    type_parameters`. `struct S<T: '|>` → `type_parameters`. ✓ all the `<...>` declaration sites.
  - `fn f<T>() where T: '|` → `function_item < source_file` — ✗ **NO where_clause ancestry** (no `<>` anchor; the
    incomplete where-clause doesn't parse). Same for `where T: Send + '|`.
  - `fn foo(x: Box<dyn Trait + '|>)` → `> < type_arguments < generic_type` — a type-USAGE `<>` (broader).
  - `let c = '|;` → `; < let_declaration` — ✗ NO type_parameters (the char-literal position → no false positive).
  → **v1 scope = `type_parameters` ancestry** (clean, no char-literal false positives, covers `fn f<'a>` /
  `fn f<T: 'a>` — the primary cases). `where`/`dyn` deferred (not pre-insert-resolvable / broader). This
  DISSOLVED the naive "check for a lifetime node" (the #346 trap) and set an honest, safe scope.
- **Decisions:** D1 the signal is a `type_parameters` ancestor; D2 a new `Context::LifetimeBound` suppressing only
  `'` (fold into blocked_quote(context); non-`'` openers still pair); D3 StringOrComment precedence; D4
  where/dyn deferred; D5 pure fns cov/MSI 100, app shim mutants::skip.
- **Prior art:** #346 Context seam + `blocked_quote` doc (names the cut); tree-sitter-rust grammar (the SPIKE,
  adoption outside §20); `node_kind_at`/`enclosing_ranges` the probe templates. §20 N/A.
- **EARS:** REQ-BOUND-SUPPRESSES, REQ-CHAR-LITERAL-PAIRS, REQ-NON-QUOTE-OPENER-PAIRS, REQ-346-UNCHANGED (see spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** Three layers, matching #346's split: a new PURE `marley_syntax` ancestry probe
(the tree read); a PURE `auto_close` decision (`Context::LifetimeBound` + `blocked_quote(context)`, `pair_action`
stays a fn of small values — the #299 no-second-language-source rule); the app closure that builds the Context
(the shim). §14: no panic (the probe is combinator-style, `unwrap_or(false)`; the walk is bounded). §20 N/A
(Marley's own auto-close + tree-sitter-rust ancestry; the SPIKE was adoption).

**★ Recon settled (confirmed against the code):**
- `blocked_quote` (auto_close.rs:202) has ONE caller (:167, inside `pair_action`); the #338 units call
  `pair_action(...)` (not `blocked_quote` directly), so the signature change ripples to exactly the :167 call.
- The app Context-build is a CLOSURE `|off| -> Context` (app.rs:13685) that computes the byte and returns
  `context_from_node_kind(node_kind_at(...))`. Position-based — it does NOT need `typed` (pair_action does the
  per-char logic), so `LifetimeBound` is returned whenever the caret is in `type_parameters` and pair_action
  suppresses only `'`.
- `auto_close_pairs_through_the_live_editor_headless` (headless_drive.rs:6232) is the drive template
  (`open_rs_fixture` + `replace_text_in_range` + assert buffer text). ★ A #362 drive is VIABLE + valuable: the
  tree parses the BUFFER TEXT (present headlessly — unlike #361's font metrics that need CoreText), so a
  `'`-at-`fn f<T: |>` drive proves the shim end-to-end AND likely KILLS the shim's mutants (a broken closure →
  the drive fails observably).

**File manifest (4 files):**
- `crates/syntax/src/lib.rs` — ADD `pub fn in_type_parameters(src: &str, lang: Lang, byte_pos: usize) -> bool`
  (parse → `descendant_for_byte_range(pos,pos)` → walk `.parent()` for a `type_parameters` kind; combinator style
  no `?` — mirror `node_kind_at`'s dead-by-design None handling) + its units.
- `crates/editor/src/auto_close.rs` — `Context` gains `LifetimeBound`; `blocked_quote(typed, prev, context)` gains
  the `|| (typed == '\'' && context == LifetimeBound)` clause; the :167 call passes `context`; the doc (:189-201,
  currently "deferred") is un-deferred; units.
- `crates/marley_app/src/app.rs` — the Context closure (:13685) gains the LifetimeBound branch (StringOrComment
  precedence). Shim (cov-excluded); mutants::skip ONLY if a mutant survives the drive (#361 lesson) + the
  detach-trap check.
- `crates/marley_app/src/headless_drive.rs` — the #362 drive (`'` at a type-param position → bare `'`; control).

**★ The probe (settled — no-`?` combinator style for cov 100):**
```
pub fn in_type_parameters(src: &str, lang: Lang, byte_pos: usize) -> bool {
    let mut parser = parse::parser_for(lang);
    parser
        .parse(src, None)
        .map(|tree| {
            let mut node = tree.root_node().descendant_for_byte_range(byte_pos, byte_pos);
            let mut found = false;
            while let Some(n) = node {
                if n.kind() == "type_parameters" { found = true; break; }
                node = n.parent();
            }
            found
        })
        .unwrap_or(false)
}
```
No early `return` (a `found`/`break` bool — clippy-clean); `.map(...).unwrap_or(false)` keeps the None-flow in a
combinator (an uncancelled parse never returns None → no uncoverable `?` region — the node_kind_at lesson).

**★ The auto_close threading (settled):**
- `Context::LifetimeBound` — a 3rd variant (doc: the caret is inside a `<...>` type-parameter list where `'` is a
  lifetime, #362).
- `blocked_quote(typed, prev, context)`: `typed == '\'' && (matches!(prev, Some(p) if p=='&'||p=='<'||is_word(p))
  || context == Context::LifetimeBound)`. The pairing arm (:165-168) becomes `... && !blocked_quote(typed, prev,
  context) && context != StringOrComment`. VERIFIED: LifetimeBound + `'` → blocked_quote true → no pair ✓;
  LifetimeBound + `(` → blocked_quote false (not `'`) && LifetimeBound != StringOrComment → PAIRS ✓; StringOrComment
  → the `!= StringOrComment` gate → no pair (unchanged) ✓; Code → the #338 behavior (unchanged) ✓.

**★ The app closure (settled — StringOrComment precedence):**
```
let ctx = context_from_node_kind(node_kind_at(&src, Rust, byte).as_deref());
if ctx == Context::StringOrComment { ctx }
else if marley_syntax::in_type_parameters(&src, Rust, byte) { Context::LifetimeBound }
else { ctx } // Code
```
(the `!probe_rust → Code` early return stays). A `'` inside a string is a string, not a bound → StringOrComment
first.

**★ Test plan.**
| # | Proves | Test |
|---|---|---|
| U1 | REQ-BOUND-SUPPRESSES (probe, marley_syntax) | `in_type_parameters` → TRUE for `fn f<T: §>`, `fn f<§>`, `fn f<T: Send + §>`, `struct S<T: §>` (§=the spike caret bytes). |
| U2 | REQ-CHAR-LITERAL-PAIRS + boundary (probe) | `in_type_parameters` → FALSE for `let c = §;`, `let x = 5;` (a code pos), `fn f<T>() where T: §{}` (the deferred `where` — documents the boundary), and a non-Rust/empty src. |
| U3 | REQ-BOUND-SUPPRESSES (decision, auto_close) | `pair_action('\'', Some(' '), None, false, Context::LifetimeBound)` → `Insert`. |
| U4 | REQ-CHAR-LITERAL-PAIRS (decision) | `pair_action('\'', Some(' '), None, false, Context::Code)` → `InsertPair`; the #338 `blocked_quote`-via-`pair_action` rows stay green (updated for the new arg). |
| U5 | REQ-NON-QUOTE-OPENER-PAIRS | `pair_action('(', Some(' '), None, false, Context::LifetimeBound)` → `InsertPair`. |
| T1 | the wiring end-to-end (headless drive) | open `fn f<T: >() {}`, caret after `T: `, type `'` → buffer is `fn f<T: '>() {}` (bare `'`, NOT `''`); a control at `let c = ` → `let c = ''` (paired). |

- **Uncoverable-honestly:** the app closure is cov-excluded (app.rs); the drive (T1) carries its wiring + likely
  kills its mutants. The PURE surfaces (`in_type_parameters` [marley_syntax] + `pair_action`/`blocked_quote`
  [editor]) are cov/MSI 100 via U1-U5.

**Risks / decisions.** (a) the probe combinator style → cov 100 (no `?`); (b) blocked_quote ripple = the one :167
call (units call pair_action); (c) LifetimeBound suppresses only `'` (verified pairing arm); (d) StringOrComment
precedence; (e) the shim's mutants — check `cargo mutants --list` at validate; the drive likely kills them,
mutants::skip only a survivor (+ detach-trap check); (f) v1 = type_parameters (where/dyn deferred; U2 documents
the boundary with a `where`-returns-false unit). §14; §20 N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built (4 files, to the manifest):**
- `syntax/lib.rs` — `pub fn in_type_parameters(src, lang, byte_pos) -> bool` (after `node_kind_at`), the
  combinator `.map(|tree| { walk .parent() for "type_parameters" }).unwrap_or(false)` (no `?`, `found`/`break`
  bool — clippy-clean, cov-100 shape). Units `t362_in_type_parameters_true_at_bound_positions` (the 4 `<...>`
  fixtures, carets via `src.find(…)+n`) + `t362_in_type_parameters_false_off_the_bound` (char-literal, code, the
  deferred `where`, empty). Both pass.
- `editor/auto_close.rs` — `Context::LifetimeBound` (3rd variant); `blocked_quote(typed, prev, context)` gains
  `|| context == Context::LifetimeBound`; the :167 pairing arm passes `context`; the doc un-deferred. Unit
  `t362_lifetime_bound_suppresses_only_the_quote` (`'`→Insert, `'` in Code→InsertPair, `(`→InsertPair). The
  #338/#346 units (call `pair_action`, not `blocked_quote`) stay green — 34 pass.
- `app.rs` — the Context closure (:13684) restructured: node_kind Context first (StringOrComment wins), else
  `in_type_parameters` → LifetimeBound, else Code. NO `mutants::skip` yet (Phase 4 checks `--list`; the drive
  likely kills its mutants).
- `headless_drive.rs` — `auto_close_suppresses_quote_in_type_parameters_headless`: open `fn f<T: >() {}`,
  `set_editor_caret("fn f<T: ".len())`, type `'` → assert `fn f<T: '>() {}` (bare `'`). Passes.

**Deviations from design:**
- (1) `use gpui::EntityInputHandler;` needed inside the drive fn (the #338 drive has the same local import) for
  `replace_text_in_range`.
- (2) The drive's char-literal CONTROL was DROPPED: `open_rs_fixture` reuses the fixed path `sample.rs`, so a
  second `open_file_in_viewer` switches back to the already-open (edited) buffer instead of reloading — the
  control captured the bound buffer. The char-literal-still-pairs behavior is covered by the pure unit
  `pair_action(…, Context::Code) → InsertPair` + the #338 drive, so the headless control was redundant; removed
  it and documented why. The bound case (the NEW behavior) is the drive's proof.

**Checks:** `cargo fmt --all` clean; per-layer `cargo nextest` green (probe 2, auto_close 34, drive 1);
`cargo clippy -p marley_syntax -p marley_editor -p marley --all-targets -- -D warnings` **exit 0**. Diff = the 4
manifest files.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose, parallel) + my own review. Verdict: 1 HIGH + 1 MEDIUM + 2 LOW — the HIGH +
MEDIUM FIXED at source; the LOWs addressed with doc caveats.**

**Critic 2 — [HIGH] REAL, FIXED: the headless drive was VACUOUS.** Fixture `fn f<T: >() {}`, caret byte 8 = the
`>` position → `next = '>'`, and `opens_here('>')` = false (`>` is neither a PAIRS closer nor whitespace), so
`pair_action` returns `Insert` (bare `'`) REGARDLESS of context — the `>` blocked the pair, not #362's
LifetimeBound. Reverting the app-closure branch left the drive green → it tested nothing. My own inspection had
confirmed the enclosing-fn `mutants::skip` (correct) but did NOT trace `opens_here` on the fixture — Critic 2
caught it. **Fix:** the double-space fixture `fn f<T:  >() {}`, caret between the two spaces (byte 8) → `next=' '`
(whitespace → `opens_here` true) so a Code context WOULD pair `''`; only LifetimeBound suppresses → bare `'`.
Asserted `!bound.contains("''")` + the exact bare-`'` string; added the double-space byte to the probe's
true-unit. Verified non-vacuous by tracing the pre-fix Code→InsertPair→`''` path (the assert fails on it).
`BF-…-vacuous-headless-drive-opens-here-masks-the-fix-001` + `PR-…-suppression-test-must-prove-the-baseline-…`.

**Critic 1 — [MEDIUM] REAL, FIXED: the `pair_action` fn's own doc was stale.** The `blocked_quote`/`Context` docs
were un-deferred correctly, but the PRIMARY public fn's doc still said "It does NOT catch the bound positions
(`T: 'a`, `+ 'a`, `'static`)" — a direct contradiction of the shipped #362 behavior on the main API. **Fix:** added
a clause noting #362 catches the `<...>` positions via `Context::LifetimeBound`; `where`/`dyn`/standalone-`'static`
remain deferred.

**LOWs (addressed):** (Critic 1) `Context::LifetimeBound` doc claimed "`'` starts a lifetime" without the one
non-lifetime `'` (a const-generic char default `<const C: char = 'x'>`) → added a caveat noting it's vanishingly
rare + fails SAFE (pair withheld, buffer never mangled). (both) the double tree-sitter parse per opener keystroke
(`node_kind_at` + `in_type_parameters`) — a known micro-cost, gated to opener keystrokes only, and #363 (this
batch) caches the tree → no action, noted.

**Both critics CONFIRMED (no change):** the probe is correct (deepest-node + inclusive `.parent()` walk,
terminates, combinator no-`?` → cov 100); LifetimeBound gates ONLY `'` (the parens in `blocked_quote` put
`context == LifetimeBound` under `typed == '\''`); `blocked_quote`'s one caller ripple; `pair_action` stays pure
(#299); the pure units kill every viable mutant (the Code-`'`→InsertPair row is load-bearing for the `||`); ★ the
app closure is inside `replace_text_in_range` which is ALREADY `#[cfg_attr(test, mutants::skip)]` → the closure
enumerates ZERO mutants (the #361 gate-red CANNOT recur here; no new skip needed, no detach-trap since no fn was
inserted) — my own pre-check reached this independently.

**★ Process incident (recorded):** while fixing the drive, a `python3` scratch-edit to remove a verification
spike from lib.rs CORRUPTED the file (duplicated its contents). Recovered by `git show HEAD:…/lib.rs > …/lib.rs`
(a read-to-stdout + my own write of the ONE file I corrupted — NOT a `git checkout`/`restore` of the working
tree, which the #259 rule bans; the other 3 #362 files were untouched) then re-applied the two #362 edits via
Edit. Lesson: prefer the Edit tool for in-tree Rust edits; a `python3` string-splice on a large source file is a
corruption risk — and a spike belongs in a scratch bin, not spliced into a real crate's test module.

**Findings acted on:** the HIGH (drive) + MEDIUM (doc) fixed; 2 LOWs addressed with caveats. `failure-record`:
the vacuous drive (7d598d50). Re-verified after fixes: probe units 2 + drive 1 pass, clippy -D warnings exit 0.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests RUN (written in Phase 3 + the inspect fixes):**
- Targeted: `cargo nextest run -p marley_syntax -p marley_editor -p marley -E 'test(t362) +
  test(auto_close_suppresses_quote_in_type_parameters)'` → **all pass** (2 syntax units incl. the non-vacuous
  double-space byte + 1 editor unit + 1 non-vacuous drive).
- Regression: `cargo nextest run -p marley_syntax -p marley_editor -p marley` → **1064 passed, 2 skipped**. No
  ripple — the #338/#346 auto_close units + drives stay green (the `blocked_quote` signature change reaches only
  the one :167 caller, not the units, which call `pair_action`). The `search_open…` flake did not recur.

**Live drive:** NONE — stated deliberately. The headless drive IS the proof: it runs the REAL editor door
(`replace_text_in_range` → the real `mutants::skip`'d Context closure → `in_type_parameters` on the buffer text →
`pair_action`) and is NON-VACUOUS (the double-space fixture makes `opens_here` true so a Code context WOULD pair
`''`; only #362's LifetimeBound suppresses it). A live synthetic drive is off-limits (chad may be at the machine)
and adds nothing over the end-to-end headless assertion.

**Full `--diff` gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15 PASS** (first run), receipt
`9016d06423fc099e2035a22fa08b171d50afe940`. gate:4 coverage ≥100% (`in_type_parameters` [marley_syntax] +
`blocked_quote`/`pair_action` [marley_editor] fully exercised by the units; the app closure is app.rs
cov-excluded, carried by the drive); gate:5 mutation MSI ≥100% (the pure-fn mutants killed by the units; ★ the app
closure enumerated NO mutants — it's inside `replace_text_in_range` which is already `#[cfg_attr(test,
mutants::skip)]`, so the #361 gate-red could not recur — confirmed no survivor); gate:14 docs PASS (the
un-deferred `pair_action` doc + the intra-doc links resolve). No red, no pre-existing exclusions.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21).**
- **CHANGELOG.md** — a `### Fixed` entry: auto-close no longer over-pairs `'` at a `<...>` lifetime-bound position
  (the `in_type_parameters` ancestry probe + `Context::LifetimeBound`; where/dyn deferred).
- **docs/marley_architecture/editor.md** — extended the `auto_close.rs` #346 Context paragraph to add the #362
  `LifetimeBound` variant + the ancestry probe (was "the `'` bound cut is deferred"; now un-deferred for `<...>`).

**Capture (forge wired).**
- `aar-submit` 865d79f1, outcome completed, effectiveness 4 (2 novel, 13 verdicts). Headline lessons: (a) ★★ the
  Plan-phase SPIKE paid off decisively — a real tree-sitter parse on the PRE-insert fixtures (run in Plan, removed
  cleanly) settled that `type_parameters` resolves [the `<>` anchor] but `where` doesn't, setting an honest v1
  scope BEFORE any code (dissolving the #346-recorded "node_kind finds a lifetime" false assumption). (b) ★★ the
  VACUOUS-DRIVE catch (Critic 2 HIGH): the `>` after the caret made `opens_here` false → the pair was blocked by
  an unrelated guard, not by #362 → the drive tested nothing; fixed with a double-space fixture (next=whitespace)
  so only LifetimeBound suppresses. (c) ★ the inherited `mutants::skip` — the app closure is inside
  `replace_text_in_range` (already skipped) → zero mutants → the #361 gate-red couldn't recur. (d) the pure/app
  split held (cov/MSI 100 on the pure fns; the shim carried by the drive). (e) a PROCESS INCIDENT: a python3
  scratch-splice corrupted lib.rs → recovered via `git show HEAD:file > file` (not a tree checkout) + Edit.
- `prevention-rule-record`: **PR-claude-suppression-test-must-prove-the-baseline-would-not-suppress-001**
  (9f80e3c9, recorded at inspect) + **PR-claude-no-adhoc-python-splice-on-source-use-edit-tool-001** (2f0d345f) —
  use the Edit tool for in-tree Rust, keep spikes in a scratch bin, recover a corrupted tracked file via
  `git show` not `git checkout`.
- `failure-record`: BF-…-vacuous-headless-drive-opens-here-masks-the-fix (7d598d50, recorded at inspect). No new
  SHIPPED failure (the vacuous drive was caught + fixed in-phase).
- **Follow-up: forge #366** (`38c21f44`, editor/auto-close/syntax/362-followup) — the deferred `where T: 'a` +
  `dyn Trait + 'a` positions (need a non-ancestry signal for `where`; `type_arguments` + a const-generic-block
  guard for dyn).

**Close + archive.** forge #362 → done; TICKET-362 → closed/ (`status: closed`); the spec/notes pair archived
active/ → completed/; spec `status: Phase 5 — Complete PASS`.

**Status: Phase 5 — Complete PASS.** Run `/commit` to deliver (LOCAL — push un-OK'd). THIRD of the goal /work
360…364 (3 of 5). ⚠️ P5 was docs-only (CHANGELOG + editor.md, NO `.rs` edit) → the Phase-4 receipt `9016d064`
HOLDS.
