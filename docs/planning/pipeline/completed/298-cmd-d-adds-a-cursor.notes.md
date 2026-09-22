# 298 — ⌘D adds a cursor + ⌘⇧L select-all-occurrences · notes

- **Spec:** ./298-cmd-d-adds-a-cursor.spec.md
- **Forge ticket:** #298 `fb7ba9ea-3757-4b7e-a928-57bc5b21a07e` · **AAR:** `3adb9d1d-4e70-4ca3-a0ca-c7d281f19e1e`
- **Deps:** #296 (multi-cursor core, `135b439`) + #297 (gestures + the app shim, `edb62d4`) — both SHIPPED.

## Phase 1 — Plan

### What this is
The payoff ticket. #296 built the N-cursor set, #297 made it reachable — and this is the gesture people
actually reach for: select a word, hit ⌘D a few times, type once, and every occurrence changes. #272 already
built the search half; what's missing is that ⌘D *replaces* instead of *adds*.

### The foundation — verified against the shipped code, not assumed
This ticket was **impossible one commit ago**. `find_all("aa")` over `"aaaa"` yields the TOUCHING matches
`(0,2)` and `(2,4)`, and under #296's `<=`-for-everything merge they collapsed into ONE cursor. I ran it:

```
find_all("aa") over "aaaa" → [(0, 2), (2, 4)]
through from_selections    → 2 cursor(s)   ✓
```

#297's inspect made the merge caret/range-asymmetric, so touching RANGES now stay two cursors. That is exactly
the fidelity ⌘D needs. **REQ-001 pins it** — not because I doubt it, but because it was a live bug one commit
ago and every other requirement rests on it.

Worth remembering: #296 *flagged* this for #298 and documented a refinement — and that refinement was
**catastrophic** (`sel.is_caret()` short-circuits the guard, so any two carets anywhere merge into one giant
range). A future implementer following that comment would have shipped total multi-cursor failure. The comment
was confident, plausible, and wrong. It is now correct.

