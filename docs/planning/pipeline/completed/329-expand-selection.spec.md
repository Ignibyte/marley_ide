---
pipeline_id: 9aadc7ad-a646-4c3c-ae8d-e68cbab0171f
ticket: forge#329 (78e4ae26-a46d-4fdf-815e-229d738faede) · local docs/planning/tickets/open/TICKET-329-expand-selection.md
aar_id: 4cb6a9a2-12a7-4dd8-8259-c5446654dd4f
status: Phase 5 — Complete PASS
title: Expand/shrink selection — tree-sitter node ancestry + marley_syntax's first node-range API
type: feature
milestone: M21
references: [forge#329, forge#313, forge#274, forge#309, forge#300, forge#305, forge#330]
---

## Title
Grow the selection identifier → call → statement → block → fn with one chord; shrink walks back down to the
ORIGINAL selection. The heart is a pure tree-sitter ancestry walk — but no tree is reachable today
(marley_syntax exposes highlight SPANS only; the `tree_sitter::Tree` is a private field of `HighlightSession`
on the worker thread). So this ticket is also the FOUNDATION: marley_syntax's first node-range API,
`enclosing_ranges`, that #305 folding and #330 sticky header will reuse.

## Scope
### In
- **`enclosing_ranges(&HighlightSession, byte_range) -> Vec<std::ops::Range<usize>>` (marley_syntax, the pure
  foundation)** — the ancestry ladder: the smallest NAMED node strictly containing `byte_range`, then each
  named ancestor up to the root, as BYTE ranges, DEDUPED (a parent whose range equals its child's collapses —
  tree-sitter nests identical-span nodes constantly). Pure over the session's held `tree` (reads the private
  field, no text needed — nodes carry byte offsets). No tree (a fresh session) → empty ladder. `parse.rs`
  stays the FFI-shim coverage exclude; `enclosing_ranges` gets the pure treatment (cov/MSI 100 over fixture
  parses).
- **`SelectionLadder` (PURE, marley_app)** — `{ anchor: Range<usize>, rungs: Vec<Range<usize>>, pos: usize }`
  in char offsets. `grow` advances `pos` toward the root (clamped at the top); `shrink` retreats toward the
  ORIGINAL selection — `pos == 0` yields the remembered `anchor` (what makes shrink correct, vs re-shrinking
  to the smallest node). The rung list is computed ONCE per ladder session (the first grow from an
  empty/invalidated state); subsequent grow/shrink only move `pos` (no re-parse). Invalidated by any edit or a
  caret move OUTSIDE the current rung (version + caret identity — the live-identity family).
