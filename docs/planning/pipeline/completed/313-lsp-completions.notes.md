# 313 — LSP completions: the as-you-type popup + a TextEdit-faithful accept — notes

- **Forge ticket:** #313 ad15e168-b72d-4a46-a64a-90f9b858408d
- **AAR:** 989649bc-21e1-47a0-8624-12a360db0ef1
- **Local ticket doc:** docs/planning/tickets/open/TICKET-313-lsp-completions.md
- **Pipeline spec:** 313-lsp-completions.spec.md

## Phase 1 — Plan

### Request
The as-you-type completion popup in the EDITOR: trigger → `textDocument/completion` → fuzzy-rank → popup
→ accept that REPLACES the typed prefix. The batch's biggest daily-value feature and its most stateful.

### Classification / tier
Work pipeline, feature, M20. ONE shippable slice: trigger → request → parse → rank → popup → accept →
undo-safe, with the stale-guard and the two suppression rules. Snippet tabstops, `completionItem/resolve`,
`additionalTextEdits` (auto-import), and multi-cursor accept are OUT (see the spec's Out).

### Forge recall (§18.3)
- `bulletin-list` → none.
- The rules that land on this ticket, three of them written HOURS ago during #312:
  - **`PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`** — #313 is the THIRD
    consumer of the shared request path. Diff its guards against hover's/definition's and justify every
    omission. The concrete inheritance: `apply_definition_response`'s live-editor identity check (#312
    F4). For a popup a stale answer is worse than for a card — an ACCEPT against a stale list edits the
    wrong buffer.
  - **`PR-claude-new-overlay-register-at-every-choke-point-001`** — with its documented EXCEPTION: this
    popup must NOT go in `text_input_blocked`, because its printables have to fall through (D5).
  - **`PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`** — an injected response proves
    only the consumer. #313's P4 must assert the REAL `textDocument/completion` traffic (the
    `[[lsp.servers]]` tee).
