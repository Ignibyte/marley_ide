# TICKET-382 — Status bar reads "focus: terminal" while an editor split holds focus

- **Forge ticket:** #382 (d145a213-f2e9-43c2-b6e0-79836bfc87e0) (bug, M25)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/382-status-bar-focus-label.spec.md
- **Source ticket:** sprint #36 "M25 — App-Grade QA Hardening"; found in the 2026-07-21 live QA run (capture 14-editor.png)
- **Status:** closed

## Summary
Clicking into a `workspace.rs` editor split inside a terminal tab's grid moves the #191 accent
focus border to the editor pane (correct) and flips the rail to the pane-2 row (correct), but the
footer keeps reading `focus: terminal`. The verified mechanism: the render shim's `footer_focus`
(app.rs:18408-18414) resolves the focused `PaneId` against the agents map (→ the run's label), then
the remotes map (→ the host), then falls to a HARDCODED `"terminal"` literal — it never consults
the focused pane's kind (`PaneState::kind`, workspace.rs:361-368) or the active tab's content
(`TabContent`, tabs.rs:23-31). So a focused editable code pane (`PaneContent::CodeView`, the
#259/#355/#357 editable-split lineage), a files/git panel, an editor TAB, and a cockpit TAB all
read `focus: terminal` — and a non-terminal tab even reads a BACKGROUND grid (`workspace()` falls
back to the first terminal grid, app.rs:5288-5294), which can leak a background agent's label.
Fix: derive the label from the active tab's kind and the focused pane's kind via a new pure fn in
the already-pure `status_bar.rs`, keeping the agent/remote arms byte-identical — the one-glance
answer to "where will my keys go", matching what the border shows. The related tab-title
observation ("Marley" → "terminal 1") is the #201 tiering working as designed and is scoped OUT.

## Acceptance
Headline: with an editor split focused inside a terminal tab, the footer reads `focus: editor` in
the same frame the accent border sits on that pane; a plain terminal pane still reads
`focus: terminal`; agent/remote labels are unchanged; files/git panes and editor/cockpit tabs each
read their own kind. Exact-string units on the pure seam (cov/MSI 100) + a driven capture of the
QA repro. Full EARS criteria live in the pipeline spec.
