# Extchange ack binding — Notes

- **Forge ticket:** #284 (c439b877-1e0d-4478-b359-f8b11a563a91)
- **AAR:** 18068cb8-5856-43a4-8231-ca2ee680c1f4
- **Local ticket doc:** docs/planning/tickets/open/TICKET-284-extchange-ack-binding.md
- **Pipeline spec:** 284-extchange-ack-binding.spec.md

## Phase 1 — Plan
- **Request:** #284 (M17-followup, from #275 inspect F3/F4a) — the ⌘S-arm + Keep-mine acknowledge the
  disk state at press/click time, not the one the warning described; a newer write between the warning
  and the ack is overwritten/re-snapshotted sight-unseen. Bind acks to the observed `(mtime,len)`.
- **Classification:** work pipeline (bug). PURE staleness decision (cov/MSI 100) + one `OpenFile` field +
  2 `mutants::skip` shim handler edits + headless flows.
- **Forge recall:** `bulletin-list` → none. #275's own D3 ("the next external change re-flags") is the
  principle being extended; the F4b inspect fix (keep the old snapshot if the file vanished) is precedent.
- **Discovery (grounded):**
  - `extchange.rs` — the pure layer: `DiskState{Present{mtime,len},Missing}`, `ExtAction`, `ExtConflict{
    Changed,Deleted}`, `external_action(snapshot,disk,dirty)`, `focus_edge`, `DiskState::snapshot`.
  - `editor_surface.rs:41 OpenFile.disk: Option<(SystemTime,u64)>` — the LOAD/SAVE snapshot (NOT the
    conflict-observed state); `:44 conflict: Option<ExtConflict>`; `:47 save_armed: bool`;
    `set_active_conflict` clears `save_armed` when conflict→None; `arm_active_save`; `set_active_disk`.
  - `app.rs:4314 save_active` (`#[cfg_attr(test, mutants::skip)]`): `check_active_file_external` first, then
    `if conflict==Changed && !armed { arm + flash + return }` (1st press), else WRITE + re-stat + racing-check
    (4342-4367). **The armed 2nd press writes with NO check that the disk moved since the arm** = F3 leak.
  - `app.rs:4251 ext_keep_mine`: `snap = disk_state(path).snapshot()` (the CURRENT disk) → clear conflict +
    `set_active_disk(snap)`. **Re-snapshots whatever is on disk NOW, even a W2 the user never saw** = F4a leak.
  - `app.rs:4177` (Conflict) + `4363` (racing) set `Changed`; `7112` renders it; `7115` Keep-mine button →
    `ext_keep_mine`. `ExtConflict::Changed` used at ~6 sites → keep it a unit variant (D2: separate field).
- **Decisions:** D1–D3 (spec). The observed pair is set wherever `conflict=Changed` is flagged (4177, 4363)
  and consumed by save_active's armed path + ext_keep_mine via the pure `acknowledgment_is_stale`.
- **§20:** N/A — Marley poll-at-choke model; extends #275 D3.

## Phase 2 — Design

### Architecture / approach
**Why TWO stored pairs (the key insight):** `save_active` runs `check_active_file_external` FIRST, which
re-stats and REFRESHES the banner's observed state to the CURRENT disk. So a single `conflict_observed`
would be clobbered to W2 before the arm's 2nd-press check could compare against W1. Therefore:
- **`OpenFile.armed_at: Option<(SystemTime, u64)>`** (REPLACES `save_armed: bool`; `Some` ⇔ armed, carrying
  the disk state at ARM time — untouched by later re-stats). Consumed by `save_active`'s 2nd press.
- **`OpenFile.conflict_observed: Option<(SystemTime, u64)>`** (NEW) — the disk state the `Changed` banner
  currently describes; refreshed wherever `Changed` is flagged. Consumed by `ext_keep_mine`.

**PURE seam** `extchange.rs`:
`pub fn acknowledgment_is_stale(acked: (SystemTime,u64), now: &DiskState) -> bool` —
`Present{mtime,len} => (mtime,len) != acked; Missing => true`. cov/MSI 100.

**editor_surface.rs** (`OpenFile`):
- `save_armed: bool` → `armed_at: Option<(SystemTime,u64)>`. `active_save_armed() -> bool` kept as
  `armed_at.is_some()` (render/existing tests unchanged); add `active_armed_at()` + `arm_active_save(at)`
  (now takes the disk pair). `set_active_conflict(None)` clears `armed_at` + `conflict_observed` too.
- NEW `conflict_observed` + `active_conflict_observed()` + `set_conflict_observed(pair)`.

**app.rs** (both `mutants::skip` shims):
- Where `Changed` is flagged (4177 conflict, 4363 racing): after `set_active_conflict(Some(Changed))`, also
  `set_conflict_observed(disk_pair)`.
- `arm_active_save` call in `save_active` (4335): pass the current disk pair → `armed_at = Some(now)`.
- `save_active` armed-2nd-press: `let now = Self::disk_state(&path)`; if `Some(acked) = active_armed_at()`
  and `acknowledgment_is_stale(acked, &now)` → DISARM (`armed_at=None`, keep `Changed`, refresh
  `conflict_observed=now.snapshot()`), flash "File changed again — ⌘S to overwrite", `return` (NO write).
  Else fall through to the existing write.
