---
pipeline_id: f65d1cee-8a7b-4647-8f27-244637c47917
ticket: forge#176 (bfb56704-ae2c-4e93-9271-af8989b4e795) · local docs/planning/tickets/open/TICKET-176-session-nice.md
aar_id: 0a8d5340-4085-4382-96d8-08c5fad44dfe
status: Phase 5 — Complete PASS
title: M11 — persist the Files-panel open state + the window geometry
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/settings.rs (PURE: files.open + window.{x,y,w,h} + sanitize_window_bounds)
  - crates/marley_app/src/app.rs (SHIM: the two toggle persists, the render settle-persist, run() applies)
---

## Title
The chrome remembers: the Files panel reopens if it was open, and the window comes back at its last
position/size — sanitized so a stale-monitor or hand-edited value can never open an invisible window.

## Scope
### In
- Settings: `files.open` (bool, false); `window.x/y` (i32) + `window.w/h` (u16, the Eq-integral rule);
  `persist_files_open` / `persist_window_bounds`; AppliedSettings fields + the 3 test constructors.
- PURE `settings.rs`: `sanitize_window_bounds(x, y, w, h, displays) -> Option<(f32, f32, f32, f32)>` —
  min size 400×300, values finite, the window's title strip must overlap SOME display; None → the caller
  centers (the safe default).
- SHIM: both Files toggle sites persist; render settle-persists the window bounds (persist only when the
  bounds stopped changing for a frame AND differ from the stored value — no 60Hz writes); `run()` reads the
  saved geometry + sanitizes against `cx.displays()` and opens Windowed there, else centered 1024×768.

### Out
- Maximized/fullscreen state; per-display DPI handling; multi-window.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `sanitize_window_bounds` shall reject sub-minimum/non-finite/off-every-display values (None) and pass sane ones through (boundary mutants killed). | unit |
| REQ-002 | `files.open` + the window settings shall round-trip the store. | unit (the reload test) |
| REQ-003 (visual) | WHEN Files is open and the window was corner-drag resized, a relaunch shall restore both (the capture's own pixel size proves the geometry). | driven |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 self-review + a targeted critic pass folded into P4's gate (a small,
pattern-mirroring diff: the #168 setting recipe ×5 + one pure fn); lenses: settle-persist duty cycle,
display-coord spaces (gpui points vs display bounds), the run() double-load. P4 tests + driven + gate.
P5 docs.
