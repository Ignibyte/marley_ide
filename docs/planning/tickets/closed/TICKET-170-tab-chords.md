# TICKET-170 — M10: tab chords (⌘T / ⌘[ / ⌘1–⌘9)

- **Forge ticket:** #170 `0cb3582e-7424-47a2-8c84-fd6246efa639` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `04f3bfca-cde2-4b88-bf97-be78e601eeab`
- **Pipeline doc:** ../../pipeline/active/tab-chords.spec.md
- **Status:** closed

## Summary
⌘T new terminal tab; ⌘[ prev-tab (pure prev_index, wrapping); ⌘1–⌘9 jump-to-tab (guarded no-op past the
end). Collision-checked. Deps #151.

## Acceptance
prev_index cov/MSI 100; the binding tests; driven — ⌘T adds terminal 2, ⌘[ cycles back, ⌘2 jumps; gate GREEN.
