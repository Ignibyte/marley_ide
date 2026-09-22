---
pipeline_id: 73aadd5a-9d21-4916-a365-e467be2110f8
ticket: docs/planning/tickets/open/TICKET-333-hand-lexer-inlay-phantom.md
status: Phase 5 — Complete PASS
title: editor — the hand-lexer fallback lexes inlay phantom text (raw-lex + map fix)
type: bug
milestone: M-unset (m21-followup)
references:
  - docs/planning/pipeline/completed/331-inlay-hints.notes.md
  - docs/planning/knowledge/architecture-decisions.md (AD-claude-two-boundary-maps-for-phantom-text-001)
---

## Title
On a row the tree-sitter cache cannot serve, the render falls back to
`code_syntax::highlight_ranges(&layout.display, lang)` — lexing the DISPLAY
string, which since #331 contains server-controlled inlay phantom text. A hint
carrying a quote / lifetime tick / `//` can start a string/char/comment token
INSIDE the phantom whose state bleeds RIGHTWARD into real code (mis-colored
cells only; layout/caret/click/selection stay correct). Fix: give the fallback
the primary path's shape — lex the RAW buffer line, map each span through
`raw_span_to_display_bytes` (phantom-safe by construction since #331's
`col_of_span_end`).

## Scope
### In
- The main row-render fallback in `crates/marley_app/src/app.rs` (~6486–6508):
  fallback lexes the raw `text` already in scope and maps spans through
  `raw_span_to_display_bytes(&text, &layout, span)` — identical shape to the
  primary tree-sitter arm directly above it.
- The sticky-header fallback (~7055–7073) — same shape adopted IF design rules
  it safe (its layout has no inlays; behavior must stay identical). Design
  decides unify-vs-leave (D5).
- Regression tests: floor-language (TOML) no-phantom identity; phantom-bleed
  fix proof on the fallback path; tab-bearing lines.

### Out (explicitly deferred)
- No change to the hand lexer itself (`lang_spec` tables), grammar coverage,
  the inlay pipeline, cache keying/invalidation, or the hints-first span
  ordering (retained as belt-and-braces).