- Also live: `BF-lsp-hover-extracted-helper-new-mutation-surface-001` (every new NAMED helper is a fresh
  cargo-mutants target → pair the extract with a direct unit or MSI goes red);
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001` (run `cargo mutants --list -f
  <file>`, never guess); the `mutants::skip` detach trap (verify with `--list` after inserting fns).

### Discovery (Explore, §18.2) — the precise surface
**REUSE AS-IS:**
- `marley_search_core::fuzzy_rank(candidates, query) -> Vec<Scored{index, score}>` (`lib.rs:34`) —
  better fit than `fuzzy_score`: it reuses ONE `Matcher` across candidates (matters at per-keystroke
  rates), returns score-desc/index-asc (stable + deterministic), and an EMPTY query yields all candidates
  in input order — exactly the "popup just opened" case. The finder already uses it (`finder.rs:57`).
- `complete::popup_window(len, selected, max) -> (usize, usize)` (`complete.rs:160`) — the visible-window
  math, a free pure fn, reusable verbatim.
- The whole #311/#312 LSP spine: `RequestPurpose` (`lsp_host.rs:33` — its doc ALREADY names #313),
  `request()` (`:448`, Ready-gated, 10s timeout drops the purpose), `take_responses()` (`:468`),
  `push_response_for_test` (`:493`), `lsp_position_for` (`app.rs:5930` — its doc names #313),
  `text_document_position_params` (`rpc.rs:221`), `consume_lsp_responses` (`app.rs:5986`).
- The caret→pixel recipe: `line_col` → `line_layout(line, tab_width).col_of_offset(in_line)`
  (`code_view.rs:42/53/97`) + `editor_geom` (`app.rs:486`), exactly as `hover_at_caret` does
  (`app.rs:5910`); and `hover_card_overlay` (`app.rs:6379`) for the cell→pixel anchor + its three guards
  (wrong-file, degenerate geometry, scrolled-out-of-viewport self-dismiss) + its child cap.

**THE #96 IDIOM TO MIRROR (state/keys/render — NOT the accept):**
- `CompletionState{candidates, entries, selected, start}` (`complete.rs:103`) + `refilter` (`:131`) +
  `move_up`/`move_down` (`:139/144`). It is an INLINE arm at `app.rs:7308`, not a `handle_*_key`.
- **The printable mechanism, confirmed:** the arm returns a `handled: bool`; printables/backspace return
  `false`, so only consumed keys `stop_propagation` — the char then types normally and a **post-edit
  refilter hook** (`app.rs:8084`) re-narrows. That two-phase "arm declines → post-edit refilter" shape is
  what #313 mirrors.
- ⚠️ Its ACCEPT (`app.rs:4515`) writes the TERMINAL input buffer — no undo groups, no `SelectionSet`, no
  IME. Not reusable.

**NET-NEW (ordered by risk, from Explore):**
1. **The editor printable/refilter seam** — HIGHEST uncertainty. #96's post-edit hook is on the TERMINAL
   path. Editor printables are CLAIMED-but-unhandled at `app.rs:7812-7822` and leave via
   `EntityInputHandler` → `ime::replace_text` (`ime.rs:100`) — "the ONE insert mechanism, IME included".
   There is NO editor-side refilter hook. Candidate sites: inside the `EntityInputHandler` impl
   (`app.rs:6947+`) or a pump tick keyed on `BufferVersion`. Phase 2 must pick and justify.
2. **`in_string_or_comment`** — see D2; net-new over `syntax_cache` (`app.rs:1023`), NOT a syntax-crate
   lookup. The cache is async/version-keyed → a staleness stance is required.
3. **The TextEdit-faithful accept, self-bracketed** — see D4. `edit_ranges_restoring` (the one engine
   shaped for "edit range ≠ cursor") is PRIVATE, so the accept brackets its own group around raw `edit()`
   calls; the Tab/indent site (`app.rs:7723`) is the shipped precedent and documents exactly this.
4. `parse_completion_result` + `strip_snippet_markers` + `completion_request_params` (the base
   `TextDocumentPositionParams` does NOT cover `context:{triggerKind, triggerCharacter}`).
5. `RequestPurpose::Completion(CompletionKey)` + its arm in the ONE drain + the key carrying
   **`BufferVersion`** (like `HoverKey`; `DefinitionKey` has none).
6. **`popup_origin` flip math** — `menu_origin` (`context_menu.rs:165`) CLAMPS only (it slides the box up
   over the caret line); no flip exists anywhere. Small pure fn, obvious truth table.
7. An editor `CompletionState` (borrows #96's idiom, reuses `popup_window`).
8. A trigger/debounce policy — no precedent (hover is dwell-triggered, goto is F12-triggered), and
   `REQUEST_TIMEOUT_TICKS` (10s) is tuned for a one-shot, not a per-keystroke stream.

### Decisions
D1 phantom `pair_action` DROPPED · D2 `in_string_or_comment` net-new over `syntax_cache` (+ a staleness
stance owed at design) · D3 hand-parse, no `lsp-types` · D4 the accept brackets its own undo group with
raw `edit()` · D5 NO `text_input_blocked` line (the deliberate exception) · D6 primary-cursor accept only
· D7 mirror #96's state/key/render, not its accept. Full text in the spec.

### Risks
- **R1 (highest) — the editor printable/refilter seam** (net-new #1). Getting it wrong makes the popup
  either not filter or eat typing.
- **R2 — the accept's undo bracketing** (D4). The empty-range coalesce is a REAL reachable hazard (a `.`
  trigger yields `start == end`), and its failure mode is the #299 incident: ⌘Z destroying the user's
  next keystroke along with the accept.
- **R3 — the stale-guard at per-keystroke rates.** Hover/goto fire once; this fires continuously. A
  dropped-vs-applied mistake here shows a menu for text that no longer exists.
- **R4 — `in_string_or_comment` staleness** (D2) — the gate may lag by a tick on the exact keystroke
  that matters.

### Ticket corrections filed
The two false claims (§4 `point_at`, §5 `pair_action`) are recorded in the local ticket doc + D1/D2.
**#301's description carries the identical `point_at` error** — correct it when #301 is worked.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture / approach
Third consumer of the #311 spine. Two pure layers + a shim, exactly as #311/#312 — but this feature is
STATEFUL (it fires per keystroke), so the design is the timing, not the payload.

**The two questions the plan owed, both resolved by reading the code:**

#### R1 RESOLVED — the editor printable/refilter seam
The editor has exactly TWO edit entry points, and #313 hooks both:
- **Printables** — the key arm CLAIMS but does not handle them (`app.rs:7812-7822`); they leave via
  `EntityInputHandler::replace_text_in_range` (`app.rs:~6990`) → `ime::replace_text` — "the ONE insert
  mechanism, IME included". **This is the hook site.** It already early-returns on `text_input_blocked()`,
  which per D5 will not list this popup, so typing reaches it.
- **Backspace** — the only other key that survives the router (`app.rs:~7888` →
  `input::apply_editor_key_multi` → `backspace_at_selections`).

Both call ONE new shim fn. Composition is excluded: after `ime::replace_text`, `marked.is_some()` means a
live IME composition (⌥E dead keys) — not a completion trigger.

**Every OTHER edit path (paste, cut, ⌘Z, ⌘/ comment toggle, Tab indent, Enter auto-indent, find-replace,
N-cursor edits) must DISMISS the popup, and hanging that off the two hooks would miss all of them** — the
#311 F1 lesson (`PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001`). So dismissal is a
PUMP POLL on the live editor identity: the menu records the `(path, version)` it is valid for, and the
pump drops it when the live pair diverges. Typing does not self-dismiss because the hook UPDATES the
recorded version as it refilters. Same shape as `HoverCard{path, version, caret_at_show}`.

#### D2 RESOLVED — the trigger is DEFERRED one tick, and that is what makes the string/comment gate work
The gate reads the app's `syntax_cache: Option<(nonce, BufferVersion, Rc<Vec<Vec<(Range, TokenKind)>>>)>`
(`app.rs:1023`). The decisive facts:
- `refresh_syntax_cache()` has **ONE call site — the RENDER** (`app.rs:8606`).
- Files **≤ `SYNTAX_SYNC_MAX_LINES` (1000)** parse **SYNCHRONOUSLY** there; only larger files go
  off-thread (M17 #274).
- `needs_syntax_refresh(cached, nonce, version)` is an exact `(nonce, version)` match.

Therefore **at the hook's moment the cache is ALWAYS one version behind** — the edit has landed but the
render has not run. A gate evaluated inline would be stale *every single keystroke*, in every file,
making it useless (and it would fail in exactly the direction that matters: the cache's comment span
still ends at the old EOL, so a caret typing INSIDE a comment reads as just past the span → popup opens).

**So the edit hook PARKS the trigger and the PUMP consumes it on the next tick — after the render has
refreshed the cache.** This is #312's `pending_center_row` idiom reused for the same reason (let the
frame settle, then act), and it costs ~16ms before the request goes out, which the server round-trip
dwarfs.

With the deferral: a file ≤1000 lines USUALLY has a fresh cache when the gate runs — which is the common
case. **Corrected at inspect (F12): "usually", not "exactly".** `cx.notify()` schedules a frame and the
pump is a detached 16ms timer; nothing orders them, so the render can lose the race and the cache still
lag. The consequence is BOUNDED rather than wrong: the freshness check is semantically identical to the
shipped `needs_syntax_refresh`, so a lost race degrades to the same stance as the >1000-line case, never
to a confidently-wrong answer. A >1000-line file may lag for real (async) → **the gate FALLS OPEN rather
than suppressing.**
Stated plainly as a limitation, with the reason: suppressing on a stale cache would make the popup fail
to appear in exactly the large files where completions matter most, whereas falling open costs a popup in
a comment that Esc dismisses and the next tick suppresses. Freshness is checked with the existing
`(nonce, version)` equality — no new predicate.

#### The layers
**1. PURE — `marley_lsp/src/completion.rs` (NEW):** hand-parsed per D3 (the `Diag`/`HoverRun` precedent —
the app never sees `lsp-types`).
- `CompletionItem { label, sort_text, filter_text, detail, kind, edit }` — Marley-local.
- `CompletionEdit { range: Option<(Position, Position)>, new_text: String }` — the ONE normalized edit,
  collapsing `textEdit{range,newText}` / `InsertReplaceEdit{replace|insert,newText}` / bare `insertText` /
  label-only. `range: None` = "no server range, use the client-computed word start" (the legacy path).
- `parse_completion_result(&Value) -> CompletionResponse { is_incomplete, items }` — accepts BOTH a
  `CompletionList{isIncomplete, items}` and a bare `CompletionItem[]`; `null`/absent → empty; a malformed
  element is SKIPPED (the per-item `filter_map`, same argument as #312 — a typed `from_value::<Vec<_>>`
  rejects the whole list on one bad element).
- `strip_snippet_markers(&str) -> String` — `$0`, `${1:name}`, `\$` escapes.
- `completion_request_params(uri, line, character, trigger: Option<&str>) -> Value` — delegates to
  `rpc::text_document_position_params` and ADDS `context:{triggerKind, triggerCharacter}` (the base shape
  does not cover it).

**2. PURE — `marley_app/src/editor_complete.rs` (NEW):**
- `CompletionKey { uri, line, character, version }` — **carries the buffer version** (like `HoverKey`,
  unlike `DefinitionKey`): an as-you-type popup MUST drop an answer for a buffer that has since changed.
- `TriggerDecision { Open, Refilter, Close, Ignore }` + `completion_trigger(typed, trigger_chars,
  in_str_or_comment, popup_open, suppressed) -> TriggerDecision` — the ONE table.
- `CompletionMenu { items, order: Vec<usize>, selected, query }` — `refilter(query)` (via
  `fuzzy_rank` over `filter_text ?? label`, then stable sort by (score desc, `sort_text`, `label`)),
  `move_up`/`move_down` (wrapping, mirroring `complete.rs`), `chosen()`.
- `popup_origin(anchor_y, cell_h, popup_h, win_h) -> f32` — the FLIP (`menu_origin` only clamps, which
  would slide the box over the caret line).
- `word_start(text, caret) -> CharOffset` — the fallback replace-start when the server sends no range.
- `token_kind_at(lines, row, byte_col) -> Option<TokenKind>` — the pure half of the string/comment gate
  (a span scan); the shim supplies the cached lines + the freshness check.

**3. SHIM — `marley_app/src/app.rs`:** fields `completion_menu`, `completion_request`,
`pending_completion_trigger`, `completion_suppressed`; the two edit hooks (park); the pump consume
(gate → send) + the identity poll (dismiss); `apply_completion_response` as a new arm in the ONE
`consume_lsp_responses` drain (never a second drain); `accept_completion` (D4); the overlay; the key arm.

### §20 confirmation
**Zed (the editor).** The design matches the observed universal behavior — a popup appears under the
caret as you type, ranks by the typed prefix, and accepting REPLACES that prefix — via the published LSP
3.17 `textDocument/completion` wire (`CompletionList`/`CompletionItem`/`TextEdit`/`InsertReplaceEdit`/
`isIncomplete`) plus Marley's OWN `fuzzy_rank`, `complete::popup_window`, `Buffer`, and undo model. The
replace-not-append semantic comes from the spec itself (`CompletionItem.textEdit` names an explicit
range), not from reading anyone's source. No Zed GPL source read. Confirmed clean-room.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/completion.rs` | NEW pure: `CompletionItem`, `CompletionEdit`, `CompletionResponse`, `parse_completion_result`, `strip_snippet_markers`, `completion_request_params`. |
| `crates/marley_lsp/src/lib.rs` | export the completion items. |
| `crates/marley_app/src/editor_complete.rs` | NEW pure: `CompletionKey`, `TriggerDecision`, `completion_trigger`, `CompletionMenu`, `popup_origin`, `word_start`, `token_kind_at`. |
| `crates/marley_app/src/lib.rs` | `mod editor_complete;`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::Completion(CompletionKey)`. |
| `crates/marley_app/src/app.rs` | 4 fields; the `replace_text_in_range` + backspace park-hooks; the pump consume (gate+send) and the identity dismiss-poll; `apply_completion_response` in the ONE drain; `accept_completion` (own undo group, raw `edit()`); `completion_popup_overlay` (+ `popup_origin`, the #311 cell→pixel anchor + its three guards + a child cap); the key arm (↑/↓/Enter/Tab/Esc consumed, printables DECLINED). |
| `crates/marley_app/src/headless_drive.rs` | the drives below. |

### Regression Test Plan (≥1 per REQ)
| Test (loc) | REQ | Asserts |
|---|---|---|
| `editor_complete::tests::trigger_table` | 001/006/007 | the full truth table: an identifier char / a server trigger char / a non-trigger; suppressed-after-Esc; in-string and in-comment → `Ignore`. |
| `completion::tests::parse_*` | 002 | BOTH shapes (`CompletionList` + bare array); each edit form (`textEdit`, `InsertReplaceEdit`, `insertText`, label-only); `null`/`[]` → empty; malformed elements skipped; a mixed array keeps the good ones. Positions OFF-GRID (neither 0 nor 1) to kill the `Some((0|1,0|1))` mutants. |
| `completion::tests::strip_snippet_markers_*` | 002 | `$0`, `${1:name}`, `\$` escaped, no markers → unchanged. |
| `completion::tests::completion_request_params_shape` | 001 | the base position params + `context{triggerKind, triggerCharacter}`. |
| `editor_complete::tests::menu_refilter_ranks_and_is_stable` | 003 | `pu` ranks `push` top; ties broken by `sort_text` then `label`; an EMPTY query preserves server order. |
| `editor_complete::tests::menu_move_and_chosen` | 004 | wrap up/down; `chosen()` follows; the degenerate 0/1-item cases. |
| `editor_complete::tests::popup_origin_flips` | 004 | fits below → below; would overflow → ABOVE the caret row; the exact boundary; a popup taller than the window clamps, never negative. |
| `editor_complete::tests::word_start_*` | 005 | the fallback replace-start over identifier chars; at BOL; after punctuation; non-ASCII. |
| `editor_complete::tests::token_kind_at_*` | 006 | inside a `Str` span / a `Comment` span / between spans / past the last span / an empty line → `None`. |
| `editor_complete::tests::completion_key_stale` | 008 | equality incl. the VERSION field (a version bump ⇒ a different key). |
| `headless_drive::lsp_completion_opens_and_ranks_headless` | 001/003/004 | type a trigger → parked → pump → request sent; feed a response → the menu opens ranked; ↑/↓ move. |
| `headless_drive::lsp_completion_accept_replaces_prefix_headless` | 005 | type `pu`, accept `push` → the buffer reads `push` (REPLACED, not `pupush`). |
| `headless_drive::lsp_completion_accept_undo_is_not_absorbed_headless` | 005 | **the D4 hazard, explicitly**: accept an EMPTY-range item (the `.`-trigger shape), type a char, ⌘Z → ONLY the char reverts; a second ⌘Z reverts the accept. Pre-fix this test is what fails. |
| `headless_drive::lsp_completion_stale_and_moved_on_dropped_headless` | 008 | a response for an older version → dropped; a response whose file is no longer focused → dropped. |
| `headless_drive::lsp_completion_esc_suppresses_headless` | 007 | Esc closes; a typed identifier char does NOT reopen; a trigger char DOES. |
| `headless_drive::lsp_completion_other_edit_dismisses_headless` | 004 | with the popup open, a paste/undo (an edit that bypasses the two hooks) → the pump poll dismisses it. |
| **A LIVE WIRE CHECK (P4)** | 001 | per `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`: assert the REAL `textDocument/completion` frame via the `[[lsp.servers]]` tee — an injected response proves only the consumer. **Assert the PAYLOAD (a non-empty position + context), not just the method name.** |

Uncoverable-by-unit: the popup's pixel placement (the harness renders no pixels — gpui 0.2.2's test
platform `draw()` is a no-op); carried by `popup_origin`'s pure truth table + the live capture.

### Risks / decisions
- **R1 (resolved above)** — two hooks + a pump dismiss-poll. The hooks know the typed char; the poll
  catches every other edit path.
- **R2 — the accept's undo bracketing (D4).** THE correctness risk. `edit_at_selections` SELF-BRACKETS
  and `begin_group` silently discards an open group's records, so the accept must bracket its own group
  around RAW `edit()` calls (the Tab/indent site, `app.rs:7723`, is the shipped precedent and says so).
  The empty-range coalesce is reachable via a `.` trigger; its test above is written to FAIL without the
  bracket.
- **R3 — the stale-guard at per-keystroke rates.** The key carries the version; the arm re-checks the
  live editor identity exactly as `apply_definition_response` does. `isIncomplete` re-queries on the next
  trigger (it does not need a separate mechanism — the next keystroke parks a fresh trigger).
- **R4 — the D2 large-file window.** >1000 lines → the cache may lag → the gate falls open. Documented
  limitation, not a silent one; a follow-up could make the gate exact by asking the syntax layer directly
  (which today exposes no node query — see the #301 correction).
- **D-des-1 — `fuzzy_rank`, not `fuzzy_score`** — one `Matcher` reused across candidates (this runs per
  keystroke), stable/deterministic ordering, and an empty query returns all candidates in server order,
  which is exactly the just-opened case.
- **D-des-2 — every new NAMED pure fn gets a DIRECT unit** (`BF-lsp-hover-extracted-helper-new-mutation-surface-001`);
  run `cargo mutants --list -f <file>` for the REAL set rather than guessing operators, and re-run it
  after inserting fns near `mutants::skip` shims (the detach trap).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`crates/marley_lsp/src/completion.rs` (NEW, pure)** — `CompletionItem` / `CompletionEdit` /
  `CompletionResponse` (Marley-local; the app never sees `lsp-types`, per D3);
  `parse_completion_result` (BOTH shapes — `CompletionList{isIncomplete,items}` and a bare array;
  per-element `filter_map` so one bad entry cannot lose the list); `edit_of` normalizing the FOUR insert
  forms with `textEdit` → `insertText` → label precedence and **preferring `replace` over `insert`** on an
  `InsertReplaceEdit` (replace is the range covering what the user already typed — the point of the
  feature); `completion_request_params` (delegates to `rpc::text_document_position_params`, adds the
  `context{triggerKind, triggerCharacter}` the base shape lacks); `strip_snippet_markers` (`$0`, `${1:x}`
  keeping the placeholder, `${1|a,b|}` keeping the first choice, `\$` escapes, an unterminated `${`
  degrading to literal text); `trigger_characters` reading `completionProvider.triggerCharacters` out of
  the raw caps #308 kept for exactly this purpose ("typed readers come with #309+").
- **`crates/marley_app/src/editor_complete.rs` (NEW, pure)** — `CompletionKey` (uri+line+character+
  **version**); `TriggerDecision`/`TriggerKind` + `completion_trigger` (the one table);
  `CompletionMenu` (the server snapshot + a ranked `order`, `refilter` via `fuzzy_rank` then the LSP
  tie-break `sortText`→`label`, wrapping `move_up`/`move_down`, `chosen`, `visible` reusing the shipped
  `complete::popup_window` verbatim); `popup_origin` (the FLIP); `word_start`; `token_kind_at` +
  `in_string_or_comment`.
- **`crates/marley_app/src/lsp_host.rs`** — `RequestPurpose::Completion(CompletionKey)` +
  `completion_trigger_chars()`.
- **`crates/marley_app/src/app.rs`** — 4 fields; the TWO typing hooks (`replace_text_in_range` for
  printables — gated on `marked.is_none()`, since a live IME composition is not typing a word — and the
  backspace arm); `consume_completion_trigger` on the pump (the deferred gate → open/refilter/close);
  `refilter_completion`; `send_completion_request`; `apply_completion_response` as a new arm in the ONE
  drain; `accept_completion_item` (D4); `dismiss_stale_completion` (the identity poll);
  `handle_completion_key` (consumes ↑/↓/Enter/Tab/Esc, DECLINES everything else);
  `completion_popup_overlay`; `caret_in_string_or_comment` (the freshness check + the coordinate map).

### Deviations from design (with reason)
- **`accept_completion` → `accept_completion_item`.** The name was already taken by #96's SHELL popup
  accept (`app.rs:4589`), and defining a second one produced `E0592 duplicate definitions` plus `E0034`
  at three existing call sites. The compiler caught it; the rename keeps #96 untouched.
- **`editor_complete` uses `crate::code_syntax::TokenKind`, not `marley_syntax::TokenKind`.** The design
  said "the cached `Vec<Vec<(Range, TokenKind)>>`" without noticing the app's cache holds its OWN kind
  (the syntax crate's kinds are mapped through `code_syntax::kind_from_syntax` on the way in). The
  compiler caught this too — the two types have the same name and are distinct. `Str`/`Comment` exist on
  both, so the gate is unchanged.
- **`caret_in_string_or_comment` probes `byte_col - 1`, not `byte_col`.** A caret typing at the END of a
  comment sits one byte past the last span byte, so probing the caret itself would answer "not in a
  comment" for the exact case the gate exists for. It probes the char BEFORE the caret (saturating at 0).
- **`park_completion_backspace` parks an EMPTY string**, and the pump special-cases it into a refilter
  rather than routing it through `completion_trigger`. Backspace can never OPEN a popup, and the empty
  string is not a char the trigger table has an opinion about; the table stays a table about *typed
  characters*.
- **`truncate_cols` (code_view.rs) widened `fn` → `pub(crate)`** so the popup's detail column reuses the
  shipped truncation instead of a second one. No behaviour change.
- **`send_completion_request` returns `false` (no repaint) on a not-ready host and never flashes** —
  unlike F12, which flashes "LSP: not ready". Typing must not nag: a passive as-you-type feature that
  scolds you on every keystroke in a non-Rust file would be unusable. Same reasoning as #311's dwell.

### Verified
`cargo check --workspace --all-targets` clean (0 errors); `cargo clippy -p marley -p marley_lsp
--all-targets` clean; `cargo fmt --all --check` clean.

**The `mutants::skip` detach trap did NOT fire** — `cargo mutants --list -f app.rs` is still **64**, the
same count #312 closed at, with `def_label` still present and NONE of the ten new skip-attributed shim
fns appearing. Real mutant sets recorded for Phase 4 (traced, not guessed —
`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001`): **completion.rs = 28**,
**editor_complete.rs = 59**. `popup_origin` alone carries **11** (three FnValue bodies `0.0`/`1.0`/`-1.0`,
the `<=`→`>` and `>=`→`<` boundary swaps, and six arithmetic swaps) — its truth table must pin the
fits-below, must-flip, and neither-fits cases at exact boundaries to kill them all.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Two critics over the ~1130-line diff: **reuse/integration/idiom-fit** and **data-state/parse-robustness**.
Both were briefed with this session's own fresh rules — `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`
and `PR-claude-two-gated-calls-must-read-the-phase-once-001` (both written hours ago, on #312's inspect
and the #309 didOpen bug) — and told that a false positive costs more than a miss. Both probe-proved
their findings rather than reasoning. **Three HIGHs, and one of them is my own rule being violated.**

### REAL — fixed in #313

| # | Sev | Finding | Fix |
|---|---|---|---|
| **F1** | **HIGH** | **The accept applies the server's REQUEST-TIME range to a buffer typed into since — `s.` → popup → type `p` → Enter gives `s.pushp`.** The menu is installed only when `version == key.version`, so its ranges are exact at install. But the popup then SURVIVES typing: a word char re-filters the snapshot and re-stamps the version (which is exactly what stops the dismiss poll dropping it), while the items' ranges are never re-fetched. The accept used them verbatim. This is REQ-005 — the feature's entire correctness claim — failing on its most common gesture, and it fires precisely when the server DOES send a `textEdit`, i.e. always with rust-analyzer. **The `None` fallback arm was correct, which is what hid it.** Worse: the design's own planned test ("type `pu`, accept `push`") accepts immediately after opening and would have passed green straight over this. | EXTEND the server range to the LIVE caret: `(a.min(b).min(caret), a.max(b).max(caret))`. LSP requires the range to contain the request position, so the client owns everything typed since. The min/max also normalizes an inverted range as a second layer under F3. The `None` arm now reads the live LINE (folding in F4). |
| **F2** | **HIGH** | **No `caret` in the popup's identity — a bare motion does not dismiss, and the accept then edits where the user is not looking.** #311's `HoverCard` carries `caret_at_show` for exactly this ("ANY caret move bumps it… arrows are handled inline, never reaching `dispatch_action`"). #313 stored only `(menu, path, version)`. A caret move bumps NO version, and the popup's key arm DECLINES ←/→/home/end (they must reach the editor's motion path), while a mouse click never touches the key router at all. So: type `s.pu` → popup → press ← → popup survives → Enter → the accept fires at the ORIGINAL offset and yanks the caret there. **This is `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` — my own rule from #312 this morning — violated: I inherited hover's version guard and dropped its caret guard without saying why.** | The 3-tuple became a real `OpenCompletion { menu, path, version, caret, is_incomplete }` struct (it was already straining), stamped on install and RE-STAMPED by each refilter, checked by the dismiss poll. Typing survives (the refilter re-stamps); a bare motion dismisses. |
| **F3** | **HIGH** | **No modifier gate on the popup's key arm — ⇧Tab silently ACCEPTS instead of dedenting, and ⌘⌥↑/↓ steal the shipped M19 multi-cursor gesture.** The arm matched `"enter" \| "tab"` and `"up" \| "down"` on the bare key name and never read `.modifiers`, while sitting ABOVE both the keymap and the editor's Tab arm in the ladder. ⇧Tab is a wrong EDIT, not a dead key. #96's arm — the very idiom #313 claims to mirror — has carried this gate all along and says why: *"Only UNMODIFIED nav keys drive the popup… a ⌘/⌃/⌥ chord falls to `_`, dismissing and dispatching normally"*. #313 copied the shape and dropped the guard. | Adopt #96's gate verbatim: any platform/control/alt/shift modifier returns `false` (declined) before the match. |
| **F4** | MED | **`word_start` materialized the WHOLE BUFFER as a `Vec<char>` on every keystroke** — ~300KB `String` + ~1.2MB `Vec<char>` + three full decode passes on a 300KB file, to walk back three characters. The critic checked whether this is house idiom: it is NOT — every other `Vec<char>` collect in the codebase is scoped to a LINE or a needle. The shape was inherited from #96's refilter, where `.text()` is free because the terminal's buffer is ONE LINE; transplanting it onto an editor rope moved it 3-4 orders of magnitude, onto the hottest path in the editor. | `word_start(text, caret)` → **`word_query(line, caret_in_line) -> (usize, String)`** — line-local (word chars never cross `\n`), and it returns the (start, query) PAIR the two shim sites were hand-rolling separately (folding in F8). Mirrors the shipped `complete::current_word`'s signature. |
| **F5** | MED | **`is_incomplete` was parsed, documented as load-bearing, and never read — and the trigger table structurally could not re-query.** Its own doc says "the next keystroke must re-query rather than re-filter", but a word char with the popup open always returned `Refilter` → no request, ever. So when a server sets it (rust-analyzer commonly does), typing narrows a PARTIAL snapshot and correct candidates silently never appear. The notes' R3 claim ("the next keystroke parks a fresh trigger") was contradicted by the table it described. | Thread it: `OpenCompletion.is_incomplete`, and the `Refilter` arm re-queries instead of narrowing when it is set. |
| **F6** | MED | **An inverted server range (`end` before `start`) parsed fine and PANICKED the app in ropey** at `Buffer::edit(start..end)` → `rope.slice(40..12)`. Probe-proven end to end. Directly contradicts the module's own contract ("a bad payload degrades to 'no candidates' and never to a panic") — true of the parse, false of the system. | `range_of` rejects the inversion at the seam, where the promise is made; the item falls back to the client's own (always well-formed) word range. F1's min/max is the second layer. |
| **F7** | MED | **`strip_snippet_markers` mis-parsed NESTED placeholders** — probe: `${1:${2:nested}}` → **`${2:nested}`**, i.e. raw LSP snippet syntax inserted into the buffer as literal text. The flat scan broke on the FIRST `}` with no depth counter, and `placeholder_of`'s result was emitted without re-stripping. Nested placeholders are spec-legal (a placeholder's body is `any`). Currently dormant (the handshake advertises no `snippetSupport`, so a compliant server sends `insertTextFormat: 1`) but live for any server configured via `[[lsp.servers]]` that emits snippets anyway — and it is a pure seam REQ-002 names and Phase 4 is about to pin at MSI 100. | Track brace depth in the body scan, and recurse the extracted placeholder through `strip_snippet_markers` so inner markers strip too. |
| **F8** | MED | **The Esc suppression latch had no word-boundary reset — one Esc silenced completions until the next `.`/`::`.** The latch cleared only on `Open` or an accept, but a word char while suppressed returns `Ignore` WITHOUT clearing, so it could never reach `Open`. Type `foo` → Esc → and no identifier anywhere on the rest of the line (or the file) reopens; writing `let mut counter = 0;` gets nothing until you happen to type a `.`. The fn's own doc scopes it to "the rest of a word"; the built latch outlived the word, the line, and the file. REQ-007's letter was satisfied — this is a state-machine gap, not a spec disagreement. | New `TriggerDecision::EndWord`: a non-word, non-trigger char ends the dismissed word, and the shim clears the latch there. |
| **F9** | LOW | `kind` was parsed with a careful `try_from` and never rendered — exported-but-unread (the #311-F3 rule). The ticket asks for a kind glyph. | Pure `kind_glyph(Option<u32>) -> &'static str`, rendered in a fixed-width cell so labels stay aligned. The field now earns its parse. |
| **F10** | LOW | `refilter`'s `score_of: HashMap` + `.unwrap_or(0)` is UNREACHABLE (both `order` and `score_of` derive from the same `ranked`, whose indices are unique) — but a silent `0` default is the wrong shape for an invariant: if it ever were reachable the effect is a silently mis-ordered menu. | Sort the `Vec<Scored>` directly — the score already rides on it. Totality is now structural rather than defended, and a per-keystroke HashMap disappears. |
| **F11** | LOW | `strip_snippet_markers` `pub` + re-exported, `token_kind_at` `pub` — both read only inside their own module (their file-mates `is_word_char`/`single_char`/`sort_key`/`plain` are all private). | Both private; the crate re-export trimmed. In-module tests still reach them. |

