# Auto-close brackets + quotes (#338) — Notes

- **Forge ticket:** #338 19428bae-8f16-4bdb-b3c8-c9850e54dd27
- **AAR:** acd1a989-2571-43db-82d1-78a7a467c007
- **Local ticket doc:** ../../tickets/closed/TICKET-338-autoclose.md
- **Pipeline spec:** 338-autoclose.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch). -->

## Phase 1 — Plan

Promoted `queued/m22-autoclose.spec.md` → `active/338-autoclose.spec.md` (pipeline_id
`c3ce81e3-dbcb-4792-876d-e4d935608c22`, AAR `acd1a989…`, local ticket doc written). Classification: work
pipeline, feature, one slice. `active/` was empty. Forge recall returned nothing above noise (top RRF ~0.04) —
the evidence below is all first-hand code reading.

### THE VERIFICATION LEDGER — the seams are real; **the spec's central bullet is not**

Run 3 of promote-don't-author. **A first pass existed in the design-note (I wrote it while blocked on #337's
gate) and this phase treated it as a CLAIM, not a fact — correctly, because it was imprecise in two places**
(the IME door and the wrap severity). That is the method working on its own author.

| # | Claim | Verdict |
|---|---|---|
| 1 | `edit_at_selections_with` exists (the #297 engine) | **VERIFIED** — buffer.rs:308, `F: Fn(&Buffer, Selection) -> String` |
| 2 | `backspace_at_selections` exists | **VERIFIED** — buffer.rs:333 — **but see F3: it hardcodes its 1-char widening** |
| 3 | `auto_close` is unbuilt ("no ticket exists") | **VERIFIED** — zero hits repo-wide |
| 4 | "every action applies through `edit_at_selections_with` (one undo group, carets restored)" | **F1 — WRONG for 3 of its 4 actions.** The engine's post-state is bare CARETS at the end of each insert. |
| 5 | The pure table is the whole risk; "the app shim only maps `Action` onto the shipped edit fns" (D-PURE-TABLE) | **F2 — the shipped fns cannot express 3 of the 4 actions, and the seam that could is PRIVATE.** |
| 6 | The insert path / where a typed char lands | **DRIFTED — my own note was imprecise, and the truth is BETTER.** Not app.rs's shim: `ime::replace_text` (crates/editor/src/ime.rs:100), in the EDITOR crate. |
| 7 | IME interaction (the spec never mentions it) | **ALREADY ANSWERED by the code's shape** — `replace_text` branch 1 IS ordinary typing; branch 2 is composition/platform-range. |
| 8 | ⌘Z restores all carets exactly (REQ-007) | **VERIFIED as a mechanism** (`edit_ranges_restoring` records `restore`) — **but F1's fix-up can break the COALESCING half; see F1b** |

---

**F1 [BLOCKING — the engine does the TEXT half only].** `edit_ranges_restoring` computes its post-state via
`selections_after_multi_edit` (selection.rs:247), and the arithmetic is unambiguous (:253-255):

```rust
let pos = sel.start().as_usize().saturating_sub(removed_before) + inserted_before + repl_len;
out.push(Selection::caret(CharOffset::from(pos)));
```

The caret lands at `start + repl_len` — **the END of the replacement** — and it is **always a bare
`Selection::caret`, never a range**. Consequences per action:

- **InsertPair** (`"()"`, repl_len 2) → the caret lands **after the `)`**, not between. Needs a −1 fix-up.
- **Wrap** (`open + sel + close`) → the caret lands after the closer **and the selection is DESTROYED**. This
  is worse than my pre-verification note said ("leaves it after the closer"): the engine cannot return a
  RANGE at all, so REQ-005's "selection preserved inside, at EACH of N cursors" cannot be expressed by this
  engine's return value. #338 must build its own `SelectionSet`.
- **TypeOver** → not an edit at all. Routed through the engine with an empty replacement it hits the no-op
  sweep (buffer.rs:396-400: `nothing_inserted && all carets` → `set_selection(set); return set`), which
  records nothing AND parks the caret where it already was. So it must be a pure selection move.
- **Insert** (the ordinary char) → unaffected; today's path exactly.

**F1b [the trap inside the fix-up].** That caret placement is not incidental — buffer.rs:428-431 calls it
*"the precondition — and the ONLY one — under which `UndoHistory::coalesces_into` may append a following
keystroke char-wise, which is what makes a typed RUN at N cursors undo as one word."* And buffer.rs:404-411
adds the single-cursor half: **one cursor ⇒ NO undo group at all**, because `UndoHistory::record` refuses to
coalesce into a bracketed group (guard: `sel_before.is_none()`), so bracketing every keystroke would make ⌘Z
undo one CHARACTER at a time and *"destroy the #253/#282 feel"* on *"every keystroke in the editor"*.
→ **A naive caret fix-up (or a stray `begin_undo_group`) silently breaks ⌘Z GRANULARITY, and no pair test
would catch it.** Design must state where the fix-up lives relative to the group, and Validate needs an
explicit typed-run-coalescing regression (type `foo` at 2 cursors → ONE ⌘Z, not three).

**F2 [D-PURE-TABLE is REOPENED — and the answer is better than the spec's].** The spec says the app shim only
maps `Action` onto shipped edit fns. It cannot:
- `edit_ranges_restoring` — the only seam that can name *edit ranges* and *restore ranges* separately — is
  **PRIVATE** (buffer.rs:359, `fn`, no `pub`). The app crate cannot reach it.
- `backspace_at_selections` **hardcodes** its consuming range (`head - 1`, buffer.rs:341-347), so
  backspace-pair cannot "ride it with a widened span" (the spec's words) without changing it.
- `set_selection` IS pub (buffer.rs:536; the app calls it 14×), so a shim-side correction is *possible* —
  but it would land in a `#[mutants::skip]` app shim, i.e. the one place a wrong caret is invisible to the
  mutation gate. That is exactly where #336's `chars().count()` bug hid.
→ **The work belongs in the EDITOR crate** (a pair-aware pub fn beside `backspace_at_selections`, using the
private engine and returning the right `SelectionSet`). That is a file-manifest change the spec never
anticipated — and it is a WIN: the editor crate is fully mutation-tested, so the selection half gets cov/MSI
100 instead of hiding in a skip. **Design must decide and record: editor-crate fn vs shim `set_selection`.**

**F3 [the door is not where my note said, and the IME question answers itself].** `ime::replace_text`
(crates/editor/src/ime.rs:100) is the door, not app.rs's `replace_text_in_range` (which drifted to :11338
post-#337 — my note's :11332 was already stale, as predicted). Its doc is explicit: *"This is where ORDINARY
TYPING lands… a printable key returns early from the app's key router — deliberately, so the platform text
path stays 'the ONE insert mechanism, IME included' — so a typed char reaches the buffer through here and
nowhere else."* And it already splits exactly where #338 needs:
- **branch 1** — `range_utf16.is_none() && marked.is_none()` → ordinary typing → `edit_at_selections` at every
  cursor. **This is auto-close's hook site.**
- **branch 2** — a platform-addressed range OR committing a live composition → one span, collapses to a
  single caret.
→ **Hooking branch 1 excludes mid-composition pairing BY CONSTRUCTION** — the spec's silence on IME turns out
to be answerable without a new mechanism. Record it as a decision rather than leaving it implicit.

**F4 [`text` is a `&str`, not a `char` — the table needs a gate].** `replace_text(buffer, marked,
range_utf16, text: &str)`. The spec's `pair_action(typed: char, …)` assumes one char. ime.rs:97-99 warns that
branch 2 *"is not exotic — macOS uses it for press-and-hold accent selection and for Text Replacement /
autocorrect, on a stock US keyboard"*, and branch 1 can still deliver a multi-char `text`. → The table must
fire **only when the inserted text is exactly one char** (and only on branch 1). A multi-char insert is an
ordinary insert. Design states the gate; Validate pins it (a Text-Replacement-style multi-char `text`
containing `(` must NOT pair).

### Confirmations
- **§20 CONFIRMED N/A-by-observation**: VS Code = observed behavior only (the pair set, autoCloseBefore,
  type-over, backspace-pair, wrap-on-type) — universal editor affordances; the decision table is Marley's own;
  no copyleft source read.
- **The EARS AC (REQ-001..009) STAND as written** — they describe observable behavior, and F1/F2 change the
  *implementation* of REQ-004/005/007, not their truth conditions. **REQ-005 is the one to watch**: it is the
  only REQ the shipped engine cannot express, so it is the one at risk of being quietly weakened at Design.
- **Decisions:** D-ADJACENCY-NOT-BOOKKEEPING, D-WRAP-ON-TYPE, D-LIFETIME-GUARD, D-OFF-IDENTICAL all **stand**.
  **D-PURE-TABLE is REOPENED** by F2 (its second clause — "the app shim only maps Action onto the shipped edit
  fns" — is false). Its FIRST clause (one total pure `pair_action`) is untouched and still right.

### Forks for Design
- **Fork A (F2):** the selection half — a new pub fn in `crates/editor/src/buffer.rs` (uses the private
  engine, returns the correct `SelectionSet`, mutation-tested) **vs** an app-shim `set_selection` correction
  (no editor-crate change, but lands in a `mutants::skip`). Recommend the former; Design must confirm the
  private engine can express all four actions, or say what it needs.
- **Fork B (F1b):** where the caret fix-up sits relative to the undo group, such that typed-run coalescing
  survives. This is the ticket's real risk and it is invisible to every pair-shaped test.

### Standing context
**LIVE synthetic-input drives are OFF-LIMITS** (chad is at the machine — a drive types into HIS frontmost
window) → units + headless + mechanism, documented as deferred-not-skipped. **Push remains UN-OK'd** (all
commits LOCAL; #337 landed at `6fd0855`). Standing traps + the two new #337 lessons (**a doc-only fix has no
verifier**; **an assertion on the default cannot fail**) are in
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md).

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

**Both forks turned out to be ONE question, and the undo machinery answers it decisively.** The answer also
makes the ticket smaller than Phase 1 feared, because the generalization's identity case IS today's behavior.

### FORK A + B — RESOLVED TOGETHER: the fix-up goes INSIDE the engine. The shim option is not "worse", it is WRONG.

Fork A asked *where* the selection half lives; Fork B asked *where the fix-up sits relative to the undo group*.
They are the same question, because **the undo group STORES the post-selection**:

- `edit_ranges_restoring` brackets with `begin_group(restore, true)` … `end_group(after.clone())`
  (buffer.rs:432/441) — so `sel_after` is baked from the engine's own computed carets.
- **REDO restores `sel_after`** — verified: buffer.rs:504-512 `match &group.sel_after` → `set_selection(...)`.
- **`coalesces_into` compares `prev.sel_after == group.sel_before`** — verified: undo.rs:181.

So the shim shape (`edit_at_selections_with(...)` then `set_selection(corrected)`) breaks **two** things that no
pair-shaped test would ever catch:

1. **Redo restores the WRONG carets** — `sel_after` still holds the uncorrected "after the `)`" set. The text
   redoes correctly; the carets land in the wrong place. Silent.
2. **The N-cursor typed run stops coalescing** — the next keystroke's group carries `sel_before = corrected`,
   so `prev.sel_after (uncorrected) != group.sel_before (corrected)` → `coalesces_into` is false → **⌘Z undoes
   one character at a time**, which is precisely "the very harm the single-cursor rule exists to prevent,
   reintroduced at exactly the moment there are more cursors to lose" (undo.rs:150-152, #297 inspect C10).

**DECISION (D-FIXUP-IN-ENGINE):** the post-selection is computed INSIDE the engine, before `end_group`. Option
(b) is rejected on correctness, not on testability — which is a stronger reason than Phase 1 had. `set_selection`
being `pub` is a trap here, not an affordance.

### The shape: generalize the engine's post-selection from a LENGTH to a per-cursor SPAN

Today `selections_after_multi_edit(set, repl_lens)` (selection.rs:247) computes, per cursor:
`pos = start − removed_before + inserted_before + repl_len` → `Selection::caret(pos)`.

Generalize the last term from `repl_len` to a caller-supplied **per-cursor (anchor_rel, head_rel)**, measured
from that cursor's own post-edit base:
`base = start − removed_before + inserted_before` → `Selection::new(base + anchor_rel, base + head_rel)`.

**The identity case `(repl_len, repl_len)` reproduces today's math EXACTLY** — same offset, and
`Selection::new(p, p)` ≡ `Selection::caret(p)`. That is the #331 idiom: the general fn's degenerate input must
be byte-identical to the pre-ticket path, pinned by a property test. All 25 `edit_at_selections*` call sites
keep today's behavior untouched; only ONE production caller of `selections_after_multi_edit` exists
(buffer.rs:402), so the blast radius is contained.

**Action → plan** (per cursor: a replacement + a post-selection span). `Selection::new(anchor, head)` is pub
(selection.rs:32), so a RANGE post-state is expressible — which is the whole reason REQ-005 becomes possible:

| Action | edit range | replacement | (anchor_rel, head_rel) | why |
|---|---|---|---|---|
| **Insert** (today) | the cursor | `c` | `(1, 1)` | the identity — today's exact behavior |
| **InsertPair** | the cursor | `open+close` | `(1, 1)` | caret BETWEEN the pair |
| **Wrap** | the selection | `open+sel+close` | `(1, 1+sel_chars)` | **REQ-005: the selection preserved on the INNER text** |
| **TypeOver** | zero-width | `""` | `(1, 1)` | the caret steps PAST the existing closer — no text written |
| **Backspace-pair** | a WIDENED 2-char consuming range | `""` | `(0, 0)` | both chars go; caret where they were |

### InsertPair's undo unit needs NO code — the machinery already does the right thing (verified)

Worth stating because the obvious instinct is to bracket a group and that instinct is harmful:

- `record` coalesces only when `rec.inserted.chars().count() == 1` (undo.rs:80). InsertPair inserts **2** chars
  → never coalesces → it becomes its own one-record group. ⌘Z removes `()` as one unit — the observed VS Code
  behavior, for free.
- The FOLLOWING char cannot coalesce into it either: the contiguity test is
  `last.at + last.inserted.chars().count() == rec.at` (undo.rs:82) → `at+2` vs the caret at `at+1` → false →
  a fresh group. So `(` then `x` gives `(x)` with a separate ⌘Z for `x`. Correct.
- Same at N≥2: `coalesces_into` requires `new.inserted.chars().count() == 1` (undo.rs:185) → a 2-char record
  never joins a run.
- **Therefore #338 must NOT call `begin_undo_group`.** Doing so would set `sel_before = Some(..)`, and
  `record`'s guard (`sel_before.is_none()`) would then refuse to coalesce the *next* ordinary keystroke —
  "destroying the #253/#282 feel … on every keystroke in the editor" (buffer.rs:404-411). The engine
  self-brackets; nothing else may.

**REQ-012 falls out**: with auto-close ON, typing `foo` yields Insert plans `(1,1)` = today's math → identical
`after` → identical `sel_after` → `coalesces_into` unchanged → ONE ⌘Z. The regression is pinned *because* the
default plan is the identity.

### TypeOver's edge — the no-op sweep (the design's one genuine subtlety)

buffer.rs:396-400: `nothing_inserted && all carets` → `set_selection(set); return set` — **early, before `after`
is computed**. An ALL-TypeOver keystroke (every cursor typing `)` before a `)`) inserts nothing at every cursor
and all are carets → the sweep fires → **the carets would not step over.** Two ways out; taking the honest one:

**(i) CHOSEN — a plan whose every member is a pure caret move NEVER enters the engine**: it is a
`set_selection` move, full stop. This is not a workaround, it is the truth — stepping over a closer writes no
text, so it must not burn an undo record, must not bump the version, and must not mark the file dirty. The
predicate ("no member removes anything and no member inserts anything") is pure and unit-testable.
**(ii) rejected** — teaching the sweep's guard about the post-selection map touches a guard with a documented
rationale (#296 inspect F6) to serve a case that shouldn't reach it.

A **MIXED** set (cursor 1 TypeOver, cursor 2 InsertPair) *does* edit, so the sweep does not fire; TypeOver's
member is a caret with an empty repl, so `acts()` (buffer.rs:418) skips its edit while `after` — computed over
the FULL set — still gives it `base + 1`. Mixed works through the engine, unmodified. **This is the case the
spec flagged as "the subtle bit" and it resolves cleanly**: per-cursor action resolution produces per-cursor
plans, and the engine was already per-cursor (`F: Fn(&Buffer, Selection) -> String`) — it only lacked the
selection half.

### The hook site + the two new gates

- **D-ORDINARY-TYPING-ONLY confirmed, no new mechanism.** `ime::replace_text` (ime.rs:100) already splits:
  branch 1 (`range_utf16.is_none() && marked.is_none()`) is ordinary N-cursor typing; branch 2 is a
  platform-addressed range or a composition commit, which "name exactly ONE span" and collapse to a single
  caret. Auto-close hooks **branch 1 only** → mid-composition pairing is excluded *by construction*. Only 2
  callers of `replace_text` exist (ime.rs:253 test harness, app.rs:11352 shim), so threading the setting in as
  a parameter is cheap.
- **D-ONE-CHAR-ONLY** — `text: &str` vs `pair_action(typed: char, …)`. The gate is a pure
  `typed_char(text) -> Option<char>` (exactly one char, else `None`) living in `auto_close.rs` → REQ-011 is a
  unit test, not a shim assertion. `None` ⇒ ordinary insert. This also covers the everyday branch-2 reach
  ime.rs:97-99 warns about (press-and-hold accents, Text Replacement).
- **D-OFF-IDENTICAL** — `auto_close: false` ⇒ every cursor's plan is the identity `(1,1)` ⇒ byte-identical to
  the pre-ticket path. Same property-test posture as #331's empty-slice pin.

### §20 — CONFIRMED, still N/A-by-observation
VS Code = **observed behavior only** (the pair set, autoCloseBefore, type-over, backspace-pair, wrap-on-type).
The decision table and the plan/engine generalization are Marley-original — they are shaped by *our* undo
machinery, which is the opposite of a port. No copyleft source read. (Reading our own crates and gpui
(Apache-2.0) is adoption, outside the wall — the #336 precedent.)

### File manifest

| File | Change |
|---|---|
| `crates/editor/src/auto_close.rs` | **NEW, PURE** — `Action{InsertPair,TypeOver,Wrap,Insert}`, the total `pair_action(typed, prev, next, has_selection)`, the pair table (`()` `[]` `{}` `""` `''` `` ` ``), the `'` lifetime guard, `typed_char(&str) -> Option<char>`. cov/MSI 100, no skips. **Lives in the EDITOR crate, not the app** (Phase 1 F2) — the whole feature stays in a mutation-tested crate. |
| `crates/editor/src/selection.rs` | Generalize `selections_after_multi_edit(set, repl_lens)` → a per-cursor post-selection span; the identity case reproduces today's math byte-for-byte (property-pinned). |
| `crates/editor/src/buffer.rs` | `edit_ranges_restoring` carries the plan (repl + span) to `after`/`end_group`; `edit_at_selections`/`_with` keep today's behavior via the identity plan; **NEW pub fn** for the pair-aware insert at N cursors; backspace-pair via a widened consuming range (a sibling/param — `backspace_at_selections` hardcodes `head−1` at :341-347). |
| `crates/editor/src/ime.rs` | `replace_text` branch 1 gains the auto-close hook, gated on the setting + `typed_char`. Branch 2 untouched. |
| `crates/editor/src/lib.rs` | `mod auto_close;` + re-export. |
| `crates/marley_app/src/settings.rs` | `editor.auto_close` (bool, **default true**) — the #330 four-wiring + `persist_auto_close` + the **NON-DEFAULT (false) round-trip leg**. |
| `crates/marley_app/src/app.rs` | Thread the setting into the `replace_text` call (:11352); the palette toggle verb; live field + resolve at boot. |
| `crates/marley_app/src/palette.rs` | "Toggle Auto-Close Brackets" row + the completeness test. |
| `crates/marley_app/src/headless_drive.rs` | The N-cursor drives (REQ-005/007/012). |

### Regression Test Plan

| REQ | Test | Where | Notes |
|---|---|---|---|
| REQ-001 | `pair_action` table: every opener × next ∈ {none, ws, closer} → `InsertPair` | auto_close.rs | pure |
| REQ-002 | opener before a WORD char → `Insert` (only the opener) | auto_close.rs | `(` before `foo` ≠ `()foo` |
| REQ-003 | closer with `next` == that closer → `TypeOver`; **+ headless** step-over | auto_close.rs + headless_drive.rs | |
| REQ-004 | backspace between an adjacent pair → both deleted; **undo round-trip** restores text AND carets | buffer.rs | the widened span |
| REQ-005 | **wrap at N=3 cursors, selection preserved on the inner text at EACH** | buffer.rs + headless | **the row that needs thought — see below** |
| REQ-006 | `'` after `&`/ident → `Insert`; after ws/`,`/`=`/BOL → `InsertPair` | auto_close.rs | the Rust-shaped guard |
| REQ-007 | N-cursor action = ONE undo unit; ⌘Z restores text + all carets exactly | buffer.rs + headless | |
| REQ-008 | OFF ⇒ **byte-identical** to the pre-ticket insert path (property, random input) + the settings **non-default (false)** round-trip leg | auto_close/buffer + settings.rs | the #337 lesson: the leg must persist FALSE, since the default is TRUE |
| REQ-009 | totality: `pair_action` over the full (typed × prev × next × sel) cross-product — no panic | auto_close.rs | the #336 sweep idiom, not hand-picked rows |
| REQ-010 | branch 2 (a platform range / live composition) never pairs | ime.rs | the branch gate |
| REQ-011 | `typed_char`: multi-char text → `None` ⇒ a Text-Replacement expansion containing `(` inserts literally | auto_close.rs | |
| REQ-012 | **typed-run coalescing survives**: type `foo` at N cursors with auto-close ON → ONE ⌘Z | buffer.rs + headless | **the row that needs thought — see below** |
| — | **the identity property**: `selections_after_multi_edit` with `(repl_len, repl_len)` ≡ the pre-ticket fn, over random sets | selection.rs | the #331 byte-identity pin |
| — | **redo** after InsertPair restores the caret BETWEEN the pair (not after the `)`) | buffer.rs | **the Fork-B bug, pinned directly** |

**The two rows that need real thought, and why a green pair test proves neither:**
- **REQ-005** — "the selection is preserved" is a claim about the returned/stored `SelectionSet`, not about text.
  A test that asserts the buffer reads `(foo)` passes even when the selection was destroyed. The assert must be
  on the **selection spans at each of the 3 cursors**, and it must also check **`sel_after`** (via a redo), or
  the shim-shaped bug survives.
- **REQ-012** — coalescing is invisible in the text and in the live selection. The only observable is **⌘Z
  granularity**: type `foo` at N cursors → exactly ONE undo step returns to empty. Negative-smoke it (the #337
  discipline): break the plan to a non-identity and watch the test fail, or it is not proving anything.

**Uncoverable / deferred:** LIVE pixel drives are **OFF-LIMITS** — chad is at the machine and synthetic input
lands on HIS frontmost window. Everything here is unit + headless + mechanism; document as
deferred-not-skipped, per #336/#337.

### Risks

1. **The engine is the app's ONE insert path** (buffer.rs:421-422, ime.rs:85-89) — every keystroke in the
   editor goes through it, so the generalization touches the hottest code in the editor. *Mitigation:* the
   identity case is today's math, pinned by a property test before any pair behavior is added; 25 call sites
   keep their exact behavior; only one production caller of the changed pure fn.
2. **A wrong `sel_after` is silent** — text correct, carets wrong, only visible via redo or a broken run.
   *Mitigation:* the two dedicated rows above; the fix lives in-engine so `sel_after` is right by construction.
3. **`begin_undo_group` is a loaded gun here** — bracketing InsertPair would break coalescing for every
   subsequent keystroke. *Mitigation:* recorded as a decision (the engine self-brackets; #338 adds no group)
   and REQ-012 catches a violation.
4. **The `'` guard is Rust-shaped** in a table that #315 will make multi-language. *Accepted + documented* per
   the spec; the table is the one place to adjust.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement (PASS 1 of 2 — the pure seam + the engine; the ime/settings/app wiring is pass 2)

### Built (pass 1)

1. **`crates/editor/src/selection.rs` — the post-state generalized from a LENGTH to an optional SPAN.**
   `selections_after_multi_edit(set, repl_lens, spans: &[Option<CaretSpan>])`. `base = start − removed_before
   + inserted_before` is now named separately, and the span supplies the rest: `Selection::new(base +
   anchor_rel, base + head_rel)`. `None`/short/`&[]` ⇒ the identity `(repl_len, repl_len)`. New
   `pub(crate) type CaretSpan = (usize, usize)`, documented as an offset from the base and **deliberately not
   bounded by the replacement's length** — type-over inserts nothing yet lands at `base + 1`.
2. **`crates/editor/src/buffer.rs` — the span rides to `after`/`end_group`.** New PRIVATE
   `edit_ranges_restoring_placing` (`F: Fn(&Buffer, Selection) -> (String, Option<CaretSpan>)`);
   `edit_ranges_restoring` is now its wrapper passing `None`, so all 25 `edit_at_selections*` call sites and
   the public signatures are untouched. Its doc states WHY the placement is decided here and not corrected
   afterwards (the D-FIXUP-IN-ENGINE evidence: `end_group` → `sel_after` → redo + `coalesces_into`), and names
   `set_selection` as the trap it is. The no-op sweep, `acts()`, the back-to-front loop and every explanatory
   comment are preserved verbatim. **No undo group was added anywhere.**
3. **`crates/editor/src/auto_close.rs` — NEW, PURE, in the EDITOR crate** (Phase 1 F2, so the feature is
   mutation-tested rather than hiding in a shim). `Action{InsertPair,TypeOver,Wrap,Insert}`; the total
   `pair_action(typed, prev, next, has_selection)`; `PAIRS` (6, quote-likes self-pairing); `pub closer_of`
   (the text half needs it); `pub typed_char(&str) -> Option<char>` (D-ONE-CHAR-ONLY); private `is_closer` /
   `is_word` / `opens_here` / `blocked_quote`. The module doc says why totality is not defensiveness here: it
   runs on **every keystroke in the editor**, so a panic is the editor dying under the user's hands.
4. **`crates/editor/src/lib.rs`** — `pub mod auto_close;` with a doc, placed after `anchor` (alphabetical),
   matching the `comment`/`ime`/`indent` "pure module the shim routes through" idiom (`#![deny(missing_docs)]`
   is on, so every public item carries one).

### Deviations from design (both small, both recorded)

- **The design said "generalize `selections_after_multi_edit`" and implied a wrapper pair** (the #331
  `line_layout`/`line_layout_with_inlays` idiom). I wrote that first, then **removed it**: #331's wrapper earns
  itself because `line_layout` has many callers, whereas this fn has exactly **one** production caller
  (buffer.rs:402), so the wrapper would have been dead the moment buffer.rs routed through the spans version.
  One fn taking `&[Option<CaretSpan>]` keeps the identical property (`&[]` ≡ the pre-#338 fn) with less code
  and no dead seam. The wrapper idiom moved UP one level instead, to `edit_ranges_restoring` — where it does
  earn itself (25 call sites).
- **`Option<CaretSpan>` rather than a resolved `CaretSpan`.** Resolving `None → (repl_len, repl_len)` in
  buffer.rs would have worked, but it computes the identity in the caller and leaves the pure fn's default
  path unexercised by production. Keeping the `Option` means the identity lives in ONE place and every
  ordinary keystroke actually takes it.

### The identity, proven at each step (not asserted)

- `Selection::caret(off)` is `{anchor: off, head: off, goal_col: None}` and `Selection::new(p, p)` is
  `{anchor: p, head: p, goal_col: None}` — **read the constructors** (selection.rs:32/41) before writing the
  doc claim, per the #337 F5 lesson (a doc-only claim has no verifier).
- After the selection.rs change: `cargo nextest run -p marley_editor --lib` → **166 passed**.
- After the buffer.rs change: **166 passed** again — including `t296_multi_edit_differential_fuzz` and the
  undo coalesce tests, which are the ones that would notice.
- Whole tree: `cargo clippy -p marley_editor --all-targets` CLEAN; **`cargo nextest run --workspace` → 1488
  passed, 5 skipped** (the same 1488 #337 shipped with). The engine change is behaviorally invisible, which
  is the entire claim.
- `cargo fmt --all`; `git add -N crates/editor/src/auto_close.rs`; §20 clean (no reference-app names in
  `crates/`).

### One bug found reviewing my own pass 1, before inspect saw it

**The `'` lifetime guard was gating the Wrap arm, where it is DESTRUCTIVE rather than conservative.** With
`let x = &foo` and `foo` selected, typing `'` hit `blocked_quote` (prev == `&`) → fell through to `Insert` →
**replaced the selection, eating `foo`**. The guard would have destroyed the user's text to avoid a bracket it
was never going to type.

It is wrong on the spec's own terms too: D-LIFETIME-GUARD says `'` must not **pair** after `&`/an ident
(`&'a`, `T: 'b`) — and a lifetime has no selection to wrap — while D-WRAP-ON-TYPE says a selection plus an
opener wraps. So the guard belongs on the `InsertPair` arm only. Fixed, with the reason written at the arm.
The lesson generalizes: **a "conservative" guard that falls through to a DESTRUCTIVE default is not
conservative.** Worth a critic lens at inspect — the other arms' fall-throughs are all `Insert` at a bare
caret, which writes one char and destroys nothing, so this arm was the only one where the default bites.

### Built (pass 2) — the wiring

5. **`crates/editor/src/buffer.rs`** — `pub insert_pairing_at_selections(set, typed, origin)`: resolves an
   `Action` per cursor from its OWN neighbours and maps it to a `(replacement, span)` plan, so a mixed
   keystroke wraps at one cursor while pairing at another. **The pure-move sweep never touches the buffer** —
   if no cursor writes, it is a `set_selection` and nothing else (D-TYPEOVER-IS-NOT-AN-EDIT; it is also the
   only shape that works, since an all-type-over sweep is exactly the engine's no-op guard and would return
   the carets unmoved). `pub backspace_pairing_at_selections` + a shared private `backspace_consuming(…,
   pairing)`, so the pre-#338 `backspace_at_selections` keeps its exact body via `pairing: false`. New
   `pub neighbours_at(off) -> (Option<char>, Option<char>)` — the one place buffer knowledge enters the
   decision, which is what keeps offset math out of the app.
6. **`crates/editor/src/auto_close.rs`** — added `pub is_pair(open, close)` (backspace-pair's whole question).
7. **`crates/editor/src/ime.rs`** — `replace_text` takes `auto_close: bool`; **branch 1 only** routes to
   `insert_pairing_at_selections`, gated on `typed_char(text)`. Branch 2 untouched, so compositions and
   platform ranges never pair — structurally, with no IME-awareness anywhere in the feature.
8. **`crates/marley_app/src/settings.rs`** — `AutoClose: bool = true` @ `editor.auto_close` + the #330
   four-wiring + `persist_auto_close`. 3 `AppliedSettings` test ctors gained the field.
9. **`crates/marley_app/src/input.rs`** — `apply_editor_key_multi` takes `auto_close`; backspace routes to the
   pairing variant when on.
10. **`crates/marley_app/src/{app,palette}.rs`** — the live `auto_close_on` field + boot resolve;
    `toggle_auto_close` (+persist); `CommandId(19)` → `toggle-auto-close` → "Toggle Auto-Close Brackets"; the
    flag threaded into both call sites.

### Deviations + traps hit in pass 2

- **The borrow order bit twice, in the same shape as #337's E0716**: both `replace_text_in_range` and the key
  router take `&mut` on the shell before they need the flag, so `self.auto_close_on` is read FIRST, into a
  local. Commented at both sites — it is not obvious, and the next edit there will hit it again.
- **`auto_close_on_for_test` was written and then DELETED, before clippy was even run on it** — this is
  **#337 F2 repeating exactly** (`font_size_for_test` written ahead of its unwritten Phase-4 drive → dead
  code → gate:2 red). The lesson was already in my own notes, so I applied it rather than re-discovering it
  at inspect: Phase 4 re-adds the hook if and when a test consumes it.

### Verification (pass 2)
`cargo check --workspace` clean; **`cargo nextest run --workspace` → 1488 passed, 5 skipped** — the same 1488
as before the ticket, which is D-OFF-IDENTICAL holding in practice (every pre-#338 test still asserts the
literal insert path; the two harnesses whose signatures changed pass `false` and say why).
`cargo clippy --workspace --all-targets` **CLEAN**. `cargo fmt --all`. §20: zero reference-app names in the
`crates/` diff. `git add -N crates/editor/src/auto_close.rs`. Diff: 9 files, +492/−28.

### Still open for Validate
Every test. Nothing here was test-expanded beyond the two harness signatures + 3 ctor fields needed to
compile. **The two rows that need real thought are REQ-005 (assert the SELECTION spans and `sel_after` via
redo — a text-only assert passes while the selection is destroyed) and REQ-012 (⌘Z granularity is the only
observable; negative-smoke it).**

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

2 parallel critics (the engine; the table + wiring). Both verified empirically — driven undo/redo through the
real IME door, a 26,496-row `pair_action` cross-product, a byte-diff of ime.rs branch 2 against `HEAD`, an awk
audit of all 234 `mutants::skip` attributes — and both left the tree exactly as they found it.

**They found a CRITICAL that my Phase 2 design had explicitly "verified" and got half right.** That is the
headline; everything else is smaller.

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | **CRITICAL** | **Redo CORRUPTS TEXT at N≥2 with auto-close on.** `begin_group(restore, /*cursor_anchored*/ **true**)` was hardcoded, but that flag NAMES the invariant "every cursor ends at the end of its own insert" — the precondition `coalesces_into` **trusts instead of checking** (it does no arithmetic contiguity test; undo.rs:162-163 explains why it needn't). `CaretSpan` exists precisely to break it. So the next typed char was appended onto the InsertPair record: `inserted = "()x"` at a site whose text reads `(x)`. Undo survives (char COUNT only); **redo replays `inserted` verbatim**. Driven: `(` → `x` → ⌘Z gives `"  "` (one step swallowed both) → ⌘⇧Z gives **`"()x ()x "`** instead of `"(x) (x) "`. auto_close=OFF and N=1 controls round-trip fine. | **REAL — the ticket's worst bug** (both my own probe and the critic's drive reproduce it) | `begin_group(restore, spans.iter().all(Option::is_none))` — **earn the tag**. Every pre-#338 caller stays `true`. |
| F2 | **HIGH** | **The Fork-B bug is ALIVE at N=1** — the common case. The engine opens **no group** for a single member (by design, so typing coalesces), so `record` stores `sel_after: None` and `redo` RECOMPUTES the caret as `rec.at + inserted.chars().count()` = **the end of the insert = after the `)`**. Wrap at N=1 loses its whole selection (live `(1,4)` → after ⌘Z⌘⇧Z → `(5,5)`). **D-FIXUP-IN-ENGINE's "sel_after is right by construction" is void where there is no group to hold it.** | **REAL** | Gate the no-group fast path on the SAME predicate: a non-identity span brackets a group. Costs no coalescing — `record` already refused a 2-char `()` and Wrap's non-empty `removed`. |
| F3 | MEDIUM | **`insert_pairing_at_selections` feeds ropey a STALE range → panic.** A `pub fn` taking offsets it did not produce; the Wrap arm's `text_in_range` is unguarded and the engine's clamp runs only AFTER `plan`. Proven: a `Selection::new(1, 99)` on a 3-char buffer panics in ropey, while the pre-#338 `edit_at_selections` on the identical input survives. Not live-reachable today (ime passes the always-clamped `buffer.selection()`), but it is a pub-API contract regression and it defeats auto_close.rs's own totality claim one level up. | **REAL** | Clamp `set` at the top, mirroring the engine (#296 F4). |
| F4 | MEDIUM | **Dead computation under a comment asserting what the callee discards.** The `match sel.is_caret()` recomputed `prev` and computed a `next` that `pair_action` **never reads** when `has_selection` is true (proven over the cross-product: 0 disagreements vs a `(None,None)` baseline). `cargo mutants` generates **no mutant** at a `match <bool>`, so it was invisible to the gate in both directions. I had spotted this myself before the critics reported. | **REAL** | Deleted. `neighbours_at(start).0` / `neighbours_at(end).1` is correct for BOTH cases uniformly (a caret has `start == end`). |
| F5 | MEDIUM | **The `'` guard's doc is FALSE against its own cited example.** It claimed `&'a`, `T: 'b` are covered; **`T: 'b`'s `'` follows a SPACE**, so the rule cannot see it — and `Foo<'a>` (the commonest lifetime position) produced **`Foo<'a>'`**. | **REAL — and it is the #337 F5 lesson landing on this ticket** | Added `<` (unambiguous — no char literal has `<` before the quote); doc + spec corrected to state plainly that the BOUND positions (`T: 'a`, `+ 'a`, `'static`) are **not** coverable by any one-char rule and await #315. |
| F6 | MEDIUM | **`persist_auto_close` carried `persist_inlay_hints`' doc** — my scripted insert landed the new fn *below* the existing doc line, capturing it. **The project's own `mutants-skip-detach-trap`, one attribute class over**, and `#![deny(missing_docs)]` is satisfied so nothing catches it. | **REAL** | Gave it its own doc. |
| F7 | MEDIUM | **The `has_selection`-first ordering is load-bearing and undocumented.** The buffer's TypeOver arm emits an **empty replacement** — safe only because TypeOver is unreachable with a live selection; reached with one it would DELETE the selection (`acts()` returns true for a non-caret). Proven safe today by a 26,496-row sweep. But the module doc explains the OTHER ordering at length and says nothing about the one guarding real text — inviting a future reader to hoist the type-over check. | **REAL (a latent trap, not a live bug)** | Named the invariant at the arm. Validate pins it. |
| F8 | LOW | **Wrap flattened a BACKWARDS selection to forwards.** Text correct, but `from_selections` maintains orientation on purpose ("a backwards drag still extends correctly"), so ⇧← would move the wrong end after a wrap. | **REAL** | Oriented the span off `anchor <= head`. REQ-005 says the selection is preserved; its direction is part of that. |
| F9 | LOW | `\|\| !sel.is_caret()` in the `writes` sweep can never change the result (a live selection always plans a non-empty replacement). | **REJECTED as a defect — kept deliberately.** It is a cheap guard on a hot correctness path whose mutants are killable by ordinary typing, so it costs no MSI. Recorded so it is not mistaken for load-bearing. | — |
| F10 | LOW | `plan` runs twice per cursor (the `writes` sweep + the engine); `neighbours_at` allocates 2 `String`s per call. | **REJECTED as a bug — noted as a cost.** `Buffer` has no interior mutability and is unmutated between the calls, so `plan` is pure — wasteful, not wrong; `any()` short-circuits, so ordinary typing pays ONE extra `plan`. `rope.char(i)` would avoid the allocs if it ever matters. | — |

**Rejected / verified clean (each cost real effort):** **the IDENTITY is provably exact** — `+` is left-associative so the old `x.saturating_sub(r) + i + repl_len` is the *identical expression tree* to `base + repl_len`, and `Selection::new(p,p)` ≡ `Selection::caret(p)` field-for-field including `goal_col`; **no `begin_undo_group` anywhere in the diff** (only doc mentions); **`sel_after` genuinely holds the corrected set** at N≥2 (the shim-shaped bug the design predicted IS avoided — F1 is a *different* mechanism, the record's `inserted` string); **REQ-012 holds** (`foo` at 2 cursors, auto-close ON → exactly **1** ⌘Z); **the all-TypeOver pure-move path is right end-to-end** (`"()()"` carets 1,3 + `)` → carets [2,4], **version unchanged**, `undo()` returns `None` — D-TYPEOVER-IS-NOT-AN-EDIT verified, not argued); **the mixed sweep works** (`"() x"` carets 1,4 + `)` → `"() x)"`, one steps over while the other pairs); **`head+1` cannot exceed `len_chars`** (`eats_closer` requires `next.is_some()`); **`pairing: false` reproduces the old body exactly** (diffed vs `HEAD`); **`neighbours_at` is astral-correct** (`"a😀b"` → the emoji comes back whole); **ime.rs branch 2 is byte-identical to `HEAD`** (38 lines, `diff` exit 0) so REQ-010 holds structurally; **no bypass path** — every other edit path is correctly unpaired (paste/cut/Enter/Tab/completion-accept/code-actions), and the terminal prompt cannot reach `replace_text` at all; **both borrow-order locals correct**; **the #330 four-wiring complete**; **`every_cockpit_command_resolves_to_a_verb` passes**; **the skip-DETACH trap is CLEAN** (233 → 234 skips, exactly +1 for `toggle_auto_close`, every attribute still above its intended fn).

**Mutation surface (counts run):** auto_close.rs **33 listed / 32 viable** (`pair_action -> Default::default()` is UNVIABLE — `Action` has no `Default`; the #203/#204 rule again). selection.rs 47, buffer.rs 88, settings.rs 136, app.rs 66 with **0** matching `auto_close` (shims skipped). Note `if has_selection` yields **no mutant** (a bare-bool `if` — the syntactic-form rule), so F7's ordering is **not** mutation-pinned and needs an explicit test.

**Verification after fixes:** all three fixes proven by a throwaway probe, then the probe DELETED (`grep probe338` → 0 files). **Negative-smoked**: restoring the hardcoded `true` fails the N≥2 redo probe with `left: "  "` vs `right: "() () "` — the bug is real and the fix is what kills it. `cargo clippy --workspace --all-targets` CLEAN; `cargo nextest run -p marley_editor --lib` → **166 passed**; `cargo fmt --all`; §20 clean.

### The lesson, and it is not "be more careful"

F1 and F2 are one root cause wearing two hats: **an invariant that the producer ASSERTS and the consumer TRUSTS.** `cursor_anchored: true` was a literal because it was unconditionally true when written; `coalesces_into` then skipped its own contiguity check *and said so in a comment*. #338 is the first code able to falsify it. Captured as
`BF-claude-invariant-tag-asserted-not-earned-corrupts-redo-001` +
`PR-claude-a-boolean-that-names-an-invariant-must-be-earned-001`.

And the reason **my design missed it while looking straight at it**: `coalesces_into`'s guard is **asymmetric** — it constrains `new.inserted.chars().count() == 1` and never touches `old.inserted`. I verified that InsertPair's 2-char record cannot join a *previous* run, wrote "Same at N≥2 → a 2-char record never joins a run", and treated that as covering the case. It doesn't: the bug is a *following* char joining the *InsertPair* group. **When a predicate is asymmetric, verifying one direction is not verifying it** — and the note I wrote sounded exactly as confident as if I had checked both. This is the batch's recurring shape (#336 F5's wrong citation, #337 F3's backwards physics, #337 F5's false capability claim): the decision survives review, the reason doesn't, because a plausible reason stops the reading.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**A test found a THIRD instance of the ticket's root cause** — one the two critics and I had all missed. That
is the headline; the rest is the plan executed.

### The bug Validate found: pair-backspace's ⌘Z restored the wrong caret

Writing REQ-004's undo round-trip (`a(|)b` ⌫ → ⌘Z) failed: **`left: [(3,3)]`, `right: [(2,2)]`** — the caret
came back one past where the user had it.

`buffer.rs:704-708`: an UNGROUPED record stores no selection, so `undo` RECOMPUTES one as
`rec.at + removed.chars().count()` — *the end of the restored text*. Its comment says so plainly ("one caret,
after the restored text"), and it was right for **every backspace the editor ever had**, because the consuming
range was `head-1..head` — it ENDED at the caret. #338's pair-backspace consumes `head-1..head+1`, so the
range now ends one char PAST the caret and the fallback's arithmetic silently answers a different question.

**This is the same shape as inspect's F1 and F2, for the third time in one ticket**: a fallback that assumes an
invariant nothing states, which #338 is the first code able to break. The fix generalizes rather than patches
— the ungrouped fast path is taken only when BOTH fallbacks would be right:

```rust
let fallback_is_right = ends_at_insert && restore.selections() == set.selections();
```

`ends_at_insert` is redo's precondition (the caret is at the end of the insert); `restore == set` is undo's
(the edit ranges ARE the user's cursors). Ordinary typing satisfies both and never takes a group, so its
coalescing is untouched; the edits that fail them — a 2-char pair, a wrap, any backspace — could never
coalesce anyway (`record` wants a one-char insert with nothing removed). **191 editor tests pass**, including
`t296_multi_edit_differential_fuzz` and every undo test, so the change is invisible to every pre-#338 path.

### Tests added

| REQ | Test | Where |
|---|---|---|
| REQ-001 | `t338_insert_pair_when_next_is_none_ws_or_closer`, `t338_insert_pair_puts_the_caret_between` | auto_close, buffer |
| REQ-002 | `t338_opener_before_a_word_char_does_not_pair` | auto_close |
| REQ-003 | `t338_type_over_an_adjacent_twin_closer`, `t338_type_over_moves_the_carets_without_editing`, `t338_mixed_type_over_and_pair_in_one_keystroke` | auto_close, buffer |
| REQ-004 | `t338_backspace_pair_takes_both_halves`, `t338_backspace_pair_undo_restores_text_and_caret`, `t338_backspace_pair_overlapping_ranges_merge_rather_than_double_delete` | buffer |
| REQ-005 | `t338_wrap_preserves_the_selection_at_each_of_n_cursors`, `t338_wrap_keeps_a_backwards_selection_backwards`, `t338_redo_at_one_cursor_restores_the_wrapped_selection` | buffer |
| REQ-006 | `t338_lifetime_guard_blocks_the_quote_after_amp_lt_or_ident`, **`t338_lifetime_guard_does_not_reach_the_bound_positions`** | auto_close |
| REQ-007 | the wrap N=3 undo leg; `t338_backspace_pair_undo_restores_text_and_caret` | buffer |
| REQ-008 | `t338_auto_close_off_types_literally`, the settings **NON-DEFAULT (`false`) leg**, `auto_close_toggle_persists_through_the_palette_verb_headless` | ime, settings, headless |
| REQ-009 | **`t338_pair_action_is_total_and_never_type_overs_a_selection`** — the cross-product sweep | auto_close |
| REQ-010 | **`t338_branch_two_never_pairs`** (an explicit range + a composition commit) | ime |
| REQ-011 | `t338_typed_char_accepts_exactly_one_char`, `t338_branch_one_pairs_a_single_char_only` | auto_close, ime |
| REQ-012 | **`t338_typed_run_still_coalesces_with_auto_close_on`** | buffer |
| F1 | **`t338_redo_after_insert_pair_replays_the_real_text_at_n_cursors`** — the text-corruption pin | buffer |
| F2 | `t338_redo_at_one_cursor_restores_the_caret_between_the_pair` | buffer |
| F3 | `t338_a_stale_selection_clamps_instead_of_panicking` | buffer |
| — | **`t338_absent_span_is_byte_identical_to_the_pre_ticket_math`** — the identity, over 2,000 randomized sets vs a verbatim copy of the pre-#338 body | selection |
| — | `t338_a_present_span_places_the_selection` (incl. a RANGE span and a backwards one) | selection |

**Two rows carry the whole ticket, and neither is a "pair test":**
- **The identity property** re-implements the pre-#338 body verbatim and asserts equality over randomized
  sets — because "auto-close OFF is the old behavior" and "ordinary typing is untouched" both reduce to it,
  and it is paid for by every keystroke in the editor.
- **The cross-product sweep** pins `has_selection ⟹ Wrap | Insert, NEVER TypeOver`. **It is the only thing
  pinning it**: `if has_selection` is a bare-bool `if`, which cargo-mutants does not mutate, so no mutant can
  fail if that arm is ever reordered — and the buffer maps TypeOver to an EMPTY replacement, which would
  DELETE the selection.

### Negative smokes — every critical row proven able to fail

| smoke | result |
|---|---|
| hardcode `cursor_anchored = true` | `t338_redo_after_insert_pair…` **FAILS** (`left: "  "`) — **and REQ-012 still PASSES**, proving the coalescing test would never have caught the corruption, and that the fix costs no coalescing |
| drop the `restore == set` half | `t338_backspace_pair_undo…` **FAILS** (`left: [(3,3)]`) |

Both reverted; `grep SMOKE` → 0.

### The gate's first run was RED, and both reds were real

**gate:5 mutation was GREEN on the first try — 70 caught / 0 missed, MSI 100%**, and `auto_close.rs`'s 33
mutants were confirmed present in the `--in-diff` set (the `git add -N`-or-they-are-silently-skipped trap
avoided). The two reds:

**gate:14 rustdoc, ×2 — both mine.** `pair_action`'s public doc linked `[`blocked_quote`]`, a **private** item
(rustdoc rejects a public→private link outright), and the module doc's `[`pair_action`]` cannot resolve from
module scope (`[`pair_action()`]` does — the same class as #337's `[`set_theme`]`, which is now twice in two
tickets).

**gate:4 coverage — 9 lines, and 2 of them were a design smell rather than a missing test.** buffer.rs:376/387
were the `None` arms of `match closer_of(typed)` inside the `InsertPair`/`Wrap` branches — **unreachable by my
own comment's admission** ("only an opener yields InsertPair"). The standing trap says a defensive branch is an
uncovered cold arm; the honest fix is not a test but **making the state unrepresentable**:

`Action::InsertPair` → **`Action::InsertPair(char)`**, `Action::Wrap` → **`Action::Wrap(char)`** — the closer
rides on the action, because `pair_action` already computed it and only an opener can produce those variants.
The buffer's arms collapse to `(format!("{typed}{close}"), Some((1, 1)))`. No `Option`, no unreachable arm, no
lie about what can happen. buffer.rs → **100%**.

The other 7 were ordinary gaps: selection.rs's property test passed `set.selections()` as an assert ARGUMENT —
a call that only executes on FAILURE, i.e. an uncovered region on a green run (inline `{set:?}` instead); and
`input.rs:266`, the `true` arm of the backspace match — **the branch the entire setting exists to select**,
which no test reached because every pre-#338 row passes `false`.

**A note on measuring it:** a per-crate `cargo llvm-cov -p marley_editor` UNDER-reports — it showed ime.rs at
99.55% with 4 "missed" lines that the workspace run covers via the app's headless drives. Reading a
per-crate number as the gate's would have sent me chasing phantom lines. The gate's own invocation
(`llvm-cov nextest --workspace` + its ignore-regex) is the only honest local check, and it pinpointed exactly
one real line.

### Gate — GREEN

```
PASS gate:1 rustfmt          PASS gate:11 shellcheck
PASS gate:2 clippy (-D warnings)  PASS gate:12 no-suppressions
PASS gate:3 tests (nextest + doctests)  PASS gate:13 source-bans (SAST)
PASS gate:7 cargo-audit      PASS gate:14 docs (rustdoc + doc-todos + brand-scrub)
PASS gate:8 cargo-deny       PASS gate:4  rust coverage (>= 100% lines)
PASS gate:9 cargo-machete    PASS gate:5  mutation (MSI >= 100%) — 71 caught / 0 missed, 0 TIMEOUTS
PASS gate:10 gitleaks        PASS gate:6  miri     PASS gate:15 visual / AX
GATE GREEN [diff]
```

**1518 tests pass**, 5 skipped (1488 + #338's 30). Receipt `80a6571feb3648e2c7e8b99a2d1a34da9c7dc5e6`
verified against a live `gate_state_hash` — MATCH, so it is commit-valid.

**Zero timeouts**, so MSI 100 is a clean 71/71 here and #345's laundering concern does not arise — worth
stating rather than assuming, since #337's run had 3 and they needed a log read to clear.

**A rustdoc lesson worth keeping: the gate reports the FIRST error, not all of them.** Fixing the two it named
and re-running burned a full cycle to discover a THIRD (`insert_pairing_at_selections`'s public doc linking the
private `edit_ranges_restoring`). Running `cargo doc -p <crate> --no-deps` directly — seconds, not half an
hour — surfaces the whole set at once. That is now the check to run before the gate, not after it.

### Live pixel drive — DEFERRED, not skipped
chad is at the machine; synthetic input lands on the FRONTMOST window and would type into his session. The
compensation is real rather than nominal: `auto_close_pairs_through_the_live_editor_headless` drives the
**actual** platform door (gpui's `EntityInputHandler::replace_text_in_range` → `ime::replace_text` branch 1 →
the buffer) and the palette verb through `dispatch_for_test`. Its assertion is `"([)"` — the `[` inserted
literally AND landed *inside* the parens, which proves REQ-008 and REQ-001's caret placement in one string.
Re-verify the rendered pixels when the machine is free (~30s, no ticket).

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

**Docs (§21).** CHANGELOG under **Added** (the premise being wrong for 3 of 4 actions; the span
generalization; the flag that lied; the three instances; the IME fork; the `'` limit stated rather than
implied). Architecture record: `editor.md` gains an `auto_close.rs` bullet (the two orderings + why the wrap
arm must not carry the guard), the engine's post-state span + **why the fix-up must be in-engine**, the
asserted-invariant class written up as a rule for the next edit action, and the IME door's "branch 1 IS the
whole IME answer"; `crate-map.md` (the editor crate's new surface + the earned tag); `roadmap.md` M22 B-a → ✅
SHIPPED.

**Knowledge captured.**
- `AD-claude-edit-post-state-span-and-earned-undo-tags-001` — the durable constraint. Its rule is the point:
  **if your caret is not at the end of your insert, you may not claim `cursor_anchored`; if your edit range is
  not the user's cursor, your `restore` must be carried in a group.** Also records why the shim shape was
  rejected on CORRECTNESS (the group stores the selection) rather than on testability.
- `BF-claude-invariant-tag-asserted-not-earned-corrupts-redo-001` +
  `PR-claude-a-boolean-that-names-an-invariant-must-be-earned-001`.
- AAR `acd1a989…` submitted: outcome completed, effectiveness **4** — not 5. A CRITICAL text-corruption bug
  reached inspect, in an area my design had explicitly "verified".

**Follow-up filed: #346** — syntax-aware suppression (no pairing inside strings/comments) AND the `'` bound
positions, deliberately as ONE ticket: both reduce to "ask the tree what we are inside", both wait on #315,
and splitting them would build the probe twice.

### What this ticket actually taught

**The method paid a third time, including against its own author.** Phase 1 caught the spec's central bullet
being wrong for three of its four actions — and it caught my own pre-verification note (written while blocked
on #337's gate) being imprecise about the IME door and understating the wrap problem. The rule held when it
was inconvenient: my note was a claim, not a fact.

**The root cause showed up THREE times, and that is the finding.** F1 (`cursor_anchored` hardcoded), F2 (redo
at N=1 with no group to hold `sel_after`), and Validate's backspace-undo (the fallback assuming the consumed
range ends at the caret) are one shape: **something was unconditionally true when it was written, so a
downstream reader skipped its own check and documented the skip as a reason — and #338 was the first code able
to falsify it.** The generalizable move is not "be careful": it is *find the consumer and read what it SKIPS
because of your flag.* If it trusts rather than verifies, a lie costs corrupted data, and no test notices
because nothing ever made it false before.

**An asymmetric predicate verified in one direction is not verified.** My design analyzed coalescing and got
half of it right — I proved a 2-char record cannot join a PREVIOUS run (`new.inserted.chars().count() == 1`)
and wrote "verified". The guard never constrains `old.inserted`, so a FOLLOWING char joins the pair's group.
That is the whole bug, and my note sounded exactly as confident as if I had checked both directions. Fourth
instance this batch of *the decision surviving review while its reason didn't*.

**The tests found what two critics and I missed** — the backspace-undo defect came out of writing REQ-004's
round-trip, not out of review. And the negative smokes proved each critical row can fail, including the one
that matters most: under the corruption **REQ-012 still passes**, so the coalescing test would never have
caught it. A green test is evidence only after you have watched it go red.

**Two process notes worth more than they look:** run `cargo doc -p <crate> --no-deps` BEFORE the gate (the
gate reports the FIRST rustdoc error, so fixing the two it named burned a full cycle to find a third), and a
per-crate `cargo llvm-cov` UNDER-reports against the gate's workspace run (ime.rs looked 4 lines short; the
app's headless drives cover them) — reading it as the gate's number sends you chasing phantom lines.

status: Phase 5 — Complete PASS
