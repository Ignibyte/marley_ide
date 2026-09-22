# LSP — drain every host's outcomes per pump tick (#413) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-413-lsp-drain-all-hosts-outcomes.md
- **Pipeline spec:** 413-lsp-drain-all-hosts.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan

- **Request:** `/work next` → top Queue row: TICKET-413 — LSP: drain every
  host's outcomes per pump tick, route by owning root (#332 inspect follow-up,
  filed by the state critic as F4 with the blast radius named).
- **Classification / tier:** chore, standard work pipeline, one shippable
  slice (drain-all + per-arm sweep + fan-in completion). M20 LSP train.
- **Recall (§18.3):**
  - #332 notes (completed archive): pump order per 16 ms tick is
    `host.drain()` for ALL hosts → `consume_lsp_responses(active_root)` →
    trigger consumers. "ONE drain, routed by purpose; each arm owns its
    stale-guard" is law (comment at the consume site). F4 verdict: REAL,
    CONTAINED — the destruction half died with F1 (teardown delivers, never
    destroys; nothing is lost, only deferred); the deferral half is THIS
    ticket, with the per-consumer blast radius named.
  - PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001
    (prevention rule, medium): a shared-plumbing fix's "siblings are
    unaffected" claim is the least-verified sentence in the report — check
    consumer by consumer. Binding shape for this pipeline: the per-arm table
    IS the deliverable.
  - #332 as-built test substrate we can reuse at validate:
    `drain_lsp_ticks_for_test`, `ingest_message_for_test`, deterministic
    ascending `take_responses`, the fake-server lane (#320), and
    `abandon_all` ordering tests in rpc.rs.
  - #359 (workspace-close host drop): lsp_hosts is
    `HashMap<PathBuf, LspHost>`; closing the last workspace on a root removes
    its host — so the drain-all loop must tolerate hosts disappearing between
    ticks (no staleness assumptions).
- **Discovery (the precise edit/file surface for Design):**
  - `crates/marley_app/src/app.rs`
    - pump (~:1926–2010): `active_root` cloned; `for host in
      view.lsp_hosts.values_mut() { host.drain() }` (ALL hosts, wire in);
      `view.consume_lsp_responses(&active_root)` (:1995 — the active-only
      outcome drain this pipeline generalizes); trigger consumers
      (`consume_completion_trigger`, `consume_signature_trigger`, symbol
      fan-out) stay active-root by design (send policy).
    - `consume_lsp_responses` (:12672–12723): `lsp_hosts.get_mut(root)` →
      `take_responses()` → flat 12-arm `match` on `RequestPurpose`, every arm
      but `WorkspaceSymbol` takes `root`. The 12 arms and their handlers:
      Hover→`apply_hover_response`, Definition→`apply_definition_response`,
      References→`apply_references_response`,
      Completion→`apply_completion_response`,
      PrepareRename→`apply_prepare_rename_response`,
      Rename→`apply_rename_response`, CodeAction→`apply_code_action_response`,
      CodeActionResolve→`apply_code_action_resolve_response`,
      SignatureHelp→`apply_signature_help_response`,
      WorkspaceSymbol→`apply_workspace_symbol_response` (NO root),
      InlayHints→`apply_inlay_response`, Formatting→`apply_formatting_response`.
    - ⌘T fan-out `consume_symbol_query` (:13138–13165): sends to EVERY Ready
      host with `workspace_symbol_support()`; fan-in
      `apply_workspace_symbol_response` (:13170–13195): stale-gen key guard →
      parse → `results.extend` → cap ×8 — root-agnostic, merge-shaped,
      currently starved of non-active answers.
  - `crates/marley_app/src/lsp_host.rs`: `responses:
    Vec<(RequestPurpose, RequestOutcome)>` (:144), `drain()` (:311 — wire +
    expiry + teardown sweep), `take_responses()` (:591). `RequestPurpose`
    enum (:43).
  - `crates/marley_lsp/src/rpc.rs`: `RequestOutcome` (:193),
    `abandon_all` (:216, ascending-id delivery).
- **Decisions:** D1 routing-not-filtering; D2 per-consumer sweep mandatory
  (12/12 recorded + tested); D3 one drain site, sorted-root deterministic
  order; D4 fan-in un-starved not redesigned. See spec. Milestone M20 (ticket
  said M-unset; the LSP train is M20 per #354/#359).
- **Prior-art sweep (§20 step 4):** Zed behavior map — responses resolve on
  arrival per server, focus-independent (the exact behavior we're adopting);
  LSP 3.17 — correlation is per-connection by id, no focus semantics in the
  protocol; deps — no owner (marley_lsp is our clean-room client; lsp-types
  types-only). Nothing dissolved a decision; the sweep CONFIRMS D1 is
  protocol-normal behavior, not an invention.

## Phase 2 — Design

### Architecture / approach

**Mechanism (settles the plan's P2 fork):** `consume_lsp_responses` loses its
`root: &Path` param and iterates internally — collect-then-dispatch:

```
fn consume_lsp_responses(&mut self) -> bool {
    // collect: one take_responses per host (iter_mut), keep non-empty
    //   batches as (root.clone(), Vec<(purpose, outcome)>)
    // sort batches by root (deterministic, #396 AAR ordering rule; ids
    //   already ascend within a host — #332)
    // dispatch: the SAME flat 12-arm match, each arm receiving the OWNING
    //   batch root (WorkspaceSymbol stays rootless)
}
```

- Collect-then-dispatch separates the `lsp_hosts.iter_mut()` borrow from the
  `&mut self` arm calls — no aliasing (the
  PR-claude-two-mut-self-accessors idiom); `take_responses` moves the data out.
- Idle tick: zero non-empty batches → empty Vec (no heap alloc), sort free,
  loop skipped → returns false (REQ-006). Non-idle: one `PathBuf` clone per
  host WITH outcomes per tick — the pump already clones `active_root` every
  tick, so this is noise.
- The pump call site drops the arg: `view.consume_lsp_responses()`. The pump's
  `active_root` stays for reconcile / `ensure_lsp_host_for_open_docs` /
  trigger consumers (send policy, out of scope).
- The law comment is amended, not retired: "ONE drain, routed by purpose — now
  for EVERY host: outcomes deliver on the tick they arrive, routed with the
  OWNING root; each arm owns its stale-guard AND its non-active-root posture."
- Hosts can't vanish between collect and dispatch (removal happens in
  `close_project_at`, a user gesture — never inside an arm). #359 recall noted
  hosts disappear BETWEEN ticks; per-tick iteration handles that by
  construction.
- No `marley_lsp` / `lsp_host.rs` changes: `take_responses` (:591),
  `drain` (:311), `abandon_all` (rpc.rs:216) already deliver per-host,
  ascending, terminal-guaranteed (#332).

**§20 reference confirmed:** matches the Zed behavior map (responses resolve
on arrival per owning server, focus-independent; per-feature guards decide
visibility). N/A→no; reference stands as planned. No Zed source read.

**Lifecycle fact the sweep rests on (verified):** `sync_active_project`
(app.rs:8138) re-walks files only — NO LSP latch or overlay is cleared on a
project switch (`close_transient_overlays` runs on rail-click menu paths, not
on switch). So cross-root outcomes WILL reach arms with latches intact; each
arm's own guard is the sole defense. That is why the sweep below is the work.

### The per-arm sweep (D2 — 12/12, each verified in source)

Legend: **DROP** = the arm's existing guard drops a non-active-root outcome
(correct: stale-question semantics); **APPLY** = the arm deliberately has no
focus guard (committed gesture / origin-targeted / global surface) and must
receive the OWNING root to be correct.

| # | Arm (handler, line) | Guards read | Non-active-root verdict |
|---|---|---|---|
| 1 | Hover (`apply_hover_response` :12729) | latch `hover_request`; active editor path via `uri_for(host(root))`; caret | **DROP** — `uri_for` under owning root canonicalizes the focused (other-root) path to None → `file_matches` false → card cleared/none shown. No flash possible cross-root. |
| 2 | Definition (`apply_definition_response` :12844) | latch; focused-uri guard (:12871) | **DROP** — the guard's own doc (:12860) names the reason: a slow answer must not YANK the user; cross-root is the same class. No jump, no picker, no "No definition found" flash (post-guard). Abandoned: quiet. |
| 3 | References (`apply_references_response` :12989) | latch; focused-uri guard (:13012) | **DROP** — no picker. Abandoned returns true to repaint away the searching card; that card's draw gate (:10762, :15293) reads the ACTIVE root's host, so it never draws for a backgrounded root anyway — clearing the latch early is strictly cleaner. `fetch_reference_texts` only runs post-guard (owning==active), so its `active_root` read stays coherent. |
| 4 | Completion (`apply_completion_response` :14520) | latch; live version vs `key.version` (:14546); uri guard (:14549) | **DROP** — version and uri guards both fail cross-root; no menu. Abandoned: silent, latch cleared. |
| 5 | PrepareRename (`apply_prepare_rename_response` :11940) | latch; focused-uri guard (:11962) | **DROP** — no draft modal, and the "Cannot rename this" decline flash is post-guard (can't fire cross-root). Abandoned: quiet latch clear. |
| 6 | Rename (`apply_rename_response` :12026) | latch ONLY — deliberately no focus guard (:12022 doc: edits the NAMED files by uri) | **APPLY with owning root** — `apply_workspace_edit(root, …)` takes encoding + `lsp_version_for` from the root's host (:12081, :12093): with the OWNING root these are correct; with the active root the version lookup would MISS and skip every file. File routing is already root-global: open buffers via the all-roots registry scan (:12160), closed files canonicalized-within-ANY-workspace-root (:12133). Summary flash while another project is focused = honest completion notice (existing surface). Abandoned: honest "didn't respond" flash — REQ-003's subject. |
| 7 | CodeAction menu (`apply_code_action_response` :12282) | latch; focused-uri guard (:12302) | **DROP** — no menu, no flashes (all post-guard). Abandoned: quiet (a 10 s-late toast is noise — #332's own words). |
| 8 | CodeActionResolve (`apply_code_action_resolve_response` :12368) | latch ONLY — no focus guard (:12366 doc: like rename) | **APPLY with owning root** — same `apply_workspace_edit` reasoning as arm 6. The user CHOSE the action; completing it in its background root is the committed-gesture semantics. Abandoned: honest flash (kept). |
| 9 | SignatureHelp (`apply_signature_help_response` :12515) | latch; focused-uri guard (:12537); caret-row guard | **DROP** — no card. Abandoned dismisses `signature_card`: safe cross-root — a still-open card can only belong to the same request lineage (any newer request overwrote the latch and fails guard 1), so at worst it dismisses this request's own stale card. |
| 10 | WorkspaceSymbol (`apply_workspace_symbol_response` :13170) | stale-gen key vs `symbol_request` — root-agnostic BY DESIGN | **APPLY (rootless)** — the merge (`extend` + ×8 cap :13186) is the fan-in this pipeline un-starves; multi-tick arrivals from multiple roots are the intended shape (D4 confirmed: gen bumps per query change; a stale gen drops; the picker-closed case returns false). |
| 11 | InlayHints (`apply_inlay_response` :4913) | latch; active version vs `key.version` (:4932); uri guard (:4935) | **DROP** — hints for a non-visible viewport are useless; the guard drops them, the cleared latch + `has_pending_inlay` belt (#331) re-requests when that root re-activates. |
| 12 | Formatting (`apply_formatting_response` :9183) | latch; then ORIGIN-targeted (#354): parked-latch origin or latchless uri→instance PINNED to the responding root (:9212, inspect F3) | **APPLY with owning root** — the arm was BUILT for background completion (#354: "a mid-format switch formats the ORIGIN, never whatever is now focused"). Owning root makes the latchless F3 pin correct cross-root; `origin_is_active` false → no caret re-seat (:9254); parked save settles on the origin id (:9292). Abandoned: settles the parked save (never lose a ⌘S). |

Sweep summary: **8 DROP arms** are correct today because every focus guard
evaluates against the OWNING root's `uri_for`, which cannot match a
focused file from another root; **4 APPLY arms** (Rename, CodeActionResolve,
WorkspaceSymbol, Formatting) are exactly the arms whose semantics are
committed/global/origin-targeted — and two of them (6, 8) are only correct
BECAUSE the owning root is passed (version lookup + encoding). No arm needs a
body change; the entire diff is the drain/dispatch site. The per-arm TESTS pin
each verdict.

### File manifest

- `crates/marley_app/src/app.rs` — `consume_lsp_responses`: drop the `root`
  param, collect-then-dispatch over all hosts sorted by root; amend the law
  comment; pump call site (:1995) drops the arg. Any test helper that calls
  `consume_lsp_responses(&root)` updates to the new signature (grep at
  implement).
- `crates/marley_app/src/app.rs` `#[cfg(test)]` — the new tests (below), next
  to the existing #322/#332/#354 app-level siblings.
- No other crate changes (marley_lsp / lsp_host.rs untouched).

### Regression test plan

App-level, headless, on the #320 fake-server lane + #332 helpers
(`ingest_message_for_test`, `drain_lsp_ticks_for_test`, `set_ready`), with two
workspace roots where marked (the #359 multi-root posture). No uncoverable
paths — everything below runs headless.

| REQ / Arm | Test (name, intent) |
|---|---|
| REQ-001 | `all_hosts_outcomes_drain_in_one_tick` — two roots, an answer queued on each host; ONE consume pass → both hosts' `take_responses` empty after (second take empty). |
| REQ-002 + Arm 6 | `nonactive_rename_answer_applies_under_owning_root` — rename latched for root A; B active; A's `WorkspaceEdit` answer drains → A's buffer text edited (registry instance), summary flash set. Also pins: version lookup via A's host (the active-root MISS would skip). |
| REQ-003 | `background_abandonment_flashes_within_one_tick` — rename latched on A; switch to B; tick A's host to expiry+1 → "Language server didn't respond" flash present while B is active (not deferred to switch-back). |
| REQ-004 + Arm 10 | `symbol_fanin_merges_answers_from_every_ready_host` — ⌘T query fans to two Ready hosts; both answer (different symbols) → `open_symbols.results` holds both hosts' rows. |
| REQ-005 | The 12 per-arm rows below (each exercises a NON-ACTIVE owning root). |
| REQ-006 | `consume_with_no_outcomes_reports_no_repaint` — hosts present, queues empty → false; zero hosts → false. |
| REQ-007 | `multi_host_dispatch_order_is_sorted_by_root` — outcomes on two hosts; assert dispatch order (observable effect order, e.g. two flashes/two applied edits in root-sorted sequence — exact assertion shape at validate). |
| Arm 1 | `nonactive_hover_answer_drops_without_card` — A's hover answer, B active → no `hover_card`, latch cleared, no flash. |
| Arm 2 | `nonactive_definition_answer_never_jumps` — A's definition answer, B active → active editor unchanged, no def_picker, no flash. |
| Arm 3 | `nonactive_references_answer_opens_no_picker` — A's refs answer, B active → `open_references` None; latch cleared. |
| Arm 4 | `nonactive_completion_answer_opens_no_menu` — A's completion answer, B active → `completion_menu` None. |
| Arm 5 | `nonactive_prepare_rename_stays_silent` — A's prepareRename answer (non-null), B active → no draft, no "Cannot rename this" flash. |
| Arm 7 | `nonactive_code_action_answer_opens_no_menu` — A's actions answer, B active → `code_action_menu` None, no flash. |
| Arm 8 | `nonactive_resolve_answer_applies_edit_under_owning_root` — resolve latched for A; B active; answer drains → A's file edited via the #322 engine, title flash set. |
| Arm 9 | `nonactive_signature_answer_shows_no_card` — A's signature answer, B active → `signature_card` None. |
| Arm 11 | `nonactive_inlay_answer_writes_no_hints` — A's inlay answer, B active → no inlay cache write (belt re-opens the resend gate). |
| Arm 12 | `nonactive_format_answer_formats_its_origin` — format latched for A's origin (parked ⌘S); B active; answer drains → origin buffer formatted + parked save settled, B's editor untouched, no caret re-seat. |

(Arms 6, 8, 10, 12 APPLY rows double as the REQ-002-class routing proofs;
arms 1–5, 7, 9, 11 DROP rows pin that no cross-root UI ever fires.)

### Risks / decisions

- **R1 — a future 13th arm.** The match is exhaustive: a new `RequestPurpose`
  variant forces a new arm at compile time, and the amended law comment now
  demands a non-active-root posture per arm. Residual risk accepted.
- **R2 — dispatch-order dependence between hosts.** Hidden cross-host order
  assumptions would be masked by HashMap luck today; sorted-root order makes
  it deterministic and REQ-007 pins it. (Within a host, #332 already
  guarantees ascending ids.)
- **R3 — background repaints.** APPLY arms now return true while another
  project is focused → a repaint of the focused view whose visible state may
  not change (flash aside). Cost: one frame; correctness unaffected.
- **R4 — the searching-card draw gate stays active-root** (references :10762) —
  deliberate: an occluding "Finding references…" card must describe the
  ACTIVE root's in-flight ask. Recorded as intended behavior, not a gap.
- **D-MECH (locked here):** internal iteration (signature drops `root`) over
  pump-side looping — keeps the ONE-drain law in one function, gives the
  ordering rule a single owner, and leaves the pump reading `active_root`
  only for send-side policy.

## Phase 3 — Implement

- **React-first: N/A** (spec: no UI delta — timing/routing of shipped surfaces only).
- **Built exactly to the design's mechanism (D-MECH):**
  - `crates/marley_app/src/app.rs` — `consume_lsp_responses` rewritten: signature
    drops `root: &Path`; collect-then-dispatch (`iter_mut` → `take_responses`
    per host → non-empty batches keep `(root.clone(), responses)`; sort by
    root; dispatch the SAME 12-arm match with the owning root via a
    `let root = root.as_path()` shadow, so every arm body is byte-identical).
    Doc comment rewritten to carry the #413 law (owning-root delivery,
    focus-guard partition, root-sorted determinism). Pump call site
    (`consume_lsp_responses()`, comment updated to the every-host posture).
  - `crates/marley_app/src/headless_drive.rs` — 70 harness call sites updated
    to the paramless signature (mechanical sed).
- **Deviation (mechanical, in-scope):** the signature change orphaned the
  harness's per-test `root` bindings that existed ONLY to feed the old param —
  50 unused-variable sites cleaned at the source (no `_root` underscoring):
  41 `let root = …clone();` lines deleted; 7 closure-return tuples
  `(key, root.clone())` simplified to return `key`; the #311 hover site's
  `(root, HoverKey)` tuple simplified to the key alone; the signature-card
  `open_card` closure dropped its unused `&Path` param (both callers + the
  feeding `let` removed). Design anticipated "test helper callers update";
  the orphan cleanup is its direct consequence.
- **Verified as-you-go:** `cargo check --workspace --all-targets` green;
  `cargo fmt --all --check` green; `cargo clippy -p marley --all-targets`
  green (only the pre-existing upstream `block v0.1.6` future-incompat note).
  No gate weakened; no tests written beyond compile-necessity (Phase 4 owns
  the new tests).

## Phase 3.5 — Inspect

Three parallel critics over the working-tree diff (correctness · data/state
integrity · simplification/reuse/provenance), each instructed to verify
concretely. The state critic was pointed at the nested-root twin hazard by the
lead's plan. Lead verified every accepted finding in source before fixing
(`resolve_under_root`/`uri_for` totality traced by hand; the shared remedy
checked per-consumer per PR-claude-critic-shared-fix-safety-claim-…-001).

### Findings ledger

| # | Sev | Critic | Finding | Verdict | Fix |
|---|---|---|---|---|---|
| F1 | HIGH | state | Nested-twin bypass on the definition arm: background F12 answer executes a jump INSIDE the focused project (latch+uri guards only, no version guard; `open_and_place_caret` opens under the ACTIVE root; foreign instance minted under B; nav_stack polluted) | **REAL, NEW** — pre-#413, dispatch root == active root == focused instance's root made the family unreachable; lead confirmed `uri_for` is total (absolute paths pass through `resolve_under_root`), so the design's "None cross-root" claim was false | D5: `outcome_is_for_focused_root` guard |
| F2 | HIGH | state | Nested-twin completion: menu opens over B's twin at a colliding version; accept applies A-computed ranges into B's diverged buffer (wrong-span edit, silent) | **REAL, NEW** (same mechanism; twin version counters independent — #354 F3 class) | D5 guard |
| F3 | MED | state | Twin inlay: A's answer projected onto B's buffer + latch churn (B's genuine pending then double-sends) | **REAL, NEW** | D5 guard |
| F4 | MED | state | Twin hover / signature / code-action / prepare-rename family: cross-instance cards + menus; worst case a user-confirmed prepare-rename→rename at the OTHER instance's position | **REAL, NEW** | D5 guard |
| F5 | MED | state | Twin references: picker over B from A's answer; identical-key (⇧F12 twice at same position) swallow variant | **REAL, NEW** — the latch-untouched drop shape preserves the colliding-key re-ask lifecycle (B's own answer still opens) | D5 guard |
| F6 | MED | state | `apply_one_file` routes a twin's edits to the lexicographically-FIRST root's instance while the version was validated against the OWNING root's host | **REAL, PRE-EXISTING-EQUAL** (the #354 D12.5 class, left open for rename/resolve) | D6: owning-root-first routing (falls back to the all-roots scan) |
| F7 | MED | simpl | fn doc + notes row 6 + lessons claimed "active root would version-miss and SKIP every file" — INVERTED: `version_conflict(_, None)` is NO conflict → REQ-009 gate fails OPEN → stale edits would APPLY with the wrong host's encoding | **REAL** (docs encode a false safety mechanism in three places) | doc + lessons rewritten with the fail-open truth |
| F8 | MED | simpl | "ids ascend within a host (#332)" overstates — answered outcomes keep wire-arrival order; only the abandonment sweeps sort ascending | **REAL** (comment + spec) | comment reworded; spec D3 + REQ-007 hedged to per-host queue order |
| F9 | LOW | corr + simpl + state (independent ×3) | "uri_for resolves under the owning root → None cross-root" mechanism false in doc + lesson (drop is by uri EQUALITY = file identity) | **REAL** | doc + lesson rewritten; the corrected mechanism is now the lesson's core |
| F10 | LOW | simpl | "the Zed-behavior posture" — first external product name in `crates/` source (convention keeps them in docs/) | **REAL** (convention) | reworded to "the spec §20 behavior-map posture" |
| F11 | LOW | simpl | Stale `consume_hover_responses` mentions ×3 (fn died into the Hover arm at #312) | **REAL, pre-existing** (two sat on this seam) | all three reworded |
| F12 | LOW | state | ⌘T nested-twin duplicate rows now routine (both hosts index the shared subtree) | **REAL, WIDENED — DEFERRED**: cosmetic; bounded by the ×8 cap; per-row jumps correct; dedupe noted as future polish | none (recorded) |
| F13 | LOW | state | References answers for a backgrounded root now drop instead of delivering at switch-back | **INTENDED** — the design's DROP semantics; switch-back delivery was an artifact of the deferral bug. Recorded per the critic's ask | arm table annotation (this row) |
| F14 | LOW | state | `FormattingKey{uri,version}` twin key-collision can cross-wire a parked ⌘S | **REAL, PRE-EXISTING, NARROWED by #413** (the correct answer can now win the race) — DEFERRED: key root/ContentId hardening is #354-domain follow-up work | none (recorded) |
| F15 | LOW | simpl | `lsp_hosts: BTreeMap` would delete the sort + REQ-007 obligation | **REJECTED** — collect-then-`sort_by` is the pump's own #396 idiom (open_docs, same file); host count tiny; HashMap get-paths untouched | none |

Correctness critic's independent pass came back clean on the mechanism:
REQ-001/002/006/007 verified in source, all 70 harness pre-image call sites
confirmed active-root (sed semantics-safe), zero stale live-code assumptions,
1000 lib tests green pre-fix. Its "twin = timing-only" side-remark is
SUPERSEDED by the state critic's trace (pre-#413 the focused instance was
always the requesting instance — the family is NEW; verdict recorded in F1).

### Fixes applied (all verified compiling + tested)

- `outcome_is_for_focused_root` helper + the guard as the FIRST statement of
  the 8 focused-editor arms (drop before latch/UI, latch untouched — the
  colliding-twin re-ask may own it; every send path overwrites its latch).
- `apply_one_file`: takes `root`, prefers the owning root's instance (mirrors
  the #354 latchless scan shape), falls back to `editor_id_for_open_path`.
- Doc corrections: consume fn doc (mechanism + order + product name),
  `apply_one_file` doc, 3× stale fn mentions, lessons block rewritten,
  spec D3/D5/D6/REQ-007/REQ-008/REQ-009.
- Post-fix: `cargo check --workspace --all-targets` green, `cargo fmt` clean,
  `cargo clippy -p marley --all-targets` clean, `cargo nextest run -p marley`
  **1016/1016 passed** (2 skipped, pre-existing).

### Validate-plan additions from inspect

- `nested_twin_definition_answer_never_jumps` (F1: A-host answer for the
  focused twin under nested roots → no jump, no picker, no nav push).
- `nested_twin_completion_answer_opens_no_menu` (F2).
- `rename_twin_routes_to_owning_instance` (F6 / REQ-009: file open under
  parent AND owning nested root → owning instance's buffer changes, twin's
  does not).
- The 8 planned per-arm cross-root DROP tests now double as the
  `outcome_is_for_focused_root` mutation kills (== flip → cross-root applies →
  the drop assertions fail; plus every same-root test pins the pass side).

### Knowledge captures

- `F-claude-413-nested-twin-cross-instance-outcome-delivery-001` (failures.md)
- `PR-claude-uri-equality-is-not-instance-identity-001` (prevention-rules.md)
- `L-claude-413-…-001` amended in place (the false mechanism was the tail
  block written by THIS pipeline's design phase — corrected before it could
  mislead, amendment noted in its meta line).

## Phase 4 — Validate

### Tests written (19 — the design plan + the 3 inspect additions), all in `headless_drive.rs`

Fixtures: `boot_two_roots_413` (two DISJOINT roots, lexicographically ordered
`a_root` < `b_root`, one file open under each) and `boot_nested_twin_413`
(parent ⊃ child with one file under BOTH — the D-OPEN-DEDUPE-SCOPE twin
fixture). Two new `#[cfg(test)]` hooks in app.rs: `push_answered_for_test`
(the Answered sibling of `drive_abandoned_for_test`) and
`buffer_text_for_test(root, path)` (which TWIN took an edit).

| REQ / Arm | Test |
|---|---|
| REQ-001 | `all_hosts_outcomes_drain_in_one_tick_headless` (rename on background A + resolve on active B land in ONE pass; second consume false) |
| REQ-002 / Arm 6 | `nonactive_rename_answer_applies_under_owning_root_headless` |
| REQ-003 | `background_abandonment_flashes_within_one_tick_headless` (flash while the OTHER project is focused) |
| REQ-004 / Arm 10 | `symbol_fanin_merges_answers_from_every_ready_host_headless` (a row set queued on EACH root's host) |
| REQ-006 | `consume_with_no_outcomes_reports_no_repaint_headless` (zero hosts AND empty-queue host) |
| REQ-007 | `multi_host_dispatch_order_is_sorted_by_root_headless` (B queued FIRST, dispatched LAST — sort, not insertion; repaint bool pinned) |
| Arm 1 | `nonactive_hover_answer_drops_without_card_headless` (+ latch-untouched D5 assert) |
| Arm 2 | `nonactive_definition_answer_never_jumps_headless` (focus kept, no picker, nav 0, latch kept) |
| Arm 3 | `nonactive_references_answer_opens_no_picker_headless` (latch kept) |
| Arm 4 | `nonactive_completion_answer_opens_no_menu_headless` |
| Arm 5 | `nonactive_prepare_rename_stays_silent_headless` (no draft, no decline flash, latch kept) |
| Arm 7 | `nonactive_code_action_answer_opens_no_menu_headless` (latch kept) |
| Arm 8 | `nonactive_resolve_answer_applies_edit_under_owning_root_headless` |
| Arm 9 | `nonactive_signature_answer_shows_no_card_headless` |
| Arm 11 | `nonactive_inlay_answer_writes_no_hints_headless` (latch kept — the #331 belt re-asks) |
| Arm 12 | `nonactive_format_answer_formats_its_origin_headless` (origin buffer + disk settle; focused project untouched) |
| REQ-008 / F1 | `nested_twin_definition_answer_never_jumps_headless` (twin uri IDENTICAL — still dropped) |
| REQ-008 / F2 | `nested_twin_completion_answer_opens_no_menu_headless` (uri AND colliding version both match — still dropped) |
| REQ-009 / F6 | `rename_twin_routes_to_owning_instance_headless` (child instance edited; lexicographically-first parent twin untouched) |

The 8 latch-kept asserts + the 2 twin tests are the mutation kills for
`outcome_is_for_focused_root` (without the guard, the arms CLEAR their latches
on the uri-drop path, or the twin tests open UI — either fails the assert).

### Runs (actual)

- New suite: `cargo nextest run -p marley -E 'test(/…413 names…/)'` →
  **19 tests run: 19 passed** (1018 skipped).
- Full workspace: `cargo nextest run --workspace` →
  **2123 tests run: 2123 passed, 5 skipped** (skips pre-existing).
- Doctests: `cargo test --workspace --doc` → all `test result: ok` (0 failed).

### Live-app drive (step 3)

N/A — no UI-affecting change: the diff alters WHEN existing surfaces fire
(response routing in the pump's consume pass), not how any pane looks or
responds to input; the spec's React-first section is N/A, there is no
`visual_acceptance` clause, and the behavior deltas (background flashes,
cross-root drops, twin routing) are exactly what the 19 headless drives
assert. No render/keystroke/mouse path was touched.

### Pre-existing (documented, not in scope)

- `block v0.1.6` future-incompat warning (upstream transitive; predates #413).
- The 5 skipped workspace tests are the suite's standing skips, untouched.

### Gate

`scripts/gates.sh --diff` — first run RED on gate:1 alone (the appended test
block predated the final `cargo fmt`); formatted at source, re-ran:

```
══ gate summary (diff) ══
  PASS  gate:1  rustfmt
  PASS  gate:2  clippy (-D warnings)
  PASS  gate:3  tests (nextest + doctests)
  PASS  gate:7  cargo-audit
  PASS  gate:8  cargo-deny
  PASS  gate:9  cargo-machete
  PASS  gate:10 gitleaks (secrets)
  PASS  gate:11 shellcheck
  PASS  gate:12 no-suppressions
  PASS  gate:13 source-bans (SAST)
  PASS  gate:14 docs (rustdoc -D warnings + doc-todos + brand-scrub)
  PASS  gate:4  rust coverage (>= 100% lines)
  PASS  gate:5  mutation (MSI >= 100%)
  PASS  gate:6  miri (unsafe crates)
  PASS  gate:15 visual / AX
  15 passed, 0 failed
GATE GREEN [diff]
```

Receipt written (`.git/ignibyte-gate-receipt` present).

## Phase 5 — Complete

- **CHANGELOG (§21a):** `[Unreleased] → Fixed` entry added — the deferral bug,
  the owner-routing correctness argument (fail-open version gate), the twin
  find, both inspect fixes, and the 19-drive test surface.
- **Architecture docs (§21b):** `docs/marley_architecture/editor.md` — new
  "#413 closes #332's deferral half" section after the #332 outcome paragraph:
  owner-routed delivery, the sorted collect-then-dispatch shape, the
  8-DROP/4-APPLY partition, the uri≠instance twin mechanism, latch-untouched
  drops, and the owning-instance WorkspaceEdit routing.
- **Parity (§21c):** N/A — no UI delta (spec React-first: N/A; no marley-web
  change to sync).
- **Knowledge (§19) — codes:**
  - `F-claude-413-nested-twin-cross-instance-outcome-delivery-001` (failures —
    appended at inspect)
  - `PR-claude-uri-equality-is-not-instance-identity-001` (prevention rules —
    appended at inspect)
  - `L-claude-413-focus-guards-partition-a-multi-root-drain-for-free-001`
    (lessons — written at design, AMENDED at inspect with the corrected
    mechanism; the amendment is the lesson)
  - `AD-claude-413-lsp-outcome-delivery-is-owner-routed-never-focus-gated-001`
    (architecture decisions — appended here)
- **Ticket:** TICKET-413 → `tickets/closed/`, status closed (2026-08-11).
  BACKLOG swept — no stale row (left the queue at promotion).
- **Deferred (recorded at inspect, no code):** F12 `FormattingKey` twin
  key-collision hardening (pre-existing, NARROWED by #413) and ⌘T nested-twin
  duplicate-row dedupe (cosmetic, cap-bounded) — candidates for a future
  key-hardening/polish ticket.
- **Archive:** this pair → `docs/planning/pipeline/completed/`.
