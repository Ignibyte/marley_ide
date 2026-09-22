---
pipeline_id: 22ddc9eb-4c2d-4db0-9f05-28737c200c25
ticket: forge#106 (387b845e-4e88-4bc6-a2b2-e62ac7f7f195) · local docs/planning/tickets/open/TICKET-106-viewer-guards.md
aar_id: 39e94a27-8066-45bd-b2cd-4dc0dcdba0f9
status: Phase 5 — Complete PASS
title: viewer guards + persistence (M4 FINALE)
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_view.rs (PURE: is_probably_binary, viewer_size_ok)
  - crates/marley_app/src/settings.rs (CodeTabWidth read-only setting + applied.code_tab_width)
  - crates/marley_app/src/app.rs (SHIM: open_file_in_viewer guards + placeholder; stores code_tab_width)
---

## Title
The M4 finale: guard the code viewer against binary/huge files (a placeholder, not a hang), and let the
viewer's tab width be configured (`code.tab_width`, respected across launches).

## Scope
### In
- PURE `is_probably_binary(bytes)` (NUL in the first 8000) + `viewer_size_ok(len, max)` (≤ cap).
- SETTINGS: read-only `code.tab_width` (u16, default 4) → `applied.code_tab_width` (clamped ≥1).
- SHIM: open_file_in_viewer reads bytes → guards → open with the configured tab width, else flash a placeholder.

### Out
- A File/Diff view-mode setting (dropped — #103 folded File↔Diff into the ⌘⇧D overlay; no toggle to persist).
- A UTF-8-ratio binary heuristic (NUL-byte is enough for v1). Actual soft-wrap.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `is_probably_binary` = a NUL byte in the first 8000 bytes; `viewer_size_ok(len, max)` = `len <= max`
  (cap `VIEWER_MAX_BYTES = 2 MiB` in the shim).
- D2 — `code.tab_width` is a READ-ONLY config setting (mirrors `terminal.cols` — no persist fn, so no dead
  code); clamped ≥1 so tab-expansion can't `%0`-panic. The user sets it in `settings.toml`.
- D3 — a guarded-out file shows a status flash ("can't open … (binary or >2MB)"), not the viewer.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `is_probably_binary(bytes)` sees a NUL in the first 8000, it shall be true (else false). | unit |
| REQ-002 | WHEN `viewer_size_ok(len, max)` runs, it shall be `len <= max`. | unit |
| REQ-003 | WHEN `code.tab_width` is set in TOML, `applied.code_tab_width` shall be it (clamped ≥1). | unit |
| REQ-004 (visual) | WHEN a binary/huge file is opened, a placeholder flash shall show, not the viewer. | self-test (engine) |
| REQ-005 | gate GREEN, cov/MSI 100 on the pure fns + the setting; the shim masked. | gate |

## Phase Plan
- **P2** — the two guards; the CodeTabWidth setting + applied wiring; the open-guard shim; test plan.
- **P3** — implement (code_view.rs + settings.rs + app.rs).
- **P3.5** — 1 critic: guards + tab_width MSI; the open guard; the read-only setting.
- **P4** — the guard + settings tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, architecture doc, archive, close #106 → **close M4 sprint #15**.
