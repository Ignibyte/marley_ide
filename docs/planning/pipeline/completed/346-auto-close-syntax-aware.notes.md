# 346 — auto-close syntax-aware — notes

- pipeline_id 4010be08-ec1c-4eeb-a464-0d6a59db8171 · forge #346 3cda6024-b130-4792-9f12-eb9ccfcb2506
- aar_id 0300a225-e00a-4e80-94ce-03543bc9482f · on `8619eae`

## Phase 1 — Plan

**Intent:** #338's auto-close pairs even inside strings/comments, and the `'` lifetime-bound positions still
pair. Both needed the tree (#315, now shipped). Make pairing syntax-aware.

**Recon (live on `8619eae`) — confirmations + the DECISIVE correction:**
1. `auto_close::pair_action` (editor/auto_close.rs:125) — pure, 26k sweep; `blocked_quote` (:186) the `'` guard.
   Caller: `Buffer::insert_pairing_at_selections` (buffer.rs:341), per-cursor `plan` → pair_action (:372). Buffer
   is text-only.
2. **Crate boundary:** `marley_editor` ⊥ `marley_syntax` (no dep either way; app deps both) → D3 (A): Context +
   name-based classifier in the editor crate, a thin `node_kind_at` probe in syntax, app wires. `delims_at`
   (syntax:563) is the existing `descendant_for_byte_range().kind()` probe (its doc: a `(` in a string lands in
   `string_content`, spike-verified).
3. **★ THE TICKET'S CENTRAL CLAIM IS HALF-WRONG.** "the node kind at the caret answers [the `'`] directly — a
   `lifetime` vs a `char_literal`." The tree is parsed from the PRE-INSERT text; while typing `'` at `T: |` or
   `let c = |`, there is NO lifetime/char node yet — BOTH read as Code. So node_kind_at solves the STRING/COMMENT
   case (the caret is inside an EXISTING string/comment) but NOT the lifetime-bound case (the `'` being typed has
   no node). → SCOPE: ship string/comment (D2); the lifetime bound is a design SPIKE (ancestry detection or
   speculative parse — a distinct harder problem) → recommended DEFERRAL + a follow-up; #338's blocked_quote
   stays.
4. Rust node kinds confirmed (tree-sitter-rust node-types): `line_comment`, `block_comment`, `string_literal`,
   `raw_string_literal`, `string_content`, `char_literal`, `lifetime`. Name-based classifier viable (design
   verifies non-Rust names: Python/JS `comment`/`string`, etc.).
5. Perf (D4): a throwaway `HighlightSession` reparse per pair-char keystroke (like enclosing_ranges app.rs:3464),
   gated on auto_close_on — rare. #349 caches later.

**knowledge-context (Plan):** 13 nodes logged — incl. the #338 auto-close AD, the #315 syntax AD, the #299
lang_spec "second source of language facts" trap (D1 — NO regex "am I in a string"), PRs on the descendant
probe. All reinforce D1 (tree-only) + D5 (Code==old).

**Prior-art sweep:** (1) our own delims_at (the probe pattern) + pair_action (#338) + HighlightSession (#315).
(2) tree-sitter node-types (adoption). (3) ropey/regex — N/A (the #299 trap forbids a regex probe). §20 = N/A.

**EARS:** REQ-NO-PAIR-IN-STRING, REQ-NO-PAIR-IN-COMMENT, REQ-CODE-BYTE-IDENTICAL, REQ-PROBE-CLASSIFIES,
REQ-TYPEOVER-UNAFFECTED.

**Risks:** (a) ★ the lifetime-bound spike (P2) — CONFIRM the pre-insert obstacle by a spike (parse + probe at the
bound caret → no lifetime node), then defer with a follow-up (recommended) OR do ancestry detection (only if a
spike shows it's clean+cheap — unlikely). (b) the char→byte offset at the probe (the buffer is char-indexed; the
tree is byte-indexed — reuse the existing char↔byte map). (c) the 26k sweep MUST stay byte-identical with
Code threaded (D5). (d) crate boundary — verify non-Rust kind-names don't collide (D3).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**★ SPIKE OUTCOME — the lifetime bound is DEFERRED (Out); ship string/comment only.** The obstacle is logically
certain (no test needed): while typing `'` at `T: |` the char is NOT yet in the buffer, so the tree parsed from
the pre-insert text CANNOT contain a `lifetime` or `char_literal` node at the caret — `descendant_for_byte_range`
returns the enclosing bound/expr/ERROR node, not `lifetime`. So node_kind_at gives `Code` for BOTH a bound and a
char-literal position → the `'` disambiguation is unsolved by the leaf-kind probe. Ancestry detection
(is-the-caret-in-`where_clause`/`trait_bounds`/`type_parameters`) is per-language + error-prone + a distinct
problem. → **`Context = { Code, StringOrComment }`** (NO dead CharLiteral/Lifetime variants); `blocked_quote`
UNTOUCHED (the bound positions keep pairing, as #338 documents); a follow-up is filed at P5.

**D3 SETTLED = (A).** `marley_editor` ⊥ `marley_syntax` (verified). Homes: `Context` + the classifier in
`marley_editor::auto_close` (pure, cov/MSI 100, Lang-free); a `node_kind_at` probe in `marley_syntax`; app.rs
wires. NO editor→syntax cycle. Langs: Rust/Python/JS/TS/Tsx/Json/Bash. The classifier matches kind-name SETS:
comment {`line_comment`,`block_comment`,`comment`}, string {`string_literal`,`raw_string_literal`,
`string_content`,`string`,`template_string`,`template_literal`,`heredoc_body`,`raw_string`}, else Code. ★ A
false-negative (a real non-Rust string name we missed → Code → pairs) is the OLD behavior, NOT a regression —
safe-by-default; the parse-a-fixture tests verify per-language and any miss just under-suppresses. Rust (the
primary) is fully covered (node-types confirmed).

**The pieces (exact):**
- `marley_syntax`: `pub fn node_kind_at(src: &str, lang: Lang, byte_pos: usize) -> Option<String>` — a FREE
  parse-then-probe fn EXACTLY mirroring `delims_at`'s pub wrapper (lib.rs:521 "a FREE fn… parses src itself…
  #349 caches later"): parse `src` for `lang`, `root.descendant_for_byte_range(byte_pos, byte_pos)?.kind()
  .to_string()`. Total (None on parse-fail/degenerate). cov/MSI 100 via a parse-a-fixture test.
- `marley_editor::auto_close`: `#[derive(Clone,Copy,PartialEq,Eq,Debug)] pub enum Context { Code,
  StringOrComment }`; `pub fn context_from_node_kind(kind: Option<&str>) -> Context` (None/unknown → Code; a
  comment/string kind → StringOrComment); `pair_action(typed, prev, next, has_selection, context: Context)` —
  the ONLY behavioral change: the InsertPair arm gains `&& context != Context::StringOrComment`, i.e.
  `Some(close) if opens_here(next) && !blocked_quote(typed, prev) && context != Context::StringOrComment`. Every
  other arm (Wrap/TypeOver/Insert) UNCHANGED → Code == the old 26k sweep byte-identical (D5). (Wrap inside a
  string is unaffected — a selection-wrap is fine; the ticket's own note. TypeOver unaffected — it's before the
  InsertPair arm.)
- `marley_editor::buffer.rs`: `insert_pairing_at_selections(set, typed, origin, context_at: impl Fn(CharOffset)
  -> Context)` — the `plan` closure calls `context_at(sel.start())` and passes it to `pair_action`. RECOMMEND
  the closure (Buffer stays tree-agnostic; per-cursor).
- **app.rs (the shim, mutants::skip):** the insert path supplies `context_at = |off| { if !auto_close_on ||
  lang.is_none() { Context::Code } else { context_from_node_kind(node_kind_at(&src, lang, char_to_byte(off))
  .as_deref()) } }`. Reparses per cursor (rare; the delims_at precedent reparses too; gated on auto_close_on so
  ZERO cost when off; #349 caches later). `lang` from the #315 lang map (`language_id_for`/similar); a
  non-supported file → Code → old behavior. `char_to_byte` reuses the buffer's existing offset map.

**File manifest:**
| file | change |
|------|--------|
| `crates/syntax/src/lib.rs` | +`pub fn node_kind_at` (free parse+probe) + its test. cov/MSI 100. |
| `crates/editor/src/auto_close.rs` | +`Context` enum + `context_from_node_kind` + the `context` param on `pair_action` (the one InsertPair-arm guard); the 26k sweep + rows extended. cov/MSI 100. |
| `crates/editor/src/buffer.rs` | `insert_pairing_at_selections` gains `context_at`; the `plan` calls it. cov/MSI 100 (tests thread a closure). |
| `crates/marley_app/src/app.rs` | the insert-path caller supplies the `context_at` closure (probe + classify). `mutants::skip` shim. |

**Mutation/coverage:** `node_kind_at` (syntax), `context_from_node_kind`+`pair_action`(+ctx)+`insert_pairing_at_selections`(editor) → cov/MSI 100 in their pure libs. The app.rs closure → shim. gate:5 mutates the editor+syntax diff lines.

### Regression Test Plan (unit/headless — no live drive)
| # | REQ | test |
|---|-----|------|
| T1 | CODE-BYTE-IDENTICAL | ★ the #338 26k sweep is updated to thread `Context::Code` and assert the SAME recorded actions — proving Code==old (the load-bearing regression). The existing #338 rows + `blocked_quote` tests stay green (updated to pass `Context::Code`). |
| T2 | NO-PAIR-IN-STRING/COMMENT | `pair_action(opener, prev, next, false, Context::StringOrComment) == Insert` for every opener/prev/next that would ELSE `InsertPair` (a focused table). |
| T3 | TYPEOVER-UNAFFECTED | `pair_action('"', _, Some('"'), false, StringOrComment) == TypeOver` (context gates only InsertPair). |
| T4 | classifier | `context_from_node_kind` per kind-name (`"line_comment"`/`"string_literal"`/`"string_content"`/`"comment"`/`"string"`→StringOrComment; `"identifier"`/None/`"lifetime"`→Code). cov/MSI 100. |
| T5 | PROBE-CLASSIFIES | `node_kind_at` parse-a-fixture: Rust `let s = "a(b"; // c(` → probe the `(` inside the string → a string kind; the `(` in code → a code kind; + a Python fixture (`# comment`, `"str"`) for the non-Rust names. |
| T6 | buffer threading | `insert_pairing_at_selections` with `|_| Context::Code` == the old behavior (existing tests) + one row with `|_| StringOrComment` → the opener does NOT pair. |

Uncoverable: the LIVE type-in-a-string drive — DEFERRED (chad at machine; the pure sweep + parse units are the
exact proof). The lifetime bound — DEFERRED (spike; follow-up).

**Risks:** (a) the 26k sweep threading Code MUST stay byte-identical (T1 — the guard). (b) per-cursor reparse
perf (rare + gated; #349). (c) the classifier's non-Rust name coverage (safe-by-default — a miss = under-suppress
= old behavior; T5 verifies). (d) char→byte at the probe (reuse the buffer map). (e) no dep cycle (D3-A).

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest + one justified deviation (ime.rs — the real production caller):
1. **syntax/lib.rs** — `pub fn node_kind_at(src, lang, byte_pos) -> Option<String>` (free parse-then-probe via
   `parse::parser_for(lang)` — the #315 grammar map, NOT hardcoded Rust; `descendant_for_byte_range(pos,pos)
   .kind()`). Mirrors the `delims_at` free-fn precedent.
2. **editor/auto_close.rs** — `#[derive(Clone,Copy,PartialEq,Eq,Debug)] pub enum Context { Code,
   StringOrComment }` + `pub fn context_from_node_kind(Option<&str>) -> Context` (name-set match, Lang-free) +
   `pair_action` gains a last `context: Context` param; the ONLY behavioral change is the InsertPair arm gaining
   `&& context != Context::StringOrComment`. The 15 `#[cfg(test)]` pair_action calls updated to pass
   `Context::Code` (a scoped `sed` on `pair_action(` lines — verified none missed).
3. **editor/buffer.rs** — `insert_pairing_at_selections_ctx(..., context_at: impl Fn(CharOffset) -> Context)`
   carries the body (the `plan` closure calls `context_at(sel.start())`); the old
   `insert_pairing_at_selections(set, typed, origin)` is now a thin DELEGATOR (`|_| Context::Code`), so its ~11
   existing test callers are UNCHANGED.
4. **★ editor/ime.rs (deviation — the real production caller, found at implement):** `pair_action`'s only
   production caller is `ime::replace_text` (:126), NOT app.rs directly. Same delegator pattern:
   `replace_text_ctx(..., context_at)` carries the body + calls `insert_pairing_at_selections_ctx(context_at)`;
   `replace_text(...)` delegates (`|_| Context::Code`), so its test callers (ime.rs:274/281) are unchanged.
5. **marley_app/app.rs** — `replace_text_in_range` (:13527, `mutants::skip`) now supplies the closure to
   `replace_text_ctx`: gated on `auto_close && language_of(path)==Rust` (the app's Rust-only syntax wiring);
   captures the PRE-edit `src` (`buffer.text()`) + probes per-cursor (`src.char_indices().nth(off)` char→byte →
   `node_kind_at` → `context_from_node_kind`). Borrow-safe (src/is_rust read from `s` before `active_ime_mut()`;
   the `move` closure owns `src`, never the buffer). Non-Rust/off → `Context::Code` → the #338 behavior.

**Deviations:** (a) ime.rs added (5 files not 4) — the production caller is there, not app.rs; both got the
delegator pattern so NO existing caller churned except the ONE app.rs site opting into the closure. (b) the
delegator pattern (keep the old fns delegating with `Code`) — chosen over touching ~13 callers; cleaner + lower
risk. No `marley_editor`→`marley_syntax` dep (the classifier takes `Option<&str>`, Lang-free).

**Verification:** `cargo check --workspace` clean; `cargo clippy --workspace --all-targets -- -D warnings` rc=0
(the cross-crate signature changes all compile, test code included). `cargo fmt --all`. Diff = syntax + auto_close
+ buffer + ime + app.rs. No test EXPANSION (Phase 4). No Zed/Warp; pair_action stayed PURE (context in, no tree).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Scaled to **1 thorough critic** (general-purpose, behavior-preservation + correctness lens) + own review — a
cross-crate change (5 files). No CRITICAL/HIGH; the load-bearing Code==old surface is airtight.

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| F1 | MEDIUM | **Per-keystroke reparse.** The `plan` closure called `context_at(sel.start())` UNCONDITIONALLY, and it's reached for EVERY typed char (ime gates on any char, not just pair chars) → ordinary typing in a Rust file triggered a full-buffer reparse (×~2 per keystroke, N+1 for N cursors). | **FIXED at source.** `context` is consulted ONLY in the InsertPair arm (which needs `closer_of(typed).is_some()`), so gated the probe: `let ctx = if auto_close::closer_of(typed).is_some() { context_at(sel.start()) } else { Context::Code };` (buffer.rs). `context_at` is a closure → not calling it is free; `Code` is inert for a non-opener. Drops the reparse for the >95% non-opener case. cargo check clean. |
| F2 | LOW | **Boundary byte at a string/comment START classified as inside.** `descendant_for_byte_range(b,b)` at the exact start byte is start-inclusive, so `foo\|"x"` typing `(` classifies as StringOrComment → suppresses, where REQ-001 would pair. | **ACCEPT — documented.** Rare (typing an opener immediately before an existing string), non-corrupting, arguably nicer, and moot for `"`-before-`"` (TypeOver short-circuits before the context check). A robust boundary-aware probe adds API complexity not worth this edge. Live-only (the Code-threaded sweep doesn't reach it). Noted for a future refinement. |
| F3 | LOW | `doc_comment` not in the classifier set. | **ACCEPT — safe-by-default + likely already covered.** tree-sitter-rust renders `///` as `line_comment` (the #315 `doc_comment_double_capture_yields_one_span` test: one `Comment` span), so it IS covered. A miss would be under-suppression (Code → pairs = old behavior), never a mis-suppression. The P4 parse-a-fixture will confirm a `///` caret → StringOrComment; if not, add `doc_comment` then. |

Critic confirmed **OK** (own-spot-checked): (a) Code==old — the `&& context != StringOrComment` is a no-op when
Code (always true); no other arm changed; the 15 test calls got `Context::Code` as the top-level last arg
(spot-checked the sweep + a `false)` + a `true)`). (b) delegators verbatim-old-body + `sel.start()` (same as
`prev`). (c) the app closure — char→byte correct (no off-by-one; EOF→len), PRE-edit src+offset (correct
semantics), borrow-safe (shared reads before `active_ime_mut`, `move` owns src), the Rust gate matches
app.rs:3426. (d) node_kind_at uses `parser_for(lang)` (not hardcoded Rust), total. (e) classifier safe-by-default;
no listed name is a code node in a shipped grammar. (f) `crates/editor/Cargo.toml` gained NO `marley_syntax` dep;
pure fns take only small values; no regex; no unsafe; docs plain-backtick.

**Result: 1 MEDIUM fixed (the reparse gate), 2 LOW accepted+documented.** Lesson (capture at P5): a per-cursor
context probe on the HOT insert path must be gated to the chars whose action actually consumes it — else it pays
the cost on every keystroke. Lenses: Code-identity, delegators, app-closure, probe, classifier, purity/cycle.
`git status --porcelain` = the 5 src files + the #346 docs.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests written** (7 new, all green; the #338 26k sweep threads `Context::Code` = T1, unchanged & green):

| # | Test | File | Proves |
|---|------|------|--------|
| T1 | the #338 26k sweep + focused rows (Code-threaded) | auto_close.rs | REQ-CODE-BYTE-IDENTICAL — `Code` == old |
| T2 | `t346_no_pairing_inside_string_or_comment` | auto_close.rs | REQ-NO-PAIR-IN-STRING/COMMENT — every opener → Insert under StringOrComment |
| T3 | `t346_type_over_is_unaffected_by_context` | auto_close.rs | REQ-TYPEOVER-UNAFFECTED — `"` before `"` → TypeOver regardless of context |
| T4 | `t346_context_from_node_kind_classifies` | auto_close.rs | classifier: the string/comment name-set → StringOrComment; identifier/lifetime/function_item/None → Code |
| T5 | `node_kind_at_distinguishes_string_comment_and_code` | syntax/lib.rs | REQ-PROBE-CLASSIFIES — a real `HighlightSession` parse; `(` in a string → contains "string", in `/* */` → "comment", on `fn f` → neither |
| T5b | `node_kind_at_doc_comment_is_a_comment` | syntax/lib.rs | F3 resolved — `///` → a "comment" kind (classifier suppresses it) |
| T6a | `t346_context_probe_only_runs_for_an_opener` | buffer.rs | the F1 gate — a call-counted closure: closer `)` → 0 probes, opener `(` → ≥1 (kills the else-equivalent gate mutants) |
| T6b | `t346_opener_in_string_context_does_not_pair` | buffer.rs | end-to-end: `(` at co(3) with `StringOrComment` → "abc(" (no pair); `Code` control → "abc()" |

**Runs (`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley_editor -p marley_syntax` → **313 passed, 0 skipped** (the sweep + all 7 new).
- `cargo nextest run -p marley` (app regression) → **722 passed, 2 skipped** — the signature ripple (pair_action +ctx, buffer/ime delegators, app closure) regressed nothing.
- `cargo mutants --in-diff` on auto_close.rs+buffer.rs+syntax/lib.rs → **15 mutants: 11 caught, 4 unviable, 0 MISSED** (MSI 100 on the viable set). The 4 unviable = `pair_action`/`context_from_node_kind` → `Default::default()` (Action/Context derive no Default) + both `insert_pairing_at_selections*` → `SelectionSet::default()` (no Default) — the #203/#204 rule (a body-mutant's viability needs a Default derive). Guard mutants (`!=`→`==`, `&&`→`||`, `delete !`, guard true/false) all killed by T1 sweep + T2/T3; the classifier arm-delete by T4; node_kind_at None/empty/xyzzy by T5.
- `cargo mutants --in-diff` on ime.rs (the gate mutates the full diff) → **2 mutants: 2 caught** (both delegator/`_ctx` body → `()`, caught by the existing ime tests that route through them). app.rs closure is inside `replace_text_in_range` (`#[cfg_attr(test, mutants::skip)]`) → not mutated.

**NO LIVE DRIVE — stated.** chad is at the machine; synthetic input is off-limits, and a live type-in-a-string is only nice-to-have here. The pure `pair_action` sweep (Code==old) + the `node_kind_at` real-parse fixtures + the `_ctx` end-to-end buffer suppression (T6b) are the exact proof of every REQ. The F2 boundary-start edge is live-only and documented (accepted).

**Gate fixes (2 reds → green):**
1. **gate:14 docs** — a broken intra-doc link `[\`insert_pairing_at_selections\`]` on `_ctx`'s doc (a bare method name doesn't resolve). Fixed to `[\`Self::insert_pairing_at_selections\`]` (it's `pub`, so the `Self::` link resolves). Not plain backticks — the target is public, so a real link is better.
2. **gate:4 coverage** — buffer.rs 4 missed lines + 1 missed function. Root cause: `t346_context_probe_only_runs_for_an_opener` used TWO separate counting closures; the FIRST (passed for the closer `)`) is — correctly, by the F1 gate — never invoked, so its body (the increment + return) was dead → uncovered (and llvm-cov counts the closure as a missed function). Fix: ONE shared closure capturing `&calls` (a shared ref → the closure is `Copy`, so each call consumes a copy, not a move); the opener call exercises the body while the closer call still proves 0 probes. buffer.rs → 100% lines/functions/regions. (The other files' <100% *region*-cover is the normal baseline — gate:4 floors *lines*, and only buffer.rs had missed lines.)

**FULL `--diff` gate → `GATE GREEN [diff]` — 15/15 passed.** Receipt `2c90ba36017ba7f42da406f35ab03aa36cfb5677`. gate:4 coverage 100% lines (workspace TOTAL back to 0 missed), gate:5 MSI 100 (13 caught / 0 missed), gate:14 docs PASS. No pre-existing failures excluded.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):**
- CHANGELOG.md — `### Added` entry for #346 (syntax-aware auto-close; the tree, not a regex; opener-gated probe; Code==old; the lifetime bound deferred).
- `auto_close.rs` `blocked_quote` doc — updated: #346 took the OTHER half (strings/comments via `Context`); this guard is unchanged and the lifetime BOUND stays deferred (no `lifetime` node in the pre-insert tree while typing `'`), tracked as the follow-up.
- `docs/marley_architecture/editor.md` — the `auto_close.rs` section gained two sub-bullets: `pair_action`'s syntax-aware `context: Context` (the node_kind_at → context_from_node_kind → Code/StringOrComment chain, Lang-free classifier, no editor→syntax dep, Code byte-identical to #338) + the buffer's per-cursor delegator threading (`insert_pairing_at_selections` → `_ctx`, opener-gated). The `'`-guard bullet's "awaits #315" → "awaits the tree (a follow-up of #346)".

**Capture (forge wired):**
- **aar-submit** 0300a225-e00a-4e80-94ce-03543bc9482f — outcome completed, effectiveness 4, 2 novel findings, 13 verdicts. Lessons: (a) the ticket's "the node kind at the caret answers the `'` directly" is half-wrong for the typing flow (pre-insert tree has no lifetime node) → the string/comment slice is clean, the bound slice deferred; (b) the delegator pattern kept ~13 callers + the ime path unchanged, Code==old for free; (c) a call-counted test closure never invoked leaves its body uncovered + a missed function → share one `Copy` closure; (d) reinforced #203/#204 — a body→`Default::default()` mutant is unviable without a Default derive (Action/Context/SelectionSet lack it → 4 unviable, legitimately excluded).
- **prevention-rule-record** ×2: `PR-claude-verify-tree-answers-directly-at-pre-edit-caret-001` (ba51b76d) + `PR-claude-shared-copy-closure-for-a-call-counted-probe-test-001` (fa4c974a).
- **failure-record**: none — F1 (the reparse) was caught+fixed in inspect (not a shipped bug); the two gate reds (intra-doc-link + uncovered-closure) were self-inflicted-then-fixed within validate, effectiveness-neutral, not recorded as failures.
- **★ Follow-up filed:** forge **#362** (bc01c12e-931f-4e92-a9da-44a82d603ee6) — "Auto-close: suppress `'`-pairing at lifetime-bound positions" — the deferred half; needs ancestry detection (a caret inside a `where_clause`/`trait_bounds`/`type_parameters`), since the pre-insert tree has no `lifetime` node while typing `'`.

**Close + archive:** forge #346 closed; TICKET-346 → closed/; pair archived active/ → completed/; spec Phase 5 PASS.

⚠️ **/commit MUST RE-RUN the full `--diff` gate** — the `auto_close.rs` `blocked_quote`-doc edit is a `.rs` change → it stales the Phase-4 receipt (`2c90ba36…`); the CHANGELOG + editor.md are docs-only. Do NOT verify the fingerprint.

**Status: Phase 5 — Complete PASS.**
