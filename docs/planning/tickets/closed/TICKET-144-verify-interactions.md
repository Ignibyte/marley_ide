# TICKET-144 — live-verify the shipped interactions (close/drag/nav/clicks) [M8 seq-8]

- **Forge ticket:** #144 `cf5fab0c-3662-40d7-8e3c-44770034a375` (chore, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `98b3f7a0-ae7d-4088-b4bb-f2bd0ac52965`
- **Pipeline doc:** ../../pipeline/active/verify-interactions.spec.md
- **Status:** closed

## Summary
Drive-verify the shipped interactions now that #140 unblocked synthetic input. Found a real bug: #131 pane
focus-nav is bound to ⌘⇧-arrow, not the documented ⌘⌥-arrow (a shift/alt param swap in keymap.rs). Fix the
4 bindings + tests; drive-verify the rest (close-×, 📁, 🧠, cockpit, drag). Harness: added drive.swift
`drag:`/`cmdopt:`. Deps #140 + M6/M7. Closes M8.

## Acceptance
⌘⌥→ moves focus (keymap unit + driven capture); the other gestures verified; FULL gate GREEN.
