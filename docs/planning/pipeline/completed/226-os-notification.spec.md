---
pipeline_id: 70714822-fa7a-4d24-840f-c8482a76fcc0
ticket: docs/planning/tickets/open/TICKET-226-os-notification-followups.md
status: Phase 5 — Complete PASS
title: "#203 follow-ups: the macOS completion notification + the tab_flashes stable-close fix"
type: feature
milestone: M12.2
references:
  - docs/planning/pipeline/completed/warp-bg-notify.notes.md
---

## Title
Two of the ticket's three items are LIVE; one died of progress. (a) A finished
background command that cleared the #203 threshold raises a macOS system
notification WHEN THE WINDOW IS INACTIVE — the in-app ● badge stays the core;
the OS note is the away-from-the-app half. (b) is DEAD — recorded, not built:
#398 re-keyed `notify_ticks` to `ContentId` and unified its scrub into the
release tail (app.rs:6060), which every close path reaches; the July premise
(PaneId keys, reap-only scrub) no longer exists. (c) `tab_flashes` is still
keyed by POSITIONAL `(project, tab)` — closing a lower-indexed sibling shifts
later indices and a surviving unviewed badge shows on the WRONG tab until
clear-on-view heals it; the keys remap on close.

## Scope
### In
- **Pure decision** (`notify.rs` extension): `os_note(window_active, notify,
  command) -> Option<OsNote>` — None while the window is active or Notify::No;
  otherwise title ("Marley — command finished/failed") + body (the command,
  truncated at a fixed cap with an ellipsis). cov/MSI 100, full-value asserts.
- **Adapter** (`marley_command`): a fire-and-forget macOS notification spawn —
  `osascript` with the ARGV pattern (`-e 'on run argv' … 'display notification
  (item 1 of argv) with title (item 2 of argv)' … -- body title`): the payload
  rides argv, never interpolated into source — injection-free by construction.
  Non-PTY spawn in the ONE sanctioned crate (the workspace clippy-bans
  `std::process::Command` elsewhere).
- **Pump wiring**: at the existing #203 decision point, when `should_notify`
  returns Succeeded/Failed AND the window is inactive (the in-house
  `last_window_active` tracking — app.rs:17280 reads
  `window.is_window_active()`), call the adapter with the pure fn's output.
  The badge path is untouched.
