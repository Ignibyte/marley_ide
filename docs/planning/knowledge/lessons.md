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
