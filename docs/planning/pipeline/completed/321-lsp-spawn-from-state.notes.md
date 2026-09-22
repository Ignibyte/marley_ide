# LSP host creation derives from open-doc STATE — Notes

- **Forge ticket:** #321 `afffcc4b-dc2f-4fc7-b166-f4057c26cbd5`
- **AAR:** `55b2817a-8e0b-4be3-8357-60f3cd2589bc`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-321-lsp-spawn-from-state.md
- **Pipeline spec:** 321-lsp-spawn-from-state.spec.md
- **pipeline_id:** `8a6480a4-16e0-4bcb-aeb9-066aa2160d2c`

## Phase 1 — Plan

- **Request:** promote the pre-authored spec, re-verify every cited seam by SYMBOL against live `main`
  @ `8dd068c`. SECOND of the goal `/work 307,321,350,345,348` (auto-approved,
  autonomous-through-commit). #307 shipped as `8dd068c`, GATE GREEN 15/15.
- **Classification / tier:** work pipeline, one shippable slice. `type: bug`, milestone M20 — every
  M20/M21 LSP feature (diagnostics, hover, go-to-def, completions, rename, code actions, signature
  help, inlay hints) is dead on the common launch path, with no error and no status segment.
- **Forge recall (§18.3):** zero active bulletins (checked at #307). The two prior-art codes the
  ticket names — `PR-claude-two-gated-calls-must-read-the-phase-once-001` and
  `BF-lsp-didopen-carries-empty-text-ready-race-001` — turn out to be directly load-bearing here
  (F2 below), not just thematically related.

### Findings

- **F1 — the gesture-trigger premise HOLDS (confirmed).** `ensure_lsp_for_opened_file` is defined at
  app.rs:4370 and called from exactly ONE place: app.rs:4507, inside `open_file_in_viewer`. Its own
  comment asserts "every editor-open path funnels through here — the ONE spawn trigger for the
  workspace's LSP host", which is true for *gestures* and precisely why a restored tab gets nothing.
  The **gate quadruple is confirmed verbatim**: `.rs` extension → `full.starts_with(&root)` →
  `!self.lsp_hosts.contains_key(&root)` → `root.join("Cargo.toml").is_file()`. The design moves all
  four unchanged.
- **F2 — THE EMPTY-TEXT RACE: investigated, and it does NOT apply. Recording both halves, because a
  reader who knows the ticket's own `Related` codes will reasonably fear it.** The worry: the pump's
  `open_files` collect reads `view.lsp_hosts.get(&active_root)` *inside* its `.map()` to decide
  `needs_text`. If the new ensure runs AFTER that collect (where the spec puts it), then on the
  creation tick the collect sees no host, materializes no text, and `reconcile` is then handed
  `("path", "", version)` — which is exactly the shape of
  `BF-lsp-didopen-carries-empty-text-ready-race-001`, where a server held an EMPTY file forever
  because the doc was recorded as synced at that version so no didChange ever followed.
  **It cannot happen, for a reason worth writing down:** `LspHost::new` sets `life: None`, and BOTH
  `needs_text` and `reconcile` open with the same guard —
  `self.life.as_ref().map(Lifecycle::phase) != Some(Phase::Ready)` → bail. A freshly-created host is
  never Ready on its creation tick, so `reconcile` sends **nothing at all** that tick; it only sends
  didOpen once Ready arrives (inside `drain()`), by which point `needs_text` — reading the same
  settled phase in the same tick — returns true and materializes the real text. The symmetric Ready
  gates plus drain-first ordering are what the earlier BF earned, and they defend this ticket for
  free. **The constraint this imposes on the design: the new ensure must sit AFTER `drain()`**, so
  every phase read in a tick observes one settled value. Both candidate placements (before the
  collect, or between collect and reconcile) satisfy that and are safe.
- **F3 — late creation is wire-legal (confirmed).** `reconcile`'s `self.docs.get_mut(&full)` `None`
  arm pushes an `open_notification` and inserts the `DocEntry`. A host created N ticks after the docs
  opened has an empty `docs` map, so every open doc takes that arm and gets a didOpen. Same mechanism
  the respawn path relies on (`docs.clear()`).
- **F4 — `resolve_binary` cannot run per-tick (confirmed).** Its single call site (app.rs:4387) is
  reachable only after the quadruple passes, and the third clause is `!contains_key(&root)`. A host is
  inserted unconditionally afterwards — **including when `resolved` is `None`** (a missing binary
  still yields `LspHost::new(root, None)`) — so `contains_key` short-circuits every later tick.
  **Pre-existing consequence, not introduced here:** if rust-analyzer is absent at first open it is
  never re-resolved for that root, even if installed later. Out of scope; noted so it is not
  mistaken for a #321 regression.
- **F5 — removing the gesture call leaves no dead code, but leaves a STALE COMMENT.** D2 deletes the
  `self.ensure_lsp_for_opened_file(&path);` line at app.rs:4507; the fn itself is retired *into* the
  new state-derived ensure, so nothing is orphaned. But app.rs:4505-4506 asserts "every editor-open
  path funnels through here — the ONE spawn trigger", and `active_lsp_segment`'s doc (app.rs:4407)
  says "`None` until a `.rs` open creates a host". Both become false the moment creation moves to the
  pump. Implement must rewrite them — the "a false doc is well-formed Rust" trap that has now cost
  this batch twice.
- **F6 — the headless seed lane needs a new variant.** `seed_one_project` (headless_drive.rs:23)
  writes `"0\n{root}\t0\tT=t"` — a terminal-only tab. REQ-001 needs the `V=<path>` editor tier
  (grid_layout.rs:265/:289 serialize/parse; the shell wire at :325/:363 emits `V=`). A sibling seed
  helper is required; it is test-only, so it carries no production risk.

### Prior-art sweep (§20, all three legs)

1. **Behavior maps / observed** — N/A. This is a Marley lifecycle bug; there is no reference-app
   behavior being matched. The observation that *drove* the ticket is our own: #313's Phase-4 drive
   booted with `src/main.rs` open and found no `lsp:` footer segment and no rust-analyzer process.
2. **Published material** — the LSP specification's lifecycle (didOpen only after `initialized`) is
   already honored by `reconcile`'s Ready gate; nothing new is needed from it.
3. **OUR PERMISSIVE DEPS** — `lsp-types 0.97` (MIT) is shipped but deliberately confined to the
   #310 diagnostics parse seam; it models *messages*, not server lifecycle or a
   server-set-from-open-docs policy, so it owns nothing here. No `tower-lsp` or similar client
   framework is a dependency (the client is ours, written from the published spec). gpui/ropey/regex:
   not their seam. **Verdict: no shipped dep owns this.**
   **The real prior art is our own code, and it is decisive:** the pump ALREADY derives doc-sync from
   state every tick (app.rs:1386-1432). This ticket does not invent a pattern; it finishes one. The
   ticket's own design fork ("reconcile on the pump" vs "fire the trigger per restored tab")
   therefore dissolves — half of option A already exists, so host creation simply joins it.

### Decisions confirmed

All five hold as written; no reopens. `D1-DERIVE-FROM-STATE`, `D2-ONE-TRIGGER` (F5 confirms nothing
is orphaned), `D3-GATE-UNCHANGED` (F1 confirms the quadruple verbatim), `D4-PARITY-WITH-SYNC`,
`D5-SHIM-STANCE`. **Sharpened by F2:** the new ensure must run after `drain()` — that is now a stated
constraint rather than an incidental property of where the spec suggested putting it.

### Discovery — the edit surface handed to Design

- `crates/marley_app/src/app.rs` — the pump tick (after `drain()`, around the `open_files` collect at
  :1410 and the `reconcile` at :1431): a new state-derived ensure consuming the same open-doc set; the
  gesture call at :4507 removed; `ensure_lsp_for_opened_file` retired into the new fn; the two stale
  comments (F5) rewritten. A `lsp_host_exists_for_test` accessor for the headless asserts.
- `crates/marley_app/src/headless_drive.rs` — a `V=`-tier seed helper (F6) + the REQ-001/003/004
  drives.
- No changes anticipated in `lsp_host.rs` (reconcile/needs_text are correct as-is) or `marley_lsp`.

**Status: Phase 1 — Plan PASS.**

## Phase 2 — Design

### Architecture / approach

The change is confined to `crates/marley_app/src/app.rs` (the pump tick + the spawn shim) plus a
test-only seed helper in `headless_drive.rs`. `marley_lsp` and `lsp_host.rs` are untouched —
`reconcile`/`needs_text` are already correct, and this ticket only changes *when a host comes into
existence*, not what it does afterwards. §14: no new error type (the seam is a `bool`-free `&mut self`
shim over an infallible insert), no panics on a response path, and process-spawn stays where it is —
`LspHost::new` does not spawn; the async spawn happens later inside the host's own lifecycle, which is
exactly why a fresh host is `life: None` (F2).

**Shape decision — (c), neither of the two offered.** The options weighed:
- **(a)** take the already-collected `&[(PathBuf, String, BufferVersion)]` — reuses the walk, but two
  thirds of each tuple (`text`, `version`) are meaningless to a spawn gate, so the signature would
  advertise inputs the fn must ignore. It also welds the ensure to the collect's ordering.
- **(b)** walk `tabs()/editor()/open_docs()` again before the collect — honest inputs, but a second
  tree walk every tick for a question that is `false` on all but one tick in the app's lifetime.
- **(c) CHOSEN — pass the paths only, as an iterator the caller already has.**
  `fn ensure_lsp_host_for_open_docs(&mut self, root: &Path, paths: impl Iterator<Item = &Path>)`.
  The pump derives `paths` from the SAME `open_files` vec it already built (`open_files.iter().map(|(p,
  _, _)| p.as_path())`), so there is no second walk, and the signature states the gate's real input:
  *is any open document a spawn-worthy `.rs` file under this root?* The `text`/`version` fields never
  enter the fn.

**Placement: after the collect, before `reconcile`** (app.rs ~:1431). This satisfies F2's constraint
by construction — `drain()` already ran at :1400, so the tick's Ready phase is settled for every
subsequent read. The borrow is free: `open_files` is an owned `Vec`, so iterating it borrows nothing
from `view`, leaving `&mut view.lsp_hosts` available. (Placing it *before* the collect would also be
sound — F2 proved the empty-text race cannot occur either way — but after-the-collect keeps the
"drain → read state → act" order the pump's own comment argues for, and avoids a reader wondering
whether a host created this tick could be drained this tick.)

**The gate moves verbatim (D3/F1).** Inside, per path: `.rs` extension → `resolve_under_root(&self.
project_root, path)` → `full.starts_with(root)`. The root-level clauses —
`!self.lsp_hosts.contains_key(root)` and `root.join("Cargo.toml").is_file()` — are hoisted OUT of the
per-path loop, because they do not vary per path. That hoist is the design's one deliberate structural
change to the gate, and it makes the "fires once" property **explicit rather than accidental**: with
the host check outside the loop, the fn returns early on every later tick without touching a path, and
with multiple open `.rs` docs the loop is a plain `.any()` that stops at the first match. Under the old
per-path form the same property held only because `contains_key` happened to be re-tested inside the
loop after the first insert. `resolve_binary` + `LspHost::new` then run exactly once, after the
`.any()` succeeds.

**Gesture removal + the retirement (D2/F5).** `ensure_lsp_for_opened_file` is **deleted**, not kept
with a changed input: its name encodes the very coupling this ticket removes, and F5 confirmed its
only caller is the line being deleted. Its doc comment's substance (why the Cargo.toml prerequisite
exists, why a bare-tempdir headless test never spawns) migrates to the new fn. Two comments become
false and are rewritten:
- `open_file_in_viewer` (app.rs:4505-4506) — "every editor-open path funnels through here … the ONE
  spawn trigger" → the truth is that opening a file no longer spawns anything directly; the pump
  notices the new document on its next tick. Worth stating positively, because the *reason* it is
  correct (a restored tab has no gesture to funnel through) is the whole ticket.
