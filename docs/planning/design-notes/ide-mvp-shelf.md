# The IDE-MVP shelf — 10 pre-authored specs (2026-07-17)

The remaining editor/IDE feature backlog, spec'd in batch onto `docs/planning/pipeline/queued/` (run 6+ of
the pre-authored-spec method — see [m22-editing-bar.md](m22-editing-bar.md) for the method and its 5-for-5
record of catching authorship errors at promotion). Authored on `main` @ `7fb27e2` with four parallel
seam-verification sweeps against live code; every cited fact carries a file:symbol reference from that
read. **Promotion still re-verifies — the confident sentences are still the dangerous ones.**

## The shelf

| Spec | Ticket | One line |
|---|---|---|
| `300-move-dup-lines` | #300 | ⌥↑↓ move / ⇧⌥↑↓ duplicate — cursors CARRIED (rebase clamps; the prior plan's finding) |
| `302-goto-line` | #302 | ⌃G overlay — parse + clamp + centered jump; the smallest ticket here |
| `303-delete-ops` | #303 | ⌥⌫/⌥⌦/⌘⌫/⌃K/⌘⇧K — and the key-translation seam that eats modifiers today |
| `304-file-symbols` | #304 | ⌘⇧O — a thin adapter over tree-sitter-rust's OWN `TAGS_QUERY` |
| `305-code-folding` | #305 | THE first visible↔buffer row projection (17-site blast radius, converted ONCE) |
| `314-lsp-formatting` | #314 | ⌥⇧F + format-on-save — the apply engine already shipped (#322); save never blocked |
| `317-find-references` | #317 | ⇧F12 — a NEW grouped row model (DefPicker is definition-shaped) |
| `315-multi-language-syntax` | #315 | The language axis + the 5→10 taxonomy growth (the unstated joint decision, now pinned) |
| `316-syntax-themes` | #316 | Per-theme SyntaxPalette, AA-contrast-proven by test; the cache worry DISSOLVED (kinds, not colors) |
| `259-editable-split-pane` | #259 | The architectural one — four single-slot assumptions named; the same-path fork pinned |

## Recommended order (dependency-aware; final call at /work time)

1. **#300 → #302 → #303** — the editing-muscle trio: independent, quick, highest felt-value per line.
2. **#304** — small (the tags query does the work); proves the symbols seam before LSP-heavy work.
3. **#305** — folding; the projection is the batch's second-biggest lift and B-c's display-map precursor.
4. **#314 → #317** — the LSP pair (both extend the fake_ls test lane; do them adjacent).
5. **#315 → #316** — HARD-ordered (the taxonomy must exist before it is themed). #315 unblocks #346.
6. **#259** — last: architectural, touches everything, plausibly splits into 2 slices at promotion.

## Cross-cutting facts the specs share (verified 2026-07-17)

- **No cached tree app-side** — #304/#305 pay the (nonce,version)-memoized throwaway parse like
  #330/#340; **#349** is the shared fix, not any one ticket's problem.
- **anchor.rs ships with ZERO production consumers** — #305 (folds) and #314 (caret carry) are its first;
  expect first-consumer bugs to be ANCHOR bugs, fixed at source there.
- **The #303 translation collapse** (app.rs:2465 — backspace early-returns before the modifier check) is
  why ⌥⌫ 1-char-deletes today and forward-delete is a no-op; #303 fixes the seam, not just chords.
- **Chords verified free** (recon): ⌥↑↓, ⇧⌥↑↓, ⌃G, ⌥⌫/⌥⌦/⌘⌫/⌃K/⌘⇧K, ⌥⌘[/], ⌥⇧F, ⇧F12. ⌘⇧O is
  GLOBAL open-remote → #304 shadows it Editor-scoped (the #325 ⌘T precedent).
- **A pre-existing bug the recon exposed, filed separately:** selection-ladder (#329) and sticky-headers
  (#330) build a throwaway RUST HighlightSession with NO language gate — the #340 M1 class, live today on
  any non-Rust file. (Bracket-match gates; they don't.)
- Closed as superseded while shelving: **#301** (all four items shipped via #338/#340; the sliver is #346).

## Standing constraints

Push discipline and live-drive rules per session state; the phase-gate hooks are inert on `queued/` —
promotion (`/pipeline:plan`) mints pipeline_id + AAR (except #300, which REUSES its prior plan's AAR
`422cffde…`), creates the local ticket doc, and re-verifies every cited seam. Batch lessons:
[m22-editing-bar.md](m22-editing-bar.md).