### REAL — notes corrected, no code change
- **F12 (LOW) — D2's "the gate is exact" is best-effort, not a property.** The critic was asked to prove
  the park→render→consume ordering and answered honestly that it cannot be proven and probably does not
  hold: `cx.notify()` schedules a frame, the pump is a detached 16ms timer, and nothing orders them. The
  **consequence is bounded** — the freshness check (`cn != nonce || cv != version`, semantically identical
  to the shipped `needs_syntax_refresh`) makes a lost race fall open exactly like the >1000-line case, so
  it never yields a confidently-wrong answer. What was wrong was only the notes' claim that a small file
  "has a FRESH cache when the gate runs → the gate is exact". **Corrected: the deferral makes the cache
  USUALLY fresh; a lost render race falls open like the large-file case.** Recorded so nobody builds a
  suppression-critical feature on a guarantee that is not there.

### REJECTED (with reason)
- **"The two completion popups can collide."** My own primary suspicion, and the critic disproved it
  properly rather than agreeing: #96's render force-dismisses on any non-terminal tab
  (`active_is_terminal` → an empty `rect_list` → `self.completion = None` on the FIRST frame), and
  `grid()` is `None` for a `CodeView` tab. So #96's popup cannot be open on an editor tab, and #313's
  cannot be open on a terminal tab. The `text_input_blocked` NOTE that implies otherwise is stale prose
  that predates #313 and is not this diff's to fix. (Ladder order recorded for the future: hover-Esc →
  def_picker → **#313** → renaming_tab → **#96** → context_menu → keymap → editor Tab.)