- `active_lsp_segment` (app.rs:4407) — "`None` until a `.rs` open creates a host" → `None` until the
  pump observes a spawn-worthy `.rs` document under the active root.

**Reference (§20) confirmed N/A.** A Marley lifecycle bug; no reference-app behavior is being matched.
The pattern is our own #309 precedent (derive from state, not from the event). The Phase-1 sweep found
no shipped dep owns it: `lsp-types 0.97` models *messages* and is deliberately confined to the #310
diagnostics parse seam; there is no `tower-lsp`-style client framework in the tree; gpui/ropey/regex
are not this seam.

### Mutation / coverage homes (verified, not assumed)

- **The pump closure is inside `RootView::new_in` (app.rs:1074), which carries
  `#[cfg_attr(test, mutants::skip)]` (app.rs:1073)** — the same arrangement as #307's arm inside
  `render`. app.rs is on gates.sh's documented coverage exclude. So the pump-side change carries **no
  live mutants and no coverage obligation**.
- **The new `ensure_lsp_host_for_open_docs` is its own `fn`, NOT inside `new_in`** — so it does **not**
  inherit that skip and MUST carry its own `#[cfg_attr(test, mutants::skip)]`, exactly as the fn it
  replaces did. Omitting it would put live mutants in a coverage-excluded file, i.e. mutants that
  cannot be killed by any coverage-visible test — an MSI red with no honest fix. Phase 4 re-runs
  `cargo mutants --list -f crates/marley_app/src/app.rs` to confirm the skip took, because inserting a
  fn adjacent to skipped shims is exactly the documented attribute-detach trap.
