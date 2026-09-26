# Failures — the local ledger

> Exported 2026-08-09 from the forge DB (table `failures`; 205 entries,
> TICKET-409 — the scrap-forge pivot). This file is the LIVE capture surface:
> `/pipeline:inspect` APPENDS new `## <code>` blocks (`/pipeline:complete` catches up);
> recall is `grep`. One entry per `## <code>` heading — never edit history.

## BF-372-getstatus-gate-strict-decode
*category: runtime · severity: low*

#372 inspect F1: the one-time GET-status gate in run_once decoded the WHOLE socket buffer strictly (str::from_utf8(&buffer).unwrap_or("")) before parsing the status line — an invalid UTF-8 byte anywhere in the buffer (a body byte, a multibyte char split at the 4KB boundary) would discard the entire buffer including the ASCII status line, so a non-2xx standing-GET response would slip past the gate and the pump would idle on it forever — the EXACT latent gap #372 exists to close. Self-inconsistent with the valid-prefix decode 7 lines below in the same loop. Fixed by decoding the valid UTF-8 prefix (buffer[..valid]); the ASCII status line always sits in the valid prefix. Found by 2 of 3 inspect critics.

## BF-373-reconnect-rebuilds-empty-fleetsync
*category: runtime · severity: medium*

#373 inspect F1: the auto-reconnect loop's run_once rebuilt a fresh EMPTY FleetSync on every connection cycle. Because the proposed fleet://events?since=<cursor> contract is a post-cursor DELTA (returns only events after the durable cursor), a reconnect would apply only the newly-arrived events onto an empty snapshot — DROPPING every seat introduced before the cursor. The reducer's max-join makes re-delivery idempotent (kills dupes) but cannot recover a seat whose event is never re-sent, so on every transient forge bounce the fleet rail would collapse to just-the-delta — the exact opposite of the ticket's "Marley down != fleet down / no gap" promise. Masked in tests only because the fixture served cumulative pages. Fix: carry ONE FleetSync across the thread's whole life (run_subscription seeds it from the persisted cursor once and passes &mut sync into every run_once) so a reconnect resumes from the held cursor AND retains the accumulated snapshot; losslessness then holds for both a cumulative-since and a delta-since forge. Guarded by a reworked REQ-005 that serves a strict delta (PAGE_C) on reconnect and asserts the snapshot stays {a,b,c}; empirically confirmed the test fails (delivers {c}) without the fix. Found by the security/state-integrity critic.

## BF-375-session-cap-no-reclamation
*category: security · severity: medium*

#375 inspect F2: the per-session Mcp-Session-Id registry shipped with a reject-new cap (8) but NO reclamation path except an explicit client DELETE — no TTL, no disconnect-reap. Because the MCP spec permits (and Marley's own #373 self-healing pump does) re-initialize-on-reconnect WITHOUT sending DELETE, 8 cumulative reconnects would orphan 8 sessions and permanently wedge the global cap → every future initialize returns 503 until the app restarts, defeating the exact self-healing the sibling tickets built. A bounded pool with a reject-at-cap policy but no reclamation is a self-DoS footgun. Fixed by reaping a session when its standing GET SSE stream drops (thread the validated session id into serve_sse_stream, terminate on the write-failure/hang-up path) — tying session lifetime to the client's presence (the natural MCP model), so a reconnect frees its old slot and the cap self-heals. Found by the protocol-lifecycle critic via a cross-ticket cross-check against the #373 pump's reconnect behavior.

## BF-agentrow-field-missed-test-literals-001
*category: compilation · severity: high*

#187 added a required field `exit: Option<(i32,u64)>` to AgentRow. I updated the AgentRow struct-literal constructions in agent_rows_sorted_by_pane_id + the two agent_badge/AgentRun sites, but MISSED the two literals in agent_row_text_full and agent_row_text_bare — so the marley crate TEST target failed to compile (E0063 missing field), which would block every downstream gate (tests/mutation/commit). cargo check (non-test) was clean, so it was invisible until a critic ran `cargo check --tests`. Caught by both inspect critics; fixed by adding `exit: None` to both. Root cause: relying on the obvious/central test site instead of grepping ALL `AgentRow {` construction sites.

## BF-cfg-gated-fn-unkillable-mutant-001
*category: test · severity: high*

A platform-conditional value written as two `#[cfg(...)]`-gated FUNCTIONS — `#[cfg(not(windows))] fn host_flavor()->Posix` / `#[cfg(windows)] fn host_flavor()->Windows` — produces an UNKILLABLE mutant on a single-platform (macOS) CI runner. cargo-mutants parses the source syntactically (cfg-agnostic) and generates `replace host_flavor -> PathFlavor with Default::default()` for BOTH fns. The not(windows) one fails to build (no Default) → unviable (safe). The cfg(windows) one is cfg'd-out on macOS, so the mutated line never compiles → the build is identical to baseline → tests pass → the mutant is MISSED/survives. No macOS test can execute cfg(windows) code, so it is unkillable → MSI capped at ~95% → gate:5 RED, unfixable by any test. Caught by the inspect mutation critic (verified via `cargo mutants --list` + a full run). Fix: replace the two cfg-gated fns with two `#[cfg]`-gated `const HOST_FLAVOR: PathFlavor`. cargo-mutants does not mutate const items and a const is not an executable coverage line. Verified: `cargo mutants --list` now shows 0 host_flavor mutants.

## BF-changelog-hook-worktree-vs-staged-001
*category: security · severity: high*

