# diagnostics

> Per-crate reference (Marley Zed-architecture, round 2) — crate dir `crates/diagnostics`. The EDITOR
> reference is **Zed** (github.com/zed-industries/zed, GPL-3.0). Warp has **no** counterpart crate
> (Warp is a terminal; diagnostics is an editor concept), so this is a Zed-only reference.

| Field | Value |
|-------|-------|
| Subsystem | [05 — LSP & Language Intelligence](../subsystems/05-lsp-language-intelligence.md) |
| License | **GPL-3.0-or-later** — explicit per-crate `license = "GPL-3.0-or-later"` + a `LICENSE-GPL` symlink. `[Zed-derived]` |
| Internal deps | **16** |
| Used by | **1** (`zed` — the top-level app) |
| Size | `diagnostics.rs` 1154 · `buffer_diagnostics.rs` 1018 · `diagnostic_renderer.rs` 332 · `items.rs` 247 · `toolbar_controls.rs` 173 (+ `diagnostics_tests.rs` 2153) |

## Purpose

`diagnostics` is the **diagnostics UI** crate — the *rendering* surface for problems reported by language
servers. It is **not** the fan-in: the actual `publishDiagnostics` handling, severity/group merge, and
`DiagnosticSummary` aggregation live upstream in `project::lsp_store`, and the wavy-underline *squiggle* is
drawn by the `editor` crate itself. This crate owns four presentation pieces:

1. the **inline message block + hover content** (`DiagnosticRenderer` / `DiagnosticBlock`),
2. the **project-wide diagnostics view** (`ProjectDiagnosticsEditor` — a `MultiBuffer` of every file's
   problems),
3. the **single-buffer diagnostics view** (`BufferDiagnosticsEditor`),
4. the **status-bar indicator** (`DiagnosticIndicator`) and the **toolbar controls** (warnings/refresh).

The seam that keeps the layering clean: `init` calls **`editor::set_diagnostic_renderer(...)`**, an
inversion that lets the `editor` crate draw diagnostic blocks/hovers **without depending on this crate**.
`diagnostics` implements the `editor::DiagnosticRenderer` trait; `editor` calls it back.

## Key types, modules & public API

### `diagnostic_renderer.rs` — the render trait impl

- **`DiagnosticRenderer`** — implements `editor::DiagnosticRenderer`:
  - **`render_group(group, buffer_id, snapshot, editor, lang_registry, cx) -> Vec<BlockProperties<Anchor>>`**
    — turns a diagnostic group into editor blocks, each placed at
    **`BlockPlacement::Near(snapshot.buffer_snapshot().anchor_after(range.start))`** — i.e. **anchored**,
    so the block survives edits.
  - **`render_hover(group, range, buffer_id, lang_registry, cx) -> Option<Entity<Markdown>>`** — the
    Markdown shown in the editor's `hover_popover` for the group under the cursor.
  - **`open_link(...)`** — intercepts diagnostic-navigation links.
- **`DiagnosticBlock { initial_range: Range<Point>, severity: DiagnosticSeverity, markdown: Entity<Markdown>,
  copy_message, diagnostics_editor }`** — one rendered block. `render_block` builds an
  **`h_flex().border_l_2().bg(bg).border_color(border)`** with a `MarkdownElement` and a `CopyButton`.
  Severity → theme color via **`cx.theme().status()` (`StatusColors`)**: `ERROR` → `error` /
  `error_background`, `WARNING` → `warning`, `INFORMATION` → `info`, `HINT` → `hint` (no hard-coded reds).
- **`diagnostic_blocks_for_group`** — groups a diagnostic's `related_information` under its primary entry
  (via the shared `group_id`). Related entries **≥ 5 rows** from the primary get an in-document
  markdown link `file://#diagnostic-{buffer}-{group}-{ix}`; clicking it (handled in `DiagnosticBlock::open_link`
  → `jump_to`) navigates between the primary site and its related sites. `append_source_and_code` decorates
  the message with the diagnostic `source`/`code` (and a `code_description` URL if present).

### `diagnostics.rs` — the project-wide view

- **`ProjectDiagnosticsEditor`** — a `MultiBuffer` + `Editor::for_multibuffer` that aggregates **every
  file's** diagnostics into one scrollable buffer. Holds
  **`diagnostics: HashMap<BufferId, Vec<DiagnosticEntry<text::Anchor>>>`** (anchored!), the per-buffer
  block ids, a `DiagnosticSummary`, `paths_to_update`, and an `include_warnings` toggle. Updates are
  **debounced** (`DIAGNOSTICS_UPDATE_DEBOUNCE = 50 ms`, summary `30 ms`) to avoid thrash on rapid
  republish. The **`Deploy`** action opens it; empty state renders **"No problems in workspace"** (with a
  "Show N warnings" button when warnings are hidden). Implements the workspace `Item` trait (tab content,
  save/reload no-ops, split, nav-history).
