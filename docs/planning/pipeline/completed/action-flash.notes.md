# action confirmation indicator (flash) — Notes

- **Forge ticket:** #77 `b0663d2a-b137-4f38-b01b-0484db040b5e`
- **AAR:** `69a8654a-9d4b-43bd-958a-6125f9dcc678`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-077-action-flash.md

## Phase 1 — Plan
- **Request:** forge #77 (M2.D seq-6, THE FINALE, auto-approved) — a transient action-confirmation flash.
- **Classification:** work pipeline, `feature`, a PURE new module (flash.rs) + an app.rs SHIM. UI polish.
- **Pre-flight facts:** the pump is a CONTINUOUS 16ms timer (`cx.background_executor().timer(from_millis(16))`
  loop, app.rs:235) that reads every PTY + refresh_agent_statuses + polls forge_pending each frame,
  repainting when `dirty` — so a tick-counted flash CAN fade when idle (decrement in the pump, mark dirty).
  The 5 action sites: #70 copy + #75 claim + #76 comment (the forge-row handler ~1896), #72 send + #73
  broadcast (dispatch_action). Date::now is BANNED → tick-count.
- **Decisions:** D1 tick-count (FLASH_TICKS=120≈2s); D2 the pump decrements + dirty; D3 one flash, a bottom
  strip; D4 the 5 actions set a Flash (no new behavior).
