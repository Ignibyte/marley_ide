---
pipeline_id: 4e610839-3f3d-4dbc-9ad5-b969c71f71c3
ticket: docs/planning/tickets/open/TICKET-366-auto-close-where-clause-lifetimes.md
status: Phase 5 — Complete PASS
title: Auto-close suppresses '-pairing at where-clause + dyn/type-usage lifetime positions via a speculative post-insert parse (the #362 deferred half)
type: feature
milestone: M22
references: [auto_close.rs:126 pair_action / :199 blocked_quote / :210 Context (LifetimeBound :219), app.rs:16873-16893 the Context closure (captures `text`), syntax/lib.rs:538 node_kind_at / :556 in_type_parameters (the probe template), 362-lifetime-bound-auto-close.spec.md (the shipped half), #346 #338, tree-sitter-rust 0.24.2 (pinned)]
---

## Title
Finish #362's deferred half: suppress `'`-pairing at the remaining lifetime-bound positions — `where T: 'a`
(all variants) and the type-USAGE positions (`Box<dyn Trait + 'a>`, `Ref<T, 'b>`, `&(dyn Trait + 'a)`,
`-> impl Iterator + 'a`) — via ONE new pure `marley_syntax` probe: a **speculative post-insert parse** (splice
`'a` at the caret, parse, the covering node kind == `lifetime` → `Context::LifetimeBound`). The Phase-1 spike
(19 fixtures, pinned tree-sitter-rust 0.24.2) proved this discriminates PERFECTLY where ancestry cannot: every
lifetime position parses `lifetime`; every char-literal/expr/const-generic-block control parses `label`/ERROR.
The ticket's const-generic guard falls out for free; its hand-rolled backward-scan option dies (the #339 pattern
— the substrate already owns the seam).

## Scope
### In
- A new pure `marley_syntax` fn (name at design, e.g. `speculative_lifetime_at(src, lang, byte_pos) -> bool`):
  splice `'a` into a copy of `src` at `byte_pos`, parse, `descendant_for_byte_range(byte_pos, byte_pos+2)`,
  return `kind == "lifetime"`. Combinator shape (no `?`), mirroring `in_type_parameters` — cov/MSI 100 via units.
