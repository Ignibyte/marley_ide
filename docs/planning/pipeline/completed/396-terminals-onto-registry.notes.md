# 396-terminals-onto-registry — Notes

- **Forge ticket:** #396 `cd706b38-4b96-4390-a7c0-86339d96f086` (feature, sprint #39 "M28 — The
  Registry Payoff"; carried from M27 — the #388 train slice-2, the #394 D5-split's L half)
- **AAR:** a9c14f62-2a23-458a-85a6-d88d43b7727c (opened at /work promotion 2026-08-04; owner
  `90f02e73-89f8-4461-82d0-fe212c0dbb01`)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-396-terminals-onto-registry.md
- **Pipeline spec:** 396-terminals-onto-registry.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan (2026-08-04, /work 396 auto-approved)
- **Request / provenance:** the #394 D5 split's second half (394-content-registry.notes.md §(0) —
  "SPLIT: #394 = slice-1 (the pure lifecycle); #396 created for slice-2 (the ownership migration)").
  No queued spec existed ("the full spec drafts at promotion" per TICKET-396) — drafted HERE from
  the ticket's own dense description + pane-composition-model.md Q2/Q3 + the #394 notes (re-read in
  full this phase; the shipped registry contract: insert-acquires-1-view, `release_view` returns
  the owned content exactly on last drop, `#[must_use]` on both handles — the #394 inspect F1).