- `ext_keep_mine`: `let now = Self::disk_state(&path)`; if `Some(acked)=active_conflict_observed()` and
  `acknowledgment_is_stale(acked,&now)` → REFRESH `conflict_observed=now.snapshot()` (keep the `Changed`
  banner), `return` (no dismiss/re-snapshot). Else the existing dismiss + `set_active_disk`.
- §20 N/A confirmed — extends #275's own D3.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/extchange.rs` | Add `acknowledgment_is_stale` (pure) + its unit test. |
| `crates/marley_app/src/editor_surface.rs` | `save_armed:bool`→`armed_at:Option`; add `conflict_observed` + accessors/setters; `set_active_conflict(None)` clears both. |
| `crates/marley_app/src/app.rs` | Set `conflict_observed` where `Changed` is flagged; `arm` carries the disk pair; `save_active` 2nd-press + `ext_keep_mine` re-warn on stale. |
| `crates/marley_app/src/headless_drive.rs` | W1-arm/W2-lands/press-2-re-warns + Keep-mine-re-flags flows (real tempdir writes + focus-edge). |

### Regression Test Plan
| Test | Proves | REQ |
|---|---|---|
| `extchange::acknowledgment_is_stale_cases` | (acked, Present same)→false; (mtime≠)→true; (len≠)→true; (Missing)→true | REQ-001/003/005 |
| `headless … arm_then_newer_write_re_warns` | dirty + W1 → banner; ⌘S arms; W2 written; ⌘S again → file on disk STILL W2 (buffer NOT written), banner still `Changed`, disarmed | REQ-001 |
| `headless … arm_unchanged_disk_overwrites` | arm, no W2, ⌘S again → disk == buffer, banner cleared | REQ-002 |
| `headless … keep_mine_newer_write_re_flags` | banner W1; W2 written; Keep-mine → banner still `Changed` (not dismissed), `conflict_observed` refreshed | REQ-003 |
| `headless … keep_mine_unchanged_dismisses` | banner W1; no W2; Keep-mine → banner None + snapshot updated | REQ-004 |

- **trybuild:** none. **Uncoverable:** the 2 handlers are `mutants::skip` fs shims — the headless flows (real
  tempdir file re-writes via `std::fs::write`, focus-edge via `activate/deactivate_window`) are the proof.

### Risks / decisions
- **`save_armed:bool`→`armed_at:Option` ripple:** kept `active_save_armed()` as `armed_at.is_some()` so the
  banner render + the #275 `editor_surface` tests (759-810) compile unchanged; only the arm SET site passes
  a pair now.
- **Disarm-on-stale (D3):** a stale 2nd ⌘S DISARMS (needs a fresh ⌘S to re-arm) rather than silently
  re-arming — the safe default; the user re-confirms against the new state. Same for Keep-mine (re-flag, the
  user re-decides).
- **conflict_observed refresh sites:** must cover BOTH 4177 (focus/interaction conflict) and 4363 (post-save
  racing) or a racing-write banner would carry a stale observed. Implement greps every `Changed` set site.

