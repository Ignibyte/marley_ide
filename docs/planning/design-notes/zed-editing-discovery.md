# Zed-level editing × Warp-level slickness — discovery (Round 2)

**Status:** discovery / design note (not a pipeline doc). Round 2 of the editor-discovery effort — deepens
[[editor-as-peer-and-terminal-fusion]] (Round 1, 2026-07-09) now that its **Tier 0 shipped as the M15 train**.
**Created:** 2026-07-11. **Owner intent (chad):** "bring Zed-level editing with Warp-level slickness" — check
Zed out, run several discovery rounds, and rewrite the parts in our own code. Inspired-by, not copied.

## Clean-room provenance (§20) — the wall we hold (same as Warp)

chad (2026-07-11): *"We did the same thing for warp — a check-out, several rounds of discovery, and we are
rewriting the parts without stealing the code. This is legal and doable. Inspired by their work, not stealing
line by line — like rock music, we have similar styles."*

Concretely, for this whole discovery track:
- **Zed material = BEHAVIOR + PUBLISHED WRITING only** — what a user sees and does, plus Zed's public
  `zed.dev/blog` + `zed.dev/docs` + conference talks. **We do NOT read or translate the Zed GPL source** (a
  reworded port is still a derivative work). Every capability below is described as *observed behavior to
  reimplement from scratch on our own stack*, never as "how Zed's code does it."
- **gpui is ours to use directly** — it is Marley's own Apache-2.0 dependency (already vetted through
  cargo-deny), not Zed's copyleft product. Its public API is fair game.
- **tree-sitter / LSP / ropey / permissive crates** — standard, permissively-licensed building blocks; ours to
  depend on directly.
