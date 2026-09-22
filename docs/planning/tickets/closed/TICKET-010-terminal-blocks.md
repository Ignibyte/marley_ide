---
ticket: TICKET-010
forge: forge#14 (4701d6d6-24a3-470c-ab77-56190f3bc301)
status: closed
type: feature
milestone: M1
sprint: M1.A — The Usable Terminal (seq 3/5)
branch: ticket-010-terminal-blocks
pipeline: docs/planning/pipeline/active/terminal-blocks.spec.md
spec: docs/specs/SPEC-terminal-blocks.spec.md
aar: b0e4c12f-407b-4d1d-9c45-62224314c22a
---

# TICKET-010 — marley_terminal (terminal_blocks)

Sprint M1.A seq 3/5 — the **product core**. The UI-agnostic PTY-backed shell **session** + per-command
**Block model** (segmented from shell DCS/OSC hooks, NOT heuristics): spawn a shell, write commands,
read output back into a `BlockList` any front-end observes. Adopts SPEC-terminal-blocks R1–R23. Promotes
marley_spike's proven PTY/grid/Block patterns into a real, reusable session. **UI-agnostic** —
visual_acceptance N/A; the render is `app_shell` (#16).

## Acceptance
- Pure surface (DCS codec `decode_hook`/`encoding_for_dcs_terminator`, the `apply_hook` state machine,
  `BlockList`, the `write_bytes` re-queue logic) — **cov 100 / MSI 100**.
- The raw PTY shim (`tty::new` spawn, leader-fd read/write, `TIOCSWINSZ` ioctl) — ACCEPTED-UNTESTABLE
  (mutants::skip + rust_cov exclude), proven by a headless real-PTY integration test (stub DCS shell).
- FULL `scripts/gates.sh` → `GATE GREEN` (gate-15 N/A); §21 CHANGELOG + arch doc.

## Notes
The brain-observation seam (AD-claude-brain-agent-session-supervision-001) rides on `blocks()`/the
BlockList now; the `local_control` `session.read` RPC tap lands at M3. See the pipeline spec/notes for
the full plan + the spike promotion + the mutation map.
