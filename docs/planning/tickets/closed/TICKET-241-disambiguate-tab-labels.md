# TICKET-241 — Disambiguate identical idle-terminal tab labels in the rail

- **Forge ticket:** #241 (71ad57c2-5b9f-4d86-a26c-5f1d4d5841ef) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** c3d175c1-4439-45ba-91d1-117a9b292d69
- **Pipeline doc:** ../../pipeline/completed/disambiguate-tab-labels.spec.md
- **Source:** chad live feedback (2026-07-10, alongside issue #2) — the M14 round; 4th ticket.
- **Status:** closed

## Summary
Idle terminals fall back to the cwd basename (`live_tab_title`), so N terminals in the same directory all read
"Marley" in the rail. Add a pure `disambiguate_labels` helper (append " 2"/" 3" to the 2nd+ occurrence of a
duplicate, first bare) + a shim pre-pass that computes per-project disambiguated labels over the FULL rail list
(scroll-safe) which the rail Tab arm + the "Search tabs" filter look up. Cockpit/editor tabs are already unique.

## Acceptance
`disambiguate_labels` suffixes duplicates from the 2nd occurrence (unit, cov/MSI 100); the rail shows distinct
labels for same-cwd terminals ("Marley"/"Marley 2"/"Marley 3", driven); disambiguation is per-project + scroll-
safe (full-list pre-pass). Full EARS in the spec.
