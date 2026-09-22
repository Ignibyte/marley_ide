# forge ticket row → copy '#N — title' — Notes

- **Forge ticket:** #70 `657f8e58-987c-49ae-8ec0-83f7d538c983`
- **AAR:** `9bade5c7-445f-41b6-a2d0-f2a8e716f0ee`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-070-ticket-copy.md

## Phase 1 — Plan
- **Request:** forge #70 (M2.C seq-5, auto-approved) — click a forge ticket → copy its ref.
- **Classification:** work pipeline, `feature`, a tiny PURE (ticket_ref) + an app.rs click shim. UI.
- **Reuse (grep'd):** `cx.write_to_clipboard(ClipboardItem::new_string(text))` (app.rs:1034, from #44 cmd-C);
  ClipboardItem imported (line 15). The forge overlay ticket-row loop is app.rs:1807.
- **Decisions:** D1 copy to clipboard (not insert-at-prompt); D2 ref = "#N — title" (em-dash, no
  status/kind); D3 the ref cloned per click (Fn listener).
- **AAR id:** `9bade5c7-445f-41b6-a2d0-f2a8e716f0ee`.

## Phase 2 — Design

### PURE — `forge_view.rs`
```rust
/// A pasteable reference for a ticket row — `#N — title` (feed it to an agent). No status/kind.
pub fn ticket_ref(row: &TicketRow) -> String {
    format!("#{} \u{2014} {}", row.number, row.title)
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `use crate::forge_view::{forge_status_line, sprint_rows, ticket_ref};` (extend).
- In the forge-overlay ticket-row loop (~1807), wrap each row's child in a clickable div:
```rust
let reference = ticket_ref(&row);
overlay = overlay.child(
    div()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_view, _event: &MouseDownEvent, _window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(reference.clone()));
            }),
        )
        .child(format!("{} #{} {} ({})", row.glyph, row.number, row.title, row.kind)),
);
```

### File manifest
- MODIFY `crates/marley_app/src/forge_view.rs` — `ticket_ref` + a test.
- MODIFY `crates/marley_app/src/app.rs` — the import + the clickable forge ticket row.

### Mutation Targets (pure)
- `ticket_ref`: the format (`#` prefix, number, the ` — ` separator, title). A test asserting the exact
  string pins each part.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `ticket_ref_formats_number_and_title` — `TicketRow{number:70, title:"copy", …}` → "#70 — copy" (U+2014) | unit |
| REQ-002 | click a forge ticket row → its `#N — title` on the clipboard | self-test (⌘⇧F → click → `pbpaste`) |
| REQ-003 | gate GREEN, cov/MSI 100 ticket_ref; app shim excluded | gate |

Uncoverable: the app.rs on_mouse_down clipboard write — masked + cov-excluded, proven by REQ-002.

### Risks / decisions
- D-2.1 the `reference` String is computed per row + `.clone()`d inside the `Fn` listener (called on each
  click). D-2.2 `ticket_ref` uses `row.number` + `row.title` (owned by the loop's `row`) before the
  display `format!` also consumes `row`'s fields — order: ref first, then the display child. D-2.3 reuses
  the #44 clipboard write verbatim (no new clipboard code).

## Phase 3 — Implement
- **Built (PURE):** `forge_view.rs` `ticket_ref(&TicketRow)` = `"#{number} \u{2014} {title}"`.
- **Built (SHIM, app.rs — masked):** each forge-overlay ticket row wrapped in a `div().on_mouse_down(Left,
  …)` → `cx.write_to_clipboard(ClipboardItem::new_string(ticket_ref(&row)))` (reusing #44's clipboard).
  The display child is unchanged. Import `ticket_ref`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 131 pass (no regression). Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + cargo-mutants + a loop-closure-capture probe + an ancestor-handler trace).
  Verdict: **PASS — ship.**
- **Confirmations:** (a) ticket_ref correct ("#70 — forge ticket row → copy", U+2014 verified byte-level;
  empty title → "#70 — "); MSI 100 reachable (a single exact assert kills the 2 whole-body mutants).
  **(b) THE KEY RISK — the per-row closure captures ITS OWN ref, NOT the last row's** — proven absent
  (Rust per-iteration `let reference` + `move`; probe: listener[0]→#70, [1]→#69, [2]→#68). (c) clipboard
  reuse verbatim from #44; no panic; overlay stays open after a copy (copy multiple).
- **Findings:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | I1 | (validate) | The 2 ticket_ref mutants survive only because no test exists yet (implement diff). | P4 REQ-001 assert (single exact string kills both + cov). |
  | L1 | LOW | The handler omitted `cx.stop_propagation()` (the #44 copy handlers have it). Harmless (overlay `.occlude()`, no ancestor mouse handler). | **FIXED** — added for parity/future-proofing. |
  | L2 | LOW/info | The row is now a wrapped div; the click target is the div (text region clickable). | Accept — cosmetic, masked shim. |
- **Fix applied (code):** L1 (`cx.stop_propagation()`). I1 = the P4 test.

## Phase 4 — Validate
- **Tests added** (forge_view.rs): `ticket_ref_formats_number_and_title` (REQ-001 — "#70 — copy" [U+2014]
  + the empty-title edge "#5 — ").
- **Runs (actual):** `cargo nextest -p marley -E 'test(ticket_ref)'` → 1 passed.
- **SELF-TEST (UI — REQ-002, drove the LIVE app + pbpaste):** set the clipboard to a sentinel; ⌘⇧F →
  waited for the async load; clicked the #70 ticket row → `pbpaste` returned **"#70 — TICKET — forge
  ticket row → copy '#N — title' [M2.C seq-5]"** (was "SENTINEL-NOT-CLICKED") — the exact ticket_ref of
  the clicked row, with the em-dash. Proves the click → clipboard copy end-to-end.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. ticket_ref tested; the click handler masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_forge_client.md ticket→agent note.
- **Knowledge:** aar-submit (5). No new rule (the loop-closure risk was ruled out, not hit).
- **Ticket:** forge #70 → done; archived. **5/6 of M2.C.** Click a forge ticket → copy its ref to feed an agent.
