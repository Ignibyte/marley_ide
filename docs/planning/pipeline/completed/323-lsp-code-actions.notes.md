# LSP code actions + quick fix (⌘.) — the picker + codeAction/resolve — Notes

- **Forge ticket:** #323 056e3f20-fa74-4d16-9d3e-6631daf27dda
- **AAR:** cbce96e2-8719-472c-8073-b5c01645fe0f
- **Local ticket doc:** docs/planning/tickets/open/TICKET-323-lsp-code-actions.md
- **Pipeline spec:** 323-lsp-code-actions.spec.md

## Phase 1 — Plan

### Request
⌘. → `textDocument/codeAction` → a picker of the server's fixes/assists → accept (resolve if needed) →
apply via the #322 engine. Where auto-import (the #313 cut) lands. The 2nd M21 ticket.

### Classification / tier
Work pipeline, feature, M21. ONE slice: request → parse → order → picker → accept → (resolve) → apply.
The apply is #322's engine, DONE — so the genuine new work is the codeAction request/parse/resolve +
the picker.

### Forge recall (§18.3)
- `bulletin-list` → none.
- The rules that land here (all already in the spec's decisions):
  - `PR-claude-second-consumer-must-inherit-the-first-consumers-guards` → D6 (key uri+version+caret +
    the live-identity dismiss). #323 is the FIFTH consumer of the #311 request path.
  - `PR-claude-new-overlay-register-at-every-choke-point` → D5 (the picker at every choke).
  - `PR-claude-synthetic-response-tests-never-prove-the-live-wire` → P4 asserts the real codeAction
    frame's `context.diagnostics` payload.
  - the #322 canonicalization rules (`PR-claude-compare-path-identity-canonically-not-raw-eq`,
    `PR-claude-canonicalize-before-containment-check`) — inherited FREE via `apply_workspace_edit`.
  - `mutants::skip` detach trap + trace-the-real-list + the "every new lsp_host accessor needs the
    sibling skip" lesson (cost a RED gate on #322).

