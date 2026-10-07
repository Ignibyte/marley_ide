# Distilled lessons — the local ledger

> Exported 2026-08-09 from the forge DB (table `distilled_lessons`; 201 entries,
> TICKET-409 — the scrap-forge pivot). This file is the LIVE capture surface:
> `/pipeline:design` and `/pipeline:complete` APPEND new `## <code>` blocks;
> recall is `grep`. One entry per `## <code>` heading — never edit history.

## DL-cluster-010ec80947d9
*topic: The shipped clamp_card_text (#377 F2) filtered only c.is_control() — Unicode general-category Cc — so bidi FORMAT characters (Cf: U+202A–202E embeds/overrides, U+2066–2069 isolates) passed through ... · confidence: 0.500*

The shipped clamp_card_text (#377 F2) filtered only c.is_control() — Unicode general-category Cc — so bidi FORMAT characters (Cf: U+202A–202E embeds/overrides, U+2066–2069 isolates) passed through to gpui rendering. Core Text honors bidi controls, so hostile text (server-controlled #377 answer / #378 dispatch failure reasons, or a hostile settings value reaching the #384 misconfiguration caption) could visually REORDER what renders — a Trojan-source display spoof where the displayed string reads differently from the stored one (e.g. a rejected URL displaying as a different URL). Found by the #384 inspect security critic while tracing the new reason path through the shared helper; fixed at the helper (one filter widening hardens all four consumers) + pinned by a unit feeding embeds/overrides/isolates and asserting the stripped output.

## DL-cluster-029a16f28dd4
*topic: At #386 implement, `cargo check -p marley` reported clean but the crate did NOT compile under `--tests`: widening `rail_rows` by one param + adding the `AppliedSettings.collapsed_sections` field br... · confidence: 0.500*

At #386 implement, `cargo check -p marley` reported clean but the crate did NOT compile under `--tests`: widening `rail_rows` by one param + adding the `AppliedSettings.collapsed_sections` field broke 14 EXISTING test sites (11 `rail_rows` callers missing arg 3 + 3 `AppliedSettings` struct literals missing the field) that plain `cargo check` never compiles (`#[cfg(test)]` is skipped). Implement was declared Phase-3 PASS on that false-clean signal; the inspect critics caught it by running `cargo check --tests`. No app bug — a verification blind spot where a public-API change fans out to test callers invisibly.

## DL-cluster-08e174cf2943
*topic: #183 RECURRENCE of BF-mutants-skip-detached-by-fn-insertion (from #184): I inserted a new fn `path_command_names` immediately above `complete_at_prompt`, placing it between complete_at_prompt's doc... · confidence: 0.500*

#183 RECURRENCE of BF-mutants-skip-detached-by-fn-insertion (from #184): I inserted a new fn `path_command_names` immediately above `complete_at_prompt`, placing it between complete_at_prompt's doc comment + `#[cfg_attr(test, mutants::skip)]` and the fn — so the attribute rebound to the new fn, silently un-skipping complete_at_prompt (a coverage-excluded untestable shim) → 9 unkillable mutants → MSI would go RED. Invisible to cargo check + cargo fmt. Caught by both inspect critics via `cargo mutants --list`. The prevention rule PR-claude-verify-skip-attr-still-attached-after-inserting-fn ALREADY EXISTED from #184 — I failed to apply it proactively. Lesson reinforced: after inserting ANY fn adjacent to a skipped shim, immediately grep -B1 the skip or run cargo mutants --list to confirm the previously-skipped fn is still absent. Fixed by reordering.

## DL-cluster-097e7e4acb7b
*topic: #308 marley_lsp: a relative PATH entry (`.` or any relative dir) made `search_path` return a RELATIVE command string; `spawn_server` then ran it with `current_dir(workspace_root)`, so a program str... · confidence: 0.500*

#308 marley_lsp: a relative PATH entry (`.` or any relative dir) made `search_path` return a RELATIVE command string; `spawn_server` then ran it with `current_dir(workspace_root)`, so a program string containing a separator is resolved by the OS against the child's cwd — executing a `rust-analyzer` a hostile cloned repo happens to ship (local code execution on opening a hostile repo + a `.rs` file). Empirically confirmed by the security critic (`Command::new("./x").current_dir(child)` runs child/x). Root cause: PATH search returned the candidate verbatim without requiring an absolute directory; the empty-element filter defused only the classic empty==cwd case, not a literal `.`/relative dir. Fixed: `search_path` skips non-absolute PATH dirs → always returns an absolute command, immune to cwd resolution.

## DL-cluster-0acca64fd98a
*topic: #328 git gutter: the derived git-marks cache is keyed on (path, git_marks_gen), and git_marks_gen was bumped only on save + external-file reload — but NOT after the app's OWN git-panel commit (git_... · confidence: 0.500*

#328 git gutter: the derived git-marks cache is keyed on (path, git_marks_gen), and git_marks_gen was bumped only on save + external-file reload — but NOT after the app's OWN git-panel commit (git_commit). So committing FROM Marley's own commit panel moved HEAD (the committed lines are no longer "changed vs HEAD" → the bars should clear) yet the cache key was unchanged → refresh_git_marks returned early → the pre-commit Added/Modified gutter bars lingered until the next save/reload/file-switch. More surprising than the external-terminal-commit case because the mutation originated INSIDE the app. Caught by inspect critic 2. Fixed: bump git_marks_gen on a successful git_commit.

## DL-cluster-0b388500d521
*topic: #331 inlay hints. Injecting phantom text made ONE column map serve two incompatible boundary semantics. `col_of_offset(i)` is the CARET map: it deliberately returns the column AFTER a phantom ancho... · confidence: 0.500*

#331 inlay hints. Injecting phantom text made ONE column map serve two incompatible boundary semantics. `col_of_offset(i)` is the CARET map: it deliberately returns the column AFTER a phantom anchored at i (the caret sits on the code side of a hint). `raw_span_to_display_bytes` and the #310 diagnostic bar both built their END boundary from it — so a code span whose one-past-end char is a hint anchor EXTENDED ACROSS the phantom. `styled_slices_with_marks` resolves a slice to the FIRST containing range and hints were appended LAST, so the code span won and the hint painted as code. Reachable on the PRIMARY tree-sitter path via rust-analyzer chaining hints (default ON, always EOL-anchored, and a literal DOES have a syntax span ending exactly there): `let n = "hello"` → Str 8..15 mapped to display 8..20 → the hint rendered string-green. Same root cause on the squiggle: a warning on `x` in `let x = 1;` underlined `x: i32`. Found INDEPENDENTLY by two critics from opposite ends (one from the column math, one from reachability). Fix: a separate END-side map (`col_ends` + `col_of_span_end`) for the spans that describe CODE; the selection band deliberately KEEPS the caret map, because it must end where the caret it follows renders.

## DL-cluster-0c3cad411a14
*topic: #325 opening the ⌘T modal picker while an editor completion popup was live left completion_menu open — its key arm runs BEFORE the picker's in the router, so ↑/↓/Enter kept driving the COMPLETION l... · confidence: 0.500*

#325 opening the ⌘T modal picker while an editor completion popup was live left completion_menu open — its key arm runs BEFORE the picker's in the router, so ↑/↓/Enter kept driving the COMPLETION list and Enter routed to accept_completion_item, inserting a candidate into the buffer BEHIND the picker (the unguarded Buffer edit path). ⌘T falls through handle_completion_key (which only consumes Esc), so the popup never self-dismissed. Fix: open_symbol_finder dismisses completion_menu + the terminal completion first, so the modal truly owns the keyboard. The class: a modal that owns the keyboard must dismiss any lower overlay whose key arm precedes it (the sibling of #324's stale-card-over-palette).

## DL-cluster-0cdcee897eb5
*topic: TICKET-024: region_widths computed center = window − docks UNCLAMPED, and SPEC R31 spec'd exactly that — below 440pt window width (reachable by ordinary resize; the gpui window sets no minimum) the... · confidence: 0.500*

TICKET-024: region_widths computed center = window − docks UNCLAMPED, and SPEC R31 spec'd exactly that — below 440pt window width (reachable by ordinary resize; the gpui window sets no minimum) the center went negative, the right dock's x = left+center regressed INSIDE the left dock (overlap), and pane_rects with negative width moved child origins leftward (reversed tiling). No panic (taffy clamps, gpui culls empty bounds) — silent wrong-geometry, the worst kind. Caught by an inspect critic probing the pure math + reading taffy/gpui negative-size behavior. Fix: clamp the remainder at 0.0 with the shrunken-window case as a REACHABLE tested arm (not dead defense: the boundary test kills clamp-deletion and max→min mutants), and co-amend the spec — the spec itself mandated the bug, so the fix was spec+code together.

## DL-cluster-0dddecc70de4
*topic: Marley #296 (multi-cursor core). Buffer::set_selection defensively CLAMPED each Selection to the rope length and then rebuilt the SelectionSet with the raw in-crate `from_members` — which does NOT ... · confidence: 0.500*

Marley #296 (multi-cursor core). Buffer::set_selection defensively CLAMPED each Selection to the rope length and then rebuilt the SelectionSet with the raw in-crate `from_members` — which does NOT re-merge. Clamping is NOT injective: two distinct out-of-bounds cursors (10 and 20 in a 3-char rope) both clamp onto offset 3 and SURVIVED AS DUPLICATES, violating the type's ORDERED+DISJOINT invariant. edit_at_selections then applied the edit at both duplicate members: buffer "abc" + carets {10,20} + "X" produced "abcXX" — one visible cursor, two inserts. A GREEN gate (100% line coverage + MSI 100) and a 50k-step differential fuzzer both missed it, because every test fed sets that had already been canonicalized by from_selections. Found by an adversarial inspect critic. FIX: set_selection re-canonicalizes after clamping — SelectionSet::from_selections(clamped) — plus a regression test asserting the duplicates collapse and the replacement is typed ONCE.

## DL-cluster-0ede4c4ba511
*topic: #387 dispatch_section_action's OpenFile arm switched the active project (to act on the ＋'s row-project) but did NOT persist_grid the switch, unlike the #174 Tab/Pane rail rows and the sibling NewTe... · confidence: 0.500*

#387 dispatch_section_action's OpenFile arm switched the active project (to act on the ＋'s row-project) but did NOT persist_grid the switch, unlike the #174 Tab/Pane rail rows and the sibling NewTerminal/OpenCockpit arms. active_project IS persisted state (serialize_shell writes it first, grid_layout.rs:330; restored at boot app.rs:2139), and the finder's own persist fires only on ⌘↵-open — so clicking Editor＋ on a non-active project then cancelling the finder then relaunching reverted the active project (self-healing on any later persist, but a real divergence). Caught by inspect (my trace + critic A). Fix: added `if project_changed { self.persist_grid(); }` to the OpenFile arm. Class: a rail affordance that switches the active project must persist the switch even when its action opens a transient modal rather than mutating durable state.

## DL-cluster-0ee6969f0d72
*topic: #229 added a full-screen dismiss backdrop behind the #181 agent launcher. The launcher-open guard (app.rs:3158-3161) closed palette/finder/history/find but NOT the #204 naming_workflow save-as draf... · confidence: 0.500*

#229 added a full-screen dismiss backdrop behind the #181 agent launcher. The launcher-open guard (app.rs:3158-3161) closed palette/finder/history/find but NOT the #204 naming_workflow save-as draft, which renders BEFORE the launcher. Via the 🧠 top-bar icon (which bypasses the key router's overlay guards), the launcher can open over an active naming draft; pre-#229 the naming box peeked around the launcher box, but #229's full-screen `.inset_0()` backdrop now FULLY HIDES the naming draft while its keyboard guard (:3512) still owns input → an invisible-focus state. Caught by the inspect critic (LOW), fixed by adding `self.naming_workflow = None;` to the guard. Lesson: a full-screen dismiss backdrop for modal A hides (not just occludes) any overlay that renders before A, so A's open path must close every such overlay. Rule: PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays-001.

## DL-cluster-0eed5cbf774c
*topic: #300 move/duplicate lines: the carried-selection math (`move_lines`/`duplicate_lines` in line_move.rs) attributed each selection endpoint to its moving block by the endpoint's raw row (`buffer.line... · confidence: 0.500*

#300 move/duplicate lines: the carried-selection math (`move_lines`/`duplicate_lines` in line_move.rs) attributed each selection endpoint to its moving block by the endpoint's raw row (`buffer.line_col(off).0`). But the universal "N full lines selected" shape (Home → ⇧↓×N) is `anchor in the block, head at col 0 of the row PAST the block`, and `indent::line_span` deliberately CARVES that col-0 row out of the block (indent.rs:45). So the boundary head sits on row b1+1, EXCLUDED from the block → it got delta 0 while the anchor got the real move delta → the carried selection distorted (shrank on move-down, grew on move-up, doubled to span both copies on duplicate-up). The TEXT edits were correct in every case; only the carried SELECTION was wrong — so no text/round-trip test catches it, only a selection-shape assertion on a full-line selection. Reachable via the standard shift-arrow line selection; it COMPOUNDS (a shrunk selection fragments the next move). Caught by an inspect critic. Fixed by attributing the endpoint by OFFSET SPAN (`[line_start(b0), line_start(b1+1)]`), inclusive of the carved boundary for MOVE + DUP-UP but exclusive for DUP-DOWN (where rebase_through already carries the boundary onto the copy). Sibling low finding L2: the phantom trailing row was movable up (no last_content_row clamp on the up-guard), dropping the file's trailing newline — fixed with a touched_content_rows clamp.

## DL-cluster-0f054a0ff249
*topic: #237's pure EditorSurface::close had a 3-branch active-index clamp (active > idx → -1; active == idx == last → len-1; else unchanged) + activate's `idx < len` guard. The 3 unit tests exercised the ... · confidence: 0.500*

#237's pure EditorSurface::close had a 3-branch active-index clamp (active > idx → -1; active == idx == last → len-1; else unchanged) + activate's `idx < len` guard. The 3 unit tests exercised the code but left 4 mutants alive (MSI ≈ 82.6%, not the required 100): close's `>`→`==`/`<`/`>=` (the test only ever closed at the LAST index, where the decrement branch and the >=-clamp branch COINCIDE — same value → operator swaps undetected) + activate's `<`→`<=` (the test used activate(5) on a 2-file surface, but `5<2`==`5<=2`==false → the boundary idx==len was never hit). Behaviorally correct code, but the strict mutation gate on the pure seam was RED. An inspect critic ran cargo mutants + gave the discriminating cases. Fixed: added a 4-file `active > idx` case, an active-MIDDLE case (active==idx, not last), and an `activate(len)` exact-boundary case → 0 missed, MSI 100.</description>

## DL-cluster-11ced1aa857a
*topic: #330 validate: gate:5 had exactly ONE survivor — `settings.rs persist_sticky_header -> Ok(())`. A `persist_X(manager, v) -> Result<(), SettingsError>` setter's body-replacement mutant returns Ok(()... · confidence: 0.500*

#330 validate: gate:5 had exactly ONE survivor — `settings.rs persist_sticky_header -> Ok(())`. A `persist_X(manager, v) -> Result<(), SettingsError>` setter's body-replacement mutant returns Ok(()) WITHOUT writing, and nothing detected it: the new `editor.sticky_header` setting had a field/applied_from/applied_defaults wiring (all covered by the existing AppliedSettings tests) but NO test proving the SETTER actually persists. Every other `persist_*` in the crate is covered by the shared `settings_round_trip_survives_reload` test; the new one simply wasn't added to it. Fixed by adding the sticky-header leg to that round-trip — persisting the NON-DEFAULT value (false; the setting defaults true) so a no-op write reloads as the default and fails the assert. MSI 18/18 → 100.

## DL-cluster-125897ba4424
*topic: Clicking a file in the file browser silently did nothing (chad live-app feedback #3). Root cause: FileTree::path_at / list_files_in yield paths RELATIVE to the project root, but open_file_in_viewer... · confidence: 0.500*

Clicking a file in the file browser silently did nothing (chad live-app feedback #3). Root cause: FileTree::path_at / list_files_in yield paths RELATIVE to the project root, but open_file_in_viewer read them with `std::fs::read(&path)`, which resolves relative paths against the PROCESS CWD. A bundled app launched via LaunchServices (`open Marley.app`) runs with CWD `/`, so `read("crates/.../buffer.rs")` → `/crates/...` → Err(_), which was silently swallowed by an `Err(_) => {}` arm. The file never opened and there was no error surfaced. It only appeared to work in a dev instance whose project root happened to be `/` (relative==absolute). Fix: pure `resolve_under_root(root, path)` (join relative onto root, pass absolute through) routed through open_file_in_viewer, so reads are CWD-independent. Class lesson: NEVER `std::fs::read` a project-relative path directly in a GUI app — always resolve against the known project root; the CWD is not the project root for a bundled/LaunchServices launch. And an `Err(_) => {}` swallow hid the failure — prefer surfacing a status flash on read error.

## DL-cluster-1335cd84bc32
*topic: #340 bracket-match: `refresh_bracket_match` (app.rs) had NO language gate, so it called `marley_syntax::matching_delimiters_in` — which ALWAYS parses `tree_sitter_rust::LANGUAGE` — for ANY editor b... · confidence: 0.500*

#340 bracket-match: `refresh_bracket_match` (app.rs) had NO language gate, so it called `marley_syntax::matching_delimiters_in` — which ALWAYS parses `tree_sitter_rust::LANGUAGE` — for ANY editor buffer. A `.json`/`.toml`/`.py` file would show a bracket tint + ⌘⇧\ jump driven by RUST grammar semantics (a `(` in a Python `#` comment false-matches because Rust has no `#` comment). Violated the spec's "Rust-only v1" Scope and REQ-008 ("render nothing / do no walk when no tree exists"), and made the shim's own doc comments ("a non-Rust file yields None") false. Root cause: the Phase-2 design premise "non-Rust files fall out for free" was a CLAIM, not a fact — it assumed no tree for non-Rust, but the pure fn is language-specific by construction and always builds a Rust tree. Caught by an inspect critic parsing JSON as Rust → `Some((0..1,12..13))`. Fixed by mirroring `refresh_syntax_cache`'s `is_rust` gate at the caller (the seam that knows the file's language); drop the cache like the no-editor branch when `!is_rust`.

## DL-cluster-156361bb3264
*topic: #328 validate: a NEW test-setup helper `run_git` asserted `assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr))`. The `String::from_utf8_lossy(&stderr)` ... · confidence: 0.500*

#328 validate: a NEW test-setup helper `run_git` asserted `assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr))`. The `String::from_utf8_lossy(&stderr)` in the message is evaluated ONLY on the panic (false) arm — and git always succeeds in the seeded tempdir, so that function-call expression is an UNCOVERED cold region. gate:4 (whole-workspace line coverage 100%, headless_drive.rs is NOT excluded) went RED. A plain `assert!(cond)` or a LITERAL-message assert has no such cold region (llvm-cov attributes the line to the executed condition check); only a runtime CALL in the message body creates a separately-counted uncovered region. Fixed → a literal message. (Also the same helper tripped gate:2 by using the disallowed std::process::Command — separate BF.)

## DL-cluster-15b3caa3c986
*topic: M29 #402 inspect F2 (critic-found, pre-ship). The wry IPC handler closure captured a strong Rc<Shared> while wry STORES that closure inside the WebView it builds (wry-0.56.0 wkwebview/mod.rs:144 ip... · confidence: 0.500*

M29 #402 inspect F2 (critic-found, pre-ship). The wry IPC handler closure captured a strong Rc<Shared> while wry STORES that closure inside the WebView it builds (wry-0.56.0 wkwebview/mod.rs:144 ipc_handler_delegate, retained :553, released only in WebView Drop :1423) and Shared owns the WebView — ownership cycle Shared→WebView→delegate→closure→Rc<Shared>, so the WebView's Rust Drop could never run. Consequence: the probe's R6 teardown observation (a #406 design input) would have silently reported OS process-exit reclamation as clean drop behavior — corrupted evidence, not just a leak. Fix: capture Rc::downgrade(&shared); handler upgrades or returns. The same shape awaits every production wry callback (#405 pane wiring, #406 lifecycle): any callback handed to a resource that the callback's captured state OWNS must capture weakly.

## DL-cluster-16d62dfefc1f
*topic: #279 inspect C1: the new grid mouse-down called content_row_texts (which walks EVERY block and allocates a String per row of the UNBOUNDED scrollback) on every left click — including single and shi... · confidence: 0.500*

#279 inspect C1: the new grid mouse-down called content_row_texts (which walks EVERY block and allocates a String per row of the UNBOUNDED scrollback) on every left click — including single and shift clicks whose decision arms never read the row text; the pre-#279 seed was zero-alloc. The #274 class ("the hot interactive path pays for the whole history") reintroduced for clicks one ticket later. Fix: the fetch gates on !shift && count>=2. CLASS: when threading a document-derived input into a hot input handler, gate the derivation on the exact arms that consume it — the decision fn taking a param does not mean every caller path must compute it.

## DL-cluster-1ad37ff64beb
*topic: #262 extended gate:14 (brand-scrub check in docs_g) but left the canonical gate table row in docs/specs/standards/quality-bar.spec.md stale — gates.sh declares that spec "the single source of truth... · confidence: 0.500*

#262 extended gate:14 (brand-scrub check in docs_g) but left the canonical gate table row in docs/specs/standards/quality-bar.spec.md stale — gates.sh declares that spec "the single source of truth" for the gate bar, so the row and the implementation diverged mid-pipeline. Caught by the inspect critic (MED); fixed before validate. Class: any change to a gate's behavior must update the quality-bar row in the same change.

## DL-cluster-1bac57ca2300
*topic: When porting the test gate from `cargo test` to `cargo nextest run`, the invocation in gates.sh and the BLOCKED-message hint in enforce-tests-ran.sh were updated, but the hook's DETECTION regex (li... · confidence: 0.500*

When porting the test gate from `cargo test` to `cargo nextest run`, the invocation in gates.sh and the BLOCKED-message hint in enforce-tests-ran.sh were updated, but the hook's DETECTION regex (line 29: `cargo (test|llvm-cov)|scripts/gates.sh`) was not — `cargo nextest` does not match `cargo (test|llvm-cov)`. A validator running only the now-prescribed `cargo nextest run` would be spuriously Stop-blocked ("tests never ran") despite running tests. Caught by an inspect critic; never shipped. Fixed: detection regex → `cargo (test|nextest|llvm-cov)`.

## DL-cluster-1bee7518b635
*topic: TICKET-397 inspect HIGH (found independently by two critics): the selection park/restore tracker overwrote last_editor_view with None whenever a non-editor surface (terminal pane, cockpit) took foc... · confidence: 0.500*

TICKET-397 inspect HIGH (found independently by two critics): the selection park/restore tracker overwrote last_editor_view with None whenever a non-editor surface (terminal pane, cockpit) took focus, so the everyday tour tab-view → terminal → split-twin never parked the outgoing cursors: the twin edited with the sibling's cursor set, the twin's own parked set was stranded, and the eventual direct twin→twin switch restored None → the caret-0 seed destroyed a real position (caret teleport to 1:1). Stale IME marked spans leaked through the same hop. Fixed: the tracker is STICKY across None frames; the park runs on EVERY view change reading the outgoing set from the OUTGOING view's instance; rows are birth-seeded with a caret-0 park so the focus-in restore is total; the restore clears the incoming row's stale composition.

## DL-cluster-1bfb771ecb49
*topic: The #352 implement phase put fold_projection() — whose parse arm is buf.text() (O(n) String) + a full tree-sitter parse — onto the 16ms pump (refresh_inlay_hints ticks unconditionally) and onto eve... · confidence: 0.500*

The #352 implement phase put fold_projection() — whose parse arm is buf.text() (O(n) String) + a full tree-sitter parse — onto the 16ms pump (refresh_inlay_hints ticks unconditionally) and onto every mouse-move. With any fold active on a Rust file that produced ~60 full-file parses/second while completely idle; pre-diff both paths were integer compares. Caught at inspect by both critics (the pump cadence PUMP_INTERVAL_MS=16 verified). Fix: fold_proj_cache memo keyed (nonce, version, anchor set) on the parse arm only — reload re-mints nonce, an edit bumps version, a fold verb changes anchors; identity arms stay cache-free. Lesson: before moving a computation onto a poll/pump/event path, price its worst arm at the path's cadence — render-path precedent (damage-driven frames) does not license pump-path cost (unconditional 60Hz).

## DL-cluster-1bff1ab281b0
*topic: marley_mcp transport (#370): read_http_request allocated `vec![0u8; content_length]` from an UNAUTHENTICATED, uncapped `Content-Length` header, BEFORE the origin/bearer guards run. A hostile loopba... · confidence: 0.500*

marley_mcp transport (#370): read_http_request allocated `vec![0u8; content_length]` from an UNAUTHENTICATED, uncapped `Content-Length` header, BEFORE the origin/bearer guards run. A hostile loopback request with `Content-Length: 999999999999` triggers an OOM → Rust handle_alloc_error process-abort, taking down the whole gpui app + all the user's terminals (availability). Caught by two inspect critics. Fixed: cap MAX_BODY_BYTES=1MiB, reject oversized as 400 before any allocation. Class: a pre-auth path must validate/cap a client-controlled size before allocating.

## DL-cluster-202005781ed6
*topic: #377 inspect F1 (MED, found independently by BOTH critics from different lenses): the answer path carried no question identity at THREE layers — (a) the chip listener captured only (seat_id, choice... · confidence: 0.500*

#377 inspect F1 (MED, found independently by BOTH critics from different lenses): the answer path carried no question identity at THREE layers — (a) the chip listener captured only (seat_id, choice), so a question swapped between paint and click dispatched the old choice against the new question AND self-concealed (the record stored the re-fetched new question, reading current forever); (b) the result channel was tagged by seat id only, so a superseded send's late Ok could overwrite a newer pick's Failed and lock the seat behind a false "answered"; (c) the wire args {session_id, choice} gave the brain nothing to refuse a stale in-flight answer with. Fixed: rendered-Question capture + exact-equality dispatch belt; a per-pick nonce keying the drain apply; prompt added to the proposed wire contract. The general class: an async answer to a mutable question needs the QUESTION'S identity carried at every hop (render→dispatch→channel→wire), not just the subject's id.

## DL-cluster-205b36bfbf3a
*topic: #268 inspect (MED): the syntax memo was keyed (path, BufferVersion) — but BufferVersion is per-Buffer monotonic FROM ZERO, and EditorSurface open/close mints fresh buffers. Reopening a file the ter... · confidence: 0.500*

#268 inspect (MED): the syntax memo was keyed (path, BufferVersion) — but BufferVersion is per-Buffer monotonic FROM ZERO, and EditorSurface open/close mints fresh buffers. Reopening a file the terminal agent rewrote on disk (Marley's core loop) hit (path, v0) == cached (path, v0) → STALE highlight spans rendered until the first keystroke; two projects holding the same path with equal edit counts collided the same way. Probe-verified: spans clamp, no panic — silently wrong colors. Fix: a process-monotonic buffer nonce (AtomicU64 minted in OpenFile::new) in the key — (nonce, version) can never collide across buffer lifetimes.

## DL-cluster-21a843a336c7
*topic: #49 bottom-anchored the terminal render (justify_end on the pane flex_col) but did NOT update pane_grid_pos — the click/pointer→content-cell mapping that is the mathematical INVERSE of the render (... · confidence: 0.500*

#49 bottom-anchored the terminal render (justify_end on the pane flex_col) but did NOT update pane_grid_pos — the click/pointer→content-cell mapping that is the mathematical INVERSE of the render (from #43's selection). pane_grid_pos assumed the first visible row is painted at the pane TOP (row = start + local_y/cell_h). After justify_end, when content < capacity the visible rows paint against the BOTTOM with (capacity - visible) empty rows ABOVE them, so every click/drag lands top_pad rows too low → drag-select highlights nothing and cmd-C copies the wrong/empty text — in the exact short-content scenario the ticket targets. Non-crashing (row_selection/selected_text are range-clamped) so it fails SILENTLY; no test caught it (both the render and the hit-test are mutants::skip + coverage-excluded shim). Caught only by the inspect critic reading the render AND its inverse together. Fix: extracted a pure row_at(local_row, start, end, capacity) = start + local_row.saturating_sub(capacity - (end-start)) that mirrors the bottom-anchor, tested cov/MSI 100; pane_grid_pos now uses it. Lesson: a rendering transform and its inverse hit-test are a coupled pair — the design that changes the render anchor must change the inverse in lockstep, and inspect should explicitly diff a render-geometry change against every consumer of the inverse mapping (mouse/selection/scroll-to).

## DL-cluster-22ddda27bf91
*topic: #181 AgentLauncherState::move_up used the modulo wrap `(sel + n - 1) % n` over a fixed 2-element list. At n=2 the `- 1`→`+ 1` mutant is EQUIVALENT (±1 are identical mod 2), so it survives cargo-mut... · confidence: 0.500*

#181 AgentLauncherState::move_up used the modulo wrap `(sel + n - 1) % n` over a fixed 2-element list. At n=2 the `- 1`→`+ 1` mutant is EQUIVALENT (±1 are identical mod 2), so it survives cargo-mutants → MSI < 100 → gate:5 RED. Caught by both inspect critics + an empirical targeted run (MISSED). Root cause: a modulo-arithmetic wrap over a small fixed list makes symmetric ± mutants indistinguishable — no test over the API can kill them. Fixed by the explicit branch form `if sel == 0 { n-1 } else { sel-1 }`, whose mutants all produce a wrong/out-of-bounds index a test catches; re-ran cargo mutants → 0 missed.

## DL-cluster-23305b5f0dff
*topic: The raw-mode keystroke mapper `key_input_from_keystroke` (marley_app app.rs) filtered ctrl/alt but NOT the platform (⌘) modifier, so in alt-screen mode an UNBOUND cmd-chord (⌘C/⌘V/⌘A/⌘S — anything ... · confidence: 0.500*

The raw-mode keystroke mapper `key_input_from_keystroke` (marley_app app.rs) filtered ctrl/alt but NOT the platform (⌘) modifier, so in alt-screen mode an UNBOUND cmd-chord (⌘C/⌘V/⌘A/⌘S — anything not in the keymap) fell through the keymap-chord check, got mapped to its bare character, and streamed to the program via encode_key→write_bytes. Result: ⌘C in vim/less would INSERT a literal 'c' instead of being a no-op (or a future copy). The cooked path was already safe (key_from_keystroke maps control||platform → Key::Other → ignored), but the new raw path forgot the platform half. Invisible to the gate because the routing lives in the mutants::skip'd + coverage-excluded app.rs shim — only inspect/headed testing catches it. FIX: `if keystroke.modifiers.platform { return None; }` at the top of key_input_from_keystroke (mirroring the cooked filter). Prevention: PR-claude-raw-input-passthrough-must-filter-platform-chords-001.

## DL-cluster-2362bf184867
*topic: #285's incremental-highlight windowing shipped GATE GREEN with a CRITICAL correctness bug: a SHRINKING edit (backspacing off a comment's tail, or a token re-tokenizing when a delimiter like `*/` is... · confidence: 0.500*

#285's incremental-highlight windowing shipped GATE GREEN with a CRITICAL correctness bug: a SHRINKING edit (backspacing off a comment's tail, or a token re-tokenizing when a delimiter like `*/` is deleted) made the surviving token's highlight VANISH. Root cause: #285 assumed `changed_ranges ∪ edit-span ⊇ every stale span`. FALSE — `changed_ranges` is a BYTE diff, not a token diff, so (a) a token whose extent shrank while its bytes stay textually unchanged is not reported (a pure deletion yields an EMPTY window [p,p); tree-sitter's set_byte_range excludes a node ending exactly at p), and (b) a token merely ADJACENT to the edit that re-tokenizes at the changed-range boundary is just outside a byte-tight window. In both, splice drops the stale cached span with nothing to replace it. WHY IT SHIPPED: the #285 equivalence corpus was HAND-PICKED and lacked shrinking-tail/adjacent-re-tokenize cases, and the inline inspect trace (+ the first critic) reasoned the fix complete — both WRONG. The bug (and, later, that the FIRST #288 fix was still incomplete — twice) was found only by a DIFFERENTIAL FUZZER (random edit chains asserting inc==fresh). FIX (#288): cover_edited_cached widens the window to the surviving footprint of every cached span overlapping OR abutting [start,old_end]; snap_to_lines snaps to whole-line bounds; equivalence scoped to same-incremental-tree (tree-sitter can itself diverge on error inputs). Durable fix: a tree-match-guarded in-test differential fuzzer, 0 same-tree divergences / 120k steps.

## DL-cluster-24218f5560a4
*topic: #324 signature-help card opened from an ASYNC response anchored to the LIVE caret and dismissed on ROW-change only, so: (1) pressing Enter right after `(` (caret to a new row) before the answer lan... · confidence: 0.500*

#324 signature-help card opened from an ASYNC response anchored to the LIVE caret and dismissed on ROW-change only, so: (1) pressing Enter right after `(` (caret to a new row) before the answer landed opened a card for the finished call on the new row; (2) the overlay re-read the live caret each frame, so the card GLIDED along the line as the cursor moved through args. The SignatureKey carried `version` but apply never used it. Fix: apply drops the answer when the live caret row != the request row (key.line); OpenSignature stores a FIXED anchor_col and the overlay anchors there (stays put). A live-version check was the wrong fix — signature help persists across arg-typing (which bumps the version without a new request), so a version drop would kill the card on every keystroke; the row guard preserves persistence.

## DL-cluster-24a5162d9039
*topic: #372 inspect F1: the one-time GET-status gate in run_once decoded the WHOLE socket buffer strictly (str::from_utf8(&buffer).unwrap_or("")) before parsing the status line — an invalid UTF-8 byte any... · confidence: 0.500*

#372 inspect F1: the one-time GET-status gate in run_once decoded the WHOLE socket buffer strictly (str::from_utf8(&buffer).unwrap_or("")) before parsing the status line — an invalid UTF-8 byte anywhere in the buffer (a body byte, a multibyte char split at the 4KB boundary) would discard the entire buffer including the ASCII status line, so a non-2xx standing-GET response would slip past the gate and the pump would idle on it forever — the EXACT latent gap #372 exists to close. Self-inconsistent with the valid-prefix decode 7 lines below in the same loop. Fixed by decoding the valid UTF-8 prefix (buffer[..valid]); the ASCII status line always sits in the valid prefix. Found by 2 of 3 inspect critics.

## DL-cluster-2724c2cf2ced
*topic: #326 off-thread search worker: ensure_search_worker_and_send gated the spawn on search_worker.is_none() and set it to Some once, never resetting to None. So the moment the worker thread exited (a p... · confidence: 0.500*

#326 off-thread search worker: ensure_search_worker_and_send gated the spawn on search_worker.is_none() and set it to Some once, never resetting to None. So the moment the worker thread exited (a panic in the walk body, or a channel send error), search_worker stayed Some holding a dangling sender — every subsequent ⌘⇧F: is_some → skip spawn → tx.send errors → returns false → the picker opens, clears its list, and shows nothing FOREVER, no error/flash/recovery short of an app restart. Latent today (run_search has no panic surface), but the non-respawn design converts any future/rare thread death into permanent silent breakage. Caught by inspect critic 2 (concurrency). Fixed: on tx.send err, set search_worker = None so the next launch respawns.

## DL-cluster-295876249bf2
*topic: The PTY resize shim advanced the PaneState.pty_size guard shadow unconditionally (`let _ = session.resize(cols,rows); pty_size = (cols,rows)`), even when the resize ioctl failed. The unchanged-guar... · confidence: 0.500*

The PTY resize shim advanced the PaneState.pty_size guard shadow unconditionally (`let _ = session.resize(cols,rows); pty_size = (cols,rows)`), even when the resize ioctl failed. The unchanged-guard compares the next computed grid against pty_size, so a failed resize (which leaves the PTY + alacritty grid at the OLD size) records the TARGET size in the shadow and elides every future retry — the pane stays silently stuck mis-sized until the rect changes to a different grid. Also contradicted pty_size's docstring ("the size last APPLIED"). FIX: advance the shadow only on success — `if state.session.resize(cols,rows).is_ok() { state.pty_size = (cols,rows); }`; a persistently-failing (dead) pane then re-attempts one cheap ioctl/frame instead of silently lying. Caught by an inspect shim-critic tracing resize()'s early-return-on-ioctl-failure against the unconditional shadow write. Prevention: PR-claude-advance-shadow-only-on-success-001.

## DL-cluster-29760bccdf20
*topic: M10 #163: the boot restore consumed PaneId blocks 1..N via a LOCAL `let mut pane_blocks` counter, but the Self constructor still hard-initialized the FIELD to 0 — the first post-boot ⌘T would re-mi... · confidence: 0.500*

M10 #163: the boot restore consumed PaneId blocks 1..N via a LOCAL `let mut pane_blocks` counter, but the Self constructor still hard-initialized the FIELD to 0 — the first post-boot ⌘T would re-mint block 1, colliding with a restored grid's block and resurrecting the #158 cross-tab aliasing that #167 had JUST structurally killed (and the exact close-cleanup would strip the WRONG tab's agents). Caught in inspect before gate/commit. Fix: seed the field from the local (`pane_blocks,`). The class: when new() advances a local accumulator that has a same-named field, the field init MUST consume the local — after writing any boot loop, grep the constructor for `field: 0,` shadows.

## DL-cluster-297c6971a305
*topic: #326 project-search worker (run_search) called std::fs::read(&path) and only THEN checked bytes.len() <= VIEWER_MAX_BYTES. std::fs::read pre-sizes its buffer from file metadata, so a multi-GB non-g... · confidence: 0.500*

#326 project-search worker (run_search) called std::fs::read(&path) and only THEN checked bytes.len() <= VIEWER_MAX_BYTES. std::fs::read pre-sizes its buffer from file metadata, so a multi-GB non-gitignored file (a CSV/sqlite/media blob in a non-git dir the walk doesn't exclude) → a multi-GB transient allocation that ABORTS the whole process on OOM (Vec::with_capacity abort is not caught by the match arm), and a TOCTOU fifo/device gap (is_file at enumeration, read later → a fifo/dev/zero swapped in blocks the worker forever or grows unbounded). The repo had ALREADY fixed this exact antipattern in load_code_view_state (stat-first, the #106 lesson) but run_search didn't mirror it. Caught by inspect critic 4 (security/resource). Fixed: std::fs::metadata gate (is_file + len<=cap) BEFORE fs::read, post-read size check kept as a TOCTOU backstop.

## DL-cluster-2a03645d7d32
*topic: The FULL/diff gate wedged ~35 min TWICE on the real-PTY integration test `workspace_two_real_sessions_are_independent` sitting at 0.0% CPU under `cargo llvm-cov` instrumentation (a session spawn/ha... · confidence: 0.500*

The FULL/diff gate wedged ~35 min TWICE on the real-PTY integration test `workspace_two_real_sessions_are_independent` sitting at 0.0% CPU under `cargo llvm-cov` instrumentation (a session spawn/handshake deadlock). NOT a #253 regression (undo/redo is pure — no PTY/session code) and NOT memory pressure (76% free — the --jobs 2 fix held). Diagnosed via `ps -o pid,pcpu,etime,command` showing the test binary alive 36min at 0% CPU (a deadlock, not slow compute). Killed the gate tree + the hung test, re-ran on a quiet system → the same test passed in ~2s and the whole diff gate finished in ~105s. A flaky real-session test under coverage instrumentation; a nextest retry/timeout on the real-session tests would harden it (surfaced to chad as a possible follow-up, not filed — out of #253 range).

## DL-cluster-2aa26f872dde
*topic: #317: find_references (a new app.rs flash-dispatcher shim over send_references_request) was missing #[cfg_attr(test, mutants::skip)] while its structural twin go_to_definition has it. app.rs is cov... · confidence: 0.500*

#317: find_references (a new app.rs flash-dispatcher shim over send_references_request) was missing #[cfg_attr(test, mutants::skip)] while its structural twin go_to_definition has it. app.rs is coverage-excluded but NOT mutation-excluded — gate:5 runs cargo mutants --in-diff over the diffed app.rs lines with no -f restriction, so a NEW shim fn's lines ARE mutated and are kept out of the set ONLY by a per-fn skip. With no test/headless drive invoking find_references (the injection lane drive_references_for_test bypasses it), its 2 mutants (body→(), delete !) would compile and SURVIVE → MSI<100 → gate:5 FAILS at validate/commit. FIX: added the skip mirroring go_to_definition. Caught by inspect critic 3 before the gate. LESSON: every new app.rs render/thread/IO/flash shim needs the mutants::skip; diff the new fn against its sibling to confirm the attribute is present AND hugs the fn (the mutants-skip-detach-trap).

## DL-cluster-2b2d8c2cbc13
*topic: #399: the arrangement snapshot walked grid.pane_ids() — which returns panes sorted by numeric PaneId (creation order) — while the codec promised "visual order" in three doc sites. The two orders di... · confidence: 0.500*

#399: the arrangement snapshot walked grid.pane_ids() — which returns panes sorted by numeric PaneId (creation order) — while the codec promised "visual order" in three doc sites. The two orders diverge under an ordinary gesture (insert_split_at mints a monotonic id but inserts ADJACENT to the target; splitting a non-last pane of a 3-pane grid yields tree order [0,3,1,2] vs id order [0,1,2,3]), so a saved arrangement could reopen with its panes silently rearranged — and re-persist wrong. The shell codec next door already did it right (serialize_grid flattens grid.group(), the DFS tree walk). The apply path compounded it by rotating pre-seed cells to after the terminal seed. Fix: snapshot + card summary walk grid.group().panes(); apply mounts pre-seed cells with SplitDirection::Before re-targeting the seed.

## DL-cluster-2cd9dce7acd4
*topic: #273 inspect (self-review, critic-confirmed): gpui's UniformListScrollHandle applies a pending deferred_scroll_to_item AFTER the base offset at the next layout and it WINS (uniform_list.rs:393-448)... · confidence: 0.500*

#273 inspect (self-review, critic-confirmed): gpui's UniformListScrollHandle applies a pending deferred_scroll_to_item AFTER the base offset at the next layout and it WINS (uniform_list.rs:393-448) — so a scroll_to_item minted for file A that is still pending when the render transition switches the shared handle to file B teleports B to A's row. Fixed: the per-file scroll-sync transition clears deferred_scroll_to_item in the same borrow_mut that restores the incoming offset. Residual constraint (doc'd on the helper): an open-file+jump-to-row single action must defer the jump a frame or park the row on the OpenFile — the transition clear cannot distinguish a fresh incoming deferred from a stale outgoing one.

## DL-cluster-2cf6a093e766
*topic: #383 implement made drive.swift's `focus` verb a programmatic raise (NSRunningApplication.activate) to fix its content-click misfire — but that removed the mouse-DOWN that gpui uses to set window k... · confidence: 0.500*

#383 implement made drive.swift's `focus` verb a programmatic raise (NSRunningApplication.activate) to fix its content-click misfire — but that removed the mouse-DOWN that gpui uses to set window keyboard focus. A raise/activation does NOT set gpui `window.focus` (only a mouse-down on a `track_focus` element does), so a following `type:` keystroke had no focused view and was dropped — breaking REQ-002's `focus type:…` typing flow. Caught by an inspect critic (traced gpui-0.2.2 source + the app's own headless_drive.rs:158-160 "a test window starts unfocused… injected keystrokes go nowhere"), then CONFIRMED empirically by driving: `focus "type:echo REQ002_probe" enter` → the terminal stayed at a bare prompt (383-req002.png); `clickat:<pane> "type:echo CLICKPROBE" enter` → the echo ran (383-clickprobe.png). Fixed harness-only: `focus` stays activation-only (per D1) and the TYPING flow routes through a `clickat:<pane>` (which posts the mouse-down that focuses the pane); the gpui-focus fact + the two captures documented in the README/spec.

## DL-cluster-2cfee35a2ce4
*topic: #315: `Lang::TypeScript`/`Lang::Tsx` compiled `tree_sitter_typescript::HIGHLIGHTS_QUERY` ALONE, but that const is the `inherits: ecma` OVERLAY only (5 captures: keyword[TS-only words]/punctuation.b... · confidence: 0.500*

#315: `Lang::TypeScript`/`Lang::Tsx` compiled `tree_sitter_typescript::HIGHLIGHTS_QUERY` ALONE, but that const is the `inherits: ecma` OVERLAY only (5 captures: keyword[TS-only words]/punctuation.bracket/type/type.builtin/variable.parameter) — no string/comment/number/function and none of the common ECMAScript keywords, which live in `tree_sitter_javascript::HIGHLIGHT_QUERY`. Because grammar_lang(TS)=Some, the render takes the tree path and never consults the hand-lexer floor → .ts/.tsx showed ONLY type annotations (blank strings/comments/keywords) — WORSE than the pre-ticket floor. A naive per-grammar "closed capture set" pin test PASSES while the language is visibly broken (it pins the 5 overlay captures). Found only by the grammar-map critic reading the actual .scm files + running each language through a probe; self-review + 3 other critics missed it. Fix: concatenate the JS base query (+ JSX for TSX) ahead of the TS overlay — the standard tree-sitter `inherits` resolution — `format!("{JS}\n{TS}")`; verified it compiles against LANGUAGE_TYPESCRIPT/LANGUAGE_TSX and now colors strings/comments/numbers/keywords/functions.

## DL-cluster-2dd1467a5c64
*topic: #314 format-on-save: the deferred save (parked while the format round-trips) completed via do_plain_save to save_active, which writes the CURRENTLY-ACTIVE editor. If the user pressed CmdS on editor... · confidence: 0.500*

#314 format-on-save: the deferred save (parked while the format round-trips) completed via do_plain_save to save_active, which writes the CURRENTLY-ACTIVE editor. If the user pressed CmdS on editor A (format armed) then switched to tab B before the format answered (50ms-2s round-trip), every completion path (apply success, settle on Err/stale, the deadline) saved B (or nothing, if B is a terminal) instead of A. Net: A was SILENTLY never saved (the latch consumed, A still dirty), and the deadline even flashed 'timed out — saved' misattributed to B. A lost save = the one unforgivable outcome (D-SAVE-NEVER-BLOCKED). Root cause: an async completion re-read 'the active editor' at completion time instead of binding to the ORIGIN editor captured at request time. Found by an adversarial critic (my own trace confirmed one-write for the single-editor case but MISSED the cross-editor switch). Fix (bounded, keeps D-ONE-WRITE): active_is_origin(key) gates every completion's do_plain_save on the active editor still being the origin (has_path(path_from_file_uri(key.uri))); a switch abandons cleanly (no wrong-editor save, origin stays visibly dirty). The proper origin-targeted save (a non-active-editor save path) is blocked on save_active's active-only #275/#284 conflict machinery — filed as a follow-up.

## DL-cluster-303d3ed43a72
*topic: #280 inspect HIGH: inserting pane_mouse_cell (with its own doc block) BETWEEN pane_grid_pos's doc+#[cfg_attr(test, mutants::skip)] and its fn re-bound the skip to the NEW fn — pane_grid_pos gained ... · confidence: 0.500*

#280 inspect HIGH: inserting pane_mouse_cell (with its own doc block) BETWEEN pane_grid_pos's doc+#[cfg_attr(test, mutants::skip)] and its fn re-bound the skip to the NEW fn — pane_grid_pos gained 14 live mutants (untested by design) and the brand-new geometry fn had ZERO, plus the old doc became a Frankenstein description of the wrong fn. STRIKE SIX of the documented detach trap (previously #183/#184/#272 and others) — it fires precisely when adding a sibling fn above a skipped shim, which is the most natural insertion point. The standing rule held: cargo mutants --list -f AFTER the edit is the only reliable detector (the critic ran it; re-seat verified 0/28). The class needs the --list check to be REFLEXIVE on any edit within 5 lines of a skip attribute.

## DL-cluster-3125c34be88e
*topic: #396 gate:14 first run RED: two workspace.rs intra-doc links rotted under the migration — a pub item (TerminalPane) linking the now-private PaneContent, and a [`terminal`](Self::terminal) anchor po... · confidence: 0.500*

#396 gate:14 first run RED: two workspace.rs intra-doc links rotted under the migration — a pub item (TerminalPane) linking the now-private PaneContent, and a [`terminal`](Self::terminal) anchor pointing at a retired accessor (replaced by terminal_id). rustdoc -D warnings caught both only at the gate; the rename/de-genericize step should sweep doc anchors alongside code sites.

## DL-cluster-31d278539865
*topic: Inspect F1 (all 3 critics): the diagnostics store was WRITTEN keyed by the server's raw publishDiagnostics uri string but READ keyed by file_uri(absolute(path)). These agree only when the server ec... · confidence: 0.500*

Inspect F1 (all 3 critics): the diagnostics store was WRITTEN keyed by the server's raw publishDiagnostics uri string but READ keyed by file_uri(absolute(path)). These agree only when the server echoes the didOpen uri byte-for-byte — rust-analyzer on a plain-ASCII path does (so the acceptance path passed), but any server that re-encodes the uri (an @scope path → %40 vs literal @, a space, a sub-delim) or resolves a symlink would SILENTLY drop every squiggle/gutter/count/F8 target for that file, with no error. Fix: key the store by canonical PathBuf (matching the #309 docs map) — decode the wire uri via a new pure path_from_file_uri (the inverse of file_uri) and normalize through the SAME absolute() the lookup uses. Guarded by a path_from_file_uri round-trip unit (decode∘encode=id over special-char paths) + a headless integration test feeding a real publishDiagnostics through the whole capture path and asserting the accessors agree on the key.

## DL-cluster-32fab906ec54
*topic: #398 rail cross-list v1 gave every terminal mounted in a multi-cell grid its own home-section row (tab_listed = single-cell tabs only). An ordinary 2-cell split then rendered THREE Terminal-section... · confidence: 0.500*

#398 rail cross-list v1 gave every terminal mounted in a multi-cell grid its own home-section row (tab_listed = single-cell tabs only). An ordinary 2-cell split then rendered THREE Terminal-section entries (the tab row + a CrossRef per cell) — a visual regression of every pre-existing split, caught by the inspect critic. Root cause: the "pane-mounted" predicate was structural (grid arity) when the real signal is FOREIGN HOSTING (content hosted by ≥2 distinct tabs). Fixed by reworking the semantics: a tab row is the home entry for ALL its cells; ⊞ marks foreign-hosted tabs; per-content cross rows only for editors (whose Editor-section entry exists nowhere else).

## DL-cluster-342aa319ba66
*topic: #315: adopting tree-sitter-toml-ng's highlights.scm mis-colored every `key = value`. The query captures `(pair (bare_key)) @property` over the WHOLE pair node; our span sweep (`sweep_disjoint`) is ... · confidence: 0.500*

#315: adopting tree-sitter-toml-ng's highlights.scm mis-colored every `key = value`. The query captures `(pair (bare_key)) @property` over the WHOLE pair node; our span sweep (`sweep_disjoint`) is outer-wins-clip-forward (sort by (start,end), keep first, clip the rest forward), whereas tree-sitter's highlight convention is innermost-wins. So the broad `(0..pair_end, Property)` span survives and the narrower value captures (Number/Str) get clipped away → `foo = 1` rendered key=Type, `1`=Property (REQ-003 inverted) — and a REGRESSION vs the hand-lexer that colored TOML correctly. Root class: an adopted query with broad container captures is incompatible with an outer-wins sweep. Fix (v1): revert grammar_lang(Toml)=None → TOML stays on the correct hand-lexer floor; Lang::Toml + the pinned grammar retained. Proper fix (follow-up): an innermost-wins sweep — but that also changes Rust `(attribute_item) @attribute` whole-attribute absorption, so it needs its own ticket + Rust-attribute fixtures. Also surfaced a latent LOW: JSON object keys are double-captured (`string.special.key`+`string`) at an identical range, so which wins rested on `sort_unstable`'s "may reorder equal elements" → switched to a stable `sort_by_key` (query order → Property deterministic).

## DL-cluster-3575ab988650
*topic: #398 made view_count>1 reachable for terminals, and THREE pre-existing per-view walks silently became wrong over shared content: (1) five close paths scrubbed the content-keyed maps (agents/remotes... · confidence: 0.500*

#398 made view_count>1 reachable for terminals, and THREE pre-existing per-view walks silently became wrong over shared content: (1) five close paths scrubbed the content-keyed maps (agents/remotes/last_agent/notify_ticks) per VIEW — closing one twin cell deleted a live agent's Fleet identity and disarmed the never-auto-close-a-remote guard; (2) the per-cell PTY resize loop over the one per-content pty_size shadow made different-sized twin cells fight (two ioctls + SIGWINCH storm per frame, reachable via split-right then add-down); (3) the pump pumped each VIEW (double-drain + notify_ticks counting 2x, halving the #203 threshold). All three were correct while view==content held (pre-#398) and none announced their assumption. Fixed: scrubs unified into the release tail firing on last-view drop only; one resize authority per content (focused cell wins); pump dedupes per content.

## DL-cluster-3b691e2b7a3b
*topic: The spec's reuse list named `directories` for home-dir resolution ("MIT/Apache"), but `directories` pulls `option-ext` (MPL-2.0) transitively via `dirs-sys`, and Marley's deny.toml allowlist delibe... · confidence: 0.500*

The spec's reuse list named `directories` for home-dir resolution ("MIT/Apache"), but `directories` pulls `option-ext` (MPL-2.0) transitively via `dirs-sys`, and Marley's deny.toml allowlist deliberately bars MPL (permissive-only). gate:8 cargo-deny rejected it at implement-verify (error[rejected]: option-ext-0.2.0 license MPL-2.0 not explicitly allowed). The direct dep's own license being permissive is NOT sufficient — the whole transitive tree must satisfy the allowlist. Fix: switched to the `home` crate (0.5.12, MIT/Apache — cargo/rustup's home resolver, effectively a leaf, no MPL); `home::home_dir() -> Option<PathBuf>` feeds the home_dir_from seam directly. option-ext is now absent from the lockfile. Caught by the gate, not shipped.

## DL-cluster-3ca70ea401a7
*topic: Validate gate went RED twice: (1) the inspect-F9 DRY extraction of `token_color` into a top-level fn made cargo-mutants mutate it standalone (`body → Default::default()`) — it survived because the ... · confidence: 0.500*

Validate gate went RED twice: (1) the inspect-F9 DRY extraction of `token_color` into a top-level fn made cargo-mutants mutate it standalone (`body → Default::default()`) — it survived because the helper is called only from the coverage-excluded render, so no test killed it (MSI 98.1%); fix = a token_color_maps_each_kind unit test with exact per-kind color assertions. (2) hover.rs had 1 uncovered line — markup_to_string's `_ => String::new()` arm (a non-string/object/array `contents`); fix = a `{"contents": 42}` test. LESSON: extracting an inline expression (previously buried in an unmutated render closure) into a named fn CREATES a new standalone mutation target — pair the DRY extraction with a direct unit test, or it fails the MSI floor even though behavior is unchanged. Both caught by the gate, fixed at source (§0), re-run GREEN (MSI 100, coverage 100).

## DL-cluster-3e0e9cbcc996
*topic: #299: toggle a comment at N cursors, type ONE character, press ⌘Z — and the editor DELETES THE USER'S CODE, unrecoverably. `"foo\nbar"` + 2 mid-line carets → ⌘/ → `"// foo\n// bar"` → type X → `"//... · confidence: 0.500*

#299: toggle a comment at N cursors, type ONE character, press ⌘Z — and the editor DELETES THE USER'S CODE, unrecoverably. `"foo\nbar"` + 2 mid-line carets → ⌘/ → `"// foo\n// bar"` → type X → `"// foXo\n// baXr"` → ⌘Z → **`"oXo\naXr"`**. The `f` and the `b` are gone, and the undo stack is now EMPTY so a second ⌘Z does nothing. ROOT CAUSE (undo.rs `coalesces_into`, shipped by #297, first REACHED by #299): `end_group` offers each closing group to `coalesces_into`, which decides "is this a typed run continuing?" from STRUCTURE — bracketed, record count == cursor count, selections line up, every record a pure insert whose NEW insert is 1 char. The ⌘/ toggle satisfies every one of those while being nothing of the sort: its records are anchored at the MIN-INDENT COLUMN, not at the cursors. So the typed char is appended onto a record whose `at` is somewhere else, and `Buffer::undo` deletes `at..at + inserted.chars().count()` — the WRONG characters. The false premise is stated VERBATIM in the fn's own doc: "Because the cursors did not move, each new insert lands precisely at the end of its own record's text, so appending char-wise is contiguous by construction — no offset arithmetic is needed or wanted." True for a typed run; false for the first bracketed group whose records are not cursor-anchored. Its single-cursor twin `record()` CHECKS contiguity (`last.at + last.inserted.chars().count() == rec.at`); `coalesces_into` only ARGUED it. Enter is affected identically (a 1-char `"\n"` insert). Found independently by THREE of four critics and by me; the FOURTH read the same code and declared it clean, having misread the guard as testing `old.inserted` when it tests `new.inserted`. FIX: `UndoGroup.cursor_anchored: bool` — set true ONLY by the N-cursor typed-insert path, false by every hand-bracketed group (Tab, ⌘/, replace-all); `coalesces_into` requires it. Kills the whole class (#307 multi-cursor Tab, #300 line-move, #303 delete-word would all have walked into it).

## DL-cluster-3e9ff495fbba
*topic: #316: the tuned Light SyntaxPalette shipped `keyword` = hsla(0.52,0.58,0.34) and `property` = hsla(0.52,0.50,0.34) — IDENTICAL hue AND lightness, 0.08 saturation apart = perceptually the same dark ... · confidence: 0.500*

#316: the tuned Light SyntaxPalette shipped `keyword` = hsla(0.52,0.58,0.34) and `property` = hsla(0.52,0.50,0.34) — IDENTICAL hue AND lightness, 0.08 saturation apart = perceptually the same dark teal. Both cleared WCAG AA (4.51:1 / 4.81:1 vs the near-white bg), so the `default_palettes_clear_aa` contrast-matrix test was BLIND to it — a keyword and a field/JSON-key rendered indistinguishably in the default Light theme (two of the most frequent token kinds). Root cause: keyword was pinned to the Light accent (0.52/0.58/0.34) and property was placed at the same accent hue+L. A parallel Dark defect: `type` hsla(0.12,0.50,0.65) ≈ `number` hsla(0.09,0.55,0.62) (both amber, dH=0.03/dL=0.03, saturation did not rescue). The class: a CONTRAST test proves legibility-vs-background but says NOTHING about slot-vs-slot distinctness — two colors can both pass AA and still be the same color. Caught by an adversarial grammar-map/WCAG critic that COMPUTED the pairwise hue/lightness/saturation deltas (the AA matrix + my self-review missed it; the design had flagged min-hue-distance as an inspect edge). Fix: moved Light property to a distinct violet hsla(0.70,0.50,0.40) (8.9:1), Dark number to a red-amber hsla(0.04,0.60,0.62), nudged Light keyword L 0.34→0.32 for AA headroom (4.51→4.98), and added a `tuned_palettes_have_distinct_slots` gate (collision iff close on hue [circular] AND lightness AND saturation).

## DL-cluster-411d9a365223
*topic: #267 inspect (critic B, HIGH): every full-capture overlay arm in the on_key_down ladder returns WITHOUT stop_propagation, so once an EntityInputHandler is registered, gpui's dispatch_keystroke fall... · confidence: 0.500*

#267 inspect (critic B, HIGH): every full-capture overlay arm in the on_key_down ladder returns WITHOUT stop_propagation, so once an EntityInputHandler is registered, gpui's dispatch_keystroke fallback (propagate && key_char.is_some()) delivers the consumed key's key_char into the editor buffer in the HEADLESS lane — finder plain-Enter lands "\n" at the caret; top-search Enter on a File hit opens the editor tab then instantly dirties it. The blocked-predicate can't save it: the overlay's Enter FLIPS its state off before the fallback runs. Real macOS discards via the branch-B doCommandBySelector path — the enforced test lane diverges from mac. Fix: cx.stop_propagation() in all 13 full-capture overlay arms + the completion popup's handled branch (its fall-through printables stay live by design).

## DL-cluster-4135521f3c7d
*topic: #308 lsp_host on_message: an `initialize` ERROR response was misclassified as a successful handshake. The code did `if let Ok(value)=result { parse }` then UNCONDITIONALLY stepped Event::Initialize... · confidence: 0.500*

#308 lsp_host on_message: an `initialize` ERROR response was misclassified as a successful handshake. The code did `if let Ok(value)=result { parse }` then UNCONDITIONALLY stepped Event::InitializeResult → phase Ready + `initialized` sent to a server that REJECTED initialize (an LSP-spec violation), status falsely "ready", the dead wire never restarts (lifecycle thinks it's Ready). Root cause: an async response carries a Result, but only the Ok arm was routed to a distinct outcome; the Err arm silently fell through to the success transition. Found by the correctness critic (P3.5). Fixed: match the Result — Ok→InitializeResult, Err→on_connection_lost() (tear down, restart cap governs). Class: any request whose RESPONSE can be an error must branch the error to a real failure transition, never let it fall through to the success path.

## DL-cluster-424ce4220e20
*topic: #399: NAME_PANE_ID was drafted as CommandId(20) after a grep for "CommandId([0-9])," / "CommandId(1[0-9])," — a pattern that structurally could not match the 20-29 block cockpit_commands() already ... · confidence: 0.500*

#399: NAME_PANE_ID was drafted as CommandId(20) after a grep for "CommandId([0-9])," / "CommandId(1[0-9])," — a pattern that structurally could not match the 20-29 block cockpit_commands() already mints. action_for_command maps 20 → "toggle-find-regex", and that resolver is the FIRST arm of the palette dispatch chain, so the new "Name Pane…" verb was dead code that silently flipped the find bar's regex mode (and duplicated id 20 in the command list). No existing test could catch it: every_cockpit_command_resolves_to_a_verb checks only the forward direction, and all the new entry points are mutants::skip-masked. Caught by two independent inspect critics tracing the dispatch chain end-to-end. Fix: CommandId(30) + a Phase-4 reverse-guard unit (specially-dispatched statics must resolve to NO verb) + a no-duplicate-ids pin.

## DL-cluster-4476f3574e57
*topic: #274 inspect TS-F2: the REQ-003 perf pin ("incremental < full/3 end-to-end") encoded the ticket's premise that "the parse half dominates" — the critic's probe measured it BACKWARDS: the tree-sitter... · confidence: 0.500*

#274 inspect TS-F2: the REQ-003 perf pin ("incremental < full/3 end-to-end") encoded the ticket's premise that "the parse half dominates" — the critic's probe measured it BACKWARDS: the tree-sitter re-parse IS 9× faster (1.0 vs 9.2 ms release on 8k lines) but the O(file) query walk (5.7 ms over 27k captures) runs in full on every incremental call, flooring end-to-end at 0.46. A pin written from an inherited premise instead of a measurement would have sent Phase 4 chasing an impossible number or, worse, invited a gate-weakening 'adjustment' under pressure. CLASS: measure the pin's decomposition BEFORE writing it — a ratio pin needs per-stage attribution (parse/query/post) from a probe, not the ticket's narrative. Fix: REQ-003 amended to pin each true thing (parse-only < 1/3, end-to-end < 3/4, frame-safety via the off-thread REQ); the query-walk windowing recorded as a measured follow-up. Also W-F1 (the dead-worker pending wedge behind its own guard — the Phase-3 comment overclaimed "can never wedge") and TS-F1 (point_at non-boundary panic reachable through the public API) fixed at source.

## DL-cluster-44c704b6e44c
*topic: TICKET-020 (layout.rs neighbor): `bool::then_some(x)` evaluates its argument `x` EAGERLY (unlike `bool::then(|| x)` which is lazy). The preceding-sibling index was written `(index >= 1).then_some(i... · confidence: 0.500*

TICKET-020 (layout.rs neighbor): `bool::then_some(x)` evaluates its argument `x` EAGERLY (unlike `bool::then(|| x)` which is lazy). The preceding-sibling index was written `(index >= 1).then_some(index - 1)` — but `index - 1` is computed regardless of the `index >= 1` guard, so for `index == 0` (a pane at the start of a Split, e.g. `neighbor(A, Left)` at a matching-axis ancestor) `0usize - 1` underflowed → panic on a reachable input path (a §14 violation). Caught by the neighbor_boundary_rule unit test (panicked at layout.rs:206). Fixed with `index.checked_sub(1)` (returns the preceding index, or None at the edge, with no eager subtraction). NOTE: clippy's `unnecessary_lazy_evaluations` lint actively pushes `.then(|| index - 1)` → `.then_some(index - 1)`, which reintroduces the bug — `checked_sub` sidesteps both.

## DL-cluster-451c0ce7edbe
*topic: #299 (⌘/ comment toggle): at PLAN time I found that `/// foo` starts with `//`, so the reference's naive prefix test ("already commented" = `trim_start().starts_with(token)`) uncomments it to `/ fo... · confidence: 0.500*

#299 (⌘/ comment toggle): at PLAN time I found that `/// foo` starts with `//`, so the reference's naive prefix test ("already commented" = `trim_start().starts_with(token)`) uncomments it to `/ foo` — mangled, irreversible. I wrote a whole "THE ROUND-TRIP HAZARD" section about it, recorded it as D7 + REQ-008 with its own pinning test, and moved on satisfied. **I had examined only the token I happened to think about.** `#` — the comment token for TWO of the three supported languages (Shell, TOML) — was never looked at, and it destroys more, and worse: `#!/bin/sh` → `!/bin/sh` (the script silently stops being executable and fails at EXEC time, not compile time); `## Section` → `# Section` → `Section`; `####` and `//////` banner rules lose one token PER PRESS down to `""`, which is an ABSORBING state because blank rows are skipped and never come back. And `//!` — a Rust MODULE DOC, the first line of literally every file in this crate — → `! PURE — …`. None of the five round-trip. Worse, D7's written RATIONALE was false: "from an already-commented start the toggle NORMALIZES (`//x` → `x` → `// x`), which is correct behavior, not a bug" is asserted as a GENERAL property and holds only when the text after the token does not itself start with the token. A test written to that rationale would have encoded the falsehood and then defended it. Raised independently by two critics — both of whom saw it precisely BECAUSE I had already blessed the `///` case, which made the asymmetry visible. FIX: the token must not be followed by MORE OF ITSELF, nor by `!` — those are richer markers (doc comments, shebangs, banners), so ⌘/ COMMENTS them (`//! x` → `// //! x` → back) rather than shredding them. A deliberate divergence from the reference (§0 — do not ship known irreversible harm). All five now round-trip byte-identically.

## DL-cluster-4599473efb5c
*topic: #330: `jump_to_sticky_header` called `open_and_place_caret(path, |buf| line_start(row))` and both its doc comment and the design notes asserted it "places the caret, pushes the NavStack, centers" —... · confidence: 0.500*

#330: `jump_to_sticky_header` called `open_and_place_caret(path, |buf| line_start(row))` and both its doc comment and the design notes asserted it "places the caret, pushes the NavStack, centers" — but `open_and_place_caret` does NOT push the NavStack. It only opens the file, sets the caret, and parks pending_center_row. Every sibling jump (#312 goto-definition, the symbol picker, the problems panel) captures `(path, active_caret())` BEFORE the jump and pushes `NavLoc` EXPLICITLY afterward — proof the helper doesn't push (they would double-push). Effect: REQ-007 ("caret placed + NavStack pushed") violated — ⌃- would not return to where you were reading after clicking a sticky header, and the planned validate drive asserting the push would have failed. The design note mis-assumed the helper's side effects rather than reading it. Caught by inspect critic 3. Fixed: capture the origin before, push NavLoc only if open_and_place_caret returned true (the jump_to_definition shape).

## DL-cluster-45ee75eae8a1
*topic: #83's first cut of marley_remote::ssh_command built the argv ["ssh", ("-p", port)?, dest] with NO guard on a destination that starts with `-`. A target like `-oProxyCommand=evil` parsed to SshTarge... · confidence: 0.500*

#83's first cut of marley_remote::ssh_command built the argv ["ssh", ("-p", port)?, dest] with NO guard on a destination that starts with `-`. A target like `-oProxyCommand=evil` parsed to SshTarget{host:"-oProxyCommand=evil"} and produced ["ssh","-oProxyCommand=evil"] — which real OpenSSH RE-PARSES AS AN OPTION (not a hostname), because ssh treats a leading-dash positional arg as a flag. Confirmed against OpenSSH_10.2p1: the dest was consumed as an option (ssh printed usage) and the ProxyCommand mechanism is real (a sentinel file was touched when a host arg was also present). This directly falsified the crate's own doc invariant ("a host or user string can never inject flags or commands"). NOT a one-shot RCE from today's exact single-arg output (when the sole dest is eaten as an option, no host remains → ssh exits at usage without connecting), but a BROKEN SECURITY INVARIANT + LATENT RCE: it goes live the instant a second positional/remote-command arg is added (the obvious next ssh-pane feature) — the CVE-2017-1000117 (git) / option-injection class. FIX (both, defense-in-depth): parse_ssh_target rejects a leading-`-` host OR user (→None); ssh_command inserts `--` before the destination so ssh stops parsing options (also defends a hand-constructed SshTarget that bypassed the parser). Both verified against real ssh.

## DL-cluster-461acecc6e32
*topic: #275 inspect F1 (HIGH, found independently by BOTH critics): the boot-restore path (EditorSurface::from_files, the #243 open-set restore) never seeded the new per-file disk snapshot — restored file... · confidence: 0.500*

#275 inspect F1 (HIGH, found independently by BOTH critics): the boot-restore path (EditorSurface::from_files, the #243 open-set restore) never seeded the new per-file disk snapshot — restored files hit the None→Noop table row at every choke, making the entire external-change feature silently OFF for the steady-state case (restart with the working set open) and letting ⌘S silently clobber agent writes, the exact guarantee the ticket exists for. CLASS: a new per-item field seeded on the INTERACTIVE creation path but forgotten on the RESTORE/deserialize path — restore is a second constructor. Fix: stat before read in the restore arm, snapshot threaded through a RestoredFile row type. Also the implement-phase launcher bug (render edge called active_project() before the launcher branch → panic-on-empty → the virgin-boot headless test wedged the suite via panic-masks-as-hang; the #234 invariant comment sat five lines below the insertion) — guarded on project_count()>0; the REAL regression pin needs activate_window (TestWindow::is_active is hardcoded false, so the edge is dead headless without it).

## DL-cluster-46f7a6059fda
*topic: #303 inspect C1-LOW (correctness critic): `delete_range_for(WholeLine, ...)` on a truly EMPTY buffer returned `Some(0..0)` (a zero-width range) instead of `None`, breaking the symmetry with the oth... · confidence: 0.500*

#303 inspect C1-LOW (correctness critic): `delete_range_for(WholeLine, ...)` on a truly EMPTY buffer returned `Some(0..0)` (a zero-width range) instead of `None`, breaking the symmetry with the other four ops (WordLeft/Right/ToLineStart/ToLineEnd all return None at their no-op edge). Because `apply_delete` opens an undo group directly (begin_undo_group → edit → end_undo_group, NOT the edit_ranges_restoring no-op guard), a zero-width edit would open an undo group over a no-op `edit(0..0,"")` — a dead ⌘Z step recorded in an empty editor (reachable: a new empty file, or a file whose content was fully deleted). Not a panic, not incorrect text, but a real asymmetry + a dead undo unit. Fixed by guarding the only-line branch: `(start < len).then(|| ...)` → None on an empty buffer. Missed by my own throwaway (I discarded the WholeLine-on-empty return with `let _ =` instead of asserting it); caught by the independent correctness critic — the value of a second lens even after a passing self-check.

## DL-cluster-47022d4671b7
*topic: #232 authored docs/marley_architecture/icon-audit.md (a catalogue with ~25 file:line refs) at DESIGN via a background Explore sweep, THEN the implement phase edited app.rs (adding a const + 3 Asset... · confidence: 0.500*

#232 authored docs/marley_architecture/icon-audit.md (a catalogue with ~25 file:line refs) at DESIGN via a background Explore sweep, THEN the implement phase edited app.rs (adding a const + 3 Assets arms + 2 svg blocks + 2 items_center), shifting every cited app.rs line by +7..+25. An inspect critic flagged the whole 'site' column as stale (MED). Non-app.rs refs stayed accurate (those files untouched). Fixed by regenerating all app.rs line numbers via a fresh grep (not hand-math) + a drift note. Root cause: a reference doc's line numbers were captured before same-ticket edits to the referenced file.

## DL-cluster-48ff876769a8
*topic: #401 inspect (correctness critic): reload_active_from_disk resets the buffer version epoch (BufferVersion::initial) and re-mints the nonce but cleared NO inlay state. InlayKey carries (uri, version... · confidence: 0.500*

#401 inspect (correctness critic): reload_active_from_disk resets the buffer version epoch (BufferVersion::initial) and re-mints the nonce but cleared NO inlay state. InlayKey carries (uri, version) only — no nonce — so on a never-edited file (version == initial both sides) a raced pre-reload inlay answer passes the apply guards numerically, projects onto the NEW text, and is cached wrong. Pre-#401 the served-check's forever-refetch bug accidentally replaced the wrong hints one round-trip later; the #401 fix removed that self-heal, so the wrong hints would have persisted until an edit/scroll/server-refresh. Pre-existing (#331-era) acceptance-of-raced-answer; surfaced because #401 deleted the accidental recovery. Fixed at the reset site: the reload now drops inlay_hints + inlay_request (beside the #328 git-marks invalidation), pinned by inlay_cache_dropped_on_disk_reload_headless.

## DL-cluster-4af238bf1324
*topic: #329: the pure `SelectionLadder` lives in a NEW file `selection_ladder.rs`. The `--diff` mutation gate feeds cargo-mutants `git diff HEAD -- crates`, and `git diff HEAD` EXCLUDES untracked files — ... · confidence: 0.500*

#329: the pure `SelectionLadder` lives in a NEW file `selection_ladder.rs`. The `--diff` mutation gate feeds cargo-mutants `git diff HEAD -- crates`, and `git diff HEAD` EXCLUDES untracked files — so a brand-new (never-`git add`ed) source file contributes ZERO lines to the diff, and ALL its mutants are silently skipped. gate:5 would print MSI 100% and GREEN without ever testing the new file's pure seam — a false green precisely on the code most in need of mutation coverage. Caught by cross-checking `cargo mutants --list --in-diff` (the new file's mutants were absent). Fixed with `git add -N <file>` (intent-to-add) so `git diff HEAD` includes it. The whole-workspace COVERAGE gate is unaffected (it's not diff-scoped), which is why the gap is easy to miss — coverage catches the file, mutation doesn't.

## DL-cluster-4bcc33481b63
*topic: #322 inspect (HIGH, correctness critic): the multi-file rename applier decided open-vs-closed by RAW PathBuf equality (has_path/locate_open_file: `f.view.path == fe.path`), but the two sides are ca... · confidence: 0.500*

#322 inspect (HIGH, correctness critic): the multi-file rename applier decided open-vs-closed by RAW PathBuf equality (has_path/locate_open_file: `f.view.path == fe.path`), but the two sides are canonicalized DIFFERENTLY. The edit target `fe.path` = path_from_file_uri(server_uri) is CANONICAL (the server only ever learns a path via uri_for → LspHost::absolute → canonicalize). The open file's `f.view.path` = root.join(rel) is NON-canonical (Project stores the root verbatim). Under a symlinked root — /tmp→/private/tmp on macOS (which the drive fixtures AND the live-drive probe project use), a symlinked $HOME, /Volumes automounts, an unresolved .. — the two differ, so an OPEN file reads as CLOSED and apply_one_file falls through to a blind fs::read/fs::write on disk. Clean buffer → the rename lands on disk, the buffer is untouched, one ⌘Z reverts nothing, a spurious 'Changed' banner fires, yet rename_summary reports success. DIRTY buffer → the server computed positions against the synced buffer text but they apply to the last-SAVED disk text → on-disk CORRUPTION while the buffer shows un-renamed text and the user is told it worked. Would have shipped and fired in the very live-drive fixture. Fix: a same_file(a,b) that compares CANONICAL forms (raw fallback when a side is absent), used by has_path + buffer_for_path_mut — the editor surface adopting the same keying discipline the LSP host already uses.

## DL-cluster-4d9f8dcd4a91
*topic: #312 jump_to_definition/nav_back called open_file_in_viewer and then IMMEDIATELY read active_editor() to place the caret — but open_file_in_viewer is `if let Some(..) = load_code_view_state(path)` ... · confidence: 0.500*

#312 jump_to_definition/nav_back called open_file_in_viewer and then IMMEDIATELY read active_editor() to place the caret — but open_file_in_viewer is `if let Some(..) = load_code_view_state(path)` with NO else, so it no-ops on a missing / non-file / >2MB / binary target and leaves the PREVIOUS file active. The target's (line, character) was then mapped against the ORIGIN's text and the origin's caret moved there, plus pending_center_row scrolled it — SILENTLY, because position_to_offset clamps rather than panicking. Reachable and common: rust-analyzer resolves definitions into >2MB generated/vendored sources (real >2MB .rs files confirmed in ~/.cargo/registry on this machine), where load_code_view_state even flashes "can't open …(>2MB)" WHILE the caret still moves. jump_to_definition also pushed the NavStack BEFORE the open, recording a return for a jump that never happened. Found by 3 of 5 inspect critics independently. FIX: open_and_place_caret verifies the landing (active_file().path == path) before touching the caret and returns false otherwise → flash, nothing moves; the NavStack push moved to after a confirmed landing.

## DL-cluster-50e1b782d35f
*topic: #400 design enumerated the cockpit open doors by grepping WRAPPER spellings (open_cockpit_section, SectionAction::OpenCockpit) and concluded "two doors". A third production caller — the "toggle-rig... · confidence: 0.500*

#400 design enumerated the cockpit open doors by grepping WRAPPER spellings (open_cockpit_section, SectionAction::OpenCockpit) and concluded "two doors". A third production caller — the "toggle-right-dock" verb (⌘⇧B) — called the callee (Project::open_or_switch_cockpit) directly and was missed; the signature change made it a compile error, so the compiler caught it at implement, but a semantic-only change (e.g. adding the acquire without changing arity) would have shipped an unbalanced view-count leak through that door. Same family as the #399 static-id census miss: a census built from a PARTIAL naming pattern reads as complete.

## DL-cluster-53356762fc3b
*topic: The #255 drag-select `dragging_selection` flag was cleared ONLY by a per-row `on_mouse_up`. In gpui, `on_mouse_up` is bounds-gated — it fires only when the button is released over the element's hit... · confidence: 0.500*

The #255 drag-select `dragging_selection` flag was cleared ONLY by a per-row `on_mouse_up`. In gpui, `on_mouse_up` is bounds-gated — it fires only when the button is released over the element's hitbox. So releasing a drag OFF any editor row (the empty area below the text, the `py_1` padding, the #246 terminal split pane, or outside the window — a very common drag gesture) never fires any row's on_mouse_up → the flag stays `true`. Then every subsequent plain HOVER move over an editor row passes the `if !dragging_selection` gate and moves the caret → the exact "every mouse-over reshuffles the caret" bug the gate was meant to prevent, reintroduced through a leaked flag. Non-corrupting (offsets stay valid) but very user-visible and easily triggered; a scripted drag that happens to release over text would not catch it. FIX: a self-heal in on_mouse_move — `if event.pressed_button != Some(MouseButton::Left) { dragging_selection = false; return; }` (a button-less move means the drag ended elsewhere → clear the flag without moving the caret). Caught by an adversarial inspect critic that read the gpui-0.2.2 div.rs bounds-gating of on_mouse_up.

## DL-cluster-53c82c9ea773
*topic: #321's three headless drives waited with `vcx.run_until_parked()` alone. That does not advance gpui's mock clock: `TestDispatcher::run_until_parked` is `while self.tick(false) {}`, and `tick` only ... · confidence: 0.500*

#321's three headless drives waited with `vcx.run_until_parked()` alone. That does not advance gpui's mock clock: `TestDispatcher::run_until_parked` is `while self.tick(false) {}`, and `tick` only promotes a delayed task once `state.time` reaches its deadline — and `state.time` moves ONLY via `advance_clock`/`advance_clock_to_next_delayed`. gpui's own docstring says so ("in tests, move time forward. This does not run any tasks, but does make timers ready.").

The pump is a `background_executor().timer(PUMP_INTERVAL_MS = 16ms)` loop, and since #321 the pump is the ONLY thing that creates an LSP host. So the pump body never ran in any of the drives, and `lsp_hosts` stayed empty.

The dangerous half is not the failing asserts — it is that the NEGATIVE arms passed. `spawn_gate_rejects_headless` (a) and (b) assert `!lsp_host_exists_for_test(root)`, which was trivially true because the pump had never ticked. They would have gone green while proving nothing about the gate they exist to test, and a future regression that made the gate spawn indiscriminately would still have passed them.

The repo already had the correct idiom in 5 places (`executor().advance_clock(Duration::from_millis(40))` followed by `run_until_parked()`), which I did not reuse. Fixed by adding a shared `tick_pump(cx)` helper doing exactly that, documenting WHY `run_until_parked` alone is insufficient, and calling it in every arm including the negatives.

Found by an inspect critic, which cited the gpui dispatcher mechanism line by line. Related: the same drives also spawned a real rust-analyzer (BF-claude-headless-test-satisfied-the-spawn-gate-and-launched-a-real-server-001) — two independent defects in one test suite, both invisible to a green compile.

## DL-cluster-5545e889b8a9
*topic: #266 P4 gate --diff went RED on clippy's deny-by-default reversed_empty_ranges: two DELIBERATE inverted-range test literals (styled_slices' `Some(7..3)` inverted-selection case and cols_to_bytes' e... · confidence: 0.500*

#266 P4 gate --diff went RED on clippy's deny-by-default reversed_empty_ranges: two DELIBERATE inverted-range test literals (styled_slices' `Some(7..3)` inverted-selection case and cols_to_bytes' expected `10..1` inverted output) read to clippy as empty-iteration bugs. Also rustfmt RED on the fresh headless-test block (fmt not run before the gate). Fixed at source, no suppression: the inverted ranges are test DATA, so they are spelled as `std::ops::Range { start, end }` struct literals, which the lint correctly ignores; re-run GREEN 15/15.

## DL-cluster-566a0ee4edfe
*topic: #302 go-to-line: goto_restore (the Esc path) placed the origin caret via set_active_selections WITHOUT first calling clear_marked, asymmetric with goto_commit (which does). clear_marked's contract ... · confidence: 0.500*

#302 go-to-line: goto_restore (the Esc path) placed the origin caret via set_active_selections WITHOUT first calling clear_marked, asymmetric with goto_commit (which does). clear_marked's contract (editor_surface.rs) is "every non-IME edit/interaction path calls this — a stale marked span after an out-of-band edit would misdirect the next IME replace." A caret placement IS such an out-of-band edit. No user-visible reproduction (requires being mid-IME-composition when the Esc control chord fires, which the OS IME layer usually commits/cancels first), so LOW — a latent robustness gap, not a confirmed bug. Caught by an inspect critic noticing the goto_commit/goto_restore asymmetry. Fixed by adding s.clear_marked() before the placement in goto_restore, and reusing the shipped set_single_caret helper in both (a cleanup the same critic flagged).

## DL-cluster-567deb8efd56
*topic: #203: the clear-on-view step (`view.tab_flashes.remove(&seen)`) mutated render-affecting state in the gpui pump WITHOUT setting `dirty = true`. The pump only calls `cx.notify()` (repaint) when `dir... · confidence: 0.500*

#203: the clear-on-view step (`view.tab_flashes.remove(&seen)`) mutated render-affecting state in the gpui pump WITHOUT setting `dirty = true`. The pump only calls `cx.notify()` (repaint) when `dirty`; the SET path rode the finishing command's pump events (dirty), but the CLEAR path did not — so on an otherwise-idle frame the completion ● badge lingered on the just-viewed tab until an unrelated repaint (a keystroke / PTY output / hover). Asymmetric and directly undercut the feature's "cleared once you view the tab" promise. Caught by the inspect shim critic (my self-review missed it). Fixed: `if view.tab_flashes.remove(&seen).is_some() { dirty = true; }` — mirroring the adjacent `status_flash.tick()` which already sets dirty on a state change.

## DL-cluster-5856ce718721
*topic: #250 faithful renderer: the editor render was changed to window over `buffer.len_lines()` (ropey), but the wheel/scroll handler still clamped `cv.scroll` via `scroll_code(.., cv.lines.len())` (the ... · confidence: 0.500*

#250 faithful renderer: the editor render was changed to window over `buffer.len_lines()` (ropey), but the wheel/scroll handler still clamped `cv.scroll` via `scroll_code(.., cv.lines.len())` (the split('\n') count frozen at open). Equal for \n/CRLF files (cosmetic), but ropey's unicode_lines default also breaks on bare CR/FF/VT/NEL/LS/PS → the bottom lines become unscrollable, AND once #251 edits the buffer the stale cv.lines count breaks the clamp for every edited file (latent MEDIUM). Caught by an inspect critic (not shipped). Fixed: the handler now clamps against active_buffer().len_lines() (the same source the render uses), falling back to cv.lines.len() for a bufferless read-only pane.

## DL-cluster-59304aee4993
*topic: marley_terminal inspect: `(hi << 4) | lo` in the hex/C-quote decoders (hi,lo each 0..=15 from hex_val) — the nibbles are DISJOINT (hi<<4 = xxxx0000, lo = 0000yyyy), so `(hi<<4) | lo == (hi<<4) ^ lo... · confidence: 0.500*

marley_terminal inspect: `(hi << 4) | lo` in the hex/C-quote decoders (hi,lo each 0..=15 from hex_val) — the nibbles are DISJOINT (hi<<4 = xxxx0000, lo = 0000yyyy), so `(hi<<4) | lo == (hi<<4) ^ lo == (hi<<4) + lo` for EVERY reachable input. cargo-mutants' `replace | with ^` is therefore an EQUIVALENT mutant: it compiles, all tests pass, it reports MISSED → MSI<100, unfixable by any test. Two sites (hex_decode + c_unescape). Fix WITHOUT suppression: rewrite `|` → `+` (identical result for disjoint nibbles — no carry) so the operator mutants (`+→-`, `+→*`) become KILLABLE by the exact-decoded-byte assertions. (Alternative was an exclude_re skip, but the `+` rewrite reaches true MSI 100 with zero suppression.)

## DL-cluster-59c01b2a5bf0
*topic: Marley #297. Backspace at N cursors turns each bare caret into a one-char CONSUMING range so it can delete backwards. Those internal ranges were handed to edit_at_selections, which recorded them as... · confidence: 0.500*

Marley #297. Backspace at N cursors turns each bare caret into a one-char CONSUMING range so it can delete backwards. Those internal ranges were handed to edit_at_selections, which recorded them as the undo group's sel_before — so ⌘Z restored the user's TEXT but gave them back SELECTIONS they never made (the consuming ranges), and the very next keystroke REPLACED those selections, eating the text the undo had just restored. Reproduced: "abcdef" with carets at 2 and 5 → ⌫ → ⌘Z → type "X" → "aXcdXf" instead of "abXcdeXf". The single-cursor path was correct (no undo group is opened for one member), so this was a 1-vs-N divergence in a DATA-LOSING direction — the worst kind. Two independent critics found it. FIX: backspace moved into the crate as Buffer::backspace_at_selections, built on a new edit_ranges_restoring(restore, targets, f, origin) that names BOTH halves explicitly — the ranges to edit, and the cursors to restore. A first attempt instead took the buffer's CURRENT selection as the snapshot; a critic's regression test caught that it silently restored one cursor when the caller edited at two. The seam now states the restore set rather than guessing it.

## DL-cluster-59f2e36f67f1
*topic: #361 Phase 3 shipped two `#[gpui::test]` headless drives (font_family_proportional_falls_back_to_builtin_headless / font_family_monospace_applies_silently_headless) that assert the RESOLVABLE-font ... · confidence: 0.500*

#361 Phase 3 shipped two `#[gpui::test]` headless drives (font_family_proportional_falls_back_to_builtin_headless / font_family_monospace_applies_silently_headless) that assert the RESOLVABLE-font monospace path (Helvetica→fallback+"not monospace" flash; Monaco→applied+silent). They CANNOT pass: `#[gpui::test]` uses gpui's NoopTextSystem, under which `all_font_names()` reports nothing resolvable, so a real family reads UNRESOLVABLE → the boot wire's `!resolves` short-circuit fires and `font_is_monospace` is never called → the not-monospace arm is unreachable headless (and Noop's synthetic advance makes every glyph equal-width, so it couldn't distinguish mono from proportional even if reached). The #344 garbage-drive comment already documented exactly this ("The RESOLVABLE case is not headless-testable here (Noop resolves nothing) — the pure `resolve_font_family(resolves=true)` unit covers it"). Caught by inspect Critic 2 (HIGH). Root cause: at design I read the #344 headless drive's CODE but not its COMMENT's rationale, so I assumed real macOS fonts (Helvetica/Monaco) would behave under the headless harness — they don't; `#[gpui::test]` is NoopTextSystem, not CoreText. Fix: removed both drives; the not-monospace DECISION is proven by the pure units (settings.rs, cov-included); the app-side metric read (font_is_monospace) is cov-excluded + headed-only, exactly like #344's font_resolves.

## DL-cluster-5a9b36c10917
*topic: TICKET-021 (manager.rs is_syncable): a fn that returns a constant `false` but had a preceding no-op statement — `pub fn is_syncable<S: Setting>(&self) -> bool { let _ = S::storage_key(); false }` (... · confidence: 0.500*

TICKET-021 (manager.rs is_syncable): a fn that returns a constant `false` but had a preceding no-op statement — `pub fn is_syncable<S: Setting>(&self) -> bool { let _ = S::storage_key(); false }` (the `let _` was added to reference the unused generic S out of a clippy unused-type-param worry) — made cargo-mutants emit "replace SettingsManager::is_syncable -> bool with false", which is an EQUIVALENT mutant: the fn IS already false, so the mutation is behaviourally identical → cargo-mutants reports it MISSED → MSI 99.x < 100. Fixed by a BARE `{ false }` body: cargo-mutants recognises "replace-with-false" as identical-to-the-original and does NOT generate it (only the killable "replace with true" remains, caught by r18_is_syncable_false). Clippy's `extra_unused_type_parameters` is pedantic (not in the gate's `-D warnings`), so the unused S is fine without the dead statement.

## DL-cluster-5c33ed3f85dc
*topic: #335 inspect F1 (HIGH, proven live by a critic): redirecting cargo-mutants' TMPDIR to a scratch INSIDE the repo (target/mutants-scratch) made every test tempdir repo-ancestored — discover_no_git_fa... · confidence: 0.500*

#335 inspect F1 (HIGH, proven live by a critic): redirecting cargo-mutants' TMPDIR to a scratch INSIDE the repo (target/mutants-scratch) made every test tempdir repo-ancestored — discover_no_git_falls_back asserts its tempdir has NO .git ancestor, so the baseline failed inside every mutant run → cargo-mutants exit 4 → gate:5 RED on the first real mutation run, bricking /commit. The implementer's reasoning had checked COPY-recursion ("target/ is never copied") but missed the ANCESTRY consequence: relocating tmp into a repo changes what every tempdir's parent chain contains. Fixed: the scratch lives under the SYSTEM tmp in a named dir (${TMPDIR}/marley-mutants-scratch/$$) — still trap-cleanable by exact name, still outside the copy set, and test tempdirs keep their system-tmp ancestry. Verified by re-running the exact failing probe (passes) + a real bounded cargo-mutants run through the redirect (ok Unmutated baseline).

## DL-cluster-5c477ae63f1d
*topic: #362's Phase-3 headless drive (auto_close_suppresses_quote_in_type_parameters_headless) was VACUOUS — it passed on the PRE-fix code too, testing nothing. Fixture `fn f<T: >() {}`, caret at byte 8 =... · confidence: 0.500*

#362's Phase-3 headless drive (auto_close_suppresses_quote_in_type_parameters_headless) was VACUOUS — it passed on the PRE-fix code too, testing nothing. Fixture `fn f<T: >() {}`, caret at byte 8 = the position of `>`, so `next = '>'`. `pair_action` only reaches InsertPair when `opens_here(next)` is true, and `opens_here('>')` = false (`>` is neither a closer in PAIRS nor whitespace). So `'` inserts BARE regardless of context — the `>` blocks the pair, NOT the #362 LifetimeBound. Reverting the entire #362 app-closure branch leaves the drive green. Caught by inspect Critic 2 (HIGH); my own inspection had confirmed the enclosing-fn mutants::skip but did NOT trace opens_here on the fixture. Root cause: I chose the incremental-typing position `fn f<T: |>` intuitively, but that exact position is where the trailing `>` already suppresses via opens_here — the one place #362 is NOT needed. Fix: a DOUBLE-space fixture `fn f<T:  >() {}` with the caret between the two spaces (byte 8) → `next = ' '` (whitespace → opens_here true) so a Code context WOULD pair `''`; only LifetimeBound suppresses it → bare `'`. Asserted `!bound.contains("''")` + the exact bare-`'` string. Added the double-space byte to the probe's true-unit. Verified non-vacuous by tracing the pre-fix path (Code → InsertPair → `''` → the assert fails). LESSON: a headless drive for an auto-close/suppression feature must verify the BASELINE would produce the OTHER outcome — trace opens_here/next at the fixture caret, or the test can pass for the wrong reason (a different guard masks the one under test).

## DL-cluster-5cb43d334588
*topic: enforce-warp-reference.sh checked for the required `## Reference (§20)` section with `grep -qE '^## Reference'` + `awk /^## Reference/` — both PREFIX matches. So a sibling heading `## References` (... · confidence: 0.500*

enforce-warp-reference.sh checked for the required `## Reference (§20)` section with `grep -qE '^## Reference'` + `awk /^## Reference/` — both PREFIX matches. So a sibling heading `## References` (plural) or `## Reference Material` with any content satisfied the gate even when the mandated `## Reference (§20)` was entirely ABSENT → false-ALLOW (a spec commits with no real Reference section). Bounded: only slips when the exact `## Reference (§20)` heading is missing (if it exists but is empty, the awk stops at the next `## ` so a following plural heading doesn't rescue it). Caught by the inspect critic. Fix: anchor both the grep presence check + the awk section-extractor to the exact `^## Reference \(§20\)`. Class: a prefix-anchored heading match (`^## X`) false-matches sibling headings sharing the prefix (`## Xs`, `## X Material`) — anchor to the full heading.

## DL-cluster-5f9fce8d3435
*topic: #378 inspect F2 (MED): delivery echoes observed while the send's receipt was in flight (phase=Sending) were DROPPED on the theory "the next echo heals it (the monotone skip is legal)" — but dispatc... · confidence: 0.500*

#378 inspect F2 (MED): delivery echoes observed while the send's receipt was in flight (phase=Sending) were DROPPED on the theory "the next echo heals it (the monotone skip is legal)" — but dispatch-started is the TERMINAL echo kind, so a start observed mid-flight had no later healer: the chip stuck at "deposited" forever while the seat ran the brief. The heals-later justification was factually incomplete for the last event in a sequence. Fixed by BUFFERING (max-join into DispatchRecord.pending) while Sending and consuming the buffer at receipt time (Deposited.observe(pending)) — nothing renders pre-receipt, so the no-forgery doctrine holds. Sibling F1 (both critics): the crate-side echo queue was an unbounded Vec whose only drain parks in the launcher state — became a per-seat max-join map, bounded by fleet size, lossless because the consumer folds the same monotone max.

## DL-cluster-6125e397f823
*topic: #236 stored the rail collapse state as a `collapsed_projects: HashSet<usize>` of raw project indices. `close_project` does `self.projects.remove(idx)` (a shifting Vec removal), but `close_project_a... · confidence: 0.500*

#236 stored the rail collapse state as a `collapsed_projects: HashSet<usize>` of raw project indices. `close_project` does `self.projects.remove(idx)` (a shifting Vec removal), but `close_project_at` did NOT remap the set (it remaps the active pointer via adjust_active + resets renaming_tab, with the comment 'project indices shift too', but missed collapsed_projects). So collapsing project B(idx 1) then closing A(idx 0) left collapsed={1} while B shifted to idx 0 → project C (now idx 1) rendered collapsed instead of B — the collapse ALIASED the wrong project. Transient view state, self-corrects on the next click, no panic → MEDIUM. An inspect critic caught it (the pure rail_rows READS harmlessly via contains(), which masked that the WRITE side aliases a live project). Fixed: remap collapsed_projects on close (drop idx, decrement indices > idx), mirroring the #177 renaming_tab index-shift guard.

## DL-cluster-616c49ef61ad
*topic: #284: in save_active, when the armed 2nd-press WRITE has a racing agent write land in the write gap (len mismatch → racing), the code re-flagged the Changed banner + refreshed conflict_observed but... · confidence: 0.500*

#284: in save_active, when the armed 2nd-press WRITE has a racing agent write land in the write gap (len mismatch → racing), the code re-flagged the Changed banner + refreshed conflict_observed but LEFT armed_at=Some(W1) — the old arm outlived its consuming write. Harmless in practice (the next press re-warns since the racing W3 ≠ W1) but a latent hole if the disk transiently returned to exactly W1. Fixed in inspect by disarming (disarm_active_save) in the racing branch too — a successful write always consumes the arm (the non-racing branch already disarmed via set_active_conflict(None)). Class: an acknowledgment/license consumed by an action must be cleared on EVERY exit path of that action, not just the common one.

## DL-cluster-67df025d4c09
*topic: #46 promised re-run via BOTH a ↻ click AND cmd-R (in SPEC-app-shell R50, the pipeline REQ-004, the requirement table, AND the ticket TITLE "click / cmd-R"), but the implement phase wired ONLY the ↻... · confidence: 0.500*

#46 promised re-run via BOTH a ↻ click AND cmd-R (in SPEC-app-shell R50, the pipeline REQ-004, the requirement table, AND the ticket TITLE "click / cmd-R"), but the implement phase wired ONLY the ↻ click — no keymap binding, no dispatch arm, no most-recent-finished-block resolver. The gap was invisible to every automated gate: the whole app.rs render/dispatch shim is #[cfg_attr(test, mutants::skip)] + coverage-excluded, and REQ-004's verification is a masked chad-visual, so cov/MSI/tests all stayed GREEN on the pure rerun_command alone while a titled feature was absent. Caught only by the inspect critic cross-reading the spec's promises against the keymap's actual bindings (grep: no "r" chord, no rerun dispatch arm, no resolver). Fix: implemented cmd-R fully — a pure BlockList::last_rerunnable() (tested), a cmd-r→"rerun-last" keybinding (+ its action_for assertion), and a #40-guarded dispatch arm. Lesson: when a ticket names TWO trigger surfaces for one action (a click AND a keyboard shortcut), and only the pure decision fn is gate-covered, the second (shim-only) trigger can silently not-ship — the masked shim hides it. An inspect step that diffs the spec's named triggers against the actual keymap/dispatch wiring is the only thing that catches a missing-but-specified shim trigger.

## DL-cluster-6a115b17b9d6
*topic: #329 inspect (C1-2): `enclosing_ranges` has a `let Some(node) = ...named_descendant_for_byte_range(s,e) else { return Vec::new() }` totality arm. A probe showed that for ANY valid `start ≤ end` ran... · confidence: 0.500*

#329 inspect (C1-2): `enclosing_ranges` has a `let Some(node) = ...named_descendant_for_byte_range(s,e) else { return Vec::new() }` totality arm. A probe showed that for ANY valid `start ≤ end` range (even wildly out-of-bounds like 100..200 on a 9-byte file) tree-sitter returns at least `Some(source_file)` — it clamps to the root, never None. The None arm is reachable ONLY by a REVERSED range (`start > end`), which the app never produces (selections have start ≤ end). Under the repo's whole-workspace `--fail-under-lines 100`, that arm would sit UNCOVERED and turn gate:4 RED at validate, with a non-obvious trigger. The code is CORRECT (Option totality per §14); the defect is a missing degenerate-input test. Caught at inspect (not shipped); the validate plan now owes `enclosing_ranges(&s, 5..2) == []`.

## DL-cluster-6b024c17fe6c
*topic: #340 bracket-match: `delims_at` (syntax/src/lib.rs) matched the delimiter pair on `child.kind()` strings alone. For an unclosed `(` (`fn f( {}`), tree-sitter's error recovery inserts a zero-width M... · confidence: 0.500*

#340 bracket-match: `delims_at` (syntax/src/lib.rs) matched the delimiter pair on `child.kind()` strings alone. For an unclosed `(` (`fn f( {}`), tree-sitter's error recovery inserts a zero-width MISSING `)` node whose `kind()` IS `")"` — so `is_delim_pair("(", ")")` passed and the walk returned `Some((4..5, 5..5))`: a lone `(` tinted (0.10) and ⌘⇧\ jumped the caret onto whitespace at the phantom position. Contradicted the fn's "returns None" contract and the spec's explicit cut of unmatched-bracket error tinting. Root cause: a `kind()`-string match is not sufficient — tree-sitter MISSING nodes have a REAL `kind()` but zero width. Fixed with one guard: `if first.is_missing() || last.is_missing() { return None }`. Probe-confirmed the behavior flipped (`Some((4..5,5..5))` → `None`) with real pairs unaffected.

## DL-cluster-6b2496c15e8f
*topic: #307's Tab arm keeps a LONE bare-caret pad ungrouped (grouping it would split one ⌘Z into two). The decision is correct, but the comment, the Phase-1 finding, and the spec's D3 all justified it wit... · confidence: 0.500*

#307's Tab arm keeps a LONE bare-caret pad ungrouped (grouping it would split one ⌘Z into two). The decision is correct, but the comment, the Phase-1 finding, and the spec's D3 all justified it with the WRONG mechanism: "begin_undo_group pins cursor_anchored: false, and coalesces_into requires it on BOTH sides, so grouping would stop the pad coalescing with following typing." That chain never runs on the path it explains. `coalesces_into` is called from exactly one place — `undo::end_group` — so it governs GROUP-into-GROUP absorption and is only reachable when a group was opened. A following keystroke at ONE cursor opens no group (`Buffer::edit_ranges_restoring_placing` takes its single-member ungrouped fast path), so the typed char reaches `UndoHistory::record`, whose coalesce guard is `group.sel_before.is_none() && group.records.len() == 1`; `cursor_anchored` is never consulted there. Grouping the pad sets `sel_before: Some(..)` and THAT is what blocks the coalesce. Two independent inspect critics found this; one proved it with a counterfactual (ungrouped → one ⌘Z yields "a"; grouped → one ⌘Z yields "a   "). The shipped precedent states it correctly at buffer.rs's single-member fast path, and I cited the wrong sibling. It matters beyond pedantry: if someone later "cleans up" begin_undo_group to pass cursor_anchored: true, the comment reads as a licence to group the lone pad and ⌘Z granularity silently regresses. This was the THIRD "right decision, wrong reason" in one ticket.

## DL-cluster-6b871a8d2362
*topic: #361's --diff gate went RED at gate:5 (mutation): 3 survivors, all in app.rs::font_is_monospace (replace→true, replace→false, delete match arm). Root cause: coverage-exclusion (the llvm-cov `--igno... · confidence: 0.500*

#361's --diff gate went RED at gate:5 (mutation): 3 survivors, all in app.rs::font_is_monospace (replace→true, replace→false, delete match arm). Root cause: coverage-exclusion (the llvm-cov `--ignore-filename-regex` that skips app.rs for gate:4) is a SEPARATE mechanism from mutation-exclusion — cargo-mutants mutates every fn in the diff regardless of the cov ignore-regex, unless the fn carries `#[mutants::skip]`. font_is_monospace is an app-side gpui metric-read shim with no unit-testable seam (the decision it delegates to, is_monospace_advance, IS unit-tested), so it needs the explicit skip — exactly like its sibling font_resolves, which already carried `#[cfg_attr(test, mutants::skip)]`. I added the cov-exclusion mentally (app.rs is in the regex) but forgot the fn-level mutants::skip. Fixed by adding `#[cfg_attr(test, mutants::skip)]` to font_is_monospace; verified via `cargo mutants --list` that both probes show no mutants and the settings.rs decision mutants remain (all killed); re-run → GATE GREEN. Lesson: an integration-only shim needs BOTH the file-level cov exclusion AND the fn-level mutants::skip.

## DL-cluster-6de4b30f24e9
*topic: #277 inspect C-F2 (probe-verified): all three editor click sites rounded x/cell_w to an INTEGER column before the nearest-boundary scan — on a 2-cell wide glyph (boundaries 0 and 2) a click at 0.5 ... · confidence: 0.500*

#277 inspect C-F2 (probe-verified): all three editor click sites rounded x/cell_w to an INTEGER column before the nearest-boundary scan — on a 2-cell wide glyph (boundaries 0 and 2) a click at 0.5 cells rounds to col 1, an exact tie resolved LATER, so the caret flip point sat at 25% of the glyph instead of its midpoint. Quantize-then-scan discards the half-cell the tie rule needs. Fix: keep the click column in the FLOAT domain until the final compare — offset_of_col_f(f32) with un-quantized distances (same later-wins tie), offset_for_click takes f32, click sites pass rel/cell_w unrounded. The integer wrapper went dead in the lib target and was deleted (no dead delegates), tests ported.

## DL-cluster-6f4687f342f5
*topic: #396 gate:4 first run RED with ONE missed line: content.rs kind()'s Content::Terminal arm. Its only caller was tests/integration.rs — the gate's llvm-cov run counts in-lib lanes (unit + cfg(test) h... · confidence: 0.500*

#396 gate:4 first run RED with ONE missed line: content.rs kind()'s Content::Terminal arm. Its only caller was tests/integration.rs — the gate's llvm-cov run counts in-lib lanes (unit + cfg(test) headless drives) but NOT the integration test binary, so an arm proven exclusively there reads as uncovered. Fix at source: assert the classification through the in-lib seeded-restore headless drive (view.content.get(cid).map(Content::kind) == Some(Terminal)).

## DL-cluster-70f7e7ab35d7
*topic: #325 ⌘T workspace-symbol picker capped its DISPLAY at 64 rows but left the stored results uncapped, and move_down + Enter operated on results.len(). So a query returning >64 (the common case — open... · confidence: 0.500*

#325 ⌘T workspace-symbol picker capped its DISPLAY at 64 rows but left the stored results uncapped, and move_down + Enter operated on results.len(). So a query returning >64 (the common case — opening the picker parks an empty query → the whole workspace) let ↓ walk the selection past row 63 (frozen highlight, invisible cursor) and Enter jump to results[selected] — a symbol never shown or highlighted. Caught by inspect (2 critics). Fix: a shared MAX_SYMBOL_ROWS const; clamp move_down + Enter to cap_with_tail(len, MAX).0; the overlay uses the same const. The class: a render-only cap must also bound navigation/selection.

## DL-cluster-72f10ec090e4
*topic: marley_editor inspect: movement.rs move_word_left/right opened with a leading `let mut i = off.as_usize().min(len)` defensive input-clamp. Because the caller-contract guarantees an in-range offset ... · confidence: 0.500*

marley_editor inspect: movement.rs move_word_left/right opened with a leading `let mut i = off.as_usize().min(len)` defensive input-clamp. Because the caller-contract guarantees an in-range offset (set_selection clamps; every movement fn returns in-range for in-range input), that `.min(len)` is IDENTITY on every contract-valid input — i.e. DEAD on the only tested path. cargo-mutants' `remove .min` / `min→max` mutant is therefore EQUIVALENT/unkillable unless a test passes an OUT-OF-RANGE offset (which the in-range contract says never happens), so it would silently block MSI 100 (or force a misuse-path test). Compounding it, the module doc overpromised "Every function returns a CharOffset in [0,len_chars]" while the sibling move_char_left returned an out-of-range value and move_line_* PANIC on an out-of-range input — a doc-vs-code mismatch + inter-sibling inconsistency. Not a runtime bug (correct for all in-range inputs). Caught by an inspect critic tracing out-of-range behavior across the 6 movement siblings + a buffer/types critic enumerating cargo-mutants viability. Fixed: removed the dead `.min(len)` (move_word_* now assume in-range like the other four fns), and softened the module doc to state the in-range caller-contract explicitly (consistent with Buffer::edit's §14 out-of-range = caller-contract-violation model).

## DL-cluster-73e772a8888c
*topic: #328: a design note that CITES the reference app ("git marks live in their own store, never on OpenFile — Zed's decoupling warning adopted") was copied VERBATIM from the spec into an app.rs code co... · confidence: 0.500*

#328: a design note that CITES the reference app ("git marks live in their own store, never on OpenFile — Zed's decoupling warning adopted") was copied VERBATIM from the spec into an app.rs code comment. gate:14 brand-scrub greps `warp|zed` (case-insensitive, word-boundary) across crates/**/*.rs and would have FAILED the commit. The spec (docs/) may cite Zed/Warp — docs aren't scanned — but a crates/ source comment must not contain the brand word. This is the exact #327 lesson (a "Zed chord" comment failed gate:14) and it slipped again despite the recorded prevention rule. Caught by inspect critic 3 (it ran the gate command live). Fixed: reworded to "git state lives in its own store, not on the file entry".

## DL-cluster-779729da4827
*topic: #298: ⌘D added a cursor the user could not see, and the viewport never moved. `follow_editor_caret()` reads `active_caret()` → `primary().head()` → member 0 — the TOPMOST cursor — and an ADDITIVE g... · confidence: 0.500*

#298: ⌘D added a cursor the user could not see, and the viewport never moved. `follow_editor_caret()` reads `active_caret()` → `primary().head()` → member 0 — the TOPMOST cursor — and an ADDITIVE gesture NEVER moves member 0 (from_selections SORTS, so a downward add appends BELOW the primary). So ⌘D scrolled to a row already on screen; the new cursor 300 rows down stayed invisible; the user saw nothing happen, pressed ⌘D again, accumulated off-screen cursors, and then typed into text they had never looked at. A REGRESSION against #272, whose ⌘D set a SINGLE selection — the new match WAS the primary, so following it worked. Making the set multi-member silently broke that guarantee while the one-line call site looked unchanged. Mirror bug: ⌘⇧L makes the primary the FIRST match in the file, so it YANKED the viewport to the top of the document, away from the user's work. AND: the comment I wrote directly above the call ("#270: a wrapping ⌘D can land the new cursor off-screen — follow it") asserts the behavior the code does NOT have — it names the ONE case (the wrap) that accidentally works, because there the new cursor happens to BECOME member 0. Third occurrence of the write-a-comment-then-violate-it class in three consecutive tickets. FIX: a pure `added_member(before, after)` seam; ⌘D scrolls to the cursor it actually created; ⌘⇧L does not scroll at all.

## DL-cluster-7b191055d4e8
*topic: #356: CodeViewState::sync_lines_from memoized its read-only `lines` on `buffer.version()` ALONE. reload_active swaps in a FRESH Buffer::from_text whose version restarts at 0 (BufferVersion::initial... · confidence: 0.500*

#356: CodeViewState::sync_lines_from memoized its read-only `lines` on `buffer.version()` ALONE. reload_active swaps in a FRESH Buffer::from_text whose version restarts at 0 (BufferVersion::initial), without resetting the view memo — so an UNEDITED pane (buffer at v0, lines_version Some(0)) reloaded to new text (also v0) would see Some(0)==Some(0) and no-op, rendering the PRE-reload lines: the exact "changes vanish" symptom the ticket set out to kill, reintroduced in the agent/disk-reload path. This is the same (nonce, version) trap #268/#273 already documented (a fresh buffer restarts at 0, so version alone cannot tell a reopened buffer from the cached one — which is why reload_active re-mints the nonce). Caught at Phase-3.5 inspect. Fixed by `f.view.lines_version = None` in reload_active (beside the nonce re-mint), forcing the next sync to rebuild.

## DL-cluster-7ebf9131b711
*topic: #44 cmd-C: content_row_texts made command-HEADER rows copyable (block.command is a content row), and selected_text slices every row in the selection incl. headers — but #43's highlight painted ONLY... · confidence: 0.500*

#44 cmd-C: content_row_texts made command-HEADER rows copyable (block.command is a content row), and selected_text slices every row in the selection incl. headers — but #43's highlight painted ONLY output rows (the header div had no selection_bg, just a hover tint). So a drag spanning from one block's output into the next block copied the intervening COMMAND text while that row showed no highlight — copy became a SUPERSET of the highlight, violating the spec's stated invariant "selected_text reuses row_selection so copy matches the highlight exactly" (D1/REQ-002). The pure fns were correct; the SHIM render was inconsistent (highlight incomplete). Caught by the inspect critic cross-reading the render (output-only tint) against content_row_texts (headers copyable). Fix: tint the command-header row via row_selection too (mirror the output branch) so highlight == copy — the right direction since Warp includes commands in a multi-block copy. Lesson: when a copy/serialize surface and a visual-highlight surface are meant to be "the same selection", an asymmetry in WHICH rows each covers is invisible to per-fn unit tests + MSI (both sides pass in isolation) — the invariant only breaks at the shim seam; verify the two surfaces enumerate the SAME row set.

## DL-cluster-80aa194e5535
*topic: Validate-phase gate:5 (mutation) surfaced 3 issues in marley_visual_harness that the headless test suite + nextest had not caught: (1) ENV RACE — capture.rs tests set_var/remove_var on MARLEY_VISUA... · confidence: 0.500*

Validate-phase gate:5 (mutation) surfaced 3 issues in marley_visual_harness that the headless test suite + nextest had not caught: (1) ENV RACE — capture.rs tests set_var/remove_var on MARLEY_VISUAL_APPROVE; under cargo-mutants' THREADED `cargo test` (not nextest's per-process isolation) they raced a concurrent APPROVE-unset test, so the *unmutated baseline* failed and the whole mutation run aborted ("1 Failure"). Fixed by `#[serial]` on all APPROVE-sensitive tests + an RAII ApproveGuard (Drop unsets even through a should_panic unwind). The established marley_core/marley_command lesson; nextest masked it. (2) EQUIVALENT LOOP-BOUND MUTANT — diff_image's `for px in 0..(capture.w * capture.h)`; the existing test used a 5×1 fixture where 5*1 == 5/1, so the `*`→`/` mutant produced the SAME loop count → observationally equivalent → survived. Fixed by adding a 1×2 fixture (w*h=2 ≠ w/h=0) asserting the second row. (3) CHILD-PROCESS BIN — the runner bin's main() executes only in a spawned child (the #[ignore] headed self-test spawns it), so no in-process test kills the "replace main with ()" mutant; fixed with #[cfg_attr(test, mutants::skip)] + the rust_cov coverage-exclude. All three only appeared in the FULL workspace mutation run, after coverage was already 100%.

## DL-cluster-81acc3c2a7c1
*topic: The #291 LIVE DRIVE surfaced a #289-era scoping limitation invisible to unit tests: `open_file_diagnostic_rows` reads only `workspace()`'s terminal grid, which via `terminal_grid_index()` is the AC... · confidence: 0.500*

The #291 LIVE DRIVE surfaced a #289-era scoping limitation invisible to unit tests: `open_file_diagnostic_rows` reads only `workspace()`'s terminal grid, which via `terminal_grid_index()` is the ACTIVE tab's grid (if a terminal) else the FIRST terminal tab's — so with the editor active + multiple terminal tabs, a failed block in a non-first terminal tab is never scanned and the gutter stays dark. Pure tests (trace_diagnostic_rows) + #289's env-blocked validation both PASSED because they never exercised the cross-tab enumeration in the shim; only driving the real multi-tab app exposed it. A single-terminal workspace works. Filed as follow-up ticket #295. Not a #291 parse bug — #291's fold is correct (proven: trace_rows=[2], gutter danger-red on a single-terminal drive).

## DL-cluster-8400f0f63e92
*topic: #393 section ＋ menu: the `SectionAction::SplitFocused(axis)` dispatch arm delegated ALL persistence to `split_focused_pane`, which persists internally EXCEPT on its #392 no-terminal early-return (`... · confidence: 0.500*

#393 section ＋ menu: the `SectionAction::SplitFocused(axis)` dispatch arm delegated ALL persistence to `split_focused_pane`, which persists internally EXCEPT on its #392 no-terminal early-return (`if try_workspace_mut().is_none() { return; }` skips `persist_grid()`). So a CROSS-PROJECT Panes＋ Split on a no-terminal project runs switch_project(p) + sync_active_project (in-memory) then returns without persisting → the active-project switch is lost on relaunch. Same class as the #387-F1 finding (a cross-project affordance must persist the switch even when the verb no-ops). Latent today (no-terminal is guard-forbidden until #395) but the code already carries the #392 guard and #395 is the next ticket; it also made dispatch_section_verb's own "applies the persist rule uniformly" docstring a lie. Fixed: the arm now does `self.split_focused_pane(axis); if project_changed { self.persist_grid(); }` (mirrors the OpenFile/OpenFileSplit arms). Found by inspect critic 1.

## DL-cluster-85bc8c214cdd
*topic: #252: inserting `save_active` (with its own doc + #[cfg_attr(test, mutants::skip)]) directly ABOVE `fn dispatch_action` stranded dispatch_action's OWN doc+skip — a doc/attribute binds to the NEXT i... · confidence: 0.500*

#252: inserting `save_active` (with its own doc + #[cfg_attr(test, mutants::skip)]) directly ABOVE `fn dispatch_action` stranded dispatch_action's OWN doc+skip — a doc/attribute binds to the NEXT item, so both skips + both docs attached to save_active, leaving fn dispatch_action with NO skip → 11 live mutants on the big dispatch shim → MSI RED. THE EXACT KNOWN TRAP (mutants-skip-detach-trap.md; hit before on #183/#184/#246). An inspect critic caught it via `cargo mutants --list` (0 dispatch_action mutants before, 11 after). Fixed: moved dispatch_action's doc+skip back directly above `fn dispatch_action`, kept save_active's own doc+skip above save_active. ALSO: adding the (⌘S,"save") keymap binding failed the `all_chords_lists_every_binding` roster-count guard (38→39) — the guard working as designed; fixed the count + asserted ⌘S is in the roster.

## DL-cluster-8618b28c7305
*topic: app.rs:6098/:6243 gated the EditorFrameGeom recorder (and the code_w probe) on `row == first` — a BUFFER row compared to a SLOT. With a collapsed fold fully above the viewport, buffer_row(slot) > s... · confidence: 0.500*

app.rs:6098/:6243 gated the EditorFrameGeom recorder (and the code_w probe) on `row == first` — a BUFFER row compared to a SLOT. With a collapsed fold fully above the viewport, buffer_row(slot) > slot for every rendered slot, so neither canvas mounted and editor_geom froze at its pre-scroll value — starving every geometry consumer (the #352 overlay anchors, click→caret, IME, h-scroll clamps) in exactly the fold-above-viewport scenario #352 fixes. Pre-existing since #305 (which moved `row` into the buffer domain and left the gates); found at #352 inspect by both critics independently. The Phase-1/2 sweeps missed it because they grepped geom.first ARITHMETIC — this was an EQUALITY gate against a same-named shadowing local. Fix: `if slot == first` at both sites.

## DL-cluster-864f4f9ebd12
*topic: #243 session-restore re-clamped the saved editor active index with a naive min() while serialize re-mapped it POSITIONALLY onto the surviving paths — an asymmetry. A file skipped BEFORE the active ... · confidence: 0.500*

#243 session-restore re-clamped the saved editor active index with a naive min() while serialize re-mapped it POSITIONALLY onto the surviving paths — an asymmetry. A file skipped BEFORE the active one on re-read (unreadable/binary/oversized since save) shifted which file is shown active on restore (cosmetic — from_files' min prevented a panic, but the wrong tab focused). Fixed by extracting a shared pure reclamp_active(surviving_orig, active) used by BOTH serialize AND the restore (the restore now enumerates orig indices in its filter_map). Found by the inspect critic + self-review. Class: a lossy transform that re-indexes on BOTH the save and restore sides must use the SAME re-clamp on both — extract it as one shared tested fn.

## DL-cluster-86c50a0d399c
*topic: M11 #174 driven validation: after cmdshift:a/cmd:t chords, a nil-source "plain" CGEvent click INHERITED the latched ⌘ from the session's synthetic-modifier state — the agent-row click hit the ⌘-cli... · confidence: 0.500*

M11 #174 driven validation: after cmdshift:a/cmd:t chords, a nil-source "plain" CGEvent click INHERITED the latched ⌘ from the session's synthetic-modifier state — the agent-row click hit the ⌘-click diff branch instead of the jump (twice; a clearmods key-up sweep did NOT reliably reset it). Fix (deterministic, landed in drive.swift): every synthetic mouse/key event now sets flags EXPLICITLY ([] for plain, the intended mask for chords) — never rely on inherited state; clearmods stays as belt-and-suspenders. The class: with CGEventPost, event flags are AMBIENT unless pinned per-event; any harness mixing chords and plain input must pin flags on EVERY event.

## DL-cluster-88679dd5b284
*topic: #240 set the editor tab's rail label to a stable "Editor" by changing Project::open_or_switch_code (tabs.rs). But Tab::code has TWO callers — the other is the session-restore arm TabLayout::Code(pa... · confidence: 0.500*

#240 set the editor tab's rail label to a stable "Editor" by changing Project::open_or_switch_code (tabs.rs). But Tab::code has TWO callers — the other is the session-restore arm TabLayout::Code(path) in the app shim (app.rs:926-936), which still derived the title from pb.file_name(). So a restored editor tab reverted to the file name after a quit-and-reopen — the exact frozen-name state #240 removes. My self-review grepped for .title READERS (found none) but not the other Tab::code WRITER. The adversarial inspect critic caught it. FIX: promoted EDITOR_TAB_TITLE to pub(crate) and used it at the restore site too. Class: a "change how X is labeled/constructed" edit must update ALL construction sites — grep for every caller of the constructor, not just the one path in view. The restore/deserialize path is a common second site that unit tests + a live driven capture (fresh session) don't exercise.

## DL-cluster-887466f9641e
*topic: #69's first cut re-fetched the forge sprint SYNCHRONOUSLY in the ⌘⇧F dispatch (on the gpui main thread). The forge is ~2.7s per tools/call (measured via curl) and current_sprint() does 2 calls → th... · confidence: 0.500*

#69's first cut re-fetched the forge sprint SYNCHRONOUSLY in the ⌘⇧F dispatch (on the gpui main thread). The forge is ~2.7s per tools/call (measured via curl) and current_sprint() does 2 calls → the open FROZE the UI ~5s: the overlay never appeared within the self-test's capture window because forge_open only flips AFTER the blocking fetch returns. Both the green gate AND the inspect critic ACCEPTED the sync design (the critic's MED estimated <100ms on localhost) — only the LIVE self-test exposed the freeze. FIX: a BACKGROUND-THREAD fetch — refresh_forge spawns a std::thread running current_sprint(), hands the result back through an mpsc::Receiver (forge_pending) that the 16ms pump try_recv-polls; the overlay opens instantly with a "loading sprint…" state (forge_pending.is_some()) and fills in when ready. ForgeClient gained #[derive(Clone)] so the thread owns a copy. This turned out to be the ticket's actual intent ("refresh + loading/… states"). Category: a UX/perf freeze, not a logic bug — invisible to cov/MSI (app.rs masked) and to a static critic; the self-test is the only oracle for UI responsiveness.

## DL-cluster-88b54966e9e3
*topic: #276 inspect F1 (BOTH critics, probe-verified): the editor Enter row deleted the selection then inserted "\n"+indent as a SECOND Buffer::edit — two undo records, while the arm's own comment claimed... · confidence: 0.500*

#276 inspect F1 (BOTH critics, probe-verified): the editor Enter row deleted the selection then inserted "\n"+indent as a SECOND Buffer::edit — two undo records, while the arm's own comment claimed "ONE edit (one undo step)" and the sibling #255 type-over path is a single edit(start..end, &repl). One ⌘Z landed the user on a state they never saw (selection deleted, no newline). Fix: compute (row, cin) = line_col(s) and the indent clone BEFORE the replace (the clone reads only chars strictly before s on its line, which the replace cannot change — probe-proved byte-identical), then ONE buffer.edit(s..e, &insert); bare caret degenerates to s==e==caret.

## DL-cluster-8a831a9ff342
*topic: A debug-panic/release-defined arm written as `debug_assert!(false, ...); <release-only fallback line>` (e.g. wrapping_add_signed, the backward-return, the floor round-down) makes the fallback line ... · confidence: 0.500*

A debug-panic/release-defined arm written as `debug_assert!(false, ...); <release-only fallback line>` (e.g. wrapping_add_signed, the backward-return, the floor round-down) makes the fallback line UNREACHABLE in a debug build (the assert panics first). Because gate:4 runs `cargo llvm-cov` in the default DEBUG profile, those lines get 0 coverage → gate:4 (--fail-under-lines 100) RED. Separately, a boundary guard like `while i > 0 && !is_char_boundary(i)` carries an EQUIVALENT mutant (`> 0` → `>= 0`, indistinguishable because is_char_boundary(0) is always true) that survives cargo-mutants → gate:5 (MSI 100, no exclusions) RED. Both caught in inspect by the mutation-readiness critic before validate. Fixed by restructuring to assert-the-invariant + always-compute-the-defined-result, and dropping the redundant guard.

## DL-cluster-8b13e00e3be1
*topic: #72's first cut cleared the composed prompt line unconditionally whenever last_agent was Some — including when state_mut(target) returned None (the last-launched agent's pane had been closed; close... · confidence: 0.500*

#72's first cut cleared the composed prompt line unconditionally whenever last_agent was Some — including when state_mut(target) returned None (the last-launched agent's pane had been closed; close-pane removes from self.agents but never clears self.last_agent). Repro: launch an agent (last_agent=X) → close pane X → without relaunching, compose a line → cmd-shift-s. The write is skipped (target gone) but the line is still wiped — user input vanishes with nothing delivered and no feedback. Memory-safe (the critic-brief called it a 'harmless no-op') but NOT harmless: it silently drops user input. Invisible to the gate (shim: mutants::skip + cov-excluded) — only manual inspect caught it. FIX: clear the compose buffer ONLY after a confirmed send — `let sent = match state_mut(target) { Some(s) => s.session.write_bytes(&payload).is_ok(), None => false }; if sent { clear }`. A missing target OR a failed PTY write now preserves the line.

## DL-cluster-8b39ab3d858b
*topic: #64's new cmd-shift-f "toggle-forge" chord was dead-on-arrival: the pre-existing find-bar intercept (`modifiers.platform && key=="f"`, no shift guard, app.rs) ran before the keymap dispatch and ret... · confidence: 0.500*

#64's new cmd-shift-f "toggle-forge" chord was dead-on-arrival: the pre-existing find-bar intercept (`modifiers.platform && key=="f"`, no shift guard, app.rs) ran before the keymap dispatch and returned, so cmd-shift-f opened the find bar and the forge overlay never toggled. The keymap action_for unit test passed (binding table, not runtime reachability); a correctness critic caught it in inspect by tracing gpui event delivery. Root cause: a hardcoded key check shadowed a keymap chord sharing its base key. Fix: added `&& !modifiers.shift` so cmd-F still finds and cmd-shift-f falls through to toggle-forge; verified by self-test-driving the real keystroke. See PR-claude-new-chord-shadowed-by-hardcoded-key-001.

## DL-cluster-8d53364ecb3b
*topic: #321's headless drives must make the LSP spawn gate PASS in order to assert that a restored editor tab creates a host. The gate requires a `Cargo.toml` in the project root — so the drives write one... · confidence: 0.500*

#321's headless drives must make the LSP spawn gate PASS in order to assert that a restored editor tab creates a host. The gate requires a `Cargo.toml` in the project root — so the drives write one into their tempdir. That silently removed the protection #308 had been relying on.

#308's notes record it explicitly: headless tests open `.rs` files under a tempdir root and "would spawn a REAL rust-analyzer 4x (restart cap) during the gate"; the mitigation was that `ensure_lsp_for_opened_file` also gates on `root.join("Cargo.toml").is_file()`, which "excludes bare-tempdir tests". By writing a manifest to exercise the gate, my drives re-opened exactly that hole.

`rust-analyzer` is installed at ~/.cargo/bin on this machine, and `resolve_binary` falls back to a PATH/`~/.cargo/bin` search for `rust`, so the pump launched a genuine rust-analyzer against a throwaway tempdir. The test binary then sat at 0% CPU indefinitely while the server tried to index. Symptom was indistinguishable from the machine's known intermittent syspolicyd fault (a hung test binary, no output) — I only separated them by checking `ps -o %cpu -p $(pgrep syspolicyd)` and finding it idle at 0%, then confirming rust-analyzer was on PATH.

FIX: seed `[[lsp.servers]]` with `{language: "rust", command: "/nonexistent/marley-test-stub-lsp"}` before boot. `resolve_binary` short-circuits on a configured entry, and an absolute command failing the existence probe yields None — so the host is still CREATED (what the drives assert) with `resolved: None`, and nothing spawns. The ticket's own spec had specified "a root WITH Cargo.toml + [[lsp.servers]] naming a stub binary" in REQ-001; I implemented the manifest half and dropped the stub half, which is what made this reachable.

## DL-cluster-8e3e9d31f9aa
*topic: #264: the mutants::skip detach trap's FOURTH strike, in a NEW shape — not an inserted fn above a skip, but a CTOR SPLIT: RootView::new kept its skip while its ~600-line body moved into unskipped ne... · confidence: 0.500*

#264: the mutants::skip detach trap's FOURTH strike, in a NEW shape — not an inserted fn above a skip, but a CTOR SPLIT: RootView::new kept its skip while its ~600-line body moved into unskipped new_in → 57 live mutants. WORSE: the --diff gate was structurally blind (the only in-diff mutant, new_in body→Default::default(), is UNVIABLE — RootView has no Default — so diff-mode prints 0-viable PASS while the next FULL run would collapse). Rule refinement: the --list re-check must follow ANY refactor that MOVES a function body (split/rename/extract), not just insertions near a skip; and a green --diff mutation on a shim-file refactor proves nothing when the moved body's only in-diff mutant is unviable.

## DL-cluster-8ee1c318d163
*topic: #187 added a required field `exit: Option<(i32,u64)>` to AgentRow. I updated the AgentRow struct-literal constructions in agent_rows_sorted_by_pane_id + the two agent_badge/AgentRun sites, but MISS... · confidence: 0.500*

#187 added a required field `exit: Option<(i32,u64)>` to AgentRow. I updated the AgentRow struct-literal constructions in agent_rows_sorted_by_pane_id + the two agent_badge/AgentRun sites, but MISSED the two literals in agent_row_text_full and agent_row_text_bare — so the marley crate TEST target failed to compile (E0063 missing field), which would block every downstream gate (tests/mutation/commit). cargo check (non-test) was clean, so it was invisible until a critic ran `cargo check --tests`. Caught by both inspect critics; fixed by adding `exit: None` to both. Root cause: relying on the obvious/central test site instead of grepping ALL `AgentRow {` construction sites.

## DL-cluster-8f59aabd3636
*topic: #243's multi-file editor save/restore CODEC was fully unit-proven (serialize/parse/reclamp/from_files round-trip, cov/MSI 100, the inspect critic concurred) — but the DRIVEN quit→relaunch showed th... · confidence: 0.500*

#243's multi-file editor save/restore CODEC was fully unit-proven (serialize/parse/reclamp/from_files round-trip, cov/MSI 100, the inspect critic concurred) — but the DRIVEN quit→relaunch showed the editor tab VANISHED. Root cause: open_file_in_viewer (app.rs) mutated the editor tab in-memory via open_or_switch_code and NEVER called persist_grid, so the layout SAVE was never TRIGGERED on file-open (a pre-existing gap; pre-#243 the single-file editor tab was equally un-persisted on open). REQ-004 ('open N → quit → relaunch → restore N') could not be met by a correct codec alone. Fixed by adding self.persist_grid() to open_file_in_viewer's success arm. LESSON: a 'persist X' feature must verify the persist is TRIGGERED end-to-end (a driven quit→relaunch), not just that the codec round-trips — every unit test + the codec passed while the feature was silently broken end-to-end; only the pixels/settings.toml caught it.

## DL-cluster-8f59ce8bea61
*topic: TICKET-008 inspect (caught BEFORE validate): the Phase-2 design's mutation-kill fixture for mask_out was degenerate — a 1×2 image masking ROW 0, i.e. the ORIGIN pixel (0,0). The index expression `(... · confidence: 0.500*

TICKET-008 inspect (caught BEFORE validate): the Phase-2 design's mutation-kill fixture for mask_out was degenerate — a 1×2 image masking ROW 0, i.e. the ORIGIN pixel (0,0). The index expression `((y*w+x)*4)` evaluates to 0 there, and 0 is a FIXED POINT of all the arithmetic operators cargo-mutants swaps: `y*w`→`y/w` (0/1==0*1), `…+x`→`…-x` (0-0==0+0), `…*4`→`…/4` (0/4==0*4). So 4 index mutants on image_diff.rs:283 produce byte-identical output and SURVIVE → MSI<100 would have hit at validate. This is a SHARPER form of the 005/007 single-row lesson (PR-claude-loopbound-fixture-and-childproc-bin-skip-001): not just loop bounds, but ANY index-arithmetic mutant is undetectable when the test exercises only index 0 (or a 1-wide grid where x is always 0). The critic verified via `cargo mutants --list` + a hand trace. Fix (test-only): use a NON-ORIGIN interior masked pixel in a multi-row, multi-col, w≠h grid with an unmasked remainder — solid(2,3,...) + titlebar_band(2,3,2) (masks rows 0-1 incl pixel (1,1), leaves row 2), asserting the masked bytes == fill AND the unmasked row == original. The production code is CORRECT; only the kill-fixture was insufficient.

## DL-cluster-902ad4b4d8df
*topic: Marley #296→#297. SelectionSet::from_selections merged ANY two members whose ranges touched (`next.start() <= cur.end()`). I justified the `<=` in a doc comment AND pinned it with a test whose stat... · confidence: 0.500*

Marley #296→#297. SelectionSet::from_selections merged ANY two members whose ranges touched (`next.start() <= cur.end()`). I justified the `<=` in a doc comment AND pinned it with a test whose stated rationale was "otherwise two abutting selections would each insert at the seam and double up". THE RATIONALE IS FALSE: two abutting ranges replace two DISJOINT spans, and the shipped back-to-front sweep applies them with zero interference (`[0..3]`+`[3..6]` over "abcdef" with "X" must yield "XX"; it yielded "X"). The cost surfaced one ticket later: ⌘⌥↓ (add cursor below) followed by ⇧↓ (extend down) produces two line-spanning ranges that abut EXACTLY at the shared line-start offset — so the flagship multi-cursor gesture destroyed its own second cursor on the very next keypress, and because a motion is not an edit, ⌘Z could not bring it back. The gate was green throughout: 100% line coverage, MSI 100, a 50k-step differential fuzzer, and 1066 passing tests — because a TEST ASSERTED THE BUGGY BEHAVIOR and thereby defended it. Found by an adversarial critic, reproduced by me. FIX: a caret-aware merge (`<=` only when either side is a caret — same-offset carets are one cursor and must collapse; two RANGES merge only on a TRUE overlap, `<`), the pinning test replaced with its inverse, and the companion `selections_after_multi_edit` switched from raw `from_members` to canonicalizing `from_selections` (once touching ranges survive, a DELETE over them emits two carets at one offset).

## DL-cluster-907f9c580a17
*topic: #330: `sticky_rows` used `if enclosing.len() > max_depth { enclosing.drain(0..enclosing.len() - max_depth); }`. `cargo mutants --list` yields `replace > with >=` — a PROVABLY EQUIVALENT mutant: `>`... · confidence: 0.500*

#330: `sticky_rows` used `if enclosing.len() > max_depth { enclosing.drain(0..enclosing.len() - max_depth); }`. `cargo mutants --list` yields `replace > with >=` — a PROVABLY EQUIVALENT mutant: `>` and `>=` differ only when `len == max_depth`, and there the mutant executes `drain(0..0)`, a no-op. Output is identical for every input, so NO test can kill it — it would survive forever, dropping MSI below the 100 floor on a pure seam and turning gate:5 red with no fix available except restructuring. The guard also generated the `- → +` and `- → /` arithmetic mutants (the latter needing a ≥5-scope test to kill at max_depth 2, since 3-2 == 3/2 == 1 — easy to miss). Caught by inspect critic 1 (logical proof + --list). Fixed: `let drop_outer = len.saturating_sub(max_depth); drain(0..drop_outer);` — identical behavior, and `saturating_sub` is a method call (cargo-mutants leaves it unmutated), so the entire guard+arithmetic mutant cluster disappears.

## DL-cluster-909a1e1c1433
*topic: Input routing keyed on the ALTERNATE screen only (input_route(alt_screen, ctrl) from #33), so interactive programs on the PRIMARY screen — Claude Code arrow-key menus, `read`, `npm init`, interacti... · confidence: 0.500*

Input routing keyed on the ALTERNATE screen only (input_route(alt_screen, ctrl) from #33), so interactive programs on the PRIMARY screen — Claude Code arrow-key menus, `read`, `npm init`, interactive git — never received arrow/nav keys; Marley routed them to its own local history recall. Root cause: the routing model conflated 'a program wants raw keys' with 'the alternate screen is active'; the real signal is 'a foreground command is running'. An inline interactive program doesn't switch to the alt-screen, so it fell to Cooked → local editor. Found by chad running the built app. FIX: route by is_command_running() (a Running block exists, bracketed by the shell Preexec/Precmd) — input_route(alt_screen, ctrl, command_running) streams Raw if any hold; while a command runs every key reaches it, local editing only at the bare prompt. cov/MSI 100 on both pure fns; orthogonal to the local-edit model.

## DL-cluster-9865193478c0
*topic: enforce-changelog.sh (a commit-content gate, mirroring enforce-commit-gate) decided from `git status --porcelain` — the WORKTREE — instead of the staged index. The worktree includes untracked (??) ... · confidence: 0.500*

enforce-changelog.sh (a commit-content gate, mirroring enforce-commit-gate) decided from `git status --porcelain` — the WORKTREE — instead of the staged index. The worktree includes untracked (??) and unstaged ( M) entries that are NOT part of a `git commit`, so any CHANGELOG.md merely present on disk satisfied the gate. Because CHANGELOG.md was itself untracked at that moment, the gate was LIVE-inert: every crates/*/src/*.rs commit was exempt for free. A subdir CHANGELOG.md decoy also passed (the match regex wasn't root-anchored). Caught by the inspect bypass critic (verified by running the hook against synthetic staged states). Fixed: gate on `git diff --cached --name-only` (the staged index = the commit), anchor the required file to `^CHANGELOG\.md$` (repo root), scope the trigger to `^crates/[^/]+/(src|examples)/.*\.rs$`. Verified: staged .rs + untracked CHANGELOG → exit 2; + staged CHANGELOG → exit 0; subdir CHANGELOG decoy → exit 2.

## DL-cluster-98ca339f23e5
*topic: #278 inspect F1 (HIGH, code-traced): the new readline arm in the terminal key region was reachable on EDITOR/COCKPIT tabs — focused_terminal() falls back to the first terminal pane (#71) and the #2... · confidence: 0.500*

#278 inspect F1 (HIGH, code-traced): the new readline arm in the terminal key region was reachable on EDITOR/COCKPIT tabs — focused_terminal() falls back to the first terminal pane (#71) and the #251 editor arm's !control gate deliberately lets ⌃-chords fall through, so ⌃K/⌃U/⌃Y edited the HIDDEN pane's prompt buffer (visible corruption on return — the #283 hidden-prompt class upgraded from raw C0 bytes to Marley-visible state). CLASS: any NEW arm in the terminal region that MUTATES prompt state must gate on the active tab actually BEING a terminal (active_tab().grid().is_some()) — the (alt_screen, command_running) read alone describes the hidden fallback pane, not the visible surface. Fix: the grid().is_some() gate; non-terminal tabs revert byte-identically to the raw route; headless editor-tab negative added at validate.

## DL-cluster-98e55ef5fa78
*topic: The #200 inline-history ghost-text render condition (caret==char-count && !completion_open) was NOT gated on `is_focused`, yet it lives inside the per-pane render loop (`for (pane_id, r) in &rect_l... · confidence: 0.500*

The #200 inline-history ghost-text render condition (caret==char-count && !completion_open) was NOT gated on `is_focused`, yet it lives inside the per-pane render loop (`for (pane_id, r) in &rect_list; let is_focused = pane_id == focused`). So every split pane at EOL with a matching history rendered its own muted ghost — but the → ACCEPT is focused-only (`focused_terminal_mut()`), so → could never accept a non-focused pane's ghost (a render/accept divergence across PANES) + it diverges from Warp (autosuggest is active-input only). Caught by the inspect critic; my self-review checked TIME divergence (same-frame render vs the → event) but missed the MULTI-PANE case. Marley already had the precedent: the #186 find-highlight is explicitly focused-only ("would bleed onto a non-focused pane"). Fixed by adding `is_focused &&` to the ghost condition. Pre-ship (inspect→validate), no shipped bug.

## DL-cluster-9a485dbc031e
*topic: TICKET-004 introduced a workspace clippy.toml (disallowed-types = std::process::Command — the SOLE enforcement of R14). clippy.toml is a gate-DEFINING file (it determines gate:2 clippy behavior), b... · confidence: 0.500*

TICKET-004 introduced a workspace clippy.toml (disallowed-types = std::process::Command — the SOLE enforcement of R14). clippy.toml is a gate-DEFINING file (it determines gate:2 clippy behavior), but gate_state_hash() in .claude/hooks/lib-hook-helpers.sh fingerprinted only crates/**/*.rs + scripts/*.sh + .claude/hooks/** + deny.toml + .gitleaks.toml + Cargo manifests/lockfile — NOT clippy.toml. So the commit receipt didn't bind clippy.toml: an attacker/sloppy edit could delete the ban from clippy.toml, run the gate green (clippy passes), restore the ban (gate_state_hash unchanged since it excludes clippy.toml), and commit R14-violating code under a stale-but-matching receipt — a config-swap-around-the-green that no honest gate run would pass. Caught by the inspect integrity critic. Fix: added clippy.toml to both ls-files pathspecs in gate_state_hash (the *.toml case filter already hashes it); verified an edit to clippy.toml now changes the fingerprint. The fix is self-certifying (lib-hook-helpers.sh is itself in the fingerprint).

## DL-cluster-9b05a8d86aae
*topic: #368 projection: a `session-end` seat_events row carrying `state:"error"` projected to a bare `SessionEvent::Ended` (which never reads the row's state), so the marley_fleet reducer — seeing a first... · confidence: 0.500*

#368 projection: a `session-end` seat_events row carrying `state:"error"` projected to a bare `SessionEvent::Ended` (which never reads the row's state), so the marley_fleet reducer — seeing a first-seen/auto-vivified seat that is NOT already in Error — set it to cleanly `Done`. A crashed seat thus rendered as a clean exit, violating the safety-critical "a seat that died on an error must never read as `Idle` or cleanly `Done`" invariant. Amplified by the cursor-only reconnect model: if the prior `api-error` (which correctly → Error) sat before the persisted cursor and the `session-end` after, replay never saw the Error at all. Root cause: the projection mapped a rich wire row onto a NARROWER event vocabulary through non-descriptor-carrying events (Ended/QuestionRaised/Heartbeat), silently discarding the row's own state/descriptor. Fix: project_row now returns Vec<SessionEvent>, routing every descriptor-bearing kind through an Upsert(full descriptor) and honoring a terminal error state — session-end+error → Upsert(Error) (retained), clean end → Ended→Done. Caught by an adversarial projection critic; the naive test (session-end with no state) had missed it.

## DL-cluster-9b137ad09198
*topic: The 4 survivors: cursor `*cursor += 1` → `*= 1`, the recurse guard `is_dir && !collapsed && toggle_at` with `&&`→`||` (×2) and dropping the `!`. My toggle tests only hit index 0 or file rows, so th... · confidence: 0.500*

The 4 survivors: cursor `*cursor += 1` → `*= 1`, the recurse guard `is_dir && !collapsed && toggle_at` with `&&`→`||` (×2) and dropping the `!`. My toggle tests only hit index 0 or file rows, so they never forced the cursor to ADVANCE through multiple nodes nor exercised the collapse-aware recursion guard. A directory-tree toggle-by-visible-index needs a test that (a) toggles a DEEP row (cursor must advance past earlier nodes) and (b) collapses a MID dir then toggles a LATER visible index (which must skip the hidden subtree). The inspect critic had spelled out this EXACT sequence; I under-tested it with shallower cases. Fixed by adding `toggle_recurses_and_respects_collapse` → MSI 100 (23/0). Lesson: when a critic hands you the precise mutation-killing sequence, write THAT sequence, don't approximate.

## DL-cluster-9e160784a0e1
*topic: The #200 ghost-accept (Key::Right at EOL inserting the suggestion suffix via buffer.edit) mutated the prompt buffer WITHOUT calling `state.history.detach()`, unlike every other buffer-editing path ... · confidence: 0.500*

The #200 ghost-accept (Key::Right at EOL inserting the suggestion suffix via buffer.edit) mutated the prompt buffer WITHOUT calling `state.history.detach()`, unlike every other buffer-editing path (the R36 contract — app.rs detaches on Char/Backspace/DeleteForward with the comment "an edit (not a motion) leaves history navigation"). Repro: mid-recall (↑ to "git", history cursor Some(0)), accept the ghost → buffer "git push" but cursor STILL Some(0); a subsequent ↑ → recall_prev clamps to entries[0]="git" and the accepted "git push" is DISCARDED + never stashed (the draft is only stashed on the None→newest transition). A genuine R36 contract violation. Caught by the inspect critic (I checked the accept's borrow/return but not its parity with the detach-on-edit contract). Fixed by adding `state.history.detach();` in the accept branch, mirroring the Char/Backspace path. Pre-ship.

## DL-cluster-9e7877133444
*topic: TICKET-017: the headed widget-gallery fixture bin (crates/ui_components/src/bin/marley_widgets_gallery.rs) was coverage-excluded in scripts/gates.sh (rust_cov --ignore-filename-regex ...ui_componen... · confidence: 0.500*

TICKET-017: the headed widget-gallery fixture bin (crates/ui_components/src/bin/marley_widgets_gallery.rs) was coverage-excluded in scripts/gates.sh (rust_cov --ignore-filename-regex ...ui_components/src/bin/) but its fns (GalleryView::new, the Render::render impl, main) were NOT annotated #[cfg_attr(test, mutants::skip)]. cargo mutants --list -p marley_ui_components showed 8 surviving-mutant candidates ON THE BIN (4 render match-arm deletions + replace-main-with-() + 3 WindowOptions/TitlebarOptions field deletions) — the bin is exercised ONLY by the #[ignore] headed test (not run in the gate), so those mutants would SURVIVE → MSI<100 → mutation gate RED. ROOT CAUSE: coverage-exclusion (rust_cov --ignore-filename-regex) and mutation-exclusion (mutants::skip) are SEPARATE mechanisms; excluding a file from the coverage denominator does NOT stop cargo-mutants from mutating it. Caught in-context at inspect via cargo mutants --list. FIX: added #[cfg_attr(test, mutants::skip)] to all 3 bin fns (the #16 marley.rs/app.rs pattern); re-listed → 0 bin mutants, 0 render mutants, 24 pure-surface mutants.

## DL-cluster-9f94b6ab11b2
*topic: #313's completion popup key arm matched bare key NAMES (`"enter" | "tab"`, `"up" | "down"`) and never read `keystroke.modifiers`, while sitting ABOVE both the keymap dispatch and the editor's Tab a... · confidence: 0.500*

#313's completion popup key arm matched bare key NAMES (`"enter" | "tab"`, `"up" | "down"`) and never read `keystroke.modifiers`, while sitting ABOVE both the keymap dispatch and the editor's Tab arm in the on_key_down ladder. Consequences while the popup was open: (1) ⇧TAB matched `"tab"` and ACCEPTED a completion instead of reaching the editor's dedent arm — a wrong EDIT, not merely a dead key; (2) ⌘⌥↑/↓ matched `"up"`/`"down"` and moved the popup selection, killing the shipped M19 #297 add-cursor-above/below gestures for as long as the popup was up.

THE TELL: the very idiom #313 claimed to mirror already had the guard, and said why. #96's shell-popup arm carries `let plain = !mods.platform && !mods.control && !mods.alt;` on every branch with the comment "Only UNMODIFIED nav keys drive the popup — a ⌘/⌃/⌥ chord falls to `_`, dismissing and dispatching normally (so ⌘↓ still jumps blocks, ⌘1 still switches tabs)". #313 copied the SHAPE (a key-name match returning handled/declined) and dropped the GUARD.

FIX: adopt #96's gate — any platform/control/alt/shift modifier returns declined before the match.

## DL-cluster-a030b0994389
*topic: #265 gate:14 went RED (rustdoc -D warnings) because a PUBLIC item's doc comment ([`Keymap::action_for`], public via re-export) used an intra-doc link to [`chords_unique_scoped`] — pub in its module... · confidence: 0.500*

#265 gate:14 went RED (rustdoc -D warnings) because a PUBLIC item's doc comment ([`Keymap::action_for`], public via re-export) used an intra-doc link to [`chords_unique_scoped`] — pub in its module, but the module is private and the fn is not re-exported, so from the public-docs view the link targets a private item (rustdoc::private_intra_doc_links). Everything else incl. coverage/MSI was green on the same run. Fix: plain backticks + a "(crate-internal, not re-exported)" note. Class: a doc link's validity depends on the CRATE-PUBLIC surface, not module-local visibility.

## DL-cluster-a2482faabc1a
*topic: #261 CRITICAL (inspect critic): 10 new SVG assets were wired through icons.rs icon_path but NOT added to the app's AssetSource (Assets::load — a STATIC per-file include_bytes! match, not a dir embe... · confidence: 0.500*

#261 CRITICAL (inspect critic): 10 new SVG assets were wired through icons.rs icon_path but NOT added to the app's AssetSource (Assets::load — a STATIC per-file include_bytes! match, not a dir embed). gpui renders a missing asset as a silent 12px blank (logs + keeps the layout box) — compile green, 368 tests green, clippy green, icon invisible at runtime. The notes even claimed a nonexistent "include_bytes! embeds the dir" compile proof. Fixed: 10 match arms + a permanent assets_serve_every_icon_variant guard walking every Icon through icon_path→Assets::load.

## DL-cluster-a257c0eb3dad
*topic: #205: the restore flow reused the pre-spawned boot PTY (spawned in the launch cwd) for the launch project's FIRST terminal UNCONDITIONALLY (`boot_session.take_if(|_| root == project_root)`), so tha... · confidence: 0.500*

#205: the restore flow reused the pre-spawned boot PTY (spawned in the launch cwd) for the launch project's FIRST terminal UNCONDITIONALLY (`boot_session.take_if(|_| root == project_root)`), so that pane's saved cwd was silently ignored — and that's the MOST COMMON case (one window launched from the project dir, the primary terminal `cd`'d into a subdir): the whole feature no-op'd for it. I had DESIGNED this as a "documented limitation" (the boot-PTY-reuse keeps the boot cwd), under-weighting that it's the common case, not an edge. Caught by the inspect shim critic. Fixed: gate the reuse on `cwd_or_root(gl.cwds.first(), &root) == root` — reuse the boot PTY only when the saved cwd resolves back to the launch root; a `cd`'d subdir spawns fresh THERE (the boot PTY is dropped, a minor startup cost). ALSO fixed a sibling MED: `cwd_or_root` used `Path::new(c).exists()` (true for a FILE) where `is_dir()` is correct — a cwd that became a file would pass to spawn_session_in → the PTY spawn fails → the pane/tab is silently dropped; `is_dir()` cleanly redirects a now-file/broken cwd to root.

## DL-cluster-a4b888db8294
*topic: #307 widened indent_edits/dedent_edits from a contiguous (first,last) range to rows: &[usize] and DELETED the `last.min(len_lines-1)` clamp. The doc comment justified the deletion by saying the emp... · confidence: 0.500*

#307 widened indent_edits/dedent_edits from a contiguous (first,last) range to rows: &[usize] and DELETED the `last.min(len_lines-1)` clamp. The doc comment justified the deletion by saying the empty-line filter drops an out-of-range row "before Buffer::line_start is ever asked about it" — framing line_start as the hazard being avoided. Reading line_start showed it opens with `let row = row.min(self.len_lines().saturating_sub(1));`: it CLAMPS, it never panics. So the safety argument named the wrong failure mode. The code was correct; the reasoning was not, and the real hazard is strictly worse than the one described — an unfiltered out-of-range row would resolve to the LAST line's start offset and silently indent or dedent the wrong line, with no crash to notice. Fixed by rewriting both fns' docs to state the actual hazard (silent wrong-line edit) and why no redundant bounds guard is added. Same class as the M22 "CHECK THE REASON, not just the fix" lesson: a plausible justification terminates review before anyone measures it.

## DL-cluster-a55a5e2c45d0
*topic: #395 REQ-003 (restore empty) was unmet: the boot restore loop (app.rs:2101) force-seeded a terminal onto ANY restored project with no terminal tab ("A project must keep ≥1 terminal tab (the workspa... · confidence: 0.500*

#395 REQ-003 (restore empty) was unmet: the boot restore loop (app.rs:2101) force-seeded a terminal onto ANY restored project with no terminal tab ("A project must keep ≥1 terminal tab (the workspace() invariant)"), so a persisted EMPTY project restored with tab_count 1, not 0. But that ≥1-terminal invariant is exactly what #391/#392 DISSOLVED at RUNTIME (the try_workspace/try_active_tab twins make the app total over a no-terminal/zero-tab project). The runtime tests (close the last tab → tab_count 0) all passed — the gap was ONLY in the BOOT path, which still enforced the old invariant. Caught at validate by a NEW headless test (boot_restores_a_zero_tab_project_empty) that seeded a zero-tab shell blob and asserted tab_count 0 (got 1). Fixed at source: added Project::empty(name, root) (the Workspace::empty analog) + relaxed the restore loop (Some(first_tab)→Project::new as before; None→Project::empty, restoring empty); the unused boot PTY reaps at the existing terminal-less arm. Triple-proven (the project_empty unit + the boot_restores headless + the live pixel capture). LESSON: dissolving a RUNTIME invariant must include auditing the BOOT/restore path that ALSO enforced it — runtime tests won't catch a boot-only enforcement.

## DL-cluster-a58d1c988bf5
*topic: #289's diagnostics shim resolved terminal file-refs against `active_project().root` to compare them to the OPEN editor file's stored path — but that stored path was built (in load_code_view_state) ... · confidence: 0.500*

#289's diagnostics shim resolved terminal file-refs against `active_project().root` to compare them to the OPEN editor file's stored path — but that stored path was built (in load_code_view_state) by resolving against `self.project_root`, a DIFFERENT field (kept in sync but distinct). Any sync-timing gap would make the resolved ref != the stored open_path, silently dropping a valid diagnostic. Fix: resolve against the SAME field (`self.project_root`) that built the stored path, so a path-equality comparison is guaranteed consistent. Class: when comparing a freshly-resolved path against a STORED resolved path, use the identical resolution root, not a parallel one.

## DL-cluster-a595def44176
*topic: #336 horizontal scroll. The per-frame scrollable-width measure used `layout.display.chars().count()` while every consumer of scroll_x — and the caret's own position via `col_of_offset`/`col_starts`... · confidence: 0.500*

#336 horizontal scroll. The per-frame scrollable-width measure used `layout.display.chars().count()` while every consumer of scroll_x — and the caret's own position via `col_of_offset`/`col_starts` — works in display CELLS. Chars and cells are a DIFFERENT DOMAIN: `char_width` (UAX#11) makes a CJK glyph 2 cells and a combining mark 0. Measured on the real code: cjk_pure 9 chars vs 18 cols; cjk 21 vs 31; emoji 14 vs 17; combining 6 vs 3 (over-counts). End-to-end trace: 100 CJK chars = 200 cells → content_px reported 800px while caret_px reported 1600px → the clamp capped scroll_x at 416 and stranded the caret 784px OFF-SCREEN PERMANENTLY (every later follow recomputes the same wrong max). That is the ticket's OWN defect — an unreachable line tail — reintroduced for exactly the lines that need it most. It also defeated the module's own documented invariant ("the caret's row is always in the max, or follow could ask for a scroll clamp then takes away") because caret_w used the same broken measure. ASCII and tabs agree EXACTLY, which is why it survives casual testing. Second half of the same bug: the width block measured `line_layout` (no inlays) while the row renders `line_layout_with_inlays`, so a trailing #331 inlay hint (14 vs 19 cols) was unscrollable. BOTH inspect critics found this independently — one from the math, one from the render. Fix: `LineLayout::display_cols()` (walks `display` via char_width; deliberately NOT `col_starts.last()`, which stops before an EOL phantom) + measure the SAME inlay-aware layout the render draws. THE POINT: `AD-claude-editor-offset-column-model-001` already declared code_view the owner of the column domain, and naming that invariant did not stop the violation — it happened in a coverage-excluded, mutants::skip'd shim where no pure test could see it.

## DL-cluster-a613af3af31a
*topic: #266: highlight_ranges guarded empty spans with `end > at` where end = at + span.text.len() — the only empty span highlight_line can emit is Plain (already excluded by the kind check), so `>` vs `>... · confidence: 0.500*

#266: highlight_ranges guarded empty spans with `end > at` where end = at + span.text.len() — the only empty span highlight_line can emit is Plain (already excluded by the kind check), so `>` vs `>=` was semantically EQUIVALENT and unkillable (probe-verified over 60k lines) → guaranteed MSI<100. Fix class: phrase a guard on the DOMAIN FACT (`!span.text.is_empty()`) whose mutant (delete-!) IS observable, not on derived arithmetic whose boundary case is unreachable. Sibling of the known syntactic-form rules.

## DL-cluster-a79a7c48efaf
*topic: #331. The LSP request-timeout path does `purposes.remove(&id)` and pushes NO response — so the consumer's apply fn never runs. Event-driven consumers (hover/completion/signature) tolerate this: eac... · confidence: 0.500*

#331. The LSP request-timeout path does `purposes.remove(&id)` and pushes NO response — so the consumer's apply fn never runs. Event-driven consumers (hover/completion/signature) tolerate this: each keystroke mints a fresh key, so their stale latch is inert and the next send overwrites it. The inlay refresh is POLL-driven off an UNCHANGING viewport key, and its send-skip (`inlay_request == Some(&key)`) is load-bearing (without it the pump re-sends every 16ms tick while the cache is empty). So a dropped purpose left the latch matching forever → no resend → hints silently dead for exactly as long as the user READS the file — which is exactly when a cold rust-analyzer exceeds the 10s timeout. AMPLIFIER: when the server finished indexing and sent `workspace/inlayHint/refresh` (the literal "hints are ready now" signal), the destructive `take_inlay_refresh` consumed it, dropped the cache, and hit the skip — the one mechanism that would heal the wedge guaranteed it stayed wedged WITH AN EMPTY CACHE. Directly contradicted `REQUEST_TIMEOUT_TICKS`'s own comment: "a slow server never wedges a consumer". Fix: ask the host (`has_pending_inlay()` over the pending table — the ONE source of truth for in-flight-ness) instead of trusting a local latch; plus clear the latch on refresh so a pre-refresh in-flight answer is discarded.

## DL-cluster-a7d94df9330a
*topic: marley_app (the gpui finale): THREE successive implement subagents, each told to build the crate + "confirm the gpui 0.2.2 API (cx.spawn/on_key_down/open_window/Keystroke)", spent their ENTIRE run ... · confidence: 0.500*

marley_app (the gpui finale): THREE successive implement subagents, each told to build the crate + "confirm the gpui 0.2.2 API (cx.spawn/on_key_down/open_window/Keystroke)", spent their ENTIRE run compiling gpui + reading its source to research signatures and NEVER wrote a single file — each killed after many idle cycles with the crate dir still absent (their last states: "confirm the fluent on_key_down..."/"start by reading the essential context files..."). Root cause: a subagent instructed to VERIFY an API on a slow-to-compile dep (gpui ~minutes/build), without a hard write-first constraint, enters an unbounded research loop — each cargo check recompiles, and it keeps finding one more thing to confirm. Fix: I stopped them and wrote the crate MYSELF against the spike's PROVEN in-repo gpui calls (crates/marley_spike/src/app.rs) using WRITE-then-cargo-check-iterate — cargo check passed 0 errors FIRST TRY; every gpui surface the subagents feared compiled as written. The exemplar + a small targeted signature grep (ThemeColors/SessionOptions/Buffer) was all that was needed; the research loop was pure waste. (The pure modules needed zero gpui knowledge and could have been written in seconds — the subagents never even got to them.)

## DL-cluster-a8285e5099e0
*topic: #393 section ＋ menu: `open_section_menu` closed only palette/finder/history/find before opening, but the key router is one linear function where ~12 editor-overlay arms (naming_workflow, def_picker... · confidence: 0.500*

#393 section ＋ menu: `open_section_menu` closed only palette/finder/history/find before opening, but the key router is one linear function where ~12 editor-overlay arms (naming_workflow, def_picker, open_references, code_action_menu, signature_card, open_symbols, open_search, open_problems, …) run BEFORE the context_menu arm and early-return. Those pickers are single CENTERED cards (no full-window inset_0 scrim — only the context_menu has the backdrop), so the LEFT-rail ＋ is NOT occluded → open a centered editor picker, then click a rail ＋: the section menu renders but the keyboard belongs to the picker (the naming_workflow case leaks typed chars into the hidden draft). Exactly the class the #181 agent launcher (the sibling mouse-opened modal) already fixed at #229/#312/#317/#323/#324/#325/#326/#327. Fixed by extracting `close_transient_overlays()` (the launcher's 13 closers, one source of truth) and calling it from BOTH the launcher-open and open_section_menu. Found by inspect critic 2 (which verified realism via the pickers' geometry).

## DL-cluster-a91284430601
*topic: The planned R9 check ("Config has no network/auth field") was a trybuild compile-fail denylisting specific accessor names (Config::marley().server_url() → E0599). The inspect clean-room critic show... · confidence: 0.500*

The planned R9 check ("Config has no network/auth field") was a trybuild compile-fail denylisting specific accessor names (Config::marley().server_url() → E0599). The inspect clean-room critic showed this is non-exhaustive: it only disproves the enumerated names, so a future un-listed accessor/field (endpoint(), ws_uri(), sync_host()) would pass while violating R9; and it tests accessors, whereas R9 constrains FIELDS (private, invisible to an external trybuild). Caught in inspect before the test was written. Fix: the primary R9 guard is an in-crate exhaustive destructure `let Config { app_id: _, logfile_name: _ } = Config::marley();` (and `AppId { id: _ }`) with NO `..` — E0027 forces every field to be named, so ANY added field fails to compile (a positive, exhaustive check). The accessor-denylist trybuild stays as documented non-exhaustive defense-in-depth.

## DL-cluster-afa44dedb88a
*topic: TICKET-022 design premise "emit-side fix only; decode already ships AnsiCQuoted" was empirically false: marley_terminal::decode_hook ran c_unescape over the WHOLE payload before the naive `;`/`=` f... · confidence: 0.500*

TICKET-022 design premise "emit-side fix only; decode already ships AnsiCQuoted" was empirically false: marley_terminal::decode_hook ran c_unescape over the WHOLE payload before the naive `;`/`=` field split, so the emitter's `\;` collapsed back to a bare `;` pre-split — `ls; pwd` still truncated to `ls` and a `;`-containing $PWD could inject phantom prompt fields (`…\;git=evil` → git_branch "evil"). NO emit-side encoding could ever survive that decode order (Hex has the same shape). Caught at inspect by two independent critics who ran the REAL decoder against real-zsh-emitted payloads (not just code review). Fix: R24 — split the still-escaped payload on unescaped separators first (backslash-consumes-next scanners), then c_unescape per piece; Hex/Plain byte-identical; all 69 pre-existing tests pass unmodified.

## DL-cluster-b1c24b8cfe66
*topic: #298 ⌘D (add-next-occurrence) died one occurrence short, PERMANENTLY, and told the user it was finished. The scan resumed at `set.last().end()` and treated an already-held match as proof of exhaust... · confidence: 0.500*

#298 ⌘D (add-next-occurrence) died one occurrence short, PERMANENTLY, and told the user it was finished. The scan resumed at `set.last().end()` and treated an already-held match as proof of exhaustion. But `SelectionSet::from_selections` SORTS, so `last()` is the bottom-most in DOCUMENT order — once the wrap added a cursor at the TOP, the resume offset stayed pinned at the BOTTOM forever. Every later press re-found that same held top match and early-returned. Every occurrence BETWEEN the wrapped match and the user's starting point was unreachable for the rest of the session. Fires whenever ⌘D starts on anything but the FIRST occurrence — the ordinary case (you spot the symbol mid-file). The user then types, and the skipped occurrence silently keeps the old text: a wrong edit that looks like a completed rename. Worse trigger: a single ⌘-click at end-of-file (or ⌘⌥↓) puts a bare CARET at `last()`, dragging the scan origin past every occurrence in the document — ⌘D dead on the first press. The guard had a passing test and a confident doc comment ("Once every occurrence is selected it is a NO-OP, by decision"); the premise — "the wrap returned a match we hold ⇒ we hold them all" — is simply false once the set can hold more than one member. Found independently by THREE of four critics with the gate fully green (1089 tests, clippy clean). FIX: step OVER held matches instead of bailing at the first; resume from the bottom-most RANGE, never a bare caret.

## DL-cluster-b244be271949
*topic: The #292 "rerun-last-failed" dispatch scanned workspace().states().find_map(...) and my code + comment assumed "ascending PaneId" order. But states() = self.panes.iter() where panes is a HashMap<Pa... · confidence: 0.500*

The #292 "rerun-last-failed" dispatch scanned workspace().states().find_map(...) and my code + comment assumed "ascending PaneId" order. But states() = self.panes.iter() where panes is a HashMap<PaneId, PaneState> (RandomState) → iteration order is arbitrary + per-process-randomized. With multiple terminal panes each holding a failure, find_map re-ran an ARBITRARY pane's failure (could differ across runs); the comment was factually false. My inspect self-review MISSED it (assumed states() was ordered); the background critic caught it. Single-terminal (the drive) was correct, so it was gate-green + drive-proven despite the multi-pane bug. Fix: collect candidates (skipping is_command_running panes), sort_by_key(id.0), take first → deterministic lowest-PaneId idle failed pane. Note: sibling #289 uses the same states() but is deliberately order-INDEPENDENT (union + sort/dedup).

## DL-cluster-b36bda08c4c9
*topic: SHIPPED-CODE BUG found by #312's LIVE drive (in #309's pump, shipped 3 commits earlier): EVERY textDocument/didOpen went out with an EMPTY text field, so rust-analyzer held every open file as a ZER... · confidence: 0.500*

SHIPPED-CODE BUG found by #312's LIVE drive (in #309's pump, shipped 3 commits earlier): EVERY textDocument/didOpen went out with an EMPTY text field, so rust-analyzer held every open file as a ZERO-LENGTH document and answered null to every hover (#311) and definition (#312) — the whole M20 request line was dead in the real app while all headless tests passed.

ROOT CAUSE — a Ready race between two Ready-gated calls with the collect in between. The pump did: (1) build `open_files`, materializing `buffer.text()` only `if host.needs_text(path, version)` (the #309 inspect's hot-path optimization, PR-claude-pump-materialize-hot-data-only-when-consumed-001); (2) `for host { host.drain(); host.reconcile(&open_files) }`. But a host reaches Phase::Ready INSIDE drain() — it is the initialize response that promotes it — and BOTH needs_text and reconcile early-return unless Ready. So on the exact tick the server went Ready: needs_text still saw the PRE-Ready phase → text = String::new(); drain() → Ready; reconcile → now Ready → sent that empty string as the didOpen text AND recorded the doc as synced at that version. No didChange ever follows (entry.synced == version) until the user happens to type, so the empty document is PERMANENT.

WHY NO TEST CAUGHT IT: every #309/#310/#311/#312 headless drive INJECTS synthetic responses via push_response_for_test into a process-less host, so none of them ever exercises the real didOpen→server→answer round trip. #310's and #311's "driven proof" were both headless for the same reason. #312's validate was the first time the live wire was actually inspected.

EVIDENCE (wire capture via an `[[lsp.servers]]` tee wrapper around the real rust-analyzer): BEFORE — 3 didOpens, each `TEXT_LEN=0`, whole client→server log 1226 bytes; live F12 and ⌘K both returned empty even for a SAME-FILE symbol (definition at line 18 of the very file being edited), which is the decisive tell that the server had no document. AFTER the fix — a single didOpen is `Content-Length: 16781` carrying the real source, log 104220 bytes.

FIX: drain FIRST in its own loop, then collect open_files (so needs_text reads the settled phase), then reconcile the active root's host. Keeps the hot-path optimization; removes the race.

## DL-cluster-b6b1629340fd
*topic: #324 signature-help card (a PASSIVE overlay — you type args through it) had two interaction bugs inspect caught: (1) handle_signature_key matched up/down on the KEY only, so a multi-signature card ... · confidence: 0.500*

#324 signature-help card (a PASSIVE overlay — you type args through it) had two interaction bugs inspect caught: (1) handle_signature_key matched up/down on the KEY only, so a multi-signature card swallowed ⇧↓ (shift-select) and ⌘↑/⌘↓ (doc start/end) — modified arrows meant for the editor were eaten; (2) the card was cleared only at the agent-launcher choke, so opening the palette/finder/history/find (whose key arms run AFTER the card's) left a stale card floating over them + eating ↑/↓/Esc — the #323-SELF-1 keyboard-dead class, half-fixed. Fix: gate the ↑/↓ arm on a BARE arrow (decline modified), and clear signature_card at the TOP of dispatch_action (one site covering all the overlay-open actions), not one inline site.

## DL-cluster-b78194a9d1ec
*topic: #273 validate: a FAILING assert in a #[gpui::test] that owns live TerminalSessions presented as an INFINITE HANG (SLOW >2040s), not a FAIL — the panic unwind skips the end-of-test reap_sessions, so... · confidence: 0.500*

#273 validate: a FAILING assert in a #[gpui::test] that owns live TerminalSessions presented as an INFINITE HANG (SLOW >2040s), not a FAIL — the panic unwind skips the end-of-test reap_sessions, so RootView's drop chain reaps the PTY child ON the unwinding thread and blocks. Two compounding instances: (1) my shrink test tripped the documented open+jump-in-one-frame constraint → assert panic → hang; (2) after kill -9 cleanup, ORPHANED zsh children made even the passing virgin-boot test wedge inside RootView::new_in's scope-end on-thread drop of the unused boot PTY (the #271-noted arm — now fixed off-thread in this ticket). Diagnosis path that worked: ps for the test binary (0.02s CPU = wedged), then `sample <pid>` — the stack named the exact drop site. Rule: a hung headless test is USUALLY a failed assert or an on-thread session drop; sample first, never just re-run.

## DL-cluster-b79597f2c5f7
*topic: #298's REQ-007 said ⌘D and ⌘⇧L must AGREE on what an occurrence IS. I unified their CASE (a `fold` flag on find_all) and left their OVERLAP semantics divergent — which is the half that corrupts dat... · confidence: 0.500*

#298's REQ-007 said ⌘D and ⌘⇧L must AGREE on what an occurrence IS. I unified their CASE (a `fold` flag on find_all) and left their OVERLAP semantics divergent — which is the half that corrupts data. `next_occurrence` probes EVERY index, so it returns SELF-OVERLAPPING matches ("aa" at 0 AND at 1 in "aaa"); `find_all` advances `i += ndl.len()`, so it never does. ⌘D's exhausted guard tested EQUALITY (`start == && end ==`), but SelectionSet's invariant merges on OVERLAP — so a match overlapping a held member without equalling it sailed through the guard, was pushed, and was UNIONED into that member by from_selections. The user's "aa" silently became "aaa": text they never selected, AND the needle for every subsequent press. Reachable with the KEYBOARD ALONE (⇧→⇧→ over an indent run selects two spaces, and "  " self-overlaps inside a 4-space indent) — no mouse, no exotic input. Payload: typing over the resulting ⌘D set yields "XbX" where the user's two selections should give "abXbX" — it eats two characters they never targeted. A MOTION, so ⌘Z cannot restore the corrupted gesture state. FIX: ONE engine (find_all) for both gestures, plus a candidate must survive the invariant AS ITSELF or it is not addable. `next_occurrence` then had no production caller and was DELETED (§0), same as its sibling select_next_match.

## DL-cluster-ba39ee5b2505
*topic: TICKET-348's bounded PTY-teardown loop polled the child via `poll_child_exit` → alacritty `next_child_event`, which is EDGE-triggered: it reads ONE SIGCHLD self-pipe byte and returns None on WouldB... · confidence: 0.500*

TICKET-348's bounded PTY-teardown loop polled the child via `poll_child_exit` → alacritty `next_child_event`, which is EDGE-triggered: it reads ONE SIGCHLD self-pipe byte and returns None on WouldBlock without calling try_wait. The pump ALSO calls poll_child_exit every tick and reaps a normally-exited shell there, consuming that byte. So a child reaped during normal use was INVISIBLE to a later teardown poll → the Drop loop ran the full HUP(2s)+KILL(2s)=4s and then GiveUp-leaked the pty. Two harms on the common "shell exited, then close the tab" path: a 4s teardown freeze + an fd leak, and it SIGHUP/SIGKILLed a reaped pid the OS may have RECYCLED (signalling an unrelated process). Caught by the reap-logic critic's observation (the pump reaps + consumes SIGCHLD) + my verification (session.rs:372 pump call; alacritty unix.rs:387-391 WouldBlock→None). Fixed at source with an OsPtyChannel.child_reaped latch set in poll_child_exit (the only time the edge is observable) and read in Drop: already-reaped → skip signals+loop → ManuallyDrop::drop (alacritty reaps from the cached status instantly). All in the mutants::skip + cov-excluded shim (no cov/MSI change).

## DL-cluster-bcdc5b086b50
*topic: #321 moved LSP host creation from a gesture-triggered call (fires once, when a user opens a file) to the pump tick (fires every frame). The gate's clauses were hoisted but their ORDER was carried o... · confidence: 0.500*

#321 moved LSP host creation from a gesture-triggered call (fires once, when a user opens a file) to the pump tick (fires every frame). The gate's clauses were hoisted but their ORDER was carried over unexamined: `if self.lsp_hosts.contains_key(root) || !root.join("Cargo.toml").is_file() { return; }`. The second clause is a filesystem stat. On the old path that stat ran only when a `.rs` file was actually opened; on the new path it ran on EVERY pump tick, forever, for any project without a host. Correct behavior, wrong cost — and invisible to every gate (no test asserts syscall counts, and coverage/mutation say nothing about it). Fixed by reordering cheapest-first: `contains_key` (in-memory, permanently retires the question once a host exists) → the `.any()` path scan (in-memory, a few open docs) → the manifest stat LAST, reached only when a candidate `.rs` doc exists. The stat deliberately stays inside the per-tick path rather than being cached, because re-evaluating it is what lets a root that gains a `Cargo.toml` later (a `cargo init` in an open project) get language support on the next tick — so it cannot be hoisted out, which is precisely why it must come last. Found by my own equivalence analysis during inspect; neither dispatched critic returned in time.

## DL-cluster-bd86bee23de0
*topic: #376 inspect F2 (HIGH): the fleet subscription start-gate ran at the boot position where project_root is the launch-cwd DISCOVERED root — but the shell restore ~700 lines later explicitly re-syncs ... · confidence: 0.500*

#376 inspect F2 (HIGH): the fleet subscription start-gate ran at the boot position where project_root is the launch-cwd DISCOVERED root — but the shell restore ~700 lines later explicitly re-syncs project_root to the restored ACTIVE project. Consequences: a Finder/Dock launch (cwd=/ or $HOME) never subscribes even though the restored workspace IS configured (the common non-dev launch); launching from configured project A with restored-active B renders A's live feed under B; a launcher boot spawns a subscription thread for an unopened workspace. Fix: moved the start block BELOW the shell build, gated on project_count() > 0, keyed shell.active_project().root. The launch-cwd root is PROVISIONAL until the restore.

## DL-cluster-c0b1c07daf7f
*topic: marley_terminal inspect: pump/ingest rendered the WHOLE read's passthrough bytes (via DcsScanner::feed's separate `out` param) BEFORE applying that read's decoded DCS hooks. PTYs COALESCE writes, s... · confidence: 0.500*

marley_terminal inspect: pump/ingest rendered the WHOLE read's passthrough bytes (via DcsScanner::feed's separate `out` param) BEFORE applying that read's decoded DCS hooks. PTYs COALESCE writes, so a fast command's `[Preexec]output[Precmd]` routinely fits one ≤4096B read. Processing order was: render `output` while the block isn't open yet → set_current_output is a no-op (current() is None) → OUTPUT DROPPED; then apply Preexec → reset term + open an EMPTY block; then Precmd → finish it → a Finished block with output_text()=="". Breaks R4 and makes the R16 `echo hi` integration test FLAKY (passes only when reads happen to split between preexec and output — exactly the timing-dependent blank-Block race the spike's no-backoff bug warned of, but ORDERING-flavored: the bytes and the hooks lost their interleaving when the scanner flattened them into two separate channels. Fix: DcsScanner::feed emits an ORDERED Vec<DcsEvent{Passthrough(bytes)|Hook(raw)}> preserving byte-stream order; ingest processes in order so Preexec resets+opens the block BEFORE the following passthrough (the command output) is rendered into it. Caught by an inspect critic tracing a coalesced read, NOT by check/clippy (it compiles fine).

## DL-cluster-c0d9308952e3
*topic: #313's completion accept applied the server's REQUEST-TIME `textEdit` range to a buffer that had been typed into since — `s.` → popup opens → type `p` → Enter produced `s.pushp`, stranding the `p` ... · confidence: 0.500*

#313's completion accept applied the server's REQUEST-TIME `textEdit` range to a buffer that had been typed into since — `s.` → popup opens → type `p` → Enter produced `s.pushp`, stranding the `p` after the insert. REQ-005 ("replacing the typed prefix, not appending after it") failing on the feature's single most common gesture.

WHY IT SURVIVED THE DESIGN: the popup deliberately SURVIVES typing — a word char re-filters the server's snapshot (no round trip, the #96 idiom) and re-stamps the recorded buffer version, which is exactly what stops the dismiss poll from dropping it. But the items — and their ranges — are never re-fetched. The install-time guard (`version == key.version`) makes the ranges exact only at install, and nothing re-checked them at accept.

WHY IT WOULD HAVE SHIPPED: (1) the `range: None` fallback arm was CORRECT (it uses the live caret), so the bug fires only when the server actually sends a `textEdit` — i.e. always with rust-analyzer, never in a hand-written fixture; (2) the design's own planned test ("type `pu`, accept `push`") accepts IMMEDIATELY after the popup opens, so it would have passed green straight over the defect. A test that never types between open and accept cannot see this class at all.

FIX: extend the server range to the LIVE caret — `(a.min(b).min(caret), a.max(b).max(caret))`. LSP requires the range to contain the request position, so the client owns everything typed since; min/max also normalizes an inverted range. Phase 4 must add a drive that TYPES between open and accept.

## DL-cluster-c0e6ac3e5327
*topic: disambiguate_labels (M14 #241) can RE-CREATE a duplicate when a sibling is ALREADY literally named "{base} {n}": ["Marley","Marley 2","Marley"] → ["Marley","Marley 2","Marley 2"] (the 3rd "Marley" ... · confidence: 0.500*

disambiguate_labels (M14 #241) can RE-CREATE a duplicate when a sibling is ALREADY literally named "{base} {n}": ["Marley","Marley 2","Marley"] → ["Marley","Marley 2","Marley 2"] (the 3rd "Marley" becomes "Marley 2", colliding with the pre-existing "Marley 2"). Cosmetic ONLY — rail click routing is by the (project,tab) index, not the label, so nothing mis-switches; it just degrades to the pre-feature ambiguous state for that one pair. Needs a sibling literally named "Marley 2" (a dir named that, or a command token reading so) — very low likelihood. Deferred with a doc note (the in-code fn-doc caveat was phase-gate-blocked as I'd set Inspect-PASS early; documented in the pipeline notes instead). Airtight fix: number against the full set of input labels (bump n until format!("{label} {n}") isn't already present). Found by the inspect critic (which CONCURRED with an otherwise-thorough self-review).

## DL-cluster-c1378c2858bf
*topic: The #244 workspace-switcher indicator (a TOP-BAR MOUSE-CLICK overlay with a full-screen inset_0 backdrop) reset only the obvious transient overlays (context_menu/agent_launcher/palette_open/naming_... · confidence: 0.500*

The #244 workspace-switcher indicator (a TOP-BAR MOUSE-CLICK overlay with a full-screen inset_0 backdrop) reset only the obvious transient overlays (context_menu/agent_launcher/palette_open/naming_workflow/renaming_tab) and MISSED the keyboard-opened no-backdrop cards finder_open/history_open/find_open + the completion popup. A mouse click bypasses the key-router's overlay-dismiss guards (routing still sends keystrokes to those overlays), so opening the switcher left a keyboard-live finder/history/find/completion invisible-but-still-typing UNDER the switcher backdrop — the #229 orphan class. Caught in inspect (my self-review found it; the critic independently confirmed it). Fixed by mirroring the SHIPPED new-agent top-bar arm (app.rs:3319-3332) which already closes palette/finder/history/find/naming for exactly this reason. Severity low (Escape-recoverable, uncommon path, no data loss). Class: a mouse-opened backdrop overlay must close the SAME set the analogous mouse-opened precedent (new-agent) closes — grep the precedent, don't re-derive the list from the overlays you first think of.

## DL-cluster-c204c353bd4f
*topic: The bracketed-paste injection guard used a single `text.replace("\x1b[201~", "")` to strip embedded end-markers before wrapping a paste in ESC[200~..ESC[201~. `str::replace` does ONE non-overlappin... · confidence: 0.500*

The bracketed-paste injection guard used a single `text.replace("\x1b[201~", "")` to strip embedded end-markers before wrapping a paste in ESC[200~..ESC[201~. `str::replace` does ONE non-overlapping left-to-right pass and does NOT re-scan its output, so a split marker reconstitutes: input `ESC[20` + `ESC[201~` + `1~` → removing the middle marker splices the neighbours into a fresh `ESC[201~`. That interior end-marker closes the bracket early, and the bytes after it run as typed input — a working paste-injection (clipboard `ESC[20 ESC[201~ 1~ rm -rf ~\n` → `rm -rf ~` submitted). Caught by the inspect critic with a PoC probe. FIX: loop until stable — `while safe.contains("\x1b[201~") { safe = safe.replace("\x1b[201~", ""); }` (each pass strictly shrinks → terminates; verified 0 residual on split/nested/adjacent/1000x-stress inputs). Also MED: cargo-mutants generates only whole-fn-return mutants on paste_bytes (none on the strip), so MSI 100 is blind to the guard — a hand-written split-marker structural regression test is required (the gate can't force it).

## DL-cluster-c221fba2a403
*topic: Inspect F1 (all 3 critics): the hover card's dismiss-on-caret-move + the response stale-guard were hung off the action dispatcher (dispatch_action guard), but plain arrows / ⌘-arrows / a mouse rail... · confidence: 0.500*

Inspect F1 (all 3 critics): the hover card's dismiss-on-caret-move + the response stale-guard were hung off the action dispatcher (dispatch_action guard), but plain arrows / ⌘-arrows / a mouse rail-click tab switch / the uniform_list's own scroll are handled INLINE and never reach dispatch_action — so pressing an arrow left the card pinned at a stale cell (REQ-004 fail) and an in-flight response was shown at a moved caret (REQ-005 fail); two unedited files both report version 0 / first-row 0 so a version+scroll-only poll couldn't tell them apart. Fix: HoverCard carries the (file path, caret, version, first-row); poll_hover_dismiss compares the LIVE editor identity each pump tick and dismisses on ANY change; consume drops a response whose key.uri no longer names the focused file. Prevention rule PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001.

## DL-cluster-c28711818a52
*topic: #212's initial implement kept #196's clickable link range = the PATH portion only (at..at+path.len()). But #212 makes a file:line:col ref clickable-to-open-at-that-line — with a path-only range, cl... · confidence: 0.500*

#212's initial implement kept #196's clickable link range = the PATH portion only (at..at+path.len()). But #212 makes a file:line:col ref clickable-to-open-at-that-line — with a path-only range, clicking the :12:5 line/col portion of `foo.rs:12:5` does NOTHING (only the filename opens). The affordance over a compound token must cover the WHOLE token. Fixed in inspect: range = at..at+trimmed.len() (the full ref minus leading bracket + trailing punctuation); unchanged for a location-less ref (trimmed==path), so #196 stays green.

## DL-cluster-c5da8dedf072
*topic: #331. `refresh_inlay_hints` retried on "the cache cannot serve this viewport". An Ok(null)/Ok([]) answer caches and terminates; an Err writes NO cache, so `served` stays false and the request re-fi... · confidence: 0.500*

#331. `refresh_inlay_hints` retried on "the cache cannot serve this viewport". An Ok(null)/Ok([]) answer caches and terminates; an Err writes NO cache, so `served` stays false and the request re-fires every round-trip, forever. Two ways to earn a guaranteed Err, both live: (1) NO LANGUAGE GATE — `language_id_for` is .rs-only, so a non-Rust file (Cargo.toml) is never didOpen'd, yet the request was still sent for it; (2) `inlay_hint_support` was written, re-exported, and CALLED NOWHERE — dead code — so the request went to every Ready server regardless of `inlayHintProvider` → MethodNotFound. Both were guarded only by server behaviour, not by code. Fix: gate on `language_id_for(&path).is_some()` (reuse the ONE table docsync syncs by, never a second copy) AND on the advertised capability (the #324 posture); clamp the requested range to `len_lines` too. LESSON: a retry loop keyed on "no cached answer" MUST make every terminal outcome write something, or an error path becomes an infinite loop — and a capability reader that nothing calls is a claim the code doesn't keep.

## DL-cluster-c65e2dac0a3c
*topic: #259 editable split pane: the FOCUSED editable pane rendered BLANK (no text/gutter/caret) — you'd type blind. code_view_body's Some-arm uniform_list row closure is 'static (can't borrow the view), ... · confidence: 0.500*

#259 editable split pane: the FOCUSED editable pane rendered BLANK (no text/gutter/caret) — you'd type blind. code_view_body's Some-arm uniform_list row closure is 'static (can't borrow the view), so it re-reads the buffer live via entity.read(app).shell.active_project().active_tab().editor(). But a split code pane lives INSIDE a Terminal tab, and Tab::editor() is None for a Terminal → the closure hit `else { break; }` on ROW 0 → zero rows, while fold_projection (the focus-aware active_editor()) reported the pane's real line count → full scroll height + blank body. TWO root causes: (1) the G4 blanket regex that moved ~43 `active_tab().editor()` bypassers onto the focus-aware active_editor() MISSED this one because its receiver was `entity.read(app)`, not the `self`/`view` the regex matched; (2) a shared render helper's 'static re-read closure diverged from the code that SIZED the list (fold_projection used active_editor(), the closure used active_tab().editor()). FIX: the closure reads entity.read(app).active_editor() (the same focus-aware resolver). Caught by an inspect critic tracing the render data-sources, not by compile (active_tab().editor() is valid, just semantically None) and not by a locked-screen validation (input/geometry were mechanism-verified; the row RENDER was not). LESSON: after a blanket accessor-move, grep the pattern with ANY receiver + multi-line; and a shared render closure that re-reads state must use the SAME resolver as whatever sized/gated it.

## DL-cluster-c6f8f22de090
*topic: #327 problems panel: the selection-keep-by-identity across a live diagnostics refresh keyed on (path, line) only. Two diagnostics can share a line (a clippy Error + a Warning), and because the list... · confidence: 0.500*

#327 problems panel: the selection-keep-by-identity across a live diagnostics refresh keyed on (path, line) only. Two diagnostics can share a line (a clippy Error + a Warning), and because the list sorts severity→path→line they sit far apart; keep_selection_by_identity's .position() returned the FIRST (path,line) match, so a refresh teleported the selection to the wrong (higher-severity) row (e.g. index 30 → 3). Bounded (lands on a real diagnostic, no crash) but undercut the 'identity' guarantee the doc advertised. Caught by inspect critic 2 (state) — it was the only un-noted gap; the doc even claimed identity-keeping. Fixed: extended the identity key to (path, line, character, severity) — the full tuple distinguishes same-line diagnostics (they differ in severity and/or character).

## DL-cluster-c9ac3b8ae504
*topic: marley_spike VALIDATE: the headless PTY integration test (R3/R4/R5) caught a THIRD real bug that inspect + check + clippy all missed — drive_reader's Retry branch decremented the poll budget with N... · confidence: 0.500*

marley_spike VALIDATE: the headless PTY integration test (R3/R4/R5) caught a THIRD real bug that inspect + check + clippy all missed — drive_reader's Retry branch decremented the poll budget with NO inter-poll delay, so max_polls=2000 exhausted in ~1.4ms, BEFORE the freshly-forked /bin/sh child wrote (~6ms). It returned ShutdownOutcome::Clean with an EMPTY grid (read 0 bytes). Proven: a sleep-drain read 'hello-marley' after 6.4ms; drive_reader(2000) returned row0='' in 1.4ms; drive_reader(50_000_000) waited ~4.2ms and DID capture it. Impact wider than the test: app::run calls the same drive_reader(2000) before opening the window → the real app would render a BLANK Block body (violates R5/R9/R10). Root cause: the proven spikes (ptyprobe/holdprobe) BOTH thread::sleep per WouldBlock; the productionized drive_reader dropped that backoff. Fix: a fixed ~1ms std::thread::sleep on the Retry branch (the budget still bounds termination; mock-reader unit tests return instantly so they're unaffected; the budget-exhaust mutants still infinite-loop → cargo-mutants timeout). The unit suite (mock reader, no real timing) could NOT catch this — only the real-PTY integration test did. This is the 3rd bug (after the Pty-drop + the ExitCode opacity) that a real end-to-end test caught past green check/clippy/unit.

## DL-cluster-ca08faaecf4e
*topic: TICKET-023: closing a pane dropped TerminalSession inline on the gpui main thread; alacritty's Pty::drop SIGHUPs then BLOCKS in child.wait() with no timeout — MEASURED 603ms UI freeze closing a pan... · confidence: 0.500*

TICKET-023: closing a pane dropped TerminalSession inline on the gpui main thread; alacritty's Pty::drop SIGHUPs then BLOCKS in child.wait() with no timeout — MEASURED 603ms UI freeze closing a pane running a foreground child, and unbounded (force-quit territory) for a HUP-immune child (trap '' HUP); window-close hung the same way. Fix: Workspace::close(pane) RETURNS the removed PaneState so the CALLER owns teardown context; the app drops it on a detached reaper thread; PtyChannel gained a Send supertrait so TerminalSession moves cross-thread (compiler-verified).

## DL-cluster-cc478832221d
*topic: M22 #337. Inspect finding F5 was "settings.rs's font_family doc promises a status flash that does not exist". I wrote the finding, wrote the resolution ("The settings doc now describes what the cod... · confidence: 0.500*

M22 #337. Inspect finding F5 was "settings.rs's font_family doc promises a status flash that does not exist". I wrote the finding, wrote the resolution ("The settings doc now describes what the code does: ... degrades to gpui's system fallback SILENTLY"), and NEVER APPLIED THE EDIT. The defective sentence — "An unresolvable family falls back to the built-in WITH a flash — never a silent wrong font" — was still in settings.rs verbatim at Phase 4, having survived Implement, Inspect's own post-fix re-verification, and a full GATE GREEN [diff].

WHY EVERY CHECK MISSED IT: the fix was DOC-ONLY. `cargo check`, `clippy -D warnings`, 615 nextest tests, `cargo fmt`, the §20 brand grep, coverage 100%, mutation MSI 100 and rustdoc -D warnings are all blind to a doc comment's CONTENT. A false doc is well-formed Rust. There is no gate for "this sentence is a lie", so the only thing standing between a wrong doc and the user is a human re-reading the file — and I substituted the LEDGER ENTRY for that re-reading.

COMPOUNDING: the reason F5 recorded was ALSO false. Both docs claimed the flash "needs a font-resolution probe this path lacks"; gpui exposes `TextSystem::font_id(&Font) -> Result<FontId>` (text_system.rs:109) and the mac backend `?`s `load_family`, so the probe exists and the cut is scope, not capability. One finding thus produced a fix that never landed AND a justification that was never true, with the ledger asserting both were handled.

CAUGHT BY: re-reading the actual source at Phase 4 while checking REQ-006 against my own tests rather than against my memory of them. An audit of the other five findings (F1/F2/F3/F4/F6) confirmed they DID land — so the failure is not sloppiness in general, it is specifically that a doc-only fix has no verifier.

This is the same class as the ticket's other two: #336 F5 (a correct fix citing `element_offset` when taffy was the carrier) and #337 F3 (a correct fix whose comment had the physics backwards). A plausible reason/record stops the review, so the reason/record never gets measured.

## DL-cluster-cd895fd9ad34
*topic: #312 nav_back (⌃- jump-back) fed the RAW popped NavStack offset to Buffer::line_col, which is an unguarded rope.char_to_line — ropey PANICS when char_idx > len_chars, killing the app on the gpui UI... · confidence: 0.500*

#312 nav_back (⌃- jump-back) fed the RAW popped NavStack offset to Buffer::line_col, which is an unguarded rope.char_to_line — ropey PANICS when char_idx > len_chars, killing the app on the gpui UI thread. set_single_caret was safe (it funnels to set_selection → clamp_char_offset), so the CARET was clamped but the ROW read was not. The trap: the sibling accessor Buffer::line_start two lines below DOES clamp and documents "never panics", so the codebase treats clamping as opt-in per-accessor — line_col did not opt in, and the code assumed the neighbourhood was uniformly safe. Reachable by ordinary use because EditorSurface::open PRESERVES an already-open buffer (M15 #249): F12 → jump → return → delete a block → ⌃- pops a now-past-EOF offset → crash. Found by 3 of 5 inspect critics independently (two reproduced it against real ropey 1.6.1). FIX: the shared open_and_place_caret reads the row back from active_caret() AFTER the clamp, never from the requested offset. Lead-verified by probe: raw 4000 → ropey panics; clamped → CharOffset(11) → row=1, with 11 == len_chars proving the exact boundary is in-bounds.

## DL-cluster-cf814d82044d
*topic: #390 un-nested split-pane cell rows from under their terminal tab into a new Panes rail section, adding a new RailLevel::Arrangement render arm (app.rs). The sibling RailLevel::Pane arm hides itsel... · confidence: 0.500*

#390 un-nested split-pane cell rows from under their terminal tab into a new Panes rail section, adding a new RailLevel::Arrangement render arm (app.rs). The sibling RailLevel::Pane arm hides itself during the "Search tabs" filter (`if !self.session_filter.is_empty() { continue; }`) because its generic "pane N" labels never match a title query — but the new Arrangement arm (whose labels "PANE n" are equally generic) was written WITHOUT that guard. Result: during a tab-search, the Panes section rendered dangling "PANE 1"/"PANE 2" headers with no (hidden) cells beneath — a cosmetic self-inconsistent regression. Found INDEPENDENTLY by BOTH inspect critics. Fix: added the identical `if !self.session_filter.is_empty() { continue; }` guard to the Arrangement arm, leaving only the Panes header during a filter (consistent with the E/T/B headers persisting when their lists filter empty). Class: a new/relocated render arm for a row must mirror the render-time guards (search filter, visibility) of the sibling row it parallels.

## DL-cluster-d0172e2f8797
*topic: #330: `collect_headers` (the all_headers full-tree walk) was written as plain recursion over `node.children()`, with a doc comment claiming "Rust nesting depth is bounded (tree-sitter caps parse re... · confidence: 0.500*

#330: `collect_headers` (the all_headers full-tree walk) was written as plain recursion over `node.children()`, with a doc comment claiming "Rust nesting depth is bounded (tree-sitter caps parse recursion), so plain recursion is safe" — FACTUALLY WRONG. The recursion depth is the FULL tree depth (every nested block/paren/expression level), not the header nesting: a probe showed one `fn` wrapping 2000 nested blocks recurses 2000 deep to find ONE header. Measured on the 8 MiB macOS main stack: survives 12k levels, OVERFLOWS and SIGABRTs at 16k (2k-5k on a smaller test-thread stack). That is an UNCATCHABLE abort (not catch_unwind-able), it runs SYNCHRONOUSLY on the gpui UI thread, and it re-runs on EVERY edit (sticky headers default ON) — so a machine-generated/minified/adversarial Rust file would take down the whole app (every open terminal + editor). tree-sitter's own parser is iterative precisely because of this. The sibling #329 enclosing_ranges correctly climbs iteratively via .parent(); this new walk was the crate's only deep-recursion path. Caught by inspect critic 1 with measured stack ceilings. Fixed: rewrote as an iterative TreeCursor preorder walk (goto_first_child/goto_next_sibling/goto_parent, O(1) stack, identical document order).

## DL-cluster-d3b7e6d3031f
*topic: StandardizedPath used `#[non_exhaustive]` on the ENUM to satisfy R13 ("no external crate can construct a variant directly or match it exhaustively"). Enum-level #[non_exhaustive] blocks external EX... · confidence: 0.500*

StandardizedPath used `#[non_exhaustive]` on the ENUM to satisfy R13 ("no external crate can construct a variant directly or match it exhaustively"). Enum-level #[non_exhaustive] blocks external EXHAUSTIVE MATCHING (must add a wildcard) but does NOT block external CONSTRUCTION of a known variant — tuple-variant fields are public, so a downstream `StandardizedPath::Posix(s)` would compile, bypassing the canonical invariant (and is_absolute() would then lie about a hand-built value). The correctness critic marked R13 "structurally ✓"; the gap was caught only when the validate phase wrote the trybuild compile-fail and it did NOT fail as expected. Fix: put `#[non_exhaustive]` on each VARIANT (not just the enum) — that blocks both external construction (E0603 "cannot be constructed because it is #[non_exhaustive]") AND non-wildcard matching. Verified by the trybuild .stderr.

## DL-cluster-d5d7265377e8
*topic: #282 grouped-undo: the new `begin_group` cleared the redo stack EAGERLY on transaction open, but the app's Tab/⇧Tab indent arm opened the group before knowing whether `edits` was empty. A no-op ⇧Ta... · confidence: 0.500*

#282 grouped-undo: the new `begin_group` cleared the redo stack EAGERLY on transaction open, but the app's Tab/⇧Tab indent arm opened the group before knowing whether `edits` was empty. A no-op ⇧Tab on a flush-left line (dedent_edits → []) therefore destroyed a pending redo stack while making zero edits — a regression (pre-#282 the no-op ran no `record`, so redo survived) that also violated the project's own `ime.rs "redo survives the no-op"` invariant. Both inspect critics found it independently (one reproduced it in a throwaway crate: "REDO LOST"). Fix: moved the redo-clear out of `begin_group` into `end_group`'s non-empty COMMIT branch, so redo is invalidated only when a real group commits — protecting all callers, not just the two that happened to guard the empty case. Regression test: undo::an_empty_group_preserves_the_redo_stack.

## DL-cluster-d734e970258f
*topic: exit_status_kind(Finished, None) = Failure → the ✗ glyph, but the shim's `status_line = match d.exit_code { None => "running" }` read only the exit_code, so a signal-killed / no-code-captured Finis... · confidence: 0.500*

exit_status_kind(Finished, None) = Failure → the ✗ glyph, but the shim's `status_line = match d.exit_code { None => "running" }` read only the exit_code, so a signal-killed / no-code-captured Finished block rendered ✗ (failed) beside the word "running" — a glyph/text contradiction. Uncaught by every gate (app.rs is coverage-excluded + render is mutants::skip) AND by the self-test (which drove a clean exit-0 command). The inspect critic found it via a probe on the Finished+None case. Fix: derive the line from BOTH status and exit_code — `(_, Some(c)) => "exit {c}", (Running, None) => "running", (_, None) => "no exit code"`. Lesson: when two rendered elements (a glyph and its text label) describe the same state, derive BOTH from the SAME source (the status class), never from a lower-level field one of them ignores — else they can disagree on an edge case no test drives.

## DL-cluster-d9926388169b
*topic: #305 code folding: auto-reveal (unfold the caret's containing fold on navigation) was hooked ONLY in follow_editor_caret (via reveal_caret_row). But the JUMP commands — go-to-line (goto_commit), sy... · confidence: 0.500*

#305 code folding: auto-reveal (unfold the caret's containing fold on navigation) was hooked ONLY in follow_editor_caret (via reveal_caret_row). But the JUMP commands — go-to-line (goto_commit), symbol-jump (jump_to_file_symbol), go-to-def/CmdT/Ctrl-minus/diagnostics-jump (open_and_place_caret to consume_pending_center), and FIND (CmdG, select_efind_current) — place the caret then call scroll_editor_to_row DIRECTLY, bypassing follow_editor_caret. scroll_editor_to_row does slot_of(row) (which snaps a hidden row to the folded header's slot) + scroll, with NO reveal — so a jump INTO a folded body centers the folded header while the caret sits invisibly inside the fold, and typing then edits hidden text. This is REQ-006's own scenario. Both inspect critics + the implementer's independent trace confirmed it; the find (select_efind_current) site was caught ONLY by the implementer's trace (a critic missed it). The design's D-AUTO-REVEAL-ON-CARET misidentified follow_editor_caret as THE shared chokepoint — the jumps funnel through scroll_editor_to_row, which cannot blanket-reveal because goto_preview scrolls with the caret parked at the origin. Fix: a reveal_and_scroll_to_row(row) helper (remove_folds_at_row(path,row,hidden_only=true) + scroll_editor_to_row(row)) wired at the 4 in-scope bypass sites; goto_preview + CmdD (out of REQ-006 scope) left. Would have shipped GREEN if Phase 4's REQ-006 drive reached for an arrow motion (which reveals) instead of a named jump verb.

## DL-cluster-d9a11007fb12
*topic: Adding gpui (Marley's UI framework, ~700-crate tree) to a crate made `cargo deny check` FAIL the supply-chain gate (gate:8) in two ways the per-crate clippy/check did not surface: (1) LICENSES — gp... · confidence: 0.500*

Adding gpui (Marley's UI framework, ~700-crate tree) to a crate made `cargo deny check` FAIL the supply-chain gate (gate:8) in two ways the per-crate clippy/check did not surface: (1) LICENSES — gpui's renderer pulls CC0-1.0 (hexf-parse via naga/blade-graphics, tiny-keccak via ahash) and the `image` crate's AVIF path pulls NCSA (libfuzzer-sys via rav1e/ravif); none were on the MIT/Apache/BSD/MPL allowlist. Pruning `image` to png-only (default-features=false) did NOT drop NCSA because image-compare keeps the AVIF feature via cargo feature-unification. (2) ADVISORIES — `cargo deny check` (run with no args = all checks) flagged 5 gpui-transitive UNMAINTAINED RUSTSEC notices (async-std/instant/paste/proc-macro-error2/rustls-pemfile) as errors, even though `cargo audit` (gate:7) treats them as exit-0 warnings. Fix: allow CC0-1.0 + NCSA (permissive, gpui-intrinsic) and add the 5 RUSTSEC IDs to deny.toml [advisories] ignore with documented reasons (the §0-sanctioned per-id reasoned ignore, not a silent baseline). Caught by an inspect critic running `cargo deny check` directly; would otherwise have failed the FULL gate at validate.

## DL-cluster-d9d486c03076
*topic: M22 #339. My Phase 2 file manifest listed "the two chips + the inline error row on the find bar". I built the STATE (efind_regex/efind_fold/efind_error), the mode fork in refresh_efind_matches, and... · confidence: 0.500*

M22 #339. My Phase 2 file manifest listed "the two chips + the inline error row on the find bar". I built the STATE (efind_regex/efind_fold/efind_error), the mode fork in refresh_efind_matches, and the palette toggles — then wrote "status: Phase 3 — Implement PASS" WITHOUT building the render half AND without recording the cut. Both critics caught it independently, from opposite lenses (the app-wiring critic by grep; the offset-seam critic by tracing the render).

THE USER-VISIBLE CONSEQUENCE: `efind_error` had FOUR writes and ZERO reads. With regex mode on and an invalid pattern `a[`, the bar rendered "a[ (no matches)" — telling the user their pattern found nothing when it never COMPILED, so they hunt the document instead of fixing the bracket. REQ-003 ("surface an invalid pattern as an inline bar error") was two-thirds met (no panic, find-next no-ops) and the actual surfacing — the point of the REQ — was missing. Separately, the two chips had no render at all, so the mode was invisible: the palette toggle changes what a match IS with zero visual feedback.

WHY NO GATE CAUGHT IT — the part that makes this a class, not a slip. `efind_error` is a PRIVATE field on a pub struct, INITIALIZED in the struct literal and only ever ASSIGNED. rustc 1.96 does NOT emit dead_code for that shape (a struct-literal init marks the field live for the dead-code pass). Both critics verified with a minimal repro: zero warnings. And app.rs is coverage-excluded + the shims are mutants::skip. So a write-only field that renders a lie is invisible to clippy, coverage, AND mutation simultaneously. This is the exact "state whose contract nothing enforces" class the batch keeps flagging (#338's tag, #339's memo key) — here it reached a shipped user-facing lie.

THE ROOT PROCESS FAILURE: I did not check my own Phase-3 output against my own Phase-2 manifest. The manifest is a checklist; I treated "PASS" as a feeling rather than a diff against the list I wrote three phases earlier. A UI ticket especially: state without render is not a feature, it is dead state, and "it compiles and the units pass" says nothing about whether a pane WORKS — which is the whole reason Validate mandates a driven pixel check.

FIX: built the inline error row (danger-colored, FindError::Display) that overrides the "(no matches)" label, plus a ".* "/"Aa " mode prefix so the chips are legible. efind_error now has readers. 1518 tests still pass; clippy clean. The RENDER must be driven-verified at Validate (deferred here — chad is at the machine, synthetic input off-limits).

Also in this review, 5th time this batch: my REASONS were wrong even where the code was right. The toggle_find_regex doc credited "efind_key = None" as the mechanism (it is belt-and-braces — the flags-in-key already invalidate); the toggle_find_case doc claimed only the regex chip can affect the error (FALSE — case_insensitive(true) expands classes and can cross REGEX_SIZE_LIMIT, so the case chip creates/clears FindErrors too). Both corrected.

## DL-cluster-db05a3eab018
*topic: #184: inserting a new fn `block_line_counts` immediately ABOVE `content_rows` placed it between content_rows's pre-existing `#[cfg_attr(test, mutants::skip)]` and content_rows itself — so the attri... · confidence: 0.500*

#184: inserting a new fn `block_line_counts` immediately ABOVE `content_rows` placed it between content_rows's pre-existing `#[cfg_attr(test, mutants::skip)]` and content_rows itself — so the attribute (plus a stale doc comment and a duplicate skip I added) all bound to the NEW fn, silently un-skipping content_rows. content_rows is a coverage-excluded gpui shim with no unit test, so its 4 mutants became unkillable → MSI < 100. Invisible to cargo check + cargo fmt (both pass — attributes/docs are syntactically fine wherever they sit). Caught by the inspect mutation critic via cargo mutants --list. Fixed by reordering so each fn owns its own doc + skip.

## DL-cluster-dc0a0ca60389
*topic: #399: the coverage gate failed on ONE "missed line" in a fully-tested pure module that NO per-line view could locate — lcov DA records all nonzero, the annotated text report showed no zero region, ... · confidence: 0.500*

#399: the coverage gate failed on ONE "missed line" in a fully-tested pure module that NO per-line view could locate — lcov DA records all nonzero, the annotated text report showed no zero region, and a region-level cross-reference of live vs dead function records found no exclusively-dead line. Root cause: the module's generic fns (index_or_push<T>, resolve_arrangement's impl FnMut param) get instantiated + inlined into every LINKING test binary's rlib covmap; the never-run copies in non-calling binaries (the integration-test binary) feed llvm-cov's line-summary merge an arithmetic phantom. Two diagnostic traps compounded it: `--show-missing-lines` changes llvm-cov's exit pathway (the same invocation PASSES with the flag, FAILS without — a masked reproduction), and ad-hoc `cargo llvm-cov` runs without `clean` merge stale profraws across builds. Fix at the root: the cov-100 pure module is now generic-free (`&mut dyn FnMut` closures, inline find-or-push) — one compiled body per fn; documented in the module and matching the gate's own syntax/parse.rs phantom-class note. Cost: three red gate cycles.

## DL-cluster-dc58685c19e9
*topic: #272 inspect (HIGH, probe-EXECUTED): the find bar's match set refreshed only in the render path, but key handlers run per event — two keys landing between frames (Enter autorepeat live; the headles... · confidence: 0.500*

#272 inspect (HIGH, probe-EXECUTED): the find bar's match set refreshed only in the render path, but key handlers run per event — two keys landing between frames (Enter autorepeat live; the headless lane dispatches ALL keystrokes then parks once) made the second Replace consume PRE-EDIT offsets: Buffer::edit → rope.remove out-of-range PANICKED, and the in-bounds variant silently ate user text ("aaa bbb" → "xbb"). The #268-class render-memo staleness is COSMETIC for highlights but CORRUPTING the moment the cached ranges feed edits. Fix: the handler re-runs the memo refresh at its top (no-op on an unchanged (nonce, version, query) key — zero fresh-path cost). Rule: any cached range set that an EDIT path consumes must be revalidated IN the handler, not left to the next frame.

## DL-cluster-dcc6259b0ed3
*topic: #373 inspect F1: the auto-reconnect loop's run_once rebuilt a fresh EMPTY FleetSync on every connection cycle. Because the proposed fleet://events?since=<cursor> contract is a post-cursor DELTA (re... · confidence: 0.500*

#373 inspect F1: the auto-reconnect loop's run_once rebuilt a fresh EMPTY FleetSync on every connection cycle. Because the proposed fleet://events?since=<cursor> contract is a post-cursor DELTA (returns only events after the durable cursor), a reconnect would apply only the newly-arrived events onto an empty snapshot — DROPPING every seat introduced before the cursor. The reducer's max-join makes re-delivery idempotent (kills dupes) but cannot recover a seat whose event is never re-sent, so on every transient forge bounce the fleet rail would collapse to just-the-delta — the exact opposite of the ticket's "Marley down != fleet down / no gap" promise. Masked in tests only because the fixture served cumulative pages. Fix: carry ONE FleetSync across the thread's whole life (run_subscription seeds it from the persisted cursor once and passes &mut sync into every run_once) so a reconnect resumes from the held cursor AND retains the accumulated snapshot; losslessness then holds for both a cumulative-since and a delta-since forge. Guarded by a reworked REQ-005 that serves a strict delta (PAGE_C) on reconnect and asserts the snapshot stays {a,b,c}; empirically confirmed the test fails (delivers {c}) without the fix. Found by the security/state-integrity critic.

## DL-cluster-dd674fc43a64
*topic: #361's `is_monospace_advance` guard `if wide_w <= 0.0 { return true }` yields an EQUIVALENT (surviving) cargo-mutants mutant `<= → ==`: it only diverges from the original at `wide_w < 0`, and there... · confidence: 0.500*

#361's `is_monospace_advance` guard `if wide_w <= 0.0 { return true }` yields an EQUIVALENT (surviving) cargo-mutants mutant `<= → ==`: it only diverges from the original at `wide_w < 0`, and there the fall-through computes `(≥0).abs() / (<0) ≤ 0 ≤ ε` which ALSO returns true — so no input distinguishes them → the mutant is MISSED → MSI < 100 (a hard-gate blocker). Caught by inspect Critic 1 (MEDIUM). Fix: restructured to a POSITIVE gate `if wide_w > 0.0 { ratio } else { true }` — the boundary now lands at `wide_w == 0` (where the else returns true but the ratio branch would divide-by-zero), so the `>`-vs-`>=` mutant is killed by a `(narrow, 0.0)` test and `>`-vs-`==`/`<` by a normal `(3.0, 8.0)` test; all killed by the already-planned Phase-4 units. Note `!(wide_w > 0.0)` also kills it but trips clippy's `neg_cmp_op_on_partial_ord` — the if/else positive form is both mutant-complete AND clippy-clean, and preserves the conservative bad-measurement→monospace default.

## DL-cluster-de9c7a1c523e
*topic: enforce-warp-reference.sh stripped HTML-comment guidance with `sed '/<!--/,/-->/d'` to test if a spec's `## Reference (§20)` section is empty. In a POSIX sed RANGE, the closing `/-->/` address is o... · confidence: 0.500*

enforce-warp-reference.sh stripped HTML-comment guidance with `sed '/<!--/,/-->/d'` to test if a spec's `## Reference (§20)` section is empty. In a POSIX sed RANGE, the closing `/-->/` address is only tested on lines AFTER the one that matched the opener — so a comment opened AND closed on the SAME line (`<!-- x -->`) never closes on that line, and the range deletes that line plus everything down to the next `-->` (or EOF), swallowing legit prose. Result: a spec whose Reference section is a one-line comment ABOVE real prose (or prose with an inline comment) was FALSELY BLOCKED from committing. Bites when an editor/prettier collapses the multi-line guidance comment to one line, or an author inlines a comment. Inherent sed semantics — reproduces identically on BSD + GNU sed. Caught by the inspect critic running the hook (my self-review + smoke used only multi-line comments, so missed it). Fix: strip same-line comments FIRST (`sed 's/<!--.*-->//g'`) then the multi-line block. Class: a sed `/open/,/close/d` range cannot delete a same-line open+close — pre-strip same-line matches before range-deleting multi-line blocks.

## DL-cluster-decf48e661c9
*topic: #325 workspace/symbol keyed its stale guard on the query STRING only. Two concurrent in-flight fan-outs of the SAME string (type `par` → backspace → `par` while the first is still indexing) both sa... · confidence: 0.500*

#325 workspace/symbol keyed its stale guard on the query STRING only. Two concurrent in-flight fan-outs of the SAME string (type `par` → backspace → `par` while the first is still indexing) both satisfied key.query == live and both merge-appended → every row duplicated. The completion sibling avoids this because CompletionKey carries the monotonic buffer version. Fix: SymbolQuery gains a monotonic gen bumped per fan-out; an older answer's gen mismatches → dropped; symbol_request also cleared on close. The class: a per-keystroke re-query keyed only on the query can't distinguish concurrent same-string fetches — carry a generation.

## DL-cluster-e33edf1ec687
*topic: #276 inspect F2: line_span included a row the selection touches only at column 0 (shift+Down / full-line sweep — the most common block gesture), so Tab indented a visually-unselected line, and inde... · confidence: 0.500*

#276 inspect F2: line_span included a row the selection touches only at column 0 (shift+Down / full-line sweep — the most common block gesture), so Tab indented a visually-unselected line, and indent_edits padded EMPTY lines into whitespace-only lines. The reference convention (VS Code/Zed observed) excludes both. Fix: col-0 carve in line_span (gated last > first so bare-caret ⇧Tab/D4 is untouched) + an empty-line filter in indent_edits. Trap within the fix: an `a_row <= c_row` orientation branch for picking the end-column is an EQUIVALENT MUTANT at row-equality (the col is dead when last==first) — pick endpoints via .min()/.max() METHOD calls instead, which cargo-mutants never mutates.

## DL-cluster-e38eb383b227
*topic: TICKET-023: marley_terminal::pump's WouldBlock retry budget (8 × ~1ms sleeps, added by the M0 BF-nonblocking-pty-busyloop fix) ran UNCONDITIONALLY — including on completely idle PTYs — costing ~12m... · confidence: 0.500*

TICKET-023: marley_terminal::pump's WouldBlock retry budget (8 × ~1ms sleeps, added by the M0 BF-nonblocking-pty-busyloop fix) ran UNCONDITIONALLY — including on completely idle PTYs — costing ~12ms per pane per call. The new multi-pane pump-all timer (16ms tick) multiplied that fixed floor by pane count: MEASURED 98ms/tick at 8 idle panes (frame budget blown at 2 panes, ~10fps at 8). Caught by an inspect critic that MEASURED the loop with real sessions instead of reading it. Fix: idle fast-path — a LEADING WouldBlock (nothing read this call) returns immediately; the ~1ms retry windows apply only MID-BURST (after bytes were read), preserving the original busyloop-fix's purpose. Re-measured: 19.7µs/tick at 8 panes (~5000×).

## DL-cluster-e3da46baa227
*topic: M22 #338. `edit_ranges_restoring` hardcoded `begin_group(restore, /*cursor_anchored*/ true)`. That flag's documented meaning is "every cursor ends exactly at the end of its own insert" — the precon... · confidence: 0.500*

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

## DL-cluster-e494a1798e0c
*topic: TICKET-397 inspect HIGH: split_file_pane seeded the newly-instance-owned #275 disk snapshot with None "as the pre-#397 split pane was". Valid when the field was per-VIEW; poison once it became the ... · confidence: 0.500*

TICKET-397 inspect HIGH: split_file_pane seeded the newly-instance-owned #275 disk snapshot with None "as the pre-#397 split pane was". Valid when the field was per-VIEW; poison once it became the ONE instance's: a split-FIRST open left disk=None, and the later tab open's own stat was discarded on the resolve HIT (make never runs) — every #275 choke mapped None to Noop, so an agent's external write would be silently clobbered by ⌘S with no Changed banner. Fixed: stat-before-read at the split birth too, so the shared instance is born tracked regardless of which view opens first.

## DL-cluster-e57ad262a972
*topic: #323 LSP code actions were DEAD against a real server and every unit/headless test passed. Marley's `initialize` advertised NO `textDocument` client capability, so per LSP 3.17 rust-analyzer withho... · confidence: 0.500*

#323 LSP code actions were DEAD against a real server and every unit/headless test passed. Marley's `initialize` advertised NO `textDocument` client capability, so per LSP 3.17 rust-analyzer withholds `CodeAction` literals (no `codeActionLiteralSupport`) — ⌘. returned "No actions here" despite a live unresolved-name diagnostic (⌘K hover on the same token proved the LSP round trip worked). Only the live drive against real rust-analyzer surfaced it; a mocked/`fake_ls` wire returns whatever the test author scripts, so it can never catch an unadvertised-capability omission. Fix: advertise `textDocument.codeAction` = `codeActionLiteralSupport`(kinds) + `isPreferredSupport` + `dataSupport` + `resolveSupport.properties=["edit"]` — exactly what #323 implements. Sibling of the #322 F-CORR-1 `documentChanges` omission (which made the version-conflict guard inert the same way). Both are honest-capabilities omissions where the SERVER gates its reply on a CLIENT capability.

## DL-cluster-e93b00476e27
*topic: #375 inspect F2: the per-session Mcp-Session-Id registry shipped with a reject-new cap (8) but NO reclamation path except an explicit client DELETE — no TTL, no disconnect-reap. Because the MCP spe... · confidence: 0.500*

#375 inspect F2: the per-session Mcp-Session-Id registry shipped with a reject-new cap (8) but NO reclamation path except an explicit client DELETE — no TTL, no disconnect-reap. Because the MCP spec permits (and Marley's own #373 self-healing pump does) re-initialize-on-reconnect WITHOUT sending DELETE, 8 cumulative reconnects would orphan 8 sessions and permanently wedge the global cap → every future initialize returns 503 until the app restarts, defeating the exact self-healing the sibling tickets built. A bounded pool with a reject-at-cap policy but no reclamation is a self-DoS footgun. Fixed by reaping a session when its standing GET SSE stream drops (thread the validated session id into serve_sse_stream, terminate on the write-failure/hang-up path) — tying session lifetime to the client's presence (the natural MCP model), so a reconnect frees its old slot and the cap self-heals. Found by the protocol-lifecycle critic via a cross-ticket cross-check against the #373 pump's reconnect behavior.

## DL-cluster-ea30834fde66
*topic: A platform-conditional value written as two `#[cfg(...)]`-gated FUNCTIONS — `#[cfg(not(windows))] fn host_flavor()->Posix` / `#[cfg(windows)] fn host_flavor()->Windows` — produces an UNKILLABLE mut... · confidence: 0.500*

A platform-conditional value written as two `#[cfg(...)]`-gated FUNCTIONS — `#[cfg(not(windows))] fn host_flavor()->Posix` / `#[cfg(windows)] fn host_flavor()->Windows` — produces an UNKILLABLE mutant on a single-platform (macOS) CI runner. cargo-mutants parses the source syntactically (cfg-agnostic) and generates `replace host_flavor -> PathFlavor with Default::default()` for BOTH fns. The not(windows) one fails to build (no Default) → unviable (safe). The cfg(windows) one is cfg'd-out on macOS, so the mutated line never compiles → the build is identical to baseline → tests pass → the mutant is MISSED/survives. No macOS test can execute cfg(windows) code, so it is unkillable → MSI capped at ~95% → gate:5 RED, unfixable by any test. Caught by the inspect mutation critic (verified via `cargo mutants --list` + a full run). Fix: replace the two cfg-gated fns with two `#[cfg]`-gated `const HOST_FLAVOR: PathFlavor`. cargo-mutants does not mutate const items and a const is not an executable coverage line. Verified: `cargo mutants --list` now shows 0 host_flavor mutants.

## DL-cluster-ea3ea4bbb5bb
*topic: #214 Phase-2 regression test plan (REQ-005 L5) specified a single "touching (keep) vs overlapping (drop)" boundary test for the new `ranges_overlap(a,b) = a.start < b.end && b.start < a.end` helper... · confidence: 0.500*

#214 Phase-2 regression test plan (REQ-005 L5) specified a single "touching (keep) vs overlapping (drop)" boundary test for the new `ranges_overlap(a,b) = a.start < b.end && b.start < a.end` helper. But `cargo mutants --list` on links.rs shows the predicate yields TWO distinct viable `<`→`<=` mutants (one per comparison), each killable only at a DIFFERENT touching orientation. The single planned L5 would kill only one → the other survives → MSI < 100 at Phase-4 validate (gate RED). Caught by the inspect critic (Critic 2) before validate — not shipped. Fixed by amending the plan to L5a (scanned STARTS at explicit END, kills a.start<b.end→<=) + L5b (scanned ENDS at explicit START, kills b.start<a.end→<=) with concrete run inputs. The code was correct; only the test plan was under-specified. Rule: PR-claude-two-sided-interval-predicate-needs-boundary-fixture-per-comparison-001.

## DL-cluster-eb82259cf8a8
*topic: marley_spike inspect found 2 CRITICAL latent bugs (both empirically proven by a critic; check+clippy were green and hid them): (1) PTY-DROP DISCARDS OUTPUT — term_io::spawn_pty built an alacritty_t... · confidence: 0.500*

marley_spike inspect found 2 CRITICAL latent bugs (both empirically proven by a critic; check+clippy were green and hid them): (1) PTY-DROP DISCARDS OUTPUT — term_io::spawn_pty built an alacritty_terminal::tty::Pty, cloned its master fd, and RETURNED only the clone while the Pty was dropped at end of fn. alacritty's `Pty::drop` does kill(SIGHUP)+child.wait() then closes the master, which DISCARDS the unread kernel PTY output queue; the surviving try_clone'd fd then reads immediate Ok(0) → empty grid. Proven: a drop-before-read probe failed 50/50 (always immediate Ok(0)); a hold-the-Pty-across-read probe passed 50/50. The planned R3/R4/R5 integration test would have failed + the headed window would render a BLANK body. Fix: spawn_pty returns the live Pty in the tuple; app.rs + the integration test bind it across the drive_reader read loop and drop it AFTER. (2) OPAQUE ExitCode UNKILLABLE MUTANT — exit_code_for(ShutdownOutcome)->std::process::ExitCode: ExitCode impls Default (=SUCCESS) so cargo-mutants' `-> Default::default()` is VIABLE, but ExitCode has no PartialEq/accessor so no unit test can observe it → the mutant survives → MSI<100, unfixable by a test. Fix: split a pure `exit_status_code(o)->u8` (testable: ==0 / !=0) + a 1-line `exit_code_for = ExitCode::from(exit_status_code(o))` marked mutants::skip. ALSO: the implement step ran clippy but not `cargo fmt --check`, so a non-canonical gpui closure failed gate:1 fmt (caught at inspect).

## DL-cluster-eedbc7f6e053
*topic: Marley #296. Buffer::edit_at_selections WRITES self.selection, but undo()/redo() restore only the TEXT — they do not maintain the selection set. So after a multi-caret edit followed by ⌘Z, buffer.s... · confidence: 0.500*

Marley #296. Buffer::edit_at_selections WRITES self.selection, but undo()/redo() restore only the TEXT — they do not maintain the selection set. So after a multi-caret edit followed by ⌘Z, buffer.selection() holds offsets PAST EOF ([3,7] in a 3-char rope). Feeding those straight back into edit_at_selections handed ropey an out-of-bounds range → PANIC at rope.rs:952 "end is out of bounds". edit_at_selections is the ONLY pub fn that accepts offsets it did not itself produce and forwards them to the rope, while its sibling set_selection already clamped defensively — the inconsistency was the tell. FIX: edit_at_selections CLAMPS + canonicalizes its INCOMING set before use; regression test drives multi-edit → undo → edit and asserts no panic + in-bounds result.

## DL-cluster-ef5de8f91459
*topic: #234 relaxed the M10 never-empties invariant (the last workspace can close → 0 projects) and guarded the main render with a launcher branch — but the design (D1) analyzed accessor-reachability ONLY... · confidence: 0.500*

#234 relaxed the M10 never-empties invariant (the last workspace can close → 0 projects) and guarded the main render with a launcher branch — but the design (D1) analyzed accessor-reachability ONLY inside render. THREE accessors of the active project reached it OFF the guarded render path and panicked at 0 workspaces: (1) [CRITICAL] the 16ms PTY pump (a cx.spawn background loop, app.rs:552→568) derefs active_project() every tick, independent of render; (2) [CRITICAL] close_project_at → persist_grid (app.rs:2061) calls workspace_mut() unconditionally after the last close — fires SYNCHRONOUSLY in the click handler, crashing before the launcher even shows; (3) [HIGH] find_match_rows (app.rs:1154) runs in the render PROLOGUE (before the launcher branch, gated only on find_open). Two adversarial critics (a dedicated panic-safety audit + a correctness critic) found all three; unit tests could not (the pump/persist/render are mutants::skip cov-excluded shims — only a driven run or the audit catches it). Fixed: hoisted the launcher branch to the very top of render, early-returned the pump closure at 0, guarded persist_grid at 0.

## DL-cluster-f197c8c2e216
*topic: #317 find-references: the "Finding references…" searching card was gated on the app-side `references_request.is_some()` latch alone. A timed-out (10s, routine while rust-analyzer indexes) or discon... · confidence: 0.500*

#317 find-references: the "Finding references…" searching card was gated on the app-side `references_request.is_some()` latch alone. A timed-out (10s, routine while rust-analyzer indexes) or disconnected references request has its purpose DROPPED by the pump with NO response pushed (lsp_host.rs:322/345), so apply_references_response never runs and the latch is never cleared → the OCCLUDING card is stranded permanently (Esc can't reach it; a fresh ⇧F12 on a dead server fails at send without touching the latch). This is the EXACT #331 has_pending_inlay lesson reintroduced — a latch made VISIBLE (a card) turns the shared naive-latch bug into a real user-facing defect. FIX: added LspHost::has_pending_references() (mirrors has_pending_inlay — scan the pending purposes table, the one source of in-flight truth); the card renders only while a References purpose is genuinely in flight; also cleared references_request in the ⌘⇧A/launcher overlay-clear as a full-reset escape hatch. Caught by inspect critic 2.

## DL-cluster-f1d2c170e779
*topic: #326 mirrored #325's symbol_request stale-key field (search_request: Option<SearchQuery>) but the worker path gates staleness on the scalar search_gen via accept_gen(msg_gen, self.search_gen) — so ... · confidence: 0.500*

#326 mirrored #325's symbol_request stale-key field (search_request: Option<SearchQuery>) but the worker path gates staleness on the scalar search_gen via accept_gen(msg_gen, self.search_gen) — so search_request was NEVER read: write-only dead state (a needless query.clone(), two `= None` clears whose comment claimed a reopen-protection that the search_gen += 1 bump actually provides, and a dead SearchQuery struct + its stale-key doc describing a mechanism search_gen implements). #325's analog was genuinely read (apply_workspace_symbol_response compares the whole (query,gen) key); #326 switched to a gen-scalar accept, making the copied field redundant, but left it behind. Two critics (concurrency + simplification) independently flagged it. Fixed: deleted the field + its 5 sites + the struct; search_gen is the single source of truth.

## DL-cluster-f1fbcf7d3298
*topic: #376 inspect F1 (HIGH, found by 2 critics): pump_fleet_live was inserted between pump_mcp_host's doc comment + #[cfg_attr(test, mutants::skip)] and its fn — the 5th strike of the recorded mutants-s... · confidence: 0.500*

#376 inspect F1 (HIGH, found by 2 critics): pump_fleet_live was inserted between pump_mcp_host's doc comment + #[cfg_attr(test, mutants::skip)] and its fn — the 5th strike of the recorded mutants-skip detach trap. Rust bound BOTH doc blocks + BOTH skip attrs to the new fn, leaving pump_mcp_host unskipped; cargo mutants --list proved an unkillable `replace pump_mcp_host with ()` (MSI RED). Sharper edge this time: gates.sh --diff uses --in-diff, and the detached mutant's span sat OUTSIDE the changed lines — it would have slipped this commit and detonated on a later unrelated run, misattributed. Fix: relocated the new fn as a self-contained doc+skip+fn block ABOVE the neighbor, restoring adjacency; --list re-run showed 0 mutants for both pumps. The trap fires on INSERT-BETWEEN-ATTR-AND-FN, not just insert-above-fn.

## DL-cluster-f3ada13b657d
*topic: TICKET-020: the design notes' nested-2x2 PaneGroup test fixture Split(H)[Split(V)[A,B],Split(V)[C,D]] was UN-BUILDABLE via the public API. PaneGroup::split only ever replaces a LEAF with a 2-child ... · confidence: 0.500*

TICKET-020: the design notes' nested-2x2 PaneGroup test fixture Split(H)[Split(V)[A,B],Split(V)[C,D]] was UN-BUILDABLE via the public API. PaneGroup::split only ever replaces a LEAF with a 2-child Split (SPEC-app-shell R8) — it never wraps an existing Split node — so you cannot construct that particular nesting by hand-assembling the intended shape; the real build single(A)→split(A,B,H)→split(A,C,V)→split(B,D,V) yields Split(H)[Split(V)[A,C],Split(V)[B,D]] (a DIFFERENT leaf arrangement: panes [A,C,B,D], grid A|B / C|D). Caught at inspect by an empirical probe (built the tree via the API + printed it) before writing the validate test — had the test hand-asserted on the un-buildable tree it would have used wrong neighbor expectations. NOT a code bug (the algebra + the R12 neighbor rule are correct); a latent test-fixture error. Fix: validate builds the fixture by CALLING split() + asserts the R12-correct neighbor results for the ACTUAL tree (neighbor(A,Right)=B, A-Down=C, B-Left=C, D-Left=C via the last-child-for-Left rule, A-Up=None).

## DL-cluster-f44d37076898
*topic: The commit-gate receipt (gate_state_hash) fingerprinted only crates/**/*.rs, while enforce-commit-gate.sh triggers only when .rs is in the changeset. So the files that DEFINE the bar — scripts/gate... · confidence: 0.500*

The commit-gate receipt (gate_state_hash) fingerprinted only crates/**/*.rs, while enforce-commit-gate.sh triggers only when .rs is in the changeset. So the files that DEFINE the bar — scripts/gates.sh, .claude/hooks/**, deny.toml, .gitleaks.toml — were committable with NO gate run and outside the fingerprint. An agent could weaken the gate (lower a floor, add an exclusion) and commit it ungated, pre-staging a future false green; TICKET-000 itself rode this bypass. Partial fix: gate_state_hash now folds the gate-defining files (scripts/*.sh, .claude/hooks/**, deny.toml, .gitleaks.toml, Cargo manifests + lockfile) into the fingerprint, so a post-green weakening invalidates a later .rs commit's receipt. Full closure (commit hook also demanding a receipt for no-.rs edits to gate-defining paths) is deferred — an all-stub workspace has no FULL-greenable gate to produce one; it lands once a crate gives real testable code. Disclosed in CONSTITUTION §15.

## DL-cluster-f4542e8286f5
*topic: Extracting a guard ladder out of the masked shim `open_file_in_viewer` into a new `load_code_view_state` MOVED the `#[cfg_attr(test, mutants::skip)]` onto the extracted helper, leaving the refactor... · confidence: 0.500*

Extracting a guard ladder out of the masked shim `open_file_in_viewer` into a new `load_code_view_state` MOVED the `#[cfg_attr(test, mutants::skip)]` onto the extracted helper, leaving the refactored `open_file_in_viewer` (a &mut self fs-IO method with no unit test) UNMASKED. app.rs has NO global mutation exclude (its gate exclude is COVERAGE-only), so mutation relies on per-fn skips → cargo-mutants' 'replace body with ()' mutant on the now-unmasked fn would survive → gate:5 (MSI 100) RED on the diff. Caught by the refactor-safety critic (my self-review missed it). Class = the known mutants::skip detach trap, this time via EXTRACT-and-relocate (not the insert-above variant): a structural edit around a masked shim silently detaches its skip from the fn that still needs it. Fixed by restoring the skip on open_file_in_viewer; verified via `cargo mutants --list -f app.rs | grep` (all 3 shims absent = masked).

## DL-cluster-f53fd83b9b6c
*topic: #298: `add_next_occurrence` chose its first-press branch with `primary().is_caret()`, using it as a stand-in for "is this the first press". It is NOT the same question. `primary()` is member 0 — th... · confidence: 0.500*

#298: `add_next_occurrence` chose its first-press branch with `primary().is_caret()`, using it as a stand-in for "is this the first press". It is NOT the same question. `primary()` is member 0 — the TOPMOST — and #297's ⌘-click can put a bare caret ABOVE an existing range. Two HIGH bugs fell out of that one substitution: (1) ⌘-click in leading indentation → that wordless caret is member 0 → the set (which already HAS a needle) goes back down the word-select branch → `word_range_at` returns None → `unwrap_or(sel)` leaves it a caret → the set comes back BYTE-IDENTICAL. Every press. ⌘D is dead for the session with no feedback and no way out but Esc. (2) If that ⌘-clicked caret DOES have a word under it, expanding it MERGES into the range beside it — 2 cursors become 1, a cursor the user explicitly placed is destroyed, and the needle silently changes from "bar" to "foobar" so every subsequent ⌘D searches for different text. Both are motions, so ⌘Z cannot undo them. The doc comment claimed the mixed shape was handled ("a caret with no word under it stays a caret, so the fn is TOTAL") — it was total, but not PROGRESSING; totality was the wrong property to check. FIX: discriminate on the WHOLE SET (all members are carets ⇒ first press), and take the needle from the topmost RANGE, not from member 0.

## DL-cluster-f5ff9c3814a0
*topic: standardize_path's helper read the cwd via `current_dir().map(...).unwrap_or_default()`, returning "" when cwd is unavailable (reachable: a terminal/editor whose working directory was deleted or ha... · confidence: 0.500*

standardize_path's helper read the cwd via `current_dir().map(...).unwrap_or_default()`, returning "" when cwd is unavailable (reachable: a terminal/editor whose working directory was deleted or had search permission revoked). With cwd="", a RELATIVE input stayed relative after normalize — yet StandardizedPath::is_absolute() hardcodes `true`, so the sole constructor of the absolute invariant emitted a relative/empty value while claiming absolute, violating R9 and making is_absolute() lie. Caught by the inspect correctness critic (probed in the typed-path scratch crate). Fix: thread cwd as a parameter to standardize_under<E>(input, cwd) (a §14 testable-IO seam) with an empty-cwd→flavor-root("/") fallback so the result is ALWAYS absolute; the relative-input and empty-cwd branches are now deterministically testable by injecting cwd.

## DL-cluster-fae9a724fb96
*topic: #261: the mutants::skip DETACH TRAP fired a THIRD time (after #183/#184) — inserting icon_label between caption_header's doc/attr stack and its fn rebound the doc + #[cfg_attr(test, mutants::skip)]... · confidence: 0.500*

#261: the mutants::skip DETACH TRAP fired a THIRD time (after #183/#184) — inserting icon_label between caption_header's doc/attr stack and its fn rebound the doc + #[cfg_attr(test, mutants::skip)] to the NEW fn, leaving caption_header unmasked (a caption_header body mutant appeared in cargo mutants --list; survived only because gpui::Div lacks Default → unviable). Fixed by re-seating the whole icon_label block ABOVE caption_header's doc; --list re-run shows 0 mutants on both helpers. The existing rule (verify with cargo mutants --list after inserting a fn near a skip) held — inspect caught it because the critic ran the list.

## DL-cluster-fb062c286d2b
*topic: collapsed_indices (root→index) aliases when two projects share the SAME root string (an 'open same folder twice' state) — both restore collapsed even if one was expanded, since root-keying can't di... · confidence: 0.500*

collapsed_indices (root→index) aliases when two projects share the SAME root string (an 'open same folder twice' state) — both restore collapsed even if one was expanded, since root-keying can't distinguish them. Cosmetic (a project renders collapsed you left expanded), no panic/data-loss, needs the unusual dup-root state. Documented as an inherent tradeoff of root-keying (the whole #245 premise — a stable key that survives reorder); found by the inspect critic. Class: keying persisted per-item state by a non-unique attribute aliases duplicates on restore — dedup the key or add an ordinal if uniqueness isn't enforced.

## DL-cluster-feb7262f4123
*topic: #314 formatting: the caret was captured as a Bias::Left Anchor before applying the format edits, to carry it through the reformat (REQ-003). But rust-analyzer returns ONE whole-file TextEdit (edit(... · confidence: 0.500*

#314 formatting: the caret was captured as a Bias::Left Anchor before applying the format edits, to carry it through the reformat (REQ-003). But rust-analyzer returns ONE whole-file TextEdit (edit(0..len, new)), which COVERS the caret's anchor, and rebase_offset collapses a COVERED Left-bias anchor to the span start (0). So for the actual (whole-file) reformat shape, the anchor resolves to offset 0 → the caret+viewport teleport to line 1 every format — the exact bug the anchor was added to prevent. The anchor only rebases correctly for GRANULAR edits (which don't cover the caret), but the spec itself said granular edits carry the caret for free — so the anchor was redundant where it worked and ineffective where it was needed. Proven by anchor.rs::covered_collapses_by_bias (covered+Left → span start) + a real-Buffer throwaway. Found by an adversarial critic; my self-review + the design + the code comment all wrongly assumed the anchor carried the whole-file case. Fix: a (line, col) re-seat — capture line_col(caret) before apply, re-seat via caret_for_line_col(row+1, col+1) after (clamps into the reformatted line), keeping the caret on its logical line best-effort.
## L-claude-export-before-off-switch-full-diff-verification-001
*topic: data migration off a live service · confidence: high*

Sequencing that worked for the forge scrap (TICKET-409): (1) inventory with exact
counts BEFORE writing anything (the counts become the acceptance baseline), (2)
export deterministically (stable ORDER BY, in-script count asserts that exit
nonzero), (3) verify by FULL-CORPUS BYTE-DIFF against the source at inspect —
counts catch loss, only diffs catch mangling (the double-encoded jsonb Tags bug
survived count checks and died under diff), (4) only then turn the service off,
keeping its datastore intact so the export remains re-checkable. The service
stayed READ-available throughout the pipeline; the off-switch was the last act.

## L-claude-rip-keeps-pure-seams-called-constant-input-001
*category: design · topic: feature rips vs dead_code (410 browser forge-identity rip) · confidence: 0.800*

When ripping a feature whose config/derivation feeds pure decision seams (planner fns
like `mount_plan`/`retry_plan`), wire the surviving call sites with a CONSTANT input
(`mount_plan(None)`) instead of deleting the calls: a `pub(crate)` pure fn referenced
only by tests trips dead_code at `-D warnings`, forcing either the fn's deletion
(losing the machine the ticket says to keep) or an `#[allow]` (gate:12 bait). A
constant literal at an excluded/`mutants::skip` call site is not a mutation target,
keeps the machine compiled + total + unit-tested, and the comment on the literal
documents the seam a future feature fills. Split the seams by SEMANTICS: delete the
ones whose meaning dies with the ripped input (`effective_retry_base`'s
transient-vs-absent `.mcp.json` disambiguation), keep the generic ones
(`retry_plan`'s desired-vs-mounted planning). Sibling of [[PR-claude-callback-stored-inside-owned-resource-captures-weak-001]] in the 410 design notes.

## L-claude-drive-gpui-menus-by-keyboard-not-coordinates-001
*category: validation harness · topic: self-test drive of rail/menu affordances (410 live parity run) · confidence: 0.750*

Driving a gpui context/＋ menu with `clickat:` coordinates is fragile twice over: rail rows
shift between captures (each click can change the active tab and re-layout, so a fraction
read from the LAST capture hits a different row NOW), and the menu itself anchors to the
pointer so row math compounds the error — two 410 misclicks opened wrong cockpit tabs and
one ⌘W landed on a terminal. The robust pattern: click ONLY to open the menu, then drive
the menu by KEYBOARD — Marley's one menu machinery opens with row 0 selected and
arrow-wraps (`context_menu.rs`), so `down`×(index) + `enter` selects any row
deterministically regardless of where the menu popped. Read the row index from the menu's
ITEMS constant in source (e.g. `SECTION_BROWSER_ITEMS`: Forge 0 / Agents 1 / Details 2 /
Browser 3), not from pixels. Also: chain activation + every drive verb in ONE shell
command (the README rule), and re-capture before any coordinate click you must still make.

## L-claude-identity-unification-sweeps-prefix-and-strip-consumers-001
*category: design · topic: path-identity / canonicalization · status: active*

When a design unifies an identity to ONE canonical spelling at the storage seam, equality compares are only HALF the consumer surface. Sweep, in the same pass: (1) `starts_with` containment/ownership lookups — #319 design found the app.rs host-spawn gate (`resolve_under_root(...).starts_with(verbatim_root)`) would go ALWAYS-FALSE once open docs stored canonical spellings, silently killing ALL LSP for symlinked-root projects — a regression the equality-focused ticket scope never mentioned; the encoding-by-root filters (jump_to_symbol/jump_to_problem) had the same class of miss, degrading to a wrong default encoding. (2) `strip_prefix` rel-derivations — git pathspecs, tree reveal, display labels — which silently fall back to absolute paths when the stored spelling stops matching the verbatim root. (3) `HashMap` keys on the identity type (consistent only if every writer uses the one helper). Rule: before locking an identity-normalization design, grep `==`, `starts_with(`, `strip_prefix(`, and map keying over the identity type; each hit is either adopted, proven-consistent, or documented-accepted. Equality is the visible bug; prefix/derivation consumers are where the regression hides.

## L-claude-fix-must-not-regress-what-the-bug-got-right-001
*category: design · topic: lifecycle-scrubs / bug-fixing · source: #353 design+inspect*

When fixing a MISSING-cleanup bug (a map never cleared, a listener never detached),
enumerate the cases where the ABSENCE of cleanup was accidentally correct before
writing the fix — the naive fix regresses exactly those. #353: `editor_folds` was
never cleared on close; two alias-root workspaces of one dir share a fold entry
across two instances, and the LEAK was what kept the survivor's folds alive when
one closed. A plain clear-on-close would have broken that; the census guard (one
`reg.iter()` scan: clear only when NO same-path instance survives) made the fix
exact for three extra lines. Second half of the lesson: the design cost was near
zero because the substrate already owned both halves — `release_view -> Option<C>`
reports the drop, and `release_grid_terminals` (#398) is the in-repo scrub
pattern to mirror — the prior-art sweep's in-repo leg turned five design
decisions into one.

## L-claude-fallback-arms-share-the-primary-arms-shape-structurally-001
*topic: #333 — the hand-lexer fallback bled phantom lexer state; the fix was already prescribed, the win was making the shape SHARED · confidence: 0.9*

When a render/compute path has a primary arm and a fallback arm over the same
consumer contract, the fallback must share the primary arm's DOMAIN SHAPE
structurally — the same helper fn — not by parallel text that happens to look
alike. #333's bug existed because the fallback lexed the display string while
the primary lexed raw and mapped; the fix (raw-lex + map through the SAME
remap) was prescribed two tickets earlier by the #331 AD, and the inspect
critic's rule-of-three finding turned "identical shape by convention" into
`spans_to_display_bytes` — one covered pure fn all three former copies ride.
Two force multipliers, both reusable: (1) in a codebase with coverage-excluded
shims, hoisting duplicated logic OUT of the shim is not just cleanliness — it
moves lines from the untested-by-policy region into the 100% denominator;
(2) the phase discipline paid concretely — the fmt Stop-hook caught an
unformatted test block BEFORE the gate burned a run on it, and the gate's
clippy red (type_complexity on a test annotation) was fixed by DELETING the
annotation and letting assert_eq back-propagate the type, not by an allow.

## L-claude-plan-phase-producer-enumeration-sharpens-requirements-001
*category: process · topic: planning*

Enumerating the ACTUAL producers of a load-bearing event at PLAN (one grep:
`git_marks_gen =` → save/reload/commit only — no activation bump) converted a
ticket clause that read as hygiene ("git_marks_key never dangles") into a
required-for-correctness requirement with its own REQ and drives, BEFORE design
locked anything. The ticket's own summary carried the folklore version ("the
activation choke bumps the gen"); the grep, not the recall, caught it. Cheap
discipline: for every "X will heal it later" assumption in a ticket, list X's
call sites at plan — if the list is empty for the scenario at hand, the healing
is imaginary and the requirement is real.

## L-claude-same-object-atomicity-beats-same-call-001
*category: design · topic: state-coupled reads · from: pipeline 320 (M20 LSP doc-sync)*

When collapsing two same-state-gated reads into one call (the PR-claude-two-gated-calls-must-read-the-phase-once-001 fix), prefer passing the LIVE OBJECT over a pre-rendered projection of it: TICKET-320's reconcile takes `&dyn DocText { version(), text() }` (implemented by the editor Buffer) instead of a `(version, text)` pair or a `text_of` closure. Same-CALL atomicity fixes gate-vs-materialize divergence; same-OBJECT atomicity also fixes projection-vs-projection divergence (a version read on one tick paired with text read on another can lie the same way the phase did). A `&dyn` trait keeps the shim non-generic AND makes the hot path provable — a counting test double is impossible with a bare `&Buffer` parameter. General rule: if a consumer's decision depends on N facts about one entity, hand it the entity, not N facts.

## L-claude-new-terminal-state-new-enum-not-err-variant-001
*category: design · topic: shared-channel type evolution · from: pipeline 332 (LSP abandoned-terminate)*

When a shared delivery channel gains a NEW terminal state (e.g. "abandoned"
joining "answered ok / answered err"), do not encode it as a new variant or
code inside the EXISTING error type — every consumer's `Err(_)` catch-all keeps
compiling and silently absorbs the new state into its semantic-error path
(exactly the F6b phantom-toast bug the #331 critics proposed, generalized).
Instead REPLACE the channel's element type with a new enum that wraps the old
payload (`RequestOutcome::Answered(Result<V, E>) | Abandoned(reason)`): every
consumer breaks at compile time and is forced to make its per-arm decision
explicitly; the old Ok/Err logic nests unchanged inside the `Answered` arm.
Corollary for the delivering side: when a teardown path both clears the stale
queue and must deliver the new signal, the order is clear-then-extend — a
clear after the push eats the very signal the fix exists to deliver.

## L-claude-354-recheck-a-queued-tickets-blocker-against-head-001
*topic: planning · confidence: high*

TICKET-354 sat queued for ~3 weeks carrying a "moderate refactor" blocker in its own text: "save_active is active-only; a by-(pi,ti) save needs the #275/#284 machinery parameterized". By the time /work picked it, #397 (editors onto the registry) had ALREADY dissolved the blocker — `EditorInstance` carried every conflict/save method target-agnostically, `(pi,ti)` no longer even identified a file, and `locate_open_file` (the fn the ticket's plan named) was deleted. The Phase-1 Explore sweep caught this ("headline correction up front"), the spec was written to the HEAD reality, and the implementation shrank to a locator + a funnel + rewiring. The lesson: a queued ticket's text is a snapshot of the codebase AT MINT TIME — at plan, re-derive its blocker/design assumptions against HEAD before designing around them (grep the named fns first; a named-fn miss is the tell). The §20 prior-art sweep's "does the substrate already own this seam?" question applies to OUR OWN intervening work, not just deps — and a blocker dissolving is a WIN to record, not a plan defect.

## L-claude-354-empty-iterator-closures-fail-the-coverage-floor-001
*topic: test · confidence: high*

Two gate:4 (100%-line floor) reds in #354 came from test-code closures that could NEVER run: `.unwrap_or_else(|_| fallback)` on a canonicalize that cannot fail for an existing fixture, and `.any(|b| ...)` over a list the fixture makes deterministically EMPTY. llvm-cov counts a never-executed closure as an uncovered function/line even when the enclosing test passes. Write coverage-floor test code with the same discipline as production: assert the STRONGER structural fact instead of iterating what is provably empty (`assert!(v.is_empty())` beats `!v.iter().any(...)`), and use `expect` where failure is impossible for the fixture (the message documents why) instead of a dead fallback closure.

## L-claude-366-speculative-post-insert-parse-classifies-unanchored-positions-001
*category: design · topic: tree-sitter / editor typing features · from: pipeline 366 (M22 auto-close where/dyn lifetimes)*

When a typing-time feature needs to classify a caret position but the PRE-insert tree has no node to anchor on
(the token is mid-typing, so ancestry probes fall to `function_item`/`source_file` — the #362 `where T: '` wall),
do not hand-roll a backward text scan: SPLICE the token you are deciding about into a copy of the text, re-parse,
and read the node kind the parser assigns the spliced span. tree-sitter-rust 0.24.2's error recovery classified a
spliced `'a` as `lifetime` at ALL 13 bound positions (every `where` variant incl. EOF via
`function_signature_item` recovery, `Box<dyn…+ '>`, `Ref<T, '>`, parenthesized dyn, `impl Trait + '` returns) and
as `label`/ERROR at ALL 6 char/expr/const-generic-block controls — a perfect discriminator that made the ticket's
proposed const-block guard unnecessary and killed its backward-scan option (which would have false-positived at
`x + '`). Splice a RECOVERABLE token (`'a`, not the bare `'` — the bare form ERRORs at EOF). The technique is the
post-insert dual of the #362 pre-insert ancestry spike: when "what am I inside?" has no answer, ask "what WOULD
this token become?". Verify against the PINNED grammar in a scratch crate first — recovery is empirical, not
specified; the units then pin the answers.

## L-claude-413-focus-guards-partition-a-multi-root-drain-for-free-001
*category: design · topic: LSP multi-root outcome routing · from: pipeline 413 (M20 drain-all-hosts) · amended at inspect: two mechanism claims corrected*

Generalizing a single-workspace response drain to every workspace host looked like a 12-consumer behavior
audit, and the sweep (mandated by PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001)
revealed a clean partition — but the design's first mechanism story was WRONG in two load-bearing ways the
inspect critics caught, and the corrected story is the lesson. (1) The 8 focused-editor arms drop
cross-root outcomes by URI EQUALITY, not root containment: `uri_for` → `canonical_under_root` is TOTAL (an
absolute path passes through unchanged — never None), so "resolves under the owning root → None
cross-root" was false; another root's FILE fails the compare only because it is a different file. That
distinction bit immediately: under NESTED roots the same file opens under both roots as twins
(D-OPEN-DEDUPE-SCOPE) with byte-identical uris, RootView-global latches survive project switches, and twin
version counters are independent (equal counts collide — the #354 F3 class), so every uri-guarded arm
could deliver cross-INSTANCE (a background F12 jump hijacking the focused project; a completion accept
editing the wrong twin). Fix: an arm-local `outcome_is_for_focused_root` guard (dispatch root == active
project root) on the 8 focused-editor arms, dropping BEFORE latch/UI with the latch untouched (a colliding
twin re-ask may own it; send paths always overwrite latches). (2) "The active root would version-miss and
SKIP every file" was INVERTED: `version_conflict` treats a `None` synced version as NO conflict (REQ-009
fails OPEN), so the wrong root would have APPLIED stale edits with the wrong host's encoding — the owning
root is required for a STRONGER reason than the design recorded. The durable shape: uri/path equality is
FILE identity, never INSTANCE identity — a feature that can face twins must compare the instance's ROOT
(or carry a request-time ContentId, the #354 D8 pattern); and when recording a "the wrong input would have
been safe" claim, check which way the guard FAILS (open vs closed) first. The guard-free arms (rename,
codeAction/resolve, workspace/symbol, formatting) still complete in the background by design — and
rename/resolve genuinely NEED the owning root (encoding + `lsp_version_for`).

## L-claude-318-extraction-testing-posture-splits-computational-vs-render-shape-001
*category: design · topic: DRY extraction under the 100% MSI floor · from: pipeline 318 (M20 overlay-card recipe)*

BF-lsp-hover-extracted-helper-new-mutation-surface-001 says an extraction from a coverage-excluded
render closure into a named fn creates a fresh cargo-mutants target that nothing kills. #318 refined
WHEN that demands a unit test: split extractions by KIND. A COMPUTATIONAL extraction (pure values in
→ values out, e.g. `overlay_quarter_geometry`'s fractions) takes direct exact-value unit tests — they
kill the arithmetic mutants and prove the property the caller relies on (here: containment by
construction). A RENDER-SHAPE extraction (a fn returning a styled `Div`, no readback surface) follows
the shipped `icon_label` precedent instead: `#[cfg_attr(test, mutants::skip)] // render-shape only —
asserted by driven captures` — the rule's substance is "no NEW un-killed mutation surface", and a
skipped shape fn creates none while its correctness proof is the pixel-parity capture, which is
stronger than any style-field assertion would be. Corollary found the same day: a plan-time
requirement to "clamp" an overlay whose height is CONTENT-DRIVEN is unsatisfiable — there is no box
to clamp; fractional geometry (`left=w/4, w=w/2, top=h/6`) is inside the window BY CONSTRUCTION, so
the right artifact is a property test on the pure helper, not a clamp call. Check for a `max_h`
before writing "clamp" into a spec.

## L-claude-318-driving-the-live-app-needs-an-active-user-session-001
*category: process · topic: selftest harness / headless mini · from: pipeline 318 (M20 overlay-card recipe)*

The selftest drive (activate → clickat → chords → screencapture) has TWO environmental preconditions the
README only half-states, both learned the hard way in one pipeline. (1) **Keyboard focus needs a clickat
FIRST** — programmatic activation does not hand gpui key focus, so chords silently vanish; the tell is
N byte-identical capture hashes (nothing ever opened). (2) **Activation itself needs an ACTIVE user
session.** The mini is headless-by-design (BetterDisplay vscreen XOR physical monitor, CRD remoting):
with the display asleep and no user at the console, apps boot to GHOST windows (no first paint, no AX
registration, `screencapture -l` fails), `caffeinate -u` wakes the compositor but NOTHING can become
frontmost — SE set-frontmost no-ops, `open` fails, `NSRunningApplication.activate` returns false, and
synthetic titlebar clicks do not grant focus. No amount of harness cleverness substitutes for a live
session; the honest move is §7's documented skip + substitute evidence (chain-identity review, headless
draw-smokes) + a named pending item to run the capture battery when a human session exists. Corollary
for capture protocols: bank the BEFORE set the moment it is obtainable — sessions are perishable.

## L-claude-418-scripted-dom-sweep-is-the-poc-test-harness-001
*category: process · topic: react-poc verification · ticket: #418*

A React-first design ticket has no Rust tests, but "screenshot and eyeball" is not
verification for an INVARIANT (like #418's "exactly one selected row"). What worked: a
scripted browser sweep via Playwright `browser_evaluate` — click EVERY selectable row
(re-query the DOM per click; React re-renders detach stale node references), await ~60ms
per click (React state is async — a same-tick DOM read sees the OLD state), and assert the
invariant after each (`querySelectorAll('.bg-rail-active').length === 1`, plus
"no header ancestor carries the fill" per state). 12 states dark + 11 light + the
regression repros ran in seconds and caught what single-state screenshots can't (the
double-fill only appears on cross-surface SEQUENCES). Pair the sweep with computed-style
geometry reads (getBoundingClientRect / getComputedStyle) and the "measure, not eyeball"
sheet writes itself. Reuse the same sweep shape for the #419–#421 React halves and any
future rail-touching POC ticket.

## L-claude-419-port-the-selector-not-the-booleans-proved-out-001
*category: architecture · topic: react-to-rust ports · ticket: #419*

The #418 PR rule ("single-selection derives from ONE selector value, never scattered per-row
booleans") ported to Rust as an internal coordinate enum (`RailSelection { None, Tab{p,t},
Cell{p,t,k} }`) compared per row — and the port PROVED the rule's value twice: (1) the invariant
became structural (a single enum value cannot name two rows; the six old per-level `active`
computations could and did — up to six at once), so the REQ-001 "≤1 selected" sweep is almost
tautological; (2) the inspect critics' entire two-fills hunt came back clean on the first pass —
the adversarial fixtures (cross-project splits, pane-only files, 1-cell grids, collapse states)
had nowhere to break the invariant, only the FORCE-EXPAND calibration around it (the one real
find). Port mechanics that worked: hoist the coordinate computation to the top of the pure fn;
give every emission site a coordinate-equality `selected`; make dot precedence an if/else-if
chain (one dot per row by construction); keep the old signal computations (pane_mounts,
force-expand) byte-identical and re-route only their DESTINATIONS. The React side's
merge-over-defaults + clamps map to Rust as: derive from validated `.get(...)` chains and let
incoherent states degrade to fewer selections, never more.

## L-claude-420-reversing-a-projection-is-cheap-when-the-model-never-moved-001
*category: architecture · topic: derived projections · ticket: #420*

Reversing #237/#240's "N files = one rail row" took ~60 lines of model code and zero
persistence/surface changes, because the original fold was implemented as a rail PROJECTION
over an unchanged one-surface model — the surface kept per-view paths/ids/active the whole
time, so the reversal was "walk the surface in the emission loop" plus a selection-variant.
The general lesson for irreversible-looking UI decisions: keep the MODEL maximal and fold in
the projection layer; a folded projection is a cheap two-way door (the #240 spec even
pre-authorized the flip), while a folded model (dropping per-view state at ingestion) would
have made #420 a migration. Corollary proven twice in one batch (#419's selection, #420's
rows): when the projection is pure and derived-fresh, redesigns land as emission-loop edits
with mutation-strong unit coverage, and the shim arms stay mechanical.

## L-claude-421-one-menu-machinery-absorbs-new-verbs-for-free-001
*category: architecture · topic: menu machinery · ticket: #421*

Adding the rail's add-project door cost one enum arm, one const table, one items_for line, one
open-shim, and two dispatch arms — and keyboard navigation, escape/click-away dismiss, origin
clamping, one-modal discipline, and the row renderer all arrived FREE, because the #393/#398
"one menu machinery" refused parallel popovers from the start. The inspect critics' whole
keyboard/dismiss hunt came back clean without a single line of new input handling. The compound
interest of the ONE-machinery rule is real: the marginal verb is now ~30 lines and its critics
have nothing to find. Corollary recorded at #421: the machinery's kind-GENERIC key router
(esc/↑/↓/enter over any kind) is what makes this safe — a kind-matching router would have been
a silent-miss trap for every new kind.

## L-claude-422-test-code-is-coverage-code-avoid-destructuring-arms-001
*category: testing · topic: coverage discipline · ticket: #422*

gate:4 counts TEST code lines too: a `let PaneGroup::Split { .. } = &g else { unreachable!() }`
in a test contributes an uncovered line per never-taken else arm — #422's first gate run went
red on SEVEN such lines, all in the new tests themselves. The house suite already knew this
("asserting the group avoids an unreachable match arm (which would be an uncovered line)" —
a comment sitting right in the file): assert WHOLE-VALUE equality against an expected value
built from the SAME pure fns (also killing f32-drift asserts for free — the expected side
computes the identical rounding). The rewrite deleted the arms structurally, which beats
covering them. Corollary for mutation: an executed line is not an observed line — the V-arm
child-bounds math ran under every H-root fixture yet 4 mutants survived until a V-ROOT nested
fixture made the accumulation OBSERVABLE through a descendant's rect. One fixture per
recursion arm, observed through outputs, not just executed.

## L-claude-415-fixed-size-equality-asserts-need-a-growth-tripwire-001
*category: testing · topic: inventory smokes / vacuous passes · from: pipeline 415 (M20)*

Tightening a per-index assert (`assert!(flags[i])`) to a fixed-size one-hot equality
(`assert_eq!(flags, expected)` over `[bool; 5]`) silently RETIRES an accidental tripwire: the
per-index form OOB-panicked when a 6th verb was appended without extending the flags mirror;
the equality form passes VACUOUSLY (both sides all-false — the new overlay is never checked).
`std::array::from_fn(|j| j == i)` does not restore it (same all-false at i=5). Pair every
fixed-size inventory equality with an explicit growth tripwire — `assert!(expected[i], "verb
table outgrew the flags mirror")` — one line, OOB on the appended index. Same class as a
zip() over two lists silently truncating: strict inventories must FAIL when the inventory
grows past the assert's world. Found by the #415 inspect critic tracing what the retired
assert did that the new one doesn't.

## L-claude-223-count-the-final-state-not-your-guess-of-its-shape-001
*category: process · topic: scripted rewrites / vacuous verification · from: pipeline 223 (M12.2)*

A python bulk-rewrite "verified" its own success by counting occurrences of the STRING IT HAD
JUST TRIED TO REMOVE — but the string was my guess at the code's shape, and rustfmt had
already reshaped the real sites (trailing-comma/line-break differences), so the replace
missed AND the counter read zero: vacuous green twice over. The gate caught it as clippy
unused-variable reds (the hoisted bindings the dead replace was supposed to wire). Two rules:
(1) after any scripted rewrite, verify with a check derived from the REQUIREMENT, not from
the rewrite's input pattern — here `grep 'text_size(px(<digit>'` (shape-independent) was the
right counter and already existed; (2) assert the replace COUNT (`assert src.count(a) == 1`)
so a miss fails loudly instead of writing the file unchanged — the later fix did exactly this
and caught its own first-try mismatch. Same family as
F-claude-420-projection-change-turned-a-red-pin-vacuous-not-red-001: a checker keyed to a
shape you control is not a checker.

## L-claude-stale-audit-ticket-reverify-before-design-001
*(category: process/recall · topic: audit-debt tickets · status: active)*

A ticket minted from a STOPPED audit run records the world at its creation
instant and can be stale by pickup time — especially a Deliberate/idle-machine
row that waits days. #407's headline item (the `refresh_efind_matches` `<`→`<=`
missed mutant, logged at #402's stopped FULL, 2026-08-06) was killed the NEXT
DAY by #404's delivery-gate sweep — recorded only in a post-archive block at the
TAIL of #404's completed notes (`404-forge-web-base.notes.md:298`), not in any
ticket. Design caught it by grepping the completed archive for the seam name
before writing the fix plan, and REQ-002 pivoted to verify-only instead of
duplicating the test. THEREFORE at design entry, for every concrete defect a
ticket names: (1) grep the current tree for an existing test/fix carrying the
ticket's vocabulary (`grep -rn <seam> crates/`), (2) read the TAIL of the most
recent completed-pipeline notes — post-archive catches land there, (3)
`git log -S <seam-fn>` since the ticket's creation date. A stale headline does
not invalidate the ticket — the remaining scope stands; re-scope, don't re-do.

## L-claude-365-headed-drives-cannot-see-cfg-test-observables-001
*category: design · topic: headed lane / test observability · from: pipeline 365 (headed font-policy verification)*

A headed drive spawns the SHIPPED binary (gpui's `Application::run()` owns the
macOS main thread — subprocess is the only headed model), so every
`#[cfg(test)]` accessor the headless lane leans on (`mono_family_for_test`,
`flash_message_for_test`) simply does not exist in the child; and a
Metal-painted surface (the flash toast) never reaches the AX tree, so AX can't
substitute. Pixels prove presence, never text. The seam that works: a RUNTIME
env-gated self-report in the binary (the `MARLEY_WEBVIEW_PROBE` /
`MARLEY_WIDGET_FIXTURE` precedent) — gate first-thing, exact-match value,
print one machine-parsable state line post-boot, quit; the DRIVE owns the
asserts. Corollary: the only settings-injection lever for a spawned child is
`$HOME` (config resolves `home::home_dir()/.marley/config`, and `home` reads
`$HOME` on unix) — `RootView::new_in(dir)` is `pub(crate)` and unreachable
from outside. Plan locked "assert via *_for_test in-process" on the headless
lane's mental model; design had to amend it — check WHICH PROCESS runs the
assert before locking an observability decision.

## L-claude-423-a-tickets-environmental-diagnosis-is-a-hypothesis-not-a-fact-001
*category: design · topic: test portability / cross-platform lanes · from: pipeline 423 (Linux-portable tests)*

TICKET-423 carried two confident environmental diagnoses and BOTH were wrong in
kind: "the twin-alias fixture is impossible on case-sensitive ext4" — actually
the fixture rides the macOS `/var → /private/var` SYMLINK (case-sensitivity is
irrelevant; the portable fix is an explicitly created symlink alias), and "the
compile_fail snapshot is rustc/platform-sensitive" — actually the box ran rustc
1.94 against the mini's 1.96 (NO toolchain pin existed; the fix is
`rust-toolchain.toml`, after which the snapshot needs no tolerance at all).
Design probed both mechanisms (read the fixture's actual construction; ran
`rustc --version` + the failing test on the box) before locking fixes — and
each probe DISSOLVED the planned remediation into something smaller and more
durable. Rule: a ticket's stated cause for an environment-sensitive failure is
a hypothesis recorded by whoever hit it, usually mid-firefight; verify the
MECHANISM (read the code, run the probe on the failing platform) before
designing tolerance, gating, or normalization — the true fix is often
upstream of all three.

## L-claude-425-type-the-seam-and-the-compiler-finishes-your-site-sweep-001
*category: implement · topic: newtype refactors / seam enumeration · from: pipeline 425 (display-map foundation)*

The Explore sweep enumerated 14 projection crossing sites + the geometry
struct; the typed-geom conversion then FAILED TO COMPILE at two more sites the
sweep had missed — `HoverCard.first_row` (a slot snapshot whose domain existed
only in a doc comment) and the #352 test geom seeder. That is the mechanism
working as designed: for a domain-typing refactor, the site manifest only
needs to be good enough to START — flipping the shared struct's fields to the
newtype turns the compiler into the exhaustive enumerator, and each E0308 is a
site WITH its domain question already answered (what does this value mean
here?). Corollaries: (1) do the STRUCT fields early, not last — every missed
consumer surfaces at once; (2) a field whose domain lives only in prose is
exactly the find — type it, don't just fix the caller; (3) the cheapest home
for new coordinate newtypes is the workspace vocabulary crate that already
carries the macro + trybuild infrastructure (`marley_text_offsets` — the
REQ-005 compile-fail cost one fixture file there vs a gpui-linking test
target anywhere else).

## L-claude-426-live-drives-need-a-click-and-reseeded-geometry-001
*category: validate · topic: selftest harness / headless geometry · from: pipeline 426 (soft wrap)*

Two drive mechanics that cost retries: (1) LIVE app — synthetic chords silently
vanish until a CLICK lands in the window (frontmost alone is not first-responder
enough); the working recipe is click-into-surface → clearmods → chord, and a
palette query typed in the same drive call as the chord can race the open —
screenshot-probe between steps when a verb seems to no-op (the config file's
write-through is the cheap oracle for whether a toggle actually fired). (2)
HEADLESS — the platform draw is a no-op, so ANY render between seeding
`editor_geom` and the probed keystroke resets it to zero; the seed must be the
LAST update before `simulate_keystrokes`, re-applied before each one (the seed
hook's own doc says so — believe it the first time).

## L-claude-427-live-drive-three-traps-001
*captured: 2026-08-14 (#427 validate) · scope: scripts/selftest harness*

Three live-drive traps that each read as "the app is broken" when it isn't:
1. **A rebuilt bundle re-triggers TCC.** Re-bundling ad-hoc re-signs the app, so
   macOS re-prompts "access files on a removable volume" — and the RESTORE's
   first `fs::read` blocks inside `__open` until the dialog is answered: the
   process sits at 0% CPU with ZERO windows (sample shows `open_editor_instance`
   → `std::fs::read` → `__open`). Fix: answer the dialog —
   `osascript -e 'tell application "System Events" to tell process
   "UserNotificationCenter" to click button "Allow" of window 1'` — the harness
   can't see it otherwise (drive.swift needs a marley WINDOW to exist).
2. **`clickat:` takes FRACTIONS (0..1 of the window), not pixels.** Pixel-style
   args ("clickat:490,340") land nowhere, gpui keyboard focus is never set, and
   every following keystroke silently vanishes — the same symptom as the
   focus-first rule, different cause.
3. **`drive.swift cmd:enter` does not reach gpui as ⌘⏎** (letters like `cmd:p`/
   `cmdshift:f` work; the named-key + command-flag combination doesn't land).
   Use System Events instead: `osascript -e '… key code 36 using {command
   down}'`. The headless drive (`simulate_keystrokes("cmd-enter")`) is the
   arm's real proof either way.
Also: the ⌘⇧F worker streams asynchronously — a ⌘⏎ typed in the same burst as
the query races an EMPTY result set and correctly no-ops; wait for the footer
count before materializing.

## L-claude-427-approve-poc-captures-by-pixels-not-presence-001
*captured: 2026-08-14 (#427 validate) · scope: React-first workflow*

The 427 POC's approved capture LOOKED right, but its declared `bg-primary/10`
match wash was rendering on ZERO rows — the materializer tested 0-based
`no - 1` against the corpus's 1-based line numbers, so no line ever carried the
class, and the flat rows read as "subtle wash" to the eye. The Rust port
implemented the DECLARED design (a visible 10% teal wash) and the parity pixel
pass caught the divergence in the POC, not the port. Lesson: at POC approval
time, pixel-sample the DECLARED effects (the same `magick`/PIL probes validate
uses) — "the capture looks like the description" is not evidence the CSS class
fired; a wash, band, or border that renders as the background exactly is
invisible to eyeballs and to a structural diff. When the render and the
declaration disagree, fix whichever side broke its OWN intent (here the POC),
and re-pin the pair by numbers.

## L-claude-428-assert-your-splices-and-drive-your-fixes-001
*captured: 2026-08-14 (#428) · scope: automated editing workflow*

Two of the inspect fixes silently DIDN'T APPLY because python `str.replace`
anchors were written from pre-`cargo fmt` text (no assert, no error) — one was
caught only because a VALIDATE DRIVE tested the behavior (the consent sweep),
the other by a compile error. The rules that held: (1) every scripted splice
asserts its anchor (`assert old in s`) so a miss fails loudly; (2) after a fix
round, GREP-AUDIT that each fix's distinctive token exists; (3) the real
protection is a test that exercises the FIXED behavior — the drive caught what
the audit alone could have missed. Same session, same class: a hand-written
settings blob with raw 0x1F bytes is invalid TOML (the #321 no-hand-blob rule
biting outside the codec — write control chars as unicode escapes when a seed
file must be authored by hand).

## L-claude-429-a-library-fn-needs-its-own-crate-tests-for-the-mutation-gate-001
*captured: 2026-08-14 (#429) · scope: testing / gate mechanics*

Six diff mutants survived the first gate run even though the marley_app
headless drives exercised every one of them end-to-end. Cause: the mutation
lane runs the MUTATED CRATE's own tests — a new `editor` public fn
(`compile_find`/`find_all_compiled`) whose only tests live in a consumer crate
has an MSI hole by construction, and the same shape left `compile_find`'s
empty-pattern arm the workspace's single uncovered line (the app guards empty
BEFORE compiling, so no cross-crate call ever reaches it). Rule: a new public
fn gets a unit test IN ITS OWN CRATE at Phase 4, stating the fn's full rule
set (empty/invalid/valid + an exact-value equivalence row), even when a drive
already proves the behavior. Corollary for comparison mutants: cover the
boundary from BOTH sides with values where the mutated operator genuinely
diverges (a zero-len entry for a `> 0` guard; a (start,len) pair where sum
and product straddle the floor; a range starting exactly AT a newline for a
`pos < start` cursor stop) — the happy-path rows all survive `<`→`<=`.

## L-claude-430-a-a-closure-can-hide-a-dead-arm-from-coverage-001
*captured: 2026-08-15 (#430) · scope: coverage forensics / code shape*

Five gate runs chased "one uncovered line no per-line view could locate" in
multibuffer.rs. The #399 generic-phantom theory (de-genericize to `&dyn Fn`)
did not clear it; rewriting the suspect closures as PLAIN LOOPS exploded the
miss from 1 line to 6 — and thereby UNMASKED it: the selection-normalize's
BACKWARD scan was dead code by construction (a non-empty slot list always
ends with a Line, so the forward scan always finds one). As a `.rev().find()`
closure the whole dead arm cost ONE uncovered line that --show-missing-lines
would not print (the #399 masked-reproduction flag bug); as loops it became
honest uncovered branches. Rules: (1) when coverage flags an unlocatable
line, REWRITE the nearest closure chain as plain control flow — the explosion
localizes it; (2) the fix for a dead defensive arm is DELETION with the
invariant stated (#426's re-expression discipline), not a test that
contrives to reach it; (3) keep `&dyn Fn` on cov-100 pure seams as belt
(one compiled body), but don't expect it to fix what is actually dead code.

## L-claude-431-carry-the-deleted-model-as-the-test-oracle-001
*category: validate · topic: equivalence proofs for consuming refactors · from: pipeline 431 (displaymap excerpt unification)*

The #425 byte-identical recipe proves a facade against the DIRECT model — but a
CONSUMING refactor deletes the direct model, so there is nothing left to
property-equal against. The adaptation that worked: carry the deleted derivation
VERBATIM into test space as the oracle (`oracle_slots` = the #427/#430
`refresh_cum` mint, moved line-for-line into the test), then sweep every stage
method against oracle-DERIVED answers over the discriminating fixture set,
probing past-range and both mover directions from every start slot. The re-homed
pinned tests keep the old model's observable values alive; the oracle keeps its
ALGORITHM alive. Duplicating the derivation in test space is not a PR-1691
violation — production holds one body; the test copy exists precisely to detect
the production body drifting.

## L-claude-431-a-verbatim-move-can-carry-a-duplication-into-plain-sight-001
*category: inspect · topic: re-homing / one-derivation · from: pipeline 431 (displaymap excerpt unification)*

"Moved verbatim" faithfully transported a duplication the old layout had hidden:
`refresh_cum`'s inline forward Line-scan and `move_selection`'s scan lived in
different model methods pre-move, and landed 20 lines apart in the new stage —
where a critic immediately flagged them as two bodies of one derivation
(PR-1691). The fix was a two-line delegation (`normalize`'s non-Line arm =
`move_selection(clamped, down=true)`, equivalence provable: inclusive-scan from
a known-non-Line slot ≡ scan-from-next; both keep the clamp on exhaustion).
**Why:** move-verbatim is the right D1 discipline for the MOVE, but re-homing is
exactly the moment co-located loops become mergeable — check for it AT the move,
or let inspect catch it as it did here. **How to apply:** after re-homing N
methods into one impl, diff their loops against each other before calling the
phase done; a delegation that preserves pinned outputs is byte-identical-safe.

## L-claude-432-resume-a-dead-critic-with-a-fix-delta-note-001
*category: inspect · topic: critic orchestration / usage limits · from: pipeline 432*

Two of four inspect critics died mid-review to an API usage limit. Instead of
respawning fresh (losing their partial file reads and verification state), they
were RESUMED via SendMessage after the limit reset — with a note listing every
fix applied to the diff since they started. Both delivered full final reports
against the CURRENT tree, re-verified the fixes they had not seen, and one
found two additional MEDIUMs (the plain-key double-delivery; the chord-choke
composition gap) precisely because its resumed context already held the ladder
map it had built before dying. **How to apply:** a limit-killed subagent is a
suspended investment, not a loss — resume it with (a) "the diff has been
UPDATED since", (b) the delta list, (c) the unfinished lens questions restated.
Fix confirmed findings from the survivors FIRST so the resumed critics review
the fixed tree, not the one that died under them.

## L-claude-433-per-crate-mutation-ignores-cross-crate-callers-001
*category: validate · topic: mutation / test scaffolding · from: pipeline 433*

The gate's ONE missed mutant was the session-level test-seed DELEGATE's body →
`()`: cargo-mutants runs a crate's mutants against that crate's own tests, so
the marley_app drives calling the delegate cross-crate killed nothing. A
`#[doc(hidden)]` cross-crate scaffold therefore needs a unit IN ITS OWN CRATE
(one 8-line test fixed the gate). **How to apply:** any pub fn added to crate
A for crate B's tests gets an A-local unit at birth — treat "who kills this
crate's mutants" as a per-crate question, never a workspace one. Same run's
harness lesson: chain every drive.swift verb in ONE invocation — synthetic
events sent across separate tool-call shells drop when the host terminal
regains frontmost (observed twice; the README's warning is load-bearing).

## L-claude-434-re-express-unreachable-arms-instead-of-excluding-them-001
*category: validate · topic: coverage discipline / totality · from: pipeline 434*

Gate:4 went red on ONE region: the `if let Some(path) = attribute_path(…)`
None arm — grammatically unreachable (tree-sitter always mints the inner
`attribute` node inside an `attribute_item`), so no fixture can cover it. The
§0 answer is never an exclude: RE-EXPRESS so the arm does not exist —
`attribute_path(p, src).is_some_and(|path| …)` folds the unreachable None into
the already-covered not-a-test path, keeping §14 totality with zero
uncoverable surface (the #426 state-the-invariant-delete-the-mutation-surface
discipline, applied to coverage). **How to apply:** when llvm-cov flags an arm
you believe unreachable, first PROVE it (a fixture attempt — `#[]`, bare `#`
here — which also pins the skip-not-fatal behavior), then re-express with the
combinator that makes the compiler erase the arm; an exclude hides the claim,
a re-expression makes it structural.

## L-claude-435
Three durable lessons from #435's validate.
1) **The uncalled-instantiation coverage class:** gate:4 "missed lines" that
   NO line view shows (llvm-cov table says N missed; `--show-missing-lines`
   and the HTML show none) are UNCALLED CLOSURES/instantiations — e.g.
   per-item closures (`.map`, `.any`) over a vec the test asserts EMPTY. Fix
   by asserting the aggregate directly (`is_empty()`), which is also the
   stronger assert. Diagnose with the JSON export's zero-count FUNCTIONS
   filtered by filename, not with line reports.
2) **Equivalent mutants die by re-expression:** `blocks[i + 1..]` where
   element `i` can never satisfy the scanned predicate leaves `+ → *`
   equivalent-alive. The reverse-walk accumulator (`greened` set) removed the
   index arithmetic entirely — no surface, and O(n) besides. Same discipline
   as the #426/#434 state-the-invariant-delete-the-mutation-surface rule.
3) **The live-drive harness on a machine in use:** synthetic UNMODIFIED keys
   (typing, F8) can fail to deliver machine-wide while ⌘-chords and mouse
   clicks work — and when another interactive session owns focus, typed
   probes risk CROSS-SESSION INJECTION: stop keyboard attempts (the 3-4 try
   rule), drive by MOUSE + crafted boot layout instead. The recipe that
   worked: write the `~/.marley/config` settings.toml shell blob directly
   (back up + restore the user's!) so the app boots INTO the exact surface
   (a Code tab on a temp fixture file), then click-only. A temp failing
   integration test under `crates/*/tests/` gives a REAL failing `cargo
   test` with a REAL panic ref — delete it before the gate.

## L-claude-443-the-editor-plugin-shares-the-target-001
*category: validate · topic: the shared target on the dev box · from: pipeline 443*

The first `.rs` edit a session makes through its own file tools starts that session's
rust-analyzer plugin, and its startup `cargo check --workspace --all-targets` runs in the shared
`/mnt/fast/target`. In #443 it held the build lock for about ten minutes and stalled a FULL gate
that was already running; the gate resumed once the check finished. **How to apply:** before a
long gate, `pgrep -af rust-analyzer` and look for a cargo child under it. Wait for an active
check to finish (never kill a running cargo), and make no `.rs` edits while the gate runs. An
in-place DIFF run's own rewrites did not wake the plugin in #443.

## L-claude-443-run-the-gate-not-a-reconstruction-001
*category: validate · topic: gate changes · from: pipeline 443*

Three separate faults in the ported gate surfaced only when the gate itself ran: the FULL
workers' shared target (a FULL run), the vacuous DIFF (a critic's listing), and the
`--in-place` plus `--jobs` usage error, which that listing missed because it rebuilt the command
without those two flags. The gate throws cargo-mutants' output away, so exit 1 read as "did not
complete a valid run" with no reason. **How to apply:** to prove a change to a gate step, run
that step's exact argument vector (`--list` appended when a real run is too long) and read the
tool's own message; when a step reports a bare exit code, rerun its command line by hand before
guessing. Budget a real `script/gates.sh --diff` into every ticket that edits the gate.


## L-claude-436-a-new-hook-is-live-in-the-same-session-001
*category: validate · topic: Claude Code hooks · from: pipeline 436*

A hook added to `.claude/settings.json` took effect in the session that added it: a probe Edit
to `docs/README.md` minutes later was blocked with the new hook's message. I expected the
opposite, remembering that Claude Code snapshots hooks at startup. A hook's script, as opposed
to its registration, is re-read on every call in any case, so an edit to an existing hook's
file is live at once. **How to apply:** prove a new or changed hook with one harmless probe
through the real harness (an Edit whose effect you can revert, or a Bash command whose text
the hook matches, built at run time), not only with crafted JSON; and expect a hook you wire
up to start judging your own next tool call.

## L-claude-436-no-taskcreate-means-no-mid-phase-stop-001
*category: process · topic: the phase hooks in a harness without TaskCreate · from: pipeline 436*

`enforce-phase-tasks.sh` blocks Stop inside a phase until TaskCreate was called and every task
resolved. This harness offers no TaskCreate, so a Stop anywhere between `/pipeline:plan` and
the archive of the doc pair is refused; the checklists live in the notes instead. **How to
apply:** do not end a turn mid-phase. Wait for background critics inside the turn (the agent
listing, or a bounded wait on something observable), and finish the phase, or reach `/commit`
or an archived pipeline, before stopping.

## L-claude-437-a-debug-marley-starts-inside-the-checkout-001
*category: validate · topic: live drives of the app on the dev box · from: pipeline 437*

A debug build reads its assets from the checkout at run time and finds the checkout by a `.git`
above the executable or the working directory (`util::dev_repo_root`). On this box the build
lands in `/mnt/fast/target`, outside the checkout, so a debug `marley` started from `$HOME` (any
Hyprland launch) panics at `settings::init` with "dev asset loading requires running from within
the checkout". Worse, the panic comes after the installation id is written, so the next launch
counts as an existing install and `agent_ui` backfills Zed's editor layout into the user's
`settings.json`. **How to apply:** start every drive with the checkout as the working directory
(`hl.exec_cmd("sh -c 'cd /srv/stacks/marley_ide && exec /mnt/fast/target/debug/marley ...'")`),
capture its output, and `cmp` the settings file after the first launch.

## L-claude-437-the-headless-live-drive-recipe-001
*category: validate · topic: live drives without touching Chad's screen · from: pipeline 437*

What worked for a GUI drive on this Hyprland 0.56 (Lua config, where `hyprctl dispatch exec
"[rules] cmd"` no longer parses): `rusty headless up`; a temporary rule
`hyprctl eval 'hl.window_rule({ match = { class = "dev.zed.Zed-Dev" }, workspace = "3 silent" })'`
for the headless output's workspace; `hyprctl eval 'hl.exec_cmd("...")'` to launch; poll
`hyprctl clients -j` for the class (Marley's is `dev.zed.Zed-Dev`, stock Zed's `dev.zed.Zed`);
`rusty headless shot <name> <dir>` and read the PNG; SIGTERM the process; `hyprctl reload`
to drop the rule, which would otherwise hide Chad's own Marley windows; `rusty headless down`.
A process's inotify watches (`/proc/<pid>/fdinfo`) show which config directories it really
reads when the screen cannot tell two settings files apart.

## L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001
*category: validate · topic: live drives without touching Chad's screen · from: pipeline 438*

`rusty headless up` gives HEADLESS-2 one of Chad's existing workspaces, not an empty one: on
2026-09-22 it took workspace 3 with three of his windows on it, and a
`hl.workspace_rule({ workspace = "9", monitor = "HEADLESS-2", default = true })` set before
`up` did not change that. The `3 silent` rule in L-claude-437-the-headless-live-drive-recipe-001
therefore tiles Marley among his windows, and a capture can show them: after a layout switch
the maximize had moved to one of his windows, and that capture had to be deleted. Maximize
Marley, and before every capture check in `hyprctl clients -j` that Marley is the workspace's
fullscreen window; delete any capture that shows something else. Three more traps. A
window-targeted dispatch such as `hl.dsp.window.fullscreen({ mode = "maximized", window =
"address:..." })` focuses that window, so hand focus straight back with `hl.dsp.focus({ window
= "address:<his window>" })`. Omarchy sets `misc:focus_on_activate = true`; turn it off for the
drive with `hl.config({ misc = { focus_on_activate = false } })`, and `hyprctl reload` restores
it. The dev channel skips the single-instance check, so a second `marley <path>` starts a
second app (it hung) instead of adding the path to the running window. A drive that needs keys
or clicks needs a compositor of its own (none is installed: no sway, cage or weston) or Chad
away from the desk; while he is at it, send no input.

## L-claude-438-the-coverage-floor-counts-lines-per-function-001
*category: validate · topic: gate:4 line coverage · from: pipeline 438*

gate:4 reads llvm's line summary, which adds up each function's own lines. A line on which
some function has only a region that never ran is missed there, even when the file view and
the lcov `DA` records show it covered and `--show-missing-lines` lists nothing. In #438 that
was a `?` alone on the line after a multi-line closure argument (`})?;`). `cargo llvm-cov
report --json` gives each function's regions; the missed line is the one where a function's
only regions have count 0. llvm also reports as missed the closing-brace line of an `if let`
on a weak handle's `upgrade()` whose fall-through no test reaches, when it ends a function or
closure. The fixes that read naturally: end the chain with `.map(|()| value)` instead of `?`,
and let a handler take the weak handle and return a `Result` its caller logs, as Zed's
sidebar does with `weak.update(..)`. The coverage table listed neither `#[path]` test module
(`rail_tests.rs`, `marley_workbench_tests.rs`), so their lines are outside the floor.

## L-claude-438-recent-projects-needs-its-test-support-in-tests-001
*category: validate · topic: feature unification in test builds · from: pipeline 438*

A crate that links `recent_projects` and turns on `project`'s or `workspace`'s `test-support`
in its dev-dependencies must turn on `recent_projects`' as well
(`recent_projects = { workspace = true, features = ["test-support"] }`). `project`'s test
support enables `remote/test-support`, which adds a `Mock` variant to
`RemoteConnectionOptions`, and `remote_connection` matches it only under its own
`test-support`, which `recent_projects/test-support` enables. Without it, `cargo check
--tests` fails inside `remote_connection` with a non-exhaustive match. cargo-shear accepts a
dev-dependency entry that only adds features to a normal dependency.

## L-claude-438-prove-a-views-own-notify-with-a-selector-001
*category: validate · topic: gpui driven tests · from: pipeline 438*

To test that a view redraws because of its own `cx.notify()` and not because something else
dirtied the window: draw a clean frame (`window.refresh()`, then park), change its state
through an event that notifies nothing else (a terminal bell: `TerminalView` sets `has_bell`
and emits `Wakeup` without a notify), then read `cx.debug_bounds(..)` for an element only the
new state draws, without refreshing again. A mutant that drops the notify, or inverts the
guard around it, then fails on the missing selector. `debug_selector` exists only in test
builds. A flag another entity reads is tested with the view closed as well
(PR-claude-state-another-entity-reads-is-kept-outside-render-001).

## L-claude-447-clippys-cargo-group-reads-the-whole-workspace-001
*category: code · topic: lint tables in Zed's workspace · from: pipeline 447*

clippy's `cargo` group reads every manifest in the workspace, not only the crate it lints. In
Zed's workspace, `cargo_common_metadata`, `negative_feature_names` and
`redundant_feature_names` report Zed's packages (`html_to_markdown`'s metadata) and Zed's
`test-support` convention from any Marley crate's run, so the Marley tables allow those three
with a comment. Their diagnostics carry no source span. A count that filters clippy's JSON by
the primary span's file showed zero while the deny-level run failed, and it missed a hit whose
primary span sat in gpui's `actions!` macro as well. Count with the gate's own command at deny
level (L-claude-443-run-the-gate-not-a-reconstruction-001).

## L-claude-447-semgreps-rust-parser-and-its-file-list-001
*category: gate · topic: semgrep 1.156 on Rust · from: pipeline 447*

Two traps. semgrep's Rust parser reads `&raw.payload`, where `raw` is a binding, as the start of
a raw borrow (`&raw const`) and gives up on the rest of the expression; `--strict` makes that a
red (exit 3), and renaming the binding fixes it. And semgrep scans only the files git lists, so a
new untracked file goes unscanned unless the run passes `--no-git-ignore`. With explicit
`crates/marley_*` targets nothing else is swept in. `SEMGREP_ENABLE_VERSION_CHECK=0` and
`--metrics=off` keep the gate off the network.

## L-claude-447-gpui-code-under-rustals-lints-001
*category: code · topic: pedantic and nursery in a gpui crate · from: pipeline 447*

What worked in `marley_workbench`:
- `actions!` passes per-action attributes through, so `#[derive(Eq)]` on each action answers
  `derive_partial_eq_without_eq` without an allow.
- `#[gpui::test]` accepts `cx: &TestAppContext`: it matches the reference's last path segment
  and passes `&mut`. So `needless_pass_by_ref_mut` is fixed in the test's signature.
- `cx.on_action(..).on_action(..).observe_global::<S>(..).detach()` leaves no `&mut App`
  unused.

Two lints do not fit gpui and are allowed in the manifest, each with its reason:
`future_not_send`, because gpui's contexts are not `Send`, and `unused_results`, because of the
`&mut App` chaining returns and `.log_err()`'s `Option`. For `unreachable_pub` against
`redundant_pub_crate`: clippy treats `pub(super)` as `pub(crate)` in a module one level deep,
so a shim only one module calls became that module's child, through `#[path]` to keep its file
where it was.

## L-claude-447-smoke-a-gate-by-extracting-its-functions-001
*category: validate · topic: proving gate changes · from: pipeline 447*

`script/gates.sh` runs every gate when invoked, so a smoke of one check cannot call the script.
What worked in #447:
- Extract each function by name with awk: a multi-line body ends at a lone `}`, a one-liner at
  `; }`. Then `eval` it along with the variables it reads.
- Plant the fault in an untracked file, or in a tracked one backed up first.
- Check the exit status, then restore.
- Compare `git diff | sha256sum` and the untracked list before and after.

Forty-two smokes ran in a few minutes against the shipped code. Two things to know when a real
run uses a planted file as its red: an untracked `.rs` that no module declares also trips
cargo-shear's unlinked-file check, and a whole static pass is under a minute once the build is
warm.

## L-claude-439-a-popover-menu-takes-focus-only-on-a-platform-frame-001
*category: validate · topic: gpui driven tests · from: pipeline 439*

`PopoverMenu` focuses the menu it opens two frames later (`window.on_next_frame`, twice), and a
test window runs next-frame callbacks only when the platform asks for a frame
(`TestWindow::simulate_frame_request`, private to gpui). `run_until_parked` and
`window.refresh()` never ask. So in a driven test the focus stays where it was, and
`menu::SelectNext`, `SelectChild` or `Cancel`, dispatched after opening a popover, reach the old
focus and do nothing. Drive a popover's menu with the pointer. Its entries carry
`MENU_ITEM-<label>` debug selectors. A submenu's trigger carries none, but the row under a
known entry can be hit by position, and a submenu anchors to its trigger's bounds from the
frame before it draws, so draw twice before reading its entries.

## L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001
*category: validate · topic: gpui driven tests · from: pipeline 439*

Focusing a panel's focus handle while its dock is closed does nothing that
`contains_focused` sees: the panel is not drawn, so its handle is in no dispatch path, and the
`on_focus_in` listeners on it do not fire. Open it the way a user does,
`workspace.focus_panel::<P>(window, cx)`, which opens the dock and moves focus in.

## L-claude-439-a-manifest-edit-makes-the-next-gate-rewrite-cargo-lock-001
*category: gate · topic: the receipt's tree check (#447) · from: pipeline 439*

After a dependency leaves or joins a manifest, the first cargo command anywhere rewrites
`Cargo.lock`. When that command is the gate's own gate:2, the lockfile changes mid-run. The
receipt step then fails every run that followed the edit, all the other gates green, because
the tree at the end is not the one the run started on. Run one cargo command (a `cargo check`
of the crate) after a manifest edit and before the gate, or expect the first gate run to fail
there and run it again.

## L-claude-439-the-agent-panel-test-recipe-001
*category: validate · topic: driving Zed agent threads in a Marley test · from: pipeline 439*

What worked in `marley_workbench`:
- `init_agent_test` sets `TestMetadataDbName` and `TestTerminalMetadataDbName` to a name of
  the test's own, then runs `agent_ui::test_support::init_test`. That installs its own
  settings store and database, so it replaces the crate's `init_test` rather than following
  it. After it come `ThreadStore`, `ThreadMetadataStore`, `LanguageModelRegistry::test`,
  `prompt_store::init` and `terminal_view::init`.
- Per workspace, `AgentPanel::test_new` and `add_panel`.
- `StubAgentConnection` with no queued updates holds a turn open, so the thread runs until
  `end_turn(session, StopReason::EndTurn)`. `with_permission_requests` plus a queued
  `SessionUpdate::ToolCall` for the same id leaves the thread waiting for a confirmation.
- Custom agents come from a user-settings string, `{"type": "custom", "command": "..."}` under
  `agent_servers`. `AgentRegistryStore::init_test_global` supplies registry names and icons.
- An agent id of `stub` resolves to `Agent::Stub` and its thread-local connection, so a menu
  entry for a configured `stub` agent starts a real thread in the test.

## L-claude-440-testing-agent-clis-without-a-pty-001
*category: validate · topic: driving terminal agents in a gpui test · from: pipeline 440*

A gpui test's terminals are display-only (`TerminalBuilder::new_display_only`), with no PTY and
no process. What worked in #440:
- `foreground_process_command_name` returns `None` for them, so recognition goes through a
  seam, `fn(&Entity<Terminal>, &App) -> Option<String>`, and the test's version reads a
  global keyed by the terminal's entity.
- The PATH lookup goes through `which::which_in` over a search path the test points at a
  `tempfile` directory. The files need the executable bit (`PermissionsExt::from_mode(0o755)`),
  so the tests are `#[cfg(unix)]`.
- `Terminal::take_pty_write_log` (test-support) returns every write, PTY or not, so a test can
  assert exactly what a launch sent. `start_init_command_startup_handshake` finishes at once
  without a PTY, so the launch runs its one code path.
- `TerminalView` forwards `Wakeup` and turns a `Bell` into a `Wakeup` for its subscribers; a
  `TitleChanged` or `BreadcrumbsChanged` reaches them as an `ItemEvent`. Emitting on the
  `Terminal` entity is enough to drive the rail.
- `cx.executor().advance_clock(WAITING_AFTER)` fires the quiet timer.

## L-claude-441-driving-tasks-and-terminal-actions-in-a-gpui-test-001
*category: validate · topic: the terminal routing's driven tests · from: pipeline 441*

What worked in #441 (`crates/marley_workbench/src/routing_tests.rs`):
- Real shells and tasks, as Zed's panel tests run them, with `cx.executor().allow_parking()`.
  Parking lets real time pass and moves the test clock with it, so a test can await
  `Workspace::spawn_in_terminal` and poll a shell with executor timers.
- Load the panel with `TerminalPanel::load`, then `add_panel`, as `crates/zed` does. `load`
  installs Zed's provider, so a provider that has to replace it meets the real order;
  `TerminalPanel::new` installs none.
- Dispatch an action from inside the workspace: focus the center pane, draw a frame
  (`window.refresh()`), then `cx.dispatch_action`. Listeners on the workspace's root are on
  the dispatch path only when focus is below it.
- A missing program is not a failed spawn. `spawn_task` runs the command in the task's shell,
  which reports exit status 127; a task fails to start only when its shell is missing
  (`Shell::Program("__nonexistent_shell__")`). Zed's own failed-spawn test skips the shell by
  calling `add_terminal_task`.
- A shell's `Terminal::working_directory()` is known only after its first output, so poll it.
- A switch back to the Zed layout builds Zed's sidebar, which reads the agent stores
  (`init_zed_sidebar`).
- Test the second run, not only the first. Zed's reuse paths, such as a task's rerun, look for
  the old terminal wherever it is. #441's rerun gap passed seven tests and a green gate before a
  rerun test found it (F-claude-441-a-rerun-reopened-the-hidden-terminal-panel-001).

## L-claude-449-driving-keys-and-docks-in-a-gpui-test-001
*category: validate · topic: key and dock tests in a Marley crate · from: pipeline 449*

What worked in #449 (`crates/marley_workbench/src/routing_tests.rs`):
- `settings::KeymapFile::load_asset_allow_partial_failure(settings::DEFAULT_KEYMAP_PATH, cx)`
  returns Zed's real default bindings whose actions the test binary links, so a crate's test
  can bind them and press the keys users press. `DEFAULT_KEYMAP_PATH` is the platform's file,
  so a test that presses Linux's keys is `#[cfg(target_os = "linux")]`.
- Draw a frame (`window.refresh()`) before each dispatch or keystroke. Dispatch looks the
  focused element up in the last rendered frame and falls back to the window's root when it
  is not there (`crates/gpui/src/window.rs:6244-6252`), so after a focus change the action
  would start above the workspace's listeners.
- `workspace::item::test::TestItem` stands in for an editor in the center, and
  `workspace::dock::test::TestPanel` for another panel in a dock. Docks sort panels by
  activation priority (`dock.rs:784-795`), so a test panel's priority decides its index; the
  Terminal Panel's is 2.
- `use super::*` brings an underscore import's methods into scope but not its name. A test
  that names the trait in a bound (`V: Focusable`) imports it itself.

## L-claude-450-driving-a-picker-and-a-keymap-in-a-marley-test-001
*category: validate · topic: modal pickers and keymap loading in a Marley crate · from: pipeline 450*

What worked in #450 (`crates/marley_workbench/src/agents_tests.rs`):
- A modal takes focus in a deferred callback (`modal_layer.rs:199-200`), so after the action
  that opens it and `run_until_parked`, `menu::Confirm` and `menu::SelectNext` dispatched from
  the focused element reach the picker as Enter and the arrow keys would.
- `Picker::set_query` takes `&mut App` while the picker is read, so clone the inner
  `Entity<Picker<_>>` out of the modal first, then `update_in` it.
- A loader that takes the keymap text as a parameter (`load_keymap_from`) lets a unit test
  reach its failure arms: text that is not JSON, and a keymap naming an unknown action, which
  `KeymapFile::load` reports while still returning the bindings that did load.
- A user's own binding is a `cx.bind_keys` after the Marley keymap, on a test action with a
  `cx.on_action` handler that records the press: the test then sees the user's action run,
  not only the picker staying shut.
- `crates/zed`'s keymap tests check a hook in `load_default_keymap` with `init_keymap_test`,
  `load_default_keymap`, `cx.clear_key_bindings` and `reload_keymaps(cx, Vec::new())`.
- An agent row or entry drawn with an SVG icon needs `AgentRegistryStore::init_test_global`
  with an agent whose metadata carries `icon_path`, as #439's registry test does.

## L-claude-442-driving-a-window-restore-in-a-test-001
*category: validate · topic: persistence tests in a Marley crate · from: pipeline 442*

What worked in #442 (`crates/marley_workbench/src/marley_workbench_tests.rs`):
- Zed's own restore path runs in a test: build a `workspace::MultiWorkspaceState` (its fields
  are public) and await `workspace::apply_restored_multiworkspace_state` with the window's
  handle (`window.window_handle().downcast::<MultiWorkspace>()`), `<dyn Fs>::global(cx)` and
  `cx.to_async()`.
- A second launch is a second window: `open_projects` again once the first window's
  `VisualTestContext` borrow has ended, then `register` builds its sidebar as `crates/zed`
  does, before the restore.
- `MultiWorkspace::serialize` spawns, and `serialize_now` reads the sidebar's blob a turn
  later, inside the `MultiWorkspace`'s update. A flag a sidebar sets from its observer on the
  `MultiWorkspace` is therefore in the next save, and the sidebar must not read the
  `MultiWorkspace` while producing the blob.
- Test the blob as JSON (`serde_json::Value`), not as a string: field order and additions
  break a byte-for-byte comparison without changing what either sidebar reads.

## L-claude-451-catching-a-zed-crates-actions-in-a-marley-test-001
*category: validate · topic: tests that need another Zed crate's actions · from: pipeline 451*

What worked in #451 (`crates/marley_workbench/src/marley_workbench_tests.rs`):
- To catch an action a Zed crate declares, depend on that crate for its types, and in
  `[dev-dependencies]` repeat it with `features = ["test-support"]` when the test build turns on
  another crate's test support: `title_bar` matches `remote`'s `Mock` connection only under its
  own `test-support`, so without it `cargo check --tests` failed in `title_bar` itself.
- A stand-in for the Zed crate's handler, registered with `workspace.register_action` as the
  crate registers its own, shows whether an action got through, without that crate's `init`
  and the globals it needs.
- A toast's primary button has no debug selector to click by. Hand `Toast::on_click` a named
  function rather than a closure, and have the test call that function; the wiring is then one
  line in the code under review.

## L-claude-452-driving-a-rows-menu-hover-and-double-click-in-a-test-001
*category: validate · topic: pointer tests on rail rows · from: pipeline 452*

What worked in #452 (`crates/marley_workbench/src/rail_tests.rs`):
- `ui::right_click_menu` opens on a right-button press: `simulate_mouse_down` and
  `simulate_mouse_up` with `MouseButton::Right` at the row's bounds, then one more frame before
  the menu's `MENU_ITEM-<label>` selectors are drawn.
- A double-click is two `MouseDownEvent`/`MouseUpEvent` pairs with `click_count` 1 and 2 at one
  position; `ClickEvent::click_count` then reads 2 on the second.
- `ListItem::end_slot_on_hover` lays its element out while hidden, so a test moves the pointer
  over the row (`simulate_mouse_move`) before clicking it, as a user must.
- Zed's inline tab rename finishes from a test with `simulate_input` into the focused rename
  editor, a fresh frame, and `menu::Confirm`.

## L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001
*category: validate · topic: testing a key context against Zed's default keymap · from: pipeline 453*

What #453 found (`crates/marley_workbench/src/rail_tests.rs`,
`zeds_default_keys_walk_and_open_the_rail`):
- Zed's default keymap binds up, down, Home, End, Page Up, Page Down, Enter and Escape to
  `menu::*` with no context (`assets/keymaps/default-linux.json:3-22`), so they reach any
  focused element that handles those actions. Left and right (`menu::SelectParent`,
  `SelectChild`) are bound only in the `menu` context (`:51-56`).
- The first keystroke test pressed up, down and Enter, and still passed with `menu` taken out
  of the rail's key context: it proved the handlers, not the context. Pressing left and right
  made it fail without `menu`.
- Take the context out and watch a keystroke test fail before trusting it, as
  `PR-claude-a-negative-assert-must-prove-the-machinery-ran-001` asks of a negative assert. An
  assertion whose expected value is also what the unchanged code gives cannot fail either: the
  first REQ-004 check expected the window's row, which was also the keyboard's.

## L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001
*category: code · topic: a text field inside a keyboard list · from: pipeline 457*

What #457 relied on, and proved with Zed's keymap bound (`crates/marley_workbench/src/rail_tests.rs`,
`ctrl_f_reaches_the_filter_and_zeds_keys_work_in_it`):
- In a single-line `Editor`, `editor::MoveUp`, `editor::MoveDown` and `editor::Cancel`
  propagate (`crates/editor/src/navigation.rs:52-55`, `crates/editor/src/editor.rs:3553`), and
  gpui then tries the next binding for the key (`crates/gpui/src/window.rs:5944-5957`). Up, down
  and Escape arrive as `menu::SelectPrevious`, `menu::SelectNext` and `menu::Cancel` on the
  container's handlers, as in Zed's pickers. Enter needs nothing: only `Editor && mode == full`
  binds it.
- A container that handles `menu::Cancel` must `cx.propagate()` when it has nothing to do, or
  it swallows what Escape reached before, here `workspace::Unfollow`.
- `HighlightedLabel::new` takes UTF-8 byte offsets and debug-panics on one that is not a char
  boundary (`crates/ui/src/components/label/highlighted_label.rs:17-32`). Compute the positions
  on the exact string the row draws, and test with a multibyte character before the match.
- A binding in a panel's context also fires from the popovers the panel opens, which keep it as
  their dispatch parent (`window.rs:4338`). `!Picker` keeps a key out of a picker's field.

## L-claude-454-driving-a-hold-and-release-switcher-in-a-gpui-test-001
*category: validate · topic: tests for switchers that confirm when a modifier is released · from: pipeline 454*

What worked in #454 (`crates/marley_workbench/src/rail_tests.rs`, the W6e section):
- `simulate_keystrokes("ctrl-tab")` sends only a KeyDown and leaves `window.modifiers()`
  unchanged, so a switcher that records the held modifiers as it opens sees none. Hold them
  first with `cx.simulate_modifiers_change(Modifiers::control())`, then open it, then let go
  with `simulate_modifiers_change(Modifiers::none())`.
- Modifier events reach only the focused path. A view in `MultiWorkspace::set_sidebar_overlay`
  is not focused by the overlay: focus it as you install it, or the release never arrives.
  Removing that one `window.focus` failed the release and Escape tests.
- `Sidebar::toggle_thread_switcher` runs deferred. Two dispatches in one `cx.update` reach it
  before the first has opened the switcher, which is how a test reaches the hook's "already
  open" arm.
- A recency rule keyed on what the window displays notes rows that pass by during one action.
  Key it on what holds focus
  (F-claude-454-recency-noted-a-terminal-the-user-never-went-to-001), and give the test inputs
  where the two rules disagree.

## L-claude-455-driving-a-real-open-and-restore-through-new-local-001
*category: validate · topic: tests of what Zed does when it opens or reopens a folder · from: pipeline 455*

What worked in #455 (`crates/marley_workbench/src/routing_tests.rs`, the W6f section):
- `Workspace::new_local` runs in a Marley test. `init_test` provides the settings and a test
  database (`db::AppDatabase::test_new`), `cx.update(AppState::test)` the app state, and the
  folder is put on the app state's `FakeFs` (`app_state.fs.as_fake().insert_tree`). A real
  `tempfile` directory at the same path lets a terminal's shell start there.
- A real restore is two opens of the same roots with `flush_all_serialization` and
  `window.remove_window()` between them. The second `new_local` finds the saved row and
  restores the project's items. `requesting_window: None` and `Some(window)` run the two
  different closures that build the workspace, so test both.
- Restoring a project whose saved center has panes swaps that center in, which drops anything
  added to the workspace before the restore finished. A terminal seeded at creation disappears
  (and its shell with it), so only a project saved with an empty center shows a seed that
  should not be there. Put the negative check there.

## L-claude-456-acting-around-zeds-own-settings-observers-001
*category: code · topic: reacting to what Zed's observers do on a settings change · from: pipeline 456*

What #456 relied on (`crates/marley_workbench/src/marley_workbench.rs`, `layout_setting_changed`):
- gpui runs a global's observers in the order they were registered. An observer registered in
  `marley_workbench::init`, before any window opens, runs before every dock's or panel's
  observer. So it sees their state before they react to a settings change, and a `cx.defer`
  from it runs after they have.
- That is how Marley can put its own step around a Zed behavior it cannot change without a
  touchpoint: note the state first, then settle it in the defer. Zed's throttled save
  (`serialize_workspace`, 200 ms) has not fired by then, so the settled state is the one saved.
- A test that removes the deferred step reproduces the original bug. That is the negative check
  that proves the fix, and it is cheap because the defer is one line.

## L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001
*category: code · topic: reading the `MultiWorkspace`'s groups after a folder change · from: pipeline 458*

What #458 found (`crates/marley_workbench/src/rail.rs`, `follow_folders`):
- A folder added to or removed from a project is reported twice, in this order:
  `project::Event::WorktreeAdded` or `WorktreeRemoved`, then `WorktreePathsChanged`
  (`Project::on_worktree_store_event`, then `emit_group_key_changed_if_needed`).
- The `MultiWorkspace` rekeys the project's group on the second event only. Between the two,
  the workspace's own `project_group_key` has changed and the stored group's key has not, so
  `MultiWorkspace::project_groups` lists that group with no workspace in it.
- Code that reads the groups on the first event finds the project nowhere. In the rail that
  dropped the project's rows for one rebuild, and the bookkeeping that forgets rows once they
  are gone forgot their switcher recency and their threads' attention dots.
- Defer the read with `cx.defer_in`. The Defer effect queues behind the second event, so it
  runs after the rekey, whatever order the subscriptions were made in. Zed's Threads Sidebar
  is safe the same way: `schedule_update_entries` spawns its update.
- The negative check is one line: refresh in the handler instead of deferring.
  `a_folder_added_or_removed_renames_the_row_and_keeps_its_rows_recency` then fails on the
  switcher's order.

## L-claude-448-running-zeds-dylint-library-on-the-fork-001
*category: tooling · topic: Zed's `tooling/lints` as a Marley gate · from: pipeline 448*

What #448 needed to know (`script/gates.sh`, gate:21):
- **Install.** `cargo install cargo-dylint dylint-link --locked` (6.0.4), and `rustup
  toolchain install` from `tooling/lints`, which reads its toolchain file
  (`nightly-2026-03-21` with `rustc-dev`, `rust-src`, `llvm-tools-preview`). The first
  `cargo dylint` builds the library and a driver for that nightly (about a minute).
- **The tree builds on the older nightly.** The root pins 1.98.1 and the library 1.96-nightly.
  Nothing in the workspace sets `rust-version`, and a cold check of the 691 crates the Marley
  crates reach took 1m 52s on the dev box, with no compile error. Recheck this when upstream
  bumps either pin.
- **Where it builds.** The check keeps `$CARGO_TARGET_DIR/dylint/target/<toolchain>` and the
  library `$CARGO_TARGET_DIR/dylint/libraries`, apart from the normal target directory.
- **Scoping without reading output.** The library's lints warn, and Zed's crates hold about
  600 hits. `#![cfg_attr(dylint_lib = "lints", deny(...))]` in a crate root makes them errors
  only there and only under the driver. A normal build never sets the cfg, and the Marley
  manifests allow `unexpected_cfgs`. dylint's own crates use the same idiom.
- **The cache follows the library.** dylint 6.0.4's driver adds each loaded library to the
  crate's dep-info and hashes the libraries' contents into rustc's tracked options
  (`dylint_driver/src/untracked_state.rs`), so a rebuilt library re-runs the check without a
  clean.
- **rustfmt and a same-line justification.** An inner attribute with a trailing `//`
  comment, such as `#![allow(clippy::expect_used)] // why`, must stay the last inner
  attribute. With another inner attribute after it, rustfmt moves the comment to its own
  line, and gate:12 then fails the `allow`, since it reads the justification on the same line.
- **Guarding a cargo command.** Guard with `pgrep -x cargo`. `pgrep -f "cargo (… |install)"`
  matched the shell running the guarded `cargo install`, so the guard reported busy and
  skipped the install.

## L-claude-460-a-fresh-install-drive-beside-chads-own-window-001
*category: validate · topic: live drives on the dev box · from: pipeline 460*

- `marley --user-data-dir <scratch dir> <folder>` starts a second app with a profile of its own
  (config, database, logs under that directory), so it behaves as a fresh install while
  Chad's own window keeps running. The dev channel skips the single-instance check, and with a
  separate data directory the second app does not hang (compare
  L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001). End it by its own pid
  and remove the directory.
- A fresh profile opens a new folder behind Zed's Restricted Mode prompt ("Unrecognized
  Project"), which is upstream's worktree trust, not a Marley behavior.
- On 2026-09-23 a launch through `hyprctl eval 'hl.exec_cmd("…/marley …")'` exited silently
  after "set environment variables from shell", with no window and no panic in the log. `setsid
  -f` from a shell holding the session's `XDG_RUNTIME_DIR`, `WAYLAND_DISPLAY` and
  `HYPRLAND_INSTANCE_SIGNATURE` worked.
- While Chad has this repository open in Marley, its rust-analyzer runs `cargo check
  --workspace` after edits to the tree, as flycheck. Check `pgrep -x cargo` and wait for it
  before the next cargo command.

## L-claude-461-carrying-an-upstream-crate-in-vendor-001
*category: tooling · topic: vendoring a git dependency · from: pipeline 461*

What #461 learned vendoring `alacritty_terminal`:
- **Cargo takes a patch into an excluded directory.** `[patch."https://github.com/zed-industries/alacritty"]`
  with a path under `vendor/`, plus `exclude = ["vendor"]`: `cargo tree` resolves the copy,
  and `Cargo.lock` drops only the entry's `source` line.
- **Workspace-inherited fields break.** `edition.workspace = true` and
  `rust-version.workspace = true` refer to the upstream workspace, which is not copied, and
  Zed's `[workspace.package]` has no `rust-version`. Spell out upstream's values.
- **Licence symlinks.** Upstream's crate directory keeps `LICENSE-APACHE` as a symlink to the
  repository root; copy the real file.
- **Standalone runs write a lockfile.** `cargo test --manifest-path vendor/<crate>/Cargo.toml`
  writes `vendor/<crate>/Cargo.lock`. An untracked lockfile under `vendor/` changes the receipt
  fingerprint, so commit it before any gate step runs such tests.
- **Leave recordings upstream.** alacritty's `tests/ref` is 46 MB, and all the crate's `typos`
  hits are in it.

## L-claude-464-zeds-task-terminals-keep-the-maximum-history-001
*category: validate · topic: testing scrollback in Zed's terminal · from: pipeline 464*

`TerminalBuilder::new` gives every task terminal `MAX_SCROLL_HISTORY_LINES` (100,000),
whatever `max_scroll_history_lines` says (`crates/terminal/src/terminal.rs:1202-1207`), and
Zed's PTY test helpers build task terminals. A PTY test cannot make such a terminal evict lines
without printing 100,000 of them. #464's first eviction test printed 500 lines into a two-line
history and found all 500 still held. Test eviction on a `Term` built from `pty_term_config(n,
..)` and fed with a `vte` processor instead, as `absolute_lines_text`'s test does, or build an
interactive terminal.

## L-claude-463-proving-a-shell-script-before-wiring-it-001
*category: validate · topic: shell integration scripts · from: pipeline 463*

- **Run the script under the real shell on a pseudo-terminal first**, before any Rust:
  `printf 'echo hi\nfalse\nexit\n' | HOME=<scratch> script -qfc "bash --rcfile <script> -i"
  /dev/null | cat -v`. The DCS frames show as `^[P…^[\` in order with the output. #463's
  script was right the first time it met Zed's `TerminalBuilder`, because this run had already
  checked each frame.
- **bash's preexec seam without a `DEBUG` trap is `PS0`.** It is expanded and printed after a line
  is read and before it runs, so `PS0+='$(fn)'` prints the frame ahead of the command's output.
  Inside it, `fc -ln -0` gives the line just read, provided history kept it.
- **A scratch `HOME` keeps a test off the user's `.bashrc`**, and a marker echoed from a scratch
  `.bashrc` proves the user's file is still sourced through `--rcfile`.

## L-claude-467-see-a-process-marley-changes-where-zed-shows-it-001
*category: validate · topic: live drives of the terminal · from: pipeline 467*

When Marley changes how a process starts (arguments, environment, a wrapper), the live check
looks at the places Zed shows that process: the tab title, the rail row, the tab's tooltip.
`ps` proves the process runs as intended; it cannot show what the user reads. Zed titles a
local shell `<cwd name> — <process name> <arguments>` from the foreground process
(`Terminal::title`), so an added argument is on screen at once.

## L-claude-467-capture-one-window-by-its-toplevel-001
*category: validate · topic: live drives without touching Chad's screen · from: pipeline 467*

`grim -T <stableId>` captures a single window by its `ext-foreign-toplevel-list` identifier,
which Hyprland 0.56 prints as `stableId` in `hyprctl clients -j`. The window can sit on a
workspace nobody is looking at, so no headless output is needed and no workspace of Chad's is
borrowed (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001). The recipe,
all through `hyprctl eval`:
`hl.window_rule({ match = { class = "dev.zed.Zed-Dev" }, workspace = "9 silent", render_unfocused = true })`,
`hl.config({ misc = { focus_on_activate = false } })`, launch from the checkout with
`--user-data-dir` on a copy of the profile, wait for the client, `grim -T`, SIGTERM the pid from
`hyprctl clients -j`, `hyprctl reload`. `render_unfocused` is a window rule;
`misc.render_unfocused` does not exist (`unknown config key`), and a failed eval exits 7.

## L-claude-468-sample-the-capture-before-trusting-a-theme-token-001
*category: code · topic: gpui styling in the rail · from: pipeline 468*

Zed's theme tokens are named for their roles, and in One Dark several share a value:
`border` (70,75,87) matches `ghost_element_selected` (69,74,86), and `element_background`
(46,52,62) matches `panel_background` (47,52,62). A border in `border` around a selected row,
or a fill in `element_background` on the panel, is therefore invisible. Read the pixels of the
live capture (`magick <png> -format "%[pixel:p{x,y}]" info:`) for each layer before settling a
color, and for "a step lighter than what it sits on" use the text color at a low alpha, which
steps the right way in dark and light themes alike.

## L-claude-465-zsh-reads-an-unquoted-replacement-by-context-001
*category: code · topic: shell integration scripts · from: pipeline 465*

- **Quote zsh's `${var//pattern/replacement}` when the replacement holds a backslash.** At a
  script's top level `r=${v//;/\;}` leaves `;` bare, while `r="${v//;/\;}"` gives `\;`;
  inside a function both give `\;`. The bash script's unquoted forms are right in bash and
  were no model for zsh. The quoted form reads the same everywhere.
- **zsh gives every `precmd` hook the command's `$?`.** A hook that returns 3 does not change
  what the next hook sees after `false` (1). Hook order still decides who prints first.
- **zsh's new-user menu checks `ZDOTDIR` before any startup file runs**, so a `ZDOTDIR` that
  holds a `.zshenv` suppresses it. Test the no-files case with an empty `HOME` under
  `script -qfc "zsh -i"`.

## L-claude-465-a-doc-opens-with-one-short-line-001
*category: code · topic: clippy in the Marley crates · from: pipeline 465*

`clippy::too_long_first_doc_paragraph` (nursery, an error in the Marley crates) fails a doc
comment whose first paragraph runs long, and it has failed gate:2 in #467 and #465 on the same
day. The first paragraph of a doc is one short line saying what the item is; every detail goes
in a paragraph after a blank `///`. Write it that way first, not after the gate.

## L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001
*category: code · topic: clippy in Zed's crates · from: pipeline 465*

Zed's `clippy.toml` disallows `std::process::Command::spawn`, `output` and `status` (and
`stdin`, `stdout`, `stderr`) everywhere, test code included, with `smol::process::Command` as
the replacement. gate:2 lints the touched Zed crate's tests, so a test that shells out to see
whether a program is installed fails it. To check for a program, scan `PATH`
(`std::env::split_paths(&path).any(|dir| dir.join(name).is_file())`); to run one, use `smol`.
Run `cargo clippy -p <every touched crate> --all-targets -- -D warnings`, Zed's crates too,
before the gate.

## L-claude-470-the-alternate-grid-counts-its-own-evicted-lines-001
*category: code · topic: blocks and the alternate screen · from: pipeline 470*

While a program holds the alternate screen, `term.grid()` is the alternate grid, and the
vendored `evicted_lines` counts the lines that grid scrolls away: entering it with
`\e[?1049h` clears it, which scrolls it, so in #470's test its evicted count read 49 right away.
Anything built from the active grid's evicted lines and history (`marley_screen_top`, a hook's
absolute line) is in the alternate grid's frame of reference then, not the scrollback's. That is
why blocks draw nothing on the alternate screen, and why a driven test of that guard passed
without it: the blocks happened to map above the alternate viewport. Test such a guard at the
function that holds it, with the content built by hand.

## L-claude-470-a-script-edit-skips-the-ledger-hook-001
*category: code · topic: the Zed ledger · from: pipeline 470*

`enforce-zed-ledger.sh` runs on the Write and Edit tools. An edit made by a script through Bash
(a `python3` patch, `sed -i`) never meets it, so a Zed path can change with no row; in #470 the
driven tests went into `crates/terminal_view/src/terminal_view.rs` that way, and gate:16 caught
it at the end. Before a scripted edit of a path outside the Marley-owned set, write its row, as
the hook would have required.

## L-claude-470-a-pty-test-child-that-exits-can-lose-its-last-bytes-001
*category: validate · topic: PTY tests · from: pipeline 470*

A PTY test whose script prints and exits at once can lose its last bytes under load: when the
child exits, alacritty's event loop drains with a single `pty_read`, and a read error there ends
the drain (`vendor/alacritty_terminal/src/event_loop.rs:266-277`). #470's pill test passed alone
in 0.2 s and failed one run in three with its suite's neighbours, 32 seconds without the last
frames. End such a script with `sleep 60` (the test drops the terminal, and the child with it):
four runs under the same load passed. #464's `build_shell_hook_terminal` scripts still exit at
once.

## L-claude-474-an-occluding-child-ends-its-groups-hover-001
*category: code · topic: gpui hover and the terminal · from: pipeline 474*

`occlude()` gives an element a `BlockMouse` hitbox, and the hit test stops at the first one
under the pointer (`crates/gpui/src/window.rs:1119`), so no hitbox behind it is hovered, the
group's own included. A child that occludes inside `visible_on_hover(group)` hides as soon as the
pointer reaches it. To keep a press on an overlay button from the element under it, stop the
press instead, in a wrapper: `on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())`.
Bubble listeners run front to back, and the terminal registered its own before the block
painted, so they run after the wrapper's. `ButtonLike` does not stop the press itself: on a left
press it only calls `prevent_default`, and it stops the release in its click handler
(`crates/ui/src/components/button/button_like.rs:872-876`).

## L-claude-474-the-input-log-misses-mouse-reports-001
*category: validate · topic: terminal tests · from: pipeline 474*

`Terminal::take_input_log` records only what `Terminal::input` sends. Mouse reports and focus
reports go out through `write_to_pty`, which `take_pty_write_log` records, input included.
#474's first test that a press on a button reaches no program read the input log, and it still
passed with the wrapper removed; the negative check of
PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001 caught it. A test that
rules out a write to the PTY reads the PTY write log, and first shows it can see such a write:
in #474 a press on the block's output, reported, comes before the presses on buttons that must
not be.

## L-claude-476-zeds-snapped-rows-can-leave-a-gap-below-the-grid-001
*category: validate · topic: terminal geometry · from: pipeline 476*

`TerminalElement::prepaint` counts rows with the line height rounded to whole device pixels
(`rows = available / round(line_height)`) and sets the grid's height to those rows, while the
grid lays its lines out at the fractional line height and `num_lines` divides by it. The two
can disagree: in #476's test window, 1024 px snapped to 1008, and 55 lines of 18.2 px take 1001,
so the grid's last line ends 7 px above the element's edge, on a full screen too. A test of
where the bottom row is drawn allows up to a row between it and the edge, or measures against
the grid's own last line.

## L-claude-476-a-fresh-folder-gets-a-first-terminal-for-a-capture-001
*category: validate · topic: the live drive · from: pipeline 476*

`just shot` copies Chad's profile, so it opens his last session, which may show another
project or a project with no terminal left. `OPEN=<path>` opens a path as `marley <path>` does,
and a folder with no saved state gets a first terminal in the Marley layout, which a seed's
terminal settings then shape. Zed shows its trust prompt for an unknown folder over the top of
the window; the terminal's rows stay visible under it.

## L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001
*category: validate · topic: terminal tests · from: pipeline 477*

`Terminal::foreground_process_command_name` reads the process info that a `Wakeup` refreshes,
and a `Wakeup` comes with output. A process that takes the foreground and prints nothing, such as
a stand-in `claude` linked to `sleep`, is not seen until something else is written. In a driven
test, send a key the tty echoes (a space) until the process shows; in a live capture, make the
stand-in print, as a real agent redraws its screen. Start the stand-in by name from a PATH entry,
or with `exec -a claude`: the command name comes from argv, and a path in argv[0] gives none.

## L-claude-478-a-new-terminal-event-reaches-every-exhaustive-match-001
*category: code · topic: Zed's terminal events · from: pipeline 478*

`terminal::Event` is matched exhaustively outside the terminal crates: `TerminalView`'s
subscription and the Agent Panel's (`agent_ui/src/agent_panel.rs`) both name every variant. A new
variant breaks `agent_ui`, which a `cargo check -p terminal_view` never builds. Search for a
variant only that enum has (`NewNavigationTarget`) to find every match before adding one, and
give each Zed file its ledger row.

## L-claude-478-omarchys-notifications-go-to-quickshell-001
*category: validate · topic: the live drive · from: pipeline 478*

On this box `org.freedesktop.Notifications` belongs to Omarchy's Quickshell shell
(`quickshell -n -p /usr/share/omarchy/shell`), not mako, and `makoctl` is not installed. To see a
notification Marley posts from the hidden workspace, run
`busctl --user monitor org.freedesktop.Notifications` during the shot: it shows the `Notify` call
and the id the server returns, which `CloseNotification` then takes down again, so a test leaves
nothing on Chad's screen.

## L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001
*category: validate · topic: Claude Code hooks · from: pipeline 482*

Claude Code writes a hook's `terminalSequence` through the last writer its interactive UI has
registered (`nye` → `H9().write`, registered with `nir` by the REPL). `claude -p` registers
none, so the hook runs, answers, and the sequence goes nowhere. Test a hook's terminal
sequence with an interactive session: a Python pty that starts `claude --plugin-dir <plugin>
"<prompt>"` and reads the raw output, in a folder Claude Code already trusts that has no
settings of its own (`/srv/stacks` on this box), so no trust prompt and no project hooks.

## L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001
*category: code · topic: gpui tasks and Zed's lints · from: pipeline 482*

Blocking work off the main thread, in a Marley crate, is
`cx.background_spawn(futures::future::lazy(move |_| …))`. `background_spawn(async move { … })`
around code that never awaits fails Zed's dylint `async_block_without_await`, an error in the
Marley crates. `smol::unblock` passes the lint but wakes gpui's test scheduler from smol's
threads, and every test that runs the crate's `init` then panics with the scheduler's
non-determinism error. A closure that only runs in production, such as a `which` fallback that
every test replaces with a fake, leaves a function no test calls: gate:4 counts it, and
`cargo llvm-cov report --json` names it; move such work where a test does run it.

## L-claude-479-drive-a-file-chooser-through-the-test-platforms-path-prompt-001
*category: validate · topic: gpui tests · from: pipeline 479*

gpui's test platform queues every system path prompt (`prompt_for_paths`). A test answers it with
`cx.simulate_path_prompt_response(|options| …)`: the closure sees the `PathPromptOptions` and
returns the chosen paths, or `None` for a cancel. It panics when several paths answer a prompt
that allows one, and `cx.did_prompt_for_paths()` says whether one is waiting. The default
settings keep `use_system_path_prompts` on, so in tests a local project's
`Workspace::prompt_for_open_path` takes this path. There is no need to turn the setting off and
inject a prompt with `set_prompt_for_open_path`, which would also leave the default path untested.

## L-claude-483-send-keys-to-one-hyprland-window-by-address-001
*category: validate · topic: e2e on Hyprland · from: pipeline 483*

Hyprland 0.56 takes dispatchers as Lua: `hyprctl eval 'hl.dispatch(hl.dsp.send_key_state({
mods = "CTRL", key = "g", state = "down", window = "address:0x…" }))'`, then the same with
`state = "up"` about 50 ms later (Omarchy's bindings split the halves because a whole
`send_shortcut` can leave a key stuck). The key reaches that window on a hidden workspace, and
the user's active window and workspace stay as they were. `hyprctl dispatch <name> <args>` is
Lua shorthand now, so the old `sendshortcut MOD, KEY, class:…` syntax fails to parse, and
`hyprctl eval` prints only "ok", never a return value. The stubs are
`/usr/share/hypr/stubs/hl.meta.lua`. Keys take xkb names (`Return`, `BackSpace`, `minus`); a
capital is `SHIFT` and the lowercase key.

## L-claude-483-a-scenario-brings-its-own-shell-001
*category: validate · topic: e2e scenarios · from: pipeline 483*

A scenario that types at a shell prompt gives the terminal a HOME of its own
(`terminal_env HOME "$E2E_WORK/home"`, whose `.bashrc` it writes). The user's own prompt makes
the shot depend on their machine: with a long starship prompt, typed text after the first
character never showed on the line though bash received it (TICKET-485), while a plain `$ `
prompt echoed every key. Pin the shell for the scenario's own proof, and give what the user's
shell exposed its own ticket and its own scenario, one that sets that prompt up on purpose.

## L-claude-480-an-e2e-fake-acts-out-the-program-001
*category: validate · topic: e2e scenarios · from: pipeline 480*

A scenario proves what a click-only button does through an action that does the same, run from
the command palette (`press "CTRL SHIFT" p`, `type_text "<action words>"`, Enter), and through a
fake that acts the program out so the result shows on screen. #480's fake `voxtype` on
Marley's PATH follows a status file, and its `record toggle` moves the file on as Voxtype does:
recording, then transcribing, then idle a few seconds later. The microphone's colors in the
shots then prove that the toggle ran and that the status was followed, with nothing read from a
log.

## L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001
*category: code · topic: gpui keys on Linux · from: pipeline 481*

`KeyDownEvent::prefer_character_input` is always `false` on gpui's Linux platforms, so a handler
that treats a key as text only when it is set treats no key as text there. Stopping a key event's
propagation in a `key_down` listener also keeps it from the focused element's text input. To
keep keys from an ancestor such as the terminal view's `key_down` while text still reaches an
editor inside it, stop only keys with Ctrl, Alt or Super, or with no `key_char`; the terminal
maps no plain character (`to_esc_str`), so those go to the editor. A stand-in agent started as
a subshell from `.bashrc` never leads the terminal's foreground process group (job control is
off while bash reads its startup files); `exec -a claude` in place of the shell does.

## L-claude-485-log-the-bytes-a-shell-writes-inside-marley-with-script-001
*category: validate · topic: e2e diagnosis · from: pipeline 485*

When a terminal shows something the program did not seem to write, log the program's own bytes
inside Marley: give the scenario's copy a `terminal.shell` of `script -q -f -c "<the shell>"
<log>`, and read the log after the steps. `script`'s header records the PTY's size when the shell
started, and the log shows what the program wrote key by key. A reproduction outside Marley can
hide a race with the terminal's first layout: a pty sized before the shell starts is never
resized under it. And timing that depends on the first frame differs on the hidden workspace,
where an unfocused window draws slowly, so a race the scenario hits may be rarer on a screen in
use.

## L-claude-484-where-a-typed-command-starts-without-touching-the-prompt-001
*category: code · topic: terminal shell integration · from: pipeline 484*

A terminal that knows when its shell reached a prompt (`precmd`) can find where the command typed
there starts without marking the prompt: the cursor at the first key the user sends after it.
Record it in the grid's absolute coordinates (`HookPosition::of`, the hooks' own), clear it at
the next `precmd` or `preexec`, and read the typed text from the cells between it and the cursor
while the cursor stays on that line. It is wrong only when a key is typed before the prompt has
drawn, and then nothing matches, so a suggestion built on it shows nothing.

## L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001
*category: validate · topic: e2e in a headless sway · from: pipeline 487*

A sway started with `WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1` has a seat with
`capabilities: 0` (`swaymsg -t get_seats`): no client ever gets a `wl_pointer` or a
`wl_keyboard`, so `swaymsg seat seat0 cursor press` changes nothing and a one-shot `wtype`
loses its keys while the app is still binding the keyboard it just gained. Hold the devices for
the whole run: a client of the wlr-virtual-pointer protocol that stays connected and moves,
clicks and scrolls on command (`script/e2e/seat-pointer.c`), and `wtype -s 86400000` for the
keyboard; later `wtype` calls then only switch the keymap. gpui binds devices that appear after
it started. With `default_border none` the one window fills the output, so the output's pixels
are the window's, and a scenario reads its click targets from its own first shot.

## L-claude-488-chromium-reports-a-new-url-not-the-documents-title-001
*category: code · topic: CDP targets · from: pipeline 488*

With target discovery on, Chromium sends `Target.targetInfoChanged` when a page's URL changes,
with the URL standing in as the title, and `Target.targetCreated` for the targets that exist
when discovery starts. It sends nothing when the document's `<title>` arrives, although
`Target.getTargets` reports it a moment later. A tab that shows the page's title asks for it:
`Target.getTargetInfo` after `Page.domContentEventFired`, `Page.loadEventFired` and
`Page.navigatedWithinDocument`. A title a single-page app changes later, with no navigation,
still goes unseen.

## L-claude-488-an-overlay-from-any-session-shows-in-every-screencast-001
*category: code · topic: CDP screencast · from: pipeline 488*

The Overlay domain's highlights are drawn into the page's compositor frame, so a highlight one
CDP session sets (`Overlay.highlightNode`, or inspect mode's hover) shows in the screencast
frames of every other session on that page. In #488's run, a stand-in agent's highlight of a
button, with its accessibility tooltip, showed in Marley's Browser tab. It ends when that
session detaches, so a client that highlights stays connected while the highlight must show.
The element picker (B3a) can use DevTools' own inspect mode for its hover, drawn once for
everyone watching.

## L-claude-489-each-wtype-resets-gpuis-compose-001
*category: validate · topic: e2e keys under sway · from: pipeline 489*

Every `wtype` process creates a virtual keyboard with a keymap of its own, and the seat hands
its keymap to the focused client each time the active keyboard changes (gpui logs "Received
keymap format NoKeymap, expected XkbV1" as the held keyboard, which has none, comes back). gpui
drops a pending xkb compose sequence when that happens, so Multi_key, `'` and `e` sent by three
`press` calls type `'e`. Send a sequence in one `wtype` call: the harness's `press_keys
Multi_key apostrophe e` types é.

## L-claude-489-zlog-filters-by-the-crate-a-line-comes-from-001
*category: code · topic: logging · from: pipeline 489*

Zed's logger matches a `ZED_LOG` or `RUST_LOG` directive (`name=level`) against the crate and
module the line was logged from, not the `log` target. `log::debug!(target: "marley_browser",
…)` written in `marley_workbench` stays hidden under `ZED_LOG=marley_browser=debug`; log it
from `marley_browser`'s own code, or filter on `marley_workbench::browser`. Marley's log is
`<data dir>/logs/Marley.log`, not stdout; the e2e harness copies it beside the shots as
`<scenario>.marley.log`.

## L-claude-489-a-fixture-that-marks-presses-must-not-take-them-001
*category: validate · topic: e2e fixtures · from: pipeline 489*

A fixture page that draws a mark where each press lands, placed over the target, must give the
mark `pointer-events: none`. Otherwise the mark, added by the `mousedown` listener, is the
topmost element when the release comes, so `mouseup` lands on it, and `click`, `dblclick` and
`contextmenu` go to the common ancestor instead of the target. In #489's first run it looked
exactly like a lost release; logging every mouse event on the page showed Marley's release
arriving.

## L-claude-490-page-navigate-answers-at-the-commit-001
*category: build · topic: CDP navigation · from: pipeline 490*

`Page.navigate` answers once the navigation commits or fails, not when it starts: a page whose
response takes three seconds holds the call for three seconds, and a stopped navigation answers
with `errorText` `net::ERR_ABORTED`. A client can keep "where the page is going" from the call
until its answer (or the main frame's `frameNavigated`) and show it meanwhile, as an omnibox
does; only a timed-out call leaves the navigation running.

## L-claude-490-a-failed-load-commits-chromiums-error-page-001
*category: build · topic: CDP navigation · from: pipeline 490*

A navigation that fails (a name that does not resolve, a refused connection) commits Chromium's
error page: `Page.frameNavigated` reports the main frame's `url` as
`chrome-error://chromewebdata/` and the URL that failed as `unreachableUrl`. An address bar that
reads `url` shows the internal page; read `unreachableUrl` first.

## L-claude-490-a-view-that-forwards-keys-sees-its-editors-keys-001
*category: build · topic: gpui key dispatch · from: pipeline 490*

A view that sends its keys somewhere else from `on_key_down` (the Browser tab sends them to the
page) gets its child editors' keys too: a key typed in the address bar bubbles through the
tab's listener after the editor has handled it. Gate the forwarding on the view's own handle,
`focus_handle.is_focused(window)`, not `contains_focused`. gpui's `handle_input` already
registers an input handler only for the exactly focused handle, so text reaches the editor
alone. #481's rich input solves the same problem the other way round, stopping the keys at the
editor's container.

## L-claude-491-ctrl-q-in-a-terminal-goes-to-the-shell-001
*category: validate · topic: e2e scenarios · from: pipeline 491*

Zed's Linux keymap binds `ctrl-q` to `zed::Quit` in `Workspace` and to
`["terminal::SendKeystroke", "ctrl-q"]` in `Terminal`, so a scenario whose focus is in a
terminal does not quit Marley with Ctrl+Q. Quit through the palette: Ctrl+Shift+P, `zed: quit`,
Enter. The same holds for every key the `Terminal` context sends on (Ctrl+R among them).

## L-claude-491-hook-stamps-are-when-the-main-thread-applies-them-001
*category: build · topic: terminal blocks · from: pipeline 491*

A block's `BlockTimes` are stamped when Zed's terminal applies each shell hook on the main
thread, not when the shell sent it. In #491's run `sleep 1` measured 957 ms: the `Preexec` came
in about 40 ms late while the main thread drew the typed command. Durations are right to tens of
milliseconds; a figure closer than that would need the time in the hook itself (bash's
`$EPOCHREALTIME` in the `preexec` and `precmd` frames) or a stamp taken where the vendored event
loop parses the frame.

## L-claude-491-a-session-per-agent-needs-room-and-a-close-001
*category: build · topic: MCP sessions · from: pipeline 491*

`marley_mcp`'s session registry refuses new sessions at its cap and never evicts, and an idle
session lives 30 minutes. Sized at 8 for one manager seat, it would have refused a desk of
agents once each Claude Code session's bridge held one, and a bridge that ended without closing
kept its slot for half an hour. #491 raised the cap to 32 and made the bridge send `DELETE` for
its session when its input ends. A client of Marley's MCP server that opens a session should
close it.

## L-claude-492-runtime-and-log-replay-network-does-not-001
*category: build · topic: CDP domains · from: pipeline 492*

Turning on `Runtime` and `Log` in a CDP session delivers what they had already buffered (console
messages, and log entries such as "Failed to load resource"), so a console ring looks complete
even when it came on late. `Network` delivers only requests made after `Network.enable`. A
missing request next to a console line about it means the network domain came on too late, not
that Chromium dropped the event.

## L-claude-492-chromium-shows-a-password-by-length-001
*category: security · topic: accessibility tree · from: pipeline 492*

`Accessibility.getFullAXTree` gives a password field's value as one bullet a character
(`•••••••`), with a `StaticText` child holding the same bullets, and no property marks the field
as a password. A snapshot that writes values leaks the length. Marley's snapshot writes no
values; `browser_look` reads the focused field's value in an isolated world, where
`type === 'password'` is known, and drops it and its selection there.

## L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001
*category: build · topic: Zed panes and focus · from: pipeline 493*

`Pane::add_item` with `focus_item` false still moves the focus when the item lands in the pane
that has it: the item comes to the front, the focused item behind it stops rendering, gpui
reports the focus lost, and the workspace's `on_focus_lost` gives it to
`focus_lost_restore_target`, the pane, which focuses its new front item. To add an item without
the focus, activate the old front item again in the same update (the new one then waits in the
tab bar), or put it in a pane without the focus. A split for it takes the focus too:
`Workspace::add_pane` focuses the new pane, so put the old focus back before the update ends.

## L-claude-493-a-move-between-panes-is-a-remove-then-an-add-001
*category: build · topic: Zed items · from: pipeline 493*

Zed calls `Item::on_removed` from `Pane::_remove_item`, for a close and for a move between panes
alike; a move adds the item to its new pane in the same update, after `on_removed`. A check
deferred from `on_removed` (`cx.defer`) runs once both are done, but `Workspace::pane_for`
answers from `panes_by_item`, which the panes' `RemovedItem` and `AddItem` events update, and
those events are queued behind the defer: it sees the item in no pane. Ask each pane
(`Pane::index_for_item`) instead.

## L-claude-493-headless-chromium-lives-on-with-no-pages-001
*category: build · topic: CDP targets · from: pipeline 493*

Headless Chromium (the `--headless` mode Marley starts) keeps running after its last page
closes, and `Target.createTarget` still opens a page on the same connection (a probe on a
scratch profile, 2026-09-25). Closing the last Browser tab can close its page without taking
the browser down. The browser also lists `browser_ui` and `background_page` targets, which come
and go with no page among them.

## L-claude-494-a-created-page-keeps-the-window-of-its-first-size-001
*category: build · topic: CDP and headless Chromium · from: pipeline 494*

In headless Chromium each page `Target.createTarget` makes has a window of its own, 780 by 580
unless the call names a size, and `Emulation.setDeviceMetricsOverride` changes the page's
viewport, not that window. A page laid out wider than its window can stop producing screencast
frames after a resize when it holds an out-of-process iframe; the startup page did not show it.
Size the window with the viewport: `Browser.getWindowForTarget`, then
`Browser.setWindowBounds`, then the override. With `--no-startup-window` a start opens no page,
and without it or a URL headless Chromium opens `chrome://newtab/`.

## L-claude-494-zed-item-ids-change-at-each-launch-001
*category: build · topic: Zed item persistence · from: pipeline 494*

An item's `ItemId` is its entity id, new at each launch. Zed deserializes an item under its
saved id, saves it again under its new one when it joins the workspace, and then runs `cleanup`
with the loaded items' new ids, so an item's own table keeps one row per live item only if it
saves on `added_to_workspace` (Zed does that for every serializable item) and its `cleanup`
deletes the rest (`delete_unloaded_items`). A table keyed by anything but the item id outlives
its items.

## L-claude-495-a-listener-and-a-binding-catch-what-headless-chromium-hides-001
*category: build · topic: CDP, isolated worlds · from: pipeline 495*

To act on something headless Chromium opens out of sight (a `<select>`'s popup), a listener in
an isolated world beats a hit test before every press. `Runtime.addBinding` with
`executionContextName` exposes a function in the worlds of that name, which
`Page.addScriptToEvaluateOnNewDocument` with `worldName` makes for each new document and
`Page.createIsolatedWorld` makes for one already loaded; its calls arrive as
`Runtime.bindingCalled` on the session, with the calling world's context. The world's listeners
see the page's own events: `preventDefault` on a select's `mousedown` keeps the popup shut, and
the page's listeners still run. Events the world dispatches reach the page's listeners as not
trusted.

## L-claude-495-a-menu-at-a-point-in-a-view-001
*category: build · topic: Zed UI · from: pipeline 495*

A `ui::ContextMenu` at an arbitrary point is the view's to draw: build it with
`ContextMenu::build`, keep the entity and a `DismissEvent` subscription in one field, render it
as `deferred(anchored().position(p).anchor(Anchor::TopLeft).snap_to_window_with_margin(…))`
with priority 1, and focus its handle. `select_toggled_or_first` starts it on the checked
entry. An entry's handler runs inside the menu's own update and may clear the field, dropping
the menu and its subscription there; the menu's dismissal after the confirm then reaches no
one, which Zed's editor relies on too.

## L-claude-496-chromium-picks-an-element-over-cdp-001
*category: build · topic: CDP, the element picker · from: pipeline 496*

Chromium's own element picker works over CDP in headless Chromium. `Overlay.setInspectMode`
with `searchForNode` and a `highlightConfig` (`showInfo`, `showAccessibilityInfo`) draws
DevTools' highlight and tooltip into the screencast frames as the pointer moves; a click
dispatched with `Input.dispatchMouseEvent` then raises `Overlay.inspectNodeRequested` with the
deepest node's `backendNodeId` and does not reach the page. Mode `none` takes the highlight away
before a `Page.captureScreenshot` crop, whose clip is in page coordinates. `Debugger.enable`
replays `Debugger.scriptParsed` for the scripts loaded already, with each URL and
`sourceMapURL`, and `Debugger.setSkipAllPauses` keeps a `debugger` statement from freezing the
page. `DOMDebugger.getEventListeners` takes one object at a time; a function that returns the
element's ancestors, document and window as an array, read with `Runtime.getProperties`, gives
each object's id and description. Listener lines and columns count from 0.

## L-claude-497-chromium-loads-a-source-map-for-its-client-001
*category: build · topic: CDP, source maps · from: pipeline 497*

A CDP client loads a page's source map the way Chromium's own tools do:
`Network.loadNetworkResource` with the page's target id as `frameId` (a page's main frame id is
its target id) and `options {disableCache, includeCredentials}` answers `resource.success`, the
status and an `IO` stream; `IO.read` gives the text (or base64 with `base64Encoded`) until
`eof`, and `IO.close` frees it. A missing map answers `success: false` with its status, not an
error. `Debugger.scriptParsed`'s `sourceMapURL` is as the script's comment wrote it, often
relative, so it is joined to the script's URL first. A listener's `lineNumber` and
`columnNumber` are its handler function's start, which a line-level map still places on the
right source line.

## L-claude-498-draw-over-a-streamed-page-from-the-frame-drawn-001
*category: build · topic: gpui over a screencast · from: pipeline 498*

Anything a view draws over a streamed page (a box, a label) is placed in render from the frame
drawn in the same render and that frame's own metadata (scroll, pinch scale, top offset, DIP
over the drawn size), as absolute children of a clipping container, not from a mapping saved at
the last paint, which lags a scroll by a frame. A child with no hitbox lets the pointer through
to the page element under it; a child that must take a click (a note's chip) `occlude()`s, so
the page element's `hitbox.is_hovered` is false and its own press handler does nothing.

## L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001
*category: validate · topic: e2e scenarios · from: pipeline 498*

A scenario that clicks a toolbar button by its window coordinates breaks silently when a later
ticket adds a button beside it: #496's scenario clicked the pick button's old place, which #498
gave to the annotate button, so its rerun turned on the wrong mode and made no pick, while
every step "passed". A ticket that adds to a toolbar reruns the scenarios that click that
toolbar, and reads their logs, not only their exit codes.

## L-claude-499-a-screencast-sends-frames-only-when-the-page-changes-001
*category: build · topic: CDP screencast, the flight recorder · from: pipeline 499*

`Page.startScreencast` sends a frame when the page's pixels change, not on a clock: a still
page sends none, so a recorder that keeps "two frames a second at most" keeps far fewer on a
quiet page (seven in ten seconds of typing and clicking in #499's run). A recording's frames
mark changes, and its length comes from its entries, not from a frame count times the rate.
Keeping a frame is cheap when the loop hands back the base64 it decoded instead of copying it
before the decode.

## L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001
*category: validate · topic: e2e scenarios · from: pipeline 500*

A `ContextMenu` a `PopoverMenu` opens on a click starts with its first entry selected, so in a
scenario one Down reaches the second entry and Enter confirms it; #500's first run pressed Down
twice and opened the third. Take the menu's shot after the steps, so the highlight shows which
entry Enter will run.

## L-claude-501-zeds-agents-take-tools-from-the-projects-context-servers-001
*category: build · topic: Zed's agents, MCP · from: pipeline 501*

Every agent of Zed's Agent Panel takes MCP tools from the project's context servers: the Zed
Agent through its profile (the default `write` has `enable_all_context_servers: true`), and each
external ACP agent through `session/new`'s `mcpServers`, which `mcp_servers_for_project` builds
from the merged `context_servers` setting (stdio and HTTP entries; a registry-only descriptor is
not handed over). So a server added to the defaults with `SettingsStore::update_default_settings`
reaches all of them at once, and a stdio entry that runs a bridge keeps any bearer out of the
settings, where an HTTP entry's headers would sit in plain view. `agent: open settings` now opens
Zed's Settings window at its AI page.

## L-claude-502-a-dev-channel-panic-reaches-stderr-only-001
*category: build · topic: panics, the dev channel · from: pipeline 502*

On the `dev` channel Zed installs no crash handler: `should_install_crash_handler` wants
`ZED_GENERATE_MINIDUMPS` or a non-dev channel with a minidump endpoint, and otherwise
`crashes::force_backtrace` is the whole panic hook (`crates/zed/src/main.rs:419`). Panics unwind,
so there is no coredump either. A panic is printed to stderr and the process exits, with
nothing in `Marley.log`. A Marley started from the menu has no terminal for stderr, so the
installed build's launcher appends stderr to `logs/stderr.log` (#502). A debug Marley started by
an agent with `setsid -f` should send stderr to a file too, never `/dev/null`. To tell a quit
from a death afterwards, read `telemetry.log` beside `Marley.log`: with metrics on it ends with
an `App Closed` event at a normal quit. (On 2026-09-25 #502 first read the end of Chad's
`Marley.log` at 20:11:05 as a death; `telemetry.log` showed `App Closed`, a quit.)

## L-claude-502-omarchy-starts-an-entry-through-uwsm-app-and-gtk-launch-001
*category: validate · topic: starting Marley from the menu · from: pipeline 502*

Omarchy's menu starts an app with `uwsm-app -- gtk-launch <id>.desktop`
(`/usr/share/omarchy/shell/services/AppLibrary.qml`), in a scope under `app-graphical.slice`.
`uwsm-app` asks its daemon for a command line and `eval`s it in the caller's own environment, so
a scenario can send the menu's exact path into its headless sway: `WAYLAND_DISPLAY` and
`SWAYSOCK` for the display, `XDG_DATA_HOME` pointing at a scratch prefix's `share` (where
`gtk-launch` then finds the entry, and where Marley keeps its data) and a scratch
`XDG_CONFIG_HOME`. Give the seat a keyboard first (`hold_keyboard`): after the run's last `wtype`
exits, a new client gets no keymap (F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001).
The Marley it starts sits in `app-Hyprland-gtk\x2dlaunch-*.scope` even in sway.

## L-claude-502-the-release-build-on-the-dev-box-001
*category: build · topic: release builds · from: pipeline 502*

`cargo build --release -p zed --bin marley` took 9 min 57 s on the dev box with the shared
target's release directory already holding Zed's dependencies (137 crates compiled), and under a
second when nothing changed. The binary is 2.1 GB with the release profile's `debug = "limited"`
(407 MB after `objcopy --strip-debug`, which takes under a second); the debug sections are never
loaded, and they keep file and line numbers in a backtrace, so the installed copy keeps them.
`desktop-file-validate` hints at `Utility;TextEditor;Development;IDE;` (Zed's categories: two
main categories); `Development;IDE;` draws no hint.

## L-claude-512-show-the-bug-on-the-unfixed-build-first-001
*category: validate · topic: proving a fix · from: pipeline 512*

For a bug, write the scenario at Plan and run it on the build before the fix, where it has to
fail the way the bug report says; then fix, rebuild and run it again. #512's scenario failed on
the unfixed build at the step the panic predicts ("no window from the second Marley") and passed
after the fix, which proves both that the scenario reaches the bug and that the change removes
it. A scenario only ever seen green could be testing something next to the bug. Orca's audit
folders do the same by reversing the patch in memory (docs/orca_architecture/07, §A8).

## L-claude-512-the-gate-receipt-binds-head-001
*category: workflow · topic: the commit receipt · from: pipeline 512*

The receipt's fingerprint (`gate_state_hash` in `.claude/hooks/lib-hook-helpers.sh`) is a hash
of `git rev-parse HEAD` followed by the gated files (the crates, scripts, hooks, e2e scenarios and
config; not `docs/`). So a commit between a green `--diff` and the code's own commit, even a
docs-only one, voids the receipt, and the commit hook then demands a new gate run. Commit the
gated change first and docs-only work after it, or put both in one commit. Docs edits alone
(CHANGELOG, knowledge, `docs/planning/`) never touch the fingerprint, which is why Complete's
paperwork after the gate is safe.

## L-claude-515-dispatch-through-the-window-from-inside-an-action-001
*category: build · topic: gpui actions · from: pipeline 515*

An action handler that forwards to another action must dispatch through the window it runs in:
register it on the workspace (`workspace.register_action(|_, action, window, cx| ...)`) and call
`window.dispatch_action(Box::new(..), cx)`. `cx.dispatch_action` from inside an action that the
command palette (or any window-level dispatch) is running logs `window not found` and does
nothing, because the window is out of the app's map during its own update. A test that calls
`cx.dispatch_action` from the outside would not see it; only the palette path shows it.

## L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001
*category: validate · topic: e2e under sway · from: pipeline 516*

In a `compositor sway` scenario, any new window (the Settings window, a dialog that is its own
window) tiles beside Marley's main window and halves its width, and the terminal rewraps. Until
TICKET-544 lands a rewrap scrambles every block before it, so a scenario that reads blocks
(`terminal_read`, the bars and pills in a shot) does so before it opens another window, and
proves what comes later through something that has no rows to lose (a Browser tab's console, a
settings file).

## L-claude-516-fake-secrets-are-put-together-at-run-time-001
*category: validate · topic: e2e fixtures with secrets · from: pipeline 516*

A scenario that needs secret-shaped values builds them in its setup from pieces (`"gh" + "p_" +
"Fake" * 9`, `"-----BEGIN OPENSSH " + "PRIVATE" + " KEY-----"`), low-entropy on purpose, and
writes them and the list the checks use into `$E2E_WORK`. The scenario file then holds nothing
gitleaks takes for a secret, the checks grep the run's own files rather than literals, and the
notes quote the markers, never the values.

## L-claude-513-gpui-asks-for-activation-from-the-window-it-activates-001
*category: platform · topic: Wayland activation · from: pipeline 513*

gpui's `Window::activate_window` on Wayland asks for an xdg-activation token with the window it
activates as the token's surface and its last pointer press as the serial (0 until the first
press), then activates with it. sway checks a token against the seat's focus, so a window asking
from behind another window, or from behind another app, is not focused; Omarchy's Hyprland, with
`misc:focus_on_activate = true`, focuses on any activation (the e2e runner turns that off during
Hyprland runs for this reason). A scenario that proves a Marley asked to come forward reads its
Wayland trace (`WAYLAND_DEBUG=client`, `xdg_activation_v1.activate`), not the compositor's focus.
gpui also keeps one pending activation: two tokens asked for at once log "activation token
received with no pending activation" for the second.

## L-claude-513-a-second-launch-reaches-wayland-before-the-check-001
*category: platform · topic: Marley's startup · from: pipeline 513*

A launch that hands off to the running Marley still builds gpui's application first, so it
connects to the compositor before Zed's single-instance check runs and exits; its Wayland trace,
when one is on, fills its output. A scenario prints such a launch's own lines by filtering the
trace out (`grep -v '^\['`), not by tailing.

## L-claude-517-a-running-bash-script-reads-its-file-as-it-goes-001
*category: workflow · topic: editing scripts · from: pipeline 517*

bash reads a script file as it executes it, a block at a time, so an edit to a script that is
running changes what it runs next: text inserted before the point it has reached shifts the byte
offset it resumes from, and it can run a line twice, run half a line, or skip one. #517 edited
`script/install-marley` while `just install` was inside its `cargo build`; the committed file went
back within the minute, before the build returned, and the edit was re-applied after the install
ended. Before editing a script, check that nothing runs it, and never edit one mid-run. The check
itself has its own trap: `pgrep -f install-marley` in a one-shot `bash -c` matches that shell,
whose command line holds the pattern (compare the cargo guard in the lessons above); look for the
process by its own name or read `/proc/<pid>/cmdline`.

## L-claude-544-when-a-fix-fails-exactly-like-the-bug-instrument-001
*category: validate · topic: debugging a fix · from: pipeline 544*

When the scenario fails after the fix in exactly the way it failed before, the fix most likely
never ran: prove that before reasoning about its logic. #544's remap was right, but its guard was
always false; one temporary `log::info!` on the path, read from the run's copied `Marley.log`
(`$SHOT_DIR/<scenario>.marley.log`), showed in one run that the code was never reached. Remove the
line before the gate.

## L-claude-547-marleys-path-is-the-login-shells-001
*category: validate · topic: e2e fakes · from: pipeline 547*

A PATH that `script/e2e.sh`'s setup exports reaches the programs in Marley's terminals only
through the scenario's own `.bashrc`; what Marley itself looks up with `which` can come from the
login shell's profile, which on this box puts `~/.local/bin` (the real `claude`) first. #480's
Voxtype fake worked only because nothing named `voxtype` sits in `~/.local/bin`.

## L-claude-535-a-setup-wait-loop-under-set-e-exits-without-a-word-001
*category: validate · topic: e2e scenarios · from: pipeline 535*

`script/e2e.sh` runs under `set -euo pipefail`, and a scenario's `setup` runs in that shell. A
wait loop there such as `PORT=$(grep -oE 'port [0-9]+' out | cut -d' ' -f2)` ends the whole run,
exit 1 and nothing printed, the first time grep finds nothing yet: the pipeline fails and the
assignment carries its status. browser-fixture.sh's `serve_site` gets away with the same line
because it runs inside a command substitution, where `set -e` does not reach. In `setup` and
`steps`, end such a pipeline with `|| true` and test the value after.

## L-claude-574-an-item-a-private-module-shares-lives-in-a-public-module-001
*category: code · topic: clippy visibility lints · from: pipeline 574*

The Marley crates run rustc's `unreachable_pub` and clippy's `redundant_pub_crate` together, so
an item in a private module (`mod rail;`) that another module of the crate needs has no
visibility that passes: `pub(crate)` and `pub(super)` trip `redundant_pub_crate`, and `pub` trips
`unreachable_pub`. Put it in a public module or at the crate root, where `pub(crate)` passes
both; #574 moved the rail's `group_names` to `marley_workbench.rs`.

## L-claude-575-log-a-hooks-calls-in-order-to-find-a-race-001
*category: validate · topic: debugging a restore · from: pipeline 575*

When one of two alike items restores wrong, dump what was saved before the relaunch, then log
each hook call with its arguments during the relaunch. #575's two rows were both saved; the log
showed a cleanup with an empty list landing between the two reads, which no reading of the code
path of one item would have shown.

## L-claude-549-a-scenario-that-reaches-zeds-agent-panel-starts-the-profiles-last-agent-001
*category: validate · topic: e2e scenarios · from: pipeline 549*

`script/e2e.sh` copies the user's Marley profile, the Agent Panel's remembered agent included,
so a scenario step that makes Zed open a thread (Add to Agent Thread, a new thread from the
panel) starts that agent: in #549's first run, the user's own Claude agent created a session.
A step that must reach the Agent Panel sets a stand-in ACP agent in the run's settings first, as
#501's scenario does, or stays out of the scenario and is checked once by hand.

## L-claude-549-wtype-sends-a-shifted-binding-by-its-symbol-001
*category: validate · topic: e2e keys · from: pipeline 549*

Under the headless sway, `press "CTRL SHIFT" period` does not match Zed's `ctrl->`; `press CTRL
greater`, the symbol the binding names, does. Press a binding written with a shifted symbol by
that symbol's key name.

## L-claude-562-model-a-redaction-rule-change-in-python-before-the-build-001
*category: code · topic: redaction rules · from: pipeline 562*

A change to `redact.rs`'s rules can be checked in a second, before a build, with a scratch model
of `redact` and `apply` in Python that reads `BUILT_IN` from the file. On these patterns Python's
`re` and Rust's `regex` agree: leftmost-first alternation, no lookaround and no backreferences.
Feed it the lines the scenario will print, plus the lines of every scenario that reads through
the redactor. #562's model found the padding bug in F-claude-562, and the e2e run then matched
the model line for line. The e2e scenario stays the proof. The model only makes the first run
green more often.

## L-claude-562-a-block-read-joins-a-wrapped-line-before-redaction-001
*category: platform · topic: terminal reads · from: pipeline 562*

A block's text comes from alacritty's `main_bounds_to_string`, which appends no newline after a
row whose last cell carries `WRAPLINE`. A line longer than the terminal is wide therefore reaches
the redactor as one line, however it wraps on screen. #562's Digest header wrapped in the
headless sway and came back whole as `Authorization: Digest [redacted: authorization]`. The
redactor's rules can end values at `\n`, since a line that only wrapped holds none.

## L-claude-503-gpui-falls-back-to-the-desktop-portal-when-every-open-command-fails-001
*category: validate · topic: e2e fakes · from: pipeline 503*

gpui's Linux `open_url` runs `open::commands`, `xdg-open` first, and stops at the first that
exits 0. Only when every one fails does it ask the XDG desktop portal on the session bus, and
the portal opens the user's own browser, even from a headless sway. A scenario that opens URLs
puts a fake `xdg-open` first on the PATH Marley starts with (the runner launches Marley from its
own shell, after `setup`). It checks the fake with `command -v` before the launch and refuses to
launch if that is wrong, and the fake ends with `exit 0` whatever its logging does.

## L-claude-503-a-scenario-cannot-redefine-the-runners-helpers-001
*category: validate · topic: e2e runner · from: pipeline 503*

`script/e2e.sh` sources the scenario (line 81) before it defines `holds` and `expect`, so a
scenario's own `expect` is replaced by the runner's and the first failed check still ends the
run. A copy made to measure coordinates has to change its checks, for instance by wrapping each
in `|| true`, not the helper.

## L-claude-561-pythons-webbrowser-tries-every-browser-it-knows-after-a-failed-one-001
*category: validate · topic: e2e fakes · from: pipeline 561*

Python's `webbrowser` runs `BROWSER`'s entries first. When one exits non-zero, it tries every
browser it registered, in order: `xdg-open`, `gio`, then the installed browsers
(`google-chrome`, `firefox`, …). Under Omarchy the captured login environment also carries
`BROWSER=omarchy-launch-browser`. So a scenario that runs a program reading `BROWSER` does four
things:
- puts a fake `xdg-open` that always exits 0 first on the terminal's PATH;
- sets `BROWSER` to that fake in its `.bashrc` whenever the value under test is absent;
- fakes the browsers after `xdg-open` into a log;
- checks that the log stays empty.

## L-claude-579-a-program-waiting-on-read-proves-no-menu-opened-001
*category: validate · topic: e2e checks · from: pipeline 579*

A scenario can show that a click opened no menu by pressing the keys that would choose one of its
entries (Down, Return) and checking that nothing was chosen. With no menu, the keys go to the
terminal: a program waiting on `read` echoes the arrow as `^[[B` and ends on Return, and the log
that an entry would have written stays as it was. The same shot then shows the echo where a menu
would have been.

## L-claude-579-clippys-const-fn-and-dylints-nightly-disagree-001
*category: code · topic: the gate's toolchains · from: pipeline 579*

Clippy runs on the pinned stable toolchain and gate:21's dylint on its own older nightly
(`nightly-2026-03-21`), so they can disagree about what a `const fn` may call. Stable clippy asked
for `in_url` to be a `const fn`. On the nightly, `char::is_control` is not const, so gate:21
failed to compile `marley_terminal` while clippy was green. A `const fn` clippy asks for is
written with what both toolchains take: comparisons on `character as u32` instead of a
recently-const method. After a red gate, read the whole log: the summary line names the gate
but not the error.

## L-claude-504-cdp-has-no-favicon-event-so-read-the-icon-after-load-001
*category: code · topic: the browser · from: pipeline 504*

CDP has no favicon event: Electron's `page-favicon-updated`, which Orca uses, is not part of it.
So a page's icon is read after `Page.loadEventFired`. The page's `link[rel~="icon" i]` hrefs are
read in an isolated world, where the page's scripts cannot see the read, and the first http,
https or `data:image/` one wins (Orca's rule; `data:,` means none), else `/favicon.ico` at the
page's origin. `Network.loadNetworkResource` loads it for the page's frame, without credentials,
and the stream reader stops past a cap. A missing icon answers 404, which fails the load's
`success` before any bytes are read, and the row keeps the globe. A server's content type is often
wrong for icons, so the format comes from the first bytes.

## L-claude-504-a-click-that-passes-can-still-miss-its-target-001
*category: validate · topic: e2e checks · from: pipeline 504*

Twice in #504 a click passed its checks without landing where the scenario says. #504's first
run passed on guessed coordinates, since a pixel square and a close button a few pixels off still
hit. In the golden set, #574's click on repo-a's terminal row landed on repo-a's header once a
Browser row pushed the row down (L-claude-498). The header also shows the project, and the next
click focused the terminal, so all nine checks passed. A check proves the outcome, not the path.
Read the shots of every scenario that clicks near a changed layout, measure, and write the
measured values. To see what a click hit, take a shot straight after it, while the pointer still
hovers: the hovered row shows its hover state (a terminal row, its close button).

## L-claude-504-rustdoc-checks-what-clippy-and-dylint-do-not-001
*category: code · topic: the gate's toolchains · from: pipeline 504*

Gate:14 builds the scope's docs with `RUSTDOCFLAGS="-D warnings"`, and neither clippy nor dylint
reads intra-doc links. #504's new module doc failed it twice while both were clean: a public
module's doc linked a `pub(crate)` constant (`private_intra_doc_links`), and ``[`format`]`` was
ambiguous, since `format!` is a macro (`broken_intra_doc_links`; ``[`format()`]`` names the
function). A crate that fails stops the build, so the crates that depend on it go undocumented
until it is fixed, and their errors show only on the next run. Before the gate, run
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` with a `-p` for each of the scope's crates, after
clippy.

## L-claude-518-a-lone-surrogate-fails-the-whole-cdp-message-001
*category: code · topic: text read from a page over CDP · from: pipeline 518*

A page's strings are UTF-16, and a cut at a code-unit index can split a surrogate pair. A lone
surrogate that reaches a CDP answer arrives as a `\ud800` escape, which serde_json refuses, so
the parse of the whole message fails and the call that asked for it fails with it: one bad
character in a sibling's text would lose the pick. #518's `DESCRIBE` makes each text well formed
(`toWellFormed`) and steps each cut back from a lone surrogate before anything crosses the
socket. Any page function that cuts or builds strings for Marley needs the same.

## L-claude-580-a-drag-past-an-absolutely-placed-lines-end-collapses-the-selection-001
*category: testing · topic: e2e scenarios in headless sway · from: pipeline 580*

A drag in a Browser tab keeps the selection it made. #518 reported that it did not, having read
the page's event log through `browser_snapshot`, whose cut ("…") hid the moment the selection
collapsed. That moment came on a mouse move, before the release: the pointer passed the end of a
paragraph placed with `position: absolute`, over a body with no height, since all its children
were absolute. A point right of the text there lies over no text box, and Chromium's hit test
gives a position that collapses the selection. A drag that ends inside the text keeps it, and so
does a drag past the end of a paragraph in normal flow. Before calling an input bug, have the page
log each event and the selection at that moment to its console, read the whole sequence with
`mcp_agent console`, and find the event after which the result goes wrong. In a scenario, end a
drag inside the text or give the page a layout in flow. A glyph's columns in a shot come from an
ImageMagick threshold of the text's row, read as `txt:`.

## L-claude-505-a-shot-that-proves-something-gets-a-pixel-check-001
*category: testing · topic: e2e scenarios · from: pipeline 505*

#505's second run passed all 12 checks, which read the saved answers, while its
`505-03-nearest` showed a blank page with one crop in its corner. Only reading the shot caught
it. The fix added `page_drawn`: ImageMagick reads one pixel of the page area in the shot
(`-crop 1x1+x+y -format "%[fx:luminance]" info:`) and the check fails unless it is the page's
light background. It read 0.17 on the bad shot and 0.95 on the good ones, so it fails on the bug
it guards against. When a criterion rests on what a shot shows, turn that into a pixel check, and
test it on a bad shot first. Also move the pointer off a clicked button before the shot, or its
tooltip covers what the shot is for.

## L-claude-505-queryaxtree-finds-by-role-and-name-in-headless-chromium-001
*category: code · topic: CDP accessibility · from: pipeline 505*

`Accessibility.queryAXTree` (experimental) answers in headless Chromium 152 with no
`Accessibility.enable`. Called on the document's object (`Runtime.evaluate("document")` in the
main world) with `accessibleName` and `role`, it returns every matching node with its
`backendDOMNodeId`, none for a name the page lacks, and `DOM.resolveNode` turns each into an
element whose box can be read. A standalone probe proved it before #505 relied on it.

## L-claude-506-a-macro-shares-javascript-between-const-page-functions-001
*category: code · topic: page functions in Rust constants · from: pipeline 506*

Marley's page functions are `const &str`s, and `concat!` joins only literals and macro calls,
not other constants. A piece of JavaScript that several functions need (`interactive!`,
`generated_name!`, `css_path!`) is a `macro_rules!` that expands to a raw string literal,
spliced in with `concat!(..., generated_name!(), ...)`. `pub(crate) use {css_path,
generated_name, interactive};` after the macros lets another module (the recorder) import them
by path. Each piece defines its helpers as `const`s at the top of the function's body, so the
pieces a function takes must come before the code that calls them, and `node --check` on the
assembled text catches a piece in the wrong place.

## L-claude-506-prove-a-generator-by-running-what-it-makes-001
*category: testing · topic: e2e scenarios · from: pipeline 506*

A drafted Playwright test is proven by running it, not by reading it. #506's scenario writes
the draft where the tool says and runs it with the Playwright of a project on the box, three
times: with its variable set (it must pass), against a page whose step no longer works (it must
fail at that step), and without the variable (it must fail and name it). Grepping the draft
checked its locators. The passing run is what showed that Marley's role and name agree with
the names Playwright computes: a mismatch would have failed there and nowhere else. Playwright
resolves `@playwright/test` from a `node_modules` link beside the scratch repository, never
inside it, so Marley's project scan never walks the modules. Its output directory lives in the
repository, so nothing is written into the project that lends the modules.

## L-claude-507-a-window-restores-only-its-active-workspace-001
*category: gpui · topic: Zed's restore · from: pipeline 507*

At launch Zed's `restore_multiworkspace` opens only the window's active workspace; the window's
other project groups come back as keys, and the rail lists only groups with an open workspace.
Their items, Browser tabs among them, deserialize when the workspace opens: a click on its row,
or its path handed over. After a relaunch #507's scenario hands beta's path over again to see
beta's tab come back, and nothing in Marley may assume every project's tabs exist after a launch.

## L-claude-507-subscribe-self-calls-back-inside-the-entitys-update-001
*category: gpui · topic: entity re-entrancy · from: pipeline 507*

`Context::subscribe_self` runs its callback inside `this.update`, so the emitting entity is being
updated while the callback runs, and reading it there (a `read`, or `WindowHandle::read` of the
window's root) panics. #507's review of which projects' browsers run reads every window's
`MultiWorkspace`, so it is deferred (`cx.defer`) from the multi-workspace's events and from its
`on_release`.

## L-claude-507-the-rails-project-row-is-its-header-001
*category: testing · topic: e2e coordinates · from: pipeline 507*

In the 1600×1000 headless sway window the rail's first project header sits at y 95; the row at
y 136 is the project's first member, a terminal, whose right-click menu is Rename and Close.
#507's first run right-clicked y 136, taken from #574's constants, which name repo-b's terminal
row, and End then Enter closed the scratch terminal instead of removing the project. Take a
scenario's rail coordinates from its own first run's shot, and shoot a context menu before Enter
(L-claude-500).

## L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001
*category: rust · topic: the dylint gate · from: pipeline 507*

Zed's `async_block_without_await` lint, an error in the Marley crates (AD-claude-448), rejects
`cx.background_spawn(async move { blocking_io() })` and an `on_app_quit` callback that returns
`async {}`. Blocking work for the background executor goes in
`futures::future::lazy(move |_| blocking_io())`, as `open_browser` does, and a callback with
nothing to wait for returns `futures::future::ready(())`. `cargo clippy` does not load the lint;
only the gate's dylint stage (gate:21) does, so #507's Code phase passed clippy and its first
gate run failed on four such blocks.

## L-claude-581-a-prompt-opened-from-a-context-menu-keeps-the-keys-001
*category: gpui · topic: prompts and focus · from: pipeline 581*

A context menu entry's handler can open Zed's prompt (`Window::prompt`) directly.
`PromptHandle::with_view` focuses the prompt as it opens, and the right-click menu's dismissal
gives the focus back to what had it only while the menu still holds the focus, so Enter and
Escape reach the prompt; Enter picks the first button, Escape the one named "Cancel". Once
answered, the prompt returns the focus to what had it when it opened: the menu, which is gone,
so the window has no focused element. A scenario clicks into a pane before its next keys, as
#581's does. Work the answer starts that updates panes in their windows (closing tabs) runs from
the app, not inside an `AsyncWindowContext` update of the same window, which is out of the app's
map during its own update (L-claude-515).

## L-claude-523-chromium-sends-no-event-for-a-scripts-title-001
*category: browser · topic: page titles · from: pipeline 523*

Chromium sends no CDP event when a script sets `document.title`: with discovery on, a scratch
Chromium 152 reported the first title in `Target.targetInfoChanged` and nothing when a timer
changed it. Marley reads a title at DOMContentLoaded, at load and after a move within the
document, so a title set later is seen only when one of those reads comes after it. The debug
build usually read late enough; the release build, faster, read before the login site's
IndexedDB read set its title, and `just install`'s golden set failed #507's and #581's scenarios.
A scenario that shows a page's state through its title sets the title and then moves within its
document (`history.replaceState`), as the fixture's login site does; the product's gap is #582.

## L-claude-523-a-terminal-beside-a-tab-through-zeds-public-calls-001
*category: gpui · topic: workspace panes · from: pipeline 523*

`TerminalPanel::add_center_terminal` puts the terminal in the workspace's active pane, and a pane
becomes active only when its focus event is handled (`set_active_pane` is private), so a split
made just before does not receive it. To open a terminal beside a tab without copying Zed's body
into a Marley crate: focus the tab (its pane is active by the time the terminal is made), call
`add_center_terminal`, then move the terminal from the tab's pane with `move_active_item` into
the pane `find_pane_in_direction(Right)` finds, or with `split_and_move` when there is none.
Later runs then reuse the right-hand pane.

## L-claude-523-an-npm-install-in-a-scenario-uses-a-loopback-registry-001
*category: testing · topic: e2e scenarios · from: pipeline 523*

A scenario runs a real `npm install` with no network through a loopback registry. It packs a
copy of the package already on the box (`npm pack <dir> --pack-destination`), writes the
package's document (`name`, `dist-tags`, `versions.<v>` with `dist.tarball`,
`dist.integrity` as `sha512-<base64>` and `dist.shasum`) as `<name>/index.html` under the
fixture's `serve_site`, and points the terminal at it (`npm_config_registry`, `npm_config_cache`
in the scratch HOME, `npm_config_update_notifier=false`, which also keeps `npm` from asking the
public registry for its own version). npm then asks for `/<name>`, follows the redirect to
`/<name>/` and fetches the tarball. #523's scenario installs `playwright-core` 1.63.0 this way
in under 200 ms.

## L-claude-582-cargo-piped-into-head-can-end-mid-build-001
*category: process · topic: cargo on the shared dev box · from: pipeline 582*

A cargo command piped into `head` (or any reader that stops early) can die partway: once `head`
has its lines it exits, and cargo's next write to the closed pipe ends it with SIGPIPE, which is
a killed cargo by another name, the thing the box's rules forbid. In #582 a `cargo dylint | grep |
head -20` stopped reading after twenty warnings; nothing was left running, and the rerun finished
from its cache. Cargo's output goes to a log file in the scratchpad, and `grep` or `tail` reads
the file afterwards.

## L-claude-524-a-restarted-mcp-server-listens-on-a-new-port-001
*category: testing · topic: tokens across a restart in an e2e scenario · from: pipeline 524*

Marley's MCP server binds `127.0.0.1:0`, so a restart puts it on a new port, and an endpoint file
kept from the run before names a port nothing listens on: a request with it is refused by the
kernel before any token is read. A scenario that shows a stale token refused sends it to the new
server: a copy of the new endpoint file with the old file's `headers` (`jq -s '.[0] * {headers:
.[1].headers}' new old`).

## L-claude-524-the-default-snapshot-holds-no-text-001
*category: testing · topic: reading a page's text through the agent tools · from: pipeline 524*

`browser_snapshot` without `full` lists the page's interactive elements with their refs (a
textbox, a button) and none of its text, so a check that a line on the page changed finds
nothing. The fixture's `mcp_agent snapshot full` returns the text as well.

## L-claude-583-one-pipe-many-clients-through-browser-sessions-001
*category: code · topic: CDP over Chromium's debugging pipe · from: pipeline 583*

Chromium's `--remote-debugging-pipe` is one connection, where its port gave each client a
connection of its own with its own domain state. `Target.attachToBrowserTarget` gives each client
a browser session instead, and a probe over a pipe (Chromium 152) showed them kept apart:
discovery and auto-attach on one session announce to that session alone, answers and events
carry their flat session id, a browser session detaches, and `Browser.close` from one closes the
browser. A relay that rewrites ids, routes by session and owns each session to one client lets
Marley, a Playwright script and an agent share the pipe as they shared the port.

## L-claude-583-grep-counting-none-ends-a-step-under-errexit-001
*category: testing · topic: e2e scenarios · from: pipeline 583*

`grep -c` exits 1 when it counts no line, and a scenario step runs under errexit, so
`count=$(… | grep -c pattern)` ends the step on the very count a check wants to see (a process
listening on no port), with nothing in the log after the step's heading. Write
`count=$(… | grep -c pattern || true)`, and wrap a filtering `grep` in a pipeline the same way.

## L-claude-583-a-program-marley-starts-by-path-runs-through-a-fixed-shell-exec-001
*category: code · topic: starting a program whose path is configured · from: pipeline 583*

gate:20's semgrep rule `command-injection-risk` refuses `std::process::Command::new` of any
program that is not a literal, and Marley's relay had to start the Chromium that
`MARLEY_CHROMIUM` or the known paths name. The relay runs the fixed `/bin/sh` with the literal
script `exec "$0" "$@"` and the program and its arguments as positional parameters, each one
word: nothing in them is read as a name or as shell, and the exec keeps fds a caller mapped
(`command-fds`). `env` would read a path holding `=` as a variable.

## L-claude-584-a-scenario-runs-an-sshd-of-its-own-001
*category: testing · topic: e2e scenarios over SSH · from: pipeline 584*

An unprivileged sshd runs on the dev box for a scenario: `/usr/bin/sshd -D -e -f <config>`
with `ListenAddress 127.0.0.1`, a free port, its own `HostKey`, `AuthorizedKeysFile` and
`PidFile` in the scenario's folder, `UsePAM no` and `StrictModes no`. The client takes
`ssh -F /dev/null -i <key> -o IdentitiesOnly=yes -o IdentityAgent=none -o
UserKnownHostsFile=<file> -o StrictHostKeyChecking=accept-new -o BatchMode=yes`, so neither
side reads or writes `~/.ssh` or asks the keyring's agent. A remote command runs through the
account's login shell, as a login from another machine does; the account's shell start-up
printed nothing into it.

## L-claude-584-a-tabs-toolbar-moves-with-the-last-actions-text-001
*category: testing · topic: e2e click targets · from: pipeline 584*

The Browser tab's toolbar ends with the last action's text ("went to
http://127.0.0.1:<port>/…"), so "Driven by <name>" and its Cut Off sit left of it by an amount
that changes with the URL, the port included: #524's point (1330, 87) landed on that text in
#584's run. A scenario cuts a client off with the client's row in Browser Clients, whose place
depends only on how many clients are listed.

## L-claude-539-chromium-152s-hints-name-no-headless-brand-001
*category: code · topic: the browser's identity over CDP · from: pipeline 539*

Probed on Chromium 152 before the design was locked:
- Only the user agent string says `HeadlessChrome/`. `Sec-CH-UA` and the full version list are
  Chromium's own (`"Not?A_Brand";v="24", "Chromium";v="152"`), with no headless brand; a 2024
  capture of Chrome 126 showed one.
- `about:blank` made by `Target.createTarget` is not a secure context and has no
  `navigator.userAgentData`, so it cannot report the hints.
- `--user-agent` fixes the string everywhere but empties the full version list.
- `Emulation.setUserAgentOverride` with only the metadata fields CDP requires (and `bitness`,
  `wow64`), leaving out `brands`, `fullVersionList` and `fullVersion`, keeps every hint as
  Chromium gives it.

So an override needs the string and the machine's platform, architecture and bitness, and no
table of brands.

## L-claude-539-a-page-created-at-about-blank-keeps-its-blank-entry-001
*category: code · topic: page history over CDP · from: pipeline 539*

A page `Target.createTarget` opens at `about:blank` keeps that entry once it navigates:
`Page.getNavigationHistory` answers `["about:blank", url]`, where a page created at its URL
answers `[url]`. `Page.resetNavigationHistory` after the URL commits leaves `[url]`. A tab opened
at a URL through a blank start would otherwise gain a Back that leads to a blank page.

## L-claude-521-gpuis-mutable-global-access-tells-every-observer-001
*category: gpui · topic: globals and observers · from: pipeline 521*

`App::global_mut`, `default_global` and `set_global` each push
`Effect::NotifyGlobalObservers`, whether or not the caller changes anything; `try_global` and
`global` push nothing (`crates/gpui/src/app.rs`). A task that polls into a global that views
observe (`observe_global`, `observe_global_in`) redraws them every round unless it reads through
`try_global` and writes only when the value changed. #521's port scan runs every three seconds
and writes `Ports` only when the listeners differ, so the rail rebuilds on a change, not on
every round.

## L-claude-521-two-loops-read-the-tcp-tables-001
*category: e2e · topic: measuring what Marley reads · from: pipeline 521*

Two loops read `/proc/net/tcp` and `tcp6` whole: #521's port scan, every three seconds while a
rail shows, and #503's offer of a printed URL (`links.rs`, `PORTS_POLL`), every two seconds while
any terminal holds one. Both run on the background executor's `Worker` threads, so a count of
Marley's reads (`rchar` in `/proc/<pid>/io`) cannot tell them apart. On the dev box one read of
both tables is about 66 KB. A scenario that measures one loop runs before any terminal prints a
URL. Per-thread counts (`/proc/<pid>/task/*/io` with each thread's `comm`) show which threads
read, which is how #521's Test phase found #503's share.

## L-claude-565-jevs-answer-shape-as-recorded-001
*category: code · topic: the System One API · from: pipeline 565*

Read from Chad's own recorded Jev runs (922 answers, 574 from `jev-1.13.0`) and his working
client, since no key was at hand for a live request:
- a noul answers `{type: "noul", noul}` alone: no `confidence`, no `probabilities`;
- a choice answers `choice`, `confidence` and `probabilities` by option;
- a score answers a fractional `score` (5,516 of 5,740 were fractional: the expected level, not a
  level), `confidence`, `probabilities` keyed by the level as a string, and a `legend` that maps
  each level to `{summary, signals}`, where one client's types say a string;
- a score's question sends its levels as `criteria`, an array; a noul's `criteria` is `{true,
  false}`, a choice's a map of option to meaning;
- the working client retries 429, 503 and 529 and cuts an error body to 300 characters.
So a reader keeps `legend` as JSON, reads a score's number as fractional, and ignores fields it
does not know.

## L-claude-565-a-poll-in-setup-dies-under-set-e-001
*category: e2e · topic: writing a scenario's setup · from: pipeline 565*

`script/e2e.sh` runs `setup` in its own shell under `set -e` and `pipefail`, so `port=$(grep -oE
'port [0-9]+' out | cut …)` in a wait loop ends the whole run at the first pass, before the
server has printed its port: `grep` finds nothing, exits 1, and the assignment fails. The fixture's
`serve_site` uses the same pipeline and survives only because it runs inside `$(serve_site …)`,
where `set -e` does not reach. In a loop that waits for a line, use a command that succeeds with
no match, such as `sed -n 's/^port \([0-9]*\)$/\1/p'`.

## L-claude-565-a-scenario-reaches-the-users-own-keyring-001
*category: e2e · topic: what the headless sway shares with the user's session · from: pipeline 565*

The headless sway `script/e2e.sh` starts keeps the user's session bus, so a scenario's Marley
reaches the user's own Secret Service (gnome-keyring on the dev box). gpui's `write_credentials`,
`read_credentials` and `delete_credentials` would write and read the user's real items, and an
`unlock()` can prompt on the user's screen. A scenario never presses a control that writes or
reads the keyring; it passes a secret through the environment, as #565's does with
`MARLEY_SYSTEM_ONE_KEY`, and the code reads the keyring only when a feature is on and nothing
else holds the key.

## L-claude-566-the-folds-state-rides-on-the-seat-001
*category: code · topic: `marley_agent::claude_events` · from: pipeline 566*

`fold` builds its `Moving` from the seat at every event and keeps nothing between events, so
anything the fold must remember across a turn is a label on the seat: #566's `TurnFacts` is the
`turn` label, in JSON, parsed at the start of `take` and written back in `into_events`. Labels are
published whole to agents through `fleet_snapshot` (#547), so a label is also what agents see: the
stop kind lands no label in `shadow`, which would have shown agents what the row does not.

## L-claude-566-a-use-whose-questions-vary-needs-a-set-per-shape-001
*category: code · topic: the System One layer · from: pipeline 566*

#565's question sets are compiled in, and a caller never supplies its own questions. A use whose
questions depend on its input, such as a noul for each part of a prompt, needs one set for each
shape, each with its own id (`stop_kind_0/1` to `stop_kind_6/1`), and puts the varying text in the
state as labeled lines (`part 1: …`), where it is masked and cut like any text. The sets and uses
are `static`, not `const`: a `const` array indexed at run time gives a reference to a temporary,
and `ask` takes a `&'static UseSpec`. A replay row names the set of its shape.

## L-claude-566-the-hooks-reference-leaves-a-failing-bash-open-001
*category: process · topic: Claude Code's hooks · from: pipeline 566*

Claude Code's hooks reference (code.claude.com/docs/en/hooks, read 2026-09-27) says `PostToolUse`
fires "after a tool call succeeds" and `PostToolUseFailure` "after a tool call fails", and says
nothing on a Bash whose command exits non-zero; it gives neither event's `tool_response` for Bash.
`PermissionDenied` fires only when auto mode denies a call, so a refusal in the dialog sends no
event of its own. Every input carries `prompt_id`, and `Stop` carries `last_assistant_message`. A
design that counts a Bash as a check must hold either way: #566 counts a Bash that ended by
`PostToolUse`, and asks the model to find the check in the message too.

## L-claude-566-an-edit-script-fed-through-the-heredoc-it-edits-ends-early-001
*category: tooling · topic: editing scenarios from the shell · from: pipeline 566*

A scenario's helpers embed Python in `<<'PY'` heredocs. A Python edit script fed to `python3 -`
through `<<'PY'` that holds such a block ends at the block's own `PY` line: Python gets half a
script, fails, and the shell runs the rest of the edit's text as commands. Write an edit script
that holds a heredoc to a file in the scratchpad and run the file, or give the outer heredoc a
delimiter the text does not hold.

## L-claude-567-a-choice-over-run-time-items-is-a-static-table-by-count-001
*category: code · topic: the System One layer · from: pipeline 567*

A choice whose options are items known only at run time, such as a page's refs, cannot be built
per call: #565's questions are `&'static`, and a caller never supplies its own. Make a set for
every item count, as #567's `find_1/1` to `find_254/1` are: one options table with `none`
first, so each count's options are a prefix of it; the names, meanings, questions and sets in
`static LazyLock` tables built once, whose `String`s a static holds, so a `&'static str` comes
from them with nothing leaked. Put the query and the items in the state as labeled lines whose
labels are the option names, and take a `UseSpec` by value, since it is `Copy`, rather than a
static table of every use and count.

## L-claude-567-a-tool-a-setting-lists-must-reach-the-servers-data-001
*category: code · topic: Marley's MCP server · from: pipeline 567*

`marley_mcp`'s listing and routing are pure over `RequestCtx`, which `serve_post` builds under
the server's data lock, so a setting that lists or hides a tool has to become server data
(`ServerData.enabled`) before any request sees it. The app's settings observers cannot take that
lock on the main thread, which the server's threads hold: send the value down a channel to a
background task that sets it, as the fleet snapshot goes (`publish`, `enabler`). The server sends
no `notifications/tools/list_changed` on such a change, and Claude Code lists its tools when it
connects, so a running session sees a tool turned on only after it reconnects the server.

## L-claude-569-a-turns-tools-are-the-descendants-born-in-it-001
*category: code · topic: reading Claude Code's processes · from: pipeline 569*

`Terminal::pid()` is the PTY's foreground process group, which for an interactive `claude` is its
own pid. That process burns CPU while it waits (its spinner), and its tree holds long-lived
children from before the turn (MCP servers, a plugin's language server), so neither its CPU nor
its tree's says a tool is at work. The turn's tools are its descendants started after the turn's
prompt: walk `/proc/<pid>/task/<tid>/children`, keep those whose `stat` `starttime` is at or after
the prompt in boot ticks, and sum their `utime` and `stime` (`procfs-core`'s `Stat` parses the
file). A descendant can end mid-walk, so skip what fails to read, and read it all off the main
thread.

## L-claude-569-proc-btime-is-whole-seconds-001
*category: code · topic: `/proc` times · from: pipeline 569*

`/proc/<pid>/stat`'s `starttime` counts clock ticks (`USER_HZ`, 100) from the boot, and the boot's
wall time is `/proc/stat`'s `btime`, in whole seconds. A wall-clock moment turned into boot ticks
through `btime` can read up to a second late, so a process started in that second reads as older
than the moment. Compare with a second's slack, taken off the moment.

## L-claude-569-a-stand-in-proves-a-cpu-rule-with-a-bounded-busy-child-001
*category: process · topic: e2e scenarios · from: pipeline 569*

A scenario proves a rule about CPU by having the stand-in agent start a child that burns a core
(`python3 -c` over a busy loop), and end it later. Bound the loop (90 seconds) and keep its pid for
the teardown: the stand-in dies with Marley's terminal at the end of a run, and an unbounded child
would outlive it and burn a core on the box. Start it a second or two after the prompt's event, so
it is plainly the turn's.

## L-claude-571-a-forms-named-controls-shadow-it-even-in-an-isolated-world-001
*category: code · topic: reading a page over CDP · from: pipeline 571*

An isolated world (`Page.createIsolatedWorld`) keeps a page's scripts from changing the
prototypes Marley's function sees, but not the DOM's named properties: a form's control named
`action` is what `form.action` gives, and an `<img name="body">` is what `document.body` gives.
A fixed function that reads a form or the document goes through the DOM's own methods and
descriptors (`Element.prototype.getAttribute.call`, `Object.getOwnPropertyDescriptor(
Document.prototype, 'body').get.call(document)`). `DOM.resolveNode` with the world's
`executionContextId` fails for a node in another frame, so fall back to the node's own world.

## L-claude-571-unreachable-pub-and-redundant-pub-crate-meet-at-a-re-export-001
*category: tooling · topic: the Marley lint table · from: pipeline 571*

In a private module, rustc's `unreachable_pub` asks for `pub(crate)` on an item another module
uses, and clippy's `redundant_pub_crate` asks for `pub`, both as errors under `-D warnings`. The
way out is the one `marley_mcp`'s other modules take: `pub` in the module, re-exported from the
crate root (`pub use session::{…}`).

## L-claude-571-a-scenario-reads-a-pages-state-from-its-title-001
*category: process · topic: e2e scenarios · from: pipeline 571*

The accessibility snapshot splits a paragraph's inline text into several nodes, so a page's own
running log does not come back as one line an agent can grep. A test page that writes its state
into `document.title` gives it whole through `browser_tabs` (#582 keeps the title current), and
the rail's row and the tab show it in every shot too.

## L-claude-508-the-agent-chip-moves-the-browser-toolbar-001
*category: process · topic: e2e scenarios · from: pipeline 508*

For a few seconds after an agent's action, the Browser tab's toolbar carries the Agent chip
("went to http://…"), which narrows the address field and moves every toolbar button left. A
scenario that clicks a toolbar button by its point runs no agent action in the seconds before, or
settles until the chip has gone; points measured in a shot without the chip hold only then.

## L-claude-508-a-stand-in-acp-agent-asks-for-permission-001
*category: process · topic: e2e scenarios · from: pipeline 508*

A scenario reaches an Agent Panel permission prompt with no model. A stand-in agent server in
Python, a `custom` entry in the run's `agent_servers`, answers `initialize` and `session/new`;
on each `session/prompt` it sends a `tool_call` update, then its own `session/request_permission`
with `allow_once` and `reject_once` options, and reads stdin until the response with that id
comes. It logs `outcome.optionId`, sends the `tool_call_update` and ends the turn. The panel draws
the prompt as it draws a real agent's, so the panel's buttons and Marley's can both be driven, and
the answer read from the log. `script/e2e/508-approvals-inbox.sh` has it.

## L-claude-568-unused-results-wants-every-returned-value-used-001
*category: code · topic: the Marley lint table · from: pipeline 568*

Marley's lint table denies `unused_results`, so a call whose value is dropped fails clippy even
where rustc says nothing: `BTreeSet::insert` and `PathBuf::pop` return a `bool`, and
`Peekable::next` an `Option`. Collect into a `Vec` with `push`, then `sort` and `dedup`; test
`next_if_eq(..).is_some()` inside the condition that needs it; walk up with `parent()` rather
than `pop()`. A float that becomes a small integer, such as a score's expected level, can come
from a comparison ladder (`(1..=5).rev().find(|level| score >= f64::from(*level) - 0.5)`) instead
of an `as` cast, which the pedantic table flags and §0 forbids allowing.

## L-claude-568-a-seat-holds-one-wait-so-each-entry-needs-its-own-terminal-001
*category: process · topic: e2e scenarios · from: pipeline 568*

A Claude Code seat holds one question at a time, keyed by its terminal, so a scenario that needs
several terminal entries in the inbox runs the stand-in `claude` in a terminal each, taking its
case from its first argument. `palette "workspace: new terminal"` opens each in the center with
the focus, from anywhere and with no point to measure, where the rail's `+` moves as the inbox
grows above it.

## L-claude-570-an-outcome-read-from-the-focus-needs-its-wait-to-start-unwatched-001
*category: process · topic: e2e scenarios · from: pipeline 570*

#570 counts a wait as the owner's when its terminal held the focus at any refresh while it
waited. A scenario opens a terminal and types into it, so a stand-in whose request comes at its
first Enter is watched from its first moment and can never read `agent`. To prove `agent`, the
stand-in sends its request after a delay, once the scenario has moved the focus on, and ends the
wait itself later with no input (a step marked `auto` with an `after`, and a `sleep` event),
while nothing brings its terminal forward.

## L-claude-570-a-choice-that-abstains-reads-as-no-signal-001
*category: code · topic: the System One layer · from: pipeline 570*

#565's reader turns a choice of `cannot_tell` (or `none`) into `Signal::Nothing`, as it does a
confidence under the floor, so a use never sees the abstaining option by name. A use that must
show its own floor for those, as the question route's `unclear` does, maps a `Nothing` on its
key, and a `Refused` or `Unavailable` reading, to that floor itself, and keeps no confidence to
show for it.

## L-claude-509-a-zed-checkpoint-is-a-commit-on-head-of-the-whole-tree-001
*category: code · topic: git · from: pipeline 509*

`Repository::checkpoint` runs, in a temporary index copied from the user's, `add --update`, adds
the untracked files `checkpoint.gitignore` lets through (no ignored files, nothing of 2 MB or
more, no binaries, archives or media), `write-tree`, and `commit-tree <tree> -p HEAD -m
Checkpoint` as Zed; the user's index, branches and `HEAD` stay as they were. So two checkpoints
compare the working trees, staged and unstaged alike, and a commit of one's tree on the other,
which `CommitView` diffs against its first parent, shows exactly what changed between them.
`Repository` has no job to write a commit from a tree or to list refs; those take a `git` spawn.

## L-claude-509-a-scenario-agent-edits-after-its-prompts-checkpoint-001
*category: process · topic: e2e scenarios · from: pipeline 509*

Per-turn diffs take the turn's start checkpoint when the prompt's frame reaches Marley. A
stand-in agent that edits a file at once can beat that checkpoint, and its edit lands in the
turn before. #509's stand-in sleeps two seconds after each prompt's event before its edits, as a
model's first reply would, and a second before its stop.

## L-claude-532-the-plus-lists-marleys-path-and-types-into-the-terminals-001
*category: process · topic: e2e scenarios · from: pipeline 532*

The rail's `+` lists the agent CLIs `which` finds on Marley's own PATH, which under the e2e
runner is the login shell's (Marley's stdout goes to a log), so its entries show wherever the
real CLIs are installed. What it types runs on the terminal's PATH, from the scenario's
`.bashrc`. A scenario that launches from the `+` puts its fakes first there and checks, before
the first launch, that the terminal's `command -v` names the fakes, writing it to a file and
stopping the run if not. The menu opens on New Terminal; Down steps to New Browser Tab, New
Agent Thread and then the CLIs in `AgentKind::ALL`'s order, skipping the header.

## L-claude-532-claude-codes-arguments-say-how-it-started-not-its-mode-001
*category: code · topic: agent CLIs · from: pipeline 532*

Claude Code 2.1.283 can enter bypass without `--dangerously-skip-permissions` in its arguments
(its settings' default mode, or Shift+Tab after `--allow-dangerously-skip-permissions`) and can
leave it with Shift+Tab after starting with it. Its arguments say how it started; the
`permission_mode` every hook event carries says what it does now, as of its latest event. Read
the reported mode first, and the arguments only for a session that sends no events.

## L-claude-510-a-scenario-clicks-a-context-menus-submenu-entry-001
*category: process · topic: e2e scenarios · from: pipeline 510*

In the headless e2e runs, a submenu of the rail's `+` (Zed's `ContextMenu`) does not follow the
keyboard: Right opened nothing, Enter opened the submenu without moving the keys into it, so a
second Enter confirmed the parent again, and the submenu drew only after the pointer came over
its entry. A scenario reaches a submenu's entry by Down steps to the parent, Enter, then a click
on the entry's measured point, and hovers the parent entry before a shot of the open submenu.

## L-claude-510-zeds-worktree-create-names-places-and-rolls-back-001
*category: code · topic: git worktrees · from: pipeline 510*

Zed's worktree service puts a worktree at `<parent>/worktrees/<project>/<name>/<project>` with the
default `git.worktree_directory`, names it `<name>` (`linked_worktree_short_name`), checks a given
name only against the registered worktree paths, and rolls a refused create back with
`remove_worktree(path, true)`, which deletes the target folder before git runs. A caller that
names its worktree keeps the name off existing folders and branches itself. Its create shows its
own toast for every failure after three early refusals (no repository, a collab project, a create
in flight), so a caller reports only its own steps. A linked worktree's repository lists the main
checkout (`is_main`) but not itself.

## L-claude-560-zed-turns-a-repositorys-trust-on-without-an-event-001
*category: code · topic: git trust · from: pipeline 560*

When the user trusts a folder, Zed's `GitStore` sets the trust on the repository's backend in a
background task (`on_trusted_worktrees_event`, then `backend.set_trusted`) and emits no
`RepositoryEvent`, so `Repository::is_trusted` turns true with nothing for a subscriber to hear.
A reader that checks the trust at its own rebuild waits for the next one. A scenario that needs
work gated on the trust makes a rebuild after the trust press, such as a command in a terminal.

## L-claude-560-merge-tree-reports-conflicts-with-exit-one-001
*category: code · topic: git · from: pipeline 560*

`git merge-tree --write-tree --name-only -z --no-messages` exits 0 for a clean merge and 1 for one
that would stop, printing the merged tree's id and then each conflicted file, every entry ended by
a NUL. A git before 2.38 refuses `--write-tree` with exit 129 and `unknown option`. `git config
--get` exits 1 for an unset key. Zed's `run_raw` takes any non-zero exit for an error, so a caller
that needs these answers runs git itself and reads the exit.

## L-claude-587-claude-code-keys-its-trust-on-the-main-checkout-001
*category: code · topic: Claude Code's folder trust · from: pipeline 587*

Claude Code keys its folder trust on the repository's root, and on the main checkout's root for
a worktree (code.claude.com permissions, "Project allow rules and workspace trust"; issue #23109,
2026-08-17). So a worktree of a repository already trusted in Claude Code starts without the
question, and #510's note that it asks in every new worktree no longer holds. The question shows
in interactive sessions only, and hooks, the plugin's included as far as its docs say, wait for
its answer, so nothing but the screen shows it. Since 2.1.263 its focus starts on "No, exit",
where Enter declines and quits. An answer moves the focus to the trust option by name.

## L-claude-587-a-zed-toast-is-a-notification-under-its-id-001
*category: code · topic: Zed notifications · from: pipeline 587*

`Workspace::show_toast` shows a `MessageNotification` under the toast's `NotificationId`, and
`dismiss_app_notification` or `dismiss_notification` with that id dismisses it. A button of
`MessageNotification` dismisses its own notification after its handler runs, and the dismissal
is deferred. So a notice shown from a handler under the clicked notification's id goes at once.
Give each notice its own id.

## L-claude-587-a-sed-template-needs-g-for-a-placeholder-twice-on-a-line-001
*category: process · topic: e2e scenarios · from: pipeline 587*

A scenario that writes a fake from a template with `sed -e "s|@NAME@|value|"` replaces only the
first `@NAME@` of each line. A Python line such as `open("@NAME@") if os.path.exists("@NAME@")`
then checks a file literally named `@NAME@`, and the fake acts as if its file were missing. Give
each substitution `g`, and have the fake log what it read.

## L-claude-446-zeds-license-check-wants-a-symlink-in-every-crate-001
*category: process · topic: licensing · from: pipeline 446*

`script/check-licenses`, which Zed's CI runs, walks every `Cargo.toml` that git tracks and wants
its folder to hold a `LICENSE-GPL` or `LICENSE-APACHE` symlink to the root file, at `../` per
level. A regular file of either name is an error, and so are a `license-file` key and any AGPL. It
stops at the first error and never reads a `LICENSE-MIT`. A new Marley crate gets
`LICENSE-APACHE -> ../../LICENSE-APACHE` and a copy of a Marley crate's `LICENSE-MIT`. A crate
vendored under `vendor/` is skipped, and keeps its own license file.

## L-claude-586-cargo-doc-open-hands-browser-a-path-001
*category: code · topic: the BROWSER opener · from: pipeline 586*

`cargo doc --open` runs `$BROWSER` with one argument, the absolute path of the doc index
(`…/target/doc/<crate>/index.html`), not a `file://` URL. Rust tools on the `opener` crate pass
their argument unchanged too. Python's `webbrowser.open` passes whatever it was given, a `file://`
URL in the common case. An opener that takes local pages reads both forms; a relative path joins
the program's folder.

## L-claude-511-a-fresh-folder-workspace-gets-its-first-terminal-after-its-openers-item-001
*category: code · topic: the Marley layout's first terminal · from: pipeline 511*

In the Marley layout, routing's `seed_first_terminal` gives every fresh folder workspace a center
terminal, created asynchronously after the workspace's new-workspace observer runs. Code that opens
a workspace and then adds its own item there (a diff, a page) races it, and the terminal usually
lands last and takes the front. Such code marks the folder first with
`worktree_agents::skip_seed` (the seed compares a root's last two parts) and drops the mark with
`drop_seed_skip` once its item is in, as #510 does for an agent's worktree.

## L-claude-511-git-titles-a-merge-by-the-name-it-was-given-001
*category: code · topic: git merge · from: pipeline 511*

`git merge --no-edit refs/heads/<b>` titles the commit "Merge branch 'refs/heads/<b>'": git's
default title quotes the argument as given. Naming the full ref is still right, since a tag of
the same name outranks the branch in git's ref lookup (gitrevisions(7)), so Marley passes
`-m "Merge branch '<b>' into <base>"` with it. git 2.55 leaves " into main" off its own default
title for main; the explicit title keeps it.

## L-claude-585-a-new-worktree-gets-its-bases-committed-tasks-001
*category: code · topic: worktree agents · from: pipeline 585*

A worktree Zed's service makes checks out its base commit, so its `.zed/tasks.json`, and the
`create_worktree` hooks Zed runs in it, are the base's committed copy, not the main checkout's
working copy and not the tasks of whatever workspace started the create. Code that must know what
the new worktree will run reads `git cat-file blob <base>:.zed/tasks.json` and parses it with
`settings::parse_json_with_comments::<task::TaskTemplates>`; the user's global tasks are the
inventory's `TaskSourceKind::AbsPath` entries.

## L-claude-585-the-rail-lists-worktrees-in-gits-order-001
*category: e2e · topic: scenarios with worktree rows · from: pipeline 585*

The rail lists a project's worktree rows in the order git lists the worktrees, by folder path, and
Zed's worktree service names each worktree at random, so a row's place moves from run to run. A
scenario that clicks a worktree's row, or an agent's row under it, works out its place from the
worktrees' folders (sorted as git sorts them, `LC_ALL=C sort`), each open worktree taking its row
and its agent's row, instead of a place measured once.

## L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001
*category: e2e · topic: stand-in programs for Marley itself · from: #531's visual check*

At start Marley loads the user's login shell environment and uses its PATH, so a stand-in the
scenario puts first on its own PATH can lose to the user's real program: #531's first run reached
the real `gh`, which asked GitHub about a made-up repository. A program Marley itself runs (not
one a terminal's shell runs, whose PATH the scenario's `.bashrc` sets) gets an override variable
Marley reads, `MARLEY_GIT`, `MARLEY_CLAUDE`, `MARLEY_GH`, and the scenario names its stand-in there.

## L-claude-554-a-right-click-in-zeds-terminal-queues-a-word-selection-001
*category: gpui · topic: Zed's terminal menu and selection events · from: #554's Code phase*

A right-click in Zed's terminal with no text selected calls `select_word_at_event_position`, which
only queues `InternalEvent::SetSelection`; the terminal applies it at its next `sync` and emits
`SelectionsChanged` then, after `deploy_context_menu` has built the menu. Anything the menu's hooks
set that a `SelectionsChanged` listener would undo is undone a frame later. State that must survive
the right-click is judged by something the click does not touch, such as a count of inputs.

## L-claude-555-the-rails-rows-move-when-needs-you-shows-001
*category: e2e · topic: clicking the rail in a scenario · from: #555's visual check*

The rail's Needs you section (#508) appears above the projects as soon as a seat waits, and every
project and terminal row below it moves down by the section's height. A scenario that clicks rail
rows by fixed coordinates clicks something else after a seat starts waiting. Bring a terminal
forward by its tab (`alt-1` to `alt-9`, Zed's `pane::ActivateItem`), or measure the row again
after the step that makes a seat wait.

## L-claude-558-a-scenarios-server-takes-a-port-picked-at-run-time-001
*category: e2e · topic: servers a scenario starts · from: #558's visual check*

The headless sway shares the box's network, and the box runs services on ports a scenario might
pick by hand: #558's first run typed 8124, which a service already held on `*:8124`, so the
workflow's server failed with `Address already in use` and the check read it as Marley's fault.
A scenario that starts a server picks its port at run time (bind port 0 on 127.0.0.1 and read the
port back) and types that. A check that a cancelled rerun started nothing compares the listener's
`ss -ltnpH` line, pid included, before and after: Zed's rerun replaces a task's terminal, so a
count of listeners still reads 1 when the rerun did run.

## L-claude-559-a-per-block-hook-has-one-slot-so-compose-it-001
*category: gpui · topic: Marley's hooks in Zed's terminal element · from: #559's design*

`MarleyBlockChip` and `MarleyBlockExtras` are single globals: the element asks each once per
block, and `cx.set_global` keeps only the last value set. A feature that adds a chip or a hover
button by setting the hook again silently removes the one set before it; #559's bookmark would
have dropped #555's Ask the agent chip and #558's Save as Workflow button. One module sets each
hook (`bookmarks::init`) and composes it from the other modules' `pub(crate)` parts; a new chip
or button is one more part in that composition, never a second `set_global`.

## L-claude-529-an-element-over-the-terminal-stops-the-release-too-001
*category: gpui · topic: Marley's elements over Zed's terminal grid · from: #529's review*

An element drawn over the terminal's rows that stops a left press still lets the release through
to the terminal, and since #579 the terminal's own release listener opens the link menu for a
plain click on a URL under the pointer. Stopping the press with `on_mouse_down` and acting in
`on_click` on the same element leaves the release unstopped. Act on the press, as Zed's editor
sticky headers do, and stop the release with an `on_mouse_up` of its own; or wrap the button in
`marley_keep_from_terminal` and check the release there.

## L-claude-530-a-nothing-typed-check-reads-the-prompt-line-001
*category: e2e · topic: checking a terminal's screen after a refused write · from: #530's visual check*

A terminal's screen keeps every earlier step's lines, including text a step pasted and cancelled
with Ctrl-C. A check that nothing was typed which greps the whole screen for the text matches an
earlier step's copy and fails on a correct run. Read the prompt line only: the screen's last line,
as `tail -1 file | grep -qE '^  \| \$ *$'` does with `mcp_agent terminal-screen` (#528, #530).

## L-claude-536-an-agents-terminal-is-titled-by-its-process-001
*category: e2e · topic: finding a stand-in agent's terminal in a scenario · from: #536's visual check*

A stand-in agent run as `exec -a claude cat -v` is an agent to Marley, which reads the process's
first argument (`foreground_process_command_from_argv`), but Zed titles the terminal from the
process's own name, so its tab and `mcp_agent terminal-screen` see `repo — cat -v`, not `claude`.
A scenario finds such a terminal by the project's name and by which terminal came last. The agent
bar the stand-in brings also takes the terminal's bottom rows, so rows measured without an agent
move up once one runs.

## L-claude-551-an-agents-block-outlives-the-agent-001
*category: terminal · topic: telling an agent's terminal from a plain one after the agent exits · from: #551's visual check*

`agent_in(terminal)` reads the foreground process, so it answers `None` the moment the agent CLI
exits, while the block that ran it (`claude`, Return) ends only then. Code that acts on a block's
end, or draws a terminal's last block, and means to leave agents out, must also test the block's
own command with `marley_agent::agent_kind_of`; otherwise an ended session reads as a plain
command that ran for hours, as the rail's `claude · done · 4 s` did in #551's run 4.

## L-claude-551-the-safety-comment-sits-directly-above-the-unsafe-001
*category: gates · topic: gate:13 and multi-line SAFETY comments · from: #551's Code gate*

gate:13 (`unjustified_unsafe` in `script/gates.sh`) accepts an `unsafe` only when `SAFETY:` is on
its own line or the line directly above. A two-line comment that starts `// SAFETY:` and wraps
onto a second line leaves the second line above the `unsafe`, and the gate goes red. Put the
context first and the one `// SAFETY:` line last, directly above the `unsafe`.

## L-claude-572-a-user-bus-check-names-marley-001
*category: e2e · topic: proving no banner reached the user's desktop · from: #572's visual check*

A scenario that runs Marley on a private bus and watches the user's own bus with `busctl monitor`
sees every program's notifications there, not only Marley's: #572's run 2 failed its "no banner
reached the user's bus" check on a `Notify` another program on the desktop posted during the
run. Match Marley's app name in the call's arguments, `STRING "Marley"`, as #535's check does,
never `Member=Notify` alone. #538's and #551's scenarios still use the loose form and can fail the
same way on a busy desktop.

## L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001
*category: gpui · topic: a watcher's state and the rail's marks · from: #572's design*

`cx.default_global::<G>()` and `global_mut` tell `G`'s observers it changed, whether or not a
value did, and the rail refreshes on the globals it observes. A watcher that writes its own state
at every `Event::Wakeup`, as a line reader does, must keep that state in a global nothing observes
and put what a row draws in a second one, written only when a mark changes (`running_errors`'
`Watch` and `ErrorMarks`), or every byte of output redraws the rail. Read with `try_global` before
deciding to write, as `links.rs`'s scan does.

## L-claude-556-nothing-typed-since-the-prompt-is-an-unset-input-start-001
*category: terminal · topic: telling an empty prompt from one the user typed at · from: #556's design*

`autosuggest::typed_text` answers `None` both when the line cannot be read and when nothing was
typed since the prompt: `AnchoredBlocks::input_start()` is unset until the first key or paste
(`Terminal::input` notes both). Code that refuses to type over the user's input must read an
unset `input_start()` as an empty line and use `typed_text` only after it is set; treating every
`None` as "cannot read" refuses at every fresh prompt.

## L-claude-553-a-shell-rule-is-checked-by-hand-in-a-pty-first-001
*category: e2e · topic: testing Marley's shell scripts before a Marley build · from: #553*

The shell integration is compiled into Marley, so each script change costs a build and a
scenario. `script -qfc "bash --rcfile crates/marley_terminal/shell_integration/marley.bash -i"
/dev/null`, fed the exact bytes Marley types (`printf '\025 echo x\r'`) with short sleeps, and a
HOME of the scratchpad, runs the real script in a pty with readline in seconds; `grep -ao
"preexec;command=[^;]*"` on its output shows the frames. For zsh, copy `marley.zsh` to a
directory's `.zshenv` and start `zsh -i` with `ZDOTDIR` there. Two of #553's bugs were found and
fixed this way between one scenario run and the next.

## L-claude-552-a-read-that-redraws-must-redraw-only-on-a-change-001
*category: gpui · topic: state a render path asks to be refreshed · from: #552's design*

A chip whose state comes from a file, re-read when its bar draws and the last read is stale, loops
if the read always calls `cx.refresh_windows()`: the refresh draws the bar, the bar finds the read
stale again a moment later, and so on while the bar is on screen. Compare what was read with what
is held and refresh only on a change; the staleness window then bounds the IO, not the frames.

## L-claude-543-a-zed-tasks-arguments-are-shell-text-001
*category: zed · topic: a program run through a Zed task · from: #543's visual check*

A Zed task's command and arguments reach its shell unquoted: `prepare_task_for_spawn` builds the
shell line with `ShellBuilder::build_no_quote`, so each argument is shell text there, and
`resolve_task` has already expanded `$` variables in it. A word meant for a second shell (ssh's
remote one) loses a level of quoting: `\;` arrived at ssh as `;`. Quote every word for the task's
shell (`ShellKind::system().try_quote`) and keep the argv a builder returns the one a direct exec
takes; a stand-in that logs its argv shows what really arrives.

## L-claude-543-tmux-on-this-box-for-a-scenario-001
*category: e2e · topic: a tmux server a scenario or a check starts · from: #543*

A tmux socket's path must fit a Unix socket's 108 bytes, and a run's folder under the scratchpad
does not: give the server a `TMUX_TMPDIR` of `mktemp -d "${TMPDIR:-/tmp}/e2e-tmux.XXXXXX"` and kill the
server (`tmux -L <name> kill-server`) and the folder at teardown. To see what tmux writes to its
client, run the client under `script -q -f` (without `-f` the log misses the last writes), and keep
`script`'s stdin open (`sleep N | script …`): stdin at `/dev/null` sends an EOF that logs the pane's
shell out. Claude Code 2.1.283 wraps a hook's `terminalSequence` for tmux itself when `TMUX` is set,
and tmux forwards it only with `allow-passthrough on`; a stand-in reproduces both.

## L-claude-588-a-detached-programs-exit-and-a-coprocs-pid-001
*category: e2e · topic: stopping what a shell runner started · from: #588*

A program started with `setsid -f` is nobody's child, so its exit status is lost unless a wrapper
records it (`prog "$@"; echo "$?" > file`, written as a quoted heredoc, since shellcheck flags
`sh -c '… $1 …'`). Bash unsets a coprocess's `NAME_PID` once it reaps it, so under `set -u` read it
as `${NAME_PID:-}`. A compositor killed with SIGKILL leaves its IPC socket and its Wayland socket
and lock in `$XDG_RUNTIME_DIR`, and a Browser tab's Chromium runs in a user unit that outlives
Marley and keeps writing its profile after the runner removed it: stop the units named by the
run's profiles before removing the profile.

## L-claude-541-semgrep-resolves-imports-and-recovers-parses-001
*category: gate · topic: a semgrep rule that must find each form · from: #541*

semgrep 1.156.0 resolves a Rust `use`: after `use alacritty_terminal::tty;`, the full-path pattern
`alacritty_terminal::tty::new(...)` matches `tty::new(...)`, so a planted file with imports cannot
tell whether the short pattern works. Plant each short form without the import, and prove each
pattern by removing it once. Its Rust parser is tree-sitter's and recovers from most broken
source without an error; `--strict` exits non-zero only on what it reports as a syntax error (a
file of random bytes does). `.paths.scanned` is in its `--json` output without `--verbose`.

## L-claude-545-a-refused-activation-token-looks-like-a-good-one-001
*category: wayland · topic: xdg-activation tokens in a check · from: #545*

wlroots answers a token request it refuses (no keyboard focus on the surface, or a serial it never
gave the client) with a random token, so a refused token cannot be told from a good one until an
`activate` with it does nothing. GTK4's launch context asks with its last pointer press's serial, so
a stand-in launcher needs a click on its window first. Tokens expire (wlroots: 30 s), so ask for one
just before it is used, and sway acts on a valid one only as `focus_on_window_activation` says (the
default `urgent` marks the window urgent; `smart` focuses it on a visible workspace).

## L-claude-486-the-first-terminal-bug-shows-on-hyprland-not-sway-001
*category: e2e · topic: a check that depends on startup timing · from: #486*

A bug that depends on whether a terminal's view lays out before its shell starts readline shows on
Hyprland's hidden workspace and not in the headless sway, where the first view lays out sooner.
Before a check's shots are trusted, see the bug on the backend the check uses: a run that is green
before the fix proves nothing.

## L-claude-597-fleet-tools-answer-but-are-not-listed-001
*category: validate · topic: e2e checks of Marley's MCP server · from: pipeline 597*

Marley's MCP server answers `fleet_snapshot` and `session_surface_to_human` on `tools/call`, but
`tools/list` lists only the served families (`Family::is_served`: terminal, browser, ports), so a
fleet tool's output schema reaches no client, and `session_surface_to_human` is refused by the
`session.write` grant (deny-by-default) before its handler runs; the app also builds no surface
index. A scenario that means to show a fleet schema or an accepted surface cannot: plan those
criteria as a review of the diff, and check the verb only as answered by its name.

## L-claude-598-a-menu-entry-moves-with-the-scenarios-that-count-to-it-001
*category: plan · topic: e2e scenarios that walk a menu by position · from: pipeline 598*

Scenarios reach a menu entry by pressing Down a fixed number of times and click a submenu at fixed
points (`WORKTREE_STEPS`, `SUBMENU_CLAUDE_X/Y`, `WORKTREE_ENTRY_Y`), and several copy the same
numbers from each other: moving New Agent in Worktree broke four of them, two in the golden set.
Before a ticket moves, adds or removes an entry in a menu, grep `script/e2e/` for the entry's
label and for step-count variables near it, put every hit in the ticket's scope, and measure the
new points from a shot of the open menu rather than by arithmetic on row heights.

## L-claude-600-a-hand-deployed-menu-stops-and-prevents-its-mouse-down-001
*category: code · topic: context menus in gpui views that track focus · from: pipeline 600*

When a view deploys a `ContextMenu` from its own `on_mouse_down(MouseButton::Right, …)` (because
`ui::right_click_menu`'s wrapper lays out with a default style and would collapse a `flex_1`
child), call `cx.stop_propagation()` and `window.prevent_default()` in the handler before focusing
the menu: an ancestor that calls `track_focus` otherwise focuses itself on the same mouse-down and
takes the keyboard from the menu. A `ContextMenu` opens with its first selectable entry selected,
so a scenario reaches entry N with N−1 Downs, and a disabled entry is skipped.

## L-claude-601-a-launch-with-a-path-restores-no-session-001
*category: validate · topic: e2e scenarios that restart Marley · from: pipeline 601*

Zed answers a launch that names a path as an open request and skips the last session's restore
(`crates/zed/src/main.rs`, the `restore_task` match): only that path opens, and every other
workspace of the window, its sidebar state and its groups stay closed. The runner's
`launch_marley` passes the scenario's `OPEN` folder, so a scenario that checks what a restart
brings back clears `OPEN` before relaunching, as the app menu starts Marley with no path.

## L-claude-602-nested-drop-targets-need-a-drag-type-each-001
*category: design · topic: gpui drag and drop · from: pipeline 602*

In gpui a drop goes to the deepest hovered element with an `on_drop` of the drag's type, which
takes the drag before it asks `can_drop`: when the answer is no, the drag is simply gone, and no
element around it sees the drop. So a target nested in another (a rail row inside its header's
block) must listen for a different drag type than the one around it: with one shared type, a
header dropped over a row would vanish instead of reaching its block. Two further facts from the
same code (`crates/gpui/src/elements/div.rs`): hover reads false for every element while any drag
is active, so a hover-kept state ends the moment a drag starts unless the drag keeps it; and a
successful `on_drop` stops propagation, so a parent's `on_mouse_up` does not run after a drop and
the drop handler has to end whatever the drag began.

## L-claude-602-zeds-project-group-list-never-holds-a-folderless-workspace-001
*category: design · topic: Zed's MultiWorkspace · from: pipeline 602*

`MultiWorkspace::project_groups` skips an empty key when it adds and when it restores a group
(`ensure_project_group_state`, `restore_project_groups`), and a new project goes to its top
(index 0). An order that must place Marley's projectless groups among projects cannot live in
that list; the rail keeps its own (`SavedOrder::headers`). A Browser tab's saved item id is not a
stable id across restarts either (the item id is the view's entity id, new each start); its page
target is.

## L-claude-603-stop-a-service-not-its-process-001
*category: design · topic: stopping servers the user did not start in Marley · from: pipeline 603*

A server that a systemd unit runs with `Restart=always` comes back five seconds after its
process gets SIGTERM, since systemd counts that as an exit to restart from; a unit with
`Restart=on-failure` stays down after a SIGTERM but reads as inactive though no one stopped it.
The process's `/proc/<pid>/cgroup` names its unit (`0::…/<name>.service`, the user's own under
`user@<uid>.service`), the same path `systemctl status <pid>` reads, so the rail can tell a
service's process from a plain one and stop the unit with `systemctl [--user] stop`. The system's
goes through polkit, which asks the desktop's agent; Marley never runs `sudo`. Only the path's
last part counts: a scope under a service's slice is a process.

## L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001
*category: validate · topic: e2e scenarios · from: pipeline 603*

`script/e2e.sh` runs a scenario's `setup` in its own shell before it starts Marley, so a PATH
exported there reaches Marley's own spawns (`process::output`), and not only its terminals, which
`terminal_env` covers. A fake that answers one case and `exec`s the real program for everything
else (the runner itself calls `systemctl --user stop` for its browser units) keeps the rest of the
run real. Units or processes a scenario starts outside Marley go in a `teardown`, which the
runner's EXIT trap calls however the run ends.

## L-claude-604-the-rails-enter-opens-through-open-row-not-a-rows-click-001
*category: design · topic: the rail's keyboard · from: pipeline 604*

AD-claude-453 says Enter runs a row's click handler; the code does not. `confirm` calls
`open_row(selection)`, which opens each kind of row itself (a port's through `open_port`), so a
row's click can change what it does, as #604's port row now marks on one click, without Enter
changing. When a row's click changes, check `open_row`'s arm for that row before touching Enter.
A click that should leave the keyboard on its row must focus the rail too (`mark_row`): the
cursor counts only while the rail holds focus, and focus leaving drops it.

## L-claude-605-zed-sends-acp-request-ids-as-strings-001
*category: validate · topic: e2e stand-in agents · from: pipeline 605*

Zed's ACP client sends its JSON-RPC request ids as strings, not numbers. A stand-in agent that
formats an id as a number (`"stand-in-%d" % request_id`) dies with a `TypeError` at
`session/new`; Zed then shows "Failed to Launch: Incoming transport closed", and the traceback is
in Marley's log under `agent_servers::acp` as `agent stderr`. Echo the id back as it came, and
number anything else (sessions) with a counter of the stand-in's own. A stand-in that declares
`loadSession: false` also cannot resume a thread after a restart: the panel shows "Loading or
resuming sessions is not supported by this agent", which is the fixture's limit and not a fault.

## L-claude-606-just-install-reads-sources-as-it-reaches-each-crate-001
*category: process · topic: building while editing on the dev box · from: pipeline 606*

`just install` builds the release binary, which takes long enough that `rustc` reaches
`marley_workbench` minutes after it started, and reads that crate's sources then, not when the
build began. Edits made to a crate while an install is compiling can land in the installed
binary, tested or not (here #606's first, unfinished edits could have). Start editing a crate
only once an install is past it, or run `just install` again after the commit, which is what
#606 did.

## L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001
*category: testing · topic: e2e scenarios that edit settings.json · from: pipeline 607*

Once Marley has written `settings.json` itself (a layout switch, the settings page, Zed's
pickers), a later edit of the file from outside does not reload in that session (TICKET-612;
a probe changed `ui_font_size` before and after a layout round trip, and only the first took).
A scenario that edits the settings mid-run does its edits before anything in Marley writes the
file, or relaunches Marley after. When a scenario's settings change seems ignored, log the
setting's value at the reader before suspecting the reader.

## L-claude-610-a-host-scenario-on-this-machine-sees-its-real-agents-001
*category: testing · topic: e2e scenarios for the host collector · from: pipeline 610*

A scenario's `sshd` on 127.0.0.1 is the dev box itself, so the collector over it, and the local
one, list every Claude Code and Codex the user has running: the rows under a host vary from run
to run and can push the panel past the window. Such a scenario puts the hosts with a problem
first (the Hosts section does), scrolls for the rest, and works out a stand-in's row from `/proc`
in the collector's own order (`standin_row` in `610-host-collector-over-ssh.sh`) rather than
fixing a y. Its notes describe the stand-ins only.

## L-claude-613-zeds-added-to-workspace-is-the-hook-for-an-item-that-changes-workspace-001
*category: design · topic: Zed items moving between workspaces · from: pipeline 613*

`workspace::move_item` works across workspaces of one window (it checks no workspace), and its
add runs the item's `added_to_pane`, so `Item::added_to_workspace` sees the new workspace. An
item that keeps workspace or project handles from its constructor (as `TerminalView` does) goes
stale after such a move unless `added_to_workspace` re-points them; that is the smallest place to
fix it, and a no-op for an item added once.

## L-claude-616-zeds-context-menu-closes-cleanly-one-submenu-deep-001
*category: design · topic: gpui context menus · from: pipeline 616*

`ui::ContextMenu` closes the whole menu when an entry of a first-level submenu is chosen, but a
choice in a submenu of a submenu closes only the submenus: a parent is dismissed only when its
open submenu was itself clicked. Keep a menu's choices at most one submenu deep, or put a second
level's actions on the thing itself (here, a thread's row once it is back).

## L-claude-617-a-stand-in-agent-without-loadsession-cannot-prove-a-thread-reopens-001
*category: testing · topic: e2e stand-in ACP agents · from: pipeline 617*

#605's stand-in agent declares `loadSession: false`, so after Marley restarts no thread of it can
be opened: the Agent Panel says "Loading or resuming sessions is not supported by this agent".
A scenario that reopens a thread across a restart needs a stand-in that keeps each session's
messages and answers `session/load` by replaying them (#617's). And since a restored workspace's
Agent Panel shows its last thread by itself, the thread a scenario opens should be another one, or
the shot cannot tell the change from Zed's restore.

## L-claude-618-hover-buttons-over-a-rows-end-need-an-opaque-shade-001
*category: design · topic: gpui rows with hover buttons · from: pipeline 618*

`visible_on_hover` hides an element but keeps its layout, so a row's hover buttons take their
width from its text all the time. To give the text the row's width, make the buttons an
`absolute()` strip at the row's end (the row `relative()`; gpui has no `inset_y_0`, use `top_0`
and `bottom_0`) and give the strip an opaque background: the theme's hover and selected fills are
translucent, so blend them over the background the row sits on (`panel_background.blend(fill)`).

## L-claude-612-a-watched-config-file-on-linux-needs-its-folder-watched-001
*category: debugging · topic: file watching on Linux · from: pipeline 612*

On Linux, Zed's `Fs::watch` on a plain file is an inotify watch on its inode, and Zed's writers
(`Fs::atomic_write`, so `update_settings_file`) save by renaming a new file over the path, which
ends the watch: the next edit is unseen until a restart. A file's inode number before and after a
write (`stat -c %i`) shows it at once, and notify logs "unable to remove watch descriptor" at the
replace. Watch the file's folder as well and filter its events to the file, as
`settings::watch_config_dir` does; #612 does it in `watch_config_file` on Linux and FreeBSD.

## L-claude-578-a-screencast-start-sends-one-frame-and-a-lost-one-is-not-sent-again-001
*category: debugging · topic: CDP screencast · from: pipeline 578*

Headless Chromium answers `Page.startScreencast` with one frame within about 20 ms, even on a page
left still for seconds (a Node probe with its built-in `WebSocket` against
`/usr/lib/chromium/chromium --headless --remote-debugging-port=0`, 2026-09-30), and sends no
other until the page's pixels change (L-claude-499). So a stream whose first frame is lost, by a
failed start, a failed decode or a frame for a page the client does not know, stays blank on a
still page. A client that streams pages to a view should watch for a first frame and start the
stream again without one, as #578's hub does.

## L-claude-578-zlog-filters-take-a-crates-name-001
*category: debugging · topic: Marley's logs in e2e runs · from: pipeline 578*

`ZED_LOG` (or `RUST_LOG`) reaches Marley from a scenario's `setup` with `export`, and `zlog` takes
a crate's name as a directive: `ZED_LOG="info,marley_workbench=debug"` logs the crate's debug
lines into `$E2E_PROFILE/logs/Marley.log`, while `marley_workbench::browser=debug` logged none of
them. A scenario that wraps another (`eval "flow_$(declare -f setup)"`) can add the export
without copying it.

## L-claude-619-a-scenario-that-sets-terminal-settings-cannot-use-terminal-env-001
*category: validate · topic: e2e scenarios · from: pipeline 619*

`terminal_env` writes a `"terminal": {"env": …}` block into the run's settings at launch and
refuses when the settings already have a `terminal` block. A scenario that also sets a terminal
setting (`terminal.max_scroll_history_lines`, `terminal.shell`) sets the environment through the
same block instead: `set_setting terminal.env '{"HOME": "…"}'`. And Zed's terminal shows a link's
tooltip only after the pointer moves over the link once the link is found with Ctrl held: point,
wait a second, move a few pixels, then shoot.

## L-claude-620-a-scroll-shot-needs-the-target-off-screen-first-001
*category: validate · topic: e2e scenarios · from: pipeline 620*

A shot meant to show that an action scrolled a line to the top proves nothing when the line was
already there: #620's first run printed exactly a screen's worth after the error, so the
bottom-anchored terminal showed it at the top before any jump. Print well over a screen below the
target, and take a shot before the action that shows the target is out of view.

## L-claude-621-a-short-terminals-rows-sit-at-the-bottom-001
*category: validate · topic: e2e scenarios · from: pipeline 621*

Since #476 a terminal draws its content against the bottom edge, so a short output's rows sit just
above the prompt at the bottom of the view, not at its top: a scenario's click "on the output"
counted from the top lands on empty rows, and a right-click there opens Zed's terminal menu with no
Block section. Count rows up from the prompt's row (the last row) instead.

## L-claude-623-zeds-task-rerun-replaces-the-tabs-terminal-001
*category: code · topic: Zed task terminals · from: pipeline 623*

Zed's `task::Rerun` with `use_new_terminal: false` keeps the tab but gives its `TerminalView` a new
`Terminal` entity (`TerminalPanel::replace_terminal`, then the view's `set_terminal`). Anything keyed
by the `Terminal` (an observer, a per-terminal cache, its blocks) starts over at a rerun: watch the
view and follow its current terminal, as `failures::watch` does.

## L-claude-625-zeds-dylint-runs-in-the-gate-not-in-just-clippy-001
*category: build · topic: the gate's lints · from: pipeline 625*

`just clippy <crates>` runs clippy only; the gate's gate:21 also runs Zed's own dylint lints on the
Marley crates, and they catch what clippy does not: an `async` block with no `.await` inside
(`executor.spawn(async move { read_dir(..) })` in an already-background task) failed #625's first
gate. Read the work in place when the task is already off the main thread, and expect gate:21 to
speak where clippy was quiet.

## L-claude-626-a-zed-function-with-an-outside-caller-gets-a-marley-twin-001
*category: code · topic: upstream discipline · from: pipeline 626*

`TerminalElement::layout_grid` is public, and Zed's `repl` crate calls it too; adding a parameter
for Marley's need would have touched a second Zed crate (`crates/repl/src/outputs/plain.rs`) and
its ledger. Before changing a Zed function's signature, grep every crate for its callers; when one
is outside the hunk's crate, keep the signature, move the body into a private twin that takes the
extra argument, and have the public function call the twin with none.

## L-claude-626-type-a-literal-dollar-without-a-shellcheck-disable-001
*category: validate · topic: e2e scenarios · from: pipeline 626*

A scenario that types a variable for the terminal to see (`$HOME`) trips shellcheck's SC2016 in
single quotes, and the gate's no-suppressions rule makes a disable a poor answer: write it in
double quotes with the dollar escaped, `type_text "grep h \$HOME"`.


## L-claude-627-count-what-happened-instead-of-sampling-a-flag-001
*category: design · topic: gpui observers · from: pipeline 627*

An observer that reacts to a state's edges only sees the state at each notify, and a terminal can
go through a whole command between two of them. To act on "the prompt came again", compare
something that only grows with each occurrence (the block count, an index) with what the last
notify saw, not a boolean that ends where it started.

## L-claude-573-detach-an-ask-in-flight-instead-of-dropping-it-with-its-line-001
*category: design · topic: System One uses · from: pipeline 573*

`system_one::ask` writes its call's row when the answer comes, in the task it returns. A use that
cancels stale work by dropping a task must keep the ask out of that task: drop only the timer
that leads to the ask, detach the task that awaits it, and compare the answer with the state
still in front. The row is always written, and a late answer can say so (`dropped`).

## L-claude-466-fish-4-waits-for-its-terminal-queries-001
*category: validate · topic: shell integration scripts · from: pipeline 466*

fish 4 asks the terminal about itself as it starts and after each prompt (DA1 `ESC [ c`, a
cursor report `ESC [ 6n`, XTGETTCAP `ESC P + q`) and waits for the answers. Under `script -qfc
'fish -i'` nothing answers, so fish shows one prompt and reads no more input. To prove a fish
script on a PTY, feed the answers on its input before and after each line:
`printf '\033[1;1R\033[?62c'`. Zed's terminal answers them, so a real Marley needs nothing.

## L-claude-466-a-bash-comment-that-starts-with-shellcheck-is-a-directive-001
*category: validate · topic: shell scripts · from: pipeline 466*

shellcheck reads any comment whose first word is `shellcheck` as a directive, so a plain remark
such as `# shellcheck cannot read fish` fails the parse (SC1126, SC1073). Start such a comment
with another word.

## L-claude-628-check-a-copy-through-the-prompt-editor-with-copy-on-select-001
*category: validate · topic: e2e scenarios · from: pipeline 628*

Since #627 the prompt editor takes the focus at every prompt, so a scenario cannot press the
terminal's copy key after a drag: the focus has gone back to the editor. Turn on
`terminal.copy_on_select` in the run's settings (in the `terminal` block that also carries the
env, as `terminal_env` refuses a second one), drag, and press Ctrl+V: the editor shows what the
drag copied.

## L-claude-629-a-search-after-the-block-keys-is-held-to-the-selected-block-001
*category: validate · topic: e2e scenarios · from: pipeline 629*

The block keys (Ctrl+Up and Ctrl+Down) select the block they land on, and Zed's search in a
terminal with a selected block is held to that block (#559). A scenario that searches to see
matches across blocks searches before the block keys, or clears the selection first.

## L-claude-534-zeds-mcp-client-sees-no-server-exit-001
*category: design · topic: MCP clients · from: pipeline 534*

Zed's `context_server` client does not notice a stdio server exiting: `wait_for_shutdown` fires on
a failed send only, a request in flight waits out its timeout (60 s by default), the server's
stderr goes to Zed's debug log, and an error's JSON-RPC code is dropped for its message. A client
that must say when its server is gone polls with a bounded call (5 s here) and treats a failure as
the exit, reads errors from their text (`resync_required`), and writes its own `Request` type for
a method whose answer is `{}` where Zed's typed one expects `null` (`ResourcesSubscribe`).

## L-claude-534-a-harness-scenario-refuses-the-root-to-hold-a-down-state-001
*category: validate · topic: e2e scenarios · from: pipeline 534*

A client that reconnects after a second leaves no time to shoot it down. `rh mcp` refuses a state
root whose mode is not 0700, so a scenario that kills `rh mcp` after `chmod 0755` on the root
holds the client down for as long as it likes, and `chmod 0700` lets the next retry in. The root
must be short (its sockets live under it): `$XDG_RUNTIME_DIR/rh534-<pid>`, not the scratch tree.

## L-claude-632-a-new-spawn-in-a-listed-site-shares-its-spawn-call-001
*category: validate · topic: gates · from: pipeline 632*

gate:22 counts spawn calls, not files: a second function that starts a process in a listed site
(`process.rs`) raises the count past `SPAWN_SITES_PIN`. When the new function differs only in how
it pipes (stderr kept), have both call one private function that spawns, so the count and the pin
stay; move the pin only for a new kind of program.

## L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001
*category: validate · topic: e2e scenarios · from: pipeline 633*

`script/e2e.sh` copies the user's `~/.config/marley/settings.json` into each run's profile, and
Marley inherits the user's PATH, so a default that starts a program where it is installed starts
the user's real one in every scenario: Zed runs each enabled context server when a project opens.
Turn such a default off in the harness's copy (as `marley.rusty_tools` is) and let the ticket's own
scenario turn it on with a stand-in first on the PATH.

## L-claude-633-the-mcp-servers-page-opens-by-a-keymap-in-the-runs-profile-001
*category: validate · topic: e2e scenarios · from: pipeline 633*

Zed's Settings window search does not list the MCP Servers page, and the palette cannot pass
`zed::OpenSettingsAt`'s path. A scenario writes `$E2E_PROFILE/config/keymap.json` binding a key to
`["zed::OpenSettingsAt", {"path": "context_servers"}]` and presses it; the page lists each server
with its state's dot.

## L-claude-475-tell-a-stale-test-from-your-change-by-running-it-without-the-change-001
*category: validate · topic: the test pass · from: pipeline 475*

When a test-environment change meets failing tests, run the failures once with the change taken
out (emptied with an Edit, put back with the inverse Edit) and compare the messages. Seven
`marley_workbench` failures (five "Your test is not deterministic", one focus assertion stale
since #627, one missing `RequestedDirectories`) failed the same way without #475's ctor, so they
went to #634's triage, not #475's. Keep such a run off the user's files: with the data
directory's fix taken out, `XDG_DATA_HOME` on a scratch folder stands in for it.

## L-claude-634-a-task-terminal-opens-its-own-block-first-001
*category: code · topic: block tests · from: pipeline 634*

Since #621 a local task terminal opens the task's own block at index 0 and the first prompt
ends it, so a test that runs its script as a task (`TerminalMode::task`) finds the script's blocks
from index 1, and counts one more finished block. Five tests in `terminal` and `terminal_view`
had read block 0 as the script's first.

## L-claude-634-gpui-fails-a-test-at-its-end-for-a-foreign-wake-001
*category: validate · topic: the test pass · from: pipeline 634*

"Detected activity on thread …, Your test is not deterministic" is raised at the test's end, so
the test's assertions all ran and passed. Its backtrace shows only the waking side (smol's
reactor `async-io`, the reaper `async-process`, the pool `blocking-N`); find the waiting side by
the thread's name: processes and pipes for the first two, `smol::unblock` for the last.
`allow_parking()` also silences it, which hides real IO instead of removing it.

## L-claude-635-a-scenarios-fixed-menu-steps-can-pass-on-the-wrong-entry-001
*category: validate · topic: e2e scenarios · from: pipeline 635*

511's `merge` pressed End, Merge being the last entry, and #589 put Remove… after it: the three
"the refusal changed nothing" checks kept passing on Remove's refusal while Merge's went untested,
and only the later merge check failed. A scenario that walks a menu by position checks the entry
it reached (a shot read, or a check on the effect only that entry has), and a menu change reruns
the scenarios that walk it.

## L-claude-635-a-harness-helper-replaces-a-scenarios-own-of-the-same-name-001
*category: validate · topic: e2e scenarios · from: pipeline 635*

`script/e2e.sh` sources the scenario before it defines its helpers, so a helper added to the
harness under a name a scenario already uses silently replaces the scenario's (571's `setting`).
Before adding one, compare its name with every scenario's functions.

## L-claude-636-a-mutation-run-reads-only-the-mutated-crates-tests-001
*category: validate · topic: mutation testing · from: pipeline 636*

`cargo-mutants` runs the mutated crate's own tests, so a module proven only by e2e scenarios or by
another crate's tests loses nearly every mutant (`marley_agent`: 5 unit tests, 3.2% killed),
while crates built with tests kill everything. Read a run per file: sort each survivor by whether
a test of its crate names its function; those that do are the ones worth reading for equivalents
and bugs (90 of 1,448 here), and a file with no test at all is one finding, not hundreds. A
common-word function name (`run`, `summary`, `matches`) fools the sort, so read before counting.
The run itself: copy mode, two workers, a target per copy, the copies under `~/.cache` on
`/home`, 1 h 27 min for 2,658 mutants, and the working tree never touched.

## L-claude-637-a-propagating-action-lets-the-keys-next-binding-run-001
*category: build · topic: gpui key dispatch · from: pipeline 637*

When several bindings match one keystroke, gpui dispatches their actions in turn, deepest context
first and the later-defined first at a depth, and stops at the first handler that does not call
`cx.propagate()` (`crates/gpui/src/window.rs`, the `match_result.bindings` loop). So a key can
mean a Marley action only sometimes: bind it in a deeper context (`MarleyShellInput > Editor`),
handle the action on the container, and propagate when there is nothing to do; the editor's own
binding for the key (`editor::MoveRight`) runs next. The propagated action still bubbles to every
ancestor that handles it first, so a workspace-level handler for the same action must also stand
aside (here, the grid's AcceptSuggestion while the editor holds the line).

## L-claude-631-the-bottom-shift-makes-every-short-terminal-look-clipped-001
*category: build · topic: terminal element geometry · from: pipeline 631*

#476's bottom shift moves the grid's bounds down past the element's bottom edge for every
terminal whose screen is not full, so `intersection == content_bounds` in the element's prepaint
is false for most terminals, and the cell layout takes its clipped path (rows culled by pixel
from the top). Anything that wants to know whether the view is clipped must test the edge it
cares about (`intersection.top() == content_bounds.top()`), not the whole rectangle. Also: the
prompt the shell waits at is no block; it is `AnchoredBlocks`' staged prompt
(`staged_line`), so a rule over "every block on screen" leaves it out unless it asks.

## L-claude-540-a-queued-spec-is-redesigned-against-what-shipped-since-001
*category: plan · topic: promoting a queued spec · from: pipeline 540*

#540's spec was drafted on 2026-09-25 and waited a week; in that week #575 gave every restored
terminal its old `MARLEY_TERMINAL_ID` through a table of Marley's own. The draft's design (two
columns on Zed's `terminals` table, a field and a setter on `TerminalView`) was the right answer
without that key and the wrong one with it: the session could ride the terminal id, and the
ticket shipped with no Zed change but the setting's. At promotion, list the tickets that shipped
since the draft on the same seams (here `git log` over `terminal_view.rs` and the workbench's
restore paths) before re-verifying line numbers; a new seam can delete half the plan.

## L-claude-638-a-forks-actions-stay-off-until-its-owner-turns-them-on-001
*category: tooling · topic: GitHub Actions on the fork · from: pipeline 638*

`Ignibyte/marley_ide` reports Actions as enabled (`actions/permissions` says `enabled: true`),
yet lists 0 workflows and has 0 runs ever, though 47 workflow files sit on `main`: GitHub keeps a
fork's workflows off until its owner chooses "I understand my workflows, go ahead and enable them"
in the Actions tab, and no API call does it. Turning them on turns on every inherited workflow at
once, so the step that turns Zed's off (`script/mutants cloud-setup`) has to follow it straight
away. A push-triggered workflow then runs from the pushed commit, with no need to be on the
default branch; a `workflow_dispatch` one needs the default branch.

## L-claude-642-marley-settings-from-settings-is-at-clippys-line-cap-001
*category: code · topic: MarleySettings · from: pipeline 642*

`MarleySettings::from_settings` (`marley_workbench.rs`) is at exactly 100 lines since #642, the
`too_many_lines` cap the Marley crates' lint table enforces, and every Marley setting adds a field
to its one struct literal. #642's five-line read of a nested key failed gate:2 at 104/100. A new
key reads through a constructor that takes the whole block, `Kind::from_content(marley)` with
`marley: Option<&settings::MarleySettingsContent>` (`ResumeAgents` and `Dictation` have it), so
the literal grows by one line; a key with real reading work gets its own function, as
`push_settings` and `fleet_settings` do. Splitting the literal itself is the next step when one
line no longer fits.

## L-claude-642-marley-open-settings-focuses-the-page-list-001
*category: e2e · topic: the Settings window in a scenario · from: pipeline 642*

`SettingsWindow::new` focuses its search bar as it builds, but `marley: open settings` then opens
the Marley page and leaves the page list focused (its Marley entry outlined), so text typed as the
window opens goes nowhere and a search shot shows the full page. Press Ctrl+F first:
`search::FocusSearch` is bound in the `SettingsWindow` context and puts the keys in the search;
Ctrl+A then selects the query for the next one. `zed::OpenSettingsAt` to a sub-page (#633's
Ctrl+Alt+Shift+M) needs no search at all.

## L-claude-642-pgrep-reads-its-pattern-as-a-regular-expression-001
*category: e2e · topic: process checks in scenarios · from: pipeline 642*

`pgrep -f` matches an extended regular expression, not a fixed string. A check for a fake's
`tail -n +1 -f <file>` never matched: ` +1` reads as one or more spaces then `1`, so the literal
`+` in the command line was never found, and the check failed while the process ran. Put each
metacharacter of a command line in a bracket (`tail -n [+]1 -f …`) or match a part with none.
The older traps still hold: a `pgrep -f` inside `bash -c` matches its own wrapper (L-448).

## L-claude-642-a-wasmtime-patch-moves-with-the-wasm-tools-it-asks-for-001
*category: tooling · topic: cargo-audit and the lockfile · from: pipeline 642*

Seven wasmtime advisories published on 2026-10-02 turned gate:7 red during #642, as three had
during #511. The fix is the family's patch release inside Zed's `"48"` requirement, but `cargo
update` with every wasmtime, wiggle, pulley and cranelift crate named still left wasmtime at
48.0.3 and printed "available: v48.0.5": 48.0.5 asks `^0.254.1` of the wasm-tools crates
(`wasmparser`, `wasm-encoder`, `wasmprinter`, `wasm-metadata`, `wit-component`, `wit-parser`),
locked at 0.254.0, and cargo keeps a locked version it was not told to move. Name those crates
in the same `cargo update` (diff a release's requirements with `index.crates.io` to find which).
The lockfile is gate-defining, so the commit hook refuses a lockfile-only commit without a green
`--diff` receipt, and a receipt binds HEAD: a bump found during a ticket's gate rides in that
ticket's commit. The gate's clippy scope builds no wasmtime; `just build` proves it compiles.

## L-claude-640-a-private-item-linked-from-a-public-doc-fails-only-the-docs-gate-001
*category: gate · topic: rustdoc on the Marley crates · from: pipeline 640*

`cargo clippy` passed, and gate:14's `rustdoc -D warnings` failed 40 minutes into the gate: the
public module `harness`'s `//!` doc linked [`Signals`], a `pub(crate)` struct
(`rustdoc::private_intra_doc_links`). Clippy does not run rustdoc's lints, so nothing before the
gate catches it. In a module doc or a `pub` item's doc, name a crate-private item in plain code
text (`Signals`), never as an intra-doc link; links are fine between private items' docs. To check
before the gate: `cargo doc -p <crate> --no-deps` with `RUSTDOCFLAGS="-D warnings"`.

## L-claude-640-a-stand-in-harness-serves-the-fleet-tools-from-a-fixture-001
*category: e2e · topic: scenarios that need a harness · from: pipeline 640*

A scenario that needs harness sessions with particular labels does not need a built rustal-harness:
`script/e2e/640-agent-state-source-progress-quota.sh` writes a stdio MCP stand-in (Python, the
standard library) that answers `initialize`, `tools/list` and `tools/call` for `fleet_snapshot`
(`{instance_id, cursor, seats}` as `structuredContent` and as text) and `fleet_events` (an empty
page while its fixture file is unchanged; after a rewrite, an `upsert` per seat and a
`question_raised` per question, with the next cursor), and `marley.harness` names it. Two details
made it reliable: the fixture writes a time-relative value as `+N` and the stand-in adds its load
time, so countdowns read the same however long Marley takes to start; and the fixture is replaced
with `os.replace`, so the stand-in never reads half a file. #534's real-harness scenario stays the
way to prove the wire itself.

## L-claude-641-a-tasks-spawned-task-is-already-wrapped-for-the-shell-001
*category: code · topic: rerunning a Zed task from Marley · from: pipeline 641*

`Terminal::task().spawned_task` is the task after `prepare_task_for_spawn`: its command is the
shell and its arguments carry the command line, built with `build_no_quote`. Handing it to
`TerminalPanel::spawn_task` again wraps it a second time. To rerun a task from Marley, keep the
`SpawnInTerminal` the task provider received (`RoutedTerminals::spawn`'s, unprepared) and spawn
that, changing only what must change (`reveal: RevealStrategy::Never` to leave the focus alone).
Zed's own Rerun avoids this by re-resolving the task from its inventory.

## L-claude-641-tmux-starts-a-login-shell-which-reads-bash-profile-001
*category: e2e · topic: a scenario's sshd and remote shell · from: pipeline 641*

In a scenario whose own sshd stands in for a host, `SetEnv HOME=… TMUX_TMPDIR=…` in its config does
reach the session, but tmux starts a login shell, which reads `.bash_profile` (not `.bashrc`), and
sshd starts it in the user's passwd home. Without a `.bash_profile` in the run's HOME the remote
prompt is the system's default with the box's host name, in the user's real home folder. Write
`PS1` and a `cd` into the run's `.bash_profile`. Check after a first run that nothing was written
to the user's files (no `~/.bash_history` line, no tmux server under `/tmp/tmux-UID`).

## L-claude-641-a-reader-that-quits-early-fails-a-pipe-under-pipefail-001
*category: e2e · topic: checks in scenarios · from: pipeline 641*

`script/e2e.sh` runs scenarios under `set -euo pipefail`. A check written `grep pattern log | head
-1 | grep -q …` passed on the first run and failed on the next: once the log held more than one
matching line, `head` quit after one, the first `grep` took SIGPIPE, and pipefail failed the whole
pipe though the last `grep` matched. Use `grep -m1`, which stops by itself, and a final `grep >
/dev/null` rather than `grep -q`. The same class as L-582's cargo piped into `head`.

## L-claude-643-a-marley-view-on-a-settings-sub-page-001
*category: code · topic: the Settings window and Marley crates · from: pipeline 643*

A Settings-window sub-page that draws Marley state does not need the settings UI to know Marley:
`settings_ui::MarleyPageViews` (in `marley_page.rs`) maps a sub-page's name to an `AnyView` a
Marley crate registers at init, and the sub-page's `render` function draws it, or a muted line.
The view lives at app level; it observes the global it draws and notifies, and gpui redraws the
Settings window that drew it. `zed::OpenSettingsAt { path }` opens such a sub-page when its
`SubPageLink`'s `json_path` (which may name no real key, as `agent.skills` and
`marley.rusty.server` do) is the only item the path matches. A dropdown in such a view is a
`ui::DropdownMenu` over a `ContextMenu` kept with `window.use_keyed_state`, as `settings_ui`'s
private `EnumVariantDropdown` builds one.

## L-claude-643-set-u-in-a-command-substitution-passes-silently-001
*category: e2e · topic: checks in scenarios · from: pipeline 643*

The runner's `set -u` turns an unbound `$1` into an error, but inside `$(…)` the error only ends
the substitution: `for pid in $(logged_pids "$1")` with no argument looped over nothing, and a
check meant to find live processes passed on an empty list. Give helpers' optional arguments a
default (`"${1:-0}"`), take a list into a variable with `|| return 1`, and fail a check whose
list is empty when an empty list cannot be right.

## L-claude-643-zeds-dylint-lints-run-only-at-gate-21-001
*category: gate · topic: Zed's lints on the Marley crates · from: pipeline 643*

Clippy and rustdoc passed and gate:21 failed 50 minutes in: seven string literals turned into
`SharedString` with `"…".into()` in a view's `render`, which Zed's dylint library flags
(`shared_string_from_str_literal`, denied in each Marley crate's root). Clippy does not run
`tooling/lints`. In Marley code, build a literal `SharedString` with
`SharedString::new_static("…")` (a `Label::new("…")` taking `impl Into<SharedString>` is fine),
and keep the root's other denied lints in mind while writing: `async_block_without_await`
(use `futures::future::lazy` for a blocking closure on `background_spawn`),
`owned_string_into_shared`, `notify_in_render`, `entity_update_in_render`,
`blocking_io_on_foreground`, `map_lookup_then_insert`. To check before the gate:
`cargo dylint --all -- --all-targets -p <crate>` from the checkout, its output in a log.

## L-claude-644-a-sidebar-with-two-views-forwards-focus-into-the-shown-one-001
*category: code · topic: the rail and Zed's sidebar focus · from: pipeline 644*

Zed copies the sidebar's `Focusable` handle into every workspace of the window when the sidebar
opens (`MultiWorkspace::apply_open_sidebar` → `Workspace::set_sidebar_focus_handle`), and uses
that copy for its focus-the-sidebar actions. A sidebar that shows one of two views (the rail's
Projects and Brain, #644) keeps returning its own handle from `focus_handle` and forwards focus
given to that handle into the view that shows, with `cx.on_focus(&handle, window, …)`: a handle
returned per view goes stale on the next flip, while the forward also catches a click on the
sidebar's own header. `on_focus` fires for the handle itself, not for its descendants, so the
forward does not loop, and `on_focus_out` on the same handle does not fire when focus moves into
a child.

## L-claude-644-a-module-nested-in-marley-workbench-is-pub-for-two-lints-001
*category: code · topic: visibility lints in Marley crates · from: pipeline 644*

An item `pub(crate)` inside a private module nested in a module trips clippy's
`redundant_pub_crate` (nursery, denied), and the same item `pub` trips `unreachable_pub` (warn in
the Marley lint table, an error under `-D warnings`). A nested module whose items the crate uses
is declared `pub mod` inside a `pub` parent (`marley_workbench::rusty::brain`, #644; the crate
root's `pub mod rusty`), and its items stay `pub(crate)`. A public module's doc then may not link
to its `pub(crate)` items: gate:14 runs rustdoc without private items and `-D warnings`, so such
a link is `rustdoc::private_intra_doc_links`; name them in plain code.

## L-claude-644-the-rails-struct-sits-at-clippys-bool-and-line-caps-001
*category: code · topic: the rail · from: pipeline 644*

`Rail` holds three `bool`s, so a fourth trips clippy's `struct_excessive_bools`, and
`Rail::new` sits at the 100-line cap of `too_many_lines`. New rail state goes into a struct of
its own built by one constructor (`BrainSide::new`, #644), so the field list and `new` each grow
by one line; subscriptions made together go through one helper (`Rail::follow_focus`).

## L-claude-644-the-stand-in-models-rustys-main-and-its-watcher-001
*category: e2e · topic: the stand-in rusty-mcp · from: pipeline 644*

Rusty changes under Marley's plans: during #644 its main shipped TICKET-040 (the root's
`archive/` left out of `brain_tree`, writes into it refused) while the box still ran the old
binaries. The stand-in models Rusty as its main ships it, the Rusty Marley meets once Chad
reinstalls, and the planning notes say which commit; check `git log` in `rusty-v3` at promotion.
It also announces `list_changed` for any change to a file in its vault, as Rusty's watcher does,
so a scenario writes a page as another program would and needs no signal of its own.

## L-claude-645-zed-binds-ctrl-alt-shift-o-twice-001
*category: e2e · topic: keys a scenario binds · from: pipeline 645*

A scenario that binds a key of its own in the run's `keymap.json` checks the chord against Zed's
keymap first (`grep -n '"ctrl-alt-shift-x"' assets/keymaps/default-linux.json`, and the
`alt-ctrl-shift-x` spelling): Zed binds `alt-ctrl-shift-o` to `projects::OpenRemote` and
`ctrl-alt-shift-o` to `dev::ResetFrameOverlayStats`, and #645's `rusty::OpenPage` bound there never
ran, silently. `ctrl-alt-shift-y` and `ctrl-alt-shift-k` are free.

## L-claude-645-a-data-action-in-a-marley-crate-deserializes-by-hand-001
*category: code · topic: actions with fields in Marley crates · from: pipeline 645*

gpui's `#[derive(Action)]` adds a registration that clippy counts as an unsafe method, so a
derived `serde::Deserialize` on the same type trips `clippy::unsafe_derive_deserialize` (pedantic,
denied in the Marley crates; Zed's crates do not run it). A Marley action with fields
(`rusty::OpenPage`, #645) derives `Action` and `JsonSchema`, and deserializes through a private
fields struct with the derive and `deny_unknown_fields`: `impl Deserialize for OpenPage` reads the
fields and builds the action. The unit actions of `actions!` in a Marley crate each carry
`#[derive(Eq)]`, or clippy asks for `Eq`.

## L-claude-645-a-markdown-heading-scroll-needs-a-frame-after-the-parse-001
*category: code · topic: Zed's markdown crate in a Marley view · from: pipeline 645*

`MarkdownElement::scroll_handle` makes the element scroll the view's own `ScrollHandle` to a
heading, an autoscroll it applies while it paints: the new offset is drawn by the frame after.
A Marley view that scrolls to a heading on load (the Page tab's `[[page#heading]]`) asks for that
frame itself once `Markdown::is_parsing` is false
(F-claude-645-a-heading-link-scrolled-nothing-in-a-page-at-rest-001). The autoscroll brings the
heading into view with three lines' margin, not to the top.

## L-claude-646-the-stand-ins-call-log-sorts-argument-keys-001
*category: e2e · topic: checking the stand-in `rusty-mcp`'s call log · from: pipeline 646*

The stand-in logs each `tools/call` with `json.dumps(arguments, sort_keys=True)`, so a scenario's
`grep -F` over `$RUSTY_STAND_IN_STATE/calls` spells the arguments in sorted key order with Python's
separators: `"case_sensitive": true, "limit": 60, "query": "orbit", "regex": true`, not the order
the Rust `json!` wrote. A check that a query was not sent while typing greps the closing quote too
(`"query": "orbit"`), or a longer query that starts the same counts.

## L-claude-646-a-panel-that-follows-a-page-tab-needs-the-tabs-own-event-001
*category: code · topic: a dock panel following a center item · from: pipeline 646*

`workspace::Event::ActiveItemChanged` fires when another item becomes active, not when the active
item changes what it shows. A Page tab navigates inside itself (a link, Back, Forward), so a panel
that follows it subscribes to the tab too (`PageEvent::UpdateTab`) and keeps that subscription
with the tab's weak handle, replacing it only when another tab becomes active. The first look at
the active item waits for `cx.defer_in`: panels are made while the workspace is being updated.

## L-claude-647-a-run-of-background-batches-waits-on-the-frame-each-time-001
*category: code · topic: background work drawn as it goes · from: pipeline 647*

Work lent to the background executor and taken back with `this.update` after each slice waits, at
each return, for the window's thread to finish the frame it is drawing. In a debug build a busy
frame is tens of milliseconds, so slices of a few milliseconds of work are dominated by the waits
(F-claude-647-the-capped-graph-took-four-seconds-in-one-step-batches-001). Size a slice at a frame
or two of work (15 ms or so), and log the time worked beside the time taken, which tells the two
apart.

## L-claude-647-mul-add-and-hypot-are-library-calls-on-x86-64-001
*category: code · topic: float loops in Marley crates · from: pipeline 647*

The nursery lint `suboptimal_flops` asks for `mul_add` in place of `a * b + c`, and `imprecise_flops`
for `hypot`. Rust's own documentation of `mul_add` warns it can be slower than a multiply and an add
where the target has no FMA instruction, as Zed's baseline x86-64 builds do not, and `hypot` guards
against overflow a pair of squares does not need. Outside hot loops follow the lint; in a loop run
millions of times, write the products into named values first (`let across = dx * dx;`) and avoid
the square root where the arithmetic allows (a push of k²/d along a unit vector is the offset times
k²/d²). In #647 this was not the slow part (the batches were), so measure before blaming either.

## L-claude-647-gpui-reports-a-wheel-notch-as-three-lines-001
*category: code · topic: zooming on the wheel · from: pipeline 647*

On Wayland and X11 gpui turns one wheel notch into `ScrollDelta::Lines(3.0)` (`SCROLL_LINES`,
`gpui_linux/src/linux/platform.rs`), and the e2e harness's `scroll` sends whole notches, positive
for the wheel down. A zoom per line of 1.15 is 1.52 a notch; 1.05 a line gives Rusty's 1.16 a notch.
`scroll -N` is the wheel up, which zooms in.

## L-claude-648-a-tooltip-needs-the-pointer-to-move-onto-it-001
*category: e2e · topic: hovering under sway · from: pipeline 648*

A scenario's `pointer_to` to the place the pointer already rests sends no motion, so nothing new is
hovered and no tooltip opens, though a click there works (`648-06`'s first run). Before shooting a
tooltip, move the pointer off (`pointer_to` an empty spot, `settle 1`) and back.

## L-claude-648-a-stand-in-can-print-its-version-from-its-own-file-name-001
*category: e2e · topic: faking an agent's updates · from: pipeline 648*

To act out an agent updating itself, copy one stand-in under the version names
(`versions/2.1.287`, `versions/2.2.0`) and have it print `os.path.basename(os.path.realpath(
__file__))` for `--version`; a link the scenario moves with `ln -sfn`, as Claude Code's own
installer moves `~/.local/bin/claude`, is then a new install with a new identity (canonical path),
and the program keeps its name `claude` in the terminal's foreground.

## L-claude-649-an-app-tools-refusal-reason-never-names-the-tool-001
*category: mcp · topic: refusals from app tools · from: pipeline 649*

`marley_mcp::dispatch` turns an app's `Err(reason)` into `"<tool>: <reason>"` in the refusal, so a
reason that opens with the tool's name reads it twice. A program reading a refusal takes the
error result's `structuredContent.reason`; its `content` text is the whole JSON.

## L-claude-649-on-release-ends-only-when-no-strong-handle-is-left-001
*category: gpui · topic: waiting for a tab to close · from: pipeline 649*

`ItemHandle::on_release` fires when the item's entity is dropped, not when its tab closes: a task
that kept the `Box<dyn ItemHandle>` it awaited from `open_abs_path` keeps the item alive past the
close. Keep the returned `Subscription`, drop the handle at once (Zed's `--wait` in
`open_listener.rs` does the same), and defer anything that touches a pane or the workspace out of
the callback, which runs inside the pane's close.

## L-claude-650-a-codex-thread-resume-renames-its-client-001
*category: integration · topic: Codex's App Server · from: pipeline 650*

In Codex 0.155.1 to 0.158.0, `thread/resume` makes the caller a subscriber and also sets the
thread's client name to the caller's until the next `turn/start`, and Codex gates plugin installs
on `codex-tui`. A second client that resumes while a turn runs changes that turn. Resume only an
idle thread; the status is broadcast to every joined connection without a subscription.

## L-claude-650-proc-net-unix-shows-who-joined-a-socket-001
*category: linux · topic: seeing a client connect to a server you started · from: pipeline 650*

An accepted Unix socket carries the listening socket's path, so `/proc/net/unix` lists one line
per accepted connection under the path with state `03`, beside the listener's `01` with flags
`00010000`. Resolve a link first: Codex 0.158 makes the path a link to its real socket. Paths
with whitespace break the column split.

## L-claude-650-the-rail-sorts-rows-so-a-scenario-uses-the-palette-001
*category: e2e · topic: clicking the rail · from: pipeline 650*

The rail orders terminal rows by what each agent needs (#542) and puts the inbox above them, so a
row's place moves with every state change; the first run's clicks closed the wrong terminal.
Switch tabs with `pane: activate previous item` and close with `pane: close active item` from the
palette, and take positions only for elements whose place the shots show stable (the inbox's
first entry, a chip after the state settles).

## L-claude-650-e2e-runs-end-without-marleys-quit-hooks-001
*category: e2e · topic: checking cleanup at quit · from: pipeline 650*

`script/e2e.sh` ends a run by stopping its sway session, which takes Marley and its children
without `on_app_quit`. A check of what Marley cleans up at quit calls `quit_marley` first (with
the close guard turned off when an agent still waits), and reads the leftovers after it. The same
goes for a relaunch: the guard asks about a working agent too, so #652's quit stalled until
`marley.ask_before_ending_a_working_agent` was set false before it.

## L-claude-651-a-shared-stand-in-lives-in-a-fixture-001
*category: e2e · topic: one stand-in for several scenarios · from: pipeline 651*

When a later ticket extends a stand-in an earlier scenario wrote inline, move it into a fixture
under `script/e2e/` (as `browser-fixture.sh` and `codex-fixture.sh` are) that both scenarios
source, and run the earlier scenario again: its shots may change for the later ticket's reasons
(#650's approval shot now lists the request #651 shows), which its notes should say.

## L-claude-652-a-seat-label-named-session-id-starts-the-seat-over-001
*category: agents · topic: labels written onto a Claude Code seat · from: pipeline 652*

`claude_events::fold` reads a change of a seat's `session_id` label as a new Claude Code session
and builds the seat again from that frame, dropping every label it did not write. A label another
path puts on the same seat must not be called `session_id`: #652's reported session rides as
`report.session_id`, and the report's labels are written again after each frame's fold, since
the fold keeps only its own.

## L-claude-653-a-stand-in-language-server-by-its-binary-path-001
*category: e2e · topic: diagnostics in a scenario · from: pipeline 653*

A scenario gets a language server of its own by naming a stand-in program in
`lsp.<server>.binary.path` (here `lsp.rust-analyzer.binary.path`) with
`session.trust_all_worktrees: true` in the run's settings: `get_language_server_binary` returns
the path without a toolchain check, and the trust is what it waits on. A Python stand-in that
answers `initialize` with full sync and publishes diagnostics on `textDocument/didOpen` gives
the editor and every diagnostics reader a known error. To show a terminal and an editor in one
shot, open the file and run `pane: split and move right`; `workspace: activate pane left` and
`right` move the focus between them.

## L-claude-654-a-new-rusty-parameter-goes-beside-the-old-ones-001
*category: rusty · topic: calling a Rusty tool across Rusty versions · from: pipeline 654*

Rusty's tool parameters are serde structs without `deny_unknown_fields`, and the installed
`rusty-mcp` can be weeks older than Rusty's code (2026-09-17 against TICKET-041's 2026-10-04 on
this box). When a ticket lands a new parameter that supersedes old ones, send it beside them
(`brain_new_page { path, folder, name }`): the new Rusty lets the new one win, the old one never
sees it. Check `~/.local/bin/rusty-mcp`'s date against the ticket's commit before relying on the
new behaviour alone.

## L-claude-655-a-panels-set-active-runs-inside-the-workspaces-update-001
*category: gpui · topic: panel activation · from: pipeline 655*

The dock calls `Panel::set_active` while the workspace is being updated (opening, focusing or
toggling a dock), so a panel that reads its workspace entity there panics on the double lease.
Defer the work with `cx.defer_in(window, ..)`. The same holds for a workspace action handler that
toasts: call `show_toast` on the `&mut Workspace` it was given, not `WeakEntity::update` on the
same workspace.

## L-claude-656-an-inline-editor-takes-enter-and-escape-from-a-menu-context-001
*category: gpui · topic: an editor in place of a value · from: pipeline 656*

A one-line `Editor` put in place of a value needs no keymap of its own for Enter and Escape. Wrap
it in a `div().key_context("menu")` with `on_action` handlers for `menu::Confirm` and
`menu::Cancel`. The editor binds Escape to `editor::Cancel`, which propagates when there is
nothing to dismiss, and binds no Enter in single-line mode. Both keys fall through to Zed's
`menu` bindings, the way the project panel's rename gets them. Keep the cancel handler's
`cx.propagate()` for when no editor is open, so Escape still reaches the tab's parents.

## L-claude-656-a-scenario-drives-a-zed-menu-by-keys-and-restores-the-dock-001
*category: e2e · topic: menus and dock widths in a scenario · from: pipeline 656*

A `ContextMenu` opened by a click, from a right-click or a `PopoverMenu`, starts with its first
entry selected, so a scenario picks the third entry with Down, Down and Return rather than with a
measured click. Every panel in a dock keeps its own width: showing Zed's outline panel in the right
dock moves the centre's header buttons, and "close all docks" moves them again. Put the dock's
first panel back with its own `toggle focus` command twice (show, then focus back to the centre),
and the measured positions hold.

## L-claude-657-the-palette-runs-the-command-used-last-for-a-shared-prefix-001
*category: e2e · topic: running an action from a scenario · from: pipeline 657*

Zed's command palette ranks recently used commands first. Once a run has used
`rusty: open local graph`, typing `rusty: open graph` and pressing Enter runs the local one again,
since its name holds every letter typed. A scenario that runs two commands, one name inside the
other, binds a key to the action in the run's keymap. Check the key against the default keymap's
deeper contexts: `Pane` binds Ctrl+Alt+Shift+G, which wins over a `Workspace` binding;
Ctrl+Alt+Shift+Y has been free in #645, #647 and #657.

## L-claude-658-watch-the-rusty-globals-link-not-its-notifications-001
*category: gpui · topic: observing Marley's Rusty global · from: pipeline 658*

`rusty::Rusty` is notified for its connection, its settings, its server and its offer, and around
each announcement, so `observe_global::<Rusty>` runs far more often than the connection changes.
A view that should act when Rusty comes up or goes off keeps the state it last saw (off, down,
up) and acts only on a change, as the Page tab's `connected` and the Tasks tab's `Link` do; a
view that reads on every call reads behind the rules it means to keep. Reads cheap enough to
compare a key, like the Graph tab's `read_if_needed`, can be called each time.

## L-claude-658-hide-a-tab-in-a-scenario-by-its-panes-other-tab-001
*category: e2e · topic: putting a tab in the background · from: pipeline 658*

In the Marley layout a file opened from Zed's file finder does not come to the pane holding a
terminal and a Rusty tab, so the Rusty tab stays its pane's active item and counts as shown. To
hide a tab in a scenario, click another tab of the same pane, such as the project's terminal, and
click the tab again to show it.

## L-claude-659-a-listsubheader-grows-in-a-column-001
*category: gpui · topic: Zed's list headers outside a list · from: pipeline 659*

`ui::ListSubHeader` renders with `flex_1`. In a `List` or a row it only widens, but placed
straight in a `v_flex` column it also grows in height, sharing the column's spare space with
every other header. Wrap it in `div().flex_none()` when it heads a section of a plain column, as
the Decisions tab's `header()` does.

## L-claude-659-reach-a-settings-item-by-the-settings-search-001
*category: e2e · topic: clicking an item deep in the Settings window · from: pipeline 659*

The Marley settings page is long, and the place of an item near a section's end moved between
runs after a scroll. Type the item's name into the Settings window's search field first: the page
narrows to the section holding it, and a scroll to its end lands the same way every run. The
search field sat at (912, 59) in the sway scenarios' 1600 by 1000 window.

## L-claude-660-a-picker-inside-a-form-opens-at-the-modal-width-001
*category: gpui · topic: embedding Zed's picker in a Marley view · from: pipeline 660*

`Picker::uniform_list` opens at `DEFAULT_MODAL_WIDTH` (34 rem) whatever holds it, and a plain
picker's opening width is also its minimum. Inside a form or panel with padding, call
`initial_width` with the container's inner width, or its rows run past the container's edge.
An embedded picker's Escape and pick reach the container through its delegate's callbacks; defer
them (`cx.defer`), since they run inside the picker's own update.

## L-claude-660-zeds-toggle-group-reads-as-a-choice-only-outlined-001
*category: ui · topic: a choice among a few in a Marley form · from: pipeline 660*

`ui::ToggleButtonGroup` defaults to the transparent style, where the unchosen buttons are plain
words with no edge. Every Zed use passes `.style(ToggleButtonGroupStyle::Outlined)`; do the same.
To show none chosen, pass `selected_index` an index past the buttons: the group lights entry 0
by default.

## L-claude-660-a-missed-click-outside-a-zed-modal-closes-it-silently-001
*category: e2e · topic: clicking inside a modal in a scenario · from: pipeline 660*

A click outside Zed's modal layer closes the modal with no event a scenario sees, so a mis-measured
click on a form's button drops the form unsent and the next steps run against a different state.
Measure a modal's buttons from a shot of that exact form (its height changes with the fields it
shows), and `expect` on what the click should have sent right after it.

## L-claude-548-workers-ai-takes-the-model-in-the-body-and-wraps-the-answer-001
*category: api · topic: Cloudflare Workers AI's REST call · from: pipeline 548*

Cloudflare's model page for Jev shows the generic `POST
https://api.cloudflare.com/client/v4/accounts/{account}/ai/run` with `Authorization: Bearer
<token>` and the model in the body (`{"model": "typesafe/jev", "input": {...}}`), not the older
`/ai/run/@cf/<model>` path. The REST page says every answer comes as `{"result", "success",
"errors", "messages"}`: the model's own answer is `result`, and an error's words are
`errors[0].message` with its `code`. Read both pages (developers.cloudflare.com/ai/models/<vendor>/
<model>/ and /workers-ai/get-started/rest-api/) before assuming either shape; a summary of the model
page alone leaves the envelope out.

## L-claude-548-format-before-the-gate-not-during-it-001
*category: process · topic: the gate's receipt · from: pipeline 548*

`just gate-diff` records the tree it ran on, and its receipt fails when a file changes while it
runs. Run `cargo fmt` (or `rustfmt` on the files touched) before starting the gate; the Stop hook's
own rustfmt check catches a missed format after the fact, and the gate then needs a second run.

## L-claude-661-hide-a-feature-from-the-palette-with-zeds-filter-001
*category: gpui · topic: a setting that hides a feature's commands · from: pipeline 661*

Zed hides commands with `command_palette_hooks::CommandPaletteFilter::update_global`:
`hide_namespace` for a whole `actions!` namespace and `hide_action_types` for one action in a
shared namespace (a shown type overrides a hidden namespace). `update_global` does nothing until
`command_palette::init` has run, which Zed does before `initialize_workspace`; set it at init and
again on every settings change, only when the value changed. The settings window builds its pages
from `cx` (`settings_data(cx)`), so a page can leave items out by state and `rebuild_pages` them on
a change; run `update_matches` after it when a search may be open.

## L-claude-662-a-tooltip-that-names-a-state-needs-an-id-per-state-001
*category: gpui · topic: a toggle button's tooltip · from: pipeline 662*

`.tooltip(..)` builds its view once per hover and keeps it in the element's state under the
element's id; a re-render after a click keeps the words it was built with. When the words name the
state the click changes ("Add to Favourites" and "Remove from Favourites"), give each state its own
element id (`IconButton::new(if on { "x-on" } else { "x-off" }, ..)`): the new id drops the shown
tooltip, and the next hover builds one with the right words. A tooltip with fixed words needs
nothing.

## L-claude-663-the-stand-in-answers-a-slow-tool-on-its-own-thread-001
*category: testing · topic: the Rusty stand-in and Marley's ping · from: pipeline 663*

Marley pings Rusty every 5 s and counts a missed answer as a lost connection. The stand-in read
stdin one request at a time, so a tool that took longer (a 7 s fetch) would have held the ping
behind it and dropped the connection mid-scenario. Rusty answers requests side by side, so the
stand-in now runs `SLOW_TOOLS` on a thread of their own. Any stand-in tool that waits on something
outside it (the network, a sleep, a big folder) goes in that set.

## L-claude-663-actions-in-marley-crates-derive-eq-001
*category: clippy · topic: `actions!` and derive_partial_eq_without_eq · from: pipeline 663*

`gpui::actions!` derives `PartialEq` on unit structs, and the Marley crates' clippy denies
`derive_partial_eq_without_eq`. Put `#[derive(Eq)]` on each action inside the macro, as
`tasks_tab.rs` does; the module's visibility has nothing to do with it (the lint looks at the
struct, which the macro always makes `pub`).

## L-claude-664-toggle-button-group-needs-a-box-of-its-own-001
*category: gpui · topic: `ToggleButtonGroup` beside other children in a row · from: pipeline 664*

Zed's `ToggleButtonGroup` makes itself as wide as its parent. Alone in a column (#660's form) that
reads as intended; in an `h_flex` beside a `flex_1` field it takes the field's space. Put it in
`div().flex_none().w(rems(N))` and let the field grow. A `ContextMenu` behind a `DropdownMenu` opens
on the choice shown, so a scenario picks an entry with Home first, then Down.

## L-claude-665-a-task-runs-one-command-in-a-center-terminal-001
*category: gpui · topic: running a command in a terminal from a Marley tab · from: pipeline 665*

To run one command in a terminal tab from Marley code, build a `TaskTemplate` (the command and its
shell-quoted arguments, a `label` for the tab's title, `RevealTarget::Center`) and pass the resolved
task to `Workspace::schedule_resolved_task`, as `remote.rs` opens ssh. Marley's terminal routing
puts it in the center, the tab shows a finished mark, and no process starts in the Marley crate, so
gate:22's spawn sites stay as they are. Set `show_command: false`: Zed's summary line names the
command without its arguments, which reads as if the wrong thing ran.

## L-claude-665-check-a-new-scenario-before-the-final-gate-001
*category: testing · topic: a scenario joins the gate's scope only at Complete · from: pipelines 664 and 665*

The Code phase's gate runs before the scenario exists, so a scenario's first gate is the final
one, and its shellcheck and typos findings cost a whole second run. Run `shellcheck -x
script/e2e/<N>-<slug>.sh` and `typos script/e2e/<N>-<slug>.sh` as soon as the scenario is written.
A fixture that needs a literal backtick or `$` goes in a quoted heredoc (`<<'EOF'`), which
shellcheck reads as meant.

## L-claude-666-a-field-per-row-from-use-keyed-state-001
*category: gpui · topic: editable rows in a view that keeps no state · from: pipeline 666*

A view drawn from a global (the Rusty's Server page) can still give each row an editor:
`window.use_keyed_state(id, cx, |window, cx| Editor::single_line(..))`, with the id built from the
row's key and a hash of the value it shows (`ElementId::NamedInteger`). A new value read back
changes the id and so makes a fresh editor holding it; an edit in progress lives until then. A
row whose value reads back the same (a credential Rusty masks every time) keeps its editor, so
clear it yourself after the write. In the Settings window, Enter with the focus off a field takes
the window back a page: a scenario must click inside the field.

## L-claude-667-zeds-mcp-client-logs-whole-messages-at-trace-001
*category: security · topic: what an MCP server's messages leave in Zed's log · from: pipeline 667*

Zed's `context_server` client writes every message it sends and receives whole to its log at trace
level (`recv:`, `outgoing message:`), the stdio transport writes the outgoing ones again, and a
message that parses as nothing is written at error level. Any server whose tools take or return a
secret (a PIN, a token, a key) needs `context_server::client::log_messages_by_size(<server name>)`
before it starts. Prove it with a run at `ZED_LOG=info,context_server=trace`: the server's lines
read `… bytes`, and a grep for the secrets the run used finds nothing.

## L-claude-668-a-scenario-can-stand-in-for-the-users-settings-001
*category: testing · topic: checking what the e2e harness copies · from: pipeline 668*

`script/e2e.sh` sources the scenario at top level before it copies `$config/settings.json` into
the run's profile, so a scenario that sets `config` at top level (to a folder under `SHOT_DIR`)
hands the harness a settings file of its own and never reads the user's. Variables it exports
there reach the harness's later lines too. That is how to check the copy itself: write the
settings a user might hold, export the variables a user might have, and let `setup` read
`$E2E_PROFILE/config/settings.json` and its own environment, which Marley inherits.

## L-claude-669-a-fourth-bool-in-marley-settings-is-a-two-variant-enum-001
*category: rust · topic: adding a switch to MarleySettings · from: pipeline 669*

`MarleySettings` holds three `bool` fields, clippy's `struct_excessive_bools` limit, and
`from_settings` sits at the `too_many_lines` limit. A new on/off setting is a two-variant enum
beside `PromptEditor` and `EmbeddedHarness`, built by a `from_content(marley)` that fits on one
line of `from_settings`, as `CodexAppServer::from_content` does; a plain `bool` with
`.and_then(..).unwrap_or(..)` fails both lints at once.

## L-claude-670-a-restore-scenario-relaunches-with-no-path-001
*category: testing · topic: checking what a window keeps across a restart · from: pipeline 670 (again after 601)*

`launch_marley` passes the scenario's `open_path`, and Zed answers a start with a path as an open
request: a new window, none of the last session's saved state. A scenario that checks what a
window keeps across a restart (the rail's width, its closed state, its folds, its groups) calls
`open_path ""` between `quit_marley` and `launch_marley`, as a start from the menu does. #670's
first run forgot it and showed a fold lost that was kept.

## L-claude-671-the-quiet-timer-makes-typing-into-an-agent-read-as-working-001
*category: product · topic: an agent's status from its output · from: pipeline 671*

An agent terminal without #519's hook events gets its status from `last_output`, which every byte
the PTY writes refreshes, and that includes the echo of what the user types. So typing into Claude
Code reads as Working for a moment, and anything that acts on Working (the attention order, a
"working" count, a notification) fires on the user's own keystrokes. A feature that reacts to an
agent's status should take it from the agent's events where they exist, and treat output-recency
as a weak signal that a typing user produces too.

## L-claude-672-the-window-draws-a-line-under-the-title-bar-001
*category: ui · topic: lining the rail up with the main column · from: pipeline 672*

The rail's header was `platform_title_bar_height` with its `border_b_1` inside that height, so its
line fell on the last row of the title bar's height, while the main column's line sits one row
lower: the window draws a 1px line under the title bar, outside its height. Everything under the
header was one pixel high as a result. Matching heights is not enough to line things up: read the
border rows from a shot on both sides (`magick shot.png -crop 1x120+X+0 txt:-` and grep the border
colour) and compare the row numbers. #672's scenario keeps that read as a check (`lines_meet`).

## L-claude-673-a-panel-whose-enabled-turns-false-stays-open-001
*category: gpui · topic: hiding a dock panel by a setting · from: pipeline 673*

A Zed dock panel whose `enabled` and `icon` turn false loses its status bar button at once, but a
dock that is showing it keeps showing it, now drawing whatever the panel renders while hidden.
#673's panel stayed open and empty when `marley.rail_containers` went off. The fix is the one the
Knowledge panel already had for Rusty going off: render nothing while hidden and close the dock
once, deferred past the panel's own update (`close_while_hidden`), resetting when the switch comes
back on. Any panel gated by a setting needs both halves; a scenario should turn the switch off with
the panel open, not only with it closed.

## L-claude-674-a-tab-change-outside-the-active-pane-reaches-no-workspace-event-001
*category: zed · topic: following every tab of a workspace · from: pipeline 674*

`Workspace::handle_pane_event` turns a pane's `ChangeItemTitle` (which an item's `UpdateTab` and
its dirty state ride on) into `workspace::Event::ActiveItemChanged` only when that pane is the
active one, and `ItemPinned` into nothing. A view that follows every tab of a workspace through the
workspace's events alone misses a rename or an edit in a split that is not focused. Subscribe to
each pane (`Workspace::panes`, resubscribed as panes come and go) for `pane::Event`, or to each
item's `ItemEvent`; the rail does the first (`follow_panes`).

## L-claude-675-route-a-feature-s-tabs-in-its-openers-not-its-callers-001
*category: rust · topic: sending every tab of a feature to one place · from: pipeline 675*

Rusty's tabs open from the header, the palette, the Brain view's tree, the page picker, the
Knowledge panel, capture and page links. Moving all of them to the Rusty group took one helper
(`rusty::in_rusty_group`) called from each tab module's `open_later` and the page module's, plus a
one-line change to each palette action so it goes through its `open_later` rather than its `open`.
The callers changed not at all. When a feature's tabs must all land somewhere new, route in the
few openers every caller already funnels through, and check the palette actions, which tend to
call the inner `open` directly.

## L-claude-677-a-shown-tooltip-keeps-its-title-until-the-pointer-leaves-001
*category: gpui · topic: a button whose tooltip changes with its state · from: pipeline 677*

gpui builds a tooltip's view when the hover starts and keeps that view while the pointer stays,
so a button whose click changes its own tooltip (Rich Input to Hide Rich Input) still shows the
old words until the pointer leaves and comes back; its `toggle_state` shows at once. A visual
check of such a tooltip moves the pointer off the button and back before the shot, and the state a
click acts on is read when the click lands, not captured when the button was drawn.

## L-claude-678-a-scripted-cut-between-two-markers-takes-everything-between-001
*category: process · topic: removing code with a script · from: pipeline 678*

Removing `render_new_page` with a cut "from its doc comment to `fn render_add_project(`" also took
five functions that sat between them in the file (`render_blocks`, `render_harness`, `render_row`,
`render_rows`, `render_filter`); the file order was not the order the helpers were written in. A
scripted cut ends at the removed function's own closing brace, and `git diff | grep '^-.*fn '`
right after it lists every function the edit removed, which caught this one before the build.

## L-claude-680-a-scenario-s-terminal-run-runs-a-program-not-a-shell-001
*category: process · topic: commands a scenario runs through terminal_run · from: pipeline 680*

`bash pages.sh` sat on #680's card for 25 seconds and came back refused: `bash`, `sh`, `zsh`,
`fish` and `pwsh` open the default `marley.agent_command_denylist`, which asks the user whatever
`agent_commands_outside_lists` says, and the copied profile's own lists apply on top. Pinning the
outside-lists setting changed nothing, since the denylist is checked first. A scenario that wants
a command to run at once writes its fixture as an executable with a shebang and runs `./name.sh`,
which is on neither list, and pins `marley.agent_commands_outside_lists` to `"run"` in `setup`
(`profile_setting`) in case the user's profile asks.
