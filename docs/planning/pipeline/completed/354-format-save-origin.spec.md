---
pipeline_id: 8b82e0f1-4057-4e8c-85c8-1ab4a99abbe7
ticket: docs/planning/tickets/open/TICKET-354-format-on-save-origin-targeted.md
status: Phase 5 — Complete PASS
title: Format-on-save — origin-targeted save-on-switch (#314 follow-up)
type: feature
milestone: M20
references:
  - docs/planning/pipeline/completed/314-lsp-formatting.spec.md
  - docs/planning/pipeline/completed/314-lsp-formatting.notes.md
  - docs/planning/knowledge/failures.md#BF-claude-deferred-save-fires-against-active-not-origin-editor-001
  - docs/planning/knowledge/prevention-rules.md#PR-claude-async-completion-binds-to-origin-not-reread-active-001
---

## Title
Complete a parked format-on-save against the ORIGIN editor after a mid-format
tab/project switch. #314 parks the ⌘S while the format round-trips; today every
completion path (apply success / settle on Err-stale-Abandoned / the 2 s
deadline) guards on `active_is_origin(key)` and ABANDONS the save when the user
switched away — the origin stays visibly dirty and the ⌘S never completes. The
proper fix: locate the origin editor from the latch key and save THAT editor.
NOTE (Phase-1 discovery): the ticket text predates #397 (editors onto the
registry) and is stale — `locate_open_file` is deleted, `(pi,ti)` no longer
identifies a file, and the "moderate refactor" it anticipated (parameterizing
the active_* conflict machinery) ALREADY SHIPPED: `EditorInstance` carries
`mark_saved`/`arm_save`/`disarm_save`/`set_conflict`/`set_disk`/… with no
notion of "active", and `EditorInstance::root()` hands the owning root for
free. The remaining work is a registry locator + an instance-targeted save
funnel + rewiring the three completion sites.

## Scope
### In
- All three completion paths (`apply_formatting_response` success arm,
  `settle_pending_save`, `check_format_save_deadline`) target the ORIGIN
  editor located by the latch key, not the active editor.
