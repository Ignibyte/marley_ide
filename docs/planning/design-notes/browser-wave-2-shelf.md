# Browser wave 2: the three pillars

Prong 3's second wave, specced on 2026-09-25 once wave 1 (#487 to #495) had landed: the Browser
tab shows Marley's own Chromium, takes input, navigates, keeps a tab per page and brings them
back after a relaunch, draws select lists, and agents see and drive it over Marley's MCP server.
Wave 2 builds the pillars of `browser-handoff.md` on top of that, in this order:

| Ticket | Slice | What |
|---|---|---|
| #496 | B3a | Pick mode, the durable bundle, picks staged for a caption and sent to the terminal, `browser_pick` |
| #497 | B3b | A pick's listener through its source map, opened in the editor at the line |
| #498 | B4 | Annotations drawn by gpui in page coordinates, by Chad and by agents |
| #499 | B5 | The flight recorder: the last minute per page, "Record this", `browser_recording` |

Two of the handoff's open decisions are taken for now, under Chad's "do your best for decisions",
and stay his to change:

- **Open decision 3, picks.** Staged: Chad captions and sends each pick (#496 D1), the handoff's
  lean.
- **Open decision 4, annotations.** Session only, dropped on navigation (#498 D3), until he says
  how they should last.

The queued pairs live in `docs/planning/pipeline/queued/496-*` to `499-*`.
