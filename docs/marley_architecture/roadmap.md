# Marley roadmap — the live forward plan

The authoritative forward plan. The original phased plans ([implementation-plan.md](implementation-plan.md) = the
*method*, [clean-build-plan.md](clean-build-plan.md) = the *milestones*) are now historical snapshots — the
product ran past them to **M15**, and the **Zed deconstruction** ([../zed_architecture/subsystems/00-overview.md](../zed_architecture/subsystems/00-overview.md))
opened a whole new dimension (Zed-level editing) that those plans never anticipated. This doc supersedes their
"Sequencing" sections going forward.

## Completed — M0 → M21 (the foundation is built; B1–B6 of the editor frontier are done)

| Milestone | Shipped |
|---|---|
| **M0** | Foundation spike — gpui window + `alacritty_terminal` PTY + one rendered Block (proves the stack) |
| **M1** | Terminal MVP — command **Blocks**, the input editor, panes/splits, the command palette, themes |
| **M1.A/B** | The pure `marley_editor` buffer + the typed `marley_settings` TOML framework + the cockpit widgets |
| **M2–M9** | Project model, the panel/dock system, cockpit sections, forge/knowledge panes *(since ripped — #411)*, agent orchestration (`marley_agent` launches/observes/controls agent CLIs), the `marley_remote` ssh pane |
| **M10–M11** | Shell hardening, session/layout persistence, tab management, rename |
| **M12** | Warp-parity polish + **clickable `file:line:col`** links (#196) + OSC 8 hyperlinks (#214) — the fusion wedge's front half |
| **M13** | The Workspace Cockpit — docks/panes, the launcher, the multi-workspace direction, rail highlight |
| **M14** | Cockpit polish (dark default, type scale, icon pass) + editable-editor prep |
| **M15** | ⭐ **The Editable Editor** (#249–258) — the on-screen editor now types / saves (⌘S) / undoes (⌘Z) / selects / copies (⌘C/X/V) / navigates, from a real `marley_editor::Buffer`, with persisted split panes |
| **M16** | ⭐ **The editor frontier** (#260–269) — **B1 KeyContext** (#265), **B2 gpui substrate** (`uniform_list` #266 + `EntityInputHandler`/IME #267), **B3 tree-sitter** semantic highlighting (#268), **B4 anchors** |
| **M17** | The solid sprint (#272–288) — auto-indent, readline keys, mouse/TUI fidelity, external-change detect, incremental + off-thread parse |
| **M18** | Terminal↔editor **fusion** (#212/#213/#270/#289–294) — a failed command's error refs light the editor's diagnostic gutter (Phase C's first real thread) |
| **M19** | **B5 multi-cursor** (#296–299) — `SelectionSet` at N, ⌘D/⌘⇧L/⌘⌥↑↓/⌘-click, comment-toggle at N cursors |
| **M20** | **B6 LSP** (#308–313) — the rust-analyzer client: lifecycle, doc sync + the encoding bridge, diagnostics, hover, go-to-def, completions; + formatting (#314) + **multi-language syntax highlighting** (#315 — Python/JS/TS/TSX/JSON/Bash on tree-sitter, the 5→10 token taxonomy) |
| **M21** | ⭐ **The workspace IDE** (#322–331) — rename, code actions, signature help, ⌘T symbols, **B7's ⌘⇧F project search** (#326), problems panel, git gutter, expand-selection, sticky header, inlay hints |

Detail: [app_shell.md](app_shell.md) · [editor.md](editor.md) · [crate-map.md](crate-map.md) · [CHANGELOG.md](../../CHANGELOG.md).

---

## The forward phases (expanded from the Zed discovery)

The editor is a real **multi-cursor** editor with a broad **LSP** client today (M16→M21 built B1–B6). The
frontier — **Zed-level editing with Warp-level slickness** — is sequenced below.

**The goal, stated precisely (chad, 2026-07-16):** rival Zed/VS Code's **EDITING ability** — NOT their IDE
breadth. Marley does not chase the extension marketplace, the debugger, or the plugin host; VS Code's moat is
30,000 extensions and it is neither winnable nor worth wanting. The bar is **"enough that a developer chooses
to write code in it."** Everything past that bar lives in [Would love to have](#would-love-to-have--not-the-bar). Every step's *how* is in [../zed_architecture/](../zed_architecture/subsystems/00-overview.md),
provenance-tagged; the licensing boundary is in [../warp_architecture/subsystems/00-overview.md](../warp_architecture/subsystems/00-overview.md)
+ [../zed_architecture/README.md](../zed_architecture/README.md).

### Phase A — Cleanup + foundation hygiene *(do first)*
Clear the debt + lock the disciplines before the deep editor work.
- **Brand-scrub** — remove the 56 Warp mentions from the source (comment-only, gate-safe) + wire the keep-clean
  lint. Plan: [../planning/design-notes/brand-scrub/README.md](../planning/design-notes/brand-scrub/README.md).
- **Top-bar workspace indicator** removal (#260) — the workspace lives in the left rail.
- **Emoji → clean-room SVG icons** (#261) — the 11 remaining colorful emoji (icon-audit groups A–E).
- **Warp-doc link fix** — the crate docs' `Subsystem` links point at `../architecture/` → should be `../subsystems/`.
- **Adopt gpui `render_to_image` + `VisualTestContext`** for the visual gate — retires the `screencapture`
  locked-screen / window-shadow hazards ([../zed_architecture/crates/gpui.md](../zed_architecture/crates/gpui.md)).
- Resolve `marley_util` (built but orphan — wire a consumer or fold it in).

### Phase B — The editor frontier *(Zed-level editing)* — **B1–B6 SHIPPED**
Dependency-ordered; **anchors were the universal prerequisite** (delta-log, rebased through the existing
`BufferDelta` — Marley-original, GPL-clean; full CRDT deferred).
- ✅ **B1 — KeyContext** *(gpui, Apache-2.0)* — context-scoped keybindings (**#265, M16**). Fixed the ⌘D
  collision + every ⌘F/⌘//Enter/Esc conflict.
- ✅ **B2 — gpui substrate** — `uniform_list` + `StyledText.with_highlights` (**#266**) + `EntityInputHandler`
  real IME (**#267**), M16. Deleted the hand-rolled render; became the syntax substrate.
- ✅ **B3 — tree-sitter** — semantic highlighting (**#268**, M16); incremental + off-thread (**#274/#285/#288**,
  M17); the node-range API (**#329**, M21) now feeds expand-selection + the sticky header; **multi-language
  (#315, M20)** — a `Lang` axis over 7 grammars (Rust/Python/JS/TS/TSX/JSON/Bash) + the 5→10 token taxonomy.
  *Highlighting is multi-language; the STRUCTURAL node APIs (fold/sticky/bracket/tags/expand-selection) are still
  Rust-only (per-grammar structure + `brackets`/`outline`/`injections`/`runnables` remain unclaimed scope).*
- ✅ **B4 — Anchors** — delta-log anchors, Marley-original (M16). *Not yet on the caret path — a concurrent edit
  would stale it.*
- ✅ **B5 — Multi-cursor** — `SelectionSet` at unbounded N; ⌘D / ⌘⇧L / ⌘⌥↑↓ / ⌘-click (**#296–299**, M19).
- ✅ **B6 — LSP** — a clean client from the *published spec* (**#308–313**, M20 + **#322–331**, M21): lifecycle,
  doc sync + the encoding bridge, diagnostics, hover, go-to-def, completions, rename, code actions, signature
  help, workspace symbols, problems panel, inlay hints.
- 🔶 **B7 — Project search → multibuffer** — ⌘⇧F content search SHIPPED (**#326**, M21); the **multibuffer
  capstone** (editable multi-file excerpts) is the remaining half → folded into M22 below.

### M22 — **The editing bar** *(the MVP: "worthy of using")*
The bar is daily-use editing, not IDE breadth. **The expensive half is already built** (multi-cursor at N,
tree-sitter expand-selection, real IME, undo that restores N cursors, the LSP breadth). What is left is mostly
cheap muscle-memory ergonomics — plus one architectural piece that pays for three features at once.

**B-a · Ergonomics — the muscle memory** *(all small + pure, on shipped infra)*
- ✅ **Horizontal scroll — SHIPPED (#336).** Was the batch's only real *defect*: the render is
  `.whitespace_nowrap()` and `CodeViewState.scroll` was a ROW index, so an over-wide line clipped and its tail
  was **unreachable by any means**. Now a pure `h_scroll.rs` seam + a clipper/shift split that keeps the gutter
  fixed by construction; row↔line stays 1:1, so it needed **no display map** — which is exactly why it went
  first. Notable: the click math needed NO change (taffy folds the shift into the probed bounds, so the scroll
  cancels), and the width measure had to move to display CELLS (`display_cols`) or CJK tails stayed stranded.
  Follow-ups: the horizontal thumb (needs a body-relative code-column origin), a column-aware
  `scroll_editor_to(row, col)` for ⌘D, and the wheel's x-delta scaled by line height.
- ✅ **Font + appearance settings — SHIPPED (#337).** `appearance.font_size` (8–32, default 13) + a
  `font_family` key, live via ⌘= / ⌘− / ⌘0, persisted on every step. **Content zooms, chrome does not** —
  `type_scale(role, font_size)` scales the terminal's command/output text and the editor's code; captions and
  the nav rail stay put, so ⌘= is `editor.fontSize`, not View→Zoom. The plan's premise — that
  `TERMINAL_FONT_SIZE` was the ONE const behind both surfaces — was **false**: `type_scale` held a second
  hardcoded scale and the two agreed only because #195 set both to 13.0, so the work was *converging* the
  sources, not threading one. The const is deleted; `font_zoom.rs` (pure, total `clamp_font_size` into
  [8,32]) is the single home for the numbers, sourced by `fallback_cell`'s guard and the setting's default —
  that literal had gone stale twice. Follow-up: an unresolvable `font_family` falls back **silently**
  (#344); the flash needs a designed probe seam (gpui does expose `TextSystem::font_id -> Result`).
- ✅ **Auto-close brackets + quotes — SHIPPED (#338).** `(` gives `()` caret-inside; `)` against its twin steps
  over; ⌫ between a pair takes both; an opener over a selection wraps it — per cursor, one undo unit.
  `editor.auto_close` (default on) + a palette toggle; OFF byte-identical. **The plan called it "one pure table
  applied through the shipped engine"; three of its four actions were inexpressible that way** — the #297
  engine does the TEXT half only (its post-state is always a bare caret at the END of each insert), so the fix
  generalized that post-state to an optional per-cursor SPAN whose identity case IS the old math (property-
  pinned against a verbatim copy of the pre-ticket body). The ticket's real lesson was an **invariant the
  producer asserts and the consumer trusts**: `cursor_anchored: true` was hardcoded, `coalesces_into` trusts it
  instead of checking, and a caret span is the first thing able to break it — redo replayed `()x` where the
  text read `(x)`. The same shape appeared twice more (redo at N=1; pair-backspace's ⌘Z). Follow-ups: syntax-
  aware suppression (no pairing inside strings/comments) and the `'` bound positions both wait on #315.
- ✅ **Move / duplicate line — SHIPPED (#300).** ⌥↑/⌥↓ move, ⇧⌥↑/⇧⌥↓ duplicate; multi-cursor blocks, cursors
  carried, one undo unit. Pure `line_move.rs` (the line-reorder seam); D-CARRY-NOT-REBASE + char-based carry
  + offset-span endpoint attribution (the inspect HIGH — a full-line selection's carved boundary head).
- ✅ **Go to line — SHIPPED (#302).** ⌃G opens an inline overlay; `50`/`50:12` + Enter jumps + centers, Esc
  restores. One new pure fn (`parse_goto` — syntax only) feeding the pre-existing `caret_for_line_col`
  (which already owned the 1↔0 conversion + past-EOF/EOL clamp — the Phase-1 finding that dissolved the
  planned `clamp_goto`); the overlay copies the `renaming_symbol` shape + the leak gate; NavStack pushes the
  origin on commit only.
- ✅ **Delete line / word — SHIPPED (#303).** ⌥⌫/⌥⌦ delete word, ⌘⌫/⌃K to line edge, ⌘⇧K the line; plus plain
  forward-delete. Pure `delete.rs` (the delete-range seam); D-MOTION-IS-THE-RANGE (reuses movement.rs) +
  D-REBASE-IS-CORRECT-HERE (the clamp #300 banned, correct for a delete). The feared "translation collapse"
  dissolved — the keymap resolves the 5 chords before key_from_keystroke, no seam rewrite.
- ✅ **Code folding — SHIPPED (#305).** ⌥⌘[ folds the innermost definition at the caret, ⌥⌘] unfolds, palette
  Fold All / Unfold All; a "⋯ N lines" tail, anchor-keyed (survives edits above, evaporates on region delete),
  auto-reveal on any jump into a fold, Rust-only (caller-gated). The real work is the **first buffer-row↔visible-row
  projection** — pure `marley_syntax::fold_regions` + `FoldProjection` (the fifth node API), converting exactly
  ONCE at the `uniform_list` rim so the ~17 interior sites stay buffer-row (identity + no parse when nothing is
  folded). Two lessons: auto-reveal had to hook the LOWEST shared primitive (`reveal_and_scroll_to_row` at every
  jump site — the caret-follow path missed the direct-scroll jumps incl. FIND); and `FoldProjection` stores
  HALF-OPEN hidden runs so `slot_of`'s boundary mutant isn't equivalent (MSI 100). The LSP overlay-card geometry
  above a fold is the deferred Slice-2 (#352).
- ✅ **Format on save — SHIPPED (#314).** ⌥⇧F formats via rust-analyzer's `textDocument/formatting`; an opt-in
  `editor.format_on_save` (default OFF) runs it before every ⌘S. The apply reuses the #322 single-doc engine
  (`apply_one_file` — resolve → reject-overlap-whole → one undo group); the pure delta is `marley_lsp::formatting`
  (capability + params + the bare `TextEdit[]` parse). Two lessons baked in: the caret is a **(line, col) re-seat**,
  NOT an anchor (rust-analyzer's whole-file edit COVERS the caret → a covered anchor collapses to 0 → line-1 jump);
  and the never-block save orchestration (park → response/deadline → ONE write, bound to the ORIGIN editor so a
  tab switch mid-format never saves the wrong file). `willSave`/`willSaveWaitUntil` (deferred in #309) stay deferred.
  Follow-up **#354 — SHIPPED**: the latch binds the origin's `ContentId`; every completion lane (response /
  Err / stale / deadline) saves the ORIGIN instance wherever focus went, with background-consent conflict
  handling and owning-root `didSave` routing — a switch mid-format-save no longer abandons.
- ✅ **Multi-cursor Tab — SHIPPED (#307).** Tab indented only the PRIMARY cursor's block, breaking a
  promise B5 shipped. The recon's finding: **BOTH branches were primary-only**, and the ticket's own verify
  step (⌘⌥↓ ×2 → Tab) exercised the bare-caret pad branch that its "The work" section never named — a
  rows-only fix would have failed the ticket's own test. Now a block op indents the union of every cursor's
  rows (`touched_rows`) and an all-carets Tab pads every caret from its own display column. The builders
  widened to a discontiguous `rows: &[usize]` and now ENFORCE the ascending+deduped contract, adopted from
  their sibling `comment_edits` (whose comment had named #307 as the incoming caller — the prior-art sweep
  amending a locked decision before any code was written). A latent bug fell out as a rider: the predicate
  read `active_selection()`, primary-only, so a mixed set ignored its range. ⌘Z granularity is deliberately
  asymmetric — block ops always group (preserving #282's no-op-can't-clobber-redo), a lone pad stays
  ungrouped so it still coalesces with following typing.

**✅ B-a COMPLETE** — with #307 done, every ergonomics item has shipped. **B-b is complete too**, so the
whole remaining M22 surface is the **B-c display-map chain** below (foundation → soft wrap → multibuffer).
- ✅ **Regex find — SHIPPED (#339).** A `.*` mode + a case chip on the ⌘F editor bar; the matches feed the
  bands/n-of-m/navigation unchanged, an invalid pattern shows an inline error, OFF is byte-identical. The
  byte→char seam is one monotone `char_indices` cursor (regex reports bytes; the editor counts chars — the
  #336 units class); full-Unicode `(?i)` deliberately diverges from the ASCII literal fold. **Capture-group
  REPLACE split to #347** (the match shape discards the captures replace needs — the reason it split). The
  prior-art sweep DISSOLVED the hand-rolled empty-match rule — `regex::find_iter` owns it, better.
- ✅ **Bracket-match highlight — SHIPPED (#340).** The caret's `(`/`[`/`{` pair lit (a subtle mark, LOWEST
  tier so find/selection win a shared cell) + ⌘⇧\ Go to Matching Bracket. Rust-only via the tree, so a `(`
  inside a string/comment provably never false-positives; the third `marley_syntax` node API
  (`matching_delimiters_in`, parse-only, ADJACENT-only). Inspect caught a dropped language gate (Rust-grammar
  on any file — REQ-008) + a missing-node phantom (`is_missing()`); the cached-tree perf follow-up is #349.

**✅ B-a COMPLETE — `/work 336-340` shipped all 5 of the M22 editing-bar batch** (#336 h-scroll · #337 font/
appearance · #338 auto-close · #339 regex find · #340 bracket-match). All LOCAL commits; push un-OK'd.

**B-b · Editing surface** — **✅ COMPLETE 2026-07-17/18 (the IDE-MVP shelf batch).**
- ✅ **Editable split pane — SHIPPED (#259).** The #246/#258 read-only split became a real editor: a
  FOCUSED code pane types/saves/selects/moves/undoes like the editor tab — the first time "the editor"
  is not a singleton. The spine: `active_editor()/_mut()` went FOCUS-aware (resolving the focused
  editable pane, return type unchanged), so ~75 call sites moved for free; `key_context()` and the
  geom/IME slot follow focus. Same file in tab + pane = two independent Buffers with the #275
  conflict machinery as the no-silent-clobber net. Slice 2 (find/fold/⌘⇧O/LSP-in-pane) = #355;
  unfocused-pane stale render = #356; the pane conflict banner = #357.
- ✅ **In-file outline / Go to Symbol in File — SHIPPED (#304).** ⌘⇧O fuzzy-picks the current Rust file's
  symbols (fns/structs/enums/unions/types/traits/methods/mods/macros), Enter jumps + centers + NavStack. NOT
  the LSP `documentSymbol` — a pure `marley_syntax::file_symbols` over tree-sitter-rust's OWN `tags.scm` (the
  4th node API); the method/function dedup + the class-split; caller-gated on Rust (the #340 shape). ⌘⇧O is
  Editor-scoped, shadowing the global open-remote (the resolved chord conflict). Rust-only v1 (#315 = the
  language axis); bodyless trait/extern sigs are a documented tags.scm limit.
- ✅ **Find references — SHIPPED (#317).** ⇧F12 → a grouped-by-file picker ("13 references in 4
  files"), per-file counts + line-text rows, one Enter to jump (+ NavStack). The "Finding references…"
  card is gated on the host's pending-request table, not an app-side latch — the #331 naive-latch
  lesson, reprised (a timed-out request must not strand the card).

**B-c · The display map — the one architectural chain** *(strictly ordered; each needs the one before; the
M32 sprint #425–#430 — `display-map-shelf.md`)*
- ✅ **Display-map foundation — SHIPPED (#425, M32).** ONE typed facade
  (`marley_app::display_map::DisplayMap` over the shipped `FoldProjection`; `BufferRow`/`DisplayRow` in
  `marley_text_offsets`, trybuild-pinned distinct) now owns every buffer-row↔display-row conversion — the
  16 raw crossing sites are gone, `EditorFrameGeom`/`HoverCard` speak typed slots (the F-#352 class is a
  compile error), and #426/#427 insert layers HERE. Byte-identical by proof (facade property-equals the
  direct projection; 2174 tests unchanged; live-drive verified). The #331 invariant kept deliberately OUT
  of the row stack: `LineLayout`'s two boundary maps stay the column authority
  (`AD-claude-two-boundary-maps-for-phantom-text-001` — code spans hug the code, caret ranges track the
  caret); wrap is what makes rows×columns negotiate.
- ✅ **Soft wrap — SHIPPED (#426, M32).** `editor.soft_wrap` (default OFF; live palette toggle) renders a
  too-wide line as N display rows via the facade's second layer: break cells over the phantom-aware
  `LineLayout` (gpui LineWrapper's RULES re-expressed on the mono grid — the prior-art sweep's win),
  fold∘wrap composition, one-slot-per-SEGMENT rim (clip-and-re-base, never re-lex), display-row ↑/↓ with
  goal preservation, caret-segment overlay anchors, and the #336 h-scroll set structurally inert while ON
  (OFF byte-identical, suite-pinned). Inspect killed three adopted-rule deviations + the tab-unsound
  pre-filter + the phantom-blind column class pre-commit.
- **Code folding** (#305) — the #329 node-range API's other named consumer.
- **Multibuffer** (B7's remainder) — **#427 read-only + #428 EDITABLE + #429 REPLACE ALL SHIPPED**: the stitched
  surface from ⌘⇧F — the pure excerpt/row model (cov/MSI 100), the #403-kind wiring (always-insert, transient
  across restarts, the writer's first dropping arm re-pointing `active_tab`), full-row match washes, the shared
  jump recipe + a materialization-origin NavStack push. #428 added the anchor-backed caret, write-through via
  the ONE insert mechanism, the group-keyed undo journal, touched-consent ⌘S, and the pinned/lazy target
  lifecycle. #429 added the overlay's replace row + ⌘⌥R regex mode (one compile per request, invalid renders
  inert), ⌘⌥⏎ Replace All through the same target recipe ($n captures; per-buffer undo groups), and the batch
  ledger making the whole gesture ONE ⌘Z/⌘⇧Z across every file — guards fail closed. #430 CLOSED the
  chain: the problems panel's editable form — diagnostics as note-banded excerpts (Row::Note slots over a
  materialized slot list), per-file-root targets, and the publish-epoch quiescent refresh (fix in place →
  save → the diagnostic leaves the surface; proven against live rust-analyzer). Remaining: the DisplayMap
  unification (the chain's later prize). The signature capstone — B-c 6/6 SHIPPED.

### Phase C — The fusion *(threads through Phase B — Marley's wedge)*
Where terminal-first *beats* Zed. Zed has **no command-block model**; Marley owns the block infra.
- tree-sitter `runnables` → a gutter run-button → **spawn as a first-class command Block** (status pill, framed
  output, per-block rerun, block-scoped jump-to-failure, brain-observable pass/fail).
- run a task → jump to the failure line (LSP diagnostics × the block-terminal).
- Front half (clickable `file:line:col` → editor) shipped #196/#214. ([../zed_architecture/subsystems/08-terminal-tasks-fusion.md](../zed_architecture/subsystems/08-terminal-tasks-fusion.md))

### Phase D — The brain *(the sellable, clean-room, open-core product)*
The proprietary product on top of the GPL/AGPL editor+terminal.
- The agent orchestration loop, provider-agnostic streaming, MCP tools, context assembly (the edit-prediction
  concept). **Must be clean-room + a separate program** across the `EditOrigin::Agent` / `session.read` / MCP seam
  — the **AGPL §13** boundary keeps it sellable. ([../warp_architecture/subsystems/04-agent-ai-mcp.md](../warp_architecture/subsystems/04-agent-ai-mcp.md) · [[brain-agent-session-supervision]])
- The brain is the **judgment half** of [The Fleet Control Plane](#the-fleet-control-plane--marley--forge-the-orchestration-end-goal)
  below (Marley = hands + eyes, the manager agent = judgment, invoked by Marley).

### Phase E — The embedded browser
Chromium / CEF pane (live preview / agent web-browsing) — the [[embedded-agent-browser-chromium-cdp]] pillar.
Now also carries the fleet plane's browser half ([orchestration-shell.md](./orchestration-shell.md) §8–9): the
**project Forge URL as a live pane**, the **CDP→MCP browser tool family**, and the **hosted agent seat** —
depends on `marley_mcp` (fleet Layers 1–2) existing first. **Opening moves designed (M26 #389 spike):
[embedded-browser-model.md](./embedded-browser-model.md)** — wry-as-child WKWebView for the Forge pane
(z-order the load-bearing risk; CEF off-screen-render the named revisit), two substrates for the CDP
fork, the `.mcp.json`-derived Forge URL + a framing-safe codec, and the follow-up train (proof-of-embed
first). **The train is moving (M29):** slice 1 ✅ **#402 — proof-of-embed, verdict GO** (wry-as-child
WKWebView holds on all three checklist points: pane-rect sync, the overlay z-order hide-shim, and the
focus/IME handoff; CEF-OSR stays the named, untriggered pivot). Slice 2 ✅ **#403 — the browser is a TAB
citizen**: it opens from the Browser＋ menu, files under the #385 Browser section, persists as a bare `B`
shell entry (no id, no URL — content was the derived Forge origin), closes, and renders a placeholder that
reads no config at all. The train then completed: **#404** the URL seam ✅, **#405** the webview + z-order
shim ✅, **#406** nav chrome + lifecycle ✅ — and **#410 (M30, the 2026-08-09 scrap-forge pivot) ripped the
train's Forge identity back out**: no production origin source remains (the mount/Retry `None` is the seam
a future URL feature fills), the pinned-origin verdict moved in-app, and the placeholder is the standing
state. The generic embedded-browser infrastructure survives fully tested; the Forge-URL product goal
itself is retired (`docs/planning/intake/scrap-forge-pivot.md`; the fleet/cockpit half ripped at
**TICKET-411**, landed 2026-08-09 — `marley_forge_client` deleted, the sprint cockpit + transport gone,
the fleet rail forge-agnostic and quiet). The CDP agent-browser lane stays a separate pillar.

### Phase F — Ops + remote runners
k8s / Acquia panes + agents-on-web (remote runners); brain optionally remote. The original M5, still ahead.
The seat/session model + VPS reach of [The Fleet Control Plane](#the-fleet-control-plane--marley--forge-the-orchestration-end-goal) land here.

---

## The Fleet Control Plane — Marley × Forge *(the orchestration end-goal)*

> *(Amended 2026-08-09 — the scrap-forge pivot, #409/#410/#411: the **Forge-facing halves** of this
> end-goal are **retired**, and #411 has landed — the transport/sprint surfaces are gone, the fleet rail
> survives forge-agnostic. The seat/session mechanism-shell half stands as design; see the banners on
> both linked docs.)*

The **second axis** of the product, orthogonal to the editor frontier above. **chad calls its two feeder
pillars "the core of Marley"** — [tmux-grade detached sessions](../planning/intake/tmux-grade-detached-agent-sessions.md)
+ [MCP-first-class](../planning/intake/mcp-first-class-control-plane.md) — and this is the concrete end-goal design
that converges them with [mission-control](../planning/intake/mission-control-hypermedia-surface.md) and Phase D's
brain. Grounded in a real night of fleet operation (2026-07-19; 4 agent seats, 5 PRs merged, every classic
remote-agent failure mode in one evening), where **everything that flowed as data worked and everything that flowed
as keystrokes-and-screenshots broke.** Full design: **[fleet-control-plane.md](./fleet-control-plane.md)** ·
implementation deep dive: **[orchestration-shell.md](./orchestration-shell.md)** (2026-07-20 — the cast
[Forge = brain/state · manager seat = policy loop · Marley = mechanism shell], the **agnostic `Session` envelope**
[mechanism-not-policy; Marley is NOT UCSOS-baked — `marley_forge_client` was adapter #1 of N until its #411 deletion], the decided forks
[push = MCP-native subscription, never raw PG `LISTEN`; the loop stays in the manager seat; `marley_agent`
ownership inversion is the prerequisite], the crate map [`marley_fleet` pure + `marley_mcp` expose], and the
**browser + hosted-agent extension** — the project's Forge web URL living in the embedded Chromium pane
*(this half retired: the 2026-08-09 scrap-forge pivot, #410/#411)*, the
CDP→MCP tool family, and Marley hosting its own Claude/Codex seat as the manager's in-shell home).

**The thesis — `tmux is the substrate, never the API`.** Keep tmux (or a pluggable multiplexer — zellij is the
Rust-native alternative) for what it is unbeatable at: process persistence across reboots, human attach/detach, a
hardened PTY layer. Stop abusing keystrokes + screen-scraping as an RPC protocol — that is the disease behind every
fleet incident. The cure is a **control plane beside the terminal stream**: agents *declare* state and accept
*typed, receipted* commands; the terminal stays the human window. Terminal = cockpit window; control plane = radio +
flight recorder; Marley renders both. This sharpens [detached-sessions.md](./detached-sessions.md)'s
"renderer ⊥ transport" into "substrate ⊥ API".

**The first thing to build is NOT Marley code — it is the event contract.** A `seat_events` table +
`LISTEN/NOTIFY` channel on Forge's Postgres, emitted by a ~20-line Claude-Code hook (turn-end + phase-boundary →
`session-start`/`turn-complete`/`phase-pass`/`halted-with-question`/`api-error`/`pr-opened`/…), carrying **stable
seat identity** + declared `state` + `capabilities`. Dogfooded in **ucsosv2 now**; Marley *consumes it unchanged*
later. Completion is a hierarchy — PTY-silence (ambiguous fallback) < harness events (deterministic) < domain events
(work-as-data) — and Postgres `LISTEN/NOTIFY` gives Marley true **push**, retiring the 15-minute poll to a heartbeat.

**Marley's first-class surface** (a data model, not plugins — §7 of the design doc has the detail): ① the event
contract as the **spine** (every pane a subscriber; no screen-scraping on the primary path); ② the **fleet rail** —
seats as first-class objects (state / ticket / phase / elapsed / held-PR / capabilities) + a base-red
**branch-health registry** cited by gates; ③ **terminal passthrough per seat** (native PTY, who's-driving indicator,
human takeover/hand-back — the one feature whose absence blocks migrating seats off tmux); ④ **structured interrupts**
— `halted-with-question` rendered as a form (the [mission-control AskUserQuestion loop](../planning/intake/mission-control-hypermedia-surface.md),
same receipted answer path); ⑤ a **dispatch composer with delivery states** (`deposited → claimed → started`,
capability-preflighted); ⑥ **Forge work objects as native panes** (ticket ↔ run ↔ gate evidence ↔ held PR ↔ AAR) —
what "Forge first-class" cashes out to; ⑦ **verify-and-merge as a guided flow**; ⑧ an **owner-class inbox** separate
from the manager firehose. The role split: **Marley = hands + eyes** (mechanics, watching, routing); the **manager
agent = judgment** (merge calls, halt decisions), *invoked by* Marley — the fleet's brain/hands split applied one
level up.

**Sequencing — NOT scheduled; the IDE milestones (M22 →) continue first** (the pillars' standing stance). Adoption
is dogfood-first and needs zero Marley code to start: **Layer 1** ships the seat-emitter hook + `dispatch.sh`
hardening in ucsosv2 (tmux unchanged, tick-detector reads Forge state first); **Layer 2** trials one bridge-owned
seat when passthrough exists; the **end state** is Marley subscribing to everything, with tmux kept as the human
viewport + degraded-mode fallback (Marley down ≠ fleet down — the mailbox + tmux path keeps working). What is
*already built* to lean on: the Forge **mailbox** (zero failures all night — the dispatch half is control-plane
shaped already), the `bridge-session-*` + `bridge-remote-start` daemon tools, phase/gate events in Forge, and the
Claude-Code hook system (the natural emitter site). Feeds **Phase D** (the brain is the *judgment* half of the role
split) and **Phase F** (the seat/session model + VPS reach).

---

## Would love to have — NOT the bar

Deliberately out of the MVP. Listed so the decision is *recorded* rather than re-litigated every planning
round. Nothing here blocks "worthy of using"; several are things Marley may never want.

**Post-MVP, wanted, has a ticket**
- **Multi-language support (#315)** — the single cheapest, highest-leverage item on this list, and explicitly
  post-MVP by decision (chad, 2026-07-16). Today `language_id_for` is 5 lines of `Some("rs") => Some("rust")`
  and it gates EVERYTHING — sync, hover, completion, diagnostics, inlay hints all go silently dead on a
  non-`.rs` file. tree-sitter has one grammar. The LSP breadth already built (M20+M21) is language-agnostic and
  would light up for free per grammar + server config. Rust-only is survivable **because Marley is a Rust
  project and dogfoods itself.**
- **Snippets** — completions actively STRIP tab-stops today (`strip_snippet_markers`, depth-tracked). v1 chose
  plain text deliberately.
- **Breadcrumb bar** — a named #330 cut; the sticky header is the partial substitute.
- **FSEvents watcher** — external-change detect is poll-at-interaction (#275/#284), not a watcher thread.
- **Anchors on the caret path** — B4 shipped anchors but the caret does not use them; a concurrent edit stales
  it. Matters only when the brain co-edits (see Phase D).
- **Grapheme-cluster correctness** — ZWJ families over-count, VS16/keycap under-count.

**Customization beyond the MVP's font/theme keys**
- **Settings UI** — deliberately absent; `CommandId(3)` "Open Settings" was REMOVED in M1.C because "an inert
  command was a lie". File-only TOML is the current contract.
- **Keybinding customization file** — none exists; `Keymap::default_bindings()` is hardcoded Rust with
  compile-time uniqueness checks. A user keymap means giving up that check.
- **User themes from file** — `ThemeRegistry::builtin()` is exactly two themes (Marley Light/Dark) + the #199
  live picker. No file loading.

**Probably never — the IDE breadth we are deliberately not chasing**
- **Extensions / plugin host** — VS Code's actual moat. Unwinnable, and chasing it means an extension host + a
  marketplace before the brain ships.
- **Debugger (DAP)** — no hits, no ticket. Reconsider only if the brain needs to drive a debug session.
- **Minimap** — no hits anywhere, docs or code.
- **Vim mode** — the Zed vim doc was mined for its **KeyContext engine** (B1, shipped); the modal editor itself
  was never the goal.
- **Collaboration** — **architecturally excluded, by design.** `anchor.rs` is deliberately CRDT-free: "one
  linear history per buffer; fragments/dense-ordering/logical clocks are a concurrent multi-writer milestone".
  Real collab means the CRDT rewrite B4 explicitly declined.
- **Remote / SSH editing** — `marley_remote` spawns ssh as a terminal PANE; there is no remote FS, no remote
  file editing, no remote LSP. Phase F may revisit for remote runners.

---

## Cross-cutting disciplines (always on)
- **Clean-room §20** — observe the reference (Warp = terminal/cockpit, Zed = editor), reimplement in our own
  code; the reference deconstructions ([../warp_architecture/](../warp_architecture/) + [../zed_architecture/](../zed_architecture/))
  are the *how*, provenance-tagged. The intended license is open-core: GPL/AGPL editor+terminal, proprietary brain.
- **The pipeline** — every ticket runs plan → design → implement → inspect → validate → complete → /commit; pure
  seams cov/MSI 100, shims `mutants::skip` + driven-validated (CONSTITUTION §0–§21).

## Related
[../zed_architecture/subsystems/00-overview.md](../zed_architecture/subsystems/00-overview.md) ·
[../warp_architecture/subsystems/00-overview.md](../warp_architecture/subsystems/00-overview.md) ·
[00-overview.md](00-overview.md) · [../planning/design-notes/zed-editing-discovery.md](../planning/design-notes/zed-editing-discovery.md)
