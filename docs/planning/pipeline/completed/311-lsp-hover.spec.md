---
pipeline_id: e962427d-11a1-4f57-adfa-d0060b1bab3c
ticket: forge#311 (ac07a89b-b4d0-4b25-aa67-687988f575a2) · local docs/planning/tickets/open/TICKET-311-lsp-hover.md
aar_id: fedf1a48-84e2-4122-b959-c0efe8a7f56e
status: Phase 5 — Complete PASS
title: LSP hover — types + docs in an overlay card (⌘K / mouse dwell) with markdown-lite
type: feature
milestone: M20
references: [forge#311, TICKET-311, "#308 lsp wire", "#309 position bridge", "#310 diagnostics", "#221 overlay card", "#267 EditorFrameGeom"]
---

## Title
The first *interactive* LSP round-trip: "what is this?" at the caret without leaving the editor.
Press ⌘K (in an editor tab) or rest the mouse ~400ms over a symbol → send `textDocument/hover` → render
the type signature + docs in a rounded overlay card, code highlighted with the existing tree-sitter/lexer.
This ticket also builds the GENERAL request→response→consumer path on the LSP host (today only the
`initialize` response is handled) — the reusable wire #312 goto / #313 completions / #317 references need.

## Scope
### In
- **A general LSP request/response path on `LspHost`** — a `request(method, params)` sender (idgen +
  purpose-tagged pending + send) and a `on_message` Response arm that stashes the correlated result for
  the app to drain on the pump tick (mirroring how `drain()` already reports dirty). Built generically
  (a `Purpose`/result slot), not hover-specific.
- **`textDocument/hover`** at the primary caret (⌘K, editor-gated) and on a ~400ms mouse dwell over the
  editor text, position mapped through the #309 bridge (`offset_to_position` + `host.encoding()`).
- **`markdown_runs` (PURE)** — split the hover markdown into a small deliberate subset: fenced code
  blocks (with the fence language), inline code, headings, and paragraphs; everything else is plain
  text. Never panics on malformed markdown (an unclosed fence renders as plain).
- **The rounded overlay card render** — the #221 recipe (`surface`/`corner_radius`/`border`), anchored
  at the caret/pointer cell, window-clamped via the tested `context_menu::menu_origin`, with a
  max-height + internal scroll for long docs. Fenced code highlighted via `code_syntax::highlight_ranges`
  + `StyledText::with_highlights`.
- **`hover_dismiss` (PURE)** — a dismissal table: any edit, Esc, caret move, or scroll → dismiss; the
  mouse moving INTO the card → NOT dismiss (so long docs can be read/scrolled).
- **The stale-response guard (PURE)** — a hover request is keyed by (uri, position, buffer version); a
  response whose key no longer matches the live caret/buffer is DROPPED (no stale card).
- **Empty/None result → NOTHING renders** (no empty-card flash).
- **Diagnostic-on-hover** — re-add the pure `diagnostic_at_row` (removed in #310 inspect F3 as unwired)
  and surface the hovered span's (most-severe) `Diag.message` in the card — the #310-deferred half.

### Out (explicitly deferred)
- Full markdown (tables, images, nested/ordered lists, blockquotes) — only the lite subset ships.
- In-card actions (clickable links, "go to definition" buttons) — a later ticket.
- Signature help (`textDocument/signatureHelp`) — a different request, not hover.
- Multi-language tree-sitter for fenced code (Rust keeps tree-sitter; other langs use the hand lexer —
  the existing `code_syntax` split, unchanged).

## Reference (§20)
**Zed (the editor — same gpui stack).** The behavior matched is the standard editor hover: a framed
card near the caret/pointer showing the symbol's type signature + doc comment, with code rendered
monospace + highlighted, dismissed by an edit or a move-away. Matched by OBSERVING the behavior and the
published wire contract (**LSP 3.17 `textDocument/hover`** → `Hover { contents: MarkupContent }`), then
reimplementing on Marley's own #221 overlay recipe + `code_syntax` highlighter — never reading or
translating Zed's GPL source (§20 clean-room). The markdown-lite subset is Marley's own deliberate
reduction (rust-analyzer answers mostly fences + paragraphs + inline code).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — ⌘K is context-gated.** ⌘K is already `clear-screen` in the terminal (`keymap.rs:189`, TERM
  ctx). Resolve by context: an editor tab focused + a Ready host → hover; otherwise the existing
  clear-screen. No new global chord; the collision is resolved by the active surface (the editor
  already gates chords this way).
- **D2 — the request/response path is GENERAL, not hover-specific.** A purpose-tagged pending +
  a drained result slot, so #312/#313/#317 reuse the same sender + response arm. Hover is its first
  consumer.
- **D3 — markdown-lite is a SMALL subset** (fences, inline code, headings, paragraphs); everything else
  renders as plain text; malformed input never panics. Full markdown is Out.
- **D4 — fenced code reuses `code_syntax::highlight_ranges`** (the multi-lang hand lexer; Rust also has
  tree-sitter) — NO new highlighter. Unknown fence language → plain.
- **D5 — diagnostic-on-hover re-adds `diagnostic_at_row`** (pure, removed in #310 F3) and shows the
  hovered span's message — the #310-deferred half lands here where it is wired.
- **D6 — the dwell detector is a pump-tick counter** (the #203 `notify_ticks` idiom): a RootView
  `last_hover_cell` + `hover_ticks`, reset on mouse-move, fires at ~400ms (≈25 × the 16ms
  `PUMP_INTERVAL_MS`). No new timer.
- **D7 — the stale-guard keys by (uri, position, buffer version)**; a response not matching the live
  caret + version is dropped (a pure compare) — the classic moved-on-since-request drop.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘K is pressed while an editor tab is focused and its LSP host is Ready, the system shall send a `textDocument/hover` request at the primary caret's position, mapped through the #309 encoding bridge. | pure hover-params unit (offset→position) + driven |
| REQ-002 | WHEN the mouse pointer rests within the editor text at one cell for ≥ the dwell threshold (~400ms) without moving, the system shall send a `textDocument/hover` request at that position. | pure dwell-threshold unit (ticks→fire, reset-on-move) + driven |
| REQ-003 | WHEN a hover markdown result is rendered, `markdown_runs` shall split it into fenced-code (with language), inline-code, heading, and paragraph runs; a known fence language shall be highlighted; malformed markdown (e.g. an unclosed fence) shall render as plain text without panicking. | pure units (fence+lang / fence-no-lang / inline / heading / para / unclosed fence) |
| REQ-004 | WHEN an edit, Esc, caret move, or scroll occurs with the card open, `hover_dismiss` shall return dismiss=true; WHEN the mouse moves INTO the card, it shall return dismiss=false. | pure `hover_dismiss` table + driven |
| REQ-005 | WHEN a hover response arrives whose key (uri, position, buffer version) no longer matches the live caret/buffer, the system shall DROP it (no card). | pure stale-key compare unit + driven |
| REQ-006 | WHEN the hover result is empty or None, the system shall render no card (no empty flash). | pure (empty markdown → no runs; render gate) + driven |
| REQ-007 | WHEN the hover card would overflow the window at its anchor, the system shall clamp its origin (via `menu_origin`) so it stays fully visible. | pure `menu_origin` (already tested) + driven |
| REQ-008 | WHEN the hover position lies on a diagnostic span, the system shall include that span's most-severe `Diag.message` in the card, via `diagnostic_at_row`. | pure `diagnostic_at_row` unit + driven |

## Phase Plan
- **P2 Design** — the `Purpose`-tagged request/response path (rpc.rs `Pending` gains a purpose OR the
  host keeps a side map; decide + document), the hover request/response/consumer flow + stale-key, the
  `markdown_runs` grammar + `HoverRun` type, `hover_dismiss` table, the dwell field/handler, the card
  render + anchor/clamp, `diagnostic_at_row` re-add. File manifest + the ≥1-test-per-REQ plan.
- **P3 Implement** — the pure seams (`markdown_runs`, `hover_dismiss`, stale-key, dwell-threshold,
  `diagnostic_at_row`) + the shim wiring (host request/response, the dwell detector, the ⌘K gate, the
  card render). `cargo check` green.
- **P3.5 Inspect** — independent critics vs the diff (the new request/response correlation is the
  highest-risk seam — id reuse, stale-drop, response-to-wrong-consumer); fix the real findings.
- **P4 Validate** — write + RUN the tests; a headless drive of the hover flow (feed a synthetic hover
  response through the request/response path, assert the card content) + the pure suites; gate green.
- **P5 Complete** — CHANGELOG + crate-map, AAR capture, close + archive.
