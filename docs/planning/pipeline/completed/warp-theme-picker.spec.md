---
pipeline_id: 5c44c7df-5e40-4d0a-bfe5-d4ccff8fbc75
ticket: forge#199 (2223ad76-e2f3-4f93-92e6-ff1ec27905c8) · local docs/planning/tickets/open/TICKET-199-warp-theme-picker.md
aar_id: fd3b6cad-19c8-40d3-90fe-69be0de38cb4
status: Phase 5 — Complete PASS (all phases PASS; 304 tests, MSI 100; live capture confirms the theme rows + a live dark→light switch)
title: Live theme picker in the command palette
type: feature
milestone: M12.2
references: []
---

## Title
Switching theme needs hand-editing the TOML today. Add a command-palette entry per theme ("Theme: <name>")
that applies it instantly + persists the choice. Most infrastructure already exists — the `ThemeRegistry`
(Dark+Light), `set_theme` (applies live + persists), and the `persist_theme`/boot-load round-trip. The
missing bit is the picker: dynamic palette commands mirroring the #87 saved-host `CONNECT_BASE` pattern.

## Scope
### In
- One palette `Command` per `ThemeRegistry` theme — `id: CommandId(THEME_BASE + i)`, `title: "Theme: <name>"`
  — pushed beside the #87 connect commands in the constructor.
- Dispatch in `handle_palette_key` (beside the `CONNECT_BASE` `checked_sub` branch): a `THEME_BASE` branch
  resolving the id → the i-th registry theme → `self.set_theme(&theme, cx)` (applies live + persists), then
  close the palette.
- A pure index helper (cov/MSI 100) for the id↔theme-index resolution (mirroring the connect index math),
  exact-value tested (in-range → Some(i), below-base/out-of-range → None).

### Out (explicitly deferred)
- Live PREVIEW on highlight (arrow-key) — the ticket says "optional"; apply-on-Enter is the core. A follow-up.
- A custom-theme editor / new themes beyond the builtin Dark/Light (M12.2 terminal-black is the Dark default).
- Reworking `set_theme`/`persist_theme`/the registry — all already exist + tested.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — `THEME_BASE` is a distinct id block above `CONNECT_BASE` (1000) — pick 2000 (a handful of themes;
  connect uses 1000+i for N hosts, no realistic collision). Confirm no overlap at design.
- **D2** — apply via the EXISTING `set_theme` (which already persists) — no new apply/persist logic.
- **D3** — the id↔index mapping goes through a small PURE helper (like the connect `checked_sub`), cov/MSI
  100 — the untestable dispatch shim stays thin.
- **D4** — auto-approved (/work 195–222): document with the live theme-switch capture.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the command palette is open, it shall list one "Theme: <name>" command per registry theme. | Driven capture (⌘⇧P → "theme" → the theme rows) + review |
| REQ-002 | WHEN a "Theme: <name>" command is selected, the cockpit shall switch to that theme's colors immediately. | Driven capture (select Light → light colors live) |
| REQ-003 | WHEN a theme is selected, the choice shall persist (survive relaunch) via the existing settings round-trip. | Existing `persist_theme`/load round-trip tests + review (dispatch calls `set_theme`) |
| REQ-004 | The id↔theme-index resolution shall map a `THEME_BASE`-range id to the correct theme index and reject out-of-range/below-base ids. | Unit test (pure helper, exact-value, cov/MSI 100) |
| REQ-005 | The change shall not alter the existing cockpit commands, the `CONNECT_BASE` connect commands, or `set_theme`/`persist_theme`. | Review + capture |

## Phase Plan
- **P2 Design** — confirm no `CONNECT_BASE`/`THEME_BASE` id collision; the pure index-helper signature
  (`theme_pick_index(id, base, count) -> Option<usize>` or similar) + test matrix; the command-registration
  loop + the dispatch branch; where the registry is reachable at dispatch time; file manifest + test plan.
- **P3 Implement** — the `THEME_BASE` const + the pure helper + the command loop + the dispatch branch.
- **P3.5 Inspect** — critics: no id collision with connect, the dispatch resolves the right theme, `set_theme`
  persists, no regression to the static/connect commands; clean-room.
- **P4 Validate** — the pure helper unit test + a driven capture (theme rows in the palette; select → live
  color switch); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
