# TICKET-046 — re-run a Block's command (click / cmd-R)

- **Forge ticket:** #46 `836fc74c-40e2-40f6-b85b-2a3620fdba53` (feature, M1.G — Block Workflows & Selection, seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `e7f1b647-55f0-41af-80d6-b7c204306916`
- **Pipeline doc:** ../../pipeline/active/rerun-block.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
Re-run a previous command from its Block (a ↻ action / cmd-R) without retyping. Pure
`Block::rerun_command() -> Option<String>` (Some(command) iff Finished + non-empty; else None) in
terminal_blocks (cov/MSI 100); the header ↻ affordance + the click (guarded by #40's is_command_running
so it never injects mid-command, resending via write_command) are the app shim. Deps #45 (header) + #40.

## Acceptance
`Block::rerun_command` at cov 100/MSI 100 (Finished+non-empty → Some; Running → None; empty → None);
the click-rerun → new block (masked visual — chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
