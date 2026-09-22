# 311 — LSP hover — notes

## Phase 1 — Plan

### Intent
LSP `textDocument/hover` at the caret (⌘K, editor-gated) or a ~400ms mouse dwell → a rounded overlay
card showing the type signature + docs (markdown-lite, code highlighted). Also builds the GENERAL
request→response→consumer path on `LspHost` (today only `initialize` is handled) — the reusable wire the
rest of M20's requests (#312 goto, #313 completions, #317 references) depend on. Surfaces #310's
deferred `diagnostic_at_row` + `Diag.message` (a hovered squiggle shows its message).

### Discovery (Explore) — landing zones
- **Overlay card #221** = an INLINED div recipe (no reusable builder), e.g. `app.rs:9006-9028`:
  `.absolute().occlude().bg(surface).rounded(corner_radius).border_1().border_color(border)`. The
  window-clamp helper to REUSE = pure `context_menu::menu_origin(x,y,w,h,win_w,win_h)` (`context_menu.rs:165`,
  tested). The completion popup (`app.rs:9971-10004`) is the closest anchored-overlay precedent.
- **#217 is INSTANT group-hover, not a dwell** — there is NO dwell/stationary detector anywhere. Must
  ADD: a RootView `last_hover_cell` + `hover_ticks`, a general editor `on_mouse_move` that records the
  cell + resets the counter, and a per-tick threshold check. Reuse the pump loop (`app.rs:727`,
  `PUMP_INTERVAL_MS=16`) + the `notify_ticks` tick→ms idiom (`app.rs:181,790`). ~400ms ≈ 25 ticks.
- **LSP request/response is the CRUX (mostly missing).** `on_message`'s Response arm handles ONLY
  `initialize` (`lsp_host.rs:351-371`); a non-init id is `resolve()`d and the result DROPPED. `Pending`
  (`rpc.rs:124`) stores `{method, deadline}` — no purpose/consumer. The only send path is the private
  `Action::SendInitialize`. Must ADD: a public `request()` sender (idgen + register + `send_body`), a
  purpose tag on the pending (or a side map), a non-init Response arm that stashes the result for the
  app to drain, and a hover params builder (`build_request` + hand JSON, the #308 idiom). Reuse:
  `IdGen`, `PendingTable`, `build_request` (`rpc.rs:214`), `route`, `drain`, `offset_to_position`
  (`position.rs:79`), `file_uri`, `host.encoding()`.
- **Highlight** — REUSE `code_syntax::highlight_ranges(line, lang)` (`code_syntax.rs:257`, multi-lang
  hand lexer; Rust also tree-sitter) + `StyledText::with_highlights`. NO new highlighter. **No markdown
  prose renderer exists** — `markdown_runs` prose handling is net-new (fences reuse the highlighter).
- **Caret pixel anchor** — `EditorFrameGeom` (`app.rs:461`, set each frame `3360-3378`); the caret→px
  formula is inlined in `bounds_for_range` (`app.rs:6150-6179`, the IME anchor). Extract a
  `caret_screen_rect()`; the pointer→caret inverse is `character_index_for_point` (`app.rs:6182`).
- **Caret + version** — `active_caret()` (`editor_surface.rs:211`), `Buffer::line_col` (`buffer.rs:149`),
  `Buffer::version()` (`buffer.rs:88`, already the syntax-cache/reconcile key). All present.
- **⌘K COLLISION** — bound to `clear-screen` (`keymap.rs:189`, TERM ctx). D1: context-gate to the editor.

### Classification
Work pipeline, feature, M20. LARGE but cohesive (one "hover" feature). The general request/response path
is foundational infra reused by #312/#313/#317 — built generically here (D2). Kept as ONE slice; the
markdown subset + no in-card-actions bound the scope (see spec Out).

### Reference (§20)
Zed (editor) — the hover-card-on-symbol behavior, matched by observing the behavior + the LSP 3.17
`textDocument/hover` wire contract, reimplemented on Marley's #221 recipe + `code_syntax`. No GPL source
read. Filled in the spec.

### Forge
- Ticket #311 claimed (`ac07a89b`); AAR opened `fedf1a48-84e2-4122-b959-c0efe8a7f56e` (inspect/complete
  capture into it).

