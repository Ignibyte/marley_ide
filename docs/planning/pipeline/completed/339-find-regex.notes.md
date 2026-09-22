# Regex find/replace (#339) — Notes

- **Forge ticket:** #339 1e6c0a52-f41b-46d5-90d7-1f98e036bd36
- **AAR:** 34cfa1c3-97cb-48a8-a60b-1e999f67b518
- **Local ticket doc:** ../../tickets/closed/TICKET-339-find-regex.md
- **Pipeline spec:** 339-find-regex.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch). -->

## Phase 1 — Plan

Promoted `queued/m22-find-regex.spec.md` → `active/339-find-regex.spec.md` (pipeline_id
`7928abd0-f636-48fc-8406-eb65ee527cc4`, AAR `34cfa1c3…`, local ticket doc written). Classification: work
pipeline, feature — **but see F8: the AC as written is bigger than one slice, and the spec hid it.**
`active/` was empty. `main` @ `a8dc510`.

### THE VERIFICATION LEDGER — **the most drift of any ticket this batch**

Run 4 of promote-don't-author, and the streak holds: an authorship error on **all four runs**. This one has
the richest set — a central decision that is false *and self-refuting*, two REQs naming controls that do not
exist, and two silent-corruption hazards the gate structurally cannot catch.

| # | Claim | Verdict |
|---|---|---|
| 1 | `find.rs` has `replace_all` (:106, back-to-front) — "regex *mode*, not replace-from-scratch" | **VERIFIED** — `replace_all(buffer, matches: &[(CharOffset,CharOffset)], repl: &str) -> usize` **TAKES** the matches; it is already mode-agnostic. But `repl` is ONE shared string → it cannot do capture-replace (the spec knows: `replace_all_with` is new). |
| 2 | **D-SAME-SHAPE** — the plumbing doesn't know the mode exists | **F1 — FALSE as stated, and the spec REFUTES ITSELF.** True for the READ path, impossible for the WRITE path. |
| 3 | `regex` + `regex-automata` already in Cargo.lock **via #326's `ignore`** — zero new crates | **F7 — the conclusion holds, the attribution is FALSE.** |
| 4 | "byte↔char at ONE seam" | **PLAUSIBLE — the converters exist** (`Buffer::char_to_byte`/`byte_to_char`, buffer.rs:108/113). Design must still name the single point. |
| 5 | The empty-match advance-by-one rule | **F3 — REAL, and worse than the spec says: it resurrects a bug the app already fixed once.** |
| 6 | `expand_captures` → per-match `replace_all_with`, one undo unit | **VERIFIED COMPATIBLE with #338** — see F9. |
| 7 | Invalid pattern → a `FindError` VALUE | **`FindError` is NEW** — zero hits repo-wide. Not a drift; the spec's phrasing implies it exists. |
| 8 | "the case chip maps to `(?i)`" · F3 = find-next | **F4 — NEITHER CONTROL EXISTS.** |
| 9 | Literal mode byte-identical with the chip off | **STANDS, and #338 proved how to make it provable** (a property test over randomized input, not an argument). |

---

**F1 [BLOCKING — D-SAME-SHAPE is false, and the spec contains its own refutation].**
The READ path really is mode-blind, and I checked rather than assumed: an exhaustive sweep for a match end
recomputed from the query's length (`+ needle/query.len()/chars().count()`) returns **zero** hits. Bands
(app.rs:4588-4595 — `partition_point` then `let (ms, me) = efind[mi]`), the n-of-m counter (index/len only),
`select_efind_current`, and ⌘D all use the returned `end` verbatim. Varying-length matches carry. The find bar
even stores `Rc<Vec<(usize, usize)>>` (app.rs:431) with the newtype stripped at exactly ONE `map`
(app.rs:10934) — a real choke point.

**But the WRITE path cannot be mode-blind, by construction.** `find_all_regex(...) -> Result<Vec<(CharOffset,
CharOffset)>, FindError>` **discards the captures**, and `expand_captures(match, captures, template)` needs
them. **The shape only carries BECAUSE it throws away exactly what capture-replace requires.** And the spec's
own D-PER-MATCH-REPLACE introduces `replace_all_with(buffer, &[(range, String)])` — *that is the replace
plumbing D-SAME-SHAPE claims is untouched*.
→ **Design must decide, and the spec never asks:** does the return type grow to carry captures, or does the
replace path re-run the regex (which violates D-BYTE-CHAR-AT-ONE-SEAM by adding a second match site)?
D-SAME-SHAPE should be **narrowed to the read path** and stated honestly, not deleted — the read half is a
real and valuable result.

**F2 [HIGH — silent corruption the gate cannot catch].** Replace-One recomputes from the **template's**
length, not the inserted text's:
```rust
app.rs:2079  Selection::caret(CharOffset::from(s + repl.chars().count()))
app.rs:2084  self.efind_resume = Some(s + repl.chars().count());
```
With `(\w+)@(\w+)` → `${2}_${1}` over `ab@cd` (0..5): the inserted text is `cd_ab` (5 chars) but
`repl.chars().count()` is **9**. The caret lands at 9 — past the replacement, inside following text (clamped
by `set_selection`, so no panic, just wrong) — and **`efind_resume = 9` silently skips every match starting in
5..9**. Both lines sit under `#[cfg_attr(test, mutants::skip)]` (app.rs:2021) in a **coverage-excluded** file,
so neither coverage nor mutation can see it. This is the #336 class exactly: real offset math hiding where no
gate looks.

**F3 [HIGH — an empty match resurrects a bug the app already fixed].** `find_all` can NEVER return an empty
match (find.rs:78 early-returns on an empty needle); regex can (`a*`, `^`, `\b`). Two consequences:
- **The F5 loop, resurrected.** Replace-One on an empty match with an empty replacement sets
  `efind_resume = s + 0 = s`; the next `partition_point(|&(ms,_)| ms < s)` (app.rs:10939) lands on the **same
  match**. That is precisely the infinite loop the resume mechanism was built to prevent.
- **Counted but invisible.** `row_selection_cols` returns `None` when `s >= e` (code_view.rs:259) and
  `styled_slices_with_marks` skips `if r.end > r.start` (code_view.rs:387) → the bar reads "3 of 7" while
  only 4 bands are painted.
→ The spec's "advance-by-one rule" covers the *finding* loop. It does not cover the *replace* loop or the
counter/paint disagreement. Both need a decision.

