# TICKET-408 — Shell-codec restore drops tabs without reclamping active_tab — the index drifts

- **Forge ticket:** #408 767ae35d-8c5e-4608-896b-241a6f99996e (chore, M29)
- **Owner:** pipeline 54f7cc66-7020-4d9d-a641-d93a80bc6c48
- **AAR:** 4c2f4f92-88bc-4d9d-8558-47c424b43585
- **Pipeline doc:** ../../pipeline/active/408-restore-active-tab-reclamp.spec.md
- **Source ticket:** filed by the M29 #403 inspect critic (deliberately deferred there as out of scope)
- **Status:** closed

## Summary
Two pre-existing index-hygiene gaps in the shell codec, both about what happens when an
entry is DROPPED while its sibling index is kept verbatim. (1) The `restore_shell`
consumer in app.rs drops tabs it cannot rebuild — a `T=` whose PTY spawn fails, a `V=`
whose files are all unreadable — but never reclamps `active_tab` onto the survivors;
`pl.active_tab.min(last)` only bounds it, so the user comes back focused on a DIFFERENT
tab than they left. The codebase already owns the primitive (`reclamp_active`,
grid_layout.rs — shared by FILE-level drops in both directions since #243); it is never
applied at tab level. Plan re-verification found the SAME class one level up: project
drops (serialize-time framing-breaking root; restore-time vanished root) keep
`active_project` verbatim/bounded — in scope, same mechanism. (2) `serialize_shell`
writes the terminal grid `blob` verbatim with no framing guard — the only writer input
without one (`root` gets `breaks_framing`, paths get it + `\x1f`). Production-unreachable
today (the grid alphabet excludes the hazard bytes), so this is a robustness asymmetry: a
hazard-bearing blob could forge entry tags. Guard it at write time like every other input.

## Acceptance
After a degraded restore (any tab/project dropped), focus lands on the same content the
user left — mapped through the survivor set, never drifted onto a neighbour; a
framing-hazard blob can never reach the wire. Full EARS criteria in the pipeline spec.
