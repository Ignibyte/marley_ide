# System One as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-847-system-one.md
- **Pipeline spec:** 847-system-one.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #846.
- **Classification / tier:** feature, S. Zed's `settings_ui.rs` (#844's hunk widened), Marley's
  `system_one_calls.rs`.
- **Recall (§18.3):** #844's segmented rows (D2: up to three).
- **Discovery:** a probe run: the uses are dropdowns (four variants); the calls tab is lines
  (calls today and spend, provider, key) and buttons.
- **Decisions:** D1 in the spec.

### Design
- `settings_ui.rs`: `(2..=3)` → `(2..=4)` in `render_dropdown`'s Marley branch; the ledger row
  says four.
- `system_one_calls.rs`: `render_header` draws Provider (provider · model, its key line, the
  Cloudflare line when it applies) and Today (spend of the budget, a bar by `spent_today` against
  `daily_budget_cents`, the calls count) as cards; the off and breaker lines kept; a muted note on
  what leaves the machine after the rows.
- **File manifest:** `crates/settings_ui/src/settings_ui.rs`, `system_one_calls.rs`; the scenario.

## Phase 2 — Code
- **Built:** `settings_ui.rs`: the Marley branch of `render_dropdown` takes two to four variants.
  `system_one_calls.rs`: `render_header` draws a Provider card (provider · model, the key line, the
  Cloudflare line when it applies) and a Today card (spend of the budget; a bar of the share spent,
  `spent_today` in billionths of a cent against `daily_budget_cents`, in thousandths through `u16`
  to keep clippy's casts quiet; the calls count); the off and breaker lines kept; the note on what
  leaves the machine after the rows, in a 720 px column so it wraps.
- **Deviation:** the first run's note ran off the tab's right edge; it wraps in its column now.
- **Gate:** red once on clippy (`doc_markdown` on `SystemOne`; `render_header` 102 lines, now with `stat_card` and `render_today`), fixed at source; the scenario ran again on the fixed code.
- **Review:** the buttons, the key field and the rows are unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/847-system-one.sh` (sway): System One on with `rules`, three uses set.
- `847-01-uses`: Settings searched for "kind": Stop Kind (Off | Shadow | **Suggest** | Act) and
  Stall Kind (… **Shadow** …) as framed buttons, each "changed · reset". REQ-001 met.
- `847-02-calls`: the calls tab: Provider (rules · jev-1.13.0, Key: not needed), Today (0¢ of 50¢,
  the empty bar, 0 calls), Run Check / Set Key / Forget Key, "No calls today.", the note wrapped.
  REQ-002 met.
- `847-03-day`: Marley Day, the uses. REQ-001 met.
- Observed, not this ticket's: in the Day shot the Stall Check After Seconds number field drew
  blank (it read 60 in Night); number fields are untouched here.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, the ledger row (four), look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 6053843c.
- Ticket closed; pipeline archived.
