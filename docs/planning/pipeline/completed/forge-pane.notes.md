# Forge pane — the current sprint's tickets — Notes

- **Forge ticket:** #64 `63c1376d-52aa-4c20-9a05-e431215a6728`
- **AAR:** `602bf407-9035-4eab-9084-8df25478928e`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-064-forge-pane.md

## Phase 1 — Plan
- **Request:** forge #64 (M2.B seq-6, auto-approved) — the visible cockpit surface. The M2.B FINALE.
- **Classification:** work pipeline, `feature`, small PURE (forge_view + keymap) + an app.rs SHIM
  (startup fetch + overlay). UI — validate self-test-captures.
- **Decisions:** D1 overlay (cmd-shift-f); D2 fetch once at startup best-effort (localhost refuses fast,
  no hang); D3 pure glyph/rows tested, fetch/overlay masked; D4 cmd-shift-f free.
- **Reuse:** `new()` uses `std::env::current_dir()` + `Project::discover_in` — read `.mcp.json` from the
  cwd there. `marley_forge_client` (#63): forge_endpoint_from + ForgeClient::current_sprint + SprintView/
  TicketView. The overlay mirrors the cmd-P finder overlay.
- **AAR id:** `602bf407-9035-4eab-9084-8df25478928e`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/forge_view.rs` (NEW; `mod forge_view;` in lib.rs)
```rust
use marley_forge_client::SprintView;

/// The status indicator glyph for a ticket status.
pub fn status_glyph(status: &str) -> &'static str {
    match status {
        "done" => "✓",
        "in-progress" => "◐",
        "open" => "○",
        _ => "•",
    }
}

/// A ticket display row for the Forge overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketRow {
    pub number: u64,
    pub glyph: &'static str,
    pub status: String,
    pub title: String,
    pub kind: String,
}

/// Project the current sprint's tickets into display rows (one per ticket, in order).
pub fn sprint_rows(view: &SprintView) -> Vec<TicketRow> {
    view.tickets
        .iter()
        .map(|ticket| TicketRow {
            number: ticket.number,
            glyph: status_glyph(&ticket.status),
            status: ticket.status.clone(),
            title: ticket.title.clone(),
            kind: ticket.kind.clone(),
        })
        .collect()
}
```

### PURE — `keymap.rs`
`(chord(true, false, false, true, "f"), "toggle-forge")` (cmd-shift-f); keymap test asserts it.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `mod forge_view;` in lib.rs; imports `use crate::forge_view::sprint_rows;` + `use
  marley_forge_client::{forge_endpoint_from, ForgeClient, SprintView};`.
- `RootView { forge_sprint: Option<SprintView>, forge_open: bool }`.
- `new()` best-effort startup fetch (non-fatal):
```rust
let forge_sprint = std::fs::read_to_string(cwd.join(".mcp.json"))
    .ok()
    .and_then(|json| forge_endpoint_from(&json))
    .and_then(|endpoint| ForgeClient::new(endpoint).current_sprint().ok());
