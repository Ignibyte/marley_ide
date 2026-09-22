---
pipeline_id: ab817ca9-6a84-45dd-8de1-cbf64762704d
ticket: forge#15 (cdfc58bf-88f3-4262-b6a7-eee6f5f6a2f3) · local docs/planning/tickets/open/TICKET-011-editor.md
aar_id: b8c1f784-05e1-44af-97e3-3e1018b9c0d8
sprint: M1.A — The Usable Terminal (aa46e22f) seq 4/5
status: Phase 5 — Complete PASS
title: marley_editor (M1.A subset) — rope-backed input-prompt buffer + EditOrigin seam
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-editor.spec.md
  - ../../../marley_architecture/crate-map.md
---

## Title

TICKET-011 — `marley_editor` (crate `crates/editor`), sprint M1.A seq 4/5: the **M1.A subset** of
[`SPEC-editor.spec.md`](../../../specs/SPEC-editor.spec.md) — the **rope-backed input-prompt buffer**.
A `ropey`-backed `Buffer` with the `CharOffset`/`ByteOffset` seam, range edits carrying an `EditOrigin`
(Human vs Agent), a single cursor/selection with movement, and submit-on-Enter read-back. **UI-agnostic**
(`visual_acceptance: N/A` — no gpui, no headed test; the themed render is `app_shell` #16). The full
editor (anchors, multi-cursor, grouped undo/redo, find, diff, layout) **grows in M1.B**.

## Scope

### In (M1.A — the input-prompt subset; spec R# in parens)
- **Buffer storage + lengths (R1):** text in a `ropey::Rope`; `len_bytes`/`len_chars`/`len_lines`.
- **The offset seam (R2):** `char_to_byte`/`byte_to_char` over `marley_text_offsets::{CharOffset,ByteOffset}`
  (seam-contracts §1 — NEVER redefine offsets); round-trip identity.
- **Edit + delta + provenance (R3, R4):** `edit(range, replacement, origin)` replaces exactly `range`,
  returns `EditResult{delta: BufferDelta(char_range/byte_range/new_char_len/new_byte_len), version}`,
  advances `BufferVersion` by 1 (R4). The edit carries `EditOrigin{Human,Agent}` **observably** (design
  picks the lean mechanism — see Decisions).
- **Single selection + clamp (R19):** a single cursor/selection (caret `head`, fixed `anchor` as
  `CharOffset`); `set_selection` clamps endpoints into `[0, len_chars]`.
- **Cursor movement (input layer):** move the caret/selection by **char / word / line** (left/right,
  word-left/right, line-home/end) — pure `CharOffset` + grapheme/word-boundary math, clamped at bounds.
- **point_at (R27):** `point_at(off)` → `Point{row(0-based), column(UTF-8 byte offset in the row)}`.
- **Submit-on-Enter:** the buffer exposes its current text (`text()` / `text_in_range`) so the input
  consumer can read the line on Enter and clear it.
- **The seam value types** the input binds to: `EditOrigin`, `BufferDelta`, `BufferVersion`, `Point`,
  the `Rope` alias (the `BufferEvent` subscription itself is M1.B — see Decisions).
- §21 — CHANGELOG + `docs/marley_architecture/editor.md`.

### Out / deferred to M1.B (note in spec — these are SPEC-editor R# NOT in M1.A)
- Stable `Anchor`s + `AnchorBias` (R5–R7); `BufferSnapshot` structural-sharing (R8).
- **Undo/redo** + Human/Agent coalescing + the 30-group bound (R9–R14) — the ticket's "grouped
  undo/redo grows in M1.B". (EditOrigin is CARRIED + observable in M1.A; its undo-grouping USE is M1.B.)
- Regex **find**/find_next + case-fold + wrap (R15–R17, R25, R26) — needs `regex-automata` (NOT a dep yet).
- **Multi-cursor** `SelectionSet::from_selections` merge (R18) — M1.A is single-selection.
- Soft-wrap **layout** pipeline (R20, R21); line **diff** (R22) — needs `imara-diff` (NOT a dep yet).
- The full `BufferEvent`/`BufferSubscription` (R23, R24) — M1.A's render reads via gpui reactivity.
- (Already SPEC-deferred) tree-sitter/syntax, rich-content, vim/readline key dispatch, pixel painting.

## Acceptance Criteria (EARS — M1.A subset of SPEC-editor)
- **AC1 (R1)** — WHEN a `Buffer` holds text, `len_bytes`/`len_chars`/`len_lines` shall match the rope.
- **AC2 (R2)** — for any valid `CharOffset`, `byte_to_char(char_to_byte(off)) == off` (and the byte dual).
- **AC3 (R3)** — WHEN `edit(range, replacement, origin)` is called, it shall replace exactly `range`,
  and the returned `BufferDelta` shall record `char_range == range`, the pre-edit `byte_range`,
  `new_char_len` (chars in replacement), and `new_byte_len` (bytes in replacement).
- **AC4 (R4)** — each applied `edit` shall advance `version` so `as_u64()` rises by exactly 1.
- **AC5 (EditOrigin)** — `edit` shall accept an `EditOrigin{Human,Agent}` and make it observable to the
  caller (the write-provenance seam — AD-claude-brain-agent-session-supervision-001).
- **AC6 (R19)** — WHEN the selection is set, endpoints shall be clamped into `[CharOffset(0), len_chars]`.
- **AC7 (movement)** — moving the caret by char/word/line shall land on the correct `CharOffset`
  (word = whitespace/word-class boundary; line = row home/end), clamped at buffer bounds.
- **AC8 (R27)** — `point_at(off)` shall return `row` = 0-based line of `off` and `column` = UTF-8 byte
  offset of `off` within that row.
- **AC-gate** — cov 100 / MSI 100 on the WHOLE pure surface (no ACCEPTED-UNTESTABLE expected; if a
  ropey re-export produces an unkillable mutant, record it explicitly); FULL `scripts/gates.sh` →
  `GATE GREEN [diff]`. **gate-15 N/A** (visual_acceptance N/A).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **UI-agnostic** — NO gpui, NO ui_components, NO headed/visual test. Deps: `ropey` (MIT, allowlisted,
   first use) + `marley_text_offsets`(path). dev: `mutants` (+ `proptest` if offset/rope invariants
   warrant). **NO `imara-diff`/`regex-automata`** in M1.A (deferred features → machete would flag them).
2. **Offsets** — import `marley_text_offsets::{CharOffset,ByteOffset}` (private-usize newtypes,
   `as_usize`/`from`/`add_signed`/`zero`); the crate declares NO local offset newtype (seam-contracts §1).
3. **The disjoint-index trap** — ALL offset arithmetic (char↔byte conversion, edit ranges, movement,
   selection clamp) is the mutation rigor center: NON-ORIGIN/multi-char/multi-LINE/multibyte fixtures
   (PR-claude-nonorigin-interior-fixture; PR-claude-use-plus-not-or-for-disjoint-bitfield-assembly if any
   bit math); assert EXACT carried EditOrigin + cursor/version values (the 004 trap).
4. **Clean-room** — Marley-original public identifiers (the spec's names: Buffer/EditOrigin/BufferDelta/
   BufferVersion/Point/EditResult/Selection); reuse the shared offset vocabulary; no Warp-internal names.

### Open design decisions (Phase 2 picks)
- **EditOrigin observability** without the full event subscription (R23 deferred): favor lean — either
  `EditResult{delta, version, origin}` (the caller gets it back) OR a stored `last_edit_origin()`. Design
  picks the one that keeps the seam REAL + the surface minimal.
- **Selection shape:** a single `Selection{anchor,head}` for M1.A vs the spec's `SelectionSet` container
  holding one member (to preserve `selection()->&SelectionSet`). Design picks; multi-cursor merge (R18)
  stays M1.B regardless.
- **Word-boundary movement:** hand-roll a simple word-class boundary over ropey chars (AVOID a new
  `unicode-segmentation` dep unless a spec clause truly needs full UAX#29).

## Phase Plan
- **P2 Design** — read SPEC-editor (the M1.A R#) + marley_text_offsets API + ropey's API (Rope ops:
  insert/remove/char↔byte/line_to_char/char_to_line); the module manifest (buffer/selection/movement/
  the seam value types); resolve the 3 open decisions; the mutation map (offset arithmetic, movement
  boundaries, clamp, version, delta lens — non-origin/multibyte fixtures). DELEGATE the read+manifest.
- **P3 Implement** — the Buffer + edit + selection + movement + the value types; ropey-backed.
- **P3.5 Inspect** — critics: the offset/range math (every conversion + edit + movement boundary), the
  EditOrigin seam realness, clean-room names, machete (only ropey+text_offsets).
- **P4 Validate** — the per-AC unit tests (non-origin/multibyte fixtures) + the Human-vs-Agent write
  integration test + (if warranted) proptest offset round-trip; FULL gate (gate-15 N/A).
- **P5 Complete** — §21; close #15; archive; → #16 app_shell (the FINALE — mounts terminal + input).
