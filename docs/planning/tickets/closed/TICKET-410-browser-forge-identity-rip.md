# TICKET-410 — The browser loses its forge identity (product-rip slice 1)

- **Ticket:** LOCAL #410 (chore, M30)
- **Tags:** M30, pivot, browser, product-rip, 405-lineage
- **Created:** 2026-08-09
- **Pipeline doc:** ../../pipeline/completed/410-browser-forge-identity-rip.spec.md
- **Status:** closed (2026-08-09 — shipped; gate GREEN [diff]; parity-paired)

## Summary
First slice of the scrap-forge product rip (intake:
`docs/planning/intake/scrap-forge-pivot.md`; ordering per the 2026-08-09 seam map —
this slice MUST precede TICKET-411, because the browser chain holds three
independent edges into `marley_forge_client` and deleting the crate first would
take the pane down as collateral). Remove the browser's forge identity while
keeping the generic embedded-browser infrastructure (wry child, z-order shim, the
#406 lifecycle machine):

- Delete `RootView.forge_web_base` and its whole chain (boot derivation
  app.rs:2489-2491, `derive_forge_web_base_now` :11111-11126, the Retry re-derive
  arms, `mount_plan` feed :10914, the origin caption :19081, the test hook).
- ~~Delete `marley_app/src/mcp_config.rs` (its only two consumers are that chain)~~
  **Promotion correction (Explore, 2026-08-09):** the claim was wrong — the boot
  call site (app.rs:2476) also feeds the sprint `forge_client` and the fleet-brain
  `endpoint_for_brain` (411-scope). 410 deletes only the `forge_web_base` legs;
  the module falls in TICKET-411 with its last consumers.
- Re-home the pinned-origin nav predicate (`same_web_origin`, webview_shim.rs:74)
  as a local in-app predicate — or drop the pin, per design.
- Retire the "Forge web view" placeholder copy + its exact-copy test; the
  `CommandId(31)` "forge" palette keyword; decide `orchestration.web_url`'s fate
  (currently DEAD — no production consumer).
- The B tab's no-origin state = the existing #404 empty-rail-home behavior.

Design decides: the browser's post-forge origin source (none-until-a-future-URL
feature vs adopting `orchestration.web_url`). Tests: the seam map counts ~2
forge-coupled mcp_config tests + 1 copy test + 2 headless forge-identity drives to
retire/repoint; the 20 `browser_state` units and the generic drives survive.

## Acceptance
No forge-derived origin anywhere in `marley_app`; the embedded-browser
infrastructure compiles, tests green, and the B tab renders its empty rail home;
`mcp_config.rs` gone. Full EARS at promotion.