- **(c)** a pure remap fn (`notify.rs` or `tabs.rs` — design places it):
  on close of tab `t` in project `p`, drop `(p, t)` and shift `(p, t' > t)`
  down one; on project close, drop project `p`'s keys and shift `(p' > p, _)`
  down one. Wired at the close paths. Unit-tested (cov/MSI 100).

### Out (explicitly deferred)
- Item (b) — dead (superseded by #398; the release tail already scrubs on
  every close path). Recorded here as the win, per the #339/#223 precedent.
- A user setting to disable the OS note (add when asked; the gate "only while
  unfocused" keeps it quiet in normal use).
- Notification CLICK actions (focus the tab) — needs a real notification
  delegate (the objc2 route); out of this slice.
- Windows/Linux notification parity (macOS-only, like the platform).
- The #204 palette items — TICKET-227 (next).

## Reference (§20)
Warp (cockpit UX) — Warp raises a system notification when a long-running
command completes while the app is unfocused (the advertised
Settings→Notifications behavior; behavior-level knowledge, no fork source —
the notification-center machinery appears in the behavior maps only as
subsystem references, e.g. `07-app-entry-build-tooling.md` "notification
actions"). Marley matches the behavior CLASS: threshold-cleared background
completion + unfocused window → one system notification with the outcome and
the command. The in-app badge (#203, shipped) remains the focused-window half.

### Prior art
1. **In-house, decisive:** the window-inactive gate ALREADY EXISTS —
   `last_window_active` + `extchange::focus_edge` reading
   `window.is_window_active()` (app.rs:17280; gpui window.rs:1721). The gate
   input is a read, not new plumbing. `should_notify` (#203) already carries
   the threshold/focus/outcome decision; the OS note is a pure extension.
2. **The mechanism fork settles on house constraints:** `osascript`-argv via
   `marley_command` — no new dependency, no unsafe/miri surface, no bundle/
   signing requirement (works from `cargo run`), spawn confined to the ONE
   sanctioned crate, injection-free via argv. The objc2/UNUserNotification
   route (proper delegates, click actions) needs a signed bundle + a new
   unsafe dep — recorded as the upgrade path for notification ACTIONS, not
   this slice.
3. **Permissive deps:** gpui owns window-activity (adopted, above); no crate
   we ship owns system notifications. Checked gpui (no notification API in
   0.2.2), alacritty_terminal, marley deps — no owner.
4. **#398's release tail** killed item (b) — the substrate already does it
   (the recorded win).

## React-first (parity)
N/A — no UI delta: the OS notification is SYSTEM chrome (Notification
Center), not app chrome — the POC cannot represent it; the in-app badge is
unchanged; (c) fixes a wrong-tab badge STATE bug the POC doesn't model
(no badge machinery there). Nothing to build React-side.

## Locked-In Decisions
- D1 — **osascript-argv via marley_command** (evidence above). Fire-and-forget:
  spawn, don't wait; a spawn failure is silently dropped (a notification is
  best-effort — never let it error a pump tick).
- D2 — **The OS note fires only on the window-INACTIVE edge of the existing
  decision** — `should_notify` stays the single threshold/outcome authority;
  no second timer, no new state. The pump-dirty lesson (F-#203) applies: the
  OS-note path adds NO render state, so no dirty flag interaction.
- D3 — **(c) remaps, not scrubs-all**: precise index shifting preserves
  sibling badges (a close shouldn't eat an unrelated tab's unviewed badge);
  the remap is a pure fn over the map, unit-tested per arm.
- D4 — **(b) recorded dead**, no code.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `should_notify` yields Succeeded/Failed for a finished background command WHILE the window is inactive, the system shall produce one `OsNote` (outcome title + truncated command body) and hand it to the adapter. | Pure-fn units (full-value per arm incl. truncation boundary); pump-wiring review at inspect. |
| REQ-002 | WHILE the window is active, the system shall raise NO OS notification (badge only — unchanged). | Pure-fn unit (`os_note(true, …) == None`); review. |
| REQ-003 | The adapter shall pass title/body as ARGV items to `osascript` (no payload interpolation into the script source). | Unit on the argv construction (marley_command); code review. |
| REQ-004 | WHEN tab `t` of project `p` closes, `tab_flashes` shall drop `(p,t)` and shift `(p,t'>t)` to `(p,t'-1)`; WHEN project `p` closes, drop `(p,_)` and shift `(p'>p,t)` to `(p'-1,t)`; sibling badges survive unmoved. | Pure remap units (every arm: drop/shift/untouched-sibling/empty); wired-call review. |
| REQ-005 | The full suite shall stay green; the adapter's actual notification display is a documented uncoverable (system UI; fire-and-forget). | `cargo nextest run --workspace`; gate `--diff`; the uncoverable recorded in notes + gate posture. |

## Phase Plan
- **P2 Design** — exact fn signatures + placements (os_note in notify.rs;
  remap fn placement; adapter fn in marley_command blocking vs async), the
  pump wiring point (where window is reachable at the decision), truncation
  cap value, close-path call sites for (c).
- **P3 Implement** — pure fns + adapter + wiring.
- **P3.5 Inspect** — critics (pump correctness vs the #398 content/view
  lessons; injection; remap edge cases).
- **P4 Validate** — units RUN; suite; gate `--diff`; adapter smoke (spawn
  osascript with a test note — observable as a real notification IF a user
  session exists; else the spawn exit status is the smoke).
- **P5 Complete** — CHANGELOG + app_shell/notify arch note; ledger; archive.
