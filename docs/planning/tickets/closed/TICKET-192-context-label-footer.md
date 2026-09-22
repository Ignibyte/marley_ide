# TICKET-192 — Move the cwd/branch context label to the footer

- **Forge ticket:** #192 (fc082ebd-f80d-4a47-9178-a14993d6bfb9) (feature, M12.1)
- **Owner:** claude (session c104f25c)
- **AAR:** 52e851c9-c9ac-4da7-b1fe-5829c8d2a9a0
- **Pipeline doc:** ../../pipeline/active/context-label-footer.spec.md
- **Source ticket:** M12.1 sprint #24 — chad live-app feedback #1
- **Status:** closed

## Summary
The "~/…/Marley · main" cwd+branch label (#142) is top-right in the titlebar; chad wants it at the bottom. Move
the render into the always-on footer (#94), right-aligned via a `flex_1` spacer. `titlebar_context_label` /
`titlebar_label` are unchanged — placement only.

## Acceptance
cwd/branch shows in the footer (bottom-right), not the top-right titlebar. Full EARS in the pipeline spec.
