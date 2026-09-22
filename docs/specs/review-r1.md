# Marley Round-1 Spec Review — Pre-Build Verification Report

**Scope:** the 15 EARS specs in `docs/specs/SPEC-*.spec.md` (M0 + M1 terminal substrate), reviewed against `_TEMPLATE.spec.md`, `standards/quality-bar.spec.md`, `marley_architecture/crate-triage.md`, and the `warp_architecture` clean-room source docs.
**Gate:** this is the gate BEFORE any code is written, held to the crown-jewel standard.

---

## VERDICT: NOT_READY

**Rationale.** The EARS craft is genuinely strong — clauses are numbered, use correct Ubiquitous/Event/State/Optional/Unwanted keywords, avoid weasel words, and map 1:1 to named tests, mutation targets, and ACCEPTED-UNTESTABLE carve-outs. Individual-spec internals are solid. But the specs fail at three places that block code today: (1) **boundaries** — the contracts between crates do not line up (editor re-forks the `text-offsets` vocabulary; `SessionId` is defined twice with divergent semantics; the syntax↔editor seam binds to types no spec owns; process-command and terminal-blocks make mutually exclusive claims about who spawns the PTY); (2) **coverage** — the M0 `warp_util` value-type crate (16 dependents, scheduled "first") and the `AssetCache` foundation that `marley_assets` is wholly expressed in terms of have **zero** spec coverage, and the binding visual gate (gate 15) points at a harness spec that does not exist; and (3) **clean-room provenance** — every REIMPLEMENT spec claims "behavior-only; no AGPL source read," yet the cited `spec_source` docs are structured per-crate transcriptions of Warp's internals, and at least six specs reproduce Warp's private type/module taxonomy inside their "Public surface (the contract)" sections. Eleven distinct blockers must be closed before the first ticket. None are form-level sloppiness; they are contradictions, holes, and an IP wall that would otherwise be laundered through the spec layer into code. With this many unbuildable/contradictory items the gate cannot be GO or GO_WITH_FIXES.

**Blocker count: 11.**

---

## Must fix before first ticket (blockers + highs worth fixing now)

Deduped across reviewers. Each item: the spec(s), the problem, the fix.

### Blockers

1. **Offset vocabulary is forked — `SPEC-editor` vs `SPEC-text-offsets`.** Editor redefines `pub struct CharOffset(pub usize)` / `ByteOffset(pub usize)` with **public** fields and declares "Marley components: none upstream at M1," contradicting `text-offsets` (the M0 hub whose whole purpose is one shared, private-field, non-interchangeable offset type that completer/search-core already reuse). Two incompatible offset vocabularies = guaranteed import/type conflict, and the unified-prompt seam (editor `Range<CharOffset>` edit vs completer `Range<ByteOffset>` span) cannot type-check.
   **Fix:** delete editor's local `CharOffset`/`ByteOffset`; depend on `marley_text_offsets` and import its private-field types; add a defined char↔byte conversion at the editor↔completer seam.

2. **`SessionId` defined twice with divergent semantics — `SPEC-marley_core` vs `SPEC-terminal-blocks`.** marley_core's `SessionId(u64)` is a process-allocated monotonic counter (`next()`/`as_u64()`); terminal-blocks redefines `SessionId(u64)` as a value decoded from a shell DCS `InitShell` hook with none of those methods. marley_core says terminal depends on it; terminal-blocks says "none upstream." `Block.session_id` will not unify with the `marley_core::SessionId` app-shell imports — wiring a pane to a terminal session cannot type-check.
   **Fix:** make terminal-blocks depend on `marley_core::SessionId` and drop its local newtype; if the shell-hook id is genuinely distinct, name it `ShellSessionId` and define the mapping. Reconcile the contradictory dependency-edge statements in both specs.

