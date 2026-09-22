# TICKET-419 — Rail single-selection highlight + dot indicators

- **Ticket:** LOCAL #419 (feature, M31)
- **Tags:** rail, highlight, selection, simple-rail
- **Created:** 2026-08-12
- **Status:** closed (2026-08-12 — shipped: RailSelection coordinate + RailDot in the pure model, the six arms re-routed, 2131 tests green, driven live, parity pair captured; GATE GREEN [diff] 15/15)

## Summary

Port #418's selection model to the Rust rail. Today one click lights up to SIX rows with the
identical accent fill — `rail_rows` computes `active` independently at every level
(tabs.rs:1047/:1103/:1146/:1164/:1180/:1241) and all render arms share `rail_highlight`
(app.rs:1421). Replace with: exactly ONE row (the focused thing itself) carries the selected
fill; ancestor rows (project, section, tab-of-pane, arrangement) never do; rows whose content
is open/focused elsewhere (pane-mounted cross-refs, active-in-background) get a small
left-edge dot indicator instead. Mind F-#386: `rail_rows` signature/behavior changes fan out
to `#[cfg(test)]` call sites — `cargo check --tests`.

## Acceptance

Headline: after any click in the rail, exactly one row renders the selected fill (pinned by
a rail_rows-level test over every RailLevel), and dot indicators replace the old
ancestor/duplicate lighting; React↔Marley parity pair captured. Full EARS in the queued spec
(`docs/planning/pipeline/queued/419-rail-single-selection-dots.spec.md`).
