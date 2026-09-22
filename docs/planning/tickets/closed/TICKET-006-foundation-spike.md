---
ticket: TICKET-006
forge: forge#11 (a8b226f5-a62b-4f28-a637-2e0bd6a00220)
status: closed
type: spike
milestone: M0
branch: ticket-006-foundation-spike
pipeline: docs/planning/pipeline/active/marley-spike.spec.md
spec: docs/specs/SPEC-foundation-spike.spec.md
aar: eddc7ec8-b689-42c2-811e-1dc9f35365c3
---

# TICKET-006 — marley_spike (foundation-spike)

The **throwaway** gpui + alacritty_terminal + vte viability spike (SPEC-foundation-spike R1–R16):
one window "Marley Spike" 800×600, a `/bin/sh -c "echo hello-marley"` PTY at 24×80, one rendered
command Block (header above body). Proves the REUSE stack composes; deleted once the M0 gate is green.

Built ON marley_visual_harness (forge #10) — its visual_acceptance (R2/R9/R10) is asserted via that
harness (a `#[ignore]` headed test spawns the marley_spike bin + asserts the AX window + the one-Block
screenshot baseline). The PTY spawn/read/grid (R3/R4/R5) is a HEADLESS integration test.

## Acceptance
- Pure logic (build_block, BlockStatus, classify_read, exit_code_to_status, exit_code_for, the grid-row
  extractor) — **100% coverage + MSI 100**, headless.
- The display/integration shim (run()/gpui render/raw PTY spawn) — ACCEPTED-UNTESTABLE (mutants::skip +
  rust_cov exclude, the harness precedent).
- FULL `scripts/gates.sh` → `GATE GREEN [full]` (15 gates; gate-15 now ENFORCES).
- §21: CHANGELOG + `docs/marley_architecture/marley_spike.md`.

## Design spike (Phase 2, FIRST)
The alacritty_terminal 0.26 PTY API — headless spawn + leader-fd read + vte→grid + row extract.
See the pipeline spec + notes for the full plan + the locked decisions + the carry-to-design.
