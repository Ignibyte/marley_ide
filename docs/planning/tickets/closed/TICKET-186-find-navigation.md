---
id: forge#186 (9ecc172c-c47d-4e41-aec9-35935ea656c5)
title: M12 — scrollback find: next/prev navigation + "n of m" match count
status: closed
milestone: M12 — The Agent Cockpit
pipeline: 48490af7-ab17-4347-88f6-3203359aed20
---

Extend the #51 find bar with a live "n of m" match counter + an active-match distinct highlight (next/prev +
wrap + scroll already exist from #51). Pure scrollback_matches (row+range list) + match_label (the counter);
shim shows the counter + tints the active match.
