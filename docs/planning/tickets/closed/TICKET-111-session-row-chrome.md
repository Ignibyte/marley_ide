# TICKET-111 — session row chrome (icon/title/subtitle) [M5 seq-5]

- **Forge ticket:** #111 `f10d1452-200b-4efb-9480-01cf31197816` (feature, M5 seq-5; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `3e93a25f-f0a2-4a43-b665-7a29ae662e59`
- **Pipeline doc:** ../../pipeline/active/session-row-chrome.spec.md
- **Status:** closed

## Summary
`session_row_icon(is_agent, running)` (3 glyphs: active ✳ / idle ✧ / terminal ▸) + `Session.running`;
`session_rows` uses it (cov/MSI 100). The sidebar row becomes two-line (title + muted subtitle);
`running` from the real AgentStatus. Deps seq-3/4 + #67/#79.

## Acceptance
session_row_icon + session_rows at cov/MSI 100; rows show icon + title + subtitle (live capture); FULL gate
GREEN. Full EARS in the spec.
