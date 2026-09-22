---
pipeline_id: d4fe3056-adaa-4d70-978e-02ff9a8f41a9
ticket: forge#331 (38801e5a-b739-43a0-ae71-a5060c8be6b1) · local docs/planning/tickets/open/TICKET-331-inlay-hints.md
aar_id: 040d8d7f-9dc3-4370-a6d9-34ffb80fd975
status: Phase 5 — Complete PASS
title: LSP inlay hints — inline type + parameter annotations (the display-map ticket)
type: feature
milestone: M21
references: [forge#331, forge#311, forge#309, forge#313, forge#310, forge#330, forge#316]
---

## Title
`let s = String::new()` renders as `let s: String = String::new()` — the muted `: String` is the SERVER's, not
the file's. The most visible "real IDE" signal rust-analyzer offers, and THE render-model ticket of the batch
(deliberately last): the editor's buffer→display mapping must learn about phantom text that occupies columns
but belongs to no buffer char.

## Scope
### In
- **The phantom-aware layout (PURE — the heart, the whole risk in ONE seam)** —
  `line_layout_with_inlays(line, tab_width, inlays: &[(char_idx, &str)]) -> LineLayout`: the existing
  single-pass walk, but before emitting the `i`-th char it emits any phantom anchored at `i` into `display`
  and advances `col` **without pushing a `col_starts` entry**. `line_layout(l, w)` becomes the empty-slice
  wrapper.
- **The contract (free, by construction — see the plan discovery)** — `col_of_offset(i)` returns the column
  AFTER any phantom anchored at `i` (the caret sits on the CODE side of a hint; typing at a hint boundary
  types into CODE). `offset_of_col_f` scans only buffer-char boundaries, so a click INSIDE a phantom snaps to
  the nearest anchor (a hint is never selectable, never a caret home). Selection rects (#255) and #310
  squiggle spans ride the same `col_starts` and shift correctly for free.
- **The byte-identity property** — `line_layout_with_inlays(l, w, &[])` ≡ today's `line_layout(l, w)`,
  byte-for-byte (a property test pins it), so hints OFF is the EXACT pre-ticket path.
- **The parse (PURE, marley_lsp)** — `InlayHint { position, label: String | InlayHintLabelPart[], kind (1=Type,
  2=Parameter), paddingLeft/paddingRight }`; label parts CONCATENATED v1; a per-element `filter_map` (one bad
  hint cannot lose the line); positions resolved through the #309 encoding bridge (a hint after an emoji must
  not shift — the F12 pin, reused).
- **The request loop** — `textDocument/inlayHint { range: the viewport ± a page }` via the #311 recipe
  (`RequestPurpose::InlayHints` + the ONE drain), re-requested debounced on version bump + on scrolling into
  unfetched rows, range-keyed stale guard (the #313 family). A `workspace/inlayHint/refresh` **ServerRequest
  arm** → re-fetch + reply null (`Incoming::ServerRequest` is ALREADY routed — a new arm, not new plumbing;
  this is the wire's first server-initiated request beyond the #308 set).
- **The render** — hints as MUTED runs inside the row's styled runs (a muted-tone token via the syntax
  palette so #316 themes it); `paddingLeft`/`paddingRight` as single spaces. `inlayHint/resolve` is NOT needed
  v1 (rust-analyzer sends type/param labels eagerly).
- **The setting + command** — `editor.inlay_hints` (default on; per-kind toggles at design's discretion) + a
  palette toggle, mirroring the #330 `editor.sticky_header` precedent (setting + `AppliedSettings` + a
  NON-DEFAULT round-trip leg + `CommandId(15)` + the three-site trio).

### Out (explicitly deferred)
- **Interaction** (click-a-hint-to-insert-it), **label-part tooltips/locations**, **`inlayHint/resolve`**,
  hint-length caps beyond the server's, and any hint kind beyond Type/Parameter.

## Reference (§20)
**Clean-room from the PUBLISHED LSP 3.17 specification** — `textDocument/inlayHint`,
`workspace/inlayHint/refresh`, `InlayHint`/`InlayHintLabelPart`/`InlayHintKind`, `paddingLeft`/`paddingRight`
— exactly the posture of #308–#313 (the wire is implemented from the spec document, never from a client's
source). **Zed = ARCHITECTURE reference only**: the deconstruction (03 §3) names `InlayMap` and its
"empty input, non-empty output" layer-contract CONCEPT, and flags its 209KB machinery as "defer wholesale" —
Marley's per-line render admits a far smaller shape (one pure function), so the concept informs, the source is
UNREAD. **VS Code = OBSERVED behavior** for the caret/click contract (the caret lands on the code side of a
hint; a hint is not selectable). No copyleft source read or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D-SHAPE → extend `line_layout` in place** (same `LineLayout`; a phantom emits into `display` and advances
  `col` BEFORE the anchor char's `col_starts.push(col)`, so it consumes columns but adds no entry), with a
  typed `Inlay { char_idx, text }` and `line_layout(l, w) = line_layout_with_inlays(l, w, &[])`. **The design
  NAMED the invariant this rests on:** `display`'s cell-accumulation and `col_starts` must stay ONE column
  domain (`cols_to_bytes` re-derives columns by walking `display`; `col_of_offset` reads them from
  `col_starts` — they agree only because one pass builds both). Advancing `col` before the push keeps
  `col_starts` in SCREEN columns, so every rider (caret pixels, `cols_to_bytes`/`raw_span_to_display_bytes`,
  `offset_of_col_f`, selection rects, #310 squiggles) is correct with NO signature change. The TRAP — leaving
  `col_starts` phantom-blind — forks the domains and silently corrupts every span/pixel; an invariant test
  pins it.
- **D-HINT-TOKEN → a new non-`Plain` `TokenKind::Hint` (REQUIRED, not cosmetic)** — `styled_slices_with_marks`
  DROPS any slice that is `Plain` + unselected + unmarked, so an untagged phantom would render in the base
  `foreground`, indistinguishable from real code. `LineLayout` exposes the phantoms' display-byte
  `hint_spans`; the render feeds them in as `Hint` → `token_color(Hint) = colors.muted`, with zero change to
  `styled_slices_with_marks`.
- **D-CONTRACT → caret on the CODE side; a click inside a phantom snaps to the nearest anchor** — both fall
  out of the construction (emit-phantom-then-push-col; `offset_of_col_f` scans buffer boundaries only). The
  truth table pins them.
- **D-OFF-IS-IDENTICAL → the empty-slice property test** — hints OFF must be byte-identical to the pre-ticket
  layout.
- **D-REQUEST → the #311 recipe + a range-keyed stale guard + a `workspace/inlayHint/refresh` arm** (reply
  null, re-fetch).
- **D-PARSE → both label shapes, parts concatenated, per-element filter_map, #309 encoding bridge.**
- **D-SETTING → `editor.inlay_hints` + a palette toggle** (the #330 four-wiring pattern incl. the NON-DEFAULT
  round-trip leg).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `line_layout_with_inlays` shall place each phantom's text into `display` at its anchor and advance the column WITHOUT adding a `col_starts` entry — so `col_starts` keeps mapping the i-th BUFFER char (`len == n_chars + 1`). | pure unit (truth table) cov/MSI 100 |
| REQ-002 | With an EMPTY inlay slice the result shall be byte-identical to today's `line_layout` (display AND every column). | pure property test |
| REQ-003 | `col_of_offset(i)` shall return the column AFTER a phantom anchored at `i` (the caret sits on the code side). | pure unit |
| REQ-004 | `offset_of_col_f` shall map a click landing INSIDE a phantom run to an anchoring buffer char (never "inside" the hint). | pure unit (both halves) |
| REQ-005 | The layout shall be correct for a hint at BOL, mid-line, EOL, two ADJACENT hints, a hint next to a TAB, and a hint next to a wide glyph. | pure unit (the truth table) |
| REQ-006 | `parse_inlay_hints` shall normalize BOTH label shapes (a bare string and `InlayHintLabelPart[]`, parts concatenated), carry kind + padding, skip a malformed element without losing the rest, and resolve positions through the negotiated encoding (a hint after an emoji lands on the right char). | pure unit cov/MSI 100 |
| REQ-007 | The editor shall request `textDocument/inlayHint` for the viewport range, re-request on a version bump / scroll into unfetched rows, and DROP a reply whose range/version is stale. | unit (guard) + headless |
| REQ-008 | WHEN the server sends `workspace/inlayHint/refresh`, the editor shall re-fetch and reply null. | headless/integration |
| REQ-009 | Hints shall render as muted runs with padding honored, inside the row's styled runs. | headless/review + LIVE(fallback) |
| REQ-010 | WHILE `editor.inlay_hints` is false, no hint shall render and the layout shall be the pre-ticket path; the palette toggle shall flip and persist it (round-trip). | unit (setting) + headless |

## Phase Plan
- **P2 Design** — CONFIRM D-SHAPE (+ the exact signature/type), the truth table, the request cadence + guard
  key, the refresh arm, the render run + the muted token, the setting/toggle wiring; the file manifest; the
  mutation surface. §20 confirm.
- **P3 Implement** — the pure layout + parse; the request/refresh wiring; the render; the setting + toggle.
- **P3.5 Inspect** — critics vs the diff; the phantom column math (the boundaries), the empty-slice identity,
  the encoding bridge, the stale guard, the refresh arm, and the riders (caret/click/selection/squiggle) get
  the hardest look.
- **P4 Validate** — pure units cov/MSI 100 (the truth table + the property + parse) + headless drives
  (request/refresh/caret/click/toggle) + the gate; live pixel drive env-blocked → units+mechanism.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md; AAR; archive; close #331 (and the M21 batch).