3. **`SPEC-terminal-blocks` R11 — stateful error from a stateless function.** R11 returns `HookError::MissingSession` when no `SessionId` is registered, but the only function returning `HookError` is the **stateless** free fn `decode_hook(encoding, payload)`, which has no access to the session registry. The clause binds to nothing satisfiable and `hook_without_session_returns_missing_session` cannot be written.
   **Fix:** split the error surface — `decode_hook` returns only codec errors (`UndecodablePayload`/`UnknownHook`); introduce a stateful `TerminalSession::apply_hook(&mut self, DcsHook) -> Result<(), HookError>` that owns the registry and is where `MissingSession` + "leave BlockList unchanged" live. Re-point R10/R11 and their tests there.

4. **Syntax↔editor seam binds to types no spec owns — `SPEC-syntax-highlight` vs `SPEC-editor`.** syntax-highlight consumes `BufferDelta`, `BufferVersion`, `Point`, `IndentUnit` "from the editor buffer model," but editor exposes `EditDelta`, a bare `u64` version, no `Point`, and no exported `Rope`; `IndentUnit` actually belongs to `marley_languages`. The crate is unbuildable against the real editor surface.
   **Fix:** pin the seam types in one spec — in `marley_editor` (or a shared buffer-core spec) define/export `BufferDelta` (rename `EditDelta`), a `BufferVersion` newtype (replace the bare `u64`), a `Point`, and the `Rope` alias; fix the syntax spec's Dependencies to source `IndentUnit` from `marley_languages` and `CharOffset` from `marley_text_offsets`.

5. **PTY spawn seam claimed by two specs — `SPEC-process-command` vs `SPEC-terminal-blocks`.** process-command's reason-to-exist ("the shell process that feeds the terminal PTY is built through `marley_command::blocking::Command`") and its integration test `pty_shell_spawn_seam` are contradicted by terminal-blocks, which spawns via `alacritty_terminal::tty` (its own openpty+fork+exec) and lists zero internal deps. The integration test cannot pass.
   **Fix:** choose one seam — either terminal-blocks builds its shell `Command` via `marley_command` and hands it to the PTY (add the dep, say so), or drop process-command's PTY claim + integration test and scope it to non-PTY child spawns. Align both specs' Dependencies/Purpose.

6. **`SPEC-input-classifier` R12 vs R13 — contradictory thresholds in the same regime.** R12 classifies `Ai` iff `ratio >= 0.6` with no token guard; R13 requires `ratio >= 0.8` to classify `Ai` while `words.len() <= 2`. For `words.len() ∈ {1,2}` and `ratio ∈ [0.6, 0.8)` the two clauses demand opposite results. Since "the clause IS the test," the R12 and R13 unit tests are mutually unsatisfiable.
   **Fix:** scope R12 to the high-token regime (`WHILE words.len() > LOW_TOKEN_COUNT`), leaving R13 the sole rule for `words.len() <= 2`; update the R12 fixture to use >2 tokens.

7. **`marley_assets` foundation types are unspecified — `SPEC-assets`.** The entire public surface and EARS (`AssetSource::Async`/`Bundled`, `AssetState::Loading`, `AsyncAssetType`, `AssetCache`, `Asset`) are expressed against a "UI foundation" that no M0/M1 spec defines (foundation-spike is throwaway; app-shell defines none). The M1 batch cannot build `marley_assets`.
   **Fix:** author the AssetCache foundation spec (owning `Asset`/`AssetCache`/`AssetSource`/`AssetState`/`AsyncAssetType`) and make it an explicit upstream dependency of `marley_assets`, or move `marley_assets` to the milestone after it. Do not gate M1 with a crate whose base types are unspecified.

