# claim a ticket from the Forge pane — Notes

- **Forge ticket:** #75 `bc3d5e08-5079-4eaa-a8a1-e66f5c37ca21`
- **AAR:** `2f1aa154-357d-4a21-8331-8fd1560258fd`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-075-claim-from-pane.md

## Phase 1 — Plan
- **Request:** forge #75 (M2.D seq-4, auto-approved) — cmd-click a forge ticket to claim it. First
  write-from-the-UI. SECURITY-SENSITIVE.
- **Classification:** work pipeline, `feature`, PURE (TicketView.id + TicketRow.id/sprint_rows) + an app.rs
  SHIM + a harness verb. UI.
- **Pre-flight facts:** TicketView = {number,title,status,kind} (NO id — add it); TicketRow =
  {number,glyph,status,title,kind} (NO id — add it); sprint_rows maps TicketView→TicketRow (app.rs
  forge_view). The forge overlay loop is app.rs:1867 (`for row in sprint_rows(view)` → the #70 copy
  handler). The codebase reads cmd as `keystroke/event.modifiers.platform`. #74 claim_ticket + #69
  forge_pending bg-fetch + the #72 blocking-IO PR + the reset_mods.swift harness fix are all in place.
  MARLEY_OWNER = `402c1331-8409-4cad-9861-76044290981e`.
