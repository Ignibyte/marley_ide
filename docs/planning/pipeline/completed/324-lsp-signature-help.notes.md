# LSP signature help (parameter hints) — Notes

- **Forge ticket:** #324 aa432c80-2bf4-40f9-acb5-e02f99416208
- **AAR:** ffda3cbe-8baa-49fc-8ea1-c5dbc5661fe4
- **Local ticket doc:** docs/planning/tickets/open/TICKET-324-lsp-signature-help.md
- **Pipeline spec:** 324-lsp-signature-help.spec.md

## Phase 1 — Plan
- **Request:** `/work 324` — LSP signature help: a card above the caret showing the call's signature with
  the active parameter lit, updating as you type commas, gone when the call closes.
- **Classification / tier:** work pipeline (a small, mostly-INTEGRATION feature — "the #313 park-and-consume
  shape with a card instead of a menu", per the forge ticket). Systems: server/ui (editor overlay) +
  marley_lsp (a new pure parse seam + the handshake capability).
- **Forge recall (§18.3):** bulletins none. Sprint #32 (M19) still active (M20/M21 tickets carry
  `sprint_id: null` — the one-active-sprint rule; no blocker). knowledge-search surfaced the
  prevention-rule cluster; the load-bearing ones this ticket rides:
  - **`PR-claude-lsp-advertise-client-capability-001`** (MY fresh #323 finding) — the crux carry-forward:
    signature help's `[start,end]` label-offset form + the per-signature active param are ONLY sent if the
    client advertises `textDocument.signatureHelp` with `labelOffsetSupport` + `activeParameterSupport`. A
    mocked-wire test can't catch the omission — the live drive must confirm the offset form arrives (REQ-008).
  - `PR-claude-suppression-test-needs-a-positive-control-001` — the string/comment gate needs an A/B control.
  - `PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001` — dismiss via the pump poll.
  - `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` — inherit the request guards.
  - `PR-claude-new-overlay-register-at-every-choke-point-001` — register the card at every overlay choke.
- **Discovery (the edit surface for Design):**
  - REUSE: `editor_complete.rs::popup_origin` (275 — the flip math; #324 wants a prefer-ABOVE variant),
    `completion::trigger_characters` (89 — the caps-reading idiom; sibling `signature_trigger_characters`),
    `editor_complete.rs::CompletionKey` (24 — the `SignatureKey` shape), the `complete.rs` wrapping-nav
    idiom, the #311 hover-card overlay recipe, the #310 string/comment suppression gate, the ONE drain
    `consume_lsp_responses`, `RequestPurpose` on `lsp_host.rs`, the handshake in `marley_lsp::handshake`.
  - NEW: `marley_lsp/src/signature_help.rs` (pure parse + params + caps + trigger chars),
    `marley_app/src/editor_signature.rs` (pure `SignatureKey` + `SignatureCard` + prefer-above origin),
    the app.rs shim (park/consume, render, drain arm, ⌘⇧Space, dismiss poll), `handshake.rs` (the capability),
    `keymap.rs` (⌘⇧Space), the `lib.rs` re-exports.
- **Decisions:** D1 reuse #313 shape whole; D2 prefer-above card (no collision with completion below);
  D3 advertise textDocument.signatureHelp (the #323 rule); D4 clamp both indices + per-sig override; D5
  label string-or-offset normalize, OOB drop; D6 documentation body OUT v1. (Full text in the spec.)

## Phase 2 — Design

### Architecture / approach
Signature help is a near-exact analog of #313 completion with two deltas: it renders a CARD (the #311
overlay recipe) not a menu, and it prefers ABOVE the caret. Data flow = the shipped #308–313 spine: a
typed trigger char (`(`/`,`) PARKS a `textDocument/signatureHelp` request through the SAME typing hook +
the `editor_complete::in_string_or_comment` suppression gate; the pump DRAINS the answer next tick
(`consume_lsp_responses` + a new `RequestPurpose::SignatureHelp` arm); a supersede + focused-file guard
admits it; the pure parser normalizes it; a card renders above the caret. Dismissal is the live-identity
pump poll (the #311/#313 idiom), never the typing hooks. §14: typed, no panics on the response path (the
parser CLAMPS indices + DROPS bad label offsets, never indexes blindly); the pure seams are gpui-free +
`char`-offset based; no process spawn (the wire is #308's). §20 reconfirmed — Zed/VS Code observed
behavior only; Marley composes its OWN card (the #221 frame + #311 recipe + the #313 flip) over the
published LSP 3.17 wire. No copyleft source read.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/signature_help.rs` (NEW) | PURE: `SignatureHelp`/`SignatureInfo`/`ParamInfo`; `parse_signature_help` (clamp BOTH indices, per-sig `activeParameter` override, label string-or-`[start,end]`-offset normalize + OOB/inverted drop); `signature_trigger_characters` (triggers + retriggers off `signatureHelpProvider`); `signature_help_params(uri,pos,context)`; `signature_help_support(caps)`. |
| `crates/marley_lsp/src/lib.rs` | re-export the new public items (NOT any private helper — the #311-F3 rule). |
| `crates/marley_lsp/src/handshake.rs` | advertise `textDocument.signatureHelp` (`labelOffsetSupport` + `activeParameterSupport` + `contextSupport`); extend `initialize_params_are_honest`. |
| `crates/marley_app/src/editor_signature.rs` (NEW) | PURE: `SignatureKey{uri,line,character,version}` (= `CompletionKey` shape); `SignatureCard{signatures, active_sig}` with `active()`, `active_param_range()`, `cycle(delta)` wrapping, `pip()` (N of M); `signature_card_origin(anchor_y,cell_h,card_h,win_h)` — prefer-ABOVE, flip-below. |
| `crates/marley_app/src/lib.rs` | module decl + re-exports. |
| `crates/marley_app/src/app.rs` | shim: `signature_card`/`signature_request` fields; park on a typed `(`/`,` (gated by `in_string_or_comment`), `)` dismiss; ⌘⇧Space manual invoke; the drain arm; `apply_signature_help_response` (supersede + focused-file guard → parse → card); the card key handler (Esc dismiss; ↑/↓ cycle when >1 sig, else DECLINE); `dismiss_stale_signature` in the pump poll; `signature_overlay` render (above the caret, active param in `accent`, the pip); `signature_card=None` at every overlay choke; test hooks. All shims `#[cfg_attr(test, mutants::skip)]`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::SignatureHelp(SignatureKey)`; `signature_help_support()` + `signature_trigger_chars()` accessors (+ the sibling `mutants::skip` — the #322 lesson). |
| `crates/marley_app/src/keymap.rs` | ⌘⇧Space → `"signature-help"` Editor-scoped; roster bump (guard tests). |
| `crates/marley_app/src/headless_drive.rs` | the drives (below). |

### Regression Test Plan (≥1 per AC)
| REQ | Test | Kind |
|---|---|---|
| REQ-002 | `parse_clamps_and_overrides` — activeSignature/activeParameter OOB → clamped; per-sig `activeParameter` overrides top-level | pure unit (`signature_help.rs`) |
| REQ-003 | `parse_label_string_and_offset` — string label → full-range highlight; `[start,end]` → that range; OOB/inverted pair DROPPED (no panic) | pure unit |
| REQ-002/003 | `parse_degrades_on_junk` — Null/`{}`/empty signatures → empty, never panic | pure unit |
| REQ-001 | `signature_help_params_shape` — `context{triggerKind,triggerCharacter?,isRetrigger}` + position | pure unit |
| REQ-001 | `signature_trigger_characters_reads_caps` — triggers + retriggers off caps; junk → empty | pure unit |
| REQ-008 | `signature_help_support` table + `initialize_params_are_honest` asserts the three sub-caps | pure unit |
| REQ-005 | `signature_card_origin_prefers_above` — above-first; flip-below only when it won't fit up top; pinned-0 fallback | pure unit (`editor_signature.rs`) |
| REQ-005/007 | `card_cycle_pip_and_key` — `cycle` wraps, `pip` = (i+1, len), any-field `SignatureKey` mismatch (incl. version) | pure unit |
| REQ-001/005 | `sig_help_parks_and_shows_card_headless` — type `(` → park → consume → card open, active param range set | headless drive |
| REQ-004 | `sig_help_suppressed_in_comment_headless` — `(` in a comment sends NOTHING; the same `(` in code DOES (A/B positive control) | headless drive |
| REQ-006 | `sig_help_dismiss_and_retrigger_headless` — caret-exit + Esc dismiss; `,` retriggers a fresh request | headless drive |
| REQ-007 | `sig_help_stale_dropped_headless` — a moved-caret / bumped-version answer dropped | headless drive |
| REQ-001/005/008 | LIVE tee drive: type `s.push_str(` in the fixture → card shows the signature with the param lit; CONFIRM the `[start,end]` OFFSET-label form arrives (REQ-008); the A/B comment control. Env-permitting; else units + the #323-proven mechanism (`PR-claude-selftest-locked-screen…`). | live |

Uncoverable-without-live: the OFFSET-label form only arrives from a real server that honors
`labelOffsetSupport` — the pure test exercises BOTH label shapes, but only the live drive proves rust-
analyzer actually sends the offset one (the REQ-008 crux; the #323 lesson).

### Risks / decisions
- **D-arrows (↑/↓ layering)**: the card consumes ↑/↓ ONLY when `signatures.len() > 1` (cycle overloads);
  with a single signature it DECLINES so the arrows move the caret → the poll dismisses. Registered AFTER
  the completion popup in the key router (completion wins ↑/↓ when both are open). Esc always dismisses.
- **D-above**: a NEW `signature_card_origin` (NOT a new param on #313's shared `popup_origin`) so #313 is
  untouched; the inverse asymmetry is what guarantees completion (below-first) + signature (above-first)
  can't collide.
- **D-capability (the crux)**: advertise `textDocument.signatureHelp` — the live drive must confirm the
  OFFSET-label form arrives, else `parse_label_string_and_offset` passes on a shape the real server never
  sends (the #323 F-VAL-1 lesson, applied preemptively).
- **Suppression-gate caveat**: reuses `in_string_or_comment` over the RENDER-refreshed highlight cache —
  same one-tick-deferred / fall-OPEN-on-miss behavior #313 documented (a `(` in a comment during the race
  asks — harmless, Esc-dismissable — never a miss in code).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` + `cargo clippy --lib` clean (the only warnings are the
`--all-targets` "test hook never used" ones — validate writes the tests that consume them, the #323 shape).

- **`marley_lsp::signature_help`** (pure): `parse_signature_help` (drops a label-less signature; clamps
  `activeSignature`/`activeParameter`; per-sig `activeParameter` overrides top-level; `param_range`
  normalizes a string label via substring-find OR a `[start,end]` UTF-16 pair via `char_col_from_column`,
  dropping inverted/OOB to `None`); `signature_trigger_characters` (trigger + retrigger); `signature_help_
  params` (context triggerKind/triggerCharacter/isRetrigger); `signature_help_support`. Re-exported.
- **`handshake.rs`**: advertises `textDocument.signatureHelp` (labelOffsetSupport + activeParameterSupport
  + contextSupport); the honest test asserts the three sub-caps + the now-2 textDocument caps.
- **`marley_app::editor_signature`** (pure): `SignatureKey` (= CompletionKey shape); `SignatureCard`
  (`active`/`active_param_range`/`multi`/`cycle`-wrapping/`pip`); `split_label` (before/param/after for the
  accent highlight — OOB → whole label in `before`); `signature_card_origin` (prefer-ABOVE, flip-below).
- **`app.rs` shim**: fields + `OpenSignature{card, anchor_path, anchor_row}`; `park_signature_trigger` (in
  the typing hook, twin of park_completion) + `consume_signature_trigger` (pump, one tick later — gate via
  `caret_in_string_or_comment`, trigger/retrigger via the caps); `send_signature_request` +
  `request_signature_help_manual` (⌘⇧Space); the `SignatureHelp` drain arm → `apply_signature_help_
  response` (supersede + focused-file guard → parse → card; empty → dismiss); `dismiss_stale_signature`
  (pump poll — caret leaves the anchor ROW or the file switches); `handle_signature_key` (Esc dismiss; ↑/↓
  cycle ONLY when `multi`, else decline — placed AFTER completion in the router); `signature_overlay`
  (caret-anchored, prefer-above, active param in `accent`, the pip) + its render-tree hookup; the
  launcher-choke clear; test hooks.
- **`lsp_host.rs`**: `RequestPurpose::SignatureHelp`; `signature_help_support()` + `signature_trigger_
  chars()` accessors (+ sibling mutants::skip); `set_caps_for_test`.
- **`keymap.rs`**: ⌘⇧Space → `signature-help` Editor-scoped; roster 56→57, scoped 15→16.

### Deviations from design (with reason)
- **`split_label` added to `editor_signature.rs`** (beyond the manifest) — the render's label-slicing is
  pure + worth a test, so it lives in the tested seam, not the masked overlay shim.
- **`set_caps_for_test` (lsp_host) + `set_signature_caps_for_test` (app)** — a NEW test hook the manifest
  didn't name. It injects negotiated caps into a process-less host so the CAPABILITY-gated trigger path
  (`signature_help_support` / the trigger chars) is drivable HEADLESSLY — the #323 resolve chain couldn't
  do this, so REQ-001/004 lean less on the flaky live drive. (The string/comment gate still reads the
  render-populated syntax cache, so the A/B suppression proof may still want the live drive — decided at
  validate.)
- **Label offsets hardcode UTF-16** (Marley's advertised encoding) rather than threading `enc` — the
  offsets are always in the negotiated encoding we advertise.
- **`activeParameter` OOB clamps to the LAST param** (the design's "clamp") rather than the spec's
  "default 0" — both never-panic; flagged for inspect to confirm the choice.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect
Three parallel critics (correctness / reuse+guards+provenance / state+layering) + my own pass. Strong
convergence — no panic paths (every index/slice guarded), but a cluster of display-integrity findings on
the PASSIVE-card interaction model. Post-fix: `cargo check` + `cargo clippy --lib` clean; workspace
1317/1317, no regression.

### REAL — fixed in #324
| # | Sev | Finding | Fix |
|---|---|---|---|
| **F-ARROWS** | MED | `handle_signature_key` matched `up`/`down` on the KEY only — a multi-signature card swallowed ⇧↓ (shift-select), ⌘↑/⌘↓ (doc start/end), contradicting its "passive" design. | Gate the ↑/↓ arm on a BARE arrow (`!shift && !platform && !alt && !control`); a modified arrow declines → falls through to the editor. |
| **F-STALE** | MED | The card anchored to the LIVE caret and `dismiss_stale_signature` was ROW-only, so it could open on the wrong row (Enter right after `(`) and GLIDE along the line on same-row caret moves. | `apply` drops the answer when the live caret row ≠ the request row (`key.line`); `OpenSignature` now stores `anchor_col` and `signature_overlay` anchors at the FIXED `(anchor_row, anchor_col)` — the card stays above the call instead of sliding with the cursor. |
| **F-CHOKE** | MED | `signature_card` cleared only at the launcher choke — a stale card floated over + ATE keys for the palette / finder / history / find (whose key arms run AFTER the card's, the #323-SELF-1 class). | Clear `signature_card` at the TOP of `dispatch_action` (except `"signature-help"`), one site covering them all; the inline launcher clear stays for the icon-click path that bypasses the router. |
| **F-DECISION** | MED | The trigger/retrigger fire-or-not DECISION was inlined in a `mutants::skip` shim → its mutants (`\|\|`→`&&`, `&& open` deletion) survive by construction. | Extracted the pure `should_request_signature(typed, triggers, retriggers, open)` into `signature_help.rs`; the shim only routes. Validate unit-tests it. |
| **F-ESC** | LOW | Esc DURING the round-trip (before the card showed) didn't cancel the in-flight request → the late answer reopened a card the user preempted. | Esc clears BOTH `signature_card` + `signature_request`; the router arm fires on `card.is_some() \|\| request.is_some()`. |
| **F-CLAMP** | LOW | An out-of-range `activeSignature`/`activeParameter` clamped to the LAST index, not the LSP-3.17 "defaults to 0". | Reset to 0 (`clamp_or_zero`). The per-signature-overrides-top-level precedence was already correct. |
| **F-DOCS** | LOW | Docs claimed a mid-surrogate `[start,end]` pair yields `None` (it SNAPS to the next char via the position bridge); the `,`-is-a-retrigger comment was inaccurate (`,` is a TRIGGER — fires unconditionally, benign off a non-call). | Corrected the module + `param_range` + `consume_signature_trigger` docs; also documented the string-label first-substring heuristic. |

### REAL — accepted / deferred (with reason)
- **HIGH "the two pure seams have zero tests"** (reuse critic, self-caveated) — EXPECTED at inspect: the
  wired-but-unused test hooks ARE the pre-validate handoff shape. Validate adds the sibling-equivalent
  cov/MSI-100 suites (the parse clamps/label-normalize, `SignatureCard` cycle/pip, `signature_card_origin`
  boundaries, `split_label`, `should_request_signature`, the `SignatureKey` version guard).
- **LOW string-label first-substring mis-highlight** (2 critics) — accepted: rust-analyzer (a
  `labelOffsetSupport` client) sends the OFFSET form; the string path is a rare fallback and the naive
  `find` matches VS Code's `indexOf`. Documented in `param_range`.
- **LOW overlay overlap at the window's vertical extremes** when BOTH completion + signature are open —
  accepted: visual-only, needs both-open AND a screen edge; forcing opposite sides couples the two
  overlays. A follow-up if it ever bites.
- **LOW same-row horizontal caret move keeps the card** (residual of F-STALE) — accepted for v1: the card
  stays on the call line (VS-Code-like); precise bracket-span dismissal (Home / past-`)` on the same row)
  is a follow-up.

### VERIFIED CLEAN (independently, by the critics)
- **Capability honesty** — `labelOffsetSupport` → the `[start,end]` arm, `activeParameterSupport` → the
  per-sig override, `contextSupport` → the sent `context`; nothing advertised is unimplemented (the #323
  lesson applied correctly + the 2-cap honesty guard pinned).
- **`mutants::skip` placement** — both new `lsp_host` accessors + all 8 app shims carry it; the 2 pure
  files carry none (the missing-skip trap avoided).
- **Guard inheritance** — supersede + focused-file faithful to `apply_completion_response`/`apply_code_
  action_response`; `signature_request` cleared right after supersede.
- **Router order + `text_input_blocked`** — completion wins ↑/↓/Enter/Esc when both are open; the card is
  correctly ABSENT from `text_input_blocked` (passive — args type through).
- **Pump-poll order** — no same-tick dismiss of a fresh card; a parked request is never poll-dismissed.
- **Provenance** — clean-room LSP 3.17, no `lsp-types` structural fingerprint, no secrets.

### Prevention rules recorded
- **`PR-claude-passive-overlay-declines-mods-clears-at-choke-001`** (prevents `BF-signature-passive-
  overlay-eats-keys-and-floats-001`) — a PASSIVE overlay (one that doesn't own the keyboard) must (a)
  DECLINE modified variants of its nav keys (bare ↑ cycles; ⇧↑/⌘↑ are the editor's) and (b) clear at the
  shared `dispatch_action` choke, not one inline site — else it eats gestures / floats over modals whose
  key arms run after it.
- **`PR-claude-caret-anchored-card-fixes-its-anchor-and-drops-on-line-change-001`** (prevents `BF-
  signature-card-anchors-to-moved-caret-001`) — a card opened from an ASYNC answer must snapshot a FIXED
  anchor at open + drop the answer when the caret left the request's line, or it glides with the cursor
  and lingers past the call. The guard must be a LINE check, NOT a version check (the card persists across
  arg-typing, which bumps the version with no new request).

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests written (24 new, all green; coverage 100% + mutation MSI 100% on the pure seams)
- **`marley_lsp::signature_help` — 11 pure units**: `parse_clamps_and_overrides` (OOB→0 not last, per-sig
  override), `parse_drops_labelless_signature`, `parse_label_string_and_offset` (offset range, first-
  substring, inverted/OOB/junk→None), `parse_label_offset_non_ascii` (🦀 UTF-16→char), `parse_degrades_
  on_junk`, `clamp_or_zero_boundary` (==len→0 kills `<`→`<=`), `should_request_signature_table` (trigger
  always / retrigger only-when-open — kills `||`→`&&`), `signature_help_params_shape`, `signature_trigger_
  characters_reads_caps`, `signature_help_support_table`, `active_param_range_reads_active`.
- **`marley_app::editor_signature` — 6 pure units**: `card_new_active_and_pip` (the single load-bearing
  clamp — removing it panics the test), `card_cycle_wraps`, `card_active_param_range`, `split_label_bounds`
  (offset + OOB/inverted/None→whole-label + non-ASCII), `signature_card_origin_prefers_above` (above/below/
  pin-0), `signature_key_version` (any field incl. version differs).
- **`marley_app` headless drives — 7**: `sig_help_shows_card_and_active_param` (REQ-005), `…suppressed_in_
  comment` (REQ-004 A/B positive control — the syntax cache populates on open, so the gate is headless-
  testable via consume's return), `…dismiss_on_move_and_esc` (REQ-006), `…stale_response_dropped`
  (REQ-007), `…row_guard_drops_moved_response` (inspect F-STALE), `…declines_modified_arrows` (inspect
  F-ARROWS — bare ↓ cycles, ⇧↓ declined), `…cleared_by_palette_open` (inspect F-CHOKE).
- One consolidation for MSI: `SignatureCard::active()` had a redundant defensive `.min` alongside `new()`'s
  clamp (un-killable mutant) → removed; `new()`'s clamp is now the single load-bearing one (the card test
  panics without it). `cargo nextest -p marley_lsp signature_help` 11/11; `-p marley editor_signature
  sig_help` 13/13; workspace 1341/1341.
- Uncoverable headlessly (stated): the actual WIRE send — `send_signature_request`'s `lsp_position_for`
  needs a didOpen'd/tracked file a process-less host doesn't have, so the headless drive proves the send
  DECISION (consume returns true); the real send + payload is the live drive.

### LIVE DRIVE (real rust-analyzer) — full success
Drove the bundled release `Marley.app` (screen unlocked, AX granted) on the reused `ca-fixture` cargo
project repointed to a signature scenario (`let mut s = String::new();  s.push_str`). Seeded `workspace.
shell` (V= editor tab); chad's `~/.marley` backed up + restored.
- **rust-analyzer ready**, caret at the end of `s.push_str`. **Typed `(`** → the card rendered **ABOVE the
  caret** (prefer-above working, not covering the args) showing **`fn push_str(&mut self, string: &str)`
  with `string: &str` HIGHLIGHTED in accent** (the active parameter) — captures `sig-02`. This is the
  REQ-001 (trigger→send) + REQ-005 (card above + active-param highlight) + the offset-label parse path
  (REQ-008) proven live, the parts no headless render can show.
- **Typed `"`** (an arg) → the card **PERSISTED and stayed PUT** — still `push_str(…string: &str)`, same
  fixed position, not gliding with the caret (capture `sig-03`). This proves REQ-006 (persistence across
  arg-typing on the same line) AND the inspect **F-STALE fix** (the fixed `anchor_col`) live.
- Note on REQ-008: unlike #323's codeAction (dead without its cap), rust-analyzer serves signatureHelp
  regardless of the client cap — advertising `textDocument.signatureHelp` UPGRADES the reply (offset labels
  + per-sig active param) rather than enabling it. The precise per-parameter highlight is consistent with
  the offset form; the pure test exercises BOTH label shapes.

### Gate — GREEN [diff]
`scripts/gates.sh --diff` → **`GATE GREEN [diff]`, 15 passed / 0 failed** (the flaky PTY coverage deadlock
did not recur; the stall-watchdog was on standby). gate:4 coverage **100% lines**, gate:5 mutation **MSI
100%** (the pure signature_help + editor_signature seams), miri + visual green. One extra loop: rustfmt
flagged the new test code (whitespace only — no coverage/mutation change), fixed + re-run. Receipt written.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **Docs (§21)**: CHANGELOG `[Unreleased]/Added` entry for #324; `docs/marley_architecture/editor.md` (the
  #324 SHIPS passage — prefer-above card, passive-overlay, the capability sibling) + `crate-map.md` (the
  `signature_help.rs` seam + the SHIM seam-list) updated.
- **Knowledge (forge §19)**: AAR `ffda3cbe…` submitted (completed, effectiveness 5, 4 novel findings). The
  two inspect failures + two prevention rules recorded at Phase 3.5: `BF-signature-passive-overlay-eats-
  keys-and-floats-001` / `PR-claude-passive-overlay-declines-mods-clears-at-choke-001`, and `BF-signature-
  card-anchors-to-moved-caret-001` / `PR-claude-caret-anchored-card-fixes-its-anchor-and-drops-on-line-
  change-001`. forge #324 closed (done).
- **Lessons**: (1) a mostly-INTEGRATION ticket (the #313 shape) still hides a CLUSTER of passive-overlay
  interaction bugs the 3 critics caught — modified-arrow swallowing, stale-card-over-modals, caret-gliding,
  the in-shim trigger decision. (2) The live drive was cheap + decisive (the #323 harness reused, fixture
  repointed) and proved the two things headless can't: the card RENDERS above with the active param lit,
  and it PERSISTS/stays-put on arg-typing (the F-STALE fix). (3) The #323 capability lesson generalizes but
  its FORCE varies — codeAction was DEAD without the cap; signatureHelp only UPGRADES (offset labels).
- Ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.

status: Phase 5 — Complete PASS
