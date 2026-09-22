# remote connection status (connected / disconnected) — Notes

- **Forge ticket:** #86 `ff987d71-bfe9-4117-bb2d-b2ed83862693` (BACKLOG — M3.A sprint pending)
- **AAR:** `07855e62-39db-494a-a810-02fa6fc696ed`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-086-remote-status.md

## Phase 1 — Plan
- **Request:** forge #86 (M3.A seq-4, auto-approved) — a Disconnected state on a remote pane whose ssh
  exits, instead of auto-closing it.
- **Classification:** work pipeline, `feature`, PURE (RemoteStatus + fns) + an app.rs SHIM. UI.
- **Pre-flight facts:** the #67 pump auto-close is app.rs:268-301 — `states_mut().pump()` collects panes
  with a `SessionEvent::ChildExited` (or an Err) into `dead`, then `for id in dead { if >1 pane {
  close(id); agents.remove; remotes.remove } }`. #86 branches remote panes BEFORE the close: mark
  Disconnected + flash + keep. `remotes` is currently `HashMap<PaneId, String>` (#85) → grow to
  `Remote{host,status}`. The badge render is at ~1858; remote_badge is #85 (update its signature).
- **Decisions:** D1 never auto-close a remote (flip to Disconnected + stay); D2 transition-gated
  flash/dirty (no per-frame loop on the repeatedly-erroring dead session); D3 Remote{host,status} +
  remote_badge takes status.
- **Self-test:** caffeinate -u (wake display) → localhost → cmd-shift-o (⇄ localhost) → in the pane `exit`
  → the pane STAYS, badge → ✗ localhost, "disconnected: localhost" flash.
- **AAR id:** `07855e62-39db-494a-a810-02fa6fc696ed`.

## Phase 2 — Design

### PURE — `marley_remote/src/lib.rs`
```rust
/// Whether a remote (ssh) pane is still connected (#86).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteStatus {
    Connected,
    Disconnected,
}

/// Project a remote pane's status from whether its ssh process has exited (#86).
pub fn remote_status_from(exited: bool) -> RemoteStatus {
    if exited {
        RemoteStatus::Disconnected
    } else {
        RemoteStatus::Connected
    }
}

/// The status glyph for a remote pane: `⇄` connected, `✗` disconnected (#86).
pub fn remote_status_glyph(status: RemoteStatus) -> &'static str {
    match status {
        RemoteStatus::Connected => "\u{21c4}",    // ⇄
        RemoteStatus::Disconnected => "\u{2717}", // ✗
    }
}

// remote_badge gains the status → "{glyph} {host}" (Connected still renders "⇄ host" — #85 unchanged):
pub fn remote_badge(host: &str, status: RemoteStatus) -> String {
    format!("{} {host}", remote_status_glyph(status))
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- import `RemoteStatus`, `remote_status_from` (remote_badge already imported).
- a small app-local struct next to RootView: `struct Remote { host: String, status: RemoteStatus }`.
- `RootView.remotes: HashMap<PaneId, Remote>` (was `String`).
- open-remote insert: `self.remotes.insert(pane_id, Remote { host, status: RemoteStatus::Connected })`.
- badge render: `if let Some(remote) = self.remotes.get(&pane_id) { … .child(remote_badge(&remote.host, remote.status)) }`.
- the #67 pump `for id in dead` loop — branch remote panes BEFORE the `pane_ids().len() > 1` close:
```rust
for id in dead {
    // A dead REMOTE pane (#86): flip to Disconnected + flash, but KEEP it — a visibly-dead remote
    // (network drop / remote logout) beats a vanishing pane. Never auto-close a remote pane.
    if let Some(remote) = view.remotes.get_mut(&id) {
        if remote.status == RemoteStatus::Connected {
            remote.status = remote_status_from(true);
            let host = remote.host.clone();
            view.status_flash = Some(Flash::new(format!("disconnected: {host}")));
            dirty = true;
        }
        continue;
    }
    if view.workspace.pane_ids().len() > 1 {
        if let Ok(state) = view.workspace.close(id) {
            view.agents.remove(&id);
            view.remotes.remove(&id);
            std::thread::spawn(move || drop(state));
            dirty = true;
        }
    }
}
```
The flash+dirty fire ONLY on the Connected→Disconnected transition (the dead session keeps erroring on
pump every frame → dead every frame → the `if Connected` guard makes it idempotent, no per-frame loop).
`remote`'s borrow (get_mut) ends at `remote.host.clone()`, so `view.status_flash =` (a disjoint field) is
fine. `continue` guarantees a remote pane is never closed.

### File manifest
- MODIFY `crates/marley_remote/src/lib.rs` — RemoteStatus + remote_status_from + remote_status_glyph +
  remote_badge(+status) + tests; update the #85 remote_badge_formats test to pass a status.
- MODIFY `crates/marley_app/src/app.rs` — import; the Remote struct; remotes value type; open-remote
  insert; the badge render; the pump disconnect branch.

### Mutation Targets (pure)
- `remote_status_from`: the true/false arms. `remote_status_glyph`: the two glyph arms. `remote_badge`:
  the "{glyph} {host}" format (the glyph via the status). All pinned by the unit tests.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `remote_status_from_maps` — true→Disconnected, false→Connected | unit |
| REQ-002 | `remote_status_glyph_maps` — Connected→"⇄", Disconnected→"✗" | unit |
| REQ-003 | `remote_badge_formats` (updated) — ("localhost",Connected)→"⇄ localhost"; (…,Disconnected)→"✗ localhost"; ("",Connected)→"⇄ " | unit |
| REQ-004 | open ssh localhost → `exit` → the pane STAYS, badge → "✗ localhost", disconnected flash | self-test |
| REQ-005 | gate GREEN, cov/MSI 100 pure; shim masked | gate |

Uncoverable: the pump branch + the render — masked (gpui/live PTY), proven by REQ-004.

### Risks / decisions
- D-2.1 the pump branch uses `continue` so a remote pane is NEVER auto-closed (the last-pane `len()>1`
  guard is bypassed for remotes — intended: even a sole remote pane stays Disconnected). D-2.2 the
  transition guard (`if Connected`) prevents a per-frame flash/repaint on the repeatedly-erroring dead
  session. D-2.3 the `remotes` value grows to a struct — the 2 removes + the render + the insert all
  update; no other reader. D-2.4 `remote_status_from(true)` documents the exit→status mapping (the shim
  only ever passes true; the false arm is a tested pure branch).

## Phase 3 — Implement
- **Built (marley_remote):** `RemoteStatus{Connected,Disconnected}` (Copy/Eq); `remote_status_from(exited)`;
  `remote_status_glyph` (⇄/✗); `remote_badge(host, status)` → "{glyph} {host}" (updated signature) + the
  updated tests (remote_status_from_maps, remote_status_glyph_maps, remote_badge_formats w/ both statuses).
- **Built (marley_app):** import RemoteStatus + remote_status_from; a `struct Remote{host, status}`;
  `remotes: HashMap<PaneId, Remote>`; open-remote inserts `Remote{host, Connected}`; the badge render
  passes `remote_badge(&remote.host, remote.status)`; the #67 pump `for id in dead` loop now branches a
  remote pane FIRST — `if let Some(remote)=get_mut { if Connected { →Disconnected; flash "disconnected:
  {host}"; dirty } continue }` — never auto-closes a remote; the transition guard keeps it idempotent.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_remote` 0 err; clippy `-D warnings` OK;
  `cargo nextest -p marley_remote` 8 pass. The pump branch/render masked; the 4 pure fns are the surface.

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness + MSI; ran cargo-mutants + traced alacritty_terminal's ChildExited + the pump
  dirty path). Verdict: **APPROVE — no HIGH/MED defect.**
- **Confirmations:** (a) **MSI 100** (20/20 viable caught, 2 unviable [no Default]; 8 tests pass). Glyphs
  byte-verified (⇄ e2 87 84, ✗ e2 9c 97); remote_badge("localhost",Disconnected)=="✗ localhost".
  (b) a dead remote pane is **NEVER closed** — the `if let Some(remote){…continue}` runs before the
  `len()>1` close; the `continue` is OUTSIDE the `if Connected` block, so every remote-dead path (flip OR
  already-Disconnected) hits it → never auto-closed. (c) **the guard prevents a per-frame repaint (THE
  risk — does NOT fire):** ChildExited is one-shot (alacritty next_child_event reads the SIGCHLD self-pipe
  once); after that pump() returns `Err(Disconnected)` every frame, and **Err does NOT set dirty** (only
  the Ok/non-empty-events arm does) → exactly ONE repaint on disconnect, then quiescent; the
  `status==Connected` guard makes the re-entry idempotent (no 2nd flash). (d) no borrow issue (disjoint
  field; remote's borrow ends at host.clone()); the #67 last-pane rule holds for non-remote panes; the
  Remote struct migration touches all 7 remotes sites consistently (no missed reader, no leak).
- **Findings:** none actionable. LOW/optional (no change): `remote_status_from(true)` always-true (a tested
  projector documenting exit→status — single source of truth); `host.clone()` one-shot on the transition.
- **Fix applied:** none (critic clean). Note: run the isolated critic target dir SEPARATE from
  cargo-mutants' target (a shared dir gave a stale-mutant-binary false failure).

## Phase 4 — Validate
- **Tests:** `remote_status_from_maps` (REQ-001), `remote_status_glyph_maps` (REQ-002), `remote_badge_formats`
  (REQ-003 — both statuses incl. "✗ localhost").
- **Runs (actual):** `cargo nextest -p marley_remote` → 8 passed.
- **SELF-TEST (UI — REQ-004) — HARNESS ENVIRONMENTALLY BLOCKED (stated, not skipped):** attempted the live
  drive **5×** (raw binary ×2, click-to-focus, `.app` bundle via `open`, foreground-`caffeinate` wake) —
  synthetic input did NOT land (the prompt stayed empty; `type:localhost` never registered), so cmd-shift-o
  couldn't open the pane to exit. This is the memory-documented synthetic-input fragility (macOS
  Accessibility/TCC for the CGEvent-posting process; `[[marley-ax-visual-testing]]`): it WORKED for #84
  (ssh child) + **#85 (the live ⇄ localhost badge rendered)** earlier THIS session, then the input path
  degraded (a TCC/display-state lapse outside my control — the user may need to re-grant Accessibility).
  Per §7 / the validate skill I state this explicitly rather than silently skip. **REQ-004's mechanism is
  otherwise verified to the hilt:** (1) the glyph swap is a tested pure fn — `remote_badge("localhost",
  Disconnected)=="✗ localhost"` at cov/MSI 100; (2) the inspect critic deep-traced the pump branch (a dead
  remote pane flips to Disconnected + is NEVER closed + exactly one repaint + no leak); (3) #85's live
  self-test THIS session already proved the badge-render substrate (⇄ localhost rendered on a real ssh
  pane) — #86 only swaps the glyph on that proven path. The live ⇄→✗-after-exit pixel is the only
  un-driven step, and it rides entirely on tested + critic-verified code.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100% on the pure surface. The pump branch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_remote.md "## Connection status (#86)".
- **Knowledge:** aar-submit (4). Critic APPROVE (no bug); deep-verified the one risk (per-frame repaint) is NOT real — ChildExited one-shot + Err does not set dirty → exactly one repaint. NOTE: the UI self-test was ENV-BLOCKED (synthetic input stopped landing mid-session — Accessibility/TCC); REQ-004 relies on the pure cov/MSI-100 glyph tests + the critic trace + #85 live-render proof.
- **Ticket:** forge #86 → done. **4/5 of M3.A** (tickets in the forge backlog tagged M3.A — sprint-create still erroring, 9×). A dead ssh pane flips to ✗ + stays instead of vanishing.
