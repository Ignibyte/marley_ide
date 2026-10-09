---
pipeline_id: bbdb06f5-ba14-44a1-9602-d436e23f75a2
ticket: docs/planning/tickets/open/TICKET-723-docs-search-over-all-of-marleys-docs.md
status: Phase 4 — Complete PASS
title: Docs search over all of Marley's docs
type: feature
slice: Marley's MCP server, #681's docs tools
references:
  - docs/planning/pipeline/completed/681-docs-and-settings-tools.spec.md
---

## Title
One docs search for agents over Zed's docs and all of Marley's user docs.

## Scope
### In
- **The docs bundle** adds `marley/walkthrough.md`.
- **A second bundle** (repo root, `include = ["CHANGELOG.md"]`) serves `marley/CHANGELOG.md`.
- **`split_page` for the changelog:** each top-level bullet starts a section; its heading is
  the bullet's bold title, else its first words.
- **Query word stems:** a word ending in -ies, -es, -s, -ing or -ed is also tried without the
  ending, if what is left has at least three letters. The text matching stays a substring match.
- **The text around the tools:** the registry's descriptions of `docs_search` and `docs_read`,
  the Marley agent's instructions (the changelog for "what changed"), and `guide.md`'s docs-tools
  lines.

### Out (explicitly deferred)
- Ranking by meaning (embeddings; Rusty's brain search covers that ground).
- The design and planning docs: they are for whoever builds Marley, not for its users or its
  agents.

## Reference (§20)
Upstream Zed's docs (`docs/src`, mdBook) are already served (#681). The rest is Marley-specific.

### Prior art
#681's `docs_tools.rs` (`fs_embed!`, `split_page`, word scoring) and `util::fs_embed` (rust-embed in
release, files from disk in debug).

## UI proof
`script/e2e/723-docs-search-over-all-of-marleys-docs.sh`, under `compositor sway`. #704's scripted
MCP client (`e2e-agent`) calls:
- `docs_search "kill switch agent control"` → a `marley/CHANGELOG.md` hit whose heading names
  agent control (REQ-001);
- `docs_search "walkthrough tour"` or a walkthrough-only phrase → a `marley/walkthrough.md` hit
  (REQ-001);
- `docs_search "terminals fonts"` → a hit for a heading or text that has "terminal" and "font" only
  in the singular (REQ-002);
- `docs_read marley/CHANGELOG.md` with the hit's heading → the entry's text (REQ-003).

`723-01-marley`, a shot of the run's Marley, records the tools answering from it.

## Locked-In Decisions
- **D1:** the changelog is split per entry, since its `###` headings hold hundreds of entries.
- **D2:** stems are tried beside the word, never instead of it, so an exact word still scores.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `docs_search`, the system shall search Zed's docs and Marley's guide, walkthrough and changelog, a changelog entry being a section of its own. | The client's replies |
| REQ-002 | WHEN a query word ends in a plural or verb ending, the system shall also match it without the ending. | The client's reply |
| REQ-003 | WHEN an agent calls `docs_read` on `marley/CHANGELOG.md` with an entry's heading, the system shall answer that entry. | The client's reply |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan.**
- **P2 Code:** `docs_tools.rs`, the registry's descriptions, the instructions, the guide; gate.
- **P3 Test:** the scenario.
- **P4 Complete.**
