---
pipeline_id: 12fce57f-1c29-4455-8660-c4a89b8fce1e
ticket: forge#201 (419815b7-21ae-4c5d-998f-ab8db82d4f57) · local docs/planning/tickets/open/TICKET-201-warp-tab-titles.md
aar_id: cc1903eb-0092-4302-848b-38da214b7309
status: Phase 5 — Complete PASS
title: cwd-aware tab titles — the CWD-basename fallback tier
type: feature
milestone: M12.2
references: []
---

## Title
Complete command-aware tab titles (forge #201). The rename editor, the persisted `custom_title`,
and the running-command basename derivation ALL already exist (#177 / #157). The one missing tier
of the ticket's `custom → command → cwd basename → generic` chain is the **CWD basename**: an
unnamed tab with no running command currently shows the static "terminal N" instead of its working
directory's last path component. Extend the pure `display_title` with a cwd-basename tier (reusing
the existing `pwd_label`) and wire the pane's shell-reported pwd into `live_tab_title`.

## Scope
### In
- Extend the pure `display_title` (titlebar.rs) with a **cwd tier** between the command-basename tier
  and the static fallback: `custom (non-blank) → command program basename → cwd basename → fallback`.
  The cwd basename reuses the existing `prompt::pwd_label` (the last non-empty path segment); a
  `None`/empty cwd skips the tier.
- Wire `live_tab_title` (app.rs) to gate the command on `session.is_command_running()` (so an idle
  tab reverts to cwd) and read the live cwd from `session.current_prompt().pwd`, passing both into the
  extended `display_title`.
- Exact-value unit tests for the full precedence chain + each tier's None/blank → next-tier fall-through
  (cov/MSI 100 on the pure seam), against the real `cargo mutants --list` set.
- Driven capture: an unnamed no-command tab shows its cwd basename; a running command still wins;
  a custom rename still wins and persists (re-verify the #177 flow is intact).

### Out (explicitly deferred)
- The double-click rename editor, `custom_title` persistence, and command-basename derivation — all
  already shipped (#177 / #157); UNCHANGED here.
- **Title truncation / a hard char cap.** The ticket text mentions "trims/limits length"; the tab
  rail already clips visually at the render layer, and custom titles + command tokens are already
  rendered un-truncated with no complaint. A hard character cap is a distinct behavior (its own AC +
  tests) → a follow-up if the rail actually overflows, not this slice.
- Command emphasis / styling of the title.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Extend `display_title`, do not mint a parallel `derive_tab_title`.** The ticket names a new
  `derive_tab_title(custom, running_cmd, cwd: &Path) -> String`, but `display_title(custom, command,
  fallback)` already exists at cov/MSI 100 with the custom+command tiers built and tested. Extending
  it (add a `cwd` param) reuses the proven seam instead of duplicating it. The design picks the exact
  refactor (likely extracting the shared command-token extractor so `rail_tab_title` and
  `display_title` don't duplicate the `split_whitespace().next()` logic).
- **D2 — `cwd: Option<&str>`, NOT `&Path`.** The shim's pwd is a shell-reported `Option<String>` and
  the existing `pwd_label` takes `&str`; a `&Path` would force an awkward conversion of a string that
  is not necessarily a canonicalizable local path. `Option<&str>` + `pwd_label` is the clean reuse;
  `None`/empty skips the tier.
- **D3 — Keep the existing `fallback` param ("terminal N"); do NOT hardcode `'shell'`.** The ticket's
  final "else 'shell'" is the conceptual generic-default tier. Marley's generic default is already the
  per-tab `row.label` ("terminal N"), which disambiguates multiple unnamed tabs strictly better than a
  single constant. The ultimate fallback only fires in the brief pre-first-prompt window before a pwd
  is known, so "terminal N" is both correct and better than 'shell'. Honors the ticket's intent, keeps
  the better label.
- **D4 — Reuse `prompt::pwd_label` for the cwd basename** (`pwd.rsplit('/').find(|s| !s.is_empty())`).
  No new path logic.
- **D5 — Clean-room (§20).** Behavior-only parity with Warp's cwd-in-tab convention; no Warp
  source/assets consulted.
- **D6 — The command tier gates on `session.is_command_running()`** (the #40/#193 foreground signal).
  Design-phase refinement: the forge ticket says *"exit → reverts to cwd"*, so only a RUNNING
  foreground command owns the title; an idle tab drops to the cwd tier. Without this, an idle tab keeps
  showing the last finished command and the cwd tier would almost never fire. Refines #157's "latest
  command" → "running command; idle → cwd" (the Warp-parity behavior). Shim-only; the pure seam is
  unchanged.
- **D7 — The cwd is read from `session.current_prompt().pwd`** (the live staged cwd that tracks `cd`),
  NOT the last block's `prompt.pwd` (stale after a `cd`). Corrects the Phase 1 note; matches the
  tab-completion path (app.rs:2740).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a tab has no custom title and no running foreground command, the system shall display the **cwd basename** (the last non-empty path segment of the live shell-reported pwd). | unit: `display_title(None, None, Some("/Users/x/Projects/Marley"), "terminal 1")` == `"Marley"`; driven: an idle no-command tab in `~/Projects/ignibyte/Marley` shows `Marley`. |
| REQ-002 | WHEN a tab has no custom title but a **running** foreground command, the system shall display the **command's program basename** and NOT the cwd; on the command's exit it shall revert to the cwd. | unit: `display_title(None, Some("vim ."), Some("/Users/x/Marley"), "t")` == `"vim"`; driven: run `vim` → title `vim`; exit → reverts to `Marley` (the `is_command_running` gate). |
| REQ-003 | WHEN a tab has a non-blank custom title, the system shall display **it** regardless of command or cwd. | unit: `display_title(Some("box"), Some("vim"), Some("/x/Marley"), "t")` == `"box"`; driven: double-click → rename → the custom title shows + persists (the #177 flow). |
| REQ-004 | WHEN a tab has no custom title, no running command, AND no/empty cwd, the system shall display the **static per-tab fallback** ("terminal N"). | unit: `display_title(None, None, None, "terminal 1")` == `"terminal 1"` AND `display_title(None, None, Some(""), "terminal 1")` == `"terminal 1"`. |
| REQ-005 | WHEN the shim renders a tab title, the system shall pass the pane's **live** pwd (`session.current_prompt().pwd`) as the cwd and gate the command on `is_command_running`, so an idle tab reflects its directory. | review: `live_tab_title` reads `current_prompt().pwd` + gates command on `is_command_running` + passes both to `display_title`; driven REQ-001/002 capture. |

## Phase Plan
- **P2 Design** — pick the exact `display_title` refactor (extract the shared command-token extractor so
  `rail_tab_title` stays DRY); confirm the `live_tab_title` pwd read mirrors app.rs:2066; run
  `cargo mutants --list -f titlebar.rs` for the real mutant set behind the test matrix.
- **P3 Implement** — extend `display_title` (+ the extractor), update the `rail_tab_title` call sites if
  the extractor changes them, wire `live_tab_title`'s pwd read.
- **P3.5 Inspect** — critics vs the diff (precedence correctness, the None/empty-cwd guard, the
  wiring's pwd source, DRY reuse); fix real findings.
- **P4 Validate** — write the exact-value matrix + RUN it; driven captures for REQ-001/002/003; gate green.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR capture; archive; close the ticket.
