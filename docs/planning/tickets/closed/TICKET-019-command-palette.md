---
ticket: TICKET-019
forge: forge#19 (67473f5f-4aaf-40a4-9c9e-8bc7236f8ef1)
status: closed
type: feature
milestone: M1
sprint: M1.B — The Cockpit (seq 3/5)
branch: ticket-019-command-palette
pipeline: docs/planning/pipeline/active/command-palette.spec.md
spec: docs/specs/SPEC-app-shell.spec.md
aar: 05ba6945-9bcc-41a7-bcbf-b7c49674f49a
---

# TICKET-019 — command palette (fuzzy launcher overlay)

Sprint M1.B "The Cockpit" seq 3/5. The SPEC-app-shell command palette (R14-R18) in `marley_app`: a pure
`filter_commands` (nucleo subsequence ranking over a static `&[Command]` — empty→all in reg order;
non-empty→ranked, non-matches excluded, score=max field score, desc, ties by reg order) + the
`Command`/`CommandId`/`ScoredCommand` model, plus the centered overlay in `RootView` (open on the #18
keymap's `open-command-palette`; activate→dispatch+dismiss; Escape→dismiss). The fuzzy launcher.

## Acceptance
- `palette.rs` (the model + `filter_commands`) — **cov 100 / MSI 100** (R15 + R16; the 5 mutation
  targets: empty-short-circuit, match-predicate, max-not-min, sort-flip, tie-break).
- The app.rs overlay shim (state + render + activate/Escape dispatch) — ACCEPTED-UNTESTABLE (already
  excluded).
- `nucleo` dep; `marley_search_core` deferred to M2. FULL `scripts/gates.sh` → `GATE GREEN` + a headed
  palette assertion. §21: CHANGELOG + app_shell.md.
