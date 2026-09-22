# 275-external-file-change — Notes

- **Forge ticket:** #275 f1b13464-6b27-4ce5-82fd-016a1b0c739c
- **AAR:** 09e38da9-301a-46dd-9f47-c31d19ef9bc3
- **Local ticket doc:** docs/planning/tickets/open/TICKET-275-external-file-change.md
- **Pipeline spec:** 275-external-file-change.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 5 of 10.
- **Recon (code-verified):** `OpenFile` (editor_surface.rs:18) has
  buffer/caret/saved_version/anchor/marked/nonce/scroll_px — no disk
  snapshot; dirty ⇔ `version != saved_version` (:65). `save_active`
  (app.rs:3967) writes then `active_mark_saved` — the pre-save check
  slots at its head; it is `mutants::skip` (fs shim). `status_flash` +
  `Flash::new` exist (app.rs:298/809 — the #203 PR: a pump/state
  change affecting render must set dirty/notify). gpui 0.2.2 exposes
  pub `Window::is_window_active()` (window.rs:1721) — the
  focus-REGAINED edge can live in the existing render choke (the
  #273/#272 idiom: remember `last_window_active`, act on the
  false→true transition) with NO new observer plumbing. The #221
  rounded-card idiom (app.rs:7370…) is the banner's render shape.
  #273's `clamp_scroll_px` is the scroll clamp; the #268 `next_nonce`
  mint is file-local (editor_surface).
- **Choke sites:** render-head edge detector (active file only, D4);
  the file-strip activate handler (`s.activate(i)` call sites ~6438);
  `save_active` head. No watcher thread (v1).
- **Risk flagged for design/critic:** the render-choke check does fs
  STATs in render — bounded to ONE stat per frame-where-edge-fires
  (the edge fires once per refocus, not per frame; tab-activate once
  per switch); assert no per-frame stat (REQ-005). Reload inside
  render must follow the borrow rules (compute → apply, the #273
  sync_editor_scroll shape) and set notify/dirty per the #203 PR.
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Pure seams (NEW `crates/marley_app/src/extchange.rs`; `mod extchange;` in lib.rs)
- `DiskState { Present { mtime: SystemTime, len: u64 }, Missing }`.
- `ExtAction { Noop, CleanReload, Conflict, Deleted }` +
  `ExtConflict { Changed, Deleted }` (the stored per-file state).
- `external_action(snapshot: Option<(SystemTime, u64)>, disk: &DiskState,
  dirty: bool) -> ExtAction` — the exact REQ-001 table: no snapshot →
  Noop; Missing → Deleted (dirty AND clean, D2); Present == snap →
  Noop; ≠ ∧ clean → CleanReload; ≠ ∧ dirty → Conflict.
- `focus_edge(last: bool, now: bool) -> bool` = `!last && now` — the
  regained-focus detector (4-row kill table).

### editor_surface.rs
- `OpenFile` += `disk: Option<(SystemTime, u64)>`,
  `conflict: Option<ExtConflict>`, `save_armed: bool` (all None/false
  at open until the app seeds the stat).
- `reload_active(&mut self, text: &str, disk)` — fresh
  `Buffer::from_text`, caret `min(new_len_chars)`, `scroll_px` clamped
  via the #273 `clamp_scroll_px` shape (rows from the new text),
  `clear_marked`, anchor=None, nonce RE-MINTED, saved_version = new
  version, disk snapshot set, conflict/armed cleared.
- Accessors: `active_disk/set_active_disk`, `active_conflict/
  set_active_conflict`, `active_save_armed/set…` (or one
  `active_ext_mut()` triple — pick at implement for borrow shape).
- **Scroll self-heal (zero new wiring):** the nonce re-mint makes the
  #273 render choke see an owner change → it parks to the DEAD old
  nonce (`park_scroll` miss = safe no-op) and restores the file's
  clamped `scroll_px` into the live handle. Critic to verify.

### app.rs (shim)
- Field `last_window_active: bool` (false at boot — the first active
  frame fires one harmless check against fresh snapshots).
- `disk_state(path) -> DiskState` — the ONE stat wrapper
  (fs::metadata → mtime+len; any error → Missing), `mutants::skip`
  shim (§14: the pure layer sees values; tests go through real
  tempdir fs).
