# TICKET-671 — The rail keeps the window's order

- **Ticket:** LOCAL #671 (bug, the rail)
- **Owner:** unassigned (noted 2026-10-06 from Chad's report)
- **Pipeline doc:** none yet
- **Source ticket:** #542 (the attention order), #602 (drag to reorder)
- **Status:** open

## Summary
Chad, 2026-10-06: "when i start typing in marley the project moves to the top above another
project? Lets remove that. for some reason it jumps back randomly also". Nothing reorders on
focus or on typing as such. The rail sorts by attention (#542, `marley.rail_order`, default
`"attention"`): an agent that prints output counts as Working, and Working sorts above Idle
(`marley_rail::project_order`, `crates/marley_rail/src/marley_rail.rs:931`; the quiet timer is
`terminal_snapshot` and `note_output` in `rail.rs`). Typing into Claude Code echoes output, so its
project climbs; when the output goes quiet the project drops back to its window place, which is
the jump back. Rows under a project sort the same way.

The fix is to make `"window"` the default, so projects and their rows stay where the window and
the drag order (#602) put them. The setting stays, so attention order can still be turned on.

## Where
- `assets/settings/default.json` (`marley.rail_order`, about `:1762`, and its comment); the
  touchpoint row in `docs/marley/zed-touchpoints.md` for `default.json`.
- `crates/settings_content/src/marley.rs`: `MarleyRailOrder`'s `#[default]` and the
  `rail_order` doc's `Default:` line; its touchpoint row.
- `crates/marley_rail/src/marley_rail.rs`: `RailOrder`'s `#[default]`, so the pure crate agrees.
- The guide and the walkthrough's #542 stops, which say the rail puts what needs you first.

## Open point for Plan
Whether the collapsed header's counts (`1 waiting, 2 working`, #542) stay. They don't move
anything, so the default plan keeps them.

## Acceptance
With no `rail_order` in the user's settings, an agent that works, waits or finishes in the lowest
project leaves every project and row where it was. Setting `"attention"` brings #542's order back.
Proof: a scenario built from #542's stand-ins, which shoots the rail before and after the bottom
project's agent works and waits.
