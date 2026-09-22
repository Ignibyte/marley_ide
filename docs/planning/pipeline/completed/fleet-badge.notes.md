---
pipeline_id: e99f3f31-61ac-429d-b09d-7ccfac33f371
ticket: forge#188 (369dae42-9dee-44e8-b34b-8b1600780139)
aar_id: e56d2026-2e72-4983-a1d5-c7e680f1a1d9
---

# Notes — M12 #188 fleet badge

## Phase 1 — Plan / Phase 2 — Design (folded; Small)
**Approach.** One pure fn + two shim render tweaks. `fleet_badge(agents)` counts RUNNING agents (Working|Waiting
per #167) → `Option<String>` (None hides the badge). The shim already renders the top-bar cockpit tab icons
(app.rs ~4971) and the footer segments (~4855) — add the badge overlay on the Agents tab icon and make the
footer agents segment (cockpit_status index 1) clickable, reusing the existing tab-click body (right_section =
Agents; open_or_switch_cockpit(Agents); persist_right_section; notify).

**File manifest.**
- `agent_view.rs` (PURE) — add `fleet_badge` + a unit test; import nothing new (AgentStatus already in scope).
- `app.rs` (SHIM) — import fleet_badge; in the tab loop, when `tab.section == Agents` and Some(count), add an
  absolute accent badge child on the icon; in the footer loop, enumerate and wrap index 1 in a clickable div.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `fleet_badge_counts_running` | REQ-001 — None at 0; Some("1") one Working; Some("2") Working+Waiting; Idle+Exited excluded (kills the filter set, the >0 guard, the count) |
| driven capture | REQ-002 — ⌘⇧A → Agents icon "1" badge; click footer segment → Agents tab opens |
| gate --diff | REQ-003 |

**Risk.** Keep `cockpit_status` pure (Vec<String>) — the shim, not status_bar, owns the click. The badge overlay
must not steal the icon's own click (both open Agents anyway, so a small non-occluding label is fine). Running =
Working|Waiting (NOT Idle, NOT Exited) — an exited/idle-only fleet shows no badge.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` warning).
- `agent_view.rs` — `fleet_badge(agents) -> Option<String>`: count agents whose status is `Working | Waiting`
  (matches!), `(running > 0).then(|| running.to_string())`.
- `app.rs` — imported `fleet_badge`. Tab loop: `let badge = fleet_badge(&self.agents);` before the loop; each
  tab is now built as a `.relative()` `tab_el` (was inline), and when `section == Agents` and `badge` is Some,
  an `.absolute()` accent pill (`top/right -3px`, rounded_full, 9px, surface-on-accent) is added as a child;
  the existing click handler is unchanged (moved onto `tab_el`). Footer loop: `.into_iter().enumerate()`; index
  1 (agent_summary) wrapped in an `.occlude()` clickable div reusing the tab-click body (right_section = Agents
  + open_or_switch_cockpit + persist_right_section + notify); other segments stay plain strings.

**Deviation:** none. The badge is `bg(accent) text_color(surface)` for contrast; it's non-`occlude()` so the
icon's own click still lands (both open Agents anyway). Footer segment div is `.occlude()` so its click is the
handled one.

## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/#167-semantics + render/state/simplification). Both verified
concretely (read gpui source for occlude/clipping, computed WCAG luminance, ran `cargo mutants --list`,
`cargo fmt --check`). No correctness defect in the logic — fleet_badge's Working|Waiting set, the >0 guard, the
badge overlay wiring (icon click not stolen), the footer index-1 target, and Exited-exclusion (no #187
regression) all verified clean. Findings + verdicts:

- **[MED] Badge text failed WCAG AA in the LIGHT theme** — `text_color(colors.surface)` on `bg(accent)` = 4.13:1
  (< 4.5:1 AA); the light `accent` is tuned so `on_accent` (white) clears AA, and every sibling text-on-accent
  pill (the #77/#187 flash at 4825, the forge rows, buttons) uses `on_accent`. REAL (render critic, computed).
  FIXED: badge → `colors.on_accent` (one token, matches all siblings).
- **[MED] The `matches!(Working|Waiting)` filter line is NOT mutation-tested** — cargo-mutants (27.1.0) emits 0
  mutants on a match-arm line and coverage is line-based, so `Working|Waiting`→`Working` (drop Waiting) or adding
  Idle would pass BOTH gates silently. REAL gate blind-spot (correctness critic, `cargo mutants --list`
  confirmed 6 mutants, none on the filter line). No code change — the Phase-4 test's "Waiting counted" +
  "Idle/Exited excluded" asserts are LOAD-BEARING (the only thing pinning REQ-001's status set). Recorded as a
  prevention rule; honored in validate. (Corrected the notes' earlier "kills the filter set" wording — no such
  mutant exists; the hand asserts stand in for it.)
- **[LOW] Duplicated open-cockpit body** at the tab loop + the footer (6 lines each). REAL/minor (both critics).
  FIXED: extracted `open_cockpit_section(&mut self, section, cx)` (mutants::skip shim); both sites now call it —
  structurally guaranteeing the "same affordance" the ticket promises. Left the keymap `toggle-right-dock`
  partial variant alone (different Details-toggle semantics — out of scope).
- **[LOW] Magic index `1` couples the footer click to cockpit_status order.** SIGN-OFF: correct as written
  (cockpit_status = [sprint, agents, focus], locked by `cockpit_status_segments_in_order`); a comment marks it.
- **[LOW] Badge count (Working+Waiting) vs footer "n working" (Working only) can differ.** SIGN-OFF: by design —
  the ticket specifies badge = the #167 running set. Distinct concept from "actively producing output".
- **[NIT] No cursor_pointer on the footer segment; 3px badge-corner sliver falls through.** SIGN-OFF: pre-existing
  pattern (the tab icons are the same); cosmetic. Badge geometry/legibility to be confirmed in the Phase-4 capture.

Lenses: correctness, #167-semantics, mutation-surface, render/gpui-layout, accessibility (WCAG), state/liveness
(#187 interaction), simplification/reuse, fmt, secrets. Post-fix: `cargo check --workspace` + `cargo fmt --check` clean.

## Phase 4 — Validate
**Test added:** `agent_view::fleet_badge_counts_running` — zero→None; one Working→Some("1"); Waiting counted;
Working+Waiting→"2"; Idle+Exited-only→None; mixed (2 running + Idle + Exited)→"2". The per-status asserts are
the load-bearing pin for REQ-001's set (cargo-mutants doesn't mutate the `matches!` arm — inspect MED).

**cargo nextest run --workspace**: 749 passed, 5 skipped (foreground, in-transcript).

**Driven live-app capture (REQ-002)** — clean boot: top-bar 🤖 Agents icon has NO badge (0 running, correct),
footer "no agents". ⌘⇧A launched claude; once it produced output the 🤖 icon showed a **"1" accent badge**
(/tmp/mly188_badge.png; footer "1 agent · 1 working"). Clicking the footer **"1 agent" segment** opened the
**Agents cockpit tab** — a new "Agents" tab appeared in the rail + the cockpit rendered the claude row + its
#180 tail (/tmp/mly188_footerclick.png). Bonus: that capture shows the badge "1" while the footer reads
"**0 working**" (claude had gone Waiting/quiet) — the by-design Working+Waiting-vs-Working-only distinction,
proven live. Badge legible at 9px in the dark theme; on_accent contrast fix confirmed rendering.

**Gate**: `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage 100%; gate:5 mutation
6 caught / 0 missed → MSI 100.0%.

**Pre-existing exclusions**: none. (Restored `~/.marley/config/settings.toml` from the pre-test backup.)

## Phase 5 — Complete
(pending)
