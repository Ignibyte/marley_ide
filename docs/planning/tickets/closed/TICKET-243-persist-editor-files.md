# TICKET-243 — Persist all open editor files across restart

- **Forge ticket:** #243 (3c1e984e-e2f9-49c6-b890-c96f237f18d5) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** a456ff66-1cb3-424e-a711-1759826ad737
- **Pipeline doc:** ../../pipeline/completed/persist-editor-files.spec.md
- **Source:** the M14 round; 5th ticket. A #237 follow-on (independent of the #242 editable editor).
- **Status:** closed

## Summary
The editor surface persists only the ACTIVE file (`TabLayout::Code(path)` = code_view()), so the other file-tabs
vanish on restart. Extend `TabLayout::Code` to carry all open paths + the active index (wire
`V=<active>\x1f<path…>`, back-compat old single-path, framing-breaking paths dropped + active re-clamped). Save
maps `surface.files()`; restore reads each (keeping the #154 guards, skipping failures + re-clamping) and
rebuilds the one editor surface, activating the saved index.

## Acceptance
The pure code-tab codec round-trips N paths + active + back-compat + framing-drop (cov/MSI 100); a grid round-trip
test; driven — 3 files → quit → relaunch → all 3 restore with the saved active shown. Full EARS in the spec.
