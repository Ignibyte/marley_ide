# Architecture decisions — the local ledger

> Exported 2026-08-09 from the forge DB (table `architecture_decisions`; 43 entries,
> TICKET-409 — the scrap-forge pivot). This file is the LIVE capture surface:
> `/pipeline:complete` APPENDS new `## <code>` blocks;
> recall is `grep`. One entry per `## <code>` heading — never edit history.

## AD-claude-active-root-not-cwd-for-project-config-001
*App-scoped per-project config resolves against the restored active root, never the process cwd · status: active*

**Context:** A GUI app launched via Finder/`open` starts with cwd `/` (macOS LaunchServices) — the process cwd is meaningless to it. Marley boots its forge client from a per-project `.mcp.json`; the original read was cwd-relative AND early (before the shell restore knows which project is active), so every non-dev launch silently got no forge client (#381, the 2026-07-21 QA: fleet writes recorded "no forge client (.mcp.json)" on a machine whose repo HAS the file). The #376 fleet subscription hit the same class (inspect F2) and was fixed by keying off the RESTORED active project's root.

**Decision:** Any boot decision that reads a per-project file/config (the forge `.mcp.json`, and future analogs) resolves against the RESTORED active project's root — an absolute path from Project discovery/restore — with the launch cwd as a fallback only, evaluated AFTER the shell restore, never against the process cwd and never at a pre-restore site. The pure candidate-order resolver lives in a coverage-INCLUDED module (not the coverage-excluded app.rs boot shim) so the decision is gate-enforced. The existence probe is on the FILE (missing-file-only fallback): a present-but-broken active-root file returns its own path (surfaces as misconfigured) rather than falling through to an unrelated repo's config.

**Rationale:** The workspace root is the identity that matters (the VS Code `.vscode`/JetBrains `.idea`/git-worktree convention), not the process cwd. `active_project().root` is absolute, so `join(file)` resolves independent of cwd `/`. Keying off the restored root (not a pre-restore guess) is `PR-claude-boot-decisions-key-the-restored-active-root-001` (#376 F2) generalized to a class. Keeping the resolver out of app.rs preserves cov/MSI enforcement on the candidate-order logic (app.rs is a whole-file coverage exclude). Proven live: a cwd-`/` launch now builds the client (the forge overlay shows "loading sprint…", not the degrade).

**Consequences:** A future active-project SWITCH must re-resolve BOTH this and the #376 subscription together (both are boot-resolved-only v1 — D-OPEN-RETARGET). #384 (fleet "misconfigured" header) reads this outcome: arm (c) "no forge client" is now only the genuinely-missing / unreadable-everywhere case, so #384's misconfigured states are the real-misconfig cases, not the common Finder-launch degrade #381 fixed.

## AD-claude-brain-agent-session-supervision-001
*Brain launches, controls, AND observes agent terminal sessions — local_control + a session.read read-back delta · status: active*

**Context:** Chad flagged a load-bearing product concept at risk of being lost (2026-06-28): "I do want the concept of the brain to be able to launch sessions it understands and controls … the brain should see into the terminal of agents working if possible." Marley's agent orchestration (INVENT, M3) has the brain delegating tasks to project-scoped agents (worktree / folder-per-agent isolation), but the SUPERVISION/OBSERVABILITY angle — the brain watching each agent's live terminal — was only implicit (scattered across the local_control triage delta + the Hook-2 subsystem note). The remote variant is already captured separately as AD-claude-retain-remote-seam-001 + the remote-connection-seam intake.

**Decision:** First-class capability: the brain (orchestrator/RLM, served over MCP) supervises agent sessions via three operations on one seam — LAUNCH (own the session lifecycle in the agent's folder/worktree), CONTROL (inject input/commands, activate/close; SystemEdit vs UserTyped origin), and OBSERVE (read the agent's terminal output back as the per-command Block stream: command+output+status+cwd+SessionId). It is built on the local_control REIMPLEMENT crate (M3) with THE key Marley delta over Warp — a `session.read` output-read-back action (Warp's local_control has none) — run over a permissive transport (rmcp/jsonrpc) so the same surface doubles as an MCP agent tool (launch_session/write/read/close). Local-first; the remote variant composes this with the remote-connection-seam (M5). Captured as intake docs/planning/intake/brain-agent-session-supervision.md.

**Rationale:** It IS possible with the existing architecture — local_control gives launch+control; the terminal Block model gives a structured output stream already tagged with cwd/exit/SessionId; the only missing primitive is output read-back, a small known delta the triage already flags. Fully local + self-hostable (no Warp-cloud / no app.warp.dev). Without explicit read-back the orchestrator is fire-and-forget — it can start agents but not watch them; observability is the difference between delegation and supervision, and it is the natural data source for the agentic-workflow visualization panel (Hook 1). Building local_control WITH session.read from day one avoids bolting observability onto a control-only protocol later.

**Consequences:** When agent orchestration (M3) is scheduled, local_control must be built WITH the session.read read-back delta from the start (not control-only). The brain-orchestrator (INVENT) consumes it as an MCP tool. SessionId (marley_core) + HostId/local-vs-remote path (marley_util) already make sessions addressable. The terminal-blocks crate (M1) is the observed unit, so its Block model must expose the command/output/status/cwd record cleanly. The remote variant (supervise an agent on a remote runner) is M5, composing the remote-connection-seam. Promote the intake via /work at M3.

## AD-claude-brand-lint-gate14-001
*Brand keep-clean lint lives inside gate:14 docs_g, not a new gate number · status: active*

**Context:** #262 needed a permanent lint against Warp/Zed brand mentions re-entering crates/**/*.rs after the audited scrub. The canonical gate set is fixed at 1-15 (gate:16 was deliberately removed, TICKET-006); brand hygiene is a docs-quality concern; docs_g already scope-excludes the reference transcriptions for its TODO scan; the sweep audit offered fold-into-docs_g (A) vs a standalone descriptively-labelled gate (B).

**Decision:** Fold the check into gate:14 docs_g as a third check after rustdoc and doc-todos: brand=$(grep -rniwE 'warp|zed' crates --include='*.rs' 2>/dev/null || true); non-empty → echo hits + return 1. Extend the run_gate label to "docs (rustdoc -D warnings + doc-todos + brand-scrub)". Use POSIX -w (never GNU \b — gates.sh is Bash-3.2/BSD-safe). Land the lint in the SAME commit as the scrub so it is born green. Update the quality-bar.spec.md gate-14 row in the same change.

**Rationale:** No new gate number keeps the canonical 1-15 spec stable; the crates/-only scope auto-exempts docs/*_architecture/ with zero maintained exclude lists and makes gates.sh's own prose immune (it is a .sh); -w eliminates the 80+ …zed-substring false positives (StandardizedPath ×24 etc.) that a naive pattern would drown in; same-commit sequencing avoids the lint-born-red trap on the pre-scrub tree.

**Consequences:** quality-bar.spec.md's gate-14 row must track every docs_g check from now on (PR-claude-gate-change-updates-quality-bar-row-001 enforces the habit). -w accepts a documented residual gap: CamelCase/underscore identifiers (WarpBlock / warp_mode) slip the whole-word match — current exposure proven zero; the stricter POSIX-ERE boundary variant is pre-written in docs/planning/design-notes/brand-scrub/sweep-and-policy.md as the escape hatch if such an identifier ever appears.

## AD-claude-byte-offset-crate-to-char-offset-editor-monotone-cursor-001
*Converting a byte-offset crate's output to the editor's char offsets: ONE monotone cursor, asserted, never per-match · status: active*

**Context:** M22 #339 wired the `regex` crate into the editor's find bar. `regex` (like most text crates — aho-corasick, memchr, anything over &str) reports BYTE offsets. The editor's entire vocabulary is CHAR offsets (CharOffset, seam-contracts §1): the buffer, selections, bands, navigation all count chars. So every match boundary the crate reports crosses a byte→char seam, and #347 (capture-replace) plus any future crate-backed search will cross the same one.

The naive implementation is one conversion per match end (buffer.byte_to_char(b), an O(log n) rope walk), so M matches cost 2M rope walks. Worse than slow: it is exactly where a UNITS bug hides. On ASCII, bytes == chars EXACTLY, so an ASCII-only test fixture cannot distinguish a correct char conversion from a byte offset passed through raw. That is the coincidence that shipped the #336 defect (chars().count() vs display cells) and it recurs for every byte/char seam.

A first draft built a HashMap<byte,char> and indexed it with map[&b] — rejected in inspect because that indexing PANICS on a missing key, an unwrap-equivalent on the input path (§14), and "the key always exists, trust me" is the invariant-nothing-enforces class the batch keeps punishing.

**Decision:** Convert with ONE monotone forward cursor, not per-match, and ASSERT the invariant it rests on. The crate's iterator (find_iter) yields matches ASCENDING and NON-OVERLAPPING, so the flat boundary sequence (start_0, end_0, start_1, end_1, ...) is NON-DECREASING. Therefore one cursor over text.char_indices(), advanced forward to each boundary, resolves them ALL in O(n + m): a `next` = char_indices().next(), a `char_ix` counter, and a `char_at(target)` closure that advances `next`/`char_ix` while the next byte < target, returning char_ix. Three properties are load-bearing: (1) NO fallible lookup — unlike the HashMap, the cursor cannot panic on a bad key; a target past EOF walks to None and returns the total char count (the sentinel falls out naturally). (2) A debug_assert!(target >= last_target) RESTORES the loud failure the map draft had — the map's violation was a panic, the cursor's would be a stale-too-small index (wrong highlights/navigation, NO panic), so the assert makes a boundary regression a test crash in debug at zero release cost. (3) The tuple (char_at(start), char_at(end)) depends on LEFT-TO-RIGHT evaluation — guaranteed by the Rust Reference for tuple expressions — and a reordering would fail loudly on every fixture (start collapses onto end), so it is self-protecting. The pure fn takes text: &str, not the buffer, so it has no rope to walk per-match even if a future author forgot the rule — the shape forbids the mistake.

**Rationale:** The multibyte test is the ONLY thing that proves the seam, and it is not optional: find_all_regex("a😀bb", "b+", false) must return CHARS (2,4), not BYTES (5,7). Every #339 offset test has a multibyte char before the match for exactly this reason; on ASCII the assertion passes with the seam broken. A negative smoke (advance by 2 not 1) confirmed the multibyte row fails when the walk is wrong, so the row genuinely tests the conversion. Why one pass beats per-match: O(n+m) instead of O(m log n), allocates nothing, and — the real reason — it removes the per-match conversion SITE entirely, so there is no place for a units bug to live; the bug can only be in ONE loop, over which the multibyte test and the debug_assert both range. Why assert rather than "prove it holds": the invariant (find_iter monotonicity) was traced to regex-automata's source and holds today, but it is the CRATE's guarantee, not ours — a version bump or a different engine could change it, and the assert is how that surfaces as a test crash rather than a silent misread in a user's find bar.

**Consequences:** #347 (capture-replace) and any future crate-backed search MUST reuse this shape, not re-derive a per-match conversion; if the capture path re-runs the regex at replace time (one of its open forks) it crosses this seam a SECOND time — the cost to weigh against growing the return type. Any byte-offset dependency wired into the editor inherits this rule (aho-corasick, a different regex engine, a project-search backend): the seam is byte→char, the answer is one monotone cursor + the multibyte test + the debug_assert. The multibyte test is a REQUIRED row, not a nicety — an ASCII-only suite is blind to the whole class; codify it. Interacts with BF-claude-width-in-chars-vs-cells-strands-cjk-tails-001 (#336): same family (an offset in the wrong unit, invisible on ASCII), different axis — the through-line is that on ASCII every unit coincides, so ASCII proves nothing about units. The pure fn taking &str (not the buffer) is deliberate and should stay — it makes the per-match rope walk unrepresentable rather than merely discouraged.

## AD-claude-caller-gates-language-not-the-pure-syntax-primitive-001
*A pure syntax primitive parses its ONE grammar unconditionally; the CALLER gates on language, never the primitive · status: active*

**Context:** marley_syntax's pure fns (highlight_lines, enclosing_ranges, all_headers, matching_delimiters_in, and now #304's file_symbols) each run the Rust grammar (tree-sitter-rust) over the source. They are Rust-only by construction — the grammar is pinned, and the pure fn has no way to know the file's language (it takes a `&str`, not a path). tree-sitter is error-tolerant: it returns a tree for ANY input, so a non-Rust file (`.py`, `.md`) parsed as Rust yields a few/no valid Rust nodes rather than an error — silently wrong, not a failure. The #340 M1 inspect established the fix (PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001); #304 adopted it; but #329 (selection ladder) and #330 (sticky headers) still parse ungated (filed as bug #351).

**Decision:** The pure syntax primitive parses its one pinned grammar unconditionally and MUST NOT try to self-gate (it can't — it has no language signal). The CALLER gates: before invoking the primitive, check `code_syntax::language_of(&active_file().path) == Language::Rust` and skip the call (returning the inert empty result) for a non-Rust file. The gate lives at the app call site, keyed on the file path, mirroring #340's `refresh_bracket_match` (which drops its cache and returns inert when `!is_rust`). #315 (the language axis) generalizes this to a per-grammar dispatch; until then, every new syntax-primitive consumer copies the #340 gate, NOT the ungated #329/#330.

**Rationale:** Putting the gate in the pure fn is impossible (no language signal in a `&str`) and would also pollute the pure/testable core with app-side path/settings knowledge. Putting it at the caller keeps the primitive a clean, total, Rust-only function (cov/MSI 100, no language branch) and localizes the policy ("which files get syntax features") to the app, where the path + settings live. Naming the rule makes the #329/#330 omission a visible bug (#351) rather than a silent latent gap, and gives every future consumer (#305 fold, #315, #316, #317) a single pattern to copy.

**Consequences:** #329/#330 are non-compliant (bug #351 — they parse a non-Rust file as Rust; harmless-ish today, wrong as more grammars land). Each new syntax-primitive consumer adds one `language_of == Rust` gate at its call site; forgetting it is the recurring failure mode. #315 (the language axis) is the eventual generalization — a per-grammar registry the primitive dispatches on — at which point the caller gate becomes "the grammar exists for this language" rather than "== Rust".

## AD-claude-carried-selection-by-offset-span-not-row-001
*Carry a selection through a line-block edit by attributing endpoints to blocks via OFFSET SPAN, not raw row · status: active*

**Context:** M19 #300 (move/duplicate lines) reorders whole lines and must CARRY the multi-cursor selection set with the moved text (D-CARRY-NOT-REBASE: the shipped rebase_selections clamp collapses a moving cursor to its line start, so the pure seam computes the carry itself). The touched rows are coalesced into contiguous blocks; each block moves by a per-block delta. The naive carry attributes each selection endpoint to its block by the endpoint's raw row (buffer.line_col(off).0). But the universal "N full lines selected" gesture (Home → ⇧↓×N) represents the selection as `anchor in the block, head at column 0 of the row PAST the block`, and indent::line_span deliberately CARVES that col-0 row out of the touched set (the reference-editor convention). So the head sits on a row the block does not contain — a row-based lookup gives it no delta, and the carried selection silently distorts (shrinks on move-down, grows on move-up, doubles across both copies on duplicate-up) while the TEXT stays perfectly correct. No text/round-trip test catches it; only a full-line-selection SHAPE assertion does. Caught by an adversarial inspect critic, not by the (green) text tests.

**Decision:** Attribute a selection endpoint to its line-block by OFFSET SPAN — `[line_start(b0), line_start(b1+1)]` (the block's char range including its exclusive end where a carved boundary head sits) — never by the endpoint's raw row. A helper `block_at_offset(buffer, blocks, off, inclusive_end)` returns the block index. The boundary (offset == line_start(b1+1)) is INCLUDED when the operation's own edit does NOT already move that offset — a length-preserving move swap, and a duplicate-UP (copy inserted above) — and EXCLUDED when the edit DOES — a duplicate-DOWN insert AT the boundary offset, which rebase_through already carries onto the copy (double-counting it would push the endpoint past end-of-buffer). Blocks are separated by ≥1 gap row (coalescing makes maximal runs), so the inclusive spans are disjoint and the attribution is unambiguous.

**Rationale:** Row-based attribution is wrong precisely at the carved boundary that the universal full-line selection produces — the most common multi-line-move case, not an exotic edge. Offset-span attribution matches line_span's carve exactly (the boundary offset is line_start(b1+1), the block's exclusive end). The inclusive/exclusive rule is derived, not guessed: it depends on whether the op's own edit already shifts the boundary offset (rebase_through shifts an offset AT an insert point; a swap shifts nothing outside its span). Verified by a probe applying the real edits and asserting both text and the carried (anchor,head): move-down a 2-line selection → (2,6) not (2,4); dup-up → (0,4) not (0,8); dup-down stays (4,8).

**Consequences:** Any future line-block carry (#302 go-to-line has none, but #303 delete-line, and any move/reorder/duplicate variant) must attribute endpoints by offset span and add a FULL-LINE-selection carry test (Home+⇧↓), not just a bare-caret test — a bare-caret test passes with the bug present. The sibling decision is D-CARRY-NOT-REBASE (why the carry is computed, not rebased). The boundary-inclusivity rule generalizes: include the exclusive-end boundary iff the operation's own edits do not already move it.

## AD-claude-content-zooms-chrome-does-not-001
*Content zooms, chrome does not — appearance.font_size scales the reading surface, never the UI · status: active*

**Context:** M22 #337 made the mono font size a live setting (`appearance.font_size`, 8–32, ⌘=/⌘−/⌘0). The obvious implementation — "make everything take font_size" — would have passed every test I had, and is what a naive reading of the ticket asks for.

Prior state: `typography::type_scale(Role)` returned a hardcoded size for all four roles (Command 13, Output 13, Caption 11, Nav 12), and `app.rs` held a SEPARATE `TERMINAL_FONT_SIZE: f32 = 13.0`. The ticket's premise was that the const was the ONE source behind both surfaces. It was false: the terminal's command headers and output rows came from `type_scale`, the pane container from the const, the editor from its own path. They agreed only because #195 set both to 13.0.

Ruled out: (a) scaling ALL roles — that is View→Zoom, and it rescales the sidebar, status bar, and dock captions along with the text; (b) per-surface keys (editor 13 / terminal 12) — a recorded wishlist item, deliberately out of scope for v1 since one key replaced one shared const.

**Decision:** `type_scale(role, font_size)` returns the LIVE `appearance.font_size` for the CONTENT roles — `Role::Command` and `Role::Output`, which are the terminal's text and the same mono reading surface as the editor's code — and keeps the CHROME roles at their fixed design sizes: `Role::Caption` = 11.0, `Role::Nav` = 12.0, for ALL inputs including NaN.

⌘= therefore means "the text I read gets bigger" — the `editor.fontSize` model — never "the whole UI rescales".

Chrome roles must remain size-INDEPENDENT for every input. That is what makes `caption_header`'s "pass the default, it is ignored" call honest rather than a latent bug, and it is pinned by a test (`t337_type_scale_content_zooms_chrome_does_not`) whose Caption/Nav rows ARE the decision — without them, "scale everything" passes.

Corollary, equally binding: there is ONE home for the numbers — `font_zoom.rs` (`FONT_SIZE_DEFAULT`, `LINE_HEIGHT_RATIO`, `FONT_SIZE_MIN/MAX`). `TERMINAL_FONT_SIZE` is deleted. `fallback_cell`'s guard and the `define_setting!` default SOURCE the default rather than re-typing it.

**Rationale:** Two independent arguments converge:

1. **The split was already written down.** The `Role` enum's own docs drew it before this ticket existed: "terminal output — the base body text" versus "left-sidebar / nav CHROME". The function simply returned a constant for all four, so the distinction was documented but not acted on. #337 acted on it.

2. **A partial zoom is worse than none.** Before #337, scaling one source and not the other re-grids the PTY (the cell metric changes → `plan_resize` → `tcsetwinsize`) while leaving the scrollback pinned — a half-zoom on the app's most visible surface. The convergence is not tidiness; it is the feature working at all.

On the one-home corollary: the `fallback_cell` guard's literal had gone stale TWICE (it said 14 after #195 moved the app to 13; #230/#223 fixed it by hand). A second copy of a number is a second chance to drift, and this ticket gave `LINE_HEIGHT_RATIO` a second reader (the terminal completion popup anchors above an input row whose height is exactly `font_size × 1.2`), which is precisely how the first drift happened.

**Consequences:** - **Any new role must be classified** content-or-chrome at birth. A new CONTENT role that forgets `font_size` is a pinned zoom; a new CHROME role that takes it rescales the UI. The test's Caption/Nav rows are the guard — do not "simplify" them.
- **A new text surface must not invent a size.** Inspect found the terminal's completion popup hardcoding `px(13.0)` AND anchoring against an input row assumed ~30px — at 32pt that row is 38.4px and the popup sat INSIDE it, `.occlude()`ing the very line being completed. Any absolutely-positioned overlay near content must derive from `font_size × LINE_HEIGHT_RATIO`, not from a calibrated constant.
- **Chrome is now the exception, so it needs a reason.** #223 (right-dock/git/diff/menu 13px + fleet badge 9px) still wants Body/Micro roles; those are chrome-by-default and should be classified deliberately.
- **Per-surface sizes remain open** (editor vs terminal) — this decision does not preclude them; it makes them a key split, not a scale rewrite.
- Interacts with `AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001` ("no third source of cell width"): honored — every consumer still reads `fallback_cell(self.font_size)`, so the size became a FUNCTION of the one seam rather than a parallel path.

## AD-claude-coverage-model-001
*Marley coverage gate = whole-workspace 100% lines, explicit ACCEPTED-UNTESTABLE · status: active*

**Context:** quality-bar.spec.md states gate-4 as "100% on touched". The inherited Ignibyte gate used a 93% whole-crate floor with filename excludes for I/O adapters. Marley is a clean build with no legacy-uncovered code.

**Decision:** Bake RUST_COV_FLOOR=100 and run cargo llvm-cov nextest --workspace --fail-under-lines 100 with NO filename excludes. ACCEPTED-UNTESTABLE is an explicit, documented exclude list added to the gate with a reason (empty for the M0 pure-lib crates; populated only for FFI/GPU/headed paths when those crates arrive).

**Rationale:** For a from-scratch build with no pre-existing uncovered lines, whole-workspace 100% IS the bar's "100%-on-touched", just stricter and far simpler than building diff-intersection tooling. An explicit exclude list keeps untestable paths visible (no silent regex), per CONSTITUTION §0. MSI 100 mutation backs it: uncovered code yields no viable mutants, so the two floors reinforce.

## AD-claude-defer-cross-platform-half-to-runner-001
*Ship the host-portable surface; defer the other-OS half to a runner-gated follow-up ticket · status: active*

**Context:** A crate with platform-conditional code (#[cfg(windows)], #[cfg(target_os=…)]) cannot reach mutation MSI 100 on a single-OS CI runner: cargo-mutants does not respect cfg, so it generates mutants for the cfg'd-out code that are reported MISSED (the mutation is a no-op on the inactive platform → build unchanged → tests pass → survives), dropping MSI below 100. §0 bars #[mutants::skip]/exclusion files. Marley's CI is macOS-only; marley_command's reason-to-exist (Windows CREATE_NO_WINDOW + JobObject, R6/R7) is entirely #[cfg(windows)]. Spike-confirmed in scratchpad/cfgprobe (both inline cfg(windows) fns and separate-file cfg(windows) modules survive).

**Decision:** Implement the cross-OS-portable surface in the primary ticket with ZERO #[cfg(other-OS)] code (so MSI 100 is reachable on the host runner), and split the other-OS half into a SEPARATE ticket that is explicitly BLOCKED on that OS's CI runner. For marley_command: TICKET-004 ships Unix-only (builders, Unix passthrough, spawn/status/output, WSL detection, the std::process::Command ban); TICKET-004b (forge #6) lands the Windows half when a Windows runner exists. The spec's own ACCEPTED-UNTESTABLE clause already says the Windows kills are "required on the Windows runner job, not waived" — this just sequences the work to match the available infrastructure.

**Rationale:** Keeps the primary ticket at the full quality bar (cov/MSI 100) on the host runner without pretending to verify code it can't compile, and without a banned exclusion. The deferred half is verified honestly when its runner exists — not silently waived. Runtime-constant-on-host logic (e.g. is_wsl() always false on macOS) is handled differently: an injected path/env seam makes its constant-fold mutant killable on-host (don't defer what a seam can test).

**Consequences:** marley_command ships without Windows console-flash suppression until 004b; consumers get the portable builder + WSL detection now. General rule for every cross-platform crate: keep non-host-platform code out of the host build (separate runner-gated ticket), or it can't pass the mutation gate on a single-OS runner. Decision recorded in memory [[cross-platform-mutation-single-runner]].

## AD-claude-doc-phase-enforcement-001
*Documentation phase: CHANGELOG is hook-enforced (staged index); architecture docs are inspect-verified · status: active*

**Context:** Marley must never lose context: every ticket should record the what/why of the change (a CHANGELOG) and keep the system-shape docs current (architecture docs), unskippably. "Did you update the right architecture doc" is judgment and can't be a single hook predicate; "is there a CHANGELOG entry in this commit" can.

**Decision:** CONSTITUTION §21 — Documentation Phase. The CHANGELOG half is machine-enforced: enforce-changelog.sh (PreToolUse) blocks a commit whose STAGED index includes crates/<c>/(src|examples)/**.rs unless the root CHANGELOG.md is also staged (mirrors the §15 commit receipt's .rs trigger; a no-source commit is exempt). It checks the staged index (git diff --cached), not the worktree, so an untracked/unstaged decoy can't satisfy it, and the required file is root-anchored (^CHANGELOG.md$). The architecture-doc half is a required, inspect-verified Phase-5 step: a new/changed crate updates its docs/marley_architecture entry.

**Rationale:** Split by what's mechanizable: a staged-index, root-anchored CHANGELOG check is exact and unskippable in the add-then-commit flow; architecture-doc currency is a judgment call best handled by process + the inspect critic. Together they are the durable memory the pipeline writes for its future self — the CHANGELOG keeps the what/why, the architecture docs keep the shape.

**Consequences:** Every crate ticket (003+) must stage a CHANGELOG entry and update its architecture doc, or the commit is blocked / inspect flags it. The hook assumes staging precedes commit (a chained `git add … && git commit` stages after the PreToolUse hook and isn't introspected — documented). The hook's git-commit detector is kept in lockstep with enforce-commit-gate.

## AD-claude-edit-post-state-span-and-earned-undo-tags-001
*The N-cursor engine owns the post-selection (a per-cursor span), and every undo tag it sets must be EARNED · status: active*

**Context:** M22 #338 (auto-close) was the first editor action whose caret does NOT land at the end of its own insert: `()` puts it between the pair, wrap wants a RANGE back, type-over writes nothing yet steps forward.

Until then, "where does the cursor go after an edit?" had exactly one answer — the end of the inserted text — and `selections_after_multi_edit` simply computed it. Because that was unconditionally true, THREE downstream mechanisms had come to depend on it without checking, each documenting the dependency as a reason not to check:
  * `UndoHistory::coalesces_into` performs NO arithmetic contiguity test. Its doc: "Because the cursors did not move, each new insert lands precisely at the end of its own record's text, so appending char-wise is contiguous by construction." It trusts the group's `cursor_anchored` flag instead.
  * `redo` recomputes an ungrouped record's caret as `rec.at + inserted.chars().count()`.
  * `undo` recomputes an ungrouped record's caret as `rec.at + removed.chars().count()`, which assumes the consumed range ENDS at the user's caret.

The design's first instinct — let the app shim call `edit_at_selections_with(...)` then correct with the public `set_selection` — was evaluated and REJECTED on correctness: the undo group STORES the post-selection, so a post-hoc correction leaves `sel_after` describing carets the user never had (redo restores them) and breaks `coalesces_into`'s `prev.sel_after == group.sel_before` chain (a typed run at N cursors stops coalescing → ⌘Z one character at a time).

Ruled out: a contiguity check inside `coalesces_into` — records from different groups live in different coordinate spaces, so the arithmetic fails even for a legitimate run. The flag is load-bearing precisely because the check is not available.

**Decision:** **1. The engine owns the post-selection, and callers express it as a per-cursor SPAN.**
`selections_after_multi_edit(set, repl_lens, spans: &[Option<CaretSpan>])` where `CaretSpan = (anchor_rel, head_rel)` is an offset from that cursor's own post-edit base — deliberately NOT bounded by the replacement length (type-over inserts nothing yet lands at `base + 1`). `None`/`&[]`/short = the identity `(repl_len, repl_len)` = the pre-#338 math exactly. A range-valued span is how wrap returns a preserved selection at all.

The post-selection is computed INSIDE `edit_ranges_restoring_placing`, before `end_group`. **No caller may edit-then-`set_selection` to fix a caret.** `set_selection` being `pub` is a trap here, not an affordance.

**2. Every undo tag the engine sets is COMPUTED, never asserted.**
  * `begin_group(restore, cursor_anchored)` takes `ends_at_insert = spans.iter().all(Option::is_none)`.
  * The single-member no-group fast path is taken only when BOTH history fallbacks would be right:
    `fallback_is_right = ends_at_insert && restore.selections() == set.selections()`
    (`ends_at_insert` is redo's precondition; `restore == set` is undo's — the edit ranges ARE the user's cursors).

**3. The rule for every future edit action:**
  * If your caret is not at the end of your insert → you must not claim `cursor_anchored`.
  * If your edit range is not the user's cursor (backspace, and anything like it) → your `restore` must be carried in a GROUP; the ungrouped fallbacks will lie.
  * The identity case must stay byte-identical to the pre-generalization math, and that must be PROVEN by a property test over randomized input, not argued — this fn is on the app's ONE insert path, so every keystroke in the editor pays for it.

**Rationale:** **The corruption was real, silent, and only visible through redo.** With `cursor_anchored` hardcoded `true`, `coalesces_into` appended the next typed char onto the pair's record: `inserted = "()x"` at a site whose text read `(x)`. Undo survived because it uses only `inserted.chars().count()`; **redo replays `inserted` verbatim** and wrote `()x`. Driven at 2 cursors: `(` → `x` → ⌘Z gave `"  "` (one step swallowed both) → ⌘⇧Z gave `"()x ()x "`. The auto_close=OFF and N=1 controls both round-tripped correctly, so it was specific to a non-identity span at N≥2. This is the same text-corrupting class M19 #299's inspect found, reintroduced by the one mechanism that could now break the precondition.

**Why the span rather than a bespoke fix per action:** the identity case reproduces today's behavior for all 25 existing call sites with no change, so the risk is bounded by a property test rather than by a review of every caller. And it is the only shape that can express wrap — the engine could not return a range at all.

**Why "earn the tag" rather than "document the tag":** the flag was ALREADY documented, accurately, in two places. Documentation did not prevent this; the producer simply kept asserting a value that had stopped being true. Only computing it from the thing that makes it true is stable under change.

**The half-verification that let it through, recorded because it is the transferable part:** the design DID analyze coalescing and got half of it right — it proved a 2-char record can never join a PREVIOUS run (`record` and `coalesces_into` both require `new.inserted.chars().count() == 1`) and wrote "verified". But that guard is ASYMMETRIC: it constrains only `new.inserted` and never `old.inserted`, so a FOLLOWING char joins the pair's group. Verifying one direction of an asymmetric predicate is not verifying it.

**Consequences:** - **Any new editor action must classify itself** against the two questions above. The cost of getting it wrong is corrupted text on redo, not a wrong caret — and no existing test will notice, because nothing ever made the invariant false before.
- **`if <bool>` yields no mutant** (cargo-mutants does not mutate a bare-bool `if`/`match`), so an ordering or a flag guarded only by a boolean needs an explicit test. #338's `has_selection`-first ordering is pinned solely by a cross-product sweep for exactly this reason.
- The pure-move path (`insert_pairing_at_selections` with no writing cursor) deliberately BYPASSES the engine — an all-type-over sweep would otherwise hit the no-op guard and not move at all. Type-over is a caret move: no undo record, no version bump, no dirty flag.
- `backspace_at_selections` keeps its exact pre-#338 body via `pairing: false`; the pairing variant is a sibling, not a parameterization of the shipped behavior.
- Interacts with `AD-claude-two-boundary-maps-for-phantom-text-001` (#331) — both are about the editor having more than one notion of "where a cursor is". They do not conflict: #331 is buffer→display, this is pre-edit→post-edit.
- Follow-ups this decision does NOT cover: syntax-aware suppression (no pairing inside strings/comments) and the `'` bound positions, both of which need the grammar (#315).

## AD-claude-editor-offset-column-model-001
*The editor's offset↔column model: a pure single-source-of-truth line_layout, char-width-1 + tab-stops, over ropey's line model shared by render + caret · status: active*

**Context:** M15 #250 made the editor render from the marley_editor::Buffer (not the lossy, tab-expanded, 200-col-truncated CodeViewState.lines) with a visible caret whose on-screen column must equal the char offset — or the caret mis-places and a later save corrupts. Constraints: the existing render used flexbox syntax-span divs on a monospace-by-convention font (no measured per-column grid); Buffer::point_at.column is BYTES (not display columns); ropey's len_lines/line model breaks on more than \n (CR/FF/VT/NEL/LS/PS) so it can diverge from split('\n'); §0 forbids suppressions (an unused pub method is dead-code under -D warnings).

**Decision:** A pure `code_view::line_layout(line, tab_width) -> LineLayout { display, col_starts }` builds BOTH the tab-expanded display string and the char→column map in one pass; `expand_tabs` is reimplemented as `line_layout(..).display` (one source of truth). v1 column model = each non-tab char is ONE display column, a tab advances to the next tab-stop (a `char_width(ch)` hook is the growth path for wide/CJK). The editor uses ropey's line model THROUGHOUT — `Buffer::line_text(row)` for the render and `Buffer::line_col(c) -> (row, char-in-line)` for the caret — so render and caret are self-consistent by construction (the caret's row indexes exactly the lines the render draws). The caret is drawn as an overlay at `col × the measured monospace em_advance cell` (reusing the terminal's measurement), NOT by rewriting the line render into a measured grid. The #246 read-only split pane keeps its split('\n')/lossy-lines world untouched. offset_of_col (column→offset, the inverse) ships with its first consumer (#254 mouse), not speculatively.

**Rationale:** One pass producing both display + map guarantees the render and the caret can never disagree (the corruption vector). Ropey-throughout dissolves the len_lines-vs-split('\n') reconciliation (self-consistent within the editor; the split pane is a separate consistent world — a lone-CR cross-pane cosmetic difference is acceptable). char-width-1 matches the pre-existing expand_tabs (no regression) while being shaped to grow. The caret-as-overlay avoids a large measured-grid render rewrite for #250 while still being pixel-exact (monospace text lands on column boundaries; the driven capture confirmed a caret bar aligned at line 1 col 0). Deferring offset_of_col respects §0 no-suppressions (an unused pure-seam method is dead code).

**Consequences:** #251 (input) moves the caret + types into the buffer (Enter⇒\n) and can upgrade the bar to a block/blink; #254 (mouse) adds offset_of_col (column→offset) over the same col_starts + its round-trip test; #255 (selection) may need a measured per-column line render (selection rects) — the overlay approach is a v1 that a grid render supersedes then. True wide/CJK/grapheme width + horizontal scroll for un-truncated long lines are deferred follow-ups (v1 clips). Any code that changes the editor's line-count source must re-point the scroll clamp too (PR-claude-scroll-clamp-must-track-the-render-line-count-source-001).

## AD-claude-embedded-browser-substrate-001
*Embedded browser substrate: wry-as-child WKWebView (confirmed GO by the #402 probe); CEF-OSR the named revisit; CDP agent lane separate · status: active*

**Context:** Marley's Browser section needs the project Forge web UI as a live pane inside a gpui app whose macOS window is ONE self-composited Metal NSView (no native subview tree). Two embed shapes existed: (A) wry-as-child WKWebView through gpui's exposed RawWindowHandle::AppKit — light/native but the child composites ABOVE gpui's scene (overlays draw behind it) and first-responder/IME handoff was unproven; (B) CEF off-screen-render into gpui's paint_surface — clean composite + CDP but a hundreds-of-MB Chromium bundle. Decided on paper at the M26 #389 spike; the two load-bearing unknowns (z-order, focus/IME) were de-risked by the M29 #402 env-gated headed probe (crates/marley_webview_probe — the tree's one wry entry point).

**Decision:** The Forge Browser pane uses wry-as-child WKWebView (v1). The z-order collision is managed by the hide-the-webview-while-any-gpui-overlay-is-up shim. The agent-browser CDP lane stays a SEPARATE headless-Chromium substrate; CEF-OSR is the named revisit only if a shared surface is forced. CONFIRMED GO (2026-08-06, #402 probe, operator-verified all three checklist points): pane-rect position/resize tracks via capture+diff set_bounds (Logical→Logical, no scale math); the hide-shim holds under BOTH set_visible(false) and zero-bounds; keyboard+IME hand off cleanly both directions (page-IPC focus_parent return trip; wry 0.56 ships first-party focus()/focus_parent()).

**Rationale:** The Forge web UI is a trusted first-party page in a single pane — the light, native, no-Chromium-bundle path wins if its two risks retire, and the probe retired both. wry consumes the same raw-window-handle 0.6 gpui exposes (sole 0.6.2 in-lock — no dual-rwh hazard); core-video is already in-lock so the CEF pivot door (paint_surface) stays open at zero cost.

**Consequences:** Production callback rule: wry retains every handler closure INSIDE the WebView it builds — a callback capturing state that owns the WebView must capture WEAKLY or WebView::Drop is unreachable (PR-claude-callback-stored-inside-owned-resource-captures-weak-001; binds #405/#406). The hide-shim must be driven by ONE decision seam over the full overlay inventory (#405's 26-state overlay_is_up). wry's Linux-only gtk3 tree carries 10 unmaintained RUSTSEC advisories — per-id justified deny.toml ignores, revisit when wry moves off GTK3. WKWebView exposes no CDP — the agent-browser lane stays separate until a shared-surface need forces CEF unification.

## AD-claude-forced-prior-art-sweep-at-plan-001
*The prior-art sweep is REQUIRED at Plan — its highest-yield leg is reading our own permissive dependencies · status: active*

**Context:** §20's wall says what we may NOT read (Warp AGPL / Zed GPL source — a reworded translation is still a derivative work). It never said what we SHOULD read. So specs locked decisions without first asking whether something we already ship owns the seam; nothing forbade the sweep, nothing required it, so it happened by luck. The owner (chad, 2026-07-17) raised the alternative — read Warp/Zed source, accept GPL, refactor later — and it was rejected on a factual basis: provenance is not refactorable. You cannot un-read source; rewriting from memory is still generally derivation, which is why clean-room design is a two-team ritual. The choice is not GPL-now-or-later, it is GPL-decided-now, and the cost would be the proprietary brain tier. Owner agreed: strengthen the reference step instead. The decisive evidence is the M22 batch itself — its bugs were never reinvention, they were Marley's own invariants (an undo tag that lied, a memo key missing its inputs, a byte-vs-char seam). Reading Zed would not have prevented one. Reading PERMISSIVE deps paid twice in five tickets.

**Decision:** Every pipeline spec carries a "### Prior art" subsection inside "## Reference (§20)", filled at PLAN before any decision is locked; enforce-warp-reference.sh blocks a staged spec that leaves it empty. Three legs: (1) the behavior maps (docs/warp_architecture, docs/zed_architecture, observed captures) — research, not source; (2) published material (docs, blogs, protocol specs); (3) OUR PERMISSIVE DEPS — gpui, ropey, regex, alacritty_terminal, tree-sitter — the highest-yield leg, because reading their source is ADOPTION, explicitly outside the wall. Ask plainly: does a crate we already ship own this seam? A sweep that finds nothing is a PASS ("none: checked gpui/ropey/regex, no owner"); silence is not. A locked decision dying because the substrate already does it is a WIN to record, not a deviation to hide. The wall does not move.

**Rationale:** It paid twice in one batch, both times by killing work before it was written. #336: reading gpui settled both design forks and killed the implementer's own proposal (content_size is children-only — right instinct, wrong knob); it set the precedent that reading a permissive dep is adoption. #339: reading the regex crate DISSOLVED a locked decision — D-EMPTY-ADVANCE proposed hand-rolling empty-match advancement that Regex::find_iter already does, and regex-automata 0.4.13 util/iter.rs:30-36 documents it verbatim. The crate's rule is BETTER than ours would have been: it advances only on an empty match overlapping a previous one, where a blind advance-by-one would make ^ skip every other line start. The same sweep made FindError mirror regex::Error's real two shapes instead of guessing, got escape_literal for one line, and made the size_limit REQ a call rather than an algorithm. A gate not advice: an unenforced rule is a suggestion, and the sweep is cheapest exactly when it is least likely to happen — under momentum, when a decision feels obvious. The permissive leg is highest-yield because it is the only one that can DELETE work rather than inform it: a behavior map tells you what to build; a dependency can tell you it is already built, better, with the maintainer's edge cases handled.

**Consequences:** Every spec needs the section, including queued pre-authored ones. The template carries it as a comment, which correctly still FAILS the hook — a template must never pass. #339's was filled at Phase 3 and is the worked example; #340's needs it at promotion. The hook gates PRESENCE not quality (same posture as the Reference section); §18.1 inspect judges the sweep, but presence forces the question, which is most of the value. It fits the pre-authored-spec method exactly: Plan RE-VERIFIES rather than authors, and the sweep is that instinct pointed outward. It does not license reading copyleft source, ever. Follow-up: docs/zed_architecture is thin on the editor side; leg 1 will lean on it now.

## AD-claude-forge-write-ack-must-check-mcp-iserror-001
*A forge write-ack parser must treat result.isError=true as failure (forge returns failed writes as HTTP 200 + isError) · status: active*

**Context:** marley_forge_client #74 added the first app→forge WRITES (ticket-claim, ticket-comment) over MCP-over-HTTP. The write-ack parser must decide success vs failure. MCP tools/call can signal failure two independent ways: (1) a JSON-RPC protocol `error`, or (2) an application/tool-level failure via `result.isError == true` with the error text in `result.content`. A live probe (curl replicating marley's exact request against the running forge) settled which forge uses: a bad ticket_id returned HTTP 200 + `{"result":{"content":[…],"structuredContent":{"code":"not_found",…},"isError":true}}` — NOT a JSON-RPC error and NOT a non-2xx status. A successful comment returned HTTP 200 + a result with NO isError.

**Decision:** parse_write_ack is FAIL-CLOSED and checks BOTH failure channels: Err on a JSON-RPC `error`, Err on `result.isError == Some(true)` (surfacing the content text as the message), Ok only on a `result` present without isError, Err(Protocol) if neither, Err(Json) if it doesn't deserialize. The RpcResult struct carries `#[serde(rename="isError", default)] is_error: Option<bool>` so a missing field defaults to None (a normal success). Non-2xx is already rejected upstream by jsonrpc_from_http.

**Rationale:** Without the isError check, a FAILED forge write (permission denied, ticket not found, already claimed) — which forge returns as HTTP 200 + a well-formed JSON-RPC result — would deserialize cleanly and read as SUCCESS. The app + user would then believe a mutation happened when it did not: a UI that lies about state. Checking only the JSON-RPC error (the obvious/naive approach) misses the real failure channel. The live probe confirmed forge uses isError, so this is a load-bearing correctness/integrity guard, not speculative defense. Applies to EVERY future forge write (claim, comment, and any later mutating tool): reuse parse_write_ack, never assume a 200 + result means the write succeeded.

**Consequences:** All forge writes route through parse_write_ack. Future write methods (e.g. ticket-update from the cockpit) must use it, not tool_text (which extracts content regardless of isError, for reads). If forge ever adds a THIRD failure channel (e.g. a non-isError result whose content encodes an error), parse_write_ack would need extending — the live-probe habit (verify the real failure shape) is the guard against that.

## AD-claude-gate15-visual-scope-001
*gate-15 visual/AX is conditional + fail-closed, in-scope = BUILT component crate · status: active*

**Context:** The visual/AX harness (marley_visual_harness) is itself an unbuilt INVENT crate. Several specs (app-shell, foundation-spike, ui-components) already declare non-N/A visual_acceptance clauses, but their component crates don't exist yet. A naive "any spec with a visual clause" gate would fail-closed on the whole repo.

**Decision:** gate-15 scans docs/specs for a non-N/A visual_acceptance whose `component:` crate EXISTS under crates/. None in scope → printed SKIP (clean). A BUILT component that declares a visual clause but has no marley_visual_harness assertion → FAIL CLOSED. The harness crate lands with the first UI crate.

**Rationale:** Scoping to built components lets the gate skip-clean today yet bite the moment a UI crate ships without its visual assertion — closing the blank-terminal-shipped-14/14-green failure mode without blocking pure-lib milestones. Fail-closed (not skip) on a declared-but-unasserted clause is the security-relevant arm.

## AD-claude-generic-encoding-standardize-core-001
*Path normalization is generic over the path encoding so both flavors are covered + mutated on one CI runner · status: active*

**Context:** marley_util::standardize_path must produce a host-flavored normalized absolute path (Posix on Unix CI, Windows on Windows). A naive host-only impl leaves the non-host flavor's branch (and the Posix↔Windows mutant) dead/unkillable on a single-platform runner, blocking the 100%-coverage + MSI-100 gate without a Windows CI box.

**Decision:** The normalization core is a single generic fn `standardize_under::<E: typed_path::Utf8Encoding>(input, cwd)` (instantiated for Utf8UnixEncoding + Utf8WindowsEncoding); `standardize_with(p, flavor)` selects the variant; `standardize_path` uses a `#[cfg]`-gated `HOST_FLAVOR` const. typed-path parses + normalizes host-INDEPENDENTLY, so in-crate tests drive BOTH flavors (Windows via absolute `C:\...` fixtures) on the macOS runner. cwd is an injected parameter (testable seam) with an empty-cwd→root fallback guaranteeing absoluteness.

**Rationale:** One generic source line set = one coverage/mutation surface, exercised by whichever flavor's test reaches it — so 100% line coverage + MSI 100 are achievable on one platform, no Windows runner, no exclusions. Generalizes AD-claude-newtype-macro-mutation-001 (put mutatable logic where the gate can reach it) to platform-conditional logic; pairs with PR-claude-cfg-conditional-as-const-001 (host selection is a const, not a mutable fn).

**Consequences:** Any cross-platform string/path logic in later crates (marley_command line endings, exe suffix, shell selection) should follow the same shape: a host-independent generic/parameterized core + a cfg-gated const selector, tested for all platforms on one runner. Requires a host-independent library (here typed-path); where none exists, parameterize the platform-specific bit explicitly.

## AD-claude-git-write-confined-to-add-restore-commit-001
*Git-write surface confined to add/restore/commit (argv, not shell) · status: active*

**Context:** M5 #116 introduces Marley's FIRST git write (the commit panel). Before this, all git usage was read-only (git diff/status). A terminal that writes to the user's repo is a real blast radius, and chad flagged security as first-class for this ground. We ruled out a general "run any git command" surface.

**Decision:** Marley's git-write surface is confined to exactly three operations — `git add -- <path>`, `git restore --staged -- <path>`, `git commit -m <msg>` — each run via `marley_command::blocking` with the path/message as a single argv argument (never a shell string; `--` stops option parsing), so a hostile path/message is an inert argument, not an injection. NO push, pull, fetch, `-f`/`--force`, rebase, reset, amend, checkout, clean, or history-rewrite. New git-write features must extend this confined adapter set deliberately, not add arbitrary git subcommands.

**Rationale:** Confining to add/restore/commit with argv-not-shell arguments makes injection structurally impossible and keeps the destructive surface (push/force/rewrite) off the table by construction, rather than relying on runtime validation.

## AD-claude-gpui-headed-visual-testing-subprocess-001
*Headed gpui visual/AX testing: subprocess model + osascript-AX + screenshot baseline for inner content · status: active*

**Context:** marley_visual_harness (gate-15) must launch a gpui app headed and assert its AXUIElement tree + screenshots. Three constraints, established by spikes: (1) gpui's Application::run() OWNS the macOS main thread (NSApp), so a fixture cannot be mounted in-process inside a #[test] and then asserted from the same process. (2) gpui 0.2.2 has NO accesskit dependency — a captured window exposes to the macOS AX tree ONLY the window (AXRole/AXTitle/AXSize/AXPosition) + its AppKit titlebar chrome (traffic-light buttons + the title static-text); all gpui-rendered inner content is Metal-painted and invisible to AX. (3) objc2 AXUIElement FFI would make the crate carry unsafe (miri/SAFETY burden). The env can run headed GUI only after the controlling terminal app is granted Accessibility + Screen Recording.

**Decision:** The harness uses a SUBPROCESS model: HeadedSession::launch spawns a gpui fixture-runner BINARY (via marley_command, the non-PTY spawn seam) and observes it OUT-OF-PROCESS — snapshot() shells out to osascript/System Events for the AX tree, capture() runs `screencapture -R<x,y,w,h>` over the window's AX frame; Drop kills the child. AX assertions are WINDOW-LEVEL (title/size/role) via osascript; inner-content assertions (header-above-body, text values) ride the SCREENSHOT baseline (which R10 already mandates). No objc2 FFI → the crate is unsafe-free. The display/subprocess shim (os_shim/launch/selftest + the runner bin) is ACCEPTED-UNTESTABLE — mutants::skip + a single documented coverage exclude in scripts/gates.sh rust_cov — and is proven by a #[ignore] headed self-test (TwoElementFixture) against a committed screenshot baseline, run in a headed lane, NOT the per-commit gate.

**Rationale:** The in-process model is impossible (main-thread block). osascript was de-risk-proven to work in this env and avoids unsafe objc2 FFI (no miri/SAFETY surface, gate:13 clean). gpui's window-level-only AX means inner content MUST use the screenshot path regardless, so the AX API carries window-level clauses and the baseline carries inner content. Keeping the flaky/slow headed self-test #[ignore] (a separate lane) lets the per-commit gate stay display-free and lets cargo-mutants run without launching a window per mutant; the shim's exclusion is the policy-sanctioned ACCEPTED-UNTESTABLE for FFI/GPU/headed paths.

**Consequences:** Every downstream UI crate ships its own runner bin + FixtureWindow impl. Inner-element visual assertions are screenshot-based, so baselines must be captured on the same display class (pin the runner; retina 2x). The harness needs a headed macOS desktop with Accessibility + Screen Recording, NOT a headless CI box. The spec's in-process FixtureWindow.build()->AnyView shape was dropped (the observing harness never builds the view in-process). R27 SyntheticInput + the xtask visual CLI are deferred.

## AD-claude-gpui-menu-accelerator-001
*gpui app-menu keyboard accelerators require set_menus + bind_keys, in that order · status: active*

**Context:** Wiring ⌘Q for the app (#380). A bundled gpui app ignores ⌘Q when it registers no menu, no action, and no binding — gpui ships NO built-in Quit action (only NoAction) and NO default menus. The mac menu item's key-equivalent is derived from gpui's OWN keymap via keymap.bindings_for_action(action) at set_menus call time (gpui-0.2.2 mac/platform.rs:303-410); Marley's SEPARATE window-level keymap.rs chord table is invisible to that path.

**Decision:** To give a gpui menu item a working ⌘-accelerator: (1) declare the action with actions!(namespace,[Name]); (2) register a global handler via cx.on_action (it is also load-bearing — the menu item is ENABLED only if is_action_available finds a global_action_listener for it); (3) bind the chord in gpui's keymap via cx.bind_keys; (4) cx.set_menus the menu — with bind_keys BEFORE set_menus. For a Quit action the handler is cx.quit() (→ [NSApp terminate:]), NEVER std::process::exit, so RootView-drop teardown runs on the accelerator exactly as on the AppleEvent quit.

**Rationale:** set_menus reads the keymap synchronously to bake each NSMenuItem's key-equivalent; a menu WITHOUT a matching binding renders with an empty equivalent (⌘Q dead) though a menu CLICK still dispatches. So both calls are required and the order matters. Verified in gpui-0.2.2 source (create_menu_item :322-340 derives from bindings_for_action; the no-binding branch :402-410 sets ns_string(\"\")) by two independent inspect critics, and empirically on the running app (AXMenuItemCmdChar='Q' present on the item ONLY because the binding preceded the menu). cx.quit()→platform.quit()→[NSApp terminate:]→will_terminate→App::shutdown→windows.clear() gives teardown parity by construction; process::exit would skip shutdown and leak/orphan.

**Consequences:** Future menu rows (Hide/Hide-Others/Show-All, File/Edit menus) follow the same 4-part shape — note Hide et al. are gpui App METHODS (cx.hide()) not actions, so each menu row needs its own action+handler+binding. Declaring the same action name twice (e.g. two actions!(marley,[Quit])) panics at App::new. A gpui type name that collides with a crate-local type (KeyBinding) must be fully-qualified (gpui::KeyBinding) at the call site.

## AD-claude-half-open-intervals-kill-the-equivalent-boundary-mutant-001
*Store projection intervals half-open so a snap-to-header boundary mutant isn't equivalent · status: active*

**Context:** #305's FoldProjection.slot_of snaps a HIDDEN buffer row to its fold header's slot. With hidden runs stored INCLUSIVE [start..=end], the boundary comparison (`he < br` deciding count-the-run-past vs snap-to-header) lands on the LAST hidden row (br == he). There, "snap to header" (= hs-1-prior) and "count the whole run then subtract" (= he - (he-hs+1) - prior = hs-1-prior) compute the IDENTICAL value — so flipping `<`↔`<=` changes no reachable output. cargo-mutants reported it MISSED; it is a TRUE EQUIVALENT mutant, unkillable by any test, which blocks the MSI-100 floor. §0 forbids a suppression, and no arithmetic reformulation of the inclusive form escaped it (the snap-vs-count identity is inherent at br==he).

**Decision:** Store the merged hidden runs HALF-OPEN [start, end) (end exclusive) instead of inclusive [start..=end]. This moves slot_of's upper boundary comparison (`br < end`) onto the first VISIBLE row AFTER the fold, where "treat as inside → snap to header" and "count past → subtract" give DIFFERENT answers (snap gives the header's slot; count gives the correct larger slot). slot_of(<first visible row after a fold>) then distinguishes the two → the boundary mutant is caught. The PUBLIC API (new(total, &[(header,end)] inclusive pairs), visible_count, buffer_row, slot_of, folded_headers) and every output value are unchanged — only the internal representation moved; all 13 truth-table tests stayed green, and fold.rs reached MSI 100 (47 mutants, 0 missed).

**Rationale:** An equivalent mutant is a property of how the code EXPRESSES a decision, not of the decision itself. When a boundary comparison sits on a point where both branches compute the same value, no test can kill the flip — but re-expressing the interval so the boundary lands on a point the branches DISAGREE about makes it killable without weakening any gate or changing behavior. Half-open intervals are the standard tool: they put the boundary on the exclusive end (a visible row here), which is exactly the position the two projection branches diverge on. Generalizes beyond folds to any prefix-sum/snap projection (sticky headers, multibuffer, soft-wrap) that maps a position inside a run to a boundary of that run.

**Consequences:** Future projection code (soft-wrap display map, multibuffer excerpts) should prefer half-open runs from the start. The lesson is durable: on a MISSED mutant that looks equivalent, first PROVE it equivalent (both branches → same value at the only differing input), then re-express the interval/boundary rather than reaching for mutants::skip. Related: PR-claude-every-caret-path-must-reach-the-reveal-hook-001 (the other #305 lesson).

## AD-claude-headed-visual-baseline-text-tolerance-deferred-001
*Headed visual baselines: strict pixel-match is brittle for text/focus; marley_spike asserts AX + non-blank, defers text-tolerant baselines to M1 · status: active*

**Context:** marley_spike's headed visual test (R2/R9/R10) first tried strict screenshot-baseline matching via the harness's Tolerance::gate15_default — which was calibrated on marley_visual_harness's STATIC, text-free TwoElementFixture (two colored boxes, pixel-stable). The marley_spike window renders TEXT ("echo hello-marley" / "hello-marley"). Across 3 headed reruns the strict match FAILED: the diff was dominated by (a) the macOS titlebar traffic-light focus state (the close button renders bright/colored when the window is frontmost vs dim when not — varies per run) and (b) gpui subpixel text anti-aliasing (the same glyphs land on slightly different subpixels). The render itself is CORRECT (the committed diff visibly shows the Block) — the integration risk is retired — but the pixels are not bit-reproducible.

**Decision:** marley_spike's headed test (#[ignore], out-of-gate) asserts the DETERMINISTIC window-level AX (role=Window, title "Marley Spike", size ~800×632) PLUS a robust structural check — the content region below the titlebar contains light text pixels (proves the Block rendered; a blank window / the Pty-drop blank-body regression has none) — and does NOT strict-pixel-match a committed baseline (the baseline PNG is not committed). The data path (grid contains "hello-marley") is proven deterministically by the HEADLESS integration test.

**Rationale:** A strict pixel-match baseline is genuinely brittle for any text-rendering window + the focus-variable titlebar; making it deterministic needs a titlebar RegionMask + a perceptual/SSIM tolerance calibrated for subpixel AA — a non-trivial harness extension that belongs to a real UI ticket, not a throwaway spike. The AX assertions are exact; the non-blank check is stable (light-vs-dark is a huge margin) and still catches catastrophic render failures. Over-investing in pixel-exactness for a crate that gets deleted at M0-green is waste.

**Consequences:** The FIRST real M1 UI crate MUST extend marley_visual_harness before screenshot baselines are usable for text panes: (1) a titlebar/chrome RegionMask (exclude the focus-variable traffic lights from the compare), and (2) a text-tolerant Tolerance (an SSIM/perceptual-distance threshold calibrated on rendered text, not the static-fixture's near-exact match) — the harness already has RegionMask + ChannelSet + a perceptual_distance primitive to build this on. Until then, headed UI acceptance = exact AX-window assertions + structural (non-blank / region-content) checks. marley_spike's R10 strict-pixel-match is explicitly deferred. Revisit Tolerance::gate15_default when the first text pane lands.

## AD-claude-highlight-query-windowing-001
*Window the tree-sitter highlight query by span-splice; defer the line-cache · status: active*

**Context:** Incremental highlight (#274) re-parses the tree cheaply (~1.0ms) but re-ran the SAME full-tree QueryCursor walk (spans_from_tree, ~5.7ms O(file) over 27k captures on 8k lines) on every keystroke, flooring end-to-end at 0.46 of a full highlight. Two candidate designs to window it: (A) SPAN-SPLICE — window the query (set_byte_range), rebase+splice the previous RAW cached span set, feed the UNCHANGED lines_from_spans on the spliced full set; (B) LINE-CACHE — reuse the cached per-line vectors outside the damaged line range (true O(damage)). Constraints: byte-identical to a full walk (the #274 equivalence corpus), the <1/3 end-to-end perf pin, and minimizing equivalence-iteration risk.

**Decision:** Ship span-splice (A). HighlightSession caches the previous RAW span set (last_spans); damage_window merges old_edited.changed_ranges(&new) with the edit span; a grow-loop (window_extent) re-queries spans_in_window to a fixpoint where every fresh span lies inside the window; splice_spans drops cached spans touching the replaced region or overlapping the window and rebases the survivors by the edit delta; lines_from_spans runs UNCHANGED on the spliced set. Defer the line-cache (B) unless the pin needs it.

**Rationale:** (A) concentrates ALL equivalence risk in the span splice — an identical span MULTISET yields identical per-line output by construction (lines_from_spans sorts+sweeps), which the corpus gates hard — so far fewer index-bookkeeping traps than (B)'s per-line Δlines shift math. (A) removes the dominant 5.7ms C query walk, which alone clears the <1/3 pin (measured ~0.22 ratio in the debug test build; the tree-sitter C walk is NOT debug-inflated, so removing it drops the ratio hard). All 21 syntax tests + the expanded corpus pass byte-identical first try. (B)'s true O(damage) is not needed for the pin.

**Consequences:** splice_spans + the rebase pass + lines_from_spans stay O(file) in span COUNT (not true O(damage)); if the pin ever tightens further, per-line-vector reuse (B) is the recorded next step. The grow-loop re-queries on straddle (usually 1-2 iters; a block-comment cascade grows the window to ~full → degrades to the #274 walk, still exact).

## AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001
*The editor's horizontal scroll: shift and clip are DIFFERENT elements, and its two geometry probes live in deliberately OPPOSITE coordinate domains · status: active*

**Context:** M22 #336 gave the editor a horizontal axis it never had (an over-wide line clipped and its tail was unreachable). Three structural facts forced the design, and each is easy to get wrong in a way that looks fine on ASCII at scroll_x == 0:

1. `code_row` (the element holding the code text) sizes to its own UNBOUNDED natural width (`whitespace_nowrap`) and is a flex SIBLING of the gutter and the #328 git lane. A negative margin on it therefore pulls the text LEFT ACROSS the gutter — the gutter is not protected by being "before" it.
2. gpui bakes an ancestor's positional offset into a child's reported absolute bounds — but via TAFFY (`layout_bounds` returns `Layout::location`, with each parent's origin accumulated down the tree), NOT via `element_offset()` (which is gpui's separate scroll-offset stack, pushed only by div's scroll handle / `anchored` / deferred draws, and which a plain margin never touches).
3. The editor's clamp needs a viewport width, but the existing geometry probe is a `w(0).h(0)` canvas that reports an ORIGIN, not a size — and it sits inside the element being shifted.

The rejected alternative is worth recording because it is the more "correct-looking" one: `uniform_list::with_horizontal_sizing_behavior(Unconstrained)` is a first-class gpui feature that provides wheel-dx routing, clamping (`scroll_offset.x.clamp(-scroll_max.width, px(0.))`) and content-width measurement for free — i.e. exactly the roadmap-B2 trade of adopting the substrate and deleting hand-rolled code. It was rejected ONLY because it shifts the whole ITEM, gutter included.

Also rejected, with measurement: per-row `overflow_x_scroll` containers sharing one ScrollHandle. gpui's `content_size` is the union of CHILDREN's bounds only, so `min_w` on the container feeds `bounds.size` — the SUBTRAHEND in `scroll_max = content_size − bounds.size` — making the clamp SMALLER, not equal. Without perfectly equal content_size the shared handle self-destructs: the clamp runs per-row cumulatively on one Rc<RefCell>, so a single BLANK LINE (no children ⇒ content_size = bounds.size ⇒ scroll_max 0) pins horizontal scroll at 0 forever and shears rows above/below it.

**Decision:** The editor's horizontal scroll keeps THREE structural rules. Each is load-bearing; violating any one produces a bug that is invisible on ASCII at rest.

1. **SHIFT and CLIP are different elements.** The row is `[git lane][gutter][clipper → code_row]`. The CLIPPER (`flex_1`, `min_w_0`, `overflow_hidden`) is the gutter's flex sibling and NEVER moves; `code_row` (`flex_shrink_0`, `ml(-scroll_x)`) carries the shift one level in. The gutter stays fixed BY CONSTRUCTION, not by arithmetic — no consumer subtracts a gutter width anywhere. `flex_shrink_0` and `min_w_0` are both required: without the former, flex squeezes `code_row` back to the clipper's width and there is nothing to scroll; without the latter, the clipper's `min-width:auto` resolves to the full text width and it stops clipping.

2. **The two geometry probes live in OPPOSITE domains, on purpose.** The `x0` probe is a canvas INSIDE `code_row`, so it RIDES the shift — which is precisely what makes `rel = click.x − x0` cancel the scroll and yield content-domain coordinates with zero click-site changes. The `code_w` probe is a `w_full().h_full()` canvas on the CLIPPER, so it stays screen-domain and is a stable divisor for the clamp. Neither is an accident; moving either one breaks something silently. `EditorFrameGeom` carries both, and the two writers must not clobber each other (the x0 write preserves `code_w`).

3. **`scroll_x` is POSITIVE-RIGHT and never reaches a gpui handle.** gpui's own offsets run negative-as-content-scrolls; Marley's is the exact negation. The two are deliberately different numbers, and `h_scroll.rs` converts nothing — the app does, at the single boundary that talks to a handle.

Consequence for clicks, stated as the rule: **do not add `+ scroll_x` to the click math.** It would double-count. The y axis needs no math for the same reason (`uniform_list` hands the row index to the closure), which is why no site has ever added `scroll_y`. The guard is `click_at_scroll_lands_on_char_under_pointer_headless`.

**Rationale:** Rule 1 exists because "put the scroll on the code element" is the obvious move and it silently destroys the gutter. Splitting shift from clip makes REQ-005 (the gutter never moves) a structural property rather than a thing every future rider must remember to respect — and inspect measured the split as a strict IMPROVEMENT over the prior layout, where a long nowrap `code_row` floored at min-content put shrink pressure on the 3px git lane and gutter.

Rule 2 is the subtle one and the reason this is an ADR rather than a comment. The probes look interchangeable — two canvases reading `bounds` — and their difference is invisible at `scroll_x == 0`, where every domain coincides. A well-meaning refactor that "cleans up" the duplication by hoisting x0 out of `code_row`, or by reusing x0's geom for the width, breaks clicks or the clamp with no compile error and no test failure outside the one pinning test. The attribution matters too: the first draft credited `element_offset()` and inspect caught it. On a comment planted as a landmine marker, a wrong citation is worse than none — a reader who checks it, finds it is about scroll offsets rather than margins, and concludes the invariant is bogus will "fix" the exact bug it prevents.

The `Unconstrained` road stays rejected only while the gutter lives inside the row. It is genuinely the better engineering if that ever changes: it deletes the hand-rolled clamp, the wheel handler and the width measurement in one move. Recording it here keeps that trade visible instead of leaving a future reader to re-derive why a first-class gpui feature went unused.

**Consequences:** - **BINDING ON #337 (font size / zoom).** Both probes feed `cell_w`, and #337 replaces the `TERMINAL_FONT_SIZE` const with a live setting — so cell_w becomes runtime-variable. A zoom changes the content width and the viewport in cells at once; the clamp self-corrects on the next frame (it recomputes from live geometry), but #337 must not introduce a THIRD source of cell width. One metrics seam, read by both probes.
- **Binding on any new h-scroll rider.** A new consumer of `scroll_x` must first answer which domain it is in. If it positions relative to `body` it needs a body-relative origin, which the frame does NOT currently publish — that is exactly why the horizontal thumb was deferred rather than shipped mis-aligned.
- **Binding on the display map (B-c).** Soft wrap breaks row↔line identity, which this design depends on (one row = one line = one clipper). The wrap work must revisit the shift site, not layer on top of it.
- The rule is enforced by one pinning test plus three doc comments; there is no compile-time guard. A `ScreenPx`/`ContentPx` newtype pair would give one and was not done — the cost across the click/drag/IME/hover sites outweighed it for a single ticket. Revisit if a third domain appears.
- Not generalized to the terminal: alacritty owns its own grid and is untouched.

## AD-claude-keycontext-pure-model-001
*KeyContext adoption = Marley-original pure model in keymap.rs, not gpui-native dispatch · status: active*

**Context:** #265 needed per-surface chord resolution (⌘D editor-vs-terminal). gpui 0.2.2 ships the full engine (KeyBinding-with-context, predicate grammar, DispatchTree) but its keystroke→action resolution walks the FOCUS TREE, and Marley has exactly ONE root FocusHandle with a self-driven 27-step on_key_down ladder — native adoption meant rewiring every surface onto per-element FocusHandles + typed actions!.

**Decision:** Implement the KeyContext MODEL as Marley-original pure code inside the existing gpui-free keymap.rs seam: bindings carry Option<KeyContext> (v1: Terminal|Editor; None=global), the active tab publishes a &'static [KeyContext] stack (Tab::key_context), action_for(chord, stack) resolves by deepest-rposition rank with global=0, and uniqueness is per (chord, context) — which makes ties structurally impossible, so no insertion-order tiebreak rule is needed (simpler than the reference's sort). The full predicate grammar, NoAction/Unbind, multi-stroke chords, and the gpui-native focus-tree migration stay deferred (roadmap steps 2-3 + #267).

**Rationale:** Keeps the keymap 100% mutation-testable and gpui-free (the pure-seam discipline); bounded diff (one module + one method + one ladder line) vs an app-wide focus rewire; the deepest-match/global-fallback semantics are behavior-compatible with the gpui Apache-2.0 model so a later native migration is a mechanism swap, not a policy change.

**Consequences:** The context stack is tab-granular (a FileTree pane inside a terminal tab publishes [Terminal]); overlay contexts don't exist (the overlay ladder keeps first refusal); the palette shows one static chord chip per command (⌘D labeled for its Terminal meaning). The guard-forbids-ties property means the tie-policy mutant (> vs >=) is only killable via a synthetic invalid-table literal test — recorded in the tests. Future editor-only chords land as ordinary Editor-context rows with zero collision triage.

## AD-claude-license-policy-001
*Marley cargo-deny license allowlist = permissive set, MPL-2.0 dropped · status: active*

**Context:** Marley must be owned outright by Ignibyte (the whole point of the clean-room build, escaping Warp's AGPL). The inherited deny.toml allowed MPL-2.0 and a long list of Ignibyte-tree advisory ignores.

**Decision:** License allowlist = {MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib} — permissive only. MPL-2.0 (weak/file-level copyleft) is dropped. Advisory ignore list emptied (Marley's tree surfaces none); a justified per-id ignore is added only when a real dep surfaces an advisory.

**Rationale:** Any copyleft (even file-level MPL) undermines "own it outright". The literal "MIT/Apache/BSD only" reading would reject common transitive ISC/Unicode-3.0/Zlib deps and cause churn; the permissive set covers them while still rejecting copyleft. Empty advisory ignores keep the gate fail-closed and honest (no dead Ignibyte baselines).

## AD-claude-mcp-first-class-control-plane-001
*MCP is a core Marley pillar: a bidirectional, first-class control plane over nearly everything, down to typing in the IDE · status: active*

**Context:** chad, 2026-07-15 (verbatim intent): "MCP enabled everything. MCP is going to be a huge part of this. It should be able to control nearly everything ... We need first class support ... all the way up to even typing in the IDE." Declared one of the TWO concepts that are going to be the core of Marley (the other: tmux-grade detached sessions, AD-claude-tmux-grade-detached-sessions-001). Current state: Marley consumes exactly one MCP server (the forge sidecar, agent-side); nothing exposes Marley itself. The architecture is pre-shaped for it: every feature ships as a pure seam + thin shim (mechanically exposable as typed tools), headless_drive already boots and drives the real app with injected input under cfg(test), and the palette's command ids are an enumerable verb set (the #204 "every cockpit cmd resolves to a verb" invariant). Mission-control (AD-claude-mission-control-hypermedia-surface-001) already locked "desktop for hands, web for intents" for humans.

**Decision:** Marley adopts MCP as a first-class citizen in BOTH directions. (1) Marley EXPOSES an MCP server whose tool surface covers the IDE end to end: workspace/tab/pane management, terminal run + block reads (terminal_blocks units are the addressable output shape), editor open/read/EDIT/TYPE (through the one insert mechanism, ime::replace_text), caret/selection/save, LSP-backed queries (diagnostics store, hover/definition/references), and palette-command dispatch by id. (2) Marley CONSUMES MCP for the agents it hosts, configured via a [[mcp.servers]] settings table mirroring the shipped [[lsp.servers]] round-trip pattern. Mutating tool tiers (type/run/edit/save) sit behind a per-server, per-tool-class permission model (approve/allowlist, the Claude Code shape); observation scopes are opt-in per pane/session honoring the TICKET-043 line ("the operator may not want the system seeing everything they do"). Local transports first (stdio/unix socket); remote MCP waits for the mission-control auth story. NOT scheduled yet — the IDE milestones continue first; intake pillar on disk at docs/planning/intake/mcp-first-class-control-plane.md.

**Rationale:** An agent that can only watch is half an agent; chad's proven manager-agent workflow needs hands. Exposing the pure seams as tools is the cheapest-possible marginal cost of the existing testing doctrine — the MCP server is a third consumer of seams that already exist for the UI and the tests, not new machinery. MCP specifically (over a bespoke RPC) because it is the open, agent-native standard: Marley's own forge sidecar, ignibyte-bridge (pty_agent), and every frontier agent runtime already speak it, so Marley plugs into the ecosystem in both roles for free. The permission model is day-one because the tool surface is exactly the dangerous kind (type into an editor, run in a terminal).

**Consequences:** Every future feature should keep its pure seam MCP-shapeable (typed in/out, no gpui) — already the house rule, now with a second reason. When promoted: a ticket BATCH (server core, read tier, write tiers + permissions, [[mcp.servers]] client config), not one ticket. The v1 slice is the read-only server (blocks/diagnostics/state) + terminal.run behind approval. Composes with the tmux pillar: session verbs (create/list/read/send/attach/surface-to-human) are an MCP tool family — together they form the VPS agent-fleet cockpit.

## AD-claude-mission-control-hypermedia-surface-001
*Desktop for hands, web for intents — mission control as a hypermedia surface; never port the organs · status: active*

**Context:** chad's web↔desktop tension (2026-07-14): (1) remote control of Marley from phone/app, (2) sharing forge/pipeline/agent-status/workflow state to the web. Apparent fork: build two apps duplicating one UI, limit the web to read-only, or port the terminal to the web ("clunky xterm silliness"). Ruled out: web IDE / phone terminal / any xterm.js port (VS Code tunnels demonstrates the cost; the web's own terminals escape the DOM to canvas); a second React/SPA frontend (client-side state replica = the cache-invalidation bug category, cf. rusty's .changed sentinel); read-only web (a false constraint — it assumes a second app that must reimplement logic to be allowed to write).

**Decision:** Pick the lane for the ORGANS, not the state. Terminal grid, editor, and embedded Chromium remain desktop-only gpui surfaces, never ported. Marley's STATE (forge, pipelines, agents, workflows) gets ONE remote "mission control" surface built as hypermedia T1 per ~/Projects/rusty/docs/hypermedia-desktop-rust.md: backend owns all truth; server-rendered askama fragments + scalar signals stream down over SSE; a closed enum of intent-level actions (approve/deny, answer AskUserQuestion, start/stop, comment, gated run-command) dispatches up into the existing command layer. The terminal shares as terminal_blocks BLOCKS rendered to HTML (Warp-share-style, streaming tail), never as a grid/PTY. Mobile = responsive PWA with push first; a thin native wrapper later, same protocol, never a second logic codebase. Flagship first slice on promotion: the AskUserQuestion push loop (question → phone → tap answer → autonomous run unblocks). Topology (Marley serves itself over Tailscale vs rusty as the always-on hub reading forge + a Marley event stream) is deferred; the protocol is identical either way. Intake pillar: docs/planning/intake/mission-control-hypermedia-surface.md.

**Rationale:** Both stated problems are one product: observation plus a small enumerable verb set — nobody types shell characters on a phone. Hypermedia collapses "two apps" into templates + one SSE route over the same Rust structs the gpui cockpit renders: no frontend build, no replica, no API drift, read-write at the intent level for free. Blocks-not-grids gives ~90% of remote-terminal value with 0% PTY-in-browser. The agent-era tools (Cursor background agents, Codex, Claude Code web/mobile) independently converged on exactly this split, and the AskUserQuestion loop directly unblocks this project's actual bottleneck (autonomous /work trains stalling on AFK forks, e.g. #202 sat for days). Converges with the embedded-browser pillar: the same askama templates + ThemeColors→CSS-variables bridge serve in-app (marley:// custom scheme into Chromium) and remote (HTTPS/SSE) — one HTML dialect, two transports.

**Consequences:** Marley grows its first outward-facing surface: security is the top design item (Tailscale-first, no public exposure v1; authn always; the closed action-verb enum IS the boundary; run-command gated; askama auto-escaping). Needs an event-stream publisher seam in the app (blocks, pipeline events, questions) and later a marley_mission crate (view structs + templates + verb enum, pure-testable to MSI 100; the HTML presentation tested web-style/driven-capture, honestly outside the MSI gate). Always-on push favors the rusty-hub topology (desktop sleeps/locks — cf. the locked-machine capture lessons); rusty should prove the T1 pattern first (deleting its React replica + .changed sentinel). The shelved html_pane/subset-gpui renderer stays shelved; it returns only if these panels are ever wanted without Chromium in-process. This is an M3/M5-adjacent PILLAR, not a near-term ticket; it consumes brain-agent-session-supervision's Block stream and composes with the remote-connection seam.

## AD-claude-newtype-macro-mutation-001
*Newtype crates: a macro emits the API; non-macro free fns hold the mutatable logic · status: active*

**Context:** marley_text_offsets needs two spec-exact pub structs (CharOffset/ByteOffset, private field) AND mutation MSI 100 with no exclusions. Probed empirically: cargo-mutants does NOT mutate macro_rules-emitted code (a macro-emitted `a+b` → 0 mutants; the identical non-macro fn → 4). A generic `Offset<M>` core gives one mutatable impl but forces `Offset`/`Char`/`Byte` to be public (a pub alias to a private type is E0446), leaking a seam crate's surface.

**Decision:** A `macro_rules!` emits only the spec-exact `pub struct`s + their boilerplate API (derives, From, the trivial a±b ops). The non-trivial logic that must be mutation-verified — signed arithmetic (`add_signed_usize`), the streaming converter (`CharCounter::char_offset`) and its boundary helper (`floor_char_boundary`) — lives in NON-macro free/inherent functions that cargo-mutants mutates. Trivial macro-emitted ops are 100% line-covered + unit-tested but generate no mutants (disclosed, not hidden).

**Rationale:** Gives a clean public surface (only the spec's types — no generic leak), DRY (one macro for two flavors), and MSI 100 on the real logic. The alternative (generic core) leaks types; pure-macro leaves the arithmetic mutation-blind. Result on marley_text_offsets: coverage 100% + MSI 100% (10/10 mutants), zero exclusions.

**Consequences:** Future Marley newtype/primitive crates follow this split. The macro-emitted trivial ops carry line+unit coverage but no mutation coverage — acceptable because the logic that can harbor real bugs is the non-macro part. Pairs with PR-claude-debug-panic-release-defined-001 (assert-invariant + always-compute) for coverable misuse arms.

## AD-claude-outer-wins-sweep-defers-broad-container-grammars-001
*The span sweep is outer-wins; grammars with broad container captures stay on the hand-lexer floor until it's innermost-wins · status: active*

**Context:** marley_syntax's `sweep_disjoint` resolves overlapping highlight capture spans OUTER-wins-clip-forward (sort by (start,end), keep the first at a position, clip later spans to start after it). tree-sitter's own highlight convention is INNERMOST-wins (the most specific capture over a byte range wins). For most grammars captures don't nest, and where they do in Rust — `(attribute_item) @attribute` over a whole `#[derive(Debug)]` — outer-wins is DESIRABLE (the whole attribute reads one Attribute color; a #315 REQ-006 enrichment). But #315 adopting tree-sitter-toml-ng surfaced `(pair (bare_key)) @property` captured over the WHOLE `key = value` pair: outer-wins let that broad Property span steal the value's color, so `foo = 1` rendered key=Type and `1`=Property (REQ-003 inverted) — and worse than the pre-#315 hand-lexer, which colored TOML correctly.

**Decision:** A grammar whose adopted highlight query has BROAD CONTAINER captures that fight the outer-wins sweep stays on the hand-lexer FLOOR (`code_syntax::Language::grammar_lang() = None`) for v1, rather than shipping mis-colored tree-sitter highlighting. TOML is the first such deferral (Rust/Python/JS/TS/TSX/JSON/Bash take the tree path). Switching `sweep_disjoint` to innermost-wins — which would fix TOML properly — is a deliberate, separately-ticketed follow-up.

**Rationale:** Reverting ONE language to its already-correct hand-lexer floor is a tiny, safe, zero-regression change scoped to a single `grammar_lang` arm. Switching the SHARED sweep to innermost-wins is a broad, load-bearing change that touches every language's rendering — including Rust's desirable whole-attribute absorption — so it belongs in its own ticket with explicit Rust-attribute regression fixtures, not smuggled into a per-language highlighting ticket where its blast radius is invisible. The clean-room posture also favors it: we adopt each grammar's query verbatim (nothing vendored/trimmed), so we do NOT edit the query to drop the broad capture — we route around it.

**Consequences:** TOML tree-sitter highlighting is deferred (its Lang::Toml + pinned grammar remain, version-tested; only the app's routing is None). The innermost-wins sweep is a tracked follow-up (must add Rust `(attribute_item)` fixtures before flipping it, since it changes attribute rendering). "Does this grammar have broad container captures?" becomes a checklist item when adopting any future grammar — pair it with the behavioral run-the-query verification (PR-claude-verify-adopted-grammar-by-running-its-query-001).

## AD-claude-pipeline-docs-layout-001
*Keep docs/planning/ as the pipeline working-scratch root (do not flatten) · status: active*

**Context:** The inherited apparatus (every hook + command + the gate-14 doc-todos exclusion) references docs/planning/{pipeline/active,tickets,intake,_templates}. Marley's durable docs are flat under docs/ (specs/, marley_architecture/, decisions/, the milestone ticket lists in tickets/). An early idea was to flatten docs/planning/ → docs/.

**Decision:** Keep docs/planning/ as the working-scratch root; create it + minimal templates. Durable committed docs stay flat under docs/; per-pipeline WORKING docs (active spec/notes, per-ticket docs, intake) live under docs/planning/.

**Rationale:** docs/planning/ is baked into ~10 hooks+commands and into the gate-14 doc-todos exclusion (working scratch may carry TODO notes). Flattening is high-churn and would break that exclusion. Keeping it cleanly separates "durable, gated docs" from "transient pipeline scratch".

## AD-claude-project-search-adopt-ignore-crate-001
*Project-wide search walker adopts ripgrep's `ignore` crate (Route A) over extending the hand-rolled DFS · status: active*

**Context:** M21 #326 (project-wide content search) needed a workspace file walker. Two routes: A) adopt the `ignore` crate (ripgrep's gitignore-aware walker) vs B) extend the existing hand-rolled `marley_project::list_files_in` DFS, which has a HARDCODED skip set (.git/node_modules/target/.DS_Store) and no .gitignore parsing. The main objection to A was dependency weight — a new transitive tree (globset, aho-corasick, regex-automata, walkdir, same-file, crossbeam-deque, memchr, bstr) each needing a deny.toml license-allow + [bans] review. marley_project had ZERO real dependencies before this.

**Decision:** Adopt the `ignore` crate (Route A) for the search walker (`marley_project::walk_text_files`), as a masked lazy-iterator shim kept in marley_project (gpui-free). The pure engine (`search_lines`) stays a separate module. `list_files_in` (⌘P file-name search) is untouched.

**Rationale:** A REAL Cargo.lock check settled it: every one of `ignore`'s transitive dependencies was ALREADY in the lock (globset 0.4.18, aho-corasick 1.1.4, regex-automata 0.4.14, walkdir 2.5.0, same-file, crossbeam-deque, memchr, bstr, log, winapi-util) — so adopting `ignore` added EXACTLY ONE crate (itself, MIT OR Unlicense → passes the MIT allow), no allowlist churn, and `[bans] multiple-versions = allow` handles any version skew. That collapsed the entire dep-weight objection. In exchange Route A buys correct .gitignore/.ignore/hidden handling for free — the correctness a content search lives on (surface real source, not dist/build/generated junk). Route B would hand-roll gitignore semantics (globbing, negation, nested files, precedence) to MSI 100 — more code and more risk to reimplement what `ignore` already does. cargo-deny confirmed green after adoption. LESSON: quantify a new dependency's MARGINAL cost against the actual lock before rejecting it on 'dep weight' — a crate whose closure is already present is nearly free.

**Consequences:** marley_project gains its first real dependency (`ignore`). walk_text_files is a masked shim (mutants::skip) behavior-verified by a tempdir integration test (a .git marker makes `ignore` honor .gitignore under its default require-git). Follow-up: `list_files_in` (⌘P) could also switch to `ignore` for gitignore-aware file-name search, out of #326 scope. The walk is lazy so a superseded search abandons mid-enumeration.

## AD-claude-rebase-clamp-correct-for-delete-wrong-for-move-001
*A caret carry REUSES rebase_selections for a delete but must be hand-carried for a move — keyed on whether the op's own edit shifts the caret's span · status: active*

**Context:** Two M19 line/word ops needed to carry the multi-cursor SelectionSet through a raw edit: #300 move/duplicate-lines and #303 delete-word/line. Both apply edits back-to-front in one undo group (the shared apply_line_reorder shim). The question each faced: after the edit, where does each caret land? `rebase_selections` (indent.rs) rebases every endpoint through a LineEdit list, and its `rebase_through` clamp sends a position INSIDE a removed span to that span's (shifted) START. #300 BANNED rebase_selections (D-CARRY-NOT-REBASE) — a move's removed span IS the row the cursor sits on, so rebasing would slam every moving cursor to a line start; #300 computes the carried set by hand. #303 initially looked like it should copy that ban, but the spec claimed the opposite (D-REBASE-IS-CORRECT-HERE).

**Decision:** A DELETE reuses rebase_selections' clamp verbatim; a MOVE hand-carries the cursors. The discriminator is whether the op's own edit already SHIFTS the caret's span in the direction the caret should follow. A delete removes the span the caret sits in and does NOT re-insert it elsewhere → the caret belongs at the deletion start, which is exactly where the clamp puts it. A move removes the caret's span AND re-inserts it at a new location → the clamp (which knows only about the removal) collapses the caret to a line start instead of following the text to its new home, so the carry must be computed manually from the move geometry.

**Rationale:** The clamp is not universally right or wrong — its correctness depends on the op's semantics. Naming the discriminator ("does the op's edit relocate the caret's span, or just remove it?") lets future line/word ops (#304+, any block carry) decide correctly by inspection instead of cargo-culting whichever sibling they read first. #303 proved the pure fn can call rebase_selections directly and return `(edits, carried)` byte-identically to move_lines' shape, so the app shim is a one-word swap; the asymmetry lives entirely in whether the pure fn computes `carried` via rebase (delete) or by hand (move).

**Consequences:** Pairs with #300's carried-selection-by-offset-span AD. A future op that both deletes AND re-inserts at the caret (e.g. a transpose, a wrap-with-pair over a selection) is the ambiguous middle case: it must decide per-endpoint whether that endpoint's span was relocated (hand-carry) or merely removed (rebase). The rule is stated in terms of span-relocation, not op-name, precisely so those hybrids resolve correctly.

## AD-claude-receipt-scope-001
*Commit receipt fingerprints gate-defining files; no-.rs change is discipline-gated · status: active*

**Context:** enforce-commit-gate.sh triggers only when .rs is in the changeset, and gate_state_hash fingerprinted only crates/**/*.rs. So a change to the gate-defining files (gates.sh, hooks, deny.toml) — including TICKET-000 itself — commits with no FULL gate run, and a future .rs commit could ride a quietly-weakened gate.

**Decision:** gate_state_hash now fingerprints crates/**/*.rs PLUS the gate-defining files (scripts/*.sh, .claude/hooks/**, deny.toml, .gitleaks.toml, the Cargo manifests + lockfile), so a post-green weakening of the gate invalidates a later .rs commit's receipt. A no-.rs change remains enforced by pipeline discipline (static gates at validate), disclosed in CONSTITUTION §15. Full closure (commit hook demanding a receipt for no-.rs edits to gate-defining paths) is a recorded ratchet, blocked until a FULL-greenable workspace exists.

**Rationale:** Binding the bar's definition into the same receipt that binds the code it judges is the cheap, non-deadlocking half of closing the bypass; the full closure can't run on an all-stub workspace (no FULL-greenable gate to mint a receipt). Honest disclosure over a silent hole (§0/§15).

## AD-claude-registry-lifecycle-fork-pinned-vs-dropped-001
*A registry resident's lifecycle (pinned vs dropped) dictates its resolve-door CONTRACT — the two are not interchangeable wiring · status: active*

**Context:** Marley's ContentRegistry (#394) is refcounted: insert() takes the first view, acquire_view() adds one, release_view() returns the owned content exactly when the last view goes. Two kinds now live in it with OPPOSITE lifecycles. #400's Cockpit is PINNED: CockpitIndex holds the insert's first view forever as an anchor, so a user-reachable release never reaches teardown. M29 #403's Browser is DROPPED on last close. #403 was specced as "run the #400 slice again for a second kind", and the obvious move was to copy open_cockpit_tab's wiring: resolve the id, then acquire a view only when a tab was actually appended.

**Decision:** The resolve door's view-accounting contract follows the LIFECYCLE, not the sibling kind's code shape. Pinned residents (cockpit): the door returns a bare id and acquires nothing; each caller acquires its own view on top of the standing anchor. Dropped residents (browser, editor): the door hands back EXACTLY ONE acquired view in both branches — a hit acquires, a miss takes insert()'s first view — and a caller that does not consume it (an open-or-switch that switched) must release it. #403 therefore reuses #397's `resolve_open` / `ResolvedOpen` contract, not #400's, despite #400 being the far closer structural sibling.

**Rationale:** Copying the pinned wiring under a dropped lifecycle is a silent permanent leak, and the arithmetic is worth writing down because it reads as correct. Under a pin, insert()'s first view belongs to the ANCHOR, so a tab's view is legitimately a second, separate acquire. Under a drop there is no anchor: insert()'s first view IS the first tab's view. So the sequence becomes — door inserts (count 1, held by nobody yet) then open_or_switch MUST append (a live tab implies a live view implies the door would have hit, not inserted) then returns true then caller acquires: count 2 with 1 tab. That extra view is unreleasable: no close path knows about it, so the resident can never reach zero and the drop-on-last-close lifecycle is defeated by the very code meant to implement it. Nothing in the compiler, clippy, or a tab-count assertion catches it; only a VIEW-count assertion does. Which lifecycle to pick is a separate question answered by what the payload will own: cockpit surfaces own nothing reapable and have standing references (the top-bar strip, persisted right_section) that outlive tabs, so pinning is honest; a Browser will own a webview session (#405) and has no standing reference, so dropped is the contract that stays correct when the payload arrives — pinning now would bake in a leak-shaped contract needing reversal exactly when teardown starts to matter.

**Consequences:** A dropped resident has no index and no anchor, so its door is find-live-or-insert scanning kind() (no BrowserIndex analog), and close-then-reopen legitimately mints a FRESH ContentId — identity is the KIND, not the id (safe because ids are session-local and never serialize). Every future registry kind must now answer the lifecycle question BEFORE copying either wiring; the two shapes look nearly identical at the call site and differ only in who owns insert()'s first view. Tests for a dropped kind must assert view counts, not tab counts — the mutation operator `delete !` on the release guard is invisible to the latter. Open follow-up at #405: release_browser_views currently drops the returned content INLINE on the UI thread, which is fine for a unit payload but is the exact hazard release_grid_terminals exists to avoid; when the webview payload lands, that drop must move to an off-thread collector.

## AD-claude-retain-remote-seam-001
*Retain a non-functional, transport-agnostic remote-connection seam (future remote runners / device handoff) · status: active*

**Context:** Warp's "remote" ability is three separate systems: local_control/warpctrl (local app automation), remote_server (SSH remote-dev over a protobuf tunnel, behind a transport-agnostic RemoteTransport trait), and live session sharing (cloud-relayed via sessions.app.warp.dev + Firebase auth). Session sharing is cloud + account-bound → SKIP bucket for Marley. But Chad wants Marley to retain the ABILITY to connect remotely to something — inert for now — because remote runners / agents-on-web / device handoff are on the M5 roadmap and retrofitting a remote boundary late is expensive.

**Decision:** Marley keeps a transport-agnostic remote-connection SEAM wired but non-functional: a RemoteTransport-style trait (connect/send/recv/is_disconnected) with only a stub impl that returns RemoteUnavailable in M0–M4. It binds to marley_util::HostId + the local-vs-remote path sum type (TICKET-002), which already make the data model remote-aware. No network, no auth, no cloud until a future ticket. Captured as intake docs/planning/intake/remote-connection-seam.md; promoted to a ticket when remote runners/device handoff are scheduled (M3–M5). The real implementation is clean-room INVENT (our own relay/auth/transport, our own Block/command model) — never Warp's cloud or Firebase.

**Rationale:** A trait + a stub costs almost nothing now; introducing a remote boundary into a codebase that assumed everything is local is costly (Warp's own remote-vs-local path split exists precisely for this). Reusing only the transport-agnostic PATTERN keeps the seam clean-room and decoupled from Warp's SKIP-bucket cloud.

**Consequences:** A future ticket builds the concrete transports (SSH/docker-exec/relay) + a Marley auth/bearer seam behind the trait. The seam must not couple to Warp's cloud/Firebase or to session-sharing's relay. Until then, callers see RemoteUnavailable and everything runs local.

## AD-claude-stateful-crate-serial-reset-testutil-001
*Deterministic tests for process-global state under the threaded cargo-mutants runner · status: active*

**Context:** marley_core's feature-flag layers (baseline, user-preference, init-guard) are process-global mutable state. Marley's gate:3 + coverage use cargo nextest (process-per-test, isolated), but gate:5 mutation runs cargo-mutants with its default test tool = `cargo test` (multi-threaded, one process). Concurrent tests mutating shared global state race → a flaky baseline makes cargo-mutants exit 4 (fail-closed), and flaky per-mutant runs corrupt MSI.

**Decision:** Global-state tests are marked `serial_test::#[serial]` and call a `#[cfg(any(test, feature = "test-util"))] reset_for_test()` (clears baseline + user-preference + thread-local override + the init guard) at the start. The thread-local override layer needs no serial (already isolated). The test-support surface (override_enabled, OverrideGuard, reset_for_test) is gated `#[cfg(any(test, feature = "test-util"))]` — NOT just `feature = "test-util"` — because the gate runs with default features, so the `test` cfg is what makes that surface compile, run under nextest, and be mutated by cargo-mutants. A startup seam (mark_initialized) flips a guard so debug reads-before-init panic (testable via #[should_panic]).

**Rationale:** serial + reset gives determinism under the threaded mutation runner without making the production layers thread-local (which would break the real app's process-global semantics). The `any(test, feature)` gating keeps the test surface out of release builds while still being gate-exercised — a `feature="test-util"`-only gate would leave it uncompiled and its mutants unkilled. Generalizes to any later crate with process-global state (config caches, registries).

## AD-claude-syntax-node-range-api-foundation-001
*marley_syntax's node-range API (enclosing_ranges) — the shared tree-ancestry foundation, delivered v1 via a synchronous throwaway session · status: active*

**Context:** Before #329, marley_syntax exposed highlight SPANS only — the `tree_sitter::Tree` was a fully private field of `HighlightSession`, which lives on the off-thread syntax worker (spawned only for files > SYNTAX_SYNC_MAX_LINES; small files parse synchronously via the stateless `highlight_lines`, discarding the tree). Three M21+ features need the same primitive — a node-range/ancestry query over the tree: #329 expand/shrink-selection, #305 folding, #330 sticky context header. The Explore pass established a decisive constraint: the parsed Tree NEVER crosses the worker→app boundary (the app cache carries only per-line `SyntaxLines`), so NO live tree is reachable app-side for either file-size tier without either extending the worker protocol to ship a computed answer or building a tree on demand. Two delivery routes were on the table: (A) a worker round-trip (extend SyntaxReq/SyntaxResp + park/consume per the #313 precedent, one session) vs (B) a persistent second app-side session (rejected — the #313 D2 "defer, don't duplicate; two trees drift" lesson).

**Decision:** Add ONE pure, byte-in/byte-out node-range API to marley_syntax — `enclosing_ranges(&HighlightSession, byte_range) -> Vec<Range<usize>>` (the ancestry ladder: smallest named node → root, adjacent-span-deduped) — rather than exposing the `tree_sitter::Tree` itself. Keep the Tree encapsulated; the crate answers node-range QUESTIONS, callers never hold a Node/Tree. Deliver the app-side ladder computation v1 SYNCHRONOUSLY via a THROWAWAY `HighlightSession` (highlight_full over the current text, discarded after the parse) for ALL file sizes — computed once per gesture and cached. The worker-async round-trip for very large files is a named, reversible follow-up. #305 fold and #330 sticky-header consume `enclosing_ranges` unchanged.

**Rationale:** (1) Encapsulation: exposing the Tree would leak tree-sitter lifetimes/!Sync into the app and every consumer; a pure byte-range API keeps the FFI boundary inside marley_syntax and lets the pure function hit cov/MSI 100 in-crate. (2) The throwaway-session route is the minimal always-correct surface: no SyntaxReq/SyntaxResp extension, no park/consume, no async stale-drop no-op on the first keypress — and for the common ≤1000-line file it is the SAME cost class as the existing per-keystroke sync highlight (imperceptible on a deliberate gesture; the ladder caches after the first grow). (3) It stays consistent with the D2 rejection of a persistent second session — a throwaway is built and discarded per gesture, so no two-tree drift. (4) Reversible: the pure API is identical whether the tree comes from a throwaway session or a future worker round-trip, so the v1 route can be swapped without touching consumers.

**Consequences:** enclosing_ranges is now the single node-range primitive #305/#330 build on (do NOT add a second tree-access path). The v1 sync route does a one-time O(file) parse on the FIRST expand of a >1000-line file (cached after) — the worker-async round-trip is the deferred optimization if that first-press hitch is ever felt. The API needs no `is_named` filter (a tree-sitter parent is always a named rule node — anonymous tokens are leaves — a grammar-agnostic structural invariant), so it works for any future grammar #315 adds. Byte↔char conversion is the caller's responsibility (the API is byte-native, tree-sitter's space).

## AD-claude-tmux-grade-detached-sessions-001
*tmux-grade detached agent sessions are a core Marley pillar: sessions outlive the app, Marley has full insight (local + VPS), manager agents orchestrate them · status: active*

**Context:** chad, 2026-07-15 (verbatim intent): "tmux or tmux like support either via agent bridge or pure tmux. We have proved on another project that a manager AI can spin up other agents to work. And then i can watch that entire process take place inside of iTerm by reading the tmux session. The session lives and continues. The manager even has the ability to load it up for me inside of iTerm. ... we need to fully have insight into tmux sessions ran from Marley so we can [view]. This enables us to be able to reach out into VPS agent environments to work." Declared one of the TWO core concepts (the other: MCP, AD-claude-mcp-first-class-control-plane-001). docs/marley_architecture/detached-sessions.md (2026-07-15, CONCEPT) already architects it: today marley_agent OWNS agent child processes, so rebuilding Marley kills the agents; the fix is "the agent session lives outside the app — Marley is a viewport onto it, not its owner." The protocol is being trialed OUTSIDE Marley now (tmux + iTerm2 -CC + the UCSOS manager) — that trial IS chad's "proved on another project."

**Decision:** Ratify the detached-sessions direction as CORE, with chad's requirements as the bar: (1) sessions survive Marley (detach/attach, quit/rebuild loses nothing); (2) real-tmux interop both directions — iTerm can attach to what the manager created, Marley can see sessions created elsewhere, and "surface this session to the human" is a first-class verb; (3) FULL insight into Marley-spawned sessions for both human and manager — enumerate, read scrollback (capture-pane -p -S -, the verified dead-pane rule), watch live, attach as a native pane; (4) VPS reach over the existing marley_remote ssh-argv seam as-is (ssh dev -t 'tmux attach ...') — no new transport, no relay; (5) manager orchestration rules from the doc hold (manager-created-only visibility, one window per ticket, remain-on-exit). Transport stays dual: tmux where a human may sit in it, ignibyte-bridge where nobody does — one seam, two backends. The adoption ladder ends at a tmux CONTROL-MODE (-CC) client rendering tmux windows as native Marley panes (the iTerm2 model) — first-class means climbing to step 3, not parking at attach-in-a-pane. NOT scheduled yet — IDE milestones first; intake pillar at docs/planning/intake/tmux-grade-detached-agent-sessions.md.

**Rationale:** The ownership inversion is the whole point: Marley restarts dozens of times a day under development, so app-owned agents die constantly — "the act of building Marley kills the agents Marley exists to run." tmux is the battle-tested, zero-license-risk (argv-only) persistence layer a human can always fall back into from any terminal; the bridge covers containerized/SaaS contexts where attach is meaningless. The manager-fleet pattern is already proven live in the out-of-Marley trial, so Marley's work is a viewport + verbs, not an invention. Insight-before-control sequencing (enumerate/read first, control-mode render later) de-risks the ladder and delivers the VPS observability chad wants earliest.

**Consequences:** AgentRun becomes "a handle to a session that may outlive me" (ownership flag or split type — open question owned by the doc). The renderer⊥transport split must stay clean: a tmux window is just another byte source for terminal_blocks. Composes with MCP: session verbs serve over MCP tools. Natural first slice when promoted: a sessions rail listing tmux sessions (local + configured ssh hosts) with scrollback peek. Open questions stay in docs/marley_architecture/detached-sessions.md (reap ownership, replay-vs-from-now, host crate for the control-mode client).

## AD-claude-two-boundary-maps-for-phantom-text-001
*Phantom text forks the buffer→display map into TWO boundary maps: code spans hug the code, caret ranges track the caret · status: active*

**Context:** M21 #331 introduced the editor's first PHANTOM text — LSP inlay hints, which occupy display columns but belong to no buffer char. Before it, `code_view::LineLayout` was a strict 1:1 buffer→display map: `col_starts[i]` = the display column of the i-th BUFFER char, and everything rode it — the caret pixel (`col_of_offset(caret) * cell_w`), the click inverse (`offset_of_col_f`), drag, selection rects (`row_selection_cols`), #310 diagnostic underlines, and the #268 tree-sitter feed (`raw_span_to_display_bytes`).

Design named ONE invariant and believed it sufficient: `display`'s cell accumulation and the values in `col_starts` must stay ONE column domain (`cols_to_bytes` re-derives a column by walking `display`; `col_of_offset` reads one from `col_starts`; they agree only because a single pass builds both). That invariant is real, was implemented correctly, and was verified exhaustively at inspect — but it was NECESSARY, NOT SUFFICIENT.

Two independent critics found the gap from opposite ends (one from the column math, one from reachability): one column domain still admits TWO incompatible BOUNDARY semantics. `col_of_offset(i)` is the CARET map — by the locked D-CONTRACT it returns the column AFTER a phantom anchored at `i`, because the caret sits on the code side of a hint. Every END boundary in the codebase was built from it. So a code span whose one-past-end char is a hint anchor EXTENDED ACROSS the phantom. Reachable on the PRIMARY path, not theoretically: rust-analyzer's chaining hints are on by default and anchor at END OF LINE, where a literal DOES have a syntax span ending exactly at the anchor — `let n = "hello"` mapped its Str span 8..15 to display 8..20 and painted the hint string-green. The same root cause made a warning on `x` in `let x = 1;` underline `x: i32`.

The two critics DISAGREED on the fix, and resolving that disagreement is what produced this decision: one proposed fixing the accessor everywhere including `row_selection_cols`; the other argued selection was already correct. The latter was right, and the reason generalizes.

**Decision:** When virtual/phantom text can occupy display columns, the buffer→display mapping MUST expose TWO end-side column accessors, and every consumer MUST be classified into exactly one:

1. **A span that describes CODE** (a syntax token, a diagnostic underline, a search hit, a fold marker) ends at the CODE-side map — `LineLayout::col_of_span_end(end_char)`, backed by `col_ends` built in the same pass — which stops at the right edge of char `end_char - 1`, BEFORE any phantom anchored at `end_char`. Used by `raw_span_to_display_bytes` and the #310 squiggle.

2. **A span that describes a CARET RANGE** (a selection band) ends at the CARET map — `col_of_offset(end)` — which sits AFTER a phantom anchored at that offset. Used by `row_selection_cols`, unchanged.

The worked example both accessors must keep producing, on `let x = 1;` with `: i32` anchored at char 5: the diagnostic bar for `x` ends at column 5 (it hugs `x`, not `x: i32`); the selection for `x` ends at column 10 (where the caret for offset 5 renders). Both are correct simultaneously. Their doc comments name the rule; the pinning test `t331_code_spans_hug_code_while_caret_ranges_track_the_caret` asserts BOTH on the same char range in one test.

A corollary rule: the anchor semantics are ASYMMETRIC. "The caret sits on the code side" is not uniform — mid-line the code is to the phantom's RIGHT (emit the phantom, THEN record the anchor's column), while at end-of-line there is no char to its right and the code is to its LEFT (record the column, THEN emit). Both are the same rule; treating them uniformly parks the End caret out past a chaining hint.

Defence in depth, not a substitute: hint spans are fed to the render's span splitter FIRST, ahead of code spans, so first-wins resolution makes the phantom's own cells win unconditionally even against a producer that legitimately encloses one.

**Rationale:** Unifying the two accessors is the obvious refactor and it is WRONG in both directions, which is precisely why this needs to be a recorded decision rather than a comment.

- Point the selection band at the code-side map and it stops at column 5 while the caret it follows renders at column 10 — a visible 5-column gap between a highlight and its own caret. A selection band is not a description of text; it is the swept region between two caret positions, so it must end where a caret ends. Consistency with the caret IS its correctness condition.
- Point a syntax token at the caret map (the pre-fix state) and the token swallows the phantom, painting the hint as code. A token IS a description of text, so it must end where the text ends.

They are different questions that merely coincided while the map was 1:1 — which is exactly why one accessor served both for 20+ tickets and why the bug was invisible until phantoms arrived. Nothing in the type system distinguishes them; both are `usize` columns off the same struct, so the only defence is the named rule plus the test.

Ordering alone was rejected as the fix: feeding hint spans first does correct the syntax COLOR, but the squiggle has no ordering escape (it is a geometry computation, not a span-resolution one), and `raw_span_to_display_bytes`'s own doc promises it maps code spans to the code's display bytes. Fixing the accessor fixes the root; ordering is kept as belt-and-braces because it makes the Hint tag independent of any other producer's capture table — which is what silently protected it before (identifiers map to `Plain` and are dropped, an accident #316's palette would end).

The decision is cheap to keep: `col_ends` is one Vec built in the same pass, and it is a provable no-op when hints are off (`col_ends[i] == col_starts[i+1]` with no phantoms), so the empty-slice byte-identity property still holds.

**Consequences:** - BINDING ON FUTURE WORK. #316 (syntax theme system) adds capture kinds — the moment it emits an `Identifier`/`Type` span, every `let` binding's type hint sits at a span's end boundary, so `col_of_span_end` becomes load-bearing on the most common hint in the language. #333 (the hand-lexer fallback lexes phantom text) should be fixed by giving the fallback the primary path's shape — lex the RAW line and map through `raw_span_to_display_bytes`, which is now phantom-safe by construction. #305 (folding) will introduce a SECOND kind of phantom (a collapsed-region marker) and must classify its consumers the same way.
- A new end-boundary consumer is a decision point, not a lookup. Adding one requires answering "does this describe code, or a caret range?" — and the answer is not inferable from the types.
- The rule is enforced by exactly one test and two doc comments. There is no compile-time guard; a newtype pair (`CodeCol`/`CaretCol`) would give one, and was NOT done — the ergonomic cost across ~15 call sites outweighed it for now. If a third phantom source lands, revisit.
- `col_starts.last()` is no longer always the total display width: with an end-of-line phantom it is the column just past the last CHAR, which is what its doc already said and what every reader wants. Nothing consumed it as a width.
- Not generalized to the terminal. The #200 ghost text is EOL-only and terminal-side, on a different render path; this decision is scoped to the editor's `code_view` seam.

## AD-claude-whole-doc-byte-span-to-row-local-display-map-001
*Rendering a whole-document byte span into a per-row editor line: localize by byte, then map with the code-side display mapper · status: active*

**Context:** M22 #340 bracket-match produces its highlight as two whole-DOCUMENT byte ranges (from a tree-sitter parse of the full source). The editor renders per ROW via `uniform_list`; each row's `raw_span_to_display_bytes(line_text, layout, span)` expects a LINE-LOCAL byte span (relative to the row's own text), and the syntax-cache spans it consumes are already line-local. But `line_start(row)` returns a CharOffset, while the bracket span is in absolute BYTES — the batch's recurring byte/char/display confusion class (on ASCII all three coincide, hiding the bug). Two candidate mappers existed: `raw_span_to_display_bytes` (the #331 code-side map, using col_of_span_end at the end boundary) and the caret-tracking cols path the find-bands use (row_selection_cols → cols_to_bytes). The #331 AD (two-boundary-maps-for-phantom-text) established that a span describing CODE must hug the code while a span describing a CARET RANGE tracks the caret.

**Decision:** To render a whole-document byte span into a per-row line: (1) compute the row's ABSOLUTE byte start as `char_to_byte(line_start(row))` — NOT the char offset; (2) keep the span only if it falls within `[row_byte_start, row_byte_start + line_text.len()]` (line_text is \n-exclusive, so a single-char delimiter always lands within exactly one row); (3) localize it as `(span.start - row_byte_start)..(span.end - row_byte_start)`, the `>= row_byte_start` filter guarding the subtraction from underflow; (4) map through `raw_span_to_display_bytes` (the CODE-side mapper), NOT the caret-tracking cols path, because the span describes real buffer code (a delimiter) and must hug the code so an inlay phantom anchored at its boundary cannot smear it.

**Rationale:** The absolute→line-local subtraction must be in BYTES (both operands byte offsets) — mixing in the char-domain line_start would silently corrupt on any multibyte line before the row. `raw_span_to_display_bytes` is width-aware (it routes bytes→chars→columns→display-bytes through the layout's width table) and phantom-safe on the end boundary (col_of_span_end), which the cols path is not — the #331 lesson inverted: a delimiter is code like a syntax span, so the code-side map is domain-correct, and using the caret path would place the tint on the wrong side of an adjacent inlay hint. Proven by a multibyte-and-tab test where raw≠display (a `)` after a 2-wide emoji, or after a tab): on ASCII the mapping is identity and proves nothing.

**Consequences:** Any future consumer that renders a whole-document byte span into the per-row editor (a second mark producer, semantic-token bands, a selection-derived highlight, find-reference underlines) must follow the same four steps — localize by BYTE via char_to_byte(line_start(row)), filter on-row, then use raw_span_to_display_bytes. The sibling AD is AD-claude-two-boundary-maps-for-phantom-text-001 (which mapper for which span kind). The per-row char_to_byte(line_start) is O(log n) on ropey and runs only for rendered rows, so it is cheap; if a future consumer needs it hot, cache the row byte-start alongside the row text in the per-row read.

## AD-claude-workspace-spawn-seam-and-caller-owned-teardown-001
*Multi-pane workspace: session-generic registry behind a spawn-per-call seam; close RETURNS the state (caller-owned teardown) · status: active*

**Context:** M1.C TICKET-023 replaced the M1.B status-line placeholder with real per-pane sessions — forcing the mock-testability seam, the non-blocking close semantics, and the rect-math mutation policy in one design. gpui hosts the render; alacritty_terminal owns the PTY underneath TerminalSession.

**Decision:** Marley's multi-pane model (marley_app::workspace, M1.C TICKET-023) keeps ALL pane/focus/layout decisions in a pure module: Workspace<S> is GENERIC over the session handle with the spawner passed per call (impl FnOnce() -> Result<S, E>), so registry/focus/routing logic is mock-tested at cov 100/MSI 100 with no PTY (the PtyChannel mock precedent; one real-spawner closure in the shim). Two load-bearing rules: (1) spawn runs FIRST — a spawn failure leaves the workspace byte-unchanged (tree/registry/focus/next_id); (2) close(pane) RETURNS the removed PaneState instead of dropping it — the CALLER owns teardown context, because dropping a real session blocks in the child reap (603 ms measured; unbounded for HUP-immune children) and the app must reap on a detached thread (PtyChannel: Send makes TerminalSession Send). Focus rules: focus-follows-split; focus-after-close = depth-first predecessor in the PRE-close order (successor when first); rect layout = pure pane_rects with NO remainder arm (the binary equal-ratio tree tiles area-exactly in f32; a remainder arm is an unkillable equivalent mutant until M2 drag-resize).

**Rationale:** Generic + spawn-per-call avoids stored-closure lifetime knots and keeps test mocks !Send-friendly; return-the-state teardown splits purity (workspace) from threading (shim) — the inspect critics MEASURED the 603 ms main-thread freeze the inline drop caused; the design-phase equivalent-mutant analysis killed the remainder arm before it shipped. The seams are exactly what M2 consumers (docks insetting bounds, drag-resize ratios, session restore) need.

## AD-marley-app-to-forge-raw-http-localhost-readonly-redacted-001
*App→forge transport: raw HTTP/1.1 over TcpStream, localhost-only, read-only, bearer-redacted · status: active*

**Context:** M2.B needs the running Marley app (not just the dev MCP tools) to read the current sprint + tickets from the local forge sidecar to show the work in a cockpit pane (#64). This requires the app to authenticate to forge (bearer in the gitignored .mcp.json), open a socket, and parse the response — the first secret-read + network I/O in the app. Forge is an MCP Streamable-HTTP JSON-RPC server on http://127.0.0.1:8080 (plain HTTP, Authorization bearer), probed live. Options: (A) raw HTTP/1.1 over std TcpStream, no dep; (B) add an HTTP crate; (C) defer. A secret is sent over the wire — security-sensitive.

**Decision:** Chose (A): a raw HTTP/1.1 POST on std::net::TcpStream in a new marley_forge_client crate — no new HTTP-client dependency. LOCALHOST-ONLY (forge_endpoint_from refuses a non-loopback authority), READ-ONLY (only sprint-current + ticket-list), bearer NEVER logged (manual redacting Debug; unused bearer() getter removed). The request build + SSE/JSON-RPC/JSON parse are pure (cov/MSI 100); the socket is a masked adapter verified by examples/check_forge.rs.

**Rationale:** Clean-room minimal-dep ethos: pure/testable request+parse, only the socket masked (like pty_os); forge is plain-HTTP localhost (no TLS); serde/serde_json already in the lock. Enforcing the localhost invariant in code (not just docs) closes an exfiltration path a security critic found (a tampered .mcp.json url could send the bearer to a remote host) — see PR-claude-enforce-security-invariants-in-code-not-just-docs-001. tools/call needs no initialize handshake (probed). Reversible: swapping in an HTTP crate later is contained behind the same pure/adapter seam.
## AD-claude-scrap-forge-local-tickets-001
*The dev pipeline runs on local files — tickets, queue, and knowledge ledger · status: active*

**Context:** The forge MCP sidecar carried the pipeline's tickets, sprints, and
knowledge recall/capture (old §19). Operationally it was fragile on this machine
(down after every reboot: TCC-blocked LaunchAgents, stale postgres lockfiles, a
start script building the tool-less binary) and its state drifted stale whenever it
was down — while the local shelves had already proven file-based batching at
production scale (15 tickets across two shelf runs).

**Decision:** Owner pivot 2026-08-09 (TICKET-409): tickets are local docs +
`BACKLOG.md` (top Queue row = `/work` next; Deliberate rows only picked
explicitly); knowledge is the greppable `docs/planning/knowledge/` ledger, seeded
by a full count-asserted export (761 blocks) and appended at phase close; the
CONSTITUTION §19 rewrite records the loosening reason in-section. The services
stop; the DB stays intact for the other projects it tracks.

**Rationale:** Recall + capture were the real value and both survive as grep +
append with zero service dependency; what was lost (semantic search,
cross-project promotion, MCP sprint bookkeeping) had not earned its operational
cost — the FIFO `ticket-next` never actually chose work.

**Consequences:** The product surface (Forge pane, forge_client, fleet-brain
wiring) becomes dead weight → TICKET-410/411 rip it, dependency-ordered per the
2026-08-09 seam map. Capture is discipline, not hook-enforced (§15 reworded).

## AD-claude-browser-no-origin-source-until-url-feature-001
*Post-#410 the embedded browser has NO production origin source; the mount/Retry `None` literals are the one seam a future URL feature fills · status: active*

**Context:** The 2026-08-09 scrap-forge pivot (TICKET-410) ripped `forge_web_base` — the browser's only origin source — while D1 required the generic infrastructure (wry child, 27-state hide-shim, #406 lifecycle, bare-`B` codec) to survive fully wired. The named fork: no source at all vs adopting the dead `orchestration.web_url` settings field as a replacement source.

**Decision:** NO origin source. `orchestration.web_url` is deleted (dead since #371 — zero production readers; old settings files parse on, no `deny_unknown_fields`). The mount gate calls `mount_plan(None)` and the Retry directive `retry_plan(None, mounted)` — both commented as THE seam; the ReAttach arm stays total (teardown, not panic) though unreachable with a `None` desired origin per `retry_plan`'s tested arms. The pinned-origin verdict (`same_web_origin` + `web_origin_of`, with the #404 parsed-host loopback re-check) moved into covered `browser.rs` so its security shape stays gate-enforced for the day an origin returns.

**Rationale:** A rip chore must not grow product surface: wiring `web_url` up would invent a settings-driven browser feature (validation posture, caption UX, security wall scope) mid-chore, un-designed. The placeholder-as-standing-state is honest ("The embedded browser awaits a page source."), and every machine seam stays compiled + mutation-enforced via the constant-input pattern ([[L-claude-rip-keeps-pure-seams-called-constant-input-001]]).

**Consequences:** A future URL feature re-adds an origin register + the ReAttach write + its own source validation, and inherits: the bare-`B` persistence decision (still framing-safe — nothing to persist), the loopback-or-widen wall question (the moved `web_origin_of` refuses non-loopback today — widening it is a deliberate security decision, not a default), and the user-typed-URL percent-encoding deferral recorded in `embedded-browser-model.md` Q4. Until then the attach/LoadSame/ReAttach paths are production-dead-but-tested; `webview_shim` stays the §0 exclude. TICKET-411 re-homes the probe edge and deletes `marley_forge_client` (whose predicate copy is the temporary duplicate).

## AD-claude-fleet-rail-quiet-no-transport-until-brain-001
*Post-#411 the fleet rail has NO transport: quiet bare-"Fleet" for any setup, machines terminate in the shared NO_TRANSPORT_REASON through the standing result channels · status: active*

**Context:** TICKET-411 deleted `marley_forge_client` (the fleet's only transport). The rail itself is forge-agnostic by charter and survives (snapshot model, demo feed, dispatch/answer machines). Named forks: ConnectionState's post-rip home, and the machines' no-transport terminal.

**Decision:** (1) `classify_fleet_setup(target, config_dir)` — the endpoint-construction and forge-client gates died with their inputs; a configured brain root classifies `Configured`, and `(Configured, no-wire)` renders the quiet bare "Fleet" the #376 contract already defined — no lying "misconfigured" reason for a user whose only fault is that no transport exists yet. (2) `ConnectionState` COLLAPSED, not re-homed: a re-homed enum would have zero non-test constructors → `variant never constructed` at `-D warnings` — the wire-suffix display retired with its producer. (3) The confirms send the typed failure through the STANDING mpsc result channels (`fleet_rail::NO_TRANSPORT_REASON`, one shared const), so the pump's fold fns stay the single phase-writers and stay CALLED — [[L-claude-rip-keeps-pure-seams-called-constant-input-001]] applied to a channel; the forge-SSE-specific echo layer (`dispatch_observe_record`, the `Delivery` buffering writer) died with its producer.

**Rationale:** The rip must not grow surface (no invented "unwired" state) and must not preserve theater (an enum only tests construct). What survives is exactly what a future non-forge brain transport re-consumes: the target gate, the cursor-dir seam, the demo gate, the fold/nonce/scrub laws, the retry-prefill contract — all pure, all tested, all called.

**Consequences:** A future brain transport re-adds: its endpoint validation arms in `classify_fleet_setup` (the D5 first-failure order is kept encoded), its own status display type for the header suffixes, real async producers for the result channels (replacing the confirm-site sends), and an echo stream if it has delivery receipts (the `pending` buffer + receipt-time max-join survive for it). Dead-code analysis is the design reviewer for rips: every kept seam must name a live caller or collapse ([[AD-claude-browser-no-origin-source-until-url-feature-001]] is the sibling decision from slice 1).

## AD-claude-editor-file-identity-canonical-at-storage-001
*category: architecture · topic: editor / path identity · status: active*

Decision (M20 #319): an open editor file has ONE stored identity — the CANONICAL spelling, produced at every open seam by `marley_project::canonical_under_root` (resolve_under_root, then canonicalize when the target exists; the resolved join when it does not — deliberately the SAME fallback `same_file` compares with, so storage and compare can never disagree). Consequences: (1) raw `==` between two STORED paths is sound; `same_file` remains the compare wherever one side may not exist, with an identical-spelling fast path. (2) Every consumer family aligned in the same change: prefix containment (host-spawn gate) and ownership ranking (encoding lookups) compare canonical-vs-canonical — `LspHost::root()` exposes the host's canonical root, and `LspHost::absolute` DELEGATES to `canonical_under_root` so the workspace has exactly one normalization algorithm; strip/derivation goes through `rel_under_root` (verbatim, else canonical root, else unchanged); WALKERS feeding stored-keyed stores walk the canonical root (`canonical_root` — the ⌘⇧F override map). (3) Workspace identity is explicitly OUT of scope: D-OPEN-DEDUPE-SCOPE stands (aliased ROOT spellings are two workspaces; `find_open`'s root half stays raw). (4) `canonical_root` fails OPEN (verbatim fallback) — fine for identity/display; write-containment guards keep their fail-closed `canonicalize().map(…).unwrap_or(false)` shape. Reference behavior (§20, behavior-level): the editor references canonicalize at ONE boundary (a worktree-entry canonical path; a canonicalized-at-construction path newtype); LSP 3.17 leaves identity normalization to the client, and rust-analyzer spells uris canonical — which is exactly why the client-side storage boundary must too.

## AD-claude-editor-fold-state-in-session-instance-scoped-001
*status: shipped (#353) · scope: editor / fold lifecycle*

Editor fold state (`editor_folds: HashMap<PathBuf, Vec<Anchor>>`) is IN-SESSION
and file-scoped: an entry lives while ANY editor instance stores its canonical
path and dies at the last same-path instance drop — scrubbed inside
`RootView::release_editor_views` (the one release choke; the editor sibling of
the #398 terminal-side map scrub), census-guarded by
`content::any_open_editor_with_path` (exact `==` on stored spellings — the map's
own key semantics; sound because #319 canonicalizes every birth). Alias-root twin
WORKSPACES (D-OPEN-DEDUPE-SCOPE: separate instances, one shared entry) keep the
survivor's folds; twin VIEWS never scrub (non-last releases report nothing).
Reopen presents UNFOLDED — persistence across close/reopen was declined at #305
and re-affirmed here (Zed-matching view-scoped behavior; VS Code-style viewState
restore would be a new feature, not this map). Resolve-HIT mounts mint rows from
the instance's STORED path so row key == scrub key by construction. A future
file-RENAME feature owns migrating/clearing the fold key (no rename surface
exists today; LSP resource ops rejected per #322 D2).

## AD-claude-one-raw-to-display-remap-for-all-span-producers-001
*category: editor render / column domains · status: active (#333)*

**Context:** #268 gave the render `raw_span_to_display_bytes` (raw line-local
bytes → display bytes, code-side end via `col_of_span_end`). By #333 three
inline copies of the per-span-list remap existed (both primary tree-sitter
arms + the new fallback), two of them inside the coverage-excluded app.rs shim,
and the #331/#340 ADs stated the mapping rule only as prose.

**Decision:** `code_view::spans_to_display_bytes(line, layout, spans)` is THE
remap for a `(raw byte Range, TokenKind)` list onto the display. Every span
producer that renders per-row — the tree-sitter memo arms, the hand-lexer
fallback (`code_syntax::highlight_display_ranges` = raw-lex + this remap), and
any FUTURE producer (semantic tokens, a second mark source) — calls it rather
than inlining the map. Kind-preserving; ascending+disjoint in/out (monotone
per-span map); phantom-safe by construction (spans hug code, never a phantom).

**Rationale:** a fallback that shares the primary's shape by TEXT drifts (the
#333 bug was exactly the drift); a shared fn makes the #331 two-boundary rule
load-bearing in one place, and it lives in the covered pure layer instead of
the excluded shim (gate:4/5 see it). Composition stays testable: the lexer fold
(`highlight_ranges`) and the remap are independently pinned, and the one-line
composition is pinned by the t333 matrix.

**Consequences:** a new per-row span producer's review question is "does it
call spans_to_display_bytes?" — an inline `.map(raw_span_to_display_bytes)` in
app.rs is now a red flag, not idiom. Siblings:
AD-claude-two-boundary-maps-for-phantom-text-001 (which end map), the #340
whole-document-span AD (byte-localize before this remap).

## AD-claude-git-gutter-state-instance-scoped-001
*status: accepted · scope: marley_app editor*

Git gutter state (`git_marks` + `git_marks_key`) is INSTANCE-LIFETIME state: it
dies with the last same-path editor instance (the #353 census-guarded release
choke, #412) and is recomputed from the working tree on reopen. Nothing
persists gutter marks across close — the same stance as fold state (#305/#353).
A future "restore the gutter instantly on reopen" feature must be a deliberate
reversal of this AD, not an accidental cache survival.

## AD-claude-lsp-docsync-doctext-live-object-001
*status: shipped (#320, M20) · category: LSP doc-sync seam*

`LspHost::reconcile` consumes LIVE document objects — `&[(&Path, &dyn DocText)]` with `DocText { version() -> BufferVersion, text() -> String }` implemented by `marley_editor::Buffer` app-side — and materializes version + text INSIDE the send branches (didOpen/didChange). It is the ONE Ready-gated read on the sync path; `needs_text` is deleted. Rationale: same-OBJECT atomicity (version and text cannot disagree, and no pre-collected projection exists for a differently-gated read to betray — the BF-lsp-didopen-carries-empty-text-ready-race-001 class becomes unrepresentable rather than re-ordered away); a `&dyn` trait rather than a bare `&Buffer` keeps the shim non-generic and makes the idle hot path PROVABLE with a counting double. Companion decisions: the pump derives both its ensure and reconcile sets from `ContentRegistry::editors_under(root)` (one filter, so #321's ensure-set == sync-set invariant is structural — PR-claude-shared-set-derivations-must-be-one-function-001), and the host's `#[cfg(test)]` sent-bodies capture (recorded outside the handle check) is the payload-level proof surface the fake_ls app lane asserts against. Ordering of drain/collect/ensure/reconcile is freshness-tuning only, never correctness.

## AD-claude-lsp-request-outcome-typed-terminal-enum-001
*status: shipped (#332) · area: marley_lsp::rpc / lsp_host / the 12 editor consumers*

Every LSP request terminates with EXACTLY ONE typed outcome:
`RequestOutcome::Answered(Result<Value, RpcError>) | Abandoned(AbandonReason::{Timeout,
Disconnected})` — the queue element REPLACES the old bare `Result`, never extends its
Err side, so every consumer's arm decision is compile-forced (an `Err(_)` catch-all
absorbing an abandonment cannot exist; see
L-claude-new-terminal-state-new-enum-not-err-variant-001). The pure mechanism
(`abandon_ids`/`abandon_all`, generic over the purpose type because `RequestPurpose`
carries app-side keys) lives in `marley_lsp::rpc` at 100% MSI; the shim only wires it.
Delivery rules: (1) expire → one `Abandoned(Timeout)` per dropped purpose, cancel still
sent; (2) teardown DELIVERS, never destroys — `on_connection_lost` preserves the queue
(an outcome enters it only by moving its purpose out of the map, so exactly-once holds
structurally; a dying server's final answers arrive in the same mpsc batch as the
disconnect) and sweeps only still-pending purposes as `Abandoned(Disconnected)` (see
PR-claude-teardown-must-deliver-not-destroy-the-outcome-queue-001); (3) consumer policy:
silent latch-clear by default; an honest, reason-agnostic "Language server didn't
respond" ONLY where the user committed an action whose silence would read as success
(rename, codeAction/resolve); (4) the #331 `has_pending_*` host queries remain as the
belt for poll-driven consumers and draw gates. Behavioral precedent: Zed's
`ConnectionResult::{Result, ConnectionReset, Timeout}` (behavior map only) and the LSP
protocol's own cancelled-is-not-a-semantic-error stance (-32800/-32801). Known deferral:
only the active root's queue drains per tick (TICKET-413).

## AD-claude-354-saves-are-instance-addressed-active-is-a-consent-mode-001
*status: adopted · topic: editor/save*

Marley's save machine is INSTANCE-addressed: the write tail (`write_and_mark(id)` — fs write → owning-root didSave via `EditorInstance::root()` → mark-saved/disk-snapshot → the #284 racing cross-check) takes a `ContentId`, and "active" is not an identity anywhere in it. What WAS "the active editor" in the pre-#354 save path is now a derived CONSENT MODE at the completion funnel (`save_editor_by_id`): active origin ⇒ interactive semantics (the #275/#284 arm flow, flashes, exact legacy behavior), background origin ⇒ strict-consent semantics (a `Changed` conflict holds the write AND the arm — arming is an active-consent gesture; `CleanReload` defers to the activation choke per #275's D4; `Deleted` recreates). Async completions (the format-on-save latch, and any future deferred write) bind the ORIGIN as a `ContentId` at request time — monotonic, never reused, so a closed origin drops naturally and a reopened file is a different generation. Rationale: the #314 C1-HIGH lost-save and its surface-granular residue both came from re-deriving identity from focus at completion time; the id binding makes the wrong-target class unrepresentable, and the consent split keeps "never clobber content the user hasn't seen" intact when the user isn't looking.

## AD-claude-366-auto-close-lifetime-context-is-two-probes-quote-gated-001
*category: editor/syntax · status: shipped (M22 #366) · supersedes: the #362 "where/dyn deferred" boundary*

The auto-close `Context::LifetimeBound` pipeline is: StringOrComment (`node_kind_at`, always wins) →
`typed_quote && (in_type_parameters || speculative_lifetime_at)` → `Code`. TWO probes, deliberately not one
generalized one: the #362 ancestry probe is exact at decl `<...>` sites and short-circuits; the #366
post-insert splice probe (splice `'a`, re-parse, covering kind == `lifetime`) is the only possible signal at
the anchor-less positions (`where`/`dyn`/type-usage) because a PRE-insert tree has no node for a mid-typing
token. Both are full-file parses, so BOTH are gated to the single keystroke whose outcome they can change —
`text == "'"` — a gate that is provably semantics-free because `LifetimeBound` is inert for every other char
(`blocked_quote` self-gates on the typed quote; the `!= StringOrComment` pairing check treats `LifetimeBound`
and `Code` identically — pinned by `t362_lifetime_bound_suppresses_only_the_quote`). Net cost: ZERO extra
parses for non-quote openers (an improvement over the #362 shape), ≤2 for a typed `'` (rare). #349's cached
tree can absorb the pre-insert parses but never the splice (modified text by construction). Loop labels
deliberately keep pairing (kind gate is exactly `"lifetime"`); the safe-direction misses (bare type positions,
braceless char const-args) withhold the pair and never mangle the buffer.

## AD-claude-413-lsp-outcome-delivery-is-owner-routed-never-focus-gated-001
*category: editor/LSP · status: shipped (pipeline 413, M20)*

LSP outcome delivery in Marley is a property of the OWNING server/root, never of window focus: the pump's
ONE consume pass drains every host per tick (root-sorted, per-host queue/arrival order — the abandonment
sweeps ascend by id) and dispatches each outcome with its owning root. Per-feature visibility is decided
ARM-LOCALLY on a fixed partition. Focused-editor features (hover, definition, references, completion,
prepare-rename, code-action menu, signature help, inlay) drop outcomes whose owning root is not the active
project's root — BEFORE any latch/UI interaction, latch untouched — because uri/path equality proves FILE
identity, never INSTANCE identity (nested-root twins: byte-identical uris, latches surviving project
switches, colliding independent version counters). Committed / global / origin-targeted features (rename,
codeAction/resolve, workspace/symbol, formatting) complete in the background and REQUIRE the owning root:
encoding + `lsp_version_for` come from its host, and `version_conflict(_, None)` fails OPEN, so wrong-root
routing is an apply-with-wrong-encoding bug, not a safe skip. WorkspaceEdits land on the OWNING root's
INSTANCE, never a lexicographically-first twin. A future 13th consumer arm must declare its side of this
partition explicitly — the match is exhaustive and the drain-site law comment demands the posture.

## AD-claude-318-overlay-chrome-two-recipes-001
*category: app shell / overlays · status: shipped (pipeline 318, M20)*

Marley's floating overlay cards are ONE modal chrome + TWO positioning recipes. The chrome
(`overlay_card_chrome`) is the 8-call core (occlude/flex/flex_col/bg(surface)/rounded(corner_radius)/
overflow_hidden/border_1/border_color) — `.occlude()` is a MODAL choice; a non-modal member over
scrollable content takes a future `block_mouse_except_scroll` variant, never this fn. Recipe A
(anchored: hover, def-picker) = fixed `W`/`MAX_H` + `menu_origin` clamp + mono type. Recipe B
(centered-quarter: palette, file finder, history, agent launcher, fleet) = `quarter_overlay_card` over
the pure `context_menu::overlay_quarter_geometry` (`w/4, h/6, w/2`) — containment BY CONSTRUCTION
(unit-proven; nothing to clamp: height is content-driven with deliberate bottom overflow, the settled
contract), 16px default type. Selection deliberately splits: clamp for the unwindowed `.take(20)`
overlays, wrap only where rows window around the selection (def-picker) — wrapping an unwindowed list
would select an unrendered row. 10 chrome sites remain unconverted (the named follow-up); a new
consumer declares its recipe + modality instead of pasting the chain.

## AD-claude-zone-a-inversion-mechanics-001
*status: adopted · topic: react-first parity · ticket: #418 (M31)*

MARLEY-PARITY.md's Zone A default is "Marley is right; the POC is the bug." When a
REDESIGN deliberately replaces a Zone A surface, the contract inverts for that surface via
three recorded moves (the #418 mechanics, now the template): (1) the zone row's reference
cell is re-pointed at a NEW capture set under marley-web/docs/captures/ (continuing the
numbering; one capture per acceptance state; note the px scale — the #418 set is 1× CSS,
the attached_assets era is 2×), with the old shots superseded ONLY for that surface's
rows; (2) an inversion note in the Zone A preamble names the surface, the tickets that
will re-align Marley, and the phrase that flips the burden ("the POC is the design source
and Marley is the bug until #N lands"); (3) the port-map row gains explicit port targets
(file:line seams + the model rule the port must keep). The inversion is SCOPED and
TEMPORARY — it ends when the ports land and the zone row's captures become plain ground
truth again. Never invert silently: without the recorded note, the parity hook's
"fix the POC, not Marley" default quietly reverts the redesign at the next ticket.

## AD-claude-415-transient-roster-membership-rule-001
*category: app shell / overlays · status: shipped (pipeline 415, M20)*

`close_transient_overlays()` membership is a RULE, not a list to intuit: every TRANSIENT
modal/picker with a key-arm in the router joins the roster — including the launcher itself
(#415; the helper was extracted FROM the launcher's closers at #393, which is exactly why the
launcher never made its own list). Persistent DOCKS are deliberately exempt: fleet is
toggle-scoped (#318), files is a workspace surface — a dock is not a transient. The exemption
and the rule live in the helper's doc comment AT the roster, not in tribal memory. Corollary
shipped with it: when a shared roster is extracted, sweep every PREEXISTING hand-closing site
onto it in the same change — the two menus that predated the helper (the #175 file-ref menu,
the terminal right-click) sat on a fossilized #398-era trio until #415
(F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001). The one-modal
smoke (`recipe_b_overlays_open_and_draw_headless`) asserts the rule as strict one-hot
equality; the POC needs none of this — its `activeOverlay` is a single enum, exclusivity by
construction (the enum-wholesale refactor is recorded prior art, not adopted).

## AD-claude-414-overlay-chrome-three-recipes-and-the-over-scroll-sibling-001
*category: app shell / overlays · status: shipped (pipeline 414, M20; completes AD-claude-318-overlay-chrome-two-recipes-001)*

The #318 sweep is FINISHED: zero verbatim copies of the 8-call chrome remain (negative-grep
pinned to the two helper bodies). The chrome now has a NON-MODAL sibling —
`overlay_card_chrome_over_scroll`, gpui's `block_mouse_except_scroll` in place of `occlude`
(clicks/hover blocked, wheel passes; gpui's own docs prefer it) — membership: overlays sitting
over scrollable content WITHOUT freezing it. First member: the editor completion popup, whose
scrolled-out-of-viewport guard is its scroll-dismiss and was UNREACHABLE under occlude (the
wheel never hit the editor). Hover keeps occlude deliberately (#318 D3). Positioning recipes are
now THREE: A anchored (caller-side W/MAX_H + menu_origin + mono — kept caller-side ON PURPOSE:
the six pickers' values differ per site and fonts need &self, so a recipe fn would be
shape-enforcement, not dedup; `anchored_overlay_card` as a &self method is the NAMED OPTION if a
7th anchored picker lands), B centered-quarter (quarter_overlay_card), C centered-input
(`naming_overlay_card` over pure `naming_card_geometry` 0.3·w / h/4 / 0.4·w, unit-pinned — the
trio's byte-identical `.p_3()` + text_color moved into the helper). Membership rule stays: a new
overlay declares its RECIPE + MODALITY instead of pasting chains; a new non-modal-over-scroll
member joins the sibling, never the modal fn.

## AD-claude-223-nine-role-type-scale-chrome-is-everything-but-command-output-001
*category: app shell / typography · status: shipped (pipeline 223, M12.2; extends the #337 decision)*

The cockpit type scale is NINE roles and the #337 invariant is now spelled as a complement:
CONTENT = Command/Output (track `appearance.font_size`); CHROME = everything else, fixed —
Caption(11)/Nav(12) since #337, and the #223 bands Panel(13, dock/git/diff/menu/hint),
Chip(10, fleet chips/meta), Badge(9), Headline(15, empty-state/browser-card), Display(22,
launcher title). Zero hardcoded `text_size(px(N))` sites remain in marley_app (negative-grep
pinned); the table is guarded by FULL-value asserts per arm (the struct-return-no-Default
trap — MSI cannot pin the data) plus fixed-across-weird-inputs probes that back the
free-fns' FONT_SIZE_DEFAULT-passing honesty pattern. Call shapes: inline
`px(type_scale(Role::X, self.font_size).size)` in methods; hoisted `*_size` bindings when a
role repeats in a fn; `FONT_SIZE_DEFAULT` in `&self`-less free fns (the caption_header
pattern). A future density tune or user chrome-size setting edits ONE table.

## AD-claude-365-env-gated-stdout-self-report-is-the-headed-state-assert-seam-001
*category: test lane / headed harness · status: shipped (pipeline 365, M22)*

When a contract lives in app STATE under a REAL platform service (real CoreText
font resolution — the #344/#361 resolvable arms), the headed assert seam is an
ENV-GATED STDOUT SELF-REPORT in the binary, not AX and not pixels: gate
first-thing on an exact env value (`MARLEY_FONT_POLICY_SELFTEST=1`; the
`MARLEY_WEBVIEW_PROBE` precedent), print ONE machine-parsable line after boot
(tab-delimited, free-text field LAST, tab/newline stripped at the printer so
the framing is a printer-local invariant), `cx.quit()`, and skip
`cx.activate` so an armed boot never steals frontmost. The drive spawns the
real binary (`CARGO_BIN_EXE_*`) with tempdir-hermetic `HOME`+`TMPDIR`+cwd, a
finite wall-clock deadline-poll (kill + a "live GUI session?" message — the
ghost-window constraint stays loud, never a hang), reads stdout BEFORE the
exit-status assert, and owns every assertion — the hook is a dumb reporter
under the caller's `mutants::skip`, adding zero mutation surface. Why not the
alternatives: `#[cfg(test)]` oracles don't exist in a spawned binary; a
Metal-painted surface never reaches the AX tree; pixels prove presence but
not text. Needs only a live WindowServer session — no Accessibility, no
Screen Recording — which makes this the LIGHTEST headed idiom; reach for it
before the AX/screenshot lane whenever the assert is about state, not looks.

## AD-claude-423-pty-exit-order-independence-is-grace-plus-state-latch-001
*category: terminal core / PTY lifecycle · status: shipped (pipeline 423, M29)*

The PTY child-exit seam must be ORDER-INDEPENDENT and its post-exit contracts
must live in STATE, because the two platforms disagree beneath errno: on Linux
the slave fds die WITH the child, so the master's read-`EIO` routinely beats
the SIGCHLD self-pipe byte (alacritty's event loop handles the identical race
with a Linux-cfg'd EIO-`continue`; we adopted the idea into Marley's per-tick
pump as a BOUNDED non-blocking grace — `eio_since` measured across pump calls,
never slept in-pump, with the boundary owned by a pure `eio_grace_expired` fn
so the `>=`-vs-`>` mutant dies at exactly-the-deadline, which a live `Instant`
can never pin); and Linux masters ACCEPT writes after the slave closes (the
bytes queue into the flip buffer), so "write-after-exit errors" can never ride
errno portably — the session latches `child_exited` when the exit event
surfaces and `write_bytes` refuses by state FIRST, uniformly. Once the exit
HAS been observed, a later EIO is an instant `Disconnected` again (no grace
re-arm per dead pane). Corollary for tests: PTY lifecycle asserts must state
their precondition (`assert!(exit_observed)`) rather than discard it — the
old member-3 test silently skipped its own premise for years.

## AD-claude-423-the-workspace-pins-its-toolchain-001
*category: build / gates / lanes · status: shipped (pipeline 423; rust-toolchain.toml)*

The workspace pins `channel = "1.96.0"` WITH `components = ["rustfmt",
"clippy", "llvm-tools"]` (the gate's needs travel with the pin — a fresh
minimal-profile rustup would otherwise honor the channel but redden gates
1/2/4 on missing components). Why: the dev-box mutation lane ran stable-1.94
against the mini's stable-1.96 and the skew masqueraded as "platform-sensitive"
test failures (trybuild ui snapshots pin rustc's diagnostic RENDERING, which
moves between minors) — and, worse, box mutation verdicts were not
same-compiler as the gate that consumes them. One pinned toolchain = one
rendering + honest cross-machine verdicts. `cargo +nightly miri` (gate:6) is
unaffected — the CLI `+toolchain` outranks the file. Bump the pin
deliberately in a normal gate-green commit; never per-machine.

## AD-claude-407-full-mutation-is-an-imported-verdict-behind-artifact-belts-001
*category: gates / mutation lanes · status: shipped (pipeline 407, M29)*

gate:5's FULL half no longer means "run the sweep here": on the 16 GB Mac the
sweep is BANNED (poisoned mutants allocate ~9 GiB/test-proc; the 2026-08-13
jetsam/WindowServer death), so FULL's default is an IMPORT of the dev-box
lane's measurement, made as trustworthy as a local run by ARTIFACT-DERIVED
belts (PR-claude-407-an-imported-verdict-needs-artifact-derived-belts-001):
literal-hex sha resolved and equal to HEAD, a MEASURED_SHA sidecar enforced
when present, outcomes-mtime ≥ HEAD's commit time, a foreign-repo probe, a
clean measured surface, and SET-EQUALITY between the outcomes and the CURRENT
tree's own `cargo mutants --list` enumeration. Key artifact realities learned
the hard way: an `--iterate` run's outcomes.json AND mutants.json hold only
run-local rows — the whole-workspace record is a GENERATION MERGE (newest
wins, enum-filtered; 407's three generations tiled 5,834/5,834 exactly), and
the completeness manifest must be generated from THIS tree, never trusted
from the pull. The counting itself lives in one `mutation_verdict <dir>`
helper (the #345 audit + MSI threshold) shared by every lane, so local and
imported verdicts can never drift. `MUT_FULL_LOCAL=1` remains the deliberate
capable-machine override (threads-capped), and the receipt fingerprint now
binds `rust-toolchain.toml` + `.config/nextest.toml` — the two gate-defining
files it had been missing.

## AD-claude-424-the-headless-pixel-stack-is-windowserver-free-and-we-wait-for-the-family-001
*category: test lanes / gpui upgrade posture · status: decided (spike 424, 2026-08-14)*

Two durable facts from the #424 spike (Zed tree pinned `a21007b7`, all reads
Apache-side). (1) **The macOS headless pixel stack needs NO WindowServer:**
offscreen Metal (device → renderTarget → readback), CoreText family
enumeration (SystemSource/CTFontManager/fontd), and glyph rasterization
(CTFontDrawGlyphs into CGBitmapContext) all produced byte-identical results
in the GUI session and over ssh — so a future headless lane via gpui's
`HeadlessAppContext` (TestPlatform + TestDispatcher + synthetic display +
INJECTED `MacTextSystem` + `current_headless_renderer`) is environmentally
sound, including real-CoreText font asserts. Its `VisualTestAppContext`
sibling wraps the REAL MacPlatform (pasteboards at construction, `show:
true` NSWindow) and is in-session-only — never the headless route. Traps:
`HeadlessAppContext::new` installs NO renderer (use `with_platform`); the
doc example injects CosmicTextSystem (wrong for macOS font policy); plain
`TestPlatform::new` silently Noops the text system; `font-kit` +
`test-support` features required. (2) **Upgrade posture: WAIT for the
published family.** gpui on crates.io stopped at 0.2.2 (Oct 2025) while the
tree split into `gpui` + `gpui_macos`/`gpui_platform`/… — early adoption
means a fork or an unpublished git family (~1,300–1,500 adapted lines, a
deny.toml sources amendment, double migration). The watch is
`crates.io max_version > 0.2.2`; the adoption recipe lives in the #424
ticket memo.

## AD-claude-425-display-map-facade-and-typed-row-spaces-001
*category: editor / display stack · status: shipped (pipeline 425, M32)*

**Context:** The B-c chain (soft wrap #426, multibuffer #427+) multiplies row
spaces, and the fold projection was consumed RAW at 16 app.rs sites — the
F-#352 row-vs-slot class waiting to recur per new space.

**Decision:** ONE facade owns every buffer-row↔display-row conversion:
`marley_app::display_map::DisplayMap` (folds-only today, delegating to
`marley_syntax::FoldProjection` unchanged), with the two row spaces as
workspace-vocabulary newtypes `marley_text_offsets::{BufferRow, DisplayRow}`
(private fields, trybuild-pinned distinct). The rim unwraps ONCE; interior
render sites keep buffer-row `usize` (D-PROJECT-AT-THE-BOUNDARY survives);
`EditorFrameGeom.first/last` + `HoverCard.first_row` are typed slots. THE
CONTAINER IS DELIBERATELY SMALL: no generic layer trait until a second layer
exists (#426) — the Zed six-layer stack is the reference CONTRACT, not the
shape ("Marley's per-line render admits a smaller shape"). Columns stay OUT:
`LineLayout`'s #331 two-boundary maps remain the column authority; the wrap
layer is where rows and columns first negotiate.

**Consequences:** #426 inserts wrap INSIDE the facade (visible_count /
buffer_row / slot_of / viewport_offset compose fold∘wrap) without re-touching
the crossing sites. #427's excerpt base slots under the fold layer the same
way. A new row space = a new newtype in the vocabulary crate + a facade layer
— never a raw usize at a call site. The `identity(total)` fast path preserves
#305's no-parse guarantee and must stay allocation-trivial.

## AD-claude-426-soft-wrap-is-a-facade-layer-on-cell-arithmetic-001
*category: editor / display stack · status: shipped (pipeline 426, M32)*

**Context:** Soft wrap breaks the row↔line identity every render/geometry rider
assumed; the #425 facade existed precisely to absorb that break.

**Decision:** Wrap is the display map's SECOND layer, never a render hack:
`wrap.rs` computes break CELLS over the one phantom-aware `LineLayout` (gpui
LineWrapper's RULES — word-boundary candidacy after the first non-whitespace,
CJK any-break via an ASCII word class, hard-break fallback, indent capped to
keep MIN_TAIL=8 content cells — re-expressed as pure integer arithmetic on the
mono grid; machinery not taken), and `DisplayMap` composes folds ∘ wrap behind
`locate/slot_at/viewport_offset_at` with an Arc'd memoized `WrapIndex` (key:
nonce, version, anchors, wrap_cols, tab_width, per-phantom (row, char, cells)).
The rim renders one slot per SEGMENT by clip-and-re-base of the whole-line
layout's byte ranges — NEVER a per-segment re-lex. Vertical motion injects a
display step into the editor's generic carry (`move_all_vertical_by`); the goal
column is a CONTENT cell within the segment, reset at mode toggle. The whole
#336 h-scroll set is gated structurally inert while ON; the clipper/shift split
survives for OFF, which is byte-identical.

**Consequences:** (1) Every column that enters segment math MUST come from the
phantom-AWARE layout (`aware_display_col` / the rim's own layout) — the
phantom-blind columns that were a sub-cell x-nuance become wrong-display-row
bugs across a coarser map (PR-claude-426). (2) The wrap walk's own theorems are
load-bearing: the first-nonspace gate + MIN_TAIL floor bound the rollback carry
under every capacity, which makes two defensive guards provably equivalent
mutants — they are REMOVED with the proof inline (AD-claude-305 discipline),
so a future edit that weakens either premise must re-prove or re-guard. (3)
The #427+ excerpt base slots UNDER folds in the same facade; wrap needs no
knowledge of it.

## AD-claude-427-multibuffer-own-row-model-snapshot-origin-001
*decided at: #427 (M32, the B-c chain's step 3) · status: shipped*

Three durable decisions in the multibuffer's first slice. (1) **Own row model
this slice** — the surface renders its own prefix-summed slot→Header|Line model
(the #426 `WrapIndex` shape) in its own `uniform_list`, NOT through the #425
DisplayMap; folding excerpts into the facade is the chain's deferred unification
prize, recorded in display_map.rs's module doc so the facade doesn't misdirect.
(2) **v1 is a SNAPSHOT with absolute rows** — text captured at materialization
via the two-source rule (live `editors_under` buffers win; else the viewer guard
chain, LOSSY + post-read re-check — exactly the worker's text); the model stores
(canonical path, absolute rows, char spans) so #428 can re-anchor. (3) **The
NavStack origin is materialization-time, pushed ONCE** — `active_editor()` is
None while the tab is active (it has no caret), so the model carries `origin`
(the editor loc at ⌘⏎), `take()`n on the first successful jump; after that the
still-open tab itself is the way back. The full-row match wash (accent 0.10;
selected 0.16 wins) is the POC's own form and what keeps `with_highlights`
single-producer (syntax ranges only — F-claude-427-a).

## AD-claude-428-excerpt-editing-rides-existing-machinery-001
*decided at: #428 (M32, the B-c chain's step 4) · status: shipped*

The editable multibuffer added NO new editing machinery: edits ride the ONE
insert mechanism (`replace_text_ctx` → the engine fns) against the file's ONE
registry instance (#397); excerpt geometry is #269 anchors resolved per frame
(the folds' shape) with an epoch re-mint on reload; undo is the per-buffer
`UndoHistory` plus a thin ROUTING journal (one entry per group, keyed by
`undo_depth`, liveness-guarded at pop); saves are `save_editor_by_id`'s
background-consent arm over a TOUCHED set. The three contracts that made it
safe, each earned at inspect: the input seam is PER-SURFACE (register your own
`handle_input` canvas — PR-claude-428-a); borrowed shared state restores itself
(the buffer's live selection — PR-claude-428-b); a routing journal over a
shared history needs a liveness guard, not just order (F-claude-428-c).
Boundary policy: window edges judged against the RESOLVED anchors; deletes
never cross them; interior edits grow/shrink naturally.

## AD-claude-429-replace-all-is-a-batch-over-the-journal-not-a-new-history-001
*decided at: #429 (M32, the B-c chain's step 5) · status: shipped*

Replace All added no second undo system. Each file's apply brackets its OWN
per-buffer undo group (the #282 sweep shape), joins the SAME cross-excerpt
journal as hand edits (`journal_note_edit`), and the gesture-ness lives in one
positional ledger entry: `batches: Vec<(start, len)>` over the journal.
`batch_covering` pops the whole run only when the journal's top closes it
EXACTLY (`start + len == journal_len`); any foreign push/pop invalidates via
`batches_invalidate` (retain `start + len <= len`); `redone_batches` mirrors
for ⌘⇧Z. Consequences: one ⌘Z reverts N files with zero new undo state; a
hand edit after an apply naturally DEMOTES the batch to per-entry undo (the
ledger drops, the journal stands); the compile-once request
(`SearchReq.compiled`, `Option<CompiledFind>`) keeps the regex fork's cost at
one compile per walk and the invalid-pattern verdict at spawn time, not
per-file. Guards fail closed at the gesture (PR-claude-429-a).

## AD-claude-430-diagnostics-refresh-gates-on-a-publish-epoch-not-row-counts-001
*decided at: #430 (M32, the B-c chain's closing step) · status: shipped*

The diagnostics multibuffer refreshes by full re-materialize at QUIESCENT
points: (publish epoch moved OR a mb save armed it) AND no mb target dirty.
The gate is a MONOTONE PUBLISH EPOCH (`DiagnosticStore::publish_epoch`,
bumped on every replace, clears included) summed across hosts — NOT a row
count: the live drive proved rust-analyzer's republish can swap error+hint
for warning+hint (sum unchanged) and a count fingerprint swallows it while
the ⌘S arming self-consumes before the async publish lands. Consequences:
any real republish triggers exactly one rebuild at the next clean tick;
typing never loses the caret (dirty holds); the snapshot semantics reset
mb-side gesture state (journal/batches/touched) at each rebuild — per-buffer
undo in the files' own tabs is untouched. The band is a REAL SLOT
(`Row::Note` over a materialized `slots: Vec<Row>`) because `uniform_list`
demands uniform row heights — in-slot stacking was a design dead end.
Targets pin/birth under `root_of_mb_path` (longest project-root prefix) —
a workspace-wide aggregation must never blanket the active root.

## AD-claude-431-excerpt-stage-is-a-facade-arm-disjoint-until-composed-001
*decided at: #431 (M33, the B-c chain's unification prize) · status: shipped*

Three durable calls in the excerpt unification. (1) **A third Option arm on the
ONE `DisplayMap` struct, not a stage enum** — every editor constructor sets
`excerpt: None`, so the fold∘wrap instance is bit-for-bit by construction and
derived equality extends inertly; an enum would force the editor arms through a
new match for zero payoff until stages actually compose. (2) **`ExcerptIndex`
is the ONE home for mb slot math** (mint + locate + slot_of_row + mover +
normalize, with normalize delegating its scan to the mover), Arc-memoized on
`MultibufferModel`, rebuilt ONLY by `refresh_cum`, served per crossing by
`model.display_map()` — the `WrapIndex` cost discipline
(BF-claude-fold-projection-parse-on-pump-tick). The model DELETED its slot
methods rather than delegate (PR-1691); `line_at` stays model-side as a
files-BORROW adapter over the index. PAIRING INVARIANT on the memo: any
`files`/`lines`/`match_meta` mutation calls `refresh_cum` in the same borrow.
(3) **Stages are DISJOINT this slice, deliberately**: the mb instance carries an
identity fold slot and `wrap: None` — those fields are the recorded landing zone
where folds/wrap-INSIDE-excerpts compose facade-internally, so the wedge
features change `display_map.rs`, not the app.rs rim. The rim speaks typed
`DisplayRow` (`selected` included); counts stay `usize` at the gpui edge; model
indices (`fi`/`li`/`mi`, `ExcerptLine.row`) stay `usize` — they are indices, not
row spaces. Deliberate in-crate module cycle, documented in both module docs:
the stage consumes the model's group vocabulary; the model memoizes the stage.
A future crate split moves `Row`/`FileExcerpts` into the facade module first.

## AD-claude-432-mb-input-rides-recorded-geometry-and-a-surface-claim-001
*decided at: #432 (M33) · status: shipped*

Three durable calls in the mb input completion. (1) **A SEPARATE recorded
geometry cell per surface** (`MbFrameGeom` beside `EditorFrameGeom`, never a
shared struct): the mb probe is zero-width but ROW-TALL — its measured height
IS `cell_h` (one pitch for bar paint, y0 normalization, and the dy/cell_h
inverse; a "simplify to zero-size" pass would kill every consumer — documented
in the struct doc), records on the first LINE row of each batch (headers/bands
have no text cell), and a batch with NO Line row ZEROES the cell so IME
consumers answer `None` instead of stale prior-range rects. (2) **Input
ownership is claimed BY SURFACE, not by key list** (PR-claude-fail-closed-
claims-enumerate-every-input-lane): the mb ⌘C/X/V arm consumes ALL modifier
variants, and the plain-key claim (the #251 editor twin) bare-returns WITHOUT
stop_propagation so the platform text path keeps delivering printables/IME —
the pattern for any future surface that coexists with the hidden workspace
terminal. (3) **Caret and composition die together, everywhere**: `mb_marked`
is a raw offset pair, so every gesture that moves/drops the caret (click,
arrows, ⌫/⌦/⏎, Esc, chord verbs, undo/redo, epoch re-mint) clears it in the
same arm — the read arms answer from the anchor + `mb_marked` and never the
target buffer's live SelectionSet (the #428-b rule, now compiler-shaped via
`mb_caret_buf`).

## AD-claude-433-terminal-problems-producer-source-owned-epoch-pair-gates-001
*decided at: #433 (M33, the Phase-C wedge's first fusion) · status: shipped*

Four durable calls in the terminal problems producer. (1) **The change signal
is SOURCE-OWNED**: `terminal_blocks` mints a monotone per-session
`block_epoch` at the ONE `SessionModel` choke both ingestion paths funnel
through (born at Preexec; finished at Precmd or child-exit — `current_mut`'s
Running filter makes every bump transition-exact, incl. the double-finish
case). The consumer never counts state; it compares epochs — the
`DiagnosticStore::publish_epoch` shape on the terminal lane. (2) **Gate
fingerprints are PAIRS, never cross-lane sums** — (LSP, terminal) compared
tuple-wise; summing lanes lets a +1 publish cancel a closing 1-epoch terminal
(the #430 collision class across lanes). The cross-SESSION sum inside the
terminal component inherits the LSP twin's accepted close-vs-bump window
(self-healing; recorded). (3) **One walk, cheap payload**: the D2 enumeration
(`projects() × terminal_grids() × states()`) has ONE spelling carrying
VERBATIM roots — no IO on the per-tick gate path; the producer canonicalizes
once per distinct root at its epoch-gated derive and scans ONLY the
supersession fold's winners. (4) **Freshness = latest completed per
(command, pwd) per pane** (`latest_failed_indices`) — a passing rerun retires
its failure, a running one does not; deliberately divergent from the #289
gutter's last-block-only lane, both recorded.

## AD-claude-434-runnables-detection-in-syntax-mint-in-app-sink-is-the-block-001
*decided at: #434 (M33, Phase C opens) · status: shipped*

The runnables architecture in three cuts. (1) **Detection lives in
marley_syntax, the mint in the app** — `runnables_in` answers WHAT is runnable
(an iterative TreeCursor walk; module-scope by the `declaration_list`-under-
`mod_item` grandparent gate; attribute paths matched `test`/`::test`; name
nodes kind-gated `identifier` so grammar-error metavariables never carry `$…`
toward a shell); `run_command` answers what to RUN — grammar knowledge and
cargo knowledge never share a crate. (2) **The spawn IS typed input**: resolve
the lowest idle workspace pane through the ONE `idle_workspace_panes` spelling
(the #40 guard; `rerun-last-failed` re-expressed over it), then
`history.record` + `write_command` + the R39 viewport re-anchor +
`jump_to_pane` — shell integration frames the Block, zero new process
machinery. The Block model is the SINK: Zed's own pipeline (per the fusion
deconstruction) ends in a terminal tab + summary line; routing the same
discovery into Blocks is the wedge. (3) **The gutter reserves the ▶ cell on
EVERY row** (uniform width — an intrinsic-width gutter cannot tolerate per-row
prefix jitter), and every row-PREFIX mirror (`code_area_left_px`, the sticky
band) composes the cell in lockstep (PR-claude-row-prefix-mirrors).

## AD-claude-435
The run-block identity architecture (M33 #435): the identity rides the BLOCK
(`Block.run_tag: Option<String>`, opaque — `terminal_blocks` stays
toolchain-free; the codec lives app-side in `marley_app::runnables`), staged
on the session as a `(tag, command)` PAIR after a SUCCESSFUL `write_command`
and MATCH-BOUND by the Preexec reporting exactly that command. The pattern:
correlation state between an app action and a shell event should (a) stage in
the same sync region as the write, (b) stage only after the write succeeds,
(c) bind by CONTENT MATCH, not by order, and (d) refuse to overwrite — then
every failure direction degrades to "no label" (fail-safe), never "wrong
label". Supersession is derivation-time, not event-time: the F8 target fn
takes `(tag, StatusKind)` per block and retires an identity on a later
same-tag Success — "clear on green" falls out of derive-at-press ONLY when
the derivation models supersession (the inspect HIGH). The chord law: an
unmodified key bound in the Terminal context yields to the PTY whenever
`is_alt_screen || is_command_running` (R40 outranks the keymap).

## AD-claude-443-mutation-topology-and-no-masks-001
*decided at: #443 (workbench shell baseline) · status: shipped*

How the fork's mutation gate runs. **DIFF** mutates in place, one job by construction (no
`--jobs`), with `-p` for every touched package, and fails closed when the diff has crate lines
but no package resolves; it keeps the shared, warm target because it is the only writer.
**FULL** runs copy mode with two workers, each copy under `env -u CARGO_TARGET_DIR` so it builds
in its own target, since cargo leaves the checkout path out of a workspace crate's artifact
names. **Both** pass `--no-config` and clear `CARGO_MUTANTS_OUTPUT`,
`CARGO_MUTANTS_MINIMUM_TEST_TIMEOUT`, `CARGO_BUILD_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR`.
**No masks:** mutation runs over every line of the Marley crates. gate:12 bans `mutants::skip`
in any spelling, `cfg_attr` included, and the receipt fingerprint covers `.cargo/config.toml`
and `.cargo/mutants.toml`. An equivalent or unreachable mutant is answered by re-expressing the
code (the removed `content_length > 0` guard, the merged `cfg` twins of `spawn`), never by a skip.

## AD-claude-436-one-owned-set-three-enforcers-001
*decided at: #436 (workbench shell W0) · status: shipped*

The ledger of changes outside the Marley-owned paths is enforced from one definition in
`.claude/hooks/lib-hook-helpers.sh` (`marley_owned_path`, `zed_ledger_check`, `upstream_base`)
at three moments: the write (`enforce-zed-ledger.sh`, a Write or Edit), the gate (gate:16, every
mode), and the commit (`enforce-commit-gate.sh`, every commit, Rust or not). A row is an exact
path, a backticked path opening the first column of the Touchpoints table. A file is judged by
the checkout it lives in (git's answer for the file, compared by `--git-common-dir`), and each
linked worktree by its own ledger. "Changed" is the work tree, the index and untracked files
against the upstream fork point, listed with `-z`. The owned set must stay disjoint from
upstream, which gate:16 checks, so "owned" can never exempt a Zed file.

## AD-claude-437-marley-identity-is-app-name-and-the-dev-channel-001
*decided at: #437 (workbench shell W1) · status: shipped*

The fork's identity in two Zed touchpoints: `paths::APP_NAME = "Marley"` (every config, data,
cache and log directory derives from it) and the `marley` binary (`default-run` and the main
`[[bin]]`, tied to `APP_NAME` by `main.rs`'s compile-time assert). The rest of Zed's identity
(the Secret Service label, the updater, the app id, the `zed://` scheme, the CLI) is keyed by the
release channel, and the channel stays `dev`, guarded by a `paths` test, until TICKET-445 gives
Marley its own. Packaging names (the crash label, the clap name, bundles, desktop entries) wait
for that ticket too.

## AD-claude-438-the-marley-layout-swaps-the-sidebar-and-two-defaults-001
*decided at: #438 (workbench shell W2) · status: shipped*

The Marley layout is a setting, `marley.layout` in a `marley` block of the settings content,
and per window a different `workspace::Sidebar`. `marley_workbench::register_sidebar` builds
Zed's sidebar or the rail; it is called from the one place Zed built its sidebar and from a
settings observer for every window, and no Zed code learns that the layout exists. Two
defaults move with it, `terminal.button: false` and `agent.dock: right`, patched below the
user's settings with `update_default_settings`, so the user's own values win in both layouts.
The rail keeps the Zed sidebar it replaces alive, with its open state, and answers the
window's persistence with that sidebar's state, so a switch loses neither the sidebar's state
nor the work it has running (F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001).
Everything that decides what the rail shows lives in the pure `marley_rail`. Considered and
dropped: a flag in Zed's key-value store, invisible to `settings.json` and profiles; dropping
and rebuilding Zed's sidebar on each switch, which lost its state and cancelled its tasks; a
builder that `zed.rs` passes in, which grows the Zed hunk while the kept sidebar needs the
concrete type anyway.

## AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001
*decided at: #438 (workbench shell W2) · status: shipped*

`marley_workbench` is the first Marley crate that depends on Zed's GPL-3.0-or-later crates
(eleven of them, `workspace` and `sidebar` among them). It keeps the Marley license, MIT OR
Apache-2.0, as CONSTITUTION §20 and the three-prong plan require: its code is written from
Zed's public contracts and never carries a Zed body over
(PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001), and both licenses
are compatible with GPLv3. The consequence is that the crate builds and ships only as part of
the fork. Anything that must stay reusable outside it goes in a crate with no Zed
dependencies, as `marley_rail` does for the rail's row model. `deny.toml` is unaffected,
since it skips unpublished workspace crates. The crates' license files wait on TICKET-446.

## AD-claude-the-workflow-is-four-phases-and-mutation-waits-for-the-end-001
*decided at: 2026-09-22 (Chad's directive) · status: shipped*

The Marley workflow is Plan → Code → Test → Complete and nothing else. Plan holds the
pre-flight, the recall, the ticket, the spec and the design; Code holds the code and a review
of its diff; Test holds the tests, the live drive and the DIFF gate; Complete holds the docs,
the knowledge capture, the archive and the commit. The separate `/work`, design, inspect and
`/commit` steps are gone. Mutation testing left the per-change gate because it was too slow
per change: `script/mutation.sh` runs it over the Marley crates at the end of a sprint, as
rustal keeps its own mutation audit outside `bin/gate.sh`. Every other Rust quality tool
stays in the gate. Loosened on a recorded reason, as the amendment rule asks: the per-change
MSI 100 floor and the mandatory critic phase.

## AD-claude-447-the-marley-crates-carry-rustals-lint-table-001
*decided at: 2026-09-22 (Chad's goal: "mimic the quality gates on rustal") · status: shipped*

Each Marley crate carries rustal's lint table in its own manifest:
- clippy's pedantic, nursery and cargo groups at warn, which `-D warnings` makes binding;
- a deny list with `missing_docs`, `missing_debug_implementations`, `unsafe_code`,
  `unwrap_used`, `expect_used`, the doc-section lints and Zed's own denies;
- `let_underscore_must_use`, added so Zed's `.rules` ban on `let _ =` for a fallible call is
  mechanical.

The table lives in each manifest because cargo gives a crate either the workspace's table or
its own, and Zed's crates keep Zed's. The seven tables differ only in crate-specific allows,
each with a comment in its manifest: the three cargo lints that read Zed's manifests, and
`future_not_send` and `unused_results` in the gpui crate. Test code may unwrap and expect
(`clippy.toml`); library code may not. In a private module, `unreachable_pub` against
`redundant_pub_crate` is settled by structure (`pub(super)` two or more levels deep, a `pub`
module when siblings reach it), never by an `#[allow(unreachable_pub)]`. The alternative, Zed's
workspace lints alone, let 679 findings stand in the ported crates: unwraps on response paths,
discarded fallible results, missing docs and casts that wrap.

## AD-claude-447-a-gate-run-names-its-mode-and-revokes-the-old-receipt-001
*decided at: 2026-09-22 · status: shipped*

`script/gates.sh` takes `--full`, `--diff` or `--fast` and nothing else; a missing or unknown
mode is a usage error before any gate runs, so no run is FULL by accident. A receipt-bearing run
removes the old receipt before its first gate. A tree that passed once and fails now cannot then
commit on the older green. The new receipt binds the fingerprint taken at the start, and the run
fails when the gated files at the end differ, so the receipt always names the tree the gates
ran on. After a static red the heavy gates report BLOCKED instead of running: coverage over a
tree that fails clippy proves nothing. The new checks take the numbers 17 to 20; retired
numbers (5 and 15) stay retired so old notes keep their meaning. All of it is carried over
from rustal's `bin/gate.sh`, scoped to the Marley crates.

## AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001
*decided at: 2026-09-22 (W3 promotion, revising the queued D4) · status: shipped*

The rail answers `is_threads_list_view_active` with `false`, though it lists threads. The trait
defaults to `true`, and Zed reads `true` with an open sidebar as "every thread in this window
is on screen":
- no OS pop-up and no sound for any thread (`conversation_view.rs:2863-2987`);
- none for the Agent Panel's terminal threads (`agent_panel.rs:2913-2932`);
- the title bar's project button becomes the recent-projects popover (`title_bar.rs:812-835`).

The rail does not list terminal threads, so claiming the list would let one finish with no
signal anywhere. With `false`, a thread off screen still pops up beside its dot: noise, not a
loss. Revisit when the rail lists every kind of thread Zed would silence.

## AD-claude-439-focus-decides-between-a-terminal-row-and-a-thread-row-001
*decided at: 2026-09-22 · status: shipped*

A window can show a center terminal and an Agent Panel thread at once, and the rail has one
selected row. The thread's row wins only while the displayed workspace's Agent Panel holds
focus; otherwise the #438 order holds (the active terminal, then the project header). The
selected row is then whatever the keyboard is typing into. The pure selector owns the order
(`marley_rail::selection`), and the rail re-reads focus when it enters or leaves a panel.

## AD-claude-440-agent-clis-are-read-from-argv-and-started-in-one-click-001
*decided at: 2026-09-22 · status: shipped*

An agent CLI in a rail terminal is recognized by its foreground argv, never the process name:
Claude Code's binary is named after its version. It is judged by its output. Waiting means a
bell, or two seconds without output; a quiet spell shows only in the row's status line, since a
dot for every pause would be noise, and the bell keeps its W2 dot. The project's `+` menu lists
the installed CLIs as entries under an "Agent CLIs" header, not in a submenu, so starting one
is a single click; the original complaint was not finding how to start an agent at all. The
launch writes only a program name from `marley_agent`'s list, after the shell's startup
handshake, so nothing but a known command reaches the shell.

## AD-claude-441-the-marley-layout-routes-terminals-without-touching-zed-001
*decided at: 2026-09-22 · status: shipped*

In the Marley layout nothing opens the bottom Terminal Panel, and no Zed crate changes for it.
A task provider of `marley_workbench`'s own replaces Zed's when the workspace announces the
panel. It sets each task's reveal target to the center and still runs it through
`TerminalPanel::spawn_task`, so Zed's rerun and reuse rules hold; a task's terminals left in the
panel move to the center before it reruns. Capture-phase listeners on the workspace's root take
New Terminal and Open in Terminal before the panel's handlers do. Both halves read the layout
at each call, so the Zed layout stays upstream's and a switch reinstalls nothing. The panel
stays loaded, out of sight, since tasks and agent logins need it. Rejected: a hunk in each of
the panel's handlers (a Zed diff per path), dropping the panel (a workspace with no provider
fails every task silently), and keymap remaps (they miss menus and the command palette).

## AD-claude-449-terminal-keys-catch-zeds-actions-and-the-keymap-waits-for-new-keys-001
*decided at: 2026-09-23 · status: shipped*

In the Marley layout the terminal keys work on center terminals because the crate catches the
actions Zed's defaults bind them to (`terminal_panel::Toggle`, `ToggleFocus`, and a
`workspace::ToggleBottomDock` that would show the Terminal Panel), not by rebinding the keys.
The palette, the menus and a user's own bindings to those actions route the same way, and no
keymap asset or Zed touchpoint is needed. All three run one toggle: from a focused center
terminal, back to the center item used last that is not a terminal; from anywhere else, the
center terminal used last, or a new one. workbench-shell D7's Marley keymap, with its one line
in `load_default_keymap`, waits for a key with no Zed action behind it, the first being the New
Agent chord (TICKET-450). Rejected: the keymap D7 planned for these keys, which would have
missed the palette and the menus and cost a Zed touchpoint for keys Zed already binds.

## AD-claude-450-new-agent-is-a-picker-behind-the-marley-keymap-001
*decided at: 2026-09-23 · status: shipped*

Starting an agent from the keyboard is `marley::NewAgent`, a picker in the workspace's modal
layer, bound to `secondary-alt-n` by the Marley keymap. The picker lists Zed's agents, then the
installed agent CLIs, marked "Thread" or "Terminal", and starts the choice in the active
project. It works in both layouts and without the rail open, which the rail's `+` menu cannot.
The rail and the picker share one module, `marley_workbench::agents`: the choices, the two
launches, and one seam, `Launcher`, for the search path and the terminal factory. The Marley
keymap is a JSON file in the crate. `load_keymap`, called last in Zed's `load_default_keymap`,
binds it tagged as a default source, so each reload binds it again, it beats Zed's defaults at
equal depth and loses to the user's keymap. The chord was swept against every context of every
keymap Zed ships. Rejected: opening the rail's `+` menu from the key (no rail in the Zed layout
or while it is closed), and binding at init (every reload clears it).

## AD-claude-442-the-rail-adds-its-fields-to-zeds-sidebar-blob-001
*decided at: 2026-09-23 · status: shipped*

The rail keeps its state in the window's saved sidebar blob, the one Zed's sidebar owns, by
adding fields to it: `width` and `width_set_by_user` under Zed's names, and
`marley_rail_closed`, which Zed's sidebar ignores. Every other field is kept, so Zed's sidebar
restores from what the rail saved, and the rail from what Zed's saved. One width holds across
both layouts: the rail forwards its width to the Zed sidebar it keeps and starts at that
sidebar's width. A close is remembered in the rail's own field, since the `MultiWorkspace`'s
`sidebar_open` only ever reopens on restore and the Marley layout builds each window's rail
open. The restore closes the rail again once no entity is being updated. Rejected: a blob of
the rail's own (the #438 failure: Zed's state leaves the database), and a Zed touchpoint for a
silent close or a close-aware restore.

## AD-claude-451-zeds-layout-presets-explain-themselves-in-the-marley-layout-001
*decided at: 2026-09-23 · status: shipped*

In the Marley layout Zed's Panel Layout presets (`workspace::UseClassicLayout`,
`workspace::UseAgenticLayout`) are caught in the capture phase and answered with a toast, not
run: they rewrite the docks the layout sets, and Agentic's `agent.dock: left` would outlive the
layout. The toast says the presets belong to Zed's layout and offers a switch to it. The title
bar's submenu stays as Zed draws it, since hiding it needs a `title_bar` touchpoint and choosing
an entry now explains itself; the palette entries route the same way. Rejected: a palette
filter (the title bar re-applies its own on every settings change), silently swallowing the
actions, and a `title_bar` touchpoint.

## AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001
*decided at: 2026-09-23 · status: shipped*

A terminal row renames and closes through Zed's own tab controls, which the rail only starts:
Rename shows the terminal and runs `TerminalView::rename_terminal`, whose editor lives in the
tab and whose result Zed persists; Close runs the pane's `close_item_by_id` with
`SaveIntent::Close`, so Zed's prompt for a running task comes up as it does for the tab. The
row offers both from a right-click menu, a double-click renames, and a close button takes the
bell's slot on hover. A custom title outranks an agent CLI's own on its row, since the user
named it. Rejected: an editor inside the row (a second rename with its own persistence, keys
and focus handling), and closing the item directly (it would skip Zed's prompts).

## AD-claude-453-the-rails-keys-are-zeds-list-actions-001
*decided at: 2026-09-23 · status: shipped*

The rail answers Zed's own list actions and binds no key of its own: `menu::SelectNext`,
`SelectPrevious`, `SelectFirst`, `SelectLast`, `SelectParent`, `SelectChild` and `Confirm`. Zed
binds up, down, Home, End and Enter to them with no context, and left and right only in the
`menu` context, so the rail's key context is `MarleyRail menu`, as the Threads Sidebar's is.
While the rail holds focus, the row the keyboard is on is the rail's one selection:
`Focus::cursor` in `marley_rail`, which `selection` prefers while that row is shown. The cursor
starts from the highlighted row, stops at the ends, and is dropped when focus leaves the rail.
Enter runs the row's click handler. A project header's right-click menu reorders through
`MultiWorkspace::move_project_group_up` and `move_project_group_down`. Rejected: Marley bindings
for keys Zed already binds (a keymap entry and a shadow sweep for each), a keyboard highlight
beside the selection (two highlighted rows), and wrapping at the ends.

## AD-claude-457-the-rails-filter-is-zeds-sidebar-filter-001
*decided at: 2026-09-23 · status: shipped*

The rail's filter adopts Zed's Threads Sidebar filter. It takes Zed's matcher,
`agent_ui::threads_archive_view::fuzzy_match_positions`, which despite its name is a substring
match that ignores ASCII case. It takes Zed's rules: a project shows for its name or a row
under it, a name match shows every row, and the fold is ignored. It takes Zed's action,
`agents_sidebar::FocusSidebarFilter`, which the Marley keymap binds to `secondary-f` in
`MarleyRail && !Picker`, and Zed's Escape. The gpui side matches the strings the rows draw and
hands the positions to `marley_rail`, whose one walk decides what shows for the rows, the
selection and the keyboard alike. Two things differ from Zed's sidebar: the rail's focus key
still lands on the rows (#453), and each edit moves the keyboard's row to the first row that
matched, header or not. Rejected: `fuzzy` or `fuzzy_nucleo` in `marley_rail` (both pull in
gpui, and neither is what Zed's sidebar uses), a matcher written for Marley, a `marley::`
action with the same meaning, and landing the focus key in the filter as Zed's sidebar does.

## AD-claude-454-the-rails-switcher-is-zeds-thread-switcher-over-the-rails-rows-001
*decided at: 2026-09-23 · status: shipped*

`ctrl-tab` in the rail and in the Agent Panel opens the rail's switcher over the window's center
terminals and threads, through `Sidebar::toggle_thread_switcher`, as Zed's sidebar opens its
thread switcher; the center panes keep Zed's tab switcher, as in Zed's layout. The Agent Panel
keeps Zed's binding. The rail gets `ctrl-tab` and `ctrl-shift-tab` from the Marley keymap in
`MarleyRail && !Picker`, and the view uses Zed's `ThreadSwitcher` key context, so Zed's
bindings step it. The view sits in the `MultiWorkspace`'s sidebar overlay, focused by the rail,
and keeps Zed's rules: two entries at least, the second selected, confirm on the release of
the modifiers it opened with, on Enter or on a click, and cancel on Escape or focus-out. Recency
is the rail's own counter over changes of the row that holds the window's focus, in memory and
pruned. The order is most recent first, then the rest in the rail's order, whatever the fold and
the filter. Rejected:
- `ctrl-tab` in the `Workspace` context, which would take Zed's tab switcher away in both
  layouts, since the keymap does not know the layout (a window-wide switcher from the center is
  Chad's call);
- reusing Zed's `ThreadSwitcher` view, whose entries are Agent Panel threads and terminals;
- ordering by a pane's `activation_history`, which counts per workspace;
- recency by what the window displays (F-claude-454-recency-noted-a-terminal-the-user-never-went-to-001);
- a preview while cycling.

## AD-claude-455-a-first-terminal-reads-zeds-own-lookup-001
*decided at: 2026-09-23 · status: shipped*

A folder project opened fresh in the Marley layout starts with one center terminal at its root,
and a project opened from saved state keeps exactly what it saved, terminals or none. The
routing's `observe_new` hook seeds the terminal. It knows fresh from restored through one Zed
touchpoint: `Workspace::opened_from_saved_state()`, which `new_local` records from its own
`workspace_for_roots` lookup in both closures that build the workspace. That is `Some(false)`
for fresh, `Some(true)` for restored, and `None` for a workspace made any other way. Observers
of a new entity run after the creating update, so the hook sees the value and a workspace
already in its window. Nothing is seeded on a layout switch, for a window with no folder, or for
`None`. Rejected:
- `is_restoring`, which `load_workspace` sets a turn after the workspace exists;
- `WorkspaceAdded`, which means pinned;
- `database_id`, which is `Some` for both kinds;
- reading the workspace database from Marley, which races the project's first save;
- seeding every project with no terminal, the gpui-era force-seed failure
  (`BF-claude-boot-restore-force-seeds-a-terminal-after-the-runtime-guard-dissolved-001`).

## AD-claude-456-a-layout-round-trip-gives-each-dock-back-its-panel-001
*decided at: 2026-09-23 · status: shipped*

Zed's dock move stays as it is. The Marley layout switch keeps a memory around it from outside
the `workspace` crate, with Zed's public dock API, and needs no touchpoint:
- The switch's settings observer is registered at init, before any dock's, so it runs first.
  It notes every workspace's docks before they move (open or not, and the active panel by
  persistent name), then settles them in a `cx.defer`, after the move and before the move's
  throttled save.
- The dock the Agent Panel entered remembers the panel it showed and whether it was open, per
  workspace, in `LayoutState::displaced`.
- The dock the Agent Panel leaves gets that panel back, open only if it was open both before
  the trip and as the Agent Panel left. This happens only while the Agent Panel was still the
  panel it showed, so a panel the user chose there in between stands, and so does a close.

Rejected:
- a touchpoint in `dock.rs`, which upstream's own tests pin to the current closing behavior;
- keeping the memory across restarts;
- remembering only when the move made the Agent Panel the dock's shown panel. That condition
  would forget the panel when the Agent Panel arrived hidden and was shown later.

## AD-claude-458-the-rail-follows-each-projects-folders-with-a-deferred-rebuild-001
*decided at: 2026-09-23 · status: shipped*

The rail subscribes to each listed workspace's project, as Zed's Threads Sidebar does, and
rebuilds on the four events that change its folders: `WorktreeAdded`, `WorktreeRemoved`,
`WorktreeOrderChanged` and `WorktreePathsChanged`. Every other project event is ignored, so a
busy project (diagnostics, language servers, scanned entries) never rebuilds the rail. The
rebuild is deferred, so it reads the `MultiWorkspace` after the project's group is rekeyed.

Rejected:
- a touchpoint that makes `MultiWorkspace::handle_project_group_key_change` notify on an empty
  key. It fixes only the last folder going, and changes Zed's own behavior to do it;
- a rebuild inside the handler, which runs while the project is in no group;
- following `WorktreePathsChanged` alone. It is safe only while the `MultiWorkspace`'s
  subscription runs before the rail's, which the order they are made in gives and nothing
  enforces, and a reorder never emits it.

## AD-claude-459-the-rails-cycle-actions-go-round-its-shown-rows-from-the-highlight-001
*decided at: 2026-09-23 · status: shipped*

Zed's Next and Previous Project and Thread reach the rail through `Sidebar::cycle_project` and
`cycle_thread`. The rail decides the target in `marley_rail` (`cycle_project`, `cycle_row`) and
opens it through `open_row`, the handler a click, Enter and the switcher use.
- The walk starts from the row the rail highlights (`selection`): the keyboard's row while the
  rail holds focus, else the row the window shows. Each action moves the highlight by one, as
  up and down do.
- It goes round the rows the rail shows, so a fold and the filter decide what it reaches, as
  in Zed's sidebar. Next and Previous Thread reach terminals and threads alike, as Zed's do.
- With nothing highlighted, Next goes to the first row and Previous to the last, as the rail's
  up and down keys do.

Rejected:
- starting from the Agent Panel's current thread while an editor has focus, as Zed does. The
  rail highlights a thread only while the panel holds focus, and the actions follow the
  highlight;
- unfolding the project Next or Previous Project reaches, as Zed's sidebar expands its target
  group. In the rail a folded project shows in the center all the same, and the fold is the
  user's choice;
- Zed's start at the first row in both directions when nothing is current.

## AD-claude-448-zeds-dylint-lints-are-errors-in-the-marley-crates-only-001
*decided at: 2026-09-23 · status: shipped*

gate:21 runs Zed's dylint library (`tooling/lints`) as upstream ships it, at its pinned
nightly, with the README's `cargo dylint --all`, over every Marley crate with all targets, in
every mode. Each Marley crate root (and `marley_terminal`'s integration test root) denies the
library's seven lints under the driver's `dylint_lib` cfg, so a hit in Marley code fails the
check, and the verdict is cargo's exit code (§0). Zed's crates keep the library's warn level,
Zed's own bar.

Rejected:
- `DYLINT_RUSTFLAGS="-D …"`: it would fail on the roughly 600 hits in Zed's crates;
- reading the check's JSON for Marley paths: §0 takes no verdict from a tool's output;
- bumping the library's nightly to the root's 1.98.1: the tree builds on the pinned one, and a
  bump would be a touchpoint and a `clippy_utils` pin to keep in step.

A lint the library adds warns in the Marley crates until the roots' lists name it.

## AD-claude-460-the-fork-starts-in-the-marley-layout-001
*decided at: 2026-09-23 · status: shipped · Chad's call*

`marley.layout` defaults to `marley`: `MarleyLayout`'s `#[default]` variant, where the Zed
default was, with `default.json` still free of a `marley` block. A user who writes `zed` keeps
Zed's layout, and a user who never chose moves to the Marley layout on update.

Zed's own `crates/zed` tests reach `marley_workbench::init` through `init_test_with_state` and
`initialize_workspace`, so they would take the new default: 24 of 93 failed without a pin.
`init_test_with_state` writes `zed` to the user settings just before `initialize_workspace`,
one hunk, so those tests keep testing upstream's layout and an upstream test merged later
needs no change of its own.

Rejected:
- a `marley` block in `default.json`, a second touchpoint for the same default;
- writing `marley` into a user's settings on first launch, which makes an absent key mean
  something other than the default;
- pinning the Zed layout test by test in `crates/zed`, which every upstream merge would have
  to repeat.

## AD-claude-461-upstream-crates-marley-changes-live-in-vendor-001
*decided at: 2026-09-23 · status: shipped · Chad's call on the location*

An upstream crate Marley has to change is copied into `vendor/<crate>` at the rev Zed pins, and
the root `Cargo.toml` points the build at the copy with a URL-keyed `[patch]` table.
`alacritty_terminal` is the first (#461), for the block terminal's event-loop hook (the
three-prong plan's D1).
- The copy stays out of Zed's workspace (`exclude = ["vendor"]`). As a member, `cargo fmt
  --all`, clippy, cargo-shear and dylint would hold upstream code to Zed's and Marley's bars,
  and alacritty's formatting settings are not Zed's.
- `vendor/` is Marley-owned for the ledger, and `vendor/README.md` records each copy's source,
  what is left out, and every Marley hunk, marked `Marley:` in the file.
- `vendor/` is in the receipt fingerprint.

Rejected:
- an `Ignibyte/alacritty` fork of Zed's fork: a second repository to manage, and a push for
  every change;
- a membership in Zed's workspace, for the reasons above;
- carrying `tests/ref`: 46 MB of recordings, and the only spelling hits the crate has.

## AD-claude-462-shell-hooks-leave-the-stream-in-the-event-loop-001
*decided at: 2026-09-23 · status: shipped*

Marley's shell hooks are taken out of the PTY stream in the vendored `alacritty_terminal`'s event
loop, under the terminal lock, by `marley_hooks::advance_with_hooks`. Each passthrough run is
parsed before the next hook's position is taken, so the hooks and the output keep their order
in a coalesced read. Each hook is sent as `Event::ShellHook` with a raw `HookPosition` (lines
evicted, history size, cursor line and column, alt screen), and `absolute_line()` counts from
the first line the grid held.
- The scanner is `marley_dcs`, a leaf crate, because `marley_terminal` depends on
  `alacritty_terminal`.
- Only Marley's selectors (`h`, `p`, `q`) leave the stream. Foreign DCS, cancelled frames and
  oversize payloads reach vte as they came, so the parser behaves as before for everything
  else.
- The grid counts evicted lines (`scroll_up` past a full history, `update_history`,
  `clear_history`), so absolute lines stay stable as the scrollback rolls. A reflowing resize
  is not covered (the plan's D2).
- Zed's `TerminalBackendEvent` mirrors the event and ignores it until #464.

Rejected:
- the filtering reader around the PTY (the plan's D1 fallback), which loses exact positions;
- a scanner inside the vendored crate, outside Marley's coverage and lint gates;
- taking every DCS out of the stream, as the gpui era's scanner did, which would change what
  vte sees if it ever handles DCS.

## AD-claude-464-blocks-are-anchored-ranges-read-from-the-grid-001
*decided at: 2026-09-23 · status: shipped*

Zed's `Terminal` keeps `marley_terminal::AnchoredBlocks`: each block records its command,
state, exit code, prompt, and three absolute lines (`prompt_line`, `output_start`,
`output_end`), the positions the vendored event loop reports with each hook. A block's output
is never stored. `Terminal::block_output` reads it from the grid on demand, mapping absolute
lines through `Grid::evicted_lines`, and returns `None` once the first line has left the
scrollback. The block keeps its metadata.
- `Precmd` finishes the running block where it falls and stages the next prompt, and `Preexec`
  opens a block whose output starts at its line. A `Preexec` with no `Precmd` before it
  finishes the running block without an exit code.
- Hooks on the alternate screen are skipped: that grid keeps no history.
- The model is new and pure, beside the gpui era's `SessionModel`, which copies output for its
  own engine.

Rejected: copying each block's output as the gpui era did. It doubles memory against the
scrollback, and stages the output twice.

## AD-claude-463-bash-loads-marleys-hooks-through-rcfile-prompt-command-and-ps0-001
*decided at: 2026-09-23 · status: shipped*

A local interactive bash that Zed spawns, given as `Shell::System` or `Shell::Program`, starts as
`bash --rcfile <data dir>/shell_integration/marley.bash`, with `MARLEY_SHELL_INTEGRATION=1`
set. The decision is one hunk in `TerminalBuilder::new`, gated by no task, not remote and a
PTY. The script is embedded in `marley_terminal` and written by the builder's background future
when its content changed.
- The script sources the user's `~/.bashrc` first, since `--rcfile` replaces it. System files
  still run.
- `__marley_precmd` goes first in `PROMPT_COMMAND`, as an array or a string, so it reads `$?`
  before anything else and returns it.
- `__marley_preexec` goes in `PS0`, which bash prints after reading a line and before running it,
  so the frame lands in stream order ahead of the output. It takes the line from history
  (`fc -ln -0`).
- A shell the user starts with arguments of their own is left alone.

Rejected:
- a `DEBUG` trap for preexec: it fires per simple command, and inside `PROMPT_COMMAND` too;
- the scripts under Zed's `assets/`, which would need ledger rows and carry Zed's license;
- writing the scripts on every spawn, or at startup for every shell.

## AD-claude-468-the-rail-draws-its-own-rows-after-warps-tab-list-001
*decided at: 2026-09-23 · status: shipped*

The rail's rows follow Warp's vertical tab list, which Chad chose as the reference
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`), and Marley draws them itself
in `rail.rs` instead of with Zed's `ListItem` and `ThreadItem`:
- `row_frame` gives every row a 1px border, clear unless selected, so moving the selection moves
  nothing; the selected row is a card, `ghost_element_selected` inside a `raised` border.
- `row_card` draws terminal and thread rows at `h_11` with a `size_7` round icon container,
  both in rems so they follow the UI font size, the title over an optional second line.
- `raised`, the theme's text color at 10%, fills the icon circle and draws the selected border: a
  step lighter than what it sits on in a dark theme, darker in a light one, as Warp's pane shows.
- A shell's icon is `>_` in the buffer font; an agent CLI's and a thread's are the agent's.
- A thread's second line is `<agent> · <status word>`, the voice of an agent CLI's row.
- A project header is a muted section label with its chevron, dot and `+`; a `Divider` runs
  above each project after the first row.

Rejected:
- `ListItem`: its outline is drawn only when set, so a border that follows the selection moves
  the row by a pixel, and its slots take no round container.
- `ThreadItem`: a fixed 16px icon slot and one line. It stays in the switcher.
- A new icon in Zed's `icons` crate for `>_`: a Zed path for what the buffer font already draws.
- Warp's `Ctrl <n>` hints: Marley binds no key to the n-th row.

## AD-claude-465-zsh-loads-marleys-hooks-through-a-zdotdir-that-hands-back-the-users-001
*decided at: 2026-09-23 · status: shipped*

A local interactive zsh that Zed spawns, given as `Shell::System` or `Shell::Program`, starts
unchanged but for its environment: `ZDOTDIR=<data dir>/shell_integration/zsh`,
`MARLEY_SHELL_INTEGRATION=1`, and `MARLEY_ZSH_ZDOTDIR` with the user's own `ZDOTDIR` when the
terminal's environment or Marley's has one. That directory holds only Marley's `.zshenv`
(`shell_integration/marley.zsh`):
- it restores the user's `ZDOTDIR`, or unsets it, and sources the user's `.zshenv`; zsh reads
  each later startup file from the `ZDOTDIR` of that moment, so `.zprofile`, `.zshrc` and
  `.zlogin` come from where they would have;
- in an interactive shell it appends an installer to `precmd_functions`, which at the first
  prompt puts `__marley_precmd` first and `__marley_preexec` last, and prints `init` and
  `bootstrapped`. First, so the frame marks where the command's output ends before another
  hook prints; zsh gives every hook the command's `$?` whatever the order.

Rejected:
- a Marley copy of each startup file that sources the user's: four files where one does;
- installing the hooks in `.zshenv` itself: whatever the user's `.zshrc` then did to the hook
  arrays (a framework that rebuilds them, a hook put first) would decide Marley's place;
- rewriting the `Shell` with arguments: zsh has no rcfile argument, and the title stays zsh's.

A user with no zsh startup files no longer gets zsh's new-user menu in Marley's terminals,
since Marley's `ZDOTDIR` holds a `.zshenv`.

## AD-claude-470-stage-one-draws-blocks-over-zeds-rows-001
*decided at: 2026-09-23 · status: shipped*

Stage one of the plan's D3 draws each block over Zed's own terminal rows, with no change to the
row model:
- `marley_terminal::visible_spans` (pure) maps blocks to a viewport's rows from the viewport's
  top as an absolute line, the frame of reference of the hooks' positions.
- `Content::marley_screen_top` (evicted lines plus history) carries that frame to the element;
  the top is it less `display_offset`, and a running block runs to the cursor's line.
- `TerminalElement` paints a faint wash (running `info`, failed `error`) after the cells'
  backgrounds, a two-pixel bar in the one-cell gutter Zed already leaves left of column 0, and a
  pill (a check, `exit N`, `running`) at the right end of a block's first row, an `AnyElement`
  laid out as Zed lays out its hyperlink tooltip.
- Nothing draws on the alternate screen: `term.grid()` is then the alternate grid, whose lines
  and evicted count are its own.

Rejected: header rows above each block and a hidden prompt (stage two, T5, which needs a
display-row map); a sticky pill for a block whose first row scrolled away (later); elapsed time
(blocks record none yet).

## AD-claude-473-the-block-keys-scroll-and-select-nothing-001
*decided at: 2026-09-23 · status: shipped*

The block keys scroll the focused terminal so a block's first line is at the top; they select
nothing, since stage one keeps no selected block. "Previous" and "next" count from the
viewport's top line (`marley_terminal::block_scroll`), and the next block on the live screen
means the live screen. The actions live in `marley_workbench`, caught at the workspace's root,
and change no Zed path; the Marley keymap binds `secondary-up` and `secondary-down` in
`Terminal`, Warp's keys, which Zed's defaults leave unbound. The handler syncs the terminal
before it reads, so every press moves one block. Rejected: handlers inside `TerminalView` (a
Zed hunk for what the workspace's root can catch); a selected block (with stage two, or when
hover actions need one).

## AD-claude-474-hover-actions-live-on-one-element-per-block-001
*decided at: 2026-09-23 · status: shipped*

Each block whose first row is on screen gets one element over its rows, laid out at that row in
`TerminalElement::prepaint` and painted after the text, with a hover group. Its first row holds
the pill and, while the pointer is anywhere over the block, Copy and Rerun: one element per
block, since gpui's group hover reaches only the group's descendants. Copy reads
`Terminal::block_output` when clicked. Rerun is drawn only while the last block is finished, so
the shell waits at its prompt, and only for a command that is not empty and verified
(AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001); it sends Ctrl-U, the
command and a carriage return, so a half-typed line cannot prefix it. Each button sits in a
wrapper that stops a left press from reaching the terminal, whose listeners run after it. The
release goes on: the button's click stops its own, and a press made elsewhere needs its release.
Rejected: `occlude` on the buttons (it ends the block's hover under the pointer, and the buttons
hide); stopping the release as well
(F-claude-474-a-press-elsewhere-lost-its-release-over-a-button-001); Rerun while a command runs
(it would type into that program); actions for a block whose first row has scrolled away
(later).

## AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001
*decided at: 2026-09-23 · status: shipped*

A hook frame is output: any program the terminal runs, or any file it prints, can emit one.
So each local terminal gives its program a random nonce in `MARLEY_SHELL_NONCE`
(`marley_terminal::shell_integration::new_nonce`, 128 bits), and its blocks are built with it
(`AnchoredBlocks::with_nonce`). Marley's bash and zsh scripts copy it into a shell variable and
unset it before the user's files run, so no program the shell starts inherits it, and add
`nonce=` to each `preexec` frame. A block records whether its frame carried the terminal's
nonce (`command_verified`), and only a verified command is offered Rerun. A frame without it
still makes a block: bars and pills only show what a frame claims. Remote terminals get no
nonce, since their program runs on another host. This follows VS Code's published shell
integration, whose reported command line carries a nonce from `VSCODE_NONCE`. Rejected:
checking the command against the text on the block's first row (prompts differ, and long or
multi-line commands wrap); Rerun that types the command without running it (it changes what
Rerun does); a nonce on `precmd` too (nothing acts on its fields yet; T2's path links will).

## AD-claude-476-the-content-sits-on-the-bottom-edge-by-moving-the-grids-origin-001
*decided at: 2026-09-23 · status: shipped*

A terminal whose live screen has room is drawn with its content against the bottom edge, the
way Warp pins its input to the bottom. It is a drawing change only: `TerminalElement::prepaint`
moves the grid's origin down by `marley_terminal::bottom_shift` rows once the content is synced,
and stores the moved bounds with `set_size`, which resizes nothing when the size is unchanged and
which the mouse maps through. The grid, the PTY's size, the scrollback and the block anchors stay.
Scrolled back by d rows, the shift is d rows smaller, so the history appears above the content.
The alternate screen, and views that are not standalone, are drawn as before. Rejected: shell
tricks (newlines at startup, a cursor move before each prompt), which change the grid and so
the lines blocks anchor to; Marley's own docked command editor, which is T3.

## AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001
*decided at: 2026-09-23 · status: shipped*

Anything Marley draws under a terminal goes through one hook in Zed's `TerminalView`: the global
`MarleyTerminalFooter`, a renderer that `render` calls with a `MarleyFooterContext` (the view's
weak handle, its terminal, project, workspace and focus handle). The view's root is a flex
column, so a footer takes its rows from the grid and the PTY is resized to the rows left. The
agent bar is Marley's code in `marley_workbench::agent_bar`, which sets the hook at `init`; it
shows while the foreground process is a known CLI agent, as Warp's toolbelt does. The branch
comes from Zed's git store, the innermost repository holding the agent's folder. Rejected: the
bar inside `terminal_view` (Marley's code in a Zed crate); an overlay on the terminal's last
rows (it would hide the agent's own footer); a bar for every terminal (a setting, if wanted).

## AD-claude-478-the-terminal-reads-the-notification-escapes-other-terminals-read-001
*decided at: 2026-09-23 · status: shipped*

Marley's terminal reads the desktop-notification escapes other terminals read, OSC 9 (iTerm2's)
and OSC 777 `notify` (rxvt's and Ghostty's), so any program can ask for a notification and
Claude Code can with its own channels or hooks. A scanner in `marley_dcs` watches the bytes the
vendored event loop hands the parser, which ignores both escapes, and reports
`Event::Notification`; Zed's `Terminal` passes it on as `Event::MarleyNotification`, and its view
marks itself as a bell does, without the sound. `marley_workbench::notifications` shows it through
gpui's `show_system_notification` unless the view is the focused terminal of the active window,
and a click shows that terminal. Rejected: OSC 99 for now (kitty's richer protocol); in-app
toasts; a notification for the focused terminal, which the user is already looking at.

## AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001
*decided at: 2026-09-23 · status: shipped*

Claude Code's `auto` channel does not recognize Marley's terminal, so Marley ships a Claude Code
plugin, installed from the agent bar's chip with Claude Code's own `claude plugin` commands from
a local marketplace Marley writes. Its `Notification` (`permission_prompt`, `idle_prompt`) and
`Stop` hooks run a script that answers with a `terminalSequence`, an OSC 777 notify Claude Code
writes to its terminal, only where `TERM_PROGRAM` is `zed`; fixed messages per event, so the
script parses nothing and needs no `jq`. Rejected: setting `preferredNotifChannel`, which is
global and would reach every terminal; hooks writing to `/dev/tty` themselves, which
`terminalSequence` does in step with Claude Code's own drawing. `terminalSequence` is in Claude
Code 2.1.281's hook schema but not its public docs: if notifications stop after an upgrade, check
that first.

## AD-claude-479-attach-file-types-paths-through-zeds-path-prompt-001
*decided at: 2026-09-23 · status: shipped*

Attach File types the chosen files' paths into the terminal as text, through
`TerminalView::add_paths_to_terminal`, the drop handler's own. That serves any CLI agent and any
shell, and a program that asked for bracketed paste gets the paths bracketed. The files come from
`Workspace::prompt_for_open_path`: the desktop's chooser by default, or Zed's own path prompt
when `use_system_path_prompts` is off or the project is remote. It lists through the project's
`DirectoryLister`, so a remote project's chooser shows the machine its terminals run on.
Rejected: a Marley picker over the project's files, because Zed's file finder cannot hand a path
back to its caller and a fuzzy picker belongs with rich input (#481); pasting a file's contents
instead of its path, because the agents read files themselves and a path keeps the prompt short.

## AD-claude-483-e2e-visualization-tests-replace-unit-tests-001
*decided at: 2026-09-23 · status: shipped*

Chad: "We are removing unit tests from the workflow entirely with instead doing e2e
visualization tests only". A ticket's proof is a scenario under `script/e2e/` that
`script/e2e.sh` runs against the real debug Marley on hidden workspace 9. The scenario brings
its own fixtures (a scratch repository, a HOME whose `.bashrc` it writes, fakes on the PATH),
sends keys to Marley's window only through Hyprland's `send_key_state`, and shoots each step.
The Test phase reads every shot, and the notes say what each shows. The tests already in the tree
stay and keep building (clippy `--all-targets`, and `cargo check --all-targets` for the vendor
copies), but no gate runs them; coverage, miri, the empty-suite gate and mutation testing
retired. Chad's choices, asked: keep the old tests rather than delete them; scripted live
captures rather than first building a Linux headless renderer for gpui (only macOS has one), so
clicks stay out of reach, since a synthetic click would move the user's pointer.

## AD-claude-480-marley-drives-voxtype-and-follows-its-status-001
*decided at: 2026-09-23 · status: shipped*

Voice input is Voxtype's, the dictation daemon Omarchy ships: Marley runs `voxtype record
toggle` from the agent bar's microphone or `marley::ToggleDictation`, and Voxtype records,
transcribes and types into the focused window. The microphone shows Voxtype's state from
`voxtype status --follow --format json`, the interface Voxtype documents for bars, which also
knows where a configured state file lives. The status is followed from the first frame that
draws a microphone, so no process runs before an agent does, and a dictation started from
Omarchy's keys shows too; an ended status restarts only on a toggle, never on a frame.
Rejected: capturing audio in Marley or a hosted transcription service (Warp's way), and
reading `$XDG_RUNTIME_DIR/voxtype/state` directly, whose path is Voxtype's configuration.

## AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001
*decided at: 2026-09-23 · status: shipped*

Marley's autosuggestions need what was typed at the shell's prompt without owning the line
editor. The typed text is read from the terminal's own cells: from the point where the first
key after the prompt was typed (the cursor when `Terminal::input` first ran after a `precmd`
hook) to the cursor, on the cursor's line, while nothing follows the cursor. That leaves the
user's prompt untouched and works for any shell with Marley's hooks. The history is the
terminal's verified commands, newest first, then the shell's history file, which the
integration names in a `history` frame. → is bound in `Terminal` to an action that types the
rest of the suggestion, or propagates so → stays the program's. Rejected: a prompt-end marker
appended to PS1 (FinalTerm's OSC 133;B), which has to follow every prompt framework that
rebuilds PS1 each time; reading readline's buffer through `bind -x`, which bash offers only
inside a bound command; reconstructing the line from the keys Marley sent, which history and
completion defeat.

## AD-claude-487-scenarios-that-click-run-in-a-headless-sway-001
*decided at: 2026-09-24 · status: shipped*

The e2e harness has two backends, chosen by the scenario: Hyprland (the default, keys to one
hidden window) and a headless sway of the run's own (`compositor sway`), whose seat is a
virtual pointer and keyboard that nothing else sees. The browser tab is driven by the mouse, and
Hyprland has no dispatcher that sends a pointer event to one window. Rejected: a nested
compositor in a window on the user's Hyprland (a window on the desktop, rules to add and
reload); `swaymsg seat cursor` (deprecated, and inert without a pointer device); a debug action
inside Marley that injects clicks (it would skip the platform layer a scenario exists to
exercise, and puts test code in the product); `ydotool` (uinput reaches every compositor,
the user's included). Scenarios proven on Hyprland stay there.

## AD-claude-488-marleys-browser-is-a-transient-unit-streamed-into-a-tab-001
*decided at: 2026-09-24 · status: shipped*

Marley's browser is a headless Chromium that Marley starts on first use as a transient user
unit, one per data directory, from the browser binary itself (never a distribution's launcher,
which would load the user's flags and extensions), with `--remote-debugging-port=0` and the
endpoint read from `DevToolsActivePort`. The Browser tab draws `Page.startScreencast` JPEG
frames (quality 85, acknowledged after decoding) as gpui images sized to the tab by the
device-metrics override, and one app-wide hub owns the connection, so the user and any agent
attached to the same Chromium share one page. The probe and #488's run confirmed the
screencast route (plan D12), so the CEF fallback stays unused. Rejected: an installed login
service (Chad chose "Marley starts it"); a fixed debugging port (profiles would collide, the
e2e run's with Chad's); a hub per tab (an agent's page and the user's would part).

## AD-claude-490-the-browser-tab-draws-its-own-chrome-and-dialogs-001
*decided at: 2026-09-25 · status: shipped*

The Browser tab's chrome is Marley's, over Chromium's Page domain: an address bar that is a
Zed single-line editor with three rules (a known scheme as typed; a host over `http` for
loopback and `https` otherwise; else a DuckDuckGo search), back and forward from
`Page.getNavigationHistory`, reload that turns into stop while the main frame loads, and the
URL a navigation goes to shown until it commits or ends. JavaScript dialogs, which headless
Chromium does not draw, are a card over the page in the tab (Zed's `AlertModal`), naming the
host that asks, and a navigation answers an open dialog with Cancel first, as Chrome closes a
page's dialog when the page is left. The browser's e2e scenarios run an offline Chromium
(`offline_chromium`: host-resolver rules that let only `localhost` and `127.0.0.1` through),
so a search or a typed name fails in the page and nothing leaves the machine. Rejected: a
window-wide modal for dialogs (the rest of Marley would wait on one page); leaving dialogs to
an agent or to CDP defaults (a page would hang with nothing on the screen); a search engine
setting now (the `marley` settings block has no browser section yet).

## AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001
*decided at: 2026-09-25 · status: shipped*

Marley starts `marley_mcp`'s server once per process, from `zed`'s `main` right after
`initialize_workspace` (Zed's tests run `initialize_workspace`, and a server started there would
write its endpoint over a running Marley's). It serves Streamable HTTP on 127.0.0.1 behind a
per-boot bearer, and writes `mcp-endpoint.json` (mode 0600, an MCP client's server entry) into
Marley's data directory, removed on quit. Tools whose answers are the app's come back from the
pure core as deferred calls, which the transport hands to the app with the server's lock
released and waits up to 30 seconds for. Wire names are `family_verb`; the first family served is
`terminal` (`terminal_list`, `terminal_blocks`, `terminal_read`) over every terminal in Marley's
windows. Claude Code reaches it through the Marley plugin's stdio bridge, which finds the
endpoint file, sends the bearer only to loopback, answers with no tools while Marley is closed
and says so when that changes. Rejected: a bare HTTP entry in the plugin (Claude Code would show
a failed server outside Marley); an environment variable for the endpoint (only Marley's own
terminals would have it); a `.mcp.json` in each project (it would spread a bearer into
repositories); answering tool calls on the transport's threads (the terminals are the main
thread's).

## AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001
*decided at: 2026-09-25 · status: shipped*

Agents reach the page in Marley's Browser tab through ten tools on Marley's MCP server, never a
second browser or page. The read tools are look (the frame as a JPEG image block, the viewport,
the focused element and the selection), an accessibility snapshot of the interactive elements
with refs across same-site frames and cross-site iframes, and rings of the newest 200 console
entries and requests, with secret-looking URL values hidden and no headers or bodies. The write
tools are navigate (http and https only), back, click and type by ref or point, press and
scroll, sent as the same CDP input events the user's hand makes, so the page sees trusted
events. Their grant class, `browser.write`, is granted when the server starts (the checks are
the client's approval of each call and the Agent chip in the tab the user watches), and an
agent's first write brings the Browser tab forward without taking the focus. No tool evaluates
script an agent supplies. Rejected: a script-evaluation tool; Playwright MCP's full tree by
default (about 11,900 tokens against about 3,400 for the interactive list); deny-by-default
writes behind a setting that does not exist yet.

## AD-claude-493-one-browser-tab-per-page-001
*decided at: 2026-09-25 · status: shipped*

Each page of Marley's Chromium is one Browser tab, a Zed item in Zed's own tab bar, and the hub
keeps each page's state by its target id. A page a page opens (`_blank`, `window.open`) opens
beside its opener's tab with the focus, as in a browser. A page an agent or any DevTools client
opens never takes the focus: it waits in the tab bar behind the tab in front of a pane with the
focus, and, when no Browser tab is open and the pane with the focus shows other work such as
the agent's terminal, it opens in a pane split beside that one. Closing a tab closes its page;
a move between panes does not; a page that closes elsewhere closes its tab. The agent tools act
on the page their `tab` names, from `browser_tabs`, or on the tab the user focused last.
Rejected: one Browser tab switching among pages (an agent's page would replace what the user
looks at); a tab strip drawn inside the Browser tab (Zed's tab bar already is one, with its
splitting and moving); bringing an agent's tab to the front of the focused pane (Zed's
focus-loss rule hands it the focus); a window of its own for agent pages.

## AD-claude-494-browser-tabs-reattach-or-reopen-001
*decided at: 2026-09-25 · status: shipped*

Browser tabs are Zed serializable items whose layout entry is the item alone; each tab's page
id, URL and title live in the tab's own table in Marley's database, per the #403 rule that no
URL enters a layout codec. At launch a restored tab claims its saved page id before the browser
reports its pages: when Marley's Chromium lived on, the tab takes its page back with everything
the page held; when it did not, the tab opens its saved URL in a new page. A start opens no page
and places no tab for the pages it finds, so which page belongs in which tab is the restored
tabs' to say, and a page no tab claims waits for `marley: open browser` or an agent. Rejected:
the URL in Zed's layout (the #403 rule); reloading every tab at launch (it would lose what a
live page holds); a tab for every page a start finds (a relaunch would open duplicates of the
restored tabs).

## AD-claude-495-marley-draws-the-pages-select-lists-001
*decided at: 2026-09-25 · status: shipped*

Marley draws a page's `<select>` lists, which headless Chromium opens out of sight. A listener
in an isolated world of every frame the hub watches (the page's, its same-site frames', and each
cross-site iframe's session) cancels the press or key that would open a single, enabled select
and reports it through a CDP binding; the Browser tab shows Zed's `ContextMenu` under it and
sets the choice in the same world, dispatching `input` and `change`. The list opens only within
a second of the user's own press or key in the page, so an agent's click opens nothing; the
agent keeps the page's keys. Rejected: a hit test (`DOM.getNodeForLocation`) before every
press, which puts a round trip in front of each click and misses a keyboard opening; letting
Chromium's invisible popup open, which swallows keys; opening the list for any click, which
would let an agent take the user's focus.

## AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001
*decided at: 2026-09-25 · status: shipped*

A pick in the Browser tab is staged, never sent on its own: it waits in a tray under the
toolbar with a caption field, and Send (or Enter in the caption) pastes one line,
`[browser pick N: <summary> on <host/path>; browser_pick id N] <caption>`, into the terminal the
user focused last, brings that terminal to the front and gives it the focus, without pressing
Enter. The line is the reference and `browser_pick` the content: the bundle (the nearest
interactive element, its ranked locators, role and name, listeners with their scripts, blockers,
box and crop) is read at the pick and kept in the hub for the session, so a pick outlives its
page, a browser restart and its row in the tray. Chromium draws the highlight (inspect mode);
Marley draws none. Rejected: sending a pick at once (open decision 3 leaned staged, and a pick
the user did not mean would reach the agent); guessing a terminal when none was used yet (the
tray says so instead); submitting the line (the user may add to it first); a popover for the
tray (it would cover the page the user is looking at).

## AD-claude-497-marley-reads-source-maps-and-finds-sources-in-the-worktrees-001
*decided at: 2026-09-25 · status: shipped*

Marley follows a pick's listeners through their scripts' source maps with a decoder of its own
(`marley_browser::source_map`: version 3, `sourceRoot`, one level of index-map sections, a base64
VLQ scan to the place asked for), since no source-map crate is in the tree and only the mappings
are needed. A map comes through the page (`Network.loadNetworkResource`) or from the script's
`data:` URL. A source is looked for only in the project's worktrees: an absolute source inside
one, else the longest suffix of its path, down to two components, that names a file there;
nothing outside the project is ever opened. Positions in the pick tools count from 1, as
editors do. Rejected: adding a source-map crate for one lookup per pick; reading the source from
the map's `sourcesContent` into a buffer (out of scope, and not the user's file); matching a
lone file name when the source names a path (a common name like `index.ts` would open the wrong
file).

## AD-claude-498-annotations-are-the-hubs-in-document-coordinates-001
*decided at: 2026-09-25 · status: shipped*

An annotation is a box in the document's CSS pixels with a note, a maker (the user or an agent)
and a time, kept per page by the hub for the session and dropped when the page shows another
document. gpui draws it over the frame, placed from that frame's metadata, and nothing is put
into the page. The user draws in an annotate mode whose drag replaces the page's press, move and
release; an agent draws through `browser_annotate` (a ref's border box, scrolled into view, or a
viewport area) and clears only its own. Rejected: drawing with a script or an overlay in the
page (the handoff's rule, and a page's own styles or scripts could hide or read it); storing
viewport coordinates (a box would slide off its content on the first scroll); keeping
annotations across launches before Chad decides open decision 4.

## AD-claude-499-the-flight-recorder-keeps-a-drawn-pages-minute-in-memory-001
*decided at: 2026-09-25 · status: shipped*

Each page a Browser tab draws keeps its last 60 seconds in memory: input as Marley sent it
(presses with their place, keys by name, typing as counts), the agent's Agent chip actions,
console entries, requests with redacted URLs, navigations, a snapshot after each load, and at
most two JPEG frames a second, 16 MiB at most. Nothing reaches the disk until the user saves
it with Record this, into `browser/recordings/<local time>/` under Marley's data directory,
never a project (the repository is public, and a recording holds what was on the screen).
Agents read recordings with `browser_recordings` and `browser_recording`. Rejected: recording
pages no tab draws (an agent's background work would fill memory unseen); keeping typed text
(a password is typed text); a video file (the frames already are the JPEGs Chromium sent);
pruning by age (a recording stays until removed by hand, as the spec's scope says).

## AD-claude-501-marleys-server-is-a-default-context-server-001
*decided at: 2026-09-25 · status: shipped*

Marley offers its MCP server to Zed's own agents as the context server `marley` among Zed's
default settings, a stdio server running Marley's bridge with `MARLEY_MCP_ENDPOINT`, added once
the server runs. The Zed Agent's Write profile lists its tools and every external agent of the
Agent Panel is handed it at `session/new`, so each can drive the Browser tab as Claude Code in a
terminal does through the plugin. Rejected: an HTTP entry with the URL and the bearer (the bearer
would sit in the settings the Settings window shows, and change at every start); writing the
entry into the user's settings file (Marley's state would land in a file the user owns).

## AD-claude-502-the-installed-marley-is-a-copy-and-a-launcher-001
*decided at: 2026-09-25 · status: shipped*

`just install` (`script/install-marley`) installs the release `marley` as a copy in
`~/.local/lib/marley/marley`, replaced by a rename, and `~/.local/bin/marley` as a launcher that
appends stderr to Marley's `logs/stderr.log` when stderr is not a terminal, then `exec`s the
binary; `marley.desktop` names the launcher. The `dev` channel stays until #445, so the installed
and the debug builds share their settings and data and must not run at once. Rejected: a link
into the target directory (gone with a `cargo clean`); Zed's `install-linux` (builds `zed` and
`cli`, names the app Zed Dev, links `~/.local/bin/zed` over Chad's Zed); `release-fast` (full
debug info, no LTO); stripping (loses a backtrace's lines to save disk that is not short); a
panic hook in Zed's `main.rs` (a Zed hunk for what a launcher does outside Zed).

## AD-claude-516-redact-at-the-tool-boundary-on-by-default-001
*decided at: 2026-09-25 · status: shipped*

Marley hides secrets in what its MCP tools hand an agent, at the tool boundary
(`terminal_blocks`, `terminal_read`, `browser_console`), never in the terminal's buffer or the
console's log: what Chad sees stays exact. It is on by default (`marley.redact_secrets_for_agents`,
unlike Warp's opt-in), since a model is always on the other end, and it names each kind
(`[redacted: github token]`) so the agent knows something was there. The rules favour hiding: a
secret-named variable loses any value (`TOKEN_LIMIT=5`), and a private key block with no END line
is hidden to the end. The user's patterns add to the rules and never replace them; one that does
not compile is left out and named in a notification. Rejected: redacting in `marley_terminal`
for everything it stores (changes what the user sees and copies); Orca's rule that hides every
`NAME=value` line (an `env` dump would lose `PATH` and every harmless value).

## AD-claude-513-one-marley-per-data-directory-001
*decided at: 2026-09-26 · status: shipped*

Marley runs Zed's single-instance check on the dev channel too, per data directory: its socket
is `<data dir>/zed-dev.sock`, so `--user-data-dir` still gives a second Marley. A launch that
finds a Marley running hands it its paths through Zed's own socket and URLs (`file://` for a
path, made absolute against the launch's working directory; `zed://open` when there is none) and
exits, since Marley ships no CLI; a data directory whose socket path does not fit a Unix socket
starts without the check. Rejected: a second socket and protocol of Marley's own (Zed's already
opens paths and focuses the app); focusing through `hyprctl` from the launcher (ties the fix to
one compositor; gpui's activation works where the compositor allows it); forwarding the
launcher's activation token now (a gpui change; ticketed).

## AD-claude-517-the-golden-set-gates-the-install-001
*decided at: 2026-09-26 · status: shipped*

Marley's regression suite is a golden set of its own e2e scenarios (`script/e2e/golden`), each
ending in machine checks through Marley's MCP server, the stand-in agent and the files the run
writes (`expect`, `holds` in `script/e2e.sh`), so a run passes or fails with no one reading its
shots. `script/regress` runs the set one scenario after another, every one in a headless sway
(`COMPOSITOR=sway` also moves the key-only scenarios off the user's Hyprland), and `just install`
runs it against the release build it made before anything is replaced; `--skip-regress` is the
way out. Rejected: comparing shots against saved ones (fragile across themes and fonts, and the
checks say what broke); running the set in `script/gates.sh --diff` now (choosing scenarios by the
files a change touches is slice 2, once the set's run time is known: about five minutes today);
testing the debug build before an install (the install would test a build it does not install).

## AD-claude-544-anchors-cross-a-rewrap-as-logical-places-001
*decided at: 2026-09-26 · status: shipped*

The blocks' anchors stay absolute lines, and a width change carries them across alacritty's
rewrap as places the rewrap keeps: logical lines (a row and the rows its `WRAPLINE` continues
into) from the cursor's logical line, and a character offset inside their own. The pivot is the
cursor, which alacritty keeps inside its logical line, not the grid's top, which a full history
cuts. The logic is pure (`marley_terminal::anchored::RowsView`, `AnchoredBlocks::rewrap`); Zed's
Resize arm only reads the main screen's rows before and after `resize` (`marley_rows_view`,
through the vendored `Term::main_grid`, since the main screen is rewrapped while a full-screen
program shows too). Rejected: the plan's OSC 8 tag on each prompt (re-finds only the prompt row,
needs a non-blank cell, and Zed underlines and opens hyperlinks); recomputing anchors from the top
of the grid (a full history cuts rows there). Accepted limits: a hook positioned before a resize
and applied after it, and the shell's own prompt redraw on SIGWINCH, can leave the newest prompt's
anchors off until the next prompt.

## AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001
*decided at: 2026-09-26 · status: shipped (slice 1; #547 publishes the snapshot)*

A terminal's Claude Code reports its hook events in band: Marley's plugin answers each hook with a
`terminalSequence`, an OSC 777 notify titled `marley-event` whose body is the base64 of a JSON
summary under 2,900 bytes, so the frame reaches the terminal it belongs to (over SSH too) with no
endpoint, token or terminal id to pass. The terminal's existing notification path carries it
(`Event::MarleyNotification`); Zed's view leaves that title unmarked and the workbench routes it
to `agent_events` instead of the desktop. A frame counts only while Claude Code is the terminal's
foreground program, and it is display data only: anything that turns an event into input must
check authenticity first. The state lives in `marley_fleet` (one `FleetSnapshot`, a seat per
terminal view), folded by the pure `marley_agent::claude_events::fold` from the previous seat and
the event, with Orca's rules (a permission wait ends only with the tool it asked for; subagents
move a count; harness-injected prompts keep the user's). Rejected: a POST to `marley_mcp` from the
hook (an endpoint file and a bearer in every terminal, and no route from a remote host); a new
`terminal::Event` variant (it breaks every exhaustive match outside the terminal crates,
L-claude-478); inferring state from output alone (the 2 s quiet timer stays the fallback for a
terminal that sends no events).

## AD-claude-535-agent-events-reach-the-phone-as-one-line-through-ntfy-on-the-box-001
*decided at: 2026-09-26 · status: shipped (the phone's hand check waits on the box's ntfy)*

When Claude Code in a terminal the user is not looking at needs input, finishes or fails, Marley
posts one line (`<project>: Claude <event>`) to an ntfy server on the same machine, which
`tailscale serve` publishes to the phone; the ntfy app shows it. Only that line, a title, a
priority and a tag go out, and for iOS only a message id and a topic hash pass through ntfy.sh's
poll relay; nothing the agent wrote travels. The trigger is the seat change #519's fold gives
(`TurnEvent::of_change`), gated as desktop banners are (`notifications::looking_at`), with a
5-second cooldown per project. The URL must be loopback and a token comes from a 0600 file, read
off the main thread. Rejected: a push gateway of Marley's own or APNs (an Apple developer account,
and the text in plaintext at the gateway, as Orca's is); sending the agent's words (they stay on
the desktop until Chad widens it); pushing every OSC notification (only Claude Code's events).

## AD-claude-550-the-close-guard-asks-in-zeds-own-close-paths-and-holds-the-view-001
*decided at: 2026-09-26 · status: shipped*

A close or a quit that would end a working agent asks first, through one global hook,
`workspace::MarleyCloseGuard`, called where Zed's own close paths start: `Pane::close_items` for
a tab (the rail's Close and `ctrl-shift-w` reach it), `prepare_windows_to_quit` once per quit,
`prepare_window_to_close` once per window, and `prepare_to_close` for a replace. Each hook hands
over the items, so the guard reads only terminal views and never an entity being updated.
"Working" is the seat's state for Claude Code (a turn, a permission or a question in flight),
else the quiet timer's. A working terminal closed from its tab is held: the guard keeps the
`TerminalView` itself (the pane's items list is its only strong owner), and `Pane::add_item`
puts the same view back, scrollback, seat and PTY intact, as a drag between panes does.
Rejected: `on_app_quit` (it cannot cancel) and `on_window_should_close` (Zed's own always
vetoes and closes later); an `Item` method (none can veto a close); asking in `prepare_to_close`
alone (a quit or a window with agents in two projects would ask twice); holding the `Terminal`
without its view (the view carries the scrollback's rendering and the seat's key).

## AD-claude-520-each-terminal-names-itself-and-the-bridge-names-the-caller-001
*decided at: 2026-09-26 · status: shipped (slice 1; #574 and #575 follow)*

Each local interactive terminal starts with `MARLEY_TERMINAL_ID`, a UUID Zed's terminal builder
mints beside #474's nonce (so a split, rebuilt through the builder, gets its own), and
`MARLEY_PROJECT`, the project's folder, which the project sets; a task, a remote terminal and a
local terminal of a remote project get both emptied, since the program inherits Marley's own
environment under the builder's map. The Claude Code plugin's bridge sends them, with its own
folder, as `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` on each request; the transport
keeps well-formed values as a `Caller` on each `AppCall`. The terminal tools default to the
caller's terminal. The id scopes defaults and is no authority: the bearer gates every call.
Rejected: the gpui entity id (it changes every launch, and no program sees it); guessing the
caller from the running command (#491's stand-in did, and two agents in two terminals look
alike); a column in Zed's `terminals` table for the restore (#575 keeps its own table, so no
Marley migration sits in upstream's list).

## AD-claude-574-browser-tools-default-to-the-callers-project-001
*decided at: 2026-09-26 · status: shipped*

A browser tool that names no tab acts in the caller's project: the project group of the caller's
terminal, else of the local workspace one of whose own folders holds its `Marley-Project`, else
its `Marley-Cwd`, the longest folder winning. It acts on the page of that project's tab the user
focused last (the hub keeps its focus history, each page once, 64 at most), else its newest. With
none, `browser_navigate` opens a page whose tab goes to the caller's own workspace, through a
placement the hub keeps by page id until the tab opens, placed as AD-claude-493 places an
agent's page, and the other tools refuse, naming `browser_navigate`. A named tab is the caller's
choice and is not refused. A caller in no project keeps the tab the user focused last anywhere.
This narrows AD-claude-493's default, the tab the user focused last, to the caller's project.
Rejected: acting on another project's tab when the caller's has none (the failure this ticket
stops); a one-shot "next page" placement (a page opened meanwhile would take it); the group's
last active workspace as the new tab's home (a linked worktree's agent would get its tab in the
main checkout's workspace); refusing a named tab of another project (#507's per-project contexts
are where projects part).
