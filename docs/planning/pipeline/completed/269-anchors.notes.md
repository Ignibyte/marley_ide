# 269-anchors — Notes

- **Forge ticket:** #269 21696db9-3828-40ec-8d10-b7985d8cb8a7
- **AAR:** a7bc5872-7fde-4970-8382-ca79c045b374
- **Local ticket doc:** docs/planning/tickets/open/TICKET-269-anchors.md
- **Pipeline spec:** 269-anchors.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 7 of 10 (taken before the B2/B3 trio —
  independent, pure-core, bounded).
- **Discovery:** types.rs has BufferVersion (monotonic u64) + BufferDelta
  {char_range, byte_range, new_char_len, new_byte_len} built per-edit in
  apply_edit (buffer.rs:133-165, the shared non-recording core that
  edit/undo/redo all traverse — the perfect single tap point). NO log is
  stored today ("existing BufferDelta log" in the ticket = the delta TYPE
  exists; the log is this slice's addition, and edits_since doubles as the
  B3 prerequisite).
- **Reference:** subsystem-02 read; the provenance section's plan
  ("delta-log-first, CRDT-deferred") is exactly this ticket; GPL boundary
  explicit (concept public, rendition GPL — no SumTree/Locator/clock).
- **Decisions:** D1-D4 in the spec (public bias convention; the one shared
  tap point; post-edit version per log row, apply iff >; borrowed
  edits_since).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Architecture / approach (§20 re-confirmed N/A — Marley-original)
New pure module `crates/editor/src/anchor.rs` + a minimal Buffer tap. No
gpui, no IO, no new shared cross-crate types (Anchor stays editor-owned;
CharOffset/BufferVersion are the existing vocabulary) — §14 clean.

**Types (anchor.rs):**
```rust
pub enum Bias { Left, Right }              // Copy+Eq+Debug+Hash
pub struct Anchor { pub version: BufferVersion, pub offset: CharOffset, pub bias: Bias }
```

**The single-delta rebase (pure fn, the arithmetic table):**
`rebase_offset(o, bias, d: &BufferDelta) -> CharOffset`, with
s = d.char_range.start, e = d.char_range.end, n = d.new_char_len:
| case | Left | Right |
|---|---|---|
| o < s (strictly before the span) | o | o |
| o > e (strictly after) | o − (e−s) + n | same |
| s < o ≤ e, s ≠ e or o ≠ s (covered / at span end) | s | s + n |
| o == s, s == e (pure insert AT o) | s (stay before) | s + n (jump after) |
| o == s, s < e (replacement starting AT o) | s | s + n |
(The last two rows collapse: for o == s both biases give s / s+n — the
uniform rule is: o ≤ s AND (o < s) → untouched; o == s → bias decides;
o in (s, e] → bias decides; o > e → shift. Implementation folds cases 3-5:
`if o < s { o } else if o > e { o - (e-s) + n } else { match bias { Left
if o == s => s, Left => s, Right => s + n } }` — wait, Left at o==s IS s =
o (stay) and Left covered also → s; so Left arm = s.min(o)=… simplified:
Left → min(o, s) = s when o ≥ s; Right → s + n. The design pins the TABLE;
implement writes the cleanest equivalent and the tests pin every row.)

**Buffer tap (buffer.rs):**
- field `deltas: Vec<(BufferVersion, BufferDelta)>` — the version stored is
  the POST-edit version (EditResult.version), pushed at the end of
  `apply_edit` (the one shared core → edit/undo/redo all logged).
- `pub fn edits_since(&self, since: BufferVersion) -> impl Iterator<Item = &BufferDelta>`
  — rows with `version > since`, in order (the B3 prerequisite).
- `pub fn anchor_at(&self, offset: CharOffset, bias: Bias) -> Anchor`
  (stamps `self.version`; offset clamped into [0, len] at creation).
- `pub fn resolve_anchor(&self, a: &Anchor) -> CharOffset` — fold
  `rebase_offset` over `edits_since(a.version)`, final clamp to len.
- ctors init the empty log.

### File manifest
| file | change |
|---|---|
| crates/editor/src/anchor.rs (NEW) | Bias, Anchor, rebase_offset + module tests |
| crates/editor/src/buffer.rs | deltas field + 2 ctor inits + the apply_edit push + edits_since/anchor_at/resolve_anchor + tests |
| crates/editor/src/lib.rs | `mod anchor;` + flat re-exports (Anchor, Bias) |

