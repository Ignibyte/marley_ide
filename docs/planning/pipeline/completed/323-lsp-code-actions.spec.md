---
pipeline_id: 5cf6eeb7-244f-4bc9-bdf9-18dc6a4021de
ticket: forge#323 (056e3f20-fa74-4d16-9d3e-6631daf27dda) · local docs/planning/tickets/open/TICKET-323-lsp-code-actions.md
aar_id: cbce96e2-8719-472c-8073-b5c01645fe0f
status: Phase 5 — Complete PASS
title: LSP code actions + quick fix (⌘.) — the picker + codeAction/resolve, applying via the #322 engine
type: feature
milestone: M21
references: [forge#322, forge#310, forge#311, forge#312, forge#313]
---

## Title
⌘. at the caret offers the server's fixes/assists (auto-import, fill-match-arms, extract, qualify-path…)
in a picker; accepting applies the fix through the #322 WorkspaceEdit engine. Where auto-import lands.

## Scope
### In
- **⌘.** (Editor-scoped) → `textDocument/codeAction` with the caret's range + `context.diagnostics` =
  the diagnostics overlapping the caret row (from the #310 store via `diagnostics_for` + `diagnostic_at_row`)
  + `triggerKind: 1` (Invoked).
- **Parse (PURE, marley_lsp)**: `(Command | CodeAction)[]` → keep CodeActions, normalize `{title, kind,
  is_preferred, disabled, edit: Option, data: Option}`; a Command-only / disabled / malformed element is
  SKIPPED (per-element `filter_map`); the Command-only count is surfaced ("N unavailable").
- **Ordering (PURE)**: quickfix-kind rows first, then refactor/source; `is_preferred` pinned to the top.
- **The picker** (the #312 `DefPicker` recipe + the #221 card): ↑/↓, Enter accepts, Esc closes;
  registered at every overlay choke point (`text_input_blocked`, ⌘⇧A clear).
- **Accept**: if the action's `edit` is absent AND `codeActionProvider.resolveProvider`, send
  `codeAction/resolve` (the raw action back), then apply the resolved `edit`; else apply directly.
- **Apply = the #322 engine, UNCHANGED**: `parse_workspace_edit(edit)` → `apply_workspace_edit(root,
  files)`. No second applier — the convergence is the point (a `ResourceOpsUnsupported` edit flashes).
- Capability gate off raw `server_caps["codeActionProvider"]`; not-ready / no-provider / empty → a quiet
  flash ("No actions here"), no picker.
- `RequestPurpose::{CodeAction, CodeActionResolve}` on the #311 recipe + the ONE drain.
- Stale guards inherited whole: key by (uri, version, caret); the live-identity pump poll dismisses on
  caret move / edit / file switch; a resolve answer for a dismissed picker is dropped.

### Out (explicitly deferred)
- **Bare Command execution** (`workspace/executeCommand`) — Command-only actions are skipped + counted;
  executing them is a named follow-up (rust-analyzer serves its edits via CodeAction+resolve).
- **The gutter lightbulb glyph** — keyboard-first (⌘.); the lightbulb affordance is a follow-up.
- **`source.organizeImports`-on-save** — composes with #314's FormatOnSave later.
- **A selection RANGE** — v1 uses the caret's single-point range; a codeAction over a selection is a
  follow-up.

## Reference (§20)
**Zed / VS Code (the editor)** — the observed behavior: ⌘. at the caret opens a menu of the server's
quick-fixes + refactors (auto-import, fill match arms, extract…), preferred-fix-first, and accepting
applies the fix. Marley matches that BEHAVIOR via the published LSP 3.17 wire (`textDocument/codeAction`,
`codeAction/resolve`, `CodeAction`/`Command`/`CodeActionContext`/`CodeActionKind`) plus Marley's OWN
picker, the #310 diagnostics store, and the #322 WorkspaceEdit engine. The menu + preferred-first are
the spec's own (`isPreferred`, `kind` hierarchy), not read from anyone's source. No GPL source read —
clean-room, confirmed at design.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — apply via the #322 engine unchanged** (`parse_workspace_edit` + `apply_workspace_edit`). The
  reason #322 was the foundation; #323 adds NO apply code.
- **D2 — Command-only actions OUT v1** (skip + count "N unavailable"). rust-analyzer serves edits via
  CodeAction+resolve; `workspace/executeCommand` is a named follow-up.
- **D3 — hand-parse** (no `lsp-types` beyond the shipped Diagnostic seam) — the #312/#313/#322 posture.
- **D4 — `needs_resolve = edit.is_none() && resolveProvider`**; resolve sends the raw CodeAction JSON
  back (the server fills `edit` from the action's `data`).
- **D5 — the picker mirrors `DefPicker`** (the shipped #312 recipe) + the #221 card; registered at every
  overlay choke point (`PR-claude-new-overlay-register-at-every-choke-point-001`).
- **D6 — inherit the request guards WHOLE** (`PR-claude-second-consumer-must-inherit-the-first-
  consumers-guards`): key (uri, version, caret); the live-identity poll dismisses; a stale/dismissed
  resolve answer is dropped.
- **D7 — the codeAction RANGE = the caret's single-point range** v1 (start == end == caret); a selection
  range is a follow-up.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘. is pressed at the caret AND the server advertises `codeActionProvider`, the system shall send `textDocument/codeAction` carrying the caret range and the diagnostics overlapping the caret row in `context.diagnostics`. | LIVE tee PAYLOAD (the range + the diagnostic, not the method name) |
| REQ-002 | The parser shall keep `CodeAction` entries (normalized) and SKIP a Command-only / disabled / malformed element, surfacing the Command-only count. | pure unit cov/MSI 100 |
| REQ-003 | The system shall order quickfix-kind actions before refactor/source, with `isPreferred` actions pinned to the top. | pure unit |
| REQ-004 | WHEN an accepted action has no `edit` AND the server advertises `resolveProvider`, the system shall send `codeAction/resolve` and apply the resolved edit; otherwise it shall apply the action's edit directly. | pure unit (needs_resolve) + headless |
| REQ-005 | WHEN a code action's WorkspaceEdit is applied, the system shall apply it through the #322 engine (`parse_workspace_edit` + `apply_workspace_edit`), never a second applier. | headless + review |
| REQ-006 | WHEN no actions are offered / no provider / not ready, the system shall show a quiet flash and open no picker. | headless + review |
| REQ-007 | WHEN the caret moves, the buffer edits, or the file switches while the picker is open, the system shall dismiss it; a resolve answer for a dismissed picker shall be dropped. | headless |
| REQ-008 | The codeAction request key (uri, version, caret) shall drop a response whose key no longer matches the live request. | pure unit + headless |

## Phase Plan
- **P2 Design** — the pure layer (`code_action.rs`: `CodeAction`, `parse_code_actions`, ordering,
  `needs_resolve`, `code_action_params`/resolve params, `code_action_support`) + the app pure
  (`editor_code_action.rs`: `CodeActionKey`, the `CodeActionMenu` picker) + the shim (request/parse/
  picker/accept/resolve/apply-via-#322/the 2 drain arms/⌘. keymap/overlay). The mutation surface. §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the parse (Command-vs-CodeAction discrimination) + the
  resolve-needed decision + the guard inheritance get the hardest look.
- **P4 Validate** — tests + gate green [diff]; the LIVE tee drive (delete a `use` → squiggle → ⌘. →
  "import" → accept → the use line is BACK on disk; assert the real codeAction frame's `context.
  diagnostics`).
- **P5 Complete** — CHANGELOG + crate-map + editor.md; AAR; archive; close #323.
