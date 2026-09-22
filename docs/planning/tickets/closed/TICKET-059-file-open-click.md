# TICKET-059 — click a file-tree row to open it

- **Forge ticket:** #59 `f35cfb9c-c034-4510-8178-500f7991e975` (feature, M2.B seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `2bffe609-9c53-416a-afbe-d4b1de755aa1`
- **Pipeline doc:** ../../pipeline/active/file-open-click.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
Completes #56: clicking a FILE row in the Files dock inserts its path at the prompt. PURE
`FileTree::path_at(index) -> Option<PathBuf>` (the row's full relative path, mirroring visit/toggle
traversal); SHIM attaches the click to file rows → write the path to the PTY (dirs still toggle).
cov/MSI 100 on path_at; the click is masked + self-test-verified. Deps #56.

## Acceptance
path_at at cov/MSI 100 (nested full path; root; out-of-range); a file-row click puts the path at the
prompt (self-test capture); FULL gate GREEN. Full EARS in the pipeline spec.
