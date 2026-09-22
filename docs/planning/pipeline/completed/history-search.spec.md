---
pipeline_id: 35c78343-8ecc-48f4-8758-639803987313
ticket: forge#60 (9620ea3a-0ade-44c7-acb0-8945b8f64adb) · local docs/planning/tickets/open/TICKET-060-history-search.md
aar_id: a697bf07-4d48-4ba3-b114-34ed89c9844c
status: Phase 5 — Complete PASS
title: command history fuzzy search (cmd-R)
type: feature
milestone: M2.B
references:
  - crates/marley_app/src/history.rs (PURE: CommandHistory::recent)
  - crates/marley_app/src/keymap.rs (PURE: cmd-r → history, rerun-last → cmd-shift-r)
  - crates/marley_app/src/app.rs (SHIM: cmd-R overlay + cooked-buffer insert)
---

## Title
cmd-R opens a fuzzy search over the session's COMMAND HISTORY (the conventional shell reverse-search),
ranked by `marley_search_core`; Enter inserts the chosen command at the prompt. The 2nd search consumer.

## Scope
### In
- PURE-1 (`history.rs`, cov/MSI 100): `pub fn CommandHistory::recent(&self) -> Vec<&str>` — the entries
  MOST-RECENT-FIRST, DEDUPED (keep the most-recent occurrence) via `entries.iter().rev().filter(seen.insert).map`.
- PURE-2 (`keymap.rs`, cov/MSI 100): cmd-r → `open-history-search` (bare); rerun-last MOVES to
  `cmd-shift-r`; the keymap test asserts both.
- PURE-3: REUSE `FinderState` (finder.rs #57) for the history-search state — no new state type.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView { history_open, history_finder: FinderState }`;
  dispatch `open-history-search`; key routing gates `history_open` first; `handle_history_key` (Enter
  inserts the chosen command into the COOKED buffer per #59, NOT raw write); the overlay mirrors the
  cmd-P finder; the source is the focused session's `history.recent()`.

### Out
- Run-on-Enter (first cut inserts, like a shell you can edit-then-run). Cross-session/persistent history.
  History dedup policy beyond most-recent-unique.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 (FORK RESOLVED) — cmd-R = history search (conventional); rerun-last → cmd-shift-r (both kept; the ↻
  block click already covers per-block rerun, dispatch arm app.rs:689 stays reachable via cmd-shift-r).
- D2 — REUSE `FinderState` (the history commands are `&str`, exactly its input) — one finder model.
- D3 — Enter inserts into the cooked prompt buffer (`buffer.edit`), NOT `write_bytes` — per #59's
  cooked-buffer lesson (else invisible in cooked mode).
- D4 — `recent()` = most-recent-first + dedup-keep-most-recent (a shell reverse-search shows the newest
  unique commands).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `recent()` runs on entries `[a,b,a]`, it shall return `[a,b]` (most-recent-first, the older `a` deduped); empty → `[]`. | unit |
| REQ-002 | WHEN the keymap is queried, cmd-r shall map to `open-history-search` and cmd-shift-r to `rerun-last`. | unit |
| REQ-003 (visual) | WHEN cmd-R is pressed after running commands, an overlay shall list the ranked history; Enter shall insert the chosen command at the prompt. | self-test (drive commands, cmd-R, capture; Enter → prompt) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on recent() + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `recent()` shape, the keymap swap, the FinderState reuse + the cmd-R shim (overlay + cooked
  insert), mutation targets, unit + self-test plan.
- **P3** — recent() + keymap change + the app.rs cmd-R shim.
- **P3.5** — critic: recent() order/dedup, the keymap non-conflict (cmd-r vs cmd-shift-r), the cooked
  insert (per #59), the shim seam, mutants.
- **P4** — recent() + keymap unit tests (cov/MSI 100) + `-p marley` green + the SELF-TEST (cmd-R overlay
  + Enter inserts) + gate GREEN.
- **P5** — docs, AAR, archive, close #60.
