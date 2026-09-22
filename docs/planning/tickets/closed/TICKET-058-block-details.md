# TICKET-058 — Details dock: focused-block inspector

- **Forge ticket:** #58 `cbbd4790-ca3e-453f-8ea3-adf4e80afce8` (feature, M2.A seq-6 — the LAST M2.A ticket)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `5c65c71b-8e99-43fb-82f4-2e5cfa4f9abd`
- **Pipeline doc:** ../../pipeline/active/block-details.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
The Right "Details" dock inspects the current command block. PURE `block_details(&Block) ->
BlockDetails` (command, status via exit_status_kind, exit code, pwd, git branch) in block_status.rs;
SHIM (app.rs) renders it for the active session's most-recent block (placeholder when none). cov/MSI
100 on the projection; the render is masked + self-test-verified. Deps: block model + #36.

## Acceptance
block_details at cov/MSI 100 (all 5 fields for finished-success / fail-code / running / no-pwd-branch);
the Details dock shows the command + status + cwd (self-test capture); FULL gate GREEN. Full EARS in the
pipeline spec.