- **Self-test:** reset_mods → ⌘⇧F → plain-click a ticket (#70 copy) → capture the "copied #N" flash → wait
  >2s → capture it faded.
- **AAR id:** `69a8654a-9d4b-43bd-958a-6125f9dcc678`.

## Phase 2 — Design

### PURE — `flash.rs` (NEW, gpui-free)
```rust
//! PURE — a transient action-confirmation flash (#77): a message + a pump-tick countdown.

/// Pump ticks (~16ms each) a confirmation flash stays visible — ~2 seconds.
pub const FLASH_TICKS: u32 = 120;

/// A transient confirmation shown after a cockpit action, counting down each pump tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flash {
    /// The message (e.g. "copied #70").
    pub message: String,
    /// Pump ticks left before it fades.
    pub remaining: u32,
}

impl Flash {
    /// Start a flash for `message` with a full [`FLASH_TICKS`] countdown.
    pub fn new(message: impl Into<String>) -> Flash {
        Flash { message: message.into(), remaining: FLASH_TICKS }
    }
    /// Advance one pump tick: `Some` the decremented flash while visible, `None` once expired.
    pub fn tick(self) -> Option<Flash> {
        match self.remaining.saturating_sub(1) {
            0 => None,
            remaining => Some(Flash { message: self.message, remaining }),
        }
    }
}
```

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `mod flash;` + `use crate::flash::Flash;`; `RootView.status_flash: Option<Flash>` (init None).
- Pump (after the forge_pending poll, before `if dirty`):
```rust
if let Some(flash) = view.status_flash.take() {
    view.status_flash = flash.tick(); // #77: count down; None clears it
    dirty = true;
}
```
- Render: a bottom strip (near the prompt/status area) showing `self.status_flash.as_ref().map(|f| &f.message)`
  while Some — a plain transient toast.
- The 5 action sites set `self.status_flash = Some(Flash::new(<msg>))`:
  | Site | msg |
  |---|---|
  | #70 copy (forge-row plain click) | `format!("copied #{}", row.number)` |
  | #72 send-to-agent (dispatch) | `"sent to agent"` |
  | #73 broadcast-to-agents (dispatch) | `"broadcast to agents"` |
  | #75 claim (forge-row cmd-click) | `format!("claiming #{}\u{2026}", row.number)` |
  | #76 comment (forge-row shift-cmd-click) | `"comment posted"` |

### File manifest
- ADD `crates/marley_app/src/flash.rs` — `Flash` + `FLASH_TICKS` + tests.
- MODIFY `crates/marley_app/src/app.rs` — mod/use, the field + init, the pump decrement, the render strip,
  the 5 action-site Flash sets.

### Mutation Targets (pure)
- `Flash::new`: `remaining = FLASH_TICKS` (a test asserting `.remaining == FLASH_TICKS`) + `.message`.
- `Flash::tick`: the `saturating_sub(1)`, the `0 => None` arm, the `Some` arm (tests: remaining 2→Some(1),
  1→None, 0→None). Kills the whole-body + arm mutants.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `flash_new_starts_full` — `Flash::new("x")` → message "x", remaining FLASH_TICKS | unit |
| REQ-002 | `flash_tick_counts_down_then_none` — 2→Some(1); 1→None; 0→None (saturating) | unit |
| REQ-003 | plain-click a forge ticket → the "copied #N" flash appears → fades (>2s) | self-test |
| REQ-004 | gate GREEN, cov/MSI 100 flash.rs; app shim masked | gate |

Uncoverable: the pump decrement + the render strip + the action-site sets — masked (gpui), proven by REQ-003.

### Risks / decisions
- D-2.1 tick-count (FLASH_TICKS=120≈2s at 16ms/tick) — Date::now banned. D-2.2 the pump (continuous 16ms
  timer #67) decrements + marks dirty → the countdown repaints + clears at 0 (no repaint stall since the
  flash-active frames set dirty). D-2.3 one flash — a new action overwrites status_flash (the newest wins).
  D-2.4 the render strip is a masked shim; the Flash logic is the tested pure surface.

## Phase 3 — Implement
- **Built (PURE, NEW):** `flash.rs` — `FLASH_TICKS=120`; `Flash{message, remaining}` + `new` + `tick()`.
  Registered `mod flash;` in lib.rs.
- **Built (SHIM, app.rs — masked):** `use crate::flash::Flash`; `RootView.status_flash: Option<Flash>`
  (init None); the pump decrements it (`status_flash.take() → tick()`, dirty=true); a bottom-center pill
  renders `flash.message` while Some; the 5 action sites set a Flash — #70 copy `copied #{n}`, #72 send
  `sent to agent`, #73 broadcast `broadcast to agents`, #75 claim `claiming #{n}…`, #76 comment `comment
  posted` (each with a `cx.notify()` on the copy path).
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 134 pass (no regression). The Flash tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + cargo-mutants + a coverage run + an off-by-one loop trace). Verdict: **PASS.**
- **Confirmations:** (a) Flash correct — new("x")→{message:"x", remaining:120}; tick 2→Some(1), 1→None,
  0→None (saturating); **MSI 100 + cov 100 from the 2 planned tests** (the tick→None mutant killed by
  2→Some(1); cargo-mutants emits NO saturating_sub/arm/const mutants — the whole-body mutant subsumes
  them; the Some(Default) mutant is unviable [no Default]). (b) NO off-by-one — fades at EXACTLY 120 ticks
  (visible 120…1 then None), never sticks (every tick strictly decrements to 0→None), never vanishes on
  the first tick (120→119 is Some). (c) the pump decrement + dirty repaints every active frame (the
  Some→None clear frame also sets dirty); None → skipped, no spurious dirty. (d) the 5 action sites are
  ADDITIVE — send/broadcast guarded by if sent/if delivered; copy gained the needed cx.notify(); row.number
  is Copy.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW | The toast pill had `.occlude()` → for ~2s after every action it swallowed clicks on the terminal underneath. | **FIXED** — dropped `.occlude()` (a passive, click-through toast). |
  | L2 | LOW | "comment posted" (#76 branch) fired unconditionally, even when comment_focused_on no-op'd on a blank line (false confirmation). | **FIXED** — `comment_focused_on` now returns bool (true iff actually posted); the flash is gated on it. |
  | I1 | INFO | flash.rs is untracked → the `--diff` gate needs it `git add`ed to see its mutant (the FULL gate is unaffected). | N/A — the gate does `git add -A` first. |
- **Fix applied (code):** L1 (occlude removed), L2 (comment_focused_on → bool + gated flash). clippy OK, 0 err.

## Phase 4 — Validate
- **Tests added** (flash.rs): `flash_new_starts_full` (REQ-001 — message + remaining==FLASH_TICKS);
  `flash_tick_counts_down_then_none` (REQ-002 — 2→Some(1), 1→None, 0→None saturating).
- **Runs (actual):** `cargo nextest -p marley -E 'test(flash)'` → 2 passed.
- **SELF-TEST (UI — REQ-003, drove the LIVE app) — PASSED (appears → fades):** reset the stuck modifier
  → ⌘⇧F → plain-clicked #77 (clipboard confirmed the #70 copy: "#77 — TICKET …") → **a cyan "copied #77"
  pill appeared at the bottom-center** (`flash_clean.png`) → waited 3s → **the pill is GONE**
  (`flash_faded.png`) — the flash faded on its own (~120 ticks ≈ 2s via the pump). Note: the "plain" click
  needed a modifier reset first (the harness's recurring stuck-cmd-shift, per
  PR-claude-selftest-stuck-synthetic-modifier) — without it the click read as shift-cmd (comment).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. Flash tested; the wiring masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; app_shell.md flash note.
- **Knowledge:** aar-submit (5). 1 critic; FIXED 2 LOW (occlude→click-through toast; comment_focused_on→bool-gated flash). Reused the reset_mods.swift + cmdclick/cmdshiftclick harness verbs.
- **Ticket:** forge #77 → done; archived. **6/6 of M2.D — SPRINT COMPLETE.** Every cockpit action now confirms with a fading flash.
