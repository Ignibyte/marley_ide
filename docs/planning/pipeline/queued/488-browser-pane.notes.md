# B0a: A Browser tab that shows Marley's own Chromium — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-488-browser-pane.md
- **Pipeline spec:** 488-browser-pane.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; Chad's answers on 2026-09-24: Marley starts the Chromium;
  compose only, CJK later; "do your best for decisions".
- **Classification:** feature; a new Marley crate plus a new module in `marley_workbench`.
  Zed paths: the root `Cargo.toml` (a member and a dependency; its ledger row exists and
  grows). No Zed crate's source changes.
- **Recall (§18.3):**
  - PR-claude-callback-stored-inside-owned-resource-captures-weak-001: the frame task and any
    CDP event subscription the view owns hold the view weakly.
  - The #406 lesson (docs/marley_architecture/embedded-browser-model.md): transport failures
    arrive as silence; the tab needs its own liveness (the socket's close fails every waiter
    and draws "the connection closed").
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001: blocking work
    (reading `DevToolsActivePort`, decoding JPEG) is
    `background_spawn(futures::future::lazy(…))`.
  - The gate:21 dylint rules in Marley crates: `SharedString::new_static`, no entity update or
    notify in render.
  - Brain consultation 6fc61fddf7124217b31f3ddc9432bf01 (opened for B0; no prior decision on
    the seam).
- **Discovery (this session):** the three Explore reports (gpui frames and input; Zed's items
  and Marley's patterns; the transport, Rusty's units, `marley_mcp`), and the CDP probe's
  captures in the scratchpad.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