8. **M0 `warp_util` value-type crate has zero spec coverage — coverage hole.** crate-triage schedules `warp_util` "first" in M0 (`FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`, path massaging, async git, worktree-name — "16 dependents, downstream blocks on them"). A grep across `docs/specs/` returns zero hits; the README lists M0 as "4 specs" while the build order lists 5 REIMPLEMENT crates — the missing one is exactly `warp_util`. `SPEC-assets` already dangles `make_absolute_url` on it.
   **Fix:** author `SPEC-marley-util` (M0, REIMPLEMENT) covering the value types/helpers the build order lists, or explicitly re-scope the unused subset and amend crate-triage + README. The value-type vocabulary M1/M2 bind to must have a defined contract before any code starts.

9. **Gate 15 visual harness spec does not exist — `standards/quality-bar.spec.md` + every `visual_acceptance`.** The fail-closed, `/commit`-enforced visual gate links to `../../pipeline/visual-testing.spec.md` as the normative AXUIElement + screenshot harness. `docs/pipeline/` is empty; no harness doc exists anywhere. Every UI spec routes its whole Visual/Behavioral Acceptance section (and large ACCEPTED-UNTESTABLE swaths of app-shell, ui-components, foundation-spike) to this nonexistent contract — no element-tree assertion API, no screenshot tolerance, no baseline/approval policy, no headed-launch driver. The M0 foundation-spike cannot pass its required visual gate against an undefined standard.
   **Fix:** author `docs/pipeline/visual-testing.spec.md` (or repoint the gate-15 link) before any UI code — define the AXUIElement assertion surface, screenshot capture+diff tolerance, baseline storage/approval workflow, and the headed-launch driver.

10. **Clean-room provenance breach — all 15 specs' `spec_source` + provenance section.** Every REIMPLEMENT spec declares `clean_room: "behavior-only; no AGPL source read"`, but the cited `warp_architecture/crates/*.md` docs are source-derived transcriptions ("Marley is forked from Warp"; each doc captures "real pub types/fns, key modules" from `lib.rs`/`main.rs`, plus private file paths and private static names like `FLAG_STATES`, `USER_PREFERENCE_MAP`, `AtomicTriState`, `CONTRACTION_REGEX`, `RESERVED_KEYWORDS`). By the project's own rule ("a reworded translation is still a derivative work"), working from a faithful structural transcription is not clean-room separation; at least six specs reproduce Warp's internal organization in their "Public surface" sections.
    **Fix:** insert a real behavioral wall — produce a behavior-only `spec_source` (observable I/O, no private module/type/static names, no file paths) authored by someone walled off from the source, and have the specs cite THAT. Until then, downgrade the `clean_room` line to reflect a fork-derived reference and obtain IP-counsel sign-off (already an open item in `clean-build-plan.md`) before any REIMPLEMENT code.

11. **Provenance gate does not cover the spec layer — `quality-bar.spec.md` clean-room gate.** The gate checks each REIMPLEMENT *code* change for the provenance line but does not gate the SPEC artifacts. Because the specs import Warp's internal type/module names into their public-surface contracts, the gate will pass code that faithfully implements a contaminated spec — the structural derivation enters upstream of the gate and is laundered through the spec.
    **Fix:** extend the provenance gate to spec artifacts — assert each spec's "Public surface" contains no Warp-internal-only identifier (e.g. `LiteCommand`, `ParsedExpression`, `ClassifiedCommand`, the `Component`/`Params`/`Options` trait triad, `enum_iterator::cardinality` array-sizing) and that `spec_source` points at a behavior-level doc. Add an inspect-phase checklist item diffing spec public-surface names against the warp_architecture crate docs.

### Highs worth fixing now