enforce-changelog.sh (a commit-content gate, mirroring enforce-commit-gate) decided from `git status --porcelain` — the WORKTREE — instead of the staged index. The worktree includes untracked (??) and unstaged ( M) entries that are NOT part of a `git commit`, so any CHANGELOG.md merely present on disk satisfied the gate. Because CHANGELOG.md was itself untracked at that moment, the gate was LIVE-inert: every crates/*/src/*.rs commit was exempt for free. A subdir CHANGELOG.md decoy also passed (the match regex wasn't root-anchored). Caught by the inspect bypass critic (verified by running the hook against synthetic staged states). Fixed: gate on `git diff --cached --name-only` (the staged index = the commit), anchor the required file to `^CHANGELOG\.md$` (repo root), scope the trigger to `^crates/[^/]+/(src|examples)/.*\.rs$`. Verified: staged .rs + untracked CHANGELOG → exit 2; + staged CHANGELOG → exit 0; subdir CHANGELOG decoy → exit 2.

## BF-claude-200-ghost-accept-omits-history-detach-001
*category: runtime · severity: medium*

The #200 ghost-accept (Key::Right at EOL inserting the suggestion suffix via buffer.edit) mutated the prompt buffer WITHOUT calling `state.history.detach()`, unlike every other buffer-editing path (the R36 contract — app.rs detaches on Char/Backspace/DeleteForward with the comment "an edit (not a motion) leaves history navigation"). Repro: mid-recall (↑ to "git", history cursor Some(0)), accept the ghost → buffer "git push" but cursor STILL Some(0); a subsequent ↑ → recall_prev clamps to entries[0]="git" and the accepted "git push" is DISCARDED + never stashed (the draft is only stashed on the None→newest transition). A genuine R36 contract violation. Caught by the inspect critic (I checked the accept's borrow/return but not its parity with the detach-on-edit contract). Fixed by adding `state.history.detach();` in the accept branch, mirroring the Char/Backspace path. Pre-ship.

## BF-claude-200-ghost-renders-on-unfocused-panes-001
*category: runtime · severity: medium*

The #200 inline-history ghost-text render condition (caret==char-count && !completion_open) was NOT gated on `is_focused`, yet it lives inside the per-pane render loop (`for (pane_id, r) in &rect_list; let is_focused = pane_id == focused`). So every split pane at EOL with a matching history rendered its own muted ghost — but the → ACCEPT is focused-only (`focused_terminal_mut()`), so → could never accept a non-focused pane's ghost (a render/accept divergence across PANES) + it diverges from Warp (autosuggest is active-input only). Caught by the inspect critic; my self-review checked TIME divergence (same-frame render vs the → event) but missed the MULTI-PANE case. Marley already had the precedent: the #186 find-highlight is explicitly focused-only ("would bleed onto a non-focused pane"). Fixed by adding `is_focused &&` to the ghost condition. Pre-ship (inspect→validate), no shipped bug.

## BF-claude-361-cov-excluded-fn-still-mutated-msi-red-001
*category: test · severity: medium*

#361's --diff gate went RED at gate:5 (mutation): 3 survivors, all in app.rs::font_is_monospace (replace→true, replace→false, delete match arm). Root cause: coverage-exclusion (the llvm-cov `--ignore-filename-regex` that skips app.rs for gate:4) is a SEPARATE mechanism from mutation-exclusion — cargo-mutants mutates every fn in the diff regardless of the cov ignore-regex, unless the fn carries `#[mutants::skip]`. font_is_monospace is an app-side gpui metric-read shim with no unit-testable seam (the decision it delegates to, is_monospace_advance, IS unit-tested), so it needs the explicit skip — exactly like its sibling font_resolves, which already carried `#[cfg_attr(test, mutants::skip)]`. I added the cov-exclusion mentally (app.rs is in the regex) but forgot the fn-level mutants::skip. Fixed by adding `#[cfg_attr(test, mutants::skip)]` to font_is_monospace; verified via `cargo mutants --list` that both probes show no mutants and the settings.rs decision mutants remain (all killed); re-run → GATE GREEN. Lesson: an integration-only shim needs BOTH the file-level cov exclusion AND the fn-level mutants::skip.

## BF-claude-361-headless-drives-unreachable-under-noop-text-system-001
*category: test · severity: high*

#361 Phase 3 shipped two `#[gpui::test]` headless drives (font_family_proportional_falls_back_to_builtin_headless / font_family_monospace_applies_silently_headless) that assert the RESOLVABLE-font monospace path (Helvetica→fallback+"not monospace" flash; Monaco→applied+silent). They CANNOT pass: `#[gpui::test]` uses gpui's NoopTextSystem, under which `all_font_names()` reports nothing resolvable, so a real family reads UNRESOLVABLE → the boot wire's `!resolves` short-circuit fires and `font_is_monospace` is never called → the not-monospace arm is unreachable headless (and Noop's synthetic advance makes every glyph equal-width, so it couldn't distinguish mono from proportional even if reached). The #344 garbage-drive comment already documented exactly this ("The RESOLVABLE case is not headless-testable here (Noop resolves nothing) — the pure `resolve_font_family(resolves=true)` unit covers it"). Caught by inspect Critic 2 (HIGH). Root cause: at design I read the #344 headless drive's CODE but not its COMMENT's rationale, so I assumed real macOS fonts (Helvetica/Monaco) would behave under the headless harness — they don't; `#[gpui::test]` is NoopTextSystem, not CoreText. Fix: removed both drives; the not-monospace DECISION is proven by the pure units (settings.rs, cov-included); the app-side metric read (font_is_monospace) is cov-excluded + headed-only, exactly like #344's font_resolves.

## BF-claude-361-is-monospace-guard-equivalent-mutant-001
*category: test · severity: medium*

#361's `is_monospace_advance` guard `if wide_w <= 0.0 { return true }` yields an EQUIVALENT (surviving) cargo-mutants mutant `<= → ==`: it only diverges from the original at `wide_w < 0`, and there the fall-through computes `(≥0).abs() / (<0) ≤ 0 ≤ ε` which ALSO returns true — so no input distinguishes them → the mutant is MISSED → MSI < 100 (a hard-gate blocker). Caught by inspect Critic 1 (MEDIUM). Fix: restructured to a POSITIVE gate `if wide_w > 0.0 { ratio } else { true }` — the boundary now lands at `wide_w == 0` (where the else returns true but the ratio branch would divide-by-zero), so the `>`-vs-`>=` mutant is killed by a `(narrow, 0.0)` test and `>`-vs-`==`/`<` by a normal `(3.0, 8.0)` test; all killed by the already-planned Phase-4 units. Note `!(wide_w > 0.0)` also kills it but trips clippy's `neg_cmp_op_on_partial_ord` — the if/else positive form is both mutant-complete AND clippy-clean, and preserves the conservative bad-measurement→monospace default.

## BF-claude-362-vacuous-headless-drive-opens-here-masks-the-fix-001
*category: test · severity: high*

#362's Phase-3 headless drive (auto_close_suppresses_quote_in_type_parameters_headless) was VACUOUS — it passed on the PRE-fix code too, testing nothing. Fixture `fn f<T: >() {}`, caret at byte 8 = the position of `>`, so `next = '>'`. `pair_action` only reaches InsertPair when `opens_here(next)` is true, and `opens_here('>')` = false (`>` is neither a closer in PAIRS nor whitespace). So `'` inserts BARE regardless of context — the `>` blocks the pair, NOT the #362 LifetimeBound. Reverting the entire #362 app-closure branch leaves the drive green. Caught by inspect Critic 2 (HIGH); my own inspection had confirmed the enclosing-fn mutants::skip but did NOT trace opens_here on the fixture. Root cause: I chose the incremental-typing position `fn f<T: |>` intuitively, but that exact position is where the trailing `>` already suppresses via opens_here — the one place #362 is NOT needed. Fix: a DOUBLE-space fixture `fn f<T:  >() {}` with the caret between the two spaces (byte 8) → `next = ' '` (whitespace → opens_here true) so a Code context WOULD pair `''`; only LifetimeBound suppresses it → bare `'`. Asserted `!bound.contains("''")` + the exact bare-`'` string. Added the double-space byte to the probe's true-unit. Verified non-vacuous by tracing the pre-fix path (Code → InsertPair → `''` → the assert fails). LESSON: a headless drive for an auto-close/suppression feature must verify the BASELINE would produce the OTHER outcome — trace opens_here/next at the fixture caret, or the test can pass for the wrong reason (a different guard masks the one under test).

## BF-claude-a-test-can-encode-a-bug-and-then-defend-it-001
*category: test · severity: critical*

Marley #296→#297. SelectionSet::from_selections merged ANY two members whose ranges touched (`next.start() <= cur.end()`). I justified the `<=` in a doc comment AND pinned it with a test whose stated rationale was "otherwise two abutting selections would each insert at the seam and double up". THE RATIONALE IS FALSE: two abutting ranges replace two DISJOINT spans, and the shipped back-to-front sweep applies them with zero interference (`[0..3]`+`[3..6]` over "abcdef" with "X" must yield "XX"; it yielded "X"). The cost surfaced one ticket later: ⌘⌥↓ (add cursor below) followed by ⇧↓ (extend down) produces two line-spanning ranges that abut EXACTLY at the shared line-start offset — so the flagship multi-cursor gesture destroyed its own second cursor on the very next keypress, and because a motion is not an edit, ⌘Z could not bring it back. The gate was green throughout: 100% line coverage, MSI 100, a 50k-step differential fuzzer, and 1066 passing tests — because a TEST ASSERTED THE BUGGY BEHAVIOR and thereby defended it. Found by an adversarial critic, reproduced by me. FIX: a caret-aware merge (`<=` only when either side is a caret — same-offset carets are one cursor and must collapse; two RANGES merge only on a TRUE overlap, `<`), the pinning test replaced with its inverse, and the companion `selections_after_multi_edit` switched from raw `from_members` to canonicalizing `from_selections` (once touching ranges survive, a DELETE over them emits two carets at one offset).

## BF-claude-aa-passing-palette-shipped-two-identical-slots-001
*category: validation · severity: high*

#316: the tuned Light SyntaxPalette shipped `keyword` = hsla(0.52,0.58,0.34) and `property` = hsla(0.52,0.50,0.34) — IDENTICAL hue AND lightness, 0.08 saturation apart = perceptually the same dark teal. Both cleared WCAG AA (4.51:1 / 4.81:1 vs the near-white bg), so the `default_palettes_clear_aa` contrast-matrix test was BLIND to it — a keyword and a field/JSON-key rendered indistinguishably in the default Light theme (two of the most frequent token kinds). Root cause: keyword was pinned to the Light accent (0.52/0.58/0.34) and property was placed at the same accent hue+L. A parallel Dark defect: `type` hsla(0.12,0.50,0.65) ≈ `number` hsla(0.09,0.55,0.62) (both amber, dH=0.03/dL=0.03, saturation did not rescue). The class: a CONTRAST test proves legibility-vs-background but says NOTHING about slot-vs-slot distinctness — two colors can both pass AA and still be the same color. Caught by an adversarial grammar-map/WCAG critic that COMPUTED the pairwise hue/lightness/saturation deltas (the AA matrix + my self-review missed it; the design had flagged min-hue-distance as an inspect edge). Fix: moved Light property to a distinct violet hsla(0.70,0.50,0.40) (8.9:1), Dark number to a red-amber hsla(0.04,0.60,0.62), nudged Light keyword L 0.34→0.32 for AA headroom (4.51→4.98), and added a `tuned_palettes_have_distinct_slots` gate (collision iff close on hue [circular] AND lightness AND saturation).

## BF-claude-additive-gesture-followed-the-primary-so-the-new-cursor-was-invisible-001
*category: runtime · severity: high*

#298: ⌘D added a cursor the user could not see, and the viewport never moved. `follow_editor_caret()` reads `active_caret()` → `primary().head()` → member 0 — the TOPMOST cursor — and an ADDITIVE gesture NEVER moves member 0 (from_selections SORTS, so a downward add appends BELOW the primary). So ⌘D scrolled to a row already on screen; the new cursor 300 rows down stayed invisible; the user saw nothing happen, pressed ⌘D again, accumulated off-screen cursors, and then typed into text they had never looked at. A REGRESSION against #272, whose ⌘D set a SINGLE selection — the new match WAS the primary, so following it worked. Making the set multi-member silently broke that guarantee while the one-line call site looked unchanged. Mirror bug: ⌘⇧L makes the primary the FIRST match in the file, so it YANKED the viewport to the top of the document, away from the user's work. AND: the comment I wrote directly above the call ("#270: a wrapping ⌘D can land the new cursor off-screen — follow it") asserts the behavior the code does NOT have — it names the ONE case (the wrap) that accidentally works, because there the new cursor happens to BECOME member 0. Third occurrence of the write-a-comment-then-violate-it class in three consecutive tickets. FIX: a pure `added_member(before, after)` seam; ⌘D scrolls to the cursor it actually created; ⌘⇧L does not scroll at all.

## BF-claude-adopted-inherits-overlay-query-shipped-alone-blanks-the-language-001
*category: runtime · severity: critical*

#315: `Lang::TypeScript`/`Lang::Tsx` compiled `tree_sitter_typescript::HIGHLIGHTS_QUERY` ALONE, but that const is the `inherits: ecma` OVERLAY only (5 captures: keyword[TS-only words]/punctuation.bracket/type/type.builtin/variable.parameter) — no string/comment/number/function and none of the common ECMAScript keywords, which live in `tree_sitter_javascript::HIGHLIGHT_QUERY`. Because grammar_lang(TS)=Some, the render takes the tree path and never consults the hand-lexer floor → .ts/.tsx showed ONLY type annotations (blank strings/comments/keywords) — WORSE than the pre-ticket floor. A naive per-grammar "closed capture set" pin test PASSES while the language is visibly broken (it pins the 5 overlay captures). Found only by the grammar-map critic reading the actual .scm files + running each language through a probe; self-review + 3 other critics missed it. Fix: concatenate the JS base query (+ JSX for TSX) ahead of the TS overlay — the standard tree-sitter `inherits` resolution — `format!("{JS}\n{TS}")`; verified it compiles against LANGUAGE_TYPESCRIPT/LANGUAGE_TSX and now colors strings/comments/numbers/keywords/functions.

## BF-claude-alacritty-pty-drop-discards-output-plus-exitcode-opacity-001
*category: runtime · severity: high*

marley_spike inspect found 2 CRITICAL latent bugs (both empirically proven by a critic; check+clippy were green and hid them): (1) PTY-DROP DISCARDS OUTPUT — term_io::spawn_pty built an alacritty_terminal::tty::Pty, cloned its master fd, and RETURNED only the clone while the Pty was dropped at end of fn. alacritty's `Pty::drop` does kill(SIGHUP)+child.wait() then closes the master, which DISCARDS the unread kernel PTY output queue; the surviving try_clone'd fd then reads immediate Ok(0) → empty grid. Proven: a drop-before-read probe failed 50/50 (always immediate Ok(0)); a hold-the-Pty-across-read probe passed 50/50. The planned R3/R4/R5 integration test would have failed + the headed window would render a BLANK body. Fix: spawn_pty returns the live Pty in the tuple; app.rs + the integration test bind it across the drive_reader read loop and drop it AFTER. (2) OPAQUE ExitCode UNKILLABLE MUTANT — exit_code_for(ShutdownOutcome)->std::process::ExitCode: ExitCode impls Default (=SUCCESS) so cargo-mutants' `-> Default::default()` is VIABLE, but ExitCode has no PartialEq/accessor so no unit test can observe it → the mutant survives → MSI<100, unfixable by a test. Fix: split a pure `exit_status_code(o)->u8` (testable: ==0 / !=0) + a 1-line `exit_code_for = ExitCode::from(exit_status_code(o))` marked mutants::skip. ALSO: the implement step ran clippy but not `cargo fmt --check`, so a non-canonical gpui closure failed gate:1 fmt (caught at inspect).

## BF-claude-answer-race-no-question-identity-001
*category: runtime · severity: high*

#377 inspect F1 (MED, found independently by BOTH critics from different lenses): the answer path carried no question identity at THREE layers — (a) the chip listener captured only (seat_id, choice), so a question swapped between paint and click dispatched the old choice against the new question AND self-concealed (the record stored the re-fetched new question, reading current forever); (b) the result channel was tagged by seat id only, so a superseded send's late Ok could overwrite a newer pick's Failed and lock the seat behind a false "answered"; (c) the wire args {session_id, choice} gave the brain nothing to refuse a stale in-flight answer with. Fixed: rendered-Question capture + exact-equality dispatch belt; a per-pick nonce keying the drain apply; prompt added to the proposed wire contract. The general class: an async answer to a mutable question needs the QUESTION'S identity carried at every hop (render→dispatch→channel→wire), not just the subject's id.

## BF-claude-approximated-the-gate-command-and-reported-green-001
*category: compilation · severity: high*

M29 #403 Phase 3: I recorded "cargo clippy --workspace --all-targets clean" in the pipeline notes. It was false. gates.sh:373 runs `cargo clippy --workspace --all-targets -- -D warnings`; I ran it WITHOUT `-D warnings` and filtered the output through `grep -E "^(error|warning: unused)"`, a pattern that does not match how rustc words dead code ("warning: method `X` is never used"). The tree was red on gate:2 the whole time — a `#[cfg(test)]` test hook with no caller yet — and /commit could not have passed. Separately, a `cargo check --workspace --all-targets` pass was read as "no existing test broke", but `cargo test` was never run, so a genuinely broken test (section_items_tables, pinned to the ＋ menu row tables I changed 3→4) sat red and unnoticed through the whole of Phase 3. Both were caught by inspect critics against the very tree the notes described.

## BF-claude-audit-doc-line-refs-stale-after-same-ticket-edits
*category: validation · severity: low*

#232 authored docs/marley_architecture/icon-audit.md (a catalogue with ~25 file:line refs) at DESIGN via a background Explore sweep, THEN the implement phase edited app.rs (adding a const + 3 Assets arms + 2 svg blocks + 2 items_center), shifting every cited app.rs line by +7..+25. An inspect critic flagged the whole 'site' column as stale (MED). Non-app.rs refs stayed accurate (those files untouched). Fixed by regenerating all app.rs line numbers via a fresh grep (not hand-math) + a drift note. Root cause: a reference doc's line numbers were captured before same-ticket edits to the referenced file.

## BF-claude-auto-reveal-bypassed-by-direct-scroll-jumps-001
*category: runtime · severity: high*

#305 code folding: auto-reveal (unfold the caret's containing fold on navigation) was hooked ONLY in follow_editor_caret (via reveal_caret_row). But the JUMP commands — go-to-line (goto_commit), symbol-jump (jump_to_file_symbol), go-to-def/CmdT/Ctrl-minus/diagnostics-jump (open_and_place_caret to consume_pending_center), and FIND (CmdG, select_efind_current) — place the caret then call scroll_editor_to_row DIRECTLY, bypassing follow_editor_caret. scroll_editor_to_row does slot_of(row) (which snaps a hidden row to the folded header's slot) + scroll, with NO reveal — so a jump INTO a folded body centers the folded header while the caret sits invisibly inside the fold, and typing then edits hidden text. This is REQ-006's own scenario. Both inspect critics + the implementer's independent trace confirmed it; the find (select_efind_current) site was caught ONLY by the implementer's trace (a critic missed it). The design's D-AUTO-REVEAL-ON-CARET misidentified follow_editor_caret as THE shared chokepoint — the jumps funnel through scroll_editor_to_row, which cannot blanket-reveal because goto_preview scrolls with the caret parked at the origin. Fix: a reveal_and_scroll_to_row(row) helper (remove_folds_at_row(path,row,hidden_only=true) + scroll_editor_to_row(row)) wired at the 4 in-scope bypass sites; goto_preview + CmdD (out of REQ-006 scope) left. Would have shipped GREEN if Phase 4's REQ-006 drive reached for an arrow motion (which reveals) instead of a named jump verb.

## BF-claude-blocking-child-reap-in-drop-on-main-thread-001
*category: performance · severity: medium*

TICKET-023: closing a pane dropped TerminalSession inline on the gpui main thread; alacritty's Pty::drop SIGHUPs then BLOCKS in child.wait() with no timeout — MEASURED 603ms UI freeze closing a pane running a foreground child, and unbounded (force-quit territory) for a HUP-immune child (trap '' HUP); window-close hung the same way. Fix: Workspace::close(pane) RETURNS the removed PaneState so the CALLER owns teardown context; the app drops it on a detached reaper thread; PtyChannel gained a Send supertrait so TerminalSession moves cross-thread (compiler-verified).

## BF-claude-boot-restore-force-seeds-a-terminal-after-the-runtime-guard-dissolved-001
*category: runtime · severity: medium*

#395 REQ-003 (restore empty) was unmet: the boot restore loop (app.rs:2101) force-seeded a terminal onto ANY restored project with no terminal tab ("A project must keep ≥1 terminal tab (the workspace() invariant)"), so a persisted EMPTY project restored with tab_count 1, not 0. But that ≥1-terminal invariant is exactly what #391/#392 DISSOLVED at RUNTIME (the try_workspace/try_active_tab twins make the app total over a no-terminal/zero-tab project). The runtime tests (close the last tab → tab_count 0) all passed — the gap was ONLY in the BOOT path, which still enforced the old invariant. Caught at validate by a NEW headless test (boot_restores_a_zero_tab_project_empty) that seeded a zero-tab shell blob and asserted tab_count 0 (got 1). Fixed at source: added Project::empty(name, root) (the Workspace::empty analog) + relaxed the restore loop (Some(first_tab)→Project::new as before; None→Project::empty, restoring empty); the unused boot PTY reaps at the existing terminal-less arm. Triple-proven (the project_empty unit + the boot_restores headless + the live pixel capture). LESSON: dissolving a RUNTIME invariant must include auditing the BOOT/restore path that ALSO enforced it — runtime tests won't catch a boot-only enforcement.

## BF-claude-boot-root-resolved-before-restore-001
*category: runtime · severity: high*

#376 inspect F2 (HIGH): the fleet subscription start-gate ran at the boot position where project_root is the launch-cwd DISCOVERED root — but the shell restore ~700 lines later explicitly re-syncs project_root to the restored ACTIVE project. Consequences: a Finder/Dock launch (cwd=/ or $HOME) never subscribes even though the restored workspace IS configured (the common non-dev launch); launching from configured project A with restored-active B renders A's live feed under B; a launcher boot spawns a subscription thread for an unopened workspace. Fix: moved the start block BELOW the shell build, gated on project_count() > 0, keyed shell.active_project().root. The launch-cwd root is PROVISIONAL until the restore.

## BF-claude-bracket-match-no-language-gate-parses-non-rust-as-rust-001
*category: validation · severity: medium*

#340 bracket-match: `refresh_bracket_match` (app.rs) had NO language gate, so it called `marley_syntax::matching_delimiters_in` — which ALWAYS parses `tree_sitter_rust::LANGUAGE` — for ANY editor buffer. A `.json`/`.toml`/`.py` file would show a bracket tint + ⌘⇧\ jump driven by RUST grammar semantics (a `(` in a Python `#` comment false-matches because Rust has no `#` comment). Violated the spec's "Rust-only v1" Scope and REQ-008 ("render nothing / do no walk when no tree exists"), and made the shim's own doc comments ("a non-Rust file yields None") false. Root cause: the Phase-2 design premise "non-Rust files fall out for free" was a CLAIM, not a fact — it assumed no tree for non-Rust, but the pure fn is language-specific by construction and always builds a Rust tree. Caught by an inspect critic parsing JSON as Rust → `Some((0..1,12..13))`. Fixed by mirroring `refresh_syntax_cache`'s `is_rust` gate at the caller (the seam that knows the file's language); drop the cache like the no-editor branch when `!is_rust`.

## BF-claude-branch-discriminator-asked-about-member-0-not-the-whole-set-001
*category: runtime · severity: high*

#298: `add_next_occurrence` chose its first-press branch with `primary().is_caret()`, using it as a stand-in for "is this the first press". It is NOT the same question. `primary()` is member 0 — the TOPMOST — and #297's ⌘-click can put a bare caret ABOVE an existing range. Two HIGH bugs fell out of that one substitution: (1) ⌘-click in leading indentation → that wordless caret is member 0 → the set (which already HAS a needle) goes back down the word-select branch → `word_range_at` returns None → `unwrap_or(sel)` leaves it a caret → the set comes back BYTE-IDENTICAL. Every press. ⌘D is dead for the session with no feedback and no way out but Esc. (2) If that ⌘-clicked caret DOES have a word under it, expanding it MERGES into the range beside it — 2 cursors become 1, a cursor the user explicitly placed is destroyed, and the needle silently changes from "bar" to "foobar" so every subsequent ⌘D searches for different text. Both are motions, so ⌘Z cannot undo them. The doc comment claimed the mixed shape was handled ("a caret with no word under it stays a caret, so the fn is TOTAL") — it was total, but not PROGRESSING; totality was the wrong property to check. FIX: discriminate on the WHOLE SET (all members are carets ⇒ first press), and take the needle from the topmost RANGE, not from member 0.

## BF-claude-broad-container-capture-plus-outer-wins-sweep-steals-the-value-001
*category: runtime · severity: high*

#315: adopting tree-sitter-toml-ng's highlights.scm mis-colored every `key = value`. The query captures `(pair (bare_key)) @property` over the WHOLE pair node; our span sweep (`sweep_disjoint`) is outer-wins-clip-forward (sort by (start,end), keep first, clip the rest forward), whereas tree-sitter's highlight convention is innermost-wins. So the broad `(0..pair_end, Property)` span survives and the narrower value captures (Number/Str) get clipped away → `foo = 1` rendered key=Type, `1`=Property (REQ-003 inverted) — and a REGRESSION vs the hand-lexer that colored TOML correctly. Root class: an adopted query with broad container captures is incompatible with an outer-wins sweep. Fix (v1): revert grammar_lang(Toml)=None → TOML stays on the correct hand-lexer floor; Lang::Toml + the pinned grammar retained. Proper fix (follow-up): an innermost-wins sweep — but that also changes Rust `(attribute_item) @attribute` whole-attribute absorption, so it needs its own ticket + Rust-attribute fixtures. Also surfaced a latent LOW: JSON object keys are double-captured (`string.special.key`+`string`) at an identical range, so which wins rested on `sort_unstable`'s "may reorder equal elements" → switched to a stable `sort_by_key` (query order → Property deterministic).

## BF-claude-buffer-version-restarts-so-path-version-cache-key-goes-stale
*category: validation · severity: medium*

#268 inspect (MED): the syntax memo was keyed (path, BufferVersion) — but BufferVersion is per-Buffer monotonic FROM ZERO, and EditorSurface open/close mints fresh buffers. Reopening a file the terminal agent rewrote on disk (Marley's core loop) hit (path, v0) == cached (path, v0) → STALE highlight spans rendered until the first keystroke; two projects holding the same path with equal edit counts collided the same way. Probe-verified: spans clamp, no panic — silently wrong colors. Fix: a process-monotonic buffer nonce (AtomicU64 minted in OpenFile::new) in the key — (nonce, version) can never collide across buffer lifetimes.

## BF-claude-caller-census-grepped-wrapper-names-not-the-callee-001
*category: validation · severity: medium*

#400 design enumerated the cockpit open doors by grepping WRAPPER spellings (open_cockpit_section, SectionAction::OpenCockpit) and concluded "two doors". A third production caller — the "toggle-right-dock" verb (⌘⇧B) — called the callee (Project::open_or_switch_cockpit) directly and was missed; the signature change made it a compile error, so the compiler caught it at implement, but a semantic-only change (e.g. adding the acquire without changing arity) would have shipped an unbalanced view-count leak through that door. Same family as the #399 static-id census miss: a census built from a PARTIAL naming pattern reads as complete.

## BF-claude-caret-map-used-for-code-span-end-swallows-phantom-001
*category: runtime · severity: high*

#331 inlay hints. Injecting phantom text made ONE column map serve two incompatible boundary semantics. `col_of_offset(i)` is the CARET map: it deliberately returns the column AFTER a phantom anchored at i (the caret sits on the code side of a hint). `raw_span_to_display_bytes` and the #310 diagnostic bar both built their END boundary from it — so a code span whose one-past-end char is a hint anchor EXTENDED ACROSS the phantom. `styled_slices_with_marks` resolves a slice to the FIRST containing range and hints were appended LAST, so the code span won and the hint painted as code. Reachable on the PRIMARY tree-sitter path via rust-analyzer chaining hints (default ON, always EOL-anchored, and a literal DOES have a syntax span ending exactly there): `let n = "hello"` → Str 8..15 mapped to display 8..20 → the hint rendered string-green. Same root cause on the squiggle: a warning on `x` in `let x = 1;` underlined `x: i32`. Found INDEPENDENTLY by two critics from opposite ends (one from the column math, one from reachability). Fix: a separate END-side map (`col_ends` + `col_of_span_end`) for the spans that describe CODE; the selection band deliberately KEEPS the caret map, because it must end where the caret it follows renders.

## BF-claude-clamp-branch-mutants-survive-non-distinguishing-tests
*category: test · severity: medium*

#237's pure EditorSurface::close had a 3-branch active-index clamp (active > idx → -1; active == idx == last → len-1; else unchanged) + activate's `idx < len` guard. The 3 unit tests exercised the code but left 4 mutants alive (MSI ≈ 82.6%, not the required 100): close's `>`→`==`/`<`/`>=` (the test only ever closed at the LAST index, where the decrement branch and the >=-clamp branch COINCIDE — same value → operator swaps undetected) + activate's `<`→`<=` (the test used activate(5) on a 2-file surface, but `5<2`==`5<=2`==false → the boundary idx==len was never hit). Behaviorally correct code, but the strict mutation gate on the pure seam was RED. An inspect critic ran cargo mutants + gave the discriminating cases. Fixed: added a 4-file `active > idx` case, an active-MIDDLE case (active==idx, not last), and an `activate(len)` exact-boundary case → 0 missed, MSI 100.</description>

## BF-claude-clamp-card-text-passed-bidi-format-chars-001
*category: security · severity: medium*

The shipped clamp_card_text (#377 F2) filtered only c.is_control() — Unicode general-category Cc — so bidi FORMAT characters (Cf: U+202A–202E embeds/overrides, U+2066–2069 isolates) passed through to gpui rendering. Core Text honors bidi controls, so hostile text (server-controlled #377 answer / #378 dispatch failure reasons, or a hostile settings value reaching the #384 misconfiguration caption) could visually REORDER what renders — a Trojan-source display spoof where the displayed string reads differently from the stored one (e.g. a rejected URL displaying as a different URL). Found by the #384 inspect security critic while tracing the new reason path through the shared helper; fixed at the helper (one filter widening hardens all four consumers) + pinned by a unit feeding embeds/overrides/isolates and asserting the stripped output.

## BF-claude-clamp-into-invariant-type-without-recanonicalizing-001
*category: runtime · severity: high*

Marley #296 (multi-cursor core). Buffer::set_selection defensively CLAMPED each Selection to the rope length and then rebuilt the SelectionSet with the raw in-crate `from_members` — which does NOT re-merge. Clamping is NOT injective: two distinct out-of-bounds cursors (10 and 20 in a 3-char rope) both clamp onto offset 3 and SURVIVED AS DUPLICATES, violating the type's ORDERED+DISJOINT invariant. edit_at_selections then applied the edit at both duplicate members: buffer "abc" + carets {10,20} + "X" produced "abcXX" — one visible cursor, two inserts. A GREEN gate (100% line coverage + MSI 100) and a 50k-step differential fuzzer both missed it, because every test fed sets that had already been canonicalized by from_selections. Found by an adversarial inspect critic. FIX: set_selection re-canonicalizes after clamping — SelectionSet::from_selections(clamped) — plus a regression test asserting the duplicates collapse and the replacement is typed ONCE.

## BF-claude-clamp-removal-doc-named-the-wrong-hazard-001
*category: validation · severity: medium*

#307 widened indent_edits/dedent_edits from a contiguous (first,last) range to rows: &[usize] and DELETED the `last.min(len_lines-1)` clamp. The doc comment justified the deletion by saying the empty-line filter drops an out-of-range row "before Buffer::line_start is ever asked about it" — framing line_start as the hazard being avoided. Reading line_start showed it opens with `let row = row.min(self.len_lines().saturating_sub(1));`: it CLAMPS, it never panics. So the safety argument named the wrong failure mode. The code was correct; the reasoning was not, and the real hazard is strictly worse than the one described — an unfiltered out-of-range row would resolve to the LAST line's start offset and silently indent or dedent the wrong line, with no crash to notice. Fixed by rewriting both fns' docs to state the actual hazard (silent wrong-line edit) and why no redundant bounds guard is added. Same class as the M22 "CHECK THE REASON, not just the fix" lesson: a plausible justification terminates review before anyone measures it.

## BF-claude-click-col-quantized-before-nearest-scan
*category: validation · severity: medium*

#277 inspect C-F2 (probe-verified): all three editor click sites rounded x/cell_w to an INTEGER column before the nearest-boundary scan — on a 2-cell wide glyph (boundaries 0 and 2) a click at 0.5 cells rounds to col 1, an exact tie resolved LATER, so the caret flip point sat at 25% of the glyph instead of its midpoint. Quantize-then-scan discards the half-cell the tie rule needs. Fix: keep the click column in the FLOAT domain until the final compare — offset_of_col_f(f32) with un-quantized distances (same later-wins tie), offset_for_click takes f32, click sites pass rel/cell_w unrounded. The integer wrapper went dead in the lib target and was deleted (no dead delegates), tests ported.

## BF-claude-clippy-reversed-empty-ranges-on-inverted-test-literals
*category: validation · severity: low*

#266 P4 gate --diff went RED on clippy's deny-by-default reversed_empty_ranges: two DELIBERATE inverted-range test literals (styled_slices' `Some(7..3)` inverted-selection case and cols_to_bytes' expected `10..1` inverted output) read to clippy as empty-iteration bugs. Also rustfmt RED on the fresh headless-test block (fmt not run before the gate). Fixed at source, no suppression: the inverted ranges are test DATA, so they are spelled as `std::ops::Range { start, end }` struct literals, which the lint correctly ignores; re-run GREEN 15/15.

## BF-claude-cmd-d-exhausted-guard-false-fired-after-the-wrap-001
*category: runtime · severity: high*

#298 ⌘D (add-next-occurrence) died one occurrence short, PERMANENTLY, and told the user it was finished. The scan resumed at `set.last().end()` and treated an already-held match as proof of exhaustion. But `SelectionSet::from_selections` SORTS, so `last()` is the bottom-most in DOCUMENT order — once the wrap added a cursor at the TOP, the resume offset stayed pinned at the BOTTOM forever. Every later press re-found that same held top match and early-returned. Every occurrence BETWEEN the wrapped match and the user's starting point was unreachable for the rest of the session. Fires whenever ⌘D starts on anything but the FIRST occurrence — the ordinary case (you spot the symbol mid-file). The user then types, and the skipped occurrence silently keeps the old text: a wrong edit that looks like a completed rename. Worse trigger: a single ⌘-click at end-of-file (or ⌘⌥↓) puts a bare CARET at `last()`, dragging the scan origin past every occurrence in the document — ⌘D dead on the first press. The guard had a passing test and a confident doc comment ("Once every occurrence is selected it is a NO-OP, by decision"); the premise — "the wrap returned a match we hold ⇒ we hold them all" — is simply false once the set can hold more than one member. Found independently by THREE of four critics with the gate fully green (1089 tests, clippy clean). FIX: step OVER held matches instead of bailing at the first; resume from the bottom-most RANGE, never a bare caret.

## BF-claude-coalesced-pty-read-passthrough-before-hooks-drops-output-001
*category: runtime · severity: high*

marley_terminal inspect: pump/ingest rendered the WHOLE read's passthrough bytes (via DcsScanner::feed's separate `out` param) BEFORE applying that read's decoded DCS hooks. PTYs COALESCE writes, so a fast command's `[Preexec]output[Precmd]` routinely fits one ≤4096B read. Processing order was: render `output` while the block isn't open yet → set_current_output is a no-op (current() is None) → OUTPUT DROPPED; then apply Preexec → reset term + open an EMPTY block; then Precmd → finish it → a Finished block with output_text()=="". Breaks R4 and makes the R16 `echo hi` integration test FLAKY (passes only when reads happen to split between preexec and output — exactly the timing-dependent blank-Block race the spike's no-backoff bug warned of, but ORDERING-flavored: the bytes and the hooks lost their interleaving when the scanner flattened them into two separate channels. Fix: DcsScanner::feed emits an ORDERED Vec<DcsEvent{Passthrough(bytes)|Hook(raw)}> preserving byte-stream order; ingest processes in order so Preexec resets+opens the block BEFORE the following passthrough (the command output) is rendered into it. Caught by an inspect critic tracing a coalesced read, NOT by check/clippy (it compiles fine).

## BF-claude-compose-buffer-cleared-before-confirmed-delivery-drops-input-001
*category: runtime · severity: low*

#72's first cut cleared the composed prompt line unconditionally whenever last_agent was Some — including when state_mut(target) returned None (the last-launched agent's pane had been closed; close-pane removes from self.agents but never clears self.last_agent). Repro: launch an agent (last_agent=X) → close pane X → without relaunching, compose a line → cmd-shift-s. The write is skipped (target gone) but the line is still wiped — user input vanishes with nothing delivered and no feedback. Memory-safe (the critic-brief called it a 'harmless no-op') but NOT harmless: it silently drops user input. Invisible to the gate (shim: mutants::skip + cov-excluded) — only manual inspect caught it. FIX: clear the compose buffer ONLY after a confirmed send — `let sent = match state_mut(target) { Some(s) => s.session.write_bytes(&payload).is_ok(), None => false }; if sent { clear }`. A missing target OR a failed PTY write now preserves the line.

## BF-claude-copy-superset-of-highlight-header-rows-untinted
*category: validation · severity: medium*

#44 cmd-C: content_row_texts made command-HEADER rows copyable (block.command is a content row), and selected_text slices every row in the selection incl. headers — but #43's highlight painted ONLY output rows (the header div had no selection_bg, just a hover tint). So a drag spanning from one block's output into the next block copied the intervening COMMAND text while that row showed no highlight — copy became a SUPERSET of the highlight, violating the spec's stated invariant "selected_text reuses row_selection so copy matches the highlight exactly" (D1/REQ-002). The pure fns were correct; the SHIM render was inconsistent (highlight incomplete). Caught by the inspect critic cross-reading the render (output-only tint) against content_row_texts (headers copyable). Fix: tint the command-header row via row_selection too (mirror the output branch) so highlight == copy — the right direction since Warp includes commands in a multi-block copy. Lesson: when a copy/serialize surface and a visual-highlight surface are meant to be "the same selection", an asymmetry in WHICH rows each covers is invisible to per-fn unit tests + MSI (both sides pass in isolation) — the invariant only breaks at the shim seam; verify the two surfaces enumerate the SAME row set.

## BF-claude-covered-left-anchor-collapses-cant-carry-whole-file-caret-001
*category: runtime · severity: medium*

#314 formatting: the caret was captured as a Bias::Left Anchor before applying the format edits, to carry it through the reformat (REQ-003). But rust-analyzer returns ONE whole-file TextEdit (edit(0..len, new)), which COVERS the caret's anchor, and rebase_offset collapses a COVERED Left-bias anchor to the span start (0). So for the actual (whole-file) reformat shape, the anchor resolves to offset 0 → the caret+viewport teleport to line 1 every format — the exact bug the anchor was added to prevent. The anchor only rebases correctly for GRANULAR edits (which don't cover the caret), but the spec itself said granular edits carry the caret for free — so the anchor was redundant where it worked and ineffective where it was needed. Proven by anchor.rs::covered_collapses_by_bias (covered+Left → span start) + a real-Buffer throwaway. Found by an adversarial critic; my self-review + the design + the code comment all wrongly assumed the anchor carried the whole-file case. Fix: a (line, col) re-seat — capture line_col(caret) before apply, re-seat via caret_for_line_col(row+1, col+1) after (clamps into the reformatted line), keeping the caret on its logical line best-effort.

## BF-claude-dead-defensive-clamp-equivalent-mutant-001
*category: test · severity: low*

marley_editor inspect: movement.rs move_word_left/right opened with a leading `let mut i = off.as_usize().min(len)` defensive input-clamp. Because the caller-contract guarantees an in-range offset (set_selection clamps; every movement fn returns in-range for in-range input), that `.min(len)` is IDENTITY on every contract-valid input — i.e. DEAD on the only tested path. cargo-mutants' `remove .min` / `min→max` mutant is therefore EQUIVALENT/unkillable unless a test passes an OUT-OF-RANGE offset (which the in-range contract says never happens), so it would silently block MSI 100 (or force a misuse-path test). Compounding it, the module doc overpromised "Every function returns a CharOffset in [0,len_chars]" while the sibling move_char_left returned an out-of-range value and move_line_* PANIC on an out-of-range input — a doc-vs-code mismatch + inter-sibling inconsistency. Not a runtime bug (correct for all in-range inputs). Caught by an inspect critic tracing out-of-range behavior across the 6 movement siblings + a buffer/types critic enumerating cargo-mutants viability. Fixed: removed the dead `.min(len)` (move_word_* now assume in-range like the other four fns), and softened the module doc to state the in-range caller-contract explicitly (consistent with Buffer::edit's §14 out-of-range = caller-contract-violation model).

## BF-claude-decoder-unescape-before-split-nullifies-emit-escaping-001
*category: validation · severity: high*

TICKET-022 design premise "emit-side fix only; decode already ships AnsiCQuoted" was empirically false: marley_terminal::decode_hook ran c_unescape over the WHOLE payload before the naive `;`/`=` field split, so the emitter's `\;` collapsed back to a bare `;` pre-split — `ls; pwd` still truncated to `ls` and a `;`-containing $PWD could inject phantom prompt fields (`…\;git=evil` → git_branch "evil"). NO emit-side encoding could ever survive that decode order (Hex has the same shape). Caught at inspect by two independent critics who ran the REAL decoder against real-zsh-emitted payloads (not just code review). Fix: R24 — split the still-escaped payload on unescaped separators first (backslash-consumes-next scanners), then c_unescape per piece; Hex/Plain byte-identical; all 69 pre-existing tests pass unmodified.

## BF-claude-deferred-save-fires-against-active-not-origin-editor-001
*category: runtime · severity: high*

#314 format-on-save: the deferred save (parked while the format round-trips) completed via do_plain_save to save_active, which writes the CURRENTLY-ACTIVE editor. If the user pressed CmdS on editor A (format armed) then switched to tab B before the format answered (50ms-2s round-trip), every completion path (apply success, settle on Err/stale, the deadline) saved B (or nothing, if B is a terminal) instead of A. Net: A was SILENTLY never saved (the latch consumed, A still dirty), and the deadline even flashed 'timed out — saved' misattributed to B. A lost save = the one unforgivable outcome (D-SAVE-NEVER-BLOCKED). Root cause: an async completion re-read 'the active editor' at completion time instead of binding to the ORIGIN editor captured at request time. Found by an adversarial critic (my own trace confirmed one-write for the single-editor case but MISSED the cross-editor switch). Fix (bounded, keeps D-ONE-WRITE): active_is_origin(key) gates every completion's do_plain_save on the active editor still being the origin (has_path(path_from_file_uri(key.uri))); a switch abandons cleanly (no wrong-editor save, origin stays visibly dirty). The proper origin-targeted save (a non-active-editor save path) is blocked on save_active's active-only #275/#284 conflict machinery — filed as a follow-up.

## BF-claude-deferred-scroll-to-item-survives-a-file-transition
*category: runtime · severity: medium*

#273 inspect (self-review, critic-confirmed): gpui's UniformListScrollHandle applies a pending deferred_scroll_to_item AFTER the base offset at the next layout and it WINS (uniform_list.rs:393-448) — so a scroll_to_item minted for file A that is still pending when the render transition switches the shared handle to file B teleports B to A's row. Fixed: the per-file scroll-sync transition clears deferred_scroll_to_item in the same borrow_mut that restores the incoming offset. Residual constraint (doc'd on the helper): an open-file+jump-to-row single action must defer the jump a frame or park the row on the OpenFile — the transition clear cannot distinguish a fresh incoming deferred from a stale outgoing one.

## BF-claude-details-status-line-keyed-off-exit-code-not-status
*category: runtime · severity: medium*

exit_status_kind(Finished, None) = Failure → the ✗ glyph, but the shim's `status_line = match d.exit_code { None => "running" }` read only the exit_code, so a signal-killed / no-code-captured Finished block rendered ✗ (failed) beside the word "running" — a glyph/text contradiction. Uncaught by every gate (app.rs is coverage-excluded + render is mutants::skip) AND by the self-test (which drove a clean exit-0 command). The inspect critic found it via a probe on the Finished+None case. Fix: derive the line from BOTH status and exit_code — `(_, Some(c)) => "exit {c}", (Running, None) => "running", (_, None) => "no exit code"`. Lesson: when two rendered elements (a glyph and its text label) describe the same state, derive BOTH from the SAME source (the status class), never from a lower-level field one of them ignores — else they can disagree on an edge case no test drives.

## BF-claude-diagnostics-single-terminal-scope-001
*category: runtime · severity: medium*

The #291 LIVE DRIVE surfaced a #289-era scoping limitation invisible to unit tests: `open_file_diagnostic_rows` reads only `workspace()`'s terminal grid, which via `terminal_grid_index()` is the ACTIVE tab's grid (if a terminal) else the FIRST terminal tab's — so with the editor active + multiple terminal tabs, a failed block in a non-first terminal tab is never scanned and the gutter stays dark. Pure tests (trace_diagnostic_rows) + #289's env-blocked validation both PASSED because they never exercised the cross-tab enumeration in the shim; only driving the real multi-tab app exposed it. A single-terminal workspace works. Filed as follow-up ticket #295. Not a #291 parse bug — #291's fold is correct (proven: trace_rows=[2], gutter danger-red on a single-terminal drive).

## BF-claude-disambiguate-recreates-dup-when-sibling-named-base-n
*category: validation · severity: low*

disambiguate_labels (M14 #241) can RE-CREATE a duplicate when a sibling is ALREADY literally named "{base} {n}": ["Marley","Marley 2","Marley"] → ["Marley","Marley 2","Marley 2"] (the 3rd "Marley" becomes "Marley 2", colliding with the pre-existing "Marley 2"). Cosmetic ONLY — rail click routing is by the (project,tab) index, not the label, so nothing mis-switches; it just degrades to the pre-feature ambiguous state for that one pair. Needs a sibling literally named "Marley 2" (a dir named that, or a command token reading so) — very low likelihood. Deferred with a doc note (the in-code fn-doc caveat was phase-gate-blocked as I'd set Inspect-PASS early; documented in the pipeline notes instead). Airtight fix: number against the full set of input labels (bump n until format!("{label} {n}") isn't already present). Found by the inspect critic (which CONCURRED with an otherwise-thorough self-review).

## BF-claude-disjoint-nibble-or-equals-xor-equivalent-mutant-001
*category: test · severity: medium*

marley_terminal inspect: `(hi << 4) | lo` in the hex/C-quote decoders (hi,lo each 0..=15 from hex_val) — the nibbles are DISJOINT (hi<<4 = xxxx0000, lo = 0000yyyy), so `(hi<<4) | lo == (hi<<4) ^ lo == (hi<<4) + lo` for EVERY reachable input. cargo-mutants' `replace | with ^` is therefore an EQUIVALENT mutant: it compiles, all tests pass, it reports MISSED → MSI<100, unfixable by any test. Two sites (hex_decode + c_unescape). Fix WITHOUT suppression: rewrite `|` → `+` (identical result for disjoint nibbles — no carry) so the operator mutants (`+→-`, `+→*`) become KILLABLE by the exact-decoded-byte assertions. (Alternative was an exclude_re skip, but the `+` rewrite reaches true MSI 100 with zero suppression.)

## BF-claude-drag-flag-leaks-on-bounds-gated-mouse-up-255
*category: runtime · severity: high*

The #255 drag-select `dragging_selection` flag was cleared ONLY by a per-row `on_mouse_up`. In gpui, `on_mouse_up` is bounds-gated — it fires only when the button is released over the element's hitbox. So releasing a drag OFF any editor row (the empty area below the text, the `py_1` padding, the #246 terminal split pane, or outside the window — a very common drag gesture) never fires any row's on_mouse_up → the flag stays `true`. Then every subsequent plain HOVER move over an editor row passes the `if !dragging_selection` gate and moves the caret → the exact "every mouse-over reshuffles the caret" bug the gate was meant to prevent, reintroduced through a leaked flag. Non-corrupting (offsets stay valid) but very user-visible and easily triggered; a scripted drag that happens to release over text would not catch it. FIX: a self-heal in on_mouse_move — `if event.pressed_button != Some(MouseButton::Left) { dragging_selection = false; return; }` (a button-less move means the drag ended elsewhere → clear the flag without moving the caret). Caught by an adversarial inspect critic that read the gpui-0.2.2 div.rs bounds-gating of on_mouse_up.

## BF-claude-dropped-my-own-render-half-and-passed-the-phase-001
*category: validation · severity: high*

M22 #339. My Phase 2 file manifest listed "the two chips + the inline error row on the find bar". I built the STATE (efind_regex/efind_fold/efind_error), the mode fork in refresh_efind_matches, and the palette toggles — then wrote "status: Phase 3 — Implement PASS" WITHOUT building the render half AND without recording the cut. Both critics caught it independently, from opposite lenses (the app-wiring critic by grep; the offset-seam critic by tracing the render).

THE USER-VISIBLE CONSEQUENCE: `efind_error` had FOUR writes and ZERO reads. With regex mode on and an invalid pattern `a[`, the bar rendered "a[ (no matches)" — telling the user their pattern found nothing when it never COMPILED, so they hunt the document instead of fixing the bracket. REQ-003 ("surface an invalid pattern as an inline bar error") was two-thirds met (no panic, find-next no-ops) and the actual surfacing — the point of the REQ — was missing. Separately, the two chips had no render at all, so the mode was invisible: the palette toggle changes what a match IS with zero visual feedback.

WHY NO GATE CAUGHT IT — the part that makes this a class, not a slip. `efind_error` is a PRIVATE field on a pub struct, INITIALIZED in the struct literal and only ever ASSIGNED. rustc 1.96 does NOT emit dead_code for that shape (a struct-literal init marks the field live for the dead-code pass). Both critics verified with a minimal repro: zero warnings. And app.rs is coverage-excluded + the shims are mutants::skip. So a write-only field that renders a lie is invisible to clippy, coverage, AND mutation simultaneously. This is the exact "state whose contract nothing enforces" class the batch keeps flagging (#338's tag, #339's memo key) — here it reached a shipped user-facing lie.

THE ROOT PROCESS FAILURE: I did not check my own Phase-3 output against my own Phase-2 manifest. The manifest is a checklist; I treated "PASS" as a feeling rather than a diff against the list I wrote three phases earlier. A UI ticket especially: state without render is not a feature, it is dead state, and "it compiles and the units pass" says nothing about whether a pane WORKS — which is the whole reason Validate mandates a driven pixel check.

FIX: built the inline error row (danger-colored, FindError::Display) that overrides the "(no matches)" label, plus a ".* "/"Aa " mode prefix so the chips are legible. efind_error now has readers. 1518 tests still pass; clippy clean. The RENDER must be driven-verified at Validate (deferred here — chad is at the machine, synthetic input off-limits).

Also in this review, 5th time this batch: my REASONS were wrong even where the code was right. The toggle_find_regex doc credited "efind_key = None" as the mechanism (it is belt-and-braces — the flags-in-key already invalidate); the toggle_find_case doc claimed only the regex chip can affect the error (FALSE — case_insensitive(true) expands classes and can cross REGEX_SIZE_LIMIT, so the case chip creates/clears FindErrors too). Both corrected.

## BF-claude-edge-triggered-exit-poll-invisible-at-teardown-001
*category: runtime · severity: medium*

TICKET-348's bounded PTY-teardown loop polled the child via `poll_child_exit` → alacritty `next_child_event`, which is EDGE-triggered: it reads ONE SIGCHLD self-pipe byte and returns None on WouldBlock without calling try_wait. The pump ALSO calls poll_child_exit every tick and reaps a normally-exited shell there, consuming that byte. So a child reaped during normal use was INVISIBLE to a later teardown poll → the Drop loop ran the full HUP(2s)+KILL(2s)=4s and then GiveUp-leaked the pty. Two harms on the common "shell exited, then close the tab" path: a 4s teardown freeze + an fd leak, and it SIGHUP/SIGKILLed a reaped pid the OS may have RECYCLED (signalling an unrelated process). Caught by the reap-logic critic's observation (the pump reaps + consumes SIGCHLD) + my verification (session.rs:372 pump call; alacritty unix.rs:387-391 WouldBlock→None). Fixed at source with an OsPtyChannel.child_reaped latch set in poll_child_exit (the only time the edge is observable) and read in Drop: already-reaped → skip signals+loop → ManuallyDrop::drop (alacritty reaps from the cached status instantly). All in the mutants::skip + cov-excluded shim (no cov/MSI change).

## BF-claude-enter-type-over-was-two-undo-steps
*category: test · severity: medium*

#276 inspect F1 (BOTH critics, probe-verified): the editor Enter row deleted the selection then inserted "\n"+indent as a SECOND Buffer::edit — two undo records, while the arm's own comment claimed "ONE edit (one undo step)" and the sibling #255 type-over path is a single edit(start..end, &repl). One ⌘Z landed the user on a state they never saw (selection deleted, no newline). Fix: compute (row, cin) = line_col(s) and the indent clone BEFORE the replace (the clone reads only chars strictly before s on its line, which the replace cannot change — probe-proved byte-identical), then ONE buffer.edit(s..e, &insert); bare caret degenerates to s==e==caret.

## BF-claude-equivalent-mutant-constant-return-with-dead-statement-001
*category: test · severity: low*

TICKET-021 (manager.rs is_syncable): a fn that returns a constant `false` but had a preceding no-op statement — `pub fn is_syncable<S: Setting>(&self) -> bool { let _ = S::storage_key(); false }` (the `let _` was added to reference the unused generic S out of a clippy unused-type-param worry) — made cargo-mutants emit "replace SettingsManager::is_syncable -> bool with false", which is an EQUIVALENT mutant: the fn IS already false, so the mutation is behaviourally identical → cargo-mutants reports it MISSED → MSI 99.x < 100. Fixed by a BARE `{ false }` body: cargo-mutants recognises "replace-with-false" as identical-to-the-original and does NOT generate it (only the killable "replace with true" remains, caught by r18_is_syncable_false). Clippy's `extra_unused_type_parameters` is pedantic (not in the gate's `-D warnings`), so the unused S is fine without the dead statement.

## BF-claude-error-reply-writes-no-cache-so-request-loops-forever-001
*category: runtime · severity: medium*

#331. `refresh_inlay_hints` retried on "the cache cannot serve this viewport". An Ok(null)/Ok([]) answer caches and terminates; an Err writes NO cache, so `served` stays false and the request re-fires every round-trip, forever. Two ways to earn a guaranteed Err, both live: (1) NO LANGUAGE GATE — `language_id_for` is .rs-only, so a non-Rust file (Cargo.toml) is never didOpen'd, yet the request was still sent for it; (2) `inlay_hint_support` was written, re-exported, and CALLED NOWHERE — dead code — so the request went to every Ready server regardless of `inlayHintProvider` → MethodNotFound. Both were guarded only by server behaviour, not by code. Fix: gate on `language_id_for(&path).is_some()` (reuse the ONE table docsync syncs by, never a second copy) AND on the advertised capability (the #324 posture); clamp the requested range to `len_lines` too. LESSON: a retry loop keyed on "no cached answer" MUST make every terminal outcome write something, or an error path becomes an infinite loop — and a capability reader that nothing calls is a claim the code doesn't keep.

## BF-claude-extchange-armed-write-leaves-stale-arm-001
*category: runtime · severity: low*

#284: in save_active, when the armed 2nd-press WRITE has a racing agent write land in the write gap (len mismatch → racing), the code re-flagged the Changed banner + refreshed conflict_observed but LEFT armed_at=Some(W1) — the old arm outlived its consuming write. Harmless in practice (the next press re-warns since the racing W3 ≠ W1) but a latent hole if the disk transiently returned to exactly W1. Fixed in inspect by disarming (disarm_active_save) in the racing branch too — a successful write always consumes the arm (the non-racing branch already disarmed via set_active_conflict(None)). Class: an acknowledgment/license consumed by an action must be cleared on EVERY exit path of that action, not just the common one.

## BF-claude-fixture-to-real-tab-orphaned-its-callers-001
*category: validation · severity: high*

M29 #403 (React POC): turning an ALWAYS-PRESENT chrome fixture (the Browser section's 'web' row) into a born-on-open TAB left three existing callers behind, each of which activated 'web' without adding it to `openBrowserTabs`. Result: a reachable state where the pane body rendered for a tab that had no rail row and did not exist — a state the Rust side cannot reach, so it would have shipped as a silent parity divergence and been photographed as truth by the REQ-007 parity capture. Sites: LeftRail's section-header onClick, CommandPalette's `go-browser` verb, and two close-fallbacks that defaulted the active tab back to the now-closable 'web'. A fourth artifact of the same omission: BrowserView's tab strip still hardcoded a Web button while ALSO mapping openBrowserTabs, rendering "Web" twice. Found by an inspect critic, not by typecheck (every site was type-correct) and not by the happy-path drive (which only exercised the new verb).

## BF-claude-focused-split-pane-rendered-blank-shared-closure-tab-only-read-001
*category: runtime · severity: high*

#259 editable split pane: the FOCUSED editable pane rendered BLANK (no text/gutter/caret) — you'd type blind. code_view_body's Some-arm uniform_list row closure is 'static (can't borrow the view), so it re-reads the buffer live via entity.read(app).shell.active_project().active_tab().editor(). But a split code pane lives INSIDE a Terminal tab, and Tab::editor() is None for a Terminal → the closure hit `else { break; }` on ROW 0 → zero rows, while fold_projection (the focus-aware active_editor()) reported the pane's real line count → full scroll height + blank body. TWO root causes: (1) the G4 blanket regex that moved ~43 `active_tab().editor()` bypassers onto the focus-aware active_editor() MISSED this one because its receiver was `entity.read(app)`, not the `self`/`view` the regex matched; (2) a shared render helper's 'static re-read closure diverged from the code that SIZED the list (fold_projection used active_editor(), the closure used active_tab().editor()). FIX: the closure reads entity.read(app).active_editor() (the same focus-aware resolver). Caught by an inspect critic tracing the render data-sources, not by compile (active_tab().editor() is valid, just semantically None) and not by a locked-screen validation (input/geometry were mechanism-verified; the row RENDER was not). LESSON: after a blanket accessor-move, grep the pattern with ANY receiver + multi-line; and a shared render closure that re-reads state must use the SAME resolver as whatever sized/gated it.

## BF-claude-fold-projection-parse-on-pump-tick-001
*category: performance · severity: high*

The #352 implement phase put fold_projection() — whose parse arm is buf.text() (O(n) String) + a full tree-sitter parse — onto the 16ms pump (refresh_inlay_hints ticks unconditionally) and onto every mouse-move. With any fold active on a Rust file that produced ~60 full-file parses/second while completely idle; pre-diff both paths were integer compares. Caught at inspect by both critics (the pump cadence PUMP_INTERVAL_MS=16 verified). Fix: fold_proj_cache memo keyed (nonce, version, anchor set) on the parse arm only — reload re-mints nonce, an edit bumps version, a fold verb changes anchors; identity arms stay cache-free. Lesson: before moving a computation onto a poll/pump/event path, price its worst arm at the path's cadence — render-path precedent (damage-driven frames) does not license pump-path cost (unconditional 60Hz).

## BF-claude-gate-moved-to-a-per-tick-path-kept-its-syscall-first-001
*category: performance · severity: medium*

#321 moved LSP host creation from a gesture-triggered call (fires once, when a user opens a file) to the pump tick (fires every frame). The gate's clauses were hoisted but their ORDER was carried over unexamined: `if self.lsp_hosts.contains_key(root) || !root.join("Cargo.toml").is_file() { return; }`. The second clause is a filesystem stat. On the old path that stat ran only when a `.rs` file was actually opened; on the new path it ran on EVERY pump tick, forever, for any project without a host. Correct behavior, wrong cost — and invisible to every gate (no test asserts syscall counts, and coverage/mutation say nothing about it). Fixed by reordering cheapest-first: `contains_key` (in-memory, permanently retires the question once a host exists) → the `.any()` path scan (in-memory, a few open docs) → the manifest stat LAST, reached only when a candidate `.rs` doc exists. The stat deliberately stays inside the per-tick path rather than being cached, because re-evaluating it is what lets a root that gains a `Cargo.toml` later (a `cargo init` in an open project) get language support on the next tick — so it cannot be hoisted out, which is precisely why it must come last. Found by my own equivalence analysis during inspect; neither dispatched critic returned in time.

## BF-claude-gate-spec-row-drift
*category: validation · severity: low*

#262 extended gate:14 (brand-scrub check in docs_g) but left the canonical gate table row in docs/specs/standards/quality-bar.spec.md stale — gates.sh declares that spec "the single source of truth" for the gate bar, so the row and the implementation diverged mid-pipeline. Caught by the inspect critic (MED); fixed before validate. Class: any change to a gate's behavior must update the quality-bar row in the same change.

## BF-claude-gate-wedge-real-pty-test-under-llvm-cov-253
*category: infra · severity: medium*

The FULL/diff gate wedged ~35 min TWICE on the real-PTY integration test `workspace_two_real_sessions_are_independent` sitting at 0.0% CPU under `cargo llvm-cov` instrumentation (a session spawn/handshake deadlock). NOT a #253 regression (undo/redo is pure — no PTY/session code) and NOT memory pressure (76% free — the --jobs 2 fix held). Diagnosed via `ps -o pid,pcpu,etime,command` showing the test binary alive 36min at 0% CPU (a deadlock, not slow compute). Killed the gate tree + the hung test, re-ran on a quiet system → the same test passed in ~2s and the whole diff gate finished in ~105s. A flaky real-session test under coverage instrumentation; a nextest retry/timeout on the real-session tests would harden it (surfaced to chad as a possible follow-up, not filed — out of #253 range).

## BF-claude-geom-recorder-buffer-row-eq-slot-gate-001
*category: runtime · severity: high*

app.rs:6098/:6243 gated the EditorFrameGeom recorder (and the code_w probe) on `row == first` — a BUFFER row compared to a SLOT. With a collapsed fold fully above the viewport, buffer_row(slot) > slot for every rendered slot, so neither canvas mounted and editor_geom froze at its pre-scroll value — starving every geometry consumer (the #352 overlay anchors, click→caret, IME, h-scroll clamps) in exactly the fold-above-viewport scenario #352 fixes. Pre-existing since #305 (which moved `row` into the buffer domain and left the gates); found at #352 inspect by both critics independently. The Phase-1/2 sweeps missed it because they grepped geom.first ARITHMETIC — this was an EQUALITY gate against a same-named shadowing local. Fix: `if slot == first` at both sites.

## BF-claude-goto-restore-missing-clear-marked-001
*category: validation · severity: low*

#302 go-to-line: goto_restore (the Esc path) placed the origin caret via set_active_selections WITHOUT first calling clear_marked, asymmetric with goto_commit (which does). clear_marked's contract (editor_surface.rs) is "every non-IME edit/interaction path calls this — a stale marked span after an out-of-band edit would misdirect the next IME replace." A caret placement IS such an out-of-band edit. No user-visible reproduction (requires being mid-IME-composition when the Esc control chord fires, which the OS IME layer usually commits/cancels first), so LOW — a latent robustness gap, not a confirmed bug. Caught by an inspect critic noticing the goto_commit/goto_restore asymmetry. Fixed by adding s.clear_marked() before the placement in goto_restore, and reusing the shipped set_single_caret helper in both (a cleanup the same critic flagged).

## BF-claude-gpui-fixture-bin-mutants-not-skipped-001
*category: test · severity: high*

TICKET-017: the headed widget-gallery fixture bin (crates/ui_components/src/bin/marley_widgets_gallery.rs) was coverage-excluded in scripts/gates.sh (rust_cov --ignore-filename-regex ...ui_components/src/bin/) but its fns (GalleryView::new, the Render::render impl, main) were NOT annotated #[cfg_attr(test, mutants::skip)]. cargo mutants --list -p marley_ui_components showed 8 surviving-mutant candidates ON THE BIN (4 render match-arm deletions + replace-main-with-() + 3 WindowOptions/TitlebarOptions field deletions) — the bin is exercised ONLY by the #[ignore] headed test (not run in the gate), so those mutants would SURVIVE → MSI<100 → mutation gate RED. ROOT CAUSE: coverage-exclusion (rust_cov --ignore-filename-regex) and mutation-exclusion (mutants::skip) are SEPARATE mechanisms; excluding a file from the coverage denominator does NOT stop cargo-mutants from mutating it. Caught in-context at inspect via cargo mutants --list. FIX: added #[cfg_attr(test, mutants::skip)] to all 3 bin fns (the #16 marley.rs/app.rs pattern); re-listed → 0 bin mutants, 0 render mutants, 24 pure-surface mutants.

## BF-claude-gpui-test-panic-masks-as-hang-when-sessions-drop-on-unwind
*category: test · severity: medium*

#273 validate: a FAILING assert in a #[gpui::test] that owns live TerminalSessions presented as an INFINITE HANG (SLOW >2040s), not a FAIL — the panic unwind skips the end-of-test reap_sessions, so RootView's drop chain reaps the PTY child ON the unwinding thread and blocks. Two compounding instances: (1) my shrink test tripped the documented open+jump-in-one-frame constraint → assert panic → hang; (2) after kill -9 cleanup, ORPHANED zsh children made even the passing virgin-boot test wedge inside RootView::new_in's scope-end on-thread drop of the unused boot PTY (the #271-noted arm — now fixed off-thread in this ticket). Diagnosis path that worked: ps for the test binary (0.02s CPU = wedged), then `sample <pid>` — the stack named the exact drop site. Rule: a hung headless test is USUALLY a failed assert or an on-thread session drop; sample first, never just re-run.

## BF-claude-gpui-tree-deny-licenses-advisories-001
*category: external · severity: high*

Adding gpui (Marley's UI framework, ~700-crate tree) to a crate made `cargo deny check` FAIL the supply-chain gate (gate:8) in two ways the per-crate clippy/check did not surface: (1) LICENSES — gpui's renderer pulls CC0-1.0 (hexf-parse via naga/blade-graphics, tiny-keccak via ahash) and the `image` crate's AVIF path pulls NCSA (libfuzzer-sys via rav1e/ravif); none were on the MIT/Apache/BSD/MPL allowlist. Pruning `image` to png-only (default-features=false) did NOT drop NCSA because image-compare keeps the AVIF feature via cargo feature-unification. (2) ADVISORIES — `cargo deny check` (run with no args = all checks) flagged 5 gpui-transitive UNMAINTAINED RUSTSEC notices (async-std/instant/paste/proc-macro-error2/rustls-pemfile) as errors, even though `cargo audit` (gate:7) treats them as exit-0 warnings. Fix: allow CC0-1.0 + NCSA (permissive, gpui-intrinsic) and add the 5 RUSTSEC IDs to deny.toml [advisories] ignore with documented reasons (the §0-sanctioned per-id reasoned ignore, not a silent baseline). Caught by an inspect critic running `cargo deny check` directly; would otherwise have failed the FULL gate at validate.

## BF-claude-grouped-undo-noop-transaction-wipes-redo-001
*category: runtime · severity: medium*

#282 grouped-undo: the new `begin_group` cleared the redo stack EAGERLY on transaction open, but the app's Tab/⇧Tab indent arm opened the group before knowing whether `edits` was empty. A no-op ⇧Tab on a flush-left line (dedent_edits → []) therefore destroyed a pending redo stack while making zero edits — a regression (pre-#282 the no-op ran no `record`, so redo survived) that also violated the project's own `ime.rs "redo survives the no-op"` invariant. Both inspect critics found it independently (one reproduced it in a throwaway crate: "REDO LOST"). Fix: moved the redo-clear out of `begin_group` into `end_group`'s non-empty COMMIT branch, so redo is invalidated only when a real group commits — protecting all callers, not just the two that happened to guard the empty case. Regression test: undo::an_empty_group_preserves_the_redo_stack.

## BF-claude-guard-phrasing-creates-equivalent-mutant
*category: test · severity: medium*

#266: highlight_ranges guarded empty spans with `end > at` where end = at + span.text.len() — the only empty span highlight_line can emit is Plain (already excluded by the kind check), so `>` vs `>=` was semantically EQUIVALENT and unkillable (probe-verified over 60k lines) → guaranteed MSI<100. Fix class: phrase a guard on the DOMAIN FACT (`!span.text.is_empty()`) whose mutant (delete-!) IS observable, not on derived arithmetic whose boundary case is unreachable. Sibling of the known syntactic-form rules.

## BF-claude-headless-drive-never-ticked-the-pump-negatives-passed-vacuously-001
*category: test · severity: critical*

#321's three headless drives waited with `vcx.run_until_parked()` alone. That does not advance gpui's mock clock: `TestDispatcher::run_until_parked` is `while self.tick(false) {}`, and `tick` only promotes a delayed task once `state.time` reaches its deadline — and `state.time` moves ONLY via `advance_clock`/`advance_clock_to_next_delayed`. gpui's own docstring says so ("in tests, move time forward. This does not run any tasks, but does make timers ready.").

The pump is a `background_executor().timer(PUMP_INTERVAL_MS = 16ms)` loop, and since #321 the pump is the ONLY thing that creates an LSP host. So the pump body never ran in any of the drives, and `lsp_hosts` stayed empty.

The dangerous half is not the failing asserts — it is that the NEGATIVE arms passed. `spawn_gate_rejects_headless` (a) and (b) assert `!lsp_host_exists_for_test(root)`, which was trivially true because the pump had never ticked. They would have gone green while proving nothing about the gate they exist to test, and a future regression that made the gate spawn indiscriminately would still have passed them.

The repo already had the correct idiom in 5 places (`executor().advance_clock(Duration::from_millis(40))` followed by `run_until_parked()`), which I did not reuse. Fixed by adding a shared `tick_pump(cx)` helper doing exactly that, documenting WHY `run_until_parked` alone is insufficient, and calling it in every arm including the negatives.

Found by an inspect critic, which cited the gpui dispatcher mechanism line by line. Related: the same drives also spawned a real rust-analyzer (BF-claude-headless-test-satisfied-the-spawn-gate-and-launched-a-real-server-001) — two independent defects in one test suite, both invisible to a green compile.

## BF-claude-headless-test-satisfied-the-spawn-gate-and-launched-a-real-server-001
*category: test · severity: high*

#321's headless drives must make the LSP spawn gate PASS in order to assert that a restored editor tab creates a host. The gate requires a `Cargo.toml` in the project root — so the drives write one into their tempdir. That silently removed the protection #308 had been relying on.

#308's notes record it explicitly: headless tests open `.rs` files under a tempdir root and "would spawn a REAL rust-analyzer 4x (restart cap) during the gate"; the mitigation was that `ensure_lsp_for_opened_file` also gates on `root.join("Cargo.toml").is_file()`, which "excludes bare-tempdir tests". By writing a manifest to exercise the gate, my drives re-opened exactly that hole.

`rust-analyzer` is installed at ~/.cargo/bin on this machine, and `resolve_binary` falls back to a PATH/`~/.cargo/bin` search for `rust`, so the pump launched a genuine rust-analyzer against a throwaway tempdir. The test binary then sat at 0% CPU indefinitely while the server tried to index. Symptom was indistinguishable from the machine's known intermittent syspolicyd fault (a hung test binary, no output) — I only separated them by checking `ps -o %cpu -p $(pgrep syspolicyd)` and finding it idle at 0%, then confirming rust-analyzer was on PATH.

FIX: seed `[[lsp.servers]]` with `{language: "rust", command: "/nonexistent/marley-test-stub-lsp"}` before boot. `resolve_binary` short-circuits on a configured entry, and an absolute command failing the existence probe yields None — so the host is still CREATED (what the drives assert) with `resolved: None`, and nothing spawns. The ticket's own spec had specified "a root WITH Cargo.toml + [[lsp.servers]] naming a stub binary" in REQ-001; I implemented the manifest half and dropped the stub half, which is what made this reachable.

## BF-claude-hot-click-path-materialized-whole-scrollback
*category: performance · severity: medium*

#279 inspect C1: the new grid mouse-down called content_row_texts (which walks EVERY block and allocates a String per row of the UNBOUNDED scrollback) on every left click — including single and shift clicks whose decision arms never read the row text; the pre-#279 seed was zero-alloc. The #274 class ("the hot interactive path pays for the whole history") reintroduced for clicks one ticket later. Fix: the fetch gates on !shift && count>=2. CLASS: when threading a document-derived input into a hot input handler, gate the derivation on the exact arms that consume it — the decision fn taking a param does not mean every caller path must compute it.

## BF-claude-hunted-the-hazard-for-one-token-and-never-checked-the-other-001
*category: runtime · severity: high*

#299 (⌘/ comment toggle): at PLAN time I found that `/// foo` starts with `//`, so the reference's naive prefix test ("already commented" = `trim_start().starts_with(token)`) uncomments it to `/ foo` — mangled, irreversible. I wrote a whole "THE ROUND-TRIP HAZARD" section about it, recorded it as D7 + REQ-008 with its own pinning test, and moved on satisfied. **I had examined only the token I happened to think about.** `#` — the comment token for TWO of the three supported languages (Shell, TOML) — was never looked at, and it destroys more, and worse: `#!/bin/sh` → `!/bin/sh` (the script silently stops being executable and fails at EXEC time, not compile time); `## Section` → `# Section` → `Section`; `####` and `//////` banner rules lose one token PER PRESS down to `""`, which is an ABSORBING state because blank rows are skipped and never come back. And `//!` — a Rust MODULE DOC, the first line of literally every file in this crate — → `! PURE — …`. None of the five round-trip. Worse, D7's written RATIONALE was false: "from an already-commented start the toggle NORMALIZES (`//x` → `x` → `// x`), which is correct behavior, not a bug" is asserted as a GENERAL property and holds only when the text after the token does not itself start with the token. A test written to that rationale would have encoded the falsehood and then defended it. Raised independently by two critics — both of whom saw it precisely BECAUSE I had already blessed the `///` case, which made the asymmetry visible. FIX: the token must not be followed by MORE OF ITSELF, nor by `!` — those are richer markers (doc comments, shebangs, banners), so ⌘/ COMMENTS them (`//! x` → `// //! x` → back) rather than shredding them. A deliberate divergence from the reference (§0 — do not ship known irreversible harm). All five now round-trip byte-identically.

## BF-claude-implement-check-skips-cfg-test-callers
*category: compilation · severity: medium*

At #386 implement, `cargo check -p marley` reported clean but the crate did NOT compile under `--tests`: widening `rail_rows` by one param + adding the `AppliedSettings.collapsed_sections` field broke 14 EXISTING test sites (11 `rail_rows` callers missing arg 3 + 3 `AppliedSettings` struct literals missing the field) that plain `cargo check` never compiles (`#[cfg(test)]` is skipped). Implement was declared Phase-3 PASS on that false-clean signal; the inspect critics caught it by running `cargo check --tests`. No app bug — a verification blind spot where a public-API change fans out to test callers invisibly.

## BF-claude-in-repo-tmpdir-breaks-repo-ancestry-tests-001
*category: test · severity: high*

#335 inspect F1 (HIGH, proven live by a critic): redirecting cargo-mutants' TMPDIR to a scratch INSIDE the repo (target/mutants-scratch) made every test tempdir repo-ancestored — discover_no_git_falls_back asserts its tempdir has NO .git ancestor, so the baseline failed inside every mutant run → cargo-mutants exit 4 → gate:5 RED on the first real mutation run, bricking /commit. The implementer's reasoning had checked COPY-recursion ("target/ is never copied") but missed the ANCESTRY consequence: relocating tmp into a repo changes what every tempdir's parent chain contains. Fixed: the scratch lives under the SYSTEM tmp in a named dir (${TMPDIR}/marley-mutants-scratch/$$) — still trap-cleanable by exact name, still outside the copy set, and test tempdirs keep their system-tmp ancestry. Verified by re-running the exact failing probe (passes) + a real bounded cargo-mutants run through the redirect (ok Unmutated baseline).

## BF-claude-index-keyed-state-not-remapped-on-vec-remove
*category: validation · severity: medium*

#236 stored the rail collapse state as a `collapsed_projects: HashSet<usize>` of raw project indices. `close_project` does `self.projects.remove(idx)` (a shifting Vec removal), but `close_project_at` did NOT remap the set (it remaps the active pointer via adjust_active + resets renaming_tab, with the comment 'project indices shift too', but missed collapsed_projects). So collapsing project B(idx 1) then closing A(idx 0) left collapsed={1} while B shifted to idx 0 → project C (now idx 1) rendered collapsed instead of B — the collapse ALIASED the wrong project. Transient view state, self-corrects on the next click, no panic → MEDIUM. An inspect critic caught it (the pure rail_rows READS harmlessly via contains(), which masked that the WRITE side aliases a live project). Fixed: remap collapsed_projects on close (drop idx, decrement indices > idx), mirroring the #177 renaming_tab index-shift guard.

## BF-claude-inlay-reload-version-epoch-collision-001
*category: runtime · severity: low*

#401 inspect (correctness critic): reload_active_from_disk resets the buffer version epoch (BufferVersion::initial) and re-mints the nonce but cleared NO inlay state. InlayKey carries (uri, version) only — no nonce — so on a never-edited file (version == initial both sides) a raced pre-reload inlay answer passes the apply guards numerically, projects onto the NEW text, and is cached wrong. Pre-#401 the served-check's forever-refetch bug accidentally replaced the wrong hints one round-trip later; the #401 fix removed that self-heal, so the wrong hints would have persisted until an edit/scroll/server-refresh. Pre-existing (#331-era) acceptance-of-raced-answer; surfaced because #401 deleted the accidental recovery. Fixed at the reset site: the reload now drops inlay_hints + inlay_request (beside the #328 git-marks invalidation), pinned by inlay_cache_dropped_on_disk_reload_headless.

## BF-claude-input-routing-alt-screen-only-misses-primary-screen-interactive-programs
*category: runtime · severity: medium*

Input routing keyed on the ALTERNATE screen only (input_route(alt_screen, ctrl) from #33), so interactive programs on the PRIMARY screen — Claude Code arrow-key menus, `read`, `npm init`, interactive git — never received arrow/nav keys; Marley routed them to its own local history recall. Root cause: the routing model conflated 'a program wants raw keys' with 'the alternate screen is active'; the real signal is 'a foreground command is running'. An inline interactive program doesn't switch to the alt-screen, so it fell to Cooked → local editor. Found by chad running the built app. FIX: route by is_command_running() (a Running block exists, bracketed by the shell Preexec/Precmd) — input_route(alt_screen, ctrl, command_running) streams Raw if any hold; while a command runs every key reaches it, local editing only at the bare prompt. cov/MSI 100 on both pure fns; orthogonal to the local-edit model.

## BF-claude-inspect-fix-recorded-but-never-applied-001
*category: validation · severity: high*

M22 #337. Inspect finding F5 was "settings.rs's font_family doc promises a status flash that does not exist". I wrote the finding, wrote the resolution ("The settings doc now describes what the code does: ... degrades to gpui's system fallback SILENTLY"), and NEVER APPLIED THE EDIT. The defective sentence — "An unresolvable family falls back to the built-in WITH a flash — never a silent wrong font" — was still in settings.rs verbatim at Phase 4, having survived Implement, Inspect's own post-fix re-verification, and a full GATE GREEN [diff].

WHY EVERY CHECK MISSED IT: the fix was DOC-ONLY. `cargo check`, `clippy -D warnings`, 615 nextest tests, `cargo fmt`, the §20 brand grep, coverage 100%, mutation MSI 100 and rustdoc -D warnings are all blind to a doc comment's CONTENT. A false doc is well-formed Rust. There is no gate for "this sentence is a lie", so the only thing standing between a wrong doc and the user is a human re-reading the file — and I substituted the LEDGER ENTRY for that re-reading.

COMPOUNDING: the reason F5 recorded was ALSO false. Both docs claimed the flash "needs a font-resolution probe this path lacks"; gpui exposes `TextSystem::font_id(&Font) -> Result<FontId>` (text_system.rs:109) and the mac backend `?`s `load_family`, so the probe exists and the cut is scope, not capability. One finding thus produced a fix that never landed AND a justification that was never true, with the ledger asserting both were handled.

CAUGHT BY: re-reading the actual source at Phase 4 while checking REQ-006 against my own tests rather than against my memory of them. An audit of the other five findings (F1/F2/F3/F4/F6) confirmed they DID land — so the failure is not sloppiness in general, it is specifically that a doc-only fix has no verifier.

This is the same class as the ticket's other two: #336 F5 (a correct fix citing `element_offset` when taffy was the carrier) and #337 F3 (a correct fix whose comment had the physics backwards). A plausible reason/record stops the review, so the reason/record never gets measured.

## BF-claude-invariant-tag-asserted-not-earned-corrupts-redo-001
*category: runtime · severity: critical*

M22 #338. `edit_ranges_restoring` hardcoded `begin_group(restore, /*cursor_anchored*/ true)`. That flag's documented meaning is "every cursor ends exactly at the end of its own insert" — the precondition `UndoHistory::coalesces_into` RELIES ON, because it performs NO arithmetic contiguity check and simply appends the next typed char onto the group's record.

#338 introduced `CaretSpan`, whose entire purpose is to put the caret somewhere OTHER than the end of its insert (`base + 1` after a 2-char `()`). The tag then LIED. Driven through the real IME door at 2 cursors with auto-close on:

  type `(`  -> "() () "        (correct)
  type `x`  -> "(x) (x) "      (correct)
  ⌘Z x1     -> "  "            WRONG: one step swallowed both
  ⌘⇧Z       -> "()x ()x "      TEXT CORRUPTION (expected "(x) (x) ")

`coalesces_into` passed every check and appended "x" onto "()" → `inserted = "()x"` at a site whose real text is "(x)". Undo survives (it uses only `inserted.chars().count()`), but REDO replays `inserted` verbatim and writes the wrong characters. auto_close=false and N=1 controls both round-trip correctly — the corruption is specific to a non-identity span at N>=2. This is the same text-corrupting class M19 #299's inspect found, reintroduced by the one mechanism that can now break the precondition.

WHY MY DESIGN MISSED IT — the interesting part. Phase 2 explicitly "verified" this area and got it HALF right: I checked that InsertPair's 2-char record can never join a PREVIOUS run (`record`'s guard and `coalesces_into` both require `inserted.chars().count() == 1`) and concluded the direction was safe. But `coalesces_into` constrains only `new.inserted` — there is NO `old.inserted` check — so a FOLLOWING char joins the InsertPair group. I verified one direction and wrote the note as though I had verified both. The design note said, in as many words: "Same at N>=2: coalesces_into requires new.inserted.chars().count() == 1 -> a 2-char record never joins a run." True, and irrelevant to the actual bug.

A SECOND instance of the same root cause (found by the other critic, same review): at N=1 the engine opens NO group, so `record` stores `sel_after: None` and `redo` RECOMPUTES the caret as `rec.at + inserted.chars().count()` — the end of the insert, i.e. after the `)`. That is precisely the bug D-FIXUP-IN-ENGINE was designed to prevent; the design's mitigation ("the fix lives in-engine so sel_after is right by construction") was void on the single-cursor path, which is the common case. Wrap at N=1 lost its whole selection the same way.

FIX: `begin_group(restore, spans.iter().all(Option::is_none))` — earn the tag — plus gating the N=1 no-group fast path on the same predicate so a non-identity span gets a group with a truthful `sel_after`. One predicate, both bugs, and every pre-#338 caller keeps `true`/no-group byte-identically. Proven by negative smoke: restoring the hardcoded `true` fails the N>=2 redo probe with left "  " vs right "() () ".

## BF-claude-launcher-dismiss-backdrop-hides-coopen-naming-draft-001
*category: runtime · severity: low*

#229 added a full-screen dismiss backdrop behind the #181 agent launcher. The launcher-open guard (app.rs:3158-3161) closed palette/finder/history/find but NOT the #204 naming_workflow save-as draft, which renders BEFORE the launcher. Via the 🧠 top-bar icon (which bypasses the key router's overlay guards), the launcher can open over an active naming draft; pre-#229 the naming box peeked around the launcher box, but #229's full-screen `.inset_0()` backdrop now FULLY HIDES the naming draft while its keyboard guard (:3512) still owns input → an invisible-focus state. Caught by the inspect critic (LOW), fixed by adding `self.naming_workflow = None;` to the guard. Lesson: a full-screen dismiss backdrop for modal A hides (not just occludes) any overlay that renders before A, so A's open path must close every such overlay. Rule: PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays-001.

## BF-claude-line-carry-attributes-endpoint-by-row-not-offset-span-001
*category: validation · severity: high*

#300 move/duplicate lines: the carried-selection math (`move_lines`/`duplicate_lines` in line_move.rs) attributed each selection endpoint to its moving block by the endpoint's raw row (`buffer.line_col(off).0`). But the universal "N full lines selected" shape (Home → ⇧↓×N) is `anchor in the block, head at col 0 of the row PAST the block`, and `indent::line_span` deliberately CARVES that col-0 row out of the block (indent.rs:45). So the boundary head sits on row b1+1, EXCLUDED from the block → it got delta 0 while the anchor got the real move delta → the carried selection distorted (shrank on move-down, grew on move-up, doubled to span both copies on duplicate-up). The TEXT edits were correct in every case; only the carried SELECTION was wrong — so no text/round-trip test catches it, only a selection-shape assertion on a full-line selection. Reachable via the standard shift-arrow line selection; it COMPOUNDS (a shrunk selection fragments the next move). Caught by an inspect critic. Fixed by attributing the endpoint by OFFSET SPAN (`[line_start(b0), line_start(b1+1)]`), inclusive of the carved boundary for MOVE + DUP-UP but exclusive for DUP-DOWN (where rebase_through already carries the boundary onto the copy). Sibling low finding L2: the phantom trailing row was movable up (no last_content_row clamp on the up-guard), dropping the file's trailing newline — fixed with a touched_content_rows clamp.

## BF-claude-line-span-col0-endpoint-drags-extra-row
*category: validation · severity: medium*

#276 inspect F2: line_span included a row the selection touches only at column 0 (shift+Down / full-line sweep — the most common block gesture), so Tab indented a visually-unselected line, and indent_edits padded EMPTY lines into whitespace-only lines. The reference convention (VS Code/Zed observed) excludes both. Fix: col-0 carve in line_span (gated last > first so bare-caret ⇧Tab/D4 is untouched) + an empty-line filter in indent_edits. Trap within the fix: an `a_row <= c_row` orientation branch for picking the end-column is an EQUIVALENT MUTANT at row-equality (the col is dead when last==first) — pick endpoints via .min()/.max() METHOD calls instead, which cargo-mutants never mutates.

## BF-claude-lines-memo-version-only-across-reload-001
*category: runtime · severity: medium*

#356: CodeViewState::sync_lines_from memoized its read-only `lines` on `buffer.version()` ALONE. reload_active swaps in a FRESH Buffer::from_text whose version restarts at 0 (BufferVersion::initial), without resetting the view memo — so an UNEDITED pane (buffer at v0, lines_version Some(0)) reloaded to new text (also v0) would see Some(0)==Some(0) and no-op, rendering the PRE-reload lines: the exact "changes vanish" symptom the ticket set out to kill, reintroduced in the agent/disk-reload path. This is the same (nonce, version) trap #268/#273 already documented (a fresh buffer restarts at 0, so version alone cannot tell a reopened buffer from the cached one — which is why reload_active re-mints the nonce). Caught at Phase-3.5 inspect. Fixed by `f.view.lines_version = None` in reload_active (beside the nonce re-mint), forcing the next sync to rebuild.

## BF-claude-llvm-cov-phantom-line-from-generic-instantiations-001
*category: infra · severity: medium*

#399: the coverage gate failed on ONE "missed line" in a fully-tested pure module that NO per-line view could locate — lcov DA records all nonzero, the annotated text report showed no zero region, and a region-level cross-reference of live vs dead function records found no exclusively-dead line. Root cause: the module's generic fns (index_or_push<T>, resolve_arrangement's impl FnMut param) get instantiated + inlined into every LINKING test binary's rlib covmap; the never-run copies in non-calling binaries (the integration-test binary) feed llvm-cov's line-summary merge an arithmetic phantom. Two diagnostic traps compounded it: `--show-missing-lines` changes llvm-cov's exit pathway (the same invocation PASSES with the flag, FAILS without — a masked reproduction), and ad-hoc `cargo llvm-cov` runs without `clean` merge stale profraws across builds. Fix at the root: the cov-100 pure module is now generic-free (`&mut dyn FnMut` closures, inline find-or-push) — one compiled body per fn; documented in the module and matching the gate's own syntax/parse.rs phantom-class note. Cost: three red gate cycles.

## BF-claude-lsp-init-error-false-ready-001
*category: runtime · severity: medium*

#308 lsp_host on_message: an `initialize` ERROR response was misclassified as a successful handshake. The code did `if let Ok(value)=result { parse }` then UNCONDITIONALLY stepped Event::InitializeResult → phase Ready + `initialized` sent to a server that REJECTED initialize (an LSP-spec violation), status falsely "ready", the dead wire never restarts (lifecycle thinks it's Ready). Root cause: an async response carries a Result, but only the Ok arm was routed to a distinct outcome; the Err arm silently fell through to the success transition. Found by the correctness critic (P3.5). Fixed: match the Result — Ok→InitializeResult, Err→on_connection_lost() (tear down, restart cap governs). Class: any request whose RESPONSE can be an error must branch the error to a real failure transition, never let it fall through to the success path.

## BF-claude-lsp-relative-path-exec-001
*category: security · severity: medium*

#308 marley_lsp: a relative PATH entry (`.` or any relative dir) made `search_path` return a RELATIVE command string; `spawn_server` then ran it with `current_dir(workspace_root)`, so a program string containing a separator is resolved by the OS against the child's cwd — executing a `rust-analyzer` a hostile cloned repo happens to ship (local code execution on opening a hostile repo + a `.rs` file). Empirically confirmed by the security critic (`Command::new("./x").current_dir(child)` runs child/x). Root cause: PATH search returned the candidate verbatim without requiring an absolute directory; the empty-element filter defused only the classic empty==cwd case, not a literal `.`/relative dir. Fixed: `search_path` skips non-absolute PATH dirs → always returns an absolute command, immune to cwd resolution.

## BF-claude-mouse-opened-backdrop-overlay-skips-keyrouter-overlay-guards
*category: runtime · severity: low*

The #244 workspace-switcher indicator (a TOP-BAR MOUSE-CLICK overlay with a full-screen inset_0 backdrop) reset only the obvious transient overlays (context_menu/agent_launcher/palette_open/naming_workflow/renaming_tab) and MISSED the keyboard-opened no-backdrop cards finder_open/history_open/find_open + the completion popup. A mouse click bypasses the key-router's overlay-dismiss guards (routing still sends keystrokes to those overlays), so opening the switcher left a keyboard-live finder/history/find/completion invisible-but-still-typing UNDER the switcher backdrop — the #229 orphan class. Caught in inspect (my self-review found it; the critic independently confirmed it). Fixed by mirroring the SHIPPED new-agent top-bar arm (app.rs:3319-3332) which already closes palette/finder/history/find/naming for exactly this reason. Severity low (Escape-recoverable, uncommon path, no data loss). Class: a mouse-opened backdrop overlay must close the SAME set the analogous mouse-opened precedent (new-agent) closes — grep the precedent, don't re-derive the list from the overlays you first think of.

## BF-claude-mutants-skip-body-moved-out-fourth-strike
*category: test · severity: high*

#264: the mutants::skip detach trap's FOURTH strike, in a NEW shape — not an inserted fn above a skip, but a CTOR SPLIT: RootView::new kept its skip while its ~600-line body moved into unskipped new_in → 57 live mutants. WORSE: the --diff gate was structurally blind (the only in-diff mutant, new_in body→Default::default(), is UNVIABLE — RootView has no Default — so diff-mode prints 0-viable PASS while the next FULL run would collapse). Rule refinement: the --list re-check must follow ANY refactor that MOVES a function body (split/rename/extract), not just insertions near a skip; and a green --diff mutation on a shim-file refactor proves nothing when the moved body's only in-diff mutant is unviable.

## BF-claude-mutants-skip-detach-strike-six
*category: test · severity: high*

#280 inspect HIGH: inserting pane_mouse_cell (with its own doc block) BETWEEN pane_grid_pos's doc+#[cfg_attr(test, mutants::skip)] and its fn re-bound the skip to the NEW fn — pane_grid_pos gained 14 live mutants (untested by design) and the brand-new geometry fn had ZERO, plus the old doc became a Frankenstein description of the wrong fn. STRIKE SIX of the documented detach trap (previously #183/#184/#272 and others) — it fires precisely when adding a sibling fn above a skipped shim, which is the most natural insertion point. The standing rule held: cargo mutants --list -f AFTER the edit is the only reliable detector (the critic ran it; re-seat verified 0/28). The class needs the --list check to be REFLEXIVE on any edit within 5 lines of a skip attribute.

## BF-claude-mutants-skip-detach-third-recurrence
*category: test · severity: medium*

#261: the mutants::skip DETACH TRAP fired a THIRD time (after #183/#184) — inserting icon_label between caption_header's doc/attr stack and its fn rebound the doc + #[cfg_attr(test, mutants::skip)] to the NEW fn, leaving caption_header unmasked (a caption_header body mutant appeared in cargo mutants --list; survived only because gpui::Div lacks Default → unviable). Fixed by re-seating the whole icon_label block ABOVE caption_header's doc; --list re-run shows 0 mutants on both helpers. The existing rule (verify with cargo mutants --list after inserting a fn near a skip) held — inspect caught it because the critic ran the list.

## BF-claude-mutants-skip-detach-trap-recurred-252
*category: test · severity: high*

#252: inserting `save_active` (with its own doc + #[cfg_attr(test, mutants::skip)]) directly ABOVE `fn dispatch_action` stranded dispatch_action's OWN doc+skip — a doc/attribute binds to the NEXT item, so both skips + both docs attached to save_active, leaving fn dispatch_action with NO skip → 11 live mutants on the big dispatch shim → MSI RED. THE EXACT KNOWN TRAP (mutants-skip-detach-trap.md; hit before on #183/#184/#246). An inspect critic caught it via `cargo mutants --list` (0 dispatch_action mutants before, 11 after). Fixed: moved dispatch_action's doc+skip back directly above `fn dispatch_action`, kept save_active's own doc+skip above save_active. ALSO: adding the (⌘S,"save") keymap binding failed the `all_chords_lists_every_binding` roster-count guard (38→39) — the guard working as designed; fixed the count + asserted ⌘S is in the roster.

## BF-claude-mutants-skip-detached-by-refactor-extract
*category: test · severity: medium*

Extracting a guard ladder out of the masked shim `open_file_in_viewer` into a new `load_code_view_state` MOVED the `#[cfg_attr(test, mutants::skip)]` onto the extracted helper, leaving the refactored `open_file_in_viewer` (a &mut self fs-IO method with no unit test) UNMASKED. app.rs has NO global mutation exclude (its gate exclude is COVERAGE-only), so mutation relies on per-fn skips → cargo-mutants' 'replace body with ()' mutant on the now-unmasked fn would survive → gate:5 (MSI 100) RED on the diff. Caught by the refactor-safety critic (my self-review missed it). Class = the known mutants::skip detach trap, this time via EXTRACT-and-relocate (not the insert-above variant): a structural edit around a masked shim silently detaches its skip from the fn that still needs it. Fixed by restoring the skip on open_file_in_viewer; verified via `cargo mutants --list -f app.rs | grep` (all 3 shims absent = masked).

