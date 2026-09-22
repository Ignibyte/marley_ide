---
pipeline_id: ab817ca9-6a84-45dd-8de1-cbf64762704d
aar_id: b8c1f784-05e1-44af-97e3-3e1018b9c0d8
---

# marley_editor (M1.A input subset) — pipeline notes

## Phase 1 — Plan (2026-06-28)

**Intent:** the M1.A input-prompt subset of SPEC-editor — a ropey-backed Buffer + the CharOffset/
ByteOffset seam + range edits carrying EditOrigin + single cursor/selection + movement + submit + point_at.
UI-agnostic (visual_acceptance N/A). Sprint M1.A seq 4/5.

**The cut call:** SPEC-editor is the FULL editor (R1–R27: buffer/anchors/selection/undo-redo/find/diff/
layout/events). The FORGE TICKET explicitly scopes M1.A to "the rope-backed INPUT prompt — Buffer +
cursor/selection + basic editing (insert/delete/move-by-char/word/line) + submit-on-Enter + the EditOrigin
seam; the full editor grows in M1.B." So M1.A IN = R1, R2, R3, R4, R19, R27 + movement + submit + the
observable EditOrigin; DEFER R5–R18, R20–R26 (anchors/snapshot/undo-redo/find/multi-cursor/layout/diff/
events) to M1.B. Reconciled spec vs ticket: SPEC WINS on naming (EditOrigin{Human,Agent}, the value-type
names) + visual N/A (NOT the ticket's stale "visual via harness"); the ticket's "ui_components dep" does
NOT apply (UI-agnostic core — no ui_components; render is app_shell #16).

**Deps confirmed:** ropey = MIT (allowlisted, first use in the lockfile) + marley_text_offsets (M0, ✅ —
CharOffset/ByteOffset: private-usize newtypes with as_usize/from/zero/add_signed/Add/Sub/Ord/serde +
CharCounter[forward byte→char]; the editor's Buffer owns the RANDOM-ACCESS char↔byte, seam-contracts §1.1).
NO imara-diff/regex-automata in M1.A (deferred → machete would flag). NO gpui/ui_components.

## Carry to Design (Phase 2)
1. **READ** docs/specs/SPEC-editor.spec.md (the M1.A R#: R1-R4, R19, R23-context, R27 + the Public surface
   lines 40-149) + crates/marley_text_offsets/src/lib.rs (the offset API — DONE, see above) + **ropey's
   actual API** (Rope::insert/remove/len_bytes/len_chars/len_lines/char_to_byte/byte_to_char/char_to_line/
   line_to_char/byte_to_line + char-iteration for movement; verify the 0.x signatures against the lockfile).
2. **Module manifest** (all PURE):
   - buffer.rs — `Buffer{rope: ropey::Rope, selection, version}` + new/from_str + len_* + char_to_byte/
     byte_to_char (random-access via ropey) + edit(range, replacement, origin)->EditResult + point_at +
     text_in_range/text + version. `BufferDelta`/`BufferVersion`/`EditResult`/`Point`/`Rope` alias here
     (or a types.rs). The edit computes the pre-edit byte_range + new_char_len/new_byte_len.
   - selection.rs — `Selection{anchor:CharOffset, head:CharOffset}` (+ new/anchor/head) + set_selection
     clamp. RESOLVE: single Selection vs a 1-member SelectionSet (Decision 2).
   - movement.rs — move by char/word/line over the buffer+selection (pure CharOffset math; word-class
     boundary hand-rolled over ropey chars — AVOID unicode-segmentation unless needed; Decision 3).
   - lib.rs — re-exports + #![deny(missing_docs)] + the UI-agnostic/pure-core doc.
3. **RESOLVE the 3 open decisions** (in the spec): EditOrigin observability (EditResult.origin vs
   last_edit_origin — favor lean+real), the selection shape, the word-boundary algorithm.
4. **Mutation map** — the rigor is ALL offset/range math: edit's char_range/byte_range/new_char_len/
   new_byte_len (off-by-one), char↔byte conversion, BufferVersion::next (+1, the 004 trap), movement
   boundaries (word/line — the comparison operators <,<=), selection clamp ([0,len] bounds), point_at
   (row line-index vs byte-column). NON-ORIGIN/multi-char/multi-LINE/MULTIBYTE (é, 😀) fixtures so the
   index mutants show; assert EXACT EditOrigin + version + offsets carried. marley_text_offsets is ALREADY
   100/100 — reuse its discipline (its add_signed_usize/floor_char_boundary patterns are the template).
5. **No shim expected** — all pure (ropey is a library, no OS). If a ropey re-export glue line is an
   unkillable mutant, record ACCEPTED-UNTESTABLE explicitly (wrapped-crate reason); else cov/MSI 100 on all.

**Phase 1 status:** PASS (autonomous-through-commit per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-06-29)

### Confirmed: ropey 1.6.1 API (load-bearing — verified from source)
`insert(char_idx: usize, &str)` + `remove(char_range: Range<usize>)` are **CHAR-indexed** (NOT byte —
the classic trap avoided). Converters: `char_to_byte`/`byte_to_char`/`char_to_line`/`line_to_char`/
`line_to_byte` (all accept one-past-end). `len_lines() = breaks + 1` → **empty rope = 1 line**. `Rope`/
`RopeSlice` impl `Display` (`.to_string()`). Pin `ropey = "1.6"`.

### The 3 decisions — RESOLVED
- **(a) EditOrigin → `EditResult{delta, version, origin}`** (add `origin`). The REAL seam: mirrors the
  deferred `BufferEvent::Edited{delta,origin,version}` field-for-field → zero reshaping when M1.B lands
  the event; per-edit + non-stale (vs a stored `last_edit_origin()`); additive superset of §3.2 (the
  `.delta`/`.version` consumer is unaffected) — sanctioned by the ticket's open-decision.
- **(b) Selection → a 1-member `SelectionSet` on the buffer** (`selection()->&SelectionSet`,
  `set_selection(SelectionSet)`). Preserves the seam-contracts §3.5 signature `app_shell` (#16, SAME
  milestone) binds to — single-Selection would force a same-milestone breaking change at M1.B. M1.A
  ships `SelectionSet::single` + `selections()`; `from_selections` (R18 sort+merge) DEFERRED.
- **(c) Word movement → hand-rolled `is_word_char(c) = c.is_alphanumeric() || c=='_'`** (no
  unicode-segmentation dep; `é`/digits/`_`=word, space/`\n`/`😀`=non-word). "skip-then-stop" semantics.
  Documented NOT-UAX#29 + char-movement is by scalar NOT grapheme — both M1.B upgrades.

### Deps
`ropey = "1.6"` (MIT, first use) + `marley_text_offsets`(path). dev: `proptest = "1"` (the R2 round-trip
property — spec-sanctioned; **DROPPABLE** if it adds a deny/audit-noisy transitive license, since the
fixture already covers R2 — confirm deny after adding). NO imara-diff/regex-automata/gpui/ui_components.

### Module manifest (crates/editor/src/, all PURE)
- **types.rs** — `EditOrigin{Human,Agent}`; `BufferVersion(u64 private)` initial/next[+1]/as_u64;
  `BufferDelta{char_range:Range<CharOffset>, byte_range:Range<ByteOffset>, new_char_len, new_byte_len}`;
  `EditResult{delta, version, origin}`; `Point{row:u32, column:u32}`; `pub type Rope = ropey::Rope`.
- **buffer.rs** — `Buffer{rope, selection:SelectionSet, version}` + new/from_str/len_bytes|chars|lines/
  version/text[to_string]/text_in_range[slice]/char_to_byte/byte_to_char/point_at/edit/selection/
  set_selection + `pub(crate) rope()`. `Default for Buffer`.
- **selection.rs** — `Selection{anchor,head: CharOffset private}` new/caret/anchor/head; `SelectionSet
  {selections:Vec<Selection>}` single/selections/`pub(crate) from_members`; `pub(crate)
  clamp_char_offset(off,len)=min(len)` ONLY (lower bound [0] is STRUCTURAL — NO redundant `.max(0)`,
  which would be an equivalent unkillable mutant).
- **movement.rs** — `is_word_char` + move_char_left/right, move_word_left/right, move_line_home/end
  (`&Buffer`+`CharOffset`->`CharOffset`, all clamped to [0,len_chars]).
- **lib.rs** — re-exports + `#![deny(missing_docs)]` + UI-agnostic/pure-core doc. CharOffset/ByteOffset
  NOT re-exported (one owner).

### edit() — the off-by-one center (4 steps, ORDER load-bearing)
1. capture PRE-edit `byte_start = char_to_byte(start)`, `byte_end = char_to_byte(end)` **before** mutating
   (post-edit would read mutated text → wrong byte_end). 2. `rope.remove(start..end)` then
   `rope.insert(start, replacement)` (remove-then-insert; char indices). 3. `version = version.next()`.
   4. `delta = {char_range: range, byte_range: byte_start..byte_end, new_char_len:
   replacement.chars().count(), new_byte_len: replacement.len()}`. `new_char_len ≠ new_byte_len` on a
   multibyte replacement (kills the .len()↔.chars().count() swap). point_at: `row=char_to_line(c)`,
   `column = char_to_byte(c) - line_to_byte(row)` (UTF-8 byte col).

### Fixture (non-origin / multi-line / MULTIBYTE) — drives every AC + mutation kill
`"abc\nzé😀w\ndef"` → len_chars 12, len_bytes 16, len_lines 3. Gold: char 6 (😀) = byte 7, row 1, col 3
(≠char-col 2 → kills point_at row→0 AND column→char). Word-right walk from 0 = [3,6,8,12]; word-left from
12 = [9,7,4,0]. edit(5..7,"X😀é") → byte_range 5..11, new_char_len 3, new_byte_len 7.

### Test/mutation plan
One test per AC1-AC8 (exact gold values) + `human_then_agent_writes_carry_origin` integration (Human,
Human, Agent edits → each EditResult.origin asserted + text "hi world" + version 3 + the §1.1
ByteOffset→CharOffset span fed to edit) + `prop_char_byte_round_trip` (R2, arbitrary Unicode). Mutation
rigor: version +1 (004 trap), edit char/byte ranges + the two len exprs (multibyte so 3≠7), char↔byte
(byte≠char at offset 6), clamp min (over-long→12, boundary 12→12, in-bounds unchanged, + independent
anchor/head over-long), movement word/line `<`vs`<=` boundaries (exact walk sequences), point_at row +
byte-col. NO ACCEPTED-UNTESTABLE expected (every ropey passthrough pinned by an exact nonzero/multibyte
gold; `pub(crate) rope()` + `Default` yield NO viable mutant — distinct from unkillable).

### Risks
ropey char-index convention CONFIRMED (low). len_lines=+1 (AC1 pins new()→1). word-class not UAX#29 +
char-not-grapheme + CRLF-naive (`\n`-only M1.A) — all documented M1.B upgrades. additive EditResult.origin
(authorized, reconcile-keep at M1.B). edit() does NOT shift selection (R5 anchors deferred — caller
re-sets caret; set_selection clamps). proptest droppable if deny-noisy.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-06-29)

Built `crates/editor` (package `marley_editor`): Cargo.toml + src/{lib,types,buffer,selection,movement}.rs.
check + clippy(-D warnings) + fmt + rustdoc(-D warnings) all clean; **no unsafe**; `cargo deny check
licenses` = ok. ropey 1.6.1 — NO API deviation (every method matched the design). proptest **STAYED**
(deny-clean: its tree is MIT/Apache, all allowlisted; unused until validate's R2 prop test).

**Deviations (justified):**
1. **`Buffer::from_str` → `Buffer::from_text`** — the spec's `from_str(&str)->Self` triggers clippy
   `should_implement_trait` (it expects `std::str::FromStr`, which is fallible). The subagent first added
   `#[allow(clippy::should_implement_trait)]`; I REMOVED it (the project's §0 no-suppressions ethos — even
   a justified inline allow is avoidable here) and RENAMED to `from_text` (infallible, Marley-original,
   clean-room-OK, lint-free, NO suppression). validate's tests + app_shell #16 use `from_text`.
2. **5 doc-comment "unsafe" rewordings** (e.g. "`unsafe`-free" → "safe-Rust") — proactively avoiding the
   literal-"unsafe" SAST gate:13 trap (learned from #14's Cargo.toml hit). The crate has zero unsafe code.
3. `Buffer` is underived (hand-written `Default` per spec); `EditResult{delta,version,origin}` (decision a);
   1-member `SelectionSet` (decision b); hand-rolled `is_word_char` (decision c); `clamp_char_offset` =
   `min` only. `from_selections` NOT added (M1.B). CharOffset/ByteOffset NOT re-exported.

**Verified:** the design's word-walks check out (move_word_right from 0 over the fixture → 3,6,8,12;
move_word_left from 12 → 9,7,4,0). No #[allow]/#[expect] anywhere (gate:12 clean).

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2 critics (offset/edit math; movement/selection/deps)

**Verdict:** ZERO correctness bugs in the offset/edit math (every fixture gold re-derived + confirmed);
all 21 viable buffer/types mutants killable → MSI 100 achievable, NO ACCEPTED-UNTESTABLE needed. ONE
substantive finding (fixed); the rest are VALIDATE OBLIGATIONS (exact golds + cross-module kills).

| Sev | Finding | Verdict / fix |
|---|---|---|
| **MEDIUM** | **dead defensive clamp + doc overpromise** — move_word_* led with `off.as_usize().min(len)`, IDENTITY on every in-range (contract-valid) input → the `.min` is an EQUIVALENT/unkillable mutant; and the module doc promised "[0,len_chars]" while move_char_left returned out-of-range + move_line_* PANIC on out-of-range. | **FIXED** at source: removed the dead `.min(len)` (move_word_* now assume in-range like the 4 siblings), softened the doc to the in-range caller-contract (consistent with edit's §14). clippy/rustdoc/fmt re-verified clean. Forge BF-…-dead-defensive-clamp-…-001 + PR-claude-no-dead-defensive-clamp-untested-001. |

**Verified GREEN (rejected as non-issues):** move_word/line panic-guards correct (`i<len` before char(i); `i>0` before char(i-1)); exact walks right[3,6,8,12]/left[9,7,4,0]; clamp is `.min` ONLY (no `.max(0)`); set_selection clamps BOTH anchor+head; EditOrigin seam REAL (origin is the param→EditResult.origin, not hardcoded); clean-room (grep warp empty; from_text rename sound — infallible ctor, not std FromStr, no #[allow]); deny ok; machete green (plain `cargo machete` — what the gate runs — does NOT flag the unused proptest dev-dep; --with-metadata would, but the gate doesn't use it).

### CARRY TO VALIDATE — kill-map obligations (critics verified; USE THESE EXACT GOLDS)
Fixture `F = "abc\nzé😀w\ndef"` (len_chars 12 / len_bytes 16 / len_lines 3; char 6=byte 7,row 1,col 3).
- **edit() is a cargo-mutants BLIND SPOT** (its only mutant is the UNVIABLE whole-body Default; there is
  NO `.len()`↔`.chars().count()` swap / reorder / stmt-delete operator). So the exact-DELTA assertion is
  the SOLE protection — AC3 MUST assert the FULL delta: `char_range==5..7`, `byte_range==ByteOffset(5)..
  ByteOffset(11)`, `new_char_len==3`, `new_byte_len==7` (3≠7 catches the swap by value) + post `text()==
  "abc\nzX😀éw\ndef"` + `version().as_u64()==1`. Plus edges: edit(8..8,"!")→byte 12..12 lens 1/1;
  edit(0..3,"")→byte 0..3 lens 0/0.
- **`rope()` IS a viable mutant** (ropey::Rope DOES impl Default → `Box::leak(Default)` empty rope) killed
  ONLY CROSS-MODULE by a movement test over NON-EMPTY content: `move_word_right(&from_text(F),0)==3`
  (mutant empty rope → char(0) panics). Validate MUST keep a movement-over-content test.
- **Exact-value golds (else the mutant survives a weak assert):** `version().as_u64()==3` after 3 edits
  (kills as_u64→1); `next` exact `==1` after one edit (kills +→* identity-at-0 and +→-); `len_lines()==3`
  on F (kills →1); `char_to_byte(6)==ByteOffset(7)` + `byte_to_char(7)==CharOffset(6)` (multibyte, kills
  →0); `point_at(6)==Point{row:1,column:3}` at row>0 (kills -→+ [→11] and -→/ [→1]).
- **Line-cov-only (NO viable mutant → mutation won't force the line):** explicitly call `Buffer::new()`
  (e.g. `new().len_lines()==1`), `Buffer::default()` (== empty), and read `version()`.
- **clamp/selection:** AC6 needs BOTH over-long-anchor-only (`new(99,5)`→`(12,5)`) AND over-long-head-only
  (`new(2,99)`→`(2,12)`) to kill a "clamp only one endpoint" mutant; boundary `new(12,12)`→`(12,12)`.
- **movement:** the exact walks (in-range starts only — out-of-range is now contract-violation); char
  boundaries right(12)→12, left(0)→0; line home(6)→4/end(6)→8, home(10)→9/end(10)→12; underscore-as-word.
- **proptest** dev-dep: validate's `prop_char_byte_round_trip` (R2) MUST use it (else it's dead weight).

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-06-29)

**Tests (25):** in-crate `#[cfg(test)]` in buffer.rs (AC1-AC5, AC8, text/text_in_range, new/default/
version line-cov), selection.rs (AC6 clamp both-endpoints + boundary + in-bounds, caret, single,
clamp_char_offset), movement.rs (char/word/line over the NON-EMPTY fixture — kills the cross-module
rope() mutant) + `tests/integration.rs` (human_then_agent public-API: Human/Human/Agent edits → origins +
text "hi world" + version 3; the §1.1 ByteOffset→CharOffset→edit currency) + `tests/prop.rs`
(prop_char_byte_round_trip — uses the proptest dev-dep). types.rs covered via Buffer.

**Results:** `cargo nextest -p marley_editor` = **25 passed**; `cargo llvm-cov` = **100%** every file
(buffer 196/196, movement 83/83, selection 69/69, types 9/9 = 357/357; lib.rs = re-exports, no code);
`cargo mutants` = **86 mutants → 72 caught + 4 timeout + 10 unviable + 0 MISSED → MSI 100%** (76 viable,
all killed). Confirmed: the rope() viable Default mutant CAUGHT by a movement test; the edit() blind spot
(only mutant unviable, no .len()↔.chars().count() operator) guarded by the exact multibyte delta assert;
the removed move_word_* clamp left NO equivalent (the 4 timeouts are identity-loop hangs, caught). fmt +
clippy clean. **No production bug; production logic untouched** (every mutant landed on original lines).

**FULL gate (011):** `GATE GREEN [diff]` (00:58:14) — **15 passed, 0 failed**. Coverage **100%**
(buffer/movement/selection/types); mutation **76 caught / 0 missed → MSI 100.0%**; gate-13 SAST clean
(the doc-comment "unsafe" rewordings proactively avoided the literal-"unsafe" trap from #14); gate-15
N/A; deny/machete green. Commit receipt written (41 b, MATCH).

**Phase 4 status:** PASS.
