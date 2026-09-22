---
pipeline_id: 4c3a1e3e-4d31-455d-ace5-5cf3897add1f
ticket: forge#340 (b35af9e6-04c1-47a6-91af-f1614a2581fe) · local docs/planning/tickets/open/TICKET-340-bracket-match.md
aar_id: d6b81d92-45c1-4f2c-bf64-9473de807e98
status: Phase 5 — Complete PASS
title: Bracket-match highlight — the caret's delimiter pair, lit; ⌘⇧\ jumps between them
type: feature
milestone: M22
references: [the #329 marley_syntax node-range API (enclosing_ranges — the shape to mirror), the #330 per-(nonce,version) tree cache, styled_slices_with_marks + MarkTier (code_view.rs:352/:331), the #331 drop-guard lesson]
---

## Title
Put the caret on a `(` and nothing tells you where it closes. This lights BOTH delimiters of the pair the
caret touches — the quiet affordance every editor has — plus **⌘⇧\ Go to Bracket** (the chord is free,
verified). Rust-only v1, because the truth source is the tree-sitter tree we already cache per
(nonce, version): no second text-scanning engine, no string/comment false positives, nothing to un-ship
when #315 lands languages.

## Scope
### In
- **The third `marley_syntax` node API** (mirroring #329's `enclosing_ranges` / #330's `all_headers` shape —
  a plain iterative walk in lib.rs, cov/MSI 100):
  `matching_delimiters_in(src: &str, byte_pos) -> Option<(Range<usize>, Range<usize>)>` — **AMENDED at Phase 2:
  a free fn that PARSES `src` itself (parse-only), NOT `&HighlightSession`.** The app has no cached session,
  and a session drags in the `highlight_full` query cost bracket-match does not need; `_in` is parse-only
  (~4ms/2000 lines measured, vs ~2× for highlight_full). The session-taking variant returns with the
  cached-tree follow-up **#349**. The walk: the smallest node containing `byte_pos` whose FIRST and LAST
  children are a delimiter pair from `{() [] {}}` (matched by anonymous-node `kind()` = `"("` etc — spike-
  confirmed) yields those two children's byte ranges. **PROVEN at Phase 2 (spike):** a `(` inside a string is
  a `string_content` node, inside a comment a `line_comment` — `descendant_for_byte_range` lands inside that
  leaf, so `matching_delimiters_in` returns None; the false-positive class genuinely cannot exist.
- **The adjacency rule (pinned)** — the probe position prefers the char BEFORE the caret when it is a
  delimiter, else the char AT the caret (matches the observed feel: type `)` and the pair lights while the
  caret sits after it). Primary cursor only v1 (documented; N-cursor highlight is noise until asked for).
- **Render as a MARK, not a token** — a new `MarkTier` variant (code_view.rs:331) through the SHIPPED
  `styled_slices_with_marks` marks channel (:352-356), subtle background tint on exactly the two delimiter
  chars' display bytes (via `raw_span_to_display_bytes` — the #331 code-side end map, so an inlay hint
  anchored at a delimiter can never smear the tint). The marks channel is CORRECT here for the same reason
  `TokenKind::Hint` was required in #331, inverted: a mark RETAINS a Plain+unselected slice, and it composes
  with selection + find bands instead of fighting the syntax layer.
- **Recompute per caret move** — ~~on the CACHED tree (the #330 cadence: one reparse per EDIT, never per caret
  move or frame) — a caret move is one bounded walk from the cached root~~ **AMENDED at Phase 1 (F1): THERE IS
  NO CACHED TREE app-side.** The live tree lives only on the syntax worker thread; the #329/#330 node-range
  callers reparse a THROWAWAY `HighlightSession` (a full whole-buffer `highlight_full`) per call. Following
  that route, a caret move = a full-file reparse (the ladder does exactly this on ⌥↑, but bracket-match fires
  on every arrow). **Design Fork A:** accept the #329-style per-caret reparse (bounded/measured), or build an
  app-side cached tree. No tree ⇒ no highlight, no cost — that half holds.
- **⌘⇧\ Go to Bracket** — caret on/inside a pair → jump to the matching delimiter (on the opener → the
  closer, and back; inside the pair → the closer, VS Code-observed). Editor-scoped, chord verified free.
  No NavStack push (an intra-expression hop, not a navigation — the #330 lesson consciously decided, not
  forgotten: siblings that JUMP FILES push; this never leaves the line-neighborhood).
### Out (explicitly)
- Non-Rust files (no tree = no truth; the hand-lexer fallback would reintroduce the string/comment
  false-positive class — #315 inherits this for free per grammar). Unmatched-bracket error tint (tree-sitter
  ERROR nodes make "unmatched" ill-defined). Rainbow/pair-depth colorization. `<>` (ambiguous with
  comparison operators at the token level; a type-context refinement later). Auto-select-to-bracket.

## Reference (§20)
**tree-sitter = published-API reuse (MIT)**; VS Code / Zed = OBSERVED behavior (adjacency preference, the
⌘⇧\ jump, highlight-not-select). The walk and the mark plumbing are Marley-original over shipped seams.

