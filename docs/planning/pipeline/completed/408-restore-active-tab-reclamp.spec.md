---
pipeline_id: 54f7cc66-7020-4d9d-a641-d93a80bc6c48
ticket: forge#408 (767ae35d-8c5e-4608-896b-241a6f99996e) · local docs/planning/tickets/open/TICKET-408-restore-active-tab-reclamp.md
aar_id: 4c2f4f92-88bc-4d9d-8558-47c424b43585
status: Phase 5 — Complete PASS
title: Shell-codec index hygiene on drop — reclamp active_tab/active_project + guard the blob
type: chore
milestone: M29
references:
  - docs/planning/pipeline/completed/403-browser-tab-residency.notes.md (the filing critic's ledger)
  - crates/marley_app/src/grid_layout.rs (the codec + reclamp_active)
  - crates/marley_app/src/app.rs (the restore consumer)
---

## Title
When the shell codec DROPS an entry (a tab it cannot rebuild, a project it cannot
represent or find), the sibling "active" index is kept verbatim or merely bounded —
so a relaunch can focus a DIFFERENT tab/project than the user left. The fix is
adoption, not invention: `reclamp_active` (M14 #243) already owns the survivor-mapping
semantics at FILE level in both directions; apply the same primitive at TAB and
PROJECT level, and close the one framing-guard asymmetry left in the writer (the
terminal grid `blob` is the only serialize input pushed to the wire unguarded).

## Scope
### In
1. **Restore-side tab reclamp** — the `restore_shell` consumer (app.rs restore loop)
   drops a `T=` tab whose PTY spawn fails (app.rs:2259 conditional push) and a `V=`
   tab whose files are all unreadable (:2332); map the saved `active_tab` through the
   surviving-original-indices via `reclamp_active` instead of `pl.active_tab.min(last)`
   (:2360). Cockpit/`B` arms push unconditionally (the :2294 LOAD-BEARING comment) and
   contribute survivors always.
2. **Restore-side project reclamp** — the same loop skips a vanished root
   (`!root.is_dir()`, :2222) and only bounds `active_project` (:2379); same mechanism.
3. **Serialize-side project reclamp** — `serialize_shell` drops a framing-breaking
   root (grid_layout.rs:344-346) but writes `shell.active_project` verbatim (:342);
   reclamp onto the surviving projects (the exact `serialize_code_paths` precedent,
   :280-298).
4. **Blob framing guard** — `serialize_shell` pushes the terminal grid `blob`
   verbatim (:387); guard it at write time against the entry-framing hazard set
   (`\t \n \r` via `breaks_framing`, plus the in-entry `\x1f` separator), the D2
   writer-guards stance. D-BLOB-GUARD locked at design: write-time DROP
   (`continue`) joining the V= arm; no `debug_assert!` (it would panic the REQ-005
   test in the debug profile — the drop path must be exercisable).
5. **Serialize-side tab reclamp** (design discovery) — the V= no-representable-path
   `continue` (grid_layout.rs:400) ALREADY drops a tab after `active_tab` was
   written verbatim (:350); the blob guard adds a second drop arm. The pre-scan
   writer reclamps `active_tab` over the surviving tabs (REQ-007).

### Out (explicitly deferred)
- The DOWNGRADE half of tab drift (an old build skipping an unknown tag shifts later
  tabs) — #403 already judged it unfixable retroactively; forward-only remains.
- Any change to the wire FORMAT itself (no new fields, no version bump — pure
  writer/reader hygiene on the existing bytes).
- The `\x1f` part-count disambiguation scheme for `T=` riders (#399) — untouched;
  the blob guard only ensures the blob can never *forge* those shapes.
- `restore_shell`'s own parsing semantics (grid_layout.rs:420) — the codec keeps its
  "clamping is the caller's job" division; restore-side fixes land in the CALLER.

## Reference (§20)
**Zed** (the same-gpui-stack editor reference — the session-persistence train).
Behavior matched: a relaunch rebuilds the workspace **and returns focus to what the
user left**, degrading per-item when a persisted item can no longer rebuild — Zed's
`SerializableItem`/registry boot path re-creates item views from serialized kinds and
drops what no longer deserializes, without letting one dead item shift which
surviving item is active (docs/zed_architecture/subsystems/07-workspace-panes-palette.md:191-193,
:270-271). Marley matches the BEHAVIOR (stable focus identity under degraded restore)
through its own #163 D2 codec — no Zed source read (§20 wall; the behavior map is
research, not source).

### Prior art
- **In-repo (the decisive find):** `reclamp_active` (grid_layout.rs:269, M14 #243)
  already owns the exact semantics — "kept-position of the first survivor whose
  ORIGINAL index is at-or-after active, else last" — is `pub(crate)`, unit-tested
  (`reclamp_active_cases`, :1139), and already shared by BOTH directions at file
  level (`serialize_code_paths` :291 and the app.rs V= restore arm :2334). The
  spec's whole mechanism is adopting the owned primitive at two more levels; nothing
  is invented. `serialize_code_paths` (:280-298) is the write-side drop+reclamp
  template; the V= restore arm (:2308-2340) is the read-side template.
- **Behavior maps:** Zed's workspace-persistence architecture (trait-per-item
  registry) documented in docs/zed_architecture/subsystems/07-workspace-panes-palette.md
  — architecture context; no finer-grained "active index after drop" behavior is
  documented there, so the micro-behavior is pinned by Marley's own #243 decision.
- **Permissive deps:** none own this seam — checked gpui (no persistence layer),
  ropey/regex/tree-sitter (not persistence), alacritty_terminal (PTY/grid, no
  session codec). serde is deliberately not in this codec's lineage (#163 hand-rolled
  framing); no adoption candidate.

## React-first (parity)
N/A — no UI delta: no chrome, layout, type, color, or affordance changes — no drawn
pixel differs. The change corrects WHICH pre-existing tab/project keeps focus after a
*degraded* restore (dropped entries), a persistence-correctness behavior the React POC
cannot mirror: marley-web has no session persistence/restore layer (Vite dev boots
fresh; the parity contract covers the visible shell, not the codec).

## Locked-In Decisions
- D1 — **Adopt `reclamp_active`, never a second primitive.** All three new reclamp
  sites call the existing `pub(crate) fn reclamp_active` (M14 #243). If any site
  needs different semantics, that is a spec bug to surface, not a fork to write.
- D2 — **Writer guards, reader trusts** (the #163 D2 stance, reaffirmed): the blob
  guard lands in `serialize_shell` (write time), NOT as restore-side tolerance.
- D3 — **Restore-side fixes live in the app.rs consumer**, honoring the codec's
  documented division ("Restore-time clamping … is the caller's job",
  grid_layout.rs:419); serialize-side fixes live in `serialize_shell`.
- D4 — **Pure-seam testability posture** (#406 lineage): survivor-collection +
  reclamp composition must be expressible as pure logic exercised at cov/MSI 100;
  app.rs wiring stays thin per the standing coverage posture (app.rs is
  coverage-excluded but NOT mutation-excluded — the new logic must be killable, via
  the pure seam and/or the headless drive lane).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN restore drops a tab that precedes the saved `active_tab` (a `T=` whose spawn fails or a `V=` with no readable file), the system shall focus the tab whose ORIGINAL index is at-or-after the saved active (else the last survivor), per `reclamp_active` semantics | unit test on the pure seam + headless drive: pre-seeded session with a doomed `V=` before the active tab → assert focused tab identity |
| REQ-002 | WHEN restore drops NO tab, the system shall focus exactly the saved `active_tab` (behavior unchanged; the reclamp is identity on a full survivor set) | existing restore tests stay green + explicit identity case in the new unit suite |
| REQ-003 | WHEN restore skips a vanished project root that precedes the saved `active_project`, the system shall activate the project whose ORIGINAL index is at-or-after the saved active (else the last survivor) | unit test on the pure seam + drive with a nonexistent persisted root before the active project |
| REQ-004 | WHEN `serialize_shell` drops a project root the wire cannot carry (framing-breaking OR empty — the reader's own unconditional drop, inspect F1), the emitted `active_project` line shall be the reclamped survivor index, never the pre-drop index | grid_layout unit test: hostile root before the active project → parse back → active lands on the same surviving project; plus the `root:""` case |
| REQ-005 | WHEN a `TabLayout::Terminal` blob contains any entry-framing hazard byte (`\t`, `\n`, `\r`, or `\x1f`), `serialize_shell` shall not emit bytes that misparse — the hazard never reaches the wire (guard shape per D-BLOB-GUARD, Phase 2) | grid_layout unit test constructing a hostile blob directly → serialize → restore_shell round-trip stays well-formed; mutation kills on the guard |
| REQ-006 | WHILE every entry survives serialization AND the saved actives are in-range, `serialize_shell` shall emit byte-identical output to today (no wire-format change; an OUT-OF-RANGE active is deliberately clamped to the last survivor — a hygiene change, unreachable from the live writer whose `switch_*`/`adjust_active` keep indices in-range) | round-trip property in the existing suite + explicit golden case with in-range actives (inspect F2) |
| REQ-007 | WHEN `serialize_shell` drops a tab (a `V=` with no representable path, or a hazard blob per D-BLOB-GUARD), the emitted `active_tab` shall be the reclamped survivor index, never the pre-drop index | grid_layout unit test: dropped tab before the active one → round-trip lands on the same surviving tab |

## Phase Plan
- **P2 Design** — decide D-BLOB-GUARD (continue vs debug_assert vs both); the shape of
  the pure survivor-collection seam (where it lives so cov/MSI 100 holds); the drive-test
  strategy for the spawn-fail T= arm (can it be forced headlessly, or does the V= case
  kill the shared seam?); the file manifest + regression test plan.
- **P3 Implement** — per design; grid_layout.rs (serialize-side) + app.rs (restore-side
  wiring through the pure seam).
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings; provenance check.
- **P4 Validate** — write + RUN the planned tests; `scripts/gates.sh --diff` green.
- **P5 Complete** — CHANGELOG + architecture docs (the codec doc), AAR capture, archive,
  close local + forge ticket.
