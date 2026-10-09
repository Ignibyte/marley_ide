# Prevention rules — the local ledger

> Exported 2026-08-09 from the forge DB (table `prevention_rules`; 312 entries,
> TICKET-409 — the scrap-forge pivot). This file is the LIVE capture surface:
> `/pipeline:inspect` APPENDS new `## <code>` blocks (`/pipeline:complete` catches up);
> recall is `grep`. One entry per `## <code>` heading — never edit history.

## PR-claude-a-boolean-that-names-an-invariant-must-be-earned-001
*severity: critical · prevents: BF-claude-invariant-tag-asserted-not-earned-corrupts-redo-001*

When a flag/tag NAMES an invariant that downstream code TRUSTS instead of checking, any change that can break the invariant must COMPUTE the flag, never keep asserting it.

The dangerous shape: a `bool` (or enum tag) is passed as a literal `true` because, at the time it was written, it was unconditionally true. Downstream then *relies* on it and skips its own verification — often with a comment proudly explaining why the check is unnecessary ("because the cursors did not move, appending char-wise is contiguous by construction"). That comment is the tell: it documents an assumption the producer is no longer keeping.

M22 #338: `begin_group(restore, /*cursor_anchored*/ true)` meant "every cursor ends at the end of its own insert". `coalesces_into` trusted it and did NO arithmetic contiguity check. Auto-close introduced a caret span whose whole job is to put the caret elsewhere — so the tag lied, the next typed char was appended onto the wrong record (`inserted = "()x"` where the text read `(x)`), and REDO replayed it verbatim: silent text corruption at N>=2 cursors. Undo hid it (it only uses the char COUNT). Fix: `begin_group(restore, spans.iter().all(Option::is_none))` — earn the tag.

THE CHECKS:
1. Grep every literal `true`/`false` passed to a parameter whose NAME is an invariant (`cursor_anchored`, `is_sorted`, `already_validated`, `is_normalized`). Ask: "is my change the first thing that can make this false?" A new feature whose PURPOSE is to vary the thing the flag asserts is exactly that.
2. Find the CONSUMER and read what it skips because of the flag. If it trusts rather than verifies, the flag is load-bearing and a lie costs real data — not a wrong result, a CORRUPTED one.
3. A flag that was "always true" is a landmine precisely because nothing tests it: there is no failing case in the suite to notice, and mutation may not reach it (a bare `bool` literal / `match <bool>` often yields no viable mutant).

AND THE HALF-VERIFICATION TRAP THAT LET IT THROUGH: I *did* analyze coalescing at design time and got half of it right — I proved the 2-char record can never join a PREVIOUS run (`new.inserted.chars().count() == 1` fails), then wrote the note as if both directions were checked. The guard constrains only `new.inserted`; there is no `old.inserted` check, so a FOLLOWING char joins fine. **When a predicate is asymmetric (it constrains one side of a relation), verifying one direction is not verifying it.** Name both directions explicitly, or you will write "verified" over the half you never looked at.

## PR-claude-a-bounded-pool-needs-a-reclamation-path-not-just-a-cap-001
*severity: high*

A bounded resource pool (session registry, connection cap, handle table) needs an explicit RECLAMATION path — a TTL sweep, a close/disconnect reap, or a tie-to-connection-lifetime — NOT just a reject-at-cap policy. A cap alone converts legitimate churn (reconnects, retries, crashed clients that never send an explicit release) into a permanent wedge: once the cap fills with orphaned entries, every new admission is refused forever until a restart. Especially dangerous when a sibling/self client is designed to re-establish (re-initialize / reconnect) rather than reuse — it will fill the pool by design. Tie the resource's lifetime to something that ends observably (the client's standing stream/connection dropping), or add a TTL. Litmus: 'if N clients each connect, get admitted, then vanish without a clean release, does the pool ever recover?' If no, the cap is a footgun.

## PR-claude-a-caller-census-greps-the-callee-name-001
*severity: medium · prevents: BF-claude-caller-census-grepped-wrapper-names-not-the-callee-001*

When enumerating every caller of a seam before changing its contract, grep the CALLEE's own name workspace-wide (the method/fn identifier itself), never just the known wrapper spellings — wrappers are the callers you already know about, and the census exists to find the ones you don't. Where feasible, verify completeness with a mechanical oracle: change the arity/type first and let the compiler emit the full caller list before designing the routing.

## PR-claude-a-fuzzer-proves-the-transform-not-the-invariant-001
*severity: high · prevents: BF-claude-stale-selection-after-undo-panics-the-rope-001*

A differential fuzzer proves the TRANSFORM; it is blind to the INVARIANT ENFORCEMENT around it whenever both the production path and the oracle are fed inputs that a canonicalizing constructor already cleaned. On Marley #296 a 50k-step fuzzer + 100% line coverage + MSI 100 all went green while THREE real bugs sat in the enforcement layer (a clamp that created duplicate cursors, a stale post-undo set that panicked ropey, a "never empty" invariant nothing enforced) — because every generated set went through from_selections first, so the fuzzer never once saw a set that violated the invariant. Therefore: (1) fuzz the RAW/hostile input path too — hand the pub fn out-of-bounds, unsorted, overlapping, and EMPTY sets, not just constructor-blessed ones; (2) for every pub fn ask "does this take offsets/indices it did NOT itself produce, and forward them to something that panics on out-of-range?" — that fn owns the clamp; (3) state divergence is a bug class of its own: if op A writes a cached derived field (self.selection) and sibling ops B/C (undo/redo) do not maintain it, the field is STALE by construction and the next consumer eats it. A green gate is necessary, not sufficient — always spawn the adversarial critic and WAIT for it, even after the gate passes.

## PR-claude-a-guard-must-check-its-precondition-not-argue-it-001
*severity: critical · prevents: BF-claude-undo-group-absorbed-a-typed-char-and-destroyed-text-001*

When a guard's correctness depends on a PRECONDITION, the guard must CHECK that precondition — not assert it in a doc comment and then test the SHAPE it usually implies. A doc comment is not an invariant; it is a hope. On #299 `UndoHistory::coalesces_into` decided "is this a typed run continuing?" from structure (bracketed · record count == cursor count · selections line up · every record a pure insert whose new insert is 1 char) and stated its real precondition in prose: "Because the cursors did not move, each new insert lands precisely at the end of its own record's text, so appending char-wise is contiguous by construction — no offset arithmetic is needed or wanted." The comment toggle satisfied every structural condition while violating the prose one (its records are anchored at the min-indent COLUMN, not at the cursors), so a typed char was appended onto a record whose `at` was somewhere else and ⌘Z DELETED THE USER'S CODE, unrecoverably. Tellingly, the fn's own single-cursor twin `record()` CHECKED the same contiguity (`last.at + last.inserted.chars().count() == rec.at`) — the grouped path had dropped the check and kept only the argument. TWO ACTIONS: (1) When you write "because X, this is safe" in a doc comment, either assert X in code or make the PRODUCER state it — a tag/flag/enum the caller sets — rather than re-deriving X from geometry at the consumer. Inferring intent from shape is what fails: shape is a coincidence, intent is not. (2) When a sibling function already checks the property you are about to assume, that asymmetry IS the bug — go look at why one checks and the other doesn't. COROLLARY: prefer the tag over the arithmetic even when the arithmetic is provably correct. On #299 two critics verified a cross-coordinate-space contiguity computation by hand and it worked — but the tag kills the whole CLASS (every future bracketed line op: multi-cursor Tab, line-move, delete-word) while the arithmetic only fixes today's instance and can be satisfied by accident tomorrow.

## PR-claude-a-negative-assert-must-prove-the-machinery-ran-001
*severity: high · prevents: BF-claude-headless-drive-never-ticked-the-pump-negatives-passed-vacuously-001*

A test that asserts something did NOT happen must first prove the machinery that would have made it happen actually RAN. Otherwise the assertion is satisfied by the machinery being inert, and it will stay green through the exact regression it exists to catch. Two concrete forms in this repo: (1) gpui headless drives — `run_until_parked()` does NOT advance the mock clock, so a `background_executor().timer(...)` (the app pump) never becomes ready; pair it with `executor().advance_clock(...)` or nothing timer-driven runs at all. (2) Any "the gate rejected it" test — assert a POSITIVE control in the same suite (an input that SHOULD pass the gate does), so a wholesale failure of the subsystem shows up as the positive test failing rather than the negatives silently passing. Practical check when writing a negative assert: mentally delete the entire feature under test — if the assert still passes, it is proving nothing. In this codebase the anti-vacuity guard is cheap: assert the precondition (the tab restored, the doc is open, the pump ticked) in the same test, next to the negative claim.

## PR-claude-a-next-free-id-claim-needs-the-full-allocator-map-001
*severity: high · prevents: BF-claude-static-command-id-collision-swallowed-the-verb-001*

Before minting a constant in any shared id space (palette CommandIds, range bases, key codes), enumerate EVERY allocator of that space — the full literal map (action_for_command), every mint site (cockpit_commands + specially-dispatched consts), and the dispatch-chain ORDER that arbitrates collisions — never a partial grep whose pattern bounds the digits you expect. Then pin both directions with units: the new id resolves to nothing in the generic resolver (the reverse guard), and the constructed command list carries no duplicate ids. A collision here is silent: the first dispatch arm wins and the new verb becomes dead code that fires someone else's action.

## PR-claude-a-single-member-accessor-is-not-a-question-about-the-set-001
*severity: high · prevents: BF-claude-branch-discriminator-asked-about-member-0-not-the-whole-set-001*

In an N-member collection with a distinguished member (primary/first/last), NEVER let a predicate on THAT MEMBER stand in for a question about THE WHOLE SET. `primary().is_caret()` is not "is this the first press". `set.last().end()` is not "where the last match ended". Both substitutions read fine and both are false the moment the set can hold a member of a different KIND — and in a multi-cursor editor it always can (⌘-click adds a bare caret beside a range; the set is SORTED, so that caret can land at member 0 or at the end). On #298 these two substitutions produced FOUR HIGH bugs: ⌘D stalling forever, ⌘D swallowing a cursor the user placed, ⌘D dying one occurrence short after the wrap, and one ⌘-click at EOF killing ⌘D for the session. THE RULE: when a set is heterogeneous, write the predicate over the set (`selections().iter().all(|s| s.is_caret())`) and derive the operand by KIND, not by POSITION (the needle from the topmost RANGE; the resume offset from the bottom-most RANGE). Ask, for every use of primary()/first()/last(): "what if this member is the OTHER kind?" — and if the answer is "then it means something different", you have found the bug. Also: "the fn is TOTAL for every set shape" is not the property that matters. Total ≠ PROGRESSING. A gesture that returns a byte-identical set forever is total and useless.

## PR-claude-a-test-can-encode-a-bug-and-defend-it-001
*severity: critical · prevents: BF-claude-a-test-can-encode-a-bug-and-then-defend-it-001*

A green gate cannot see a bug that a TEST ASSERTS. When you write a test whose comment says "X MUST happen — otherwise Y", the rationale Y is a CLAIM, not a fact, and once written it becomes armour: coverage, mutation score and fuzzing will all applaud the buggy behavior forever after, and the next ticket that trips over it will be told the tests pass. On Marley #296 the assertion "two abutting selections MUST union — otherwise they would each insert at the seam and double up" was FALSE (abutting ranges replace DISJOINT spans; a back-to-front sweep applies them cleanly), and it pinned a bug that made the flagship multi-cursor gesture destroy its own cursor one ticket later. THEREFORE: (1) any test comment of the form "MUST … otherwise <bad thing>" is a hypothesis — go RUN the <bad thing> and see it happen, or delete the claim; (2) treat a test that asserts a DESTRUCTIVE simplification (merging, collapsing, dropping, clamping-to-one) with special suspicion — losing data/state is easy to justify with a plausible-sounding hazard; (3) when a later ticket's bug traces back to behavior an existing test demands, the test is the first suspect, not the last — fix the test, do not work around it.

## PR-claude-a-width-in-chars-is-not-a-width-in-cells-001
*severity: critical · prevents: BF-claude-width-in-chars-vs-cells-strands-cjk-tails-001*

`text.chars().count()` is NOT a width. It is a COUNT, in a different domain from display CELLS, and the two agree on pure ASCII — which is precisely why the mistake ships: every test you write by hand passes. UAX#11 (`unicode_width`, what `char_width` implements) makes a CJK glyph 2 cells and a combining mark 0, so a char count under-measures every CJK line by up to 2x and over-measures every combining one. When a codebase has declared ONE owner of the column domain (Marley: `code_view`, per `AD-claude-editor-offset-column-model-001`), any code needing a width in cells MUST ask that owner — add an accessor if none exists rather than counting chars at the call site. Two corollaries learned the hard way: (1) measure the layout the render actually DRAWS — if phantom/virtual text (inlay hints, ghost text) occupies columns, a measurement built from the phantom-free layout silently excludes it; (2) the mistake hides where tests cannot reach it. On #336 the pure seam took `usize` widths and could not tell they were miscounted, so its units passed 100% coverage AND 100% mutation while the shim that fed them was wrong — the bug lived in coverage-excluded, mutants::skip'd render code. **A pure seam being green says nothing about whether its inputs are in the right units.** Pin cross-domain measurements with a render-level test using a CJK line and a line with a trailing phantom, not with a unit on the seam.

## PR-claude-absolutize-via-injected-cwd-seam-001
*severity: medium · prevents: BF-standardize-cwd-failure-non-absolute-001*

When a function's contract is "always returns an absolute/normalized path" and it depends on the process cwd, (a) thread the cwd in as a parameter to the core (a §14 testable-IO seam) rather than reading `current_dir()` inside, so the relative-input and cwd-unavailable branches are deterministically testable without making the syscall fail; and (b) on cwd-unavailable, fall back to an absolute base (the flavor root) so the invariant holds — never return a silently-relative value behind an `is_absolute()` that hardcodes true. If an invariant accessor hardcodes a value, guarantee that invariant at construction; don't let it lie.

## PR-claude-adopted-query-inherits-its-scope-gaps-document-them-001
*severity: low*

When a feature ADOPTS a substrate's own query/parser (tree-sitter's tags.scm, a grammar's highlights.scm, a library's built-in matcher) instead of hand-rolling, it inherits that query's SCOPE GAPS — verify and DOCUMENT what the query does NOT capture, don't assume it covers everything the feature's title implies. Concrete: tree-sitter-rust's `tags.scm` method pattern matches `function_item` (a fn WITH a body), so a BODYLESS trait method signature (`fn m(&self);` = a `function_signature_item`) is NOT extracted — a `trait { fn m(); }` yields the trait but not its method signatures, while a trait DEFAULT method (`fn m() {}`) IS captured. This is not a bug in the extractor (it faithfully runs the query); it is a documented v1 scope limit. Write a throwaway that probes the EDGES the query might miss (bodyless sigs, macro 2.0, proc-macros, nested forms) so the gap is a known/documented limit, not a surprise bug report later. Pairs with the general "read the permissive dep's source — adopting its query is a win — but adopt its CONTRACT, gaps and all."

## PR-claude-advance-shadow-only-on-success-001
*severity: medium*

When you keep a cached "shadow" of an external mutable state (a last-applied size, a last-synced revision, a last-written value) and use it as an idempotence/unchanged guard, advance the shadow ONLY after the operation that mutates the real state has SUCCEEDED. Writing the shadow unconditionally after a fallible call (`let _ = op(x); shadow = x;`) makes the guard lie on failure: it records the intended value while the real state stayed old, so the guard then elides every retry and the divergence is permanent+silent. Pattern: `if op(x).is_ok() { shadow = x; }`. The retry cost of NOT advancing on failure (one cheap re-attempt per tick for a dead resource) is almost always cheaper than a silent permanent divergence.

## PR-claude-agent-output-size-not-a-completion-signal-001
*severity: low*

NEVER treat a background agent/critic's output-file SIZE (or any metadata heartbeat) as a completion signal — the transcript buffers and flushes irregularly, so a small/steady size (e.g. 156 bytes) can mean "actively working", not "stalled/done". Killing an agent on that signal discards real in-progress work (on #293 I killed 2 critics mid-analysis; one had a live lead on the exact gpui stop_propagation question). Wait for the harness's actual task-completion notification. If you must bound the wait, spawn synchronously (run_in_background:false) or set a real timeout, and when a change is small enough to be self-verified, do the rigorous self-review + a live/differential check rather than gating on unreliable critics. (Pairs with the known critic-stall at 156B — distinguish the two ONLY via the completion notification, never file size.)

## PR-claude-an-equivalent-mutant-means-delete-the-redundancy-001
*severity: medium*

An EQUIVALENT MUTANT (one no test can kill because it does not change behavior) is a DESIGN SMELL, not a gate problem. It is the mutation engine telling you that an operator in your code is REDUNDANT — some other line already guarantees what it computes. Do not suppress it, do not carve an exclusion, do not write a test that pretends to cover it. DELETE THE REDUNDANCY, and the operator goes with it. Two on #298, both fixed by removing code: (a) a cyclic walk written as `filter(start >= resume).chain(filter(start < resume))` — widening the second to `<=` re-offers matches the FIRST half already offered, so nothing observable changes; rewritten as a rotation (`position(|m| m.start >= resume)` + `chain`) it says the same thing with ONE comparison, and four mutants collapsed into one killable `>=`. (b) `word_range_at`'s `pivot + 1` — the pivot's char is a word char BY CONSTRUCTION and the forward loop re-tests it, so `pivot + 1` → `pivot * 1` lands in the same place; deleting the `± 1` (each scan re-tests its own boundary char, so no adjustment is needed at all) collapsed the whole `pivot` branch into a guard. Shorter code, identical behavior, no unkillable operator. NOTE the trap in (b): naively removing only the `+ 1` MOVED the hole to `i - 1` → `i / 1` — the redundancy itself was the bug, so remove the whole redundant construct, then RE-RUN `cargo mutants` to confirm you did not just relocate it. SECOND, LOAD-BEARING LESSON: (b) was a PRE-EXISTING MSI hole since #265 that had never gone red, because the mutation gate only mutates files IN THE DIFF. A green MSI is a statement about the files you touched, NOT about the codebase — so when a ticket first touches a long-untouched file, expect latent mutants and budget for fixing them at source (§0), not for arguing they are someone else's.

## PR-claude-assert-no-field-via-exhaustive-destructure-001
*severity: medium · prevents: BF-r9-no-field-denylist-non-exhaustive-001*

To assert a struct does NOT have a field of some kind (e.g. "Config has no network/auth field"), use an in-crate exhaustive destructure as a compile-time check: `let TheStruct { known_a: _, known_b: _ } = value;` with NO `..` rest pattern. Adding any new field makes this fail to compile (E0027) — a positive, exhaustive guard. A trybuild/denylist that asserts specific forbidden names DON'T compile is non-exhaustive (misses un-enumerated names) and can only test accessors, not private fields. Prefer the exhaustive destructure as the primary check; keep a denylist only as documented defense-in-depth.

## PR-claude-assert-the-raw-representation-not-a-normalizing-projection-001
*severity: medium*

When a fix changes a RAW representation (e.g. output_styled: the per-row styled lines) but a downstream PROJECTION already NORMALIZES away the change (e.g. output_text = the styled lines joined + trim_end — which drops trailing blanks regardless), tests written over the projection CANNOT distinguish the fixed code from the buggy code (or from an identity no-op). The mutation gate is then VACUOUSLY satisfied — cargo-mutants' whole-fn-return mutants get killed by the projection tests without any test exercising the new behavior, so reverting the fix keeps MSI 100. #50: trimming trailing blank rows from output_styled was invisible to every output_text-based pump test. Rule: when the acceptance is about a raw representation, assert THAT representation directly (output_styled().len()/content), not just the normalized projection. At inspect, ask: "does an existing normalizer make my change unobservable to the current tests?" — if yes, a direct raw-representation test is mandatory, and the MSI number is meaningless until it exists.

## PR-claude-async-answer-carries-question-identity-every-hop-001
*severity: high · prevents: BF-claude-answer-race-no-question-identity-001*

An asynchronous answer to a MUTABLE question (a seat's question card, any confirm-style prompt whose content can change under the user) must carry the QUESTION'S identity — not merely the subject's id — at every hop: capture the rendered question at the interaction site, gate dispatch on exact equality with the current question, tag the in-flight result with a per-dispatch nonce so a superseded send can never overwrite a newer outcome, and put the identity on the wire so the far side can refuse stale answers. Subject-id-only tagging self-conceals: the record re-fetches the NEW question and reads current forever.

## PR-claude-async-completion-binds-to-origin-not-reread-active-001
*severity: high*

When an async operation (an LSP round-trip, a debounced save, a deferred write) parks state and completes on a LATER tick, its completion must act on the ORIGIN captured at request time — the (project, tab, path, editor) the user targeted — NOT on a re-read of "the currently active X". Between park and completion the focus/active-selection can change (a tab switch, a pane focus, a new file), so a completion that calls a save_active/edit_active/close_active-style "operate on the active thing" primitive will hit the WRONG target: it saves/edits the file the user switched TO, and silently drops the operation on the file they targeted. For a SAVE this is a lost save (unforgivable). Bind the latch to the origin identity (path or proj/tab), and at completion either operate on that origin by locating it, or — if the primitive is active-only — guard the completion on the active target still being the origin and abandon cleanly (visibly, never saving the wrong target) when it isn't. Trace the cross-target race (switch during the round-trip) explicitly; a single-target trace passes while the bug ships.

## PR-claude-async-response-error-arm-must-branch-001
*severity: medium · prevents: BF-claude-lsp-init-error-false-ready-001*

When handling an async request's RESPONSE that carries a Result/Ok-or-Err (LSP initialize, any RPC reply, a spawned-command result), the error arm MUST route to a distinct failure transition — never let it fall through to the success path. The trap shape: `if let Ok(v)=result { use v } ; do_success_transition()` runs the success transition on BOTH Ok and Err. Use `match result { Ok=>success, Err=>failure }`. For a handshake/liveness reply specifically, an Err means the peer is unusable → tear down + let the restart/backoff policy govern; do NOT report "ready". Bit #308 (init-error → false-Ready, no restart).

## PR-claude-audit-every-per-view-walk-when-views-exceed-one-001
*severity: high · prevents: BF-claude-per-view-walks-over-shared-content-001*

When a change makes view_count > 1 reachable for refcounted shared content (a one-instance-many-views migration), grep EVERY loop/walk keyed by the view (pane, cell, row) that reads or writes per-CONTENT state (map scrubs, tick counters, resize shadows, pumps) — each one silently assumed view == content. For each walk: dedupe per content, elect one authority view (focused wins, else first in deterministic walk order), or move the mutation into the last-release arm. The walks compile clean and stay green under 1-view tests; only a twin-aware test or an adversarial critic pass surfaces them.

## PR-claude-authority-type-doc-must-move-with-semantics-001
*severity: medium*

When a ticket changes a load-bearing invariant (e.g. display columns = char indices → UAX#11 width cells), sweep EVERY doc statement of the OLD invariant in the same diff — especially the AUTHORITY TYPE's struct/module doc and test comments ("strictly increasing", "each char is one column"), not just the fn you edited. The struct doc is the first thing the next consumer reads; a stale contract there re-seeds the exact conflation the ticket fixed (Marley #277: LineLayout's doc still asserted one-column v1 after line_layout went width-based). Grep the invariant's key phrases across the crate before closing inspect.

## PR-claude-bare-literal-body-for-constant-return-fns-001
*severity: low · prevents: BF-claude-equivalent-mutant-constant-return-with-dead-statement-001*

A fn whose entire purpose is to return a CONSTANT (an always-`false` `is_syncable`, an always-empty/always-`None` stub) must have a BARE literal body — `{ false }`, not `{ let _ = x; false }`. cargo-mutants recognises a "replace-body-with-that-literal" mutation as identical-to-the-original and SKIPS it (so a bare-literal constant fn yields only the killable opposite-value mutant). Adding a no-op preceding statement (`let _ = x;`, a discarded call) makes the body non-identical to the literal, so cargo-mutants now EMITS the "replace whole body with the literal" mutant — which is EQUIVALENT (unkillable, since the fn already returns that literal) → MSI < 100 with no way to kill it and no lawful suppression (§0). If you must reference an unused generic/param to satisfy a lint, first check whether the lint is even in the gate's set (clippy `extra_unused_type_parameters` is pedantic, NOT in `-D warnings`) — usually you can just drop the statement; if truly needed, use `PhantomData` in a FIELD, never a statement inside a constant-returning body.

## PR-claude-blanket-accessor-move-align-render-resolvers-001
*severity: high*

When making an accessor focus/context-aware and moving inline bypasser call-chains onto it via a blanket regex (e.g. `X.active_tab().editor()` → `X.active_editor()`): (1) the regex WILL miss chains whose receiver is not the identifier you anchored on — here a render row closure's receiver was `entity.read(app)`, not `self`/`view`. AFTER the move, grep the ORIGINAL pattern (`.active_tab().editor()`, multi-line-tolerant, ANY receiver) and confirm the only remaining hits are the accessor's own `let tab = …` internals — a non-zero count elsewhere is a straggler. (2) A shared render helper with a `'static` closure that RE-READS state (uniform_list rows, canvas paint) must resolve the surface the SAME way as whatever SIZED or GATED that render (fold_projection, the row count, the caret list). If the sizer uses the new focus-aware accessor but the closure keeps the old tab-only read, the two diverge → a list with full height and ZERO rendered rows (blank), which compiles fine and passes input/geometry mechanism-tests but has no pixels. Add a render test asserting the focused surface emits ≥1 row.

## PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays-001
*severity: medium*

In gpui, `.occlude()` (= HitboxBehavior::BlockMouse) blocks SCROLL as well as clicks/hover on elements behind it — so an overlay drawn OVER a scroll container creates a scroll dead-zone wherever it sits. Use `.occlude()` only for MODAL surfaces (palettes, menus, dialogs) that intentionally freeze the background. For a NON-MODAL overlay pinned over a scrollable area (a sticky header, a floating badge over scrollback, a hover card), use `.block_mouse_except_scroll()` instead — it blocks click/selection fall-through but lets the wheel/trackpad reach the scroll container beneath. gpui's own div.rs docs say block_mouse_except_scroll "should be preferred" for this. Symptom if you get it wrong: scrolling stops working when the pointer is over the overlay band.

## PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001
*severity: high*

ANY blocking I/O (a network fetch, a slow file read, a subprocess wait) triggered from a gpui event handler / dispatch / render runs on the MAIN THREAD and FREEZES the UI for its full duration. A "bounded by a 5s timeout" is NOT safe — 5s of frozen UI is a terrible freeze, and a slow-but-succeeding call (the forge is ~2.7s/call) freezes for its real duration every time. RULE: move it to a BACKGROUND thread — `std::thread::spawn` a worker that does the blocking call + sends the result through an `mpsc::channel`; poll the `Receiver` with `try_recv()` in the pump loop (which already runs each 16ms tick) and `cx.notify()` when it arrives; show a "loading…" state while the receiver is pending (`is_some()`). Requires the worker's inputs be `Clone + Send + 'static` (e.g. `#[derive(Clone)]` on the client). CRITICAL META-LESSON: this class of bug is INVISIBLE to the green gate (the shim is cov-excluded + mutants::skip) AND to a static inspect critic (which ACCEPTED the sync design as a MED) — the LIVE SELF-TEST is the ONLY oracle for UI responsiveness. When a UI action does I/O, the validate self-test MUST confirm the overlay/pane appears IMMEDIATELY (capture within a few hundred ms), not just eventually. Related: PR-claude-insert-at-prompt-goes-to-cooked-buffer (the self-test is the only proof an input-path feature works).

## PR-claude-boot-decisions-key-the-restored-active-root-001
*severity: high · prevents: BF-claude-boot-root-resolved-before-restore-001*

Any boot-time decision keyed on "the active project" (subscriptions, per-project services, root-keyed config lookups) must execute AFTER the workspace/shell restore — the launch-cwd discovery root is PROVISIONAL and the restore re-syncs project_root to the restored active project. Placing the decision at the discovery site silently keys the wrong root on Finder/Dock launches, restored-workspace boots, and launcher boots. Mirror the recents-fold placement (guarded on project_count() > 0, reading shell.active_project().root).

## PR-claude-branch-form-wrap-not-modulo-for-small-fixed-lists-001
*severity: medium · prevents: BF-modulo-wrap-equivalent-mutant-at-small-n-001*

For a wrapping selection index over a SMALL FIXED list (n=2, and generally any small n), do NOT write the wrap as modulo arithmetic `(sel + n - 1) % n` / `(sel + 1) % n`: at n=2 a `-1` ≡ `+1` (mod 2), so cargo-mutants' `- → +` mutant is EQUIVALENT and survives → MSI-100 gate fails, and no unit test can kill it. Use the explicit branch form instead — `if sel == 0 { n - 1 } else { sel - 1 }` (up) and `if sel + 1 == n { 0 } else { sel + 1 }` (down) — where each arithmetic mutant yields a wrong or out-of-bounds index that a wrap test (0→1→0, both directions) catches. Verify with `cargo mutants -f <file>` (0 missed) before declaring the pure state done.

## PR-claude-buffer-dont-drop-events-during-transient-states-001
*severity: medium · prevents: BF-claude-terminal-echo-dropped-during-transient-state-001*

When a state machine ignores incoming events during a transient state (receipt-in-flight, connecting, loading), check whether any droppable event is TERMINAL in its sequence — "a later event will heal the gap" is only true for non-final events. A terminal event dropped during the transient window never heals: buffer it (max-join for monotone lifecycles) and consume the buffer when the transient state resolves. Sibling rule: any queue whose only drain can PARK (a launcher/zero-project state, a suspended consumer) must be bounded by construction — a per-key max-join map bounded by population beats a cap-and-drop Vec when the consumer folds a monotone join anyway.

## PR-claude-build-test-fixtures-via-the-real-api-not-hand-assembled-001
*severity: medium · prevents: BF-claude-unbuildable-test-fixture-for-leaf-split-algebra-001*

For a CONSTRUCTIVE data-structure algebra (a tree/graph built only through its mutating API — e.g. a pane-group where `split` only ever replaces a LEAF with a 2-child node, `close` only collapses), BUILD every test fixture by CALLING the real API in sequence, never by hand-assembling the enum/struct into the shape you imagine — the API may be unable to produce that shape, making the fixture (and its asserted expectations) a latent lie. At design/inspect time, PROBE empirically: construct the fixture via the API, print it (`{:?}`) + its derived views (e.g. `panes()`), and re-derive the expected results (e.g. `neighbor`) from the ACTUAL tree per the spec's rule — do not trust a geometric/intuitive picture. This is doubly important when the spec's rule (e.g. an adjacency rule with a defined tie-break like 'take the last child for Left/Up moves') diverges from naive intuition: the test must assert the SPEC's rule on the REAL tree, not the intuitive answer on an imagined tree.

## PR-claude-cache-inertness-prove-via-behavior-not-the-field-001
*severity: medium*

When a design makes a stale cache INERT by non-consultation (an early-return that precedes every read of it) rather than by clearing it, prove the inertness through the CONSULTED BEHAVIOR — the observable output the feature would have changed (e.g. "the selection is byte-identical to the caret; ⌃W did not grow it") — NOT by asserting the internal cache/position FIELD is empty. If that field is a shared, view-scoped global (a RootView field like selection_ladder/selection_ladder_at, not a per-editor value), a single-window test that switches state across arms (open .rs, ⌃W, then switch to .json) will legitimately still OBSERVE the value the first arm left there — the gate deliberately did not clear it — so a field assert fails on correct code. Two fixes: (a) assert the behavior, not the field; or (b) boot each arm in an INDEPENDENT window/context so no shared global carries across. Corollary anti-vacuity: the two-arm fixture must be REAL content of the active language (fn f(){...} with genuine structural nodes), so the inert arm proves the LANGUAGE GATE, not merely "this file happened to have no valid nodes." (Recorded for #350; the AAR 34b5f8ea references this under the longer over-length slug ...prove-via-consulted-behavior-not-the-field.)

## PR-claude-cache-key-every-field-load-bearing-when-a-reset-breaks-mono-001
*severity: high*

When a cache is keyed by a multi-field tuple (e.g. `(nonce, version)`), do NOT assume one field alone identifies the entry — verify each field is load-bearing by tracing what RESETS or RE-MINTS the other. A field you think is "globally monotonic" often is not: in Marley, `reload_active` resets a buffer's `version` to `initial()` (0) AND re-mints its `nonce`, so `version` is monotonic only WITHIN one buffer-life, not across a reload. Consequences for a `(nonce, version)` cache guard: a version-only match false-HITs a freshly-reloaded v0 buffer against a stale `(otherNonce, 0)` entry (0 == 0); a nonce-only match false-HITs the pre-edit tree after an edit (nonce stable, version bumped). Both are silent-wrong reads (a stale cached value, no crash). So the guard MUST be an exact AND of every key field, and the reviewer must trace the reset/re-mint path for each to prove necessity — an AND-guard that looks like belt-and-suspenders may have both halves load-bearing. #349's `*cn == nonce && *cv == version` bracket-match tree-cache guard is correct precisely because the reload path makes each half necessary.

## PR-claude-cache-keys-need-identity-not-just-a-restarting-counter-001
*severity: medium*

Never key a derived-data cache on (name, counter) when the counter RESTARTS per instance (marley_editor::BufferVersion starts at 0 for every fresh Buffer): a reopened/recreated instance replays old keys and serves stale data with no error. Mint a process-monotonic identity (AtomicU64 fetch_add at construction) and key on (identity, counter). Symptom to watch for in review: any HashMap/memo keyed by a path or id PLUS a version/generation that is not globally monotonic.

## PR-claude-cached-server-ranges-must-extend-to-the-live-caret-001
*severity: high · prevents: BF-completion-accept-applies-request-time-range-to-a-typed-buffer-001*

A position/range an external service computed is valid ONLY for the document state it was computed from. If your UI CACHES that answer and lets the user keep typing (an as-you-type popup that filters a snapshot rather than re-querying), the cached ranges silently rot while the popup looks fine — and the moment of use is an EDIT, so applying them verbatim corrupts text. Rule: at apply time, reconcile the stale range against the LIVE cursor rather than trusting it (for LSP completions: `start.min(caret)` / `end.max(caret)` — the spec requires the range to contain the request position, so the client owns everything typed since). Two things make this class nearly invisible: (1) the NO-RANGE fallback path is usually written against the live caret and is therefore CORRECT, so the bug fires only against real servers that do send ranges and never against a hand-written fixture; (2) the obvious test — "open the popup, accept, assert the text" — accepts with ZERO keystrokes in between and passes straight over it. Any test for a cached-answer feature MUST exercise the gap: open → type → accept. Same shape wherever a cached remote answer is later APPLIED (a rename's ranges, a code action's edits, a formatting result computed before the last keystroke).

## PR-claude-callback-stored-inside-owned-resource-captures-weak-001
*severity: medium · prevents: BF-claude-wry-ipc-closure-strong-rc-cycle-001*

When registering a callback with a resource (webview/watcher/subscription/timer), first trace where the callback is STORED. If the resource retains the callback AND the callback's captured state owns (or transitively owns) that resource, a strong capture (Rc/Arc clone) forms a cycle that makes the resource's Drop unreachable — teardown then happens only at process exit, which silently invalidates any drop-path behavior you think you are observing or testing. Capture Weak (Rc::downgrade/Arc::downgrade) and upgrade-or-return inside the callback. Applies directly to wry 0.56 (with_ipc_handler and every with_*_handler is retained inside the WebView, released only in its Drop — wkwebview/mod.rs:144/:553/:1423); check the same for gpui subscriptions and any FFI delegate object. Verify by tracing the retain site in the dependency's source, not by assuming the closure dies with your struct.

## PR-claude-canonicalize-before-containment-check-on-external-write-path-001
*severity: high*

Before writing to a path that came from an EXTERNAL source (an LSP WorkspaceEdit uri, a tool response, any network/protocol-supplied path), a workspace-containment check with `path.starts_with(root)` is INSUFFICIENT: `Path::starts_with` is COMPONENT-WISE and does NOT resolve `..`, so `<root>/../../.zshrc` starts_with `<root>` returns true while `fs::write` resolves the `..` to a path OUTSIDE the root — a real RCE primitive (append shell code to ~/.zshrc / ~/.gitconfig / ~/.ssh/authorized_keys; the file exists so it survives a read-gate + size/binary checks). A symlink inside the workspace pointing outward is a second bypass. The naive `file:///etc/passwd` absolute path IS blocked by starts_with; only traversal/symlink escapes it — which is exactly the case a quick test misses. FIX: canonicalize the target BEFORE the check (`path.canonicalize()` — the file must exist to be written, so it succeeds and collapses `..`+symlinks), `starts_with` a CANONICALIZED root, and then read/write the CANONICAL path so the traversal can't sneak back in. Reject on canonicalize error. Note `marley_project::resolve_under_root` is NOT a containment primitive — it returns an absolute path unchanged; do not reach for it here. Caught at #322 inspect (my own first containment guard had exactly this hole; the parse critic found it).

## PR-claude-cap-cargo-jobs-late-session-to-avoid-build-deadlock-001
*severity: medium*

After a long autonomous session that has spawned MANY concurrent background cargo runs (parallel test builds, subagents each invoking cargo) and/or killed rustc mid-build, a DEFAULT full-parallelism workspace build or gate (`scripts/gates.sh`, `cargo build/clippy/nextest --workspace` at N = all cores) can DEADLOCK on the gpui-heavy crates: rustc/clippy-driver sit STAT S at 0.0% CPU for MINUTES, the "Checking …" log frozen, the SAME PIDs stuck — NOT resource starvation (memory free, load moderate). Distinguish a real stall (0% CPU + frozen log + same PIDs) from slow-but-healthy progress (CPU > 0, or PIDs cycling / log advancing — e.g. a gpui BIN's `--list` is legitimately slow but completes). THE FIX: cap parallelism with `CARGO_BUILD_JOBS=4` (proven — a `clippy --workspace` frozen 12 min completed in 3m27s job-capped; the whole gate then ran green job-capped, just slower). RECOVERY from a stall: (1) `pkill -9` ALL cargo/rustc/clippy-driver/cargo-nextest/cargo-mutants — leave NO orphaned 0%-CPU rustc (orphans block the next build); (2) verify the toolchain with a tiny crate (`cargo check -p <small-crate>`, seconds); (3) optionally clear the stalled crate's `target/debug/incremental/<crate>-*`; (4) re-run with `CARGO_BUILD_JOBS=4` (+ `CARGO_INCREMENTAL=0`). Do NOT lower a gate floor or skip a step to route around it — the cap makes the SAME full gate pass, just slower.

## PR-claude-cap-client-size-before-alloc-pre-auth-001
*severity: high*

On any PRE-AUTH request path (before origin/bearer/permission checks run), NEVER allocate a buffer sized by a client-controlled value (`Content-Length`, a declared array length, a frame size). `vec![0u8; n]`/`Vec::with_capacity(n)` with an attacker-chosen `n` is an OOM → process-abort DoS that takes down the whole app. Cap the value against a sane maximum and reject (400/413) BEFORE allocating; prefer running the auth guards on the already-read headers before touching the body.

## PR-claude-caret-anchored-card-fixes-its-anchor-and-drops-on-line-change-001
*severity: medium · prevents: BF-signature-card-anchors-to-moved-caret-001*

A card opened from an ASYNC LSP answer and anchored at the caret must (a) snapshot a FIXED anchor position at open (row + column) and render THERE, not re-read the live caret each frame — else it glides along the line as the cursor moves; and (b) drop the answer at apply-time when the live caret has left the request's line (compare the live caret row to the request key's line), because dismiss-on-caret-move can't catch a response that anchors itself to the already-moved caret. NOTE the guard must be a LINE/position check, NOT a buffer-version check, when the feature is meant to PERSIST across edits that don't re-request (signature help survives typing args, which bumps the version with no new request) — a version drop would kill it on every keystroke.

## PR-claude-caret-placement-path-must-clear-marked-001
*severity: low · prevents: BF-claude-goto-restore-missing-clear-marked-001*

Every caret-PLACEMENT path (a jump, a go-to-line/symbol/definition, an Esc-restore, a click-through) must call clear_marked BEFORE writing the caret, symmetric with every sibling placement — a stale IME composition (marked) span after an out-of-band caret move misdirects the next IME replace. When adding a new placement site, grep the existing ones for clear_marked and match them; an asymmetry (one placement clears, another doesn't) is the tell. Prefer the shipped set_single_caret helper for the write, but note it does NOT bundle clear_marked, so keep the explicit clear_marked before it.

## PR-claude-cargo-mutants-guard-mutants-depend-on-syntactic-form-001
*severity: low*

cargo-mutants generates DIFFERENT mutants for the SAME logical boolean guard depending on its SYNTACTIC form. A `match … if GUARD => …` arm yields THREE mutants (replace the guard with `true`, with `false`, AND `delete !`). But an `if let Some(x) = opt { if !cond { … } }` — or any plain `if !cond { … }` statement — yields ONLY `delete !` (no true/false replacement). So refactoring a match-with-guard into an if-let / if cascade SHRINKS the mutant set: do NOT carry over the old form's expected true/false cases; RUN `cargo mutants --list -f <file>` on the ACTUAL post-refactor code and write exactly the killers. Bit #201: the design assumed 8 `display_title` mutants (guard true/false/delete-! ×2 + body ×2); the real set was 4 (body ×2 + delete-! ×2). Reinforces PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001 — that rule says 'trace, don't guess operators'; this adds 'the guard-mutant SET itself depends on the match-arm-vs-if syntactic form'.

## PR-claude-carry-a-selection-endpoint-by-offset-span-not-row-001
*severity: high · prevents: BF-claude-line-carry-attributes-endpoint-by-row-not-offset-span-001*

When carrying a SELECTION through a line-block edit (move/duplicate/reorder), attribute each endpoint to its block by OFFSET SPAN — `[line_start(b0), line_start(b1+1)]` — NOT by the endpoint's raw row. A "full lines" selection's HEAD is not on any line the block contains: `line_span`'s col-0 carve puts it on the row PAST the block, at the block's exclusive-end offset, so a row-based block lookup misattributes it (gets no delta) and silently distorts the carried selection while the TEXT stays correct — no text/round-trip test catches it; only a selection-SHAPE assertion on a full-line (Home+⇧↓) selection does. The boundary's inclusivity is direction-sensitive: include it when the op's own edit does NOT already move that offset (a length-preserving move swap), exclude it when the edit does (a duplicate-down insert AT the boundary, which `rebase_through` already carries). Always add a full-line-selection carry test, not just a bare-caret one.

## PR-claude-carry-accumulated-state-across-a-reconnect-for-delta-streams-001
*severity: high*

When a client reconnects to a cursor'd/delta event stream (reads only events AFTER a durable cursor), it MUST carry its accumulated reduced state forward across the reconnect — do NOT rebuild from an empty state and replay only the post-cursor delta, or every entity established before the cursor is silently dropped on each reconnect. A max-join/idempotent reducer removes duplicates but cannot resurrect an entity whose event is never re-sent. Litmus: write the reconnect test so the post-reconnect read returns a STRICT DELTA (only the new entity, no re-send of prior ones) and assert the full set survives — a fixture that re-sends everything (cumulative pages) hides the bug. This is distinct from a fresh app restart, where starting empty is correct but the server must then serve a from-the-beginning/snapshot read.

## PR-claude-catalogue-doc-line-refs-regenerate-after-code-edits-001
*severity: low*

When a ticket authors a catalogue/reference doc containing file:line refs AND edits those same files in the same ticket, (re)generate the line numbers AFTER the code edits — via a fresh grep, not hand-math — or grep-verify them at Complete. Refs captured before the edits go stale by the net line delta (bit #232's icon-audit.md, BF-claude-audit-doc-line-refs-stale-after-same-ticket-edits). For durable arch docs prefer greppable anchors (fn / surface name + the token/escape) over bare line numbers, and note that line numbers drift. Line refs in files the ticket does NOT touch stay accurate.

## PR-claude-cfg-conditional-as-const-001
*severity: high · prevents: BF-cfg-gated-fn-unkillable-mutant-001*

A platform/host-conditional VALUE (chosen by `#[cfg(...)]`) must be a `#[cfg]`-gated `const` (or `static`), NOT a `#[cfg]`-gated `fn`. On a single-platform CI runner, cargo-mutants generates a mutant for the cfg'd-OUT function body that never compiles on the active platform → the build is unchanged → the mutant survives unkillable → MSI < 100, and `#[mutants::skip]`/exclusions are banned (§0 zero-exclusion). cargo-mutants does not mutate `const`/`static` items and a const decl is not an executable coverage line, so the gated const has zero mutants and zero coverage gap. Applies to any per-OS constant: path flavor, line separator, default shell, exe suffix — directly relevant to marley_command (TICKET-004) which will have Windows-specific values. (For conditional LOGIC, prefer one always-compiled fn with `#[cfg]` blocks inside, exercised on the active platform, over two cfg-gated fns.)

## PR-claude-cfg-test-helpers-land-with-their-tests-001
*severity: low*

A `#[cfg(test)]` helper written in the IMPLEMENT phase (before its Validate-phase test callers exist) is dead code under the gate's `clippy -D warnings` — but NOT under `cargo check` (which the implement phase runs). This is EXPECTED, not a defect: the caller lands with the Validate tests and closes the warning exactly when the gate first runs. Don't panic at the intermediate warning; DO ensure the Validate test plan calls every such helper (or the gate stays red). Mirror the shipped precedent (a cfg(test) persist helper + its round-trip test land together).

## PR-claude-check-tests-after-signature-change-001
*severity: medium*

After changing a PUBLIC function signature (adding/removing a param) or a struct's fields during implement, run `cargo check -p <crate> --tests` (or `cargo nextest run --no-run`) — plain `cargo check` skips `#[cfg(test)]`, so broken test callers and struct literals stay invisible until validate or the /commit gate. A signature/field change fans out to EVERY caller, including test callers; confirm the TEST target compiles before declaring Implement PASS, so the breakage surfaces in-phase rather than as a surprise pile of errors at validate.

## PR-claude-check-your-own-warning-comment-applies-to-the-branch-below-001
*severity: high*

When you write a comment explaining why some shape is HARMFUL, immediately check the code you are about to write directly beneath it. On Marley #297 I wrote, verbatim: "a group per keystroke would make ⌘Z undo one CHARACTER at a time instead of a typed word — destroying the #253/#282 feel" — fixed it for the N=1 case, and then in the `else` branch six lines below did EXACTLY that for every N≥2 keystroke. Typing "hello" at 3 cursors took 5 ⌘Z. A critic found it and pointed out that the code was violating a principle it had just written down. The mechanism of the trap: you reason about the harm in the case you are fixing, feel the reasoning is discharged, and never re-apply it to the neighbouring branch. THEREFORE: after writing any "this would be bad because…" comment, re-read every sibling branch of the construct you are in and ask "does this harm apply here too?" — a warning comment is a spec for ALL the arms, not a footnote on the one you were thinking about. Corollary for the same class: a 1-vs-N (or fast-path-vs-slow-path) split is where behavior silently diverges — test the N case against the SAME acceptance criteria as the 1 case, not merely "does it work".

## PR-claude-clamp-adjust-tests-must-distinguish-each-branch-001
*severity: medium*

A pure index-clamp / adjust-after-remove fn with multiple branches (e.g. `if active > idx { active-1 } else if active >= len { len-1 } else { active }`) needs unit tests where EACH branch produces a DISTINCT result — otherwise the comparison-operator mutants (`>`→`==`/`<`/`>=`, `<`→`<=`) survive even though the code is correct. Two traps: (1) only testing the LAST-index case makes the decrement branch and the >=-clamp branch coincide (same value → the swap is invisible); test the active-AFTER-idx case (distinct decrement) AND the active-MIDDLE case (active==idx, not last → stays) AND the active-last case (clamps). (2) A bounds guard `idx < len` needs a test at idx == len EXACTLY (`<` vs `<=` only differ there); `activate(bignum)` doesn't hit the boundary. Run `cargo mutants -f <file>` on any pure clamp/adjust seam and confirm 0 missed. Bit #237 (EditorSurface::close/activate — 4 survivors until distinguishing cases added).

## PR-claude-clamping-setter-does-not-clamp-the-readback-001
*severity: high · prevents: BF-lsp-navback-stale-offset-line-col-panics-001*

When a setter CLAMPS its input, the clamped value exists only in the state — any downstream read that needs the EFFECTIVE value must read it BACK from the state, never reuse the requested value. In Marley: set_single_caret(off) clamps via set_selection → clamp_char_offset, so the row must come from ed.active_caret(), not from `off`. Corollary (the part that actually bites): a sibling accessor that clamps is NOT evidence that this one does. Buffer::line_start clamps and documents "never panics"; Buffer::line_col two lines below is an unguarded rope.char_to_line that PANICS out of bounds. Clamping is opt-in per-accessor — read the specific accessor's contract before feeding it a stored/replayed offset. Any offset that outlives the buffer snapshot it was taken from (a jump-back stack, an undo record, a bookmark, an anchor-less position) is a stale-offset candidate: route it through the clamping setter first, then read back.

## PR-claude-clear-input-only-after-confirmed-delivery-001
*severity: medium*

When an action CONSUMES user-composed input (a prompt line, a compose field, a form) by sending/moving it somewhere, clear the source ONLY after the delivery is CONFIRMED — the target existed AND the write/insert succeeded. NEVER clear unconditionally on the happy-path branch, because a stale target (a closed pane, a dropped connection, a None lookup) or a failed write then silently DROPS the user's input with no feedback — the worst UX failure (lost work, no error). Pattern: `let delivered = match target() { Some(t) => t.write(payload).is_ok(), None => false }; if delivered { clear_source() }` — not `target().map(|t| t.write(payload)); clear_source();`. This class is INVISIBLE to cov/MSI when the code is a masked app shim — only manual inspect or a self-test that closes-the-target-then-sends catches it. Related: the cockpit's last_agent/last_target pointers can go stale (a pane closes without clearing the pointer) → always treat a target lookup as fallible.

## PR-claude-clearmods-before-driven-clicks-001
*severity: medium*

During driven live-app validation (scripts/selftest/drive.swift), a synthetic chord (cmd:X, cmdshift:X, rightclickat) can leave the session's logical modifier state STUCK, so a LATER "plain" clickat is delivered as a ⌘-click (or shift-click) and silently does the wrong thing — e.g. a file-tree click that should open a code view does nothing. Symptom: directory toggles work but file-opens/plain-clicks mysteriously no-op after you've used chords earlier in the run. FIX: prepend the `clearmods` action to any drive.swift click sequence that follows earlier chords — e.g. `drive.swift clearmods focus wait:500 clearmods clickat:0.2,0.3`. drive.swift has a dedicated `clearmods` action (posts key-ups for every modifier vk) precisely for this. Also: file-tree rows have NO w_full, so their hit area is only the icon+text — click squarely on the filename text, not the empty space to its right.

## PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001
*severity: medium*

When a shim GATES behavior on a closed enum (a tab kind, a pane kind, a variant), use an exhaustive `match` on the enum, NOT an `is_some()`/accessor predicate chain with a catch-all `else`. A predicate chain (`if x.grid().is_some() {…} else if x.cockpit().is_some() {…} else {…}`) compiles fine when a new variant is added but silently routes it into the last/`else` arm — a latent mislabel/misbehavior with NO build error. An exhaustive `match &x.content { A => …, B => …, C => … }` makes a NEW variant fail the build AT THE GATE, forcing the author to handle it. This is the pure-side exhaustiveness discipline extended to the shim's enum gates. Inspect #382 F1: the footer tab-gate used `grid()`/`cockpit_section()` `is_some()` → `else = Editor`; a future `TabContent::Browser` (a roadmap pillar) would have silently read "editor". Fixed to `match &active_tab().content`. Applies wherever a closed domain enum drives a decision — prefer the field/enum match over accessor-`is_some()` chains for exactly the same reason the pure seam bans `_ =>` wildcards.

## PR-claude-commit-hook-checks-staged-index-001
*severity: high · prevents: BF-changelog-hook-worktree-vs-staged-001*

A PreToolUse `git commit` hook that requires (or forbids) a file in the commit must inspect the STAGED INDEX — `git diff --cached --name-only` — NOT `git status --porcelain` (the worktree). The worktree includes untracked/unstaged paths that won't be in the commit, so a present-but-unstaged decoy satisfies the gate, and an untracked *required* file makes the gate silently inert. Anchor required-file paths to the repo root (`^FILE$`, not a substring/any-dir match). This assumes the flow stages before committing (`git add -A` as its own step, then `git commit`); document that a chained `git add … && git commit` stages after the PreToolUse hook runs and isn't introspected. (The content-fingerprint commit-gate is exempt from this because it hashes worktree CONTENT against a receipt, which is staging-invariant.)

## PR-claude-compare-path-identity-canonically-not-raw-eq-001
*severity: high*

When comparing a path from an EXTERNAL/canonical source (an LSP server's uri, a tool's output — anything that routed through canonicalize()) against a path your app stored NON-canonically (a `root.join(rel)` where the root was kept verbatim), NEVER use raw `PathBuf ==`. The spellings diverge under a symlinked root (/tmp→/private/tmp on macOS, symlinked $HOME, /Volumes automounts), an unresolved `..`, or an APFS case difference — and a plain dev path (/Users/name/code/proj) does NOT trip it, so it passes all local testing then bites a symlinked user. Compare CANONICAL forms: `match (a.canonicalize(), b.canonicalize()) { (Ok(ca),Ok(cb)) => ca==cb, _ => a==b }` (raw fallback when a side does not exist yet). Concrete miss (#322 HIGH): a rename's open-vs-closed routing used raw `==`; under /tmp→/private/tmp an OPEN file read as CLOSED → the rename blind-wrote to DISK instead of the buffer → dirty-buffer file corruption. The LSP host already canonicalized its OWN copies for exactly this reason — the bug was the ONE seam (the editor surface) that had not adopted the discipline. When one subsystem canonicalizes its keys, every subsystem comparing against its values must too.

## PR-claude-composite-type-over-one-edit-001
*severity: medium*

Any editor op that REPLACES a selection and inserts derived text (type-over Enter, paste-with-transform, snippet expand) must be ONE Buffer::edit(sel_start..sel_end, &derived), never delete-then-insert: two edits = two undo records, and the first ⌘Z lands on a state the user never saw (see BF-claude-enter-type-over-was-two-undo-steps, Marley #276). Compute the derived text BEFORE the replace from inputs the replace cannot change (e.g. chars strictly before the selection start); if the derivation reads anything at-or-after sel_start it must be computed pre-delete and proven invariant. Pin with an undo-count assert (one undo() restores the exact pre-op text), not just a text assert.

## PR-claude-compound-token-affordance-covers-whole-token-001
*severity: medium*

When adding a click/hover affordance over a COMPOUND token that has a primary sub-token plus modifiers (a `file:line:col` ref, a `user@host:port`, a `pkg@version`), make the affordance's hit-region cover the WHOLE token, not just the primary sub-token. A range scoped to only the primary part (e.g. #196's link range = the path, not the `:line:col`) means clicking the modifier portion silently does nothing — the user naturally aims at the `:12`. When EXTENDING an existing affordance whose range was the sub-token, widen the range to the full trimmed token and confirm the pre-existing tests only change for the compound case (a modifier-less token keeps the old range). Caught in #212 inspect (F1).

## PR-claude-const-arithmetic-in-diff-use-literal-001
*severity: medium*

A NEW `const` whose value is an arithmetic expression (`2 * 1024 * 1024`, `1 << 20`) leaves mutable operators that cargo-mutants (`--in-diff`) mutates — and no behavioral test can pin a bare const's value, so those mutants are unkillable → MSI < 100. Write size/limit consts as a bare literal (`2_097_152 // 2 MiB`). Existing literal consts elsewhere aren't flagged (not in the diff, no operators).

## PR-claude-consumed-license-cleared-on-every-exit-path-001
*severity: low · prevents: BF-claude-extchange-armed-write-leaves-stale-arm-001*

A one-shot license/acknowledgment (a ⌘S arm, a confirm flag, a nonce) CONSUMED by an action must be cleared on EVERY exit path of that action, not just the common one. In #284 the armed-write's SUCCESS path cleared the arm (via set_active_conflict(None)), but its racing-write branch (a concurrent write raced the write gap → re-flag Changed) left armed_at set — the license outlived its consuming write. When an action has multiple terminal branches (ok / racing / error), audit each for the state it must reset; a single reset before the branch split is safer than per-branch resets one branch forgets.

## PR-claude-contrast-test-is-blind-to-slot-distinctness-001
*severity: high · prevents: BF-claude-aa-passing-palette-shipped-two-identical-slots-001*

A WCAG contrast / AA test proves each color is legible AGAINST THE BACKGROUND — it is structurally BLIND to whether two colors are distinct FROM EACH OTHER. Two syntax/palette slots can both clear AA and still be the same color (identical hue + lightness, differing only in saturation), so a role-coded palette (keyword vs property, type vs number, …) needs a SECOND gate: a pairwise distinctness test. Metric that works: two slots collide when close on hue (treat hue as CIRCULAR — 0.02 and 0.98 are 0.04 apart) AND lightness AND saturation — the saturation clause lets a near-gray comment differ from a saturated function at a close hue+L. When tuning a palette, VERIFY BOTH gates (AA vs background AND pairwise distinctness) by COMPUTING the numbers, not eyeballing — and watch the AA MARGINS too: a slot at ~4.5:1 is brittle (a 0.01 lightness nudge or a color-space rounding change flips it sub-AA), so tune for real headroom (≥~4.8) rather than just-passing. An adversarial critic that computes the pairwise deltas catches what the AA matrix cannot.

## PR-claude-copied-stale-key-must-be-read-or-deleted-001
*severity: low · prevents: BF-copied-stale-key-field-never-read-001*

When adapting a sibling feature's stale-key/request-tracking field (e.g. copying #325's `symbol_request: Option<Key>`), verify the NEW accept-path actually READS it. If the new design gates staleness on a scalar generation (`accept_gen(msg_gen, self.gen)`) instead of comparing the whole key, the copied Option<Key> field becomes write-only dead state — a needless clone, misleading "reopen protection" comments, and a dead struct. Either wire the accept to read the field (single source of truth) or delete the field + struct. Grep the field for READ sites (`.as_ref()`, match, `.gen`/`.query`) before keeping it; zero reads = delete it.

## PR-claude-cov-100-pure-modules-stay-generic-free-001
*severity: medium · prevents: BF-claude-llvm-cov-phantom-line-from-generic-instantiations-001*

A pure module under the 100%-line coverage floor must not define generic fns or take `impl Fn*` parameters — write `&mut dyn Fn*` for closures and concrete types otherwise. Generic instantiations get inlined into every linking test binary's rlib covmap, and the never-run copies in binaries that don't call the module produce a phantom "missed line" in llvm-cov's line-summary merge that NO per-line view (lcov DA, annotated text, region cross-reference) can locate. When diagnosing a coverage number that per-line views contradict: (1) always `cargo llvm-cov clean` before an ad-hoc measurement — profraws merge across stale builds; (2) never trust an exit code measured with `--show-missing-lines` added — the flag changes llvm-cov's exit pathway and can flip a fail to a pass; reproduce with the gate's verbatim flags.

## PR-claude-cov-excluded-is-not-mutants-excluded-shim-needs-skip-001
*severity: high*

Coverage-exclusion and mutation-exclusion are SEPARATE mechanisms — do not assume one implies the other. A fn whose FILE is in the llvm-cov `--ignore-filename-regex` (so gate:4 coverage ignores it) is STILL mutated by cargo-mutants (gate:5), which does not consult that regex. An app-side / integration-only shim (a gpui metric read, a process-spawn adapter, a render measurement) that has no unit-testable seam therefore needs an EXPLICIT `#[cfg_attr(test, mutants::skip)]` on the fn IN ADDITION to the file-level cov exclusion — otherwise its body/return/match mutants survive → MSI < 100 → gate RED. When adding such a shim, mirror the sibling shim that already solved it (Marley: font_resolves, editor_draw_for, the marley_command adapters all carry the skip). Corollary at design/implement time: if the decision is extracted to a pure unit-tested fn (correct), the thin app-side caller that only reads gpui/OS state and delegates is a shim → skip it; the pure fn carries the cov/MSI.

## PR-claude-cov-gate-is-lines-not-regions-001
*severity: low*

This repo's coverage gate (scripts/gates.sh gate:4) is `cargo llvm-cov --fail-under-lines 100` — it enforces the LINE metric, NOT the REGION % that leads the llvm-cov summary table. The summary shows regions first (e.g. TOTAL 99.67%), which can look like a failure while lines are 100% (0 missed lines) and the gate passes. When gate:4 is red, read the LINES columns (the 3rd triple: total/missed/cover), not the region %. A file can be 88% regions but the gate only cares once it has MISSED LINES. To diagnose: `cargo llvm-cov ... --show-missing-lines` and look at Missed Lines, or diff a clean-baseline stash to separate a real regression from pre-existing region gaps that never gated.

## PR-claude-covered-anchor-collapses-use-linecol-reseat-for-full-replace-001
*severity: medium*

An Anchor carries a caret through an edit ONLY when the edit does not COVER the anchor's position. A covered Bias::Left anchor collapses to the edit's span start; a covered Bias::Right to its end. So an Anchor CANNOT preserve an interior caret through a WHOLE-covering replace — e.g. a document-formatting response that arrives as one whole-file TextEdit edit(0..len, new): every anchor inside collapses to 0 (Left) or EOF (Right), teleporting the caret. Before reaching for an Anchor to carry a caret/selection through an applied edit, ask whether the edit can COVER that position (a whole-file reformat, a select-all replace, an organize-imports rewrite). If yes, use a (line, column) re-seat instead: capture line_col before apply, re-seat via caret_for_line_col (clamped into the new line) after — it keeps the caret on its logical line best-effort (the VS Code/Zed behavior). Reserve the Anchor for GRANULAR edits that provably don't cover the caret. And VERIFY the carry against the edit shape the server actually emits, not the shape you assume.

## PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001
*severity: medium*

When a critic proposes fixing a bug in SHARED plumbing and asserts the other consumers are unaffected, treat that as the least-verified sentence in the report and check it consumer by consumer before accepting. On #331 a critic proposed making the LSP timeout path deliver a synthetic `Err` to every abandoned request ("siblings are unaffected — the clear is a no-op they'd have done on the next send"). Reading each sibling's `Err` arm disproved it in minutes: prepare-rename → a "Cannot rename this" flash, code-action → "Code actions failed", resolve → "Code action resolve failed" — i.e. three shipped features would fire a phantom toast TEN SECONDS after the user's keypress. The critic had traced the bug's mechanism exhaustively and simply not opened the callers of its fix. General shape: a critic's evidence for the DEFECT and its evidence for the REMEDY have different depths, because the remedy is where it stopped looking. The underlying invariant may still be right ("an abandoned request must always terminate") — take the contained fix now, file the general one as a follow-up with the blast radius named, and do not let a good invariant smuggle an unreviewed behaviour change into an unrelated ticket's diff.

## PR-claude-cross-project-affordance-must-persist-the-switch-001
*severity: medium*

In Marley, any rail/UI affordance that activates a different project via `switch_project(p)` (the #174 cross-project idiom: `project_changed = p != active_project_index(); switch_project(p); if project_changed { sync_active_project() }`) MUST call `persist_grid()` after the switch when project_changed — because `active_project` is persisted state (serialize_shell emits it as the first field; boot restores it). This holds EVEN WHEN the affordance's action opens a transient modal (a finder, a picker) rather than mutating durable tab/grid state: the modal may persist only on a later confirm gesture (e.g. the finder persists only on ⌘↵-open), so a switch-then-cancel-then-quit would otherwise lose the switch on relaunch. Verbs that add/mutate tabs (new_terminal_pane, open_or_switch_cockpit) already persist and cover the switch; the trap is the modal-opening arm that changes no durable state itself. Mirror the Tab/Pane rows, which persist unconditionally after the switch.

## PR-claude-dead-option-branch-flakes-100pct-coverage-001
*severity: medium*

On a 100%-line coverage gate, an `?`, `.unwrap_or_else(||…)`, or `.map(…).unwrap_or_else(||…)` applied to an Option that is ALWAYS `Some` (e.g. `str::splitn(n,_).next()`, or a `flatten` that always yields ≥1 leaf) creates an UNREACHABLE None/error branch. llvm-cov attributes it inconsistently across cov invocations (a plain `cargo llvm-cov --workspace` may score it 100% while the gate's `cargo llvm-cov nextest --manifest-path … --no-tests=warn` scores it <100%), so it reads as a flaky per-file coverage miss on code the diff didn't even touch. Fix at the source: use eager `.unwrap_or(value)` (no closure region) or restructure so there is no unreachable arm — do NOT chase it with a test (the branch can't be reached). Pin the exact line with `cargo llvm-cov report --json` → segments with `hasCount && count==0`, not lcov `DA:` records (which round it covered).

## PR-claude-debug-panic-release-defined-001
*severity: high · prevents: BF-debug-assert-arm-coverage-001*

For a "debug-panic, release-defined" arm (misuse should panic in debug but yield a defined result in release), write `let result = <defined_release_value>; debug_assert!(<invariant>, ...); result` — assert the INVARIANT and ALWAYS compute the defined result. Do NOT write `debug_assert!(false, ...); <release-only line>`: that line is unreachable in a debug build, so it gets zero coverage under a debug `cargo llvm-cov` run (breaks a 100% line-coverage gate), and a guard written only to gate the panic tends to carry an equivalent, unkillable mutant. The assert-the-invariant form keeps the defined result on the normal path (debug-covered + mutation-visible) while still panicking on misuse in debug. Also: drop provably-redundant loop guards (e.g. a lower-bound `i > 0` when the loop self-terminates) — they are equivalent mutants under cargo-mutants with no blanket exclusions.

## PR-claude-decision-moved-into-a-gate-excluded-file-needs-its-own-test-001
*severity: medium*

When you move a BEHAVIORAL DECISION out of a pure, 100%-covered, MSI-100 seam and into a call-site argument (a bool flag, an enum, a config value), you have moved it into a file that may be EXEMPT from the gates that were protecting it — and the gate will stay green while the behavior silently flips. On #298 I made the find bar's case-folding explicit as `find_all(.., fold: true)` at its app.rs call site. A critic flipped it to `false` — silently making the shipped find bar case-SENSITIVE — and ALL 452 app tests passed, because `app.rs` is excluded from the coverage gate AND the enclosing fn carried a `#[cfg_attr(test, mutants::skip)]` whose stated rationale ("find_all + the index clamp are pure-tested") had just become FALSE for the new argument. THE RULE: after adding an argument that encodes behavior at a call site, (1) check whether that file is in the coverage/mutation exclusion list and whether the enclosing fn is `mutants::skip`ped; if so (2) add ONE test that FAILS when the argument is flipped, and (3) re-read the skip's rationale — a `mutants::skip` justified by "the pure seam is tested" is invalidated the moment the call site starts carrying a decision the pure seam cannot see. VERIFY BY SABOTAGE: flip the value, run the suite, confirm something goes red. If nothing does, the decision is unpinned. The cheapest pin is often to make an EXISTING assertion depend on the behavior — e.g. give the find fixture a MIXED-CASE far match, so the shipped "it centered the far match" assert can only pass when folding is live.

## PR-claude-defer-redo-clear-to-transaction-commit-001
*severity: medium · prevents: BF-claude-grouped-undo-noop-transaction-wipes-redo-001*

An undo/redo transaction seam (begin_group/end_group or equivalent) must clear the redo stack on COMMIT of a non-empty group — NOT eagerly on begin. If `begin` clears redo, any no-op transaction (an empty block-indent/dedent, a 0-match replace-all) that opens then drops its group silently destroys a pending redo while changing nothing. Put the invalidation where the actual mutation is proven (a non-empty `end_group`), so it protects every caller instead of relying on each call site to guard the empty case. Verify with a regression test: begin→(no records)→end must leave redo intact.

## PR-claude-deferred-gpui-handle-op-needs-notify-in-headless-001
*severity: medium*

A headless test asserting the effect of a DEFERRED gpui handle op — `UniformListScrollHandle::scroll_to_item`, `focus`, or any queued layout effect that applies during prepaint/layout, not synchronously — must call `cx.notify()` in the mutating `window.update` block so the following `run_until_parked` renders a frame that applies it. Without the notify, no frame is scheduled, the deferred op never runs, and the `*_for_test` accessor reads the PRE-op value: the assertion then passes or fails VACUOUSLY (e.g. `editor_scroll_y` stays 0 no matter what you scrolled to). Mirror an existing proven drive (e.g. `per_file_scroll_memory_survives_switches_headless`, which notifies after `scroll_editor_to_row`). Distinct from the mock-clock pump trap (#321/#349, where the fix is `advance_clock`+`run_until_parked`): here the missing ingredient is the notify-driven RENDER, not a clock tick. Make the drive non-vacuous by construction — a fixture where the pre-fix code produces the opposite observable (e.g. the new cursor off-screen so a primary-follow would leave scroll at 0).

## PR-claude-delegation-equivalence-test-tautological-001
*severity: medium*

When a refactor extracts a pure `_from(&Tree/&T, …)` factor and makes the original delegate to it (`wrapper(x, r) = _from(parse(x), r)`), an equivalence test `assert_eq!(_from(tree, r), wrapper(x, r))` is TAUTOLOGICAL for every mutant INSIDE `_from`: both operands route through `_from`, so a factor-internal mutant (loop body, a `!=`/`==` dedup, a descent) moves both sides equally, the equality still holds, and the mutant SURVIVES that test. The factor's mutation coverage then silently rests on some OTHER (hardcoded-vector) test reached through the delegation — delete that test as "redundant now that equivalence is proven" and the factor's MSI collapses to ~0 with no red. FIX: make the equivalence unit ALSO assert `_from(tree, r)` against a LITERAL expected value for at least one representative fixture (the exact ranges/vec), so it independently kills the factor's walk/body mutants; keep the `_from == wrapper` sweep as the DELEGATION contract (it still kills the wrapper-body/`unwrap_or_default` mutants). Verify with `cargo mutants --list -f <file>` that the literal assert covers each factor mutant. Seen in #363 (enclosing_ranges/all_headers → _from) and latent in #349 (matching_delimiters_in → _from); caught by an inspect critic — MSI stayed 100 via the retained hardcoded tests, but the credit and deletion-safety were wrong until the literal asserts were added.

## PR-claude-deleting-a-guard-name-the-hazard-you-measured-001
*severity: medium · prevents: BF-claude-clamp-removal-doc-named-the-wrong-hazard-001*

When a refactor DELETES a bounds check, clamp, or guard, the replacement justification must name the failure mode you actually MEASURED in the callee, not the one you assumed. Read the callee and record what it really does with the out-of-range input: panic, clamp, saturate, or return a sentinel. A callee that CLAMPS is more dangerous than one that panics — the bad input produces a plausible wrong answer (e.g. an edit attributed to the last line) instead of a loud failure, so the upstream filter is doing MORE work than "avoiding a crash", not less. Write that in the doc. Corollary: do not add a redundant explicit guard "to be safe" when an existing filter already makes the call unreachable — that is a cold branch, an uncovered line, and an equivalent mutant; document the reliance instead and pin it with a test at the extreme (a far-past-EOF row, usize::MAX).

## PR-claude-deny-check-before-heavy-dep-tree-001
*severity: high · prevents: BF-claude-gpui-tree-deny-licenses-advisories-001*

When a ticket adds a large/transitive dependency tree (gpui, wgpu, any framework pulling hundreds of crates), run `cargo deny check` (ALL checks — licenses AND advisories AND bans) EARLY in implement/inspect, not just `cargo check`/`clippy`. A heavy tree commonly surfaces (a) licenses outside the allowlist (CC0-1.0, NCSA, Unicode, etc. — add permissive ones to deny.toml with a per-license reason) and (b) UNMAINTAINED RUSTSEC advisories that `cargo deny check` errors on but `cargo audit` only warns on (add the RUSTSEC IDs to deny.toml [advisories] ignore with a documented reason — the §0-sanctioned reasoned ignore). Note: pruning a crate's features (e.g. image default-features=false) may NOT drop a transitive dep if another crate re-enables the feature via cargo feature-unification — verify with `cargo deny check`, and fall back to allowlisting the license.

## PR-claude-derive-wire-list-from-single-registry-001
*severity: medium*

When a table of things (tools, commands, routes) has BOTH a typed registry AND a serialized "list" surface (e.g. MCP `tools/list`), DERIVE the list from the registry — never hand-repeat the names/descriptions in a parallel literal. A parallel table drifts silently: an entry added to one only becomes uncallable (list-only) or undiscoverable (registry-only), and no test catches it. Make the registry a `const` single source; the list maps over it, with only the genuinely-per-item bits (e.g. JSON schemas) in a match that mirrors the dispatch match.

## PR-claude-derived-value-change-must-sweep-all-consumers-001
*severity: medium*

When you change the SOURCE of a shared derived quantity (a row/line height, a cell size, a unit scale, a base font size), grep for EVERY consumer and update them together — not just the one the ticket names. Here the terminal row height moved from gpui's ambient `window.line_height()` (~26px) to a font-relative `fallback.h` (16.8px) for the metric + the render, but a THIRD consumer — the scroll-wheel pixel→rows divisor — still used `window.line_height()`, so trackpad scroll silently ran ~35% off. The divergence hid in mutants::skip'd shim code (no gate catch) and was non-fatal (an accumulator masked it), so only an inspect audit found it. Rule: a value that WAS consistent across N call sites because they shared one source must stay consistent — when you re-source it, sweep all N. Bind it once and reuse the binding everywhere (including inside closures — capture the Copy value) so they can't drift.

## PR-claude-destructive-keystroke-validate-via-targeted-menu-001
*severity: medium*

When live-validating a GLOBAL-destructive keystroke (⌘Q quit, and any accelerator that closes/quits) and the machine is in ACTIVE use — screen unlocked AND a non-target app frontmost (check: console user + `osascript … first process whose frontmost is true`; e.g. MSTeams/a call) — do NOT fire a global synthetic keystroke. A CGEvent/System-Events keystroke goes to the FRONTMOST app, so a stray ⌘Q could quit the user's app (or a live call), not the app under test. Instead prove the wiring with TARGETED System-Events menu ops that dispatch only to the target process: (a) read AXMenuItemCmdChar/AXMenuItemCmdModifiers on the menu item to confirm the accelerator is empirically wired (for a gpui menu this is ALSO the set_menus+bind_keys-ordering witness — the equivalent is present only if the binding was registered before the menu); (b) menu-CLICK the item to dispatch the SAME action live and assert the app exits + teardown is clean. Defer only the raw global-keystroke fire to an idle/unattended machine, and record it as covered-by-mechanism (the keystroke routes through the identical handler the menu-click exercised) — not as unverified.

## PR-claude-detection-tracks-runner-001
*severity: medium · prevents: BF-tests-ran-runner-detection-001*

When you change a canonical command that a hook/gate also DETECTS (e.g. swapping the test runner cargo test → cargo nextest), update every detector in lockstep — the transcript-scanning regex, the blocked-message hint, AND the invocation — not just the invocation. A detector that lags the runner spuriously blocks the very command it now prescribes. Grep the whole apparatus for the old token before declaring the swap done.

## PR-claude-diff-gate-mutation-tests-touched-shim-fn-001
*severity: medium*

> **Superseded in the fork (2026-09-22, TICKET-436).** The advice below is gpui-era. The fork's
> gate:12 bans `mutants::skip` in any form (`AD-claude-443-mutation-topology-and-no-masks-001`):
> a shim function that a diff touches is tested (a gpui driven test, a loopback or real-PTY
> test) or re-expressed so that its logic lives in a tested pure function.

In the `--diff` gate, cargo-mutants tests a function the moment ANY line of it enters the diff — even a long-committed app.rs shim function that was never previously mutated. So when you edit an untestable live-gpui-state shim fn (one that reads Window/Context/live grids), add `#[cfg_attr(test, mutants::skip)]` in the SAME change if it lacks one, matching the established app.rs convention (~79 siblings). This is not a floor-drop per §0: keep all real DECISION logic in the pure crates (marley_agent/agent_view) where it stays coverage+mutation-tested at 100%; the shim fn is pure orchestration over live state. Symptom if you forget: gate:5 reports one surviving mutant `replace <RootView::fn> with ()` while coverage is still 100%.

## PR-claude-diff-gate-stage-new-crate-before-mutation-001
*severity: high*

A NEW crate/file must be `git add`-ed BEFORE running `scripts/gates.sh --diff` (or any `cargo mutants --in-diff`). `--diff` mutation scopes to `git diff HEAD`, which EXCLUDES untracked files — so a brand-new crate's mutants are NOT tested until it is staged (the first #308 --diff run mutated only 6 tracked-edit lines and reported a false MSI 100%; staging the new marley_lsp files surfaced the full 127 mutants and caught 4 real gaps). Coverage (gate:4) is whole-workspace and DOES see untracked files, so it can pass while mutation silently skips the new code — do not trust a --diff green on a pipeline that adds files without confirming the new files are in `git diff HEAD --stat`.

## PR-claude-diff-phase-3-against-your-own-phase-2-manifest-001
*severity: high · prevents: BF-claude-dropped-my-own-render-half-and-passed-the-phase-001*

Before writing "Implement PASS", DIFF what you built against the Phase-2 file manifest you wrote — item by item. "PASS" is a checklist result, not a feeling.

M22 #339 shipped state (efind_regex/efind_fold/efind_error) + logic + palette, then declared Phase 3 PASS while SILENTLY dropping the manifest's "two chips + inline error row" render. The result was a write-only field that rendered a user-facing LIE ("no matches" for a pattern that never compiled), and NO gate could see it — a private struct-literal-initialized field that is only assigned emits no rustc dead_code warning, and app.rs is coverage-excluded + mutants::skip. Two critics caught it; the toolchain could not.

THE CHECK (cheap, mechanical):
1. Open the Phase-2 `.notes.md` file manifest. For each row, confirm the file was actually touched the way the row says. `git diff HEAD --stat` vs the manifest list. A manifest row with no corresponding diff is either DONE-elsewhere (prove it) or a CUT (record it + file the follow-up) — never silent.
2. For a UI ticket specifically: **state without render is DEAD STATE, not a feature.** Grep every new field for a READER, not just a writer. A field with writes and zero reads is the tell. `grep -n "self.<field>" | grep -v "= "` — if that is empty, the field does nothing yet.
3. "It compiles and the units pass" is not "the phase is done." A field can be written, type-check, and pass every unit while rendering nothing — which is why Validate mandates a DRIVEN pixel check for any render change. If you cannot drive it (chad at the machine), the render is still owed and must be named as deferred, not assumed working.

WHY IT EVADES GATES: rustc does not warn on a private field that is initialized in a struct literal and only ever assigned (the init marks it live for the dead-code pass). Combine that with coverage-excluded app.rs and mutants::skip shims and a write-only field is invisible to clippy + coverage + mutation at once. The manifest diff is the ONLY thing that catches a dropped render, because it is the only check that knows what you INTENDED to build.

This is the "invariant nothing enforces" class pointed at your own process: the manifest is a contract you wrote and then did not keep.

## PR-claude-disjoint-bit-pack-is-equivalent-mutant-001
*severity: medium*

Packing disjoint bit-fields with OR — `(r<<16) | (g<<8) | b` where the operands occupy non-overlapping bits — makes `|→^` and `|→+` EQUIVALENT mutants: for disjoint operands, `a|b == a^b == a+b` for all inputs, so cargo-mutants generates them but no test can kill them → MSI can never reach 100%. Don't hand cargo-mutants an unkillable mutant. Fix: avoid the bit-pack when a structured constructor exists — build the target struct field-by-field (`gpui::Rgba{ r: r as f32/255.0, ... }`) instead of packing an int and re-parsing it. Same family as the #23 remainder-arm and #26 inert-framework-call: restructure so every generated mutant is observable, rather than justifying/suppressing.

## PR-claude-display-cap-must-equal-navigation-cap-001
*severity: high · prevents: BF-symbol-picker-render-cap-diverges-from-nav-001*

A picker/list overlay that CAPS how many rows it DISPLAYS must apply the SAME cap to navigation (move_up/down), the selected index, and accept (Enter). If the cap lives only in the render, the internal selection can walk past the visible rows — ↓ moves an invisible cursor and Enter jumps to a row the user never saw or highlighted. Lift the cap to a SHARED const used by the render AND the key handler; clamp the accept read through it too. Especially bites when the source list is unbounded (a server that returns the whole workspace).

## PR-claude-display-sanitizers-strip-bidi-cf-not-just-cc-001
*severity: medium · prevents: BF-claude-clamp-card-text-passed-bidi-format-chars-001*

A sanitizer that clamps text for DISPLAY must strip bidi control characters, not just is_control(): Rust's char::is_control() is general-category Cc only, while the Trojan-source primitives — U+202A–U+202E (LRE/RLE/PDF/LRO/RLO) and U+2066–U+2069 (LRI/RLI/FSI/PDI) — are Cf (format) and pass straight through to a bidi-honoring text engine (Core Text, browsers), letting untrusted input visually reorder the rendered string. When auditing a render path for hostile text, walk the actual Unicode categories the filter admits; when one shared clamp serves several consumers, fix the helper so every consumer hardens at once.

## PR-claude-display-sleep-blocks-capture-001
*severity: medium*

In a LONG driven-validation session the physical display can go to sleep, after which `screencapture` (both `-l<windowid>` and full-screen) returns a pure-black PNG (not an error — a valid black image; avg brightness ≈0). Synthetic CGEvents (drive.swift clicks/keys, a mouse-move, a shift-key, a scroll) do NOT wake a slept display, and `caffeinate -u -t N` did not wake it either (likely the session is screen-locked). Symptom: captures that worked earlier in the session suddenly go black, often with the window also shrinking to a default size. This is an ENVIRONMENT block, not a code bug. Confirm it's display-sleep (not a broken app) by sampling full-screen brightness — if the WHOLE screen is black, it's the display. When blocked: (1) note it explicitly per §7 (don't silently skip); (2) lean on the headless proofs that DON'T need the display — gate:15 visual/AX uses marley_visual_harness offscreen rendering, plus the inspect critic's static trace + unit tests; (3) flag a one-glance visual confirm for when the display is awake. Prefer front-loading UI captures early in a session before the display can sleep.

## PR-claude-dissolving-a-guard-must-sweep-the-reachability-comments-001
*severity: low*

When a ticket makes a previously-unreachable state REACHABLE (dissolving a guard, relaxing an invariant) — or removes a code path — grep the whole crate for the comments/docstrings that asserted the OLD unreachability or behavior and sweep them in the SAME change. Earlier tickets routinely leave gravestones like "test-only reachable until #N" / "a refused close shows a status flash" pinned to the now-changed reality; leaving them asserts a false unreachability (or a deleted behavior) that misleads the next reader, and it is inconsistent to refresh one sibling while leaving nine stale. Two greppable anchors catch most: the future-reference to the current ticket number ("until #<this-ticket>") and the docstring of every fn whose body the diff touched. A docstring on the CHANGED fn that describes the deleted path is a MED (it tells the opposite of the truth); the scattered reachability gravestones are LOW but cheap to sweep at the natural point — this ticket. (Marley #395: dissolving the never-empties guards left 13 "test-only until #395/#392" comments + one close_tab_at docstring describing the removed status flash.)

## PR-claude-dissolving-a-runtime-invariant-must-audit-the-boot-restore-path-001
*severity: medium*

When you dissolve or relax a RUNTIME invariant (a guard that "X always holds"), the same invariant is often ALSO enforced independently in the BOOT / restore / deserialization path — and relaxing only the runtime side leaves the boot side silently re-imposing the old constraint. The runtime tests (drive the app into the newly-legal state) will all pass while a persist→relaunch round-trip quietly repairs the state back to the old invariant. Concretely: search the restore/new_in/from_settings path for the same predicate or a "must keep ≥1 …" / force-seed / default-fill comment, and add a BOOT-level test (seed the persisted form of the newly-legal state, boot, assert it survives) — not just a runtime-drive test. (Marley #395: dissolving the never-empties guards left app.rs's boot restore force-seeding a terminal onto any terminal-less project — REQ-003 "restore empty" was unmet until a headless boot-restore test caught it; the fix needed a zero-item constructor the restore loop could route the empty case through.)

## PR-claude-doc-only-fix-has-no-verifier-reread-it-001
*severity: high · prevents: BF-claude-inspect-fix-recorded-but-never-applied-001*

A DOC-ONLY fix has NO verifier — no gate can catch a false sentence, so re-read the file, never re-run the tools.

When an inspect/review finding's resolution is "reword the doc/comment", the entire toolchain goes blind: cargo check, clippy -D warnings, the test suite, fmt, coverage 100%, mutation MSI 100 and even rustdoc -D warnings all pass a doc comment that states the exact opposite of what the code does. A false doc is well-formed Rust. This is unlike a code fix, where the compiler or a test will usually object if the edit never landed.

THE RULE, in three parts:
1. Before writing "fixed" in a ledger for a doc-only finding, `grep` the defective sentence and prove it returns ZERO hits. Do not write the resolution from intent. (M22 #337 F5: the ledger said "the settings doc now describes what the code does"; the defective sentence was still there verbatim, and shipped through Implement, Inspect's own post-fix re-verify, and a full GATE GREEN.)
2. A LEDGER IS A DOC. The rule "a doc does not get to make a claim the code doesn't keep" applies to the ledger row that enforces it. When one recorded fix turns out unapplied, AUDIT THE REST of that ledger's findings against the code — do not assume the miss was isolated (in #337 it was, and proving that took one grep per finding).
3. Check the finding's REASON, not just its fix. #337's F5 justified the cut with "a resolution probe this path lacks" when gpui exposes `TextSystem::font_id -> Result`; #337's F3 shipped a correct fix whose comment had the physics backwards; #336's F5 cited `element_offset` when taffy was the carrier. A PLAUSIBLE reason terminates review before anyone measures it — so measure the reason, especially when the fix looks right.

Smell: any finding whose fix touches only `///`, `//`, or a .md file. That is precisely where to slow down, because nothing downstream will slow down for you.

## PR-claude-domain-sweep-includes-equality-gates-001
*severity: high · prevents: BF-claude-geom-recorder-buffer-row-eq-slot-gate-001*

When sweeping for value-domain mixing (buffer rows vs slots, chars vs cells, px vs cols), grep EQUALITY and COMPARISON sites, not just offset arithmetic — `x == first` / `x < last` gates hide domain bugs that `(x - first) * k` greps never surface, especially where a domain migration renamed or shadowed one side (the #305→#352 `row == first` recorder gate froze editor_geom under folds and survived two arithmetic-focused sweeps). Sweep pattern: for each domain boundary variable, grep every operator use (==, !=, <, <=, >, >=, min, max, ..) and classify each side's domain.

## PR-claude-dont-flip-validate-pass-before-gate-green-001
*severity: medium*

During /pipeline:validate, keep the spec frontmatter at "Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate" until scripts/gates.sh is GREEN. Only flip to "Phase 4 — Validate PASS" AFTER green. If you flip early and the gate comes back RED, the enforce-phase-gate.sh hook reads the "Validate PASS" status and BLOCKS the very source-fix edits validate needs to make green — you then have to sed the status back to Inspect PASS before you can edit. The status flip is a completion marker, not an intention.

## PR-claude-dont-ship-a-pure-seam-method-unused-until-a-later-ticket-001
*severity: medium*

Do not add a pub method/fn to a pure seam that no non-test code calls yet, planning to use it in a later ticket. In a crate compiled as a binary (or a private `mod`), an unused pub item is dead code → `cargo clippy -- -D warnings` (gate:2) fails, and `#[allow(dead_code)]` is a forbidden suppression (§0). Ship each seam method WITH its first real consumer. #250 added `LineLayout::offset_of_col`/`width` "for #254" but nothing in #250 called them → removed; #254 adds `offset_of_col` when it wires the mouse click. (Tests calling the fn do NOT satisfy the non-test dead-code lint.)

## PR-claude-drag-flag-needs-buttonless-move-selfheal-001
*severity: high*

A boolean "drag in flight" flag (dragging_selection / dragging_divider / dragging_*) must NOT be cleared solely by an `on_mouse_up` on the dragged element: gpui's on_mouse_up is bounds-gated (fires only when the button is released over that element's hitbox), so a release OFF the element (empty space, padding, a sibling pane, outside the window) leaks the flag `true` — and any handler gated on that flag then mis-fires on a plain button-less hover. Guard it BOTH ways: (a) self-heal in the on_mouse_move handler — `if event.pressed_button != Some(MouseButton::Left) { flag = false; return; }` so the first button-less move clears the stale flag before doing drag work; and/or (b) clear via a non-bounds-gated path (a window-level mouse-up, or `on_mouse_up_out`). Prefer (a) — it needs no extra element and self-corrects on the very next move.

## PR-claude-drive-tiling-split-via-context-menu-001
*severity: low*

To drive a multi-pane TILING split in the live app (for validating pane layout/tiling), open the pane's right-click context menu and pick "Split Right"/"Split Down": `drive.swift ... rightclickat:<fx>,<fy> wait:500 enter` — the menu opens with "Split Right" pre-selected (index 0) for a pane body, so right-click→enter tiles horizontally; repeat to add panes. Do NOT use ⌘D — it maps to `new_terminal_pane()` (a NEW TAB), not `split_focused_pane()`, so you'll get terminal 1/2/3/4 tabs each with a single pane, not a tiled multi-pane tab. The tiling split (`MenuAction::SplitRight` → `split_focused_pane(PaneAxis::Horizontal)`) is only reachable through the context menu.

## PR-claude-early-return-arms-before-a-guard-change-behavior-001
*severity: medium*

When refactoring a dispatch fn to add early-return match arms, check whether the new arms sit BEFORE or AFTER an existing guard (e.g. a `if control||platform return Other` gate) — an arm placed before the guard silently bypasses it, changing behavior for the guarded inputs (here: cmd/ctrl+arrow triggered prompt motion instead of being ignored). This is invisible when the changed path lives in a `mutants::skip`'d shim (no test guards it). Preserve the original ordering relationship: keep truly modifier-agnostic arms (backspace/enter) before the guard, put everything the guard should still catch AFTER it. State the intended modifier behavior in the fn's doc so code and design can't drift.

## PR-claude-ears-behavior-covered-mutant-still-alive-on-sibling-line-001
*severity: medium*

Mutation-readiness ≠ EARS coverage. An EARS REQ row proves a BEHAVIOR, but a cargo-mutants mutant lives on a SPECIFIC code line — the same logical guard on a SIBLING arm (e.g. the "clear question if state != Waiting" guard appears on both the Upsert arm AND the StateChange arm; a test that exercises only StateChange leaves the Upsert line's `!=`→`==` mutant alive), or the same predicate reached through an ALTERNATE public entry (e.g. get() vs seats() both do `id ==` — a test asserting order via seats() never kills get()'s mutant or covers its line). Also assert the field the mutant ACTUALLY changes: a state-machine observe() whose `>`→`>=` mutant only flips the returned `.advanced` flag (not the resulting `.state`) survives any test that asserts only `.state`. Procedure: run `cargo mutants --list -f <file>` for the REAL viable set, then map EACH viable mutant to a test that (a) executes its exact line and (b) asserts the specific field/branch it perturbs — do not assume the EARS-named behavior test transitively kills a sibling-line or alternate-entry mutant. Confirmed on #367 marley_fleet: 5 of 33 viable mutants (B1–B5) would have survived the tests the REQ rows literally described.

## PR-claude-edge-triggered-source-latch-the-fact-not-re-poll-001
*severity: medium*

When you poll an EDGE-triggered event source (one that yields a fact exactly once and then reports "nothing" — a self-pipe drained by one read, a `signalfd`/`epoll` edge, a `try_wait` gated behind a SIGCHLD pipe, an mpsc drain), and that same source is ALSO consumed on another code path, a second consumer CANNOT re-observe the already-consumed event — it will see "nothing" and mis-conclude the event never happened. Do NOT build a second path that re-polls the source expecting to see the same edge; LATCH the fact in owned state the first time any path observes it (a `bool`/`Option<Status>` on the struct that owns the source), and have every other path read the latch. In TICKET-348 the PTY teardown loop re-polled `next_child_event` (edge-triggered on a SIGCHLD self-pipe) for a child the pump had already reaped-and-consumed, so it never saw the exit and ran a full 4s + leaked; the fix was an `child_reaped` latch set in the one poll and read in Drop. Ask: "is this exit/event signal consumed by exactly one read, and is anyone else reading the same source?" If yes, latch it.

## PR-claude-empty-iterator-closure-uncovered-001
*severity: low*

A unit test whose assertion runs a closure via `.any()`/`.all()`/`.filter(pred)` over a slice or iterator that is EMPTY in the tested scenario leaves the closure body UNCOVERED (0 executions) → gate:4 coverage RED, even though the test passes. Classic trap: slicing a "between two markers" range (`rows[a+1..b]`) that collapses to empty in the very case you're testing (e.g. a collapsed project sits immediately before its sibling). Fix: assert over a NON-EMPTY projection — filter the FULL collection by a discriminant that yields the target rows and run the closure over those — so the predicate executes. Verify the specific uncovered line via the llvm-cov JSON (`cargo llvm-cov report --json` → segments with count 0), don't guess.

## PR-claude-enforce-security-invariants-in-code-not-just-docs-001
*severity: high*

A security invariant named in a decision ("localhost only", "read-only", "never logged") must be ENFORCED in code, not merely documented — a doc line is not a guard. In #63 the forge-client's D1 said "localhost 127.0.0.1:8080 only", but the code connected to whatever authority `.mcp.json` contained, so a misconfigured/tampered config could send chad's forge BEARER over cleartext HTTP to an arbitrary host (a security critic caught it). FIX pattern: guard the secret-bearing destination at CONSTRUCTION — `forge_endpoint_from` now rejects a non-loopback url via `is_loopback_authority` (parse host from the authority, accept `localhost` or `IpAddr::is_loopback()`), returning None so the endpoint never constructs and no request is ever built. GENERAL: when a secret is sent somewhere (a token in a header, a credential to a host), the ALLOWED destination must be checked in code before the secret is used; and delete any `pub fn secret_getter()` that has no caller (dead public API that hands out the token — a future careless `log!("{}", ep.bearer())` waiting to happen). Also: redact secrets in Debug via a MANUAL impl (a derived Debug leaks) and PROVE it with a probe test asserting the real secret is absent + a `***` sentinel present. The two independent critics (correctness + security) on the security-sensitive ticket were worth it — the security lens found what the correctness lens didn't look for.

## PR-claude-event-driven-deadline-poll-not-fixed-budget-001
*severity: high*

A headless/integration test that waits on a REAL subprocess or background worker (a spawned shell reporting `pwd`, an off-thread parse landing) must poll EVENT-DRIVEN to a generous FINITE wall-clock deadline — NOT a fixed iteration count. A fixed budget (e.g. `while polls < 200 { sleep(25ms); ... }` = ~5s) passes on an idle box but FAILS under load (a busy box, or `cargo mutants` running the suite as parallel threads with no per-test isolation), and a failed mutation-gate BASELINE is exit 4 = "not a valid measurement" → the commit gate fails CLOSED non-deterministically. Use a shared `poll_until(deadline)` helper: loop the real-sleep + mock-clock-tick body while the probe is unmet AND `std::time::Instant::now() < deadline` (a real OS clock, independent of any test-framework mock clock like gpui's `advance_clock`). Prove the ceiling's FINITENESS by the helper RETURNING not-met on an always-unmet probe + a tiny ceiling — NEVER by a wall-clock UPPER-bound assertion, which re-introduces the exact load-flake being removed. Keep the helper single-path (no early return / no timeout tail) so every line runs on both the met and timed-out paths → coverage stays 100 in a `#[cfg(test)]` module counted by execution. Corollary: before "reusing the existing env-knob idiom the other timing drives use", VERIFY it exists (grep) — a ticket's confident suggestion can rest on a false premise.

## PR-claude-every-caret-path-must-reach-the-reveal-hook-001
*severity: high*

When a display feature (fold auto-reveal, scroll-into-view, a caret-follow side effect) is hooked at a "shared caret-placement primitive," do NOT trust the design's naming of that primitive — ENUMERATE every caret-placement PATH (go-to-line, symbol-jump, go-to-def, find-next/prev, nav-back, diagnostics-jump, add-cursor) and grep the hook's single caller-set to prove each path actually reaches it. Motion (arrow/word/page) and jumps frequently take DIFFERENT sinks: motions call the caret-follow primitive; jumps place the caret then call the raw scroll (scroll_to_row / park-and-center) directly, bypassing the hook. The raw scroll usually CANNOT blanket-carry the side effect because a preview/peek path scrolls with the caret parked elsewhere. Fix with a dedicated jump helper (reveal+scroll) at each bypass site, excluding the preview path — and pick the test verb DELIBERATELY: a REQ that says "navigate into a hidden row reveals it" ships GREEN past the bug if the test reaches for a motion (which reveals) instead of a named jump (which bypasses). This is the #336 lesson generalized: hook the LOWEST shared primitive, and when there is no single one, cover EVERY branch.

## PR-claude-every-caret-placement-path-calls-clear-marked-before-placement-001
*severity: low · prevents: BF-claude-goto-restore-missing-clear-marked-001*

Every out-of-band caret-placement path (go-to-line commit AND restore, click-to-place, jump-back, find-next landing, go-to-definition landing) MUST call `clear_marked()` immediately BEFORE the placement, symmetric with every sibling placement path — a stale IME composition span left un-cleared misdirects the next `ime::replace_text`. `set_single_caret`/`set_active_selections` do NOT bundle `clear_marked`, so it is a separate explicit call. When adding a new placement site, grep the existing ones and match: a site missing `clear_marked` where its siblings have it is the defect (the row-vs-offset invisible-defect class — the text lands right, the next composition doesn't).

## PR-claude-exhaustive-match-over-defensive-catch-all-for-cov-msi-001
*severity: medium*

Under a 100% coverage/MSI floor, a defensive UNREACHABLE `_ =>` catch-all arm is UNCOVERABLE (and its delete-arm mutant survives). When dispatching on a CLOSED enum, `match` the enum EXHAUSTIVELY (no `_`): every arm is reachable+covered, arm-deletes stop compiling (unviable mutants), and adding a variant becomes a compile error — additivity is compiler-enforced. Likewise, an infallible `to_value(x).unwrap_or_else(|_| …)` fallback CLOSURE is an uncovered function → use `.unwrap_or_default()` (no closure, the never-taken branch is a region not a line).

## PR-claude-external-uri-key-normalize-both-sides-001
*severity: high*

When a store/cache is WRITTEN with a key derived from an external process's identifier (an LSP `publishDiagnostics` uri, a remote path, any wire string) but READ with a key your own code derives from a local value, the two keys must be produced by the SAME normalization — or the store silently misses whenever the peer re-encodes the identifier. FIX: on write, DECODE the wire identifier back to the canonical local form (uri → `path_from_file_uri` → `absolute()`/`canonicalize()`), then key by that — never store the raw wire string. Symptom: works on a plain-ASCII path (peer happens to echo byte-identical) but drops everything for a path with `@`/space/sub-delims (percent-encoding differs) or a symlink (canonicalization differs), with NO error. Mirror the existing canonical-PathBuf keying (Marley's #309 `docs` map) rather than inventing a parallel string key. Add a round-trip test pinning write-key == read-key for a non-ASCII identifier.

## PR-claude-extract-security-critical-predicate-to-shared-leaf-001
*severity: low*

A SECURITY-CRITICAL predicate duplicated verbatim across sibling crates (e.g. a loopback/allowlist guard in both an MCP client and server) should be extracted to a shared leaf crate both already depend on, so it has ONE audit point + ONE test suite — even though "two tiny crates" argues for duplication. Weigh security-criticality over the small-crate convenience; if the extraction reopens already-shipped code, file it as a focused refactor rather than leaving two copies to drift.

## PR-claude-fixture-to-instance-audit-every-activator-001
*severity: high · prevents: BF-claude-fixture-to-real-tab-orphaned-its-callers-001*

When a permanently-present surface becomes a born-on-open instance (a fixture becomes a real tab/pane/row), grep for EVERY site that activates it and prove each one also OPENS it. Activation and existence were the same thing before the change and are now two things; every caller written under the old invariant is now a latent orphan-state bug — it will select something that does not exist. Type checking cannot catch this (the key is still a valid variant) and a happy-path drive of the new verb cannot either (it only exercises the site you just wrote). Also check the inverse: close/fallback paths that defaulted TO the fixture now default to something closable, and any strip or list that hardcoded it will render it twice once it joins the collection.

## PR-claude-focus-affordance-draws-after-overlapping-sibling-chrome-001
*severity: medium*

A per-item focus/selection affordance (a border, highlight, ring) that shares an edge or area with a LATER-drawn sibling element (a drag-divider, gutter, adjacent panel) will be OCCLUDED by paint order — gpui and most retained renderers paint later children on top. Draw the affordance AFTER the overlapping sibling so it is the topmost item-chrome, but still BEFORE the modal/overlay layer so modals still occlude it. Implementation: hoist the per-item draw out of its loop into an `Option<T>` (single-select) or `Vec<T>` and drain it at the correct layer, keeping the moved draw byte-identical. VERIFY by enumerating that every modal/overlay still renders after the drain point (so the affordance did not jump above the modals). Keep the affordance non-`.occlude()` so it does not steal hit-testing from the sibling underneath (e.g. the divider's drag). (Caught at #228: the #191 focus border was drawn in the pane loop, before the #130 divider loop, so the `.occlude()` divider covered the border's shared-edge bar; moving the border draw after the divider loop fixed it.)

## PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays-001
*severity: medium*

When you add a full-screen (`.inset_0()`) click-away dismiss backdrop behind a modal A, that backdrop HIDES (not merely occludes at its own bounds) every other overlay that renders BEFORE A in the child order. So A's open path must close EVERY overlay that can co-exist with A and renders earlier — otherwise a co-open overlay becomes fully invisible while it may still own the keyboard (an invisible-focus state). Audit the open site: enumerate the overlays that render before A and confirm each is cleared (or is provably mutually exclusive with A). This is stricter than the pre-existing "close other overlays" habit, which only mattered for partial occlusion / stacking; a full-screen backdrop makes total-hide the failure mode. (Caught at #229: the launcher's new backdrop fully hid a co-open #204 naming draft that the open guard didn't close.)

## PR-claude-fuzz-the-equivalence-claim-001
*severity: high*

When shipping an optimization whose contract is EQUIVALENCE to a slow reference (incremental == full re-highlight, a cache == recompute, a fast path == a slow path), a hand-picked corpus + an inline adversarial trace are NOT sufficient — they miss whole input classes. Write a DETERMINISTIC DIFFERENTIAL FUZZER (a seeded LCG driving random operation chains, asserting fast==slow every step, panicking with the exact repro) as a PERMANENT in-test regression, and run a beefed-up version (100k+ steps, adversarial alphabet) before shipping. Proof: #285's tree-sitter windowing shipped GATE GREEN (cov/MSI 100) with a CRITICAL bug (highlighting vanished on backspace-at-comment-end); the inline trace + first critic judged it correct; only a differential fuzzer found it — and then found the FIRST fix still incomplete TWICE. When the reference itself is non-deterministic across the two paths (e.g. tree-sitter's incremental parse can diverge from a fresh parse on error inputs), GUARD the equivalence assertion on the shared precondition (same tree) so the fuzzer tests YOUR code, not the library's non-determinism — plus a sanity assert that the guard passes for the vast majority of steps.

## PR-claude-gap-fix-test-must-reproduce-the-original-failure-001
*severity: medium*

A regression test for a "gap fix" (code that makes a previously-silent failure observable) must REPRODUCE the original failure condition, not a convenient near-substitute. #372's get-rejection test closed the socket after a non-2xx GET header — but a closed socket makes even a GAP-LESS pump surface StreamClosed (from the Ok(0)), so the test guarded only the classification, not the no-idle property it claimed. The original failure was "the server HOLDS a rejected stream open and the pump idles forever" — so the fixture must HOLD the stream open; then a regression that removed the gate idles to the deadline and the test fails. Litmus: mentally delete the fix and confirm the test goes red. If deleting the fix leaves the test green (because some other path incidentally produces the asserted value), the test proves nothing about the fix.

## PR-claude-gate-change-updates-quality-bar-row-001
*severity: medium*

WHEN a change adds, removes, or alters any check inside scripts/gates.sh (or a gate's label/scope), THE SAME change must update the corresponding gate row in docs/specs/standards/quality-bar.spec.md — gates.sh names that spec the single source of truth, so a behavior/spec divergence is a defect even when the gate itself is green. Verify with: grep the gate number's row and confirm it names every check the function now performs.

## PR-claude-gate-defining-files-in-receipt-fingerprint-001
*severity: medium · prevents: BF-clippy-toml-not-in-receipt-fingerprint-001*

Whenever a ticket adds a NEW gate-defining config file (anything that changes what a quality gate accepts — clippy.toml, rustfmt.toml, rust-toolchain.toml, a nextest/llvm-cov/cargo-mutants config, .config/*), fold it into the commit-receipt fingerprint (gate_state_hash in lib-hook-helpers.sh) in the SAME ticket. Otherwise the file can be weakened around a green receipt (swap-out → run gate green → swap-in → commit) without invalidating the receipt, certifying code under a ruleset no honest run would pass. The receipt must bind every file that defines the bar, not just the code. Verify by editing the file and confirming gate_state_hash changes. (gate_state_hash is itself fingerprinted, so the edit self-certifies.)

## PR-claude-gate-hang-diagnose-via-per-proc-cpu-253
*severity: medium*

When the gate runs unexpectedly long (>10 min for a `--diff` run), do NOT keep waiting blindly — run `ps -o pid,pcpu,etime,command` on the cargo/nextest/test-binary processes. A test binary alive for many minutes at 0.0% CPU is a DEADLOCK, not slow computation (especially real-PTY/session integration tests under `cargo llvm-cov` instrumentation, where the extra slowdown races the shell/session handshake). Kill the gate tree + the hung test and re-run on a quiet system; a real-process test can flakily hang under coverage while passing in seconds on a retry. Distinguish this from memory thrash by checking `memory_pressure` (a healthy free % rules out the --jobs fan-out).

## PR-claude-gate-verdict-owns-the-exit-code-never-pipe-001
*severity: medium*

Never pipe a quality gate (or any pass/fail command) through tail/head/grep when its exit code is the verdict — the pipeline returns the LAST command's status and a RED gate exits 0 (bit at #352: `gates.sh --diff | tail -40` reported success on a gate:12 failure; only the printed GATE RED text told the truth). Run the gate bare (background output goes to a file anyway), read the file for the summary, and treat the printed verdict line as authoritative over any pipe status.

## PR-claude-git-add-intent-stages-empty-blob-re-add-at-commit-001
*severity: medium*

`git add -N <newfile>` (intent-to-add) stages the file as git's EMPTY blob (e69de29bb2d1…) with its CONTENT left UNSTAGED — it exists in the index at zero length. This is needed so cargo-mutants sees a NEW pure file at the `--diff` gate (untracked files are invisible to `--in-diff`, so their mutants are silently skipped — the M21 lesson). But it creates a commit-time trap: the gates read the WORKTREE (so they pass green on the real content), while a commit captures the INDEX — a BARE `git commit` would land the file EMPTY, breaking the build (a `pub use newmod::…` fails to compile) in a way the gates never saw. At /commit, ALWAYS explicitly `git add <the new file>` (not just `git add -N`) so its content is staged, and verify with `git show :<path> | wc -l` (or `git diff --cached --stat`) that the staged file is non-empty before committing. A bare `git commit` after only `git add -N` is the failure mode; an explicit `git add <path>` (or `git add -A` / `git commit -a`) neutralizes it.

## PR-claude-gpui-keyboard-focus-needs-a-mouse-down-not-a-raise-001
*severity: high*

In the self-test harness, a `type:`/keystroke lands in a pane ONLY after a `clickat:<pane>` — gpui sets window keyboard focus on a pane's mouse-DOWN (a `track_focus` element auto-focuses on MouseDown), and a programmatic app raise/activation (`NSRunningApplication.activate`, `osascript … set frontmost`) posts NO mouse-down, so it sets NO keyboard focus. Never rely on a raise/activation alone to make keys land — click the target pane first, with coordinates from a FRESH capture. The trap's tell: the window is frontmost and the pane even shows the accent focus BORDER (that is the workspace-internal focused-pane model, NOT gpui `window.focus`), yet typed keys vanish. Empirically proven 2026-07-22 (#383): `focus type:echo` → nothing; `clickat:<pane> type:echo` → runs. Corollary: when a harness verb that used to CLICK is changed to a non-clicking raise, re-drive every typing flow that depended on it — the click may have been silently establishing focus.

## PR-claude-gpui-no-box-sizing-border-inflates-001
*severity: medium*

gpui/taffy has NO box-sizing (there is no `box_sizing` in the gpui source; `Style.border_widths` maps to taffy's border, which INFLATES an auto-sized element's outer box = content + padding + border). So adding `.border_1()` to an auto-sized element GROWS it by 2px per axis. For a fixed-cell element — a terminal cursor cell, a grid cell, an icon that must align to a mono column — a bare border on ONE state (e.g. an unfocused/hover variant) widens that cell and misaligns everything after it, and toggling the state (focus/hover) makes the cell JITTER. Rules: (a) prefer a FILL (`bg`) or a text-color change for state variants of a fixed-cell element — a fill never changes the box size; (b) if you genuinely need a border look, put the SAME `.border_1()` on ALL states (with a matching-color/transparent border where you don't want it visible) so every state reserves the identical 1px and the box is stable; (c) `bg`/`text_color` are paint-only (they don't touch taffy layout) — safe for size-invariant state changes. This bit #220 (the focused-vs-unfocused block cursor: a hollow outline would have widened the one-cell caret, breaking the #218 one-cell + #88 gapless-flush invariant → used a dim fill instead) and will bite the M13 editor caret (#208) the same way.

## PR-claude-grep-all-ctor-sites-on-label-change-001
*severity: medium*

When changing how a type is labeled or constructed (a tab title, a display name, a default), grep for EVERY caller of that constructor and update ALL of them — not just the path in view. A second construction site — commonly the deserialize / session-restore path — silently defeats the change, and it is NOT exercised by unit tests or a fresh-session driven capture (only by quit-and-reopen). Before declaring the change complete, enumerate the constructor's full call-site set (e.g. `grep 'Type::ctor('`) and confirm each agrees. Grepping only the READERS of the field is insufficient — you must find the other WRITERS.

## PR-claude-grep-all-struct-literals-when-adding-required-field-001
*severity: high · prevents: BF-agentrow-field-missed-test-literals-001*

When adding a REQUIRED field to a struct that is built with struct literals (not only via a constructor), grep every `StructName {` construction site across the crate — production AND all test modules — and update each; do not assume the central/obvious test is the only literal. Then run `cargo check --tests -p <crate>` (NOT just `cargo check`) before declaring implement done: a plain `cargo check` compiles the non-test build clean while the `#[cfg(test)]` literals still fail E0063, hiding the break until the mutation/validate gate.

## PR-claude-grep-existing-name-before-adding-a-ui-surface-001
*severity: low*

Before adding a NEW named UI surface (a struct field, a dispatch verb, an icon, a panel title, a command), grep the codebase for that NAME first — a legacy surface can already own it. On #369 the design + Explore map scoped to the intended seam (reviving the dormant right DOCK slot) and correctly found it independent of the RightSection cockpit tabs, but MISSED a pre-existing `fleet_open` ⌘⇧E overlay (#68) that lists LOCAL launched agents — a DIFFERENT object also called "Fleet". No code clash resulted (distinct field/verb names were chosen at implement time), but the app now has TWO "Fleet" surfaces, a UX collision worth flagging. Lesson: an Explore/design pass scoped to the NEW feature's intended location will not surface a same-named surface living elsewhere; run a name-collision grep (`rg -n '<name>'`) across the whole crate at design time so the collision is a deliberate decision (rename, unify, or accept) rather than a discovered-at-implement surprise. This also feeds a possible follow-up: unify the legacy local-agents overlay into the control-plane fleet envelope.

## PR-claude-grep-for-absence-must-prove-the-command-ran-001
*severity: medium*

A grep that PROVES A NEGATIVE ("0 hits = the shims are correctly skipped", "empty = no brand leaks") is indistinguishable from a command that never produced output at all. Any failure upstream in the pipe — a missing binary, a bad flag, a non-zero exit swallowed by the pipe — yields the same empty result and reads as a pass. Caught live on #331: a critic ran `timeout 300 cargo mutants --list | grep -i inlay`, got nothing, and nearly recorded "correctly skipped" — but `timeout` DOES NOT EXIST on macOS (it is GNU coreutils; the BSD userland has no such binary), so the command never ran. Always pair an absence check with independent proof of execution: pipe to `wc -l` and assert the total is non-zero, check `${PIPESTATUS[0]}`, or print a sentinel from the producing command. On macOS specifically, do not reach for `timeout` (use the tool's own timeout parameter), and prefer the Bash tool's `timeout` argument over shell-level wrappers.

## PR-claude-guard-last-one-out-shareable-resource-001
*severity: high*

Before removing or tearing down a resource when ONE owner closes, check whether that resource can be SHARED by other live owners (a reachability question), and if so guard the teardown LAST-ONE-OUT (`if !any_surviving_owner_uses(resource) { drop it }`). A "trivial one-liner" cleanup (e.g. `map.remove(&closed.key)`) can hide this: an unconditional remove kills a resource a surviving co-owner still needs. Confirm the model actually forbids sharing (dedup on open) before removing unconditionally; if it does not (Marley #359: opening the same directory twice yields two projects sharing one LspHost), the correct fix is a guarded last-one-out remove. Bonus structural guarantee: derive the teardown guard's key and the container's key from the SAME field under the SAME equality relation (both raw `Path` Eq), so a stale/wrong drop is impossible by construction rather than by coincidence.

## PR-claude-guard-on-an-unfloored-two-sided-quantity-001
*severity: medium*

When a pure fn's None/early-return guard compares a quantity against 0 (e.g. `if x <= 0.0 { return None }`), check whether that quantity is FLOORED at 0 upstream (a `.max(0.0)`, or a fn like `max_scroll_x` that early-returns 0 and never goes negative). If it is, `x <= 0` and `x == 0` agree on the ENTIRE reachable domain, so any `==`/`!=` mutant of the comparison is EQUIVALENT and unkillable → MSI < 100 with no test that can fix it. Guard instead on an UNFLOORED, two-sided quantity that reaches BELOW 0 on a reachable (or totality) input — e.g. `view = sane(viewport)` (a degenerate negative viewport) and `over = content - viewport` (content narrower than the viewport) — so a narrower/exact-fit/negative test distinguishes every boundary operator. The behavior is identical on real inputs; only the mutation-killability changes. Trace the actual mutant set with `cargo mutants --list` before assuming which operators are emitted (this cargo-mutants emitted only `<=`→`>`, not the `==` twin — but the unfloored form is the robust default regardless).

## PR-claude-guard-shadowed-comparison-is-an-equivalent-mutant-001
*severity: medium*

A comparison that executes only under a guard excluding its boundary case is an EQUIVALENT-mutant factory: `if a != caret { ... a < caret ... }` makes `<` vs `<=` indistinguishable (they differ only at a == caret, unreachable) — cargo-mutants counts the swap MISSED and MSI can never reach 100. Replace the operator with method calls, which cargo-mutants does not mutate: `a.min(b)..a.max(b)` for normalization, `reversed = a == e` for direction (== mutants stay killable by both-orientation tests). Trace with `cargo mutants --list` after the refactor to confirm the mutant class disappeared.

## PR-claude-handoff-tracker-sticky-across-none-frames-001
*severity: high · prevents: BF-claude-selection-home-tracker-severed-by-none-frame-001*

A focus/identity tracker that mediates a park/restore (or any ownership handoff between peers) must be STICKY across frames where the tracked kind is absent, and the handoff must fire on EVERY real transition — including indirect ones (A → unrelated-surface → B), not just direct A → B. Overwriting the tracker with None on an unrelated frame severs the chain: the outgoing peer never parks, the incoming peer inherits foreign state, and the stranded park corrupts a LATER direct transition. Design the state machine over (last-known-peer, current-peer) and make the restore total (birth-seed the parked slot) so no path depends on "was there a direct switch". (TICKET-397: the editor selection home; the same shape applies to any shared-handle owner tracking.)

## PR-claude-headless-gpui-test-uses-noop-text-system-not-coretext-001
*severity: high*

A `#[gpui::test]` headless drive runs on gpui's NoopTextSystem, NOT the real platform text system (CoreText on macOS): `all_font_names()` reports nothing resolvable, and glyph advances are synthetic (every char equal-width). So a headless drive CANNOT test anything that depends on a real font resolving, on real font metrics, or on distinguishing monospace from proportional — a family the user names will read UNRESOLVABLE and any advance-ratio probe will read every font as monospace. Before writing a headless drive that boots with a real font name or asserts a font-metric outcome, check whether the path is even reachable under Noop (often it short-circuits on `!resolves`). Such paths are HEADED-only (the real-CoreText / render_to_image lane) or must be proven by a PURE unit on the decision, with the gpui-metric READ left cov-excluded — mirror the sibling that already solved it (Marley #344's font_resolves). Lesson meta: when a prior ticket restricted its headless test to a degenerate case (e.g. a GARBAGE font), read WHY in its comment before assuming the non-degenerate case is testable the same way — the restriction usually encodes exactly this Noop limitation.

## PR-claude-headless-test-panic-before-reap-hangs-not-fails-001
*severity: high*

In a Marley gpui headless_drive test (VisualTestContext), an assertion that PANICS before `reap_sessions` runs HANGS the process on the PTY/session drop chain instead of failing — nextest shows it as SLOW forever, masking the real assertion message. So structure every headless test as: run all `window.update` steps and CAPTURE their results into locals, call `reap_sessions`, and ONLY THEN assert on the captured values. A hang in a headless LSP/editor test is almost always a pre-reap panic, not a deadlock in the code under test — make it hang-safe first to read the true failure. (Corollary seen here: a `consume`-style step that correctly DROPS a stale response does not clear a card built by a PRIOR step, so a stale-drop test must start from a clean state / run first, or it will see the leftover.)

## PR-claude-history-range-read-needs-over-screen-test-001
*severity: medium*

When code reads a terminal/grid HISTORY/scrollback region (alacritty negative Line indices, `-history..screen`), a test that feeds only ≤ screen_lines of output does NOT exercise the history path — the grid never scrolls, so `history == 0` and the read is byte-identical to a screen-only read. cargo-mutants proves it: the `delete -` mutant on `for line in -history..screen` SURVIVES a ≤screen suite (indistinguishable) and is killed ONLY by a test that feeds MORE THAN screen_lines rows and asserts the scrolled-off rows (esp. the FIRST line) survive. So MSI 100 on such code REQUIRES an over-screen integration test; a ≤screen suite at MSI 100 is false comfort. Build it via the session MockPtyChannel seam: `[Preexec] + N>screen distinct rows + [Precmd]` → assert the finished block keeps all N (first row present). Applies to any scrollback/history-range read (#52).

## PR-claude-hold-pty-across-read-split-opaque-exitcode-fmt-check-001
*severity: high · prevents: BF-claude-alacritty-pty-drop-discards-output-plus-exitcode-opacity-001*

When reading a PTY via alacritty_terminal (or any owning PTY handle), the handle MUST stay alive until the read loop hits EOF — `Pty::drop` closes the master fd, which DISCARDS the kernel's unread output queue, so a try_clone'd fd then reads immediate Ok(0) (empty). Return/bind the Pty across the whole read; drop it only after EOF. cargo check + clippy do NOT catch this (it's a runtime ordering bug) — a real spawn-read probe (or the integration test) does. Separately: a function returning an OPAQUE type with no PartialEq/accessor (std::process::ExitCode, etc.) cannot have its cargo-mutants `-> Default::default()` mutant killed by a unit test — split a pure, observable decision fn (e.g. `-> u8`) that's mutation-tested + make the opaque-returning fn a thin `mutants::skip` wrapper. And ALWAYS run `cargo fmt --check` in implement, not just clippy — clippy passes non-canonical formatting that gate:1 rejects.

## PR-claude-home-crate-vet-transitive-licenses-001
*severity: medium · prevents: BF-directories-pulls-mpl-option-ext-001*

For user home/config/data directory resolution under a permissive-only (no-MPL) license policy, use the `home` crate (MIT/Apache, cargo-team) — NOT `directories`/`dirs`/`dirs-sys`, which pull `option-ext` (MPL-2.0) transitively and trip cargo-deny. More generally: a spec's "reuse X (MIT/Apache)" claim covers only X's DIRECT license; always vet the TRANSITIVE tree against the allowlist (cargo-deny catches it, but choose the dep with a clean tree up front). When a spec's named dependency violates the license policy, switching deps is a correct, documented deviation.

## PR-claude-hook-the-shared-primitive-not-one-of-its-callers-001
*severity: high*

When adding a behavior that must ALWAYS accompany an existing one, hook the SHARED PRIMITIVE, not a convenience wrapper around it — and prove which is which by grepping for the primitive, not the wrapper. On #336 the horizontal caret-follow was hooked into `follow_editor_caret` (5 call sites) with a comment stating "one of the two silently not running is precisely the bug this ticket exists to fix" — while THREE gestures called the underlying `scroll_editor_to_row` directly and bypassed it entirely, including the two flagship cases (go-to-definition landing, find-next jump): the row scrolled in, the column did not, and the target stayed off-screen. The comment named the right invariant and enforced it in the wrong place. The test: `grep` the PRIMITIVE's name and count callers; if any caller is not the wrapper you hooked, you hooked too high. Hooking the primitive also makes every FUTURE caller inherit the pairing for free, which is the actual goal — an invariant that new code must remember to opt into is not an invariant. When the primitive takes less context than the behavior needs (here: the follow reads the primary caret, but ⌘D's vertical scroll targets an ADDED cursor), record the imperfection at the site and file the richer signature as a follow-up rather than leaving the gap unmentioned.

## PR-claude-hung-gpui-test-sample-the-pid-before-rerunning-001
*severity: medium*

A hung #[gpui::test] in the Marley headless lane is USUALLY a masked failure, not slowness: a panic unwind skips reap_sessions, so the live TerminalSession drops on the unwinding thread and BLOCKS forever (SLOW timers climb, CPU stays ~0). Diagnose, never re-run blind: `ps aux | grep marley_app-` (a test binary with ~0.02s CPU after minutes = wedged), then `sample <pid> 2` — the stack names the exact site (an assert's unwind-drop, or an ON-THREAD session drop like new_in's boot-PTY arm). Also: two cargo invocations sharing the target dir serialize on the flock — a "hang" right after launching a background cargo job is usually lock contention, and `| tail` buffers all output until EOF, so an empty output file proves nothing about progress.

## PR-claude-implement-cargo-check-all-targets-catches-fixture-breaks-001
*severity: medium*

When implement ADDS A FIELD to a struct (esp. a config/settings/state struct that has struct-literal constructions), a plain `cargo check -p <crate>` compiles the LIB but NOT the test targets — so `#[cfg(test)]` fixtures that construct the struct as a literal (`Foo { a, b }`) still miss the new field and fail with E0063 ONLY under `--all-targets`. The implement-phase compile check MUST be `cargo check -p <crate> --all-targets` (and `cargo clippy --all-targets`) so a broken test fixture is caught at implement, not at the Phase-4 gate. After adding a field: grep every `StructName {` (incl. `crates/*/src/**/tests` + `#[cfg(test)] mod`) and update each literal. This recurred in #87 (3 AppliedSettings test fixtures) — the lib compiled clean, the test build didn't.

## PR-claude-index-highlight-needs-owner-guard-001
*severity: medium*

When a render highlights a row by a bare INDEX compare (`active_row == Some(row)`) and the same index space repeats across multiple render passes (e.g. a per-pane loop where each pane restarts `row = 0`, or a list rendered in several columns), the index-only compare spuriously highlights the SAME-index row in EVERY pass — not just the one the index refers to. Gate the highlight on the OWNER (the focused pane / active column / an `is_focused` flag), OR re-derive it from the row's own content (like a sibling per-row find_bg that re-runs the match test on THIS pane's text). Symptom: a phantom highlight roams unrelated panes/columns in lockstep as the cursor moves. Applies to find cursors, active-match tints, selection echoes — any single-owner index painted across a multi-owner render.

## PR-claude-inert-framework-call-is-an-unkillable-mutant-drop-or-justify-001
*severity: medium*

Before calling a framework setup/registration API, confirm it has an OBSERVABLE effect on the paths you actually use — read the framework's accessor source. If get/set/read resolve WITHOUT the registration (e.g. from a compile-time key, not the registry), the registration call is inert for your usage → a mutant deleting it survives → MSI < 100. Fix at the SOURCE: drop the inert call (it is dead code for your access pattern; re-add it the day you call the API that consumes it — reload/subscribe/enumerate). Only `mutants::skip` it if it is a genuine IO seam with no testable effect. Do not leave an inert call in and hope a test kills it — no test can, because it changes nothing observable.

## PR-claude-input-handler-overlay-arms-must-stop-propagation-001
*severity: high*

When a gpui view registers an input handler (Window::handle_input / EntityInputHandler), EVERY key-ladder arm that CONSUMES a keystroke must call cx.stop_propagation() — a bare `return` leaves propagate true and dispatch_keystroke's fallback delivers the key's key_char (Enter="\n", Tab="\t", printables) to replace_text_in_range. A state-flag gate inside the handler cannot compensate: an overlay-closing Enter flips the flag BEFORE the fallback runs. Audit the whole ladder the day the first handler is registered, not per-overlay later; keys that must reach the IME (printables, dead keys) are the only ones left propagating, and only from the arm that owns text input.

## PR-claude-insert-at-prompt-goes-to-cooked-buffer-not-raw-pty-001
*severity: high*

To programmatically INSERT text at Marley's shell prompt (file-open, fuzzy-open, history recall, snippets), edit the LOCAL cooked prompt buffer — `state.buffer.edit(caret..caret, text, EditOrigin::Human)` then advance `state.caret` (mirror the cmd-V paste at app.rs ~872) — do NOT `session.write_bytes(text)` to the PTY. In COOKED mode (at the prompt, not alt-screen, no command running) Marley OWNS the line editor: it renders `state.buffer` and only sends the whole line to the PTY on Enter (write_command). A raw `write_bytes` reaches the shell's ZLE (proven: a `\r\n`-terminated raw write RUNS the command end-to-end) but does NOT update Marley's rendered buffer, so the inserted text is invisible/desynced — the shell has it, the screen doesn't. Diagnostic that nailed it in #59: the click FIRED + path was correct + write returned Ok, yet nothing showed → a `\r\n` write ran a command → isolated it to the raw-PTY-vs-cooked-buffer split. AFFECTS: #57 cmd-P finder Enter and #60 history search ALSO write_bytes the path → same invisible-insert bug (follow-up filed). Only the self-test caught this — a headless test or "write returned Ok" would have shipped it broken. When adding any "put text at the prompt" feature, route it through buffer.edit and self-test-capture the visible prompt.

## PR-claude-inspect-a-retained-artifact-before-parsing-it-001
*severity: medium*

Before writing a parser/audit/scan over a tool's output file (JSON, log, report), design it against a REAL retained instance of that file — not an assumed schema. Confirm three things from the artifact: the exact field names, the path-relativity (is a referenced path relative to CWD, to the output dir, or absolute?), and a real POSITIVE case (a row that should match your predicate, so you prove the parser fires, not just that it runs). #345's Timeout audit was designed against a retained `mutants.out.old/outcomes.json`: it confirmed the rows carry `.summary` + `.log_path`, that `log_path` is RELATIVE to `mutants.out/` (the file lived at `mutants.out.old/log/…`), and it held a live positive (a `Timeout` mutant whose log said `... FAILED`) — turning "the audit is plausible" into "the audit catches a proven real case." An assumed schema would likely have used the wrong path base and silently reported 0 matches forever (a false-negative advisory, not a crash — the worst kind, because it looks like it works).

## PR-claude-inspect-critic-must-leave-the-tree-as-it-found-it-001
*severity: medium*

An inspect critic that writes a throwaway probe MUST delete it AND revert any registration it added (a `mod foo;` line in lib.rs, a test-manifest entry), then confirm `git status --porcelain` shows only the expected modified files before returning. Two critics on #307 left artifacts behind despite explicit instructions — one left `zz_throwaway_critic.rs` PLUS its `mod` line in lib.rs, which also made `cargo fmt --check` dirty and would have polluted the commit. The orchestrator must therefore re-verify `git status` + `cargo fmt --all --check` after the critics return, and treat cleanup as the orchestrator's responsibility rather than trusting the instruction. Related and stricter: a critic must NEVER run `git checkout`/`stash`/`restore` (that destroyed an uncommitted tree on #259).

## PR-claude-inspect-critic-must-not-git-checkout-the-working-tree-001
*severity: high*

An inspect/review subagent must NEVER run `git checkout`/`git restore`/`git stash`/`git reset` on the working tree — the pipeline's Phase-3 changes are UNCOMMITTED, so any such command SILENTLY WIPES the implementation under review (a #205 critic ran `git checkout grid_layout.rs` after temporarily adding tests for a mutation run, destroying the uncommitted codec; it rebuilt the file from its earlier read + verified byte-accuracy 3 ways, but this is a near-miss data-loss). When spawning critics: (1) tell them the diff is UNCOMMITTED and they must treat the working tree as READ-ONLY — never checkout/restore/stash/reset/commit; (2) for mutation testing, `cargo mutants` already copies the tree to its own scratch dir (`cargo mutants --list` is pure read; a full `cargo mutants` run is self-contained) — do NOT hand-add tests + revert; (3) if a critic needs a mutated tree, use a separate `git worktree`, never the live files. After a critic that touched files, the ORCHESTRATOR must VERIFY the Phase-3 changes are intact (grep for the change markers + `cargo check` + run the tests) BEFORE proceeding — which caught this one clean.

## PR-claude-integration-only-coverage-fails-gate4-001
*severity: medium · prevents: BF-gate-cov-integration-lane-blindspot-001*

The gate's 100%-lines coverage run counts in-lib test lanes (unit modules + cfg(test) headless drives) but NOT tests/*.rs integration binaries. Any pure-fn line whose ONLY exerciser is an integration test will read uncovered and fail gate:4 — give it an in-lib caller too (a unit row for payload-less arms; a headless-drive assert when the arm needs live state, e.g. content.rs kind()'s Terminal arm asserted via the seeded-restore drive in #396).

## PR-claude-intent-add-new-files-before-diff-mutation-001
*severity: high · prevents: BF-new-untracked-file-skipped-by-in-diff-mutation-001*

When a ticket adds a NEW source file with a pure seam that must hit MSI 100, run `git add -N <file>` (intent-to-add) BEFORE the `--diff` gate — otherwise `git diff HEAD -- crates` (the `--in-diff` source) excludes the untracked file and cargo-mutants silently skips all its mutants, printing a false MSI-100 green. Verify with `cargo mutants --list --in-diff <diff>` that the new file's mutants actually appear. The whole-workspace coverage gate still measures the file (not diff-scoped), so a green coverage number does NOT imply the mutation gate saw it. Applies to every new-file pure module (search.rs, editor_problems.rs, selection_ladder.rs, …).

## PR-claude-invalidation-gen-bumped-by-all-mutation-sources-001
*severity: medium · prevents: BF-in-app-commit-stale-derived-cache-001*

A derived-state cache keyed on a generation/version counter must bump that counter on EVERY action that mutates the underlying state — including the app's OWN write paths, not just external/obvious events. When you add a gen-bump for the common triggers (save, reload), enumerate ALL the code paths that change the same underlying state (here: HEAD moves via the in-app commit/stage/discard actions, not only save/external-reload) and bump at each, or the app's own action leaves its own derived view stale. Grep for the write actions on the underlying state and confirm each bumps the gen.

## PR-claude-invariant-invisible-logic-needs-structural-tests-001
*severity: high*

When a transform's only OBSERVABLE projection is invariant under the very operation you're testing, output-only tests cannot kill that operation's mutants — you must assert the intermediate STRUCTURE. Example: coalesce_row partitions cells into styled runs, but Block::output_text() concatenates all run texts, and concatenation is invariant under re-partitioning — so every merge-vs-split mutant (the style-compare guard) survives an output_text-only test suite (measured: 7/9 mutants survived, MSI 22%). Fix: the test must assert run COUNT + per-run fg/bg/flags, not just the flattened text. Smell: a new function whose result feeds a lossy/normalizing projection (join, sum, sort, dedup, trim) that the existing tests assert — the pre-projection detail is untested. cargo-mutants --in-diff will expose it, but design the structural test up front.

## PR-claude-inverse-fn-tie-edge-needs-explicit-ac-test-001
*severity: medium*

A nearest/rounding/inverse function's tie-or-boundary rule (e.g. "on a tie, round to the LATER boundary") is often NOT guarded by the mutation set: cargo-mutants mutates the comparison operator one way (`<=`→`>`) but may never emit the other direction (`<=`→`<`) that would flip the tie, and if every return-value mutant dies to *any* non-trivial output, MSI can reach 100% without ever exercising the tie. So assert the exact tie/boundary VALUE as a first-class acceptance test (e.g. `offset_of_col(tie_col) == later_index`), never rely on mutation coverage to prove the tie direction. An inspect critic should specifically check that each nearest/rounding/clamp edge has a value-pinning test, not just that MSI is 100%.

## PR-claude-inverted-range-test-datum-spell-as-struct-literal-001
*severity: low*

When a Rust test needs a deliberately inverted/empty range as a DATUM (asserting defensive behavior on `start > end` input or output), never write the `a..b` literal with a > b — clippy's deny-by-default `reversed_empty_ranges` rejects it as an empty-iteration bug. Spell it `std::ops::Range { start: a, end: b }`: the struct literal states "this is data, not iteration", the lint correctly ignores it, and no #[allow] suppression is needed (CONSTITUTION §0).

## PR-claude-iterate-cursor-not-recurse-over-unbounded-tree-001
*severity: high · prevents: BF-recursive-tree-walk-stack-overflow-on-deep-input-001*

Walk a parse tree (or any structure whose depth comes from UNTRUSTED input) ITERATIVELY — a TreeCursor loop (goto_first_child / goto_next_sibling / goto_parent), never recursion. Recursion depth over a syntax tree is the FULL tree depth (every nested block/paren/expression level), NOT the semantic nesting you care about, so a machine-generated or adversarial file overflows the stack; on a UI thread that is an UNCATCHABLE SIGABRT (not catch_unwind-able) taking the whole app down, and a hot path (per-edit) re-runs it constantly. Never write "the input's depth is bounded" in a comment to justify recursion unless you MEASURED it — tree-sitter's own parser is iterative precisely because the claim is false. Marley precedent: #329 `enclosing_ranges` climbs iteratively via `.parent()`; #330 `collect_headers` must too.

## PR-claude-key-arm-above-the-keymap-must-gate-on-modifiers-001
*severity: high · prevents: BF-completion-popup-key-arm-missing-modifier-gate-001*

A key arm that sits ABOVE the keymap dispatch and matches on bare key NAMES will silently swallow every CHORD built on those names. Matching `"tab"` eats ⇧Tab (dedent); matching `"up"/"down"` eats ⌘⌥↑/↓ (add-cursor); matching `"enter"` eats ⌘Enter. The failure is not a dead key — it is the WRONG ACTION firing, and for an edit key that means corrupted text. Any overlay arm placed before the keymap must gate on modifiers FIRST and decline every chord: `if m.platform || m.control || m.alt || m.shift { return false }` before the key match. In Marley the shipped precedent is #96's popup arm, which carries exactly this gate and states the reason ("so ⌘↓ still jumps blocks, ⌘1 still switches tabs"). GENERAL LESSON, and the one that actually bit: when you mirror an existing idiom, diff YOUR arm against THEIRS line by line rather than reproducing its shape from memory — the guard you cannot see is the one that was bought by a bug. Check what your arm shadows by grepping the keymap for the bare key name (`keymap.rs` chords) and the router arms below your insertion point.

## PR-claude-key-migration-sweeps-ordering-surfaces-001
*severity: medium*

When migrating a domain key (old id type → new id type), sweep every SORT/ordering keyed by the old id, not just the equality gates: `Ord` on the new key can silently reorder user-visible lists even when all other behavior is byte-identical (in #396, fleet rows + broadcast fan-out sorted by PaneId tab-block order flipped to ContentId launch order — flagged independently by 3 of 5 inspect critics). Either preserve the old order at the render edge or record the delta as an explicit sanctioned deviation BEFORE validate, so the zero-visible-change claim is scoped honestly. Companion to PR-claude-domain-sweep-includes-equality-gates-001.

## PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001
*severity: medium · prevents: BF-claude-bracket-match-no-language-gate-parses-non-rust-as-rust-001*

When a pure primitive is language/format-specific BY CONSTRUCTION (e.g. it hardcodes a grammar like tree_sitter_rust), it will NOT self-gate — it returns a plausible-but-wrong result for foreign input, not None. The language/format gate belongs at the CALLER (the app seam that knows the file's language), and a sibling feature usually already has it to mirror (here `refresh_syntax_cache`'s `is_rust` check). Never write "non-X falls out for free" in a design without VERIFYING it: feed the primitive a non-X input and watch what it returns — if it returns Some/a value, the gate is required, not free.

## PR-claude-live-drive-click-first-activation-does-not-make-the-window-key-001
*severity: medium*

On macOS 26, NSRunningApplication.activate() (drive.swift `focus`) makes the app frontmost but does NOT make the gpui window KEY — synthetic keystrokes and chords are silently dropped until a real synthetic CLICK lands in the window. A live drive must post a click before any type:/cmd:/cmdshift: action (`focus clickat:… type:…`), and a no-change capture after keystrokes should be diagnosed by testing a MOUSE action first (mouse landing while keys don't is the tell). Two adjacent traps from the same session: `open <app> --env` on an ALREADY-RUNNING app silently activates the stale instance and drops the env (kill first, verify with pgrep); a sandbox HOME on a removable volume (/Volumes/*) fires a TCC "removable volume" prompt that wedges boot pre-window — put drive sandboxes on the internal disk (TCC dialogs are the operator's, never clicked).

## PR-claude-live-refresh-selection-identity-key-must-be-unique-001
*severity: low · prevents: BF-live-list-selection-identity-key-not-unique-001*

When a live-refreshed list keeps its selection by IDENTITY (re-finding the selected row by value after a rebuild, not by index), the identity key must UNIQUELY identify a row. A coarse key (e.g. (path, line)) that two rows can share makes `.position()` match the FIRST collision and teleport the selection to the wrong row. Include enough fields to disambiguate every row that can legitimately coexist (for diagnostics: path + line + character + severity; ideally a stable per-item id if the source provides one). Prove it with a test that has two rows sharing the coarse key and asserts the refresh keeps the RIGHT one.

## PR-claude-liveness-published-only-after-peer-status-verified-001
*severity: medium*

Publish a "connected/live" state only after the PEER's response is verified (status line typed non-failure, handshake acked) — never at socket-connect or request-write success. A write success is not an established stream: a server that accepts the connection and never answers otherwise shows "live" unboundedly (the bounded-flap 404 case is the benign sibling; the silent-accept case is the killer). Place the publish at the same point the code types the peer's status.

## PR-claude-llvm-cov-line-miss-is-an-uncovered-closure-region-001
*severity: medium*

When a `--diff`/FULL coverage gate reports a file BELOW 100% lines but `cargo llvm-cov report --show-missing-lines` lists NO lines for that file, the miss is NOT a fully-dead line — it is a never-called CLOSURE (an uncovered region on a line whose OTHER region is hit, so HTML renders it 'covered' and --show-missing-lines omits it, yet cargo-llvm-cov counts it against LINE coverage). Diagnose by: (1) the missed-FUNCTIONS column in the summary table (it equals the count of uncovered closures); (2) lcov LF/LH confirming N uncovered; then (3) READ the code for an unreached closure — a `.unwrap_or_else(||…)`/`.map_or`/`.or_else` fallback that no test's input triggers, or a sort comparator's trailing `.then_with(||…)` tiebreak reached only when the prior keys tie. Cover it with a test that forces that exact branch (a non-`file:` uri for a uri-decode fallback; two items equal on all prior sort keys for a tiebreak). Confirm with an ISOLATED `cargo llvm-cov nextest --fail-under-lines 100` (~5 min) BEFORE re-running the full ~20 min gate. Distinct from the sibling trap where a never-taken `panic!`/`unreachable!` MATCH ARM is a fully-uncovered line that --show-missing-lines DOES list (fix: replace the match with a branch-free whole-value assert_eq!).

## PR-claude-llvm-cov-stale-profraw-masquerades-as-phantom-region-001
*severity: high*

Before concluding a coverage miss is an llvm-cov "phantom region" artifact (and arguing an exclude), run `cargo llvm-cov clean --workspace` and re-measure. If the numbers reproduce exactly, the measurement is REAL. Stale `.profraw` from an earlier build produces precisely the symptoms that look like a tooling bug: the per-line `--text` export disagreeing with the summary, and — the tell — "the counts did not move after I changed the code" (they didn't move because the report was never measuring the new code; that observation is near-proof of staleness, NOT of an artifact). To name the uncovered symbol use `--json`, not `--text`: the JSON lists functions PER INSTANTIATION (a crate compiled into two objects yields two entries per fn, distinguished by crate hash) while the summary MERGES them by taking the max, so a fn that is zero in one instantiation but covered in the other is invisible in `--text`. The one true miss is the entry with NO covered sibling. Real case (#313): a `.then_with(|| ...)` third tie-break rung was zero in both instantiations — a genuinely reachable, genuinely untested branch that the phantom hypothesis would have excluded away.

## PR-claude-loopbound-fixture-and-childproc-bin-skip-001
*severity: medium · prevents: BF-claude-mutation-survivors-loopbound-childbin-serial-001*

Mutation-killing fixtures for an array/pixel walk with a `len = w * h` (or `w * stride`) loop bound MUST use dimensions where `w * h != w / h` AND `w * h != w + h` — i.e. a multi-ROW image, never a `w×1` strip — and assert elements BEYOND the first row/segment; otherwise cargo-mutants' `*`→`/` and `*`→`+` arithmetic mutants are observationally EQUIVALENT and survive. Separately: any runner/fixture BINARY whose `main()` executes only in a spawned CHILD process (observed out-of-process by a headed/#[ignore] test) cannot have its mutants killed in-process — mark `main()` `#[cfg_attr(test, mutants::skip)]` and add it to the coverage exclude, like the rest of a display/subprocess shim. And: tests that set/unset a process env var need `#[serial]` because cargo-mutants runs the THREADED `cargo test`, where nextest's per-process isolation does not apply.

## PR-claude-lsp-advertise-client-capability-001
*severity: high · prevents: BF-lsp-codeaction-inert-without-client-capability-001*

A result-reading LSP feature STILL needs its CLIENT capability advertised in `initialize` when the SERVER gates its reply on that capability. rust-analyzer withholds `CodeAction` literals without `textDocument.codeAction.codeActionLiteralSupport`, and returns the legacy `changes` map (no version) without `workspace.workspaceEdit.documentChanges`. A mocked/fake-LS wire returns whatever the test scripts, so it can NEVER catch an unadvertised-capability omission — the feature passes every unit test and is dead against the real server. Rule: for any request/response LSP feature, (1) advertise exactly the client capabilities the feature implements (nothing more — honest), and (2) prove it with ONE live drive against the real server, not just the fake fixture. Applies to codeAction, workspaceEdit/documentChanges, and any future feature whose server reply is capability-gated.

## PR-claude-lsp-version-guard-inert-without-documentchanges-cap-001
*severity: high*

An LSP client's per-file "apply this edit only if the doc version still matches" staleness guard is INERT unless the client advertises `capabilities.workspace.workspaceEdit.documentChanges: true` at initialize. Without that capability, a spec-conformant server (rust-analyzer) returns a WorkspaceEdit as the LEGACY `changes` map ({uri: [TextEdit]}), which carries NO version — so `version_conflict(edit_version=None, synced)` is always false and the edit applies UNCONDITIONALLY against whatever the buffer now holds (a stale rename/code-action edit lands on shifted spans → corruption). The guard reads correct in code and does nothing at runtime. Caught at #323 inspect; it also silently defeated #322 rename's version guard (both shipped relying on it). FIX: advertise `documentChanges: true` (honest only if your parser handles the `documentChanges` shape — Marley's does, and prefers it) so the server sends VERSIONED `TextDocumentEdit`s; do NOT also advertise `resourceOperations` unless you actually apply file create/rename/delete. A belt-and-suspenders alternative is a client-side live-buffer-version recheck at apply time (independent of the edit carrying a version). LESSON: a guard that depends on a field the server only sends when you asked for it is not a guard until you've asked — assert the advertised capability, and test that the version actually arrives on the wire.

## PR-claude-match-arm-filter-needs-hand-asserts-not-mutation-001
*severity: medium*

cargo-mutants does NOT mutate match/`|` pattern arms, and llvm-cov coverage is line-based (a `matches!(x, A | B)` filter line executes regardless of which variants it admits). So a pure fn whose CORRECTNESS lives in a status/variant SET (e.g. fleet_badge counting `Working | Waiting`) is NOT protected by the MSI-100 + coverage-100 gates against a set regression (drop a variant, add a wrong one). When a filter/match set is the actual spec, write EXPLICIT unit asserts for each side: one input per admitted variant → included, and one per excluded variant → excluded. Treat those asserts as load-bearing (never prune as "redundant with mutation") and say so in the test comment.

## PR-claude-method-call-code-mutation-hollow-001
*severity: medium*

cargo-mutants mutates OPERATORS and whole-function returns, but NOT method calls (`.exists()`, `.parent()`, `.file_name()`), struct-field assignments, or an in-loop `return`. So a pure fn built mostly from method calls + struct construction (path walking, filesystem-shaped logic, builders) generates FEW or ZERO viable mutants, and MSI 100 becomes a HOLLOW signal — satisfied by an unrelated helper's mutants (or an unviable whole-body mutant that gets excluded) while the load-bearing logic is mutation-untested. #53 `discover_in`: its entire nearest-`.git`-walk produced zero viable mutants; MSI 100 rode on the separate `name_for`'s two String mutants. Rule: when a fn is method-call/struct-heavy, do NOT trust MSI as proof of correctness — the real guard is 100% COVERAGE + BEHAVIORAL assertions pinning the exact contract (nearest≠farthest, dir-OR-file, fallback value). At inspect, enumerate the actual mutants (`cargo mutants --list`); if the load-bearing fn has none viable, verify the behavioral tests would catch the bug a mutant WOULD represent (e.g. temporarily add `#[derive(Default)]` to make the whole-body mutant viable, confirm it's caught). Also keep a small pure helper (like `name_for`) that DOES yield viable mutants so the crate never hits the gate's 0-viable-mutants fail-closed branch. Sibling of the #50 vacuous-MSI-via-normalizing-projection rule — both are "MSI 100 doesn't mean what it looks like."

## PR-claude-modal-open-dismisses-lower-overlay-keys-001
*severity: high · prevents: BF-modal-open-over-live-completion-steals-keys-001*

Opening a MODAL that owns the keyboard must FIRST dismiss any other overlay whose key arm runs BEFORE it in the router. A live completion popup (or any earlier-dispatched overlay) will keep consuming ↑/↓/Enter/Tab — so the modal appears but can't be navigated, and the stolen Enter may edit the buffer behind it. The chord that OPENS the modal often falls through the lower overlay's handler (which only consumes Esc), so the lower overlay never self-dismisses. Clear it in the modal's open fn (mirror the launcher-open choke). Sibling of the passive-card-over-palette class.

## PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001
*severity: medium*

A modal opened by a MOUSE click (a rail/toolbar affordance: the #181 agent launcher, the #393 section ＋ menu) bypasses the key router's overlay guards, so it must explicitly close EVERY interactive overlay whose key-arm precedes it in the router — or it opens keyboard-dead (keys go to the still-open picker underneath; a text-entry draft even captures them). Occlusion is NOT a safe excuse: centered pickers are single `.absolute()` cards with no full-window scrim, so a left-rail affordance is not covered and the co-open is reachable. Do not hand-copy the closer list per call site (it drifts as new overlays are added) — factor ONE shared `close_transient_overlays()` helper and call it from every mouse-opened modal. When adding a new interactive overlay with a key-arm, add it to that one helper. (Marley: the launcher accreted this fix across 8 separate inspect findings #229/#312/#317/#323/#324/#325/#326/#327 before #393 unified it.)

## PR-claude-mouse-overlay-mirror-new-agent-close-set-001
*severity: medium*

When adding a NEW overlay that is (a) opened by a MOUSE click (a top-bar icon / indicator / button) and (b) paints a full-screen dismiss backdrop, it bypasses the key-router's overlay-dismiss guards — so its open-handler MUST explicitly close every OTHER transient overlay, including the keyboard-opened no-backdrop cards (finder_open / history_open / find_open) and the completion popup, not just the obvious floating menus (context_menu / agent_launcher / palette). Mirror the canonical `new-agent` top-bar arm's close-set by grepping it (app.rs ~3319 — it enumerates palette_open/finder_open/history_open/find_open/naming_workflow) rather than re-deriving the list from memory; a partial reset leaves a keyboard-live overlay orphaned under the new backdrop (the #229 class). Do NOT reset persistent docks (forge_open/fleet_open/files_open). Prevents BF-claude-mouse-opened-backdrop-overlay-skips-keyrouter-overlay-guards.

## PR-claude-mutation-does-not-mutate-call-arguments-001
*severity: high · prevents: BF-claude-undo-group-absorbed-a-typed-char-and-destroyed-text-001*

MSI 100 does NOT cover a literal you pass at a CALL SITE. cargo-mutants mutates a function's BODY (its return value, its operators, its match arms) — it does NOT mutate the ARGUMENTS a caller passes in. So a `true`/`false`/`0`/`"//"` written at a call site is invisible to mutation, and a green MSI says nothing about whether it is the right value. On #299 the ENTIRE FIX for a data-corrupting undo bug was two literals at two call sites — `begin_group(sel, false)` for hand-bracketed groups and `begin_group(restore, true)` for the typed-insert path. Flip either one and the bug returns (⌘Z deletes the user's code), yet `cargo mutants --list` produced no mutant for either, and the file still reported MSI 100. THE RULE: when a fix or a behavior lives in an ARGUMENT rather than in a function body, mutation cannot defend it — pin it with a BEHAVIOR test in BOTH directions. On #299 that is two tests, and neither is redundant: one fails if a hand-bracketed group is wrongly marked cursor-anchored (the bug returns), the other fails if the typed-insert path is NOT marked (typed-run coalescing breaks). GENERALISE: this is the same family as `PR-claude-decision-moved-into-a-gate-excluded-file-needs-its-own-test-001` (#298 — moving the find bar's case-fold decision to a call-site argument put it in the one file excluded from BOTH the coverage and mutation gates). Both say: **a decision that migrates from a function body to a call site walks out from under the gates that were protecting it.** Whenever you introduce a flag/enum/token parameter, ask what now pins its VALUE — and if the answer is "nothing", write the test that flips it and watch something go red. VERIFY BY SABOTAGE.

## PR-claude-name-the-function-that-actually-runs-on-that-path-001
*severity: medium · prevents: BF-claude-undo-granularity-comment-cited-the-wrong-coalesce-path-001*

When you justify a grouping / coalescing / batching decision, name the function that ACTUALLY executes on the path you are describing, and prove it is reachable there — do not cite the mechanism that governs the neighbouring case. Concretely in this codebase: a single-cursor edit never opens an undo group, so `undo::coalesces_into` (reachable only from `end_group`, i.e. group-into-group) CANNOT explain single-cursor ⌘Z granularity; the operative guard is `UndoHistory::record`'s `sel_before.is_none() && records.len() == 1`. Test for this class: ask "which call site invokes the function I just named, and does my scenario reach it?" If the answer is a different arity/branch than the one you are documenting, you have cited the wrong sibling. Two tells that you are in this trap: (1) a shipped precedent nearby already states the rule correctly in different words — find it and match it (here, `Buffer::edit_ranges_restoring_placing`'s single-member fast-path note); (2) your justification would still "sound right" if the flag it depends on were flipped, which means a future cleanup of that flag silently invalidates your decision. A correct decision defended by an unreachable mechanism is a latent regression, because the next person will refactor the mechanism and take the decision with it.

## PR-claude-nav-home-entry-is-the-container-not-the-cell-001
*severity: medium · prevents: BF-claude-per-cell-home-rows-tripled-the-section-001*

When cross-listing container-hosted content into a navigator's home sections, the container's OWN row is the home entry for everything it hosts — derive extra per-content rows only when the content's home section has no row for it at all (a kind that files under a different section), and derive markers from CROSS-CONTAINER hosting (content in >= 2 distinct containers), never from container arity or per-cell structure. An arity-based rule multiplies rows for every ordinary split and churns when cells close; a hosting-count rule is stable under both. Sanity-check any new listing rule against the plainest pre-existing state (one split, one tab) before shipping it.

## PR-claude-negative-smoke-must-reproduce-the-reverted-bug-observably-001
*severity: high*

A negative-smoke ("break the fix, watch the test fail") proves the test can catch the bug ONLY if reverting the fix actually makes the test FAIL on that fixture. Before trusting a smoke, trace the reverted code on the chosen fixture and confirm it diverges OBSERVABLY — a downstream clamp, a self-destroying input, or an overshoot-that-still-advances can mask the reverted bug so the smoke passes either way (false confidence). Two #347 cases: (1) an F2 "resume from expanded length" smoke on a 2-match fixture was masked because after replacing match 0 the second match's index clamped via `.min(matches.len()-1)` to 0 for ANY resume value — fix: a ≥3-match fixture with a SURVIVING MIDDLE match so a too-large resume skips it. (2) an F3 "advance past a zero-width match" smoke could not reproduce the loop because the reverted guard overshot (`s+template_len`) but still ADVANCED — over 2 line-starts the overshoot clamped invisibly; fix: a short-line fixture where the reverted overshoot SKIPS a match, asserting the advance lands on the IMMEDIATELY-NEXT match, not merely "the index moved". Rule: pick the fixture so the reverted fix produces a DIFFERENT observable (different final text / a specific next index), not just "still terminates" or "still changed something".

## PR-claude-new-asset-needs-loader-registration-and-guard-001
*severity: high*

WHEN adding a rendered asset (SVG icon, image, font) to a gpui app whose AssetSource is a static match (include_bytes! per path — Marley's Assets::load), the SAME change must (1) add the loader match arm and (2) extend/keep a guard test that walks EVERY enum variant through its path fn into AssetSource::load asserting Some(non-empty) — because a missing registration renders as a SILENT blank: compile, unit tests, and clippy all stay green. Never claim a "compile proves the asset exists" verification without checking the loader actually embeds a directory.

## PR-claude-new-chord-shadowed-by-hardcoded-key-001
*severity: high*

When adding a keymap chord whose BASE KEY matches an existing HARDCODED key check in the input path (app.rs has direct `event.keystroke.key == "x"` intercepts for the find bar, paste, etc. that run BEFORE the keymap dispatch and `return`), the chord can be silently SHADOWED. The keymap `action_for` unit test does NOT catch this — it verifies the binding table, not runtime reachability. BEFORE relying on a new chord: (1) grep app.rs for any hardcoded `key == "<base>"` intercept sharing the base key, and confirm it excludes the new modifiers (the cmd-F find-bar check needed `&& !modifiers.shift` so cmd-shift-f falls through); (2) in validate, SELF-TEST-DRIVE THE REAL KEYSTROKE and capture the effect — never accept the unit test alone for an input-path feature. #64's cmd-shift-f was dead-on-arrival (opened the find bar) until fixed; the shadow was invisible to cov/MSI (app.rs is masked) and to the passing keymap test.

## PR-claude-new-overlay-register-at-every-choke-point-001
*severity: medium*

A new keyboard-owning overlay is not done when its key arm works — register it at EVERY choke point, and grep for the existing overlays' names to find them all. In Marley that means at least: (1) `text_input_blocked` (the #267 one-predicate gate on the OS/IME text path). Do NOT reason "cx.stop_propagation() already covers me" — it covers the common path only; gpui's mac window computes `is_composing` from `marked_text_range()` (exactly what this predicate forces to None) and routes a live IME composition AND non-printing keys (arrows/escape — an overlay's own keys) to the input context's handleEvent: BEFORE run_callback. (2) Every "close the other overlays first" arm reachable by MOUSE (e.g. the agent-launch icon), because an icon click bypasses the key router's overlay guards entirely and leaves your overlay stacked, swallowing keys under the new one. The tell that a choke point exists: a predicate or an arm that already enumerates the shipped overlays by name, and whose doc says something like "a NEW overlay only needs a line here" — believe it, and add the line.

## PR-claude-new-palette-arm-extend-exhaustive-map-test-001
*severity: medium*

Adding a `CommandId(N) => Some("verb")` arm to `action_for_command` (or any table with an exhaustive "maps every row" test) REQUIRES extending THAT test with the new id → exact verb. Otherwise the new line is uncovered → fails cov-100 AND the diff-mutation (None/""/xyzzy/delete-arm all survive). A headless drive dispatching the VERB string (through dispatch_action) does NOT exercise action_for_command (which maps CommandId→string), so it never covers the new arm.

## PR-claude-new-persist-fn-needs-round-trip-test-001
*severity: medium*

When adding a persist_X(manager, ...) fn to marley_app/settings.rs (the #26/#87 pattern), it is NOT covered by the applied_from/applied_defaults tests (those only READ). Add it to settings_round_trip_survives_reload (or a dedicated test): persist a NON-DEFAULT value, drop, reload, assert it survives — this both covers the fn AND kills its delete/replace mutant (a broken write reloads as the default, which the assert catches). #95 shipped persist_right_section uncovered → gate RED at MSI 80% until the round-trip test was extended.

## PR-claude-new-persist-setting-needs-round-trip-test-001
*severity: medium*

A new `persist_*` settings fn (a one-liner `manager.set::<T>(...)`) needs its OWN round-trip test (persist → load_manager_in → get::<T>, asserting equality) — even though it looks trivial. Without a test caller its body line is uncovered (gate 4 rust_cov `--fail-under-lines 100` does NOT exclude settings.rs) → coverage <100, AND its `→ Ok(())` mutant survives (gate 5 MSI 100, and under --diff since it's an in-diff line) → the commit gate goes RED. Adding the setting's field to an existing AppliedSettings fixture covers only the GET/default path, not the persist path. Mirror the existing `*_setting_round_trips` tests. Bit #234 (persist_recents).

## PR-claude-new-render-arm-mirror-sibling-guards-001
*severity: medium*

When adding a NEW render arm (or relocating rows into one) that parallels an existing sibling row in the same list — e.g. a new RailLevel variant beside the existing ones in Marley's rail render match — audit the sibling arm for RENDER-TIME GUARDS and replicate the applicable ones. Common guards that are easy to miss: the "Search tabs"/filter skip (`if !self.session_filter.is_empty() { continue; }`), visibility/collapse gates, and label-match filters. A row with a GENERIC label (one that can never match a title/text search — "PANE n", "pane k") MUST carry the same filter-skip its generic-labeled sibling carries, or a filter leaves dangling parent rows with hidden children. Diff the new arm against its nearest sibling arm line-by-line for `continue`/guard statements before shipping — the compiler won't catch a missing render guard (it's behavioral, not type-level).

## PR-claude-new-setting-needs-nondefault-roundtrip-leg-001
*severity: medium · prevents: BF-persist-setter-body-mutant-survives-no-roundtrip-001*

Every NEW setting needs FOUR wirings, not three: the `define_setting!`, the `AppliedSettings` field + both resolvers (`applied_from`/`applied_defaults` — the compiler forces these), AND a leg in the shared `settings_round_trip_survives_reload` test. Without the fourth, the `persist_X -> Ok(())` body mutant survives (the setter silently never writes) and gate:5 goes red — the compiler cannot force it because the setter is only reachable from the app shim. Persist the NON-DEFAULT value in the round-trip (e.g. `false` for a default-true setting) so a write that never happened reloads as the default and fails the assert; persisting the default value would pass even with the mutant.

## PR-claude-no-adhoc-python-splice-on-source-use-edit-tool-001
*severity: medium*

Never string-splice a real source file with an ad-hoc `python3`/`sed`/`awk` script (find-index + slice + concat). It is a corruption risk — a wrong index or a lost newline duplicates or mangles the file (Marley #362: a python splice to remove a verification spike DUPLICATED crates/syntax/src/lib.rs, producing two copies of every fn). Use the Edit tool (exact-match old→new, atomic, fails loudly on ambiguity) for every in-tree edit. A verification SPIKE belongs in a throwaway scratch bin or a `/tmp` crate, NOT spliced into a real crate's `#[cfg(test)]` module — if it must go in-tree, add it with Edit and remove it with Edit (the exact inverse block), never a script. Recovery when a splice does corrupt a tracked file: `git show HEAD:<path> > <path>` restores ONLY that file from HEAD (a read-to-stdout + your own write — NOT `git checkout`/`restore`, which the tree-safety rule bans because it can wipe other uncommitted work), then re-apply the intended edits with Edit.

## PR-claude-no-dead-defensive-clamp-untested-001
*severity: medium · prevents: BF-claude-dead-defensive-clamp-equivalent-mutant-001*

A defensive input-clamp (`x.min(len)`, `x.max(0)`, a saturating guard) that is IDENTITY on every input the caller-contract permits is DEAD CODE on the tested path → cargo-mutants' remove/min↔max mutant is EQUIVALENT/unkillable (no contract-valid test observes it), silently blocking MSI 100. Resolve by EITHER (a) removing the clamp and documenting the in-range caller-contract (let misuse be a contract violation — panic/garbage — consistent with the crate's other entry points), OR (b) genuinely exercising the misuse path with an out-of-range fixture so the clamp becomes observable/killable. Do NOT ship an untested defensive clamp. Same family as the structural-lower-bound `.max(0)` equivalent (usize ≥ 0 already) — only clamp a bound the input can actually violate under contract, and keep sibling functions' out-of-range behavior + their doc promises CONSISTENT.

## PR-claude-no-default-struct-return-needs-full-value-assert-001
*severity: medium*

A pure fn that returns a STRUCT with no `Default` derive (e.g. `type_scale(Role) -> TextStyle{size, weight}`) yields ZERO viable cargo-mutants: the only whole-fn mutant is `-> Default::default()`, which is UNVIABLE without a Default impl, and cargo-mutants does NOT mutate the field literals or swap enum-arm bodies inside the match. So cov 100 + MSI 100 are BOTH reachable with a WEAK test — asserting one field, or exercising an arm without pinning its value — and the gate stays green while sibling arms remain indistinguishable (e.g. `Command→(14,Medium)` vs `Output→(14,Normal)` collapse under a size-only assert). This is the struct-return cousin of the hsla-not-mutated (palette) and self-oracle traps: MSI cannot enforce value correctness when there are no viable mutants. Rule: for any arm-returning pure fn whose return type has no Default, the test MUST assert the FULL returned value (all fields) for EVERY arm — rely on exact-value regression asserts, never on MSI, to pin the data. (For fns returning a primitive like f32, the whole-fn→0.0/1.0/-1.0 mutants ARE viable but force only ONE discriminating assert — still assert every arm's exact value.)

## PR-claude-no-public-to-private-intra-doc-link-001
*severity: medium*

A doc comment on a PUBLIC item (a pub struct/field/fn reachable in the crate's public API) must not use an intra-doc link `[`x`]` to a pub-fn-or-type that lives in a PRIVATE module (`mod foo;`, not `pub mod`). rustdoc's `private_intra_doc_links` lint (`-D warnings`, gate:14) rejects it because the link resolves only with `--document-private-items`. Use plain backticks `` `x` `` (no brackets) for such references, or make the target public. Private items' own docs may link freely (they're not built without the flag).

## PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001
*severity: medium*

In a `set -euo pipefail` hook, never end a pipeline with `grep -q` (or any early-exiting consumer) fed by a producer that may still be writing: the quiet grep exits at its first hit and closes the pipe, the upstream stage takes SIGPIPE, and pipefail turns a SUCCESSFUL detection into a failed pipeline — a hook that false-BLOCKS only once the input grows past the pipe buffer (Marley: enforce-tests-ran.sh worked for months, then false-blocked a compliant session at a 49MB/9k-command transcript; three genuine `cargo nextest run --workspace` runs were 'not detected'). Fix shape: count survivors with `grep -c`/`grep -vc` (consumes all input, deterministic, identical semantics) and compare to 0. Audit any hook combining pipefail + `-q` mid-pipeline the same way.

## PR-claude-non-exhaustive-variant-blocks-construction-001
*severity: medium · prevents: BF-non-exhaustive-enum-vs-variant-001*

To make an enum's variants non-constructible AND non-exhaustively-matchable from other crates (the "construct only via a smart constructor" pattern), put `#[non_exhaustive]` on each VARIANT, not just on the enum. Enum-level `#[non_exhaustive]` only forces a wildcard arm in downstream matches — it does NOT prevent constructing a known variant (tuple/struct-variant fields inherit the enum's public visibility). Always back a "cannot construct externally" invariant with a `trybuild` compile-fail test; a unit test cannot prove a compile error, and reviewers routinely mis-recall the enum-vs-variant `#[non_exhaustive]` distinction.

## PR-claude-nonblocking-readloop-needs-backoff-and-real-io-test-001
*severity: high · prevents: BF-claude-nonblocking-pty-busyloop-no-backoff-races-child-001*

A poll loop over a NON-BLOCKING fd (O_NONBLOCK PTY/socket) MUST yield between WouldBlock polls (a small thread::sleep or an OS poll/epoll wait) — a bare busy-loop with only a poll-COUNT budget exhausts in microseconds and races a child that needs milliseconds to write, returning an empty/clean result. A mock-reader unit test (which returns scripted bytes instantly) CANNOT catch this — it has no real timing — so any such loop needs a real-IO integration test (spawn an actual child over a real PTY and assert the output landed). More generally: bugs in process/PTY/socket plumbing (drop ordering, fd lifetime, poll timing) survive green `cargo check`+`clippy`+unit tests and are caught only by an end-to-end test against the real OS resource — budget for that test in every adapter that spawns or reads a child.

## PR-claude-none-state-assert-is-some-not-map-001
*severity: medium*

Under a 100%-line coverage floor, assert a "state is None at boot" condition with `.is_none()`/`.is_some()`, NEVER `option.map(|x| …)` — `Option::map` short-circuits on `None`, so the mapping closure is never executed and llvm-cov counts it as a missed function + missed line. This bites specifically when the value under test is None on the path being asserted (e.g. a fresh boot before a feed installs a snapshot).

## PR-claude-nonorigin-interior-fixture-for-index-arithmetic-mutants-001
*severity: medium · prevents: BF-claude-origin-collapse-fixture-survives-index-mutants-001*

To kill cargo-mutants index/offset-arithmetic mutants (in `(y*w + x)*k`-style addressing), the test fixture MUST exercise a NON-ORIGIN interior cell: pick y≥1 AND x≥1, in a grid that is multi-row AND multi-col AND non-square (w≠h), with an UNMASKED/untouched remainder to assert against. Reason: at the origin the index is 0, a fixed point of `*` `/` `+` `-` (0*k==0/k==0+0==0-0), so `*`→`/`, `+`→`-`, `+`→`*` etc. all produce identical output and survive; a 1-wide (or 1-tall) grid makes x (or y) always 0 with the same effect; a square grid lets a w↔h transpose mutant survive. This generalizes the multi-row loop-bound rule (PR-claude-loopbound-...) to ALL index arithmetic, not just bounds. Assert BOTH the touched cells (== expected) AND an untouched cell (== original) so over-/under-indexing both die.

## PR-claude-ordered-events-coalesced-stream-hooks-001
*severity: high · prevents: BF-claude-coalesced-pty-read-passthrough-before-hooks-drops-output-001*

When a single read from a coalescing stream (PTY/socket) interleaves CONTROL hooks (DCS/OSC/escape boundaries) with CONTENT bytes, preserve byte-stream ORDER end-to-end — emit ONE ordered event stream (Passthrough(bytes) | Hook(...) interleaved) and process in order. NEVER flatten content into one buffer and hooks into a separate list (loses "content came after that hook"): a hook that opens/resets the content sink must be applied BEFORE the following content is rendered, else the content is dropped/mis-attributed. PTYs coalesce, so a fast command's open-hook+output+close-hook arrive in ONE read — the bug only shows when reads DON'T split on the boundary, so it's timing-flaky and passes check/clippy/split-read tests. A COALESCED-read integration test (one read carrying the whole command) catches it.

## PR-claude-ordering-invariant-not-provable-by-call-count-001
*severity: medium*

A test that claims to prove an ORDERING or TIMING invariant — probe-before-tick, event-before-render, read-before-write, check-before-first-advance — via a CALL COUNT of the callback/probe CANNOT discriminate the two orderings: both typically invoke the callback the same number of times, and only the SIDE EFFECT the ordering gates (a mock-clock advance, a spawned task running, a repaint, a written byte) differs. Assert that gated side effect DIRECTLY, so the OTHER (un-fixed / wrong-order) outcome would FAIL the test. Concretely for a probe-first vs body-first poll helper: `calls == 1` holds for BOTH (probe-first probes then skips the loop; body-first ticks once then probes) — the discriminator is the TICK, so read the mock clock (gpui `BackgroundExecutor::now()`, which `advance_clock` moves) before/after and assert a ZERO delta to prove no tick preceded the probe. Seen in #364 (the poll_until_pre pre-satisfied unit); same family as the vacuous-drive (#362, PR-...-suppression-test-must-prove-the-baseline-would-not-suppress) and tautological-equivalence (#363, PR-claude-delegation-equivalence-test-tautological) lessons — trace whether the assertion would still pass against the pre-fix baseline; if yes, it proves nothing.

## PR-claude-ownership-move-reproves-every-per-copy-default-001
*severity: high · prevents: BF-claude-view-to-instance-field-kept-per-copy-default-001*

When a field's OWNER changes from per-view/per-copy to per-instance (a registry/shared-ownership migration), every "this copy can leave it unset/default" argument dies with the old ownership and must be RE-PROVEN against ALL birth orders: the first birth's default becomes every later view's reality, and a later birth's own initialization is typically DISCARDED on the dedupe hit. Sweep every constructor/seed site of the moved field and ask "what does view N+1 inherit if THIS site ran first?" (TICKET-397: a split-first open left the shared #275 disk snapshot None, blinding external-change detection for the tab twin — silent save-clobber.)

## PR-claude-paired-copy-and-highlight-must-cover-same-row-set-001
*severity: medium*

When two surfaces are specified to represent "the same selection" — e.g. a COPY/serialize path (`selected_text`/`copy_payload`) and a VISUAL highlight path (the per-row tint) — an asymmetry in WHICH rows/cells each one covers is invisible to per-function unit tests and to mutation testing, because each side is internally correct in isolation. The invariant only breaks at the shim seam where they meet. Verify explicitly that the two surfaces ENUMERATE THE SAME set: if copy includes a row kind (e.g. command-header rows), the highlight must tint that same row kind (and vice-versa). In #44, copy included command-header rows but the #43 highlight painted only output rows → a drag across a header copied un-highlighted text. Fix by making the highlight cover the header rows too (Warp copies commands, so copy-includes-command is the right direction; the highlight just had to match). Add a masked-visual check that drags across BOTH a header and output.

## PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent-001
*severity: low*

When choosing a theme accent (or any colored surface that carries text/icons), pick its lightness so the paired on_accent text clears WCAG AA contrast (≥4.5:1 for normal text, ≥3:1 for large/bold). A MID-lightness accent (L≈0.42–0.55) is the trap: it fails against BOTH white AND black text, so neither on_accent choice is readable. This bites the LIGHT theme hardest — a teal/blue accent at L~0.42 with white text lands ~3.4:1 (fails AA-normal). Fix: push the accent dark enough (L≈0.32–0.36) that white text clears AA, or light enough that black text does. In a headless/env-blocked context you CANNOT eyeball this — reason it numerically (hsla L → relative luminance → contrast ratio) at design time. Assert the semantic-color distinctness in tests, but contrast is a numeric design check, not a test. Load-bearing whenever a palette adds colored chrome (status dots, badges, buttons).

## PR-claude-parallel-critics-must-not-share-a-working-tree-001
*severity: high · prevents: BF-claude-cmd-d-exhausted-guard-false-fired-after-the-wrap-001*

NEVER run parallel adversarial critics that MUTATE a shared working tree — they corrupt each other's evidence, and a corrupted critic reports FALSE PASSES on real bugs. On #298 four concurrent critics wrote scratch tests into the same two source files. Critic 1 got two of its findings back as PASSING because it compiled against another critic's transient patch; it only caught this because a result was ARITHMETICALLY IMPOSSIBLE against the on-disk code, whereupon it copied the crate to an isolated workspace and re-ran everything there. Critic 4 watched the tree change three times mid-review. Both HIGH findings would have been silently missed. MITIGATIONS, in order of preference: (a) spawn critics with `isolation: "worktree"` so each gets its own checkout; (b) instruct each critic to write scratch tests ONLY to a uniquely-named `tests/<critic-id>.rs` integration file and NEVER to touch `src/`; (c) make critics read-only (Explore) and have them hand back runnable test SOURCE for the orchestrator to execute serially. ALWAYS, regardless: snapshot the uncommitted tree (`git diff > snap.patch` + per-file `shasum`) BEFORE spawning any critic, and verify the shasums afterward — on #298 that snapshot is the only reason the ticket's diff survived four agents editing it. This composes with PR-claude-inspect-critic-must-not-git-checkout-the-working-tree: a critic must never `git checkout` (it wipes the uncommitted ticket), and must revert its scratch with surgical Edits.

## PR-claude-parallel-extraction-paths-each-need-region-test-001
*severity: medium*

A shim/extraction line added to N parallel code paths (e.g. a live-snapshot fn AND a command-finish fn that both read the same source) needs a test exercising the NEW region on EACH path. Identical code on two paths compiles to DISTINCT coverage regions — a closure/branch covered on one path leaves the other's region uncovered → the coverage gate goes RED even though the logic is "obviously" tested. Enumerate every path that carries the new expression (grep the callers) and add one covering case per path. (Caught at #214 validate: the same `cell.hyperlink().map(|h| h.uri().to_string())` extraction went into both `term_to_styled_rows` [live grid] and `full_term_to_styled_rows` [command-finish]; the OSC-8 grid test covered only the former's closure → a finished-block test was needed for the latter.)

## PR-claude-passive-overlay-declines-mods-clears-at-choke-001
*severity: medium · prevents: BF-signature-passive-overlay-eats-keys-and-floats-001*

A PASSIVE editor overlay (one that does NOT own the keyboard — you keep typing through it, e.g. a signature-help card) must (a) consume ONLY a BARE variant of its nav keys — a bare ↑ cycles, but ⇧↑/⌘↑/⌥↑/⌃↑ belong to the editor (shift-select, word-nav, doc start/end), so match on keystroke.modifiers, not just keystroke.key, and DECLINE the modified variants so they fall through; and (b) clear itself at the SHARED dispatch_action choke (alongside the hover dismiss), excluding only the action that opens it — one site, not one inline clear per launcher — because a caret-based pump-poll misses an overlay-open that moves no caret, and the overlay's key arm runs BEFORE the palette/finder/history/find arms, so a stale one floats over them + eats their keys (the keyboard-dead class).

## PR-claude-per-keystroke-refetch-needs-a-generation-001
*severity: medium · prevents: BF-query-keyed-refetch-duplicates-same-string-001*

A per-keystroke server re-query whose stale guard is keyed ONLY on the query string cannot distinguish two concurrent in-flight fetches of the SAME string (type X → backspace → X while the first is slow) — both pass the guard and, if the results MERGE-append, duplicate the rows. Carry a monotonic generation bumped on every fan-out and key on (query, gen); an older fetch's gen mismatches and is dropped. This is what the completion sibling gets for free by keying on the buffer version. Also clear the live request on close so a reopened same-query session can't inherit an in-flight answer.

## PR-claude-per-pane-render-affordance-must-gate-on-is-focused-001
*severity: medium*

A visual affordance rendered INSIDE Marley's per-pane render loop (`for (pane_id, r) in &rect_list` with `let is_focused = pane_id == focused`) that is DRIVEN or ACCEPTED only on the focused pane — an inline autosuggest ghost, an active-input hint, a caret/cursor overlay, a focused-only highlight — MUST gate its RENDER on `&& is_focused`. Otherwise it renders on EVERY split pane while only the focused one can act on it → a render/accept divergence (you see a thing the key won't act on) + an off-reference look (Warp/fish show autosuggest only on the ACTIVE input). Marley has the precedent to mirror: the #186 find-highlight is explicitly focused-only (comment: "Scoped to the FOCUSED pane… would bleed… onto a non-focused pane"). When adding ANY focused-pane-only affordance to the pane loop: grep for `is_focused` (it's already bound in the loop) and add the gate. NOTE the self-review trap that let #200 through: reasoning only about SAME-FRAME / TIME divergence (does the render match the accept at the moment of the key?) misses the MULTI-PANE case (the render runs for every pane, the accept for one) — check both axes: does this show on non-focused panes, and can each shown instance be acted on?

## PR-claude-persist-clear-stale-companion-key-on-reset-001
*severity: medium*

When a piece of state is persisted across TWO keys — a current/primary key AND a legacy/companion key that is only WRITTEN in the non-empty (state-present) branch — the empty/reset branch MUST explicitly CLEAR the companion key too, not just the primary. Otherwise a stale companion resurrects the old state on the next restore, because the restore path reads the companion as a fallback. Concretely (#247): `persist_grid` wrote both the #163 `shell` and the legacy #205 `grid`, but its close-all branch wrote only an empty `shell` and left `grid` stale → a boot with an empty shell would resurrect the stale grid instead of showing the launcher; the fix cleared `grid` in the empty branch. Audit every persist path that writes >1 key: does the reset/empty branch zero ALL of them? Prefer a single source of truth; if a legacy companion must persist, clear it wherever the primary is cleared.

## PR-claude-persist-verify-trigger-not-just-codec-001
*severity: high*

For a "persist X across restart" feature, the unit-tested codec (serialize/parse round-trip) is NECESSARY but NOT SUFFICIENT — you must also verify the SAVE is actually TRIGGERED when X changes (grep for the persist call on the mutation path — e.g. the handler that creates/mutates X must call persist_grid/persist_*), AND confirm end-to-end with a driven quit→relaunch (or by reading the persisted file immediately after the change). A correct codec whose save is never invoked leaves the feature silently broken while every unit test + the codec pass green. Corollary: a lossy transform that re-indexes on BOTH save and restore must share ONE tested re-clamp fn on both sides.

## PR-claude-persisted-order-names-its-walk-001
*severity: medium · prevents: BF-claude-persisted-visual-order-came-from-id-sort-not-the-tree-001*

When persisting or replaying an ORDERED collection that has more than one plausible iteration order (a pane grid has id order via pane_ids() AND tree/DFS order via group().panes(); a registry has insertion AND key order), the codec must NAME the walk it uses and take it from the structure that defines the observable order (the layout tree for anything called "visual order"), not from a sorted id set that merely correlates with it early on. Check what the ADJACENT shipped codec walks (serialize_grid flattens the tree) and match it; a divergence between "usually identical" orders surfaces only after mid-structure inserts/closes, long past review.

## PR-claude-phantom-text-forks-start-and-end-boundary-maps-001
*severity: high · prevents: BF-claude-caret-map-used-for-code-span-end-swallows-phantom-001*

When virtual/phantom text (inlay hints, ghost text, folded regions) enters a 1:1 buffer→display column map, ONE column domain is necessary but NOT sufficient — the phantom also forks BOUNDARY SEMANTICS, and a single accessor can no longer serve both. Classify every consumer before shipping: a span describing a CARET RANGE (a selection band) must end at the caret map, so it stops exactly where the caret it follows renders — ending it elsewhere detaches the band from its own caret by the phantom's width. A span describing CODE (a syntax token, a diagnostic underline, a search hit) must end at a code-side map that stops BEFORE any phantom anchored at its one-past-end char — otherwise the phantom falls INSIDE the span and the token paints over its own hint. Build both maps in the same pass (`col_starts` + `col_ends`) and make each accessor's doc name which kind of consumer it serves. The tell that you have this bug: an end boundary computed as `col_of_offset(span.end)`, where `span.end` is exclusive and the map is defined to sit AFTER an anchored phantom. Related: the anchor rule is asymmetric — a mid-line phantom has its code to the RIGHT (emit, then record the column), an end-of-line phantom has its code to the LEFT (record the column, then emit); both are "the caret sits on the code side", and treating them uniformly parks the End caret out in the phantom.

## PR-claude-piecewise-branch-needs-distinguishing-input-per-branch-001
*severity: medium*

A pure fn with a piecewise/conditional branch needs a test input that reaches EACH branch with a DISTINGUISHING (non-degenerate) value — otherwise the rarely-hit branch's operator-mutants (`*`,`/`,`+`,`%`, comparisons) SURVIVE and MSI drops below 100. A degenerate input (0, or an identity value) that makes the branch compute the same result under the mutated op does NOT kill it. Verify the mutant set with `cargo mutants --list -f <file>` and pick an input per branch that yields a value the mutated op cannot reproduce. Worked example (#194, WCAG `relative_luminance`): the sRGB LOW branch `c/12.92` (for `c ≤ 0.03928`) is only reached non-degenerately by a very-dark, NON-black sample — pure black (`c=0`) gives 0 through `/`, `*`, or `%` alike, so it doesn't distinguish the mutants; an `hsla(0,0,0.02,1)` sample → 0.001548 kills `/`→`*` (0.2584) and `/`→`%` (0.02). General rule: for every branch, ask "what input lands here, and does its expected output differ under each op-mutant?"

## PR-claude-pin-word-class-and-merge-semantics-before-asserting-edges-001
*severity: low*

When writing an edge test for a word/line delete or motion, PIN the two semantics the assertion depends on BEFORE choosing the expected value: (1) the word class is `is_word_char` = alphanumeric ∨ `_`, so emoji AND punctuation STOP a word run (a `WordRight` over "aé😀 b" from 0 ends at CHAR 2 = "aé", not past the emoji) — this is the char-CLASS twin of the #336/#300 char-vs-BYTE discipline; (2) the multi-cursor range merge coalesces adjacent AND overlapping AND duplicate ranges (`s <= last.end`), so two carets on ADJACENT lines produce ONE merged span, not two. A "failing" edge assertion is as often a miscalibrated expectation as a real bug — check the class/merge rule first; in #303 both throwaway trips were the test's error and both, once corrected, CONFIRMED the code. Prefer a multibyte fixture whose word boundary and byte boundary DIFFER (e.g. "aébc": char-end 4, byte-end 5) so the assertion actually separates char from byte.

## PR-claude-pkill-cargo-mutants-orphans-temp-dirs-clean-before-rerun-001
*severity: low*

If you `pkill -9` a hung `cargo-mutants` run (e.g. to clear the #27 real-PTY-under-mutants stall), it leaves orphaned source-tree copies in the OS temp dir (`$TMPDIR/cargo-mutants-<project>-*.tmp`). The NEXT mutation run's parallel workers (`-j4`) then fail with `Worker thread failed: File exists (os error 17)` and cargo-mutants reports `did not complete a valid run (baseline failed)` — which the gate surfaces as gate:5 FAIL with NO "X caught / Y missed" summary and an empty `mutants.out/*.txt`. This is NOT a surviving mutant. Diagnose via `mutants.out/debug.log` (look for `os error 17` / the ERROR lines) — if the baseline shows `outcome=Success` but workers EEXIST, it's the temp collision. Fix: `rm -rf $TMPDIR/cargo-mutants-<project>-* mutants.out mutants.diff` then re-run. Confirm the code is actually clean with a targeted `cargo mutants --file <changed> --re <fn>` (fast) before re-running the whole gate. Rule: after force-killing cargo-mutants, always sweep its temp dirs before the next mutation run.

## PR-claude-place-drop-discriminators-before-the-last-survivor-001
*severity: medium*

When testing a survivor-mapping fix that replaces a bare bound (.min(last) / clamp-to-len), place the EXPECTED survivor strictly BEFORE the last survivor in every fixture: whenever the expected target IS the last survivor, the old bound and the new mapping coincide and the test passes against both — proving nothing. Derive each fixture by computing what the OLD code would produce and assert it differs. (#408: naive 3-project scenarios silently proved nothing; 4-entry scenarios with the target at kept-position 1 discriminated.)

## PR-claude-poll-driven-inflight-check-must-read-the-owner-001
*severity: medium · prevents: BF-claude-poll-driven-request-wedges-on-dropped-purpose-001*

A POLL-driven async request (one a pump re-evaluates every tick from state, not from an event) needs a send-skip to avoid a per-tick storm — and that skip must read the request-owner's own pending table, never a local "I sent this" latch. An EVENT-driven sibling (keystroke, dwell, click) is immune to a stale latch because each event mints a fresh key and the next send overwrites it; a poll-driven one keyed on unchanging state (a viewport, a selection) keeps matching the same key forever, so any path that abandons a request WITHOUT delivering a response — a timeout dropping its purpose, a disconnect clearing the table — wedges it permanently. Do not copy a sibling consumer's latch pattern without first asking whether its key changes on its own; that asymmetry is invisible in the diff. Two corollaries: (1) if a timeout comment promises "a slow server never wedges a consumer", a consumer that can wedge is a broken promise, not a new limitation; (2) a destructive `take()` of a server-sent refresh flag must not sit UPSTREAM of a skip that can swallow it — you will consume the exact signal that would have healed the wedge, and leave the cache empty while doing it.

## PR-claude-predicate-over-computed-collection-also-assert-size-or-contents-001
*severity: medium*

A uniqueness/predicate assertion over a COMPUTED collection does NOT kill a `-> empty`/`vec![]` mutant on the collection-builder fn: an empty collection is VACUOUSLY unique / sorted / and vacuously satisfies most `all()`/`any()`/predicate checks. So a test like `assert!(predicate(builder()))` line-covers `builder` but leaves its `-> vec![]` mutant alive → MSI < 100. ALSO assert the collection's SIZE (`builder().len() == N`) or specific CONTENTS (`builder().contains(<known elem>)`). Worked example (#197 keymap): `all_chords -> vec![]` survived `chords_unique(all_chords()) == true` (empty slice is vacuously unique) until `all_chords().contains(⌘⇧L)` + `.len() == 36` were added. Corollary (also #197): a `pub fn` in a crate-PRIVATE module used ONLY by tests trips `dead_code` under clippy `-D warnings` (test usage doesn't count for the lib target) → give it a real PRODUCTION use — a `debug_assert!` invariant self-check is ideal (the call survives cfg-stripping so it's live in every profile) — NEVER `#[allow(dead_code)]` (§0 bars suppressions).

## PR-claude-prefer-safe-equivalent-over-justifying-unsafe-001
*severity: medium*

When an `unsafe` block trips MULTIPLE gates (a SAST/source-ban finding, the miri gate, a clippy lint), STOP and ask whether a SAFE equivalent achieves the same effect before you justify the unsafe (a `// SAFETY:` comment) or exempt the crate (a miri-exempt/allowlist). Removing the unsafe fixes every unsafe-triggered gate AT ONCE and is the §0-"fix at source, don't exempt" answer. TICKET-348 used `unsafe { ManuallyDrop::drop(&mut self.pty) }` to conditionally skip a field's Drop; that ONE unsafe tripped the SAST (which greps for `unsafe` without a same-line `// SAFETY:`) AND pulled the crate into the miri gate (a new `unsafe` silently makes the crate miri-checked — and a real-I/O crate whose tests spawn processes CANNOT run under miri). The safe equivalent — make the field `Option<T>` and, on the skip path, `std::mem::forget(self.field.take())` (take() moves the value out of the Option so mem::forget, a SAFE function, can skip its Drop) — did the identical job with zero unsafe, clearing both gates and keeping the crate's "unsafe-free" invariant. General shape: to conditionally NOT run a value's Drop from inside `Drop::drop(&mut self)`, prefer `Option<T>` + `mem::forget(take())` over `ManuallyDrop<T>` + `unsafe ManuallyDrop::drop`.

## PR-claude-preseed-shell-blob-to-drive-capture-when-input-blocked-001
*severity: low*

When validating a Marley RENDER change and synthetic input is environment-blocked (CGEvents don't land on the bare `target/debug/marley` — needs Accessibility perm + a real .app), do NOT give up on the driven capture — PRE-SEED an isolated config so the app RESTORES the target state on boot, then capture. Recipe: `mkdir -p $ISO/.marley/config`, write `settings.toml` with `[workspace]\nshell = "<blob>"` where the blob is the shell codec's serialize_shell format `<active_project>\n<root>\t<active_tab>\t<entry>…` (entries: `T=<grid>` terminal, `C=<key>` cockpit, `V=<path>` editor; a split terminal = `T=H:t,t` [horizontal 2-terminal] or `T=V:t,t`; an untitled terminal is just `T=<grid>`, a titled one `T=<title>\x1f<grid>`). The root must be a REAL dir (cwd/PTY restore). Launch `HOME=$ISO target/debug/marley` (HOME is the config-dir lever — marley_config_dir = $HOME/.marley/config), wait, then `screencapture -l<winid>` its window and READ the PNG. Capture-via-CGWindowList is NOT blocked even when input is. This gives a REAL end-to-end render validation (the app restores + paints the exact state) without any synthetic click/keystroke. Never write into a real user's ~/.marley (isolate via HOME); kill only your own pid (never `pkill -x marley` if the user has an instance running).

## PR-claude-presence-grep-must-prove-the-hit-is-the-defect-001
*severity: medium*

A PRESENCE-grep must prove the HIT IS THE DEFECT, not a citation of it — the mirror of "an absence-grep must prove the command ran".

Verifying a fix by `grep <the defective string> | wc -l` and demanding 0 is wrong whenever the correct fix RECORDS what it replaced. A good corrective doc often quotes the old claim ("this previously promised X; it was never true, see #NNN") precisely so the next reader learns why the line is gone — and that quotation matches the grep. The check then reports a regression that does not exist, and the natural next move (delete the "offending" line) destroys the most valuable part of the fix.

Discovered immediately by applying PR-claude-doc-only-fix-has-no-verifier-reread-it-001 to its own ticket (M22 #337): `grep -rc "WITH a flash" crates/` returned 1 — settings.rs:48, inside the corrective doc quoting the promise it had just removed.

THE RULE: for a presence-grep, `-c`/`wc -l` is never the answer. Print the hits with `-n` and READ each one, classifying it as THE DEFECT or A CITATION OF THE DEFECT. Both directions of the grep family now have the same shape: an absence-grep's 0 is meaningless unless the command ran (PR-claude-grep-for-absence-must-prove-the-command-ran); a presence-grep's N is meaningless unless you read what matched. COUNT IS NOT EVIDENCE. The corollary for writing: keep quoting the old wrong claim in the corrective doc — it is worth more than grep-cleanliness — and expect the grep to hit it.

## PR-claude-progress-gate-retry-sleeps-and-measure-hot-loops-at-n-entities-001
*severity: high · prevents: BF-claude-pump-idle-sleep-floor-times-panes-frame-killer-001*

A retry/backoff sleep inside a poll-style function MUST be gated on PROGRESS (something was read/written THIS call), never unconditional: an unconditional sleep floor becomes a fixed per-entity cost that a per-frame loop multiplies by entity count (one idle PTY pump = ~12ms; 8 panes = 98ms/16ms tick — measured). When a change turns a single-entity hot path into an N-entity loop (pump-all, render-all, poll-all), MEASURE the loop at realistic N with real resources BEFORE shipping — an inspect critic timing 1000 iterations catches in minutes what code review missed. Corollary: teardown that can block (child reap, blocking close) never runs on the UI thread — return the owned state to the caller and drop it on a reaper thread (make the handle type Send).

## PR-claude-project-rich-row-through-descriptor-event-honor-terminal-state-001
*severity: high*

When projecting a rich wire row (carrying a full descriptor + a triggering kind) onto a NARROWER internal event vocabulary, route every descriptor-bearing row through the event that carries the descriptor (e.g. an Upsert that sets title/labels/state together), NOT through a label-less state/terminal event that silently drops the descriptor. Specifically: (1) a terminal event (session-end/close) MUST honor the row's terminal STATE — a row that ended in error projects to the retained-error terminal, never a clean "done" (laundering a crash to a clean exit is a safety-critical invariant break). (2) A "raise a sub-state" event (question/prompt) that carries no descriptor must be preceded by a descriptor Upsert so a first-seen-via-that-event entity still renders its identity/chips. (3) A degrade for a malformed sub-payload must not overshoot the row's OWN state signal (a malformed question on an at-menu row should still surface the Waiting block, not fall back to a no-op heartbeat that hides it). Confirmed on #368 marley_forge_client: session-end+error→Done, halted-drops-descriptor, and malformed-halt-hides-block were all one root cause — the projection returned single non-Upsert events; the fix returns Vec and routes descriptor-bearing kinds through Upsert.

## PR-claude-public-doc-no-link-to-private-item-001
*severity: medium*

A `pub` item's doc comment must NOT intra-doc-link (`[`Name`]`) to a PRIVATE item — rustdoc emits `private_intra_doc_links`, which `-D warnings` (gate:14) turns into an error. Reference a private const/fn as plain inline code (`` `MAX_FRAME_LEN` ``) or inline its value, not as a `[link]`. Verify with `cargo doc --no-deps -p <crate>` before the gate. (Also in gate:14: no `Zed`/`Warp` brand mention in source outside `docs/*_architecture/` — the brand-scrub; reword to a generic term.)

## PR-claude-pump-materialize-hot-data-only-when-consumed-001
*severity: medium*

A per-frame pump that collects state for a downstream consumer must MATERIALIZE expensive data (a full-buffer `text()` clone, a large serialization) only when the consumer will actually use it — not unconditionally every tick. Gate the expensive clone behind a cheap immutable query of the consumer's state (e.g. `host.needs_text(path, version)` comparing a last-synced version) so an idle frame copies nothing. Bit #309: the LSP pump cloned every open buffer's whole text (up to the 2MB viewer cap) every 16ms even with no Ready server and no edit — reconcile only used it on a version change. Also: feed the consumer the FULL set it is designed for (all open files, not just the active one) or its diff machinery (didClose-of-absent) churns on every selection change.

## PR-claude-pump-state-change-must-set-dirty-to-repaint-001
*severity: medium*

In Marley's gpui live-pump loop, the frame repaints ONLY when the local `dirty` flag is set before the `if dirty { cx.notify(); }` gate. Therefore ANY pump-side mutation of state that changes what renders MUST set `dirty = true` (or it won't repaint on an otherwise-idle frame — no PTY output that tick — and the change is invisible until an unrelated repaint). This is asymmetric-bug-prone: a SET path often rides existing dirty-making events (e.g. the pump events from a finishing command) while the paired CLEAR/decay path does not. Mirror the existing `status_flash.tick()` pattern, which sets `dirty = true` on its state change. When adding pump-side state that affects render (badges, flashes, status maps), grep the block for `dirty = true` and ensure every mutating branch sets it. Bit #203: the tab-completion badge's clear-on-view removed the map entry without `dirty`, so a viewed badge lingered on an idle frame.

## PR-claude-raw-input-passthrough-must-filter-platform-chords-001
*severity: medium*

When you add a NEW input path that passes keystrokes THROUGH to a child (raw-mode PTY streaming, an embedded editor, a webview), it must filter the SAME OS-level modifier chords the existing (cooked/local) path filters — especially the platform/⌘ (and ctrl where the app owns chords). A new passthrough that only checks the modifiers it cares about (ctrl/alt) lets unbound app-level chords (⌘C/⌘V/⌘Q) leak to the child as bare characters. Mirror the established filter (here: cooked `key_from_keystroke` maps control||platform → a no-op) at the TOP of the new mapper: `if keystroke.modifiers.platform { return None; }`. General rule: two input routes to the same surface must agree on which modifier chords are the app's vs the child's — diff the guard sets when adding the second route. These leaks hide in mutants::skip'd shim code, so they're an inspect/headed-review catch, not a gate catch.

## PR-claude-read-to-the-end-of-the-constructor-before-asserting-post-state-001
*severity: medium*

A struct literal is not the constructor's post-state. Before asserting "what the object IS when the function returns" — especially in a doc comment or a safety argument a later reader will trust — read to the END of the constructor, not to the first field assignment that confirms your hypothesis. Concretely: `LspHost::new` sets `life: None` in its struct literal and then, still inside `new`, runs `if resolved.is_some() { host.life = Some(...); host.execute(boot) }`, driving the host to `Phase::Initializing` before it returns. A claim that "a freshly-created host is `life: None`" read only the literal and was wrong for every host that actually spawns. The correct invariant was reachable but different (Ready is unreachable from `new` because Ready only comes from `on_message` ← `drain()`), so the CONCLUSION survived while the stated MECHANISM was false — the batch's recurring "right answer, wrong reason" class. General move: when documenting a returned object's state, grep the whole `fn new`/builder body for post-literal mutation (`self.x =`, `.execute(`, `.push(`, an `if`/`match` that reassigns) before writing the sentence.

## PR-claude-real-pty-flake-under-mutants-baseline-not-code-001
*severity: low*

When a FULL gate hangs (SLOW >200s) in the coverage or mutation step on a real-PTY / real-subprocess integration test, but that test passes in isolation (<1s) AND the same gate config passed on prior tickets, suspect ENVIRONMENTAL scheduling/PTY contention on a machine loaded by a long session — NOT a code defect. Diagnose fast: (1) run the named test alone; (2) run the crate's real-PTY suite under the exact runner that hung (`cargo test` = threaded, ONE process, parallel — cargo-mutants' baseline mode; vs `cargo nextest`/`llvm-cov nextest` = one PROCESS PER test, which isolates PTYs and rarely flakes); (3) scan for shells orphaned to launchd (PPID 1) from earlier killed gate runs. Fix by killing accumulated runners (`pkill -9 -f 'cargo-mutants|llvm-cov|nextest'`) + confirming the PTY suite runs fast, then re-run — do NOT edit the test or weaken the gate for a transient. A pure DELETION diff makes it unambiguous: `mutation: no mutable lines in the diff — pass`, so any mutation-gate hang there is 100% infra.

## PR-claude-recanonicalize-after-clamping-into-an-invariant-type-001
*severity: high · prevents: BF-claude-clamp-into-invariant-type-without-recanonicalizing-001*

A defensive CLAMP is not injective — it maps many distinct inputs onto the same boundary value. So whenever you clamp (or saturate, or truncate, or round) the members of a collection that carries a uniqueness/disjointness/ordering invariant, you MUST re-run the canonicalizing constructor afterwards, never the raw in-crate builder. Clamping two out-of-bounds cursors both onto EOF turns two members into DUPLICATES, and a downstream "apply to every member" op then applies the operation TWICE at one logical position. Corollary: an invariant that is only DOCUMENTED is fiction — enforce it at every entry point (including the empty/degenerate input) or delete the claim. If a fn's doc says "never empty" / "always disjoint", grep every constructor and every mutator and prove it; a sibling fn that defends and one that doesn't is the tell that the invariant is aspirational.

## PR-claude-receipt-binds-gate-definition-001
*severity: high · prevents: BF-commit-receipt-scope-001*

A commit-gate receipt must fingerprint the gate-DEFINING files (the gate script, the enforcement hooks, supply-chain config like deny.toml/.gitleaks.toml, and the dependency manifests + lockfile), not only the application source. If the fingerprint covers only app code, the gate itself can be weakened (a lowered floor, an added exclusion, an edited hook) and committed ungated — pre-staging a future false green. Bind the bar's definition into the same receipt that binds the code it judges.

## PR-claude-recheck-mutants-skip-after-refactoring-shims-001
*severity: medium*

After ANY structural edit around a `#[cfg_attr(test, mutants::skip)]`-masked shim — extracting a helper out of it, inserting a fn above it, moving/splitting it — RE-RUN `cargo mutants --list -f <file>` and grep the shim fn NAMES to confirm EACH is still absent (masked). The skip attribute binds to the fn directly beneath it; an extract-and-relocate can move the attr onto the new helper while leaving the original unmasked, and an insert can rebind it to the wrong fn. In a crate with no global mutation exclude (per-fn skips only, e.g. marley_app), a detached skip leaves a fs-IO/gpui shim with a viable 'replace body with ()' mutant that no unit test can kill → gate:5 (MSI 100) RED. The `--list` grep is the cheap deterministic check; do it before handing to validate. (Prevents BF-claude-mutants-skip-detached-by-refactor-extract; complements the insert-above variant in [[mutants-skip-detach-trap]].)

## PR-claude-recheck-the-parsed-value-the-consumer-uses-001
*severity: high · prevents: BF-claude-wall-admit-vs-parsed-host-divergence-001*

When a security gate validates a string with a HAND-ROLLED parse (prefix strip, split_once, rsplit) and a downstream consumer re-parses the same string with a REAL parser (WHATWG url, an HTTP client, a browser/webview), the gate's admit decision is untrustworthy for the consumer: the two parsers can disagree on which bytes are the host/path/port (e.g. last-`:` port tail vs last-`@` userinfo split), so a hostile input can be admitted under one reading and consumed under another. Always re-assert the security property (loopback-ness, allowed host, scheme) on the PARSED value the consumer actually uses — the check must sit on the same side of the parse as the consumption. Proven by BF-claude-wall-admit-vs-parsed-host-divergence-001 (#404: wall-admitted `http://127.0.0.1:pass@evil.com/` derived a non-loopback browser origin until web_origin_of re-checked the parsed host).

## PR-claude-redundant-arithmetic-is-an-equivalent-mutant-001
*severity: medium*

A redundant arithmetic offset that appends framing bytes invisible to downstream parsing is an EQUIVALENT mutant — no test can kill it, so it silently blocks MSI 100. Example: `s[..idx+1]` where `idx = rfind("\n\n")` includes the boundary's first `\n`, which `.lines()` ignores and `trim()` skips — so `[..idx+1]` and `[..idx]` produce byte-identical parse results, and cargo-mutants' `+`→`*`/`+`→`-` on the `+1` survives every possible test. The fix is NOT to fight the mutant with an ever-more-elaborate test; it is to SIMPLIFY the redundant code (`[..idx+1]` → `[..idx]`), which removes the mutation site by construction. Procedure: when a cargo-mutants target refuses to die, first prove it is genuinely equivalent (do BOTH forms yield identical observable output for every input?) before writing more tests — an equivalent mutant means redundant/dead arithmetic to simplify, not a missing test. (Distinct from a merely-hard-to-kill mutant, which DOES have a killing test — e.g. the sibling `idx+2` remainder offset is killable by asserting the returned remainder.) Confirmed on #368; a mutation-readiness critic flagged it before Phase 4 burned time on an unkillable target.

## PR-claude-redundant-guard-before-saturating-cast-is-equivalent-mutant-001
*severity: medium*

A defensive `.max(0.0)`/`.min(...)`/clamp that guards a value which a LATER operation already handles (e.g. Rust's `as usize`/`as u32` float cast saturates negatives→0 and NaN→0) is an EQUIVALENT-MUTANT magnet: cargo-mutants can delete the redundant guard with no behavioral change, so no test kills it → MSI < 100 → gate RED. Two fixes: (a) drop the redundant guard and document the downstream operation's guarantee (preferred when the cast/op genuinely covers it), or (b) make the guard load-bearing by branching so a test can pin the boundary. Same for a `<=`/`>=` guard whose only distinguishing input (a negative, when the domain is non-negative) is untested — either add the boundary test or accept it as a justified equivalent at the gate. Spotted in M12 #179 (bottom_anchored_row: `.max(0.0)` before `as usize`, and `cell_h <= 0.0` with no negative test) — both pre-empted from a critic's dry-run flag, saving a mutation-RED round.

## PR-claude-reference-app-citation-stays-in-docs-not-source-comments-001
*severity: high · prevents: BF-zed-brand-word-in-source-comment-001*

A design note that CITES a reference app (Zed/Warp) belongs in docs/ (the spec/notes), NOT in a crates/ source comment — gate:14 brand-scrub greps `warp|zed` (-i -w) across crates/**/*.rs and hard-fails the commit. When copying a rationale sentence from the spec into a code comment, PARAPHRASE the brand away (e.g. "a decoupling discipline: git state lives in its own store" not "Zed's decoupling warning adopted"). Before the gate, run `grep -rniwE 'warp|zed' <your changed crates files>` — it must be empty. This has now bitten twice (#327 "Zed chord", #328 "Zed's decoupling") — make the paraphrase reflex automatic when a §20 reference is fresh in mind.

## PR-claude-registry-release-returning-a-resource-must-be-must-use-001
*severity: low*

A registry/pool method whose return value IS a resource that must be routed somewhere specific for teardown — e.g. `release_view(id) -> Option<Content>` returns the owned content on the last drop so the caller can reap it OFF-THREAD (a main-thread PTY/socket drop blocks/freezes the UI) — must be `#[must_use = "<what to do with it>"]`. Without it, a caller writing `reg.release_view(id);` (result ignored) compiles and drops the resource INLINE on the calling thread, silently defeating the off-thread-teardown contract the whole refcount design exists for. Same for an acquire/try method returning `Option<()>`/bool success — a dropped result hides a stale-id/failed-acquire no-op. Add the `#[must_use]` when you write the method (before any consumer exists), so the compiler enforces the contract the moment the consumer is wired. Cheap, no runtime cost, breaks nothing (no callers yet).

## PR-claude-regression-guard-must-exercise-the-claimed-path-001
*severity: medium*

A regression guard for a SPECIFIC code path must EXERCISE that path, not merely reproduce its output — a fixture whose inputs route through a DIFFERENT branch can pass while proving nothing about the intended mechanism. When you write a test to guard "mechanism M does X", trace the fixture through the real code and confirm it actually reaches M's branch. #358: a "cursors on separate rows all survive the dedent" guard used carets at offsets [4,11,18] on `"    aa\n    bb\n    cc\n"`, believing they were "one char into each indent" — but offset 4 is the first LETTER (col 4, past the 4-space indent [0,4)), so those carets rebased via `rebase_through`'s DELTA path (p+delta), NOT the in-indent CLAMP path (`p < at+remove → at`) that the ticket is actually about. The assertion `[(0,0),(3,3),(6,6)]` was still correct and the test passed, but it never touched the clamp arm. Fix: move the carets INSIDE the indent (offsets [1,8,15], col 1) → the SAME assertion but now genuinely via the clamp — a strictly better guard. The tell: if you can't state which branch each fixture input hits, you don't yet know the test exercises what you think.

## PR-claude-relax-nonempty-invariant-audit-all-accessors-001
*severity: high*

When relaxing an "always ≥1" invariant so the EMPTY state becomes reachable (e.g. the last workspace/tab/project can now close), the main render branch is NOT enough. Audit EVERY accessor of the active/first element — `active_project()`/`workspace()`/`self.items[self.active]` — across FOUR off-render-path surfaces that each panic on the empty collection: (a) background loops — a PTY pump / timer / poll spawned via cx.spawn that derefs the active element every frame, independent of render; (b) the PERSIST path — close→persist often calls the active-element accessor unconditionally, firing synchronously in the click handler BEFORE the empty state even renders; (c) the render PROLOGUE — code above the empty-state branch (find snapshots, geometry) that touches the active element; (d) key/action dispatch installed off the render tree. Place the empty-state branch at the VERY TOP of render (before any prologue access) and add a `count==0` guard to every background/persist accessor. Verify with a dedicated panic-safety critic AND a driven run — unit tests can't (these are usually mutants::skip cov-excluded shims). Bit #234 (launcher): the render-branch alone left 3 reachable panics.

## PR-claude-relocated-tmpdir-changes-every-tempdirs-ancestry-001
*severity: high · prevents: BF-claude-in-repo-tmpdir-breaks-repo-ancestry-tests-001*

Before pointing TMPDIR (or any tempdir root) INSIDE a repository, check what the new ancestry breaks: every tempfile::tempdir() in every test now has the repo's .git (and its Cargo.toml, workspace files, gitignores) as ancestors, so any walk-up discovery test (nearest-.git, workspace-root, config-file search) silently flips its premise. Prefer a NAMED subdirectory of the system tmp (deterministic, trap-cleanable by exact name) — it gains the same cleanup properties without changing any test's parent chain. Checking copy-recursion alone is not enough; ancestry is the second consequence.

## PR-claude-relocating-a-guard-to-a-hot-path-reorder-cheapest-first-001
*severity: medium · prevents: BF-claude-gate-moved-to-a-per-tick-path-kept-its-syscall-first-001*

When you MOVE an existing guard from a cold path (a user gesture, a boot step, an error branch) to a hot one (a render frame, a pump tick, a poll loop), its clause ORDER stops being a style question and becomes a cost decision — re-derive it, do not carry it over. Rank the clauses by what they cost and how often they short-circuit: in-memory map/flag lookups first (especially any clause that becomes permanently true and retires the check), then in-memory scans over small collections, then syscalls (stat/read/env) LAST. A clause that was free to evaluate once per gesture can become a syscall per frame with no test, no coverage signal and no mutant to catch it — the gate suite is silent on cost. Two follow-on questions to answer explicitly in a comment at the site: (1) can the expensive clause be hoisted out of the loop entirely, or does re-evaluating it deliver a feature? (if it does — e.g. noticing a file that appears later — say so, because the next reader will otherwise "optimize" it out); (2) is there a cheap clause that makes the expensive one unreachable in the common case? Put that one first.

## PR-claude-remap-index-keyed-state-on-collection-remove-001
*severity: medium*

Any shim/view state keyed on a Vec/collection INDEX — a HashSet<usize>, an Option<usize> cursor, a HashMap<usize,_>, a "selected/renaming/collapsed index" — MUST be remapped when the underlying Vec removes an element (which shifts every later index down by one). A raw stored index does NOT vanish on removal; it silently ALIASES a different (live) element. On a `Vec::remove(idx)`: drop any stored index == idx, and decrement every stored index > idx. Grep the close/remove handler for the sibling guards already there (e.g. `renaming_tab = None`, `adjust_active`) and mirror them for the new state. Reading via `contains()`/`.get()` masks the bug (it's inert on a stale index) — the corruption is on the WRITE/shift side. Bit #236 (collapsed_projects not remapped on close_project).

## PR-claude-rename-sweeps-intradoc-links-001
*severity: medium*

When you rename a Rust item (method/type/fn), grep the crate for intra-doc links to the OLD name — `[`old`](Self::old)`, `[`Type::old`]`, `[crate::old]` — and update them. rustdoc under `-D warnings` (the docs gate) turns a now-unresolved link into a HARD compile-doc failure, and it only fires on pub-exported items, so it can hide until the gate runs. The rename's own compile + tests + clippy all pass green (the doc string isn't type-checked), so nothing but rustdoc catches it. Fast local check: `RUSTDOCFLAGS="-D warnings" cargo doc -p <crate> --no-deps`.

## PR-claude-render-memo-ranges-must-refresh-in-edit-handlers-001
*severity: high*

A render-path memo of buffer RANGES (find matches, diagnostics, links) is safe for HIGHLIGHTS but not for EDITS: key handlers run per event while the memo refreshes per frame, and two events between frames (autorepeat; gpui's test lane dispatches all keystrokes then parks once) hand the second edit PRE-EDIT offsets — ropey panics out-of-range or silently replaces the wrong text. Any handler that FEEDS memoized ranges into Buffer::edit (or line_col/navigation) must call the memo refresh at its own top; key the memo (identity-nonce, version, inputs) so the refresh no-ops when fresh. Also re-verify mutants::skip attachment with `cargo mutants --list` after inserting ANY fn above a skipped one — strike 5 of the detach trap landed exactly this way (the new fn's doc+attr slid between the old doc+attr and its fn).

## PR-claude-render-transform-and-inverse-hit-test-change-in-lockstep-001
*severity: high*

A rendering TRANSFORM (how content is positioned: a vertical anchor, a scroll offset, a viewport window, a zoom) and its INVERSE (the hit-test that maps a pointer/coordinate back to a content cell — mouse selection, click-to-focus, scroll-to-row) are a COUPLED PAIR. Changing one without the other silently breaks input mapping. When a ticket changes a render-geometry property (here: top-anchored → bottom-anchored via justify_end), inspect MUST enumerate every consumer of the inverse mapping (pane_grid_pos and its callers: mouse-down/move selection, any coordinate→row/col fn) and confirm each is updated in lockstep. These bugs are especially dangerous because both the render and the hit-test are typically untested shim (mutants::skip + coverage-excluded), so no gate catches the desync — and the failure is silent (wrong selection, not a crash). Mitigation: extract the coordinate math into a pure tested fn (here row_at) so at least the arithmetic is cov/MSI-guarded, and add a design checklist item "if this changes how content is positioned, what reads positions back?".

## PR-claude-resolve-against-same-root-that-built-stored-path-001
*severity: medium*

When comparing a freshly-resolved path against a STORED already-resolved path (e.g. matching a terminal file-ref to the open editor file), resolve the fresh path with the IDENTICAL root/base that produced the stored path — not a parallel field that "should be the same." A stored absolute path built by resolve_under_root(A, rel) only compares equal to a fresh resolve_under_root(B, rel) when A==B; two roots kept-in-sync but distinct (e.g. self.project_root vs active_project().root) can diverge on a sync-timing edge and silently drop the match. Trace where the stored path was constructed and reuse that exact root. Caught in #289 inspect (F1).

## PR-claude-restore-is-a-second-constructor-001
*severity: high*

When adding a per-item field that must be SEEDED at creation (a disk snapshot, a nonce, a watcher handle), enumerate EVERY constructor path before closing implement: the interactive open path, the boot/deserialize RESTORE path, duplication/split paths, and any from_* bulk builder. Restore is a second constructor — it rebuilds the same items without running the interactive seeding code, and an unseeded sentinel (None/default) that maps to a silent no-op turns the whole feature off precisely for the steady-state restored working set (Marley #275: from_files left disk:None → every external-change choke Noop'd after any restart; both critics rated it the ticket's headline guarantee voided). Grep for the type's constructor call sites (OpenFile::new / from_files) and seed or thread the field through each.

## PR-claude-resume-reverify-fmt-and-gate-001
*severity: medium*

After an interrupted session (reboot/crash/OOM), a prior phase's "cargo fmt / gate green" claim in the notes is STALE — a partial write can leave a new file unformatted (or worse) while it still COMPILES and passes `cargo test`, so it hides until the gate's `cargo fmt --check`. On resume, re-run `cargo fmt --all -- --check` and re-establish the gate BEFORE trusting the working tree or the notes' phase claims.

## PR-claude-retest-env-boundaries-before-scoping-validation-001
*severity: medium*

RETEST environment boundaries before designing around them: a documented "X is blocked/broken in this env" claim from a PAST session must be re-verified with a 2-minute live probe before any new ticket scopes its validation down (e.g. "typed paths can't be driven"). The Marley harness memory has now caught THREE separate sessions re-asserting a dead "synthetic typing doesn't register" boundary that a single retest disproved each time — the usual real causes were frontmost loss between shell calls or keys landing in an overlay, both fixable in the drive invocation (ONE call: focus + verbs). Cost of skipping the probe: tickets get weaker validation plans than the env actually supports.

## PR-claude-retry-keyed-on-empty-cache-needs-every-outcome-to-write-001
*severity: medium · prevents: BF-claude-error-reply-writes-no-cache-so-request-loops-forever-001*

A retry loop whose condition is "no cached answer covers what I need" turns every non-writing outcome into an infinite loop. Enumerate the terminal outcomes: a success caches and stops, an empty success must ALSO cache (an empty answer is an answer) — but an error typically returns early and writes nothing, so it re-fires per round-trip forever. Either record a negative result, or make the guaranteed-error cases unreachable by gating the request up front. Gate on the SAME authority the rest of the subsystem uses (for LSP: `language_id_for`, the one table document-sync syncs by — a file it declines was never opened, so asking about it can only error) plus the server's advertised capability. A capability-reader function that exists, is exported, and is called nowhere is not harmless dead code — it is a claim the code does not keep; wire it or delete it. Bound any derived range to the real extent too (a 20-line file has no rows 0..69).

## PR-claude-reversed-literal-range-trips-clippy-build-values-001
*severity: low · prevents: BF-new-untracked-file-skipped-by-in-diff-mutation-001*

A test that intentionally exercises a REVERSED/degenerate range (e.g. `enclosing_ranges(&s, 5..2)` to hit an empty/None totality arm) must NOT write the reversed range as a literal `5..2` — clippy's `reversed_empty_ranges` lint fires on literal reversed ranges and, under `-D warnings`, is a hard BUILD error that cascades (breaking coverage + mutation for that crate too). Build it from values: `let (start, end) = (5usize, 2usize); f(start..end)`. The lint only fires on literals it can const-evaluate.

## PR-claude-run-the-gates-exact-command-never-an-approximation-001
*severity: high · prevents: BF-claude-approximated-the-gate-command-and-reported-green-001*

Never report a gate as green from a command you composed yourself. Run the literal command scripts/gates.sh runs (read the line), or run gates.sh. Two specific traps this closes: (a) dropping `-D warnings` turns hard errors into warnings you then filter away — and grep patterns guessed from memory miss rustc's actual wording (dead code is "warning: method `X` is never used", not "warning: unused"); (b) `cargo check --all-targets` proves test code COMPILES, never that it PASSES — a test whose expectation your change invalidated stays red and silent until `cargo test` runs. Before writing "gate green" or "clippy clean" into any phase record, paste the real output of the real command.

## PR-claude-sanitizer-must-loop-until-stable-single-replace-reconstitutes-001
*severity: high*

A single `str::replace(needle, "")` (or one regex pass) is NOT a safe sanitizer when removing a delimiter/marker/token from untrusted input: `replace` does one non-overlapping left-to-right pass and never re-scans its output, so removing one occurrence can SPLICE the surrounding bytes into a fresh occurrence — `AB` + `<needle>` + `CD` where `AB+CD == <needle>` reconstitutes it. For a security guard (stripping a bracketed-paste `ESC[201~` end-marker, a comment close, a quote, a path separator), this is bypassable. Rule: strip in a LOOP until the needle no longer occurs — `let mut s = input.to_string(); while s.contains(needle) { s = s.replace(needle, ""); }` (each pass strictly shrinks → terminates), or use a single streaming scan that removes matches from a growing output that can't re-form them. AND: MSI cannot protect this — cargo-mutants often generates only whole-function-return mutants for a `Vec<u8>`/`String` builder and NONE on the `.replace` call, so 100% MSI stays green even if you delete the guard entirely; you MUST add a hand-written split-token regression golden (feed the reconstituting input, assert no interior marker survives) — a structural test, not a mutation-forced one. This is the sanitizer cousin of PR-invariant-invisible-logic-needs-structural-tests.

## PR-claude-saturating-sub-instead-of-len-guard-001
*severity: medium · prevents: BF-equivalent-guard-mutant-blocks-msi-100-001*

Under an MSI-100 floor, prefer `let n = len.saturating_sub(k); drain(0..n)` over `if len > k { drain(0..len - k) }`. The guard form generates an EQUIVALENT (unkillable) mutant: `>` vs `>=` differ only at `len == k`, where both drain `0..0` — a no-op — so no test can distinguish them and MSI can never reach 100 without restructuring. It also spawns `-`→`+`/`-`→`/` arithmetic mutants (the `/` one needs a ≥5-element case at k=2, since `3-2 == 3/2`). `saturating_sub` is a METHOD CALL, which cargo-mutants leaves unmutated, so the whole guard+arithmetic cluster vanishes while behavior is identical. General rule: when a guard exists only to make an arithmetic op safe, use the saturating/checked method instead — smaller, fully-killable mutation surface.

## PR-claude-scanning-clip-loops-breed-equivalent-mutants-distribute-instead-001
*severity: medium*

A per-line scanning clip loop (persistent skip cursor + crossing-span break) is an equivalent-mutant factory: the skip is pure optimization and the break is semantically dead under a disjoint-input invariant, so their comparison mutants are output-identical (probe-proven 0-diff) and MSI 100 becomes unattainable. Restructure interval-clipping as PER-SPAN DISTRIBUTION: first/last touched lines via slice::partition_point (method calls — unmutated), saturating_sub for totality, min/max intersection, and ONE guard (cs < ce) whose <= mutant is killed by an empty-intersection fixture (a span crossing an EMPTY line). Also expose multi-stage pipelines (sweep → clip) as separate fns so each stage's edge behavior is observable — an empty span admitted by a sweep mutant is invisible after clipping but test-visible at the sweep seam.

## PR-claude-scroll-clamp-must-track-the-render-line-count-source-001
*severity: medium*

When you change the line/row COUNT source a view renders over (e.g. from a frozen pre-rendered `cv.lines` to a live `buffer.len_lines()`), update the SCROLL/clamp handler to read the SAME source in the same commit. A scroll handler that clamps against a stale/different count silently mis-clamps: the bottom rows become unreachable, and the bug is invisible until the two counts diverge (a different line-break model, or the underlying model being edited). Grep every `scroll_code`/`visible_range`/clamp site that pairs with the changed render and re-point it.

## PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001
*severity: medium*

When adding a SECOND consumer to a shared async request/response path, diff it against the first consumer's guards and justify every one you omit — the first consumer's guards were bought by a bug, and the shared path does not carry them for you. In Marley #312 (definition) vs #311 (hover) on the same LspHost path: hover dropped a response whose focused file no longer matched key.uri; definition kept only the stale-KEY guard and acted unconditionally. A stale-key guard proves "no NEWER request superseded this" — it does NOT prove the user is still looking at the file they asked about. Consequences of the omission: a slow answer (10s timeout; rust-analyzer takes seconds while indexing) YANKS the user out of whatever they moved on to; and if the user is on a non-editor tab, code that reads active_editor() to record a return position silently records NOTHING, so the jump lands with no way back. Rule of thumb: any consumer whose action MOVES THE USER needs a liveness guard on the live identity (path/uri), not just a freshness guard on the request. Related: PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001 (#311 F1) is the same lesson for the overlay half.

## PR-claude-sed-range-prestrip-same-line-before-multiline-delete-001
*severity: medium*

When stripping paired delimiters (HTML comments `<!-- -->`, block comments, heredoc-ish fences) from text in shell to test "is there real content", a `sed '/OPEN/,/CLOSE/d'` RANGE cannot delete a delimiter that OPENS and CLOSES on the SAME line — POSIX sed only tests the closing address on lines AFTER the opener, so a one-line `<!-- x -->` starts a range that runs to the next CLOSE (or EOF), swallowing legit following content → a false-positive "empty" verdict (in a commit hook: a false-BLOCK). ALWAYS pre-strip same-line pairs first, then range-delete multi-line blocks: `sed 's/<!--.*-->//g' | sed '/<!--/,/-->/d'`. This is inherent sed semantics (BSD == GNU), so it reproduces on the macOS runner. Test the collapsed-to-one-line comment case explicitly — a multi-line-comment-only smoke passes while the same-line variant silently breaks (bit #248's enforce-warp-reference.sh; the inspect critic caught it, the author's multi-line-only smoke missed it). Related: anchor heading greps to the FULL heading (`^## Reference \(§20\)` not `^## Reference`) so sibling headings (`## References`) don't false-match.

## PR-claude-selection-bg-distinct-from-container-001
*severity: medium*

A selection/active-state background that equals its container's background is an INVISIBLE highlight. When a row/item marks its active or hovered state with a `bg(X)`, X must be perceptibly distinct from the container fill it sits on — never the same token. This bites silently after a theme/token recalibration: Marley #194 recalibrated the dock to `surface`, which made the #152 rail's active-row `bg(surface)` highlight surface-on-surface = invisible (the active tab read only via brighter text) — undetected until a Warp-parity capture (#219). Guard it with a PURE helper that derives the highlight from a DIFFERENT token (e.g. an accent wash `accent.opacity(α)`) plus a unit test asserting the highlight is distinct from the container bg (`hl != container`, translucent `a<1.0`, or a lightness/contrast delta ≥ threshold) — so a future token change that collapses the two fails a test instead of shipping an invisible selection. Applies to any list/tree/tab/menu selection or hover fill.

## PR-claude-self-oracle-test-cant-kill-mutants-001
*severity: high*

A test that builds its EXPECTED value by calling the same helper the code-under-test calls cannot kill a mutant in that helper — the mutation breaks both the actual and the expected identically, so the assert still passes. Here every color test asserted `ansi_color_to_hsla(x) == hsla_from_rgb(expected_rgb)`, but ansi_color_to_hsla ITSELF calls hsla_from_rgb, so all 7 hsla_from_rgb mutants survived (MSI drop). Fix: pin the shared helper against GROUND-TRUTH constants computed independently of it (e.g. hsla_from_rgb(255,0,0) must equal a hand-derived HSL h=0/s=1/l=0.5, not `some_other_call_that_also_uses_hsla_from_rgb`). Smell: `assert_eq!(f(x), g(y))` where f and g share a callee — that shared callee is untested. Give at least one leaf helper a from-first-principles oracle test.

## PR-claude-selftest-cannot-drive-key-char-ui-input-001
*severity: low*

The self-test harness (scripts/selftest/drive.swift, synthetic CGEvent keycodes) can drive UI that reads `keystroke.key` (the terminal input path, the keymap chords — cmd-P/cmd-shift-p OPEN fine) but CANNOT drive UI that reads `keystroke.key_char` (the printable-character field): macOS does not populate key_char for keycode-posted synthetic events, so a text field / overlay query that pushes key_char stays EMPTY under the harness even though it works for a real keyboard. Seen in #57: cmd-P opened the finder overlay + rendered the ranked list live, but typing 'app' did not filter (key_char empty). This is the same keycode-vs-character gap as the shell (the finder/palette read key_char; the terminal reads key). Implication for validate: for a key_char-based typing interaction, the harness proves OPEN + RENDER + the initial (empty-query) list + selection highlight, but the type-to-filter must be UNIT-tested (the pure results/push at MSI 100) and the live filter noted as un-driveable (NOT silently skipped). Do not treat an empty query under the harness as a bug — it's the synthetic-input limitation; real users set key_char. (A future harness upgrade could post events via a real HID/AX typing API that populates characters.)

## PR-claude-selftest-finder-plain-enter-inserts-path-001
*severity: medium*

Driving Marley's ⌘P finder: plain Enter INSERTS the chosen path into the focused terminal's prompt (the #65 semantic — it does NOT open the file; on a non-terminal tab the text lands in an invisible background terminal), while ⌘-Enter opens the file in the editor. drive.swift's `cmd:` verb takes a single base char (no `cmd:enter`), so the deterministic driven route to OPEN a file is a plain click on the Files-tree row (opens a full editor tab) or palette "Split Right → File" for the #246 pane. Also: `type:` cannot send '.' (not in the keycode map) — type the stem ("keymap", not "keymap.rs").

## PR-claude-selftest-focus-marley-before-driving-input-001
*severity: medium*

The Marley self-test harness (scripts/selftest/drive.swift) posts synthetic scroll/click CGEvents at SCREEN coordinates — they are delivered to whatever window is frontmost at that screen point, NOT to a window by id. If the HOST terminal (Warp, which runs this agent session) covers the Marley app window, the events drive WARP, not Marley — while `screencapture -l<winid>` STILL correctly captures the occluded Marley by window-id. So the CAPTURE looks right (the affordance renders) but the INPUT did nothing (the view never responds). A scroll/click-only drive WITHOUT a leading `focus` (or `osascript ... set frontmost`) in the SAME drive.swift invocation silently drives the wrong window. ALWAYS start a scroll/click self-test drive with `focus` (drive.swift `focus` clicks the Marley window to raise it) in the same call, immediately before the scroll/click. A `type:` sequence that begins with `focus` masks this bug because the focus raises Marley first — which is why typing worked but a later scroll-only drive silently failed. Symptom: the new affordance renders in the capture but never responds (e.g. 'the view never leaves the bottom no matter how hard I scroll'). Chad caught this live during #198.

## PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism-001
*severity: low*

The Marley self-test live-capture / synthetic-input harness is FULLY BLOCKED when the macOS machine is LOCKED or the display is asleep: `screencapture -l<winid>` returns "could not create image from window", a full-screen `screencapture -x` shows the lock screen ("Enter Password"), and synthetic CGEvents are intercepted by the lock screen. `caffeinate -u` wakes the display TO the lock screen but CANNOT unlock it, and an agent must NEVER attempt a password. This can happen PARTWAY through a validate run (earlier captures in the same session may have succeeded before the machine auto-locked). Do NOT treat it as a code failure or silently skip: (1) CONFIRM it via a full-screen `screencapture -x` — a real render is a large PNG (~1MB); a locked/asleep screen is tiny-black (~150KB) or shows the lock UI; (2) DOCUMENT it explicitly in the validate notes as env-blocked + outside-my-control (see the marley-ax-visual-testing memory); (3) VERIFY the UI behavior via the unit tests + the critic-traced MECHANISM — this is strongest when the changed code path is byte-identical to a previously-live-proven idiom (e.g. #204's workflow-invoke buffer-insert IS the shipped history-finder/completion insert, already live-proven in #200/#65). The gate's gate-15 visual/AX is HEADLESS (a committed-baseline fixture), so it stays green regardless of the lock — the gate is NOT blocked.

## PR-claude-selftest-rebuild-and-cwd-before-blaming-code-001
*severity: medium*

When a UI change "doesn't show" in the live self-test app, suspect the HARNESS before the code — two traps hit in #56: (1) STALE BINARY — `bundle-app.sh` only rebuilt when the binary was absent, so it served a pre-change binary and the new dock rendered empty; the code was correct all along. ALWAYS rebuild (`cargo build -p marley`) before a capture; a working feature that renders blank is usually stale, not broken. (2) LAUNCH CWD — a `.app`/`open`/LaunchServices launch gets cwd `/`, so any cwd-dependent feature (project discovery, file tree) sees `/` not the repo and looks empty. For those, launch the BARE binary from the project dir (`run_in_background`, inherits cwd) and screencapture by CGWindowID (works even though System Events can't see a bare binary). Diagnostic that saved #56: a one-line `eprintln!` of the built state in `new()` proved the model had 661 rows → isolated the problem to the render/harness, not the data. Fixed bundle-app.sh to always rebuild + documented both in scripts/selftest/README.md.

## PR-claude-selftest-screencapture-shadow-offsets-small-target-clicks-001
*severity: low*

`screencapture -l<winid>` captures the window WITH its macOS drop shadow, so the captured PNG is slightly larger than the window frame and a click FRACTION computed on that PNG maps a few px off the real window frame that drive.swift clicks (`w.x + w.w*fx`) — worst at the bottom/side shadows. For a SMALL target (a ~22px button) this offset can miss the target entirely (it did on #198's jump-to-bottom button, 2 attempts). Do NOT rabbit-hole pixel-hunting the exact click (the harness guidance: stop after 2-3 failed UI attempts). Instead verify a small button's BEHAVIOR via its underlying mechanism + the visibility gate: e.g. the jump-to-bottom button's handler calls `viewport.scroll_down(...)` re-anchor, which the wheel-down ALSO calls — drive the wheel-down (lands reliably in the pane body, a big target) to exercise the identical code path, and confirm the button (gated on `!at_bottom`) disappears at the bottom; combine with the pure unit test on the re-anchor. State the harness pixel-precision limitation explicitly in the validate notes (§7) — do not hand-wave it as 'verified'.

## PR-claude-selftest-seed-workspace-shell-to-unlock-the-app-bundle-001
*severity: high*

RETIRES the "a live pixel capture is orchestration-blocked because `open` forces cwd=/" claim (#310). The cwd only ever mattered for AUTO-DISCOVERING the project root. Seed `workspace.shell` in `~/.marley/config/settings.toml` with the #163 codec (`"0\n<root>\t0\tT=t"`, literal \n/\t escapes so TOML unescapes them — write it with a QUOTED heredoc or the shell eats the backslashes) and the root becomes explicit, so a `.app` bundle's cwd=/ is irrelevant. That unlocks the bundle, and ONLY a bundle can be activated deterministically: `scripts/selftest/bundle-app.sh debug` + `open target/Marley.app` + `osascript -e 'tell application "System Events" to set frontmost of process "marley" to true'` — re-activate in the SAME bash call before each input burst. A BARE binary cannot be System-Events-activated (it never registers with the window server), so its input lands on whatever is frontmost. Two more traps: (1) `drive.swift`'s `focus` verb clicks (center-x, 12%-down), which is LAYOUT-DEPENDENT and on the current layout lands in the Files pane — it is not a generic "make Marley frontmost" verb, and for an editor drive it steals focus to the wrong pane; (2) a persisted editor tab (`V=` in the shell blob) restores the file already-open, which does NOT fire `open_file_in_viewer` — reseed a terminal-only `T=t` shell if the drive needs the file-open path (e.g. to spawn the LSP).

## PR-claude-selftest-stuck-synthetic-modifier-clear-with-keyup-001
*severity: medium*

Marley self-test harness: repeated ⌘⇧ chord drives (scripts/selftest/drive.swift cmdshift:*) can leave the OS/CGEvent synthetic-input layer with a STUCK cmd-shift modifier — after which `type:<text>` is misread as CHORDS (e.g. "bcastmark"'s "a" chars fire ⌘⇧A=new-agent, spawning phantom agents; a chord-free key like "z" types nothing because ⌘⇧Z is unbound). Tell-tale: type: spawns agents or types nothing, while MOUSE clicks (clickat:) still work (they don't use the stuck keyboard modifier). It is NOT an app bug — a real user releasing ⌘⇧ physically never hits it — and killing/relaunching the app does NOT clear it (the stuck flag is OS-level). FIX: post CGEvent keyUp events for the modifier virtual keycodes (L/R command 55/54, shift 56/60, option 58/61, control 59/62) via a tiny Swift script (`CGEvent(keyboardEventSource:virtualKey:keyDown:false).post(tap:.cghidEventTap)`), then `type:` works again. Run this reset before a keyboard-driven self-test if a prior run left modifiers stuck. Diagnostic marker for "is type: working": type a chord-free string (only letters NOT in the keymap: p/d/w/b/r/a/f/e/s/g are bound, so use e.g. "zxmnot") and confirm it appears at the prompt.

## PR-claude-selftest-wake-display-with-caffeinate-before-screencapture-001
*severity: low*

The live-app self-test (drive.swift + screencapture) FAILS silently when the macOS display has slept during a long session: `screencapture -l$WIN` errors "could not create image from window" and a full-screen `screencapture -o` returns an ALL-BLACK PNG (not the app). Worse, synthetic input to a slept display may not register (the ticket's chord appeared not to fire — the ssh child was absent — until the display was awake). BEFORE a self-test in a long-running session, WAKE the display: run `caffeinate -u -t <secs>` (declares user activity → wakes the display) and keep it alive across the drive+capture (`caffeinate -u -t 20 &`), then re-drive from a clean state (reset_mods + re-focus). Symptom→cause map: an all-black capture or "could not create image from window" = slept display, NOT a render bug — don't chase the code. Applies to every UI self-test at /pipeline:validate.

## PR-claude-serialize-at-the-right-codec-layer-001
*severity: low*

When deciding a persisted encoding for a NEW content kind in Marley's layout codec, first identify WHICH layer serializes that artifact — they reserve different bytes. A whole-TAB kind goes through the SHELL codec (serialize_shell: entries T=/C=/V= separated by \t \n, plus \x1f) which reserves ONLY \t \n \r \x1f — so a URL or path is already framing-safe there. A split-CELL (grid leaf) goes through the GRID codec (breaks_grid_framing) which additionally reserves , : = — so a URL WOULD break a grid leaf. Reason about framing-safety at the artifact's ACTUAL layer, never a sibling's: a `TabContent::Browser` is a shell tab (safe), not a grid leaf. Corollary: the safest choice at either layer is to persist a bare marker + re-derive the value from its single source of truth, so no reserved-byte payload is ever stored.

## PR-claude-shared-copy-closure-for-a-call-counted-probe-test-001
*severity: medium*

A test that call-COUNTS a closure to prove a gate ("the probe runs for an opener but NOT for a closer") leaves the closure's body UNCOVERED for the never-invoked case — and llvm-cov also counts that closure as a MISSED FUNCTION, so gate:4 (100% lines, whole-workspace) goes red even though the test passes. Fix: define ONE probe closure and SHARE it across both the invoked and non-invoked calls. A closure that touches its counter via `&Cell`/`&RefCell` (e.g. `calls.set(calls.get()+1)`) captures the cell by SHARED REFERENCE, which makes the closure `Copy`, so each call consumes a copy (not a move) and both calls drive the same counter. The invoked call (the opener) then EXERCISES the body while the non-invoked call (the closer) still proves 0 invocations. Two separate closure literals is the trap — the closer's literal is dead code. Verify with `cargo llvm-cov nextest -p <crate> --show-missing-lines` before the full gate; a `--diff` gate still floors LINE coverage whole-workspace, not just the diff.

## PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001
*severity: medium*

When replacing inline-owned content (reaped via RAII/Drop when its single owner closes) with a SHARED id-keyed registry (HashMap<Id, Content> referenced by N views), you LOSE the Drop-based teardown: a plain map gives resolution, not drop-on-last-close. The registry MUST carry an explicit per-id refcount (or a view-set of referencing cells) so closing one of N views only decrements, and only the LAST close removes the entry and runs teardown (e.g. hands an owned PTY/session to a reaper thread). gpui's Entity<T> refcounts natively; a hand-rolled registry does not. Design the refcount + the moved-teardown contract as a first-slice requirement of the refactor, not a footnote — every existing close path (pump/close-tab/close-project/close-focused/close-pane) must re-route through the registry decrement.</rule>
<severity>medium</severity>
<language>rust</language>

## PR-claude-shared-vocabulary-across-a-parse-emit-seam-must-agree-001
*severity: medium*

When one function EMITS a string another function PARSES (a display→parse, serialize→deserialize, or format→tokenize seam), verify the two agree on separators/grammar with a round-trip assertion BEFORE relying on it — a mismatch degrades silently to an identity no-op that compiles, runs, and looks plausible (e.g. display() joined on '-' but parse() split on '+' → the tokenizer returned the input unchanged, so the whole integration did nothing). The seam's own doc/example is the contract: match it. This is the emit-side twin of the TICKET-022 decode-order rule — check the OTHER side's grammar, don't assume it.

## PR-claude-shim-aggregate-all-terminal-grids-001
*severity: medium*

A workspace-wide shim aggregation (diagnostics gutter, completion badges, any "scan every terminal" feature) must iterate ALL terminal grids across the project's tabs — NOT `workspace()` / `terminal_grid_index()`, which returns only the active-or-first terminal grid. When the active tab is the editor, `terminal_grid_index()` falls back to the FIRST terminal tab, silently dropping failed blocks in other terminal tabs. Use a `Project`-level iterator over every tab's grid. Corollary: a pure-seam unit test can't catch this — it lives in the shim's terminal enumeration, so a driven/headless MULTI-terminal-tab case is required to prove a cross-tab aggregation.

## PR-claude-shim-needs-both-cov-exclude-and-mutants-skip-001
*severity: high · prevents: BF-claude-gpui-fixture-bin-mutants-not-skipped-001*

An ACCEPTED-UNTESTABLE shim target (a gpui render/ module, an OS-FFI file, or a headed-only fixture/runner BIN) must carry BOTH exclusions, because coverage-exclusion and mutation-exclusion are SEPARATE mechanisms: (1) the rust_cov `--ignore-filename-regex` entry in scripts/gates.sh (keeps it out of the 100% line-coverage denominator), AND (2) `#[cfg_attr(test, mutants::skip)]` on EVERY fn in it (keeps cargo-mutants from generating mutants for it). Excluding a file from coverage does NOT stop it from being mutated — a headed-only bin/module that is coverage-excluded but not mutants::skip'd still yields surviving mutants (its fns are never run by the gate's tests) → MSI<100 → mutation gate RED. VERIFY with `cargo mutants --list -p <crate>` at inspect: the shim files (src/render/, src/bin/) must show ZERO listed mutants; if any leak, add the missing mutants::skip. This applies to the widget render bins/modules exactly as it did to marley_app's app.rs + bin/marley.rs and marley_spike's runner.

## PR-claude-silent-no-op-loader-verify-the-landing-001
*severity: high · prevents: BF-lsp-goto-failed-open-writes-caret-into-wrong-file-001*

A loader shaped `if let Some(x) = load(target) { ... }` with NO else FAILS SILENTLY and leaves the previous state active. Any caller that afterwards writes state DERIVED FROM THE REQUESTED TARGET (a caret offset, a scroll row, a history/undo push, a selection) must VERIFY THE LANDING first — else the failure writes target-derived state into the wrong object, silently and with no error to signal it. In Marley: open_file_in_viewer no-ops on missing / non-file / >2MB / binary; verify `active_editor().is_some_and(|ed| ed.active_file().path == path)` before touching the caret. Two tells that this is the bug: (1) the loader flashes its own error yet the caller proceeds anyway; (2) the caller's own clamp (position_to_offset) turns a wrong target into a plausible-looking result instead of a crash, so nothing surfaces it. Also order the writes so a failure leaves NOTHING behind: read the origin before the operation, but commit it (push/record) only after a confirmed landing.

## PR-claude-single-quote-shell-command-args-001
*severity: high*

Any path or user/agent-derived string interpolated into a shell COMMAND string that will be executed (written to a PTY, passed to `sh -c`, etc.) MUST be POSIX single-quote wrapped, escaping an embedded single quote as `'\''` (close-quote, backslash-escaped-quote, reopen-quote): `format!("cd '{}'", s.replace('\'', "'\\''"))`. Single quotes neutralize EVERY shell metacharacter (`;` `$` backtick `$(…)` space newline glob) — only `'` itself is special inside `'…'`, and the escape handles it. Do NOT build the command with bare interpolation or double quotes (which still expand `$`/backtick). Verify with a real shell: `cd_command("/tmp/x; echo PWNED")` must parse to ONE literal arg, never echo PWNED. (M18 #294 `cwd_link::cd_command`.)

## PR-claude-sleep-n-validates-is-command-running-ui-001
*severity: low*

To validate UI that reacts to a FOREGROUND command running (any `is_command_running`-gated behavior — prompt-row hiding #193, command-aware tab titles #201, background-command-done notifications #203), drive a plain `sleep N` in the live bundled app and screencapture the running vs. exited states, rather than launching an agent (`claude`). `sleep N` toggles `is_command_running` deterministically and needs no external binary on the .app's PATH (a LaunchServices `.app` launch gets a minimal env / cwd=/), so it can't false-negative on a missing agent. Capture three frames: idle (baseline), while `sleep N` runs (the gated change), after it exits (restored). Proven on #193: idle `❯` shown → running: `❯` hidden + block visible → exit: `✓` block + `❯` returns.

## PR-claude-sort-order-with-no-mutant-needs-3plus-scrambled-fixture-001
*severity: medium*

A `sort_by_key` / ordering step gets NO cargo-mutants mutant (cargo-mutants does whole-body replacement — `vec![]` / `Default` — never a closure/comparator mutation), so MSI 100 says NOTHING about whether the sort is correct; the ORDERED behavioral assert is the only guard. And a 2-ELEMENT ordered assert over a HashMap/HashSet is a FLAKY regression detector: HashMap iteration order is process-random, so a 2-elem `assert_eq!(rows, [a,b])` on UN-sorted code passes ~50-75% of runs (measured in #68). FIX: test the sort with **≥3 elements inserted OUT of the target order** (e.g. insert ids 3,1,2 → assert sorted 1,2,3) — then an accidental-correct order without sorting is ~1/n!, a reliable guard. This is the ordering variant of the invariant-invisible-logic family (a no-mutant step needs a STRUCTURAL/behavioral test with a discriminating fixture, not a minimal one). Applies to any HashMap→sorted-Vec projection (agent_rows, sprint/ticket lists, file listings). The critic caught this on #68 by MEASURING the 2-elem HashMap order flakiness.

## PR-claude-spawn-once-worker-drop-handle-on-send-error-001
*severity: medium · prevents: BF-spawn-once-worker-never-respawns-001*

A spawn-once background worker (gated on `handle.is_none()`, stored as `Some((tx, rx))`) MUST drop its handle back to None whenever `tx.send()` errors — a send error means the thread exited, and without the reset every future launch sends into a dead channel and the feature is silently dead for the app's lifetime with no recovery short of restart. Pattern: `let sent = match &self.worker { Some((tx,_)) => tx.send(req).is_ok(), None => false }; if !sent { self.worker = None; } sent`. The next launch then respawns. Applies to any persistent worker+channel (search, syntax, indexer).

## PR-claude-spawn-resolve-absolute-not-cwd-relative-001
*severity: high · prevents: BF-claude-lsp-relative-path-exec-001*

A PATH search that feeds a process spawn MUST return an ABSOLUTE program path (skip non-absolute PATH dirs, or canonicalize the hit). A relative command string containing a separator is resolved by the OS against the child's `current_dir` — and spawns commonly set `current_dir` to a workspace/repo root — so a relative resolved-command executes a binary the (possibly hostile) repo ships. Filtering only the empty PATH element (empty==cwd) is INCOMPLETE: a literal `.` or any relative dir is the same hole. Also: probe existence under the SAME cwd the spawn will use, or the check and the exec can disagree. Bit #308 (relative-PATH → repo-local rust-analyzer exec).

## PR-claude-split-totality-audit-at-always-run-fault-line-001
*severity: medium*

When a "make X total / no-panic over a new state" audit balloons past one clean slice (many reachable call sites across subsystems), SPLIT it at the ALWAYS-RUN vs USER-TRIGGERED fault line, and ship the always-run slice FIRST. Rationale: the always-run paths (the gpui render fn every frame, the pump every tick, persist on every change) hit the new state BEFORE any user action — so they are the PREREQUISITE (a user can't even reach/act on the state until the app can display+persist it). The user-triggered slice (keybindings, overlays, commands, the raw-key router) is second and CARRIES THE PRODUCT QUESTIONS (what should ⌘D / a keystroke do in the new state — no-op vs create vs flash), so it naturally pairs with the behavioral ticket that makes the state reachable. Contract heuristic: add an Option-returning `try_` twin of the panicking accessor for the always-run sites; KEEP the panicking accessor for the genuinely-gated sites (a precondition guarantees the old invariant by their trigger) — this minimizes churn vs making the accessor itself Option (which forces every call site, gated or not). Reject a "sentinel empty value" (e.g. an empty PaneGrid) when the value type has an internal invariant the empty case violates (PaneGrid.focused: PaneId must index a real pane). Prove the always-run slice behavior-neutral: guards stay ON → the new state is TEST-ONLY reachable → the full suite is byte-identical + a pure predicate + adversarial byte-identity review carry it (the gpui render path is coverage/mutation-excluded).

## PR-claude-stat-before-read-attacker-influenced-path-001
*severity: high*

Before reading a file whose PATH is attacker-influenced (e.g. a clickable path scanned from terminal output, a link, any untrusted string), `stat` it FIRST and reject a non-regular file and an oversize file BEFORE the read — never `std::fs::read`/`read_to_string` the whole thing and check size after. A bare read of a fifo or a character device (`/dev/zero`, `/dev/urandom`) BLOCKS or grows unbounded → a UI-thread freeze / OOM; a size guard that inspects already-read bytes is too late. Pattern: `let meta = std::fs::metadata(p)?; if !meta.is_file() || meta.len() > MAX { reject } ` then a bounded read (or `File::open(p)?.take(MAX+1)`). Worked example (#196 clickable links): `open_file_in_viewer` read-before-guard → a clicked `/dev/zero`/fifo from output froze the app; fixed with metadata-first. Corollary for the SAME feature class: an inline clickable element (a link overlapping selectable text) must open on `on_click` (mouse-UP, drag-suppressed) NOT `on_mouse_down`, and must NOT `stop_propagation` if it sits over a drag-select region — else press-opens and you can't start a selection on it.

## PR-claude-state-machine-accessor-test-must-satisfy-preconditions-001
*severity: medium*

When unit-testing an accessor that reads STAGED or DERIVED state from a state machine (a `current_*`/`staged_*`/`pending_*` getter), the test must drive the machine's FULL precondition sequence to actually populate that state — not just fire the one hook that looks like it sets it. Example: `SessionModel::current_prompt()` returns `staged_prompt`, which `Precmd` only sets AFTER an `InitShell` registered the session — a `Precmd` on an unregistered session returns `Err(MissingSession)` and stages nothing. A test that fires `precmd` without `init` gets `None` (or an `.unwrap()` panic), the assertion silently reads as `is_none()`, and the accessor's `-> None` mutant SURVIVES → gate:5 MSI<100. This is a false-green trap: the test looks like it exercises the accessor but the staged state was never set. Rule: trace the state's write path in the source, enumerate every precondition (session registered? prior hook applied? not-yet-consumed?), and satisfy them ALL in the fixture before asserting `Some`. Add the symmetric `None` assertions (before-stage AND after-consume) to kill the accessor mutant from both sides.

## PR-claude-status-glyphs-from-common-matched-set-001
*severity: low*

When choosing non-ASCII glyphs to render in the monospace terminal font (status indicators, prompt markers, icons), pick from a COMMONLY-PRESENT, visually-MATCHED set rather than the first codepoint that looks right. A missing glyph renders as tofu (□), which you CANNOT catch in a headless/env-blocked context. Concretely: the success/failure pair should be ✓ U+2713 CHECK MARK + ✗ U+2717 BALLOT X (both in the Dingbats block, both conventionally present and stylistically matched), NOT ✕ U+2715 MULTIPLICATION X (a math symbol, less certain in terminal fonts like Menlo, and stylistically mismatched with the ballot check). Prefer glyphs from a single block (Dingbats, Geometric Shapes) so weights/metrics agree. When unsure of coverage, favor the more common codepoint or an ASCII fallback, and flag the choice for the human's headed visual verification. Applies to every M1.E visual glyph (status dots, the ❯ prompt marker, dock icons).

## PR-claude-subprocess-positional-arg-guard-leading-dash-and-double-dash-001
*severity: high*

When building a subprocess argv where a caller-influenced value lands in a POSITIONAL slot (an ssh/git/rsync destination, a filename, a URL), a `Vec<String>` argv only stops SHELL injection — it does NOT stop OPTION-SMUGGLING: most CLIs re-parse any argument that starts with `-` as a FLAG, so a positional value like `-oProxyCommand=evil` (ssh) or `--upload-pack=evil` (git/rsync) becomes an option, not the intended operand. This is the CVE-2017-1000117 class and is a real (latent) RCE for tools with command-executing options. TWO-PART GUARD (do both): (1) VALIDATE — reject a value that starts with `-` at parse time (a legitimate host/user/filename never does), for a clean early failure; (2) SEPARATE — insert the end-of-options marker `--` immediately before the positional arg(s) in the argv (`["ssh", flags…, "--", dest]`), which defends even a value that skipped the validator (a hand-built struct, a later code path). A doc comment claiming "argv, so no injection" is INCOMPLETE until both are in place — argv defeats shell injection, `--`+leading-dash-reject defeats option-smuggling. Applies to every future marley_remote/subprocess builder.

## PR-claude-suppression-test-must-prove-the-baseline-would-not-suppress-001
*severity: high*

A test that a feature SUPPRESSES some default behavior (auto-close pairing, a warning, a scroll, an auto-insert) must be constructed so the BASELINE (pre-feature / feature-off) would produce the OTHER, un-suppressed outcome at that exact input — otherwise the test passes for the wrong reason: an unrelated guard already produces the suppressed result, and reverting the feature leaves the test green (a vacuous pass). Concretely: trace every OTHER gate on the code path at the fixture's exact position. Marley #362: an auto-close drive typed `'` at `fn f<T: |>` where the trailing `>` makes `opens_here(next)` false — pairing was already blocked, so the LifetimeBound suppression was never exercised; the fix was a fixture where `next` is whitespace (`opens_here` true) so a Code context WOULD pair and only the feature suppresses. Verification ritual: mentally (or actually) revert the feature and confirm the assertion FAILS; prefer asserting the ABSENCE of the suppressed artifact (`!buf.contains("''")`) so the baseline's presence of it is the failure signal.

## PR-claude-suppression-test-needs-a-positive-control-001
*severity: high*

A test that asserts ABSENCE (a popup must NOT open, a request must NOT be sent, an event must NOT fire) proves nothing on its own — a dead harness produces the same zero. Always pair it with a POSITIVE CONTROL in the same run: an adjacent case that MUST produce the effect. Score the test only if the control fires. Real case (#313 REQ-006): the "`.` inside a comment sends 0 completion requests" check read 0 and was one step from PASS — the paired code control ALSO read 0, which is the only thing that revealed the LSP had never spawned and the whole drive was dead. The strongest form is A/B in one session, one line apart (comment vs code), so the only difference is the property under test.

## PR-claude-survivor-model-must-mirror-reader-drops-001
*severity: medium*

When a WRITER pre-computes a survivor set to reclamp an index (or any survivor-derived value) over what reaches the wire, the survivor predicate must model EVERY guaranteed drop on the READ path too — not just the writer-side drops. Enumerate the parser/consumer drop arms (empty-field skips, unknown-tag skips, validation continues) and either mirror each in the writer predicate or prove it unreachable from writer output. A survivor the reader deletes shifts every later index exactly like the bug being fixed.

## PR-claude-symbol-referencing-test-misses-const-and-boundary-mutants-001
*severity: medium*

A test that references a SYMBOL (a const, or a comparison's operand) moves WITH a mutation of that symbol's definition and cannot kill it. Two shapes bit #308: (1) a const with arithmetic (`MAX_FRAME_LEN = 64*1024*1024`) — cargo-mutants mutates the `*`→`+`, but every test using `MAX_FRAME_LEN + 1` recomputes against the mutated value and still passes; pin it with a HARD-LITERAL assert (`assert_eq!(MAX_FRAME_LEN, 67_108_864)`). (2) a boundary comparison (`len > CAP`, `now - t < window`) — a `>`→`>=` / `<`→`<=` mutant only dies at the EXACT boundary input (feed exactly CAP → still Ok; crash at exactly `window` ticks → still tolerated). Symbolic/relative tests pass on both operators. Always add an exact-value pin + an exact-boundary case for a const or an inequality.

## PR-claude-synthetic-response-tests-never-prove-the-live-wire-001
*severity: high*

A test that INJECTS a response into your own queue proves your consumer, never your producer — so a feature whose value depends on an external process answering correctly is NOT proven until the real wire is inspected. Marley's #309/#310/#311/#312 all "passed driven proof" via `push_response_for_test` into a process-less LSP host; every one of them was dead in the real app because the didOpen the app SENT carried an empty document, and no injected-response test can see that. For any external-process integration, add ONE end-to-end check that observes the actual traffic: the cheapest is a tee wrapper installed through the app's own server config (`[[lsp.servers]] command = "…/tee.sh"`) that pipes stdin/stdout through `tee`, then assert on the captured frames (method present AND payload non-degenerate — assert `TEXT_LEN > 0`, not merely `didOpen` appears; the bug shipped WITH a correct-looking method name). Diagnostic tell that the far side never received your document: a request that should resolve trivially (a definition in the SAME file) returns empty — if a same-file jump fails, stop suspecting your parser and go look at what you SENT.

## PR-claude-test-code-must-avoid-never-run-branches-001
*severity: medium*

Test code counts toward the coverage floor, so it must contain NO lazy-evaluated never-run branch: `else { panic!() }`, `match { specific => …, _ => panic!() }`, and a formatted `assert!(cond, "…{}", expr)` message all leave an uncovered line on the (always-taken) success path. Extract an enum payload with an IRREFUTABLE or-pattern (`let (A(x) | B(x)) = v;`), assert a variant with `matches!`, and use plain `assert!`/`assert_eq!` with no formatted message. This is why a brand-new crate's own tests can drop workspace coverage below 100%.

## PR-claude-test-destructures-use-accessor-expect-under-a-lines-floor-001
*severity: low*

Under a 100%-lines coverage floor, TEST code is instrumented too: a `let Pattern = value else { panic!(...) }` destructure in a passing test leaves the panic body as a permanently-uncovered line and turns the coverage gate red. Destructure through an accessor instead — `value.as_x().expect("...")` — the expect call sits on an executed line and doubles as the assertion. The floor applies to everything in a touched file, including its #[cfg(test)] module.

## PR-claude-test-helper-assert-message-literal-not-lazy-call-001
*severity: low · prevents: BF-lazy-assert-message-uncovered-cold-arm-001*

In a whole-workspace-100%-line-coverage repo, do NOT put a runtime function CALL in an `assert!`/`assert_eq!` failure message inside a test helper whose happy path always holds (e.g. `String::from_utf8_lossy(&out.stderr)`, a `format!`, a `.to_string()`) — the message args are evaluated only on the panic arm, which never runs, so llvm-cov flags them as an uncovered cold region and gate:4 goes RED. Use a LITERAL &'static str message (`"a git setup command failed"`) or a plain conditionless assert. Same rule for test-only spawn: `std::process::Command` is banned by `clippy::disallowed_types` EVEN in test setup — use `marley_command::blocking::Command`.

## PR-claude-timeout-killing-cargo-orphans-the-test-binary-001
*severity: medium*

Killing `cargo test`/`cargo nextest` on a timeout does NOT kill the test binary it spawned — the binary is a grandchild and survives as an orphan. In a project whose tests hold a scarce resource, that orphan poisons every later run, and the symptom looks nothing like the cause. Cost me ~45 min on Marley #312: a deliberate "does this test catch the regression?" teeth-check was expected to hang (the documented headless_drive PTY drop-chain edge), so I bounded it with `subprocess.run(timeout=180)`. That killed cargo; the test binary kept running, wedged in the PTY drop chain, for 98 minutes — after which `cargo nextest run --workspace` and the gate's coverage step both hung on an UNRELATED PTY test (`workspace_two_real_sessions_are_independent`, 0% CPU for 42 min) while that same test passed in 0.11s in isolation. After `kill -9` on the orphan: 1257 tests, 3.7s. RULES: (1) after any timeout-killed cargo run, `pgrep -f 'target/debug/deps/'` and kill survivors before trusting the next run; (2) when a test is EXPECTED to hang, prefer nextest's own `slow-timeout`/`terminate-after` config (it kills the test process, not just cargo) — note `--slow-timeout` is a config key, NOT a CLI flag, so passing it as a flag errors out and silently runs nothing; (3) diagnostic tell — if a test hangs at 0% CPU inside a big run but passes standalone, suspect a leaked process holding a shared resource, not the test.</rule>
<parameter name="severity">high

## PR-claude-totality-branch-needs-degenerate-input-test-001
*severity: medium · prevents: BF-totality-branch-only-degenerate-input-uncovered-001*

In a whole-workspace 100%-line-coverage repo, a totality arm written to satisfy §14 (an Option/Result None/Err branch, a `let-else { return default }`) that VALID inputs never reach still needs an explicit test that forces it — or gate:4 goes RED on a non-obvious branch. When you add such a guard, immediately ask "what degenerate input reaches this?" and write that unit: a REVERSED range (start > end) for a byte-range descendant lookup, an EMPTY collection, a fresh/uninitialized session, a malformed parse. Trace the API's ACTUAL behavior — an out-of-bounds range may CLAMP rather than fail, so out-of-bounds ≠ the empty case; only the truly degenerate input triggers the arm. Enumerate these at design/implement so validate isn't surprised.

## PR-claude-totality-substrate-close-cross-function-invariant-accessors-001
*severity: low*

In a no-panic / totality substrate ticket (whose job is making a subsystem total over a newly-reachable empty state), convert even the bare panicking accessors that are safe-by-cross-function-invariant TODAY to their local `try_*` guard — not just the ones reachable right now. A site whose safety rests on "caller X only produces this input when a live Y exists" (e.g. `activate_search_hit`'s `workspace_mut()` safe only because `top_search_sources` yields no session ids without a workspace) is a latent panic the moment an imminent feature breaks that invariant (cross-workspace sources, a global section, a registry). The substrate's value is LOCALITY of totality, not merely today's reachability — the inspect completeness pass must flag every invariant-guarded (not locally-guarded) accessor and the ticket closes them.

## PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001
*severity: low*

When reasoning about whether a test matrix achieves MSI 100 on a pure fn, do NOT enumerate the operator/method mutations by hand — RUN `cargo mutants --list -f <file>` (or `--list` filtered to the fn) to get the DEFINITIVE mutant set, then write exactly the cases that kill those. I mis-guessed twice: on #199 I worried about a `<`→`!=` survivor (cargo-mutants 27.x does NOT emit `!=` or `>=` for a `<` — its swap set is `< → {==, >, <=}`), and on #198 I assumed a `.min(1.0)` deletion mutant that also isn't generated. Concretely, cargo-mutants (27.x) mutates: (a) binary operators via a FIXED small swap set (`<` → `==`/`>`/`<=`; NOT `!=`/`>=`), (b) the fn BODY → its typed defaults (for `Option<usize>`: `None`, `Some(0)`, `Some(1)` — the un-obvious `Some(1)` body-default is real + easy to miss), and (c) it does NOT mutate method calls (`checked_sub`/`map`/`filter`/`.min`/`.max`/`.clamp` are untouched, so a redundant `.min(1.0)` is dead-but-unmutated, not an unkillable mutant). Consequences of guessing: wasted effort chasing phantom survivors (the `!=`/`>=` that never exist), AND a risk of MISSING a real body-default mutant (the `Some(1)` you didn't anticipate). Base the matrix on the real `--list`; keeping an extra boundary case never hurts, but don't reason from an over-broad or wrong operator set.

## PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001
*severity: high*

A transient editor overlay anchored to a caret/position (a hover card, a completion popup, an inline widget) must be dismissed by comparing the LIVE editor identity — (file path, caret offset, buffer version, first-visible row) — in the pump/render loop each tick, NOT only via an action-dispatch guard. Reason: the interactions that should dismiss it mostly BYPASS the action dispatcher — inline-handled arrow/caret motion, IME typing (goes through the input handler), the uniform_list's own scrolling, and MOUSE tab/pane switches (call switch_tab directly). A guard hung off dispatch_action silently misses all of them, leaving a stale card pinned at a moved caret or floating over the wrong file (two unedited files both report version 0 / first-row 0, so a version+scroll-only poll can't tell them apart — you MUST include the file identity). Also: the async response consumer must re-validate the response's file/position against the CURRENT editor (not just the last-request key), because the user can switch files between request and reply. Symptom: hover/popup works when driven by chords but sticks/ghosts on arrows and rail clicks.

## PR-claude-tree-node-kind-match-must-reject-is-missing-001
*severity: low · prevents: BF-claude-tree-walk-trusts-kind-admits-missing-node-phantom-001*

A tree-sitter walk that classifies nodes by `kind()` string alone admits phantoms: error recovery inserts MISSING nodes that carry a REAL `kind()` (e.g. an inserted `)` has kind `")"`) but zero width (start_byte == end_byte). Before trusting a matched node as a real token, reject `node.is_missing()` (or equivalently a zero-width span). Applies to any feature that reads structure off the tree — bracket match, delimiter pairs, span extraction.

## PR-claude-two-comparison-overlap-needs-boundary-per-side-001
*severity: medium*

For a range-overlap predicate `a.start < b.end && b.start < a.end`, cargo-mutants emits a SEPARATE `<`→`<=` mutant for EACH `<`. Each dies only at its OWN touching orientation: `a.start < b.end`→`<=` dies only when `a.start == b.end` (a starts exactly where b ends); `b.start < a.end`→`<=` dies only when `b.start == a.end` (a ends exactly where b starts). A single "touching" fixture exercises ONE orientation and leaves the OTHER mutant alive → MSI < 100. Write BOTH touching orientations (each asserting the touching pair is KEPT / not-overlapping). Generalizes to any predicate with N independent boundary comparisons — enumerate the `cargo mutants --list` output and give each comparison its own boundary case; do not assume one "edge" test covers a symmetric-looking pair. (Caught at #214 inspect: the design's single L5 boundary test left one of ranges_overlap's two `<`→`<=` mutants alive.) Pairs with failure BF-claude-osc8-test-plan-under-specified-ranges-overlap-boundary-001.

## PR-claude-two-engines-answering-the-same-question-will-diverge-001
*severity: high · prevents: BF-claude-two-match-engines-disagreed-on-overlap-and-grew-the-selection-001*

When two features must AGREE about a definition ("what is an occurrence", "what is a word", "what is a match"), make them CALL THE SAME FUNCTION — do not give them two implementations and a test asserting they agree. Two engines answering one question WILL diverge, and the divergence hides in whichever axis you were not thinking about. On #298 I *found* the divergence at plan time (⌘D's `next_occurrence` was case-SENSITIVE, ⌘⇧L's `find_all` case-FOLDING), wrote it up as REQ-007 ("they shall agree"), fixed the CASE axis with a `fold` flag — and shipped them still disagreeing on the OVERLAP axis: `next_occurrence` probes every index so it yields SELF-OVERLAPPING matches that `find_all` (which skips past each hit) never returns. That axis was the one that corrupted user data — an overlapping match, fed to a merge-on-overlap set invariant, silently GREW the user's selection past what they selected. THE TEST: if you catch two implementations disagreeing on ONE axis, do not patch that axis — ask what OTHER axes they could differ on, and if the answer is "I'd have to check", DELETE ONE OF THEM. §0 then does the rest: the redundant engine loses its last caller and gets deleted, which is a feature, not a cost. Corollary for a REQ of the form "X and Y shall agree": the acceptance test must compare X and Y over a fixture that exercises EVERY axis of the definition, not just the one you noticed.

## PR-claude-two-gated-calls-must-read-the-phase-once-001
*severity: critical · prevents: BF-lsp-didopen-carries-empty-text-ready-race-001*

When two calls are gated on the SAME mutable state (a lifecycle phase, a connection status, a readiness flag), never let that state change BETWEEN them — read it once, or order the mutation before both. The classic shape, and how it bit Marley #309: a pump collected `needs_text(..)`-gated data, THEN ran `drain(); reconcile(..)` — but `drain()` is what promotes the host to Ready, and both `needs_text` and `reconcile` are Ready-gated. On the tick the phase flipped, the collector saw NOT-Ready (materialized nothing) while the consumer saw Ready (sent the nothing). The pattern is especially vicious when the consumer then RECORDS the bad value as authoritative — here the doc was marked synced at that version, so no didChange followed and the empty document was permanent, not transient. Smell test: if an optimization asks "will the consumer want this?" before the consumer runs, any state the consumer's own gate reads must already be settled. Prefer collapsing the two reads into one call that decides AND materializes (pass a `&Buffer` or a `impl Fn(&Path) -> String`, not a pre-rendered String), so the coupling cannot exist.

## PR-claude-two-mut-self-accessors-need-a-combined-accessor-001
*severity: medium*

Two `&mut self` accessor methods CANNOT be called in one expression — `f(obj.a_mut(), obj.b_mut())` is E0499 (each borrows the whole `obj`), UNLIKE two struct FIELDS `f(&mut obj.a, &mut obj.b)` which the borrow checker splits. When a caller needs two mutable sub-borrows of the same object together (e.g. an input layer editing a buffer + caret), add ONE combined accessor returning the disjoint field borrows: `fn a_and_b_mut(&mut self) -> (&mut A, &mut B) { let f = &mut self.inner; (&mut f.a, &mut f.b) }`. Don't reach for `.unwrap()`/clone/refcell to dodge it.

## PR-claude-unique-to-one-site-claim-needs-exhaustive-sweep-001
*severity: medium*

When a spec/recon scopes a fix by claiming a bug or behavior is "unique to" one gesture/site/caller (e.g. #342's D3 "the mismatch is unique to ⌘D"), that is a load-bearing scoping claim — enumerate EVERY caller of the shared primitive AND every sibling gesture of the same class BEFORE locking it, not just the callers the ticket cites. #342's recon named the callers of `scroll_editor_to_row` and concluded "⌘D-only", but missed `add-cursor-below` (⌘⌥↓), a DIFFERENT gesture that also adds a non-primary cursor and scrolls via `follow_editor_caret` (the primary) — the inspect critic's exhaustive sweep caught it (→ #360). The cited-callers list is not the sweep. Grep all callers of the primitive AND grep the sibling verbs (add-cursor, multi-paste, select-all) for the same "adds a non-primary cursor then scrolls" shape. Over-claiming ships a silent analogous bug; the honest scope is "this is what THIS ticket fixes" plus a follow-up for the rest.

## PR-claude-unreachable-match-arm-is-uncoverable-001
*severity: medium*

A match arm that no reachable input can hit is a permanently-uncovered line — it fails a 100% line-coverage floor and hosts an unkillable mutant. This bites when an enum is handled in two places: a caller intercepts some variants upstream, then a helper matches the enum with an explicit arm per remaining variant PLUS a `_ => …` catch-all — the catch-all is now unreachable. Fix: make the match EXHAUSTIVE with no catch-all (list every variant), so every arm is reachable by some input. Only possible if the enum is not `#[non_exhaustive]` — check first; a non_exhaustive enum forces a `_` arm you cannot cover (then the fn needs a different shape, e.g. return an Option the caller handles, or fold the two match sites into one exhaustive match).

## PR-claude-update-caller-docs-when-method-behavior-changes-001
*severity: low*

When you change a method's behavior (not just its signature), re-grep its CALLERS and fix their doc comments too — a caller-level doc that paraphrases the old behavior silently rots. #249 changed EditorSurface::open from "refresh+activate an existing path" to "switch, preserving the buffer"; the method's own rustdoc was updated but the caller Project::open_or_switch_code still documented "refresh + activate" (a critic caught it). After a behavior change, search for the method name across docs/comments, not only call sites.

## PR-claude-use-plus-not-or-for-disjoint-bitfield-assembly-001
*severity: low · prevents: BF-claude-disjoint-nibble-or-equals-xor-equivalent-mutant-001*

When assembling a value from DISJOINT bit-ranges (e.g. `(hi << 4) | lo` where hi,lo ∈ 0..=15, or `(b0<<8)|b1` for byte pairs), prefer `+` over `|`: for disjoint operands they're numerically identical (no carry), but `|`/`^`/`+` are all equal there, so cargo-mutants' `|→^` (and `|→&` partially) are EQUIVALENT/unkillable mutants → MSI<100 with no test able to fix it. Writing `+` makes the operator mutants (`+→-`, `+→*`) observably wrong on a nonzero high range → killable by an exact-value assert. Only reach for an `exclude_re` mutants-skip if `+` is genuinely unsuitable.

## PR-claude-use-then-lazy-not-then-some-eager-for-fallible-args-001
*severity: medium · prevents: BF-claude-then-some-eager-arg-underflow-panic-001*

`bool::then_some(x)` evaluates `x` EAGERLY — even when the bool is false. So NEVER use `cond.then_some(expr)` when `expr` can panic/underflow (a subtraction like `i - 1`, an index, a `.parse()`, an allocation-heavy build) or is expensive; the guard does NOT protect it. Use `cond.then(|| expr)` (lazy — only runs when true) or, better, a TOTAL operation that encodes the guard: for `(i >= 1).then_some(i - 1)` use `i.checked_sub(1)`; for a fallible parse use the `Option`/`Result` directly. Beware: clippy's `unnecessary_lazy_evaluations` lint suggests replacing `.then(|| expr)` with `.then_some(expr)` — that suggestion is WRONG (a latent panic) whenever `expr` is fallible/panicking, so prefer the total operation (`checked_sub`, `get`, etc.) which clippy leaves alone. Verify guarded arithmetic against the boundary input (index 0, empty, max) in a unit test.

## PR-claude-verify-a-claimed-external-api-is-public-before-locking-001
*severity: high*

When a ticket, spec, or a prior ledger asserts that an external library "exposes API Z" as the seam to build on (e.g. #344's "gpui exposes TextSystem::font_id(&Font) -> Result as the probe"), VERIFY that API is actually `pub` AND callable from our crate BEFORE locking the design — grep its `pub fn`/`pub`, don't trust the claim. #344's central mechanism was FALSE: `TextSystem::font_id` is a PRIVATE `fn` (gpui text_system.rs:109), uncallable from Marley, so the whole probe pivoted to the pub `TextSystem::all_font_names()` membership at design. A claimed-API-existence sentence is exactly the kind of confident sentence to attack: a claim about a library's public surface is verifiable in one grep, and building a design on a private/nonexistent method wastes the whole implement phase. Corollary: also verify the API behaves correctly under the TEST platform, not just production — gpui's `NoopTextSystem` (the `gpui::test` backend) returns `all_font_names()`=[] but `font_id()`=`Ok(FontId(1))` always (echoes), so a resolve-round-trip probe would have read a garbage font as "resolved" in tests (a false pass); `all_font_names` membership is correct under both real-mac and Noop.

## PR-claude-verify-adopted-grammar-by-running-its-query-001
*severity: critical · prevents: BF-claude-adopted-inherits-overlay-query-shipped-alone-blanks-the-language-001*

When adopting a tree-sitter grammar's highlight query, the load-bearing verification is RUNNING the query over a real fixture and asserting the EXPECTED KINDS color — NOT pinning the grammar's closed capture-name set. Two traps a capture-set pin test cannot catch: (1) an `inherits:`/overlay query const (e.g. tree_sitter_typescript::HIGHLIGHTS_QUERY) is INCOMPLETE alone — concatenate the inherited base query first (`format!("{base}\n{overlay}")`, the standard inherits resolution) or the language renders almost blank while the pin test passes; (2) a broad container capture (e.g. `(pair) @property` over a whole key=value) fights an outer-wins/clip-forward span sweep and steals the inner value's color — tree-sitter convention is innermost-wins. Before shipping a newly-adopted grammar: open a real file of it and assert a STRING, COMMENT, NUMBER, KEYWORD, and FUNCTION each color; if the sweep is not innermost-wins, keep any grammar with broad container captures on the hand-lexer floor until the sweep is fixed. A grammar-map critic that reads the .scm AND probes each language earns its keep here — self-review and non-domain critics miss these.

## PR-claude-verify-decoder-parse-order-before-scoping-encode-fix-001
*severity: high · prevents: BF-claude-decoder-unescape-before-split-nullifies-emit-escaping-001*

When adding or switching an ENCODE side to an existing codec (escaping, quoting, framing), do not lock scope to "emit-only" until you have empirically driven the REAL decoder with adversarially-escaped payloads and proven the escape survives its full parse order. Specifically: if the decoder unescapes/decodes BEFORE it splits on structural separators, no emitter escaping of those separators can ever work — the split must happen on the still-escaped bytes (split-before-unescape). A decode clause existing in the spec (e.g. "AnsiCQuoted decode ships") proves the codec arm exists, NOT that the surrounding grammar honors it. Cheapest check: one probe feeding an escaped-separator payload through the real decode fn and asserting the field value round-trips.

## PR-claude-verify-every-specified-trigger-is-actually-wired-001
*severity: medium*

When a ticket/spec names MORE THAN ONE trigger surface for an action (e.g. "click X OR press cmd-Y"), each trigger is a separate deliverable — but only the shared pure decision fn is gate-covered (cov/MSI). The app.rs shim that wires a keybinding/dispatch/click is #[cfg_attr(test, mutants::skip)] + coverage-excluded, and the visual acceptance is a masked human check — so a SPECIFIED-BUT-UNWIRED trigger (e.g. the keybinding shipped without a keymap binding + dispatch arm) leaves every automated gate GREEN while the feature is partially absent. At inspect, explicitly diff the spec's named triggers against the actual wiring: for each promised keybinding, grep the keymap bindings + the dispatch match arm; for each click, grep the render affordance. A trigger with no wiring is a real gap even at MSI 100. Either implement it or move it to the deferred list in the SAME edit — never close silently. (#46: cmd-R was in R50 + REQ-004 + the ticket title but only the ↻ click shipped; the critic caught it by grepping the keymap for a missing "r" chord.)

## PR-claude-verify-helper-side-effects-against-siblings-001
*severity: high · prevents: BF-jump-helper-assumed-to-push-navstack-001*

Before writing "this helper also does X" in a doc/design note, READ the helper — and cross-check its EXISTING callers. If every sibling caller performs X itself around the helper (e.g. all four jump sites capture the origin and `nav_stack.push(NavLoc{..})` after `open_and_place_caret`), that is proof the helper does NOT do X (they would double-do it). Assuming a shared helper's side effects is how a REQ silently goes unimplemented while the doc claims it shipped. The sibling-call-sites are the cheapest, most reliable spec for a helper's actual contract.

## PR-claude-verify-skip-attr-still-attached-after-inserting-fn-001
*severity: high · prevents: BF-mutants-skip-detached-by-fn-insertion-001*

When inserting a new function ADJACENT to one that carries `#[cfg_attr(test, mutants::skip)]` (or any attribute), verify the attribute still binds to its intended fn — an insertion placed between an attribute/doc and its fn silently re-targets the attribute to the new fn. cargo check + cargo fmt both PASS (attributes are valid wherever they land), so it's invisible pre-gate. After adding a fn near a skipped shim, run `grep -n -B1 "fn <name>"` to confirm the skip sits directly above the RIGHT fn, or `cargo mutants --list -f <file>` to confirm the previously-skipped fn is still absent from the live list. A newly-exposed coverage-excluded shim fn = unkillable mutants = MSI RED.

## PR-claude-verify-tree-answers-directly-at-pre-edit-caret-001
*severity: medium*

When a ticket/spec claims "the tree/parser answers X directly" (e.g. "the node kind at the caret separates a lifetime from a char literal"), VERIFY the claimed node actually EXISTS at the caret in the real flow BEFORE scoping it in. For an editor typing flow the tree is parsed from the PRE-INSERT text, so a token the user is mid-way through typing (`'` at `T: |`, a bracket about to be inserted) has NO corresponding node yet — the "just read node_kind_at" plan is false for exactly the position the feature targets. Spike it at plan/design: parse the fixture, probe the pre-insert caret, confirm the node is there. #346: the string/comment slice was clean (the caret sits inside an EXISTING leaf) but the lifetime-bound slice was NOT (no lifetime node pre-insert) → deferred to #362 with an ancestry approach instead. Splitting the ticket on this line shipped the real half instead of shipping something half-broken.

## PR-claude-version-epoch-resets-invalidate-numeric-keyed-caches-001
*severity: medium · prevents: BF-claude-inlay-reload-version-epoch-collision-001*

When an identity component resets to an initial epoch (e.g. a disk reload replacing the buffer resets BufferVersion to initial while re-minting the nonce), every cache or in-flight request key that matches by the NUMERIC identity alone (uri, version) must be explicitly invalidated at the reset site — numeric equality across epochs is a collision, not a match. Audit each stale-guard whose key omits the epoch-carrying component (nonce). Never rely on a polling loop's accidental refetch to self-heal a raced apply: fixing that loop's waste (as #401 did) silently removes the self-heal and promotes the race from transient to sticky.

## PR-claude-version-only-memo-unsound-across-buffer-identity-change-001
*severity: medium*

A cache/memo keyed on `buffer.version()` ALONE is unsound across a buffer-IDENTITY change. A reload/reopen mints a FRESH `Buffer` whose version restarts at 0 (`BufferVersion::initial`), so a stale memo at `Some(0)` collides with the new buffer's `Some(0)` and silently serves pre-change data. When you add a version-memoized field (syntax parse, rendered lines, scroll clamp, …), you MUST also reset/invalidate it wherever the buffer is REPLACED (e.g. `reload_active`) — the same place the `nonce` is re-minted — OR key the memo on `(nonce, version)` like the #268 syntax memo, never `version` alone.

## PR-claude-view-toggles-must-not-corrupt-observation-reads-001
*severity: medium*

When a per-pane VIEW toggle (fold, filter, collapse) changes a shared row/text accessor, check every OTHER consumer of that accessor — especially agent OBSERVATION reads (agent_tail/agent_last_line/Fleet status, #78/#180). A human's view toggle must never corrupt what the brain observes about an agent. Give observation a fold/filter-INDEPENDENT text source (walk the raw blocks) and keep the view-aware accessor only for copy/find/selection (which legitimately follow the visible rows). Grep the shared accessor's call sites before shipping the toggle.

## PR-claude-wheel-delta-truncation-needs-fractional-accumulator-001
*severity: medium*

A scroll-wheel/trackpad handler that converts a pixel delta to integer rows with a plain `as usize`/`as i64` cast silently drops precise (sub-row) scrolling: macOS trackpads emit many small `ScrollDelta::Pixels` events (~0.1 row each) that truncate to 0, so the input feels dead while large flicks / line-based wheels still work. Fix: carry a fractional-row remainder in per-view state and accumulate — `let total = remainder + delta; let whole = total.trunc(); (whole, total - whole)` — so successive sub-row deltas sum to a whole step. Keep the accumulator a pure tested helper; only the per-event state lives in the shim. Same class as any quantize-a-continuous-signal path (drag, zoom, velocity).

## PR-claude-when-you-find-a-hazard-enumerate-every-value-that-reaches-it-001
*severity: high · prevents: BF-claude-hunted-the-hazard-for-one-token-and-never-checked-the-other-001*

When you find a hazard in a parameterized code path, you have found ONE INSTANCE, not the hazard. Before you record it as an accepted decision, ENUMERATE EVERY VALUE THE PARAMETER ACTUALLY TAKES and re-run the hazard against each. On #299 I discovered at plan time that the comment token `//` makes `/// doc` look "already commented" (it starts with `//`), so ⌘/ shreds it to `/ doc`. I wrote a whole hazard section, recorded it as a decision with its own pinning test, and moved on — having checked exactly ONE of the token table's values. The OTHER token, `#` (Shell and TOML — two of the three supported languages), was worse: `#!/bin/sh` → `!/bin/sh`, which stops the script executing SILENTLY, at exec time, not compile time. And back on `//`, `//!` — a Rust MODULE DOC, the first line of literally every file in the crate — was shredded too. Five irreversible cases; I had looked at one. THE TEST: for any hazard found in a fn taking a token/format/language/mode/flag, write down the COMPLETE set of values that reach it in production (grep the call sites and the table), and evaluate the hazard for each. If the set is open (user-supplied), evaluate the classes. A hazard "accepted as reference parity" for one value is not accepted for the others — and the blast radius is rarely uniform (a mangled `///` is a compile error the toolchain catches in seconds; a mangled `#!` ships). COROLLARY, and the reason this was caught: the critics found the `#` half PRECISELY BECAUSE I had blessed the `//` half in writing. A recorded decision makes the asymmetry visible. Record your decisions even when you think they are complete — especially then.

## PR-claude-widen-the-fuzz-generator-to-the-state-you-just-made-legal-001
*severity: high*

A fuzzer's GENERATOR can exclude the exact state your change just made reachable — and then it will pass 50,000 steps while proving nothing about the new code. On Marley #297 the merge rule changed so that ADJACENT selections became legal for the first time; the differential fuzzer's generator advanced `cursor = end + 1`, forcing a gap of ≥1 between every pair, so in 50k steps it never once produced two adjacent members. It was validating the equivalence claim only over the region the change did NOT touch, and the load-bearing fix for the new state (canonicalizing a delete that lands two cursors on one offset) was guarded by nothing — reverting it left the entire gate green. THEREFORE, whenever a change makes a previously-impossible state POSSIBLE: (1) go read the fuzzer's GENERATOR, not just its assertions, and ask "can this even produce the new state?"; (2) widen it, and expect it to immediately find something — mine failed on the first run after widening; (3) add a hand-written fixture for the new state too, because a generator can drift back; (4) remember cargo-mutants will NOT save you here — it does not mutate method calls, so a `from_selections`→`from_members` regression has no mutant at all. Coverage, mutation and fuzzing each have blind spots, and they overlap: the new state was in none of them.

## PR-claude-window-scoped-capture-cannot-prove-input-landed-001
*severity: critical*

`screencapture -l<winid>` captures the target window CORRECTLY EVEN WHEN IT IS FULLY COVERED by another app — so a window-scoped capture can never tell you whether your synthetic input reached that window. Synthetic CGEvents go to whatever is FRONTMOST at those screen coordinates. The failure mode is silent and dangerous: the capture looks plausible (just "nothing happened") while the keystrokes land in another application. Real case (#313): a concurrent Claude agent's iTerm2 window came frontmost and covered Marley; `.` keystrokes went into THAT AGENT'S PROMPT while the window capture kept showing a pristine Marley. When driven input appears to do nothing, take a FULL-SCREEN capture (`screencapture -x`) — it is the only thing that shows layering and frontmost — and check `osascript -e 'tell application "System Events" to get name of first process whose frontmost is true'`. Better: assert the app's OWN state (a wire log, a file on disk) rather than pixels, so a no-op cannot read as a pass. If you did pollute another session, verify the exact state with a full-screen capture before repairing it, repair narrowly, and re-capture to confirm.

## PR-claude-windowed-query-splice-equivalence-001
*severity: high*

When windowing a tree-sitter query to splice fresh in-window captures into cached out-of-window captures, the spliced set equals a full re-walk (as a MULTISET) ONLY IF: (1) the window ⊇ changed_ranges ∪ the edit span; (2) a grow-loop widens the window and RE-QUERIES until every fresh span lies fully inside it — this absorbs BOTH single-capture straddlers (a multi-line block comment) AND multi-capture matches that yield distant out-of-window captures (QueryCursor yields ALL captures of any match intersecting the range); (3) cached spans TOUCHING the replaced region [start,old_end) or OVERLAPPING the window are dropped, and the survivors rebased by the edit delta (before start: identity; at/after old_end: +（new_end-old_end)). Gate with an equivalence corpus that includes a block-comment cascade (changed_ranges ≈ whole file → window grows to full → degrade to the full walk) plus straddle/boundary/deletion edits — a wrong window FAILS the corpus (GATE RED), never ships wrong colors. Verify set_byte_range = INTERSECTING (not contained) against the pinned tree-sitter version; the grow-loop keeps correctness robust even if that assumption is wrong.

## PR-claude-workspace-states-is-unordered-001
*severity: medium*

`PaneGrid::states()` (and any `HashMap::iter()`) yields panes in ARBITRARY, per-process-randomized order — NEVER assume "first" / "ascending PaneId" / any order from it. If a feature needs ONE deterministic pane, collect + `sort_by_key(|(id,_)| id.0)` (PaneId is not Ord) then take first; if it needs ALL panes, make the logic order-INDEPENDENT (union + sort/dedup, like #289's open_file_diagnostic_rows). A comment claiming an order that the container doesn't provide is a bug even when a single-pane test/drive passes — the multi-pane path is where it bites, and pure/single-pane tests won't catch it (it lives in the shim's pane enumeration). Verify multi-pane selection with a driven or headless multi-pane case.

## PR-claude-write-first-not-research-first-known-api-exemplar-001
*severity: medium · prevents: BF-claude-subagent-research-paralysis-on-slow-compile-dep-001*

When implementing a new crate/module over a KNOWN external API that has a PROVEN in-repo exemplar (e.g. a spike using the same gpui/dep), do NOT delegate with "confirm the API" framing — a subagent will loop recompiling a slow dep to research signatures and never write code. Instead: (a) extract the exemplar's exact calls FIRST + a targeted signature grep of the specific types you'll touch, then (b) WRITE all files against those calls, then (c) `cargo check` and fix ONLY the errors the compiler reports, iterating. Write-then-check beats research-first: the compiler is the fastest, most authoritative API oracle. If delegating, give the subagent the exemplar's calls INLINE + a hard "write the no-dep pure modules FIRST, then iterate on the shim; do NOT pre-research the API" constraint, and TIME-BOX the API-fixing. For a UI/gpui shim specifically, the pure wiring modules need ZERO API knowledge — always land those first for guaranteed progress.
## PR-claude-decode-jsonb-strings-twice-and-assert-shape-001
*severity: medium · prevents: F-claude-export-joined-double-encoded-jsonb-as-chars-001*

When exporting a jsonb column, never assume one decode: a jsonb value of type
STRING can itself contain serialized JSON (a writer-side double-encoding defect
that the source system tolerates invisibly). After decoding, ASSERT the shape you
need (`isinstance(x, list)` before joining; `isinstance(x, dict)` before keying) —
`", ".join()` over a str is silent character soup, not an error. And verify
exports by DIFFING the rendered output against the source rows (full-corpus when
cheap), not by counting blocks: counts catch loss, only diffs catch mangling.
## PR-claude-drift-smoke-revert-surgically-001
*severity: low · prevents: (TICKET-409 validate near-miss)*

A drift smoke (inject a bad line → assert the check reds → revert → assert green)
must revert SURGICALLY (delete the injected line: `sed '$ d'`, or a targeted
patch) — never `git checkout -- <file>` — whenever the target file carries
UNCOMMITTED work. Checkout restores HEAD and silently clobbers the in-flight
edits along with the injection; the smoke then reports a false RED on the
post-revert assert (or worse, passes while your work is gone). Symptom to watch:
the revert-side count matches the inject-side count.

## PR-claude-dead-code-analysis-designs-the-rip-keep-list-001
*severity: medium*

When a rip keeps "generic seams for the future", run the dead-code analysis as a DESIGN input before locking the keep-list, not as a compile surprise after: at `-D warnings`, a kept `pub(crate)` fn (or a pub fn in a private module) needs a live non-test CALLER, and a kept enum needs a live non-test CONSTRUCTOR for every variant — tests don't count, and a `pub use` from the crate root silently exempts an item (which hides a cold seam rather than justifying it; roster such exemptions deliberately). The workable patterns, in preference order: (1) keep the seam CALLED with a constant input at the surviving call site (mount_plan(None), demo_feed_permitted(false)); (2) route the terminal through the STANDING async plumbing so the fold fns keep their caller (the #411 no-transport sends through the result channels); (3) COLLAPSE what only the dead producer could construct (ConnectionState's wire suffixes) — re-homing a type nobody constructs is preservation theater that -D warnings rejects. Applied at #410/#411; proven by F1's mid-implement flip from re-home to collapse.

## PR-claude-walker-yield-spelling-must-match-store-keys-001
*severity: medium · prevents: F-claude-search-walk-verbatim-root-misses-canonical-override-keys-001*

A filesystem WALKER (ignore::WalkBuilder, read_dir recursion, glob) yields paths spelled under its WALK ROOT — it never canonicalizes. So any store keyed by one normalization and PROBED with walker-yielded paths must be given a walk root in the SAME normalization, or every probe misses under an aliased root with no error (the lookup just "finds nothing"). This is the producer-side twin of the both-sides-normalize store rule (#310 F1): equality/prefix/strip CONSUMERS are greppable (`==`, `starts_with(`, `strip_prefix(`), but a walker feeding a `HashMap::get` looks innocent because the compare is hidden inside the map. When canonicalizing an identity: after sweeping consumers, list every ITERATOR/WALKER whose yields become probe keys (searches, indexers, file watchers, batch appliers) and hand each the canonicalized base. Caught at #319 inspect: the search override map (dirty-buffer-wins) went always-miss under symlinked roots while every direct compare in the diff was sound.

## PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001
*severity: medium · prevents: F-claude-git-marks-close-leak-sibling-of-editor-folds-001*

When you add (or review) a LIFECYCLE SCRUB for one per-path / per-content map (a
close-clears-entry, a last-view-drop scrub), sweep EVERY SIBLING map keyed the
same way in the same struct before closing the ticket — the decl block around
the map you're fixing is where its siblings live (`git_marks` sat five fields
from `editor_folds`, same key spelling, same missing scrub, invisible to every
test because a leak has no render). For each sibling: same lifecycle → scrub in
the same choke (or ticket it explicitly); different lifecycle (self-healing,
generation-keyed, bounded ring) → write WHY it needs no scrub into the review
notes. The sweep is one grep for the struct's `HashMap<PathBuf`/`HashMap<...Id`
fields plus a remove/retain/clear cross-grep per field — minutes, and it turned
a "done" ticket into a found sibling leak at #353 inspect.

## PR-claude-identity-claims-over-display-maps-probe-zero-width-boundaries-001
*severity: low · found at: #333 inspect (two critics independently, live probes)*

Any "byte-identical" / "same output" claim between a computation done DIRECTLY
on a display string and the same computation done on the raw line THEN mapped
through the display-column layer must be probed at ZERO-WIDTH char boundaries
before it goes in a doc comment or an AC. The column maps deliberately refuse
to make an empty cell window a boundary (`cols_to_bytes`: a zero-width char is
never a band boundary; its pixels ride the preceding base cell), so a combining
mark immediately AFTER a token is absorbed into the mapped span while the
direct computation leaves it outside — the two agree on every ASCII/tab/CJK
case and diverge exactly there. Tabs, wide glyphs, and multibyte are the
boundary cases everyone probes; zero-width is the one that hides (its sibling
class: #336 chars-vs-cells, #340 byte-vs-char — domains that coincide on easy
input). Pin the divergence as a deliberate-difference test (grapheme cluster
stays one styled run — usually the BETTER behavior), don't paper it with
"identical" prose.

## PR-claude-per-path-scrub-sweeps-derived-key-twins-001
*severity: medium · extends: PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001*

When scrubbing a per-path / per-content map on lifecycle close, sweep the map's
DERIVED-STATE TWINS too — cache keys, memo keys, "last computed for" markers
(`Option<(PathBuf, gen)>`, `(path, version)` tuples) that NAME the same path —
not just sibling maps. A key that outlives its entry INVERTS the failure mode:
the map leak was memory-only, but a dangling key short-circuits the recompute
gate (`cached == fresh → early return`) over the now-empty map, turning the FIX
into a user-visible wrong-render the leak never had (Marley #412: close an
active dirty file, reopen with no gen bump in between → unmarked gutter).
Checklist per scrubbed map: grep the struct for key tuples naming the same
domain; for each, clear it with the entry (same guard) or write down why it
cannot dangle. And before trusting "some event re-mints it anyway", enumerate
that event's ACTUAL producers — #412's "activation bumps the gen" belief was
folklore; the three real bumps were save/reload/commit, none firing on reopen.

## PR-claude-shared-set-derivations-must-be-one-function-001
*severity: medium · prevents: silent divergence of consumers that must see the same set*

When an invariant says two consumers act on THE SAME derived set ("host creation derives from the same open-document set reconcile syncs" — Marley #321), the derivation must be ONE function both consumers call, never N textual copies of the filter chain. Copies satisfy the invariant only while every future edit touches all of them — #320's inspect found the root-filtered editor iteration hand-copied 4× (two pump passes + two override snapshots), where editing one copy would silently split the ensured set from the synced set with no compile error and no test catching the drift (both sides remain individually plausible). Extract the shared derivation onto the NARROWEST owner — a method on the field's own type (`ContentRegistry::editors_under`), not the enclosing view — so it also preserves field-disjoint borrow access for callers that hold `&mut` to sibling fields. Smell: the same `.iter().filter_map(..).filter(..)` chain pasted with a comment claiming it matches another site.

## PR-claude-mutants-baseline-builds-only-the-tested-package-001
*severity: medium · prevents: gate:5 fail-closed baselines and gate:4 dead-branch reds on cross-crate test fixtures*

cargo-mutants runs its BASELINE (and every mutant) in a pristine COPY of the source tree, building `--package=<the tested package>` ALONE — so an artifact another crate's build produces (a fixture bin like `fake_ls`, forced in the repo tree by a sibling crate's `env!("CARGO_BIN_EXE_…")` integration test) does NOT exist in that copy, and a test that needs it fails the unmutated baseline → cargo-mutants exit 4 → the gate fails CLOSED with "0 mutants tested". The tempting fixes each break a different gate: a conditional build-if-missing fallback leaves forever-uncovered lines in the normal run (gate:4, 100% lines); a loud assert-with-remedy fails the baseline (gate:5); `std::process::Command` in test code is a disallowed type (gate:2). The shape that satisfies all three at once: build the fixture UNCONDITIONALLY through the `marley_command` adapter (`blocking::Command::new(env!("CARGO"))…status()`) — an up-to-date no-op in the repo run, a real build in the mutants copy, every line always executed. `CARGO_TARGET_DIR` inheritance keeps the build and a `current_exe()`-derived resolver agreeing under llvm-cov's target redirect. Found in #320 across two consecutive red gate cycles.

## L-claude-assert-messages-implicit-capture-for-coverage-001
*severity: low · category: test mechanics · from: pipeline 320 gate:4*

Under a 100%-line coverage bar, `assert!(cond, "… {}", expr)` with TRAILING format args is a trap: rustfmt wraps the args onto their own lines, and those lines hold code that executes ONLY on assertion failure — a permanently uncovered line in a passing suite (#320's gate:4 red was exactly one such `bin.display()` line). The drives' standing idiom avoids it structurally: bind the value first (`let at = bin.display();`) and use IMPLICIT capture in the format string (`assert!(cond, "… {at} …")`) — a string literal carries no code region, so nothing failure-only ever owns a line. Same applies to `expect()`/`panic!` message construction on must-pass paths.

## PR-claude-teardown-must-deliver-not-destroy-the-outcome-queue-001
*severity: high · prevents: F-claude-teardown-clear-eats-queued-terminal-outcomes-001*

When an invariant of the form "every X terminates with exactly one outcome" is
added to a system, re-audit every path that BULK-RESETS the state carrying
those outcomes — bulk resets written BEFORE the invariant existed are its
likeliest violators, because they were designed under "this state is
disposable" semantics. Concretely: a teardown that owns a delivery queue whose
consumers have their own staleness guards must DELIVER the queue, not clear it
— staleness is the consumer's job (its key-guards already exist and the
delivery layer's own comments usually say so); destruction is nobody's job, and
it silently reopens the never-terminates hole one layer deeper. Wire nuance
that makes this reachable, not theoretical: a channel reports buffered data
BEFORE the disconnect (std mpsc `try_recv` yields all `Ok`s, then
`Disconnected`), so "final answers + death" arrive in ONE batch and the answers
are already queued when the teardown runs. Corollary: if a plan-time decision
locks an ORDERING around such a clear ("push after the clear so it isn't
eaten"), treat the clear itself as the suspect — the safe shape is usually to
remove the destruction, not to sequence around it.

## PR-claude-single-slot-latch-supersede-must-settle-not-discard-001
*severity: high*

When a SINGLE-SLOT latch parks CONSENTED deferred work (a ⌘S save, a queued send, an acknowledged action) and a new request wants the slot, check the occupant's TARGET before clearing: a SAME-target occupant is superseded (the newer request subsumes it — one action, the newer wins), but a DIFFERENT-target occupant is someone else's consented work — SETTLE it (complete it per its own fallback semantics: plain-save, flush, fire) before parking. An unconditional `slot = None` at the head of "begin" silently swallows the first consent with no error, no flash, no fallback — and the loss only fires on the path where BOTH requests use the latch (the tell in #354: ⌘S on a non-Rust file bypassed the latch and the parked save survived; ⌘S on a Rust file killed it). Trace the second-consent path (the user re-invokes the same verb on ANOTHER target while one is parked) explicitly in design — the single-target supersede trace passes while the cross-target loss ships.

## PR-claude-version-epoch-guards-invalidate-on-reload-001
*severity: high · prevents: F-claude-354-reload-epoch-collision-formats-stale-save-001*

Any in-flight request guard keyed on a `BufferVersion` (formatting, inlays, any "drop if the buffer moved" check) MUST be invalidated when the buffer RELOADS: reload re-mints the identity nonce but RESETS the version epoch to 0, and a clean-at-open request also carries v0 — so `version == key.version` passes for a response computed against the PRE-reload text (0 == 0), and stale edits splice into the reloaded buffer (worst case: a parked save then writes the corruption to disk). The version alone cannot distinguish epochs — the key carries no nonce. At every reload site, clear each version-keyed in-flight field (the #401 inlay fix; #354 added `formatting_request`) — or key on `(nonce, version)`. When ADDING a new version-keyed request kind, grep the reload path(s) and add its clear in the same commit; this class has now recurred twice (#401 inlays, #354 formatting).

## PR-claude-undefer-sweeps-test-comments-and-assert-messages-001
*severity: medium · category: docs/consistency · status: active*

When a change UN-DEFERS a documented limitation (a "deferred / NOT covered / CANNOT be done / see #ticket"
claim anywhere in the crate), the doc fix is a SWEEP, not an edit: the deferral story is told at every layer —
fn docs, enum-variant docs, module headers, AND `#[test]` header comments AND `assert!` message strings — and
they go stale in packs. Grep the whole crate for the deferral's vocabulary ("defer", "NOT covered", "CANNOT",
the superseded ticket number) at implement when making the un-defer edit, and re-grep at inspect. The class has
fired twice on the same seam: #362 updated `blocked_quote`/`Context` docs but its inspect had to catch the stale
`pair_action` fn doc (MEDIUM); #366 updated all three fn docs but its inspect had to catch the
`t338_lifetime_guard_does_not_reach_the_bound_positions` TEST comment + assert message still claiming the bound
positions "CANNOT" be covered "(#315)" — a direct contradiction of the shipped behavior, hiding in the tests
module where doc passes don't look.

## PR-claude-uri-equality-is-not-instance-identity-001
*severity: high · category: editor/LSP state routing*

A guard that compares path-derived spellings (uri, canonical path) proves FILE identity, never INSTANCE
identity. Marley opens one file as multiple instances (nested-root twins, D-OPEN-DEDUPE-SCOPE) with
independent version counters, and `canonical_under_root`/`uri_for` are TOTAL over absolute paths
(pass-through, never None) — "resolved under root X" adds nothing for stored-absolute paths, and two hosts
spell the same file identically. Any consumer keyed or guarded by uri/path that can face twins must ALSO
compare the instance's ROOT (or carry a request-time ContentId — the #354 D8-ORIGIN-ID pattern), and any
edit-applying path must route to the instance whose HOST validated the version (the owning root), never a
deterministic-but-wrong all-roots pick. Check the failure direction while you're there: a version lookup
that MISSES yields None and `version_conflict(_, None)` fails OPEN (applies), so a wrong-root lookup is an
apply-with-wrong-encoding bug, not a safe skip. When a dispatch loop gains a new axis (which root / which
instance), re-derive every consumer's guard against the NEW axis — #413's inspect caught two HIGHs this
way after the design sweep had wrongly cleared them, and the same sweep discipline applied to the REMEDY
(latch-untouched drop) is what kept the colliding-twin re-ask lifecycle correct.

## PR-claude-never-type-blind-into-the-live-app-001
*severity: high · category: selftest harness*

Never send `type:`/backspace into the live app without FIRST verifying, from a fresh capture, WHICH
surface holds keyboard focus and where the caret is. On #318 a "type hello then 5 backspaces" focus
probe assumed a terminal prompt; the restored workspace actually had an EDITOR focused (the user's own
open keymap.rs), so the probe typed into their buffer and the blind backspaces ate five neighboring
characters — and a later "the caret looks like it's after the junk" read of a screenshot was ALSO wrong,
merging two lines. Repairs: undo-to-floor converges IF the user made no edits of their own (verify the
DISK file is git-clean first, then spam spaced ⌘Z and crop-verify the restored text + a 0-error
diagnostics footer). Rules: focus clicks go to a coordinate whose worst case is a caret move (editor
code area — never tree rows, never unknown panes); text-entry tests belong in scratch fixtures, not the
user's restored workspace; after ANY accidental buffer touch, verify cleanup with a cropped screenshot
read, not memory of what the keys "should" have done.

## PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001
*severity: high · prevents: F-claude-418-selection-invariant-only-lived-in-one-components-handlers-001*

When a UI invariant is "exactly one of these N rows/items is marked" and the mark derives
from MULTI-WRITER shared state, do not compute per-row booleans (`selected = a && b && c`
repeated per row kind) — one stale field lights two rows or none, and every new writer
anywhere in the app is a new way to break it. Instead: (1) derive ONE selection value — a
discriminated union computed in one place that mirrors the app's authoritative routing
order (in the POC: `activeAgentId` outranks `activeSection`, project-2 outranks both —
copy the CENTER's router, don't invent a second precedence); a single value structurally
cannot name two rows. (2) CLAMP stale inputs in the derivation (out-of-range index → null,
a mode flag without its data → default) and give incoherent states an explicit fallback
selection (the workspace/overview row) so zero-fill is unrepresentable. (3) Route the
component's own mutations through ONE `select…()` helper that owns the reset discipline,
so a new handler can't forget a reset. The per-row-boolean shape passes every single-click
test and fails only on cross-surface sequences — test those (foreign writer, then look at
the rail). Rust-side twin: the #419 port's `rail_rows` faces the identical choice; port the
selector, not the booleans.

## PR-claude-invariant-rewrites-must-reaudit-redundancy-consumers-001
*severity: medium · prevents: F-claude-419-force-expand-protected-a-breadcrumb-not-the-selection-001*

When a change REDUCES a redundant UI signal to a single source of truth (six lit rows → one
selected row; several markers → one dot), every mechanism that was CALIBRATED against the old
redundancy must be re-audited, not just kept "byte-identical". A guard that "preserves existing
behavior exactly" can still break the new model: it was correct only because redundancy masked
its gaps (any of six lit rows satisfied "the active thing stays visible"; with one, the guard
must protect exactly THAT one). Concretely: enumerate every reveal/force-expand/auto-scroll/
protection mechanism keyed to the old signal and ask "which row does this protect NOW, and is
that the row that matters?" — at #419 the #386 force-expand was correctly preserved, but the
#398 active-section override inside it protected a breadcrumb row while the selection sat in a
collapsible section. The inspect lens that catches this class: trace COLLAPSE/visibility
interactions against the new single source, state by state.

## PR-claude-absence-asserts-die-silently-under-projection-changes-001
*severity: medium · prevents: F-claude-420-projection-change-turned-a-red-pin-vacuous-not-red-001*

When a change alters what a projection EMITS (labels, row kinds, shapes), every test that
asserts the ABSENCE of an old-shaped item is suspect-vacuous, not suspect-red: `!rows.any(|r|
r.label == "old-shape")` passes forever once nothing emits that shape, proving nothing. At any
re-pin pass: (1) enumerate the tests over the changed projection and classify each assert as
PRESENCE (goes red honestly) or ABSENCE (goes vacuous silently); (2) re-pin ABSENCE asserts to
the new vocabulary explicitly (assert no `RailLevel::File` row — a LEVEL, not a label, where
possible: levels are type-checked, labels are stringly); (3) never drive a re-pin from the red
list alone — diff the DECLARED-affected set against the actual reds and investigate every test
that "should have" gone red but stayed green. Prefer structural predicates (level/kind/coordinate)
over label greps in new pins — labels change with projections; levels change with compiles.

## PR-claude-splicing-a-fn-between-doc-attr-and-item-steals-them-001
*severity: medium*

Rust outer docs (`///`) and attributes attach to the NEXT item. Splicing a new fn between an
existing fn's doc/attr block and that fn silently re-attaches BOTH to the new fn: the sibling
loses its rustdoc AND any load-bearing attribute — at Marley #421 that was
`#[cfg_attr(test, mutants::skip)]`, so an intentionally-masked shim would have started chewing
missed mutants on the next diff that touched it, and the new fn led with the WRONG doc. It
compiles clean; nothing warns. Rule: when inserting near an attributed item, insert ABOVE the
whole doc+attr+item unit or BELOW the item's closing brace — never between; at inspect, `grep
-B3 "fn <sibling>"` and confirm doc↔attr↔fn adjacency for every neighbor of an insertion.

## PR-claude-a-pure-helper-inherits-its-callers-coordinate-world-001
*severity: high · prevents: F-claude-422-pure-walk-right-call-site-wrong-twice-001*

A pure geometry/layout helper is only as correct as the WORLD its caller hands it: the bounds
(inset? band-offset?), and the gate (which app state makes the render arm live at all). When a
shim call site pairs a new pure helper with an existing renderer (panes + their dividers, rows
+ their overlays), inspect must line the TWO call sites up literally — same bounds BINDING
(hoist one `let` and share it, never re-derive), same gating predicate (copy the sibling's
condition, not a lookalike accessor whose docs admit a fallback). The two #422 highs were
exactly these: a re-derived bounds missing an inset, and `try_workspace()` (documented
fallback: another tab's grid) standing in for `active_is_terminal`. Test-side rule: an
agreement unit (helper output vs the sibling renderer's output OVER THE SAME INPUTS) catches
the drift class the pure suites structurally cannot.

## PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001
*severity: low · category: capture protocol / browser driving*

When key-driving an overlay whose rows select on `mouseEnter` (the POC's OverlayRow; any
hover-selects list) via Playwright/CDP, PARK THE POINTER OFF THE CARD FIRST (e.g.
`mouse.move(40, 900)` into inert chrome). Chromium synthesizes mouse events after DOM
changes under a stationary cursor, so a repaint that puts a row under the parked pointer
fires `mouseenter` and STEALS the selection right after your keystroke — the #416 BEFORE
capture read "row 3 selected" after one ArrowUp (neither wrap's 24 nor clamp's 0) until the
mouse moved off-card, after which the same drive read the true wrap-to-24. The steal is
invisible in the screenshot alone (it looks like a plausible selection); only the
off-by-everything index exposed it. Corollary: prefer DOM-query asserts (which row carries
the selected class, by index) over eyeballing the highlight — the index mismatch is what
caught it.

## PR-claude-fire-and-forget-spawns-still-reap-001
*severity: medium · prevents: F-claude-226-fire-and-forget-spawn-never-reaped-zombie-accumulation-001*

"Fire-and-forget" is about not BLOCKING, never about not REAPING. Dropping a
std::process::Child does not reap it (documented std behavior; the workspace sets no SIGCHLD
disposition), so a spawn-and-drop leaves a zombie until the parent exits — monotonic in a
long-running app, and the failure mode when the per-uid process cap fills is app-wide fork
failure, not a local error. Every non-waited spawn takes the detached-reaper shape:
`let mut child = cmd.spawn()?; std::thread::spawn(move || { let _ = child.wait(); });`
(the app.rs reap-thread / lsp wait-on-drop house pattern). Grep-check when adding any spawn
that outlives its call: `spawn().map`, `let _ = cmd.spawn()`, or a dropped `Ok(child)` are
the tells. Known latent instance: `open_url_with` (rare-fire; owned by the next ticket that
touches it).

## PR-claude-423-a-fixture-path-playing-two-roles-splits-loudly-001
*severity: medium · prevents: the #423 boot-call index-out-of-bounds class*

When a test fixture uses ONE path for two roles by accident (the #423 twin-alias
tests used the tempdir as BOTH the settings/config home AND workspace root_a),
any refactor that splits the roles (a symlink alias, a subdir, a rename) must
AUDIT EVERY CONSUMER of the original path and re-bind each to the role it
actually meant — `boot(cx, &root_a)` silently meant `boot(cx, <config dir>)`
only while the two were the same path, and the split turned it into an
empty-config boot (`index out of bounds: len 0`). The grep is cheap: every use
of the old binding, classified config-role vs root-role, BEFORE the split
compiles. Corollary: run the touched tests IMMEDIATELY after such a rewrite —
the failure is loud and instant, but only if you look.

## PR-claude-407-an-imported-verdict-needs-artifact-derived-belts-not-attestation-001
*severity: high · prevents: the stale/partial-import false-green class (407 inspect)*

When a gate accepts an IMPORTED measurement (outcomes computed elsewhere — a
dev-box mutation sweep, a remote bench, any pulled artifact) in place of
running the tool locally, an operator-typed provenance claim is NOT a check:
symbolic refs make `attested == HEAD` a tautology (`MUT_OUTCOMES_SHA=HEAD`),
and honest staleness (yesterday's pull, a forgotten push) sails through.
Every belt must derive from the ARTIFACTS or the REPO, not the operator's
memory: (1) demand a literal hex sha and resolve it; (2) an
importer-side identity sidecar, enforced when present, authoritative once the
producer writes it; (3) freshness — the artifact mtime must not predate the
commit it claims to measure (rsync -a preserves producer mtimes); (4) tree
identity — a file the artifact enumerates must exist in HEAD; (5)
COMPLETENESS — the artifact's own enumeration manifest (mutants.json) must be
EXACTLY covered by its outcomes (set equality, which also mechanically
enforces any merge recipe); (6) the measured surface must be clean in the
worktree. Each belt is one guard clause; together they make the imported
green as trustworthy as a local run under §15's posture (accidents blocked;
deliberate fabrication stays out of scope by design).

## PR-claude-426-segment-math-eats-only-phantom-aware-columns-001
*severity: high · prevents: the F-#352 class re-emerging on the column axis*

When a display column is mapped into a COARSER space (a wrap segment, a display
row, an excerpt), the input column must come from the SAME layout family the
coarse map was built over. Marley has two column domains per line — the
phantom-less `line_layout` and the phantom-aware `line_layout_with_inlays` — and
before #426 several sites (caret paint, ⌘K hover, completion/signature/rename
anchors, IME rect) computed phantom-less columns as a bounded x-nuance. Feeding
those into phantom-aware `seg_of_cell` turned a sub-cell offset into a WRONG
DISPLAY ROW. The rule: a new consumer that maps columns across spaces first asks
"which layout produced this column?" and routes through the aware accessor
(`aware_display_col` / the rim's own layout) — never mix families across a
segment/row boundary. Corollary: a field holding a column (`HoverCard.anchor_col`)
must have ONE domain across all writers; two writers in different domains is the
defect even before a reader breaks.

## PR-claude-427-a-registry-insert-owns-view-one-001
*severity: high · prevents: the F-claude-427-b leak re-emerging at every new Content kind*

The ContentRegistry contract: `insert()` ACQUIRES the first view. A new kind's
open/materialize path takes NO extra `acquire_view` — the tab's view is the
insert's view. An acquire beside an insert is correct ONLY in the pinned-anchor
shape (the cockpit append), where the anchor owns the insert view and the tab
needs its own. Review grep: `acquire_view` within a few lines of
`.insert(Content::` is a leak until an anchor is shown to hold the first view.

## PR-claude-427-b-insert-new-items-outside-doc-attr-blocks-001
*severity: medium · prevents: the F-claude-427-c class (docs/attrs/masks silently re-attaching)*

When adding a new item beside an existing one, never split an item from its doc
comment or attributes: insert ABOVE the neighbor's doc block or BELOW its
closing brace, then RE-READ the seam — the neighbor's first doc line and its
`#[cfg_attr(test, mutants::skip)]` must still sit directly above the neighbor's
own `fn`. Attribute re-attachment produces no compiler signal; mutation masks
and rustdoc move silently.

## PR-claude-427-c-a-dropping-persist-writer-reindexes-001
*severity: medium · prevents: restore focusing the wrong tab for every future transient kind*

The first writer arm that DROPS items from a persisted list breaks every
persisted INDEX into that list (#427: `active_tab` counted live tabs; the file
holds survivors). When a writer filters, every index it persists must be
re-derived by counting SURVIVORS before the live index, clamped to the
persisted list — restore-side clamps cannot correct the shift, only bound it.

## PR-claude-428-a-new-editable-surfaces-register-their-own-input-seam-001
*severity: critical · prevents: F-claude-428-a re-emerging on every future editable surface*

A gpui surface that wants OS text input (typing, IME) must render its own
paint-scoped `canvas` calling `window.handle_input(&focus,
ElementInputHandler::new(bounds, entity), app)` — the registration is
per-frame, per-surface. Adding arms to the `EntityInputHandler` methods alone
leaves the surface mute. Review check for any new editable surface: where is
its `handle_input` canvas? And the validate drive must include a LIVE typed
character, not only direct method calls.

## PR-claude-428-b-shared-state-borrowed-across-surfaces-restores-itself-001
*severity: high · prevents: the F-claude-428-b class (a borrowing surface corrupting the owner's state)*

When surface B edits state owned by surface A's views (a buffer's live
`SelectionSet`, a scroll offset, a composition span), B saves the state before
installing its own and restores it after — inside the same operation. Do not
lean on A's park/restore chokes: they key on A's OWN transitions and do not fire
for B's frames. (The undo record is exempt — it captured its restore set at
edit time.)

## PR-claude-429-a-gestures-verify-their-own-preconditions-not-ambient-state-001
*severity: high · prevents: the F-claude-429-a class on every future bulk gesture*

A bulk gesture (replace-all, apply-fix-all, save-all) must verify its OWN
preconditions before touching buffers: (1) the surface it targets actually
opened (check the open call's success signal, never `active_*()` afterwards);
(2) its input set is COMPLETE (a streaming producer needs a Done flag the
gesture checks); (3) its match semantics equal the PREVIEW's (one shared
filter for what counts as a hit). Each miss produced a silent-rewrite path in
#429; each guard is one line.

## PR-claude-fail-closed-claims-enumerate-every-input-lane-001
*severity: high · category: input routing / surface ownership · from: pipeline 432*

**Rule:** when a surface claims its input ("while tab X is active, keys never
reach hidden surface Y"), enumerate and claim EVERY lane — modifier chords (ALL
variants, not just unshifted), plain printables, motion/scroll keys, and the
platform IME path — and trace each lane to its terminal sink in the ladder.
**Why:** #432 closed the ⌘V lane and inspect immediately found three siblings
open: ⌘⇧V (a `!shift` gate), every plain printable (double-delivery into the
hidden PTY via the R29 tail), and the mirror-image gap on the editor surface.
A fail-closed claim scoped to the one lane that prompted it is a leak inventory
waiting to be taken. **How to apply:** the claim's arm consumes by SURFACE
(`active_*_id()` gate), not by key list, wherever the platform path allows;
where a bare `return` must coexist with IME delivery, do NOT stop_propagation;
and the validate drive asserts the hidden sink's buffer UNCHANGED for every
lane (chord, shifted chord, plain char, motion), not just the headline one.

## PR-claude-row-prefix-mirrors-move-in-lockstep-001
*severity: high · category: editor render geometry · from: pipeline 434*

**Rule:** the editor row's leading cells (git lane, the #434 runnable ▶,
gutter digits, their gaps) are MIRRORED by at least two off-row compositions —
`h_scroll::code_area_left_px` (the thumb's "aligned by construction" origin)
and the sticky-header band's hand-built spacer prefix. Any change to the row
prefix updates EVERY mirror in the same diff, and the mirrors' pinned tests
(t341) get re-pinned — a passing stale pin is the trap, not the safety net.
**Why:** #434 added one reserved ▶ cell; both mirrors silently drifted a
cell+gap left, and t341 kept passing because it pinned the stale composition.
**How to apply:** grep `code_area_left_px` + `sticky_header_overlay` (and any
future prefix consumer) whenever the row build's leading children change; the
inspect checklist for gutter tickets names this sweep.

## PR-claude-unmodified-terminal-chords-yield-to-the-pty-001
severity: high — silently breaks TUI programs
An UNMODIFIED key bound in the Terminal key context steals that key from the
PTY: the keymap arm resolves BEFORE the R40 raw route, and R40's contract is
"while a full-screen program owns the alternate screen, stream EVERY key"
(mc binds F8=delete; vim/htop bind F-keys). Every pre-#435 Terminal row was
⌘-modified — never PTY-relevant — so the ladder order had no gate.
**Rule:** any Terminal-context keymap row on an unmodified (or
shift-only/alt-only) chord MUST fall through to the raw route when the
focused terminal `is_alt_screen() || is_command_running()` (the #435 F8 gate
is the template, app.rs key ladder). Check this at design time whenever a
chord decision names the Terminal context; the keymap-shadow sweep alone is
NOT enough — it checks other keymap rows, not the PTY.

## PR-claude-parallel-mutation-workers-get-their-own-target-001
*severity: high · prevents: F-claude-443-a-full-mutation-workers-shared-one-target-001*

A mutation run with more than one worker gives each worker its own target directory. Copy mode
isolates the sources but not an inherited `CARGO_TARGET_DIR`, and cargo leaves the checkout path
out of a workspace crate's artifact name, so parallel copies overwrite each other's binaries and
leave mutated artifacts that the real tree then reuses. Run parallel copies under
`env -u CARGO_TARGET_DIR`, or run one worker in place. To audit a finished run, grep a mutant's
log for compiler warnings that point at another mutant's line; after any run that shared the
target, `cargo clean -p` the mutated crates before trusting a test result.

## PR-claude-a-port-carries-every-mask-as-a-decision-001
*severity: medium · prevents: F-claude-443-b-the-port-stripped-the-mutation-masks-001*

When code moves between repos with different gates, every skip, mask and exclusion it carried
is carried over as an explicit decision: a documented exclusion in the destination's gate, or
tests that make the mask unnecessary. Stripping a mask so the code compiles is a silent floor
drop. Run the destination's whole gate, mutation included, before calling a port done.

## PR-claude-a-workspace-mutation-run-names-its-packages-001
*severity: critical · prevents: F-claude-443-c-diff-mutation-mutated-no-marley-crate-001*

A mutation gate run from a workspace root names what it mutates: `-p` for each package, or
`--workspace`. cargo-mutants otherwise mutates only `default-members`, and an in-diff run whose
diff misses them exits 0 as if the diff held nothing mutable. A gate that treats "no mutants"
as a pass must also fail closed when the diff has crate lines but no package was named. Before
trusting a new or ported mutation gate, run `cargo mutants --list --in-diff <diff>` with the
gate's exact flags and check the count is not zero.

## PR-claude-a-test-child-that-ignores-signals-ends-on-its-own-001
*severity: medium · prevents: F-claude-443-e-a-hup-ignoring-test-child-outlived-its-test-001*

A test that spawns a child which ignores termination signals gives that child a bounded life
(`exec sleep N`, with N above the test's own deadline) and waits for a marker the child prints
once it is ready, never a fixed sleep. PTY children call `setsid`, so neither nextest's
process-group kill nor the leader's hang-up reaches them when the test process dies first.
After any mutation or test run with such a test, `pgrep -af` for its command line.

## PR-claude-a-path-hook-resolves-the-files-own-repo-001
*severity: high · prevents: F-claude-436-a-a-path-hook-judged-files-by-the-sessions-directory-001*

A hook that classifies a file path resolves the repository from the file, never from the
session's working directory: walk up to the nearest existing directory and ask git
(`git -C <dir> rev-parse --show-toplevel --show-prefix`), compare repositories by
`--path-format=absolute --git-common-dir` so linked worktrees count as the same repository, and
read per-checkout state (a ledger, ignore rules) from that checkout. Test it from `/`, from a
scratch directory and from a linked worktree, not only from the repository root.

## PR-claude-a-rule-the-gate-checks-is-also-checked-at-commit-001
*severity: medium · prevents: F-claude-436-b-a-gate-only-rule-was-not-checked-at-commit-001*

When a rule lives in a gate whose run the commit hook does not require for every kind of change
(here, the receipt only for `.rs`), put the rule's check in the commit hook too, or say plainly
in every doc that it holds only when the gate runs. Check what the commit ships (the index, and
the work tree for `commit -a`), not only the work tree. And list paths from git with `-z`
whenever they are compared with strings from elsewhere; the quoted default never matches.

## PR-claude-a-zed-fork-identity-is-more-than-app-name-001
*severity: medium · from: pipeline 437*

`paths::APP_NAME` separates a Zed fork's directories, but not the rest of its identity. The
release channel keys the Secret Service items (a fixed label on every channel but `dev`), the
auto-updater (which on a non-`dev` channel downloads stock Zed and installs it over the stock
app) and the app id; the `zed://` scheme belongs to whichever Zed registered it with the desktop.
When a fork changes its name, keep it on `dev` or give it its own keyring label, update source,
app id and scheme in the same change, and guard the channel with a test while it depends on it.

## PR-claude-state-another-entity-reads-is-kept-outside-render-001
*severity: high · prevents: F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001*

Anything another entity reads from a view (a `Sidebar` or `Panel` trait method, a flag the
status bar polls) is computed where the view learns of the change, in its event handlers, and
stored; `render` only draws it. A view that is hidden is not rendered, and hidden is often the
very state in which the reader asks. Test such a flag with the view closed.

## PR-claude-a-swapped-out-zed-entity-is-kept-not-dropped-001
*severity: medium · prevents: F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001*

When Marley replaces a Zed view at runtime (a sidebar, a panel, a dock item), keep the replaced
entity alive and hand it back when the swap reverses, rather than dropping it and building a new
one. The replaced view may own running tasks and state that Zed persists through it; while the
stand-in is registered, it answers the persistence hooks (`serialized_state`) with the kept
view's state. Test the round trip, closed and open, and across a restart.

## PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001
*severity: medium · prevents: F-claude-438-c-a-marley-helper-paraphrased-gpl-code-001*

A Marley crate (MIT OR Apache-2.0) may call any public function of a GPL Zed crate, and may read
Zed's code to learn what to call; it never carries a Zed function body over with its names
changed. Write from the called functions' signatures and docs, in the crate's own shape, and
when the result still reads like the upstream body, restructure it. The inspect phase's
provenance lens compares new Marley code against the Zed files it was modelled on.

## PR-claude-a-new-gate-check-is-proven-red-by-a-planted-fault-001
*severity: medium · prevents: F-claude-447-a-warning-check-read-quiet-cargo-output-001*

Every new gate check gets a negative smoke. Plant the fault the check exists to catch, run the
check's own code (the function from `script/gates.sh`, not a reconstruction of it), and see it
go red; remove the fault and see it go green. A check whose input a flag or a filter can empty,
such as `--quiet`, a filter on a diagnostic's source span, or a tool that reads only the files
git lists, passes on a clean tree whether it works or not. Only the planted red proves it.

## PR-claude-bind-a-multi-line-closure-before-its-question-mark-001
*severity: low · prevents: the gate:4 misses in #438 and #440 (L-claude-438-the-coverage-floor-counts-lines-per-function-001)*

In a Marley crate, never end a multi-line closure argument with `})?;`. llvm's per-function line
count reads that line as the `?`'s error path alone and reports it missed, even when the file
view and `--show-missing-lines` show nothing. Bind the closure first, then call:

```rust
let launch = |terminal: &mut Terminal, cx: &mut Context<Terminal>| {
    terminal.write_init_command_after_startup(input, cx)
};
let written = terminal.update(cx, launch)?;
```

The `?` then shares a line with code that ran. It bit #438 (`})?;` after an update on a weak
handle) and #440 (two `terminal.update` calls in the agent launch).

## PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001
*severity: high · prevents: F-claude-441-a-task-provider-read-the-workspace-inside-its-update-001*

Before implementing a Zed trait method or callback in a Marley crate (a `TerminalProvider`, an
observer, a subscription), find its caller. When the caller runs inside an entity's update, as
`Workspace::spawn_in_terminal` calls the provider from inside the workspace's, nothing the
callback calls may read or update that entity. That includes Zed code that only holds a handle
to it, such as `TerminalPanel::spawn_task`. Defer the work to the window's next turn
(`window.spawn`, then `update_in`). Drive the callback in a test through Zed's public entry
point, the way Zed calls it, never by calling the method outside an update.

## PR-claude-defer-in-does-not-leave-the-entitys-own-update-001
*severity: high · prevents: F-claude-442-a-close-deferred-on-the-rail-ran-inside-the-rails-update-001*

`cx.defer_in(window, |this, window, cx| …)` and `cx.defer(|this, cx| …)` on a `Context<T>` run
their callback inside an update of `T`. They escape the update in progress, not `T`'s lease.
When the deferred work makes other code read `T` (a `MultiWorkspace` reading its sidebar, a
pane reading its item, a panel reading its workspace), defer with `window.defer(cx, …)` or
`cx.defer(…)` on the `App`, capturing weak handles, so no entity is leased when it runs. Drive
the deferral in a test through the Zed entry point that calls it, where a double lease
panics.

## PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001
*severity: medium · prevents: L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001*

Before a driven test counts as proof, break the code it guards once and watch the test fail:
take the key context out, drop the guard, skip the call. Four tests in #453 and #457 passed
with their code removed:
- #453's keystroke test pressed only keys Zed binds with no context, so it passed without the
  `menu` context;
- #453's focus check expected the window's row, which was also the keyboard's;
- #457's no-fold test pressed left and then right, so the second key undid the first one's
  wrong fold;
- #457's first-match test typed a query whose first match was also the window's row.

Each assertion expected a value the broken code also produced. Pick inputs on which the right
and the wrong code disagree, record each negative check in the notes, and restore the file by
checksum afterwards.

## PR-claude-a-gate-step-starts-from-no-output-an-earlier-run-left-001
*severity: medium · prevents: F-claude-465-gate4-counted-lines-from-a-stale-executable-001*

A gate step that reads a build directory shared across runs (`/mnt/fast/target` and its
`llvm-cov-target` serve every run and every project on this box) must not take in outputs an
earlier run left: remove them before it runs, or pass the tool only what this run built. A
tool's own clean may cover only the packages it runs; read what it reads before trusting it,
as cargo-llvm-cov's `object_files` showed in #469. The negative smoke plants an output from
older source and watches the step stay true to the tree.

## PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001
*severity: high · prevents: F-claude-474-rerun-would-have-run-a-command-that-output-printed-001*

A block's fields come from hook frames, and a frame is output: any program or printed file can
emit one. Before a block action turns a field into input, a path to open or a place to go (Rerun,
T2's path links against the block's cwd, an agent reading a block), check that the frame it came
from carried the terminal's nonce. Today only `preexec` carries it (`command_verified`); an
action on a `precmd` field (`pwd`, the exit code) adds the nonce to `precmd` first. Showing a
field (a pill, a bar) needs no check.

## PR-claude-scripts-that-start-detached-processes-trap-the-signals-001
*severity: medium · prevents: F-claude-488-a-signal-skipped-the-e2e-cleanup-001*

A bash script whose cleanup lives in `trap … EXIT` and that starts anything that outlives it
(`setsid -f`, `systemd-run`, a compositor) also traps the signals that end it:
`trap 'exit 130' INT TERM HUP`. Bash runs the EXIT trap on `exit` and at the end of the script,
not when a signal it does not trap kills it, so `timeout`, Ctrl-C or a closed terminal would
leave the detached processes running. And a script that hands its own background jobs to
sourced code tells that code to `wait "$pid"`, never a bare `wait`.

## PR-claude-a-per-frame-path-is-optimized-in-the-dev-build-001
*severity: medium · prevents: F-claude-489-the-browser-decoded-frames-unoptimized-001*

Code that runs for every frame or every input event, including the dependencies it calls
(decoders, converters, parsers), is timed in the debug build before its ticket closes. When it
is slow there, it gets `opt-level = 3` in `[profile.dev.package]` beside Zed's own hot crates
(`tree-sitter`, `taffy`, `resvg`, `serde_json`). The debug build is the one the user runs, so a
number measured only in release does not describe what they get.

## PR-claude-watch-before-you-announce-ready-001
*severity: medium · prevents: F-claude-492-the-observers-came-on-after-the-page-showed-001*

A component that others wait on (a hub whose state flips to ready, a server that writes its
endpoint, a session other tasks act through) turns on everything it watches before it
announces readiness, in the same sequence, not in a task spawned after the flip. Whoever waits
on the state acts as soon as it flips, and their first action races any watcher still being
set up. The first events are the ones an agent most wants: the page's own load.

## PR-claude-lay-out-what-no-view-draws-yet-001
*severity: medium · prevents: F-claude-493-a-page-behind-a-tab-kept-chromiums-default-size-001*

A surface an agent can read while no view draws it (a page behind another tab, a terminal in a
hidden pane) gets the geometry of the view that will show it as soon as it exists, not at that
view's first paint. The agent's reading is meant to be what the user would see, and a surface
laid out only on paint reads at a default size until someone looks.

## PR-claude-a-waiter-outlasts-the-failure-it-waits-through-001
*severity: medium · prevents: F-claude-494-waiting-tabs-gave-up-on-a-failure-001*

A task that waits on a component's readiness to finish something the user is shown as pending
(a tab's page, a panel's data) waits through a failure and the restart after it, unless the
failure means the job is gone; the task belongs to the thing that shows it pending, so it ends
when that does. A wait that returns on the first failure strands its dependents once the
component recovers, and nothing retries them.

## PR-claude-a-views-key-needs-its-fields-context-too-001
*severity: low · prevents: F-claude-496-ctrl-shift-c-in-a-tab-field-opened-the-collab-panel-001, F-claude-496-escape-never-reached-a-browser-page-001*

Before binding a key in a Marley view's context, or relying on a key reaching the view's
`on_key_down`, grep Zed's keymaps for that key. A binding of it in `Workspace` or `Pane` runs
before the view's key listener: give the view's context a `null` binding for a key the view must
receive. A binding under a negated predicate (`!Terminal`, `!Editor`) matches at the deepest
context, so from any field inside the view it outranks the view's own binding: bind the key in
`<View> > Editor` too. Check the key from the view itself and from each of its fields.

## PR-claude-an-event-handler-never-unwraps-what-another-event-sets-001
*severity: high · prevents: F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001*

In a handler for a stream of protocol events (Wayland, CDP, ACP, a hook feed), never `unwrap` or
`expect` state that a different event initializes. A peer can send events in an order the code
did not expect, or never send the initializing one at all: treat the state as absent and return,
drop the event, or queue it until the state arrives, and log a failure to build it rather than
panic. When touching such a handler, including an upstream Zed one, grep it for `.unwrap()` and
`.expect(` on shared state and ask what happens if the setup event is missing or fails.

## PR-claude-redact-the-whole-text-before-cutting-it-001
*severity: high · prevents: F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001*

Run a redaction pass over the whole text before any truncation, paging or tailing, and cut the
redacted text. A rule that anchors on a header or a prefix (`-----BEGIN`, `Bearer `, `NAME=`, a
URL's userinfo) cannot match once a cut has taken the anchor, and the secret's body then passes.
When a tool answers part of a larger text, count the redactions over the whole and say so in its
schema.

## PR-claude-check-a-unix-socket-path-against-sun-path-001
*severity: high · prevents: F-claude-513-a-socket-path-too-long-read-as-a-running-marley-001*

A Unix socket's path must be shorter than `sun_path` (108 bytes on Linux, 104 on FreeBSD and
macOS). When the path comes from a directory the user or a harness chooses (`--user-data-dir`, a
profile copy, a scratch folder), check its length before binding, and treat "too long" as its own
case with its own message, never as "in use" or any other failure the caller acts on. A scenario
that makes sockets under its own folders keeps those folders short (the runtime directory).

## PR-claude-compare-against-the-state-the-change-acts-on-001
*severity: high · prevents: F-claude-544-the-resize-arms-columns-changed-is-always-false-001, F-claude-544-a-rewrap-cut-history-rows-without-counting-them-001*

When a handler decides whether a change happened (the width changed, rows left the history), read
the "before" from the thing being changed, at the moment before the change, not from a cache or a
copy that another path updates on its own schedule: `set_size` had already written the new bounds
into the cache the Resize arm compared with. And when a vendored or upstream routine drops data
(a `truncate`, a `drain`, a rotate), check that every counter that promises to account for dropped
data is told.

## PR-claude-drop-the-unbounded-fields-first-001
*severity: medium · prevents: F-claude-519-a-frame-bound-dropped-the-shown-fields-before-the-unbounded-ones-001*

When a payload is kept under a size by dropping fields, drop first the fields that have no bound
of their own (paths, free text passed through as it came), and only then the ones already cut to
a length; and a check of the bound asserts which fields survived, not only the size.

## PR-claude-name-the-fakes-the-app-runs-001
*severity: high · prevents: F-claude-547-a-scenarios-click-ran-the-real-claude-001*

A scenario's fake for a program that Marley itself starts (not one typed in a terminal) is named
through an environment variable Marley reads (`MARLEY_CLAUDE`, `MARLEY_CHROMIUM`), never through
the PATH alone: the app's PATH can come from the login shell and find the real program first.
Keep the real program's own state pointed at scratch too (`CLAUDE_CONFIG_DIR`), so a fake that is
missed still touches nothing of the user's.

## PR-claude-empty-a-variable-the-child-must-not-inherit-001
*severity: medium · prevents: F-claude-520-a-key-removed-from-the-builders-map-still-reached-the-program-001*

To keep a variable from a terminal's program, set it to an empty value in the environment map
Zed's terminal builder is given, and make its readers treat empty as unset; removing the key
from the map does nothing, since the program inherits Marley's own environment under the map. A
scenario that proves the variable is gone exports a foreign value before the launch, so an
inherited leak shows.

## PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001
*severity: medium · prevents: F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001*

State a Marley item restores from Zed's per-item tables is read before any cleanup can run, at
`init` or from memory kept since, never from the table at the item's `deserialize`: Zed runs
more than one `cleanup` for one kind (the workspace's, and a panel's with its own items), each
with its own list of live items, and one can land between two items' restores.

## PR-claude-a-labels-values-decide-which-redaction-rule-owns-it-001
*severity: high · prevents: F-claude-562-a-one-word-value-rule-would-have-hidden-the-scheme-and-left-the-credential-001*

Before a label joins a redaction rule, write down the values it carries and check them against
that rule's value pattern. The `secret` rule hides one word, so a label whose value is a scheme
and a credential (`Authorization`, `Proxy-Authorization`) or a list (`Cookie`) gets a rule of
its own, one that keeps the label and the scheme and hides everything after them. A new rule's
place in `BUILT_IN` is checked against every rule before it, with a line that both could match.

## PR-claude-defer-a-pane-change-out-of-an-items-own-event-001
*severity: high · prevents: F-claude-503-a-tab-opened-inside-a-terminal-views-event-would-update-the-view-again-001*

Code that runs while an item is being updated must not add, activate or close items in that
item's pane: its own subscription callbacks, an `on_action` it registered, and the hooks Zed's
views call in their updates (`MarleyTerminalUrl`, `MarleyTerminalFooter`). Changing the pane's
front item calls `deactivated` on the old one, which updates it. Defer the change with
`window.defer` or a spawned task, as Zed's `open_path_like_target` spawns its open.

## PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001
*severity: high · prevents: F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001*

A program Marley writes for others to run (an opener, a hook, a helper in a terminal's
environment) finds the Marley that wrote it from where it was written: a file beside it in
`<data_dir>`, or a variable Marley set for it. It never uses the default data directory, where
the user's main Marley lives. A second Marley (`--user-data-dir`, an e2e profile copy) would
otherwise reach the main one, and a scenario would drive the user's own window.

## PR-claude-a-view-that-redraws-on-a-changed-snapshot-keeps-what-it-draws-in-it-001
*severity: medium · prevents: F-claude-504-a-favicons-arrival-changed-nothing-the-rail-compares-001*

A view that redraws only when a compared snapshot changes (the rail's `refresh`) must put in that
snapshot a value for everything its render reads from elsewhere: an id, a count or a version for
an image, a handle or a side map. Otherwise a change that touches only the side value draws
nothing until an unrelated change. When a row gains something drawn from a side map, add its key
to the row's pure snapshot in the same change.

## PR-claude-capture-a-screencast-page-whole-and-cut-it-locally-001
*severity: medium · prevents: F-claude-505-a-clipped-capture-left-the-tab-showing-only-the-crop-001*

While a page's screencast runs, capture it whole (`Page.captureScreenshot` with no `clip`) and cut
the part you want in Rust: `pick::crop` does this. A clipped capture can race into the stream as a
frame of the clip, and the tab draws that frame as the page. The next frame comes only when the
page changes, so a still page stays wrong.

## PR-claude-close-chromium-over-cdp-before-stopping-its-unit-001
*severity: medium · prevents: F-claude-507-a-stopped-unit-lost-the-cookie-set-before-the-stop-001*

Before Marley stops a Chromium unit, it closes Chromium over CDP (`Browser.close`), waits for the
unit to stop, and only then runs `systemctl --user stop` for what is left:
`browser::stop_chromium` does this. A scenario that stops a unit to test what a profile keeps
closes it the same way (`browser_close` in the fixture), or signs in more than 30 seconds before
the stop.

## PR-claude-find-a-typed-commands-block-by-its-command-001
*severity: medium · prevents: F-claude-523-a-runs-watcher-read-the-startups-block-001*

Code that types a command into a terminal and waits for it to finish finds the command's block
by its command (`AnchoredBlock::command`, compared with the line typed), never by its position
among the terminal's blocks: a new terminal's startup opens a block of its own first.

## PR-claude-a-string-a-page-script-hands-marley-is-made-well-formed-001
*severity: low · prevents: F-claude-582-a-title-watcher-cut-its-title-without-making-it-well-formed-001*

Any script Marley runs in a page that hands Marley a string (a binding's payload, a value
`Runtime.evaluate` or `Runtime.callFunctionOn` returns) passes each string through
`toWellFormed` (guarded, `value.toWellFormed ? value.toWellFormed() : value`) and cuts it off a
surrogate pair, as `pick.rs`'s and `recorder.rs`'s `cap` do. The Plan's recall for a new page
script searches the ledger for "surrogate" along with the ticket's own terms.

## PR-claude-bound-every-read-before-auth-in-size-and-time-001
*severity: high · prevents: F-claude-524-the-mcp-servers-read-before-auth-had-no-bound-001, F-claude-524-a-431-closed-with-the-request-unread-001*

Every read a server makes before it knows who is asking has a bound in bytes per unit (a line, a
frame, a body), in units (header lines), and in time over the whole request, not per read call:
a peer that trickles a byte a second defeats a per-read timeout. A refusal written before the
request was read to its end drains the rest (bounded) after shutting the write side, so the peer
reads it. PR-claude-cap-client-size-before-alloc-pre-auth-001 covers the size of one allocation;
this covers the rest of the path.

## PR-claude-an-event-a-command-caused-is-kept-whatever-the-view-001
*severity: medium · prevents: F-claude-583-a-script-runs-start-was-lost-while-no-tab-drew-the-page-001*

A record of something Marley itself did to a page (a script run, an agent's action) is kept
whatever state the page's view is in: a rule that keeps entries only while a view is drawn is for
what the page does on its own, and a command that moves views around can run inside the moment
no view draws. Timing that only a release build hits is found by the install's golden set, so a
check on such a record stays in a golden scenario.

## PR-claude-prove-what-a-program-says-on-the-file-it-reads-001
*severity: medium · prevents: F-claude-584-a-cut-off-clients-bridge-said-marley-was-not-running-001*

A scenario that checks what a program tells its user after Marley changes the program's state
points the program at the file it really reads, as Marley left it (gone, rewritten, emptied),
never at a copy the scenario saved before the change. A copy proves how the program handles the
copy; the user's program sees what Marley did to the real file.

## PR-claude-every-marley-folder-a-commit-adds-to-is-in-gate-10s-tree-scan-001
*severity: high · prevents: F-claude-584-gitleaks-saw-a-scenario-only-after-it-was-pushed-001*

Every folder or script of Marley's own that a commit can add to is in gate:10's working-tree
scan. The history scan sees a file only after it is committed, and on this branch a push follows
the commit. A new Marley-owned folder or script joins the list in the change that creates it.

## PR-claude-hold-a-target-at-its-start-until-it-has-what-it-needs-001
*severity: medium · prevents: F-claude-539-a-cross-site-iframes-script-kept-the-headless-name-001*

When a target must have a setting before its first script runs (an identity, an emulation, a
binding), attach it held at its start (`waitForDebuggerOnStart: true`), send the setting, then
`Runtime.runIfWaitingForDebugger`. An override sent after an unheld attach races the target's
own load. Resume every held target, including those not handled, so none stays paused. A probe
that answers events at once does not show the race; Marley's path through the relay and the hub
does.

## PR-claude-a-page-sent-on-from-blank-gets-its-size-and-stream-again-001
*severity: medium · prevents: F-claude-539-a-reopened-tab-drew-its-page-87-pixels-short-001*

A page created at `about:blank` and then sent to its URL takes what its tab sends during the
first commit unreliably: a command can fail with "Not attached to an active page", and the
stream can keep the window's size. Send the size again and restart the stream once the URL
commits, and do not count on the tab's size-change check, which sends nothing when the size
has not changed.

## PR-claude-run-a-views-poll-while-it-shows-001
*severity: low · prevents: F-claude-521-the-port-scan-ran-behind-a-closed-rail-001*

When a view's rows come from a poll (a scan, a timer, a watch of the machine), start the poll
when the view shows and stop it when the view hides, not when the view is built and dropped. A
panel or sidebar that closes stays alive, so an entity's life says nothing about whether anyone
sees it. Follow what shows it (for the rail, the `MultiWorkspace`'s open sidebar), keep a flag
so a start and a stop pair up, and give the poll back on release when it is still held.

## PR-claude-count-one-user-action-once-when-each-call-reports-it-001
*severity: low · prevents: F-claude-566-parallel-tools-counted-one-interrupt-twice-001*

Claude Code reports one user action once per call in flight: an Escape during parallel tools is a
`PostToolUseFailure` with `is_interrupt` for each. A fold or a use that counts the action (a stop,
an interrupt, a refusal) counts its first report in the turn and lets the rest only end their
calls, and anything it asks or logs for the action keys on the change the first report made (the
seat leaving `working`), not on each frame.

## PR-claude-an-answer-for-a-seat-names-the-session-prompt-and-stop-it-belongs-to-001
*severity: low · prevents: F-claude-566-an-outcome-crossed-sessions-in-one-terminal-001, F-claude-566-an-earlier-stops-kind-would-show-at-a-later-stop-001*

A seat is its terminal's, and in it sessions, prompts and stops follow one another. What one event
of a seat leaves for a later one, such as an answer that lands after a model call or an outcome
the next prompt logs, carries the session, the prompt and the stop it belongs to, and the later
event checks all three before it acts. What describes one stop is cleared at every turn's start
and end, not at the user's prompt alone, since a harness's prompt starts a turn too.

## PR-claude-a-state-fact-holds-only-what-code-computed-001
*severity: medium · prevents: F-claude-569-the-command-in-flight-went-out-as-a-fact-001*

A System One state's facts are kept at every detail and never cut, so a metadata-only project
sends them: a fact holds only what code computed, such as a name, a count, a state or a duration
in words. Text an agent or the user wrote, such as a command, a tool's input, a path or a prompt,
goes in with `StateBuilder::text`, which masks it whole, cuts it to 300 characters and leaves it
out at `Detail::Facts`. Name the tool as a fact and put its line in as text.

## PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001
*severity: low · prevents: F-claude-571-the-cards-buttons-would-hand-it-the-focus-as-it-left-001*

A gpui element whose `on_click` takes the focus hands it the clicks of its children too, since a
click bubbles up after the child's handler. A button inside such an element calls
`cx.stop_propagation()` in its own handler, above all when the handler removes the element.

## PR-claude-a-state-another-view-lists-is-announced-by-an-event-001
*severity: medium · prevents: F-claude-508-a-held-click-reached-the-inbox-only-with-another-refresh-001*

Before a view lists a state another entity keeps, check how the view listens to that entity. A
subscriber (`cx.subscribe`) hears only the events the entity emits; `cx.notify()` reaches
observers alone. The rail subscribes to the Browser hub's events because the hub notifies on
every frame. A change the listing must show emits an event the view already refreshes on, when
the state begins and when it ends.

## PR-claude-a-check-on-terminal-text-reads-a-reply-from-where-it-starts-001
*severity: low · prevents: F-claude-481-the-rich-inputs-check-raced-the-echo-of-its-paste-001*

A scenario that reads a terminal's text for a program's reply to typed or pasted input cannot
count on the reply starting a line: the tty echoes the input as it arrives, and a program that
answers a line before the rest of the input comes writes into the middle of the echo. Take the
reply from its own marker to the line's end and match that, never the whole screen line.

## PR-claude-a-number-taken-across-awaits-is-taken-by-one-task-at-a-time-001
*severity: medium · prevents: F-claude-509-two-quick-closes-could-pin-one-turn-number-twice-001*

When a task reads shared state (a listing, a counter, a flag), awaits, and then writes what it
read decides (the next number, a ref, "done once"), and a second task of the same kind can start
before the first writes, the two can read the same value. Chain such tasks (each awaits the one
before it, kept as a shared task), or read and write in one step on the main thread with nothing
awaited between. A detached task that must finish keeps a detached waiter when the owner of its
handle can drop.

## PR-claude-a-check-on-a-shared-log-names-its-writer-001
*severity: low · prevents: F-claude-532-the-enter-check-counted-a-log-every-stand-in-writes-001*

When several stand-ins write one log, a check on it names which one wrote what it counts: its
case, its arguments or its pid, written with each line. A count alone passes when the wrong one
did the thing, and a click that lands on the wrong row does exactly that.

## PR-claude-what-a-terminal-shows-now-is-read-from-its-visible-rows-001
*severity: medium · prevents: F-claude-587-the-trust-watch-read-an-answered-question-from-the-scrollback-001*

`Terminal::last_n_non_empty_lines` skips blank rows, so on a sparse screen its lines come from
the scrollback, where what a program cleared or answered stays. A check on what a terminal shows
now, such as a dialog, a prompt or a menu, reads the visible rows
(`Terminal::with_renderable_cells`, a row per `point.line`). It may take text from the joined
lines only once a part of the thing is on a visible row.

## PR-claude-column-significant-git-output-is-read-untrimmed-001
*severity: medium · prevents: F-claude-511-the-merge-checks-trimmed-porcelains-leading-space-001*

git output whose columns carry meaning, such as `status --porcelain` (two status letters, either
a space), `diff --name-status` or `ls-files -s`, is split into lines before anything is trimmed,
and each line is sliced as it came. A helper that trims the whole output serves one-value answers
only: `rev-parse`, `config --get`, `symbolic-ref`, a count.

## PR-claude-a-repositorys-text-in-a-zed-prompt-goes-in-a-code-block-001
*severity: high · prevents: F-claude-592-the-launch-approval-was-rendered-as-markdown-001*

`window.prompt`'s message and detail are Markdown on Linux (`crates/ui_prompt/src/ui_prompt.rs`):
single line breaks join, `--` and quotes are rewritten, and markup can hide text. Text a user
must read as written, above all a repository's command or a path on a question that approves it,
goes in a fenced code block whose fence is longer than any run of backticks inside
(`launch::verbatim`). Marley's own sentences can stay plain.

## PR-claude-keys-after-a-paste-go-in-a-later-write-001
*severity: high · prevents: F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001*

A program may read a bracketed paste together with whatever input is waiting, as Python's REPL
does, and take a following Enter or key as text. Anything Marley sends after `Terminal::paste`,
an Enter above all, goes through `terminal_drive::paste_then`, which sends it after
`AFTER_PASTE`. A visual check of a paste-then-Enter path runs a program that reads bracketed
pastes (`python3 -q`), not a stand-in that reads lines, and checks the program's output on the
screen, not the tool's answer.

## PR-claude-a-render-hook-reads-its-context-not-its-view-001
*severity: high · prevents: F-claude-595-the-footer-read-its-own-view-while-it-rendered-001*

A hook a Zed view calls from its own `render` (`MarleyTerminalFooter`, `MarleyTerminalSuggestion`,
an element callback) runs while that view is leased. It reads what its context hands it
(`context.terminal`, `context.project`) and never `view.read(cx)` or a helper that does; a
helper shared with non-render paths gets a variant that takes the inner entity. The weak handle
in the context is for click handlers, which run later.

## PR-claude-a-scenario-that-opens-zeds-terminal-menu-turns-zeds-agent-off-001
*severity: high · prevents: F-claude-555-a-menu-walked-by-its-keys-opened-zeds-own-agent-001*

The run's profile is a copy of the user's, so Zed's Agent Panel there is the user's own agent. A
scenario that opens Zed's terminal or editor menu sets `"agent": {"enabled": false}` in
`$E2E_PROFILE/config/settings.json` during `setup` (`zed_agent_off` in
`555-send-a-block-to-the-agent.sh`), which takes Inline Assist and Add to Agent Thread out of the
menus. It counts menu items only over items that cannot be disabled, and it brings a terminal
forward by its tab (`alt-N`), not by a rail row, since the rail's rows move when Needs you shows.

## PR-claude-name-a-context-bound-key-through-its-focus-handle-001
*severity: medium · prevents: F-claude-563-text-for-action-misses-a-terminal-binding-001*

To show the key bound to an action in a nested context (`Terminal`, `Editor`, a modal's), look it
up with `window.bindings_for_action_in(action, &that_element's_focus_handle)` and take the last
binding, or build a `KeyBinding::for_action_in`; `ui::text_for_action` and
`Window::highest_precedence_binding_for_action` see only the frame's root contexts, so they find
`Workspace` bindings and silently miss the rest.

## PR-claude-543-follow-a-views-terminal-through-set-terminal-001
*severity: medium · prevents: F-claude-543-a-rerun-task-lost-its-notifications-001*

A subscription to a `TerminalView`'s terminal made in `observe_new` sees only the view's first
terminal: a task's Rerun swaps in a new one with `TerminalView::set_terminal`. Subscribe through a
helper, and observe the view (`cx.observe_in(&cx.entity(), window, …)`) to subscribe again when
`view.terminal().entity_id()` changes, as `notifications::watch` does.

## PR-claude-588-every-scenario-names-what-marley-opens-001
*severity: high · prevents: F-claude-588-a-scenario-that-opened-nothing-restored-the-users-session-001*

Every e2e scenario's `setup` calls `open_path` on a folder of the run's own (a scratch repository
in `$E2E_WORK`), and gives its terminals a HOME of their own with `terminal_env`. `script/e2e.sh`
exits with `no folder to open` otherwise: a run that opens nothing restores the user's last
session from the profile copy, Agent Panel threads and all.

## PR-claude-596-look-up-across-windows-before-updating-one-001
*severity: high · prevents: F-claude-596-a-lookup-inside-a-window-update-lost-that-windows-terminals-001*

Inside `window.update(…)` (or `update_in`, `update_window`), never call a helper that reads windows
through their handles (`cx.windows()`, `WindowHandle::read`, `downcast::<…>()?.read(cx)`, such as
`mcp::terminals`): the window being updated is out of the app for the duration and reads as gone,
so the lookup silently misses exactly the window in hand. Do the cross-window lookup first, in
`cx.update` or before the update, and inside the update reach the window's own entities through
the `window` argument (`window.root::<MultiWorkspace>()`).

## PR-claude-599-say-which-parts-of-a-url-make-it-the-same-page-001
*severity: medium · prevents: F-claude-599-a-page-that-moved-to-its-own-anchor-got-a-second-tab-001*

Before reusing a Browser tab "already on" a URL, decide which parts of the address make it the
same page for this caller, and write it down at the call: a terminal link keeps the fragment (a
single-page app routes on it), a document Marley ships drops it (its own links add one), and a
query string may or may not count. Use `browser::show_tab_where` with that predicate rather than
`open_url_tab`'s exact match when the page can change its own address, and let the scenario click
a link in the page before it asks for the tab again.

## PR-claude-600-a-lookup-others-call-inside-updates-reads-no-entity-001
*severity: high · prevents: F-claude-600-a-registry-lookup-read-the-workspace-its-caller-was-updating-001, F-claude-600-a-follow-up-inside-the-windows-root-update-would-update-it-again-001*

A global registry that other modules query (`BrowserProject::of`, the rail, MCP tools) will be
queried from inside an update of the very entity it describes. Store at registration whatever the
lookup compares (entity ids, keys, names) and let the lookup read no entity. When you hand a
callback a window to act in, run it through `AnyWindowHandle::update`, never through
`WindowHandle<Root>::update`, which leases the root view the callback may need.

## PR-claude-607-ask-the-dock-whether-a-panel-shows-001
*severity: medium · prevents: F-claude-607-polling-followed-set-active-not-what-shows-001*

Work a panel should do only while it is seen (polling, sampling, a timer) checks
`dock.visible_panel()` (which is `None` for a closed dock), downcast through `to_any()`, each
time it runs, and a panel that draws may start it. Do not count `Panel::set_active` calls:
Zed's docks send them for closed docks too, and moves between docks do not pair them.

## PR-claude-614-a-port-of-a-container-engines-own-unit-is-never-stopped-through-it-001
*severity: high · prevents: stopping every container from one port row*

A listener's cgroup can name the container engine's own unit (`docker-proxy` runs in
`system.slice/docker.service`). A port's Stop that stops the listener's service would then stop
Docker and every container. Clear the service for `ENGINE_UNITS` wherever a listener's service is
read for a Stop (`ports::attribute` and `ports::stop`), and route a container's port through the
engine's own `stop`.

## PR-claude-615-a-menus-trigger-owns-its-tooltip-001
*severity: low · prevents: F-claude-615-a-rows-tooltip-lay-over-the-menu-it-opened-001*

An element that opens a `right_click_menu` sets its tooltip inside the menu's `trigger`, behind
the `is_menu_active` flag the trigger receives, never on the element outside it: gpui clears a
tooltip that is waiting to show only on the pointer's next move, so one set outside lies over
the menu it just opened.

## PR-claude-read-a-terminals-typed-line-where-its-frame-is-drawn-001
*severity: medium · prevents: F-claude-573-the-grids-typed-line-was-never-read-001*

A terminal's output and its echo of typed keys arrive as `Event::Wakeup`, and its `last_content`
is synced only when the view draws. An observer (`cx.observe`) hears only `cx.notify()`, which a
block's start and end send, and a `Wakeup` subscriber reads the frame before the one the event
brought. To follow what is typed at a prompt, read it in a hook the terminal element calls while
it lays out (`MarleyTerminalSuggestion`), where the content is the frame being drawn.

## PR-claude-a-test-binary-that-reaches-the-data-folder-sets-its-own-001
*severity: medium · prevents: F-claude-475-the-tests-wrote-the-users-data-folder-001*

A crate whose tests reach `paths::data_dir()` or `paths::config_dir()` (a real shell through
`TerminalBuilder::new`, a store opened by `prompt_store::init`, any Marley module that writes under
the data directory) gives its test binary a scratch folder before the first test:
`#[ctor::ctor(unsafe)] fn …() { terminal::marley_use_test_data_dir(); }` in a `#[cfg(test)]`
module, with a `// SAFETY:` line above it (gate:13) and `ctor` among the dev-dependencies. Not a
call in a test's setup: the directory is a `OnceLock` that panics when set after its first read,
and under cargo test's threads another test can read it first. To see what a run writes, set
`XDG_DATA_HOME` to an empty folder and list it afterwards.

## PR-claude-ask-holds-focus-whether-a-terminal-view-has-the-keys-001
*severity: medium · prevents: F-claude-634-a-terminal-read-as-unfocused-while-its-prompt-editor-had-the-keys-001*

Never ask `terminal_view.focus_handle(cx).contains_focused(window, cx)` whether a terminal has the
keys: it is false while the prompt editor in its footer has them, the default at every prompt. Ask
`rich_input::holds_focus(view.read(cx), window, cx)`. The same trap waits for anything else drawn
in the footer or the overlay that takes focus.

## PR-claude-workbench-io-goes-through-gpui-or-a-seam-001
*severity: medium · prevents: F-claude-634-real-io-in-the-workbench-failed-103-tests-001*

In the workbench, blocking work runs as `cx.background_spawn(futures::future::lazy(..))`, never
`smol::unblock`; and a process, a socket or a read of the machine (`/proc`, the PATH, a user's
config) that a shared test setup reaches gets a seam the setup sets (`Ports::proc_root`,
`Launcher`). Otherwise gpui's test scheduler fails every test that opens the surface, after its
assertions passed.

## PR-claude-an-editor-inside-a-terminal-rebinds-the-terminal-keys-it-must-keep-001
*severity: high · prevents: F-claude-635-ctrl-shift-w-in-the-prompt-editor-closed-the-window-001*

Any focusable element drawn inside a terminal view (the prompt editor, a rich input, a filter)
loses every Terminal-context binding that an unscoped binding shares, because gpui ranks unscoped
bindings at the deepest context. List them (a script over `default-linux.json`: keys bound both
with no context and in `Terminal`) and bind, in the element's own context, the ones whose
terminal meaning the user expects there; Ctrl-Shift-W is the one that was lost.

## PR-claude-631-a-gesture-belongs-to-the-element-it-was-pressed-in-001
*severity: medium · prevents: F-claude-631-a-drag-at-a-prompt-selected-nothing-001*

When something takes the focus in reaction to a press (Marley's prompt editor docking at a
prompt, a dialog, a palette), every handler that finishes that gesture (drag, release, click
count) must ask whether the press happened in its element, not whether its element holds the
focus now. Before moving the focus on a press, list the pressed element's drag and release
handlers and their guards; in Zed's terminal element those are the `MouseMoveEvent` drag branch
and the left `on_mouse_up`, both once gated on `focus.is_focused`.

## PR-claude-639-a-change-to-the-grid-on-the-main-thread-carries-the-hooks-in-flight-001
*severity: medium · prevents: F-claude-639-a-hook-parsed-before-a-clear-was-applied-after-it-001*

A hook's absolute line is taken on the PTY thread when its frame is parsed, and used on the main
thread when the hook is applied. Anything the main thread does to the grid in between (Zed's
clear, a resize's rewrap, a scroll Marley drives) must also say how to carry a line taken before
it: count the change on the grid, carry the count in `HookPosition`, and map the line on apply.
Carrying only the anchors already applied leaves the hooks in flight behind. One passing run
proves nothing about such a race; run the scenario several times.

## PR-claude-641-what-a-view-draws-from-another-entity-redraws-on-its-own-001
*severity: low · prevents: F-claude-641-a-down-terminals-count-did-not-redraw-during-a-check-001*

When a Marley hook draws into a Zed view from state the view does not own (a global, another
entity's counter, a clock), something must notify the view when that state changes: an
`observe`, a `notify` where the state changes, or a ticker of its own for a clock or a count the
owner cannot notify about. A loop that also does other work (a check, a request) is not the
redraw: it stops ticking while it waits.

## PR-claude-641-insert-an-item-after-an-item-never-between-it-and-its-attributes-001
*severity: medium · prevents: F-claude-641-an-inserted-method-took-the-next-methods-cfg-001*

Adding a method or field to an upstream file by matching an existing item's first line puts the
new item between that item and its attributes (`#[cfg]`, `#[derive]`, doc comments). Anchor an
insertion on the end of the item before (its closing brace) or on a blank line, and read the
lines above the anchor first. A `#[cfg(test-support)]` moved this way passes clippy
`--all-targets` and fails only the build without the feature, so build the binary before the gate.

## PR-claude-643-check-a-typed-requests-answer-against-a-real-peer-001
*severity: medium · prevents: F-claude-643-zeds-typed-ping-never-parses-a-servers-answer-001*

Before relying on one of `context_server`'s typed requests, check that something in Zed already
sends it to real servers (`grep -rn "request::<Name>"`); a request no caller uses may carry a
response type no server's answer parses as. When none does, define a `Request` of Marley's own
whose response is `serde_json::Value` or the shape the MCP specification gives, and see it
answered by the stand-in or a real server in the scenario's first run.

## PR-claude-646-a-count-beside-a-label-is-a-muted-label-not-a-countbadge-001
*severity: low · prevents: F-claude-646-a-section-count-drew-as-an-alert-badge-off-the-panel-001*

`ui::CountBadge` is an overlay for an icon (absolute, top right, error-tinted), not a count to sit
beside text. A count in a list header, a section title or a row is a `Label` with
`LabelSize::Small` and `Color::Muted` (in `ListSubHeader::end_slot`, as the `ui` crate's example
does). The rail's Browser rows rejected it for the same reason
(AD-claude-504-browser-tabs-are-rows-of-their-project-in-the-rail-001); a component's preview in
Zed's component gallery shows what it draws before it is chosen.

## PR-claude-647-log-a-background-runs-work-and-wall-time-and-read-them-in-the-scenario-001
*severity: medium · prevents: F-claude-647-the-capped-graph-took-four-seconds-in-one-step-batches-001*

A Marley feature that runs a computation off the window's thread in slices logs one line when the
run ends with its size, its slice count, the time spent working and the time taken, and its
scenario greps that line at the largest size it supports and reads the figures in the Test phase.
A wall time far above the work time is the window's thread, not the computation.

## PR-claude-669-a-runs-rail-shows-only-what-the-run-started-001
*severity: medium · prevents: F-claude-669-scenario-coordinates-measured-with-the-machines-containers-in-the-rail-001*

A rail section that lists something of the machine's (its containers, its hosts, its processes) is
off in `script/e2e.sh`'s copy of the settings, and a scenario that tests it turns it on and gives
its own rows an order that puts them first, so no coordinate rests on what the machine runs and no
click can reach the machine's own. When a new section reads the machine, its setting joins the
copy's Python block in the same ticket.

## PR-claude-681-a-settings-value-in-effect-counts-the-project-files-001
*severity: medium · prevents: F-claude-681-merged-settings-read-as-the-value-in-effect-skipped-the-project-001*

`SettingsStore::merged_settings` (and `raw_user_settings`) leave out every project's
`.zed/settings.json`; Zed applies those per location (`Settings::get(Some(SettingsLocation))`). Code
that tells a person or an agent the value "in effect" either names the location it means, or reads
`get_all_files()` in order and takes the first file that sets the key, and says which. A scenario
for it sets the key in a project file and checks the value answered, not only the file named.

## PR-claude-686-a-prompt-that-comes-on-its-own-is-decided-in-the-e2e-copy-001
*severity: low · prevents: F-claude-686-the-agents-offer-came-over-a-scenarios-question-001*

A ticket that adds a notification, an offer or a dialog that comes on its own at start (not from
something the scenario does) adds, in the same change, the setting that decides it to
`script/e2e.sh`'s copy of the settings, so no other scenario meets it; its own scenario undoes
that in `setup`.

## PR-claude-687-a-scenario-types-into-a-thread-only-after-checking-it-runs-the-stand-in-001
*severity: medium · prevents: F-claude-687-a-scenario-a-step-off-opened-a-real-agent-on-the-users-login-001*

A scenario that opens an Agent Panel thread and types into it first `expect`s proof that the
thread runs its stand-in, such as the stand-in's log or environment file, before any
`type_text` or Return goes into it. A thread of Zed's own agent is opened where it cannot be
missed: New Agent Thread's submenu, whose first entry is Zed Agent. It is never opened with
`agent: new thread`, which opens the panel's last agent.

## PR-claude-693-pass-another-programs-codes-by-their-form-not-a-list-001
*severity: low · prevents: F-claude-693-a-fixed-list-of-another-programs-codes-went-stale-001*

When Marley relays another program's error codes, it matches their form, such as the
harness's `rh: CODE: reason` with `[a-z_]+`, and passes the code through. It does not keep its
own list of the codes. The other program owns its vocabulary and adds to it without telling
Marley.

## PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001
*severity: medium · prevents: F-claude-695-taking-upstreams-lockfile-dropped-marleys-security-pins-001*

When an upstream merge conflicts on `Cargo.lock`, take Marley's with `git checkout HEAD --
Cargo.lock` (not `--theirs`, and not `--ours`, which gives back the staged file), then build so
cargo adds upstream's new dependencies. Check that `cargo audit` holds before the gate.

## PR-claude-695-a-setting-that-moves-the-layout-stays-out-of-test-runs-001
*severity: medium · prevents: F-claude-695-the-users-font-size-moved-every-scenarios-clicks-001*

A setting that changes where things sit on screen, such as font sizes and families, never
reaches a run: `script/e2e.sh`'s copy of the user's settings drops it, as it drops the settings
that reach outside the run. A ticket that adds such a setting adds it to the copy's list.

## PR-claude-695-a-scenario-chooses-a-menu-entry-from-home-001
*severity: low · prevents: F-claude-695-scenarios-counted-on-a-menu-opening-on-its-first-entry-001*

A scenario that chooses a menu entry with the keys presses Home once the menu is open, then
counts Downs from the first entry. A menu opens with nothing chosen (Zed #64365), and Home holds
whatever upstream does next.

## PR-claude-697-a-hosted-view-focuses-where-its-own-host-does-001
*severity: medium · prevents: F-claude-697-a-thread-in-a-center-tab-took-no-typing-001*

A Marley `Item` that hosts a view another crate draws (a panel's view, a modal's body) gives as
its `focus_handle` the handle that view's own host focuses on activation, not the view's
`Focusable`. Read the host's activation path first (for the Agent Panel,
`activation_focus_handle`); a view's root handle often only tracks focus for its children. The
scenario types into the hosted view right after it opens, so a wrong handle shows as lost text.

## PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001
*severity: high · prevents: F-claude-701-a-listener-on-a-tab-updated-its-workspace-and-panicked-001*

A Marley `Item` (a tab) whose click handlers update its own workspace writes them as plain
closures over a `WeakEntity<Workspace>`, never `cx.listener`. A listener leases the item for the
whole handler, and many workspace operations read every item: `open_workspace_for_paths`
(`is_dirty`), closes, saves, splits. The read panics as a double lease. `RustyHome`'s rows show
the safe shape: a cloned weak handle, or an `*_later` opener that defers.

## PR-claude-702-making-a-workspace-in-the-background-keeps-the-focus-001
*severity: high · prevents: F-claude-702-a-group-made-at-start-took-the-windows-focus-001*

Code that makes a workspace the window won't show (`OpenMode::Add`, `open_workspace_by_id` into a
window) takes `window.focused(cx)` first and focuses it again once the workspace's task is done, if
it lost the focus. `Workspace::new` focuses the new workspace's pane, and its setup can move focus
after that. A scenario that has to click before its first key works is reporting this bug, not a
quirk to work around: find where the focus went before writing a lesson.