- **The overlay's 3rd copy of the card recipe** — correctly waits for **#318**. #313 adds no new variant,
  reuses the same tokens and the accent idiom, and widened the shipped `truncate_cols` rather than
  writing a second truncator. Extracting for a third caller now would pre-empt #318's mechanical job.
- **`complete::popup_window` reuse + the `selected < order.len()` invariant** — verified correct on every
  path (`refilter` resets to 0 and its `false` return closes; `move_*` early-return on empty and wrap mod
  len; `new` sets 0; the response arm rejects empty items). No panic.
- **The `BF-lsp-didopen-carries-empty-text-ready-race-001` class does not recur.** The pump order is
  `consume_lsp_responses` → `consume_completion_trigger` → `dismiss_stale_completion`, so the refilter's
  re-stamp lands BEFORE the dismiss poll reads it — typing cannot self-dismiss. And
  `apply_completion_response`'s two `active_editor()` reads use the SAME bindings (read once, stored),
  honoring `PR-claude-two-gated-calls-must-read-the-phase-once-001`.
- **The double-park** (type fast: park `a`, park `b`, the first never consumed) is benign — the refilter
  and the request both read the LIVE buffer, so the parked char selects the decision, not the data. The
  only degradation is a `.` + fast next char yielding `triggerKind: 1` instead of 2.