- **Pre-flight:** all green (cargo 1.96.0 / mutants 27.1.0 / llvm-cov 0.8.7 / hooks wired /
  marley-web OK); no active pipeline; bulletins: none; tree clean at a2ed09c (#352 shipped).
- **Recall (§19, knowledge-context on the open AAR — 13 surfacings logged):** the binding cluster,
  by name: `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001` (the #388-inspect
  catch the #394 registry embodies), `PR-claude-registry-release-returning-a-resource-must-be-must-
  use-001` (every #396 call site consumes the return),
  `PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001` (the Content matches),
  `BF-claude-skip-detach-pump-fleet-live-001` + `PR-claude-trace-the-real-cargo-mutants-list`
  (app.rs shim turf), `PR-claude-domain-sweep-includes-equality-gates-001` (fresh from #352 — the
  P2 inventory greps comparisons, not just calls), ADs `AD-claude-pane-content-id-registry-001` +
  `AD-claude-one-instance-many-views-001`. The #394 promote-entry's flagged uuid cluster
  (452ae5ee/7f19c7c9/71b36786/e331b13c + fde3f2a1) feeds the leak-path/reaper critics at inspect.
- **Hard invariants carried from #394 (the promote-entry's list, verbatim obligations):** the codec
  NEVER learns ids (#163/#205/#177 byte-identical); the reaper fires byte-identically for the
  1-view case (every view IS 1-view until #398); ZERO user-visible change (the full suite is the
  proof); a half-migrated reaper/persistence must never ship (the strangler steps each compile
  green).
- **STALENESS FLAG (load-bearing for P2):** every line ref in the ticket (workspace.rs:317/322,
  :627/633/705/710; app.rs:1574/1585/7442/7556/8869/8875/18049/18055/:2140/:2160; the ~46 accessor
  count) is from 2026-07-23; #352 alone moved app.rs by ~+250 lines and the #352 P1 sweep proved
  such inventories drift AND undercount (two missed IME sites there). P2's first act = the fresh
  inventory at HEAD, greps INCLUDING equality/comparison sites.
- **Decisions locked (D1-D8) + five D-OPENs** (VIEW-SHAPE / PAYLOAD / S-GENERICITY /
  ID-ALLOC-AT-RESTORE / ENUM-HOME) — see the spec; the D-OPENs are all settle-with-the-code-in-hand
  questions, none blocks the plan.
- **React-first: N/A** (behavior-neutral; the spec's section leads with N/A per the
  enforce-react-parity contract) with the ContentId↔PaneItem vocabulary tie (MARLEY-PARITY.md
  § Shared vocabulary — no POC edit expected).
- **EARS drafted (7):** ownership-once + pure fns (REQ-001); every close path through
  `release_view` + off-thread reap (REQ-002); persistence byte-identity + rebuild (REQ-003); the
  side-table re-key behavior-neutral (REQ-004); full suite byte-identical (REQ-005); floors + gate
  (REQ-006); the explicit N/A statements (REQ-007).
- **Sizing:** L (multi-day by the Q3 table; one shippable slice by construction — the strangler
  order keeps every intermediate green; the D5 split already removed the half-migrated risk).

## Phase 2 — Design (2026-08-04, auto run)

### (0) THE FRESH INVENTORY (Explore fan-out at HEAD a2ed09c — the ticket's numbers corrected)
- **Drift map:** app.rs is 19,992 lines; sites below ~7,600 moved −2..+14, above ~8,900 moved
  +29..+135. workspace.rs (1,751) / tabs.rs (2,154) / grid_layout.rs (946) / content_registry.rs
  (220): ZERO drift — every workspace.rs ref in the ticket is still exact.
- **Accessors (ticket ~40+6 → REAL 36+5, but the ticket under-scoped the class):** app.rs
  `focused_terminal` ×15 + `focused_terminal_mut` ×21 (=36); `.states()` ×2 + `.states_mut()` ×3
  (=5); PLUS the un-ticketed bulk: by-id `.terminal(`/`.terminal_mut(` ×27, `.state(`/`.state_mut(`/
  `focused_state_mut(` ×14, `.session` field touches ×63 (app.rs) + 4 (headless) + 7 (integration),
  5 free fns taking `&TerminalPane<TerminalSession>` (block_line_counts/content_rows/
  content_row_texts/observed_row_texts/grid-pos-mapper, app.rs:2969-3059). Defs at workspace.rs
  :627/:633/:705/:710 exact as ticketed.
- **Births (9 flows, single constructor):** the ONLY `TerminalPane{..}` literal is inside
  `PaneState::new` (workspace.rs:342-358; internal callers :466 new_with_base, :537 split_focused).
  App flows: boot pre-spawn :1430; #163 restore first-terminal :2026-2043 + restore_panes extra
  panes :1958; legacy #205 restore :2173-2178; launch_agent :3677; new-tab :6736-6744; split
  :6860-6882; open-project :7355-7373; open-remote :7692-7696.
- **Close/drop paths — 7 reaps CONFIRMED at drifted lines** (pump dead-pane :1588/:1599; close_tab_at
  :7453/:7469 — implicit whole-Tab container drop; close_project_at :7554/:7582 — implicit ×2
  levels; close-pane ⌘W :8898/:8904; pane-× :18184/:18190; boot reaps :2147/:2167) **+ FOUR the
  ticket missed:** (M1) `headless_drive.rs:193-203 reap_sessions` — mem::replace of the WHOLE shell
  + off-thread drop, ~220 call sites — must swap the registry alongside or every drive leaks; (M2)
  `tests/integration.rs:188` inline on-thread drop (deliberate, documented); (M3)
  `workspace.rs:685 set_content` in-place overwrite — drops old content on the calling thread,
  UN-reaped — zero production callers today, but it is the replace-path hazard: re-signature under
  the migration; (M4) **RootView has NO Drop impl** — app QUIT drops the whole shell synchronously
  on the UI thread through five container levels today; post-migration the same quit drops
  `content` (one map) — equivalent shape, flagged not changed.
- **Side-tables:** `agents` (:190, 22 keyed sites), `remotes` (:387, 10), `notify_ticks` (:192, 2 —
  scrubbed at only ONE of five close paths, the #226 gap: PRESERVE), `tab_flashes`
  ((project,tab)-keyed — does NOT re-key), scalars `last_agent` (:583) + `pending_agent_send`
  (:159) + `completion` (:448) + `MenuKind::Block.pane`. Pure consumers: agent_view.rs
  (fleet_badge/agent_rows/fleet_status_for/agent_pane_ids), status_bar.rs
  (agent_summary/cockpit_status).
- **PaneId identity-proxy sites (the #352-class sweep):** completion pane-binding :7850/:19330;
  Block-menu target :6895→:6980/:7001/:7012/:7032; notify foreground :1524; resync skip :5638;
  render focus :17208/:18062; **PTY-resize rect-match :17164 (the sharpest future multi-view
  hazard — two views ⇒ two rects ⇒ conflicting resizes against one pty_size; 1-view-safe today)**;
  `locate_pane` bridge tabs.rs:594-601 ("at most one grid can match" — PaneId stays the VIEW key,
  invariant HOLDS post-migration since ContentId never enters the tree); MCP opaque handle =
  PaneId.0 (:7083/:7092 — stays pane-keyed, it names a SURFACE).
- **Persistence:** persist builds PaneId-keyed lookup maps (:4907-4971 persist_grid,
  :4976-5033 build_shell_layout) — grid_layout.rs needs ZERO signature change (PaneId only as the
  caller-map key type; leaves stay `t/t=cwd/c=path/f/g`). Restore callers :2003→:2044, :2179-2181.
- **Registry (#394) unchanged, ZERO consumers**, `#[must_use]` on acquire/release; **NO iteration
  API** — the 5 `.states*()` loops (pump :1488, resync :5637, agent-pick :8977, ⌘K sources :9582,
  PTY-resize :17158) need one.
- **Test lanes:** workspace.rs `MockSession{drops:Rc<Cell<u32>>}` drop-counter (27/47 tests) + 3
  `<()>` tests; tabs.rs `<()>` only (42/58 tests — ZERO teardown coverage for tab/project close —
  the implicit container drops are un-instrumented today); integration.rs = the one real-PTY grid
  lane; headless = real sessions via RootView.

### (1) D-OPEN resolutions (all five, with the code in hand)
- **VIEW-SHAPE → `PaneContent::Terminal(ContentId)`** (the variant IS the kind; `kind()`/
  `rail_section()`/`focus_label` stay pure with unchanged outputs; FileTree/CodeView/Git variants
  untouched this slice).
- **PAYLOAD → `Content::Terminal(Box<TerminalPane<TerminalSession>>)`** — the WHOLE pane state
  moves (byte-identity for the 1-view case; the per-view split of viewport/scroll_remainder/
  selection/folds + the pty_size two-rects question are #398's design load, RECORDED for it in the
  inventory above, not solved here).
- **S-GENERICITY → DISSOLVE.** Ownership leaves the tree, so `PaneContent`/`PaneState`/`PaneGrid`/
  `TabContent`/`Tab`/`Project`/`Workspace` lose `<S>` entirely (~70 signature sites in
  workspace.rs+tabs.rs + 14 app.rs instantiations + ~60 test-instantiation touches ≈ 147, all
  mechanical); `TerminalPane<S>` KEEPS its param (it is the payload type; tests may still fake a
  session). The 27 MockSession drop-counter tests split: their GRID-algebra halves stay (id-typed
  grids, no sessions); their close-kills-the-session halves are REGISTRY territory — already proven
  by the #394 matrix pattern (release returns the owned content) — plus NEW app-level close-path
  units (REQ-002). rail_rows<S> → rail_rows (free fn, tabs.rs:786).
- **ID-ALLOC-AT-RESTORE (and at every birth) → spawn → insert → mount, leak-free by ORDER:**
  (1) spawn the session (fallible — nothing inserted on Err), (2) `content.insert(Content::
  Terminal(..))` → id, (3) mount the id (grid ctor / `split_focused` re-shaped to take the id —
  the tree insert is infallible once the state exists). No leak window; the closes are the exact
  inverse (tree-remove → `release_view` → reap-on-Some).
- **ENUM-HOME → a NEW `crates/marley_app/src/content.rs`**: `Content` (6 variants: Terminal
  wired + Editor/FileTree/Git/Cockpit/Browser declared-unit with `/// lands in slice-N` docs — the
  #385-comment discipline), `ContentKind` (Copy, the pure classifier), `Content::kind()` +
  `as_terminal()/as_terminal_mut()` (pure matches, own `#[cfg(test)]` rows, cov/MSI 100), re-export
  via lib.rs `pub use` (the #371/#394 gate-safety idiom). The generic registry stays untouched in
  content_registry.rs; **one small ADDITION there: `iter()`/`iter_mut()` over `(ContentId, &C)` /
  `(ContentId, &mut C)`** (+ unit rows keeping 100/100) — required by the five iteration loops.

### (2) Architecture — resolution moves UP one level
The grid stops being able to answer "give me the terminal" alone (it holds ids); resolution lives
on RootView where BOTH fields exist: `fn focused_terminal(&self) -> Option<&TerminalPane<
TerminalSession>>` = `self.try_workspace()?.focused_terminal_id()` → `self.content.get(id)?.
as_terminal()` (+ `_mut` twin). The 36 focused_terminal* sites keep their SHAPE (receiver already
`self.…` — they re-point to the RootView helpers); the 27 by-id sites get the by-id twin
(`fn terminal_at(&self, pane)`); iteration sites (pump/resize/resync/agent-pick/⌘K) rewrite over
`self.content.iter_mut()` filtered to Terminal (1-view ⇒ identical coverage, and the shape is
future-proof against double-pumping one content in two views) EXCEPT where the PaneId matters
(resize needs the pane→rect match: iterate the grid's pane→id pairs then resolve). Where &mut shell
and &mut content are needed simultaneously (the pump, split-spawn, closes), FIELD-SPLIT
(`let RootView { shell, content, .. } = self` or split at the closure boundary) — never a double
`&mut self` method chain. `PaneGrid::close` returns `PaneState` (now id-bearing) — the caller
extracts the id → `release_view` → `Some(content)` → `thread::spawn(move || drop(content))` at the
SAME sites the reap lives today; the implicit container drops (close_tab_at/close_project_at) stop
being implicit: collect the removed Tab/Project's content-ids FIRST (walk its grids), tree-drop the
(now session-less) container inline, release each id, spawn ONE reap thread with the collected
Vec<Content> (today: one thread with the container; equivalent boundedness, same thread count).
reap_sessions (headless) swaps BOTH `shell` and `content` and drops the pair off-thread.
set_content → `set_content_id(pane, ContentId)` returning the OLD id for the caller to release
(#[must_use]); its 3 tests re-keyed.

### (3) File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/content.rs` | NEW — `Content`/`ContentKind`/pure accessors + tests |
| `crates/marley_app/src/content_registry.rs` | + `iter()`/`iter_mut()` (+unit rows; nothing else) |
| `crates/marley_app/src/lib.rs` | `mod content;` + `pub use content::{Content, ContentKind};` |
| `crates/marley_app/src/workspace.rs` | de-genericize; `PaneContent::Terminal(ContentId)`; `PaneState::new(ContentId)`; grid ctors/split take ids; `set_content_id`; MockSession tests re-expressed (grid halves stay; drop halves → registry/app units); TerminalPane def STAYS here (payload type) |
| `crates/marley_app/src/tabs.rs` | de-genericize (38 sites incl. rail_rows); `<()>` test turbofish drops |
| `crates/marley_app/src/app.rs` | `content: ContentRegistry<Content>` field + boot init; the RootView resolution helpers; ~150 site re-points (accessors/births/closes/iterations/side-tables); side-tables re-key to ContentId (agents/remotes/notify_ticks + last_agent/pending_agent_send as Content-scalars; completion/MenuKind::Block/tab_flashes STAY pane-keyed — they name views/surfaces) |
| `crates/marley_app/src/agent_view.rs`, `status_bar.rs` | key-type rename PaneId→ContentId in the 6 pure fns + their tests (mechanical; assertions unchanged) |
| `crates/marley_app/src/headless_drive.rs` | reap_sessions swaps shell+content; any direct constructions re-keyed |
| `crates/marley_app/tests/integration.rs` | re-written over grid+registry together (becomes the REQ-002 real-PTY off-thread proof) |

Side-table re-key boundary (D6 refined by the inventory): `agents`/`remotes`/`notify_ticks` +
`last_agent`/`pending_agent_send` re-key (they track the INSTANCE — an agent/remote/notify follows
its terminal); `completion` + `MenuKind::Block.pane` + `tab_flashes` + the MCP handle STAY
PaneId/(p,t)-keyed (they name a VIEW/SURFACE — the popup's anchor pane, the menu's target cell, the
tab chip, the jumpable surface). This line is the Q2 instance-vs-view split applied to keys.

### (4) Strangler order (each step `cargo check`-green)
1. **content.rs + ContentKind + registry iter + lib.rs wiring + `RootView.content` field (init
   empty at boot).** Dormant-but-public (the #371 idiom); full suite untouched.
2. **THE TYPE SWAP (the atom — one coherent step, compiler-driven):** workspace.rs de-genericize +
   `Terminal(ContentId)` + ctor/split/set_content re-signatures → tabs.rs de-genericize → app.rs:
   the resolution helpers, then every broken site re-pointed in rustc-error order (births spawn→
   insert→mount; closes tree-remove→release→reap; iterations onto iter_mut/pane-pairs; the 5 free
   fns' callers resolve first). workspace/tabs/integration tests re-keyed in the same step.
3. **Side-table re-key** (app.rs sites + the 6 pure fns + their tests).
4. **headless reap_sessions + any drive-side re-points.**
5. `cargo nextest run --workspace` — full suite byte-identical; fix any behavioral drift AT SOURCE.

### (5) Regression Test Plan
| REQ | Test |
|---|---|
| REQ-001 | content.rs pure rows (kind/as_terminal per variant, exhaustive — no catch-all); registry iter rows; app-level unit: open/split N terminals → `content.len() == N`, every id view_count 1; #382 focus_label + #390 rail suites green unchanged |
| REQ-002 | workspace close-path units re-expressed: close returns the id-bearing state; app-level per-path units (pump-dead / close-tab / close-project / close-pane / pane-× / both boot arms): registry len returns to expected + `release_view` consumed; integration.rs real-PTY: split→close→release→off-thread drop (the on-thread-vs-off assertion via the existing #348 timing lane where present); set_content_id returns the old id (#[must_use]) |
| REQ-003 | #163/#205/#177 round-trip suites byte-identical; new restore unit: restored shell ⇒ registry len == restored terminal count, all view_count 1; inspect grep: no ContentId in any persist/serialize path |
| REQ-004 | agent_view/status_bar pure-fn tests re-keyed with assertions unchanged; notify/badge suites green; the #226 scrub-gap landscape asserted UNCHANGED (the one pump-scrub site still the only notify_ticks scrub) |
| REQ-005 | `cargo nextest run --workspace` FULL green; test-diff review at inspect: re-keying only, no assertion weakened |
| REQ-006 | gate `--diff` green; `cargo mutants --list -f` on content.rs + content_registry.rs + workspace.rs + tabs.rs (the real list); skip-detach re-check on app.rs |
| REQ-007 | the validate notes state driven-capture N/A + React-first N/A explicitly |

### (6) Risks / decisions record
- **The atom (step 2) is wide** — mitigated by: compiler-driven (the type swap makes rustc
  enumerate every site — no grep-miss risk), the field-split discipline for shell+content double
  borrows, and NO behavior edits inside the step (pure re-pointing; any judgment call gets a
  `// #396:` comment for the critics).
- **close_tab_at/close_project_at change from one-container-drop to collect-ids→release-each→one
  reap-thread of Vec<Content>** — same thread count, same boundedness; the CONTAINER (session-less)
  now drops inline (cheap — no PTY). Inspect verifies no path drops a Content on the UI thread.
- **Quit-path**: content map drops on the UI thread at quit exactly as the shell did — unchanged
  behavior, now explicit; noted for a future ticket if quit-time reaps ever matter.
- **MockSession coverage moves**: the grid tests stop proving session-drop (they can't — no
  sessions in the tree); the proof RELOCATES to registry+app units + integration. Inspect confirms
  no drop-assertion is silently lost (count the before/after drop-assertions).
- **pty_size/viewport per-view debt** recorded for #398 (the inventory's Q2-b table is its input).
- Recall applied: must_use consumed everywhere (REQ-002), closed-enum exhaustive matches
  (content.rs), equality-gate sweep done (the identity-proxy table above), skip-detach re-check at
  validate, `PR-claude-domain-sweep-includes-equality-gates-001` satisfied by inventory group 6.

## Phase 3 — Implement (2026-08-04, auto run)

**React-first: N/A** — no UI delta (the spec's parity section: behavior-neutral ownership
migration; the ContentId↔PaneItem tie is vocabulary, not pixels). Recorded per the hook contract.

### What was built (the five steps, as designed)
1. **content.rs (NEW)** — `Content` (Terminal wired via `Box<TerminalPane<TerminalSession>>`;
   Editor/FileTree/Git/Cockpit/Browser declared with `/// lands in slice-N` docs), `ContentKind`,
   `kind()`/`as_terminal()`/`as_terminal_mut()`; lib.rs `pub use` (the #371/#394 idiom).
   Registry additions: `iter()`/`iter_mut()` over `(ContentId, &C)` (the five iteration loops) +
   `#[cfg(test)] ContentId::test(n)` (grid/tab suites mint view ids without a live registry).
2. **THE TYPE SWAP** — `PaneContent::Terminal(ContentId)`; S-genericity dissolved across
   workspace.rs/tabs.rs (`PaneContent`/`PaneState`/`PaneGrid`/`TabContent`/`Tab`/`Project`/
   `Workspace` all non-generic; `TerminalPane<S>` keeps its param as the payload type;
   `TerminalPane::new(session)` ctor hosts the one literal). All 9 births re-ordered
   spawn → `content.insert` → mount-the-id (leak-free by order); all close paths are the inverse
   (tree-remove → `release_view` — `#[must_use]` consumed — → one off-thread reap). RootView
   resolution helpers (the level where both fields live): `focused_terminal(_mut)`,
   `terminal_across_grids(_mut)`, `workspace_terminal(_mut)` (active-grid by-pane),
   `pane_of_content` (reverse lookup), `release_grid_terminals` (collect-ids → one reap thread).
   Pump rewritten two-phase (collect `(PaneId, ContentId)` → pump content → notify after the
   borrow ends); dead-pane close releases through the same funnel.
3. **Side-table re-key (instance-trackers → ContentId):** `agents`, `remotes`, `notify_ticks`
   decls, `last_agent`, `pending_agent_send`; pure consumers re-keyed (agent_view.rs `AgentRow.pane:
   ContentId`, `fleet_status_for(&[ContentId], …)`, sorts via `Ord` on the newtype; status_bar.rs).
   Renders resolve view→content at the edge: pane badges + status-bar focused lookups go
   `grid.terminal_id(pane)` → map; Fleet/⌘K-overlay row clicks go the other way
   (`pane_of_content` → `jump_to_pane`). VIEW/surface-keyed stayed pane-keyed as designed:
   `completion`, `MenuKind::Block.pane`, `tab_flashes`, the MCP surface handle. The #226
   notify-scrub gap is PRESERVED as-is (only the close paths that scrubbed before scrub now).
4. **Test-lane re-points:** headless `reap_sessions` swaps BOTH `shell` and `content` into one
   detached drop (the M1 miss from the inventory — without it every drive would leak/hang);
   integration.rs re-built over grid+registry (insert → split with the id; close → `release_view`
   returns the owned session, dropped inline by the caller's documented choice); drive helpers
   re-pointed to the RootView resolvers.
5. **Green:** `cargo check --workspace --all-targets` zero errors; `cargo clippy --workspace
   --all-targets` zero warnings; `cargo fmt` applied. No new `#[allow]`, no TODO markers.
   Persistence codec untouched (ids never serialize); grid_layout.rs untouched (zero signature
   change, as inventoried).

### Deviations from design (each with reason)
- **`SplitError` deleted outright** (not just re-shaped): `split_focused(axis, dir, ContentId) ->
  PaneId` is infallible once the content exists — the error type lost its only producer, and a
  dead pub type fails the gate. lib.rs export dropped.
- **`split_spawn_failure` test deleted** — its subject dissolved: spawn now happens BEFORE any
  tree op, so "split rolls back on spawn-Err" has no site; the fallible half lives in the birth
  flows (nothing inserted/mounted on Err), proven at the app level.
- **workspace.rs MockSession drop-counter harness retired** (design anticipated the split): the
  grid algebra is id-typed now — sessions can't be dropped by a tree the tree doesn't own. The
  teardown proof RELOCATES to registry units (#394 release-returns-content matrix) + the
  integration real-PTY lane; drop-count asserts re-expressed as view-id asserts (e.g. close
  returns the view for the CALLER to release). A comment at the old harness site records the
  relocation.
- **`set_content` re-signed** `-> Option<PaneContent>` `#[must_use]` (inventory M3: the in-place
  overwrite was the one path that dropped old content silently on the calling thread; now the old
  view id is handed back to the caller to release).
- **`agent_tail_lines(ContentId, n)`** (was by-pane + grid walk): the tail is a CONTENT read —
  `content.get(cid)` directly; the grid walk was only ever an ownership detour. Also cleaned an
  orphaned doc-block (a stale `refresh_agent_statuses` header) fused above it by an earlier edit.
- **Jump plumbing:** fleet rows carry ContentId, so jumps resolve `pane_of_content` →
  `jump_to_pane(pane)` at the click site — `jump_to_pane` itself stays pane-shaped (it IS a view
  operation: switch project/tab, focus the cell).

## Phase 3.5 — Inspect (2026-08-04, auto run)

Five parallel critics over the full diff, one lens each: **leak/release paths**, **1-view behavior
equivalence**, **general correctness**, **key-identity integrity**, **simplification/reuse**.
Load-bearing verdict: **zero logic bugs.** Every acquire has exactly one release on every path
(all 8 insert sites traced; all close paths verified one-thread-per-gesture, same tick); no
double-release is reachable (UI-thread serialization + monotonic never-reused ids + None-safe
release); the pump was verified arm-by-arm byte-equivalent against `git show HEAD`; persistence is
byte-identical (grid_layout.rs zero hunks, identical restore spawn inputs); the #226 scrub
inventory maps 1:1 old→new; every equality gate compares the same id kind with pre-migration
meaning; no ContentId reaches persistence or MCP.

### Findings ledger (finding → verdict → action)
| # | sev | finding | verdict | action |
|---|-----|---------|---------|--------|
| 1 | high | Fleet/⌘K rows + broadcast fan-out now order by ContentId (launch order), not PaneId (tab-block order) — visible when agents launch across tabs out of tab order | **ACCEPTED deviation** (3 critics concur it's real; deterministic either way; the design locked the Ord sort; a stored birth-pane would be wrong under #398 multi-view) | recorded below; docs now say "launch order" |
| 2 | low | `mcp_surface_index` omits kept-Exited agents whose pane the pump auto-reaped (old code advertised dead handles whose jump then failed) | **ACCEPTED deviation** — strictly more truthful enumeration | recorded below |
| 3 | low | Session-less payloads (containers, editor buffers) drop inline where the old container drop rode the reaper thread | **ACCEPTED** — TICKET-348 is PTY-scoped and every session still reaps off-thread; payloads that changed thread are memory-only | comments reworded truthful ("no PTY — editor buffers memory-only") |
| 4 | low | notify_ticks scrubbed at only one of five close paths | **REJECTED** — the #226 gap, preserved by explicit design (REQ-004) | none |
| 5 | high | 5 hand-rolled `terminal_id→content.get→as_terminal` chains duplicate the new helpers (4 block fns + cockpit Details) | FIXED | `workspace_terminal(_mut)` / reuse the bound `fcid` |
| 6 | med | `terminal_across_grids(_mut)` existed for one caller that is active-grid by construction (ssh compose) | FIXED | re-pointed to `workspace_terminal(_mut)`; pair deleted |
| 7 | med | `ContentRegistry::iter()` had zero callers and a doc claiming the pump uses it (also a cov-100 hole in waiting) | FIXED | deleted; `iter_mut` re-doc'd (its real consumer: the integration lane) |
| 8 | med | pump inner per-grid `collect::<Vec<_>>` — one alloc per grid per 16ms tick; misleading borrow comment | FIXED | flattened iterator chain; comment states the real reason (disjoint fields legal; snapshot = stable single pass) |
| 9 | med | fleet glyph built cids via `pane_ids()` (alloc + sort) per rail row per frame | FIXED | `states()` collect — order-free (the aggregate is a max) |
| 10 | med | naming lies: `AgentRow.pane: ContentId`, `agent_pane_ids() -> Vec<ContentId>` | FIXED | renamed `content` / `agent_content_ids` (+ docs, tests, both app.rs consumers) |
| 11 | med | four RootView field docs still taught pane-keying (agents/notify_ticks/remotes/last_agent) | FIXED | re-keyed wording |
| 12 | med | workspace.rs docs contradicted the migration (module "spawn seam"; `close()` teardown story; broken `focused_terminal` link) + app.rs dispatch_action spawn-failure claim | FIXED | all four reworded |
| 13 | low | duplicated MockSession-retirement comment (two stacked drafts) | FIXED | single version |
| 14 | low | stale pump comments ("grids_mut loop"; "resolve across all grids"; "borrows must not overlap") | FIXED | truthful rewrites |
| 15 | low | pending_agent_send probed the registry 3× per delivery | FIXED | 2 lookups, one less Option layer |
| 16 | low | birth/close artifacts: launch_agent bare block + `let _ = pane_id`; open_remote same; ⌘W dead `closing` binding; pump dead-arm re-derived the loop's own `cid` | FIXED | plain-statement splits; bindings deleted; `closed.is_some()` |
| 17 | low | `Content::Terminal(Box::new(TerminalPane::new(s)))` incantation ×10 | FIXED | `Content::terminal(session)` ctor — one birth idiom (8 app + 2 integration sites) |
| 18 | low | per-frame double cid resolve (agent+remote badges; status-bar labels) | FIXED | bind once (`badge_cid` / `fcid`) |
| 19 | low | `pane_of_content` doc named a non-caller (notify foreground check) and the body alloc+sorted `pane_ids()` | FIXED | doc names real callers (MCP index, Fleet/⌘K clicks); `states().find_map` body |

**Verification after fixes:** `cargo check --workspace --all-targets` zero errors; clippy zero
warnings; full suite re-run **1986/1986 passed** (5 skipped — the documented live-GUI/port
exclusions). No suppressions added anywhere.

### Deviations recorded by inspect (appended to the Phase 3 list)
- **Fleet/⌘K row + broadcast order = launch order** (ContentId Ord), not the old tab-block
  (PaneId) order. Visible only when agents are launched across tabs out of tab-creation order;
  sanctioned as the better ordering for a fleet; validate must not chase it as a regression.
- **`mcp_surface_index` no longer lists unmounted agents** — the old index advertised dead pane
  handles whose `surface_to_human` jump failed; now such agents surface nothing (failure moved
  from use-time to enumeration-time; handle semantics unchanged).
- **Non-PTY teardown thread**: containers and their editor/cockpit payloads drop inline at close
  (memory-only); the reaper contract is PTY-scoped and every `TerminalSession` still reaps
  off-thread — one thread per close gesture, unchanged counts.
- **Design's registry-iteration requirement narrowed**: the five app loops stayed VIEW-driven
  (grid walk + by-id resolve), so `iter()` was never needed and is gone; `iter_mut()` remains for
  the real-PTY integration lane's pump-all.

## Phase 4 — Validate (2026-08-04, auto run)

**Driven capture: N/A** — structural ownership migration, zero UI delta (React-first: N/A per the
spec); the behavior proof is the full-suite byte-identity + the headless drives, not pixels.
**React-first parity pair: N/A** — same reason, stated per the hook contract.

### Tests added (per the Phase 2 plan)
- **content.rs** (REQ-001): `payloadless_kinds_classify` (all 5 declared kinds, no catch-all),
  `non_terminal_has_no_terminal_payload` (both borrow flavors). The live-payload arms are proven
  where a session exists: the integration lane asserts `kind == Terminal` on a REAL spawned
  session; the seeded-restore headless drive asserts every restored content classifies Terminal.
- **content_registry.rs**: `iter_mut_visits_each_live_entry_once` (content-not-views iteration —
  a 2-view entry is one visit; mutations land on real entries).
- **headless_drive.rs seeded restore** (REQ-003 + REQ-001 app-level): restore rebuilds the
  registry — `content.len() == restored terminal count`, every id at `view_count == 1`, every
  content `kind() == Terminal`.
- **integration.rs** (REQ-002 real-PTY, written at implement, verified here): split → close →
  `release_view` returns the owned session; plus the REQ-001 kind assert.
- Re-expressed suites from implement stand as planned: workspace close-path units (id-bearing
  state returned), tabs/agent_view/status_bar re-keys with assertions unchanged.

### Runs (actual)
- `cargo nextest run --workspace`: **1989 tests run: 1989 passed, 5 skipped** (the 5 = the
  documented live-GUI/bound-port exclusions, pre-existing).
- `cargo test --workspace --doc`: ok (0 failures).
- `cargo mutants --list -f content.rs -f content_registry.rs`: 38 viable mutants (the real list);
  skip-detach re-check: 310 `mutants::skip` in the masked app.rs shim, ZERO in the pure files.
- #226 scrub landscape asserted unchanged at inspect (scrub inventory 1:1 old→new).

### Gate
First `--diff` run RED on two gates, both fixed at source (no baselines, no suppressions):
- gate:14 docs — two stale workspace.rs links (pub `TerminalPane` → private `PaneContent`;
  dead `Self::terminal` anchor from the retired accessor) → de-linked / re-anchored to
  `terminal_id`.
- gate:4 coverage — ONE missed line: content.rs `kind()`'s Terminal arm, whose only caller was
  the integration binary (not counted by the gate's cov run) → the in-lib seeded-restore drive
  now exercises it (the kinds_ok assert above).
Re-run: **GATE GREEN [diff] — 15 passed, 0 failed** (rustfmt, clippy -D, tests, coverage ≥100%,
MSI 100%, miri, audit, deny, machete, gitleaks, shellcheck, no-suppressions, source-bans, docs,
visual/AX). Receipt written by the green run.

### Pre-existing
None touched; no new exclusions; the 5 suite skips predate the ticket.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.

## Phase 5 — Complete (2026-08-04, auto run)
- CHANGELOG entry added (the migration, its invariants, and the three sanctioned deltas).
- `docs/marley_architecture/pane-composition-model.md` Q5 slice-2 → ✅ SHIPPED (M28 #396), with
  the resolution-helper/birth-idiom/re-key summary and the recorded deltas.
- Parity sync: N/A — no UI delta; `marley-web` and `MARLEY-PARITY.md` untouched (the ContentId ↔
  PaneItem vocabulary tie holds as documented).
- Knowledge captured (forge): failures `BF-gate-cov-integration-lane-blindspot-001` +
  `BF-doc-link-rot-under-migration-001`; prevention rules
  `PR-claude-key-migration-sweeps-ordering-surfaces-001` +
  `PR-claude-integration-only-coverage-fails-gate4-001`; AAR `a9c14f62` submitted (completed,
  effectiveness 5, 4 novel findings).
- Ticket: forge #396 → done; local doc → `tickets/closed/`, status closed.
- Archive: doc pair → `docs/planning/pipeline/completed/`.