- No display-lex + span-remap variant (killed by the tab-expansion trap — a
  phantom changes tab-stop expansion, so the two display strings are not
  related by insertion; #331 F8).
- The hover-card fenced-code lexing (`app.rs` ~15567) — already lexes raw text
  with no layout; untouched.
- The exotic-line-separator root cause (`lines.get(row)` permanently None on
  bare `\r`/FF/NEL/LS/PS files) — separate concern, not this ticket.

## Reference (§20)
**Zed (editor reference — same-gpui-stack).** Behavior matched: in Zed's editor
the display pipeline is a stack of coordinate TRANSFORMS over buffer-anchored
syntax (inlay → fold → tab → wrap → block → highlight, per our behavior map
`docs/zed_architecture/subsystems/03-editor-multibuffer.md`); syntax
highlighting is computed from BUFFER text and inlay hint text is never lexer
input, so code coloring is identical with hints on or off. Marley matches that
behavior: the fallback lexes the raw buffer line and only MAPS the result into
display space; phantom text can never seed lexer state. Behavior-level only —
observed stack order from our own transcribed map, no GPL source read (§20).

### Prior art
1. **Behavior maps:** `docs/zed_architecture/subsystems/03-editor-multibuffer.md`
   — the DisplayMap transform stack places highlighting over buffer-anchored
   spans with inlays as a prior coordinate transform (never lexed);
   `docs/zed_architecture/crates/editor.md` (inlay_highlights as a separate
   highlight source). This is exactly the "lex raw, map to display" shape.
2. **Published material:** LSP 3.17 `textDocument/inlayHint` — hints are
   presentation-only annotations rendered by the client; they are not document
   content. Supports the contract that hint text must not affect tokenization.
3. **Permissive deps:** no crate we ship owns this seam — checked tree-sitter
   (spans are source-byte domain; no display-string concept — its shape is the
   one the fix adopts), gpui (rendering, no lexing), ropey (storage). The
   owner is IN-REPO prior art: #331's `raw_span_to_display_bytes` +
   `col_of_span_end` (AD-claude-two-boundary-maps-for-phantom-text-001
   explicitly prescribes this fix for #333), and the primary tree-sitter arm
   at the same call site is the shape to copy verbatim.

## React-first (parity)
N/A — no UI delta: this is a correctness fix on a rare Rust-side render
fallback — it makes actual colors match the ALREADY-INTENDED colors (what the
primary path renders); no chrome/layout/type/color design changes. The React
POC has no hand-lexer/inlay-phantom seam to mirror (the fallback is a
Rust-internal mechanism), so there is nothing to build React-first.

## Locked-In Decisions
- D1 — FALLBACK-SHAPE: the fallback adopts the primary path's exact shape:
  `highlight_ranges(&text, lang)` on the RAW line, then per-span
  `raw_span_to_display_bytes(&text, &layout, span)`. No display-string lexing
  remains in the row render. (Prescribed by
  AD-claude-two-boundary-maps-for-phantom-text-001.)
- D2 — NO-REMAP: lexing the display string and shifting spans afterward is
  rejected — tab-stop expansion makes the phantom-free and phantom-bearing
  display strings not insertion-related (#331 F8 trap).
- D3 — HINTS-FIRST-STAYS: the hints-first ordering into the span splitter is
  retained unchanged (defence in depth; phantom's own cells paint Hint
  regardless of producer).
- D4 — FLOOR-REGRESSION: with no phantoms present the new shape must cover the
  same rendered text per token: tab-free lines assert span identity vs the old
  display-lex; tab-bearing lines assert per-token covered display text is
  identical.
- D5 — HEADER-SITE (settled at Design): UNIFY. The hand lexer treats `\t`
  exactly like a space (neither can start/alter any non-Plain token; Plain is
  dropped), so with no phantoms raw-lex + map is byte-identical to the old
  display-lex — proven by identity tests (T3/T4). Unifying removes the last
  display-string lex in the codebase.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a row renders via the hand-lexer fallback and its layout contains inlay phantom text, the system shall lex the RAW buffer line and map each span through `raw_span_to_display_bytes`, so that no fallback token span begun inside a phantom covers display bytes of real code to its right. | Unit test: Rust line + phantom carrying `"` / `'a` / `// fn main` → cells right of the phantom lex identically with hints present vs absent; negative control shows the old display-lex shape DID bleed on the same input. |
| REQ-002 | WHEN the fallback renders a floor-language line (TOML permanent floor; transient floor for grammar languages) with no inlays present, the system shall produce the same per-token covered display text as the pre-change display-lex — EXCEPT a zero-width char (combining mark) immediately after a token, which the end map absorbs into the token's span (pinned deliberate difference: keeps the grapheme cluster one styled run, agreeing with the primary arm since #268; found + probed at inspect). | Unit test: tab-free lines → span identity old-vs-new; tab-bearing lines → per-token covered display text equality; one pinned zero-width-absorption case asserting the NEW behavior. |
| REQ-003 | WHEN a phantom's own cells render on the fallback path, the system shall paint them `Hint` (hints-first ordering unchanged). | Existing #331 ordering tests stay green + one fallback-path assertion. |
| REQ-004 | The change shall pass the full diff gate at the §0 floors (fmt, clippy, tests, coverage 100 %, mutation MSI 100 % on touched lines). | `scripts/gates.sh --diff` exit code 0. |

## Phase Plan
- **P2 Design** — confirm the two call sites' exact rewrite, rule D5
  (header-site unify or leave), enumerate the test matrix (phantom-bleed,
  floor identity, tabs, empty-span edge), mutation surface.
- **P3 Implement** — apply the fallback rewrite per design.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN tests; `scripts/gates.sh --diff` green.
- **P5 Complete** — archive, ledger capture (§19), close the ticket (its
  BACKLOG row was removed at promotion).
