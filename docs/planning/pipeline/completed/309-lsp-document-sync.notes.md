# LSP document sync + UTF-16 bridge — Notes

- **Forge ticket:** #309 5bc7247a-15e9-47f7-a2ce-826b5bfddb65
- **AAR:** 697adc7c-9c71-4992-9f42-c9097e72b37c
- **Local ticket doc:** docs/planning/tickets/open/TICKET-309-lsp-document-sync.md
- **Pipeline spec:** 309-lsp-document-sync.spec.md

## Phase 1 — Plan
- **Request:** /work 308-317 train, #309 (goal-hooked, auto-approved). Builds on #308's marley_lsp
  wire (SHIPPED `708f2043`).
- **Classification / tier:** feature, M20, one slice: the position bridge (heart) + doc-sync lifecycle.
- **Forge recall:** knowledge-context(Plan) logged 13 surfacings. Binding: the #250 offset↔column AD
  `902ea928` (the 2-D Point is BYTE-column — do NOT confuse with the LSP encoding column); the #308
  PRs (stage-new-crate-before-mutation; const/boundary hard-pins; no-private-doc-link). marley_lsp is
  now a TRACKED crate, so `--diff` mutation sees its edits (but a NEW file in it still needs staging).
- **Discovery (the seams):**
  - `editor::Buffer::line_col(CharOffset) -> (row, char_col)` — SHIPPED + tested; the offset→(line,col)
    half of the bridge. `char::len_utf16()` gives the UTF-16 half.
  - `Buffer::edits_since(version) -> Iterator<&BufferDelta>` + `version()` — the incremental source.
  - **`BufferDelta` (types.rs:45) carries `char_range`/`byte_range`/`new_char_len`/`new_byte_len` but
    NOT the inserted text** → incremental change events need `new_text` added to the delta (D3). The
    `edit()` fn (buffer.rs:247) already has the text.
  - `#308 lsp_host`: holds `negotiated` (the position encoding + — to capture — the sync kind);
    `send_body` is the outbound seam; `ensure_lsp_for_opened_file`/`open_file_in_viewer` (app.rs:2851)
    the didOpen trigger; `save_active`/`mark_saved` (4551/4626) the didSave; `close_tab_at` (3959) the
    didClose.
  - `lsp-types` re-adds here (deferred from #308) for Position/Range/change-event shapes.
- **Decisions:** D1-D5 in the spec (one pure encoding-aware bridge; incremental+full fallback on the
  negotiated sync kind; extend BufferDelta.new_text; per-doc sync state on the host; pure-seam/shim).

## Phase 2 — Design

### Architecture (§14; §20 confirmed)
**§20:** matches the spec — Zed's doc-sync + UTF-16-column behavior (server view in lockstep, positions
in the negotiated encoding), clean-room from the published LSP 3.17 sync spec. No Zed source read.