**F4 [BLOCKING for the AC — two REQs describe controls that do not exist].**
- **There is no case chip.** `fold: true` is **hardcoded** at app.rs:10932. REQ-002 ("map the case chip to
  `(?i)`") is **build a chip** — new state + render + binding.
- **There is no F3.** Zero `"f3"` bindings in `crates/`; find-next is **Enter / ⇧Enter** in `handle_efind_key`
  (app.rs:2087-2092). REQ-003/REQ-007 name a control that was never built.
→ REQ-002/003/007 are AMENDED in the spec. This is the #337 shape again: the spec describes an app adjacent
to the one that exists.

**F5 [HIGH — the two modes will disagree, and the spec never names the decision].** `fold` does **not**
lowercase the haystack; it compares per char (find.rs:81-86):
`c == ndl[k] || (fold && c.eq_ignore_ascii_case(&ndl[k]))` — **ASCII-only and length-preserving by
construction**, and find.rs:165 pins it (`find_all("straße", true)` is ASCII-only). The regex crate's `(?i)`
is **Unicode simple case folding**: `(?i)k` matches U+212A KELVIN SIGN. **Same query, same chip, different
match counts.** Matching today's behavior needs `RegexBuilder::unicode(false)` — which also changes `.` and
`\w` semantics, i.e. it is a product decision, not a flag.

**F6 [MEDIUM — regex injection through ⌘F].** app.rs:6361 stuffs raw selected text into `efind_query`. Select
`foo(bar)` and press ⌘F → a *valid* regex meaning something else. Select `a[` → the bar opens **already in
the error state**. Needs `regex::escape` on reseed, or reseed forces literal mode.

**F7 [the dep claim — conclusion right, reasoning wrong].** Verified with `cargo tree -i regex`:
- **`regex` v1.12.4** comes from **`gpui_util` → gpui** and **`tree-sitter` → marley_syntax**.
- **`ignore`** (a real direct dep of `marley_project`) pulls `globset` → **`regex-automata` + `regex-syntax`**
  — but **NOT `regex` itself**.
So "already in Cargo.lock" is TRUE (no new audit/deny surface) but "via #326's `ignore` adoption" is FALSE for
`regex`. It matters: **`crates/editor` depends on none of gpui, tree-sitter, or ignore** — its Cargo.toml is
deliberately `ropey` + `marley_text_offsets` and nothing else (the comment says so). It would gain its **third
dependency**, in the workspace's purest crate. That is a decision to take, not a footnote.

**F8 [scope — the AC is bigger than one slice, and the spec hid it].** Between F4 (build a case chip; build
F3 or re-scope to Enter/⇧Enter), F1 (the write path needs a captures-carrying return or a second match site),
F2 and F3 (two silent-corruption fixes in a coverage-excluded shim), this is not "a chip on the existing
bar". Design should say plainly whether it is one slice or two (candidate split: regex FIND + the chip, then
capture REPLACE).

### Verified clean (checked, not assumed)
- **F9 — the #338 interaction is CLEAN.** `Buffer::begin_undo_group` still passes `cursor_anchored: false`
  (buffer.rs:683), so `replace_all`'s existing group (find.rs:106-118 — `begin_undo_group(sel)` … back-to-front
  … `end_undo_group(sel)`) already satisfies `AD-claude-edit-post-state-span-and-earned-undo-tags-001`'s rule
  ("an action whose edit range is not the user's cursor must carry `restore` in a group"). It does, and it is
  not cursor-anchored. No interaction. Recorded because I looked, not because I assumed.
- **Nothing assumes equal match lengths** (the hypothesis I most expected to fail) — zero hits for a
  query-length-derived end.
- **⌘D is safe** — `needle_of` (multi_cursor.rs:150-155) takes literal `text_in_range` and `occurrences`
  hardcodes `fold: false`. ⌘D is a PEER consumer of `find_all`, not downstream of the bar, so the mode cannot
  reach it.
