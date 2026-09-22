# Zed per-crate references (Marley round 2)

Granular per-crate deconstruction of Zed's editing-critical crates, mirroring [`../../warp_architecture/crates/`].
Complements the strategic [`../subsystems/`](../subsystems/00-overview.md) map — the subsystem docs are the
*why/how*, these are the *API reference + the concrete Marley reimplementation map*. Source-derived from the Zed
clone (GPL-3.0, session scratch, never committed). Each doc carries a license row + per-capability provenance so
the GPL boundary stays auditable — see [`../README.md`](../README.md).

## The 16 crate references

| Crate | License | Marley reimplementation headline |
|---|---|---|
| [gpui](gpui.md) | **Apache-2.0** | Adopt directly — `uniform_list`/`ShapedLine::split_at`/`EntityInputHandler`/`.key_context()`/`render_to_image` each *delete* Marley hand-rolled code. Zero copyleft. |
| [sum_tree](sum_tree.md) | **Apache-2.0** | The augmented-B-tree engine; adoptable into any layer incl. the brain. Optional/last (only for custom seekable summary dimensions). |
| [rope](rope.md) | GPL-3.0 | Study-only — keep **ropey (MIT)**, which already ships the UTF-16 methods. |
| [text](text.md) | GPL-3.0 | The CRDT buffer. Marley: **delta-log anchors first** (rebased through the existing `BufferDelta` — GPL-clean); full `Fragment`/`Locator` CRDT deferred. |
| [clock](clock.md) | GPL-3.0 | Lamport + version-vector. **Defer the whole crate** — `BufferVersion(u64)` suffices until true concurrent same-region co-editing. |
| [editor](editor.md) | GPL-3.0 (51 dependents) | The 54-module catalog: the 6-layer DisplayMap, `SelectionsCollection` multi-cursor, `handle_input` IME. Reimplement the logic; anchors are the prerequisite. |
| [multi_buffer](multi_buffer.md) | GPL-3.0 | The signature capstone (`singleton` unifies the plain editor). Marley: excerpts over N `marley_editor::Buffer`s; scheduled LAST, gated on anchors + search. |
| [language](language.md) | GPL-3.0 (74 dependents) | The incremental `SyntaxMap` engine — the crown-jewel glue to re-author (→ `marley_syntax`). |
| [language_core](language_core.md) | GPL-3.0 | The pure data layer (`Grammar`/`HighlightMap`/configs) → a `marley_grammar` crate. |
| [languages](languages.md) | GPL-3.0 | Registration + LSP-adapter wiring; the grammar *assets* live in a separate sibling crate. |
| [lsp](lsp.md) | GPL-3.0 (25 dependents) | A self-contained protocol client. Marley builds from the **published spec + MIT `lsp-types`**, not Zed's fork. |
| [diagnostics](diagnostics.md) | GPL-3.0 | UI-only (the merge lives in `project`, the squiggle in `editor`). Reuse #198 scrollbar for gutter marks. |
| [project](project.md) | GPL-3.0 | The host-agnostic hub. **Marley's `Project` ≈ Zed's Worktree**; a light local hub is the multi-workspace direction. |
| [worktree](worktree.md) | GPL-3.0 | Immutable snapshot + `BackgroundScanner` + `IgnoreStack` over the MIT `ignore`/`notify` crates. |
| [fs](fs.md) | GPL-3.0 | The `Arc<dyn Fs>` trait + **`FakeFs`** — Marley's highest-leverage testability adoption (a scanner that hits cov/MSI-100). |
| [search](search.md) | GPL-3.0 | Content search (Marley has **none** today) → editable multibuffer results, phased. |

## The through-line (see the [subsystem overview](../subsystems/00-overview.md))
The permissive base — **gpui + sum_tree (Apache), tree-sitter + grammars + `.scm` + `lsp-types` + `ignore`/
`notify`/`regex` (MIT)** — is adoptable directly. The GPL is almost entirely in Zed's *orchestration logic*
(`editor`/`text`/`language`/`project`/`multi_buffer`), which Marley re-authors from concepts inside the
intended-GPL editor tier. **Anchors** (delta-log, `[Marley-original]`) are the universal prerequisite. The sold
**brain** stays clean via the `EditOrigin::Agent` seam.
