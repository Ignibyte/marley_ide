---
pipeline_id: c7b460d4-5479-4989-b11a-d03ceeeb504b
ticket: forge#347 (1df08cb6-9ac1-4cd1-ad0c-820386f268a2) · local docs/planning/tickets/open/TICKET-347-regex-capture-replace.md
aar_id: 3b62706e-970a-47e3-82d4-e12f82d7ea2c
status: Phase 5 — Complete PASS
title: Regex REPLACE — capture groups ($1/${name}/$$) + the two #339 silent-corruption fixes (F2 expanded-length, F3 empty-match advance)
type: feature
milestone: M22
references: [find.rs find_all_regex/FindError, app.rs Replace-One caret+efind_resume (F2) / Replace-All partition_point resume (F3) / the mutants::skip replace shim, buffer.rs replace_all + begin_undo_group(cursor_anchored:false), regex-1.12.4 Captures::expand (string.rs:1884), AD-claude-edit-post-state-span-and-earned-undo-tags-001, #339 regex-find (D-SAME-SHAPE/D-BYTE-CHAR-AT-ONE-SEAM/D-EMPTY-ADVANCE), #336 chars().count() cov-blind-shim]
---

## Title
Add capture-group REPLACE to the find bar — `$1` / `${name}` / `$$`, adopting the `regex` crate's own
`Captures::expand` — and fix the two PRE-EXISTING silent-corruption bugs #339 surfaced but could not reach with a
literal-only Replace: **F2** (Replace-One's caret + `efind_resume` are computed from the *template* length, not
the *expanded* text) and **F3** (an empty regex match never advances the resume cursor → the resume mechanism's
own infinite loop resurrected).

## Scope
### In
- A pure `expand_captures` seam — a THIN adapter over `regex::Captures::expand` (the crate owns the `$1`/`${name}`/
  `$$` grammar), returning the expanded `String` for one match + template. cov/MSI 100.
- A pure `replace_all_with(buffer, &[(range, String)]) -> usize` primitive — per-match owned replacements applied
  back-to-front in ONE undo group carrying `restore` (cursor-anchored:false), mirroring the existing `replace_all`.
  cov/MSI 100.
- The read→write wiring for capture replace (THE FORK — see D2): the app produces per-match expansions from the
  live query + template and drives the buffer primitive.
- **F2 fix** — Replace-One's post-replace caret AND `efind_resume` use the EXPANDED text's char length.
- **F3 fix** — Replace-One / the Replace resume advances past a zero-width match (no re-hit, no loop).

### Out (explicitly deferred)
- Any change to LITERAL replace behavior — it stays byte-identical (a literal template's expanded length == its
  own length, so the F2 fix is a no-op for it; D-LITERAL-UNCHANGED).
- A replace PREVIEW / confirm-each-match UI, replace-in-selection scoping, replace history — not this ticket.
- The regex FIND path's match semantics (shipped in #339) — unchanged except as the FORK (D2) requires to reach
  the captures.

## Reference (§20)
N/A — Marley-specific. The find/replace bar is Marley's own editor surface (no Warp/Zed source read). "Capture-group
replace with `$1`/`${name}`" is a universal editor expectation; the `regex` crate's `Captures::expand` grammar is
the de-facto standard and is ADOPTION (outside the wall).

### Prior art
1. **★ THE `regex` CRATE OWNS THE EXPANSION (the highest-yield leg — a decision that dissolves into adoption).**
   `regex 1.12.4` ships `Captures::expand(&self, replacement: &str, dst: &mut String)`
   (`~/.cargo/registry/src/…/regex-1.12.4/src/regex/string.rs:1884`), whose documented grammar is EXACTLY the
   ticket's ask: unbraced `$ref` (longest `[0-9A-Za-z_]`, numeric → index else name), braced `${ref}`, invalid
   ref → empty string, `$$` → literal `$`. So `expand_captures` is a THIN adapter, NOT a reimplementation —
   the same win #339 recorded when the crate dissolved D-EMPTY-ADVANCE. **Caveat that FEEDS the fork:** a
   `Captures<'h>` BORROWS the haystack, so it cannot be stored in a find-result `Vec` that outlives the borrow —
   "carry the captures" therefore means carry OWNED group byte-ranges (and re-interpolate) OR re-run the regex at
   replace time (live `Captures`, direct `expand`). D2 settles which.
2. **OUR OWN CODE (#339):** the regex FIND path (`find.rs` `find_all_regex`/`FindError`) + the app's `efind` state
   + the existing literal `replace_all` (`buffer.rs`, `begin_undo_group(cursor_anchored:false)`) are the seams to
   extend. #339's D-SAME-SHAPE (the read plumbing is mode-agnostic) is TRUE for find and the reason the fork
   exists (the shape carries by discarding captures). #338's `AD-claude-edit-post-state-span-and-earned-undo-tags-001`
   governs the undo-group `restore` tag.
3. **gpui / ropey / alacritty:** checked — no owner for capture-replace (this is a `regex` + our-buffer seam).

## Locked-In Decisions
- **D1-ADOPT-EXPAND** — `expand_captures` is a THIN adapter over `regex::Captures::expand`; do NOT hand-roll the
  `$1`/`${name}`/`$$` grammar (the #339 crate-owns-it lesson). The pure seam is the adapter + its call site.
- **D2-THE-FORK — read-carries-captures (A) vs re-run-at-replace (B): OPEN, settled at P2. RECON LEANS B.** The
  find path returns `(CharOffset, CharOffset)` from `re.find_iter` and DISCARDS captures (`find.rs:253-256`); the
  compiled `Regex` is built PER-CALL and NEVER stored (`find.rs:211`, a local); the app memoizes only
  `efind_matches: Rc<Vec<(usize,usize)>>` (`app.rs:486`). So there is NO live `Captures` and NO stored `Regex` at
  replace time. (A) grow the read result to carry OWNED capture byte-ranges — a wider type rippling through the
  `(CharOffset,CharOffset)` shape that bands / n-of-m / navigation consume UNCHANGED (#339 D-SAME-SHAPE) + the
  memo key. (B) re-run the regex at replace time for live `Captures` + `expand` — and since the regex is ALREADY
  recompiled per call (nothing to reuse either way), B adds no *stored* state, only a second match pass. The
  D-BYTE-CHAR-AT-ONE-SEAM cost of B is CONTAINED if the re-run + the byte↔char conversion live in ONE pure fn in
  `find.rs` (e.g. `replace_all_regex(text, pattern, ci, template) -> Result<Vec<(range, String)>, FindError>`),
  not duplicated into the cov-blind app.rs. **P2 confirms B (or picks A with reason);** the `Captures`-borrows-
  haystack finding rules out "store the `Captures` directly" regardless.
- **D3-REPLACE-ALL-WITH** — the new `replace_all_with(buffer, &[(CharOffset..CharOffset, String)]) -> usize`
  applies owned per-match replacements back-to-front in ONE undo group via `begin_undo_group(sel)` (which passes
  `cursor_anchored: false`, `buffer.rs:732` → `undo.rs`) — carrying `restore`, per
  `AD-claude-edit-post-state-span-and-earned-undo-tags-001`. It is modeled EXACTLY on the existing literal
  `replace_all` (**which lives in `find.rs:114`, NOT buffer.rs — the ticket was wrong**) and built on
  `Buffer::edit` (`buffer.rs:250`). Home: `find.rs` (pure, cov/MSI 100).
- **D4-EXPANDED-LENGTH (F2)** — Replace-One's post-replace caret (`app.rs:2233`) AND `efind_resume`
  (`app.rs:2247`) are computed from the number of chars actually INSERTED (the expanded text), never
  `repl.chars().count()` (the template). Literal templates are unaffected (expanded length == own length).
- **D5-ADVANCE-EMPTY (F3) — UNIFIES WITH F2.** The zero-width guard ALREADY exists but keys off the *template*:
  `resume = if s == e && repl.is_empty() { s + 1 } else { s + repl.chars().count() }` (`app.rs:2245-2248`). Under
  capture expansion that is wrong twice: the length must be the EXPANDED text's (F2), AND the emptiness check must
  be the EXPANDED text's — a zero-width match whose expansion is empty (e.g. template `${invalidref}` → "") must
  still advance `s + 1`, but today `repl.is_empty()` is false (the template is non-empty) so it would use the
  template length. Fix: `let inserted = expanded.chars().count(); resume = if s == e && inserted == 0 { s + 1 }
  else { s + inserted }`. (FIND is already safe — `find_iter` force-advances on an empty overlap; this is the
  REPLACE loop, ours. The literal empty-match case is already handled; the capture-era case is the delta.)
- **D-LITERAL-UNCHANGED** — literal (non-regex) replace stays byte-identical; the capture path is reached only in
  regex mode.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-005-EXPAND | WHEN a capture template (`$1`/`${name}`/`$$`) is applied to a match, the system shall produce the regex-crate expansion of that template against that match's captures. | pure unit on the `find.rs` expand/replace seam: `(\w+)@(\w+)` + `${2}_${1}` on `ab@cd` → `cd_ab`; `$$` → `$`; an invalid ref → empty. cov/MSI 100. |
| REQ-005-UNDO | WHEN Replace-All applies N expansions, the system shall apply them back-to-front as ONE undo unit that carries `restore` (cursor-anchored:false). | pure `replace_all_with` unit (ordering + one group, mirroring the `replace_all` test) + a headless drive (model: `editor_replace_flows_headless` @ headless_drive.rs:1111) — regex replace-all, assert buffer text, then ONE `cmd-z` restores. |
| REQ-010-F2 | WHEN Replace-One replaces a match, the system shall set the caret AND `efind_resume` from the EXPANDED text's char length (not the template's). | ★ headless drive (keystroke path, model :1111 + hooks `efind_refresh_for_test`/`efind_index_for_test` @ app.rs:12985/13000): regex `(\w+)@(\w+)` + `${2}_${1}` on a **≥3-match fixture with a surviving MIDDLE match — `ab@cd e@f g@h`** (inspect F1: a 2-match fixture is masked by the `.min(len-1)` resume clamp — after match 0 self-destroys its `@` the set drops to 1 so any resume clamps to index 0). Replace-One → the MIDDLE match `e@f` IS reached + replaced; caret at the replacement end. NEGATIVE-SMOKE (revert to template length → resume overshoots, `e@f` left unreplaced → texts DIFFER → fails). A literal-only fixture CANNOT prove it. |
| REQ-011-F3 | WHEN Replace-One acts on a zero-width match whose expansion is empty (e.g. `(?m)^` + an empty/invalid-ref template), the system shall advance the resume cursor so the same match is not re-selected. | ★ headless drive (model: `efind_empty_match_replace_advances_headless` @ headless_drive.rs:6172): regex `(?m)^` + template **`${9}`** (invalid ref → empty expansion) on a **short-line fixture `a\nb\nc`** (inspect F2: over 2 lines the reverted guard's overshoot is invisible). Replace-One advances to the IMMEDIATELY-NEXT zero-width match — index steps 0→(offset 2)→(offset 4), every line-start. NEGATIVE-SMOKE (key the guard off the template's emptiness → `${9}` len 4 overshoots `s+4`, SKIPS the line at offset 2 → the asserted immediately-next target fails). |

## Phase Plan
- **P2 Design** — ★ SETTLE D2 (A vs B) with evidence from the `find_all_regex`/`FindError` shape + the `efind`
  flow + the second-seam risk; pin the `expand_captures` adapter signature (does it take `&Captures` or owned
  ranges — ties to D2) + the `replace_all_with` signature + the undo-group compliance + the exact F2/F3 fix sites
  (verified line numbers) + the headless-drive test plan (each REQ, each NEGATIVE-SMOKED).
- **P3 Implement** — `expand_captures` (adapter) + `replace_all_with` (buffer) + the fork's read/replace wiring +
  the F2 + F3 fixes at source.
- **P3.5 Inspect** — the fork's second-match-site drift (if B) or the wider-type thread (if A); F2 expanded-length
  at BOTH the caret and the resume; F3 advance; the undo `restore` tag; ★ that the headless drives actually
  exercise the cov-blind app.rs shim (the #336 lesson).
- **P4 Validate** — the pure `expand_captures`/`replace_all_with` units + the negative-smoked headless drives +
  the `--diff` gate (cov/MSI 100 on the pure seams; app.rs shim mutants::skip).
- **P5 Complete** — CHANGELOG + find.rs/app.rs docs + editor.md (the find-bar section: capture replace + the two
  fixes); AAR capture; close the ticket.