- **`IncludeWarnings(bool)`** — a `gpui::Global` toggle; `actions!` defines `Deploy`, `ToggleWarnings`,
  `ToggleDiagnosticsRefresh`.
- Helpers `context_range_for_entry` / `heuristic_syntactic_expand` compute how much surrounding source to
  show around each entry in the multibuffer.

### `buffer_diagnostics.rs` — the single-buffer view

- **`BufferDiagnosticsEditor`** — the same idea scoped to one buffer (`set_diagnostics`,
  `diagnostics_are_unchanged`, `max_diagnostics_severity(include_warnings)`, its own `DiagnosticSummary`).
  Also a workspace `Item`.

### `items.rs` — the status-bar indicator

- **`DiagnosticIndicator`** (a `StatusItemView`) — shows error/warning counts with `XCircle` / `Warning`
  icons (a `Check` icon when clean), plus the **diagnostic under the cursor** as a clickable button that
  fires `GoToDiagnostic` / opens the project view. Reads `project.diagnostic_summary(...)` and updates on
  a debounced task. Hidden when `ProjectSettings.diagnostics.button` is off.

### `toolbar_controls.rs`

- **`ToolbarControls`** (a `ToolbarItemView`) + the **`DiagnosticsToolbarEditor`** trait
  (`include_warnings` / `toggle_warnings` / `is_updating` / `stop_updating` / `refresh_diagnostics` /
  `get_diagnostics_for_buffer`) — the include-warnings and refresh toggles, abstracted so both the
  project and per-buffer editors satisfy them.

### Everything is anchored — the crux

The renderer receives `DiagnosticEntryRef<'_, Point>` and emits `BlockProperties<Anchor>`; the project
view stores `DiagnosticEntry<text::Anchor>`. That is the whole point: diagnostics arrive from the server as
**untrusted UTF-16 positions**, get **clipped and anchored upstream** in `lsp_store`
(`DiagnosticEntry<Unclipped<PointUtf16>>` → clip → `Anchor`), and this crate renders **anchored** entries
that survive subsequent edits. Without an anchor layer, a diagnostic republished after three keystrokes
would underline the wrong span.

## Depends on (internal)

**16** workspace crates:

- **`editor`** — `Anchor`, `Editor`, `MultiBuffer`, the `DiagnosticRenderer` trait,
  `display_map::{ BlockPlacement, BlockProperties, BlockStyle }`, `hover_popover::diagnostics_markdown_style`.
  The primary coupling.
- **`project`** — `DiagnosticSummary`, `Project`, `ProjectPath`, `project_settings::{ DiagnosticSeverity,
  ProjectSettings }` (the fan-in source this crate consumes).
- **`language`** — `Diagnostic`, `DiagnosticEntry` / `DiagnosticEntryRef`, `Buffer`, `BufferSnapshot`,
  `Point`.
- **`lsp`** — `DiagnosticSeverity` (the enum drives the severity→color match).
- **`markdown`** — `Markdown` / `MarkdownElement` / `CopyButton` for the message body.
- **`theme`** / **`theme_settings`** — `ActiveTheme`, `StatusColors`, font/line-height for block sizing.
- **`workspace`** — `Item`, `StatusItemView`, `ToolbarItemView`, nav history.
- **`ui`**, **`settings`**, **`text`** (`Anchor`/`Point`/`BufferId`), **`zed_actions`**, **`agent_settings`**,
  **`component`**, **`collections`**, **`util`**.

## Used by (internal dependents)

**1** — only the top-level **`zed`** app, which wires `diagnostics::init(cx)`. It is a leaf UI crate; nothing
else consumes it.

## Related crates

- `project` (`lsp_store`) — computes the diagnostics this crate renders: `setup_lsp_messages` registers
  `on_notification::<PublishDiagnostics>`, merges by severity, assigns `group_id`, produces
  `DocumentDiagnostics { Vec<DiagnosticEntry<Unclipped<PointUtf16>>> }`, and tracks `DiagnosticSummary`.
- `editor` — draws the actual **wavy underline** squiggle (`HighlightStyle { underline: UnderlineStyle {
  wavy: true, … } }`) and the gutter/scrollbar `ColoredRange` marks in its own `element.rs`/`display_map.rs`;
  it *calls back* into this crate's registered `DiagnosticRenderer` for the blocks/hover.
- [`lsp`](./lsp.md) — the protocol client whose notifications ultimately feed the summaries here.

## Marley — reimplementation on our stack