### Regression test plan (per REQ; mutant-aware)
| REQ | Tests |
|---|---|
| REQ-001 | anchor_at stamps version+clamps; same-version resolve = identity (0, mid, len, past-len→len) |
| REQ-002 | insert before (+n), delete before (−len), replace before (net); edit strictly after → unchanged (kills the +/− and boundary swaps) |
| REQ-003 | replacement covering: Left→s, Right→s+n (o mid-span, o==e; distinct s/e/n values so each term is pinned) |
| REQ-004 | pure insert AT the anchor: Left stays, Right jumps (o==s==e) |
| REQ-005 | a 5-edit mixed sequence: resolve-once == resolve-stepwise-with-fresh-anchors chain; exact final offsets |
| REQ-006 | edit → undo → redo: anchor tracks through all three (log grows by 3) |
| REQ-007 | edits_since: exact slice by version (0, mid, latest→empty); order preserved |
| REQ-008 | delete spanning EOF around the anchor → clamped; anchor past len at creation → clamped |
| REQ-009 | run `cargo mutants --list -f anchor.rs -f buffer.rs` at validate; kill-all (the boundary `<`/`>`/`<=` swaps need o==s, o==e, o==s==e cases — rows above provide each) |
- trybuild: none (no type-level contract worth a compile-fail here — Anchor
  fields are deliberately pub for B5's serialization needs).
- Uncoverable: none (pure).

### Risks / decisions
- R1 Fencepost (version stored = POST-edit): `>` vs `>=` in edits_since is
  THE classic slip — REQ-007's mid-version case pins it (an anchor at
  version v must NOT re-apply v's own delta).
- R2 Undo path: undo/redo go through apply_edit (confirmed at plan) — the
  log is complete; a future edit path bypassing apply_edit would silently
  skip logging (inspect checks for other rope mutations).
- R3 Log growth unbounded — documented v1 policy (spec Out).

## Phase 3 — Implement
- **Built to manifest:** anchor.rs (Bias, Anchor, rebase_offset — the
  3-branch implementation of the design table + 4 arithmetic tests
  compile-necessary for the module); buffer.rs — the `deltas` log field +
  2 ctor inits + the apply_raw push (step 5, post-edit version) +
  `edits_since` (filter `v > since`) + `anchor_at` (clamp at creation) +
  `resolve_anchor` (fold + final clamp); lib.rs flat re-exports.
- **Deviations:** none (the design's "implementation folds cases 3-5" note
  realized as the single covered-arm match).
- **Verification:** fmt; check green; `cargo nextest run -p marley_editor`
  → 71/71 (pre-existing 63 + the 4 new arithmetic tests + prop tests).

## Phase 3.5 — Inspect
- **Critic run:** 1, measurement-heavy: FULL cargo-mutants over both files
  (not just --list), a probe crate with 2,430 composition triples + a
  4,716-assertion char-identity oracle + the fencepost/undo/foreign-anchor
  probes.
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| F1 | MED | 5 surviving mutants on the new Buffer API (edits_since body/filter ×4 + resolve_anchor body→CharOffset::default — VIABLE, CharOffset derives Default); anchor_at has zero viable (no Anchor Default; .min unmutated) | REAL — the Phase-4 kill list | validate adds: resolve-across-edits (≠0, ≠original) kills 4; the MID-HISTORY anchor shape (edit→anchor→edit; must not replay the pre-anchor row) is the ONLY `>`→`>=` killer |
| F2 | — | the o==e covered convention: DEFENSIBLE AND UNIQUELY RIGHT — the only choice where edit(s..e,repl) ≡ delete-then-insert (the buffer's own primitive order); machine-checked over 2,430 triples + buffer-level equivalence; the divergent cell (Left@o==e, n>0) is pinned by the existing test | VERDICT recorded | rationale docstring ADDED to rebase_offset; B5 note: pick per-edge biases deliberately (inward = no-growth) |
| F3 | LOW | clamp idiom duplicated ×2 while clamp_char_offset is already imported | REAL | FIXED — both sites reuse it |
| F4 | LOW | foreign/future anchors resolve to clamped garbage silently; rebase_offset debug-panics on a hand-built inverted range | REAL (doc) | FIXED — caller contract on resolve_anchor + the valid-range word on rebase_offset |
| F5 | INFO | log completeness (apply_raw is the ONLY rope mutator; undo/redo logged — probe: 4 rows through edit/undo/redo/undo), fencepost (first row keyed v1; > exact), underflow-safe, final clamp unreachable for legit anchors (per-arm invariant) but kept as the deserialization net, collapsed-anchor-doesn't-uncollapse-on-undo is the documented non-CRDT limit | all verified clean | none |

- **Mutation ground truth (full runs):** anchor.rs 15/15 caught ALREADY
  (MSI 100 from the in-file tests); buffer.rs 39 caught / 5 missed (F1) /
  7 unviable (the #203/#204 Default-derive viability rule held exactly).
- **Post-fix verify:** fmt + 71/71.

## Phase 4 — Validate
- **Tests added (the inspect F1 kill list + the REQ rows):** 6 Buffer-level
  anchor tests — identity+creation-clamp; the mixed 3-edit sequence
  (insert-at/delete-before/edit-after — kills the edits_since body+filter
  family); the MID-HISTORY fencepost (edit→anchor→edit; + edits_since
  exact-slice asserts — the only `>`→`>=` killer); covered-replacement
  collapse; undo/redo tracking; trailing-deletion clamp. Plus the 4
  arithmetic tests in anchor.rs from implement.
- **RUN:** `cargo nextest run -p marley_editor` → **77/77**;
  `cargo nextest run --workspace` → **909/909 passed, 5 skipped**.
- **Driven UI capture: N/A** — pure library layer, no render/input surface
  (the app doesn't consume anchors yet; B5 wires them).
- **Gate:** first `--diff` RED on gate:2 — clippy `large_enum_variant` on
  `PaneContent` (the +24-byte delta log tipped `TerminalPane` to 280B vs
  56B second-largest). Fixed AT SOURCE per the lint: the Terminal variant
  is boxed (4 sites; accessor derefs; a doc comment records why). Re-run →
  **GATE GREEN [diff] 15/15** (MSI 100 — the 5 inspect-named mutants
  killed). Receipt written.
- **Pre-existing:** `block v0.1.6` note only.

## Phase 5 — Complete
- CHANGELOG entry; editor.md gains the anchor.rs bullet + the buffer
  delta-log note.
- AAR submitted. Captured: the o==e composition-invariance verdict is
  recorded in the rebase_offset docstring (the durable artifact); no new
  failure (the clippy red was fixed in-phase; the box is an improvement).
- TICKET-269 → closed/; forge #269 → done; docs → completed/.