- **The tree-source delivery (app shim)** — the FIRST grow needs a parsed tree at the current buffer text:
  - **Large file (`> SYNTAX_SYNC_MAX_LINES`)**: the worker already holds a live `HighlightSession` at
    `(nonce, version)`. Extend the worker protocol with a ladder query (a `SyntaxReq`/`SyntaxResp` addition)
    and PARK the keystroke one tick (the #313 consume precedent), applying the selection when the reply lands.
  - **Small file (`≤ SYNTAX_SYNC_MAX_LINES`)**: there is NO retained tree (the sync path calls the stateless
    `highlight_lines`, which discards its tree). Build a THROWAWAY `HighlightSession` synchronously
    (`highlight_full(text)` then `enclosing_ranges`) — one parse of ≤1000 lines on a chord press, the same
    cost class as the existing per-keystroke sync highlight, discarded after (NOT a persistent second session,
    so the #313 D2 drift concern does not apply). No park.
- **Byte↔char bridge** — tree-sitter speaks BYTES, `Selection`/`SelectionSet` speak CHAR offsets. Convert the
  caret/selection char range → bytes before `enclosing_ranges`, and each returned byte range → chars before it
  becomes a `Selection` (the #309 position-bridge discipline, in-file via the rope). An EMOJI-bearing fixture
  is a MANDATORY test (a naive byte==char would land mid-grapheme).
- **The chord (Editor-scoped)** — grow + shrink, `KeyContext::Editor`. Leading candidate **⌃W (grow) / ⌃⇧W
  (shrink)** (JetBrains "extend/shrink selection"; audited FREE — `cmd-shift-e` is taken by #68 Fleet, `⌥↑/⌥↓`
  collide with #300 move-line, `⌥⇧↑/↓` are #300 duplicate). Design confirms against `keymap.rs` +
  `chords_unique_scoped`.
- **Multi-cursor v1** — the PRIMARY cursor grows/shrinks; the others collapse to it (the #307 Tab precedent).

### Out (explicitly deferred)
- **N-cursor ladders** (each cursor its own ladder) — the named follow-up.
- **`.scm`-driven "smart" ranges** (argument-lists, string-interiors), **node-kind display** (a status hint
  naming the current rung), **non-Rust languages** (inherited automatically when #315 lands grammars — the API
  is grammar-agnostic).
- The `#305` fold + `#330` sticky-header CONSUMERS of `enclosing_ranges` (separate tickets; this ships the API).

## Reference (§20)
**Zed (the editor) — BEHAVIOR reference only; tree-sitter — published-API REUSE (MIT, permissive).** Expand/
shrink-selection by AST ancestry is a universal editor affordance (JetBrains, VS Code, Zed all ship it). The
`docs/zed_architecture/04-language-syntax-treesitter.md` §6.8 deconstruction ("the cheapest high-value feature
once a tree exists — a `TreeCursor` ancestry walk, no `.scm` query") is a behavior/shape reference; Zed's
`syntax_ancestor`/GPL source is UNREAD. The walk uses the PUBLISHED `tree_sitter` cursor API (`goto_*`,
`node.parent()`, byte ranges) — permissive reuse, no copyleft translation. Clean-room §20 holds.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D-ROUTE → sync throwaway session for ALL sizes (v1)** (DESIGN refined the plan's hybrid): the Explore pass
  established the `tree_sitter::Tree` never crosses the worker→app boundary (the cache carries only per-line
  spans) — so BOTH tiers need an on-demand tree. v1 generalizes the small-file arm: a THROWAWAY
  `HighlightSession` (`highlight_full` over the current text, discarded) computes the ladder ONCE per gesture
  (first grow), cached thereafter. Chosen for minimal always-correct surface (no `SyntaxReq`/`SyntaxResp`
  extension, no park/consume, no async stale-drop) and same-cost-class for the ≤1000-line common case. Route B
  (a PERSISTENT second session) stays REJECTED (#313 D2). The worker-async round-trip for >1000-line files (the
  `(nonce,version)`-guarded channel) is a NAMED follow-up — reversible; `enclosing_ranges` is identical either
  way.
- **D-LADDER → compute-once, navigate-`pos`** — the rung list is built on the first grow and cached in
  `SelectionLadder`; grow/shrink move `pos` only. `shrink` to `pos == 0` restores the remembered `anchor`.
- **D-INVALIDATE → version + caret identity** — any edit (version change) or a caret move outside the current
  rung drops the ladder; the next grow recomputes (the live-identity family, per #327's selection-identity
  lesson).
- **D-BYTES → char↔byte at the app boundary** — `enclosing_ranges` is byte-in/byte-out (tree-native); the app
  converts through the rope. Emoji fixture mandatory.
- **D-CHORD → ⌃W / ⌃⇧W, Editor-scoped** (design free-checks; `cmd-shift-e` taken, `⌥↑↓` collide).
- **D-MULTICURSOR → primary grows, others collapse** (v1; N-ladder deferred).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `enclosing_ranges(&session, byte_range)` shall return the ancestry ladder of NAMED nodes strictly containing the range, innermost-first up to the root, as byte ranges. | pure unit cov/MSI 100 (fixture: identifier→call→statement→block→fn on real Rust) |
| REQ-002 | The ladder shall DEDUPE adjacent identical ranges (a parent whose byte range equals its child's appears once). | pure unit |
| REQ-003 | A byte range already spanning the root shall yield a ladder that tops out at the root (no duplicate root, no panic); a session with no tree shall yield an empty ladder. | pure unit |
| REQ-004 | An EMPTY selection (caret, `start == end`) shall ladder from the smallest named node containing that point. | pure unit |
| REQ-005 | The char↔byte conversion shall be exact across a multi-byte (emoji) line — every rung maps to a char-boundary `Selection`, never mid-grapheme. | pure unit (emoji fixture) + headless |
| REQ-006 | WHEN the first `grow` fires from an empty/invalidated ladder, the system shall compute the rung list once and select the innermost enclosing rung; a subsequent `grow` shall advance to the next rung without re-parsing. | unit (SelectionLadder) + headless |
| REQ-007 | `shrink` shall retreat one rung toward the ORIGINAL selection, and at the bottom restore EXACTLY the remembered anchor range (not the smallest node). | unit (SelectionLadder) |
| REQ-008 | WHILE a ladder is active, any edit or a caret move outside the current rung shall invalidate it, so the next `grow` recomputes from the new position. | unit + headless |
| REQ-009 | WITH multiple cursors, `grow`/`shrink` shall act on the PRIMARY cursor and collapse the others to it (v1). | unit/review |
| REQ-010 | The Editor-scoped chord shall drive grow/shrink — a large-file first grow via the worker round-trip (parked one tick), a small-file first grow synchronously — landing a visible selection. | headless + review (live env-blocked) |

## Phase Plan
- **P2 Design** — CONFIRM D-ROUTE (the hybrid; the exact `SyntaxReq`/`SyntaxResp` ladder-query shape + the
  park/consume wiring), D-CHORD (free-check ⌃W/⌃⇧W vs keymap.rs). The `enclosing_ranges` cursor-walk algorithm
  (named-node filter, strictly-containing test, dedupe), the `SelectionLadder` state machine, the char↔byte
  bridge, the file manifest, the mutation surface. §20 confirm.
- **P3 Implement** — `enclosing_ranges` in marley_syntax; `SelectionLadder` + the app shim (worker protocol
  extension, the small-file sync path, the chord dispatch, byte↔char); every new pure fn a direct unit.
- **P3.5 Inspect** — critics vs the diff; the cursor-walk (named-only, strict-containment, dedupe, no-tree),
  the ladder state (shrink-to-anchor, invalidation identity), the byte↔char (emoji), the worker
  park/consume race, the render/selection application get the hardest look.
- **P4 Validate** — pure units cov/MSI 100 (the ladder over a fixture parse; dedupe; root; empty selection;
  byte↔char emoji; SelectionLadder grow/shrink/invalidate table) + headless drives (chord→park→worker→grow 3
  rungs; shrink returns to original; edit mid-ladder invalidates) + the gate; live drive env-blocked (screen
  locked) → units+mechanism fallback.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md (marley_syntax gains its first node-range API); AAR;
  archive; close #329.
