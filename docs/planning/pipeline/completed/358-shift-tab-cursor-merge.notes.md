# ⇧Tab cursor merge (2→1) — Notes

- **Forge ticket:** #358 `ae7dbf9e-c8ef-4b48-87fb-5ad1ac5f0349`
- **AAR:** `d08e90f8-d95a-45e0-a8e8-a35d4d62fd67`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-358-shift-tab-cursor-merge.md
- **Pipeline spec:** 358-shift-tab-cursor-merge.spec.md
- **pipeline_id:** ead13037-dd2c-45cd-9232-743f5c68c7ad · on `b6ca633`

## Phase 1 — Plan

**Request:** ⇧Tab dedent merges two carets that both sit inside the same row's indentation (2→1). #307 follow-up,
low-priority, from-inspect, "arguably correct". A DESIGN-FORK ticket (3 options, "do not pre-judge").

**Classification / tier:** work pipeline, one shippable slice (a documenting test + a regression guard + a doc
note — NO behavior change, since the recon shows the behavior is already correct).

**Forge recall (§18.3):** no bulletins. `knowledge-context` (Plan) surfaced 13 nodes (3 ADRs, 3 distilled lessons,
5 prevention rules, 2 failures — the #297/#307 multi-cursor decisions + the covering-collapse convention). Logged
to the AAR.

### ★ Recon — the fork DISSOLVES to Opt 1 (accept + document)

**The mechanism (confirmed live on `b6ca633`):** `rebase_through(pos, edits)` (indent.rs:150), `LineEdit =
(at, remove, text)` (`at` = line_start, `remove` = the stripped indent count, `text` = "" for a dedent). The clamp
(:158): `if p < at + remove { return CharOffset::from((at as isize + delta).max(0) as usize); }` — a position
inside `[at, at+remove)` clamps to the shifted `at`. Two carets inside the same row's stripped indent → the same
shifted `at` → `rebase_selections` (:175) writes them through `SelectionSet::from_selections`, which merges
same-offset carets by the `overlaps` `<=` caret law (selection.rs). The merge is the crate's law, not a bug.

**★ THE FORK DISSOLVES (the decisive finding — trace on the REAL fn):**
- **Opt 2 (co-located cursors)** contradicts `from_selections`' whole reason to exist (a same-offset pair inserts
  its text twice) → WRONG.
- **Opt 3 (`line_start + (col - stripped).max(0)`) is a PURE NO-OP.** For a caret INSIDE the removed span,
  `col = p - at` is by definition `< remove` (that IS "inside `[at, at+remove)`"), so `(col - remove).max(0)` is
  ALWAYS 0 → `at + 0` = `at` = the current clamp EXACTLY. Reported case: at=3, remove=4, carets at cols 1 and 3 →
  `(1-4).max(0)=0` and `(3-4).max(0)=0` → both → offset 3 → STILL merge. For a caret OUTSIDE the span,
  `rebase_through` never reaches the clamp (it takes `p + delta`). So Opt 3 changes NOTHING for any caret. It is
  not even a partial fix.
- **Topologically forced:** after removing the N indent chars, the columns the two carets held (`[line_start,
  line_start+N)`) NO LONGER EXIST — there is no distinct position for a second cursor. NO clamp formula can keep
  them distinct. One cursor at the new line start is the only well-defined outcome.