**DESIGN PIVOT (discovery-driven):** the #269 delta log `deltas: Vec<(BufferVersion, BufferDelta)>` is
UNBOUNDED and stores NO inserted text (`edit()` stores it only in the undo stack). So the planned
`BufferDelta.new_text` extension is REJECTED (memory regression + duplicates undo). → **full-text
didChange for v1** (spec-valid, rust-analyzer-accepted); incremental deferred. The editor crate is
untouched. **The position bridge is the full deliverable** (pure, the #310+ dependency).

**The bridge is pure over `&str`** (no `editor::Buffer` coupling — keeps marley_lsp the dumb transport):
- `marley_lsp/src/position.rs` (PURE, NEW): `PositionEncoding` MOVES here from handshake.rs (a position
  concept; handshake re-imports it). `Position{line:u32, character:u32}`. Functions:
  - `column_in_encoding(line_text, char_col, enc) -> u32` — sum of `len_utf16`/`len_utf8`/1 over the
    first `char_col` chars; clamp `char_col` > line len to the line's end column.
  - `char_col_from_column(line_text, column, enc) -> usize` — walk chars accumulating units to `column`
    (or a partial unit boundary → the char before); clamp.
  - `offset_to_position(text, char_offset, enc) -> Position` — count `\n` before offset for the line,
    `char_offset - line_start` for char_col, then `column_in_encoding` on that line's text.
  - `position_to_offset(text, line, character, enc) -> usize` (char offset) — the inverse; clamp
    past-EOF/past-EOL. (For #310+ inbound; #309 round-trips it.)
- `marley_lsp/src/docsync.rs` (PURE, NEW): the notification bodies + version logic:
  - `open_notification(uri, language_id, version, text) -> String` (didOpen JSON).
  - `change_notification_full(uri, version, text) -> String` (didChange, one `{text}` event).
  - `save_notification(uri, text?) -> String`; `close_notification(uri) -> String`.
  - `DocSync{ version: i32 }` with `bump() -> i32` (monotonic; didOpen=version 0, first change=1…).
  - `language_id_for(path) -> Option<&str>` (ext→LSP languageId; "rust" for .rs; unified with #315's
    table later — v1 just rust).
- `marley_lsp/src/lib.rs` — export the position + docsync surface.
- `marley_app/src/lsp_host.rs` (SHIM, extend): per-open-doc `HashMap<PathBuf, DocSync>` on the host;
  `did_open(path, text)` (once, before any change), `did_change(path, text)` (bump+send full text),
  `did_save(path)`, `did_close(path)`. Each builds the body via docsync + `send_body`. Reads the
  encoding from the host's `negotiated` (default utf-16).
- `marley_app/src/app.rs` (SHIM, extend): call `did_open` at the `open_file_in_viewer` spawn-trigger
  site (right after ensure_lsp, once the host exists + is a rust file under a Cargo root); the pump
  detects an open editable buffer whose `version()` advanced since last-synced → `did_change`; the
  save path (`mark_saved`) → `did_save`; `close_tab_at` for an editor tab → `did_close`.
- **fake_ls (extend):** a `record` mode — capture received didOpen/didChange/didSave/didClose to a
  file (or reply on request) so the integration test asserts the ORDER (didOpen before didChange) +
  that the last didChange text equals the buffer text.

### File manifest
- NEW `crates/marley_lsp/src/position.rs`, `crates/marley_lsp/src/docsync.rs`.
- MOD `crates/marley_lsp/src/lib.rs` (mods + re-exports), `handshake.rs` (import PositionEncoding from
  position), `src/bin/fake_ls.rs` (record mode).
- MOD `crates/marley_app/src/lsp_host.rs` (per-doc sync + did_* senders), `src/app.rs` (the 4 hook
  sites), `tests/` (marley_lsp integration for the sync order).
- MOD `CHANGELOG.md`.

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Kind |
|---|---|---|
| 001 | `column_in_encoding` UTF-16: emoji line (1 char→2 units), BMP-multibyte (1 unit), ascii | unit |
| 002 | `column_in_encoding` UTF-8 (bytes) + UTF-32 (chars) branches | unit |
| 003 | past-EOL char_col clamps to line-end column; empty line; no panic | unit |
| 004 | docsync: didOpen built once (version 0); a change before open is impossible (state guard); didClose drops | unit + fake_ls order |
| 005 | `change_notification_full` carries the whole text; `DocSync::bump` monotonic (0→1→2) | unit + integration |
| 006 | bridge round-trip offset→position→offset for ascii/emoji/multi-line/EOL/past-EOF | unit |
| 007 | didSave built + sent after write | fake_ls integration + driven |
| — | fake_ls integration: open a doc, 2 changes, save, close → recorded order is didOpen,didChange,didChange,didSave,didClose; last change text == final buffer text | integration |
| — | driven re-verify (real rust-analyzer): type into a .rs, server stays in lockstep (end-to-end provable at #310) | driven (P4) |

### Risks
- **R1** pump-cost: scanning open buffers' versions each 16ms tick is cheap (a version compare); only
  build+send text on an actual change. Verify no per-tick allocation on the idle path.
- **R2** the didOpen/didClose lifecycle must pair exactly (no didChange before didOpen; no leak on
  close) — the per-doc state guard; the fake_ls order test pins it.
- **R3** `char::len_utf16` for a non-BMP char = 2 (surrogate pair) — the emoji case; pinned.
- **R4** incremental deferral is a documented v1 scope (follow-up ticket for ranged didChange +
  the app-side change-capture queue).

## Phase 3 — Implement
Built the pure `marley_lsp` seams (`position.rs` — the encoding-aware bridge; `docsync.rs` — the
notification builders + `DocVersion`) + moved `PositionEncoding` from handshake to position + the
lib re-exports; the app-side `lsp_host` gained per-doc reconcile + `did_save` + `docs.clear()` on
connection loss, and app.rs gained the pump reconciliation (over the active workspace's editor
files) + the save hook. `cargo check --workspace` green; clippy `-D warnings` clean. Mutants:
position 37 + docsync 21 = 58 to kill in P4; lsp_host = 0 (all reconcile/did_save/absolute skipped).

**Deviations from design (with reason):**
- **Pump RECONCILIATION, not 4 hooks.** The design named open/change/save/close hooks; the async
  spawn (a file is open ticks before the server is Ready) makes hook-driven didOpen fire too early.
  The pump reconciles the active host's doc set to the open editor files each tick (didOpen when
  newly seen + Ready, didChange on a version advance, didClose on drop) — the Ready gate + the diff
  handle the timing and open/close for free. didSave stays a save-path hook (an on-disk event).
- **v1 syncs the ACTIVE file per editor tab**, not every file in a split surface (`EditorSurface`
  exposes `active_buffer()` but not a per-file (path, &Buffer) iterator). Splits/multi-file-per-tab
  sync is a follow-up. Noted.
- `position.rs` has NO #309 consumer (full-text didChange needs no position math) — it is built +
  tested here as the ticket's stated heart; its first reader is #310 (diagnostics ranges). Like
  #308's `Negotiated`, an intended-next-ticket seam, not dead code.
- **fake_ls "record" mode deferred to P4** (test infrastructure, per §3 — not written in Implement).

## Phase 3.5 — Inspect
**Lenses:** 3 parallel critics — correctness (position math), state/data-integrity, simplification/
provenance. Plus 1 issue I fixed while prepping (the offset↔position pair inlined the encoding
accumulation, duplicating column_in_encoding/char_col_from_column and leaving them uncalled →
refactored to delegate via a `line_text_from` byte-slice helper; the simplification critic
independently confirmed this resolved its "two granularities don't compose" note).

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| S1 | **MED** | reconcile syncs only the SURFACE'S ACTIVE FILE (all files funnel into one editor surface), so switching files churns didClose/didOpen + only one doc is ever open server-side + unsaved background edits invisible | **REAL** (state) | new `EditorSurface::open_docs()` yields (path, &Buffer) for EVERY open file; the pump feeds all — no churn, all docs sync |
| S2 | **MED** | the pump clones full-buffer `.text()` for every open file EVERY 16ms tick, even idle / no-Ready-host; reconcile only uses it on a version change | **REAL** (state + simplification) | new `LspHost::needs_text(path, version)` immutable gate; the pump materializes `.text()` only when true, else `String::new()` |
| S3 | **MED** | the 4 docsync builders hand-roll the `{jsonrpc,method,params}` envelope; `rpc::build_notification` exists (build_cancel/initialized/exit all use it) | **REAL** (simplification) | routed all 4 through `build_notification` — one tested envelope, no duplicated literals |
| S4 | LOW | no path canonicalization → a `/tmp` vs `/private/tmp` (macOS) absolute file-ref fails `starts_with(root)` (never syncs) or double-keys | **REAL** (state) | canonicalize `root` at `new()` + `full` in `absolute()` (fallback to joined for a not-yet-on-disk file) |
| CX | — | the position bridge (column/char_col/offset↔position, clamping, round-trip, panic-safety) | **CLEAN** (correctness — full trace on `"a😀\nbc"`; the '\n' belongs to the line before; `line_start_byte=byte_i+1` correct; no OOB) | — |
| PROV | — | §20 provenance (position math + didOpen/change JSON) | **CLEAN** — plain LSP-spec-shaped std code, no lsp-types, no Zed lsp_store structure | — |

**Rejected / no-fix:** `usize` vs `CharOffset` in position.rs (defensible — position.rs is pure over
&str, no editor dep; declares no competing newtype); the `editor::ime::utf16_ix_to_char_ix` twin
(cross-crate boundary + OPPOSITE rounding by design — informational parity note, not a bug); the
whole-buffer O(n) line walk (inherent to being editor-decoupled; cold path — #310 maps a position
once per result, not per tick).

**P4 mutation-survival notes (from the correctness critic — cargo-mutants won't self-catch):**
- `line_text_from` start `byte_i + 1` (position.rs) is LOAD-BEARING: a `+1`→`+0` mutant makes every
  non-first line read as `""` → assert `offset_to_position("ab\ncd", 4, Utf16) == {line:1, character:1}`
  (NOT `{1,0}`).
- Boundary comparisons (`i >= char_col`, `units >= column`, `cur_line == line`, `cur_line < line`)
  need exact-boundary asserts: char_col/column at 0 AND exactly at a char boundary; a multi-line
  round-trip; `character` past EOL must assert it lands ON the '\n' offset.

**Post-fix:** cargo check + clippy -D clean; lsp_host 0 mutants (all shim); position 38 + docsync 21
pure mutants for P4.

## Phase 4 — Validate
**Tests written (all RUN green):** `marley_lsp` 74 unit (position: per-encoding column, char_col
inverse + mid-multi-unit, offset↔position ROUND-TRIP on `"a😀\nbc"`, the critic's `byte_i+1`
non-first-line pin, past-EOL/EOF clamps, trailing-newline no-panic; docsync: DocVersion monotonic,
language_id table, the 4 notification shapes via parsed-Value compare) + `marley` 461 lib (incl. the
new `EditorSurface::open_docs` all-files test). Pure files position 38 + docsync 21 mutants.

**Gate (`scripts/gates.sh --diff`): GATE GREEN [diff] — 15/15.** coverage **100% lines**, mutation
**MSI 100%** (58/58 caught, 0 missed — first try, the critic's mutation pins held), miri + visual +
all static PASS. One red fixed mid-validate: gate:12 (no-suppressions) — editing the `negotiated`
`#[allow(dead_code)]` comment pushed it >100 cols so rustfmt moved it off the attribute line (losing
the same-line justification); shortened it back onto one line.

**Live smoke drive (self-test harness):** the #309 app-side change is the per-tick pump reconcile +
the save hook — no new render, so the observable is "the app stays responsive and syncs without
crashing" (the content-correctness proof is #310's diagnostics, per the ticket). Fresh boot →
tree-click `anchor.rs` (fresh open) → the reconcile spawned rust-analyzer + sent didOpen/didChange:
the footer reached **`lsp: ready`**, the editor rendered, and `marley` stayed ALIVE (no hang/crash
from the 16ms reconcile or the canonicalize IO). Capture: `scratchpad/marley-309-smoke.png`.

**Deferred (per ticket):** the fake_ls "record-order" integration — the ticket routes the true
end-to-end sync proof to #310 (a diagnostic under an emoji-bearing line landing at the right
column); #309's reconcile ordering is inspect-verified CLEAN + the smoke drive confirms live-safety.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG entry added; crate-map `marley_lsp` row updated (position bridge + doc sync
  + the +position/docsync shim seams).
- **Knowledge (forge):** `PR-claude-pump-materialize-hot-data-only-when-consumed-001` (the hot-path
  fix) recorded at inspect; AAR 697adc7c submitted (1 novel, effectiveness 4). No BF — all inspect
  findings were quality/perf caught pre-ship, no bug reached the gate.
- **Shipped:** the pure encoding-aware position bridge (`position.rs`) + doc-sync notification builders
  (`docsync.rs`) at cov/MSI 100, the app-side pump reconciliation (all open files, text gated on a
  version change, canonical keying) + save hook + `EditorSurface::open_docs`. Full-text didChange v1;
  incremental deferred (unbounded delta log, no inserted text). GATE GREEN [diff] 15/15; live-driven
  `lsp: ready` with the per-tick reconcile crash-free. LOCAL-ONLY (push un-OK'd).
- **Follow-ups (not blockers):** incremental (ranged) didChange (needs a bounded change-capture queue
  or snapshot-diff); multi-file-per-split sync (v1 syncs all files' active buffers via open_docs, which
  already covers the one-surface model); the fake_ls record-order integration (the true content proof
  is #310's diagnostics).
