# TICKET-205 — session persistence: restore each terminal pane's cwd on relaunch

- **Forge ticket:** #205 (85a7388c-4d30-427c-a1b6-163ffb97edfb) (feature, M12.2)
- **Owner:** autonomous (/work 195-222)
- **AAR:** 592d2ba3-0e6b-461f-b088-c4769c8f3726
- **Pipeline doc:** ../../pipeline/active/warp-session-cwd.spec.md
- **Source ticket:** sprint #25 (f3ecc094) — M12.2 persistence
- **Status:** closed

## Summary
Restore each terminal pane's working directory on relaunch. #163 already persists the whole shell
(projects, tabs, the split tree, kinds, actives — relaunch-proven); the one gap is per-pane cwd — a
terminal is serialized as just `"t"`, so restored terminals respawn in the project ROOT, not their
previous directory. Extend the pure grid-blob codec (`serialize_grid`/`restore_grid`) to carry an
optional framing-safe cwd (`"t"` → `"t=<cwd>"`, back-compatible with old blobs), and have the shim
capture each terminal's live `current_prompt().pwd` and spawn the restored terminal in that cwd when it
still exists on disk (else fall back to the project root).

## Acceptance
Open two terminals in different directories (`cd` one to `/tmp`), relaunch → each restores in its
directory; a persisted cwd that no longer exists → the project root. Full EARS criteria (REQ-001…005)
live in the pipeline spec.
