# 312 — LSP go-to-definition + NavStack — notes

## Phase 1 — Plan

### Intent
F12 on a symbol → `textDocument/definition` → open its file (cross-file), jump the caret, center; ⌃-
returns via a NavStack; multiple results open a picker. The #311 general request/response path's second
consumer. Quiet flash on not-found/not-ready.

### Discovery (Explore) — landing zones + net-new
- **REUSE AS-IS:** `open_file_at(path, line, col)` (`app.rs:2886`, the #212 terminal-ref path: opens via
  `open_file_in_viewer` + `caret_for_line_col` + `set_single_caret`); `scroll_editor_to_row(row)`
  (`app.rs:6152`, `ScrollStrategy::Center`); `offset_for_click` (#254); `Flash::new`/`status_flash`
  (`flash.rs`); the whole #311 spine (`RequestPurpose`, `request`, `take_responses`, `uri_for`,
  `encoding`, `push_response_for_test`); `offset_to_position`/`position_to_offset`/`file_uri`/
  `path_from_file_uri`; `Keymap::action_for` + `KeyContext::Editor`.
- **THE #273 TRAP (load-bearing):** `open_file_at` sets the caret but does NOT center, and an open +
  same-frame `scroll_editor_to_row` is WIPED by `sync_editor_scroll` (`app.rs:6259`) when the new file
  takes the shared scroll handle. The proven fix is TWO frames (open, `run_until_parked`, then scroll —
  `headless_drive.rs:764`). → NET-NEW: a `pending_center_row` parked on the view, consumed a frame later.
- **NavStack DOES NOT EXIST** — `nav.rs` is terminal block-nav (a false friend); no editor jump history
  anywhere. Entirely net-new: a pure `Vec<(PathBuf, CharOffset)>` (cap 50, dedupe same-spot).
- **No (label, location) picker** — `FinderState` (files) + `PaletteState` (commands) are BOTH
  hardwired; the overlay is an inline `app.rs:9623` block, copy-pasted per overlay. NET-NEW: a picker
  state mirroring the `move_up`/`move_down`/`selected` idiom + a new overlay block.
- **⌘-click COLLISION:** the editor mouse-down `cmd_click` branch (`app.rs:3531/3548`) is multi-cursor
  toggle. So click-to-jump collides with a shipped gesture → **DEFERRED (D2, chad chose F12-only)**.
- **anchor.rs** (`anchor_at`/`resolve_anchor`, `buffer.rs:220/233`) is built + tested but DORMANT (no
  callers). Optional for #312 (targets resolve over the synced buffer); a follow-up if a jump must
  survive edits between request and response.
- **Keymap:** F12 = `chord(false,false,false,false,"f12")`; ⌃- = `chord(false,true,false,false,"-")`;
  add as `Some(KeyContext::Editor)` rows + `dispatch_action` arms (mirror ⌘K #311). (⌘⇧O is already a
  global chord — a future go-to-symbol row must be Editor-scoped to shadow it.)

### Classification
Work pipeline, feature, M20. One shippable slice: F12 goto + jump-back + picker + normalization. The
click gesture + ⌘⇧O/⌃G push sources + anchors are OUT (deferred, see spec).

### Reference (§20)
Zed (editor) — the F12/⌃- go-to-definition + jump-back triad, matched via the LSP 3.17
`textDocument/definition` wire + Marley's own open/scroll/NavStack. No GPL source read. Filled in spec.

### Risks / decisions
- **R1 — the deferred-center race (#273).** Getting the two-frame open→center right is the highest-risk
  shim; the headless drive must prove the target row is centered after the frame boundary.
- **R2 — the 3-shape response normalization** must not panic on a malformed/partial `LocationLink`
  (missing `targetRange`, a bare `Location`). Pure, defensive, tested per shape.
- **R3 — NavStack cap + dedupe** — a same-spot re-push (F12 twice) must not grow the stack; the cap
  drops the oldest. Pure, pinned.
- **D2 (chad) — F12-only; ⌘-click deferred** (multi-cursor untouched); follow-up owns the click.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture / approach
Second consumer of the #311 request/response spine; two pure layers + a shim.

**1. Pure — `marley_lsp`:**
- NEW `definition.rs`: `DefLocation { uri: String, line: u32, character: u32 }` (the normalized target)
  + `parse_definition_result(&Value) -> Vec<DefLocation>` — hand-parsed (the #310/#311 idiom, no
  lsp-types), normalizing the THREE shapes: a `Location` object (`{uri, range}`) → 1; a `LocationLink`
  object (`{targetUri, targetSelectionRange|targetRange}`) → 1; an ARRAY of either → N (each element
  discriminated by `uri` vs `targetUri`); `null`/absent/malformed → skipped (empty vec). Uses
  `range.start` (or `targetSelectionRange.start`) for the caret line/character.
- `rpc.rs`: NEW `text_document_position_params(uri, line, character) -> Value` — the shared
  `{textDocument:{uri}, position:{line,character}}` every position request uses. `hover.rs`'s
  `hover_request_params` DELEGATES to it (DRY; #313/#317 reuse it too).

**2. Pure — `marley_app` NEW `editor_nav.rs`:**
- `NavLoc { path: PathBuf, offset: CharOffset }`.
- `NavStack { stack: Vec<NavLoc> }` — `push` (dedupe: a re-push equal to the top is a no-op; cap 50 by
  dropping the FRONT/oldest), `pop() -> Option<NavLoc>`, `len`/`is_empty`.
- `DefPicker { items: Vec<(String, DefLocation)>, selected: usize }` — `move_up`/`move_down` (wrap),
  `chosen() -> Option<&(String, DefLocation)>`, mirroring `FinderState`'s idiom (the picker STATE is
  pure; the overlay render is shim).
- `DefinitionKey { uri: String, line: u32, character: u32 }` — the stale-guard key (a newer F12 or a
  move drops an older response), mirroring `HoverKey` (no version — a definition target doesn't go
  stale on an edit).

**3. Shim — `marley_app`:**
- `lsp_host.rs`: `RequestPurpose::Definition(DefinitionKey)` variant.
- `app.rs`:
  - `request_definition_at_caret` (F12): build the `DefinitionKey` + `text_document_position_params` →
    `host.request("textDocument/definition", …, Definition(key))` → store `definition_request` (mirror
    `request_hover`).
  - `consume_definition_responses(root)` (pump, next to `consume_hover_responses`): drain
    `take_responses()`; match `Definition(key)`; stale-guard (`key == definition_request`);
    `parse_definition_result` → **0 → a quiet `Flash` "no definition found"; 1 → `jump_to_definition`;
    N → open the `DefPicker`**.
  - `jump_to_definition(loc)`: push the CURRENT (path, caret) to the `NavStack`; `open_file_in_viewer`
    the target (`path_from_file_uri`); map the caret via `position_to_offset(target_text, loc.line,
    loc.character, enc)` → `set_single_caret` (encoding-correct — NOT `caret_for_line_col`, which takes
    a 1-based CHAR col, wrong for a UTF-16 target); park `pending_center_row = Some(loc.line)`.
  - **`pending_center_row: Option<usize>`** consumed at the START of the LSP pump section (BEFORE
    `consume_definition_responses`) → a guaranteed 1-TICK delay so the open's `sync_editor_scroll` (the
    #273 wipe) has already run when `scroll_editor_to_row(row)` fires (the `headless_drive.rs:764`
    two-frame pattern, provided by the pump).
  - `nav_back` (⌃-): `NavStack::pop()` → `open_file_in_viewer` + `set_single_caret(loc.offset)` + park
    the center row (does NOT push).
  - the `DefPicker` overlay (mirror the `app.rs:9623` finder block, `path:line` rows) + its up/down/
    enter/esc handling in `on_key_down` (like the finder); Enter → `chosen()` → `jump_to_definition`.