- **`completion_request` leaking a `Some` on timeout** — benign and identical to #312's
  `definition_request`: read only as a supersede check, never as a send gate.
- **Parse robustness otherwise** — all probed: `null`/`[]`/`{}`/`{"items":null}`/`{"items":"nonsense"}`/
  `[null,null]`/bare string/number/no-label/non-string-label → 0 items, no panic. Malformed range,
  negative line, line 2^32 → `range: None` (the `try_from` guard). Mixed good/bad keeps the good.
  `InsertReplaceEdit` with both → `replace` wins (correct — it covers what was typed). Snippet stripping
  applied at all three edit sites. Deep nesting: serde_json rejects at depth 128 first.

### Verified after the fixes
`cargo check --workspace --all-targets` clean; `cargo clippy -p marley -p marley_lsp --all-targets`
**0 issues**; `cargo fmt --all --check` clean; **595 tests pass**. The `mutants::skip` detach trap did
NOT fire — app.rs still **64**, unchanged since #312 closed.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate — **COMPLETE (resumed 2026-07-15 after the stop).**

### Tests added (26 new; all green)
**`marley_lsp/completion.rs` (11):** the request params + `context` (both trigger kinds);
`trigger_characters` off the raw caps (incl. junk entries skipped); BOTH response shapes; all FOUR edit
forms incl. the `insert`-only rung and `replace`-wins-over-`insert`; the optional fields + `kind`'s
`try_from` guard; 11 malformed payloads → no candidates, no panic; a mixed array keeping the good;
**F6** — an inverted range REJECTED both by-line and by-character, while an EMPTY range (the `.`-trigger
shape) survives; malformed positions dropping the RANGE not the item; snippet stripping on all three
edit paths (and NOT on format 1); **F7** — the snippet grammar incl. `${1:${2:nested}}` → `nested`, the
`\$` escape guard, and the `&&` depth-scan guard.
**`marley_app/editor_complete.rs` (9):** the full trigger table incl. **F8**'s `EndWord`; rank + the LSP
tie-break (with EQUAL fuzzy scores — 36 == 36, probe-verified — so the comparator actually runs);
move/chosen + the degenerate sizes; the windowed `visible`; **`popup_origin`'s flip at exact
boundaries** (11 mutants); `word_query`; `in_string_or_comment`; `kind_glyph`; the `CompletionKey`
version identity.
**Headless drives (4):** opens+ranks; **the F1 type-then-accept regression**; the D4 undo
non-absorption; stale + superseded responses dropped.

