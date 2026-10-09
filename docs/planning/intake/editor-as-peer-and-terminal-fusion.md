---
status: superseded
created: 2026-07-09
ticket: unassigned
pipeline_spec: unassigned
note: audit (2026-10-09): superseded by the Zed fork (three-prong-plan.md), whose editor supplies Tiers 2 and 3; the fusion wedge shipped as TICKET-619 to TICKET-623
---

# Editor as a first-class peer + terminal↔editor fusion — the "Warp × Zed" milestone (future)

## What
Make Marley's code surface a **real editor**, a first-class peer to the terminal (not the read-only
`CodeView` viewer it is today), and fuse the two into the thing neither Warp nor Zed has: a
**terminal-first tool where the editor and the block-terminal are one integrated surface.**

chad (2026-07-09): "we could have a Warp/Zed combo in this application that would knock the socks off
people." Grounded by a clean-room seeker sweep (4 agents; sources below).

## The strategic finding — this is FAR less work than "build Zed"
Three of the four hard parts already exist or come free; the genuinely-new work is wiring + tree-sitter + LSP.
1. **The edit engine already exists** — `crates/editor` (`marley_editor::Buffer`): ropey-backed range
   edits, versioning, `EditOrigin::{Human,Agent}` (the Agent variant is a ready agent-write hook,
   currently wired to nothing). It's TRAPPED driving the one-line terminal prompt (`input.rs::apply_key`);
   the code view never touches it.
2. **gpui ships the entire RENDERING half of an editor** (the Apache-2.0 dep Marley already vets) — and
   Marley currently hand-rolls all of it. Unused gpui capabilities that replace hand-rolled code:
   `EntityInputHandler`/`Window::handle_input`/`UTF16Selection` (real OS/IME text input + caret geometry,
   vs the two-div `split_at_caret`); `StyledText.with_highlights(HighlightStyle)` (feed `code_syntax`
   spans → one shaped element/line, vs N colored divs; also gives caret/selection geometry via
   `TextLayout::{index_for_position, position_for_index}`); `uniform_list()`/`UniformListScrollHandle`
   (windowed monospace rows — code, terminal grid, file tree, finder — vs one-div-per-line un-virtualized);
   `list()`/`ListState` (variable-height terminal blocks); `anchored()`/`deferred()` (caret-anchored
   completion/hover popups); gpui `ScrollHandle` + `overflow_y_scroll` (vs hand-rolled offset math at 4 sites).
   NB: gpui ships NO parser/rope/tree-sitter — language intelligence stays Marley's (`code_syntax` + Buffer + a future tree-sitter dep).
3. **The pane/tab peerage is already built** — `PaneContent::CodeView` (a grid pane) + `TabContent::CodeView`
   (a full-screen tab) are first-class beside terminals (`workspace.rs`, `tabs.rs`). Structurally the editor
   is already a peer; it's just read-only.
