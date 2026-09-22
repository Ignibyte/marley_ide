# TICKET-178 — M11: completion popup live-filter

- **Forge ticket:** #178 `16f5b5df-c67a-46bd-9072-2a0b159d224e` (feature; sprint #22)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `4c31d15e-e8be-40a1-96a6-c92e7e5d111d`
- **Pipeline doc:** ../../pipeline/active/completion-filter.spec.md
- **Status:** closed

## Summary
CompletionState.entries snapshot + refilter(word) (narrow via complete_word, clamp selected, keep-open ≥2);
the popup narrows on a printable + re-widens on backspace instead of dismissing. Deps #96, #172.