### Prior art
*(The required sweep — run at Phase 1 promotion. §20's wall is unchanged; reading our own permissive deps is
adoption, outside it.)*

1. **Behavior maps / observed** — VS Code and Zed both light the caret's delimiter pair and jump on a chord;
   the observed details taken: the adjacency preference (the char BEFORE the caret when it is a delimiter),
   highlight-not-select, and jump-to-match (opener→closer, inside→closer). Behavior only; no copyleft source.
2. **Published material** — tree-sitter's node/cursor API is documented; the "smallest node at a byte, inspect
   its first/last child" shape is the standard tree-query idiom.
3. **OUR PERMISSIVE DEPS — and the sweep paid.**
   - **`tree-sitter` 0.26.11 OWNS the node navigation** — `Node::child`/`child_count`/
     `descendant_for_byte_range`/`parent` (binding_rust/lib.rs:1713-1972) — but has **NO** delimiter/bracket
     concept (grep for matching/bracket/delimiter → nothing). So `matching_delimiters` is a **thin adapter
     over the crate's node primitives**, not a hand-rolled tree walk — the #339 pattern (build on what the
     crate owns). The pure fn shrinks to: `descendant_for_byte_range(pos, pos)`, climb to the smallest node
     whose FIRST and LAST children are a delimiter pair, return their two byte ranges.
   - **`marley_syntax` already has the walk shape** — `enclosing_ranges` (#329) and `all_headers` (#330) are
     iterative in-crate walks over the private tree. `matching_delimiters` is a THIRD in the same file, same
     access pattern. **But NOT a literal copy** (F2): those use `named_descendant_for_byte_range` (NAMED
     nodes); delimiters are ANONYMOUS leaves, reached by the non-named `descendant_for_byte_range` +
     `child(i)`. The tree field is PRIVATE, so this MUST live in `marley_syntax`, not the app.
   - **Checked and found the cadence NOT owned** (F1): there is no cached tree on the app side — the
     `(nonce,version)` caches store computed spans, and the #329/#330 route reparses a throwaway session. So
     the recompute cadence is genuinely ours to design (Fork A), not a seam to reuse. Recording the miss is
     the point of the sweep — it separates what tree-sitter already gives (node nav) from what we must build
     (the per-caret cadence).

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-TREE-IS-TRUTH** — Rust-only via the cached HighlightSession; no text-scanner second engine.
- **D-PAIR-SET** — `{() [] {}}` v1; `<>` named out.
- **D-MARK-NOT-TOKEN** — a MarkTier through the marks channel; the two spans map through
  `raw_span_to_display_bytes` (phantom-safe by #331's construction). **NOTE (F4):** that fn is the CODE-SPAN
  mapper; today's marks (find bands) use the cols path instead. Using it here is a deliberate choice — a
  delimiter is a raw byte span like a syntax span — not existing precedent. Confirmed defensible, Design pins
  it. Adding a `MarkTier` variant touches 2 exhaustive consumers (code_view.rs:410, app.rs:4690).
- **D-BEFORE-THEN-AT** — the adjacency preference, pinned in the pure table.
- **D-PRIMARY-ONLY** — one pair lit, the primary caret's; recompute keyed on (nonce, version, caret).
- **D-NO-NAVSTACK** — ⌘⇧\ does not push (intra-expression; documented against the #330 sibling rule).

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | return the innermost pair for a caret inside nested pairs (`f(g(x))` — caret in `g(…)` lights g's parens, not f's) | pure table |
| REQ-002 | prefer the delimiter BEFORE the caret over the one AT it when both touch | pure table |
| REQ-003 | light a multi-line pair (an fn body's `{ … }`) on both rows | headless |
| REQ-004 | NOT match a delimiter character inside a string or comment (`"("`, `// (` — no node, no match) | pure (real tree) |
| REQ-005 | render the two delimiters as a mark tier that composes with selection + find bands (a selected delimiter shows both) | pure slices table |
| REQ-006 | jump to the matching delimiter on ⌘⇧\ (opener→closer, closer→opener, inside→closer) without pushing the NavStack | headless |
| REQ-007 | recompute only on caret move / version bump, reading the CACHED tree (no reparse per frame) | review + the #330 cache test shape |
| REQ-008 | render nothing (and do no walk) when no tree exists (non-Rust / fresh session) | pure + headless |

## Phase Plan
P2 confirm the MarkTier variants + where find bands feed marks (the exact call site) + the caret-move
recompute hook (#330's refresh shape); P3 the marley_syntax walk first (its truth table on real parses —
including the string/comment negative rows), then the mark feed + the chord; P3.5 critics on the walk's
edge rows (caret at byte 0, EOF, a delimiter as the FIRST char of the file, an ERROR-node region) + the
adjacency table + the no-reparse cadence; P4 tables + headless drives + gate (marley_syntax lib.rs extends —
no new file expected); P5 docs (editor.md + crate-map's marley_syntax row gains the third API).
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