- ⟹ **Opt 1 (accept + document) is the ONLY correct answer.** The merge is correct + recoverable (an EDIT → ⌘Z
  restores the 2-cursor set, unlike the #297 MOTION scar). The shippable slice = a documenting drive + a
  regression guard + a doc note. NO behavior change.

**Test helpers (confirmed):** `open_editor_file(cx, name, content)` → `(_tmp, window, vcx, _f)`;
`set_editor_carets(&window, &mut vcx, &[offsets])` (headless_drive.rs:6917); `editor_text` + `editor_selections`
(:403, `.len()` = the cursor count); `simulate_keystrokes("shift-tab" / "cmd-z")`. ⇧Tab is a SYNCHRONOUS edit —
NO `run_until_parked`/pump tick needed (the #349 mock-clock trap does NOT apply). The existing
`shift_tab_dedents_every_cursors_line_headless` (:8192, carets `[4,11,18]`) is the 3-row model + regression guard
(it asserts text + undo-text; #358 ADDS the cursor-count assertion).

**Decisions recorded:** D1-ACCEPT (merge topologically forced + recoverable), D2-OPT3-IS-A-NO-OP (traced),
D3-NO-BEHAVIOR-CHANGE (test + doc note only). See the spec.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Approach.** Opt 1 (accept + document), confirmed by the recon. NO behavior change: the deliverable is a doc note
in `marley_editor` + two headless drives in the app. §14 holds trivially (no new code path); §20 = N/A (Marley's own
multi-cursor/dedent; the "cursors in deleted whitespace collapse" outcome matches mainstream editors — confirmation,
not a source).

**★ D2 ratified airtight (traced on the real `rebase_through`, indent.rs:150-164):**
- `LineEdit = (at, remove, text)`; the clamp (:158): `if p < at + remove { return CharOffset::from((at as isize +
  delta).max(0) as usize) }`; the outside path (:161-163): `delta += text.len - remove; ... p + delta`.
- Reported case (`"aa\n    bb\ncc\n"`, at=3=row-1 start, remove=4, delta=0 at that edit): p=4 → `4 < 7` → returns
  `at`=3; p=6 → `6 < 7` → returns 3. Both → 3 → `from_selections` merges (the `overlaps` `<=` caret law). ✓
- **Opt 3 ≡ current, for BOTH paths:** an INSIDE caret (`p < at+remove`, i.e. `col = p-at < remove`) → Opt 3 gives
  `at + (col - remove).max(0)` = `at + 0` (col<remove ⟹ negative ⟹ 0) = `at` = the current clamp. An OUTSIDE caret
  (`p ≥ at+remove`) never reaches the clamp arm — both current and Opt 3 take `p + delta`. So Opt 3 is a genuine
  no-op for every caret. Inspect can re-derive this; there is no surprise.

**Helpers confirmed (headless_drive.rs):**
- `editor_selections` (:403) returns `Vec<(usize, usize)>` = per-cursor `(start, end)` offsets. `.len()` = the
  cursor count; a caret is `(o, o)`. So T1 can assert the EXACT merged offset `[(3,3)]` and the restored
  `[(4,4),(6,6)]`.
- The existing `shift_tab_dedents_every_cursors_line_headless` (:8192) is SYNCHRONOUS — `simulate_keystrokes(
  "shift-tab")` then assert, NO `run_until_parked`. ⇧Tab is an immediate edit → NO pump tick (the #349 mock-clock
  trap does NOT apply). `open_editor_file` + `set_editor_carets` + `editor_text` are the rest of the idiom.

**File manifest:**
| File | Change |
|---|---|
| `crates/editor/src/indent.rs` | `rebase_selections` doc (:166-174) gains a paragraph: two carets inside ONE row's removed indent collapse to one cursor — correct (the columns are deleted; nowhere distinct to go), NOT the #297 silent-loss class because it is an EDIT (⌘Z restores the set). Plain backticks. No code change. |
| `crates/marley_app/src/headless_drive.rs` | ADD T1 (merge + ⌘Z-restore) + T2 (3-row regression guard) `#[gpui::test]` drives, modeled on `shift_tab_dedents_every_cursors_line_headless`. |

**Regression Test Plan:**
| # | Test | Kind / home | Proves |
|---|------|-------------|--------|
| T1 | `shift_tab_merges_two_carets_in_one_indent_and_undo_restores_headless` | headless, app | REQ-MERGE-CORRECT (`[4,6]` → `shift-tab` → text `"aa\nbb\ncc\n"` + `editor_selections()==[(3,3)]`, exactly 1 cursor at the new line start) + REQ-RECOVERABLE (`cmd-z` → text restored + `editor_selections()==[(4,4),(6,6)]`, the 2-cursor set back) |
| T2 | `shift_tab_keeps_all_cursors_on_separate_rows_headless` | headless, app | REQ-COMMON-CASE-SAFE (`[4,11,18]` on `"    aa\n    bb\n    cc\n"` → `shift-tab` → text `"aa\nbb\ncc\n"` + `editor_selections().len()==3`, all survive — the #297-scar guard that no OTHER cursor is lost) |

No new pure-fn surface → cov/MSI unchanged: `rebase_through`/`rebase_selections` are UNTOUCHED (their existing
cov/MSI-100 tests stay green); the app ⇧Tab arm is `mutants::skip`/cov-excluded → the drives carry it. NO live
synthetic drive (headless = `cargo nextest`, safe with chad at the machine). NO uncoverable path.

**Risks / decisions:** (a) the #297 scar → the COUNT assertions (T1 `len==1` documented-correct, T2 `len==3`
guard). (b) `editor_selections` exposes offsets → T1 asserts the exact merged/restored positions, not just the
count. (c) synchronous ⇧Tab → no pump tick. (d) NO behavior change (D3) — a doc note + tests only. §14 trivial;
§20 = N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest, 2 files. `cargo fmt --all`; `cargo check -p marley -p marley_editor --all-targets` CLEAN
(only the pre-existing `block v0.1.6` warning) — the drives compile. Diff = exactly indent.rs + headless_drive.rs.

- **crates/editor/src/indent.rs** — ADDED a paragraph to the `rebase_selections` doc (after the #297 C11
  rationale): two carets inside ONE row's removed indent collapse to one cursor and that IS correct (the clamp +
  `from_selections`/`overlaps` merge; the columns are deleted → nowhere distinct); NOT the #297 silent-loss class
  because a dedent is an EDIT → ⌘Z restores. Uses `[\`rebase_through\`]` (pub → resolves) + plain backticks for
  `from_selections`/`Selection::overlaps`. NO code change.
- **crates/marley_app/src/headless_drive.rs** — ADDED two `#[gpui::test]` drives after
  `shift_tab_dedents_every_cursors_line_headless`:
  - `shift_tab_merges_two_carets_in_one_indent_and_undo_restores_headless` — `[4,6]` → ⇧Tab → text `"aa\nbb\ncc\n"`
    + `editor_selections() == [(3,3)]` (the merge), then ⌘Z → text restored + `[(4,4),(6,6)]` (the 2-cursor set
    back — the #297-scar recoverability).
  - `shift_tab_keeps_all_cursors_on_separate_rows_headless` — `[4,11,18]` → ⇧Tab → text `"aa\nbb\ncc\n"` +
    `editor_selections() == [(0,0),(3,3),(6,6)]` (all 3 survive — the guard).
  Synchronous (no `run_until_parked`), modeled exactly on the existing dedent drive.

**Deviations from design:** none. The exact-offset assertions (`[(3,3)]`, `[(4,4),(6,6)]`, `[(0,0),(3,3),(6,6)]`)
are the predicted post-dedent/post-undo positions; Phase 4 RUNS the drives and confirms/adjusts if any exact offset
differs (the `len` — 1 / 2 / 3 — is the load-bearing assertion). Per the ticket's no-behavior-change nature the
drives ARE the deliverable, so they were written in Phase 3 (compile-checked) rather than deferred whole to Phase 4;
Phase 4 owns the RUN + the gate.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**1 critic** (general-purpose) — scaled to the tiny no-behavior-change surface (a doc note + 2 test drives). It
verified the tree untouched + `cargo check` clean. **Verdict: both load-bearing claims HOLD** — the ⌘Z-restore is
TRUE (fully traced) and all offsets are arithmetically correct.

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| F1 | MEDIUM | **T2's carets `[4,11,18]` are on the FIRST LETTER (col 4), not "one char into the indent" as the comment claimed** — and they exercise the DELTA path, not the CLAMP path the doc note describes for separate rows. (Buffer `"    aa\n…"`: row 0's indent is `[0,4)`, so offset 4 = `a`.) The assertion `[(0,0),(3,3),(6,6)]` is still correct (4→0 via delta, 11→3, 18→6) and the test passes, but the fixture mislabels + doesn't test the claimed mechanism. | **REAL — FIXED at source** (the critic's better option). Moved the carets to `[1, 8, 15]` (col 1, genuinely INSIDE each row's indent) → the IDENTICAL assertion `[(0,0),(3,3),(6,6)]` but now via the CLAMP arm (1→clamp `at`=0, 8→clamp `7-4`=3, 15→clamp `14-8`=6) — a faithful #358 regression guard (in-indent carets on separate rows each clamp + all survive). Comment corrected. |
| F2 | LOW | the doc note's separate-rows parenthetical "each clamps to its own line start / are unaffected" is loose — "unaffected" is imprecise (offsets shift) and "clamps" is true only for in-indent carets. | **FIXED — tightened** to "each land at their OWN row's new start — distinct offsets — so all survive" (accurate for both in-indent and past-indent carets; the #337-F5 doc-accuracy rule). |
| — | (verified) | (a) the doc note's 4 claims all hold: `rebase_through:158` clamps in-span → `at`; `from_selections`/`overlaps` `<=` merges same-offset carets; the ⌘Z-restore is TRUE — the ⇧Tab dedent arm brackets `begin_undo_group(before=[caret(4),caret(6)])`, and `Buffer::undo` returns+applies `sel_before` → both carets restored (so T1's `[(4,4),(6,6)]` is exact). | **CLEARED** (critic-traced through undo.rs/buffer.rs). |
| — | (verified) | (c) Opt 3 is a genuine no-op for every caret (re-derived, no hole); (d) indent.rs is doc-comment-only, the drives are `#[gpui::test]`, `[rebase_through]` links a pub fn, `from_selections`/`overlaps` are plain backticks (overlaps is private), no Zed/Warp. | **CLEARED.** |

**Result: 0 CRITICAL/HIGH. 1 MEDIUM (T2 fixture fidelity) + 1 LOW (doc looseness) FIXED at source; both load-bearing
claims (the merge + the ⌘Z-restore) confirmed correct by trace.** The MEDIUM fix makes T2 genuinely exercise the
in-indent clamp the ticket is about (not the delta path), with the same assertion — a strictly better regression
guard. `cargo check -p marley -p marley_editor` clean after the fixes. Lenses: doc-accuracy (#337-F5),
test-rigor/#297-scar, the Opt-3 no-op, provenance.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests RUN (the 2 drives were written in Phase 3; Phase 4 confirms the offsets + the gate):**

| # | Test | Result |
|---|------|--------|
| T1 | `shift_tab_merges_two_carets_in_one_indent_and_undo_restores_headless` | **PASS** — `[4,6]` → ⇧Tab → text `"aa\nbb\ncc\n"` + `editor_selections() == [(3,3)]` (the merge to 1 cursor); ⌘Z → text restored + `[(4,4),(6,6)]` (the 2-cursor set back). ★ The `[(4,4),(6,6)]` ⌘Z-restore was TRACED (not run) at inspect — the run CONFIRMS it exactly. |
| T2 | `shift_tab_keeps_all_cursors_on_separate_rows_headless` | **PASS** — `[1,8,15]` (in-indent, the F1 fix) → ⇧Tab → text `"aa\nbb\ncc\n"` + `editor_selections() == [(0,0),(3,3),(6,6)]` (all 3 survive; each clamps to its own row start). |

**Runs (`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley -E 'test(shift_tab_merges_two_carets) + test(shift_tab_keeps_all_cursors)'` →
  **2 passed** — every exact-offset assertion confirmed as traced (no adjustment needed).
- `cargo nextest run -p marley_editor` → **241 passed, 0 skipped** (the doc-only indent.rs change did NOT break the
  `rebase_through`/`rebase_selections`/`overlaps` tests — behavior unchanged).
- `cargo nextest run -p marley` → **729 passed, 2 skipped** (T1/T2 + no regression; was 727, +2).

**NO LIVE SYNTHETIC DRIVE — stated.** chad may be at the machine; the two headless drives are `cargo nextest`
through the REAL ⇧Tab dedent arm (the app-side keystroke handler), so they ARE the exact proof of the merge +
recoverability. A live keyboard/screencapture drive is off-limits AND unnecessary here.

**FULL `--diff` gate → `GATE GREEN [diff]` — 15/15 on the first run.** Receipt
`0a638a52ce6627f37fa9bd276ba304b35c070a2e`. gate:4 coverage 100% (no new uncovered lines — the change is a doc
comment + tests), gate:5 MSI 100 (no new viable mutants — the indent.rs diff is doc-comment-only, headless_drive.rs
is test-only, the app ⇧Tab arm is `mutants::skip`), gate:14 docs PASS (the `[rebase_through]` intra-doc link
resolves; plain backticks elsewhere). No pre-existing failures; the `search_open…` flake did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):**
- CHANGELOG.md — `### Changed` entry for #358 (the documented-correct merge + ⌘Z-restore + the guards; Opt 3 was a
  no-op → no behavior change).
- `docs/marley_architecture/editor.md` — the `SelectionSet`/`overlaps` merge-law bullet gained a `M22 #358`
  sub-bullet (the in-indent merge is correct via `rebase_through`'s clamp + `from_selections`; NOT the #297 motion
  scar because a dedent is an EDIT → ⌘Z restores; separate rows survive). `rebase_selections`' code doc was written
  in Phase 3.

**Capture (forge wired):**
- **aar-submit** d08e90f8 — outcome completed, effectiveness 4, 1 novel finding, 13 verdicts. Lessons: (a) THE FORK
  DISSOLVED ON THE ARITHMETIC — Opt 3 (`col - stripped`) is a pure no-op on the real `rebase_through` (an inside
  caret has col<remove by definition), and the merge is topologically forced → Opt 1 (accept+document) is the only
  answer (the #339 dissolve precedent — trace the proposed fix against the real code). (b) the inspect F1 catch: a
  test can assert the right OUTCOME while exercising the WRONG MECHANISM — T2's `[4,11,18]` carets were on the first
  letter (delta path), not in-indent (clamp path); moved to `[1,8,15]` for the same assertion via the clamp. (c) the
  ⌘Z-restore recoverability is what makes this correct-not-a-defect (an edit restores; the #297 motion scar did not).
- **prevention-rule-record:** `PR-claude-regression-guard-must-exercise-the-claimed-path-001` (7cf23efe) — a
  regression guard for a specific code path must EXERCISE that path (trace the fixture through the real branch), not
  just reproduce its output.
- **failure-record:** none — no shipped bug (no behavior change); the F1 T2-caret was a test-fidelity fix in-phase.
- No follow-up ticket — resolved as accept+document; Opt 3 proven inert, nothing deferred.

**Close + archive:** forge #358 closed; TICKET-358 → closed/; pair archived active/ → completed/; spec Phase 5 PASS.

⚠️ **/commit — the receipt HOLDS.** Phase-5 edits are docs-only (CHANGELOG + editor.md); the source (indent.rs
doc-comment + headless_drive.rs drives) is UNCHANGED since the Phase-4 receipt `0a638a52…` → a fingerprint verify
is valid (no `.rs` staged past the receipt).

**Status: Phase 5 — Complete PASS.**
