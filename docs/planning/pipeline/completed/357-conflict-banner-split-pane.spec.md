---
pipeline_id: fbb34c40-99ef-4520-a4c7-c11b3f9cad7b
ticket: forge#357 (02cbc091-eebf-4ca2-a73d-8b58fbaee5a6) · local docs/planning/tickets/open/TICKET-357-conflict-banner-split-pane.md
aar_id: 43fac3b4-aa2d-4df2-ade9-f753fcebb936
status: Phase 5 — Complete PASS
title: Render the #275 external-change conflict banner on a focused editable split pane
type: feature
milestone: M22
references:
  - crates/marley_app/src/app.rs
---

## Title
#259 (inspect F3): `check_active_file_external` is focus-aware, so an external change/delete of a FOCUSED split
pane's file arms an `ExtConflict` on the pane's surface — but the #275 Keep-mine/Reload banner renders ONLY
inside the editor-TAB block (app.rs:15621, over the tab's `center_bounds`). A focused split pane in a terminal
tab shows the dirty ● but no banner (no data loss — `save_active` re-checks/arms/flashes before writing; the UX
surface is just incomplete). Fix: extract the inline banner into `ext_conflict_banner` and render it on the
FOCUSED split pane too.

## Scope
### In
- `crates/marley_app/src/app.rs` (render shim, `mutants::skip`):
  - Extract the inline editor-tab banner (app.rs:15621–15690) into
    `fn ext_conflict_banner(&self, colors: &ThemeColors, cx: &mut Context<Self>) -> Option<gpui::Div>` — reads
    `self.active_editor().and_then(|s| s.active_conflict())`, builds the `.absolute` top-right #221 card
    (Changed → "File changed on disk" + Keep-mine + Reload; Deleted → "File removed…" + Dismiss), buttons call
    the unchanged `ext_keep_mine`/`ext_reload`/`ext_dismiss_deleted`. `.absolute` top-right = relative to its
    PARENT, so each render site positions it in its own area with no coord change.
  - Editor-tab block: replace the inline build with `let banner = self.ext_conflict_banner(&colors, cx);`.
  - Split-pane render: on the FOCUSED editable pane, add the banner to the pane body (`active_editor()` = the
    focused surface, so the banner reads THIS pane's conflict and its buttons act on it). Gated on
    `workspace().focused() == pane_id` so it renders once, on the correct pane.

### Out (explicitly deferred)
- The conflict machinery (`check_active_file_external`, `save_active` arm/overwrite, `ext_*` handlers) — unchanged.
- An UNFOCUSED pane's banner — a conflict is only armed on the focused surface (the check is focus-aware) and
  the `ext_*` buttons act on `active_editor()`, so the banner correctly renders only on the focused pane.

## Reference (§20)
N/A — Marley-specific. The external-change Keep-mine/Reload banner is Marley's own #275/#284 surface (a
no-silent-clobber UX); no Warp/Zed analog. No copyleft source consulted.

### Prior art
1. **OUR own code (highest-yield):** the banner + the `ext_keep_mine`/`ext_reload`/`ext_dismiss_deleted` handlers
   already exist (#275/#284, app.rs:15621 / 6882 / 6915). `active_editor()` is already focus-aware (#259), so it
   ALREADY resolves the focused split pane's surface — the banner content + button targets are correct for a
   split pane the moment it is rendered there. The fix is a pure render RELOCATION (extract + reuse), no new logic.
2. **Behavior maps / published:** n/a.

## Locked-In Decisions
- **D1 — extract + reuse, don't duplicate.** One `ext_conflict_banner` builder, called by both the editor tab
  and the focused split pane; the banner's `.absolute` coords are parent-relative, so no per-site positioning code.
- **D2 — focused pane only.** Gated on `workspace().focused() == pane_id`; a conflict can only be armed on the
  focused surface and the buttons act on `active_editor()`, so the focused pane is the only correct host.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a focused editable split pane's surface has an armed `ExtConflict`, the split-pane render shall include the #275 banner (whose buttons drive `ext_keep_mine`/`ext_reload`/`ext_dismiss_deleted`). | headless drive: split → focus → arm a conflict → render (no panic) + drive `ext_reload`/`ext_keep_mine` on the pane's surface (they clear the conflict) |
| REQ-002 | The editor-TAB banner shall be unchanged (same card, same buttons) after the extraction. | existing #275 editor-tab tests stay green; the extracted fn is byte-equivalent |

## Testing boundary (honest)
Pure render shim (`ext_conflict_banner` + the two call sites are `mutants::skip`; app.rs coverage-excluded). No
new pure seam (the banner is gpui element construction; the DECISION is the trivial `match conflict`). REQ-001 is
a headless DRIVE (the #275/#198 pattern: the banner's buttons are the tested `ext_*` `pub(crate)` handlers, driven
directly); the extraction is behavior-preserving (editor-tab tests green).

## Phase Plan
- **P2 Design** — the `ext_conflict_banner` signature + the two call-site edits.
- **P3 Implement** — extract + wire both sites.
- **P3.5 Inspect** — adversarial: the extraction is byte-equivalent, focused-only gating, no double banner, the
  `.absolute` positioning still correct per parent, the ext_* handlers act on the right surface.
- **P4 Validate** — a headless split-pane conflict drive; the existing #275 tests green; gate.
- **P5 Complete** — archive, AAR, close #357.
