---
pipeline_id: b39d555a-49a4-4095-b626-bdf7594b3ddc
ticket: forge#362 (bc01c12e-931f-4e92-a9da-44a82d603ee6) · local docs/planning/tickets/open/TICKET-362-lifetime-bound-auto-close.md
aar_id: 865d79f1-e854-4f82-bd8e-6781ec822db5
status: Phase 5 — Complete PASS
title: Auto-close suppresses '-pairing inside a type_parameters list (a new marley_syntax ancestry probe + Context::LifetimeBound)
type: feature
milestone: M22
references: [auto_close.rs:125 pair_action / :165-168 the ' pairing arm / :202 blocked_quote (the #338 guard + its "deferred" doc :189-201) / :211 Context enum / :223 context_from_node_kind, syntax/lib.rs:526 node_kind_at (the probe template) / :489-511 enclosing_ranges (the .parent() walk idiom) / :361 HighlightSession::tree, app.rs:13682-13689 the Context-build shim, #346 #338 #340 #299]
---

## Title
Add a pure `marley_syntax` ancestry probe (`in_type_parameters`-style) — a caret inside a tree-sitter
`type_parameters` node typing `'` is a lifetime, not a char literal — threaded via a new
`auto_close::Context::LifetimeBound` variant so `pair_action`'s `'` arm suppresses the pair at declaration-site
lifetime positions (`fn f<'a>`, `fn f<T: 'a>`). Rust-only v1; `where`/`dyn` positions deferred.

## Scope
### In
- A pure `marley_syntax` fn (name at design, e.g. `in_lifetime_bound_position(src, lang, byte_pos) -> bool`) that
  parses `src`, `descendant_for_byte_range(pos,pos)`, walks `.parent()`, and returns true when a `type_parameters`
  ancestor is found (the spike-confirmed node). Mirrors `node_kind_at` + the `enclosing_ranges` parent-walk.
- Extend `auto_close::Context` with a `LifetimeBound` variant; fold it into `blocked_quote` (or the `'` arm) so
  `LifetimeBound` suppresses ONLY `'` — a non-`'` opener still pairs (LifetimeBound ≠ StringOrComment).
- The app Context-build (app.rs:13682): after the string/comment `node_kind` check (StringOrComment wins), when
  `'` is being typed and the probe says type_parameters → `Context::LifetimeBound`.

### Out (explicitly deferred)
- **`where T: 'a` positions** — the SPIKE proved the pre-insert incomplete where-clause has NO `where_clause`
  ancestry (no `<>` structural anchor; the caret falls to `function_item`). Not reachable via ancestry pre-insert.
- **`dyn Trait + 'a` / `Ref<'a, T>` positions** — `type_arguments` ancestry (a type-USAGE `<>`), broader than
  the declaration-site scope; a possible v2 (`type_arguments` has no char-literal risk EXCEPT a rare const-generic
  block) — deferred to keep v1 the clean, zero-false-positive core.
- Non-Rust files (the probe is `Lang::Rust`; the caller gates on Rust, the #340/#346 pattern).

## Reference (§20)
N/A — Marley's own auto-close + tree-sitter-rust ancestry. No Warp/Zed source read.

### Prior art
- **Our own code:** #346's `Context`/`context_from_node_kind`/`pair_action` seam (auto_close.rs) is the exact
  structure to extend; `blocked_quote`'s doc (:189-201) literally names this cut and its approach ("ancestry
  detection — a caret inside a `where_clause`/`trait_bounds`/`type_parameters`"). `node_kind_at` (syntax/lib.rs:526)
  is the parse-then-probe template; `enclosing_ranges`/`all_headers` already walk `.parent()`.
- **Our permissive deps (the highest-yield leg — the SPIKE):** tree-sitter + tree-sitter-rust (reading the grammar
  / probing the parse is adoption, outside §20). The Phase-1 spike PARSED the real pre-insert fixtures and RECORDED
  the ancestry: `fn f<T: '|>` / `fn f<'|>` / `struct S<T: '|>` → `type_parameters` (the `<>` anchors it even with
  the incomplete bound); `where T: '|` → NO ancestry (falls to `function_item`); `dyn Trait + '|` → `type_arguments`;
  `let c = '|` → `let_declaration` (NO type_parameters → no false positive). This dissolved the "just check for a
  lifetime node" naive plan and scoped v1 to `type_parameters`.
- **Behavior maps / published:** N/A — a standard editor auto-close-in-context behavior; no reference-app source.

## Locked-In Decisions
- **D1 — the signal is a `type_parameters` ANCESTOR** (spike-confirmed pre-insert-resolvable). Covers the
  declaration-site `<...>`: `fn f<'a>`, `fn f<T: 'a>`, `fn f<T: Send + 'a>`, `struct/enum/impl/trait <'a>`. A char
  literal is never inside `type_parameters` (the rare const-generic-default `<const C: char = 'x'>` would
  under-suppress — the SAFE direction, you type the closing `'` yourself — not a regression).
- **D2 — a new `Context::LifetimeBound` variant that suppresses ONLY `'`.** Fold into `blocked_quote(typed, prev,
  context)` (or the `'` arm): `'` after `&/</word` OR (`typed == '\'' && context == LifetimeBound`). A non-`'`
  opener (`(` at `T: Fn(`) still pairs because the pairing condition keeps `context != StringOrComment` and
  LifetimeBound ≠ StringOrComment. `pair_action` stays PURE (Context in, never the tree — the #299 trap).
- **D3 — StringOrComment PRECEDENCE.** The app builds Context: string/comment `node_kind` first (a `'` inside a
  string is a string, not a bound), else the type_parameters probe → LifetimeBound, else Code.
- **D4 — `where` / `dyn` DEFERRED** (spike: not pre-insert-resolvable / broader). v1 is the clean type_parameters
  core; a follow-up can add `type_arguments` (with the const-generic-block caveat) + a where-clause approach.
- **D5 — cov/MSI: the ancestry probe + `blocked_quote` + `pair_action` are PURE** (marley_syntax + editor, both
  cov-INCLUDED → cov/MSI 100 via units); the app Context-build is the shim (app.rs cov-excluded → mutants::skip if
  it has viable mutants, the #361 lesson).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-BOUND-SUPPRESSES | WHEN `'` is typed inside a `<...>` type-parameter list (`fn f<'`, `fn f<T: '`, `struct S<T: '`), `pair_action` shall return `Insert` (no pair). | pure units: the ancestry probe → true for the `type_parameters` fixtures; a `pair_action` row (context=LifetimeBound, typed=`'` → Insert). |
| REQ-CHAR-LITERAL-PAIRS | WHEN `'` is typed at a char-literal position (`let c = '`), the system shall still `InsertPair` (`''`). | pure units: the probe → false for `let c = '|`; the #338 `pair_action` rows (context=Code) stay InsertPair. |
| REQ-NON-QUOTE-OPENER-PAIRS | WHEN a non-`'` opener (`(` at `T: Fn`) is typed at a bound, it shall still `InsertPair`. | pure unit: `pair_action('(', …, context=LifetimeBound) → InsertPair`. |
| REQ-346-UNCHANGED | The string/comment suppression (#346) shall be unchanged. | the #346 `pair_action`/`context_from_node_kind` units stay green. |

## Phase Plan
- **P2 Design** — settle the probe fn name/signature + the `type_parameters`-only scope (confirm the spike's node
  kind against the grammar version) + the `Context::LifetimeBound` + `blocked_quote(context)` threading + the app
  Context-build precedence + the test lane (probe units on the spike fixtures + `pair_action` rows + a headless
  drive nice-to-have).
- **P3 Implement** — the marley_syntax probe + the auto_close Context/blocked_quote + the app shim.
- **P3.5 Inspect** — ★ the probe returns true ONLY for type_parameters (not a char literal); `pair_action` pure
  (#299); LifetimeBound suppresses only `'`; StringOrComment precedence; Rust-gated; the pure fns cov/MSI 100; the
  app shim mutants::skip'd if it has mutants (#361); the mutants::skip detach trap checked (`cargo mutants --list`).
- **P4 Validate** — the pure units (probe + pair_action) + the `--diff` gate.
- **P5 Complete** — CHANGELOG + un-defer the `blocked_quote` doc (:189-201 says "deferred" — #362 does it) + editor.md.