## BF-claude-mutation-survivors-loopbound-childbin-serial-001
*category: test · severity: medium*

Validate-phase gate:5 (mutation) surfaced 3 issues in marley_visual_harness that the headless test suite + nextest had not caught: (1) ENV RACE — capture.rs tests set_var/remove_var on MARLEY_VISUAL_APPROVE; under cargo-mutants' THREADED `cargo test` (not nextest's per-process isolation) they raced a concurrent APPROVE-unset test, so the *unmutated baseline* failed and the whole mutation run aborted ("1 Failure"). Fixed by `#[serial]` on all APPROVE-sensitive tests + an RAII ApproveGuard (Drop unsets even through a should_panic unwind). The established marley_core/marley_command lesson; nextest masked it. (2) EQUIVALENT LOOP-BOUND MUTANT — diff_image's `for px in 0..(capture.w * capture.h)`; the existing test used a 5×1 fixture where 5*1 == 5/1, so the `*`→`/` mutant produced the SAME loop count → observationally equivalent → survived. Fixed by adding a 1×2 fixture (w*h=2 ≠ w/h=0) asserting the second row. (3) CHILD-PROCESS BIN — the runner bin's main() executes only in a spawned child (the #[ignore] headed self-test spawns it), so no in-process test kills the "replace main with ()" mutant; fixed with #[cfg_attr(test, mutants::skip)] + the rust_cov coverage-exclude. All three only appeared in the FULL workspace mutation run, after coverage was already 100%.

## BF-claude-new-chord-shadowed-by-hardcoded-key-check-001
*category: runtime · severity: medium*

#64's new cmd-shift-f "toggle-forge" chord was dead-on-arrival: the pre-existing find-bar intercept (`modifiers.platform && key=="f"`, no shift guard, app.rs) ran before the keymap dispatch and returned, so cmd-shift-f opened the find bar and the forge overlay never toggled. The keymap action_for unit test passed (binding table, not runtime reachability); a correctness critic caught it in inspect by tracing gpui event delivery. Root cause: a hardcoded key check shadowed a keymap chord sharing its base key. Fix: added `&& !modifiers.shift` so cmd-F still finds and cmd-shift-f falls through to toggle-forge; verified by self-test-driving the real keystroke. See PR-claude-new-chord-shadowed-by-hardcoded-key-001.

## BF-claude-new-render-arm-missing-sibling-filter-guard-001
*category: runtime · severity: medium*

#390 un-nested split-pane cell rows from under their terminal tab into a new Panes rail section, adding a new RailLevel::Arrangement render arm (app.rs). The sibling RailLevel::Pane arm hides itself during the "Search tabs" filter (`if !self.session_filter.is_empty() { continue; }`) because its generic "pane N" labels never match a title query — but the new Arrangement arm (whose labels "PANE n" are equally generic) was written WITHOUT that guard. Result: during a tab-search, the Panes section rendered dangling "PANE 1"/"PANE 2" headers with no (hidden) cells beneath — a cosmetic self-inconsistent regression. Found INDEPENDENTLY by BOTH inspect critics. Fix: added the identical `if !self.session_filter.is_empty() { continue; }` guard to the Arrangement arm, leaving only the Panes header during a filter (consistent with the E/T/B headers persisting when their lists filter empty). Class: a new/relocated render arm for a row must mirror the render-time guards (search filter, visibility) of the sibling row it parallels.

## BF-claude-nonblocking-pty-busyloop-no-backoff-races-child-001
*category: runtime · severity: high*

marley_spike VALIDATE: the headless PTY integration test (R3/R4/R5) caught a THIRD real bug that inspect + check + clippy all missed — drive_reader's Retry branch decremented the poll budget with NO inter-poll delay, so max_polls=2000 exhausted in ~1.4ms, BEFORE the freshly-forked /bin/sh child wrote (~6ms). It returned ShutdownOutcome::Clean with an EMPTY grid (read 0 bytes). Proven: a sleep-drain read 'hello-marley' after 6.4ms; drive_reader(2000) returned row0='' in 1.4ms; drive_reader(50_000_000) waited ~4.2ms and DID capture it. Impact wider than the test: app::run calls the same drive_reader(2000) before opening the window → the real app would render a BLANK Block body (violates R5/R9/R10). Root cause: the proven spikes (ptyprobe/holdprobe) BOTH thread::sleep per WouldBlock; the productionized drive_reader dropped that backoff. Fix: a fixed ~1ms std::thread::sleep on the Retry branch (the budget still bounds termination; mock-reader unit tests return instantly so they're unaffected; the budget-exhaust mutants still infinite-loop → cargo-mutants timeout). The unit suite (mock reader, no real timing) could NOT catch this — only the real-PTY integration test did. This is the 3rd bug (after the Pty-drop + the ExitCode opacity) that a real end-to-end test caught past green check/clippy/unit.

## BF-claude-origin-collapse-fixture-survives-index-mutants-001
*category: test · severity: medium*

