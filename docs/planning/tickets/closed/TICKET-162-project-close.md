# TICKET-162 — M10: project close (a rail × per project row)

- **Forge ticket:** #162 `a1b62026-cb96-4d82-8ebb-56b8de86600f` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `3af93b17-5174-4795-95b6-b91e1c26d0f7`
- **Pipeline doc:** ../../pipeline/active/project-close.spec.md
- **Status:** closed

## Summary
An × per rail Project row closes that project (Workspace::close_project — refuses the last with a flash);
the whole removed Project (all tabs' PTYs) drops on a thread; sync_active_project + persist after. Mirrors
#161's × pattern. Shim-only (the algebra is cov/MSI 100 from #150). Deps #156, #161.

## Acceptance
Driven — × the only project → refusal flash; (picker-permitting) × a 2nd project → the subtree vanishes +
Files/branch resync; FULL gate GREEN. Full EARS in the spec.