**Marley today: none.** No diagnostics, and (the hard prerequisite) **no anchor layer** — every
diagnostic is a position into a buffer version the user has already typed past. So diagnostics can only be
built **after** anchors + the UTF-16 coordinate module land (see the `lsp` crate doc and subsystem §11).

**Why diagnostics is the *first* LSP feature to build:** it is a pure **inbound notification** — no request
round-trip, no completion menu, no resolve step. `publishDiagnostics` arrives, you anchor it and draw. That
makes it the cheapest-value-first target once the client + anchors exist.

**Rebuild plan (all `[Marley-original]` on gpui):**

1. **Fan-in** in Marley's `LspStore`: an `on_notification::<PublishDiagnostics>` handler → clip each entry's
   UTF-16 range → `Anchor` → a `DiagnosticEntry<Anchor>` list keyed per buffer + a `DiagnosticSummary`
   count. (Mirrors `merge_lsp_diagnostics`; the `Diagnostic`/`DiagnosticSeverity` shapes are MIT.)
2. **Squiggle:** a wavy-underline decoration in Marley's editor render — gpui exposes
   `UnderlineStyle { wavy: true, thickness, color }` over the diagnostic's anchor range; unused-code
   (`UNNECESSARY` tag) fades out instead.
3. **Gutter / scrollbar marks: reuse #198.** Per-severity marks on the **`scrollbar_thumb` / `at_bottom`
   track** already shipped for the terminal viewport (`crates/marley_app/src/viewport.rs`) — the diagnostic
   marker gutter is the same track with `ColoredRange`-style ticks.
4. **Hover popover:** a gpui `anchored()` block at the token showing the message (plain text first — Marley
   has no Markdown popover renderer yet — Markdown later). The **`h_flex().border_l_2()` + severity `bg`/
   `border` from the theme's status colors** is a clean recipe to re-implement (no hard-coded reds; use
   Marley's theme system, which already has accent/status washes from the M12.2 WARP-PARITY work).
5. **Project panel (optional, later):** a `MultiBuffer` analog aggregating every file's diagnostics, sorted
   by location, grouped by `group_id`, debounced ~50 ms — plus a **status-item** count indicator (Marley
   already has a status/tab-rail surface from #203's completion-badge work to hang it on).
6. **Related-info navigation:** the `file://#diagnostic-…` synthetic-link trick maps onto Marley's #196 link
   mechanism conceptually (intercept the URL, jump to the anchored sibling entry).

## Provenance

- **`[permissive/public: LSP spec, lsp-types MIT]`** — the `Diagnostic`, `DiagnosticSeverity`,
  `DiagnosticTag`, `DiagnosticRelatedInformation`, and `PublishDiagnosticsParams` **data shapes** are the
  published LSP spec / MIT `lsp-types`. Marley reuses them freely.
- **`[Zed-derived]`** — this crate is **GPL-3.0-or-later**. The *specific UI design* (grouped message
  blocks with a left border + copy button, the `file://#diagnostic-…` link-back navigation, the
  MultiBuffer project panel, the status indicator, the `set_diagnostic_renderer` inversion) is **design
  reference only**; Marley writes its own gpui code.
- **`[Marley-original]`** — Marley's diagnostic decorations, gutter/scrollbar marks, hover popover, and any
  project panel, all on gpui, reusing Marley's #198 scrollbar / #196 links / theme status colors.
- **Brain boundary:** diagnostics are a **GPL editor feature** (fine to be Zed-derived in design). This
  crate does **not** touch edit prediction — the AI-edit surface that repurposes LSP goto-definition as RAG
  is the **proprietary brain** and must stay clean of any Zed-derived source (subsystem §9).

## Notes / gotchas

- **Scope trap:** this crate does **not** compute diagnostics and does **not** draw the squiggle. Fan-in /
  merge = `project::lsp_store`; underline + gutter marks = `editor`. `diagnostics` renders **blocks,
  hovers, the project/buffer views, and the status/toolbar items**. Easy to over-scope a Marley port.
- **Anchors are a hard dependency** — `DiagnosticEntry<Anchor>` everywhere; the anchor layer must exist
  first, exactly as for the rest of the LSP subsystem.
- **Registration is a global inversion** — `editor::set_diagnostic_renderer` in `init` lets `editor` draw
  without a dependency edge back to `diagnostics`. Replicate this direction in Marley to avoid a cycle.
- **Debounced** (50 ms updates / 30 ms summary) — republished diagnostics on every keystroke would
  otherwise churn the multibuffer and status bar.
- **Grouping via `group_id`** — a primary error and its `related_information` render as one visual group;
  the ≥ 5-row threshold decides when a related entry gets its own navigable link vs. inlining.
