# TICKET-443 — Land the ported Marley tree behind a green gate

- **Ticket:** LOCAL #443 (chore, workbench shell baseline)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/443-land-the-port.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md ("Before #436: the baseline commit")
- **Status:** closed

## Summary
The 2026-09-18 port of the Marley crates, workflow and design record into the fork was never
committed, so every `--diff` gate would mutate all five Marley crates again and no ticket of the
workbench-shell sprint could commit. The first FULL run on 2026-09-22 found three things between
the tree and a green gate. First, 218 of the 408 archived gpui-era specs predate the §20
reference rule. Second, `marley_mcp/src/transport.rs` lost its `mutants::skip` masks in the port,
so its mutants survive under the fork's no-exclusion mutation gate. Third, the FULL mutation
workers share `CARGO_TARGET_DIR` and overwrite each other's test binaries, so some verdicts
describe the wrong mutant. This ticket fixes all three and lands the tree in one commit on
`marley/workbench-shell`.

## Acceptance
`script/gates.sh --diff` ends `GATE GREEN [diff]` over the whole ported tree, the transport's
behavior is proven over a real loopback socket, and the baseline commit passes every commit
hook. Full EARS in the pipeline spec.