4. **The "feel fast" is inherited** — the GPU-rendered 120fps / ~2ms-input-latency story IS the gpui premise.
   Zed's expensive half is already Marley's. Remaining felt-speed work: keep the editor input→paint path off
   the main-thread critical path + incrementalize syntax highlighting (cache; don't re-highlight every frame).

## The differentiator (build toward this)
Both Zed-study agents independently converged on the **terminal↔editor↔block fusion** as Marley's wedge —
where terminal-first beats Zed rather than merely matches it:
- Clickable `file:line:col` in terminal output → open at that exact line (**this is the front half of the
  already-queued M12.2 #196 clickable-links ticket**).
- Run a task → jump to the failure line.
- tree-sitter `runnables`-style queries → gutter "run" buttons → **spawn as a terminal BLOCK** (status/output/rerun).
Zed has the editor + a plain terminal; Warp has the block-terminal + no editor. Only Marley has both first-class.

## Sequenced roadmap (grounded in the code)
- **Tier 0 — viewer → editor (mostly INTEGRATION):** bind `Buffer` into the CodeView tab/pane content; add an
  editor key-routing branch parallel to `focused_terminal_mut()` (`app.rs:3576-3590`) → `focused_editor_mut()`;
  extend `movement`/`apply_key` to multi-line (vertical motion, newline-on-Enter, shift-select); adopt
  `EntityInputHandler` for real input; render via `StyledText.with_highlights` + `uniform_list`; reuse
  `text_selection.rs` geometry for click/drag-select; add save + dirty state; add undo/redo (Buffer has a
  version counter, needs a history stack).
- **Tier 1 — the wedge (LOW effort / HIGH diff):** clickable `file:line:col` → editor (≈ #196); run→jump-to-failure.
- **Tier 2 — code editor (GREENFIELD, the real lift):** tree-sitter as ONE query engine (highlight/indent/
  outline/brackets/runnables; incremental; off-main-thread against a buffer snapshot) replacing the hand
  lexer; **anchors** (stable logical positions — the must-have for LSP/async/multi-cursor even without collab);
  LSP adapter (completions/diagnostics/go-to-def/hover; separate semantic layer; speaks UTF-16; runs against
  snapshots). Buffer scale-upgrade to a SumTree-backed model (UTF-16 + line indexing + anchors) when needed.
- **Tier 3 — power:** multibuffer (search/refs/diagnostics as editable excerpts — the signature payoff),
  multi-cursor, project-wide find/replace, symbol search, code actions, format-on-save, context-scoped
  keybindings (editor-scope vs terminal-scope — resolves the ⌘D collision: Zed's ⌘D = select-next-match vs
  Marley's #197 ⌘D = new-terminal; adopt gpui's `KeyContext` predicate model), maybe vim mode (plausibly
  higher priority for a terminal audience).

## Progress (2026-07-11 — Tier 0 SHIPPED as the M15 train)

**Tier 0 is DONE.** The M15 "Editable Editor" train (`/work 249-258`, sprint #28) executed this tier end-to-end —
the on-screen editor is no longer the read-only `CodeView` viewer; it types, saves, undoes, selects, copies, and
navigates from a real `marley_editor::Buffer`:
- #249 doc model (Buffer+caret+saved_version behind each open file) · #250 faithful renderer (draw from the
  Buffer, exact offset↔column map + caret) · #251 input intercept (`focused_editor` key route, Enter→`\n`) ·
  #252 save ⌘S + dirty ● · #253 undo/redo (coalesced ⌘Z/⌘⇧Z) · #254 click-to-place-caret · #255 selection
  (shift+arrows + drag + highlight + type-over) · #256 clipboard (⌘C/⌘X/⌘V) · #257 motion parity (Home/End,
  ⌥←→ word, ⌘←→ line, ⌘↑↓ doc, Up/Down) · #258 split-file panes persist across restart.
- **Tier 1 wedge — the clickable `file:line:col` (#196 + #214 OSC8) already shipped** in M12.2.
- **NOT yet adopted from Tier 0's "how":** the gpui substrate swaps (`EntityInputHandler`, `StyledText
  .with_highlights`, `uniform_list`) — M15 hand-rolled the render/input the intake suggested gpui could carry.
  These are a perf/correctness refactor still on the table (they also unlock virtualization for large files).
- **Deferred within Tier 0:** #259 (make the split-file *pane* editable, not just the tab — a `PaneContent`
  model change over the 16+ `active_tab().editor()` sites) is filed for scheduling.

**So the frontier is now Tier 2/3** (tree-sitter, LSP, anchors, multi-cursor, multibuffer, the fusion wedge). The
DEEP editing discovery for that frontier — the granular "Zed editing × Warp slickness" feature map + the gap
analysis against post-M15 Marley — lives in **[[zed-editing-discovery]]** (discovery Round 2).

## Two wins available NOW (before "the editor")
1. **#196** (already in M12.2) — clickable file paths → open in the code view — IS the wedge's first step.
2. Adopt gpui `uniform_list` + `StyledText.with_highlights` for the EXISTING CodeView + terminal render:
   a perf/correctness win today AND the editor substrate; also makes **#198 (scrollbar)** fall out via gpui's
   scroll handles instead of hand-rolled offset math.

## Clean-room provenance (§20)
- **gpui capabilities** = our own Apache-2.0 dependency's public API (already vetted through cargo-deny) —
  free to use directly.
- **Zed material** = BEHAVIOR + PUBLISHED-WRITING reference only (zed.dev/blog + zed.dev/docs); NO Zed GPL/AGPL
  source was read. Reimplement from the descriptions; do not copy. (Same discipline as `docs/warp_architecture/`.)
- Key Zed writeups to reimplement-from (not copy): gpui ownership model, Rope+SumTree, text coordinate
  systems + anchors, syntax-aware editing, syntax-aware tasks (runnables→terminal), language-extensions/LSP.

## Composes with
- **[[embedded-agent-browser-chromium-cdp]]** — editor + terminal + browser + agents + knowledge = the complete
  agentic workbench. The `EditOrigin::Agent` hook is the agent-writes-to-files seam; CDP is the agent-sees-web seam.

## Promotion
Candidate, not a pipeline doc. This is a MILESTONE (M-editor / M3-era), not a sprint. Promote via `/work` when
scheduled; likely sequence Tier 0 → 1 first (reachable, overlaps M12.2 polish), then Tier 2 (tree-sitter + LSP)
as the greenfield milestone. Record the architecture choice as a forge AD when promoted.
