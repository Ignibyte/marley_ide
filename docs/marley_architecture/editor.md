# marley_editor — the text-editing core + the app's editable editor

**Status:** `marley_editor` delivered at M1.A · TICKET-011 · forge #15; grown into a real **editable editor** in
**M15** (the `/work 249-258` train). Contract: [`SPEC-editor.spec.md`](../specs/SPEC-editor.spec.md).
**Pure/shim split:** the `marley_editor` crate (`crates/editor/`) is 100% pure (no gpui, no OS/IO; cov 100 /
MSI 100); the app-side editor (`crates/marley_app/{editor_surface,code_view,input}.rs`) is pure logic seams
(also cov/MSI 100) driven by thin gpui shims in `app.rs` (render + `on_key_down` routing + save), each
`#[cfg_attr(test, mutants::skip)]` and driven-capture-validated.

## Purpose

Marley's text-editing core and the editable file editor it backs. The pure crate models a `ropey`-backed
`Buffer` — an ordered-disjoint **cursor set** (N-caret capable since M19 #296), range edits carrying **write
provenance**, char/word/line/vertical movement, and coalesced undo/redo. The app layer wraps one `Buffer` per open file into a multi-file editor
**surface** with a faithful monospace render, keyboard input, save + a dirty ●, click-to-caret, drag/shift
selection, and clipboard. Offsets are the one workspace vocabulary from `marley_text_offsets`
(`CharOffset`/`ByteOffset`) — the crate declares no local newtype.

## The pure core — `crates/editor/` (all PURE; cov 100 / MSI 100)

- **`types.rs`** — `EditOrigin{Human,Agent}` (the write-provenance seam — the brain's edit tag);
  `BufferVersion` (monotonic private `u64`, `initial()`=0 / `next()`=+1); `BufferDelta` (char+byte ranges +
  char+byte insert lengths, captured pre-edit); `EditResult{delta, version, origin}`; `Point{row, column}`
  (row 0-based, `column` = UTF-8 **byte** offset within the row); `pub type Rope = ropey::Rope`.
- **`buffer.rs`** — `Buffer{rope, selection, version, undo}`: `new`/`from_text`; `len_bytes|chars|lines`;
  `text`/`text_in_range`; `char_to_byte`/`byte_to_char` (random-access converters); `point_at` (byte column).
  M15 render/caret access: `line_text(row)` (a ropey line minus its trailing `\n`; out-of-range → `""`),
  `line_col(c) -> (row, char-in-line)` (a **char** index, unlike `point_at`'s byte column), `line_start(row)`
  (a row's absolute char offset, clamped). `edit(range, replacement, origin) -> EditResult` records the step on
  the undo history (coalescing a typed run); it delegates the rope mutation to a **private `apply_raw`** (remove
  + insert char-indexed → `version.next()` → build the delta) that does NOT record — so `undo`/`redo` re-apply
  the inverse through it and the history **drains, never loops**. `undo()`/`redo() -> Option<HistoryMove>`
  return the caret site + (for a grouped edit) the `(anchor, caret)` selection to restore (M17 #282);
  `begin_undo_group`/`end_undo_group` bracket N edits into one undo unit. `selection`/`set_selection`
  (clamps every endpoint into `[0, len_chars]`, then **re-canonicalizes** — see the invariant below).
  **`edit_at_selections(set, replacement, origin)`** (M19 #296) — the N-caret edit: an edit applied at every
  member **BACK-TO-FRONT**, so **no rebasing is needed at all** (an edit at a higher offset cannot shift a lower
  one — the idiom `find::replace_all` already used). **`edit_at_selections_with(set, f, origin)`** (M19 #297)
  generalizes it to a replacement computed PER CURSOR, against the PRE-edit buffer — Enter's auto-indent clones
  the leading whitespace of *each* cursor's own line, which one shared string cannot say.
  **`backspace_at_selections(set, origin)`** (M19 #297) lives in the crate precisely because backspace is the one
  gesture whose EDIT RANGES are not the user's CURSORS (a bare caret becomes a one-char *consuming* range), and
  only something that knows both halves can record an undo snapshot that gives the user back their CARETS —
  handing the ranges over as if they were cursors made ⌘Z restore selections the user never made, so the next
  keystroke replaced them and ate the restored text.
  Two undo rules the N-cursor path turns on: **one cursor ⇒ NO undo group** (a group exists to make N edits
  atomic; with one edit there is nothing to make atomic, and bracketing it anyway defeats `record`'s
  typed-run coalescing, so ⌘Z would undo one CHARACTER at a time); and **a typed RUN at N cursors coalesces**
  (`UndoHistory::end_group` merges a group into the previous one when it is the same run continued), so a word
  typed at three cursors is still one ⌘Z. A no-op sweep records nothing. **Never nest `edit_at_selections`**
  inside another `begin/end_undo_group`: `begin_group` OVERWRITES an open group and silently discards its
  records.
  **M22 #338 — the engine's post-state is a per-cursor SPAN, and the undo group is where the truth lives.**
  Until auto-close, "where does the caret go?" had one answer for every edit — the end of the inserted text —
  so `selections_after_multi_edit` just computed it. Three of auto-close's four actions need a different
  answer and one needs a RANGE back (wrap's preserved selection), so the fn takes an optional per-cursor
  `CaretSpan` = `(anchor_rel, head_rel)` from the cursor's own post-edit base. **`None`/`&[]` is the identity
  and is byte-identical to the pre-#338 fn** — pinned by a property test over randomized sets against a
  verbatim copy of the old body, because this is the app's ONE insert path and "auto-close OFF is the old
  behavior" has to be provable rather than argued. `insert_pairing_at_selections` / `backspace_pairing_at_selections`
  are the two public doors; `neighbours_at(off)` is the single place buffer knowledge enters the decision.
  - **The fix-up MUST be in-engine — a shim-side `set_selection` correction is WRONG, not merely untestable.**
    The undo group STORES the post-selection: `redo` restores `sel_after` verbatim, and `coalesces_into`
    compares `prev.sel_after == group.sel_before` to decide whether a typed run continues. Correcting the
    selection *after* the engine leaves `sel_after` describing carets the user never had — redo puts them back
    wrong, and the next keystroke fails the run test and starts a fresh undo unit (⌘Z one char at a time, at
    exactly the N-cursor moment where that hurts most). `set_selection` being `pub` makes that mistake easy to
    reach; the placing engine is why it is never necessary.
  - **THE CLASS THIS TICKET KEEPS RE-LEARNING — an invariant the producer ASSERTS and the consumer TRUSTS.**
    It surfaced three times in one change, always the same way: something was unconditionally true when it was
    written, so a downstream reader skipped its own check (and said so in a comment), and #338 was the first
    code able to falsify it.
    1. `begin_group(restore, /*cursor_anchored*/ true)` was hardcoded. `coalesces_into` does **no** arithmetic
       contiguity check — it trusts the flag and appends the next char onto the group's record. Under a caret
       span the cursor sits mid-insert, so `()` + `x` recorded `inserted = "()x"` while the text read `(x)`;
       undo survived on the char COUNT, and **redo replayed the corruption**. → the tag is COMPUTED.
    2. The single-member fast path opens no group, so `redo` recomputes the caret as `rec.at + inserted.len()`
       — the end of the insert, i.e. after the `)`. Wrap lost its selection the same way.
    3. `undo`'s ungrouped fallback recomputes as `rec.at + removed.len()` — the end of the restored text —
       which assumes **the consumed range ends at the caret**. True for every backspace until pair-backspace
       widened it forward, after which ⌘Z restored the caret one char past where the user had it.
    (2) and (3) are one predicate: the ungrouped fast path is taken only when **both** fallbacks would be
    right — `ends_at_insert && restore == set`. **The rule for the next edit action: find the consumer and
    read what it SKIPS because of your flag.** If it trusts rather than verifies, a lie costs corrupted data,
    not a wrong answer — and no test will notice, because nothing ever made it false before.
- **`auto_close.rs`** (M22 #338) — the pair/type-over/wrap decision, and the whole judgement of auto-close:
  `pair_action(typed, prev: Option<char>, next: Option<char>, has_selection) -> Action`, where
  `Action ∈ {InsertPair(char), TypeOver, Wrap(char), Insert}`. Total by requirement, not by habit — it runs on
  **every keystroke in the editor**, so a panic here is not a wrong bracket, it is the editor dying under
  someone's hands. Taking plain `Option<char>` neighbours rather than the buffer is what keeps it a pure
  function of four small values with no offset math to hide in. Also `typed_char(&str) -> Option<char>` and
  `is_pair`/`closer_of` (the tables the text half needs).
  - **`InsertPair`/`Wrap` CARRY the closer**, because only an opener can produce them. A caller re-deriving it
    via `closer_of` would hold an `Option` whose `None` arm the table already ruled out — an unreachable
    branch, i.e. an uncovered cold arm and a lie about what can happen. The value rides along so the
    impossible state is *unrepresentable* rather than merely untested.
  - **Two orderings, both load-bearing.** (1) **`has_selection` is checked FIRST** — the buffer maps
    `TypeOver` to an *empty replacement*, which is safe only because it can never be reached with a selection;
    reached with one, it would DELETE it. Nothing but a test pins this: `if has_selection` is a bare-bool `if`
    and cargo-mutants does not mutate it, so the cross-product sweep asserting `has_selection ⟹ Wrap | Insert`
    is the only guard. (2) **`TypeOver` before `InsertPair`** — the quote-likes close themselves, so with `"`
    beside the caret typing `"` matches both rules; stepping over is how you leave a string you just opened,
    and pairing there gives `""""`.
  - The **`'` guard is Rust-shaped and its limit is stated**: it suppresses after `&`, `<`, and identifier
    chars (`&'a`, `Foo<'a>`, `don't`) but NOT in bound positions (`T: 'a`, `+ 'a`, `'static`), whose `prev` is
    a space exactly like `let c = 'x'`. No one-char rule separates them; that awaits the tree (a follow-up of
    #346 — see below). It also must not gate the WRAP arm — there the fall-through is `Insert`, which REPLACES,
    so the guard would eat the user's selection to avoid a bracket it was never going to type. **A
    "conservative" guard whose fall-through is destructive is not conservative.**
  - **`pair_action` is syntax-aware (M22 #346) via a fifth small value, `context: Context` (`Code` |
    `StringOrComment`) — the caret's surrounding node kind, so the fn stays pure while the caller does the
    tree-reading.** Inside a string or comment an opener inserts only itself (no partner). The chain: the app
    reparses the file (a throwaway `HighlightSession`, gated to opener keystrokes so ordinary typing pays
    nothing) → `marley_syntax::node_kind_at(src, lang, byte)` reads the node kind at the caret → the pure,
    language-FREE `auto_close::context_from_node_kind(Option<&str>)` classifier maps the comment/string kind
    names to `StringOrComment` (any unlisted/`None` kind → `Code`, so an un-enumerated grammar under-suppresses,
    never mis-suppresses). `marley_editor` gains NO dependency on `marley_syntax` — the classifier takes a plain
    `&str`, and the app is the only place that holds both. The `Code` arm is byte-identical to #338 (the 26k
    sweep re-runs threaded with `Context::Code`). **M22 #362 added a third `Context` variant, `LifetimeBound`,
    for the `'` lifetime-**bound** cut #346 deferred:** while typing `'` the pre-insert tree has no `lifetime`
    node yet, so a node-KIND lookup can't tell `fn f<T: '` from `let c = '` — but a new pure
    `marley_syntax::in_type_parameters(src, lang, byte)` ANCESTRY probe (walk `.parent()` for a `type_parameters`
    node) can, because the `<>` delimiters anchor that node even with an incomplete inner bound (a spike verified
    this pre-insert; a bare `let c = '` has no such ancestor). The app builds the Context string/comment-first,
    then the lifetime probes → `LifetimeBound`, else `Code`. `LifetimeBound` suppresses ONLY `'` (folded into
    `blocked_quote`); a non-`'` opener (`(` for `T: Fn(…)`) still pairs. **M22 #366 closed the deferred rest of
    the class with a SECOND probe for the anchor-less positions**, `marley_syntax::speculative_lifetime_at(src,
    lang, byte)`: splice `'a` at the caret into a copy of `src`, re-parse, and gate on the covering node kind
    being exactly `lifetime` — TRUE at every `where` variant (incl. at-EOF via `function_signature_item`
    recovery) and at `Box<dyn … + '>` / `Ref<T, '>` / `&(dyn … + ')` / `-> impl Iterator + '`; FALSE
    (`label`/ERROR) at char-literal, expression, and const-generic-block positions, so no hand-rolled block
    guard exists. The app gates BOTH probes on the typed text being exactly `"'"` (`LifetimeBound` is inert for
    any other char — `blocked_quote` self-gates on the typed quote — so the gate is provably semantics-free),
    which means non-quote openers stopped paying the #362-era second parse. Loop labels still pair — the
    deliberately-open miss. The 0.24.2 recovery shapes are pinned by the `t366` units.
  - **The buffer threads context per cursor** (M22 #346): `insert_pairing_at_selections(set, typed, origin)`
    keeps its signature and delegates to `insert_pairing_at_selections_ctx(…, context_at: impl Fn(CharOffset) ->
    Context)` with `|_| Code`, so every existing caller and the `ime::replace_text` path are unchanged; only the
    app supplies a tree-probing closure. The probe is consulted ONLY when `closer_of(typed).is_some()` (an
    opener), which is what keeps the reparse off the ordinary-typing path.
- **`selection.rs`** — `Selection{anchor, head}` (a zero-width caret has `anchor == head`) + `start`/`end`/
  `len_chars`/`is_caret`; `clamp_char_offset` = `min(len)` only (the lower bound is structural on an unsigned
  offset). **`SelectionSet` is the N-cursor set** (M19 #296 landed the long-deferred "M1.B"), and its invariant
  is **ORDERED by `start()` · DISJOINT · NON-EMPTY** — *enforced*, not merely documented:
  - `from_selections(Vec<Selection>)` sorts then merges via **`overlaps`**, whose caret/range asymmetry is
    load-bearing (see Key decisions): `<=` where either side is a CARET (same-offset carets are ONE cursor and
    must collapse, or it inserts twice), `<` where both are RANGES (touching ranges are two real cursors over
    disjoint spans). An empty input yields a single caret at `0`. A merge that yields a RANGE drops the goal
    column; one that yields a CARET keeps it, so a collapsed pair is not stranded at the column a short line
    clamped it to.
    - **M22 #358 — this `<=` collapse is why two cursors inside ONE row's indent merge on ⇧Tab, and that is
      correct.** A dedent removes the leading whitespace both carets sat in, so `indent::rebase_through` clamps
      both to the new line start and `from_selections` merges them here — the columns they held no longer exist,
      leaving nowhere distinct for a second cursor. It is emphatically NOT the #297 silent-loss class: that was a
      MOTION (unrecoverable), whereas a dedent is an EDIT, so a single ⌘Z restores the whole cursor set (the
      undo group's `sel_before`). Cursors on SEPARATE rows each land at their own row's new start and all
      survive. `indent::rebase_selections`' doc carries the full rationale; headless drives pin the merge, the
      ⌘Z restore, and the separate-rows guard.
  - `Selection` carries **`goal_col: Option<usize>`** — the sticky column for vertical motion. `primary()` is
    member 0 (the TOPMOST — *not* "last-added": the set is ordered by offset, so insertion order is not
    recoverable), `last()` its twin.
  - Clamping is **not injective** — two distinct out-of-bounds cursors land on the same offset — so every path
    that clamps (`set_selection`, `edit_at_selections`) re-canonicalizes afterwards rather than rebuilding raw.
  - `selections_after_multi_edit(set, repl_len)` is the one real computation: after replacing every member's
    range with the same text, member *i* becomes a caret at
    `start_i − removed_before_i + inserted_before_i + repl_len` (pure `usize` — two monotonically increasing
    accumulators, no signed cast that could silently wrap). Typing over a selection collapses it to a caret
    after the inserted text. Proven by a 50k-step differential fuzzer against an independent front-to-back
    oracle that rebases through the real `rebase_offset` + the actual `BufferDelta`s.
- **`movement.rs`** — `move_char_left/right`, `move_word_left/right` (hand-rolled word class
  `is_alphanumeric() || '_'`, skip-then-consume — NOT UAX#29), `move_line_home/end`, and the M15 #257 vertical
  `move_up`/`move_down` (same visual **char** column on the adjacent row, clamped to a shorter line; last/first
  row stays; no goal-column memory). Selection helpers: `extend_or_move` (#255 — shift pins the anchor + moves
  the head; unshifted collapses a real selection to its edge, else moves) and `extend_or_go` (#257 — generalizes
  shift-extend to any pre-computed motion target).
- **`find.rs`** (M16 #265, M19 #298) — the search primitives: `word_range_at` (the word containing/just-before a
  caret, sharing `movement::is_word_char` so ⌘D and ⌥-arrow never disagree on what a word is), `find_all`
  (**the ONE LITERAL match engine** — every non-overlapping occurrence, char-offset, with an explicit `fold`
  flag), and `replace_all` (the find bar's back-to-front sweep in one undo group).
  **`fold` is a parameter, not a policy**: the find bar passes `true` (its shipped ASCII-case-folding), ⌘D and
  ⌘⇧L pass `false` — they exist to be *typed over*, so grabbing `FOO` when the user selected `foo` would rewrite
  text they never targeted. #265's `select_next_match` and its wrapping single-match scan `next_occurrence` were
  RETIRED by #298: `next_occurrence` probed every index and so returned SELF-OVERLAPPING matches ("aa" at 0 *and*
  at 1 in "aaa") that `find_all` never yields, which made ⌘D and ⌘⇧L silently disagree about what an occurrence
  *is* — and fed to the set invariant, an overlapping match MERGES, growing the user's selection past what they
  selected. One engine, one definition. The gestures that compose these now live in `multi_cursor.rs`.
  **M22 #339 — `find_all_regex` is the find bar's REGEX mode, a PEER not a replacement.** It returns the same
  `(CharOffset, CharOffset)` shape, so the bands / n-of-m / navigation consume it with no idea a regex was
  involved — but it is a genuinely different engine (`regex` over a materialized `&str`), and **⌘D / ⌘⇧L
  deliberately never reach it** (they are literal by construction — a multi-cursor gesture on a regex would be
  meaningless). The design that makes it a peer:
  - **The byte→char seam is ONE forward pass, and that is the whole risk.** `regex` reports BYTE offsets; the
    editor counts CHARS. Converting each match on its own is a rope walk per match AND is exactly where a
    units bug hides — on ASCII bytes == chars, the coincidence that shipped #336. Because `find_iter`'s
    boundaries only ever advance (ascending + non-overlapping → the flat start/end sequence is
    non-decreasing), a single monotone cursor over `char_indices()` resolves them all in O(n + m), with no map
    and no fallible lookup that could panic on a bad key. A `debug_assert!(target >= last_target)` restores
    the LOUD failure the rejected `HashMap` draft had: if the crate ever regressed, a stale index would be a
    silent misread, so the assert turns it into a test crash instead.
  - **Full Unicode `(?i)`, and the divergence from literal fold is a DECISION** (Fork B, pinned like #337's
    asymmetry). `find_all`'s `fold` is ASCII-only and length-preserving *by construction* (that is why it can
    step by `needle.len()`); regex `(?i)` is Unicode simple case folding, so `(?i)k` matches U+212A KELVIN
    SIGN where `find_all` does not. Same chip, each mode's own definition — `unicode(false)` was rejected
    because a regex mode whose `\w+` could not match `café` would be broken.
  - **Empty matches are the crate's problem, already solved** (D-EMPTY-ADVANCE *dissolved* — the prior-art
    sweep's first kill). `find_iter` forcefully advances past an empty match that overlaps the previous one
    (regex-automata's `Searcher`), so `^` / `\b` / `a*` stay finite WITHOUT a hand-rolled advance — and its
    rule is better than the one the spec proposed, which would have made `^` skip every other line start. Do
    NOT re-add a loop. (The replace-SIDE empty-match loop is separate — it lives in `resume_after` now, see the
    #347 REPLACE bullet below.)
  - **`FindError`** (a VALUE, `InvalidPattern`/`TooComplex` mirroring `regex::Error`'s only two shapes) + a
    bounded `size_limit` — an invalid pattern renders inline and disables navigation, never a panic (it is
    typed one keystroke at a time). `escape_literal` wraps `regex::escape` so the ⌘F reseed finds selected
    text literally and `regex` stays confined to this crate. The app's find-bar memo key GREW to carry the
    mode flags — the chips change what a match IS, so they are part of the query's identity.
  - **M22 #347 — regex REPLACE expands capture groups, and re-runs the pattern to reach them.** `$1` / `${name}`
    / `$$` are the `regex` crate's own `Captures::expand` (adopted — the same crate-owns-it kill as
    D-EMPTY-ADVANCE; no hand-rolled `$`-parser). The design crux was a fork that DISSOLVED on the evidence: the
    read path stores only `(CharOffset, CharOffset)` offset pairs and never keeps the compiled `Regex`, and a
    `Captures` borrows its haystack, so "carry the captures out of the find" is impossible — and since the regex
    is recompiled per call anyway, **re-running it at replace time adds no stored state.** `replace_all_regex`
    owns that re-run in ONE place, reusing the shared `build_regex` compile (extracted from `find_all_regex`, so
    both map `regex::Error` to `FindError` identically) and the SAME single monotone byte→char walk, and returns
    `(start, end, expanded)` per match. `replace_all_with` then applies the owned per-match strings back-to-front
    in one undo group — the `replace_all` sibling. **`resume_after` is where the two silent-corruption fixes
    live, and it is PURE on purpose:** the Replace-One arm sits in the coverage-excluded `handle_efind_key`, so
    the arithmetic that decides the caret + the resume cursor is extracted to a mutation-tested fn — `start +
    inserted_chars` (F2: the EXPANDED text's length, not the template's, so a capture that expands to a
    different length no longer skips/re-hits the following match) with a `start == end && inserted_chars == 0 →
    start + 1` guard (F3: a zero-width match whose expansion is empty still advances). Literal Replace is
    byte-identical — the branch diverges only in regex mode.
- **`anchor.rs`** (M16 #269) — positions that survive edits: `Anchor{version, offset, bias}` +
  `rebase_offset` (the public patch old→new arithmetic; the `o == e` boundary counts as ON the span —
  the unique composition-invariant convention, machine-checked at inspect over 2,430 triples). The
  `Buffer` grew the matching delta log (`deltas: Vec<(BufferVersion, BufferDelta)>`, fed ONLY by
  `apply_raw` so edit/undo/redo all rebase), `edits_since(version)` (also B3 tree-sitter's feed),
  `anchor_at` (clamps at creation) and `resolve_anchor` (fold + clamp; foreign anchors are the
  caller-contract exclusion). Deliberately CRDT-free — one linear history per buffer; fragments/
  dense-ordering/logical clocks are a concurrent multi-writer milestone. Prereq for B5 multi-cursor
  + B6 LSP. (Side effect: `PaneContent::Terminal` is now boxed — the growing pane state tipped
  clippy's large-enum-variant, and the box is the right fix for `PaneState` move costs anyway.)
- **`multi_cursor.rs`** (M19 #297) — what a human can REACH: `add_cursor_vertical` (⌘⌥↑/↓ — grows from the
  source's outer EDGE, not its `head()`, which is the end you *dragged*; a no-op at the document edges),
  `toggle_cursor_at` (⌘-click; never empties the set), `collapse_to_primary` (Esc — one BARE caret), and the
  N-cursor motion drivers `move_all_char` / `move_all_horizontal` / `move_all_vertical` / `move_all_to`. The
  drivers REUSE the shipped `extend_or_move`/`extend_or_go` per member; only the vertical one carries the goal
  column, which is the whole reason two cursors survive a short line.

  **M19 #298 added the occurrence gestures** — `add_next_occurrence` (⌘D), `select_all_occurrences` (⌘⇧L), and
  `added_member`. Four HIGH bugs in the first cut of these came from ONE mistake, and the shape of the fix is
  the **template for every multi-cursor-aware op** (#299 comment-toggle, #300 move/duplicate-line, #303
  delete-word/line):

  > **Ask the SET, and take operands by KIND — never by POSITION.** `primary()` is merely member 0 (the
  > TOPMOST) and `last()` merely the bottom-most; since #297's ⌘-click, *either can be a bare caret*. So
  > `primary().is_caret()` is NOT the question "is this the first press", and `last().end()` is NOT "where the
  > last match ended". The discriminator is the WHOLE SET (**all members are carets ⇒ first press**); the needle
  > comes from the **topmost RANGE** and the resume offset from the **bottom-most RANGE**. Getting this wrong
  > made a ⌘-clicked caret in leading indentation stall ⌘D *forever*, made one at end-of-file kill it for the
  > session, and made the first press *swallow* a range the user had built.

  Two more rules the gesture encodes, both earned:
  - **An already-held match is STEPPED OVER, not treated as exhaustion.** The set is SORTED, so once the wrap
    adds a cursor at the TOP the resume offset stays pinned at the BOTTOM — a press that stopped at the first
    held match left every occurrence *between* them permanently unreachable, and told the user it was done.
    (The cyclic walk is a **rotation** of `find_all`'s ascending list about the first match at/after the resume
    offset — deliberately not a `>=`/`<` filter pair, which admits an equivalent mutant.)
  - **A candidate must survive `from_selections` AS ITSELF, or it is not addable.** If the invariant merges it
    into a member we already hold, adding it would GROW that member past the text the user selected rather than
    add a cursor — and the needle for the next press would become a string they never chose.

  `added_member(before, after)` is what the app scrolls to. It exists because an **additive gesture never moves
  member 0**: following `primary()` (which is what `follow_editor_caret` does) scrolls to a row already on
  screen and leaves the new cursor invisible hundreds of rows away. ⌘⇧L deliberately does not scroll at all.
  **M22 #360 extended this to the ⌘⌥↑/⌘⌥↓ add-cursor gesture** (`add_cursor_vertical`): it, too, now follows
  `added_member` + the column-aware `scroll_editor_to(row, Some(head))` instead of the bare `follow_editor_caret`.
  ⌘⌥↓ grows from `set.last()` so the new caret sorts to the BOTTOM (member 0 stays put), which the old
  primary-follow left off-screen; ⌘⌥↑ grows from `primary()` and its new caret sorts to member 0, so the old
  follow tracked it only by luck. Naming the new cursor explicitly makes both directions correct regardless of
  sort.
- **`undo.rs`** — `UndoHistory` (M17 #282; widened to N cursors by M19 #297 — `SelSnapshot` was DELETED and the
  group now carries a whole `SelectionSet`, so ⌘Z restores all N cursors, and `Buffer::snapshot_of` +
  `HistoryMove::ranged_anchor` evaporated with it): a stack of `UndoGroup{records, sel_before, sel_after}` (each an
  invertible `EditRecord{at, removed, inserted, origin}` list; gpui-free, no rope — the `Buffer` owns one and
  drives the apply). An ordinary edit is a one-record group; `begin_group(sel_before)`/`end_group(sel_after)`
  bracket N edits into ONE group (a block indent / replace-all), carrying the pre/post selection so a grouped
  undo restores anchor+caret via `HistoryMove`. `record` **coalesces** a contiguous single-char insert onto the
  top group's sole record via a 5-part guard (ungrouped single-record top · last is an insert-run · new is a
  pure 1-char insert · same origin · contiguous `at`) — so one ⌘Z undoes a typed word, not one keystroke. A
  delete, a multi-char insert/paste, an origin change, or a caret jump starts a new group. Redo clears on a
  NON-EMPTY group's COMMIT (`end_group`), NOT on `begin_group` — so a no-op transaction (an empty ⇧Tab dedent, a
  0-match replace-all) can't wipe a pending redo.

  **`UndoGroup.cursor_anchored` — are this group's records AT the cursors?** (M19 #299.) A *typed run* at N
  cursors must still undo a word at a time, so #297 let a closing group COALESCE into the previous one
  (`coalesces_into`). That guard decided "is this a typed run continuing?" from **shape** — bracketed · record
  count == cursor count · selections lining up · every record a pure insert whose new insert is one char — and
  wrote its actual precondition only in prose: *"because the cursors did not move, each new insert lands
  precisely at the end of its own record's text."* #299's comment toggle satisfies **every structural
  condition** while violating the prose one: its records sit at the **min-indent column**, not at the cursors.
  So the next typed char was appended onto a record whose `at` was somewhere else, and ⌘Z deleted the wrong
  characters — **the user's code, unrecoverably** (the undo stack was drained with it).

  > **THE RULE THIS EARNED: a guard must CHECK its precondition, not argue it in a doc comment.** `record()`,
  > the single-cursor twin, has always *checked* the same contiguity (`last.at + last.inserted.chars().count()
  > == rec.at`). The grouped path only *claimed* it. **When a sibling checks what you assume, that asymmetry IS
  > the bug.**

- **Tab / ⇧Tab — line indentation at N cursors** (M17 #276, made multi-cursor at M22 #307). Both branches of
  the one key-dispatch arm act on the WHOLE cursor set, and they share a tail: build an ascending
  `LineEdit` list, optionally bracket an undo group, apply BACK-TO-FRONT, `rebase_selections`, write back.
  - **The block branch** (⇧, or any member holding a range) feeds `indent::touched_rows` — the deduped
    ascending union of every member's `line_span` — into `indent_edits`/`dedent_edits`. Two cursors sharing
    a row therefore produce ONE edit, not two. Those builders take a **discontiguous `rows: &[usize]`** and
    **enforce** the ascending+deduped contract themselves (`sort_unstable` + `dedup`), matching their
    sibling `comment::comment_edits`: both are `pub` cross-crate fns whose output the caller applies
    back-to-front, where a mis-ordered list corrupts the buffer and a duplicate row indents a line twice.
    They are total over any row value — `line_text` answers `""` past the end, so the empty-line filter
    (indent) and the `strip > 0` guard (dedent) drop such a row before `line_start` sees it. That filter is
    load-bearing for CORRECTNESS, not for panic-safety: `line_start` **clamps** rather than panicking, so a
    row that slipped through would resolve to the LAST line's start and silently edit the wrong line.
  - **The bare-caret branch** pads EVERY caret to its own next display tab stop, each column read through
    the #250 layout so tabs earlier in the line count. `SelectionSet` is ordered and same-offset carets
    collapse at construction, so the list is strictly ascending — `rebase_through`'s precondition, held by
    construction and `debug_assert`ed at the site. Two carets on ONE row are both sized in PRE-edit space:
    each lands directly after its own padding, but the later one may not finish on a post-edit stop, where
    a sequential editor would. Carets on different rows never interact.
  - **The grouping asymmetry is measured, not stylistic.** The block branch groups UNCONDITIONALLY — the
    empty-edit case must still open-then-drop, which is exactly what stops a no-op ⇧Tab clearing a pending
    redo. A LONE pad stays UNGROUPED, because `UndoHistory::record` refuses to coalesce into a bracketed
    group (guard: `sel_before.is_none() && records.len() == 1`), so wrapping one pad would stop the next
    typed character joining it and split one ⌘Z into two. Note the mechanism: a single-cursor edit never
    opens a group, so `coalesces_into` — which governs GROUP-into-GROUP absorption and needs
    `cursor_anchored` on both sides — is unreachable on that path. Multiple pads DO group (one ⌘Z reverts
    them all) and cannot be absorbed anyway.
  - The predicate asks the SET, not `primary()`. It used to read `active_selection()`, which reports the
    primary only — so a mixed set (caret primary + ranged secondary) looked selection-free, took the pad
    branch, and ignored the range entirely.

  The fix makes the PRODUCER state it: `cursor_anchored` is `true` only for `edit_ranges_restoring`'s N-cursor
  typed-insert branch (each cursor genuinely ends at the end of its own insert), and `false` for every
  hand-bracketed group — `Buffer::begin_undo_group`'s default, used by Tab/⇧Tab, ⌘/ and replace-all, whose
  records are LINE-anchored. `coalesces_into` now requires it on both sides. **Any new multi-edit line op
  (#300 move-line, #303 delete-word, #307 multi-cursor Tab) gets this for free by using `begin_undo_group`** —
  and must never nest `edit_at_selections` inside its group (that self-brackets, and `begin_group` overwrites
  an open group, silently discarding its records).

  A tag was chosen over restoring the arithmetic *even though the arithmetic is provably correct* (two critics
  verified a cross-coordinate-space contiguity computation by hand): inferring intent from geometry is what
  failed, and the tag kills the whole class rather than today's instance.
- **`comment.rs`** (M19 #299) — the ⌘/ **line-comment toggle**, expressed as the same `LineEdit` list Tab emits,
  because a comment toggle IS a line-prefix edit. `comment_edits(buffer, rows, token) -> Vec<LineEdit>` is the
  whole decision, and it is **language-agnostic**: the token arrives as a `&str` (the app resolves it from
  `code_syntax`'s existing `lang_spec().line_comment` — the SAME table the highlighter reads, never a second
  source of truth). Four rules carry it:
  - **Direction**: every live row already commented → UNCOMMENT; else COMMENT ALL.
  - **The marker column is the MINIMUM indent of the live rows**, not each row's own — the latter staircases the
    markers down the block. Each row's *relative* indent survives after the token.
  - **Uncomment strips the token + AT MOST ONE space, and that "one" is load-bearing**: it is exactly what makes
    uncomment an *inverse* of comment-at-minimum-indent (`    //     bar();` → `        bar();`, not
    `    bar();`). Strip all the following whitespace and the deep row silently loses its indent.
  - **Blank *and whitespace-only* rows are skipped**, and excluded from the minimum — a stray 3-space row inside
    a 4-space block must not drag every marker to column 3. (Deliberately a *different* predicate from
    `indent_edits`'s `.is_empty()`: four invisible spaces on a blank-looking row are harmless; a dangling `// `
    is litter.)

  **`is_commented` diverges from the reference, on purpose.** The naive prefix test (`starts_with(token)`) calls
  anything beginning with the token "already commented", and uncommenting then eats one token **irreversibly**:
  `//! PURE — …` (a module doc — the first line of every file in this crate) → `! PURE — …`; `#!/bin/sh` (a
  shebang — the script silently stops executing, failing at *exec* time) → `!/bin/sh`; `//////`/`####` banner
  rules lose a token per press down to `""`, an absorbing state. So the token must not be followed by **more of
  itself, nor by `!`** — those are richer markers, and ⌘/ *comments* them (`//! x` → `// //! x` → back) instead
  of shredding them (§0 — do not ship known, irreversible harm).
- **`indent.rs`** (M17 #276) — the v1 (non-tree-sitter) indentation ops. `indent_for_newline(line,
  caret_col_chars)` — Enter's leading-whitespace clone, clipped at the caret when it sits inside the indent
  (D1; the clip falls out of `take(col).take_while(ws)` iterator order). `line_span(buffer, anchor, caret)` —
  the touched rows, with the universal **col-0 carve**: a selection ending at column 0 of a later row
  (shift+Down) drops that row (endpoints picked by `.min()/.max()` offset — an orientation *branch* is an
  equivalent mutant at row-equality). **`touched_rows(buffer, set)`** (M19 #299) — the N-cursor generalization
  of `line_span`: the deduped, ascending UNION of every member's span. **Asks the SET, never `primary()`** — a
  line op over a multi-cursor set acts on what ALL the cursors touch, and two cursors on the same row must yield
  that row ONCE (a duplicate would emit two edits at one offset and double the prefix). Each member keeps the
  col-0 carve for free. This is also the seam **#307** needs to make Tab/⇧Tab multi-cursor aware — they still
  act on the PRIMARY's span alone, because `indent_edits`/`dedent_edits` take a CONTIGUOUS `(first, last)` range
  while a multi-cursor row set is discontiguous; widening those two to `&[usize]` is a user-visible behavior
  change and gets its own ticket. `indent_edits`/`dedent_edits` — per-line `LineEdit = (CharOffset,
  remove_chars, text)` lists, ASCENDING, applied **back-to-front** by the caller (the #272 idiom): indent
  inserts one stop of spaces at each line start (EMPTY lines skipped — no whitespace-only residue); dedent
  strips up to one stop of spaces or one leading `\t` per line (a no-strip line contributes NO entry — a
  `(at,0,"")` no-op would be a text-invisible undo blank). `spaces_to_next_tab_stop(col, tab_width)` — the
  bare-caret Tab pad, display-column-aware over the #250 `line_layout`. `rebase_through(pos, edits)` — one
  delta fold carrying BOTH selection endpoints through a list: an edit AT-or-before the position shifts it
  (an insert exactly at the position pushes it right, so the selection keeps covering the same TEXT — the
  p==at case is the only distinguishing input for two `<`→`<=` mutants); a position inside a removed span
  clamps to the edit's shifted start (the #269 covering-collapse convention). The app arm: Tab/Enter rows
  BEFORE the editor Char|Other claim (Tab's key_char `"\t"` must never reach the IME — the pre-#276
  literal-tab bug; ⇧Tab arrives keyless as `Other` and was a silent no-op); ⌘/⌃ chords are excluded by the
  arm gate. Enter's type-over is ONE `Buffer::edit` with the clone computed BEFORE the replace (inspect F1 —
  delete-then-insert was two undo steps and the first ⌘Z exposed a never-seen state). Undo: Enter = one
  step; and (M17 #282) a block indent/dedent + replace-all now undo in ONE ⌘Z via the `begin_undo_group`/
  `end_undo_group` seam, restoring the selection shape — the #276/#272 per-line v1 convention retired.

## The app-side editor — `crates/marley_app/`

- **`editor_surface.rs`** — the per-workspace multi-file model. `OpenFile{view: CodeViewState, buffer, caret,
  saved_version, anchor: Option<CharOffset>}` — the `buffer` is seeded from the file's **RAW** text (the render
  `view.lines` are lossy: tab-expanded + truncated); dirty ⇔ `buffer.version() != saved_version`; `anchor` is
  the selection anchor (`None` = a bare caret). `EditorSurface{files: Vec<OpenFile>, active}`: `open`
  (dedupe-by-path — re-opening PRESERVES the buffer, no disk-clobber of unsaved edits), `close`/`activate`,
  `from_files` (the #243 restore), and the accessors the shims edit through — including the E0499-safe combined
  disjoint-field borrows `active_buffer_and_caret_mut` (#251) and `active_buffer_caret_anchor_mut` (#255),
  `active_selection` (the normalized `(min,max)`), and `file_dirty_flags` (the per-file ●). It derives none of
  `Clone/PartialEq/Eq/Debug` (`Buffer` derives none, and nothing needs them).
- **`code_view.rs`** — the pure render/geometry math. The heart is `LineLayout{display, col_starts}` from
  `line_layout(line, tab_width)`, built in ONE pass so the render and the caret can never diverge:
  `col_of_offset(char_idx) -> display column` (#250) and its inverse `offset_of_col_f(col: f32) -> char_idx`
  (#254; FLOAT since #277 — the click passes the unrounded `x / cell_w`, else a wide glyph's flip point
  quantizes from its midpoint to 25%; nearest boundary, ties → later, which is also the never-mid-cluster
  rule for zero-width combiners). Columns are REAL display cells since M17 #277: `char_width(ch)` (UAX#11
  via unicode-width — wide/fullwidth 2, zero-width 0, control `None`→1) is the ONE width authority;
  `col_starts` is NON-decreasing (zero-width ties), tabs advance to the next stop over the ACCUMULATED
  column, and the `display` string is never padded (columns are a mapping). `offset_for_click` (#254),
  `row_selection_cols` (#255 highlight columns), and `paste_edit(selection, caret, clip_chars) -> (start,
  end, new_caret)` (#256) layer on top. The #266 render seams: `cols_to_bytes(display, c0, c1)` (display
  WIDTH-columns → a byte range — a #277 rewrite: the old `nth(col)` char-index lookup only coincided with
  columns under all-width-1; now a width-accumulating span-contains walk where an emitted endpoint never
  splits a wide glyph, a strictly-interior band is empty, and combiners ride their base cell on both edges)
  and `styled_slices(syntax, selection)` (the boundary-cut splitter producing the ascending-disjoint
  `(byte range, TokenKind, selected)` slices `StyledText::with_highlights` requires — verified against
  gpui's `compute_runs` contract). Known #277 residuals (recorded follow-up): grapheme sequences diverge
  both ways (ZWJ families over-count, VS16/keycap under-count — the str-level width is already
  sequence-correct, so a cluster walk is the eventual fix), and the caret's cell grid vs the font's real
  advance are two authorities wherever they disagree. It also still owns the M4 read-only viewer helpers
  (`code_lines`, `gutter_*`, `visible_range`, `scroll_code`, `is_probably_binary`, `parse_file_ref`,
  `CodeViewState` — char-truncated at 200, no caret math, consciously width-agnostic).
- **`input.rs`** — the pure key → editor-op mapping. `apply_key` drives the single-line **command prompt**
  (Enter ⇒ `Submit`); `apply_editor_key` (#251) drives the multi-line **editor** — identical typing (delegates
  `Char`/`Backspace`/`Left`/`Right`) but Enter inserts `\n`. Plus `selection_replacement` (#255 type-over),
  `split_caret_char` (the prompt's #218 block-cursor glyph split), and `submit_line` (the dead `split_at_caret`
  was removed in #266 — zero production callers; the prompt always used `split_caret_char`). Since #267 the
  editor's PLAIN-CHAR insertion no longer flows through `apply_editor_key` — see `ime.rs` below; the fn still
  serves Enter/Backspace/motions on the router. `swallow_hidden_prompt_key(is_terminal, key)` (#283) is the
  guard the shim applies at the TOP of the terminal fallthrough: a MODIFIED Enter/Tab/Backspace (⌘/⌃) on a
  NON-terminal tab skipped the editor router (gated `!platform && !control`) and would otherwise act on the
  HIDDEN prompt via `focused_terminal` (#71) — swallowed there, byte-identical for terminal tabs (mirrors the
  #278 `grid().is_some()` gate).
- **Editor find & replace** (#272) — the Editor-context ⌘F bar (state family `efind_*` on RootView,
  the #47 idiom): pure `marley_editor::find_all` (ASCII-fold, non-overlapping) + `replace_all`
  (back-to-front, one undo step per match) feed a `(nonce, version, query)`-keyed match memo that
  refreshes at the render head AND at the key-handler top (a stale range set must never feed
  Buffer::edit — the #272 inspect's executed-panic lesson); matches render through
  `styled_slices_with_marks` (the #266 channel generalized to N mark ranges, Current beats Match);
  Replace One advances via a resume offset (needle-containing replacements can't loop); unowned
  ⌘/⌃ chords fall through the open bar (⌘Z/⌘1/⌘F-reseed work); a tab switch closes it. ⌘A
  select-all is an Editor-context row beside it.
- **`marley_syntax`** (#268; **multi-language M20 #315**) — semantic highlighting: tree-sitter (MIT, the FFI
  stays in the deps — the crate itself is safe Rust) parses the WHOLE document with the FILE'S grammar. A `Lang`
  axis (`lang.rs`) selects the grammar + its OWN adopted `HIGHLIGHT(S)_QUERY` for Rust/Python/JavaScript/
  TypeScript/TSX/JSON/Bash (TS/TSX CONCATENATE the JavaScript base ahead of the `inherits: ecma`/`jsx` overlay —
  the overlay alone is blank); the language-agnostic `kind_of_capture` maps capture names onto the **10-kind
  palette** (grown 5→10 in #315: +Function/Type/Attribute/Punctuation/Property) via tree-sitter's shared base-name
  convention, with one full-name override — an object/JSON key `string.special.key`→Property (REQ-003, so keys
  read distinct from string values); `sweep_disjoint` (a STABLE sort, so an identical-range double-capture is
  deterministic) + `clip_to_lines` (per-span `partition_point` distribution) yield per-line ascending-disjoint
  RAW-byte spans. The app memoizes per `(buffer-nonce, version)` — `OpenFile.nonce` is the process-monotonic
  buffer identity (a bare path+version key goes stale on reopen; see the #268 inspect) — and the render remaps raw
  spans onto the tab-expanded display via `raw_span_to_display_bytes`, feeding the #266 substrate unchanged. A
  no-grammar file (Markdown/Plain) + **TOML** (deferred at #315 inspect — its `(pair) @property` broad capture
  fights the outer-wins sweep; awaits an innermost-wins sweep) + the #246 pane keep `code_syntax`'s hand lexer.
  The off-thread worker rebuilds its `HighlightSession` when the file's language changes (a switch is a new nonce
  → a full parse anyway; the rebuild re-points the parser at the new grammar). **M20 #316** themes those 10 kinds
  per-theme: `app.rs::token_color` no longer carries hardcoded `hsla` — it resolves the 9 syntax kinds through the
  active theme's `ThemeColors.syntax` (`marley_ui_components::SyntaxPalette`, one `Hsla` per `SyntaxSlot`,
  contrast-proven ≥AA + pairwise-distinct by test), with `Hint`→`muted` / `Plain`→`foreground` app-side. Because
  the syntax cache stores KINDS not colors, a theme switch recolors code with no cache to invalidate.
  Small files (≤1000 lines) parse synchronously per version change (1-3ms — zero added latency);
  LARGE files go off-thread + incremental since M17 #274 (B3.2): `HighlightSession` keeps the
  previous `Tree` AND the previous TEXT snapshot (`BufferDelta` carries no removed text/points —
  the snapshot makes every `InputEdit` point pure string math: `point_at` (row, BYTE-col, floor-
  guarded on non-boundary bytes) + `syntax_edit`, probe-verified against tree-sitter's own node
  positions), and its `fits` guard checks the SPLICE CONTRACT itself (boundary + prefix/suffix
  memcmp — a length-consistent lie falls back to the always-correct full parse: total, never
  wrong). The app enqueues to ONE worker thread (a `Parser` is Send, !Sync) over a
  generation-dropped channel the pump drains: the worker COALESCES bursts (drain-to-latest) and
  goes incremental only when its session sits at exactly `(nonce, parent_version)` — skips
  self-heal to full; single-delta steps (from `edits_since`) ride the edit, multi-delta gaps
  (autorepeat/undo) take the full path. The pending guard stops per-frame re-enqueues; a
  `Disconnected` receiver tears the worker down (sync fallback — highlighting never dies with
  the thread); a foreign-nonce cache DROPS at enqueue (stale-but-aligned means same-file only).
  Measured (inspect probe, release, 8k lines): re-parse 1.0 vs 9.2ms fresh (the 9× tree-sitter
  win); the O(file) query walk (5.7ms over 27k captures) floored end-to-end at 0.46 of full until
  M17 #285 WINDOWED it. After the re-parse, `damage_window` merges `old_edited.changed_ranges(&new)`
  with the edit span; a grow-loop widens the window and re-queries `spans_in_window` (`set_byte_range`)
  until every fresh span lies INSIDE it (no straddler); `splice_spans` drops the cached spans touching
  the replaced region or overlapping the window, rebases the survivors by the edit delta, and unions
  the fresh — the result equals a full walk of the new tree as a MULTISET, so `lines_from_spans` yields
  byte-identical per-line output (the #274 equivalence corpus is the gate), degrading to the full walk
  on a block-comment cascade. `HighlightSession` caches the previous RAW span set (`last_spans`) as the
  splice base. That drops the query cost to O(damage) and tightens the end-to-end pin from 3/4 to the
  originally-intended 1/3 of a full highlight. **M17 #288 (a CRITICAL #285 fix):** `changed_ranges` is a
  BYTE diff, not a token diff, so a SHRINKING edit — a backspace off a comment's tail, or a token
  re-tokenizing when a delimiter is deleted (`*/` gone → the comment swallows forward) — left the surviving
  token unreported and outside a byte-tight window; `splice` dropped it with nothing to replace it and the
  highlight VANISHED. `cover_edited_cached` now widens the window to the surviving new-coord footprint of
  every cached span OVERLAPPING OR ABUTTING `[start, old_end]`, and `snap_to_lines` snaps it to whole-line
  bounds (same-line boundary re-tokenization). The equivalence contract is scoped precisely: "SAME
  incremental tree ⇒ same spans" — tree-sitter's incremental parser can itself diverge from a fresh parse on
  rare error inputs (~1/100k on random garbage), which is not this crate's windowing. A deterministic
  tree-match-guarded differential fuzzer (`t288_differential_fuzz`) is the durable regression — the shape
  that FOUND the bug, now 0 same-tree divergences over ~120k adversarial steps. (True O(damage) via
  per-line-vector reuse — retiring the remaining O(file) span bookkeeping — is the recorded next step if the
  pin ever tightens further.)
- **`marley_editor::ime`** (#267) — the pure NSTextInputClient half: `replace_text` (`insertText:`),
  `replace_and_mark` (`setMarkedText:` — sets the pending composition span + the composition-internal
  selection via relative-UTF-16 mapping), `unmark`, `text_for_range` (clamp + adjusted write-back),
  `selected_utf16` (reversed-aware), `marked_utf16`, over a UTF-16↔`CharOffset` seam on `Buffer`
  (`char_to_utf16`/`utf16_to_char`, ropey-backed, clamped, mid-surrogate rounds down at BOTH seams). Edit
  target precedence: explicit range → marked → selection → caret. A fully-empty call is a no-op (an
  input-source switch must not burn the redo stack). The `OpenFile.marked` span lives on
  `editor_surface.rs`; every non-IME mutation path clears it (click, chord verbs, cut/paste, undo/redo,
  file open/close/activate) — a stale span would misdirect the next IME replace.
  **M22 #338 — this is where AUTO-CLOSE hooks, and the existing fork IS the whole IME answer.**
  `replace_text` already splits on `range_utf16.is_none() && marked.is_none()`: branch 1 is ordinary typing at
  N cursors; branch 2 is "the platform named exactly ONE span" — an explicit `replacementRange` or a
  composition being committed. Auto-close routes **branch 1 only** (`auto_close && typed_char(text)` →
  `insert_pairing_at_selections`), so **mid-composition pairing is excluded BY CONSTRUCTION** — there is not a
  line of IME-awareness anywhere in the feature, and there did not need to be. That exclusion has everyday
  reach rather than just CJK: macOS sends an explicit `replacementRange` for press-and-hold accent selection
  and Text Replacement / autocorrect on a stock US keyboard. The one-char gate is the same idea from the other
  side — `text` is a `&str`, not a key, so a multi-char insert is an expansion or a commit, never a keystroke,
  and a snippet containing `(` must not sprout a `)` in the middle of the user's text.
- **The gpui shims** (`app.rs`, render-skipped, driven-validated — see [`app_shell.md`](app_shell.md) §M15 for
  the routing detail). Since #266 the editor arm of `code_view_body` renders on the gpui-native substrate:
  `uniform_list("editor-lines", len_lines, closure)` virtualizes the real viewport over the WHOLE buffer (the
  fixed `VIEWER_ROWS = 40` window is retired — files scroll to EOF via the `RootView.editor_scroll:
  UniformListScrollHandle` + `.track_scroll`; the editor-tab wheel handler is deleted, the list scrolls
  natively), and each row is ONE `StyledText::with_highlights` over the tab-expanded `display` string —
  syntax tints from `highlight_ranges` + the #255 selection as a 28%-accent `background_color` range, cut
  disjoint by `styled_slices`. Rows render `line_text(row)` through `line_layout` (tabs expanded app-side,
  `.whitespace_nowrap()` clips over-wide lines) and the caret bar stays a pure-math overlay at
  `col × the measured monospace cell` (#250) — `TextLayout` geometry waits for a proportional font. Known
  #266 trade: ONE shared scroll handle means per-file scroll memory is gone until #270 restores it via
  `scroll_to_item`. *(Coda — M17 #273: HEALED. Each `OpenFile` parks its pixel offset; the render
  detects owner transitions by the #268 nonce at one choke point (park outgoing → clear any pending
  deferred jump → restore incoming, clamped vs the current line count via `clamp_scroll_px`); pub
  `scroll_editor_to_row(row)` (non-strict Center) is the shared mechanism for #270/#272/#212/#213 —
  with the documented constraint that open-then-jump must span two frames, since the transition
  clears pending deferreds. The virgin-boot launcher arm also gained the off-thread boot-PTY reap
  the restored arm always had.)* *(Coda — M18 #270: CARET-FOLLOW wired. `follow_editor_caret()` calls
  `scroll_editor_to_row(line_col(caret).0)` after each keyboard caret motion — the #257 key branch, the
  platform ⌘-motion branch (⌘↓ doc-end / ⌘←→ line), and the #272 ⌘D — so a far motion no longer leaves
  the caret off-screen (the non-strict Center no-ops when already visible). A click is not wired (it lands
  visible). Headless-proven via `editor_scroll_y_for_test`; the prerequisite for #290 diagnostic nav.)* Since #267 the same first-row canvas records the full `EditorFrameGeom`
  (x0/y0/rendered-rows/cell) and its PAINT closure registers the `ElementInputHandler`
  (`Window::handle_input` — paint-scoped + focus-gated, so ONLY a painted editor frame accepts OS text;
  terminal frames never register). `RootView: EntityInputHandler` routes the platform's calls to the
  `ime` ops; the editor key arm claims printables/dead-keys but PROPAGATES them to the handler (one
  insert mechanism), stops propagation on every handled key, and every full-capture overlay arm stops
  too (a consumed Enter's "\n" key_char must never reach the handler fallback); `text_input_blocked()`
  mirrors the overlay ladder; Esc deliberately deselects. The shell `on_key_down` router intercepts an active editor
  tab: typing → `apply_editor_key`; the #257 motion keys (Home/End, ⌥←→ word, ⌘←→ line, ⌘↑↓ document, Up/Down
  vertical) → the `movement` fns + `extend_or_go`; ⌘S saves the buffer to disk (mark-clean only on a successful
  write, #252); ⌘Z/⌘⇧Z → `Buffer::undo`/`redo` (#253); ⌘C/⌘X/⌘V → the clipboard branch over `paste_edit` (#256);
  a click → `offset_for_click` + `line_start` (#254); a mouse-drag / shift-arrow → the anchor + `row_selection_cols`
  highlight (#255).

## The multi-file & split-pane model

Opening a file lands in the workspace's ONE `EditorSurface` (a single left-rail row with a file-tab strip),
not a new rail tab (the #237 anti-clutter model). Each tab is an independent `OpenFile` with its own buffer,
caret, selection, dirty state, and undo history. The persisted **open set** (paths + active index) restores via
`from_files` (#243). Separately, a terminal can **split right into a read-only file pane** (#246), which
persists across restart through the `c=<path>` grid codec (#258, a strict sibling of #205's terminal `t=<cwd>`);
that pane stays read-only — the editable split pane is the fast-follow #259.

## External file-change detection (M17 #275 — the agent workflow)

Every `OpenFile` carries a **(mtime, len) disk snapshot**, statted BEFORE the content read (the safe race
direction — a write landing in the gap leaves the snapshot older, so the next check converges) at open, at
boot-restore (`RestoredFile` threads it through `from_files` — restore is a second constructor; the inspect
HIGH was exactly this seam unseeded), and after each save (write-then-stat is inherent there; a post-write
LEN cross-check flags a racing writer instead of silently adopting its file). The pure
`extchange::external_action(snapshot, disk, dirty)` table decides `Noop | CleanReload | Conflict | Deleted`
(`None` snapshot = untracked = Noop — also the dismissed-removal semantic); `focus_edge` is the
inactive→active detector. **Poll-at-interaction, no watcher thread (v1):** the check runs at the window
focus-REGAINED render edge (guarded on `project_count() > 0` — the launcher's empty set panics on
`active_project()`), every reveal-activation (file-tab click, ⌘]/⌘[/⌘1-9, workspace cycle, rail jump,
close-reveal, open-switch — the check early-returns free on terminal tabs), and the head of ⌘S. Clean +
changed → **silent reload**: fresh buffer (undo restarts — an external epoch), caret clamped, nonce
re-minted (the #268 memo re-parses; the #273 scroll sync re-clamps via its owner-change path — one clamp
authority), flash; a content-identical rewrite (idempotent formatter) skips the epoch and just re-snapshots.
Dirty + changed → the floating **banner** (a #221 card, `block_mouse_except_scroll` per the #185 rule):
Keep-mine re-snapshots the disk (D3 — the next change re-flags; a vanished file keeps the old snapshot so
Deleted still flags), Reload discards explicitly (the flash says "unsaved edits discarded").
**⌘S under a Changed conflict arms on the first press** (warn, no write) and overwrites on the second;
Deleted needs no arming — ⌘S recreates and re-tracks; Dismiss untracks (snapshot `None`) so the banner
stays gone until a save re-establishes tracking. **#284 — acknowledgments bind to the OBSERVED disk**: the
arm carries `armed_at: Option<(mtime,len)>` and the `Changed` banner a `conflict_observed` pair; a pure
`acknowledgment_is_stale(acked, now)` makes the armed 2nd ⌘S DISARM + re-warn (and Keep-mine re-flag) when
a NEWER write landed since the acknowledgment — never overwriting / re-snapshotting content the user never
saw. Testability: gpui's TestWindow reports INACTIVE, so the focus edge only exists in tests that drive
`activate_window`/`deactivate_window` — the headless flows do (including the launcher-guard pin on a virgin
boot). Recorded follow-up: the real FSEvents watcher.

## The multibuffer (M32 #427 — the read-only face)

The first multi-file surface: ⌘⏎ on a live ⌘⇧F result set materializes a **Multibuffer tab**
(rail section Editor, title `Search: <query>`, status label "search results"). The PURE model
(`multibuffer.rs`, cov/MSI 100) is a snapshot built at materialization: per-file excerpt groups
(match rows ± `CONTEXT_LINES`, windows merged as half-open runs, out-of-order matches healed),
char-domain match spans per line, in-range match counts, the carried `dropped` cap tail, and the
prefix-summed row model (`locate`: slot → Header(file) | Line(file, line) — the #426 `WrapIndex`
shape). The render is the repo's second `uniform_list` (own scroll handle): header bands via the
search overlay's group idiom, `highlight_ranges` syntax colors ONLY (ascending+disjoint — the
`with_highlights` contract; the match marker is a full-ROW wash, the POC's own form), first-run
hairlines, and a trailing footer slot. Sourcing = the two-source rule: `editors_under` live
buffers win, else the viewer guard chain (canonical-under-root → stat → 2MB cap → binary sniff →
LOSSY decode + post-read size re-check). Jump = the shared `jump_to_match(path, row, col)`
recipe; the model's `origin` (the editor loc captured at ⌘⏎) is pushed once on the first
successful jump — after that the still-open tab is the way back. Lifecycle: a #403-checklist
kind — always-insert (insert's view is the tab's), released on tab/project close, TRANSIENT
across restarts (the persist writer drops it and re-counts `active_tab` over survivors).
**#428 — the editable face.** A single anchor-backed caret (placed by click at line end;
←/→ walk it; ↑/↓ and Esc drop it) routes typing/IME through the surface's OWN
`handle_input` canvas (PR-claude-428-a: the input seam is per-surface) into the standing
`replace_text_ctx` → `Buffer::edit_*` chain on the file's ONE registry instance; ⌫/⌦ ride
`apply_editor_key_multi` (pairing-aware), Enter is a plain newline. Excerpt windows are
#269 anchor pairs `(start Bias::Left, end Bias::Right)` resolved at render top
(`sync_multibuffer_live`, memo (nonce, version); an epoch mismatch — reload — RE-MINTS
from the current rows and drops the caret); boundary deletes are blocked at the resolved
window edges. The buffer's live `SelectionSet` is saved/restored around every mb edit
(PR-claude-428-b). Undo: per-buffer `UndoHistory` + the model's journal — one entry per
undo group (`journal_note_edit` retain+push), popped only when `undo_depth()` still
matches (stale entries drain). ⌘S = `save_editor_by_id(_, Sweep)` over touched ∩ dirty
with an outcome flash. Lifecycle: open files pin a view at materialize; a not-open file's
instance births at first caret (`open_editor_instance`); all targets release at close as
editor views (folds/marks scrub applies).

**#429 — replace all.** The ⌘⇧F overlay grows a replace row (Tab hops query↔replace;
the focused field carries the ▏ mark; replace edits never re-park the walk) and a regex
mode (⌘⌥R, the passive ".*" chip). A request compiles ONCE at `consume_search_query` —
`editor::compile_find` → `SearchReq.compiled`, the walk running `find_all_compiled` (the
find bar engine's hoisted-compile fork; empty matches filtered before assembly) through
`editor_search::to_file_matches` (the literal engine's cap/preview mirror) — and an
invalid pattern renders inert ("invalid pattern"), no walk spawned. ⌘⌥⏎
(`mb_replace_all_apply`) materializes-then-applies per file through the SAME #428 target
recipe (live buffers and disk-only files alike): literal = `find_all` fold +
`replace_all`, regex = `replace_all_regex` with `$n` templates, each buffer its OWN undo
group; every file joins touched + the journal, and ONE `batch_note` makes the gesture a
single ⌘Z/⌘⇧Z across every file (`batch_covering` pops the run; `batches_invalidate` on
foreign pops). Guards fail CLOSED — empty/invalid query, a still-streaming walk
(`complete` on the live gen), and an apply whose surface did not open all flash-and-bail
with zero edits (F-claude-429-a / PR-claude-429-a: a gesture verifies its own
preconditions, not its trigger's). The flash: "N replaced in M files (· K skipped)".

**#430 — the diagnostics form.** ⌘⏎ in the ⌘⇧M problems panel materializes the
aggregated `problem_rows` set through `editor_problems::build_problems` (group per file,
severity-first order; raw LSP `(line, character)` → char cols via the ONE
`position_to_offset` path; per-match `NoteMeta` bands parallel to surviving matches).
The model's slot domain is now a MATERIALIZED `slots: Vec<Row>` (Header | `Row::Note`
band | Line — the old header+lines prefix sums could not interleave bands under
`uniform_list`'s uniform heights); movers/jump/caret skip Note like Header, and
`refresh_cum` normalizes a rebuild-shifted selection FORWARD to the nearest Line (the
backward arm was dead by construction — deleted, invariant stated). Targets pin/birth
under the FILE's own project root (`root_of_mb_path` — longest-prefix, active fallback;
blanket-active-root split-brained cross-project buffers). The surface is a per-project
SINGLETON ("Problems" tab; re-materialize swaps in place, new pins acquired before old
release). REFRESH is quiescent: the pump re-materializes when the workspace PUBLISH
EPOCH (`DiagnosticStore::publish_epoch` — monotone, bumped per publish incl. clears)
moves AND no mb target is dirty; ⌘S arms as belt (the row-SUM fingerprint provably
swallows "one fixed + one new" — caught live). All-resolved renders "All problems
resolved" in the footer slot; the tab never closes itself. Fail-closed guards on the
materialize (empty set / zero sourceable excerpts).

**#431 — the excerpt stage joins the facade (the chain's unification prize, cashed).**
The mb's parallel row model is GONE: the materialized slot list now lives in the ONE
`display_map.rs` facade as `ExcerptIndex` (`slots: Vec<Row>` + a `line_rows` snapshot),
Arc-memoized on the model and rebuilt ONLY by `refresh_cum` (the `WrapIndex`
discipline — a `DisplayMap` per crossing, the index never). `DisplayMap` carries it as
a third arm (`excerpts()` beside `new`/`with_wrap`/`identity`; the editor constructors
set it `None` — bit-for-bit) with typed doors — `excerpt_total`, `excerpt_locate`
(`None` iff empty; past-end saturates), `excerpt_slot_of_row`,
`excerpt_move_selection` (pure mover; normalize delegates its scan to it — ONE forward
scan) — and the app.rs rim converts ONCE per slot (`DisplayRow`, `selected` typed; the
gpui edge stays raw counts). The model kept its public build/rebuild/anchor/journal
surface plus `line_at` (a files-borrow adapter over the index) and DROPPED
`total_rows`/`locate`/`move_selection`/`slot_of_row` (one home; no delegating copies).
Stages are DISJOINT this ticket — the mb instance's identity fold slot + absent wrap
are exactly where the wedge features (folds/wrap INSIDE excerpts) will compose,
facade-internally. Byte-identical by proof: the #427/#430 mint carried verbatim as the
in-test ORACLE (the #425 recipe adapted to a consuming refactor), the full suite
unchanged, live-driven on the search surface (materialize → movers → jump on real
pixels). Deliberate in-crate module cycle: the stage consumes the model's group
vocabulary (`FileExcerpts`/`Row`); the model memoizes the stage.

**#432 — input completeness (the three #428 v1 seams close).** A recorded per-frame
**mb geometry** (`MbFrameGeom`, the `EditorFrameGeom` twin in its own cell — a
zero-width, ROW-TALL probe on the first Line row's text cell; its height IS `cell_h`;
a batch with no Line row ZEROES the cell so consumers answer `None`, never stale
rects) powers all three arms. **Click-column**: the click maps `x − x0 / cell_w`
(float, unrounded) through the editor's own `LineLayout` inversion
(`offset_for_click` — tab-aware, wide-glyph midpoint, [0, line-end] clamps);
`mb_place_caret` grew `col: Option<f32>` (`None` = the line-end v1 arm); the caret
bar paints at `col_of_offset` over the SAME layout — one column map, exact
round-trip. **⌘V paste** (`mb_paste`): clear composition → ONE `edit_at_selections`
on the transient caret set → `mb_after_edit` — a multi-line clip is the same single
edit and the window GROWS via the standing #428 anchors (Enter's shape; one journal
entry, one ⌘Z). **IME**: every `EntityInputHandler` method gained its mb arm — the
commit arm shipped at #428; compose (`replace_and_mark`) rides the transient
chassis' `&mut mb_marked` param; the READ arms answer from the caret ANCHOR +
`mb_marked`, never the target's live SelectionSet (PR-claude-428-b);
`bounds_for_range`/`character_index_for_point` answer from the recorded geometry
through the #431 doors (cross-file points → `None`). **Fail-closed routing** (the
inspect's own harvest — three MEDIUMs): the mb ⌘C/X/V arm consumes ALL modifier
variants (⌘⇧V's "paste without formatting" leaked pre-fix), an mb PLAIN-KEY claim
(the #251 twin) killed the #428-era double-delivery into the hidden workspace
terminal's prompt/PTY, and the composition now dies with EVERY caret gesture
(click/chord-verbs/arrows/⌫/⌦/⏎/epoch re-mint — the #428 notes had claimed
place-caret cleared; it never did). Proven: 7 headless drives (real keystrokes +
real `EntityInputHandler` calls + hidden-prompt purity asserts) and live — a
mid-word click landed the bar at the clicked column, a two-line ⌘V grew the window
on real pixels, ⌥E e composed exactly one é.

**#433 — the terminal lane's producer (the Phase-C wedge's first fusion thread).**
`problem_rows`' `terminal` parameter — `&[]` since #327 — receives real rows: failed
command Blocks' file:line refs (the #212/#291 scanners re-aimed any-file via
`links::file_refs`), scanned workspace-wide through ONE walk (all projects ×
`terminal_grids()` × panes — PR-1345's one spelling; VERBATIM roots in the walk, the
producer canonicalizes once per distinct root at its epoch-gated derive) with a
per-pane supersession fold (`latest_failed_indices`: latest COMPLETED block per
(command, pwd) governs, Failure via `exit_status_kind`; a passing rerun retires its
predecessor's rows; a running rerun does not — the recorded delta from the #289
gutter's last-block policy, which keeps its own semantics). The change signal is
SOURCE-OWNED: `terminal_blocks` mints a monotone per-session `block_epoch` (born at
Preexec + finished at Precmd/child-exit, bumped in the one `SessionModel` choke both
`pump` and `apply_hook` funnel through — the `DiagnosticStore::publish_epoch` twin),
and both surfaces' refresh fingerprints are tuple PAIRS ((LSP total, term epoch) for
the panel; (LSP epoch, term epoch) for the mb) — tuple-compared, never cross-lane
summed. ⌘⇧M rows carry the ❯ source glyph (`problem_glyph`); the selection identity
is the 5-tuple with `source`; jumps and the #430 mb materialize ride unchanged
machinery; everything re-derives statelessly (a closed tab's rows vanish at the next
derive). Recorded accepted windows: the cross-session epoch SUM inherits the LSP
twin's close-vs-bump cancellation window (self-healing); terminal text reaching the
mb auto-read lane is bounded by the standing viewer guard chain (the #212 exposure
class, threat-modeled at inspect).

**#434 — runnables: the gutter ▶ spawns a command Block (Phase C opens).**
`marley_syntax::runnables_in` (the 6th node API — parse-only, total, caller-gated
on `language_of == Rust` like #340/#305) walks the tree once per (nonce, version)
memo: `#[test]`-family fns (a contiguous preceding-attribute run whose path is or
ends `::test`; module scope only — the `declaration_list` must belong to a
`mod_item`, so impl/trait bodies and fn-in-fn never mark; `metavariable` names
from error recovery are kind-gated out) carry their in-file mod-chain path;
the top-level `fn main` marks `Main`. The app's pure seam (`runnables.rs`) owns
the toolchain half — `cargo test <path>` (substring, no `--exact` — recorded) /
`cargo run` — grammar and cargo never share a crate. The gutter reserves ONE ▶
mini-cell on every row (uniform width; populated first-segment-only on runnable
rows; its own stop-propagation hitbox — the fold zone and caret keep their
behavior), and BOTH row-prefix mirrors (`code_area_left_px`, the sticky band)
compose the new cell in lockstep (PR-claude-row-prefix-mirrors). The click walks
`idle_workspace_panes` (the ONE #292-shaped spelling, `.0`-sorted, the #40 idle
guard; `rerun-last-failed` re-expressed over it), flashes-and-bails with zero
side effects when nothing is idle, else spawns AS TYPED INPUT (history +
`write_command` + the R39 viewport re-anchor) and `jump_to_pane` reveals+focuses
— the Block model IS the sink (the Zed-degradation the fusion doc maps; the
deviation is the wedge).

Deferred: wrap/folds INSIDE excerpts
(the wedge the #431 stack shape enables), the singleton unification (the editor tab
as a one-excerpt multibuffer — recorded, not scheduled), an mb selection model
(⌘C/⌘X consume but copy nothing until one exists), and the EDITOR-tab ⌘⇧V residual
(the editor block keeps `!shift`; its fall-through still reaches the hidden
terminal — the #432 ladder fix's one recorded sibling, a follow-up candidate).

**#435 — run blocks: identity, rerun, and Terminal-F8 jump-to-failure (the fusion wedge closes).**
The #434 spawn's Block now CARRIES its identity: `Block.run_tag` (opaque `String` — `terminal_blocks`
stays toolchain-free; the codec `encode_run_tag`/`decode_run_tag` lives in `marley_app::runnables`),
staged on the session as a `(tag, command)` PAIR in the same sync region as a SUCCESSFUL
`write_command` and MATCH-BOUND by the Preexec reporting exactly that command — the `staged_prompt`
correlation shape hardened until a mislabel is impossible by construction (a failed write stages
nothing; a racing earlier command mismatches and leaves the stage; a second stage refuses to
overwrite; InitShell wipes — every failure direction yields a PLAIN block, never a wrong label).
The header renders a muted ▶ before the command (both render sites; the POC's 6px gap). The ↻/menu
rerun BRANCHES on the tag: decodable → the D6 re-mint through the codec, recorded in history, a
fresh tag staged (the rerun's block is itself a run block), in the block's OWN pane; plain →
byte-identical #175. Terminal-scoped F8 (`run-jump-failure`; free — the Editor F8 rows are disjoint
contexts, the #265 ⌘F precedent) targets the LATEST failed run block whose identity has NO later
same-tag Success (supersession-aware `latest_failed_run_block` — clear-on-green falls out of the
derive; cross-pane supersession is the recorded v1 bound: blocks carry no cross-session clock, and
the rerun is same-pane so REQ-004's scenario is exactly per-pane), derives its refs per press in
OUTPUT order (`links::file_refs_ordered` — the ONE scan; `file_refs` is now its sorted projection),
cycles + wraps via the `run_jump_cursor` (self-healing: monotone Pane/Block ids can't false-match),
and jumps through the verified-landing `open_and_place_caret` with a NavStack push per successful
jump. Bare F8 YIELDS to the PTY while a program owns input (`is_alt_screen || is_command_running` —
R40's stream-every-key outranks the keymap; the first unmodified Terminal row, so the ladder gate is
new law: see PR-claude-unmodified-terminal-chords-yield-to-the-pty-001).

## Current state

A real **MULTI-CURSOR** plain-text editor: type/backspace/enter, full caret motion, click + drag/shift
selection, clipboard, coalesced undo/redo, and save with a dirty ●, over a faithful un-truncated monospace
render — **all of it at N cursors, end to end** (M19 #296 + #297, proven on live pixels).

**⌘⌥↑/↓** adds a cursor above/below at a sticky **goal column**; **⌘-click** adds or removes one (never the
last); **Esc** — and a plain click — collapses to one. Every edit path acts at EVERY cursor in ONE undo unit:
typing (which reaches the buffer only through the platform text path, `insertText:` → `ime::replace_text` — the
app's single insert mechanism), backspace, paste, cut, Enter's *per-cursor* auto-indent, find-replace. Motion
moves every cursor. A single ⌘Z reverts all N *and restores all N cursors*, and a typed RUN at N cursors undoes
as one word, exactly as it does at one.

**LANGUAGE INTELLIGENCE is in (M20).** A real rust-analyzer runs behind the editor: diagnostics as
squiggles in the shared M18 gutter lane with a footer count and F8 to walk them (#310), **hover** on ⌘K or a
~400ms dwell (#311), **go-to-definition** on F12 with a ⌃- jump-back and a picker for several results
(#312), and **find references** on ⇧F12 — a picker of every usage grouped by file, each row showing its
line's text, ↑/↓ over the references (headers skipped), type-to-filter, Enter jumping (⌃- returns) (#317) —
all over one purpose-tagged request→response path (#308/#309's wire + doc sync + the encoding-aware
position bridge). Since #332 that path's element is a typed `RequestOutcome` —
`Answered(Ok | rpc Err)` or `Abandoned(Timeout | Disconnected)` — delivered EXACTLY ONCE per request:
the 10 s expire enqueues the Timeout abandon (still `$/cancelRequest`ing), and connection teardown
DELIVERS rather than destroys (the queue survives `on_connection_lost`; an answered-then-died request
keeps its answer, and only still-pending purposes sweep as Disconnected). Every consumer arm handles
Abandoned explicitly — the enum replacement makes an `Err(_)` catch-all absorbing an abandonment a
compile error, so no arm can flash its semantic error for a question the server never answered; the
two committed-action arms (rename, resolve) flash an honest "Language server didn't respond", the
rest end silently with their latch cleared. One hazard worth remembering, because it made the whole
line silently useless once: the
pump must `drain()` BEFORE it collects the text to sync, since a host reaches `Ready` inside `drain()` and
the sync's own gates read that phase — collect first and the didOpen ships an empty document, after which
every answer is null (#312 validate; #320 hardens it).

**#413 (M20) closes #332's deferral half: delivery is a property of the OWNING server, never of focus.**
`consume_lsp_responses` drains EVERY host each pump tick — collect-then-dispatch, hosts in root-sorted
order (#396 determinism), per-host queue order within (the abandonment sweeps ascend by id) — and each
outcome dispatches with its OWNING root. Before, only the active root's queue emptied while every host's
wire pumped, so a backgrounded workspace's outcomes deferred unboundedly until switch-back and the ⌘T
fan-out could never complete its merge for non-active roots. Owner-routing is correctness, not plumbing:
rename/resolve read `encoding()` + `lsp_version_for` from the root's host, and a wrong-root lookup MISSES
→ `version_conflict(_, None)` is "no conflict" → the #322 REQ-009 gate fails OPEN (a stale edit applies,
translated with the wrong host's encoding). The per-arm audit (all 12) produced the durable partition: the
8 focused-editor arms (hover / definition / references / completion / prepare-rename / code-action menu /
signature help / inlay) drop non-focused-root outcomes via `outcome_is_for_focused_root` BEFORE any
latch/UI interaction — because uri equality is FILE identity, never INSTANCE identity: nested roots hold
the same file as TWINS (D-OPEN-DEDUPE-SCOPE) with byte-identical uris (`canonical_under_root` passes
absolute paths through — never None), the RootView-global request latches survive project switches, and
independent twin version counters collide at equal counts (the #354 F3 class) — without the guard a
background F12 could execute a jump inside the focused project and a completion accept could edit the
wrong twin (both caught at #413's inspect, never shipped). The drop leaves the latch UNTOUCHED: a
colliding twin re-ask from the focused root may own it now, and every send path overwrites its latch (the
#331 pending-belt covers inlay's send-skip), so a stale latch never wedges a resend. The 4 guard-free arms
(rename, codeAction/resolve, workspace/symbol, formatting) complete in the background by design —
committed gesture / global merge / #354's origin-targeting. The same sweep closed the adjacent
pre-existing twin-routing gap: `apply_one_file` now prefers the OWNING root's instance for a WorkspaceEdit
(the #354 D12.5 class, closed for rename/resolve).

Still missing: no anchors on the caret path (a concurrent edit would stale it). **Soft wrap SHIPPED
(#426, M32)** as the display map's second layer: `editor.soft_wrap` (default OFF; palette "Toggle Soft
Wrap", live + persisted) renders a too-wide line as N display rows — `crates/marley_app/src/wrap.rs`
computes break cells over the ONE phantom-aware `LineLayout` (gpui LineWrapper's RULES on the mono grid:
word-boundary candidates after the first non-whitespace, the CJK any-break clause, hard-break fallback,
continuation indent capped to keep `MIN_TAIL = 8` content cells), and `DisplayMap` composes folds ∘ wrap
(`locate(slot) → (row, segment)`, `slot_at(row, cell)`, `viewport_offset_at` — the caret-segment anchor
every overlay card + IME uses). The rim renders one slot per SEGMENT: display-byte slices with
clipped/re-based highlight ranges, first-row-only gutter numbering, per-segment carets/clicks/squiggles,
the ⋯ fold marker on the last segment. ↑/↓ move by DISPLAY row under wrap (`move_all_vertical_by` — the
injected-step generic; the goal column is a content cell within the segment, reset at toggle), Home/End
stay buffer-line (recorded deferral), and the whole #336 h-scroll set is structurally inert while ON
(scroll pinned 0, content width zeroed → thumb `None`, wheel/follow/park all gated) — the clipper/shift
split REMAINS for OFF, which is byte-identical to pre-#426. **The foundation underneath (#425)**: the
display-map facade owns every buffer-row↔display-row conversion behind typed row spaces
(`marley_text_offsets::{BufferRow, DisplayRow}`, trybuild-pinned distinct); the rim unwraps ONCE
(D-PROJECT-AT-THE-BOUNDARY — interior sites keep buffer-row `usize`); `EditorFrameGeom.first/last` +
`HoverCard.first_row` are typed slots (the F-#352 class fails to compile); the #352 memo lives in the
`display_map()` builder, joined by the #426 wrap memo (nonce, version, anchors, wrap_cols, tab_width,
phantom key). Columns stay OUT of the row stack — `LineLayout`'s #331 two-boundary maps remain the column
authority, and wrap negotiates rows × columns ONLY through `wrap.rs`'s segment slices of that one layout
(anchors resolve through the phantom-AWARE column — the `aware_display_col` discipline; a phantom-blind
column fed into segment math is the F-#352 class on the column axis). **#431 (M33) folded the
multibuffer's excerpt projection into this same facade** — `ExcerptIndex` beside `WrapIndex`, the mb
instance of the ONE `DisplayMap` type with its own typed doors (see "The multibuffer" §#431) — so
fold ∘ wrap ∘ excerpt is one stack with two instances, and the parallel row model the mb carried since
#427 is deleted. Formatting (#314) and find-references (#317) closed out M20's LSP remainder.

**Horizontal scroll (#336, M22) SHIPS — and it is deliberately NOT soft wrap.** The editor had no horizontal
axis at all: `CodeViewState.scroll` is a row index, `clamp_scroll_px` took only `(px, total_rows, cell_h)`,
and a long line's tail was unreachable by any means. H-scroll keeps row↔line 1:1, so it needs **no display
map** — which is the whole reason it could ship in M22's first tranche while wrap waits on B-c.

**The shape: a clipper, not a margin.** `code_row` sizes to its own unbounded text (`whitespace_nowrap`) and
is a flex SIBLING of the gutter, so a negative margin on it pulls the text LEFT ACROSS the gutter. The
structure is `[git lane][gutter][clipper(flex_1, min_w_0, overflow_hidden) → code_row(flex_shrink_0,
ml(-scroll_x))]`. The clipper is the gutter's sibling and never moves, so the gutter stays fixed **by
construction** rather than by arithmetic. `flex_shrink_0` is load-bearing (without it flex squeezes the row
to the clipper's width and there is nothing to scroll), and `min_w_0` is too (without it the clipper's
`min-width:auto` resolves to the full text width). Inspect measured this as a strict IMPROVEMENT over the old
structure, where a long nowrap `code_row` floored at min-content put shrink pressure on the 3px git lane and
the gutter.

**The click needs no math, and that is the interesting part.** `x0` is probed by a canvas INSIDE the shifted
`code_row`, and **taffy folds the negative margin into a child's absolute bounds** (`layout_bounds` returns
taffy's `Layout::location`, with each parent's origin accumulated down the tree). So `x0` already carries
−scroll_x, both terms of `rel = click.x − x0` are window-space, and the scroll CANCELS — `rel` is
content-domain for free. The y axis needs no math for the same reason: `uniform_list` hands the row index
straight to the closure, which is exactly why no click site has ever added `scroll_y`.
**Attribute this to taffy, NOT to `element_offset()`** — that is gpui's separate scroll-offset stack, which a
margin never touches. The distinction is load-bearing rather than pedantic: the comment at the click site is
a landmine marker, and a reader who checks a wrong citation, finds it is about scroll offsets, and concludes
the invariant is bogus would then "fix" the very bug it prevents. (The first draft cited the wrong mechanism;
inspect caught it.)

**Two probes, two domains, on purpose.** The x0 canvas lives inside the shift and rides it (that is what makes
the click free). The clipper's `code_w` canvas lives OUTSIDE the shift and stays screen-domain (a stable
divisor for the clamp). Both feed `cell_w`, so #337's font-metrics work touches both — see
`AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001`.

**The scroll WHEEL's x-delta scales by cell WIDTH (#343).** gpui's `pixel_delta(line_height)` applies ONE scalar
to both axes, so #336's `pixel_delta(cell_h).x` scaled a `ScrollDelta::Lines` (mouse) horizontal delta by the
line HEIGHT — ~1.5-2.4x too fast on a real mouse's tilt-wheel (trackpads send `ScrollDelta::Pixels`, exact px,
and bypassed it, which is why it shipped). The handler now MATCHES `ScrollDelta` and routes through the pure
`h_scroll::wheel_x_px`: `Pixels` passes through as exact px, `Lines` scales by `cell_w` (floored at 1). The sign
stays negated (positive-right); only the `Lines` magnitude changed.

**#337 SHIPPED and the constraint held.** The ADR's demand was "no THIRD source of cell width — one metrics
seam, read by both probes". `appearance.font_size` did not add one: every consumer still goes through
`fallback_cell(self.font_size)` (the one live field), so making the size a setting turned the existing seam
into a *function* of it rather than growing a parallel path. The editor chain needed no change at all —
`geom.x0`/`y0` are MEASURED from a real canvas and `gutter_width()` returns a char count rendered as mono
text, so both absorb a scale change by construction. What #337 *did* fix next door: the TERMINAL's hit-test
was using the fallback's DERIVED `0.6·size` width instead of the measured `em_advance` — wrong already at
13pt beyond ~column 146, and scale-invariant (zoom neither caused nor worsened it).

**The column-domain rule, learned the hard way.** Width is measured with `LineLayout::display_cols()`, never
`display.chars().count()`. `char_width` (UAX#11) makes a CJK glyph 2 cells and a combining mark 0, and the two
agree ONLY on ASCII — so the char count tested clean and then halved every CJK line's scroll range, leaving
its tail permanently unreachable: this ticket's own defect, aimed at exactly the lines that need it. It is not
`col_starts.last()` either, which deliberately stops before an end-of-line #331 phantom (the EOL asymmetry)
and would leave a trailing inlay hint unscrollable. `code_view` owns the column domain
(`AD-claude-editor-offset-column-model-001`) — anything needing a width in cells asks it.

**The follow rides the shared primitive, and is column-aware (#342).** `scroll_editor_to(row, target)` runs the
horizontal follow, NOT the `follow_editor_caret` helper: gestures that place a caret and scroll (⌘D, the #312
go-to-definition landing, the #272 find-next jump) go through the primitive, and hooking the helper left them
scrolling the row in while the column stayed out — the flagship cases, broken. #336 read the PRIMARY caret, which
was wrong for ⌘D (whose vertical targets the newly ADDED cursor); **#342 makes the target explicit** — `target =
Some(offset)` follows a SPECIFIC caret (⌘D passes its added cursor's `head()` so both axes track it), `None`
follows the primary, and `scroll_editor_to_row(row) = scroll_editor_to(row, None)` keeps every other caller
byte-identical. The target is a `CharOffset`, not a raw column, so the DISPLAY column is derived by the shipped
tab-aware `col_of_offset` (the char-vs-cell trap — `AD-claude-two-boundary-maps-for-phantom-text-001`). The
ANALOGOUS `add-cursor-below` (⌘⌥↓) gap — same class (its new caret sorts to the bottom while the follow reads the
top primary) — is tracked as #360; the "unique to ⌘D" recon claim was an over-claim, and the #342 seam is exactly
what #360 reuses.

**The horizontal thumb SHIPPED (#341), display-only.** #336 deferred it on a premise that dissolved at design:
that placement needed a THIRD probe, because `geom.x0` is a WINDOW coordinate that rides the scroll while an
`.absolute()` child of the `relative()` body is positioned body-relative — so the frame publishes no
body-relative code-column origin. #341 DERIVES that origin rather than probing for it: `h_scroll::code_area_left_px`
= the git lane (`GIT_LANE_PX` 3px) + two row gaps + the gutter's `gutter_width` digit columns, reading the SAME
`ROW_GAP_PX` the code row lays out with (the row's `.gap_2()` became `.gap(px(ROW_GAP_PX))` so the constant is
single-source), so the thumb aligns with the code column **by construction** — no new geometry, the values were
already in scope. Its `(left, width)` come from the pure `h_scroll::h_thumb` fraction over the VIRTUAL scroll
extent `viewport_px + max_scroll_x` — NOT `content_px` — so #336's two-cell overscroll slack lives inside the
denominator and `left_f + width_f == 1.0` EXACTLY at max scroll, instead of the thumb sliding off the track's right
edge. It hides when the code fits (`over = content − viewport ≤ 0`, the same "overflow precedes slack" test the
clamp makes) or before the first frame (`code_w == 0`, which lags the synchronously-set `editor_content_w` by a
frame — a real `{view==0, content>0}` state a `view ≤ 0` guard must catch, else a 16px glitch bar at the origin).
The guard compares the UNFLOORED `view`/`over` against 0 so its boundary mutants stay killable (a `≤ 0` over a
0-floored quantity has an unkillable `== 0` twin); the render mirrors #198's terminal scrollbar horizontally,
`MIN_THUMB_PX` floor and all, in the `code_view_body` shim (no input handler — display-only). **Drag-to-scroll
stays deferred** — zero precedent (#198 deferred it too), so a track hit-test → fraction → `clamp_scroll_x` is a
separate follow-up, not this slice. **The road not taken:** `uniform_list`'s
`with_horizontal_sizing_behavior(Unconstrained)` is a real, first-class feature that would give wheel-dx,
clamping and content-width measurement FREE — but it shifts the WHOLE item, gutter included. Honoring the
fixed gutter under it means lifting the gutter out of the row into a synced fixed column: a restructure of the
render path #328, #310, #330 and #331 all ride. If the gutter ever leaves the row, switch to it and delete
this ticket's hand-rolled clamp, wheel and width — that is the roadmap-B2 trade, deferred on the gutter's
account with the reason on the record. Completions
(#313) SHIP: a trigger table → `textDocument/completion` → fuzzy-ranked popup → a TextEdit-faithful accept
(it replaces the typed prefix, as one undo unit). **Rename (#322, M21) SHIPS**: F2 → `prepareRename` → an
inline draft → `textDocument/rename` → a hand-parsed `WorkspaceEdit` applied across every touched file —
open ones through their buffers (⌘Z-reversible), closed ones written to disk. Its `workspace_edit` engine
(`resolve_text_edits` last-to-first + overlap-reject; the closed-file write contained to a canonicalized
root; open-vs-closed routed by CANONICAL path identity so a symlinked root can't misroute a buffer edit to
disk) is the ONE apply path #323 code-actions reuse (and #314 organize-imports will). **File identity
(#319, M20)**: the open seams now STORE one canonical spelling — `marley_project::canonical_under_root`
(canonicalize-with-join-fallback, the same fallback `same_file` compares with) at the loader, the open
probes, session-restore, arrangements, and `open_and_place_caret`'s entry — so the #322 compare-time
discipline became the belt rather than the primary. The consumer families were aligned in the same change:
the host-spawn gate and encoding-by-root lookups compare canonical-vs-canonical (`LspHost::root()`;
`LspHost::absolute` delegates to the one helper), `rel_under_root` derives pathspec/reveal/display rels,
and the ⌘⇧F live-buffer override walk uses the canonical root (a walker's yields inherit the walk root's
spelling — the inspect-caught miss, pinned by test). A symlinked project root can no longer double-open a
file, fail the F12 landing verification, or silently search stale disk text for a dirty buffer; workspace
identity is untouched (D-OPEN-DEDUPE-SCOPE — aliased ROOT spellings stay two workspaces). **Code actions (#323,
M21) SHIP**: ⌘. → `textDocument/codeAction` (the caret's range + the caret-row diagnostics as `context.
diagnostics`) → a picker (quickfix-first, `isPreferred` pinned; a bare `Command` is skipped + counted "N
unavailable"; the picker owns the keyboard, the DefPicker modal contract) → Enter applies an eager `edit`
directly or first sends `codeAction/resolve` (its OWN request slot) and applies the resolved edit — both
through that SAME engine, no second applier. Live-proven end to end against rust-analyzer (⌘. on an
unresolved `HashMap` → "Import `std::collections::HashMap`" → the `use` line lands). The live drive taught
the capability lesson units can't: a result-reading client must STILL advertise `textDocument.codeAction`
(`codeActionLiteralSupport`) or the server withholds the `CodeAction` literals and ⌘. is dead — the
`documentChanges` sibling (`PR-claude-lsp-advertise-client-capability-or-the-server-withholds-the-feature`).
**Signature help (#324, M21) SHIPS**: type `(`/`,` (or ⌘⇧Space) and a card appears ABOVE the caret (the
prefer-above `signature_card_origin`, so it never covers the args nor collides with the below-first
completion popup) showing the call's signature with the active parameter lit; it PERSISTS at a fixed anchor
as you type args and dies when the caret leaves the call's line / Esc / any dispatched action. The #313
park-and-consume shape with a PASSIVE card — it does not own the keyboard (bare ↑/↓ cycle overloads, a
MODIFIED arrow stays the editor's — inspect F-ARROWS). The parse clamps the active indices (OOB→0, LSP
3.17), honours the per-signature `activeParameter`, and normalizes a parameter label given as a string OR
`[start,end]` UTF-16 offsets; the handshake advertises `textDocument.signatureHelp` (labelOffsetSupport +
activeParameterSupport) so a real server returns the offset labels — the #323 capability sibling, though
here it UPGRADES the reply rather than enabling it (rust-analyzer serves signature help either way). Two
caveats worth knowing. The string/comment gate reads
the tree-sitter highlight cache, which refreshes in the RENDER — so the trigger is deferred one pump tick
and, if the render loses the race (or the file exceeds `SYNTAX_SYNC_MAX_LINES`), the gate falls OPEN rather
than suppressing: a popup in a comment that Esc dismisses, never a missing popup where it matters. And the
popup's dismissal POLLS the live editor identity (path/version/caret) instead of hanging off
`dispatch_action`, because inline arrows, mouse clicks, paste and undo never reach the action path — the
same lesson #311's hover card learned (`PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001`).
**Workspace symbols (#325, M21) SHIP**: ⌘T in the editor opens a query picker over `workspace/symbol` — type
a name, land on any symbol in the project (the cross-file counterpart to the in-file outline #304, which is
NOT shipped — there is no `documentSymbol` code, and ⌘⇧O is bound to open-remote (#84), so #304 will need its
own chord). Unlike the passive
signature card, this is the `finder.rs` MODAL (it owns the keyboard: ↑/↓, Enter, Esc) — server-filtered, so
each keystroke PARKS a request (the #313 debounce, one pump tick) that fans out to EVERY Ready host and the
merged answer REPLACES the list; a stale answer for an older query is dropped by a monotonic `gen` key (the
stale key is the QUERY, not the caret — `SymbolQuery{query,gen}`). The pure `workspace_symbol.rs` normalizes
BOTH `SymbolInformation` and `WorkspaceSymbol` (a server that defers a `WorkspaceSymbol.location.range` → the
row lands at line 0 v1; `workspaceSymbol/resolve` is the follow-up) via `path_from_file_uri` + a per-element
`filter_map` (a malformed element → a shorter list, never a panic). Rows render `symbol_kind_glyph` (the LSP
`SymbolKind` table, distinct from completion's `kind_glyph`) + name + a muted container + a right-aligned
`path:line`, CAPPED at `MAX_SYMBOL_ROWS` with an honest `cap_with_tail` "+N more" — and that ONE cap bounds
BOTH the render and the ↑/↓/Enter clamp, so navigation can never address an unshown row (inspect
`BF-symbol-picker-render-cap-diverges-from-nav-001`). Enter reuses the #312 `open_and_place_caret` +
NavStack + encoding-aware landing WHOLE (a failed open flashes and moves nothing; ⌃- returns), resolving the
encoding from the deepest host root that prefixes the chosen path (multi-root). Two lessons the modal taught:
opening it must first DISMISS a live completion popup or the lower overlay steals the keys
(`BF-modal-open-over-live-completion-steals-keys-001`), and ⌘T is Editor-SCOPED, shadowing the global ⌘T
new-tab (the #265 ⌘D pattern) — symbols in the editor, a new tab in a terminal. The handshake advertises
`workspace.symbol.symbolKind` (the full 1..=26 set) — the #324 posture: it UPGRADES the reply's kind set,
it does not enable the feature (rust-analyzer serves `workspace/symbol` without it).
**Project-wide content search (#326, M21) SHIPS**: ⌘⇧F in the editor greps file CONTENTS across the whole
workspace (roadmap B7 phase 1 — originally a READ-ONLY results surface; phase 2 SHIPPED across #427/#428/#429:
the results materialize as an editable multibuffer with one-gesture Replace All — see "The multibuffer" above). It's the cross-file complement to ⌘P (file NAMES) and the one-buffer editor find. A modal
picker (the ⌘P finder recipe, owns the keyboard) shows every matching line grouped by file with the match
span highlighted; ↑/↓ walk matches, Enter reuses #312's `open_and_place_caret` + NavStack (⌃- returns; the
match's `col` is a CHAR offset so the caret maps through the bridge as `Utf32`, no host-encoding lookup).
The pure `marley_project::search` engine — `search_lines(text, needle, opts)` → `(row, col, len, preview)`,
literal substring with a case + a whole-word toggle (ASCII-fold, length-preserving so `col` stays exact;
regex + Unicode fold are follow-ups) — is gpui-free and tool-shaped (a future `workspace.search` MCP
read-tier). The walk adopts ripgrep's **`ignore`** crate (gitignore-aware; its whole transitive closure was
already in `Cargo.lock`, so it added exactly one crate — the D-WALKER decision) as a LAZY iterator, still
belt-filtered by `should_skip`, with a stat-first size gate (a huge non-gitignored file never pre-allocates —
inspect mirrored `load_code_view_state`). The search runs OFF the UI thread (the #274 syntax-worker
precedent: worker + channel + pump-drain); a new query CANCELS the old walk via a shared `AtomicU64`
generation, and any OPEN file is searched via its LIVE buffer text — a rename you just typed is findable
before ⌘S (the correctness heart). The app-pure `editor_search` glue (`accept_gen`, `locate_match`,
`admit_matches`, `visible_matches`, `footer_summary`, the caps + honest "+N more" tail, render cap ==
nav clamp) carries the mutation surface. A spawn-once worker DROPS its handle on a send error so the next
⌘⇧F respawns (`BF-spawn-once-worker-never-respawns-001`), and ⌘⇧F is Editor-scoped (the #325 / #265
shadowing pattern; the global ⌘⇧F toggle-forge row it shadowed retired at #411 — the scope stays right).
**Problems panel (#327, M21) SHIPS**: ⌘⇧M in the editor opens a list of EVERY error and warning
rust-analyzer knows across the WHOLE workspace — including files you don't have open — grouped severity-first,
jumpable. F8 (#290/#310) walks the FOCUSED file + the footer counts it per-file; this is the workspace view
(roadmap: the read-only list — its EDITABLE form shipped at #430 as the diagnostics multibuffer, see "The multibuffer" above). The
missing primitive was enumeration: the #310 `DiagnosticStore` exposed only `for_path`, so #327 adds a pure
`iter()` (+ `total_len` for a cheap fingerprint) and an `LspHost` passthrough, then the pure `problem_rows`
aggregates EVERY per-root host's store — plus a wired-but-empty terminal-lane producer (the #310 two-producer
doctrine lifted workspace-wide; the workspace terminal-scan is the D-TERMINAL follow-up) — sorting
`(severity, path, line)` via the `Severity` `Ord` (Error first) and capping with the honest "+N more" tail.
`ProblemRow` is gpui-free + tool-shaped (a future `workspace.problems` MCP read-tier). The ⌘⇧M finder picker
(a NEW Editor-scoped chord that SHADOWS NOTHING — the chord was free, unlike #325/#326) renders a
`severity_glyph` (colored by the gutter danger palette) + `path:line` + message; Enter reuses #312's
`open_and_place_caret` + NavStack and opens even a CLOSED file (the panel's whole point), resolving the OWNING
host's NEGOTIATED encoding (LSP diags are encoded columns, NOT #326's char-offset Utf32 — the #325 posture,
load-bearing on a non-ASCII line). The panel RE-DERIVES live on the pump when a `publishDiagnostics` lands
(gated on the cheap `sum(diagnostics_total())` fingerprint — a same-count content change is a documented,
self-healing miss the footer backstops), and the selection is kept by `(path, line, character, severity)`
IDENTITY, not index, so a diagnostic clearing above it never teleports the cursor (inspect caught that a
`(path, line)`-only key collides for two diagnostics sharing a line —
`BF-live-list-selection-identity-key-not-unique-001`). The `cockpit_status` footer gains a "N workspace" tier
when the workspace count differs from the focused file's.

**Git gutter (#328, M21) SHIPS**: the editor gutter grows a second lane — a thin colored bar per edited line,
added (success) / modified (accent) / a deleted-run caret (danger), computed against `HEAD`. Git previously
surfaced only as the ⌘⇧D diff overlay and the changes/commit panel; the editor knew nothing per line. Route A
(chosen over an in-process `imara-diff`, the deferred route-B upgrade): project git's OWN unified diff. The
pure `git_diff::gutter_marks_from_hunks(&FileDiff) -> Vec<(row, GitMark)>` walks each hunk's `+`/`-`/context
body against a 0-based NEW-side row counter (`parse_hunk_new_start` reads the `@@ +c` header start), and
`flush_segment` classifies each change SEGMENT between context lines — a pure `+` run → `Added`, a
`-`-then-`+` replaced run → `Modified` on the new rows, a pure `-` run → a `Deleted` boundary marker between
rows (the LSP zero-width convention adapted). Zero new deps — it reuses the shipped `parse_diff`; malformed
input → no marks, never a panic. The HEAD source is a READ-ONLY `git diff HEAD -- <relpath>` spawn
(`marley_command::blocking`, the `git_working_diff_in` argv posture — no git writes, path as a single argv arg
so no injection); an untracked file (`git status --porcelain` `??`) has no HEAD blob → every line `Added`.
Adopting the reference decoupling discipline, the marks live in their OWN `HashMap<PathBuf, Vec<(row,
GitMark)>>`, never a field on the open-file entry, cached on `(path, git_marks_gen)` — the generation bumps
on save, external-change reload, AND an in-app commit (inspect caught the commit case:
`BF-in-app-commit-stale-derived-cache-001`), so per-keystroke stays inert and an idle frame recomputes
nothing (the saved-state cadence, not a per-tick `git rev-parse` — a dirty buffer shows the last SAVED
state's marks; the external-terminal HEAD-move is a named v1 staleness limitation, self-healing on the next
save/switch). #412 closes the store's lifecycle: `RootView::release_editor_views` scrubs a dropped path's
entry — and clears a `git_marks_key` naming it — at the last same-path instance drop (census-guarded like
`editor_folds`; the key clear is load-bearing — no activation bump exists, so a dangling key would
early-return the refresh over the scrubbed map and blank a reopened dirty file's gutter). The render is a
fixed 3px bar child BEFORE the number cell, `egit` captured before the
`'static` `uniform_list` closure (the sibling of the #289/#310 `ediag` diagnostic-tint capture), COEXISTING
with the diagnostic number tint — two independent producer lanes (the #310 one-lane rule governs diagnostic
PRODUCERS, not git). v1 cuts: hunk revert / stage-from-gutter, blame, the deleted-hunk block render, an
index-vs-HEAD toggle.

**Expand/shrink selection (#329, M21) SHIPS — and with it marley_syntax's FIRST node-range API.** ⌃W grows
the selection out along the tree-sitter node ancestry (identifier → call → statement → block → fn); ⌃⇧W walks
it back to exactly where it started. Until #329 marley_syntax exposed highlight SPANS only — the
`tree_sitter::Tree` was a private field of `HighlightSession` — so the ticket doubles as the FOUNDATION #305
folding and #330 sticky-header reuse. The pure `enclosing_ranges(&HighlightSession, byte_range) ->
Vec<Range<usize>>` reads the held tree: `named_descendant_for_byte_range` finds the smallest named node
covering the range, then a `.parent()` climb collects each ancestor's `byte_range()` to the root, deduping
adjacent identical spans (tree-sitter nests `expression_statement == call_expression` constantly). It needs NO
`is_named` filter — a tree-sitter parent is ALWAYS a named rule node because anonymous string-literal tokens
are always leaves (a grammar-agnostic structural invariant; the guard would only be an unreachable branch). It
stays in `lib.rs` at cov/MSI 100 (a plain value-returning Node walk, unlike the `QueryCursor` chain that
forces the `parse.rs` exclude). The pure `SelectionLadder { anchor, rungs, pos }` (marley_app) is the state:
`grow` advances `pos` toward the root, `shrink` retreats toward the anchor and at `pos == 0` restores EXACTLY
the original selection (the remembered anchor is what makes shrink correct); the rung equal to the anchor is
dropped so the first grow always visibly expands. **D-ROUTE (a design refinement of the ticket's hybrid): v1
computes the ladder SYNCHRONOUSLY via a throwaway `HighlightSession` over the current text for ALL file
sizes** — the Explore pass established the parsed `Tree` never crosses the worker→app boundary (the cache
carries only per-line spans), and a one-shot parse on a deliberate ⌃W gesture is the same cost class as the
existing per-keystroke sync highlight; the worker-async round-trip (the `(nonce,version)`-guarded channel) for
very large files is a named follow-up. The ladder is built once per gesture and cached keyed on `(nonce,
version)`; an edit (version bump) or a caret move (the live primary no longer equals the cached rung)
invalidates it, so the next grow rebuilds (the live-identity family). tree-sitter speaks BYTES and `Selection`
speaks CHAR offsets, so the app converts through the rope (`char_to_byte`/`byte_to_char`, emoji-safe). The
⌃W/⌃⇧W chords are Editor-scoped and both FREE (a terminal tab's ⌃W still deletes-word; ⌥↑↓ was avoided —
#300 move-line owns it). Multi-cursor v1: the PRIMARY grows and `SelectionSet::single` collapses the others
(#307). §20: tree-sitter is published-API reuse (MIT); the AST-ancestry BEHAVIOR is a universal editor
affordance, source unread.

**Sticky context header (#330, M21) SHIPS — the #329 foundation's first consumer.** Scroll into the middle of
a long function and the enclosing `fn`/`impl`/`mod`/`trait` header stays pinned at the viewport top, clickable
to jump back. This is the editor twin of the terminal's `sticky_block` (#185) — same UX, same overlay recipe,
different source of truth (syntax scopes instead of block extents). The pure `all_headers(&HighlightSession)
-> Vec<Range<usize>>` collects every header-bearing node (the `Node::kind()` table:
function/impl/mod/trait_item for Rust v1; a body-less `function_signature_item` is NOT a header) in document
order. **The walk is ITERATIVE (a `TreeCursor`), and that is load-bearing:** inspect proved recursion here
recurses the FULL tree depth — every nested block/paren/expression level, not the header nesting (one `fn`
around 2000 nested blocks recurses 2000 deep to find ONE header) — measured to SIGABRT at ~16k levels on the
8 MiB main stack. That is an uncatchable abort, on the UI thread, on every edit (sticky is default-on), so a
machine-generated file would take the app down; tree-sitter's own parser is iterative for exactly this reason
(a 2000-level regression test pins it). The pure `sticky_rows(headers, first_visible_row, max_depth)` decides
the pin: a header pins while the viewport top has scrolled strictly PAST its own row (`header_row <
first_visible_row`) and is still within its span (`<= end_row`) — a header AT the top does NOT pin (the
`sticky_block` no-double-render edge), and it releases one row past the closing brace; nested scopes stack,
and past `max_depth` (2) the INNERMOST are kept (the immediate context beats the outer module).
**The cadence is what keeps scroll free:** the header list is cached per `(nonce, version)` — ONE throwaway
reparse per EDIT (the #329 sync route), never per scroll frame — and the pure filter runs each frame over the
cached spans (`first_visible_row` = `editor_geom.get().first`, the value `uniform_list`'s range callback
already records; NOT scroll-y math). The band is an absolute overlay above the list (never inserted into it —
that would fight `uniform_list`'s row math): `surface` bg + a bottom hairline so code scrolls visibly under
it, each row through the same `line_layout` + syntax-cache highlight path, gutter-aligned, and
`block_mouse_except_scroll` — NOT `.occlude()`, which would make the band a scroll dead-zone (#185's own
inspect lesson). It renders nothing when nothing pins, so it can never eat a click. A click places the caret
+ pushes the NavStack (inspect caught that `open_and_place_caret` does NOT push — every sibling jump pushes
explicitly). **D-SCROLL resolved to a no-op:** every scroll-to-row uses `ScrollStrategy::Center`, so a jump
target always lands mid-viewport, never at the top edge under the band — no `sticky_height` inset is needed
(a hypothetical top-reveal strategy would need one). `editor.sticky_header` (default on — the first `editor.*`
key) + a "Toggle Sticky Header" palette verb. §20 N/A: composition of Marley's own parts (the #329 tree
access + the #185 overlay) over published tree-sitter reads; VS Code sticky-scroll is the observed behavior
reference (its region default informs `max_depth = 2`), and Zed's `BlockMap` display-stack is deliberately not
used — the overlay shape is Marley's own.
**LSP inlay hints (#331, M21) SHIP — THE display-map ticket, deliberately last in the batch.** `let s =
String::new()` renders as `let s: String = String::new()`; the muted `: String` is rust-analyzer's, not the
file's. Everything before this rendered a strict 1:1 buffer→display mapping — `col_starts[i]` is the display
column of the i-th BUFFER char, and the caret pixel, the click inverse, drag, selection rects and #310
squiggles all ride it. A phantom is text that **occupies display columns but belongs to no buffer char**, so
the mapping itself had to learn about it. The whole risk is contained in one pure seam:
`line_layout_with_inlays(line, tab_width, &[Inlay{char_idx, text}])` emits each phantom into `display` and
advances the column while pushing NO `col_starts` entry. `line_layout` is now the empty-slice wrapper, and a
property test pins them byte-identical — **hints OFF is provably the exact pre-ticket path**, not merely a
similar one.

**Two invariants govern it. Naming the first was necessary and not sufficient — that is the lesson.**

1. **One column domain.** `display`'s cell accumulation and the values in `col_starts` must agree, because
   `cols_to_bytes` re-derives a column by WALKING `display` while `col_of_offset` READS one from `col_starts`.
   They agree only because a single pass builds both. Advancing `col` for a phantom BEFORE the anchor char's
   `col_starts.push(col)` keeps the map in SCREEN columns, so every rider stays correct with no signature
   change. The trap is the opposite: a phantom-BLIND `col_starts` forks the domains and silently shifts every
   span and pixel left by the phantom's width.
2. **A span describing CODE hugs the code; a span describing a CARET RANGE tracks the caret.** One domain
   still leaves two BOUNDARY semantics, and inspect found the gap: `col_of_offset` is the CARET map — by
   design it sits AFTER a phantom anchored at that offset (the caret goes on the code side of a hint). That
   makes it the right END for a selection band, which must stop exactly where the caret it follows renders, or
   it detaches from that caret by the phantom's width. It is the WRONG end for a syntax token or a diagnostic
   underline, which describe the code itself: a phantom anchored at the span's one-past-end char falls INSIDE
   the span, and the token paints over its own hint. Hence `col_ends` + `LineLayout::col_of_span_end` — the
   END-side map, built in the same pass — used by `raw_span_to_display_bytes` and the #310 squiggle, while
   `row_selection_cols` deliberately keeps `col_of_offset`. **Worked example** — `let x = 1;` with `: i32`
   anchored at char 5: the squiggle for `x` ends at column **5** (it hugs `x`, not `x: i32`) while the
   selection for `x` ends at column **10** (the caret for offset 5 renders there). Both are correct. A future
   refactor will want to unify these two accessors; it must not — see
   `AD-claude-two-boundary-maps-for-phantom-text-001`.

**The reachability that made it real, not theoretical:** rust-analyzer's chaining hints are on by default and
anchor at END OF LINE, where a literal *does* have a syntax span ending exactly at the anchor — `let n =
"hello"` mapped its `Str` span across the phantom and painted the hint string-green.

**The EOL asymmetry.** "The caret sits on the code side" is not uniform: mid-line the code is to the phantom's
RIGHT (emit, then push the column); at EOL there is no char to its right, so the code is to its LEFT (push,
then emit). Treating them alike parked the End caret out past a chaining hint and put an empty line's caret at
column 1.

**Render.** `TokenKind::Hint` → `colors.muted` is REQUIRED, not cosmetic: `styled_slices_with_marks` drops a
slice that is `Plain` + unselected + unmarked, so an untagged phantom would fall through to the base
foreground and read as real code. Hint spans go FIRST, ahead of the code spans — the splitter takes the first
containing range, so ordering makes the phantom's cells win unconditionally instead of depending on another
producer's capture table (which is what silently protected it before: identifiers map to `Plain` and are
dropped, an accident #316's palette would have ended).

**Cadence + the two gates.** `RequestPurpose::InlayHints` rides the #311 recipe and the ONE drain, keyed by
the VIEWPORT RANGE rather than a caret — hints are a property of a region and the request fires on scroll as
well as on edit. `workspace/inlayHint/refresh` (the wire's first server-initiated request beyond the #308 set)
replies `AckNull` and raises a latch the pump drains into a re-fetch. The gates exist because **an error reply
writes no cache**, so `served` stays false and an ungated ask repeats every round-trip forever: the send is
gated on `language_id_for` (the ONE table #309 syncs by — a file it declines was never `didOpen`'d, so asking
can only error) and on the advertised `inlayHintProvider`.

**The served-check's own forever-loop (#401).** The SUCCESS path had a sibling of the error loop: the cache's
row range stored an INCLUSIVE `want_last` (clamped to `len_lines − 1`) in the `Range.end` position while the
check compares a genuinely EXCLUSIVE viewport end — with the file bottom on screen `end_row == len_lines`
could never be `≤ len_lines − 1`, so a short/fully-visible file re-sent the byte-identical request once per
round-trip forever (found at #352's inspect, preserved there for byte-identity, deleted at #401). The fix
stores the exclusive end at the ONE write site (`apply_inlay_response`); the wire params still carry #331's
pinned inclusive pair (the strict-LSP boundary-row residual is recorded in #401's Out). Same ticket, the
adjacent hole the fix would have made sticky: a disk reload re-mints the nonce but RESETS the version epoch,
and `InlayKey` carries no nonce — a raced pre-reload answer collides numerically on a never-edited file; the
reload now drops the cache + in-flight key beside its #328 git-marks invalidation (rule: version-epoch resets
must invalidate every numeric-keyed cache). The pin drives the REAL mint/send decision headless — a
process-less host replayed to `Ready` through the real `Lifecycle` (`send_body` tolerates the missing child),
the first headless coverage of the send decision itself.

**The poll-vs-event latch asymmetry — worth internalizing before adding the next consumer.** The send-skip is
load-bearing here (the pump re-evaluates every tick, so without it a request storms while the cache is empty).
But a timed-out request drops its purpose and delivers NO response, so a latch-only skip keeps matching a key
whose answer will never arrive. Hover/completion/signature are immune by accident — each keystroke mints a
fresh key that overwrites the stale latch — but this refresh is keyed on an UNCHANGING viewport, so it went
dead for exactly as long as the user sat reading, which is precisely when a cold server exceeds the 10s
timeout. `LspHost::has_pending_inlay()` asks the pending table, the one source of truth for in-flight-ness, so
an abandoned request self-heals into a retry (one per timeout window). **#332 SHIPPED the general fix**: every
abandoned request now delivers a typed `Abandoned(Timeout | Disconnected)` outcome through the ONE drain, so
the arm itself clears the latch — the host query stays as the belt (spec D4), and the wedge is dead by
construction (proven end-to-end headless: mint → 626 real pump ticks → typed delivery → re-send). The obvious
version — a synthetic `Err` — remains recorded as wrong: three shipped features flash semantic errors on
`Err`, so they would toast ten seconds after the user's keypress; the new enum makes that absorption a
compile error rather than a code-review catch.

**Fixed (#333) — the fallback lexes RAW, and both arms share one remap:** the former narrow limit (the
fallback hand-lexed `layout.display`, phantoms included, so a hint carrying `"` or `//` could bleed lexer
state rightward on a row the memo can never serve — a Rust file with an exotic line separator: the memo
splits rows on `\n` only while ropey also breaks on bare `\r`/FF/NEL/LS/PS; the #305 fold-ellipsis marker is
a second, race-free phantom producer on async-window tail rows) is closed. The fallback is
`code_syntax::highlight_display_ranges`: hand-lex the RAW line, then map through
`code_view::spans_to_display_bytes` — the ONE raw→display remap that the primary tree-sitter arm, the
main-row fallback, and the sticky-header fallback all ride (any future span producer maps through it too).
Phantom text cannot seed lexer state by construction; hints-first ordering stays as belt-and-braces. One
pinned deliberate difference vs the old display-lex on hint-free rows: a zero-width combining mark
immediately after a token is absorbed into the token's span (`cols_to_bytes` never makes an empty cell
window a boundary — the grapheme cluster stays one styled run, agreeing with the primary arm since #268).

**§20:** clean-room from the PUBLISHED LSP 3.17 specification (`textDocument/inlayHint`,
`workspace/inlayHint/refresh`, `InlayHint`/`InlayHintLabelPart`/`InlayHintKind`, padding) — exactly the
#308–#313 posture. Zed's `InlayMap` is an ARCHITECTURE reference only: the deconstruction names its "empty
input, non-empty output" layer-contract CONCEPT and flags its 209KB machinery as "defer wholesale"; Marley's
per-line render admits one pure function instead, and the source is UNREAD. VS Code supplies the OBSERVED
caret/click behavior (the caret lands on the code side; a hint is not selectable).

Tab still indents only the PRIMARY cursor's line span
(multi-cursor block indent is #299/#300) — but it **carries** the other cursors through the edit rather than
destroying them. One nuance: the editor uses
ropey's line model throughout (render + caret self-consistent), which breaks on more than `\n` (bare `\r`,
VT/FF/NEL/…), so on an exotic file its line count can exceed `text.split('\n')` — a cosmetic difference only
against the #246 read-only split pane, which splits on `\n`.

**Bracket-match highlight (#340, M22) SHIPS — the third `marley_syntax` node API + a mark render.** The caret
on or beside a `(`/`[`/`{` lights both halves of its pair; **⌘⇧\ Go to Matching Bracket** hops between them.

`matching_delimiters_in(src, byte_pos) -> Option<(Range<usize>, Range<usize>)>` is the third node API after
#329's `enclosing_ranges` and #330's `all_headers`, and the first that PARSES `src` itself rather than taking a
`&HighlightSession` — the app holds no cached tree, and bracket-match needs only the tree, not the highlight
query (parse-only, ~4 ms/2000 lines measured; the cached-tree variant is the **#349** follow-up). It is
**adjacent-only, not enclosing** (REQ-003): `descendant_for_byte_range(pos, pos)` must land ON a delimiter
token, then the parent's first + last children are the pair — a caret in open code lights nothing, unlike a
walk to the smallest enclosing bracket node (which would light a function's body braces from everywhere inside
it; the first implementation did exactly that and a probe caught it against the acceptance criterion). The probe
prefers the byte BEFORE the caret, then AT it (D-BEFORE-THEN-AT — typing `)` lights its pair behind the cursor).
Two guards make it total and honest: `set_language(&lang).ok()?` + `child_count().checked_sub(1)?` keep it
panic-free with no dead `return None` line (coverage on the non-excluded `lib.rs` punishes those), and
`first.is_missing() || last.is_missing()` rejects an error-recovery MISSING delimiter — for an unclosed `(`,
tree-sitter inserts a zero-width `)` node whose `kind()` is still `")"`, so a `kind()`-string match alone would
light a phantom (an unmatched bracket lights nothing — the spec cut unmatched-error tinting).

**Why the tree is truth (spike-proven, not asserted):** a `(` inside a string is a `string_content` node and
inside a comment a `line_comment` — `descendant_for_byte_range` lands inside that leaf, never on a `"("` node —
so the classic "a bracket in a string highlights" false-positive class genuinely cannot exist. This is the whole
reason bracket-match is Rust-only-via-the-tree rather than a text scan; #315 extends it per grammar for free.

**Render as a MARK, not a token.** The two delimiters join the SHIPPED `styled_slices_with_marks` marks channel
as a new `MarkTier::Bracket`, the LOWEST tier (Current > Match > Bracket — a find band or the selection wins any
cell they share, so bracket-match never fights find). The pair arrives as whole-DOCUMENT bytes; the render
localizes each delimiter to its row (`span - char_to_byte(line_start(row))`, keeping only the delimiters on that
row) and maps through `raw_span_to_display_bytes` — the CODE-side/#331 map, because a delimiter IS code and must
hug the code (the caret-tracking cols path the find bands use would smear the tint onto an inlay hint anchored at
the delimiter boundary). On ASCII, bytes/chars/columns coincide, so a multibyte-and-tab fixture is the only test
that proves that seam is width-aware (the #336/#339 blind spot).

**The cadence** is the app-side `refresh_bracket_match`, memoized on `(nonce, version, caret_byte)` and gated on
`language_of(path) == Rust` (mirroring `refresh_syntax_cache` — REQ-008: `matching_delimiters_in` always parses
as Rust, so the language gate at the caller is what keeps bracket-match Rust-only; without it a `.json`/`.py`
would light brackets under Rust's grammar). A static caret costs nothing; a caret move alone invalidates
(unlike #330's version-only key). **⌘⇧\ pushes NO NavStack** (D-NO-NAVSTACK — an intra-expression hop, not a
navigation the ⌃- stack should record); the one wart is a caret parked exactly between an empty `()`, where the
adjacent delimiters leave nowhere to jump (named, not hidden).

**M22 #349 — the cadence stops reparsing on a large file.** `matching_delimiters_in` parses `src` from scratch
each call — the profiling spike (release) measured that at ~3.4 ms/1.8k lines, ~8.6 ms/4.5k, ~19 ms/10k, and 10k
crosses a 60 fps frame, so a held arrow on a big file dropped frames. The fix caches the tree the syntax WORKER
already parsed for spans: it hands its `Tree` back on `SyntaxResp` (a `HighlightSession::tree()` accessor + a
`ts_tree_copy` clone — a refcount bump, so the worker keeps its own for the next incremental parse; `Tree: Send`
crosses the channel), and the app caches `tree_cache: Option<(nonce, version, Tree)>` in the pump's accept-gate,
mirroring `syntax_cache`. `refresh_bracket_match` then reads the cached tree on an EXACT `(nonce, version)` hit via
the factored pure `matching_delimiters_from(&Tree, byte_pos)` (microseconds, no reparse, no text materialization),
and on any miss falls back to `matching_delimiters_in` — byte-for-byte the prior behavior (a file below the
async threshold, or an edit that bumped the version before the worker re-sent). **The exact-AND key guard's two
fields are BOTH load-bearing**: a reload resets a buffer's version to `0` *and* re-mints its nonce, so a
version-only guard would false-hit a just-reloaded v0 buffer and a nonce-only guard would false-hit the pre-edit
tree — a stale tree is a silent wrong-highlight, so the guard is exact and the fallback always correct. The
`tree_sitter::Tree` re-export (`marley_syntax::Tree`) keeps tree-sitter confined to `marley_syntax` — the app holds
the tree opaquely and only passes it back.

**M22 #363 — the deferred half lands: #329/#330 read the cache too.** #349 stopped short of the two non-per-caret
consumers; #363 routes them through the same `tree_cache`, a verbatim transplant of the #349 factor + hit-or-fallback
×2. Two more pure parse-free siblings — `enclosing_ranges_from(&Tree, byte_range)` (the #329 ancestry walk) and
`all_headers_from(&Tree)` (the #330 header collect) — are factored out of the session-taking originals, which now
delegate `session.tree.as_ref().map(_from).unwrap_or_default()` (behavior-identical). `step_selection_ladder` (⌃W)
and `refresh_sticky_headers` (scroll) then read the cached tree on an exact `(nonce, version)` hit → `_from` (no
reparse), else the byte-identical session-parse — the same exact-AND guard as `refresh_bracket_match`. The sticky
reroute is the one non-trivial part: its `all_headers` call lived INSIDE a `self.active_editor().map(|s| …)` closure,
so the `&self.tree_cache` read is un-nested ABOVE the row-mapping (the match yields an owned `Vec<Range>`, releasing
the borrow before the row-map's fresh `&self` — mirroring `refresh_bracket_match`). Both reroutes sit inside the
already-`mutants::skip`'d caller fns → no new mutation surface; the pure `_from` factors carry cov/MSI 100. One
inspect lesson: a `_from == wrapper` equivalence unit is TAUTOLOGICAL for the factor's OWN mutants (both operands
route through `_from`), so each equivalence unit also asserts `_from` against a hardcoded literal
(`PR-claude-delegation-equivalence-test-tautological-001`; latent in #349's `matching_delimiters_in` too). The three
cache readers are now `refresh_bracket_match`, `step_selection_ladder`, `refresh_sticky_headers`.

**The M1 caller-side language gate is a CLASS, closed across every structural consumer (#350, M22).** The
rule bracket-match established — a language-specific pure primitive that *always* parses as Rust
(`matching_delimiters_in`, `enclosing_ranges`, `all_headers`) cannot self-gate, so its CALLER must check
`language_of(&path) == Language::Rust` — applies to every app-side seam that builds a throwaway
`HighlightSession`. Two shipped before the rule existed: `step_selection_ladder` (⌃W/⌃⇧W, #329) and
`refresh_sticky_headers` (#330) each parsed Rust unconditionally, so a `.json`/`.py`/`.toml` file got
walked as a Rust **error-recovery** tree (a ladder over plausible-but-wrong rungs; a sticky header pinned
from a hallucinated `function_item`). #350 back-fills both, verbatim to `refresh_bracket_match`, so the
three gated callers are now `refresh_bracket_match`, `step_selection_ladder`, `refresh_sticky_headers`.
The two back-filled seams are **asymmetric on purpose**: the ladder needs no explicit clear — its
early-return precedes every read of `selection_ladder`/`selection_ladder_at`, and a ladder cached from a
Rust file is already inert on a non-Rust file via the `(nonce, version)` validity check, so it is never
consulted — whereas sticky `.take()`s its cache (dropping pinned headers in one repaint on a Rust→non-Rust
switch) because `sticky_rows` reads `self.sticky_headers` on **every** pump frame and an uncleared cache
would keep rendering. When #315's language axis reaches the structural APIs, all three gates widen together
(the primitive stops being Rust-only; the caller passes the file's real language) — the gate is the seam
that makes that a one-line change per caller, not a re-audit.

**Move / duplicate lines (#300, M19) SHIP — a line-REORDER seam beside indent's line-EDIT seam.** ⌥↑/⌥↓ move
the caret's line/block; ⇧⌥↑/⇧⌥↓ duplicate it (the copy on the pressed side). Multi-cursor blocks, cursors
carried, one undo unit.

The pure home is a NEW `crates/editor/src/line_move.rs` (distinct from `indent.rs`'s indentation, but reusing
its `LineEdit`/`touched_rows` + `movement::VDir`): `move_lines(buffer, set, dir) -> Option<(Vec<LineEdit>,
SelectionSet)>` and `duplicate_lines(...) -> (Vec<LineEdit>, SelectionSet)` — each returns the edit list AND
the CARRIED cursor set. The app shim (`apply_line_reorder`) mirrors #299's grouped-line-edit idiom
(`begin_undo_group` → raw `edit()` BACK-TO-FRONT → `end_undo_group`) with ONE substitution: the seam's carried
set replaces #299's `rebase_selections` call.

- **D-CARRY-NOT-REBASE** — `rebase_selections`' clamp (a position inside a removed span → the span's shifted
  start) would collapse a moving cursor to its line start, because a move's removed span IS the row the cursor
  sits on. So the seam computes the carry itself. (The mirror-image decision, documented against #303's
  deletions, where that same clamp is exactly RIGHT.)
- **The carry is CHAR arithmetic** — `± (gap line chars + 1)` for a move; on ASCII bytes==chars, so a multibyte
  gap line is the only separator (the #336/#339 coincidence).
- **Endpoint attribution by OFFSET SPAN, not raw row** (the inspect lesson, `AD-claude-carried-selection-...`):
  a "full lines" selection's HEAD is at col 0 of the row PAST the block — a row `line_span` carves OUT — so a
  row-based block lookup misses it and silently distorts the carried selection while the TEXT stays correct.
  Attribute by `[line_start(b0), line_start(b1+1)]`, INCLUSIVE of that boundary for a move + duplicate-up,
  EXCLUSIVE for duplicate-down (where `rebase_through` already carries the boundary onto the copy).
- **D-LEADING-BLOCK-GATES** — the op is all-or-nothing: `None` (a clean no-op) when the leading block is at the
  edge, else every block moves by one; blocks are ≥1 gap apart so the no-collision proof is trivial.
- **The phantom-row clamp** — a caret on ropey's trailing empty line acts on the last CONTENT line, so ⌥↑
  there cannot drop the file's trailing `\n`.
- **The undo group is non-cursor-anchored** (`begin_undo_group(before, false)`), so the next typed char is its
  OWN undo unit — the #338 corruption class is impossible for a line op by construction.

**Destructive ops (#303, M19) SHIP — the deletion halves of the #257 word/line motions.** ⌥⌫/⌥⌦ delete the
word left/right, ⌘⌫/⌃K to line start/end, ⌘⇧K the whole line; plus plain forward-delete. The pure home is a
NEW `crates/editor/src/delete.rs` (a sibling to `line_move.rs`): `DeleteOp` (5 variants), `delete_range_for(op,
buffer, sel) -> Option<Range<CharOffset>>`, and `delete_edits(buffer, set, op) -> Option<(Vec<LineEdit>,
carried SelectionSet)>` — the same `(edits, carried)` shape `move_lines` returns, so `apply_delete` is a
one-word swap on the #300 `apply_line_reorder` grouped-edit shim.
- **D-MOTION-IS-THE-RANGE** — the deleted range comes straight from `move_word_left/right` / the line
  boundaries; motion and deletion share one boundary function, so they can never disagree. (The readline
  `KillWordBack` already reused `move_word_left` as a deletion endpoint — the same idea, now in the editor.)
- **D-SELECTION-WINS** — a non-empty selection is what every op deletes (decided in the pure fn, before the op
  match).
- **D-REBASE-IS-CORRECT-HERE — the same clamp #300 BANNED.** `#300`'s move-line couldn't use
  `rebase_selections` (its clamp collapses a moving cursor to a line start — D-CARRY-NOT-REBASE, it carries
  manually). A delete *wants* that clamp: a caret inside the span it just deleted belongs at the span's start.
  The discriminator is whether the op's own edit already SHIFTS the caret's span — a delete removes the span
  the caret sits in (clamp-to-start is the home), a move relocates it (clamp would collapse it). The paired
  decisions document the asymmetry so neither ticket cargo-cults the other (AD-claude-rebase-clamp-correct-for-delete-wrong-for-move).
- **The two pinned edges (round-trip tests):** ⌃K at EOL eats the `\n` (joins the next line — the observed
  emacs/VS-Code behavior); ⌘⇧K on the LAST line (no trailing `\n`) eats the PRECEDING `\n`, so no orphan blank
  remains and the text round-trips. WholeLine on a truly empty buffer returns `None` (inspect C1-LOW — else a
  dead ⌘Z step).
- **The routing is 5 keymap rows, NOT a seam rewrite (the Phase-1 catch).** The spec feared a "translation
  collapse": `key_from_keystroke` returns `Key::Backspace` before the modifier gate, so ⌥⌫ 1-char-deletes
  today. But the keymap chord path (`binding_from_keystroke` → `keymap.action_for`, app.rs) already runs
  BEFORE `key_from_keystroke` and preserves every modifier, so five Editor-scoped rows + dispatch arms route
  all five chords with no change to the translation seam. ⌃K is Editor-scoped, so the terminal's readline ⌃K
  (`op_for_ctrl_key`→KillToEnd) is untouched.
- **The DISTINCT-dispatch-op** — `apply_delete` is its own grouped-edit path (`delete_edits` →
  begin/end_undo_group → `set_selection`), NEVER `apply_editor_key_multi`'s Backspace arm, so a word-delete
  cannot inherit the #338 `backspace_pairing` auto-close widening.
- **Plain forward-delete** — the one input-layer edit: a `Key::DeleteForward` arm in `apply_editor_key_multi`
  → the new `Buffer::delete_forward_at_selections` (one char right of a bare caret, or the selection; a
  zero-width no-op at EOF), reusing the tested `edit_ranges_restoring` machinery. It was a silent no-op before.

**Go to line (#302, M19) SHIPS — one pure fn plus an overlay shim, because the jump machinery already existed.**
⌃G opens a small inline overlay; `50` (or `50:12`) + Enter places the caret and centers the view, Esc restores
the caret you started from. The pre-authored spec planned a `clamp_goto`, but attacking that sentence against
live code (Phase-1 F1) found `caret_for_line_col` *already* owns the 1-based→0-based conversion AND the
past-EOF/past-EOL clamp (the off-by-one lives in that one seam), and `scroll_editor_to_row` already centers
(`ScrollStrategy::Center`) — so the whole ticket collapsed to:
- **The one new pure fn** — `parse_goto(input) -> Option<(usize, Option<usize>)>` (code_view.rs, beside
  `caret_for_line_col`): SYNTAX only. `split_once(':')`; each part an all-digit run parsing to ≥1; empty /
  non-digit / bare-or-trailing `:` / `0` / a stray third field / overflow → `None`. It validates the grammar;
  `caret_for_line_col` owns the RANGE clamp. Total, no panic — the mutation surface (8 mutants, the split, the
  two `?`, the `>= 1` filter) is a truth table.
- **The overlay = the `renaming_symbol` shape, copied whole** (the smallest shipped precedent): one
  `Option<GotoDraft { input, origin_caret }>` field, a `match` arm HIGH in the `on_key_down` ladder (escape→
  restore, enter→commit, backspace→pop+preview, a single ASCII digit/`:`→push+preview, everything else
  swallowed), **one line in `text_input_blocked`** (the leak gate — the #339 don't-drop-half lesson, pinned by
  a drive asserting the buffer stays byte-identical while typing), and a small FIXED-position chip (not
  caret-anchored — the live preview scrolls the caret off-screen, so a caret-anchored card would vanish).
- **D-PUSH-ON-COMMIT-ONLY** — Enter pushes the ORIGIN caret onto the NavStack (the `jump_to_sticky_header`
  idiom — push where ⌃G was pressed, so ⌃- returns), then places + centers the target; Esc restores the origin
  and pushes nothing (a cancel is not a navigation).
- **D-LIVE-PREVIEW-SCROLL** — a valid in-progress input scrolls the would-be target into view WITHOUT moving
  the caret; the caret moves only on Enter.
- **The clear_marked-before-placement symmetry** (inspect L1) — commit AND restore both `clear_marked()`
  before the out-of-band caret write, symmetric with every sibling placement path; a restore-path omission
  would leave a stale IME composition span to misdirect the next replace.

**Go to Symbol in File (#304, M19) SHIPS — the FIRST in-file MEANING query (names, not ranges).** ⌘⇧O opens a
fuzzy picker of the current Rust file's symbols; Enter jumps + centers + pushes the NavStack. The pure home is
a NEW `crates/syntax/src/symbols.rs` — the **4th `marley_syntax` node API** beside `enclosing_ranges` (#329),
`matching_delimiters_in` (#340), `all_headers` (#330):
- **`file_symbols(src) -> Vec<Symbol>` adapts the grammar's OWN `tags.scm`** (D-TAGS-QUERY-IS-THE-EXTRACTOR) —
  a QUERY RUN, not a hand-walk. `parse.rs` gains `tags_query()`, the `OnceLock<Query>` twin of `highlight_query()`
  compiling `tree_sitter_rust::TAGS_QUERY` (the crate ships it, registry-verified). `Symbol { name, kind, line,
  col }` + `SymbolKind` are pure + LSP-agnostic (the app maps to a glyph). Positions are 1-based CHAR (feed
  `caret_for_line_col`; the #336 char-vs-byte discipline).
- **The method/function double-match** — a method's `function_item` matches BOTH `@definition.method` (inside a
  `declaration_list`) AND `@definition.function` (the bare pattern, no parent constraint) → two entries at the
  same name position; `dedup_double_matches` keeps the more-specific Method, order-independently (extracted so
  BOTH capture orderings are unit-testable). `@definition.class` conflates struct/enum/union/type → split back
  by `node.kind()`. `@reference.*` (calls, impl blocks) dropped. **Limit:** bodyless trait/`extern` sigs are
  `function_signature_item`, which tags.scm doesn't capture (documented).
- **D-CALLER-GATES-LANGUAGE** — the pure fn parses Rust unconditionally and CANNOT self-gate; the caller gates
  on `code_syntax::language_of(path) == Language::Rust` (COPY #340's `refresh_bracket_match` gate, NOT the
  ungated #329/#330 — those are the new bug #351). A non-Rust file → "(no symbols)", no walk
  (AD-claude-caller-gates-language-not-the-pure-syntax-primitive).
- **The picker is SYNCHRONOUS** — `OpenFileSymbols { finder, symbols }` mirrors #325's `OpenSymbols` shape but
  the symbols are in hand (memoized per `(nonce, version)` — the #330 sticky cache shape), filtered LOCALLY
  each keystroke with `FinderState::results` (the ⌘P `fuzzy_rank`, empty query = document order) — NO per-key
  server round-trip. The jump reuses the #302 `caret_for_line_col` + NavStack-origin-push idiom. ⌘⇧O is
  Editor-scoped, SHADOWING the global open-remote (the #325 ⌘T precedent).
- **The `.into_iter()`-over-Option coverage idiom** — `for tree in rust_parser().parse(..).into_iter()` covers
  an always-`Some` parse with NO dead region (the loop body runs on the one `Some`, the loop ends through the
  iterator's `None`) AND no `for_loops_over_fallibles` lint — a reusable way to keep a parse-bearing pure fn
  line-total without moving it into the coverage-excluded `parse.rs` (the #300 dead-branch lesson, refined).

**Code folding (#305, M19) SHIPS — the FIRST buffer-row ↔ visible-row projection.** ⌥⌘[ folds the innermost
definition at the caret, ⌥⌘] unfolds, and the palette carries Fold All / Unfold All. A folded region hides
`header_row+1 ..= end_row` behind a muted "⋯ N lines" tail (the #331 inlay channel). The chevron in the gutter is
a follow-up (the state ships; the ▾/▸ glyph is deferred).

- **The seam is the whole ticket.** The editor's `uniform_list("editor-lines", len_lines(), …)` handed the slot
  index STRAIGHT to `line_text(row)` — a strict 1:1 map that ~17 interior sites (carets, click/drag, selections,
  #310 squiggles, the git lane, gutter numbers, sticky headers) silently assumed. Hiding rows severs that identity
  everywhere at once, so the conversion happens **exactly ONCE at the rim**: `total = proj.visible_count()`, and
  the callback's first line is `let row = proj.buffer_row(slot)`. Every interior site keeps buffer-row semantics
  untouched. Only the OUTBOUND crossings convert too — `scroll_editor_to_row` → `slot_of`, and the sticky band's
  `geom.first` (a slot) → `buffer_row` before `sticky_rows`. With nothing folded, `fold_projection` returns identity
  and does NO parse — byte-identical to the pre-#305 path. (The 5 LSP overlay cards' `(buffer_row - geom.first)*cell_h`
  geometry above a fold was the documented Slice-2 known-limit — CLOSED by #352, next bullet.)
- **#352 (M28) closes the Slice-2 known-limit — the frame-geometry fan-out projects too.** The nine sites that
  mixed BUFFER rows with the slot-domain `editor_geom.first/last` all convert now: the four LSP overlay cards +
  the IME caret rect anchor via `FoldProjection::viewport_offset(buffer_row, first_slot, last_slot)` — one pure
  seam owning the window check + slot offset (a row hidden inside a collapsed fold anchors at its header's slot,
  `slot_of`'s snap; an at/past-EOF stale anchor returns `None` and hides); the hover-dwell + IME point inverses
  convert their pixel-derived SLOT through the shipped `buffer_row(slot)` (clamps stay in slot domain; below the
  last visible row the RAW slot is kept byte-for-byte — the shipped below-EOF clamp path); the content-width
  probe measures `buffer_row(slot)` per iterated slot; the inlay fetch window pages ±`INLAY_PAGE` in buffer
  domain off the projected viewport ends (the exclusive end maps through `last-1` so the short-file served-check
  stays byte-identical). Two adjacent fixes rode along: the geometry RECORDER's mount gates were themselves a
  units bug (`row == first` — buffer row vs slot — froze `editor_geom` whenever a collapsed fold sat fully above
  the viewport; now `slot == first` at both canvases), and `fold_projection()`'s parse arm is memoized
  (`FoldProjCache`, keyed `(nonce, version, anchor set)`) because the 16ms pump (`refresh_inlay_hints`) and
  mouse-move read the projection now. No-fold: byte-identical and parse-free at every touched site.
- **#353 (M19 follow-up) closes the lifecycle — a fold entry dies with its FILE.** The #305 W-2 parked leak
  (`editor_folds` never cleared on close; stale anchors could spuriously re-fold a reopened file) is
  structurally gone: every editor view release routes through `RootView::release_editor_views`, and when the
  registry reports a last-view INSTANCE drop the entry for that stored path is scrubbed — **census-guarded**
  (`content::any_open_editor_with_path`, exact `==` on the stored canonical spelling, the map's own key
  semantics): two alias-root workspaces of one dir (D-OPEN-DEDUPE-SCOPE keeps their instances separate) share
  one fold entry, so the survivor keeps its folds and only the final same-path drop clears. Twin VIEWS of one
  instance never scrub (the release returns nothing until the last view). Reopen presents unfolded — the
  in-session fold model, matching the Zed behavior map's view-scoped FoldMap. The resolve-HIT mount arms mint
  their rows from the instance's STORED path (row == instance by construction), so the verbs' insert key and
  the scrub's remove key can never diverge. Fold state persistence across close/reopen stays deliberately
  out (declined at #305); `git_marks`' identical leak class closed at #412 — the same arm and census, plus
  the cache-key clear (a `git_marks_key` dangling on a dropped path would have early-returned the reopen
  refresh and blanked a dirty file's gutter — the sweep of a per-path map includes its derived-key twins).
- **The pure engine — the 5th `marley_syntax` node API, in a NEW `fold.rs`.** `fold_regions(src) -> Vec<FoldRegion>`
  walks the tree ITERATIVELY (the `all_headers` shape — recursion overflows the UI stack on a deep file) for the
  multi-line FOLD_KINDS (function/impl/mod/trait/struct/enum_item + `match_expression`). `FoldProjection::new(total,
  &[(header,end)])` merges the active folds into sorted hidden runs; `visible_count` / `buffer_row(slot)` /
  `slot_of(row)` (a hidden row → its header's slot) are a prefix-sum bijection over visible rows, total (out-of-range
  saturates, no panic). The mutation surface, at cov/MSI 100.
- **HALF-OPEN hidden runs, to kill an equivalent mutant.** With inclusive `[start ..= end]` runs, `slot_of`'s
  interval boundary lands on the LAST hidden row — where "snap to the header" and "count the run past" compute the
  IDENTICAL value, an equivalent (unkillable) mutant. Storing runs HALF-OPEN `[start, end)` moves the boundary onto
  the first VISIBLE row after the fold, where the two genuinely differ, so `slot_of(<that row>)` distinguishes them
  → MSI 100. (A reusable lesson: when a boundary mutant is equivalent, re-express the interval so the boundary sits
  on a point the two branches disagree about.)
- **Fold state rides ANCHORS; the F1 re-derive.** Each active fold is keyed by its header's `Buffer::anchor_at`
  (`anchor.rs`'s FIRST production consumer). `fold_projection` resolves each anchor to its live row, then keeps the
  fold ONLY if `fold_regions` still has a region starting there — so typing above a fold carries it (the anchor
  shifts), and deleting its region evaporates it (`resolve_anchor` is clamped-not-`Option`, so region VALIDITY is
  re-checked against a fresh parse, not an anchor-None).
- **Auto-reveal must hook the LOWEST shared primitive (a code-review find).** Any navigation into a hidden row must
  unfold first, or it strands the caret on the folded `⋯` header. The reveal was first wired only into
  `follow_editor_caret` — but the JUMP commands (go-to-line, symbol-jump, go-to-def / ⌘T / nav-back / diagnostics,
  and **find**) place the caret and call `scroll_editor_to_row` DIRECTLY, bypassing it. The fix is a shared
  `reveal_and_scroll_to_row` (reveal the target row's fold, then scroll) at every jump site; the go-to-line PREVIEW,
  whose caret stays at the origin, deliberately keeps the plain scroll. (Two adversarial critics + the implementer's
  trace confirmed the bypass; the find site was one a critic missed — `PR-claude-every-caret-path-must-reach-the-reveal-hook-001`.)

**LSP formatting (#314, M20) SHIPS — ⌥⇧F + opt-in format-on-save.** ⌥⇧F formats the active Rust document through
rust-analyzer (`textDocument/formatting` → rustfmt); `editor.format_on_save` (default OFF, a palette toggle) runs
it before every ⌘S. The apply is transactional (one undo unit; an overlapping/inverted batch applies nothing) and
the save is never blocked or lost.

- **The apply is the #322 engine, unchanged.** `apply_one_file` resolves the server's `TextEdit[]` to char offsets
  (the #309 bridge), rejects overlaps whole, and splices in one undo group — so the new pure `marley_lsp::formatting`
  is only wire shapes (`formatting_support` / `formatting_request_params` / `parse_formatting_edits`, the last
  reusing `workspace_edit::text_edit_of`). The `FormattingKey` stale guard lives app-side (a version needs `Buffer`).
- **The caret is a (line, column) re-seat, NOT an anchor.** rust-analyzer returns ONE whole-file `TextEdit`, which
  COVERS the caret — and `rebase_offset` collapses a covered `Bias::Left` anchor to the span start (0), teleporting
  the caret to line 1 every format. Capturing `line_col(caret)` before the apply and re-seating via the shipped
  `caret_for_line_col` after keeps it on its logical line (best-effort, the VS Code/Zed behavior). A code-review
  find: `PR-claude-covered-anchor-collapses-use-linecol-reseat-for-full-replace-001` (an Anchor carries a caret
  only through an edit that does NOT cover it — reserve it for granular edits).
- **Format-on-save is ONE write, bound to the ORIGIN, never lost.** ⌘S with the setting armed parks a latch (no
  write) + sends the format; the response applies the edits then does the single write of the *formatted* text.
  A ~2 s deadline (its OWN countdown, not the 10 s request timeout), a server error, or an intervening edit each
  fall through to a plain save — a save is never lost or blocked (D-SAVE-NEVER-BLOCKED). **#354 made the binding
  literal:** the latch carries the origin's `ContentId` (captured at ⌘S), and every completion lane resolves it
  through the registry and saves THAT instance — active or backgrounded, same project or not
  (`PR-claude-async-completion-binds-to-origin-not-reread-active-001`). The retired `active_is_origin` guard was
  surface-granular (`has_path` spans a pane's rows), so a same-project row switch could plain-save the wrong,
  newly-active file — id-granularity closes that class. A background origin saves in CONSENT mode (the #275
  table on the instance: `Changed` holds write+arm, `CleanReload` defers to activation, `Deleted` recreates),
  `didSave` routes by the instance's OWNING root, ⌘S-on-B settles (never discards) A's parked save, and a buffer
  reload clears the in-flight formatting key (the #401 version-epoch class). anchor.rs's SECOND production
  consumer after #305 folds (via the (line, col) capture, not the anchor itself — see above; the re-seat is
  view-scoped — a background origin's views reconcile on activation).

## Key decisions

- **`EditOrigin{Human,Agent}` on every `EditResult`** — the observable write-provenance seam. It is the exact
  boundary where the sold **brain** applies `EditOrigin::Agent` edits to the (open-core) editor without
  inheriting copyleft, and undo coalescing already refuses to merge an Agent write into a Human run.
- **The `SelectionSet` invariant is ENFORCED, not documented** (M19 #296) — ordered · disjoint · non-empty, at
  every entry point, because an invariant nothing enforces is fiction. A green gate proved that the hard way
  here: 100% coverage, MSI 100, and a 50k-step differential fuzzer all passed while a clamp that produced
  duplicate cursors, a stale post-undo set that panicked ropey, and a "never empties" claim nothing upheld sat
  in the code — the fuzzer only ever saw sets the canonicalizing constructor had already cleaned. **A fuzzer
  proves the transform; only enforcement proves the invariant.**
- **The BUFFER owns the cursor set; the app surface owns nothing** (M19 #297). `OpenFile` used to keep its own
  `caret` + `anchor` *beside* a `SelectionSet` the `Buffer` already had and the app never read — two sources of
  truth for one cursor, one of them vestigial. Deleting the duplicate (rather than teaching it to sync)
  dissolved the E0499 three-way `&mut` accessor that 17 call sites existed to work around, and made the
  "jump leaves a stale anchor" bug *unrepresentable* rather than merely fixed.
- **The merge rule is caret/range-ASYMMETRIC, and the asymmetry is load-bearing** (M19 #297).
  `SelectionSet::overlaps` uses `<=` where either side is a CARET (two carets at one offset are one cursor, and
  leaving them separate makes that cursor insert its text twice) and `<` where BOTH are RANGES (two merely
  *touching* ranges are two distinct cursors over two DISJOINT spans; the back-to-front sweep applies them with
  zero interference). #296 used `<=` for everything and justified it with the claim that abutting selections
  "would double up at the seam" — **that claim is false**, and it cost a cursor: ⌘⌥↓ followed by ⇧↓ produces two
  line-spanning ranges that abut exactly at the shared line-start offset, so the flagship gesture destroyed its
  own second cursor on the very next keypress. Because that is a MOTION, ⌘Z could not bring it back. **A test
  asserted the bug and thereby defended it** — coverage, mutation and a 50k-step fuzzer all applauded.
- **The goal column lives ON `Selection`, so its reset is FREE** (M19 #297). Every existing constructor sets
  `goal_col: None`, so a horizontal motion, an edit, a click or a merge clears it *by default* — you cannot
  forget to. Only the vertical path carries it forward. Without it, two cursors crossing a SHORT line both clamp
  to its end and never recover the column they wanted. The guarantee, stated honestly: cursors on **distinct
  rows** never merge; two on the SAME row that clamp to the same column occupy the same OFFSET, and one offset
  is one cursor — they collapse, as in every editor, and the survivor keeps a goal column so it is not stranded.
- **A single `line_layout` source of truth** — the render's tab-expanded `display` and the caret's `col_starts`
  map come from one pass, so the caret can never drift from the glyphs across tabs/multibyte.
- **`undo` drains via `apply_raw`** — the inverse re-applies without re-recording, so the history is a clean
  stack, not a loop; all offset math is `.chars().count()` (multibyte-safe).
- **Reuse over fork** — `apply_editor_key` delegates to the prompt's tested `apply_key` (one divergence:
  Enter); the app shims stand on cov/MSI-100 pure seams so the gpui glue stays thin.

## Verification

The pure crate: the multibyte/multi-line fixture `"abc\nzé😀w\ndef"` across `buffer`/`movement`/`selection`/`undo`
unit tests + a Human-vs-Agent integration + a proptest char↔byte round-trip; cov 100 / MSI 100. The app seams
(`editor_surface`, `code_view`, `input`) are likewise cov/MSI 100; the `app.rs` shims are driven-capture
validated (typed runs, click-to-caret, drag-select, ⌘S write, ⌘Z/⌘⇧Z, ⌘X→⌘V). `edit()`'s delta is a
cargo-mutants blind spot — the exact by-value multibyte-delta assertion is its sole guard.

## The frontier — toward a Zed-class editor

The roadmap from this single-cursor plain-text baseline is mapped in the **Zed** architecture reference (built
from a source-level read for a clean-room reimplementation inside Marley's own — intended-GPL — editor tier):
- [`../zed_architecture/subsystems/00-overview.md`](../zed_architecture/subsystems/00-overview.md) — the
  strategic map + the dependency-ordered sequence (KeyContext → gpui substrate → tree-sitter → **anchors** →
  multi-cursor → LSP) and the provenance boundary that keeps the brain copyleft-clean.
- [`../zed_architecture/subsystems/02-text-buffer-anchors.md`](../zed_architecture/subsystems/02-text-buffer-anchors.md)
  — **anchors** (edit-surviving positions), the quiet prerequisite for multi-cursor, async LSP, and marks. The
  recommended clean path is **delta-log anchors** rebased through Marley's *existing* `BufferDelta` log — no
  SumTree/CRDT rewrite.
- [`../zed_architecture/subsystems/03-editor-multibuffer.md`](../zed_architecture/subsystems/03-editor-multibuffer.md)
  — the road from Marley's `Buffer`/`SelectionSet` to a real editor: `SelectionSet` → multi-cursor, a
  DisplayMap-style transform layer (fold/tab/wrap/highlight), and the multibuffer (N files' excerpts in one view).

The **`EditOrigin::Agent` brain seam** is the through-line: it exists today, and every frontier capability is
tagged for provenance so the brain never links Zed-derived editor code.

## See also

- [app_shell.md](app_shell.md) (the §M15 shim routing) · [crate-map.md](crate-map.md) ·
  [`SPEC-editor.spec.md`](../specs/SPEC-editor.spec.md) · `marley_text_offsets` (the offset vocabulary owner).
