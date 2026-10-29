---
pipeline_id: 5b6a0289-eb05-4ea3-af34-e4e672b18a9f
ticket: docs/planning/tickets/open/TICKET-947-pairing-card-as-the-board.md
status: Phase 4 — Complete PASS
title: The pairing card as the AgentAccess board draws it
type: feature
slice: The Marley look, Wave 6 (AgentAccess)
references: [docs/marley/design-review-2026-10-23/F-tools.md]
---

## Title
An outside program's pairing question (#811) is a sentence with the code inside it and four
buttons. The AgentAccess board draws it as a card: the name in the title, where it asks from and
whether over SSH, the code in six digit boxes, the three grants as a radio list with what each
allows, Refuse and Pair, and the two-minute line. This draws it so.

## Scope
### In
- `marley-mcp-bridge` sends `Marley-Via: ssh` when it runs under SSH (`SSH_CONNECTION` set).
- `marley_mcp::Caller` gains `reach: Reach` (`Local` or `Ssh`), read from that header.
- `asks::Ask` gains a code drawn as digit boxes and a radio list whose pick a button reads.
- `clients::show_pairing` draws the board's card; Pair lets the client in with the grant picked.

### Out (explicitly deferred)
- Outside Clients' power words (Read / Read and Write / Read, Write and Run): a separate board
  finding, not in #947's ticket.
- Proving the SSH claim: the header is a label, like `Marley-Cwd`; the bearer still gates the call.

## Reference (§20)
N/A — Marley-specific: Marley's own pairing of outside MCP clients (#811). The look follows
Marley's AgentAccess board in the design review of 2026-10-23
(`docs/marley/design-review-2026-10-23/F-tools.md`, "Pairing question").

### Prior art
- Zed's `ui` crate has `ToggleButton`s and checkboxes, no radio group; a radio here is a row with
  a filled or empty dot, as Marley's own lists draw a choice.
- `asks::Ask` (#871, #938) is the card every agent question draws; the code and the radio list go
  in it, so the pairing stays in Zed's notification frame.
- `marley_mcp::transport` already reads `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` into
  `Caller` (#520); `Marley-Via` is read the same way.
- `util::paths::PathExt::compact` shortens the home folder to `~`, as the board's "~/work".

## UI proof
`script/e2e/947-pairing-card-as-the-board.sh` (sway):
- `947-01-local`: `marley-mcp-bridge pair laptop` run outside Marley: the card with
  "“laptop” asks to pair with Marley", "From <folder>. Check the code matches what the command
  printed.", six digit boxes holding the printed code, the three grants with Read picked, Refuse
  and Pair, the two-minute line (REQ-001, REQ-002, REQ-004).
- `947-02-picked`: Read and Write clicked: its dot filled, Read's empty (REQ-003).
- After Pair: the command prints "Paired: laptop may read and write." (REQ-003).
- `947-03-ssh`: the same command with `SSH_CONNECTION` set, as `laptop2`: "… over SSH." (REQ-002).
- `947-04-day`: the card in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Read is picked when the card opens: the least power, as Read was the primary button.
- D2 — `Reach` is an enum, not a `bool`, read from a header the bridge sets from `SSH_CONNECTION`;
  it changes words only, never what a client may do.
- D3 — The radio's pick lives in the card (`Rc<Cell<Power>>`) and redraws the window; Pair reads
  it once. Refuse and an expired wait are as before.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an outside program asks to pair, the card shall show "“<name>” asks to pair with Marley", the code in six digit boxes, the three grants each with its line, Refuse and Pair, and "No answer in two minutes pairs nothing.", in Marley Night and Marley Day. | `947-01-local`, `947-04-day` |
| REQ-002 | The card shall say "From <folder> over SSH." only for a bridge run under SSH, and "From <folder>." otherwise. | `947-01-local`, `947-03-ssh` |
| REQ-003 | WHEN the user picks a grant and presses Pair, Marley shall let the client in with that grant. | `947-02-picked`, the command's output |
| REQ-004 | WHEN the card opens, Read shall be picked. | `947-01-local` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the header, `Reach`, `Ask`'s code and radio, the card; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