```
- Dispatch `"toggle-forge"` → `self.forge_open = !self.forge_open`.
- Overlay (mirrors the finder overlay) when `forge_open`: a header (`🔨 {name} (#{number}) · {N} tickets`)
  + `sprint_rows(view)` rows (`{glyph} #{number} {title} ({kind})`), OR a `🔨 forge unreachable`
  placeholder when `forge_sprint` is None.

### File manifest
- NEW `crates/marley_app/src/forge_view.rs` — status_glyph + TicketRow + sprint_rows + tests.
- MODIFY `crates/marley_app/src/lib.rs` — `mod forge_view;`.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-shift-f binding + test.
- MODIFY `crates/marley_app/src/app.rs` — forge fields, new() fetch, toggle-forge, overlay, imports.
- MODIFY `crates/marley_app/Cargo.toml` — add marley_forge_client.

### Mutation Targets (pure)
- `status_glyph`: the 4 arms (each a DISTINCT glyph → a per-status test kills arm swaps + the catch-all).
- `sprint_rows`: each field mapping (number/glyph/status/title/kind) — a row-field assertion catches a
  wrong-field/constant mutant. keymap: the cmd-shift-f arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `status_glyph_per_status` — done→✓, in-progress→◐, open→○, `weird`→• | unit |
| REQ-002 | `sprint_rows_maps_each_ticket` — a 2-ticket SprintView (distinct statuses) → 2 TicketRows with the right glyph + fields; empty tickets → `[]` | unit |
| REQ-003 | keymap `cmd_shift_f_toggle_forge` — `action_for(cmd-shift-f)=="toggle-forge"` | unit |
| REQ-004 | cmd-shift-f → the overlay shows the live sprint + tickets | self-test (launch → cmd-shift-f → capture) |
| REQ-005 | gate GREEN, cov/MSI 100 forge_view + keymap; app shim excluded | gate |

Uncoverable: the app.rs startup fetch + the overlay render — masked + cov-excluded, proven by REQ-004.

### Risks / decisions
- D-2.1 startup fetch is best-effort (D2) — a localhost connect refuses fast if forge is down → None →
  "unreachable" placeholder; no UI hang. D-2.2 `status`/`title`/`kind` are cloned into TicketRow (owned)
  so the rows outlive the borrow of `forge_sprint`. D-2.3 the glyphs are display-only (the pure test
  pins them so the overlay is trustworthy without re-testing the render).

## Phase 3 — Implement
- **Built (PURE):** `forge_view.rs` — `status_glyph` (4 arms), `TicketRow`, `sprint_rows` (verbatim from
  design). `mod forge_view;` in lib.rs. `keymap.rs` — cmd-shift-f → toggle-forge + the keymap test.
- **Built (SHIM, app.rs — mutants::skip):** `RootView { forge_sprint: Option<SprintView>, forge_open }`
  (+ new() best-effort startup fetch: current_dir → read .mcp.json → forge_endpoint_from → ForgeClient
  → current_sprint().ok()); dispatch `toggle-forge`; the 🔨 cmd-shift-f overlay (header + sprint_rows, or
  "forge unreachable"). `marley_app/Cargo.toml` += marley_forge_client.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK (marley +
  marley_forge_client); `cargo nextest -p marley` 124 pass (keymap test added, no regression). Tests are
  Phase 4. (Pre-existing `block v0.1.6` future-compat note — unrelated.)

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + real cargo-mutants + a gpui-events + live-forge trace). Found a REAL HIGH + 2
  MED — the value of inspect-before-validate.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | HIGH | cmd-shift-f was SHADOWED — the hardcoded cmd-F find-bar check (`platform && key=="f"`, app.rs:948) had no shift guard, ran BEFORE the keymap dispatch, and returned → the forge overlay was UNREACHABLE (the find bar opened instead). The keymap `action_for` test verifies the binding but not runtime reachability. | **FIXED** — added `&& !modifiers.shift` to the find-bar check: plain cmd-F still finds; cmd-shift-f falls through to `toggle-forge`. Self-test-verified in P4. |
  | F2 | MED | `fetch` had no socket timeout — an UP-but-wedged forge would hang app launch forever (the fetch runs synchronously in new()). "localhost refuses fast" only covers the DOWN case. | **FIXED** — `set_read_timeout`/`set_write_timeout` (5s) in the adapter. |
  | F3 | MED | `current_sprint` called `ticket-list` with empty args → forge returns the project's open-bucket ACROSS ALL SPRINTS (probe: #52/#51/#6 are M1.H, not M2.B), mislabeled "the current sprint's tickets". | **FIXED** — `SprintMeta` gains `id`; `tool_call_request` gains an `arguments` param; `current_sprint` passes `{sprint_id: sprint.id}` to ticket-list. The live example now shows ONLY M2.B tickets. |
  | F4 | LOW | overlay `.take(30)` truncates a >30-ticket sprint. | Accepted (consistent with the finder's take(20)). |
- **Verified:** status_glyph strings MATCH forge's real statuses (open/in-progress/done, hyphenated — a
  wrong string = every ticket shows •); sprint_rows maps fields incl. type→kind; no keymap-internal
  collision; the fetch can't panic (all .ok()/.and_then) and now can't hang (timeouts); loopback connect
  refuses in ~9ms.
- **Fixes touched #63's crate** (tool_call_request sig + SprintMeta + adapter) — legitimate: #64's overlay
  surfaced the semantics gap; #63's tests updated (request now asserts the arguments thread through). All
  9 forge_client tests pass; the live example re-verified sprint-scoped.
- **New P4 targets:** the F3 arguments threading is covered by the updated #63 request test; forge_view's
  status_glyph (all 4 glyphs) + sprint_rows (full-row asserts, not just len — MSI has no field-swap mutant).

## Phase 4 — Validate
- **Tests added** (forge_view.rs): `status_glyph_per_status` (REQ-001 — ✓/◐/○/• incl. catch-all);
  `sprint_rows_maps_each_ticket` (REQ-002 — a 2-ticket view → full-row asserts on number/glyph/status/
  title/kind, per the critic's no-field-swap-mutant caveat; empty → empty). keymap cmd-shift-f→toggle-forge.
  Plus the #63 request test updated (asserts the `arguments` thread through) + SprintMeta id.
- **Runs (actual):** `cargo nextest -p marley -E 'test(status_glyph) or test(sprint_rows) or test(keymap)'`
  → 5 passed; workspace 587 passed clean.
- **SELF-TEST (UI — REQ-004, drove the LIVE app):** cmd-shift-f → the 🔨 Forge overlay opened showing the
  **live sprint "M2.B — The Agent Cockpit (#10) · 2 tickets"** with **○ #65** (open bug) + **◐ #64**
  (in-progress feature) — the right status glyphs (`scratchpad/forge_overlay.png`). Proves BOTH inspect
  fixes live: F1 (cmd-shift-f reaches the overlay, not the find bar) + F3 (sprint-scoped — no cross-sprint
  tickets).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. (First runs
  RED on a cargo-mutants "baseline failed" = **disk full** — this long session's critic scratch target
  dirs had filled the disk; freed ~48GB, re-ran green. NOT a code issue.)
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG (completes M2.B); marley_forge_client.md consumed-by note (sprint-scope + timeout).
- **Knowledge:** aar-submit (5); BF-claude-new-chord-shadowed-by-hardcoded-key-check-001 + PR-claude-new-chord-shadowed-by-hardcoded-key-001 (the cmd-shift-f shadow, HIGH — captured in inspect).
- **Ticket:** forge #64 → done; archived. **6/6 of M2.B — THE COCKPIT IS COMPLETE.** Marley shows the sprint (forge) + agents (#62) + terminal in one window. #65 (finder cooked-buffer) remains as a filed follow-up.
