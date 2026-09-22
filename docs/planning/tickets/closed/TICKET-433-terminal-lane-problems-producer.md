# TICKET-433 — The terminal-lane problems producer

- **Ticket:** LOCAL #433 (feature, M33)
- **Tags:** fusion, problems, terminal-blocks, diagnostics
- **Created:** 2026-08-15
- **Provenance:** the #310 two-producer doctrine's standing empty seam
  (`problem_rows(lsp, terminal: &[], cap)`), deferred again at #327 and
  #430 (D-TERMINAL); shelf: ../../design-notes/m33-tail-and-wedge-shelf.md
- **Pipeline doc:** ../../pipeline/completed/433-terminal-lane-problems-producer.spec.md
- **Status:** closed (2026-08-15 — the #310 seam filled; GATE GREEN [diff] 15/15)

## Summary
Failed command Blocks finally feed the workspace problems aggregation: the
#289 per-block file:line ref fold (already scanning failed output for the
editor gutter) goes WORKSPACE-wide (every project's every terminal grid —
the PR-1345 cross-tab iterator rule) and its refs enter `problem_rows`'
`terminal` parameter as `ProblemSource::Terminal` rows. The ⌘⇧M panel and
the #430 diagnostics multibuffer then show build failures even with no LSP
running — the first Phase-C fusion thread through the panel.

## Acceptance
Headline: a failed `cargo build` block in ANY terminal tab yields rows in
⌘⇧M (path:line, terminal-sourced glyph), jumpable; rows clear when the
block is superseded by a passing rerun. Full EARS in the queued spec.
