---
pipeline_id: c90869d1-0f23-490f-8e84-51445a2d4953
ticket: docs/planning/tickets/open/TICKET-353-editor-folds-not-cleared.md
status: Phase 5 — Complete PASS
title: editor_folds cleared on last-view close (#305 W-2 follow-up)
type: bug
milestone: M19
references:
  - docs/planning/pipeline/completed/305-code-folding.notes.md
  - docs/planning/pipeline/completed/319-editor-file-identity.notes.md
  - docs/planning/pipeline/completed/352-fold-overlay-projection.spec.md
---

## Title
Clear `editor_folds[path]` when a file actually closes — the #305 W-2 parked fix.
`app.rs` stores active folds in `editor_folds: HashMap<PathBuf, Vec<Anchor>>`
(decl app.rs:284); the fold verbs insert/remove, but nothing clears an entry when
the file closes. Consequences (both LOW, flagged by both #305 inspect critics):
a bounded anchor leak (one `Vec<Anchor>` per distinct-file-ever-folded, forever)
and a stale-fold-on-reopen edge (old anchors, stamped with the dead buffer
instance, resolve against the fresh buffer; the #305 F1 re-derive drops most but
not all — an old anchor landing on a valid region header re-folds spuriously).

## Scope
### In
- Clear the `editor_folds` entry for a file exactly when its editor **instance**
  fully drops — the last view's release (`ContentRegistry::release_view`
  returning `Some`), mirroring the #398 `release_grid_terminals` scrub of the
  terminal-side content-keyed maps.
- All instance-drop paths funnel through `crate::content::release_editor_views`
  (app.rs call sites: tab close :8491, file-strip close :8583, project close
  :8674, pane close :9910 and :20077, viewer speculative releases :5646/:5682;
  headless_drive.rs:1111) — the fix rides that choke, whatever wiring shape
  design picks.
- Unit tests per §7: clear-on-close per path, twin-view retention,
  reopen-presents-unfolded.

### Out (explicitly deferred)
- Fold persistence across close/reopen — declined at #305 (in-session model);
  this ticket re-affirms it (D2).
- File-rename fold-key migration — no rename surface exists today: LSP
  `RenameFile`/resource ops are rejected outright (#322 D2,
  `marley_lsp/src/workspace_edit.rs`), and there is no file-tree rename. A
  future rename feature must move/clear the fold key (ledger note at P5).
- Any change to the #305 fold verbs or the #352 fold projection.

## Reference (§20)
**Zed (the editor reference).** Per the behavior map
`docs/zed_architecture/subsystems/03-editor-multibuffer.md` (:29, :271, :335),
fold state in Zed lives in the editor view's DisplayMap transform stack
(FoldMap) — **view-scoped state that dies with the view**; the maps record no
cross-close fold persistence. Marley matches that BEHAVIOR: closing the last
view of a file discards its fold state; reopening presents the file unfolded
(the #305-locked in-session fold model — nothing persists). Clean-room: behavior
maps + our own substrate only; no copyleft source read.

### Prior art
1. **Behavior maps** — `docs/zed_architecture/subsystems/03-editor-multibuffer.md`
   (FoldMap is display-map/view state; no persistence semantics);
   `docs/zed_architecture/crates/language.md` :116 (folding is editor-layer, not
   language-layer). Warp maps: no editor-folding analog (terminal/cockpit).
2. **Published** — LSP `textDocument/foldingRange` defines *ranges*, not
   lifecycle/persistence; VS Code-style viewState fold restore is a product
   choice #305 already declined. Nothing further to adopt.
3. **Deps / in-repo substrate (the paying leg)** — no external crate owns this
   seam (gpui renders, ropey holds text; the state maps are ours — checked, no
   owner). The substrate we already ship DOES own the hook:
   `ContentRegistry::release_view -> Option<C>` (content_registry.rs:87) reports
   exactly the last-view drop, and `release_grid_terminals` (app.rs:5852, #398)
   is the established pattern — scrub the content-keyed maps (`agents`,
   `remotes`, `notify_ticks`, `last_agent`) inside the `Some(content)` arm.
   **ADOPT that pattern for the editor side; invent nothing.**

## React-first (parity)
N/A — no UI delta: state hygiene on the close paths. The only user-observable
consequence is a reopened file presenting unfolded instead of (rarely,
stale-anchor-dependent) re-collapsed — that is the bug being fixed, restoring
the #305-locked in-session behavior; no chrome, layout, type, color, or
affordance changes, and the React POC models no fold persistence to mirror.

## Locked-In Decisions
- **D1 — CLEAR POINT = instance drop, never view release.** The fold entry
  clears exactly when `release_view` returns `Some` for an editor instance (the
  last view went away). A twin view surviving elsewhere keeps the entry; the
  viewer's speculative releases (:5646/:5682) stay no-ops by construction (a row
  still holds the instance). Mirrors the #398 terminal-side scrub.
- **D2 — NO PERSISTENCE (re-affirm #305).** Fold state is in-session,
  per-open-instance; reopen presents unfolded. This kills the
  stale-fold-on-reopen edge at the root; the #305 F1 re-derive stays as
  defense-in-depth for live-instance re-projection.
- **D3 — RESOLVED at design: the census guard.** Two same-path instances under
  alias roots (#319 D3 keeps them separate) share one fold entry. A simple
  clear-on-any-instance-drop would REGRESS the case today's leak accidentally
  gets right (the survivor keeps its folds), so the scrub clears only when NO
  surviving editor instance stores the same path. Census = one
  `ContentRegistry::iter()` scan (the #397 surface find_open already walks),
  matching by exact `==` on stored spellings — the map's own key semantics,
  sound because #319 made stored paths canonical. Adds REQ-004.
- **D4 — RESOLVED at design: App-method wrapper over the pure primitive.**
  `content::release_editor_views` (content.rs:261) stays the pure seam but now
  returns the stored paths of instances that fully dropped (still dropping them
  inline — the #397 posture). A new App method `release_editor_views(&mut
  self, ids)`, sibling to `release_grid_terminals` (app.rs:5852), calls it and
  census-guard-scrubs `editor_folds`. ALL call sites (app.rs :5646 :5682 :8491
  :8583 :8674 :9910 :20077, headless_drive.rs:1111) migrate to the method.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the last view of an open file's editor instance is released (file-strip close, tab close, pane close, or project close), the system shall remove that file's `editor_folds` entry. | Unit tests per close path: fold → close → entry absent; mutation-covered (gate:5). |
| REQ-002 | WHILE another view of the same editor instance remains open (a twin), WHEN one view is released, the system shall retain the file's `editor_folds` entry unchanged. | Unit test: two views of one instance, release one → entry intact; fold count unchanged. |
| REQ-003 | WHEN a previously-folded file is reopened after its instance dropped, the system shall present it with zero active folds (the fold projection over the fresh instance is the identity/unfolded projection). | Unit test: fold → close last view → reopen → `fold_count == 0`. |
| REQ-004 | WHILE a distinct same-path editor instance survives (alias-root workspaces, #319 D3), WHEN another instance of that path fully drops, the system shall retain the shared `editor_folds` entry. | content.rs census-helper unit truth table + headless two-alias-root-project drive: fold, drop one instance → survivor's fold count unchanged. Kills the guard→always-clear mutant. |

## Phase Plan
- **P2 Design** — settle D3 (read `content_registry.rs`'s census/iteration
  surface) and D4 (wrapper vs return-paths; enumerate all call sites incl.
  headless_drive); file manifest; regression test plan (per-path clear, twin
  retention, reopen-unfolded, alias-root arm); confirm the §20 match line.
- **P3 Implement** — the chosen wiring; scrub at the instance-drop arm; update
  the `editor_folds` decl doc-comment (the #319 finding-6 note evolves per the
  D3 arm); all call sites migrated.
- **P3.5 Inspect** — independent critics vs the diff (twin/alias edge cases,
  leak-fixed-not-moved, provenance); fix the real findings.
- **P4 Validate** — write + RUN the planned tests; `scripts/gates.sh --diff`
  green.
- **P5 Complete** — CHANGELOG + arch docs (§21), ledger appends (§19: the
  rename-feature prevention note, lessons), archive the pair, close the ticket.
