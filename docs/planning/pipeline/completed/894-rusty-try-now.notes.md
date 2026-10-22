# Rusty dropped, with Try Now — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-894-rusty-dropped-and-a-missing-cli.md
- **Pipeline spec:** 894-rusty-try-now.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row (#893 is a large new view over Zed's diff; this is smaller).
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #857's card; Rusty's stand-in (`crates/marley_rusty/stand_in`).
- **Discovery:** the keeper waits on a timer between tries, up to 60 s; the screens print the
  reason as a line.
- **Decisions:** D1 in the spec.

### Design
- `rusty.rs`: `Rusty.wake: Option<oneshot::Sender<()>>`; the keeper makes a pair before each wait
  and waits on `select(timer, receiver)`; `try_now(cx)` sends; `render_dropped(id, cx)` is #857's
  card with the reason and Try Now.
- The three screens call `render_dropped` where they printed the line (Home keeps "still
  connecting" while starting).
- **File manifest:** `rusty.rs`, `rusty/home_tab.rs`, `rusty/memory_tab.rs`,
  `rusty/secrets_tab.rs`, the scenario.

## Phase 2 — Code
- **Built:** `rusty.rs`: `Rusty.wake`; the keeper makes a oneshot pair before each wait and waits
  on `select(timer, woken)`, clearing `wake` after; `try_now` takes and fires it; `render_dropped`
  is #857's card with the reason (the title's own words stripped from `unavailable`'s text) and Try
  Now. Home's Rusty card (`home_tab.rs`, still "connecting" as a line while starting), the Memory
  and Secrets tabs use it.
- **Fix in Test:** the first run's line repeated the card's title; it is stripped.
- **Gate:** green, 17 of 17 (clippy: `select` already imported).


## Phase 3 — Test
- **Scenario:** `script/e2e/894-rusty-try-now.sh` (sway): a wrapper `rusty-mcp` that fails while
  a flag exists.
- `894-01-dropped`: Home's RUSTY card: the R tile in the error's tint, "Rusty is not connected",
  "rusty-mcp did not answer within 5 s", Try Now. REQ-001 met.
- `894-03-day`: Marley Day, the same. REQ-001 met.
- `894-02-back`: 70 s on (each try up to 5 s, the waits doubling from 1 s, so Marley was in its
  32 s wait), the flag removed and Try Now clicked: 3 s later Rusty's cards (Brain, Tasks, …,
  Recent pages) show. REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W6 line.
- Knowledge: none new. Brain: `brain decide` on consultation b1de08bb.
- Ticket closed whole: a missing agent CLI is said on Troubleshooting (#885) and no launcher offers
  one, recorded in the spec.
