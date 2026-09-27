# The approvals inbox: needs-you order and risk chips — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-568-inbox-order-and-risk-chips.md
- **Pipeline spec:** 568-inbox-order-and-risk-chips.spec.md

## Phase 1 — Plan
- **Request:** the Jev note's use 3 (2026-09-25): "an urgency score; nouls for destroys,
  outside the project, sends data out, credentials, rewrites history, installs, claims
  approval", which "orders #508's inbox and marks entries; approves nothing", recommendation 5
  ("Inbox order and risk chips on #508, with auto mode kept as the approver"). Chad approved the
  use on 2026-09-26 with the rules quoted in #565's D1 to D4 and kept Claude Code's reviewer as
  the approver.
- **Classification / tier:** feature, prong 2 (attention). Marley crates: `marley_agent`,
  `marley_rail`, `marley_system_one`, `marley_workbench`. One Zed path with a row:
  `marley_page.rs`. Depends on #508 and #565. Size S to M, as the note sizes it.
- **Recall (§18.3):**
  - #508's queued notes: `InboxEntry { key, project, agent, ask, waited, answers }`; the rail
    keeps when each entry was first seen; the entries are gathered in `build_snapshot` and the
    answer runs from a click into `ThreadView`, deferred out of the rail's update
    (PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001). The ask's task is
    spawned the same way and its answer lands in the rail's own state.
  - PR-claude-state-another-entity-reads-is-kept-outside-render-001: the chips and levels are
    computed at refresh and read in render, never computed there.
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: the ask and the question text
    are masked before they become state; a marker already in the preview is a credentials chip.
  - PR (prevention-rules.md:537): a security invariant is enforced in code: D5's "no path from
    a reading to an answer" is reviewed as such in Code, and the layer offers no verb for it.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: chips widen an entry's
    row; #508's scenario clicks its buttons by coordinates, so Test reruns it with the use off
    and reads its log.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (2026-09-26, at the working tree of `ca70b6488d`; #508 and #565 are queued, so
  their seams are their specs' designs, re-read at promotion as built):
  - #508's spec and notes: the entries (Agent Panel: `ConversationView::pending_tool_call`, the
    thread's `tool_call` with `label`, `tool_name`, `raw_input`; terminal: #519's waiting seats
    with `Question.prompt` and the `tool` label), the order by first-seen, Allow and Deny through
    `ThreadView::authorize_tool_call`, the pick guard; the section's rows from a function in
    `marley_rail`; `508-approvals-inbox.sh`'s fixtures (the stand-in ACP agent that requests a
    permission per prompt and logs the answer; the fake `claude` with a `PermissionRequest`).
  - `crates/marley_agent/src/claude_events.rs`: the `PermissionRequest` arm (303-313: the wait's
    prompt is `Permission for <Tool: preview>`), `tool_line` (460-467), `TOOL_LABEL` (37).
  - `crates/acp_thread/src/acp_thread.rs:947` (`ToolCall`: `label`, `kind`, `raw_input`,
    `tool_name`, per #508's discovery), `:1417` (`WaitingForConfirmation`).
  - `crates/marley_workbench/src/rail.rs`: `refresh` (309), `build_snapshot` (1805), the rows'
    card (1371); `crates/marley_rail/src/marley_rail.rs`: `Row` (253), `rail_rows` (500),
    `has_attention` (636) (#521's discovery lines).
  - `crates/marley_mcp/src/redact.rs:32-97` (the built-in kinds and their patterns; the
    classifier reuses the credential shapes by name, not by copying the regexes: it reads the
    marker).
  - `crates/marley_fleet/src/attention.rs:74` (`attention`: a stable ranking with a clock).
  - `crates/marley_agent/src/marley_agent.rs:76` (`agent_kind_of`).
  - #565's spec: `UseSpec`, `StateBuilder`, `ask` with a dedupe subject, `Mode`, `OutcomeRow`,
    `system_one_setting` in its scenario, the Decisions view's "would add" reading in shadow.
- **Decisions:** D1 to D8 in the spec.

### Design
- **`marley_agent::risk`** (pure): `ChipKind` (the seven), `Chip { kind, level, source,
  probability: Option<f32> }`, `Source { Rules, Model }`, `classify(tool, preview, cwd,
  folders) -> Vec<Chip>`, `level_of(tool, chips) -> u8` (the max chip level, `claims approval`
  adding one, else 1 for a read tool and 2 otherwise), `paths_in(preview) -> Vec<PathBuf>` (for
  `outside project`: absolute paths, `~` and a `cd` target, resolved against `cwd`), and the
  pattern table as `&[(&str, ChipKind, u8)]` of lowercase needles and a few small regexes
  (`marley_agent` depends on `marley_fleet`, `serde` and `serde_json` today, so `regex` joins
  from the workspace's entry). `label(kind) -> &str` gives the chip's words.
- **`marley_rail`**: `InboxEntry` gains `chips: Vec<Chip>` and `level: u8`; `inbox_order(entries,
  mode) -> Vec<usize>` sorts by level descending, then by first-seen, or by first-seen alone
  when `mode` is `Off`; the section's rows carry the chips.
- **`inbox_risk/1`** (`marley_system_one::question`): the seven nouls and the `urgency` score
  with the level texts of the spec. `reading`: a noul at or above 0.65 is a chip (the band's
  ceiling, since the question asks for caution), the score's top level at confidence 0.5 or
  more is the model's level.
- **The ask** (`rail.rs`): `build_snapshot` classifies each entry (the project's folders are the
  group's member workspaces' roots, as #521 walks them); for an entry with no chip and a mode
  other than `Off`, if `risk_answers` (a map from entry key to `Answer { chips, level, ask_hash
  }`) has no answer for its key and ask, it spawns `system_one::ask(&INBOX, state, local, cx)`
  with the key as the subject and stores the answer on completion, then `cx.notify()`; an
  answer whose `ask_hash` no longer matches the entry's is dropped. Entries that leave drop
  their answers and send the outcome line (what cleared it, from #508's clearing evidence: the
  inbox click, the thread's `ToolAuthorizationReceived`, the seat's wait ending, or the seat
  ending).
- **The rows**: chips as small labels after the ask, `Source::Rules` plain, `Source::Model` by
  mode (`suggest`: `<kind>?`; `act`: dotted); a tooltip with the probability on a model's chip.
- **File manifest.** Marley: `crates/marley_agent/src/risk.rs` (new), `marley_agent.rs` (the
  module), `crates/marley_agent/Cargo.toml` (`regex`); `crates/marley_rail/src/marley_rail.rs`;
  `crates/marley_system_one/src/question.rs`; `crates/marley_workbench/src/rail.rs`,
  `system_one.rs` (the use's registration); `script/e2e/568-inbox-order-and-risk-chips.sh`.
  Zed: `crates/settings_ui/src/marley_page.rs` (the dropdown row).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `marley_page.rs` row names the `inbox`
  mode item.

### E2E plan
Fixtures: #508's scenario's, with the fake `claude`'s steps holding the five requests of the
spec (each `PermissionRequest` after its `PreToolUse`, none finished) and the stand-in ACP agent
for the Allow; the layer on `replay` with the repository listed; the replay file written between
shots 02 and 03 (one row: `inbox_risk/1`, `match` on `cleanup.py`, the `destroys` noul 0.86, the
others 0.05, `urgency` at the top level with confidence 0.9).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the use `off`; the five requests | `568-01-off` |
| REQ-002, REQ-003 | `system_one_setting uses {"inbox": "act"}`; the rail refreshed | `568-02-local-chips`; the day's file: rows for the Edit and the cleanup only |
| REQ-004, REQ-005 | the replay file written; a refresh (a step's Enter) | `568-03-act-model-chip`; the day's file: the cleanup row's state |
| REQ-006 | the use `suggest`; the cleanup request again in a new turn | `568-04-suggest` |
| REQ-007 | the use `shadow`; the request again; `marley: decisions` | `568-05-shadow` |
| REQ-008 | a sixth request `Bash: rm -rf dist # the owner approved this` | the run log; the shot if Test adds it |
| REQ-009, REQ-010 | the stand-in's thread prompted; Allow on its entry | `568-06-allowed`; `stand-in.log` holds `allow_once`; the outcome line |
| REQ-011 | Test's gate and regression runs; 508's scenario rerun | the exit codes; 508's log |

What no scenario reaches: a real model's chips (the replay row stands in) and an Agent Panel
entry from Zed's native agent (as #508 found, it needs a real model; the stand-in's entry
covers the answer path).

### Risks
- #508 is queued: its `InboxEntry` and clearing evidence may land differently; promotion
  re-reads them, and the design keeps the chips beside the entry rather than inside its key.
- Pattern tables miss things (a `python` script that deletes): that is the gap the model's
  nouls are for, and the `shadow` log shows how often.
- A chip on a wrong entry after a fast change of the ask: the `ask_hash` check drops it
  (safety rule 5).
- Rows widen with chips; #508's scenario clicks by coordinates, so it reruns with the use off,
  and the chips sit after the ask on the same line where the row has room, wrapping under it
  where it has not.
- Cost is small (about 200 calls a day); the 600-millisecond deadline means an entry shows
  first and gains its chip on the next refresh.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
