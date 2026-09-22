# Zed subsystem overview — the strategic map (Round 1 synthesis)

Ties together the 9 subsystem deconstructions below. Built from a source-level read of the Zed clone (GPL-3.0,
in session scratch — never committed), the way the Warp map was built. The goal: a reimplementation plan for
**Zed-level editing** in Marley's own code, with the provenance boundary that keeps the sellable **brain** clean.

## The provenance map (the legal-relevant headline)

The single most important finding, consistent across all 9 subsystems: **the hard, reusable machinery is
permissively licensed; the GPL is almost entirely in Zed's *composition/orchestration logic*, not in the
primitives.** So Marley reimplements a bounded set of *glue*, inside the intended-GPL editor tier, and adopts the
rest directly.

| Layer | License | Marley posture |
|---|---|---|
| **gpui** (elements, `EntityInputHandler`/IME, `StyledText`+highlights, `uniform_list`, `anchored`, focus + **KeyContext**, text shaping) | **Apache-2.0** | Adopt directly — it's already Marley's own dependency. NOT part of the GPL surface. |
| **`sum_tree`** (the augmented B-tree behind the rope + multibuffer + display map) | **Apache-2.0** | Adopt directly. |
| **tree-sitter + the grammars + `.scm` query files** | **MIT / grammar-authors'** | Adopt directly — one dep yields highlight/indent/brackets/outline/injections/runnables. |
| **LSP protocol + `lsp-types` + JSON-RPC** | **published spec / MIT** | Write a clean client *from the spec*. |
| the underlying algos — ropes, `notify`/`ignore`/`globset`/`regex`/`aho-corasick` | **MIT/Apache** | Adopt directly (ropey is already Marley's store). |
| **`editor`, `multi_buffer`, `text`, `rope`, `clock`, `language`, `project`, `workspace`, `vim` — the ORCHESTRATION** | **GPL-3.0** | **Reimplement from the design (concepts), not the source.** This is the bounded work, all inside Marley's intended-GPL editor tier. |
| Zed's **edit-prediction / AI-edit** path | GPL glue, but the *concept* is the brain | **Marley-original brain code** — must never link Zed-derived editor code (open-core boundary). |

**The boundary that protects the money:** `EditOrigin { Human, Agent }` already exists in `marley_editor`
(`editor/types.rs`). It is the exact seam where the sold **brain** applies edits to the GPL editor without
inheriting copyleft. Every doc tags capabilities `[Zed-derived]` / `[gpui Apache-2.0]` / `[permissive/public]` /
`[Marley-original]` so the eventual legal review is mechanical and the brain stays clean.

## The one prerequisite that unlocks everything: ANCHORS

Five of the nine subsystems independently converge on the same gate. Marley's `marley_editor` today stores raw
`CharOffset`s (single cursor) that go stale on any edit. **Anchors** — logical positions that survive edits —
are the prerequisite for: multi-cursor (selections must auto-adjust), async LSP (a reply that lands 200ms later
must map to the right place), diagnostics/marks, and the multibuffer. The cheap, GPL-clean path (from the text
doc): **delta-log anchors** — `Anchor{version, offset, bias}` rebased through Marley's *existing* `BufferDelta`
log — `[Marley-original]`, no SumTree rewrite, no CRDT. The full Zed CRDT + Lamport clock + op-based undo is a
*deferred* milestone gated on real concurrent multi-writer (human+agent co-edit) — not needed for single-user.

## The reimplementation sequence (dependency-ordered)

1. **KeyContext** (gpui, Apache-2.0) — context-scoped keybindings. Small, contained (Marley's keymap is already
   a pure isolated module), and it fixes the live **⌘D collision** (#197 new-terminal vs editor select-next-match)
   plus every looming ⌘F/⌘//Enter/Esc conflict at once. **Do first.**
2. **gpui substrate adoption** — `uniform_list` (virtualized rows: code, terminal grid, file tree, finder) +
   `StyledText.with_highlights` (shaped spans, replaces one-div-per-line + the hand lexer's render) +
   `EntityInputHandler` (real IME + caret geometry). A perf/correctness refactor that also becomes the syntax
   substrate.
3. **tree-sitter** — one dep → semantic highlight (retire `code_syntax.rs`) + indent + brackets + outline +
   injections + **runnables**. Renders straight into `StyledText.with_highlights`. Incremental, off-main-thread.
4. **Anchors** (delta-log, Marley-original) — the quiet prerequisite; land before 5 + 6.
5. **Multi-cursor + line ops** — upgrade the single `(caret, anchor)` to `marley_editor::SelectionSet` (the type
   already exists, single-member today); on anchors. Big power payoff, no external service.
6. **LSP** — a clean client from the spec; results anchored; goto reuses the shipped **#196** `file:line:col`→
   editor; diagnostic marks reuse **#198** scrollbar; edits via `EditOrigin::Agent`; format-on-save hooks #252.
7. **Project search → multibuffer** — content search (Marley has none today) → editable multi-file excerpts (the
   signature capstone), on anchors + a DisplayMap-style transform layer.

## Marley's structural WIN (not just parity): the terminal↔editor↔block fusion

Zed has an editor + a *plain* terminal + a task system — but **no command-block model at all** (it polls the OS
process table; no shell hooks). So even Zed's own tasks decay at the sink: a runnable becomes a whole terminal
*tab* with a plain-text scrollback line, and the structured result (exit code, cwd, rerun spec) is thrown away
because there's nowhere to hang it. **Marley already owns the block infrastructure.** Routing a tree-sitter
`runnable` into a first-class **Block** (inline status pill, framed output, per-block rerun, block-scoped
jump-to-failure, brain-observable pass/fail) is *reuse of shipped infra* for Marley and a whole new subsystem for
Zed. This is the wedge — where terminal-first *beats* Zed rather than matches it. The front half (clickable
`file:line:col` → editor) already shipped (#196/#214).

## A structural insight for the workspace direction

Marley's `Project{root, name, is_git}` is really equivalent to Zed's **Worktree**, not Zed's **Project** (the
host-agnostic hub that ties worktrees + language servers + buffers + search). Marley has no hub — and a
lightweight local-only hub is exactly where chad's multi-workspace/launcher direction lands:
`Workspace(window) → Project(hub) → Worktree(folder)`.

## Index — the 9 subsystem deconstructions

| # | Subsystem | Doc | Headline |
|---|---|---|---|
| 01 | gpui / UI framework | [01-gpui-ui-framework.md](01-gpui-ui-framework.md) | Apache-2.0; adopt `EntityInputHandler`/`StyledText`/`uniform_list`/`KeyContext` directly |
| 02 | text / buffer / anchors | [02-text-buffer-anchors.md](02-text-buffer-anchors.md) | `sum_tree` Apache / `rope`+`text` GPL; delta-log anchors first, CRDT deferred |
| 03 | editor / multibuffer | [03-editor-multibuffer.md](03-editor-multibuffer.md) | the 6-layer DisplayMap, `SelectionsCollection`, the MultiBuffer capstone |
| 04 | language / tree-sitter | [04-language-syntax-treesitter.md](04-language-syntax-treesitter.md) | one parse → many readers; only the incremental `SyntaxMap` glue is GPL |
| 05 | LSP / language intelligence | [05-lsp-language-intelligence.md](05-lsp-language-intelligence.md) | clean client from the spec; edit-prediction = the brain seam |
| 06 | project / fs / search | [06-project-fs-search.md](06-project-fs-search.md) | Marley's Project ≈ Zed's Worktree; zero content search today |
| 07 | workspace / panes / palette | [07-workspace-panes-palette.md](07-workspace-panes-palette.md) | Marley already strong here; the Item/pane peerage model |
| 08 | terminal / tasks / fusion | [08-terminal-tasks-fusion.md](08-terminal-tasks-fusion.md) | Zed has NO blocks — the fusion wedge, concretely |
| 09 | vim / keymap / contexts | [09-vim-keymap-contexts.md](09-vim-keymap-contexts.md) | KeyContext (gpui) fixes ⌘D + every collision; do first |

## Status
Round 1 (subsystem map) complete, 2026-07-11. Round 2 = per-crate deep-dives of the editing-critical crates
(mirroring `warp_architecture/crates/`). Rounds 3–5: re-review the Warp map, redo the Marley architecture docs,
scrub Warp/Zed brand mentions from the Marley source.