- `check_active_file_external(&mut self, cx)` — stat → pure action →
  apply: CleanReload = fs::read_to_string (a read error after a
  Present stat = a race → flash + skip, never panic) →
  `reload_active` → flash "reloaded from disk"; Conflict/Deleted →
  set the per-file state; everything that changes state notifies
  (the #203 dirty PR).
- Choke wiring: (1) render head (next to `refresh_efind_matches`) —
  `let now = window.is_window_active(); if focus_edge(self.last_window_active, now) { self.check…(cx) } self.last_window_active = now;`
  (2) the file-strip activate listener → check after `s.activate(i)`;
  (3) `save_active` HEAD → check first, then the arm machine:
  `Some(Changed) && !armed` → arm + flash + NO write; else write →
  mark_saved → conflict/armed cleared → re-snapshot. Deleted needs no
  arming (nothing to clobber) — write recreates + re-snapshots.
- Banner (the #221 card idiom, in the code-view arm above content,
  when `active_conflict().is_some()`): Changed → "File changed on
  disk" + [Keep mine] [Reload]; Deleted → "File removed on disk — ⌘S
  recreates" + [Dismiss]. The click listeners call pub(crate) METHODS
  (`ext_keep_mine()` / `ext_reload()` / `ext_dismiss_deleted()`) so
  headless tests drive the handlers directly (the #198 precedent:
  listeners reuse tested methods). Keep-mine = conflict None + armed
  false + RE-SNAPSHOT current disk (D3). Reload = the CleanReload
  apply even though dirty (an explicit click). Deleted-dismiss =
  conflict None + `disk = None` (table: no snapshot → Noop → never
  re-flags; a later ⌘S-recreate restores tracking via its fresh
  snapshot — a recreated-by-agent file goes unnoticed until then,
  accepted v1, noted).

### §20 confirmation
The universal external-change convention (VS Code/JetBrains behavior
class, public product behavior): silent clean reload, non-modal dirty
warning, explicit-intent overwrite. Design matches: poll-at-
interaction (no modal), per-file banner, double-⌘S. No copyleft
source consulted. Holds.

### Manifest
| file | change |
|---|---|
| crates/marley_app/src/extchange.rs | NEW — the 2 enums + 2 pure fns + tests |
| crates/marley_app/src/lib.rs | `mod extchange;` |
| crates/marley_app/src/editor_surface.rs | OpenFile fields + reload_active + accessors |
| crates/marley_app/src/app.rs | last_window_active + disk_state + check + 3 chokes + arm machine + banner + methods |
| crates/marley_app/src/headless_drive.rs | REQ-002..005 flows |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | extchange: the FULL table — None-snap (dirty+clean)→Noop; Missing (dirty+clean)→Deleted; equal→Noop; mtime-only-differs clean→CleanReload; len-only-differs clean→CleanReload (the D1 tie-breaker — separate vectors kill each field's eq mutant); differs dirty→Conflict. focus_edge 4 rows. |
| REQ-002 | headless `ext_clean_reload_on_activate_headless`: 2 files open; rewrite file A on disk (LENGTH change — same-second mtime safe) while B active; activate A → text == disk, clean, caret-at-old-EOF clamped, flash set. |
| REQ-003 | headless `ext_conflict_banner_flows_headless`: dirty A, rewrite disk, activate-choke → Changed conflict, text untouched, dirty stays; `ext_keep_mine()` → dismissed + re-snapshot (next choke Noop); re-dirty + rewrite → conflict; `ext_reload()` → disk text, clean. |
| REQ-004 | headless `ext_save_arm_and_deleted_headless`: conflicted + REAL ⌘S → fs UNCHANGED + armed (flash); ⌘S again → fs == buffer, conflict cleared; remove_file + choke → Deleted; ⌘S → recreated. |
| REQ-005 | negative (same flow file): rewrite disk, render frames with the window CONTINUOUSLY active (no edge, no activate, no ⌘S) → no reload, no conflict. |
| uncoverable | the real macOS focus-regain END-TO-END (if the test context can't toggle window activation — the `focus_edge` unit + the shared `check_active_file_external` path carry it; stated explicitly). Banner PIXELS → driven capture, ENV-BLOCKED protocol if the machine is still locked. |

### Risks / decisions
- R1 — stats only on edge/activate/save frames (REQ-005 pins no
  per-frame stat); the check body is O(1) stat + optional read.
- R2 — same-mtime-same-len rewrite invisible (D1; the watcher upgrade
  closes it). Test fixtures always change length.
- R3 — reload clears marked/anchor (a stale IME span or selection
  into a shorter buffer would be UB-ish; clear both).
- R4 — the efind memo self-heals on reload (nonce+version key moves →
  render-head refresh recomputes) — the #272 mechanism, no wiring.
- R5 — a read that fails between stat and read (agent mid-write race)
  flashes + skips; the next choke retries. Never a panic (§14).

## Phase 3 — Implement
- **extchange.rs (NEW):** DiskState/ExtAction/ExtConflict,
  `external_action` (the let-else None→Noop + Missing→Deleted +
  tuple-eq → Noop/dirty-split), `focus_edge`, `DiskState::snapshot()`.
- **editor_surface.rs:** OpenFile += disk/conflict/save_armed;
  `open()` now returns `bool` (newly-appended vs switched — the app
  seeds vs checks); `set_active_conflict(None)` also DISARMS (the
  second-press license dies with the banner); `arm_active_save`;
  `reload_active(text, disk)` — fresh buffer (undo history restarts —
  an external epoch, deliberate), caret min-clamped, anchor/marked
  cleared, nonce re-minted, clean, snapshot set, banner cleared.
- **tabs.rs:** `open_or_switch_code` forwards the bool (new tab =
  true).
- **app.rs:** `last_window_active` field; `disk_state` stat shim (any
  error → Missing); `check_active_file_external` (stat → table →
  apply; read-race flashes + skips); `reload_active_from_disk` shared
  by CleanReload and the banner's Reload; the 3 banner handler
  methods (keep-mine re-snapshots D3 / reload / dismiss-deleted
  untracks); `save_active` rebuilt with the pre-save choke + the
  Changed-arm machine + post-write re-snapshot; the render-head
  focus-edge choke; the strip-activate choke; open_file_in_viewer
  seeds-or-checks; the floating #221 banner card (top-right overlay,
  non-modal, buttons → the pub(crate) methods).
- **Deviation from design (simplification):** `reload_active` does
  NOT clamp `scroll_px` itself — the #273 sync clamps on restore
  against the CURRENT row count, and the nonce re-mint forces exactly
  that restore path. One clamp authority.
- **IMPLEMENT BUG (caught in Phase 3, fixed at source):** the render
  edge initially ran BEFORE the launcher branch and called
  `active_project()` — which PANICS on the empty project set; the
  virgin-boot headless test hung the whole suite (the documented
  panic-masks-as-hang class: unwind skips reap_sessions). The #234
  comment five lines below the insertion states the invariant
  verbatim ("before ANY workspace() access, each of which panics on
  an empty project set"). Fix: the edge call is guarded on
  `project_count() > 0` (launcher frames have no editor anyway).
  Inspect records the failure; the suite pinned it (981/981 after).
- check + clippy `-D warnings` + fmt clean; suite 981/981.

## Phase 3.5 — Inspect
### Critic findings ledger (2 critics: state machine ×10 hunts w/ APFS
probes; choke/borrow/render ×10 hunts w/ gpui-source verification)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| F1 | [HIGH, BOTH critics independently] Boot-RESTORED files never seed a snapshot (`from_files` → `disk: None`) → every choke Noops → the feature is silently OFF for the steady-state working set and ⌘S silently clobbers agent writes — the exact headline guarantee. | REAL | The restore arm stats each file BEFORE reading and threads the snapshot through the new `RestoredFile` row type into `from_files`. Phase 4 adds the restore-then-rewrite flow. |
| F2 | [MED] Snapshot-taken-AFTER-content at reload/open (+ save inherently): a write racing into the gap pairs OUR stale text with the racer's fresh snapshot → permanently invisible divergence. | REAL | Stat-BEFORE-read at reload + open + the restore arm (the safe direction — a racing write leaves the snapshot older → the next choke converges); save keeps write-then-stat (needs OUR mtime) + a post-write LEN cross-check that flags `Changed` instead of clearing when a racer landed (same-len residue = the accepted D1 class). |
| F3 | [LOW] The ⌘S arm license isn't bound to the disk state it warned about (agent writes AGAIN between presses → press 2 overwrites W2 under W1's warning). | REAL, follow-up | Acknowledgment-binding (stamp the observed (mtime,len) into the arm/keep-mine and re-warn when it moves) — one follow-up ticket with F4a. Moderated by the banner staying visible. |
| F4 | [LOW] (a) keep-mine acknowledges CLICK-time disk sight-unseen (same class as F3). (b) keep-mine while the file just vanished → snapshot None → untracked forever. | (a) follow-up · (b) REAL | (b) fixed: Missing keeps the OLD snapshot so the next choke flags Deleted. |
| F5 | [LOW] A content-identical rewrite (idempotent formatter, touch-storm) cost a full epoch: undo history destroyed + re-parse + flash, for zero change (APFS mtime moves every write). | REAL | reload compares the read text to the buffer first — identical → just re-snapshot, no epoch, no flash. |
| F6 | [LOW] Banner Reload on a dirty buffer discards edits with no undo escape and a bland flash. | REAL (deliberate epoch, better said) | Flash now reads "Reloaded from disk — unsaved edits discarded" when the buffer was dirty. |
| F7 | [LOW] Open-path vanish-between-read-and-stat → None → forever-Noop. | CLOSED by F2's reorder (stat-first: a vanish before the read simply fails the open). |
| C-B | [MED] The banner card body was CLICK-THROUGH over the editor's caret/drag listeners (only the buttons stopped propagation) — the house #185 rule mandates blocking for non-modal overlays on scroll areas. | REAL | `.block_mouse_except_scroll()` on the card root (wheel stays live beneath). |
| C-T | [MED] Editor-TAB activation (⌘]/⌘[/⌘1-9/rail jump/workspace cycle/close-reveal) was NOT a choke — the editor is a full-screen TAB, so returning to it inside the window is the highest-traffic reveal and nothing fired. REQ-005's "exactly three" contradicted D4's own rationale. | REAL | The check added at all 7 reveal sites (next/prev-tab, switch-tab-N, next/prev-workspace, jump_to_pane, close_editor_file's not-drained branch — it early-returns free for terminal tabs); REQ-005 amended to "the choke set". |
| C-P4 | [MED, Phase-4 directive] gpui's TestWindow::is_active is HARDCODED false — the focus edge (and its launcher guard) is dead in every headless frame; my Phase-3 note "the suite pinned it" was wrong in spirit (the virgin-boot test passes because the edge never fires). Activation IS drivable: `activate_window()` / `VisualTestContext::deactivate_window`. | REAL (test-plan) | Phase 4 MUST: (a) virgin boot + activate → park (the REAL launcher-guard pin); (b) REQ-002's focus-regain via deactivate/activate; (c) REQ-005's negative consumes the boot edge FIRST (activate once) so it can't pass vacuously. |
| — | [INFO] APFS mtime = ns resolution (probe: 0 collisions in 1000 same-length rapid rewrites) — the D1 blind spot is ~nonexistent on target; len stays as belt vs mtime-restoring tools. `extchange` unit tests land in Phase 4 (the manifest's "+ tests" was premature at implement). | noted | — |
| — | IMPLEMENT-phase bug (pre-critic, caught by the hung suite): the render edge ran before the launcher branch → `active_project()` PANICS on the empty set → the virgin-boot headless test wedged the suite (panic-masks-as-hang). Guarded on `project_count() > 0`. The #234 invariant comment sat five lines below the insertion. | REAL | failure-record below; the REAL regression pin is C-P4(a). |
| — | Cleared (verified): ⌘S/open-file UNREACHABLE on the launcher (the key listener attaches only on shell frames — conclusive trace); REQ-005 stat sites exactly the choke set; borrow/TOCTOU clear (no await, single-threaded); banner only renders in the code-view arm (a terminal tab with a conflicted background file floats nothing) + follows switches; notify discipline clear on every path; save error-path flashes correctly; per-OpenFile state dies with close (no leaks); tabs.rs `unwrap_or(false)` arm dead-defensive; scroll one-clamp-authority holds; `len_chars` clamp char-safe. | — | — |

## Phase 4 — Validate
- **extchange units (3):** the FULL REQ-001 table (None×3 incl.
  Missing-untracked; Missing→Deleted both dirtiness values;
  equal→Noop both; mtime-only + len-only vectors kill the tuple-eq
  per field; the dirty split both directions), focus_edge 4 rows,
  snapshot round-trip. Viable mutant set: 6 (3 unviable —
  ExtAction/SystemTime lack Default) — all covered.
- **Surface units (2):** `t275_reload_active_epoch_and_clamps` (fresh
  buffer, caret 17→3 clamp, anchor/marked cleared, nonce re-minted,
  clean, snapshot set, banner+arm cleared, undo history restarted)
  and `t275_from_files_seeds_disk_and_conflict_clear_disarms` (the F1
  thread + the license-dies-with-the-banner rule).
- **Headless flows (4, the #264 lane):**
  `ext_clean_reload_and_focus_edge_headless` — activate-choke reload
  (text/clean/flash/caret-clamp) + the ACTIVATION-DRIVEN focus edge
  (deactivate → rewrite → `activate_window` → reload; the C-P4
  directive — TestWindow is inactive by default so this is the only
  live-edge exercise) + the REQ-005 poll-only negative (edges
  consumed, idle frames, disk change UNDETECTED);
  `ext_conflict_banner_flows_headless` — flag/keep-mine-no-reflag/
  D3-reflag-on-next-change/explicit-reload via the pub(crate) handler
  methods (the #198 listener-reuse rule);
  `ext_save_arm_and_deleted_headless` — REAL ⌘S keystrokes: first
  press arms (fs byte-identical), second overwrites, delete→Deleted
  banner, ⌘S recreates + re-tracks;
  `ext_launcher_guard_activation_headless` — the REAL implement-bug
  pin: virgin boot + deactivate/activate → the guarded edge does NOT
  panic on the zero-project launcher.
- **Driven capture: ENV-BLOCKED (machine locked; black 8×8 RGB probe
  again)** — the banner card's PIXELS join the unlock re-verify batch
  (#272 efind bar · #276 indent · #277 CJK advance · #275 banner);
  the banner's full state machine is headless-proven; the card render
  is the shared #221 idiom.
- **Gate:** first `--diff` run RED — clippy dead-code on the test-only
  flash accessor (→ `#[cfg(test)]`, the #273 idiom) + gate:5 MSI with
  3 missed: the accessor's constant-Some body mutants (removed from
  the set by the same cfg(test); the reload flow also asserts the
  EXACT message now) and `ext_dismiss_deleted` with a deleted body —
  NO test drove the Dismiss handler (the deleted flow now flags →
  dismisses → proves the dismissal STICKS through a re-check → ⌘S
  recreates). Re-run: **GATE GREEN [diff] — 15/15 PASS** (suite
  995/995 in-gate; coverage 100; MSI 100 with timeouts-as-caught per
  the Stryker convention; no pre-existing failures; nothing
  excluded).

## Phase 5 — Complete
- CHANGELOG under "### Added"; editor.md gains the full
  external-change section (snapshot discipline, the choke set, the
  epoch semantics, the banner rules, testability).
- AAR 09e38da9 submitted (completed; materialized:
  BF-claude-restore-path-skipped-new-field-seeding,
  PR-claude-restore-is-a-second-constructor-001).
- Follow-up ticket **#284** (acknowledgment-binding — F3/F4a) minted,
  on the shelf with #282/#283.
- Forge #275 closed (done); local ticket → closed/.
- Lessons (in the AAR): (1) restore is a second constructor —
  enumerate every from_* builder when adding a seeded field; (2)
  stat-before-read is the safe race direction everywhere content and
  metadata are captured together; (3) TestWindow reports inactive —
  a window-activation feature is dead headless unless the test
  drives activate_window (the "suite pinned it" claim was wrong until
  the activation flow existed); (4) the panic-masks-as-hang class
  struck again at the render head — the #234 launcher invariant
  applies to EVERY new pre-launcher-branch line.
