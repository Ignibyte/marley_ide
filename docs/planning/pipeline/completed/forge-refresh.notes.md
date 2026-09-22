# Forge pane refresh + honest states — Notes

- **Forge ticket:** #69 `2baadc57-69db-4cdd-a190-0cdd2dc22b16`
- **AAR:** `654f86a6-ad26-4abb-9194-56589cc5b6f1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-069-forge-refresh.md

## Phase 1 — Plan
- **Request:** forge #69 (M2.C seq-4, auto-approved) — make the frozen #64 forge pane refresh on open.
- **Classification:** work pipeline, `feature`, a small PURE (forge_status_line) + an app.rs SHIM. UI.
- **Current state (grep'd):** new() builds ForgeClient inline + discards it, keeping only forge_sprint;
  the overlay inlines the header format + a "forge unreachable" string.
- **Decisions:** D1 sync re-fetch on open (bounded by the #63 5s timeout; bg-thread deferred); D2 store the
  ForgeClient (bearer already in memory + redacting-Debug guarded); D3 extract forge_status_line (pure).
- **AAR id:** `654f86a6-ad26-4abb-9194-56589cc5b6f1`.

## Phase 2 — Design

### PURE — `forge_view.rs`
```rust
/// The Forge overlay's header line: the sprint summary, or a clear unreachable placeholder.
pub fn forge_status_line(sprint: Option<&SprintView>) -> String {
    match sprint {
        Some(view) => format!(
            "{} (#{}) \u{b7} {} tickets",
            view.sprint.name,
            view.sprint.number,
            view.tickets.len()
        ),
        None => "forge unreachable — is the sidecar running?".to_string(),
    }
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `RootView.forge_client: Option<ForgeClient>` (+ new()).
- new() refactor: build the client first, then the sprint from it —
```rust
let forge_client = std::env::current_dir()
    .ok()
    .and_then(|cwd| std::fs::read_to_string(cwd.join(".mcp.json")).ok())
    .and_then(|json| forge_endpoint_from(&json))
    .map(ForgeClient::new);
let forge_sprint = forge_client
    .as_ref()
    .and_then(|client| client.current_sprint().ok());
```
- Dispatch `toggle-forge`: `if !self.forge_open { self.forge_sprint = self.forge_client.as_ref()
  .and_then(|c| c.current_sprint().ok()); } self.forge_open = !self.forge_open;` (re-fetch on OPEN only).
- Overlay: `use crate::forge_view::{forge_status_line, sprint_rows};`; header
  `format!("\u{1f528} {}", forge_status_line(self.forge_sprint.as_ref()))`; render `sprint_rows` rows only
  `if let Some(view) = &self.forge_sprint`.

### File manifest
- MODIFY `crates/marley_app/src/forge_view.rs` — `forge_status_line` + a test.
- MODIFY `crates/marley_app/src/app.rs` — forge_client field, new() refactor, toggle-forge re-fetch, the
  overlay header, the import.

### Mutation Targets (pure)
- `forge_status_line`: the Some/None arms + the Some format (name/number/count — a built-SprintView test
  with distinct values pins each field). None → the exact message.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `forge_status_line_some_and_none` — `Some(SprintView{name:"M2.C",number:11, 2 tickets})` → "M2.C (#11) · 2 tickets"; `None` → "forge unreachable — is the sidecar running?" | unit |
| REQ-002 | ⌘⇧F re-fetches + shows the sprint header + tickets | self-test (⌘⇧F → capture) |
| REQ-003 | gate GREEN, cov/MSI 100 forge_status_line; app shim excluded | gate |

Uncoverable: the app.rs re-fetch + overlay — masked + cov-excluded, proven by REQ-002.

### Risks / decisions
- D-2.1 sync re-fetch on the UI thread — bounded by the #63 5s timeout; localhost is fast (imperceptible);
  a wedged forge is the 5s worst case (bg-thread deferred). D-2.2 storing ForgeClient keeps the bearer in
  memory (already the case; the #63 redacting Debug + no-log still hold). D-2.3 re-fetch ONLY on open (not
  close) so closing is instant + we don't hammer forge. D-2.4 forge_status_line replaces the inline header
  → the overlay header is now pure-tested (was untestable inline).

## Phase 3 — Implement
- **Built (PURE):** `forge_view.rs` `forge_status_line(Option<&SprintView>)` (Some→header / None→placeholder).
- **Built (SHIM, app.rs — masked):** `RootView.forge_client: Option<ForgeClient>`; new() refactored to
  build the client first then the sprint from it (+ init); `toggle-forge` re-fetches on OPEN (sync,
  #63-timeout-bounded); the overlay header now uses `forge_status_line` + renders sprint_rows only when
  Some (replacing the inline header + the old "forge unreachable" string). Import `forge_status_line`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK (forge_client is
  read in the re-fetch → no dead-code); `cargo nextest -p marley` 130 pass (no regression). Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + a `-j2` cargo-mutants + a re-fetch-state trace + a security grep). Verdict:
  **PASS — correct, MSI 100 reachable, security original.**
- **Confirmations:** (a) forge_status_line correct (Some→"M2.C (#11) · 2 tickets" [U+00B7 middle dot],
  None→the U+2014 em-dash message); MSI 100 with a Some+None pair (both arms needed for COVERAGE; a single
  exact assert kills the 2 whole-body mutants). (b) re-fetch ONLY on the closed→open transition (not
  close); client retained (current_sprint takes &self). (d) **the stored bearer is still never logged** —
  RootView has NO Debug derive, ForgeClient has NO Debug, ForgeEndpoint's redacting Debug (#63) holds; the
  dropped ForgeError carries at most the URL.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | MED | The sync re-fetch runs on the gpui main thread → opening ⌘⇧F could block up to ~2×5s if forge is UP-but-wedged. `TcpStream::connect` itself has no timeout (only read/write do). | **Accepted** — the ticket picks sync-with-timeout (bg-thread deferred); the #63 LOOPBACK guard means connect only targets loopback → instant-or-refuse (no ~75s SYN-drop). Read/write bounded. |
  | F2 | LOW | The startup fetch was redundant (every open re-fetches, overwriting it) + added a launch-time round-trip. | **FIXED** — `forge_sprint` starts `None`; the open re-fetch populates it → launch never blocks on forge (also removes the last startup network call). |
  | F3 | LOW/info | A failed re-fetch clears a previously-good sprint → "unreachable". | Accept — the intended honest-state behavior (no stale data). |
- **Validate note (critic):** the forge_status_line test literals MUST use U+00B7 (·) + U+2014 (—), not
  ASCII `.`/`-`, or they fail against the correct impl.
- **Fix applied (code):** F2 (drop the startup fetch, forge_sprint starts None). clippy OK. F1/F3 accepted.
- **Overlay:** the header is now `🔨 {forge_status_line(...)}` (emoji in the shim, text pure) — visually
  identical to #64's Some case; the None message is the richer honest placeholder.

## Phase 4 — Validate
- **THE SELF-TEST CAUGHT A REAL BUG the critic accepted:** the sync re-fetch FROZE the UI ~5s (the forge
  is ~2.7s/call × 2 — measured via curl; the critic's MED underestimated it as <100ms). The overlay never
  appeared within the capture window. → **Redesigned to a BACKGROUND-THREAD fetch** (the ticket's intent —
  "loading/… states"): `refresh_forge` spawns a `std::thread` that runs `current_sprint()` and hands the
  result back through a `mpsc::Receiver` (`forge_pending`), which the pump `try_recv`-polls each tick;
  `forge_pending.is_some()` = "loading". `ForgeClient` gained `#[derive(Clone)]`. The ⌘⇧F open now NEVER
  blocks the UI. **Lesson: a sync network call on the gpui main thread freezes the UI — the self-test is
  the only thing that proved it (a green gate + a critic PASS did not).**
- **Tests added** (forge_view.rs): `forge_status_line_some_and_none` (REQ-001 — the exact "M2.C (#11) · 2
  tickets" [U+00B7] + the U+2014 unreachable message).
- **Runs (actual):** `cargo nextest -p marley -E 'test(forge_status_line)'` → 1 passed.
- **SELF-TEST (UI — REQ-002, drove the LIVE app):** ⌘⇧F → the overlay appeared at 400ms showing
  **"🔨 loading sprint…"** (`forge_loading.png`) — INSTANT, no freeze — then filled in with the
  **re-fetched current sprint "🔨 M2.C — The Living Cockpit (#11) · 3 tickets"** + its tickets (○#70,
  ◐#69, ○#65) (`forge_loaded.png`). Proves the re-fetch (fresh, not the stale M2.B) + the loading state +
  non-blocking.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. ForgeClient Clone derive + the bg-fetch/pump-poll are masked; forge_status_line is the tested pure surface.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Changed` (bg refresh + loading state); marley_forge_client.md refresh note.
- **Knowledge:** aar-submit (5); BF-claude-sync-network-fetch-on-gpui-main-thread-freezes-ui-001 + PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001 (HIGH — the self-test caught a UI freeze the gate + critic accepted).
- **Ticket:** forge #69 → done; archived. **4/6 of M2.C.** The forge pane refreshes async (loading -> sprint), no UI freeze.
