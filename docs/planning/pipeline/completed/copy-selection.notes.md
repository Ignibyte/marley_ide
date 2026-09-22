# copy the selection (cmd-C) — Notes

- **Forge ticket:** #44 `e352a858-a281-4757-91a7-10a7f630019d`
- **AAR:** `5203985e-387c-4339-970b-ecbdc34b15ff`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-044-copy-selection.md
- **Pipeline spec:** copy-selection.spec.md

## Phase 1 — Plan
- **Request:** forge #44 (M1.G "Block Workflows & Selection" seq-2, auto-approved) — cmd-C copy,
  completing copy/paste (#42 paste).
- **Classification / tier:** work pipeline, `feature`, PURE (3 fns extending #43's text_selection.rs
  — cov/MSI 100) + a SHIM cmd-C handler. marley_app only.
- **Discovery (§18):**
  - gpui `cx.write_to_clipboard(ClipboardItem::new_string(text))` (app.rs:1041 / platform.rs:1519) —
    the write API, mirrors #42's `read_from_clipboard`.
  - #43 shipped `Selection` + `row_selection` (text_selection.rs) — `selected_text` reuses it so copy
    == highlight. `block.output_styled()` gives the per-line StyledLines for the rows-build.
  - The cmd-V handler (app.rs:612, `modifiers.platform && key=="v"`) — the cmd-C handler mirrors it
    (`key=="c"`), placed beside it.
  - Dep #43 (done).
- **Decisions:** D1–D3 in the spec (selected_text reuses row_selection; copy_payload empty-guard;
  pure fns + shim write).
- **Open questions for Design:** `row_slice` return `String` (owned, simple) vs `&str` (borrow) —
  lean String (selected_text collects anyway); whether copy_payload lives in text_selection.rs (yes,
  with the selection model); the rows-build in the shim (block command + output_styled lines — match
  the render's row order so indices align).
- **AAR id:** `5203985e-387c-4339-970b-ecbdc34b15ff`.

## Phase 2 — Design

### PURE — extend `crates/marley_app/src/text_selection.rs`
```rust
/// Chars `[from, to)` of `s`, clamped to its char length (`to <= from` → empty). Char-safe (via
/// `chars()`), so multi-byte output slices correctly. Private — the row primitive of `selected_text`.
fn row_slice(s: &str, from: usize, to: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let from = from.min(chars.len());
    let to = to.min(chars.len());
    if to <= from { return String::new(); }
    chars[from..to].iter().collect()
}

/// The selected text over `rows` (R47/R48) — each row's `row_selection` span (the SAME geometry the
/// #43 highlight uses, so copy == highlight) sliced by `row_slice`, joined with `\n`.
pub fn selected_text(rows: &[String], sel: Selection) -> String {
    let (start, end) = sel.normalized();
    let mut out = String::new();
    for row in start.row..=end.row {
        if row >= rows.len() {
            break;
        }
        let row_len = rows[row].chars().count();
        if let Some((from, to)) = row_selection(sel, row, row_len) {
            if row > start.row {
                out.push('\n');
            }
            out.push_str(&row_slice(&rows[row], from, to));
        }
    }
    out
}

/// The clipboard payload for a cmd-C (R48): `None` when there is no selection OR the selected text is
/// empty (so cmd-C with nothing selected is a no-op, never an empty clipboard write); else `Some`.
pub fn copy_payload(rows: &[String], selection: Option<Selection>) -> Option<String> {
    let text = selected_text(rows, selection?);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
```

### SHIM — `crates/marley_app/src/app.rs` (~612, beside the cmd-V handler)
```rust
if event.keystroke.modifiers.platform && event.keystroke.key == "c" {
    if let Some(state) = view.workspace.state(view.workspace.focused()) {
        let rows = content_row_texts(state);
        if let Some(text) = copy_payload(&rows, state.selection) {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }
    return;   // cmd-C is a platform chord — it never reaches the PTY (Ctrl-C is control+c)
}
```
+ a `content_row_texts(state) -> Vec<String>` shim helper: for each block, push `block.command` then
each `output_styled()` line's joined text — matching the render's row order (so the selection indices
align). (The prompt row is omitted — the first cut copies Block content; a selection spilling onto
the prompt row is skipped by `selected_text`'s `row >= rows.len()` break — safe.)

### File manifest
- M `crates/marley_app/src/text_selection.rs` — `row_slice` (private) + `selected_text` +
  `copy_payload` (pub) + tests.
- M `crates/marley_app/src/app.rs` — the cmd-C handler + `content_row_texts`.
- M `docs/specs/SPEC-app-shell.spec.md` — the copy clause (R48) + Mutation-Targets. CHANGELOG; arch.

### Mutation Targets
- `row_slice` — the `from.min`/`to.min` clamps, the `to <= from` empty guard, the `chars[from..to]`
  slice. (Killed by the slice/clamp/empty/multibyte fixtures — the `.min` clamps are gate-invisible
  like #43's, so the clamp fixtures are written deliberately.)
- `selected_text` — the `start.row..=end.row` range, the `row >= rows.len()` break, the `row >
  start.row` `\n` guard. (Killed by single vs multi-row + the exact joined output.)
- `copy_payload` — the `selection?` (None → None) + the `text.is_empty()` guard (empty → None). (Killed
  by the None / empty-selection / non-empty fixtures.)

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `row_slice_char_safe` — `"hello"[1..3]`→`"el"`; `"abc"[1..99]`→`"bc"` (to clamp); `"abc"[3..3]`→`""`; `"abc"[3..1]`→`""` (to<from); `"café"[1..4]`→`"afé"` (multibyte) | unit |
| REQ-002 | `selected_text_joins_rows` — `["abc","def","ghi"]` `(0,1)..(2,2)`→`"bc\ndef\ngh"`; single `["hello"]` `(0,0)..(0,3)`→`"hel"` | unit |
| REQ-003 | `copy_payload_empty_guard` — `(rows, None)`→`None`; `(rows, Some(empty sel (0,2)..(0,2)))`→`None`; `(rows, Some((0,0)..(0,3)))`→`Some("hel")` | unit |
| REQ-004 | the cmd-C round-trip | shim + masked visual (select→cmd-C→paste) — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs cmd-C keystroke + clipboard write + `content_row_texts` (shim exclude; needs a
live window + clipboard).

### Risks / decisions
- D-2.1 `selected_text` REUSES `row_selection` (#43) — one geometry, so the copied text can't drift
  from the highlight. D-2.2 `copy_payload`'s empty-guard prevents clobbering the clipboard with `""` on
  an empty/absent selection. D-2.3 `row_slice` private (via selected_text), but the tests call it
  in-crate for precise clamp/boundary coverage; it's on the live shim→copy_payload→selected_text→
  row_slice chain (not dead). D-2.4 cmd-C (platform) vs Ctrl-C (control) — no conflict; the platform
  chord is caught before `key_input_from_keystroke` (which returns None for platform chords).

## Phase 3 — Implement
- **Built (per manifest):** `text_selection.rs` — `row_slice` (private, char-safe clamp),
  `selected_text` (reuses `row_selection` so copy == highlight, `\n`-joined), `copy_payload` (`?` on
  the selection + `is_empty` guard → None). `app.rs` — `content_row_texts(state)` (block command +
  `output_styled` lines, render row order), the cmd-C handler (~612, before cmd-V; `key=="c"` +
  platform → `copy_payload` → `cx.write_to_clipboard(ClipboardItem::new_string(text))`, `return`).
  Imports: `copy_payload` + gpui `ClipboardItem`. SPEC-app-shell R48 + row 48 + Mutation-Targets;
  CHANGELOG.
- **Deviations:** none. (Prompt-row copy still deferred as designed — block content only.)
- **Verification at this phase:** `cargo check --workspace` 0 err; fmt; clippy `-D warnings` 0; docs
  gate 0. The chain shim→copy_payload→selected_text→row_slice is live (row_slice not dead). Unit tests
  are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a 34-assertion verbatim probe + a real scoped cargo-mutants + the render cross-read).
  Verdict: pure logic CORRECT; one real consistency bug + two Phase-4 test carry-forwards.
- **Findings table:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | **MED** | copy included command-HEADER rows (`content_row_texts` pushes `block.command`), but #43's highlight painted only OUTPUT rows — so dragging across a header copied text that showed NO highlight, violating the spec's "copy == highlight" (D1/REQ-002). | REAL | **FIXED** — the command-header render now tints via `row_selection` (mirror of the output branch), so highlight == copy. Warp copies commands too, so copy-includes-command is right; the highlight just had to match. app.rs header render. |
  | F2 | MED→LOW | `selected_text`'s `row >= rows.len()` break is correct (spill truncates, no panic) but UNTESTED — the planned REQ-002 fixtures all have `end.row < len`, so a `>=`→`>` mutant could survive. | REAL (Phase-4) | P4 adds a SPILL fixture: `selected_text(["abc","def"], (0,1)..(5,2)) == "bc\ndef"` + `copy_payload(rows, Some((9,0)..(9,3))) == None`. |
  | F3 | LOW | `row_slice`'s `to.min` clamp is load-bearing but GATE-INVISIBLE (no `.min` mutant + `selected_text` never passes out-of-range indices since `row_selection` pre-clamps) — reachable only via a DIRECT row_slice test. `from.min` is provably redundant (the `to<=from` guard covers `from>len`) → an equivalent mutant. | REAL (Phase-4, already planned) | P4 keeps the direct REQ-001 `row_slice` clamp test (`"abc"[1..99]→"bc"`) — necessary, not optional. |
- **Verified CORRECT (probe):** the `\n` placement (`row > start.row`, 2 `\n`, none leading/trailing —
  kills the `>=`/`<`/trailing mutants); char boundaries (`café`→`afé`, byte-slice would panic);
  `copy_payload` empty-guard (None/empty→None, non-empty→Some — both `is_empty` branches); the shim
  row-order ALIGNS with the render walk (command + output_styled per block, prompt omitted, break-safe);
  cmd-C (`platform`) `return`s before the PTY path — no Ctrl-C (`control`) SIGINT collision.
- **Mutants:** 28 (12 in the 3 new fns — the planned REQ-001/002/003 fixtures kill all 12 EXCEPT the
  F2 break, which the spill fixture now covers; 15 #43 caught; 1 unviable `Default`). A `-j2`
  cross-attribution artifact noted (a spurious "caught" on `row_slice→String::new()` — treated as
  unkilled; REQ-001 kills it legitimately).

## Phase 4 — Validate
- **Tests added (text_selection.rs):** `row_slice_char_safe` (slice/clamp/empty/to<from/from>len/
  multibyte); `selected_text_joins_rows` (multi-row; backward==forward; single-row WITH a following
  unselected row [covers the `row_selection` None branch]; multibyte; SPILL past content [F2]);
  `copy_payload_empty_guard` (None / empty-span→None / Some / past-content→None).
- **Coverage fix (found by the gate):** the first gate run FAILED cov (text_selection 99.62%) — the
  original `selected_text` looped `start.row..=end.row`, so `row_selection` there was ALWAYS `Some`
  (a structurally-dead `None` region). REWROTE `selected_text` to walk all rows via
  `iter().enumerate()` and let `row_selection` FILTER (`None` now reachable for out-of-selection rows)
  — also drops the manual `row >= rows.len()` break (spill stops at `rows.len()` naturally) and is
  simpler. The single-row test gained a trailing unselected row to exercise the `None`.
- **Runs (actual):** `cargo nextest run -p marley` → 107 passed; `--workspace` → 522 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15** (2nd run), coverage 100%, mutation
  **9 caught / 0 missed → MSI 100.0%**. Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` — the copy bullet.
  SPEC-app-shell R48 at implement.
- **Knowledge captured:** failure `BF-claude-copy-superset-of-highlight-header-rows-untinted`
  (2aa3f9d4, validation) + prevention rule
  `PR-claude-paired-copy-and-highlight-must-cover-same-row-set-001` (778d9e61, medium) — paired
  copy/highlight surfaces must enumerate the SAME row set; the asymmetry is invisible to per-fn tests +
  MSI, only breaking at the shim seam. aar-submit `completed` (score 5). Wins: (1) the inspect critic
  caught the copy⊋highlight drift by cross-reading the render against content_row_texts — a
  cross-surface bug no unit test would find; (2) the GATE caught a structurally-dead `None` branch in
  the first `selected_text`, which the rewrite (iterate-all + row_selection filters) turned into a
  simpler, spill-natural, coverage-clean form.
- **Ticket:** forge #44 → done; local doc → closed/; pipeline pair archived. 2 of 6 in M1.G — copy/
  paste COMPLETE.