### Discovery — the precise surface
**REUSE AS-IS (the apply is done):**
- `marley_lsp::parse_workspace_edit` (workspace_edit.rs:82) + `RootView::apply_workspace_edit` (app.rs:6321,
  private fn, same impl → #323 calls it directly) — the #322 engine. `apply_code_action_edit` =
  `parse_workspace_edit(edit)` → `apply_workspace_edit(root, files)`, with the `ResourceOpsUnsupported`
  flash (the #322 shape).
- `LspHost::diagnostics_for(path) -> &[Diag]` (lsp_host.rs:443) + `marley_lsp::diagnostic_at_row(diags,
  row)` (diagnostics.rs:206) + `Diag`/`DiagSpan` (diagnostics.rs:64/52) — for `context.diagnostics`.
- The #311 request recipe: `RequestPurpose` (lsp_host.rs:34), `request()` (:459), the ONE drain
  `consume_lsp_responses` (app.rs:6116). Raw caps on `Negotiated.server_caps`.
- `DefPicker` (editor_nav.rs:70 — `new`/`move_up`/`move_down`/`chosen`) — the picker shape to mirror for
  `CodeActionMenu`.
- `lsp_position_for` (app.rs) for the caret's (uri, encoded position) → the range's start==end (D7).

**NET-NEW (ordered by risk):**
1. **`parse_code_actions`** — the Command-vs-CodeAction discrimination (the risk): a `Command` has a
   top-level string `command` and NO `edit`/`kind`/`diagnostics`; a `CodeAction` has `title` + any of
   `kind`/`edit`/`diagnostics`/`data`. Keep CodeActions; skip Command-only + `disabled` + malformed;
   count the Command-only skips. Per-element `filter_map` (the #322 posture).
2. **The ordering** — quickfix-kind first, then refactor/source, `isPreferred` pinned. A pure sort with
   a kind-rank + a preferred-flag key.
3. **`needs_resolve`** — `edit.is_none() && resolveProvider` (D4). The resolve sends the raw action JSON
   back → `apply_code_action_resolve_response` applies the filled `edit`.
4. **`code_action_params(uri, start, end, diagnostics)`** — codeAction takes a RANGE + a context, which
   `text_document_position_params` does NOT cover (it's position-only). New builder.
5. **`code_action_support(server_caps)`** — None/Basic/WithResolve off `codeActionProvider`.
6. `RequestPurpose::{CodeAction(CodeActionKey), CodeActionResolve(CodeActionKey)}` + the 2 drain arms.
7. **`CodeActionKey`** (uri, version, caret) + the `CodeActionMenu` picker (borrows DefPicker) + the
   overlay render + the ⌘. keymap.

### Decisions
D1 apply via #322 engine unchanged · D2 Command-only OUT (skip+count) · D3 hand-parse · D4
needs_resolve=edit.is_none()&&resolveProvider · D5 picker mirrors DefPicker + register at chokes · D6
inherit the request guards whole · D7 caret single-point range v1. Full text in the spec.

### Risks
- **R1 — the Command-vs-CodeAction discrimination.** Getting it wrong either shows unusable Command rows
  or drops real CodeActions. Pin the truth table (a pure `command`-only vs a `{title,edit}` vs a
  `{title,kind,data}` deferred-edit vs `disabled`).
- **R2 — the resolve round-trip.** The accept fires a SECOND request only when the edit is deferred; a
  mistake either double-applies or applies nothing. The `needs_resolve` unit + the headless resolve
  chain pin it.
- **R3 — stale picker at per-caret rates.** The live-identity dismiss (D6) + the key drop.
- **R4 (LOW) — the apply is #322's**, already proven; the only #323 risk on apply is passing the wrong
  `edit` JSON (the resolved one vs the original). The resolve arm must apply the RESOLVED action's edit.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture / approach
Two pure layers + a shim, the #311/#312/#313/#322 shape — but the APPLY is done (#322), so #323 is
"request → parse → order → picker → accept → (resolve) → hand the edit to #322". The genuine new code is
the codeAction request shape, the Command-vs-CodeAction parse, the resolve round-trip, and the picker.

**Layer 1 — PURE `marley_lsp/src/code_action.rs` (NEW):**
- `CodeAction { title, kind: Option<String>, is_preferred: bool, disabled: bool, edit: Option<Value>,
  raw: Value }` — Marley-local, but keeps the `raw` JSON: `codeAction/resolve` sends the whole action
  back for the server to fill `edit` from its `data` (D4).
- `parse_code_actions(&Value) -> CodeActionSet { actions: Vec<CodeAction>, unavailable: usize }` — the
  array → per-element `filter_map`: an element that is a **bare `Command`** (a top-level string `command`
  and NO `edit`/`kind`/`diagnostics`/`data`) is SKIPPED + counted into `unavailable` (D2); a `disabled`
  action is skipped; a malformed element is skipped; a `CodeAction` is normalized + kept. The kept set
  is sorted **quickfix-first, then refactor/source, `isPreferred` pinned** via a stable sort on
  `(is_preferred DESC, kind_rank ASC, input order)`.
- `code_action_kind_rank(kind: Option<&str>) -> u8` — `quickfix*`→0, `refactor*`→1, `source*`→2, other→3
  (LSP `CodeActionKind` is dotted-hierarchical: match the prefix).
- `needs_resolve(action: &CodeAction, resolve_provider: bool) -> bool` = `action.edit.is_none() &&
  resolve_provider` (D4).
- `resolved_edit(resolved: &Value) -> Option<Value>` — pull `edit` out of a resolved CodeAction (for the
  resolve response).
- `code_action_params(uri, start: Position, end: Position, diagnostics: Vec<Value>) -> Value` — the
  request: `{textDocument{uri}, range{start,end}, context{diagnostics, triggerKind:1}}` (the base
  position-params builder does NOT cover a range + context).
- `diagnostic_to_lsp(diag: &Diag) -> Value` — rebuild `{range, message, severity}` from a stored `Diag`
  for `context.diagnostics`. **R1: the store's `Diag` is lossy** (no `code`/`data`) — rust-analyzer
  matches a quick-fix to a diagnostic mostly by RANGE (it re-derives at the position), so the rebuild is
  usually enough; the live drive settles it, and the fallback (a #310 raw-diagnostic store) is a named
  follow-up, not a v1 blocker.
- `code_action_support(server_caps) -> CodeActionSupport { None, Basic, WithResolve }` — off
  `codeActionProvider` (`false`/absent→None; `true` or an object w/o `resolveProvider`→Basic; an object
  with `resolveProvider:true`→WithResolve).

**Layer 2 — PURE `marley_app/src/editor_code_action.rs` (NEW):**
- `CodeActionKey { uri, line, character, version }` — the stale guard (carries the version, like
  `CompletionKey`: an accepted/resolved edit for a buffer that has since changed must not apply).
- `CodeActionMenu { actions: Vec<CodeAction>, selected }` — `move_up`/`move_down` (wrapping, the
  `DefPicker`/`complete.rs` idiom), `chosen()`, `visible(max)` (reuse `complete::popup_window`).

**Layer 3 — SHIM `marley_app/src/app.rs`:** fields `code_action_menu: Option<OpenCodeActions>` (the
menu + its `CodeActionKey` + `unavailable`), `code_action_request: Option<CodeActionKey>`. Fns (mirror
the #313/#322 request path):
- `request_code_actions()` (⌘.) — Ready+provider gated; build the caret single-point range (D7) via
  `lsp_position_for`, gather `context.diagnostics` from `diagnostics_for(path)` filtered to the caret row
  (`diagnostic_at_row` region) → `diagnostic_to_lsp`, send; empty/no-provider → a quiet flash.
- `apply_code_action_response(key, result, root)` — the #312 supersede + live-identity guards, then
  `parse_code_actions`; empty → flash "No actions here"; else open the picker (with the `unavailable`
  count).
- the picker key handling (↑/↓/Enter/Esc — mirror `handle_def_picker_key`); Enter → `accept_code_action`.
- `accept_code_action(root)` — `needs_resolve`? send `codeAction/resolve(raw)` keyed by the same
  `CodeActionKey`, close the picker : apply the edit directly.
- `apply_code_action_resolve_response(key, result, root)` — the supersede + version guard, then
  `resolved_edit` → `apply_code_action_edit`.
- `apply_code_action_edit(edit: &Value, root)` — **`parse_workspace_edit(edit)` → `apply_workspace_edit
  (root, files)` (REUSE #322 unchanged)** + the `ResourceOpsUnsupported`/summary flash (the #322 shape).
- `RequestPurpose::{CodeAction(CodeActionKey), CodeActionResolve(CodeActionKey)}` (lsp_host) + the 2 arms
  in the ONE drain (app.rs:6116).
- ⌘. keymap row (KeyContext::Editor); the picker overlay (mirror `def_picker_overlay`); the
  `text_input_blocked` line; a `code_action_support()` accessor on `LspHost` **with the sibling
  `#[cfg_attr(test, mutants::skip)]`** (the #322 RED lesson — every new lsp_host accessor gets the skip).

### §20 confirmation
**Zed / VS Code (the editor).** ⌘. opens a menu of the server's fixes/assists (preferred-first,
auto-import), and accepting applies the fix — matched via the published LSP 3.17 wire
(`textDocument/codeAction`, `codeAction/resolve`, `CodeAction`/`Command`/`CodeActionKind`/`isPreferred`)
plus Marley's OWN picker, the #310 store, and the #322 engine. The menu + ordering are the spec's own
(`isPreferred`, the dotted `kind` hierarchy). No GPL source read — clean-room confirmed.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/code_action.rs` | NEW pure: `CodeAction`/`CodeActionSet`/`CodeActionSupport`; `parse_code_actions`, `code_action_kind_rank`, `needs_resolve`, `resolved_edit`, `code_action_params`, `diagnostic_to_lsp`, `code_action_support`. |
| `crates/marley_lsp/src/lib.rs` | `mod code_action;` + re-exports. |
| `crates/marley_app/src/editor_code_action.rs` | NEW pure: `CodeActionKey`, `CodeActionMenu`. |
| `crates/marley_app/src/lib.rs` | `mod editor_code_action;`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::{CodeAction, CodeActionResolve}`; `code_action_support()` accessor (skip'd). |
| `crates/marley_app/src/app.rs` | 2 fields; `request_code_actions`/`apply_code_action_response`/the picker key+render/`accept_code_action`/`apply_code_action_resolve_response`/`apply_code_action_edit`; the 2 drain arms; `text_input_blocked` line. |
| `crates/marley_app/src/keymap.rs` | ⌘. → `code-action` (KeyContext::Editor) + the roster guard bump. |
| `crates/marley_app/src/headless_drive.rs` | the drives below. |

### Regression Test Plan (≥1 per REQ)
| Test (loc) | REQ | Asserts |
|---|---|---|
| `code_action::tests::parse_discriminates_and_orders` | 002/003 | a `{title,edit}` CodeAction kept; a `{title,kind,data}` deferred-edit kept; a bare `{title,command}` Command SKIPPED + counted; a `disabled` skipped; malformed skipped; quickfix ordered before refactor/source; `isPreferred` pinned top. Off-grid positions. |
| `code_action::tests::needs_resolve_table` | 004 | `edit present` → false; `edit absent + resolveProvider` → true; `edit absent + no provider` → false. |
| `code_action::tests::resolved_edit_extracts` | 004 | a resolved action's `edit` pulled; absent → None. |
| `code_action::tests::code_action_params_shape` | 001 | the `{textDocument, range{start,end}, context{diagnostics, triggerKind:1}}` shape; a rebuilt diagnostic carries range+message+severity. |
| `code_action::tests::code_action_support_table` | 001/004 | false/absent→None; true/obj→Basic; `resolveProvider:true`→WithResolve. |
| `code_action::tests::kind_rank` | 003 | quickfix<refactor<source<other; dotted prefixes (`quickfix.import`) rank by prefix. |
| `editor_code_action::tests::menu_move_and_chosen` | 002 | wrap up/down; chosen follows; degenerate sizes. |
| `editor_code_action::tests::code_action_key_version` | 008 | equality incl. the version field. |
| `headless::code_action_opens_picker_and_applies_direct` | 004/005 | ⌘. → response with an edit-bearing action → picker → accept → the buffer edit applied via the #322 engine. |
| `headless::code_action_resolve_then_applies` | 004/005 | an edit-LESS action + resolveProvider → accept sends resolve → the resolved edit applied. |
| `headless::code_action_command_only_skipped` | 002 | a Command-only response → the picker shows the count / a flash when ALL are Commands. |
| `headless::code_action_no_provider_flashes` | 006 | ⌘. with no host / no provider → a flash, no picker. |
| `headless::code_action_stale_dropped` | 007/008 | a response / resolve answer whose key no longer matches → dropped. |
| **A LIVE WIRE CHECK (P4)** | 001/005 | tee + seeded bundle: delete a `use` line → the squiggle → ⌘. → "import" → accept → the use line BACK on disk (applied via #322); assert the real codeAction frame's `range` + `context.diagnostics` (payload). If the rebuilt diagnostic doesn't yield the import (R1), the fallback is a #310 raw-diagnostic store — decided at validate from the live result. |

**Uncoverable-by-unit:** the picker's pixel placement (no test pixels) → `def_picker`/`popup_window`
reuse + the live capture.

### Risks / decisions
- **R1 (top) — `context.diagnostics` fidelity.** The #310 store's `Diag` is lossy (no `code`/`data`);
  rebuilt `{range,message,severity}` may not match a server's quick-fix association. Mitigation:
  rust-analyzer is range-driven; the live drive is the arbiter; the fallback (store raw diagnostic JSON)
  is a named #310 follow-up, kept OUT of #323 unless the live drive forces it.
- **R2 — the Command-vs-CodeAction discrimination** (parse). The truth table is the mutation hotspot —
  pin the four element shapes exactly.
- **R3 — the resolve edit identity.** The resolve arm must apply the RESOLVED action's `edit`, not the
  original (which was absent). The `resolved_edit` seam + the headless resolve chain pin it.
- **R4 — apply safety is #322's, FREE.** `apply_workspace_edit`'s version-conflict + the canonical
  routing protect the code-action apply exactly as they protect rename; #323 adds no apply risk beyond
  passing the right `edit` JSON.
- **D-des-1 — the `code_action_support` accessor gets the sibling `mutants::skip`** (the #322 RED
  lesson — a new lsp_host accessor whose logic is a tested pure seam).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`marley_lsp/src/code_action.rs` (NEW, pure)** — `CodeAction`/`CodeActionSet`/`CodeActionSupport`;
  `parse_code_actions` (the Command-vs-CodeAction discrimination: usable ⇔ it can produce an edit — an
  eager `edit` OR a `data` blob to resolve; a bare `command`-only → skip+count `unavailable`; `disabled`
  → skip; malformed → skip; then sort `isPreferred`-first, then kind-rank, stable); `code_action_kind_
  rank` (dotted-prefix quickfix<refactor<source<other); `needs_resolve` (edit-absent && resolveProvider);
  `resolved_edit`; `code_action_params` (range + context.diagnostics + triggerKind:1); `diagnostic_to_
  lsp` (rebuild {range,message,severity} from a stored `Diag`, `severity_number` the inverse map);
  `code_action_support`.
- **`marley_app/src/editor_code_action.rs` (NEW, pure)** — `CodeActionKey` (uri+line+char+**version**);
  `CodeActionMenu` (wrapping move_up/down, chosen, `visible` reusing `complete::popup_window`).
- **`marley_lsp/src/lib.rs` / `marley_app/src/lib.rs`** — module + re-exports.
- **`marley_app/src/lsp_host.rs`** — `RequestPurpose::{CodeAction, CodeActionResolve}(CodeActionKey)` +
  `code_action_support()` accessor (with the sibling `mutants::skip` — the #322 RED lesson, applied at
  IMPLEMENT this time, not after a red gate).
- **`marley_app/src/app.rs`** — `OpenCodeActions{menu, key, unavailable}` + 2 fields; `request_code_
  actions` (⌘.: caret range + context.diagnostics overlapping the caret row via `diagnostics_for` +
  `diagnostic_to_lsp`); `apply_code_action_response` (supersede + live-identity guards → parse → picker);
  `handle_code_action_key`; `accept_code_action` (`needs_resolve`? send `codeAction/resolve(raw)` :
  apply); `apply_code_action_resolve_response`; **`apply_code_action_edit` = `parse_workspace_edit` →
  `apply_workspace_edit` (the #322 engine, UNCHANGED)**; the 2 drain arms; `code_action_overlay` (the
  def_picker card recipe); the `text_input_blocked` + key-router branches; the ⌘. dispatch arm.
- **`marley_app/src/keymap.rs`** — ⌘. → `code-action` (Editor-scoped) + the roster guard (55→56, 14→15).

### Deviations from design (with reason)
- **`diagnostic_to_lsp` maps `Severity`→number directly** (Error=1…Hint=4) with a private `severity_
  number` — the store's inbound `severity_from_lsp` is private and one-way; a local inverse is cleaner
  than exposing it.
- **The apply flash phrasing is inline in `apply_code_action_edit`** (`"{title}: N files changed"`) not a
  new pure fn — the fn is a `mutants::skip` shim (it calls `apply_workspace_edit`), so the branch isn't a
  mutation target; a separate pure summary would be over-engineering (unlike `rename_summary`, which is
  reused by two sites).
- **`accept_code_action` clones the chosen action** (`chosen().cloned()`) before `take`-ing the menu —
  `CodeAction: Clone`, and the borrow can't span the `take`.

### Verified
`cargo check --workspace` clean; `cargo clippy -p marley -p marley_lsp --all-targets` **0 warnings**;
`cargo fmt --all --check` clean. **The `mutants::skip` detach trap did NOT fire** — app.rs still **64**
(`handle_code_action_key` needed its skip — caught HERE at implement, +5→back to 64 — the #322 lesson
applied; no code-action shim fn leaked, no sibling detached). Real mutant sets traced: **code_action.rs
= 31**, **editor_code_action.rs = 19**. The `parse_code_actions` discrimination + the kind-rank +
`needs_resolve` are the hotspots — Phase 4 pins the four element shapes + the resolve table.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Two parallel critics (correctness/parse-resolve, reuse/guards/provenance) + my own review. The reuse
critic found the crux CLEAN (the #322 apply reuse = zero duplication, guard-inheritance split correct,
provenance clean-room) and independently confirmed my two self-found fixes. The correctness critic found
one real MED (a guard that reads correct but was inert at runtime) + two LOWs.

### REAL — fixed in #323

| # | Sev | Finding | Fix |
|---|---|---|---|
| **F-CORR-1** | **MED** | **The `version_conflict` staleness guard is INERT — it protects nothing at runtime.** Marley's `initialize` advertised no `workspace.workspaceEdit.documentChanges`, so rust-analyzer returns a WorkspaceEdit as the legacy `changes` map (NO version); `version_conflict(None, synced) == false` → a rename/code-action edit applies UNCONDITIONALLY. The accept→resolve window (user types after Enter, before `codeAction/resolve` returns) was unguarded, and — worse — this silently defeated the SHIPPED #322 rename's version guard too. | Advertise `capabilities.workspace.workspaceEdit.documentChanges: true` (honest — `parse_workspace_edit` handles + prefers it; `resourceOperations` deliberately NOT advertised) so the server sends VERSIONED `TextDocumentEdit`s, making the #322 version-conflict guard live for BOTH rename and code actions. Handshake test updated. New rule `PR-claude-lsp-version-guard-inert-without-documentchanges-cap`. |
| **F-CORR-2** | LOW | A fresh ⌘. clobbered an in-flight `codeAction/resolve` — both shared the ONE `code_action_request` slot, so `request_code_actions` overwrote the resolve's key → the resolve answer was dropped SILENTLY → the accepted action never applied, no feedback. | A SEPARATE `code_action_resolve_request` slot; `request_code_actions` touches only the codeAction slot, `accept`/`apply_resolve` use the resolve slot. |
| **F-CORR-3** | LOW | `parse_code_actions` kept a `{title, data: null}` action as "resolvable" (`data` key present) — but a null `data` carries nothing to resolve → accept would flash "no edit". | Gate on a NON-NULL data: `obj.get("data").is_some_and(\|d\| !d.is_null())`. |
| **SELF-1** | MED | The code-action picker did NOT clear at the agent-launcher-open choke point — its key arm runs before the launcher's and would swallow every key (the #312-F16 class), leaving the launcher keyboard-dead. | `self.code_action_menu = None` at the launcher-open site, next to `self.def_picker = None`. |
| **SELF-2** | LOW | `code_action_kind_rank` was `pub` + re-exported but used only inside `parse_code_actions` (the #311-F3 / #322-apply_resolved rule) — its Phase-4 test reaches it privately. | Made private; dropped from the `lib.rs` re-export. (Both critics also flagged this — 3-way agreement.) |

### REAL — deferred with reason
- **F-CORR-3 part 2 (LOW) — a `data`-only action on a BASIC (no-resolve) server** is shown but can't
  apply (accept flashes "no edit"). rust-analyzer advertises `resolveProvider: true`, so this needs a
  non-conformant server; the accept-time flash is honest. Threading resolve-support into the parse to
  count it `unavailable` is a follow-up, not v1.

### VERIFIED CLEAN (independently, by the critics + me)
- **The #322 apply reuse (the crux)** — `apply_code_action_edit` = `parse_workspace_edit` →
  `apply_workspace_edit`, the SAME engine rename uses (canonical routing + version-conflict +
  containment + last-to-first, all inherited free), with the same `ResourceOpsUnsupported` reject. Zero
  duplication.
- **Guard inheritance** — `apply_code_action_response` KEEPS supersede + focused-file (the picker
  anchors at the caret, mirroring `apply_prepare_rename_response`); `apply_code_action_resolve_response`
  OMITS focused-file with the SAME stated reason as `apply_rename_response` (an edit names files by uri).
- **The resolve routing** — accept sends `action.raw`; the resolve arm applies `resolved_edit(response)`,
  never the original absent edit; the purpose variants + the key disambiguate.
- **The parse discrimination** — eager-edit kept, deferred-data kept, bare-command counted, command+edit
  kept, disabled/malformed skipped; the caret-row diagnostic filter is inclusive both ends; no off-by-one.
- **Panic-safety** — no `unwrap`/`expect` on a response path; the menu guards `is_empty`/`.get`; the
  overlay guards each row.
- **mutants::skip consistency** — all 8 shims carry it (7 app.rs + the lsp_host accessor); no pure fn
  wrongly skipped. **Provenance** — hand-parsed from the published LSP 3.17 spec, no source read, no
  secret, no NEW write surface (the apply flows through #322's contained engine).

### Verified after the fixes
`cargo check --workspace` clean; `cargo clippy -p marley -p marley_lsp --all-targets` **0 warnings**;
`cargo fmt --all --check` clean; the handshake test (documentChanges) green. app.rs still **64** (the new
resolve slot + reroute live in `mutants::skip` shims; no detach). `code_action.rs` gained the data-null
gate (Phase 4 tests it).

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests written (all green)
- **`marley_lsp/src/code_action.rs` — 9 pure units** (cov/MSI 100): `parse_discriminates_and_orders`
  (the four element shapes + isPreferred-pin + kind order), `parse_keeps_a_codeaction_with_a_command_
  and_edit`, `parse_degrades_on_junk` (Null/{}/array-of-junk → empty; two-commands → `unavailable==2`
  kills `+=`→`*=`), `kind_rank_orders` (exact + dotted-prefix + near-miss), `needs_resolve_table`,
  `resolved_edit_extracts`, `code_action_params_shape`, `diagnostic_to_lsp_rebuilds` (**extended in
  validate to ALL FOUR severities** — Info→3/Hint→4 were 2 uncovered `severity_number` arms = the gate:4
  red + 2 diff-mutants; the four-severity sweep closes both), `code_action_support_table`.
- **`marley_app/src/editor_code_action.rs` — 3 pure units**: `menu_move_and_chosen` (wrap + empty-safe),
  `menu_visible_windows` (reuses `complete::popup_window`), `code_action_key_version` (any field incl.
  version differs ⇒ different question).
- **`marley_app/src/headless_drive.rs` — 6 driven RootView tests** (the real app, real keystrokes, no
  pixels): `lsp_code_action_opens_and_applies_direct` (REQ-002 count + REQ-004 direct + REQ-005 #322
  apply), `…command_only_flashes` (REQ-002/006 → "No applicable actions here", no picker), `…no_host_
  flashes` (REQ-006 not-ready → "LSP: not ready"), `…stale_response_dropped` (REQ-008), `…resolve_
  applies_and_stale_dropped` (REQ-004 resolve-apply + REQ-007 supersede-drop; the first half is the
  positive control the drop half needs), `…picker_owns_the_keyboard` (REQ-007 by construction — a caret-
  move key routed through the REAL handler is eaten by the picker, so the caret can't move out from
  under it; this is why there is no caret-move pump-dismiss).
- Two new test hooks (`drive_code_action_for_test`, `drive_code_action_resolve_for_test`) + accessors,
  all `#[cfg(test)]`. `cargo nextest -p marley lsp_code_action` = 6/6; `-p marley_lsp code_action` = 9/9;
  `editor_code_action` = 3/3; whole workspace = 1317/1317.

### LIVE DRIVE (real rust-analyzer) — and the bug it caught
Drove the bundled release `Marley.app` (screen unlocked, AX granted) on a real standalone cargo fixture
(`scratchpad/ca-fixture`: `HashMap` used with NO `use` → an unresolved-name diagnostic + an auto-import
quick-fix). Seeded `workspace.shell` with a `V=` editor tab so `main.rs` opens on boot; chad's real
`~/.marley` backed up + restored.

- **rust-analyzer connected** — `lsp: ready · 2 errors`, red squiggles under both `HashMap`, and **⌘K
  hover showed the real message "cannot find type `HashMap` in this scope"** (proving editor focus +
  editor-scoped cmd-chord dispatch + the LSP round trip all live).
- **⌘. → "No actions here"** — the picker did NOT open. This is **F-VAL-1**, a real bug NO unit or
  headless test could see: Marley's `initialize` advertised **no `textDocument` capability at all**, so
  per LSP 3.17 rust-analyzer returns **no `CodeAction` literals** (no `codeActionLiteralSupport`) — the
  whole feature is DEAD against a real server. Same class as F-CORR-1 (documentChanges): an honest-
  capabilities omission, invisible to a mocked wire.
- **Fix**: advertise `textDocument.codeAction` with exactly what #323 implements — `codeActionLiteral
  Support` (the kinds the picker ranks), `isPreferredSupport`, `dataSupport`, `resolveSupport.properties
  = ["edit"]`. Honest; each maps to shipped behavior. Handshake test updated (was "textDocument is
  None" → now asserts the codeAction cap is the ONE textDocument feature).
- **Re-drove after the fix → full end-to-end success** (captures `ca-11/12/13`): ⌘. opened the picker
  **"2 actions" — "Import `std::collections::HashMap`" (isPreferred, selected first) + "Qualify as
  `std::collections::HashMap`"** (REQ-001/002/003, the CARD RENDERS). **Enter** → rust-analyzer defers
  these edits via `data`, so the accept fired the **`codeAction/resolve` round trip** (REQ-004 resolve
  branch, live) → the resolved WorkspaceEdit applied through the **#322 engine** (REQ-005) → **`use
  std::collections::HashMap;` inserted at the top**, flash **"Code action: 1 file changed"**, and the
  squiggles CLEARED. The one path the headless lane can't prove (pixels + a real server's real
  actions + the live resolve chain) is now proven.

New prevention rule: **`PR-claude-lsp-advertise-client-capability-or-the-server-withholds-the-feature`**
— a result-reading feature still needs its CLIENT capability advertised when the SERVER gates its reply
on it (codeAction literals, and the F-CORR-1 documentChanges sibling); a mocked-wire test never catches
it — drive a real server. Failure recorded: F-VAL-1.

### Gate — GREEN [diff]
`scripts/gates.sh --diff` → **`GATE GREEN [diff]`, 15 passed / 0 failed** (attempt 1; the flaky real-PTY
`workspace_two_real_sessions_are_independent` coverage-phase deadlock did NOT recur — a stall-aware
watchdog was on standby). gate:4 coverage **100% lines** (the severity-arm fill closed the only red);
gate:5 mutation **MSI 100%**; miri + visual green. Receipt written for `/commit`. One extra loop this
phase: the handshake fix (F-VAL-1) changed `.rs`, so the gate + release bundle were rebuilt and re-run.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **Docs (§21)**: CHANGELOG `[Unreleased]/Added` entry for #323 (the picker + resolve + the #322 apply
  reuse + the handshake capability); `docs/marley_architecture/editor.md` (the #323 SHIPS passage +
  the capability lesson) + `crate-map.md` (the `code_action.rs` seam + the SHIM seam-list) updated.
- **Knowledge (forge wired §19)**: AAR `cbce96e2…` submitted (outcome=completed, effectiveness 5, 2
  novel findings). Failure `BF-lsp-codeaction-inert-without-client-capability-001` (high — the whole
  feature was dead against a real server; caught ONLY by the live drive). Prevention rule
  `PR-claude-lsp-advertise-client-capability-001` (advertise the client capability a server gates its
  reply on; prove with a live drive, not the fake wire). forge #323 closed (status=done).
- **Lesson**: this ticket is the case study for the ticket's OWN stated risk —
  `PR-claude-synthetic-response-tests-never-prove-the-live-wire`. 18 green unit/headless tests + a
  green gate, and the feature was still inert against rust-analyzer. The live drive (unlocked screen,
  real server, seeded editor tab) was decisive AND cheap once the `V=` editor-leaf seed retired the
  fragile file-nav. Two capability omissions of the same class now fixed (this + #322's documentChanges).
- Ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.

status: Phase 5 — Complete PASS
