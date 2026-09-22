---
pipeline_id: da3b98db-4377-4c74-9673-15b344fc0e4a
ticket: forge#350 (1c88f88a-41b4-4e67-8071-3d1647ad8f26) · duplicate forge#351 closed · local docs/planning/tickets/open/TICKET-350-structural-language-gate.md
aar_id: 34b5f8ea-5b31-4d24-a034-cbd598d0c256
status: Phase 5 — Complete PASS
title: Language-gate selection-ladder + sticky headers — the #340 M1 class, back-filled
type: bug
milestone: M22
references: [step_selection_ladder builds an ungated Rust session (app.rs:3408), refresh_sticky_headers likewise (app.rs:3484), the #315 comment marking BOTH as known pre-existing (app.rs:3405-3407 / :3481-3483), the gate template refresh_bracket_match (app.rs:3518-3535, the expression at :3522), the ONLY other production HighlightSession::new(Lang::Rust) is the syntax worker's initial session — correctly rebuilt per req.lang (app.rs:13106/:13120), all three fns are mutants::skip shims (drives, not cov, pin them), the #329/#330 test hooks (selection_ladder_pos_for_test :3453, refresh_sticky_headers_for_test :3755), PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001, BF-claude-bracket-match-no-language-gate-parses-non-rust-as-rust-001]
---

## Title
`step_selection_ladder` (⌃W/⌃⇧W expand-selection, #329) and `refresh_sticky_headers` (#330) build a
throwaway **Rust** `HighlightSession` unconditionally — neither checks the active file's language. On
a `.json`/`.py`/`.toml` file, ⌃W walks a Rust error-recovery tree's ancestry (plausible-but-wrong
rungs) and sticky headers can pin garbage if the Rust grammar hallucinates a `function_item`-shaped
node. This is exactly the #340 inspect M1 class — bracket-match got its caller-side gate at inspect;
these two shipped before the rule existed. #315's implementer SAW the gap and correctly scoped past it,
leaving a comment at both sites ("stay Rust-PARSED … NOT language-gated like `refresh_bracket_match`,
pre-existing behavior"). This ticket back-fills the gate, mirroring bracket-match verbatim.

## Scope
### In
- **The ladder gate** — `step_selection_ladder` (app.rs:3375) returns early when the active file is
  not Rust, BEFORE any parse: the same `language_of(&s.active_file().path) ==
  crate::code_syntax::Language::Rust` expression bracket-match uses (:3522). Both grow AND shrink go
  inert (⌃W/⌃⇧W are no-ops on a non-Rust file — no rungs, selection untouched). A stale ladder cached
  from a Rust file needs no explicit clear: its `(nonce, version)` validity check already makes it
  inert (document, don't add code).
- **The sticky gate** — `refresh_sticky_headers` (app.rs:3464) takes the DROP-CACHE shape its own
  no-editor branch already has (`return self.sticky_headers.take().is_some()`): a non-Rust active file
  drops any pinned spans with exactly one repaint, so switching a Rust tab → a JSON tab un-pins
  cleanly. Placed with the setting/no-editor checks, before any parse.
- **Verification is drive-shaped, not cov-shaped** — all three fns (grow/shrink wrappers at
  :3354/:3361, the engine, the sticky refresh) are `#[cfg_attr(test, mutants::skip)]` shims in
  coverage-excluded app.rs; the pinning is the #340 REQ-008 **two-arm proof**: identical content
  opened as `.rs` → feature ACTIVE; as `.json` → feature INERT. The existing hooks
  (`selection_ladder_pos_for_test`, `refresh_sticky_headers_for_test`) carry the asserts.
- Both gate comments replace the #315 "pre-existing behavior" paragraphs (the debt they documented is
  paid; say what the gate does and that #315-structural widening moves BOTH gates together).
### Out (explicitly)
- Widening the structural node APIs (fold/sticky/bracket/tags/expand-selection) beyond Rust — the
  per-grammar structure milestone, not this ticket; when it lands, these gates widen WITH bracket's.
- A word/line fallback ladder for non-Rust files (Zed-style plain-text expand) — a feature, not this
  bug. The cached-tree perf fix (#349). Any change to the pure `enclosing_ranges`/`all_headers`
  primitives (the caller gates; the primitive stays unconditional — the M1 prevention rule).

## Reference (§20)
N/A — a correctness back-fill of Marley's own #340 rule; the Rust-only-structural stance matches the
shipped fold (#305), symbols (#304), and bracket (#340) gates. No copyleft source involved.

### Prior art
1. **OUR OWN CODE (the whole ticket):** `refresh_bracket_match` :3518-3535 is the template — its gate
   expression, its drop-the-cache-on-non-Rust shape, and its doc comment naming the rule ("the pure fn
   ALWAYS parses as Rust, so this gate — not the pure fn — is what keeps it Rust-only"). #304's
   D-CALLER-GATES-LANGUAGE is the same decision, third time. The prevention rule + failure record
   (PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001 /
   BF-claude-bracket-match-no-language-gate-parses-non-rust-as-rust-001) are the provenance.
2. **Behavior maps / published** — none needed (no external behavior matched).
3. Checked tree-sitter's own error-recovery docs only to confirm WHY the misparse is dangerous
   (recovery always yields a tree — a non-Rust file never fails loudly, it fails plausibly).

## Locked-In Decisions
- **D1-MIRROR-BRACKET-VERBATIM** — the same expression, inline at both call sites; no new helper
  (three near-identical gates in shims are cheaper than an abstraction the structural-widening
  follow-up would immediately rewrite).
- **D2-LADDER-EARLY-RETURN** — gate at the fn top; grow and shrink both inert; the stale-ladder
  inertness rides the existing validity key (no new state).
- **D3-STICKY-DROP-CACHE** — the no-editor branch's take-and-repaint-once shape; a language switch
  un-pins in one frame.
- **D4-PRIMITIVES-UNTOUCHED** — `enclosing_ranges`/`all_headers` stay unconditional Rust parsers
  (the M1 rule: the CALLER gates).
- **D5-DUP-HYGIENE** — forge #351 is the same bug filed twice (empty tags, #304-era); closed as
  duplicate at authoring, this spec is the single record.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | keep ⌃W expand-selection ACTIVE on a `.rs` file (rungs grow through real Rust nodes) | existing #329 headless drives stay green (the two-arm proof's Rust arm) |
| REQ-002 | make ⌃W and ⌃⇧W no-ops on the SAME content opened as `.json` — selection unchanged, no ladder built | headless drive: `selection_ladder_pos_for_test()` stays None, selection byte-identical |
| REQ-003 | keep sticky headers pinning on a `.rs` file | existing #330 drives green (the Rust arm) |
| REQ-004 | pin NO sticky header on a non-Rust file, and DROP an existing pinned cache on switching to one (one repaint) | headless: `refresh_sticky_headers_for_test()` → true once (the drop), then false; cache None |
| REQ-005 | leave bracket-match, folding, and file-symbols behavior byte-identical | their existing gated tests green (no shared code touched) |
| REQ-006 | replace the two #315 "pre-existing behavior" comments with the gate's own doc | diff review |

## Phase Plan
P2 confirm the two sites + the gate expression against live code (this spec's line numbers go stale
the moment anything above them moves — re-grep by symbol); confirm no NEW ungated
`HighlightSession::new(Lang::Rust)` production site appeared since f044546 (the enumeration here found
exactly two + the worker's correct one). P3 two small gate insertions + comment rewrites; `cargo fmt`.
P3.5 critics on: the ladder's early-return interaction with `self.selection_ladder` staleness (prove
inert-by-validity, don't assume), the sticky drop's repaint-once contract, and whether any OTHER
caller reaches the engine (grep `step_selection_ladder(` — should be exactly grow/shrink). P4 the
two-arm drives (the #340 REQ-008 shape: same bytes, two extensions) + gate `--diff`. P5 docs
(editor.md's structural-APIs paragraph gains "ladder + sticky are gated like bracket") + AAR noting
the M1 class is now CLOSED across all structural consumers. Standing traps:
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md); **builds job-capped**.
