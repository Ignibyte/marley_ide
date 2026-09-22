# TICKET-352 — Fold projection for the LSP overlay-card geometry (thread `visible_row` through the frame-geom layer — #305 Slice 2)

- **Forge:** #352 `8f3852b9-de73-48bc-988e-20a3799ebb23` (sprint "M28 — The Registry Payoff" `6558258f` — the train-independent item)
- **Type:** feature (bug-flavored — a #305 DOCUMENTED known-limit cashed in, kept as filed)
- **Milestone:** M28 (train-independent carry; original area M19/#305)
- **Status:** closed (done 2026-08-04 — shipped in M28; AAR `ee294a3d` submitted)
- **Depends:** #305 (SHIPPED — `FoldProjection` + the fold state, completed M19). Train-independent: NO
  #388/#394 ContentId-registry coupling — the sprint's parallel-safe lane.
- **React-first:** N/A — no UI delta: the POC has no fold engine (verified 2026-08-04 — the only "fold"
  in `marley-web/artifacts/marley-ide/src` is find-bar CASE folding, `findFold`), so the fold-above-caret
  scenario cannot exist there; the fix is Rust-side geometry correction verified via headless drives +
  live capture. The overlay cards themselves are already mirrored in the POC (`EditorView.tsx` hover
  card :317-319, rename draft :220) and their unfolded geometry is untouched (no-fold byte-identity is
  an AC).
- **Pipeline:** active — `docs/planning/pipeline/active/352-fold-overlay-projection.spec.md` (Phase 1 PASS; in Phase 2 — Design)

## Summary
#305 shipped THE buffer-row↔visible-row projection (`FoldProjection`: `slot_of`/`buffer_row`) and
converted the rim + scroll + sticky crossings — leaving ONE documented known-limit: the frame-geometry
fan-out where a BUFFER row is mixed with the slot-recorded `editor_geom.first/last`. With a fold above
the caret on-screen the row-anchored overlay cards mis-position (wrong Y — a bounded visual
imperfection, not a correctness break). The 2026-07-18 ticket text names ~7 sites at stale line
numbers; the 2026-08-04 re-sweep (HEAD `940286d`) verifies **NINE sites in app.rs, in four classes**:
(i) forward row→Y anchors — `hover_card_overlay` :13001 (:13021/:13031), `completion_popup_overlay`
:13076 (:13097/:13105), `signature_overlay` :13171 (:13182/:13189), `rename_draft_overlay` :13234
(:13246/:13253), plus the sweep-missed IME caret rect `bounds_for_range` :14901 (:14917/:14925 — #267,
predates the sweep); (ii) inverse pixel→row — the hover-dwell `on_editor_mouse_move` :12940 (feeds the
LSP hover REQUEST row) and the missed IME `character_index_for_point` :14944; (iii) viewport windows —
the content-width probe in `code_view_body` :5852 and the inlay-hints fetch window in
`refresh_inlay_hints` :4502/:4511 (named un-projected by the #305 inspect, absent from the ticket
text); (iv) none elsewhere — every other overlay (#312 def-picker, #317 references, #323 code-action,
goto/symbols/search/problems) is centered/window-anchored, verified. Today each forward site
copy-pastes `(row - geom.first) * cell_h` with NO shared helper. The fix: extract ONE shared projection
helper beside `FoldProjection` (reusing the shipped `slot_of`/`buffer_row` — no second projection),
then thread it through the verified sites so each computes `(slot_of(buffer_row) - geom.first) *
cell_h` (`geom.first` stays a slot). Small/mechanical once the helper exists; the RISK is site-set
completeness — Phase 2 re-sweeps.

## Out (separate tickets / later)
The #318 shared overlay-card RECIPE extraction (style/chrome — the sibling refactor, not absorbed);
any fold-behavior change; sticky-header interactions beyond what #305 shipped; the #333/#351 lexer /
ungated-parse gaps; the #305 W-2 fold-state-leak follow-up.

## Headline acceptance
With a fold above the caret, every row-anchored card computes its Y from the projected slot (headless
drive per card); with no folds, every touched site is byte-identical to today (drives + identity
units); an anchor row inside a collapsed fold renders at the fold header's slot; the extracted helper
is a pure seam at cov/MSI 100 (`BF-lsp-hover-extracted-helper-new-mutation-surface-001`); a live
capture shows a card correctly positioned below a fold. Full EARS in the pipeline spec.
