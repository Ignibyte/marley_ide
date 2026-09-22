---
pipeline_id: affc5cc8-f050-447b-99cc-41a6b4783be9
ticket: forge#313 (ad15e168-b72d-4a46-a64a-90f9b858408d) · local docs/planning/tickets/open/TICKET-313-lsp-completions.md
aar_id: 989649bc-21e1-47a0-8624-12a360db0ef1
status: Phase 4 — Validate INCOMPLETE (gate 14/15; coverage RED at 1 line; live drive not run)
title: LSP completions — the as-you-type popup with fuzzy ranking + a TextEdit-faithful accept
type: feature
milestone: M20
references: [forge#313, TICKET-313, "#311 request path", "#312 lsp_position_for", "#96 complete.rs popup idiom", "#296/#299 edit+undo", "marley_search_core::fuzzy_rank"]
---

## Title
Type in the editor and the language server offers what comes next: a popup under the caret, fuzzy-ranked
by what you have typed, that on Enter **replaces the prefix rather than appending after it**. The third
consumer of the #311 request path — only the payload and the accept are per-feature. The most STATEFUL
LSP feature in M20: it fires per keystroke, so its stale-guard, its suppression rules, and its undo
bracketing are the design, not the trimming.

## Scope
### In
- **`textDocument/completion` on a trigger** — an identifier char or one of the server's advertised
  `triggerCharacters` (`.`, `::`) sends a request at the caret through the #311 path
  (`RequestPurpose::Completion`), reusing #312's `lsp_position_for` + `text_document_position_params`.
- **`parse_completion_result` (PURE)** — hand-parsed per the crate's house rule (see D3): a
  `CompletionList {isIncomplete, items}` OR a bare `CompletionItem[]` (both shapes are real) →
  `Vec<CompletionItem{label, sort_text, filter_text, kind, detail, edit}>`, where `edit` normalizes
  `textEdit` (a `TextEdit{range,newText}` or an `InsertReplaceEdit{insert,replace,newText}`) and the
  legacy `insertText`/label-only fallback into ONE Marley-local value. Malformed elements SKIPPED.
- **Fuzzy filter + rank (PURE)** — over `filter_text ?? label` via **`marley_search_core::fuzzy_rank`**
  (the finder's matcher — one matcher, everywhere), then stable-sorted by (score desc, `sort_text`,
  `label`). An empty prefix keeps server order. The rendered list is capped and windowed via the shipped
  `complete::popup_window`.
- **The popup** — anchored one row UNDER the caret (the #311 `hover_card_overlay` cell→pixel recipe),
  **flipping ABOVE when it would not fit** (net-new pure `popup_origin`; `menu_origin` only clamps).
  ↑/↓ move, Enter/Tab accept, Esc closes. Printables FALL THROUGH and re-filter (the #96 idiom).
- **Accept = the item's TextEdit, applied faithfully** — typing `s.pu` then accepting `push` REPLACES
  `pu`. Snippet-format items insert their PLAIN text with `$0`/`${1:x}` markers stripped
  (`strip_snippet_markers`, PURE). The accept is **its own undo unit that the next typed char cannot
  absorb** (D4 — this is a real hazard, not ceremony; see the note).
- **Stale guard** — the key carries **uri + position + BufferVersion** (like `HoverKey`, unlike
  `DefinitionKey`): a late answer for an older keystroke or an edited buffer is DROPPED, and the
  live-editor identity is re-checked exactly as `apply_definition_response` does (#312 F4).
- **Suppression** — no popup inside a STRING or COMMENT (net-new `in_string_or_comment` over the app's
  existing `syntax_cache`; see D2), and **Esc suppresses re-open until the next real trigger**.

### Out (explicitly deferred)
- **Snippet TABSTOPS** — v1 strips the markers and drops the caret at the end; real `$0`/`${1:…}`
  navigation is its own feature (the ticket says so).
- **`completionItem/resolve`** — the lazily-filled `documentation`/`detail` second round trip. v1 shows
  what the first response carried.
- **`additionalTextEdits` (auto-import)** — rust-analyzer sends these to add a `use` line. Applying edits
  ELSEWHERE in the file while the caret sits here is a distinct undo/anchor problem; v1 applies the
  primary edit only and does not silently half-apply — a follow-up owns it.
- **Multi-cursor accept** — the ticket's VALIDATE line mentions "multi-cursor accept applies per cursor",
  but a server's `TextEdit` names ONE range from ONE position; replaying it at N cursors is an invented
  semantic, not an LSP one. v1 accepts at the primary cursor. A follow-up may define the N-cursor rule.
- Auto-trigger on every keystroke without a prefix (a bare `a` opening the whole crate universe) — the
  trigger table gates this; tuning the aggressiveness is post-ship polish.

## Reference (§20)
**Zed (the editor)** — the behavior matched is the universal as-you-type completion popup: it appears
under the caret as you type, ranks by what you have typed, and **accepting replaces the typed prefix
rather than appending to it**, all reimplemented on the published **LSP 3.17 `textDocument/completion`**
wire (`CompletionList`/`CompletionItem`/`TextEdit`/`InsertReplaceEdit`, and the `isIncomplete` re-query
rule) plus Marley's own `fuzzy_rank`, `complete::popup_window`, editor `Buffer`, and undo model. No Zed
GPL source read (§20 clean-room); the replace-not-append semantic is the spec's own
(`CompletionItem.textEdit` names an explicit range), not a translation. Marley's own shipped #96 shell
popup is the interaction idiom being mirrored in-house.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the ticket's `pair_action` ordering is DROPPED as a PHANTOM dependency.** Plan-phase Explore
  proved `pair_action` has ZERO occurrences in `crates/` — #301 (auto-pairs) is unimplemented, so
  "#313's keystroke hook runs after the #301 pair_action consult" orders against a symbol that does not
  exist. #313 hooks the printable path directly and is not blocked by #301. (#301's own description
  carries the same false claim and should be corrected when it is worked.)
- **D2 — `in_string_or_comment` is NET-NEW over the app's `syntax_cache`, NOT a `crates/syntax` lookup.**
  The ticket claims "the tree knows — crates/syntax `point_at`". FALSE: `point_at(src, byte) -> (row,
  byte-col)` is a coordinate converter, and `marley_syntax`'s entire public surface (7 items) exposes no
  tree/node query — the `tree_sitter::Tree` never escapes the crate. The gate is instead built from the
  cached `Vec<Vec<(Range<usize>, TokenKind)>>` the app already maintains (`RootView::syntax_cache`),
  asking whether the offset falls in a `TokenKind::Str | Comment` span. **Design must resolve the
  staleness question**: that cache is async + version-keyed, so on the very keystroke that triggers a
  completion it may lag the buffer. Phase 2 picks the stance (a stale-cache tick suppresses, or falls
  open) and states why.
- **D3 — hand-parse the payload; do NOT reach for `lsp-types`.** The crate's rule is explicit and
  observed: "the ONLY `lsp-types` consumer is `parse_publish_diagnostics`" (#310), and `hover.rs` — a
  response parser — states it hand-parses to keep that surface at the publish seam. Hand-parsing also
  buys per-item skip-on-malformed (a `from_value::<Vec<CompletionItem>>` rejects the WHOLE list on one
  bad element), which is the same argument #312 made. `handshake.rs` does sanction `TextEdit` "earning
  its dependency" later; taking that exception would need an argued deviation in Phase 2, and the
  default is hand-parse.
- **D4 — the accept BRACKETS ITS OWN undo group and therefore uses raw `edit()`, not
  `edit_at_selections`.** Not ceremony — the hazard is live and narrow: for N=1 `edit_at_selections`
  opens NO group, so its record can coalesce with the next typed char. An accept whose `TextEdit` range
  is EMPTY (`start == end` — exactly what a server returns after a `.` trigger) satisfies every coalesce
  condition, so ⌘Z would revert the accept AND the character typed after it — the #299 incident's shape.
  A hand-bracketed group is `cursor_anchored: false` by construction, which nothing can absorb. The
  shipped precedent is the Tab/indent site, which brackets its own group and keeps raw `edit()` calls
  for exactly this reason (`edit_at_selections` SELF-BRACKETS and `begin_group` silently discards an
  open group's records).
- **D5 — `text_input_blocked` gets NO line for this popup.** Every other overlay is listed there; this
  one must NOT be, because its printables have to fall through so typing keeps filtering. That absence
  is load-bearing and already documented in the predicate's own NOTE for #96. (This is the deliberate
  exception to `PR-claude-new-overlay-register-at-every-choke-point-001`.)
- **D6 — v1 accepts at the PRIMARY cursor only** (see Out). A server `TextEdit` names one range from one
  position; fanning it to N cursors is a semantic Marley would be inventing.
- **D7 — the popup mirrors #96's STATE/KEY/RENDER idiom, not its accept path.** `complete.rs`'s state is
  welded to shell path-completion (a dir-listing snapshot, a prefix filter, no ranking) and it accepts
  into the TERMINAL input buffer — no undo groups, no `SelectionSet`, no IME. The transferable assets are
  `popup_window` (reused as-is) and the two-phase "arm declines the printable → post-edit refilter" shape.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an identifier character or one of the server's advertised trigger characters is typed in a focused editor tab whose host is Ready, the system shall send a `textDocument/completion` request at the caret via the #311 path. | pure trigger-table units + a driven wire check |
| REQ-002 | WHEN a completion response arrives, `parse_completion_result` shall normalize BOTH a `CompletionList{isIncomplete, items}` and a bare `CompletionItem[]` into `Vec<CompletionItem>`, normalizing `textEdit` / `InsertReplaceEdit` / `insertText` / label-only into one edit value; a malformed element shall be skipped and a `null`/empty result shall yield no items. | pure units (both shapes + each edit form + malformed/null) |
| REQ-003 | WHEN a prefix has been typed, the system shall rank the items by `marley_search_core::fuzzy_rank` over `filter_text ?? label`, stable-sorted by (score desc, `sort_text`, `label`); an empty prefix shall preserve server order. | pure rank units (incl. tie-stability) |
| REQ-004 | WHILE the popup is open it shall render under the caret's row, FLIPPING above the caret when it would not fit below the window, with the selected row lit; ↑/↓ shall move the selection and a printable shall fall through and re-filter. | pure `popup_origin` truth table + pure state units + driven |
| REQ-005 | WHEN an item is accepted (Enter/Tab), the system shall apply that item's edit RANGE — replacing the typed prefix, not appending after it — as ONE undo unit, and a character typed immediately after the accept shall NOT coalesce into it (one ⌘Z reverts only the accept). | pure edit-shape units + a driven undo drive (the empty-range case explicitly) |
| REQ-006 | WHEN the caret is inside a string or a comment, the system shall not open the popup. | pure `in_string_or_comment` units + driven |
| REQ-007 | WHEN Esc is pressed the popup shall close and stay closed until the next real trigger (a typed identifier char shall NOT reopen it). | pure suppression-state units + driven |
| REQ-008 | WHEN a response arrives whose key (uri + position + buffer version) no longer matches the live request, or whose file is no longer focused, the system shall DROP it — no popup, no edit. | pure key units + a headless drive (stale + moved-on) |

## Phase Plan
- **P2 Design** — the pure/shim split; the `CompletionKey` (uri+position+**version**); the response-arm
  wiring into the ONE `consume_lsp_responses` drain; **the editor printable/refilter seam** (the highest
  uncertainty — #96's post-edit hook is on the TERMINAL path; editor printables leave the router and land
  in `EntityInputHandler`/`ime::replace_text`, so this seam is net-new); the D2 staleness stance; the
  accept's undo bracketing; the file manifest + ≥1 test per REQ.
- **P3 Implement** — the pure seams then the shim wiring; `cargo check` green.
- **P3.5 Inspect** — critics vs the diff. Named risk seams: the accept's edit-range faithfulness + its
  undo bracketing (D4), the stale-guard under per-keystroke rates, the printable fall-through, and the
  `in_string_or_comment` staleness.
- **P4 Validate** — pure suites to cov/MSI 100 + headless drives + **a LIVE wire check** (per
  `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`: an injected response proves only
  the consumer — assert the real `textDocument/completion` traffic via the `[[lsp.servers]]` tee).
- **P5 Complete** — CHANGELOG + architecture docs, AAR capture, close + archive.
</content>
</invoke>