TICKET-008 inspect (caught BEFORE validate): the Phase-2 design's mutation-kill fixture for mask_out was degenerate — a 1×2 image masking ROW 0, i.e. the ORIGIN pixel (0,0). The index expression `((y*w+x)*4)` evaluates to 0 there, and 0 is a FIXED POINT of all the arithmetic operators cargo-mutants swaps: `y*w`→`y/w` (0/1==0*1), `…+x`→`…-x` (0-0==0+0), `…*4`→`…/4` (0/4==0*4). So 4 index mutants on image_diff.rs:283 produce byte-identical output and SURVIVE → MSI<100 would have hit at validate. This is a SHARPER form of the 005/007 single-row lesson (PR-claude-loopbound-fixture-and-childproc-bin-skip-001): not just loop bounds, but ANY index-arithmetic mutant is undetectable when the test exercises only index 0 (or a 1-wide grid where x is always 0). The critic verified via `cargo mutants --list` + a hand trace. Fix (test-only): use a NON-ORIGIN interior masked pixel in a multi-row, multi-col, w≠h grid with an unmasked remainder — solid(2,3,...) + titlebar_band(2,3,2) (masks rows 0-1 incl pixel (1,1), leaves row 2), asserting the masked bytes == fill AND the unmasked row == original. The production code is CORRECT; only the kill-fixture was insufficient.

## BF-claude-osc8-test-plan-under-specified-ranges-overlap-boundary-001
*category: test · severity: medium*

#214 Phase-2 regression test plan (REQ-005 L5) specified a single "touching (keep) vs overlapping (drop)" boundary test for the new `ranges_overlap(a,b) = a.start < b.end && b.start < a.end` helper. But `cargo mutants --list` on links.rs shows the predicate yields TWO distinct viable `<`→`<=` mutants (one per comparison), each killable only at a DIFFERENT touching orientation. The single planned L5 would kill only one → the other survives → MSI < 100 at Phase-4 validate (gate RED). Caught by the inspect critic (Critic 2) before validate — not shipped. Fixed by amending the plan to L5a (scanned STARTS at explicit END, kills a.start<b.end→<=) + L5b (scanned ENDS at explicit START, kills b.start<a.end→<=) with concrete run inputs. The code was correct; only the test plan was under-specified. Rule: PR-claude-two-sided-interval-predicate-needs-boundary-fixture-per-comparison-001.

## BF-claude-overlay-keys-leak-keychar-to-ime-fallback
*category: runtime · severity: high*

#267 inspect (critic B, HIGH): every full-capture overlay arm in the on_key_down ladder returns WITHOUT stop_propagation, so once an EntityInputHandler is registered, gpui's dispatch_keystroke fallback (propagate && key_char.is_some()) delivers the consumed key's key_char into the editor buffer in the HEADLESS lane — finder plain-Enter lands "\n" at the caret; top-search Enter on a File hit opens the editor tab then instantly dirties it. The blocked-predicate can't save it: the overlay's Enter FLIPS its state off before the fallback runs. Real macOS discards via the branch-B doCommandBySelector path — the enforced test lane diverges from mac. Fix: cx.stop_propagation() in all 13 full-capture overlay arms + the completion popup's handled branch (its fall-through printables stay live by design).

## BF-claude-per-cell-home-rows-tripled-the-section-001
*category: validation · severity: high*

#398 rail cross-list v1 gave every terminal mounted in a multi-cell grid its own home-section row (tab_listed = single-cell tabs only). An ordinary 2-cell split then rendered THREE Terminal-section entries (the tab row + a CrossRef per cell) — a visual regression of every pre-existing split, caught by the inspect critic. Root cause: the "pane-mounted" predicate was structural (grid arity) when the real signal is FOREIGN HOSTING (content hosted by ≥2 distinct tabs). Fixed by reworking the semantics: a tab row is the home entry for ALL its cells; ⊞ marks foreign-hosted tabs; per-content cross rows only for editors (whose Editor-section entry exists nowhere else).

## BF-claude-per-view-walks-over-shared-content-001
*category: runtime · severity: high*

#398 made view_count>1 reachable for terminals, and THREE pre-existing per-view walks silently became wrong over shared content: (1) five close paths scrubbed the content-keyed maps (agents/remotes/last_agent/notify_ticks) per VIEW — closing one twin cell deleted a live agent's Fleet identity and disarmed the never-auto-close-a-remote guard; (2) the per-cell PTY resize loop over the one per-content pty_size shadow made different-sized twin cells fight (two ioctls + SIGWINCH storm per frame, reachable via split-right then add-down); (3) the pump pumped each VIEW (double-drain + notify_ticks counting 2x, halving the #203 threshold). All three were correct while view==content held (pre-#398) and none announced their assumption. Fixed: scrubs unified into the release tail firing on last-view drop only; one resize authority per content (focused cell wins); pump dedupes per content.

## BF-claude-perf-pin-encoded-an-unmeasured-premise
*category: performance · severity: medium*

#274 inspect TS-F2: the REQ-003 perf pin ("incremental < full/3 end-to-end") encoded the ticket's premise that "the parse half dominates" — the critic's probe measured it BACKWARDS: the tree-sitter re-parse IS 9× faster (1.0 vs 9.2 ms release on 8k lines) but the O(file) query walk (5.7 ms over 27k captures) runs in full on every incremental call, flooring end-to-end at 0.46. A pin written from an inherited premise instead of a measurement would have sent Phase 4 chasing an impossible number or, worse, invited a gate-weakening 'adjustment' under pressure. CLASS: measure the pin's decomposition BEFORE writing it — a ratio pin needs per-stage attribution (parse/query/post) from a probe, not the ticket's narrative. Fix: REQ-003 amended to pin each true thing (parse-only < 1/3, end-to-end < 3/4, frame-safety via the off-thread REQ); the query-walk windowing recorded as a measured follow-up. Also W-F1 (the dead-worker pending wedge behind its own guard — the Phase-3 comment overclaimed "can never wedge") and TS-F1 (point_at non-boundary panic reachable through the public API) fixed at source.

## BF-claude-persist-x-codec-correct-but-save-never-triggered
*category: validation · severity: high*

#243's multi-file editor save/restore CODEC was fully unit-proven (serialize/parse/reclamp/from_files round-trip, cov/MSI 100, the inspect critic concurred) — but the DRIVEN quit→relaunch showed the editor tab VANISHED. Root cause: open_file_in_viewer (app.rs) mutated the editor tab in-memory via open_or_switch_code and NEVER called persist_grid, so the layout SAVE was never TRIGGERED on file-open (a pre-existing gap; pre-#243 the single-file editor tab was equally un-persisted on open). REQ-004 ('open N → quit → relaunch → restore N') could not be met by a correct codec alone. Fixed by adding self.persist_grid() to open_file_in_viewer's success arm. LESSON: a 'persist X' feature must verify the persist is TRIGGERED end-to-end (a driven quit→relaunch), not just that the codec round-trips — every unit test + the codec passed while the feature was silently broken end-to-end; only the pixels/settings.toml caught it.

## BF-claude-persisted-visual-order-came-from-id-sort-not-the-tree-001
*category: validation · severity: medium*

#399: the arrangement snapshot walked grid.pane_ids() — which returns panes sorted by numeric PaneId (creation order) — while the codec promised "visual order" in three doc sites. The two orders diverge under an ordinary gesture (insert_split_at mints a monotonic id but inserts ADJACENT to the target; splitting a non-last pane of a 3-pane grid yields tree order [0,3,1,2] vs id order [0,1,2,3]), so a saved arrangement could reopen with its panes silently rearranged — and re-persist wrong. The shell codec next door already did it right (serialize_grid flattens grid.group(), the DFS tree walk). The apply path compounded it by rotating pre-seed cells to after the terminal seed. Fix: snapshot + card summary walk grid.group().panes(); apply mounts pre-seed cells with SplitDirection::Before re-targeting the seed.

## BF-claude-poll-driven-request-wedges-on-dropped-purpose-001
*category: runtime · severity: medium*

#331. The LSP request-timeout path does `purposes.remove(&id)` and pushes NO response — so the consumer's apply fn never runs. Event-driven consumers (hover/completion/signature) tolerate this: each keystroke mints a fresh key, so their stale latch is inert and the next send overwrites it. The inlay refresh is POLL-driven off an UNCHANGING viewport key, and its send-skip (`inlay_request == Some(&key)`) is load-bearing (without it the pump re-sends every 16ms tick while the cache is empty). So a dropped purpose left the latch matching forever → no resend → hints silently dead for exactly as long as the user READS the file — which is exactly when a cold rust-analyzer exceeds the 10s timeout. AMPLIFIER: when the server finished indexing and sent `workspace/inlayHint/refresh` (the literal "hints are ready now" signal), the destructive `take_inlay_refresh` consumed it, dropped the cache, and hit the skip — the one mechanism that would heal the wedge guaranteed it stayed wedged WITH AN EMPTY CACHE. Directly contradicted `REQUEST_TIMEOUT_TICKS`'s own comment: "a slow server never wedges a consumer". Fix: ask the host (`has_pending_inlay()` over the pending table — the ONE source of truth for in-flight-ness) instead of trusting a local latch; plus clear the latch on refresh so a pre-refresh in-flight answer is discarded.

## BF-claude-prefix-grep-heading-false-allows-sibling-heading
*category: test · severity: low*

