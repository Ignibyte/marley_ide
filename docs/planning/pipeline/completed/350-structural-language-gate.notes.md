# Language-gate selection-ladder + sticky headers — Notes

- **Forge ticket:** #350 `1c88f88a-41b4-4e67-8071-3d1647ad8f26` (dup #351 closed)
- **AAR:** `34b5f8ea-5b31-4d24-a034-cbd598d0c256`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-350-structural-language-gate.md
- **Pipeline spec:** 350-structural-language-gate.spec.md
- **pipeline_id:** `da3b98db-4377-4c74-9673-15b344fc0e4a`

## Phase 1 — Plan

- **Request:** promote the pre-authored spec, re-verify every cited seam by SYMBOL against live `main`
  @ `df00e6b`. THIRD of the goal `/work 307,321,350,345,348`. #307 (`8dd068c`) + #321 (`df00e6b`)
  shipped GATE GREEN.
- **Classification / tier:** work pipeline, one shippable slice. `type: bug` — a correctness back-fill
  of the #340 M1 rule (structural Rust parsing on non-Rust files), milestone M22 (the integrity-five
  batch).
- **Forge recall (§18.3):** zero active bulletins. The provenance the spec names —
  `PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001` and
  `BF-claude-bracket-match-no-language-gate-parses-non-rust-as-rust-001` — is #340's own inspect
  finding; this ticket applies that exact rule to the two seams that predated it.

### Findings (re-verified by symbol against `df00e6b`)

- **F1 — ENUMERATION HOLDS (measured).** Every `HighlightSession::new(marley_syntax::Lang::Rust)` in
  app.rs: **:3425** (`step_selection_ladder`), **:3501** (`refresh_sticky_headers`), and the #274
  off-thread worker at **:13180/:13185/:13194**. The worker is CORRECT — it rebuilds its session per
  `req.lang` (`if session_lang != req.lang { session = HighlightSession::new(req.lang); … }`,
  :3193-ish under the worker), so it is out of scope. Exactly the two ungated production sites the
  spec names, no drift. (The spec's frontmatter line numbers are stale — :3408/:3484 — but the
  SYMBOLS resolve; recorded so implement greps by symbol.)
- **F2 — the gate template is copyable verbatim (read app.rs:3535-3556).** `refresh_bracket_match`
  computes `is_rust = language_of(&s.active_file().path) == crate::code_syntax::Language::Rust`
  INSIDE its `active_editor().map(...)` block, then:
  ```rust
  if !is_rust {
      self.bracket_match_key = None;
      return self.bracket_match.take().is_some();  // drop the cache, repaint once iff it lit
  }
  ```
  — identical to its no-editor branch. Both target fns get the analogous shape.
- **F3 — D2 (ladder) holds, with a placement refinement.** `step_selection_ladder`'s FIRST
  `active_editor().map(...)` block (app.rs:3393-3401) already reads `(cs, ce, nonce, version)`. The
  language read joins THAT block and the fn early-returns on non-Rust BEFORE the `valid` check at
  :3404. Consequences, all verified by reading:
  - A ladder cached from a Rust file is **never consulted** on a non-Rust file — the early return
    precedes both the `valid` gate and any `selection_ladder`/`selection_ladder_at` read, so **no
    explicit clear is needed** (D2's claim). The stale ladder simply sits until you return to a Rust
    file, where the `(nonce, version)` validity check rebuilds it. That is the same lifetime a
    caret-move or edit already gives it.
  - Both grow AND shrink go inert on non-Rust — both `grow_selection`/`shrink_selection` route
    through `step_selection_ladder`, so one early return covers both chords.
- **F4 — D3 (sticky) confirmed.** `refresh_sticky_headers` (app.rs:3481) opens with the setting-off
  drop (`return self.sticky_headers.take().is_some()`, :3483) then the no-editor drop (:3489, same
  shape). The non-Rust case reuses that shape verbatim, placed **after the no-editor branch, before
  the same-`(nonce,version)` cache check** at :3492 — so switching from a Rust tab to a JSON tab
  un-pins in one repaint, and the cache key stays coherent.
- **F5 — the #315 "stay Rust-PARSED" comments MUST be rewritten (both fns).** app.rs:~3420 (ladder)
  and ~3498 (sticky) each carry a paragraph: "these structural seams … stay Rust-PARSED … NOT
  language-gated like `refresh_bracket_match`, pre-existing behavior." That debt is being PAID, so
  the comments must be replaced with the gate's own doc. Leaving them would make the code document
  the opposite of what it now does — the "a false doc is well-formed Rust" trap that cost #307 (three
  stale comments) and #321 (`send_definition_request`'s doc + `LspHost::drop`'s doc) real fixes.