- The byte↔char converters exist (`Buffer::char_to_byte`/`byte_to_char`, buffer.rs:108/113).
- **The bands + `replace_all` rely on ORDERING + NON-OVERLAP** — a contract the TYPE does not carry
  (`partition_point`, and the back-to-front sweep's "must be ascending + non-overlapping"). Regex output is
  ascending and non-overlapping by construction, but the design should say so rather than inherit it silently.

### Forks for Design
- **Fork A (F1):** the captures problem — grow the return type vs re-run the regex at replace. Narrow
  D-SAME-SHAPE to the read path and say so.
- **Fork B (F5):** `unicode(false)` to match today's ASCII fold, or accept that the two modes count
  differently. A product call.
- **Fork C (F8):** one slice or two.
- **Fork D (F7):** `crates/editor` takes its third dep, or the regex search lives elsewhere.

### Standing context
**§20 CONFIRMED** — regex find/replace + `$1` capture syntax are universal editor affordances (the `regex`
crate's own `Replacer` syntax is the de-facto standard); no copyleft source read.
**LIVE synthetic-input drives are OFF-LIMITS** (chad is at the machine) → units + headless + mechanism.
**Push remains UN-OK'd** (all commits LOCAL). Batch lessons in
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md) — most relevant here: **a boolean/claim that names
an invariant must be EARNED**, and **F2/F3 live in a coverage-excluded, mutants::skip shim — the exact place
#336's bug hid.**

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

Four forks, all settled with evidence. **One spec decision DISSOLVED entirely** (the crate already does it,
better), and the ticket is **SPLIT**.

### FORK C — SPLIT. #339 = regex FIND + the chips. Capture-REPLACE is its own ticket.

**#339 (this ticket):** the `.*` mode chip, the case chip (new — F4), `find_all_regex`, the byte↔char seam,
the invalid-pattern error state, reseed-escaping.
**DEFERRED to a follow-up:** capture-group replace — `expand_captures`, `replace_all_with`, and the two
silent-corruption fixes (F2's template-length caret/resume, F3's replace-side empty-match loop).

The evidence for splitting rather than pushing through:
- **F1 is a design problem, not a task.** `find_all_regex -> Vec<(CharOffset, CharOffset)>` discards the
  captures `expand_captures` needs, so capture-replace forces a real choice (grow the return type vs re-run
  the regex at replace, which breaks D-BYTE-CHAR-AT-ONE-SEAM). That deserves its own design + critics, not the
  tail end of a ticket whose first half is already sizeable.
- **F2/F3-replace are fixes in a `mutants::skip` shim inside a coverage-excluded file** (app.rs:2079/2084).
  Neither coverage nor mutation can see them — **the exact place #336's `chars().count()` bug hid**. They need
  their own inspect pass with critics pointed at them, not a footnote.
- **The find half stands alone and is provable.** Searching `fn \w+_test` is useful on its own, the read path
  is verified mode-blind (F1's true half), and OFF-identity is a property test.
- The batch's pattern: every ticket shipped with a NAMED cut and a filed follow-up (#341, #344, #346). This
  is the same discipline, one size up.
**REQ-005/010/011 move to the follow-up. They are NOT dropped** — the spec records the cut and the ticket
carries the F1/F2/F3 findings already written.

### FORK B — RESOLVED: regex mode uses FULL UNICODE `(?i)`. The divergence is real, deliberate, and pinned.

The disagreement is genuine: `fold` compares per-char with `eq_ignore_ascii_case` (find.rs:81-86) — ASCII-only
and length-preserving **by construction** (that assumption is *why* the literal path can do `i += ndl.len()`),
pinned at find.rs:165. `(?i)` is Unicode simple case folding: `(?i)k` matches U+212A KELVIN SIGN.

**Rejected: `RegexBuilder::unicode(false)`** — it would match today's counts, but it also makes `\w`, `\d`,
`\s` and `\b` ASCII-only. **A regex mode whose `\w+` matches `caf` and not `café` is a broken regex mode** —
Unicode-aware classes are most of why anyone turns regex on. Crippling the feature to preserve an ASCII-fold
quirk optimizes for an invariant nobody asked for; find.rs:165 pins that quirk as a *known limit*, not a
contract.

**Decision:** the chip means "case-insensitive" in both modes; the *definition* is the mode's own, and regex
mode's is the more correct one (it finds `KELVIN` for `k`, and `straße`/`STRASSE` per Unicode). A user who
turns on `.*` is opting into regex semantics. **Pinned by a test that names the disagreement** (the same
posture as #337's `inf → 13` / `1e30 → 32` asymmetry: record it as a decision so the next reader meets it
deliberately, not as a surprise).

### FORK D — RESOLVED: `crates/editor` declares `regex`. It is its third dep and that is correct.

The crate's identity is **gpui-free and IO-free** (lib.rs: "no UI, no OS or IO"), not "two dependencies".
`regex` is pure computation — it violates neither clause, and crate-map's 🟢 PURE stays true. The alternative
— putting `find_all_regex` in another crate — would **split the find seam across two crates**, which is
strictly worse: two find modules, two homes, and a cross-crate call for one function. `regex` is already
in-lock (via gpui + tree-sitter, **not** `ignore` — F7), so there is no new audit/deny surface; only a
Cargo.toml line. The comment there will say why.

### D-EMPTY-ADVANCE — **DISSOLVED. The crate already does it, and better.**

The spec proposed hand-rolling "empty matches advance one char instead of looping; the table row is the
guard". `regex::Regex::find_iter` **already implements it** — regex-automata 0.4.13 `util/iter.rs:30-36`
states the problem and the fix verbatim: *"if an empty match is found… iteration would never end. Instead, a
`Searcher` knows how to detect these cases and forcefully advance iteration in the case of an empty match that
overlaps with a previous match."* Note it is **more careful than the spec's rule** — it advances on an empty
match that *overlaps a previous match*, rather than blindly on every empty match, which is what keeps `^`
matching every line start rather than every other one.

→ **A spec decision dying because the substrate already does it is a WIN to record, not a deviation to hide**
(the #336 precedent, and the same lesson: reading a permissive dep's source settles the fork and kills my own
hand-rolled proposal). `find_iter` IS the implementation; the test row survives as a PIN on the crate's
behavior at our seam (`a*`, `^`, `\b` → finite + ordered), not as a guard on our own loop.

### The haystack + the byte↔char seam — the design's real mechanics

**`find_all` is rope-native and alloc-free**: it walks `rope.char(i + k)` in char space and never touches
bytes. **`regex` needs a contiguous `&str`.** So `find_all_regex(text: &str, …)` takes the materialized
document and the caller does `buffer.text()` (= `rope.to_string()`, buffer.rs:96) — **a full-document
allocation, per query keystroke** (the bar re-searches when `efind_key = (file, version, query)` changes).

That cost is accepted and named: the search is already O(n) per keystroke, so this is the same complexity with
a bigger constant — **and only the REGEX path pays it.** The literal path stays rope-native, so OFF is
byte-identical *and* cost-identical. If it ever matters, the exit is `rope.chunks()` + a streaming engine, and
that is a follow-up with a measurement behind it, not a guess now.

**The byte→char conversion is ONE LINEAR PASS, not M rope walks.** `Buffer::byte_to_char` is an O(log n) rope
walk (buffer.rs:113), so mapping M matches × 2 ends that way is 2M walks. Instead: the regex yields **ascending
byte offsets** over a haystack we already hold as a `&str`, so a single walk of `text.char_indices()`, zipped
against the sorted boundaries, maps them all in O(n + M) with no rope involvement. **That is
D-BYTE-CHAR-AT-ONE-SEAM, honestly implemented** — one pass, one place, no per-match cost.

### The empty-match paint — kept, with a documented cut

An empty match is **counted by n-of-m but paints nothing** (`row_selection_cols` → `None` when `s >= e`,
code_view.rs:259; `styled_slices_with_marks` skips `r.end > r.start`, code_view.rs:387) → the bar can read
"1 of 40" over zero bands for `^`.

**Rejected: suppress empty matches from the list** — that would make `^`, `$` and `\b` report **0 matches**,
which is a worse lie than an unpainted band and breaks the most plausible reason to type them.
**Decision: keep them.** The count stays honest, and navigation still *works* — Enter moves the caret to each,
and the caret is visible even where a band is not. The zero-width band (a thin caret-like marker, VS Code's
shape) is **render work in a coverage-excluded shim** and belongs with its own design pass. Documented + filed.

### §20 — CONFIRMED
The `regex` crate is **published-API reuse** (MIT/Apache, already a transitive dep) — and reading its source
to settle D-EMPTY-ADVANCE is **adoption, outside the §20 wall** (the #336 gpui precedent). VS Code = OBSERVED
bar behavior only (the `.*` chip, the inline invalid-pattern error). `$n` capture syntax is the regex crate's
own `Replacer` convention. No copyleft source read.

### File manifest

| File | Change |
|---|---|
| `crates/editor/Cargo.toml` | **+`regex = "1"`** (Fork D) — with a comment: already in-lock via gpui/tree-sitter; pure computation, so the crate stays gpui-free/IO-free. |
| `crates/editor/src/find.rs` | **+`FindError`** (NEW — zero hits today; a VALUE, no `unwrap` on the compile path) · **+`find_all_regex(text: &str, pattern: &str, case_insensitive: bool) -> Result<Vec<(CharOffset, CharOffset)>, FindError>`** — `RegexBuilder` + `size_limit` (REQ-008), `(?i)` via `.case_insensitive(true)`, `find_iter` (which owns the empty-match rule), then the ONE `char_indices` pass. `find_all` **untouched**. |
| `crates/editor/src/lib.rs` | re-export `find_all_regex`, `FindError`. |
| `crates/marley_app/src/app.rs` | the bar's `efind_regex: bool` + `efind_fold: bool` state (**both chips are new** — `fold: true` is hardcoded at :10932); `refresh_efind_matches` (:10932) branches on the mode and holds `FindError` for the render; the ⌘F reseed (:6361) escapes (F6/REQ-012). Shim. |
| `crates/marley_app/src/` (render) | the two chips + the inline error row on the find bar. Shim. |
| `crates/marley_app/src/headless_drive.rs` | the drives. |

### Regression Test Plan

| REQ | Test | Where | |
|---|---|---|---|
| REQ-001 | `find_all_regex("fo foo fooo", "fo+")` → 3 correct CHAR ranges; **+ a multibyte row**: a match after an emoji lands on the right CHARS (the byte→char pass is the whole risk) | find.rs | pure |
| REQ-002 | the case chip → `.case_insensitive(true)`; **+ THE DIVERGENCE ROW**: `(?i)k` matches U+212A while `find_all("k", true)` does not — named in a comment as Fork B's decision | find.rs | pure |
| REQ-003 | an invalid pattern (`a[`) → `Err(FindError)`, no panic; the bar shows it, find-next no-ops | find.rs + headless | |
| REQ-004 | `a*`, `^`, `\b` → finite + ordered. **This PINS THE CRATE's guarantee at our seam, not our own loop** (D-EMPTY-ADVANCE dissolved) | find.rs | pure |
| REQ-006 | **the OFF-identity, as a PROPERTY over randomized input** (#338's shape): with the chip off, `find_all` is called and its result is byte-identical to the pre-ticket path. `find_all`'s body is untouched — the property guards the ROUTING | find.rs + app | property |
| REQ-007 | regex matches feed the UNCHANGED bands / n-of-m / find-next (**Enter/⇧Enter, not F3** — F4) | headless | |
| REQ-008 | an over-limit pattern (`a{1000}{1000}{1000}`) → `Err(FindError)`, not an OOM | find.rs | pure |
| REQ-009 | `Cargo.lock` diff is EMPTY (regex was already in-lock); `crates/editor/Cargo.toml` gains exactly one line | gate + review | |
| REQ-012 | ⌘F reseed escapes: selecting `foo(bar)` finds `foo(bar)` literally; `a[` does not open the bar in an error state | headless | |
| — | empty matches are COUNTED but paint no band — the documented cut, pinned so it is a decision | find.rs/headless | |

**What a green test would NOT prove — the batch's recurring lesson, applied up front:**
- **REQ-006's OFF-identity is about ROUTING, not `find_all`.** `find_all`'s body is untouched, so a test of
  `find_all` proves nothing about the chip. The property must assert *the mode-off path calls the literal
  search and returns its exact result* — i.e. it has to go through the branch, or it is testing the wrong fn.
- **REQ-001's ranges can be right in BYTES and wrong in CHARS** and most fixtures would never notice: on pure
  ASCII the two coincide *exactly* — the same coincidence that let #336's `chars().count()` bug ship. The
  multibyte row is not a nice-to-have; it is the only row that tests the seam.
- **REQ-002's divergence row is the decision.** Without it, `unicode(false)` could be added later "to be
  consistent" and every other test would still pass while `\w` silently stopped matching `café`.

**Uncoverable / deferred:** LIVE pixel drives are **OFF-LIMITS** — chad is at the machine and synthetic input
lands on his frontmost window. Units + headless + mechanism, documented as deferred-not-skipped (the #336/#337/
#338 posture). The two chips' *rendering* is shim work verified by mechanism + a headless state assert.

### Risks

1. **The full-document alloc per query keystroke** (`buffer.text()`). Same O(n) as the existing search, bigger
   constant, regex-path only. *Mitigation:* named, not hidden; the literal path is untouched so OFF pays
   nothing; the exit (`rope.chunks()` streaming) is a measured follow-up, not a guess.
2. **The byte→char pass is the ticket's only real offset math**, and ASCII fixtures cannot see a mistake in
   it. *Mitigation:* the multibyte row + it lives in the PURE crate (cov/MSI 100), not a shim.
3. **`crates/editor` gains a dep** — reversible, and the crate's actual contract (gpui-free, IO-free) is
   preserved.
4. **The split leaves Replace literal-only**, so the bar will have a `.*` chip whose Replace does not expand
   `$1`. *Mitigation:* that is the follow-up's whole content, and the chip does not *claim* capture support;
   Replace keeps working literally. State it in the CHANGELOG so it reads as a scope, not a bug.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement (PASS 1 of 2 — the pure seam; the app's chips + mode branch are pass 2)

### Built (pass 1)

1. **`crates/editor/Cargo.toml`** — `regex = "1"` (Fork D), with the comment stating what was verified: in-lock
   via gpui + tree-sitter (**not** `ignore` — F7), pure computation, so the crate stays gpui-free / IO-free
   and its 🟢 PURE identity holds. In-lock version confirmed 1.12.4.
2. **`crates/editor/src/find.rs`** — additive only; **`find_all` is byte-for-byte untouched**.
   - **`FindError`** — `InvalidPattern(String)` + `TooComplex`, with `Display` + `std::error::Error` (§14's
     typed-error rule). **The two variants are not invented**: `regex::Error` has exactly two shapes
     (`Syntax(String)`, `CompiledTooBig(usize)` — error.rs:8-31, read before writing), so the split mirrors
     the crate rather than guessing, and it lets the bar distinguish "invalid pattern" from "too complex" —
     different user mistakes.
   - **`find_all_regex(text, pattern, case_insensitive)`** — `RegexBuilder` + `size_limit(1 MiB)` + full
     Unicode (**no `.unicode(false)`** — Fork B) + `find_iter`. An empty pattern → `Ok(vec![])`, matching
     `find_all`'s empty-needle rule so the bar's "empty list = no query" invariant survives.
   - The module doc's "**`find_all` is the ONE match engine**" was **narrowed to "the ONE LITERAL match
     engine"** and `find_all_regex` documented as a *peer, not a replacement* — with the note that ⌘D/⌘⇧L
     deliberately never reach it (they are literal by construction). Leaving that header as-is would have made
     it false the moment this landed, which is the #337 F5 class.
3. **`crates/editor/src/lib.rs`** — re-export `find_all_regex`, `FindError`.

### Deviation: the byte→char map became a MERGE WALK (better than designed)

The design said "one linear pass … building `byte -> char`". I wrote that first — a `HashMap<usize, usize>`
over `char_indices()` plus a `text.len()` sentinel — and then **rejected my own code before it left the
keyboard**: it resolves each boundary with `map[&b]`, which **panics on a missing key**. I could prove the key
always exists (a regex boundary always lands on a char boundary of the `&str` it searched), but "it cannot
panic, trust me" is an unwrap-equivalent on an input path (§14) — and an invariant nothing enforces is exactly
what this batch keeps punishing.

**The replacement is strictly better and simpler.** `find_iter`'s boundaries are ascending and non-overlapping,
so the flat sequence `(start_0, end_0, start_1, end_1, …)` is **non-decreasing**. One monotone cursor over
`char_indices()` therefore resolves every boundary: O(n + m), **no allocation, and no fallible lookup at all**.
A target past the last char (a match ending at EOF) walks the cursor to `None` and returns the total char
count — the sentinel falls out naturally instead of being a special case.

### Verified, not assumed (a throwaway probe, run then DELETED — `grep probe_regex_offsets` → 0)

All eight behaviors confirmed against the real crate before Phase 4 formalizes them:

| probe | result |
|---|---|
| ASCII `fo+` over `"fo foo fooo"` | 3 correct ranges |
| **multibyte** — `b+` over `"a😀bb"` | **chars (2,4)**, not bytes (5,7) — the byte→char walk is right |
| **EOF sentinel** — `é` over `"xé"` | (1,2) — a match ending at EOF resolves |
| empty pattern | `Ok(vec![])` — matches `find_all` |
| `a*` over `"aaa"` | 1 match, finite — `find_iter` owns the advance |
| **Fork B's divergence** — `(?i)` `k` over U+212A | 1 match (KELVIN SIGN) — the Unicode fold is real |
| `a[` | `Err(InvalidPattern(_))` |
| `a{1000}{1000}{1000}` | `Err(TooComplex)` — **the size_limit bites**, no OOM |

**Verification:** `cargo build -p marley_editor` clean (the dep resolves); `cargo clippy -p marley_editor
--all-targets` CLEAN; `cargo nextest run -p marley_editor --lib` → **194 passed** (`find_all`'s own tests are
untouched — they are the OFF-path proof); `cargo fmt --all`; §20: zero reference-app names in the diff.

### Built (pass 2) — the app shim

4. **`crates/editor/src/find.rs`** — `pub escape_literal(text) -> String`. It wraps `regex::escape` rather
   than re-exporting it so **`regex` stays confined to the editor crate**: marley_app asks the editor to turn
   text into a pattern (the editor's vocabulary) and never grows the dependency. Fork D bought one Cargo.toml
   line, not two.
5. **`crates/marley_app/src/app.rs`** — `efind_regex` (default **false** — literal is the pre-#339 path),
   `efind_fold` (default **true** — the bar has always folded, so the chip preserves shipped behavior),
   `efind_error: Option<FindError>`. The mode fork in `refresh_efind_matches`; an `Err` yields **no matches**
   (so navigation no-ops and the bands clear rather than stranding the previous pattern's highlights under an
   error). `toggle_find_regex` / `toggle_find_case` drop `efind_key` (see below) and the regex toggle clears
   the error — a stale "invalid pattern" must not survive the switch back to literal, where nothing can be
   invalid. The ⌘F reseed escapes via `escape_literal` (F6/REQ-012).
6. **`crates/marley_app/src/palette.rs` + app.rs** — `CommandId(20)`/`(21)` → "Toggle Find: Regex Mode" /
   "Toggle Find: Match Case", palette-only: **the bar owns printable keys**, so a chip chord would type into
   the query. The #330/#331/#338 toggle precedent exactly.

### A bug caught while wiring, worth more than the wiring: THE MEMO KEY

`efind_key` was `(nonce, version, query)` — the #272 memo. **A chip toggle would have been silently inert
until the next keystroke**: the key would still match, `refresh_efind_matches` would early-return, and the bar
would show the PREVIOUS mode's matches. The user toggles `.*`, nothing happens, they type a char, and it
suddenly works.

The key now carries both flags: `(nonce, version, query, regex, fold)`. **The chips change what a match IS,
so they are part of the query's identity** — that is the honest framing, not "add the flags to the cache".
The toggles also null the key, so the effect lands on the next refresh rather than waiting for an edit.

This is the same shape as the batch's recurring lesson: a memo whose key omits an input it depends on is an
invariant nothing enforces. Nothing would have failed — it would just have been wrong, quietly, exactly like
#338's tag.

### Deviations
- **`escape_literal` is a new pub fn the design did not list.** The design said the reseed escapes; it did not
  say *where*. `regex::escape` in app.rs would have forced `regex` into marley_app's Cargo.toml — a second
  crate taking the dep to call one function. Wrapping it in the editor keeps Fork D's answer ("the editor owns
  the find seam") true rather than nominal.
- **The test hooks were written and DELETED before clippy finished** — `efind_mode_for_test` /
  `efind_error_for_test` were dead (`-D warnings`). **Third instance this batch** (#337's `font_size_for_test`,
  #338's `auto_close_on_for_test`). The rule is now reflex: Phase 4 re-adds a hook when a test consumes it.

### Verification (pass 2)
`cargo check --workspace` clean; **`cargo nextest run --workspace` → 1518 passed, 5 skipped** (unchanged —
`find_all`'s own tests are the OFF-path proof and none moved); `cargo nextest run -p marley --lib -E
'test(/palette|command/)'` → **20 passed**, including `every_cockpit_command_resolves_to_a_verb` (both new
rows resolve); `cargo clippy --workspace --all-targets` **CLEAN**; `cargo fmt --all`. Diff: 5 files, +252/−19.

### Still open for Validate
Every test. The pure seam's probe was run and DELETED (`grep probe_regex_offsets` → 0), so Phase 4 writes the
real rows — **the multibyte one is not optional**: on ASCII, bytes == chars exactly, which is the coincidence
that shipped #336's bug. Also: the memo-key row (toggle a chip → the matches change WITHOUT a keystroke) is
the one that would have caught the bug above, and REQ-006's OFF-identity is about the ROUTING, not `find_all`.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

2 parallel critics (the byte→char offset seam + the crate contract; the app wiring + the memo). Both verified
empirically — a 750-combo differential over the offset seam, the monotonicity chain traced to
regex-automata's source, a minimal rustc repro of the write-only-field blind spot, 60+66 mutant lists — and
both left the tree byte-identical (md5-checked).

**They converged on the SAME HIGH from opposite ends, and it is mine: I dropped the render half of my own
design and wrote "Phase 3 PASS" over the gap.** The offset seam — the thing I flagged as the whole risk —
came back *proven sound*. The defect was the shim.

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | **HIGH** | **`efind_error` is WRITE-ONLY — the bar renders a lie.** 4 writes, 0 reads. With regex on + `a[`, the bar shows "a[ **(no matches)**" — the pattern never COMPILED, but the user is told nothing is there and hunts the document. My Phase-2 manifest listed "the two chips + the inline error row"; I built the state + logic + palette and declared PASS **without the render and without recording the cut**. | **REAL — both critics, independently** | Built the inline error row (danger, `FindError::Display`) that overrides the `(no matches)` label; `efind_error` now has readers. |
| F2 | **HIGH/MED** | **The two chips have no render — the mode is invisible.** Same root cause. The palette toggle changes what a match IS with zero visual feedback; `foo(bar)` silently means something different. REQ-002 was literally "BUILD a case chip". | **REAL** | Added a `.* `/`Aa ` mode prefix to the find label — `.*` = regex on, `Aa` = case-sensitive; the shipped literal+fold default shows nothing, so the bar is unchanged for existing behavior. |
| F3 | **LOW→fixed-in-scope** | **An empty regex match + empty replacement loops Enter forever** — `efind_resume = s + 0 = s`, next `partition_point` lands on the same zero-width match. The design DEFERRED "the replace-side empty-match loop" to #347, but the critic is right that **#339 is what makes it reachable** (regex matches empty; `find_all` never did), so the fix belongs here. | **REAL — scope adjusted INTO #339** | A no-op-edit guard: `s == e && repl.is_empty()` → resume `s + 1` (exactly how `find_iter` escapes an empty match). #347 keeps the capture-length half of F2/F3. |
| F4 | **MEDIUM** | **The merge walk swapped a LOUD unenforced invariant for a SILENT one.** The rejected `map[&b]` design panicked on a bad key; the merge walk's `find_iter`-monotonicity assumption, if it ever broke, returns a stale index — wrong highlights, no panic. | **REAL (defensive)** | A `debug_assert!(target >= last_target)` — restores the loud failure in debug/test at zero release cost. The invariant holds today (traced to source + 750-combo proof); this catches a future regression as a crash, not a misread. |
| F5 | LOW | **The memo-key comment inverts the causality.** It said the key "has to" carry the chips or a toggle would be inert — but both toggles null the key, so the described bug is unreachable today with or without the flags. A reader could "remove the flags since we null it anyway", reintroducing the hazard for a future non-toggle writer. | **REAL — my REASON was wrong (5th time this batch)** | Rewrote: the flags belong in the key because they are INPUTS to the search (identify the computation), and the key-null is belt-and-braces that also protects a future non-toggle writer. |
| F6 | LOW | **The toggle-error-clear rationale is FALSE.** `toggle_find_regex` cleared `efind_error`; its doc said "only the regex chip can affect the error". But `case_insensitive(true)` expands char classes and can cross `REGEX_SIZE_LIMIT`, so the CASE chip creates/clears `FindError`s too. | **REAL — my REASON was wrong** | Dropped BOTH toggles' error-clears (the unconditional reassign in `refresh_efind_matches` already resets it every refresh) and corrected the docs: `TooComplex` is case-dependent. |
| F7 | LOW | **`efind_error` not cleared on the bar-close path** — one of the three derived-match fields was reset, the error survived. | **REAL (latent)** | Cleared it alongside `efind_key`/`efind_matches` in the close branch. Inert today (a refresh always precedes a read), but the field now HAS a reader, so the reset must be complete. |

**Rejected / verified clean (each cost the critics real effort):**
- **The offset seam is PROVEN sound, twice.** (a) Monotonicity is TRUE for every pattern — traced
  `find_iter` → `Matches` → `meta::FindMatches::next` → `Searcher::try_advance` (regex-automata 0.4.13
  util/iter.rs:422-441): the next search starts at `end_i`, and `handle_overlapping_empty_match` only bumps
  `start` FORWARD, so the boundary sequence is non-decreasing by construction; plus a 750-combo differential
  (`a*`/`^`/`$`/`\b`/`\B`/`(?m)^`/`(a|)`/`(?:)`/… over multibyte text) with zero regressions. (b) `char_at`
  matches an independent `text[..b].chars().count()` oracle on all 750; every edge (empty text, target 0,
  target==len, EOF match, shared boundary, empty-match double-call) correct. (c) **Evaluation order is
  GUARANTEED, not folklore** — the Rust Reference `[expr.operand-order]` lists "Tuple expression" explicitly
  and its worked example is two `next()` calls in a tuple, the identical shape; concern REJECTED.
- **The full-document alloc is genuinely memoized** — the early-return (app.rs:11002) is strictly BEFORE the
  `text()` call (11011), so a memo hit never materializes the document; no per-frame alloc. The per-keystroke
  alloc is Design's named Risk #1, accepted, regex-path only.
- **The ⌘F reseed escape is CORRECT and matches VS Code exactly** — it escapes only in regex mode; VS Code's
  `findController.ts` does the same (`isRegex ? escape(sel) : sel`), and re-escaping on toggle WOULD be the
  bug (type `\d+`, toggle on, get `\\d\+`). REJECTED as a concern.
- **The memo key is COMPLETE** — `active_nonce()` is monotonic-per-process and re-minted on reload (pinned by
  `nonces_are_unique_per_open_file_lifetime`), so no two files collide; the search's full input set (text via
  nonce+version, query, fold, the regex arm-selector) is all in the key. `efind_resume` is correctly OUTSIDE
  it (selects which match, not what a match is).
- **The error path's BEHAVIOR is right** (only the display was missing): matches emptied → bands clear,
  Enter/⇧Enter no-op (`match_navigation(0,…)` → 0, `select_efind_current` early-returns), no underflow.
- **`REGEX_SIZE_LIMIT` proven both ways** — rejects `a{1000}{1000}{1000}`, accepts 13 ordinary patterns
  (`\w+`, `(?i)foo`, `[a-zA-Z0-9_]{1,64}`, a 200-char literal, …). Not too tight.
- **§14 clean** — no `unwrap`/`expect` on an input path; `FindError` has `Display` + `Error`; the
  `regex::Error` match's `other =>` catch-all is REQUIRED (`#[non_exhaustive]`, verified). No ReDoS (finite
  automaton). Deep nesting → typed `Syntax` error, not a stack overflow.
- **Palette + mutation discipline clean** — all four wiring halves line up, `every_cockpit_command_resolves_to_a_verb`
  passes (20 tests); find.rs has 60 mutants incl. `find_all_regex`/`escape_literal`/`FindError`; app.rs's new
  shims yield 0 (skipped); **detach trap CLEAR** (234→236, exactly the 2 new skipped fns, each attribute above
  its own fn).

**The two lessons (captured):**
`BF-claude-dropped-my-own-render-half-and-passed-the-phase-001` +
`PR-claude-diff-phase-3-against-your-own-phase-2-manifest-001` — **before writing "Implement PASS", DIFF what
you built against the Phase-2 manifest you wrote, item by item.** "PASS" is a checklist result, not a feeling.
A UI ticket's corollary: **state without render is DEAD STATE** — grep every new field for a READER, and a
field with writes and zero reads is invisible to clippy + coverage + mutation at once (rustc does not warn on
a struct-literal-initialized private field that is only assigned — both critics repro'd it). The manifest
diff is the only check that knows what you INTENDED.

And F5/F6 make it **five findings this batch where the code was right and the REASON was wrong**. The pattern
is stable enough to name flatly: a plausible comment terminates review. Measure the reason, not just the fix.

**Verification after fixes:** `cargo clippy --workspace --all-targets` CLEAN; **`cargo nextest run
--workspace` → 1518 passed, 5 skipped**; `cargo fmt --all`; §20 clean; `efind_error` now has readers
(`grep -n "self.efind_error" | grep -v "= "` → the label guard + the render row). **The render change is
DRIVEN-VERIFICATION OWED at Phase 4** — deferred here (chad at the machine; synthetic input off-limits), to be
proven by the headless render assert + mechanism, not assumed.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**A negative smoke caught one of MY OWN tests not proving what it claimed** — the batch's recurring lesson,
this time landing on the validation itself.

### Tests added

| Where | Test | Pins |
|---|---|---|
| find.rs | `t339_regex_basic` | REQ-001 — ascending, non-overlapping ranges |
| find.rs | **`t339_regex_offsets_are_chars_not_bytes`** | REQ-001 — **the byte→char seam** (`b+` over `a😀bb` → chars (2,4) not bytes (5,7); EOF sentinel; `(?m)^` char offsets) |
| find.rs | `t339_regex_case_insensitive` | REQ-002 — the `(?i)` chip |
| find.rs | **`t339_regex_unicode_fold_diverges_from_literal_ascii_fold`** | REQ-002 — **Fork B pinned as a decision**: `(?i)k` matches KELVIN, `find_all` does not |
| find.rs | `t339_invalid_pattern_is_a_value` | REQ-003 — `a[`/`(un`/`\` → `Err`, no panic |
| find.rs | `t339_empty_matching_patterns_stay_finite` | REQ-004 — `a*`/`^`/`\b` finite + ordered (pins the CRATE's guarantee) |
| find.rs | `t339_size_limit_rejects_pathological_accepts_ordinary` | REQ-008 — `a{1000}{1000}{1000}` → `TooComplex`; 5 ordinary patterns compile |
| find.rs | `t339_escape_literal_round_trips` | REQ-012 support — `a[b` invalid unescaped, literal escaped; `a.b` metachar case |
| find.rs | `t339_empty_pattern_matches_nothing` | parity with `find_all`'s empty-needle rule |
| find.rs | `t339_find_error_display_and_traits` | `FindError` Display + Error + Eq |
| find.rs | **`t339_literal_via_regex_matches_find_all_over_randomized_input`** | REQ-006 — **the OFF-identity, as ROUTING**: 2,000 randomized ASCII pairs where `find_all_regex(escaped)` == `find_all`; metachar alphabet so `escape_literal` is exercised |
| headless | **`efind_regex_invalid_pattern_is_state_not_a_lie_headless`** | the inspect HIGH — an invalid pattern sets `efind_error` (the render now READS it; the `(no matches)` lie is gone) |
| headless | `efind_regex_feeds_the_match_plumbing_headless` | REQ-007 — a regex query feeds the counter (3 fn decls via `fn \w+`) |
| headless | `efind_chip_toggle_researches_without_a_query_change_headless` | the pass-2 memo behavior — toggle re-searches, same query, 1→2 |
| headless | **`efind_empty_match_replace_advances_headless`** | F3 — an empty-match Replace-One ADVANCES through the REAL keystroke path, not a copy |

Two `#[cfg(test)]` hooks added because tests CONSUME them (the #337 F2 discipline — not before): `efind_refresh_for_test`
(set query + refresh + read the derived-match state the render draws) and `efind_index_for_test` (a PURE reader,
so the F3 drive proves the PRODUCTION Replace-One arm advanced rather than a copy of the guard).

### The two test-writing mistakes I made and fixed (both proven, not assumed)

- **`a*` over `"aaa"`** — I asserted a phantom trailing empty `(3,3)`. `find_iter` does NOT emit an empty match
  adjacent to a non-empty one; it gives just `(0,3)`. Verified against the crate in a throwaway before
  correcting — I did not guess a second time.
- **`escape_literal` round-trip** — I asserted the *unescaped* `a.b(c)*` is invalid; it is a *valid* regex
  (`.` any-char, `(c)*` zero c's), so `.is_err()` was wrong. Swapped to `a[b` (genuinely invalid unescaped)
  plus an `a.b` metachar case that proves the escape changes the meaning.

### Negative smokes — every critical row proven able to fail

| smoke | result |
|---|---|
| `char_at` advances by 2 not 1 | `t339_regex_offsets_are_chars_not_bytes` **FAILS** (`(4,8)` not `(2,4)`) |
| the F3 guard reverted to `s + repl.len()` always | `efind_empty_match_replace…` **FAILS** — `0 -> 0 -> 0`, the exact infinite loop |
| the mode DROPPED from the memo key | the toggle drive **STILL PASSES** — see below |

**The third smoke is the finding.** Dropping the mode from the memo key did NOT fail my toggle test, because
the toggles ALSO null the key (inspect F5's point exactly): both mechanisms are present, so removing either
alone still re-searches. **My test asserts the BEHAVIOR (toggle → different count), which is correct and robust
across either single removal — but its comment over-claimed "the mode is in the memo key" as the sole
mechanism.** Corrected the comment to say what is true: two mechanisms, the test guards the observable, and a
key-only smoke cannot fail while the toggle-null stands. That is the "a passing test proves nothing until you
watch it fail" lesson applied to my own validation — the smoke is what caught the over-claim.

### Gate — GREEN, first try

```
PASS gate:1 rustfmt … gate:14 docs   PASS gate:4 coverage (100%)
PASS gate:5 mutation (MSI 100 — 15 caught / 0 missed)   PASS gate:6 miri   PASS gate:15 visual/AX
GATE GREEN [diff]
```
Receipt `bf31419f5434300eef53dd3cb56a3b2ed232656b` verified against a live `gate_state_hash` — MATCH,
commit-valid. `find.rs` confirmed in the `--in-diff` mutation set (9 mutants; the `git add -N` trap avoided).

**One timeout, and I checked it rather than trusting the label** (#345 exists precisely because a laundered
survivor hides as a timeout). It was `refresh_efind_matches → ()`, and its log shows **all four of #339's new
headless drives FAILED against it** — they are what kills it. The "Timeout" is the #345 artifact: `cargo test`
does not fail-fast, so it ground on past the failures until the deadline. MSI 100 is honest, and a bonus proof
that the drives are load-bearing (they are the only thing testing the shim's refresh).

### Live pixel drive — DEFERRED, not skipped
The render change (the inline error row + the `.* `/`Aa ` mode prefix) is verified by the headless STATE assert
(`efind_error` is set → the render takes the error path, not `(no matches)`) + mechanism (the label is computed
in `render()` from exactly that state). chad is at the machine, so the pixel — the error text and chip prefix
actually painting — is deferred (~30s when free, no ticket). The STATE that drives the label is pinned; only
the glyphs are unverified.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

**Docs (§21).** CHANGELOG under **Added** (the split, the two nonexistent controls, the dissolved decision,
the Unicode divergence, the byte→char pass, the write-only-field lie). `editor.md` gains the `find_all_regex`
peer-engine section (the monotone cursor + debug_assert, Fork B, D-EMPTY-ADVANCE dissolved, FindError + the
memo key); `crate-map.md` records the editor's 3rd dep + the new surface; `roadmap.md` M22 B-a → ✅ SHIPPED
with the #347 split noted.

**Knowledge captured.**
- `AD-claude-byte-offset-crate-to-char-offset-editor-monotone-cursor-001` — the durable seam. **#347 and any
  future crate-backed search MUST reuse it** rather than re-derive a per-match conversion; the multibyte test
  is a required row (ASCII proves nothing about units).
- `AD-claude-forced-prior-art-sweep-at-plan-001` — the process change made THIS session, and it paid again
  here (the sweep dissolved D-EMPTY-ADVANCE).
- `BF-claude-dropped-my-own-render-half-and-passed-the-phase-001` +
  `PR-claude-diff-phase-3-against-your-own-phase-2-manifest-001`.
- AAR `34cfa1c3…` submitted: outcome completed, effectiveness **4** — a user-facing render LIE reached inspect
  because I passed Phase 3 without diffing my build against my own manifest.

**Follow-ups:** **#347** (capture-group REPLACE — filed at the Phase 2 split, carrying F1/F2/F3-replace).

### What this ticket taught

**The method's 4th run caught the most drift yet** — a self-refuting central decision, two REQs naming
controls that never existed, and two silent-corruption hazards in a coverage-excluded shim. And the
**prior-art sweep — required as of this session — dissolved a whole locked decision**: `regex::find_iter`
already owned the empty-match advance, and better than the rule the spec proposed. Reading the dependency
deleted the work. That is the sweep's thesis proven on the first ticket after it became mandatory.

**But the ticket's real lesson is about ME, not the spec.** I built the state, the logic, and the palette,
then wrote "Phase 3 PASS" while silently dropping the RENDER half my own Phase-2 manifest listed. The result
was `efind_error` shipping write-only — the bar rendering "(no matches)" for a pattern that never compiled —
and NO gate could see it (a struct-literal-initialized private field emits no rustc warning; app.rs is
coverage-excluded + mutants::skip). Two critics caught it from opposite ends. **The fix is a checklist, not a
resolution to try harder: diff Phase 3 against the Phase 2 manifest, item by item. "PASS" is a checklist
result, not a feeling. And for a UI ticket, state without a reader is dead state — grep every new field for a
read.**

**Six findings this batch were correct-code-wrong-reason, several of them mine** (F5's memo-key causality,
F6's false "only regex affects the error"). The negative smoke extended the lesson to my TESTS: my memo-toggle
test proved the observable but its comment credited the wrong mechanism, and only a smoke that *failed to fail*
surfaced it. **A passing test proves nothing until you have watched it fail — including watching it fail for
the reason you claim.**

And the smaller discipline that saved a second mistake: I **read the crate before asserting** (the `a*`
phantom-empty and `a.b(c)*`-is-valid errors were caught by a throwaway probe, not guessed twice).

status: Phase 5 — Complete PASS
