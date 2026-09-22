# M22 — "The editing bar" (the batch plan)

**The goal (chad, 2026-07-16):** rival Zed/VS Code's **editing ability** — not their IDE breadth. The bar is
"enough that a developer chooses to write code in it." Languages beyond Rust are explicitly **post-MVP**
([roadmap → Would love to have](../../marley_architecture/roadmap.md#would-love-to-have--not-the-bar)).

**The method (new, this batch):** the tickets + design specs are **pre-authored in batch** (the Fable method)
instead of being written per-ticket at plan time. Each pre-authored spec lives in
`docs/planning/pipeline/queued/`; when a ticket starts, `/pipeline:plan` **promotes** it (mv → `active/`,
mint the `pipeline_id`, confirm the seams still hold against live code) rather than authoring from scratch.
Phase 2 (design) becomes *confirm the locked decisions*, not *derive them*. The one-active-pipeline rule is
untouched — `queued/` is inert to the hooks.

## The 16 tickets, execution-ordered

| # | Ticket | Title | Group | Size | Status |
|---|--------|-------|-------|------|--------|
| 1 | NEW | **Horizontal scroll** — the unreachable-tail defect | B-a | M | ✅ spec queued |
| 2 | NEW | **Appearance: font size/family + ⌘=/⌘−/⌘0 zoom** — retire the hardcoded 13.0 | B-a | S–M | ✅ spec queued |
| 3 | NEW | **Auto-close brackets + quotes** — pair/type-over/backspace-pair/wrap-selection | B-a | S | ✅ spec queued |
| 4 | #300 | **Move / duplicate line** — ⌥↑↓ / ⇧⌥↑↓ at N cursors | B-a | S | forge body exists; spec next pass |
| 5 | #303 | **Delete line / word** — ⌘⇧K / ⌥⌫ / ⌥⌦ / ⌘⌫ | B-a | S | forge body exists; spec next pass |
| 6 | #314 | **Format on save** — rustfmt via the LSP formatting request; edits through the #322 engine | B-a | M | forge body exists; spec next pass |
| 7 | #307 | **Multi-cursor Tab** — indent every cursor's span, not just primary | B-a | S | forge body exists; spec next pass |
| 8 | NEW | **Regex find/replace** — a `.*` mode on ⌘F; capture-group replace | B-a | S–M | ✅ spec queued |
| 9 | NEW | **Bracket-match highlight** — tree-sitter delimiter pairs + ⌘⇧\ jump | B-a | S | ✅ spec queued |
| 10 | #259 | **Editable split pane** — the read-only #246 pane gains a real editor (two views, one buffer) | B-b | M–L | forge body exists; spec next pass |
| 11 | #304 | **In-file outline** — `documentSymbol` → the finder modal; ⚠️ needs a NEW chord (⌘⇧O = open-remote) | B-b | M | forge body exists; spec next pass |
| 12 | #317 | **Find references** — the LSP references request → a jumpable list (the #327 shape) | B-b | M | forge body exists; spec next pass |
| 13 | NEW | **Display-map foundation** — generalize #331's phantom layer into a coordinate-transform stack | B-c | L | to spec (after B-a lands) |
| 14 | NEW | **Soft wrap** — one buffer line → N display rows (needs 13) | B-c | L | to spec |
| 15 | #305 | **Code folding** — the #329 node-range API's other consumer (needs 13) | B-c | M–L | forge body exists; spec after 13 |
| 16 | NEW | **Multibuffer** — B7's remainder; editable multi-file excerpts; unlocks #326 ph.2 + editable problems (needs 13–15) | B-c | XL | to spec |

**Ordering rationale:** 1 first (the only genuine *defect* — a long line's tail is unreachable today, not
merely unwrapped). 2–9 are small/pure muscle-memory wins on shipped infra. 10–12 widen the surface. 13–16 are
one strictly-ordered architectural chain — each needs the one before, and 13 must keep
`AD-claude-two-boundary-maps-for-phantom-text-001` (code spans hug the code; caret ranges track the caret).

## Facts verified against live code (2026-07-16, `main` @ `a9eb589`)

> **Read this section as CLAIMS, not facts — two of the seven below were DISPROVEN by the first two
> pipelines that touched them** (struck through, with what is actually true). Both were written from a
> grep, and both read as the most confident bullets here. That is the batch's central lesson: *a grep hit
> is not a seam*, and **the pre-authored spec's confident sentences are the dangerous ones — "X is the ONE
> Y" is a claim to verify, not a fact to build on.** Every promotion re-verifies against live code.

- **Every chord this batch needs is FREE**: ⌘⇧K, ⌥⌫, ⌘⇧\, ⌘=, ⌘−, ⌘0, ⌥↑, ⌥↓ — zero keymap.rs hits.
  (#337 CONFIRMED for ⌘=/⌘−/⌘0.)
- **`regex` + `regex-automata` are already in Cargo.lock** (via #326's `ignore` adoption) — the regex ticket
  adds **zero** new crates.
- **`find.rs` already has `replace_all`** (:106, back-to-front) — ticket 8 is regex *mode*, not replace-from-scratch.
- ~~**`TERMINAL_FONT_SIZE` (app.rs:783) is the ONE font-size const**, shared by terminal AND editor.~~
  **FALSE — #337 Phase 1.** It was one of SEVERAL sources: the terminal's command/output text came from
  `typography::type_scale` (a hardcoded scale), the pane container from the const, and the editor from its
  own path — they agreed only because #195 happened to set both to 13.0. The ticket's real work was
  *converging* the sources, not threading the one that existed. #337 deleted the const; `font_zoom.rs` +
  `type_scale(role, font_size)` are now the seam, and **`font_zoom::FONT_SIZE_DEFAULT` / `LINE_HEIGHT_RATIO`
  are the ONE home for those numbers** (the 13.0 guard had already gone stale twice).
- **The marks channel exists for bracket-match**: `styled_slices_with_marks(syntax, selections, marks:
  &[(Range, MarkTier)])` (code_view.rs:352) — a mark RETAINS a Plain+unselected slice, which is exactly what
  a bracket highlight needs (the #331 drop-guard lesson, inverted).
- ~~**3 `ScrollWheelEvent` handlers** exist (app.rs:12209/12531/13153) — h-scroll hooks the editor one.~~
  **FALSE — #336 Phase 1.** All three exist, but **none is the editor's** (its handler was deleted in #266).
  I counted grep hits and assumed one was the editor's; reading them showed otherwise. #336 created one.

### Pre-verified for the auto-close ticket (#338) — found while blocked on #337's gate

The three seams its spec cites are all REAL: `edit_at_selections_with` (buffer.rs:308),
`backspace_at_selections` (:333), and `auto_close` has zero repo-wide hits (the "no ticket exists" claim
holds). **But its central bullet — "every action applies through `edit_at_selections_with` (the #297
engine)" — is wrong for three of its four actions**, and the engine's own docs say why:

- **The engine places the caret AT THE END OF THE INSERT** ("every cursor ends exactly at the end of its own
  insert", buffer.rs:428-429). So `InsertPair` → `"()"` leaves the caret *after* the `)`, not between; `Wrap`
  leaves it after the closer, not around the inner text. **The engine does the TEXT half; the SELECTION half
  is #338's own work.**
- **That placement is load-bearing, not incidental** — buffer.rs:429-431 calls it "the precondition — and the
  ONLY one — under which `coalesces_into` may append a following keystroke char-wise." A caret fix-up that
  ignores this can silently break typed-run coalescing (⌘Z granularity), which no pair test would catch.
- **`TypeOver` is not an edit at all** — a pure caret move. Routed through the engine with an empty
  replacement it hits the no-op sweep (buffer.rs:397: nothing inserted + all carets → early return,
  records nothing), so it must not go through it.
- **The typed char arrives via the IME door**, not a key-down handler: `replace_text_in_range` (app.rs:11332,
  gpui's `EntityInputHandler`) — buffer.rs:421-422 calls `edit_ranges_restoring` "the app's ONE insert path:
  every typed character lands here." Design must say what pairing does **during an IME composition**
  (`clear_marked`/marked text is right next door at app.rs:11997) — a Japanese composition committing `(`
  must not pair mid-compose.

## Standing traps every M22 pipeline inherits (from the M16–M21 ledger)

`git add -N` a NEW file before the `--diff` gate · a new setting needs a NON-DEFAULT round-trip leg · app.rs
shims need per-fn `mutants::skip` (app.rs is coverage-excluded, NOT mutation-excluded) · run `cargo mutants
--list` on the ACTUAL code, never guess the set · no Zed/Warp in `crates/` comments (gate:14) · `cargo fmt
--all` before the gate · a defensive branch in a test helper is an uncovered cold arm — remove it · assert the
VALUES mutants corrupt (a display-only truth table lets column mutants live) · the mutation baseline is
load-flaky under `cargo test` threading (#334) — diagnose, never retry-until-green · an absence-grep must
prove the command ran (`| wc -l`; `timeout` does not exist on macOS).