### Risks / decisions
- **R1 — the response-to-consumer correlation is the highest-risk seam.** A wrong-id or reused-id could
  route a response to the wrong consumer; the stale-guard (D7, key by uri+position+version) is the
  safety net. Design must make the pending purpose + the drop-on-mismatch explicit + tested.
- **R2 — the dwell detector must not fire during a drag** (select/divider/files-edge drags already own
  mouse-move). Gate the dwell on "not dragging + pointer inside editor text".
- **R3 — `diagnostic_at_row` was removed in #310 F3**; re-adding it must restore its unit + keep MSI 100.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture / approach
Two clean-room pure layers + a thin shim, extending the #308 wire.

**1. The GENERAL request→response→consumer path (the reusable infra, D2).** Today `rpc.rs::Pending`
is `{method, deadline}` (no purpose) and `on_message` only handles the `initialize` response. Add, in
the APP shim (`lsp_host.rs`) — NOT in the pure rpc layer, since a purpose is an app concept:
- `enum RequestPurpose { Hover(HoverKey) }` (extensible: #312 `Definition`, #313 `Completion`, …).
- fields `purposes: HashMap<i64, RequestPurpose>` + `responses: Vec<(RequestPurpose, Result<Value, RpcError>)>`.
- `pub(crate) fn request(&mut self, method, params, purpose) -> Option<i64>` — only when `Ready`;
  `idgen.next_id()` → `pending.register(id, method, tick, HOVER_TIMEOUT_TICKS)` → `purposes.insert(id,
  purpose)` → `send_body(build_request(id, method, params))`. Returns the id (`None` if not Ready).
- `on_message` Response arm gains a non-init branch: `if let Some(p) = self.purposes.remove(&id) {
  self.responses.push((p, result)); }` (the pending was already `resolve()`d for the timeout side).
- the `expire()` timeout path ALSO `purposes.remove()`s each timed-out id (drop its purpose — no
  late/stale delivery).
- `pub(crate) fn take_responses(&mut self) -> Vec<(RequestPurpose, ...)>` — the app drains it each pump
  tick right after `host.drain()` (same clock, same dirty-repaint idiom).
The pure `rpc.rs` is UNCHANGED (Pending stays generic) — the purpose lives entirely in the shim.

**2. Pure seams — `marley_lsp` (gpui-free, LSP-generic, NO editor dep):** a NEW `hover.rs`:
- `hover_request_params(uri: &str, line: u32, character: u32) -> Value` — the `textDocument/hover`
  params (hand-built JSON, the #308 idiom; testable).
- `parse_hover_result(value: &Value) -> Option<String>` — the LSP `Hover.contents` → one markdown
  string. Handles all 3 shapes hand-parsed (avoids widening the lsp-types surface): a bare string; a
  `{value}` object (MarkupContent / MarkedString); a `{language, value}` object → wrapped as a
  ```lang fence so `markdown_runs` highlights it; an ARRAY → each element parsed + joined by `\n\n`.
  Empty/whitespace/null → `None` (drives REQ-006).
- `markdown_runs(md: &str) -> Vec<HoverRun>` where `enum HoverRun { Code{lang: Option<String>, text},
  Heading{text}, Para{text} }` — block-level split: a ``` line opens/closes a fence (its info-string =
  the lang); a `#{1,6} ` line = a Heading; blank-line-separated prose = a Para. An UNCLOSED fence at
  EOF → the buffered lines emit as Para(s) (plain — never a dangling Code, satisfies "malformed →
  plain, no panic"). **Design reduction D-des-1: inline code within a Para is preserved as literal
  text (backticks kept, rendered plain in v1)** — a true inline-span split is deferred; the spec's
  "inline-code run" is delivered as inline-preserved-in-Para. The highlighting charm (fenced blocks)
  is fully delivered.

**3. Pure seams — `marley_app` (gpui-free, app-specific): a NEW `editor_hover.rs`:**
- `struct HoverKey { uri: String, line: u32, character: u32, version: BufferVersion }` (derives
  PartialEq/Eq/Clone) — the stale-guard key. The app holds `hover_request: Option<HoverKey>` = the
  LATEST request; a response matches iff `response_key == hover_request` (a moved caret / an edit
  [version bump] / a newer request all make it differ → DROP). REQ-005 is this `==`.
- `enum DismissTrigger { Edit, Esc, CaretMove, Scroll, MouseInsideCard, MouseOutside }` +
  `hover_dismiss(t) -> bool` (Edit/Esc/CaretMove/Scroll/MouseOutside → true; MouseInsideCard → false).
  REQ-004.
- `const DWELL_TICKS: u32 = 25;` (≈400ms / 16ms) + `should_fire_dwell(ticks) -> bool` (`ticks ==
  DWELL_TICKS` — fire exactly once at the threshold). REQ-002 pure half.

**4. Pure seam re-add — `marley_lsp/diagnostics.rs`:** restore `diagnostic_at_row(diags, row) ->
Option<&Diag>` (removed in #310 F3) — `diags.iter().filter(|d| (d.span.start.line as usize) <= row &&
row <= (d.span.end.line as usize)).min_by_key(|d| d.span.severity)` (now via `d.span.*`). REQ-008.

**5. The shim wiring — `app.rs` + `lsp_host.rs`:**
- **⌘K (D1):** add `(cmd-k, "lsp-hover", Some(KeyContext::Editor))` to `keymap.rs` — it SHADOWS the
  global/terminal `clear-screen` when an editor tab is focused (exactly like ⌘D-on-editor at
  keymap.rs:248; `chords_unique_scoped` allows it — different context). A new `"lsp-hover"` arm in
  `dispatch_action` (app.rs:4720): build the caret's `HoverKey` (uri = `file_uri(active_file.path)`,
  (line,char) = `offset_to_position(text, caret, host.encoding())`, version = `buffer.version()`),
  `host.request("textDocument/hover", &hover_request_params(...), Hover(key.clone()))`, store
  `hover_request = Some(key)` + the anchor cell (row, display col).
- **Dwell (D6):** new RootView fields `last_hover_cell: Option<(usize,usize)>` + `hover_ticks: u32`. A
  general editor-text `on_mouse_move` (gated: not dragging — R2) maps the pointer→cell
  (`character_index_for_point`); if the cell changed → reset `hover_ticks=0`, set `last_hover_cell`,
  and dismiss any open card; if unchanged → leave it. Each pump tick: `hover_ticks += 1`; if
  `should_fire_dwell(hover_ticks)` and a host is Ready → fire a hover request at that cell.
- **Response consume (pump):** after `host.drain()`, for each `(Hover(key), result)` in
  `host.take_responses()`: if `Some(key) != self.hover_request` → DROP (stale, REQ-005); else
  `parse_hover_result(result)` → `None` → clear (REQ-006); else `markdown_runs` → set `hover_card =
  Some(HoverCard{ runs, anchor, diagnostic: diagnostic_at_row(diags, key.line).map(|d| d.message) })`
  + `dirty = true`.
- **Card render:** the #221 recipe overlay div at the anchor px (`caret_screen_rect` extracted from
  `bounds_for_range`'s inlined math — REUSE), origin clamped by `context_menu::menu_origin(x, y, w, h,
  win_w, win_h)` (REQUSE, tested), `.occlude()`, max-height + `overflow_y_scroll`. Each `HoverRun`:
  `Code{lang}` → `StyledText::with_highlights(highlight_ranges(line, language_of_lang(lang)))` per line
  (REUSE); `Heading` → bold text; `Para` → muted text. The optional diagnostic message renders first
  (danger-tinted).
- **Dismissal:** the edit / Esc / caret-move / scroll sites call `hover_dismiss(trigger)`; on true →
  clear `hover_card` + `hover_request` + `last_hover_cell`. The card div `.occlude()`s so a mouse INTO
  it is `MouseInsideCard` (no dismiss).

### §20 confirmation
Zed (editor). This design MATCHES the observed hover BEHAVIOR (a framed card at the symbol with the
type + docs, code highlighted, dismissed on edit/move) via the published LSP 3.17 `textDocument/hover`
wire contract + Marley's own #221 recipe / `code_syntax` / `menu_origin` — NO Zed GPL source read or
translated. `markdown_runs` is Marley's own deliberate reduction. Confirmed N/A-of-copyleft.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/hover.rs` | NEW pure: `HoverRun`, `markdown_runs`, `parse_hover_result`, `hover_request_params`. |
| `crates/marley_lsp/src/diagnostics.rs` | RE-ADD pure `diagnostic_at_row` (via `d.span.*`) + its export. |
| `crates/marley_lsp/src/lib.rs` | export the `hover` module items + `diagnostic_at_row`. |
| `crates/marley_app/src/editor_hover.rs` | NEW pure: `HoverKey`, `DismissTrigger`+`hover_dismiss`, `DWELL_TICKS`+`should_fire_dwell`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose`, `purposes`/`responses` fields, `request()`, `take_responses()`, the non-init Response arm, expire→purposes cleanup. |
| `crates/marley_app/src/keymap.rs` | `(cmd-k, "lsp-hover", KeyContext::Editor)`. |
| `crates/marley_app/src/app.rs` | the `lsp-hover` dispatch arm, the dwell fields+mouse-move+pump-fire, the response drain+stale-check, the card render + `caret_screen_rect` extraction, the dismissal wiring, `mod editor_hover`. |
| `crates/marley_app/src/lib.rs` (or app mod decl) | `mod editor_hover;`. |

### Regression Test Plan (≥1 per REQ)
| Test (loc) | REQ | Asserts |
|---|---|---|
| `hover::tests::hover_params_shape` (marley_lsp) | 001 | `hover_request_params` → the `{textDocument:{uri}, position:{line,character}}` JSON. |
| `editor_hover::tests::dwell_fires_once_at_threshold` (marley_app) | 002 | `should_fire_dwell(24)=false`, `(25)=true`, `(26)=false`; `DWELL_TICKS` hard-pinned. |
| `hover::tests::markdown_runs_*` (fence+lang / fence-no-lang / heading / para / unclosed→plain / inline-preserved) | 003 | block split correct; unclosed fence → Para (no panic); inline backticks survive in Para. |
| `hover::tests::parse_hover_result_shapes` | 003/006 | string / `{value}` / `{language,value}`→fence / array-join / empty→None. |
| `editor_hover::tests::hover_dismiss_table` | 004 | Edit/Esc/CaretMove/Scroll/MouseOutside→true, MouseInsideCard→false. |
| `editor_hover::tests::hover_key_stale_compare` | 005 | equal keys match; differ in uri/line/char/version → no match. |
| `hover::tests::parse_hover_result_empty_is_none` | 006 | empty/whitespace/null contents → None. |
| `context_menu::tests::menu_origin_clamps` (EXISTING) | 007 | reused — the clamp is already proven. |
| `diagnostics::tests::diagnostic_at_row_most_severe` (marley_lsp) | 008 | row on a span → the covering diag; two overlapping → the most-severe (Error); row outside → None. |
| `headless_drive::lsp_hover_request_response_renders_headless` (marley_app) | 001/005/006/008 | boot editor, ⌘K/`request`, feed a synthetic hover Response through the REAL request/response path (a `feed_lsp_response_for_test` hook), assert `hover_card` populated with the runs + the diagnostic; feed a STALE-key response → dropped; feed empty → no card. |
| `keymap::tests` (extend `chords_unique_scoped` coverage) | 001 | the editor ⌘K binding coexists with clear-screen (no same-context dup). |

Uncoverable-by-unit: the live pixel card render (gpui 0.2.2 test platform has no pixels) + a live
rust-analyzer hover round-trip (orchestration-blocked, per #310) → the headless request/response drive
+ the caret-overlay-primitive mechanism carry it; a live pixel capture is a documented no-code follow-up.

### Risks / decisions
- **R1 (highest) — response→consumer correlation.** A reused/wrong id could misroute a response.
  Mitigation: monotonic `IdGen` (never reuses — #308 invariant) + the purpose map removed on resolve +
  the stale-key `==` drop as the safety net. The inspect critics target this seam.
- **R2 — dwell must not fire during a drag** (select/divider/files-edge own mouse-move). Gate the dwell
  mouse-move on `!dragging_* && pointer in editor text`.
- **R3 — `diagnostic_at_row` re-add** must restore its unit + keep MSI 100 (now via `d.span.*`).
- **D-des-1 — inline code preserved-in-Para (not a separate run) in v1** (recorded above).
- **D-des-2 — `parse_hover_result` hand-parsed** (not lsp-types) to keep the dependency surface at the
  one #310 publish-parse seam + be forgiving of the 3 contents shapes.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`crates/marley_lsp/src/hover.rs` (NEW, pure)** — `HoverRun{Code{lang,text}|Heading{text}|Para{text}}`,
  `markdown_runs` (fence/heading/para/`---`-separator block split; unclosed fence → plain Para),
  `parse_hover_result` (the 3 `Hover.contents` shapes hand-parsed → one markdown string; empty→None),
  `hover_request_params`. gpui-free + editor-free.
- **`crates/marley_lsp/src/diagnostics.rs`** — re-added `diagnostic_at_row` (via `d.span.*`) + export.
- **`crates/marley_lsp/src/lib.rs`** — export the `hover` module + `diagnostic_at_row`.
- **`crates/marley_app/src/editor_hover.rs` (NEW, pure)** — `HoverKey{uri,line,character,version}` (the
  stale-guard `==` key), `DismissTrigger` + `hover_dismiss` (all-but-`MouseInsideCard`→true),
  `DWELL_TICKS=25` + `should_fire_dwell`.
- **`crates/marley_app/src/lsp_host.rs`** — the GENERAL request/response path: `RequestPurpose::Hover`,
  `purposes`/`responses` fields, `request()` (idgen+register+purpose+send, Ready-gated),
  `take_responses()`, `uri_for()`, the non-init Response arm (route to `responses`), expire →
  `purposes.remove`, on_connection_lost clears both. `REQUEST_TIMEOUT_TICKS = 10 s`.
- **`crates/marley_app/src/keymap.rs`** — `(cmd-k, "lsp-hover", KeyContext::Editor)` (shadows terminal
  clear-screen).
- **`crates/marley_app/src/app.rs`** — the shim: `HoverCard` state + 4 RootView fields; `hover_at_caret`
  (⌘K), `request_hover` (build key + send), `consume_hover_responses` (drain + stale-drop + parse +
  markdown + diagnostic), `diagnostic_message_at_row`, `tick_hover_dwell`, `on_editor_mouse_move`,
  `dismiss_hover`, `poll_hover_dismiss` (edit+scroll), `hover_card_overlay` + `hover_run_element` (the
  #221 recipe, `menu_origin`-clamped, fenced code highlighted). Wired: the `lsp-hover` dispatch arm +
  a dismiss-on-any-other-action guard, the pump (consume+tick+poll after `host.drain()`), the editor
  body `on_mouse_move`, the Esc hook in `on_key_down`, the card `on_mouse_move` no-op, the root overlay.
- **`crates/marley_app/src/lib.rs`** — `mod editor_hover;`.
- **`crates/marley_app/Cargo.toml`** — added `serde_json = "1"` (the `Value` in the request/response path).

### Deviations from design (with reason)
- **All SIX `DismissTrigger` variants are constructed in NON-test code** (else clippy's per-target
  dead-code lint fails `-D warnings`): Edit + Scroll via `poll_hover_dismiss` (version / `geom.first`
  compare — the IME + the list-owned scroll don't reach `dispatch_action`), CaretMove via the
  dispatch guard, MouseOutside via the editor-body `on_mouse_move`, Esc via `on_key_down`,
  MouseInsideCard via the card's own `on_mouse_move` no-op (documents the occlude contract).
- **D-des-4 — scroll dismissal is the `geom.first` poll + an anchor-out-of-viewport render hide**, not
  a wheel hook (gpui's `uniform_list` owns editor scrolling; there is no wheel handler to hook).
- **The fence-language map is inline in the render shim** (a 5-arm match: rust/toml/json/sh → Language)
  rather than a new pure `code_syntax` fn — keeps the manifest tight; the render is `mutants::skip`.
- **`serde_json` added as a direct dep of `marley_app`** (was only transitive) — the general
  request/response path names `Value`/`RpcError` in the app shim.
- **D-des-1 (from design) held** — inline code is preserved as literal text within a `Para` (v1).

### Verified
- `cargo check --workspace` clean; `cargo clippy -p marley_lsp -p marley --all-targets` clean (all
  dismiss variants constructed — no dead-code); `cargo fmt --all --check` clean.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Four critics (correctness, provenance/security, state-integrity, simplification). Convergence on the
dismissal-completeness gap (arrows/tab-switch) as the top issue.

### Findings + verdicts
- **F1 [HIGH] — a keyboard caret move (arrows) neither dismissed the card NOR invalidated the in-flight
  response.** REAL. The dispatch guard only covers DISPATCHED actions, but arrows / ⌘-arrows are
  handled INLINE and `return` without dispatching, so the card stayed pinned at the stale cell (REQ-004
  fail) and `consume`'s `hover_request` stale-guard (which a bare caret move never updates) showed the
  answer at a moved caret (REQ-005 fail). **FIX:** `HoverCard` now carries `path` + `caret_at_show`;
  `poll_hover_dismiss` compares the LIVE (file, caret, version, first-row) each tick and dismisses on
  ANY change (the robust hook — the IME, inline arrows, the list-owned scroll, and mouse tab-clicks all
  bypass `dispatch_action`); `consume` drops a response whose `key.uri` no longer names the focused
  file. Prevention rule recorded.
- **F2 [MED] — a diagnostic-only location showed nothing (REQ-008).** REAL. The diagnostic was only
  read inside the `Some(md)` arm, so a location with a diagnostic but an empty LSP hover (common:
  `expected ';'`) showed no card. **FIX:** `consume` computes the diagnostic unconditionally and shows
  a card when `!runs.is_empty() || diagnostic.is_some()`.
- **F3 [MED] — a mouse rail-click tab switch left a stale card over the wrong file.** REAL (the
  version+first-row poll couldn't distinguish two unedited files, both `version 0`/`first 0`). **FIX:**
  the F1 identity poll (path compare) dismisses it; `hover_card_overlay` ALSO refuses to paint when
  `card.path` ≠ the focused file (no wrong-file flash).
- **F4 [LOW] — the dwell could fire mid-drag-select.** REAL (`tick_hover_dwell` had no drag guard).
  **FIX:** `if self.dragging_selection { return false; }`.
- **F5 [LOW] — a non-empty result parsing to zero runs (e.g. `"---"`) rendered an empty bordered card.**
  REAL. **FIX:** folded into F2's `runs.is_empty() && diagnostic.is_none()` → no card.
- **F6 [LOW, security] — the card render was not size-capped** (a hostile ≤64 MiB hover → millions of
  gpui children → layout stall). REAL, defense-in-depth. **FIX:** `MAX_RUNS = 40` + `MAX_CODE_LINES =
  80` `.take()` caps in the render.
- **F9 [LOW] — the token→color map was duplicated a THIRD time.** REAL. **FIX:** extracted
  `fn token_color(kind, colors)` and routed all three sites (the hover fence + the two code-view
  renders) through it. #316 replaces the magic literals with a theme palette.
- **F10 [LOW] — `consume` re-cloned the active root every idle tick.** REAL. **FIX:** the pump passes
  its existing `&active_root`.
- **F12 [LOW] — `hover_anchor` stored `(row, col)` but `row == key.line`.** REAL. **FIX:** `hover_anchor`
  is now `Option<usize>` (column only); the row comes from `key.line`.
- **F7 [LOW] — a `MarkedString{language,value}` whose `value` contains ``` corrupts the re-parsed
  fence.** REJECTED for now (DEFERRED). A benign rendering glitch (a stray paragraph), no panic/inject;
  the clean fix (carry the language structurally instead of round-tripping through a fence) is
  disproportionate for a rare edge. Noted as a follow-up.
- **F8 [LOW] — the `MouseInsideCard` `on_mouse_move` handler is inert** (`.occlude()` does the real
  work). ACCEPTED. It constructs the variant that gives `hover_dismiss` its only real branch (and the
  tested policy #312+ reuse) and documents the occlude contract; removing the variant would make
  `hover_dismiss` a constant `true`.
- **F11 [LOW] — the scroll poll + the anchor-out-of-viewport render clip overlap.** ACCEPTED. The poll
  dismisses on any scroll; the clip is a ≤1-frame safety before the next tick + guards an off-screen
  anchor. Both cheap.
- **F14 [LOW] — a non-active workspace's host `responses` queue drains only on reactivation.** ACCEPTED.
  Bounded by its pre-switch in-flight count (~1-2, hover only fires for the active root), cleared on
  reactivation/`on_connection_lost`/drop. A multi-workspace cleanup is a follow-up.

### Provenance / security (CLEARED)
- Clean-room confirmed — `hover.rs` is a hand-rolled 3-variant block scanner, structurally unlike Zed's
  pulldown-cmark `RichText`/`MarkdownElement` + `hover_popover.rs`; the purpose-tagged pending is a
  generic LSP-client pattern, not a Zed idiom. No copyleft markdown/highlight dep (grep: no
  pulldown-cmark/comrak/syntect); the only Cargo delta is `serde_json` (permissive, already in-lock).
- No panic from untrusted server JSON (`.get()?`/`.and_then(as_str).unwrap_or("")`, guarded `lines[i]`,
  ASCII-only `trimmed[hashes..]`); recursion bounded by serde_json's 128-depth parse cap; body ≤64 MiB
  (frame decoder); no server string reaches spawn/FS/format-string; no secrets.

### Mutation pins for Phase 4 (the pure seams — all three critics)
- `editor_hover.rs::hover_dismiss`: a 6-arm table (`MouseInsideCard`→false, the other five→true — kills
  `matches!`→true/false + wrong-variant). `should_fire_dwell`: 24→false, 25→true, 26→false (pins
  `DWELL_TICKS` + the `==`).
- `hover.rs::markdown_runs`: closed fence (with + without lang), unclosed→Para, heading flush,
  blank-line + `---` separator splits, adjacent fences. `heading_text`: 1- and 6-hash accept, 0/7
  reject, `#no-space` reject. `is_separator`: `len>=3` boundary (`--`→false, `---`→true), all-`-`/`*`/`_`
  accept, mixed reject. `parse_hover_result`/`markup_to_string`: bare string, `{value}`,
  `{language,value}`→fence, array-join, empty/whitespace/absent-`contents`→None (REQ-006).
- `diagnostics.rs::diagnostic_at_row`: row on a span → covering diag; two overlapping (Error+Warning) →
  Error (kills `min_by_key`→`max`); `row==start.line`/`row==end.line` boundaries (kills each `<=`→`<`);
  row outside → None.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests added
- **`crates/marley_lsp/src/hover.rs` (7 unit tests)** — REQ-001/003/006 + inspect pins:
  `hover_request_params_shape`; `markdown_runs_fenced_code` (with/without lang + adjacent fences);
  `markdown_runs_headings_paras_separators` (heading, `#`..`######`, blank-line + `---` splits,
  inline-code preserved in a Para); `markdown_runs_heading_and_separator_boundaries` (7-hash / no-space
  → Para, `--`→Para, `***`→rule, mixed→Para); `markdown_runs_unclosed_fence_is_plain` (+ empty /
  whitespace → no runs); `parse_hover_result_shapes` (string / `{value}` / `{language,value}`→fence /
  array-join); `parse_hover_result_empty_is_none` (empty / whitespace / absent / null / `[]` → None).
- **`crates/marley_lsp/src/diagnostics.rs` (1 test)** — `diagnostic_at_row_most_severe_and_boundaries`
  (REQ-008: overlapping Error+Warning → Error; the inclusive start/end-line boundaries; row outside → None).
- **`crates/marley_app/src/editor_hover.rs` (3 tests)** — `hover_dismiss_table` (all-but-`MouseInsideCard`
  → true; REQ-004), `should_fire_dwell_only_at_threshold` (24/25/26 + `DWELL_TICKS` pin; REQ-002),
  `hover_key_equality_per_field` (a difference in ANY of uri/line/character/version breaks equality; REQ-005).
- **`crates/marley_app/src/keymap.rs`** — extended the `all_chords_lists_every_binding` roster guard for
  the new ⌘K Editor binding (51→52 chords, 10→11 scoped).
- **`crates/marley_app/src/headless_drive.rs` (1 integration test)** —
  `lsp_hover_response_builds_and_drops_headless`: drives the REAL `consume_hover_responses` via
  `#[cfg(test)]` hooks (`drive_hover_for_test` sets the request + pushes a response to a process-less
  host; `push_response_for_test` / `hover_card_for_test` / `hover_request_is_some_for_test`), asserting a
  SUPERSEDED response is dropped (REQ-005, the live request stays pending), a matching response builds a
  1-run card (REQ-001/003), and a matching-but-empty result clears it (REQ-006).

### Test results
- `cargo nextest run` (per-crate + the headless test) — all **12 new tests pass**; the full-workspace
  run is green after two fixes below. `cargo test --doc` — clean.

### Two failures found + fixed DURING validate
- **The keymap roster guard failed** (`all_chords_lists_every_binding`) — adding the ⌘K binding is
  exactly what that guard exists to force a conscious update of. FIXED: 52 chords / 11 scoped + the ⌘K
  assertions.
- **The headless hover test HUNG (not failed) then, once made hang-safe, FAILED on a test-logic bug.**
  The harness's sharpest edge (a module NOTE): a headless test that PANICS before `reap_sessions` HANGS
  on the PTY drop chain instead of failing. FIX 1: read ALL results first, `reap_sessions`, THEN assert
  — converting the hang into a readable failure. That failure was a TEST bug, not a code bug: the stale
  case ran AFTER a case that had built a card, so the (correctly) dropped stale response left that prior
  card in place — `after_stale` saw the leftover, not `None`. FIX 2: reordered to run the stale case
  FIRST (from a clean state) — the drop is now proven against `None`. (Confirms the stale-guard drops
  without clearing an unrelated card — the dismiss path owns clearing.)

### Driven / visual verification
- The headless integration test is the driven behavioral proof (the README-preferred lane) — it drives
  the REAL app consume path (parse → markdown → stale-guard → file-match → card). The PIXEL card render
  is the #221 overlay recipe + the tested `col_of_offset`/`token_color`/`menu_origin` seams. A LIVE
  rust-analyzer PIXEL capture is orchestration-blocked here (per #310: `open` forces cwd=`/`); documented
  as env-blocked, carried by the headless drive + the mechanism.

### Gate
- First `--diff` run went **RED on two gates** (fixed at source, §0, no suppressions):
  - **gate:5 mutation 98.1%** — the F9-extracted `token_color` (a top-level fn now, so cargo-mutants
    mutates it standalone) survived `body → Default::default()`; it's called only from the
    coverage-excluded render, so nothing killed it. FIX: a `token_color_maps_each_kind` unit test
    (exact-value assertions per `TokenKind`, using `ThemeColors::default_for(Dark)`).
  - **gate:4 coverage** — `hover.rs` had 1 uncovered line: `markup_to_string`'s `_ => String::new()`
    arm (a non-string/object/array `contents`). FIX: added `{"contents": 42}` + `{"contents": true}`
    cases → the `_` arm covered.
- Re-run: **GATE GREEN [diff]**, 15/15. Coverage 100% on the changed files; mutation **54 caught / 0
  missed → MSI 100.0%**.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

### Documentation (§21)
- **CHANGELOG.md** — the #311 entry (above #310): hover ⌘K/dwell → overlay card, the general
  request/response path, the pure `markdown_runs`/`parse_hover_result` seams, the live-identity dismiss.
- **crate-map.md** — extended the `marley_lsp` row: +#311 hover payload + the general
  request→response→consumer path; LOC ~3000→~3300.

### Knowledge captured (forge)
- **Failures:** `BF-lsp-hover-caret-move-not-dismissed-001` (inspect F1 — the dismiss/stale-guard
  bypassed the inline caret/scroll/tab-switch paths), `BF-lsp-hover-extracted-helper-new-mutation-
  surface-001` (the F9 `token_color` extraction created a standalone mutant + a coverage miss — both
  gate-caught, fixed at source).
- **Prevention rules:** `PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001` (dismiss a
  caret-anchored overlay by polling the LIVE editor identity, not via the action dispatcher),
  `PR-claude-headless-test-panic-before-reap-hangs-not-fails-001` (a headless test that panics before
  `reap_sessions` HANGS — read-all → reap → assert).
- **AAR** `fedf1a48` submitted (outcome completed, effectiveness 4, 4 novel findings).

### Lessons (local)
- The GENERAL request/response path (purpose-tagged pending + drained `responses` + non-init arm) is
  the reusable spine for #312/#313/#317 — only the `HoverKey` payload was hover-specific. Building it
  generically here (D2) was the right call.
- Extracting a DRY helper out of a mutation-excluded render closure creates a NEW standalone mutation
  target — pair the extraction with a direct unit test (the `token_color` gate-red).
- A headless LSP/editor test that hangs is almost always a pre-`reap_sessions` panic masked by the PTY
  drop chain — make it hang-safe FIRST (read → reap → assert) to read the real failure; then a stale-
  drop test must start from a clean state (a dropped stale response doesn't clear a card built earlier).

### Deferred to follow-ups (not in #311 scope)
- Full markdown (tables, images, nested lists) + a true inline-code span split (v1 preserves inline as
  plain-in-Para); in-card actions (links, goto buttons); signature help; the `MarkedString` value-with-
  backticks fence-corruption edge (inspect F7); a live rust-analyzer PIXEL capture when driven interactively.

status: Phase 5 — Complete PASS
