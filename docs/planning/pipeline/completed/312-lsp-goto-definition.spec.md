---
pipeline_id: ff05fa26-b99b-4395-97e2-e74c4761487d
ticket: forge#312 (ff827668-3ded-4e51-9e0b-465184afbcca) · local docs/planning/tickets/open/TICKET-312-lsp-goto-definition.md
aar_id: 19a888c7-9045-451e-a00b-155378a34939
status: Phase 5 — Complete PASS
title: LSP go-to-definition (F12) + a NavStack jump-back (⌃-)
type: feature
milestone: M20
references: [forge#312, TICKET-312, "#311 request/response path", "#309 position bridge", "#212 open_file_at", "#273 scroll_editor_to_row"]
---

## Title
The navigation that makes the editor feel like an IDE: press F12 on a symbol → land on its definition
(including in ANOTHER file), centered; press ⌃- → jump back. Multiple results (a trait with impls) open
a picker. Built on the #311 general request/response path (its second consumer) + the shipped
open-file/jump primitives.

## Scope
### In
- **`textDocument/definition` via F12** (editor-scoped) at the primary caret — sent through the #311
  path (`RequestPurpose::Definition`); the response consumed on the pump like hover.
- **`parse_definition_result` (PURE)** — normalize the THREE real response shapes (`Location` |
  `Location[]` | `LocationLink[]`) into a `Vec<DefLocation{uri, line, character}>`; empty/null → empty.
- **Single result → open + jump + center.** Reuse `open_file_at(path, line, col)` (#212) to open the
  target (another file if needed) + place the caret, then CENTER via `scroll_editor_to_row` — through a
  DEFERRED `pending_center_row` (the #273 trap: an open + same-frame scroll is wiped by
  `sync_editor_scroll`; center on the NEXT frame).
- **Multiple results → a picker** of `path:line` rows (a NEW `(label, DefLocation)` picker state
  mirroring `FinderState`'s move/selected idiom + an overlay block); Enter jumps to the chosen one.
- **The NavStack (PURE)** — `push(Loc) / pop() -> Option<Loc>`; a goto jump pushes the pre-jump
  location; **⌃-** pops + returns (open + center). Caps at 50; dedupes a same-spot re-push. Built
  push-ready for #304/#305 (⌘⇧O/⌃G) to feed later.
- **Cross-file target position** resolves via `position_to_offset` over the target buffer's CURRENT
  synced text (#309 keeps it synced).
- **Nothing found / host not Ready → a quiet `Flash`** (`flash.rs`), moving nothing.

### Out (explicitly deferred)
- **⌘-click-to-jump** — ⌘-click is the shipped multi-cursor gesture (#297/#298); reassigning it is a
  product decision (chad chose "F12-only, defer click" — multi-cursor untouched). A follow-up ticket
  owns the click gesture.
- **⌘⇧O go-to-symbol / ⌃G go-to-line as NavStack push sources** — those features aren't shipped
  (#304/#305 open); the NavStack is built push-ready but #312's only push site is the F12 goto.
- **Edit-surviving return positions via anchors** — the dormant `anchor.rs` layer; #312 uses
  `position_to_offset` over the synced buffer. A target surviving edits BETWEEN request and response is
  a follow-up.
- Peek/inline-preview of the definition; type-definition / implementation (separate requests).

## Reference (§20)
**Zed (the editor).** The behavior matched is the universal editor go-to-definition triad — F12 (or a
modifier-click) lands on a symbol's definition across files, ⌃-/the back chord returns, a multi-result
picker lists the candidates — reimplemented on the published **LSP 3.17 `textDocument/definition`** wire
+ Marley's own `open_file_at` / `scroll_editor_to_row` / a new pure `NavStack`. No Zed GPL source read
(§20 clean-room); the 3-shape response normalization is from the LSP spec.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — reuse the #311 general request/response path** (`RequestPurpose::Definition`, `request()`,
  `take_responses()`); `request_definition` + `consume_definition_responses` mirror hover exactly.
- **D2 — F12 is the v1 trigger; the click gesture is DEFERRED** (chad: "F12-only, defer click" — ⌘-click
  stays multi-cursor). A follow-up owns click-to-jump.
- **D3 — a DEFERRED center** (`pending_center_row` parked on the view, consumed a frame later after
  `sync_editor_scroll`) — the #273 open-then-scroll-in-one-frame-is-wiped trap; the two-frame pattern is
  proven at `headless_drive.rs:764`.
- **D4 — NavStack is a pure `Vec<(PathBuf, CharOffset)>`** (cap 50, dedupe same-spot); pushed by the
  #312 goto; push-ready for future jump sources.
- **D5 — a NEW `(label, DefLocation)` picker** (no generic list-of-location picker exists; the file +
  command pickers are hardwired) — mirror `FinderState` + the `app.rs:9623` overlay block.
- **D6 — cross-file targets resolve via `position_to_offset`** over the synced buffer (anchors dormant +
  optional).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN F12 is pressed in a focused editor tab whose LSP host is Ready, the system shall send a `textDocument/definition` request at the primary caret's position (via the #309 bridge + the #311 path). | pure params unit + driven |
| REQ-002 | WHEN a definition response arrives, `parse_definition_result` shall normalize `Location`, `Location[]`, and `LocationLink[]` into a `Vec<DefLocation>`; an empty/null result → an empty vec. | pure units (3 shapes + empty/malformed) |
| REQ-003 | WHEN exactly one location is returned, the system shall open its file (if not the current file), place the caret at its (line, character), and CENTER the view on that row. | driven + the deferred-center unit |
| REQ-004 | WHEN more than one location is returned, the system shall show a picker of `path:line` rows; selecting one shall jump to it. | pure picker-state units + driven |
| REQ-005 | WHEN a goto jump occurs, the pre-jump location shall be pushed to the NavStack; WHEN ⌃- is pressed, the system shall pop and return to the last pushed location. The stack caps at 50 and dedupes a same-spot re-push. | pure NavStack units + driven |
| REQ-006 | WHEN no definition is found OR the host is not Ready, the system shall show a quiet status flash and move the caret nowhere. | pure (empty→none) + driven |
| REQ-007 | WHEN the definition target is in another file, the caret shall land at the correct (line, character) mapped through `position_to_offset` over that file's synced text. | pure position unit + driven |

## Phase Plan
- **P2 Design** — `RequestPurpose::Definition` + the send/consume shims; `parse_definition_result` +
  `DefLocation` (marley_lsp); the `NavStack` + the picker state (marley_app pure); the deferred-center
  field + its pump consume; the F12 + ⌃- keymap rows; file manifest + ≥1-test-per-REQ plan.
- **P3 Implement** — the pure seams + the shim wiring; `cargo check` green.
- **P3.5 Inspect** — critics vs the diff (the response-shape normalization + the deferred-center race +
  the NavStack cap/dedupe are the risk seams).
- **P4 Validate** — pure suites + a headless drive (feed a synthetic definition response → assert the
  jump target + the NavStack push/pop); gate green.
- **P5 Complete** — CHANGELOG + crate-map, AAR, close + archive.
