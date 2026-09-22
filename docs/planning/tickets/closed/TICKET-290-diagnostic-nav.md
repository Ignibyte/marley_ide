# TICKET-290 — next/prev diagnostic navigation (cycle a failed command's error locations)

- **Forge ticket:** #290 `af6d6c64-5811-4ecc-ba98-8603cb2654e5` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `4799d92a-ca36-4108-9fcd-e2fb23c35b16`
- **Pipeline doc:** ../../pipeline/active/290-diagnostic-nav.spec.md
- **Status:** closed

## Summary
Walk the caret through the #289 diagnostic rows. F8 jumps to the next error
location (⇧F8 the previous), in source order, wrapping at the ends, moving the
caret to that row and scroll-following (reuse #270). Pure `next_diagnostic` /
`prev_diagnostic` over the sorted rows + the caret's current row; the
Editor-context keymap bindings + a `dispatch_action` arm are the shim.

## Acceptance
F8 moves the caret to the next diagnostic row (⇧F8 the previous), wrapping; with
no diagnostics it is a no-op. Full EARS in the pipeline spec.
