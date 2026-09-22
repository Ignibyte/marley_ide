# TICKET-396 — Terminals onto the ContentId registry: the load-bearing ownership migration (the #394 slice-2 capstone)

- **Forge:** #396 `cd706b38-4b96-4390-a7c0-86339d96f086` (sprint #38 `fc972431`)
- **Type:** feature
- **Milestone:** M27
- **Status:** closed (done 2026-08-04 — GATE GREEN [diff] 15/15, suite 1989/1989; forge #396 done; AAR `a9c14f62` submitted)
- **Depends:** #394 (the pure `ContentRegistry<C>`) — HARD; #394 ships the refcounted lifecycle this loads onto
- **React-first:** N/A — behavior-neutral ownership migration, zero user-visible change (the validate
  parity check doubles as the no-change proof). Vocabulary tie: keep **ContentId** aligned with the
  React POC's `PaneItem` stand-in (`marley-web/docs/MARLEY-PARITY.md` § Shared vocabulary) so the
  later pane-composition slices (add-to-pane, arrangements, residents) port cleanly.
- **Pipeline:** completed — `docs/planning/pipeline/completed/396-terminals-onto-registry.spec.md`
  (all phases PASS, auto-approved run 2026-08-04)

## Summary
The **L-sized, load-bearing** half of the #388 pane-composition spine (the D5 split of #394; Opus design
2026-07-22). #394 shipped the pure generic `ContentRegistry<C>` (insert / acquire_view / release_view /
drop-on-last-close, unit-proven cov/MSI 100, gate-green via the #371 `orchestration_for` pub-use template).
This ticket does the actual terminal-**ownership** migration onto it — deliberately its own reviewed ticket
so a half-migrated reaper/persistence never ships (the #392→#395 discipline).

Scope (from `pane-composition-model.md` Q3 + the #394 Explore):
- **The concrete `Content` enum** — declare all 6 kinds (Terminal wired; Editor[=CodeView]/Git live-but-later;
  Cockpit/Browser net-new declared — the #385-comment discipline); `Content::Terminal` owns the
  `Box<TerminalPane<TerminalSession>>` (or the bare `TerminalSession`); `RootView` gains
  `content: ContentRegistry<Content>` at boot.
- **Migrate terminal ownership:** `PaneContent<S>::Terminal(Box<TerminalPane>)` → a `ContentId` view
  (workspace.rs:317/322); the **~46 app.rs accessor sites** (`focused_terminal`/`_mut` ×~40 + `states`/`_mut`
  ×6; defs workspace.rs:627/633/705/710) resolve through the registry (an id + `ContentKind` tag keeps
  `kind()` pure — the #388 open-Q1, recommended).
- **The close→reaper contract moves into the registry:** the **7 spawn-drop sites** (5 user: pump-dead-pane
  app.rs:1574/1585, close-tab :7442, close-project :7556, close-focused :8869/8875, close-pane :18049/18055;
  + 2 boot-cleanup :2140/:2160) route through `release_view` → the registry returns the owned session ONLY
  on the last-view drop → the caller keeps the ~600ms **off-thread** `thread::spawn(drop)` reap (the
  `OsPtyChannel` Drop / TICKET-348 bounded reap in `terminal_blocks/pty_os.rs:102` + `session.rs` reap_step
  — UNCHANGED; only *who holds the session until the last view* changes). Byte-identical to today for the
  1-view case (every view is 1-view until add-to-pane / slice-5).
- **Persistence rebuild:** restore builds registry entries from the restored grid shapes; ids NEVER serialize;
  the #163/#205/#177 round-trips stay byte-identical (codec stays coordinate/shape-based).
- **Re-key** the `agents`/`remotes`/`notify_ticks` maps from `PaneId` → `ContentId` where they track the
  instance (agents follow content, not the cell).

## Out (later train slices)
Editors onto the registry (#388 slice-3, the #259 two-Buffers collapse); add-to-pane / one-instance-many-views
(slice-5); nameable arrangements (slice-6); cockpit/browser residents (slice-8).

## Headline acceptance
Every terminal is owned once in the registry; each close path routes through `release_view` and the PTY reaps
off-thread exactly as today (bounded, TICKET-348); the #163/#205 persistence round-trips are byte-identical
(ids never serialized); the full suite passes with zero user-visible change. Inspect critics: leak-path (every
acquire has a matching release on every close/replace/error path), reaper-timing equivalence, double-release
safety, restore-rebuild correctness, the 1-view byte-identity. Behavior-neutral → no driven capture.