- `keymap.rs`: `(f12, "go-to-definition", Editor)` + `(ctrl+-, "nav-back", Editor)`; `dispatch_action`
  arms. Update the `all_chords` roster guard (+2 → 54 chords, 13 scoped).
- `lib.rs`: `mod editor_nav;`.

### §20 confirmation
Zed (editor). MATCHES the observed F12 → definition-across-files + ⌃- jump-back + multi-result picker
behavior via the LSP 3.17 `textDocument/definition` wire + Marley's own `open_file_in_viewer`/
`position_to_offset`/`scroll_editor_to_row`/`NavStack`. No GPL source read; the 3-shape normalization is
from the published spec. Confirmed clean-room.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/definition.rs` | NEW pure: `DefLocation`, `parse_definition_result`. |
| `crates/marley_lsp/src/rpc.rs` | NEW `text_document_position_params`. |
| `crates/marley_lsp/src/hover.rs` | `hover_request_params` delegates to `text_document_position_params` (DRY). |
| `crates/marley_lsp/src/lib.rs` | export `definition` items + `text_document_position_params`. |
| `crates/marley_app/src/editor_nav.rs` | NEW pure: `NavLoc`, `NavStack`, `DefPicker`, `DefinitionKey`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::Definition(DefinitionKey)`. |
| `crates/marley_app/src/keymap.rs` | F12 + ⌃- Editor rows; roster guard bump. |
| `crates/marley_app/src/app.rs` | request/consume/jump/nav-back shims, `pending_center_row` + its 1-tick consume, `def_picker` + overlay + key handling, dispatch arms, `definition_request` field, `mod` decl consumers. |
| `crates/marley_app/src/lib.rs` | `mod editor_nav;`. |

