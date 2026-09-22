---
pipeline_id: ad5c8ba6-8722-4675-a8d5-716fb9ce6db9
ticket: forge#324 (aa432c80-2bf4-40f9-acb5-e02f99416208) · local docs/planning/tickets/open/TICKET-324-lsp-signature-help.md
aar_id: ffda3cbe-8baa-49fc-8ea1-c5dbc5661fe4
status: Phase 5 — Complete PASS
title: LSP signature help (parameter hints) — a card above the caret while you type a call
type: feature
milestone: M21
references: [forge#324, forge#313, forge#311, forge#323]
---

## Title
Type `foo(` and a card appears ABOVE the caret showing the call's signature with the ACTIVE parameter
lit; type args + `)` and it's gone. The third leg of the as-you-type triad (completions #313 / signature
help / inlay hints #331) — the #313 park-and-consume shape with a card instead of a menu.

## Scope
### In
- **Trigger** off `signatureHelpProvider.triggerCharacters` (rust-analyzer: `(` `,`) + `retriggerCharacters`,
  read off raw `server_caps` by a `signature_trigger_characters` sibling of `completion::trigger_characters`.
  The typing hook PARKS the request, the pump consumes next tick — the #313 idiom VERBATIM, including the
  **string/comment suppression gate** (a `(` in a comment asks nothing). **⌘⇧Space** invokes manually
  (`context.triggerKind` = 1 Invoked vs 2 TriggerCharacter; `isRetrigger` on a `,`).
- **Parse (PURE, `marley_lsp::signature_help`)**: `SignatureHelp{signatures[], activeSignature?,
  activeParameter?}` → **clamp BOTH indices into range** (server off-by-N is routine — clamp, never
  index-panic); a per-signature `activeParameter` **overrides** the top-level one (3.16+). A
  `ParameterInformation.label` arrives as a **string** OR as **`[start,end]` UTF-16 offsets into the
  signature label** — normalize BOTH to a highlight range over the label, **dropping** (not panicking on)
  an out-of-bounds / inverted pair.
- **Card render** (the #221 anchored card): positioned ABOVE the caret row (a prefer-above variant of the
  #313 `popup_origin` flip math) so it never covers the arguments being typed; the active signature line
  monospace with the active-parameter run in `accent` (the ONE highlight, reusing the token-color seam); a
  **"N of M" pip** when multiple signatures, **↑/↓ cycles** (wrapping, the `complete.rs` idiom).
- **Dismissal = the live-identity pump poll** (not the typing hooks): the caret leaves the call's argument
  region / the file switches / Esc. A `,` **retrigger** re-requests (the active parameter is server-computed).
- **Layering vs the completion popup**: both may be live at once (`s.push(|` → completion for the arg +
  signature for the call). v1: completion owns BELOW the caret (#313 flips below-first), signature owns
  ABOVE (this flips above-first) → they never overlap.
- **Stale guard** inherited whole: `SignatureKey (uri, version, caret)` — the #313 `CompletionKey` shape; a
  late answer for a moved caret / edited buffer is dropped.
- `RequestPurpose::SignatureHelp` on the #311 recipe + the ONE drain (`consume_lsp_responses`).
- **Handshake advertises `textDocument.signatureHelp`** (`signatureInformation.parameterInformation.
  labelOffsetSupport: true` + `signatureInformation.activeParameterSupport: true` + `contextSupport: true`)
  — the #323 lesson (`PR-claude-lsp-advertise-client-capability-001`): without `labelOffsetSupport` rust-
  analyzer sends string labels only (or degrades), and without `activeParameterSupport` the per-signature
  active param is withheld. Honest — #324 implements all three.

### Out (explicitly deferred)
- **The signature/parameter `documentation` body** — v1 shows the signature LINE + the active-param
  highlight + the pip only; rendering the markdown doc block (the #311 card already can) is a named follow-up.
- **A fuzzy signature picker** — multiple overloads cycle with ↑/↓ + the pip, no filter box.
- **Nested-call re-anchoring beyond the server's `activeParameter`** — we trust the server's computed index
  (retrigger on `,`), not a client-side paren-depth parser.

## Reference (§20)
**Zed / VS Code (the editor)** — the observed behavior: while typing a call, a small card shows the
signature with the current parameter emphasized, updating as you type commas, and vanishing when the call
closes. Marley matches that BEHAVIOR via the published **LSP 3.17 wire** (`textDocument/signatureHelp`,
`SignatureHelp`/`SignatureInformation`/`ParameterInformation` with string-or-offset labels,
`SignatureHelpContext`/`triggerKind`/`isRetrigger`) plus Marley's OWN card (the #221 frame, the #313
`popup_origin` flip, the #311 overlay recipe) and the #310 suppression gate. The card, the prefer-above
placement, and the active-param highlight are Marley's own composition — no GPL source read. Clean-room,
reconfirmed at design.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — reuse the #313 shape whole**: park-on-type → pump-consume; the string/comment suppression gate
  (with a positive control, `PR-claude-suppression-test-needs-a-positive-control-001`); `SignatureKey` =
  the `CompletionKey (uri, version, caret)` stale shape; the live-identity dismiss poll
  (`PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001`); register the overlay at every
  choke point (`PR-claude-new-overlay-register-at-every-choke-point-001`).
- **D2 — prefer-ABOVE card** (a `popup_origin` variant that tries above first, flips below only when it
  won't fit up top) so the card never covers the args; completion (below-first) + signature (above-first)
  can't collide.
- **D3 — advertise `textDocument.signatureHelp`** (`labelOffsetSupport` + `activeParameterSupport` +
  `contextSupport`) — the #323 carry-forward; honest, each maps to shipped behavior. Proven by the live
  drive (the OFFSET label form actually arrives).
- **D4 — clamp both indices, never panic**; a per-signature `activeParameter` overrides the top-level.
- **D5 — parameter label as string OR `[start,end]` offsets**, both normalized to a highlight range over
  the signature label; an OOB / inverted pair is dropped (the run just isn't highlighted), never a panic.
- **D6 — `documentation` body OUT v1** (a follow-up); the signature line + active-param highlight + pip is
  the value.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `signatureHelpProvider` trigger char (`(`/`,`) or ⌘⇧Space is entered at a call site AND the server advertises `signatureHelpProvider`, the system shall send `textDocument/signatureHelp` carrying `context{triggerKind, triggerCharacter?, isRetrigger}`. | LIVE tee PAYLOAD + headless |
| REQ-002 | The parser shall clamp `activeSignature` and `activeParameter` into range (never index-panic), a per-signature `activeParameter` overriding the top-level one. | pure unit cov/MSI 100 |
| REQ-003 | The parser shall normalize a `ParameterInformation.label` given as a string OR as `[start,end]` offsets into a highlight range over the signature label, dropping an out-of-bounds / inverted pair. | pure unit |
| REQ-004 | WHEN a trigger char is typed inside a string or comment, the system shall send NO request; the same char in code shall trigger. | headless A/B (positive control) |
| REQ-005 | WHEN a signature-help response is applied, the system shall show a card ABOVE the caret row with the active parameter highlighted, plus a "N of M" pip + ↑/↓ cycling when multiple signatures. | headless + LIVE capture |
| REQ-006 | WHEN the caret leaves the call's argument region, the file switches, or Esc is pressed, the system shall dismiss the card; a `,` shall retrigger a fresh request. | headless |
| REQ-007 | The signature-help request key (uri, version, caret) shall drop a response whose key no longer matches the live request. | pure unit + headless |
| REQ-008 | The initialize handshake shall advertise `textDocument.signatureHelp` (`labelOffsetSupport` + `activeParameterSupport` + `contextSupport`) so the server returns offset labels + per-signature active params. | review + LIVE (offset-label form arrives) |

## Phase Plan
- **P2 Design** — the pure layer (`signature_help.rs`: `parse_signature_help`, the clamp + per-signature
  override, the label-string/offset normalize, `signature_trigger_characters`, `signature_help_params` +
  context, `signature_help_support`) + the app pure (`editor_signature.rs`: `SignatureKey`, the
  `SignatureCard` state — active sig/param, ↑/↓ cycle, the prefer-above origin) + the shim (park/consume,
  the card render, the drain arm, ⌘⇧Space keymap, the dismiss poll, the handshake capability). The mutation
  surface. §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the clamp/override + the label-offset normalize + the suppression
  gate + the capability advertisement get the hardest look.
- **P4 Validate** — tests + gate green [diff]; the LIVE tee drive (type `s.push_str(` → the card shows the
  signature with the param lit; the A/B comment control; confirm the OFFSET-label form arrives — REQ-008).
- **P5 Complete** — CHANGELOG + crate-map + editor.md; AAR; archive; close #324.
