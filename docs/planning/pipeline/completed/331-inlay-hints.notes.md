# LSP inlay hints — inline type + parameter annotations (the display-map ticket) — Notes

- **Forge ticket:** #331 (38801e5a-b739-43a0-ae71-a5060c8be6b1)
- **AAR:** 040d8d7f-9dc3-4370-a6d9-34ffb80fd975
- **Local ticket doc:** docs/planning/tickets/open/TICKET-331-inlay-hints.md
- **Pipeline spec:** 331-inlay-hints.spec.md

## Phase 1 — Plan
- **Request:** render rust-analyzer's inline type/parameter hints (`let s: String = …`, the `: String` muted
  and NOT in the file). Deliberately the LAST M21 ticket: it is the render-model change the whole batch was
  sequenced around.
- **Classification / tier:** work pipeline, one shippable slice — but the batch's highest-risk seam. A pure
  layout change + a pure parse + a request/refresh loop + a render run + a setting.
- **Forge recall:** the #308–#313 LSP family (request recipe, encoding bridge, stale-guard), the #330
  `editor.*` setting + palette-toggle precedent + its four-wiring lesson, and the M21 gate lessons all apply.
- **The stated risk (from the ticket):** the editor render is a strict 1:1 buffer→display mapping —
  `col_starts[i]` = the display col of the i-th BUFFER char (code_view.rs:45); caret-x (app.rs:3636), click-x
  (:3700), drag, selection rects (#255) and #310 squiggles ALL ride it. Nothing renders phantom mid-line text
  today (#200's ghost is EOL-only + terminal-side). Injecting hint text would desync every rider unless the
  MAPPING learns about phantoms.

### **The plan's load-bearing discovery: the existing `LineLayout` already supports phantoms.**
Reading the seam (`code_view.rs:38-128`) closely:
```rust
pub struct LineLayout {
    pub display: String,      // what the editor DRAWS (tabs already expanded)
    col_starts: Vec<usize>,   // col_starts[i] = display col where the i-th CHAR begins; len == n_chars + 1
}
pub fn line_layout(line: &str, tab_width: usize) -> LineLayout {
    for ch in line.chars() {
        col_starts.push(col);                  // ← the map entry for THIS buffer char
        if ch == '\t' { …expand to the next stop, col += spaces… }
        else { display.push(ch); col += char_width(ch); }
    }
    col_starts.push(col);                      // the EOL position (total width)
}
```
The two fields are already DECOUPLED in exactly the way phantoms need: `display` is the drawn string,
`col_starts` maps only BUFFER chars. So a phantom is simply **"emit text into `display` + advance `col`,
WITHOUT pushing a `col_starts` entry"**, before the anchor char's own `col_starts.push(col)`. Consequences,
all free:
- **`col_of_offset(i)` returns the col AFTER a phantom anchored at `i`** — because the phantom advanced `col`
  before `col_starts.push(col)` ran for char `i`. That IS the D-CONTRACT (caret on the code side).
- **`offset_of_col_f` (`:66`) scans `col_starts` only** — it has no entry inside the phantom, so a click in the
  phantom resolves to the nearest anchoring boundary (`i-1` or `i`, tie → later). "A hint is never a caret
  home" is free; no change to the fn at all.
- **Selection rects + squiggles are free** — `row_selection_cols` (`:135`) and the #310 underline mapping both
  go through `col_of_offset`, so they shift with the phantoms automatically.
- **The empty-slice identity is free** — with no inlays the loop is byte-for-byte the current one, so
  `line_layout(l, w) = line_layout_with_inlays(l, w, &[])` and hints OFF is provably the pre-ticket path
  (a property test pins it).
**So the ticket's feared "display-map rewrite" is ONE pure function with an extra emit step.** This is exactly
what the ticket predicted ("Marley's per-line render makes a far smaller shape possible") — the risk is real
but contained, and the containment is now verified rather than hoped.
- **Discovery (the rest of the surface):** `RequestPurpose` (lsp_host.rs:38 — the #311 recipe);
  **`Incoming::ServerRequest{id, method, params}` is ALREADY routed** (rpc.rs:64/:87/:393 → lsp_host.rs:431),
  so `workspace/inlayHint/refresh` is a new ARM, not new plumbing; `char_width` (`code_view.rs:86`, UAX#11) is
  the ONE column authority a phantom's width must also use; the #330 setting/palette precedent (`e88e632`)
  gives the four-wiring shape (define_setting + AppliedSettings ×2 + persist + the NON-DEFAULT round-trip leg,
  and the CommandId trio — next free is **15**).
- **Decisions:** D-SHAPE (extend in place; `line_layout` = the empty-slice wrapper), D-CONTRACT (code-side
  caret, click snaps to an anchor — both by construction), D-OFF-IS-IDENTICAL (the property test), D-REQUEST
  (#311 + range-keyed guard + the refresh arm), D-PARSE (both label shapes, parts concatenated, filter_map,
  #309 bridge), D-SETTING (`editor.inlay_hints` + palette toggle). §20 = LSP-3.17-spec clean-room (the
  #308–#313 posture); Zed's InlayMap = architecture CONCEPT only, source unread; VS Code = observed
  caret/click behavior.
- **The open question for P2:** the inlay parameter type — a bare `&[(usize, &str)]` (leanest, and the app
  already owns the hint store) vs a typed `Inlay { char_idx, text }` (self-documenting, room for kind/padding
  later). Design picks; the truth table is identical either way.

## Phase 2 — Design

### §20 confirm
Clean-room from the PUBLISHED LSP 3.17 spec (`textDocument/inlayHint`, `workspace/inlayHint/refresh`,
`InlayHint`/`InlayHintLabelPart`/`InlayHintKind`, `paddingLeft`/`paddingRight`) — the #308–#313 posture: the
wire is built from the spec document. Zed's `InlayMap` = the "empty input, non-empty output" layer-contract
CONCEPT only (deconstruction 03 §3; its 209KB machinery is "defer wholesale") — Marley's per-line render
admits one pure function instead; source UNREAD. VS Code = OBSERVED caret/click behavior. Holds.

### (1) D-SHAPE — CONFIRMED, and the invariant it rests on is now NAMED
```rust
pub fn line_layout_with_inlays(line: &str, tab_width: usize, inlays: &[Inlay]) -> LineLayout
pub struct Inlay { pub char_idx: usize, pub text: String }   // typed (room for kind/padding later)
pub fn line_layout(line, tab_width) -> LineLayout { line_layout_with_inlays(line, tab_width, &[]) }
```
The walk is today's, plus ONE step: before the anchor char's `col_starts.push(col)`, emit each phantom
anchored at that char into `display` and advance `col` by its `char_width` sum (the ONE UAX#11 authority —
a wide-glyph hint must measure right; hints carry no tabs). Also record the phantom's DISPLAY-BYTE range.

**THE INVARIANT (implicit today, never named, enforced only by `line_layout` building both outputs in one
pass): `display`'s cell-accumulation and the values in `col_starts` are the SAME column domain.**
`cols_to_bytes` re-derives columns by WALKING `display`; `col_of_offset` READS them from `col_starts`. They
agree only because one loop produced both.

- **This design keeps them in one domain** — the phantom advances `col` BEFORE the push, so `col_starts[i]`
  is the SCREEN column of buffer char `i` (after any phantom at `i`). Therefore: `col_of_offset` →
  screen col (the caret pixel `left(px(col * cell_w))` is correct — it lands after the hint, on the code);
  `cols_to_bytes(display, c0, c1)` walks the same screen domain → `raw_span_to_display_bytes` maps code spans
  correctly; `offset_of_col_f` inverts screen cols → a click is correct, and a click INSIDE a phantom (whose
  columns have no `col_starts` entry) resolves to the nearest anchoring boundary — "a hint is never a caret
  home", free; `row_selection_cols` + the #310 squiggles ride `col_of_offset` → correct, free. **NO signature
  change to any of them.**
- **THE TRAP, documented (the Explore pass's worked proof):** the *obvious* implementation — inject into
  `display` but leave `col_starts` phantom-BLIND — FORKS the domains and corrupts everything. Proof: `"let x
  = 1;"` + a 5-cell `": i32"`; the literal `1`'s raw span 8..9 → phantom-blind `col_of_offset` → cols 8..9 →
  `cols_to_bytes` walks the phantom-ful display and returns bytes 8..9 = **the `3` of `i32`** — the Number
  tint paints the hint and the real `1` renders untinted, off by exactly the phantom width; the caret,
  squiggles, IME box and click mapping drift the same way. **A direct invariant test pins this** (assert
  `col_of_offset(i)` equals the column `cols_to_bytes`-style accumulation of `display` reaches at char `i`),
  so a future refactor cannot silently un-name it.
- `col_starts` is PRIVATE (`code_view.rs:47`) and touched only inside `code_view.rs` → the whole change is
  contained there.

### (2) D-HINT-TOKEN — a new `TokenKind::Hint` is REQUIRED (not cosmetic)
`styled_slices_with_marks` ends with `if kind != TokenKind::Plain || selected || mark.is_some()` — it DROPS
any slice that is Plain + unselected + unmarked. A phantom left untagged is exactly that → dropped → rendered
in the base `foreground`, **visually indistinguishable from real code**. So: `LineLayout` gains
`hint_spans: Vec<Range<usize>>` (the phantoms' DISPLAY-BYTE ranges — the fn that places them knows exactly
where they landed; empty for `line_layout`), and the render pushes `(span, TokenKind::Hint)` into the `syntax`
vec. `TokenKind::Hint` survives the guard → `token_color(Hint) = colors.muted`. **Zero change to
`styled_slices_with_marks`.** (`Comment` already maps to `muted`; a distinct variant is what #316's palette
will separate.) The pinning test `token_color_maps_each_kind` (app.rs:14917) gains a Hint line.

### (3) D-PARSE / D-PARAMS / D-CAPS (marley_lsp — a new `inlay.rs`, the #311/#312/#313 module shape)
- `parse_inlay_hints(&Value) -> Vec<InlayHint>` mirroring `definition.rs:32-65` (array → per-element
  `filter_map`, so one bad hint cannot lose the line). `InlayHint { line: u32, character: u32, label: String,
  kind: Option<InlayHintKind>, padding_left: bool, padding_right: bool }`; label = a bare string OR
  `InlayHintLabelPart[]` CONCATENATED (v1). **Positions via `try_from`/`as_u64`, NEVER `as u32`** — the #312
  lesson (a cast would truncate 2^32 → 0, a confidently-wrong hint position).
- `inlay_hint_params(uri, start_line, end_line) -> Value` — its OWN shape `{textDocument:{uri}, range:{start,
  end}}` (NOT the `text_document_position_params` base; the `signature_help.rs:188` precedent).
- `inlay_hint_support(server_caps) -> bool` — the `signatureHelpProvider` bool/object shape
  (`signature_help.rs:179`).
- **D-CAPS (the #323 lesson):** ADVERTISE `textDocument.inlayHint: { dynamicRegistration: false }` +
  `workspace.inlayHint: { refreshSupport: true }` in `initialize_params_json` — a result-reading client that
  does not advertise gets nothing, and refreshSupport is what makes the server SEND the refresh at all
  (advertise ONLY what we implement — we implement both). **The tripwire test `handshake.rs:343-351` asserts
  textDocument has EXACTLY 2 caps — it must go to 3** (that assert is deliberate; update it + its comment).

### (4) D-REQUEST + D-REFRESH (the one genuinely novel seam)
- `RequestPurpose::InlayHints(InlayKey)` where `InlayKey { uri: String, version: BufferVersion, first_row:
  usize, last_row: usize }` — **range/version-scoped, not caret-scoped** (the latest VIEWPORT request wins).
  Fired from the pump on a version bump or a scroll into unfetched rows, over the viewport ± a page.
  `inlay_request: Option<InlayKey>` is the stale guard, the #313 shape verbatim: `if self.inlay_request
  .as_ref() != Some(&key) { return false }`, then re-check the live `(path, version)` before applying.
- The reply projects LSP `(line, character)` → `(row, char_idx)` through the **#309 encoding bridge ONCE at
  apply time** (the #310 diagnostics pattern — never at render), and caches
  `inlay_hints: Option<(u64 nonce, BufferVersion, Vec<(usize row, usize char_idx, String text)>)>`. The
  render filters that to the row (the #328/#330 cache shape).
- **D-REFRESH:** `reply_for_server_request` is a PURE `&str -> ReplyPolicy` with no host access, so it can
  give the null reply for free (`"workspace/inlayHint/refresh" => ReplyPolicy::AckNull`) but CANNOT trigger a
  re-fetch. Minimal new plumbing (there is no precedent — this is the novel bit): a `inlay_refresh: bool` flag
  on `LspHost`, SET in the `Incoming::ServerRequest` arm when the method matches, drained by
  `take_inlay_refresh() -> bool`; the pump consumes it → clear the cache + re-request. ~15 lines, one flag,
  one take, one arm — no new channel.

### (5) D-SETTING — the #330 four-wiring, verbatim
`define_setting!(InlayHints: bool = true, "editor.inlay_hints")` + `AppliedSettings.inlay_hints` +
`applied_from` + `applied_defaults` + `persist_inlay_hints` + **a NON-DEFAULT leg in
`settings_round_trip_survives_reload`** (persist `false`; the #330 lesson — else the `persist_X -> Ok(())`
mutant survives). Palette "Toggle Inlay Hints" `CommandId(15)` — `cockpit_commands` + `action_for_command` +
`dispatch_action` TOGETHER (the two completeness tests). OFF → the render passes `&[]` → the empty-slice path
→ provably the pre-ticket layout.

## Architecture / file manifest
| File | Change |
|---|---|
| `crates/marley_app/src/code_view.rs` | ADD `pub struct Inlay { char_idx, text }`; `pub fn line_layout_with_inlays(line, tab_width, &[Inlay]) -> LineLayout` (the one-pass walk + the phantom emit BEFORE the anchor's `col_starts.push`); `line_layout` becomes its empty-slice wrapper; `LineLayout` gains private `hint_spans: Vec<Range<usize>>` (display-byte) + `pub fn hint_spans(&self) -> &[Range<usize>]`. Pure, cov/MSI 100. |
| `crates/marley_app/src/code_syntax.rs` | `TokenKind::Hint` variant (non-Plain — survives the drop guard). |
| `crates/marley_lsp/src/inlay.rs` | NEW. `InlayHint`, `InlayHintKind`, `parse_inlay_hints`, `inlay_hint_params`, `inlay_hint_support`. Pure, cov/MSI 100. |
| `crates/marley_lsp/src/lib.rs` | `mod inlay;` + re-exports. |
| `crates/marley_lsp/src/handshake.rs` | Advertise `textDocument.inlayHint` + `workspace.inlayHint.refreshSupport`; UPDATE the 2→3 tripwire test + its comment. |
| `crates/marley_lsp/src/rpc.rs` | `"workspace/inlayHint/refresh" => ReplyPolicy::AckNull` in `reply_for_server_request`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::InlayHints(InlayKey)`; `inlay_refresh: bool` + `take_inlay_refresh()`; the `ServerRequest` arm sets the flag on that method (mutants::skip on the shim). |
| `crates/marley_app/src/app.rs` | `token_color` Hint arm (+ the pinning test line); fields `inlay_hints` cache + `inlay_request` guard + `inlay_hints_on: bool`; `refresh_inlay_hints` (pump: fire on version/scroll/refresh-flag, mutants::skip); `apply_inlay_response` (the #313 guard + the #309 projection, mutants::skip); the row render uses `line_layout_with_inlays` + pushes `hint_spans` as `TokenKind::Hint` into `syntax`; `toggle_inlay_hints` + the dispatch arm + the `CommandId(15)` cockpit row; test hooks. |
| `crates/marley_app/src/settings.rs` | `InlayHints` setting + `AppliedSettings` field + both resolvers + `persist_inlay_hints` + the NON-DEFAULT round-trip leg. |
| `crates/marley_app/src/palette.rs` | `CommandId(15) => "toggle-inlay-hints"`. |

## Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| 001 | `inlays_consume_columns_without_col_starts_entries` — a hint mid-line: `display` contains it, `col_starts.len() == n_chars + 1` still, and later chars' columns shift by the hint's width. | code_view pure (cov/MSI 100) |
| 001b | **`layout_invariant_col_starts_matches_display_accumulation`** — for a line WITH hints, every `col_of_offset(i)` equals the column reached by accumulating `char_width` over `display` up to that char's display byte. **Pins the named invariant** (the trap: a phantom-blind `col_starts` fails here). | code_view pure |
| 002 | `empty_inlays_is_byte_identical_to_line_layout` — for a corpus of lines (tabs, wide glyphs, emoji, empty), `line_layout_with_inlays(l, w, &[]) == line_layout(l, w)` (whole struct). | property |
| 003 | `col_of_offset_lands_after_a_phantom_at_that_char` — the caret sits on the CODE side. | pure |
| 004 | `click_inside_a_phantom_snaps_to_an_anchor` — both halves (left → the char before, right → the anchor char); never a "hint offset". | pure |
| 005 | `layout_truth_table` — hint at BOL / mid-line / EOL / two ADJACENT hints / hint next to a TAB (the stop math still composes) / hint next to a WIDE glyph. | pure (the table) |
| 005b | `raw_span_to_display_bytes_maps_code_spans_past_a_phantom` — the Explore trap, inverted: with a hint before it, the literal's span still maps to the literal's display bytes (NOT the hint's). | pure |
| 006 | `parse_inlay_hints_*` — a bare-string label; `InlayHintLabelPart[]` concatenated; kind Type/Parameter/absent; padding flags; a malformed element skipped (others survive); a `2^32` position skipped not truncated. | marley_lsp pure (cov/MSI 100) |
| 006b | `inlay_hint_params_shape` + `inlay_hint_support_table` + `handshake advertises inlayHint + refreshSupport` (the updated tripwire). | marley_lsp pure |
| 007 | `inlay_stale_reply_dropped_headless` — a superseded key / a moved version → the reply is DROPPED; the matching one applies + caches. | headless |
| 008 | `inlay_refresh_request_refetches_headless` — the `workspace/inlayHint/refresh` ServerRequest → `AckNull` reply + the flag drains → a re-fetch. (+ `reply_for_server_request` unit for the new arm.) | headless + pure |
| 009 | `hints_render_muted_headless` — the row's `syntax` gains the hint span as `TokenKind::Hint`; `token_color(Hint) == colors.muted`. | headless/review + LIVE(fallback) |
| 010 | `inlay_setting_roundtrip` (NON-DEFAULT leg) + `toggle_off_uses_empty_slice_headless` (OFF → the layout equals `line_layout`). | unit + headless |
| — | LIVE: a muted `: String` mid-line; a click just after `s` types into CODE (file byte-checked); OFF is pixel-identical. **env-blocked (screen locked)** → units+mechanism. | uncoverable-live |

## Risks / decisions
- **The two-column-domain fork is THE risk** — contained by building both outputs in one pass (as today) and
  pinned by the 001b invariant test. Named in the code comment so it is not implicit again.
- **`TokenKind::Hint` is required, not cosmetic** (the drop guard) — and it touches the shared `token_color`,
  so the hover-fence render path (app.rs:4471+) inherits it harmlessly (it passes no hints).
- **The refresh flag is the one novel seam** (no precedent) — kept to a bool + a take + one arm.
- **Two pinning tests will fail until updated**: the handshake 2→3 cap count, and `token_color_maps_each_kind`.
  Both are deliberate tripwires — update, never weaken.
- **`git add -N crates/marley_lsp/src/inlay.rs`** before the `--diff` gate (the #329 mutation-gap lesson).
- Scope honesty: this is the batch's largest diff (10 files). Every piece is required by an AC; the pure seams
  (`line_layout_with_inlays`, `parse_inlay_hints`) carry the mutation surface, the app/host shims are
  `mutants::skip` + drive-verified.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean, lib clippy clean, `cargo nextest run -p marley
-p marley_lsp` → **749 passed** (the two tripwires updated, see below).
- **`crates/marley_app/src/code_view.rs`** — `pub struct Inlay { char_idx, text }`;
  `line_layout_with_inlays(line, tab_width, &[Inlay])` = today's one-pass walk + an `emit_hints_at` step that,
  BEFORE each anchor char's `col_starts.push(col)`, pushes the hint text into `display`, advances `col` by its
  `char_width` sum, and records the display-BYTE range in the new private `hint_spans` (+ `pub fn
  hint_spans()`). An EOL hint (anchor == `n_chars`) emits before the final push; an out-of-range anchor is
  ignored (total). `line_layout` is now the empty-slice wrapper. **The INVARIANT is written at the fn** —
  `display`'s cell accumulation and `col_starts` are ONE column domain, which is why the phantom advances
  `col` before the push (keeping `col_starts` in SCREEN columns) and why the trap (a phantom-blind
  `col_starts`) would silently shift every span and pixel.
- **`crates/marley_app/src/code_syntax.rs`** — `TokenKind::Hint` (non-Plain, with the reason in the doc).
- **`crates/marley_lsp/src/inlay.rs`** (NEW) — `InlayHintKind{Type,Parameter}`, `InlayHint`,
  `parse_inlay_hints` (array → per-element `filter_map`; `u32::try_from(as_u64())` NEVER `as u32`; label a
  bare string OR parts concatenated; kind 1/2; padding flags), `inlay_hint_params` (its own
  `{textDocument, range}` shape), `inlay_hint_support`. `lib.rs` gains `pub mod inlay;` + the re-exports.
- **`crates/marley_lsp/src/handshake.rs`** — advertises `textDocument.inlayHint {dynamicRegistration:false}`
  + `workspace.inlayHint {refreshSupport:true}` (the #323 lesson; refreshSupport is what makes the server send
  the refresh at all). **Tripwire updated 2→3** with its comment.
- **`crates/marley_lsp/src/rpc.rs`** — `"workspace/inlayHint/refresh"` joins the `AckNull` arm.
- **`crates/marley_app/src/editor_inlay.rs`** (NEW) — `InlayKey { uri, version, first_row, last_row }` (the
  range/version identity; the `CompletionKey` sibling keys on a caret because its request fires per keystroke).
- **`crates/marley_app/src/lsp_host.rs`** — `RequestPurpose::InlayHints(InlayKey)`; an `inlay_refresh` LATCH
  + `take_inlay_refresh()`; the `ServerRequest` arm raises it on that method (the reply stays the shared
  policy). N refreshes coalesce into one refetch — correct semantics, nothing lost.
- **`crates/marley_app/src/app.rs`** — `InlayCache = (nonce, version, fetched_rows, [(row, char_idx, text)])`
  + `INLAY_PAGE = 50`; fields `inlay_hints` / `inlay_request` / `inlay_hints_on` (from `applied.inlay_hints`);
  `refresh_inlay_hints` (pump — off drops the cache; drains the refresh latch; requests when the cache can't
  serve the viewport: new version/nonce, scrolled out of `fetched_rows`, or refreshed; skips a duplicate
  in-flight key); `apply_inlay_response` (the #313 guard — latest key only, version + uri re-checked — then
  the **#309 projection ONCE here**: `char_col_from_column(line_text, hint.character, enc)`, padding folded
  into the text, a hint past the buffer end DROPPED not clamped); the drain arm; `inlay_hints_for_active`
  (row → sorted phantoms, captured before the `'static` closure like `egit`); the row render uses
  `line_layout_with_inlays` and **extends `syntax` with `hint_spans` as `TokenKind::Hint`** (required — the
  drop guard); `token_color(Hint) = colors.muted`; `toggle_inlay_hints` + the dispatch arm + the
  `CommandId(15)` cockpit row. All app/host shims carry `mutants::skip`.
- **`settings.rs`** — `InlayHints: bool = true, "editor.inlay_hints"` + `AppliedSettings.inlay_hints` + both
  resolvers + `persist_inlay_hints` + the 3 test ctors + **the NON-DEFAULT round-trip leg** (persist `false`).
- **`palette.rs`** — `CommandId(15) => "toggle-inlay-hints"`.

**Deviations from design:** none material. The `InlayCache` gained the `fetched_rows` range (the design's
"range-keyed" guard made concrete — hints cover a region, so scrolling out must refetch even at the same
version). `InlayKey` lives in a new `editor_inlay.rs` (the `editor_complete.rs`/`CompletionKey` precedent).

**Two deliberate tripwires updated (never weakened):** the handshake's "exactly 2 textDocument caps" assert
(now 3, naming inlayHint + why), and — pending at Validate — `token_color_maps_each_kind` needs a `Hint` line.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

3 parallel critics over the diff: (1) the phantom column math, (2) the LSP payload + request/refresh state,
(3) render + wiring + provenance. Each verified concretely (hand traces, throwaway probes, real commands);
both probe-writing critics left the tree clean (re-verified against the pre-probe `--shortstat`).

**The named invariant HOLDS — verified, not assumed.** Critic 1 asserted
`display.chars().map(char_width).sum() == *col_starts.last()` across the cross-product of 7 lines
(empty/plain/tabs/CJK/combiner) × 5 hint texts × every anchor `0..=n`: zero forks. `col_starts` is genuinely
in SCREEN columns and `len == n_chars + 1` never moves. The ticket's headline case is right:
`"let x = 1;"` + `Inlay{5, ": i32"}` → `col_starts[8] == 13` → `raw_span_to_display_bytes(8..9)` → display
`13..14` == the REAL `1`, not the phantom's `3`.

**But the invariant I named was necessary, not sufficient** — one column domain, two BOUNDARY semantics.
Critics 1 and 3 INDEPENDENTLY found the same HIGH from opposite ends (1 from the math, 3 from reachability),
which is what makes it credible rather than clever.

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | HIGH | **The end-boundary swallow.** `raw_span_to_display_bytes` builds a span's END as `col_of_offset(span.end)` — which, by the D-CONTRACT, is the column AFTER a phantom anchored there. So a code span whose one-past-end char is a hint anchor EXTENDS ACROSS the phantom. `styled_slices_with_marks` picks the FIRST containing range and hints were appended LAST → the code span wins → **the hint paints as code**. Critic 3 proved reachability on the PRIMARY tree-sitter path via rust-analyzer **chaining hints** (default ON, always EOL-anchored, and a literal DOES have a span ending exactly there): `let n = "hello"` → Str `8..15` mapped to display `8..20` → the hint rendered string-green. The same root cause hits the squiggle (`app.rs`): a warning on `x` in `let x = 1;` underlined `x: i32`. | **REAL** (2 critics, independently; reproduced) | Added `col_ends` + `LineLayout::col_of_span_end` — the END-side map. Used for the two spans that describe CODE (`raw_span_to_display_bytes`, the diagnostic bar). **Verified: Str `8..15` → display `8..15` = `"hello"`; squiggle `ds=4 de=5`.** |
| F1b | — | Critic 3 prescribed fixing the accessor **everywhere, including `row_selection_cols`**. | **REJECTED in part** — and this is the ticket's real lesson. A selection band must end where the caret it follows RENDERS (offset 5 → col 10, per the locked D-CONTRACT); ending it at col 5 would detach the band from its own caret by the phantom's width. Critic 1 independently reached the opposite conclusion and was right. | `row_selection_cols` deliberately KEEPS `col_of_offset`. Both behaviours now proven on the same char range: squiggle `de=5`, selection `(4,10)`. |
| F2 | HIGH | The Hint tag's survival depended on **no span-producer ever enclosing a phantom** — an accident of another crate's capture table (identifiers are `Plain` → dropped), not a property. `app.rs:754` names #316's palette as the next change here; the moment it adds an `Identifier`/`Type` capture, every `let` binding's type hint mis-colors. | **REAL** (latent; F1 fixes today's instance, not the class) | Hints built FIRST, code spans extended after — first-wins ordering makes the phantom's own cells win **unconditionally**, including against a span that legitimately encloses one (the hand lexer). The tag no longer depends on another producer's table. |
| F3 | MED | **The EOL hint sat on the wrong side of the EOL caret.** `emit_hints_at(n_chars)` ran BEFORE the final `col_starts.push`, so End parked the caret out past a chaining hint (`.bar()  Foo` → col 15 instead of 10), and an empty line's caret rendered at column 1. My own code comment asserted the opposite ("the code side") — the parenthetical was inverted. | **REAL** — and it exposed that "the caret sits on the code side" is an ASYMMETRIC rule, not a uniform one | Push THEN emit at EOL. The asymmetry is the contract: mid-line the code is to the phantom's RIGHT (emit→push); at EOL there is no char to its right, so the code is to its LEFT (push→emit). Both are "the caret sits on the code side". **Verified: EOL caret 15; empty-line caret 0.** |
| F4 | MED | **No language gate.** `language_id_for` is `.rs`-only, so a non-Rust file is never `didOpen`'d — but `refresh_inlay_hints` asked anyway. `Err` writes no cache → `served` stays false → **resend, forever, one per round-trip**. (`Ok(null)`/`Ok([])` cache correctly; only the error path loops.) Guarded today only by server behaviour, not by code. | **REAL** | Gate on `marley_lsp::language_id_for(&path).is_some()` — the ONE table #309 syncs by, not a second copy. Also clamped `want_last` to `len_lines` (a 20-line file was asking for rows 0..69). |
| F5 | MED | `inlay_hint_support` was **dead code** — written, re-exported, called NOWHERE. So the request went to every Ready server regardless of `inlayHintProvider` → MethodNotFound → the same unbounded error loop. A capability reader nothing reads is a claim the code doesn't keep. | **REAL** | Wired it: a `LspHost::inlay_hint_support` wrapper mirroring `signature_help_support` exactly, gating the send. |
| F6 | MED | **The poll-driven skip wedges on timeout.** `expire` does `purposes.remove(&id)` and pushes NO response, so `apply_inlay_response` never runs, `inlay_request` stays `Some(key)`, and the skip suppresses every resend. Critic 2 called it HIGH, critic 3 LOW ("self-heals on scroll"). **Critic 2's refinement is the tiebreaker:** the key only changes on scroll/edit, so the wedge lasts exactly as long as the user READS — which is exactly when a cold rust-analyzer exceeds `REQUEST_TIMEOUT_TICKS` (10s). And the AMPLIFIER: when r-a finishes indexing and sends `workspace/inlayHint/refresh` — the literal "hints are ready now" signal — `take_inlay_refresh` destructively consumed it, dropped the cache, and hit the skip. The one mechanism that would heal the wedge guaranteed it stayed wedged *with an empty cache*. Directly contradicts `REQUEST_TIMEOUT_TICKS`'s own comment: *"a slow server never wedges a consumer."* | **REAL** — settled at MED, fixed | `LspHost::has_pending_inlay()` — the pending table is the ONE source of truth for in-flight-ness, so a dropped purpose self-heals into a resend (cadence: one per timeout window). Plus `refreshed` now clears `inlay_request` too, so a pre-refresh in-flight answer is discarded rather than cached. |
| F6b | — | Critic 2's proposed fix: push a synthetic `Err` for every abandoned purpose in `lsp_host`, claiming *"siblings are unaffected."* | **REJECTED — the claim is false.** I read every sibling `Err` arm: `prepare_rename` → `declined` → a **"Cannot rename this"** flash; `code_action` → **"Code actions failed"**; `code_action_resolve` → **"Code action resolve failed"**. A synthetic Err would fire those **10 seconds after** the user pressed F2 — a delayed phantom UI on three shipped features (#322/#323). The invariant ("a request always terminates") is genuinely better, but it needs a distinct `Abandoned` signal each consumer handles, not an `Err` cosplay. | Contained fix (F6) now; **follow-up #332 filed** for the general hole. |
| F7 | LOW | `label_text` rejected an empty parts array but ACCEPTED an empty bare string → an invisible phantom that still takes a row slot and a sort key. | **REAL** | `Value::String(s) if !s.is_empty()`. The check is on both shapes or neither. |
| F8 | LOW | The hand-lexer fallback lexes `layout.display`, phantoms included — a hint carrying `"` or `'` (a lifetime: `: &'a str`) can bleed lexer state RIGHTWARD into real code. | **REAL but NARROW — accepted + documented, not fixed.** Traced the reachable set honestly: after an edit BOTH caches invalidate together (the inlay cache is keyed on version, the syntax cache on nonce), and F4 closes non-Rust entirely — leaving only a Rust file with an exotic line separator (bare `\r`/FF/NEL/LS/PS → `lines.get(row)` → None → hand lexer forever). F2 already guarantees the phantom's OWN cells paint Hint; only a bleed to its right remains, and it is cosmetic (the layout/caret/click stay correct). Fixing it properly couples the hint render to the syntax cache — a real entanglement for a rare cosmetic case. | Documented; **follow-up #333**. |
| F9 | LOW | The #330 sticky-header row renders without hints (`line_layout`, no inlays). | **NOT A DEFECT — deliberate, now recorded.** The pinned band is a compact context header with no caret and no click→offset map; phantom text there is noise. Out of #331's REQ set. | None. |
| F10 | LOW | `inlay_hints_for_active` clones every hint `String` per frame. | **REAL but ACCEPTED** — confirmed once per FRAME (captured beside `egit`/`ediag` before the `'static` closure), not per row. Hundreds of small allocs/frame; `Rc<str>` if profiling ever says so. | None. |

**Rejected as unfounded (all empirically):** the click contract (every column inside a phantom returns a
BUFFER offset — `f(5)=4, f(6.9)=4, f(7)=5, f(10)=5`; the midpoint tie resolves LATER per the documented rule);
double-emit at BOL/empty (structurally impossible — the loop covers `0..n_chars`, the EOL emit `n_chars`,
disjoint); out-of-range anchors (silently ignored, `col_starts` pristine, no panic); two hints at one anchor
(both emit in slice order — `sort_by_key` is stable); tab adjacency (the tab stop advances over the
ACCUMULATED column — `"a\tb"`+`"XX"`@1 → `"aXX b"`, reaching the same stop as the baseline); wide glyphs
(`char_width` measures phantom and glyph alike); the empty-slice identity (holds over the corpus × tab widths
{1,4,8}, including the two NEW fields under the derived `PartialEq`); the span domain (`hint_spans`, the
tree-sitter feed and the fallback are all display-byte — the HIGH was an end-boundary error WITHIN the shared
domain, not a domain mismatch); `parse_inlay_hints` (ZERO `as u32` — #312 honoured; dropping a label-less hint
is correct per LSP 3.17, where `position` and `label` are both REQUIRED; an unknown kind still renders, since
`kind` only classifies); the stale gate (re-checks nonce+version against the LIVE editor, so a stranded cache
is a silent-ABSENCE bug, never a wrong-position one); the OFF path (byte-identical to pre-#331).

**Verification after the fixes:** `cargo check --workspace` clean; `cargo nextest run -p marley -p marley_lsp`
→ **749 passed, 2 skipped** (no regression from the EOL reorder or the two new fields — the empty-slice path
is untouched by construction); clippy clean; `cargo fmt --all`; §20 scrub `grep -rniwE 'warp|zed' crates
--include='*.rs'` → EMPTY. Mutation surface re-listed on the ACTUAL code (never guessed): `col_of_span_end`
exposes exactly 2 viable mutants (`→0`, `→1`), both killed by any end-column assert ≠ 0,1; `.max(c0)` adds
none (method calls are unmutated); app.rs and lsp_host.rs list **0** inlay mutants (every shim skipped,
including the two new host queries), and the skip-detach trap was checked — the fns neighbouring my
`lsp_host.rs` insertion still list 0.

**Validate owes (carried forward):** the truth table must assert **`col_starts`, not just `display`** — on a
tab-free line `col` never feeds `display` (only the tab branch reads it), so the `+=`→`-=`/`*=` mutants leave
`display` byte-identical and corrupt only `col_starts`; a display-only assertion lets both survive.
`col_of_offset(5) == 10` is the only thing that kills them — the mutation gate FORCES the assertion that
proves the invariant. Also owed: `token_color_maps_each_kind` needs its `Hint` line; kind `1`→Type and
`2`→Parameter must be asserted DISTINCTLY (deleting arm 1 falls through to `_ => None`, so "kind is Some"
won't catch it); `u32_at` needs a hint at a line AND character that is neither 0 nor 1.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**Tests added — 25 (11 code_view pure · 10 marley_lsp pure · 1 rpc arm · 1 token_color line · 2 headless).**

| REQ | Test | Result |
|---|---|---|
| 001 | `t331_inlays_consume_columns_without_col_starts_entries` — display + `col_starts` + `hint_spans`, `len == n_chars+1` | PASS |
| 001b | `t331_layout_invariant_col_starts_matches_display_accumulation` — pins THE named invariant over 4 line/hint shapes | PASS |
| 002 | `t331_empty_inlays_is_byte_identical_to_line_layout` — 8 lines × tab widths {1,4,8}, whole-struct (all 4 fields) | PASS |
| 003 | `t331_col_of_offset_lands_after_a_phantom_at_that_char` | PASS |
| 004 | `t331_click_inside_a_phantom_snaps_to_an_anchor` — 6 columns incl. the exact midpoint tie | PASS |
| 005 | `t331_layout_truth_table` — BOL / adjacent / TAB / wide glyph / out-of-range | PASS |
| 005b | `t331_raw_span_to_display_bytes_maps_code_spans_past_a_phantom` → `13..14` == `"1"` | PASS |
| F1 | `t331_code_span_end_stops_before_an_end_anchored_phantom` — the chaining-hint swallow: Str `8..15`, NOT `8..20` | PASS |
| F1 | `t331_col_of_span_end_table` — 6 rows + the empty-line no-panic + the `.max(c0)` empty-span guard | PASS |
| F1b | `t331_code_spans_hug_code_while_caret_ranges_track_the_caret` — BOTH semantics on one range | PASS |
| F3 | `t331_eol_hint_leaves_the_caret_on_the_code_side` — End at col 10 not 15; empty line at col 0 not 1 | PASS |
| 006 | `parse_bare_string_label` · `parse_label_parts_concatenated` · `parse_kind_table` (Type/Parameter DISTINCT + unknown-still-renders) · `parse_padding_flags` · `parse_skips_malformed_keeps_the_rest` · `parse_non_array_is_empty` · `parse_skips_oversized_position_never_truncates` (2^32 line AND character, negative, missing key, wrong type) | PASS |
| F7 | `parse_rejects_every_empty_label_shape` — `""` / `[]` / `[{value:""}]` / `[{novalue}]` → None; non-empty still parses | PASS |
| 006b | `inlay_hint_params_shape` · `inlay_hint_support_table` (object AND bool forms SEPARATELY → kills `\|\|`→`&&`; `false` vs absent → kills `==`→`!=`) | PASS |
| 008 | `reply_policy_table` gains `workspace/inlayHint/refresh` → `AckNull` | PASS |
| 009 | `token_color_maps_each_kind` gains its owed `Hint` → `muted` line | PASS |
| 007 | `inlay_stale_reply_dropped_headless` — a superseded key DROPPED; the live key applies + caches, projected to char 9 | PASS |
| 010 | `inlay_toggle_off_uses_empty_slice_headless` — ON places on row 1; OFF empties AND the layout `==` `line_layout` | PASS |
| 010 | `settings_round_trip_survives_reload` NON-DEFAULT leg — confirmed present (kills `persist_inlay_hints → Ok(())`) | PASS |

**Runs (actual):** `cargo nextest run --workspace` → **1460 passed, 5 skipped**. `cargo test --workspace --doc` → 0 tests (no doctests in scope). `scripts/gates.sh --diff` → **GATE GREEN [diff], 15 passed / 0 failed** — mutation **34 caught / 0 missed → MSI 100.0%**, coverage 100% lines, `inlay.rs` 100% regions/functions/lines.

**Three gate reds, all fixed at source:**
1. **gate:2 clippy** — `single_range_in_vec_init` on 5 `assert_eq!(l.hint_spans(), [5..10])`. Clippy's two suggestions were both WRONG for the intent (`(5..10).collect::<Vec<usize>>()` and `[0; 1]` are arrays of *usize*; `hint_spans()` returns `&[Range<usize>]`) — but the lint's premise was right: the literal genuinely reads two ways. Fixed by naming the intent (`one_span(5..10)`), not by suppressing.
2. **gate:4 coverage** — exactly ONE missed line workspace-wide, in `inlay.rs`: (a) `u32_at`'s `obj.get(key)?` None path — every fixture supplied both keys, so a position MISSING `line`/`character` was never exercised → added those cases (+ a wrong-type case); (b) **my own test helper's `if let (Some(o), Some(e))` else-arm**, unreachable because every caller passed an object. Same class as the #328 lesson: a defensive branch in a test helper is an uncovered cold arm. Fixed by REMOVING the branch (build the object by insertion), not by testing the unreachable.
3. **gate:5 mutation — a FLAKY BASELINE, not survivors (#334 filed).** Two runs failed with `exit 4 = baseline failed`, each on a DIFFERENT pre-existing headless test (`new_tab_inherits_live_cwd_headless`, then `cmd_slash_is_a_true_noop…` panicking at `app.rs:995` `spawn_session(…).expect("spawn zsh session")`). Same root: every headless boot spawns a REAL zsh PTY, and `cargo mutants` drives plain `cargo test` (no `--test-tool=nextest` anywhere in the gate), so ~599 lib tests run as parallel THREADS in one process — where gate:3 uses nextest's process-per-test. Under load the spawns contend: one shell missed the test's 5s wall-clock cwd poll, the other failed to spawn at all. **Evidence it is environmental, not #331:** the diff touches no session/terminal/cwd file; the tests pass 5/5 in isolation; `nextest --workspace` is 1460/1460; and the same mutation run had already completed clean once. It went green the moment load fell 8.17 → 4.40 (**EXIT 0, baseline Success, 34 caught / 0 missed**). Not retried-until-green — diagnosed, then re-run. **#334** carries the real fix (the wall-clock bound / test-tool mismatch); it can non-deterministically BLOCK the commit gate, since a failed baseline correctly fails CLOSED.

**LIVE pixel drive — NOT DONE, and the reason CHANGED mid-phase. This is the phase's one gap.**
Every M21 ticket before this deferred the live drive to units+mechanism because the screen was LOCKED. It no longer is: `CGSSessionScreenIsLocked` is absent and `kCGSSessionOnConsoleKey` is true, and a full-screen capture confirmed a live desktop. **But two captures 32 minutes apart showed the frontmost app change (iTerm2 → ChatGPT/Codex) with an active "Pursuing goal … 3h 57m" session — chad is BACK and working on this machine.** Synthetic input lands on the FRONTMOST window at the screen point (`PR-claude-selftest-focus-marley-before-driving-input`, learned the hard way when a drive hit Warp instead of Marley), so `drive.swift focus … type:` would steal focus from and type into his live session. That is not a cost worth a screenshot. So: **deferred, not blocked, and not silently skipped** — REQ-009's *placement* and the OFF-identity are proven by the two headless drives + the pure property; what remains unproven by pixels is only that the muted tone READS as muted against the theme. It needs ~2 minutes on an idle box: bundle → open a small cargo fixture with `let s = String::new();` → wait for rust-analyzer → capture → read. Worth doing before the M21 batch is called done.

**Pre-existing, not in scope:** #334's flake (above). `block v0.1.6` future-incompat warning (an upstream dep, workspace-wide, untouched).

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

**Docs (§21):** CHANGELOG entry above #330 (the pure layout + both invariants + the EOL asymmetry + the
required Hint token + hints-first ordering + the parse seam + the request loop + the two gates +
`has_pending_inlay` + the setting/verb + the v1 cuts). `docs/marley_architecture/editor.md` — a full "LSP
inlay hints (#331, M21) SHIP" section after the #330 sticky-header one, carrying the worked example (squiggle
ends at col 5, selection at col 10, both correct), the reachability that made the HIGH real (chaining hints
are default-on and EOL-anchored, where a literal DOES have a span ending at the anchor), the poll-vs-event
latch asymmetry, and the #333 limit. `crate-map.md` — `marley_lsp` gains the #331 inlay payload seam;
`marley_app` gains `code_view`'s `col_ends`/`col_of_span_end` end-side map + `editor_inlay.rs`'s `InlayKey`.

**Knowledge captured:** AAR `040d8d7f` submitted (completed, effectiveness 5, **9 novel findings**) with 3
failures + 5 prevention rules + 1 architecture decision. The ADR is the one that matters:
**`AD-claude-two-boundary-maps-for-phantom-text-001`** — phantom text forks the buffer→display map into TWO
boundary maps, and a future refactor will WANT to unify `col_of_offset`/`col_of_span_end` and must not
(unifying is wrong in BOTH directions: point selection at the code map and the band detaches from its own
caret by the phantom's width; point a token at the caret map and it swallows the hint). Recorded as binding on
#316 (its first `Identifier`/`Type` capture puts the most common hint in the language on a span's end
boundary), #333, and #305 (folding = a second phantom source). Also recorded: no compile-time guard exists — a
`CodeCol`/`CaretCol` newtype pair would give one and was deliberately NOT done; revisit if a third phantom
source lands.

**The five lessons worth carrying:**
1. **Naming an invariant at design was necessary but not sufficient.** "One column domain" was right,
   implemented right, and verified exhaustively — and still left TWO boundary semantics unaddressed. Two
   critics found it independently from opposite ends (the math, and reachability), which is what made it
   credible rather than clever. An invariant that holds can still be incomplete.
2. **A critic's evidence for the DEFECT and for its REMEDY have different depths** — the remedy is where it
   stopped looking. Critic 2 traced the timeout wedge exhaustively, then asserted its shared-plumbing fix left
   siblings "unaffected". Reading the three sibling `Err` arms it never opened disproved that in minutes
   (delayed "Cannot rename this" toasts 10s after F2). Take the contained fix, file the general invariant with
   its blast radius named (#332).
3. **A defensive branch in a TEST HELPER is an uncovered cold arm** — the #328 class, hit again. My `if let
   (Some, Some)` merge could never fail because every caller passed an object. Remove the branch; don't test
   the unreachable.
4. **The mutation gate forces the assertion that proves the invariant.** On a tab-free line `col` never feeds
   `display`, so `*col +=` → `-=`/`*=` leave `display` byte-identical and corrupt only `col_starts`. A
   display-only truth table lets both live; `col_of_offset(5) == 10` is the only thing that kills them.
5. **A flaky mutation BASELINE fails closed and blocks the commit gate** — diagnose, never retry-until-green.
   Root cause was environmental (every headless boot spawns a real zsh; `cargo mutants` drives plain `cargo
   test`, so ~599 tests run as parallel THREADS in one process where gate:3 uses nextest's process-per-test;
   under the user's own agent load the spawns contend). Proven environmental by four independent facts before
   re-running, and it went green when load fell 8.17 → 4.40. #334 carries the fix.

**Follow-ups filed:** #332 (a typed `Abandoned` signal per consumer — with the naive synthetic-`Err` version
explicitly ruled out and why), #333 (the hand-lexer phantom bleed — narrow, cosmetic, with the tab-expansion
reason a remap won't work), #334 (the load-flaky mutation baseline).

**Outstanding — the honest gap:** the LIVE pixel drive was NOT run. Not the old lock block (the screen is
unlocked now) — chad is BACK at the machine and actively working (two captures 32 min apart show the frontmost
app change into a live Codex session), and synthetic input lands on the frontmost window, so driving would type
into his session. REQ-009's placement + the OFF-identity are proven by the two headless drives and the pure
property; only "the muted tone reads as muted" wants pixels. ~2 minutes on an idle box.

status: Phase 5 — Complete PASS
