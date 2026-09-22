# Changelog

All notable changes to **Marley** are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Marley is pre‑1.0 and
milestone‑tracked, so versions are placeholders until the first release. A
`CHANGELOG.md` entry is **mandatory** for every change that touches Rust source —
`enforce-changelog.sh` blocks a commit without one (CONSTITUTION §21).

## [Unreleased]

### Added

- **Run blocks — identity, rerun, and Terminal-F8 jump-to-failure**
  (TICKET-435, M33 — the fusion wedge closes; React-first). A Block born from
  the #434 gutter ▶ now CARRIES its identity: `Block.run_tag`, staged as a
  `(tag, command)` pair on the session in the same sync region as a successful
  `write_command` and MATCH-BOUND by the Preexec reporting exactly that
  command (the staged-prompt correlation, hardened — a tag can only ever label
  the command it was minted for; stale/failed-write/racing cases fail SAFE to
  a plain block). The header shows a muted ▶ before the command (both header
  render sites, POC-parity 6px gap). The ↻/menu rerun is identity-aware: a
  decodable tag re-mints through `decode_run_tag → run_command` (recorded in
  history, a FRESH tag staged — the rerun's block is itself a run block, in
  its own pane); plain blocks keep the byte-identical #175 path. Terminal-F8
  (`run-jump-failure` — the #290 convention on the other surface; the spawn's
  `jump_to_pane` means focus is already there) jumps the editor to the LATEST
  failed run block's refs in OUTPUT order (`links::file_refs_ordered` — the
  one scan `file_refs` now projects from), cycling + wrapping per press with
  a verified landing (`open_and_place_caret`) and a NavStack push per jump;
  a same-tag green rerun RETIRES the identity (supersession-aware
  `latest_failed_run_block`), so F8 fails closed on green/refless/no-target.
  Bare F8 yields to the PTY while a program owns input (alt-screen or a
  running command — R40 outranks the keymap; mc keeps its F8).

- **Runnables — the gutter ▶ spawns a command Block** (TICKET-434, M33 — Phase C
  opens; React-first). A new pure `marley_syntax::runnables_in` node API (the 6th:
  an iterative TreeCursor walk, spike-pinned on tree-sitter-rust 0.24.2) marks the
  runnable lines of a Rust file — `#[test]`-family fns (stacked attributes,
  `#[tokio::test]`; the path IS/ends-with `test`, so `#[cfg(test)]` self-excludes)
  with their full in-file mod-chain path, plus the top-level `fn main` — module-
  scope gated (impl/trait bodies and fn-in-fn never mark; grammar-error
  `$metavariable` names never reach the mint). The editor gutter reserves a ▶
  mini-cell (muted → accent hover, its OWN stop-propagation hitbox beside the fold
  zone) on exactly those rows; the click resolves the LOWEST IDLE workspace pane
  (the #292 walk, now the ONE shared spelling `idle_workspace_panes` — the #40
  never-inject-mid-command guard; none idle → flash-and-bail, zero side effects),
  mints the honest D6 command (`cargo test <mod::chain::name>` substring filter —
  no `--exact`, the limits recorded; plain `cargo run`), and spawns it AS TYPED
  INPUT (`history.record` + `write_command` + the R39 viewport re-anchor) — shell
  integration frames it as a first-class Block with the live status pill, and
  `jump_to_pane` reveals + focuses it. Where Zed's runnables pipeline degrades
  into a terminal tab with a summary line, Marley routes the same discovery into
  the shipped block model — the fusion wedge's premise, live. Inspect hardened the
  landing: both row-prefix geometry mirrors (the h-scroll thumb origin + the
  sticky-header band) re-composed in lockstep (a new PR records the class), and
  the coverage red was fixed by RE-EXPRESSING the one grammatically-unreachable
  arm rather than excluding it. Proof: 13 units (the spike fixtures verbatim +
  the inspect pins) + 2 drives (the gated row set; spawn-records-history +
  reveal + the busy-pane flash with byte-identical state) + a LIVE drive (▶ on
  exactly the right rows of a real file; the click spawned `cargo test tests::…`
  as a framed Block streaming real `test result: ok` lines). Suite 2278/2278;
  GATE GREEN [diff] 15/15.
- **Failed command Blocks feed the problems panel — the terminal lane's producer
  fills the #310 two-producer seam** (TICKET-433, M33 — the Phase-C wedge's first
  fusion thread; React-first for the panel's source glyph). Failed Blocks' file:line
  refs — the shipped #212/#291 scanners re-aimed from one-open-file to ANY file
  (`links::file_refs`) — enter `problem_rows`' long-empty `terminal` parameter,
  scanned WORKSPACE-WIDE (all projects × all terminal grids × all panes; each ref
  resolved against its OWNING project's canonical root) with a per-pane SUPERSESSION
  fold (latest completed block per (command, pwd) governs; a passing rerun silently
  retires its failure's rows). The refresh gates grew a terminal lane:
  `terminal_blocks` now mints a monotone per-session `block_epoch` (born + finished,
  through the one hook-machine choke — the `publish_epoch` twin), and BOTH surfaces'
  fingerprints became tuple PAIRS (LSP, terminal) so either lane moving re-derives
  and the lanes can never cancel. ⌘⇧M rows carry the ❯ source glyph (built in the
  POC first); the selection-keep identity grew `source` (two producers can collide
  at (path, line, 0, Error)); terminal rows jump and materialize into the #430
  multibuffer through the existing machinery unchanged. Everything derives
  statelessly — closing the failed block's tab drops its rows at the next derive.
  Proof: 11 units (epoch both hook paths + child-exit double-finish guard;
  extractor; fold table incl. the pre-prompt (cmd, None) group; the 5-tuple
  collision) + 3 drives (the PR-1345 multi-project multi-tab corollary with the
  exact-landing-line jump; through-the-gate supersession + identity keep +
  tab-close drop; the out-of-root absolute ref failing closed) + a LIVE no-LSP
  drive (real failed commands → "2 problems", both ❯ rows, `lsp: failed` in the
  status bar). Suite 2265/2265 → 2269; GATE GREEN [diff] 15/15.
- **The multibuffer's input goes complete — click-column caret, ⌘V paste, IME
  composition, and a fail-closed key ladder** (TICKET-432, M33 — the #428 recorded v1
  seams close; React-first for the paste leg). A recorded per-frame mb geometry
  (`MbFrameGeom`, the editor-geom twin) powers precise CLICK-COLUMN caret placement
  (float col through the editor's own `LineLayout` inversion — tab-aware, wide-glyph
  midpoint, clamped; the caret bar paints on the same map, exact round-trip), ⌘V
  PASTE through the one insert mechanism (a multi-line clip is ONE edit; the window
  grows exactly like Enter — one journal entry, one ⌘Z; built + captured in the POC
  first, parity pixel-pair verbatim on the standing token families), and full IME
  COMPOSITION on the mb surface (every `EntityInputHandler` method gained its mb arm;
  read arms answer from the caret anchor + `mb_marked`, never the file's live
  selection set; the candidate window anchors from the recorded geometry). The
  inspect pass hardened the whole input boundary — its four-critic sweep found and
  fixed three MEDIUMs: ⌘⇧V leaked past a `!shift` gate into the HIDDEN workspace
  terminal, plain printables had double-delivered into that same hidden prompt/PTY
  since #428 (an mb plain-key claim — the #251 editor twin — closes the lane), and
  a live composition survived tab switches/caret gestures at stale offsets (caret
  and composition now die together at every seam). Proven by 7 new headless drives
  (real keystrokes, real trait calls, hidden-prompt purity asserts) + a live drive
  (mid-word click → bar at that column; two-line paste grew the window on real
  pixels; ⌥E e composed one é through the real dead-key path). Suite 2251/2251;
  GATE GREEN [diff] 15/15.
- **The problems panel grows its editable form — diagnostics materialize as a multibuffer you
  fix in place** (TICKET-430, M32 — the B-c chain's closing step 6, React-first). ⌘⏎ in the
  ⌘⇧M panel (hint: "⌘⏎ edit as multibuffer") materializes the workspace's aggregated LSP
  diagnostics as the editable #427/#428 surface: severity-first file groups, each diagnostic's
  glyph + message on a BAND slot directly above its excerpt line (`Row::Note` — the model's
  slot list is now materialized, `slots: Vec<Row>`, replacing the header+lines prefix sums),
  true line numbers, washes, and a problems footer ("N problems in M files (+K more)"). Full
  #428 editing rides unchanged (caret, write-through, journal, touched-consent ⌘S); each file
  pins/births under ITS OWN project root (`root_of_mb_path` — the re-verify critic's
  split-brain find). The surface is a SINGLETON per project; re-materialize replaces in place.
  Refresh is QUIESCENT: the pump re-materializes when the workspace PUBLISH EPOCH moves (a
  monotone per-publish counter — the live drive proved the row-sum fingerprint swallows
  "one fixed + one new") and NO mb target is dirty — typing holds the snapshot, ⌘S releases
  it (and arms as belt); all-resolved leaves "All problems resolved" with the tab open.
  Guards fail CLOSED (empty set → flash, panel stays). Proven live against a real
  rust-analyzer: materialize → fix in place → save → the fixed pair left the surface (the
  collision case itself) → all-resolved.
- **⌘⇧F phase 2 — the replace row, regex mode, and one-gesture Replace All** (TICKET-429,
  M32 — the B-c chain's step 5, React-first). The project-search overlay grows a REPLACE row
  (Tab hops query↔replace; the focused field carries the ▏ caret mark; replace edits never
  re-park the walk) and a REGEX mode (⌘⌥R toggles the passive ".*" chip; the request compiles
  ONCE — `editor::compile_find`/`find_all_compiled`, the hoisted-compile fork of the find
  bar's engine — and an invalid pattern renders "invalid pattern" inert, no walk spawned).
  ⌘⌥⏎ is Replace All: per file — live buffers and disk-only files alike (not-open files
  birth through the same lazy-target recipe) — the literal arm re-finds case-folded and the
  regex arm re-matches with `$1` capture templates, each buffer bracketing its OWN undo
  group; every touched file joins the ⌘S consent set and the journal, and ONE batch entry
  makes the whole gesture a single ⌘Z (batch revert) / ⌘⇧Z (batch re-apply) across every
  file. Guards fail CLOSED: empty/invalid query, a still-streaming walk ("Search still
  running — try again"), and a materialize that did not open flash-and-bail with zero edits
  (F-claude-429-a). The flash reports "N replaced in M files (· K skipped)"; empty regex
  matches are filtered before edits (walk and apply agree). The results multibuffer stays
  the editable #428 surface underneath.
- **The multibuffer becomes a real editor — excerpt edits write through to the real Buffers**
  (TICKET-428, M32 — the B-c chain's step 4, React-first). Clicking an excerpt line places a
  CARET (line end v1; ←/→ walk it); typing/IME (its own paint-scoped `handle_input` canvas —
  the input seam is per-surface), ⌫/⌦ (pairing-aware via the editor's own key fn), and Enter
  (newline; the window GROWS) route through the ONE insert mechanism into the file's ONE
  registry Buffer — instantly visible in the file's own tab. Excerpt windows are #269 ANCHOR
  pairs resolved per frame (memo (nonce, version); a reload re-mints on epoch mismatch);
  edits above shift rows, interior joins shrink, and the window's own edges are pinned:
  deletes never eat text beyond the excerpt. The buffer's live selection is saved/restored
  around every mb edit (genuinely transient). ⌘Z/⌘⇧Z route through a cross-excerpt journal —
  one entry per undo GROUP (coalesce-aware retain+push), popped only when the target's
  `undo_depth` still matches (tab-side interleave drains, never reverts foreign history).
  ⌘S sweeps TOUCHED ∩ dirty only (`save_editor_by_id` background consent; a changed-on-disk
  file HOLDS with the #275 net; the outcome flashes "N saved · M held"). Header bands carry
  the dirty dot; already-open files pin a registry view at materialize, not-open files birth
  lazily at first caret, and every target releases at close. Replace-all + the results form
  land at #429; the problems form at #430.
- **The multibuffer — ⌘⇧F results materialize as ONE stitched, jumpable surface** (TICKET-427,
  M32 — the B-c chain's step 3, React-first). ⌘⏎ in the project-search overlay (hint: "⌘⏎ open
  as multibuffer") builds a read-only SNAPSHOT: per-file excerpt groups (each match line ± 2
  context lines, overlapping/adjacent windows merged as half-open runs) under header bands
  (repo-relative path + in-range match count, "+" when the per-file cap cut), TRUE file line
  numbers, hand-lexer syntax colors, a full-row wash on every match line, hairlines between
  disjoint windows, and the overlay's own footer summary (+K more / truncated / skipped)
  scrolling with the content. ↑/↓ move the selection over lines (headers skipped, ends
  clamped); Enter/click jumps to the file:match (Utf32 caret through the shared `jump_to_match`
  recipe) while the tab stays open — the materialization-time editor loc is pushed ONCE as the
  NavStack origin. Text sources live dirty buffers first (`editors_under`), else disk through
  the viewer guard chain (lossy decode + post-read TOCTOU re-check, the worker's own posture);
  a per-file failure skips with a flash, and an all-fail set never opens a tab (the overlay
  stays). The pure model (`multibuffer.rs`, cov/MSI 100) owns the merge math, the prefix-summed
  slot→Header|Line row model, counts, and footer; the tab is a #403-checklist Content kind —
  always-insert (the registry insert's first view IS the tab's one view), dropped-on-last-close,
  and NOT persisted (the writer's first dropping arm re-points `active_tab` at a survivor).
  Editing/live re-sync land at #428; replace-all + the results form at #429; folding excerpts
  into the #425 DisplayMap stays the chain's deferred unification prize.
- **Soft wrap — a too-wide line renders as stacked display rows, and the h-scroll era ends where
  wrap begins** (TICKET-426, M32 — the B-c chain's step 2, React-first). `editor.soft_wrap`
  (default OFF; "Toggle Soft Wrap" in the palette, live + write-through) turns the #425 display
  map's second layer on: `wrap.rs` computes break points in display CELLS over the one
  phantom-aware `LineLayout` (gpui LineWrapper's RULES — word-boundary candidates, the CJK
  any-break clause, hard-break fallback, capped continuation indent with an 8-cell MIN_TAIL —
  re-expressed as pure grid arithmetic; Apache-side read, no machinery taken), and the facade
  composes folds ∘ wrap behind the same five methods (`locate`, `slot_at`,
  `viewport_offset_at`). The rim renders one slot per SEGMENT — clipped-and-re-based
  syntax/selection/find/bracket/squiggle ranges, first-row-only gutter numbering, per-segment
  carets and clicks, the ⋯ fold marker riding the last segment — ↑/↓ walk display rows with the
  goal column preserved (`move_all_vertical_by`, the injected-step generic the classic vertical
  now shares), overlay cards + IME anchor at the caret's segment, and the whole #336 h-scroll
  set goes structurally inert while ON (scroll pinned, thumb hidden, wheel/follow no-ops). OFF
  is byte-identical — pinned by the suite, the zero-width probes, and the inspect's own
  regression catch (the EOL squiggle widening a naive segment clip would have dropped). Inspect
  earned its keep: three break-rule deviations from the adopted reference, the tab-unsound
  pre-filter (the #336 stranded-tail class, reintroduced and killed), and the phantom-blind
  column class (the F-#352 shape on the column axis) all died pre-commit; two proven-equivalent
  mutants were re-expressed with their proofs inline (AD-claude-305 discipline). Validated
  end-to-end: 24 pure tests + 2 real-keystroke drives + a live-app palette drive with
  pixel-sampled captures, and the React POC pair (the design source) matching 1:1.
  GATE GREEN [diff] 15/15.

- **The display-map foundation — every buffer-row↔display-row conversion now routes through ONE
  typed facade** (TICKET-425, M32 — the B-c chain's step 1; byte-identical UI by construction and
  by proof). `crates/marley_app/src/display_map.rs` introduces `DisplayMap` — folds-only today,
  delegating to the shipped `FoldProjection` unchanged — and `marley_text_offsets` grows the two
  ROW spaces, `BufferRow` and `DisplayRow` (the `uniform_list` slot), so a cross-space mix (the
  F-#352 geometry-freeze class) is now a COMPILE error, trybuild-pinned. The #305 rim discipline
  survives verbatim (one unwrap at the render rim; every interior site keeps buffer-row `usize`),
  `EditorFrameGeom.first/last` and `HoverCard.first_row` are typed slots (the latter's domain had
  been prose-only — the compiler found it, plus one more site the seam sweep missed), and the
  #352 memo moved intact into the `display_map()` builder. Proof of byte-identity: the facade
  property-equals the direct projection over discriminating fixtures (nested/adjacent/EOF/
  header-0/out-of-order/one-row), the full 2174-test suite passes with zero behavioral edits to
  existing tests, and a live-app drive shows the editor painting normally through the new path.
  Why now: #426 soft wrap breaks row↔line identity and #427+ excerpts break row↔buffer identity —
  both now insert a layer HERE instead of re-touching 16 call sites. GATE GREEN [diff] 15/15.

- **The `render_to_image` question is answered with numbers: WAIT, with the risk pre-cleared**
  (TICKET-424, a timeboxed spike — docs only, nothing ships). The Zed tree split gpui into an
  unpublished crate family and the offscreen-pixel mechanism spans ~1,300–1,500 adaptation lines
  across ≥8 files and two crate layouts — a fork or a git-rev family, each with a `deny.toml`
  amendment and double-migration risk, for a capability nothing currently blocks on (crates.io
  has been silent since 0.2.2, Oct 2025 — the watch: `max_version > 0.2.2`). The payoff side is
  now PROVEN, though: three probes (offscreen Metal render+readback; CoreText family
  enumeration; glyph rasterization) ran byte-identical in the GUI session and over ssh with no
  WindowServer — so when a release lands, `HeadlessAppContext` + injected `MacTextSystem` +
  `current_headless_renderer` gives a true-headless pixel lane (and could take the #344/#361
  font drives headless too). The adoption recipe + traps (the `::new`-installs-no-renderer
  constructor, the CosmicTextSystem doc example, the in-session-only `VisualTestAppContext`
  twin) live in the closed ticket's decision memo. #271 stays Deliberate, now pre-de-risked.

- **The FULL mutation audit is GREEN — measured, not assumed — and gate:5 grew honest lanes**
  (TICKET-407, the audit-debt closeout). The verdict: **5,834/5,834 mutants covered, 4,832
  caught / 0 missed, MSI 100.0%** — `GATE GREEN [full]`, the first FULL green since the debt was
  declared at #402. The 54 timeouts are all classified (39 genuine hangs — the no-progress-loop
  class; 15 caught-by-assert #345 mislabels; ZERO slow-but-passing false kills), so the standing
  Timeout=caught convention holds with evidence, and the remedy tree's `.cargo/mutants.toml`
  last resort died unused. Mechanically, `mutation_g` was re-plumbed for the 2026-08-13 policy:
  DIFF stays local; FULL's local sweep is banned-by-default (`MUT_FULL_LOCAL=1` opt-in, with a
  `NEXTEST_TEST_THREADS=2` bomb cap); FULL's default is now an IMPORT lane over the dev-box
  measurement, with artifact-derived provenance belts — literal-hex sha, HEAD equality, a
  `MEASURED_SHA` sidecar, an mtime freshness check, a foreign-repo probe, and SET-EQUALITY
  between the outcomes and the current tree's own full enumeration (which is what forced the
  three-generation merge to be exact: gen0 credit + gen1 sweep + gen2 retest = 5,834, no gaps,
  no stale rows). The receipt fingerprint also grew two gate-defining files it had been missing:
  `rust-toolchain.toml` and `.config/nextest.toml`. REQ-002's Replace-One contract was
  sabotage-smoked live (flip → RED → revert → GREEN); inspect's two critics found and closed
  two real import-lane false-green holes before they ever shipped.

- **The dev-box lane's blind spot is dead — every test runs on both platforms, no exclusions**
  (TICKET-423). The box excluded four tests via `-E` and forgave two more with retries; a mutant
  whose only killer sat there read MISSED. All six (plus two more the no-exclusion run exposed)
  are fixed at the source, and two of the "test problems" were PRODUCT bugs in `marley_terminal`:
  on Linux the master's EIO beats the SIGCHLD byte, so the pump erred `Disconnected` with the
  reap latch unset (now: a bounded non-blocking `EIO_EXIT_GRACE` rides it out — the alacritty
  event-loop precedent, adopted and restructured — with the pure `eio_grace_expired` boundary
  unit-pinned), and Linux masters ACCEPT post-close writes, so errno could never carry the
  post-exit write contract (now: the session latches `child_exited` and `write_bytes` refuses by
  STATE, uniformly). The twin-alias fixtures build their OWN symlink (`alias_twin_roots`) instead
  of riding macOS's `/var` alias; the trybuild "platform variance" was rustc 1.94-vs-1.96 skew —
  killed forever by `rust-toolchain.toml` (1.96.0 + components, both machines verified on the
  same compiler hash); every real-PTY wait is now a wall-clock deadline (15 s ceilings,
  early-exit — the fixed-budget forms timed out under lane load). Receipts: box
  `2169/2169, 0 exclusions` (WARM rerun, `423-verify.log`); local gate GREEN [diff] 15/15 after
  two round-1 reds (a private-intra-doc link; the `<`→`<=` boundary mutant) were fixed at
  source. Rider for the lane: drop the `-E` filter + `NEXTEST_RETRIES=1` from the box runner —
  they are dead weight now (runner is owned outside this repo).

- **The #344/#361 font-policy boot paths are finally proven end to end** (TICKET-365 — the
  headed lane's first stdout-report drives). `#[gpui::test]` can never see a real font
  (NoopTextSystem resolves nothing and fakes equal-width advances), so since #361 the RESOLVABLE
  arms rode pure-unit proof plus trust in two `mutants::skip` probes. Now
  `MARLEY_FONT_POLICY_SELFTEST=1` makes the real binary print ONE tab-delimited post-boot line
  (resolved mono family + flash text, both tab/newline-stripped at the printer) and quit — and
  two `#[ignore]` drives (`tests/headed_fonts.rs`) spawn it on the live WindowServer session
  with a tempdir-hermetic `HOME`/`TMPDIR`/cwd, asserting Monaco APPLIES silently and Helvetica
  falls back to Menlo with the content-pinned "not monospace" flash. Boot + stdout only — no
  Accessibility, no Screen Recording, no activation (the armed path skips `cx.activate`). The
  inspect-phase sabotage smoke (always-false `font_is_monospace`) flips the Monaco drive RED —
  the end-to-end false-warn regression the pure units structurally cannot catch, now caught.
  Critics: zero HIGH/MED; eight LOW hardenings applied (framing-safe printer, hermetic spawn,
  stdout-first crash diagnostics). Gate GREEN [diff], 15/15.

- **The full-gate mutation audit's 16 survivors are dead** (TICKET-407, gate:5 debt — the first
  COMPLETE 5,830-mutant enumeration: 4,758 caught / 16 missed / 54 timeout / 1,002 unviable,
  finished on the dev-box lane after the 16 GB Mac was ruled out). Six untested clusters each
  gained a distinguishing behavior test: `mouse_report`'s modifier bits on release/wheel/
  X10-release (byte-exact vectors — the old matrices carried mods only on press/drag),
  `param_range`'s offset-pair boundaries (len≠2 junk / empty range / end==total),
  ime branch-2's no-op guard (insert at a collapsed range AND an empty-text delete),
  `move_block_edit`'s up-swap with unequal line lengths (both `b0 - 1` terms pinned),
  `default_opener`'s never-asserted Linux arm, and `probe_web_origin`'s EXCLUSIVE 1024
  status-line cap (held-open loopback peer). `selected_utf16`'s `a != caret` match guard was
  mutation-EQUIVALENT (anchor is Some only when `!is_caret`, which IS `anchor == head`) —
  deleted per the PR-#298 doctrine, never suppressed. Validation per the 2026-08-13 decision:
  per-site scoped runs only (151 mutants, 0 missed), the full re-run deferred onto the box
  seed's `--iterate` credit. Lane forensics + the fix ledger:
  `docs/planning/pipeline/parked/407-*.notes.md` (parked 2026-08-14 behind the box sweep;
  resume note at its head).

- **Workflows grew up: a guided param prompt and a real delete** (TICKET-227, M12.2 — the #204
  follow-ups, React-first). Invoking a `{{param}}` workflow no longer dumps the raw template at
  the prompt: a guided modal (the Recipe-C naming-card family, React-approved at the POC first)
  collects one value per param — Enter advances, Enter on the last substitutes and INSERTS the
  filled command (still no auto-run; the review posture holds; a hand-desynced settings entry
  flashes instead of silently inserting nothing). "Delete Workflow…" opens a finder-family
  picker (family-true after inspect: static header, the query on its own › line) whose Enter
  removes + persists + REBUILDS the whole `WORKFLOW_BASE` palette block — the #204 R1 contract:
  ids are positional, so the save path converted to the same ONE builder and append-drift is
  structurally dead. Both overlays joined the #415 roster/blocked/snapshot sweeps (the
  26-state overlay inventory is 28 now, flip-table extended). The inspect caught the class
  recurring one layer out — two mouse diff-openers and the fleet-dispatch refuse-guard
  enumerated overlay states by hand and missed the new pair (a hidden picker's blind Enter
  would have DELETED a workflow); all three fixed, and the sweep rule sharpened in the ledger
  (F-claude-227-…-unrostered-mouse-openers-001). Pure `ParamPrompt` machine unit-pinned
  (multibyte, Done-no-increment, zip totality, the desync arm); the headless drive proves both
  fork arms, the guided insert, the R1 re-map (delete the middle → the survivor dispatches),
  and the roster flips. Suite 2155/2155; GATE GREEN [diff] 15/15 first try. Rename/reorder/
  template-edit recorded out (the settings file remains their surface); the live pixel pair
  rides #417.

- **Background commands notify the OS when you're away** (TICKET-226, M12.2 — the #203
  follow-ups). A background command that clears the 10s badge threshold now ALSO raises a macOS
  Notification Center banner ("Marley — command finished/failed" + the command, ≤120 chars) —
  but only while the Marley window is INACTIVE (at the desk the in-app ● badge suffices; the
  gate reads the existing render focus-edge flag). Pure `notify::os_note` decides;
  `marley_command::blocking::notify_macos` posts via osascript with constant script lines and
  the payload as opaque run-handler argv — injection-free by construction (unit-pinned), the
  program injected for tests (the `open_url_with` seam pattern), and the child REAPED by a
  detached thread (the inspect caught the zombie-accumulation class; now
  PR-claude-fire-and-forget-spawns-still-reap-001). Plus item (c): `tab_flashes`' positional
  `(project, tab)` keys now REMAP on tab/project close (the #236 close-shift discipline,
  unit-pinned arms) so a surviving unviewed badge never shows on the wrong tab. Item (b)
  (notify_ticks scrub) died of progress — #398's ContentId re-key + release-tail scrub already
  covers every close path; recorded, not built. First gate run went red on
  coverage/mutation (the spawn was untestable as first shaped) — restructured to the testable
  seam and re-run GREEN 15/15; suite 2153/2153; the real osascript argv smoked in-transcript.

### Changed

- **The multibuffer's row model moves into the DisplayMap facade — fold∘wrap∘excerpt
  becomes ONE projection stack** (TICKET-431, M33 — the B-c chain's recorded unification
  prize, byte-identical by proof). The mb's parallel slot model (#427's deliberate own-row
  deferral) is deleted: the materialized Header|Note|Line slot list now lives in
  `display_map.rs` as `ExcerptIndex` (slots + a `line_rows` snapshot), Arc-memoized on the
  model, rebuilt only by `refresh_cum` (the `WrapIndex` discipline), and served through a
  third `DisplayMap` arm (`excerpts()`; editor constructors unchanged — the fold∘wrap
  instance is bit-for-bit) with typed doors: `excerpt_total` / `excerpt_locate` (None iff
  empty, past-end saturates) / `excerpt_slot_of_row` / `excerpt_move_selection` (the pure
  mover; normalize DELEGATES its forward scan to it — one scan, the inspect find). The
  app.rs rim converts once per slot (`selected` is a typed `DisplayRow`; the gpui edge
  stays raw counts) and the model dropped `total_rows`/`locate`/`move_selection`/
  `slot_of_row` outright — one home for slot math, no delegating copies (`line_at` stays
  as the files-borrow jump adapter). Stages are disjoint this slice; the mb instance's
  identity fold slot + absent wrap are where folds/wrap-INSIDE-excerpts will compose.
  Proof: the #427/#430 mint carried VERBATIM as the in-test oracle sweeping every stage
  answer over discriminating fixtures (out-of-order windows, adjacent bands, note-less +
  Note-bearing, empty, post-refresh reshapes incl. an emptied trailing group); the full
  suite green with zero assertion-value edits (2244/2244); live-driven end to end on the
  search surface (⌘⇧F → ⌘⏎ materialize → movers → jump, real pixels). Four critics
  (correctness, provenance, state-integrity, simplification): byte-identical, no high
  findings. GATE GREEN [diff] 15/15 (cov 100 / MSI 100 on the new stage).
- **Keycap chips take the inset fill — both sides converge** (TICKET-225, M12.2 — the #222
  inspect's deferred one-token polish, built React-first). The palette keycap chips fill with
  the `background` token instead of `surface` (Marley: `keyboard_shortcut.rs`; POC:
  `bg-muted`→`bg-background` — a plan-time finding showed the POC chip was a pre-existing THIRD
  look, lighter-than-card, never reconciled). Dark theme: the classic inset keycap, ~6.6:1
  chip-text contrast, and the dark chip only gains contrast on the selected cyan row; light
  theme flips symmetrically on both sides to lighter/raised — and the change FIXES a real AA
  failure there (POC light chips were 3.13:1, now 4.58:1; Marley 4.4→4.8). The inspect critic
  also caught the relocated defect class: the widgets-gallery fixture would have rendered
  background-on-background, so it now wraps the chip in a `surface` card like the real
  consumer. React-approved at 5173 (DOM token check + capture) before the 1:1 Rust port.
  Suite 2148/2148; GATE GREEN [diff] 15/15.

- **Chrome type scale — the last 28 hardcoded text sizes route through type_scale** (TICKET-223,
  M12.2 — finishing what the #230 slice started; the ticket's other item, the `fallback_cell`
  stale-14.0 guard, turned out already superseded by #337, so only its doc line changed to name
  `FONT_SIZE_DEFAULT`). Five new FIXED chrome roles — `Panel`(13) for the dock/git/diff/menu/hint
  band, `Chip`(10) for fleet chips/meta, `Badge`(9), `Headline`(15) for empty-state/browser-card
  headlines, `Display`(22) for the launcher title — plus six sites landing on the existing
  `Caption`(11)/`Nav`(12) for free. Zero pixel delta (the 28-row inspect ledger verified every
  old literal equals its role's table value exactly; full-value table pins + fixed-across-
  [8/20/32/0/NaN] probes keep it held — the struct-return-no-Default trap means MSI alone
  cannot); a negative grep pins the literal count at ZERO so the next density tune edits one
  table. The #337 invariant is now spelled "everything except Command/Output stays put under
  ⌘=". First gate run went red on clippy unused-bindings — a python site-rewrite had counted a
  PRE-rustfmt string shape and reported vacuous success (the L-420 class; the gate caught it);
  wired and re-run **GATE GREEN [diff] 15/15**, suite 2148/2148.

- **POC parity — the two recorded overlay drifts are corrected React-side** (TICKET-416, M20 —
  the #318 D4/D8 verdicts landed; marley-web repo). The shared `useOverlaySelection` hook now
  CLAMPS at the list edges — token-equivalent to Marley's `palette.rs`/`finder.rs` saturating
  pair — instead of wrapping (wrap stays only with the selection-windowed code-actions picker,
  both sides verified); the `maxHeight: 83.3333%` cap and the then-inert internal scroll region
  are gone, so card height is content-driven with Marley's deliberate bottom overflow
  (DOM-proven at 900×400: card bottom 371px past the window edge, `maxHeight: none`, nothing
  scrollable). Driven before/after at localhost:5173: ArrowUp at row 0 wrapped to the last row
  before, stays at 0 after; 30 ArrowDowns pin at the last row. MARLEY-PARITY.md's overlay
  bullets updated to the clamp contract. Three further pre-existing POC divergences the inspect
  surfaced (render-cap `.take(20)`, Enter-on-empty, re-anchor keying) are queued as intake
  (`poc-overlay-parity-nits.md`), and the capture protocol gained
  PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001 (Chromium's
  synthesized hover steals selection under a parked pointer). Typecheck clean; GATE GREEN
  [fast] (no Rust in the changeset).

- **Overlay chrome — the sweep is finished; the chrome grew its non-modal sibling** (TICKET-414,
  M20 — the #318 named follow-up). The ten sites still hand-writing the 8-call card chrome
  (references — its searching/results `base` closure preserved —, code_action, file_symbols,
  symbols, search, problems, the editor completion popup, and the naming trio) now ride the
  helpers; the negative grep pins the core to exactly the two helper bodies workspace-wide —
  zero verbatim sites remain, 9/10 conversions chain-identical (per-site inspect ledger; gpui
  order-independence proven at source: every style call is a per-field write, no field written
  twice). The one deliberate delta: the completion popup takes the new
  `overlay_card_chrome_over_scroll` (gpui's `block_mouse_except_scroll`, its own docs' preferred
  choice) so the wheel reaches the editor beneath — which makes the popup's
  scrolled-out-of-viewport guard actually fire on scroll (scroll-dismiss was DEAD under
  `.occlude()`: the wheel never arrived). The naming trio extracted as Recipe C
  (`naming_overlay_card` over pure, unit-pinned `naming_card_geometry` — 0.3·w / h/4 / 0.4·w);
  the anchored six stay caller-side on purpose (per-site values + `&self` fonts; the
  `anchored_overlay_card` option is recorded for a 7th picker). Hover keeps its occlude (#318
  D3). Suite 2148/2148; live captures environment-blocked (no active session — rides #417).
  GATE GREEN [diff] 15/15.

### Fixed

- **Overlays — the transient-close roster now closes the launcher too** (TICKET-415, M20 — the
  #318 smoke find). `close_transient_overlays()` cleared palette/finder/history/find + eleven
  Some-typed overlays but not `agent_launcher` — the helper was extracted FROM the launcher's own
  closers (#393), which is exactly why the launcher never made its own list — so any other
  mouse-opened modal stacked over a lingering launcher (fleet drew over it in the #318 smoke).
  The launcher joined the roster; the membership rule is now written AT the roster (transient
  modals/pickers with key-arms join; persistent docks — fleet is toggle-scoped — are exempt).
  Inspect then found the same class one layer out: the two menus PREDATING the helper (the #175
  file-ref menu, the terminal right-click) still hand-closed a fossilized #398-era trio, silently
  missing every overlay the roster gained since — both now call the helper (a centered picker
  leaves the terminal/link exposed, so the keyboard-dead co-open was live there;
  F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001). The #318 smoke
  tightened from per-index asserts to strict one-hot five-flag equality per iteration + a
  verb-table tripwire (an appended verb can't pass vacuously), a direct roster-clears-all cleanup
  assert, the fleet exemption pinned BEFORE its toggle-close (the toggle guard would mask a
  wrongly-widened roster), and a post-loop all-closed tail. Suite 2147/2147; live pixel drive
  environment-blocked (0×0 off-screen windows, no active session — L-claude-318; rides the #417
  battery). GATE GREEN [diff] 15/15.

- **Pane dividers — the one you grab is the one that moves** (TICKET-422, M20 — Chad's "drag
  and drop works only on the first item and it's very clunky"). Every divider now works, nested
  splits included: a pure `divider_rects` walk (one descriptor per Split — path to the owning
  split, axis, a band-inheriting grab strip centered on the true shared edge, and the owning
  split's extent as the delta normalizer) replaces the flattened per-leaf handles whose sink
  (`resize_boundary`, retired) only ever mutated the top-level boundary 0. The drag applies via
  the new total `PaneGroup::resize_at(path, delta, min)` (`resize_split`'s clamp math unchanged;
  bad/stale paths are no-ops); the strip sits IN the content band (the old one drew 30px over
  the title bar and fell 30px short — pixel-verified dead), horizontal strips on Vertical
  splits, and a missed button release ends the drag on the next observed evidence (gpui's
  `dragging()` predicate; a mid-drag last-terminal close no longer panics). Inspect's two HIGH
  catches were both at the ONE call site: raw bounds where the panes tile gutter-inset (strips
  up to 8px off the seam), and `try_workspace()`'s documented fallback leaking occluding strips
  over non-terminal tabs — now the same bounds binding + the tiling's own gate, with an
  AGREEMENT unit (divider centers ≡ `pane_rects` edges over the same inputs) pinning the class
  (F-claude-422-pure-walk-right-call-site-wrong-twice-001,
  PR-claude-a-pure-helper-inherits-its-callers-coordinate-world-001). Suite 2146/2146; driven
  live (2-cell strip band+seam pixel-exact; a nested 3-cell split renders BOTH dividers on
  their seams). GATE GREEN [diff] 15/15.

### Changed

- **Rail — projects are addable from the rail itself** (TICKET-421, M31 — the last M31 port
  slice). The Workspace dock header gained the #418 top-band accent ＋, opening an add-project
  menu through the ONE context-menu machinery (`MenuKind::AddProject` + a const row table — the
  #393/#398 idiom) whose two rows dispatch the two SHIPPED verbs: "Open Folder…" →
  `open_project_picker` (the ⌘O door; gpui's native directory prompt, cancel arm untouched) and
  "New empty workspace" → `new_empty_workspace` (the launcher door) — labels byte-identical
  across the menu, the launcher, and the POC (one vocabulary, two doors). No new project
  machinery; the post-add tail (append + activate + Files sync + recent + persist) is the
  shipped `open_project_path`, unchanged — and the F-#236 pin now proves add NEVER re-keys
  index-keyed view state (append activates `len-1`; remap stays close-only).
  `caption_header`/`dock_panel` grew an optional trailing header-action slot (label-only callers
  pixel-identical). Inspect caught a doc/attr THEFT (the new fn spliced between a sibling's doc +
  `mutants::skip` and the fn — both re-attached to the wrong item, silently stripping an
  intentional mask; PR-claude-splicing-a-fn-between-doc-attr-and-item-steals-them-001). Suite
  2137/2137; driven live end-to-end (＋ → menu → "New empty workspace" → a real project appended
  + activated + Files re-walked + PTY live, then closed clean). GATE GREEN [diff] 15/15.

- **Rail — open editor files are per-file rows again; the frozen "Editor" row retired**
  (TICKET-420, M31 — the #237/#240 rail PROJECTION reversed; the one-surface editor model,
  the in-editor file strip, and `EDITOR_TAB_TITLE` all stay). `rail_rows` projects a CodeView
  tab as one `RailLevel::File` row per open view (surface order; disambiguated basename labels
  via the shipped `disambiguate_labels`; coordinates carry the editor tab + view index + cid;
  derived fresh per render, zero persistence change). `RailSelection` gained `File{p,t,view}` —
  the active view's row is the ONE selected row under the #419 model; background-active Rings
  land on a non-active project's active view; the pane-mount signal stays on CrossRef rows
  (model A coexistence). Click = the #174 cross-project body + `activate(view)` + the #275
  choke + persist; × = the new (p,t)-addressed `close_editor_view_at` (`Removed` releases one
  view AND persists the shrunk open set — a pre-existing durability gap where a closed file
  resurrected on relaunch, now fixed on both the rail and strip paths; `WouldDrain` closes the
  editor tab). Inspect also caught a VACUOUS re-pin (an absence assert keyed to a label no row
  carries anymore — re-keyed structurally; PR-claude-absence-asserts-die-silently-under-
  projection-changes-001). Suite 2134/2134; driven live (open two files → three basename rows,
  click moves the fill alone, × removes exactly that view); GATE GREEN [diff] 15/15.

- **Rail — ONE selected row ever; presence dots replace ancestor lighting and the ⊞**
  (TICKET-419, M31 — the first Rust slice of the #418 design). `rail_rows` now derives ONE
  internal `RailSelection` coordinate (None / Tab / Cell — two selected rows are unrepresentable
  by construction) instead of six independent per-level `active` flags; `RailRow.active` became
  `selected`, `pane_marked` died into `dot: RailDot { None, Filled, Ring }` (Filled = mounted
  elsewhere, the #398 foreign-host signal unchanged; Ring = container-of-focus / background-active;
  Filled outranks Ring). All six render arms route the fill from `selected` only; a 12px leading
  dot slot joins Tab/CrossRef/Arrangement rows (labels stay byte-positioned); the two ⊞ render
  sites and the #236 active-project background/emphasis retired. The #386 force-expand survives
  byte-identically — EXCEPT the #398 Editor-override is now gated on MULTI-cell grids (inspect
  find: a 1-cell CodeView grid selects its own Tab row, so force-expand must protect THAT row's
  section or one chevron click hid the selection with no ring anywhere;
  F-claude-419-force-expand-protected-a-breadcrumb-not-the-selection-001). Suite: 2131/2131 green
  (6 ancestor-lighting pins re-pinned to the truth table + 6 new #419 units incl. the
  every-RailLevel ≤1 sweep and the collapse/edge cases); driven live end-to-end (boot → expand →
  new terminal → split: the fill moved alone, containers ringed); React↔Marley parity pair
  captured — behavior 1:1, one recorded POC mock-data limitation (self-split reads Filled there,
  Ring here — Marley-authoritative per the #398 corrected semantics). GATE GREEN [diff] 15/15.

- **Rail — the M31 simple-rail DESIGN pass + the Zone A parity inversion** (TICKET-418, M31;
  marley-web + docs only, zero Rust). The new ChatGPT-style rail is settled in the React POC:
  single-selection fill (at most ONE row ever — ancestry lighting dies), 5px left-edge presence
  dots (filled = pane-mounted, the #398 ⊞ relocated; ring = section's remembered-active),
  small-caps non-selectable section headers with hover-revealed ＋/chevron, the 4→3-level indent
  ladder, hover-revealed ×s, a persistent accent add-project ＋. Selection derives from ONE
  discriminated selector mirroring the center router — the inspect critics proved per-row booleans
  break under foreign writers (4 HIGH fixes: stale `activeAgentId`, unreset `paneCellFocus`,
  palette-stranded `activeProject: 2`, the center split view writing cell ordinals into
  `activePane`). MARLEY-PARITY.md re-baselined: captures 34–38 + the measured geometry sheet are
  the port contract; for the rail the POC is the design source and Marley is the bug until the
  Rust ports land (#419 selection+dots, #420 per-file editor rows, #421 add-project). Knowledge:
  F-claude-418-selection-invariant…-001, PR-claude-single-selection-is-a-derived-selector…-001.

- **Editor/shell — the overlay-card chrome is ONE helper, and the centered-quarter geometry is
  pure and proven** (TICKET-318, M20 — the #312 inspect F17 follow-up). The 8-call card style
  (`occlude/flex/flex_col/bg/rounded/overflow_hidden/border_1/border_color`) was copy-pasted
  verbatim at 17 sites; seven now ride `overlay_card_chrome` (hover, def-picker, palette, file
  finder, history, agent launcher, fleet — the launcher and fleet joined because they are
  token-identical members of the centered-quarter recipe), with the remaining ten as a named
  follow-up. The five fractional cards assemble via `quarter_overlay_card` over a new PURE
  `context_menu::overlay_quarter_geometry` (`left = w/4, top = h/6, width = w/2`) whose unit
  tests prove containment BY CONSTRUCTION (the ticket's original "convert onto menu_origin's
  clamp" died at design: these cards have no `max_h` — height is content-driven by settled
  contract — so there is no box to clamp). Zero visual delta is the contract: every converted
  site's effective builder chain was verified call-by-call against HEAD at inspect, the geometry
  is bit-identical f32, and a new headless smoke (`recipe_b_overlays_open_and_draw_headless`)
  drives all five overlays through a real draw — which surfaced, without fixing (pure refactor),
  that `close_transient_overlays` omits the agent launcher (recorded for a follow-up). Selection
  behavior deliberately unchanged: palette/finder/history clamp, def-picker wraps (structurally
  coupled to its selection-windowed rows — the #312 open question, now decided). Driven
  before-captures are banked; the after-capture run is a named pending item (the headless mini
  had no interactive session — activation is unobtainable without one; §7-documented with
  chain-identity + draw-smoke as substitute evidence).

### Fixed

- **Editor — every workspace's LSP outcomes now land the tick they arrive, routed by their
  OWNING root** (TICKET-413, M20 — the #332 inspect follow-up, F4's deferral half).
  `consume_lsp_responses` drained only the ACTIVE root's host while every host's wire pumped,
  so a backgrounded workspace's answers AND abandonments sat queued until switch-back: a
  committed rename's "Language server didn't respond" flashed minutes late, and the ⌘T
  `workspace/symbol` fan-out (which SENDS to every Ready host) structurally couldn't merge
  non-active answers. The consume pass now empties EVERY host per tick (root-sorted, per-host
  queue order) and dispatches with the owning root — which is also what makes background
  rename/resolve CORRECT: encoding + `lsp_version_for` come from the owning host; the active
  root's lookup would MISS and the #322 REQ-009 conflict gate fails OPEN (a stale edit would
  apply with the wrong encoding), so owner-routing is correctness, not plumbing. The mandated
  per-consumer sweep became the ticket's real find: uri equality is FILE identity, never
  INSTANCE identity — under NESTED roots the same file opens under both roots as twins with
  byte-identical uris, request latches survive project switches, and independent twin version
  counters collide at equal counts, so every focused-editor arm could have delivered
  cross-instance (a background F12 executing a jump inside the focused project; a completion
  accept editing the wrong twin). Two inspect fixes close the family: the 8 focused-editor
  arms drop non-focused-root outcomes BEFORE any latch/UI touch (`outcome_is_for_focused_root`,
  latch deliberately untouched — a colliding twin re-ask may own it), and `apply_one_file`
  routes a WorkspaceEdit to the OWNING root's instance, never the lexicographically-first twin
  (the class #354 D12.5 closed for formatting, now closed for rename/resolve too). 19 headless
  drives pin it: the one-pass multi-host drain, sorted dispatch, background-abandonment flash
  timing, the two-host ⌘T merge, all 12 arms' cross-root postures, and the nested-twin guards
  (uri AND version matching — still dropped).

- **Editor — auto-close no longer over-pairs `'` at the `where`/`dyn`/type-usage lifetime
  positions** (TICKET-366, M22 — the #362 deferred half, closing the class). #362's
  `type_parameters` ancestry probe could never reach `where T: '` (the pre-insert tree has no
  `<>` anchor — the caret falls to `function_item`), so typing a where-clause or `dyn`-bound
  lifetime still produced `''`. The fix is a second pure probe,
  `marley_syntax::speculative_lifetime_at`: splice `'a` at the caret into a copy of the
  buffer, re-parse, and read the node kind the grammar assigns the spliced span — `lifetime`
  at every bound/type-usage position (all `where` variants incl. at-EOF, `Box<dyn … + '>`,
  `Ref<T, '>`, `&(dyn … + ')`, `-> impl Iterator + '`), `label`/ERROR at every
  char-literal/expression/const-generic position (`Foo<{ ' }>` keeps pairing — the ticket's
  feared block guard fell out free, and its hand-rolled backward-scan option died to the
  substrate, the #339 pattern). The app's Context closure now gates BOTH lifetime probes to
  the one keystroke they can answer (`text == "'"` — `LifetimeBound` is provably inert for
  any other char), which also stops the #362-era full-file parse on every non-quote opener
  keystroke. Loop labels still pair (deliberate, out of scope). Spike-pinned on
  tree-sitter-rust 0.24.2: 15 TRUE + 14 FALSE/degenerate probe units and a non-vacuous
  where-clause headless drive keep the recovery shapes honest across grammar bumps.

### Changed

- **Editor — a parked format-on-save now completes on its ORIGIN editor after a mid-format
  switch** (TICKET-354, M20 — the #314 C1-HIGH follow-up). The latch binds the ⌘S's
  instance as a `ContentId` at park; every completion lane (apply success, server
  Err/Abandoned, stale version, the 2 s deadline) resolves it O(1) through the #397
  registry and saves THAT editor — a tab/project switch no longer abandons the save
  (and the old surface-granular guard's residue, an unrequested plain save of the
  newly-active file on a same-project row switch, is structurally gone). "Active" is
  now a consent MODE, not an identity: an active origin takes the interactive path
  verbatim (arm flow, flash order — behavior-identical when you don't switch), while a
  background origin runs the #275 decision table with strict consent — a `Changed`
  conflict holds the write AND the arm (the banner greets your return), `CleanReload`
  defers to activation, `Deleted` recreates. The write tail is ONE machine
  (`write_and_mark`): fs write → `didSave` routed to the origin's OWNING root host
  (not the active project's) → mark-saved → the #284 racing-length cross-check.
  Inspect hardened three adjacent seams: ⌘S on file B now SETTLES (plain-saves) file
  A's parked save instead of silently discarding it; a buffer reload clears the
  in-flight formatting key (the #401 version-epoch-collision class — a stale answer
  can no longer splice pre-reload edits over reloaded text and save them); and the
  latchless ⌥⇧F response pins its target to the responding root (nested-root twins
  can't swallow another instance's edits). Ten new headless drives cover every lane,
  incl. a real-server (fake_ls) wire proof of the owning-root `didSave`; 2101
  workspace tests + the 15-gate `--diff` run green. The response queue's element is
  now `RequestOutcome::{Answered(Result<Value, RpcError>), Abandoned(Timeout |
  Disconnected)}` — pure types + generic `abandon_ids`/`abandon_all` helpers in
  `marley_lsp::rpc` (unit- + mutation-covered at 100), the Zed-observed
  `ConnectionResult` SHAPE reimplemented for Marley's tick/pump architecture. Both
  abandonment paths deliver exactly once per dropped purpose: the 10 s expire enqueues
  `Abandoned(Timeout)` (still emitting `$/cancelRequest`), and `on_connection_lost` now
  DELIVERS rather than destroys — inspect proved the old `responses.clear()` ate a dying
  server's final answers (mpsc yields buffered chunks before `Disconnected`, so
  "answered-then-died" arrives in one batch), so the queue is preserved and only
  still-pending purposes sweep as `Abandoned(Disconnected)`. All 12 consumer arms are
  explicit — replacing the queue's element type breaks every `Err(_)` catch-all at
  compile time, so the F6b trap (a synthetic `Err` flashing "Cannot rename this" /
  "Code actions failed" ten seconds after the keypress) is unrepresentable. Ten arms end
  silently with their latch cleared; the two COMMITTED-action arms (rename,
  `codeAction/resolve`) flash an honest "Language server didn't respond" instead of
  their semantic errors; references returns repaint-needed so the occluding "Finding
  references…" card drops the same frame; formatting still settles a parked save. Inspect
  also split the shared rename latch — `prepare_rename_request` gets its OWN slot (the
  F-CORR-2 idiom), so a superseded prepare's terminal outcome can no longer clobber a
  committed rename. The #331 `has_pending_*` host queries stay as the belt (spec D4).
  Follow-up filed: TICKET-413 (only the active root's queue drains per tick — a
  backgrounded workspace's outcomes defer to switch-back). 10 new tests, including the
  poll-driven inlay wedge dying end-to-end through 626 REAL pump ticks (mint → genuine
  expire → typed delivery → latch clear → re-send), plus the answered-then-died
  teardown-preservation proof at the host layer.

- **LSP doc-sync — the #309 empty-didOpen race class is now unrepresentable, and the
  app-level live-wire test lane finally exists** (TICKET-320, M20 — the #312 follow-up).
  #312 fixed the shipped bug by ORDERING alone (drain before collect) — correct, and one
  refactor away from regressing. Now `LspHost::reconcile` takes the LIVE document objects
  (`&[(&Path, &dyn DocText)]`; `DocText { version(), text() }`, implemented by the editor
  `Buffer`) and reads version + text INSIDE the branches that send: one Ready-gated read
  that decides AND materializes, so no pre-collected projection exists for a
  differently-gated read to disagree with. `needs_text` is deleted (it existed only to
  duplicate reconcile's decision table in the caller). The pump's collect becomes two
  borrow-clean passes over the new `ContentRegistry::editors_under(root)` — THE one
  root-filtered editor iteration, now also shared by the #326/#397 override snapshots, so
  the #321 ensure-set == sync-set invariant holds by construction instead of by four
  hand-copies staying identical. Reorder the pump's drain/collect/ensure/reconcile however
  you like: the worst case is a one-tick (16 ms) sync delay, never a wrong payload — the
  #312-era "do not introduce a drain() between" warning is retired. The #309-S2 idle hot
  path is now PROVABLE, not just preserved: a counting test double shows an unchanged
  document materializes nothing (and the didClose stream is sorted, making the whole
  notification order deterministic). The missing proof surface ships with it: a
  `#[cfg(test)]` sent-bodies capture on the host (recorded outside the child-handle check,
  so process-less hosts record too) + `RootView::lsp_sent_bodies_for_test`, the
  `didopen_payload_carries_real_text_against_fake_ls_headless` drive — a REAL spawned
  `fake_ls` through the app's own `[[lsp.servers]]` config (resolved from the test target
  dir; built-or-refreshed unconditionally via the `marley_command` adapter, which is what
  keeps the cargo-mutants pristine-copy baseline green AND every line covered), reaching
  Ready via a real `drain()` of the child's initialize response with ZERO injection APIs,
  asserting the captured didOpen PAYLOAD byte-equal to the buffer (exactly one didOpen;
  zero didChange for an unedited buffer) — plus counting/pre-Ready/real-edit host units
  and the `scripts/selftest/README.md` "LSP wire capture (tee wrapper)" procedure with the
  same-file-definition-empty diagnostic tell. Gate green 15/15; diff mutants 4/4 caught
  (MSI 100).

### Fixed

- **Git gutter — a file's marks die with its file; a stale cache key can no longer blank a
  reopened dirty file's gutter** (TICKET-412, M19 — the #353 sibling, found at its inspect).
  `git_marks` had the exact `editor_folds` leak class one field down: populated per viewed
  file, removed only when that same file was ACTIVE again with an empty diff, scrubbed by no
  close path — a bounded memory leak for the app's life. The fix rides the same
  census-guarded arm in `RootView::release_editor_views`: the entry AND a `git_marks_key`
  naming the dropped path clear at the last same-path instance drop (an alias-root survivor
  keeps the shared entry, the #353 census). The key half is load-bearing, not hygiene: no
  generation bump fires on activation, so a map-only scrub would have MINTED a wrong-render
  the leak never had — close an active dirty file, reopen it with no save/commit/reload in
  between, and `refresh_git_marks` early-returns on the matching stale key over the scrubbed
  map, leaving the dirty file's gutter unmarked. The clear also heals the
  closed-then-externally-modified reopen (no reload bump fires for a closed file). Proven by
  six headless drives (row + tab close scrub, twin-view retention, alias-root survivor
  census, key-clears-with-its-entry negative + positive, real-git close→reopen recompute);
  gate green with coverage 100% and MSI 100 on the diff.

- **Editor — the hand-lexer fallback can no longer lex inlay phantom text; both syntax
  arms now share one raw→display remap** (TICKET-333, the #331 F8 follow-up). On a row
  the tree-sitter cache cannot serve, the render fell back to lexing `layout.display` —
  which since #331 contains server-controlled inlay hint text, so a hint carrying `"` or
  `//` could open a string/comment token INSIDE the phantom whose state bled rightward
  into real code (mis-colored cells only; layout/caret/click/selection were always
  correct; reachable on a Rust file whose rows the memo can never serve — the memo
  splits on `\n` while ropey also breaks on bare `\r`/FF/NEL/LS/PS — with the #305
  fold-ellipsis marker a second, race-free phantom producer found at inspect). The
  fallback now has the primary arm's shape: `code_syntax::highlight_display_ranges`
  hand-lexes the RAW buffer line and maps each span through the new
  `code_view::spans_to_display_bytes` — the ONE remap helper both the tree-sitter arm
  and the fallback (main row + sticky header) now ride, hoisted at inspect from three
  inline copies in the coverage-excluded app.rs shim into the covered pure layer.
  Phantom text can no longer seed lexer state by construction (`col_of_span_end` keeps
  every mapped span off a phantom); hints-first ordering stays as belt-and-braces. On
  hint-free rows the mapping is byte-identical to the old display-lex except one pinned
  deliberate difference: a zero-width combining mark immediately after a token is
  absorbed into the token's span (grapheme cluster stays one styled run, agreeing with
  the primary arm since #268). Proven by the six-test t333 matrix (phantom no-bleed +
  negative control showing the old shape's bleed, floor identity for TOML/Rust/Markdown,
  tab+CJK identity, the zero-width pin, disjoint-ascending with mid-line + EOL phantoms)
  plus a live driven capture (Cargo.toml on the permanent TOML floor, string tokens
  pixel-sampled green); gate green with coverage 100% and MSI 100 on the diff.

- **Code folding — a fold entry dies with its file; reopening never resurrects stale folds**
  (TICKET-353, M19 — the #305 W-2 parked leak). `editor_folds` was insert-only across
  file/tab close: a bounded per-file anchor leak, plus a stale-fold-on-reopen edge (old
  anchors, stamped with the dead buffer instance, resolving against the fresh one). Every
  editor view release now routes through `RootView::release_editor_views` — the editor
  sibling of the #398 terminal-side scrub: the pure seam
  (`content::release_editor_views`, now `#[must_use]`) reports the stored canonical paths
  of instances whose LAST view dropped, and the wrapper removes each from `editor_folds`
  unless `content::any_open_editor_with_path` finds a surviving same-path instance — the
  census guard that keeps an alias-root twin workspace's folds alive (two instances of one
  canonical path share one entry; a plain clear would have regressed the case the leak
  accidentally got right). Twin views of one instance retain folds (non-last releases
  report nothing); reopen presents unfolded (the in-session fold model — Zed-matching
  view-scoped behavior). The resolve-HIT mount arms (viewer + split) now mint rows from
  the instance's STORED path, closing a TOCTOU corner where a join-spelled birth could
  leave the row key and scrub key divergent. Found at inspect and ticketed: `git_marks`
  carries the identical leak class (TICKET-412). Proven by five headless drives
  (row-close, tab-close, twin-view retention, reopen-unfolded, alias-root survivor) + the
  pure census truth table; gate green with MSI 100 on the diff.
  root can no longer double-open a file, fail the F12 landing, or search stale text**
  (TICKET-319, M20). Every editor-open seam now stores
  `marley_project::canonical_under_root` (canonicalize, with the `resolve_under_root`
  join as fallback — deliberately the same fallback `same_file` compares with, so storage
  and compare can never disagree): the shared loader, both open probes, both
  session-restore arms, arrangement `key_ok` + mount, and `open_and_place_caret`
  (normalize-at-entry, so every jump caller — F12/⇧F12/⌘T/search/problems/⌃- — lands on a
  raw `==` of two helper-produced spellings). Prefix/derivation consumers swept in the
  same pass (the L-identity-unification design lesson): the rust-analyzer host-spawn gate
  compares canonical-vs-canonical via the extracted, unit-tested `wants_rust_host` (a
  verbatim compare would have silently disabled ALL LSP for symlinked-root projects); the
  ⌘T/problems encoding lookups rank by the HOST's canonical root (new `LspHost::root()`);
  `LspHost::absolute` now DELEGATES to `canonical_under_root` (one normalization algorithm
  in the workspace); and the new `rel_under_root` derives root-relative spellings for the
  git-gutter pathspec, RevealInTree, and the search/problems display labels. Inspect
  caught a real miss, fixed + pinned
  (`F-claude-search-walk-verbatim-root-misses-canonical-override-keys-001` /
  `PR-claude-walker-yield-spelling-must-match-store-keys-001`): ⌘⇧F's dirty-buffer
  overrides are keyed by stored (canonical) paths, so the search now WALKS the canonical
  root — the verbatim walk missed every live override under an aliased root and silently
  searched stale disk text. `same_file` gains the identical-spelling fast path; links.rs
  stays pure under a canonical-root caller contract with its two accepted misses (an
  abs-verbatim-alias ref; an internal-symlink-crossing rel ref) documented AND pinned by
  test. Legacy persisted layouts (verbatim-joined spellings) restore to the canonical
  identity with no duplicate tabs (drive-tested); wire formats unchanged. Workspace
  identity is untouched (D-OPEN-DEDUPE-SCOPE: aliased ROOT spellings stay two
  workspaces). 11 new tests (units + 2 headless drives + doctests); workspace 2059/2059
  green; gate GREEN [diff] (lines 100%, MSI 100 on touched lines).

### Changed

- **The scrap-forge pivot, product-rip slice 2 — `marley_forge_client` and every forge
  product surface are gone; the generic seams survive** (TICKET-411, M30; completes the
  pivot — the CONSTITUTION preamble amendment landed first, isolated, at `12a775e`).
  DELETED: the whole `marley_forge_client` crate (client, fleet transport, backoff,
  84 of its 105 tests — the other 21 moved), `forge_view.rs` + the ⌘⇧F sprint overlay +
  `toggle-forge` (⌘⇧F stays Editor-scoped project-search), `RightSection::Forge`
  (cockpit = Details/Agents; `section_tabs` 3→2, cockpit slots 3→2, the Browser-section
  ＋/context menus 4→3 rows with **Agents re-seated as the #387 row-0 default**), the
  status-bar sprint segment (the agent segment is index 0 and keeps the click-to-Agents
  affordance), `Icon::Forge` + its asset, the fleet-brain transport
  (`FleetSubscription`/`endpoint_for_brain`/`ConnectionState`/arm-c
  `ForgeClientMissing` + `NO_FORGE_CLIENT_REASON`), `mcp_config.rs` (orphan-proven —
  nothing reads `.mcp.json` anymore; a tombstone drive pins it), and
  `OverlayStates.forge_open` (the frozen overlay inventory is 26). RE-HOMED: the #406
  browser-failure probe → `marley_app::browser_probe` (module + 8 units verbatim, a
  9th for the inlined status-line parser, 13 livewire rows →
  `tests/browser_probe_livewire.rs`; gates.sh drops the dead adapter exclude).
  SURVIVES: the forge-agnostic fleet RAIL — snapshot model, demo feed, and the
  dispatch/answer machines, whose confirms now resolve through the STANDING result
  channels with the shared typed `NO_TRANSPORT_REASON` ("no fleet transport") — the
  folds, nonce-supersede law, and retry-prefill stay live and newly pinned (the
  supersede row is deterministically testable now and tested); `classify_fleet_setup`
  narrows to (target, config-dir) so a configured brain root classifies `Configured`
  and the header shows the quiet bare "Fleet" (#369's line — no lying reason). Stale
  persisted `right_section = "forge"` / grid `C=forge` fall back to Details (both
  pinned). React-first: the POC ripped + visually verified before the Rust
  (ForgePane/ForgeOverlay deleted, 12 files edited, typecheck green). Workspace
  2048/2048 green; gate GREEN [diff] (19/19 diff mutants; lines 100%).

- **The scrap-forge pivot, product-rip slice 1 — the browser loses its forge identity;
  the generic embedded browser stays** (TICKET-410, M30; seam-map lineage from #405/#406).
  `RootView.forge_web_base` and its whole `.mcp.json` origin chain are DELETED — the boot
  derivation, `derive_forge_web_base_now`, the Retry re-derive (`effective_retry_base` and
  its arms test retire; `retry_plan` survives, generalized to desired-vs-mounted), the
  origin-caption fallback, and the `#[cfg(test)]` oracle. The mount gate is
  `mount_plan(None)` and the Retry arm `retry_plan(None, …)` — the ONE documented seam a
  future URL feature fills, kept CALLED so the pure planners stay compiled + mutation-
  enforced (`L-claude-rip-keeps-pure-seams-called-constant-input-001`). The pinned-origin
  nav verdict MOVED in-app — `browser.rs::{same_web_origin, web_origin_of}`, byte-identical
  with its 4 tests, `url = "2"` promoted (in-lock already; zero new supply chain) — so the
  #404 parsed-host loopback re-check stays under the cov/MSI-100 floor after TICKET-411
  deletes `marley_forge_client` (whose copy is untouched till then; the probe edge stays
  wired for 411 to re-home). The "Forge web view" placeholder copy is re-voiced to
  **"The embedded browser awaits a page source."** (React-first: settled + screenshot-
  verified in marley-web, ported 1:1, parity-paired live — the same pixel tiers as the
  #406 capture); CommandId(31) "Browser: Reload" drops its `forge` keyword; the DEAD
  `orchestration.web_url` field is deleted (never production-read; old settings files
  still parse — proven on the real TOML codec). `mcp_config.rs` stays (promotion
  correction: its boot call site also feeds the sprint client + fleet-brain bearer — it
  falls in 411 with them); its 2 forge-coupled composition tests retire now. Tests: the
  boot-derivation drive is replaced by `browser_boot_ignores_mcp_json_headless` (a present
  `.mcp.json` feeds the browser NOTHING), the retry drive repoints to the no-source arm,
  and the 10 generic drives + 20 `browser_state` + 6 `browser.rs` units survive untouched.
  Workspace 2152/2152 green; gate GREEN [diff] (10/10 diff mutants caught).

- **The scrap-forge pivot, process half — the pipeline goes purely local: tickets are
  files, the queue is a file, knowledge is a greppable ledger; the forge sidecar
  services stop** (TICKET-409, LOCAL — the first local-only ticket; owner decision
  2026-08-09, recorded in `docs/planning/intake/scrap-forge-pivot.md`). The
  CONSTITUTION was amended in an isolated commit per its own rule: §19 is now **Local
  Knowledge & Tickets** (ticket docs + an ordered `BACKLOG.md` whose top Queue row is
  what `/work` picks; numbering = 1 + max, never renumbered; a `## <code>`-block
  ledger under `docs/planning/knowledge/`), §18.3 recall/capture became grep/append,
  §15 stopped claiming the retired capture hook, and the parked §19 hook trio was
  deleted (verified unwired). Before unplugging, everything came OUT of the sidecar:
  **761 knowledge blocks** (312 prevention rules, 205 failures, 201 distilled
  lessons, 43 architecture decisions — full-corpus byte-diffed against the DB at
  inspect) and **all 16 open tickets** (faithful docs + provenance ids + a
  two-section backlog: 15 Queue rows topped by the product-rip pair TICKET-410/411,
  3 Deliberate rows for idle/upstream/hardware-gated work). The 8 pipeline skills +
  3 templates + `.mcp.json(.example)` are forge-free (the S1 negative smoke pins the
  documented CONSTITUTION exception set; a drift-inject smoke proves it reds).
  Inspect caught a real export bug — the DB double-encodes `tickets.tags`, and the
  Tags lines rendered as character soup — fixed and captured to the new ledger as
  its first two live appends. History is immutable: completed pipelines, closed
  tickets, and this changelog keep every forge reference. The product surface
  (Forge pane, `marley_forge_client`, fleet-brain wiring) rips next via
  TICKET-410/411; the forge DB stays intact on disk for the other projects it
  tracks, with the services stopped and their broken LaunchAgents disabled.

### Fixed

- **Shell-codec index hygiene on drop — restore focuses the tab/project you LEFT, not a
  drifted neighbour; the grid blob gains the writer's last missing framing guard**
  (TICKET-408, forge #408; M29 wake — filed by #403's inspect critic). Every place the
  persistence codec DROPS an entry used to keep the sibling "active" index verbatim or
  merely bounded (`.min(last)`), so a relaunch after a degraded save/restore (a `T=`
  whose PTY spawn fails, a `V=` with no readable file, a vanished or unrepresentable
  project root) came back focused one slot off. All four sites now map the saved index
  through the SURVIVING original indices via the shared #243 `reclamp_active` —
  serialize-side (`serialize_shell` pre-scans survivors, covering the pre-existing
  `V=`-drop drift and the empty-root line the reader skips — inspect F1) and
  restore-side (the app.rs loop records survivors as it pushes). The terminal grid
  `blob` — the ONE writer input pushed to the wire unguarded while roots and paths had
  D2 guards — now drops its tab on an entry-framing hazard via `breaks_entry_framing`
  (`\t\n\r` + `\x1f`, one predicate now shared with the #243 path filter), which also
  closes a laundering hole (a hand-mangled multi-`\x1f` `T=` no longer re-emits as a
  forged frame on the next save). Hazard-free layouts serialize byte-identically
  (pinned by the pre-existing exact-bytes golden); out-of-range actives are
  deliberately clamped (unreachable from the live writer). Pure seams at cov/MSI 100
  (15/15 diff mutants); restore behavior proven by three headless drives seeded
  through the real codec — the drive fixtures place every expected survivor strictly
  BEFORE the last one, because at the last survivor the old `.min(last)` coincides
  with the fix and proves nothing.

### Changed

- **The delivery gate goes DIFF-first — FULL demoted to a deliberate overnight audit; mutation
  jobs go mode-aware** (post-mortem of the 2026-08-07 double freeze). Two WindowServer-watchdog
  freezes (hard reboots) landed mid-FULL-gate: an hour-plus of dual rustc+ld mutation chains
  beside a desktop session outruns the 16GB box — the second freeze cut down a whole-workspace
  mutation run at 836 outcomes / 0 missed. The receipt was always mode-blind, so the per-change
  bar loses nothing: `/commit` now runs `scripts/gates.sh --diff`, the enforce-commit-gate
  message names both modes, and gate:5 picks jobs by mode — DIFF 2 (minutes of exposure),
  FULL 1 (idle/overnight only), `MUT_JOBS` overrides either. The RAM history is recorded at
  the knob itself (gates.sh gate:5).

### Added

- **Browser nav chrome + lifecycle — loading truth, the honest error card + retry, the reload
  verb, and teardown total from every state** (TICKET-406, forge #406; M29 — the #389 Phase-E
  train's closing slice: THE EMBEDDED-BROWSER TRAIN IS COMPLETE). One pure state machine
  (`browser_state.rs`, cov/MSI 100) owns loading × error × reload with sticky first-fault-wins
  cards; every adapter (wry callbacks, probe thread, pump, palette verb, render) is
  decision-free. wry 0.56 delivers NOTHING on a transport failure (source-sweep fact: `Started`
  fires on COMMIT, `didFail*` unimplemented), so the error truth is an out-of-band origin probe
  (`marley_forge_client::probe_web_origin` — a NEW unmasked `probe.rs`, raw-TcpStream GET,
  every-resolved-addr connect, total wall-clock read budget, livewire-tested across nine loopback
  rows) racing each load, plus a pure 15s tick deadline and the macOS renderer-terminate hook; a
  committed 4xx/5xx body looks success-shaped to wry, and the probe's status outranks it.
  Teardown settled INLINE by type necessity (`wry::WebView` is `!Send`; its `Drop` already
  detaches) — `reap_browser_holder`/`teardown_browser_life` is the ONE seam, and
  INV-NO-ORPHAN-WEBVIEW is driven from every phase × close path. Staleness is one law on two
  axes (webview generation + probe attempt), closing the inspect-found race where a pre-retry
  probe's late fault stuck a false card over a recovered page. Retry re-derives `.mcp.json`
  active-root-keyed, and a transiently unreadable config can no longer masquerade as removed.
  "Browser: Reload" ships as static `CommandId(31)`; the chrome states are the React-settled
  two-tier cards with NO header row (judged on A/B pixels). Live-proven: error card with the
  forge down, ↻ Retry recovering onto a fixture origin, the palette verb reloading through the
  overlay hide/show, ⌘W leaving no orphan child. `browser_attach_error` is retired into
  `Error(AttachFailed)` — the #405 stored-error gained its card and its retry.

- **The Forge URL pane — the wry child at the pane rect, the 27-state z-order hide-shim,
  pinned-origin nav** (TICKET-405, forge #405; M29, the #389 Phase-E train's slice 3 — "Forge
  opens in the browser" is LIVE). A wry-as-child WKWebView (the #402 GO substrate) attaches
  through the gpui window's raw handle at the Browser pane's rect (`center_bounds`), tracks it
  per-frame via the probe-proven capture+diff sync (rounded-i32 key, Logical→Logical — no scale
  math), loads the #404-derived origin RE-derived at every boot (nothing persisted — the #403
  bare-`B` codec byte-unchanged), and pins navigation to that origin
  (`marley_forge_client::same_web_origin` rides the ONE parser + the parsed-host loopback
  re-check; wry denies handler-less `window.open` — no pin escape, verified in-source). **The
  z-order hide-shim**: the webview hides while ANY of the frozen **27-state** overlay inventory
  is up (`browser::OverlayStates` + `overlay_is_up` — one predicate, one choke point) built by
  ONE snapshot fn whose discipline the inspect round NAMED: **mirror the DRAW GATE, not the bare
  state** — its three catches were the top-search results dropdown the design sweep misfiled as
  chrome, the references card's in-flight arm (both under-hides), and the rename draft's
  live-editor half whose bare state would have blanked the pane FOREVER (the over-hide).
  Unconfigured → the #403 placeholder stands; no bearer can reach any URL (pinned end to end).
  Pure seams at cov/MSI 100 (42/42 diff mutants); the wry FFI is one masked shim file
  (`webview_shim.rs`). **Driven live, pixel-verified**: mount at the exact rect, palette
  hide/reappear, tab-away/back, resize tracking, quit→relaunch remount from the bare `B`.
  Evidence correction carried to #406: WebKit renders the sidecar's 401 JSON as a BLANK page —
  the error card owns making that honest.

- **`forge_web_base` — the `.mcp.json` → Forge web-origin derivation seam, pinned**
  (TICKET-404, forge #404; M29, the #389 Phase-E train's slice-3 input). The third consumer of the
  one #381 active-root `.mcp.json` resolution (after the MCP client and the fleet brain endpoint):
  `marley_forge_client::forge_web_base` derives the Forge web-UI base as the wall-admitted MCP
  endpoint's **WHATWG origin** (ASCII serialization — path dropped, scheme-default port elided,
  IPv6 canonicalized), riding `forge_endpoint_from` so one parse and one loopback wall govern all
  three consumers, and the bearer stays structurally out of the URL (#370/#375). Every unresolved
  arm — no file, unreadable, unparseable, no forge entry, bearer-less, wall-refused — derives
  `None` (the Browser section's empty rail home, never a broken page). The `url` crate (already
  in-lock transitively, Apache/MIT) was promoted to a direct dependency rather than hand-rolling
  origin serialization (the #277 precedent; the #339 rule). **Inspect caught and closed a real
  wall bypass** (`BF-claude-wall-admit-vs-parsed-host-divergence-001`): the hand-parse wall reads
  `http://127.0.0.1:pass@evil.com/` as loopback-with-port while WHATWG reads host `evil.com` — the
  derivation now re-asserts loopback on the PARSED host, so a tampered config can never point the
  future #405 pane off-loopback (`PR-claude-recheck-the-parsed-value-the-consumer-uses-001`).
  Design-time probe of the real sidecar: no web UI serves at the origin yet (401/404) — the seam
  pins the contract; the escape hatch stays out until a real mismatch exists. Pure seam at
  cov/MSI 100; consumer wiring lands with #405.

- **The embedded browser becomes a tab citizen — `TabContent::Browser` + the bare `B` shell tag**
  (TICKET-403, forge #403; M29, the #389 Phase-E train's slice 2). A Browser tab now opens from a
  4th Browser＋ row, files under the rail's Browser section beside its transitional cockpit tenants,
  survives a restart, and closes — rendering a deliberate **placeholder** ("No page loaded"): no
  webview, no URL, no `.mcp.json` read, so it is total over an unconfigured machine by construction.
  #396's DECLARED `Content::Browser` is cashed into the #394 registry as ONE app-wide resident
  (identity follows the single boot-time `.mcp.json` resolution that already feeds the forge client
  and the brain endpoint). Persistence is a **bare `B`** beside `T=`/`C=`/`V=` — payload-less because
  v1 content IS the derived Forge origin, so no id and no URL reach the wire; additive by the shipped
  unknown-tag skip, and a shell without a Browser tab serializes byte-identically to before.
  **The lifecycle is the load-bearing design call, and it is the INVERSE of #400's cockpit.** Browser
  drops on last close, so `resolve_browser` hands back exactly one acquired view in both branches
  (the `resolve_open` contract) and a switch must give it back — copying the cockpit's
  acquire-only-on-append shape, which is correct only because a pinned anchor holds its insert's
  first view, would have stranded a view no close path could release. Dropped now because the #405
  webview is the reapable thing; pinning would have baked in a leak-shaped contract needing reversal.
  React-first per `MARLEY-PARITY.md` (the POC's always-present "Web" fixture became a real
  born-on-open row); parity verified by sampled pixels, not eyeballed. Inspect (4 critics, 20
  findings — 11 fixed, 9 rejected with reasons) caught, among others, a **red gate the Phase-3 notes
  had recorded as green** (clippy run without `-D warnings`, output filtered by a pattern that misses
  rustc's dead-code wording → `PR-claude-run-the-gates-exact-command-never-an-approximation-001`),
  a broken pinned menu-table test, three React callers left activating a tab they no longer opened
  (→ `PR-claude-fixture-to-instance-audit-every-activator-001`), and `build_shell_layout`'s
  non-exhaustive predicate chain, where a future tab kind would have silently stopped persisting —
  now an exhaustive match, as are `Content`'s five accessors. Deferred with reasons as #408.

- **Embedded-browser proof-of-embed probe — wry-as-child WKWebView verdict: GO** (TICKET-402,
  forge #402; M29 opens roadmap Phase E — the #389 train slice-1). A new dedicated bin-only crate
  `marley_webview_probe` — deliberately **the ONE place `wry` enters the tree** (`cargo tree -i
  wry` → the probe is its sole consumer; the `marley` product binary stays wry-free at the linker
  level) — attaches a wry 0.56 child WKWebView to a gpui window through the exposed
  `RawWindowHandle::AppKit` and walks the embedded-browser-model.md Q1 go/no-go checklist headed +
  operator-driven: (1) pane-rect position/resize tracking via a render capture+diff `set_bounds`
  sync (Logical→Logical — no scale math needed); (2) the z-order hide-shim — a palette-style
  overlay overlapping the webview by construction, the webview hidden via first-party
  `set_visible(false)` with zero-bounds exercised as the in-session alternate (⌘M); (3) focus/IME
  handoff both directions incl. the page-IPC `focus_parent()` return trip (wry 0.56 ships
  first-party `focus()`/`focus_parent()`). **All three points held (operator-confirmed
  2026-08-06): wry-as-child stands — slices 3–4 (#405/#406) proceed on substrate A; CEF-OSR stays
  the named pivot, untriggered.** Env-gated (`MARLEY_WEBVIEW_PROBE=1`; an ungated run refuses
  with exit 2 before any window — unit-pinned pure `probe_enabled`, the crate's one un-skipped
  mutation surface: the diff's 3 mutants, all killed). §0 ACCEPTED-UNTESTABLE shape: a named,
  documented coverage exclude in gates.sh; every headed fn `mutants::skip`-justified. Inspect
  (3 critics, 12-finding ledger) caught pre-gate: wry's Linux-only gtk3 tree turning gate:8 RED —
  10 unmaintained RUSTSEC advisories, fixed with per-id justified `deny.toml` ignores (the
  TICKET-007/022 pattern; `[graph]` target-scoping rejected — it would unscope license vetting) —
  and a real Rc CYCLE: wry retains the IPC closure inside the WebView, so a strong `Rc<Shared>`
  capture made the WebView's `Drop` unreachable (→ Weak capture;
  `PR-claude-callback-stored-inside-owned-resource-captures-weak-001` — the shape every
  production wry callback in #405/#406 must honor). Gate GREEN [diff], 15/15;
  2050/2050 workspace-green.

### Changed

- **Fleet header "misconfigured" diagnosability state** (TICKET-384, forge #384; M25 carried in
  M28; the #376 inspect-F8 shelf note + the QA-2026-07-21 find, cashed in). A configured-but-
  broken fleet setup now NAMES itself instead of degrading silently: the boot start-gate's
  outcomes are classified ONCE at the #376 post-shell-restore site (keyed to the restored active
  root, never the launch cwd) by a new pure `fleet_live::classify_fleet_setup` into
  `FleetSetup { Unconfigured, Misconfigured(Misconfig{arm, reason}), Configured }`, mirroring the
  gate's own abort order as the deterministic one-reason precedence — arm (b) a rejected
  `brain_endpoint` (non-loopback / non-`http://` / unparseable), the config-dir micro-arm
  (structurally unreachable today, classified per the no-silent-abort contract), arm (c) no forge
  client from the post-#381-resolved `.mcp.json`. Misconfigured roots title the right-dock header
  **"Fleet · misconfigured"** (`fleet_header_title` widened to (setup, wire); Misconfigured wins
  every wire state — arm (c)'s empty-bearer subscription still starts and may reach Live, but a
  config fault outranks wire truth) and render ONE compact muted reason caption atop the rail
  body — clamped AT CONSTRUCTION (`Misconfig::new`, type-enforced) through the shared
  `clamp_card_text`, which is hardened here to also strip bidi controls (U+202A–202E overrides +
  U+2066–2069 isolates — the Trojan-source display-reorder gap, closed for all four consumers);
  arm (c)'s reason IS the in-card literal via the new shared `fleet_rail::NO_FORGE_CLIENT_REASON`
  const (header and card tell one story by construction). Arm (a) unconfigured roots stay
  byte-identical ("Fleet", demo verb, rail empty state untouched); Live/Reconnecting render
  byte-identical; no write-path or `marley_forge_client` changes (`ConnectionState` untouched —
  the #376 D3 ownership split holds). **React-first:** the POC gained its missing fleet-dock
  surface (`components/FleetDock.tsx`, palette-cycled mock states per arm) — the approved look
  (caption 11px muted at the empty-hint inset) ported 1:1. Floors: cov 100 / MSI 100; 6 new
  classifier/title units (incl. the full 3×3 title product space + a hostile-url clamp pin), a
  bidi-clamp unit, 3 new headless boot drives (arms a/b/c — arm c pinning
  misconfigured-over-a-running-wire), live sandbox capture of the arm-(b) header + reason with a
  byte-identical ⌘Q settings round-trip; 2043/2043 workspace-green.

- **Cockpit onto the registry — the residency half** (TICKET-400, forge #400; M28 The Registry
  Payoff; the #388 slice-8, cockpit only). The three cockpit surfaces (Details / Agents / Forge)
  become `Content::Cockpit(RightSection)` residents of the #394 content registry under the same
  one-instance-many-views lifecycle as terminals (#396) and editors (#397) — app-wide
  **SINGLETONS** resolved through the ONE `resolve_or_register_cockpit` door (a 3-slot
  `CockpitIndex`, lazy, self-healing), so a second project's Forge tab is a second VIEW of the
  same instance — the app's first legitimately >1-view content, exercising the #394
  acquire/release machinery for real. `TabContent::Cockpit` becomes id-bearing (`(ContentId,
  RightSection)`, the #388 Q1 shape): the TAG keeps `cockpit_section`/`rail_section`/the codec
  writer pure (zero registry reads), while the render dispatch resolves the section through the
  registry with the tag as a total fallback (equal by construction). Lifecycle is **PINNED**
  (D-OPEN-SINGLETON-LIFECYCLE): the insert's first view is a standing anchor the index holds
  forever — closing every cockpit tab (either close path; both now route the cockpit-ids-only
  `release_cockpit_views` seam with the `#[must_use]` consumed and the never-reaped invariant
  debug-asserted) leaves the resident alive, and reopening reunites with the SAME id. All three
  open doors (the section verb, the rail ＋ menu, ⌘⇧B — the third caught by the compiler after
  the design's caller census missed it) funnel through one total `open_cockpit_tab` helper
  (acquire-iff-appended). Persistence is byte-identical: the bare `C=<section>` codec is
  untouched (zero grid_layout edits; ids never serialize) — live-verified: before/after captures
  of all three sections are `cmp` BYTE-IDENTICAL and the ⌘Q wire round-trips exactly, with the
  #95 `right_section` selection semantics unchanged. `addable(Cockpit)` stays `false` — no
  pane-cell arm shipped, so there is NO visible delta anywhere; cockpit-in-a-pane is the
  exposure follow-up (D-OPEN-KIND-GATING). The Details/Agents/Forge label match collapsed to
  one mint (`right_dock::section_label`). React-first: N/A (pure ownership migration; the
  ContentId↔PaneItem vocabulary tie holds). Floors: cov 100 / MSI 100 on the pure seams; 3 new
  headless drives (cross-project sharing + a REAL double-boot, the ⌘⇧B door pin/close/reunite,
  project-close release) + 5 new units; 2033/2033 workspace-green.

- **Nameable arrangements — the `[[panes]]` settings table** (TICKET-399, forge #399; M28 The
  Registry Payoff; the #388 Q4/Q5 slice-6). A multi-cell arrangement stops being a positional
  "PANE n" and becomes a NAMED, durable object. The verb: **"Name Pane…"** in the ⌘⇧P palette
  (`NAME_PANE_ID` — id 30; an inspect-caught collision at 20 would have silently fired the find
  bar's regex toggle, now pinned by a reverse guard), or **double-click the rail's Arrangement
  row** (the #177 rename idiom, seeded with the current name) — both open the #204-shaped inline
  card (caption / `{draft}▏` / the cells summary), registered at every #267 choke point
  (`text_input_blocked`, the #378 two-owners exclusion incl. a symmetric fleet-brief refusal,
  `close_transient_overlays`, close-tab/-project dismissal). Enter commits: the name sanitizes
  (#177 write-time discipline), the tab binds `Tab::pane_name` (the rail's Panes row now reads
  the NAME; unnamed tabs stay byte-identical "PANE n", hidden under a tab filter while NAMED
  rows participate by name), and a **`[[panes]]` settings entry** persists (the #204 `Vec<T>`
  template + tolerance arms): `name` + `scope` (the project root, #234/#245 keyed-by-root) +
  `axis` + per-cell `kind`/`key`/`ordinal` — a terminal's cwd, an editor's path, and the
  **ordinal** that discriminates same-key instances WITHOUT ever serializing a `ContentId`
  (ids are session-local, #396; #398 twins share an ordinal, two shells in one cwd count up; a
  future kind's cells load, drop at resolve, and round-trip byte-preserved). An EMPTY commit
  UN-names (the #177 rule — the durable entry stays; recipes outlive bindings). **Restart:** the
  name rides the shell codec's `T=` entry as a second `\x1f` slot (`title\x1fname\x1fblob`, an
  empty title slot when untitled — part-count discriminates, legacy wires parse unchanged), so
  the ONE #163 rebuild authority brings the arrangement back exactly once, NAMED — no
  double-build, no zombie resurrection of a deliberately-closed tab. **Reopen-by-name:** dynamic
  **"Pane: {name}"** palette rows (`PANE_ARRANGEMENT_BASE`, the new top id block — the #398 add
  block became band-scoped + capped; scope-filtered to the active project, sanitized display,
  hand-edited duplicates collapse first-wins, rebuilt every open with a by-value dispatch
  snapshot) resolve the entry against the world with the D2 dangling-drop stance — a dead cwd or
  vanished file drops its CELL silently with the rest intact (deliberately NOT the shell
  restore's root-fallback: the arrangement promised THAT cwd), all-dropped skips with a flash —
  then rebuild a NEW named tab through the registry: each instance group spawns/opens once and
  every further cell `acquire_view`s it, so deliberate twins REUNITE as one-instance-many-views
  and editors re-share any already-open Buffer (`resolve_open`). Persisted order survives the
  rebuild (the layout tree's DFS walk is the snapshot order; an inspect-caught id-sort skew is
  now pinned). Reuse extractions along the way: `valid_spawn_dir` (the one #281 spawn-cwd
  predicate), `open_editor_instance` (the one #275 stat-before-read opener — now stat-LENGTH
  gates the read too, refusing an over-cap file without allocating it), `editor_cell` (the one
  lazy CodeView-cell constructor). React-first per the 2026-08-04 policy: the naming flow +
  named rows prototyped and confirmed in marley-web, ported 1:1; the parity pair pixel-sampled
  (the card background is rgb(26,27,31) in both).

- **Add-anything-to-a-pane + the ContentId cross-link — one instance, many views goes LIVE**
  (TICKET-398, forge #398; M28 The Registry Payoff; the #388 Q5 slice-5 — the refcount machinery
  #394 built and #396/#397 populated finally runs in anger). A new verb set VIEWS already-open
  content into a split cell, never a second instance: right-click any rail content row (a
  terminal tab row, a code tab row, a pane cell row, a cross-link row) → the 2-row **AddToPane
  menu** ("Add to Split Right" / "Add to Split Down", `MenuKind::AddToPane` — the target content
  AND its row's project ride the kind, #175-style, and the dispatch activates that project first
  per the #174/#387-F1 idiom); or the ⌘⇧P palette's dynamic **"Add to Split Right: <content>"**
  rows (`ADD_TO_PANE_BASE`, one per open terminal/editor, #241-disambiguated, REBUILT from the
  current registry at every palette open — never appended, the #204/R1 lesson closed: open
  content deletes constantly, and the ContentId snapshot makes a dead row inert rather than
  misdispatched). Every surface funnels through ONE dispatch (`add_content_to_split`):
  `acquire_view` → split the ACTIVE tab's focused pane (terminal cid mounts directly; an editor
  mounts a fresh 1-row view surface — the #397 hit-path idiom, zero IO) → persist. **Visible
  payoff: a terminal added to a split renders ONE session in two cells — typed input lands in
  the one PTY and the block appears in every view, across tabs; an editor added shows the live
  unsaved Buffer.** The cross-link half (#390's deferred slice): Panes CELL rows read their
  CONTENT's label (terminal → the #177/#201 title lineage via a per-frame registry-derived
  `ContentLabels` input, `rail_rows` stays pure; code cell → file basename; "files"/"git diff")
  — never the positional "pane n" (the "PANE n" arrangement header stays positional until #399
  naming) — and home sections cross-link pane-mounted content: a terminal hosted by TWO OR MORE
  tabs marks every hosting tab's row with a muted trailing **⊞** (the post-add "original stays
  open" state — inspect corrected the initial per-cell rule, which would have tripled the
  Terminal section for every ordinary split), while a pane-mounted FILE gets a per-file
  **`RailLevel::CrossRef`** row under Editor (basename + ⊞; the #246 split-file's first-ever
  home entry; click focuses the mounting cell; participates in the tab search by label; the
  focused CodeView cell now owns the ACTIVE section, so a collapse can never hide the focused
  file's only home entry). Lifecycle symmetry hardened while making view_count>1 reachable:
  the content-keyed maps (`agents`/`remotes`/`last_agent`/`notify_ticks`) now scrub exactly on
  the LAST view's release (a twin's close keeps a live agent's Fleet identity), the PTY resize
  pass elects ONE authority cell per content (focused wins — no more twin-cell SIGWINCH fights),
  the pump ticks each content once (the #203 notify threshold stops double-counting), and the
  user-growable workflows palette range is hard-capped at its 1000-id budget (workflow #1001 can
  no longer mint an id that shadows the add rows). Terminal twins share display state
  (viewport/selection/folds — tmux-mirror v1, recorded as D-TERM-VIEW-STATE); a persisted twin
  restores as per-cell instances (D6 — session-local sharing until #399's key rebinding; the
  codec is byte-identical). React-first: the whole flow was built + confirmed in marley-web
  first, and the inspect semantics correction was back-ported there before the Rust landed.

- **Editors move onto the ContentId registry — one Buffer, many views; the #259 two-Buffers
  divergence collapses** (TICKET-397, forge #397; M28 The Registry Payoff; the #388 Q5 slice-3 —
  the registry's first user-VISIBLE payoff). The same file open in the editor tab and a split pane
  is now ONE `EditorInstance` (buffer + undo history + dirty state + the #275 external-change
  snapshot) owned once in `ContentRegistry<Content>` under `Content::Editor`; tab file rows and
  split cells are VIEW rows holding `(ContentId, per-view state)` — render-lines cache (re-keyed
  `(nonce, version)` so a reload's re-mint invalidates EVERY view), IME span, scroll parks (re-keyed
  to a per-row `view_key`), and a parked selection (birth-seeded caret-0; the buffer's live cursor
  set follows focus via a sticky park/restore choke — per-view cursors over per-file buffers).
  **Visible payoff: edits typed in either view appear live in both, one dirty ●, one ⌘Z history
  (either view's undo rewinds the shared edits in order), and splitting a dirty file shows the
  UNSAVED text (never a stale disk read — even when the disk copy has vanished).** The #275
  self-conflict arm ("saving either copy trips the other's banner", #259) is structurally gone —
  one snapshot per file; the `extchange` decision table is untouched. Births route through ONE
  `resolve_open` seam (`(root, same_file)`-scoped construct-or-resolve; an already-open hit does
  ZERO disk IO); every close path releases per view-row, the last release dropping inline (a
  buffer free needs no PTY-style reaper). LSP doc-sync (and the references/search overlay
  snapshots) derive from the registry — one `didOpen` per uri however many views, `didClose`
  exactly on last-view close, split-only files JOIN the sync/overlay sets (previously LSP-dark),
  the reconcile input path-sorted (deterministic). Persistence byte-identical: the `V=`/`c=`
  codecs are untouched, ids never serialize, and a restored tab+split of one path rebuilds one
  instance with two views from ONE disk read (proven by a byte-identity round-trip drive).
  Recorded deltas beyond the headline: same-root projects union into their shared LSP host's open
  set (no more didOpen/didClose churn per project switch); the rename applier edits open buffers
  via the registry (split-only files included; a two-root nested open picks the
  lexicographically-first root deterministically); split-restored files gain a #275 disk snapshot;
  twin views share syntax/fold/symbol memos (one parse per file, was one per copy). Unlocks the
  #398 add-to-pane cross-link and #400 cockpit residency.

- **Terminals move onto the ContentId registry — the load-bearing ownership migration**
  (TICKET-396, forge #396; M28 The Registry Payoff; the #388 Q5 slice-2). Terminal sessions now live
  ONCE in the app-level `ContentRegistry<Content>`; the pane tree holds `ContentId` VIEWS
  (`PaneContent::Terminal(ContentId)`), dissolving the workspace/tabs `<S>` genericity
  (`PaneGrid<S>` → `PaneGrid`; `TerminalPane<S>` remains as the owned payload type). Every birth is
  spawn → `Content::terminal` insert → mount (leak-free by order); every close is tree-remove →
  `release_view` (`#[must_use]`) → one off-thread reap per gesture (the TICKET-348 ~600ms contract
  preserved); restore rebuilds registry entries from persisted shapes — `ContentId` never
  serializes. Side-tables that track the INSTANCE (`agents`, `remotes`, `notify_ticks`,
  `last_agent`, `pending_agent_send`) re-key to `ContentId`; view/surface-keyed state (completion,
  block menu, tab flashes, the MCP pane handle) stays pane-keyed. Zero user-visible change (full
  suite 1989/1989 byte-identical) except three recorded, inspect-sanctioned deltas: Fleet/⌘K rows +
  broadcast fan-out order by launch order (was tab-block order); `mcp_surface_index` no longer
  advertises dead pane handles for auto-reaped exited agents; non-PTY container payloads drop
  inline at close (the reaper contract is PTY-scoped). Unlocks Q5 slice-3 (editors onto the
  registry) and the #398 add-to-pane cross-link.

- **Fold projection reaches the frame-geometry layer — overlay cards anchor correctly below folds**
  (TICKET-352, forge #352; M28 The Registry Payoff; the #305 Slice 2). The #305 known-limit closes: the
  NINE `geom.first`-mixes-buffer-row sites (the hover/completion/signature/rename cards + the IME caret
  rect, the hover-dwell + IME pixel→row inverses, the content-width probe, the inlay-hints fetch window)
  now project through ONE new pure seam — `FoldProjection::viewport_offset(buffer_row, first_slot,
  last_slot)` (slot-domain window check + offset; a row hidden inside a collapsed fold anchors at its
  fold header's slot; an at/past-EOF stale anchor hides rather than saturating) — and the two inverses
  convert their pixel-derived SLOT via the shipped `buffer_row(slot)` (clamps stay in slot domain;
  below the last visible row the raw slot is kept byte-for-byte). No-fold behavior is byte-identical
  (the identity projection short-circuits, parse-free). Inspect found + fixed two adjacent defects: the
  geometry RECORDER's own mount gates compared a buffer row to a slot (`row == first` → `slot == first`
  at both canvases — pre-existing since #305; it froze `editor_geom` whenever a collapsed fold sat
  above the viewport), and `fold_projection()`'s parse arm is now memoized (`FoldProjCache`, keyed on
  nonce/version/anchor-set) because the 16ms pump + mouse-move read the projection — a held fold no
  longer costs a full text+parse per tick. Proven: two new `fold.rs` truth tables + a headless dwell
  drive (1987/1987 green) and a live fixture capture pair — the hover card on a fully-folded file
  anchors at the header's VISUAL row instead of eight rows low. GATE GREEN [diff].

- **The never-empties guards dissolve — a workspace may now be fully empty ("terminate the last
  terminal")** (TICKET-395, forge #395; M27 The Pane Plane; feature). This ends the M10 era where a
  project always held ≥1 terminal tab. `close_tab_refusal` shrinks to **IndexOutOfRange-only**, the
  `TabError::LastTab`/`LastTerminal` variants + their status-flash consumer are removed, and the
  #387-pinned guard tests are **deliberately rewritten** to the new policy (an in-range close always
  succeeds; the sole terminal and the last tab close). An emptied project renders a muted **hint center**
  — *Nothing open in this workspace* · ⌘T New terminal · ⌘P Open file · ＋ Create from a section header —
  with the rail's four **empty section headers** (Editor · Terminal · Panes · Browser). It is **NOT the
  launcher**: an open-but-empty workspace stays open (the #247 launcher remains the zero-WORKSPACE state).
  A persisted empty project **restores empty** — the boot no longer force-seeds a terminal (a new
  `Project::empty`; the `workspace()` ≥1-terminal invariant the force-seed upheld dissolved in #391/#392,
  so the app is total over a zero-tab / no-terminal project). The PTY reaper on close is unchanged. Built
  on #392's crash-safe totality substrate (the `try_active_tab`/`try_workspace` twins); the empty-workspace
  UX was **driven-captured** (verified in pixels).

### Fixed

- **Inlay served-check: store the exclusive end** (TICKET-401, forge #401; the #352 inspect-F5
  find, deliberately preserved bit-for-bit there, deleted here on purpose). The inlay cache stored
  an INCLUSIVE `want_last` (clamped to `len_lines − 1`) as the `Range.end` that the served-check
  compares a genuinely EXCLUSIVE viewport end against — with the file bottom on screen,
  `end_row == len_lines` can never be `≤ len_lines − 1`, so a short/fully-visible file re-sent the
  byte-identical `textDocument/inlayHint` request once per LSP round-trip, forever (waste only —
  hints rendered correctly; the in-flight guard bounded cadence to one per timeout window).
  `apply_inlay_response` now stores `first_row..last_row + 1` at the cache's single write site;
  the wire keeps #331's pinned inclusive params (the strict-server boundary-row residual is the
  spec's recorded Out, with the follow-up shape). Inspect surfaced the adjacent hole the fix would
  have made sticky: a disk reload resets the buffer version epoch while re-minting the nonce, and
  `InlayKey` carries no nonce — a raced pre-reload answer collides numerically and, with the
  refetch-loop gone, would persist; the reload now drops the inlay cache + in-flight key beside
  its #328 git-marks invalidation (forge: BF-…-inlay-reload-version-epoch-collision-001 +
  PR-…-version-epoch-resets-invalidate-numeric-keyed-caches-001). Validation drives the REAL
  mint/send path headless for the first time: `LspHost::set_ready_for_test` replays
  `SpawnOk → InitializeResult` through the real `Lifecycle` (a process-less host's `send_body`
  tolerates the missing child), so "no second request" is pinned as observable state — negative
  smoke proved the pin (reverting the fix flips the unit RED). Floors: cov 100 / MSI 100; 2 new
  headless drives; 2045/2045 workspace-green; GATE GREEN [diff] 15/15 first run.

### Added

- **Internal: the ContentId registry — the pure refcounted content-lifecycle spine of the #388
  pane-composition model** (TICKET-394, forge #394; M27 The Pane Plane; chore, additive). A new gpui-free
  `content_registry.rs`: a `ContentId` newtype + a generic `ContentRegistry<C>` with `insert` (one initial
  view) / `acquire_view` / `release_view` (returns the owned content **exactly** on the last-view drop, for
  off-thread teardown) / `get` / `get_mut` / `view_count` / `len`. The explicit **per-id view-count** is the
  drop-on-last-close gpui's `Entity` gives natively — a bare `HashMap` gives resolution but not refcount
  (the binding catch). **Wired but UNUSED this slice** (the #371 `orchestration_for` `pub use` template
  keeps it gate-green + non-dead-code); terminals migrate onto it in **#396**. This is the **D5 split** of
  the queued spec: the L-sized terminal-**ownership** migration (the `Content` enum + ~46 accessor sites +
  7 reaper-preserving close paths + the persistence rebuild) is #396 — its own reviewed ticket, so a
  half-migrated reaper never ships. The pure lifecycle is proven at coverage + mutation 100% (the boundary
  is airtight — `release_view` never underflows, drops on exactly the last view, and returns the moved-out
  original); `#[must_use]` on `release_view`/`acquire_view` guards the consumer from silently reaping on the
  calling thread.
- **The rail section ＋ becomes a small anchored MENU of that section's create-verbs** (TICKET-393,
  forge #393; M27 The Pane Plane; feature) — chad's "expand that + button to include various things". The
  #387 ＋ dispatched one hardcoded verb; it now opens a section-scoped menu **folded into the ONE #166/#175
  context-menu machinery** (render + keyboard nav + esc/click-away + one-modal all reused, via a new
  `MenuKind::Section` + `MenuAction::Section(SectionAction)` — no parallel popover). Reuse-only v1:
  **Terminal** [New Terminal · New Terminal at Root], **Editor** [Open File… · Open in Split (#246)],
  **Browser** [Forge · Agents · Details], **Panes** [Split Right · Split Down]; **item 0 is the #387
  default** per section (muscle memory). The pure `section_items` table carries cov/MSI 100; the render is
  the byte-identical `menu.items()` loop (zero new render code). The documented **extension point** for the
  future "various things" (SSH/#87, workflows/#204, a URL/Phase-E target, New File) is deferred, not built.
  Every item acts on the ＋'s row project (the #174 activate-first + #387-F1 persist-the-switch rule).
  Inspect fixed two MEDs: the `SplitFocused` arm now persists a cross-project switch even on the
  no-terminal no-op, and a new shared `close_transient_overlays()` helper (factored from the #181 agent
  launcher's 13 closers, now the single source of truth for both mouse-opened modals) stops the menu
  opening keyboard-dead beneath a centered editor picker.
- **Internal: the app is total over a zero-tab AND a no-terminal project (the crash-safe substrate for
  closing the last terminal)** (TICKET-392, forge #392; M27 The Pane Plane; chore, behavior-neutral).
  Completes the totality #391 began. `active_tab()`/`active_tab_mut()` are bare indexes
  (`&self.tabs[self.active]`) that panic on a zero-tab project — reachable the moment #395 dissolves the
  `LastTab` guard. This adds the total twins **`try_active_tab()`/`try_active_tab_mut() -> Option<&(mut)
  Tab<S>>`** (`self.tabs.get(self.active)`), hardens `terminal_grid_index` to `.get()`, and routes the
  ~40 zero-tab / no-terminal-reachable app.rs sites — the render center dispatch (a leading
  `tab_count()==0 → blank center` guard, so nothing bare runs first; #395 fills the hints), the status
  bar, both `key_context()` reads, the terminal raw-key router, the pump tick, and every keystroke-reachable
  handler — through the `try_active_tab`/`try_workspace` twins. The panicking accessors stay for the
  tab/terminal-guaranteed sites. **Zero user-visible change** — the never-empties guards remain in force,
  so a zero-tab / no-terminal project is reachable only in tests and the full suite is byte-identical. The
  D4 split's totality half (chad chose "split it"): #395 flips the guards and ships the empty-workspace UI
  on top of this substrate. Verified by the pure `try_active_tab*`/codec seams at coverage + mutation 100%
  and two adversarial critics (byte-identity + an exhaustive accessor-reachability audit) — the audit
  closed one latent accessor (`activate_search_hit`) that was safe only by a cross-function invariant.
- **Internal: the app can render + persist a project with no terminal (the enabler for closing the last
  terminal)** (TICKET-391, forge #391; M27 The Pane Plane; chore, behavior-neutral). Marley's
  `workspace()`/`workspace_mut()` accessors assume every project holds ≥1 terminal tab (they `.expect()` a
  terminal grid) — the reason the never-empties guards forbid closing the last terminal (#202's ~90-site
  reality). This adds the total twins **`try_workspace()`/`try_workspace_mut() -> Option<&(mut) PaneGrid>`**
  (`None` when the active project has no terminal) and switches the **always-run render/pump/persist** paths
  to them — the render body's focused pane, the editable-surface resync, the PTY-resize loop, `cockpit_body`'s
  Details inspector (now empty when there's no focused terminal), and `persist_grid` (a second guard, mirroring
  the zero-project arm, for a tabs-but-no-terminal project). The panicking accessors stay for the
  terminal-gated call sites. **Zero user-visible change** — the guards remain in force, so a no-terminal
  project is reachable only in tests; the full suite is byte-identical. This is the always-run slice of the
  no-terminal totality (a D4 split — the user-triggered handlers move to #392, which dissolves the guards and
  ships the empty-workspace state). Verified by the pure `terminal_grid_index` predicate + a no-terminal codec
  round-trip + adversarial byte-identity review (the gpui render path is coverage/mutation-excluded).
- **The rail gains a 4th section — Panes — that dynamically collects every split view** (TICKET-390,
  forge #390; M27 The Pane Plane; feature). The left rail is now four fixed-order sections per workspace:
  Editor · Terminal · **Panes** · Browser. Any tab whose pane-grid holds ≥2 cells lists under Panes as a
  **"PANE n"** arrangement row (one per split view, numbered 1-based in tab order), with that tab's pane
  cells nested beneath it — **un-nested** from where they used to sit (under the terminal tab, the #155
  nesting). The split tab keeps its normal row under Terminal too (the origin stays; the Panes row is a
  *view* of it — chad's model A). Clicking an arrangement focuses its tab (the #174 cross-project idiom);
  the section's ＋ splits the focused pane. `RailSection::Panes` is a **derived** section — no tab files
  under it (`rail_section()` never yields it); it's computed by `rail_rows` from grid shape each render, so
  it needs no new persisted state and renumbers on close. The pure `RailSection`/`RailLevel::Arrangement`/
  `section_action` seams carry cov/MSI 100 (57 tabs tests); the render arm is the masked shim (the 4-section
  rail + PANE 1 + the un-nest were confirmed live via a pre-seeded split-terminal restore). This ships the
  **display** half of the #388 pane-composition model early, on (tab,pane) coordinates — the rows gain a
  `ContentId` when #394 lands. Inspect fixed the new render arm missing the sibling row's "Search tabs"
  filter guard.
- **The rail's type sections gain a hover-revealed ＋ that creates under that section** (TICKET-387,
  forge #387; M26 The Sectioned Shell; feature). Each Editor / Terminal / Browser section header now carries
  a ＋ affordance, hidden at rest and revealed on hover (the #217 `group_hover` opacity-0→1 idiom). Clicking
  it dispatches an EXISTING verb only (reuse-only — zero new creation logic) via a new pure `section_action`
  routing table: **Terminal＋** → a new terminal tab (`new_terminal_pane`), **Editor＋** → the ⌘P file finder
  (`start_file_finder`, extracted so the `open-file-finder` command and the ＋ share one entry point — its
  ⌘↵ opens the picked file as a CodeView tab under Editor), **Browser＋** → open-or-switch the Forge cockpit
  (`open_or_switch_cockpit`). The ＋ acts on ITS row's project (the #174 cross-project idiom — switch +
  Files/cwd/branch re-sync) and calls `cx.stop_propagation()` so the click never also fires #386's
  click-to-collapse on the parent header (the hit-target separation, verified against gpui's reverse-bubble
  dispatch). Separately, the never-empties guard is re-expressed as a pure, exhaustively-tested
  `close_tab_refusal` truth table (`close_tab` delegates to it, behaviour-identical): the Terminal-keeps-≥1
  invariant holds exactly as before, while Editor/Browser sections may empty out — and the design's
  "`LastTab` is subsumed by `LastTerminal`" hypothesis was **verified false** (the sole lone-terminal tab
  refuses as `LastTab`, a lone terminal among other tabs as `LastTerminal` — distinct, both reachable). The
  pure seams carry cov/MSI 100; the ＋ render + dispatch is the masked app shim.
- **Design (spike): the embedded-browser model — Forge in the Browser section** (TICKET-389, forge #389;
  M26; docs-only, no product code). A decision-complete architecture doc —
  `docs/marley_architecture/embedded-browser-model.md` — that opens roadmap **Phase E**. Settles the
  substrate: **wry-as-child `WKWebView` for the Forge pane v1** (feasible because gpui 0.2.2 exposes its
  `NSView` via `raw-window-handle` 0.6.2 `HasWindowHandle` — verified at `window.rs:1548`), with the
  native-child **z-order vs gpui overlays** collision named as the load-bearing risk (primary mitigation:
  hide the webview while an overlay is up) plus a keyboard/IME focus-handoff sub-risk, both de-risked by a
  scratch proof-of-embed as train slice 1. **CEF off-screen-render** (composites via gpui's in-tree
  `paint_surface` + `core-video`, brings CDP) is the named revisit; the agent-browser **CDP** lane is a
  separate headless-Chromium substrate (two lanes, not one). Also: Forge-in-browser (URL derived from the
  #381 `.mcp.json` seam, bearer NEVER in the URL/JS, pinned-origin nav, native cockpit coexists), a
  **framing-safe codec** (a whole-tab Browser is a bare `TabLayout::Browser` shell tag mirroring Cockpit's
  `C=`, URL re-derived on restore — never persisted; the deferred split-cell reuses a bare grid `b` leaf),
  and the ordered **Phase-E follow-up train** (proof-of-embed → the codec → the Forge URL pane → nav
  chrome → the CDP lane). The spike ships the design + the plan, not code.
- **Design (spike): the four-section pane-composition model** (TICKET-388, forge #388; M26; docs-only,
  no product code). A decision-complete architecture doc — `docs/marley_architecture/pane-composition-model.md`
  — for chad's `Editor · Terminals · Panes · Browser` rail: content moves behind a **Marley-owned
  `ContentId` registry** (NOT gpui `Entity` — the gpui-free model stays pure), **one-instance-many-views**
  (collapses the #259 two-Buffers dup, with a registry refcount for drop-on-last-close), the **Panes
  section** that dynamically lists split views (the split's origin stays under Editor/Terminals), nameable
  arrangements, and a global cross-workspace section (multi-workspace-gated) — plus the ordered **M27
  refactor train** (8 slices) that implements it. Supersedes #385/#386's pane-nesting-under-tab (the
  un-nesting is a train slice, not an amendment). The spike ships the design + the plan, not code.
- **Rail sections are now interactive — active-section highlight, click-to-collapse with persistence,
  and force-expand-active** (TICKET-386, forge #386; M26 The Sectioned Shell; feature). The #385
  Editor/Terminal/Browser section headers gain: (1) the section holding the active tab carries the
  accent-wash highlight (the #219 `rail_highlight` idiom, one level down from the workspace-centric
  rail-highlight); (2) a ▸/▾ chevron toggles the section's collapse (its tab rows hide), and the
  collapsed set persists across relaunch keyed by project-root + section label
  (`rail.collapsed_sections` — the #245 persist-by-root pattern, index-remapped on project close like
  #236); (3) the ACTIVE section is force-expanded at render regardless of its collapse key, so its
  active tab is never hidden (the #305 auto-reveal invariant, by construction — no activation site
  touches the collapse set, so it holds for palette/keyboard/open-file/＋ alike). The footer's `focus:`
  label keeps "cockpit" for a cockpit tab (a recorded no-op — the footer names the surface's nature;
  the Browser section names its category; they legitimately differ until Forge opens in the Phase-E
  browser). New pure `RailSection { Hash, from_label }` + `collapsed_section_keys` /
  `collapsed_sections_from_keys` / `remap_section_indices_after_remove` (all mirroring #245/#236) carry
  the logic at cov/MSI 100; the render arm (chevron + click + wash) is the masked app shim,
  driven-validated (collapsing a non-active section hid its tabs; the active section stayed expanded +
  highlighted; the collapse persisted to settings).
- **The left rail groups each project's tabs into three fixed-order type sections — Editor / Terminal /
  Browser** (TICKET-385, forge #385; M26 The Sectioned Shell; feature). chad's 2026-07-22 direction: the
  flat per-project tab list becomes a fixed skeleton where opening a thing files its tab under its kind's
  section. A pure `RailSection { Editor, Terminal, Browser }` (with `ALL` fixing the order) + a total
  `TabContent::rail_section()` (CodeView→Editor / Terminal→Terminal / Cockpit→Browser — the cockpit/Forge
  tabs are the Browser section's transitional home until Forge opens in an embedded browser, Phase E) + a
  new `RailLevel::Section` variant; `rail_rows` now emits, per non-collapsed project, each section header
  (even when empty — the skeleton always shows) then the tabs that file under it, in storage order.
  **Display-only:** each tab row keeps its original `Project.tabs()` index (the switch target), so storage
  order, the active-tab index, and the shell/grid codec are byte-identical — the grouping exists only in
  the rail's row stream. Split-terminal panes still nest under their tab row inside the Terminal section.
  Pure seam cov/MSI 100; the render arm (a muted caption between the project and tab indents) is the masked
  app shim, driven-validated (a restored workspace rendered the three ordered headers — the editor under
  Editor, the terminal + its two panes under Terminal, Browser as an empty header). Section interaction
  (active highlight, collapse-persist) is #386; per-section ＋ actions #387.
- **The footer's `focus:` label now names the focused pane's kind, not a hardcoded "terminal"**
  (TICKET-382, forge #382; M25 App-Grade QA Hardening; bug). Found in the 2026-07-21 live QA run
  (`14-editor.png`): clicking into an editor split moved the accent focus border to the editor pane but
  the status bar kept reading `focus: terminal`. The render shim's `footer_focus` was tab-kind-invariant
  — it resolved the focused `PaneId` against the agents/remotes maps then fell to a hardcoded
  `"terminal"`, never consulting the pane's kind; and with a non-terminal tab active it read a BACKGROUND
  grid pane (the `workspace()` first-terminal-grid fallback), which could even leak a background agent's
  label into the footer. Now a pure `status_bar::focus_label` derives the word from a `FocusTab` the shim
  builds by an exhaustive `match` on the active tab: an editor tab → `editor`, a cockpit tab → `cockpit`,
  a terminal tab → the focused pane's identity (agent run / remote host / kind, with a `CodeView` pane
  reading `editor` — the #259 EditorSurface — not `PaneKind::label()`'s "code"). The non-terminal arms
  never read the grid, so the background-agent leak is closed by construction (the `FocusTab::Editor`/
  `Cockpit` variants carry no agent field — the leak is unrepresentable). Both matches are exhaustive
  over their closed enums, so a future pane OR tab kind fails the build into the label rather than
  silently mislabeling. Pure seam cov/MSI 100; proven live (an editor split focused now reads
  `focus: editor`, matching the border). The #201 tab-title fallback is a separate surface, unchanged.
- **The forge `.mcp.json` resolves against the restored active project root — a Finder/`open` launch
  now builds a forge client** (TICKET-381, forge #381; M25 App-Grade QA Hardening; bug). Found in the
  2026-07-21 live QA run: fleet question-pick and dispatch-send recorded "no forge client (.mcp.json)"
  on a machine whose repo HAS the file — because the app was launched via `open`, so
  `std::env::current_dir()` was `/` and the boot's cwd-relative `.mcp.json` read missed. Pre-existing
  since #64 (⌘⇧F's sprint cockpit has been dead under Finder launches from day one); M24 raised the
  stakes — every fleet write and #376's live-wire bearer source silently degraded on the most common
  non-dev launch path. The boot read now resolves via a new pure `mcp_config::mcp_json_path` (active
  project root first, launch cwd as fallback) at the post-shell-restore placement #376 established, so
  a `open`-launched Marley finds the restored project's file; a dev `cargo run` (cwd == active root) is
  byte-identical, and the zero-project launcher boot falls back to cwd = today's behavior. The probe is
  on the FILE (missing-file-only fallback) so a present-but-broken active-root file surfaces as
  misconfigured rather than being masked by an unrelated repo's config. The `marley_forge_client`
  loopback wall + bearer-iff-url-match are untouched — only *which file* is read moved. Pure seam
  cov/MSI 100; proven live end-to-end (a cwd-`/` launch's forge overlay shows "loading sprint…", i.e. a
  client was built, not the "no forge client" degrade).
- **⌘Q now quits Marley — the standard macOS application menu + Quit accelerator**
  (TICKET-380, forge #380; M25 App-Grade QA Hardening; bug). A focused Marley ignored ⌘Q — found in
  the 2026-07-21 live QA run — because the gpui app registered no menu, no `Quit` action, and no
  `cmd-q` binding; only the AppleEvent quit (`osascript … to quit`) worked. `marley_app::run()` now
  declares a `Quit` gpui action (`gpui::actions!(marley, [Quit])`) handled by `cx.quit()`, binds
  `cmd-q` to it, and sets a standard "Marley" application menu with a "Quit Marley" item. gpui derives
  the menu item's ⌘Q key-equivalent from its OWN keymap at `set_menus` time, so BOTH the binding and
  the menu are required and `bind_keys` MUST run before `set_menus` — a menu-only fix would render
  Quit with no ⌘Q. `cx.quit()` funnels through the same `[NSApp terminate:]` route as the AppleEvent
  quit, so the RootView-drop teardown (the #375 discovery-file removal, the #376 fleet-subscription
  stop+join) runs identically on ⌘Q — teardown parity by construction, never `process::exit`.
  Shim-only (the app.rs gpui boot closure; no keymap.rs or RootView change); proven on the running app
  — the live "Quit Marley" item carries ⌘Q (`AXMenuItemCmdChar = "Q"`) and a menu-click exits cleanly
  with zero orphans. Quit-only for now; the Hide/Hide-Others/Show-All trio + About are a follow-up.
- **Build-artifact purge at pipeline end — and cargo-mutants no longer leaks multi-GB temp trees**
  (TICKET-335, forge #335; tooling chore). Two halves: (1) the leak fixed at source —
  `scripts/gates.sh mutation_g` now redirects cargo-mutants' scratch (its tempfile copies honor
  POSIX `TMPDIR`) to a NAMED dir under the system tmp (`marley-mutants-scratch/<pid>` — deliberately
  OUTSIDE the repo: an in-target scratch made every test tempdir repo-ancestored and broke the
  no-`.git`-ancestor discovery test inside every mutant baseline, a critic-proven gate-bricker) and
  trap-cleans exactly this invocation's dir on any exit (INT/TERM exit explicitly so a signal is
  never swallowed); (2) a new `scripts/purge-build-artifacts.sh` reclaiming the verified-rebuildable
  set (coverage/release/incremental/doc/trybuild/mutants scratch + legacy system-tmp orphans) —
  **~19.4 GB reclaimed on first run with `cargo check` still warm at 7.7s** — with `--dry-run`/real
  parity by a single enumeration path, strict args (a misspelled flag refuses instead of silently
  running), a repo-root assertion, a live-guard refusing while any cargo-mutants OR gates.sh runs
  (pids named), and a double deny-guard that rejects `target/debug/deps` by name AND by ancestor.
  Wired into the /commit closeout as an advisory post-commit step (`|| true` — a purge failure can
  never fail a commit). cargo-sweep for the deps cache remains explicitly out of scope (its own
  future decision).
- **`marley_mcp` session TTL sweep — idle sessions expire, closing the #375 slot wedge**
  (TICKET-379, forge #379; M24 ④; chore). A client that initialized but never opened a stream and
  never returned could hold one of the 8 session slots until restart. Sessions now carry a last-seen
  stamp (set at assign, refreshed by the gate on every validated use — injected clock, no
  `SystemTime` in pure code) and a lazy pre-dispatch sweep expires anything idle ≥ 30 minutes
  (`SESSION_TTL_MS`). The sweep runs BEFORE the decision, so an expired id is simply absent — it gets
  the SHIPPED 404 (byte-identical to an unknown id, nothing for a client to distinguish; #373's
  reconnect already recovers via `SessionExpired`). Every #375 invariant holds: post-auth only (an
  unauthenticated request never touches the registry), cap-8 reject-new (fresh sessions are never
  evicted — expiry ≠ eviction), redacted Debug, idempotent terminate; a broken clock fails OPEN to
  the exact #375 behavior (TTL is hygiene, not an auth boundary). Pure registry logic cov/MSI 100
  (a real mutants run: 30 caught, 2 unviable, 0 missed); the #375 tests pass with expectations
  unchanged.
- **Fleet dispatch composer — briefs to seats as mailbox data with live delivery-state chips**
  (TICKET-378, forge #378; M24 Fleet Layer 2 ③; L2 gated-writes ②). Every seat card grows a "send"
  affordance opening an inline draft (the #177/#204 idiom, third instantiation — Esc cancels, Enter
  confirms, the draft owns the keyboard); a non-blank confirm dispatches exactly one receipted
  `session.send` `tools/call` whose arguments ARE the shipped `marley_fleet::SendRequest {id, text}`
  serialized verbatim — the FROZEN proposed v1 contract, fixture-recorded for Layer 0. The brief is
  mailbox DATA on the control plane: no code path can reach a PTY (the evidence-night
  dispatch-by-keystroke failure class this ticket exists to kill). Delivery truth renders as a chip
  driven ONLY by the shipped `DeliveryState` machine: an Accepted receipt seeds `deposited`, and the
  proposed `dispatch-claimed`/`dispatch-started` seat_events echoes advance it monotonically —
  projected adapter-side into a per-seat max-join queue on the subscription handle (bounded by fleet
  size even when the launcher parks the drain; lossless because the consumer folds the same max). A
  failed/refused send surfaces its clamped reason and PRESERVES the brief — the failed chip re-opens
  the composer prefilled. Inspect (2 critics) caught the unbounded echo queue and an UNHEALABLE
  stuck-at-deposited case (a terminal `dispatch-started` echo dropped while the receipt was in
  flight never heals — echoes observed mid-flight now buffer in the record and max-join at receipt
  time), plus the write-doctrine doc undercount (now a QUARTET). Pure seams cov/MSI 100; real-socket
  contract/refusal/echo-drain scenarios + a headless composer drive; glue masked.
- **Fleet structured interrupts — the Waiting seat's question is answerable (receipted `session.answer`)**
  (TICKET-377, forge #377; M24 Fleet Layer 2 ②; the first L2 gated write — the in-shell slice of the
  mission-control AskUserQuestion loop). The #369 question card's inert option chips are now PICKABLE:
  a pick dispatches exactly one receipted MCP `tools/call` — contract-first, the FROZEN proposed v1
  shape `session.answer {session_id, choice, prompt}` (choice = the option string the human saw,
  verbatim; prompt = the question's identity so the brain can REFUSE a stale answer), fixture-recorded
  as the Layer-0 handoff literal. A per-seat local overlay (Pending → Answered / Failed-with-reason)
  renders in the card until the seat's own event stream flips it — the reducer stays the sole producer
  of snapshot truth (no optimistic forgery); a failure/refusal surfaces its clamped reason and re-arms
  the card. Answers are control-plane DATA, never keystrokes (the press-digit-4-at-a-screenshot
  incident this plane exists to kill). Inspect (2 critics, converging independently) caught the
  question-identity hole at every hop pre-commit: the rendered question is now captured at the chip and
  equality-gated at dispatch, each pick carries a nonce so a superseded send's late result can never
  overwrite a newer outcome, and the prompt rides the wire. Pure seams (the answer machine, the
  builder/ack lane) cov/MSI 100; three real-socket fixture scenarios + a headless machine drive; glue
  masked.
- **Fleet rail live wire — the app starts `FleetSubscription` from `[[projects.orchestration]]`**
  (TICKET-376, forge #376; M24 Fleet Layer 2 ①; closes orchestration-shell §12.1 precondition 1 — the
  "DISCOVERED GAP" from the M23.5 round). At boot, when the restored ACTIVE project's root has an
  orchestration entry naming a loopback `brain_endpoint`, the app starts exactly one self-healing fleet
  subscription (the shipped #368/#372/#373 pump) and the right-dock rail renders live `FleetSnapshot`s;
  the dock header carries the TYPED connection state (`Fleet · live` / `Fleet · reconnecting` — never
  inferred from snapshot age; plain `Fleet` when unconfigured, byte-identical to #369). New pure seams:
  `fleet_live::subscription_target` (exact-root start-gate), `fleet_header_title`,
  `demo_feed_permitted` (the demo verb is gated off while live), `fleet_cursor_dir` (a per-endpoint
  cursor home — sanitized url + FNV discriminator — so two brains sharing a config dir can't poison
  each other's replay window), and `marley_forge_client::endpoint_for_brain` (the same loopback wall as
  `.mcp.json` parsing; the bearer rides ONLY when the configured url string-equals the `.mcp.json`
  forge url, else empty — a credential is never sent to a service it wasn't issued for). The
  subscription publishes a typed `ConnectionState` (Live only once the standing GET's own status line
  verifies — a write success is not an established stream), drains through the existing pump
  (latest-wins cell, deposit-only callback, dirty-on-change), and stops+joins on quit via the RootView
  drop. A transient disconnect never clears the rail (the #373 carry-forward, now proven at the app
  seam by a new `tests/fleet_livewire.rs` real-socket lane: converge/Live, forge-death→Reconnecting
  with the snapshot retained, bounded stop, and a strict-delta union across an in-fixture reconnect).
  Inspect caught the skip-detach trap's 5th strike (re-bound; both pump shims 0-mutant), the
  boot-root-before-restore mis-keying (the gate now runs AFTER the shell restore), and the premature
  Live publish. Pure seams cov/MSI 100; glue masked.
- **`marley_mcp` server hardening — CSPRNG bearer, 0600 discovery file, per-session `Mcp-Session-Id`**
  (TICKET-375, forge #375; M23.5 Fleet Layer-1 Consolidation ④; the #370 security/protocol follow-ups). The
  loopback expose server is hardened for the more-than-one-local-client + hostile-local-process threat
  models: (a) the per-boot bearer is now minted from 128 bits of OS CSPRNG (`/dev/urandom`, no new dep)
  instead of the guessable `marley-{port}-{salt}` derivation — the server REFUSES to start (typed error, app
  stays up) if entropy is unavailable rather than falling back to anything weaker, and the bearer/session
  compares are constant-time-ish (no early content exit); (b) the discovery file (`mcp-endpoint.json`, which
  carries the bearer) is written owner-only (0600 via an fchmod on the open handle) and removed on clean
  shutdown (a `Drop` on the host) so a stale bearer never lingers; (c) the MCP `Mcp-Session-Id` lifecycle is
  implemented server-side — a fresh id on the initialize response, echoed on every later request (400 when
  missing, 404 when unknown/terminated → the client re-initializes), HTTP DELETE terminates, and a bounded
  registry (cap 8, reject-new-never-evict) that reclaims a session when its standing stream drops so
  reconnect churn can't wedge the cap. The pre-dispatch guard order is preserved and extended:
  cap → Origin → bearer → session → dispatch. Pure seams (hex formatter, constant-time compare, session
  registry + gate, discovery IO) at cov/MSI 100; the socket/entropy/Drop wiring stays masked.
- **`[mcp.expose]` — operator grants now govern the live MCP server** (TICKET-374, forge #374; M23.5 Fleet
  Layer-1 Consolidation ③; closes #371's S3 loop). #371 shipped the `[[mcp.servers]]` grants + the pure
  `grants() → GrantTable` accessor, but the `mcp-serve` server still ran the hardcoded deny-by-default
  default — so `session.surface_to_human` was undeniably denied for every operator. This adds the missing
  hop: a NEW singleton `[mcp.expose]` settings table (`marley_mcp::ExposeConfig { allow, allow_write }`)
  whose `allow_write` classes build the `GrantTable` the running loopback server enforces. The singleton
  form is deliberate — a transport-less "us" row in `[[mcp.servers]]` would resolve to the `NoTransport`
  error, so the expose grants live in their own `[mcp.expose]` table (coexisting with `[[mcp.servers]]`
  under `[mcp]`), Marley's first struct-valued setting. An ABSENT table yields `GrantTable::default()`
  byte-for-byte, so an unconfigured Marley behaves exactly as before (deny-by-default preserved), and a
  malformed table or field fails CLOSED to the empty default (never a panic, never a loosened grant).
  Grant resolution is pure (cov/MSI 100); the one-line `start_mcp_server` callsite swap is the masked glue.
- **Fleet subscription — auto-reconnect + jittered backoff (self-healing pump)** (TICKET-373, forge #373;
  M23.5 Fleet Layer-1 Consolidation ②; closes the #368 inspect LOW). A transient forge bounce no longer
  silently kills live fleet updates: the subscription thread now wraps its single-attempt cycle in an
  UNTIL-STOPPED reconnect loop — every cause except a clean stop (a closed stream, a hard disconnect, a 404
  session-expiry, an unreachable forge) waits out a full-jitter exponential backoff (500ms → 30s cap, the
  AWS full-jitter variant) then re-initializes, replays from the durable cursor, re-subscribes, and
  resumes. A NEW pure `backoff` module (`should_retry`/`next_attempt`/`next_delay_ms`/`wait_slice_ms`, cov/
  MSI 100) holds the decisions with an injected clock + jitter (no `rand` dep); the masked pump supplies the
  ambient clock and owns the attempt counter. The reconnect is LOSSLESS even against a strict post-cursor
  DELTA feed: the accumulated `FleetSnapshot` is carried forward across cycles (an inspect catch — rebuilding
  from empty would drop every seat introduced before the cursor). The wait polls the stop flag at ≤1s
  granularity, so `stop()`/drop is honored promptly mid-backoff. Retries are until-stopped (a monitoring
  surface must not silently give up); the #369 staleness indicator communicates the degradation meanwhile.
  #372's fixture harness grew reconnect/backoff/stop-mid-wait scenarios (12 total).
- **Fleet subscription — live-wire READINESS: a real-socket integration harness + typed exit signals**
  (TICKET-372, forge #372; M23.5 Fleet Layer-1 Consolidation ①; the #368 named follow-up, respec'd). The
  #368 pump shipped ACCEPTED-UNTESTABLE (a fake-feed harness its only proof) because Layer 0 (the ucsosv2
  seat-emitter + the real `fleet://events` resource) does not exist yet. This converts it into
  integration-tested behavior WITHOUT Layer 0: a test-only loopback fixture forge server
  (`std::net::TcpListener` + thread-per-connection, no async runtime — the workspace's first) speaks the
  pump's ACTUAL dialect and drives the REAL `FleetSubscription` pump over a REAL socket, replaying the
  frozen #368 wire contract; 11 scenarios prove the MCP lifecycle order, the `Mcp-Session-Id` echo, the
  cursor'd replay-before-subscribe, byte-split invariance over ≥3 TCP segments, snapshot-equality with the
  pure `FleetSync` path, restart-from-persisted-cursor, and a clean mid-stream close with NO retry. The
  pump now returns a typed `SubscriptionExit` (`Stopped`/`StreamClosed`/`Disconnected`/`SessionExpired`/
  `HandshakeFailed`) via a poll-able `exit_reason()` — the signal #373's reconnect policy will consume — and
  the read-loop disposition + the 404→session-expired classification are lifted into pure, cov/MSI-100
  seams (`read_step`, `classify_http_failure`, `classify_if_failure`, `http_status_code`). Closes a latent
  gap: a non-2xx standing-GET response now surfaces a typed reason instead of idling forever. A "Going live
  (L0-day) runbook" lands in `orchestration-shell.md` §12.1 (settings row + honest preconditions + the
  (a)–(e) verification checklist), so L0-day is config + verify. The masked socket/thread pump stays
  `mutants::skip` + coverage-excluded; the literal real-ucsosv2 run is a tiny ticket minted at L0-land.
- **Settings — `[[mcp.servers]]` + `[[projects.orchestration]]` config tables (the fleet control plane's
  wiring layer)** (TICKET-371, forge #371; M23, ticket ⑤ — completing the Layer-1 sequence). Two
  hand-editable settings tables on the shipped `[[lsp.servers]]`/`[[workflows]]`/`[[remote.hosts]]` idiom:
  `[[mcp.servers]]` (`marley_mcp::McpServerConfig` — name · stdio `command`/`args` OR http `url` · `enabled`
  · `allow`/`allow_write` tool-class grants) with a PURE transport resolver (command→Stdio / url→Http /
  neither-or-both→a typed PER-ENTRY error, so one malformed entry never wipes the table) and a `grants()`→
  `GrantTable` accessor — closing #370's S1 loop (#370 OWNS the permission type; #371's config FILLS it);
  and `[[projects.orchestration]]` (root · `brain_endpoint` for #368 · `web_url` for the Phase-E pane) keyed
  by project root with an `orchestration_for` resolver. Every non-identity field is `#[serde(default)]` and
  `enabled` hand-defaults TRUE (a hand-`Default` pinned to the serde decode), so a hand-edited entry omitting
  a key loads with its default rather than dropping the table (the #204 lesson). Generic vocabulary
  throughout (no Forge/UCSOS string — a non-Forge project points everything anywhere). WIRING ONLY, no live
  connection (#368/#370 consume it); pure seam cov/MSI 100. Follow-up: wire a chosen server's grants into
  #370's `start_mcp_server`.
- **`marley_mcp` — Marley's EXPOSE-side MCP server (the control plane's first server surface)** (TICKET-370,
  forge #370; M23, ticket ④ of the Layer-1 fleet train — insight before control). A NEW serde-only crate
  (hand-rolled, no rmcp — the L1 surface is `initialize` + 5 methods + 1 notification, so a second async
  runtime in a gpui app isn't warranted): a PURE protocol core — JSON-RPC 2.0 dispatch, a `(family, verb)`
  tool registry (one `const` source of truth; `fleet` read + `session` write), deny-by-default permission
  tiers (read = loose; writes = an explicit per-tool-class grant, and marley_mcp OWNS the `GrantTable` that
  #371 deserializes into), the `fleet.snapshot` read tool + a subscribable `fleet://snapshot` resource
  (#367's `FleetSnapshot` IS the schema — one seam, three consumers), and the ONE gated write
  `session.surface_to_human(id)` (resolve the id → focus the pane; unknown → a typed `isError` refusal) —
  all at cov/MSI 100 — plus a MASKED `std::net` transport shim (loopback-only bind, `Origin` + bearer guards
  run BEFORE dispatch, a per-boot bearer never logged, a discovery file the manager's `.mcp.json` reads).
  The app hosts it via a `mcp-serve` verb (opt-in) + the pump-fed `mcp_host` glue, executing the surface
  effect through the shell's `jump_to_pane`. A manager seat (an MCP client, location-independent) can now
  read the fleet and surface sessions to the human. Inspect caught + fixed a pre-auth `Content-Length` OOM
  DoS, a `tools_list` drift, dead code, and made the subscribe/SSE model honest. Follow-ups: an
  `Mcp-Session-Id`-keyed per-resource subscription (L1 pushes the one self-healing resource unconditionally),
  a CSPRNG bearer, the fleet-id→local-pane bridge (Layer-0), and extracting the shared `is_loopback` /
  protocol-version predicate into `marley_fleet`.
- **The shell grows its first fleet surface — a READ-ONLY fleet control-plane rail** (TICKET-369,
  forge #369; M23, ticket ③ of the Layer-1 fleet train — insight before control). The dormant RIGHT
  dock slot (idle since the #153 cockpit move to full-screen tabs) is revived to host a rail rendering
  `marley_fleet::FleetSnapshot`: per-seat cards with a state chip from the CLOSED vocabulary
  (Starting/Working/Idle/Waiting/Error/Done — **Error is visually distinct from Idle**, so the
  dead-vs-idle blindness dies here), a subtle transport hint, opaque label chips rendered GENERICALLY
  (Marley renders the strings, never interprets them — a Forge-specific string in the render path is a
  defect), a Waiting seat's `Question` as a RENDER-ONLY card (answering is a Layer-2 receipted verb —
  inert options + an honest "answered in Layer 2" hint), staleness dimming (`marley_fleet::is_stale`
  against a rail-owned `STALE_AFTER_MS` = 30s) + a `last_event` relative time, seats in first-seen order
  VERBATIM (never re-sorted — prominence is per-seat treatment, not reordering), and a non-blank empty
  state. A new `toggle-fleet-dock` verb opens the slot (session-only in v1; ⌘⇧B still opens the Details
  cockpit tab, untouched) and a `fleet-demo-feed` fixture verb installs a demo snapshot for the headless
  drives. Every render DECISION is a pure fn in the new `crates/marley_app/src/fleet_rail.rs` (cov/MSI
  100 — 26 mutants: 22 caught, 4 unviable `Default`-body on no-`Default` types); the app.rs render arm is
  a `mutants::skip` shim (the #307 pattern) proven by two headless drives that EXECUTE `fleet_rail_body`
  for both the empty and the populated snapshot. Depends on #367 (the pure `marley_fleet` crate) and
  re-derives nothing. Follow-ups: an attention-SORTED rail, dock-open persistence (a `DockRight` setting),
  overflow wheel-scroll, and unifying the twin "Fleet" surfaces (this control-plane rail vs. the #68
  local-launched-agents ⌘⇧E overlay).
- **`marley_forge_client` grows into Adapter #1 — the fleet MCP subscription + `seat_events`→envelope
  projection** (TICKET-368, forge #368; M23, ticket ② of the Layer-1 fleet train; contract-first, live
  wire pending Layer 0). The crate gains a pure `fleet` module: SSE multi-event framing (generalizing
  the single-`data:`-line parser); the MCP resource-subscription request builders + response parsers
  (`initialize` → `notifications/initialized` → `resources/subscribe` → `resources/read` → react to
  `notifications/resources/updated`), with the bearer confined to lib.rs's HTTP framing; the durable
  cursor/replay via `FleetSync` (project a page → `marley_fleet::reduce` → advance the cursor); and the
  pure, table-driven projection of `seat_events` rows onto `marley_fleet`'s generic
  `SessionEvent`/`Session`. The **proposed v1 wire contract** (the fake-feed fixtures are the contract
  handed to Layer 0): a single cursor'd durable log `fleet://events?since=<cursor>` returning a
  `FleetPage { cursor, events: [SeatEventRow] }`, each row a self-contained descriptor + trigger with
  `ts_ms: u64` epoch-millis (no date dependency). The projection routes every descriptor-bearing kind
  through an `Upsert` (state + opaque labels together); a `session-end` carrying `state: error` surfaces
  `Error` (retained) rather than laundering a crashed seat to a clean `Done` (a safety-critical inspect
  fix); a `halted-with-question` emits `[Upsert(Waiting), QuestionRaised]` so the seat keeps its chips;
  an unknown/future kind degrades to a `Heartbeat` (never dropped). Push is an MCP resource subscription
  — Marley never opens a Postgres connection. The live standing-SSE socket pump is masked
  (accepted-untestable, exercised on the live wire); the pure seams are cov/MSI 100 (108/108 viable
  mutants killed). A 4-critic adversarial inspect caught 11 real issues (1 High, 5 Med, 5 Low) + an
  MSI-blocking equivalent mutant — `BF-…-session-end-error-launders-to-done-001`,
  `PR-…-project-rich-row-through-descriptor-event-honor-terminal-state-001`,
  `PR-…-redundant-arithmetic-is-an-equivalent-mutant-001`.

- **`marley_fleet` — the generic fleet `Session` envelope + reducer (the M23 Layer-1 foundation)** (TICKET-367,
  forge #367; M23, ticket ① of the fleet control plane's Layer 1). A NEW pure crate (deps = `serde` only; no
  gpui, no transport, no Forge/UCSOS vocabulary — a project-specific string here is a defect, the projection
  lives in `marley_forge_client`). Carries: the generic `Session` envelope (stable `id`; the closed `State`
  vocabulary `Starting|Working|Idle|Waiting|Error|Done` with `Error ≠ Idle` first-class; `Option<Question>`
  present ⇒ `Waiting` — reducer-enforced, one-directional; opaque `BTreeMap` labels rendered uninterpreted;
  `last_event_ms` epoch-millis; an `Option<Transport>` render hint); the `FleetSnapshot` + a pure, replay-safe,
  idempotent reducer (`SessionEvent` stream → snapshot; a non-`Upsert` event for an unknown id auto-vivifies a
  placeholder — a live seat is never dropped; `Ended` never launders `Error` to `Done`; a `max`-join
  `last_event_ms` tolerant of clock skew); read-time staleness/attention derivation (no clock — `now_ms` +
  `stale_after_ms` injected; attention ranks `Error` → `Waiting`-with-question → `Stale`, stable within-reason
  order); the `Deposited→Claimed→Started` delivery monotone-join types; and the
  `session.send`/`read`/`open`/`surface_to_human` request + generic `Receipt<T>` receipt types as serde data —
  one seam, three consumers (the fleet rail #369, the adapter projection #368, the MCP tool schema #370).
  cov/MSI 100 (33/33 viable mutants killed, 5 unviable auto-excluded); zero exclusions. The inspect caught 5
  viable mutants (B1–B5) that would have survived the tests the EARS rows literally describe → prevention rule
  `PR-claude-ears-behavior-covered-mutant-still-alive-on-sibling-line-001`.
- **Editor-feature parity in a focused editable split pane is verified + guarded** (TICKET-355, forge #355;
  M15 → ships M22, the #259 "slice 2"). Investigation found the parity was ALREADY delivered by #259 (focus-aware
  `active_editor()`/`active_editor_mut()` + `editor_geom` written by the focused pane) + the top-level focus-aware
  overlays (find bar, completion/hover/signature/rename popups, def/refs/code-action/symbol pickers — none
  tab-gated; #259 removed every `active_tab().editor()` chain) + #357 (the banner) + #356 (the read-only sync).
  Three headless DRIVES now prove + guard the contract — ⌘⇧O go-to-symbol, ⌘F find, and ⌥⌘[ fold all operate on
  the FOCUSED split pane's surface — so a regression to tab-only focus would fail. The LSP-server features
  (hover/completion/def/refs/rename/signature) ride the same `active_editor()`+`editor_geom` mechanism (a live
  server to drive is #320).
- **The external-change conflict banner now shows on a focused editable split pane, not only the editor tab**
  (TICKET-357, forge #357; M15 → ships M22, a #259/#275 follow-up). An external change/delete of a focused split
  pane's file arms an `ExtConflict` (the #259 focus-aware check), but the #275 Keep-mine/Reload banner rendered
  only in the editor-tab block, so the split pane showed the dirty ● with no banner (no data loss — `save_active`
  re-checks/arms/flashes before writing). The inline banner is extracted to a reusable `ext_conflict_banner`
  builder and rendered on the focused split pane too; its buttons already act on `active_editor()` (the focused
  surface). Byte-equivalent extraction — the editor-tab banner is unchanged.
- **`appearance.font_family` warns when the font resolves but is NOT monospace** (TICKET-361, forge #361; M22, a
  #344/#337 follow-up). #344 already fell back + warned when a family did not resolve, but a family that resolves
  yet is PROPORTIONAL (e.g. `Helvetica` / `Arial`) passed silently and broke the terminal grid's column
  alignment. Now `new_in` also probes monospace-ness at boot: `font_is_monospace` compares gpui's per-character
  advance of `i` vs `m` (`TextSystem::advance`), gated on the resolve check first (`resolve_font` panics on a
  missing font). If the two advances differ beyond a tolerant, size-independent 5% ratio the family is treated
  like an unresolvable one — the built-in mono `Menlo` is applied and a `status_flash` names the family + "is not
  monospace". The decision is a pure `resolve_font_family` arm + `is_monospace_advance` (cov/MSI 100 in
  settings.rs); the gpui metric read is a boot shim (`mutants::skip`, like #344's `font_resolves`). The resolvable
  case is not headless-testable (gpui's test `NoopTextSystem` resolves no real font), so the decision is proven by
  pure units and the metric read is headed-only.
- **Regex Replace expands capture groups — `$1` / `${name}` / `$$`** (TICKET-347, forge #347; M22). The find
  bar's regex Replace now substitutes captures into the replacement, so `(\w+)@(\w+)` with `${2}_${1}` turns
  `ab@cd` into `cd_ab`. The expansion is the `regex` crate's own `Captures::expand` (adopted, not re-implemented):
  unbraced `$ref`, braced `${ref}`, an invalid ref → the empty string, `$$` → a literal `$`. A pure
  `find::replace_all_regex` re-runs the compiled pattern at replace time — the read path keeps only match offsets
  and never holds the compiled regex, and a `Captures` borrows its haystack, so re-running is the honest way to
  reach the groups — and returns `(start, end, expanded)` per match through the same single byte→char walk the
  find path uses; `find::replace_all_with` applies them back-to-front in one undo group. Two silent-corruption
  bugs that only became reachable once captures landed are fixed at the same seam, via a pure `find::resume_after`
  (extracted so the arithmetic is mutation-tested rather than trapped in the coverage-excluded key handler):
  **Replace-One's caret and its resume cursor now come from the inserted (expanded) text's length, not the
  template's** — a capture template that expands longer or shorter than its own source no longer skips or
  re-hits following matches — and **a zero-width match whose expansion is empty now advances by one** instead of
  re-selecting itself. Literal (non-regex) Replace is byte-for-byte unchanged.
- **Auto-close is syntax-aware — no pairing inside strings or comments** (TICKET-346, forge #346; M22). Typing a
  bracket or quote with the caret inside a `"string"` or a `// comment` now inserts only that character, instead
  of sprouting a matching partner in the middle of your text. The judgement is the tree-sitter tree's, never a Typing a
  bracket or quote with the caret inside a `"string"` or a `// comment` now inserts only that character, instead
  of sprouting a matching partner in the middle of your text. The judgement is the tree-sitter tree's, never a
  regex: a thin `marley_syntax::node_kind_at` probe reads the node kind at the caret, a pure name-based
  `auto_close::context_from_node_kind` classifier maps it to a `Context` (`Code` vs `StringOrComment`), and
  `pair_action` gained a `context` parameter — whose `Code` arm is **byte-identical to #338** (the 26k-row sweep,
  re-threaded with `Context::Code`, proves it). The probe is gated to opener keystrokes only, so ordinary typing
  never pays a reparse. `marley_editor` stays independent of `marley_syntax` (the classifier is Lang-free); the
  app wires the two. The `'` lifetime-bound positions (`T: 'a`, `+ 'a`, `'static`) remain a documented deferral —
  the tree is parsed from the pre-insert text, so there is no `lifetime` node at the caret while the `'` is being
  typed; that cut needs ancestry detection and is tracked as a follow-up. #338's one-char `blocked_quote` guard is
  unchanged.
- **`appearance.font_family` warns when it doesn't resolve** (TICKET-344, forge #344; M22). An unresolvable
  family used to degrade SILENTLY to gpui's system substitute — which may be PROPORTIONAL, breaking the terminal
  grid's column alignment — with nothing to explain it (the #337 REQ-006 cut). #344 probes the requested family
  ONCE at boot and, on a non-resolve, falls back to the built-in mono `TERMINAL_FONT` ("Menlo") EXPLICITLY and
  sets a `status_flash` naming the family. The probe is gpui's PUBLIC `TextSystem::all_font_names()` membership —
  NOT the `TextSystem::font_id` the ticket claimed (that method is private and uncallable), and NOT `resolve_font`
  (it PANICS on a fully-unresolvable font, unsafe at boot). A pure `settings::resolve_font_family` policy
  (cov/MSI 100) decides the applied family + the warning; the `new_in` boot ctor does the probe and wires it.
  Drawn out because the probe + fallback policy + surface is its own design + tests — the honest reason #337
  deferred it. A resolvable font applies silently; an empty setting is the built-in. (Follow-up #361: warn when
  a font RESOLVES but isn't monospace — a separate metric-introspection feature.)
- **The editor's horizontal scrollbar thumb** (TICKET-341, forge #341; M22). #336 shipped horizontal scroll
  (wheel + caret-follow + click) but deferred the visible thumb (REQ-008); this adds it, display-only — a
  bottom-edge bar over the code column whose left + width come from a pure `h_scroll::h_thumb` fraction, hidden
  when the code fits. Three things make it correct where the deferral thought it was hard:
  - **The denominator is the VIRTUAL scroll extent `viewport_px + max_scroll_x`, not `content_px`.** #336 adds
    two cells of overscroll slack to `max_scroll_x`, so `scroll_x` ranges past `content − viewport`; a `content_px`
    denominator would push the thumb off the track's right edge at full scroll. Over the virtual extent,
    `left_fraction + width_fraction == 1.0` exactly at `scroll_x == max_scroll_x` — the thumb reaches the edge
    precisely when fully scrolled. Reuses the #198 render pattern (`MIN_THUMB_PX` floor) mirrored horizontally.
  - **The body-relative origin is DERIVED, not probed.** The ticket's premise — that a thumb over the code column
    needs a THIRD body-relative geometry probe (because `geom.x0` is a window coordinate that rides the scroll) —
    dissolved: `h_scroll::code_area_left_px` = the git lane (3px) + two row gaps + the gutter's digit columns,
    reading the SAME `ROW_GAP_PX` the code row lays out with (the row's `.gap_2()` is now `.gap(px(ROW_GAP_PX))`,
    single-source), so the thumb aligns by construction — the values were already in render scope.
  - **The pure seam (`h_thumb`, `code_area_left_px`) is cov/MSI 100** (32/32 diff mutants caught); the render is a
    `code_view_body` shim (coverage-excluded, `mutants::skip`). The `None` guard compares the UNFLOORED `view` and
    `over = content − viewport` against 0 so its boundary mutants stay killable, and hides the thumb pre-first-frame
    (`code_w == 0`, which lags the synchronously-set content width by one frame). Drag-to-scroll remains a
    follow-up — zero precedent in the codebase (#198 deferred it too). Verified headlessly (the geometry is a pure
    assert + alignment is correct-by-construction; live pixel deferred — the machine was in use).

### Changed

- **Selection-ladder (⌃W) and sticky-headers now reuse the cached syntax tree instead of reparsing**
  (TICKET-363, forge #363; M22, performance — the deferred half of #349). #349 cached the worker's tree and
  routed bracket-match through it, but two other consumers still parsed a throwaway `HighlightSession` per query:
  `step_selection_ladder` (#329, ⌃W/⌃⇧W grow/shrink) and `refresh_sticky_headers` (#330, on scroll). #363 mirrors
  #349 exactly — two new pure parse-free factors `marley_syntax::enclosing_ranges_from(&Tree, byte_range)` +
  `all_headers_from(&Tree)` (the session-taking `enclosing_ranges`/`all_headers` now delegate to them, behavior-
  identical, their tests unchanged) — and both callers read the app's `tree_cache` on an EXACT `(nonce, version)`
  hit → the `_from` variant (microseconds, no reparse), else the byte-identical session-parse fallback (the same
  exact-AND guard as `refresh_bracket_match`; both fields load-bearing — a reload resets version to 0 AND re-mints
  the nonce). No behavior changed: `enclosing_ranges_from == enclosing_ranges` and `all_headers_from ==
  all_headers` are pinned by equivalence + hardcoded-vector units (an equivalence-only unit is tautological for
  the factor's own mutants, so each also asserts `_from` against a literal). Both reroutes sit inside the already
  `mutants::skip`'d caller fns, so they add no mutation surface; the pure factors carry cov/MSI 100.
- **⇧Tab: two cursors inside one row's indentation merge to one, and that is documented as correct**
  (TICKET-358, forge #358; M22, editor/multi-cursor). When two cursors both sit inside the same row's leading
  whitespace and ⇧Tab dedents that row, the whitespace they occupied is removed — so the two cursors collapse
  to a single one at the new line start. There is nowhere distinct for a second cursor to go once its columns
  are deleted, so this is correct, not a lost cursor; and because a dedent is an EDIT (unlike the #297 motion
  scar), one ⌘Z restores the whole cursor set. No behavior changed — the ticket's proposed "clamp differently"
  fix turned out to be a no-op against the real rebase (an in-indent cursor already clamps to the line start
  regardless). The behavior is now pinned by headless drives: the merge, the ⌘Z restore, and a guard that
  cursors on separate rows all survive. A #307 follow-up.
- **Bracket-match reuses the cached syntax tree instead of reparsing on every caret move** (TICKET-349,
  forge #349; M22, performance). The off-thread syntax worker already parses a tree-sitter tree for
  highlighting; it now hands that tree back to the app (a cheap `ts_tree_copy` clone), which caches it keyed
  by `(nonce, version)` exactly like the span cache. When the caret settles on a delimiter and the cached
  tree's key matches the live buffer, bracket-match answers from the cached tree in microseconds (the new
  pure `marley_syntax::matching_delimiters_from`) instead of reparsing the whole file; any miss — a file below
  the async threshold, or an edit that bumped the version before the worker re-sent — falls back to the full
  parse (`matching_delimiters_in`), byte-for-byte the prior behavior. The profiling spike that gated this
  measured the old per-caret reparse at ~3.4 ms for 1.8k lines, ~8.6 ms for 4.5k, and ~19 ms for 10k — over a
  60 fps frame at 10k lines, which is where a held arrow dropped frames; 2k–5k files were already sub-frame.
  The exact-AND key guard is load-bearing in both fields: a reload resets a buffer's version to 0 *and*
  re-mints its nonce, so neither alone identifies the entry. Routing the selection-ladder (#329) and
  sticky-headers (#330) through the same cache is a filed follow-up.
- **The mutation gate (gate:5) stops laundering slow tests into MSI 100** (TICKET-345, forge #345; M22).
  `scripts/gates.sh` counts a `Timeout` mutant as caught — sound when the timeout means "the mutant
  caused a hang" (the Infection/Stryker convention: a hang IS detection), but NOT when it means "the
  suite was slow today", which silently launders a genuinely undetected mutant into MSI 100 — the one
  thing the floor exists to make impossible. #337's run had three Timeouts; every one was killed
  decisively by a failing assert and mislabeled only because plain `cargo test` doesn't fail-fast and
  the wall clock blew cargo-mutants' deadline (a retained `mutants.out.old` still holds a live case:
  the `fold.rs:75 delete-!` mutant labeled `Timeout`, its log showing `... FAILED`). Two changes make
  the classifier honest:
  - **Mutants now run under nextest** (`--test-tool=nextest` — fail-fast, process-per-test), so a mutant
    killed by a failing assert is detected in seconds instead of riding the whole suite out to the
    deadline. This converges gate:5 onto the runner gate:3 has used since inception (nextest + a separate
    `cargo test --doc`); it needs no `.config/nextest.toml` (nextest runs without one — gate:3 proves it
    daily), so the change stands alone.
  - **A post-run audit** greps every remaining `Timeout`'s log for failing-test markers and prints
    `mutation: N timeout(s); M mislabeled (…)`, so a caught-by-assert mislabel is visible instead of
    silently absorbed. The `Timeout`-counts-as-caught convention and the MSI arithmetic are **unchanged**
    (§0 — no floor lowered, `MUT_MSI_MIN` untouched); the audit is visibility only and cannot mask a
    `MissedMutant` (it touches only `Timeout` rows).

### Fixed

- **Self-test harness: the `focus` verb is a layout-independent app raise, not a fixed content click**
  (TICKET-383, forge #383; M25 App-Grade QA Hardening; chore). `scripts/selftest/drive.swift`'s `focus`
  clicked a FIXED window fraction (0.5, 0.12) chosen for the boot-default layout; on a restored workspace
  (Files panel + an editor split) that point is a file-tree row — the 2026-07-21 QA run's first `focus`
  silently opened `.cargo/audit.toml` and mis-sent the follow-on keystrokes, costing several diagnostic
  rounds before the harness (not the app) was identified. `focus` now brings the app frontmost via
  `NSRunningApplication.activate()` — no content click, layout-independent, pid-keyed for both the bundle
  and bare-binary launch lanes. Pane targeting moves to the caller: `clickat:<pane>` with coordinates from
  a fresh capture. A README pre-drive checklist states the two re-learned rules (activate in the SAME
  shell command as the drive verbs; `focus` is activation-only) plus the gpui fact an inspect critic
  surfaced and a driven test confirmed: a `type:` lands in a pane ONLY after a `clickat:` — gpui sets
  keyboard focus on a mouse-DOWN, and a raise posts none (`focus type:echo` types nothing;
  `clickat:<pane> type:echo` runs it). Harness-only; the app is unchanged.
- **An unfocused editable split pane no longer shows stale text — its read-only lines re-sync from the live
  Buffer** (TICKET-356, forge #356; M15 → ships M22, a #259 follow-up). A FOCUSED editable split pane renders
  live from its Buffer, but an UNFOCUSED one fell back to `CodeViewState.lines` — set once at file-open and
  never re-synced — so a focused edit "vanished" on click-away until re-focus (no data loss; the Buffer is
  truth). Now a version-memoized `CodeViewState::sync_lines_from` (reusing the tested `code_lines`) re-syncs
  each open file's read-only lines from its Buffer in a `&mut` pre-pass at the top of `render` — O(1) unless the
  buffer advanced — so the unfocused pane's existing (overlay-free) read-only render shows current text. The
  focused pane is skipped (it renders from the Buffer, not `cv.lines`); a reload resets the memo (a fresh Buffer
  restarts at version 0 — the #268/#273 `(nonce, version)` trap, caught at inspect).
- **The diagnostics gutter now scans EVERY terminal tab, not just the active-or-first** (TICKET-295, forge
  #295; M18 origin → ships M22). `open_file_diagnostic_rows` iterated `workspace().states()` — the
  active-or-first terminal grid only — so with the editor tab active (the normal case for viewing diagnostics)
  a failed command block in any OTHER terminal tab never lit the gutter. It now iterates a new pure
  `Project::terminal_grids()` (the per-project sibling of the workspace-wide `grids()`), scanning the last
  failed block of every terminal grid in the ACTIVE project; the per-grid body + the `merged_rows` union are
  unchanged (each grid appears once, `merged_rows` sort+dedups → the result stays order-independent). Found by
  the #291 live drive. `terminal_grids()` is pure (cov/MSI 100 in tabs.rs); the shim (`mutants::skip`) is a
  STRICT SUPERSET of the #291-proven single-grid path, so nothing previously scanned is dropped.
- **The command palette no longer advertises a keycap chord that a context-scoped keymap row has shadowed**
  (TICKET-306, forge #306; M22, a #298/#265 follow-up). With an editor pane focused the palette showed a ⌘⇧L
  chip next to "Split Right", but ⌘⇧L on the editor resolves to `select-all-occurrences` (#298) and will not
  split — the advertised chord did something else (the class also covered ⌘D → new-terminal and ⌘⌥↑/↓ → pane
  focus). The chip is now resolved PER ROW against the FOCUSED surface's context stack: a new pure
  `palette::displayed_binding(command, keymap, stack)` returns a command's chord iff
  `keymap.action_for(chord, stack)` still equals that command's own action (`action_for_command(id)`), else no
  chip — so a shadowed or off-surface chord no longer lies. The resolver was already built (#265's `action_for`
  + the dispatch table), so this is a pure CONSUMER fix (3 mutants, MSI 100 in `palette.rs`) plus a one-line
  render swap that reuses the existing None→no-chip branch (`RootView::render` is `mutants::skip`); the chip
  decision now matches exactly what pressing the chord on that surface dispatches (`app.rs:14503`).
- **The worker-thread headless polls no longer flake the commit gate under load** (TICKET-364, forge #364; a #334
  follow-up). #334 hardened the real-PTY polls with an event-driven `poll_until` (a generous 30s finite wall-clock
  deadline driving the mock-clock pump) but explicitly deferred seven "Shape-2" polls that wait on a background
  syntax-tree landing (keyed on `(nonce, version)`) — `syntax_async` (×2), bracket-match, and the #329/#330/#349
  cache drives. Those still used a fixed `for _ in 0..100` (~2.5s) budget that, under load, could lapse before the
  off-thread parse landed and fail the `cargo mutants` baseline closed (exit 4 → a non-deterministic gate block).
  A new probe-FIRST sibling `poll_until_pre` gives them the same finite `POLL_CEILING` deadline; it evaluates the
  probe BEFORE the first pump tick (unlike `poll_until`'s body-then-probe), because `syntax_async`'s "the cache
  lags immediately after the keystroke" assertion reads the pre-tick state. Test-harness-only (`#[cfg(test)]`) — no
  product code changed; `poll_until` and its real-PTY sites are untouched.
- **Auto-close no longer over-pairs `'` at a lifetime-bound position** (TICKET-362, forge #362; M22, a #346/#338
  follow-up). Typing `'` inside a `<...>` type-parameter list (`fn f<'a>`, `fn f<T: 'a>`, `struct S<T: 'a>`) used
  to auto-pair into `''` as if it were a char literal — #338's one-character guard only caught `'` after `&`/`<`/
  an identifier (`&'a`, `Foo<'a>`), not the SPACE-preceded bound positions, whose `prev` is identical to
  `let c = 'x'`. #362 adds a pure `marley_syntax::in_type_parameters` ancestry probe (a Phase-1 spike confirmed the
  PRE-insert tree resolves a `type_parameters` node — the `<>` delimiters anchor it even while the inner bound is
  incomplete, unlike a bare `let c = '`) feeding a new `auto_close::Context::LifetimeBound` that suppresses only
  `'` (a `(` for `T: Fn(…)` still pairs). The decision is pure (cov/MSI 100 in `marley_syntax` + `marley_editor`);
  the app-side tree read is a boot-path shim. The `where T: 'a` and `dyn Trait + 'a` positions stay deferred — the
  pre-insert tree has no `<>` anchor for them.
- **⌘⌥↓ (add-cursor-below) now follows the newly-added cursor, not the stationary primary** (TICKET-360, forge
  #360; a #342 follow-up). The add-cursor arm followed `active_caret()` = the PRIMARY (member 0), and ⌘⌥↓ grows
  the set from `set.last()` so the new caret sorts to the BOTTOM while the primary stays member 0 at the top,
  stationary — so sustained ⌘⌥↓ marched a column of carets past the bottom viewport edge and the view never
  followed them ("cursors you cannot see"). (⌘⌥↑ worked only by luck: its new caret sorts to member 0, which the
  primary-follow happened to track.) Fixed by gating the follow on `added_member(&before, &set)` — which names the
  actually-new cursor regardless of how it sorts — and routing through the column-aware `scroll_editor_to(row,
  Some(head))`, the exact seam #342 built for ⌘D's add-next-occurrence. Both ⌘⌥↑/⌘⌥↓ now follow the new cursor
  explicitly rather than relying on the sort. Proven by three headless drives against the real `editor_scroll`
  handle (`editor_scroll_y`).
- **A load-flaky headless test no longer blocks the commit gate under load** (TICKET-334, forge #334; a
  #331-adjacent test-flake). `new_tab_inherits_live_cwd_headless` (and its sibling
  `mouse_modes_track_decset_headless`) polled a REAL PTY for its shell-integration `pwd` with a FIXED
  200-iteration (~5s wall-clock) budget. On an idle box the handshake completes in ~0.1s, but under load — the
  box busy, and `cargo mutants` driving `cargo test` so the ~599 marley lib tests run as parallel threads in one
  process with no per-test isolation, several spawning their own PTYs — the spawn+rc+integration handshake could
  exceed 5s and the poll gave up (`last saw None`). Because the mutation gate treats a failed baseline as exit 4
  ("not a valid measurement") and fails CLOSED, this could BLOCK the commit gate non-deterministically — most
  likely exactly when the machine was busy — and would bite CI on a shared runner. Fixed with a shared
  event-driven `poll_until` helper: it polls until the probe is met OR a generous FINITE 30s ceiling elapses (a
  `std::time::Instant` deadline, independent of gpui's mock `advance_clock`), so a healthy box still exits in
  ~0.1s while a slow box waits LONGER instead of failing. Entirely test-harness (`#[cfg(test)] headless_drive.rs`)
  — no app change. The distinct Shape-2 worker-thread polls (a probe-first `for _ in 0..100` class) are a
  separate follow-up.
- **Closing a workspace now shuts down its rust-analyzer instead of leaking it for the process lifetime**
  (TICKET-359, forge #359; M20, a #321 follow-up). `RootView.lsp_hosts` was insert-only: `close_project_at`
  cleaned up every other per-project map (`collapsed_projects`/`agents`/`remotes`/`last_agent`) but never the LSP
  hosts, so a closed workspace's rust-analyzer (1–4 GB RSS) kept running until the app quit. #321 widened the
  blast radius by deriving host creation from open-document state on the pump, so a restored multi-project session
  spawned one rust-analyzer per root — several unreclaimed = a user-visible memory problem on a long session.
  `LspHost::drop` already did the polite `shutdown`+`exit`+reap; it was simply never called for a closed
  workspace. Fix: `close_project_at` removes the closed workspace's host from the map so `Drop` fires —
  LAST-ONE-OUT, gated on a pure `Workspace::has_project_with_root` (cov/MSI 100), because two open projects can
  share a root and thus one host (opening the same directory twice is not deduped), and an unconditional remove
  would kill a server a surviving co-owner still needs. The map is keyed by the raw project root (= the insert
  key), so the guard's `Path` equality and the map key's `Path` equality are the same relation on the same field
  — a stale or wrong drop is structurally impossible. Dropping a host on the last EDITOR-tab close remains a
  separate deferred (#321) decision. Proven by two headless drives (distinct-roots drop-and-survive; same-root
  last-one-out) through the real `close_project_at`.
- **The editor scroll wheel's horizontal delta scales by cell width, not line height** (TICKET-343, forge #343;
  M22). #336's editor wheel handler computed the horizontal delta as `pixel_delta(cell_h).x` — gpui's
  `pixel_delta(line_height)` applies ONE scalar to both axes, so a `ScrollDelta::Lines` (mouse) x-delta was
  scaled by the LINE HEIGHT (~19px) instead of the cell WIDTH (~8px), making a real mouse's horizontal
  tilt-wheel scroll ~1.5-2.4x too fast. Narrow: trackpads send `ScrollDelta::Pixels` (exact px, bypasses the
  scalar), so the primary dev path was unaffected — why it shipped. Fix: a pure
  `h_scroll::wheel_x_px(raw_x, precise, cell_w)` (cov/MSI 100) — `Pixels` passes through as exact px, `Lines`
  scales by `cell_w` (floored at 1, `sane`d) — and the wheel handler now matches `ScrollDelta` and routes
  through it. gpui's single-scalar `pixel_delta` can't express per-axis scaling, so the editor scales its own x
  axis. The sign convention (dx negated, positive-right) is unchanged; only the `Lines` magnitude changed.
- **⌘D's horizontal scroll-follow tracks the added cursor, not the primary** (TICKET-342, forge #342; M22). #336
  put the editor's horizontal caret-follow on the shared `scroll_editor_to_row` primitive, which reads
  `active_caret()` = the PRIMARY (topmost) cursor. For ⌘D add-next-occurrence — the one gesture whose VERTICAL
  scroll targets the newly ADDED, non-primary cursor — the horizontal then followed the wrong cursor, so an
  occurrence added off-screen-right arrived with its row in view but its column still clipped. Latent (the
  follow's identity case usually hid it), but real. Added a column-aware
  `scroll_editor_to(row, Option<CharOffset>)`: `Some(offset)` follows a SPECIFIC caret (⌘D passes its added
  cursor's `head()` so both axes track the same cursor), `None` follows the primary — so
  `scroll_editor_to_row(row) = scroll_editor_to(row, None)` is byte-identical for every other caller
  (goto-line, go-to-def, find, the keyboard follow). The target is a `CharOffset`, not a raw column, so the
  DISPLAY column is derived by the shipped tab-aware `col_of_offset` (the char-vs-cell trap). Wiring over the
  tested #336 h-scroll primitives (`follow_caret_x`/`caret_px`), so the new fns are `mutants::skip` shims;
  proven by two headless drives of the REAL `add-next-occurrence` handler (the follow lands on the added
  selection's head — the caret at the word's end). The inspect found the analogous `add-cursor-below` (⌘⌥↓)
  gap — the recon's "unique to ⌘D" was an over-claim — filed as #360, which reuses this same seam.
- **No test can hang a gate again — bounded PTY teardown + a nextest terminate ceiling** (TICKET-348,
  forge #348; M22). A gate started at 20:41 was still "running" 11 hours later, wedged on ONE test —
  `resize_real_pty_succeeds`, SLOW **>39,240s at 0.0% CPU** (blocked, not slow), with nextest reporting it
  SLOW forever without killing it. A hung gate that reports nothing is the worst failure mode: no red, no
  green, just silence. The unbounded wait was **delegated**: `terminal_blocks` has zero `Drop`/threads/
  `waitpid`, so dropping a session ran `alacritty_terminal`'s `tty::Pty::Drop`, which sends `SIGHUP` then
  blocks on an **unbounded `child.wait()`** — forever, if the child ignores `SIGHUP`. Two fixes:
  - **A `.config/nextest.toml` terminate ceiling** — `slow-timeout = { period = "60s", terminate-after = 3 }`:
    SLOW at 60s, SIGTERM→SIGKILL at 180s, and the terminated test **FAILS** the run. One knob covers every
    nextest lane — gate:3 (`nextest run`), gate:4 (`llvm-cov nextest`), gate:15 (the visual harness), and
    #345's `cargo mutants --test-tool=nextest`. macOS has no `timeout(1)`, so this is where the ceiling lives.
  - **A bounded session teardown** — a pure, clock-injected `reap_step` escalation (`SIGHUP` → 2s deadline →
    `SIGKILL` → 2s deadline → give up) executed by the `pty_os` shim, which pre-empts alacritty's blocking
    `Drop`: on success the child is already reaped (its `child.wait()` returns from cache at once); on the
    pathological give-up path the `Pty` is `mem::forget`'d so that blocking `Drop` never runs (D5
    zombie-over-hang — a leaked fd is recoverable, a hang is not). `resize_real_pty_succeeds` now passes in
    **33 ms**. The child-exit poll (`next_child_event`) is **edge-triggered** — it consumes the one `SIGCHLD`
    self-pipe byte — so a child reaped during normal pumping is invisible to a later poll; a `child_reaped`
    latch makes teardown of an already-exited child instant instead of running the full ~4s loop. The
    real-PTY tests stay under coverage — no exclusion (§0); the bound is the fix. The teardown is a safe
    `Option<Pty>` + `mem::forget` (no `unsafe`).

- **Selection-ladder and sticky headers now gate on language — no more Rust parse on a `.json`/`.py`
  file** (TICKET-350, forge #350; M22). `step_selection_ladder` (⌃W/⌃⇧W expand-selection, #329) and
  `refresh_sticky_headers` (#330) each built a throwaway **Rust** `HighlightSession`
  *unconditionally*, neither checking the active file's language. On a non-Rust file that meant ⌃W
  walked a Rust **error-recovery** tree's node ancestry — plausible-but-wrong rungs — and sticky
  headers could pin a hallucinated `function_item`-shaped node from misparsed text.
  - **Both now gate on `language_of(&path) == Language::Rust`** at the caller, mirroring
    `refresh_bracket_match` verbatim — the #340 **M1 rule**: a language-specific pure primitive
    (`enclosing_ranges`/`all_headers` always parse as Rust) cannot self-gate, so the caller must. This
    closes the M1 class across every structural consumer; the three gated callers are now
    `refresh_bracket_match`, `step_selection_ladder`, and `refresh_sticky_headers`.
  - **The two seams are asymmetric on purpose.** The ladder needs **no explicit clear**: its
    early-return precedes every read of `selection_ladder`/`selection_ladder_at`, and a stale ladder
    cached from a Rust file is already made inert by the `(nonce, version)` validity check — so a
    non-Rust file simply never consults it. Sticky **does** `.take()` its cache (dropping any pinned
    headers in one repaint on a Rust→non-Rust switch) because `sticky_rows` reads
    `self.sticky_headers` on **every** pump frame — an uncleared cache would keep rendering.
  - **Bracket-match, folding, and file-symbols are byte-identical** — they share no touched code. The
    stale `#315` "stays Rust-parsed, pre-existing behavior" comments at both sites (a false doc the
    compiler can't catch) were replaced with the gate's own doc.

- **A restored editor tab now spawns the LSP** (TICKET-321, forge #321; M20). Quit Marley with a
  `.rs` file open — the state #205's persistence exists to preserve — relaunch, and you silently got
  **no diagnostics, no hover, no go-to-definition, no completions**, with no error and no status
  segment, until you happened to open some *other* file. #308's only spawn trigger was the open
  *gesture* (`open_file_in_viewer`), and a restored tab never gestures.
  - **Host creation now derives from open-document STATE**, on the pump, joining the #309 doc-sync
    reconcile that was already state-derived and consuming the same open-document set — so however a
    document came to be open in an editor tab of the active project, the host follows. The gesture
    call is removed: one trigger, not two.
  - **The spawn gate is unchanged in behavior** (`.rs` · under root · no existing host · root has
    `Cargo.toml`) but reordered cheapest-first because it now runs every pump tick rather than once
    per file-open: `contains_key` → the in-memory path scan → the manifest `stat` LAST. The stat
    stays per-tick deliberately — that re-evaluation is what lets a later `cargo init` in an open root
    summon a host on the next tick.
  - **Late creation is already wire-legal**: `reconcile`'s newly-seen-document arm sends the
    `didOpen`, the same path a respawned server uses to re-open its documents.
  - The recorded empty-`didOpen` corruption (`BF-lsp-didopen-carries-empty-text-ready-race-001`)
    **cannot recur**: `LspHost::new` can never return a `Ready` host (Ready is reachable only via
    `(Initializing, InitializeResult)` ← `on_message` ← `drain()`, which has already run this tick),
    so both Ready-gated reads decline a host created mid-tick and the first thing the server receives
    is a `didOpen` built from materialized text on a later tick. **Do not introduce a `drain()`
    between the ensure call and the reconcile** — that ordering, not the placement, is the invariant.

- **Multi-cursor Tab / ⇧Tab — both branches of the indent arm learn about the whole cursor set**
  (TICKET-307, forge #307; M19). With three cursors on three blocks, Tab indented only the **primary**
  cursor's block and padded only the primary's caret; #297 had already stopped the other cursors being
  destroyed, but they still went unindented. Both branches now act on the whole set: a block op indents
  the **union** of every cursor's rows (`indent::touched_rows` — two cursors sharing a row yield one
  indent, not two), and an all-carets Tab pads **every** caret, each sized from its own display column
  through the #250 layout.
  - **The row-list builders.** `indent_edits`/`dedent_edits` widened from a contiguous `(first, last)`
    range to a discontiguous `rows: &[usize]`, and now **enforce** the ascending + deduped contract
    internally rather than trusting callers. That stance is adopted, not invented: the shipped sibling
    `comment::comment_edits` already had exactly this signature and exactly this enforcement, and its
    own comment named #307 as the incoming second caller — a `pub` cross-crate fn whose results the
    caller applies back-to-front cannot afford a mis-ordered list (buffer corruption) or a duplicate row
    (a line indented twice).
  - **The range clamp is gone, as unreachable rather than as a shortcut.** `Buffer::line_text` answers
    `""` for a row past the end, so indent's empty-line filter and dedent's `strip > 0` guard both drop
    such a row before `line_start` is consulted. Worth stating precisely, because `line_start` **clamps**
    instead of panicking: an unfiltered out-of-range row would not crash, it would silently attribute the
    edit to the **last** line. An explicit bounds guard would be redundant — every row it could reject is
    already rejected — so the reliance is documented and pinned by `usize::MAX` test rows instead.
  - **A latent bug fixed as a rider.** The branch predicate read `active_selection()`, which reports the
    **primary** only. A mixed set (caret primary + ranged secondary) therefore looked selection-free, took
    the pad branch, and ignored the range entirely. The predicate now asks the set.
  - **⌘Z granularity is deliberately asymmetric, and now tested.** A block op **always** groups, including
    when it produces no edits — that open-then-drop is what keeps a no-op ⇧Tab from clobbering a pending
    redo (#282). A **lone** pad stays **ungrouped**, so it still coalesces with the next typed character
    and one ⌘Z reverts both; grouping it would set `sel_before`, which `UndoHistory::record` refuses to
    coalesce into, silently splitting one ⌘Z into two. Multiple pads group so a single ⌘Z reverts them all.
  - Documented divergence, pinned by test: two carets on the **same row** are both sized in PRE-edit
    space, so each lands directly after its own padding but the later one may not finish on a post-edit
    tab stop, where an editor applying the pads sequentially would. Carets on different rows never interact.

### Added

- **Editable split-file pane — the editor stops being a singleton** (TICKET-259, forge #259; M15). The
  #246/#258 split-right pane showed a file beside your terminal — read-only. Now a **focused** split pane is a
  real editing surface: type, caret, selection, save, undo, clipboard, motion — the editor stack, in a pane.
  This is the batch's one genuinely architectural change: the first time "the editor" is not a singleton. The
  load-bearing parts:
  - **The pane grows a `Buffer`.** `PaneContent::CodeView(CodeViewState)` (a lossy, read-only display snapshot)
    became `CodeView(EditorSurface)` — the same editable surface type an editor tab holds — following the
    shipped #237 recast pattern exactly, so `PaneState::code_view()` still reads through the active file and
    the render + the #258 `c=<path>` persistence codec are unchanged (the buffer reloads from the path on
    restore).
  - **One focus-aware accessor, not a rewrite.** `active_editor()`/`active_editor_mut()` became focus-aware —
    an editor tab resolves to its own surface, a terminal tab resolves to its **focused** editable pane (else
    the terminal) — keeping their return type, so ~75 through-accessor call sites were untouched; only the
    ~43 inline `active_tab().editor()` bypassers moved onto them. It mirrors the shipped `focused_terminal()`
    accessor verbatim. `key_context()` and the single `editor_geom`/IME slot follow focus too: exactly one
    focused text-input surface writes the slot each frame (macOS's model), chosen by focus.
  - **Two independent buffers, safely.** The same file open in the editor tab **and** an editable pane are two
    independent `Buffer`s (edits do not mirror live); the shipped #275 external-change machinery is the net —
    saving either copy trips the other's disk-stat conflict banner, so a silent last-write-wins clobber cannot
    happen. True shared-buffer multi-view is a named follow-up.
  - **A code review caught a flagship-breaking bug the author's self-review missed.** A shared `'static`
    render row closure re-read the surface via `active_tab().editor()` — which is `None` for a split pane's
    terminal host tab — so a **focused** editable pane rendered *blank* (full scroll height, zero rows) while
    you typed blind. The blanket accessor-move had missed that one chain (its receiver was `entity.read(app)`,
    not the pattern moved), and the closure had diverged from the focus-aware `fold_projection` that sized the
    list. Fixed to the focus-aware `active_editor()`, proven by a headless drive that splits + focuses + types.
  - Scoped to the core editing surface (slice 1); editor-feature parity in the pane (find, folding, symbols,
    LSP overlays) is a tracked follow-up.

- **Find references (⇧F12) — every usage, grouped by file, one Enter from the caret** (TICKET-317, forge #317;
  M20 language intelligence). ⇧F12 on a symbol answers "who uses this?": a picker of every reference, grouped
  under a file header, each row showing its line's text; ↑/↓ move (headers skipped), type to fuzzy-filter, Enter
  jumps (centered, NavStack-pushed so ⌃- returns), Esc closes. The read-only sibling of go-to-definition and the
  last leg of navigate-by-meaning. The load-bearing parts:
  - **A NEW grouped row model — not a stretched definition picker.** The recon proved #312's `DefPicker` is
    definition-shaped (a flat `Vec<(String, DefLocation)>`, `path:line` labels, no line text, no grouping), so
    references needed their own type: file `Header` rows interleaved with `Reference` rows carrying the line
    text. The genuine reuse is the jump path (open + center + push the NavStack), the #312 position-keyed stale
    guard, the `referencesProvider` capability reader, `cap_with_tail`, `FinderState`, and `fuzzy_score`.
  - **The grouping is pure and contrast-… er, coverage- and mutation-proven.** `group_references`
    (dedupe exact duplicates · current file first · honest ~200 cap with a "+K more" tail · a `Header` before
    each file's run · a computed `N references in M files` line) lives in `editor_references.rs` — 100% line
    coverage and 100% mutation (MSI), no IO. The line text is filled app-side: an OPEN buffer's row reads LIVE
    text (an unsaved edit wins over disk), a cold file reads disk once per file, an unreadable file degrades to a
    path-only row — never a panic, never a stall.
  - **A code review caught two real bugs the author's self-review missed.** (1) The "Finding references…" card
    was a naive latch that a timed-out or disconnected request never clears (the pump drops a dead request's
    purpose with no response) — so the occluding card could strand forever. The fix trusts the pending-request
    table as the one source of in-flight truth (`has_pending_references`), and the launcher's "close everything"
    also clears the latch. (2) The `find_references` shim was missing its mutation-skip attribute (it would have
    failed the mutation gate). The `Range` parser also clamps `end_col >= start_col`, so a malformed server span
    can never underflow a downstream slice.
  - ⇧F12 is bound Editor-scoped (plain F12 stays go-to-definition) with a "Find All References" command-palette
    row.

- **Per-theme syntax colors — a contrast-proven `SyntaxPalette`** (TICKET-316, forge #316; M20 language
  intelligence). Switch Marley's theme and the chrome restyled, but code kept one hardcoded look — after #315
  that was ~6 fixed `hsla` literals (identical in Light and Dark). Now every theme carries a `SyntaxPalette`:
  one color per the #315 taxonomy's 9 syntax slots, tuned per built-in theme, and **contrast-proven by test** —
  each slot clears WCAG AA (≥4.5:1) against that theme's background, so a palette typo cannot ship illegible
  code. The load-bearing parts:
  - **The palette is a field on `ThemeColors`.** `token_color` already routed every render (editor rows, the
    #246 split pane, the #331 hover fence) through one seam, and they all pass `&ThemeColors` — so resolving
    through `colors.syntax` restyles code everywhere with zero call-site change. The 6 hardcoded literals died.
  - **The live switch is free.** The syntax cache stores KINDS (`TokenKind`), not colors — colors resolve
    per-frame in `token_color` — so a theme flip repaints correctly with no cache to invalidate.
  - **Contrast is proven, not eyeballed — AND so is distinctness.** A contrast test proves each color is legible
    against the background but is BLIND to whether two slots are the same color: a code review caught the Light
    `keyword` and `property` shipping as an identical dark teal (both cleared AA). So there are two gates — the
    AA matrix and a pairwise slot-distinctness test (close on hue + lightness + saturation).
  - **A derived-default generator** (insurance) gives a palette-less future theme an accent-anchored spread that
    passes the same AA gate; v1 ships none, so it's test-exercised only.

- **Multi-language syntax highlighting — Python, JavaScript, TypeScript, TSX, JSON, Bash** (TICKET-315, forge #315;
  M20 language intelligence). Highlighting was Rust-only; it now threads a **language axis** through the syntax
  crate — one `Lang` enum, each with its tree-sitter grammar + the grammar's OWN adopted highlight query — so any
  file with a grammar takes the tree-sitter path and everything else (Markdown, plain text) keeps the shipped
  hand-lexer floor. The token **taxonomy grew 5→10 slots** (added Function, Type, Attribute, Punctuation,
  Property), which as a named side effect gives *Rust* richer color too (function names, types, and delimiters
  that previously rendered plain). Colors are reasonable v1 fixed hues; per-theme palettes are #316. The
  load-bearing parts:
  - **One capture→taxonomy map serves every grammar.** tree-sitter's highlight capture names are a shared
    convention (`function.builtin`, `constant.builtin`, …), so a single base-name map covers all languages, with
    one full-name override: an object/JSON **key** (`string.special.key`) reads as a Property, not a string value.
  - **The off-thread worker rebuilds its parse session when the file's language changes** — a switch is also a new
    file identity, so it re-parses whole anyway; the rebuild just re-points the parser at the new grammar.
  - **TypeScript/TSX concatenate the JavaScript base query.** The TS grammar ships an `inherits: ecma` *overlay*
    (types + TS-only keywords only) — compiled alone it leaves `.ts`/`.tsx` almost blank, so the arms prepend the
    JS (and, for TSX, the JSX) query — the standard `inherits` resolution (a code-review find; a naive capture-set
    pin test passes while the language is broken, so the real guard is a behavioral fixture per language).
  - **TOML stays on the hand-lexer floor for now.** Its adopted query captures `@property` over the whole
    `key = value` pair, which our outer-wins span sweep would let steal the value's color — so `.toml` keeps its
    (correct) hand-lexer highlighting until the sweep is innermost-wins (a follow-up). JSON's key double-capture is
    made deterministic with a stable sort.

- **LSP formatting — ⌥⇧F, and opt-in format-on-save** (TICKET-314, forge #314; M20 language intelligence).
  ⌥⇧F formats the active Rust document through rust-analyzer (`textDocument/formatting` → rustfmt); an opt-in
  `editor.format_on_save` (default OFF, a palette toggle) runs it before every ⌘S. Two invariants carry it: the
  apply is **transactional** (one undo unit — ⌘Z restores the pre-format text byte-identically; an
  overlapping/inverted edit batch applies nothing) and the **save is never blocked or lost** (a slow/absent
  formatter degrades to a plain save on a ~2 s deadline). The interesting parts:
  - **The apply is the engine #322 already shipped.** `apply_one_file` resolves the server's `TextEdit[]` to char
    offsets (the #309 encoding bridge), rejects overlaps whole, and splices in one undo group — so the new pure
    surface (`marley_lsp::formatting`) is just the wire shapes: `formatting_support` (the capability read),
    `formatting_request_params` (honest `tabSize`/`insertSpaces`), and `parse_formatting_edits` (the bare
    `TextEdit[] | null`, malformed elements skipped).
  - **The caret is re-seated by (line, column), not carried by an anchor.** rust-analyzer returns one whole-file
    `TextEdit`, which *covers* the caret — and a covered anchor collapses to the edit's start (offset 0), which
    would teleport the caret to line 1 on every format. Capturing the caret's line/column before the apply and
    re-seating it after keeps it on its logical line (a code-review find).
  - **Format-on-save is one write, never lost.** ⌘S parks the save, sends the format, and writes exactly once —
    of the *formatted* text — when the response lands; a ~2 s deadline (its own countdown, not the 10 s request
    timeout), a server error, or an intervening edit each fall through to a plain save. A version guard drops a
    response the buffer has edited past; the completion is bound to the origin editor so a tab switch mid-format
    never saves the wrong file.

- **Code folding — ⌥⌘[ / ⌥⌘] , Fold All / Unfold All** (TICKET-305, forge #305; M19 editor power-tools).
  Fold the innermost definition at the caret (fn / impl / mod / trait / struct / enum / `match` bodies) to a
  single header line with a muted "⋯ N lines" tail; ⌥⌘] unfolds, and the palette carries Fold All / Unfold All.
  Folds ride their header's **anchor**, so they survive edits above and evaporate when their region is deleted;
  any navigation *into* a folded body (go-to-line, go-to-definition, find, symbol-jump) auto-reveals it. Rust-only
  v1 (the caller gates the language). The real ticket, though, is the seam beneath it:
  - **The first buffer-row ↔ visible-row projection.** The editor's `uniform_list` rendered a strict 1:1 mapping —
    every interior site (carets, clicks, drags, selections, squiggles, gutter, sticky headers) read the slot index
    straight as a buffer row. Folding severs that identity everywhere at once, so the pure `marley_syntax::FoldProjection`
    does the conversion **exactly once at the rim** (`total = visible_count()`, then `let row = buffer_row(slot)`),
    leaving the ~17 interior sites untouched; only the outbound crossings (scroll → `slot_of`, sticky → `buffer_row`)
    convert too. With nothing folded it is identity and does no parse — byte-identical to the pre-fold path.
  - **The pure fold model is a fifth `marley_syntax` node API.** `fold_regions(src)` walks the tree iteratively
    (recursion would overflow the UI stack on a deep file) for the multi-line FOLD_KINDS; `FoldProjection` is a
    prefix-sum over merged hidden runs — `visible_count` / `buffer_row` / `slot_of` (a hidden row maps to its
    header's slot), a bijection over visible rows, total (out-of-range saturates, no panic).
  - **Auto-reveal had to hook the *lowest* shared primitive.** The reveal was first wired only into the caret-follow
    path — but the jump commands (go-to-line, symbol-jump, go-to-def / ⌘T / nav-back / diagnostics, and **find**)
    place the caret and scroll *directly*, bypassing it, so a jump into a fold would strand the caret on the hidden
    `⋯` header. Fixed with a shared `reveal_and_scroll_to_row` at every jump site (a code-review find; the find site
    was one an adversarial critic missed). The go-to-line *preview*, whose caret stays at the origin, deliberately
    does not reveal.

- **Go to Symbol in File — ⌘⇧O** (TICKET-304, forge #304; M19 editor power-tools). A fuzzy picker of the
  current Rust file's symbols — fns, structs, enums, unions, type aliases, traits, impl/trait methods, mods,
  and macros. Type to filter (empty query lists them in document order — the outline glance), Enter jumps to
  the chosen one and centers the view, pushing the jump-back stack so ⌃- returns; Esc cancels. Editor-scoped,
  shadowing the global ⌘⇧O open-remote (so on a terminal tab ⌘⇧O still opens a remote).
  - **Symbol extraction is a query run, not a hand-rolled walk.** The pure `marley_syntax::file_symbols` adapts
    tree-sitter-rust's *own* `tags.scm` — reading the shipped grammar's MIT query file (adoption, the
    highest-yield leg of the prior-art sweep) instead of reinventing a node walk. `TAGS_QUERY` compiles once
    through the same `OnceLock` idiom the highlight query already uses.
  - **A method matches twice.** The grammar's tags query captures an impl/trait method as *both* a method and a
    plain function (the free-`fn` pattern has no parent constraint), so the extractor dedupes by the name's
    position, keeping the more specific `Method` — the central correctness edge, order-independent. And its
    single `@definition.class` capture conflates struct/enum/union/type, split back apart by the matched node's
    kind so each gets its own glyph.
  - **Rust-only v1, and the *caller* gates the language.** The pure extractor parses its one grammar
    unconditionally and cannot self-gate; the picker gates on `language_of == Rust` before extracting (the same
    shape bracket-match uses), so a non-Rust file opens the picker with "(no symbols)" and does no walk. (Two
    earlier features that parse ungated were filed as a separate bug.)
  - **The picker is synchronous.** Unlike the workspace-symbol picker (which re-queries the language server per
    keystroke), this file's symbols are already in hand — extracted once, memoized per buffer version, and
    filtered locally with the same fuzzy ranker ⌘P uses. A known v1 limit: bodyless trait/`extern` signatures
    (`function_signature_item`) aren't in the grammar's tags query, so they don't appear.

- **Destructive ops — delete word (⌥⌫/⌥⌦), to line edge (⌘⌫/⌃K), delete line (⌘⇧K)** (TICKET-303, forge #303;
  M19 editor power-tools). The #257 word-motions finally get their destructive halves — deleting a word is no
  longer one Backspace at a time. ⌥⌫/⌥⌦ delete the word left/right, ⌘⌫ to line start, ⌃K to line end, ⌘⇧K the
  whole line; multi-cursor aware, one undo unit per press. Plain forward-delete (fn+Delete) also works now —
  it was a silent no-op before.
  - **The deleted range IS the motion range.** A pure `delete_range_for` reuses the same `move_word_left`/
    `move_word_right` and line boundaries that ⌥←/⌥→ travel, so motion and deletion can never disagree — there
    is no second boundary engine to drift.
  - **The feared "translation collapse" wasn't one.** The spec predicted these chords needed a restructure of
    the key-translation seam (`⌥⌫` silently does a 1-char delete today because `key_from_keystroke` returns
    Backspace before checking modifiers). Attacking that against live code found the keymap already resolves
    chords *before* that seam and already preserves modifiers — so the five chords are just five
    editor-scoped keymap rows, no seam surgery. Only plain forward-delete needed a one-line new arm.
  - **The same clamp #300 banned is exactly right here.** Move-line couldn't use `rebase_selections` (its
    clamp collapses a moving cursor to a line start); a delete *wants* that — a caret inside the span it just
    deleted belongs at the span's start. The two tickets document the asymmetry so neither is cargo-culted: a
    delete reuses the clamp, a move carries manually, keyed on whether the op's own edit already moved the caret.
  - **Two edges are pinned by round-trip tests.** ⌃K at end-of-line eats the newline (joins the next line);
    ⌘⇧K on the last line eats the *preceding* newline so no orphan blank line is left behind.
  - A word-delete is its own grouped-edit path, deliberately *not* folded into the backspace handler — so
    ⌥⌫ can't inherit the auto-close-pair widening that ⌫ has. ⌃K is editor-scoped, so the terminal's readline
    ⌃K (kill-to-end) is untouched.

- **Go to line — ⌃G** (TICKET-302, forge #302; M19 editor power-tools). In a 2000-line file you no longer
  scroll to find a line. ⌃G opens a small inline overlay; type `50` (or `50:12` for a column), Enter jumps
  the caret there and centers the view on it, Esc restores the caret you started from. Editor-scoped, so the
  terminal's raw ⌃G/BEL is untouched.
  - **The jump machinery already existed end-to-end — this ticket was one pure function plus wiring.**
    `caret_for_line_col` already owned the 1-based→0-based conversion *and* the past-EOF/past-EOL clamp, and
    `scroll_editor_to_row` already centers (`ScrollStrategy::Center`). Attacking the pre-authored spec's own
    confident sentence against live code dissolved its planned `clamp_goto` entirely — the only genuinely new
    code is the pure `parse_goto` (the `N`/`N:C` syntax) and the overlay shim.
  - **Odd input is inert, never an error.** An empty string, a bare/trailing colon, `0`, a stray third field
    (`50:12:3`), or a 30-digit overflow all parse to `None` — the overlay simply doesn't jump; it never
    panics or mis-places. A past-EOF number is valid and clamps to the last line.
  - **Enter records where you were; Esc doesn't.** A commit pushes the *origin* caret onto the jump-back
    stack (the sticky-header idiom), so ⌃- returns to where ⌃G was pressed; a cancel restores silently and
    pushes nothing.
  - **The overlay owns the keyboard — typed digits never reach the buffer.** The leak gate every inline
    overlay needs (`text_input_blocked` + the arm's `stop_propagation`) is pinned by a drive asserting the
    buffer stays byte-identical while you type. A live preview scrolls to the would-be target as you type,
    but the caret doesn't move until Enter.
  - Every caret-placement path (both commit and Esc-restore) clears any marked IME span *before* the
    out-of-band placement — an inspect finding that a restore-path omission would otherwise misdirect the
    next composition replace.

- **Move & duplicate lines — ⌥↑/⌥↓ move, ⇧⌥↑/⇧⌥↓ duplicate** (TICKET-300, forge #300; M19 editor
  power-tools). The universal reordering ops — moving code no longer means cut-and-paste. ⌥↑/⌥↓ move the
  caret's line, or the whole block of lines a selection touches, up or down one row, carrying the cursors
  with the text (a held ⌥↓ walks a block down the file); ⇧⌥↑/⇧⌥↓ duplicate the block, the copy landing on
  the pressed side. Multi-cursor aware — each disjoint block moves independently — and one press is one undo
  step.
  - **The cursors are carried, not rebased.** The obvious tool (the shipped `rebase_selections`, which Tab and
    ⌘/ use) has a clamp: a position inside a removed span collapses to that span's start — and a move's
    removed span *is* the row the cursor sits on, so rebasing would slam every cursor to a line start. The
    pure `move_lines`/`duplicate_lines` compute the carried cursors themselves and the app writes them
    directly.
  - **The carry is character arithmetic, not bytes.** A cursor rides ± the swapped gap line's length in
    *chars* (including its newline). On an ASCII line bytes and chars coincide, so a multibyte gap line is the
    only test that proves it — the same blind spot behind two earlier defects (#336, #339).
  - **An adversarial review caught a defect that ships green.** The universal "select N whole lines" gesture
    (Home, then ⇧↓) puts the selection's head at column 0 of the row *past* the block — a row the block-finder
    deliberately excludes. Attributing that boundary head to its block by its raw row (instead of by offset
    span) left it with no carry, so the *text* moved correctly but the *selection* silently shrank or grew.
    No text test can see this; only a full-line-selection shape assertion does. Fixed by attributing endpoints
    by offset span.
  - **Edge cases named, not hidden.** The op is all-or-nothing at the buffer edge (the topmost block can't
    move up, the bottommost can't move down — so cursors on rows 0 and 5 both stay rather than half-moving,
    which keeps the no-collision reasoning one line). A caret on the invisible trailing empty line acts on the
    last real line, so ⌥↑ there can't drop the file's trailing newline. And the undo group is
    non-cursor-anchored by construction, so the next keystroke you type is its own undo step — the move can't
    be swallowed into it.

- **Bracket-match highlight — the caret's delimiter pair, lit; ⌘⇧\ jumps between them** (TICKET-340, forge
  #340; M22 "the editing bar"). Put the caret on or beside a `(`, `[`, or `{` and both halves of its pair get a
  subtle background tint; **⌘⇧\ "Go to Matching Bracket"** hops the caret to the other delimiter (and back).
  The quiet affordance every editor has. Rust-only in v1, and that is a correctness decision, not a cut.
  - **The truth source is the tree-sitter tree, so a false positive genuinely cannot exist.** A `(` inside a
    string is a `string_content` node and inside a comment a `line_comment` — never a delimiter node — so
    `matching_delimiters_in` returns nothing there. This was proven at the *tree* level (a spike dumping the
    node at the inner `(`), not merely asserted from highlight spans. A hand-lexer would reintroduce exactly
    the string/comment false-positive class the tree rules out; other languages inherit the guarantee for free
    as #315 adds their grammars.
  - **The match is ADJACENT-only, not enclosing** — a caret resting in open code lights *nothing*, unlike a
    walk that would light the enclosing function's body braces from everywhere inside it. The first
    implementation followed the design's prose ("the smallest node containing the caret") and a probe caught it
    contradicting the ticket's own acceptance criterion; the rewrite looks for a delimiter the caret actually
    touches (preferring the one just *before* the caret, so typing `)` lights its pair behind the cursor).
  - **The highlight is a mark, not a token** — the two delimiters join the same render channel as the find
    bands, at the LOWEST priority (a find match or the selection wins any cell they share), mapped through the
    code-side display map so an inlay hint anchored at a delimiter can never smear the tint. The pair arrives as
    whole-document bytes and is localized per row before mapping — the only test that proves that seam is a
    multibyte one (on ASCII bytes, chars, and columns coincide — the same blind spot behind the #336 defect).
  - **It recomputes per caret move, parse-only, memoized on (nonce, version, caret).** There is no cached
    syntax tree on the app side (the design's "recompute on the cached tree" premise was false — the live tree
    lives only on the worker thread), so each distinct caret position pays one bounded parse (~4 ms on a
    2000-line file, measured — not guessed). A static caret costs nothing; the cached-tree optimization is
    filed as **#349**.
  - **Two defects an adversarial review caught before commit.** (1) There was no language gate — the always-Rust
    parser ran on *any* file, so a `.json` or `.py` would have shown a bracket tint driven by Rust's grammar
    (violating the "Rust-only" scope); the pure function is language-specific by construction and cannot
    self-gate, so the check belongs at the caller, mirroring the syntax-cache. (2) An unclosed `(` makes
    tree-sitter's error recovery insert a *zero-width MISSING* `)` whose kind is still `")"` — matching a
    `kind()` string alone accepted the phantom; an `is_missing()` guard rejects it, so an unmatched bracket
    lights nothing.
  - **⌘⇧\ deliberately does not push the navigation stack** — it is an intra-expression hop, not a jump between
    locations, so ⌃- should not treat it as somewhere you came back from. (The one rough edge: with the caret
    parked exactly between an empty `()`, the jump is a no-op — the two delimiters are adjacent, so there is
    nowhere to go. Named, not hidden.)

- **Regex find — a `.*` mode and a case chip on the ⌘F editor bar** (TICKET-339, forge #339; M22 "the editing
  bar"). Toggle `.*` and the query becomes a regex; `fn \w+` finds every function, `\d{1,3}(\.\d{1,3}){3}`
  finds IP addresses. The matches feed the existing bands, the n-of-m counter, and Enter/⇧Enter navigation
  unchanged. An invalid pattern shows an inline error instead of matching nothing. With the chip off, the bar
  is byte-identical to before.
  - **The plan claimed "the plumbing won't know the mode exists"; it was half true and self-refuting.** The
    read path (bands, counter, navigation) genuinely is mode-blind — verified, nothing recomputes a match end
    from the query's length, so a varying-length regex match carries. But the SAME fact — the match list is
    just `(start, end)` pairs — is *why* capture-group replace cannot ride it: those pairs discard the capture
    groups replace needs. So the ticket **split**: this ships regex FIND; capture-group replace is **#347**.
    Until then the `.*` chip's Replace works literally (it never claims capture support).
  - **Two things the plan referenced did not exist.** There was no case chip (the bar folded case
    *unconditionally*), and no F3 (find-next is Enter/⇧Enter). So the case chip is genuinely new, and it
    reveals a deliberate split: the literal path folds **ASCII-only** (`k` never matches KELVIN SIGN), regex
    mode folds with full **Unicode**. Same chip, each mode's own definition — because a regex mode whose `\w+`
    could not match `café` would be a broken regex mode. `unicode(false)` was rejected for exactly that.
  - **The empty-match hazard is the crate's problem, and it already solved it.** `^`, `\b`, `a*` can match
    nothing; a naive scan loops forever on a zero-width match. `regex`'s own iterator advances past an empty
    match that overlaps the previous one — and does it *better* than the hand-rolled rule the plan proposed,
    which would have made `^` skip every other line start. Reading the dependency deleted the work. (One
    replace-side instance did survive into the app — an empty-match Replace-One that looped Enter; fixed here
    with a one-char advance, since regex find is what made it reachable.)
  - **Bytes to chars in one pass.** `regex` reports byte offsets; the editor counts chars. Converting each
    match on its own would be a rope walk per match — and worse, it is exactly where a units bug hides, because
    on ASCII bytes and chars coincide (the same coincidence that shipped the #336 defect). Instead a single
    forward cursor over the text resolves every match boundary in one pass, guarded by an assertion that fires
    loudly in debug if the crate's boundaries ever regressed.
  - **The find bar's `regex` dependency lives in the editor crate** — pure computation, so the crate stays
    gpui-free and IO-free. It was already in the lock file (via gpui and tree-sitter), so no new audited
    dependency, just an edge.
  - Known honesty note recorded in the code: an unresolvable capture-replace is the follow-up's job (#347);
    the regex `find` half is complete.

- **Auto-close brackets + quotes** (TICKET-338, forge #338; M22 "the editing bar"). Typing `(` gives `()` with
  the caret inside; typing `)` against its own twin steps over it instead of doubling it; ⌫ between a pair
  takes both halves; typing an opener over a selection wraps it. At N cursors, per cursor, in one undo unit —
  one keystroke can wrap at one cursor while pairing at another. `editor.auto_close` (default on) + a "Toggle
  Auto-Close Brackets" palette row; **OFF is byte-identical to the pre-ticket insert path**.
  - **The plan said this was one pure decision table applied through the shipped multi-cursor engine. Three of
    its four actions turned out to be inexpressible that way.** The #296/#297 engine does the **text** half
    only: its post-state is always a bare caret at the END of each insert. So insert-pair landed *after* the
    `)`; wrap **destroyed the very selection it exists to preserve** (the engine cannot return a range at all);
    and type-over is not an edit — routed through the engine it hits the no-op guard and the caret never moves.
  - **So the engine's post-state generalized from a LENGTH to an optional per-cursor SPAN** — the caller says
    where its cursor lands, relative to its own edit. The identity case *is* the old math, which is what makes
    "auto-close off is the old behavior" provable rather than argued: a property test asserts equality against
    a verbatim copy of the pre-ticket body over 2,000 randomized cursor sets. That matters because this is the
    app's **one insert path** — every keystroke in the editor pays for it.
  - **A flag that named an invariant, and lied.** `cursor_anchored: true` was hardcoded; its documented meaning
    is "every cursor ends at the end of its own insert", and `coalesces_into` **trusts it instead of checking**.
    A caret span is the first thing in the editor's history able to falsify it — so the next typed char was
    appended onto the pair's undo record (`inserted = "()x"` where the text read `(x)`), and **redo replayed it
    verbatim**. Undo hid it; only redo showed the corruption. The tag is computed now, not asserted.
  - **The same shape appeared three times in one ticket** — a fallback that silently assumes something nothing
    states, which this change was the first code able to break. Redo at one cursor recomputed the caret as "the
    end of the insert" (there was no undo group to hold the truth), and pair-backspace's ⌘Z restored the caret
    one char *past* where the user had it (undo's fallback assumes the consumed range ends AT the caret — true
    for every backspace until this one widened it forward). One predicate now governs both.
  - The decision table's two orderings are load-bearing and documented: a live **selection is checked first**
    (the buffer maps type-over to an *empty* replacement, which would delete a selection it ever reached), and
    **type-over before insert-pair** — with `"` beside the caret, typing `"` matches both rules, and stepping
    over is what stops `""""`.
  - **Auto-close never fires during an IME composition, and that cost no IME-awareness at all**: the platform
    text door already forks between ordinary typing and "the platform named one span" (an explicit
    `replacementRange`, a composition commit). Hooking only the first excludes the rest by construction — which
    matters on a stock US keyboard too, since macOS uses that path for press-and-hold accents and Text
    Replacement. A multi-char insert never pairs, for the same reason: it is not a keystroke.
  - **Known limit, stated rather than implied:** the `'` guard suppresses pairing after `&`, `<`, and
    identifier chars (`&'a`, `Foo<'a>`, `impl<'a>`, and the apostrophe in `don't`) but **not** in bound
    positions (`T: 'a`, `+ 'a`, `'static`) — their preceding char is a space, exactly like `let c = 'x'`, so no
    rule reading one character back can separate them. That waits on the grammar (#315). An earlier draft of
    this rule claimed `T: 'b` was covered; it never was.

- **Font size is yours now — `appearance.font_size` + ⌘= / ⌘− / ⌘0 live zoom** (TICKET-337, forge #337; M22
  "the editing bar"). The mono text you actually read — the terminal's command headers and output rows, and
  the editor's code — scales from 8 to 32pt, live, and the choice persists. `appearance.font_family` rides
  along as a hand-edited key.
  - **Content zooms; chrome does not.** ⌘= means *"the text I read gets bigger"*, not *"the whole UI
    rescales"* — the `editor.fontSize` model, not View→Zoom. `type_scale(role, font_size)` returns the live
    size for `Command`/`Output` and keeps `Caption` (11pt) and `Nav` (12pt) fixed. The split was already
    written in the `Role` enum's own docs ("terminal output — the base body text" vs "left-sidebar / nav
    **chrome**"); this ticket just acted on it.
  - **The ticket's premise was false, and finding that out was the work.** The plan said `TERMINAL_FONT_SIZE`
    was *the ONE* font-size const behind both surfaces, so the job would be threading a setting through it.
    It wasn't: the terminal's text came from `typography::type_scale`'s hardcoded scale, the pane container
    from the const, and the editor from its own path. **They agreed only because #195 happened to set both to
    13.0.** The real work was *converging* the sources. The const is now **deleted** — scaling one source and
    not the other re-grids the PTY while leaving the scrollback pinned, a half-zoom on the app's most visible
    surface.
  - **Clamp at every door** (`font_zoom.rs`, pure, cov/MSI 100). `clamp_font_size` is total: non-finite or
    ≤ 0 → the default, anything else clamps into [8, 32]. Both the settings resolver *and* `set_font_size`
    clamp, which is not belt-and-braces — **the chord door is the one that repeats.** Without it, holding ⌘−
    walks the size to zero and the app renders nothing you could read to undo it, and the value is *persisted*,
    so it would come back at every boot.
  - **`FONT_SIZE_DEFAULT` and `LINE_HEIGHT_RATIO` have exactly one home.** The 13.0 in `fallback_cell`'s
    guard had already gone stale **twice** (it said 14 after #195 moved the app to 13). #337 gave the 1.2 line
    ratio a second reader — the terminal's completion popup anchors above an input row whose height is
    exactly `font_size × 1.2` — so it is a named constant now rather than a second literal.
  - The terminal's hit-test now uses the **measured** cell width instead of the fallback's derived one. That
    was wrong *already*, at the default 13pt, from about column 146 — any pane wider than ~1143px. Zoom
    didn't create it; it made it impossible to keep ignoring while touching the metric next door.
  - **Known cut:** an unresolvable `appearance.font_family` degrades to a system font **silently** (forge
    #344). The status flash the plan asked for needs a resolution probe wired at a designed seam — gpui does
    expose one (`TextSystem::font_id -> Result`), so this is a scope cut, not a limitation.

### Fixed

- **Horizontal scroll — a long line's tail is no longer unreachable** (TICKET-336, forge #336; M22 "the
  editing bar"). The editor had **no horizontal axis at all**: `CodeViewState.scroll` is a row index,
  `clamp_scroll_px` took only `(px, total_rows, cell_h)`, and each row renders `.whitespace_nowrap()` — so a
  line wider than the code area clipped at the edge and its tail could not be seen or reached by any means.
  VS Code clips too (`wordWrap: off` is its default) but gives you a scrollbar; Marley gave nothing.
  - **Deliberately not soft wrap.** H-scroll keeps the row↔line identity 1:1 that the `uniform_list` render
    and every `col_starts` rider assume, so it needs **no display map**; wrap breaks that identity and waits
    on the M22 B-c chain. That is why this shipped first, and cheaply.
  - **Pure seam** `h_scroll.rs` (cov/MSI 100): `clamp_scroll_x`, `follow_caret_x`, `content_px`, `caret_px`,
    `clamp_click_to_code`. `scroll_x` is **POSITIVE-RIGHT** (0 at rest, growing reveals the tail) —
    deliberately the opposite sign to gpui's own offsets, because this module never touches a gpui handle and
    the app converts at the single boundary that does.
  - **A clipper, not a margin on the row.** `code_row` sizes to its own unbounded text and is a flex SIBLING
    of the gutter, so a negative margin on it would pull the text LEFT ACROSS the gutter. The shape is
    `[git lane][gutter][clipper(flex_1, min_w_0, overflow_hidden) → code_row(flex_shrink_0, ml(-scroll_x))]`:
    the clipper is the gutter's sibling and never moves, the shift lives one level in, and the gutter stays
    fixed **for free**. `flex_shrink_0` is load-bearing — without it flex squeezes the row back to the
    clipper's width and there is nothing to scroll.
  - **The click math needed no change, and that is the most interesting result.** `x0` is probed by a canvas
    INSIDE the shifted element, and taffy folds the negative margin into a child's absolute bounds — so `x0`
    already carries −scroll_x, both terms of `rel = click.x − x0` are window-space, and the scroll CANCELS:
    `rel` is content-domain for free. The y axis needs no math for the same reason (the list hands the row
    index straight to the closure). Shipped as a named invariant + a pinning test rather than an edit; the
    comment explicitly warns against "fixing" it with `+ scroll_x`, which would double-count.
  - **`LineLayout::display_cols()`** — width in display CELLS. Never `chars().count()`: `char_width` (UAX#11)
    makes a CJK glyph 2 cells and a combining mark 0, and the two agree only on ASCII. Inspect caught the
    shim counting chars, which halved every CJK line's width and left its tail permanently unreachable —
    this ticket's own defect, aimed at exactly the lines that need it.
  - **The caret follow rides `scroll_editor_to_row`**, the shared primitive — not the `follow_editor_caret`
    helper. Three gestures (⌘D, the #312 go-to-definition landing, the #272 find-next jump) call the
    primitive directly and would otherwise have scrolled the row in while leaving the column out.
  - Per-file horizontal memory (`OpenFile.scroll_x`, session-only, parked with its vertical twin) + an x-only
    wheel handler scoped to the editable arm (the #246 read-only split pane must not drive the editor's
    scroll). v1 cuts: the horizontal thumb (deferred — it needs a body-relative code-column origin the frame
    does not publish) and soft wrap.

### Added

- **LSP inlay hints — inline type + parameter annotations** (TICKET-331, forge #331; sprint M21). `let s =
  String::new()` now renders as `let s: String = String::new()`, the muted `: String` supplied by
  rust-analyzer rather than living in the file. The most visible "real IDE" signal the server offers — and
  THE display-map ticket of the batch, deliberately last: the editor's buffer→display mapping had to learn
  about phantom text that occupies columns but belongs to no buffer char.
  - **Pure phantom-aware layout** `code_view::line_layout_with_inlays(line, tab_width, &[Inlay{char_idx,
    text}])` (cov/MSI 100). A phantom emits into `display` and consumes columns while adding NO `col_starts`
    entry, so the map still indexes BUFFER chars (`len == n_chars + 1`). `line_layout(l, w)` is now the
    empty-slice wrapper and a property test pins the two byte-identical across tabs/wide glyphs/combiners ×
    tab widths {1,4,8} — so hints OFF is provably the exact pre-ticket path.
  - **Two invariants, both named in code — the ticket's real content.** (1) `display`'s cell accumulation and
    the values in `col_starts` are ONE column domain: `cols_to_bytes` re-derives a column by walking
    `display` while `col_of_offset` reads one from `col_starts`, and they agree only because one pass builds
    both. Advancing `col` for a phantom BEFORE the anchor's `col_starts.push` keeps the map in SCREEN columns
    (leaving it phantom-blind forks the domains and silently shifts every span and pixel). (2) **A span that
    describes CODE hugs the code; a span that describes a CARET RANGE tracks the caret.** `col_of_offset` is
    the caret map — it deliberately sits AFTER a phantom anchored at that offset — which makes it the right
    END for a selection band (it must stop where its caret renders) and the WRONG end for a syntax token or a
    diagnostic underline. The new `col_ends` / `LineLayout::col_of_span_end` is the code-side end map, used by
    `raw_span_to_display_bytes` and the #310 squiggle; `row_selection_cols` deliberately keeps `col_of_offset`.
    On `let x = 1;` + `: i32`@5 the squiggle for `x` ends at column 5 while the selection for `x` ends at
    column 10 — and both are correct.
  - **The end-of-line asymmetry.** "The caret sits on the code side" is not a uniform rule: mid-line the code
    is to the phantom's RIGHT (emit, then record the column), but at EOL there is no char to its right and the
    code is to its LEFT (record, then emit). Emitting first parked the End caret out past a rust-analyzer
    chaining hint and put an empty line's caret at column 1.
  - **`TokenKind::Hint` → `muted`, required rather than cosmetic** — `styled_slices_with_marks` DROPS a slice
    that is `Plain` + unselected + unmarked, so an untagged phantom would fall through to the base foreground
    and read as real code. Hint spans are fed FIRST, ahead of the code spans: the splitter resolves each slice
    to the first containing range, so ordering makes the phantom's own cells win unconditionally rather than
    depending on another producer's span table.
  - **Pure payload seam** `marley_lsp::inlay` (cov/MSI 100): `parse_inlay_hints` normalizes both LSP 3.17
    label shapes (a bare string and `InlayHintLabelPart[]`, parts concatenated v1), carries kind (1=Type,
    2=Parameter; an unknown kind still renders) and the padding flags, and skips a malformed element per-
    element so one bad hint cannot lose the line's others. Positions go through `u32::try_from`, never
    `as u32` — a cast would truncate a 2^32 line to line 0 and place a confidently-wrong hint (#312's lesson).
    Plus `inlay_hint_params` (the request's own range shape) and `inlay_hint_support`.
  - **The request loop**: `RequestPurpose::InlayHints` on the #311 recipe + the ONE drain, keyed by the
    VIEWPORT RANGE (hints are a property of a region, and the request fires on scroll as well as on edit), for
    the viewport ± `INLAY_PAGE` clamped to the buffer; the #309 encoding bridge projects each hint once at
    apply, and a hint past the last line is dropped rather than clamped. `workspace/inlayHint/refresh` — the
    wire's first server-initiated request beyond the #308 set — replies `AckNull` and raises a latch the pump
    drains into a re-fetch.
  - **Two gates that keep the loop honest**: an error reply writes no cache, so an ungated ask repeats every
    round-trip forever. The request is gated on `language_id_for` (the ONE table #309 syncs by — a file it
    declines was never `didOpen`'d) and on the server's advertised `inlayHintProvider`.
  - **`LspHost::has_pending_inlay()`** — the pending table is the one source of truth for in-flight-ness. A
    timed-out request drops its purpose and delivers nothing, so a latch-only skip would keep matching a key
    whose answer is never coming. Its event-driven siblings are immune (a keystroke mints a fresh key); this
    refresh is POLL-driven off an unchanging viewport, so it would have gone silently dead for exactly as long
    as the user sat reading — precisely when a cold server exceeds the 10s timeout.
  - `editor.inlay_hints` (default on) + a "Toggle Inlay Hints" palette verb. v1 cuts: click-a-hint-to-insert,
    per-label-part tooltips/locations, `inlayHint/resolve`, and length caps beyond the server's.

- **Sticky context header — the enclosing fn/impl pinned at the top while you scroll inside it** (TICKET-330,
  forge #330; sprint M21). Scroll into the middle of a long function and the editor still shows where you are:
  the enclosing `fn`/`impl`/`mod`/`trait` header pinned at the viewport top, clickable to jump back to it. The
  editor twin of the terminal's `sticky_block` (#185), built on the #329 node-range API — its first consumer.
  - **Pure collector** `marley_syntax::all_headers(&HighlightSession) -> Vec<Range<usize>>` (cov/MSI 100):
    every header-bearing node (function/impl/mod/trait) in document order, as byte ranges. The walk is
    ITERATIVE (a `TreeCursor`): recursion would have recursed the FULL tree depth — every nested block and
    paren, not the header nesting — overflowing the UI thread's stack on a deeply nested file and aborting the
    whole app, uncatchably, on every edit.
  - **Pure pin decision** `marley_app::sticky_header::sticky_rows(headers, first_visible_row, max_depth)`
    (cov/MSI 100): a header pins while the viewport top has scrolled strictly PAST its own row and is still
    within its span — so a header sitting at the top does NOT pin (no double-render, the `sticky_block` edge)
    — releasing one row past the closing brace. Nested scopes stack (impl over fn); when more than
    `max_depth` (2) enclose, the innermost are kept (the immediate context, not the outer module).
  - **Scroll stays free**: the header list is computed ONCE per `(nonce, version)` — one throwaway reparse per
    EDIT, never per scroll frame — and the pure filter runs each frame over the cached spans.
  - **The band** draws as an absolute overlay above the list (never inserted into it, which would fight
    `uniform_list`'s row math): surface background + a bottom hairline so code scrolls visibly under it, each
    row syntax-highlighted and gutter-aligned, and `block_mouse_except_scroll` — not `occlude` — so the band
    is not a scroll dead-zone (the #185 lesson). Nothing renders when nothing pins, so it can never eat a
    click. Clicking a pinned header places the caret there and pushes the NavStack (⌃- returns).
  - **`editor.sticky_header`** (default on — the first `editor.*` setting) + a "Toggle Sticky Header" palette
    command.
  - v1 cuts (named): no breadcrumb BAR, `max_depth` stays 2, the terminal `sticky_block` is untouched, and
    `struct`/`enum`/`const` scopes wait for the #315-era grammar work.
- **Expand / shrink selection (⌃W / ⌃⇧W) — grow the selection along the syntax tree** (TICKET-329, forge
  #329; sprint M21). ⌃W in the editor grows the selection out along the tree-sitter node ancestry
  (identifier → call → statement → block → fn); ⌃⇧W walks it back down to exactly where you started. This is
  also a FOUNDATION: marley_syntax exposed highlight spans only (the `tree_sitter::Tree` was fully
  encapsulated) — #329 adds its FIRST node-range API, which #305 folding and #330 sticky-header will reuse.
  - **Pure foundation** `marley_syntax::enclosing_ranges(&HighlightSession, byte_range) -> Vec<Range<usize>>`
    (cov/MSI 100): the ancestry ladder — the smallest named node enclosing the range, then each ancestor up to
    the root, as byte ranges, innermost-first, with adjacent identical spans deduped (tree-sitter nests
    identical-span nodes constantly). Reads the session's held tree; no `is_named` filter is needed because a
    tree-sitter parent is always a named rule node (anonymous string-literal tokens are always leaves). Empty
    on a fresh session or a degenerate (`start > end`) range; never panics.
  - **Pure ladder** `marley_app::selection_ladder::SelectionLadder { anchor, rungs, pos }` (cov/MSI 100):
    `grow` advances toward the root (clamped), `shrink` retreats toward the ORIGINAL selection and at the
    bottom restores EXACTLY the remembered anchor (what makes shrink land where you started, not on the
    smallest node). Built once per gesture and navigated by `pos`.
  - **The app wiring**: the first grow parses the current buffer text once (a throwaway `HighlightSession`,
    the same cost class as the existing per-keystroke sync highlight; a worker-async round-trip for very large
    files is a named follow-up), projects the byte ancestry to CHAR rungs through the rope (exact byte↔char,
    emoji-safe), and caches the ladder keyed on `(nonce, version)`. An edit (version bump) or a caret move
    (primary changes) invalidates it, so the next grow rebuilds. Multi-cursor v1: the PRIMARY cursor grows and
    the others collapse to it.
  - The ⌃W/⌃⇧W chords are Editor-scoped (both free — a terminal tab's ⌃W still deletes-word); ⌥↑↓ was avoided
    (it collides with #300 move-line).
  - v1 cuts (named): no `.scm`-driven "smart" ranges, no node-kind display, N-cursor ladders, and non-Rust
    languages (inherited automatically once #315 lands grammars — the API is grammar-agnostic).
- **Git gutter — added / modified / deleted markers against HEAD, right in the editor** (TICKET-328, forge
  #328; sprint M21). The editor gutter learns what changed: a thin colored bar per edited line — added
  (success green), modified (accent), a deleted-run caret (danger) — computed against `HEAD`. Until now git
  surfaced only as the ⌘⇧D diff overlay and the changes/commit panel; the editor knew nothing per line. The
  marks reflect the last SAVED working-tree-vs-HEAD state and refresh on save, external-change reload, and an
  in-app commit.
  - **Pure projection** `marley_app::git_diff` (cov/MSI 100): `gutter_marks_from_hunks(&FileDiff) ->
    Vec<(row, GitMark)>` walks each hunk's `+`/`-`/context body against a 0-based NEW-side row counter (from
    `parse_hunk_new_start`, the `@@ +c` header start), classifying each change segment via `flush_segment` — a
    pure `+` run → `Added`, a `-`-then-`+` replaced run → `Modified` on the new rows, a pure `-` run →
    a `Deleted` boundary marker between rows. Route A (project git's OWN unified diff — reusing the shipped
    `parse_diff`, zero new deps, argv-only) over route B (in-process `imara-diff`, the deferred keystroke-fresh
    upgrade). `GitMark` is gpui-free; malformed/empty diff → no marks, never a panic.
  - **Decoupled per-path store** (`marley_app`): the marks live in their OWN `HashMap<PathBuf, Vec<(row,
    GitMark)>>`, never a field on the open-file entry. A read-only `git diff HEAD -- <relpath>` spawn
    (`marley_command::blocking`, the `git_working_diff_in` argv posture — no git writes) → `parse_diff` →
    project; an untracked file (`git status --porcelain` `??`) has no HEAD blob → every line `Added`. Cached on
    `(path, git_marks_gen)` where the generation bumps on save / external reload / in-app commit — so an idle
    frame recomputes nothing and per-keystroke stays inert (the saved-state cadence, not a per-tick git spawn).
  - **A second gutter lane** — a fixed 3px bar child before the number cell, coexisting with the #289/#310
    diagnostic number tint (two independent producer lanes, no collision).
  - v1 cuts (named): hunk revert / stage-from-gutter, inline blame, the deleted-hunk block render, an
    index-vs-HEAD toggle, and route B's live-buffer diff (marks are saved-state while a buffer is dirty).
- **Problems panel (⌘⇧M) — every diagnostic in the workspace, one jumpable list** (TICKET-327, forge #327;
  sprint M21). ⌘⇧M in the editor opens a list of every error and warning rust-analyzer knows across the WHOLE
  workspace — INCLUDING files you don't have open — grouped severity-first, jumpable. F8 (#290/#310) walks the
  FOCUSED file; this is the workspace view. Phase 1 is a READ-ONLY list (a diagnostics-multibuffer inherits
  the later gate, like #326's search).
  - **Pure engine** `marley_lsp::diagnostics` (cov/MSI 100): `DiagnosticStore::iter` (the missing enumeration
    primitive) + `total_len` (the cheap re-derive fingerprint); `problem_rows` aggregates every host's store
    (plus a wired-but-empty terminal-lane producer seam — the #310 two-producer doctrine), sorts by
    `(severity, path, line)` via the `Severity` `Ord` (Error first), caps with a "+N more" tail. `ProblemRow`
    is gpui-free + tool-shaped for a future `workspace.problems` MCP read-tier.
  - **UI** `marley_app::editor_problems`: the ⌘⇧M finder picker (a NEW Editor-scoped chord that shadows
    nothing) renders a severity glyph + `path:line` + message; Enter reuses #312's `open_and_place_caret` +
    NavStack and opens even a CLOSED file (the panel's point), resolving the OWNING host's negotiated encoding.
    Live re-derives on each `publishDiagnostics` (gated on the cheap total-count fingerprint), keeping the
    selection by `(path, line, character, severity)` identity so a diagnostic clearing above it does not
    teleport the selection.
  - **Footer** — `cockpit_status` gains a workspace tier ("N workspace") when the workspace count differs from
    the focused file's.
- **Project-wide content search (⌘⇧F) — grep file CONTENTS across the workspace** (TICKET-326, forge #326;
  sprint M21; roadmap B7 phase 1). ⌘⇧F in the editor opens a picker that searches file CONTENTS across the
  whole project — the cross-file complement to ⌘P (file NAMES) and the one-buffer editor find. Type a needle,
  see every matching line grouped by file with the match highlighted; ↑/↓ walk matches, Enter opens the file
  at the match (+ the NavStack, ⌃- returns). Phase 1 is a READ-ONLY results surface; the editable multibuffer
  / replace-all is a named deferred follow-up.
  - **Pure engine** `marley_project::search` (cov/MSI 100): `search_lines(text, needle, opts)` → every
    matching line as `(row, char-col, len, preview)` — literal substring with a case-sensitive + a whole-word
    toggle (ASCII-fold, length-preserving so `col` stays an exact CHAR offset); regex + Unicode case-fold are
    named follow-ups. gpui-free + tool-shaped — the same typed shape a future `workspace.search` MCP read-tier
    would expose.
  - **Walker** adopts ripgrep's **`ignore`** crate (gitignore-aware): its whole transitive closure was already
    in `Cargo.lock`, so it added EXACTLY ONE crate. `walk_text_files` is a LAZY iterator (a superseded walk
    abandons mid-enumeration), still belt-filtered by `should_skip`. Binary/oversize files skip by the
    editor's OWN `is_probably_binary` + `VIEWER_MAX_BYTES` (one truth about "text"), stat-first so a huge file
    never pre-allocates a giant buffer.
  - **Off-thread worker** (the #274 syntax-worker precedent) streams results as files complete; a new query
    CANCELS the old walk via a shared generation, and any OPEN file is searched via its LIVE buffer text (not
    stale disk) — a rename you just typed is findable before ⌘S.
  - **App glue** `marley_app::editor_search` (cov/MSI 100): `accept_gen` (the stale drop), `locate_match`
    (flat selection → file/match), `admit_matches` + `visible_matches` + `footer_summary` + the
    `MAX_FILES`/`MAX_MATCHES_PER_FILE`/`MAX_ROWS`/`MAX_TOTAL_RANGES` caps with the honest "+N more" tail (the
    render cap EQUALS the ↑/↓/Enter clamp). Enter reuses #312's `open_and_place_caret` + NavStack.
  - **⌘⇧F is Editor-scoped**, SHADOWING the global ⌘⇧F toggle-forge (the #325 ⌘T / #265 ⌘D pattern) — search
    in the editor, Forge toggle in a terminal / cockpit.
- **Workspace symbols (⌘T) — jump to any symbol in the project by name** (TICKET-325, forge #325; sprint
  M21). ⌘T in the editor opens a SERVER-filtered picker over `workspace/symbol` — type a name, land on it
  anywhere in the project (the cross-file sibling of in-file symbols). Each keystroke re-queries every
  Ready host (debounced one pump tick), the merged list REPLACES, a stale-query answer drops; Enter opens
  the file at the symbol + pushes the NavStack (⌃- returns).
  - **Pure seam** `marley_lsp::workspace_symbol` (cov/MSI 100): `parse_workspace_symbols` normalizes BOTH
    `SymbolInformation` and `WorkspaceSymbol` (a deferred `location.range` → line 0), decoding the uri via
    `path_from_file_uri`, per-element `filter_map` (a malformed element → a shorter list, never a panic);
    `workspace_symbol_params(query)`; `workspace_symbol_support`.
  - **App picker** `marley_app::editor_symbols`: `SymbolQuery` (the query + a monotonic `gen` — the stale
    key that supersedes two concurrent same-string fan-outs); `symbol_kind_glyph` (LSP `SymbolKind`,
    distinct from completion's `kind_glyph`); `cap_with_tail` (the honest "+N more" tail);
    `MAX_SYMBOL_ROWS` (shared by the render AND the ↑/↓/Enter clamp, so navigation can't address an
    unshown row). The picker reuses the ⌘P finder recipe (a modal owning the keyboard) + the #312
    navigation (`open_and_place_caret` + NavStack + encoding); multi-root — every Ready host is queried
    and merged.
  - **⌘T is Editor-scoped**, SHADOWING the global ⌘T new-tab (the ⌘D #265 pattern) — VS-Code-like symbols
    in the editor, Warp-like new-tab in a terminal.
  - **Handshake** advertises `workspace.symbol.symbolKind` (the full 1..=26 set) — an UPGRADE (the kind
    set), not an enable (rust-analyzer serves symbols without it).
- **LSP signature help (parameter hints, ⌘⇧Space) — a card above the caret while you type a call**
  (TICKET-324, forge #324; sprint M21). Type `foo(` and a card appears ABOVE the caret showing the
  signature with the ACTIVE parameter lit; type args + commas and it updates and stays put; leave the
  call's line and it's gone. The #313 park-and-consume shape with a card instead of a menu. Live-proven
  against rust-analyzer: `s.push_str(` → `fn push_str(&mut self, string: &str)` with `string: &str`
  highlighted, persisting as the argument is typed.
  - **Pure seam** `marley_lsp::signature_help` (cov/MSI 100): `parse_signature_help` normalizes a
    `SignatureHelp` — `activeSignature`/`activeParameter` out of range reset to 0 (LSP 3.17), a
    per-signature `activeParameter` overrides the top-level one, and a `ParameterInformation.label` given
    as a string OR `[start,end]` UTF-16 offsets normalizes to a highlight range (out-of-bounds / inverted
    dropped, a mid-surrogate boundary snaps). `should_request_signature` (the trigger-vs-retrigger
    decision), `signature_help_params` (the `context`), `signature_help_support`, `signature_trigger_
    characters`.
  - **App picker** `marley_app::editor_signature`: `SignatureKey` (uri, version, caret — the stale guard),
    `SignatureCard` (↑/↓ cycle overloads + a "N of M" pip), `split_label` (the accent highlight),
    `signature_card_origin` (prefer-ABOVE, so the card never covers the args and never collides with the
    below-first completion popup). The shim parks on `(`/`,` (string/comment-suppressed) → pump-consumes →
    a card at a FIXED anchor above the call; ⌘⇧Space invokes manually; a bare ↑/↓ cycles (a modified arrow
    stays the editor's); the caret leaving the call's line, Esc, or any dispatched action dismisses.
  - **Handshake** advertises `textDocument.signatureHelp` (`labelOffsetSupport` + `activeParameterSupport`
    + `contextSupport`) — the #323 capability lesson, so a real server returns the offset labels + the
    per-signature active parameter.
- **LSP code actions + quick fix (⌘.) — the picker + codeAction/resolve, applying via the #322 engine**
  (TICKET-323, forge #323; sprint M21). ⌘. at the caret asks the server for its fixes/assists (auto-
  import, qualify-path, fill-match-arms, extract…) and opens a picker; ↑/↓ move, Enter accepts, Esc
  closes. Accepting applies the fix through the **#322 WorkspaceEdit engine, unchanged** — no second
  applier. Where auto-import lands. Live-proven end to end against real rust-analyzer: ⌘. on an
  unresolved `HashMap` → a picker of "Import `std::collections::HashMap`" (preferred, first) + "Qualify
  as …" → Enter → the deferred edit resolves and the `use` line is inserted, "1 file changed".
  - **Pure seam** `marley_lsp::code_action` (cov/MSI 100): `parse_code_actions` normalizes a server's
    `(Command | CodeAction)[]` into a rankable set — a usable action carries a `title` and can produce an
    edit (an eager `edit`, or a NON-NULL `data` blob to resolve one); a bare `Command` is skipped and
    counted ("N unavailable" — v1 does not execute commands), a `disabled`/malformed element is dropped;
    the kept set sorts `isPreferred`-first then by kind rank (quickfix < refactor < source < other).
    `needs_resolve = edit.is_none() && resolveProvider`; `resolved_edit` reads the filled `edit` off a
    resolve answer; `code_action_params` builds the range + `context.diagnostics` (the caret-row
    diagnostics via `diagnostic_to_lsp`) + `triggerKind`; `code_action_support` reads the capability.
  - **App picker** `marley_app::editor_code_action` (pure): `CodeActionKey` (uri, version, caret — the
    stale guard) + `CodeActionMenu` (↑/↓ wrap, `chosen`, `visible` via the shipped `complete::popup_
    window`). The shim requests on ⌘. (Editor-scoped), opens the picker at the caret, and on Enter either
    applies an eager `edit` directly or sends `codeAction/resolve` (its OWN request slot) and applies the
    resolved edit — both through the #322 engine. The picker owns the keyboard (the DefPicker modal
    contract); a superseded codeAction/resolve answer is dropped by its key.
  - **Handshake** now advertises `textDocument.codeAction` (`codeActionLiteralSupport` + `isPreferred
    Support` + `dataSupport` + `resolveSupport.edit`) — without it a real server withholds `CodeAction`
    literals and ⌘. is dead (found by the live drive). Sibling of the #322 `documentChanges` advertisement.
- **LSP rename symbol (F2) — prepareRename + the multi-file WorkspaceEdit engine** (TICKET-322, forge
  #322; sprint M21). F2 on a symbol renames every occurrence across the whole workspace — the open ones
  through their editor buffers (one ⌘Z reverts a file), the closed ones written to disk — as one action
  per file. An inline draft (the `renaming_tab` idiom) opens seeded with the symbol's current name from
  the server's `prepareRename` range; type the new name, Enter sends `textDocument/rename`, Esc cancels.
  The M21 foundation: it builds the ONE WorkspaceEdit apply path that #323 code actions (and the
  import-organizing half of #314 formatting) reuse.
  - **Pure engine** `marley_lsp::workspace_edit` (cov/MSI 100): `parse_workspace_edit` normalizes BOTH
    `changes` and `documentChanges` shapes into per-file edit lists (a resource op —
    create/rename/delete — rejects the whole edit v1; a malformed element is skipped; a `2^32` position
    is skipped, not truncated); `resolve_text_edits` maps LSP ranges to char offsets through the #309
    encoding bridge, sorts last-to-first, and rejects overlap (adjacency survives) or an inverted range;
    `apply_text_edits` splices; `rename_support`/`parse_prepare_rename` read the caps + the 3 prepare
    shapes. The shared `resolve_text_edits` is what makes #314/#323 a thin reuse, not a second applier.
  - **The applier routes open-vs-closed by CANONICAL path identity** (`same_file`) — an open file under
    a symlinked root (`/tmp`→`/private/tmp`) must edit its buffer, never blind-write to disk (inspect
    caught this as a HIGH file-corruption bug). Closed-file writes are contained to a workspace root
    **after canonicalization**, so a server-named `../` traversal can't escape it (a second inspect fix).
    A version-mismatched open doc is skipped (REQ-009); the result flashes `renamed in N files (M
    skipped)`.
  - Validated live against real rust-analyzer: F2 renamed a `fn` across two files — the open file's
    buffer (⌘Z-reversible) and a closed file on disk — off the real `prepareRename`/`rename` frames.

- **LSP completions — the as-you-type popup with fuzzy ranking + a TextEdit-faithful accept**
  (TICKET-313, forge #313; sprint M20). Typing an identifier — or one of the server's trigger
  characters (`.`, `::`) — in an editor tab sends `textDocument/completion` and opens a popup under the
  caret: fuzzy-ranked, kind-glyphed, detail right-aligned, ↑/↓/Enter/Tab/Esc. Accepting **replaces what
  you typed** rather than appending to it, as one undo unit. The third consumer of the #311
  request→response path.
  - **Pure seams** (100% coverage/MSI): `parse_completion_result` accepts BOTH real response shapes
    (`CompletionList{isIncomplete,items}` and a bare `CompletionItem[]`) and normalizes the FOUR insert
    forms (`textEdit` / `InsertReplaceEdit` preferring `replace` / `insertText` / label-only) into one
    `CompletionEdit`; a malformed element is skipped per-item, so one bad entry cannot lose the list,
    and an INVERTED server range is rejected at the seam (it panicked ropey — inspect F6).
    `strip_snippet_markers` handles nested placeholders (`${1:${2:nested}}`) and `\$` escapes.
    `completion_trigger` is the one trigger table; `CompletionMenu` ranks via `fuzzy_rank` (the finder's
    matcher, one matcher everywhere) then the LSP tie-break `sortText` → label; `popup_origin` FLIPS
    above the caret near the window bottom (`menu_origin` only clamps, which would hide the word being
    completed).
  - **Accept extends the server's range to the LIVE caret** (inspect F1) — the popup survives typing, so
    a range captured at request time is stale by the time you press Enter; `s.` → popup → type `p` →
    Enter must give `s.push`, not `s.pushp`.
  - **Dismissal polls the live editor identity** (path/version/**caret**) on the pump rather than
    hanging off `dispatch_action` — inline arrows, mouse clicks, paste, undo and N-cursor edits never
    reach the action path, and an accept against a stale menu edits where you are not looking.
  - **Nothing opens inside a string or comment**, gated on the tree-sitter highlight cache. The check is
    deferred one tick because the cache refreshes in the RENDER, so an inline gate is stale on every
    keystroke; a lost render race falls OPEN rather than suppressing (documented, not silent).
  - Esc closes and suppresses re-open for the rest of the word — and the suppression **expires at the
    word's end** (inspect F8), so one Esc cannot silence completions for the rest of the file.
  - Verified live against a real rust-analyzer through an `[[lsp.servers]]` tee: the actual
    `textDocument/completion` frame carries a correct position + `triggerKind: 2`, the accept is
    byte-exact on disk, and a `.` typed inside a comment sends nothing while the same `.` one line down
    in code opens the popup.

- **LSP go-to-definition (F12) + a NavStack jump-back (⌃-)** (TICKET-312, forge #312; sprint M20). The
  navigation that makes the editor feel like an IDE: F12 on a symbol sends `textDocument/definition` at
  the caret, opens the target file (across files), lands the caret at its position, and centers it; ⌃-
  returns to exactly where you jumped from. Several results (a trait with impls) open a `path:line`
  picker. The second consumer of the #311 request→response path — only the payload is per-feature.
  - **Pure seams** (100% coverage/MSI): `parse_definition_result` normalizes the THREE real LSP 3.17
    response shapes (`Location` | `Location[]` | `LocationLink[]`, the last preferring
    `targetSelectionRange` so the caret lands on the identifier rather than the doc comment) into a flat
    `Vec<DefLocation>`; a malformed/partial/hostile element is SKIPPED, so a bad payload degrades to "no
    definition found" and never to a wrong jump or a panic (an out-of-`u32` position is skipped, not
    truncated). `NavStack` (LIFO, capped at 50 dropping the oldest, a same-spot re-push deduped) and the
    `DefPicker` selection arithmetic are pure; `rpc::text_document_position_params` is the shared
    `TextDocumentPositionParams` shape hover now delegates to and #313/#317 will reuse.
  - **The caret is mapped through the #309 bridge, not `caret_for_line_col`** — an LSP `character` is an
    ENCODED column, not a 1-based char column, so a line with non-ASCII before the symbol lands wrong
    otherwise. Pinned by a headless drive whose target line is `let 🦀 = target;` (one char, two UTF-16
    units), where the answer's column 6 must resolve to char offset 29.
  - **The center is DEFERRED one pump tick** (the #273 trap: an open plus a same-frame scroll is wiped
    when the new file takes the shared scroll handle).
  - Keymap: F12 + ⌃- are Editor-scoped (they resolve to nothing off an editor tab). ⌘-click-to-jump is
    deliberately NOT taken — it is the shipped #297/#298 multi-cursor gesture.

- **LSP hover — types + docs in an overlay card (⌘K / mouse dwell)** (TICKET-311, forge #311; sprint
  M20). The first *interactive* LSP round-trip: ⌘K in an editor tab (or a ~400ms mouse dwell over a
  symbol) sends `textDocument/hover` and renders the type signature + docs in a rounded overlay card,
  code highlighted by the existing tree-sitter/lexer. Hovering a diagnostic span also shows its message.
  - **A general request→response→consumer path on the LSP host** — until now only the `initialize`
    response was handled; #311 adds a purpose-tagged `request()` + a drained `responses` queue + the
    non-init response arm, so a client request routes its answer back to a consumer. This is the
    reusable wire #312 goto / #313 completions / #317 references build on; hover is its first user.
  - **Pure seams** (100% coverage/MSI): `markdown_runs` (a deliberate markdown-lite subset — fenced
    code blocks with their language, headings, paragraphs, `---` separators; an unclosed fence or any
    malformed input renders as plain text, never a panic), `parse_hover_result` (the three
    `Hover.contents` shapes → one markdown string; empty → nothing), and the app-side `hover_dismiss`
    table + the `HoverKey` stale-guard + the dwell threshold.
  - **The card is the #221 overlay recipe**, window-clamped via `menu_origin`, fenced code highlighted
    through `code_syntax`. Dismissed by an edit, Esc, a caret move, a scroll, a file switch, or the
    pointer leaving — via a pump-tick poll on the LIVE editor identity (file + caret + version +
    scroll), since arrows/typing/list-scroll/rail-clicks all bypass the action dispatcher (inspect F1).
    A stale response (the user moved on) is dropped.

- **LSP diagnostics → the M18 gutter/F8 lane + squiggles + a status count** (TICKET-310, forge #310;
  sprint M20). A language server's `textDocument/publishDiagnostics` now lands in the editor: a colored
  underline under each diagnostic span, a gutter-row tint + F8/⇧F8 navigation (the SECOND producer into
  the M18 #289 terminal-diagnostic lane), and a `"N errors, M warnings"` count in the cockpit footer.
  Built on the #309 position bridge, so the squiggle column is emoji-correct (a UTF-16 column maps to
  the right `char` column against the live line text).
  - **A pure diagnostics model** (`marley_lsp`'s `diagnostics` module): a per-PATH store (a publish
    REPLACES a file's set; an EMPTY publish CLEARS it — the stale-squiggle guard), the Marley `Diag`
    value (the app never sees `lsp-types` — the ONE seam is the publish parser), the row union with the
    terminal lane (`merged_rows`), the per-row underline spans (`row_underlines` — multi-line spans
    span each row; a zero-width span becomes a 1-char squiggle), and the status count. Tested to 100%
    coverage/MSI.
  - **Keyed by canonical PATH, not the wire uri string** (inspect F1). The store is written with the
    path DECODED from the published uri (a new pure `path_from_file_uri`, the inverse of `file_uri`)
    and normalized through the SAME `absolute()` the lookup uses — so a server that re-encodes the uri
    (percent-escaping an `@scope` path, a space) or resolves a symlink can't silently drop every
    squiggle/gutter/count. Mirrors the #309 `docs` map (also `absolute`-keyed). A headless integration
    test drives a real `publishDiagnostics` through the whole capture path and asserts the editor
    accessors see it (and an empty publish clears it), pinning the write-key ↔ read-key agreement.
  - **The squiggle is an overlay bar, not a text highlight** — gpui's `with_highlights` requires
    disjoint sorted runs, but diagnostics overlap; the underline is an absolute-positioned `div` on the
    mono cell grid (the #250 caret-bar primitive) via the tested `col_of_offset` map. Error →
    `danger`, lesser severities → `muted` (the amber `warning` role + a wavy squiggle land with the
    #316 theme work). Diagnostic message capture + hover are #311.

- **LSP document sync + the UTF-16 position bridge** (TICKET-309, forge #309; sprint M20). Every open
  editor file now flows to the language server as a `textDocument` lifecycle, and — the load-bearing
  part — every position crossing the wire goes through ONE pure, encoding-aware bridge (`marley_lsp`'s
  `position` module): a buffer `char` offset ↔ the LSP `character` column in the server's negotiated
  encoding (UTF-16 by default, where an emoji is one `char` but two code units). This is the seam
  every later M20 feature (#310 diagnostics, #311 hover, #312 goto, …) addresses through.
  - **Sync is a pump reconciliation, not per-event hooks.** Once the server reaches Ready, each frame
    diffs its open-document set against the editor's open files — didOpen for a newly-seen rust file,
    a full-text didChange when a buffer version advances, didClose for a dropped file — so the async
    spawn (a file is open ticks before the server is) and open/change/close all fall out of one diff.
    ALL open files sync (not just the focused one), so switching files never churns close/open, and
    the whole-buffer text is materialized only for a doc that actually changed this tick (idle frames
    copy nothing). didSave rides the save path; paths are canonicalized so a `/tmp`↔`/private/tmp`
    spelling can't drop or double-key a file.
  - **Full-text didChange in v1** (incremental deferred — the unbounded #269 delta log carries no
    inserted text; a `{text}`-only change is spec-valid under any sync kind and rust-analyzer accepts
    it). The pure bridge (`offset↔position`, `column_in_encoding`, clamping, round-trip) + the doc-sync
    notification builders are tested to 100% coverage/MSI; live-driven to `lsp: ready` with the
    per-tick reconcile running crash-free.

- **LSP client core — the M20 language-intelligence foundation** (TICKET-308, forge #308; sprint M20). A new
  `marley_lsp` crate brings up the wire every later M20 feature (#309–317) speaks through: it spawns a
  language server (rust-analyzer in v1) and moves `Content-Length`-framed JSON-RPC over stdio, with a
  lifecycle that spawns → handshakes → stays ready, and restarts a crashed server on a capped budget. No
  language features yet by design — this ticket is the transport, proven live by an `LSP:` status-bar readout.
  - **Clean-room from the published spec.** Implemented from the Microsoft LSP 3.17 specification + the MIT
    crates.io `lsp-types` crate — not translated from any copyleft reference (§20).
  - **A dumb transport, a smart split.** The crate knows nothing of buffers, files, or languages; it is pure
    decision seams (an incremental `Content-Length` frame decoder, the JSON-RPC route + pending-request table
    with timeout + cancel, the spawn→initialize→ready→crashed lifecycle machine with a sliding 3-per-60s
    restart cap, the `initialize` handshake) tested to 100% coverage/MSI, over a thin `#[mutants::skip]`
    process shim (piped stdio pumped on plain threads, killed + reaped on drop) verified end-to-end by a
    scripted fake-server integration suite.
  - **Configured per-language from day one** — a `[[lsp.servers]]` settings block (round-trips; a hand-edited
    entry omitting `args` loads clean, never wiping the list) so #315's languages attach servers without a
    schema break. Default = `rust-analyzer` resolved on `PATH`, then `~/.cargo/bin` (a GUI-launched app's PATH
    often lacks it); absent → a quiet `lsp: not found`, never a crash. Requires a workspace `Cargo.toml` to
    attach (what rust-analyzer needs anyway).
  - **Inspect hardening:** an `initialize` **error** reply now tears the wire down (restart cap governs)
    instead of falsely reporting ready; the PATH search returns only **absolute** commands so a hostile repo's
    relative-PATH `rust-analyzer` can never be executed against the workspace cwd; the frame decoder caps a
    header that never terminates (memory-DoS guard).

- **⌘/ toggles line comments — aligned at one column, over every cursor, in one undo unit** (TICKET-299,
  forge #299; sprint M19). Arguably the most-pressed editing chord after save, and Marley did not have it. It
  comments (or uncomments) every row touched by **every** cursor, in ONE undo unit.
  - **The markers align at ONE column** — the *minimum* indent of the touched rows — instead of staircasing
    down each row's own indent, which is what makes it look right. Each row keeps its **relative** indent,
    because uncomment strips the token **and at most one following space**: that "one" is exactly what makes
    uncomment an *inverse* of comment-at-minimum-indent. (`    //     bar();` → `        bar();`, not
    `    bar();`. Strip all the following whitespace instead and the deep row silently loses its indent.)
  - Blank **and whitespace-only** rows are skipped — never given a bare token, and never allowed to drag the
    marker column down to their own depth.
  - **Language-aware through the ONE table that already existed** (`code_syntax`'s `lang_spec().line_comment`,
    shared with the syntax highlighter — never a second source of truth for "what is a line comment in Rust").
    `.rs` → `//`; `.toml` / `.sh` / `.bash` / `.zsh` → `#`. JSON, Markdown and Plain have no token, so ⌘/ is a
    **true no-op** there: text, buffer version and the dirty flag all untouched. Never mangle text we cannot
    comment.
  - Reuses `indent.rs` wholesale (`LineEdit`, `line_span`, `rebase_selections`) — a comment toggle *is* a
    line-prefix edit, structurally identical to Tab — plus a new `touched_rows`, the deduped N-cursor union.
  - **Proven on live pixels** (a ragged 4/8/4 block, three cursors, one column, byte-identical round trip).
    GATE GREEN [diff] 15/15, coverage 100 / MSI 100.

- **⌘D adds the next occurrence as a CURSOR — the additive multi-select; ⌘⇧L takes them all** (TICKET-298,
  forge #298; sprint M19). #272 shipped ⌘D as a single-selection *advance*: the selection jumped to the next
  match, replacing itself. It now **KEEPS what you have and ADDS** — select a word, press ⌘D a few times, type
  once, and every occurrence changes. **⌘⇧L** selects every occurrence at once. Both are Editor-scoped, so on a
  terminal ⌘D still opens a new terminal and ⌘⇧L still splits the pane; the cost, stated plainly, is that while
  an editor pane is focused **⌘⇧L no longer splits** (it remains on ⌘⇧P → "Split Right").
  - **`find_all` is now the ONE match engine.** Every consumer — the find bar, ⌘D, ⌘⇧L — asks it what an
    occurrence *is*, and states its case choice at the call site via an explicit `fold` flag: the find bar
    passes `true` (its shipped ASCII-case-folding), and ⌘D/⌘⇧L pass `false`. These gestures exist to be **typed
    over**, so silently grabbing `FOO` when the user selected `foo` would rewrite text they never targeted.
  - **#265's `select_next_match` and its wrapping single-match scan `next_occurrence` are RETIRED.**
    `next_occurrence` probed *every* index, so it returned SELF-OVERLAPPING matches (`"aa"` at 0 **and** at 1 in
    `"aaa"`) that `find_all` never yields — the two gestures disagreed about what an occurrence is, and feeding
    an overlapping match to the set invariant MERGED it, silently growing the user's selection past what they
    had selected. One engine, one definition.
  - Rests on #297's caret/range-**asymmetric** merge: `find_all("aa")` over `"aaaa"` yields the *touching*
    matches `(0,2)`+`(2,4)`, which must survive as **two** cursors. Under #296's original rule they collapsed
    into one, and this gesture was impossible.
  - **Proven on live pixels**: ⌘D ⌘D → `let ⟦rope⟧ = buffer.⟦rope⟧();` → one typed run rewrote **both** →
    one ⌘Z reverted **both** → ⌘⇧L selected every `rope` in the file. GATE GREEN [diff] 15/15, coverage 100 /
    MSI 100.

### Fixed

- **LSP `didOpen` carried an EMPTY document — so every hover and definition answered null in the real
  app** (found by #312's live drive; the bug is #309's, shipped three commits earlier). The pump
  collected the `needs_text`-gated buffer text BEFORE `host.drain()` — but a host reaches `Ready`
  *inside* `drain()` (the `initialize` response is what promotes it), and both `needs_text` and
  `reconcile` are Ready-gated. On the exact tick a server went Ready the collector still saw pre-Ready
  and materialized nothing, while `reconcile` — Ready by then — sent that empty string as the document
  *and recorded it as synced*, so no `didChange` ever followed until the user happened to type. The
  empty document was therefore permanent: rust-analyzer held every open file as zero-length and answered
  null to everything, leaving hover (#311) and go-to-definition (#312) dead outside the tests.
  - **Fix: drain first, then collect, then reconcile.** The #309 hot-path optimization (materialize the
    buffer text only for a doc the host actually needs) is kept; the race is gone.
  - **Proven on the wire**, before and after, via a tee wrapper installed through the app's own
    `[[lsp.servers]]` config: three `didOpen`s at `TEXT_LEN=0` (whole client→server log 1226 bytes)
    became one `didOpen` of `Content-Length: 16781` carrying real source (log 104220 bytes).
  - **No existing test could have caught it**: every M20 drive injects a synthetic response into a
    process-less host, so none exercises the real didOpen→server→answer round trip — which is why
    #309/#310/#311 all shipped "driven-proven" with this live. The diagnostic tell was that a
    definition in the *same file* came back empty (it needs nothing but the document itself). Structural
    hardening + the missing Ready (`fake_ls`) test lane are tracked as **#320**.

- **An undo group could absorb the next typed character and DESTROY text** (TICKET-299; the flaw shipped in
  #297 and #299 was the first thing to reach it). Toggle a comment at two cursors, type one letter, press ⌘Z —
  and the editor deleted your code:

  ```
  "foo\nbar"  →⌘/→  "// foo\n// bar"  →type X→  "// foXo\n// baXr"  →⌘Z→  "oXo\naXr"
  ```

  The `f` and the `b` are gone, and the undo stack is drained, so a second ⌘Z cannot bring them back.

  `UndoHistory::coalesces_into` decided *"is this a typed run continuing?"* from **shape** — bracketed, record
  count equal to the cursor count, selections lining up, every record a pure insert whose new insert is one
  char — and stated its actual precondition only in prose: *"because the cursors did not move, each new insert
  lands precisely at the end of its own record's text, so appending char-wise is contiguous by construction."*
  That is true for a typed run and **false** for a group whose records are anchored at the min-indent *column*
  rather than at the cursors. The typed char was appended onto a record whose offset was somewhere else, and
  undo then deleted the wrong characters. Its own single-cursor twin, `record()`, has always **checked** that
  contiguity; the grouped path merely **argued** it.

  Fixed by making the producer **state** the property — `UndoGroup.cursor_anchored`, set only by the N-cursor
  typed-insert path — instead of having the consumer re-derive it from geometry. That kills the whole class:
  every future hand-bracketed line op (multi-cursor Tab, move-line, delete-word) would otherwise have walked
  into it. #297's typed-run coalescing, the entire reason the guard exists, is preserved and pinned.

- **⌘/ irreversibly shredded five kinds of marker** (TICKET-299). The reference editors' naive prefix test
  treats *anything* starting with the comment token as "already commented", and uncommenting then eats one
  token — with no way back:

  | line | naive ⌘/ gave | recoverable? |
  |---|---|---|
  | `//! PURE — …` — a Rust **module doc**; the first line of literally every file in this crate | `! PURE — …` | no |
  | `#!/bin/sh` — a **shebang**; the script silently stops being executable and fails at *exec* time | `!/bin/sh` | no |
  | `/// doc` | `/ doc` | no |
  | `//////` · `####` — a banner rule | one token shorter *per press*, down to `""` — and a blank row is an absorbing state | no |
  | `## Section` | `# Section` → `Section` | no |

  Marley **deliberately diverges** from the reference here (§0 — do not ship known, irreversible harm): the
  token must not be followed by more of itself, nor by `!`. Those are richer markers, so ⌘/ *comments* them
  (`//! x` → `// //! x` → back to `//! x`) rather than shredding them. Every case is now a clean two-cycle, and
  a plain `// x` / `//x` / `# x` still uncomments exactly as before.

- **`word_range_at` carried an unkillable mutant since #265** — a latent MSI hole, invisible because the
  mutation gate only mutates files in the diff and `find.rs` had not been touched since. `pivot + 1` →
  `pivot * 1` was an *equivalent* mutant: the pivot's char is a word char by construction and the forward scan
  re-tests it, so both forms land in the same place and no test could ever kill it. Fixed by **deleting the
  redundant arithmetic** rather than suppressing the mutant — with each scan re-testing its own boundary char
  the pivot needs no ± 1 at all, so the whole `pivot` branch collapses into an `on_word`/`after_word` guard.
  Shorter code, no unkillable operator, identical behavior (TICKET-298).

- **Multi-cursor, end to end — the gestures, the shim, and the pixels** (TICKET-297, forge #297; sprint M19).
  #296 gave the editor crate an N-cursor `SelectionSet`, but **no key in Marley could create a second cursor**
  and the app still carried a single caret per open file. This makes multi-cursor real:
  - **⌘⌥↑/↓** add a cursor above/below at a sticky **goal column**; **⌘-click** adds one, or removes the one
    already there (never the last); **Esc** — and a plain click — collapses back to one.
  - **Every edit acts at every cursor, in ONE undo unit**: typing (which reaches the buffer only through the
    platform text path, `insertText:` → `ime::replace_text` — the app's single insert mechanism), backspace,
    paste, cut, Enter's *per-cursor* auto-indent, find-replace. Motion moves every cursor. A single ⌘Z reverts
    all N **and restores all N cursors**, and a typed *run* at N cursors undoes as one word, exactly as at one.
  - **The `Buffer` is now the single owner of the cursor set.** `OpenFile` used to keep its own `caret` +
    `anchor` beside a `SelectionSet` the buffer already had and the app never read — two sources of truth, one
    vestigial. Deleting the duplicate dissolved the E0499 three-way `&mut` accessor that 17 call sites existed
    to work around, and made a real bug (a file:line jump leaving a stale anchor, painting a selection you never
    made) *unrepresentable* rather than merely fixed.
  - `SelSnapshot` is **deleted** — the undo history carries a whole `SelectionSet`, which also retired the two
    helpers #296 had been forced to write to bridge the mismatch.
  - The goal column is what stops multi-cursor destroying itself: without it, two cursors crossing a short line
    both clamp to its end and lose the column they wanted.

  **Proven on live pixels**, not mechanism: ⌘⌥↓ → two carets rendered at the same column → type → both lines →
  one ⌘Z → text *and* both cursors restored → Esc → one caret → ⌘-click → a cursor added. GATE GREEN [diff]
  15/15, coverage 100 / MSI 100.

  Adversarial review found **ten real bugs (six HIGH) in code where the gate was fully green** — including one
  #296 shipped: `from_selections` merged two merely *touching* ranges, so ⌘⌥↓ followed by ⇧↓ destroyed its own
  second cursor on the very next keypress, unrecoverably (a motion, not an edit). **A test asserted that
  behavior and thereby defended it**, with a rationale that turns out to be false — two abutting ranges replace
  *disjoint* spans and the back-to-front sweep applies them cleanly. The merge is now caret/range-asymmetric:
  `<=` only where a caret is involved, `<` for two ranges.

- **Multi-cursor core — the ordered-disjoint `SelectionSet` + the N-caret edit** (TICKET-296, forge #296;
  sprint M19). The editor crate had been carrying its own deferral in a doc comment since M1: *"`SelectionSet`
  always holds exactly one member; the multi-member sort/merge constructor is deferred to M1.B."* This lands
  M1.B — the keystone the rest of M19 (gestures, ⌘D-adds-cursor, the multi-cursor-aware comment/move-line/
  delete ops) is built on. Three pure additions in `crates/editor`:
  - **`SelectionSet::from_selections`** — sorts by `start()` and merges into a set that is **ordered · disjoint ·
    non-empty**, the invariant every N-cursor op relies on. The merge boundary is `<=` rather than `<`, which is
    *required*: two carets at the same offset satisfy `next.start == cur.end`, and leaving them unmerged would
    make one logical cursor insert its text twice. The price — two merely adjacent ranges also union — is the
    safe direction to err (a lost cursor cannot corrupt text; a duplicate one can) and is documented on the fn
    with the exact refinement ⌘D will need.
  - **`Buffer::edit_at_selections`** — applies an edit at all N members **back-to-front** inside ONE
    `begin/end_undo_group`, so ⌘Z reverts every cursor in a single step. Back-to-front needs **no rebasing at
    all** (an edit at a higher offset cannot shift a lower one) — the idiom the shipped `find::replace_all`
    already used, so the plan's harder cumulative-rebase design was dropped at grounding.
  - **`selections_after_multi_edit`** — the one real computation: where the N carets land afterwards, as a pure
    `usize` forward shift (two monotonically increasing accumulators, no signed cast that could silently wrap).
  Proven by a **50k-step differential fuzzer** against an independent front-to-back oracle that rebases through
  the real `rebase_offset` and the actual `BufferDelta`s — plus, after an adversarial inspect pass the green
  gate could not have caught, hard enforcement of the invariant itself: a clamp is not injective, so every path
  that clamps now re-canonicalizes (two out-of-bounds cursors used to collapse into duplicates and type twice),
  and `edit_at_selections` clamps its incoming set (a post-undo selection legitimately points past EOF and used
  to panic ropey). Coverage 100 / MSI 100. **Pure crate only** — the app surface, the N-caret render, and the
  gestures that can actually create a second cursor land together in #297, where they can be driven and proven
  on pixels. GATE GREEN [diff].

### Fixed

- **Editor caret-follow scroll — the caret stays on screen after a motion**
  (TICKET-270, forge #270; sprint M18). The editor's uniform_list scrolled only via the wheel; no caret
  motion adjusted it, so a far motion (⌘↓ to document end, a wrapping ⌘D select-next, Down past the
  viewport) moved the caret off-screen — the press looked like a no-op though the state was correct.
  #273 had already built `scroll_editor_to_row` (the shared non-strict gpui scroll whose doc names "#270
  caret-follow" as a consumer) but the motion sites never called it. Now a `follow_editor_caret()` shim
  scrolls the caret's row into view (a no-op when already visible) after each keyboard caret motion — the
  #257 key branch (Up/Down/Word/Home/End), the platform ⌘-motion branch (⌘↓ doc-end, ⌘←→ line), and the
  #272 ⌘D. Proven by a headless test (a 120-line file → ⌘↓ → the list follows to the bottom). The
  prerequisite for #290's diagnostic navigation. GATE GREEN [diff].

- **Terminal pane grid sizes to the content rect — the alt-screen top row + top-of-pane clicks are correct**
  (TICKET-287, forge #287; sprint M17). Pre-existing since M5 #108: the resize loop fed `plan_resize` the
  FULL pane rect while the render carves the 24px title bar off the content div, so the PTY grid was
  `floor(pane.h / cell_h)` rows — ~1 too many. Consequences: a full-screen TUI's true TOP row was clipped
  off-screen (the grid is bottom-packed under `overflow_hidden`), and #280's `pane_mouse_cell` (which trusts
  `pty_size`) inherited the ~1-row skew for clicks near the top of a tall pane. Fix: a pure
  `inset_top(rect, inset)` (mirroring `inset_right`) carves the title bar, and the resize loop now sizes the
  PTY to `inset_top(pane, PANE_TITLE_H)` — the same content rect the render draws into. ONE corrected
  `pty_size` heals both (the render + mouse mapping already consume it). `inset_top` unit + a `plan_resize`
  carved-height golden at cov/MSI 100. GATE GREEN [diff].

- **Incremental highlighting: a shrinking edit no longer drops the surviving token (CRITICAL #285 fix)**
  (TICKET-288, forge #288; sprint M17). #285's windowing assumed `changed_ranges ∪ edit-span` covered every
  stale span — FALSE for a SHRINKING edit. Backspacing characters off the end of a comment made
  `changed_ranges` EMPTY (only the node's extent shrank; its bytes are textually unchanged), the damage
  window degenerated to `[p, p)`, the windowed query returned nothing (tree-sitter's intersect excludes a
  node ending EXACTLY at an empty window edge), and the splice dropped the stale cached comment with nothing
  to replace it — the highlight VANISHED. It also hit a token merely ADJACENT to the edit that re-tokenizes
  when a delimiter is removed (deleting a `*/` swallows the following code into the comment). Fix: a pure
  `cover_edited_cached` widens the window to the surviving new-coord footprint of every cached span
  OVERLAPPING OR ABUTTING the replaced region `[start, old_end]`, and `snap_to_lines` snaps it to whole-line
  bounds (same-line boundary re-tokenization). A deterministic in-test differential fuzzer
  (`t288_differential_fuzz`, tree-match-guarded) is the durable regression — the exact shape that FOUND the
  bug; it now reports ZERO same-tree divergences across ~120k adversarial steps. Equivalence is scoped to
  "same incremental tree ⇒ same spans"; tree-sitter's incremental parser can itself diverge from a fresh
  parse on rare error inputs (pre-existing since #274, not this crate's windowing). cov/MSI 100 on the new
  seams; the #274 corpus + perf pin stay green. GATE GREEN [diff].

- **Honor DECCKM (application cursor keys) — SS3 arrows + the alt-scroll fallback**
  (TICKET-286, forge #286; sprint M17). Follow-up to #280 inspect F2: two consumers ignored DECCKM
  (application cursor-key mode, DECSET 1). `encode_key`'s unmodified arrows / Home / End always sent the
  legacy CSI form (`ESC [ A`), and the #280 alternate-scroll wheel fallback always sent CSI A/B — but the
  xterm spec sends the SS3 form (`ESC O A`) when a full-screen program (vim's smkx, less) has enabled
  application cursor-key mode. Now a single flag threads `TermMode::APP_CURSOR` from the session snapshot:
  `MouseModes.app_cursor` + `session.is_app_cursor()` + an `encode_key(input, app_cursor)` param (the
  `paste_bytes(text, bracketed)` precedent — terminal state as a param, not baked into `KeyInput`); a
  shared `cursor_key_bytes(final, app_cursor)` puts the SS3-vs-CSI rule in one place used by BOTH the
  unmodified cursor branch and the alt-scroll fallback. A MODIFIED cursor key stays CSI (`ESC[1;<param>X`)
  even under DECCKM, per xterm. Wire-only (both forms drive vim identically, so no visual delta); every
  pre-#286 byte is unchanged with `app_cursor=false`. Goldens for all six keys × both modes + modified +
  a `MockPtyChannel` DECSET 1 round-trip; cov/MSI 100. GATE GREEN [diff].

- **External-change acknowledgments bind to the observed disk state — a newer write re-warns**
  (TICKET-284, forge #284; sprint M17). Follow-up to #275: the ⌘S-under-conflict ARM and the banner's
  Keep-mine acknowledged whatever was on disk at press/click time, not the state the `Changed` warning
  DESCRIBED. So an agent write W1 flags the banner → you arm (1st ⌘S) or click Keep-mine → the agent lands
  a NEWER write W2 → the armed 2nd ⌘S overwrote W2 under W1's license, and Keep-mine re-snapshotted W2
  sight-unseen. Now each acknowledgment binds to the observed `(mtime, len)`: a pure
  `acknowledgment_is_stale(acked, now)` check, an `armed_at` pair on the arm (was a bool), and a
  `conflict_observed` field the `Changed` banner carries. A stale armed press DISARMS + re-warns ("File
  changed again — ⌘S to overwrite") with no write; a stale Keep-mine re-flags (refreshes the observed state,
  keeps the banner) instead of dismissing. An unchanged disk proceeds exactly as before. Extends #275's D3
  ("the next external change re-flags") to in-flight acknowledgments; inspect also disarmed the racing-write
  branch so a successful write always consumes the arm. Pure decision at cov/MSI 100 + headless
  W1-arm/W2-lands/press-2-re-warns and Keep-mine-re-flags flows. GATE GREEN [diff].

- **Editor tab: modified Enter/Tab/Backspace no longer act on the hidden terminal prompt**
  (TICKET-283, forge #283; sprint M17). A #251-era leak surfaced by #276's routing critic: the editor
  key-router is gated `!platform && !control`, so a MODIFIED Enter/Tab/Backspace (⌘Enter, ⌃Enter,
  ⌘Backspace, ⌃Tab) on an editor tab skipped it and fell to the terminal handlers, which resolve
  `focused_terminal` to a HIDDEN pane (#71) — ⌘Enter SUBMITTED its invisible prompt, ⌘Backspace edited it,
  ⌃Enter/⌃Tab wrote raw bytes to the hidden PTY. A pure `swallow_hidden_prompt_key(is_terminal, key)`
  predicate + a one-line shim gate at the top of the terminal fallthrough (mirroring the #278
  `grid().is_some()` gate) now swallows exactly those keys on a non-terminal tab; a terminal tab is
  byte-identical and every other key keeps its #267 fallthrough. The fix also covers cockpit tabs (same
  hidden-pane leak) for free. Cov/MSI 100 on the predicate + a headless flow (type into the terminal,
  open a file, the four chords leave the hidden prompt unchanged). GATE GREEN [diff].

- **Wide glyphs (CJK/emoji) stop drifting the caret, click, selection, and highlights**
  (TICKET-277, forge #277; sprint M17). #267 made CJK input work; every wide glyph still counted
  ONE display column in the #250 `line_layout`, so on wide lines the caret bar, click→caret
  mapping, selection tint, find bands, and the #268 syntax remap all drifted left, compounding
  per glyph. `line_layout` now advances by the new `char_width` authority (UAX#11 via
  unicode-width 0.2.2 — wide/fullwidth 2, zero-width marks 0, control→1; tabs stay positional
  over the ACCUMULATED column so stops compose with wide prefixes; the display string is never
  padded — columns are a mapping). The plan's recon caught the ticket premise that "downstream
  follows for free" being wrong for `cols_to_bytes`: it was display-CHAR-INDEX based (columns
  only coincided under all-width-1) and every highlight band funnels through it — rewritten as a
  width-accumulating span-contains walk (an emitted endpoint never splits a glyph; a strictly
  interior band is empty; zero-width combiners ride their base cell on both edges, probe-verified).
  Inspect's conflation sweep then found the click path quantizing `x/cell_w` to an integer column
  BEFORE the nearest-boundary scan — putting a wide glyph's flip point at 25% instead of its
  midpoint — fixed by keeping the click in the float domain end-to-end (`offset_of_col_f`; the
  dead integer wrapper deleted). Bonus: #276's bare-caret Tab pad became width-correct
  automatically. ZWJ families (over-count) and VS16/keycap sequences (under-count) are the
  recorded grapheme-cluster follow-up; the terminal grid is alacritty's own. 6 width kill-list
  units + a headless CJK caret-walk flow; code_view.rs mutation 0 missed; suite 981/981;
  GATE GREEN [diff] 15/15. Font-advance pixels ENV-BLOCKED (machine locked) — batched for the
  unlock re-verify.

- **Per-file editor scroll memory is back — and the shared scroll mechanism lands** (TICKET-273,
  forge #273; sprint M17). Heals the disclosed #266 trade: the ONE shared editor scroll handle
  carried the offset across file-tab switches. Each `OpenFile` now parks its PIXEL offset
  (`base_handle.offset()` — the test-only `logical_scroll_top_index` is asserts-only) and the render
  detects owner transitions at a SINGLE choke point keyed by the #268 buffer nonce (no switch-site
  enumeration: strip clicks, opens, closes, project/workspace switches, restore all funnel through
  the next frame), restoring via `set_offset` clamped against the file's CURRENT line count (an
  external shrink can't restore into blank space — gpui's same-frame prepaint clamp is the backstop,
  cited). The transition also CLEARS any pending deferred `scroll_to_item` — a jump minted for the
  outgoing file would otherwise teleport the incoming one (found in inspect, and the constraint is
  documented on the new helper: open-then-jump must span two frames). Ships **pub
  `scroll_editor_to_row(row)`** (non-strict Center: no-op when visible) — the one mechanism #270
  caret-follow, #272 find-next, and the #212/#213 open-at-line fusion consume. Bonus stability fix
  (the #271-noted arm, stack-sampled live when the headless lane wedged): the VIRGIN-boot launcher
  path now reaps its unused boot PTY **off-thread** like the restored path — its scope-end on-thread
  drop could block the whole constructor under orphaned-child load. Driven-capture proven
  (deep→switch→back restores the exact rows); headless flows + a 12-mutant kill list at MSI 100.

### Added

- **Grouped undo — one ⌘Z per block indent/dedent and per replace-all, with selection restore**
  (TICKET-282, forge #282; sprint M17). Multi-edit ops used to unwind one primitive at a time (a
  10-line Tab indent took 10 ⌘Z, a replace-all of N took N), and undo restored only the caret, never
  the anchor. `marley_editor::undo` is now a stack of `UndoGroup`s: an ordinary edit is a one-record
  group (single-char typing still COALESCES into one), while a bracketed transaction
  (`begin_undo_group`/`end_undo_group`) collects N edits into ONE undo unit that pops/redoes together
  and carries a pre/post `SelSnapshot` (anchor + caret). `undo`/`redo` return a `HistoryMove {caret,
  selection}`; the app's ⌘Z/⌘⇧Z arms apply the restored `(anchor, caret)` via `HistoryMove::ranged_anchor`
  — a grouped edit restores the selection SHAPE, an ordinary one collapses to the caret (which also fixes
  a latent stale-anchor phantom selection). Adopted for the #276 Tab/⇧Tab block indent/dedent arm and #272
  `replace_all` (the undo lands the caret at the first match). Inspect (two independent critics, converged)
  caught a redo-loss regression — `begin_group` cleared the redo stack eagerly, so a no-op ⇧Tab on a
  flush-left line (empty `edits`) wiped a pending redo with nothing to show for it; fixed by moving the
  redo-clear into `end_group`'s non-empty COMMIT branch, protecting every caller
  (`PR-claude-defer-redo-clear-to-transaction-commit-001`). Also fixed the earlier full-suite HANG risk: an
  existing headless test's stale `cmd-z cmd-z` (v1 per-line) assertion failed under the one-step group and
  panicked BEFORE `reap_sessions`, dropping a live session that blocks forever — updated to one `cmd-z`.
  Pure seam at cov/MSI 100 (the coalesce∩group guards, the reverse-order group replay, the selection
  snapshots, `ranged_anchor`) + a headless indent-then-one-⌘Z integration flow; the #276/#272 REQ wordings
  restored to the stronger one-step convention. Behavioral (keystroke→undo state), so the headless lane is
  the proof; GATE GREEN [diff] 15/15.

- **Mouse reporting to TUI programs — vim/htop/less finally get your clicks** (TICKET-280,
  forge #280; sprint M17, terminal polish #3, the LAST ticket of the /work 272-281 range). When
  a full-screen program requests mouse tracking (the DECSET flags alacritty already parses,
  never before consulted), the grid's clicks, drags, and wheel encode per the xterm ctlseqs
  spec — SGR 1006 (`ESC[<b;x;yM/m`), legacy X10 with its 223 clamp and identity-losing
  release=3, wheel 64/65, mod bits 4/8/16 — and stream to the PTY instead of driving Marley's
  selection/scrollback. With only ALTERNATE_SCROLL set in alt-screen the wheel falls back to
  arrow keys. ⇧ held bypasses everything (the universal copy-out-of-htop hatch); tracking off =
  byte-identical pre-#280 behavior (the #279 + R39 suites pin it). Drags throttle per CELL; a
  new pure `pane_mouse_cell` maps to the VISIBLE pty grid (bottom-anchored like #179 — distinct
  from scrollback content rows). Inspect (Sonnet): the mutants::skip DETACH TRAP struck a SIXTH
  time (pane_grid_pos's skip re-bound to the new fn — re-seated, --list-verified 0/28); the
  encoder matrix probe-verified byte-exact; two follow-ups minted — DECCKM (#286, the fallback
  + encode_key arrows together) and the M5-era pty_size title-bar skew (#287, ≈1 row clipped +
  mouse-row skew, one fix heals both). Validate caught cargo-mutants' package-scoping: the
  session accessor's kill test must live IN-CRATE (MockPtyChannel DECSET round-trip). GATE
  GREEN [diff] 15/15; the htop captures join the unlock batch. Left-button-only v1 (right keeps
  Marley's menu; documented).

- **Double-click word / triple-click line / ⇧-click extend in the terminal** (TICKET-279, forge
  #279; sprint M17, terminal polish #2). The universal selection trio on the grid: double-click
  selects the same-class run under the pointer with a path/URL-friendly word class
  (`./-_~:@?&#%` stay inside words so `/path/to/file.rs` and URLs grab whole; `()[]{}<>"',;=` +
  backtick form their own runs — clicking `=` in `a=b` selects just `=`, the documented
  query-string trade), triple-click selects the whole row (trailing spaces included — copy trims
  nothing the highlight shows), ⇧-click extends the head keeping the anchor sacred (xterm-style;
  ⇧ beats any click count). The whole decision is ONE pure fn (`click_selection`) feeding the
  existing #43/#44 pipeline — what highlights is exactly what copies, unit-paired. Inspect
  (Sonnet): the row-text fetch initially materialized the ENTIRE scrollback on every click —
  now gated to the double/triple arms only (single/⇧ clicks are zero-alloc again); ⌘-clicks fold
  to the single-seed arm per the design (the #196 link precedence untouched, traced); and the
  zero-viable-mutant classes (no Default derives) got coverage-by-intent — the D2 table asserted
  char by char. A missed equivalent mutant forced the bounds walk onto load-bearing take_while
  arithmetic. Word-snap drag after double-click = the recorded follow-up. GATE GREEN [diff]
  15/15; the live captures join the unlock batch.

- **New splits and tabs inherit the focused pane's live cwd** (TICKET-281, forge #281; sprint
  M17, terminal polish #4). Four dirs deep and splitting no longer dumps you back at the project
  root: splits (⌘⇧L/⌘⇧J + the context menu) and new terminals (⌘T/⌘D/"+") spawn in the focused
  pane's live prompt pwd (the #201 shell-integration source) when it's a real absolute directory
  — else the project root exactly as before (#160's behavior, now the fallback). ONE validation
  authority: the new `valid_dir_or` (non-empty ∧ absolute ∧ is_dir), with the #205 restore's
  `cwd_or_root` delegating — strengthened so a RELATIVE persisted cwd can no longer resolve
  against the app process's cwd. `launch_agent` stays at the project root deliberately (an
  agent's context is the repo); ⌘T from an editor tab inherits the most-recent working
  terminal's cwd (read-only — unlike the #278 class, nothing hidden mutates). The cwd flows as
  an exec-cwd PathBuf (never shell-interpolated). Proven end-to-end on real PTYs headless (cd →
  tracked pwd → ⌘T → the new shell reports the inherited dir; canonicalized against macOS's
  /var→/private/var split); GATE GREEN [diff] 15/15.

- **Readline keys at the cooked prompt — ⌃A/⌃E/⌃K/⌃U/⌃W/⌃Y muscle memory** (TICKET-278, forge
  #278; sprint M17, terminal polish #1). The emacs bindings every terminal hand expects, over
  Marley's own prompt buffer: line home/end, kill-to-end/-start, kill-word-back (the #257 word
  class — bash's ⌥⌫, one word definition everywhere), and yank from a ONE-slot kill (the ring +
  consecutive-kill append are recorded polish; an EMPTY kill never clobbers the slot — the
  readline rule). The six chords are claimed BEFORE the #40 route (which Raw-routes EVERY ctrl —
  the ticket's premise corrected at plan), only when a terminal tab is active and no program owns
  the input; ⌃C/⌃D/⌃Z and all other ⌃ stream byte-identically, and a running command still gets
  raw 0x01 for ⌃A. Inspect's HIGH: the arm was reachable on editor/cockpit tabs and edited the
  HIDDEN terminal's prompt (the #283 class, upgraded from stray bytes to visible corruption) —
  now gated on `grid().is_some()` with a headless editor-tab negative pinning it. Kill-list units
  (multibyte spans, D2 empty-kill, the injected-empty-slot pin, moved-ness returns) + a full
  headless chord flow; GATE GREEN [diff] 15/15. The live ⌃U capture joins the unlock batch.

- **Incremental + off-thread tree-sitter — big files stop taxing the keystroke** (TICKET-274,
  forge #274; sprint M17, the B3.2 perf gate the #268 close required before self-hosting). The
  measured 25.9ms whole-file parse per keystroke at 8k lines now runs on a WORKER thread and
  re-parses only the damage: `marley_syntax::HighlightSession` keeps the previous `Tree` AND the
  previous text snapshot (`BufferDelta` carries no removed text or points — with the snapshot in
  hand every `InputEdit` point is pure string math, `point_at`/`syntax_edit`, multibyte-exact and
  probe-verified against tree-sitter's own node positions). Single-delta steps go incremental;
  multi-delta gaps (autorepeat, undo bursts) take a full parse — both off-thread behind a
  generation-dropped channel the pump drains; the worker COALESCES bursts (drain-to-latest) and a
  validity rule (session at exactly (nonce, parent_version), else full) makes skips self-healing.
  Files ≤1000 lines keep the zero-latency sync path verbatim. Inspect's probe corpus proved
  incremental ≡ full on 15/15 acid cases (block-comment cascades, multibyte, 10-step chains) and
  measured the ticket's premise BACKWARDS: the re-parse is 9× faster (1.0 vs 9.2ms) but the
  O(file) QUERY WALK (5.7ms/27k captures) floors end-to-end at 0.46 — REQ-003 re-pinned to what's
  true (parse-only <1/3, end-to-end <3/4) and the walk-windowing recorded as the measured
  follow-up. Also hardened at inspect: `point_at` non-boundary panic (floor guard), the splice-lie
  hole (prefix/suffix memcmp → full-parse fallback — "total, never wrong"), the dead-worker
  pending wedge (Disconnected teardown), cross-nonce stale renders (foreign cache drops). The
  async loop is proven headless end-to-end (real worker thread + test-clock pump). GATE GREEN
  [diff] 15/15; large-file color captures join the unlock re-verify batch.

- **External file-change detection — the agent workflow's missing half** (TICKET-275, forge
  #275; sprint M17). An agent rewriting files in the terminal next to the editor finally
  registers: every `OpenFile` carries a (mtime, len) disk snapshot (statted BEFORE reading —
  the safe race direction — at open, boot-restore, and save), and a pure decision table
  (`extchange::external_action`) runs at the poll choke set — the window-focus-REGAINED render
  edge (launcher-guarded: the empty project set panics on `active_project()`, caught live when
  the virgin-boot headless test wedged the suite), every reveal-activation (file-tab click, tab
  switch ⌘]/⌘[/⌘1-9, workspace cycle, rail jump, close-reveal, open-switch), and the head of
  ⌘S. No watcher thread in v1 (APFS ns-mtime probe: the same-second blind spot is
  ~nonexistent). Clean buffer + changed disk → SILENT reload (caret clamped, nonce re-minted so
  the syntax memo re-parses and the scroll sync re-clamps, undo history restarts as an external
  epoch, "Reloaded from disk" flash; a content-identical rewrite skips the epoch entirely).
  Dirty + changed → a floating non-modal banner (mouse-blocked per the #185 rule): Keep mine
  re-snapshots (the next change re-flags), Reload discards (the flash says so). ⌘S under a
  conflict warns + ARMS on the first press and only overwrites on the second; a post-write LEN
  cross-check flags a write that raced ours. Deleted → banner, buffer kept, Dismiss untracks,
  ⌘S recreates. Inspect's headline catch (both critics): the boot-RESTORE path never seeded
  snapshots — the feature would have been silently OFF for every restored working set
  (`PR-claude-restore-is-a-second-constructor-001`). Full decision-table kill list; 4 headless
  flows incl. the ACTIVATION-DRIVEN focus edge (gpui's TestWindow is inactive by default —
  `activate_window` is the only live-edge exercise) and the real launcher-guard pin; GATE
  GREEN [diff] 15/15. Banner pixels ENV-BLOCKED (machine locked) — the unlock re-verify batch.
  Follow-up: acknowledgment-binding (the arm/keep-mine license vs a NEWER external write).

- **Auto-indent on Enter + Tab/⇧Tab block indent — code typing feels right** (TICKET-276,
  forge #276; sprint M17). Enter on the editor now clones the current line's leading whitespace
  (clipped at the caret when it sits inside the indent — D1), as ONE `Buffer::edit` even over a
  selection (the #255 one-step type-over rule; inspect caught the delete-then-insert two-undo
  version and the fix computes the clone BEFORE the replace, provably unchanged by it). Tab with a
  selection / ⇧Tab run per-line indent/dedent ops from the new pure `marley_editor::indent` module
  — `line_span` (with the universal col-0 carve: a shift+Down selection ending at column 0 doesn't
  drag that row in), `indent_edits` (empty lines skipped — no whitespace-only residue),
  `dedent_edits` (one stop of spaces or one `\t`, per line), applied back-to-front with BOTH
  selection endpoints rebased through `rebase_through` (an insert at the endpoint pushes it right —
  the selection keeps covering the same TEXT; an endpoint inside stripped whitespace clamps to the
  line start). A bare-caret Tab inserts display-column-aware spaces to the next tab stop
  (`spaces_to_next_tab_stop` over the #250 `line_layout`, so tabs in the prefix count); bare ⇧Tab
  dedents the caret's line. Both are ROUTER-handled rows BEFORE the Char|Other claim — Tab's
  key_char "\t" never reaches the IME as a literal tab again (the pre-#276 bug), ⇧Tab stops being a
  silent no-op, and the terminal's cooked Tab completion + raw Tab/BackTab (CSI Z) are byte-identical
  (the arm is editor-gated; goldens re-ran green). Undo follows the #272 v1 convention: Enter = one
  step, multi-line ops unwind per-line (grouped undo recorded as a follow-up). 48/48 mutants caught
  (MSI 100), 7-unit kill list + 2 headless flows through the real key ladder; driven capture
  ENV-BLOCKED (machine locked) — batched for re-verify on unlock with #272's.

- **Editor find & replace (⌘F) + ⌘A select-all — the missing basics** (TICKET-272, forge #272;
  sprint M17). The editor finally has find: an Editor-context ⌘F bar (the SAME chord as the
  terminal's scrollback bar — the #265 disjoint contexts doing exactly their job) with live
  "i of N", wrapping Enter/⇧Enter that selects each match and centers it via the #273 scroll
  mechanism, a replace field (Tab toggles focus; Enter there = Replace One with a RESUME cursor so
  a needle-containing replacement advances instead of looping; ⌘Enter = Replace All, back-to-front
  via the new pure `find_all`/`replace_all` — each match its own undo step), and Esc back to the
  buffer. Matches tint through the GENERALIZED #266 highlight channel (`styled_slices_with_marks`
  — N mark ranges with Current-beats-Match tiers alongside syntax + selection; the same channel the
  future IME underline rides). Matching is ASCII-case-insensitive (the terminal find's fold).
  UNOWNED ⌘/⌃ chords fall THROUGH the open bar — ⌘Z undoes a bad replace, ⌘1 switches tabs, ⌘F
  reseeds — and a tab switch away from the editor closes the bar (render-choke; keys never leak
  into a terminal). The inspect round was load-bearing: a stale-match-set edit path that could
  PANIC ropey between frames now refreshes in the handler; the mutants::skip detach trap struck a
  FIFTH time (caught by `--list`, re-seated); space is typable in both fields. ⌘A selects the whole
  buffer on the editor only. Driven captures env-blocked (machine locked) — carried by the headless
  flows driving the real key ladder + the pixel-proven #266/#47 render idioms; re-verify live on
  unlock.
- **Semantic syntax highlighting — tree-sitter parses the editor's Rust** (TICKET-268, forge #268;
  sprint M16, editor-frontier B3 slice 1). A new `marley_syntax` crate (entirely safe Rust — the C
  FFI stays inside the MIT `tree-sitter` 0.26 + `tree-sitter-rust` 0.24 dependencies) parses the
  whole document and maps the grammar's own `highlights.scm` captures onto Marley's existing 5-kind
  palette: `kind_of_capture` (the 21-name capture set PINNED by a truth-table test asserted against
  the shipped query — a grammar bump fails loudly), `sweep_disjoint` (duplicate doc-comment captures
  and nested spans clip away; its own seam so an admitted empty span stays test-observable), and
  `clip_to_lines` (per-span distribution via `partition_point` — multi-line block comments/strings
  contribute one clipped span per crossed line). The editor render memoizes the parse per
  `(buffer-nonce, version)` — the NONCE is a process-monotonic buffer identity minted per open file,
  because `BufferVersion` restarts at 0 and a bare `(path, version)` key served STALE spans after
  reopening a file rewritten on disk (inspect). Rows feed the #266 `styled_slices`/`with_highlights`
  substrate through a pure raw→display byte remap (tabs widen, multibyte exact). **The semantic wins
  land**: `"a // not a comment"` paints as ONE string; `pub(crate)` inside a doc comment stays
  comment-muted; `formatter` is never a keyword — all driven-capture proven. Non-Rust languages and
  the #246 read-only pane keep the hand lexer (byte-identical, capture-verified). Measured: 1.1ms
  per refresh at 319 lines, 2.4ms at 721, **25.9ms at 8k lines (app.rs)** — fine at dogfood sizes,
  and the recorded follow-up (off-thread + incremental re-parse via tree-sitter `InputEdit` fed by
  the #269 delta log) is REQUIRED before self-hosting on large files. Reference §20: grammar-driven
  highlighting behaviorally per `docs/zed_architecture/subsystems/04-language-syntax-treesitter.md`;
  tree-sitter + grammar + query consumed as shipped MIT artifacts; the wrapper, mapping, sweep/clip,
  and cache orchestration are Marley-original.
- **Real OS text input for the editor — IME, dead keys, composition (`EntityInputHandler`)**
  (TICKET-267, forge #267; sprint M16, editor-frontier B2 input). The editor's plain-character
  insertion moves off the hand-rolled `Key::Char` arm onto the platform's NSTextInputClient path:
  `RootView` implements gpui's `EntityInputHandler` (8 methods routing to a NEW pure
  `marley_editor::ime` module — `replace_text`/`replace_and_mark`/`unmark`/`text_for_range`/
  `selected_utf16`/`marked_utf16` over a new UTF-16↔`CharOffset` buffer seam, cov/MSI 100), and the
  #266 editor canvas registers an `ElementInputHandler` during paint (paint-scoped + focus-gated ⇒
  terminal frames never register; the prompt path is untouched by construction). **Dead keys and IME
  now work for the first time**: ⌥E composes ´ as a marked span and the commit replaces it in place
  (driven-capture proven — é on screen); CJK preedit grows/commits through the same two ops. The
  editor key arm becomes a routing table: motions/backspace/Enter stay router-handled and now
  `stop_propagation` (Enter would double-insert via its simulated key_char otherwise); printables +
  dead keys are claimed-but-propagated to the handler — ONE insert mechanism. Every full-capture
  overlay arm in the key ladder now stops propagation too (inspect HIGH: an overlay-consuming Enter's
  "\n" otherwise leaks into the buffer through the headless dispatch fallback — the state-flag gate
  can't catch it because the overlay closes itself first), `text_input_blocked()` gates the handler
  during overlay typing (the completion popup deliberately excluded — its printables fall through by
  design), a pending composition is cleared by every non-IME mutation path (click, chord verbs,
  cut/paste, undo/redo, file switches), and **Esc on the editor now deliberately deselects** (was an
  accident of the old catch-all arm). The per-frame `EditorFrameGeom` (x0→x0/y0/rows/cell) answers
  the IME geometry queries as pure cell arithmetic. Composition selection reuses the existing
  selection channel; the marked-text underline is deferred to #268's highlight generalization; the
  terminal prompt keeps its raw path (a future ticket). Reference §20: the editor-as-
  NSTextInputClient model per `docs/zed_architecture/crates/gpui.md`; mechanism = gpui's Apache-2.0
  public API; the routing, UTF-16 seam, IME ops, and marked-state model are Marley-original.
- **The editor renders on the gpui-native substrate: `uniform_list` + `StyledText::with_highlights`**
  (TICKET-266, forge #266; sprint M16, editor-frontier B2 render). The editable editor tab's hand-rolled
  render (a fixed `VIEWER_ROWS = 40` window of per-line flex divs, one colored div per syntax span) is
  replaced by true viewport virtualization: `uniform_list` sized from `buffer.len_lines()` with a
  `UniformListScrollHandle` (`.track_scroll`) — **files scroll to EOF; the 40-row cap is retired** — and
  each visible row is ONE `StyledText::with_highlights` carrying syntax tints plus the #255 selection as a
  28%-accent `background_color` range. Two new pure seams feed it (cov/MSI 100):
  `code_syntax::highlight_ranges` (folds the lexer's sequential chunks into ascending-disjoint BYTE ranges,
  Plain dropped) and `code_view::styled_slices` + `cols_to_bytes` (the boundary-cut splitter honoring
  `with_highlights`' disjoint-sorted char-boundary contract — probe-verified against gpui's `compute_runs`
  over 133k cases — and the multibyte-safe display-column→byte bridge). The caret bar + click/drag mapping
  deliberately KEEP the pure monospace cell math (`ccol * cell.w`, `offset_for_click`) — exact on a mono
  grid; `TextLayout::position_for_index`/`index_for_position` wait for a proportional font. The editor-tab
  wheel handler is deleted (the list scrolls natively); listeners re-plumb through the entity handle inside
  the list closure; the #246 read-only pane path is untouched (driven-capture verified). Dead
  `input::split_at_caret` removed (zero production callers — the terminal prompt uses `split_caret_char`).
  **Known regression (deliberate, this slice):** per-FILE scroll memory — the old per-file `cv.scroll`
  offset is superseded by one shared scroll handle, so switching file tabs carries the offset instead of
  restoring each file's own; the #270 caret-follow ticket restores per-file offsets via `scroll_to_item`.
  Reference §20: the virtualized-line-list + ranged-style editor model behaviorally per
  `docs/zed_architecture/crates/gpui.md`; mechanism = gpui's own Apache-2.0 public API; row closure/span
  adapters/mouse plumbing Marley-original.
- **Anchors — editor positions that survive edits (the B5/B6 prerequisite)** (TICKET-269, forge #269;
  sprint M16, editor-frontier B4). `marley_editor` gains `Anchor{version, offset, bias}` + `rebase_offset`
  (the public patch old→new arithmetic: edits before shift, after leave, covering collapse by `Bias`
  Left/Right, a pure insert AT the anchor honors the lean) and the `Buffer` grows a delta log fed by the one
  shared apply path — so undo/redo move anchors like any edit — plus `edits_since(version)` (the incremental
  feed B3 tree-sitter will also consume), `anchor_at`, and `resolve_anchor` (fold + clamp). Deliberately
  CRDT-free: one linear history per buffer; the fragment/dense-order/logical-clock machinery is deferred to
  a real concurrent multi-writer milestone. The `offset == span-end` boundary counts as ON the span — the
  unique convention under which one `edit(s..e, repl)` rebases identically to the buffer's own
  delete-then-insert decomposition (machine-checked exhaustively at inspect: 2,430 composition triples + a
  4,716-assertion char-identity oracle). Full mutation ground truth: anchor.rs 15/15, buffer.rs all viable
  mutants killed (the mid-history fencepost test is the lone `>`→`>=` killer). Side effect:
  `PaneContent::Terminal` is now boxed (the growing pane state tipped clippy's large-enum-variant; the box
  also cheapens every `PaneState` move). Reference §20: N/A — Marley-original; the anchor concept is public
  CS, explicitly not the GPL text-crate rendition.
- **Context-scoped keybindings (KeyContext) — one chord, per-surface meaning — and the editor gets ⌘D
  select-next-match** (TICKET-265, forge #265; sprint M16, the editor-frontier B1 opener). The keymap's
  bindings may now carry a `KeyContext` (`Terminal` | `Editor`; absent = global): the active tab publishes its
  context stack (`Tab::key_context()` — terminal → `[Terminal]`, editor → `[Editor]`, cockpit → `[]`) and
  `action_for(chord, stack)` resolves deepest-context-first with global fallback; chord uniqueness is now per
  `(chord, context)` (`chords_unique_scoped` — the same chord under disjoint contexts is the point). This
  retires the flat-table collision class: ⌘D stays "new-terminal" on terminal/cockpit surfaces but is
  **"select-next-match" on the editor** — a new pure `marley_editor::find` module (`word_range_at` /
  `next_occurrence` / `select_next_match`): first press selects the word under the caret (same word class as
  ⌥-arrow motion), each repeat advances the selection to the next occurrence of the selected text, wrapping
  (sole occurrence self-finds — a deterministic no-op). ⌘F likewise became a `Terminal`-context row
  ("open-find") — an editor or cockpit tab no longer opens the scrollback find bar (was a hardcoded
  context-blind arm). Resolution semantics follow the gpui (Apache-2.0) KeyContext model as Marley-original
  pure code (cov/MSI 100; no per-surface FocusHandle rewire — the app's single key ladder passes the stack).
  Live-proven via driven captures: terminal ⌘D → a new "Marley 5" tab in the rail; editor ⌘D → `http`
  word-highlight that advances on repeat; editor ⌘F → no find bar. Multi-cursor ⌘D (B5), editor-find, and
  per-context palette chips are deferred. Reference §20: the Zed editor's context-scoped keymap BEHAVIOR via
  `docs/zed_architecture/subsystems/09-vim-keymap-contexts.md` (§A/§D); mechanism = gpui Apache-2.0 model,
  implementation Marley-original.

- **Split-file panes persist across restart (`c=<path>` grid codec)** (TICKET-258, forge #258; sprint M15). A
  split-right file pane (the #246 read-only `CodeView` pane) now survives a quit/relaunch: it serializes into the
  grid codec as `c=<path>` (mirroring the #205 terminal `t=<cwd>`, framing-guarded — a path holding a blob
  delimiter drops to a bare `c`), and on restart the restore arm re-reads the file into a read-only pane
  (unreadable/oversize/binary or a pre-#258 bare `c` → dropped, no crash), on both the whole-shell and legacy
  restore paths. The pane stays read-only; the editable split pane is a fast-follow (#259). Reference §20: N/A —
  Marley/IDE-specific pane container; the in-pane editing feel is inherited from the #249-257 editor stack.
- **Editor caret motion parity — Home/End, ⌥←→ (word), ⌘←→ (line), ⌘↑↓ (document), Up/Down (vertical)**
  (TICKET-257, forge #257; sprint M15). The editor now honors the standard mac caret-motion keymap: Home/End to
  line start/end, ⌥←/⌥→ by word, ⌘←/⌘→ to line start/end, ⌘↑/⌘↓ to document start/end, and Up/Down to the same
  visual column on the adjacent row (clamped to a shorter line; the last/first row stays). The SHIFT-variant of
  any motion EXTENDS the #255 selection; unshifted collapses/moves. Vertical motion has no goal-column memory in
  v1 (uses the current column). Page-up/down, smart-home, and find-motion are deferred. Reference §20: the
  universal mac editor caret-motion convention — Marley's own impl over the tested `marley_editor::movement`.
- **Copy / cut / paste in the editor (⌘C / ⌘X / ⌘V)** (TICKET-256, forge #256; sprint M15) — ⌘C copies the
  editor selection to the system clipboard, ⌘X cuts (copy + delete), and ⌘V pastes — replacing an active
  selection or inserting at the caret, with the caret placed after the pasted text. The ops run on the editor's
  Buffer (undo-recorded, so one ⌘Z reverts a cut/paste) over the #255 selection, and reuse the same gpui
  clipboard the terminal already uses; a terminal tab keeps its own copy/paste unchanged. ⌘C with no selection is
  a no-op (copy-the-line deferred). Multi-cursor clipboard, rich clipboard, and ctrl-editing (#257) are deferred.
  Reference §20: the universal editor clipboard convention — Marley's own edits over the existing plumbing.

- **Text selection in the editor — shift+arrows + mouse-drag** (TICKET-255, forge #255; sprint M15) — the editor
  caret is now a live selection: Shift+Left/Right extends a range from a fixed anchor (an unshifted arrow
  collapses it to the selection edge), a mouse-drag selects (mousedown sets the anchor, the drag extends the
  head), the selected span is highlighted (an accent-alpha tint over the selected columns, aligned to the #250
  grid across tabs and multibyte text), and typing or backspace over a non-empty selection replaces it (a
  backwards right-to-left selection still edits the normalized range) then collapses to a caret. Single selection
  (no multi-cursor yet); shift+word/Home/End (#257), double/triple-click word/line, and clipboard copy/paste
  (#256) are deferred. Reference §20: the universal monospace-editor selection convention — Marley's own model
  over `marley_editor::Selection` + the #250 column map.

- **Click in the editor to place the caret** (TICKET-254, forge #254; sprint M15) — a left-click anywhere on an
  editor line moves the caret to the character nearest the click (and typing then inserts there), by inverting
  #250's exact char↔column map so the caret lands on the clicked char even across tabs and multibyte text. The
  clicked row is exact (per-line hit); the column rounds to the nearest character boundary (a click on a char's
  right half lands after it), clamped past line-end to the line end and below the last line to the last line.
  Only editor tabs respond (a terminal tab / a read-only split pane is unaffected). Drag-to-select and
  double/triple-click word/line selection are deferred (#255+). Reference §20: the universal monospace-editor
  click-to-place convention — Marley's own pure inverse over the #250 column map.

- **Undo/redo for the editor with coalesced typing (⌘Z / ⌘⇧Z)** (TICKET-253, forge #253; sprint M15) — the
  editor Buffer now keeps an in-memory undo history: each edit records an invertible step, ⌘Z reverts the last
  step and ⌘⇧Z re-applies it, and consecutive single-char inserts COALESCE into one step (one ⌘Z removes a typed
  word/run, not one character). A backspace/delete, a multi-char insert/paste, an origin change (Human vs Agent),
  or a caret jump starts a new step; a fresh edit after an undo clears the redo stack. The inverse is applied
  without re-recording (the history drains, never loops), and all offset math is char-indexed (multibyte-safe).
  ⌘Z/⌘⇧Z with a terminal tab active are a no-op. Undo does not re-clean the dirty ● (an undo bumps the buffer
  version — the #252 version-based dirty model). Transient/in-memory (no persistent history, no undo-across-close).
  Reference §20: the universal coalesced-typing undo convention — Marley's own pure implementation.

- **Save the editor with ⌘S + a dirty ● on unsaved tabs** (TICKET-252, forge #252; sprint M15) — ⌘S writes the
  active editor file's buffer to its path on disk (only marking it clean on a SUCCESSFUL write — a save error
  surfaces a status flash and the file stays dirty), and the #237 file-tab strip shows a ● on every file with
  unsaved edits (`buffer.version() != saved_version`, per-file), cleared on save. ⌘S with a terminal tab active
  is a no-op. Save-as / save-all / external-change detection are deferred. Reference §20: Warp's edited/unsaved
  indicators + the standard editor ●/⌘S convention.

- **You can now TYPE into the editor — keys edit the file buffer + move the caret** (TICKET-251, forge #251;
  sprint M15) — THE marquee interaction of the editable editor: the file editor is now genuinely editable. When
  an editor tab is active, plain keys are captured and routed to the active file's `marley_editor::Buffer` +
  caret (via a new `input::apply_editor_key`, reusing the prompt's tested `apply_key` with ONE divergence —
  Enter inserts `\n` instead of submitting) instead of leaking to a hidden terminal; the #250 render redraws the
  edit + the caret live. Char insert, Backspace, and char-wise Left/Right work; ⌘/ctrl chords (⌘W/⌘P/⌘D) still
  reach the keymap, and the terminal prompt is unchanged when a terminal tab is active. Up/Down/Home/End/word-
  wise are deferred to #257; ctrl-editing + copy/paste to #256. Reference §20: Warp's focus routing (the focused
  surface owns the keystroke).

- **Faithful editor render — the editor draws from the Buffer with a visible caret + exact offset↔column
  mapping** (TICKET-250, forge #250; sprint M15) — THE make-or-break of the editable editor. The editor now
  renders each visible line from the active file's `marley_editor::Buffer` (not the lossy, tab-expanded,
  200-col-truncated `CodeViewState.lines`), **un-truncated**, so on-screen columns line up EXACTLY with
  character offsets — and draws a caret at the active file's caret position (a bar at `column × the measured
  monospace cell`). A pure `code_view::line_layout` builds the tab-stop display string AND the char→column map
  in one pass (the SINGLE source of truth — `expand_tabs` now delegates to it); `Buffer::line_text` /
  `line_col` give per-line access + the caret's offset → `(row, char-in-line)` (a CHAR index, not `Point`'s
  byte column). The #246 read-only split pane keeps its lossy-lines render (unchanged); the wheel-scroll now
  clamps against the buffer's line count. The caret is static at offset 0 for now — #251 moves it + types.
  Reference §20: Warp's monospace cell grid + block caret (observed capture in `docs/warp_architecture/observed/`).

- **Editor doc model — each open editor file is now backed by an editable `marley_editor::Buffer` + caret +
  `saved_version`** (TICKET-249, forge #249; sprint M15 "The Editable Editor") — the FOUNDATION of the editable
  editor. Each open file becomes an `OpenFile { view, buffer, caret, saved_version }` inside the workspace's
  editor surface, with the buffer seeded from the file's RAW text (not the lossy tab-expanded/truncated render
  lines). `active_file()` still returns the read-only `CodeViewState`, so rendering, the #243 path-persistence,
  and the file-tab strip are all unchanged; new accessors (`active_buffer_mut` / `active_caret` /
  `active_saved_version` / `active_is_dirty` / …) expose the edit state for the tickets to come. A file is dirty
  ⇔ `buffer.version() != saved_version`; re-opening an already-open file now switches to it and PRESERVES its
  buffer (no disk-refresh that would clobber unsaved edits). Invisible on screen until #250 renders from the
  buffer — this ticket is the model only. Reference §20: Warp's command-input buffer+caret editing model
  (behavior). (the 2nd ticket of the M15 train)

- **Split a terminal right into a read-only file view** (TICKET-246, forge #246; sprint M14) — chad's "terminal
  + split-right-to-a-file". The Command Palette's "Split Right → File" opens the ⌘P finder; the file you pick
  opens read-only in a pane BESIDE the terminal (not a new terminal, no PTY), titled by its filename. Resizes and
  closes like any pane. Read-only for now (editing rides on the editable editor); the file pane is not yet
  restored on restart. (a #237 deferred item)

- **Boot to the launcher when there's no session to restore + a "New empty workspace" action** (TICKET-247,
  forge #247; sprint M14) — a genuinely fresh/empty start (no saved session, or you closed the last workspace)
  used to force-seed a default workspace; it now shows the #234 launcher instead. The launcher gains a "New empty
  workspace" button (a workspace at your home dir, no folder picker). A saved session still restores normally.
  (a #234 follow-on)

- **Click the top-bar workspace indicator to switch workspaces** (TICKET-244, forge #244; sprint M14) — the
  focused-workspace indicator (#235, "name · branch") was display-only; it's now a click-to-open popover that
  lists the open workspaces, and clicking a row switches the focused one (reusing the #233 rail-click switch).
  Backdrop-to-dismiss, the active row accent-highlighted. Switches the active project within today's Workspace
  container (not the multi-workspace re-architecture). (a #235 follow-on)

- **Files open as editor tabs in one surface, not a rail row each** (TICKET-237, forge #237; sprint M13) —
  opening files (⌘P, the Files tree, a link / search / diff result) used to add a read-only tab to the left
  rail per file, so N open files cluttered the rail with N rows. Now each workspace has ONE editor surface
  (one rail row); every file-open lands there as a horizontal FILE TAB (name · × to close, click to switch) —
  the VSCode model. Re-opening a file switches to its existing tab (deduped by path); closing the last file
  drops the editor tab. Read-only for now (reuses the existing code-view doc); editing, terminal-split-to-a-
  file, and persisting more than the active file across restart are deferred follow-ups. (chad feedback #9,
  anti-clutter) — the FINAL ticket of the M13 workspace-cockpit train (#228–#237).

- **A collapsible workspace rail + a focused-workspace highlight** (TICKET-236, forge #236; sprint M13) —
  each workspace (project) row in the left rail now has a ▸/▾ chevron that collapses/expands its tabs (and
  nested panes), and the focused workspace gets a clear highlight background so you can see at a glance
  which workspace the actions act on. (chad feedback #8 + the rail-highlight ask)

- **A focused-workspace indicator + a workspace-scoped top bar** (TICKET-235, forge #235; sprint M13) — the
  top bar now shows which workspace you're focused on (its name · branch) right next to the workspace
  actions, and the cockpit tabs (Details / Agents / Forge) moved from the far right into that same left
  cluster — so all the workspace-scoped controls sit together and it's clear they act on the focused
  workspace. Switching the focused workspace (⌘⇧]) updates the indicator. (chad feedback #6+#7)

- **A launcher / landing page when no workspace is open** (TICKET-234, forge #234; sprint M13; supersedes
  #202) — closing your last workspace no longer dead-ends. The M10 never-empties guard was relaxed so a
  zero-workspace state is reachable, and the app then shows a PhpStorm-style landing page — a big "Open a
  workspace" with your **recent workspaces** (click one to reopen it) and an "Open Folder…" button (the
  native picker). Opening a workspace records its root in a persisted, dedup'd, capped recents list. The
  shell only renders with ≥1 workspace, so the launcher state is panic-safe (the render, the 16ms PTY
  pump, and the persist path all branch/guard at zero). (chad direction: closing everything → a landing page)

- **Cycle the focused workspace with ⌘⇧] / ⌘⇧[** (TICKET-233, forge #233; sprint M13) — the first slice of
  the workspace-centric re-arch. Marley already holds multiple open projects (a "workspace" = a repo); now
  ⌘⇧] / ⌘⇧[ cycles which one is focused (a level up from ⌘] / ⌘[ for tabs), and the sidebar, Files, ⌘P, git
  branch, and titlebar all follow the focused workspace. Pins the workspace terminology (Session ⊃ Workspace
  ⊃ Tab) as the foundation for the launcher, workspace-scoped top bar, and rail highlight still to come; the
  codebase-wide type rename is deferred to its own pass (it collides with a second `Project` type). (chad
  direction: workspace-centric)

- **A scrollbar + jump-to-bottom for pane scrollback** (TICKET-198, forge #198; sprint M12.2) — a scrolled-up
  terminal pane now shows a scrollbar thumb on its right edge (its position + height reflect where you are +
  how much of the output fits), and a round ▾ jump-to-bottom button appears at the bottom-right while you're
  scrolled up off the latest output — click it (or scroll down) to snap back to the tail. The thumb and
  button appear only when the content overflows the viewport. Pure `scrollbar_thumb` / `at_bottom` geometry
  (cov/MSI 100) over the existing viewport model; the jump reuses the tested scroll-to-follow.

- **A live theme picker in the command palette** (TICKET-199, forge #199; sprint M12.2) — the command palette
  now lists a "Theme: <name>" entry per built-in theme (Marley Dark / Marley Light); pick one and the whole
  cockpit switches colors instantly (and the choice persists across relaunches). Previously you had to
  hand-edit the TOML settings. Reuses the existing theme registry + the apply/persist path; a new pure
  `theme_pick_index` (cov/MSI 100) resolves the palette command id to its theme.

- **Inline history autosuggestion (ghost text) at the prompt** (TICKET-200, forge #200; sprint M12.2) — as you
  type at the prompt, a dimmed "ghost" completion of the most-recent matching command from history now appears
  after the caret (fish/Warp style); press → to accept it. The ghost shows only at end-of-line, on the focused
  pane, and when the Tab-completion popup isn't open — it's display-only until you accept. Pure
  `history::suggest(prefix, history) -> Option<suffix>` (cov/MSI 100).
- **CWD-aware tab titles** (TICKET-201, forge #201; sprint M12.2) — a terminal tab with no custom name and no
  running foreground command now shows its working-directory basename (e.g. `Marley` for
  `~/Projects/ignibyte/Marley`) instead of a static `terminal N`; a running command still shows its program
  token (`vim`, `sleep`) and the tab reverts to the cwd on the command's exit; a double-click rename (custom
  title) still wins over both (#177). Completes the `custom → running command → cwd basename → generic` chain.
  Pure `titlebar::display_title` gains a cwd tier (reusing `prompt::pwd_label`), cov/MSI 100; the shim reads the
  live `current_prompt().pwd` and gates the command on `blocks().current()`.
- **Background-command completion badge** (TICKET-203, forge #203; sprint M12.2) — when a command finishes in a
  NON-focused pane after running past a ~10s threshold, its tab's rail row now shows a completion ● (green
  success / red failure) so you can start a long job, switch away, and get told when it's done; the badge
  persists until you view that tab. Pure `notify::should_notify(pane_focused, status, elapsed_secs,
  threshold_secs) -> Notify` (cov/MSI 100); the pump counts per-pane running ticks for the elapsed and keys a
  per-tab badge. (An OS-level notification is a deferred follow-up.)
- **Command-palette workflows** (TICKET-204, forge #204; sprint M12.2) — save a command under a name (with
  optional `{{param}}` placeholders) and re-run it from the command palette (a Warp Workflows analog).
  "Save Command as Workflow…" names the focused terminal's last command; each saved workflow shows as
  "Workflow: <name>" and inserts its command at the prompt when invoked (a no-param workflow substituted; a
  `{{param}}` template inserted to fill inline — not auto-run). Pure `workflows::substitute(template, args)
  -> Result<String>` + `params_of` (cov/MSI 100); persisted as a `[[workflows]]` setting. (A dedicated
  interactive param-prompt modal + workflow edit/delete are deferred.)
- **Session cwd persistence** (TICKET-205, forge #205; sprint M12.2) — a relaunch now restores each terminal
  pane in its previous working directory, not just the layout. #163 already restored the tab + split-tree
  structure; this adds the cwd: the pure grid-blob codec encodes a terminal leaf as `t=<cwd>` (framing-safe,
  back-compatible with old `t` blobs), and on boot each terminal respawns in its saved cwd when that directory
  still exists (a deleted/moved cwd falls back to the project root). Pure `serialize_grid`/`restore_grid`
  round-trip, cov/MSI 100.
- **OSC 8 explicit hyperlinks** (TICKET-214, forge #214; sprint M12.2) — terminal output that uses the OSC 8
  escape (`ESC]8;;URI ST … text … ESC]8;; ST`, where the visible text differs from the target URI, e.g.
  `click` → `https://example.com`) now renders as a correct clickable link. #196 shipped text-scan link
  detection; this carries the alacritty grid cell's hyperlink URI through the `terminal_blocks` styled model
  (`StyledRun.hyperlink`; a run breaks on a hyperlink change, keeping `output_text` byte-identical) and makes
  the render prefer an explicit hyperlink over the text-scan heuristic. Pure codec + link-composer at cov/MSI 100.

### Added

- **cwd↔editor link — "Open Terminal Here" + "cd Terminal to Editor Dir" from the palette**
  (TICKET-294, forge #294; sprint M18 — closes the M18 Terminal↔Editor Fusion train). Two cockpit
  palette commands tie the two surfaces' working directories together. **"Open Terminal Here"** spawns
  a new terminal tab rooted at the ACTIVE editor file's directory (pure `dir_of` — the file's parent —
  fed through the #281 `valid_dir_or`/`spawn_session_in`); **"cd Terminal to Editor Dir"** writes a
  shell-safe `cd <dir>` to the focused terminal (pure `cd_command` — POSIX single-quote wrapped, an
  embedded `'`→`'\''`, so a space/`;`/`$()`/backtick in the path is neutralized — verified injection-proof
  in a real shell). `new_terminal_pane`'s spawn+add-tab tail is extracted to a shared
  `spawn_terminal_tab_in(cwd)`. Proven live: opening a file then invoking "Open Terminal Here" spawned a
  terminal whose `pwd` was the file's directory. `dir_of` + `cd_command` at cov/MSI 100. A per-tab/tree
  affordance + a keybinding are follow-ups. GATE GREEN [diff].

- **File-ref context menu in terminal output — right-click a `path:line` → open / split / reveal / copy**
  (TICKET-293, forge #293; sprint M18). Broadens the #212 single left-click→open with a RIGHT-click
  context menu on a linkified `path:line` in block output: **Open in Editor** (#212), **Open in Split
  Right** (#246), **Reveal in File Tree**, **Copy Path**. A pure `MenuKind::FileRef` + `FILE_REF_MENU_ITEMS`
  (a payload-less marker — `MenuKind` is `Copy` and a ref carries a `PathBuf`, so the shim holds the target
  in `file_ref_menu_target`) and a new pure `FileTree::reveal(path)` (expand every ancestor directory, then
  return the file's visible-row ordinal; `None` when it's not in the tree). The link span gains an
  `on_mouse_down(Right)` that opens the menu for a `File` link and `stop_propagation`s so the #175 block
  menu doesn't also fire. Proven live: right-clicking a `path:line` showed the 4-item FileRef menu (not the
  block menu → propagation correct), and "Open in Split Right" opened the file in a right split pane.
  `items_for(FileRef)` + `FileTree::reveal`/`expand_parts` at cov/MSI 100. The persistent reveal highlight,
  disabled-item states, a modifier-click variant, and a URL-link menu are follow-ups. GATE GREEN [diff].

- **Close the run→fix loop — "Re-run Last Failed Command" from the palette, markers clear on green**
  (TICKET-292, forge #292; sprint M18). The payoff of the fusion wedge: after editing+saving a file a
  failed command referenced, re-run it without hunting for the block, and watch the stale gutter markers
  clear on green. A pure `block_status::last_failure_block_index(kinds)` finds the MOST RECENT `Failure`
  block (`rposition`, scanning back past later Success/Running commands — distinct from the existing
  `rerun-last`/⌘⇧R, which re-runs the most recent FINISHED command via the focused terminal). A new
  cockpit palette command "Re-run Last Failed Command" (⌘⇧P) dispatches `rerun-last-failed`, which scans
  the workspace terminals — the lowest-PaneId IDLE one with a failure (deterministic; `states()` is a
  HashMap) — and re-runs it via the shipped #175 `rerun_block`; it works with the EDITOR focused (no
  focused terminal), which the existing `rerun-last` cannot. Clear-on-green is inherent: `rerun_block`
  writes a NEW last block, and #289's Failure-only gutter drops a green last block. Proven live: a failed
  `python3 boom.py` → ⌘⇧P → "rerun" → the command re-ran as a new block (palette-focused, no terminal
  focus). `last_failure_block_index` at cov/MSI 100. The per-block "re-run to verify" affordance and the
  cross-pane "most recent failure" selection are follow-ups (the latter with #295). GATE GREEN [diff].

- **Multi-frame stack-trace navigator — a Python traceback's frames light the diagnostics gutter too**
  (TICKET-291, forge #291; sprint M18). Extends #289's gutter to the frames of a panic/traceback/stack. The
  crux is the Python `File "<path>", line <N>, in <fn>` shape, which #212's `path:line` scanner MANGLES (the
  quoted path + the separated line) — the rust panic (`panicked at p:l:c`), backtrace (`at p:l`), and node
  (`at f (p:l:c)`) frames are already `path:line`-shaped, so `scan_links` covers them. A pure
  `parse_trace_frames(output)` parses every frame in order (`python_frame` — a `split_once` chain with no
  index arithmetic — else the first `scan_links` file-ref via the reused #213 primitive `first_file_ref_on_line`),
  and a pure `trace_diagnostic_rows` folds the open-file frames into `open_file_diagnostic_rows` (unioned with
  #289's `diagnostics_for_file`, sort+dedup collapsing the `path:line` overlap) — so the #289 gutter marks them
  and #290's F8 walks them, no new UI. Proven live on the running app: a Python-traceback failure block →
  `trace_rows=[2] diag_rows=[]` (the frame's row comes ONLY from #291; `scan_links` finds nothing in the Python
  shape) → the editor gutter line lit danger-red (measured (218,98,105)). `parse_trace_frames` +
  `trace_diagnostic_rows` at cov/MSI 100 (the real 32-mutant set). GATE GREEN [diff].

- **F8 / ⇧F8 walk the caret through a failed command's error locations**
  (TICKET-290, forge #290; sprint M18). Builds on #289's diagnostics gutter: F8 jumps the caret to the
  next diagnostic row (⇧F8 the previous), in source order, wrapping at the ends, scroll-following (#270)
  — the universal next-error convention. Pure `next_diagnostic(rows, current)` (the first row strictly
  after the caret, else wrap to the first) + `prev_diagnostic` (strictly before, else wrap to the last),
  `None` on no diagnostics; the strict comparison lets a held F8 advance off a diagnostic row. The two
  Editor-scoped keymap bindings (#265 KeyContext) + a `dispatch_action` arm are the shim. `next_diagnostic`
  + `prev_diagnostic` at cov/MSI 100. GATE GREEN [diff].

- **Editor diagnostics gutter — a failed command's file:line refs light up the open file's gutter**
  (TICKET-289, forge #289; sprint M18). The "errors show up IN the editor" half of the wedge. When a
  command block fails and its output references lines in the file open in the editor, those rows' gutter
  line-numbers turn red (the IDE-standard inline diagnostics). A pure `diagnostics_for_file(output,
  open_path, root)` filters the block's `file:line` refs (reusing #212's `scan_links`) to those resolving
  (#190) to the open file, maps 1-based lines → 0-based rows, and sorts + dedupes; the editor render
  captures the sorted set (mirroring #272's find-match capture) and tints the gutter `danger` at each. The
  source is each terminal pane's LAST block (Failure only) so the markers self-clear when a new command
  runs. Unblocks #290 (nav across them) and #292 (run→fix loop). `diagnostics_for_file` at cov/MSI 100.
  GATE GREEN [diff].

- **A failing command → a "Jump to Failure" block action → the editor at the failing line (the run→fix loop)**
  (TICKET-213, forge #213; sprint M18). The run→fix half of the fusion wedge, built on #212. When a command
  block FAILS (non-zero exit) and its output references a source location, the block's right-click menu (#175)
  gains a "Jump to Failure" row (leading the 7-row table) that opens the editor at the primary failure line.
  A pure `first_failure_ref(output)` picks the primary ref — the FIRST output line carrying a `file:line`
  (compilers/tests emit the primary error first); the row is conditional on `has_failure` (the block failed
  AND a ref exists, computed when the menu opens), so a succeeding block or a failed block with no ref shows
  the unchanged 6-row menu. Dispatching it reuses #212's `open_file_at`. `first_failure_ref` + the `items_for`
  gating at cov/MSI 100. GATE GREEN [diff].

- **Click a `file:line:col` in terminal output → open the editor at that line (the fusion wedge foundation)**
  (TICKET-212, forge #212; sprint M18). The first ticket of the M18 Terminal↔Editor Fusion train — where
  terminal-first beats a plain editor+terminal. #196 already scanned block output for clickable file paths and
  STRIPPED a trailing `:line[:col]` to find the path (its own doc: "the line/col is #212's concern") but
  discarded the location. Now the pure `parse_line_col` CAPTURES it — `LinkTarget::File` carries `{line, col}`
  (the last digit group is the line, a preceding one the col; a non-digit/empty group stops the scan) — the
  whole `path:line:col` ref is clickable (not just the filename), and clicking opens the file with the caret
  placed at that line/col via the pure `caret_for_line_col` (1-based → 0-based; col past EOL clamps to the line
  end, line past EOF to the last line). A bare path opens at the top exactly as #196. Reuses #196's scanner,
  #190 `resolve_under_root`, and the M15 editable editor's caret. Four downstream M18 tickets (diagnostics
  gutter, jump-to-failure, trace-frame navigator, file-ref context menu) build on this parser + open-at-line.
  Parser + caret math at cov/MSI 100. GATE GREEN [diff].

### Changed

- **Incremental highlighting windows the tree-sitter query walk to the edit's damage**
  (TICKET-285, forge #285; sprint M17). Follow-up to #274: the incremental path re-parsed the tree
  cheaply (~1.0ms) but still ran the SAME full-tree `QueryCursor` walk (`spans_from_tree`, ~5.7ms over
  27k captures on 8k lines) on every keystroke, flooring end-to-end incremental at 0.46 of a full
  highlight. Now, after the re-parse, `damage_window` merges `old_edited.changed_ranges(&new)` with the
  edit span; a grow-loop widens the window and re-queries a NEW `spans_in_window` (`set_byte_range`)
  until every fresh span lies INSIDE it (no straddler); `splice_spans` drops the cached spans touching
  the replaced region or overlapping the window, rebases the survivors by the edit delta, and unions the
  fresh in-window spans. `HighlightSession` caches the previous RAW span set (`last_spans`) as the splice
  base. The spliced set equals a full walk of the new tree as a MULTISET — so `lines_from_spans` yields
  BYTE-IDENTICAL per-line output (the #274 equivalence corpus, expanded with straddle / deletion /
  boundary / macro-adjacent cases, is the gate); a block-comment cascade grows the window to ~full and
  degrades gracefully to the #274 walk. Drops the query cost from O(file) to O(damage) and tightens the
  end-to-end perf pin from 3/4 to the originally-intended 1/3 of a full highlight. Pure `damage_window` /
  `window_extent` / `splice_spans` at cov/MSI 100. GATE GREEN [diff].

- **Headless driven testing — real keystrokes into the real app, no GUI needed** (TICKET-264, forge #264;
  sprint M16). A new `headless_drive` test lane adopts gpui's `test-support` harness (dev-dependency only;
  the shipping binary is byte-equivalent): `#[gpui::test]`s boot the REAL `RootView` in a test window on a
  tempdir config (`RootView::new_in` — the §14 `*_in(dir)` seam added to the boot ctor, so every settings
  read/persist lands in the tempdir and the user's `~/.marley` is unreachable), focus the root handle,
  inject actual keystrokes through the real key ladder, and assert state: virgin boot → the #247 launcher;
  seeded boot → a restored workspace; ⌘T → tab+1; the #265 ⌘D split (terminal +1 tab; editor: tab count
  unchanged AND the caret word "alpha" 0..5 actually selected — swallow-proof). Retires the locked-screen /
  frontmost-window / shadow-offset screencapture hazards for behavioral checks; runs in plain `cargo
  nextest` (40-70ms, 5 runs zero-flake). Ground-truth correction recorded: gpui 0.2.2 has NO
  `render_to_image` (the ticket's citation was the newer Zed tree; the 0.2.2 test platform produces no
  pixels) — the PIXEL half + gate:15 migration is the filed follow-up (#271). Hard-won lane rules
  documented: dropping a live `TerminalSession` on the test thread blocks forever (stack-sampled) → every
  test ends with an off-thread `reap_sessions`, mirroring the app's own boot idiom. Reference §20: N/A —
  Marley's own Apache-2.0 dependency, its public test harness.
- **Warp reference docs: the 78 crate pages' Subsystem cross-links resolve again** (TICKET-263, forge #263;
  sprint M16, docs-only). Every `docs/warp_architecture/crates/*.md` Subsystem field linked a nonexistent
  `../architecture/` directory — now `../subsystems/` (the real home; all 7 targets verified on disk). The
  link-checker written as the inspect step also caught one adjacent pre-existing stale link (the crates
  README's "Project map" row → a nonexistent `../README.md`, now the subsystems overview). Zero broken
  relative links remain in the crates corpus. Reference §20: N/A — reference-corpus hygiene.
- **The last 11 colorful emoji are gone — every rendered glyph is now a theme-tinted clean-room SVG**
  (TICKET-261, forge #261; sprint M16). chad: "NO mo emojis." The #232 audit's three remaining groups convert
  to the #137 vendored icon system: the overlay/search headers (🔍 across the palette / find bar / session
  filter / top search, 🧠 launcher → the sparkle, 📄 finder, 🕐 history, 🔨 forge → the wrench, 🛰 fleet,
  🔀 agent-diff) now lead with a muted tinted icon via a shared `icon_label` row; `file_icon` and `pane_icon`
  became `Icon`-returning pure seams (.rs/.json → generic code-brackets [no language logos — §20], .toml →
  gear, .md → lined document, else file page; Terminal/FileTree/CodeView/Git → terminal/folder/file/branch);
  10 new self-authored filled-geometry SVGs land with NOTES.md provenance. The inspect critics caught a
  CRITICAL: the new assets weren't registered in the static `Assets::load` include_bytes! match — every new
  icon would have rendered as a silent blank; fixed plus a permanent `assets_serve_every_icon_variant` guard
  (every Icon → path → load → non-empty). Also re-seated a `mutants::skip` that the new helper had detached
  from `caption_header` (the documented trap's third strike). Live-proven: palette magnifier, Files-panel
  gear/document/brackets rows, fleet grid — all monochrome, zero emoji; a 22-pattern sweep greps clean.
  Buckets B (affordance/status glyph sets) and the artist brand list stay deferred. Reference §20: N/A —
  Marley's own clean-room icon set.
- **The top-bar workspace indicator + switcher popover are gone — the left rail is the single workspace
  home** (TICKET-260, forge #260; sprint M16). chad (live review): "I don't want the project workspace at the
  top. we opt in for the left." The #235 "Marley · main" indicator and the #244 click-to-switch popover are
  removed from the top bar entirely (render blocks, the `workspace_switcher_open` state, the pure
  `titlebar::focused_workspace_indicator` / `workspace_switcher_rows` / `SwitcherRow` + their tests, and the
  indicator consts — purely subtractive, no dead code left). The left Workspace rail remains the canonical
  affordance (#233 click-switch, #236 highlight); the cockpit tabs (Details/Agents/Forge) re-anchor to the
  vacated slot-3 x — which also fixes a live defect: at the 1024px default width they previously sat fully
  under the centered occluding search bar. The M8 #142 OS-titlebar/footer `~/…/Marley · main` label is
  separate and unchanged. Live-proven via driven captures (no indicator, click at the old spot opens no
  popover and hits the working relocated tabs, rail highlight intact). Reference §20: N/A — Marley-specific
  chrome removal per owner feedback.
- **Brand-scrub: the Marley source is brand-clean, and gate:14 keeps it that way** (TICKET-262, forge #262;
  sprint M16). All 56 whole-word "Warp" comment mentions across 11 `crates/**/*.rs` files (zero "Zed"; 100%
  comment prose — no strings, identifiers, themes, or config carried a brand) are reworded into Marley's own
  terms with every ticket ref + technical value preserved; the `color.rs` clean-room attestation now reads
  "independently authored, not lifted from any AGPL-licensed source" (a strictly broader claim). gate:14 gained
  a third check — `grep -rniwE 'warp|zed' crates --include='*.rs'` (POSIX `-w` guards the 80+ `…zed`-substring
  words like `StandardizedPath`) — so a brand mention can never re-enter the source; the reference
  transcriptions under `docs/*_architecture/` keep theirs deliberately (exempt by the `crates/`-only scope).
  Proven by a negative smoke (injected `// Warp` → gate:14 RED naming the exact line → revert) and a full
  GATE GREEN [diff] (15/15). The gate-14 row in `quality-bar.spec.md` documents the new check. Reference §20:
  N/A — Marley's own source hygiene.
- **Every pipeline spec must now carry a `## Reference (§20)` section** (TICKET-248, forge #248; sprint M15) —
  it names the reference app (Warp for the terminal/cockpit/UX; Zed for the editor) + how Marley matches its
  BEHAVIOR, or `N/A — Marley-specific + why`. Plan fills it, design confirms it + states the match, and a new
  `enforce-warp-reference.sh` commit hook blocks a staged spec whose section is empty — making clean-room-from-
  Warp (§20) disciplined, not ad-hoc. Observed captures live in the new `docs/warp_architecture/observed/`.
  (the first M15 ticket; the editor train inherits it)

- **Collapsed rail projects stay collapsed across a restart** (TICKET-245, forge #245; sprint M14) — the left
  rail's per-project collapse state (#236) was in-memory only, so a project you collapsed sprang back open on
  relaunch. It's now persisted (keyed by the project's root path, so it survives reordering) and restored on boot.
  (a #236 follow-on)

- **Open editor files now survive a restart** (TICKET-243, forge #243; sprint M14) — the editor surface
  persisted only its active file, so the other file-tabs vanished on relaunch; and opening a file didn't persist
  the workspace at all. Now the whole set of open files (+ which one is active) is saved and restored — open
  several files, quit, reopen, and they're all back. (a #237 follow-on)

- **Identical terminal tab names are disambiguated in the rail** (TICKET-241, forge #241; sprint M14) — several
  idle terminals rooted in the same directory all showed the same cwd-basename label (a wall of "Marley" rows).
  Duplicate labels now get a " 2"/" 3" suffix on the 2nd-and-later occurrence per workspace (unique labels — the
  cockpit tabs, the editor — are untouched). The numbering is computed over the full tab list, so it stays stable
  when the rail is scrolled or filtered. (chad live feedback, alongside #2)

- **Editor surface's rail row shows a stable "Editor" label** (TICKET-240, forge #240; sprint M14) — opening
  files into the editor surface left its left-rail row frozen on the FIRST file's name while the file-tab strip
  correctly showed each file. The rail row is now a calm, stable "Editor" (the strip stays the source of truth for
  which file); both editor-tab creation paths — opening a file and restoring a session — share one
  `EDITOR_TAB_TITLE`, so a restored editor tab keeps "Editor" too. A tab rename still overrides it. (chad live
  feedback #2)

- **Removed the redundant "MARLEY" session header from the left rail** (TICKET-239, forge #239; sprint M14) —
  the left rail's first row was an all-caps header echoing the top-level session/workspace name, duplicating what
  the "Workspace" panel title, the status bar, and the tabs already convey. Dropped it: the pure `tabs::rail_rows`
  no longer emits a Workspace row, and the `RailLevel::Workspace` variant + its render arm are gone — the rail now
  begins with the project row. (chad live feedback #1)

- **Command-block status indicator now renders SVG icons + a full emoji→icon audit** (TICKET-232,
  forge #232; sprint M13) — the per-block exit-status indicator (running / success / failure) rendered
  as text dingbats (○ ✓ ✗); it now renders clean-room monochrome SVG icons (`dot` / `check` / `cross`)
  through the #137 icon system, so it's crisp at any size and tints consistently with the theme. Also
  ships a comprehensive **emoji→icon audit** (`docs/marley_architecture/icon-audit.md`) cataloguing
  every rendered glyph, plus a **brand-icon list** splitting the standard utility icons (no artist
  needed) from the brand/identity set (app icon, agent mark, workspace mark). The audit found that 11
  colorful emoji still render across three subsystems the #137 top-bar migration never touched (the
  overlay/search headers, the file-type icons, the pane-title icons) — scoped into follow-up tickets.
  (chad live feedback #1: "remove all emojis for icons" + "a list of icons an artist should make")

- **Darker panel/dock surfaces (dark mode)** (TICKET-231, forge #231; sprint M13) — the dark theme's panels,
  docks, sidebar, and overlays used a mid gray (lightness 0.155) that read lighter than Warp's panels. Lowered
  it to a darker Warp-like gray (0.11) — still clearly raised above the near-black terminal background, the
  border still visible, and light-text legibility slightly improved (higher contrast). One theme-value change
  re-tones every surface consistently. (chad live feedback #3)

- **Sidebar and file-browser text calibrated to Warp size** (TICKET-230, forge #230; sprint M13; partially
  folds #223) — the file browser's file names rendered at the app's ~16px default (the largest text in the
  cockpit) and the session rail's tab rows at 13px, both bigger than Warp's ~12px sidebar. Both now render at
  12px via a new `Nav` type-scale role, so the left sidebar reads at Warp's density. The other ~9 chrome text
  sizes now also source from the `type_scale` table (no visual change, one place to tune); the remaining
  right-dock/completion/menu 13px + the fleet badge 9px are a follow-up (#223). Also refreshed a stale 14px
  default in the cell-metric fallback guard. (chad live feedback #2)

- **Command-palette shortcuts are now keycap chips** (TICKET-222, forge #222; sprint M12.2, Warp parity) —
  the palette used to show a command's shortcut as plain joined text ("cmd b") right after the title; it now
  renders each key as a little bordered, rounded keycap chip (`[cmd] [b]`), right-aligned at the end of the
  row like Warp. Reuses the existing `KeyboardShortcut` widget (the pure key-parse was already there; the
  chip render got a real keycap look) + the existing tokens — no new values. Original tokens — no Warp
  assets (clean-room). Completes the M12.2 Warp-parity sweep (#216–222).

- **The floating overlays are now rounded cards** (TICKET-221, forge #221; sprint M12.2, Warp parity;
  closes #224) — the command palette, agent launcher, file/history finders, forge & fleet overlays, the
  diff/find/completion overlays, and the right-click menu used to be square-cornered boxes; they now render
  as rounded, subtly-framed cards like Warp's overlays (a 6px corner radius + a 1px muted border on the
  borderless ones, with the content clipped to the rounded shape). Reuses the existing corner_radius/border
  tokens — no new values; this also reclaims the corner_radius token that #218 had orphaned (#224 resolved).
  Borders, separators, and the pane divider already read Warp-like and were left unchanged. Original tokens —
  no Warp assets (clean-room).

- **The terminal cursor dims on unfocused panes** (TICKET-220, forge #220; sprint M12.2, Warp parity) — the
  block cursor (from #218) used to render solid-bright on every pane, so in a split you couldn't tell which
  pane was active from its cursor. It now stays solid on the focused pane and dims to a translucent accent
  fill on unfocused panes — the classic focused/unfocused terminal cursor (Warp does the same). Same
  one-cell size (a dim FILL, not an outline — gpui has no box-sizing, so a border would have widened the
  cell). The drag-selection tint was evaluated against Warp and kept as-is. Original tokens — no Warp assets
  (clean-room).

- **The selected session in the sidebar is now clearly highlighted** (TICKET-219, forge #219; sprint M12.2,
  Warp parity) — the active tab/pane in the left rail used to be marked only by slightly brighter text; its
  intended highlight was the same gray as the sidebar, so it was effectively invisible. It now shows a
  distinct accent-tinted rounded highlight box (like Warp's selected row), and every clickable rail row
  (project/tab/pane) lights up with a subtler wash on hover. The highlight color comes from a new pure
  `rail_highlight` helper guarded to always differ from the sidebar background (cov/MSI 100). Reuses the
  `corner_radius` token (partially reclaiming #224). Original tokens — no Warp assets (clean-room).

- **The prompt matches Warp's bare look** (TICKET-218, forge #218; sprint M12.2, Warp parity) — the shell
  prompt used to sit in a gray rounded input box with a bright accent `❯` and a thin bar caret; it now sits
  directly on the terminal background (no box), the `❯` marker and the cwd breadcrumb are dimmed (muted), and
  the caret is a solid block cursor that reverse-videos the character under it (a standalone block at
  end-of-line) — matching Warp. The git-branch segment stays accent. Original tokens — no Warp assets
  (clean-room). Also adds a pure `split_caret_char` caret-split helper (the block-cursor logic, cov/MSI 100)
  and `left`/`right` verbs to the self-test driver.

- **Command-block actions now reveal on hover** (TICKET-217, forge #217; sprint M12.2, Warp parity) — each
  command block's ⧉ copy-command / ⧉ copy-output / ↻ rerun affordances used to sit visible in the block
  header at all times; they're now hidden by default and fade in only when you hover that block's header —
  matching Warp and decluttering the transcript so the command text reads cleanly at rest. The actions stay
  reachable when not hovering via the right-click block menu (unchanged), and the status glyph / command
  text / block separator are untouched. A gpui group-hover idiom (clean-room — no Warp assets).

- **The panel headers are more compact** (TICKET-216, forge #216; sprint M12.2, Warp parity) — the
  "Workspace" / "Files" / dock panel headers had a bit of extra vertical padding (tuned when the panel text
  was larger); they're now tightened to a compact band matching Warp's section headers. The rest of the
  cockpit's density already matched Warp after the color + font passes, so nothing else changed.

- **The cockpit text is calibrated to Warp's density** (TICKET-195, forge #195; sprint M12.2, Warp parity) —
  the terminal text and cockpit type scale were a hair large; they're now tightened ~1pt to match Warp's
  density — terminal text ~13pt (was 14), captions 11pt (was 12). Monospace columns stay aligned (the cell
  metric tracks the size), and the text stays comfortably legible. Original measured values — no Warp assets.

- **The dark theme reads terminal-black with gray panels** (TICKET-194, forge #194; sprint M12.2) — the dark
  theme's terminal pane and the surrounding panels used to sit only ~4% apart in lightness, so they blended
  together. The terminal pane (and the code view) is now a near-black, and the docks / sidebar / Files / footer /
  prompt surfaces are a distinctly lighter gray — a clean separation that matches the Warp look. Text and the
  ANSI colors stay legible on the darker background (proven by a new pure `contrast_ratio` WCAG guard, not
  eyeballed). Original values — no Warp assets copied (clean-room). The light theme is unchanged.

- **The folder + branch label moved to the bottom bar** (TICKET-192, forge #192; sprint M12.1) — the
  "~/…/Marley · main" project path and git branch used to sit in the top-right of the title bar; it now lives on
  the right side of the always-on status bar at the bottom of the window.

- **The active pane is now framed on all four sides** (TICKET-191, forge #191; sprint M12.1) — the focused
  pane's accent highlight was only a top + left edge; it's now a full border around all four sides of the pane,
  so the active pane reads as a clear box. (The right and bottom edges sit just inside the pane, visible thanks
  to the M12.1 right-gutter fix.)

### Fixed

- **The agent-launch picker now closes when you click outside it** (TICKET-229, forge #229; sprint M13) —
  launching an agent (⌘⇧A) opened the launch picker, but clicking outside it did nothing (only Escape/Enter
  closed it). Clicking anywhere outside the picker box now dismisses it (a full-screen click-away backdrop,
  matching the context menu); a click inside the box is unaffected, and keyboard selection still works (chad
  live feedback #5).

- **Focus border no longer hidden by the split divider** (TICKET-228, forge #228; sprint M13) — after
  splitting panes, the focused pane's accent focus border was covered by the draggable split divider on the
  shared edge (a paint-order bug: the border was drawn before the divider). The border is now drawn after the
  dividers (topmost pane-chrome, still under the modal overlays), so it shows on all four edges; the divider
  drag is unaffected (chad live feedback #4). A render-order fix; the pure `focus_border_rects` is unchanged.

- **The shell prompt no longer doubles up while a command or agent is running** (TICKET-193, forge #193;
  sprint M12.1) — when you launched `claude` (or ran any foreground command) in a terminal pane, Marley kept
  showing its own `❯` input row below the program's own prompt, so there were two places to type. Marley now
  hides its prompt input row whenever the pane's foreground command is running — matching how a terminal hands
  stdin to the running program — and brings it back the moment the command exits. The scrollback blocks stay
  visible throughout, and the row is dropped from the pane's internal row count too, so scrolling stays aligned.

- **Clicking a file in the browser now opens it** (TICKET-190, forge #190; sprint M12.1) — clicking a file in the
  file browser (or opening one via ⌘P) silently did nothing in the packaged app: file paths are stored relative
  to the project root, but the app read them relative to its launch directory (`/` for a double-clicked app), so
  the read failed and was swallowed. Paths are now resolved against the project root, so a file click reliably
  opens its code view from any working directory. (This also fixes a case where opening the same file two
  different ways could create duplicate tabs.)
- **The left panel is now labeled "Workspace", and the file browser "Files"** (TICKET-190, forge #190; sprint
  M12.1) — the left dock (your open tabs, with a "Search tabs" box) was mislabeled "Files" while the actual file
  browser had no title, so "click a file in the Files panel" led you to the wrong panel. The tab navigator now
  reads "Workspace" and the file browser has a "Files" header.

- **Split panes no longer run off the right edge of the window** (TICKET-189, forge #189; sprint M12.1) — when
  you split a terminal tab into side-by-side panes, the rightmost pane used to sit flush against (and visually
  bleed over) the window's right edge. It now ends 8pt inside the edge, giving every terminal tab a consistent
  right margin — split or single. Full-screen cockpit and code tabs are unaffected.

### Added

- **URLs and file paths in output are clickable** (TICKET-196, forge #196; sprint M12.2) — a URL printed by a
  command now renders underlined and opens in your browser on click; a file path (e.g. `src/app.rs`, a
  compiler/grep location) opens in the code view. The surrounding text is untouched, and you can still
  select/copy across a link (a link opens on a clean click, but a drag that starts on it selects instead).
  Clicking a path safely rejects a non-regular file (a device/fifo) and anything over 2 MB before reading.
  (Explicit OSC 8 terminal hyperlinks are a separate follow-up.)

- **Split a pane with the keyboard** (TICKET-197, forge #197; sprint M12.2) — splitting a terminal pane
  was right-click-menu-only; now **⌘⇧L** splits it to the right and **⌘⇧J** splits it down (the vim `l`/`j`
  directions), and both are in the command palette ("Split Right" / "Split Down"). Separately, the ⌘D
  command is now honestly labeled **"New Terminal"** — it makes a new terminal, not a split (its action was
  the misleading `"split-pane"`).

- **Tab-complete commands, not just directories** (TICKET-183, forge #183; sprint M12) — at the start of a
  prompt, Tab now completes against your `$PATH` executables and recent command history (recent first), instead
  of the current directory's files. Type `ca`+Tab and it offers `cargo`, `cat`, …; a later word on the line still
  completes file paths as before, and a path-form command (`./build`, `/usr/bin/…`) completes files too.

- **Step through find matches with a live "n of m" counter** (TICKET-186, forge #186; sprint M12) — the ⌘F
  find bar now shows your position as "n of m" across the whole scrollback, Enter / ⇧Enter jump to the next /
  previous match (wrapping) and scroll it into view, and the match you're on is highlighted brighter than the
  rest — including matches in a command line, not just its output.

- **The command header sticks to the top while you scroll its output** (TICKET-185, forge #185; sprint M12) —
  when you scroll deep into a long command's output, that command's header pins to the pane's top edge (so you
  always know which command you're reading), and switches as you scroll into the next block — the Warp/devtools
  sticky-scroll feel. Scrolling still works normally over the pinned header.

- **Fold a command's output to its header** (TICKET-184, forge #184; sprint M12) — every command block header now
  has a ▾/▸ chevron; click it to collapse that command's output rows to just the header (with its exit ✓/✕), and
  click again to expand. Handy for tidying long outputs. Fold state is per-block and resets when the command list
  changes; folding a block in an agent's pane doesn't affect what the cockpit observes about the agent.

- **Choose the agent and give it a starting prompt** (TICKET-181, forge #181; sprint M12) — ⌘⇧A (and the
  top-bar agent icon) now opens a small picker: pick the agent kind (Claude or Codex, ↑/↓) and optionally type
  an initial prompt, then Enter launches it in a split and hands it that prompt to start on. Esc cancels.
  Previously ⌘⇧A launched Claude immediately with no choice.

- **A live fleet badge on the top-bar Agents icon** (TICKET-188, forge #188; sprint M12) — the top-bar Agents
  cockpit icon now shows a small count of running agents (Working or Waiting), hidden when none are running;
  clicking the icon or the footer "n agents · n working" segment opens the Agents cockpit tab — a glance and one
  click to the observe surface.

- **See how an agent finished — exit code + run time** (TICKET-187, forge #187; sprint M12) — when an agent's
  shell exits, its Agents-tab row now stays and is marked ✓ (clean exit) or ✕ (non-zero), with the total run
  duration (e.g. `2m14s`), and a one-time flash reads "claude finished (exit 0, 2m14s)". Previously a finished
  agent just lingered with a stale status glyph and no result. Finished rows persist for the session.

- **Watch an agent's output from the Agents tab** (TICKET-180, forge #180; sprint M12) — each agent in the
  Agents cockpit tab now shows a live tail of its last few output lines beneath its status row, updating as
  the agent works, so you can keep an eye on it without switching to its pane.

### Fixed

- **⌘-clicking an agent shows its own project's diff; a dead agent says so** (TICKET-182, forge #182; sprint
  M12) — ⌘-clicking an agent in the Agents tab now shows the working-tree diff of the agent's project (not
  whichever project is active), and sending to an agent whose shell has exited now flashes "…has exited"
  instead of silently doing nothing (your typed line is kept either way).

- **Clicks near a command boundary land on the right block** (TICKET-179, forge #179; sprint M12) — the
  terminal's click→row hit-test was off by up to a row in the bottom sliver of each line (it measured from
  the pane top using the wrong height), so a right-click or text selection near a command's last line could
  target the next command. Now measured from the pane bottom, matching how rows are drawn.

### Changed

- **Internal cleanup: retired right-dock plumbing removed** (TICKET-171, forge #171; sprint M11) — dead code
  left behind when the right dock became full-screen cockpit tabs (#153); no user-visible change. An old
  `docks.right` line in a settings file is now simply ignored.

- **The completion menu filters as you type** (TICKET-178, forge #178; sprint M11) — with the Tab-completion
  menu open, typing more of the name now narrows the list in place (and backspace widens it back) instead of
  dismissing it; the menu closes once a single match remains, which the next Tab completes.

### Fixed

- **Sending to agents works across tabs** (TICKET-174, forge #174; sprint M11) — ⌘⇧S (send to the last
  agent) and the broadcast now reach agents in background tabs and other projects; previously they silently
  failed while looking successful (the broadcast even tagged skipped agents with the ticket). Clicking a
  rail tab in another project now also refreshes the Files tree and branch to that project.

- **Background tabs stay live** (TICKET-173, forge #173; sprint M11) — every tab's terminal is now serviced
  continuously, not just the visible one: a long-running command in a background tab keeps running instead
  of stalling once its output buffer fills, agent status glyphs in the rail stay truthful across tabs, and
  live tab titles keep updating. A pane whose shell exits in a background tab now cleans up there too.

### Added

- **Rename a tab** (TICKET-177, forge #177; sprint M11) — double-click a tab in the rail to rename it; the
  custom name overrides the live command name and survives a relaunch (closing the gap where restored tabs
  reverted to "terminal N"). Renaming is for terminal tabs; cockpit/code tabs keep their fixed names.

- **The window and Files panel remember themselves** (TICKET-176, forge #176; sprint M11) — the window
  reopens at its last position and size (safely re-centered if that spot no longer exists — say, an
  unplugged monitor), and the Files panel comes back open if you left it open.

- **Right-click a command block for its actions** (TICKET-175, forge #175; sprint M11) — right-clicking
  inside a block now opens a menu with Copy Command, Copy Output, and Rerun (the same actions as the block's
  header buttons) plus the split actions; right-clicking empty space or a full-screen program (vim, less)
  keeps the plain split menu, and right-clicking away closes an open menu.

- **Agent rows navigate** (TICKET-174, forge #174; sprint M11) — clicking an agent in the Agents tab or the
  Fleet overlay now jumps to its pane wherever it lives (switching project and tab as needed); a row whose
  pane is gone says so instead of doing nothing.

- **Self-test harness: typed input re-verified** (TICKET-172, forge #172; sprint M11) — the drive harness's
  `type:` verb was retested end-to-end (types, executes, renders a ✓ block) after stale session notes
  claimed it broken; the README gains the retest note, the bare-verb-names trap, and the recheck recipe.
  No code changed.

- **Tab-completion menu for ambiguous matches** (TICKET-96, forge #96; sprint M10) — when Tab finds several
  matches and the shared prefix is already typed, a small menu lists them above the prompt: Tab/↓/↑ cycle
  (shift-Tab backward), Enter or a click accepts, Esc closes, and just typing on continues naturally.

### Changed

- **New terminals start in the project's folder** (TICKET-160, forge #160; sprint M10) — new tabs (⌘T/+),
  splits (⌘D / the context menu), and agent launches now open their shell in the ACTIVE project's root
  instead of wherever the app was launched from.


- **The whole workspace survives a relaunch** (TICKET-163, forge #163; sprint M10) — projects, their tabs
  (terminal splits, cockpit sections, open code files), and which tab/project was active are all restored on
  boot. Terminals respawn fresh in their project's directory; a since-deleted project or file is skipped
  gracefully. Previously only the first tab's pane arrangement came back.

- **Rail tabs show their agents' status** (TICKET-167, forge #167 + #158; sprint M10) — a tab hosting an
  agent shows its status glyph (● working in accent, ◔ waiting, ○ idle, ✓ exited) before the title. Under
  the hood, pane ids are now globally unique across tabs, fixing a latent cross-tab mix-up in agent tracking
  and making closes clean up their agent/remote entries exactly.


- **The rail and Files tree scroll** (TICKET-169, forge #169; sprint M10) — when the left lists overflow
  (many tabs, a deep repo), the wheel now scrolls them past the fold, clamped at both ends. The Files offset
  resets when the tree refreshes.


- **Files panel: fresh on open, resizable by drag** (TICKET-168, forge #168; sprint M10) — opening the Files
  panel now re-scans the project (files created since the last open appear), and its right edge drags to
  resize (160–480pt), with the width remembered across restarts.


- **Tab chords** (TICKET-170, forge #170; sprint M10) — `⌘T` opens a new terminal tab, `⌘[` cycles to the
  previous tab (`⌘]` already cycles forward), and `⌘1`–`⌘9` jump straight to that tab.


- **Right-click opens a split menu** (TICKET-166, forge #166; sprint M10) — right-clicking a terminal now
  shows a Warp-style menu at the pointer: Split Right, Split Down (new), and Close Pane. Esc or clicking away
  dismisses; ↑↓ + Enter work too. (Previously a right-click split immediately.)

### Fixed

- **Code tabs scroll again** (TICKET-165, forge #165; sprint M10) — the M9 render move left the full-screen
  code viewer unscrollable (files longer than 40 lines were cut off). The wheel now scrolls it with the same
  sub-row precision as the terminal, clamped at both ends.

### Changed

- **Each file opens its own code tab** (TICKET-164, forge #164; sprint M10) — opening a second file no longer
  replaces the code tab: every file gets its own tab in the rail, and re-opening a file switches to (and
  refreshes) its existing tab. Close them like any tab (× / ⌘W).


- **Projects can be closed from the rail** (TICKET-162, forge #162; sprint M10) — each project row has an ×;
  closing a project drops all its tabs and terminals, and the Files tree, branch, and cwd resync to the
  surviving project. Closing the last project is refused with a flash.


- **Tabs can be closed** (TICKET-161, forge #161; sprint M10) — each rail tab row has an ×, and `⌘W` now
  closes panes first, then the tab at its last pane (a cockpit/code tab closes immediately). Closing a
  project's last terminal tab is refused with a flash, so a project always keeps a terminal.


- **The rail comes alive** (TICKET-157, forge #157; sprint M9) — a terminal tab row now shows its running
  command (the program) instead of a static "terminal N", and a project row shows its real git branch as a
  subtitle (`name · branch`; just the name when there's no branch). Completes the M9 workspace re-architecture.


- **A workspace holds multiple projects** (TICKET-156, forge #156; sprint M9) — `⌘O` opens a folder as a new
  project in the workspace (its own root, cwd, branch, file tree, and tabs); click a project in the rail to
  switch to it. The Files tree, titlebar path, branch, and git commands all follow the active project.


- **Right-click a terminal to split it** (TICKET-155, forge #155; sprint M9) — splitting is now opt-in per tab:
  right-click a terminal pane to tile it (reusing the drag-resize + ⌘⌥-arrow nav), and the split panes appear
  nested under the terminal tab in the rail. Full-screen tabs stay the default.


- **Files is a left panel and opening a file opens a full-screen code tab** (TICKET-154, forge #154; sprint M9)
  — the 📁 icon (or the "Files" command) toggles the project file tree as a LEFT panel that expands from the
  left, never a mid-screen split. Clicking a file opens it as a full-screen CodeView tab in the rail (one code
  tab is reused). The tiled file/code panes are retired.


- **Details / Agents / Forge open as full-screen tabs** (TICKET-153, forge #153; sprint M9) — clicking a
  cockpit icon now opens it as a tab that fills the workspace and appears in the rail, instead of a panel
  wedged on the right. The right dock is retired; `⌘⇧B` opens the Details tab.


- **The sidebar is now a Workspace → Project → Tab tree** (TICKET-152, forge #152; sprint M9) — the flat
  session list is replaced by the real hierarchy: a workspace header, its projects, and each project's tabs.
  Click a tab to switch to it (the active tab is highlighted and fills the workspace). "Search tabs" filters
  the tab rows.


- **New terminals open as full-screen tabs, not splits** (TICKET-151, forge #151; sprint M9) — the `+` button
  now adds a terminal *tab* that fills the workspace, instead of tiling the screen. Switch tabs with `⌘]`.
  Splitting is now opt-in — `⌘D` still splits the current tab into tiles (and the existing tiled panes within
  a tab are preserved). This is the first visible step of the Workspace → Project → Tab shell.


- **Internal groundwork for the workspace / project / tab shell** (TICKET-150, forge #150; sprint M9) — a new
  `Workspace → Project → Tab` model lands underneath the UI, and the former tiled-grid type is renamed
  `PaneGrid` (it now lives inside a terminal tab, so all of the split/resize behavior is preserved). No
  user-visible change yet; the app switches over to full-screen tabs in the following M9 tickets.

### Fixed

- **Pane focus navigation now uses ⌘⌥-arrow** as documented (TICKET-144, forge #144; sprint M8) — it was
  mistakenly bound to ⌘⇧-arrow (a shift/alt mix-up), so the ⌘⌥-arrow keys did nothing. Found by live-driving
  the shipped interactions once synthetic input was unblocked (#140).

### Added

- **The title bar shows where you are** (TICKET-142, forge #142; sprint M8) — the unified title bar now
  displays the abbreviated working directory and the current git branch (e.g. `~/…/Marley · main`), read live
  from the project root and `.git/HEAD`, instead of showing nothing.


- **Search you can drive from the keyboard** (TICKET-141, forge #141; sprint M8) — the top-bar search results
  now have a highlighted selection you move with ↑/↓ and open with ↵, and every result activates (a file
  opens in the viewer, a session focuses its pane), not just files on a mouse click.


- **Files always open on the right** (TICKET-139, forge #139; sprint M8) — opening a file, code, or
  source-control pane now places it at the rightmost position consistently, instead of splitting whichever
  pane happened to be focused (so the order no longer depends on what you had selected).


- **One unified title bar** (TICKET-138, forge #138; sprint M8) — the icons and search now share the row with
  the macOS traffic-light buttons (Warp-style), instead of sitting in a separate bar below them. The
  redundant "Marley" title is gone and the pane grid gets that vertical space back.


- **Real icons instead of emoji** (TICKET-137, forge #137; sprint M8) — the top-bar and cockpit glyphs are now
  crisp monochrome SVG icons that follow the theme color (the active cockpit icon tints to the accent),
  replacing the emoji (which rendered in their own colors and couldn't be tinted). Swappable for a different
  icon set later.


- **Launch an agent from the top bar** (TICKET-136, forge #136; sprint M7) — a 🧠 icon next to the "+" starts
  an agent session (it appears in the sidebar and the Agents panel), so you can add a terminal *or* an agent
  straight from the top bar.

- **Add a terminal with the "+"** (TICKET-135, forge #135; sprint M7) — a "+" in the top bar spawns a new
  terminal session (it shows in the sidebar as "terminal 2", "terminal 3", …). ⌘D now shares the same path.

- **Cockpit tabs are now top-bar icons** (TICKET-134, forge #134; sprint M7) — Details / Agents / Forge show
  as a compact icon cluster in the top bar (the active one highlighted) instead of overlapping text tabs.

- **File-explorer icon in the top bar** (TICKET-133, forge #133; sprint M7) — a folder icon in the top bar
  opens the file tree, so it's reachable with a click instead of only through the command palette.

- **A real top bar** (TICKET-132, forge #132; sprint M7) — the global search now lives in a dedicated bar
  at the top instead of floating over the terminal, and the pane grid sits below it so each pane's close
  button is clickable again (it was being covered by the floating search/tabs).

- **The Warp layout, landed** (TICKET-127, forge #127; sprint M6 finale) — Marley boots into the Warp
  arrangement: the sessions sidebar on the left, a full-width grid of titled panes (a terminal, plus files /
  code / source-control panes you open on demand), the cockpit tabs across the top, and the right side left
  free. `default_grid` (a single terminal) is now the tested first-run default. This closes milestone M6.

- **Move focus between panes + a Warp focus edge** (TICKET-131, forge #131; sprint M6) — ⌘⌥-arrow moves focus
  to the neighboring pane by direction, and the focused pane is now marked with a bright cyan left+top edge
  (the Warp style) instead of a full border.

- **Drag to resize panes** (TICKET-130, forge #130; sprint M6) — a divider handle between tiled panes lets
  you drag the split ratio (previously a fixed 50/50), clamped so no pane collapses. The handle shows as a
  subtle line that brightens on hover.

- **The sidebar shows real terminals only** (TICKET-129, forge #129; sprint M6 9/10) — a files / code / git
  pane no longer appears in the workspace list as a phantom "terminal 2". The sidebar now lists just the
  actual terminal sessions.

- **Details / Agents / Forge tabs moved to the top** (TICKET-126, forge #126; sprint M6 8/10) — the cockpit
  tabs now live in the top bar instead of a right-side rail, so the right side is free for the pane grid
  (the last of the four duplicated surfaces, gone).

- **Open any pane on demand** (TICKET-128, forge #128; sprint M6 7/10) — a "Files" command in the palette
  opens the file explorer pane (joining ⌘-click to open a code pane and ⌘⇧C for source control). Opening a
  files/git pane focuses the one that's already there instead of piling up duplicates.

- **Source control is now its own pane** (TICKET-125, forge #125; sprint M6 6/10) — ⌘⇧C opens a Git pane in
  the grid (change list, stage toggles, message box, and Commit), retiring the M5 right-side git panel. All
  three side surfaces (files, code, git) are now real tileable panes — the right side is clear. The git-write
  surface stays confined to add / restore / commit exactly as before.

- **The code viewer is now its own pane** (TICKET-124, forge #124; sprint M6 5/10) — opening a file shows
  it in a real CodeView pane in the grid (line-number gutter, scroll, and syntax highlighting), reusing one
  pane instead of a right-side panel. Retires the M5 side-panel — the second duplicate surface is gone.

- **The file explorer is now its own pane** (TICKET-123, forge #123; sprint M6 4/10) — the project tree
  moved out of the left sidebar into a dedicated Files pane in the grid (click folders to expand, ⌘-click a
  file to view it). The sidebar is now just your sessions, like Warp — no more duplicate file tree.

- **Marley remembers your pane layout** (TICKET-122, forge #122; sprint M6 3/10) — the pane arrangement is
  saved and restored across launches, so Marley reopens with the panes you left open (and lets us boot
  straight into a terminal + files + code + git layout). Replaces the temporary boot behavior from #121.

- **Files/code/git panes now render their content** (TICKET-121, forge #121; sprint M6 2/8 — the first
  visible step) — a non-terminal pane draws a real body in the grid (a file explorer shows the project
  tree with type icons; a code pane shows the file; a git pane shows source control), tiled beside the
  terminal, each with its own title bar — no longer an empty frame. The app currently boots with a Files
  pane beside the terminal so you can see it; the next ticket makes the opening layout configurable.

- **Panes can now be more than terminals** (TICKET-120, forge #120; sprint M6 1/8 — the Warp-layout
  foundation) — internally, a pane now holds typed content (a terminal **or** a session-less file /
  code / git panel) instead of always a terminal. Nothing changes on screen yet; this is the structural
  groundwork that lets the file explorer, code viewer, and git panel become real, tileable panes in the
  grid (the next M6 tickets) rather than fixed docks and overlays.

- **The workspace remembers your layout** (TICKET-118, forge #118; sprint M5 12/12 — FINALE, closes M5) —
  Marley reopens with the source-control panel in the state you left it (persisted to `settings.toml`).
  Completes **M5 — The Warp Workspace**: a sessions sidebar (grouped, searchable), pane title bars, a
  file explorer with type icons, the code viewer beside the terminal, a git panel with stage + commit, and a
  global top search bar.

- **A global search bar at the top** (TICKET-117, forge #117; sprint M5 11/12) — one "Search sessions,
  agents, files…" field at the top of the window fuzzy-searches your sessions, project files, and palette
  actions at once, tagged by kind; a file result opens it in the viewer. The Warp top command bar.

- **Stage and commit from the git panel** (TICKET-116, forge #116; sprint M5 10/12) — click a file to stage
  or unstage it, type a message, and hit Commit. Marley's first git write, deliberately confined to
  `add` / `restore --staged` / `commit` (never push, force, or history-rewrite; the message is passed as an
  argument, never a shell string).

- **A git changes panel** (TICKET-115, forge #115; sprint M5 9/12) — ⌘⇧C opens a source-control panel
  showing the working tree's uncommitted changes (each file's status + whether it's staged), or "No open
  changes". Read-only for now; committing arrives next.

- **The code viewer sits beside the terminal** (TICKET-114, forge #114; sprint M5 8/12) — opening a file now
  shows the viewer as a right-side panel (the terminal shrinks to make room, the viewer has a filename title
  and an × to close), instead of a modal overlay covering the terminal. The Warp code pane.

- **File-type icons in the tree** (TICKET-113, forge #113; sprint M5 7/12) — the Files tree now shows a
  type icon per file (Rust, TOML, JSON, Markdown, or a generic document) and dims dot-files, the Warp
  explorer look.

- **Search your sessions** (TICKET-112, forge #112; sprint M5 6/12) — a "Search tabs" field atop the sidebar
  fuzzy-filters your sessions by title or subtitle as you type. Completes the Warp sessions sidebar.

- **Richer session rows** (TICKET-111, forge #111; sprint M5 5/12) — each sidebar row now shows a two-line
  layout (a title over a muted subtitle) and an icon that reflects an agent's state: active ✳ / idle ✧ /
  terminal ▸.

- **Sessions grouped by workspace** (TICKET-110, forge #110; sprint M5 4/12) — the sessions sidebar now
  clusters sessions under named workspace headers (like Warp's Rusty / Forge / Dev N), in first-seen order.

- **A sessions sidebar** (TICKET-109, forge #109; sprint M5 3/12) — the left dock now lists your active
  terminal and agent sessions at the top (each a clickable row that focuses its pane, the current one
  highlighted), above the file tree — the Warp "active sessions on the left". Promotes the ⌘⇧E Fleet overlay
  into always-on chrome.

- **Pane title bars** (TICKET-108, forge #108; sprint M5 2/12) — every pane now shows a title bar at its top
  edge (a type icon, the pane's name, and an × to close it), the first of the Warp-style workspace chrome.

- **Typed panes — the Warp-workspace foundation** (TICKET-107, forge #107; sprint M5 1/12) — every workspace
  pane now carries a kind (terminal / files / code / git), defaulting to terminal and preserved across
  split/close. Internal groundwork so the grid can host file, code, and git panes beside terminals (M5).

- **The code viewer guards against binary/huge files + honors `code.tab_width`** (TICKET-106, forge #106;
  sprint M4 10/10 — FINALE, closes M4) — opening a binary or >2 MB file now shows a placeholder instead of
  rendering garbage or stalling, and the viewer's tab width is configurable via `code.tab_width` in
  `settings.toml` (default 4). Completes **M4 — The Code Panel**: a read-only code viewer with syntax
  highlighting, a line-number gutter, scrolling, working-tree diffs (⌘⇧D), ⌘-click-an-agent-to-see-its-diff,
  and click-to-open file references.

- **Click a diff's file header to open that file** (TICKET-105, forge #105; sprint M4 9/10) — clicking a
  file name in the diff overlay opens it in the code viewer. Recognizes `path:line` references (a
  `parse_file_ref` seam), so opening agent/terminal output tokens at a line is a small follow-up.

- **⌘-click an agent to see what it changed** (TICKET-104, forge #104; sprint M4 8/10) — the payoff of the
  command center: ⌘-click an agent in the dock's Agents section to open the working-tree diff, headed by a
  `N files · +A −R` summary, so you can review the code your agents produced. (v1 shows the current working
  diff; per-agent scoping is a follow-up.)

- **The diff render is a tested projection** (TICKET-103, forge #103; sprint M4 7/10) — the ⌘⇧D diff view
  now renders through a pure `diff_rows` projection (file/hunk/line rows with color roles), so the +/-
  styling is unit-tested rather than inline render logic.

- **See the working-tree diff with ⌘⇧D** (TICKET-102, forge #102; sprint M4 6/10) — ⌘⇧D shows the repo's
  uncommitted `git diff` in a colored overlay (green additions, red removals), per file and hunk. Read-only
  (Marley never writes to git). The seam the agent-diff review is built on.

- **Scroll the code viewer** (TICKET-101, forge #101; sprint M4 5/10) — ↑/↓ scroll the viewer a line at a
  time and ⌘↑/⌘↓ jump to the top/bottom; the viewer renders only its visible window (so large files stay
  fast). Sets up jump-to-a-line for the diff and file-reference features.

- **A line-number gutter in the code viewer** (TICKET-100, forge #100; sprint M4 4/10) — the viewer shows
  right-aligned line numbers in a gutter sized to the file's length.

- **Syntax highlighting in the code viewer** (TICKET-099, forge #99; sprint M4 3/10) — the viewer now colors
  keywords, strings, comments, and numbers for Rust / TOML / JSON / Shell (Markdown + plain text pass
  through). A lightweight, dependency-free clean-room lexer — no tree-sitter or syntect.

- **⌘-click a file in the tree to view it** (TICKET-098, forge #98; sprint M4 2/10) — ⌘-clicking a file in
  the Files tree now opens it in the code viewer (a plain click still inserts the path at the prompt);
  directories are guarded. Shares one open path with the ⌘P-finder ⌘↵ open.

- **A read-only code viewer** (TICKET-097, forge #97; sprint M4 1/10 — begins The Code Panel) — open a file
  from the ⌘P finder with **⌘↵** to view it in a scrollable overlay with line numbers (tabs expanded, long
  lines truncated); Esc closes. The foundation for the code/diff panel — syntax highlighting, git + agent
  diffs, and jump-to-file follow.

- **The cockpit rail remembers its active tab** (TICKET-095, forge #95; sprint M2.F 6/6 — **completes The
  Persistent Cockpit**) — the right dock now persists which section (Details / Agents / Forge) you last had
  open and reopens on it after a relaunch. Together M2.F turns the transient ⌘⇧ overlays into an always-on
  cockpit rail: tabbed Details / Agents / Forge sections, a live per-kind pane inspector, a status-bar
  footer, and remembered dock state.

- **A cockpit status bar** (TICKET-094, forge #94; sprint M2.F 5/6) — a thin always-on footer strip under
  the panes shows the whole cockpit's state at a glance: the active sprint + ticket count, the agent-fleet
  summary ("2 agents · 1 working"), and the focused pane. Updates live every frame.

- **The Details tab inspects the focused pane by kind** (TICKET-093, forge #93; sprint M2.F 4/6) — the
  right dock's **Details** tab is now a live inspector that adapts to whatever pane is focused: a terminal
  shows its running/last command + status + exit code + cwd + git branch; an agent shows its CLI + status +
  ticket + last line; a remote shows its host + connection state. Recomputed every frame.

- **The Forge sprint view is always visible in the dock** (TICKET-092, forge #92; sprint M2.F 3/6) — the
  right dock's **Forge** tab now shows the active sprint's tickets (status glyph + number + title), auto-
  refreshing, with the same modifier trio as the ⌘⇧F overlay: plain-click copies the `#N — title` ref,
  ⌘-click claims the ticket, ⇧⌘-click posts your composed line as a comment. ⌘⇧F stays as a quick toggle.

- **The agent Fleet is always visible in the dock** (TICKET-091, forge #91; sprint M2.F 2/6) — the right
  dock's **Agents** tab now lists every running agent (glyph + label + ticket + status + quiet-age + last
  output line), each row clickable to jump to that agent's pane; an empty fleet shows a launch hint. You no
  longer have to toggle the ⌘⇧E overlay to see your agents (it stays as a quick peek).

- **The right dock is now a tabbed cockpit rail** (TICKET-090, forge #90; sprint M2.F 1/6 — begins The
  Persistent Cockpit) — the right dock gained **Details / Agents / Forge** tabs (click to switch; the active
  tab is accent-highlighted). Details keeps the focused-pane inspector; the Agents and Forge tabs are the
  foundation for making the agent Fleet + sprint view always-visible instead of ⌘⇧-overlays (filled in the
  next tickets). The ⌘⇧E / ⌘⇧F overlays remain as quick toggles.

- **Per-agent "quiet" age in the Fleet** (TICKET-082, forge #82; sprint M2.E 5/5 — **completes The Observing
  Cockpit**) — each agent's Fleet row now shows how long it's been quiet ("quiet 5s" / "quiet 1m") next to
  its status, so a briefly-thinking agent reads apart from a long-stalled one. Pairs with the ◔ waiting
  status. Together M2.E turns the Fleet into a live board: last output line, waiting state, quiet age, the
  ticket each agent is on, and click-to-jump.

- **Click an agent in the Fleet to jump to it** (TICKET-081, forge #81; sprint M2.E 4/5) — clicking any
  row in the ⌘⇧E Fleet overlay now focuses that agent's pane and closes the overlay, so you can jump
  straight to the agent you want to watch or talk to.

- **See which ticket each agent is on** (TICKET-080, forge #80; sprint M2.E 3/5) — when you send (⌘⇧S) or
  broadcast (⌘⇧G) a line that mentions a ticket ("go work #77"), Marley remembers agent→#77 and shows it
  on the agent's pane badge + its Fleet row, so you can tell who's working what.

- **Agents show a "waiting" status** (TICKET-079, forge #79; sprint M2.E 2/5) — a running agent that's been
  quiet for ~1s reads as **◔ waiting** (probably at its prompt, waiting for you) instead of ● working, so
  you can tell at a glance which agents need your attention. Reverts to ● working the moment it produces
  output again.

- **An agent's last output line in the Fleet** (TICKET-078, forge #78; sprint M2.E 1/5 — begins The
  Observing Cockpit) — the ⌘⇧E Fleet overlay now shows each agent's most-recent output line next to its
  status, so you can see what every agent is doing at a glance without focusing its pane. Refreshed live
  each frame; truncated hard (claude's output is noisy).

- **Tab completion at the prompt** (TICKET-089, forge #89; Terminal Polish — resolves the long-deferred
  #51/#33 fork as a Marley-local engine) — pressing Tab at the Marley prompt now completes a path against
  the current directory (a single match fills in, and dirs get a trailing `/`; multiple matches extend to
  the longest common prefix like a real shell) instead of inserting a literal tab. A running program still
  gets Tab (e.g. shell completion inside `ssh`/an editor). The multi-match popup is a follow-up (#96).

- **Saved ssh hosts + a "connect: …" palette action** (TICKET-087, forge #87; sprint M3.A 5/5 — **completes
  The Remote Seam**) — a `[[remote.hosts]]` TOML setting (`name` + `target`) saves your ssh destinations,
  and the command palette (⌘⇧P) lists a "connect: {name} → {target}" entry per host that opens a remote
  pane (the #84 path). A host with an invalid target is skipped (never a boot crash — the settings load is
  fully tolerant). `remote_palette_actions` is pure (cov/MSI 100); the open path is now a shared
  `open_remote_target` used by both ⌘⇧O and the palette.

- **Remote connection status (connected / disconnected)** (TICKET-086, forge #86; sprint M3.A 4/5) — when
  a remote pane's ssh exits (network drop, remote logout, or a clean `exit`), the pane now STAYS visible
  with its badge flipped ⇄ → "✗ {host}" and a "disconnected: {host}" flash, instead of silently
  vanishing (a visibly-dead remote beats a disappearing pane — mirrors the last-pane rule). Non-remote
  panes still auto-close on exit. `RemoteStatus` + `remote_status_from` / `remote_status_glyph` are pure
  (cov/MSI 100); the pump flips the status exactly once on the transition (no repaint churn on the
  repeatedly-erroring dead session).

- **Remote-pane badge (⇄ host)** (TICKET-085, forge #85; sprint M3.A 3/5) — a remote (ssh) pane now shows a
  "⇄ {host}" pill at its top-left corner (mirroring the agent badge, which sits top-right), so a remote
  pane reads as remote at a glance + names its host. Tagged when the pane opens (#84), dropped when it
  closes. Verified live: ⌘⇧O on `localhost` shows a "⇄ localhost" badge.

- **Open a remote ssh pane** (TICKET-084, forge #84; sprint M3.A 2/5) — compose an ssh target at the prompt
  (`user@host:port`), press ⌘⇧O, and a new pane opens running `ssh <target>` — a normal terminal pane,
  connected remotely. `SessionOptions` gained an `args` field so the existing PTY spawn can run any program
  (the local zsh spawn is unchanged); the ssh argv comes from `marley_remote::ssh_command` (#83's guarded,
  injection-safe form). Verified live: typing `localhost` + ⌘⇧O spawned `ssh localhost` (confirmed via
  `ps`).

- **`marley_remote` — the ssh target seam** (TICKET-083, forge #83; sprint M3.A 1/5 — begins The Remote
  Seam) — a new pure crate that parses a typed `[user@]host[:port]` (including bracketed IPv6 `[::1]:22`)
  into an `SshTarget` and builds the `ssh` argv. Marley will spawn the user's own `ssh` client as a
  terminal pane (like Terminal.app), so `ssh` owns all security (keys, known_hosts, auth). The argv is a
  `Vec<String>` and a leading-`-` host/user is rejected + a `--` guards the destination, so nothing can
  inject a shell command or smuggle an ssh option (CVE-2017-1000117 class). cov/MSI 100.

### Fixed

- **Search focus no longer sticks or grabs the wrong box** (TICKET-119, forge #119) — the "Search tabs",
  commit-message, and top-search inputs are now mutually exclusive and released when you click the terminal,
  so keystrokes always reach the box you clicked (and the terminal, once you click back into it).

- **A command's full output is kept, even beyond the screen** (TICKET-052, forge #52; Terminal Polish —
  closes the last M1.H limitation) — a command whose output exceeded the visible screen (a long log, `cat`
  of a big file) used to keep only the last screenful in its block; the earlier lines were lost. The block
  now captures the full output from alacritty's scrollback history (up to 10000 lines) when the command
  finishes, so scrolling a finished block shows everything.

- **Picking a file while a command runs now reaches the program** (TICKET-071, forge #71; Terminal Polish)
  — choosing a path via ⌘P (Enter) or a file-tree click used to always insert into Marley's prompt buffer;
  while a foreground program was running (an editor, a REPL) the path landed in the invisible cooked buffer
  instead of the program. It now mirrors paste: a running program receives the path over the PTY; at the
  bare prompt it still inserts into the prompt line.

- **Caret no longer floats a space after the typed text** (TICKET-088, forge #88; Terminal Polish) — the
  prompt input row used `.gap_2()`, which inserted an 8px gap between your typed text and the caret bar (a
  "weird space after the last character"). The text + caret + trailing text are now one gapless inner flex,
  so the caret sits flush against your last character; the `❯`/cwd/git segments keep their spacing.

- **cmd-P finder Enter now inserts the path visibly** (TICKET-065, forge #65; sprint M2.C 6/6 — completes
  M2.C) — picking a file in the ⌘P fuzzy finder and pressing Enter used to `write_bytes` the path to the
  PTY, where it reached the shell's line editor but NOT Marley's rendered prompt (invisible in cooked
  mode — the same class as #59's file-click bug). It now inserts into the cooked buffer at the caret
  (`buffer.edit`), so the path appears at the prompt, editable. Also the first-ever visual verification of
  the finder Enter (a #57 gap — its self-test couldn't drive the type-to-filter).

### Changed

- **The agent badge is now LIVE** (TICKET-067, forge #67; sprint M2.C 2/6) — each pump tick recomputes a
  launched agent's status from its pane (`agent_status_from` #61): ● while the agent runs a command, ○
  at the prompt. So the ⌘⇧A badge tracks whether your agent is busy or waiting. Also fixes a #62
  tag-leak (the pump's auto-close of a dead pane now drops its agent tag, like the explicit close does).
  SHIM-only wiring; the status decision stays the tested pure fn.

- **The Forge pane refreshes on open** (TICKET-069, forge #69; sprint M2.C 4/6) — ⌘⇧F now re-fetches the
  current sprint each time it opens (was frozen at startup), so a sprint that changed — or a forge that
  started after Marley — shows up. The fetch runs on a BACKGROUND thread (the forge takes ~seconds/call),
  so the overlay opens instantly with a "loading sprint…" state and fills in when the result arrives —
  the UI never freezes. Honest header/placeholder via a pure `forge_status_line`.

### Added

- **Action confirmation flashes** (TICKET-077, forge #77; sprint M2.D 6/6 — completes M2.D) — every
  cockpit control/write action now shows a brief "it worked" toast that fades on its own (~2s): copy →
  "copied #N", ⌘⇧S → "sent to agent", ⌘⇧G → "broadcast to agents", ⌘-click → "claiming #N…", ⌘⇧-click →
  "comment posted". A pure `flash::Flash` counts down in pump ticks (not wall-clock — `Date::now` is
  unavailable); cov/MSI 100. Verified live: a copy flashed "copied #77", then it faded.

- **Comment on a forge ticket from the cockpit** (TICKET-076, forge #76; sprint M2.D 5/6) — compose a note
  at the prompt, then **shift+cmd**+left-click a ticket row in the ⌘⇧F Forge overlay to post it as a
  comment. Completes the forge-row modifier trio: plain click = copy the ref (#70), cmd = claim (#75),
  shift+cmd = comment (#76). A pure `forge_view::prepare_comment` (trim; never post a blank); the write
  runs off the UI thread and reuses the #74 closed write set (localhost, bearer never logged). Verified
  live: a shift-cmd-click posted the composed line as a comment on the ticket.

- **Claim a forge ticket from the cockpit** (TICKET-075, forge #75; sprint M2.D 4/6) — **cmd**+left-click a
  ticket row in the ⌘⇧F Forge overlay to claim it (a plain click still copies its ref — #70). The first
  *write from the UI*: the claim runs off the UI thread and the pane re-fetches to show the new owner. The
  cmd gate makes it deliberate — a plain click can never accidentally mutate forge. Verified live: a
  cmd-click set the ticket's owner to the Marley identity (only the clicked row changed).

- **The app can write to forge — claim + comment** (TICKET-074, forge #74; sprint M2.D 3/6) — the first
  mutating calls: `ForgeClient::claim_ticket` and `comment_ticket`, a *closed set* of two explicit methods
  (no generic tool surface — the app can never invoke an arbitrary mutating tool). A pure `parse_write_ack`
  is fail-closed: it rejects a JSON-RPC error AND an MCP `isError` result (verified live — forge reports a
  failed write as HTTP 200 + `isError:true`, so a naive check would read failure as success). Localhost +
  bearer-guarded, never logged; the request builders are `pub(crate)`. cov/MSI 100 on the pure surface.

- **Broadcast a line to all agents** (TICKET-073, forge #73; sprint M2.D 2/6) — the fleet counterpart of
  send-to-one: ⌘⇧G sends the composed prompt line to *every* running agent at once ("all of you: run the
  tests"). A pure `agent_view::agent_pane_ids` (the sorted target set); cov/MSI 100. Clears the prompt only
  once it reaches at least one agent (no agents → your line is preserved).

- **Send a line to a running agent** (TICKET-072, forge #72; sprint M2.D 1/6) — the first *control* step:
  compose a command at any prompt and press ⌘⇧S to send it to the last-launched agent (claude), then the
  prompt clears. A pure `marley_agent::send_payload` (line + `\r`, byte-identical to typing + Enter into
  the agent); cov/MSI 100. The compose line is cleared only on confirmed delivery — a closed target never
  drops your input.

- **Click a forge ticket to copy its ref** (TICKET-070, forge #70; sprint M2.C 5/6) — clicking a ticket
  row in the ⌘⇧F Forge overlay copies a pasteable `#N — title` to the clipboard, so you can ⌘V it into an
  agent pane ("go work #70"). Lightly links the work to the agents. A pure `forge_view::ticket_ref`;
  cov/MSI 100. Read-only (no forge writes).

- **Fleet overlay (⌘⇧E) — all agents at a glance** (TICKET-068, forge #68; sprint M2.C 3/6) — the agent
  counterpart of the ⌘⇧F Forge pane: a toggled overlay listing every launched agent with its live status
  (● working / ○ idle / ✓ exited), sorted by pane id. A pure `agent_view::agent_rows` + `agent_status_label`;
  cov/MSI 100. Reads the live agents map, so the fleet reflects #67's live statuses.

- **Agent status badge on its pane** (TICKET-066, forge #66; sprint M2.C 1/6) — the first Living Cockpit
  step: an agent pane (launched via ⌘⇧A) now shows a small corner pill with its label + a status glyph
  (● working / ○ idle / ✓ exited), so you can tell which panes are agents. A pure `agent_view`
  (`agent_status_glyph` + `agent_badge`); cov/MSI 100. The status is Idle for now — #67 drives it live.

- **The Forge pane — cmd-shift-f shows the current sprint** (TICKET-064, forge #64; sprint M2.B 6/6 —
  **completes M2.B "The Agent Cockpit"**) — the visible cockpit surface. cmd-shift-f toggles an overlay
  showing the live sprint + its tickets (each with a status glyph: ✓ done / ◐ in-progress / ○ open),
  read from `marley_forge_client` at startup (best-effort — "forge unreachable" if it's down). A pure
  `forge_view` (`status_glyph` + `sprint_rows`); cov/MSI 100. Marley now shows the work (from forge) +
  the agents (#62) + the terminal in one window. Inspect caught + fixed: cmd-shift-f was shadowed by the
  cmd-F find-bar check; the ticket list is now scoped to the sprint (was the whole project's open-bucket);
  the startup fetch has a socket timeout.

- **`marley_forge_client` — the app→forge read-only seam** (TICKET-063, forge #63; sprint M2.B 5/6) —
  lets the running app read the current sprint + its tickets from the local forge sidecar (the foundation
  for the Forge pane). A new crate over a raw HTTP/1.1 `POST` on `std::net::TcpStream` (no new HTTP
  dependency): parse the endpoint from `.mcp.json`, POST a JSON-RPC `tools/call`, parse the SSE +
  envelope + sprint/ticket JSON into a `SprintView`. **Security: localhost only (a non-loopback URL is
  refused so the bearer can't reach a remote host), READ-ONLY, and the bearer is NEVER logged** (a manual
  `Debug` redacts it, probe-proven). cov/MSI 100 on the pure layer; the live socket is the masked adapter
  (`examples/check_forge.rs` verifies it end-to-end).

- **cmd-shift-a launches an agent pane** (TICKET-062, forge #62; sprint M2.B 4/6) — the first agent-
  cockpit interaction. cmd-shift-a splits a pane, spawns a shell, runs the agent CLI (`claude`) in it,
  and tags the new pane as an `AgentRun` (a `RootView` pane→tag map; a later ticket surfaces + observes
  it). A pure `marley_agent::launch_command(kind)` (the inverse of `agent_kind_of`) picks the program;
  closing a pane drops its tag. cov/MSI 100 on `launch_command` + the keymap; the launch is the masked
  app shim, verified live (cmd-shift-a → the pane split + real claude started + rendered). rerun-last
  stays on cmd-shift-r.

- **`marley_agent` — the agent-run model** (TICKET-061, forge #61; sprint M2.B 3/6) — the pure
  foundation of the agent cockpit. A new gpui-free crate: `agent_kind_of(command)` recognizes an agent
  CLI (`claude`/`codex`) from a command's leading program (path-stripped, args ignored),
  `agent_status_from(exited, active)` projects a pane's flags into an `AgentStatus` (Idle/Working/Exited,
  exited-first), and `AgentRun { kind, label, status }` ties them together. cov/MSI 100. No UI yet — a
  later ticket launches + tags an agent pane with an `AgentRun`.

- **cmd-R command history fuzzy search** (TICKET-060, forge #60; sprint M2.B 2/6) — the conventional
  shell reverse-search + the second consumer of `marley_search_core`. cmd-R opens an overlay that
  fuzzy-ranks the session's past commands (most-recent-first, deduped); Enter inserts the chosen command
  at the prompt (via the cooked buffer, per #59). A pure `CommandHistory::recent()` + a reused
  `FinderState`; cov/MSI 100. **Keymap change:** cmd-R now opens history search; the previous
  "rerun-last" moves to **cmd-shift-r** (the ↻ block-click still re-runs a specific block).

- **Click a file in the tree to insert its path** (TICKET-059, forge #59; sprint M2.B 1/6) — completes
  the Files dock: clicking a FILE row inserts that file's path at the shell prompt (directories still
  toggle collapse). A pure `FileTree::path_at(index)` reconstructs the row's full relative path
  (mirroring the tree traversal), cov/MSI 100. Self-testing caught + fixed a real bug: the insert must
  go through Marley's cooked prompt buffer (`buffer.edit` at the caret), not a raw PTY write — in cooked
  mode Marley owns the line editor, so a raw write reaches the shell but not the rendered line.

- **The Details dock inspects the current command block** (TICKET-058, forge #58; sprint M2.A 6/6 —
  **M2.A COMPLETE**) — the Right "Details" dock (empty since M1.B) shows the active session's most-recent
  command block: the status glyph + command, an `exit N`/`running` line, the working directory, and the
  git branch. A pure `block_details` in `block_status.rs` projects the block's fields (status reusing
  `exit_status_kind`); the render is the coverage-excluded app shim, verified live (drove `echo hi` →
  the dock showed `✓ echo hi`, `exit 0`, the cwd). Inspect caught a glyph/text mismatch (a signal-killed
  block would show ✗ next to "running") — the status line now derives from the status class. cov/MSI 100.
  With this, the M2.A workspace is populated end to end: **Files tree ← terminal → Details inspector.**

- **cmd-P fuzzy file-open** (TICKET-057, forge #57; sprint M2.A 5/6) — the moat's signature quick-open
  and the first real consumer of `marley_search_core`. Bare **cmd-P** (distinct from the command
  palette's cmd-shift-p) opens an overlay that fuzzy-ranks the project's files as you type; Enter
  inserts the chosen path at the shell prompt. A pure `FinderState` (new `marley_app::finder`, mirroring
  `PaletteState`) owns the query + selection and ranks via `fuzzy_rank`; the overlay + key routing are
  the coverage-excluded app shim. cov/MSI 100 on the model; verified live via the self-test harness
  (cmd-P opens the ranked, highlighted list). Also added a `cmd:` chord action to `drive.swift`.

- **The Files dock shows the project file tree** (TICKET-056, forge #56; sprint M2.A 4/6) — the Left
  "Files" dock (empty since M1.B) now renders the project's files as an indented, collapsible tree. A
  pure `FileTree` in `marley_project` (`from_files` nests + merges + orders dirs-before-files, `visible_rows`
  is a collapse-aware flattening, `toggle` flips a directory's collapse) is built once at startup from
  the discovered project; the dock renders each row with a `▾`/`▸` disclosure and click-to-collapse.
  cov/MSI 100 on the model; the render/click is the coverage-excluded app shim, verified live via the
  self-test harness (captured the tree + drove a click-collapse). Also added a `clickat:` action to
  `scripts/selftest/drive.swift` and fixed `bundle-app.sh` to always rebuild.

- **`marley_project` file listing** (TICKET-055, forge #55; sprint M2.A 3/6) — `list_files_in(root)`
  walks a project tree and returns every non-skipped regular file (relative to the root, sorted),
  plus `should_skip` (a curated `.git`/`node_modules`/`target`/`.DS_Store` skip-set) and a
  `FileListing { files, truncated }` result. The walk prunes skipped directories, ignores symlinks so
  it can never loop, skips unreadable directories without panicking, and is bounded at `MAX_FILES`
  (the `truncated` flag signals "showing the first N of many" — no silent cap). Feeds the Files-dock
  tree (#56) and fuzzy file-open (#57). cov/MSI 100.

- **`marley_search_core` — the shared fuzzy matcher** (TICKET-054, forge #54; sprint M2.A 2/6) — a new
  crate wrapping the `nucleo` subsequence matcher behind `fuzzy_score(text, query) -> Option<u32>` and
  `fuzzy_rank(candidates, query) -> Vec<Scored>` (empty query → input order; else matches by score
  desc, index tie-break). The command palette's local `field_score` was extracted into it and the
  palette refactored on top — **one tested matcher**, `nucleo` moved from `marley_app` to the new
  crate. Palette behavior is unchanged (its full suite still passes). cov/MSI 100. First consumer:
  fuzzy file-open (#57).

- **`marley_project` — project discovery (the M2 moat foundation)** (TICKET-053, forge #53; sprint
  M2.A "Project, Files & Search" 1/6) — a new pure, gpui-free crate that makes Marley aware of a
  *project* (a repo), not just a cwd. `Project { root, name, is_git }` + `Project::discover_in(start)`
  walks up for the NEAREST `.git` ancestor — a directory OR a file, so git worktrees and submodules
  are recognized — and falls back to `start` (not a repo) when there is none. The `Project` becomes
  the context the rest of M2.A builds on (search #54, file listing #55, the file tree #56, the docks).
  cov/MSI 100. No app wiring yet (a later step once it has consumers).

### Fixed

- **Command blocks stack as scrollback (history no longer clears)** (TICKET-050, forge #50; sprint
  M1.H "Real Terminal Feel" 2/…) — "every command clears the previous" (chad, live testing). Each
  block captured the full screen-height render grid *including the trailing blank rows*, so a one-line
  command like `pwd` became a full-screen-tall block that filled the viewport and pushed the previous
  command off-screen. A block's captured output now trims its trailing blank rows (a new pure
  `trim_trailing_blank_rows` in `terminal_blocks`, cov/MSI 100), so each block is only as tall as its
  real output and blocks accumulate as scrollback like a normal terminal. The alt-screen grid
  (vim/top/less) keeps the full grid. SPEC-terminal-blocks R19 amended.

- **Terminal is bottom-anchored like a normal terminal** (TICKET-049, forge #49; sprint M1.H "Real
  Terminal Feel" 1/…) — the pane top-packed its content, so short output + the prompt sat at the TOP
  of the pane with empty space below (chad, live testing: "not acting like a normal terminal where you
  can see the previous output"). The pane content column is now `justify_end` (bottom-anchored): the
  prompt sits at the bottom, output right above it, empty space at the TOP — like Terminal/iTerm/Warp.
  Also removes a latent prompt-clipping risk (`overflow_hidden` now clips the oldest top row, never the
  prompt). Shim-only layout fix; verified by window capture. SPEC-app-shell R39 amended.

### Added

- **Clear (cmd-K) + jump-to-block nav (⌘↑/⌘↓)** (TICKET-048, forge #48; sprint M1.G "Block Workflows
  & Selection" 6/6, FINALE) — ⌘↑/⌘↓ jump the viewport between command-block boundaries (to the
  previous/next command, not line-by-line), and cmd-K clears the focused shell. Backed by a pure
  `nav.rs` (cov/MSI 100) — `block_boundary_rows(line_counts)` (each block's header content-row) +
  `jump_target(boundaries, top, forward)` (the nearest boundary above/below, no wrap at the ends). The
  keymap bindings + the jump-scroll (reusing #47's `scroll_focused_to_row`) + the `\x0c` clear are the
  app-shell shim. SPEC-app-shell gains R52. **Closes M1.G "Block Workflows & Selection".**

- **Find in scrollback (cmd-F)** (TICKET-047, forge #47; sprint M1.G "Block Workflows & Selection"
  5/6) — cmd-F opens a find bar over the focused pane's block output: typing highlights the rows
  containing a match, Enter/Shift-Enter cycle through the matches and scroll to the current one, Esc
  closes. Backed by a pure `find.rs` (cov/MSI 100) — `find_matches(haystack, query)` (all
  non-overlapping, case-insensitive matches; `"aa"` in `"aaa"` → one match; an empty query matches
  nothing) + `match_navigation(len, current, forward)` (wrap-around next/prev). The find-bar overlay,
  the highlight paint, and the scroll-to are the app-shell shim. SPEC-app-shell gains R51.

- **Re-run a Block's command (click / cmd-R)** (TICKET-046, forge #46; sprint M1.G "Block Workflows &
  Selection" 4/6) — a ↻ affordance on the block header (and cmd-R for the most recent finished block)
  resends a previous command to the shell without retyping. Backed by a pure `Block::rerun_command()`
  in `terminal_blocks` (cov/MSI 100) — `Some(command)` iff the block is Finished with a non-empty
  command, else `None` (a still-running or empty block is not re-runnable). The affordance + the
  resend (`write_command`, guarded by #40's `is_command_running` so it never injects mid-command) are
  the app-shell shim (cmd-R resolves the most recent finished command via a pure
  `BlockList::last_rerunnable`). SPEC-terminal-blocks gains R30, SPEC-app-shell R50.

- **Block actions: copy command / copy output** (TICKET-045, forge #45; sprint M1.G "Block Workflows
  & Selection" 3/6) — Warp's signature block interaction: hovering a command block reveals
  copy-command and copy-output affordances at the header's right edge; clicking one copies that text
  to the clipboard (no manual selection). Backed by a pure `Block::copy_text(BlockCopy)` in
  `terminal_blocks` (cov/MSI 100) — `Command` returns the command line, `Output` returns `output_text`
  (so a block-action copy agrees with a drag-copy of the same output). The hover affordances + the
  click (which looks the block up by pane+index at click-time and reuses #44's clipboard write) are
  the app-shell shim. SPEC-terminal-blocks gains R29, SPEC-app-shell R49.

- **Copy the selection (cmd-C)** (TICKET-044, forge #44; sprint M1.G "Block Workflows & Selection"
  2/6) — completes copy/paste (#42 gave paste): cmd-C now copies the #43 text selection to the
  clipboard. A new pure `selected_text(rows, sel)` builds on #43's `row_selection` (so the copied text
  matches the drag-highlight exactly) + `copy_payload(rows, selection)` (cov/MSI 100) which returns
  `None` for an absent or empty selection — cmd-C with nothing selected is a no-op, never an empty
  clipboard write. The keystroke handling + the clipboard write are the app-shell shim (cmd-C is a
  platform chord, distinct from the Ctrl-C that streams to the PTY). SPEC-app-shell gains R48.

- **Terminal text selection** (TICKET-043, forge #43; sprint M1.G "Block Workflows & Selection" 1/6)
  — there was no way to select terminal output (paste shipped in #42, but copy had nothing to copy).
  Drag now selects text: a new pure `text_selection` module (cov/MSI 100) models `Selection{anchor,
  head}` over content `GridPos{row, col}` (char columns) with `normalized()` (a backward drag reads
  forward) and `row_selection(sel, row, row_len)` — the per-row selected char span the highlight
  render paints. `PaneState` gains the per-pane `selection`; a mouse-down starts it, a drag extends
  it, a new command clears it. The FOUNDATION for copy (#44). SPEC-app-shell gains R47.

- **Clipboard paste + bracketed-paste mode** (TICKET-042, forge #42; sprint M1.F "Real Interactivity"
  3/3 — the finale) — the app had no paste at all; cmd-V now pastes the clipboard. When a program has
  bracketed paste on (DECSET 2004 — editors, REPLs, Claude Code), the paste is wrapped in
  `ESC[200~`…`ESC[201~` so a multi-line paste arrives as literal DATA instead of each line executing —
  and any embedded `ESC[201~` end-marker is stripped first (a paste-injection guard: a pasted marker
  can't close the bracket early and run the tail as commands). Backed by a pure gpui-free
  `paste_bytes(text, bracketed)` + `TerminalSession::is_bracketed_paste()` (cov/MSI 100); the cmd-V
  handler routes via #40 (to the running program, or the local prompt buffer). Closes M1.F.
  SPEC-terminal-blocks R28 + SPEC-app-shell R46.

- **Full interactive key coverage** (TICKET-041, forge #41; sprint M1.F "Real Interactivity" 2/3) —
  with #40 routing input to running programs, the pure gpui-free `encode_key` now emits the keys
  interactive menus + editors need beyond the basics: **Shift-Tab** (`BackTab` → `ESC[Z` — backward
  nav in menus / Claude Code), **F1–F12** (SS3 for F1–4, CSI for F5–12), **Insert**, and **modified
  cursor keys** — Shift/Alt/Ctrl+arrow (and Home/End) now emit the xterm `ESC[1;<param><final>` form
  via a new `modifier_param(shift, alt, ctrl) = 1 + Shift + 2·Alt + 4·Ctrl` (word-jump + selection in
  inline editors). `KeyInput` gains a `shift` field; all new encoder arms + the modifier arithmetic
  are cov/MSI 100 (golden byte vectors); the gpui key-name mapping is the app-shell shim.
  SPEC-terminal-blocks R25 extended.

### Fixed

- **Interactive prompts on the primary screen now receive keystrokes** (TICKET-040, forge #40; sprint
  M1.F "Real Interactivity" 1/3) — arrow-key menus and inline interactive programs (Claude Code's
  approval/selection prompts, `read`, `npm init`, interactive git) didn't get arrow keys: input only
  streamed to the PTY on the ALTERNATE screen (vim/top, #33), so a primary-screen interactive program
  fell through to Marley's local history recall. Input now routes by whether a foreground command is
  running — `input_route(alt_screen, ctrl, command_running)` streams `Raw` if any hold, and a new
  gpui-free `TerminalSession::is_command_running()` (a `Running` block exists, cov/MSI 100) supplies
  the signal. While a command runs, every key reaches the program; at the bare prompt the local line
  editor + history are unchanged. SPEC-terminal-blocks R26 extended.

### Added

- **Typography scale + focus/hover polish** (TICKET-039, forge #39; sprint M1.E "The Warp Look" 6/6 —
  the FINALE) — the cohesion pass. A new gpui-free `typography` module (cov/MSI 100) formalizes the
  ad-hoc `FontWeight`/`text_sm` scattered in the render into one tested scale: `type_scale(Role)`
  (`Command→(14, Medium)`, `Output→(14, Normal)`, `Caption→(12, Normal)`) + `weight_value(TextWeight)`
  (`Normal→400 / Medium→500 / Bold→700`, wrapped in `gpui::FontWeight` by the shim). Added a `muted`
  caption color to `ThemeColors` (dark+light) — the secondary/metadata tone. Applied: the Block
  command header now uses the `Command` role + a hover tint; the dock header uses the `Caption` role
  in `muted`. Closes M1.E — `cargo run -p marley` reads as a finished Warp-style terminal.
  SPEC-ui-components (`muted`) + SPEC-app-shell (R45) updated.

- **Dock panels + titlebar cohesion** (TICKET-038, forge #38; sprint M1.E "The Warp Look" 5/6) — the
  left/right docks rendered as bare `surface` divs with a hardcoded "Files"/"Details" label. They're
  now real PANELS: a header (the title with a divider beneath), a divider border on the dock's inner
  edge toward the center, a padded content area (M2 fills it), over the `surface` elevation — and the
  root already paints the theme `background`, so docks/center/native-"Marley"-titlebar read as one
  cohesive Warp-style surface. The label is extracted to a pure `dock_title(DockSide)` (cov/MSI 100);
  the panel layout is the app-shell shim. Icons + real dock content deferred to M2. SPEC-app-shell
  gains R44.

- **Prompt + input row chrome** (TICKET-037, forge #37; sprint M1.E "The Warp Look" 4/6) — the prompt
  was a bare `{before}▏{after}` row. It's now a Warp-style input ROW: a `surface` strip with a `❯`
  accent marker, the cwd/git context (`~/marley (main)`-style) from the live shell prompt, then the
  editable buffer with a styled accent caret bar. Backed by a new pure surface (cov/MSI 100):
  `TerminalSession::current_prompt()` in gpui-free terminal_blocks exposes the precmd-staged context
  (the cwd/git the next command will run in), and marley_app's `prompt` module decides the segments —
  `prompt_segments(info)` (a `Cwd` segment iff `pwd` is set + a `Git` segment iff `git_branch` is set,
  ordered) + `pwd_label(pwd)` (the last path component, so the cwd stays compact). First cut is
  pwd + git_branch (virtual_env/node_version deferred); the input-row layout is the app-shell shim.
  SPEC-app-shell gains R43. (The full shell-ZLE prompt richness remains the deferred #33 intake.)

- **Block chrome — command header + exit-status indicator** (TICKET-036, forge #36; sprint M1.E "The
  Warp Look" 3/6) — command Blocks rendered as bare stacked text with no status. Each Block now gets
  a styled command HEADER row: an exit-status indicator (a distinct glyph AND color — green `✓`
  success / red `✕` failure / neutral `○` running, so it reads even in grayscale) followed by the
  command, with a separator above the block. Driven by a new pure `block_status` module (cov/MSI
  100): `exit_status_kind(state, exit)` (state checked first — `Pending`/`Running` → `Running`;
  `Finished`+`Some(0)` → `Success`; `Finished`+`Some(nonzero)`|`None` → `Failure`) +
  `status_indicator(kind, &ThemeColors)` (using the #35 `success`/`danger`/`border` roles). The card
  layout is the app-shell shim; the classification is the tested core. Warp's signature Block look.
  SPEC-app-shell gains R42.

- **Warp-matched theme palette** (TICKET-035, forge #35; sprint M1.E "The Warp Look" 2/6) — the
  `ThemeColors::default_for` palette was placeholder grayscale (flat bg/fg/surface/border + a stock
  blue accent). Replaced dark + light with an ORIGINAL Warp-matched palette (clean-room — the
  observable aesthetic derived into original `hsla`, not lifted): a deep cool near-black dark
  (tinted, not flat gray), a warm off-white light, a distinctive Marley teal-cyan accent, tuned
  surface/border elevation, and a warm `danger` red. Added a `success` green field to `ThemeColors`
  (the positive/OK semantic the #36 exit-status indicator needs). The whole cockpit re-skins
  automatically — everything already renders from the token bundle; the metric tokens
  (corner_radius/control_height/control_padding) are unchanged. Pure data + the tested `default_for`
  accessor. SPEC-ui-components updated.

- **Monospace terminal font + accurate cell metric** (TICKET-034, forge #34; sprint M1.E "The Warp
  Look" 1/6) — terminal text rendered in the ambient (proportional) font, so columns didn't align
  and the #30 cell metric was measured against the wrong glyphs. Now the Block command/output, the
  prompt, and the alt-screen grid render in a monospace font (`Menlo`, the macOS system mono —
  clean-room), and the pane cell metric is read from THAT font: the width from its em-advance, the
  height from a pure `fallback_cell(font_size)` (cov 100/MSI 100 — `0.6·size` advance / `1.2·size`
  height, a `> 0.0` guard defaulting to 14.0) which the terminal rows also render as their
  line-height, so the metric (feeding #30 resize / #32 scrollback / #33 grid) and the drawn glyphs
  agree. Closes the `terminal-monospace-font-and-cell-metric` intake. SPEC-app-shell gains R41.

- **Interactive/raw mode — vim, top, less, Ctrl-C** (TICKET-033, forge #33; sprint M1.D "The Daily
  Driver" 6/6, the flagship) — input was line-buffered, so no full-screen programs and no Ctrl-C.
  Now `marley_terminal` gains a pure gpui-free `keys` module (cov 100/MSI 100): `encode_key` maps a
  `KeyInput` to PTY bytes (printable UTF-8, ESC-prefixed under alt; `Ctrl+key` → the C0 byte via
  `ctrl_byte` = `(c as u8) & 0x1f`; Enter/Backspace/Tab/Escape + the arrow/Home/End/Page/Delete CSI
  sequences), and `input_route(alt_screen, ctrl)` decides Raw vs Cooked. `TerminalSession` exposes
  `is_alt_screen` (DECSET-1049 detection via `term.mode()`) + `grid_styled_rows` (the live grid,
  reusing #31's styled rows). The app.rs shim streams every keystroke to the PTY while the alternate
  screen is active — rendering the live colored grid instead of Blocks — and routes Ctrl-C/D/Z to
  the PTY in cooked mode, so **vim/top/less/htop and Ctrl-C now work**. SPEC-terminal-blocks gains
  R25/R26, SPEC-app-shell R40. Tab-completion at the bare prompt is DEFERRED — it needs the
  local-editing-vs-shell-ZLE model decision (intake `prompt-shell-line-editing-model.md`).

- **`marley_app` scrollback viewport** (TICKET-032, forge #32; sprint M1.D "The Daily Driver" 5/6)
  — output past the pane fold was clipped with no way to scroll up. A NEW pure `Viewport` (cov
  100/MSI 100) tracks a following-vs-held scroll state + the visible-window math: `visible(content,
  capacity)` returns the `[start, end)` row slice (the bottom `capacity` rows while following, a
  held `top`-window otherwise), `scroll_up` (materialises `top` at the current bottom then clamps
  at row 0), `scroll_down` (re-anchors to following at the bottom). New output stays in view while
  following; while scrolled up the same window holds as content grows, so a running command doesn't
  yank the view. Held on `PaneState` (per-pane). The app.rs shim counts content rows, reuses the
  #30 `pty_size` rows as capacity, slices the rendered rows to the window, and wires the scroll
  wheel + PageUp/PageDown + Shift+Arrows; running a command re-anchors. A `Workspace::state_mut`
  accessor lets the wheel target a specific pane. SPEC-app-shell gains R39. Builds on #30 (metric) +
  #31 (styled rows).

- **ANSI color** (TICKET-031, forge #31; sprint M1.D "The Daily Driver" 4/6) — output was monochrome
  (`term_to_rows` flattened the alacritty grid to `Vec<String>`, discarding per-cell color). Now
  color is preserved end to end. `marley_terminal` (kept gpui-free) gains a styled model: `StyledRun`
  carries alacritty's own `Color`/`Flags`, `coalesce_row` (cov 100/MSI 100) merges adjacent
  same-style cells into runs and trailing-trims so `Block::output_text()` stays **byte-identical**
  (all 13 callers + both integration suites unchanged), `term_to_styled_rows` replaces
  `term_to_rows`, and `Block` exposes `output_styled()`. `marley_app` gains `color.rs`: an
  `AnsiPalette` (an ORIGINAL 16-color table + the theme's fg/bg) and `ansi_color_to_hsla` (cov
  100/MSI 100 — Named table / Indexed 6×6×6 cube + grayscale ramp / Spec truecolor passthrough) +
  `run_paint` (INVERSE swaps fg/bg, BOLD → weight); the render paints one colored span per run.
  SPEC-terminal-blocks gains R20b, SPEC-app-shell R38. The biggest visual upgrade — output finally
  looks like a terminal. (Underline/italic/dim + bg-fill + the monospace-font metric fidelity are
  tracked follow-ups.)

### Removed

- `marley_app::terminal_view` (`block_row`/`BlockRow`) — superseded by the styled per-run render
  (TICKET-031); the plain-text path is now `Block::output_text` + `output_styled`.

- **`marley_app` PTY resize-to-pane** (TICKET-030, forge #30; sprint M1.D "The Daily Driver" 3/6) —
  every pane's PTY was a fixed 80×24, so output wrapped at 80 columns in a wide pane and a window
  resize never reflowed. Now each pane sizes its PTY to its rect: a pure `plan_resize(rect, cell,
  current) -> Option<(cols, rows)>` (cov 100/MSI 100) floor-divides the pane `Rect` by the
  monospace `CellSize` per axis, clamps each to ≥1 (guarding a zero/negative/NaN cell), and returns
  `Some` only when the grid changed — folding the unchanged-guard into the result so a resize fires
  exactly on a grid change, not every frame. The high end saturates on the `u16` cast and NaN is
  absorbed by `f32::max`, so the only guard is `cell > 0.0` (no inert clamp). The app.rs shim reads
  the gpui cell metrics (`em_advance` × `line_height`), computes each pane's rect via `pane_rects`,
  and calls `TerminalSession::resize` (R17) on changed panes; `PaneState.pty_size` tracks the last
  applied grid (init `(0,0)` sentinel). Settings `terminal.cols/rows` stay the initial spawn size.
  SPEC-app-shell gains R37; closes the #23/#26 "PTY stays 80×24" deferral.

- **`marley_app` command history** (TICKET-029, forge #29; sprint M1.D "The Daily Driver" 2/6) —
  ↑/↓ at the prompt recall previous commands like a shell (they did nothing before). A NEW pure
  `CommandHistory` (cov 100/MSI 100) — a bounded per-pane ring: `record` (dedup-last + capacity
  eviction, resets navigation), `recall_prev` (stash the in-progress line on the first ↑, walk
  toward older, clamp at the oldest), `recall_next` (walk newer, restore the stashed draft past
  the newest), `detach` (an edit leaves navigation). Lives on `PaneState` (per-pane, travels with the pane on
  split/close). The app.rs shim routes ↑/↓ ONLY when the command palette is closed (the palette
  owns them while open), records each submitted command, and detaches on an edit key (Char/
  Backspace/DeleteForward — not a caret motion). A `with_capacity` seam makes the eviction bound
  cheaply testable. SPEC-app-shell gains R36. Builds on #28's editable line.

- **`marley_app` in-line prompt editing** (TICKET-028, forge #28; sprint M1.D "The Daily Driver"
  1/6) — the input prompt was append + backspace only (arrows did nothing); now it's a real
  editable line. `Key`/`apply_key` gain `Left`/`Right`/`WordLeft`/`WordRight`/`Home`/`End`/
  `DeleteForward`, each motion DELEGATING to the already-shipped, already-tested
  `marley_editor::movement` fns (`move_char/word/line_*`, which clamp internally) — so the pure
  surface is just the right-fn-per-key wiring + `DeleteForward`'s guard, not re-tested motion.
  Insert/backspace already worked at an interior caret (now pinned by tests). A pure
  `split_at_caret` (multibyte-safe char split) fixes the render so the caret glyph paints AT its
  position instead of always at the line start. `key_from_keystroke` maps ←/→ (alt → word), home/
  end, and macOS forward-delete. SPEC-app-shell gains R35. The enabler for #29 history + #33 raw
  mode.

- **`marley_app` settings wired — the theme + dock state survive relaunch** (TICKET-026,
  forge #26; sprint M1.C "The Wired Cockpit" 5/6 — THE CLOSER) — `marley_app` finally DEPENDS on
  `marley_settings` (#21 shipped the framework with zero consumers). A NEW pure `settings` module
  (cov 100/MSI 100): the schema (`appearance.theme` / `docks.left` / `docks.right` /
  `terminal.cols` / `terminal.rows` via `define_setting!`) + `applied_from` (reads a loaded
  manager, VALIDATES the saved theme name against the registry — unknown/absent → the Dark
  default) + `applied_defaults` (the boot-default `AppliedSettings`, its non-theme fields drawn
  from the `Setting::default_value()`s so the file-missing and load-error paths can't drift) + the
  `*_in(dir)` IO seam (`settings_file_in` / `load_manager_in` — create-dir then load
  `<dir>/settings.toml`; a missing file yields defaults and is NOT created; invalid TOML surfaces
  an error) + `persist_theme` / `persist_dock_*`. `RootView` boots from `marley_config_dir()`
  (R33 — applies the saved theme + dock states + terminal size; a missing/invalid file boots
  defaults, never fails; a corrupt file is never clobbered) and PERSISTS the theme + dock states
  on change (R34 — a persist failure never crashes the terminal). Terminal size is boot-applied
  from the file (no runtime resize UI yet). Config-dir isolation via the `_in(dir)` fns (tests
  pass a tempdir; no live `~/.marley`, §14). SPEC-app-shell gains R33/R34. marley_app gains
  `marley_settings` + `marley_core` deps (its first use of either). **The M1.C cockpit is now
  wired end to end: split panes, toggle docks, switch theme — and the theme + docks come back on
  relaunch.**

- **`marley_app` palette dispatch** (TICKET-025, forge #25; sprint M1.C "The Wired Cockpit"
  4/6) — Enter finally RUNS the command (R17): a NEW pure `PaletteState` (owns the query;
  ↑/↓ clamp at both ends over the current filtered list; every edit resets the selection to the
  top — which is also the shrink-rebound mechanism; `activate` → the SELECTED command via `.get`,
  `None` on an empty list) + the pure `action_for_command` CommandId→verb table routed through
  the ONE `dispatch_action` router the chords use (split-pane / close-pane / toggle-left-dock /
  toggle-right-dock / **toggle-theme — the first runtime theme switch**, flipping dark↔light via
  the pure `toggled_appearance` + `ThemeRegistry::default_for`; the dispatch caller notifies, the
  same cx-less pattern as `toggle_dock_state`). The overlay renders the selected row highlighted
  (accent/on_accent) with per-row chord chips (`KeyBinding::display` emits the `+`-separated form
  `marley_ui_components::KeyboardShortcut::parse` tokenizes — the #17 widgets' first real
  consumer). A consistency test pins every listed command to a real dispatch verb.
  "Open Settings" (CommandId(3)) is REMOVED — an inert command was a lie; ids stay stable and
  the id maps to `None` (tested). SPEC-app-shell gains R32.

- **`marley_app` docks for real** (TICKET-024, forge #24; sprint M1.C "The Wired Cockpit" 3/6) —
  the shell body finally renders R4–R6's three regions: left dock ("Files") | center pane
  tiling | right dock ("Details"), each dock a themed placeholder panel (real content = the M2
  panel system). A NEW pure `layout::region_widths` (R31, cov 100/MSI 100): Open dock → the dock
  width, Closed → exactly zero, center = the remainder CLAMPED AT ZERO (inspect-probed: an
  unclamped remainder let a window narrower than its open docks paint the right dock OVER the
  left and reverse the pane tiling — reachable by ordinary resize below 440pt) — and the render
  sizes the dock panels AND insets the #23 `pane_rects` center from that ONE computation (no
  independent inset math). `Keymap::default_bindings` gains `cmd-b → toggle-left-dock` and
  `cmd-shift-b → toggle-right-dock` (R19 amended); `dispatch_action` wires both to the dock
  flip; the palette lists "Toggle Left Dock"/"Toggle Right Dock" with their chords registered on
  the `Command` (chip render + Enter-dispatch = seq-4).
  `toggle_dock` (R6) now backs onto a shared `toggle_dock_state` used by both the public API and
  the chord path.

- **`marley_app` panes for real** (TICKET-023, forge #23; sprint M1.C "The Wired Cockpit" 2/6) —
  the center region now renders the `PaneGroup` tree for REAL, replacing the M1.B
  `⬜ panes:N docks:LR` status-line placeholder. A NEW pure `workspace` module (cov 100/MSI 100):
  `pane_rects` (depth-first rect tiling — axis partition × ratios, origin accumulation; no
  remainder arm, the binary equal-ratio tree tiles exactly in f32 [D-2.2]) + `Workspace<S>` (a
  per-pane state registry {session, input `Buffer`, caret} GENERIC over the session handle behind
  a spawn-per-call seam — the `PtyChannel` mock precedent) + the focused-pane model
  (`split_focused` spawns FIRST so a failure leaves the workspace untouched, then focuses the new
  pane; `close_focused` drops the pane's state — killing its PTY — and focuses the depth-first
  predecessor [successor when first]; `focus` rejects absent panes; registry keys ==
  `group.panes()` after every op). The app.rs shim renders one absolutely-positioned div per rect
  (accent border = the focus affordance, click-to-focus), pumps EVERY session per frame, and
  routes keys/submits to the FOCUSED pane — cmd-d/cmd-w now act on focus instead of
  `panes().first()`. `marley_visual_harness` gains `send_keystroke` (osascript System Events, the
  existing os_shim exclude class — input restricted to a single ASCII-alphanumeric key so a quote
  can never break out of the generated AppleScript) so the headed lane can drive a real cmd-d
  split. SPEC-app-shell gains R27–R30. `PaneId` gains `Hash`. **Inspect-driven hardening:**
  (1) `marley_terminal::pump` gains an IDLE FAST-PATH — a leading `WouldBlock` (nothing read this
  call) returns immediately instead of sleeping through the retry budget; the ~1 ms retry windows
  now apply only mid-burst. Measured: 8 idle panes cost 98 ms per 16 ms tick before, **19.7 µs**
  after (~5000×) — the retry sleeps were the multi-pane frame killer. (2) `Workspace::close`
  (generalizing `close_focused`) RETURNS the closed pane's state; the app drops it on a reaper
  thread (`PtyChannel` gains a `Send` supertrait) — a session drop blocks in the child reap
  (603 ms measured with a running foreground child; unbounded for a HUP-immune one) and must
  never freeze the main thread. (3) A pane whose shell exits (`ChildExited`/disconnect) is
  auto-closed by the pump loop unless it is the last pane (a visibly-dead last shell beats a
  vanishing window). (4) The palette overlay now `.occlude()`s — clicks on the open palette no
  longer silently refocus the pane beneath it.

- **`marley_settings`** (TICKET-021, forge #21; sprint M1.B "The Cockpit" 5/5 — THE FINALE) — Marley's
  typed declarative TOML settings framework: `SettingsValue` / `Setting` traits + a `SettingsManager`
  (`load` / `register` / `get` [lazy] / `set` / `clear` / `reload` / `default_values` / `subscribe` /
  `is_syncable`) over a retained `toml::Table` working tree + `define_settings_group!` / `define_setting!`
  macros (a `const` assert guards a non-private setting's `toml_path`) + `ChangeEvent` / `SettingsError` +
  `SecondsDuration` (persisted as whole seconds). The whole crate is PURE — **cov 100 / MSI 100** (no
  exclude); reuses `toml` + `serde`. **Completes the M1.B "The Cockpit" sprint (#17–#21).**
- **`marley_app` workspace layout** (TICKET-020, forge #20; sprint M1.B "The Cockpit" 4/5) — the pure
  `PaneGroup` tree algebra: `single` (R7), `split` (replace a leaf with a 2-child Split; Before/After +
  axis, R8), `close` (collapse a 2-child Split to its survivor; `LastPane`/`PaneNotFound` guards,
  R9/R10/R13), `panes` (depth-first, R7), `neighbor` (the R12 boundary-adjacency rule via a recursive
  search, incl a nested 2×2), and equal-ratio renorm (R11); plus `DockSide`/`DockState` + `toggled`.
  `RootView` gains the 3-region docks (`dock`/`toggle_dock`/`pane_group`) + cmd-d/cmd-w split/close of the
  center pane-group. `layout.rs` is cov 100 / MSI 100; the render + reflow + wiring are the app.rs shim
  (ACCEPTED-UNTESTABLE). A frozen M2-facing contract.
- **`marley_app` command palette** (TICKET-019, forge #19; sprint M1.B "The Cockpit" 3/5) — a pure
  `filter_commands` (nucleo subsequence ranking over a static `&[Command]`: empty query → all in
  registration order; non-empty → ranked, non-matches excluded, MAX field score, descending, ties by
  registration order) + the `Command` / `CommandId` / `ScoredCommand` model. `RootView` gains a centered
  overlay: `cmd-shift-p` (the keymap) opens it, typing filters live, Enter activates the top result +
  dismisses, Escape dismisses. `palette.rs` is cov 100 / MSI 100; the overlay state + render are the
  app.rs shim (ACCEPTED-UNTESTABLE). Adds the `nucleo` dep; the M1 static-list wrapper migrates onto
  `marley_search_core` in M2.
- **`marley_app` keymap layer** (TICKET-018, forge #18; sprint M1.B "The Cockpit" 2/5) — a pure
  `KeyBinding` (modifiers + base key) + `Keymap` (chord→action-name) with `default_bindings()`
  (cmd-shift-p→"open-command-palette", cmd-d→"split-pane", cmd-w→"close-pane") + `action_for()` (`None`
  for an unbound chord). `RootView` now intercepts bound cockpit chords in `on_key_down` (kept off the
  terminal prompt); the action handlers land in #19 (palette) + #20 (panes). `keymap.rs` is cov 100 /
  MSI 100; the gpui `Keystroke`→`KeyBinding` translation is ACCEPTED-UNTESTABLE (the app.rs shim). A
  frozen M2-facing contract.
- **`marley_ui_components` widgets** (TICKET-017, forge #17; sprint M1.B "The Cockpit" 1/5) — the 5
  reusable cockpit UI primitives, added to the existing crate: **button** (`visual_state` / `label_text` /
  `content_order` / `button_colors`), **switch** (`knob_position` / `track_color`), **dialog**
  (`content_summary` / `outcome_for`), **tooltip** (`is_visible` / `poll`), and a **keyboard-shortcut
  chip** (`parse` / `keys`), plus a Marley-owned `IconName`. Each is a long-lived struct holding persistent
  hover/press/visibility latches (host-driven via `set_hover`/`set_press`/`poll`), rendering from a
  borrowed `&ThemeColors`. The pure decision accessors are **cov 100 / MSI 100**; the hand-rolled raw-gpui
  render (`render/`) + the widget-gallery fixture bin are ACCEPTED-UNTESTABLE (`mutants::skip` +
  rust_cov-excluded), asserted by a `#[ignore]` headed widget-gallery visual test. Hand-rolled on raw gpui
  (decision B — `gpui-component` resolves with gpui 0.2.2 but drags in 32 non-optional deps for 5 trivial
  widgets). The UI primitives the M1.B command palette (#19) + settings (#21) render from.
- **`marley_app`** (`marley`; TICKET-012, forge #16; sprint M1.A 5/5 — THE FINALE) — the first runnable
  Marley: `cargo run -p marley` boots one gpui "Marley" window mounting a live `marley_terminal` session +
  the `marley_editor` input; type a command, Enter runs it in a real `zsh`, and its output segments into
  per-command **Blocks** via a bundled Marley-original shell-integration rc (emitting Marley's DCS hook
  frames, wired through `ZDOTDIR`) — painted in the Dark theme from `marley_ui_components`. The minimal cut
  of SPEC-app-shell (docks, pane-group algebra, command palette, keymap, multi-theme → M1.B). The pure
  wiring (`themes`/`input`/`terminal_view`/`shell_integration`) is **cov 100 / MSI 100**; the gpui
  `RootView`/`run()`/pump-timer shim (`app.rs`) is ACCEPTED-UNTESTABLE (mutants::skip + rust_cov-excluded),
  asserted by a headed visual test. A headless real-zsh `#[serial]` integration test proves `echo hi` → a
  Finished Block. Crate layout: dir `crates/marley_app` (so gate-15 asserts the component), package
  `marley`, lib `marley_app`, bin `marley`. Documented in `docs/marley_architecture/app_shell.md`.
  **Completes the M1.A "Usable Terminal" sprint (#12–#16).**
- **`marley_editor`** (`editor`; TICKET-011, forge #15; sprint M1.A 4/5) — the M1.A input-prompt subset
  of SPEC-editor: a **UI-agnostic**, `ropey`-backed text-editing `Buffer` over the `marley_text_offsets`
  `CharOffset`/`ByteOffset` seam. Range `edit`s carry an `EditOrigin` (`Human` vs `Agent`) returned on
  `EditResult{delta,version,origin}` — the observable write-provenance seam for the brain. Single
  cursor/selection (1-member `SelectionSet`), char/word/line movement (hand-rolled word-class), `point_at`
  (row + UTF-8 byte column), random-access `char_to_byte`/`byte_to_char`, and submit read-back. All PURE —
  **cov 100% (357/357) / MSI 100% (76 viable, 0 missed)**; the themed render is `app_shell` (#16). Deps
  `ropey` (MIT) + `marley_text_offsets`. The full editor (anchors, multi-cursor, grouped undo/redo, find,
  diff, layout) grows in M1.B. Documented in `docs/marley_architecture/editor.md`.
- **`marley_terminal`** (`terminal_blocks`; TICKET-010, forge #14; sprint M1.A 3/5) — the M1.A product
  core: the **UI-agnostic** PTY-backed shell session + per-command **Block model**. Spawns a shell,
  writes commands, reads output back, and segments it into Blocks from shell DCS/OSC hooks (not output
  heuristics, R8), producing a `BlockList` any front-end observes (the render is `app_shell`, #16). The
  pure surface — the stateless DCS codec (`decode_hook`/`encoding_for_dcs_terminator`) + the ordered
  `DcsScanner` + the `apply_hook` state machine + `BlockList` + the `write_bytes` re-queue logic — is
  **cov 100 / MSI 100** (unit-tested via a `PtyChannel` trait + a `MockPtyChannel`); the ONLY
  ACCEPTED-UNTESTABLE shim is `pty_os.rs`'s 4 raw OS calls (spawn/read/write/winsize). `unsafe`-free
  (`rustix::termios::tcsetwinsize`, not alacritty's aborting `on_resize`). Promotes the proven
  marley_spike PTY/grid patterns into a real session; `SessionId` from `marley_core`. Deps
  `alacritty_terminal`/`rustix`/`marley_core`. Documented in `docs/marley_architecture/terminal_blocks.md`.
- **`marley_ui_components`** (TICKET-009, forge #13; sprint M1.A 2/5) — the M1.A cut of
  SPEC-ui-components: the workspace's single **theming value vocabulary** — `Appearance` (Light/Dark) +
  `ThemeColors` (7 `gpui::Hsla` roles + `corner_radius`/`control_height`/`control_padding` metrics) +
  `ThemeColors::default_for` (the Light/Dark fallback palettes, R17, differing field-wise). The lowest
  gpui-dependent crate (the `Hsla` palette can't live in the gpui-free `marley_core`);
  `terminal_blocks`/`editor`/`app_shell` render from a borrowed `&ThemeColors`. A pure value-type
  library (no Render/headed surface → gate-15 N/A, no `mutants::skip`); cov 100 / MSI 100 whole-crate.
  The 5 widgets (button/switch/dialog/tooltip/keyboard-shortcut) of SPEC-ui-components defer to M1.B.
  Documented in `docs/marley_architecture/ui_components.md`.
- **`marley_spike`** (TICKET-006, forge #11) — the **throwaway** end-to-end viability spike proving
  the REUSE stack (`gpui` window + `alacritty_terminal` PTY + `vte` parse) composes: it opens one
  window "Marley Spike" 800×600, spawns `/bin/sh -c "echo hello-marley"` over a PTY at 24×80, drives
  the bytes through the parser into the grid, and renders one command **Block** (header above body).
  The pure decision surface (`build_block`/`exit_code_to_status`, `classify_read`, `exit_status_code`/
  `exit_code_for`, `body_from_rows`/`term_to_rows`, the seam-tested `drive_reader` read loop) is
  **100% coverage + mutation MSI 100**; the gpui/PTY shim (`app`/`term_io` + the runner bin) is
  ACCEPTED-UNTESTABLE (`mutants::skip` + the documented `rust_cov` exclude, the harness precedent) and
  exercised by a headless real-PTY integration test plus a `#[ignore]` headed visual test (which
  asserts the AX window + non-blank rendered content — strict screenshot pixel-match is deferred to M1
  since gpui text AA + titlebar focus exceed the harness's static-fixture tolerance). The pipeline
  caught three real bugs green `check`/`clippy`/unit all missed — an `alacritty` Pty-drop that
  discarded child output, an opaque-`ExitCode` unkillable mutant, and a non-blocking read loop with no
  backoff that raced the child to a blank body. **Throwaway** (deleted once the M0 gate is green;
  nothing depends on it). Deps `gpui`/`alacritty_terminal`. Documented in
  `docs/marley_architecture/marley_spike.md`.
- **`marley_visual_harness`** (TICKET-007, forge #10) — the **gate-15** dev/test harness, the
  normative target every `visual_acceptance` clause binds to. Launches the app headed (the
  subprocess model — a gpui runner bin observed out-of-process via osascript-AX + `screencapture`,
  since gpui owns the main thread), asserts the macOS AXUIElement element tree, and diffs
  screenshots against fail-closed approved baselines (`resolve_approval` promotes a committed
  baseline only under `MARLEY_VISUAL_APPROVE=1`; every CI run writes pending artifacts instead).
  The pure decision surface (geometry predicates, `compare`/`state_delta`, the `evaluate_baseline`/
  `resolve_approval` fail-closed core, `AxQuery`/`AxSnapshot`) is **100% coverage + mutation MSI
  100**; the display/subprocess shim (`os_shim`/`launch`/`selftest` + the runner bin) is
  ACCEPTED-UNTESTABLE (`mutants::skip` + a single documented coverage exclude in `rust_cov`), proven
  by a `#[ignore]` headed self-test against a committed baseline. `unsafe`-free (AX via osascript,
  not objc2 FFI). A spike established gpui exposes only window-level AX (inner content rides the
  screenshot baseline). Deps `gpui`/`image`/`image-compare`/`marley_command`; `deny.toml` now allows
  **MPL-2.0 / CC0-1.0 / NCSA** + ignores 5 gpui-transitive *unmaintained* advisories (chad-approved;
  private non-OSS app). R27 synthetic-input + the `xtask visual` CLI are deferred. Documented in
  `docs/marley_architecture/marley_visual_harness.md`.
- **`marley_command`** (TICKET-004, forge #7) — the workspace's single **non-PTY**
  child-process spawn seam: `blocking::Command` (over `std::process::Command`) and
  `r#async::Command` (over `async_process::Command`) drop-in builders, default stdio,
  env/cwd, `spawn`/`status`/`output`, the `ExitStatus`/`Output`/`Stdio` re-exports, and
  WSL detection (`is_wsl_from` reader seam + `is_wsl`). Adds a workspace `clippy.toml`
  banning `std::process::Command` everywhere but this seam (R14). **Unix-only**; the
  Windows console-flash suppression (`CREATE_NO_WINDOW`) + `JobObject` child-lifetime
  binding are deferred to **TICKET-004b** (forge #6, needs a Windows CI runner —
  `#[cfg(windows)]` code yields unkillable mutants on the macOS-only runner). Deps
  `async-process`. Coverage 100 / mutation MSI 100. Documented in
  `docs/marley_architecture/marley_command.md`.
- **`marley_core`** (TICKET-003, forge #5) — the offline-boot app/runtime core:
  `SessionId` (seam-contracts §2 owner, process-unique monotonic), the `~/.marley`
  path layout (`PathError`, never panics), the single-channel `Config` (lazy,
  pointer-stable, no cloud/auth surface), and the `FeatureFlag` registry (three-layer
  test-override → user-preference → baseline resolution, RAII thread-local override,
  debug init-guard, variant-count-sized storage). Deps `home` (not `directories` —
  avoids its MPL `option-ext`), `once_cell`, `serde`, `enum-iterator`. Coverage 100 /
  mutation MSI 100. Documented in `docs/marley_architecture/marley_core.md`.
- **`marley_util`** (TICKET-002, forge #4) — the workspace value-type vocabulary and
  seam-contracts §8 sole owner: `FileId` (process-unique), `ContentVersion` (monotonic
  revisions), `HostId` (local/remote), `StandardizedPath` + `PathFlavor` +
  `standardize_path` (`.`/`..` collapse, separator canonicalization, cwd-absolutization),
  and `LocalOrRemotePath`. Leaf crate; deps `serde` + `typed-path`. Coverage 100 /
  mutation MSI 100; `StandardizedPath` construction is closed (`#[non_exhaustive]`
  variants, proven by a trybuild compile-fail). Documented in
  `docs/marley_architecture/marley_util.md`.
- **Strict documentation phase** (forge #3) — `enforce-changelog.sh` (PreToolUse)
  blocks a Rust‑source commit that lacks a `CHANGELOG.md` entry, mirroring the §15
  commit receipt (no‑`.rs` changes are exempt); a strict `pipeline:complete` doc
  step; and **CONSTITUTION §21 — Documentation Phase**. A phase‑skip‑resistance
  audit confirmed the five skip-resistance hooks (phase‑gate, phase‑tasks,
  pipeline‑completion, tests‑ran, commit‑gate) bite as designed (`enforce-quality`
  covers formatting separately). The changelog hook checks the **staged index**, so
  an untracked/unstaged `CHANGELOG.md` can't satisfy it.

### Changed

- **`marley_visual_harness` — text-tolerant screenshot baselines** (TICKET-008, forge #12). The
  gate-15 harness can now baseline-test **text-rendering** windows (the M1 prerequisite): a titlebar
  `RegionMask` (`RegionMask::titlebar_band`) excludes the focus-variable macOS titlebar chrome from the
  comparison; `Tolerance::gate15_text` absorbs gpui subpixel anti-aliasing while `max_changed_fraction`
  (>2% of pixels) stays the binding gate for real content changes; and `mask_out` +
  `Screenshot::assert_matches_baseline_masked` thread the mask through the compare. `gate15_default` +
  the existing self-test are untouched. The pure additions are 100% coverage / MSI 100; a `#[ignore]`
  headed proof matches deterministically across ≥3 runs. Closes
  `AD-claude-headed-visual-baseline-text-tolerance-deferred-001` and unblocks the M1 UI crates.
- The commit-receipt fingerprint (`gate_state_hash`) now also binds `clippy.toml`, so
  a gate-defining clippy config (e.g. the R14 `disallowed-types` ban) can't be weakened
  around a green receipt (TICKET-004 inspect;
  `PR-claude-gate-defining-files-in-receipt-fingerprint-001`).

### Removed

- **Gate-16 (spec-layer clean-room provenance)** — `scripts/spec-provenance.sh`, the
  planned `marley_spec_provenance` crate (TICKET-005, cancelled), and `SPEC-gate.spec.md`.
  Marley is a private, non-OSS rebuild; a raw identifier diff of each spec's public
  surface against the `warp_architecture` docs can't distinguish Marley's own names from
  the fork's shared Rust/terminal vocabulary (a spike flagged 194 legitimate names), so
  the spec-layer clean-room wall is now enforced by review, not an automated gate. The
  quality bar is **15 gates**. (TICKET-006 — owner decision.)

- **`marley_spike`** (TICKET-027, forge #27; sprint M1.C "The Wired Cockpit" 6/6 — the FINAL
  ticket) — the M0 viability spike, explicitly throwaway ("deleted once the M0 gate is green").
  It had proven the REUSE stack composes (gpui + `alacritty_terminal` + vte → one window + a PTY
  + a rendered Block); those patterns now ship in `marley_terminal` + `marley_app`, the live
  gpui+PTY exemplars. Deleted: the `crates/marley_spike` crate (the `members = ["crates/*"]` glob
  auto-drops it), `SPEC-foundation-spike.spec.md` (its `component: marley_spike` gate-15 binding),
  the as-built `marley_spike.md`, the `scripts/gates.sh` coverage-exclude for its shim files, and
  the crate-map node/row; the lineage/precedent prose in the terminal-blocks/app-shell docs + two
  headed tests is reworded to "the M0 spike". Every FULL gate run now stops covering + mutating
  dead code. **This closes the M1.C "The Wired Cockpit" sprint (#22–#27).**

### Fixed

- **Command Blocks: `;`-safe DCS hooks end to end** (TICKET-022, forge #22; sprint M1.C "The Wired
  Cockpit" 1/6) — two halves of one bug. (1) `marley_app`'s shell-integration rc now emits every
  DCS frame under the `AnsiCQuoted` (`q`) selector, escaping the dynamic values (preexec
  `command`, precmd `pwd`) via a `__marley_quote` zsh chain (`\` `;` ESC `\n` `\t` `\r`, backslash
  first; the `__MARLEY_REPLY` no-fork idiom). (2) `marley_terminal::decode_hook` now splits an
  AnsiCQuoted payload on UNESCAPED separators BEFORE un-escaping (SPEC-terminal-blocks R24) —
  previously the whole payload was un-escaped first, so the emitter's `\;` collapsed back to a
  bare `;` before the field split and NO emit-side encoding could survive it (caught by the
  inspect critics running the real decoder). A command containing `;` (`ls; pwd`), a raw
  backslash, or an ESC byte now round-trips into its Block exactly, and a `;`-containing `$PWD`
  can no longer truncate or inject phantom prompt fields (`…\;git=evil` → literal pwd, no `git`
  field). Hex/Plain decode semantics are byte-identical; every pre-existing `marley_terminal`
  test passes with assertions unmodified. (`=` inside a value was never at risk —
  `split_once('=')` keeps it.) SPEC-app-shell gains R25/R26; SPEC-terminal-blocks gains R24.

### Security

- **Two new quick-xml RUSTSEC advisories acknowledged with per-id ignores** (TICKET-022 gate run) —
  RUSTSEC-2026-0194 (quadratic duplicate-attribute check) + RUSTSEC-2026-0195 (NsReader
  namespace-allocation DoS), both published 2026-06-29 against `quick-xml` 0.30/0.39. Both enter
  the tree ONLY via `wayland-scanner`/`xcb` — Linux-only gpui windowing deps compiled out of the
  shipped macOS target — parsing bundled protocol XML at build time (no untrusted input, no
  semver-compatible fix until upstream bumps to quick-xml ≥0.41). Ignored per-id with
  justification in `deny.toml` (gate:8) + the new `.cargo/audit.toml` (gate:7), the TICKET-007
  visible-per-id pattern; revisit when gpui updates its tree.

## [0.0.0] — 2026-06-28

### Added

- **Marley quality pipeline finalized** (TICKET-000, forge #1) — `scripts/gates.sh`
  ported to the 16‑gate quality bar (`docs/specs/standards/quality-bar.spec.md`);
  the `.claude/` enforcement hooks gate `crates/`; `deny.toml` permissive license
  allowlist with **MPL‑2.0 dropped** + crate `license.workspace` inheritance;
  `CONSTITUTION.md` rewritten for Marley (gpui/`crates`) with **§20 clean‑room
  provenance**; gate‑15 (visual/AX, conditional + fail‑closed) and gate‑16
  (spec‑provenance) wired; `docs/planning/` working tree + templates bootstrapped.
- **`marley_text_offsets`** (TICKET-001, forge #2) — the type‑safe `CharOffset` /
  `ByteOffset` newtypes (private fields, non‑interchangeable; mixing fails to
  compile) and the forward‑only streaming `CharCounter` byte→char converter, per
  `SPEC-text-offsets.spec.md` R1–R22. The first crate to drive the FULL gate green:
  **100% line coverage + mutation MSI 100%**. Deps: `serde`, `get-size2`.