- **F6 — the two-arm proof template exists (`bracket_match_off_for_non_rust_files_headless`,
  headless_drive.rs:6136).** Its idiom: write the SAME content to a `.rs` and a `.json`, drive the
  refresh, assert the feature is ACTIVE on `.rs` and INERT on `.json` — so the inert result IS the
  language gate, not merely "a non-Rust file has no valid Rust nodes." Copy that shape for the ladder
  and sticky drives. Hooks confirmed present: `selection_ladder_pos_for_test` (:5207),
  `refresh_sticky_headers_for_test` (:5342), the ⌃W/⌃⇧W chords for grow/shrink, the `data.json`
  fixture (:2318). The existing `sticky_headers_cached_and_pinned_headless` (:5335) and the
  ladder drives are the Rust-arm regressions.

### Prior-art sweep (§20, all three legs)

1. **Behavior maps / observed** — N/A. This is a correctness back-fill of Marley's own #340 rule; no
   reference-app behavior is being matched. The Rust-only-structural stance already ships for
   fold (#305), symbols (#304), and bracket-match (#340).
2. **Published material** — tree-sitter's error-recovery docs, checked only to confirm WHY the
   misparse is dangerous: recovery ALWAYS yields a tree, so a non-Rust file never fails loudly — it
   fails plausibly (garbage rungs, hallucinated `function_item`-shaped nodes). That is the whole
   reason a language gate, not a "did it parse?" check, is required.
3. **OUR OWN CODE (the whole ticket):** `refresh_bracket_match` is the template (F2), and the
   `D-CALLER-GATES-LANGUAGE` decision from #304/#340 is the rule. The pure primitives
   (`enclosing_ranges`, `all_headers`) parse Rust unconditionally and CANNOT self-gate — the caller
   must — which is exactly the M1 prevention rule. gpui/ropey/regex own nothing here.

### Decisions confirmed

All hold as written. `D1-MIRROR-BRACKET-VERBATIM` (F2 — the same expression inline at both sites, no
new helper; three near-identical gates in shims are cheaper than an abstraction the #315 structural-
widening follow-up would immediately rewrite), `D2-LADDER-EARLY-RETURN` (F3), `D3-STICKY-DROP-CACHE`
(F4), `D4-PRIMITIVES-UNTOUCHED` (the pure fns stay unconditional Rust parsers — the M1 rule),
`D5-DUP-HYGIENE` (#351 closed at authoring). The EARS AC (REQ-001…006) hold; no reopens.

### Discovery — the edit surface handed to Design

- `crates/marley_app/src/app.rs` — two gate insertions (`step_selection_ladder` ~:3393,
  `refresh_sticky_headers` ~:3489) + the two `#315` comment rewrites (~:3420, ~:3498). No other file.
- `crates/marley_app/src/headless_drive.rs` — two two-arm drives (ladder, sticky), copying the
  bracket-match template; the existing Rust-arm drives stay green.
- **Mutation/coverage:** all three touched fns are `#[cfg_attr(test, mutants::skip)]` shims in
  coverage-excluded app.rs, so the pinning is the drives (the #340 shape), NOT cov/MSI. `--diff`
  mutation will likely read "no mutable lines" like #321. Design confirms the skip stays bound.

**Status: Phase 1 — Plan PASS.**

## Phase 2 — Design

### Architecture / approach

Two gate insertions in `crates/marley_app/src/app.rs`, each mirroring `refresh_bracket_match`'s
caller-side language gate verbatim, plus the two stale-comment rewrites. No pure crate changes —
`marley_syntax`'s `enclosing_ranges`/`all_headers` stay unconditional Rust parsers (the M1 rule: the
CALLER gates, because a language-specific pure primitive cannot self-gate). §14 holds trivially: no
new error path, no IO, no panic surface — the gate is a `==` comparison and an early return over
existing cache fields.

**Reference (§20) confirmed N/A.** A correctness back-fill of Marley's own #340 M1 rule; no
reference-app behavior is matched. The prior art is our own `refresh_bracket_match` and the
`D-CALLER-GATES-LANGUAGE` decision from #304/#340. No copyleft source read.

### Gate 1 — `step_selection_ladder` (app.rs ~:3392)

The FIRST `active_editor().map(...)` block (:3393-3401) reads `(cs, ce, nonce, version)`. Fold the
language read into that same tuple and early-return on non-Rust, BEFORE the `valid` computation:

```rust
let Some((is_rust, cs, ce, nonce, version)) = self.active_editor().map(|s| {
    let primary = s.active_selections().primary();
    (
        language_of(&s.active_file().path) == crate::code_syntax::Language::Rust,
        primary.start().as_usize(),
        primary.end().as_usize(),
        s.active_nonce(),
        s.active_buffer().version(),
    )
}) else {
    return;
};
if !is_rust {
    return; // non-Rust → no ladder (the #340 M1 gate; the pure `enclosing_ranges` always parses Rust)
}
```

- **`is_rust` uses the exact bracket-match expression** —
  `language_of(&s.active_file().path) == crate::code_syntax::Language::Rust`.
- **No explicit ladder clear needed (D2, verified F3).** The `if !is_rust { return; }` sits BEFORE the
  `valid` check (:3404) and BEFORE any `self.selection_ladder`/`self.selection_ladder_at` read, so a
  ladder cached from a prior Rust file is never CONSULTED on a non-Rust file — it just sits until you
  return to a Rust file, where the `(nonce, version)` validity check rebuilds it (the same lifetime a
  caret-move already gives it). Leaving it is correct, not a leak: `SelectionLadder` is a small
  `Vec<Range>`, and dropping it here would cost a redundant repaint with no user-visible difference.
- **Both grow and shrink covered.** `grow_selection` (:3372) and `shrink_selection` (:3379) are the
  only callers and both route through `step_selection_ladder`, so one early return makes ⌃W AND ⌃⇧W
  inert on a non-Rust file.

### Gate 2 — `refresh_sticky_headers` (app.rs ~:3481)

Fold the language into the EXISTING nonce/version read (:3485-3487) — one `map`, not a second editor
read — and drop the cache on non-Rust right after the no-editor branch, before the same-version check:

```rust
let Some((is_rust, nonce, version)) = self.active_editor().map(|s| {
    (
        language_of(&s.active_file().path) == crate::code_syntax::Language::Rust,
        s.active_nonce(),
        s.active_buffer().version(),
    )
}) else {
    return self.sticky_headers.take().is_some(); // no editor → drop the cache
};
if !is_rust {
    return self.sticky_headers.take().is_some(); // non-Rust → drop the cache (the #340 M1 gate)
}
```

- **Reuses `return self.sticky_headers.take().is_some();` verbatim (D3)** — the no-editor branch's
  drop-and-repaint-once shape. So switching a Rust tab → a JSON tab un-pins in ONE repaint, and the
  cache key stays coherent (a subsequent Rust tab recomputes because the cache is `None`).
- **Placement before the same-`(nonce,version)` check** is required: otherwise a JSON file whose
  `(nonce, version)` happened to match a stale Rust cache would keep the stale headers pinned.

### The two comment rewrites (F5 — mandatory, the "false doc" trap)

Both fns carry a `#315` paragraph asserting the OPPOSITE of what the code will now do. Replace both
with the gate's own doc. Ladder (~:3418, above the removed `HighlightSession::new`):

> Non-Rust files get NO ladder: `enclosing_ranges`'s `HighlightSession` always parses as Rust (the
> pure primitive can't self-gate — the #340 M1 rule), so the CALLER gates above. When #315's language
> axis reaches the structural APIs, this gate widens to is-tree-supported alongside bracket-match's.

Sticky (~:3496, above its removed `HighlightSession::new`):

> Reached only for a Rust file (gated above, mirroring `refresh_bracket_match`): `all_headers`' session
> always parses as Rust, so a non-Rust file would pin misparsed "headers" without the caller-side gate.

### Mutation / coverage homes (verified)

- **All four fns carry `#[cfg_attr(test, mutants::skip)]`** — verified by reading: `step_selection_ladder`
  (:3391), `refresh_sticky_headers` (:3480), `grow_selection` (:3370), `shrink_selection` (:3377).
  app.rs is coverage-excluded. So the gates carry **no live mutants and no coverage obligation**; the
  headless drives are the only proof.
- **The gate insertion does not add a new fn**, so the `mutants::skip` detach trap (a new fn rebinding
  an adjacent attribute) is not in play. Phase 4 still re-runs
  `cargo mutants --list -f crates/marley_app/src/app.rs | grep -c 'step_selection_ladder\|refresh_sticky'`
  → must stay 0.
- Consequence: `--diff` mutation will read "no mutable lines in the diff — pass" (like #321). Correct;
  the drives carry it.

### File manifest

| File | Change |
|---|---|
| `crates/marley_app/src/app.rs` | Gate 1 in `step_selection_ladder` (fold `is_rust` into the first tuple + early return); Gate 2 in `refresh_sticky_headers` (fold `is_rust` into the nonce/version map + drop-cache return); rewrite the two `#315` comments. |
| `crates/marley_app/src/headless_drive.rs` | Two two-arm drives (ladder, sticky) copying `bracket_match_off_for_non_rust_files_headless`; one switch-drop drive. The existing Rust-arm drives stay unchanged. |

No other files. `enclosing_ranges`, `all_headers`, `language_of`, `Language`, the cache fields — all
reused unchanged.

### Regression Test Plan

The template is `bracket_match_off_for_non_rust_files_headless` (headless_drive.rs:6136): it drives
the refresh via the `_for_test` hook with `run_until_parked()` (NO `tick_pump` — these are direct
calls, not pump-timer-driven, so #321's mock-clock trap does not apply here). The fixture writes the
SAME content to a `.rs` and a `.json` so the `.json` inertness proves the GATE, not "no valid Rust
nodes" (F6).

| REQ | The behavior | Test | Where |
|---|---|---|---|
| REQ-001 | ⌃W keeps expand-selection ACTIVE on a `.rs` file (rungs climb) | existing `expand_shrink_round_trip_headless` etc. stay green (the Rust arm / positive control) | headless (existing) |
| REQ-002 | ⌃W and ⌃⇧W are NO-OPS on the SAME content opened as `.json` | `selection_ladder_off_for_non_rust_files_headless`: caret in `fn main(){ let x=1; }` written to `code.rs` → ⌃W grows (`selection_ladder_pos_for_test` advances, selection changes); same to `code.json` → ⌃W leaves `selection_ladder_pos_for_test()` None and the selection byte-identical; ⌃⇧W likewise inert | headless (NEW) |
| REQ-003 | sticky headers keep pinning on a `.rs` file | existing `sticky_headers_cached_and_pinned_headless` (:5335) stays green | headless (existing) |
| REQ-004 | NO sticky header on a non-Rust file | `sticky_headers_off_for_non_rust_files_headless`: same content `.rs` → `refresh_sticky_headers_for_test()` true, cache Some; `.json` → false, cache None | headless (NEW) |
| REQ-004b | a pinned Rust cache DROPS on switching to a non-Rust file (one repaint) | same drive, second phase: pin on the `.rs`, open the `.json`, assert `refresh_sticky_headers_for_test()` returns true ONCE (the drop) then false, cache None | headless (NEW, the switch-drop) |
| REQ-005 | bracket-match, folding, file-symbols byte-identical | their existing gated tests stay green — this ticket touches no shared code (diff review + suite) | existing suite |
| REQ-006 | the two `#315` comments replaced with the gate's doc | diff review | — |

**Fixture (the F6 point, load-bearing):** the content must be a valid-ish Rust snippet whose Rust
parse yields real structural nodes — `fn main() { let x = 1; }`. Written to `code.rs` it climbs/pins;
written to `code.json` the gate suppresses it. If the fixture were non-Rust-shaped (`{"a":1}`) the
`.json` arm could pass because there are no Rust nodes to find, proving nothing — exactly the trap the
bracket-match drive's comment warns about.

**LIVE drives OFF-LIMITS** (chad at the machine) — every proof here is headless via the `_for_test`
hooks; no live drive is wanted or needed. Stated so validate does not attempt one.

### Risks / decisions

- **R1 — the fixture must yield real Rust nodes (F6).** Enforced above; `fn main() { let x = 1; }` is
  the chosen content, and the `.rs` arm asserting the feature is ACTIVE is what proves the `.json`
  arm's inertness is the gate.
- **R2 — the ladder's stale-cache interaction (D2).** Proven inert-by-non-consultation, not asserted:
  the early return precedes every `selection_ladder`/`selection_ladder_at` access. A drive that grows
  on a `.rs` (building a ladder), then switches to a `.json` and grows again, asserting the pos stays
  None and the selection is unchanged, pins that the stale ladder is not consulted.
- **R3 — no other caller (verified).** `step_selection_ladder` ← only `grow_selection`/`shrink_selection`
  (:3372/:3379); `refresh_sticky_headers` ← only the pump (:1510) + the test hook (:3773). One gate per
  fn covers every path.
- **R4 — mutation reads "no mutable lines".** Expected (skipped shims); the drives are the proof, and
  Phase 4 confirms the skip stayed bound.

**Status: Phase 2 — Design PASS.**

## Phase 3 — Implement

One group, `crates/marley_app/src/app.rs` only. `cargo check -p marley --all-targets`: **0 errors, 0
warnings**, `cargo fmt --all` applied. `language_of` + `crate::code_syntax::Language` were already in
scope (bracket-match uses them in the same file), so no import change.

- **Gate 1 (`step_selection_ladder`):** `is_rust` folded into the first `active_editor().map(...)`
  tuple (`(is_rust, cs, ce, nonce, version)`), then `if !is_rust { return; }` immediately after the
  `else { return; }` — before the `valid` computation and any `selection_ladder` read, per D2. The
  early-return carries a comment stating the non-consultation invariant (a stale ladder idles, never
  consulted) and that both grow/shrink route through here.
- **Gate 2 (`refresh_sticky_headers`):** `is_rust` folded into the existing nonce/version `map`
  (`(is_rust, nonce, version)`), then `if !is_rust { return self.sticky_headers.take().is_some(); }`
  after the no-editor branch and BEFORE the same-`(nonce,version)` check — the placement comment
  states why (a matching key on a non-Rust file would otherwise keep stale headers pinned) and that
  the drop shape is the no-editor branch's, so a Rust→non-Rust switch un-pins in one repaint.
- **Both `#315` "stay Rust-PARSED … pre-existing behavior" comments DELETED and replaced** with the
  gate's-own-doc wording (the pure primitive always parses Rust and can't self-gate → the caller
  gates → the #340 M1 rule; #315-structural-widening moves both gates together). The now-false docs
  are gone — the F5 requirement, the class that cost fixes on #307 and #321.

### Deviations from design

None. The gate insertions add no new fn, so the existing `#[cfg_attr(test, mutants::skip)]` on both
fns is untouched (no detach-trap risk); Phase 4 re-runs `--list` to confirm.

**Status: Phase 3 — Implement PASS.**

## Phase 3.5 — Inspect

One critic was dispatched on the placement/comment lens (proportionate to a small, mechanical
change). **Process note (§15): it ran ~14 minutes without returning** — the same subagent latency the
whole batch has shown — so I closed the phase on my own six-lens review against live code, each item
verified by a read, not by assertion. If the critic's report lands before validate closes, its
findings will be folded in.

### Findings (all verified by reading the code; the six lenses a–f from the dispatch)

| # | Lens | Result | Evidence |
|---|---|---|---|
| a | Gate 1 placement | **CORRECT** | The `if !is_rust { return; }` (app.rs:3405) sits BEFORE `let valid = ...` (:3413) and before every `self.selection_ladder`/`self.selection_ladder_at` read. So D2's "a stale ladder is never consulted on a non-Rust file" holds by construction. `is_rust` is `language_of(&s.active_file().path) == crate::code_syntax::Language::Rust` — byte-identical to `refresh_bracket_match`'s expression, `==` (not `!=`), the right `Language::Rust` variant. Both `grow_selection` (:3372) and `shrink_selection` (:3379) call `step_selection_ladder`, so one return covers ⌃W and ⌃⇧W. |
| b | Gate 2 placement | **CORRECT** | The `if !is_rust { return self.sticky_headers.take().is_some(); }` (app.rs:3503) sits AFTER the no-editor branch (:3501) and BEFORE the same-`(nonce,version)` check (:3510). Placement is load-bearing: after the check, a non-Rust file whose `(nonce,version)` matched a stale Rust cache would take the same-version early return and keep the misparsed headers PINNED. The drop expression matches the no-editor branch verbatim. |
| c | The `.take()` asymmetry | **CORRECT and REQUIRED, not a leak** | The render reads `self.sticky_headers.as_ref()` UNCONDITIONALLY (app.rs:5611 in the sticky-header render — it gates only on the `sticky_header` SETTING, not language). So a stale non-Rust cache WOULD render pinned garbage → the `.take()` on the non-Rust gate is required. The ladder has NO per-frame reader: `self.selection_ladder` is touched only inside `step_selection_ladder`, which now early-returns on non-Rust → no clear needed. The asymmetry (sticky drops, ladder idles) is exactly right. |
| d | The comments (F5) | **CLEAN** | Both `#315 … stay Rust-PARSED … pre-existing behavior` paragraphs are GONE from both fns and replaced with text describing the gated behavior. A full-body grep of both fns for `rust-parsed`/`not language-gated`/`pre-existing behavior`/`degraded` returns nothing — no surviving false comment (the #307/#321 class). |
| e | `language_of` default | **SAFE** | `language_of` (code_syntax.rs:62-80) matches known extensions and defaults `_ => Language::Plain`. A file with no extension, a `.txt`, and the plain/scratch buffer all resolve to `Plain` (non-Rust), so the gate correctly makes ⌃W and sticky headers inert on them — matching bracket-match's Rust-only stance. No surprising Rust default that would defeat the gate. |
| f | Brand + scope | **CLEAN** | No `Zed`/`Warp` in any added line; the source diff is app.rs-only (the spec file is a git-mv). app.rs is coverage-excluded and both fns are `mutants::skip`'d, so no cov/MSI obligation — the drives are Phase 4's proof. |

**No findings.** The change is a faithful copy of the shipped `refresh_bracket_match` gate at both
sites, with the correct cache-lifetime asymmetry and both false comments retired. Nothing to fix.

**Status: Phase 3.5 — Inspect PASS.**

## Phase 4 — Validate

### Tests added (both in headless_drive.rs, copying `bracket_match_off_for_non_rust_files_headless`)

- `selection_ladder_off_for_non_rust_files_headless` (REQ-002) — via a shared `ladder_after_ctrl_w(cx,
  name)` helper: `.rs` arm ⌃W builds a ladder (`selection_ladder_pos_for_test().is_some()`) and grows
  the selection (the anti-vacuity control); `.json` arm with the SAME content leaves the pos `None` and
  the selection a bare caret; ⌃⇧W also inert.
- `sticky_headers_off_for_non_rust_files_headless` (REQ-004 + REQ-004b) — `.rs` pins spans `Some([(0,3)])`;
  switching to the same content as `.json` DROPS the cache (`refresh_sticky_headers_for_test()` returns
  true once — the repaint-worthy drop — then false; `sticky_header_spans_for_test()` None).
- Positive controls unchanged: `expand_shrink_round_trip_headless`, `sticky_headers_cached_and_pinned_headless`.

**A test bug the run caught, worth recording (F7 — the `.take()` asymmetry biting the TEST).** The
first ladder drive used ONE window: grow on `.rs` (pos `Some(1)`), switch to `.json`, ⌃W, assert pos
`None`. It FAILED with pos `Some(1)` — but that was the TEST wrong, not the gate. `selection_ladder`/
`selection_ladder_at` are RootView-level fields, and #350's D2 deliberately does NOT clear the ladder
on a non-Rust file (it idles, never consulted). So the `.rs` ladder's pos survived the switch; the gate
correctly did not REBUILD it (the selection stayed a bare caret — that assert passed). The fix: give
each arm an INDEPENDENT boot (`open_editor_file`), so the `.json` window's ladder is genuinely empty
and its `None` means "⌃W built nothing." This is exactly the asymmetry inspect (c) verified: sticky's
cache is `.take()`-dropped (so its one-window switch-drop works and the cache goes None), the ladder
idles (so it must not be cross-contaminated across arms). Recorded as a lesson, not a code change —
the code is correct; the test had to respect the design's own cache-lifetime asymmetry.

### Results (ACTUAL)

```
cargo test -p marley --lib -- (the two new drives)
test headless_drive::sticky_headers_off_for_non_rust_files_headless ... ok
test headless_drive::selection_ladder_off_for_non_rust_files_headless ... ok
test result: ok. 2 passed; 0 failed

cargo fmt --all --check                         → clean
cargo mutants --list -f app.rs | grep -c 'step_selection_ladder|refresh_sticky'  → 0 (skip intact)
```

**NEGATIVE SMOKE — the batch's standing check.** Neutered BOTH gates (`if !is_rust` → `if false`) and
re-ran the two drives:

```
test headless_drive::sticky_headers_off_for_non_rust_files_headless ... FAILED
test headless_drive::selection_ladder_off_for_non_rust_files_headless ... FAILED
test result: FAILED. 0 passed; 2 failed
```

Both fail without the gates — the `.json` arm behaves like the `.rs` arm — so the drives genuinely
detect the gate's presence. app.rs was restored immediately and verified (the two gates back to
`if !is_rust`, zero `NEG-SMOKE` markers, the diff back to the gate-insertion form). The five
`if !is_rust` sites in app.rs are all legitimate Rust-only gates (the two new ones + bracket-match +
file-symbols ×2), none neutered.

**GATE — `scripts/gates.sh --diff` → GATE GREEN [diff] (15/15).** All static gates green; gate:4
coverage ≥100% lines PASS; gate:5 mutation "no mutable lines in the diff — pass" (both gated fns are
`#[cfg_attr(test, mutants::skip)]` in coverage-excluded app.rs, so the diff carries no mutable lines —
the two-arm drives are the proof, by design); gate:6 miri PASS; gate:15 visual/AX 15 passed PASS.
Receipt `4fa975ec9d9b312f248b9a6c37cc8d6080d37573` written and verified worktree-bound against
`gate_state_hash`. (The gate stalled several times on the intermittent syspolicyd OS fault — a silent
0%-CPU nextest/doctest hang, confirmed via a fresh-binary probe each time; it self-cleared and no gate
was ever weakened.)

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**
