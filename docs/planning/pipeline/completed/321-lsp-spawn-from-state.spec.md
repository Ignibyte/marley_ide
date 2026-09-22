---
pipeline_id: 8a6480a4-16e0-4bcb-aeb9-066aa2160d2c
ticket: forge#321 (afffcc4b-dc2f-4fc7-b166-f4057c26cbd5) · local docs/planning/tickets/open/TICKET-321-lsp-spawn-from-state.md
aar_id: 55b2817a-8e0b-4be3-8357-60f3cd2589bc
status: Phase 5 — Complete PASS
title: LSP host creation derives from open-doc STATE — a restored tab spawns rust-analyzer
type: bug
milestone: M20
references: [ensure_lsp_for_opened_file — the ONLY creation site, gesture-triggered (app.rs:4368-4396), its ONE caller open_file_in_viewer (app.rs:4507; grep-verified sole), the gate quadruple (.rs · under-root · !contains_key · Cargo.toml), the pump ALREADY derives doc-sync from state (app.rs:1386-1432 — open_files from tabs' editors' open_docs → host.reconcile), LspHost::reconcile re-didOpens after respawn (lsp_host.rs:343 docs.clear) so LATE host creation is already legal, the V= restored-editor tier (grid_layout.rs:266-295) never passes the gesture, the seeded-blob headless lane (headless_drive.rs:20-26 seed via persist_shell; settings.rs:883 blob example), the [[lsp.servers]] tee (settings.rs:81-84 · language_servers :201), observed live at #313 P4: booted with main.rs open — no lsp segment, no rust-analyzer process, PR-claude-two-gated-calls-must-read-the-phase-once-001, BF-lsp-didopen-carries-empty-text-ready-race-001]
---

## Title
Quit Marley with a `.rs` file open — the thing #205's persistence exists to preserve — relaunch, and
you silently get no diagnostics, no hover, no go-to-def, no completions on the file you are looking
at. #308's ONLY spawn trigger is the open GESTURE (`open_file_in_viewer` → `ensure_lsp_for_opened_file`),
and a restored tab never gestures. Every M20/M21 feature rides the host, so all of them are dead on
the common launch path until you happen to open some OTHER file. The recon found the fix is smaller
than the ticket sketched: doc-sync is ALREADY state-derived on the pump (#309 reconciles
didOpen/didChange/didClose from `open_docs()` every tick) — host CREATION is the one remaining
event-derived piece. Move it onto the same derivation and the class of "opened by a path that forgot
to trigger" bugs closes for good.

## Scope
### In
- **The state-derived ensure:** a new `ensure_lsp_hosts_for_open_docs(...)` invoked in the pump tick
  immediately before the existing reconcile block (app.rs:1431-1435), consuming the SAME
  `open_files` vec the pump already builds (:1410-1430): if any open doc is a `.rs` under
  `active_root`, no host exists for that root, and `root/Cargo.toml` is a file → create the host
  (byte-for-byte today's gate quadruple + `resolve_binary` + `LspHost::new`, app.rs:4380-4396).
- **One trigger, not two:** the gesture call at `open_file_in_viewer`:4507 is REMOVED and
  `ensure_lsp_for_opened_file` retires into the new state fn (its doc comment moves + updates). A
  fresh gesture-open now spawns on the next pump tick — sub-frame latency; the pre-host lookups
  (F12/hover/footer) already no-op harmlessly today, so nothing observes the tick.
- **Late creation is already legal:** `LspHost::reconcile` Ready-gates and didOpens NEWLY-SEEN files
  (lsp_host.rs:169-177), and a respawn clears `docs` so reconcile re-didOpens (:343) — the wire
  sequence for a host created N ticks after the docs opened is the SAME sequence as a respawn.
  No new wire logic.
- **Scope = the active root**, exactly matching the reconcile it joins (a restored-but-inactive
  workspace spawns when activated, the tick its doc-sync would start — consistent, documented).
- **Test hook:** a `lsp_host_exists_for_test(root)` (or count) accessor for the headless asserts.
### Out (explicitly)
- Multi-root simultaneous spawn (per-active-root today, per-active-root after). #319 (symlinked-root
  file identity). #320 (the fake_ls app-level Ready lane — its absence shapes REQ-002's split below).
  #355 (pane surfaces joining `open_docs` — spawn AND sync widen together there; this ticket keeps
  parity with doc-sync by construction). Host shutdown/reap on last-doc-close (nothing shuts hosts
  down today; deriving DELETION from state is a separate decision, not smuggled in here).

## Reference (§20)
N/A — Marley-specific lifecycle bug; the "derive from state, not events" shape is Marley's own #309
precedent, not an observed reference behavior.