- **Consequence for the plan:** every REQ is proven by a headless drive; there is no pure seam here to
  carry cov/MSI. That is correct rather than a gap — the change *is* orchestration.

### File manifest

| File | Change |
|---|---|
| `crates/marley_app/src/app.rs` | ADD `ensure_lsp_host_for_open_docs(&mut self, root, paths)` (`mutants::skip`), carrying the hoisted gate + `resolve_binary` + `LspHost::new`; CALL it in the pump between the `open_files` collect and `reconcile`; DELETE `ensure_lsp_for_opened_file` and its call at :4507; REWRITE the two stale comments; ADD a `lsp_host_exists_for_test(root) -> bool` accessor. |
| `crates/marley_app/src/headless_drive.rs` | ADD `seed_project_with_editor_tab(dir, root, file)` building the blob through `grid_layout::serialize_shell` (see below) + the REQ drives. |

No other files. `lsp_host.rs`, `marley_lsp`, `grid_layout.rs`, `settings.rs` unchanged.

**The seed helper builds a REAL blob, not a hand-rolled one.** `seed_one_project` currently formats
`"0\n{root}\t0\tT=t"` by hand — tolerable for the simplest tier, but the `V=` tier carries a
`\x1f`-separated payload (`<active>\x1f<path>…`) that would be duplicated format knowledge. The
layout types are `pub` (`ShellLayout` :241, `ProjectLayout` :230, `TabLayout::Code { paths, active }`
:207) and `serialize_shell` is `pub` :329, so the helper constructs
`TabLayout::Code { paths: vec![file], active: 0 }` and calls `serialize_shell`, then persists it with
`persist_shell` exactly as the existing helper does. The test then cannot drift from the wire, and if
the codec changes the helper follows for free.

### Regression Test Plan

| REQ | The behavior | Test | Where |
|---|---|---|---|
| REQ-001 | a RESTORED editor tab creates the host with no gesture: seed a `V=<path>` blob over a root WITH `Cargo.toml` → boot → pump → host exists | `restored_editor_tab_spawns_lsp_headless` — the ticket's core; uses the new `serialize_shell`-backed seed helper + `lsp_host_exists_for_test` | headless |
| REQ-002 | the restored doc's didOpen carries NON-EMPTY text once Ready | **Split, and the split is forced — see below.** Headless half: assert `needs_text(path, version)` is false pre-Ready and true once Ready with the doc unseen, i.e. the contract that makes the text real when it is finally sent (F2's mechanism). | headless (partial) |
| REQ-003 | a FRESH gesture-open still creates the host, within one pump tick | `gesture_open_spawns_on_next_tick_headless`: `open_file_in_viewer` → pump → host exists | headless |
| REQ-004 | spawn NOTHING when the gate fails — no `Cargo.toml`, no `.rs` open, or a host already present | `spawn_gate_rejects_headless` (3 arms) + the existing bare-tempdir boots stay spawn-free (they are the standing regression) | headless |
| REQ-005 | the footer `lsp:` segment appears on a restored tab with no further interaction | `active_lsp_segment()` is `Some` after REQ-001's boot+pump | headless |
| REQ-006 | `reconcile`'s didChange/didClose behavior byte-identical (this adds a caller ABOVE it, not logic inside) | the existing #309 tests stay green, unmodified; diff review confirms `lsp_host.rs` is untouched | existing suite |

