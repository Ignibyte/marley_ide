---
pipeline_id: 0f37229d-d325-4894-9fba-b8bb3df5d1b3
ticket: forge#337 (169609bd-40e6-4e08-901d-2a86cfc15226) · local docs/planning/tickets/closed/TICKET-337-appearance-font.md
aar_id: d8d283b2-cfda-49e7-a41b-dd8fbfa8e340
status: Phase 5 — Complete PASS
title: Appearance settings — font size (+family) with ⌘= / ⌘− / ⌘0 live zoom
type: feature
milestone: M22
references: [the #330 editor.* settings four-wiring precedent, the #199 live theme picker, the window/pane resize path (PTY re-grid), workspace.rs:122 (the stale-guard lesson), AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001 (#336 — BINDING: no third source of cell width)]
---

<!-- PRE-AUTHORED (the Fable method): written in batch at docs/planning/pipeline/queued/m22-appearance-font.spec.md
     (committed 2f55c8d) from a live read at `main` @ a9eb589, then PROMOTED here by /pipeline:plan, which
     re-verified every cited seam. The Phase 1 verification ledger is in the notes. NOTE: #336 landed at
     df7784b between authoring and promotion, so app.rs line numbers below are STALE BY CONSTRUCTION — the
     ledger carries the current ones. -->

## Method note (run 2 of promote-don't-author)
Phase 1 did not author this spec — it promoted it and **re-verified the cited seams**, which on run 1 (#336)
caught a REQ resting on a seam deleted in #266. The load-bearing check here is the **consumer sweep**: this
ticket replaces a const that other code reads, and a MISSED reader is a silently stale value — the exact class
the spec already names (workspace.rs's guard "already went stale once"). The ledger is the Phase 1 notes.

## Title
The font size is a hardcoded `const TERMINAL_FONT_SIZE: f32 = 13.0` (app.rs:783) — the single const behind
BOTH surfaces (`text_size(px(TERMINAL_FONT_SIZE))` at app.rs:4573/4815/9643/9715/9817/9867; the editor and
the terminal share one mono metric). Nobody changes their font size by editing a Rust const. This ticket
makes it a **setting with live zoom chords**: `appearance.font_size` + ⌘= / ⌘− / ⌘0 — the first thing a
developer reaches for in the first ten minutes ("changing font size, text, themes" — chad). Themes already
have the #199 live picker; user theme FILES stay on the wishlist.

## Scope
### In
- **`appearance.font_size`** (numeric, default 13, clamped **[8, 32]**) via the #330 four-wiring
  (define_setting + `AppliedSettings` + both resolvers + `persist_font_size` + the NON-DEFAULT round-trip
  leg). A hand-edited invalid value (0, negative, NaN, 300) resolves to the clamped/default value — never a
  zero-size layout, never a panic (the settings framework's hand-edited-file posture).
- **`appearance.font_family`** (String, default `""` = the built-in mono). A set family resolves through
  gpui's font system; ~~an UNRESOLVABLE family falls back to the default **with a status flash** (no silent
  wrong font)~~ — **AMENDED at Phase 4: no flash ships.** An unresolvable family degrades to gpui's SYSTEM
  fallback, silently (→ **#344**). Recorded rather than quietly rewritten, because this sentence is the one
  the code contradicted for the whole ticket: it was copied verbatim into `settings.rs`'s doc, inspect
  flagged it (F5), the ledger recorded the fix — **and the fix was never applied.** See the notes.
- **Zoom chords** — ⌘= (+1), ⌘− (−1), ⌘0 (reset to 13): all three FREE (verified against keymap.rs),
  GLOBAL scope (zoom is app-chrome, not editor-buffer), each step clamps, applies live, and persists
  (the #199 picker precedent: a live change writes through). Plus three palette rows ("Increase / Decrease /
  Reset Font Size") — the CommandId trio + completeness tests.
- **The const dies.** `TERMINAL_FONT_SIZE` is replaced by a metrics read off `AppliedSettings` at ONE seam —
  a `font_metrics(applied) -> (font, size, cell_w, cell_h)` recomputed only when size/family change (cache
  keyed on the pair). Every `text_size(...)` consumer and every cell-math consumer (caret x, click x,
  selection bands, gutter width, popup origins, the terminal grid) reads the cached metrics. The
  workspace.rs:122 non-positive guard (which already went stale once — "was a stale 14") reads the SAME
  source, killing that class.
- **PTY re-grid on zoom** — cell size change ⇒ every terminal pane's cols/rows change ⇒ drive the EXISTING
  window/pane resize path (the shell sees SIGWINCH; TUIs reflow). One zoom = one re-grid pass.
### Out (explicitly)
- Per-surface sizes (editor 13 / terminal 12) — wishlist; one shared key v1, matching the shared const it
  replaces. User theme files (wishlist, recorded). A settings UI (deliberately absent — M1.C). Line-height /
  letter-spacing keys. Font ligatures.

## Reference (§20)
**N/A — Marley-specific plumbing** over its own settings framework; ⌘±/⌘0 zoom is a universal OS-level
convention (observed everywhere: VS Code, browsers, terminals), not a copyleft behavior. No source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-ONE-KEY** — one shared `appearance.font_size` for both surfaces (it replaces one shared const; the
  split key is a recorded wishlist item).
- **D-CLAMP-AT-EVERY-DOOR** — the [8,32] clamp applies at settings-resolve AND at the zoom step
  (`zoom_step(cur, delta) -> f32`, a pure fn with the truth table), so no path can smuggle a bad size in.
- **D-ONE-METRICS-SEAM** — a single cached `font_metrics` read; ZERO remaining `TERMINAL_FONT_SIZE`
  literals (a grep gate in validate: the identifier is gone from the tree).
- **D-LIVE-PERSIST** — every zoom step persists immediately (#199's write-through posture), so a crash
  never loses the chosen size.
- **D-REGRID-REUSE** — the PTY resize rides the existing resize path; no second resize mechanism.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | resolve `appearance.font_size` at boot into the shared metrics (both surfaces render at it) | unit + headless |
| REQ-002 | survive a settings round-trip with a NON-DEFAULT size (persist 16 → reload → applied 16) | the round-trip leg |
| REQ-003 | clamp any resolved or stepped size into [8, 32]; 0/negative/NaN/300 from a hand-edited file → the default/clamped value, never a zero-size layout | pure `zoom_step` + resolver table |
| REQ-004 | on ⌘= / ⌘− / ⌘0: apply the new size LIVE (metrics recomputed once; editor cell math + terminal grid both follow) and persist it | headless + unit |
| REQ-005 | re-grid every live PTY on a size change through the existing resize path (cols/rows recomputed; the shell observes the winsize change) | headless/integration |
| REQ-006 | **AMENDED at Phase 4 — see the notes.** apply a resolvable `appearance.font_family`; an unresolvable one degrades to gpui's system fallback **SILENTLY**. ~~fall back to the default family WITH a status flash~~ — **the flash is CUT to a follow-up** | unit (resolver) + mechanism |
| REQ-007 | leave ZERO `TERMINAL_FONT_SIZE` identifiers in the tree (the one-seam metrics read replaced every consumer) | grep gate in validate |
| REQ-008 | expose the three palette rows, each resolving to its verb (the completeness tests extend) | existing completeness tests |

## Phase Plan
P2 confirm the metrics seam placement (where cell_w/cell_h are measured today) + enumerate every
`text_size`/cell-math consumer (the six known sites + workspace.rs:122 + any strays); P3 settings wiring →
the pure `zoom_step` → the metrics seam → chords/palette → the re-grid call; P3.5 critics on the consumer
sweep (a missed hardcoded metric = the workspace.rs:122 class), the clamp totality, and the re-grid path;
P4 tables + headless zoom drive + the grep gate + gate; P5 docs (settings.md key table + editor.md).
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