### Prior art
1. **OUR OWN CODE (decisive):** the pump's reconcile block (app.rs:1386-1432) IS the pattern — the
   ticket's design fork ("reconcile on the pump like #309" vs "fire the trigger per restored tab")
   dissolves because half the reconcile already exists; this ticket completes it. The knowledge
   provenance: PR-claude-two-gated-calls-must-read-the-phase-once-001 +
   BF-lsp-didopen-carries-empty-text-ready-race-001 (the same family — LSP state derived from an
   event rather than the truth).
2. **Published material** — the LSP spec's lifecycle (didOpen after initialized) is already honored
   by reconcile's Ready gate; nothing new needed.
3. Checked gpui — no owner (not a UI seam).

## Locked-In Decisions
- **D1-DERIVE-FROM-STATE** — host existence = f(open docs), computed on the pump from the SAME vec
  doc-sync consumes; the gesture path stops being a trigger. **Phase-1 constraint (F2): the ensure
  MUST run after `drain()`**, so every Ready-phase read in a tick observes one settled value — the
  invariant `BF-lsp-didopen-carries-empty-text-ready-race-001` earned. (That BF cannot recur here:
  `LspHost::new` sets `life: None`, and both `needs_text` and `reconcile` bail unless Ready, so a
  freshly-created host sends nothing on its creation tick rather than an empty didOpen.)
- **D2-ONE-TRIGGER** — remove the `open_file_in_viewer` call rather than keep both (two triggers =
  drift; the one-tick latency is unobservable and REQ-003 pins the gesture path stays whole).
- **D3-GATE-UNCHANGED** — the quadruple (.rs / under-root / no-host / Cargo.toml) moves verbatim; a
  bare-tempdir headless boot still NEVER spawns (the existing tests' invariant, kept).
- **D4-PARITY-WITH-SYNC** — whatever `open_docs()` enumerates is what can summon a host; when #355
  widens the enumeration, spawn widens with sync automatically. No pane-special-casing here.
- **D5-SHIM-STANCE** — the ensure fn stays a `mutants::skip` spawn-touching shim (as today at :4377);
  behavior is pinned by the headless boot asserts + the live drive, not by mutation.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | create the LSP host for a RESTORED editor tab with no open gesture: seed a blob whose tab carries the `V=<path>` tier over a root WITH Cargo.toml + `[[lsp.servers]]` naming a stub binary → boot → pump → host exists | headless (the seed lane + the new hook); the ticket's core |
| REQ-002 | send the restored doc's didOpen with NON-EMPTY text once the server is Ready | live drive against real rust-analyzer: `lsp: ready` + the wire tee shows didOpen TEXT_LEN>0 (the BF-empty-text lesson); headless half: `needs_text` reports the doc pre-Ready (the materialization contract) |
| REQ-003 | still spawn on a FRESH gesture open (tree-click a `.rs` in a virgin workspace) within one pump tick | headless: open via `open_file_in_viewer`, pump once, host exists |
| REQ-004 | spawn NOTHING when the gate fails: no Cargo.toml (bare tempdir), no `.rs` open, or a host already present | existing bare-tempdir boots stay spawn-free + a no-manifest seeded boot asserts absence |
| REQ-005 | show the footer `lsp:` segment on a restored tab without further interaction | live drive (the observed-missing symptom, now present) |
| REQ-006 | leave reconcile's didChange/didClose behavior byte-identical (this ticket adds a caller ABOVE it, not logic inside it) | diff review + existing #309 tests green |

## Phase Plan
P2 confirm the pump block's exact shape @ live (the `view.` closure, borrow order — the new ensure
needs `&mut view.lsp_hosts` while `open_files` is already collected: the collect-then-mutate split at
:1410 exists precisely for this, reuse it); confirm `open_docs()`'s tuple shape; decide the hook's
form. P3 the state fn + the pump call + the gesture-call removal + comment moves; compile-green.
P3.5 critics on: the borrow across collect/ensure/reconcile, a root whose Cargo.toml appears LATER
(gate re-evaluates every tick — document that a manifest dropped mid-session now summons a host on
the next tick, a small behavior GAIN), resolve_binary cost per tick (only runs when gate passes AND
no host — once), and the removed gesture call leaving no dead code. P4 the headless quartet + the
live drive (per session drive-rules at run time) + gate `--diff`. P5 docs (lsp.md/app_shell.md: "host
creation is state-derived; the pump owns it") + AAR; the lesson is the ticket's own: derive from
state, not from the event that first produced it. Standing traps:
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md); **builds job-capped**.
