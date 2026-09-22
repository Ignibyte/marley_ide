# 273-scroll-memory — Notes

- **Forge ticket:** #273 0d3556f7-8948-4f21-904b-8f9a5dbb83bf
- **AAR:** 1ca979c4-1d8b-4994-9e56-33d2a8e332cb
- **Local ticket doc:** docs/planning/tickets/open/TICKET-273-scroll-memory.md
- **Pipeline spec:** 273-scroll-memory.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 1 of 10 (run order:
  273→272→276→277→275→274→278→281→279→280 — this ships the scroll
  mechanism #272 consumes next).
- **API ground truth (vendored gpui 0.2.2, read this session):**
  - `UniformListScrollHandle` wraps `Rc<RefCell<UniformListScrollState>>`
    with a PUB `base_handle: ScrollHandle` (uniform_list.rs:80/110).
  - `logical_scroll_top_index()` is `#[cfg(any(test, feature =
    "test-support"))]` (uniform_list.rs:218) — TEST-ONLY; production
    capture goes through `base_handle.offset() -> Point<Pixels>`
    (div.rs:3083, pub) and restore through `set_offset` (div.rs:3211,
    pub). Pixel restore is EXACT (mid-row survives).
  - `scroll_to_item(ix, ScrollStrategy)` (uniform_list.rs:145) defers to
    the next layout; NON-STRICT = no-op when visible; `Center` places
    mid-viewport — the caret-follow/find-next semantic. Strict +
    offset variants exist if ever needed.
  - `ScrollHandle::logical_scroll_top()` (div.rs:3217) is pub but walks
    `child_bounds`, which a VIRTUALIZED uniform_list does not populate
    per-row — do NOT use it for capture; offset() is the truth.
- **Switch sites (grepped):** the file-tab strip click (`s.activate(i)`
  app.rs:5964), `open_file_in_viewer` (app.rs:2448; callers 1823/2186/
  2395/3073/7421 all funnel through it), the tab-strip × close. The
  #267/#268 precedent: editor_surface's activate/open/close are the
  surface-side choke points; the HANDLE lives on RootView, so the
  capture/restore wrapper is app-side around those calls.
- **Forge recall:** fresh from this session — the #266 F4 ledger row
  (the trade being healed), PR-claude-cache-keys-need-identity (the
  nonce marks buffer generations for the clamp), the #264 headless lane
  (logical_scroll_top_index IS readable in tests — marley_app has gpui
  test-support as a dev-dep).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Approach — ONE render-time choke point, not site enumeration
Reading the sites showed the enumeration trap: tab-strip activate
(app.rs:5964), open_file_in_viewer (2448 → open_or_switch_code →
surface.open), close_editor_file (3428 → s.close), PLUS project switches
(a different project's surface + file under the SAME shared handle),
workspace switches, and the restore path — every one changes the active
file identity, and missing any leaks offsets across files. Instead:
**the render path detects the transition** (the #268
`refresh_syntax_cache` precedent — same place, same reason):

- NEW `RootView.scroll_owner: Option<u64>` — the NONCE of the file whose
  offset the shared handle currently holds.
- In the code-view render arm (right beside `refresh_syntax_cache`,
  BEFORE EditorDraw borrows): `sync_editor_scroll()` — if the active
  file's nonce != scroll_owner: (1) PARK the handle's current
  `base_handle.offset().y` into the OLD owner's `OpenFile.scroll_px`
  (found by nonce across ALL projects' surfaces; owner closed → no-op,
  its memory dies with it); (2) RESTORE the new file's `scroll_px` via
  `base_handle.set_offset(point(0, clamped))`; (3) scroll_owner =
  Some(new nonce). Same-nonce frames: no-op (the handle is the live
  truth while a file is active — D2 survives, generalized).
- No capture at switch sites at all — every path (strip click, open,
  close, project/workspace switch, restore) funnels through the next
  render frame by construction.

### Pieces
| piece | what |
|---|---|
| `OpenFile.scroll_px: f32` (+ `scroll_px()/set_scroll_px()` on the surface by NONCE + active) | the parked value; NOT persisted (D4) |
| `EditorSurface::park_scroll(nonce, px) -> bool` + `active_scroll_px()` + existing `active_nonce()` | surface accessors (pure, unit-tested) |
| pure `clamp_scroll_px(px: f32, total_rows: usize, cell_h: f32) -> f32` | clamp into `[-(rows-1)*cell_h, 0]` (offsets are NEGATIVE scrolled down; gpui re-clamps against real content at paint — this guard just prevents a blank over-scroll after shrink, REQ-004) |
| `RootView::sync_editor_scroll()` (shim, masked) | the transition detector above |
| `RootView::scroll_editor_to_row(row)` — **pub** | `editor_scroll.scroll_to_item(row, ScrollStrategy::Center)` (non-strict — D3). Pub deliberately: it IS the shipped mechanism for #270/#272/#212/#213, exercised by the REQ-005 headless test now (pub + tested = no dead-code, no suppression — §0) |

### File manifest
| file | change |
|---|---|
| crates/marley_app/src/editor_surface.rs | OpenFile.scroll_px + park_scroll/active_scroll_px accessors + tests |
| crates/marley_app/src/code_view.rs | pure `clamp_scroll_px` + tests (lives with the other scroll math) |
| crates/marley_app/src/app.rs | scroll_owner field/init; sync_editor_scroll() called in the code-view render arm; pub scroll_editor_to_row |
| crates/marley_app/src/headless_drive.rs | the REQ-001/002/003/005 flow tests |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | headless: open A (120-line fixture) + B; scroll A via `base_handle.set_offset` (simulating wheel) or scroll_editor_to_row; switch B; switch back A → offset restored (read `logical_scroll_top_index()`/offset — test-support) |
| REQ-002 | headless: same flow through open_file_in_viewer for a NEW third file — outgoing parked, fresh file at top |
| REQ-003 | headless: close the active file-tab → survivor's offset restored |
| REQ-004 | pure clamp units (0 rows, 1 row, exact fit, shrink-below-saved, positive px input clamps to 0) + headless: park deep offset on A, shrink A's buffer via edit, switch back → clamped (no blank) |
| REQ-005 | headless: scroll_editor_to_row(100) on the 120-line file → top index moves near-center; call again with a VISIBLE row → top index unchanged (non-strict no-op) |
| REQ-006 | existing suite + the #246 driven capture in P4's capture set |
| surface | editor_surface units: park by nonce (hit/miss/closed), scroll_px round-trip, fresh file = 0.0 |
- Uncoverable: sync_editor_scroll's handle borrow + set_offset live in
  the documented app.rs exclude — asserted end-to-end by the headless
  flows (which run the REAL render via the #264 lane).

### Risks / decisions
- R1 Pixel (not row) storage — D1 stands; set_offset restores exactly;
  the clamp guards shrink.
- R2 The park-by-nonce lookup walks every project's surface files —
  O(open files), single-digit; fine.
- R3 scroll_owner survives the file's CLOSE (dangling nonce) — park hits
  the no-op branch; the next transition overwrites. Unit row covers it.
- R4 A terminal-tab interlude (editor not rendered) parks nothing — the
  handle keeps the file's live offset and the owner nonce still matches
  on return. Correct by construction; headless row pins it.
- §20 re-confirmed: N/A — Marley's own behavior restoration; gpui public
  API only.

## Phase 3 — Implement
- **Built to manifest:** `OpenFile.scroll_px: f32` (0.0 fresh) +
  `active_scroll_px()` + `park_scroll(nonce, px) -> bool` (find-by-nonce;
  miss = closed owner = false); pure `code_view::clamp_scroll_px(px,
  total_rows, cell_h)` (clamp into `[-(rows-1)·cell_h, 0]`, negative
  cell_h guarded via max(0.0), positive px → 0); app.rs —
  `scroll_owner: Option<u64>` field/init, `sync_editor_scroll()` (owner
  transition: park the handle's `base_handle.offset().y` into the old
  nonce via a labeled `'park` loop over `project_count()`/`tab_mut`
  index accessors [no projects_mut/tabs_mut iterators exist — deviation
  from the design's sketch, same semantics], restore
  `set_offset(point(0, clamped))`, take ownership; same-owner = no-op)
  called in the code-view render arm right after `refresh_syntax_cache`;
  **pub** `scroll_editor_to_row(row)` → non-strict
  `scroll_to_item(row, ScrollStrategy::Center)`.
- **Deviations:** the park loop uses index accessors (above); everything
  else per design.
- **Verification:** check clean; fmt; clippy -D warnings clean; full
  `cargo nextest run --workspace` **952/952** (no regression; the new
  paths get their tests in Phase 4).

## Phase 3.5 — Inspect
- **Critic run:** 1 deep critic (probe crate for the clamp fenceposts;
  every dispatch/notify path enumerated; gpui claims cited to exact
  lines) + a parallel self-review that found+fixed the deferred-leak
  BEFORE the report landed (the critic then critiqued the LIVE revised
  code and confirmed the fix correct).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| S1 (self) | MED-class, pre-empted | a pending `deferred_scroll_to_item` from the OUTGOING file survives the transition and fires on the NEW file's next layout (gpui applies the deferred AFTER the base offset and it WINS — uniform_list.rs:393-448) | REAL (critic confirmed the mechanics) | FIXED during inspect: the transition takes ONE `borrow_mut` that clears the deferred AND set_offsets (no re-entrant borrow — three distinct RefCells, critic-verified div.rs:3041/3068) |
| F1 | LOW (forward-design) | the unconditional deferred-clear means an "open file + scroll_editor_to_row" in ONE handler gets wiped next frame — sync can't tell a stale outgoing deferred from a fresh incoming one. Zero callers today (grep-verified) — latent, bites #272/#212/#213 | REAL, latent | doc CONSTRAINT added to the helper (jump on a later frame / park the row on the OpenFile); carried in the notes for the consumer tickets |
| F2 | INFO | clamp NaN totality holds only by unreachability (parked = f32::from(real Pixels); probe: NaN propagates without panic, the bound can never be NaN via the cell_h guard) | verified | doc already states §14 totality; no change |
| F3 | INFO | the EditorDraw None arm (app.rs:5972) is pre-#273 dead-from-the-tab-caller (serves the #246 PANE path) | pre-existing | no action (the #266 F5 note already records it) |

- **Verified clean (critic, cited):** transition completeness — ALL
  switch paths land on notify → the render arm → sync (strip click,
  file ×, all 5 open_file_in_viewer callers, rail project/tab/pane,
  ⌘⇧[/], tab chords, agent jump, restore); the editor-tab-REMOVED paths
  dangle scroll_owner with a DEAD nonce benignly (nonces never reused →
  the park scan misses → discarded → next frame re-owns); same-file-two-
  projects = two nonces = independent memories by construction; sign/
  units (set_offset doc'd negative-down, no internal clamp, Pixels↔f32
  lossless); over-scroll CANNOT paint blank (same-frame prepaint clamps
  the tracked cell at div.rs:1746-1758 + the in-list clamp at
  uniform_list.rs:376-383 — the code_view doc's claim confirmed at exact
  lines); clamp fenceposts total (rows 0/1, cell_h 0/neg/NaN, ±inf,
  usize::MAX); cell_h/row domains match the real list (fallback.h IS
  the row line_height and measure_item source); borrow discipline (the
  park read is a dropped temporary; the write block holds the OUTER
  cell while set_offset borrows the handle's OWN cells); non-strict
  Center semantics cited; NO mutants::skip detach (app.rs still 17
  listed, none in the new fns).
- **Phase-4 kill list (critic, 12 viable / 0 equivalent):** T1
  in-range identity kills all 6 clamp mutants; T2 below-bound pins the
  bound formula; T3 positive→0; T4 rows∈{0,1} + cell_h∈{0,-3} → 0;
  T5 the two-file park/read-back (-42.0 — the value mutants need the
  exact read-back, the #200 Some(1) lesson); T6 unknown-nonce miss
  (false + neighbors untouched).
- **Post-fix verify:** check clean on the live tree (critic re-ran).

## Phase 4 — Validate
- **Kill-list tests (all 12 viable killed):** `clamp_scroll_px_kill_list`
  (T1 in-range identity kills all 6 clamp mutants; T2 bound formula incl.
  −∞; T3 positive→0; T4 rows∈{0,1} + cell_h∈{0,−3});
  `park_scroll_by_nonce_round_trips_and_misses_safely` (T5 exact −42.0
  read-back through activation + neighbor isolation; T6 unknown-nonce
  miss touches nothing).
- **Headless flows (the #264 lane):**
  `per_file_scroll_memory_survives_switches_headless` — REQ-005
  off-screen row 100 centers (offset < −100) + visible-row no-op;
  REQ-002 open B parks A, B at top; REQ-001 back to A restores
  PIXEL-EXACTLY; REQ-003 closing B leaves A untouched.
  `parked_scroll_clamps_after_buffer_shrink_headless` — REQ-004: A
  parked deep, gutted to 1 line while inactive, restore clamps into
  range.
- **A REAL environment bug found + fixed en route:** the full suite
  WEDGED on `boot_headless_virgin_dir_is_the_launcher` (0.02s CPU for
  30+ min). Stack-sampled live: `RootView::new_in` → scope-end
  `drop_in_place<TerminalSession>` — the VIRGIN-boot arm dropped its
  unused boot PTY **on-thread** (the exact pre-existing hazard ticket
  #271 notes; the restored arm already reaps off-thread), and orphaned
  zsh children from an earlier kill made the child-reap block
  indefinitely. FIXED at source in this diff (the #247 launcher arm now
  `take()`s + reaps off-thread, mirroring the restored arm). Also: my
  FIRST draft of the shrink test hung the lane by tripping the F1
  constraint itself (open+jump in ONE closure → the transition wiped
  the jump → assert panic → unwind skipped reap_sessions → the drop
  block masked the failure as a hang) — the test now opens first and
  jumps on a later frame, exactly as the constraint documents.
- **Full suite:** **956/956** (5 skipped) post-fix.
- **Driven captures (bundled, PNGs read):** `273-1` movement.rs
  wheel-scrolled deep (rows 101-147); `273-2` switch to .mcp.json —
  its OWN top-of-file position; `273-3` switch back — the IDENTICAL
  rows 101-147 restored (pre-#273 the offset carried across). REQ-001/
  002 on pixels; REQ-006 rides the unchanged #246/terminal surfaces in
  the same captures.
- **Gate:** **GATE GREEN [diff] 15/15** first post-fix run (receipt
  written).

## Phase 5 — Complete
- (pending)