enforce-warp-reference.sh checked for the required `## Reference (§20)` section with `grep -qE '^## Reference'` + `awk /^## Reference/` — both PREFIX matches. So a sibling heading `## References` (plural) or `## Reference Material` with any content satisfied the gate even when the mandated `## Reference (§20)` was entirely ABSENT → false-ALLOW (a spec commits with no real Reference section). Bounded: only slips when the exact `## Reference (§20)` heading is missing (if it exists but is empty, the awk stops at the next `## ` so a following plural heading doesn't rescue it). Caught by the inspect critic. Fix: anchor both the grep presence check + the awk section-extractor to the exact `^## Reference \(§20\)`. Class: a prefix-anchored heading match (`^## X`) false-matches sibling headings sharing the prefix (`## Xs`, `## X Material`) — anchor to the full heading.

## BF-claude-pty-size-shadow-advanced-on-failed-resize-elides-retries-001
*category: runtime · severity: medium*

The PTY resize shim advanced the PaneState.pty_size guard shadow unconditionally (`let _ = session.resize(cols,rows); pty_size = (cols,rows)`), even when the resize ioctl failed. The unchanged-guard compares the next computed grid against pty_size, so a failed resize (which leaves the PTY + alacritty grid at the OLD size) records the TARGET size in the shadow and elides every future retry — the pane stays silently stuck mis-sized until the rect changes to a different grid. Also contradicted pty_size's docstring ("the size last APPLIED"). FIX: advance the shadow only on success — `if state.session.resize(cols,rows).is_ok() { state.pty_size = (cols,rows); }`; a persistently-failing (dead) pane then re-attempts one cheap ioctl/frame instead of silently lying. Caught by an inspect shim-critic tracing resize()'s early-return-on-ioctl-failure against the unconditional shadow write. Prevention: PR-claude-advance-shadow-only-on-success-001.

## BF-claude-pump-idle-sleep-floor-times-panes-frame-killer-001
*category: performance · severity: high*

TICKET-023: marley_terminal::pump's WouldBlock retry budget (8 × ~1ms sleeps, added by the M0 BF-nonblocking-pty-busyloop fix) ran UNCONDITIONALLY — including on completely idle PTYs — costing ~12ms per pane per call. The new multi-pane pump-all timer (16ms tick) multiplied that fixed floor by pane count: MEASURED 98ms/tick at 8 idle panes (frame budget blown at 2 panes, ~10fps at 8). Caught by an inspect critic that MEASURED the loop with real sessions instead of reading it. Fix: idle fast-path — a LEADING WouldBlock (nothing read this call) returns immediately; the ~1ms retry windows apply only MID-BURST (after bytes were read), preserving the original busyloop-fix's purpose. Re-measured: 19.7µs/tick at 8 panes (~5000×).

## BF-claude-pump-state-change-without-dirty-repaint-001
*category: runtime · severity: medium*

#203: the clear-on-view step (`view.tab_flashes.remove(&seen)`) mutated render-affecting state in the gpui pump WITHOUT setting `dirty = true`. The pump only calls `cx.notify()` (repaint) when `dirty`; the SET path rode the finishing command's pump events (dirty), but the CLEAR path did not — so on an otherwise-idle frame the completion ● badge lingered on the just-viewed tab until an unrelated repaint (a keystroke / PTY output / hover). Asymmetric and directly undercut the feature's "cleared once you view the tab" promise. Caught by the inspect shim critic (my self-review missed it). Fixed: `if view.tab_flashes.remove(&seen).is_some() { dirty = true; }` — mirroring the adjacent `status_flash.tick()` which already sets dirty on a state change.

## BF-claude-raise-only-focus-drops-gpui-keyboard-focus-001
*category: test · severity: high*

#383 implement made drive.swift's `focus` verb a programmatic raise (NSRunningApplication.activate) to fix its content-click misfire — but that removed the mouse-DOWN that gpui uses to set window keyboard focus. A raise/activation does NOT set gpui `window.focus` (only a mouse-down on a `track_focus` element does), so a following `type:` keystroke had no focused view and was dropped — breaking REQ-002's `focus type:…` typing flow. Caught by an inspect critic (traced gpui-0.2.2 source + the app's own headless_drive.rs:158-160 "a test window starts unfocused… injected keystrokes go nowhere"), then CONFIRMED empirically by driving: `focus "type:echo REQ002_probe" enter` → the terminal stayed at a bare prompt (383-req002.png); `clickat:<pane> "type:echo CLICKPROBE" enter` → the echo ran (383-clickprobe.png). Fixed harness-only: `focus` stays activation-only (per D1) and the TYPING flow routes through a `clickat:<pane>` (which posts the mouse-down that focuses the pane); the gpui-focus fact + the two captures documented in the README/spec.

## BF-claude-raw-input-leaks-unbound-cmd-chords-to-program-001
*category: runtime · severity: medium*

The raw-mode keystroke mapper `key_input_from_keystroke` (marley_app app.rs) filtered ctrl/alt but NOT the platform (⌘) modifier, so in alt-screen mode an UNBOUND cmd-chord (⌘C/⌘V/⌘A/⌘S — anything not in the keymap) fell through the keymap-chord check, got mapped to its bare character, and streamed to the program via encode_key→write_bytes. Result: ⌘C in vim/less would INSERT a literal 'c' instead of being a no-op (or a future copy). The cooked path was already safe (key_from_keystroke maps control||platform → Key::Other → ignored), but the new raw path forgot the platform half. Invisible to the gate because the routing lives in the mutants::skip'd + coverage-excluded app.rs shim — only inspect/headed testing catches it. FIX: `if keystroke.modifiers.platform { return None; }` at the top of key_input_from_keystroke (mirroring the cooked filter). Prevention: PR-claude-raw-input-passthrough-must-filter-platform-chords-001.

## BF-claude-references-new-shim-fn-missing-mutants-skip-would-fail-msi-001
*category: test · severity: high*

#317: find_references (a new app.rs flash-dispatcher shim over send_references_request) was missing #[cfg_attr(test, mutants::skip)] while its structural twin go_to_definition has it. app.rs is coverage-excluded but NOT mutation-excluded — gate:5 runs cargo mutants --in-diff over the diffed app.rs lines with no -f restriction, so a NEW shim fn's lines ARE mutated and are kept out of the set ONLY by a per-fn skip. With no test/headless drive invoking find_references (the injection lane drive_references_for_test bypasses it), its 2 mutants (body→(), delete !) would compile and SURVIVE → MSI<100 → gate:5 FAILS at validate/commit. FIX: added the skip mirroring go_to_definition. Caught by inspect critic 3 before the gate. LESSON: every new app.rs render/thread/IO/flash shim needs the mutants::skip; diff the new fn against its sibling to confirm the attribute is present AND hugs the fn (the mutants-skip-detach-trap).

## BF-claude-references-searching-card-naive-latch-strands-on-timeout-001
*category: runtime · severity: high*

#317 find-references: the "Finding references…" searching card was gated on the app-side `references_request.is_some()` latch alone. A timed-out (10s, routine while rust-analyzer indexes) or disconnected references request has its purpose DROPPED by the pump with NO response pushed (lsp_host.rs:322/345), so apply_references_response never runs and the latch is never cleared → the OCCLUDING card is stranded permanently (Esc can't reach it; a fresh ⇧F12 on a dead server fails at send without touching the latch). This is the EXACT #331 has_pending_inlay lesson reintroduced — a latch made VISIBLE (a card) turns the shared naive-latch bug into a real user-facing defect. FIX: added LspHost::has_pending_references() (mirrors has_pending_inlay — scan the pending purposes table, the one source of in-flight truth); the card renders only while a References purpose is genuinely in flight; also cleared references_request in the ⌘⇧A/launcher overlay-clear as a full-reset escape hatch. Caught by inspect critic 2.

## BF-claude-relax-invariant-offpath-accessors-panic-at-empty
*category: runtime · severity: critical*

#234 relaxed the M10 never-empties invariant (the last workspace can close → 0 projects) and guarded the main render with a launcher branch — but the design (D1) analyzed accessor-reachability ONLY inside render. THREE accessors of the active project reached it OFF the guarded render path and panicked at 0 workspaces: (1) [CRITICAL] the 16ms PTY pump (a cx.spawn background loop, app.rs:552→568) derefs active_project() every tick, independent of render; (2) [CRITICAL] close_project_at → persist_grid (app.rs:2061) calls workspace_mut() unconditionally after the last close — fires SYNCHRONOUSLY in the click handler, crashing before the launcher even shows; (3) [HIGH] find_match_rows (app.rs:1154) runs in the render PROLOGUE (before the launcher branch, gated only on find_open). Two adversarial critics (a dedicated panic-safety audit + a correctness critic) found all three; unit tests could not (the pump/persist/render are mutants::skip cov-excluded shims — only a driven run or the audit catches it). Fixed: hoisted the launcher branch to the very top of render, early-returned the pump closure at 0, guarded persist_grid at 0.

## BF-claude-render-anchor-changed-without-updating-inverse-hit-test
*category: validation · severity: medium*

#49 bottom-anchored the terminal render (justify_end on the pane flex_col) but did NOT update pane_grid_pos — the click/pointer→content-cell mapping that is the mathematical INVERSE of the render (from #43's selection). pane_grid_pos assumed the first visible row is painted at the pane TOP (row = start + local_y/cell_h). After justify_end, when content < capacity the visible rows paint against the BOTTOM with (capacity - visible) empty rows ABOVE them, so every click/drag lands top_pad rows too low → drag-select highlights nothing and cmd-C copies the wrong/empty text — in the exact short-content scenario the ticket targets. Non-crashing (row_selection/selected_text are range-clamped) so it fails SILENTLY; no test caught it (both the render and the hit-test are mutants::skip + coverage-excluded shim). Caught only by the inspect critic reading the render AND its inverse together. Fix: extracted a pure row_at(local_row, start, end, capacity) = start + local_row.saturating_sub(capacity - (end-start)) that mirrors the bottom-anchor, tested cov/MSI 100; pane_grid_pos now uses it. Lesson: a rendering transform and its inverse hit-test are a coupled pair — the design that changes the render anchor must change the inverse in lockstep, and inspect should explicitly diff a render-geometry change against every consumer of the inverse mapping (mouse/selection/scroll-to).

## BF-claude-restore-path-skipped-new-field-seeding
*category: validation · severity: high*

#275 inspect F1 (HIGH, found independently by BOTH critics): the boot-restore path (EditorSurface::from_files, the #243 open-set restore) never seeded the new per-file disk snapshot — restored files hit the None→Noop table row at every choke, making the entire external-change feature silently OFF for the steady-state case (restart with the working set open) and letting ⌘S silently clobber agent writes, the exact guarantee the ticket exists for. CLASS: a new per-item field seeded on the INTERACTIVE creation path but forgotten on the RESTORE/deserialize path — restore is a second constructor. Fix: stat before read in the restore arm, snapshot threaded through a RestoredFile row type. Also the implement-phase launcher bug (render edge called active_project() before the launcher branch → panic-on-empty → the virgin-boot headless test wedged the suite via panic-masks-as-hang; the #234 invariant comment sat five lines below the insertion) — guarded on project_count()>0; the REAL regression pin needs activate_window (TestWindow::is_active is hardcoded false, so the edge is dead headless without it).

## BF-claude-restore-reclamp-asymmetry-vs-serialize
*category: validation · severity: low*

#243 session-restore re-clamped the saved editor active index with a naive min() while serialize re-mapped it POSITIONALLY onto the surviving paths — an asymmetry. A file skipped BEFORE the active one on re-read (unreadable/binary/oversized since save) shifted which file is shown active on restore (cosmetic — from_files' min prevented a panic, but the wrong tab focused). Fixed by extracting a shared pure reclamp_active(surviving_orig, active) used by BOTH serialize AND the restore (the restore now enumerates orig indices in its filter_map). Found by the inspect critic + self-review. Class: a lossy transform that re-indexes on BOTH the save and restore sides must use the SAME re-clamp on both — extract it as one shared tested fn.

## BF-claude-root-keyed-state-aliases-duplicate-roots
*category: validation · severity: low*

collapsed_indices (root→index) aliases when two projects share the SAME root string (an 'open same folder twice' state) — both restore collapsed even if one was expanded, since root-keying can't distinguish them. Cosmetic (a project renders collapsed you left expanded), no panic/data-loss, needs the unusual dup-root state. Documented as an inherent tradeoff of root-keying (the whole #245 premise — a stable key that survives reorder); found by the inspect critic. Class: keying persisted per-item state by a non-unique attribute aliases duplicates on restore — dedup the key or add an ordinal if uniqueness isn't enforced.

## BF-claude-rustdoc-private-intra-doc-link
*category: compilation · severity: low*

#265 gate:14 went RED (rustdoc -D warnings) because a PUBLIC item's doc comment ([`Keymap::action_for`], public via re-export) used an intra-doc link to [`chords_unique_scoped`] — pub in its module, but the module is private and the fn is not re-exported, so from the public-docs view the link targets a private item (rustdoc::private_intra_doc_links). Everything else incl. coverage/MSI was green on the same run. Fix: plain backticks + a "(crate-internal, not re-exported)" note. Class: a doc link's validity depends on the CRATE-PUBLIC surface, not module-local visibility.

## BF-claude-scroll-clamp-used-stale-line-count-source-250
*category: runtime · severity: low*

#250 faithful renderer: the editor render was changed to window over `buffer.len_lines()` (ropey), but the wheel/scroll handler still clamped `cv.scroll` via `scroll_code(.., cv.lines.len())` (the split('\n') count frozen at open). Equal for \n/CRLF files (cosmetic), but ropey's unicode_lines default also breaks on bare CR/FF/VT/NEL/LS/PS → the bottom lines become unscrollable, AND once #251 edits the buffer the stale cv.lines count breaks the clamp for every edited file (latent MEDIUM). Caught by an inspect critic (not shipped). Fixed: the handler now clamps against active_buffer().len_lines() (the same source the render uses), falling back to cv.lines.len() for a bufferless read-only pane.

## BF-claude-second-construction-path-defeats-label-change
*category: runtime · severity: medium*

#240 set the editor tab's rail label to a stable "Editor" by changing Project::open_or_switch_code (tabs.rs). But Tab::code has TWO callers — the other is the session-restore arm TabLayout::Code(path) in the app shim (app.rs:926-936), which still derived the title from pb.file_name(). So a restored editor tab reverted to the file name after a quit-and-reopen — the exact frozen-name state #240 removes. My self-review grepped for .title READERS (found none) but not the other Tab::code WRITER. The adversarial inspect critic caught it. FIX: promoted EDITOR_TAB_TITLE to pub(crate) and used it at the restore site too. Class: a "change how X is labeled/constructed" edit must update ALL construction sites — grep for every caller of the constructor, not just the one path in view. The restore/deserialize path is a common second site that unit tests + a live driven capture (fresh session) don't exercise.

## BF-claude-section-menu-opens-keyboard-dead-under-editor-overlay-001
*category: runtime · severity: medium*

#393 section ＋ menu: `open_section_menu` closed only palette/finder/history/find before opening, but the key router is one linear function where ~12 editor-overlay arms (naming_workflow, def_picker, open_references, code_action_menu, signature_card, open_symbols, open_search, open_problems, …) run BEFORE the context_menu arm and early-return. Those pickers are single CENTERED cards (no full-window inset_0 scrim — only the context_menu has the backdrop), so the LEFT-rail ＋ is NOT occluded → open a centered editor picker, then click a rail ＋: the section menu renders but the keyboard belongs to the picker (the naming_workflow case leaks typed chars into the hidden draft). Exactly the class the #181 agent launcher (the sibling mouse-opened modal) already fixed at #229/#312/#317/#323/#324/#325/#326/#327. Fixed by extracting `close_transient_overlays()` (the launcher's 13 closers, one source of truth) and calling it from BOTH the launcher-open and open_section_menu. Found by inspect critic 2 (which verified realism via the pickers' geometry).

## BF-claude-section-menu-splitfocused-skips-persist-on-noterminal-earlyreturn-001
*category: runtime · severity: medium*

#393 section ＋ menu: the `SectionAction::SplitFocused(axis)` dispatch arm delegated ALL persistence to `split_focused_pane`, which persists internally EXCEPT on its #392 no-terminal early-return (`if try_workspace_mut().is_none() { return; }` skips `persist_grid()`). So a CROSS-PROJECT Panes＋ Split on a no-terminal project runs switch_project(p) + sync_active_project (in-memory) then returns without persisting → the active-project switch is lost on relaunch. Same class as the #387-F1 finding (a cross-project affordance must persist the switch even when the verb no-ops). Latent today (no-terminal is guard-forbidden until #395) but the code already carries the #392 guard and #395 is the next ticket; it also made dispatch_section_verb's own "applies the persist rule uniformly" docstring a lie. Fixed: the arm now does `self.split_focused_pane(axis); if project_changed { self.persist_grid(); }` (mirrors the OpenFile/OpenFileSplit arms). Found by inspect critic 1.

## BF-claude-section-plus-openfile-skips-persist-001
*category: runtime · severity: medium*

#387 dispatch_section_action's OpenFile arm switched the active project (to act on the ＋'s row-project) but did NOT persist_grid the switch, unlike the #174 Tab/Pane rail rows and the sibling NewTerminal/OpenCockpit arms. active_project IS persisted state (serialize_shell writes it first, grid_layout.rs:330; restored at boot app.rs:2139), and the finder's own persist fires only on ⌘↵-open — so clicking Editor＋ on a non-active project then cancelling the finder then relaunching reverted the active project (self-healing on any later persist, but a real divergence). Caught by inspect (my trace + critic A). Fix: added `if project_changed { self.persist_grid(); }` to the OpenFile arm. Class: a rail affordance that switches the active project must persist the switch even when its action opens a transient modal rather than mutating durable state.

## BF-claude-sed-range-cannot-close-same-line-html-comment
*category: test · severity: high*

enforce-warp-reference.sh stripped HTML-comment guidance with `sed '/<!--/,/-->/d'` to test if a spec's `## Reference (§20)` section is empty. In a POSIX sed RANGE, the closing `/-->/` address is only tested on lines AFTER the one that matched the opener — so a comment opened AND closed on the SAME line (`<!-- x -->`) never closes on that line, and the range deletes that line plus everything down to the next `-->` (or EOF), swallowing legit prose. Result: a spec whose Reference section is a one-line comment ABOVE real prose (or prose with an inline comment) was FALSELY BLOCKED from committing. Bites when an editor/prettier collapses the multi-line guidance comment to one line, or an author inlines a comment. Inherent sed semantics — reproduces identically on BSD + GNU sed. Caught by the inspect critic running the hook (my self-review + smoke used only multi-line comments, so missed it). Fix: strip same-line comments FIRST (`sed 's/<!--.*-->//g'`) then the multi-line block. Class: a sed `/open/,/close/d` range cannot delete a same-line open+close — pre-strip same-line matches before range-deleting multi-line blocks.

## BF-claude-selection-home-tracker-severed-by-none-frame-001
*category: runtime · severity: high*

TICKET-397 inspect HIGH (found independently by two critics): the selection park/restore tracker overwrote last_editor_view with None whenever a non-editor surface (terminal pane, cockpit) took focus, so the everyday tour tab-view → terminal → split-twin never parked the outgoing cursors: the twin edited with the sibling's cursor set, the twin's own parked set was stranded, and the eventual direct twin→twin switch restored None → the caret-0 seed destroyed a real position (caret teleport to 1:1). Stale IME marked spans leaked through the same hop. Fixed: the tracker is STICKY across None frames; the park runs on EVERY view change reading the outgoing set from the OUTGOING view's instance; rows are birth-seeded with a caret-0 park so the focus-in restore is total; the restore clears the incoming row's stale composition.

## BF-claude-session-cwd-boot-pty-reuse-ignores-saved-cwd-001
*category: runtime · severity: medium*

#205: the restore flow reused the pre-spawned boot PTY (spawned in the launch cwd) for the launch project's FIRST terminal UNCONDITIONALLY (`boot_session.take_if(|_| root == project_root)`), so that pane's saved cwd was silently ignored — and that's the MOST COMMON case (one window launched from the project dir, the primary terminal `cd`'d into a subdir): the whole feature no-op'd for it. I had DESIGNED this as a "documented limitation" (the boot-PTY-reuse keeps the boot cwd), under-weighting that it's the common case, not an edge. Caught by the inspect shim critic. Fixed: gate the reuse on `cwd_or_root(gl.cwds.first(), &root) == root` — reuse the boot PTY only when the saved cwd resolves back to the launch root; a `cd`'d subdir spawns fresh THERE (the boot PTY is dropped, a minor startup cost). ALSO fixed a sibling MED: `cwd_or_root` used `Path::new(c).exists()` (true for a FILE) where `is_dir()` is correct — a cwd that became a file would pass to spawn_session_in → the PTY spawn fails → the pane/tab is silently dropped; `is_dir()` cleanly redirects a now-file/broken cwd to root.

## BF-claude-session-end-error-launders-to-done-001
*category: runtime · severity: high*

#368 projection: a `session-end` seat_events row carrying `state:"error"` projected to a bare `SessionEvent::Ended` (which never reads the row's state), so the marley_fleet reducer — seeing a first-seen/auto-vivified seat that is NOT already in Error — set it to cleanly `Done`. A crashed seat thus rendered as a clean exit, violating the safety-critical "a seat that died on an error must never read as `Idle` or cleanly `Done`" invariant. Amplified by the cursor-only reconnect model: if the prior `api-error` (which correctly → Error) sat before the persisted cursor and the `session-end` after, replay never saw the Error at all. Root cause: the projection mapped a rich wire row onto a NARROWER event vocabulary through non-descriptor-carrying events (Ended/QuestionRaised/Heartbeat), silently discarding the row's own state/descriptor. Fix: project_row now returns Vec<SessionEvent>, routing every descriptor-bearing kind through an Upsert(full descriptor) and honoring a terminal error state — session-end+error → Upsert(Error) (retained), clean end → Ended→Done. Caught by an adversarial projection critic; the naive test (session-end with no state) had missed it.

## BF-claude-single-str-replace-sanitizer-reconstitutes-split-tokens
*category: security · severity: medium*

The bracketed-paste injection guard used a single `text.replace("\x1b[201~", "")` to strip embedded end-markers before wrapping a paste in ESC[200~..ESC[201~. `str::replace` does ONE non-overlapping left-to-right pass and does NOT re-scan its output, so a split marker reconstitutes: input `ESC[20` + `ESC[201~` + `1~` → removing the middle marker splices the neighbours into a fresh `ESC[201~`. That interior end-marker closes the bracket early, and the bytes after it run as typed input — a working paste-injection (clipboard `ESC[20 ESC[201~ 1~ rm -rf ~\n` → `rm -rf ~` submitted). Caught by the inspect critic with a PoC probe. FIX: loop until stable — `while safe.contains("\x1b[201~") { safe = safe.replace("\x1b[201~", ""); }` (each pass strictly shrinks → terminates; verified 0 residual on split/nested/adjacent/1000x-stress inputs). Also MED: cargo-mutants generates only whole-fn-return mutants on paste_bytes (none on the strip), so MSI 100 is blind to the guard — a hand-written split-marker structural regression test is required (the gate can't force it).

## BF-claude-skip-detach-pump-fleet-live-001
*category: test · severity: high*

#376 inspect F1 (HIGH, found by 2 critics): pump_fleet_live was inserted between pump_mcp_host's doc comment + #[cfg_attr(test, mutants::skip)] and its fn — the 5th strike of the recorded mutants-skip detach trap. Rust bound BOTH doc blocks + BOTH skip attrs to the new fn, leaving pump_mcp_host unskipped; cargo mutants --list proved an unkillable `replace pump_mcp_host with ()` (MSI RED). Sharper edge this time: gates.sh --diff uses --in-diff, and the detached mutant's span sat OUTSIDE the changed lines — it would have slipped this commit and detonated on a later unrelated run, misattributed. Fix: relocated the new fn as a self-contained doc+skip+fn block ABOVE the neighbor, restoring adjacency; --list re-run showed 0 mutants for both pumps. The trap fires on INSERT-BETWEEN-ATTR-AND-FN, not just insert-above-fn.

## BF-claude-specified-keybinding-cmd-r-unimplemented-only-click-shipped
*category: validation · severity: medium*

#46 promised re-run via BOTH a ↻ click AND cmd-R (in SPEC-app-shell R50, the pipeline REQ-004, the requirement table, AND the ticket TITLE "click / cmd-R"), but the implement phase wired ONLY the ↻ click — no keymap binding, no dispatch arm, no most-recent-finished-block resolver. The gap was invisible to every automated gate: the whole app.rs render/dispatch shim is #[cfg_attr(test, mutants::skip)] + coverage-excluded, and REQ-004's verification is a masked chad-visual, so cov/MSI/tests all stayed GREEN on the pure rerun_command alone while a titled feature was absent. Caught only by the inspect critic cross-reading the spec's promises against the keymap's actual bindings (grep: no "r" chord, no rerun dispatch arm, no resolver). Fix: implemented cmd-R fully — a pure BlockList::last_rerunnable() (tested), a cmd-r→"rerun-last" keybinding (+ its action_for assertion), and a #40-guarded dispatch arm. Lesson: when a ticket names TWO trigger surfaces for one action (a click AND a keyboard shortcut), and only the pure decision fn is gate-covered, the second (shim-only) trigger can silently not-ship — the masked shim hides it. An inspect step that diffs the spec's named triggers against the actual keymap/dispatch wiring is the only thing that catches a missing-but-specified shim trigger.

## BF-claude-ssh-command-leading-dash-host-is-option-smuggling-injection-001
*category: security · severity: high*

#83's first cut of marley_remote::ssh_command built the argv ["ssh", ("-p", port)?, dest] with NO guard on a destination that starts with `-`. A target like `-oProxyCommand=evil` parsed to SshTarget{host:"-oProxyCommand=evil"} and produced ["ssh","-oProxyCommand=evil"] — which real OpenSSH RE-PARSES AS AN OPTION (not a hostname), because ssh treats a leading-dash positional arg as a flag. Confirmed against OpenSSH_10.2p1: the dest was consumed as an option (ssh printed usage) and the ProxyCommand mechanism is real (a sentinel file was touched when a host arg was also present). This directly falsified the crate's own doc invariant ("a host or user string can never inject flags or commands"). NOT a one-shot RCE from today's exact single-arg output (when the sole dest is eaten as an option, no host remains → ssh exits at usage without connecting), but a BROKEN SECURITY INVARIANT + LATENT RCE: it goes live the instant a second positional/remote-command arg is added (the obvious next ssh-pane feature) — the CVE-2017-1000117 (git) / option-injection class. FIX (both, defense-in-depth): parse_ssh_target rejects a leading-`-` host OR user (→None); ssh_command inserts `--` before the destination so ssh stops parsing options (also defends a hand-constructed SshTarget that bypassed the parser). Both verified against real ssh.

## BF-claude-stale-selection-after-undo-panics-the-rope-001
*category: runtime · severity: high*

Marley #296. Buffer::edit_at_selections WRITES self.selection, but undo()/redo() restore only the TEXT — they do not maintain the selection set. So after a multi-caret edit followed by ⌘Z, buffer.selection() holds offsets PAST EOF ([3,7] in a 3-char rope). Feeding those straight back into edit_at_selections handed ropey an out-of-bounds range → PANIC at rope.rs:952 "end is out of bounds". edit_at_selections is the ONLY pub fn that accepts offsets it did not itself produce and forwards them to the rope, while its sibling set_selection already clamped defensively — the inconsistency was the tell. FIX: edit_at_selections CLAMPS + canonicalizes its INCOMING set before use; regression test drives multi-edit → undo → edit and asserts no panic + in-bounds result.

## BF-claude-stale-ui-match-set-feeds-edit-offsets-across-frames
*category: runtime · severity: high*

#272 inspect (HIGH, probe-EXECUTED): the find bar's match set refreshed only in the render path, but key handlers run per event — two keys landing between frames (Enter autorepeat live; the headless lane dispatches ALL keystrokes then parks once) made the second Replace consume PRE-EDIT offsets: Buffer::edit → rope.remove out-of-range PANICKED, and the in-bounds variant silently ate user text ("aaa bbb" → "xbb"). The #268-class render-memo staleness is COSMETIC for highlights but CORRUPTING the moment the cached ranges feed edits. Fix: the handler re-runs the memo refresh at its top (no-op on an unchanged (nonce, version, query) key — zero fresh-path cost). Rule: any cached range set that an EDIT path consumes must be revalidated IN the handler, not left to the next frame.

## BF-claude-static-command-id-collision-swallowed-the-verb-001
*category: validation · severity: high*

#399: NAME_PANE_ID was drafted as CommandId(20) after a grep for "CommandId([0-9])," / "CommandId(1[0-9])," — a pattern that structurally could not match the 20-29 block cockpit_commands() already mints. action_for_command maps 20 → "toggle-find-regex", and that resolver is the FIRST arm of the palette dispatch chain, so the new "Name Pane…" verb was dead code that silently flipped the find bar's regex mode (and duplicated id 20 in the command list). No existing test could catch it: every_cockpit_command_resolves_to_a_verb checks only the forward direction, and all the new entry points are mutants::skip-masked. Caught by two independent inspect critics tracing the dispatch chain end-to-end. Fix: CommandId(30) + a Phase-4 reverse-guard unit (specially-dispatched statics must resolve to NO verb) + a no-duplicate-ids pin.

## BF-claude-subagent-research-paralysis-on-slow-compile-dep-001
*category: compilation · severity: medium*

marley_app (the gpui finale): THREE successive implement subagents, each told to build the crate + "confirm the gpui 0.2.2 API (cx.spawn/on_key_down/open_window/Keystroke)", spent their ENTIRE run compiling gpui + reading its source to research signatures and NEVER wrote a single file — each killed after many idle cycles with the crate dir still absent (their last states: "confirm the fluent on_key_down..."/"start by reading the essential context files..."). Root cause: a subagent instructed to VERIFY an API on a slow-to-compile dep (gpui ~minutes/build), without a hard write-first constraint, enters an unbounded research loop — each cargo check recompiles, and it keeps finding one more thing to confirm. Fix: I stopped them and wrote the crate MYSELF against the spike's PROVEN in-repo gpui calls (crates/marley_spike/src/app.rs) using WRITE-then-cargo-check-iterate — cargo check passed 0 errors FIRST TRY; every gpui surface the subagents feared compiled as written. The exemplar + a small targeted signature grep (ThemeColors/SessionOptions/Buffer) was all that was needed; the research loop was pure waste. (The pure modules needed zero gpui knowledge and could have been written in seconds — the subagents never even got to them.)

## BF-claude-svg-asset-not-registered-renders-blank
*category: runtime · severity: high*

#261 CRITICAL (inspect critic): 10 new SVG assets were wired through icons.rs icon_path but NOT added to the app's AssetSource (Assets::load — a STATIC per-file include_bytes! match, not a dir embed). gpui renders a missing asset as a silent 12px blank (logs + keeps the layout box) — compile green, 368 tests green, clippy green, icon invisible at runtime. The notes even claimed a nonexistent "include_bytes! embeds the dir" compile proof. Fixed: 10 match arms + a permanent assets_serve_every_icon_variant guard walking every Icon through icon_path→Assets::load.

## BF-claude-sync-network-fetch-on-gpui-main-thread-freezes-ui-001
*category: performance · severity: medium*

#69's first cut re-fetched the forge sprint SYNCHRONOUSLY in the ⌘⇧F dispatch (on the gpui main thread). The forge is ~2.7s per tools/call (measured via curl) and current_sprint() does 2 calls → the open FROZE the UI ~5s: the overlay never appeared within the self-test's capture window because forge_open only flips AFTER the blocking fetch returns. Both the green gate AND the inspect critic ACCEPTED the sync design (the critic's MED estimated <100ms on localhost) — only the LIVE self-test exposed the freeze. FIX: a BACKGROUND-THREAD fetch — refresh_forge spawns a std::thread running current_sprint(), hands the result back through an mpsc::Receiver (forge_pending) that the 16ms pump try_recv-polls; the overlay opens instantly with a "loading sprint…" state (forge_pending.is_some()) and fills in when ready. ForgeClient gained #[derive(Clone)] so the thread owns a copy. This turned out to be the ticket's actual intent ("refresh + loading/… states"). Category: a UX/perf freeze, not a logic bug — invisible to cov/MSI (app.rs masked) and to a static critic; the self-test is the only oracle for UI responsiveness.

## BF-claude-terminal-arm-reachable-from-editor-tab
*category: validation · severity: high*

#278 inspect F1 (HIGH, code-traced): the new readline arm in the terminal key region was reachable on EDITOR/COCKPIT tabs — focused_terminal() falls back to the first terminal pane (#71) and the #251 editor arm's !control gate deliberately lets ⌃-chords fall through, so ⌃K/⌃U/⌃Y edited the HIDDEN pane's prompt buffer (visible corruption on return — the #283 hidden-prompt class upgraded from raw C0 bytes to Marley-visible state). CLASS: any NEW arm in the terminal region that MUTATES prompt state must gate on the active tab actually BEING a terminal (active_tab().grid().is_some()) — the (alt_screen, command_running) read alone describes the hidden fallback pane, not the visible surface. Fix: the grid().is_some() gate; non-terminal tabs revert byte-identically to the raw route; headless editor-tab negative added at validate.

## BF-claude-terminal-echo-dropped-during-transient-state-001
*category: runtime · severity: medium*

#378 inspect F2 (MED): delivery echoes observed while the send's receipt was in flight (phase=Sending) were DROPPED on the theory "the next echo heals it (the monotone skip is legal)" — but dispatch-started is the TERMINAL echo kind, so a start observed mid-flight had no later healer: the chip stuck at "deposited" forever while the seat ran the brief. The heals-later justification was factually incomplete for the last event in a sequence. Fixed by BUFFERING (max-join into DispatchRecord.pending) while Sending and consuming the buffer at receipt time (Deposited.observe(pending)) — nothing renders pre-receipt, so the no-forgery doctrine holds. Sibling F1 (both critics): the crate-side echo queue was an unbounded Vec whose only drain parks in the launcher state — became a per-seat max-join map, bounded by fleet size, lossless because the consumer folds the same monotone max.

## BF-claude-then-some-eager-arg-underflow-panic-001
*category: runtime · severity: medium*

TICKET-020 (layout.rs neighbor): `bool::then_some(x)` evaluates its argument `x` EAGERLY (unlike `bool::then(|| x)` which is lazy). The preceding-sibling index was written `(index >= 1).then_some(index - 1)` — but `index - 1` is computed regardless of the `index >= 1` guard, so for `index == 0` (a pane at the start of a Split, e.g. `neighbor(A, Left)` at a matching-axis ancestor) `0usize - 1` underflowed → panic on a reachable input path (a §14 violation). Caught by the neighbor_boundary_rule unit test (panicked at layout.rs:206). Fixed with `index.checked_sub(1)` (returns the preceding index, or None at the edge, with no eager subtraction). NOTE: clippy's `unnecessary_lazy_evaluations` lint actively pushes `.then(|| index - 1)` → `.then_some(index - 1)`, which reintroduces the bug — `checked_sub` sidesteps both.

## BF-claude-toggle-tests-too-shallow-missed-recurse-guard-mutants
*category: test · severity: medium*

The 4 survivors: cursor `*cursor += 1` → `*= 1`, the recurse guard `is_dir && !collapsed && toggle_at` with `&&`→`||` (×2) and dropping the `!`. My toggle tests only hit index 0 or file rows, so they never forced the cursor to ADVANCE through multiple nodes nor exercised the collapse-aware recursion guard. A directory-tree toggle-by-visible-index needs a test that (a) toggles a DEEP row (cursor must advance past earlier nodes) and (b) collapses a MID dir then toggles a LATER visible index (which must skip the hidden subtree). The inspect critic had spelled out this EXACT sequence; I under-tested it with shallower cases. Fixed by adding `toggle_recurses_and_respects_collapse` → MSI 100 (23/0). Lesson: when a critic hands you the precise mutation-killing sequence, write THAT sequence, don't approximate.

## BF-claude-tree-walk-trusts-kind-admits-missing-node-phantom-001
*category: validation · severity: low*

#340 bracket-match: `delims_at` (syntax/src/lib.rs) matched the delimiter pair on `child.kind()` strings alone. For an unclosed `(` (`fn f( {}`), tree-sitter's error recovery inserts a zero-width MISSING `)` node whose `kind()` IS `")"` — so `is_delim_pair("(", ")")` passed and the walk returned `Some((4..5, 5..5))`: a lone `(` tinted (0.10) and ⌘⇧\ jumped the caret onto whitespace at the phantom position. Contradicted the fn's "returns None" contract and the spec's explicit cut of unmatched-bracket error tinting. Root cause: a `kind()`-string match is not sufficient — tree-sitter MISSING nodes have a REAL `kind()` but zero width. Fixed with one guard: `if first.is_missing() || last.is_missing() { return None }`. Probe-confirmed the behavior flipped (`Some((4..5,5..5))` → `None`) with real pairs unaffected.

## BF-claude-two-match-engines-disagreed-on-overlap-and-grew-the-selection-001
*category: runtime · severity: high*

#298's REQ-007 said ⌘D and ⌘⇧L must AGREE on what an occurrence IS. I unified their CASE (a `fold` flag on find_all) and left their OVERLAP semantics divergent — which is the half that corrupts data. `next_occurrence` probes EVERY index, so it returns SELF-OVERLAPPING matches ("aa" at 0 AND at 1 in "aaa"); `find_all` advances `i += ndl.len()`, so it never does. ⌘D's exhausted guard tested EQUALITY (`start == && end ==`), but SelectionSet's invariant merges on OVERLAP — so a match overlapping a held member without equalling it sailed through the guard, was pushed, and was UNIONED into that member by from_selections. The user's "aa" silently became "aaa": text they never selected, AND the needle for every subsequent press. Reachable with the KEYBOARD ALONE (⇧→⇧→ over an indent run selects two spaces, and "  " self-overlaps inside a 4-space indent) — no mouse, no exotic input. Payload: typing over the resulting ⌘D set yields "XbX" where the user's two selections should give "abXbX" — it eats two characters they never targeted. A MOTION, so ⌘Z cannot restore the corrupted gesture state. FIX: ONE engine (find_all) for both gestures, plus a candidate must survive the invariant AS ITSELF or it is not addable. `next_occurrence` then had no production caller and was DELETED (§0), same as its sibling select_next_match.

## BF-claude-unbuildable-test-fixture-for-leaf-split-algebra-001
*category: validation · severity: medium*

TICKET-020: the design notes' nested-2x2 PaneGroup test fixture Split(H)[Split(V)[A,B],Split(V)[C,D]] was UN-BUILDABLE via the public API. PaneGroup::split only ever replaces a LEAF with a 2-child Split (SPEC-app-shell R8) — it never wraps an existing Split node — so you cannot construct that particular nesting by hand-assembling the intended shape; the real build single(A)→split(A,B,H)→split(A,C,V)→split(B,D,V) yields Split(H)[Split(V)[A,C],Split(V)[B,D]] (a DIFFERENT leaf arrangement: panes [A,C,B,D], grid A|B / C|D). Caught at inspect by an empirical probe (built the tree via the API + printed it) before writing the validate test — had the test hand-asserted on the un-buildable tree it would have used wrong neighbor expectations. NOT a code bug (the algebra + the R12 neighbor rule are correct); a latent test-fixture error. Fix: validate builds the fixture by CALLING split() + asserts the R12-correct neighbor results for the ACTUAL tree (neighbor(A,Right)=B, A-Down=C, B-Left=C, D-Left=C via the last-child-for-Left rule, A-Up=None).

## BF-claude-unclamped-remainder-region-reverses-layout-under-min-window-001
*category: validation · severity: medium*

TICKET-024: region_widths computed center = window − docks UNCLAMPED, and SPEC R31 spec'd exactly that — below 440pt window width (reachable by ordinary resize; the gpui window sets no minimum) the center went negative, the right dock's x = left+center regressed INSIDE the left dock (overlap), and pane_rects with negative width moved child origins leftward (reversed tiling). No panic (taffy clamps, gpui culls empty bounds) — silent wrong-geometry, the worst kind. Caught by an inspect critic probing the pure math + reading taffy/gpui negative-size behavior. Fix: clamp the remainder at 0.0 with the shrunken-window case as a REACHABLE tested arm (not dead defense: the boundary test kills clamp-deletion and max→min mutants), and co-amend the spec — the spec itself mandated the bug, so the fix was spec+code together.

## BF-claude-undo-granularity-comment-cited-the-wrong-coalesce-path-001
*category: validation · severity: medium*

#307's Tab arm keeps a LONE bare-caret pad ungrouped (grouping it would split one ⌘Z into two). The decision is correct, but the comment, the Phase-1 finding, and the spec's D3 all justified it with the WRONG mechanism: "begin_undo_group pins cursor_anchored: false, and coalesces_into requires it on BOTH sides, so grouping would stop the pad coalescing with following typing." That chain never runs on the path it explains. `coalesces_into` is called from exactly one place — `undo::end_group` — so it governs GROUP-into-GROUP absorption and is only reachable when a group was opened. A following keystroke at ONE cursor opens no group (`Buffer::edit_ranges_restoring_placing` takes its single-member ungrouped fast path), so the typed char reaches `UndoHistory::record`, whose coalesce guard is `group.sel_before.is_none() && group.records.len() == 1`; `cursor_anchored` is never consulted there. Grouping the pad sets `sel_before: Some(..)` and THAT is what blocks the coalesce. Two independent inspect critics found this; one proved it with a counterfactual (ungrouped → one ⌘Z yields "a"; grouped → one ⌘Z yields "a   "). The shipped precedent states it correctly at buffer.rs's single-member fast path, and I cited the wrong sibling. It matters beyond pedantry: if someone later "cleans up" begin_undo_group to pass cursor_anchored: true, the comment reads as a licence to group the lone pad and ⌘Z granularity silently regresses. This was the THIRD "right decision, wrong reason" in one ticket.

## BF-claude-undo-group-absorbed-a-typed-char-and-destroyed-text-001
*category: runtime · severity: critical*

#299: toggle a comment at N cursors, type ONE character, press ⌘Z — and the editor DELETES THE USER'S CODE, unrecoverably. `"foo\nbar"` + 2 mid-line carets → ⌘/ → `"// foo\n// bar"` → type X → `"// foXo\n// baXr"` → ⌘Z → **`"oXo\naXr"`**. The `f` and the `b` are gone, and the undo stack is now EMPTY so a second ⌘Z does nothing. ROOT CAUSE (undo.rs `coalesces_into`, shipped by #297, first REACHED by #299): `end_group` offers each closing group to `coalesces_into`, which decides "is this a typed run continuing?" from STRUCTURE — bracketed, record count == cursor count, selections line up, every record a pure insert whose NEW insert is 1 char. The ⌘/ toggle satisfies every one of those while being nothing of the sort: its records are anchored at the MIN-INDENT COLUMN, not at the cursors. So the typed char is appended onto a record whose `at` is somewhere else, and `Buffer::undo` deletes `at..at + inserted.chars().count()` — the WRONG characters. The false premise is stated VERBATIM in the fn's own doc: "Because the cursors did not move, each new insert lands precisely at the end of its own record's text, so appending char-wise is contiguous by construction — no offset arithmetic is needed or wanted." True for a typed run; false for the first bracketed group whose records are not cursor-anchored. Its single-cursor twin `record()` CHECKS contiguity (`last.at + last.inserted.chars().count() == rec.at`); `coalesces_into` only ARGUED it. Enter is affected identically (a 1-char `"\n"` insert). Found independently by THREE of four critics and by me; the FOURTH read the same code and declared it clean, having misread the guard as testing `old.inserted` when it tests `new.inserted`. FIX: `UndoGroup.cursor_anchored: bool` — set true ONLY by the N-cursor typed-insert path, false by every hand-bracketed group (Tab, ⌘/, replace-all); `coalesces_into` requires it. Kills the whole class (#307 multi-cursor Tab, #300 line-move, #303 delete-word would all have walked into it).

## BF-claude-undo-restored-internal-edit-ranges-as-user-selections-001
*category: runtime · severity: high*

Marley #297. Backspace at N cursors turns each bare caret into a one-char CONSUMING range so it can delete backwards. Those internal ranges were handed to edit_at_selections, which recorded them as the undo group's sel_before — so ⌘Z restored the user's TEXT but gave them back SELECTIONS they never made (the consuming ranges), and the very next keystroke REPLACED those selections, eating the text the undo had just restored. Reproduced: "abcdef" with carets at 2 and 5 → ⌫ → ⌘Z → type "X" → "aXcdXf" instead of "abXcdeXf". The single-cursor path was correct (no undo group is opened for one member), so this was a 1-vs-N divergence in a DATA-LOSING direction — the worst kind. Two independent critics found it. FIX: backspace moved into the crate as Buffer::backspace_at_selections, built on a new edit_ranges_restoring(restore, targets, f, origin) that names BOTH halves explicitly — the ranges to edit, and the cursors to restore. A first attempt instead took the buffer's CURRENT selection as the snapshot; a critic's regression test caught that it silently restored one cursor when the caller edited at two. The seam now states the restore set rather than guessing it.

## BF-claude-view-to-instance-field-kept-per-copy-default-001
*category: runtime · severity: high*

TICKET-397 inspect HIGH: split_file_pane seeded the newly-instance-owned #275 disk snapshot with None "as the pre-#397 split pane was". Valid when the field was per-VIEW; poison once it became the ONE instance's: a split-FIRST open left disk=None, and the later tab open's own stat was discarded on the resolve HIT (make never runs) — every #275 choke mapped None to Noop, so an agent's external write would be silently clobbered by ⌘S with no Changed banner. Fixed: stat-before-read at the split birth too, so the shared instance is born tracked regardless of which view opens first.

## BF-claude-wall-admit-vs-parsed-host-divergence-001
*category: security · severity: high*

forge_web_base (#404, caught at inspect pre-merge): the loopback wall's hand-parse reads the LAST-colon tail of an authority as a port (is_loopback_authority rsplit_once(':')), while the WHATWG parser reads a last-@ as userinfo — so `http://127.0.0.1:pass@evil.com/` was wall-ADMITTED (hand-parsed host 127.0.0.1) yet Url::parse takes host evil.com, and the origin derivation returned Some("http://evil.com"): a NON-loopback browser base from a hostile .mcp.json. Empirically confirmed with 4 shapes (127.0.0.1:pass@, :8080@, localhost:@, [::1]:pass@ → all Some("http://evil.com")). Fix: web_origin_of re-asserts loopback on the PARSED host (url::Host match: Domain=="localhost" | Ipv4/Ipv6 is_loopback) before serializing — the check sits on the value the origin is BUILT from, closing the divergence for every shape (all 4 → None post-fix; legit origins unchanged). Wall-side numeric-port validation left as a recorded hardening candidate (the MCP-client arms are inert: the raw-socket adapter can't connect to such an authority).

## BF-claude-wholeline-delete-empty-buffer-returns-zero-width-not-none-001
*category: runtime · severity: low*

#303 inspect C1-LOW (correctness critic): `delete_range_for(WholeLine, ...)` on a truly EMPTY buffer returned `Some(0..0)` (a zero-width range) instead of `None`, breaking the symmetry with the other four ops (WordLeft/Right/ToLineStart/ToLineEnd all return None at their no-op edge). Because `apply_delete` opens an undo group directly (begin_undo_group → edit → end_undo_group, NOT the edit_ranges_restoring no-op guard), a zero-width edit would open an undo group over a no-op `edit(0..0,"")` — a dead ⌘Z step recorded in an empty editor (reachable: a new empty file, or a file whose content was fully deleted). Not a panic, not incorrect text, but a real asymmetry + a dead undo unit. Fixed by guarding the only-line branch: `(start < len).then(|| ...)` → None on an empty buffer. Missed by my own throwaway (I discarded the WholeLine-on-empty return with `let _ =` instead of asserting it); caught by the independent correctness critic — the value of a second lens even after a passing self-check.

## BF-claude-width-in-chars-vs-cells-strands-cjk-tails-001
*category: runtime · severity: critical*

#336 horizontal scroll. The per-frame scrollable-width measure used `layout.display.chars().count()` while every consumer of scroll_x — and the caret's own position via `col_of_offset`/`col_starts` — works in display CELLS. Chars and cells are a DIFFERENT DOMAIN: `char_width` (UAX#11) makes a CJK glyph 2 cells and a combining mark 0. Measured on the real code: cjk_pure 9 chars vs 18 cols; cjk 21 vs 31; emoji 14 vs 17; combining 6 vs 3 (over-counts). End-to-end trace: 100 CJK chars = 200 cells → content_px reported 800px while caret_px reported 1600px → the clamp capped scroll_x at 416 and stranded the caret 784px OFF-SCREEN PERMANENTLY (every later follow recomputes the same wrong max). That is the ticket's OWN defect — an unreachable line tail — reintroduced for exactly the lines that need it most. It also defeated the module's own documented invariant ("the caret's row is always in the max, or follow could ask for a scroll clamp then takes away") because caret_w used the same broken measure. ASCII and tabs agree EXACTLY, which is why it survives casual testing. Second half of the same bug: the width block measured `line_layout` (no inlays) while the row renders `line_layout_with_inlays`, so a trailing #331 inlay hint (14 vs 19 cols) was unscrollable. BOTH inspect critics found this independently — one from the math, one from the render. Fix: `LineLayout::display_cols()` (walks `display` via char_width; deliberately NOT `col_starts.last()`, which stops before an EOL phantom) + measure the SAME inlay-aware layout the render draws. THE POINT: `AD-claude-editor-offset-column-model-001` already declared code_view the owner of the column domain, and naming that invariant did not stop the violation — it happened in a coverage-excluded, mutants::skip'd shim where no pure test could see it.

## BF-claude-workspace-states-hashmap-order-assumed-001
*category: runtime · severity: medium*

The #292 "rerun-last-failed" dispatch scanned workspace().states().find_map(...) and my code + comment assumed "ascending PaneId" order. But states() = self.panes.iter() where panes is a HashMap<PaneId, PaneState> (RandomState) → iteration order is arbitrary + per-process-randomized. With multiple terminal panes each holding a failure, find_map re-ran an ARBITRARY pane's failure (could differ across runs); the comment was factually false. My inspect self-review MISSED it (assumed states() was ordered); the background critic caught it. Single-terminal (the drive) was correct, so it was gate-green + drive-proven despite the multi-pane bug. Fix: collect candidates (skipping is_command_running panes), sort_by_key(id.0), take first → deterministic lowest-PaneId idle failed pane. Note: sibling #289 uses the same states() but is deliberately order-INDEPENDENT (union + sort/dedup).

## BF-claude-wry-ipc-closure-strong-rc-cycle-001
*category: runtime · severity: medium*

M29 #402 inspect F2 (critic-found, pre-ship). The wry IPC handler closure captured a strong Rc<Shared> while wry STORES that closure inside the WebView it builds (wry-0.56.0 wkwebview/mod.rs:144 ipc_handler_delegate, retained :553, released only in WebView Drop :1423) and Shared owns the WebView — ownership cycle Shared→WebView→delegate→closure→Rc<Shared>, so the WebView's Rust Drop could never run. Consequence: the probe's R6 teardown observation (a #406 design input) would have silently reported OS process-exit reclamation as clean drop behavior — corrupted evidence, not just a leak. Fix: capture Rc::downgrade(&shared); handler upgrades or returns. The same shape awaits every production wry callback (#405 pane wiring, #406 lifecycle): any callback handed to a resource that the callback's captured state OWNS must capture weakly.

## BF-clippy-toml-not-in-receipt-fingerprint-001
*category: security · severity: medium*

TICKET-004 introduced a workspace clippy.toml (disallowed-types = std::process::Command — the SOLE enforcement of R14). clippy.toml is a gate-DEFINING file (it determines gate:2 clippy behavior), but gate_state_hash() in .claude/hooks/lib-hook-helpers.sh fingerprinted only crates/**/*.rs + scripts/*.sh + .claude/hooks/** + deny.toml + .gitleaks.toml + Cargo manifests/lockfile — NOT clippy.toml. So the commit receipt didn't bind clippy.toml: an attacker/sloppy edit could delete the ban from clippy.toml, run the gate green (clippy passes), restore the ban (gate_state_hash unchanged since it excludes clippy.toml), and commit R14-violating code under a stale-but-matching receipt — a config-swap-around-the-green that no honest gate run would pass. Caught by the inspect integrity critic. Fix: added clippy.toml to both ls-files pathspecs in gate_state_hash (the *.toml case filter already hashes it); verified an edit to clippy.toml now changes the fingerprint. The fix is self-certifying (lib-hook-helpers.sh is itself in the fingerprint).

## BF-commit-receipt-scope-001
*category: security · severity: high*

The commit-gate receipt (gate_state_hash) fingerprinted only crates/**/*.rs, while enforce-commit-gate.sh triggers only when .rs is in the changeset. So the files that DEFINE the bar — scripts/gates.sh, .claude/hooks/**, deny.toml, .gitleaks.toml — were committable with NO gate run and outside the fingerprint. An agent could weaken the gate (lower a floor, add an exclusion) and commit it ungated, pre-staging a future false green; TICKET-000 itself rode this bypass. Partial fix: gate_state_hash now folds the gate-defining files (scripts/*.sh, .claude/hooks/**, deny.toml, .gitleaks.toml, Cargo manifests + lockfile) into the fingerprint, so a post-green weakening invalidates a later .rs commit's receipt. Full closure (commit hook also demanding a receipt for no-.rs edits to gate-defining paths) is deferred — an all-stub workspace has no FULL-greenable gate to produce one; it lands once a crate gives real testable code. Disclosed in CONSTITUTION §15.

## BF-completion-accept-applies-request-time-range-to-a-typed-buffer-001
*category: runtime · severity: high*

#313's completion accept applied the server's REQUEST-TIME `textEdit` range to a buffer that had been typed into since — `s.` → popup opens → type `p` → Enter produced `s.pushp`, stranding the `p` after the insert. REQ-005 ("replacing the typed prefix, not appending after it") failing on the feature's single most common gesture.

WHY IT SURVIVED THE DESIGN: the popup deliberately SURVIVES typing — a word char re-filters the server's snapshot (no round trip, the #96 idiom) and re-stamps the recorded buffer version, which is exactly what stops the dismiss poll from dropping it. But the items — and their ranges — are never re-fetched. The install-time guard (`version == key.version`) makes the ranges exact only at install, and nothing re-checked them at accept.

WHY IT WOULD HAVE SHIPPED: (1) the `range: None` fallback arm was CORRECT (it uses the live caret), so the bug fires only when the server actually sends a `textEdit` — i.e. always with rust-analyzer, never in a hand-written fixture; (2) the design's own planned test ("type `pu`, accept `push`") accepts IMMEDIATELY after the popup opens, so it would have passed green straight over the defect. A test that never types between open and accept cannot see this class at all.

FIX: extend the server range to the LIVE caret — `(a.min(b).min(caret), a.max(b).max(caret))`. LSP requires the range to contain the request position, so the client owns everything typed since; min/max also normalizes an inverted range. Phase 4 must add a drive that TYPES between open and accept.

## BF-completion-popup-key-arm-missing-modifier-gate-001
*category: runtime · severity: high*

#313's completion popup key arm matched bare key NAMES (`"enter" | "tab"`, `"up" | "down"`) and never read `keystroke.modifiers`, while sitting ABOVE both the keymap dispatch and the editor's Tab arm in the on_key_down ladder. Consequences while the popup was open: (1) ⇧TAB matched `"tab"` and ACCEPTED a completion instead of reaching the editor's dedent arm — a wrong EDIT, not merely a dead key; (2) ⌘⌥↑/↓ matched `"up"`/`"down"` and moved the popup selection, killing the shipped M19 #297 add-cursor-above/below gestures for as long as the popup was up.

THE TELL: the very idiom #313 claimed to mirror already had the guard, and said why. #96's shell-popup arm carries `let plain = !mods.platform && !mods.control && !mods.alt;` on every branch with the comment "Only UNMODIFIED nav keys drive the popup — a ⌘/⌃/⌥ chord falls to `_`, dismissing and dispatching normally (so ⌘↓ still jumps blocks, ⌘1 still switches tabs)". #313 copied the SHAPE (a key-name match returning handled/declined) and dropped the GUARD.

FIX: adopt #96's gate — any platform/control/alt/shift modifier returns declined before the match.

## BF-copied-stale-key-field-never-read-001
*category: runtime · severity: low*

#326 mirrored #325's symbol_request stale-key field (search_request: Option<SearchQuery>) but the worker path gates staleness on the scalar search_gen via accept_gen(msg_gen, self.search_gen) — so search_request was NEVER read: write-only dead state (a needless query.clone(), two `= None` clears whose comment claimed a reopen-protection that the search_gen += 1 bump actually provides, and a dead SearchQuery struct + its stale-key doc describing a mechanism search_gen implements). #325's analog was genuinely read (apply_workspace_symbol_response compares the whole (query,gen) key); #326 switched to a gen-scalar accept, making the copied field redundant, but left it behind. Two critics (concurrency + simplification) independently flagged it. Fixed: deleted the field + its 5 sites + the struct; search_gen is the single source of truth.

## BF-debug-assert-arm-coverage-001
*category: test · severity: high*

A debug-panic/release-defined arm written as `debug_assert!(false, ...); <release-only fallback line>` (e.g. wrapping_add_signed, the backward-return, the floor round-down) makes the fallback line UNREACHABLE in a debug build (the assert panics first). Because gate:4 runs `cargo llvm-cov` in the default DEBUG profile, those lines get 0 coverage → gate:4 (--fail-under-lines 100) RED. Separately, a boundary guard like `while i > 0 && !is_char_boundary(i)` carries an EQUIVALENT mutant (`> 0` → `>= 0`, indistinguishable because is_char_boundary(0) is always true) that survives cargo-mutants → gate:5 (MSI 100, no exclusions) RED. Both caught in inspect by the mutation-readiness critic before validate. Fixed by restructuring to assert-the-invariant + always-compute-the-defined-result, and dropping the redundant guard.

## BF-directories-pulls-mpl-option-ext-001
*category: compilation · severity: medium*

The spec's reuse list named `directories` for home-dir resolution ("MIT/Apache"), but `directories` pulls `option-ext` (MPL-2.0) transitively via `dirs-sys`, and Marley's deny.toml allowlist deliberately bars MPL (permissive-only). gate:8 cargo-deny rejected it at implement-verify (error[rejected]: option-ext-0.2.0 license MPL-2.0 not explicitly allowed). The direct dep's own license being permissive is NOT sufficient — the whole transitive tree must satisfy the allowlist. Fix: switched to the `home` crate (0.5.12, MIT/Apache — cargo/rustup's home resolver, effectively a leaf, no MPL); `home::home_dir() -> Option<PathBuf>` feeds the home_dir_from seam directly. option-ext is now absent from the lockfile. Caught by the gate, not shipped.

## BF-doc-link-rot-under-migration-001
*category: validation · severity: low*

#396 gate:14 first run RED: two workspace.rs intra-doc links rotted under the migration — a pub item (TerminalPane) linking the now-private PaneContent, and a [`terminal`](Self::terminal) anchor pointing at a retired accessor (replaced by terminal_id). rustdoc -D warnings caught both only at the gate; the rename/de-genericize step should sweep doc anchors alongside code sites.

## BF-equivalent-guard-mutant-blocks-msi-100-001
*category: validation · severity: medium*

#330: `sticky_rows` used `if enclosing.len() > max_depth { enclosing.drain(0..enclosing.len() - max_depth); }`. `cargo mutants --list` yields `replace > with >=` — a PROVABLY EQUIVALENT mutant: `>` and `>=` differ only when `len == max_depth`, and there the mutant executes `drain(0..0)`, a no-op. Output is identical for every input, so NO test can kill it — it would survive forever, dropping MSI below the 100 floor on a pure seam and turning gate:5 red with no fix available except restructuring. The guard also generated the `- → +` and `- → /` arithmetic mutants (the latter needing a ≥5-scope test to kill at max_depth 2, since 3-2 == 3/2 == 1 — easy to miss). Caught by inspect critic 1 (logical proof + --list). Fixed: `let drop_outer = len.saturating_sub(max_depth); drain(0..drop_outer);` — identical behavior, and `saturating_sub` is a method call (cargo-mutants leaves it unmutated), so the entire guard+arithmetic mutant cluster disappears.

## BF-file-open-relative-path-cwd-190
*category: runtime · severity: high*

Clicking a file in the file browser silently did nothing (chad live-app feedback #3). Root cause: FileTree::path_at / list_files_in yield paths RELATIVE to the project root, but open_file_in_viewer read them with `std::fs::read(&path)`, which resolves relative paths against the PROCESS CWD. A bundled app launched via LaunchServices (`open Marley.app`) runs with CWD `/`, so `read("crates/.../buffer.rs")` → `/crates/...` → Err(_), which was silently swallowed by an `Err(_) => {}` arm. The file never opened and there was no error surfaced. It only appeared to work in a dev instance whose project root happened to be `/` (relative==absolute). Fix: pure `resolve_under_root(root, path)` (join relative onto root, pass absolute through) routed through open_file_in_viewer, so reads are CWD-independent. Class lesson: NEVER `std::fs::read` a project-relative path directly in a GUI app — always resolve against the known project root; the CWD is not the project root for a bundled/LaunchServices launch. And an `Err(_) => {}` swallow hid the failure — prefer surfacing a status flash on read error.

## BF-fusion-click-range-covers-only-primary-subtoken-001
*category: runtime · severity: medium*

#212's initial implement kept #196's clickable link range = the PATH portion only (at..at+path.len()). But #212 makes a file:line:col ref clickable-to-open-at-that-line — with a path-only range, clicking the :12:5 line/col portion of `foo.rs:12:5` does NOTHING (only the filename opens). The affordance over a compound token must cover the WHOLE token. Fixed in inspect: range = at..at+trimmed.len() (the full ref minus leading bracket + trailing punctuation); unchanged for a location-less ref (trimmed==path), so #196 stays green.

## BF-gate-cov-integration-lane-blindspot-001
*category: test · severity: low*

#396 gate:4 first run RED with ONE missed line: content.rs kind()'s Content::Terminal arm. Its only caller was tests/integration.rs — the gate's llvm-cov run counts in-lib lanes (unit + cfg(test) headless drives) but NOT the integration test binary, so an arm proven exclusively there reads as uncovered. Fix at source: assert the classification through the in-lib seeded-restore headless drive (view.content.get(cid).map(Content::kind) == Some(Terminal)).

## BF-in-app-commit-stale-derived-cache-001
*category: runtime · severity: low*

#328 git gutter: the derived git-marks cache is keyed on (path, git_marks_gen), and git_marks_gen was bumped only on save + external-file reload — but NOT after the app's OWN git-panel commit (git_commit). So committing FROM Marley's own commit panel moved HEAD (the committed lines are no longer "changed vs HEAD" → the bars should clear) yet the cache key was unchanged → refresh_git_marks returned early → the pre-commit Added/Modified gutter bars lingered until the next save/reload/file-switch. More surprising than the external-terminal-commit case because the mutation originated INSIDE the app. Caught by inspect critic 2. Fixed: bump git_marks_gen on a successful git_commit.

## BF-jump-helper-assumed-to-push-navstack-001
*category: runtime · severity: high*

#330: `jump_to_sticky_header` called `open_and_place_caret(path, |buf| line_start(row))` and both its doc comment and the design notes asserted it "places the caret, pushes the NavStack, centers" — but `open_and_place_caret` does NOT push the NavStack. It only opens the file, sets the caret, and parks pending_center_row. Every sibling jump (#312 goto-definition, the symbol picker, the problems panel) captures `(path, active_caret())` BEFORE the jump and pushes `NavLoc` EXPLICITLY afterward — proof the helper doesn't push (they would double-push). Effect: REQ-007 ("caret placed + NavStack pushed") violated — ⌃- would not return to where you were reading after clicking a sticky header, and the planned validate drive asserting the push would have failed. The design note mis-assumed the helper's side effects rather than reading it. Caught by inspect critic 3. Fixed: capture the origin before, push NavLoc only if open_and_place_caret returned true (the jump_to_definition shape).

## BF-lazy-assert-message-uncovered-cold-arm-001
*category: validation · severity: low*

#328 validate: a NEW test-setup helper `run_git` asserted `assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr))`. The `String::from_utf8_lossy(&stderr)` in the message is evaluated ONLY on the panic (false) arm — and git always succeeds in the seeded tempdir, so that function-call expression is an UNCOVERED cold region. gate:4 (whole-workspace line coverage 100%, headless_drive.rs is NOT excluded) went RED. A plain `assert!(cond)` or a LITERAL-message assert has no such cold region (llvm-cov attributes the line to the executed condition check); only a runtime CALL in the message body creates a separately-counted uncovered region. Fixed → a literal message. (Also the same helper tripped gate:2 by using the disallowed std::process::Command — separate BF.)

## BF-live-list-selection-identity-key-not-unique-001
*category: runtime · severity: low*

#327 problems panel: the selection-keep-by-identity across a live diagnostics refresh keyed on (path, line) only. Two diagnostics can share a line (a clippy Error + a Warning), and because the list sorts severity→path→line they sit far apart; keep_selection_by_identity's .position() returned the FIRST (path,line) match, so a refresh teleported the selection to the wrong (higher-severity) row (e.g. index 30 → 3). Bounded (lands on a real diagnostic, no crash) but undercut the 'identity' guarantee the doc advertised. Caught by inspect critic 2 (state) — it was the only un-noted gap; the doc even claimed identity-keeping. Fixed: extended the identity key to (path, line, character, severity) — the full tuple distinguishes same-line diagnostics (they differ in severity and/or character).

## BF-lsp-codeaction-inert-without-client-capability-001
*category: validation · severity: high*

#323 LSP code actions were DEAD against a real server and every unit/headless test passed. Marley's `initialize` advertised NO `textDocument` client capability, so per LSP 3.17 rust-analyzer withholds `CodeAction` literals (no `codeActionLiteralSupport`) — ⌘. returned "No actions here" despite a live unresolved-name diagnostic (⌘K hover on the same token proved the LSP round trip worked). Only the live drive against real rust-analyzer surfaced it; a mocked/`fake_ls` wire returns whatever the test author scripts, so it can never catch an unadvertised-capability omission. Fix: advertise `textDocument.codeAction` = `codeActionLiteralSupport`(kinds) + `isPreferredSupport` + `dataSupport` + `resolveSupport.properties=["edit"]` — exactly what #323 implements. Sibling of the #322 F-CORR-1 `documentChanges` omission (which made the version-conflict guard inert the same way). Both are honest-capabilities omissions where the SERVER gates its reply on a CLIENT capability.

## BF-lsp-diag-uri-key-mismatch-001
*category: validation · severity: high*

Inspect F1 (all 3 critics): the diagnostics store was WRITTEN keyed by the server's raw publishDiagnostics uri string but READ keyed by file_uri(absolute(path)). These agree only when the server echoes the didOpen uri byte-for-byte — rust-analyzer on a plain-ASCII path does (so the acceptance path passed), but any server that re-encodes the uri (an @scope path → %40 vs literal @, a space, a sub-delim) or resolves a symlink would SILENTLY drop every squiggle/gutter/count/F8 target for that file, with no error. Fix: key the store by canonical PathBuf (matching the #309 docs map) — decode the wire uri via a new pure path_from_file_uri (the inverse of file_uri) and normalize through the SAME absolute() the lookup uses. Guarded by a path_from_file_uri round-trip unit (decode∘encode=id over special-char paths) + a headless integration test feeding a real publishDiagnostics through the whole capture path and asserting the accessors agree on the key.

## BF-lsp-didopen-carries-empty-text-ready-race-001
*category: runtime · severity: critical*

SHIPPED-CODE BUG found by #312's LIVE drive (in #309's pump, shipped 3 commits earlier): EVERY textDocument/didOpen went out with an EMPTY text field, so rust-analyzer held every open file as a ZERO-LENGTH document and answered null to every hover (#311) and definition (#312) — the whole M20 request line was dead in the real app while all headless tests passed.

ROOT CAUSE — a Ready race between two Ready-gated calls with the collect in between. The pump did: (1) build `open_files`, materializing `buffer.text()` only `if host.needs_text(path, version)` (the #309 inspect's hot-path optimization, PR-claude-pump-materialize-hot-data-only-when-consumed-001); (2) `for host { host.drain(); host.reconcile(&open_files) }`. But a host reaches Phase::Ready INSIDE drain() — it is the initialize response that promotes it — and BOTH needs_text and reconcile early-return unless Ready. So on the exact tick the server went Ready: needs_text still saw the PRE-Ready phase → text = String::new(); drain() → Ready; reconcile → now Ready → sent that empty string as the didOpen text AND recorded the doc as synced at that version. No didChange ever follows (entry.synced == version) until the user happens to type, so the empty document is PERMANENT.

WHY NO TEST CAUGHT IT: every #309/#310/#311/#312 headless drive INJECTS synthetic responses via push_response_for_test into a process-less host, so none of them ever exercises the real didOpen→server→answer round trip. #310's and #311's "driven proof" were both headless for the same reason. #312's validate was the first time the live wire was actually inspected.

EVIDENCE (wire capture via an `[[lsp.servers]]` tee wrapper around the real rust-analyzer): BEFORE — 3 didOpens, each `TEXT_LEN=0`, whole client→server log 1226 bytes; live F12 and ⌘K both returned empty even for a SAME-FILE symbol (definition at line 18 of the very file being edited), which is the decisive tell that the server had no document. AFTER the fix — a single didOpen is `Content-Length: 16781` carrying the real source, log 104220 bytes.

FIX: drain FIRST in its own loop, then collect open_files (so needs_text reads the settled phase), then reconcile the active root's host. Keeps the hot-path optimization; removes the race.

## BF-lsp-goto-failed-open-writes-caret-into-wrong-file-001
*category: runtime · severity: high*

#312 jump_to_definition/nav_back called open_file_in_viewer and then IMMEDIATELY read active_editor() to place the caret — but open_file_in_viewer is `if let Some(..) = load_code_view_state(path)` with NO else, so it no-ops on a missing / non-file / >2MB / binary target and leaves the PREVIOUS file active. The target's (line, character) was then mapped against the ORIGIN's text and the origin's caret moved there, plus pending_center_row scrolled it — SILENTLY, because position_to_offset clamps rather than panicking. Reachable and common: rust-analyzer resolves definitions into >2MB generated/vendored sources (real >2MB .rs files confirmed in ~/.cargo/registry on this machine), where load_code_view_state even flashes "can't open …(>2MB)" WHILE the caret still moves. jump_to_definition also pushed the NavStack BEFORE the open, recording a return for a jump that never happened. Found by 3 of 5 inspect critics independently. FIX: open_and_place_caret verifies the landing (active_file().path == path) before touching the caret and returns false otherwise → flash, nothing moves; the NavStack push moved to after a confirmed landing.

## BF-lsp-hover-caret-move-not-dismissed-001
*category: runtime · severity: high*

Inspect F1 (all 3 critics): the hover card's dismiss-on-caret-move + the response stale-guard were hung off the action dispatcher (dispatch_action guard), but plain arrows / ⌘-arrows / a mouse rail-click tab switch / the uniform_list's own scroll are handled INLINE and never reach dispatch_action — so pressing an arrow left the card pinned at a stale cell (REQ-004 fail) and an in-flight response was shown at a moved caret (REQ-005 fail); two unedited files both report version 0 / first-row 0 so a version+scroll-only poll couldn't tell them apart. Fix: HoverCard carries the (file path, caret, version, first-row); poll_hover_dismiss compares the LIVE editor identity each pump tick and dismisses on ANY change; consume drops a response whose key.uri no longer names the focused file. Prevention rule PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001.

## BF-lsp-hover-extracted-helper-new-mutation-surface-001
*category: test · severity: medium*

Validate gate went RED twice: (1) the inspect-F9 DRY extraction of `token_color` into a top-level fn made cargo-mutants mutate it standalone (`body → Default::default()`) — it survived because the helper is called only from the coverage-excluded render, so no test killed it (MSI 98.1%); fix = a token_color_maps_each_kind unit test with exact per-kind color assertions. (2) hover.rs had 1 uncovered line — markup_to_string's `_ => String::new()` arm (a non-string/object/array `contents`); fix = a `{"contents": 42}` test. LESSON: extracting an inline expression (previously buried in an unmutated render closure) into a named fn CREATES a new standalone mutation target — pair the DRY extraction with a direct unit test, or it fails the MSI floor even though behavior is unchanged. Both caught by the gate, fixed at source (§0), re-run GREEN (MSI 100, coverage 100).

## BF-lsp-navback-stale-offset-line-col-panics-001
*category: runtime · severity: high*

#312 nav_back (⌃- jump-back) fed the RAW popped NavStack offset to Buffer::line_col, which is an unguarded rope.char_to_line — ropey PANICS when char_idx > len_chars, killing the app on the gpui UI thread. set_single_caret was safe (it funnels to set_selection → clamp_char_offset), so the CARET was clamped but the ROW read was not. The trap: the sibling accessor Buffer::line_start two lines below DOES clamp and documents "never panics", so the codebase treats clamping as opt-in per-accessor — line_col did not opt in, and the code assumed the neighbourhood was uniformly safe. Reachable by ordinary use because EditorSurface::open PRESERVES an already-open buffer (M15 #249): F12 → jump → return → delete a block → ⌃- pops a now-past-EOF offset → crash. Found by 3 of 5 inspect critics independently (two reproduced it against real ropey 1.6.1). FIX: the shared open_and_place_caret reads the row back from active_caret() AFTER the clamp, never from the requested offset. Lead-verified by probe: raw 4000 → ropey panics; clamped → CharOffset(11) → row=1, with 11 == len_chars proving the exact boundary is in-bounds.

## BF-mcp-pre-auth-body-alloc-001
*category: security · severity: medium*

marley_mcp transport (#370): read_http_request allocated `vec![0u8; content_length]` from an UNAUTHENTICATED, uncapped `Content-Length` header, BEFORE the origin/bearer guards run. A hostile loopback request with `Content-Length: 999999999999` triggers an OOM → Rust handle_alloc_error process-abort, taking down the whole gpui app + all the user's terminals (availability). Caught by two inspect critics. Fixed: cap MAX_BODY_BYTES=1MiB, reject oversized as 400 before any allocation. Class: a pre-auth path must validate/cap a client-controlled size before allocating.

## BF-modal-open-over-live-completion-steals-keys-001
*category: runtime · severity: high*

#325 opening the ⌘T modal picker while an editor completion popup was live left completion_menu open — its key arm runs BEFORE the picker's in the router, so ↑/↓/Enter kept driving the COMPLETION list and Enter routed to accept_completion_item, inserting a candidate into the buffer BEHIND the picker (the unguarded Buffer edit path). ⌘T falls through handle_completion_key (which only consumes Esc), so the popup never self-dismissed. Fix: open_symbol_finder dismisses completion_menu + the terminal completion first, so the modal truly owns the keyboard. The class: a modal that owns the keyboard must dismiss any lower overlay whose key arm precedes it (the sibling of #324's stale-card-over-palette).

## BF-modulo-wrap-equivalent-mutant-at-small-n-001
*category: test · severity: high*

#181 AgentLauncherState::move_up used the modulo wrap `(sel + n - 1) % n` over a fixed 2-element list. At n=2 the `- 1`→`+ 1` mutant is EQUIVALENT (±1 are identical mod 2), so it survives cargo-mutants → MSI < 100 → gate:5 RED. Caught by both inspect critics + an empirical targeted run (MISSED). Root cause: a modulo-arithmetic wrap over a small fixed list makes symmetric ± mutants indistinguishable — no test over the API can kill them. Fixed by the explicit branch form `if sel == 0 { n-1 } else { sel-1 }`, whose mutants all produce a wrong/out-of-bounds index a test catches; re-ran cargo mutants → 0 missed.

## BF-mutants-skip-detached-by-fn-insertion-001
*category: test · severity: high*

#184: inserting a new fn `block_line_counts` immediately ABOVE `content_rows` placed it between content_rows's pre-existing `#[cfg_attr(test, mutants::skip)]` and content_rows itself — so the attribute (plus a stale doc comment and a duplicate skip I added) all bound to the NEW fn, silently un-skipping content_rows. content_rows is a coverage-excluded gpui shim with no unit test, so its 4 mutants became unkillable → MSI < 100. Invisible to cargo check + cargo fmt (both pass — attributes/docs are syntactically fine wherever they sit). Caught by the inspect mutation critic via cargo mutants --list. Fixed by reordering so each fn owns its own doc + skip.

## BF-mutants-skip-detached-recurrence-183-001
*category: test · severity: high*

#183 RECURRENCE of BF-mutants-skip-detached-by-fn-insertion (from #184): I inserted a new fn `path_command_names` immediately above `complete_at_prompt`, placing it between complete_at_prompt's doc comment + `#[cfg_attr(test, mutants::skip)]` and the fn — so the attribute rebound to the new fn, silently un-skipping complete_at_prompt (a coverage-excluded untestable shim) → 9 unkillable mutants → MSI would go RED. Invisible to cargo check + cargo fmt. Caught by both inspect critics via `cargo mutants --list`. The prevention rule PR-claude-verify-skip-attr-still-attached-after-inserting-fn ALREADY EXISTED from #184 — I failed to apply it proactively. Lesson reinforced: after inserting ANY fn adjacent to a skipped shim, immediately grep -B1 the skip or run cargo mutants --list to confirm the previously-skipped fn is still absent. Fixed by reordering.

## BF-new-untracked-file-skipped-by-in-diff-mutation-001
*category: validation · severity: high*

#329: the pure `SelectionLadder` lives in a NEW file `selection_ladder.rs`. The `--diff` mutation gate feeds cargo-mutants `git diff HEAD -- crates`, and `git diff HEAD` EXCLUDES untracked files — so a brand-new (never-`git add`ed) source file contributes ZERO lines to the diff, and ALL its mutants are silently skipped. gate:5 would print MSI 100% and GREEN without ever testing the new file's pure seam — a false green precisely on the code most in need of mutation coverage. Caught by cross-checking `cargo mutants --list --in-diff` (the new file's mutants were absent). Fixed with `git add -N <file>` (intent-to-add) so `git diff HEAD` includes it. The whole-workspace COVERAGE gate is unaffected (it's not diff-scoped), which is why the gap is easy to miss — coverage catches the file, mutation doesn't.

## BF-non-exhaustive-enum-vs-variant-001
*category: compilation · severity: medium*

StandardizedPath used `#[non_exhaustive]` on the ENUM to satisfy R13 ("no external crate can construct a variant directly or match it exhaustively"). Enum-level #[non_exhaustive] blocks external EXHAUSTIVE MATCHING (must add a wildcard) but does NOT block external CONSTRUCTION of a known variant — tuple-variant fields are public, so a downstream `StandardizedPath::Posix(s)` would compile, bypassing the canonical invariant (and is_absolute() would then lie about a hand-built value). The correctness critic marked R13 "structurally ✓"; the gap was caught only when the validate phase wrote the trybuild compile-fail and it did NOT fail as expected. Fix: put `#[non_exhaustive]` on each VARIANT (not just the enum) — that blocks both external construction (E0603 "cannot be constructed because it is #[non_exhaustive]") AND non-wildcard matching. Verified by the trybuild .stderr.

## BF-persist-setter-body-mutant-survives-no-roundtrip-001
*category: validation · severity: medium*

#330 validate: gate:5 had exactly ONE survivor — `settings.rs persist_sticky_header -> Ok(())`. A `persist_X(manager, v) -> Result<(), SettingsError>` setter's body-replacement mutant returns Ok(()) WITHOUT writing, and nothing detected it: the new `editor.sticky_header` setting had a field/applied_from/applied_defaults wiring (all covered by the existing AppliedSettings tests) but NO test proving the SETTER actually persists. Every other `persist_*` in the crate is covered by the shared `settings_round_trip_survives_reload` test; the new one simply wasn't added to it. Fixed by adding the sticky-header leg to that round-trip — persisting the NON-DEFAULT value (false; the setting defaults true) so a no-op write reloads as the default and fails the assert. MSI 18/18 → 100.

## BF-query-keyed-refetch-duplicates-same-string-001
*category: runtime · severity: medium*

#325 workspace/symbol keyed its stale guard on the query STRING only. Two concurrent in-flight fan-outs of the SAME string (type `par` → backspace → `par` while the first is still indexing) both satisfied key.query == live and both merge-appended → every row duplicated. The completion sibling avoids this because CompletionKey carries the monotonic buffer version. Fix: SymbolQuery gains a monotonic gen bumped per fan-out; an older answer's gen mismatches → dropped; symbol_request also cleared on close. The class: a per-keystroke re-query keyed only on the query can't distinguish concurrent same-string fetches — carry a generation.

## BF-r9-no-field-denylist-non-exhaustive-001
*category: test · severity: medium*

The planned R9 check ("Config has no network/auth field") was a trybuild compile-fail denylisting specific accessor names (Config::marley().server_url() → E0599). The inspect clean-room critic showed this is non-exhaustive: it only disproves the enumerated names, so a future un-listed accessor/field (endpoint(), ws_uri(), sync_host()) would pass while violating R9; and it tests accessors, whereas R9 constrains FIELDS (private, invisible to an external trybuild). Caught in inspect before the test was written. Fix: the primary R9 guard is an in-crate exhaustive destructure `let Config { app_id: _, logfile_name: _ } = Config::marley();` (and `AppId { id: _ }`) with NO `..` — E0027 forces every field to be named, so ANY added field fails to compile (a positive, exhaustive check). The accessor-denylist trybuild stays as documented non-exhaustive defense-in-depth.

## BF-recursive-tree-walk-stack-overflow-on-deep-input-001
*category: runtime · severity: high*

#330: `collect_headers` (the all_headers full-tree walk) was written as plain recursion over `node.children()`, with a doc comment claiming "Rust nesting depth is bounded (tree-sitter caps parse recursion), so plain recursion is safe" — FACTUALLY WRONG. The recursion depth is the FULL tree depth (every nested block/paren/expression level), not the header nesting: a probe showed one `fn` wrapping 2000 nested blocks recurses 2000 deep to find ONE header. Measured on the 8 MiB macOS main stack: survives 12k levels, OVERFLOWS and SIGABRTs at 16k (2k-5k on a smaller test-thread stack). That is an UNCATCHABLE abort (not catch_unwind-able), it runs SYNCHRONOUSLY on the gpui UI thread, and it re-runs on EVERY edit (sticky headers default ON) — so a machine-generated/minified/adversarial Rust file would take down the whole app (every open terminal + editor). tree-sitter's own parser is iterative precisely because of this. The sibling #329 enclosing_ranges correctly climbs iteratively via .parent(); this new walk was the crate's only deep-recursion path. Caught by inspect critic 1 with measured stack ceilings. Fixed: rewrote as an iterative TreeCursor preorder walk (goto_first_child/goto_next_sibling/goto_parent, O(1) stack, identical document order).

## BF-rename-open-file-misroutes-to-disk-under-symlinked-root-001
*category: runtime · severity: high*

#322 inspect (HIGH, correctness critic): the multi-file rename applier decided open-vs-closed by RAW PathBuf equality (has_path/locate_open_file: `f.view.path == fe.path`), but the two sides are canonicalized DIFFERENTLY. The edit target `fe.path` = path_from_file_uri(server_uri) is CANONICAL (the server only ever learns a path via uri_for → LspHost::absolute → canonicalize). The open file's `f.view.path` = root.join(rel) is NON-canonical (Project stores the root verbatim). Under a symlinked root — /tmp→/private/tmp on macOS (which the drive fixtures AND the live-drive probe project use), a symlinked $HOME, /Volumes automounts, an unresolved .. — the two differ, so an OPEN file reads as CLOSED and apply_one_file falls through to a blind fs::read/fs::write on disk. Clean buffer → the rename lands on disk, the buffer is untouched, one ⌘Z reverts nothing, a spurious 'Changed' banner fires, yet rename_summary reports success. DIRTY buffer → the server computed positions against the synced buffer text but they apply to the last-SAVED disk text → on-disk CORRUPTION while the buffer shows un-renamed text and the user is told it worked. Would have shipped and fired in the very live-drive fixture. Fix: a same_file(a,b) that compares CANONICAL forms (raw fallback when a side is absent), used by has_path + buffer_for_path_mut — the editor surface adopting the same keying discipline the LSP host already uses.

## BF-resolve-against-same-root-that-built-stored-path-001
*category: runtime · severity: medium*

#289's diagnostics shim resolved terminal file-refs against `active_project().root` to compare them to the OPEN editor file's stored path — but that stored path was built (in load_code_view_state) by resolving against `self.project_root`, a DIFFERENT field (kept in sync but distinct). Any sync-timing gap would make the resolved ref != the stored open_path, silently dropping a valid diagnostic. Fix: resolve against the SAME field (`self.project_root`) that built the stored path, so a path-equality comparison is guaranteed consistent. Class: when comparing a freshly-resolved path against a STORED resolved path, use the identical resolution root, not a parallel one.

## BF-search-read-whole-file-before-size-gate-001
*category: performance · severity: high*

#326 project-search worker (run_search) called std::fs::read(&path) and only THEN checked bytes.len() <= VIEWER_MAX_BYTES. std::fs::read pre-sizes its buffer from file metadata, so a multi-GB non-gitignored file (a CSV/sqlite/media blob in a non-git dir the walk doesn't exclude) → a multi-GB transient allocation that ABORTS the whole process on OOM (Vec::with_capacity abort is not caught by the match arm), and a TOCTOU fifo/device gap (is_file at enumeration, read later → a fifo/dev/zero swapped in blocks the worker forever or grows unbounded). The repo had ALREADY fixed this exact antipattern in load_code_view_state (stat-first, the #106 lesson) but run_search didn't mirror it. Caught by inspect critic 4 (security/resource). Fixed: std::fs::metadata gate (is_file + len<=cap) BEFORE fs::read, post-read size check kept as a TOCTOU backstop.

## BF-shrinking-edit-window-drops-token-001
*category: runtime · severity: critical*

#285's incremental-highlight windowing shipped GATE GREEN with a CRITICAL correctness bug: a SHRINKING edit (backspacing off a comment's tail, or a token re-tokenizing when a delimiter like `*/` is deleted) made the surviving token's highlight VANISH. Root cause: #285 assumed `changed_ranges ∪ edit-span ⊇ every stale span`. FALSE — `changed_ranges` is a BYTE diff, not a token diff, so (a) a token whose extent shrank while its bytes stay textually unchanged is not reported (a pure deletion yields an EMPTY window [p,p); tree-sitter's set_byte_range excludes a node ending exactly at p), and (b) a token merely ADJACENT to the edit that re-tokenizes at the changed-range boundary is just outside a byte-tight window. In both, splice drops the stale cached span with nothing to replace it. WHY IT SHIPPED: the #285 equivalence corpus was HAND-PICKED and lacked shrinking-tail/adjacent-re-tokenize cases, and the inline inspect trace (+ the first critic) reasoned the fix complete — both WRONG. The bug (and, later, that the FIRST #288 fix was still incomplete — twice) was found only by a DIFFERENTIAL FUZZER (random edit chains asserting inc==fresh). FIX (#288): cover_edited_cached widens the window to the surviving footprint of every cached span overlapping OR abutting [start,old_end]; snap_to_lines snaps to whole-line bounds; equivalence scoped to same-incremental-tree (tree-sitter can itself diverge on error inputs). Durable fix: a tree-match-guarded in-test differential fuzzer, 0 same-tree divergences / 120k steps.

## BF-signature-card-anchors-to-moved-caret-001
*category: runtime · severity: medium*

#324 signature-help card opened from an ASYNC response anchored to the LIVE caret and dismissed on ROW-change only, so: (1) pressing Enter right after `(` (caret to a new row) before the answer landed opened a card for the finished call on the new row; (2) the overlay re-read the live caret each frame, so the card GLIDED along the line as the cursor moved through args. The SignatureKey carried `version` but apply never used it. Fix: apply drops the answer when the live caret row != the request row (key.line); OpenSignature stores a FIXED anchor_col and the overlay anchors there (stays put). A live-version check was the wrong fix — signature help persists across arg-typing (which bumps the version without a new request), so a version drop would kill the card on every keystroke; the row guard preserves persistence.

## BF-signature-passive-overlay-eats-keys-and-floats-001
*category: runtime · severity: medium*

#324 signature-help card (a PASSIVE overlay — you type args through it) had two interaction bugs inspect caught: (1) handle_signature_key matched up/down on the KEY only, so a multi-signature card swallowed ⇧↓ (shift-select) and ⌘↑/⌘↓ (doc start/end) — modified arrows meant for the editor were eaten; (2) the card was cleared only at the agent-launcher choke, so opening the palette/finder/history/find (whose key arms run AFTER the card's) left a stale card floating over them + eating ↑/↓/Esc — the #323-SELF-1 keyboard-dead class, half-fixed. Fix: gate the ↑/↓ arm on a BARE arrow (decline modified), and clear signature_card at the TOP of dispatch_action (one site covering all the overlay-open actions), not one inline site.

## BF-spawn-once-worker-never-respawns-001
*category: runtime · severity: medium*

#326 off-thread search worker: ensure_search_worker_and_send gated the spawn on search_worker.is_none() and set it to Some once, never resetting to None. So the moment the worker thread exited (a panic in the walk body, or a channel send error), search_worker stayed Some holding a dangling sender — every subsequent ⌘⇧F: is_some → skip spawn → tx.send errors → returns false → the picker opens, clears its list, and shows nothing FOREVER, no error/flash/recovery short of an app restart. Latent today (run_search has no panic surface), but the non-respawn design converts any future/rare thread death into permanent silent breakage. Caught by inspect critic 2 (concurrency). Fixed: on tx.send err, set search_worker = None so the next launch respawns.

## BF-standardize-cwd-failure-non-absolute-001
*category: runtime · severity: high*

standardize_path's helper read the cwd via `current_dir().map(...).unwrap_or_default()`, returning "" when cwd is unavailable (reachable: a terminal/editor whose working directory was deleted or had search permission revoked). With cwd="", a RELATIVE input stayed relative after normalize — yet StandardizedPath::is_absolute() hardcodes `true`, so the sole constructor of the absolute invariant emitted a relative/empty value while claiming absolute, violating R9 and making is_absolute() lie. Caught by the inspect correctness critic (probed in the typed-path scratch crate). Fix: thread cwd as a parameter to standardize_under<E>(input, cwd) (a §14 testable-IO seam) with an empty-cwd→flavor-root("/") fallback so the result is ALWAYS absolute; the relative-input and empty-cwd branches are now deterministically testable by injecting cwd.

## BF-symbol-picker-render-cap-diverges-from-nav-001
*category: runtime · severity: high*

#325 ⌘T workspace-symbol picker capped its DISPLAY at 64 rows but left the stored results uncapped, and move_down + Enter operated on results.len(). So a query returning >64 (the common case — opening the picker parks an empty query → the whole workspace) let ↓ walk the selection past row 63 (frozen highlight, invisible cursor) and Enter jump to results[selected] — a symbol never shown or highlighted. Caught by inspect (2 critics). Fix: a shared MAX_SYMBOL_ROWS const; clamp move_down + Enter to cap_with_tail(len, MAX).0; the overlay uses the same const. The class: a render-only cap must also bound navigation/selection.

## BF-tests-ran-runner-detection-001
*category: validation · severity: medium*

When porting the test gate from `cargo test` to `cargo nextest run`, the invocation in gates.sh and the BLOCKED-message hint in enforce-tests-ran.sh were updated, but the hook's DETECTION regex (line 29: `cargo (test|llvm-cov)|scripts/gates.sh`) was not — `cargo nextest` does not match `cargo (test|llvm-cov)`. A validator running only the now-prescribed `cargo nextest run` would be spuriously Stop-blocked ("tests never ran") despite running tests. Caught by an inspect critic; never shipped. Fixed: detection regex → `cargo (test|nextest|llvm-cov)`.

## BF-totality-branch-only-degenerate-input-uncovered-001
*category: validation · severity: low*

#329 inspect (C1-2): `enclosing_ranges` has a `let Some(node) = ...named_descendant_for_byte_range(s,e) else { return Vec::new() }` totality arm. A probe showed that for ANY valid `start ≤ end` range (even wildly out-of-bounds like 100..200 on a 9-byte file) tree-sitter returns at least `Some(source_file)` — it clamps to the root, never None. The None arm is reachable ONLY by a REVERSED range (`start > end`), which the app never produces (selections have start ≤ end). Under the repo's whole-workspace `--fail-under-lines 100`, that arm would sit UNCOVERED and turn gate:4 RED at validate, with a non-obvious trigger. The code is CORRECT (Option totality per §14); the defect is a missing degenerate-input test. Caught at inspect (not shipped); the validate plan now owes `enclosing_ranges(&s, 5..2) == []`.

## BF-zed-brand-word-in-source-comment-001
*category: validation · severity: high*

#328: a design note that CITES the reference app ("git marks live in their own store, never on OpenFile — Zed's decoupling warning adopted") was copied VERBATIM from the spec into an app.rs code comment. gate:14 brand-scrub greps `warp|zed` (case-insensitive, word-boundary) across crates/**/*.rs and would have FAILED the commit. The spec (docs/) may cite Zed/Warp — docs aren't scanned — but a crates/ source comment must not contain the brand word. This is the exact #327 lesson (a "Zed chord" comment failed gate:14) and it slipped again despite the recorded prevention rule. Caught by inspect critic 3 (it ran the gate command live). Fixed: reworded to "git state lives in its own store, not on the file entry".

## boot-local-shadows-field
*category: runtime · severity: high*

M10 #163: the boot restore consumed PaneId blocks 1..N via a LOCAL `let mut pane_blocks` counter, but the Self constructor still hard-initialized the FIELD to 0 — the first post-boot ⌘T would re-mint block 1, colliding with a restored grid's block and resurrecting the #158 cross-tab aliasing that #167 had JUST structurally killed (and the exact close-cleanup would strip the WRONG tab's agents). Caught in inspect before gate/commit. Fix: seed the field from the local (`pane_blocks,`). The class: when new() advances a local accumulator that has a same-named field, the field init MUST consume the local — after writing any boot loop, grep the constructor for `field: 0,` shadows.

## F-claude-writer-survivor-model-missed-reader-drop-001
*category: validation · severity: medium*

#408 inspect F1: serialize_shell pre-scan counted a root:empty project as a reclamp survivor, but restore_shell unconditionally drops empty-root lines — the reclamped active_project indexed a line the parse removes (the drift class the ticket fixes, reintroduced one level removed). Found by two critics independently; unreachable from the live writer (roots come from opened is_dir directories). Fix: the survivor filter gained && !p.root.is_empty(); emission iterates the survivor list so writer and reclamp agree by construction. Paired rule: PR-claude-survivor-model-must-mirror-reader-drops-001.

## synthetic-chords-latch-session-modifiers
*category: infra · severity: medium*

M11 #174 driven validation: after cmdshift:a/cmd:t chords, a nil-source "plain" CGEvent click INHERITED the latched ⌘ from the session's synthetic-modifier state — the agent-row click hit the ⌘-click diff branch instead of the jump (twice; a clearmods key-up sweep did NOT reliably reset it). Fix (deterministic, landed in drive.swift): every synthetic mouse/key event now sets flags EXPLICITLY ([] for plain, the intended mask for chords) — never rely on inherited state; clearmods stays as belt-and-suspenders. The class: with CGEventPost, event flags are AMBIENT unless pinned per-event; any harness mixing chords and plain input must pin flags on EVERY event.
## F-claude-export-joined-double-encoded-jsonb-as-chars-001
*category: validation · severity: medium*

TICKET-409's ticket exporter rendered every Tags line as per-character soup
(`[, ", M, 2, 0, …`): the forge DB stored `tickets.tags` DOUBLE-encoded (a jsonb
STRING containing a serialized JSON array), so one `json.loads` yielded a str and
`", ".join(str)` iterated characters. Caught by the inspect export-fidelity critic
(full-corpus diff vs the DB); descriptions/ledger bodies were byte-exact — only
the doubly-encoded column mangled. Fixed by inner-decoding
(`json.loads(json.loads(raw))` with an isinstance guard) and rewriting the 16
Tags lines before anything was committed.

## F-claude-search-walk-verbatim-root-misses-canonical-override-keys-001
*severity: medium · category: state-integrity · status: fixed (#319 inspect)*

Marley #319 inspect (correctness critic, F2). The ticket canonicalized every STORED editor path, and the design swept equality/prefix/strip CONSUMERS — but missed a PRODUCER: ⌘⇧F's `consume_search_query` snapshots dirty buffers into `overrides: HashMap<PathBuf, String>` keyed by the stored (now canonical) paths, while `run_search` walks `walk_text_files(&req.root)` with the VERBATIM project root. A directory walker's yielded paths inherit the WALK ROOT's spelling, so under a symlinked root (`/tmp`→`/private/tmp`) every yielded path spelled verbatim, every override probe missed, and search silently read stale DISK text for dirty open files — defeating dirty-buffer-wins — with `jump_to_match` offsets computed against the wrong text landing carets mid-token. Pre-fix it HIT for tree-opened files (verbatim stored) — the canonicalization change itself flipped this store from mostly-hit to always-miss under aliased roots, while every direct compare in the diff was sound. FIX: walk the canonical root (`SearchReq.root = canonical_root(&root)`) and keep the verbatim root only for the instance-scope filter (`i.root() == root`, verbatim==verbatim by D3).

## F-claude-git-marks-close-leak-sibling-of-editor-folds-001
*severity: low · category: state-integrity · status: fixed (#412)*

Found by the #353 inspect correctness critic while fixing the `editor_folds`
close-leak: `git_marks: HashMap<PathBuf, Vec<(usize, GitMark)>>` (app.rs ~:320)
has the IDENTICAL class one field down — populated per viewed file whenever the
active file carries diff marks, removed only when that file is ACTIVE again with
an empty diff, scrubbed by no close path. A file closed while carrying marks
leaks its entry for the app's life (memory-only; reopen self-heals the render via
the #275 activation choke bumping `git_marks_gen`). Deferred to TICKET-412 (one
shippable slice: the scrub lines need git-diff drive fixtures to clear the
mutation bar); the #353 choke (`RootView::release_editor_views`) is the natural
scrub point, `git_marks_key` included.

## F-claude-hit-arm-row-mint-fresh-spelling-diverges-from-instance-001
*severity: low · category: state-integrity · status: fixed (#353 inspect)*

#353 inspect (correctness critic). The viewer/split `find_open` HIT arms minted
the view row's `CodeViewState` from the freshly computed `canonical_under_root`
spelling while the instance kept its birth spelling. A TOCTOU birth (the file
landing between the canonicalize fallback — which yields the lexical JOIN — and
the stat gate) stores a join-spelled instance under a symlinked root; the next
open computes the canonical spelling, `same_file` hits, and row-key ≠
instance-key: the fold verbs key by ROW, the #353 last-close scrub keys by
INSTANCE, so the entry becomes unremovable (the leak survives in that corner) and
one instance transiently owns two fold entries. FIX (by construction): on every
resolve-HIT, mint the row from the instance's STORED path
(`content.get(id).as_editor().path()`) — the `add_content_to_split` idiom, now at
all three mount families. Outside the race stored == computed byte-equal, so the
change is behavior-neutral there.

## F-claude-teardown-clear-eats-queued-terminal-outcomes-001
*severity: high · category: state-integrity · found: #332 inspect (correctness + state critics, independently)*

The #332 implementation delivered Abandoned outcomes on both abandonment paths
but kept `on_connection_lost`'s pre-existing `responses.clear()` ahead of the
sweep — and the clear itself was a signal-eater. Two proven traces: (A) a dying
server writes its final answers and exits; mpsc yields the buffered chunks
BEFORE reporting Disconnected, so one `drain()` routes the answers into
`responses` (purposes removed at delivery) and THEN fires the teardown — the
clear destroyed an answered-then-died request's only terminal outcome, and
`abandon_all` could not re-deliver (the purpose had already left the map). A
committed rename would end in silence — the exact outcome the ticket's own arms
forbid. (B) cross-tick: a backgrounded root's queued Abandoned(Timeout) (only
the active root's queue is drained) destroyed by a later EOF teardown. Fix:
teardown DELIVERS rather than destroys — keep the queue, extend with the sweep;
exactly-once holds structurally because queue-entry and map-removal are the
same move. The spec's own D2 ("push AFTER the clear") had encoded the bug as a
safety rule — the plan-time model treated queued outcomes as "stale answers"
when they are terminal signals.

## F-claude-shared-latch-cross-purpose-abandon-clobber-001
*severity: medium · category: state-integrity · found: #332 inspect (correctness critic)*

`PrepareRename` and `Rename` purposes shared ONE latch (`rename_request`) keyed
by a generation-free `RenameKey` (uri/line/character). A superseded prepare's
terminal outcome (Abandoned, or a late answer) could pass the guard against the
NEWER committed rename's latch — same key value — and clear it, so the rename's
real answer would guard-fail and drop: a committed rename with no edit and no
flash. #332 added the fully-silent lane, but the Answered-lane confusion
pre-existed ("Cannot rename this" mid-rename). Fix: give prepareRename its own
slot (`prepare_rename_request`) — the F-CORR-2 code-action-resolve precedent
(`code_action_resolve_request` "its OWN slot"). Class: two request kinds may
share a latch only if their keys are generation-disambiguated or their
lifecycles cannot overlap; when in doubt, one slot per purpose kind.

## F-claude-354-supersede-slot-drops-cross-file-parked-save-001
*category: runtime · severity: medium*

#354 (caught at inspect by the lead AND both correctness critics independently): `begin_format_on_save` opened with an unconditional `pending_save_after_format = None` — correct for the #314 C1-MEDIUM SAME-file orphan supersede, but once saves became origin-bound it silently discarded a DIFFERENT file's parked, consented ⌘S: park on A (slow formatter), switch to Rust file B, ⌘S — A's latch replaced, A's response superseded-dropped, deadline finds None; A never saves, no flash, stays dirty. The tell: a ⌘S on a NON-Rust B took the plain-save arm, left A's latch alone, and A completed fine — the loss fired only on the format-armed path. Fix: `take()` the latch; a different-origin occupant is SETTLED (plain-save via the origin funnel, exactly as its own Err/deadline would) before the new park; same-origin keeps the supersede. Class: a single-slot latch holding consented work must settle, never discard, a cross-target occupant (PR-claude-single-slot-latch-supersede-must-settle-not-discard-001).

## F-claude-354-reload-epoch-collision-formats-stale-save-001
*category: runtime · severity: medium · status: pre-existing, fixed in #354*

Found by a #354 inspect critic, PRE-EXISTING since #314: `reload_active_from_disk` re-mints the buffer nonce but RESETS the version epoch, and cleared only the INLAY in-flight state (#401) — `formatting_request` survived. A ⌘S on a clean-at-open buffer parks key (uri, v0); an agent rewrites the file; the activation choke CleanReloads (fresh buffer, v0 again); the late format response passes v0==v0, splices whole-file edits computed against the OLD text over the reloaded text, and the parked save WRITES the corruption to disk. The #401 comment named this exact collision class for inlays; the formatting key recurrence went unnoticed through #314/#332/#354 planning until an inspect trace. Fix: reload clears `formatting_request` too (response dies superseded; the surviving latch deadline plain-saves the RELOADED text, safe). Class rule: PR-claude-version-epoch-guards-invalidate-on-reload-001.

## F-claude-314-origin-guard-was-surface-granular-001
*category: runtime · severity: medium · status: residue of the shipped #314, fixed by #354*

The shipped #314 C1-HIGH fix guarded completions with `active_is_origin` = `active_editor().has_path(origin)` — but `has_path` spans ALL rows of the pane's surface, and within one project every open file shares ONE surface. So the guard it was built to be — "only save when the user is still ON the origin" — passed after any SAME-PROJECT file-row switch: viewing B with A open in the same surface, A's settle/deadline ran `do_plain_save` → `save_active` → wrote B (B's own content to B's own path — no cross-file corruption, but an UNREQUESTED save of a file the user may have wanted unsaved, plus a "timed out — saved" flash attributed to the wrong file), while A stayed abandoned-dirty. The C1 fix was verified only against cross-surface/cross-project switches; the same-surface case had no test (the origin-guard behavior had ZERO drives). #354's ContentId-granular binding closes the class; the pointed regression drive is `format_save_never_writes_the_nonorigin_active_editor_headless`. Lesson: an identity guard is only as fine-grained as its comparison key — "the surface holds the path" is not "the active view IS the origin".

## F-claude-413-nested-twin-cross-instance-outcome-delivery-001
*severity: high · category: state-integrity · status: fixed at inspect (pre-validate — never shipped)*

#413 generalized the LSP outcome drain from the active root to every host, dispatching with the owning
root. The design's per-arm sweep cleared all 8 focused-editor arms as safe via their focused-uri guards,
reasoning "canonical-under-root → None for another root's path". That mechanism was false at source:
`resolve_under_root` passes absolute paths through (`uri_for` is total), so the guards drop by URI
EQUALITY — FILE identity, not INSTANCE identity. Under NESTED roots (A=/proj, B=/proj/sub) the same file
opens under both roots as twins (D-OPEN-DEDUPE-SCOPE) with byte-identical uris; RootView-global request
latches survive project switches; twin version counters are independent so equal counts collide (#354 F3).
Consequences (all NEW with the drain change — pre-#413, dispatch root == active root == the focused
instance's root made the family structurally unreachable): a background root's F12 answer could execute a
jump inside the focused project (the definition arm has no version guard); a completion menu could open on
the focused twin and its accept apply A-computed ranges into B's diverged buffer; inlay / hover /
signature / code-action / prepare-rename / references could deliver cross-instance state (prepare-rename
worst case: a user-confirmed rename sent to the WRONG host at the other instance's position). Adjacent
pre-existing find: `apply_one_file` routed a twin's edits to the lexicographically-FIRST root's instance
while the version was validated against the OWNING root's host (the class #354 D12.5 fixed for
formatting). Fixes: `outcome_is_for_focused_root` on the 8 arms (drop before latch/UI, latch untouched);
owning-root-first instance routing in `apply_one_file`. Found by the state-integrity critic at §18.1;
lead-confirmed by tracing `resolve_under_root`/`uri_for` totality before accepting the shared fix
per-consumer (PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001 applied to the REMEDY).

## F-claude-418-selection-invariant-only-lived-in-one-components-handlers-001
*severity: high · category: ui-state · ticket: #418 (marley-web POC rail)*

The #418 single-selection rail enforced "exactly ONE row lights" only inside LeftRail's own
click handlers (each patching `activeProject`/`paneCellFocus`/`activeAgentId` resets), while
the fields deciding the selection are shared AppState that a dozen OTHER writers patch —
center surfaces (ManagerAgentSetup, SplitTerminalView), the command palette (splits,
toggle-two-projects, go-home), keyboard shortcuts (⌘\, ⌘⇧L), and utils (addToPane,
arrangements). Every foreign writer broke the invariant a different way: stale
`activeAgentId` double-lit (agent row + section row), stale `paneCellFocus` lit a cell the
user never chose or nothing (index out of range after a pane rebuild), palette-hiding
project 2 stranded `activeProject: 2` as a persistent zero-fill, and the center split view
wrote the cell ORDINAL into `activePane` where the rail reads a pane ID. The persistence
layer then embalmed each broken tuple (only `paneNaming` was sanitized at load). Found by
the #418 inspect critics; the old rail's always-lit project row had masked the whole class.

## F-claude-419-force-expand-protected-a-breadcrumb-not-the-selection-001
*severity: medium · category: ui-model · ticket: #419*

The #398 "focused cell is a CodeView ⇒ Editor is the active section" override was written when
SIX rows lit at once — whichever section force-expanded, some lit row stayed visible. #419's
single-selection model broke that assumption for the 1-cell-grid edge: a one-pane grid whose
cell is a CodeView selects its own TAB row (RailSelection::Tab — the >1 filter routes cell
selection only for real splits), but the un-gated override still declared Editor the active
section, so Editor force-expanded while the section actually HOLDING the selected row stayed
collapsible — one chevron click hid the selection with zero rings anywhere. The pre-change
breadcrumb (the CrossRef row's active fill) had died with the multi-light. Found by the #419
inspect two-fills critic tracing collapse interactions. Fix: gate the override on
`pane_ids().len() > 1` so force-expand always protects the section hosting THE SELECTION
(1-cell → the Tab row's own section; multi-cell → Editor for the cross-link breadcrumb).

## F-claude-420-projection-change-turned-a-red-pin-vacuous-not-red-001
*severity: medium · category: testing · ticket: #420*

Reversing the #237/#240 rail projection (the one "Editor" tab row → per-file basename rows)
made an existing collapse test VACUOUSLY GREEN instead of red: it asserted the collapsed
Editor section hides the code tab's row by grepping for the row's LABEL ("codefile" — the tab
title), and post-change no row carries a tab title there (rows carry file basenames), so the
absence assert passes whether or not the collapse skip works. The companion pins that asserted
PRESENCE went red and self-reported; the ABSENCE pin failed silent. A re-pin pass driven by the
red list alone would have shipped a dead test. Caught only because the #420 inspect model critic
ran the declared-red set and diffed it against the ACTUAL reds.

## F-claude-422-pure-walk-right-call-site-wrong-twice-001
*severity: high · category: shim-integration · ticket: #422*

The #422 divider fix put all the intelligence in a pure, correct, self-consistent descriptor
walk — and the ONE production call site fed it the wrong world twice: (1) raw `center_bounds`
where the panes tile over `inset_right(center_bounds, PANE_GUTTER)` — every strip sat
gutter·ratio off the true seam (fully outside its own 6px hit zone at ratio ≥ 0.75) and the
drag lagged the pointer; (2) gated on `try_workspace()`, whose documented FALLBACK (the first
terminal grid anywhere) let occluding strips paint over code/cockpit tabs and resize a hidden
grid. Both were invisible to the pure suite by construction — the walk agrees with
`pane_rects` "to the ULP" only when handed the SAME bounds, and no unit sees the shim's gate.
Found by the #422 inspect critics tracing the CALL SITE numerically against the tiling code
three lines above it.

## F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001
*severity: low · category: state-integrity · ticket: #415*

#393 extracted `close_transient_overlays()` from the launcher's closer list and converted the
two NEW-at-the-time call sites (launcher-open, section ＋ menu) — but never SWEPT the two
mouse-opened menus that PREDATED the helper (the #175 file-ref menu at app.rs:5513, the
terminal right-click listener at :19940). Both kept a hand-copied trio (completion /
renaming_tab / naming_pane) fossilized at their #398-era membership, silently missing every
overlay the roster gained since — the pickers, and #415's launcher. Live consequence: a
centered picker (⌘T symbols etc — single card, no scrim, terminal/link exposed) + a right-click
→ the menu opened with the picker keyboard-live underneath, the #229/#393 orphan class those
same comments cite. The #415 inspect correctness critic found it by grepping every multi-field
hand-closer against the roster. Fix: both sites now call the helper. Class rule (PR-claude-
mouse-opened-modal-must-close-all-transient-overlays-001) already existed — the failure mode is
extraction-without-sweep: when a shared roster is factored out, EVERY existing hand-closing
site must be converted in the same change, or greppable drift accrues at the sites the helper
was specifically invented to fix.

## F-claude-226-fire-and-forget-spawn-never-reaped-zombie-accumulation-001
*severity: medium · category: resource-leak · ticket: #226*

The #226 notification adapter's first shape spawned osascript and DROPPED the Child handle —
`cmd.spawn().map(|_child| ())` — assuming drop detaches. It doesn't: dropping a
std::process::Child never reaps, no SIGCHLD disposition exists in the tree, so every
notification left one <defunct> zombie for the app's LIFETIME. Monotonic in an explicitly
long-running terminal app; the eventual failure is app-wide — when the per-uid process cap
fills with zombies, EVERY later fork fails (new PTYs, git, wsl). Caught by the #226 security
critic, which also found the house already solves it three ways (app.rs reap thread,
marley_lsp wait-on-drop, visual-harness wait-on-drop) and that `open_url_with`
(marley_command/lib.rs:60) carries the SAME latent pattern at a much lower fire rate
(user link clicks — noted, unfixed, next ticket touching it owns it). Fix: detached reaper
thread — `let mut child = cmd.spawn()?; std::thread::spawn(move || { let _ = child.wait(); });`.

## F-claude-227-new-modals-exposed-two-more-unrostered-mouse-openers-001
*severity: medium · category: state-integrity · ticket: #227*

Adding the #227 param-prompt/delete-picker pair exposed TWO more instances of the
mouse-bypasses-the-key-ladder class, made sharper because a hidden picker's Enter now
DELETES AND PERSISTS: (1) the two mouse DIFF-openers (⌘-click a cockpit agent row; click a
git-panel path) set `diff = Some` without roster-closing — the diff painted OVER a
still-keyboard-live centered modal (Esc closed the hidden modal, not the diff; blind Enter
in the hidden picker deleted a workflow sight-unseen); (2) `begin_fleet_dispatch`'s
two-text-owners refuse-guard listed only naming_workflow/naming_pane, so the seat "send"
chip (a mouse listener) opened the dispatch card keyboard-dead over the new modals. Both
found by the #227 correctness critic tracing every arm above the new ones + every mouse
opener against the roster. Fixes: the two diff-openers call `close_transient_overlays()`
(the #393/#415 stance); the refuse-guard gained both new states. The recurring lesson
sharpens PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001: adding a NEW
transient overlay means sweeping (a) the roster + blocked + snapshot memberships AND
(b) every EXISTING mouse opener/refuse-guard that enumerates overlay states by hand —
the #415 sweep caught the hand-CLOSERS; this one caught the hand-GUARDS and the
still-unrostered opener.

## F-claude-426-adopted-rules-drift-three-ways-before-first-test-001
*severity: high · found at: #426 inspect (break-math critic, hand-traces) · status: fixed pre-commit*

wrap_breaks claimed to implement gpui LineWrapper's break rules and deviated three
ways at once, none caught by compilation: (1) the first_non_whitespace candidacy
gate was DROPPED, so an indented line broke AT its first word — a bare-indent
first row and an over-capacity carry; (2) overflow was an `if`, not a loop, so a
candidate rollback could land carried cells in a continuation whose capacity had
just SHRUNK by the indent with no re-check — a segment wider than the wrap width
it exists to enforce; (3) the word class used Unicode `is_alphanumeric`, which
counts CJK as word chars and silently defeated the any-break clause the reference
deliberately leaves CJK to. All three surfaced only under adversarial hand-traces
of the REFERENCE's own examples ("    indented words…", "Hello world你好世界").
When a spec says "rules adopted from X", the inspect must re-trace X's OWN
documented cases against the new code — matching the happy path is not adoption.

## F-claude-426-cheap-bound-inverted-tabs-strand-the-tail-001
*severity: high · found at: #426 inspect (three critics + lead independently) · status: fixed pre-commit*

The wrap pre-filter skipped the break walk when `2×chars + phantoms ≤ wrap_cols`,
justified as "a cell is at most 2 chars wide" — the claim is INVERTED (a char is
at most N cells), and false for tabs (one char, up to tab_width cells). A
tab-heavy long line was pre-filtered to a single segment AND rendered with the
h-scroll apparatus deliberately inert (wrap-ON pins scroll_x, zeroes content_w) —
the exact stranded-unreachable-tail failure #336 existed to kill, reintroduced by
the feature meant to finish it. Bound fixed to `chars × max(2, tab_width)`. Rule
of the class: an optimization bound stated as "sound because <inequality>" must
have the inequality written in the CORRECT DIRECTION in the comment, and a unit
fixture must sit on the boundary's falsifying input (here: a tab-indented line).

## F-claude-426-a-clip-undid-a-widening-the-off-path-depended-on-001
*severity: high (default-config regression) · found at: #426 inspect (rim critic, EOL trace) · status: fixed pre-commit*

The per-segment squiggle clip `de.min(seg_c1)` silently undid the pre-existing
`.max(ds + 1)` EOL widening that gives a zero-width end-of-line diagnostic
("expected `;`") its 1-cell bar — on the LAST segment seg_c1 equals the display
width, so the widened bar collapsed and the `ds >= de` guard dropped it, with
wrap OFF (the default). A "byte-identical when off" claim survives only if every
new clamp is checked against the widenings/asymmetries the clipped values already
carry (EOL widening, the #331 col_starts/display_cols phantom asymmetry). Fixed:
the right edge clips only at a REAL wrap boundary (non-last segments).

## F-claude-427-a-second-producer-broke-the-with-highlights-contract-001
*severity: high (silent wrong pixels — the ticket's core visual) · found at: #427 inspect (2 critics independently) · status: fixed pre-commit*

The multibuffer render APPENDED match-band byte ranges after the syntax ranges in
one `with_highlights` vec — unordered and overlapping. gpui's run walk is
strictly sequential (a range starting before the cursor emits its full length AT
the cursor, then regresses), so bands painted at the wrong bytes on any line
with a token at/after the match — essentially every code line, with no panic to
catch it. The house contract (ascending, disjoint, char-boundary — #266) lives
at the PRODUCERS (`highlight_ranges`, `styled_slices*`); a new `StyledText`
site composing TWO producers must merge through one of them or stay
single-producer. Fixed by parity, better than merging: the approved POC bands
the whole LINE, so the render feeds syntax-only highlights and washes the row —
the second producer deleted entirely.

## F-claude-427-b-insert-already-holds-the-first-view-001
*severity: high (a leak per materialization) · found at: #427 inspect (2 critics) · status: fixed pre-commit*

`open_multibuffer_from_search` called `acquire_view` right after
`content.insert(...)` — but the registry's `insert` ACQUIRES the first view
(views = 1), so the tab's single close-release left the boxed snapshot (up to
500 files of excerpt text) stranded at views = 1 forever; no reaper covers the
kind. The exact trap is documented at `open_browser_tab` ("copying that shape
here would strand a view forever") and STILL got copied — a doc warning at one
site does not guard the next site; only the review grep does (see
PR-claude-427-a). Fixed: the insert's view IS the tab's view (the
`resolve_open` miss-arm shape), with the contract stated where the acquire was.

## F-claude-427-c-a-new-fn-spliced-between-doc-and-item-001
*severity: medium (silent mutation-mask strip) · found at: #427 inspect (2 critics) · status: fixed pre-commit*

`multibuffer_body` was inserted BETWEEN `cockpit_body`'s doc comment +
`#[cfg_attr(test, mutants::skip)]` and its `fn`. Docs and attrs attach to the
NEXT item: the cockpit shim silently lost its mask (and its rustdoc opened the
new fn), while the new fn carried a duplicate attr. Nothing compiles
differently — only a later dev-box mutants sweep would have surfaced the
unmasked shim as MSI churn. Fixed: the new fn moved wholly above the neighbor's
doc block, one attr each.

## F-claude-428-a-registered-nowhere-the-input-seam-is-per-surface-001
*severity: critical (the feature's core interaction dead) · found at: #428 inspect · status: fixed pre-commit*

The multibuffer's typing arm was wired into `replace_text_in_range` — and never
fired: gpui's text-input seam is REGISTERED PER FRAME by a paint-scoped canvas
(`window.handle_input(ElementInputHandler)`), and only the editor body carried
one. A new editable surface must register its own handler in its own body;
adding an arm to the `EntityInputHandler` methods is necessary but NOT
sufficient. Headless drives that call the method directly mask the gap — only
the live drive (or knowing this rule) catches it.

## F-claude-428-b-a-transient-buffer-selection-is-not-transient-001
*severity: high (silent cursor corruption) · found at: #428 inspect (2 critics) · status: fixed pre-commit*

The mb edit arms set a single-caret `SelectionSet` on the shared buffer and
relied on D-OPEN-SELECTION-HOME to protect the file's own views. But the
park/restore choke fires only on editor-view CHANGES — an mb frame is not one,
so `last == current` on return and the user's cursors (multi-cursor sets
included) were silently replaced; the twin-view path could park the corruption
permanently. Any non-editor surface borrowing a buffer's live selection must
SAVE and RESTORE it around the edit itself — the choke will not save you.

## F-claude-428-c-journal-entries-need-a-liveness-guard-not-just-order-001
*severity: high (foreign-history undo) · found at: #428 inspect (2 critics) · status: fixed pre-commit*

The cross-excerpt undo journal recorded (id, depth) but popped UNGUARDED: no-op
edits journaled phantom entries, coalesced continuations duplicated entries
(adjacent-only dedupe), and tab-side edits/undos made entries stale — each path
ended with mb ⌘Z reverting history the multibuffer never made. A routing journal
over a shared history must (1) note only REAL edits (version changed), (2) key
one entry per undo GROUP (retain-then-push on the (id, depth) key), and (3)
verify at pop that the target's depth still matches before undoing — stale
entries drain, never fire.

## F-claude-429-a-apply-must-fail-closed-when-its-surface-did-not-open-001
*severity: high (silent rewrite of unpreviewed text) · found at: #429 inspect (both critics) · status: fixed pre-commit*

Replace All ran `open_multibuffer_from_search()` then took `active_mb_id()` —
but the materialize NO-OPS (overlay stays) on an empty/streaming/unsourceable
set, and the active tab behind the modal was whatever OLD multibuffer the user
had open: the new query applied over the old file list. A gesture that targets
"the surface I just asked for" must verify that surface actually OPENED (here:
the overlay's `open_search == None` is the success signal) and fail closed
otherwise — resolving "the current X" after a fallible open is the same
re-derive-identity-at-completion trap as the #314 lost save. The sibling find:
the apply consumed empty regex matches the walk deliberately hides (`a*`:
1 shown, 3 spliced) — producer and consumer of "a match" must share ONE filter.

## F-claude-430-a-per-root-source-recipes-break-on-workspace-wide-aggregations
*severity: MED · found: #430 inspect (self-review) · subsystem: problems multibuffer / content sourcing*

The diagnostics materialize copied the search materialize's live-buffer
override map verbatim — `editors_under(active project root)`. Search results
are root-scoped by construction, but `problem_rows` aggregates EVERY LSP
host: a diagnostic in another project's file sourced STALE DISK text while
its live buffer differed, silently (no error — the mb just showed old code).
Fix: workspace-wide aggregations build their override map over ALL open
editors (`content.iter()`), any root. Class: when a consumer's input domain
WIDENS (per-root → workspace), every helper recipe copied from the narrow
consumer must be re-audited for the same scope assumption.

## F-claude-430-b-cross-project-target-birth-split-brains-the-buffer
*severity: HIGH · found: #430 re-verify critic · subsystem: multibuffer target lifecycle*

The scope-widening class (F-claude-430-a) had a SECOND layer: fixing the
TEXT-sourcing override map workspace-wide still left the pin/birth paths
(`pin_mb_targets`, `ensure_mb_target`) blanket-resolving under the ACTIVE
project root. A cross-project diagnostic's first caret click called
`open_editor_instance(active_root, path)` — `find_open` is deliberately
root-scoped, so it missed the file's REAL open instance and minted a second,
independently-live one from disk: mb edits never reached the real tab, and
whichever saved last silently clobbered the other. Fix: `root_of_mb_path`
(longest project-root prefix, active fallback) threaded through pin + birth.
Class rule: when an input domain widens (per-root → workspace), audit EVERY
root-taking helper on the consumer's path — the sourcing map, the dedupe
lookup, AND the birth — one missed layer re-opens the hole.

## F-claude-432-a-composition-state-outlived-the-caret-that-owned-it-001
*severity: high (caught at inspect, before ship) · category: IME / state lifecycle · pipeline 432*

The mb composition span (`mb_marked`, a RAW OFFSET pair in app state) was
supposed to die with every caret-drop gesture — the #428 notes even CLAIMED
"cleared at arrows/Esc/undo/redo/place-caret". Inspect proved the claim false at
three seams: `mb_place_caret` never cleared (the claim was simply wrong —
verified against `git show HEAD`), the chord-verb choke cleared only the
EDITOR's composition (`active_editor_mut()` else-arm missing — a tab switch left
the mb span standing while the mb caret survived the switch), and the epoch
re-mint dropped the caret but kept the span. Each stale span misdirects the next
IME commit through `edit_target`'s marked-first preference — clamped, so
corruption-shaped, cross-file in the worst case. Fixed by making caret and
composition die TOGETHER at every seam (click, chord choke, ←/→/⌫/⌦/⏎, epoch).
Lesson inside the failure: a lifecycle CLAIM in notes is not a lifecycle — grep
the sites before citing them.

## F-claude-432-b-mb-plain-keys-double-delivered-into-the-hidden-terminal-001
*severity: high (pre-existing since #428; caught at inspect) · category: input routing / fail-closed · pipeline 432*

Typing a printable on a multibuffer tab delivered TWICE: the platform text path
fed the mb caret (correct) while the same keystroke fell the whole key ladder to
the R29 tail and typed into the INVISIBLE workspace terminal's prompt — or
streamed bytes to a live PTY mid-command; shift-arrows scrolled the hidden
scrollback. Invisible precisely because the pane is hidden, and no mb test
asserted prompt purity. The #432 ⌘C/X/V arm closed the chord lane; the critics'
five-scenario ladder trace exposed the plain lane (and the ⌘⇧V shifted residual
in the same sweep). Fixed with the #251 editor claim's exact mb twin (bare
return, no stop_propagation — the platform text path must keep delivering).
The EDITOR-tab ⌘⇧V residual (editor block keeps `!shift`) remains recorded as
follow-up.

## F-claude-435-a
severity: high — spec-violating logic bug caught at inspect
The #435 F8 target derivation picked "the latest failed run block EVER"
(`rposition(is_run && failed)`), so REQ-004's fail→rerun→PASS scenario kept
targeting the superseded failure — "derive at press time" only equals
"clears on green" when the derivation MODELS supersession. The fix feeds the
pure fn `(run_tag, StatusKind)` per block and skips a Failure whose tag has
a later same-tag Success in the pane. **Why it slipped:** the implementer
simplified the design's `(bool, StatusKind)` input to `(bool, bool)` — the
type simplification silently DISCARDED the information supersession needs
(who succeeded, and with which identity). A pure-fn input type that drops a
field the spec's acceptance criteria mention (REQ-004 says "superseded") is
the bug's shape. Also: the same inspect caught `cargo check --tests` being
read as "tests pass" (a stale keymap pin was red) — check compiles, nextest
RUNS.

## F-claude-443-a-full-mutation-workers-shared-one-target-001
*severity: high · category: gate integrity · pipeline 443*

The fork's FULL gate ran cargo-mutants in copy mode with two workers, and both copies inherited
the shell's `CARGO_TARGET_DIR` (`/mnt/fast/target`). Cargo names a workspace path package's
artifacts by a hash that leaves out the checkout's location, so both copies built `marley_mcp`
into the one `marley_mcp-51cde4ca6951f9f4` test binary. Between one worker's build and its test
run, the other worker rebuilt that binary with its own mutant: the log for `tools.rs:71`
(`surface_result -> Default`) shows nextest compiling a binary that warns about `transport.rs:53`
(`unused variable: f`), the other worker's Debug mutant, and the mutant was reported MISSED
although an existing test kills it. The same race can report a survivor as CAUGHT. It also
poisoned the main tree: the copies' last builds stayed in the shared target with fingerprints
cargo accepted for the real sources (the dep-info paths are relative to the package root), so
afterwards `cargo nextest run -p marley_terminal` ran a mutated binary and
`session_id::tests::r2_u64_roundtrip` failed on correct code until `cargo clean -p` of the five
Marley crates. The gpui era also ran FULL with two jobs on a shared target, so its FULL verdicts
may carry the same noise; DIFF verdicts (in place, one job) never did. Fixed in #443: each FULL
copy builds in its own target directory.

## F-claude-443-b-the-port-stripped-the-mutation-masks-001
*severity: medium · category: gate integrity / port · pipeline 443*

The 2026-09-18 port removed the `mutants::skip` attributes from the Marley crates (the fork does
not carry the `mutants` crate) while the fork's constitution says mutation runs with no
exclusions. `marley_mcp/src/transport.rs` had been masked wholesale and the raw PTY shim
`marley_terminal/src/pty_os.rs` partly, so 44 transport mutants and five shim mutants survived
the first FULL run. Nothing flagged it at port time: the port ran tests, clippy and fmt, not
mutation. Fixed in #443 with loopback tests that drive a real server and real-PTY tests that
tell the program, the working directory, the window size and the reap apart.

## F-claude-443-c-diff-mutation-mutated-no-marley-crate-001
*severity: critical · category: gate integrity / port · pipeline 443*

`script/gates.sh --diff` ran `cargo mutants --in-diff` from the workspace root with no `-p` and
no `--workspace`. The fork's root manifest inherits Zed's `default-members = ["crates/zed"]`,
and cargo-mutants 27 mutates only the default members unless told otherwise, so a diff of 33
Marley files matched no mutant, cargo-mutants exited 0 with "No mutants to filter" before
writing `mutants.out`, and the gate printed "mutation: no mutable lines in the diff — pass".
Every DIFF green since the port would have tested nothing. The gpui-era root had no
`default-members`, so the port brought the hole in and nothing flagged it. Found by the #443
gate critic; confirmed with `cargo mutants --list --in-diff` (0 mutants without `-p`, 603 with
it). Fixed: DIFF passes `-p` for every touched package and fails closed when the diff has crate
lines but no package resolves.

## F-claude-443-d-pipefail-hooks-raced-sigpipe-again-001
*severity: high · category: hooks · pipeline 443*

`PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001` already named this class, and 11 sites in
three fork hooks still fed a producer into `grep -q` under `set -o pipefail`. When grep exits
at its first match the producer takes SIGPIPE and pipefail reports failure. In
`enforce-warp-reference.sh` that turned a present `## Reference (§20)` into "missing" (9 of 20
runs over the 409 staged specs blocked, each time naming a different valid spec). In
`enforce-commit-gate.sh` and `enforce-changelog.sh`, where the pipeline ends `|| exit 0`, it
turned a match into "allow": the commit gate let a 64 KB commit command through without a
receipt 46 times in 100. Inputs under 16 KB never raced, which is why the hooks looked sound.
Fixed with here-strings, which have no concurrent writer.

## F-claude-443-e-a-hup-ignoring-test-child-outlived-its-test-001
*severity: medium · category: tests / real PTY · pipeline 443*

`teardown_kills_a_child_that_ignores_hangup` ran `trap '' HUP; while :; do sleep 1; done`.
alacritty calls `setsid` in the PTY child, so nextest's process-group kill never reaches it,
and it ignores the HUP a closing leader sends. Whenever its test process died first (a mutant
that hangs `Drop`, one that spins `pump`, a Ctrl-C) the shell lived on: three were found
reparented to `systemd --user`, each forking `sleep` every second and holding a pts and cargo's
jobserver pipe. The same test waited a fixed 300 ms before hanging up; a slow start let the HUP
beat the trap, the shell died on it, and the test passed without reaching SIGKILL, which would
let `Drop -> ()` survive. Fixed: `echo trap-set; exec sleep 30` bounds the child's life, and the
test waits for `trap-set` before it hangs up.

## F-claude-443-f-the-sse-stream-read-its-version-after-the-head-001
*severity: low · category: MCP transport · pipeline 443*

`serve_sse_stream` wrote the response head and only then read the snapshot version it would
wait on, so a change landing between the two was never pushed to that stream: an
under-notify, which the module's own design rules out. The loopback tests also gave the head
and each expected push only the 300 ms quiet window, so a slow box could fail the baseline or
count a mutant as caught. Found by the #443 test critic. Fixed: the version is read before the
head goes out, and the tests wait up to the reply deadline for a whole message.

## F-claude-443-g-diff-mutation-was-a-usage-error-001
*severity: high · category: gate integrity / port · pipeline 443*

The fork's DIFF gate called `cargo mutants --in-diff mutants.diff --in-place --jobs 1`, and
cargo-mutants 27.1.0 rejects `--jobs` beside `--in-place` at argument parsing ("the argument
'--in-place' cannot be used with '--jobs <JOBS>'", exit 1). The gate discards cargo-mutants'
output and maps exit 1 to "did not complete a valid run", so the first real `--diff` run in the
fork showed gate:5 red with no reason given. It failed closed, but no DIFF green was ever
possible in the fork. The inspect critic had proven the missing `-p` with a `--list` run that
left out `--in-place` and `--jobs`, so the finding was right and the check still missed this.
Only the gate's own invocation found it. Fixed: DIFF drops `--jobs`, since in place is one job
by construction. Class: prove a gate change with the gate's exact command line, not a
reconstruction of it.

## F-claude-436-a-a-path-hook-judged-files-by-the-sessions-directory-001
*severity: high · category: hooks · pipeline 436*

The first draft of `enforce-zed-ledger.sh` decided whether a file was in this repository by
comparing its path with `PROJECT_ROOT`, the git root of the session's working directory. Claude
Code runs command hooks in the session's current directory, which follows a Bash `cd` and
EnterWorktree, so from `/`, a scratchpad, another repository or a linked worktree every write to
a Zed file in the main checkout passed. A regex strip of `.claude/worktrees/<name>/` also broke on
names with a `/`. Found by the #436 path critic. Fixed: the hook asks git about the file itself
(`rev-parse --show-toplevel` and `--show-prefix` from the nearest existing directory) and gates
it only when its `--git-common-dir` is this repository's, against that checkout's own ledger.

## F-claude-436-b-a-gate-only-rule-was-not-checked-at-commit-001
*severity: medium · category: hooks / gate integrity · pipeline 436*

gate:16's ledger check ran only when `script/gates.sh` ran. The commit hook demands a receipt
only when `.rs` files change, and the receipt fingerprints neither the ledger nor a non-Rust Zed
file, so a keymap or asset change made with Bash could be committed without its row; gate:16
also read the work tree while a commit ships the index. Three docs still said gate:16 catches
any such change. Found by the #436 gate critic. Fixed: `enforce-commit-gate.sh` runs the ledger
check on every `git commit`, and the check lists the index, the work tree and untracked files.

## F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001
*severity: high · category: gpui / workspace sidebar · pipeline 438*

The rail's `Sidebar::has_notifications` read a snapshot that only `Rail::render` assigned. The
`MultiWorkspace` draws the sidebar only while it is open, and the status bar asks
`has_notifications` for its toggle's dot only while the sidebar is closed. So in the one state
where Zed read the flag, nothing refreshed it: a bell never lit the toggle, and a dot showing at
close stayed after the bell cleared. The planned bell test ran with the rail open and would have
passed. Found by the #438 correctness critic. Fixed: `refresh`, which runs on every event the
rail hears, builds and stores the snapshot, `render` draws from it, and the flag reads it.

## F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001
*severity: medium · category: workspace sidebar / persistence · pipeline 438*

The first layout switch built a new sidebar on every swap and dropped the old one. Switching to
the Marley layout serialized the window at once with the rail registered, whose
`serialized_state` was the trait default `None`, so Zed's saved sidebar width and History view
left the database; the swap back built a fresh Zed sidebar, never restored it, and reopened it
because the rail had forced the sidebar open. Dropping Zed's sidebar also cancelled the tasks it
owned, such as restoring an archived thread's worktree midway. Found by the #438 state critic.
Fixed: the rail keeps Zed's sidebar entity with its open flag, answers `serialized_state` from
it, and hands it back on the swap to Zed.

## F-claude-438-c-a-marley-helper-paraphrased-gpl-code-001
*severity: medium · category: provenance · pipeline 438*

`display_names` in the rail, an MIT OR Apache-2.0 crate, followed `sidebar.rs:1451-1462` (GPL)
statement for statement with the names changed; a fully written `std::collections::HashMap`
path that the file had already imported gave it away. The Phase 2 notes said "written fresh".
The code was glue the API forces, but the crate's license label holds only if nothing GPL is in
it. Found by the #438 upstream-discipline critic. Fixed: rewritten from the public contracts
(`compute_disambiguation_details`, `ProjectGroupKey::display_name`) in the rail's own shape.

## F-claude-447-a-warning-check-read-quiet-cargo-output-001
*severity: medium · category: quality gate · pipeline 447*

gate:14's new check fails the run when the doc build prints a `warning:` line, and the first
version ran `cargo doc --quiet`. `--quiet` makes cargo drop its own warnings, such as an unused
manifest key or an output filename collision, so the only warnings left for the check to see
were rustdoc's, which `-D warnings` already turns into errors. The check could not fail. Found
in the Test phase while writing its negative smoke, which plants an unused manifest key, before
the gate shipped. Fixed: gate:14 runs `cargo doc` without `--quiet`, and the planted key now
turns it red.

## F-claude-441-a-task-provider-read-the-workspace-inside-its-update-001
*severity: high · category: gpui re-entrancy · pipeline 441*

The first `RoutedTerminals::spawn` called `TerminalPanel::spawn_task` at once. The workspace
calls its `TerminalProvider` from `Workspace::spawn_in_terminal`, inside its own update, and
`spawn_task` reads the workspace (`terminal_panel.rs:638-642`). Every task in the fork would
have panicked: "cannot read workspace::Workspace while it is already being updated". Found by
the Code phase's review before any test ran; the REQ-001 test, run against the first version,
reproduces the panic. Fixed: the provider spawns on the window and calls the panel from there
with `update_in`.

## F-claude-441-a-rerun-reopened-the-hidden-terminal-panel-001
*severity: medium · category: behavior · pipeline 441*

`spawn_task` reruns a task in its last terminal wherever that terminal is: `terminals_for_task`
searches the panel's panes and the center's (`terminal_panel.rs:779-820`), and
`replace_terminal` reveals the pane that holds it (`:1121-1127`). In the Marley layout, a task
that last ran in the Zed layout reran in the hidden panel and opened it, against REQ-001. Task
terminals are never saved (`persistence.rs:67`), so only a switch within one session hits it.
Found in the Complete phase, while checking the CHANGELOG's claim that Zed's rerun rules hold
in the center; no test had rerun a task. Fixed: in the Marley layout the provider first moves
the task's terminals from the panel to the active pane (`workspace::move_item`), and the switch
test reruns a task that last ran in the panel.

## F-claude-442-a-close-deferred-on-the-rail-ran-inside-the-rails-update-001
*severity: high · category: gpui re-entrancy · pipeline 442*

The rail's `restore_serialized_state` runs inside the `MultiWorkspace`'s update, so the first
version deferred the restore's close with `cx.defer_in(window, |rail, window, cx| …)`. That
callback runs inside an update of the rail. `close_sidebar` reads its sidebar
(`sidebar_side`, `multi_workspace.rs:331`), so every restore of a closed rail would have
panicked: "cannot read marley_workbench::rail::Rail while it is already being updated". Found
in the Code phase by the new driven test, which restores through Zed's own
`apply_restored_multiworkspace_state`. Fixed: the close defers with `window.defer`, which runs
with no entity leased, and captures only the `MultiWorkspace`'s weak handle.

## F-claude-453-the-rail-missed-a-projects-last-folder-going-001
*severity: low · category: behavior · pipeline 453*

The rail rebuilds on each workspace's `workspace::Event` and on the `MultiWorkspace`'s events
and notifies. Removing a project's last folder reaches neither: the workspace's
`project::Event::WorktreeRemoved` arm emits no event (`workspace.rs:1759-1763`), and
`MultiWorkspace::handle_project_group_key_change` returns early on an empty key without a notify
(`multi_workspace.rs:615-628`). The project's row stayed in the rail until another change
rebuilt it. Zed's Threads Sidebar subscribes to each project's worktree events
(`sidebar.rs:1005-1031`). Found in the Code phase while writing the test for Enter with no row
highlighted, which reaches that state only after another change in the window. Fixed in #458:
the rail follows each project's folder events
(`AD-claude-458-the-rail-follows-each-projects-folders-with-a-deferred-rebuild-001`).

## F-claude-454-recency-noted-a-terminal-the-user-never-went-to-001
*severity: medium · category: behavior · pipeline 454*

The first version noted each change of the terminal or thread the window *showed*: the focused
Agent Panel's thread, else the displayed workspace's active terminal. Opening a thread from the
rail activates its project first. That project's active terminal was then the window's row for
one rebuild, before the Agent Panel took focus, so it was noted as more recent than the
terminal the user actually came from. `ctrl-tab` would then have gone to a terminal the user
never chose. Found in the Code phase by
`threads::the_switcher_opens_a_thread_and_opening_it_notes_nothing`, the first time it ran.
Fixed: the window's row counts a terminal only while it holds the window's focus
(`Focus::terminal_focused`), as a thread already needed the panel's focus. The planned guard
against noting while the switcher is open then protected nothing, and was removed.

## F-claude-456-a-layout-round-trip-closed-the-right-dock-and-lost-its-panel-001
*severity: medium · category: behavior · pipeline 456 (found in #438 inspect S9)*

The Marley layout switch only patches `agent.dock`, and each dock's own settings observer moves
the Agent Panel. When the Agent Panel was visible, the dock it entered opened on it and forgot
the panel it showed (`crates/workspace/src/dock.rs:638-705`); the dock it left closed with no
active panel (`:895-926`). With the right dock open on another panel and the Agent Panel open on
the left, a switch to the Marley layout and back left the right dock closed, its panel lost,
and that state was then saved. Found by #438's inspect, and carried through #442 and #451.
Fixed: the switch notes every dock before the move, and afterwards gives the dock the Agent
Panel leaves the panel it showed before the trip, while the Agent Panel was still what that dock
showed. `a_round_trip_leaves_each_dock_as_it_was` fails without the fix
(`left: … (false, None)`).

## F-claude-467-a-marley-shells-title-showed-its-rcfile-001
*severity: medium · category: behavior · pipeline 467 (found in #463's live effect, 2026-09-23)*

#463 starts each local bash as `bash --rcfile <data dir>/shell_integration/marley.bash`, and
Zed's `Terminal::title` names the foreground process with every argument after the program
(`crates/terminal/src/terminal.rs:3086-3095`). Every shell tab and every terminal row in the rail
then read `marley_ide — bash --rcfile /home/…/marley.bash`. #463's live check read the running
shells in `ps`, not their titles on screen, so it passed. Found in the capture taken before the
rail restyle. Fixed: the title lists the arguments through
`marley_terminal::shell_integration::shown_arguments`, which leaves out the run `for_program`
adds. `marley_bash_is_titled_without_the_integrations_arguments` fails without the fix
(`left: ".tmpW4qYkg — bash --rcfile /home/cpeppers/.local/share/marley/shell_integration/marley.bash"`).

## F-claude-465-gate4-counted-lines-from-a-stale-executable-001
*severity: medium · category: gate · pipeline 465 (fix: TICKET-469)*

gate:4 went red on #465 with 45 missed lines in `marley_terminal/src/shell_integration.rs`, all
on doc comments, a derive and a blank line. The tree's own tests covered every line. The
coverage JSON held two instrumented copies of `marley_terminal`: the test build, and a regular
build whose `shell_integration.rs` regions sat on the lines of the file as it was at #467. That
copy came from `marley_workbench`'s test executable, built in #468's coverage run and still in
`/mnt/fast/target/llvm-cov-target/debug/deps`. cargo-llvm-cov cleans the artifacts of the
packages it runs but reads every workspace test executable in its target directory, and this
one's line map was of the old source, with no counts from this run. Removing it, the only
stale executable naming that file, gave the tree's report. TICKET-469 makes the gate
independent of what earlier runs left.

## F-claude-474-the-occluding-buttons-hid-under-the-pointer-001
*severity: medium · category: behavior · pipeline 474 (found by the driven test, before commit)*

The plan had the hover buttons `occlude`, so the terminal would not take a click on them. The
first driven click copied nothing. With the pointer on Copy, the button's `BlockMouse` hitbox
stopped the hit test, the block's hitbox behind it was no longer hovered, the group's hover
ended and `visible_on_hover` hid the button, so the click met no listener. Fixed: a wrapper
stops the press instead (L-claude-474-an-occluding-child-ends-its-groups-hover-001).

## F-claude-474-a-press-elsewhere-lost-its-release-over-a-button-001
*severity: low · category: behavior · pipeline 474 (found in Test, before commit)*

The wrapper that replaced `occlude` stopped the left release as well as the press. A press on
the terminal's text released over a block's button then never reached the terminal: a
selection drag never ended (its phase stayed `Selecting` and copy on select did not run), and
with mouse reporting on, the program got a press and no release. The test at the time read the
input log and could not see it (L-claude-474-the-input-log-misses-mouse-reports-001). Fixed: the
wrapper stops the press alone, and the button's click stops its own release. The test's press
on the output, released over Copy, fails with the release stop put back (`left: [Some(32)]`,
`right: [Some(32), Some(35)]`).

## F-claude-474-rerun-would-have-run-a-command-that-output-printed-001
*severity: high · category: security · pipeline 474 (found in the Code review, before commit)*

Blocks come from hook frames, and the terminal took any frame it read, from the shell or from
the output of a program. The first Rerun sent the recorded command of any finished block. A
file shown with `cat`, a remote host over ssh or a log line could print a `precmd`, the text
`$ make test` and a `preexec` whose command was another one, then a `precmd` to finish it: a
block that looks like `make test` with a failed pill, whose Rerun would run the other command
in the user's shell. Found by the security lens of the diff review, reading what Rerun sends.
Fixed within #474: the terminal's nonce
(AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001), and Rerun only
for a verified command. `marley_no_rerun_for_a_command_that_output_printed` fails without the
check (`assertion failed: cx.debug_bounds("marley-block-rerun-1").is_none()`).

## F-claude-481-the-rich-input-dropped-every-typed-character-on-linux-001
*severity: high · category: behavior · pipeline 481 (found by the e2e scenario, before commit)*

The rich input's container stopped every key event unless `prefer_character_input` was set with
a character, to keep the terminal view's `key_down`, on the view's root, from sending the keys
to the program. gpui's Linux platforms never set `prefer_character_input` (`gpui_linux`, Wayland
and X11, always `false`), so no key reached the editor's text input: the editor grew with
Shift-Enter, which is an action, but stayed empty, Enter sent nothing and Escape kept no draft.
The first e2e run showed it; the in-process test harness does not go through that path. Fixed:
the container stops only the keys the terminal maps (`to_esc_str`: chords with Ctrl, Alt or
Super, and keys that type nothing), and text goes on to the editor
(L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001).

## F-claude-485-a-shell-started-at-the-debug-size-misdrew-its-first-long-prompt-001
*severity: medium · category: behavior · found by #483's e2e runner, fixed in #485*

Zed opens every PTY at `TerminalBounds::default()`, 100 × 6, and the view resizes it at its
first layout. A shell that reached its first prompt before that layout had readline lay the
prompt out for 100 columns; after the resize, readline's multi-line redraw (`redraw_prompt`)
restored that layout, and each key after it was drawn against it: `\r`, the whole prompt, a
run of backspaces, the key. With starship's two-line prompt on a deep path, only the first typed
character showed and the cursor sat inside the prompt, while bash ran the whole command. A
plain-pty reproduction sized before bash started never shows it; a `script` log inside Marley
does. Fixed: a new PTY opens at the size the last PTY terminal was given; the first terminals of
a launch still open at the debug size (TICKET-486).

## F-claude-488-a-signal-skipped-the-e2e-cleanup-001
*severity: medium · found in: pipeline 488's Test phase · class: cleanup on a signal*

A browser scenario hung on a bare `wait` (it waited for the harness's own keyboard holder, a
background job of the same shell), and `timeout` ended the run with SIGTERM. `script/e2e.sh`
trapped only EXIT, which bash does not run when an untrapped signal kills it, so the run's
headless sway, its Marley and its Chromium unit stayed up, with the profile copy on disk. Fixed
in the harness: `trap 'exit 130' INT TERM HUP` beside the EXIT trap, so a signal ends the run
through its cleanup; the header says to wait for a background step by its pid.

## F-claude-489-the-browser-decoded-frames-unoptimized-001
*severity: medium · found in: pipeline 489's Test phase · class: a per-frame path in a debug build*

The first latency report through Marley was a median of 114 ms from a key to its frame, against
6 to 7 ms at the protocol. Timing the decode found each 1100 by 900 JPEG frame taking 130 ms in
the debug build, where `image`, `zune-jpeg` and `marley_browser`'s pixel loop ran unoptimized;
a key waited behind one decode or two. The debug build is the one Chad runs. Fixed by building
`image`, `zune-core`, `zune-jpeg` and `marley_browser` at `opt-level = 3` in
`[profile.dev.package]`, as Zed does for its own hot crates: median 20 ms, 95th percentile 36 ms.

## F-claude-490-the-offline-resolver-rule-mapped-the-loopback-address-001
*severity: low · found in: pipeline 490's Test phase · class: an e2e fixture that blocks what it serves*

`offline_chromium` started the run's Chromium with `--host-resolver-rules="MAP * ~NOTFOUND,
EXCLUDE localhost"`, on the belief that an IP literal never reaches the host resolver. It does:
the rule mapped `127.0.0.1` too, and every page of the first run, the fixture's own site
included, came back as Chromium's "This site can't be reached" with `ERR_NAME_NOT_RESOLVED`.
Fixed by excluding `127.0.0.1` beside `localhost`. The run still showed one thing working:
the address bar kept the typed URL over the error page.

## F-claude-491-the-root-gitignore-hid-the-plugins-mcp-json-001
*severity: medium · found in: pipeline 491's Code phase · class: a file the build embeds that git ignores*

The plugin's new `marley/.mcp.json` never showed in `git status`: the root `.gitignore` ignores
every `.mcp.json` (a local MCP config carries bearers). `include_str!` still found the file on
disk, so the build, clippy and a local run would all have passed, and the commit would have
left it out: a clean checkout fails to build, and a plugin written from an older tree declares
no MCP server. Caught by reading `git status` after writing the file. Fixed with an exception
after the rule, `!crates/marley_workbench/claude_plugin/marley/.mcp.json`, and the ledger's
`.gitignore` row.

## F-claude-492-the-observers-came-on-after-the-page-showed-001
*severity: medium · found in: pipeline 492's Test phase · class: readiness announced before its watchers run*

`browser_network` listed neither the page's document nor its `/api` fetch, though the console
had the fetch's 404. `BrowserHub::attached` set the hub to Showing, and only then spawned the
task that turned on Runtime, Network and Log in the page's session. The agent's first
`Page.navigate` was waiting for Showing, and it reached Chromium between `Runtime.enable` and
`Network.enable`. The small fixture page loaded and fetched before the network was watched.
The console looked whole only because Runtime and Log replay what they buffered. A probe on a
scratch Chromium showed every request reported on the page's session when the domains were on
first. Fixed: `start` awaits `Page::observe` before `attached` announces the page.

## F-claude-493-a-closed-page-was-closed-again-by-its-tabs-removal-001
*severity: low · found in: pipeline 493's Code phase (the review) · class: a teardown that undoes itself twice*

When a page closed elsewhere (a script, an agent, another DevTools client), the hub's
`PageClosed` removed its Browser tab from its pane, and the tab's `Item::on_removed`, written
for the user's close, sent `Target.closeTarget` for the page again. Chromium answered "no such
target", logged as an error for every page an agent closed, and the hub's `closing` list kept
the page's id for good. Fixed before the Test phase: `close_tabs` has the tab forget its page
before it removes the tab, so the removal closes nothing.

## F-claude-493-a-page-behind-a-tab-kept-chromiums-default-size-001
*severity: medium · found in: pipeline 493's Test phase · class: what an agent reads is not what the user would see*

A page is laid out at its tab's size when the tab paints. An agent's page, opened behind the
user's tab so the focus stays, never painted, and kept Chromium's default viewport of 780 by
493: the first run's `browser_look --tab` answered at that size while the tabs around it were
1100 by 860, and the page would have laid out again the moment the user looked at it. Fixed:
a page takes the viewport of the page it opens beside (its opener, else the page the user
focused last) when it is attached; the rerun's look answered 1100 by 860.

## F-claude-494-waiting-tabs-gave-up-on-a-failure-001
*severity: medium · found in: pipeline 494's Code phase (the review) · class: a waiter that ends on a transient failure*

Once a start made no blank page, a tab opened while the browser starts, and a tab restored at
launch, each waited for the hub with `showing`, which answers a failure at once, and ended
there. After `marley: open browser` started the browser again the tab would have waited for
ever for a page nobody opened for it, saying "Opening a page…". Fixed before the Test phase:
both wait on `shown`, which polls until the hub shows its pages however long a failure lasts,
in a task the tab holds, so the wait ends with the tab.

## F-claude-494-a-created-page-with-a-cross-site-iframe-stopped-its-screencast-001
*severity: medium · found in: pipeline 494's Test phase (the #488 regression) · class: an emulated size the real surface does not have*

`488-04-wider` showed the page at 1100 pixels after the right dock closed, twice. Chromium had
the page at 1340 (the page said so), and the tab still drew the old frame, only when the page
held a cross-site iframe. #494 made the first page a `Target.createTarget` page, and a probe
on a scratch Chromium showed such a page, with an out-of-process iframe, sending no screencast
frame after `Emulation.setDeviceMetricsOverride` in four of six runs, and a page Chromium
opened at start in none of five. A created page sits in a headless window of its first size
(780 by 580) whatever the override. Fixed: `Page::set_viewport` gives the page's window the
viewport's size (`Browser.getWindowForTarget`, `Browser.setWindowBounds`) before the override,
four of four in the probe and every rerun since. The stall had hit every page Ctrl+T or an
agent made since #493.

## F-claude-494-the-relaunched-marley-got-no-keymap-001
*severity: low · found in: pipeline 494's Test phase · class: an e2e seat whose devices come and go*

The first relaunch in the headless sway panicked at start in gpui's Wayland keyboard code
(`keymap_state.as_mut().unwrap()` on a `Modifiers` event). Its first keymap was `NoKeymap`: the
seat's newest virtual keyboard was a step's `wtype`, gone by then, and the keyboard holder the
run started with was older. Fixed in the harness: `hold_keyboard` starts a new holder before
each launch, so the seat's live keyboard is the holder's when Marley binds it. gpui's unwrap
is Zed's and stays, since a real compositor always sends a keymap.

## F-claude-495-the-list-opened-for-an-agents-click-in-a-focused-tab-001
*severity: low · found in: pipeline 495's Test phase (checking D4 against the run) · class: an agent's action read as the user's*

The tab first opened a select's list when the user had pressed in the page within a second or
when the page had the focus, the second for keys. An agent's `browser_click` on a select while
the user's focus sat in that page would then have opened a list the user never asked for, and
moved the focus into it. Fixed before the commit: the tab's own key path stamps the same moment
a press does, and the list opens only within a second of that stamp; the scenario's last step,
an agent's click with the page focused, opens nothing.

## F-claude-496-ctrl-shift-c-in-a-tab-field-opened-the-collab-panel-001
*severity: low · found in: pipeline 496's Test phase (the first run's shots) · class: a key bound in a view's context lost to a negated predicate · prevented by: PR-claude-a-views-key-needs-its-fields-context-too-001*

`marley::PickElement` was bound to Ctrl-Shift-C in `MarleyBrowser`. Pressed from a pick's
caption field, the key opened Zed's collab panel instead: Zed binds it in `!Terminal`, a
predicate that holds at every depth and so matches at the deepest one, the field's `Editor`,
which outranks a binding in the field's ancestor `MarleyBrowser`. From the page itself both sit
at the same depth and Marley's keymap, bound later, won, so the first press in the run worked
and the second did not. Fixed before the commit: `MarleyBrowser > Editor` binds the key too.

## F-claude-496-escape-never-reached-a-browser-page-001
*severity: low · found in: pipeline 496's Code phase (the review, before Escape could end pick mode) · class: a key bound above a view that forwards raw keys · prevented by: PR-claude-a-views-key-needs-its-fields-context-too-001*

Since #489 the Browser tab sent each key to the page from its `on_key_down`, after Zed's
bindings. Zed binds Escape to `workspace::Unfollow` in `Workspace`, whose handler never
propagates, so Escape never reached a page: a page's own dialog or menu could not be closed
with it, and pick mode could not end on it. Fixed before the commit: `MarleyBrowser` binds
Escape to `null`, which drops the `Workspace` binding while the tab has the focus; the address
bar's, the dialog's and a select list's Escape bindings sit deeper and still win. Other keys Zed
binds above the tab (Ctrl-S, Ctrl-W) still stay with Zed.

## F-claude-496-the-picks-text-read-a-fields-value-001
*severity: medium · found in: pipeline 496's Code phase (the review against the security line) · class: what the user typed reaching an agent · prevented by: the #492 rule that a snapshot writes no values*

`DESCRIBE`, the function that reads a picked element, took its text from `innerText` and, for
an element with none, from `value`. For a picked password field that is the password, which the
bundle would have handed to the agent through `browser_pick`, and for any field what the user
typed. Fixed before the commit: the text is `innerText`, and a value only for an `<input>`
button, where it is the label; the accessible name, which a field's label gives, still names a
field.

## F-claude-498-a-full-snapshot-gave-no-heading-a-ref-001
*severity: low · found in: pipeline 498's Test phase (the stand-in agent's first annotate) · class: a tool's handle covered less than the tools that take it*

`browser_snapshot` gave refs to interactive nodes only, with `full` as without, so an agent
reading a full snapshot saw "Third heading" and had no ref to name it by: `browser_annotate`,
which marks any element, could only reach buttons, links and fields. Fixed before the commit:
with `full`, every node the tree writes a line for gets a ref, but text and the document. The
default snapshot is unchanged, so #492's refs and their token cost stay as they were.

## F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001
*severity: high · found in: pipeline 502's Test phase (the first start through the menu's path, in the headless sway) · class: an unwrap on state another event sets · prevented by: #512*

gpui's Wayland client unwraps its xkb state in the `wl_keyboard` `modifiers` and `key` arms
(`crates/gpui_linux/src/linux/wayland/client.rs:1921`, `:1926`, `:1958`), and the `keymap` arm
sets that state only for an `xkb_v1` keymap that compiles (and `expect`s the compile). A
compositor may send `modifiers` with no usable keymap first: wlroots does when the seat has no
keyboard. In #502's first run the scenario's last `wtype` had exited, the Marley that
`uwsm-app -- gtk-launch` started got no keymap, and it panicked on its first keyboard event. On
the `dev` channel the panic went to stderr only (L-claude-502-a-dev-channel-panic-reaches-stderr-only-001).
Upstream Zed has the same code on 2026-09-25. Not fixed in #502: its scenario gives the seat a
keyboard first, as a desktop has one, and #512 makes the handler skip keyboard events until a
keymap arrives.

## F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001
*severity: medium · found in: pipeline 515's Test phase (the first run) · class: gpui dispatch during an update · prevented by: L-claude-515-dispatch-through-the-window-from-inside-an-action-001*

`marley: open settings` was a global `cx.on_action` handler that called
`cx.dispatch_action(&zed_actions::OpenSettingsPage { .. })`. Run from the command palette, it
opened nothing and the log said `window not found` (`crates/gpui/src/app.rs:2572`): the palette
dispatches its chosen action while the window is being updated, and an app-level dispatch looks
the window up in the app's map, where it is not until the update ends. Fixed before the commit:
the action is registered on the workspace (`workspace.register_action`) and calls
`window.dispatch_action`, as `agent_panel.rs` opens its settings page.

## F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001
*severity: high · found in: pipeline 516's Code review · class: redaction order · prevented by: PR-claude-redact-the-whole-text-before-cutting-it-001*

`terminal_read` cut a block's output to its last 2,000 lines and 256 KiB (`tail`) and then ran
the redactor over what was left. A private key whose `-----BEGIN` line fell before the cut would
reach the agent as its body and its END line: the private-key rule needs the BEGIN line to match,
and no other rule takes base64 lines. The same holds for any rule that anchors on a prefix a cut
can take away (`Bearer `, a URL's `scheme://user:`, a variable's name). Found by reading the diff
against the rules before Test, so it never shipped. Fixed: the whole output is redacted first,
then cut; `redacted` counts over the whole output, and the schema says so.

## F-claude-516-a-rewrap-moves-every-blocks-rows-001
*severity: high · found in: pipeline 516's Test phase (the first run) · class: block anchors across a resize · prevented by: TICKET-544*

A block is a range of absolute lines (`output_start`, `output_end`). In #516's first run the
Settings window opened tiled beside the main window, the terminal narrowed from about 120
columns to 30, and alacritty rewrapped its lines: the prompt line became two rows and every long
output line two or three. The anchors stayed where they were, so the block's bar and pill
vanished and `terminal_read` answered rows that began inside the command line (`ake sh
secrets.sh`) and stopped mid-line. Every block before a rewrapping resize is wrong from then
on, in the view and for agents. The plan's D2 calls reflow "the known weak spot" and deferred the
choice until a real resize trace; this is one. Not fixed in #516, whose scenario reads the
terminal before any window opens; TICKET-544 takes it.

## F-claude-513-a-socket-path-too-long-read-as-a-running-marley-001
*severity: high · found in: pipeline 513's Test phase (the first run on the fixed build) · class: Unix socket path length · prevented by: PR-claude-check-a-unix-socket-path-against-sun-path-001*

With the single-instance check running on the dev channel, the scenario's Marley never opened a
window. Its profile sat under the session's long shots folder, so `<profile>/zed-dev.sock` was
134 bytes, over the 108 a Unix socket's address holds. Zed's `listen_for_cli_connections` failed
to bind, the check reads any failure as "a Marley runs", the hand-off could not connect ("path must
be shorter than SUN_LEN"), and Marley exited: any `--user-data-dir` deep enough would stop Marley
from starting at all. Fixed before the commit: `single_instance::socket_fits()` gates the check,
and a data directory whose socket does not fit starts without it and logs why; the e2e runner
makes its profiles under `$XDG_RUNTIME_DIR/marley-e2e/`.

## F-claude-544-the-resize-arms-columns-changed-is-always-false-001
*severity: high · found in: pipeline 544's Test phase (the first run on the fixed build) · class: a stale-comparison guard · prevented by: PR-claude-compare-against-the-state-the-change-acts-on-001*

The first fix gated the anchors' rewrap on the Resize arm's own `columns_changed`, which compares
`last_content.terminal_bounds` with the event's bounds. `Terminal::set_size` stores the new bounds
in `last_content.terminal_bounds` before it queues the event, so by the time the arm runs the two
are equal and the test is always false: the rewrap never ran, and the scenario failed exactly as
on the unfixed build (upstream's `reset_cwd_history` on a width change never runs either). Found
with a temporary log line that never printed. Fixed before the commit: the arm reads the grid's
rows before and after `resize` and rewraps when the grid's own width changed.

## F-claude-544-a-rewrap-cut-history-rows-without-counting-them-001
*severity: medium · found in: pipeline 544's Plan (the anchors' sweep) · class: the eviction counter · prevented by: PR-claude-compare-against-the-state-the-change-acts-on-001*

The vendored alacritty's `shrink_columns` rewraps into more rows and then `truncate`s the storage
to the history limit, cutting the oldest rows without adding them to `evicted_lines`, the counter
#462 added so absolute lines stay true. With a full history, every narrowing shifted each absolute
line by the rows it cut: a block's output read the wrong rows, and `block_output_kept` misjudged
what had left the scrollback. Fixed in #544: the cut rows are counted, recorded in
`vendor/README.md`.

## F-claude-546-block-reads-answered-from-the-alternate-screen-001
*severity: medium · found in: pipeline 544's Plan (the anchors' sweep) · class: two screens, one reader · prevented by: PR-claude-compare-against-the-state-the-change-acts-on-001*

`absolute_lines_text` (behind `terminal_read`) and `block_output_kept` (behind `terminal_blocks`)
read `term.grid()`, the active grid. While a full-screen program holds the alternate screen, that
is the alternate grid, whose own evicted count climbs as the program scrolls: an agent asking for
an earlier block while the user sat in `less` was told its output had left the scrollback (#546's
unfixed run), or could get the program's rows. Fixed in #546: both read the main screen
(`Term::main_grid`, `Term::main_bounds_to_string`, vendored).

## F-claude-519-a-frame-bound-dropped-the-shown-fields-before-the-unbounded-ones-001
*severity: low · found in: pipeline 519's Test (the hook's size check) · class: a bound that drops the wrong field first · prevented by: PR-claude-drop-the-unbounded-fields-first-001*

`event.py` keeps its summary under 2,900 bytes by dropping fields while it is over: message, then
preview, then prompt, then the transcript path and the working directory. The fields the row
shows are cut to 300 and 200 characters already; the paths have no bound. With 5,000-byte paths
the loop dropped the prompt and the preview before reaching the paths, so the frame carried the
event and nothing the row shows, and the scenario's first size check passed on it because it
checked only the size and the event's name. Fixed in #519 before the commit: the paths go first,
and the check requires each row's fields in the decoded summary.

## F-claude-547-a-scenarios-click-ran-the-real-claude-001
*severity: high · found in: pipeline 547's Test (the first run) · class: a fake the app never found · prevented by: PR-claude-name-the-fakes-the-app-runs-001*

The scenario put its stand-in `claude` first on the PATH it exported and clicked the agent bar's
update chip. Marley found `claude` with `which`, on the PATH the app sees, which the login shell's
profile had led with `~/.local/bin`: the real Claude Code ran `claude plugin marketplace update
marley`. Only the scratch `CLAUDE_CONFIG_DIR` kept it off Chad's Claude Code configuration (it
refused the scratch file's shape and changed nothing). Fixed in #547: `MARLEY_CLAUDE` names the
program, as `MARLEY_CHROMIUM` does, and the scenario sets it.

## F-claude-547-a-timer-armed-at-the-first-event-fired-before-the-last-was-a-minute-old-001
*severity: low · found in: pipeline 547's Test · class: a timer measured from the wrong moment · prevented by: PR-claude-compare-against-the-state-the-change-acts-on-001*

The rail armed a 60-second refresh at the first frame of a turn (the prompt), but the seat's
quiet time runs from its last event (the tool call, milliseconds later). At the refresh the seat
had been quiet just under a minute, so the row stayed `working` and the timer waited another
minute. Fixed in #547: the delay is computed from each working seat's last event to the moment
its row next changes (`AgentEvents::next_quiet_change`).

## F-claude-520-a-key-removed-from-the-builders-map-still-reached-the-program-001
*severity: medium · found in: pipeline 520's Test (the task's shot) · class: an environment the child inherits · prevented by: PR-claude-empty-a-variable-the-child-must-not-inherit-001*

`TerminalBuilder::new` removed `MARLEY_TERMINAL_ID` and `MARLEY_PROJECT` from its `env` map for a
task terminal, and the task still printed Marley's inherited values: the PTY's program gets
Marley's own process environment with the map laid over it, so a key missing from the map is
inherited, not unset. The interactive terminals looked right only because their values were set.
Fixed in #520: the hunks set the variables empty where they name nothing, and the readers treat
empty as none.

## F-claude-574-a-placement-outlived-its-page-001
*severity: low · found in: pipeline 574's Code phase (the review of the diff) · class: state kept for an event that may never come*

The first draft kept a new page's placement, the workspace its tab goes to, until `place_tab`
took it, so a page whose attach failed, or one made just before a restart, left its placement in
the hub for the rest of the run. Fixed before Test: the placement goes with a failed attach, at a
start and when its page goes.

## F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001
*severity: medium · found in: pipeline 575's Test (the right terminal's id after the first relaunch) · class: a cleanup whose caller lists part of the items · prevented by: PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001*

The first design read a restored terminal's saved id from Marley's table in `deserialize`. The
terminal panel restores its own terminals and then runs `TerminalView::cleanup` with the
panel's items alone (none in the Marley layout), which deletes every terminal row of the
workspace; at the relaunch it ran between the two center terminals' restores, and the second
one found no row and got a new id. Found by logging the hook's calls in the order they came.
Fixed in #575: the ids the table held at the start, and each id saved since, are kept in memory
and the restore reads there. Zed's own `terminals` rows meet the same delete (TICKET-577).

## F-claude-577-a-restored-center-terminal-opened-in-the-projects-folder-001
*severity: medium · found in: pipeline 575's Test, confirmed red in pipeline 577's on the build before · class: a cleanup whose caller lists part of the items · prevented by: PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001*

Zed's terminal panel cleans up the `terminals` rows after its own restore with the panel's items
alone, whether or not it restored any, so in the Marley layout it deleted the center terminals'
rows, and a center terminal whose restore read its row after that came back in the project's
folder. #577's scenario showed it on the build before the fix: the split's terminal, left in
`beta`, came back in the repository's root. Fixed in #577: the panel's cleanup also keeps the
terminal items of the workspace's saved layout (`TerminalDb::marley_saved_terminal_items`).
