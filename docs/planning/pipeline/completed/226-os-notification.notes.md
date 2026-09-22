# 226 — OS notification + notify cleanup — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-226-os-notification-followups.md
- **Pipeline spec:** 226-os-notification.spec.md

## Phase 1 — Plan
- **Request:** batch position 6: the #203 follow-ups (a/b/c). Auto-approved.
- **Classification / tier:** feature, single slice (a pure extension + one
  adapter fn + wiring + a pure remap).
- **Recall (§18.3):**
  - notify.rs (#203): pure `should_notify(pane_focused, elapsed, threshold,
    status) -> Notify` — the single decision authority; the pump computes
    inputs and paints badges.
  - F-#398 (failures.md:563): the content/view split — notify_ticks is
    ContentId-keyed with the release-tail scrub (app.rs:6048-6060) firing on
    last-view drop; the pump dedupes per content. **Item (b) is DEAD.**
  - F-#203 (failures.md:603): pump state changes must set `dirty` — the OS
    note adds no render state (D2), but any (c) wiring that touches
    tab_flashes outside the pump must respect repaint semantics (the close
    paths already notify).
  - warp-bg-notify archive: the #203 pipeline record.
- **Sweep:** the window-inactive gate exists in-house (`last_window_active`,
  `focus_edge`, `window.is_window_active()` at app.rs:17280; gpui
  window.rs:1721). marley_command is the ONE sanctioned non-PTY spawn seam
  (workspace clippy-bans std::process::Command elsewhere) — the adapter lives
  there; osascript-argv is injection-free by construction. objc2 recorded as
  the future click-actions path. gpui 0.2.2 has no notification API.
- **Current-state verification (in-transcript):** notify_ticks:
  `HashMap<ContentId, u32>` scrubbed at :6060 (b dead); tab_flashes:
  `HashMap<(usize,usize), Notify>` positional — set :1867, clear-on-view
  :2047, read :18810 (c live).
- **Decisions:** D1 osascript-argv adapter; D2 single decision authority +
  no new pump render state; D3 remap not scrub-all; D4 (b) recorded dead.

## Phase 2 — Design

**Architecture.** §20 posture confirmed (behavior-class match; N/A React).
Three touches, all in existing seams:

1. **notify.rs (pure):** `pub struct OsNote { title: String, body: String }`;
   `pub fn os_note(window_active: bool, n: Notify, command: &str) ->
   Option<OsNote>` — None on active/No; titles "Marley — command finished" /
   "… command failed"; body = command truncated at `OS_NOTE_BODY_MAX = 120`
   CHARS (char-boundary safe, '…' appended when cut). Plus the (c) remaps —
   `remap_flashes_after_tab_close(map, p, t)` /
   `remap_flashes_after_project_close(map, p)` — pure map-in/map-out fns
   (placement: notify.rs owns Notify + the flashes semantics), following the
   HOUSE PRECEDENT `remap_indices_after_remove` (#236 collapsed_projects) /
   `remap_section_indices_after_remove` (#386): drop the closed key, shift
   higher indices down, leave siblings untouched.
2. **marley_command (adapter):** `blocking::notify_macos(title, body)` —
   fire-and-forget spawn; argv factored pure as
   `notify_macos_argv(title, body) -> Vec<String>` (unit-testable: script
   lines are CONSTANTS, payload rides as trailing run-handler argv — the
   injection-free proof is that the constants contain no interpolation).
   Spawn errors dropped (best-effort; never errors a pump tick). osascript's
   trailing-args-as-argv behavior gets a REAL smoke at validate.
3. **app.rs wiring:** (a) the #203 decision edge (~:1843-1859) captures the
   finishing block's command into the finishes tuple `(id, n, command)`; the
   finishes loop (:1865) — after the badge insert — calls
   `os_note(view.last_window_active, n, &command)` and hands Some to the
   adapter. `last_window_active` is maintained by the render focus-edge path
   (:17280) and flips on the deactivation render — no new plumbing, no new
   render state (D2/F-#203 clean). (c) `close_tab_at` (:8687 region, after
   the renaming_tab/naming_pane dismissals) applies the tab remap;
   `close_project_at` (:8861 region, beside the collapsed remaps) applies
   the project remap.

**Manifest.**
- `crates/marley_app/src/notify.rs` — OsNote + os_note + OS_NOTE_BODY_MAX +
  the two remap fns (+ tests at validate).
- `crates/marley_command/src/blocking.rs` — notify_macos + notify_macos_argv
  (+ argv unit at validate).
- `crates/marley_app/src/app.rs` — the tuple extension + the two wiring
  lines + the two remap call sites.

**Test plan.**

| REQ | Test | Assert |
|---|---|---|
| REQ-001 | notify.rs units | full-value OsNote per arm: Succeeded/Failed titles, body passthrough, the 120-char boundary (at, under, over — over gets '…'; multibyte-safe probe) |
| REQ-002 | notify.rs unit | `os_note(true, _, _) == None`; `os_note(_, Notify::No, _) == None` |
| REQ-003 | marley_command unit | `notify_macos_argv` exact vector (constants + trailing [body, title]); no payload substring inside any constant |
| REQ-004 | notify.rs units | remap arms: closed-key dropped; higher shifted; lower/other-project untouched; empty map; project arm analogs |
| REQ-005 | suite + gate | 2148+ green; `--diff` green. Uncoverable: the actual Notification Center display (system UI; fire-and-forget) — recorded; the validate smoke spawns the REAL osascript once and checks exit status (+ the visible banner IF a user session exists) |

**Risks.** osascript trailing-argv parsing without a program file — validated
by the real smoke before the wiring ships; if it misparses, the fallback shape
is a stdin-fed script (`osascript - args…`), same injection posture. The
truncation cap (120) is a judgment call — recorded, trivially tunable.

## Phase 3 — Implement
- **React-first: N/A** (system chrome; POC can't represent it).
- notify.rs: `OsNote` + `os_note` (window gate → outcome titles → 120-char
  char-safe truncation with '…') + `OS_NOTE_BODY_MAX` + the two remap fns
  (cmp-match arms: drop equal, shift greater, keep less/other).
- marley_command/blocking.rs: `notify_macos_argv` (the FULL argv, pure —
  consumed VERBATIM by the spawn so the test pins what ships; script lines
  constants, payload as trailing run-handler args) + `notify_macos`
  (fire-and-forget spawn, stdio nulled, child handle deliberately dropped).
- app.rs wiring: finishes tuple widened to `(PaneId, Notify, String)` (the
  block's command captured at the decision edge); the finishes loop fires
  `os_note(view.last_window_active, n, &command)` → adapter, result dropped
  (best-effort; F-#203 note in-code: no new render state, no dirty
  interaction). `close_tab_at` + `close_project_at` apply the remaps beside
  the renaming_tab/naming_pane dismissals + #236 collapse remaps.
- One compile fix: the `finishes` Vec type widened (the declaration was above
  my read window). `cargo check --all-targets` 0 errors; fmt clean.
- Pre-smoked at design: the osascript trailing-argv shape parses (in-transcript,
  exit 0, argv items round-tripped).
- Deviations: none.

## Phase 3.5 — Inspect

Two critics (security/injection; correctness/state), both tracing to source
(incl. gpui's activation plumbing and std's Child-drop semantics).

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | MEDIUM | The dropped Child never reaps — one zombie per notification, monotonic; the eventual per-uid-cap failure is app-wide fork failure. The house solves it three ways already (app.rs reap thread, lsp + harness wait-on-drop). | REAL | Fixed — detached reaper thread in `notify_macos`. F- + PR- appended; `open_url_with`'s same latent pattern recorded (rare-fire, next-ticket-owned). |
| F2 | MINOR | `last_window_active` staleness — bounded to ~1 frame both directions (gpui refreshes on every activation change; becomeKey paints synchronously); the occlusion-freeze sub-case is coalesced away in practice; boot-false unreachable as a wrong banner (10s threshold vs first-second paint). | REAL but bounded | No fix (recorded; the observe_window_activation observer is the named upgrade if it ever bites). |
| F3 | NIT | 120-char cut can split a ZWJ grapheme (cosmetic broken glyph in the banner). | ACCEPTED | Recorded; chars-not-bytes keeps it panic-free. |
| F4 | INFO | `blocks().last()` at the finish edge is the shipped #203 shape — a same-tick new command MISSES the finish (badge and note both) rather than misattributing; running=false at the edge means last IS the finished block. | PRE-EXISTING | Unchanged risk, recorded. |

Clean: injection (execve argv — no shell; constants-only script lines verified
by grep; AppleScript receives argv as opaque text, no run/do-shell in
constants; NUL → spawn Err → dropped; ANSI/RTL inert in the GUI), spawn-storm
(pump dedupes per content; no repeat-fire — ticks reset + else-if guard),
PATH (absolute /usr/bin/osascript), dep edge (marley_command already a dep),
remap coverage COMPLETE (the only tab/project removal paths route through the
two remapped fns; no reorder exists — every add is a push), remap off-by-one
(injective after Equal-drop; underflow impossible), no stale (p,t) copies
(all touchpoints resolve at use time, same-thread), truncation exact at
120/121 + multibyte-safe.

Ledger appends: F-claude-226-fire-and-forget-spawn-never-reaped-zombie-
accumulation-001; PR-claude-fire-and-forget-spawns-still-reap-001.

## Phase 4 — Validate
- **Tests written:** os_note gate/content full-value pins + the 120/121/CJK/
  empty truncation boundaries; the two remap arm suites (drop/shift/untouched/
  empty); the argv full-vector pin with the hostile-payload injection proof;
  the spawn-seam test (program-injected `true`, reaper JOINED — deterministic;
  nonexistent program → Err).
- **First gate run RED (gate:4 + gate:5):** `notify_macos`'s body was
  untested (13 uncovered lines; the swallowed-spawn mutant survived — MSI
  93.3%). Root cause: the first shape hardcoded the program, making the spawn
  unreachable without posting a real banner. Fixed by the house
  `open_url_with` testable-seam pattern: `notify_macos(program, title, body)
  -> io::Result<JoinHandle<()>>` (production passes the `OSASCRIPT` const and
  drops the handle; tests inject `true` and JOIN the reaper — no
  detached-thread coverage race, no banner in tests).
- **Runs:** suite 2153/2153 + marley_command 19/19; re-run gate →
  **GATE GREEN [diff], 15 passed 0 failed** (coverage 100%, MSI 100%),
  receipt written.
- **Real adapter smoke:** the EXACT shipped argv spawned against the real
  `/usr/bin/osascript` `display notification` — exit 0, empty stderr
  (in-transcript). The visible banner is the documented uncoverable half
  (system UI); the pump wiring lives in the documented coverage-exempt app.rs
  shim, reviewed at inspect.
- Pre-existing: `block v0.1.6` note — not in scope.

## Phase 5 — Complete
- CHANGELOG entry; app_shell.md notify paragraph updated. Ledger appends were
  made at inspect (F-…-zombie-…-001, PR-…-still-reap-001); no further L-/AD-
  (the PR rule carries the durable content).
- Ticket → closed/; backlog clean; archived.
