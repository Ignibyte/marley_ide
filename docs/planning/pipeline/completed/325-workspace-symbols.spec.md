---
pipeline_id: 8260ef53-4700-47ea-bb4f-ee2cdb8eb448
ticket: forge#325 (e3000b55-3787-4d70-8470-156fa935d128) · local docs/planning/tickets/open/TICKET-325-workspace-symbols.md
aar_id: d18ca6f9-f7e7-439b-8b7d-47ed4b8bd232
status: Phase 5 — Complete PASS
title: Workspace symbols (⌘T) — jump to any symbol in the project by name
type: feature
milestone: M21
references: [forge#325, forge#304, forge#312, forge#313, forge#317]
---

## Title
⌘T opens a query picker; type `parse_comp` and land on `parse_completion_result` in another crate. The
CROSS-FILE sibling of #304's in-file symbols (⌘⇧O), riding `workspace/symbol` — server-filtered, so each
keystroke re-queries and rust-analyzer fuzzy-ranks.

## Scope
### In
- **⌘T** opens a query picker (the `finder.rs` modal recipe — `FinderState`'s query+selected idiom + the
  #221 card; owns the keyboard: ↑/↓, Enter, Esc). **Server-filtered**: each keystroke PARKS a request
  (debounced one pump tick, the #313 idiom) → `workspace/symbol {query}` to EVERY Ready host; the response
  REPLACES the list. A stale answer for an older query string is DROPPED (the stale key = the query, the
  #313 stale-guard shape keyed on the query rather than the caret).
- **Parse (PURE, `marley_lsp::workspace_symbol`)**: `(SymbolInformation | WorkspaceSymbol)[]` — BOTH shapes
  are real (a `WorkspaceSymbol`'s `location` may omit `range` when the server defers it → treat missing
  range as line 0 v1; `workspaceSymbol/resolve` is a named follow-up). Normalize to `{name, kind,
  container, path, line, col}` via `path_from_file_uri` + a per-element `filter_map` (the #312 posture — a
  malformed element degrades to a shorter list, never a panic).
- **Rows**: `kind_glyph` (reuse from #313/#304, extend the table for the `SymbolKind` numbers it lacks) +
  name + a muted `container` + a right-aligned `path:line` via the shipped `truncate_cols`. The list is
  CAPPED (~64) with an honest "+N more" tail (the #317 rule).
- **Enter** = `open_and_place_caret` (#312's verify-the-landing loader, guards inherited WHOLE — a failed
  open flashes and moves NOTHING) + a NavStack push so `⌃-` returns. Esc closes. Registered at every
  overlay choke point.
- **Position mapping**: the answer's line/col are LSP-encoded → map through `lsp_position_for`'s inverse at
  OPEN time exactly as #312 does (an emoji before the symbol must not shift the landing).
- **Capability gate** off `server_caps["workspaceSymbolProvider"]`; no host / not ready / no provider → a
  quiet flash, no picker. **Multi-root**: query EVERY Ready host (one per workspace root in `App.lsp_
  hosts`), merge the results — a two-project workspace searches both.
- **Handshake** advertises `workspace.symbol.symbolKind.valueSet` — the #323/#324 capability posture: it
  UPGRADES the reply (the full `SymbolKind` set the client understands) rather than ENABLING the feature
  (rust-analyzer serves `workspace/symbol` without it, the signatureHelp posture, NOT codeAction's
  dead-without-it). Honest — we render every advertised kind's glyph.

### Out (explicitly deferred)
- **Symbol-kind filter chips** (functions-only, etc.) — v1 shows every kind.
- **`workspaceSymbol/resolve`** — a missing WorkspaceSymbol range stays line 0; resolving it is a follow-up.
- **Client-side fuzzy re-rank** — the server ranks; we PRESERVE its order (the #313 empty-query reason).

## Reference (§20)
**Zed / VS Code (the editor)** — the observed behavior: ⌘T opens a name query and lists matching symbols
from anywhere in the project (fuzzy, server-ranked), Enter jumps to the definition. Marley matches that
BEHAVIOR via the published **LSP 3.17 wire** (`workspace/symbol`, `SymbolInformation` / `WorkspaceSymbol`
with an optional-range `location`, `SymbolKind`, `WorkspaceSymbolClientCapabilities.symbolKind`) plus
Marley's OWN finder picker (`finder.rs`), the #304 row render, and the #312 navigation (open + NavStack +
encoding). The picker, the cap/tail, and the multi-root merge are Marley's own composition — no GPL source
read. Clean-room, reconfirmed at design.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the `finder.rs` MODAL recipe** (FinderState query+selected; owns the keyboard; Esc/Enter) + the
  #221 card + the #304 row render. A modal (unlike #324's passive card) — the finder/def_picker rules.
- **D2 — server-filtered, debounced one pump tick** (#313 park); the response REPLACES the list; the stale
  key is the QUERY STRING (an answer for an older query is dropped).
- **D3 — parse BOTH shapes**; a missing `WorkspaceSymbol` range → line 0 (resolve is a follow-up); a
  malformed element is skipped (`filter_map`, the #312 posture).
- **D4 — Enter reuses #312 `open_and_place_caret` + NavStack + the encoding-aware landing WHOLE** (guard
  inheritance — `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`).
- **D5 — multi-root**: query every Ready host, merge the results (a workspace with two projects searches
  both).
- **D6 — advertise `workspace.symbol.symbolKind`** — UPGRADE, not enable (rust-analyzer serves without it).
  Honest; each advertised kind renders a glyph.
- **D7 — v1 cuts**: no kind-filter chips, no `workspaceSymbol/resolve`, no client-side re-rank.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘T is pressed AND ≥1 Ready host advertises `workspaceSymbolProvider`, the system shall open a query picker and, per keystroke (debounced one pump tick), send `workspace/symbol` carrying the typed query to every Ready host. | LIVE tee PAYLOAD + headless |
| REQ-002 | The parser shall normalize BOTH `SymbolInformation` and `WorkspaceSymbol` into `{name, kind, container, path, line, col}`, treating a missing range as line 0, and SKIP a malformed element. | pure unit cov/MSI 100 |
| REQ-003 | A `workspace/symbol` response shall REPLACE the picker list; an answer whose query key ≠ the live query shall be dropped. | pure unit (key) + headless |
| REQ-004 | The picker shall render each row as glyph + name + muted container + right-aligned `path:line`, capping the list at ~64 with a "+N more" tail. | pure unit + headless |
| REQ-005 | WHEN a row is accepted (Enter), the system shall open the file and place the caret at the symbol (encoding-aware), pushing the prior location to the NavStack; a failed open shall flash and move nothing. | headless + review |
| REQ-006 | WHEN no host / not ready / no `workspaceSymbolProvider`, the system shall show a quiet flash and open no picker. | headless |
| REQ-007 | WHEN multiple Ready hosts exist, the system shall query every one and merge the results. | headless (2-host) + review |
| REQ-008 | The initialize handshake shall advertise `workspace.symbol.symbolKind.valueSet` so the server returns the full kind set. | review + LIVE |

## Phase Plan
- **P2 Design** — the pure layer (`workspace_symbol.rs`: `parse_workspace_symbols` for both shapes +
  missing-range, `WorkspaceSymbolResult{name,kind,container,path,line,col}`, `workspace_symbol_params`,
  `workspace_symbol_support`, the `kind_glyph` extension, the cap/tail helper) + the app pure (a
  `SymbolQuery` stale key + the picker state, reusing `FinderState`) + the shim (⌘T, park/debounce/consume,
  the multi-root fan-out + merge, the drain arm, Enter→#312 open, the row render, the handshake cap). The
  mutation surface. §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the two-shape parse + the stale-query guard + the multi-root
  merge + the guard inheritance get the hardest look.
- **P4 Validate** — tests + gate green [diff]; the LIVE tee drive (⌘T `main` → a row for `main` in
  src/main.rs → Enter lands centered, ⌃- returns; assert the real `workspace/symbol` frame carries the
  typed query).
- **P5 Complete** — CHANGELOG + crate-map + editor.md; AAR; archive; close #325.
