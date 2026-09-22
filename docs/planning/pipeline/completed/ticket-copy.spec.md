---
pipeline_id: c76c9dfd-43f6-43a3-b8d7-e5f288c6b27f
ticket: forge#70 (657f8e58-987c-49ae-8ec0-83f7d538c983) · local docs/planning/tickets/open/TICKET-070-ticket-copy.md
aar_id: 9bade5c7-445f-41b6-a2d0-f2a8e716f0ee
status: Phase 5 — Complete PASS
title: forge ticket row → copy '#N — title'
type: feature
milestone: M2.C
references:
  - crates/marley_app/src/forge_view.rs (PURE: ticket_ref)
  - crates/marley_app/src/app.rs (SHIM: click a forge ticket row → copy to clipboard)
---

## Title
Click a ticket row in the ⌘⇧F Forge overlay to COPY a pasteable ref (`#N — title`) to the clipboard —
so you can ⌘V it into an agent pane ("go work #70"). Lightly links the work to the agents. Read-only.

## Scope
### In
- PURE (`forge_view.rs`, cov/MSI 100): `ticket_ref(row: &TicketRow) -> String` = `"#{number} — {title}"`
  (the U+2014 em-dash separator; no status/kind — a clean ref).
- SHIM (`app.rs`, mutants::skip + cov-excluded): each forge-overlay ticket row gets an
  `on_mouse_down(Left)` → `cx.write_to_clipboard(ClipboardItem::new_string(ticket_ref(&row)))` (reusing
  the #44 clipboard seam). The row's display text is unchanged.

### Out
- Inserting at the prompt (chose clipboard so you can paste into ANY agent pane). Forge WRITES. Clicking
  a fleet/agent row. A visual "copied!" toast (later).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — COPY to clipboard (reuse #44's `cx.write_to_clipboard`), not insert-at-prompt — the ref goes to
  whichever agent pane you ⌘V into.
- D2 — the ref is `#N — title` (em-dash), no status/kind — clean + pasteable.
- D3 — the ref string is captured per row + cloned per click (the listener is `Fn`).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `ticket_ref(row)` is called, it shall be `"#{number} — {title}"` (U+2014 separator). | unit |
| REQ-002 (visual) | WHEN a forge ticket row is clicked, its `#N — title` ref shall be placed on the clipboard. | self-test (⌘⇧F → click a row → `pbpaste`) |
| REQ-003 | `scripts/gates.sh` GREEN, cov/MSI 100 on ticket_ref; app shim excluded. | gate |

## Phase Plan
- **P2** — `ticket_ref` + the click-to-copy shim, mutation targets, test plan.
- **P3** — ticket_ref + the app.rs clickable row.
- **P3.5** — critic: the ref format, the per-row closure capture (right ref per row), the clipboard reuse.
- **P4** — ticket_ref unit test (cov/MSI 100) + the SELF-TEST (⌘⇧F → click → pbpaste the ref) + gate GREEN.
- **P5** — docs, AAR, archive, close #70.