### Regression Test Plan (≥1 per REQ)
| Test (loc) | REQ | Asserts |
|---|---|---|
| `rpc::tests::text_document_position_params_shape` | 001 | the `{textDocument,position}` JSON. |
| `definition::tests::parse_*` (Location / Location[] / LocationLink[] / empty+null+malformed) | 002/006 | each shape → the right `Vec<DefLocation>`; empty/null/partial → `[]`, no panic. |
| `editor_nav::tests::nav_stack_push_pop_cap_dedupe` | 005 | push→pop LIFO; a same-top re-push is a no-op; the 51st push drops the oldest. |
| `editor_nav::tests::def_picker_move_and_chosen` | 004 | move_up/down wrap; `chosen()` returns the selected item. |
| `position::tests::*` (EXISTING #309 `position_to_offset`) | 007 | reused — the encoded (line,char) → offset map. |
| `headless_drive::lsp_goto_definition_jumps_and_navstack_headless` | 001/003/005/007 | feed a synthetic 1-location definition response → assert the caret lands at the target offset + the NavStack pushed the origin; `nav-back` returns; a multi-location response opens the `DefPicker`; an empty response flashes + moves nothing. |
| `keymap::tests::all_chords_lists_every_binding` (extend) | 001/005 | F12 + ⌃- present; counts 54 / 13. |

**Phase 3.5 additions to this plan (BINDING on Phase 4 — see the Inspect ledger below):**

| Test (loc) | Why | Asserts |
|---|---|---|
| `app::tests::def_label_*` | **MSI floor 100** — `def_label` is the one new app.rs fn with no `mutants::skip`; its 2 mutants (`String::new()`, `"xyzzy".into()`) survive the planned picker drive. Precedent: `token_color`. | BOTH arms + the LITERAL string (`loc.line + 1` is inside `format!` → NO mutant guards the 0→1-basing): a `file:` uri → `"/x/m.rs:4"`; a non-`file:` uri → the raw-uri fallback. |
| `definition::tests::parse_*` (position choice) | Kills `start_of`'s four `Some((0|1,0|1))` mutants — the "sneaky `Some(1)`" trap. | Use a position neither 0 nor 1 in BOTH fields (e.g. `line: 5, character: 2`), PLUS a present-but-PARTIAL range (`{"range":{"start":{"line":5}}}` → `[]`). |
| `definition::tests::parse_location_link_malformed_selection_falls_back` | Inspect F9 | a `LocationLink` whose `targetSelectionRange` is `null`/a string but whose `targetRange` is valid → the target from `targetRange` (not `[]`). |
| `definition::tests::parse_line_out_of_u32_range_is_skipped` | Inspect F10 | `line: 2^32` → `[]` (a clean skip, NOT a truncation to line 0). |
| `headless_drive::lsp_goto_definition_stale_offset_nav_back_headless` | **Inspect F1 (HIGH)** | push an origin, SHRINK the buffer below that offset, ⌃- → NO panic; the caret clamps to EOF. |
| `headless_drive::lsp_goto_definition_unopenable_target_headless` | **Inspect F2 (HIGH)** | a definition response naming a target that cannot open → the caret and the NavStack are BOTH untouched (nothing moved in the origin file). |
| `headless_drive::lsp_goto_definition_moved_on_response_headless` | Inspect F4 | switch the focused file between F12 and the response → the answer is DROPPED (no jump, no NavStack push). |

Uncoverable-by-unit: the pixel center + a live rust-analyzer definition round-trip (orchestration-blocked
per #310); the headless drive + the pure seams + `scroll_editor_to_row`'s existing proof carry it. The
deferred-center 1-tick timing is proven in the headless drive (open → park → tick → centered row).

### Risks / decisions
- **R1 (highest) — the deferred-center race (#273).** `pending_center_row` MUST be consumed a tick
  AFTER it's set (place its consume BEFORE `consume_definition_responses` in the pump) or the open's
  `sync_editor_scroll` wipes it. The headless drive pins the two-frame ordering.
- **R2 — `parse_definition_result` defensiveness.** A partial `LocationLink` (no `targetRange`), a bare
  string, a number → skipped, never a panic. Tested per-shape.
- **R3 — NavStack cap + dedupe** — a same-spot F12×2 must not grow the stack; the 51st push drops the
  front. Pure, pinned.
- **D-des-1 — the jump uses `position_to_offset` + `set_single_caret`, not `open_file_at`'s
  `caret_for_line_col`** — the LSP `character` is an ENCODED column, not a 1-based char col.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`crates/marley_lsp/src/definition.rs` (NEW, pure)** — `DefLocation{uri,line,character}` +
  `parse_definition_result` (hand-parsed; discriminates `Location` [`uri`+`range`] from `LocationLink`
  [`targetUri` + `targetSelectionRange` preferred over `targetRange` — lands on the SYMBOL, not the doc
  comment]; an ARRAY maps each element; a malformed/partial element is SKIPPED via `?`, so a bad
  payload → an EMPTY vec, never a panic).
- **`crates/marley_lsp/src/rpc.rs`** — NEW `text_document_position_params` (the shape every position
  request shares); **`hover.rs::hover_request_params` now DELEGATES to it** (DRY; #313/#317 reuse).
- **`crates/marley_lsp/src/lib.rs`** — export `definition` items + `text_document_position_params`.
- **`crates/marley_app/src/editor_nav.rs` (NEW, pure)** — `NavLoc`, `NavStack` (push with same-top
  DEDUPE + `NAV_STACK_CAP = 50` dropping the OLDEST, pop), `DefPicker` (items + selected, wrapping
  move_up/move_down, chosen), `DefinitionKey` (uri+line+character — no version: an edit doesn't move
  where a symbol is DEFINED).
- **`crates/marley_app/src/lsp_host.rs`** — `RequestPurpose::Definition(DefinitionKey)`.
- **`crates/marley_app/src/keymap.rs`** — `(f12, "go-to-definition", Editor)` + `(⌃-, "nav-back",
  Editor)`; roster guard 52→54 chords, 11→13 scoped.
- **`crates/marley_app/src/app.rs`** — 4 fields (`nav_stack`, `def_picker`, `definition_request`,
  `pending_center_row`); `go_to_definition` (F12 send + a "LSP: not ready" flash when the host isn't
  Ready), `apply_definition_response` (stale-drop / 0→flash / 1→jump / N→picker),
  `jump_to_definition` (NavStack push → open → caret via `position_to_offset` → park the center row),
  `place_caret_at_position` (shared by jump + nav-back), `nav_back` (⌃- pop → open + caret + park;
  empty → flash), `consume_pending_center` (the deferred scroll), `def_label` (`path:line`),
  `def_picker_overlay` (the #221 recipe, `menu_origin`-clamped, 20 rows) + the picker's on_key_down
  branch (↑/↓/Enter/Esc own the keyboard), the dispatch arms, and the pump wiring.
- **`crates/marley_app/src/lib.rs`** — `mod editor_nav;`.

### Deviations from design (with reason)
- **ONE drain, routed by purpose — `consume_hover_responses` → `consume_lsp_responses`** (+ the hover
  body extracted to `apply_hover_response`). The design said "add `consume_definition_responses` next
  to `consume_hover_responses`", but `take_responses()` empties the WHOLE queue — two per-feature
  drains would SWALLOW each other's answers (whichever ran first would discard the other's). The router
  is the correct shape and is what every later purpose (#313/#317) plugs into. (Adding the second
  `RequestPurpose` variant also made the old irrefutable `let RequestPurpose::Hover(key) = purpose`
  refutable — the match was forced anyway, which is how the bug surfaced.)
- **`NavStack::len`/`is_empty` removed** — nothing wired them (clippy dead-code); `pop() -> None` is
  what the shim needs for its flash. The #311-F3 lesson: no unwired API. The cap/dedupe test asserts
  via pops.
- **A "LSP: not ready" flash on a `None` from `request()`** — the design left REQ-006's not-ready half
  implicit; F12 must not feel dead.

### Verified
- `cargo check --workspace` + `cargo check -p marley --all-targets` clean; `cargo clippy -p marley_lsp
  -p marley --all-targets` clean; `cargo fmt --all --check` clean.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

5 independent critics over the staged diff (~845 LOC): shim-correctness, pure-seams+REQ, data/state
integrity, simplification/reuse, security/provenance/panic-safety. Each was briefed with the prior
forge traps for this subsystem (the #310 `external-uri-key-normalize-both-sides` bug, the #311
`transient-overlay-dismiss-poll-live-editor-identity` bug, `BF-lsp-hover-extracted-helper-new-mutation-surface`,
the `mutants::skip` detach trap) and told that a false positive costs more than a miss. **Three critics
independently found the SAME two crashes** — the convergence is what promoted them to HIGH. Every
finding below was re-verified by the lead against the real code before it was accepted or rejected.

### REAL — fixed in #312

| # | Sev | Finding | Fix |
|---|---|---|---|
| **F1** | **HIGH** | **`nav_back` PANICS the app on a stale NavStack offset.** `nav_back` fed the RAW popped offset to `Buffer::line_col`, which is an unguarded `rope.char_to_line` — ropey panics out of bounds. (Its sibling `line_start` two lines below explicitly clamps + documents "never panics", so clamping here is opt-in and this call did not opt in.) `set_single_caret` clamps, so the caret was safe; the ROW read was not. Reachable by ordinary use: `EditorSurface::open` PRESERVES an already-open buffer (M15 #249), so F12 → jump → return → delete a block → ⌃- pops a now-past-EOF offset → **crash on the UI thread**. One critic reproduced it against real ropey; a second reproduced it with a probe in-tree. | Structural: the new `open_and_place_caret` reads the row back from `active_caret()` AFTER the clamp, never from the requested offset. **Lead-verified by probe:** raw 4000 → ropey panics; clamped → `CharOffset(11)` → `row=1`, and 11 == `len_chars`, so the exact boundary is proven in-bounds too. Probe removed (`buffer.rs` byte-identical to HEAD). |
| **F2** | **HIGH** | **A failed open teleports the caret into the WRONG file.** `open_file_in_viewer` is `if let Some(..) = load_code_view_state(path)` with **no else** — it no-ops on a missing / non-file / >2MB / binary target and leaves the CURRENT file active. Both new callers then set the caret at the TARGET's offset and parked its row — silently, in the file the user was actually reading (`position_to_offset` clamps, so no panic to signal it). Reachable and common: rust-analyzer resolves into >2MB generated sources (a critic found real >2MB `.rs` files in this machine's `~/.cargo/registry`); the loader even flashes "can't open …(>2MB)" *while the caret still moves*. | `open_and_place_caret` VERIFIES the landing (`active_file().path == path`) before touching the caret, and returns `false` otherwise → a flash, nothing moved. `jump_to_definition` now pushes the NavStack only AFTER a confirmed landing (it reads the origin before the open, pushes after), so a failed jump leaves no bogus return entry. |
| **F3** | MED | **REQ-006 unmet: F12 is SILENTLY dead when the root has no host.** Only the `request() -> None` branch flashed. But `ensure_lsp_for_opened_file` spawns only for a `.rs` file under a root with a `Cargo.toml` — so F12 in a README, or anywhere in a non-Cargo project, hits an EMPTY `lsp_hosts` and returned in silence. No-host ⊃ not-Ready, and it is the *more common* case. (Two critics traced the other early-returns as unreachable: `active_editor()` is `Some` because the keymap row is Editor-scoped, and `uri_for` cannot fail on an absolute path.) | Split `go_to_definition` → `send_definition_request() -> bool`, so EVERY not-sent road lands on ONE flash site. |
| **F4** | MED | **`apply_definition_response` had no live-identity guard** — its hover sibling drops an answer whose file changed (#311 inspect F1), this acted unconditionally. Two consequences, both verified: a slow answer (10s timeout; r-a takes seconds while indexing) YANKS the user out of whatever they moved on to; and from a TERMINAL tab there is no `active_editor()` for the NavStack push to read, so the jump lands with an EMPTY stack and **⌃- cannot undo it (REQ-005)**. Also lets the picker steal a focused finder's keyboard. | Mirror hover: drop the response when the focused file's uri ≠ `key.uri`. One guard fixes all three facets. |
| **F5** | MED | **`def_picker` missing from `text_input_blocked`** (the #267 one-predicate choke point, whose own doc says "a NEW overlay only needs a line here"). The critic checked whether `stop_propagation` already covers it — it covers the common path, so the naive "typing corrupts the buffer" bug does NOT exist. The residual hole is real though: gpui's mac `window.rs` computes `is_composing` from `marked_text_range()` (exactly what this predicate forces to `None`) and routes non-printing keys (↑/↓/Esc — the picker's own keys) to `handleEvent:` BEFORE `run_callback`. Every shipped picker is listed; this one wasn't. | `|| self.def_picker.is_some()`. |
| **F6** | MED | **`go_to_definition` re-implemented `request_hover`'s prologue** — ~20 of ~30 lines byte-identical; the `enc` lookup went 2× → 4×. This diff authored the duplicate, and #313/#317 are both position requests that would copy it a third and fourth time. | Extracted `lsp_position_for(root, caret) -> Option<(uri, Position)>` — the shim twin of this diff's own `text_document_position_params` extract. Both hover and definition route through it. Hover regression-checked: `lsp_hover_response_builds_and_drops_headless` + all 572 crate tests green. |
| **F7** | MED | **The picker's selected row abandoned the shipped accent idiom** — `bg(background)` on a `bg(surface)` card (ΔL 0.06 dark / 0.04 light) where palette/finder/history all use `bg(accent) + on_accent`. The #219 lesson's neighborhood (a weak fill reads as no highlight). | `row.bg(colors.accent).text_color(colors.on_accent)`. |
| **F8** | LOW | **The picker selection could wrap past the rendered rows.** Render was `.take(20)`; `move_down` wraps over the FULL `items.len()`. With >20 targets (REQ-004's own motivating case — a trait with many impls) the selection walks into rows that are never drawn, with no scrollbar: Enter jumps to something the user never saw. Found by 4 of 5 critics. | Window the render around the selection (`skip(selected.saturating_sub(MAX_ROWS-1)).take(MAX_ROWS)`) — every target stays reachable and the selected row is always on screen. Rejected the alternative (truncate `items` at construction): that would silently drop targets. |
| **F9** | LOW | **A present-but-malformed `targetSelectionRange` defeated the `targetRange` fallback.** `obj.get(..)` answers `Some` for a `null`/string value, so `.or_else(|| obj.get("targetRange"))` never ran and a valid `targetRange` sitting right beside it was discarded → "No definition found" for a jump the code had the data to make. Critic ran it: `tsr=string, tr ok -> []`. | Fall back on PARSE failure, not just absence: `.and_then(start_of).or_else(|| obj.get("targetRange").and_then(start_of))`. |
| **F10** | LOW | **`as u32` silently TRUNCATED an out-of-range line into a wrong jump.** `as_u64()` admits any non-negative integer, then `as u32` wraps: critics ran it — `line: 5e9 -> 705032704`, `line: 2^32 -> line 0`. That contradicts the module's own documented contract ("a malformed element is SKIPPED… a bad payload returns an EMPTY vec"): every other bad field degrades to "no definition", this one to a confidently-wrong target. | `u32::try_from(..).ok()?` — out-of-range now joins the negative/fractional cases as a clean skip. Verified: adds no new mutants. |
| **F11** | LOW | **`nav_back() -> bool` was unwired API** (the #311-F3 rule this diff's own notes cite when deleting `NavStack::len`/`is_empty`). `dispatch_action` returns `()` and its caller `cx.notify()`s unconditionally, so both `return true`s were dead information. | `fn nav_back(&mut self)`; the arm collapses to `"nav-back" => self.nav_back(),`. |
| **F12** | LOW | **The picker's key routing was inline in the render closure** — its six peers are all extracted `handle_*_key` methods. The one place shim routing leaked into the render altitude. | Extracted `handle_def_picker_key`; the ladder arm is now the house 4-liner. Bonus: gives Phase 4 a callable seam for REQ-004's Enter→jump. |
| **F13** | LOW | **The ⌘⇧A / agent-icon arm didn't clear `def_picker`** — the #229-F1 class. That arm exists *because* the icon's mouse-click bypasses the key router; the picker's arm runs before the launcher's and swallows every key → a keyboard-dead launcher under an invisible-to-the-router picker. | `self.def_picker = None;` alongside the existing `naming_workflow = None`. |
| **F14** | LOW | **`place_caret_at_position`'s doc claimed "Shared by a definition jump and a ⌃- return"** — it had ONE caller; `nav_back` re-implemented it inline, and that fork is exactly where F1's panic lived. | Dissolved into `open_and_place_caret`, which both callers genuinely share — the doc is now true, and the shared tail is what makes F1/F2 unrepeatable by construction. |
| **F15** | LOW | `DefinitionKey.line` documented as "in the negotiated encoding" — a line index is encoding-independent (only `character` is). `DefLocation` got this right; the qualifier was copied one field too far. | Doc corrected. |

### REAL — handed to Phase 4 (test design; no code change)

- **`def_label` has 2 live mutants and no planned test → MSI floor 100 would go RED.** Traced against the
  REAL lists, not guessed: app.rs base (HEAD) = 62 mutants, staged = 64, delta = exactly
  `replace def_label -> String with {String::new(), "xyzzy".into()}`. `def_label` is the one new app.rs fn
  without a `mutants::skip`, and the planned picker drive doesn't kill either (the picker opens with 2
  items regardless of label text). **This is `BF-lsp-hover-extracted-helper-new-mutation-surface-001`
  exactly.** Phase 4 MUST add a direct unit test — precedent: `token_color` (app.rs, an unskipped free fn
  with its own unit). Assert BOTH arms and the literal string: `loc.line + 1` sits inside `format!`, which
  cargo-mutants does not descend into, so the 0→1-based conversion has NO mutant guarding it.
- **`start_of`'s four `Some((0|1,0|1))` mutants need non-0/1 test positions.** The real list (post-F10 fix,
  re-verified) is 9 mutants / 7 viable — `Some(Default::default())` + `vec![Default::default()]` are
  unviable because `DefLocation` has no `Default`. A natural "definition at the top of the file"
  (`line: 0, character: 0`) leaves `Some((0,0))` ALIVE — the "sneaky `Some(1)`" trap. Use a position that
  is neither 0 nor 1 in BOTH fields (e.g. `line: 5, character: 2`), PLUS a present-but-partial range case
  (`{"range":{"start":{"line":5}}}` → `[]`), which kills all four independently.
- **New regression tests the fixes earn:** a stale-offset ⌃- (F1: pop an offset past EOF → no panic, caret
  at EOF); a failed-target jump (F2: an unopenable uri → caret and NavStack both untouched); a moved-on
  response (F4: switch files between F12 and the answer → the answer is dropped, nothing jumps).

### RESIDUAL RISK — recorded, deliberately NOT changed

- **The deferred center is timing-dependent, not gated on the render having run.** `PUMP_INTERVAL_MS = 16`
  equals the frame cadence and nothing orders them; if tick N+1 beat the pending render, `sync_editor_scroll`
  (nonce still stale) would wipe the deferred scroll — the #273 trap re-manifested. **Not changed:** one
  critic verified the pump ordering CLEAN and explicitly declined to report this; the other rated it LOW with
  "could not prove the live ordering… failure mode is cosmetic — the jump lands, the centering doesn't". D3
  is a LOCKED Phase-2 decision with a shipped precedent (`headless_drive.rs:764`), and the proposed
  `scroll_owner`-gate trades this for a park that can stick when the editor is not rendered. **Phase 4's
  headless drive pins the two-frame ordering; revisit only if it flakes.**
- **A DIRTY target buffer can land the caret off-by-some** (the server's (line, character) refer to text the
  buffer no longer has). Explicitly Out of scope per the spec ("Edit-surviving return positions via
  anchors… a follow-up"); `position_to_offset` clamps, so it is a wrong caret, never a crash. Follow-up
  alongside the dormant `anchor.rs`.

### REJECTED (with reason)

- **"Reuse `open_file_at` for the jump."** D-des-1 is SOUND — the critic read both and self-rejected:
  `caret_for_line_col` counts CHARS (`line_text(row).chars().count()`), `position_to_offset` maps ENCODED
  code units. Routing F12 through it would land the caret wrong on any line with non-ASCII before the symbol.
- **"Capture the NavStack origin at REQUEST time and carry it with the key."** Rejected: with F4's liveness
  guard, the only surviving case is "the user moved the caret within the SAME file while waiting" — and
  pushing where they actually were immediately before the jump IS the pre-jump location (REQ-005). The
  file-switch case that genuinely broke is what the guard drops. Carrying the origin would add state for no
  behavioural gain.
- **"Re-push a popped NavStack entry when its file won't open."** Rejected: that wedges ⌃- on a dead file
  forever. Dropping lets a second ⌃- walk further back. Documented as a deliberate decision on `nav_back`.
- **The #310 `external-uri-key-normalize-both-sides` trap is NOT repeated** (the primary suspicion going in).
  Traced: the stale-guard compares OUR key against OUR key — `request()` stores the purpose in `purposes` and
  correlation is by JSON-RPC **id**, never by the server's echoed uri string. `path_from_file_uri(&loc.uri)`
  decodes the target for OPENING only; it is not a key lookup. A space/`@scope`/symlink respelling cannot
  drop a response. (`path_from_file_uri` also verified against real formats: percent-decoding, `file://localhost/`,
  a bad escape → `None`, its own tests pin `/Users/My Name/a@b/c(d),e` and `%40scope`.)
- **The `mutants::skip` DETACH trap did NOT fire** — verified by diffing the mutable-fn set: HEAD's 13 fns,
  staged = those ∪ `{def_label}`, and after my 4 new skip-attributed fns the count is **still 64** with none
  of them present. Every skip binds to its intended function.
- **The `apply_hover_response` extraction is faithful** — two critics diffed it against `git show HEAD`:
  HEAD set `changed = true` AFTER the stale-guard `continue` and BEFORE the two later ones, so
  stale→`return false` / later-continues→`return true` / fall-through→`true` is the exact truth table, and
  `changed |= …` is monotonic like `changed = true`. The router itself is right: `take_responses()` drains the
  WHOLE queue, so two per-feature drains would swallow each other's answers.
- **Out-of-workspace targets WORK** (the main real F12 case): rust-src is installed, `resolve_under_root`
  passes an absolute path through, `load_code_view_state` imposes no root containment, and
  `ensure_lsp_for_opened_file` correctly declines to spawn a second host (`!full.starts_with(&root)`).
- **`NavStack` cap/dedupe (R3) is provably correct** — walked the boundary: push #50 → `49 >= 50` false → len 50;
  push #51 → `remove(0)` → len 50. Never exceeds 50; `>=` is right. Top-only dedupe is what REQ-005 asks (a
  full-stack dedupe would wrongly collapse A→B→A).
- **`parse_definition_result` (R2) is genuinely defensive** — 14 hostile payloads run through the real crate
  (`null`, `[]`, `[null,null]`, bare string, number, bool, nested arrays, every missing/partial field,
  negative + fractional line): all → `[]`, none panic. Skipping-on-partial is the RIGHT call: LSP requires
  both `line` and `character`, so defaulting to 0 would invent data the server never sent.
- **Security lens: one LOW (F10), nothing else.** Trust model stated first — the server is a local binary the
  user configured, running as the user, already able to read any file it names; "we open the file it names"
  is the contract, and traversal grants it nothing it couldn't ask for directly. Verified: the repo's own
  brand gate (`grep -rniwE 'warp|zed' crates --include='*.rs'`) returns nothing (§20 clean-room holds; the
  hand-parse is an independent read of the spec — it FLATTENS to `{uri,line,character}` and discards
  `end`/`originSelectionRange`, which no transcription would); `gitleaks protect --staged` → no leaks;
  `RUSTDOCFLAGS="-D warnings" cargo doc` → exit 0, no doc link points at a private item; serde_json's
  depth-128 limit errors rather than overflowing; non-`file:` schemes → a clean flash, no panic; and
  server-supplied paths cannot reach a spawn — blocked four ways (`.rs`-only, `starts_with(root)`, the binary
  comes from user settings, and a host for that root already exists by definition when a response is handled).

### Follow-up ticket filed
- **#318** — extract the shared overlay-card recipe. `def_picker_overlay` is 15-of-16 builder calls identical
  to `hover_card_overlay` (#311), and the 8-call core repeats verbatim in palette/finder/history. Correctly
  OUT of #312: extracting for one caller is a net loss, and the real win must convert palette/finder/history
  off their hand-rolled `bounds.w * 0.25` positioning onto `menu_origin` — its own blast radius, its own
  driven capture.

### Verified after the fixes
`cargo check` + `cargo clippy -p marley_lsp -p marley -p marley_editor --all-targets` clean; `cargo fmt --all
--check` clean; **741 tests pass** (`marley` + `marley_lsp` + `marley_editor`), including the #311 hover
headless drive that guards the `request_hover` refactor. `cargo mutants --list` re-run on all three touched
files: app.rs still 64 (no skip detached), definition.rs 9/7-viable, editor_nav.rs 18.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests added (24 new; every REQ + every inspect finding)

**Pure — `marley_lsp`:**
- `rpc::tests::text_document_position_params_shape` (REQ-001) — the `{textDocument, position}` wire shape
  four features now fill.
- `definition::tests::*` (REQ-002/006) — 9 tests: each of the three spec shapes (`Location`,
  `Location[]` in order, `LocationLink[]` preferring `targetSelectionRange`, a BARE `LocationLink`
  falling back to `targetRange`); `null`/`[]`/`[null,null]` → no targets; **12 malformed elements**
  skipped, never a panic; a mixed array keeping only the good elements (the reason this crate
  hand-parses instead of `from_value::<Vec<Location>>`, which rejects the whole array on one bad
  element); plus the two inspect edges — F9 (an unparseable `targetSelectionRange` falls back to
  `targetRange` rather than dropping a target the code had the data for) and F10 (`line = 2^32` is
  SKIPPED, not truncated to line 0; `u32::MAX` still fits).
  **Every fixture sits at line 5, character 2 — deliberately neither 0 nor 1 in either field**, which is
  what kills `start_of`'s four `Some((0|1, 0|1))` mutants (the "sneaky `Some(1)`" trap: a natural
  top-of-file fixture would leave `Some((0,0))` alive).

**Pure — `marley_app`:**
- `editor_nav::tests::*` (REQ-004/005) — 5 tests: LIFO pop-then-empty; dedupe is TOP-ONLY (a re-push
  is a no-op, a different offset stacks, and **A→B→A is not a duplicate** — a full-stack dedupe would
  wrongly collapse it); the cap walked at the EXACT boundary (at 50 nothing is evicted; the 51st drops
  the oldest and the depth stays 50); the picker's wrap in both directions; `chosen()` follows the
  selection; and the degenerate sizes (the `is_empty` guards are load-bearing — without them `len()-1`
  underflows and `% 0` divides by zero).
- `app::tests::def_label_renders_path_and_one_based_line` — **the inspect's binding MSI requirement.**
  Asserts both arms and the LITERAL string, because `loc.line + 1` sits inside `format!`, which
  cargo-mutants does not descend into: no mutant guards the 0→1-based conversion, so only the literal
  catches an off-by-one.

**Headless drives (5) — the REAL consume path, via `drive_definition_for_test` + real keystrokes:**
- `lsp_goto_definition_jumps_and_navstack_headless` (REQ-001/003/005/007) — a single-`Location` answer
  OPENS the target file, lands the caret, parks the centre row, and pushes the origin; **⌃- (a real
  `simulate_keystrokes("ctrl--")`, so the Editor-scoped keymap row and the dispatch arm are proven too)**
  returns to exactly where F12 was pressed and consumes the entry. **REQ-007 is proven by construction:**
  the target's line 2 is `let 🦀 = target;` — a crab is ONE char but TWO UTF-16 units, so the answer's
  UTF-16 column 6 must land at char offset **29**; read as a char column it would be 30. The test pins 29.
- `lsp_goto_definition_multi_opens_picker_and_empty_flashes_headless` (REQ-004/006) — two targets open a
  2-row picker and do NOT jump; real ↓ then Enter jumps to the SECOND candidate and closes it; an empty
  answer flashes "No definition found" and moves the caret nowhere.
- `lsp_goto_definition_unopenable_target_moves_nothing_headless` (**inspect F2, HIGH**).
- `lsp_goto_definition_moved_on_response_is_dropped_headless` (**inspect F4**).
- `lsp_nav_back_survives_a_stale_offset_headless` (**inspect F1, HIGH**).

### The tests have TEETH (verified by reverting each fix, not assumed)
- **F2:** reverting the landing check → the test FAILS, and reproduces the exact bug — the caret moved
  from offset 16 to **offset 27 in `main.rs`** (the target's (line 2, char 6) mapped against the ORIGIN's
  text). That is the finding, verbatim.
- **F1:** reverting the clamped read-back → the test **HANGS** (timed out at 180s) rather than failing —
  the panic escapes `window.update` before `reap_sessions`, and the PTY drop chain blocks. This is the
  module's documented harness edge, now empirically confirmed and written into the test's own comment: a
  hang there IS the F1 regression.

### 🔴 THE LIVE DRIVE FOUND A CRITICAL BUG IN SHIPPED #309 — fixed here

F12 is a real UI change, so it was driven on the live app against a real rust-analyzer. **It did not
work — and the cause was not #312.**

**Symptom:** F12 flashed "No definition found" for every symbol, and ⌘K hover (shipped #311) rendered no
card at the same verified caret. The decisive tell: F12 on a symbol whose definition is **in the same
file** (`Bias`, defined 16 lines above the caret) also came back empty. A same-file jump needs nothing
but the document itself — so the server had no document.

**Wire capture** (an `[[lsp.servers]]` tee wrapper around the real rust-analyzer — the only thing that
showed the truth):
```
BEFORE:  didOpen find.rs    v=0 TEXT_LEN=0   ''
         didOpen anchor.rs  v=0 TEXT_LEN=0   ''
         didOpen buffer.rs  v=0 TEXT_LEN=0   ''      (whole client→server log: 1226 bytes)
AFTER:   Content-Length: 16781
         {"method":"textDocument/didOpen","params":{"textDocument":{"uri":".../find.rs",
          "languageId":"rust","version":0,"text":"//! PURE — the editor's SEARCH primitives…
                                                  (whole log: 104220 bytes)
```

**Root cause — a Ready race between two Ready-gated calls with a collect in between.** The pump built
`open_files`, materializing `buffer.text()` only `if host.needs_text(..)` (the #309 inspect's hot-path
optimization), and only THEN ran `drain(); reconcile(..)`. But a host reaches `Phase::Ready` **inside**
`drain()` — the initialize response is what promotes it — and both `needs_text` and `reconcile` are
Ready-gated. On the exact tick the server went Ready: `needs_text` still saw PRE-Ready → text `""`;
`drain()` → Ready; `reconcile` → Ready → sent that empty string as the document, and recorded it as
synced. No didChange follows (`entry.synced == version`) until the user happens to type, so **the empty
document is permanent**. rust-analyzer held every open file as zero-length and answered null to
everything: hover (#311) and definition (#312) were both dead in the real app.

**Fix (in `app.rs`'s pump): drain FIRST, then collect, then reconcile.** Keeps the hot-path optimization;
removes the race. Proven on the wire, before and after (above).

**Why no test caught it:** every #309/#310/#311/#312 headless drive INJECTS a response via
`push_response_for_test` into a process-less host, so none of them ever exercises the real
didOpen→server→answer round trip. #310's and #311's "driven proof" were headless for exactly this
reason. #312's validate is the first time the live wire was inspected. Recorded as
`BF-lsp-didopen-carries-empty-text-ready-race-001` + `PR-claude-two-gated-calls-must-read-the-phase-once-001`
+ `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`. **The regression test needs a
Ready-capable host in the app's headless lane, which does not exist yet (`fake_ls` reaches Ready only
from `marley_lsp`'s own integration tests — `CARGO_BIN_EXE_` does not resolve cross-crate) → #320.**

⚠️ **SCOPE NOTE FOR CHAD:** this fix is in #309's code, not #312's. It is here because #312's REQ-003
cannot be verified live without it, and because #313–#317 would otherwise be built on a foundation where
every LSP answer is null. It also means **#309/#310/#311 shipped with this live**, and their "driven
proof" claims should be read as headless-only.

### Live drive — what was proven on the real app, and what was not
| | Proof |
|---|---|
| F12 → Editor-scoped keymap row → dispatch → `go_to_definition` | ✅ **PIXEL** — "LSP: not ready" flash on a `.md` file (no host spawns for non-`.rs`). **This is inspect F3's fix rendering live**; pre-fix this was a silent no-op. |
| request → real rust-analyzer → response → `consume_lsp_responses` → Definition arm → stale+liveness guards → `parse_definition_result` → REQ-006's 0-target flash | ✅ **PIXEL** — "No definition found". That flash cannot render unless every one of those steps ran. |
| The didOpen bug + its fix | ✅ **WIRE** — TEXT_LEN=0 ×3 → Content-Length 16781 of real source. |
| REQ-003's jump PIXEL | ❌ **ENV-BLOCKED** — see below. |

**Why the jump pixel is env-blocked (stated plainly, not skipped):** after the fix, driving F12 again
required synthetic input, and `PixelCityBros` — **a MonoGame app of chad's own, running from
`~/Projects/pcb_monogame`** — held frontmost and re-took it after every activation (SDL/MonoGame apps
re-assert focus in their run loop). `osascript … set frontmost` returned exit 0 while Marley stayed
unfocused, so F12 never reached it (0 definition requests on the wire, grey traffic lights in every
capture). Killing another of the user's running apps to win a screenshot is not an acceptable trade, so
the drive stops here. The jump/NavStack/picker are carried by the 5 headless drives — which the harness
README names the PREFERRED lane for behavioural checks ("Headless-first (M16 #264)… this harness remains
the tool for PIXEL proofs"). Re-verify the jump pixel opportunistically when the machine is free; it
needs ~2 minutes and no ticket.

### Harness additions (`scripts/selftest/drive.swift`)
#312's two chords were undriveable: added the `f12` named key (keycode 111) and a `ctrl:<key>` verb
(⌃- is the jump-back chord). Same established pattern as #218's `left`/`right`.

### Gate — GREEN [diff], 15/15
```
══ gate summary (diff) ══
  PASS  gate:1  rustfmt              PASS  gate:11 shellcheck
  PASS  gate:2  clippy (-D warnings) PASS  gate:12 no-suppressions
  PASS  gate:3  tests (nextest+doc)  PASS  gate:13 source-bans (SAST)
  PASS  gate:7  cargo-audit          PASS  gate:14 docs (rustdoc -D warnings + brand-scrub)
  PASS  gate:8  cargo-deny           PASS  gate:4  rust coverage (>= 100% lines)
  PASS  gate:9  cargo-machete        PASS  gate:5  mutation (MSI >= 100%)
  PASS  gate:10 gitleaks (secrets)   PASS  gate:6  miri (unsafe crates)
                                     PASS  gate:15 visual / AX
  15 passed, 0 failed
GATE GREEN [diff]
```
`cargo nextest run --workspace` = **1257 passed, 5 skipped**. Coverage 100% and **MSI 100%** confirm the
two binding inspect requirements landed: `def_label`'s 2 mutants are dead (the literal-string assertion)
and `start_of`'s four `Some((0|1,0|1))` mutants are dead (the off-grid line-5/char-2 fixtures).

Only one red on the first full run — `gate:1 rustfmt`, from edits made after the last format. Fixed at
source (`cargo fmt --all`); re-run green.

### ⚠️ A 45-minute self-inflicted detour, recorded so it is not repeated
The gate's coverage step hung for 42 minutes on an UNRELATED PTY test
(`workspace_two_real_sessions_are_independent`, 0.0% CPU the whole time) — while that same test passed in
**0.11s** standalone and `cargo llvm-cov nextest -p marley` passed 483 tests in 2.1s.

Cause: **my own F1 teeth-check orphan.** Deliberately reverting the F1 fix makes that test HANG (by
design — the documented drop-chain edge), so I bounded it with `subprocess.run(timeout=180)`. That killed
*cargo*, but the test binary is a GRANDCHILD and survived — wedged in the PTY drop chain for 98 minutes,
holding the resource every later PTY test needs. `kill -9` on it → `cargo nextest run --workspace` went
from a 9-minute hang to **1257 tests in 3.7s**.

Recorded as `PR-claude-timeout-killing-cargo-orphans-the-test-binary-001`: after ANY timeout-killed cargo
run, `pgrep -f 'target/debug/deps/'` and kill survivors before trusting the next run; prefer nextest's
`slow-timeout`/`terminate-after` **config** (it kills the test process, not just cargo — and note
`--slow-timeout` is a config key, NOT a CLI flag: passing it as a flag errors out and silently runs
nothing, which cost another cycle). Diagnostic tell: a test hung at 0% CPU inside a big run but green
standalone = a leaked process holding a shared resource, not a bad test.

### Pre-existing / out of scope
None. The one bug found in shipped code (the empty-didOpen Ready race, #309) was FIXED here rather than
deferred — see the scope note above; #320 carries its structural hardening + the missing test lane.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

### Documentation (§21 — both halves)
- **CHANGELOG.md** — an `### Added` entry for #312 (F12 goto + NavStack + picker; the pure seams; the
  encoding-correct caret via the #309 bridge; the deferred centre; the deliberate ⌘-click non-decision),
  and an `### Fixed` entry for the empty-`didOpen` Ready race with its wire evidence.
- **`docs/marley_architecture/crate-map.md`** — the `marley_lsp` row gains the #312 definition payload +
  the shared `text_document_position_params`, and now records the doc-sync ordering as load-bearing
  (drain before collect, or the didOpen ships an empty document).
- **`docs/marley_architecture/editor.md`** — "Current state" said **"no LSP"**, which M20 had made flatly
  false. Replaced with what actually ships (diagnostics #310, hover #311, goto #312, over the #308/#309
  wire), the drain-before-collect hazard, and an honest remainder (#313/#314/#317). Also dropped the
  stale "no find" (the #272 find bar shipped).

### Knowledge captured (forge)
`aar-submit` → **10 novel findings**, 3 failures + 7 prevention rules:
- `BF-lsp-navback-stale-offset-line-col-panics-001` (HIGH) → `PR-claude-clamping-setter-does-not-clamp-the-readback-001`
- `BF-lsp-goto-failed-open-writes-caret-into-wrong-file-001` (HIGH) → `PR-claude-silent-no-op-loader-verify-the-landing-001`
- `BF-lsp-didopen-carries-empty-text-ready-race-001` (**CRITICAL**, shipped #309) →
  `PR-claude-two-gated-calls-must-read-the-phase-once-001` + `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`
- `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` (inspect F4)
- `PR-claude-new-overlay-register-at-every-choke-point-001` (inspect F5)
- `PR-claude-timeout-killing-cargo-orphans-the-test-binary-001` (the 45-minute self-inflicted gate hang)

### What worked
- **Five critics on distinct lenses, told that false positives cost more than misses.** Three of them
  INDEPENDENTLY found the same two crashes — that convergence, not any single critic's confidence, is
  what promoted them to HIGH. The lens split also paid: the reuse critic self-rejected its own
  "reuse `open_file_at`" suggestion after reading the code (it counts CHARS, not encoded columns), which
  is exactly the discipline that keeps an inspect worth its cost.
- **Reverting each fix to prove the test bites.** The F2 revert reproduced the bug verbatim (caret to
  offset 27 in the ORIGIN file); the F1 revert hung, confirming the comment I had written about that
  harness edge. A test that passes with and without the fix is worthless, and only the revert shows which.
- **Driving the live app instead of trusting green tests.** Everything was green and the feature was dead.
  Only the real wire showed it.

### What bit
- **The live drive was the whole point and nearly wasn't done.** Every M20 ticket before this one
  declared "driven proof" from headless tests that inject their own answers — which cannot see a broken
  producer. Four features shipped that way; all four were non-functional. The lesson is not "test more",
  it is that **a test that fabricates the other side's reply proves only your half.**
- **A same-file definition returning empty was the tell**, and it took too long to reach for it. It needs
  nothing but the document, so its failure indicts the sync, not the parser. Reading code did not find
  this; teeing the wire found it in one shot.
- **My own orphaned test binary cost 45 minutes** and looked exactly like a code regression (the gate
  hanging on an unrelated PTY test). `subprocess.run(timeout=)` kills cargo, not its grandchild.
- **A `git add -A` swept up two docs another agent wrote mid-session** (`docs/README.md`,
  `detached-sessions.md`, timestamped 11:44). Unstaged, left intact. Stage by path when a repo is shared.

### Scope taken deliberately (flagged for chad)
The empty-`didOpen` fix is **#309's code, shipped inside #312**. Justification: REQ-003 cannot be verified
live without it, and #313–#317 would otherwise be built on a foundation where every LSP answer is null.
The consequence to be honest about: **#309/#310/#311 shipped with this bug live**, and their driven-proof
claims should be read as headless-only.

### Follow-ups filed
- **#318** — extract the shared overlay-card recipe (a 5th copy-paste; needs palette/finder/history
  converted off hand-rolled positioning → its own blast radius).
- **#319** — editor file identity vs LSP canonical paths: under a symlinked project root, F12 and the file
  tree spell the same file two ways → two tabs, two buffers, one file (defeats the #249 guarantee).
- **#320** — make the doc-sync text materialization race-proof BY CONSTRUCTION (pass `&Buffer`/`text_of`
  into `reconcile`, delete `needs_text`) + build the missing app-level Ready (`fake_ls`) test lane that
  would have caught the didOpen bug. **Assert the payload, not the method** — the bug shipped with a
  perfectly correct `"method":"textDocument/didOpen"`.
- Re-verify the REQ-003 jump PIXEL opportunistically when the machine is free (~2 min, no ticket) — the
  drive was blocked by another of chad's apps holding frontmost, not by anything in the code.

status: Phase 5 — Complete PASS
