# TICKET-388 — SPIKE: the shared-handle pane model (sections navigate, panes compose)

- **Forge ticket:** #388 `4c8cbe75-f7c6-4d98-9893-130e22d922e1` (spike, M26)
- **Owner:** unclaimed (queued)
- **AAR:** — (opened at promotion)
- **Pipeline doc:** ../../pipeline/queued/388-spike-pane-handle-model.spec.md
- **Source:** chad's 2026-07-22 model-A lock: sections are a navigator; panes compose views of
  open things; "make a Pane" = save + name a multi-cell arrangement
- **Status:** closed

## Summary
Design (docs only — nothing ships) the shared-handle refactor that lets a pane cell hold ANY open
thing by reference while the original stays filed under its section. Today pane content is owned
inline (`PaneContent<S>` is a closed 4-kind enum owning values; a file open in tab + split =
two Buffers), so "one instance, many views" is impossible without a registry/handle model. The
spike settles the registry shape (generalizing the existing PaneId-keyed pattern; gpui `Entity`
studied for the render layer), the shared-buffer/per-view-caret semantics, the full migration
ripple with per-file sizing, nameable-arrangement persistence, and a sliced M27 ticket train.

## Acceptance
A design doc + forge ADs (handle model; one-instance-many-views invariant) + an ordered,
one-shippable-slice-each follow-up ticket list; zero shipped app code. Full EARS in the pipeline
spec.
