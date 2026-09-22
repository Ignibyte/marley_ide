# TICKET-097 — the code-doc model + viewer overlay

- **Forge ticket:** #97 `14d9c7b3-0025-4b0d-9919-678f6594fa6d` (feature, M4 seq-1 FOUNDATION; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `7e428549-7220-4cf4-9360-23753befef6c`
- **Pipeline doc:** ../../pipeline/active/code-doc-viewer.spec.md
- **Status:** closed

## Summary
The code-doc model + a read-only viewer overlay. PURE `code_view.rs`: `CodeLine`/`code_lines`/
`CodeViewState` (cov/MSI 100); the shim renders a numbered-line overlay when `code_view` is Some (Esc
closes). Opening + live proof come at seq-2. Deps #57 + #32 + #55.

## Acceptance
code_lines + CodeViewState::new at cov/MSI 100 (split/number/tab-stop/truncate/empty); the overlay renders
(masked, live at seq-2); FULL gate GREEN. Full EARS in the spec.
