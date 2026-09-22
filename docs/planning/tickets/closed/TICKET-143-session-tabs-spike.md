# TICKET-143 — SPIKE: terminal session tabs vs the sidebar [M8 seq-7]

- **Forge ticket:** #143 `0b72dbcb-5461-41b7-bf4b-8f27fea2a58f` (spike, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `912f9291-5c4f-4fea-82b2-e31f0393766d`
- **Deliverable:** docs/planning/design-notes/session-tabs-vs-sidebar.md
- **Status:** closed

## Outcome
Compared Marley's model to chad's **actual Warp** (3 reference screenshots). Finding: **Warp uses a left
sidebar session-list (grouped by project) + tiled panes with per-pane title bars — exactly Marley's current
model; Warp has NO horizontal top-tab strip.** Recommendation: **do not add a horizontal tab strip** (it would
diverge from Warp + duplicate the sidebar). Instead polish the sidebar to behave like Warp's session tabs.

## Follow-ups created
- forge #145 (F1) — sidebar click-to-focus + active highlight.
- forge #146 (F2) — sidebar live title (running command / cwd).
- forge #147 (F3) — sidebar real branch (drop hardcoded "main").
- forge #148 (F4, larger/own milestone) — multi-project workspaces.
