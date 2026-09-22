# TICKET-140 — live synthetic-input self-test (unblock click/key verification) [M8 seq-4]

- **Forge ticket:** #140 `bccf4e3a-626f-47ee-898b-6905a7c58abb` (chore, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `ebc5bdc0-ff91-440a-9fcc-7d32102eaac4`
- **Pipeline doc:** ../../pipeline/active/selftest-input.spec.md
- **Status:** closed

## Summary
Synthetic input already works (`AX_TRUSTED`; a driven "+" click spawned terminal 2). Add an
`AXIsProcessTrusted` preflight to `drive.swift` (fail loud on a fresh machine, not a silent no-op) + document
the click/key recipes in the README. Scripts + docs only (`--fast` gate). Unblocks #144. Deps M7.

## Acceptance
`drive.swift check` → `AX_TRUSTED`; a driven "+" click → terminal 2 (live capture); the preflight fails loud
without the grant (code-reviewed); `--fast` gate GREEN.
