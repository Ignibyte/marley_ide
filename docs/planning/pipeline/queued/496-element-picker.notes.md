# B3a: Pick an element in the Browser tab and send it to the agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-496-element-picker.md
- **Pipeline spec:** 496-element-picker.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar A (`browser-handoff.md`), after wave 1 (#487 to #495).
- **Classification:** feature, size L; `marley_browser` (the inspect calls, the bundle's
  reads), `marley_workbench` (pick mode, the tray, Send, the tools), `marley_mcp` (two rows).
  No Zed path expected.
- **Recall (§18.3):**
  - #492's snapshot refs and isolated world; #493's `tab` argument; #495's rule that a list
    opens only for the user, which pick mode shares (only the user turns it on).
  - `browser-handoff.md` open decision 3 ("Picks sent to the agent at once, or staged for Chad
    to confirm and caption first. The intake leans toward staged"): D1 takes the lean, under
    Chad's "do your best for decisions".
  - The measured cost of snapshots (about 3,400 tokens for an interactive list): a pick's
    bundle stays small and names one element.
- **The probe (2026-09-25):** see the spec's prior art.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