**REQ-002's live half is genuinely uncoverable this session, and I am not going to dress it up.**
Proving a real `textDocument/didOpen` with `TEXT_LEN>0` on the wire needs either (a) a live
rust-analyzer + the `[[lsp.servers]]` tee — a LIVE drive, off-limits (chad is at the machine), or (b)
an app-level fake-server lane. **That lane does not exist: it is #320's deliverable and #320 is still
OPEN** (confirmed in the open-ticket list). So the wire assertion cannot be made headlessly today.
What carries REQ-002 instead: the F2 mechanism, which is not hand-waving but a read invariant —
`LspHost::new` sets `life: None`; both `needs_text` and `reconcile` bail unless `Phase::Ready`; so the
creation tick sends nothing at all, and the first thing the server ever receives for that document is
a didOpen built from a `needs_text`-materialized text in a tick where the phase is settled. The
headless half above pins the `needs_text` side of that. **Recorded as deferred-not-skipped, with the
blocking dependency named** (#320), so this is a scheduling fact rather than a hole someone has to
rediscover.

### Risks / decisions

- **R1 — active-root scoping is inherited, not introduced.** The ensure is scoped to
  `shell.active_project().root`, exactly like the `reconcile` it sits beside. A restored-but-inactive
  workspace therefore spawns when it is activated — the same tick its doc-sync would begin. This is
  consistency with the existing reconcile, not a new limitation; multi-root simultaneous spawn was
  already out of scope in the spec.
- **R2 — one-tick latency on a fresh gesture-open is unobservable.** Between the open and the next
  pump tick the host is absent, and every pre-host lookup already tolerates that: `active_lsp_segment`
  is `lsp_hosts.get(root).map(...)` → `None`; `active_diagnostic_summary` uses `?` on the same get;
  `workspace_diag_total` sums over `values()` (empty); the F12/hover/completion request paths go
  through `lsp_hosts.get_mut(root).is_some_and(...)` → false. All already handle "no host yet" because
  that is the state during the entire async spawn+initialize window today.
- **R3 — a `Cargo.toml` appearing LATER now summons a host (a behavior GAIN).** The gate is
  re-evaluated each tick, so `cargo init` in an open root gets language support on the next tick
  instead of requiring a file-open gesture. Documented rather than suppressed. Its twin is the
  pre-existing F4 limitation, unchanged by this ticket: a host inserted with `resolved: None` (binary
  missing) is never re-resolved, because `contains_key` short-circuits.
- **R4 — the `mutants::skip` attribute-detach trap.** Adding a fn near skipped shims can rebind an
  attribute; Phase 4 must confirm via `cargo mutants --list` rather than trusting the source read.
- **R5 — parity with #355.** Whatever `open_docs()` enumerates is what can summon a host, so when
  editable pane surfaces join that enumeration, spawn widens with sync automatically. No pane
  special-casing here.

**Status: Phase 2 — Design PASS.**

## Phase 3 — Implement

Built to the manifest in two groups; `cargo check -p marley --all-targets` clean (0 errors),
`cargo fmt --all` applied.

### Group 1 — `crates/marley_app/src/app.rs`

- **ADDED `ensure_lsp_host_for_open_docs<'a>(&mut self, root: &Path, mut paths: impl Iterator<Item =
  &'a Path>)`** carrying `#[cfg_attr(test, mutants::skip)]` (the design's explicit requirement — it is
  not inside `new_in`, so it inherits nothing). The gate is hoisted exactly as designed: the two
  root-level clauses (`contains_key` / `Cargo.toml`) return early BEFORE any path is touched, then a
  single `paths.any(...)` tests `.rs` extension AND `resolve_under_root(...).starts_with(root)`. On a
  hit, `resolve_binary` + `LspHost::new` run once. The doc comment carries the migrated substance
  (why the Cargo.toml prerequisite exists; why a bare-tempdir headless boot never spawns) plus the
  new fact that creation derives from open-doc STATE, and states the "fires at most once per root"
  property as structural rather than incidental.
- **CALLED from the pump** between the `open_files` collect and the `reconcile` block, as
  `open_files.iter().map(|(path, _, _)| path.as_path())`. No borrow fight: `open_files` is an owned
  `Vec`, so iterating it holds nothing from `view`. The call site carries a comment naming the
  after-`drain()` requirement AND why a host created there is harmless that tick (`life: None` ⇒ both
  `needs_text` and `reconcile` decline it), so the F2 reasoning lives next to the code that depends on
  it rather than only in this file.
- **DELETED `ensure_lsp_for_opened_file`** entirely, and its call inside `open_file_in_viewer`.
- **REWROTE the two false comments.** `open_file_in_viewer`'s "the ONE spawn trigger" now says
  opening a file no longer spawns anything directly and names the restored tab as the reason the
  gesture could not remain the trigger. `active_lsp_segment`'s "`None` until a `.rs` open creates a
  host" now says "until the pump observes a spawn-worthy `.rs` document under the active root".
- **ADDED `lsp_host_exists_for_test(&self, root: &Path) -> bool`** under `#[cfg(test)]`, matching the
  house idiom used by `font_size_for_test` / `git_marks_for_test` (checked before writing).
- **A THIRD stale reference, found by grep rather than by the compiler.** `send_definition_request`'s
  doc (app.rs ~:9637) explained the no-host case in terms of `ensure_lsp_for_opened_file` — a function
  that no longer exists. Backticked prose, so nothing would ever have failed on it. Repointed at
  `[`Self::ensure_lsp_host_for_open_docs`]` with the wording corrected to "when an open document is a
  `.rs` file under a root with a `Cargo.toml`". A repo-wide sweep confirms the only remaining mentions
  are in ARCHIVED notes for #308/#309/#312, which are point-in-time records — correct to leave.

### Group 2 — `crates/marley_app/src/headless_drive.rs`

- **ADDED `seed_project_with_editor_tab(dir, root, file)`** beside `seed_one_project`. It builds a
  `ShellLayout { active_project: 0, projects: [ProjectLayout { root, active_tab: 0, tabs: [TabLayout::
  Code { paths: [file], active: 0 }] }] }` and serialises it with `grid_layout::serialize_shell`,
  then persists via `persist_shell` — so the fixture cannot drift from the wire. Field names were read
  from `grid_layout.rs` first rather than guessed.

### Deviations from design

None. One thing the design did not anticipate, handled above: the third stale doc reference (app.rs
~:9637). It is the same class the design already flagged for the two known comments, so it is a wider
instance of a named risk rather than a new decision.

### Known-and-expected: two `never used` warnings

`lsp_host_exists_for_test` and `seed_project_with_editor_tab` currently have no callers, so the check
reports two `never used` warnings. **These would fail gate:2 (`clippy -D warnings`) if left**, and are
resolved by Phase 4, which is what consumes both. Recorded here so inspect does not report them as
defects and validate does not mistake them for pre-existing noise.

**Status: Phase 3 — Implement PASS.**

## Phase 3.5 — Inspect

Two critics were dispatched (lens 1 LSP lifecycle/tick ordering, lens 2 gate equivalence + hygiene).
**Process note, recorded honestly (§15): neither returned a report before this phase closed** — their
transcripts sat at 156 bytes for >10 minutes, i.e. they never got going. Rather than block the
pipeline a second time on subagent latency, I ran **both lenses myself** against live code; every
finding below carries the command or read that produced it. This is the second ticket running where
critic latency has been the long pole (#307's took ~14 min each), which is worth noting as a process
cost rather than a per-ticket accident.

### Findings

| # | Severity | Finding | Verdict | Resolution |
|---|---|---|---|---|
| F1 | **MEDIUM** | The designed gate hoist put a **syscall ahead of the in-memory checks**, on a path that runs EVERY pump tick | **REAL (perf regression), FIXED** | Phase 2 specified `contains_key(root) \|\| !root.join("Cargo.toml").is_file()` as one early return. The second half is a filesystem stat, so any project without a host would stat `Cargo.toml` on every single tick, forever — where the OLD gesture-triggered code stat'd only when a `.rs` file was actually opened. Reordered to cheapest-first: `contains_key` (in-memory, retires the question permanently once a host exists) → the `.any()` path scan (in-memory, a handful of open docs) → the manifest stat LAST, reached only when a candidate `.rs` doc exists. Behavior is unchanged; only the cost is. The stat deliberately stays *inside* the per-tick path rather than being cached, because that re-evaluation is what delivers R3 (a root that gains a `Cargo.toml` later gets a host on the next tick) — so it cannot be hoisted out, which is exactly why it must come last. Both facts are now comments at the site. |
| F2 | **MEDIUM** | The Phase-4 fixture would have been **vacuous**: a seeded `V=` tab whose file does not exist on disk restores to NOTHING | **REAL (test validity), directed at Phase 4** | Read the restore path (app.rs:1715-1766): every persisted path is re-read from disk, unreadable ones are skipped, and `if !readable.is_empty()` gates tab creation — so with no file on disk the editor tab is dropped entirely. REQ-001 would then find no host and PASS, for precisely the wrong reason. **Phase 4 must `std::fs::write` the fixture file before booting.** Two companion facts confirmed while tracing: a `V=`-only project gains a spawned terminal tab at index 0 (the ≥1-terminal invariant, app.rs:1770), so the editor tab is not index 0; and the pump's collect walks `tabs().iter().filter_map(\|tab\| tab.editor())` — ALL tabs — so tab ordering does not affect the mechanism. |
| F3 | LOW | A **third** stale reference to the deleted `ensure_lsp_for_opened_file`, in prose the compiler cannot check | **REAL, FIXED** | `send_definition_request`'s doc (app.rs ~:9637) explained the no-host case in terms of the deleted fn. Backticked prose, so nothing would ever have failed on it — the "a false doc is well-formed Rust" class. Repointed at `[`Self::ensure_lsp_host_for_open_docs`]` with the wording corrected to "when an open document is a `.rs` file under a root with a `Cargo.toml`". A repo-wide sweep confirms the only surviving mentions are in ARCHIVED notes for #308/#309/#312, which are point-in-time records — correct to leave untouched. |
| F4 | — | Is `#[cfg_attr(test, mutants::skip)]` actually bound to the new fn, or detached by the adjacent insertion? | **CORRECTLY BOUND (measured)** | `cargo mutants --list -f crates/marley_app/src/app.rs \| grep -c ensure_lsp` → **0**, both before and after the F1 edit. Neighbouring fns still generate mutants (`refresh_efind_matches`, `accept_syntax_resp`, `needs_syntax_refresh`), so the skip did not over-apply either. The documented detach trap did not bite. |
| F5 | — | Does the restructured gate change the observable outcome ("is a host created?") for any input? | **IDENTICAL except the intended widening** | Enumerated: no `.rs` under root → neither spawns. `.rs` under root with no manifest → neither. `.rs` under root with a host present → neither. `.rs` under root, manifest, no host → both spawn. Zero open docs → `.any()` on empty is false → no spawn, matching "no gesture, no spawn". **The one difference is the fix itself:** the old form only ever examined the single gestured path, so with a `.rs` already open, opening a `.md` spawned nothing; the new form scans all open docs. That is the restored-tab case, i.e. the ticket. |
| F6 | — | Can the recorded empty-`didOpen` corruption (`BF-lsp-didopen-carries-empty-text-ready-race-001`) recur now that host creation joins this tick? | **NO (re-derived from the code)** | `LspHost::new` sets `life: None`; both `needs_text` and `reconcile` open with `life.as_ref().map(Lifecycle::phase) != Some(Phase::Ready) → bail`. So on the creation tick `reconcile` sends **nothing at all** — not an empty didOpen. The first thing the server ever receives for a doc is a didOpen built from a `needs_text`-materialized text in a tick where `drain()` has already settled the phase. The vec is rebuilt from scratch each tick, so the creation tick's `""` text is discarded rather than cached. |
| F7 | — | Does the seed helper's blob actually round-trip, or would the fixture restore to nothing? | **ROUND-TRIPS (verified)** | `serialize_code_paths` always emits the `<active>\x1f<path…>` form, even for a single path (grid_layout.rs:269-287); `parse_code_paths` takes the `contains('\x1f')` branch and reconstructs `(paths, active)` (grid_layout.rs:292-302). The bare-path branch is pre-#243 back-compat only. Combined with F2's on-disk requirement, the fixture is sound. |
| F8 | — | Does a restored `V=` tab actually surface docs to the pump's collect (the whole ticket's premise)? | **YES (traced end to end)** | `TabLayout::Code` → re-read files → `Tab::code_surface(EDITOR_TAB_TITLE, EditorSurface::from_files(...))` (app.rs:1763) → `tab.editor()` is `Some` → `open_docs()` yields its paths → the pump's `open_files` → the new gate. The chain is intact. |

**Rejected / not findings:** the two `never used` warnings (`lsp_host_exists_for_test`,
`seed_project_with_editor_tab`) are expected and consumed by Phase 4 — recorded at Phase 3, not
defects. No `unwrap`/`expect` added on any input-reachable path; no secrets; no process-spawn
introduced (`LspHost::new` does not spawn — that is why a fresh host is `life: None`). §20 clean: no
Zed/Warp brand names in `crates/`, no copyleft source read. The intra-doc link sits on a private fn
and gate:14's rustdoc runs without `--document-private-items`, so it can neither resolve nor fail
there (the #307 finding).

**Status: Phase 3.5 — Inspect PASS.**

## Phase 4 — Validate

### The critics landed here, and they changed the tests substantially

Both #321 critics returned DURING this phase (after the inspect ledger had closed), and between them
they found **two CRITICALs in the tests I had just written**. Recorded here rather than back-dated into
the inspect ledger, because that is where the work actually happened.

| # | Severity | Finding | Verdict | Fix |
|---|---|---|---|---|
| V1 | **CRITICAL** | **The drives never ticked the pump**, so nothing under test ever ran | **REAL, FIXED** | `run_until_parked()` does NOT advance gpui's mock clock: `TestDispatcher::run_until_parked` is `while self.tick(false) {}`, and `tick` only promotes a delayed task once `state.time` reaches its deadline — moved solely by `advance_clock`. The pump is a `timer(PUMP_INTERVAL_MS = 16ms)` loop and, since #321, the ONLY creator of a host. So the pump body never ran and `lsp_hosts` stayed empty. The positive asserts would have failed — but the dangerous half is that `spawn_gate_rejects_headless`'s arms (a) and (b) assert *absence* and so **passed vacuously**, proving nothing about the gate they exist to test. Fixed with a shared `tick_pump(cx)` (`advance_clock(40ms)` + `run_until_parked()`), applied to EVERY arm including the negatives, with a doc explaining why the bare form is insufficient. The repo already had this idiom in 5 places; I had not reused it. |
| V2 | **CRITICAL** | The fixture's `active_tab: 0` selects the **force-inserted terminal**, not the editor tab — so the anti-vacuity guard itself failed | **REAL, FIXED** | Restore force-inserts a terminal at index 0 when a project has none (the ≥1-terminal invariant), shifting the editor tab to index 1; `switch_tab(active_tab.min(last))` with `0` then activates the TERMINAL, whose `editor()` is `None`. Every `assert!(restored, "…else this test is vacuous")` would have failed. Fixed by seeding the terminal EXPLICITLY with `active_tab: 1` — which also makes the fixture faithful: the never-empties invariant means the app can never persist a `V=` project line with no `T=`, so the old blob imitated a shape production never writes. |
| V3 | **HIGH** | `write_cargo_fixture` defeated #308's protection and the drives launched a **real rust-analyzer** | **REAL, FIXED (found independently, then confirmed by a critic)** | #308 relied on the `Cargo.toml` gate to keep headless tests from spawning a server ("the reason a bare-tempdir headless test never spawns one"). These drives must WRITE a manifest to exercise the gate, re-opening that hole — and `rust-analyzer` is installed at `~/.cargo/bin` on this machine, so `resolve_binary`'s PATH search found it and `LspHost::new` spawned it **synchronously** against a tempdir. The test binary then sat at 0% CPU indefinitely. **The symptom was indistinguishable from the machine's known syspolicyd fault**; I separated them by checking `ps -o %cpu -p $(pgrep syspolicyd)` (idle at 0%) and then confirming rust-analyzer was on PATH. Fixed by seeding `[[lsp.servers]]` at a nonexistent absolute command via a new `settings::persist_language_servers`: `resolve_binary` short-circuits on the configured entry, the existence probe fails, so the host is still CREATED (what the drives assert) with `resolved: None` and nothing spawns. The spec's REQ-001 had said "Cargo.toml + `[[lsp.servers]]` naming a stub binary" — I had implemented the manifest half and dropped the stub half. |
| V4 | MEDIUM | The new fn's doc claimed "however a document came to be open, the host follows" | **REAL, FIXED** | False for #259 split-pane editors: the pump's set is `tabs().filter_map(tab.editor())`, and `Tab::editor()` matches `TabContent::CodeView` only — a file opened via `split_file_pane` lives in `PaneContent::CodeView` **inside a terminal tab** and never contributes. Not a regression (the deleted fn was never called from there either), but the doc asserted a property the code lacks. Narrowed to "in an editor TAB of the ACTIVE project". |
| V5 | MEDIUM | `resolve_under_root(&self.project_root, …)` resolves against a FIELD while testing containment against the `root` PARAMETER | **REAL (latent), DOCUMENTED** | Carried verbatim from the deleted fn, where `root` was a local clone of the active root so the two were provably equal. Now that `root` is a parameter and the doc says "one host per project root", a future caller looping every project would resolve non-active roots' relative paths against the wrong base. Unreachable today — every path from `open_docs()` is already absolute, so `resolve_under_root` is the identity — so this is a documented trap rather than a code change. |
| V6 | LOW | The pump comment said "both `needs_text` and `reconcile` decline it this tick" | **REAL, FIXED** | `needs_text` is never *reached*: the collect runs BEFORE the ensure, so `lsp_hosts.get()` is `None` and `is_some_and` short-circuits. Right outcome, wrong mechanism — again. Reworded. |
| V7 | LOW | A real behavioral delta the spec's D3 ("the quadruple moves verbatim") does not cover | **REAL, RECORDED** | The old call sat BEFORE `load_code_view_state`, so opening a binary/oversized/unreadable `.rs` spawned a host anyway. The new derivation only sees documents that actually opened, so that case now spawns nothing. Strictly an improvement, but an observable change in "is a host created?" that the spec claimed was verbatim. |

**Also fixed pre-emptively (found by running `cargo mutants --list` before the gate rather than after):**
`persist_language_servers` lives in `settings.rs`, a PURE mutation-included file, and generated a live
`replace with Ok(())` mutant. The existing round-trip test used `manager.set` directly and would not
have killed it → a gate:5 red. Rewired that test to persist through the helper, matching the house
idiom (`persist_workflows`/`persist_recents` round-trips exist partly to kill exactly this mutant).

### Tests added — and proven non-vacuous

All in `crates/marley_app/src/headless_drive.rs`; app.rs is coverage-excluded and the new fn is
`mutants::skip`'d (`cargo mutants --list -f app.rs | grep -c ensure_lsp` → **0**, re-verified after
every edit), so these drives are the only proof.

- `restored_editor_tab_spawns_lsp_headless` — REQ-001 + REQ-005 + REQ-002(headless). Asserts the tab
  restored (anti-vacuity), the host exists, the footer segment shows, and the not-yet-Ready host has
  didOpen'd **nothing** (`lsp_open_doc_count_for_test == Some(0)`) — the invariant that makes an
  empty-text didOpen unreachable.
- `gesture_open_spawns_on_next_tick_headless` — REQ-003; the gesture path still works, one tick later.
- `spawn_gate_rejects_headless` — REQ-004, three arms (no manifest / no `.rs` open / idempotence
  across repeated ticks), each with its own anti-vacuity guard.

Supporting seams: `tick_pump`, `write_cargo_fixture`, `seed_stub_language_server`,
`seed_project_with_editor_tab` (built through `serialize_shell`), plus three `#[cfg(test)]` hooks
(`lsp_host_exists_for_test`, `lsp_segment_for_test`, `lsp_open_doc_count_for_test`) and
`LspHost::open_doc_count`.

### Results (ACTUAL)

```
cargo test -p marley --lib -- headless_drive
test result: ok. 158 passed; 0 failed; 0 ignored; 544 filtered out

  test headless_drive::restored_editor_tab_spawns_lsp_headless ... ok
  test headless_drive::gesture_open_spawns_on_next_tick_headless ... ok
  test headless_drive::spawn_gate_rejects_headless ... ok

cargo test -p marley --lib -- settings::tests::language_servers
test result: ok. 1 passed; 0 failed
```

The two `never used` warnings from Phase 3 are gone (Phase 4 consumes both helpers), so gate:2's
`-D warnings` is satisfied.

**NEGATIVE SMOKE — the check that would have caught V1 and V2 on its own.** With the
`ensure_lsp_host_for_open_docs` call commented out of the pump and nothing else changed:

```
test headless_drive::restored_editor_tab_spawns_lsp_headless ... FAILED
test headless_drive::gesture_open_spawns_on_next_tick_headless ... FAILED
test headless_drive::spawn_gate_rejects_headless ... FAILED
test result: FAILED. 0 passed; 3 failed
```

All three fail — **including the gate-rejection test**, which is the one that had been passing
vacuously. The call site was restored immediately afterwards and verified back in place. Given that
two of this ticket's defects were "green tests that could not detect the feature's absence", this
smoke is the evidence that matters more than the green run.

### REQ-002's wire half — deferred, with a NAMED blocker

Proving a real `textDocument/didOpen` with `TEXT_LEN>0` needs either a live drive (off-limits — chad
is at the machine) or an app-level fake-server lane. **That lane does not exist: it is #320's
deliverable and #320 is still open.** Not faked, not hand-waved. What carries it meanwhile: the
headless half above pins that a not-yet-Ready host has sent nothing, and the Ready-gate invariant
(re-derived independently by a critic: `Ready` is reachable only via `(Initializing,
InitializeResult)` ← `on_message` ← `drain()`) guarantees the first thing the server receives is built
from a materialized text.

### REQ-006 — no new test

`lsp_host.rs` gained only a `#[cfg(test)]` accessor and a doc correction; `reconcile`'s logic is
untouched, and the existing #309 doc-sync tests stay green unmodified.

### The gate: a legitimate clippy red, fixed at source

The first full gate came back **14/15** — coverage 100%, MSI, miri, visual, docs, secrets, audit,
deny, machete, shellcheck all green — with clippy the one red: `persist_language_servers` was a
`pub fn` with **no production caller** (`[[lsp.servers]]` is hand-edited by the user and only ever
READ), so in the non-test build it is dead code, which `-D warnings` rightly rejects. Fixed at source
by gating it `#[cfg(test)] pub(crate)`, which is honestly what it is — a test-support seam. NOT
suppressed. The re-run then came back **GATE GREEN [diff] 15/15**; mutation reads "no mutable lines in
the diff — pass" because the production surface is the `mutants::skip`'d app.rs and the one previously-
mutable line (`persist_language_servers`) is now test-only. The proof for this ticket is the drives +
the negative smoke, as designed. Receipt `ded8a592…`, verified against `gate_state_hash`.

**Status: Phase 4 — Validate PASS.**

## Phase 5 — Complete

**Docs (§21):** `CHANGELOG.md` gained a `### Fixed` entry (the silent-no-LSP symptom, the
state-derived creation, the gate reorder rationale, the empty-didOpen invariant with its
"no drain() between the ensure and the reconcile" warning). `docs/marley_architecture/crate-map.md`'s
`marley_lsp` row extended the existing doc-sync-ordering paragraph with #321's host-creation change,
the gate ordering, the manifest-vs-binary asymmetry, and the constructor-post-state rule.

**Capture (forge):** AAR `55b2817a` submitted — completed, effectiveness 4, 6 novel findings.
Materialized across inspect + validate:

| Code | What it pins |
|---|---|
| `BF-claude-gate-moved-to-a-per-tick-path-kept-its-syscall-first-001` | The hoisted gate put a `stat` ahead of the in-memory checks on a per-tick path |
| `PR-claude-relocating-a-guard-to-a-hot-path-reorder-cheapest-first-001` | Moving a guard from a cold path to a hot one makes clause ORDER a cost decision |
| `BF-claude-headless-test-satisfied-the-spawn-gate-and-launched-a-real-server-001` | Writing a `Cargo.toml` to exercise the gate defeated #308's protection → a real rust-analyzer spawned |
| `BF-claude-headless-drive-never-ticked-the-pump-negatives-passed-vacuously-001` | `run_until_parked()` doesn't advance the mock clock; the pump never ran; the negative asserts passed vacuously |
| `PR-claude-a-negative-assert-must-prove-the-machinery-ran-001` | A "did not happen" assert must first prove the machinery ran |
| `PR-claude-read-to-the-end-of-the-constructor-before-asserting-post-state-001` | The struct literal's `life: None` is not `LspHost::new`'s post-state; it drives to `Initializing` before returning |

**Follow-ups filed earlier (not re-filed):** #358 (⇧Tab same-row cursor merge, from #307), #359
(workspace close leaks its LSP host — pre-existing, widened by #321). Evidence added to #348 that its
stated root cause may be misattributed (the machine's syspolicyd fault produces the identical
SLOW-forever nextest signature without the PTY test running).

**The lessons worth carrying forward:**
1. **The critics earned more here than on #307.** Two independent CRITICALs made the tests
   non-functional *while looking green* — the pump never ticked, and the fixture activated the wrong
   tab. Neither a green compile nor a green test run showed it. The **negative smoke** (delete the
   feature → all three fail, including the previously-vacuous negative arm) is what converted "the
   tests pass" into "the tests mean something", and it is now the thing I run whenever a test's job is
   to prove a gate REFUSES.
2. **Read the REQ's verify clause as a spec.** REQ-001 said "Cargo.toml + `[[lsp.servers]]` naming a
   stub binary". I built the manifest half and dropped the stub half, and the drives launched a real
   rust-analyzer whose hang was indistinguishable from the machine's OS fault. The verify clause was
   not a hint.
3. **"Right conclusion, wrong reason" hit a fourth time this batch** — the `life: None` claim. The
   conclusion (no empty didOpen) held; the mechanism was false. Read to the end of the constructor.
4. **Running `cargo mutants --list` BEFORE the gate** caught the `persist_language_servers` mutant that
   would have been a gate:5 red — cheaper than discovering it 20 minutes into a run the OS fault kept
   stalling.

**Ticket + archive:** forge #321 closed done; `TICKET-321` → `tickets/closed/` (Status: closed); this
pair archived to `pipeline/completed/`.

**Status: Phase 5 — Complete PASS.**