- The app Context closure (app.rs:16873): it already captures the inserted `text`; when `text == "'"` and the
  cheaper answers didn't decide (StringOrComment wins; `in_type_parameters` true already → LifetimeBound), run
  the speculative probe → `Context::LifetimeBound`. **No editor-crate change**: `pair_action`/`blocked_quote`/
  `Context` already gate LifetimeBound to `'` only (#362).
- A non-vacuous headless drive for a `where` position (double-space fixture per the #362 prevention rule —
  `next` must be whitespace so a Code context WOULD pair; only LifetimeBound suppresses).
- Probe units on the spike's fixture classes: where (body/EOF/+Send/2nd-pred/struct/impl/trailing-comma), dyn
  in `Box<>` (+EOF), `Ref<T, >`, dyn-paren, impl-trait return → TRUE; const-generic blocks (`Foo<{ }>`,
  turbofish), char-literal, expr-plus, call-arg, `<`-comparison → FALSE; empty/EOF-degenerate src.

### Out (explicitly deferred)
- **Loop labels** (`'outer: loop`): the splice parses `label` at those positions — deliberately NOT suppressed
  (kind gate is `lifetime` only). Labels keep today's pairing behavior; a possible follow-up.
- Non-Rust files (the caller stays Rust-gated — the #340/#346/#362 pattern; #315 owns multi-language).
- Replacing/folding the shipped `in_type_parameters` probe (the splice subsumes its TRUE cases, but retiring a
  tested #362 surface is not this ticket; design may ORDER the two probes for cost, not remove one).
- The tree cache (#363) — the splice parses a MODIFIED copy, inherently uncacheable; cost is gated to `'`
  keystrokes only (rare), so no cache work here.

## Reference (§20)
N/A — Marley-specific auto-close depth. The reference apps inform the FAMILY behavior (editors suppress bracket
pairing contextually — Zed behavior maps list the inventory, e.g. `jsx_tag_auto_close.rs`, without Rust-`'`
semantics), but the `where`/`dyn` lifetime discrimination is Marley's own mechanism built on its own substrate
(tree-sitter-rust error recovery). No Warp/Zed source read; no observed capture needed — the acceptance bar is
"a `'` typed at a lifetime position inserts bare", asserted headlessly.

### Prior art
1. **Behavior maps** — `docs/zed_architecture/crates/editor.md` (:145-149) inventories Zed's bracket/auto-close
   surface (nothing on Rust `'` bound handling); `docs/warp_architecture/` is terminal-side, no owner. Research
   only; nothing to adopt.
2. **Published** — the Rust Reference's token grammar: `'IDENT` is lifetime-or-label, disambiguated by
   POSITION (bounds take lifetimes; expressions take labels) — exactly the property the probe reads back out of
   the parser. tree-sitter's error recovery is documented as best-effort/unspecified, so the discrimination was
   NOT assumed: it was spike-verified against the PINNED grammar (0.24.2) we ship.
3. **Our permissive deps (the paying leg)** — tree-sitter + tree-sitter-rust 0.24.2 (`~/.cargo/registry/src/…`,
   adoption outside the wall): node-types.json confirms `where_clause`/`trait_bounds`/`lifetime`/`label`/
   `type_arguments`/`const_block` kinds; the Phase-1 SPIKE (scratchpad crate, 19 fixtures, 3 probe modes)
   recorded: (a) pre-insert ancestry — NO `where` anchor exists (all where variants → `function_item`/
   `struct_item`/`impl_item`/`source_file`, body present or not) → #362's ancestry approach CANNOT extend to
   `where`; (b) `type_arguments` ancestry exists for `Box<dyn…>`/`Ref<T,…>` but misses dyn-paren + impl-trait
   return; (c) the SPECULATIVE `'a` splice → node kind `lifetime` at ALL 13 lifetime positions (incl. where-EOF
   via `function_signature_item` recovery) and `label`/ERROR at ALL 6 non-lifetime controls (incl. both
   const-generic blocks — the ticket's guard falls out free). **The ticket's option (a) hand-rolled backward
   text scan DIES to the substrate** (it would false-positive at `x + '`); option (b) wins outright — the #339
   `regex`/`find_iter` pattern, recorded as the win it is.

## React-first (parity)
N/A — no UI delta: this changes keystroke auto-close SEMANTICS inside the Rust editor buffer (whether a typed
`'` inserts `''` or `'`), not any chrome, overlay, surface, layout, type, color, or affordance. The React POC
has no code-editor auto-close engine to mirror; parity zones (MARLEY-PARITY.md) cover what is SEEN, and the
seen surface here (text appearing as typed) is unchanged in shape.

## Locked-In Decisions
- **D1 — ONE signal for BOTH deferred classes: the speculative post-insert parse.** Splice `'a` at the caret
  byte, parse the copy, `descendant_for_byte_range(pos, pos+2)`, `kind == "lifetime"` → LifetimeBound.
  Spike-proven on the pinned grammar: 13/13 lifetime positions TRUE (where body+EOF, +Send, 2nd predicate,
  struct/impl where, trailing-comma predicate start, Box<dyn+>, Box<dyn+ EOF, Ref<T,>, dyn-paren, impl-trait
  return), 6/6 controls FALSE (const-generic block ×2 → `label`, char-literal, expr-plus, call-arg,
  `<`-comparison → `label`/ERROR). NOT adopted: extending ancestry with `type_arguments` (+ a hand block-guard)
  — strictly weaker (misses where entirely, misses dyn-paren + impl-return, needs the guard the splice gets free).
- **D2 — the probe is a NEW pure `marley_syntax` fn**, combinator shape (`.map(...).unwrap_or(false)`, no `?`,
  no early return — the `in_type_parameters`/`node_kind_at` cov-100 template). Splice via byte-safe
  `String` build (`src[..pos] + "'a" + src[pos..]`; design settles the char-boundary contract with the caller,
  which already computes a real char-boundary byte).
- **D3 — `'`-keystroke gating in the APP closure, zero editor-crate delta.** The closure already captures the
  inserted `text` (app.rs:16873); the splice runs only when `text == "'"`. Precedence: StringOrComment first
  (#346, unchanged) → `in_type_parameters` (#362, unchanged — its TRUE already yields LifetimeBound without the
  splice) → the splice (NEW) → Code. `pair_action`/`blocked_quote`/`Context` are UNTOUCHED (#362 already gates
  LifetimeBound to `'`).
- **D4 — labels do NOT suppress.** The kind gate is exactly `"lifetime"`; `label` positions (loop labels, expr
  recovery) keep pairing as today. Out-of-scope follow-up, documented at the probe.
- **D5 — cost is bounded and honest**: the extra parse runs ONLY on `'` keystrokes in Rust files outside
  strings/comments not already inside `type_parameters` — rare by construction. No cache interaction (#363's
  cache is for the live text; the splice text differs by construction).
- **D6 — cov/MSI**: the new probe is pure → units carry cov/MSI 100 (marley_syntax is cov-included). The app
  closure lives inside `replace_text_in_range` (already `mutants::skip` — #362 confirmed it enumerates ZERO
  mutants); the headless drive carries its wiring, non-vacuously (double-space fixture; baseline-would-pair
  traced per PR-…-suppression-test-must-prove-the-baseline-would-not-suppress).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-WHERE-SUPPRESSES | WHEN `'` is typed at a where-clause bound position (`where T: |`, `where T: Send + |`, a 2nd predicate `U: |`, struct/impl where, at EOF or with the body present), the system shall insert a bare `'` (no `''`). | probe units: splice→`lifetime` TRUE on every where fixture; headless drive: double-space `where T:  {}` fixture → buffer contains bare `'`, `!contains("''")`. |
| REQ-DYN-SUPPRESSES | WHEN `'` is typed at a type-usage lifetime position (`Box<dyn Trait + |>`, `Ref<T, |>`, `&(dyn Trait + |)`, `-> impl Iterator + |`), the system shall insert a bare `'`. | probe units TRUE on each; the pure `pair_action(…, LifetimeBound)` row already ships (#362). |
| REQ-CONST-BLOCK-PAIRS | WHEN `'` is typed inside a const-generic block (`Foo<{ | }>`, `::<{ | }>` turbofish), the system shall still pair (`''`). | probe units FALSE on both block fixtures (spike: `label`, never `lifetime`). |
| REQ-CHAR-LITERAL-PAIRS | WHEN `'` is typed at a char-literal/expression position (`let c = |`, `x + |`, a call argument), the system shall still pair (`''`). | probe units FALSE on the expr controls; the #338 `pair_action` Code rows stay green. |
| REQ-PRIOR-UNCHANGED | The #346 string/comment suppression and the #362 `type_parameters` suppression shall be unchanged. | the existing #338/#346/#362 unit + drive suites stay green (regression run at validate). |

## Phase Plan
- **P2 Design** — settle the probe name/signature + splice mechanics (char-boundary contract, EOF byte); the
  app-closure branch order + `text == "'"` gate; the unit fixture table (spike-derived, incl. degenerate
  bytes); the drive fixture (double-space, baseline-traced); confirm zero editor-crate delta; check
  `cargo mutants --list` expectations.
- **P3 Implement** — the marley_syntax probe + units; the app-closure branch; the headless drive.
- **P3.5 Inspect** — critics: probe purity/cov-shape; false-positive hunt (any expr position where the splice
  parses `lifetime`?); the drive's baseline traced (vacuous-drive PR); closure precedence (StringOrComment
  first); §20 provenance.
- **P4 Validate** — RUN the units + drives + full regression on the three crates; `scripts/gates.sh --diff` green.
- **P5 Complete** — CHANGELOG; un-defer the `pair_action`/`blocked_quote` doc lines (:120-125, :193-198 name
  where/dyn as deferred — #366 closes them); editor.md architecture note; ledger capture; close TICKET-366
  (row already off the backlog); archive the pair.
