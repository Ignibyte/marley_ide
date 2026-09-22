# remote-pane badge (⇄ host) — Notes

- **Forge ticket:** #85 `b17286a1-73ef-4055-9fea-8f4e406478bf` (BACKLOG — M3.A sprint pending)
- **AAR:** `c8533790-e1cf-48ca-94af-2e676f576786`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-085-remote-badge.md

## Phase 1 — Plan
- **Request:** forge #85 (M3.A seq-3, auto-approved) — a ⇄ host badge on remote panes. Mirror #66.
- **Classification:** work pipeline, `feature`, PURE (remote_badge) + an app.rs SHIM. UI.
- **Pre-flight facts:** the #66 agent badge renders at app.rs:1843 (`if let Some(run) = self.agents.get(
  &pane_id) { div().absolute().top(px(4.)).right(px(8.))…bg(colors.accent).text_color(colors.on_accent).
  child(agent_badge(run)) }`) — mirror for remote at top-LEFT. `split_focused` returns `Result<PaneId>`
  (the #84 dispatch can capture the id — change `.is_ok()` to `if let Ok(pane_id)`). The agents map (#62)
  + its close cleanup (#67) is the pattern for `remotes`.
- **Decisions:** D1 remotes: HashMap<PaneId, host>, tagged on open + dropped on close; D2 top-left badge;
  D3 remote_badge pure (marley_remote).
- **Self-test:** reset_mods → type `localhost` → cmd-shift-o → the "⇄ localhost" badge on the new pane.
- **AAR id:** `c8533790-e1cf-48ca-94af-2e676f576786`.

## Phase 2 — Design

### PURE — `marley_remote/src/lib.rs`
```rust
/// A pane badge for a remote (ssh) session: `⇄ {host}` (#85).
pub fn remote_badge(host: &str) -> String {
    format!("\u{21c4} {host}")
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- import `remote_badge`; `RootView.remotes: HashMap<PaneId, String>` (init `HashMap::new()`).
- `open-remote` dispatch (#84) — capture the pane id + tag it:
```rust
if let Ok(pane_id) = self.workspace.split_focused(…, || spawn_remote_session(…)) {
    if let Some(state) = self.workspace.state_mut(compose) { state.buffer = Buffer::new(); state.caret = CharOffset::zero(); }
    self.status_flash = Some(Flash::new(format!("ssh {host}"))); // borrows host
    self.remotes.insert(pane_id, host);                          // moves host (after the flash)
}
```
- Render (pane loop, next to the #66 agent badge ~1843) — the badge at top-LEFT:
```rust
if let Some(host) = self.remotes.get(&pane_id) {
    pane = pane.child(div().absolute().top(px(4.)).left(px(8.)).px_1()
        .bg(colors.accent).text_color(colors.on_accent).child(remote_badge(host)));
}
```
- Cleanup: `self.remotes.remove(&closing)` in `close-pane` + `self.remotes.remove(&id)` in the pump
  auto-close (right next to the existing `self.agents.remove(...)` at both sites — #67 pattern).

### File manifest
- MODIFY `crates/marley_remote/src/lib.rs` — `remote_badge` + a test.
- MODIFY `crates/marley_app/src/app.rs` — import, the remotes field + init, the open-remote tag, the badge
  render, the 2 close-cleanup removes.

### Mutation Targets (pure)
- `remote_badge`: the ⇄ glyph (U+21C4) + the host (a `remote_badge("localhost")=="⇄ localhost"` +
  `remote_badge("")=="⇄ "` test pins both).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `remote_badge_formats` — "localhost"→"⇄ localhost"; ""→"⇄ " | unit |
| REQ-002 | cmd-shift-o `localhost` → the "⇄ localhost" badge renders on the pane | self-test |
| REQ-003 | the remotes tag is dropped on close (mirror #67) | review |
| REQ-004 | gate GREEN, cov/MSI 100 remote_badge; shim masked | gate |

Uncoverable: the remotes tag/render + cleanup — masked (gpui), proven by REQ-002.

### Risks / decisions
- D-2.1 top-left badge (agent is top-right) — a pane is agent XOR remote; distinct corners are safe.
- D-2.2 host moved into `remotes` AFTER the flash (which only borrows it) → no clone. D-2.3 the tag
  lifecycle mirrors `agents` (#62/#67): insert on open, remove at BOTH close sites → no leak.

## Phase 3 — Implement
- **Built (marley_remote):** `remote_badge(host) -> "⇄ {host}"` (+ the `remote_badge_formats` test).
- **Built (marley_app):** import `remote_badge`; `RootView.remotes: HashMap<PaneId, String>` + init empty;
  the open-remote dispatch now `if let Ok(pane_id) = split_focused(…) { clear; flash; self.remotes.insert(
  pane_id, host) }` (host moved AFTER the flash borrows it); the badge render (top-LEFT pill next to the
  #66 agent badge); `self.remotes.remove(&id)` at BOTH close sites (the pump auto-close + close-pane, next
  to `agents.remove`).
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_remote` 0 err; clippy `-D warnings` OK;
  `cargo nextest -p marley_remote` 6 pass (incl. remote_badge_formats). The badge render/tag are masked;
  remote_badge is the pure surface.

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness + MSI; ran cargo-mutants + traced the tag lifecycle + PaneId reuse).
  Verdict: **APPROVE — no HIGH/MED findings.**
- **Confirmations:** (a) remote_badge correct + **MSI 100** (2 mutants on it — String::new()/"xyzzy" — both
  killed by remote_badge_formats; crate 18/18 caught, 1 pre-existing-unviable). `remote_badge("")=="⇄ "`
  (bytes e2 87 84 20). (b) **the remotes tag lifecycle is AIRTIGHT — no leak:** insert ONLY on
  `Ok(pane_id)`; removed at BOTH (and the only two) close paths (the pump auto-close @295 + close-pane
  @1033, each beside `agents.remove`); `Workspace::close` is the only pane-destroying method (no
  close-all/split-replace; the last-pane rule refuses); **PaneId is monotonic (never recycled)** so even a
  hypothetical leak could never badge a wrong pane. (c) top-LEFT badge vs the top-RIGHT agent badge → no
  collision; a pane is agent XOR remote (the only inserts are agents.insert@932 + remotes.insert@1022, one
  pane per split → at most one map). (d) host move-after-flash valid (format! borrows, then insert moves;
  single move, no clone).
- **Findings:** none actionable. LOW/optional (no change): the ⇄ pill structurally mirrors the agent pill
  (consistent with the existing pattern — a shared helper would be marginal); the U+21C4 glyph is correct +
  self-consistent (fn + test both `\u{21c4}`).
- **Fix applied:** none (critic clean).

## Phase 4 — Validate
- **Test:** `remote_badge_formats` (REQ-001 — "localhost"→"⇄ localhost"; "deploy@prod"→…; ""→"⇄ ").
- **Runs (actual):** `cargo nextest -p marley_remote` → 6 passed (incl. remote_badge_formats).
- **SELF-TEST (UI — REQ-002, drove the LIVE app) — PASSED:** (the display had slept mid-session → a black
  capture; woke it with `caffeinate -u`, then re-drove.) reset mods → typed `localhost` → ⌘⇧O. Result
  (`remote_badge.png`): the new (right) pane shows the **"⇄ localhost" badge** as a cyan accent pill at the
  **top-left** (opposite the top-right agent badge) + the compose prompt cleared + `ps` confirms the ssh
  child. The badge renders exactly as designed.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. remote_badge tested; tag/render masked.
- **Pre-existing:** none. (Env note: macOS display sleep blanks `screencapture` — wake with `caffeinate -u`
  before a self-test in a long session.)

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_remote.md "## Remote-pane badge (#85)".
- **Knowledge:** aar-submit (5); PR-claude-selftest-wake-display-with-caffeinate-before-screencapture-001 (a slept macOS display → black screencapture + input may not register → caffeinate -u before a long-session self-test). Critic APPROVE (no bug); the remotes tag lifecycle is leak-free (PaneId monotonic).
- **Ticket:** forge #85 → done. **3/5 of M3.A** (tickets in the forge backlog tagged M3.A — sprint-create still erroring, 8×). The ⇄ host badge marks remote panes.
