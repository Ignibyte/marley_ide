---
pipeline_id: f2e24ac0-dfa3-4cf8-839e-98ce0c0288cb
ticket: forge#355 (9ef38bb2-3b70-4c56-b268-1e3288e74a2a) · local docs/planning/tickets/open/TICKET-355-split-pane-slice2-parity.md
aar_id: 09df44b8-981a-4eab-ad46-38df45ef0f4c
status: Phase 5 — Complete PASS
title: Verify + guard editor-feature parity in a focused editable split pane (slice 2)
type: feature
milestone: M22
references:
  - crates/marley_app/src/headless_drive.rs
  - crates/marley_app/src/app.rs
---

## Title
#259 shipped the core editable split pane; the recon named "slice 2" (find/fold/⌘⇧O + the ~25 LSP/nav caret
features) as a follow-up "scoped tab-side for v1 (D-CORE-EDITING-ONLY-V1)". **Investigation (this pipeline)
found that scoping is SUPERSEDED — the parity is ALREADY DELIVERED** by the composition below, so #355 is a
VERIFICATION ticket: prove the slice-2 features operate on a FOCUSED split pane, fix any residual gap, document.

**Why parity already holds (each verified against the code):**
- **#259** made `active_editor()`/`active_editor_mut()` FOCUS-AWARE (they resolve the focused split pane's
  surface) and removed EVERY `active_tab().editor()` chain (its F1; "0 left outside the accessor"). `editor_geom`
  is written by the FOCUSED surface (the pane's `editor_draw_for`), so caret-anchored overlays position on it.
- **The slice-2 overlays are TOP-LEVEL + focus-aware** (rendered after the split-pane loop, not tab-gated):
  `hover_card_overlay`/`completion_popup_overlay`/`signature_overlay`/`rename_draft_overlay` read
  `self.active_editor()` + `self.editor_geom`; `def_picker_overlay`/`references_overlay`/`code_action_overlay`/
  `goto_overlay`/the ⌘⇧O + ⌘T pickers are state-driven overlays (populated by focus-aware handlers); the ⌘F
  `efind` bar reads `efind_open` + `editor_geom`; `fold_projection` reads `active_editor()`.
- **#357** wired the ONE remaining tab-block-gated overlay (the #275 conflict banner) into the pane.
- **#356** keeps an unfocused pane's read-only lines live.

## Scope
### In
- `crates/marley_app/src/headless_drive.rs` — headless DRIVE tests proving representative slice-2 features work
  on a FOCUSED split pane (the mechanism is shared, so a representative set proves the class):
  - **⌘⇧O go-to-symbol-in-file** in a pane → the picker populates from the PANE's file + Enter jumps the PANE's
    caret.
  - **⌘F find** in a pane → `efind_matches` reflects matches in the PANE's buffer.
  - **⌥⌘[ fold** in a pane → the fold applies to the PANE's region (the projection hides rows).
- Any residual gap the drives surface → fixed at the source.
- Docs: record that slice-2 parity is delivered by #259/#356/#357 + the top-level focus-aware overlays.

### Out (explicitly deferred)
- The LSP-SERVER-dependent features (hover / completion / go-to-def / find-references / rename / signature) —
  headless-driving them needs a live language server (`fake_ls` is #320, not wired). They use the IDENTICAL
  focus-aware mechanism (`active_editor()` + `editor_geom`) the driven features prove, so they are covered by
  that mechanism proof + the per-fn code confirmation, not a separate LS drive.
- Adding NEW editor features — parity of the EXISTING set only.

## Reference (§20)
N/A — Marley-specific. The editable split pane (a focused code pane inside a terminal tab, #259) is Marley's own
surface; no Warp/Zed analog. No copyleft source consulted.

### Prior art
1. **OUR own code (the whole point):** #259's focus-aware `active_editor()`/`active_editor_mut()` + `editor_geom`,
   the top-level focus-aware overlays, #357's banner, #356's read-only sync ALREADY compose to deliver slice-2
   parity. The `split_file_pane_for_test` harness (headless_drive.rs #259) + the existing per-feature headless
   tests (`go_to_file_symbol_jumps_and_pushes_navstack_headless`, the find drives) are the reusable drive
   patterns. The fix is a VERIFICATION, not new feature logic.
2. **Behavior maps / published:** n/a.

## Locked-In Decisions
- **D1 — verify, don't re-port.** The architecture already delivers parity; the deliverable is the drive tests
  that PROVE + GUARD it (regression net for the "features follow focus into the pane" contract), plus docs.
- **D2 — representative subset proves the class.** All slice-2 features share the focus-aware
  `active_editor()`/`editor_geom` mechanism; driving ⌘⇧O + ⌘F + fold in a pane proves it; the LS-dependent
  features are code-confirmed to use the same mechanism (D-out).
- **D3 — a failing drive is a real gap → fix at the source** (then the fix is the code deliverable). If all
  pass, parity is confirmed with no production change (the tests are the deliverable).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘⇧O is invoked on a FOCUSED editable split pane, the go-to-symbol picker shall populate from THAT pane's file and Enter shall move THAT pane's caret. | headless drive (split → focus → ⌘⇧O → assert picker rows from the pane's file + caret jump) |
| REQ-002 | WHEN ⌘F + a query is entered on a FOCUSED editable split pane, `efind_matches` shall reflect matches in THAT pane's buffer. | headless drive (split → focus → ⌘F + type → assert `efind_matches`) |
| REQ-003 | WHEN ⌥⌘[ is invoked on a FOCUSED editable split pane at a foldable region, the fold shall apply to THAT pane (its visible rows shrink). | headless drive (split → focus → ⌥⌘[ → assert the pane's fold projection) |

## Testing boundary (honest)
The features run through `active_editor()`-aware handlers in app.rs (`mutants::skip` / coverage-excluded), so
the headless DRIVES are the proof (the #307 pattern — the drive is the proof; no new production seam unless a
gap is found). The LS-dependent slice-2 features are covered by the shared mechanism + per-fn code confirmation
(D-out), not a live-LS drive (`fake_ls` = #320).

## Phase Plan
- **P2 Design** — the exact drive tests (extend `split_file_pane_for_test`); observables (`open_file_symbols`,
  `efind_matches`, the fold projection / caret).
- **P3 Implement** — write the drives; fix any gap they surface.
- **P3.5 Inspect** — adversarial: do the drives ACTUALLY exercise the pane (not the editor tab)? are the
  assertions pane-specific? any slice-2 feature genuinely still tab-gated the recon-scan missed?
- **P4 Validate** — run the drives; gate green.
- **P5 Complete** — document parity; archive; AAR; close #355.