### The tests have TEETH (verified by reverting, not assumed)
- **F1:** reverting the live-caret range extension → the drive FAILS and reproduces the bug verbatim:
  `"    vec_opse\n"` — the `e` typed after the popup opened, stranded after the insert. **This is the
  test the design would NOT have written**: the planned one accepted immediately after opening and
  passes straight over the defect. It only exists because the inspect critic found the bug first.

### Gate — 14/15, coverage RED at ONE line
```
PASS gate:1 rustfmt · gate:2 clippy · gate:3 tests · gate:7 audit · gate:8 deny · gate:9 machete
PASS gate:10 gitleaks · gate:11 shellcheck · gate:12 no-suppressions · gate:13 SAST · gate:14 docs
PASS gate:5 mutation (MSI >= 100%) · gate:6 miri · gate:15 visual/AX
FAIL gate:4 rust coverage (>= 100% lines)
```
`cargo nextest run --workspace` = **1281 passed**. **MSI 100 PASSES** — the two survivors were both real
test gaps and are dead: `sort_key`'s mutants survived because the "tie" test never tied (`refilter("a")`
over `["bbb","aaa"]` filtered `bbb` out entirely, so the comparator never ran), and
`strip_snippet_markers` had two unexercised guard branches.

**The unresolved red:** `marley_app/src/editor_complete.rs` reports **1 missed line / 1 missed function**
(99.73%); `marley_lsp/src/completion.rs` was closed to 100% by covering `edit_of`'s `insert`-only rung.
Attempts made: covered `move_up`-from-a-middle-row, covered the suppressed+popup-open row, deleted the
dead `completion_key_for_test`, and dropped `CompletionMenu`'s unwired `Clone`/`PartialEq`/`Eq` derives
(correct on the #311-F3 rule regardless of coverage).

### The coverage red — RESOLVED, and it was NOT phantom

The previous session's read ("the tooling contradicts itself → probably the `parse.rs` phantom-region
artifact → argue an exclude") was **wrong, and the evidence that said so was itself the artifact**:
`cargo llvm-cov report` was being run against **stale `.profraw` from an earlier build**. That single
fact explains every confusing datapoint at once — the per-line `--text` export disagreeing with the
summary, and above all "the counts did not move after a real code change removed derives" (of course
they did not; the report was never measuring the new code).

A clean `cargo llvm-cov clean --workspace` + re-run reproduced the numbers **exactly** (671/3 · 41/1 ·
369/1), proving the measurement real and stable rather than flaky. The `--json` export then named the
line, where `--text` could not: the JSON lists functions **per instantiation** (the crate compiles into
two objects — hashes `CsghHCtDPXlv4` and `Cs1sikSq0HlsQ`), and the summary merges them by taking the
max. 23 zero-count entries appear; 22 have a covered sibling. **Exactly one has none:**

```
refilter::{closure#1}::{closure#1}  @ line 215   — zero in BOTH instantiations
```

That is `refilter`'s THIRD tie-break rung, `.then_with(|| … label.cmp(&… label))`. `then_with` only
runs when the previous comparison returned `Equal`, so reaching it needs an equal fuzzy score **and an
equal `sortText`**. The suite never produced that: the `nosort` case *looks* like it covers the label
tie-break, but with no `sortText` `sort_key` already RETURNS the label, so that tie breaks one rung
earlier at line 213. Only a server sending the SAME `sortText` for two candidates gets there — which
rust-analyzer does routinely. **A real, reachable, untested branch.**

Fixed at source with one test (equal score + equal `sortText` + different labels), **proven to bite by
deleting the rung**: `bxx` stays first (stable sort keeps server order) and the test fails with the
exact expected diff. Coverage then closed completely — `editor_complete.rs` **689/0 regions · 41/0 fns
· 375/0 lines = 100%**, workspace **26176/0 lines, 3102/0 fns**, `--fail-under-lines 100` exits 0. **No
exclude, no floor change, no suppression (§0).**

**NEW RULE (forge): `PR-claude-llvm-cov-stale-profraw-masquerades-as-a-phantom-region`** — before
concluding "phantom", `llvm-cov clean` + re-run; if the numbers reproduce they are real. Reach for
`--json` (per-instantiation, names the symbol) over `--text` (merges, hides it). A *"the counts didn't
move after I changed the code"* observation is not evidence of a tooling artifact — it is near-proof
the report is stale.

### The LIVE drive — DONE (and it retires #310's "orchestration-blocked")

Driven against a **real rust-analyzer** through an `[[lsp.servers]]` tee wrapper
(`PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`), on a throwaway probe cargo
project (indexes in seconds; the Marley repo would take minutes and risk typing into real source).

- **REQ-001 — the PAYLOAD on the wire, not the method name:**
  `{"method":"textDocument/completion","params":{"position":{"line":2,"character":14},
  "context":{"triggerKind":2,"triggerCharacter":"."}}}`. `character:14` is exactly past the `.` on
  `    let n = s.` (4 spaces + 10 chars), and `triggerKind: 2` (Character, not Invoked) proves the
  trigger table classified it — a constant would not.
- **The #312 BF stays fixed live:** `didOpen TEXT_LEN = 55` carrying the real source
  (`'fn main() {\n    let s = String::new();\n    let n = s\n}\n'`), not the zero-length document.
- **REQ-005 — byte-exact off disk** (accept → ⌘S → read the file, no pixel-reading):
  `'…\n    let n = s.parse\n}\n'`. The typed `p` was REPLACED, not appended. No `s.pparse`/`s.parsep`.
- **REQ-003/004** — the popup opened under the caret with real String methods, narrowed correctly on
  `p` (`parse`/`pop`/`push`/`push_str`/…), `ƒ` kind glyphs (F9), 8-row cap, dismissed on accept.
- **F5 confirmed live:** the `.` and the `p` each sent a request (2 total) — the `isIncomplete`
  re-query, not a narrow of a stale snapshot.
- **REQ-006 / the D2 deferred gate — the design's riskiest call, proven A/B with a CONTROL.** Same
  file, same keystroke, one line apart: `.` typed inside `// comment s` → **0 requests, no popup**
  (with `lsp: ready` in the footer, so the absence is suppression and not a dead server); `.` typed on
  `let t = s` → **1 request + popup**. The notes could only argue D2 on paper; it holds live.
- Also incidentally live-proven for the first time (they shipped headless-only): **#308** `lsp: ready`
  against real rust-analyzer, **#309** doc sync, **#310** diagnostics (`2 errors, 1 warning` + real
  squiggles).

