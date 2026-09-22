# TICKET-399 — Nameable arrangements — the [[panes]] settings table (the #388 slice-6)

- **Forge:** #399 `d7b3b391-534f-4f39-93c0-033c408f55ab` (sprint #39 `6558258f` "M28 — The Registry Payoff")
- **Type:** feature
- **Milestone:** M28
- **Status:** closed
- **Depends:** #398 (slice-5: add-to-pane + the Pane-row `ContentId` labels) — HARD; #396/#397 transitively (slice-5's own deps: terminals + editors registered)
- **React-first:** APPLICABLE — Zone A rail + a new naming flow. Prototype the name-an-arrangement
  draft and the named Arrangement rows in marley-web (`LeftRail.tsx` — the POC already renders
  `PANE ${pane.id}` rows; it gains the naming flow here as the prototype), confirm at
  localhost:5173, then port 1:1 (`marley-web/docs/MARLEY-PARITY.md` port map; ContentId↔PaneItem
  vocabulary stays aligned per § Shared vocabulary).
- **Pipeline:** queued — docs/planning/pipeline/queued/399-nameable-arrangements.spec.md

## Summary
The #388 train slice-6 (`pane-composition-model.md` Q4/Q5): arrangements — multi-cell tab layouts —
become nameable, persistable objects. A "make a Pane"/"name this arrangement" verb reuses the #204
inline-draft naming idiom (draft field + key-ladder branch + `text_input_blocked` line + centered
card — no new modal machinery); a new `[[panes]]` settings table (the #204 `[[workflows]]`
`define_setting!` template) stores name + scope root + shape + per-cell stable content keys; rail
Arrangement rows show the name instead of the #390 "PANE n"; restore rebuilds named arrangements
and DROPS dangling references (a key that no longer resolves loses its cell; an arrangement whose
cells all drop is skipped — the #205 guard stance + the #386 forgiving-restore posture) rather than
ever failing the restore. Codec discipline per #163/#205/#396: shapes/coordinates/names serialize,
`ContentId`s NEVER do — refs re-resolve to fresh ids at restore time (terminals respawn fresh-in-cwd
per #205, with a per-cell ordinal so two same-cwd cells don't collapse; editors reopen by path).

## Out (later train slices / explicitly not this)
Global cross-workspace Panes (slice-7, gated on multi-workspace); arrangement sharing/export; any
auto-capture of layouts (naming is an explicit verb — no implicit save of every split); an
arrangements manager UI (list/delete beyond the rail); cockpit/browser content in arrangements —
lands with #400 automatically once those kinds are registry residents (the resolution seam is
kind-agnostic over stable keys).

## Headline acceptance
Naming a live multi-cell arrangement persists a `[[panes]]` entry that round-trips through the
settings file (the persist TRIGGER exercised, not just the codec); the rail row shows the name;
after restart the named arrangement is rebuilt with dangling cells silently dropped and the rest
intact; unnamed multi-cell tabs still read "PANE n" byte-identically; the persisted bytes never
contain a `ContentId`; the #163/#205/#177 round-trip suites and the #390 rail suites pass
unchanged; Validate captures the React↔Marley parity pair. Full EARS in the pipeline spec.