## Phase 3 — Implement
`cargo check --workspace` green. Built to the manifest, no deviations of substance.
- **extchange.rs** — `acknowledgment_is_stale(acked, now)` (Present ⇒ `!=`, Missing ⇒ true), after `focus_edge`.
- **editor_surface.rs** — `save_armed: bool` → `armed_at: Option<(SystemTime,u64)>`; NEW `conflict_observed`;
  `active_save_armed() = armed_at.is_some()` (render + #275 tests unchanged); NEW `active_armed_at`,
  `arm_active_save(at)`, `disarm_active_save`, `active_conflict_observed`, `set_conflict_observed`;
  `set_active_conflict(None)` + `reload_active` clear BOTH new fields. Updated 2 in-crate test
  `arm_active_save()` calls to pass a pair (compile fix).
- **app.rs** — `check_active_file_external` Conflict arm + the racing post-save re-flag set
  `conflict_observed = disk.snapshot()`. `save_active`: reads `armed_at`; on `Changed` — 1st press arms
  against `now.snapshot()`; an armed press that's `acknowledgment_is_stale` DISARMS + refreshes
  `conflict_observed` + re-warns ("File changed again"), NO write; armed-and-matching falls through to write.
  `ext_keep_mine`: reads `conflict_observed`; a stale ack refreshes it + keeps the banner (no dismiss); else
  the existing D3 dismiss + re-snapshot (F4b vanished-file guard preserved).
- **Deviation:** the 1st-press arm binds to `now.snapshot()` (a fresh stat) not the check-time
  `conflict_observed` — a hair more current, same value in practice; keeps `now` used in all match arms. Tests
  → Phase 4.

## Inspect (Phase 3.5)
Two critics (general-purpose, then sonnet with a tighter prompt) BOTH stalled — the sonnet one was still
reading files after 5min with no findings emitted (the recurring critic pathology this session). Per the
slow-critic precedent I ran the adversarial review INLINE, tracing the state machine concretely.
| # | Finding | Sev | Verdict | Fix |
|---|---------|-----|---------|-----|
| F1 | The armed-WRITE racing branch (`save_active`, when a W3 races into the write gap → `racing`) set `Changed` + `conflict_observed=W3` but left `armed_at=Some(W1)` — the old arm outlived its consuming write. Harmless in practice (the next press re-warns since W3≠W1) but a latent hole if the disk transiently returns to exactly W1. | LOW | REAL (mine) | Added `s.disarm_active_save()` in the racing branch (the non-racing branch already disarms via `set_active_conflict(None)`) — a successful write always consumes the arm. |
| — | Keep-mine on a file that VANISHED after the banner (`now == Missing` → `acknowledgment_is_stale` true → re-flag with `conflict_observed=None`, keep `Changed`). | nit | ACCEPTED edge | Self-heals: the next choke runs `external_action(snapshot, Missing, dirty) → Deleted`. Out of #284's "newer WRITE" scope (deletions are the Deleted path). Documented, not fixed. |

**Traced CLEAN:**
- **Arm 2nd-press uses `armed_at`, not `conflict_observed`** — the crux. `save_active` runs
  `check_active_file_external` first, which re-flags `Changed` + refreshes `conflict_observed` to the CURRENT
  disk (W2). But `armed_at` is set ONLY by `arm_active_save` (never by the check) and `set_active_conflict
  (Some(Changed))` does NOT clear it (only `None` does) — so the 2nd press compares W2 (now) vs W1 (armed_at)
  → stale → re-warn. Had it used `conflict_observed` it would compare W2==W2 and wrongly overwrite. Verified.
- **Both `Changed`-flag sites set `conflict_observed`** (the Conflict arm + the racing post-save re-flag) —
  no `Changed` banner carries a stale/None observed.
- **No panic / unwrap** on the fs paths: `now.snapshot()` is `Option`-matched; a `Changed` banner implies
  Present so the `if let Some(observed) = now.snapshot()` arm always fires (Missing → the arm-skip is
  unreachable under a real `Changed`, and a no-op if it ever isn't).
- **Pure fn kill-list** covered by the Phase-4 truth table (Present-same→false, mtime≠→true, len≠→true,
  Missing→true) — kills the tuple-eq per field + the Missing arm.
- **Clean-room** — grep clean (no Warp/Zed/iTerm/tmux). Post-fix `cargo check --workspace` green.

## Phase 4 — Validate
**Tests added:** `extchange::acknowledgment_is_stale_cases` (Present-same→false, mtime≠→true, len≠→true,
Missing→true) + `headless_drive::ext_arm_then_newer_write_re_warns_headless` (arm W1 → W2 lands → 2nd ⌘S
leaves the disk W2, banner Changed; a fresh re-arm + matching press then overwrites — this sequence also
kills the `armed_at`/`disarm`/`arm_active_save` accessor mutants) + `ext_keep_mine_newer_write_re_flags_
headless` (banner W1 → W2 → Keep-mine keeps Changed, not dismissed; a 2nd Keep-mine on the now-current W2
dismisses). REQ-002/004 (unchanged-disk proceeds) covered by the EXISTING `ext_save_arm_and_deleted` +
`ext_conflict_banner_flows` tests, which stay green (no regression).
**Test run:** 8 extchange tests PASS (2 new + 6 existing incl. the t275 editor_surface tests).
**Drive-capture:** behavioral (disk/banner state) — the headless flows with real tempdir `fs::write`s ARE
the proof; no new render surface (the banner render is unchanged). Live capture N/A.
**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — coverage 100%, mutation **MSI 100.0% (13
caught / 0 missed)**, all 15 gates PASS. First-try green.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` entry; `editor.md` #275 section updated (the ack-binding: `armed_at`
  + `conflict_observed` + `acknowledgment_is_stale`) and the "acknowledgment-binding" follow-up retired (only
  the FSEvents watcher remains).
- **Knowledge:** `failure-record` BF-claude-extchange-armed-write-leaves-stale-arm-001 (inspect) +
  `prevention-rule-record` PR-claude-consumed-license-cleared-on-every-exit-path. AAR submitted.
- **Lessons:** (1) when TWO consumers ack the SAME logical state but one runs a re-stat FIRST (save_active's
  `check_active_file_external`), a single stored pair gets clobbered — the arm needs its OWN `armed_at`
  distinct from the banner's `conflict_observed` (the crux). (2) A license consumed by an action must be
  cleared on EVERY exit path (the racing-write branch leaked the arm). (3) Both critics stalled AGAIN
  (general-purpose + sonnet) — the inline adversarial trace is the reliable path for a moderate diff; the
  headless re-arm+overwrite / re-flag+dismiss sequences double as the accessor-mutant kill-list (MSI 13/13).
- **Ticket + pipeline:** closed + archived.
