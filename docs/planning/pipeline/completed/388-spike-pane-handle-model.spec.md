---
pipeline_id: 9b78e721-6fe2-4b37-8d99-74b0408349ca
ticket: forge#388 (4c8cbe75-f7c6-4d98-9893-130e22d922e1) · local docs/planning/tickets/open/TICKET-388-spike-pane-handle-model.md
aar_id: — (opened at promotion)
status: Phase 5 — Complete PASS
title: SPIKE — the shared-handle pane model: sections navigate, panes compose (one instance, many views)
type: spike
milestone: M26
references:
  - docs/planning/pipeline/queued/385-sectioned-rail.spec.md (the navigator half this composes with)
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md (research map)
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md (research map)
---

## Title
Design the architecture that delivers chad's locked model A + his 2026-07-22 refinement: the rail is
FOUR fixed-order per-workspace sections — **Editor · Terminals · Panes · Browser** — where the sections
NAVIGATE what's open and the **Panes section dynamically collects every SPLIT VIEW** (opening a split
allocates a Pane into the Panes section, while the thing you split STAYS listed under Editor/Terminals —
the "original stays open" rule). "make a Pane" = save + name a multi-cell arrangement; any open thing
can be added to a pane cell. Cross-workspace panes get a **top-level GLOBAL Panes section** above the
workspaces. A SPIKE — the deliverable is a decision-complete design + a sliced refactor train, not
shipped code. (This supersedes #385's pane-nesting-under-tab; the un-nesting into the Panes section is
part of the M27 train this spike produces, NOT a #385/#386 redo.)

## Scope
### In
Six questions, each answered in a design doc (`docs/marley_architecture/` or
`docs/planning/design-notes/`):
0. **The Panes SECTION + the four-section rail (chad's 2026-07-22 refinement — the centerpiece).**
   The rail becomes FOUR per-workspace sections `Editor · Terminals · Panes · Browser` (#385/#386
   shipped three + nested split panes under their tab — this UN-NESTS them). Design: **(a)** how a
   SPLIT VIEW dynamically allocates into the Panes section — a Pane entry appears the moment you split,
   keyed to the live split view, and disappears when it closes; **(b)** the "original stays"
   relationship — the thing you split (a terminal/editor) STAYS listed under Editor/Terminals, and the
   Pane under Panes is a VIEW referencing it (one-instance-many-views, question 2); **(c)** a Pane
   row's identity (a `PaneId` in a tab's grid today → a content handle) + what clicking it does (focus
   that split view); **(d)** the GLOBAL cross-workspace Panes section — a top-level section above the
   workspaces for panes composing content from >1 workspace (needs the multi-workspace model, still
   ahead: design the shape, gate the build on multi-workspace). This section is WHY the handle model
   (below) is needed — a Pane referencing a thing in another section or workspace cannot own it.
1. **The registry/handle shape.** Content moves behind an id: a `ContentId`-keyed app-side
   registry the gpui-free model layer (tabs.rs/workspace.rs) references by id instead of owning
   by value. Evaluate: generalizing the EXISTING PaneId-keyed pattern (the map's noted asymmetry —
   terminals are already semi-handle-based: `S = TerminalSession` + app-side
   `HashMap<PaneId, AgentRun>`/remotes) vs an inversion where `Workspace` itself hosts the
   registry. gpui `Entity<T>`/`WeakEntity` ownership (Apache-2.0 — reading it is adoption) studied
   for the RENDER layer only; the model layer stays gpui-free (a hard constraint — every model
   module is pure by design).
2. **One instance, many views.** Shared Buffer, per-view caret/scroll/selection. What supersedes
   the #259 two-Buffers stance and how the #275 conflict machinery collapses (same-instance views
   can't conflict with each other; external-change detection remains). The view-state split table:
   what is per-view (caret, scroll, selection, find state?) vs per-instance (buffer, undo history,
   dirty flag, language/LSP binding).
3. **The migration ripple, inventoried and sized per file.** From the 2026-07-22 map:
   `PaneContent` (workspace.rs:317) + `TabContent` (tabs.rs:23) → id-bearing; `PaneState::kind()`
   derivation; accessors (`grid()`/`editor()`/`cockpit_section()`, `PaneState::terminal()`/
   `code_view()`); the grid-layout leaf codec (leaf alphabet `t/t=<cwd>/c=<path>/f/g`, reserved
   framing bytes); `rail_rows`; `focus_label`; the two `EditorSurface` construction sites
   (tabs.rs:64; app.rs:1962/:5255). Each entry sized (S/M/L) with its test surface named.
4. **Nameable pane arrangements** (chad's "panes become their own thing", model-A form): the
   persistence shape for saved, named multi-cell arrangements referencing content by id —
   extend the shell codec vs a new settings table (the #204 `[[workflows]]` round-trip idiom);
   dangling-id semantics on restore.
5. **The refactor train** — an ordered list of one-shippable-slice tickets (M27 candidates), each
   with its risk named, sequenced so every intermediate state ships green (the strangler route:
   which content kind converts first and why — terminals' existing semi-handle shape suggests
   terminal-first; validate that instinct against the editor's two-Buffer debt).
- **Optional scratch proof** — a minimal registry + two-views compile-proof in scratch/branch;
  never staged.
- Forge captures: `architecture-decision-record` for (a) the handle model, (b) the
  one-instance-many-views invariant; the AAR carries the sweep's findings.

### Out (explicitly deferred)
- ANY shipped app code (the spike's hard wall — proof code stays scratch).
- The refactor itself (→ the M27 train this spike produces).
- Browser content in panes (#389 owns the substrate question; this spike leaves a
  `ContentKind::Browser` slot in the design).
- Drag-and-drop composition UX (a later train; the spike designs the model, not the gesture).

## Reference (§20)
**N/A — Marley-specific architecture** (chad's model-A lock, 2026-07-22). Zed's workspace is the
closest studied system — via docs/zed_architecture/subsystems/07-workspace-panes-palette.md +
03-editor-multibuffer.md (our clean-room research maps: Zed panes hold Views of shared
entity-backed Buffers — the same one-instance-many-views destination) — **research maps only;
never the GPL source.** gpui itself (Apache-2.0) IS readable/adoptable and its `Entity`
ownership model is a primary study input for the render layer.

### Prior art
1. **Behavior maps** — 07-workspace-panes-palette.md (Zed's pane/item model: items are entities,
   panes hold views — the existence proof that model A works at scale on this exact UI stack);
   03-editor-multibuffer.md (shared-buffer excerpt views — the per-view vs per-buffer state split,
   which is question 2's frame). Research, not source.
2. **Published material** — standard MVC/entity-component splits; nothing protocol-shaped here.
3. **Our permissive deps — the highest-yield leg.** **gpui (Apache-2.0)**: `App` owns entities;
   `Entity<T>` is a cheap cloneable handle; `WeakEntity` breaks cycles; reading
   `~/.cargo/registry/src/**/gpui-0.2.2` is adoption. The spike MUST answer: does the render layer
   adopt `Entity<T>` for content instances, or does Marley keep its own registry + ids end-to-end
   (preserving the gpui-free pure-model discipline that gives cov/MSI 100 on every model module)?
   That tension — gpui's idiom vs the house purity discipline — is the spike's central fork, and
   the reason this is a spike, not a ticket.
4. **In-repo owners** — the PaneId-keyed app-side maps (agents/remotes), `Workspace<S>`'s generic
   session seam (workspace.rs — already spawn-per-call, mock-tested), the #275 conflict machinery,
   the #259 focus-aware `active_editor()` spine (~75 call sites moved once already — evidence
   sizing matters), and the shell/grid codecs.

## Locked-In Decisions
- **D1 — Model A is settled** (chad, 2026-07-22): sections navigate; panes compose; nameable
  arrangements are the "panes as their own thing" form. The spike does NOT relitigate A-vs-B.
- **D2 — The model layer stays gpui-free.** Whatever handle shape wins, tabs.rs/workspace.rs
  remain pure and fully unit-testable; gpui types stop at the render shim. (House discipline; also
  what keeps the eventual refactor testable at cov/MSI 100.)
- **D3 — Docs-only.** Nothing ships; scratch proof code never staged. Verified at the gate by an
  empty `crates/` diff.
- **D4 — The output is decision-complete:** a successor ticket must be able to start its P2 design
  from the doc without reopening the forks.
- **D5 — The rail is FOUR sections `Editor · Terminals · Panes · Browser`** (chad, 2026-07-22). The
  Panes section dynamically lists every split view (the split's origin STAYS in its home section — the
  Pane is a view referencing it); a top-level GLOBAL Panes section holds cross-workspace panes (designed
  now, built when the multi-workspace model lands). #385/#386's pane-under-tab nesting is superseded by
  the Panes section — the un-nesting ships in the M27 train, not as a #385/#386 amendment.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The spike shall produce a design doc recording the chosen handle/registry model (incl. the gpui-Entity vs own-registry fork, decided with rationale) and the one-instance-many-views state-split table. | Doc review — decision-complete per D4. |
| REQ-002 | The design doc shall carry the full migration-ripple inventory with per-file S/M/L sizing and named test surfaces, covering at minimum the eight seams the 2026-07-22 map flagged. | Doc review vs the map's list. |
| REQ-003 | The spike shall record the nameable-arrangement persistence shape incl. dangling-id restore semantics. | Doc review. |
| REQ-004 | The spike shall record forge ADs for the handle model and the one-instance-many-views invariant, and emit an ordered one-shippable-slice-each M27 ticket list. | Forge records exist; list in doc. |
| REQ-005 | The spike shall ship zero app code: `git diff` under `crates/` is empty at complete. | Diff review at the gate. |
| REQ-006 | The design doc shall specify the four-section rail (Editor/Terminals/Panes/Browser), the dynamic split-view→Panes allocation with the original-stays relationship, the Pane-row identity + focus behavior, and the global cross-workspace Panes section (its shape + the multi-workspace gate on building it). | Doc review. |

## Phase Plan
- **P2 Design** — the study plan (gpui Entity read; the Zed research maps; the in-repo seams) +
  the doc's skeleton. Human checkpoint on the central fork before writing the verdict.
- **P3 Implement** — write the design doc (the spike's "implementation" is the document; optional
  scratch proof).
- **P3.5 Inspect** — critics attack the doc's confident sentences (the pre-authored method — 7/7
  hit rate): the state-split table's edge cases (undo across views? find state?), the codec
  migration's back-compat, the train's green-intermediate claim.
- **P4 Validate** — REQ checklist against the doc; empty-crates-diff check; gate `--fast`
  (docs-only commit).
- **P5 Complete** — ADs + AAR capture, archive, close #388; hand the M27 train to /spec or /work.
