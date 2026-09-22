---
pipeline_id: 37fb2e53-7306-4c93-b55c-44583a44fa7d
ticket: forge#214 (c9d31e78-86c0-4eda-9692-75876d29b5b5) · local docs/planning/tickets/open/TICKET-214-warp-osc8-hyperlinks.md
aar_id: 91bfb0b8-a74d-45a1-af6a-08544f740b7c
status: Phase 5 — Complete PASS
title: Honor OSC 8 explicit hyperlink escapes in terminal output
type: feature
milestone: M12.2
references: []
---

## Title
Carry an OSC 8 explicit hyperlink (`ESC]8;;URI ST … text … ESC]8;; ST`) from the alacritty
grid cell through the `terminal_blocks` styled-output model, and make the `marley_app` render
treat an explicit hyperlink as authoritative over the #196 text-scan heuristic — so output where
the DISPLAY TEXT differs from the TARGET URI (the whole point of OSC 8, e.g. `click` →
`https://example.com`) renders as a correct clickable link. The deferred half of #196 (which
shipped text-scan links but explicitly punted OSC 8 to "a separate `terminal_blocks` change",
links.rs:7-8).

## Scope
### In
- **`terminal_blocks/src/styled.rs` (PURE):** `StyledRun` gains `hyperlink: Option<String>`;
  `coalesce_row` carries a per-cell URI and BREAKS a run on a hyperlink change (not just fg/bg/flags).
- **`terminal_blocks/src/session.rs` (SHIM):** the two grid-read cell-tuple sites (~:428 block-output
  capture, ~:450 `grid_styled_rows`) extract `cell.hyperlink().map(|h| h.uri().to_string())`.
- **`marley_app/src/links.rs` (PURE):** a composer that PREFERS an explicit per-run hyperlink over
  `scan_links` for that run (the OSC 8 span is authoritative), reusing the #196 `Link`/`LinkTarget`.
- **`marley_app/src/app.rs` (SHIM):** the output render reads `run.hyperlink` and feeds the composer;
  the click path is unchanged (`marley_command::open_url`).

### Out (explicitly deferred)
- OSC 8 `id=…` grouping params + joining a hyperlink across wrapped/multiple lines (v1 = per-run URI;
  a wrapped hyperlinked run yields one link per line — acceptable).
- A hyperlink-id → URI intern map (v1 stores the URI inline as `Option<String>` on the run — D1).
- Link *styling* (underline/accent color) — that is #196 render territory, not new here.
- Re-parsing the OSC 8 escape bytes in Marley — alacritty already parses them into `cell.hyperlink()` (D5).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — inline `Option<String>` URI on `StyledRun`**, not an id→uri intern map. Simplest; a run maps to
  exactly one URI; hyperlinked runs are rare so the per-run `String` is a bounded cost. (Ticket offered
  either; pick inline for v1.)
- **D2 — the coalesce break compares by `as_deref()`** (`run.hyperlink.as_deref() == uri.as_deref()`) so
  a same-URI cell does not clone the `String` on every compare; the owned `String` is materialized only at
  a run-break. (Design confirms the exact expression against the real mutant set.)
- **D3 — an explicit OSC 8 hyperlink is AUTHORITATIVE** over the text-scan heuristic for its run: the run's
  whole character range → the URI verbatim, with NO re-scan of the display text (which may not look like a URL).
- **D4 — `output_text` / the flattened run text stays BYTE-IDENTICAL** (R20b): a mid-run hyperlink change
  splits one run into two, but the concatenated text is unchanged. This is a hard regression guard (assert
  the RUN STRUCTURE changed AND the joined text did not — the raw representation, not the normalizing
  projection; #50 lesson).
- **D5 — the URI is taken verbatim from alacritty's parsed `Hyperlink::uri()`** — Marley does not re-parse
  the escape. Clean-room §20: OSC 8 is a published terminal standard; alacritty (Apache-2.0/MIT) does the parse.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a row's cells carry an OSC 8 hyperlink URI, `coalesce_row` shall set the resulting `StyledRun`'s `hyperlink` to `Some(uri)` for hyperlinked cells and `None` otherwise. | styled.rs unit; cov/MSI 100 |
| REQ-002 | WHEN two horizontally-adjacent cells share fg/bg/flags but carry DIFFERENT hyperlinks (incl. `Some` vs `None`), `coalesce_row` shall split them into SEPARATE runs. | styled.rs unit (run count + boundaries); mutation kills the `== hyperlink` guard |
| REQ-003 | WHILE cells carry hyperlinks, the flattened `Block::output_text` (and the joined run text) shall remain byte-identical to the pre-#214 output for the same cells; AND cells with no hyperlink shall coalesce exactly as before. | styled.rs unit asserting BOTH run structure (2 runs) AND concatenated text == the single-run text; existing coalesce tests stay green |
| REQ-004 | WHEN the alacritty grid cell has a hyperlink, the session grid-read shall carry `cell.hyperlink().uri()` into the resulting `StyledRun.hyperlink`. | terminal_blocks integration test: feed real OSC 8 bytes through a `Term` → assert a `StyledRun` carries the URI |
| REQ-005 | WHEN a rendered output run carries an explicit hyperlink, the link composer shall emit a `Link` over the run's full range targeting that URI (authoritative), while a run WITHOUT a hyperlink shall still yield its `scan_links` result at the correct line offset. | links.rs unit (a `scan_links`-invisible display text like `click` → one Url link; multi-run offset); cov/MSI 100 |
| REQ-006 | WHEN a program emits an OSC 8 hyperlink (`printf '\e]8;;https://example.com\e\\click\e]8;;\e\\\n'`), Marley shall render `click` as a clickable link opening `https://example.com`. | Driven capture if unlocked; ELSE REQ-001..005 units + the REQ-004 integration test + the mechanism (reuses the shipped #196 `Link`→`open_url`) — env-blocked fallback per the locked-screen rule |

## Phase Plan
- **P2 Design** — confirm the `coalesce_row` tuple/signature + the `as_deref` break expression; confirm the
  session.rs grid-read sites are `mutants::skip`/cov-excluded vs R26-covered (drives whether an integration
  test is REQUIRED for REQ-004); shape the links.rs composer (per-run range math); reassess the ticket's
  capture-vs-render split (expected: one slice). Regression test plan from the real `cargo mutants --list`
  on styled.rs + links.rs.
- **P3 Implement** — styled.rs (pure) first, then session.rs (shim), then links.rs (pure), then app.rs (shim);
  `cargo check --all-targets` guides the ripple (the `StyledRun` field breaks `plain_lines` + the coalesce
  test literals — compile-fix with `hyperlink: None`).
- **P3.5 Inspect** — independent critics vs the diff (byte-identity regression; the guard mutant set; the
  render preferring explicit; clean-room).
- **P4 Validate** — write + RUN the tests; gate green [diff]; driven capture or env-blocked fallback.
- **P5 Complete** — CHANGELOG + app_shell.md / terminal-blocks doc; AAR capture; close #214; archive.