- **Decisions:** D1 cmd-click (deliberate/unambiguous, clean row); D2 bg-thread claim + re-fetch (#72/#69);
  D3 fixed MARLEY_OWNER + only claim_ticket; D4 the id threads TicketView→TicketRow→handler.
- **RISK:** adding a REQUIRED `id` to TicketView/TicketRow breaks the existing parse_tickets + sprint_rows
  + any TicketRow-constructing test fixtures (they lack id) → update those fixtures in implement/validate.
- **Self-test:** reset_mods.swift → ⌘⇧F → cmdclick a ticket → raw forge read (owner_id==MARLEY_OWNER) → release.
- **AAR id:** `2f1aa154-357d-4a21-8331-8fd1560258fd`.

## Phase 2 — Design

### PURE
- `forge_client/lib.rs` `TicketView`: add `pub id: String` (first field; deserializes the response `id`).
- `forge_view.rs` `TicketRow`: add `pub id: String`; `sprint_rows` sets `id: ticket.id.clone()`.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
```rust
/// The owner id the Marley cockpit claims tickets under (a fixed app identity).
const MARLEY_OWNER: &str = "402c1331-8409-4cad-9861-76044290981e";

// on RootView (mutants::skip):
fn claim_forge_ticket(&mut self, ticket_id: String) {
    let Some(client) = self.forge_client.clone() else { return; };
    let (tx, rx) = std::sync::mpsc::channel();
    self.forge_pending = Some(rx); // reuse #69's channel → the overlay shows "loading" then the re-fetch
    std::thread::spawn(move || {
        let _ = client.claim_ticket(&ticket_id, MARLEY_OWNER);
        let _ = tx.send(client.current_sprint().ok());
    });
}
```
The forge-row handler (app.rs:1867) — capture `row.id`, branch on the cmd modifier:
```rust
let reference = ticket_ref(&row);
let ticket_id = row.id.clone();
overlay = overlay.child(
    div().on_mouse_down(MouseButton::Left, cx.listener(move |view, event: &MouseDownEvent, _w, cx| {
        if event.modifiers.platform {
            view.claim_forge_ticket(ticket_id.clone()); // cmd+click → CLAIM (#75)
            cx.notify();
        } else {
            cx.write_to_clipboard(ClipboardItem::new_string(reference.clone())); // plain click → copy (#70)
        }
        cx.stop_propagation();
    })).child(format!("{} #{} {} ({})", row.glyph, row.number, row.title, row.kind)),
);
```

### HARNESS — `scripts/selftest/drive.swift`
Add a `cmdclick:fx,fy` verb: parse the fraction like `clickat:`, then post `leftMouseDown`/`leftMouseUp`
with `.flags = .maskCommand` (mirrors clickat's helper + cmdshift's flag pattern).

### File manifest
- MODIFY `crates/marley_forge_client/src/lib.rs` — TicketView.id + fixture updates in the parse test.
- MODIFY `crates/marley_app/src/forge_view.rs` — TicketRow.id + sprint_rows + fixture updates in the tests.
- MODIFY `crates/marley_app/src/app.rs` — MARLEY_OWNER, claim_forge_ticket, the cmd-click branch.
- MODIFY `scripts/selftest/drive.swift` — the cmdclick verb.

### Mutation Targets (pure)
- `TicketView.id`: a parse_tickets test with a distinct `"id":"tid-1"` → assert `view.id == "tid-1"`.
- `sprint_rows`: the existing whole-body mutant is guarded by the sprint_rows test; add `assert_eq!(row.id,
  "tid-1")` so a dropped id copy is caught.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `parse_tickets` — a response with `"id":"tid-1"` → `TicketView.id == "tid-1"` | unit |
| REQ-002 | `sprint_rows` — `row.id == ticket.id` (a distinct id in the fixture) | unit |
| REQ-003/004 | ⌘⇧F → cmd-click a ticket → claimed for MARLEY_OWNER (raw forge read); plain click still copies | self-test |
| REQ-005 | gate GREEN, cov/MSI 100 on the id-threading; app shim masked | gate |

Uncoverable: `claim_forge_ticket` + the click handler — masked (live socket + gpui), proven by REQ-003.
**Fixture updates (the new REQUIRED id breaks existing tests):** lib.rs parse_tickets fixture JSON +
expected TicketViews; forge_view sprint_rows/status_glyph fixtures; #70 ticket_ref test's TicketRow — all
gain an `id`.

### Risks / decisions
- D-2.1 the claim runs on a BG thread + re-fetches via `forge_pending` (the #72 no-blocking-IO PR + #69's
  channel + loading state). D-2.2 `event.modifiers.platform` is the codebase's cmd field (used at
  app.rs:378/529/595) — a PLAIN click has platform=false → copy (no accidental claim). D-2.3 the id is
  now REQUIRED on TicketView/TicketRow → update every test fixture (compile-forced). D-2.4 SECURITY: only
  claim_ticket (#74 closed set), MARLEY_OWNER fixed, localhost + bearer-guarded, no bearer in any log.

## Phase 3 — Implement
- **Built (PURE):** `TicketView.id: String` (forge_client, deserializes "id"); `TicketRow.id: String` +
  `sprint_rows` copies `ticket.id.clone()` (forge_view).
- **Built (SHIM, app.rs — masked):** `const MARLEY_OWNER`; `claim_forge_ticket(id)` (bg-thread claim →
  re-fetch via the #69 forge_pending channel + loading state); the forge-row on_mouse_down now branches on
  `event.modifiers.platform` (cmd) → `claim_forge_ticket(row.id)` + notify, else → the #70 copy.
- **Built (HARNESS):** `cmdclick:fx,fy` verb in drive.swift (a `.maskCommand`-held left-click) + the
  `cmdClick` helper.
- **Fixtures updated (the new REQUIRED id):** lib.rs parse_tickets (JSON + expected, now REQ-001 with a
  distinct id `tid-63`); forge_view ticket_ref/forge_status_line (id: String::new()) + sprint_rows (REQ-002
  — distinct `t63`/`t65` ids thread TicketView→TicketRow).
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_forge_client --tests` 0 err; clippy OK;
  swift parses; `cargo nextest` 144 pass (no regression). The id-assertion tests are Phase 4 (already in
  the fixtures above).

## Phase 3.5 — Inspect
- **Critics:** 2 (correctness + SECURITY). Both **PASS — ship**. No code changes.
- **Correctness (probe + cargo-mutants):** the id threads TicketView→TicketRow; **MSI 100 both pure files**
  (lib.rs 36 caught / forge_view 10 caught, 0 missed — parse_tickets `Ok(vec![])` killed by the REQ-001
  id assert; sprint_rows `vec![]` killed by the REQ-002 distinct-id asserts; no dedicated mutant for the
  `id: ticket.id.clone()` field line — it rides the whole-body mutant + exact-value asserts). **The
  required `id` deserialize is SAFE: all 77 live-forge tickets have a non-null id** (a real ticket-list
  never Errs; serde ignores extras). The cmd-branch: `event.modifiers.platform` is gpui's Cmd bool (only
  Cmd sets it) → a plain/shift/ctrl/alt click keeps the #70 copy (no accidental claim); cmd claims;
  stop_propagation in both. No panic/borrow (client moved into the thread, errors `let _ =`). The bg
  re-fetch reuses #69's forge_pending + pump exactly.
- **Security (6 invariants CONFIRMED):** (1) the mutation fires ONLY on a deliberate Cmd+Left mouse-down —
  button-gated (right/middle never reach the Left listener) + Cmd-gated (a plain click stays read-only
  copy); no accidental mutation. (2) no bearer leak — no logging in app.rs; the claim errors are discarded
  + only carry the URL; the channel payload is response data; the redacting Debug holds. (3) closed write
  set — hardcoded "ticket-claim" + MARLEY_OWNER; row.id is forge-supplied (not user-typed); no arbitrary
  tool/owner/ticket. (4) localhost — the same loopback-guarded startup endpoint. (5) ticket_id safe
  (forge-supplied UUID, json!-escaped, #74). (6) MARLEY_OWNER is an identity, not a secret.
- **Findings (all LOW / no code change):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | V1 | LOW (VALIDATE gate) | A rejected claim is silently swallowed (`let _ =`); MARLEY_OWNER appears on 0/77 tickets. If forge validated owner_id, the claim would no-op. | **Validate MUST confirm the claim LANDS** via a raw forge read (owner_id set to MARLEY_OWNER), not just "no crash". Forge's owner_id is a free UUID (no users table, per the schema) → expected to accept 402c1331; the self-test verifies it. |
  | S1 | LOW | Cmd+double-click fires claim twice → idempotent re-claim of the SAME ticket for the SAME owner. | Accept — benign. |
  | S2 | LOW | MARLEY_OWNER hardcoded. | Accept — an identity, not a secret; future config is a product decision. |
  | O1 | LOW/info | The post-claim overlay reflects the claim via the status glyph (no owner field). | Accept — matches spec. |
- **No code change** — both critics clean; V1 is the load-bearing validate check.

## Phase 4 — Validate
- **Tests (already in the P3 fixtures):** `parse_sprint_and_tickets` (REQ-001 — TicketView.id "tid-63"
  parses off the response); `sprint_rows_maps_each_ticket` (REQ-002 — distinct t63/t65 ids thread
  TicketView→TicketRow).
- **Runs (actual):** `cargo nextest -p marley_forge_client` 11 passed; `sprint_rows_maps_each_ticket` passed.
- **SELF-TEST (UI + SECURITY — REQ-003/004, drove the LIVE app + raw forge read) — PASSED, claim LANDED:**
  reset the stuck modifier (reset_mods.swift) → ⌘⇧F (the overlay loaded the M2.D sprint: ○#77, ○#76, ◐#75)
  → **cmd-clicked #76** (`cmdclick:0.40,0.31`, the new drive verb). A raw forge ticket-list read confirmed
  **#76 owner_id went None → `402c1331…` (== MARLEY_OWNER)** — the claim really landed (not a silent
  no-op; forge accepts the free-UUID owner). **#77 + #75 owners UNCHANGED** — only the clicked row was
  claimed (no accidental mutation; the cmd-gate works). Released the test claim afterward (#76 → None).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. TicketView.id + sprint_rows.id tested; the claim shim + cmd-branch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_forge_client.md "## Claim from the UI (#75)".
- **Knowledge:** aar-submit (5). Both critics clean (no bug). The self-test + a raw forge read confirmed the claim LANDS (owner→MARLEY_OWNER) — reused the AD-forge-write-ack + reset_mods.swift + the new cmdclick drive verb.
- **Ticket:** forge #75 → done; archived. **4/6 of M2.D.** cmd-click a forge ticket → claim it (the first write from the UI, cmd-gated).
