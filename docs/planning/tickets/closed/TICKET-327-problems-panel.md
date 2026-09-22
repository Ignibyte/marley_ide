# TICKET-327 — The problems panel (⌘⇧M): every diagnostic in the workspace, one jumpable list

- **Forge ticket:** #327 898479e5-952e-4e0a-b532-307a7f11398f (feature, M21)
- **Owner:** claude (this session)
- **AAR:** d1fda6ae-20d0-415b-861b-e41a18f6a222
- **Pipeline doc:** ../../pipeline/active/327-problems-panel.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
⌘⇧M opens a list of every error and warning rust-analyzer knows about across the WHOLE workspace — including
files you don't have open — grouped, severity-sorted, jumpable. F8 (#290/#310) walks the FOCUSED file; this
is the workspace view. Phase 1 is a READ-ONLY list (a diagnostics-multibuffer inherits the later gate, like
#326's search). Adds the missing `DiagnosticStore::iter()` enumeration primitive (pure), aggregates across
every per-root host, composes a pure `problem_rows` (merged with the M18 terminal failed-block lane, the #310
two-producer doctrine, sorted severity→path→line, capped + "+N more"), and a ⌘⇧M finder-recipe picker whose
Enter opens even a CLOSED file at the line (#312 open_and_place_caret + NavStack). Live re-derives on
publishDiagnostics with the selection kept by identity (path+line) not index; a `cockpit_status` workspace
tier. ⌘⇧M is a NEW Editor-scoped binding (shadows nothing).

## Acceptance
⌘⇧M opens a workspace picker listing two errors planted across two probe files (one CLOSED); Enter on the
closed-file row opens it at the squiggle (+ NavStack, ⌃- returns); fixing one + saving drops its row on the
next publish. Pure `DiagnosticStore::iter` + `problem_rows` at cov/MSI 100 (aggregation, the two-producer
merge, severity→path→line sort, caps + tail, the selection-by-identity decision); the footer workspace tier
formatter + its order tests. Full EARS in the pipeline spec.