### One REAL finding the drive caught that no unit could — FIXED here

**The selected row's detail column was illegible.** `completion_popup_overlay` set `.text_color(muted)`
on the detail CHILD and `on_accent` on the row PARENT — an explicit child color does not inherit, so
`muted` (tuned against `surface`) survived onto the selected row's accent wash and rendered as a
near-invisible salmon on cyan. Pixel-sampled, not eyeballed: detail `rgb(74,191,207)` **≡ the
background**, vs a clean light-gray-on-dark on every unselected row. Every sibling accent row in the
app pairs `bg(accent)` with `on_accent`; this one row did not. Fixed by gating the detail's color on
`is_selected` — re-driven and re-sampled: `rgb(42,95,108)` dark-on-cyan, legible, with the unselected
rows unregressed. Shim-only (`mutants::skip` + coverage-excluded), so no MSI/coverage churn.

### Drive-harness lessons (forge)

- **`PR-claude-selftest-seed-workspace-shell-to-unlock-the-app-bundle`** — **this retires the #310
  "a LIVE-ra PIXEL capture is orchestration-blocked (`open` forces cwd=/)" claim.** cwd only ever
  mattered for *auto-discovering* the project. Seeding `workspace.shell` (the #163 codec:
  `"0\n<root>\t0\tT=t"`) in settings makes the root explicit, so the `.app` bundle's cwd=`/` is
  irrelevant — and a bundle CAN be `System Events`-activated, which a bare binary cannot. `open
  target/Marley.app` + `osascript … set frontmost` = deterministic frontmost. Do this instead of
  launching the bare binary and hoping it is not covered.
