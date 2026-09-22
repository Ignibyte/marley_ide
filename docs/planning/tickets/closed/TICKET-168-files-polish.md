# TICKET-168 — M10: Files panel polish (refresh on open + drag width)

- **Forge ticket:** #168 `c458c35e-ea56-472b-a6b7-cb311a51d560` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `8324936d-ab1d-45cf-8d61-f5fe5456674e`
- **Pipeline doc:** ../../pipeline/active/files-polish.spec.md
- **Status:** closed

## Summary
The Files tree refreshes on every open (new files appear) and the panel's right edge drags to a persisted
width (pure clamp 160–480 + a files_width setting mirroring right_section; the #130 drag pattern). Deps
#154, #130.

## Acceptance
Clamp + setting round-trip at cov/MSI 100; driven — a bash-created file appears after toggling open, the edge
drags (terminal shifts); gate GREEN.
