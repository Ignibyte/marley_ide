# TICKET-400 — Cockpit onto the registry: the residency half (the #388 slice-8, cockpit only)

- **Forge:** #400 `95954769-f57f-4167-99ff-e8b01bfd5756` (sprint #39 `6558258f-a0d2-45fb-b601-cd5329a1cd77`)
- **Type:** feature
- **Milestone:** M28
- **Status:** closed
- **Depends:** #396 — **HARD** (its `Content` enum declares `Cockpit` net-new; this ticket cashes that
  declaration and follows its migration pattern). #398 (slice-5) — **SOFT, exposure half only**: the
  residency itself needs only #396 and may land before #398 (the ordering is flexible; #398 additionally
  needs #397, so residency-first is the likely path).
- **React-first:** N/A expected — pure residency, no UI delta (ownership/identity migration), with the
  ContentId↔PaneItem vocabulary tie; **CONDITIONAL, Phase 2 re-confirms** against #398's shipped state —
  if cockpit-in-a-pane becomes user-reachable in this slice, the arm flips to UI-affecting and the
  reachability is prototyped in marley-web first (the POC's `views/AgentsPane.tsx` / `views/ForgePane.tsx`
  / `views/BlockDetailsPane.tsx` already exist per the MARLEY-PARITY.md port map). Full section: the
  queued spec's `## React-first (parity)`.
- **Pipeline:** queued — `docs/planning/pipeline/queued/400-cockpit-residency.spec.md`

## Summary
The #388 train slice-8, **cockpit half only** (the Browser half stays gated on #389/Phase E). The three
cockpit surfaces — Details / Agents / Forge, today `TabContent::Cockpit(RightSection)` tabs selected by
the top-bar strip (the M9 #153 dock retirement; the ticket's "right_dock-resident" names the
`right_dock.rs` *vocabulary*, not a live side dock) — become **`Content::Cockpit` registry residents**
obeying the same one-instance-many-views lifecycle as terminals (#396) and editors (#397): one instance
per surface (3 app-wide singletons), any number of tabs/cells hold the `ContentId`, the view-count guards
the drop. Largely structural — the codec (`C=<section key>`), the `right_section` persistent selection
(#95), and the cockpit renders stay byte-identical; the payoff is uniform content citizenship (the Zed
map's "no uniform panel citizenship" GAP row closed for this kind, Marley's own enum+registry route),
which is the precondition for cockpit-in-a-pane (slice-5's gesture applies to registered kinds) and for
Browser residency later. Load-bearing wrinkle: cockpit residents are the first content whose view-count
legitimately exceeds 1 (two projects' cockpit tabs of one section = two views of one instance), and they
must be **pinned** — a last-view close must never reap a cockpit (no teardown exists; the top-bar
selection is a standing reference).

## Out (other tickets / later slices)
Browser residency + any webview substrate (#389/Phase E train); the add-to-pane gesture itself (#398);
any cockpit surface redesign (rows/glyphs/labels byte-stable); global cross-workspace panes (slice-7,
gated on multi-workspace); new cockpit features; moving `forge_sprint`/`agents` data into instances
(D-OPEN-STATE-MIGRATION's recommendation: identity only this slice).

## Headline acceptance
The three cockpit surfaces resolve through the registry by `ContentId` (singletons — the same section
always resolves the same id, across open/switch/second-project/restore); every cockpit close path routes
`release_view` and **no cockpit content is ever reaped by a last-view drop** (the pinned-lifecycle
guard); the top-bar strip, cockpit tab switching, `right_section` selection + #95 persistence, and the
`C=<key>` codec round-trips are byte-identical (existing suites green unchanged; ids never serialize);
parity handled per the React-first arm above. Inspect critics: reap-safety walk over every close path,
dock-render byte-identity, state-migration completeness, acquire/release leak pairing.