- A by-target save path over the #397 registry: an instance-by-path locator
  (the `buffer_for_open_path_mut` scan generalized to hand back the
  `EditorInstance`, or a `ContentId` bound at park time — design picks) + a
  `save_instance`-style funnel carrying `save_active`'s write + didSave +
  mark-saved + external-conflict (#275/#284) semantics; `save_active` becomes
  a caller of it (one machine, not a fork).
- An instance-scoped external-conflict check (the `check_active_file_external`
  decision table — `extchange::external_action` is already target-agnostic —
  run against the origin instance before its write).
- The success arm reads version/caret from the ORIGIN buffer (stale check
  against the origin, caret re-seat in the origin editor).
- `didSave` routed to the ORIGIN's owning project root host.
- Background-origin conflict safety: a Changed-conflicted origin that is not
  active is never written and never armed by the completion.

### Out (explicitly deferred)
- Cross-project RESPONSE delivery: outcomes for a non-active root are not
  drained until the project is re-activated — that is TICKET-413. For a
  cross-project switch the 2 s deadline (global pump tick) already fires and,
  with this ticket, plain-saves the origin; the response-apply lane across
  projects completes when 413 lands.
- Any change to when format-on-save arms (setting, language, capability
  checks — #314 as-shipped).
- `willSaveWaitUntil` protocol migration (a #314 locked decision, not
  re-opened).
- Save-on-close / dirty-tab-close prompting (closing the origin tab drops the
  parked save; close flows are their own feature).

## Reference (§20)
Zed (the editor reference — same-gpui-stack) + the published LSP spec.
Behavior matched: in Zed and VS Code (published behavior/docs), format-on-save
is a PER-DOCUMENT operation — the save completes on the document that was
saved, regardless of where focus moves while the formatter runs; switching
tabs never loses a requested save. The LSP protocol itself is
document-addressed: `textDocument/formatting` and `textDocument/didSave` carry
the target URI, nothing about them is focus-dependent
(`docs/zed_architecture/subsystems/05-lsp-language-intelligence.md` — the
protocol is a published Microsoft spec; format-on-save is listed among the
per-document LSP features). Marley matches that observable contract: the ⌘S
pressed on file A completes on file A. Clean-room: behavior-level only — no
Zed/Warp source read; the completion/latch mechanism is Marley's own (#314).

### Prior art
1. **Behavior maps** — `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`
   (LSP features are per-document; provenance banner: the protocol + lsp-types
   are public/MIT) and `docs/zed_architecture/crates/language.md`
   (format-on-save is a per-language *setting*, orthogonal to focus). Nothing
   in the maps ties save completion to focus — supporting origin-targeting.
2. **Published material** — the LSP spec: `textDocument/formatting` /
   `didSave` are URI-addressed; `willSaveWaitUntil` is the protocol-canonical
   format-before-save (noted; #314 locked explicit Formatting + latch, not
   re-opened). VS Code / Zed published docs describe format-on-save completing
   on the saved document irrespective of subsequent focus.
3. **Our permissive deps** — none own this seam: gpui (Apache-2.0) is the UI
   runtime, it has no buffer/file-save model; ropey owns rope text, not file
   IO or save targeting; regex / tree-sitter / alacritty_terminal are
   unrelated. The seam is Marley's own #275/#284/#314 app-state machine. The
   real prior art is IN-REPO, and it DISSOLVED the ticket's anticipated
   refactor (a §20-style win): #397's registry already parameterized the
   whole #275/#284 surface — `EditorInstance` owns `mark_saved`/`arm_save`/
   `disarm_save`/`set_conflict`/`set_disk`/`conflict`/`armed_at` with no
   "active" notion, and `EditorInstance::root()` is the owning-root answer;
   `apply_one_file` (#322) already mutates buffers BY PATH across all
   projects via `buffer_for_open_path_mut` (deterministic lexicographic
   tie-break) — the exact pattern the save extends. Checked: no crate we
   ship owns save targeting.

## React-first (parity)
N/A — no UI delta: this changes WHICH buffer a deferred save writes and where
didSave routes — no chrome, overlay, pane surface, layout, type, color, or
affordance changes. The only user-visible surface is the existing global
status flash (already shipped by #314); design may refine its wording for a
background-origin save (text-only, inside the existing flash affordance —
still no new visual surface).

## Locked-In Decisions
- **D-ORIGIN-BIND** — a format-on-save completion acts on the ORIGIN captured
  at park time, NEVER on a re-read of "the active editor"
  (PR-claude-async-completion-binds-to-origin-not-reread-active-001, HIGH).
  The binding is the latch's origin identity — `path_from_file_uri(key.uri)`
  resolved through the registry scan, or a `ContentId` captured at park
  (design picks the mechanism; either satisfies the rule). The cross-target
  race (switch during the round-trip) is traced explicitly in design and
  asserted in tests.
- **D-ONE-WRITE-KEPT** — #314's invariant stands: every completion path TAKES
  `pending_save_after_format` before saving; at most one write per parked
  save; the apply path stays the sole formatted-text writer.
- **D-ONE-MACHINE** — one save/conflict machine: the by-target save funnel
  carries the #275/#284 semantics via the already-target-agnostic
  `EditorInstance` methods (#397), and `save_active` becomes a caller of it.
  No duplicated save/conflict logic anywhere.
- **D-CONFLICT-SAFE-BACKGROUND** — a Changed-conflicted origin that is NOT the
  active editor is never written and never armed by a completion: arming is an
  active-consent gesture (#275's "never overwrite content the user never saw");
  the origin stays dirty + conflicted for the user's return. A `Deleted`
  conflict keeps #275 semantics (the write recreates; no arming needed).
- **D-OWNING-ROOT-DIDSAVE** — the post-save `didSave` notifies the ORIGIN's
  owning project root host, not the active project's.
- **D-CLOSED-ORIGIN-DROPS** — if the origin file is no longer open in any
  editor at completion, the parked save is dropped cleanly (no write, no
  misleading flash, latch cleared).
- **D-NO-UI-DELTA** — no new visual surface; flash wording tweaks only, within
  the shipped flash affordance.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a format-on-save is parked on editor A and the formatting response arrives with A's version unchanged AFTER the user switched to another tab, the system shall apply the edits to A's buffer and complete the parked save by writing A's formatted text to A's path (A marked clean). | headless unit: park on A → switch to B → `push_response_for_test` → drain; assert A's disk content formatted, A clean, latch gone |
| REQ-002 | WHEN a parked format-on-save settles via server Err, Abandoned, stale version, or the 2 s deadline AND the origin editor is open but not active, the system shall plain-save the ORIGIN (its buffer text written, origin marked clean) instead of abandoning. | headless units per path: Err response / stale version / tick past deadline after a switch; assert origin file written + clean |
| REQ-003 | WHILE a completion fires after a switch, the system shall never write any file other than the origin — the active editor's buffer and file stay untouched. | the REQ-001/002 fixtures assert B's disk file + buffer + dirty-state unchanged |
| REQ-004 | IF the origin has a `Changed` external conflict and is not active at completion, the system shall not write the origin file and shall not arm or consume an overwrite license (origin stays dirty + conflicted). | headless unit: seed Changed conflict on A → park → switch → complete; assert disk unchanged, A dirty, not armed |
| REQ-005 | WHEN an origin save completes, the system shall deliver `didSave` to the LSP host of the origin's owning project root. | headless unit (fake_ls / host hook): origin in project P1, active P2 → complete; assert P1's host got didSave, P2's did not |
| REQ-006 | IF the origin file is no longer open in any editor at completion, the system shall drop the parked save without writing any file and clear the latch. | headless unit: park → close A's tab → complete/deadline; assert no write, latch None |
| REQ-007 | WHILE the user has not switched away, the system shall behave exactly as #314 shipped (one write of formatted text on success; plain save on Err/stale/deadline; conflict arm flow on the active editor). | the existing #314 test suite stays green |

## Phase Plan
- **P2 Design** — the by-target save signature + how the #275/#284 machinery
  parameterizes (per-editor methods vs (pi,ti) addressing — use the Explore
  seam map); the origin-located completion flow per path; caret re-seat in a
  background editor; flash wording; the regression test plan incl. mutation
  kills (mind the skip-detach trap — save_active area is `mutants::skip`
  shim country).
- **P3 Implement** — code per design; `cargo mutants --list -f app.rs` after
  any edit near a skip attribute (strike-six rule).
- **P3.5 Inspect** — independent critics vs the diff; trace the cross-target
  race + conflict matrix adversarially; fix real findings.
- **P4 Validate** — write + RUN tests; gate green (`--diff`).
- **P5 Complete** — archive, ledger capture (§19), close the ticket.
