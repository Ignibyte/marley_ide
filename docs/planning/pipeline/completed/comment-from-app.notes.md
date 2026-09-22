# comment on a ticket from the app — Notes

- **Forge ticket:** #76 `4fdd974f-20c7-40bc-bd3e-a315afd7fcad`
- **AAR:** `695c166c-f900-4b4e-b599-737be9c63f63`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-076-comment-from-app.md

## Phase 1 — Plan
- **Request:** forge #76 (M2.D seq-5, auto-approved) — comment on a forge ticket from the cockpit. Write.
- **Classification:** work pipeline, `feature`, PURE (prepare_comment) + an app.rs SHIM + a harness verb. UI.
- **Pre-flight facts:** the #75 forge-row handler is at app.rs:1896 (`if event.modifiers.platform { claim }`)
  — add a nested `if event.modifiers.shift` for comment. #74 comment_ticket + #72 compose-at-prompt/clear
  + #69 forge_pending + the reset_mods.swift + the #75 cmdclick verb are all in place. comment_ticket takes
  (ticket_id, body) — no owner/author.
- **Decisions:** D1 compose at the prompt (drivable); D2 shift+cmd+click posts; D3 prepare_comment blocks
  blank; D4 clear-on-valid-attempt (async can't sync-confirm); D5 security inherits #74.
- **Self-test:** reset_mods → type a comment at the prompt → ⌘⇧F → cmdshiftclick a ticket → raw forge read
  (the ticket has the new comment).
- **AAR id:** `695c166c-f900-4b4e-b599-737be9c63f63`.

## Phase 2 — Design

### PURE — `forge_view.rs`
```rust
/// Prepare a composed comment for posting: trim it; `None` for blank/whitespace (don't post an empty
/// comment), else `Some` the trimmed body.
pub fn prepare_comment(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `use crate::forge_view::{…, prepare_comment};` (extend).
```rust
fn comment_focused_on(&mut self, ticket_id: String) {
    let line = self
        .workspace
        .state(self.workspace.focused())
        .map(|state| state.buffer.text());
    let Some(body) = line.as_deref().and_then(prepare_comment) else {
        return; // nothing composed → no comment, no clear (the #72 F1 spirit)
    };
    let Some(client) = self.forge_client.clone() else {
        return;
    };
    let (tx, rx) = std::sync::mpsc::channel();
    self.forge_pending = Some(rx); // #69 loading + re-fetch
    std::thread::spawn(move || {
        let _ = client.comment_ticket(&ticket_id, &body);
        let _ = tx.send(client.current_sprint().ok());
    });
    if let Some(state) = self.workspace.focused_state_mut() {
        state.buffer = Buffer::new(); // clear-on-valid-attempt (async can't sync-confirm like #72)
        state.caret = CharOffset::zero();
    }
}
```
The forge-row handler (app.rs:1896) — nest the shift check:
```rust
if event.modifiers.platform {
    if event.modifiers.shift {
        view.comment_focused_on(ticket_id.clone()); // shift+cmd → COMMENT (#76)
    } else {
        view.claim_forge_ticket(ticket_id.clone()); // cmd → CLAIM (#75)
    }
    cx.notify();
} else {
    cx.write_to_clipboard(ClipboardItem::new_string(reference.clone())); // plain → COPY (#70)
}
cx.stop_propagation();
```

### HARNESS — `drive.swift`
Add a `cmdshiftclick:fx,fy` verb: a left-click with `.flags = [.maskCommand, .maskShift]` (mirror the #75
`cmdClick` helper + the `cmdshift:` flag set).

### File manifest
- MODIFY `crates/marley_app/src/forge_view.rs` — `prepare_comment` + a test.
- MODIFY `crates/marley_app/src/app.rs` — `comment_focused_on`, the shift branch, the import.
- MODIFY `scripts/selftest/drive.swift` — the `cmdshiftclick` verb.

### Mutation Targets (pure)
- `prepare_comment`: the trim, the `is_empty` check, the None/Some arms. Tests: "  hi  "→Some("hi") (trim
  works + Some), "   "→None (whitespace→empty→None), ""→None, "a"→Some("a") (non-blank).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `prepare_comment_trims_and_rejects_blank` — "  hi  "→Some("hi"); "   "→None; ""→None; "a"→Some("a") | unit |
| REQ-002 | compose → shift+cmd+click a ticket → the comment posts (raw forge read); plain/cmd click doesn't | self-test |
| REQ-003 | reuses #74's closed write set (localhost, bearer never logged) | review |
| REQ-004 | gate GREEN, cov/MSI 100 prepare_comment; app shim masked | gate |

Uncoverable: `comment_focused_on` + the handler — masked (live socket + gpui), proven by REQ-002.

### Risks / decisions
- D-2.1 modifier precedence: shift+cmd → comment, cmd (no shift) → claim, plain → copy — MUTUALLY
  EXCLUSIVE (platform gates cmd; shift sub-gates comment vs claim) → no cross-fire. D-2.2 the body is read
  from the FOCUSED prompt (the #72 compose pattern) — the overlay open over the terminal keeps the focused
  pane's buffer. D-2.3 clear-on-valid-attempt (the async comment can't sync-confirm delivery like #72's
  write_bytes.is_ok()) — a blank comment (prepare_comment None) → no send + no clear. D-2.4 SECURITY:
  reuses #74 comment_ticket verbatim (closed set, localhost, bearer never logged).

## Phase 3 — Implement
- **Built (PURE):** `forge_view::prepare_comment(&str) -> Option<String>` (trim; None if blank).
- **Built (SHIM, app.rs — masked):** `comment_focused_on(id)` — the focused prompt line → prepare_comment →
  (if Some + client) a bg-thread comment_ticket + re-fetch via #69's forge_pending + clear the prompt; the
  forge-row handler now nests `if shift → comment else → claim` under `if platform`. Import prepare_comment.
- **Built (HARNESS):** `cmdshiftclick:fx,fy` verb + the `cmdShiftClick` helper in drive.swift.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; swift parses;
  `cargo nextest -p marley` 133 pass (no regression). prepare_comment test is Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 2 (correctness + SECURITY). Both **PASS — no defects. No code changes.**
- **Correctness (probe + cargo-mutants):** prepare_comment correct ("  hi  "→Some("hi"), "   "/""→None,
  "a"→Some("a"), "a b"→Some("a b")); **MSI 100 reachable** — the 3 whole-body mutants (None / Some("") /
  Some("xyzzy")) are killed by the Some+None test pair; there is NO is_empty-flip or trim-delete mutant.
  **The 3-way modifier precedence is correct, no cross-fire:** plain→COPY #70, cmd-only→CLAIM #75,
  cmd+shift→COMMENT #76, shift-only→COPY (platform gates the writes); `cx.stop_propagation()` unconditional.
  The #75 change (cmd+shift now comments, plain cmd still claims) is intended, not a regression. No
  panic/borrow (body owned, no dangling &str); a BLANK prompt → prepare_comment None → full no-op (returns
  before forge_pending/clear). The cmdshiftclick verb posts cmd+shift (both flags).
- **Security (6 invariants CONFIRMED):** (1) the comment fires ONLY on a deliberate shift+cmd-click (both
  Cmd AND Shift; comment_focused_on called nowhere else). (2) no bearer leak — no logging; the comment
  error is discarded; the channel is `Receiver<Option<SprintView>>`; **ForgeClient has NO Debug at all** +
  ForgeEndpoint's redacting Debug. (3) the body is serde-escaped (no CRLF/header injection; honest
  Content-Length). (4) closed write set — hardcoded "ticket-comment"; row.id forge-supplied; append-only.
  (5) localhost — same startup loopback-guarded endpoint. (6) no new secret.
- **Findings (all INFO / no code change):** no length cap on the body (forge/socket bounds it);
  clear-on-attempt loses the text if the async comment fails (documented D-2.3 UX tradeoff, not security);
  str::trim is Unicode-aware (NBSP→None — more correct).
- **No code change** — both critics clean; the prepare_comment tests are Phase 4.

## Phase 4 — Validate
- **Test added** (forge_view.rs): `prepare_comment_trims_and_rejects_blank` (REQ-001 — "  hi  "→Some("hi"),
  "a"→Some, "go work #77"→Some [inner spaces kept], "   "/""→None).
- **Runs (actual):** `cargo nextest -p marley -E 'test(prepare_comment)'` → 1 passed.
- **SELF-TEST (UI — REQ-002, drove the LIVE app + raw forge read) — PASSED, comment LANDED:** reset the
  stuck modifier → typed `probe76comment` at the prompt (drivable, the #72 compose pattern) → ⌘⇧F (overlay
  loaded) → **shift-cmd-clicked #77** (`cmdshiftclick:0.40,0.24`, the new drive verb). A raw forge
  `ticket-get` on #77 **contains "probe76comment"** — the composed comment was posted end-to-end. Proves
  the 3rd modifier-branch (shift+cmd=comment) + comment_focused_on + comment_ticket wire correctly.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. prepare_comment tested; the comment shim masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_forge_client.md "## The forge-row modifier trio" + comment note.
- **Knowledge:** aar-submit (5). Both critics clean (no bug). The self-test + a raw ticket-get confirmed the comment LANDS. Established the plain/cmd/shift+cmd row-modifier convention.
- **Ticket:** forge #76 → done; archived. **5/6 of M2.D.** shift+cmd-click a forge ticket → comment on it.
