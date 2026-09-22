---
pipeline_id: 4010be08-ec1c-4eeb-a464-0d6a59db8171
ticket: forge#346 (3cda6024-b130-4792-9f12-eb9ccfcb2506) · local docs/planning/tickets/open/TICKET-346-auto-close-syntax-aware.md
aar_id: 0300a225-e00a-4e80-94ce-03543bc9482f
status: Phase 5 — Complete PASS
title: Auto-close — no pairing inside strings/comments (the tree-aware #338 cut); lifetime-bound spike
type: feature
milestone: M22
references: [auto_close.rs:125 pair_action / :186 blocked_quote, buffer.rs:341 insert_pairing_at_selections, syntax/lib.rs:563 delims_at (the descendant_for_byte_range().kind() probe), app.rs:3464 HighlightSession throwaway, tree-sitter-rust node-types (lifetime/char_literal/line_comment/block_comment/string_literal/string_content/raw_string_literal), #338 auto-close AD, #315 syntax AD, #299 lang_spec "second source of language facts" trap, #340 delims_at]
---

## Title
Make auto-close **syntax-aware**: suppress pairing when the caret is inside a string or a comment, using the
tree-sitter tree. Keep `pair_action` pure by threading a `Context` in. The `'` lifetime-bound positions are a
DESIGN SPIKE (they have a pre-insert-tree obstacle) — likely a follow-up.

## Scope
### In
- A `Context` enum (Code · StringOrComment · [CharLiteral · Lifetime — see the spike]) and a pure classifier in
  `marley_editor::auto_close`, plus a thin `marley_syntax::node_kind_at(...) -> Option<String>` tree probe.
- `pair_action` gains a `context` parameter; `StringOrComment` → `Insert` (no pairing) for every pair char;
  `Code` → the existing #338 behavior, byte-identical.
- `Buffer::insert_pairing_at_selections` threads a per-cursor context (a `context_at` closure or a `&[Context]`).
- The app.rs wiring: probe the tree (a throwaway `HighlightSession` reparse, gated on `auto_close_on` + a pair
  char) at each caret → the classifier → the buffer's insert.

### Out (explicitly)
- **The `'` lifetime-bound positions (`T: 'a`, `+ 'a`, `'static`) — a DESIGN SPIKE, recommended DEFERRAL.** The
  tree is parsed from the text BEFORE the `'` is inserted, so at `T: |` / `let c = |` there is NO `lifetime` or
  `char_literal` node yet — the ticket's "the node kind at the caret answers it directly" is FALSE for the
  typing flow. Separating a bound from a char literal needs ancestry-based bound detection (is the caret's
  ancestor a `where_clause`/`trait_bounds`/`type_parameters`?) or speculative post-insert parsing — a distinct,
  per-language, error-prone problem. Design SPIKES it; if not clean+cheap, it becomes a follow-up and #338's
  one-char `blocked_quote` stays (the bound positions keep pairing, as #338 already documents). NOT shipped
  half-broken.
- The tree cache (#349, later in this goal) — #346 ships a throwaway reparse; #349 can swap the source.
- Any change to the #338 `Code`-context behavior (the 26k sweep stays byte-identical).

## Reference (§20)
N/A — auto-close is Marley's own editor behavior (no Warp/Zed source read). "No pairing inside strings/comments"
is a universal editor expectation (observed, e.g. VS Code's `autoCloseBefore` + its string/comment guard);
reading the tree-sitter grammars (node-types.json) is ADOPTION (outside the wall).

### Prior art
1. **OUR OWN CODE (the seam is built):** `syntax::delims_at` (syntax/lib.rs:563) already probes
   `root.descendant_for_byte_range(pos, pos).kind()`, and its doc CONFIRMS (spike-verified) the tree lands in
   `string_content` — NOT on a `"("` node — for a `(` inside a string. So `node_kind_at` is a thin extraction of
   that pattern. `pair_action`/`blocked_quote` (#338) is the pure table; `HighlightSession` + the #315 per-Lang
   grammars are the parse. The #340 pattern is the model.
2. **tree-sitter (adoption):** `tree-sitter-rust` node-types confirm `line_comment`/`block_comment`/
   `string_literal`/`raw_string_literal`/`string_content`/`char_literal`/`lifetime`. A name-based classifier is
   viable (design verifies the non-Rust comment/string names).
3. Checked ropey/regex — N/A (this is a parse-tree question; the #299 trap FORBIDS a regex "am I in a string").

## Locked-In Decisions
- **D1-CONTEXT-IN-NOT-TREE** — `pair_action` stays PURE: a `context: Context` param, computed by the caller.
  NO regex/heuristic "am I in a string" (the #299 `lang_spec` "second source of language facts" trap). The tree
  is the ONLY source.
- **D2-STRING/COMMENT-FIRST** — the shippable slice is the string/comment suppression, which the pre-insert tree
  cleanly answers (the caret is INSIDE an existing string/comment leaf). The lifetime bound is the spike (Out).
- **D3-CRATE-BOUNDARY** — `marley_editor` ⊥ `marley_syntax` (no dep either way; app deps both). RECOMMEND
  (A): `Context` + a Lang-FREE name-based classifier in `marley_editor::auto_close` (cov/MSI 100, no Lang dep) +
  a thin `marley_syntax::node_kind_at` probe + app.rs wiring — no dep cycle, the classifier stays a pure
  editor-crate seam. Design verifies the cross-language kind-names don't collide (else fall to a shared-crate
  Context). Probe returns the raw node-kind `String`; the editor classifier maps kind-name SETS.
- **D4-THROWAWAY-REPARSE** — the probe reparses via a throwaway `HighlightSession` per pair-char keystroke,
  gated on `auto_close_on` (rare — 6 chars, feature-on). Correct + simple; #349 can later cache. A file with no
  supported Lang → `Context::Code` → the #338 behavior.
- **D5-CODE-IS-OLD** — `Context::Code` (and a no-tree/unsupported-lang fallback) MUST reproduce the #338 26k
  sweep byte-identical — the pure fn's Code arm is the untouched old logic + `blocked_quote`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-NO-PAIR-IN-STRING | WHEN a pair char is typed with the caret inside a string, the system shall Insert (no pairing). | unit: `pair_action('(', prev, next, false, Context::StringOrComment) == Insert` (every opener). |
| REQ-NO-PAIR-IN-COMMENT | WHEN a pair char is typed inside a comment, the system shall Insert. | same via `Context::StringOrComment`. |
| REQ-CODE-BYTE-IDENTICAL | WHEN the context is Code, the system shall behave exactly as #338. | the existing 26k sweep re-run with `Context::Code` threaded == the recorded actions; the #338 tests stay green. |
| REQ-PROBE-CLASSIFIES | The tree probe + classifier shall return StringOrComment for a caret inside a Rust `"s"`/`//c`/`/*c*/` and Code elsewhere (per supported language). | parse-a-fixture units on `node_kind_at` + `context_from_node_kind` (a real `HighlightSession` parse + a caret assert). |
| REQ-TYPEOVER-UNAFFECTED | The context shall NOT change TypeOver (`"a|"` + `"` steps past) — it gates only the InsertPair arm. | unit: `pair_action('"', _, Some('"'), false, StringOrComment) == TypeOver`. |

## Phase Plan
- **P2 Design** — ★ the lifetime-bound SPIKE (parse `fn f<'a>` / `T: 'a` / `let c='x'`, probe the pre-insert
  caret, CONFIRM no lifetime/char node → decide defer vs ancestry) + settle D3 (verify non-Rust kind-names) +
  the `Context` enum + classifier + `node_kind_at` + the `insert_pairing_at_selections` threading + the app.rs
  probe site (char→byte, the file's Lang, the throwaway reparse) + the test plan.
- **P3 Implement** — the enum + classifier (editor) + `node_kind_at` (syntax) + `pair_action` ctx + the
  threading + the app.rs wiring.
- **P3.5 Inspect** — pair_action stayed pure; Code==old (the sweep); no dep cycle; the #299 no-heuristic;
  the classifier kind-name coverage; the probe on the correct char→byte offset.
- **P4 Validate** — the sweep-with-Code-identity + the new-context rows + the classifier parse-a-fixture units +
  the `--diff` gate. **Live type-in-a-string drive DEFERRED (chad at machine); the pure + parse units carry it.**
- **P5 Complete** — CHANGELOG + auto_close.rs/blocked_quote doc (string/comment now covered; the bound spike's
  outcome) + editor.md; file the lifetime-bound follow-up if deferred.