### Grounding (checked in the code, not recalled)
- **⌘⇧L is genuinely TAKEN** — `keymap.rs:121` binds `chord(cmd, ¬ctrl, ¬alt, shift, "l")` → `"split-right"`
  (M12.2 #197), and `keymap.rs:590` asserts it resolves on a terminal. **D1** resolves it by Editor-scoping,
  the shipped shadow precedent.
- **`select_next_match` has exactly ONE production caller** — `app.rs:4700`, the ⌘D arm. The find bar uses
  `find_all` (`app.rs:5501`) and `replace_all`, not this. So rebinding ⌘D really does make it dead.
- **The roster guard counts ROWS, not unique chords.** `all_chords()` listed 47 → 49 on #297 for two chords
  that *already existed globally* (⌘⌥↑/↓), which proves it does not dedup. So #298: **+1 chord (50), +1 scoped
  row (9)**. The ⌘D rebind changes the ACTION on an existing row — no count change. Two assertions to update
  (`keymap.rs:827` and `:849`).

### The open decision I found at PLAN time (and would otherwise have shipped)
**⌘D and ⌘⇧L currently disagree about what an "occurrence" is.**
- `next_occurrence` (⌘D's engine): `rope.char(i + k) == ndl[k]` — **case-SENSITIVE**.
- `find_all` (⌘⇧L's engine): `c == ndl[k] || c.eq_ignore_ascii_case(&ndl[k])` — **case-INSENSITIVE**.

Over `"foo FOO"`, ⌘D would select one occurrence and ⌘⇧L two — **two gestures shipped in the same ticket,
disagreeing on the same text.** And it isn't cosmetic: these gestures exist to be *typed over*, so if ⌘D
silently selects `FOO` when you asked for `foo`, the next keystroke rewrites text you never targeted. Same
class of harm as #297's undo-restores-the-wrong-ranges bug.

`find_all`'s folding is *correct for its existing caller* (the find bar inherited the terminal find's
case-folding precedent), so this is not "fix `find_all`" — it is "these two new callers want a different
matcher". Phase 2 picks the mechanism; **that they must agree, and agree on case-SENSITIVE, is settled.**
REQ-007 pins it.

### Classification
Work pipeline, `feature`, M19. **One shippable slice** — two small pure seams over search machinery that
already exists, plus two keymap rows. The heavy lifting (the N-cursor set, the N-band highlight, the
N-cursor edit in one undo unit) all shipped in #296/#297; this ticket *verifies* those rather than rebuilding
them.

### Risks
- **The wrap/no-op boundary.** `next_occurrence` WRAPS, so ⌘D past the last match re-finds an earlier one.
  Without an explicit exhausted-guard the set would merge it away and the keypress would do nothing *by
  accident*. "It happens to work" is exactly how #296's merge bug survived a full green gate — make it a
  decision (D3) and a test.
- **The LAST-cursor search (D2).** Searching from the primary instead of the bottom-most cursor would re-add
  an already-selected match and stall. Off-by-one at a match boundary is the obvious failure.
- **A green gate is not evidence.** On #297, five critics found TEN real bugs (six HIGH) with clippy clean,
  1066 tests passing, 100% coverage, MSI 100, and a 50,000-step differential fuzzer all green — and a *test*
  was actively defending one of them. Inspect spawns the critics and **waits**.

---

## Phase 2 — Design

### D6 — the case-sensitivity mechanism: `find_all(buffer, needle, fold: bool)`
**RESOLVED.** Every caller was read, not recalled:
- `find_all` has **exactly ONE production caller** — the find bar (`app.rs:5501`). (The `find_all` in
  `marley_visual_harness/ax.rs` is an unrelated AX-tree search.)
- `next_occurrence` has **exactly ONE caller** — `select_next_match`, which this ticket deletes. It survives
  only because the new seam uses it.

So the fix is not "change `find_all`'s behavior" — its folding is *correct for the find bar*, which inherited
the terminal find's case-folding precedent and is documented as such. The fix is to make the choice
**EXPLICIT at every call site**, because the whole bug was a *silent* disagreement:

```rust
pub fn find_all(buffer: &Buffer, needle: &str, fold: bool) -> Vec<(CharOffset, CharOffset)>
```
The find bar passes `true` (its shipped behavior, now visible in the code rather than buried in a matcher).
`select_all_occurrences` passes `false`. `next_occurrence` is already exact, so ⌘D needs no change — and the
two gestures now agree by construction.

Rejected: a separate `find_all_exact` (duplicates the scan — and two scans is how they drifted apart in the
first place); flipping `find_all`'s default (would silently change the find bar's shipped behavior).

**Mutation note:** the fold makes the hit test `c == ndl[k] || (fold && c.eq_ignore_ascii_case(&ndl[k]))`,
whose `||`→`&&` and `&&`→`||` mutants are both killed by the case-sensitivity fixtures (REQ-007) — the
`||`→`&&` mutant matches nothing when `fold == false`, and the `&&`→`||` mutant folds case even when it must
not.

### D7 — `select_next_match` is DELETED
Confirmed dead once ⌘D rebinds: its only production caller is the ⌘D arm. Its two halves — `word_range_at`
(the caret's word, sharing `movement::is_word_char` so ⌘D and ⌥-arrow never disagree about what a word is) and
`next_occurrence` (the wrapping scan) — survive and are exactly what the new seam is built from. The shapes
genuinely differ, which is why it cannot simply be called: it searches from THE selection and REPLACES; we
search from the LAST cursor and ADD. Deleting it also removes its `lib.rs` re-export and its tests (§0 — a
`pub` fn nothing calls is an `#[allow(dead_code)]` in spirit).

### D8 — the seams live in `multi_cursor.rs`
It is the gesture home (`add_cursor_vertical`, `toggle_cursor_at`, `collapse_to_primary`), and ⌘D-adds-cursor
is a multi-cursor gesture. It imports `find` — no cycle (`multi_cursor` → `find` → `buffer`).

### The seams

```rust
pub fn add_next_occurrence(set: &SelectionSet, buffer: &Buffer) -> SelectionSet
```
* **The primary is a bare CARET ⇒ the FIRST press.** Turn every CARET member into the word under it
  (`word_range_at`); RANGE members are left alone; a caret with no word under it stays a caret. This is total
  for every reachable set shape — a ⌘-clicked caret sitting beside a range is a state #297 made possible — and
  for the overwhelmingly common one-caret case it is exactly #272's feel, preserved.
* **Else ⇒ ADVANCE.** The needle is the PRIMARY's text; the search starts at **`set.last().end()`** — the
  BOTTOM-most cursor, not the primary (**D2**). Searching from the primary would re-find a match that is
  already selected and the gesture would stall on the second press.
* **The EXHAUSTED guard is EXPLICIT (D3).** `next_occurrence` WRAPS, so once every occurrence is selected it
  re-finds one we already have. Without a guard, `from_selections` would merge the duplicate away and the
  keypress would do nothing **by accident**. Detect it (the found range equals an existing member) and return
  the set unchanged **by decision**. "It happens to work" is precisely how #296's merge bug survived a fully
  green gate — this gets its own test.

```rust
pub fn select_all_occurrences(set: &SelectionSet, buffer: &Buffer) -> SelectionSet
```
* A bare caret first resolves to the word under it (so ⌘⇧L works straight from a caret, like ⌘D); otherwise
  the primary's text. `find_all(buffer, &needle, false)` → `from_selections`. No matches → the set unchanged.

### Architecture / §20
Pure logic in `crates/editor` (cov/MSI 100); gpui confined to `app.rs`'s `mutants::skip` dispatch arms.
**§20 confirmed:** the reference is Zed/VS Code *behavior* — ⌘D = add-selection-to-next-find-match (first press
selects the word, each further press adds the next occurrence, wrapping, and once all are selected it does
nothing); ⌘⇧L = select-all-occurrences. Both are reimplemented here from observed behavior on Marley's own
`SelectionSet` + the shipped `find.rs`. The ⌘⇧L chord collision is resolved with Marley's *own* shipped
context-shadowing mechanism. No Zed or VS Code source read or translated.

### VERIFY, DON'T REBUILD
Two acceptance criteria are satisfied by code that already shipped, and this ticket must **not touch them**:
- **The N-selection HIGHLIGHT already renders** — #297's N bands (`code_view::row_selection_cols_all` +
  `styled_slices_with_marks`). ⌘D producing N selections lights them up for free.
- **Typing over N selections in ONE undo unit already works** — #296/#297's `edit_at_selections`, reached
  through the platform text path.
REQ-006 *verifies* both, end to end. If it fails, the bug is in #296/#297, not here.

### File manifest
| File | Change |
|---|---|
| `crates/editor/src/find.rs` | `find_all` gains `fold: bool` (explicit at every call site); **`select_next_match` DELETED** + its tests. |
| `crates/editor/src/multi_cursor.rs` | NEW `add_next_occurrence` + `select_all_occurrences`. |
| `crates/editor/src/lib.rs` | drop the `select_next_match` re-export; add the two new seams. |
| `crates/marley_app/src/keymap.rs` | the Editor ⌘D row's ACTION → `add-next-occurrence` (same chord, no count change); NEW Editor-scoped ⌘⇧L → `select-all-occurrences`, shadowing the global `split-right`. **Roster guard: `chords.len()` 49→50, scoped 8→9** (it counts ROWS, not unique chords — proven on #297). `keymap.rs:590` (⌘⇧L → `split-right` on a TERMINAL) must still pass: that is the proof the shadow is correctly scoped. |
| `crates/marley_app/src/app.rs` | the `"select-next-match"` arm → `"add-next-occurrence"`; NEW `"select-all-occurrences"` arm. Both mirror #297's add-cursor arms (read `active_selections()` → seam → `set_active_selections` → `follow_editor_caret`). |
| `crates/marley_app/src/headless_drive.rs` | REQ-006 end-to-end through the real key path. **Ends with `reap_sessions`** — a headless test that panics before it HANGS on the PTY drop chain. |

### Regression Test Plan

| REQ | Test | Where | Proves |
|---|---|---|---|
| **REQ-001** | **`touching_occurrences_stay_two_cursors`** — **written FIRST**: `find_all("aa")` over `"aaaa"` → `(0,2)`+`(2,4)` → through `from_selections` → **2 members**. | `multi_cursor.rs` | **The foundation.** It was a live bug one commit ago; every other requirement rests on it. |
| REQ-002 | `first_press_from_a_bare_caret_selects_the_word` — one caret → one word selection (#272's feel); N carets → N words; a caret with no word under it stays a caret; a RANGE member is left alone. | `multi_cursor.rs` | The first press. |
| REQ-003 | `a_further_press_adds_the_next_occurrence` — 2nd press → 2 selections, the original KEPT; the search starts from the LAST cursor (a fixture where searching from the primary would return an already-selected match); a press that must WRAP past EOF. | `multi_cursor.rs` | The additive advance + D2. |
| REQ-004 | `exhausted_is_a_no_op` — press N+2 times over N occurrences → still exactly N selections, unchanged. | `multi_cursor.rs` | D3 — by decision, not by accident. |
| REQ-005 | `select_all_occurrences_takes_every_match` (incl. straight from a bare caret) + the keymap RESOLUTION test: ⌘⇧L → `select-all-occurrences` on `KeyContext::Editor`, and still `split-right` on Terminal/global. | `multi_cursor.rs` + `keymap.rs` | ⌘⇧L + the shadow. |
| REQ-006 | `t298_cmd_d_then_type_replaces_every_occurrence_headless` — the REAL key path: ⌘D ⌘D → 2 selections → type → BOTH replaced → ONE ⌘Z reverts both. | `headless_drive.rs` | **Verifies** #296/#297; does not rebuild them. |
| REQ-007 | `cmd_d_and_cmd_shift_l_agree_and_are_case_sensitive` — over `"foo FOO foo"`, BOTH seams match exactly the two lowercase `foo`s. Plus a find-bar test that `find_all(.., true)` still folds. | `multi_cursor.rs` + `find.rs` | The divergence, closed. Also kills the fold's `\|\|`/`&&` mutants. |
| REQ-008 | `scripts/gates.sh --diff` | gate | cov 100 + MSI 100. The real `cargo mutants --list` set is traced, never guessed — and `Selection`/`SelectionSet` derive no `Default`, so `-> Default::default()` body mutants are UNVIABLE and both seams need hand-written behavior assertions. |
| — | **THE LIVE DRIVE** (Validate): ⌘D ⌘D → two highlighted occurrences → type → both replaced in one undo unit → ⌘⇧L → every occurrence. Capture and READ the pixels. **Never put drive.swift's `focus` verb between the gesture and the assertion — it CLICKS, and a plain click collapses the cursor set by design (#297 D4).** | `scripts/selftest` | The only proof the gesture works. |

No new uncoverable paths.

### Risks
- **The wrap/exhausted boundary is the subtle one.** An off-by-one at a match edge either stalls the gesture
  (never adds) or loops it (adds forever). D3's guard is explicit *and* tested for exactly that reason.
- **`find_all`'s new param touches the find bar.** One production call site; a test pins that folding still
  works there, so the shipped behavior cannot regress silently.
- **A green gate is not evidence.** On #297, five critics found TEN real bugs (six HIGH) with clippy clean,
  1066 tests, 100% coverage, MSI 100 and a 50k-step fuzzer all green — and a *test* was defending one of them.
  Inspect spawns the critics and **waits**.

## Phase 3 — Implement

Built to the Phase 2 manifest, sequenced so the tree compiles at each step. Five files, +205/−119.
`cargo nextest run --workspace` → **1089 passed**, clippy `-D warnings` clean.

### `crates/editor/src/find.rs` — make the matcher's case choice VISIBLE
- **`find_all(buffer, needle, fold: bool)`** — the hit test is now
  `c == ndl[k] || (fold && c.eq_ignore_ascii_case(&ndl[k]))`. The folding did not change; *who asks for it*
  did. This is the mechanism Phase 2 chose for REQ-007: the two callers now state their choice at the call
  site, so the silent disagreement cannot be reintroduced by someone who never opens `find.rs`.
- **`select_next_match` DELETED** (+ its tests + its `lib.rs` re-export). Its only production caller was the
  ⌘D arm, which rebinds. §0 — a `pub fn` nothing calls is an `#[allow(dead_code)]` wearing a disguise. Its two
  halves survive and are exactly what the new seam is built from.
- The module doc named `select_next_match` — rewritten.
- ~15 existing test call sites updated to pass `fold = true` (they assert the *folding* behavior, which is the
  find bar's, and it still holds).

### `crates/editor/src/multi_cursor.rs` — the two seams (the gesture home, imports `find`; no cycle)
- **`add_next_occurrence(set, buffer)`**
  - *The first press* (`primary().is_caret()`): map **every** member — a CARET becomes the word under it, a
    caret with no word under it stays a caret, a RANGE member is left alone. Total for every reachable shape,
    including the ⌘-clicked-caret-beside-a-range that #297 made possible. One caret → one word = #272's feel,
    preserved.
  - *Every press after*: the needle is the **primary's** text; the search starts at **`set.last().end()`** —
    the bottom-most cursor, per **D2**. From the primary it would re-find a match already selected and stall.
  - **D3 — the exhausted guard is EXPLICIT.** `next_occurrence` wraps, so once every occurrence is held it
    hands back one we already have. `from_selections` would merge the duplicate away and the keypress would
    do nothing *by accident*. It now does nothing *by decision*, and the doc says why: "it happens to work"
    is precisely how #296's merge bug survived a green gate.
- **`select_all_occurrences(set, buffer)`** — a bare caret first resolves to the word under it (so ⌘⇧L works
  straight from a caret, like ⌘D), then `find_all(buffer, &needle, /* fold */ false)` → `from_selections`.
  **Case-SENSITIVE** (REQ-007). Empty result ⇒ no-op.

### `crates/marley_app` — the shim
- **keymap.rs**: the Editor-scoped ⌘D row's action `select-next-match` → **`add-next-occurrence`** (same
  chord, same shadow — only the action moved). NEW Editor-scoped **⌘⇧L → `select-all-occurrences`**,
  shadowing the global `split-right` (keymap.rs:121, M12.2 #197) on the editor only. Roster guard:
  `chords.len()` 49 → **50**, scoped rows 8 → **9** (`all_chords()` counts ROWS, not unique chords).
  `keymap.rs:590` — ⌘⇧L still resolves to `split-right` on a **terminal** — passes untouched. That test is
  the proof the shadow is scoped correctly, so it stays exactly as it was.
- **app.rs**: one dispatch arm for both actions, mirroring #297's add-cursor arm — `clear_marked()`, read
  `active_selections()` + `active_buffer()`, call the seam, `set_active_selections(set)`,
  `follow_editor_caret()` (a wrapping ⌘D can land the new cursor off-screen). The find bar's `find_all` call
  passes `true` with a comment naming the divergence, so the next reader sees that ⌘D and the find bar
  disagree **on purpose**.

### Deviations from design
None. One thing the design did not name: the pre-existing **`keycontext_resolves_cmd_d_by_surface`** test
(#265) pins ⌘D's editor action by string — it failed on the rebind, which is exactly its job. Updated to
`add-next-occurrence`; the terminal/global `new-terminal` assertions are untouched.

## Inspect (Phase 3.5)

**FIVE HIGH bugs, two MEDIUM, four LOW — every one of them behind a fully green gate** (1089 tests passing,
clippy `-D warnings` clean, rustdoc clean). Four independent critics, each on a distinct lens, each required to
REPRODUCE rather than speculate. They converged: three of them found the flagship bug independently.

This is the third ticket in a row where the gate said yes and the code was wrong (#296: three bugs; #297: ten,
six HIGH). The gate is a floor, not evidence.

### The two root causes

Every HIGH traces to one of two decisions that *read* fine and were false:

1. **The branch discriminator asked about MEMBER 0, not the SET.** `primary().is_caret()` was standing in for
   "is this the first press". It is not the same question: `primary()` is member 0 — the TOPMOST — and #297's
   ⌘-click can put a bare caret *above* a range.
2. **Two match engines that disagreed about what an occurrence IS.** `next_occurrence` probes every index, so it
   returns SELF-OVERLAPPING matches ("aa" at 0 *and* 1 in "aaa"); `find_all` skips past each hit, so it never
   does. REQ-007 demanded the two gestures agree — I unified their CASE and left their OVERLAP divergent, which
   is the half that corrupts data.

### The findings

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | **HIGH** | **⌘D died one occurrence short, permanently.** The scan resumed at `set.last().end()` and treated an already-held match as proof of exhaustion. `from_selections` SORTS, so once the wrap added a cursor at the TOP, the resume offset stayed pinned at the BOTTOM — every later press re-found that same held match and declared itself finished. Occurrences BETWEEN the wrapped match and the start point were unreachable *forever*. Caret in the third of four `foo`s → press 4+ is a no-op with `(4,7)` still unselected. **The user types, and one occurrence silently keeps the old text.** Found independently by critics 2, 3 AND 4. | **REAL** — reproduced 3× | Step OVER held matches instead of bailing at the first one. |
| F2 | **HIGH** | **⌘-click at EOF killed ⌘D for the session.** Same root, worse trigger: `last()` can be a bare CARET (⌘-click, ⌘⌥↓) with nothing to do with the needle — one at end-of-file dragged the scan origin past every occurrence in the document. | **REAL** — critic 1 | The resume offset now comes from the bottom-most **RANGE**; stray carets are passengers, not the steering wheel. |
| F3 | **HIGH** | **A wordless caret on top stalled ⌘D forever.** ⌘-click in leading indentation → that caret is member 0 → the set goes back down the word-select branch → `word_range_at` → `None` → the caret stays a caret → the set returns byte-identical. Every press. Dead until Esc. | **REAL** — critics 1, 2 | Discriminate on the WHOLE SET (all-carets ⇒ first press). |
| F4 | **HIGH** | **The first press swallowed a cursor the user had placed.** Same wrong discriminator, other direction: with a word under it, the ⌘-clicked caret expanded to the whole word, MERGED into the range beside it — 2 cursors → 1 — and the needle silently changed from `bar` to `foobar`. A motion, so ⌘Z cannot bring it back. | **REAL** — critic 2 | Subsumed by F3's fix: a set holding a range never takes the word-select branch. |
| F5 | **HIGH** | **A self-overlapping needle GREW the selection instead of adding a cursor.** The exhausted guard tested EQUALITY; the invariant merges on OVERLAP. A match overlapping a held member without equalling it sailed through, was pushed, and got UNIONED into that member: the user's `"aa"` became `"aaa"` — text they never selected, and the needle for every press after. Critic 1 proved it reachable **keyboard-only** (`⇧→⇧→` over an indent run) and showed the payload: typing over the set yields `"XbX"` where the user's two selections should give `"abXbX"` — it eats two characters they never targeted. *(I found this one myself before the critics reported; critic 1 independently confirmed and went further.)* | **REAL** — me + critic 1 | ONE engine (`find_all`), plus a candidate must survive the invariant **as itself** or it is not addable. |
| F6 | **HIGH** | **The viewport followed the wrong cursor, so ⌘D looked broken.** `follow_editor_caret()` → `active_caret()` → `primary().head()` → member 0 — which an ADDITIVE gesture *never moves*. ⌘D scrolled to a row already on screen; the new cursor stayed invisible hundreds of rows away; the user pressed again into text they could not see. **The comment I wrote on that very line asserted the opposite** ("a wrapping ⌘D can land the new cursor off-screen — follow it") — it names the ONE case that worked by accident. A regression against #272, where the new match *was* the primary. *(Found myself; critics 2 and 4 confirmed independently.)* | **REAL** — me + critics 2, 4 | New pure seam `added_member(before, after)`; the ⌘D arm scrolls to it. ⌘⇧L now scrolls **not at all** — following the primary yanked the viewport to the FIRST match in the file. |
| F7 | MED | **The find bar's `fold: true` was pinned by NOTHING.** Critic 3 flipped it to `false` — silently making the shipped find bar case-SENSITIVE — and all 452 app tests passed. `app.rs` is excluded from the coverage gate AND `refresh_efind_matches` carries a `mutants::skip` whose rationale ("find_all + the index clamp are pure-tested") became false the moment I moved a behavioral decision into that call site. | **REAL** — critic 3 | The find-bar fixture's far match is now MIXED CASE, so folding is load-bearing for the existing centering assert. **Verified by sabotage:** `fold=true` → passes in 0.226s; `fold=false` → the test no longer passes (it panics on the centering assert and then *hangs* on the PTY drop chain — this repo's documented panic-masks-as-hang signature). |
| F8 | MED | The deletion left the **normative** spec and two architecture docs naming a function and an action string that no longer exist (`SPEC-app-shell.spec.md` R19a, `app_shell.md`, `editor.md`, `keymap.rs` module doc, 4 `headless_drive.rs` comments), and R19a said nothing about the new ⌘⇧L shadow. | **REAL** — critic 3 (+ me) | All scrubbed; R19a now specifies both Editor rows and states the split-right cost. |
| F9 | LOW | The inner `match action`'s `_ =>` arm silently routed any future third action to `select_all_occurrences`. | **REAL** — critic 4 | Split into two explicit dispatch arms (which the F6 fix wanted anyway — they no longer share a body). |
| F10 | LOW | ⌘⇧L REPLACES the set (every other cursor dropped), and can return without the user's own span when that span is a non-canonical overlapping drag. | **ACCEPTED, now documented** — replacement is the reference behavior; the exclusion is inherent to non-overlapping occurrences, and ⌘D declines to add such a span for the same reason. Both gestures agree, which is REQ-007. | Doc-comment states it. |
| F11 | LOW | A ⌘-clicked caret exactly at a new match's start is absorbed into it. | **ACCEPTED** — one offset is one cursor; the caret is upgraded to the selection. Stated as a decision, not left an accident. |
| F12 | LOW | The palette still shows a ⌘⇧L keycap for "Split Right" on an editor tab. | **REAL but PRE-EXISTING class** (#265 did it to ⌘D, #297 to ⌘⌥↑/↓) | **Follow-up ticket** — make the palette's binding hint context-aware. Out of scope. |

### What the critics RULED OUT (executed, not assumed)
- **The keymap shadow is exactly right.** ⌘⇧L → `select-all-occurrences` on Editor, still `split-right` on
  Terminal/cockpit, resolves at any stack depth. Critic 4 **reversed the entire binding table** and re-resolved
  all 50 rows × 5 stacks → 0 differences: resolution ranks by context depth, not row order. It counted the
  roster from source (50 rows = 41 global + 9 scoped; 45 unique chords) rather than trusting my bumped numbers,
  and enumerated every `(chord, context)` pair → **0 same-context collisions**. `split-right` stays reachable
  from the editor via ⌘⇧P.
- **No find-bar regression.** `replace_all` takes a pre-computed match list and never calls `find_all`, so
  Replace All / Replace One could not have flipped to case-sensitive.
- **Unicode is clean, and `fold: false` is strictly *more* precise.** `find_all` indexes the rope by CHAR; no
  byte confusion, no panic, exact offsets for `héllo` / `日本` / emoji.
- **No empty needle is reachable.** `word_range_at` can never return an empty range (`end ≥ pivot+1`), and both
  matchers guard `ndl.is_empty()` anyway.
- **IME, repaint, and the no-editor-focused path are all safe.** `clear_marked` COMMITS a composition (the
  marked text already lives in the buffer); `cx.notify()` fires unconditionally on the key path; `editor_mut()`
  → `None` is a clean no-op.
- **No user keymap file exists**, so deleting the action string cannot break a user's binding.

### §0 consequence — `next_occurrence` is now DEAD, so it is DELETED
With ⌘D on `find_all`, the wrapping single-match scan has no production caller. Same reasoning that retired its
sibling `select_next_match` in Phase 3. Its valuable assertions (char-offset scanning, degenerate needles) were
ported onto `find_all`'s exact path, which had none — every prior `find_all` test passed `fold: true`.

### Process lessons
1. **PARALLEL CRITICS ON ONE WORKING TREE CORRUPT EACH OTHER'S EVIDENCE.** Critic 1 got two findings back as
   false PASSES because another critic's transient patch was on disk when it compiled; it caught the impossible
   result, copied the crate to an isolated workspace, and re-ran everything there. Critic 4 saw the tree change
   three times mid-review. **A critic that mutates the tree must work in an isolated copy or a worktree** — and
   I should snapshot the tree before spawning any (I did, which is the only reason the diff survived intact).
2. **"It has its own test" is not the same as "the test asserts the right thing."** The D3 exhausted-guard had a
   passing test and a confident doc comment. Both encoded a premise — *"the wrap returned a match we hold ⇒ we
   hold them all"* — that is simply false once the set can hold more than one member.
3. **A comment can assert the behavior the code does not have.** I wrote "a wrapping ⌘D can land the new cursor
   off-screen — follow it" directly above a call that follows the primary, which a wrapping ⌘D is the *only*
   case that accidentally satisfies. Third occurrence of this class in three tickets.

### Forge capture
- **Failures:** `BF-claude-cmd-d-exhausted-guard-false-fired-after-the-wrap-001`,
  `BF-claude-two-match-engines-disagreed-on-overlap-and-grew-the-selection-001`,
  `BF-claude-additive-gesture-followed-the-primary-so-the-new-cursor-was-invisible-001`,
  `BF-claude-branch-discriminator-asked-about-member-0-not-the-whole-set-001`.
- **Prevention rules:** `PR-claude-parallel-critics-must-not-share-a-working-tree-001`,
  `PR-claude-two-engines-answering-the-same-question-will-diverge-001`,
  `PR-claude-a-single-member-accessor-is-not-a-question-about-the-set-001`,
  `PR-claude-decision-moved-into-a-gate-excluded-file-needs-its-own-test-001`.
- **Follow-up:** forge **#306** — the command palette advertises a chord a context-scoped row has shadowed
  (⌘⇧L → "Split Right" on an editor tab). Pre-existing class, four instances now; not a #298 blocker.

### State at Phase 3.5 exit
1092 tests pass · clippy `-D warnings` clean · rustdoc clean · no scratch files. Seven new regression tests in
`multi_cursor.rs` — each FAILS against the code as Phase 3 wrote it. Phase 4 owes the traced mutant kill set
(`cargo mutants --list` on the ACTUAL code — never guessed operators), coverage/MSI 100 on the new seams, and
the LIVE DRIVE.

## Phase 4 — Validate

**GATE GREEN [diff] — 15/15.** 1100 tests · coverage 100% lines · MSI 100% · clippy `-D warnings` · rustdoc ·
miri · visual. Proven on live pixels.

### The traced mutant set (never guessed)
`cargo mutants --list -f <file>` on both seams, then killed exactly what it printed.

| file | mutants | caught | unviable | timeout | **missed** |
|---|---|---|---|---|---|
| `multi_cursor.rs` | 44 | 33 | 11 | 0 | **0** |
| `find.rs` | 51 | 44 | 0 | 7 | **0** |

The 11 unviable are the `-> Default::default()` body mutants: `Selection`/`SelectionSet` derive no `Default`,
so they do not compile. `Option<Selection>` / `Option<(String, CharOffset)>` / `Vec<..>` DO have one, so
`added_member`, `needle_of` and `occurrences` all got viable `None` / `vec![]` / `Some(("xyzzy", 0))` body
mutants — those needed real behavior assertions, not just execution.

### Two EQUIVALENT mutants — fixed by deleting code, not by suppressing them (§0)
1. **`add_next_occurrence`'s cyclic split.** The first draft used two filters — `start >= resume` chained with
   `start < resume`. Widening the second to `<=` is an **equivalent mutant**: the extra matches it admits are
   exactly those the *first* half already offered, so nothing observable changes and no test could ever kill
   it. Rewrote the split as a **rotation** (`position(|m| m.start >= resume)` + `chain`), which says the same
   thing with one comparison. Four mutants collapsed into one killable `>=`.
2. **`word_range_at`'s `pivot + 1`** — a **pre-existing MSI hole since #265**, confirmed by mutating `find.rs`
   at HEAD in a throwaway worktree (same mutant, MISSED). `pivot`'s char is a word char by construction and
   the forward loop re-tests it, so `pivot + 1` → `pivot * 1` lands in the same place. It never surfaced
   because the gate only mutates files in the diff and `find.rs` had not been touched since. The fix is not a
   test — it is deleting the arithmetic: with both scans re-testing their own boundary char, the pivot needs
   no ± 1 at all, so the whole `pivot` branch collapses into an `on_word` / `after_word` guard. Shorter code,
   no unkillable operator. (Naively removing only the `+ 1` just moved the hole to `i - 1` → `i / 1`; the
   redundancy *was* the bug.)

### Tests added
- **REQ-001 (the foundation, written first)** — `find_all("aa")` over `"aaaa"` → the TOUCHING matches
  `(0,2)`+`(2,4)` survive `from_selections` as **TWO** cursors, and ⌘D reaches both. This was a live bug one
  commit ago; everything in the ticket rests on #297's caret/range-asymmetric merge.
- **The 7 inspect regressions** (each FAILS against the Phase-3 code) — the wrap dead-end, the ⌘-click-at-EOF
  kill, the wordless-caret stall, the swallowed range, the self-overlap growth, the viewport following the
  wrong cursor, and ⌘D≡⌘⇧L.
- `cmd_d_from_a_middle_occurrence_still_reaches_every_one` is pinned **press-by-press**, not just at the end:
  the traced set proved "eventually selects all four" is too weak to kill the `>=`→`<` split mutant (the
  mutant also reaches all four — in the wrong ORDER). The order IS the behavior.
- A caret exactly at the next match's start (absorbed, and still counted as ADDED) — two mutants live only
  there: `held`'s `&&`→`||` and `added_member`'s `&&`→`||`.
- `select_all_occurrences`' every exit path; ⌘D on a sole occurrence.
- **Keymap resolution battery** — ⌘⇧L → `select-all-occurrences` on Editor; still `split-right` on Terminal,
  in the cockpit, with no context, and ⌘⇧J untouched. `split_chords_bound` (#197) passes UNCHANGED — that is
  the proof the shadow is scoped and did not simply steal the chord.
- **REQ-003/005/006 headless** (`cmd_d_multi_cursor_edit_and_undo_headless`) — through the real key router:
  ⌘D ⌘D → `[(0,5),(11,16)]` → one `z` → `"z beta z gamma alpha"` → one ⌘Z reverts BOTH → the cursor SET is
  restored → ⌘⇧L → all three occurrences.

### Dead code removed
`select_all_occurrences`' `if matches.is_empty()` guard was **unreachable** (the needle is the buffer's own
text, so it always occurs at least where it came from) — it was the single uncovered line. Deleted rather than
covered with a fake test.

### THE LIVE DRIVE — proven on pixels
Bundled debug, quit the stale instance, `open target/Marley.app`, drove `find.rs` in the editor.
**No `focus` verb after the first click** — `focus` CLICKS at (0.5, 0.12), and a plain click COLLAPSES the
cursor set by design (#297 D4), which would have destroyed the very thing under test.

| capture | what the pixels showed |
|---|---|
| `06-cmdD-twice.png` | **`let ⟦rope⟧ = buffer.⟦rope⟧();`** — ⌘D ⌘D → TWO highlighted occurrences, the first KEPT. Tab count unchanged → the Editor row fired, not the global `new-terminal`. |
| `07-typed.png` | **`let ZZZ = buffer.ZZZ();`** — ONE typed run rewrote BOTH cursors (REQ-006). |
| `08-undone.png` | ONE ⌘Z reverted the typed run at BOTH cursors together (`ZZZ`→`Z` in both places — the #297 group-level coalescing; the first keystroke replaced a selection, so it is its own group). |
| `10-cmdshiftL.png` | **⌘⇧L → EVERY `rope` selected** (line 26 ×2, 31, 32, 47) — and the pane did NOT split. |

`find.rs` on disk was never touched (⌘S was never pressed; zero `ZZZ` on disk afterwards, verified).

### Pre-existing, documented, not in scope
- `headless_drive.rs` shows 4 missed *regions* (0 missed lines) — the gate's floor is LINES, which is 100%.
- The `word_range_at` MSI hole was pre-existing (#265) but is FIXED here, since §0 forbids leaving a red gate.