- **`PR-claude-selftest-drive-focus-verb-is-layout-dependent`** — `drive.swift`'s `focus` clicks
  `(center-x, 12%-down)`, which on the current layout lands in the **Files pane**, not the editor. It
  is not a generic "make Marley frontmost" verb; for an editor drive it steals focus. Activate via
  System Events instead.
- **`PR-claude-selftest-a-suppression-test-needs-a-positive-control`** — my first REQ-006 A/B read
  `0 requests` in the comment and I was one step from scoring it PASS. The **code control also read
  `0`**, which is what exposed that the whole drive was dead (the LSP had not spawned). A test that
  asserts *absence* proves nothing without a paired positive that must fire.
- **I typed into another agent's session — the documented trap, live.** iTerm2 (a concurrent Claude
  agent) came frontmost and covered Marley; `screencapture -l<winid>` kept capturing Marley correctly
  *from behind*, which masked it completely, while the synthetic `.` keystrokes went into that agent's
  prompt. Verified via a FULL-SCREEN capture (the only thing that shows layering), and cleaned up (two
  targeted backspaces; that session was otherwise untouched and still running). **A window-scoped
  capture cannot prove your input landed — assert the app's own state, and take a full-screen capture
  when input silently does nothing.**

### A REAL bug found in passing — FOLLOW-UP, not this ticket

**A restored editor tab does not spawn the LSP.** #205 persists the editor tab into `workspace.shell`
(`V=0\x1F<path>`), so a relaunch RESTORES `main.rs` already-open — but #308's only spawn trigger is
`open_file_in_viewer`, which a restored tab never goes through. Observed directly: the editor showed
`main.rs` with **no `lsp:` segment at all** in the footer, and no server process. So reopening Marley
onto a `.rs` file gives you no diagnostics/hover/completions until you open some *other* file. Not
#313's code (it is #308/#205's seam) → **follow-up ticket #321**.

### Gate — GREEN

Re-run after both changes (the tie-break test + the contrast fix); see the receipt below.

status: Phase 4 — Validate PASS (coverage 100 at source, live drive done, 1 real finding fixed)

## Phase 5 — Complete
- (pending)
</content>
