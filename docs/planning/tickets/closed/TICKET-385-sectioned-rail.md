# TICKET-385 — Type-sectioned rail: Editor / Terminal / Browser fixed-order sections

- **Forge ticket:** #385 `09006da6-d499-4035-9f47-046accb4fbcd` (feature, M26)
- **Owner:** unclaimed (queued)
- **AAR:** — (opened at promotion)
- **Pipeline doc:** ../../pipeline/queued/385-sectioned-rail.spec.md
- **Source:** chad's 2026-07-22 sectioned-shell direction (this conversation), extending the 2026-07-10 workspace-centric vision
- **Status:** closed

## Summary
The left rail's flat per-project tab list becomes three fixed-order type sections — **Editor**,
then **Terminal**, then **Browser** — and every tab files under the section matching its
`TabContent` kind (CodeView→Editor, Terminal→Terminal, Cockpit→Browser as the transitional
resident until the Phase-E webview). Display-only: `rail_rows` buckets; storage order, the
active-tab index, and the shell codec are untouched. Empty sections render their (muted) header
so the fixed skeleton always shows; split-pane rows keep nesting under their tab.

## Acceptance
Rail shows the three fixed-order section headers per project with each tab under its kind's
section; opening a thing files it under its section; storage/codec byte-identical (existing
persistence tests unchanged); empty sections keep their header; live capture proves the grouping.
Full EARS in the pipeline spec.