- The `enforce-warp-reference.sh` commit-gate (#248) already forces every editor spec to name its reference +
  hold this wall. This doc is that reference for the editor track.

## Where Marley's editor is TODAY (post-M15) — the honest baseline

The on-screen editor is a **real editor** as of the M15 `/work 249-258` train — but a *single-cursor,
plain-text* one. What exists:

| Capability | Marley today | Where |
|---|---|---|
| Editable text model | `marley_editor::Buffer` — ropey rope, range `edit(range, replacement, EditOrigin)`, versioned, char-indexed | `crates/editor` |
| Render from the buffer | exact offset↔column map + caret bar; tab-stops; un-truncated | #250 `code_view::line_layout` |
| Text input | a `focused_editor` key route (Enter→`\n`), hand-rolled (NOT gpui `EntityInputHandler` yet) | #251 `input::apply_editor_key` |
| Save + dirty | ⌘S → `fs::write`, per-file dirty ● | #252 |
| Undo/redo | coalesced ⌘Z/⌘⇧Z, a two-`Vec` `EditRecord` stack | #253 `undo::UndoHistory` |
| Mouse caret | click→CharOffset (inverts the #250 map) | #254 |
| Selection | ONE selection (anchor+head), shift+arrows + drag + highlight + type-over | #255 |
| Clipboard | ⌘C/⌘X/⌘V over the selection | #256 |
| Motion | Home/End, ⌥←→ word, ⌘←→ line, ⌘↑↓ doc, Up/Down, all shift-extend | #257 |
| Multi-file | an editor tab holds N files (a strip); split-file panes persist | #237/#243/#258 |
| Syntax color | a hand lexer (`code_syntax`) — regex/heuristic, not tree-sitter | pre-M15 |

**The honest gaps vs. "a real code editor":** single cursor only; no tree-sitter (so no structural highlight /
indent / brackets / folds / outline); no LSP (no completions / diagnostics / go-to-def / hover / rename); no
project-wide find/replace or symbol search; no anchors (stable logical positions that survive edits — the
prerequisite for async LSP + multi-cursor + collab); a hand-rolled input/render path where gpui offers a
faster, IME-correct one; `Buffer` is a simple rope (fine now; a SumTree upgrade is the scale path).

## The Zed editing capability map (behavior → Marley reimplementation)

Each row is *observed Zed editing behavior* + the gap vs. post-M15 Marley + the reimplementation approach on
**our** stack (gpui + marley_editor + permissive crates). Grouped by the natural build order.

### Group 1 — the text substrate (the prerequisite for everything above plain-text)

| Zed behavior (observed) | Marley today | Reimplement-on-our-stack |
|---|---|---|
| **Anchors** — you can hold a logical position (a bookmark, a diagnostic, a cursor) that stays correct as text is inserted/deleted elsewhere; async results (an LSP reply that arrives 200ms later) still land in the right place. | none — offsets are raw `CharOffset`s that go stale on any edit | add an anchor layer to `marley_editor` (a position that resolves against a buffer version; bias left/right). The single must-have unlock for LSP + multi-cursor + marks. |
| **UTF-16 coordinate bridge** — the buffer speaks the LSP wire coordinate (UTF-16) without corrupting the char/grapheme model the UI uses. | char-indexed only | a coordinate adapter (`char ↔ utf-16 ↔ byte`) beside `marley_text_offsets`; only needed once LSP lands. |
| **Fast at scale** — large files stay smooth (structural sharing, windowed render). | ropey is fine for now; render is one-div-per-line (un-virtualized) | adopt gpui `uniform_list` for windowed rows (the intake's Tier-0 "how" we skipped); a SumTree-backed buffer only if/when files get huge. |

### Group 2 — syntax intelligence (tree-sitter — ONE engine, many features)

Zed's structural feel comes from tree-sitter: a real parse tree, not regex. One dependency yields a family of
features. **tree-sitter is permissive (MIT) — ours to adopt directly.**

| Zed behavior (observed) | Marley today | Reimplement-on-our-stack |
|---|---|---|
| **Semantic highlight** — colors follow the grammar (a fn name vs a keyword vs a type), stable across edits, incremental. | a hand lexer (`code_syntax`) — approximate, per-file | a `tree-sitter` parse per open buffer, `highlights.scm` queries → spans → gpui `StyledText.with_highlights` (one shaped element/line). Incremental re-parse on edit; run off the main thread against a buffer snapshot. |
| **Smart indent / auto-indent** — Enter indents to the block; `}` dedents. | none (Enter inserts a bare `\n`) | tree-sitter `indents.scm` → the newline handler in `apply_editor_key`. |
| **Bracket match + auto-close** — matching bracket highlight, typing `(` inserts `)`. | none | tree-sitter node bounds → a bracket overlay; a small auto-pair table in the input path. |
| **Code folding** — collapse a fn/block by its structure. | none | tree-sitter `folds.scm` → fold ranges → a gutter affordance + the render skips folded rows. |
| **Outline / breadcrumbs** — a symbol tree of the file; jump to a symbol. | none | tree-sitter `outline` query → a palette-driven symbol jump (reuses the ⌘P finder overlay idiom). |
| **Selection expand/shrink** (⌥↑/⌥↓ — grow the selection to the enclosing node) | none | tree-sitter node ancestry from the caret → extend the #255 selection to the node bounds. |

### Group 3 — language intelligence (LSP — the "IDE" layer)

A separate semantic service (speaks UTF-16, async, per-language). **The `lsp-types` + async plumbing are
permissive.** This is the biggest greenfield lift and depends on **anchors** (Group 1) landing first.

| Zed behavior (observed) | Marley today | Reimplement-on-our-stack |
|---|---|---|
| **Completions** — as-you-type suggestions, docs, snippets. | none (the #200 ghost-text is shell-history, not code) | an LSP client adapter; results anchored; rendered via the caret-anchored popup idiom (gpui `anchored()`), reusing the completion-overlay pattern the shell already has. |
| **Diagnostics** — squiggles + a gutter mark + hover detail. | none | LSP publishDiagnostics → anchored ranges → an underline overlay + a gutter icon + the scrollbar markers (#198's scrollbar can host them). |
| **Go-to-definition / references** | none | an LSP request → open the target at line:col (this REUSES the shipped #196 `file:line:col` → editor mechanism — the wedge already built). |
| **Hover** — type/docs on hover. | none | LSP hover → a `deferred()` popup at the mouse. |
| **Rename symbol / code actions / format** | none | LSP workspace-edit → apply as `Buffer::edit`s (the `EditOrigin::Agent`/tool path is the ready seam); format-on-save hooks the #252 save. |

### Group 4 — multi-cursor & structural editing (the "power editing" feel)

| Zed behavior (observed) | Marley today | Reimplement-on-our-stack |
|---|---|---|
| **Multiple cursors** — add a cursor (⌘-click / ⌥-drag), type into all, columnar edits. | ONE selection only (#255) | upgrade `EditorSurface`'s single `(caret, anchor)` to a `SelectionSet` (the `marley_editor::SelectionSet` type already exists, single-member today); the render loops selections; edits apply to each (needs anchors so they stay coherent). |
| **Select next occurrence** (Zed ⌘D) → add a cursor at the next match. | ⌘D is Marley's #197 **new-terminal** | ⚠️ **keybinding-context collision** — resolve via gpui `KeyContext` (editor-scope ⌘D = select-next-match; terminal-scope ⌘D = new-terminal). See Group 6. |
| **Column / block selection** (⌥-drag a rectangle) | none | a block-select mode over the #254 click→col map + the SelectionSet. |
| **Move / duplicate / join lines** (⌥↑↓, ⌘⇧D, ⌘J) | none | pure line ops on the Buffer (mirrors the #253/#257 pure-fn pattern; easy wins). |

### Group 5 — navigation & search (project-scale)

| Zed behavior (observed) | Marley today | Reimplement-on-our-stack |
|---|---|---|
| **Command palette — everything** — every action reachable + fuzzy-searchable, with keybindings shown. | Marley HAS a palette (#19/#25) + the ⌘P file finder (#97) + ⌘R history | extend the existing palette with the editor actions; the infra is there. This is where Marley is ALREADY Warp-slick. |
| **Project-wide find/replace** — search all files, edit results in place. | in-file find only (#198-era) | a search index over the project + a **multibuffer** result view (Group 7). |
| **Symbol search** (⌘T — project symbols) | none | tree-sitter outline across files (Group 2) → the finder overlay. |
| **Go-to-line / go-to-column** | motion exists; no explicit jump | a `:line` palette verb → `Buffer::line_start` (trivial). |

### Group 6 — keybinding CONTEXTS (the terminal↔editor discipline)

Zed resolves keybindings by *context* (an editor pane vs a terminal vs a modal). Marley now has BOTH a terminal
and an editor sharing one keymap — the ⌘D collision (#197 new-terminal vs Zed's select-next-match) is the first
symptom, and more will come (⌘F, ⌘/, Enter, Esc all mean different things in each surface).

- **Reimplement:** adopt gpui's `KeyContext` predicate model — the keymap dispatches by the focused surface's
  context (`editor` vs `terminal` vs `palette`). This is the clean fix for every present + future collision and
  the right foundation before multi-cursor/LSP add dozens of editor-only chords. **High-leverage, do early.**

### Group 7 — multibuffer (Zed's signature payoff — the last-tier differentiator)

Zed's standout: search results, references, and diagnostics open as **editable excerpts from many files in one
buffer** — fix all call-sites in one view, save, done. It sits on anchors + LSP + project search.

- **Marley today:** none. **Reimplement:** a `MultiBuffer` composed of anchored excerpt-ranges over N `Buffer`s,
  rendered as one `uniform_list`; edits route back to each source Buffer. The capstone — schedule LAST.

## The Warp-slickness axis — what makes it *feel* like Marley, not "a Zed clone"

"Zed-level editing" is the capability. "Warp-level slickness" is the FEEL + the terminal-first identity. This is
the half that keeps Marley itself (and where we're already strong):

- **Felt speed is inherited** — the GPU-rendered ~120fps / low-input-latency story IS the gpui premise; Zed's
  expensive half is already ours. The discipline: keep input→paint off the main-thread critical path;
  incrementalize highlighting (cache, don't re-highlight per frame).
- **The command-block terminal** — Marley's Warp-parity blocks (M12) are a thing Zed's plain terminal isn't.
- **Palette-everything + the finder overlays** — already Warp-slick; the editor actions plug into it.
- **The cockpit + workspace shell** — the docks, panes, the left-rail workspace, the theming (dark, #194/#231).

## The synthesis — the differentiator (from Round 1, sharpened)

Both Zed-study passes converged: Marley's wedge is the **terminal ↔ editor ↔ block fusion** — the thing neither
Warp (block-terminal, no editor) nor Zed (editor, plain terminal) has:
- **`file:line:col` in terminal output → open at that exact line** — SHIPPED (#196/#214). The front half is done.
- **run a task → jump to the failure line** — LSP diagnostics (Group 3) × the block-terminal.
- **tree-sitter `runnables` → a gutter "run" button → spawn as a terminal BLOCK** (status/output/rerun) — the
  fusion nobody else can do, because only Marley has both surfaces first-class.
- **`EditOrigin::Agent`** — the ready seam for agents to write files (composes with the embedded-browser +
  brain pillars → the full agentic workbench).

## Prioritized roadmap (M16+ — the frontier after M15's Tier 0)

Sequenced by dependency + leverage (each is a milestone-sized train, not a ticket):

1. **Keybinding contexts** (Group 6) — small, high-leverage, unblocks every editor-only chord. Do FIRST.
2. **gpui substrate adoption** (Group 1 render half) — `uniform_list` + `StyledText.with_highlights` +
   `EntityInputHandler`; a perf/correctness refactor that also virtualizes large files. Pairs with…
3. **tree-sitter** (Group 2) — ONE dependency → highlight + indent + brackets + folds + outline + expand-select.
   The biggest single felt-quality jump; replaces the hand lexer.
4. **Anchors** (Group 1) — the quiet prerequisite; land before LSP + multi-cursor.
5. **Multi-cursor + line ops** (Group 4) — on anchors + the SelectionSet; big power-user payoff, no external service.
6. **LSP** (Group 3) — the IDE layer; the greenfield lift; reuses the #196 goto mechanism.
7. **Project search + symbol search** (Group 5) → **multibuffer** (Group 7) — the capstone.

The **fusion wedge** (terminal↔editor↔block) threads through — each tier makes it sharper; #196 already shipped
its first step.

## Open questions (for the rounds ahead — chad steers)

- **Depth of "Zed-level"** — full LSP/multibuffer parity, or the 80% that a terminal-first audience actually
  wants (tree-sitter + multi-cursor + go-to-def, skip collab/some LSP surface)? This sizes M16+.
- **Vim mode** — plausibly HIGH priority for a terminal audience; where in the sequence?
- **Observation rounds** — Round 3 should be hands-on: run Zed, capture the *feel* of specific interactions
  (multi-cursor, the multibuffer, expand-selection, the palette) to `docs/warp_architecture/observed/` (or a new
  `docs/zed_reference/observed/`) — behavior captures only, per §20. Which interactions matter most to nail?
- **Substrate refactor timing** — adopt the gpui `uniform_list`/`StyledText` substrate BEFORE tree-sitter (so
  highlight rides the new path), or ship tree-sitter on the current render first?

## Composes with / promotion

Deepens [[editor-as-peer-and-terminal-fusion]]; composes with [[embedded-agent-browser-chromium-cdp]] +
[[brain-agent-session-supervision]] (the agentic workbench). This is a design note, not a pipeline doc — each
roadmap tier promotes via `/work` into its own milestone train when scheduled. Record the substrate + tree-sitter
+ LSP choices as forge ADs at promotion.