- **`SPEC-editor` R23 + BufferEvent — no subscription seam.** R23 emits `BufferEvent::Edited` "to subscribers," but Buffer exposes no subscribe/observe API; `edited_event_emitted` is untestable. `BufferEvent::SelectionChanged` is an orphan variant with no triggering clause. **Fix:** add a subscription seam (e.g. `subscribe() -> Receiver<BufferEvent>`); add an EARS clause governing `SelectionChanged` or remove the variant.
- **`SPEC-editor` R9/R11 — selection coupling with no selection API.** Undo coalescing keys on "no intervening selection change" and undo "restores selection," but Buffer owns no selection state. **Fix:** add the selection-ownership seam to Buffer (`set_selection`/`selection`) that undo restores, or rewrite R9/R11 to not depend on selection and name the owner.
- **`SPEC-text-offsets` R1 — internally false constructor claim.** "constructible only via `From<usize>`" contradicts `zero()`, `range()`, `add_signed`, and Add/Sub operators. **Fix:** reword to "the tuple field is private; construction is via the documented API (`From`, `zero`, `range`, arithmetic)."
- **`SPEC-text-offsets` R16-R19 (CharCounter) — missing unwanted-case clause.** No clause for non-monotonic input (byte before current position) or non-boundary byte — the exact bug class the crate exists to prevent. **Fix:** add an IF/Unwanted clause with defined behavior (debug_assert/panic/result) + AC + test.
- **`SPEC-terminal-blocks` R9 — encoding selection not in the contract.** R9's core verb is "SELECT its codec from the terminating DCS char / first OSC param," but `decode_hook(encoding, payload)` receives the encoding pre-selected; the selection logic (where an encoding bug lives) is untested. **Fix:** expose `encoding_for_dcs_terminator(b) -> Option<DcsEncoding>` and test terminator→encoding, or rewrite R9 to cover only decode-under-supplied-encoding and add a separate selection requirement+test.
- **`SPEC-syntax-highlight` R15 — color resolution + fallback has no API.** R15 resolves `HighlightId`→`ColorU` with a registered fallback, but `highlights_in_range` returns ids only and there is no resolver/fallback in the surface. **Fix:** add `color_for(id) -> ColorU` and a fallback registration; bind R15's two assertions to it.
- **`SPEC-syntax-highlight` R12 — DecorationEvent has no emission sink.** R12 emits `DecorationEvent::Updated{version}` but the surface declares the enum with no subscribe/channel/callback. **Fix:** add a delivery surface (subscribe callback, or have recompute return `Vec<DecorationEvent>`), mirroring `marley_settings`.
- **`SPEC-ui-components` Test Plan — headless unit tests against opaque structs with no seams.** The unit list asserts label text, icon order, chosen `VisualState` colors, scrim/card layout, tooltip visibility on opaque `#[derive(Default)]` components whose `render` returns opaque `AnyElement` — exactly the clauses the Visual section says have "no headless harness." Tests cannot be written against the contract for R3/R4/R6/R8/R9/R11/R13/R15/R16. **Fix:** expose pure decision seams (`Button::visual_state()`, `current_colors(appearance)`, switch knob-position accessor, `Tooltip::is_visible()`, icon/label order descriptor); move pixel-layout assertions to the visual harness only.
- **`SPEC-settings` R6 — load-before-register ordering contradiction.** `load(path)` constructs the manager; `register::<S>` runs after — yet R6 says load "populates each REGISTERED setting." At load time nothing is registered. **Fix:** keep the parsed `toml::Value` and resolve lazily at `register`/`get`; reword R6 to "WHEN `get::<S>()` is called after a valid file was loaded and S registered, resolve from the file's `toml_path` node." Make R8/R9 consistent.
- **`SPEC-completions` R15/R16 — stateless fn cannot cache or expose a snapshot.** Both presuppose a stateful engine holding a cross-call cache, but `suggestions(...)` is a free async fn with no receiver. **Fix:** add `pub struct Completer` with `async fn suggestions(&mut self, …)` + `fn snapshot(&self)`, or delete the cache requirement.
- **`SPEC-app-shell` R23/R24 — no theme-set API.** Active-theme change / default-theme clauses require theme selection on `RootView`, which exposes only `new`/`dock`/`toggle_dock`/`pane_group`; the keymap maps no theme action. **Fix:** add `set_theme(&mut self, &Theme)` / `active_theme(&self)`, default to Dark (R24), re-anchor R23/R24.
- **`SPEC-command-palette` R17 — nucleo gate not in the surface.** R17 mandates nucleo subsequence ranking with "non-matching → no result," but no public item ranks: the mixer merges pre-scored `QueryResult<T>` from caller sources. **Fix:** expose `pub fn fuzzy_rank(query, candidates) -> Vec<(usize,f32)>` and bind R17 to it, or move R17 out (ranking is a per-source concern) and drop the mutation-target line.
- **`SPEC-completions` R5 vs R6 — classification precedence undefined.** R5 makes the first pipeline token a `Subcommand`; R6 makes any `-`/`$`-prefixed token a `Flag`/`Variable`. For a first-position `$FOO`/`-x` both fire with no stated precedence. **Fix:** state precedence (command-head position wins regardless of sigil), so exactly one classification applies.
- **completer↔classifier — contradictory M1 `ParsedTokensSnapshot` contract.** completer R15 asserts the classifier consumes its `ParsedTokensSnapshot` and ships a seam test; the classifier explicitly refuses that input in M1, parses its own `Tokens`, and lists no dep on completer. **Fix:** pick one — classifier accepts `ParsedTokensSnapshot` (add dep, change `detect_input_type`), or drop completer R15's claim + listed consumer + seam test until the shared-snapshot milestone.
- **Command palette implemented twice — `SPEC-command-palette` (search-core) vs `SPEC-app-shell`.** search-core claims to be the one engine behind the inline command menu; app-shell rolls its own `filter_commands`/`ScoredCommand{score:u32}` (vs `QueryResult{score:f32}`) and does not depend on search-core. Both M1; `SearchMixer` has no M1 consumer. **Fix:** wire app-shell's palette onto `marley_search_core`'s `SearchMixer` (register commands as a `SyncDataSource`), or explicitly note in both specs + build order that M1's palette is intentionally a static list and search_core is built/tested in isolation pending M2. Unify the command model + score type.
- **`Appearance` name-collision — `SPEC-app-shell` vs `SPEC-ui-components`.** app-shell: `enum Appearance { Light, Dark }` + separate `ThemeColors`. ui-components: `struct Appearance { accent/surface/text/danger; metrics }` that every widget renders from. When RootView composes the widgets the shapes won't match. **Fix:** one theming vocabulary — keep `Appearance` for the Light/Dark discriminant, export a shared `ThemeColors` bundle, and rename ui-components' render input from `Appearance` to `ThemeColors`; ui-components depends on the theming spec.
- **Clean-room HIGHs — internal taxonomy reproduced in public surfaces.**
  - `SPEC-completions`: deletes-needed `LiteCommand`/`ParsedToken`/`ParsedExpression`/`classify_command`/`matchers` + legacy/v2 split (Warp's internal `src/parsers/` design) — keep only observable `suggestions()`/`describe()`/`SuggestionResults`/`SuggestionType`.
  - `SPEC-ui-components`: strike the `Component`/`Params`/`Options` trait triad and "borrowed from Warp's ui_components" — specify observable widget behavior only.
  - `SPEC-markdown-render`: re-derive the `FormattedText` line/inline model name set from the GFM features the R# clauses require; remove the "deferred de-Warp rebrand" of internal table tag strings.
- **`SPEC-syntax-highlight` reuse — `tree-sitter-toml` is unbuildable against core.** `tree-sitter-toml` is pinned at 0.20.0 (old tree-sitter 0.20 ABI) while the spec reuses tree-sitter core 0.26.9; they cannot link. **Fix:** replace with `tree-sitter-toml-ng` (0.7.0, MIT) at lines 9 and 97; add CI that all grammar crates resolve against one tree-sitter core minor.

---

## Fold into the relevant ticket (mediums / lows)

Fix during implementation of the owning crate; not gate-stoppers.

**EARS / testability mediums**
- `SPEC-editor` R15/R17 + FindOptions: no clause for `case_insensitive` effect; R17 only covers `wrap==true`; `find()`'s `wrap` field unspecified.
- `SPEC-editor` R19 + Selection: clamping "on construction" unenforceable with public fields and no buffer length in a bare Selection — make fields private / clamp at the SelectionSet/Buffer boundary and name the API entry point.
- `SPEC-editor` R20 + `LayoutLine.width`/`soft_wrap_width`: units undefined — state "monospace display columns."
- `SPEC-process-command` R13 `is_wsl()`: no injection seam — add `is_wsl_from(version_reader)` so both branches test on any host.
- `SPEC-process-command` R14/R15: R14 is a workspace gate (map to no-suppressions gate, not a unit test); R15 cross-OS parity is a design invariant — decompose into per-platform assertions or mark ACCEPTED with verification named.
- `SPEC-marley_core` R16: no function "applies DEFAULT_FLAGS" — add `apply_default_flags()` or specify `mark_initialized` applies them; reword the trigger.
- `SPEC-marley_core` R9/R18 + `SPEC-editor` R8: reframe non-pass/fail clauses as ACCEPTED-UNTESTABLE with mechanism — trybuild compile-fail for R9 no-fields, `static_assert` for R18 compile-time sizing, structural-sharing test for R8 (drop literal "O(1)").
- `SPEC-foundation-spike` R14: split the three negatives; add a test for "no keyboard input path" or mark that sub-clause ACCEPTED-UNTESTABLE.
- `SPEC-settings` R3 `pub const fn toml_path_hierarchy` returning `Vec<&'static str>`: const fn cannot allocate a Vec — drop `const` or return a const-friendly form.
- `SPEC-syntax-highlight` R10 vs R2: precedence of "language loaded"→true vs "oversized buffer"→false unstated, and `has_highlighting(&self)` cannot see the buffer — define `has_highlighting() = loaded AND last-parsed-size ≤ MAX_PARSE_BYTES`.
- `SPEC-syntax-highlight` R11 (MAX_SYNTAX_TREES) vs singular-Tree Purpose: clarify the multi-tree model and add `retained_tree_count()` so the cap/eviction is black-box testable.
- `SPEC-syntax-highlight` R5/R8: white-box mechanism claims — reframe to observable outcomes (`Tree::changed_ranges` for R5; a `#[cfg(test)]` query-invocation counter for R8).
- `SPEC-markdown-render` R2 vs AC#2: "count of #" vs "clamped 1..6" disagree with pulldown-cmark (7+ `#` is a paragraph) — pick library behavior, fix AC#2, fix the test.
- `SPEC-markdown-render` R7 vs R14: a ` ```yaml ` fence matches both CodeBlock and Embedded — define the exact discriminator; pick non-overlapping test inputs.
- `SPEC-markdown-render` R5/R4: GFM default `---` (no colon) has no `TableAlignment` mapping — add `Default`/`None` or map to Left; ensure every column has a defined alignment.
- `SPEC-markdown-render` R17: "unmodelable input → Err" has no concrete trigger (markdown is total) — name the realistic Err source (invalid embedded-YAML body) or make `parse_markdown` infallible and delete R17.
- `SPEC-terminal-blocks` R6: Precmd populates "the NEXT block's PromptInfo" before that block exists — specify the staged-prompt buffer + lifecycle (second Precmd replaces; InitShell/shutdown discards).
- `SPEC-input-classifier` R11: ambiguous whether `=` is in the metachar set or the set-definition equals — list the chars as explicit literals.
- `SPEC-input-classifier` R12: `is_first_token_command` derivation undefined inside `detect_input_type`, and numerator(R10 skips head)/denominator(`words.len()`) basis is an unstated off-by-one — define both.
- `SPEC-completions` R4: cursor-in-inter-token-whitespace result undefined — add a clause (`None`, or empty insertion context) + test.
- `SPEC-assets` R18: trybuild `.stderr` snapshot of an absolute path embeds `CARGO_MANIFEST_DIR` (flaky per machine) — snapshot a manifest-relative path or use a normalized/substring matcher.
- `SPEC-assets` R6/R7: "no alloc / no copy" is unverifiable without an allocation-counting harness — add one or demote to an implementation note.
- `SPEC-app-shell` R16: `score` ordering with no named matcher crate (reuses only `gpui`) — name the matcher/algorithm or reuse `marley_search_core`; pin the score definition.
- `SPEC-app-shell` R12 `neighbor`: "immediately adjacent" undefined for nested/2x2 trees — define the adjacency rule + nested fixture, or scope to flat layout M1 ships.

**Cross-spec / type mediums**
- `SPEC-command-palette`: central `Action` trait bound (`QueryResult<T: Action>`, etc.) is never declared/imported — declare it or import from its owner; state bounds (`Clone + 'static`).
- `SPEC-completions` `CompletionContext`: `EnvVars`/`PathSeparators`/`PathEntry`/`Description` referenced but undefined; attributed to `marley_command`, which is a spawn wrapper with no env/cwd model — define them in the completer crate or name the real supplier.

**Standards / coverage mediums**
- `SPEC-process-command` R15: unmapped to any test and not ACCEPTED-UNTESTABLE — map to a shared golden builder-config fixture diffed across platform runners, or reclassify with cross-host justification.
- `settings_value_derive`: double-claimed-covered (README line 55) but deferred to a non-existent sibling spec — add the spec or formally drop it and correct README + build-order M1 row.
- crate-triage M1 build order omits any terminal/block-model crate — add `marley_terminal` (block model + DCS hooks + session) to the REIMPLEMENT column.

**Lows**
- `SPEC-process-command`: WASM passthrough stated in Purpose but no EARS clause governs it — add a `WHERE target is wasm` clause or move WASM to out-of-scope.
- `SPEC-foundation-spike` R3/R1: decouple the PTY-spawn requirement from the window-open trigger (the spawn is what's tested); add a test/ACCEPTED note for R1's `ExitCode::SUCCESS`-on-clean-shutdown half.
- `SPEC-settings` R16 vs R13: Duration serialized as whole seconds breaks sub-second round-trip equality — state the precision contract (whole-second only, or encode ms/float).
- `SPEC-syntax-highlight` R18: uses `WHERE` for an error condition — rewrite as `IF…THEN`; enumerate `SyntaxError` variants in the public surface.
- `SPEC-syntax-highlight` mutation/gate-6: "no unsafe, no miri" holds only if every grammar binds via safe `LanguageFn` — pin tree-sitter ≥ 0.22 grammars and note the safe-binding requirement.
- `SPEC-editor` mutation header: uses `cargo mutants --in-diff` (per-commit weaker mode) vs the bar's full-mode MSI ratchet — normalize to plain `cargo mutants`.
- `SPEC-ui-components` frontmatter: missing the required `status: draft` key (and the `milestone`/bucket traceability tag flagged by another reviewer — confirmed `status` is absent) — add it to match the template and the other 14 specs.
- `SPEC-marley_core` deps: `enum-iterator` claimed "BSD/Zlib — within allowlist" but gate 8 is MIT/Apache/BSD only (no Zlib) — confirm the SPDX, pin a compliant version or add a documented cargo-deny allowance; fix the "within allowlist" wording.
- `SPEC-command-palette` R6: circular with R5 (mixer only trusts `accepts()`'s bool) — fold into R5 or expose a real mixer-layer filter surface.
- `SPEC-assets` R11: cache key "hash digest as lowercase hex" without naming the algorithm — state "lowercase-hex SHA-256 digest of url."
- `SPEC-assets` R19/R20 scope-creep: reintroduces the Warp-CDN/wasm asset machinery (`remote_asset!`/`bundled_or_fetched_asset!`/`make_absolute_url`) that crate-triage marked SKIP and that contradicts the local-first charter — drop it (keep only `bundled_asset!` + data-URI/url sources) or justify against the charter and own `make_absolute_url` in a real spec.
- crate-triage: `sum_tree` is not a bare crates.io package (it's `zed-sum-tree`/`gpui_sum_tree`); `get-size` (text-offsets) is unmaintained at 0.1.4 (use `get-size2` or feature-gate); `input-classifier` `reuses: [heuristics]` names a non-existent crate (real reuses are `rust-stemmers`, `regex`) — fix the structured `reuses` field so deny.toml allowlist generation is correct.
- vim line: build order tags `vim` M1 (deferred/opt-in) while `SPEC-editor` defers it to M2 — move the build-order line to M2.
- `SPEC-marley_core` R18 / R11, `SPEC-text-offsets`/`SPEC-process-command` public-surface names: low-risk clean-room shaping — restate R18 as the observable "adding/removing a variant requires no manual length update," treat `impl_offset!`/`CharCounter` as implementer's choice, and document the std-mirroring rationale for process-command's module split.

---

## Per-dimension summary

| Dimension | Verdict | # findings |
|---|---|---|
| EARS validity & testability (core/offsets/process/spike/editor) | needs-work | 16 (1 blocker, 4 high, 7 med, 4 low) |
| EARS validity & testability (blocks/syntax/markdown/settings/ui) | blocking | 19 (1 blocker, 5 high, 11 med, 2 low) |
| EARS validity & testability (palette/completions/classifier/shell/assets) | blocking | 16 (1 blocker, 4 high, 8 med, 3 low) |
| Cross-spec interface consistency | blocking | 11 (4 blocker, 4 high, 3 med) |
| Coverage & completeness vs M0+M1 build order | blocking | 9 (1 blocker, 1 high, 5 med, 2 low) |
| Standards compliance (test plan / mutation / visual-AX) | blocking | 6 (1 blocker, 1 med, 4 low) |
| Reuse-crate validity | minor | 4 (1 high, 1 med, 2 low) |
| Clean-room soundness | blocking | 8 (2 blocker, 3 high, 2 med, 1 low) |

*(Counts include cross-reviewer duplicates of the same root issue — e.g. the editor offset fork appears in three dimensions; the deduped blocker total is 11.)*

---

## What is solid

The EARS form-discipline is genuinely strong and consistent across all 15 specs: clauses are numbered, use the correct Ubiquitous/Event/State/Optional/Unwanted keywords, carry one testable response each, avoid weasel words ("gracefully/appropriately"), and map 1:1 to named unit/integration/visual tests with acceptance-table rows and MSI-100% mutation targets. The ACCEPTED-UNTESTABLE carve-outs for GPU paint, openpty/ioctl, the tokio bridge, and `cfg(windows)` are specifically and consistently declared, and miri (gate 6) is correctly reasoned N/A for M0/M1 because all GPU/objc2/wgpu unsafe lives inside the REUSE `gpui` crate. Coverage is strong for 14 of the build-order crates — marley_core correctly folds in `warp_features`, settings folds `settings_value`, input-classifier folds `natural_language_detection`, syntax-highlight folds `languages`, assets folds `asset_macro`, and the block model (a build-order omission) is well covered by `SPEC-terminal-blocks`. Scope-creep is otherwise clean (only the asset-CDN item over-reaches), and the SKIP of Warp's cloud/auth/channel surface plus the INVENT foundation-spike are sound. The defects are concentrated at the crate boundaries, in two foundation coverage holes, in a missing test-harness contract, and in the clean-room provenance wall — all closable with targeted edits, not a redesign. Once the 11 blockers are resolved the spec set is close to build-ready.
