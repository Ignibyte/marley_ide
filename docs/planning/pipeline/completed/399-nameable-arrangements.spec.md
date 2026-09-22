---
pipeline_id: 29dc884e-ea8f-4b3a-992a-8159469ba47c
ticket: forge#399 (d7b3b391-534f-4f39-93c0-033c408f55ab) · local docs/planning/tickets/open/TICKET-399-nameable-arrangements.md
aar_id: b6ede482-6e0b-4a5f-93b3-efea59198e2c
status: Phase 5 — Complete PASS
title: Nameable arrangements — the [[panes]] settings table (the #388 slice-6)
type: feature
milestone: M28
references:
  - docs/marley_architecture/pane-composition-model.md
  - crates/marley_app/src/settings.rs
  - crates/marley_app/src/workflows.rs
  - crates/marley_app/src/grid_layout.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/app.rs
  - crates/marley_settings/src/manager.rs
---

## Title
The #388 train slice-6 (pane-composition-model.md Q4/Q5; HARD after #398 — needs slice-5's
add-to-pane + the `ContentId`-derived Pane-row labels; #396/#397 transitively): arrangements become
nameable, persistable objects. Today the Panes section is fully DERIVED — `rail_rows` emits one
"PANE {n}" Arrangement row per multi-cell tab, a 1-based per-project counter that renumbers on
close, "no stored identity" by its own comment (tabs.rs:844-857); the row render is read-only
(app.rs:16515-16559). The work, four pieces on shipped seams:
- **The naming verb** — "make a Pane"/"name this arrangement", reusing the #204 inline-draft idiom
  verbatim in shape: a draft field (the `naming_workflow: Option<(String, String)>` pattern,
  app.rs:171), a key-ladder branch (Enter commits trimmed/non-empty, Esc cancels, draft owns the
  keyboard + `stop_propagation` — :15048-15104), a line in `text_input_blocked` (:9488 — the #267
  "a NEW overlay only needs a line here" choke point), the #378 F3/F4 two-text-owners exclusion
  (:7078), cleared by `close_transient_overlays` (:6739), a centered framed card (:18213-18239).
  No new modal machinery. Entry candidates (P2 settles the exact set): a palette verb targeting the
  active multi-cell tab (flash when it isn't one — the #204 blank-guard stance, :3278-3279) and/or
  the #177 double-click-to-rename idiom on the Arrangement row (which #390 left read-only).
- **The `[[panes]]` settings table** — a `define_setting!(PaneArrangements: Vec<PaneArrangement>,
  "panes")` on the shipped `marley_settings` framework, the exact #204 `[[workflows]]` template
  (settings.rs:75-79) with its full tolerance discipline: round-trip identity, absent/malformed →
  empty default (no boot crash), `#[serde(default)]` on optional keys so one hand-edited entry
  never drops the rest (the workflows.rs:17-21 F3 lesson). An entry stores name + scope root +
  shape (grid_layout's split algebra) + per-cell stable content KEYS — never a `ContentId`.
- **Named rail rows** — the `rail_rows` Panes branch (tabs.rs:843-877) shows the arrangement's
  name where a name is bound; unnamed multi-cell tabs keep "PANE n" byte-identically.
- **Restore + dangling-drop** — restore rebuilds named arrangements; each cell's key re-resolves
  to an existing-or-fresh `ContentId` through the #394/#396 registry (`insert`/`acquire_view`,
  content_registry.rs:56/66). A key that no longer resolves → that CELL drops silently (the #205
  `is_file`/`is_dir` guard stance — app.rs:2201/:1344); all cells dropped → the ARRANGEMENT is
  skipped (the #386 forgiving-restore posture). Restore never fails and never invents content.
  Terminals are the lossy case by design: cwd keys RESPAWN fresh (never reattach — #205), and cwd
  is non-unique, so a persisted cell carries an ordinal (the #388 Q4 inspect) so two same-cwd
  cells don't collapse onto one instance.

## Scope
### In
- **The pure arrangement model + codec** (new gpui-free module, the workflows.rs shape; cov/MSI
  100): `PaneArrangement { name, scope, layout, cells }` serde struct; per-cell stable-key
  encode/decode (kind + cwd/path + ordinal — D-OPEN-REF-SHAPE); name validation/sanitization at
  WRITE time (the #177 `sanitize_title` discipline — grid_layout.rs:316-322: strip framing +
  `\x1f`, trim, cap 60; the #163 D2 writer-guards-reader-trusts stance).
- **The `[[panes]]` setting + persist helper** (settings.rs): the `Vec<T>` array-of-tables
  template + `persist_panes` (round-trip test THROUGH the helper — kills its `Ok(())` mutant, the
  #204/#234 house pattern, settings.rs:914-965) + the `AppliedSettings` field.
- **The naming draft** (masked app.rs shims): field + key-ladder branch + card render + every #204
  choke point (`text_input_blocked`, two-owners exclusion, `close_transient_overlays`, close-tab/
  project index-shift clearing — the renaming_tab :7424/:7522 discipline).
- **The rail label delta**: `rail_rows` takes the name bindings as input; named → the name,
  unnamed → "PANE n" byte-identical (the #390 suites carried).
- **The restore seam**: a pure resolution fn (persisted entry + per-key resolution outcomes →
  rebuilt shape with dropped cells enumerated) + the masked rebuild wiring through the registry;
  P2 settles the boot-path authority against the #163 shell restore (no double-build, no orphan —
  see Phase Plan).
- **Regression pins**: #163/#205/#177 persistence round-trips byte-identical; #390 rail suites
  green unchanged; the #394/#396 registry contracts untouched.

### Out (explicitly deferred)
- **Global cross-workspace Panes** — slice-7, gated on the multi-workspace model (#388 Q0).
- **Arrangement sharing/export** — the settings file is the only store.
- **Any auto-capture of layouts** — naming is an explicit verb; no implicit save of every split.
- **An arrangements manager UI** (list/delete/reorder surface) — the rail + the verb are the whole
  surface this slice.
- **Cockpit/browser content in arrangements** — lands with #400 automatically via the registry
  (the resolution seam is kind-agnostic over stable keys; new kinds add a key form, this seam
  doesn't change).

## Reference (§20)
**Warp — Launch Configurations, at the published-behavior level.** The checked behavior map names
the surface but does not map it: docs/warp_architecture/crates/warp.md:48 lists `launch_configs`
among Warp's public modules — NAME-level only, no behavior documented in our maps. At the
published level (Warp's public docs, no source): a Launch Configuration is a NAMED, persisted
file capturing a window/tab/pane arrangement — the split layout, each pane's starting directory,
optional startup commands — saved from the live session via a palette verb and reopened BY NAME;
reopening spawns FRESH sessions in the saved cwds, never reattaching processes. That is exactly
this ticket's shape and Marley's #205 respawn stance. tmux's named sessions/layout strings are the
same convention at the terminal-multiplexer level. Zed's mapped analog is session persistence, not
named layouts: docs/zed_architecture/subsystems/07-workspace-panes-palette.md:191-193 maps
`SerializableItem`/`SerializableItemRegistry` (a kind string → deserializer at boot; "a restored
session rebuilds the right item views") — adopted at the behavior level as the
rebuild-views-from-stable-keys precedent; no named-arrangement feature is mapped. Clean-room §20
untouched: no Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked, thin here.** warp.md:48 (`launch_configs` module name only);
   zed 07-workspace-panes-palette.md:76/:191-193 (`SerializableItem` registry — session restore,
   not naming). The #388 spike doc itself is the primary design source
   (pane-composition-model.md Q4: the `[[panes]]` sketch, stable keys, the ordinal, dangling-drop,
   skip-empty — this spec implements that paragraph).
2. **Published material.** Warp Launch Configurations (named YAML arrangement files, save-from-
   session, reopen-by-name, fresh spawn — behavior only, per its public docs); tmux named
   sessions/layouts as convention.
3. **Our permissive deps — checked, no external owner.** serde/toml already carry the codec
   (`marley_settings` rides them; manager.rs persists the working tree); gpui renders only;
   ropey/regex/alacritty_terminal own nothing near a named-layout seam. The REAL adoption leg is
   IN-HOUSE shipped seams, and this ticket is mostly composition of them: `marley_settings` owns
   the file + typed registry (`define_setting!`); the #204 `[[workflows]]` template owns the
   Vec<T>-table + tolerance discipline (settings.rs:914-965, workflows.rs:17-21); grid_layout owns
   the shape algebra + framing hygiene (`breaks_grid_framing` :309-311, `sanitize_title`
   :316-322, the `t=<cwd>`/`c=<path>` leaf keys :79-87); the #204/#177/#378 inline-draft idiom
   owns naming UX; the #394 `ContentRegistry` owns resolution. No new machinery kinds.

## React-first (parity)
**UI-AFFECTING — Zone A (the workspace rail's Panes section + a new centered naming card;
port-map rows: `components/LeftRail.tsx` ↔ app.rs (rail)/tabs.rs — `marley-web/docs/
MARLEY-PARITY.md`).** Implement builds in marley-web FIRST: the POC's `LeftRail.tsx` already
renders the #390 Panes section with `PANE ${pane.id}` rows (artifacts/marley-ide/src/components/
LeftRail.tsx:253-307); it does NOT yet have an arrangement-naming flow — **the POC gains it here
as the prototype**: the name-an-arrangement draft (the #204-card shape: centered framed input,
caption, `{name}▏` caret line, context line) + the named Arrangement rows, iterated at
localhost:5173 (`pnpm --filter @workspace/marley-ide run dev`) until the look and flow are
confirmed — THEN ported 1:1 into the pure seams + masked app.rs shims. The shipped "PANE n"
unnamed presentation stays Marley-authoritative and byte-identical (D-OPEN-UNNAMED-DEFAULT).
Validate captures the React↔Marley parity pair (named rows + the open naming card). POC
discipline: the POC's persistence is mocked state — the settings file never ports
(`stateStorage.ts` ↔ `marley_settings` is a vocabulary row, not a data port); ContentId↔PaneItem
vocabulary stays aligned (§ Shared vocabulary).

## Locked-In Decisions
- **D1 — shapes/coordinates/names serialize; `ContentId`s NEVER.** A `ContentId` is session-local
  (#396's discipline: "ids never serialize; the persistence rebuilds from shapes"). The persisted
  cell is a stable KEY — kind + cwd/path (+ ordinal) — re-resolved to an existing-or-fresh id at
  restore through the registry. A persisted byte stream containing an id is a spec violation
  testable at the codec unit.
- **D2 — dangling refs DROP; restore never fails, never invents.** A key that no longer resolves
  (file deleted, dir gone) drops its CELL silently with the rest intact; all cells dropped → the
  arrangement is SKIPPED (#386 forgiving-restore); no phantom content, no panic, no error modal.
  Terminals respawn fresh-in-cwd (#205 — scrollback/process gone by design); a dead cwd is a drop,
  not a root-fallback (the arrangement promised THAT cwd; P2 confirms against the :1344 root-
  fallback precedent and pins whichever arm survives with a unit).
- **D3 — the naming verb reuses the #204/#177 inline-draft idiom; NO new modal machinery.** Draft
  field + key-ladder branch + `text_input_blocked` line + two-owners exclusion +
  `close_transient_overlays` + centered card — the second consumer inherits the FIRST consumer's
  guards intact (`PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`): trim,
  empty-name-refuses-silently, blank-target flashes, draft owns every key including Enter's "\n".
- **D4 — the settings table is `[[panes]]` under the existing settings file + codec-tier
  discipline.** The #204 `Vec<T>` template on `marley_settings` (#163/#205 tier pattern):
  round-trip identity through the persist helper; absent/malformed → empty default; optional keys
  `#[serde(default)]`; write-time name sanitization (#177 `sanitize_title` discipline); a
  pre-#399 settings file loads unchanged (no migration, no version bump — the key is simply new).

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-REF-SHAPE** — how a content reference serializes per kind: terminal = cwd? editor =
  path? **Recommendation: path/cwd-based re-resolution mirroring the shape codec** — the same
  stable keys the #205/#258 leaf codec already uses (`t=<cwd>`/`c=<path>`, grid_layout.rs:79-87),
  carried as typed TOML fields (or the leaf-token alphabet inside a `layout` string — P2 picks the
  exact encoding against hand-editability + the framing rules :309-311), plus the per-cell ordinal
  for same-cwd terminal twins (#388 Q4 inspect). Marley editors always carry a path, so the
  untitled-buffer edge doesn't bite today.
- **D-OPEN-SCOPE** — per-project vs global arrangements. **Recommendation: per-project first** —
  `scope = <project-root>` (the #234/#245 keyed-by-root precedent; restore keys off the RESTORED
  root, never the launch cwd — `PR-claude-boot-decisions-key-the-restored-active-root-001`);
  global is slice-7's gated territory.
- **D-OPEN-APPLY-SEMANTICS** — applying a named arrangement (boot restore; any reopen verb P2
  keeps in scope): new tab vs replace focused. **Recommendation: new tab** — non-destructive,
  matches the #393 create-verbs' additive posture and Warp's published reopen-into-new behavior.
  P2 also settles the boot-path AUTHORITY sub-question here: the #163 shell blob already rebuilds
  every live tab including split grids (T= entries, grid_layout.rs:329-372) — a named
  arrangement's live tab must not rebuild twice (candidates: shell rebuild + name RE-BIND via a
  rider the #177 `title\x1f` slot already precedents, vs `[[panes]]`-driven rebuild with shell
  dedup; the spec locks only the observables — back once, named, no duplicate).
- **D-OPEN-UNNAMED-DEFAULT** — do unnamed multi-cell tabs still read "PANE n"?
  **Recommendation: yes, byte-identical** — the tabs.rs:857 label arm untouched for unnamed rows;
  naming is opt-in; the #390 suites pin it.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the operator invokes the name-an-arrangement verb on a multi-cell tab and commits a non-empty name, the system shall persist a `[[panes]]` entry (name + scope root + shape + per-cell stable keys) that survives a settings reload. | settings round-trip unit THROUGH the persist helper (kills its `Ok(())` mutant — `PR-claude-persist-verify-trigger-not-just-codec-001`); the #204 tolerance arms carried (absent/malformed → empty; missing optional key → serde default, not wipe-all) |
| REQ-002 | WHILE a live multi-cell tab has a bound name, its rail Arrangement row shall display that name (sanitized per the #177 write-time discipline) instead of "PANE n". | `rail_rows` unit (named binding → name label; hostile name → sanitized at write, framing chars never persisted); #390 row-shape suites green |
| REQ-003 | WHEN the app restarts, a named arrangement shall be rebuilt — shape restored, each cell's key re-resolved through the registry (terminal: fresh respawn in cwd per #205, ordinals keep same-cwd twins distinct; editor: reopened by path), the name bound to the rebuilt tab, exactly once (no duplicate tab). | restore-seam units (full-resolve permutation + the once-only pin); driven restart check at Validate |
| REQ-004 | WHEN a persisted cell's key no longer resolves at restore, the system shall drop that cell silently with the remaining cells intact; WHEN every cell of an arrangement fails to resolve, the system shall skip that arrangement entirely; the restore shall never fail and never fabricate content. | pure resolution units per permutation (one-dangling / mixed-kinds-dangling / all-dangling / empty-cells entry); §18.1 inspect: no panic/error path on any malformed entry |
| REQ-005 | WHEN an arrangement is persisted, the serialized `[[panes]]` bytes shall contain names, shapes, coordinates, and stable content keys only — never a `ContentId`. | codec unit asserting over the serialized TOML (no id field exists to leak — the struct shape is the proof, pinned); §18.1 inspect on the encode path |
| REQ-006 | WHILE a multi-cell tab is unnamed, its rail row shall read "PANE n" byte-identically to #390, and Esc / empty-name commits shall leave all state untouched (no entry persisted, no label change). | rail byte-pin unit; draft-cancel + empty-commit units; #390 rail suites green unchanged |
| REQ-007 | WHILE the naming draft is open, it shall own the keyboard per the #204 discipline — registered at every choke point: `text_input_blocked`, the two-text-owners exclusion, `close_transient_overlays`, close-tab/index-shift clearing. | headless overlay-gate test (the #267 suite's pattern); §18.1 inspect walking the choke-point roster (`PR-claude-new-overlay-register-at-every-choke-point-001`) |
| REQ-008 | WHEN the full gate runs, the pre-existing #163/#205/#177 persistence round-trips and the #390/#394/#396 suites shall pass unchanged, and Validate shall capture the React↔Marley parity pair (named rail rows + the open naming card). | full suite + gate `--diff` green; parity captures at Validate (env-blocked fallback documented if the machine blocks driving — the #204/#205 precedent) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the `PaneArrangement` codec (serde shape + any key encode/decode),
the dangling-drop resolution fn (exhaustive over the resolve/drop/skip space — no defensive
catch-all), name validation/sanitization, the `rail_rows` label decision. MASKED: the app.rs draft
shims (key-ladder branch, card render), the persist/restore wiring, the verb plumbing (app.rs is
coverage-excluded). Typed inputs; no `unwrap` on file-derived values (a hand-edited `[[panes]]`
entry is hostile input — the #204 tolerance tests are the template). The app.rs edits land near
masked draft/persist shims — skip-detach territory: re-run `cargo mutants --list -f` on the
ACTUAL touched files after placement, and re-verify neighboring `#[mutants::skip]` bindings
(`PR-claude-trace-the-real-cargo-mutants-list`).

## Phase Plan
- **P2 Design** — settle D-OPEN-REF-SHAPE (exact per-kind key encoding + the ordinal), SCOPE,
  APPLY-SEMANTICS (incl. the restore-authority fork vs the #163 shell blob — no double-build, no
  orphan), UNNAMED-DEFAULT; the exact `[[panes]]` TOML schema + the name↔tab binding
  representation; the naming-verb entry-point set; per-REQ test plan; **both-halves manifest,
  marley-web first** (LeftRail.tsx + the naming-card component + mocked state), then the Rust
  manifest (new pure module, settings.rs, tabs.rs, app.rs shims); verify #398's shipped Pane-row
  label shape and bind to it.
- **P3 Implement** — **React-first: prototype the naming flow + named rows in marley-web and
  confirm at localhost:5173 (per `## React-first (parity)`)**; then Rust: the pure codec seam
  FIRST (arrangement model + keys + resolution + validation, unit-proven), then the settings
  table + persist helper, then the masked verbs/draft/rail/restore wiring.
- **P3.5 Inspect** — adversarial: restore-with-dangling permutations (each kind, mixed, all,
  hand-mangled entries); codec back-compat with pre-#399 settings files (no `[[panes]]` key;
  entries missing optional keys); name collisions (two entries, same name/scope) + empty/hostile
  names; the draft at EVERY choke point; double-rebuild/orphan on restore; ids-never-serialize;
  skip-detach re-check; provenance (§20).
- **P4 Validate** — write + RUN units per REQ; driven restart check; the React↔Marley parity
  pair; `cargo mutants --list -f` on the actual touched files; gate green (`--diff`), cov/MSI 100
  on the pure seams; #163/#205/#177/#390/#394/#396 regressions green.
- **P5 Complete** — CHANGELOG; mark slice-6 shipped in pane-composition-model.md (Q5 train);
  AAR capture; archive; close #399.
